use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "cell-storage-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn command(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_cell"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn default_store_is_a_bbg_directory() {
    let dir = Directory::new();
    let output = dir.command(&["history", &"00".repeat(32)]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["history"], serde_json::json!([]));
    assert!(dir.0.join("bbg").is_dir());
    assert!(!dir.0.join("cell.redb").exists());
}

#[test]
fn legacy_default_requires_explicit_store_selection() {
    let dir = Directory::new();
    let legacy = dir.0.join("cell.redb");
    std::fs::write(&legacy, b"legacy sentinel").unwrap();
    let cell = "00".repeat(32);
    let output = dir.command(&["history", &cell]);
    assert!(!output.status.success());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert!(
        error["error"]
            .as_str()
            .unwrap()
            .contains("legacy cell.redb exists")
    );
    assert!(!dir.0.join("bbg").exists());

    let output = dir.command(&["--store", "selected", "history", &cell]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.0.join("selected").is_dir());
    assert_eq!(std::fs::read(&legacy).unwrap(), b"legacy sentinel");
}

#[test]
fn normal_open_refuses_an_existing_file_without_replacing_it() {
    let dir = Directory::new();
    let legacy = dir.0.join("old.redb");
    std::fs::write(&legacy, b"legacy sentinel").unwrap();
    let output = dir.command(&["--store", "old.redb", "history", &"00".repeat(32)]);
    assert!(!output.status.success());
    assert_eq!(std::fs::read(&legacy).unwrap(), b"legacy sentinel");
    assert!(!dir.0.join("bbg").exists());
}

#[cfg(feature = "legacy-redb-migration")]
#[test]
fn migration_failure_is_reported_before_normal_graph_open() {
    let dir = Directory::new();
    std::fs::write(dir.0.join("cell.redb"), b"invalid redb source").unwrap();
    let output = dir.command(&[
        "--store",
        "unused",
        "migrate-redb",
        "cell.redb",
        "destination",
    ]);
    assert!(!output.status.success());
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert!(
        !error["error"]
            .as_str()
            .unwrap()
            .contains("legacy cell.redb exists")
    );
    assert!(!dir.0.join("unused").exists());
    assert!(!dir.0.join("bbg").exists());
    assert_eq!(
        std::fs::read(dir.0.join("cell.redb")).unwrap(),
        b"invalid redb source"
    );
}
