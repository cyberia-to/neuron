#![cfg(feature = "query")]
mod support;
use inf_source::RelationSource;
use inf_value::Value;
use neuron_node::query;
use support::*;
#[test]
fn durable_execution_rows_keep_one_subject_and_exact_snapshot_without_authority() {
    let dir = Directory::new();
    let (id, prog, job, attempt, head, rows);
    {
        let (agent, grant) = dir.open();
        id = activate(&agent, 30_000);
        prog = install(&agent, id, 1, "let x = host(event); ~mem + x");
        let other = install(&agent, id, 2, "~mem + event");
        job = agent
            .submit(id, admission(prog, 3, "7", 4000))
            .unwrap()
            .invocation;
        drive(&agent, id, job);
        attempt = agent.begin_attempt(id, job).unwrap();
        let independent = agent
            .submit(id, admission(other, 4, "11", 4000))
            .unwrap()
            .invocation;
        drive(&agent, id, independent);
        head = agent.inspect(id).unwrap().head;
        let snapshot = query::read(&agent.graph, id).unwrap();
        assert!(!snapshot.provable());
        assert_eq!(snapshot.snapshot(), None);
        assert_eq!(snapshot.scan("progs").count(), 2);
        assert_eq!(snapshot.scan("invocations").count(), 2);
        let columns = snapshot.schema("operations").unwrap();
        rows = snapshot.scan("operations").collect::<Vec<_>>();
        assert_eq!(rows.len(), 1);
        for (name, value) in [
            ("neuron", Value::neuron(id)),
            ("network", Value::hash(NETWORK)),
            ("commit", Value::hash(head.commit)),
            ("revision", Value::Word(head.index)),
            ("invocation", Value::hash(job)),
            ("prog", Value::hash(prog)),
            ("attempt", Value::hash(attempt.attempt)),
        ] {
            assert_eq!(rows[0][columns.col(name).unwrap()], value);
        }
        for name in snapshot.all_relations() {
            assert!(!snapshot.writable(name));
        }
        let mut revoked = grant.get().unwrap();
        revoked.enabled = false;
        revoked.revision += 1;
        grant.replace(0, revoked).unwrap();
        // No activation or signing on the read path, even after grant revocation.
        assert_eq!(
            query::read(&agent.graph, id)
                .unwrap()
                .scan("operations")
                .collect::<Vec<_>>(),
            rows
        );
        assert!(query::read(&agent.graph, [99; 32]).is_err());
        assert_eq!(agent.inspect(id).unwrap().head, head);
    }
    let (agent, _) = dir.open();
    let old = query::read(&agent.graph, id).unwrap();
    assert_eq!(old.scan("operations").collect::<Vec<_>>(), rows);
    agent
        .record_outcome(id, job, attempt.operation, attempt.attempt, value("32"))
        .unwrap();
    drive(&agent, id, job);
    let new = query::read(&agent.graph, id).unwrap();
    assert_ne!(
        new.scan("neuron_runtime").next(),
        old.scan("neuron_runtime").next()
    );
    assert_eq!(old.scan("operations").collect::<Vec<_>>(), rows);
    let cols = new.schema("invocations").unwrap();
    let completed = new
        .scan("invocations")
        .find(|r| r[cols.col("invocation").unwrap()] == Value::hash(job))
        .unwrap();
    assert_eq!(
        completed[cols.col("status").unwrap()],
        Value::str("completed")
    );
}
