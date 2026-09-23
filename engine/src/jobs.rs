use crate::neuron::Scope;
use crate::{Authority, Error, GraphPort, Neuron, RuntimePort, RuntimeStep, View};
use neuron_model::execution::{self as model, NeuronState, PendingOperation, Status};
use neuron_model::{Builder, Head, Lifecycle, NeuronId, Particle, Reader, Value::*};

pub const SLICE_STEPS: u64 = 1000;
#[derive(Debug, Clone)]
pub enum JobProgress {
    Idle,
    Yielded {
        invocation: Particle,
        head: Head,
    },
    Awaiting {
        invocation: Particle,
        operation: Particle,
        tag: u64,
    },
    Unknown {
        invocation: Particle,
        operation: Particle,
        attempt: Particle,
    },
    Event {
        invocation: Particle,
        operation: Particle,
        tag: u64,
        selector: Vec<u8>,
    },
    Finished {
        invocation: Particle,
        status: Status,
        head: Head,
    },
}
#[derive(Debug, Clone)]
pub struct Dispatch {
    pub neuron: NeuronId,
    pub network: Particle,
    pub prog: Particle,
    pub invocation: Particle,
    pub operation: Particle,
    pub attempt: Particle,
    pub policy: Particle,
    pub epoch: u64,
    pub tag: u64,
    pub arguments: Vec<u8>,
    pub authorization: Particle,
    pub executor: Particle,
    pub generation: u64,
}

/// A live, non-clonable permission to request the one-use dispatch claim.
/// Only a newly completed attempt preparation can construct it.
pub struct AttemptPermit {
    pub(crate) dispatch: Dispatch,
}
impl AttemptPermit {
    pub fn metadata(&self) -> &Dispatch {
        &self.dispatch
    }
}

fn charge(state: &mut NeuronState, id: Particle, amount: u64, used: u64) -> Result<(), Error> {
    let job = state.invocations.get_mut(&id).ok_or(Error::Missing)?;
    if amount > job.available()? || used < job.used || used - job.used > amount {
        return Err(Error::Budget);
    }
    job.charged = job.charged.checked_add(amount).ok_or(Error::Budget)?;
    job.used = used;
    job.reserved = 0;
    state.charged = state.charged.checked_add(amount).ok_or(Error::Budget)?;
    Ok(())
}
/// Complete a subtree and refund unused child allowance without creating budget.
fn settle(state: &mut NeuronState, mut id: Particle, cancel: bool) -> Result<(), Error> {
    let mut cancelling = cancel;
    loop {
        let job = state.invocations.get(&id).ok_or(Error::Missing)?;
        if job
            .children
            .iter()
            .any(|child| !state.invocations[child].status.terminal())
        {
            state.invocations.get_mut(&id).ok_or(Error::Missing)?.status = Status::Joining;
            return Ok(());
        }
        let prog = state.progs.get_mut(&job.prog).ok_or(Error::Missing)?;
        let status = if cancelling {
            Status::Cancelled
        } else if job.fault.is_some() {
            Status::Failed
        } else if prog.revision != job.base_revision {
            Status::Conflict
        } else {
            prog.state = job.result.ok_or(Error::Conflict)?;
            prog.revision = prog.revision.checked_add(1).ok_or(Error::Budget)?;
            Status::Completed
        };
        let parent = job.parent;
        let refund = job.available()?;
        state.invocations.get_mut(&id).ok_or(Error::Missing)?.status = status;
        let Some(parent) = parent else {
            return Ok(());
        };
        let p = state.invocations.get_mut(&parent).ok_or(Error::Missing)?;
        p.delegated = p.delegated.checked_sub(refund).ok_or(Error::Budget)?;
        if p.status != Status::Joining {
            return Ok(());
        }
        id = parent;
        cancelling = false;
    }
}
fn finish(
    state: &mut NeuronState,
    id: Particle,
    result: Option<Particle>,
    fault: Option<Particle>,
    cancel: bool,
) -> Result<(), Error> {
    let job = state.invocations.get_mut(&id).ok_or(Error::Missing)?;
    job.checkpoint = None;
    job.pending = None;
    job.reserved = 0;
    job.result = result;
    job.fault = fault;
    settle(state, id, cancel)
}

