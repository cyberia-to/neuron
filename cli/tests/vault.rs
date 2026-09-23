#[path = "../../node/tests/support/vault.rs"]
mod fixture;
use serde_json::Value;
use std::{
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

fn call(directory: &Path, args: &[&str], password: &str) -> Output {
    let mut process = Command::new(env!("CARGO_BIN_EXE_neuron"))
        .arg("--store")
        .arg(directory.join("bbg"))
        .arg("--vault-home")
        .arg(directory.join("vault"))
        .args([
            "--vault-root",
            fixture::ROOT,
            "--vault-domain",
            fixture::DOMAIN,
            "--vault-secrets-stdin",
        ])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(process.stdin.take().unwrap(), "{password}").unwrap();
    process.wait_with_output().unwrap()
}
fn ok(directory: &Path, args: &[&str]) -> Value {
    let out = call(directory, args, fixture::PASSWORD);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!String::from_utf8_lossy(&out.stdout).contains(fixture::PASSWORD));
    assert!(!String::from_utf8_lossy(&out.stderr).contains(fixture::PASSWORD));
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn vault_backed_neuron_runs_across_processes_without_a_key_file() {
    let dir = tempfile::tempdir().unwrap();
    fixture::create(dir.path());
    let active = ok(dir.path(), &["activate"]);
    let neuron = active["neuron"].as_str().unwrap();
    let source = dir.path().join("counter.rune");
    std::fs::write(&source, "~mem + event").unwrap();
    let installed = ok(
        dir.path(),
        &[
            "install",
            neuron,
            source.to_str().unwrap(),
            "--initial",
            "0",
        ],
    );
    let prog = installed["prog"].as_str().unwrap();
    let nonce = "06".repeat(32);
    let args = ["submit", neuron, prog, "7", "--nonce", &nonce];
    let first = ok(dir.path(), &args);
    assert_eq!(first["state"], "7");
    let retry = ok(dir.path(), &args);
    assert_eq!(first["admission_commit"], retry["admission_commit"]);
    assert_eq!(retry["state"], "7");
    let bad = call(dir.path(), &["identity"], "wrong-synthetic-password");
    assert!(!bad.status.success());
    assert!(bad.stdout.is_empty());
    let both = call(
        dir.path(),
        &["--key-file", "nonexistent.key", "identity"],
        fixture::PASSWORD,
    );
    assert!(!both.status.success());
    assert!(both.stdout.is_empty());
    assert!(!dir.path().join("owner.key").exists());
}
