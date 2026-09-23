---
title: cell
tags: cell, soft3
status: experimental implementation
---
# cell

Cell is the shared model and host for addressable, stateful organs, services,
ledgers and cybergraph regions. In cyb, a loaded cell grows an ability of the robot.

Start with [the specification](specs/README.md).
The runtime-free `neuron-id` and `neuron-model` packages also supply Cyber's
identity references and bound action envelopes. The legacy runtime packages
below still use their `cell-*` names and import `neuron-model` through a Cargo
alias. [The dependency audit](audit/node-model-dependency-2026-09-23.md) records
this boundary; publishing the model does not complete runtime convergence.
The [convergence explanation](docs/cell-convergence.md) records the reasoning,
source review and relationship to the existing stack.
The [foundations and Hermes review](docs/foundations-agent-review.md) explains
the 0.2 corrections and the evidence needed to demonstrate a stronger agent.

The Rust workspace now runs local rune cells with persistent state, graph
history, resumable tool calls and recovery across processes. This is the first
integration slice of the 0.2 design, with an explicit experimental local profile.
It does not yet claim the complete runtime baseline or agent/Hermes parity.

Build with the companion repositories under ~/cyber and the upstream changes
listed in [implementation status](docs/implementation.md):

```nu
cd ~/cyber/cell
cargo build -p cell-cli
let created = (./target/debug/cell --store demo.bbg create examples/counter.rune | from json)
./target/debug/cell --store demo.bbg submit $created.cell 7
./target/debug/cell --store demo.bbg submit $created.cell 5
./target/debug/cell --store demo.bbg inspect $created.cell
./target/debug/cell --store demo.bbg history $created.cell
```

The counter's final state is 12. Each command opens the same graph in a new
process. `~mem` is application state; `event` is the admitted noun; the returned
noun becomes the next state. Source executes through parse → lower → interpret.

For effects, [the tool example](examples/tool.rune) suspends at `host(event)`.
Create it with `--allow host`, submit an input, then use `take CELL OPERATION` to
record an attempt before execution. Record its result with
`outcome CELL OPERATION ATTEMPT VALUE`, or its definite failure with
`fail CELL OPERATION ATTEMPT REASON` followed by `run CELL`. An unresolved attempt
reopens as `unknown-outcome`; the operator reconciles it by those same identities.
`emit` is the only built-in automatic executor; other acts require an adapter.
All acts default to denied. [The profile](specs/local-runtime.md) states limits.

`pause`, `resume`, `cancel` and `retire` control the instance. Paused instances
can retain outcomes; an unresolved attempted effect blocks cancellation/retirement.
`--nonce` on create/submit makes retries explicitly reproducible. One origin's
nonce binds one event across the database, including its destination and context.
`history --after N --limit K` renders the retained graph records in bounded pages.

History is represented in cybergraph and persisted through its storage stack.
Cell defines and coordinates its transitions. The cyb log organ presents history.

The default store is a `bbg` directory using BBG's Fjall backend. `--store`
selects another BBG directory. Graph adapters can also share an already opened
BBG Database with other storage views. Default builds do not enable redb.

An existing `cell.redb` requires explicit migration or `--store` selection;
the CLI stops before creating an empty default session. To import it:

```nu
cargo build -p cell-cli --features legacy-redb-migration
./target/debug/cell migrate-redb cell.redb bbg
./target/debug/cell --store bbg history CELL_ID
```

Replace `CELL_ID` with the instance's existing particle. The destination must
be fresh. Migration preserves the source and imports through Cybergraph/BBG;
an interrupted destination cannot be used as a completed session. Since the
source remains, continue selecting the imported directory with `--store bbg`.
The [storage migration audit](audit/shared-bbg-database.md) records validation.

## repository

- specs/ — normative candidate contracts, version 0.2.
- docs/ — explanations and source findings.
- model/ — canonical data, manifests, identities and snapshots.
- engine/ — lifecycle, admission, budget reservations, effects and recovery ports.
- rune/ — source, subject and bounded checkpoint adapter.
- node/ — cybergraph/BBG and local authorization composition.
- cli/ — the same engine exposed as the `cell` executable.
- LICENSE — the Cyber License used by the companion repositories.

Companion repositories are siblings under ~/cyber. Relative source links assume
that layout. [Integration](specs/integration.md) identifies dependency changes
needed to implement the contracts; present code is distinguished from the target.

Run `cargo test --workspace` and
`cargo clippy --workspace --all-targets --no-deps -- -D warnings` here.
Format only these packages with
`cargo fmt -p neuron-model -p neuron-id -p cell-engine -p cell-rune -p cell-node -p cell-cli`;
`--all` also traverses the companion path dependencies.
