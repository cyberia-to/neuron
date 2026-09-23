//! Explicit worker placement data, with no authority or signing identity.
use crate::{Builder, Error, Particle, Reader, Source, Value::*};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Worker {
    pub machine: String,
    pub environment: String,
    pub proof: String,
    pub network: Particle,
    pub warrior: String,
    pub version: String,
    pub backend: String,
    pub worker: Particle,
    pub device: Particle,
    pub boot: Particle,
    pub max_arguments: u64,
    pub acts: Vec<u64>,
}
impl Worker {
    pub fn validate(&self) -> Result<(), Error> {
        for value in [
            &self.machine,
            &self.environment,
            &self.proof,
            &self.warrior,
            &self.version,
            &self.backend,
        ] {
            if value.is_empty() || value.len() > 128 || !value.is_ascii() {
                return Err(Error::InvalidData);
            }
        }
        if self.max_arguments == 0 || self.max_arguments > 8 * 1024 * 1024 || self.acts.len() > 256
        {
            return Err(Error::Limit);
        }
        if self.acts.windows(2).any(|w| w[0] >= w[1]) {
            return Err(Error::InvalidData);
        }
        Ok(())
    }
    pub fn encode(&self, b: &mut Builder) -> Result<Particle, Error> {
        self.validate()?;
        let acts = self
            .acts
            .iter()
            .map(|&a| b.uint(a))
            .collect::<Result<Vec<_>, _>>()?;
        let acts = b.list(&acts)?;
        b.values(
            "neuron/worker/1",
            &[
                Text(&self.machine),
                Text(&self.environment),
                Text(&self.proof),
                Ref(self.network),
                Text(&self.warrior),
                Text(&self.version),
                Text(&self.backend),
                Ref(self.worker),
                Ref(self.device),
                Ref(self.boot),
                Uint(self.max_arguments),
                Raw(acts),
            ],
        )
    }
    pub fn read<S: Source>(r: &mut Reader<'_, S>, id: Particle) -> Result<Self, Error> {
        let fields = r.record(id, "neuron/worker/1", 12)?;
        let value = Self {
            machine: r.text(fields[0])?,
            environment: r.text(fields[1])?,
            proof: r.text(fields[2])?,
            network: r.reference(fields[3])?,
            warrior: r.text(fields[4])?,
            version: r.text(fields[5])?,
            backend: r.text(fields[6])?,
            worker: r.reference(fields[7])?,
            device: r.reference(fields[8])?,
            boot: r.reference(fields[9])?,
            max_arguments: r.uint(fields[10])?,
            acts: r
                .list(fields[11], 256)?
                .into_iter()
                .map(|v| r.uint(v))
                .collect::<Result<_, _>>()?,
        };
        value.validate()?;
        Ok(value)
    }
}
