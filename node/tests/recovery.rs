use cell_engine::{Config, Engine, Error, GraphPort, Progress};
use cell_model::{Builder, Content, Head, Particle, Reader, Source};
use cell_node::{Graph, LocalWard, Rune};
use std::sync::atomic::{AtomicUsize, Ordering};

struct FailGraph {
    graph: Graph,
    fail: AtomicUsize,
    after_commit: bool,
}
impl Source for FailGraph {
    fn get(&self, id: &Particle) -> Result<Content, cell_model::Error> {
        self.graph.get(id)
    }
}
impl GraphPort for FailGraph {
    fn head(&self, cell: Particle) -> Result<Option<Head>, Error> {
        self.graph.head(cell)
    }
    fn resolve(&self, cell: Particle, request: Particle) -> Result<Option<Head>, Error> {
        self.graph.resolve(cell, request)
    }
    fn history(
        &self,
        cell: Particle,
        after: Option<u64>,
        limit: usize,
    ) -> Result<Vec<Head>, Error> {
        self.graph.history(cell, after, limit)
    }
    fn commit(
        &self,
        cell: Particle,
        request: Particle,
        expected: Option<Head>,
        head: Head,
        content: Builder,
        claim: Option<(Particle, Particle)>,
    ) -> Result<Head, Error> {
        let fail = self
            .fail
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
            .ok()
            == Some(1);
        if fail && !self.after_commit {
            return Err(Error::Graph("injected failure before commit".into()));
        }
        let result = self
            .graph
            .commit(cell, request, expected, head, content, claim)?;
        if fail {
            return Err(Error::CommitUnknown(
                "injected loss of commit receipt".into(),
            ));
        }
        Ok(result)
    }
}
struct Directory(std::path::PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "cell-recovery-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn graph(&self) -> Graph {
        Graph::open(self.0.join("bbg")).unwrap()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn value(s: &str) -> Vec<u8> {
    cell_rune::value(s).unwrap()
}

#[test]
fn interrupted_slice_consumes_reservation_after_reopen() {
    let dir = Directory::new();
    let cell;
    {
        let engine = Engine::new(
            FailGraph {
                graph: dir.graph(),
                fail: AtomicUsize::new(0),
                after_commit: false,
            },
            Rune,
        );
        cell = engine
            .create(
                b"~mem + event".to_vec(),
                value("5"),
                [1; 32],
                Config {
                    step_limit: 1000,
                    ..Config::default()
                },
            )
            .unwrap();
        engine.submit(cell, [2; 32], value("7"), None).unwrap();
        engine.graph.fail.store(2, Ordering::SeqCst);
        assert!(matches!(engine.tick(cell), Err(Error::Graph(_))));
        assert_eq!(engine.inspect(cell).unwrap().live.unwrap().reserved, 1000);
    }
    let engine = Engine::new(dir.graph(), Rune);
    assert!(matches!(engine.tick(cell), Err(Error::Budget)));
    let inspection = engine.inspect(cell).unwrap();
    assert_eq!(inspection.state, value("5"));
    assert!(inspection.live.is_none());
    let mut r = Reader::new(&engine.graph, 20_000);
    let commit = r
        .record(inspection.head.commit, "cell/commit/1", 10)
        .unwrap();
    let evidence = r.list(commit[6], 2).unwrap();
    let usage = r.reference(evidence[0]).unwrap();
    let usage = r.record(usage, "cell/local-resources/1", 4).unwrap();
    assert_eq!(r.uint(usage[0]).unwrap(), 1000);
}

#[test]
fn lost_admission_receipt_is_resolved_without_second_execution() {
    let dir = Directory::new();
    let engine = Engine::new(
        FailGraph {
            graph: dir.graph(),
            fail: AtomicUsize::new(0),
            after_commit: true,
        },
        Rune,
    );
    let cell = engine
        .create(
            b"~mem + event".to_vec(),
            value("5"),
            [3; 32],
            Config::default(),
        )
        .unwrap();
    engine.graph.fail.store(1, Ordering::SeqCst);
    assert!(matches!(
        engine.submit(cell, [4; 32], value("7"), None),
        Err(Error::CommitUnknown(_))
    ));
    let head = engine.inspect(cell).unwrap().head;
    assert_eq!(
        engine.submit(cell, [4; 32], value("7"), None).unwrap(),
        head
    );
    assert!(matches!(engine.tick(cell).unwrap(), Progress::Complete(_)));
    assert_eq!(engine.inspect(cell).unwrap().state, value("12"));
    assert!(matches!(engine.tick(cell).unwrap(), Progress::Idle));
}

#[test]
fn result_consumption_and_usage_are_retained_for_each_sequential_act() {
    let dir = Directory::new();
    let engine = Engine::with_ward(dir.graph(), Rune, LocalWard);
    let cell = engine
        .create(
            b"let x = host(1); let y = host(x); x + y".to_vec(),
            value("9"),
            [5; 32],
            Config {
                allowed_acts: vec![0xAC75_0000_0000_0006],
                ..Config::default()
            },
        )
        .unwrap();
    engine.submit(cell, [6; 32], value("0"), None).unwrap();
    let mut operations = Vec::new();
    loop {
        match engine.tick(cell).unwrap() {
            Progress::Advanced(_) => (),
            Progress::Awaiting { operation, .. } => {
                assert!(!operations.contains(&operation));
                operations.push(operation);
                let attempt = engine.begin_attempt(cell, operation).unwrap();
                engine
                    .record_outcome(
                        cell,
                        operation,
                        attempt.attempt,
                        value(if operations.len() == 1 { "10" } else { "32" }),
                    )
                    .unwrap();
            }
            Progress::Complete(_) => break,
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(operations.len(), 2);
    assert_eq!(engine.inspect(cell).unwrap().state, value("42"));
    let mut consumed = Vec::new();
    let mut usage = None;
    for head in engine.graph.history(cell, Some(0), 100).unwrap() {
        let mut r = Reader::new(&engine.graph, 20_000);
        let c = r.record(head.commit, "cell/commit/1", 10).unwrap();
        for record in r.list(c[6], 8).unwrap() {
            let id = r.reference(record).unwrap();
            if let Ok(f) = r.record(id, "cell/outbox-entry/1", 5)
                && r.optional_ref(f[4]).unwrap().is_some()
            {
                consumed.push(id);
            }
            if let Ok(f) = r.record(id, "cell/local-resources/1", 4) {
                usage = Some((r.uint(f[0]).unwrap(), r.uint(f[2]).unwrap()));
            }
        }
    }
    assert_eq!(consumed.len(), 2);
    let (charged, used) = usage.unwrap();
    assert_eq!(charged, used);
    assert!(used > 0);
}
