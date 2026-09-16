---
title: local implementation status
date: 2026-09-11
status: archived original cell implementation report
---
# local implementation

Historical evidence from 2026-09-11, before neuron convergence. Current behavior
and checks are recorded in [the implementation ledger](../../soft3/audit/neuron-cell/implementation.md).
Old API/package names below identify that original revision.

Cell 0.1.0 implements the first local integration slice of design 0.2. It starts
rune source immediately, persists state and deep continuations, and coordinates
effects through graph receipts. It is usable from the CLI and the same Rust
engine. Full runtime baseline conformance, cyb extraction and the soma agent
composition remain work ahead. No Hermes parity or superiority result is claimed.

## component boundaries

| Component | Implemented responsibility |
|---|---|
| [cell-model](../model/src/lib.rs) | Canonical Model B values, versioned manifest suite, birth/snapshot/operation records, bounded readers |
| [cell-engine](../engine/src/lib.rs) | Instance admission, lifecycle, checkpoints, reservations, outbox/attempt/outcome, explicit GraphPort/RuntimePort/WardPort |
| [cell-rune](../rune/src/lib.rs) | Mounted mem/here, lexical event, bounded parse/lower/machine adapter, noun artifacts |
| [cell-node](../node/src/lib.rs) | Cybergraph adapter and private-owner authorization adapter |
| [cell CLI](../cli/src/main.rs) | Create, submit, run, inspect, history, take, outcome, fail, pause, resume, cancel, retire |
| [cybergraph applications](../../cybergraph/src/application.rs) | Content verification/closure, application transactions, selected heads, receipts and history reads |
| [BBG application storage](../../bbg/rs/src/storage/application.rs) | Shared BBG database transactions for bytes, head, history, request and global claims; Fjall by default |
| [rune machine](../../rune/rs/interp/machine.rs) | Explicit reduction stack, nested suspension, bounded checkpoints and restore |

No Bevy dependency exists in model or engine. The CLI history command is a
projection of graph history. There is no second authoritative cell log.
New crates use sibling path dependencies; dependencies are not yet published.

Local upstream commits required by this slice:

| Repository | Commit | Branch |
|---|---|---|
| bbg | c731055 | feat/atomic-application-storage |
| cybergraph | 89eb69c | feat/durable-applications |
| rune | cc186f3 | feat/resumable-authority-bound-runtime |

These commits remain local. Cargo.lock records the resolved dependency versions;
the table pins the implementation changes that path dependency versions alone
cannot identify. Existing unrelated workspace changes are outside these commits.

The table records the initial 2026-09-11 integration. The subsequent shared BBG
Database migration, feature selection and validation are recorded in the
[storage migration audit](../audit/shared-bbg-database.md). The CLI now selects
a BBG directory and offers optional explicit import from legacy redb files.

## working behavior

An installed definition is source plus policy, resource contract and runtime
identity. A birth nonce creates an independent instance even for identical code.
Activation follows the birth as a separate retained transition. Application
state changes only after successful completion. Failed invocations keep prior
state; their diagnosis is retained and exposed at the fault head by inspect.

An event pins input, origin, nonce, destination, authority and optional context.
The same request returns its original receipt. Altering input/context/destination
at that origin+nonce conflicts across the local database, including after task
completion. New inputs while an invocation is live return Busy. History uses
bounded ordered range reads; transitions do not rescan completed history.

Every runtime slice reserves its maximum exposure before computation. Successful
settlement charges actual steps. If settlement is interrupted, recovery charges
the reserved slice before rerunning its preceding checkpoint. A lost commit
reply is explicit CommitUnknown; replay of the same request resolves its receipt.

Every tool request retains its operation and continuation before authorization.
The engine defaults to deny; LocalWard implements a stored private-owner allow
list. An attempt retains the selected operation, policy and epoch before the
executor is handed arguments. If its reply is absent after restart it becomes
unknown. `take` cannot create another attempt, and cancellation cannot silently
erase it. A correlated success or definite failure settles that attempt. Repeated
outcomes return their original receipt; changed terminal outcomes conflict.
Sequential effects retain separate operation identities and consumer records.

Pause preserves live work and accepts outcome recording. Resume uses the saved
checkpoint. Retirement requires no live invocation; an attempted unknown effect
must be reconciled before it can be retired. A parked cell holds graph data,
with no resident interpreter required.

## limitations and next components

