mod support;
use neuron_engine::{Error, JobProgress};
use neuron_model::{Particle, execution::Status};
use neuron_node::{EffectOutcome, LocalWorker};
use support::*;

fn bind(agent: &Agent, id: Particle, boot: u8) -> LocalWorker {
    let generation = agent.inspect(id).unwrap().state.writer_generation;
    LocalWorker::bind(
        agent,
        id,
        generation,
        LocalWorker::descriptor(NETWORK, [41; 32], [42; 32], [boot; 32], vec![HOST], 4096),
    )
    .unwrap()
}
fn waiting(agent: &Agent, id: Particle, prog: Particle, nonce: u8) -> Particle {
    let job = agent
        .submit(id, admission(prog, nonce, "7", 4000))
        .unwrap()
        .invocation;
    assert!(matches!(
        drive(agent, id, job),
        JobProgress::Awaiting { .. }
    ));
    job
}

#[test]
fn guarded_tool_round_trip_and_unknown_recovery_do_not_repeat() {
    let dir = Directory::new();
    let (id, prog, unknown, attempt, operation);
    {
        let (agent, _) = dir.open();
        id = activate(&agent, 30_000);
        prog = install(&agent, id, 1, "let x = host(event); ~mem + x");
        let worker = bind(&agent, id, 1);
        let job = waiting(&agent, id, prog, 2);
        let token = worker.prepare(&agent, id, job).unwrap();
        let output = worker
            .execute(&agent, token, |_| EffectOutcome::Value(value("7")))
            .unwrap();
        output.reconciliation.unwrap().unwrap();
        assert!(matches!(
            drive(&agent, id, job),
            JobProgress::Finished {
                status: Status::Completed,
                ..
            }
        ));
        assert_eq!(agent.state(id, prog).unwrap(), value("7"));
        unknown = waiting(&agent, id, prog, 3);
        let token = worker.prepare(&agent, id, unknown).unwrap();
        attempt = token.metadata().attempt;
        operation = token.metadata().operation;
        let output = worker
            .execute(&agent, token, |_| {
                EffectOutcome::Unknown("provider connection lost".into())
            })
            .unwrap();
        assert!(output.reconciliation.is_none());
        assert!(
            agent.inspect(id).unwrap().state.invocations[&unknown]
                .pending
                .as_ref()
                .unwrap()
                .dispatch
                .is_some()
        );
        assert!(matches!(
            worker.prepare(&agent, id, unknown),
            Err(Error::UnknownOutcome)
        ));
    }
    let (agent, _) = dir.open();
    let worker = bind(&agent, id, 2);
    assert!(matches!(
        worker.prepare(&agent, id, unknown),
        Err(Error::UnknownOutcome)
    ));
    assert!(matches!(
        drive(&agent, id, unknown),
        JobProgress::Unknown { .. }
    ));
    agent
        .record_outcome(id, unknown, operation, attempt, value("9"))
        .unwrap();
    drive(&agent, id, unknown);
    assert_eq!(agent.state(id, prog).unwrap(), value("16"));
}

#[test]
fn current_revocation_and_replaced_boot_prevent_adapter_calls() {
    let dir = Directory::new();
    let (agent, grant) = dir.open();
    let id = activate(&agent, 30_000);
    let prog = install(&agent, id, 1, "host(event)");
    let old = bind(&agent, id, 1);
    let revoked_job = waiting(&agent, id, prog, 2);
    let token = old.prepare(&agent, id, revoked_job).unwrap();
    let mut revoked = grant.get().unwrap();
    revoked.enabled = false;
    revoked.revision += 1;
    grant.replace(0, revoked).unwrap();
    assert!(matches!(
        old.execute(&agent, token, |_| panic!("revoked effect")),
        Err(Error::Denied)
    ));
    assert!(
        agent.inspect(id).unwrap().state.invocations[&revoked_job]
            .pending
            .as_ref()
            .unwrap()
            .dispatch
            .is_none()
    );
    let mut restored = grant.get().unwrap();
    restored.enabled = true;
    restored.revision += 1;
    grant.replace(1, restored).unwrap();
    let job = waiting(&agent, id, prog, 3);
    let token = old.prepare(&agent, id, job).unwrap();
    let _new = bind(&agent, id, 2);
    assert!(matches!(
        old.execute(&agent, token, |_| panic!("stale boot effect")),
        Err(Error::Fenced)
    ));
    assert!(
        agent.inspect(id).unwrap().state.invocations[&job]
            .pending
            .as_ref()
            .unwrap()
            .dispatch
            .is_none()
    );
}

