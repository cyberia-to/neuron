use serde_json::Value;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Store(std::path::PathBuf);
impl Store {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "cell-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn call(&self, args: &[&str]) -> (bool, Value) {
        let output = Command::new(env!("CARGO_BIN_EXE_cell"))
            .arg("--store")
            .arg(self.0.join("graph.redb"))
            .args(args)
            .output()
            .unwrap();
        let bytes = if output.status.success() {
            &output.stdout
        } else {
            &output.stderr
        };
        (
            output.status.success(),
            serde_json::from_slice(bytes)
                .unwrap_or_else(|_| panic!("{}", String::from_utf8_lossy(bytes))),
        )
    }
    fn ok(&self, args: &[&str]) -> Value {
        let (ok, value) = self.call(args);
        assert!(ok, "{value}");
        value
    }
    fn source(&self, text: &str) -> String {
        let path = self.0.join("cell.rune");
        std::fs::write(&path, text).unwrap();
        path.to_str().unwrap().to_owned()
    }
}
impl Drop for Store {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn counter_and_history_survive_separate_processes() {
    let store = Store::new();
    let source = store.source("~mem + event");
    let created = store.ok(&["create", &source]);
    let cell = created["cell"].as_str().unwrap();
    let nonce = "01".repeat(32);
    assert_eq!(
        store.ok(&["submit", cell, "7", "--nonce", &nonce])["state"],
        "7"
    );
    assert_eq!(
        store.ok(&["submit", cell, "7", "--nonce", &nonce])["state"],
        "7"
    );
    assert!(!store.call(&["submit", cell, "8", "--nonce", &nonce]).0);
    assert_eq!(store.ok(&["submit", cell, "5"])["state"], "12");
    let history = store.ok(&["history", cell]);
    assert_eq!(history["history"][0]["index"], 0);
    assert_eq!(history["history"].as_array().unwrap().len(), 8);
    assert_eq!(store.ok(&["inspect", cell])["state"], "12");
}

#[test]
fn executor_recovery_requires_correlated_outcome_and_pause_keeps_it() {
    let store = Store::new();
    let source = store.source("let x = host(event); ~mem + x");
    let created = store.ok(&["create", &source, "--initial", "10", "--allow", "host"]);
    let cell = created["cell"].as_str().unwrap();
    let pending = store.ok(&["submit", cell, "7", "--context", cell]);
    let operation = pending["operation"].as_str().unwrap();
    assert_eq!(pending["status"], "awaiting-executor");
    let receipt = store.ok(&["take", cell, operation]);
    let attempt = receipt["attempt"].as_str().unwrap();
    assert_eq!(store.ok(&["run", cell])["status"], "unknown-outcome");
    assert!(!store.call(&["take", cell, operation]).0);
    assert!(!store.call(&["cancel", cell]).0);
    store.ok(&["pause", cell]);
    assert_eq!(
        store.ok(&["outcome", cell, operation, attempt, "32"])["state"],
        "10"
    );
    assert_eq!(store.ok(&["resume", cell])["state"], "42");
    assert_eq!(
        store.ok(&["outcome", cell, operation, attempt, "32"])["state"],
        "42"
    );
    assert!(!store.call(&["outcome", cell, operation, attempt, "99"]).0);
    assert_eq!(store.ok(&["retire", cell])["lifecycle"], "retired");
    assert!(!store.call(&["resume", cell]).0);
}

#[test]
fn denied_effect_cannot_turn_into_successful_zero() {
    let store = Store::new();
    let source = store.source("let x = emit(event); 99");
    let created = store.ok(&["create", &source, "--initial", "10"]);
    let cell = created["cell"].as_str().unwrap();
    assert!(!store.call(&["submit", cell, "7"]).0);
    assert_eq!(store.ok(&["inspect", cell])["state"], "10");
    assert!(
        store.ok(&["history", cell])["history"]
            .as_array()
            .unwrap()
            .iter()
            .any(|h| h["events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["entry"] == "$denied"))
    );
}

#[test]
fn definite_failure_closes_attempt_and_retains_diagnostic() {
    let store = Store::new();
    let source = store.source("let x = host(event); 99");
    let created = store.ok(&["create", &source, "--initial", "10", "--allow", "host"]);
    let cell = created["cell"].as_str().unwrap();
    let pending = store.ok(&["submit", cell, "7"]);
    let operation = pending["operation"].as_str().unwrap();
    let taken = store.ok(&["take", cell, operation]);
    let attempt = taken["attempt"].as_str().unwrap();
    store.ok(&[
        "fail",
        cell,
        operation,
        attempt,
        "executor rejected request",
    ]);
    assert!(!store.call(&["run", cell]).0);
    let final_state = store.ok(&["inspect", cell]);
    assert_eq!(final_state["state"], "10");
    assert_eq!(final_state["fault"], "executor rejected request");
    assert!(final_state["invocation"].is_null());
    store.ok(&[
        "fail",
        cell,
        operation,
        attempt,
        "executor rejected request",
    ]);
    assert!(!store.call(&["outcome", cell, operation, attempt, "1"]).0);
    store.ok(&["retire", cell]);
}