| Next gate | Owner and concrete work |
|---|---|
| Runtime baseline resources | cell/body: complete metric manifests, aggregate read/write budgets, deadlines, physical placement and process memory limits |
| Public graph publication | cybergraph/bbg: atomic chain/state publication and coordinated per-neuron writer; the new private application API does not replace this |
| Real ward integration | ward/node: immutable grants, scope/disclosure, delegation, revocation, current epoch/fencing; LocalWard is a bootstrap adapter |
| Cyb convergence | cell-prysm/cyb: mount real definitions, views and events through this engine; migrate existing core/shell cell implementations |
| Durable reactive channels | cybergraph/cell: snapshot-and-follow, cursor persistence, backpressure, cancellation and deadlines |
| Evolution | cell: backup closure, migration, upgrade/fork and fenced relocation with pending obligations |
| Agent composition | soma: now/soul application bindings, durable tasks/children/joins, model/tool adapters, learning and workspace conflict handling |
| User delivery | plan/sense: schedule occurrences and delivery identities independently of task execution |
| Shared resources and secrets | body/sigma/vault: child reservations, provider cost accounting, secret handles and effect adapters |
| Comparative release | soma/cell: run all A-series capability/evaluation gates against the pinned Hermes baseline |

There is one unresolved act per instance and one live invocation. The only
automatic executor is graph-retained emit; host/query/link/seal/subscribe need
concrete external adapters. Reactive Event suspension is rejected by cell's local
adapter. There is no model provider, shell/browser integration, agent task loop,
remote authentication/transport, protocol finality or execution proof here.

The [local profile](../specs/local-runtime.md) lists exact bounds. Compute steps
are retained, but the complete baseline ResourceContract is not implemented.
Logical node limits do not claim body-enforced RSS. The current rune word/hash
semantics are pinned as a host-observed ABI, not Nox proof-compatible execution.
The private owner trusts its own database/executor reports. A remote writer or
reader requires an authentication/disclosure adapter.

## initial verification (2026-09-11)

The [local conformance map](legacy-conformance-local.md) lists all C01–C58 gates.
Executable suites cover:

- [Model data](../model/tests/data.rs): full-width uint, canonical particle,
  malformed lengths, schema rejection, bounded reads and forged content.
- [Runtime integration](https://github.com/cyberia-to/neuron/blob/34bbff9dbf36421a45c5c283e6c1336dcafc997a/node/tests/runtime.rs): isolated births/state,
  repeat/conflicting input including changed destination, lifecycle, authority
  denial, context and unknown attempt recovery.
- [Recovery](../node/tests/recovery.rs): failure before settlement, loss of an
  admitted receipt, reservation recovery, sequential acts and consumed usage.
- [CLI processes](../cli/tests/process.rs): a new OS process for every command,
  durable counter/history, paused results, unknown outcome, denial and failure.
- [BBG](../../bbg/rs/tests/application_storage.rs): reopen, rollback, conditional
  heads, concurrent writers, unique claims and bounded reads.
- [Cybergraph](../../cybergraph/tests/applications.rs): content closure, validator
  rejection, required references, fingerprints and content corruption detection.
- Rune's complete workspace tests, including [authority](../../rune/rs/interp/tests/authority.rs),
  [checkpoints](../../rune/rs/interp/tests/checkpoints.rs) and
  [language integration](../../rune/rs/interp/tests/integration.rs).

Cell's workspace tests and strict Clippy pass. The targeted rune library Clippy
check passes. Strict whole-library Clippy in BBG/cybergraph encounters pre-existing
warnings in prune/shard backends and the public API respectively; the new
application modules have no reported warnings. These warnings remain visible.
The full disk-barrier/power-loss matrix was not executed. Fault injection
in cell tests targeted the graph port, and the initial content corruption test
used redb. Current backend verification is recorded in the storage migration
audit linked above.

Reproduce from each repository:

```text
cell:       cargo test --workspace
cell:       cargo clippy --workspace --all-targets --no-deps -- -D warnings
rune:       cargo test --workspace
rune:       cargo clippy -p rune-parse -p rune-lower -p rune-interp --lib --no-deps
bbg:        cargo test --manifest-path rs/Cargo.toml --features backend-hdd --test application_storage
cybergraph: cargo test --features local-storage --test applications
```

Use package-scoped cargo fmt or rustfmt on changed files. Cargo fmt --all in this
sibling workspace also formats path dependencies, so it is not a scoped check.
