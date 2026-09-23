mod support;
use neuron_engine::{Action, Authority, Error, GraphPort, Neuron, Overlay};
use neuron_model::{
    Builder, Content, Head, Particle, Reader, Source, Value::*, execution::NeuronState,
};
use neuron_node::{Graph, Rune};
use support::*;
struct Forged;
impl Authority for Forged {
    fn with_current(
        &self,
        _: &Action<'_>,
        commit: &mut dyn FnMut() -> Result<(), Error>,
    ) -> Result<(), Error> {
        commit()
    }
    fn authorize(&self, _: &Action<'_>) -> Result<Vec<u8>, Error> {
        Ok(vec![1; 102])
    }
}

#[test]
fn revocation_after_signing_blocks_activation_and_normal_publication() {
    use std::sync::{Arc, Mutex};
    struct RevokeAfterSigning {
        inner: neuron_node::LocalAuthority<neuron_node::KeyVault>,
        grant: neuron_node::GrantHandle,
        kind: Arc<Mutex<Option<String>>>,
    }
    impl Authority for RevokeAfterSigning {
        fn authorize(&self, action: &Action<'_>) -> Result<Vec<u8>, Error> {
            let evidence = self.inner.authorize(action)?;
            let mut kind = self.kind.lock().unwrap();
            if kind.as_deref() == Some(action.kind) {
                *kind = None;
                let mut next = self.grant.get()?;
                let prior = next.revision;
                next.revision += 1;
                next.enabled = false;
                self.grant.replace(prior, next)?;
            }
            Ok(evidence)
        }
        fn with_current(
            &self,
            action: &Action<'_>,
            commit: &mut dyn FnMut() -> Result<(), Error>,
        ) -> Result<(), Error> {
            self.inner.with_current(action, commit)
        }
    }
    let dir = Directory::new();
    let (inner, grant) = authority();
    let id = inner.subject();
    let kind = Arc::new(Mutex::new(Some("activate".into())));
    let agent = Neuron::with_authority(
        dir.graph(),
        Rune,
        RevokeAfterSigning {
            inner,
            grant: grant.clone(),
            kind: kind.clone(),
        },
    );
    assert!(matches!(
        agent.activate(id, NETWORK, POLICY, 10000),
        Err(Error::Denied)
    ));
    assert!(agent.graph.head(id).unwrap().is_none());
    let mut next = grant.get().unwrap();
    next.enabled = true;
    next.revision += 1;
    grant.replace(1, next).unwrap();
    let before = agent.activate(id, NETWORK, POLICY, 10000).unwrap();
    *kind.lock().unwrap() = Some("install".into());
    assert!(matches!(
        agent.install(id, [1; 32], b"event".to_vec(), value("0"), config()),
        Err(Error::Denied)
    ));
    assert_eq!(agent.graph.head(id).unwrap(), Some(before));
    assert!(agent.inspect(id).unwrap().state.progs.is_empty());
}
#[test]
fn engine_adapter_cannot_publish_with_fabricated_authorization_evidence() {
    let dir = Directory::new();
    let agent = Neuron::with_authority(dir.graph(), Rune, Forged);
    assert!(agent.activate([7; 32], NETWORK, POLICY, 1000).is_err());
    assert!(agent.graph.head([7; 32]).unwrap().is_none());
}

struct RewriteRoot(Graph);
impl Source for RewriteRoot {
    fn get(&self, id: &Particle) -> Result<Content, neuron_model::Error> {
        self.0.get(id)
    }
}
impl GraphPort for RewriteRoot {
    fn head(&self, n: Particle) -> Result<Option<Head>, Error> {
        self.0.head(n)
    }
    fn resolve(&self, n: Particle, r: Particle) -> Result<Option<Head>, Error> {
        self.0.resolve(n, r)
    }
    fn history(&self, n: Particle, a: Option<u64>, l: usize) -> Result<Vec<Head>, Error> {
        self.0.history(n, a, l)
    }
    fn commit(
        &self,
        n: Particle,
        r: Particle,
        e: Option<Head>,
        h: Head,
        mut b: Builder,
        c: Option<(Particle, Particle)>,
    ) -> Result<Head, Error> {
        assert_eq!(h.index, 0);
        let (mut state, auth) = {
            let overlay = Overlay {
                graph: &self.0,
                batch: &b,
            };
            let mut reader = Reader::new(&overlay, 100_000);
            let f = reader.record(h.commit, "neuron/activation/1", 3)?;
            let root = reader.reference(f[1])?;
            (
                NeuronState::read(&mut reader, root)?,
                reader.reference(f[2])?,
            )
        };
        state.limit += 1;
        let root = state.encode(&mut b)?;
        let commit = b.values("neuron/activation/1", &[Ref(n), Ref(root), Ref(auth)])?;
        self.0.commit(n, r, e, Head { index: 0, commit }, b, c)
    }
}
#[test]
fn valid_signature_cannot_be_reused_for_a_different_canonical_root() {
    let dir = Directory::new();
    let (auth, _) = authority();
    let id = auth.subject();
    let agent = Neuron::with_authority(RewriteRoot(dir.graph()), Rune, auth);
    assert!(agent.activate(id, NETWORK, POLICY, 1000).is_err());
    assert!(agent.graph.head(id).unwrap().is_none());
}
