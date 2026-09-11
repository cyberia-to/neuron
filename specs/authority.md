---
title: authority
tags: cell, soft3, spec
status: draft
spec-version: "0.1"
---
# authority

Ward owns grants, policy evaluation, delegation, revocation and the execution
boundary. Vault holds secrets and performs permitted secret operations.
Cell binds those facilities to instance lifecycle and operation identity.

## authority context

An execution context binds (CellId, DefinitionId, invocation EventId,
governing policy/head, grant references, executor epoch, delegated budget).
Its live handle is owned by the host. Serialized checkpoints hold references
sufficient to request rebinding; they cannot restore a live grant by themselves.

Program-visible caps are an inspectable projection or opaque handle. Ward checks
the host-held context on every act. Mutation of rune subject, nested invocation,
returned host data or a fabricated noun cannot enlarge authority.
The runtime adapter must preserve the context across all control-flow changes.

Authentication identifies who submitted a request. Authorization decides whether
that principal may perform the requested operation at the current state.
A valid signature or valid execution proof alone does not authorize it.

## policy ownership

Soul describes the robot's configuration and intended policy. Ward interprets
that policy, holds grants and issues decisions. The instance's governing policy
specifies who can activate, upgrade, relocate, delegate and retire it.
These references are authenticated graph records; their private material follows
the graph's privacy policy.

An installation manifest requests rights. Activation receives at most the
intersection of those requests, the issuer's available rights and current policy.
New rights on upgrade require a new decision. A model-generated preference,
memory entry or imported instruction is ordinary input.

## delegation and revocation

Delegation binds recipient instance/definition, permitted acts, target scopes,
expiry, further-delegation limit and resource budget. Delegated rights are a
subset of the delegator's current authority. Cross-cell calls authenticate both
the caller and any explicitly delegated rights.

Revocation changes the governing grant state. Before dispatch, ward MUST check
the current grant and executor epoch. A prior Authorized record documents a
decision; it cannot override a later revocation. A dispatch racing revocation
has an explicit linearization point at the enforcing executor and its result
records that ordering.

Revocation cannot undo an external operation already accepted. Its outcome
remains tracked. Rebinding after restart or relocation verifies current policy
and refuses stale epochs, expired delegation and missing authority evidence.

## all world access

Graph reads, graph writes, surface output, file/process/network operations,
device use and cross-runtime invocation pass their appropriate ward boundary.
Bootstrap components receive explicit host authority before loading cells.

Sandboxed runtimes expose only gated imports or equivalent host acts.
Native Rust adapters are trusted host code: in-process Rust can access ambient
process capabilities. A profile needing confinement of untrusted native code
requires an enforced process/sandbox boundary through body, or a supported
confined runtime. A manifest scan alone cannot prove confinement of dynamic code.

Secret references identify vault operations. Secrets do not appear in ordinary
Snapshot, checkpoint, public receipt or view fields. A permitted secret-returning
operation uses an explicitly private channel and retention policy.
Credentials and signing keys are never implicit dependencies of a definition.

## permission interaction

A ward decision is allow, deny or pending. Pending includes a durable request
identity and context digest for the user's decision. The relevant cell waits
without holding a mutable graph transaction or runtime stack.
A response binds the exact release, target, arguments and requested rights.
A changed request requires a new decision.

Denial is a definite outcome and may be consumed by program logic.
Unavailable authorization infrastructure produces AuthorityUnavailable and leaves
the operation undispatched. Rendering the prompt is a cyb/prysm concern.
