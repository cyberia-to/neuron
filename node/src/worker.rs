//! Embedded synchronous host-act worker; no private keys or subject identity.
use crate::Graph;
use neuron_engine::{Authority, Dispatch, Error, Neuron, RuntimePort};
use neuron_model::{Head, NeuronId, Particle, worker::Worker};

pub const MACHINE: &str = "native-rust/1";
pub const ENVIRONMENT: &str = "neuron/host-act/1";
pub const PROOF: &str = "none/1";
pub const WARRIOR: &str = "neuron-host";
pub const BACKEND: &str = "embedded-sync/1";

pub struct LocalWorker {
    descriptor: Worker,
    id: Particle,
    generation: u64,
}
/// An affine handoff. Serialized/public Dispatch metadata cannot create it.
pub struct DispatchToken {
    permit: neuron_engine::AttemptPermit,
}
impl DispatchToken {
    pub fn metadata(&self) -> &Dispatch {
        self.permit.metadata()
    }
}

#[derive(Debug)]
pub enum EffectOutcome {
    Value(Vec<u8>),
    Failed(String),
    Unknown(String),
}
pub struct Execution {
    pub dispatch: Dispatch,
    pub claim: Head,
    pub outcome: EffectOutcome,
    /// None means unknown. Err retains the returned outcome for reconciliation;
    /// it never authorizes rerunning the adapter.
    pub reconciliation: Option<Result<Head, Error>>,
}

impl LocalWorker {
    pub fn descriptor(
        network: Particle,
        worker: Particle,
        device: Particle,
        boot: Particle,
        mut acts: Vec<u64>,
        max_arguments: u64,
    ) -> Worker {
        acts.sort_unstable();
        acts.dedup();
        Worker {
            machine: MACHINE.into(),
            environment: ENVIRONMENT.into(),
            proof: PROOF.into(),
            network,
            warrior: WARRIOR.into(),
            version: env!("CARGO_PKG_VERSION").into(),
            backend: BACKEND.into(),
            worker,
            device,
            boot,
            max_arguments,
            acts,
        }
    }
    pub fn bind<R: RuntimePort, A: Authority>(
        agent: &Neuron<Graph, R, A>,
        neuron: NeuronId,
        expected_generation: u64,
        descriptor: Worker,
    ) -> Result<Self, Error> {
        descriptor.validate()?;
        if descriptor.machine != MACHINE
            || descriptor.environment != ENVIRONMENT
            || descriptor.proof != PROOF
            || descriptor.warrior != WARRIOR
            || descriptor.version != env!("CARGO_PKG_VERSION")
            || descriptor.backend != BACKEND
        {
            return Err(Error::Unsupported);
        }
        let (generation, id, _) =
            agent.place_worker(neuron, expected_generation, Some(&descriptor))?;
        Ok(Self {
            descriptor,
            id: id.ok_or(Error::Conflict)?,
            generation,
        })
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn contract(&self) -> &Worker {
        &self.descriptor
    }
    pub fn prepare<R: RuntimePort, A: Authority>(
        &self,
        agent: &Neuron<Graph, R, A>,
        neuron: NeuronId,
        invocation: Particle,
    ) -> Result<DispatchToken, Error> {
        let permit = agent.begin_worker_attempt(neuron, invocation, self.id, self.generation)?;
        Ok(DispatchToken { permit })
    }
    pub fn execute<R: RuntimePort, A: Authority>(
        &self,
        agent: &Neuron<Graph, R, A>,
        token: DispatchToken,
        effect: impl FnOnce(&Dispatch) -> EffectOutcome,
    ) -> Result<Execution, Error> {
        let d = token.metadata();
        if d.executor != self.id
            || d.generation != self.generation
            || d.network != self.descriptor.network
        {
            return Err(Error::Fenced);
        }
        let mut effect = Some(effect);
        let mut outcome = None;
        let (d, claim) = agent.dispatch_once(token.permit, &mut |d| {
            outcome = Some(effect.take().ok_or(Error::UnknownOutcome)?(d));
            Ok(())
        })?;
        let outcome = outcome.ok_or(Error::UnknownOutcome)?;
        let reconciliation = match &outcome {
            EffectOutcome::Value(value) => Some(agent.record_outcome(
                d.neuron,
                d.invocation,
                d.operation,
                d.attempt,
                value.clone(),
            )),
            EffectOutcome::Failed(reason) => Some(agent.record_failure(
                d.neuron,
                d.invocation,
                d.operation,
                d.attempt,
                reason.clone(),
            )),
            EffectOutcome::Unknown(_) => None,
        };
        Ok(Execution {
            dispatch: d,
            claim,
            outcome,
            reconciliation,
        })
    }
}
