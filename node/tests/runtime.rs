use cell_engine::{Config, Engine, Error, Progress};
use cell_model::Lifecycle;
use cell_node::{Graph, Rune};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

struct StorePath(PathBuf);
impl StorePath {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "cell-runtime-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn open(&self) -> Engine<Graph, Rune> {
        Engine::with_ward(
            Graph::open(self.0.join("graph.redb")).unwrap(),
            Rune,
            cell_node::LocalWard,
        )
    }
}
impl Drop for StorePath {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn value(source: &str) -> Vec<u8> {
    cell_rune::value(source).unwrap()
}
fn drive(engine: &Engine<Graph, Rune>, cell: [u8; 32]) -> Result<Progress, Error> {
    for _ in 0..100 {
        match engine.tick(cell)? {
            Progress::Advanced(_) => (),
            other => return Ok(other),
        }
    }
    panic!("runtime did not reach a boundary")
}

#[test]
fn two_births_isolate_state_and_reopen_preserves_results_and_deduplication() {
    let path = StorePath::new();
    let (first, second, receipt);
    {
        let engine = path.open();
        first = engine
            .create(
                b"~mem + event".to_vec(),
                value("0"),
                [1; 32],
                Config::default(),
            )
            .unwrap();
        second = engine
            .create(
                b"~mem + event".to_vec(),
                value("0"),
                [2; 32],
                Config::default(),
            )
            .unwrap();
        assert_ne!(first, second);
        receipt = engine
            .submit(first, [3; 32], value("7"), Some(first))
            .unwrap();
        assert!(matches!(
            drive(&engine, first).unwrap(),
            Progress::Complete(_)
        ));
        assert_eq!(engine.inspect(first).unwrap().state, value("7"));
        assert_eq!(engine.inspect(second).unwrap().state, value("0"));
    }
    let engine = path.open();
    assert_eq!(engine.inspect(first).unwrap().state, value("7"));
    assert_eq!(
        engine
            .submit(first, [3; 32], value("7"), Some(first))
            .unwrap(),
        receipt
    );
    assert!(matches!(
        engine.submit(second, [3; 32], value("7"), Some(first)),
        Err(Error::Conflict)
    ));
    assert!(matches!(
        engine.submit(first, [3; 32], value("8"), Some(first)),
        Err(Error::Conflict)
    ));
    engine.submit(first, [4; 32], value("5"), None).unwrap();
    assert!(matches!(
        drive(&engine, first).unwrap(),
        Progress::Complete(_)
    ));
    assert_eq!(engine.inspect(first).unwrap().state, value("12"));
}

#[test]
fn accepted_effect_is_unknown_after_restart_and_recorded_result_resumes_once() {
    const HOST: u64 = 0xAC75_0000_0000_0006;
    let path = StorePath::new();
    let (cell, operation, attempt);
    {
        let engine = path.open();
        cell = engine
            .create(
                b"let x = host(7); ~mem + x".to_vec(),
                value("10"),
                [5; 32],
                Config {
                    allowed_acts: vec![HOST],
                    ..Config::default()
                },
            )
            .unwrap();
        engine
            .submit(cell, [6; 32], value("0"), Some(cell))
            .unwrap();
        operation = match drive(&engine, cell).unwrap() {
            Progress::Awaiting { operation, .. } => operation,
            other => panic!("{other:?}"),
        };
        let receipt = engine.begin_attempt(cell, operation).unwrap();
        attempt = receipt.attempt;
        assert!(matches!(
            engine.begin_attempt(cell, operation),
            Err(Error::UnknownOutcome)
        ));
    }
    let engine = path.open();
    assert!(
        matches!(drive(&engine, cell).unwrap(), Progress::Unknown { operation: op, attempt: at } if op == operation && at == attempt)
    );
    assert_eq!(
        engine.inspect(cell).unwrap().live.unwrap().context,
        Some(cell)
    );
    assert!(matches!(engine.cancel(cell), Err(Error::UnknownOutcome)));
    let receipt = engine
        .record_outcome(cell, operation, attempt, value("32"))
        .unwrap();
    assert!(matches!(
        drive(&engine, cell).unwrap(),
        Progress::Complete(_)
    ));
    assert_eq!(engine.inspect(cell).unwrap().state, value("42"));
    assert_eq!(
        engine
            .record_outcome(cell, operation, attempt, value("32"))
            .unwrap(),
        receipt
    );
    assert!(matches!(
        engine.record_outcome(cell, operation, attempt, value("99")),
        Err(Error::Conflict)
    ));
    assert!(matches!(drive(&engine, cell).unwrap(), Progress::Idle));
}

#[test]
fn lifecycle_and_denied_effects_preserve_state() {
    let path = StorePath::new();
    let engine = path.open();
    let cell = engine
        .create(b"host(1)".to_vec(), value("9"), [7; 32], Config::default())
        .unwrap();
    engine.manage(cell, Lifecycle::Paused).unwrap();
    assert!(matches!(
        engine.submit(cell, [8; 32], value("0"), None),
        Err(Error::Lifecycle)
    ));
    engine.manage(cell, Lifecycle::Active).unwrap();
    engine.submit(cell, [8; 32], value("0"), None).unwrap();
    let op = match drive(&engine, cell).unwrap() {
        Progress::Awaiting { operation, .. } => operation,
        other => panic!("{other:?}"),
    };
    assert!(matches!(engine.begin_attempt(cell, op), Err(Error::Denied)));
    assert_eq!(engine.inspect(cell).unwrap().state, value("9"));
    engine.cancel(cell).unwrap();
    engine.manage(cell, Lifecycle::Retiring).unwrap();
    engine.manage(cell, Lifecycle::Retired).unwrap();
    assert!(matches!(
        engine.manage(cell, Lifecycle::Active),
        Err(Error::Lifecycle)
    ));
}
