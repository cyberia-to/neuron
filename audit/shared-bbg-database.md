---
title: shared BBG database and legacy CLI migration
tags: cell, cybergraph, bbg, storage, audit
date: 2026-09-12
status: consumer-validated
---

# Shared BBG database and legacy CLI migration

Cell's Graph adapter now accepts an existing BBG Database and re-exports its
Database/Backend selection types through Cybergraph. Ordinary opens use a
Fjall directory. Cell retains GraphPort as its storage boundary and introduces
no independent database ownership or raw application mutation API.

The CLI defaults to the bbg directory. If cell.redb exists, an invocation
without --store stops before opening a graph. Explicit --store selection
continues to work. The optional legacy-redb-migration build feature exposes
migrate-redb SOURCE DESTINATION, forwarding through Cybergraph to BBG before
any normal graph open. Default builds do not enable redb.

Migration preserves the source and requires a fresh destination. Incomplete
imports cannot be opened as a completed session. BBG owns that enforcement
and the legacy data format. The Cell CLI forwards the outcome as JSON success
or an error; successful migration does not imply network finality.

## Validation

Validation ran on macOS arm64 against the actual workspace and sibling paths.
The initial dependency check exposed Foculus's stale BBG requirement. Its
requirements and Cybergraph's direct path requirements were aligned with the
actual sibling versions before testing; Cargo.lock records that resolution.

The default workspace passed 15 tests: four CLI process tests, three CLI storage
tests, two model tests, three node recovery tests and three runtime tests.
With legacy-redb-migration enabled, the selected node and CLI packages passed
14 tests: the same four process, three recovery and three runtime tests plus
four storage tests including migration failure routing. The default production
dependency tree contains vendored Fjall 2.11.2 and no redb dependency.

Strict default workspace Clippy and strict migration-enabled node/CLI Clippy,
scoped rustfmt and git diff --check passed.
Three existing Fjall compiler warnings remain; no Cell warning was reported.

The existing node runtime/recovery and CLI process fixtures now use BBG
directories. New CLI storage tests cover the default path, prevention of an
implicit empty session beside cell.redb, explicit store selection, preservation
of wrong-format files, and optional migration failure before normal graph open.
BBG tests own valid legacy-file import and interrupted import recovery.

Executed commands from this repository:

```sh
cargo test --workspace --offline
cargo test -p cell-node -p cell-cli --offline --locked --features legacy-redb-migration
cargo clippy --workspace --all-targets --offline --locked --no-deps -- -D warnings
cargo clippy -p cell-node -p cell-cli --all-targets --offline --locked --features legacy-redb-migration --no-deps -- -D warnings
cargo tree -p cell-cli --offline --locked --edges normal --prefix none
```

The original Cell graph-port fault tests remain tests of caller recovery.
They do not simulate physical power loss. The
[Cybergraph audit](../../cybergraph/audit/shared-bbg-database.md) records its
content and shared-owner checks; native node acceptance remains separate.
