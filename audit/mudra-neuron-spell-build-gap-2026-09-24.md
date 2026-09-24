# neuron's clean checkout does not build: it targets mudra APIs not yet on mudra's origin

Property #39 — every phase-1 component builds and tests from a clean checkout
of its default branch against the default branches of its siblings.
Lane C · content, filed against neuron because that is where the build fails.

2026-09-24, neuron `origin/main` at `c875411`, mudra `origin/master` at
`e2d1a63`.

## the break

`node/src/authority.rs` calls two functions on a module that does not exist
on mudra's origin:

```
$ git -C neuron show origin/main:node/src/authority.rs | grep -n mudra::neuron
28:        mudra::neuron::sign(&self.key, subject, statement).map_err(|_| Error::Denied)
32:    mudra::neuron::verify_statement(subject, statement, evidence)
```

`node/tests/vault_custody.rs` calls the same module. `cli/src/custody.rs`
calls a constant on a second module that does not exist on mudra's origin
either:

```
$ git -C neuron show origin/main:cli/src/custody.rs | grep -n mudra::spell
50:                        .unwrap_or_else(|| mudra::spell::COSMOS_PATH.into()),
```

mudra's origin/master ships neither module:

```
$ git -C mudra show origin/master:src/lib.rs | grep 'pub mod'
pub mod claim;
pub mod cosmos;
pub mod domain;
pub mod proof;
pub mod seed;
```

Confirmed by an actual build: with mudra's path dependency pointed at a
worktree of `origin/master` (`e2d1a63`) instead of the owner's working
tree, `cargo check --tests` from neuron's workspace root fails before it
even reaches `node`'s own `authority.rs` — `vault`, pulled in as a
neuron workspace path dependency, hits the same missing modules first:

```
$ RUSTC_BOOTSTRAP=1 cargo check --tests
    Checking cyber-vault v0.1.0 (.../vault)
error[E0433]: cannot find `neuron` in `mudra`
   --> vault/src/operation.rs:270:28
    |
270 |                 if !mudra::neuron::verify_statement(request.subject, request.statement, evidence) {
    |                            ^^^^^^ could not find `neuron` in `mudra`
error[E0433]: cannot find `neuron` in `mudra`
   --> vault/src/operation.rs:301:28
error[E0433]: cannot find `spell` in `mudra`
  --> vault/src/record.rs:72:20
   |
72 |             mudra::spell::derive(words, passphrase).map_err(|_| Error::InvalidInput)?,
   |                    ^^^^^ could not find `spell` in `mudra`
error[E0433]: cannot find `neuron` in `mudra`
  --> vault/src/use_secret.rs:74:35
   |
74 |             let evidence = mudra::neuron::sign(signing, request.subject, request.statement)
   |                                   ^^^^^^ could not find `neuron` in `mudra`
error[E0433]: cannot find `spell` in `mudra`
  --> vault/src/use_secret.rs:98:24
   |
98 |                 mudra::spell::signing_key(&spell, path).map_err(|_| Error::InvalidInput)?;
   |                        ^^^^^ could not find `spell` in `mudra`
error: could not compile `cyber-vault` (lib) due to 5 previous errors
```

The break is wider than neuron's own two call sites: `vault` (a workspace
path dependency here, and its own repo, `cyberia-to/vault`) uses
`mudra::spell::derive`/`signing_key` and `mudra::neuron::sign`/
`verify_statement` in its own `src/`, four more call sites than the two
`neuron` has. `cargo check --tests` against mudra's real origin stops on
`vault` before it ever reaches `node`'s or `cli`'s own errors — a hard
compile error, not a warning, and it blocks the whole workspace.

## this is not a stale reference — it is a forward reference

Both modules exist, complete and already shaped to match every call site
above, in the owner's local mudra working tree (not committed, not on any
mudra branch, not part of any open mudra pull request):

- `src/neuron.rs` — `sign`, `verify_statement` (compat alias
  `sign_statement`), an `NSIG1`-tagged proof format built on
  `claim::sign_arbitrary`/`claim::neuron_of`, matching `authority.rs`'s
  `(subject, statement) -> evidence` / `(subject, statement, evidence) ->
  bool` shapes exactly.
- `src/spell.rs` — `pub const COSMOS_PATH` at the same value origin's
  `src/seed.rs` already has (`m/44'/118'/0'/0/0`); `seed.rs` itself is
  dropped from the local tree's `pub mod` list in `lib.rs`, i.e. `seed` was
  renamed to `spell` locally, not left standing alongside it.

vault#7 (`cyberia-to/vault#7`, open) independently names the same gap from
vault's side: "rebuild against mudra's spell rename, cover Header/Keys."
Two consumers were already written against mudra's next version before that
version reached mudra's own origin.

## why this is not a neuron-side fix

Reimplementing `mudra::neuron::{sign, verify_statement}` inside neuron
without mudra's `neuron` module would mean re-deriving the `NSIG1` proof
format and its binding to `claim::neuron_of` independently, forking a
security-relevant encoding that already exists, reviewed, in the owner's
tree. Same for re-deriving `COSMOS_PATH` under a different name. Both are
one `git add`/commit/push away in mudra, not a code problem in neuron.

## remains

Push mudra's local `src/neuron.rs` and `src/spell.rs` (and the `seed` →
`spell` rename in `lib.rs`) to mudra's origin — an owner action per the
isolation rule (uncommitted owner working trees are never staged or pushed
by a worker). Once mudra's origin carries both modules, neuron's clean
checkout is expected to build with no change on neuron's side; re-run
`cargo check --tests` in both `cli/` and `node/` to confirm and close this
row's neuron half.

This is independent of neuron#1 (row 39, the separate `neuron-id` crate gap
for bbg/tok/mudra) and of mudra#38/#38(2)/#39 (mudra's own `cyber-nox`/
`zheng` pin and `bbg_root` gaps) — none of those touch `mudra::neuron` or
`mudra::spell`.
