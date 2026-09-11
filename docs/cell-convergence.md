---
title: cell convergence
tags: cyb, cell, soft3, architecture
status: draft
date: 2026-09-11
---
# cell convergence

Explanation of the standalone `cell` repository. This document records the
unification and source findings. The complete candidate contracts are in
[specs](../specs/README.md); implementation follows their review.

## established direction

The user established two premises on 2026-09-11:

1. The definitions in [cyb anatomy](../../cyb/anatomy.md) govern the robot's organs.
   Other repositories, including soma, adapt their vocabulary and ownership
   to those definitions with explicit attention to meaning.
2. Cell is a convergence task. The runtime organ, local node, and protocol
   concepts should be examined together for a single repository.

The user accepted the convergence direction and requested the full model in
`cell/specs`. Those contracts remain draft for review. This explanation defers
to them. The history ownership correction is explained in
[history ownership](history-ownership.md).

## what the sources already say

The protocol's [cell page](../../cyber/cell.md) was changed on 2026-09-08
to a four-rung ladder: runtime, building, ledger, knowledge. The same day's
addition to [cyb cells](../../cyb/parts/cells.md) explicitly places the robot's organ
on that ladder. The anatomy's warning about a distinct protocol cell still
reflects the older separation. The proposed alignment preserves anatomy's
definition of the organ and explains its relationship to the larger family.

| Source | Observed meaning or implementation | Consequence |
|---|---|---|
| [cyb anatomy](../../cyb/anatomy.md) | Cell grows a robot's abilities through live-loaded programs | Extensibility is the primary product role inside cyb |
| [cyb cells](../../cyb/parts/cells.md) | A rune page, loading and rendering milestones, then the cell ladder | A page is the first runtime profile |
| [cyber cell](../../cyber/cell.md) | Bounded state, boundary, lifecycle, address, finality | Provides the common semantic direction |
| [soft3 stack completeness](../../soft3/roadmap/stack-completeness.md) | Explicit cell/cyb-core convergence and a missing common runtime contract | Extraction should close these seams |
| [cyb core Cell](../../cyb/core/src/cell.rs) | A Cybergraph, signal construction, tape persistence, replay and replica ingestion | Contains reusable local hosting mechanisms |
| [published cyb Cell](../../cyb/crates/cyb/src/cell.rs) | A second implementation in the separately packaged cyb crate | Both consumers must converge on one implementation |
| [rune event driver](../../rune/rs/interp/event.rs) | `run_cell` threads state through successive events | An existing bridge from programs to stateful instances |
| [robot world](../../cyb/shell/src/worlds/robot/mod.rs) | Loader/evaluator functions and a native Bevy page | The current `setup_cell` draws directly; it does not call those loader/evaluator functions |
| [3c](../../cyber/3c.md) | Cross-cell read, write and trade, independent of transport | Supplies protocol semantics; the concrete shared envelope still needs a contract |

Reviewed source revisions: cyb `48fd4841854973bb0dd381e4455b3840bb674de9`,
cyber `11753dd21644aa77f1415650b84f21c2966f6e9b`,
rune `67281db727070c1160c12be341c6fd85e39e5380`,
soft3 `77c780cef2ce07bff29855278a465ff070a91102`.

## proposed common identity

A cell is an addressable owner of bounded state, governed by explicit rules for
admitting inputs, changing that state, producing acts and establishing which
version is authoritative. A runtime-cell supplies those rules through a loaded
program and grows an ability of the robot.

Four distinctions make this useful in code:

| Entity | Identity and lifetime |
|---|---|
| Definition | Immutable release particle identifying code, state schema, entrypoints and required artifacts; publication records identify its author |
| Instance | Stable cell identity with its own history, governance and active definition; an upgrade preserves this identity |
| Replica | A particular host's copy of an instance at an explicit version and root |
| Host | A running environment that places instances, maintains replicas and connects execution, storage, authorization and synchronization |

A name resolves to an instance or a release according to an explicit address
kind. A content particle identifies a particular immutable value. An endpoint
locates a host. These addresses have separate lifecycles.

One definition can create many instances. One instance can have many replicas.
One host can hold many instances. Moving an instance between hosts preserves its
identity; creating an independent copy gives the copy a new identity and lineage.

