---
title: neuron clean checkout against sibling defaults — 2026-10-17
date: 2026-10-17
status: red
---

# neuron clean checkout against sibling defaults — 2026-10-17

Property #39: every phase-1 component builds and tests from a clean checkout of
its default branch against the default branches of its siblings. On 2026-10-17
neuron fails this check. The cause is not the mudra gap recorded in
[mudra-neuron-spell-build-gap-2026-09-24](mudra-neuron-spell-build-gap-2026-09-24.md),
which is closed. neuron main now depends on sibling work that is only on
unmerged owner DECIDE pull requests and release lines.

## command

All siblings are detached worktrees at their origin default branch, placed
beside a neuron checkout of `origin/main` (c875411, then a7e1385 after
neuron#12; neither changes a manifest):

```
cargo test --locked --workspace
```

Sibling revisions: cybergraph 0a70b6a, foculus c9dcc32, bbg eb152fa,
nox 7a52650, zheng 5733dfd, lens 8b26cef, vault c980557, rune 67281db.

```
error: failed to select a version for `cybergraph`.
    ... required by package `neuron-node v0.1.0 (.../neuron/node)`
package `neuron-node` depends on `cybergraph` with feature `legacy-redb-migration`
but `cybergraph` does not have that feature.
```

Resolution fails, so no crate in the workspace compiles.

## what neuron main expects, and where it lives

Each row is the next blocker found after substituting the branch named in the
row before it. The run was stopped at the last row.

| need | default has | carried by |
|---|---|---|
| cybergraph features `local-storage`, `legacy-redb-migration` | no `[features]` | cybergraph#3 (DECIDE, `feat/durable-applications` 0eae7ae) |
| cyber-nox 0.3 (cybergraph#3 and neuron's lock) | nox master 0.2.0 | nox `release/0.4` 12adaf8 |
| zheng 0.4 (neuron's lock) | zheng master 0.3.3 | zheng `release/0.4` 8f20262 |
| vault feature `local-host` | features `default`, `graph-store` | vault#1 (DECIDE, `feat/spell-cli` f3fe2e4); also vault#10 |
| cyber-lens 0.2 (required by bbg master) | lens master 0.1.3 | lens `release/0.2` eeb58cf |
| foculus pins bbg 0.3 | foculus master pins bbg ^0.2 | foculus#5 (DECIDE, `fix/shared-storage-dependencies` e43f3a6) |
| `rune_lower::lower_bounded`, `rune_parse::parse_bounded`, `rune_interp::{codec, machine}` | absent | rune#6 (DECIDE, `feat/resumable-authority-bound-runtime` bbad0f1) |
| foculus#5 against zheng `release/0.4` | — | does not compile: unresolved `zheng::{fold, decide, ccs, phi}`, `zheng::types::{Accumulator, ProofGroup, Statement, TraceProof}`, `bbg::CheckpointProof::step_count` |

With every branch above in place, the workspace resolves only without
`--locked` (Cargo.lock no longer matches) and stops on foculus. No published set
of sibling heads builds neuron main today.

## what still verifies

`neuron-id` and `neuron-model` have no path dependency on cybergraph, vault,
foculus or rune. In an isolated workspace with only `id/` and `model/` against
nox master 7a52650 and hemera main fd55f99:

- `cargo test -p neuron-model -p neuron-id --all-features`: main 10 tests;
  after neuron#12, 37 tests; default features 36; `--no-default-features` 5;
  zero warnings
- `cargo clippy -p neuron-model --all-features --all-targets`: clean

## earlier row-39 findings in this repository

- 2026-09-23: bbg, tok and mudra needed `neuron-id`, which existed only in a
  working tree (neuron#1). Fixed on main by 3be383a, which added the `id` crate.
- 2026-09-24: neuron and vault called `mudra::neuron` and `mudra::spell` before
  mudra's origin had them (neuron#8,
  [mudra-neuron-spell-build-gap-2026-09-24](mudra-neuron-spell-build-gap-2026-09-24.md)).
  Fixed by mudra#2, merged 2026-10-10.

## remains

The owner decides cybergraph#3, vault#1, foculus#5 and rune#6. One bump PR
per repository then moves the nox, zheng and lens release lines to their
defaults, or moves neuron's pins back to the defaults. After that, rerun the
command above.
