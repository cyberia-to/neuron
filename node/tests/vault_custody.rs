#![cfg(feature = "vault")]
#[path = "support/vault.rs"]
mod fixture;
mod support;
use neuron_engine::{Authority, Error, GraphPort, Neuron, ProgramConfig};
use neuron_model::{Lifecycle, Reader};
use neuron_node::{Grant, GrantHandle, LocalAuthority, Rune, custody::LocalVault};
use support::{Directory, NETWORK, POLICY};

#[test]
fn encrypted_custody_signs_real_transitions_and_revocation_stops_publication() {
    let dir = Directory::new();
    let home = fixture::create(&dir.0);
    let signer = fixture::open(&home);
    let id = signer.subject();
    let grant = GrantHandle::new(Grant {
        neuron: id,
        network: NETWORK,
        policy: POLICY,
        epoch: 0,
        revision: 0,
        enabled: true,
        acts: [].into(),
        progs: None,
    })
    .unwrap();
    let authority = LocalAuthority::new(LocalVault::new(signer), grant.clone()).unwrap();
    let agent = Neuron::with_authority(dir.graph(), Rune, authority);
    agent.activate(id, NETWORK, POLICY, 100_000).unwrap();
    let prog = agent
        .install(
            id,
            [6; 32],
            b"event".to_vec(),
            neuron_rune::value("0").unwrap(),
            ProgramConfig {
                step_limit: 1000,
                max_inflight: 1,
                allowed_acts: vec![],
            },
        )
        .unwrap();
    for head in agent.graph.history(id, None, 20).unwrap() {
        let mut r = Reader::new(&agent.graph, 50_000);
        let f = if head.index == 0 {
            r.record(head.commit, "neuron/activation/1", 3).unwrap()
        } else {
            r.record(head.commit, "neuron/commit/1", 8).unwrap()
        };
        let auth = r.reference(f[if head.index == 0 { 2 } else { 7 }]).unwrap();
        let f = r.record(auth, "neuron/authorization/1", 6).unwrap();
        let statement = r.reference(f[4]).unwrap();
        let evidence = r.reference(f[5]).unwrap();
        // Evidence is a plain Blob; no spell/root/private-key is graph content.
        let bytes = r.content(evidence).unwrap().bytes;
        assert!(mudra::neuron::verify_statement(id, statement, &bytes));
    }
    let head = agent.graph.head(id).unwrap();
    let mut next = grant.get().unwrap();
    next.revision += 1;
    next.enabled = false;
    grant.replace(0, next).unwrap();
    assert!(matches!(
        agent.manage(id, prog, Lifecycle::Paused),
        Err(Error::Denied)
    ));
    assert_eq!(agent.graph.head(id).unwrap(), head);
    // The publication guard independently rejects a previously signed decision.
    let action = neuron_engine::Action {
        neuron: id,
        network: NETWORK,
        policy: POLICY,
        epoch: 0,
        prog: Some(prog),
        invocation: None,
        act: None,
        kind: "pause",
        statement: [1; 32],
        allowed_acts: &[],
    };
    assert!(matches!(
        agent
            .authority
            .with_current(&action, &mut || panic!("published")),
        Err(Error::Denied)
    ));
    drop(agent);
    assert_eq!(fixture::open(&home).subject(), id);
}