The existing `cyb_core::Cell` combines host/replica functions. In particular, it
holds signals authored by several neurons. Holding those signals grants no
authority to change the original cells' state. Extraction should expose the
distinction between locally governed instances and observed foreign replicas.

Ownership concerns mutable state and admission rules. Immutable particles may
be shared, deduplicated and referenced by many cells. A parent can manage a child
through capabilities and lifecycle requests; physical containment alone grants
no write authority.

## the ladder as profiles of one model

| Profile | State and behavior | Authority and finality |
|---|---|---|
| Runtime organ | Program state, subscriptions, pending work, optional prysm surface | A neuron's delegated authority; durable local ordering initially |
| Building | Shared service state and several clients' views | Service governance and an explicit admission/finality policy |
| Ledger | The ledger and transition rules defined by its economic protocol | Issuer/governance rules plus the required consensus and evidence |
| Knowledge | An owned cybergraph region and authenticated derived state | Validator governance and foculus finality |

The profiles share lifecycle, identity, transition records, receipts, recovery
and synchronization interfaces. Their transition rules and required evidence
remain explicit. A service deployment chooses its finality contract; moving the
same code from a laptop to a server does not silently change that contract.

The ledger profile can accommodate [oikos](../../cyber/research/oikos.md)
when that proposal's protocols are settled. Knowledge-cell division remains
[research](../../cyber/research/spectral%20cell%20division.md). Neither the
atomic-trade protocol nor spectral division is a prerequisite for the runtime
organ. Their unresolved semantics remain visible in the extension points.

## the transition contract

The common semantic shape is:

```text
definition + prior state + admitted input + recorded witnesses
    → proposed next state + requested acts + evidence
```

Runtime adapters also need operational outcomes: completed, waiting for an act
result, waiting for an event, yielded at a resource limit, or failed. Durable
suspension requires an explicit checkpoint or continuation format understood by
that runtime version. Rune's existing shallow hint parking is only part of this.

Cell owns instance lifecycle and the coordination of a transition. The runtime
evaluates the program. Ward decides which acts may happen and invokes registered
executors. Cybergraph records the interaction history through its durable
storage boundary. Recorded results become inputs for continuation and replay.

An external act needs a stable operation identity and a recoverable progression:

```text
input accepted → next state and pending act committed
              → ward authorization → execution
              → outcome recorded → continuation
```

The commit of next state and pending acts needs one atomic recovery boundary.
Publication and acknowledgement follow the promised durability level. Pure
validation precedes mutation; replay applies recorded decisions and witnesses.
Rendering the current view can be repeated from state, while delivery and other
consequential acts require their recorded operation identity.

If an external system completed an act but its reply was lost, recovery must
represent the result as unknown, query the external operation when possible,
and apply that adapter's retry policy. An execution proof alone cannot resolve
this interval. Remote idempotency support is an explicit adapter capability.

Input identifiers, state versions, code releases and operation identifiers must
be bound together in receipts. An exact duplicate is distinct from a conflicting
proposal for the same position. Foculus owns conflict resolution where consensus
is required; cell surfaces that result to its lifecycle and recovery logic.

## three kinds of history

The following histories describe different facts:

1. A cell's interaction history: admitted events, transitions, pending acts,
   outcomes, activation and retirement.
2. Protocol signal chains: authored cybergraph changes, their ordering and
   consensus treatment.
3. A replica's progress: which remote records and checkpoints it has verified
   and retained.

Cybergraph owns the authoritative history/write interface. BBG and its storage
backends implement durable representation and persistence barriers. Cell defines
its transition records and coordinates recovery through that interface. Tape
owns framing. The cyb log organ presents history, including filtering, search
and navigation; it consumes the graph's cursor/checkpoint query interfaces.

An interaction can refer to a protocol signal; these records share an explicit
correlation rather than becoming independent authorities for the same fact.
Local instances can share physical storage while retaining separate logical
streams, access scopes and replication policies. A private task history is
published only according to its export policy.

The graph's storage implementation remains usable before cells are loaded.
Log's presentation can use cell interfaces. The same bootstrap distinction
applies to ward and vault: foundational services are bound before extensions
request their storage or authority.

## authority, evidence and communication

An execution receives a host-bound authority context identifying the instance,
code release, granted rights and revocation state. The program can inspect a
projection of those rights and request attenuation. Ward checks the request
against the host-held grant; rewriting a subject value cannot create authority.

