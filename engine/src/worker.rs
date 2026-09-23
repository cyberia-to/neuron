//! Authorized placement and a durable one-use dispatch claim.
use crate::neuron::Scope;
use crate::{Action, AttemptPermit, Authority, Dispatch, Error, GraphPort, Neuron, RuntimePort};
use neuron_model::{
    Builder, Head, Lifecycle, NeuronId, Particle, Reader, Value::*, worker::Worker,
};

impl<G: GraphPort, R: RuntimePort, A: Authority> Neuron<G, R, A> {
    /// Local placement CAS. Caller must own the local database; remote takeover
    /// is a different protocol. Unknown effects retain their previous placement.
    pub fn place_worker(
        &self,
        neuron: NeuronId,
        expected: u64,
        worker: Option<&Worker>,
    ) -> Result<(u64, Option<Particle>, Head), Error> {
        let old = self.inspect(neuron)?;
        if old.state.writer_generation != expected {
            return Err(Error::Conflict);
        }
        let mut state = old.state.clone();
        let mut b = Builder::new();
        if worker.is_some_and(|w| w.network != state.network) {
            return Err(Error::Denied);
        }
        let worker = worker.map(|w| w.encode(&mut b)).transpose()?;
        let generation = expected.checked_add(1).ok_or(Error::Budget)?;
        state.writer_generation = generation;
        state.worker = worker;
        let record = b.values("neuron/placement/1", &[Uint(generation), Optional(worker)])?;
        let head = self.transition(
            &old,
            state,
            b,
            Scope {
                kind: "place-worker",
                prog: None,
                invocation: None,
            },
            Some(record),
            &[record],
        )?;
        Ok((generation, worker, head))
    }

    /// Consume a durable claim before calling an embedded synchronous effect.
    /// Every duplicate (including after a lost receipt) fails without the callback.
    pub fn dispatch_once(
        &self,
        permit: AttemptPermit,
        effect: &mut dyn FnMut(&Dispatch) -> Result<(), Error>,
    ) -> Result<(Dispatch, Head), Error> {
        let d = &permit.dispatch;
        let old = self.inspect(d.neuron)?;
        let mut state = old.state.clone();
        if state.network != d.network
            || state.policy != d.policy
            || state.epoch != d.epoch
            || state.writer_generation != d.generation
            || d.generation == 0
            || state.worker != Some(d.executor)
        {
            return Err(Error::Fenced);
        }
        let job = state.invocations.get(&d.invocation).ok_or(Error::Missing)?;
        let program = state.progs.get(&d.prog).ok_or(Error::Missing)?;
        if job.prog != d.prog || job.epoch != d.epoch || program.lifecycle != Lifecycle::Active {
            return Err(Error::Denied);
        }
        let pending = job.pending.as_ref().ok_or(Error::Missing)?;
        if pending.stage != 1 || pending.dispatch.is_some() {
            return Err(Error::UnknownOutcome);
        }
        if pending.id != d.operation || pending.attempt != Some(d.attempt) || pending.tag != d.tag {
            return Err(Error::Conflict);
        }
        let mut r = Reader::new(&self.graph, 100_000);
        let worker = Worker::read(&mut r, d.executor)?;
        if worker.network != d.network
            || !worker.acts.contains(&d.tag)
            || !program.allowed_acts.contains(&d.tag)
            || d.arguments.len() as u64 > worker.max_arguments
        {
            return Err(Error::Denied);
        }
        if neuron_model::execution::read_artifact(&mut r, pending.arguments)? != d.arguments {
            return Err(Error::Conflict);
        }
        let attempt = r.record(
            pending.attempt_record.ok_or(Error::Unsupported)?,
            "neuron/attempt/1",
            4,
        )?;
        if r.reference(attempt[0])? != d.operation
            || r.reference(attempt[1])? != d.attempt
            || r.reference(attempt[2])? != d.authorization
            || r.reference(attempt[3])? != d.executor
        {
            return Err(Error::Conflict);
        }
        let auth = r.record(d.authorization, "neuron/authorization/1", 6)?;
        let signed = r.reference(auth[4])?;
        let auth_fields = r.record(signed, "neuron/authority-statement/1", 10)?;
        let binding = r.reference(auth_fields[9])?;
        let binding = r.record(binding, "neuron/attempt-authorization/1", 4)?;
        if r.reference(binding[0])? != pending.record
            || r.reference(binding[1])? != d.attempt
            || r.reference(binding[2])? != d.executor
            || r.uint(binding[3])? != d.generation
        {
            return Err(Error::Conflict);
        }

        let allowed = program.allowed_acts.clone();
        let mut b = Builder::new();
        let record = b.values(
            "neuron/dispatch/1",
            &[
                Ref(d.operation),
                Ref(d.attempt),
                Uint(d.generation),
                Ref(d.executor),
            ],
        )?;
        state
            .invocations
            .get_mut(&d.invocation)
            .unwrap()
            .pending
            .as_mut()
            .unwrap()
            .dispatch = Some(record);
        let publication = self.prepare(
            &old,
            state,
            b,
            (record, record),
            Scope {
                kind: "dispatch-claim",
                prog: Some(d.prog),
                invocation: Some(d.invocation),
            },
            &[record],
        )?;
        let head = publication.head;
        let mut publication = Some(publication);
        let action = Action {
            neuron: d.neuron,
            network: d.network,
            policy: d.policy,
            epoch: d.epoch,
            prog: Some(d.prog),
            invocation: Some(d.invocation),
            act: Some(d.tag),
            kind: "dispatch-claim",
            statement: record,
            allowed_acts: &allowed,
        };
        // No recursive authorize while the ward read guard is held. Preparation
        // above signs; this callback only commits and crosses the effect boundary.
        self.authority.with_current(&action, &mut || {
            let p = publication.take().ok_or(Error::UnknownOutcome)?;
            self.graph.commit_once(crate::AtomicWrite {
                namespace: p.neuron,
                request: p.request,
                expected: Some(old.head),
                head: p.head,
                content: p.content,
                claim: Some((p.request, p.event)),
            })?;
            effect(d)
        })?;
        if publication.is_some() {
            return Err(Error::Denied);
        }
        Ok((permit.dispatch, head))
    }
}
