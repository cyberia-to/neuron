---
title: integration
tags: cell, soft3, spec
status: draft
spec-version: "0.1"
---
# integration and extraction

## repository boundaries

| Area | Responsibility |
|---|---|
| cell model | Identities, schemas, invariants, lifecycle and transition vocabulary |
| cell engine | Instance admission/scheduling/recovery and coordination through ports |
| cell node | Local hosting/replica composition and concrete stack adapters |
| cell rune | Rune definition/subject/checkpoint mapping into RuntimePort |
| cell prysm | View binding and input event conversion |
| cell CLI | Headless composition exercising the production contracts |

Separate Rust crates follow these dependency boundaries when implementation
begins. Model has no Bevy/application dependency. Engine consumes ports.
Node/adapters compose existing stack crates. Published and workspace consumers
use one extracted implementation.

## ownership

| Owner | Mechanism |
|---|---|
| cybergraph | Authoritative graph history, validation/write/query interface, application record publication and conditional head updates |
| bbg/storage | Durable records/content, authenticated state, transactions and storage barriers |
| hemera/lens | Content identity and commitments |
| foculus | Signal ordering, reconciliation, conflict resolution and finality |
| tape/radio | Framing / transport |
| rune/nox/wysm/glia | Evaluation and runtime-specific execution |
| zheng | Execution proof generation and verification |
| ward/vault | Authorization and effect routing / secret operations |
| body | Machine resources, devices and process supervision |
| name/state | Name resolution / verified external-state perception |
| soma | Agent cognition, task strategy and learning |
| plan/sense | Future schedules / conversations and delivery |
| cyb log | History presentation, filtering, search and navigation |
| cyb memory/brain/time | Filesystem, spatial and temporal graph presentations |
| cyb | Application assembly, native window/chrome and interaction |

The word log may also occur in storage's write-ahead log or signal log; those
refer to physical/protocol data structures. They do not create a new organ owner.

## required upstream work

| Boundary | Current source evidence | Required contract |
|---|---|---|
| cybergraph → bbg | Chain append precedes fallible state insertion | Atomic validation/application and head comparison |
| graph → storage | Cybergraph::new holds memory; cyb_core::Cell appends tape | Durable graph session with explicit transaction outcome |
| bbg → disk | ShardStore::commit lacks Result; FjallStore discards errors | Fallible atomic persistence and recovery barriers |
| graph → content | Current cell tape carries references; content also lives in sidecars | Content closure retained with acknowledged commits |
| runtime → ward | Rune reads caps from mutable absolute subject axes | Host-bound context across nested/sequential/reactive execution |
| rune → cell | run_cell exists; deep suspension/witness plumbing is partial | Versioned resumable checkpoints and durable act boundary |
| graph → cell | Existing graph types expose generic signals, not cell records | Opaque application record validation/projection and conditional heads |
| signal → replica | Duplicate/equivocation paths are conflated in cyb wrappers | Exact-content dedup, explicit gap/conflict handling |
| graph → subscription | Callback subscription lacks durable cell cursor contract | Snapshot-and-follow and recoverable cursors |
| data → wire | Stack schemas/codecs are evolving | Pinned structural identity/codec vectors, dialect registration and bounded decoding |
| act → placement | Epoch enforcement is not wired as a uniform boundary | Ward/executor fencing and current-grant checks |
| profile → proof | Host witnesses and verification routes are partial | Explicit supported claim/finality sets and admission rejection for gaps |

These are implementation requirements in the owning repositories. A cell release
cannot advertise their guarantees merely because an adapter trait exists.

## migration sources

- cyb/core/src/cell.rs and cyb/crates/cyb/src/cell.rs converge into one host/replica
  implementation. Preserve supported consumer behavior through adapters.
- cyb's signal construction delegates to the shared writer coordinator and
  existing graph protocol types; independently produced step counters disappear.
- cyb/shell/src/worlds/robot/mod.rs supplies the page-loading/presentation seam.
  Connect actual loaded definitions to the visible path through the adapter.
- rune/rs/interp/event.rs supplies the existing reactive shape. Generic evaluator
  and continuation work remains in rune; lifecycle mapping belongs to cell.
- cyb's graph.log and particles.jsonl become migration inputs to durable graph
  sessions. Import validates framing, content identity, ordering and completeness.

Import is repeatable by original record identity and reports rejected/conflicting
records and missing bytes. It never treats missing history as an empty success.
Migration retains original data until the imported state/head and required
content are verified. Historical imports lack new guarantees unless the evidence
exists; an old unsigned or unproven record remains labelled accordingly.

## cell profiles and the 21 organs

A repository owns implementation, an organ names a responsibility, and a cell
instance owns state/lifetime/authority. Their counts are independent.
Existing native components can expose cell-compatible entrypoints while their
foundational libraries remain statically linked.

Soma tasks run inside an owning runtime-cell. A new cell is appropriate for an
independent state/authority/deployment boundary, rather than every inference
step. Instructional skills remain particles interpreted by soma; executable
extensions can provide cell definitions.

Cyb's current anatomy governs soul/configuration, avatar/visualization,
now/context and log/history presentation when adapting older soma documents.
The specification introduces no separate global configuration or memory owner.

## acceptance boundaries

Runtime-cell and local host support are the first conformance target.
Remote building support adds authenticated delivery and the declared service
finality contract. Ledger/knowledge support adds their exact protocol suites.
Every advertised combination of runtime, profile, storage and transport must
state its supported schemas, evidence, durability and limits.
