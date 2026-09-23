//! Versioned neuron/prog/invocation records over the shared graph data codec.
use crate::{Builder, Error, Lifecycle, NeuronId, Particle, Reader, Source, Value::*};
use std::collections::BTreeMap;

pub const MAX_PROGS: usize = 256;
pub const MAX_INVOCATIONS: usize = 1024;
pub const MAX_IMPORTS: usize = 4096;
pub const MAX_CHILDREN: usize = 256;
pub const MAX_STEPS: u64 = 1_000_000;

pub fn artifact(b: &mut Builder, bytes: Vec<u8>, media: &str) -> Result<Particle, Error> {
    let len = bytes.len() as u64;
    let content = b.blob(bytes)?;
    let codec = b.blob(b"hemera/bytes/0.3".to_vec())?;
    b.values(
        "neuron/artifact/1",
        &[Ref(content), Ref(codec), Uint(len), Text(media)],
    )
}
pub fn read_artifact<S: Source>(r: &mut Reader<'_, S>, id: Particle) -> Result<Vec<u8>, Error> {
    let f = match r.record(id, "neuron/artifact/1", 4) {
        Ok(f) => f,
        Err(Error::UnsupportedSchema) => return r.artifact(id),
        Err(e) => return Err(e),
    };
    let content = r.reference(f[0])?;
    let codec = r.reference(f[1])?;
    if codec != *hemera::hash(b"hemera/bytes/0.3").as_bytes() {
        return Err(Error::UnsupportedSchema);
    }
    let size = r.uint(f[2])?;
    r.text(f[3])?;
    let c = r.content(content)?;
    if !c.blob || c.bytes.len() as u64 != size {
        return Err(Error::InvalidData);
    }
    Ok(c.bytes)
}

