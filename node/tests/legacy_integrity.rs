#[path = "support/legacy.rs"]
mod legacy;
mod support;
use neuron_engine::{Error, GraphPort};
use neuron_model::{Builder, Content, Head, Particle, Reader, Snapshot, Source};
use neuron_node::Graph;
use support::*;
struct MissingArtifact<'a> {
    graph: &'a Graph,
    missing: Particle,
}
impl Source for MissingArtifact<'_> {
    fn get(&self, id: &Particle) -> Result<Content, neuron_model::Error> {
        if *id == self.missing {
            return Err(neuron_model::Error::Missing(*id));
        }
        self.graph.get(id)
    }
}
impl GraphPort for MissingArtifact<'_> {
    fn head(&self, n: Particle) -> Result<Option<Head>, Error> {
        self.graph.head(n)
    }
    fn resolve(&self, n: Particle, r: Particle) -> Result<Option<Head>, Error> {
        self.graph.resolve(n, r)
    }
    fn history(&self, n: Particle, a: Option<u64>, l: usize) -> Result<Vec<Head>, Error> {
        self.graph.history(n, a, l)
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
        Err(Error::Unsupported)
    }
}
#[test]
fn valid_live_head_does_not_hide_a_missing_historical_state_artifact() {
    let dir = Directory::new();
    let graph = dir.graph();
    legacy::replay(&graph, legacy::CAPTURE).unwrap();
    let origin = legacy::named("counter");
    let current = neuron_engine::legacy::inspect(&graph, origin).unwrap();
    let mut r = Reader::new(&graph, 100_000);
    let birth = r.record(origin, "cell/birth/1", 7).unwrap();
    let snapshot = r.reference(birth[5]).unwrap();
    let snapshot = Snapshot::read(&mut r, snapshot).unwrap();
    assert_ne!(
        snapshot.application_state,
        current.snapshot.application_state
    );
    let filtered = MissingArtifact {
        graph: &graph,
        missing: snapshot.application_state,
    };
    assert!(
        Reader::new(&filtered, 100_000)
            .artifact(current.snapshot.application_state)
            .is_ok()
    );
    assert!(neuron_engine::legacy::inspect(&filtered, origin).is_err());
}
