//! A single key subject with durable, independently progressing programs and jobs.
use crate::{Error, GraphPort, RuntimeInput, RuntimePort};
use neuron_model::execution::{self as model, Invocation, NeuronState, Prog, Status};
use neuron_model::{Builder, Head, Lifecycle, NeuronId, Particle, Reader, Value::*};
use std::collections::BTreeMap;

/// The host must verify the supported identity and current policy before signing.
/// `statement` is the canonical digest binding every field of this request.
pub struct Action<'a> {
    pub neuron: NeuronId,
    pub network: Particle,
    pub policy: Particle,
    pub epoch: u64,
    pub prog: Option<Particle>,
    pub invocation: Option<Particle>,
    pub act: Option<u64>,
    pub kind: &'a str,
    pub statement: Particle,
    pub allowed_acts: &'a [u64],
}
pub trait Authority: Send + Sync {
    fn authorize(&self, action: &Action<'_>) -> Result<Vec<u8>, Error>;
    /// Hold the current policy decision through a local linearization point.
    /// Adapters without this guarantee must fail closed.
    fn with_current(
        &self,
        _action: &Action<'_>,
        _commit: &mut dyn FnMut() -> Result<(), Error>,
    ) -> Result<(), Error> {
        Err(Error::Unsupported)
    }
}
#[derive(Clone, Copy)]
pub(crate) struct Scope<'a> {
    pub kind: &'a str,
    pub prog: Option<Particle>,
    pub invocation: Option<Particle>,
}
pub struct NoAuthority;
impl Authority for NoAuthority {
    fn authorize(&self, _: &Action<'_>) -> Result<Vec<u8>, Error> {
        Err(Error::Denied)
    }
}

pub struct Neuron<G, R, A = NoAuthority> {
    pub graph: G,
    pub runtime: R,
    pub authority: A,
}
#[derive(Clone, Debug)]
pub struct View {
    pub head: Head,
    pub root: Particle,
    pub state: NeuronState,
}
pub(crate) struct Publication {
    pub neuron: NeuronId,
    pub request: Particle,
    pub head: Head,
    pub content: Builder,
    pub event: Particle,
    pub allowed_acts: Vec<u64>,
}
#[derive(Clone, Debug)]
pub struct ProgramConfig {
    pub step_limit: u64,
    pub max_inflight: u64,
    pub allowed_acts: Vec<u64>,
}
impl Default for ProgramConfig {
    fn default() -> Self {
        Self {
            step_limit: 1_000_000,
            max_inflight: 1,
            allowed_acts: Vec::new(),
        }
    }
}
pub struct Admission {
    pub prog: Particle,
    pub nonce: [u8; 32],
    pub input: Vec<u8>,
    pub context: Option<Particle>,
    pub parent: Option<Particle>,
    pub allowance: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub invocation: Particle,
    pub head: Head,
}

