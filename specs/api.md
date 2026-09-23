---
title: API and host composition
tags: neuron, prog, soft3, spec
status: accepted
spec-version: "0.3"
---
# API and host composition

The minimal model exposes identity independently of execution. The native engine
adds durable progs and invocations for an existing subject. Product hosts compose
custody, registry, policy, storage and workers; the engine remains headless.
[Integration](integration.md) defines the crate and owner boundaries.

## Native engine surface

`Neuron<G, R, A>` composes GraphPort, RuntimePort and Authority. `Neuron::new`
uses NoAuthority for reads; `with_authority` supplies explicit mutation rights.
The actual Rust API lives in engine/src rather than the retired Cell facade.

| Operation | Contract |
|---|---|
| `inspect`, `state` | Bounded selected root/prog artifact read for NeuronId |
| `activate` | Authenticated root for an existing subject, network, policy and finite budget |
| `install` | Code/state artifacts, stable installation nonce and bounded config → prog data ID |
| `submit` | Prog, stable nonce, input, optional context/parent and allowance → invocation receipt |
| `lookup_admission` | Verify and return the original nonce claim without new admission |
| `tick`, `tick_invocation` | Bounded fair/specific progression → idle/yield/wait/unknown/event/terminal status |
| `begin_attempt` | Manual attempt metadata; repeated lookup cannot grant another dispatch |
| `begin_worker_attempt` | Fresh non-clonable permit for the guarded worker path |
| `record_outcome`, `record_failure`, `wake` | Correlated retained observation/event and single consumption |
| `manage`, `upgrade` | Program lifecycle or explicit code/state revision change |
| `change_policy`, `rebind` | Authorized policy/epoch transition and explicit invocation rebinding |
| `cancel`, `archive` | Settle permitted obligations or reject Busy/UnknownOutcome; retain history/claims |
| `prepare_import`, `activate_import`, `import` | Validated legacy origin mapping and atomic source fences |

Management transitions use current selected state and CAS. Stable caller nonces
are specifically exposed for installation, admission and import; a caller must
not assume every management method has a user-supplied idempotency key. Lost
mutation replies require inspection of the selected records before new work.

`ProgramConfig` specifies step allowance, max inflight work and requested act tags.
`Admission` pins prog, nonce, input, optional context/parent and allowance. `View`
returns actual head/root/state; `Receipt` returns invocation and admitted head.
These native types do not pretend to carry an unimplemented consensus proof or
a generic cross-network transaction.

## Ports and ownership

| Surface | Actual responsibility |
|---|---|
| GraphPort | Content source, head, bounded history, request lookup, conditional commit and fresh `commit_once` |
| MigrationPort | Atomic import publication with exact source heads and fences |
| RuntimePort | Value/checkpoint validation, start and bounded step with serialized continuation |
| Authority | Exact statement authorization plus current-grant guard through commit/dispatch |
| Worker adapter | Existing worker descriptor/placement, generation, one-use claim and physical handoff |
| SharedGraph | Native verification and Cybergraph/BBG application transactions over the existing Database |
| Product Host/Registry | Named robot attachments, native custody, explicit networks/devices, current binding revisions |
| Soma Agent | Task/context/strategy/controls, child joins, schedules, observations and learning proposals |
| Inf query adapter | Optional immutable execution projection with scope/head metadata and no authority |

Ward, vault, body, radio, prysm, Foculus and evidence verifiers retain their own
interfaces. Their responsibilities are not fabricated as ten new mandatory Rust
traits in this engine. Cybergraph exposes opaque application storage without
importing the neuron engine or renderer. Identity-only browser/SDK/query clients
must not pull in execution, BBG, GUI or inference through type re-exports.

## Operator and product composition

The [standalone neuron CLI](cli.md) supports execution and migration with explicit
key/store/network/grant selection. `cy neuron` manages the named robot's bindings;
`cy task` and the cyb body operate the same Soma/neuron composition and database.
These entry points have different jobs and must be named accurately in tool
registries. Opening a view or inspecting a subject creates no key or VM authority.

Immediate source evaluation remains available through Rune and the existing cyb
console/view host. Pure one-shot evaluation can use a retained snapshot without
installing a prog. Durable state/effects use explicit install/admit/execute with
captured subject and request identity. Compiling a source file or opening a URI
cannot create a second principal or implicitly grant world access.

Protected read/query/export APIs belong to the hosting privacy profile. A future
subscription surface must meet [communication](communication.md)'s cursor and
backpressure contract. Current GraphPort history polling and optional inf tables
make no generic live-stream, remote execution or arbitrary proof service claim.

## Errors and recovery

| Native error | Meaning / recovery |
|---|---|
| Data | Canonical/value/schema/limit failure; retain source and correct input |
| Graph | Backend or validated graph failure; stop dependent work and inspect |
| CommitUnknown | Resolve the original request/head before dispatching dependent work |
| Runtime | Bounded interpreter/adapter failure; inspect retained fault |
| Missing | Required subject/prog/invocation/content absent in this scope |
| Conflict | Stale predecessor, changed nonce content, result revision or evidence mismatch |
| Busy / Lifecycle | Outstanding obligations or invalid program transition |
| Denied / Fenced | Missing current authority or retired/stale writer placement |
| UnknownOutcome | Attempt may have executed; reconcile without blind retry |
| Budget | Finite allocation, accounting or bounded collection exceeded |
| Unsupported | Required adapter/codec/profile/guard semantics unavailable |

Applications may refine these diagnostics into their own machine-readable error
schemas while preserving uncertainty and retry conditions. A rendered message,
open socket or exit from a host callback cannot substitute for a retained receipt.
