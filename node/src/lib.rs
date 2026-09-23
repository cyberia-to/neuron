//! Neuron execution → shared cybergraph/BBG transactions and host authority.
use neuron_engine::{Error, GraphPort};
pub mod authority;
#[cfg(feature = "vault")]
pub mod custody;
pub mod transfer;
pub mod archive;
pub use archive::ArchiveGraph;
pub mod worker;
#[cfg(feature = "query")]
pub mod query;
pub use transfer::stage_legacy;
pub use worker::{DispatchToken, EffectOutcome, Execution, LocalWorker};
mod validation;
pub use authority::{Grant, GrantHandle, KeyVault, LocalAuthority, SigningVault};
use cybergraph::application::{
    ApplicationGraph, Error as GraphError, Head as GraphHead, Proposal, StorageError,
};
pub use cybergraph::application::{Backend, Database};
use cybergraph::content::{Codec, Content as GraphContent};
use neuron_model::{Builder, Content, Error as DataError, Head, Particle, Source};
pub use neuron_rune::Rune;

pub struct Graph(pub ApplicationGraph);
impl Graph {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        ApplicationGraph::open(path).map(Self).map_err(graph_error)
    }
    pub fn from_database(database: Database) -> Self {
        Self(ApplicationGraph::from_database(database))
    }
    #[cfg(feature = "legacy-redb-migration")]
    pub fn migrate_redb(
        source: impl AsRef<std::path::Path>,
        destination: impl AsRef<std::path::Path>,
    ) -> Result<(), Error> {
        ApplicationGraph::migrate_redb(source, destination).map_err(graph_error)
    }
}
fn graph_error(e: GraphError) -> Error {
    match e {
        GraphError::Storage(StorageError::Conflict | StorageError::HeadMismatch) => Error::Conflict,
        GraphError::Storage(StorageError::Fenced) => Error::Fenced,
        GraphError::Storage(StorageError::CommitUnknown(reason)) => Error::CommitUnknown(reason),
        other => Error::Graph(other.to_string()),
    }
}
impl neuron_engine::MigrationPort for Graph {
    fn commit_migration(&self, write: neuron_engine::MigrationWrite) -> Result<Head, Error> {
        let neuron_engine::MigrationWrite {
            neuron,
            request,
            expected,
            head: next,
            content,
            event,
            manifest,
            sources,
        } = write;
        validation::publication(self, neuron, Some(expected), next, &content)?;
        let required = content.content.keys().copied().collect();
        let content = content
            .content
            .into_values()
            .map(|v| {
                GraphContent::new(if v.blob { Codec::Blob } else { Codec::Data }, v.bytes)
                    .map_err(|e| Error::Graph(format!("{e:?}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let sources = sources
            .iter()
            .map(|(id, h)| (*id, graph_head(*h)))
            .collect::<Vec<_>>();
        let proposal = Proposal {
            namespace: neuron,
            request,
            expected: Some(graph_head(expected)),
            head: graph_head(next),
            content,
            required,
            claims: vec![(request, event)],
        };
        self.0
            .commit_migration(
                &proposal,
                &cybergraph::application::NamespaceMigration {
                    manifest,
                    sources: &sources,
                },
                |_| Ok(()),
            )
            .map(head)
            .map_err(graph_error)
    }
}
fn head(h: GraphHead) -> Head {
    Head {
        index: h.index,
        commit: h.commit,
    }
}
fn graph_head(h: Head) -> GraphHead {
    GraphHead {
        index: h.index,
        commit: h.commit,
    }
}
impl Source for Graph {
    fn get(&self, id: &Particle) -> Result<Content, DataError> {
        let content = self
            .0
            .get(id)
            .map_err(|e| DataError::Source(e.to_string()))?
            .ok_or(DataError::Missing(*id))?;
        Ok(Content {
            id: content.id(),
            bytes: content.bytes().to_vec(),
            blob: content.codec() == Codec::Blob,
        })
    }
}
impl GraphPort for Graph {
    fn commit_once(&self, write: neuron_engine::AtomicWrite) -> Result<Head, Error> {
        let neuron_engine::AtomicWrite {
            namespace,
            request,
            expected,
            head: next,
            content,
            claim,
        } = write;
        validation::publication(self, namespace, expected, next, &content)?;
        let required = content.content.keys().copied().collect();
        let content = content
            .content
            .into_values()
            .map(|v| {
                GraphContent::new(if v.blob { Codec::Blob } else { Codec::Data }, v.bytes)
                    .map_err(|e| Error::Graph(format!("{e:?}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let proposal = Proposal {
            namespace,
            request,
            expected: expected.map(graph_head),
            head: graph_head(next),
            content,
            required,
            claims: claim.into_iter().collect(),
        };
        self.0
            .commit_fresh(&proposal, |_| Ok(()))
            .map(head)
            .map_err(graph_error)
    }
    fn head(&self, namespace: Particle) -> Result<Option<Head>, Error> {
        self.0
            .head(&namespace)
            .map(|h| h.map(head))
            .map_err(graph_error)
    }
    fn resolve(&self, namespace: Particle, request: Particle) -> Result<Option<Head>, Error> {
        self.0
            .resolve(&namespace, &request)
            .map(|h| h.map(head))
            .map_err(graph_error)
    }
    fn history(
        &self,
        namespace: Particle,
        after: Option<u64>,
        limit: usize,
    ) -> Result<Vec<Head>, Error> {
        self.0
            .history(&namespace, after, limit)
            .map(|v| v.into_iter().map(head).collect())
            .map_err(graph_error)
    }
    fn commit(
        &self,
        namespace: Particle,
        request: Particle,
        expected: Option<Head>,
        next: Head,
        content: Builder,
        claim: Option<(Particle, Particle)>,
    ) -> Result<Head, Error> {
        let validation = validation::publication(self, namespace, expected, next, &content);
        let required = content.content.keys().copied().collect();
        let content = content
            .content
            .into_values()
            .map(|v| {
                GraphContent::new(if v.blob { Codec::Blob } else { Codec::Data }, v.bytes)
                    .map_err(|e| Error::Graph(format!("{e:?}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let proposal = Proposal {
            namespace,
            request,
            expected: expected.map(graph_head),
            head: graph_head(next),
            content,
            required,
            claims: claim.into_iter().collect(),
        };
        self.0
            .commit(&proposal, |_| {
                validation.map_err(|e| GraphError::Rejected(e.to_string()))
            })
            .map(head)
            .map_err(graph_error)
    }
}