#[test]
fn unsupported_profile_wrong_network_and_tampered_input_fail_before_dispatch() {
    let dir = Directory::new();
    let (agent, _) = dir.open();
    let id = activate(&agent, 30_000);
    let mut descriptor =
        LocalWorker::descriptor(NETWORK, [1; 32], [2; 32], [3; 32], vec![HOST], 4096);
    descriptor.proof = "unimplemented-remote-proof/1".into();
    assert!(matches!(
        LocalWorker::bind(&agent, id, 0, descriptor),
        Err(Error::Unsupported)
    ));
    let wrong = LocalWorker::descriptor([90; 32], [1; 32], [2; 32], [3; 32], vec![HOST], 4096);
    assert!(matches!(
        LocalWorker::bind(&agent, id, 0, wrong),
        Err(Error::Denied)
    ));
    assert_eq!(agent.inspect(id).unwrap().state.writer_generation, 0);
    let prog = install(&agent, id, 1, "host(event)");
    let worker = bind(&agent, id, 1);
    let job = waiting(&agent, id, prog, 2);
    let token = worker.prepare(&agent, id, job).unwrap();
    // The public metadata can be cloned, but cannot construct an AttemptPermit
    // or DispatchToken. The only live token retains the exact admitted arguments.
    let mut display = token.metadata().clone();
    display.arguments = value("999");
    let execution = worker
        .execute(&agent, token, |actual| {
            assert_eq!(actual.arguments, value("7"));
            assert_ne!(actual.arguments, display.arguments);
            EffectOutcome::Value(actual.arguments.clone())
        })
        .unwrap();
    execution.reconciliation.unwrap().unwrap();
}

#[test]
fn claimed_effect_panic_survives_restart_without_redispatch() {
    let dir = Directory::new();
    let (id, job, operation, attempt);
    {
        let (agent, _) = dir.open();
        id = activate(&agent, 30_000);
        let prog = install(&agent, id, 1, "host(event)");
        let worker = bind(&agent, id, 1);
        job = waiting(&agent, id, prog, 2);
        let token = worker.prepare(&agent, id, job).unwrap();
        operation = token.metadata().operation;
        attempt = token.metadata().attempt;
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                worker.execute(&agent, token, |_| panic!("crash after durable claim"))
            }))
            .is_err()
        );
    }
    let (agent, _) = dir.open();
    let worker = bind(&agent, id, 2);
    assert!(matches!(
        worker.prepare(&agent, id, job),
        Err(Error::UnknownOutcome)
    ));
    assert!(
        agent.inspect(id).unwrap().state.invocations[&job]
            .pending
            .as_ref()
            .unwrap()
            .dispatch
            .is_some()
    );
    agent
        .record_outcome(id, job, operation, attempt, value("7"))
        .unwrap();
    assert!(matches!(
        drive(&agent, id, job),
        JobProgress::Finished {
            status: Status::Completed,
            ..
        }
    ));
}

#[test]
fn new_policy_can_reconcile_and_cancel_revoked_act_or_explicitly_rebind() {
    let dir = Directory::new();
    let (agent, grant) = dir.open();
    let id = activate(&agent, 30_000);
    let prog = install(&agent, id, 1, "host(event)");
    let worker = bind(&agent, id, 1);
    let first = waiting(&agent, id, prog, 2);
    let second = waiting(&agent, id, prog, 3);
    let a = worker.prepare(&agent, id, first).unwrap();
    let b = worker.prepare(&agent, id, second).unwrap();
    let a = worker
        .execute(&agent, a, |_| EffectOutcome::Unknown("lost".into()))
        .unwrap()
        .dispatch;
    let b = worker
        .execute(&agent, b, |_| EffectOutcome::Unknown("lost".into()))
        .unwrap()
        .dispatch;
    agent.change_policy(id, [91; 32], 1).unwrap();
    let mut updated = grant.get().unwrap();
    updated.policy = [91; 32];
    updated.epoch = 1;
    updated.revision = 1;
    updated.acts.clear();
    grant.replace(0, updated).unwrap();
    assert!(matches!(agent.rebind(id, first), Err(Error::Busy)));
    let receipt = agent
        .record_outcome(id, first, a.operation, a.attempt, value("7"))
        .unwrap();
    assert_eq!(
        agent
            .record_outcome(id, first, a.operation, a.attempt, value("7"))
            .unwrap(),
        receipt
    );
    assert!(matches!(agent.rebind(id, first), Err(Error::Denied)));
    agent
        .cancel(id, first, "capability revoked".into())
        .unwrap();
    agent.archive(id, first).unwrap();
    agent
        .record_outcome(id, second, b.operation, b.attempt, value("9"))
        .unwrap();
    let mut updated = grant.get().unwrap();
    updated.acts.insert(HOST);
    updated.revision = 2;
    grant.replace(1, updated).unwrap();
    agent.rebind(id, second).unwrap();
    assert!(matches!(
        drive(&agent, id, second),
        JobProgress::Finished {
            status: Status::Completed,
            ..
        }
    ));
    assert_eq!(agent.state(id, prog).unwrap(), value("9"));
}
