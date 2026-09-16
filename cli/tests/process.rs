use serde_json::{Value, json};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
#[path = "../../node/tests/support/legacy.rs"]
mod legacy;
struct Store(std::path::PathBuf);
impl Store {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "neuron-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        let store = Self(path);
        store.ok(&["keygen", store.0.join("owner.key").to_str().unwrap()]);
        store
    }
    fn call(&self, args: &[&str]) -> (bool, Value) {
        let output = Command::new(env!("CARGO_BIN_EXE_neuron"))
            .arg("--store")
            .arg(self.0.join("bbg"))
            .arg("--key-file")
            .arg(self.0.join("owner.key"))
            .args(["--grant-act", "host", "--grant-act", "emit"])
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
    fn activate(&self) -> String {
        self.ok(&["activate"])["neuron"]
            .as_str()
            .unwrap()
            .to_owned()
    }
    fn install(&self, neuron: &str, source: &str, initial: &str, allow: Option<&str>) -> String {
        let path = self.0.join("prog.rune");
        std::fs::write(&path, source).unwrap();
        let mut args = vec![
            "install",
            neuron,
            path.to_str().unwrap(),
            "--initial",
            initial,
        ];
        if let Some(name) = allow {
            args.extend(["--allow", name]);
        }
        self.ok(&args)["prog"].as_str().unwrap().to_owned()
    }
}
impl Drop for Store {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn one_identity_two_programs_and_receipts_survive_separate_processes() {
    let store = Store::new();
    let neuron = store.activate();
    let p = store.install(&neuron, "~mem + event", "0", None);
    let q = store.install(&neuron, "~mem + event", "10", None);
    assert_ne!(neuron, p);
    assert_ne!(p, q);
    let nonce = "01".repeat(32);
    let first = store.ok(&["submit", &neuron, &p, "7", "--nonce", &nonce]);
    assert_eq!(first["state"], "7");
    let retry = store.ok(&["submit", &neuron, &p, "7", "--nonce", &nonce]);
    assert_eq!(retry["admission_commit"], first["admission_commit"]);
    assert_eq!(retry["state"], "7");
    assert!(
        !store
            .call(&["submit", &neuron, &p, "8", "--nonce", &nonce])
            .0
    );
    assert_eq!(store.ok(&["submit", &neuron, &p, "5"])["state"], "12");
    assert_eq!(store.ok(&["inspect", &neuron, "--prog", &q])["state"], "10");
    let history = store.ok(&["history", &neuron]);
    assert_eq!(history["history"][0]["index"], 0);
    assert_eq!(history["provenance"], "neuron-v1");
    let view = store.ok(&["inspect", &neuron]);
    assert_eq!(view["progs"].as_array().unwrap().len(), 2);
    assert!(view.get("cell").is_none());
    assert_eq!(view["budget"]["held"], 0);
}

#[test]
fn executor_recovery_requires_correlated_outcome_and_pause_retains_it() {
    let store = Store::new();
    let neuron = store.activate();
    let p = store.install(&neuron, "let x = host(event); ~mem + x", "10", Some("host"));
    let pending = store.ok(&["submit", &neuron, &p, "7"]);
    let job = pending["invocation"].as_str().unwrap();
    let operation = pending["operation"].as_str().unwrap();
    assert_eq!(pending["status"], "awaiting-executor");
    let receipt = store.ok(&["take", &neuron, job]);
    let attempt = receipt["attempt"].as_str().unwrap();
    assert_eq!(receipt["neuron"], neuron);
    assert_eq!(receipt["prog"], p);
    assert_eq!(
        store.ok(&["run", &neuron, job])["status"],
        "unknown-outcome"
    );
    assert!(!store.call(&["take", &neuron, job]).0);
    assert!(!store.call(&["cancel", &neuron, job]).0);
    store.ok(&["pause", &neuron, &p]);
    assert_eq!(
        store.ok(&["outcome", &neuron, job, operation, attempt, "32"])["state"],
        "10"
    );
    store.ok(&["resume", &neuron, &p]);
    assert_eq!(store.ok(&["run", &neuron, job])["state"], "42");
    assert_eq!(
        store.ok(&["outcome", &neuron, job, operation, attempt, "32"])["state"],
        "42"
    );
    assert!(
        !store
            .call(&["outcome", &neuron, job, operation, attempt, "33"])
            .0
    );
}

#[test]
fn definite_failure_closes_attempt_and_retains_diagnostic() {
    let store = Store::new();
    let neuron = store.activate();
    let p = store.install(&neuron, "let x = host(event); 99", "10", Some("host"));
    let pending = store.ok(&["submit", &neuron, &p, "7"]);
    let job = pending["invocation"].as_str().unwrap();
    let operation = pending["operation"].as_str().unwrap();
    let taken = store.ok(&["take", &neuron, job]);
    let attempt = taken["attempt"].as_str().unwrap();
    store.ok(&[
        "fail",
        &neuron,
        job,
        operation,
        attempt,
        "executor rejected request",
    ]);
    let result = store.ok(&["run", &neuron, job]);
    assert_eq!(result["status"], "failed");
    assert_eq!(result["state"], "10");
    assert_eq!(
        result["invocations"][0]["fault"],
        "executor rejected request"
    );
    assert!(result["invocations"][0]["pending"].is_null());
    store.ok(&[
        "fail",
        &neuron,
        job,
        operation,
        attempt,
        "executor rejected request",
    ]);
    assert!(
        !store
            .call(&["outcome", &neuron, job, operation, attempt, "1"])
            .0
    );
    store.ok(&["retire", &neuron, &p]);
}

#[test]
fn keys_are_explicit_and_existing_files_are_never_overwritten() {
    let store = Store::new();
    let neuron = store.activate();
    let p = store.install(&neuron, "event", "0", None);
    let key = std::fs::read(store.0.join("owner.key")).unwrap();
    assert!(
        !store
            .call(&["keygen", store.0.join("owner.key").to_str().unwrap()])
            .0
    );
    assert_eq!(std::fs::read(store.0.join("owner.key")).unwrap(), key);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(store.0.join("owner.key"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    let read = Command::new(env!("CARGO_BIN_EXE_neuron"))
        .arg("--store")
        .arg(store.0.join("bbg"))
        .args(["inspect", &neuron])
        .output()
        .unwrap();
    assert!(read.status.success());
    let denied = Command::new(env!("CARGO_BIN_EXE_neuron"))
        .arg("--store")
        .arg(store.0.join("bbg"))
        .args(["submit", &neuron, &p, "7"])
        .output()
        .unwrap();
    assert!(!denied.status.success());
    assert_eq!(store.ok(&["inspect", &neuron, "--prog", &p])["state"], "0");
    assert_eq!(store.ok(&["identity"])["neuron"], neuron);
    assert!(
        !String::from_utf8_lossy(&read.stdout)
            .contains(&key.iter().map(|b| format!("{b:02x}")).collect::<String>())
    );
}

#[test]
fn legacy_import_is_available_through_the_real_cli_and_retains_provenance() {
    let store = Store::new();
    {
        let graph = neuron_node::Graph::open(store.0.join("bbg")).unwrap();
        legacy::replay(&graph, legacy::CAPTURE).unwrap();
    }
    let neuron = store.activate();
    let origin = legacy::named("counter")
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let mapping = format!("{origin}={}", "a1".repeat(32));
    let nonce = "a2".repeat(32);
    let imported = store.ok(&["import", &neuron, &mapping, "--nonce", &nonce]);
    assert_eq!(
        store.ok(&["import", &neuron, &mapping, "--nonce", &nonce]),
        imported
    );
    let prog = imported["mappings"][0]["prog"].as_str().unwrap();
    assert_eq!(
        store.ok(&["inspect", &neuron, "--prog", prog])["state"],
        "7"
    );
    assert_eq!(store.ok(&["legacy-inspect", &origin])["state"], "7");
    assert_eq!(
        store.ok(&["legacy-history", &origin])["provenance"],
        "legacy-cell-v1"
    );
    assert_eq!(store.ok(&["inspect", &neuron])["imports"], json!([origin]));
}

#[test]
fn separate_source_staging_is_resumable_through_process_commands() {
    let source = Store::new();
    {
        let graph = neuron_node::Graph::open(source.0.join("bbg")).unwrap();
        legacy::replay(&graph, legacy::CAPTURE).unwrap();
    }
    let target = Store::new();
    let neuron = target.activate();
    let nonce = "bc".repeat(32);
    let path = source.0.join("bbg");
    let mut result = Value::Null;
    for _ in 0..50 {
        result = target.ok(&[
            "stage-import",
            &neuron,
            "--source",
            path.to_str().unwrap(),
            "--nonce",
            &nonce,
            "--pages",
            "2",
        ]);
        if result["complete"] == true {
            break;
        }
    }
    assert_eq!(result["complete"], true);
    assert_eq!(
        target.ok(&[
            "stage-import",
            &neuron,
            "--source",
            path.to_str().unwrap(),
            "--nonce",
            &nonce
        ]),
        result
    );
    let origin = legacy::named("counter")
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert!(!source.call(&["legacy-inspect", &origin]).0);
    let mapping = format!("{origin}={}", "bd".repeat(32));
    let imported = target.ok(&["import", &neuron, &mapping, "--nonce", &"be".repeat(32)]);
    let prog = imported["mappings"][0]["prog"].as_str().unwrap();
    assert_eq!(
        target.ok(&["inspect", &neuron, "--prog", prog])["state"],
        "7"
    );
}

#[test]
fn declared_emit_is_rendered_from_a_durable_operation() {
    let store = Store::new();
    let neuron = store.activate();
    let p = store.install(&neuron, "let x = emit(event); 42", "0", Some("emit"));
    let output = store.ok(&["submit", &neuron, &p, "7"]);
    assert_eq!(output["state"], "42");
    assert_eq!(output["emitted"][0]["value"], "7");
}

#[test]
fn read_only_source_inspection_and_keyless_logical_export_preserve_the_full_legacy_archive() {
    let source = Store::new();
    std::fs::remove_file(source.0.join("owner.key")).unwrap();
    let source_path = source.0.join("bbg");
    {
        let graph = neuron_node::Graph::open(&source_path).unwrap();
        legacy::replay(&graph, legacy::CAPTURE).unwrap();
    }
    let target = Store::new();
    let target_path = target.0.join("bbg");
    let id = target.ok(&["identity"])["neuron"]
        .as_str()
        .unwrap()
        .to_owned();
    let key = std::fs::read(target.0.join("owner.key")).unwrap();
    let operator = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_neuron"))
            .current_dir(&source.0)
            .args(args)
            .output()
            .unwrap();
        let bytes = if out.status.success() {
            &out.stdout
        } else {
            &out.stderr
        };
        let result: Value = serde_json::from_slice(bytes)
            .unwrap_or_else(|_| panic!("{}", String::from_utf8_lossy(bytes)));
        (out.status.success(), result)
    };
    let args = [
        "legacy-source-inspect",
        source_path.to_str().unwrap(),
        "--destination",
        target_path.to_str().unwrap(),
    ];
    let (ok, before) = operator(&args);
    assert!(ok, "{before}");
    assert_eq!(before["validated"], true);
    assert_eq!(before["source_sealed"], false);
    assert_eq!(before["origins"].as_array().unwrap().len(), 3);
    assert!(before["rows"].as_u64().unwrap() > 3000);
    assert!(before["sizing"]["available_bytes"].as_u64().unwrap() > 0);
    assert_eq!(before["sizing"]["capacity_reserved"], false);
    assert!(!source.0.join("owner.key").exists());
    assert!(!target_path.exists());
    let (ok, again) = operator(&args);
    assert!(ok, "{again}");
    assert_eq!(again["digest"], before["digest"]);
    assert_eq!(again["last_transaction"], before["last_transaction"]);
    assert!(
        !operator(&[
            "legacy-source-inspect",
            source_path.to_str().unwrap(),
            "--max-rows",
            "1"
        ])
        .0
    );
    assert!(neuron_node::Graph::open(&source_path).is_ok());
    let nonce = "ef".repeat(32);
    let mut result = Value::Null;
    for _ in 0..50 {
        let (ok, next) = operator(&[
            "legacy-export",
            source_path.to_str().unwrap(),
            target_path.to_str().unwrap(),
            "--target",
            &id,
            "--nonce",
            &nonce,
            "--pages",
            "2",
        ]);
        assert!(ok, "{next}");
        result = next;
        if result["complete"] == true {
            break;
        }
    }
    assert_eq!(result["complete"], true);
    assert_eq!(result["rows"], before["rows"]);
    assert_eq!(result["logical_bytes"], before["logical_bytes"]);
    assert_eq!(result["dispatch"], false);
    assert!(neuron_node::Graph::open(&source_path).is_err());
    let (ok, sealed) = operator(&args);
    assert!(ok, "{sealed}");
    assert_eq!(sealed["source_sealed"], true);
    assert_eq!(sealed["seal"]["nonce"], nonce);
    assert_eq!(sealed["seal"]["target"], id);
    assert_eq!(sealed["digest"], before["digest"]);
    assert_eq!(sealed["origins"], before["origins"]);
    assert!(
        !operator(&[
            "legacy-export",
            source_path.to_str().unwrap(),
            target_path.to_str().unwrap(),
            "--target",
            &id,
            "--nonce",
            &"aa".repeat(32)
        ])
        .0
    );
    assert!(!source.0.join("owner.key").exists());
    assert_eq!(std::fs::read(target.0.join("owner.key")).unwrap(), key);
    // Export created no active subject. Explicit authenticated activation/import
    // recovers all three original programs, including the unknown tool attempt.
    assert!(!target.call(&["inspect", &id]).0);
    assert_eq!(target.activate(), id);
    let mappings = ["counter", "tool", "reserved"]
        .iter()
        .enumerate()
        .map(|(i, name)| {
            format!(
                "{}={}",
                legacy::named(name)
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>(),
                format!("{:02x}", i + 30).repeat(32)
            )
        })
        .collect::<Vec<_>>();
    let imported = target.ok(&[
        "import",
        &id,
        &mappings[0],
        &mappings[1],
        &mappings[2],
        "--nonce",
        &"fa".repeat(32),
    ]);
    assert_eq!(imported["mappings"].as_array().unwrap().len(), 3);
    let counter = imported["mappings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["origin"] == mappings[0][..64])
        .unwrap();
    assert_eq!(
        target.ok(&["inspect", &id, "--prog", counter["prog"].as_str().unwrap()])["state"],
        "7"
    );
    let (ok, after) = operator(&args);
    assert!(ok, "{after}");
    assert_eq!(after["digest"], before["digest"]);
}

#[test]
fn archive_operator_rejects_invalid_destination_and_busy_source_before_sealing() {
    let source = Store::new();
    let path = source.0.join("bbg");
    let target = Store::new();
    let id = target.ok(&["identity"])["neuron"]
        .as_str()
        .unwrap()
        .to_owned();
    let nonce = "51".repeat(32);
    let held = neuron_node::Graph::open(&path).unwrap();
    legacy::replay(&held, legacy::CAPTURE).unwrap();
    assert!(
        !source
            .call(&["legacy-source-inspect", path.to_str().unwrap()])
            .0
    );
    drop(held);
    let initial = source.ok(&["legacy-source-inspect", path.to_str().unwrap()]);
    let file = target.0.join("existing-file");
    std::fs::write(&file, b"preserve me").unwrap();
    assert!(
        !source
            .call(&[
                "legacy-export",
                path.to_str().unwrap(),
                file.to_str().unwrap(),
                "--target",
                &id,
                "--nonce",
                &nonce
            ])
            .0
    );
    assert_eq!(std::fs::read(file).unwrap(), b"preserve me");
    let after = source.ok(&["legacy-source-inspect", path.to_str().unwrap()]);
    assert_eq!(after["source_sealed"], false);
    assert_eq!(after["last_transaction"], initial["last_transaction"]);
    assert!(
        !source
            .call(&[
                "legacy-export",
                path.to_str().unwrap(),
                path.to_str().unwrap(),
                "--target",
                &id,
                "--nonce",
                &nonce
            ])
            .0
    );
    assert_eq!(
        source.ok(&["legacy-source-inspect", path.to_str().unwrap()])["last_transaction"],
        initial["last_transaction"]
    );
    let empty = target.0.join("empty");
    std::fs::create_dir(&empty).unwrap();
    assert!(
        !source
            .call(&["legacy-source-inspect", empty.to_str().unwrap()])
            .0
    );
    assert_eq!(std::fs::read_dir(empty).unwrap().count(), 0);
}
