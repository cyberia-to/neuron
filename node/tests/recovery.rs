#[path = "support/legacy.rs"]
mod legacy;
mod support;
use neuron_engine::{
    Error, GraphPort, ImportOrigin, JobProgress, MigrationPort, MigrationWrite, Neuron,
};
use neuron_model::{Builder, Content, Head, Particle, Reader, Source, execution::Status};
use neuron_node::{Graph, Rune};
use std::sync::atomic::{AtomicUsize, Ordering};
use support::*;
struct FailGraph {
    graph: Graph,
    fail: AtomicUsize,
    after_commit: bool,
}
impl FailGraph {
    fn fail_now(&self) -> bool {
        self.fail
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
            .ok()
            == Some(1)
    }
    fn perform(&self, f: impl FnOnce() -> Result<Head, Error>) -> Result<Head, Error> {
        let fail = self.fail_now();
        if fail && !self.after_commit {
            return Err(Error::Graph("injected failure before commit".into()));
        }
        let result = f()?;
        if fail {
            return Err(Error::CommitUnknown(
                "injected loss of committed receipt".into(),
            ));
        }
        Ok(result)
    }
}
impl Source for FailGraph {
    fn get(&self, id: &Particle) -> Result<Content, neuron_model::Error> {
        self.graph.get(id)
    }
}
impl GraphPort for FailGraph {
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
        n: Particle,
        r: Particle,
        e: Option<Head>,
        h: Head,
        b: Builder,
        c: Option<(Particle, Particle)>,
    ) -> Result<Head, Error> {
        self.perform(|| self.graph.commit(n, r, e, h, b, c))
    }
}
impl MigrationPort for FailGraph {
    fn commit_migration(&self, write: MigrationWrite) -> Result<Head, Error> {
        self.perform(|| self.graph.commit_migration(write))
    }
}
#[test]
fn interrupted_slice_charges_reservation_after_reopen_and_cannot_reset_subject_budget() {
    let dir = Directory::new();
    let (id, prog, job);
    {
        let (auth, _) = authority();
        id = auth.subject();
        let agent = Neuron::with_authority(
            FailGraph {
                graph: dir.graph(),
                fail: AtomicUsize::new(0),
                after_commit: false,
            },
            Rune,
            auth,
        );
        agent.activate(id, NETWORK, POLICY, 1000).unwrap();
        prog = agent
            .install(id, [1; 32], b"~mem + event".to_vec(), value("5"), config())
            .unwrap();
        job = agent
            .submit(id, admission(prog, 2, "7", 1000))
            .unwrap()
            .invocation;
        agent.graph.fail.store(2, Ordering::SeqCst);
        assert!(matches!(
            agent.tick_invocation(id, job),
            Err(Error::Graph(_))
        ));
        assert_eq!(
            agent.inspect(id).unwrap().state.invocations[&job].reserved,
            1000
        );
    }
    let (agent, _) = dir.open();
    assert!(matches!(
        agent.tick_invocation(id, job).unwrap(),
        JobProgress::Yielded { .. }
    ));
    assert!(matches!(
        agent.tick_invocation(id, job).unwrap(),
        JobProgress::Finished {
            status: Status::Failed,
            ..
        }
    ));
    let view = agent.inspect(id).unwrap();
    assert_eq!(view.state.charged, 1000);
    assert_eq!(view.state.held, 0);
    assert_eq!(agent.state(id, prog).unwrap(), value("5"));
    assert!(matches!(
        agent.submit(id, admission(prog, 3, "7", 1000)),
        Err(Error::Budget)
    ));
    assert!(matches!(
        agent.activate(id, NETWORK, POLICY, 2000),
        Err(Error::Conflict)
    ));
}

