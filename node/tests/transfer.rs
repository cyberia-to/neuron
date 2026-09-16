#[path = "support/legacy.rs"]
mod legacy;
mod support;
use neuron_engine::{Error, GraphPort, ImportOrigin, JobProgress};
use neuron_node::{Graph, stage_legacy};
use support::*;

fn source() -> Directory {
    let dir = Directory::new();
    legacy::replay(&dir.graph(), legacy::CAPTURE).unwrap();
    dir
}
fn origins() -> Vec<ImportOrigin> {
    ["counter", "tool", "reserved"]
        .into_iter()
        .enumerate()
        .map(|(i, name)| ImportOrigin {
            origin: legacy::named(name),
            install_nonce: [i as u8 + 10; 32],
        })
        .collect()
}

#[test]
fn separate_store_resumes_every_page_and_preserves_history_unknowns_and_other_namespaces() {
    let source = source();
    let target = Directory::new();
    let id;
    let mut progress = None;
    {
        let (agent, _) = target.open();
        id = activate(&agent, 3_000_000);
        let prog = install(&agent, id, 1, "~mem + event");
        let job = agent
            .submit(id, admission(prog, 2, "5", 4000))
            .unwrap()
            .invocation;
        drive(&agent, id, job);
    }
    for _ in 0..100 {
        let (agent, _) = target.open();
        let next = stage_legacy(&agent, id, &source.0.join("bbg"), [80; 32], 1).unwrap();
        if let Some(prior) = progress.as_ref() {
            let prior: &cybergraph::application::TransferProgress = prior;
            assert_eq!(prior.manifest, next.manifest);
            assert!(next.rows >= prior.rows && next.bytes >= prior.bytes);
        }
        let done = next.complete;
        progress = Some(next);
        assert!(Graph::open(source.0.join("bbg")).is_err());
        assert_eq!(agent.inspect(id).unwrap().state.progs.len(), 1);
        if done {
            break;
        }
        // No effect or semantic import appears while copying.
        assert!(agent.inspect(id).unwrap().state.imports.is_empty());
    }
    let progress = progress.unwrap();
    assert!(progress.complete && progress.rows > 3000);
    let (agent, _) = target.open();
    let retry = stage_legacy(&agent, id, &source.0.join("bbg"), [80; 32], 1).unwrap();
    assert_eq!(retry, progress);
    assert!(stage_legacy(&agent, id, &source.0.join("bbg"), [81; 32], 1).is_err());
    for origin in origins() {
        assert!(matches!(
            legacy::advance(&agent.graph, origin.origin),
            Err(Error::Fenced)
        ));
    }
    let receipt = agent.import(id, [90; 32], origins()).unwrap();
    let counter = receipt
        .mappings
        .iter()
        .find(|m| m.origin == legacy::named("counter"))
        .unwrap();
    assert_eq!(agent.state(id, counter.prog).unwrap(), value("7"));
    assert_eq!(
        neuron_engine::legacy::resolve(&agent.graph, counter.origin, [2; 32])
            .unwrap()
            .unwrap()
            .commit,
        legacy::named("completed_request")
    );
    let tool = receipt
        .mappings
        .iter()
        .find(|m| m.origin == legacy::named("tool"))
        .unwrap();
    assert!(
        matches!(drive(&agent,id,tool.invocation.unwrap()),JobProgress::Unknown {operation,attempt,..}
        if operation==legacy::named("operation") && attempt==legacy::named("attempt"))
    );
    assert_eq!(agent.inspect(id).unwrap().state.progs.len(), 4);
    assert!(
        agent
            .graph
            .history(counter.origin, None, 100)
            .unwrap()
            .len()
            > 1
    );
    assert_eq!(
        agent.import(id, [90; 32], origins()).unwrap().manifest,
        receipt.manifest
    );
    drop(agent);
    if let Some(binary) = std::env::var_os("NEURON_LEGACY_BINARY") {
        for path in [source.0.join("bbg"), target.0.join("bbg")] {
            let output = std::process::Command::new(&binary)
                .arg("--store")
                .arg(path)
                .arg("inspect")
                .arg(
                    legacy::named("counter")
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>(),
                )
                .output()
                .unwrap();
            assert!(!output.status.success());
        }
    }
}

#[test]
fn denied_or_busy_source_cannot_begin_transfer() {
    let source = source();
    let target = Directory::new();
    let (agent, grant) = target.open();
    let id = activate(&agent, 3_000_000);
    let live = source.graph();
    assert!(stage_legacy(&agent, id, &source.0.join("bbg"), [80; 32], 1).is_err());
    drop(live);
    let mut revoked = grant.get().unwrap();
    revoked.enabled = false;
    revoked.revision = 1;
    grant.replace(0, revoked).unwrap();
    assert!(matches!(
        stage_legacy(&agent, id, &source.0.join("bbg"), [80; 32], 1),
        Err(Error::Denied)
    ));
    assert!(
        source
            .graph()
            .head(legacy::named("counter"))
            .unwrap()
            .is_some()
    );
}