impl<G: GraphPort, R: RuntimePort, A: Authority> Neuron<G, R, A> {
    pub fn tick(&self, neuron: NeuronId) -> Result<JobProgress, Error> {
        let view = self.inspect(neuron)?;
        let ids = view.state.invocations.keys().copied().collect::<Vec<_>>();
        if ids.is_empty() {
            return Ok(JobProgress::Idle);
        }
        for offset in 0..ids.len() {
            let i = (view.state.cursor as usize % ids.len() + offset) % ids.len();
            let id = ids[i];
            let job = &view.state.invocations[&id];
            if view.state.progs[&job.prog].lifecycle == Lifecycle::Active
                && job.epoch == view.state.epoch
                && (job.status == Status::Running
                    || job
                        .pending
                        .as_ref()
                        .is_some_and(|p| matches!(p.stage, 2 | 4)))
            {
                return self.run(view, id, (i + 1) as u64);
            }
        }
        Ok(JobProgress::Idle)
    }
    pub fn tick_invocation(&self, neuron: NeuronId, id: Particle) -> Result<JobProgress, Error> {
        let view = self.inspect(neuron)?;
        let cursor = view.state.cursor;
        self.run(view, id, cursor)
    }
    fn run(&self, old: View, id: Particle, cursor: u64) -> Result<JobProgress, Error> {
        let mut state = old.state.clone();
        let job = state.invocations.get(&id).ok_or(Error::Missing)?;
        if job.status.terminal() {
            return Ok(JobProgress::Finished {
                invocation: id,
                status: job.status,
                head: old.head,
            });
        }
        if job.status == Status::Joining {
            return Ok(JobProgress::Idle);
        }
        if state.progs[&job.prog].lifecycle != Lifecycle::Active {
            return Err(Error::Lifecycle);
        }
        let prog = job.prog;
        let mut b = Builder::new();
        if let Some(p) = &job.pending {
            match p.stage {
                0 => {
                    return Ok(JobProgress::Awaiting {
                        invocation: id,
                        operation: p.id,
                        tag: p.tag,
                    });
                }
                1 => {
                    return Ok(JobProgress::Unknown {
                        invocation: id,
                        operation: p.id,
                        attempt: p.attempt.ok_or(Error::Conflict)?,
                    });
                }
                3 => {
                    return Ok(JobProgress::Event {
                        invocation: id,
                        operation: p.id,
                        tag: p.tag,
                        selector: model::read_artifact(
                            &mut Reader::new(&self.graph, 50_000),
                            p.arguments,
                        )?,
                    });
                }
                2 | 4 => {}
                _ => return Err(Error::Conflict),
            }
        }
        if job.epoch != state.epoch {
            return Err(Error::Denied);
        }
        if job.reserved > 0 {
            let reserved = job.reserved;
            let used = job.used;
            charge(&mut state, id, reserved, used)?;
            let head = self.transition(
                &old,
                state,
                b,
                Scope {
                    kind: "recover-reservation",
                    prog: Some(prog),
                    invocation: Some(id),
                },
                Some(id),
                &[],
            )?;
            return Ok(JobProgress::Yielded {
                invocation: id,
                head,
            });
        }
        let available = job.available()?;
        if available == 0 {
            if job
                .children
                .iter()
                .any(|child| !state.invocations[child].status.terminal())
            {
                return Ok(JobProgress::Idle);
            }
            let fault = model::artifact(
                &mut b,
                b"invocation budget exhausted".to_vec(),
                "text/plain",
            )?;
            let mut records = Vec::new();
            if let Some(p) = &job.pending
                && matches!(p.stage, 2 | 4)
            {
                records.push(self.consumed(&mut b, id, p)?);
            }
            finish(&mut state, id, None, Some(fault), false)?;
            let status = state.invocations[&id].status;
            let head = self.transition(
                &old,
                state,
                b,
                Scope {
                    kind: "budget-exhausted",
                    prog: Some(prog),
                    invocation: Some(id),
                },
                Some(fault),
                &records,
            )?;
            return Ok(JobProgress::Finished {
                invocation: id,
                status,
                head,
            });
        }
        if let Some(p) = &job.pending
            && p.failed
        {
            let fault = p.value.ok_or(Error::Conflict)?;
            let consumed = self.consumed(&mut b, id, p)?;
            finish(&mut state, id, None, Some(fault), false)?;
            let status = state.invocations[&id].status;
            let head = self.transition(
                &old,
                state,
                b,
                Scope {
                    kind: "effect-failed",
                    prog: Some(prog),
                    invocation: Some(id),
                },
                Some(fault),
                &[consumed],
            )?;
            return Ok(JobProgress::Finished {
                invocation: id,
                status,
                head,
            });
        }
        let slice = available.min(SLICE_STEPS);
        state
            .invocations
            .get_mut(&id)
            .ok_or(Error::Missing)?
            .reserved = slice;
        state.cursor = cursor;
        let reserved = self.transition(
            &old,
            state,
            b,
            Scope {
                kind: "reserve",
                prog: Some(prog),
                invocation: Some(id),
            },
            Some(id),
            &[],
        )?;
        let old = self.inspect(old.state.neuron)?;
        if old.head != reserved {
            return Err(Error::Conflict);
        }
        let job = &old.state.invocations[&id];
        let mut reader = Reader::new(&self.graph, 50_000);
        let checkpoint = model::read_artifact(&mut reader, job.checkpoint.ok_or(Error::Missing)?)?;
        let reply = job
            .pending
            .as_ref()
            .and_then(|p| p.value)
            .map(|v| model::read_artifact(&mut reader, v))
            .transpose()?;
        let step = self
            .runtime
            .step(&checkpoint, reply.as_deref(), slice, job.limit);
        self.accept_step(old, id, step)
    }
    fn consumed(
        &self,
        b: &mut Builder,
        id: Particle,
        p: &PendingOperation,
    ) -> Result<Particle, Error> {
        Ok(b.values(
            "neuron/consumed/1",
            &[Ref(p.id), Ref(p.outcome.ok_or(Error::Conflict)?), Ref(id)],
        )?)
    }
    fn accept_step(
        &self,
        old: View,
        id: Particle,
        step: Result<RuntimeStep, Error>,
    ) -> Result<JobProgress, Error> {
        let mut state = old.state.clone();
        let job = &state.invocations[&id];
        let prog = job.prog;
        let mut b = Builder::new();
        let mut records = Vec::new();
        if let Some(p) = &job.pending {
            records.push(self.consumed(&mut b, id, p)?);
        }
        let used = match &step {
            Ok(
                RuntimeStep::Done { used, .. }
                | RuntimeStep::Yield { used, .. }
                | RuntimeStep::Act { used, .. }
                | RuntimeStep::Event { used, .. },
            ) => Some(*used),
            Err(_) => None,
        };
        let valid = used.is_some_and(|used| used >= job.used && used - job.used <= job.reserved);
        let amount = if valid {
            used.ok_or(Error::Budget)? - job.used
        } else {
            job.reserved
        };
        let prior_used = job.used;
        charge(
            &mut state,
            id,
            amount,
            if valid {
                used.ok_or(Error::Budget)?
            } else {
                prior_used
            },
        )?;
        let step = if !valid && step.is_ok() {
            Err(Error::Budget)
        } else {
            step
        };
        match step {
            Err(error) => {
                let fault = model::artifact(&mut b, error.to_string().into_bytes(), "text/plain")?;
                finish(&mut state, id, None, Some(fault), false)?;
            }
            Ok(RuntimeStep::Done { result, .. }) => match self.runtime.validate_value(&result) {
                Ok(()) => {
                    let result = model::artifact(&mut b, result, "application/x-rune-noun")?;
                    finish(&mut state, id, Some(result), None, false)?;
                }
                Err(error) => {
                    let fault =
                        model::artifact(&mut b, error.to_string().into_bytes(), "text/plain")?;
                    finish(&mut state, id, None, Some(fault), false)?;
                }
            },
            Ok(RuntimeStep::Yield { checkpoint, .. }) => {
                let checkpoint =
                    model::artifact(&mut b, checkpoint, "application/x-rune-checkpoint")?;
                let job = state.invocations.get_mut(&id).ok_or(Error::Missing)?;
                job.checkpoint = Some(checkpoint);
                job.pending = None;
                job.status = Status::Running;
            }
            Ok(step) => {
                let (tag, arguments, checkpoint, event) = match step {
                    RuntimeStep::Act {
                        tag,
                        arguments,
                        checkpoint,
                        ..
                    } => (tag, arguments, checkpoint, false),
                    RuntimeStep::Event {
                        tag,
                        selector,
                        checkpoint,
                        ..
                    } => (tag, selector, checkpoint, true),
                    _ => return Err(Error::Conflict),
                };
                if !event && !state.progs[&prog].allowed_acts.contains(&tag) {
                    let fault = model::artifact(&mut b, b"undeclared act".to_vec(), "text/plain")?;
                    finish(&mut state, id, None, Some(fault), false)?;
                } else {
                    let job = state.invocations.get_mut(&id).ok_or(Error::Missing)?;
                    let arguments = model::artifact(&mut b, arguments, "application/x-rune-noun")?;
                    let checkpoint =
                        model::artifact(&mut b, checkpoint, "application/x-rune-checkpoint")?;
                    let operation = b.values(
                        "neuron/operation-id/1",
                        &[Ref(state.neuron), Ref(prog), Ref(id), Uint(job.ordinal)],
                    )?;
                    job.ordinal = job.ordinal.checked_add(1).ok_or(Error::Budget)?;
                    let executor = b.blob(if event {
                        b"neuron/local-event/1".to_vec()
                    } else {
                        b"neuron/local-manual-reconciliation/1".to_vec()
                    })?;
                    let record = b.values(
                        "neuron/operation/1",
                        &[
                            Ref(state.neuron),
                            Ref(state.network),
                            Ref(prog),
                            Ref(id),
                            Ref(operation),
                            Uint(tag),
                            Ref(arguments),
                            Ref(state.policy),
                            Uint(job.epoch),
                            Ref(executor),
                        ],
                    )?;
                    job.checkpoint = Some(checkpoint);
                    job.status = Status::Waiting;
                    job.pending = Some(PendingOperation {
                        id: operation,
                        record,
                        tag,
                        arguments,
                        attempt: None,
                        outcome: None,
                        stage: if event { 3 } else { 0 },
                        value: None,
                        failed: false,
                        dispatch: None,
                        attempt_record: None,
                    });
                    records.push(record);
                }
            }
        }
        let status = state.invocations[&id].status;
        let pending = state.invocations[&id].pending.clone();
        let head = self.transition(
            &old,
            state,
            b,
            Scope {
                kind: "step",
                prog: Some(prog),
                invocation: Some(id),
            },
            Some(id),
            &records,
        )?;
        if status.terminal() || status == Status::Joining {
            return Ok(JobProgress::Finished {
                invocation: id,
                status,
                head,
            });
        }
        if let Some(p) = pending {
            if p.stage == 3 {
                return Ok(JobProgress::Event {
                    invocation: id,
                    operation: p.id,
                    tag: p.tag,
                    selector: model::read_artifact(
                        &mut Reader::new(&self.graph, 50_000),
                        p.arguments,
                    )?,
                });
            }
            return Ok(JobProgress::Awaiting {
                invocation: id,
                operation: p.id,
                tag: p.tag,
            });
        }
        Ok(JobProgress::Yielded {
            invocation: id,
            head,
        })
    }
    pub fn begin_attempt(&self, neuron: NeuronId, id: Particle) -> Result<Dispatch, Error> {
        self.begin_attempt_inner(neuron, id, None)
    }
    pub fn begin_worker_attempt(
        &self,
        neuron: NeuronId,
        id: Particle,
        worker: Particle,
        generation: u64,
    ) -> Result<AttemptPermit, Error> {
        Ok(AttemptPermit {
            dispatch: self.begin_attempt_inner(neuron, id, Some((worker, generation)))?,
        })
    }
    fn begin_attempt_inner(
        &self,
        neuron: NeuronId,
        id: Particle,
        selected: Option<(Particle, u64)>,
    ) -> Result<Dispatch, Error> {
        let old = self.inspect(neuron)?;
        let mut state = old.state.clone();
        let job = state.invocations.get(&id).ok_or(Error::Missing)?;
        if job.epoch != state.epoch {
            return Err(Error::Denied);
        }
        let prog = job.prog;
        let program = &state.progs[&prog];
        if program.lifecycle != Lifecycle::Active {
            return Err(Error::Lifecycle);
        }
        let p = job.pending.as_ref().ok_or(Error::Missing)?;
        if p.stage == 1 {
            return Err(Error::UnknownOutcome);
        }
        if p.stage != 0 {
            return Err(Error::Conflict);
        }
        if !program.allowed_acts.contains(&p.tag) {
            return Err(Error::Denied);
        }
        let mut b = Builder::new();
        let attempt = b.values(
            "neuron/attempt-id/1",
            &[Ref(p.id), Uint(0), Uint(job.epoch)],
        )?;
        let mut r = Reader::new(&self.graph, 50_000);
        let arguments = model::read_artifact(&mut r, p.arguments)?;
        let (executor, generation) = if let Some((worker, generation)) = selected {
            if state.worker != Some(worker) || state.writer_generation != generation {
                return Err(Error::Fenced);
            }
            let contract = neuron_model::worker::Worker::read(&mut r, worker)?;
            if contract.network != state.network
                || !contract.acts.contains(&p.tag)
                || arguments.len() as u64 > contract.max_arguments
            {
                return Err(Error::Denied);
            }
            (worker, generation)
        } else {
            (b.blob(b"neuron/local-manual-reconciliation/1".to_vec())?, 0)
        };
        let bound = b.values(
            "neuron/attempt-authorization/1",
            &[Ref(p.record), Ref(attempt), Ref(executor), Uint(generation)],
        )?;
        let auth = self.evidence(
            &mut b,
            &state,
            Scope {
                kind: "dispatch",
                prog: Some(prog),
                invocation: Some(id),
            },
            Some(p.tag),
            bound,
            &program.allowed_acts,
        )?;
        let record = b.values(
            "neuron/attempt/1",
            &[Ref(p.id), Ref(attempt), Ref(auth), Ref(executor)],
        )?;
        let dispatch = Dispatch {
            neuron,
            network: state.network,
            prog,
            invocation: id,
            operation: p.id,
            attempt,
            policy: state.policy,
            epoch: job.epoch,
            tag: p.tag,
            arguments,
            authorization: auth,
            executor,
            generation,
        };
        let pending = state
            .invocations
            .get_mut(&id)
            .and_then(|j| j.pending.as_mut())
            .ok_or(Error::Missing)?;
        pending.stage = 1;
        pending.attempt = Some(attempt);
        pending.attempt_record = Some(record);
        self.transition(
            &old,
            state,
            b,
            Scope {
                kind: "attempt",
                prog: Some(prog),
                invocation: Some(id),
            },
            Some(record),
            &[record],
        )?;
        Ok(dispatch)
    }
    pub fn record_outcome(
        &self,
        neuron: NeuronId,
        id: Particle,
        operation: Particle,
        attempt: Particle,
        value: Vec<u8>,
    ) -> Result<Head, Error> {
        self.runtime.validate_value(&value)?;
        self.outcome(neuron, id, operation, attempt, value, false)
    }
    pub fn record_failure(
        &self,
        neuron: NeuronId,
        id: Particle,
        operation: Particle,
        attempt: Particle,
        reason: String,
    ) -> Result<Head, Error> {
        if reason.len() > 4096 {
            return Err(Error::Budget);
        }
        self.outcome(neuron, id, operation, attempt, reason.into_bytes(), true)
    }
    fn outcome(
        &self,
        neuron: NeuronId,
        id: Particle,
        operation: Particle,
        attempt: Particle,
        value: Vec<u8>,
        failed: bool,
    ) -> Result<Head, Error> {
        let mut b = Builder::new();
        let value = model::artifact(
            &mut b,
            value,
            if failed {
                "text/plain"
            } else {
                "application/x-rune-noun"
            },
        )?;
        let record = b.values(
            "neuron/outcome/1",
            &[
                Ref(id),
                Ref(operation),
                Ref(attempt),
                Ref(value),
                Uint(u64::from(failed)),
            ],
        )?;
        let request = b.values("neuron/outcome-request/1", &[Ref(operation), Ref(attempt)])?;
        if let Some(head) = self.retry(neuron, request, record)? {
            return Ok(head);
        }
        let old = self.inspect(neuron)?;
        let mut state = old.state.clone();
        let job = state.invocations.get_mut(&id).ok_or(Error::Missing)?;
        let p = job.pending.as_mut().ok_or(Error::Missing)?;
        let prog = job.prog;
        if p.id != operation || p.attempt != Some(attempt) || p.stage != 1 {
            return Err(Error::Conflict);
        }
        p.stage = 2;
        p.outcome = Some(record);
        p.value = Some(value);
        p.failed = failed;
        self.publish(
            &old,
            state,
            b,
            (record, request),
            Scope {
                kind: "reconcile",
                prog: Some(prog),
                invocation: Some(id),
            },
            &[record],
        )
    }
    pub fn wake(
        &self,
        neuron: NeuronId,
        id: Particle,
        operation: Particle,
        nonce: [u8; 32],
        value: Vec<u8>,
    ) -> Result<Head, Error> {
        self.runtime.validate_value(&value)?;
        let mut b = Builder::new();
        let request = Self::request(&mut b, neuron, nonce)?;
        let value = model::artifact(&mut b, value, "application/x-rune-noun")?;
        let nonce = b.nonce(nonce)?;
        let event = b.values(
            "neuron/wake/1",
            &[Ref(id), Ref(operation), Raw(nonce), Ref(value)],
        )?;
        if let Some(head) = self.retry(neuron, request, event)? {
            return Ok(head);
        }
        let old = self.inspect(neuron)?;
        let mut state = old.state.clone();
        let job = state.invocations.get_mut(&id).ok_or(Error::Missing)?;
        let p = job.pending.as_mut().ok_or(Error::Missing)?;
        let prog = job.prog;
        if p.stage != 3 || p.id != operation {
            return Err(Error::Conflict);
        }
        let response = b.values(
            "neuron/event-response/1",
            &[Ref(id), Ref(operation), Ref(event), Ref(value)],
        )?;
        p.stage = 4;
        p.outcome = Some(response);
        p.value = Some(value);
        self.publish(
            &old,
            state,
            b,
            (event, request),
            Scope {
                kind: "wake",
                prog: Some(prog),
                invocation: Some(id),
            },
            &[response],
        )
    }
    pub fn cancel(&self, neuron: NeuronId, id: Particle, reason: String) -> Result<Head, Error> {
        if reason.len() > 4096 {
            return Err(Error::Budget);
        }
        let old = self.inspect(neuron)?;
        let mut state = old.state.clone();
        let job = state.invocations.get(&id).ok_or(Error::Missing)?;
        if job.status.terminal() {
            return Ok(old.head);
        }
        if job.pending.as_ref().is_some_and(|p| p.stage == 1) {
            return Err(Error::UnknownOutcome);
        }
        if job
            .children
            .iter()
            .any(|id| !state.invocations[id].status.terminal())
        {
            return Err(Error::Busy);
        }
        let prog = job.prog;
        let reserved = job.reserved;
        let used = job.used;
        let mut b = Builder::new();
        let mut records = Vec::new();
        if let Some(p) = &job.pending
            && matches!(p.stage, 2 | 4)
        {
            records.push(self.consumed(&mut b, id, p)?);
        }
        charge(&mut state, id, reserved, used)?;
        let fault = model::artifact(&mut b, reason.into_bytes(), "text/plain")?;
        finish(&mut state, id, None, Some(fault), true)?;
        self.transition(
            &old,
            state,
            b,
            Scope {
                kind: "cancel",
                prog: Some(prog),
                invocation: Some(id),
            },
            Some(fault),
            &records,
        )
    }
    /// Archive only complete top-level trees. Charges and graph request claims remain.
    pub fn archive(&self, neuron: NeuronId, id: Particle) -> Result<Head, Error> {
        let old = self.inspect(neuron)?;
        let mut state = old.state.clone();
        let job = state.invocations.get(&id).ok_or(Error::Missing)?;
        if job.parent.is_some() || !job.status.terminal() {
            return Err(Error::Busy);
        }
        let prog = job.prog;
        let mut pending = vec![id];
        while let Some(id) = pending.pop() {
            let job = state.invocations.remove(&id).ok_or(Error::Missing)?;
            pending.extend(job.children);
        }
        self.transition(
            &old,
            state,
            Builder::new(),
            Scope {
                kind: "archive",
                prog: Some(prog),
                invocation: Some(id),
            },
            Some(id),
            &[],
        )
    }
}