#[derive(Clone, Debug)]
pub struct Prog {
    pub source: Particle,
    pub state: Particle,
    pub revision: u64,
    pub lifecycle: Lifecycle,
    pub step_limit: u64,
    pub max_inflight: u64,
    pub allowed_acts: Vec<u64>,
}
impl Prog {
    pub fn validate(&self) -> Result<(), Error> {
        if self.step_limit == 0
            || self.step_limit > MAX_STEPS
            || self.max_inflight == 0
            || self.max_inflight > MAX_INVOCATIONS as u64
            || self.allowed_acts.len() > 256
        {
            return Err(Error::Limit);
        }
        if self.allowed_acts.windows(2).any(|w| w[0] >= w[1]) {
            return Err(Error::InvalidData);
        }
        Ok(())
    }
    pub fn encode(&self, b: &mut Builder) -> Result<Particle, Error> {
        self.validate()?;
        let tags = self
            .allowed_acts
            .iter()
            .map(|v| b.uint(*v))
            .collect::<Result<Vec<_>, _>>()?;
        let tags = b.list(&tags)?;
        b.values(
            "neuron/prog/1",
            &[
                Ref(self.source),
                Ref(self.state),
                Uint(self.revision),
                Uint(match self.lifecycle {
                    Lifecycle::Installed => 0,
                    Lifecycle::Active => 1,
                    Lifecycle::Paused => 2,
                    Lifecycle::Retiring => 3,
                    Lifecycle::Retired => 4,
                }),
                Uint(self.step_limit),
                Uint(self.max_inflight),
                Raw(tags),
            ],
        )
    }
    pub fn read<S: Source>(r: &mut Reader<'_, S>, id: Particle) -> Result<Self, Error> {
        let f = r.record(id, "neuron/prog/1", 7)?;
        let lifecycle = match r.uint(f[3])? {
            0 => Lifecycle::Installed,
            1 => Lifecycle::Active,
            2 => Lifecycle::Paused,
            3 => Lifecycle::Retiring,
            4 => Lifecycle::Retired,
            _ => return Err(Error::InvalidData),
        };
        let step_limit = r.uint(f[4])?;
        let max_inflight = r.uint(f[5])?;
        if step_limit == 0
            || step_limit > MAX_STEPS
            || max_inflight == 0
            || max_inflight > MAX_INVOCATIONS as u64
        {
            return Err(Error::Limit);
        }
        let allowed_acts = r
            .list(f[6], 256)?
            .into_iter()
            .map(|v| r.uint(v))
            .collect::<Result<Vec<_>, _>>()?;
        if allowed_acts.windows(2).any(|w| w[0] >= w[1]) {
            return Err(Error::InvalidData);
        }
        Ok(Self {
            source: r.reference(f[0])?,
            state: r.reference(f[1])?,
            revision: r.uint(f[2])?,
            lifecycle,
            step_limit,
            max_inflight,
            allowed_acts,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Status {
    Running = 0,
    Waiting = 1,
    Joining = 2,
    Completed = 3,
    Cancelled = 4,
    Failed = 5,
    Conflict = 6,
}
impl Status {
    pub fn terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Cancelled | Self::Failed | Self::Conflict
        )
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Joining => "joining",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
            Self::Conflict => "conflict",
        }
    }
    fn read(n: u64) -> Result<Self, Error> {
        match n {
            0 => Ok(Self::Running),
            1 => Ok(Self::Waiting),
            2 => Ok(Self::Joining),
            3 => Ok(Self::Completed),
            4 => Ok(Self::Cancelled),
            5 => Ok(Self::Failed),
            6 => Ok(Self::Conflict),
            _ => Err(Error::InvalidData),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PendingOperation {
    pub id: Particle,
    pub record: Particle,
    pub tag: u64,
    pub arguments: Particle,
    pub attempt: Option<Particle>,
    pub outcome: Option<Particle>,
    pub stage: u64,
    pub value: Option<Particle>,
    pub failed: bool,
    pub dispatch: Option<Particle>,
    pub attempt_record: Option<Particle>,
}
impl PendingOperation {
    pub fn validate(&self) -> Result<(), Error> {
        let valid = match self.stage {
            0 | 3 => {
                self.attempt.is_none()
                    && self.outcome.is_none()
                    && self.value.is_none()
                    && !self.failed
            }
            1 => {
                self.attempt.is_some()
                    && self.outcome.is_none()
                    && self.value.is_none()
                    && !self.failed
            }
            2 => self.attempt.is_some() && self.outcome.is_some() && self.value.is_some(),
            4 => {
                self.attempt.is_none()
                    && self.outcome.is_some()
                    && self.value.is_some()
                    && !self.failed
            }
            _ => false,
        };
        if valid
            && ((self.dispatch.is_none() && self.attempt_record.is_none())
                || matches!(self.stage, 1 | 2))
        {
            Ok(())
        } else {
            Err(Error::InvalidData)
        }
    }
    pub fn encode(&self, b: &mut Builder) -> Result<Particle, Error> {
        self.validate()?;
        b.values(
            "neuron/pending/1",
            &[
                Ref(self.id),
                Ref(self.record),
                Uint(self.tag),
                Ref(self.arguments),
                Optional(self.attempt),
                Optional(self.outcome),
                Uint(self.stage),
                Optional(self.value),
                Uint(u64::from(self.failed)),
                Optional(self.dispatch),
                Optional(self.attempt_record),
            ],
        )
    }
    pub fn read<S: Source>(r: &mut Reader<'_, S>, id: Particle) -> Result<Self, Error> {
        let f = r.record(id, "neuron/pending/1", 11)?;
        let stage = r.uint(f[6])?;
        let failed = r.uint(f[8])?;
        if stage > 4 || failed > 1 {
            return Err(Error::InvalidData);
        }
        let value = Self {
            id: r.reference(f[0])?,
            record: r.reference(f[1])?,
            tag: r.uint(f[2])?,
            arguments: r.reference(f[3])?,
            attempt: r.optional_ref(f[4])?,
            outcome: r.optional_ref(f[5])?,
            stage,
            value: r.optional_ref(f[7])?,
            failed: failed == 1,
            dispatch: r.optional_ref(f[9])?,
            attempt_record: r.optional_ref(f[10])?,
        };
        if (stage == 1 || stage == 2) && value.attempt.is_none()
            || stage == 2 && (value.outcome.is_none() || value.value.is_none())
            || (stage == 0 || stage == 3) && (value.attempt.is_some() || value.outcome.is_some())
        {
            return Err(Error::InvalidData);
        }
        value.validate()?;
        if let Some(dispatch) = value.dispatch {
            let fields = r.record(dispatch, "neuron/dispatch/1", 4)?;
            if r.reference(fields[0])? != value.id
                || Some(r.reference(fields[1])?) != value.attempt
                || r.uint(fields[2])? == 0
            {
                return Err(Error::InvalidData);
            }
            let worker = r.reference(fields[3])?;
            crate::worker::Worker::read(r, worker)?;
        }
        Ok(value)
    }
}

#[derive(Clone, Debug)]
pub struct Invocation {
    pub prog: Particle,
    pub code: Particle,
    pub input: Particle,
    pub context: Option<Particle>,
    pub base_revision: u64,
    pub checkpoint: Option<Particle>,
    pub status: Status,
    pub limit: u64,
    pub charged: u64,
    pub reserved: u64,
    pub used: u64,
    pub pending: Option<PendingOperation>,
    pub result: Option<Particle>,
    pub fault: Option<Particle>,
    pub parent: Option<Particle>,
    pub delegated: u64,
    pub children: Vec<Particle>,
    pub epoch: u64,
    pub ordinal: u64,
}
impl Invocation {
    pub fn validate(&self) -> Result<(), Error> {
        if self.limit == 0
            || self.limit > MAX_STEPS
            || self.reserved > self.available()?
            || self.used > self.charged
            || self.children.len() > MAX_CHILDREN
        {
            return Err(Error::InvalidData);
        }
        if let Some(p) = &self.pending {
            p.validate()?;
        }
        let valid = match self.status {
            Status::Running => {
                self.checkpoint.is_some() && self.pending.is_none() && self.result.is_none()
            }
            Status::Waiting => {
                self.checkpoint.is_some() && self.pending.is_some() && self.result.is_none()
            }
            Status::Joining => {
                self.checkpoint.is_none()
                    && self.pending.is_none()
                    && (self.result.is_some() ^ self.fault.is_some())
                    && self.reserved == 0
            }
            Status::Completed | Status::Conflict => {
                self.checkpoint.is_none()
                    && self.pending.is_none()
                    && self.result.is_some()
                    && self.reserved == 0
            }
            Status::Cancelled | Status::Failed => {
                self.checkpoint.is_none()
                    && self.pending.is_none()
                    && self.fault.is_some()
                    && self.reserved == 0
            }
        };
        if valid {
            Ok(())
        } else {
            Err(Error::InvalidData)
        }
    }
    pub fn available(&self) -> Result<u64, Error> {
        self.limit
            .checked_sub(self.charged)
            .and_then(|n| n.checked_sub(self.delegated))
            .ok_or(Error::InvalidData)
    }
    pub fn encode(&self, b: &mut Builder) -> Result<Particle, Error> {
        self.validate()?;
        let pending = self.pending.as_ref().map(|p| p.encode(b)).transpose()?;
        b.values(
            "neuron/invocation/1",
            &[
                Ref(self.prog),
                Ref(self.code),
                Ref(self.input),
                Optional(self.context),
                Uint(self.base_revision),
                Optional(self.checkpoint),
                Uint(self.status as u64),
                Uint(self.limit),
                Uint(self.charged),
                Uint(self.reserved),
                Uint(self.used),
                Optional(pending),
                Optional(self.result),
                Optional(self.fault),
                Optional(self.parent),
                Uint(self.delegated),
                Refs(&self.children),
                Uint(self.epoch),
                Uint(self.ordinal),
            ],
        )
    }
    pub fn read<S: Source>(r: &mut Reader<'_, S>, id: Particle) -> Result<Self, Error> {
        let f = r.record(id, "neuron/invocation/1", 19)?;
        let pending = r
            .optional_ref(f[11])?
            .map(|id| PendingOperation::read(r, id))
            .transpose()?;
        let children = r
            .list(f[16], MAX_CHILDREN)?
            .into_iter()
            .map(|v| r.reference(v))
            .collect::<Result<Vec<_>, _>>()?;
        let value = Self {
            prog: r.reference(f[0])?,
            code: r.reference(f[1])?,
            input: r.reference(f[2])?,
            context: r.optional_ref(f[3])?,
            base_revision: r.uint(f[4])?,
            checkpoint: r.optional_ref(f[5])?,
            status: Status::read(r.uint(f[6])?)?,
            limit: r.uint(f[7])?,
            charged: r.uint(f[8])?,
            reserved: r.uint(f[9])?,
            used: r.uint(f[10])?,
            pending,
            result: r.optional_ref(f[12])?,
            fault: r.optional_ref(f[13])?,
            parent: r.optional_ref(f[14])?,
            delegated: r.uint(f[15])?,
            children,
            epoch: r.uint(f[17])?,
            ordinal: r.uint(f[18])?,
        };
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug)]
pub struct NeuronState {
    pub neuron: NeuronId,
    pub network: Particle,
    pub policy: Particle,
    pub epoch: u64,
    pub limit: u64,
    pub charged: u64,
    pub held: u64,
    pub progs: BTreeMap<Particle, Prog>,
    pub invocations: BTreeMap<Particle, Invocation>,
    pub imports: BTreeMap<Particle, Particle>,
    pub cursor: u64,
    pub writer_generation: u64,
    pub worker: Option<Particle>,
}
impl NeuronState {
    pub fn validate(&self) -> Result<(), Error> {
        if self.writer_generation == 0 && self.worker.is_some() {
            return Err(Error::InvalidData);
        }
        if self.progs.len() > MAX_PROGS
            || self.invocations.len() > MAX_INVOCATIONS
            || self.imports.len() > MAX_IMPORTS
            || self
                .charged
                .checked_add(self.held)
                .is_none_or(|n| n > self.limit)
        {
            return Err(Error::Limit);
        }
        for prog in self.progs.values() {
            prog.validate()?;
        }
        let mut held = 0u64;
        let mut charged = 0u64;
        for (id, job) in &self.invocations {
            job.validate()?;
            charged = charged.checked_add(job.charged).ok_or(Error::Limit)?;
            if !self.progs.contains_key(&job.prog) {
                return Err(Error::InvalidData);
            }
            if !job.status.terminal() {
                held = held.checked_add(job.available()?).ok_or(Error::Limit)?;
            }
            if let Some(parent) = job.parent {
                let p = self.invocations.get(&parent).ok_or(Error::InvalidData)?;
                if !p.children.contains(id) {
                    return Err(Error::InvalidData);
                }
            }
            let mut seen = std::collections::BTreeSet::new();
            let mut delegated = 0u64;
            for child in &job.children {
                if !seen.insert(child)
                    || self
                        .invocations
                        .get(child)
                        .is_none_or(|c| c.parent != Some(*id))
                {
                    return Err(Error::InvalidData);
                }
                let child = &self.invocations[child];
                if job.status.terminal() && !child.status.terminal() {
                    return Err(Error::InvalidData);
                }
                let allocation = if child.status.terminal() {
                    child
                        .charged
                        .checked_add(child.delegated)
                        .ok_or(Error::Limit)?
                } else {
                    child.limit
                };
                delegated = delegated.checked_add(allocation).ok_or(Error::Limit)?;
            }
            if delegated != job.delegated {
                return Err(Error::InvalidData);
            }
            let mut ancestor = job.parent;
            let mut depth = 0;
            while let Some(parent) = ancestor {
                depth += 1;
                if parent == *id || depth > 64 {
                    return Err(Error::InvalidData);
                }
                ancestor = self
                    .invocations
                    .get(&parent)
                    .ok_or(Error::InvalidData)?
                    .parent;
            }
        }
        if held != self.held || charged > self.charged {
            return Err(Error::InvalidData);
        }
        Ok(())
    }
    pub fn encode(&self, b: &mut Builder) -> Result<Particle, Error> {
        self.validate()?;
        let progs = self
            .progs
            .iter()
            .map(|(k, v)| Ok((*k, v.encode(b)?)))
            .collect::<Result<BTreeMap<_, _>, Error>>()?;
        let invocations = self
            .invocations
            .iter()
            .map(|(k, v)| Ok((*k, v.encode(b)?)))
            .collect::<Result<BTreeMap<_, _>, Error>>()?;
        let progs = b.map(&progs)?;
        let invocations = b.map(&invocations)?;
        let imports = b.map(&self.imports)?;
        b.values(
            "neuron/root/1",
            &[
                Ref(self.neuron),
                Ref(self.network),
                Ref(self.policy),
                Uint(self.epoch),
                Uint(self.limit),
                Uint(self.charged),
                Uint(self.held),
                Ref(progs),
                Ref(invocations),
                Ref(imports),
                Uint(self.cursor),
                Uint(self.writer_generation),
                Optional(self.worker),
            ],
        )
    }
    pub fn read<S: Source>(r: &mut Reader<'_, S>, id: Particle) -> Result<Self, Error> {
        let f = r.record(id, "neuron/root/1", 13)?;
        let progs_id = r.reference(f[7])?;
        let invocations_id = r.reference(f[8])?;
        let imports_id = r.reference(f[9])?;
        let progs = r
            .map(progs_id, MAX_PROGS)?
            .into_iter()
            .map(|(k, v)| Ok((k, Prog::read(r, v)?)))
            .collect::<Result<_, Error>>()?;
        let invocations = r
            .map(invocations_id, MAX_INVOCATIONS)?
            .into_iter()
            .map(|(k, v)| Ok((k, Invocation::read(r, v)?)))
            .collect::<Result<_, Error>>()?;
        let value = Self {
            neuron: r.reference(f[0])?,
            network: r.reference(f[1])?,
            policy: r.reference(f[2])?,
            epoch: r.uint(f[3])?,
            limit: r.uint(f[4])?,
            charged: r.uint(f[5])?,
            held: r.uint(f[6])?,
            progs,
            invocations,
            imports: r.map(imports_id, MAX_IMPORTS)?.into_iter().collect(),
            cursor: r.uint(f[10])?,
            writer_generation: r.uint(f[11])?,
            worker: r.optional_ref(f[12])?,
        };
        value.validate()?;
        if let Some(worker) = value.worker
            && crate::worker::Worker::read(r, worker)?.network != value.network
        {
            return Err(Error::InvalidData);
        }
        Ok(value)
    }
}
