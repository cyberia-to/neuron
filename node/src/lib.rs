//! Production composition: cell engine → cybergraph → BBG local transactions.
use cell_engine::{Error, GraphPort};
use cell_model::{Builder, Content, Error as DataError, Head, Particle, Source};
pub use cell_rune::Rune;
use cybergraph::application::{
    ApplicationGraph, Error as GraphError, Head as GraphHead, Proposal, StorageError,
};
pub use cybergraph::application::{Backend, Database};
use cybergraph::content::{Codec, Content as GraphContent};

/// Bootstrap adapter for a private database owner. A production ward service
/// can implement the same port without depending on the interpreter.
pub struct LocalWard;
impl cell_engine::WardPort for LocalWard {
    fn authorize(&self, request: &cell_engine::Authorization<'_>) -> Result<(), Error> {
        if request.allowed_acts.contains(&request.act) {
            Ok(())
        } else {
            Err(Error::Denied)
        }
    }
}

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
        GraphError::Storage(StorageError::CommitUnknown(reason)) => Error::CommitUnknown(reason),
        other => Error::Graph(other.to_string()),
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
    fn head(&self, cell: Particle) -> Result<Option<Head>, Error> {
        self.0.head(&cell).map(|h| h.map(head)).map_err(graph_error)
    }
    fn resolve(&self, cell: Particle, request: Particle) -> Result<Option<Head>, Error> {
        self.0
            .resolve(&cell, &request)
            .map(|h| h.map(head))
            .map_err(graph_error)
    }
    fn history(
        &self,
        cell: Particle,
        after: Option<u64>,
        limit: usize,
    ) -> Result<Vec<Head>, Error> {
        self.0
            .history(&cell, after, limit)
            .map(|v| v.into_iter().map(head).collect())
            .map_err(graph_error)
    }
    fn commit(
        &self,
        cell: Particle,
        request: Particle,
        expected: Option<Head>,
        next: Head,
        content: Builder,
        claim: Option<(Particle, Particle)>,
    ) -> Result<Head, Error> {
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
            namespace: cell,
            request,
            expected: expected.map(graph_head),
            head: graph_head(next),
            content,
            required,
            claims: claim.into_iter().collect(),
        };
        self.0
            .commit(&proposal, |_| Ok(()))
            .map(head)
            .map_err(graph_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cell_engine::WardPort;

    fn particle(byte: u8) -> Particle {
        [byte; 32]
    }

    #[test]
    fn local_ward_authorizes_only_a_listed_act() {
        let ward = LocalWard;
        let allowed = [1u64, 3, 7];
        let ok = cell_engine::Authorization {
            cell: particle(1),
            operation: particle(2),
            policy: particle(3),
            epoch: 0,
            act: 3,
            allowed_acts: &allowed,
        };
        assert!(ward.authorize(&ok).is_ok());
        let denied = cell_engine::Authorization {
            act: 4,
            ..ok
        };
        assert!(matches!(ward.authorize(&denied), Err(Error::Denied)));
        let empty = cell_engine::Authorization {
            allowed_acts: &[],
            ..ok
        };
        assert!(matches!(ward.authorize(&empty), Err(Error::Denied)));
    }

    #[test]
    fn head_and_graph_head_round_trip() {
        let h = Head {
            index: 42,
            commit: particle(9),
        };
        let g = graph_head(h);
        assert_eq!(g.index, h.index);
        assert_eq!(g.commit, h.commit);
        assert_eq!(head(g), h);
    }

    #[test]
    fn graph_error_maps_conflict_and_head_mismatch_to_conflict() {
        for storage in [StorageError::Conflict, StorageError::HeadMismatch] {
            assert!(matches!(
                graph_error(GraphError::Storage(storage)),
                Error::Conflict
            ));
        }
    }

    #[test]
    fn graph_error_maps_commit_unknown_and_preserves_reason() {
        let e = graph_error(GraphError::Storage(StorageError::CommitUnknown(
            "reason".into(),
        )));
        assert!(matches!(e, Error::CommitUnknown(r) if r == "reason"));
    }

    #[test]
    fn graph_error_falls_back_to_graph_string_for_everything_else() {
        for storage in [
            StorageError::Fenced,
            StorageError::InvalidSequence,
            StorageError::Limit,
            StorageError::Corrupt,
            StorageError::Storage("boom".into()),
        ] {
            assert!(matches!(
                graph_error(GraphError::Storage(storage)),
                Error::Graph(_)
            ));
        }
        assert!(matches!(
            graph_error(GraphError::InvalidProposal),
            Error::Graph(_)
        ));
    }
}
