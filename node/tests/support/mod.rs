#![allow(dead_code)]
use neuron_engine::{Admission, GraphPort, JobProgress, Neuron, ProgramConfig};
use neuron_model::{Head, Particle, Reader};
use neuron_node::{Grant, GrantHandle, Graph, KeyVault, LocalAuthority, Rune, SigningVault};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
pub const HOST: u64 = 0xAC75_0000_0000_0006;
pub const NETWORK: Particle = [31; 32];
pub const POLICY: Particle = [32; 32];
pub type Agent = Neuron<Graph, Rune, LocalAuthority<KeyVault>>;
pub struct Directory(pub PathBuf);
impl Directory {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "neuron-state-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    pub fn graph(&self) -> Graph {
        Graph::open(self.0.join("bbg")).unwrap()
    }
    pub fn open(&self) -> (Agent, GrantHandle) {
        let (authority, grant) = authority();
        (Neuron::with_authority(self.graph(), Rune, authority), grant)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
pub fn authority() -> (LocalAuthority<KeyVault>, GrantHandle) {
    let vault = KeyVault::new(mudra::SigningKey::from_bytes((&[7u8; 32]).into()).unwrap());
    let grant = GrantHandle::new(Grant {
        neuron: vault.subject(),
        network: NETWORK,
        policy: POLICY,
        epoch: 0,
        revision: 0,
        enabled: true,
        acts: [HOST].into(),
        progs: None,
    })
    .unwrap();
    (LocalAuthority::new(vault, grant.clone()).unwrap(), grant)
}
pub fn value(source: &str) -> Vec<u8> {
    neuron_rune::value(source).unwrap()
}
pub fn config() -> ProgramConfig {
    ProgramConfig {
        step_limit: 4000,
        max_inflight: 4,
        allowed_acts: vec![HOST],
    }
}
pub fn activate(agent: &Agent, limit: u64) -> Particle {
    let id = agent.authority.subject();
    agent.activate(id, NETWORK, POLICY, limit).unwrap();
    id
}
pub fn install(agent: &Agent, id: Particle, nonce: u8, source: &str) -> Particle {
    agent
        .install(
            id,
            [nonce; 32],
            source.as_bytes().to_vec(),
            value("0"),
            config(),
        )
        .unwrap()
}
pub fn admission(prog: Particle, nonce: u8, input: &str, allowance: u64) -> Admission {
    Admission {
        prog,
        nonce: [nonce; 32],
        input: value(input),
        context: None,
        parent: None,
        allowance,
    }
}
pub fn drive(agent: &Agent, id: Particle, invocation: Particle) -> JobProgress {
    for _ in 0..100 {
        match agent.tick_invocation(id, invocation).unwrap() {
            JobProgress::Yielded { .. } => {}
            other => return other,
        }
    }
    panic!("did not reach execution boundary")
}
pub fn authorization(agent: &Agent, neuron: Particle, head: Head) -> Particle {
    let mut r = Reader::new(&agent.graph, 50_000);
    let f = if head.index == 0 {
        r.record(head.commit, "neuron/activation/1", 3).unwrap()
    } else {
        r.record(head.commit, "neuron/commit/1", 8).unwrap()
    };
    let auth = r.reference(f[if head.index == 0 { 2 } else { 7 }]).unwrap();
    let f = r.record(auth, "neuron/authorization/1", 6).unwrap();
    assert_eq!(r.reference(f[0]).unwrap(), neuron);
    let statement = r.reference(f[4]).unwrap();
    let proof = r.reference(f[5]).unwrap();
    let bytes = r.content(proof).unwrap().bytes;
    assert!(neuron_node::authority::verify_evidence(
        neuron, statement, &bytes
    ));
    assert!(!neuron_node::authority::verify_evidence(
        [9; 32], statement, &bytes
    ));
    statement
}
pub fn verify_history(agent: &Agent, id: Particle) {
    for head in agent.graph.history(id, None, 1000).unwrap() {
        authorization(agent, id, head);
    }
}