#[test]
fn lost_admission_receipt_resolves_without_second_invocation() {
    let dir = Directory::new();
    let (auth, _) = authority();
    let id = auth.subject();
    let agent = Neuron::with_authority(
        FailGraph {
            graph: dir.graph(),
            fail: AtomicUsize::new(0),
            after_commit: true,
        },
        Rune,
        auth,
    );
    agent.activate(id, NETWORK, POLICY, 10_000).unwrap();
    let p = agent
        .install(id, [1; 32], b"~mem + event".to_vec(), value("5"), config())
        .unwrap();
    agent.graph.fail.store(1, Ordering::SeqCst);
    assert!(matches!(
        agent.submit(id, admission(p, 2, "7", 4000)),
        Err(Error::CommitUnknown(_))
    ));
    let old = agent.inspect(id).unwrap();
    let receipt = agent.submit(id, admission(p, 2, "7", 4000)).unwrap();
    assert_eq!(receipt.head, old.head);
    assert_eq!(agent.inspect(id).unwrap().state.invocations.len(), 1);
    assert!(matches!(
        agent.tick_invocation(id, receipt.invocation).unwrap(),
        JobProgress::Finished {
            status: Status::Completed,
            ..
        }
    ));
    assert_eq!(agent.state(id, p).unwrap(), value("12"));
    assert!(matches!(agent.tick(id).unwrap(), JobProgress::Idle));
}

#[test]
fn each_sequential_act_has_its_own_consumption_and_retained_charge() {
    let dir = Directory::new();
    let (agent, _) = dir.open();
    let id = activate(&agent, 10_000);
    let p = install(&agent, id, 1, "let x = host(1); let y = host(x); x + y");
    let job = agent
        .submit(id, admission(p, 2, "0", 4000))
        .unwrap()
        .invocation;
    let mut operations = Vec::new();
    loop {
        match drive(&agent, id, job) {
            JobProgress::Awaiting { operation, .. } => {
                assert!(!operations.contains(&operation));
                operations.push(operation);
                let d = agent.begin_attempt(id, job).unwrap();
                agent
                    .record_outcome(
                        id,
                        job,
                        operation,
                        d.attempt,
                        value(if operations.len() == 1 { "10" } else { "32" }),
                    )
                    .unwrap();
            }
            JobProgress::Finished {
                status: Status::Completed,
                ..
            } => break,
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(operations.len(), 2);
    assert_eq!(agent.state(id, p).unwrap(), value("42"));
    let mut consumed = Vec::new();
    for head in agent.graph.history(id, Some(0), 100).unwrap() {
        let mut r = Reader::new(&agent.graph, 50_000);
        let c = r.record(head.commit, "neuron/commit/1", 8).unwrap();
        for record in r.list(c[6], 32).unwrap() {
            let record = r.reference(record).unwrap();
            if let Ok(f) = r.record(record, "neuron/consumed/1", 3) {
                consumed.push(r.reference(f[0]).unwrap());
            }
        }
    }
    assert_eq!(consumed, operations);
    let view = agent.inspect(id).unwrap();
    let j = &view.state.invocations[&job];
    assert_eq!(j.charged, j.used);
    assert!(j.used > 0);
    assert_eq!(view.state.charged, j.charged);
}

#[test]
fn activation_failure_or_lost_receipt_resumes_without_partial_fences() {
    for after_commit in [false, true] {
        let dir = Directory::new();
        let (auth, _) = authority();
        let id = auth.subject();
        let agent = Neuron::with_authority(
            FailGraph {
                graph: dir.graph(),
                fail: AtomicUsize::new(0),
                after_commit,
            },
            Rune,
            auth,
        );
        legacy::replay(&agent.graph, legacy::CAPTURE).unwrap();
        agent.activate(id, NETWORK, POLICY, 3_000_000).unwrap();
        let origins = || {
            vec![
                ImportOrigin {
                    origin: legacy::named("counter"),
                    install_nonce: [20; 32],
                },
                ImportOrigin {
                    origin: legacy::named("tool"),
                    install_nonce: [21; 32],
                },
            ]
        };
        agent.graph.fail.store(1, Ordering::SeqCst);
        assert!(agent.import(id, [22; 32], origins()).is_err());
        for o in origins() {
            assert_eq!(
                agent
                    .graph
                    .graph
                    .0
                    .migration_target(&o.origin)
                    .unwrap()
                    .is_some(),
                after_commit
            );
        }
        drop(agent);
        let (agent, _) = dir.open();
        let receipt = agent.import(id, [22; 32], origins()).unwrap();
        assert_eq!(receipt.mappings.len(), 2);
        let retry = agent.import(id, [22; 32], origins()).unwrap();
        assert_eq!(retry.head, receipt.head);
        assert!(matches!(
            drive(
                &agent,
                id,
                receipt
                    .mappings
                    .iter()
                    .find(|m| m.origin == legacy::named("tool"))
                    .unwrap()
                    .invocation
                    .unwrap()
            ),
            JobProgress::Unknown { .. }
        ));
    }
}
