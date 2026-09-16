//! Read-only interpretation of immutable runtime-cell v1 history.
//! A legacy birth hash is an origin particle, never a native NeuronId.
use crate::{Error, GraphPort};
mod artifacts;
mod records;
use neuron_model::{Head, Particle, Reader, Snapshot};
pub use records::{Live as LegacyContinuation, Pending as LegacyOperation};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct LegacyView {
    pub origin: Particle,
    pub head: Head,
    pub snapshot: Snapshot,
    pub source: Particle,
    pub allowance: u64,
    pub allowed_acts: Vec<u64>,
    pub charged: u64,
    pub live: Option<LegacyContinuation>,
}
pub fn inspect<G: GraphPort>(graph: &G, origin: Particle) -> Result<LegacyView, Error> {
    let head = graph.head(origin)?.ok_or(Error::Missing)?;
    let mut r = Reader::new(graph, 100_000);
    if head.index == 0 && head.commit != origin {
        return Err(Error::Conflict);
    }
    let (_, snapshot) = records::snapshot_at(&mut r, head)?;
    if snapshot.profile != records::profile(&mut neuron_model::Builder::new())? {
        return Err(Error::Unsupported);
    }
    let definition = r.record(snapshot.definition, "cell/definition/1", 11)?;
    let source = r.reference(definition[2])?;
    r.artifact(source)?;
    r.artifact(snapshot.application_state)?;
    let (allowance, allowed_acts) = records::policy_values(&mut r, snapshot.authority_policy)?;
    let live = records::read_live(&mut r, &snapshot)?;
    if let Some(live) = &live {
        r.artifact(live.checkpoint)?;
        let event = r.record(live.event, "cell/event/1", 10)?;
        let input = r.reference(event[4])?;
        r.artifact(input)?;
        if let Some(p) = &live.pending {
            r.artifact(p.arguments)?;
        }
    }
    // Each old invocation emits cumulative usage; sum its final charge, not all
    // intermediate reservations/settlements. Terminal invocations remain history.
    let mut usage = BTreeMap::<Particle, u64>::new();
    let mut current = None;
    let mut after = None;
    let mut previous = None;
    let mut expected = 0u64;
    loop {
        let page = graph.history(origin, after, 512)?;
        if page.is_empty() {
            break;
        }
        for h in &page {
            if h.index != expected || h.index > head.index {
                return Err(Error::Conflict);
            }
            expected = expected.checked_add(1).ok_or(Error::Budget)?;
            let mut r = Reader::new(graph, 100_000);
            artifacts::snapshot(&mut r, *h)?;
            if h.index == 0 {
                if h.commit != origin {
                    return Err(Error::Conflict);
                }
                r.record(h.commit, "cell/birth/1", 7)?;
            } else {
                let f = r.record(h.commit, "cell/commit/1", 10)?;
                if r.reference(f[0])? != origin
                    || r.uint(f[1])? != h.index
                    || Some(r.reference(f[2])?) != previous
                {
                    return Err(Error::Conflict);
                }
                for event in r.list(f[5], 1)? {
                    let id = r.reference(event)?;
                    artifacts::event(&mut r, id)?;
                    let e = r.record(id, "cell/event/1", 10)?;
                    if r.text(e[3])? == "main" {
                        current = Some(id);
                    }
                }
                for record in r.list(f[6], 64)? {
                    let id = r.reference(record)?;
                    artifacts::operation(&mut r, id)?;
                    let resource = match r.record(id, "cell/continuation/1", 9) {
                        Ok(c) => {
                            current = Some(r.reference(c[6])?);
                            Some(r.reference(c[8])?)
                        }
                        Err(neuron_model::Error::UnsupportedSchema) => None,
                        Err(e) => return Err(e.into()),
                    };
                    match r.record(resource.unwrap_or(id), "cell/local-resources/1", 4) {
                        Ok(v) => {
                            let invocation = current.ok_or(Error::Conflict)?;
                            let charged = r.uint(v[0])?;
                            let old = usage.entry(invocation).or_default();
                            if charged < *old {
                                return Err(Error::Conflict);
                            }
                            *old = charged;
                        }
                        Err(neuron_model::Error::UnsupportedSchema) => {}
                        Err(e) => return Err(e.into()),
                    }
                }
            }
            previous = Some(h.commit);
        }
        after = page.last().map(|h| h.index);
    }
    if previous != Some(head.commit) || graph.head(origin)? != Some(head) {
        return Err(Error::Conflict);
    }
    let charged = usage
        .values()
        .try_fold(0u64, |n, v| n.checked_add(*v).ok_or(Error::Budget))?;
    Ok(LegacyView {
        origin,
        head,
        snapshot,
        source,
        allowance,
        allowed_acts,
        charged,
        live,
    })
}

pub fn resolve<G: GraphPort>(
    graph: &G,
    origin: Particle,
    nonce: [u8; 32],
) -> Result<Option<Head>, Error> {
    let mut b = neuron_model::Builder::new();
    let nonce = b.nonce(nonce)?;
    let owner = records::authority(&mut b)?;
    let request = b.values(
        "cell/request-id/1",
        &[
            neuron_model::Value::Ref(owner),
            neuron_model::Value::Raw(nonce),
        ],
    )?;
    graph.resolve(origin, request)
}