impl<G: GraphPort, R: RuntimePort> Neuron<G, R> {
    pub fn new(graph: G, runtime: R) -> Self {
        Self {
            graph,
            runtime,
            authority: NoAuthority,
        }
    }
}
impl<G: GraphPort, R: RuntimePort, A: Authority> Neuron<G, R, A> {
    pub fn with_authority(graph: G, runtime: R, authority: A) -> Self {
        Self {
            graph,
            runtime,
            authority,
        }
    }
    pub fn inspect(&self, neuron: NeuronId) -> Result<View, Error> {
        let head = self.graph.head(neuron)?.ok_or(Error::Missing)?;
        let mut r = Reader::new(&self.graph, 2_000_000);
        let f = if head.index == 0 {
            r.record(head.commit, "neuron/activation/1", 3)?
        } else {
            r.record(head.commit, "neuron/commit/1", 8)?
        };
        if r.reference(f[0])? != neuron || head.index > 0 && r.uint(f[1])? != head.index {
            return Err(Error::Conflict);
        }
        let root = r.reference(f[if head.index == 0 { 1 } else { 4 }])?;
        let state = NeuronState::read(&mut r, root)?;
        if state.neuron != neuron {
            return Err(Error::Conflict);
        }
        Ok(View { head, root, state })
    }
    pub fn state(&self, neuron: NeuronId, prog: Particle) -> Result<Vec<u8>, Error> {
        let view = self.inspect(neuron)?;
        let prog = view.state.progs.get(&prog).ok_or(Error::Missing)?;
        Ok(model::read_artifact(
            &mut Reader::new(&self.graph, 50_000),
            prog.state,
        )?)
    }
    pub fn activate(
        &self,
        neuron: NeuronId,
        network: Particle,
        policy: Particle,
        limit: u64,
    ) -> Result<Head, Error> {
        if limit == 0 {
            return Err(Error::Budget);
        }
        let state = NeuronState {
            neuron,
            network,
            policy,
            epoch: 0,
            limit,
            charged: 0,
            held: 0,
            progs: BTreeMap::new(),
            invocations: BTreeMap::new(),
            imports: BTreeMap::new(),
            cursor: 0,
            writer_generation: 0,
            worker: None,
        };
        let mut b = Builder::new();
        let root = state.encode(&mut b)?;
        if let Some(head) = self.graph.resolve(neuron, neuron)? {
            let mut r = Reader::new(&self.graph, 20_000);
            let f = r.record(head.commit, "neuron/activation/1", 3)?;
            return if r.reference(f[1])? == root {
                Ok(head)
            } else {
                Err(Error::Conflict)
            };
        }
        let auth = self.evidence(
            &mut b,
            &state,
            Scope {
                kind: "activate",
                prog: None,
                invocation: None,
            },
            None,
            root,
            &[],
        )?;
        let commit = b.values("neuron/activation/1", &[Ref(neuron), Ref(root), Ref(auth)])?;
        let action = Action {
            neuron,
            network,
            policy,
            epoch: 0,
            prog: None,
            invocation: None,
            act: None,
            kind: "activate",
            statement: commit,
            allowed_acts: &[],
        };
        let mut content = Some(b);
        let mut result = None;
        self.authority.with_current(&action, &mut || {
            result = Some(self.graph.commit(
                neuron,
                neuron,
                None,
                Head { index: 0, commit },
                content.take().ok_or(Error::Denied)?,
                Some((neuron, root)),
            )?);
            Ok(())
        })?;
        result.ok_or(Error::Denied)
    }
    pub(crate) fn evidence(
        &self,
        b: &mut Builder,
        state: &NeuronState,
        scope: Scope<'_>,
        act: Option<u64>,
        statement: Particle,
        allowed: &[u64],
    ) -> Result<Particle, Error> {
        let Scope {
            kind,
            prog,
            invocation,
        } = scope;
        let statement = b.values(
            "neuron/authority-statement/1",
            &[
                Ref(state.neuron),
                Ref(state.network),
                Ref(state.policy),
                Uint(state.epoch),
                Optional(prog),
                Optional(invocation),
                Uint(u64::from(act.is_some())),
                Uint(act.unwrap_or(0)),
                Text(kind),
                Ref(statement),
            ],
        )?;
        let evidence = self.authority.authorize(&Action {
            neuron: state.neuron,
            network: state.network,
            policy: state.policy,
            epoch: state.epoch,
            prog,
            invocation,
            act,
            kind,
            statement,
            allowed_acts: allowed,
        })?;
        if evidence.is_empty() || evidence.len() > 65_536 {
            return Err(Error::Denied);
        }
        let evidence = b.blob(evidence)?;
        Ok(b.values(
            "neuron/authorization/1",
            &[
                Ref(state.neuron),
                Ref(state.network),
                Ref(state.policy),
                Uint(state.epoch),
                Ref(statement),
                Ref(evidence),
            ],
        )?)
    }
    pub(crate) fn request(
        b: &mut Builder,
        neuron: NeuronId,
        nonce: [u8; 32],
    ) -> Result<Particle, Error> {
        let nonce = b.nonce(nonce)?;
        Ok(b.values("neuron/request-id/1", &[Ref(neuron), Raw(nonce)])?)
    }
    pub(crate) fn committed_event(&self, head: Head) -> Result<Particle, Error> {
        let mut r = Reader::new(&self.graph, 20_000);
        let f = r.record(head.commit, "neuron/commit/1", 8)?;
        Ok(r.reference(f[5])?)
    }
    pub(crate) fn retry(
        &self,
        neuron: NeuronId,
        request: Particle,
        event: Particle,
    ) -> Result<Option<Head>, Error> {
        self.graph
            .resolve(neuron, request)?
            .map(|h| {
                if self.committed_event(h)? == event {
                    Ok(h)
                } else {
                    Err(Error::Conflict)
                }
            })
            .transpose()
    }
    pub(crate) fn publish(
        &self,
        old: &View,
        state: NeuronState,
        b: Builder,
        event_request: (Particle, Particle),
        scope: Scope<'_>,
        records: &[Particle],
    ) -> Result<Head, Error> {
        let p = self.prepare(old, state, b, event_request, scope, records)?;
        let allowed = p.allowed_acts.clone();
        let action = Action {
            neuron: p.neuron,
            network: old.state.network,
            policy: old.state.policy,
            epoch: old.state.epoch,
            prog: scope.prog,
            invocation: scope.invocation,
            act: None,
            kind: scope.kind,
            statement: p.head.commit,
            allowed_acts: &allowed,
        };
        let mut publication = Some(p);
        let mut result = None;
        self.authority.with_current(&action, &mut || {
            let p = publication.take().ok_or(Error::Denied)?;
            result = Some(self.graph.commit(
                p.neuron,
                p.request,
                Some(old.head),
                p.head,
                p.content,
                Some((p.request, p.event)),
            )?);
            Ok(())
        })?;
        result.ok_or(Error::Denied)
    }
    pub(crate) fn prepare(
        &self,
        old: &View,
        mut state: NeuronState,
        mut b: Builder,
        event_request: (Particle, Particle),
        scope: Scope<'_>,
        records: &[Particle],
    ) -> Result<Publication, Error> {
        let (event, request) = event_request;
        let Scope {
            kind,
            prog,
            invocation,
        } = scope;
        state.held = state
            .invocations
            .values()
            .filter(|j| !j.status.terminal())
            .try_fold(0u64, |n, j| {
                n.checked_add(j.available()?)
                    .ok_or(neuron_model::Error::Limit)
            })?;
        let after = state.encode(&mut b)?;
        let statement = b.values(
            "neuron/proposal/1",
            &[
                Ref(state.neuron),
                Ref(old.head.commit),
                Ref(after),
                Ref(event),
                Refs(records),
            ],
        )?;
        // Recording an old result or stopping work does not execute the prog's
        // revoked capabilities. Subject, network, policy and prog scope still
        // require a current management grant.
        let allowed = if matches!(
            kind,
            "reconcile"
                | "cancel"
                | "archive"
                | "paused"
                | "retiring"
                | "retired"
                | "recover-reservation"
        ) {
            &[][..]
        } else {
            prog.and_then(|id| state.progs.get(&id))
                .map(|p| p.allowed_acts.as_slice())
                .unwrap_or(&[])
        };
        let auth = self.evidence(
            &mut b,
            &old.state,
            Scope {
                kind,
                prog,
                invocation,
            },
            None,
            statement,
            allowed,
        )?;
        let index = old.head.index.checked_add(1).ok_or(Error::Budget)?;
        let commit = b.values(
            "neuron/commit/1",
            &[
                Ref(state.neuron),
                Uint(index),
                Ref(old.head.commit),
                Ref(old.root),
                Ref(after),
                Ref(event),
                Refs(records),
                Ref(auth),
            ],
        )?;
        Ok(Publication {
            neuron: state.neuron,
            request,
            head: Head { index, commit },
            content: b,
            event,
            allowed_acts: allowed.to_vec(),
        })
    }
    pub(crate) fn transition(
        &self,
        old: &View,
        state: NeuronState,
        mut b: Builder,
        scope: Scope<'_>,
        payload: Option<Particle>,
        records: &[Particle],
    ) -> Result<Head, Error> {
        let Scope {
            kind,
            prog,
            invocation,
        } = scope;
        let event = b.values(
            "neuron/transition/1",
            &[
                Ref(state.neuron),
                Ref(old.head.commit),
                Text(kind),
                Optional(payload),
            ],
        )?;
        self.publish(
            old,
            state,
            b,
            (event, event),
            Scope {
                kind,
                prog,
                invocation,
            },
            records,
        )
    }
    pub fn install(
        &self,
        neuron: NeuronId,
        nonce: [u8; 32],
        source: Vec<u8>,
        initial: Vec<u8>,
        mut config: ProgramConfig,
    ) -> Result<Particle, Error> {
        config.allowed_acts.sort_unstable();
        config.allowed_acts.dedup();
        let mut b = Builder::new();
        let nonce_value = b.nonce(nonce)?;
        let id = b.values("neuron/prog-id/1", &[Ref(neuron), Raw(nonce_value)])?;
        let request = Self::request(&mut b, neuron, nonce)?;
        let source_id = model::artifact(&mut b, source.clone(), "application/x-rune")?;
        let state = model::artifact(&mut b, initial.clone(), "application/x-rune-noun")?;
        let prog = Prog {
            source: source_id,
            state,
            revision: 0,
            lifecycle: Lifecycle::Active,
            step_limit: config.step_limit,
            max_inflight: config.max_inflight,
            allowed_acts: config.allowed_acts,
        };
        let definition = prog.encode(&mut b)?;
        let event = b.values("neuron/install/1", &[Ref(id), Ref(definition)])?;
        if self.retry(neuron, request, event)?.is_some() {
            return Ok(id);
        }
        let old = self.inspect(neuron)?;
        let mut state = old.state.clone();
        if state.progs.len() >= model::MAX_PROGS || state.progs.contains_key(&id) {
            return Err(Error::Budget);
        }
        self.runtime.start(RuntimeInput {
            source: &source,
            state: &initial,
            event: &initial,
            context: None,
            step_limit: prog.step_limit,
        })?;
        state.progs.insert(id, prog);
        self.publish(
            &old,
            state,
            b,
            (event, request),
            Scope {
                kind: "install",
                prog: Some(id),
                invocation: None,
            },
            &[definition],
        )?;
        Ok(id)
    }
    pub fn lookup_admission(&self, neuron: NeuronId, nonce: [u8;32]) -> Result<Option<Receipt>, Error> {
        let mut b = Builder::new();
        let request = Self::request(&mut b, neuron, nonce)?;
        let Some(head) = self.graph.resolve(neuron,request)? else { return Ok(None); };
        let invocation = self.committed_event(head)?;
        let mut reader = Reader::new(&self.graph,50_000);
        let fields = reader.record(invocation,"neuron/event/1",8)?;
        if reader.reference(fields[0])? != neuron || fields[2] != b.nonce(nonce)? { return Err(Error::Conflict); }
        Ok(Some(Receipt{invocation,head}))
    }
    pub fn submit(&self, neuron: NeuronId, admission: Admission) -> Result<Receipt, Error> {
        let mut b = Builder::new();
        let request = Self::request(&mut b, neuron, admission.nonce)?;
        let prior = self.graph.resolve(neuron, request)?;
        let old = self.inspect(neuron)?;
        let epoch = if let Some(head) = prior {
            let event = self.committed_event(head)?;
            let mut r = Reader::new(&self.graph, 50_000);
            let f = r.record(event, "neuron/event/1", 8)?;
            r.uint(f[5])?
        } else {
            old.state.epoch
        };
        let input = model::artifact(&mut b, admission.input.clone(), "application/x-rune-noun")?;
        let nonce = b.nonce(admission.nonce)?;
        let event = b.values(
            "neuron/event/1",
            &[
                Ref(neuron),
                Ref(admission.prog),
                Raw(nonce),
                Ref(input),
                Optional(admission.context),
                Uint(epoch),
                Optional(admission.parent),
                Uint(admission.allowance),
            ],
        )?;
        if let Some(head) = self.retry(neuron, request, event)? {
            return Ok(Receipt {
                invocation: event,
                head,
            });
        }
        let mut state = old.state.clone();
        let prog = state.progs.get(&admission.prog).ok_or(Error::Missing)?;
        if prog.lifecycle != Lifecycle::Active {
            return Err(Error::Lifecycle);
        }
        if admission.allowance == 0
            || admission.allowance > prog.step_limit
            || state.invocations.len() >= model::MAX_INVOCATIONS
            || state
                .invocations
                .values()
                .filter(|j| j.prog == admission.prog && !j.status.terminal())
                .count()
                >= prog.max_inflight as usize
        {
            return Err(Error::Budget);
        }
        let mut r = Reader::new(&self.graph, 50_000);
        let source = model::read_artifact(&mut r, prog.source)?;
        let initial = model::read_artifact(&mut r, prog.state)?;
        if let Some(context) = admission.context {
            r.content(context)?;
        }
        let checkpoint = self.runtime.start(RuntimeInput {
            source: &source,
            state: &initial,
            event: &admission.input,
            context: admission.context,
            step_limit: admission.allowance,
        })?;
        let checkpoint = model::artifact(&mut b, checkpoint, "application/x-rune-checkpoint")?;
        let job = Invocation {
            prog: admission.prog,
            code: prog.source,
            input,
            context: admission.context,
            base_revision: prog.revision,
            checkpoint: Some(checkpoint),
            status: Status::Running,
            limit: admission.allowance,
            charged: 0,
            reserved: 0,
            used: 0,
            pending: None,
            result: None,
            fault: None,
            parent: admission.parent,
            delegated: 0,
            children: Vec::new(),
            epoch,
            ordinal: 0,
        };
        if let Some(parent) = admission.parent {
            let p = state.invocations.get_mut(&parent).ok_or(Error::Missing)?;
            if p.status.terminal()
                || p.status == Status::Joining
                || p.epoch != epoch
                || p.children.len() >= model::MAX_CHILDREN
                || p.available()?
                    .checked_sub(p.reserved)
                    .is_none_or(|n| n < job.limit)
            {
                return Err(Error::Budget);
            }
            p.delegated = p.delegated.checked_add(job.limit).ok_or(Error::Budget)?;
            p.children.push(event);
        } else if state
            .charged
            .checked_add(state.held)
            .and_then(|v| v.checked_add(job.limit))
            .is_none_or(|v| v > state.limit)
        {
            return Err(Error::Budget);
        }
        state.invocations.insert(event, job);
        let head = self.publish(
            &old,
            state,
            b,
            (event, request),
            Scope {
                kind: "admit",
                prog: Some(admission.prog),
                invocation: Some(event),
            },
            &[],
        )?;
        Ok(Receipt {
            invocation: event,
            head,
        })
    }
    pub fn manage(
        &self,
        neuron: NeuronId,
        prog: Particle,
        lifecycle: Lifecycle,
    ) -> Result<Head, Error> {
        let old = self.inspect(neuron)?;
        let mut state = old.state.clone();
        let p = state.progs.get_mut(&prog).ok_or(Error::Missing)?;
        if p.lifecycle == lifecycle {
            return Ok(old.head);
        }
        if !matches!(
            (p.lifecycle, lifecycle),
            (Lifecycle::Installed | Lifecycle::Paused, Lifecycle::Active)
                | (Lifecycle::Active, Lifecycle::Paused)
                | (
                    Lifecycle::Installed | Lifecycle::Active | Lifecycle::Paused,
                    Lifecycle::Retiring
                )
                | (Lifecycle::Retiring, Lifecycle::Retired)
        ) {
            return Err(Error::Lifecycle);
        }
        if matches!(lifecycle, Lifecycle::Retiring | Lifecycle::Retired)
            && state
                .invocations
                .values()
                .any(|j| j.prog == prog && !j.status.terminal())
        {
            return Err(Error::Busy);
        }
        p.lifecycle = lifecycle;
        self.transition(
            &old,
            state,
            Builder::new(),
            Scope {
                kind: lifecycle.name(),
                prog: Some(prog),
                invocation: None,
            },
            Some(prog),
            &[],
        )
    }
    pub fn upgrade(
        &self,
        neuron: NeuronId,
        prog: Particle,
        source: Vec<u8>,
        initial: Vec<u8>,
    ) -> Result<Head, Error> {
        let old = self.inspect(neuron)?;
        let mut state = old.state.clone();
        let p = state.progs.get_mut(&prog).ok_or(Error::Missing)?;
        if matches!(p.lifecycle, Lifecycle::Retiring | Lifecycle::Retired) {
            return Err(Error::Lifecycle);
        }
        self.runtime.start(RuntimeInput {
            source: &source,
            state: &initial,
            event: &initial,
            context: None,
            step_limit: p.step_limit,
        })?;
        let mut b = Builder::new();
        p.source = model::artifact(&mut b, source, "application/x-rune")?;
        p.state = model::artifact(&mut b, initial, "application/x-rune-noun")?;
        p.revision = p.revision.checked_add(1).ok_or(Error::Budget)?;
        let record = b.values(
            "neuron/upgrade/1",
            &[Ref(prog), Ref(p.source), Ref(p.state)],
        )?;
        self.transition(
            &old,
            state,
            b,
            Scope {
                kind: "upgrade",
                prog: Some(prog),
                invocation: None,
            },
            Some(record),
            &[record],
        )
    }
    pub fn change_policy(
        &self,
        neuron: NeuronId,
        policy: Particle,
        epoch: u64,
    ) -> Result<Head, Error> {
        let old = self.inspect(neuron)?;
        if epoch <= old.state.epoch {
            return Err(Error::Conflict);
        }
        let mut state = old.state.clone();
        state.policy = policy;
        state.epoch = epoch;
        let mut b = Builder::new();
        let record = b.values("neuron/policy-change/1", &[Ref(policy), Uint(epoch)])?;
        self.transition(
            &old,
            state,
            b,
            Scope {
                kind: "change-policy",
                prog: None,
                invocation: None,
            },
            Some(record),
            &[record],
        )
    }
    pub fn rebind(&self, neuron: NeuronId, invocation: Particle) -> Result<Head, Error> {
        let old = self.inspect(neuron)?;
        let mut state = old.state.clone();
        let job = state
            .invocations
            .get_mut(&invocation)
            .ok_or(Error::Missing)?;
        if job.status.terminal()
            || job.reserved > 0
            || job
                .pending
                .as_ref()
                .is_some_and(|p| !matches!(p.stage, 2..=4))
        {
            return Err(Error::Busy);
        }
        job.epoch = state.epoch;
        let prog = job.prog;
        let mut b = Builder::new();
        let record = b.values("neuron/rebind/1", &[Ref(invocation), Uint(state.epoch)])?;
        self.transition(
            &old,
            state,
            b,
            Scope {
                kind: "rebind",
                prog: Some(prog),
                invocation: Some(invocation),
            },
            Some(record),
            &[record],
        )
    }
}
