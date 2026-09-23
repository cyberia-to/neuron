---
title: bbg, tok and mudra could not build from a clean neuron checkout
tags: neuron, bbg, tok, mudra, audit
date: 2026-09-23
status: fixed
---

# bbg, tok and mudra could not build from a clean neuron checkout

`bbg/rs/Cargo.toml`, `tok/rs/Cargo.toml` and `mudra/Cargo.toml` each declare a
path dependency on `neuron-id` at `../../neuron/id` (or `../neuron/id`).
`bbg/rs/src/types.rs` re-exports it publicly: `pub use neuron_id::NeuronId;`.

That `id` crate did not exist in this repository's git history — not on
`main`, not in any prior commit. `neuron`'s own `Cargo.toml` did not list it
as a workspace member either. A genuine clean checkout of `neuron` alongside
`bbg`, `tok` or `mudra` — a fresh clone, or the launch worker's isolated
worktree convention (`git worktree add ... origin/main`) — failed before
compiling a single file:

```
$ cargo check --workspace --tests   # in a fresh neuron checkout, with bbg/tok/mudra as path siblings
error: failed to get `neuron-id` as a dependency of package `bbg v0.3.0 (.../bbg/rs)`
Caused by:
  unable to update .../neuron/id
Caused by:
  failed to read `.../neuron/id/Cargo.toml`
Caused by:
  No such file or directory (os error 2)
```

Every prior launch worker verification of bbg, tok and mudra building "from
a clean checkout" ran against the owner's live working tree, where this
crate exists uncommitted (part of a larger, still in-progress `cell`→`neuron`
package rename spanning `model`, `engine`, `node`, `cli` and docs, well
outside the scope of this fix). The launch mirror's per-repo symlinks resolve
relative path dependencies against that same live tree unless a repo is
checked out into its own worktree, which is exactly what masked this gap:
bbg/tok/mudra never actually got exercised against a clean `neuron`.

## Fix

Added the standalone `id` crate exactly as it already existed, uncommitted,
in the owner's tree: a single `#![no_std]` type alias, `pub type NeuronId =
[u8; 32]`, no dependencies. Registered it as a workspace member. This is
independent of the rest of the in-progress `cell`→`neuron` rename — nothing
else in this repository or its consumers needed to change.

## Validation

macOS arm64, this repository's own worktree (`launch/127-neuron-audit`,
based on `origin/main`), 2026-09-23:

```
$ cargo check --workspace --tests
Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.09s
$ cargo test --workspace
15 tests across the workspace, 0 failed
```

Cargo's own resolution during that check pulled in `bbg`, `cyber-tok` and
`foculus` from their path-dependency chain through `cell-node`/`cell-cli`
and confirmed they now resolve `neuron-id` correctly. `mudra` was not
directly exercised here (it is not a dependency of anything in this
workspace) but declares the identical `neuron-id = { path = "../neuron/id" }`
dependency and resolves through the same mechanism.
