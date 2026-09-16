//! One neuron subject, durable programs and invocations over graph/runtime ports.
mod jobs;
pub mod legacy;
mod migration;
mod worker;
pub use migration::*;
mod neuron;
pub use jobs::*;
pub use neuron::*;
use neuron_model::{Builder, Content, Error as DataError, Head, Particle, Source};

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
    Fenced,
    Budget,
    Unsupported,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "neuron: {self:?}")
    }
}
impl std::error::Error for Error {}
impl From<DataError> for Error {
    fn from(e: DataError) -> Self {
        Self::Data(e)
    }
}

pub trait GraphPort: Source {
    fn commit_once(&self, _write: AtomicWrite) -> Result<Head, Error> {
        Err(Error::Unsupported)
    }
    fn head(&self, namespace: Particle) -> Result<Option<Head>, Error>;
    fn resolve(&self, namespace: Particle, request: Particle) -> Result<Option<Head>, Error>;
    fn history(
        &self,
        namespace: Particle,
        after: Option<u64>,
        limit: usize,
    ) -> Result<Vec<Head>, Error>;
    fn commit(
        &self,
        namespace: Particle,
        request: Particle,
        expected: Option<Head>,
        head: Head,
        content: Builder,
        claim: Option<(Particle, Particle)>,
    ) -> Result<Head, Error>;
}
impl<G: GraphPort> GraphPort for &G {
    fn commit_once(&self, write: AtomicWrite) -> Result<Head, Error> {
        (**self).commit_once(write)
    }
    fn head(&self, n: Particle) -> Result<Option<Head>, Error> {
        (**self).head(n)
    }
    fn resolve(&self, n: Particle, r: Particle) -> Result<Option<Head>, Error> {
        (**self).resolve(n, r)
    }
    fn history(&self, n: Particle, after: Option<u64>, limit: usize) -> Result<Vec<Head>, Error> {
        (**self).history(n, after, limit)
    }
    fn commit(
        &self,
        n: Particle,
        r: Particle,
        expected: Option<Head>,
        head: Head,
        b: Builder,
        claim: Option<(Particle, Particle)>,
    ) -> Result<Head, Error> {
        (**self).commit(n, r, expected, head, b, claim)
    }
}

/// A fresh publication cannot resolve an old receipt as newly committed work.
pub struct AtomicWrite {
    pub namespace: Particle,
    pub request: Particle,
    pub expected: Option<Head>,
    pub head: Head,
    pub content: Builder,
    pub claim: Option<(Particle, Particle)>,
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
    fn validate_checkpoint(&self, _bytes: &[u8], _limit: u64) -> Result<(), Error> {
        Err(Error::Unsupported)
    }
    fn start(&self, input: RuntimeInput<'_>) -> Result<Vec<u8>, Error>;
    fn step(
        &self,
        checkpoint: &[u8],
        reply: Option<&[u8]>,
        slice: u64,
        total: u64,
    ) -> Result<RuntimeStep, Error>;
}

/// A migration publication must atomically validate all origins and fence them.
pub trait MigrationPort: GraphPort {
    fn commit_migration(&self, write: MigrationWrite) -> Result<Head, Error>;
}
pub struct MigrationWrite {
    pub neuron: Particle,
    pub request: Particle,
    pub expected: Head,
    pub head: Head,
    pub content: Builder,
    pub event: Particle,
    pub manifest: Particle,
    pub sources: Vec<(Particle, Head)>,
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
