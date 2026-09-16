mod support;
use neuron_engine::{Error, JobProgress};
use neuron_model::{Lifecycle, execution::Status};
use support::*;

#[test]
fn one_subject_multiple_programs_concurrent_results_and_archived_claims() {
    let dir = Directory::new();
    let (id, p, q, a, b, c, receipt, charged);
    {
        let (agent, _) = dir.open();
        id = activate(&agent, 30_000);
        p = install(&agent, id, 1, "~mem + event");
        q = install(&agent, id, 2, "~mem + event");
        assert_ne!(p, q);
        assert_ne!(id, p);
        receipt = agent.submit(id, admission(p, 3, "7", 4000)).unwrap();
        a = receipt.invocation;
        b = agent
            .submit(id, admission(p, 4, "9", 4000))
            .unwrap()
            .invocation;
        c = agent
            .submit(id, admission(q, 5, "11", 4000))
            .unwrap()
            .invocation;
        assert!(matches!(
            drive(&agent, id, c),
            JobProgress::Finished {
                status: Status::Completed,
                ..
            }
        ));
        assert!(matches!(
            drive(&agent, id, a),
            JobProgress::Finished {
                status: Status::Completed,
                ..
            }
        ));
        assert!(matches!(
            drive(&agent, id, b),
            JobProgress::Finished {
                status: Status::Conflict,
                ..
            }
        ));
        assert_eq!(agent.state(id, p).unwrap(), value("7"));
        assert_eq!(agent.state(id, q).unwrap(), value("11"));
        let view = agent.inspect(id).unwrap();
        assert_eq!(view.state.held, 0);
        charged = view.state.charged;
        assert!(charged > 0);
        agent.archive(id, a).unwrap();
        verify_history(&agent, id);
    }
    let (agent, _) = dir.open();
    assert_eq!(agent.inspect(id).unwrap().state.charged, charged);
    assert_eq!(
        agent.submit(id, admission(p, 3, "7", 4000)).unwrap(),
        receipt
    );
    assert!(matches!(
        agent.submit(id, admission(q, 3, "7", 4000)),
        Err(Error::Conflict)
    ));
    assert!(matches!(
        agent.submit(id, admission(p, 3, "8", 4000)),
        Err(Error::Conflict)
    ));
    assert!(
        agent
            .inspect(id)
            .unwrap()
            .state
            .invocations
            .contains_key(&b)
    );
    assert_eq!(agent.state(id, q).unwrap(), value("11"));
}

#[test]
fn unknown_tool_does_not_block_other_program_and_outcome_is_consumed_once() {
    let dir = Directory::new();
    let (id, p, job, dispatch);
    {
        let (agent, _) = dir.open();
        id = activate(&agent, 30_000);
        p = install(&agent, id, 1, "let x = host(event); ~mem + x");
        let q = install(&agent, id, 2, "~mem + event");
        job = agent
            .submit(id, admission(p, 3, "7", 4000))
            .unwrap()
            .invocation;
        assert!(matches!(
            drive(&agent, id, job),
            JobProgress::Awaiting { tag: HOST, .. }
        ));
        dispatch = agent.begin_attempt(id, job).unwrap();
        let independent = agent
            .submit(id, admission(q, 4, "11", 4000))
            .unwrap()
            .invocation;
        assert!(
            matches!(agent.tick(id).unwrap(),JobProgress::Finished{invocation,..} if invocation==independent)
        );
        assert_eq!(agent.state(id, q).unwrap(), value("11"));
        assert!(matches!(
            agent.cancel(id, job, "stop".into()),
            Err(Error::UnknownOutcome)
        ));
    }
    let (agent, _) = dir.open();
    assert!(
        matches!(drive(&agent,id,job),JobProgress::Unknown{attempt,..} if attempt==dispatch.attempt)
    );
    assert!(matches!(
        agent.begin_attempt(id, job),
        Err(Error::UnknownOutcome)
    ));
    agent.manage(id, p, Lifecycle::Paused).unwrap();
    let head = agent
        .record_outcome(id, job, dispatch.operation, dispatch.attempt, value("32"))
        .unwrap();
    assert!(matches!(
        agent.tick_invocation(id, job),
        Err(Error::Lifecycle)
    ));
    agent.manage(id, p, Lifecycle::Active).unwrap();
    assert!(matches!(
        drive(&agent, id, job),
        JobProgress::Finished {
            status: Status::Completed,
            ..
        }
    ));
    assert_eq!(agent.state(id, p).unwrap(), value("32"));
    assert_eq!(
        agent
            .record_outcome(id, job, dispatch.operation, dispatch.attempt, value("32"))
            .unwrap(),
        head
    );
    assert!(matches!(
        agent.record_outcome(id, job, dispatch.operation, dispatch.attempt, value("33")),
        Err(Error::Conflict)
    ));
    assert_eq!(agent.inspect(id).unwrap().state.held, 0);
    verify_history(&agent, id);
}