A local call and a remote call use the same destination, operation, payload and
receipt semantics. Local dispatch can use an in-process path; radio provides
remote transport. Authentication, authorization, input ordering and duplicate
handling apply on both paths.

The 3c verbs describe crossing a cell boundary. A write exports evidence or a
request that the receiving cell admits under its own rules. A read supplies an
answer with provenance and freshness. Trade adds a separate economic protocol.
UI intents, tool requests and graph Signals still need explicit encodings and
correlation; their current Rust types are not interchangeable.

Evidence should identify the claim it supports: content integrity, authorship,
state inclusion, transition execution, conditional host results, or consensus
finality. It also binds the relevant definition, state version and trust anchor.
A foreign root appearing in the graph is an anchor with specific provenance;
it does not establish every claim about that foreign state.

The [state organ's T0/T1/T3 contract](../../cyb/parts/state.md) continues to govern
external-state answers. A local disk acknowledgement or model output must not
acquire a stronger tier merely by passing through cell. Conditional computation
records its assumptions and external observations explicitly.

## one repository, explicit internal boundaries

Repository: `~/cyber/cell`. The following are dependency boundaries;
crate splits should follow real dependency and consumer needs.

| Module / possible crate | Owns |
|---|---|
| `model` / `cell-model` | Instance/release/replica identities, transition and receipt semantics, narrow ports used by the engine |
| `engine` / `cell-engine` | Loading, activation, event admission, transition coordination, suspension, recovery, upgrade and retirement |
| `node` / `cell-node` | Local placement of owned instances and observed replicas; adapters to cybergraph, foculus, radio and durable storage |
| `rune` / `cell-rune` | Definition loading and execution through rune, subject mapping, resumable acts and versioned checkpoints |
| `prysm` / `cell-prysm` | Cell presentation and input bindings using prysm; platform bindings remain optional |
| `cli` | Headless composition for exercising the same engine and node used by cyb |

The model has no application or renderer dependency. The engine consumes narrow
ports; concrete integrations implement them. Rune owns its language and evaluator,
including generic continuation machinery. The cell adapter owns how a loaded
definition becomes an instance. Additional runtimes enter through that seam.

Dependency ownership stays explicit:

| Component | Remains its responsibility |
|---|---|
| cybergraph | Validating and applying cybergraph writes, query and subscription contracts |
| bbg, hemera, lens | State representation, storage primitives, content identity and commitments |
| foculus | Signal ordering, reconciliation, conflict resolution and consensus finality |
| tape, radio | Framing and transport respectively |
| rune, nox, wysm, glia | Language/evaluation and runtime-specific computation |
| zheng | Execution proofs and their verification |
| ward, vault | Authorization/execution routing and secret operations respectively |
| log | History presentation, filtering, search and navigation over cybergraph |
| body | Physical placement resources, devices and process supervision |
| name, state | Name resolution and verified external-state access |
| soma | Meaning and execution strategy of agent tasks |
| cyb | Product composition, native window/chrome and interaction |

The repository unifies cell mechanisms across scales. The existing stack
repositories supply their own mechanisms as dependencies.

## what this means for the 21 organs and soma

An organ is an anatomical responsibility; a cell is a stateful instance with a
boundary and lifecycle; a repository is an implementation ownership boundary.
Their cardinalities differ. One organ can expose several cell entrypoints, and
one task can call several organs.

The cell organ supplies the robot's extension facility. The same repository
also supplies the reusable hosting and protocol machinery below that facility.
Native implementations of the other organs can expose cell-compatible ports
without becoming dynamically loaded Rust libraries. Live-loaded rune definitions
grow behavior through the host interfaces present in the binary.

Soma benefits from durable environments for an integration, workspace or an
independently authorized worker. A new task starts inside its owning cell;
creating a new cell is appropriate when state ownership, authority, deployment
or lifetime must be independent. This avoids imposing a chain and deployment
lifecycle on every short reasoning step.

A Hermes skill can remain versioned instructional content in memory, interpreted
by soma. A runnable extension with state and acts can be a cell definition.
Scheduled triggers come from plan; incoming conversations from sense; cognition
stays in soma. Cell supplies the common execution container and recovery seams.

## verified implementation gaps

These findings constrain extraction; this review covers the named paths.

1. `Cell::commit` applies a signal before writing its tape frame and discards
   write/flush errors. `SharedCell::open_default` silently falls back to ephemeral
   storage on open failure. A stronger durability contract requires explicit
   commit semantics and visible failure.
2. `receive_frame` treats `ChainError::Equivocation` as a benign duplicate.
   Foculus already distinguishes chain-position identity (`hash`) from semantic
   content identity (`content_id`). The new ingress must distinguish duplicates,
   gaps and actual conflicts and route conflicts to the proper policy.
3. Cybergraph appends to its SignalChain before bbg insertion. Box moves now
   reach that insertion, which can reject a double spend. Validation and atomic
   application need a substrate contract; cell cannot manufacture atomicity by
   wrapping a partially mutating call.
4. `Cybergraph::chains` is keyed by neuron; Signal also carries a destination
   network. Several independently ordered cells using one neuron cannot simply
   instantiate copies and each begin at step zero. An explicit relation between
   CellId, network, writer and stream is required.
5. `run_cell` wraps its base subject as `[event [state base]]`, while the act
   handler reads caps at absolute axis 30. A compiled Rust probe against current
   rune crates demonstrated the layout mismatch described below.
6. Host results are prepended to the subject during ordinary act evaluation.
   The same probe demonstrated caps moving between two sequential acts.
7. The robot world's loading/evaluation helpers currently have no call site in
   that module's setup path. The visible page is native construction. Connecting
   the loaded definition to the actual product path is part of the extraction.
8. Rune has an event driver and shallow continuation support; a universal durable
   continuation format and uniformly recorded host witnesses remain work.
9. Two Cell implementations exist in cyb. Published CLI consumers and the native
   workspace must both move to the extracted implementation.

### authority probe

An external temporary Cargo harness linked `rune-ast`, `rune-interp` and
`rune-subject` by path and ran offline on 2026-09-11. Repository code was unchanged.
Its base subject used `now = 11`, `here = 12`, `caps = 13`; its Host recorded each
`perform` call's caps argument and returned atom zero.

For gate `[16 [[EMIT [1 7]] [1 0]]]`, direct evaluation receives caps 13. The same
gate through `run_cell(base, gate, 0, host, [99])` receives caps 11. Nesting the
same act as the first act's continuation receives 13 and then 12.

Observed output, with assertions passing:

```text
direct act sees caps: [Atom(13)]
run_cell act sees caps: [Atom(11)] (base.now)
two sequential acts see caps: [Atom(13), Atom(12)] (caps, then here)
```

This verifies argument propagation in the current Host ABI. It does not test
ward enforcement, which is still to be built. The architectural consequence is
a host-bound authority context plus a defined runtime subject mapping.

## decisions captured in the specification

1. [Birth and naming](../specs/model.md): stable identity from canonical Birth,
   optional human names, separately versioned definitions and replicas.
2. [History and stream scope](../specs/history.md): cell-scoped records in
   cybergraph, coordinated per-neuron protocol writers, explicit proof of
   filtered history. Independent consensus profiles supply their own protocol.
3. [State representation](../specs/data.md): canonical stack data and referenced
   application state, with identity and commitments delegated to the stack.
4. [Evolution](../specs/evolution.md): atomic activation/migration, preserved
   obligations, verified artifact closure and enforced placement fencing.

## experiments that can decide the design

The next executable examples should test the common contract in increasing
scope, after the discussion establishes it:

- One rune counter definition runs in a headless host and a cyb surface. Two
  instances retain distinct state; restarting either restores its own value.
- One cell requests a second cell's operation. In-process and radio delivery
  preserve the same authority, identifiers, receipts and duplicate handling.
- A soma task pauses for a ward decision, survives restart and consumes a
  recorded tool result without executing the completed operation again.
- Fault injection interrupts persistence, state application and external
  delivery at each boundary. Recovery exposes conflicts and unknown outcomes.
- An upgrade changes the definition while preserving instance identity and
  state lineage. A move to another body restores referenced artifact bytes and
  prevents two active owners of the same execution lease.

These examples establish a useful runtime-cell and its reusable host. Building,
ledger and knowledge profiles then exercise additional admission and finality
policies against that same model.
