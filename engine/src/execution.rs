use crate::{Engine, Error, GraphPort, RuntimePort, RuntimeStep, records as rec};
use cell_model::{Builder, Head, Lifecycle, Particle, Reader, Value::*};

#[derive(Debug)]
pub enum Progress {
    Idle,
    Advanced(Head),
    Complete(Head),
    Awaiting {
        operation: Particle,
        tag: u64,
        arguments: Vec<u8>,
    },
    Unknown {
        operation: Particle,
        attempt: Particle,
    },
}
#[derive(Debug)]
pub struct Attempt {
    pub operation: Particle,
    pub attempt: Particle,
    pub tag: u64,
    pub arguments: Vec<u8>,
}

impl<G: GraphPort, R: RuntimePort> Engine<G, R> {
    pub fn tick(&self, cell: Particle) -> Result<Progress, Error> {
        let mut loaded = self.load(cell)?;
        if loaded.snapshot.lifecycle != Lifecycle::Active {
            return Err(Error::Lifecycle);
        }
        let Some(mut live) = loaded.live.clone() else {
            return Ok(Progress::Idle);
        };
        if let Some(p) = &live.pending {
            if p.stage == "pending" {
                return Ok(Progress::Awaiting {
                    operation: p.id,
                    tag: p.tag,
                    arguments: Reader::new(&self.graph, 20_000).artifact(p.arguments)?,
                });
            }
            if p.stage == "attempt-recorded" {
                let mut r = Reader::new(&self.graph, 20_000);
                let fields = r.record(p.attempt.ok_or(Error::Conflict)?, "cell/attempt/1", 7)?;
                return Ok(Progress::Unknown {
                    operation: p.id,
                    attempt: r.reference(fields[0])?,
                });
            }
            if p.stage == "resolved" {
                let mut r = Reader::new(&self.graph, 20_000);
                let outcome = r.record(p.outcome.ok_or(Error::Conflict)?, "cell/outcome/1", 6)?;
                if outcome[2] == Builder::new().variant("outcome/failed-definite", &[])? {
                    let diagnostic = r.optional_ref(outcome[4])?.ok_or(Error::Conflict)?;
                    let bytes = r.artifact(diagnostic)?;
                    let reason = std::str::from_utf8(&bytes).map_err(|_| Error::Conflict)?;
                    self.finish(cell, &loaded, None, Some(reason), live.charged, live.used)?;
                    return Err(Error::Runtime(reason.to_owned()));
                }
            }
        }
        if live.reserved > 0 {
            live.charged = live
                .charged
                .checked_add(live.reserved)
                .ok_or(Error::Budget)?;
            live.reserved = 0;
            self.save_live(cell, &loaded, &live, "$recover-budget", Builder::new(), &[])?;
            loaded = self.load(cell)?;
        }
        let remaining = live.limit.checked_sub(live.charged).ok_or(Error::Budget)?;
        if remaining == 0 {
            self.finish(
                cell,
                &loaded,
                None,
                Some("runtime budget exhausted"),
                live.charged,
                live.used,
            )?;
            return Err(Error::Budget);
        }
        live.reserved = remaining.min(rec::SLICE);
        self.save_live(cell, &loaded, &live, "$reserve", Builder::new(), &[])?;
        loaded = self.load(cell)?;
        let mut r = Reader::new(&self.graph, 30_000);
        let checkpoint = r.artifact(live.checkpoint)?;
        let reply = if let Some(p) = &live.pending {
            let fields = r.record(p.outcome.ok_or(Error::Conflict)?, "cell/outcome/1", 6)?;
            let value = r.optional_ref(fields[3])?.ok_or(Error::Conflict)?;
            Some(r.artifact(value)?)
        } else {
            None
        };
        let step = self
            .runtime
            .step(&checkpoint, reply.as_deref(), live.reserved, live.limit);
        let step = match step {
            Ok(step) => step,
            Err(error) => {
                self.finish(
                    cell,
                    &loaded,
                    None,
                    Some(&error.to_string()),
                    live.charged
                        .checked_add(live.reserved)
                        .ok_or(Error::Budget)?,
                    live.used,
                )?;
                return Err(error);
            }
        };
        let used = match &step {
            RuntimeStep::Done { used, .. }
            | RuntimeStep::Yield { used, .. }
            | RuntimeStep::Act { used, .. }
            | RuntimeStep::Event { used, .. } => *used,
        };
        let delta = used.checked_sub(live.used).ok_or(Error::Conflict)?;
        if delta > live.reserved {
            return Err(Error::Budget);
        }
        live.charged = live.charged.checked_add(delta).ok_or(Error::Budget)?;
        live.reserved = 0;
        live.used = used;
        live.pending = None;
        let mut b = Builder::new();
        match step {
            RuntimeStep::Done { result, .. } => Ok(Progress::Complete(self.finish(
                cell,
                &loaded,
                Some(result),
                None,
                live.charged,
                used,
            )?)),
            RuntimeStep::Yield { checkpoint, .. } => {
                live.checkpoint = b.artifact(checkpoint, "application/x-rune-checkpoint")?;
                Ok(Progress::Advanced(self.save_live(
                    cell,
                    &loaded,
                    &live,
                    "$yield",
                    b,
                    &[],
                )?))
            }
            RuntimeStep::Act {
                tag,
                arguments,
                checkpoint,
                ..
            } => {
                live.checkpoint = b.artifact(checkpoint, "application/x-rune-checkpoint")?;
                let pending = rec::operation(
                    &mut b,
                    cell,
                    loaded.head,
                    &live,
                    &loaded.snapshot,
                    tag,
                    arguments,
                )?;
                let record = pending.record;
                live.pending = Some(pending);
                Ok(Progress::Advanced(self.save_live(
                    cell,
                    &loaded,
                    &live,
                    "$act",
                    b,
                    &[record],
                )?))
            }
            RuntimeStep::Event { .. } => {
                self.finish(
                    cell,
                    &loaded,
                    None,
                    Some("event subscriptions are unsupported by this local adapter"),
                    live.charged,
                    used,
                )?;
                Err(Error::Unsupported)
            }
        }
    }
    pub fn begin_attempt(&self, cell: Particle, operation: Particle) -> Result<Attempt, Error> {
        let loaded = self.load(cell)?;
        if loaded.snapshot.lifecycle != Lifecycle::Active {
            return Err(Error::Lifecycle);
        }
        let mut live = loaded.live.clone().ok_or(Error::Missing)?;
        let p = live.pending.as_mut().ok_or(Error::Missing)?;
        if p.id != operation {
            return Err(Error::Conflict);
        }
        if p.stage != "pending" {
            return Err(Error::UnknownOutcome);
        }
        let mut r = Reader::new(&self.graph, 20_000);
        let (_, allowed) = rec::policy_values(&mut r, loaded.snapshot.authority_policy)?;
        if let Err(error) = self.ward.authorize(&crate::Authorization {
            cell,
            operation,
            policy: loaded.snapshot.authority_policy,
            epoch: loaded.snapshot.epoch,
            act: p.tag,
            allowed_acts: &allowed,
        }) {
            let mut b = Builder::new();
            let reason = b.artifact(error.to_string().into_bytes(), "text/plain")?;
            let decision = b.values(
                "cell/local-denial/1",
                &[
                    Ref(cell),
                    Ref(p.record),
                    Ref(loaded.snapshot.authority_policy),
                    Uint(loaded.snapshot.epoch),
                    Ref(reason),
                ],
            )?;
            self.save_live(cell, &loaded, &live, "$denied", b, &[decision])?;
            return Err(error);
        }
        let arguments = r.artifact(p.arguments)?;
        let mut b = Builder::new();
        let id = b.values(
            "cell/attempt-id/1",
            &[Ref(p.id), Uint(0), Uint(loaded.snapshot.epoch)],
        )?;
        let contract = rec::named(&mut b, "local-executor/manual-reconciliation/1")?;
        let decision = b.values(
            "cell/local-authorization/1",
            &[
                Ref(cell),
                Ref(p.record),
                Ref(loaded.snapshot.authority_policy),
                Uint(loaded.snapshot.epoch),
            ],
        )?;
        let record = b.values(
            "cell/attempt/1",
            &[
                Ref(id),
                Ref(p.id),
                Uint(0),
                Uint(loaded.snapshot.epoch),
                Ref(decision),
                Optional(None),
                Ref(contract),
            ],
        )?;
        let result = Attempt {
            operation: p.id,
            attempt: id,
            tag: p.tag,
            arguments,
        };
        p.attempt = Some(record);
        p.stage = "attempt-recorded";
        self.save_live(cell, &loaded, &live, "$attempt", b, &[record])?;
        Ok(result)
    }
    pub fn record_outcome(
        &self,
        cell: Particle,
        operation: Particle,
        attempt: Particle,
        value: Vec<u8>,
    ) -> Result<Head, Error> {
        self.settle_outcome(cell, operation, attempt, Ok(value))
    }
    pub fn record_failure(
        &self,
        cell: Particle,
        operation: Particle,
        attempt: Particle,
        reason: String,
    ) -> Result<Head, Error> {
        if reason.len() > 4096 {
            return Err(Error::Budget);
        }
        self.settle_outcome(cell, operation, attempt, Err(reason))
    }
    fn settle_outcome(
        &self,
        cell: Particle,
        operation: Particle,
        attempt: Particle,
        result: Result<Vec<u8>, String>,
    ) -> Result<Head, Error> {
        let loaded = self.load(cell)?;
        let mut b = Builder::new();
        let (disposition, value, evidence) = match result {
            Ok(value) => {
                self.runtime.validate_value(&value)?;
                (
                    b.variant("outcome/succeeded", &[])?,
                    Some(b.artifact(value, "application/x-rune-noun")?),
                    None,
                )
            }
            Err(reason) => (
                b.variant("outcome/failed-definite", &[])?,
                None,
                Some(b.artifact(reason.into_bytes(), "text/plain")?),
            ),
        };
        let outcome = b.values(
            "cell/outcome/1",
            &[
                Ref(operation),
                Ref(attempt),
                Raw(disposition),
                Optional(value),
                Optional(evidence),
                Optional(None),
            ],
        )?;
        let nonce = b.values("cell/outcome-request/1", &[Ref(operation), Ref(attempt)])?;
        let (event, request) = rec::event(
            &mut b,
            cell,
            nonce,
            "$outcome",
            outcome,
            loaded.live.as_ref().and_then(|v| v.context),
            loaded.snapshot.authority_policy,
        )?;
        if let Some(head) = self.graph.resolve(cell, request)? {
            let mut r = Reader::new(&self.graph, 20_000);
            let c = r.record(head.commit, "cell/commit/1", 10)?;
            let events = r.list(c[5], 1)?;
            let ev = r.reference(*events.first().ok_or(Error::Conflict)?)?;
            let ev = r.record(ev, "cell/event/1", 10)?;
            return if r.reference(ev[4])? == outcome {
                Ok(head)
            } else {
                Err(Error::Conflict)
            };
        }
        let mut live = loaded.live.clone().ok_or(Error::Missing)?;
        let p = live.pending.as_mut().ok_or(Error::Missing)?;
        let mut r = Reader::new(&self.graph, 20_000);
        let attempt_record = p.attempt.ok_or(Error::Conflict)?;
        let a = r.record(attempt_record, "cell/attempt/1", 7)?;
        if p.id != operation || r.reference(a[0])? != attempt || p.stage != "attempt-recorded" {
            return Err(Error::Conflict);
        }
        p.outcome = Some(outcome);
        p.stage = "resolved";
        let mut snapshot = loaded.snapshot.clone();
        let mut records = rec::write_live(&mut b, &mut snapshot, &live, loaded.head)?;
        records.push(outcome);
        self.publish(cell, &loaded, snapshot, b, (event, request), &records)
    }
}
