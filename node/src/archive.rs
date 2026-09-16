//! An explicit read-only graph port for original or sealed legacy source stores.
use crate::{graph_error, head as model_head};
pub use cybergraph::application::{
    ArchiveSeal, ArchiveSummary, MAX_INSPECTION_BYTES, MAX_INSPECTION_ROWS,
};
use neuron_engine::{Error, GraphPort};
use neuron_model::{Builder, Content, Error as DataError, Head, Particle, Source};
pub struct ArchiveGraph(cybergraph::application::Archive);
impl ArchiveGraph {
    pub fn open(path: &std::path::Path) -> Result<Self, Error> {
        cybergraph::application::Archive::open(path)
            .map(Self)
            .map_err(graph_error)
    }
    pub fn seal_for_transfer(
        self,
        target: Particle,
        nonce: Particle,
    ) -> Result<cybergraph::application::Transfer, Error> {
        self.0.seal_for_transfer(target, nonce).map_err(graph_error)
    }
    pub fn sources(&self) -> Vec<(Particle, Head)> {
        self.0
            .sources()
            .iter()
            .map(|(id, head)| (*id, model_head(*head)))
            .collect()
    }
    pub fn seal(&self) -> Option<&ArchiveSeal> {
        self.0.seal()
    }
    pub fn last_transaction(&self) -> Option<Particle> {
        self.0.last_transaction()
    }
    pub fn inspect(&self, rows: u64, bytes: u64) -> Result<ArchiveSummary, Error> {
        self.0.inspect(rows, bytes).map_err(graph_error)
    }
}
impl Source for ArchiveGraph {
    fn get(&self, id: &Particle) -> Result<Content, DataError> {
        let value = self
            .0
            .get(id)
            .map_err(|e| DataError::Source(e.to_string()))?
            .ok_or(DataError::Missing(*id))?;
        Ok(Content {
            id: value.id(),
            bytes: value.bytes().to_vec(),
            blob: value.codec() == cybergraph::content::Codec::Blob,
        })
    }
}
impl GraphPort for ArchiveGraph {
    fn head(&self, namespace: Particle) -> Result<Option<Head>, Error> {
        self.0
            .head(&namespace)
            .map(|head| head.map(model_head))
            .map_err(graph_error)
    }
    fn resolve(&self, namespace: Particle, request: Particle) -> Result<Option<Head>, Error> {
        self.0
            .resolve(&namespace, &request)
            .map(|head| head.map(model_head))
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
            .map(|heads| heads.into_iter().map(model_head).collect())
            .map_err(graph_error)
    }
    fn commit(
        &self,
        _: Particle,
        _: Particle,
        _: Option<Head>,
        _: Head,
        _: Builder,
        _: Option<(Particle, Particle)>,
    ) -> Result<Head, Error> {
        Err(Error::Denied)
    }
}
