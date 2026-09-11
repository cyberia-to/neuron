//! Local lifecycle and execution coordinator.
use crate::{Error, GraphPort, Live, RuntimeInput, RuntimePort, records as rec};
use cell_model::{Builder, Head, Lifecycle, Particle, Reader, Snapshot, Value::*};

pub struct Engine<G, R> {
    pub graph: G,
    pub runtime: R,
    pub ward: Box<dyn crate::WardPort>,
}
#[derive(Debug)]
pub struct Inspection {
    pub cell: Particle,
    pub head: Head,
    pub snapshot: Snapshot,
    pub state: Vec<u8>,
    pub live: Option<Live>,
    pub fault: Option<String>,
}
pub(crate) struct Loaded {
    pub head: Head,
    pub snapshot_id: Particle,
    pub snapshot: Snapshot,
    pub live: Option<Live>,
}
#[derive(Debug, Clone)]
pub struct Config {
    pub step_limit: u64,
    pub allowed_acts: Vec<u64>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            step_limit: 1_000_000,
            allowed_acts: Vec::new(),
        }
    }
}

impl<G: GraphPort, R: RuntimePort> Engine<G, R> {
    pub fn new(graph: G, runtime: R) -> Self {
        Self::with_ward(graph, runtime, crate::DenyAll)
    }
    pub fn with_ward(graph: G, runtime: R, ward: impl crate::WardPort + 'static) -> Self {
        Self {
            graph,
            runtime,
            ward: Box::new(ward),
        }
    }
    pub fn create(
        &self,
        source: Vec<u8>,
        initial: Vec<u8>,
        nonce: [u8; 32],
        mut config: Config,
    ) -> Result<Particle, Error> {
        if config.step_limit == 0
            || config.step_limit > 1_000_000
            || config.allowed_acts.len() > 256
        {
            return Err(Error::Budget);
        }
        config.allowed_acts.sort_unstable();
        config.allowed_acts.dedup();
        self.runtime.start(RuntimeInput {
            source: &source,
            state: &initial,
            event: &initial,
            context: None,
            step_limit: config.step_limit,
        })?;
        let mut b = Builder::new();
        let policy = rec::policy(&mut b, config.step_limit, &config.allowed_acts)?;
        let definition = rec::definition(&mut b, source, policy, config.step_limit)?;
        let profile = rec::profile(&mut b)?;
        let state = b.artifact(initial, "application/x-rune-noun")?;
        let empty = b.empty_map()?;
        let snapshot = Snapshot {
            definition,
            application_state: state,
            lifecycle: Lifecycle::Installed,
            authority_policy: policy,
            profile,
            epoch: 0,
            inbox: empty,
            continuations: empty,
            outbox: empty,
            subscriptions: empty,
            management: empty,
        };
        let snapshot = snapshot.encode(&mut b)?;
        let nonce = b.nonce(nonce)?;
        let cell = b.values(
            "cell/birth/1",
            &[
                Uint(1),
                Raw(nonce),
                Ref(policy),
                Ref(definition),
                Ref(profile),
                Ref(snapshot),
                Optional(None),
            ],
        )?;
        self.graph.commit(
            cell,
            cell,
            None,
            Head {
                index: 0,
                commit: cell,
            },
            b,
            None,
        )?;
        if self.load(cell)?.snapshot.lifecycle == Lifecycle::Installed {
            self.manage(cell, Lifecycle::Active)?;
        }
        Ok(cell)
    }
    pub fn inspect(&self, cell: Particle) -> Result<Inspection, Error> {
        let loaded = self.load(cell)?;
        let state = Reader::new(&self.graph, 20_000).artifact(loaded.snapshot.application_state)?;
        let mut fault = None;
        if loaded.head.index > 0 {
            let mut r = Reader::new(&self.graph, 20_000);
            let commit = r.record(loaded.head.commit, "cell/commit/1", 10)?;
            for event in r.list(commit[5], 1)? {
                let event = r.reference(event)?;
                let fields = r.record(event, "cell/event/1", 10)?;
                if r.text(fields[3])? == "$fault" {
                    let payload = r.reference(fields[4])?;
                    fault =
                        Some(String::from_utf8(r.artifact(payload)?).map_err(|_| Error::Conflict)?);
                }
            }
        }
        Ok(Inspection {
            cell,
            head: loaded.head,
            snapshot: loaded.snapshot,
            state,
            live: loaded.live,
            fault,
        })
    }
    pub(crate) fn load(&self, cell: Particle) -> Result<Loaded, Error> {
        let head = self.graph.head(cell)?.ok_or(Error::Missing)?;
        let mut r = Reader::new(&self.graph, 50_000);
        if head.index > 0 {
            let fields = r.record(head.commit, "cell/commit/1", 10)?;
            if r.reference(fields[0])? != cell || r.uint(fields[1])? != head.index {
                return Err(Error::Conflict);
            }
        } else if head.commit != cell {
            return Err(Error::Conflict);
        }
        let (snapshot_id, snapshot) = rec::snapshot_at(&mut r, head)?;
        if snapshot.profile != rec::profile(&mut Builder::new())? {
            return Err(Error::Unsupported);
        }
        let live = rec::read_live(&mut r, &snapshot)?;
        Ok(Loaded {
            head,
            snapshot_id,
            snapshot,
            live,
        })
    }
    pub fn submit(
        &self,
        cell: Particle,
        nonce: [u8; 32],
        input: Vec<u8>,
        context: Option<Particle>,
    ) -> Result<Head, Error> {
        let loaded = self.load(cell)?;
        let mut b = Builder::new();
        let payload = b.artifact(input.clone(), "application/x-rune-noun")?;
        let (event, request) = rec::event(
            &mut b,
            cell,
            nonce,
            "main",
            payload,
            context,
            loaded.snapshot.authority_policy,
        )?;
        if let Some(head) = self.graph.resolve(cell, request)? {
            let mut r = Reader::new(&self.graph, 10_000);
            let commit = r.record(head.commit, "cell/commit/1", 10)?;
            let events = r.list(commit[5], 1)?;
            return if events.len() == 1 && r.reference(events[0])? == event {
                Ok(head)
            } else {
                Err(Error::Conflict)
            };
        }
        if loaded.snapshot.lifecycle != Lifecycle::Active {
            return Err(Error::Lifecycle);
        }
        if loaded.live.is_some() {
            return Err(Error::Busy);
        }
        if let Some(id) = context {
            self.graph.get(&id)?;
        }
        let mut r = Reader::new(&self.graph, 20_000);
        let definition = r.record(loaded.snapshot.definition, "cell/definition/1", 11)?;
        let source_id = r.reference(definition[2])?;
        let source = r.artifact(source_id)?;
        let state = r.artifact(loaded.snapshot.application_state)?;
        let (limit, _) = rec::policy_values(&mut r, loaded.snapshot.authority_policy)?;
        let checkpoint = self.runtime.start(RuntimeInput {
            source: &source,
            state: &state,
            event: &input,
            context,
            step_limit: limit,
        })?;
        let checkpoint = b.artifact(checkpoint, "application/x-rune-checkpoint")?;
        let live = Live {
            event,
            context,
            checkpoint,
            charged: 0,
            reserved: 0,
            used: 0,
            limit,
            pending: None,
        };
        let mut snapshot = loaded.snapshot.clone();
        let records = rec::write_live(&mut b, &mut snapshot, &live, loaded.head)?;
        self.publish(cell, &loaded, snapshot, b, (event, request), &records)
    }
    pub fn manage(&self, cell: Particle, lifecycle: Lifecycle) -> Result<Head, Error> {
        let loaded = self.load(cell)?;
        if loaded.snapshot.lifecycle == lifecycle {
            return Ok(loaded.head);
        }
        let allowed = matches!(
            (loaded.snapshot.lifecycle, lifecycle),
            (Lifecycle::Installed | Lifecycle::Paused, Lifecycle::Active)
                | (Lifecycle::Active, Lifecycle::Paused)
                | (
                    Lifecycle::Installed | Lifecycle::Active | Lifecycle::Paused,
                    Lifecycle::Retiring
                )
                | (Lifecycle::Retiring, Lifecycle::Retired)
        );
        if !allowed {
            return Err(Error::Lifecycle);
        }
        if matches!(lifecycle, Lifecycle::Retiring | Lifecycle::Retired) && loaded.live.is_some() {
            return Err(Error::Busy);
        }
        let mut snapshot = loaded.snapshot.clone();
        snapshot.lifecycle = lifecycle;
        let mut b = Builder::new();
        let payload = lifecycle.encode(&mut b)?;
        let (event, request) = rec::system_event(
            &mut b,
            cell,
            loaded.head,
            lifecycle.name(),
            payload,
            snapshot.authority_policy,
        )?;
        self.publish(cell, &loaded, snapshot, b, (event, request), &[])
    }
    pub(crate) fn publish(
        &self,
        cell: Particle,
        loaded: &Loaded,
        snapshot: Snapshot,
        mut b: Builder,
        event_request: (Particle, Particle),
        records: &[Particle],
    ) -> Result<Head, Error> {
        let (event, request) = event_request;
        let after = snapshot.encode(&mut b)?;
        let next = rec::commit(
            &mut b,
            cell,
            loaded.head,
            (loaded.snapshot_id, after),
            event,
            records,
            &snapshot,
        )?;
        self.graph.commit(
            cell,
            request,
            Some(loaded.head),
            next,
            b,
            Some((request, event)),
        )
    }
    pub(crate) fn save_live(
        &self,
        cell: Particle,
        loaded: &Loaded,
        live: &Live,
        kind: &str,
        mut b: Builder,
        extra: &[Particle],
    ) -> Result<Head, Error> {
        let mut snapshot = loaded.snapshot.clone();
        let mut records = rec::write_live(&mut b, &mut snapshot, live, loaded.head)?;
        records.extend_from_slice(extra);
        if let Some(old) = &loaded.live
            && old.checkpoint != live.checkpoint
            && let Some(consumed) = rec::consumed(&mut b, old)?
        {
            records.push(consumed);
        }
        let (event, request) = rec::system_event(
            &mut b,
            cell,
            loaded.head,
            kind,
            live.event,
            snapshot.authority_policy,
        )?;
        self.publish(cell, loaded, snapshot, b, (event, request), &records)
    }
    pub(crate) fn finish(
        &self,
        cell: Particle,
        loaded: &Loaded,
        result: Option<Vec<u8>>,
        fault: Option<&str>,
        charged: u64,
        used: u64,
    ) -> Result<Head, Error> {
        let live = loaded.live.as_ref().ok_or(Error::Missing)?;
        let mut b = Builder::new();
        let mut snapshot = loaded.snapshot.clone();
        if let Some(result) = result {
            self.runtime.validate_value(&result)?;
            snapshot.application_state = b.artifact(result, "application/x-rune-noun")?;
        }
        let payload = if let Some(fault) = fault {
            b.artifact(fault.as_bytes().to_vec(), "text/plain")?
        } else {
            snapshot.application_state
        };
        snapshot.inbox = rec::inbox(
            &mut b,
            live.event,
            if fault.is_some() {
                "cancelled"
            } else {
                "completed"
            },
            loaded.head.index.checked_add(1).ok_or(Error::Budget)?,
        )?;
        snapshot.continuations = b.empty_map()?;
        snapshot.outbox = b.empty_map()?;
        let usage = b.values(
            "cell/local-resources/1",
            &[Uint(charged), Uint(0), Uint(used), Uint(live.limit)],
        )?;
        let mut records = vec![usage];
        if let Some(consumed) = rec::consumed(&mut b, live)? {
            records.push(consumed);
        }
        let (event, request) = rec::system_event(
            &mut b,
            cell,
            loaded.head,
            if fault.is_some() {
                "$fault"
            } else {
                "$complete"
            },
            payload,
            snapshot.authority_policy,
        )?;
        self.publish(cell, loaded, snapshot, b, (event, request), &records)
    }
    pub fn cancel(&self, cell: Particle) -> Result<Head, Error> {
        let loaded = self.load(cell)?;
        let live = loaded.live.as_ref().ok_or(Error::Missing)?;
        if live
            .pending
            .as_ref()
            .is_some_and(|p| p.stage == "attempt-recorded")
        {
            return Err(Error::UnknownOutcome);
        }
        self.finish(
            cell,
            &loaded,
            None,
            Some("cancelled by local authority"),
            live.charged
                .checked_add(live.reserved)
                .ok_or(Error::Budget)?,
            live.used,
        )
    }
}
