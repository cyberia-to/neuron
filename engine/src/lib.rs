//! Instance lifecycle and durable execution over graph/runtime ports.
mod engine;
mod execution;
mod records;
use cell_model::{Builder, Content, Error as DataError, Head, Particle, Source};
pub use engine::*;
pub use execution::{Attempt, Progress};
pub use records::{Live, Pending};

#[derive(Debug)]
pub enum Error {
    Data(DataError),
    Graph(String),
    CommitUnknown(String),
    Runtime(String),
    Missing,
    Conflict,
    Busy,
    Lifecycle,
    Denied,
    UnknownOutcome,
    Budget,
    Unsupported,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cell: {self:?}")
    }
}
impl std::error::Error for Error {}
impl From<DataError> for Error {
    fn from(e: DataError) -> Self {
        Self::Data(e)
    }
}

pub trait GraphPort: Source {
    fn head(&self, cell: Particle) -> Result<Option<Head>, Error>;
    fn resolve(&self, cell: Particle, request: Particle) -> Result<Option<Head>, Error>;
    fn history(&self, cell: Particle, after: Option<u64>, limit: usize)
    -> Result<Vec<Head>, Error>;
    fn commit(
        &self,
        cell: Particle,
        request: Particle,
        expected: Option<Head>,
        head: Head,
        content: Builder,
        claim: Option<(Particle, Particle)>,
    ) -> Result<Head, Error>;
}

pub struct RuntimeInput<'a> {
    pub source: &'a [u8],
    pub state: &'a [u8],
    pub event: &'a [u8],
    pub context: Option<Particle>,
    pub step_limit: u64,
}
#[derive(Debug)]
pub enum RuntimeStep {
    Done {
        result: Vec<u8>,
        used: u64,
    },
    Yield {
        checkpoint: Vec<u8>,
        used: u64,
    },
    Act {
        tag: u64,
        arguments: Vec<u8>,
        checkpoint: Vec<u8>,
        used: u64,
    },
    Event {
        tag: u64,
        selector: Vec<u8>,
        checkpoint: Vec<u8>,
        used: u64,
    },
}
pub trait RuntimePort {
    fn validate_value(&self, bytes: &[u8]) -> Result<(), Error>;
    fn start(&self, input: RuntimeInput<'_>) -> Result<Vec<u8>, Error>;
    fn step(
        &self,
        checkpoint: &[u8],
        reply: Option<&[u8]>,
        slice: u64,
        total: u64,
    ) -> Result<RuntimeStep, Error>;
}

/// Host authority is never loaded from a runtime subject or checkpoint.
pub struct Authorization<'a> {
    pub cell: Particle,
    pub operation: Particle,
    pub policy: Particle,
    pub epoch: u64,
    pub act: u64,
    pub allowed_acts: &'a [u64],
}
pub trait WardPort: Send + Sync {
    fn authorize(&self, request: &Authorization<'_>) -> Result<(), Error>;
}
pub struct DenyAll;
impl WardPort for DenyAll {
    fn authorize(&self, _: &Authorization<'_>) -> Result<(), Error> {
        Err(Error::Denied)
    }
}

pub struct Overlay<'a, G> {
    pub graph: &'a G,
    pub batch: &'a Builder,
}
impl<G: Source> Source for Overlay<'_, G> {
    fn get(&self, id: &Particle) -> Result<Content, DataError> {
        self.batch
            .content
            .get(id)
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| self.graph.get(id))
    }
}
