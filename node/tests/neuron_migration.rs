#[path = "support/legacy.rs"]
mod legacy;
mod support;
use neuron_engine::{Error, GraphPort, ImportOrigin, JobProgress};
use neuron_model::execution::Status;
use support::*;
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
fn three_legacy_origins_converge_preserving_history_claims_unknown_effect_and_reserved_work() {
    let dir = Directory::new();
    let (id, receipt, old_heads, old_charge);
    {
        let (agent, _) = dir.open();
        legacy::replay(&agent.graph, legacy::CAPTURE).unwrap();
        old_heads = origins()
            .iter()
            .map(|o| (o.origin, agent.graph.head(o.origin).unwrap().unwrap()))
            .collect::<Vec<_>>();
        old_charge = origins()
            .iter()
            .map(|o| {
                neuron_engine::legacy::inspect(&agent.graph, o.origin)
                    .unwrap()
                    .charged
            })
            .sum::<u64>();
        id = activate(&agent, 3_000_000);
        receipt = agent.import(id, [90; 32], origins()).unwrap();
        let view = agent.inspect(id).unwrap();
        assert_eq!(view.state.progs.len(), 3);
        assert_eq!(view.state.invocations.len(), 2);
        assert_eq!(view.state.charged, old_charge);
        for (origin, head) in &old_heads {
            assert_eq!(agent.graph.head(*origin).unwrap(), Some(*head));
            let fence = agent.graph.0.migration_target(origin).unwrap().unwrap();
            assert_eq!(fence.namespace, id);
            assert_eq!(fence.manifest, receipt.manifest);
        }
        // This is the actual pre-migration binary captured in the implementation
        // baseline. CI without that historical executable still exercises the
        // writer fence below; the migration rehearsal supplies it explicitly.
        if let Some(binary) = std::env::var_os("NEURON_LEGACY_BINARY") {
            drop(agent);
            let output = std::process::Command::new(binary)
                .arg("--store")
                .arg(dir.0.join("bbg"))
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
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("Storage(Corrupt)"),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    let (agent, _) = dir.open();
    let retry = agent.import(id, [90; 32], origins()).unwrap();
    assert_eq!(retry.head, receipt.head);
    assert_eq!(retry.mappings, receipt.mappings);
    let mut changed = origins();
    changed[0].install_nonce = [99; 32];
    assert!(matches!(
        agent.import(id, [90; 32], changed),
        Err(Error::Conflict)
    ));
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
    // The old engine code path cannot create a successor once the origin is fenced.
    assert!(matches!(
        legacy::advance(&agent.graph, counter.origin),
        Err(Error::Fenced)
    ));
    let tool = receipt
        .mappings
        .iter()
        .find(|m| m.origin == legacy::named("tool"))
        .unwrap();
    let job = tool.invocation.unwrap();
    assert!(
        matches!(drive(&agent,id,job),JobProgress::Unknown{operation,attempt,..} if operation==legacy::named("operation") && attempt==legacy::named("attempt"))
    );
    assert!(matches!(
        agent.begin_attempt(id, job),
        Err(Error::UnknownOutcome)
    ));
    agent
        .record_outcome(
            id,
            job,
            legacy::named("operation"),
            legacy::named("attempt"),
            value("42"),
        )
        .unwrap();
    assert!(matches!(
        drive(&agent, id, job),
        JobProgress::Finished {
            status: Status::Completed,
            ..
        }
    ));
    assert_eq!(agent.state(id, tool.prog).unwrap(), value("42"));
    let reserved = receipt
        .mappings
        .iter()
        .find(|m| m.origin == legacy::named("reserved"))
        .unwrap();
    let job = reserved.invocation.unwrap();
    let before = agent.inspect(id).unwrap().state.charged;
    assert!(matches!(
        agent.tick_invocation(id, job).unwrap(),
        JobProgress::Yielded { .. }
    ));
    assert_eq!(agent.inspect(id).unwrap().state.charged, before + 1000);
    assert!(matches!(
        drive(&agent, id, job),
        JobProgress::Finished {
            status: Status::Completed,
            ..
        }
    ));
    assert_eq!(agent.state(id, reserved.prog).unwrap(), value("12"));
    assert_eq!(agent.inspect(id).unwrap().state.held, 0);
    verify_history(&agent, id);
}

#[test]
fn stale_target_or_source_heads_publish_neither_mapping_nor_fences() {
    let dir = Directory::new();
    let (agent, _) = dir.open();
    legacy::replay(&agent.graph, legacy::CAPTURE).unwrap();
    let id = activate(&agent, 4_000_000);
    let plan = agent.prepare_import(id, [90; 32], origins()).unwrap();
    install(&agent, id, 70, "event");
    assert!(matches!(agent.activate_import(plan), Err(Error::Conflict)));
    assert!(agent.inspect(id).unwrap().state.imports.is_empty());
    let plan = agent.prepare_import(id, [90; 32], origins()).unwrap();
    legacy::advance(&agent.graph, legacy::named("counter")).unwrap();
    assert!(matches!(agent.activate_import(plan), Err(Error::Conflict)));
    for o in origins() {
        assert!(agent.graph.0.migration_target(&o.origin).unwrap().is_none());
    }
    assert!(agent.inspect(id).unwrap().state.imports.is_empty());
    agent.import(id, [90; 32], origins()).unwrap();
}

#[test]
fn revocation_between_prepare_and_activate_keeps_origins_writable() {
    let dir = Directory::new();
    let (agent, grant) = dir.open();
    legacy::replay(&agent.graph, legacy::CAPTURE).unwrap();
    let id = activate(&agent, 4_000_000);
    let plan = agent.prepare_import(id, [90; 32], origins()).unwrap();
    let before = agent.inspect(id).unwrap().head;
    let mut revoked = grant.get().unwrap();
    revoked.enabled = false;
    revoked.revision += 1;
    grant.replace(0, revoked).unwrap();
    assert!(matches!(agent.activate_import(plan), Err(Error::Denied)));
    assert_eq!(agent.inspect(id).unwrap().head, before);
    for origin in origins() {
        assert!(
            agent
                .graph
                .0
                .migration_target(&origin.origin)
                .unwrap()
                .is_none()
        );
    }
    legacy::advance(&agent.graph, legacy::named("counter")).unwrap();
    let mut restored = grant.get().unwrap();
    restored.enabled = true;
    restored.revision += 1;
    grant.replace(1, restored).unwrap();
    let receipt = agent.import(id, [90; 32], origins()).unwrap();
    let mut revoked = grant.get().unwrap();
    revoked.enabled = false;
    revoked.revision += 1;
    grant.replace(2, revoked).unwrap();
    assert_eq!(
        agent.import(id, [90; 32], origins()).unwrap().head,
        receipt.head
    );
}
