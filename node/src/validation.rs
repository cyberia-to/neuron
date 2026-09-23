//! Authenticate native application publications at the graph adapter boundary.
use crate::{Graph, authority::verify_evidence};
use neuron_engine::{Error, Overlay};
use neuron_model::{Builder, Head, Particle, Reader, Value::*, execution::NeuronState};

pub(crate) fn publication(
    graph: &Graph,
    namespace: Particle,
    expected: Option<Head>,
    next: Head,
    b: &Builder,
) -> Result<(), Error> {
    let overlay = Overlay { graph, batch: b };
    let mut r = Reader::new(&overlay, 4_000_000);
    let native = if next.index == 0 {
        r.record(next.commit, "neuron/activation/1", 3)
    } else {
        r.record(next.commit, "neuron/commit/1", 8)
    };
    let f = match native {
        Ok(f) => f,
        Err(neuron_model::Error::UnsupportedSchema) => {
            // Unsigned legacy history is an explicit import format. No runtime
            // writer can execute it; the storage fence protects activated origins.
            if next.index == 0 {
                if expected.is_some() || next.commit != namespace {
                    return Err(Error::Conflict);
                }
                r.record(next.commit, "cell/birth/1", 7)?;
            } else {
                let f = r.record(next.commit, "cell/commit/1", 10)?;
                let previous = expected.ok_or(Error::Conflict)?;
                if r.reference(f[0])? != namespace
                    || r.uint(f[1])? != next.index
                    || r.reference(f[2])? != previous.commit
                {
                    return Err(Error::Conflict);
                }
            }
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };
    if r.reference(f[0])? != namespace {
        return Err(Error::Denied);
    }
    let root = r.reference(f[if next.index == 0 { 1 } else { 4 }])?;
    let state = NeuronState::read(&mut r, root)?;
    if state.neuron != namespace {
        return Err(Error::Denied);
    }
    let mut canonical = Builder::new();
    let (statement, authority_state) = if next.index == 0 {
        if expected.is_some()
            || state.epoch != 0
            || state.charged != 0
            || state.held != 0
            || !state.progs.is_empty()
            || !state.invocations.is_empty()
            || !state.imports.is_empty()
            || state.writer_generation != 0
            || state.worker.is_some()
        {
            return Err(Error::Conflict);
        }
        (root, state.clone())
    } else {
        let previous = expected.ok_or(Error::Conflict)?;
        if previous.index.checked_add(1) != Some(next.index)
            || r.uint(f[1])? != next.index
            || r.reference(f[2])? != previous.commit
        {
            return Err(Error::Conflict);
        }
        let parent = if previous.index == 0 {
            r.record(previous.commit, "neuron/activation/1", 3)?
        } else {
            r.record(previous.commit, "neuron/commit/1", 8)?
        };
        if r.reference(parent[0])? != namespace {
            return Err(Error::Denied);
        }
        let before = r.reference(parent[if previous.index == 0 { 1 } else { 4 }])?;
        if r.reference(f[3])? != before {
            return Err(Error::Conflict);
        }
        let prior = NeuronState::read(&mut r, before)?;
        if prior.neuron != namespace
            || prior.network != state.network
            || state.charged < prior.charged
            || state.epoch < prior.epoch
            || state.writer_generation < prior.writer_generation
            || (state.writer_generation == prior.writer_generation && state.worker != prior.worker)
            || (state.writer_generation != prior.writer_generation
                && prior.writer_generation.checked_add(1) != Some(state.writer_generation))
        {
            return Err(Error::Denied);
        }
        let event = r.reference(f[5])?;
        let records = r
            .list(f[6], 4096)?
            .into_iter()
            .map(|v| r.reference(v))
            .collect::<Result<Vec<_>, _>>()?;
        let statement = canonical.values(
            "neuron/proposal/1",
            &[
                Ref(namespace),
                Ref(previous.commit),
                Ref(root),
                Ref(event),
                Refs(&records),
            ],
        )?;
        (statement, prior)
    };
    let auth = r.reference(f[if next.index == 0 { 2 } else { 7 }])?;
    let auth = r.record(auth, "neuron/authorization/1", 6)?;
    if r.reference(auth[0])? != namespace
        || r.reference(auth[1])? != authority_state.network
        || r.reference(auth[2])? != authority_state.policy
        || r.uint(auth[3])? != authority_state.epoch
    {
        return Err(Error::Denied);
    }
    let signed = r.reference(auth[4])?;
    let fields = r.record(signed, "neuron/authority-statement/1", 10)?;
    if r.reference(fields[0])? != namespace
        || r.reference(fields[1])? != authority_state.network
        || r.reference(fields[2])? != authority_state.policy
        || r.uint(fields[3])? != authority_state.epoch
        || r.uint(fields[6])? != 0
        || r.uint(fields[7])? != 0
        || r.reference(fields[9])? != statement
    {
        return Err(Error::Denied);
    }
    r.optional_ref(fields[4])?;
    r.optional_ref(fields[5])?;
    r.text(fields[8])?;
    let evidence = r.reference(auth[5])?;
    let evidence = r.content(evidence)?;
    if !evidence.blob || !verify_evidence(namespace, signed, &evidence.bytes) {
        return Err(Error::Denied);
    }
    Ok(())
}
