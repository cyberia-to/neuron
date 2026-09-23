---
title: Node model dependency
date: 2026-09-23
status: tested dependency candidate
---
# Node model dependency

Cyber and Mudra consume `neuron-id` and the runtime-free `neuron-model`. This
change publishes those packages from the ongoing Neuron convergence without
including unfinished runtime, CLI or hosting changes.

`neuron-id` is a dependency-free, no-std 32-byte identity type. The model exposes
native/foreign subject and network references, revision-bound action context,
navigation references and canonical signed-action envelopes. Records and worker
execution descriptions remain behind the default `records` feature; `serde`
enables bounded canonical JSON transport. Secret storage remains in Vault.

Envelope decoding checks shape and canonical encoding, not cryptographic
authority. Callers must verify Mudra evidence and admission policy. A well-formed
102-byte `NSIG1` envelope is deliberately not treated as an authorization result.

Existing `cell-*` runtime packages keep their names and import the renamed model
through a Cargo alias. Existing schema-suite records keep their encoding; the
Neuron schema suite has its own name. This is a dependency boundary, not a claim
that the full Neuron runtime design has been implemented.

Validation on macOS arm64:

- `cargo test -p neuron-model --all-features`: 10 tests passed.
- `cargo test --workspace --locked`: 22 tests passed, including the existing
  persistent runtime and graph recovery tests.

The source lock in Cyber identifies the companion revisions for the node
profile. The owner's broader unfinished convergence remains outside this commit.