#[test]
fn child_allowances_are_transferred_and_parent_waits_without_double_holding() {
    let dir = Directory::new();
    let (agent, _) = dir.open();
    let id = activate(&agent, 3000);
    let p = install(&agent, id, 1, "~mem + event");
    let q = install(&agent, id, 2, "~mem + event");
    let parent = agent
        .submit(id, admission(p, 3, "7", 2000))
        .unwrap()
        .invocation;
    let mut child = admission(q, 4, "11", 500);
    child.parent = Some(parent);
    let child = agent.submit(id, child).unwrap().invocation;
    assert_eq!(agent.inspect(id).unwrap().state.held, 2000);
    let mut excessive = admission(q, 5, "0", 1600);
    excessive.parent = Some(parent);
    assert!(matches!(agent.submit(id, excessive), Err(Error::Budget)));
    assert!(matches!(
        agent.cancel(id, parent, "stop".into()),
        Err(Error::Busy)
    ));
    assert!(matches!(
        drive(&agent, id, parent),
        JobProgress::Finished {
            status: Status::Joining,
            ..
        }
    ));
    assert_eq!(agent.state(id, p).unwrap(), value("0"));
    assert!(matches!(
        drive(&agent, id, child),
        JobProgress::Finished {
            status: Status::Completed,
            ..
        }
    ));
    let view = agent.inspect(id).unwrap();
    let parent = &view.state.invocations[&parent];
    assert_eq!(parent.status, Status::Completed);
    assert_eq!(parent.delegated, view.state.invocations[&child].charged);
    assert_eq!(view.state.held, 0);
    assert_eq!(view.state.charged, parent.charged + parent.delegated);
    assert_eq!(agent.state(id, p).unwrap(), value("7"));
}

#[test]
fn upgrade_retains_admitted_code_and_stale_result_cannot_replace_new_state() {
    let dir = Directory::new();
    let (agent, _) = dir.open();
    let id = activate(&agent, 10_000);
    let p = install(&agent, id, 1, "~mem + event");
    let job = agent
        .submit(id, admission(p, 2, "7", 4000))
        .unwrap()
        .invocation;
    agent
        .upgrade(id, p, b"~mem * event".to_vec(), value("10"))
        .unwrap();
    assert!(matches!(
        drive(&agent, id, job),
        JobProgress::Finished {
            status: Status::Conflict,
            ..
        }
    ));
    assert_eq!(agent.state(id, p).unwrap(), value("10"));
    let next = agent
        .submit(id, admission(p, 3, "3", 4000))
        .unwrap()
        .invocation;
    drive(&agent, id, next);
    assert_eq!(agent.state(id, p).unwrap(), value("30"));
    agent.manage(id, p, Lifecycle::Retiring).unwrap();
    agent.manage(id, p, Lifecycle::Retired).unwrap();
    assert_eq!(agent.inspect(id).unwrap().state.neuron, id);
}

#[test]
fn supported_signatures_bind_subject_network_policy_and_current_revocation() {
    let dir = Directory::new();
    let (agent, grant) = dir.open();
    let id = activate(&agent, 10_000);
    assert!(matches!(
        agent.activate([99; 32], NETWORK, POLICY, 100),
        Err(Error::Denied)
    ));
    // Another network cannot silently replace the activated namespace.
    assert!(matches!(
        agent.activate(id, [99; 32], POLICY, 10_000),
        Err(Error::Conflict)
    ));
    let p = install(&agent, id, 1, "host(event)");
    let job = agent
        .submit(id, admission(p, 2, "7", 4000))
        .unwrap()
        .invocation;
    drive(&agent, id, job);
    let before = agent.inspect(id).unwrap().head;
    let mut revoked = grant.get().unwrap();
    revoked.enabled = false;
    revoked.revision = 1;
    grant.replace(0, revoked.clone()).unwrap();
    assert!(matches!(agent.begin_attempt(id, job), Err(Error::Denied)));
    assert_eq!(agent.inspect(id).unwrap().head, before);
    revoked.enabled = true;
    revoked.revision = 2;
    grant.replace(1, revoked).unwrap();
    let dispatch = agent.begin_attempt(id, job).unwrap();
    assert_eq!(dispatch.neuron, id);
    assert_eq!(dispatch.network, NETWORK);
    verify_history(&agent, id);
}
