---
title: api
tags: cell, soft3, spec
status: draft
spec-version: "0.2"
---
# API and host composition

This is the semantic API. Rust names are candidate names; an implementation must
preserve the specified identities, results and failure behavior. Adapters may use
async methods, channels or explicit polling without changing the contract.

## model values and API records

HostId is an authenticated host principal managed through vault/mudra. A host's
ephemeral process boot id is separate; restarting does not mint a new CellId.

Query = (cell, at, selector, requested_evidence, limit, cursor, budget, deadline).
at is a Head or latest_selected; selector is an application schema/query particle.
budget references finite graph-access limits; deadline is a TimePoint under a
supported clock. Pagination consumes the enclosing caller/task budget and cannot
reset it. The actual limits intersect the host and reader's granted limits.
Page = (cell, head, records, cursor, completeness, evidence).
records is an ordered list of particles; completeness is complete, partial with
a reason, or unavailable with an available history boundary.
HostInfo = (host, boot, supported_schemas, runtimes, profiles, codecs,
durability, resource_limits).
Error = (code, request, cell, head, detail, retry_condition).
Optional fields are absent when unknown. Error detail is structured, private
under the request's scope. Retry conditions are explicit.

## caller operations

| Operation | Input | Result |
|---|---|---|
| publish | definition artifacts, author context, request nonce | DefinitionId and publication receipt |
| create | Birth and content closure, author context | CellId and birth receipt |
| inspect | identity, optional Head, reader context | definition/state/placement metadata and evidence |
| manage | cell, expected Head, lifecycle/upgrade/relocation Event | receipt or pending decision |
| submit | Event and authenticated context | admission/commit receipt or pending proposal |
| resolve | request/transaction/operation identity | retained receipt/outcome or Unknown/HistoryUnavailable |
| query | Query and reader context | Page |
| subscribe | Subscription and reader context | durable stream handle/cursor |
| acknowledge | subscription, cursor, consumer commit | receipt |
| cancel | target invocation/operation, expected state, context | definite or pending cancellation receipt |
| export | cell, Head, backup/replication policy, context | verified manifest and export receipt |
| restore | Backup, content, context, requested placement | replica/activation status and evidence |

All mutating requests have stable origin-scoped nonces and idempotent lookup.
A control operation cannot bypass the same event/authority/history boundaries
used by runtime-originated work. Completion of a method is not inferred from
an open socket or a rendered message; callers inspect its receipt stage.

The local facade additionally accepts run_source(source, runtime, input, at,
application_context, authority_context, limits, request_nonce). at selects the
retained snapshot for read/view execution. It derives and retains a pinned
minimal Definition. Pure read/view gates execute against a supplied retained
snapshot; a stateful gate composes create/activate/submit, reusing stable request
identities on retry. The result identifies the definition and any created instance
plus the applicable result/receipt. Authenticated local provenance satisfies the
local profile; remote publication is a separate publish operation. This facade
preserves [instant start](foundations.md) and all effect/durability gates.

## required ports

| Port | Required operations and obligations |
|---|---|
| GraphPort | get/head/query; conditional transaction; lookup transaction; snapshot-and-follow; retain/export closure; fail with structured persistence errors |
| RuntimePort | inspect/start/resume; enforce declared resources; produce defined RunStep/checkpoints |
| WardPort | request/rebind/attenuate/revoke/check-and-dispatch; bind decisions to current context/epoch |
| VaultPort | authenticated identity and permitted secret operations |
| FinalityPort | validate proposal, select/verify head, verify authority/epoch continuity |
| EvidencePort | verify exact claims/anchors, expose assumptions and freshness |
| TransportPort | authenticated bounded envelope streams and acknowledgements through radio |
| ContentPort | retain/fetch verified artifacts under graph access and durability policy |
| BodyPort | bounded resources, process placement, device/executor handles |
| SurfacePort | render prysm output at a pinned view version, submit input events |

Ports name responsibilities. An implementation may expose several through one
stack adapter. The engine depends on narrow interfaces; cybergraph has no
dependency on cell's concrete runtime code.

GraphPort transaction outcomes are Committed(receipt), Pending(proposal id),
Rejected(error), or Unknown(transaction id). Pending has durable proposal data
and no selected authoritative head. Unknown prohibits dependent dispatch until
lookup/recovery settles the result. Repeating an identical transaction id returns
the existing outcome; changing its content returns Conflict.

## errors and retries

| Code | Meaning / permitted recovery |
|---|---|
| InvalidData | Malformed canonical data; correct the request |
| UnsupportedSchema / UnsupportedRuntime / UnsupportedProtocol | Required semantics absent; install a compatible implementation |
| UnsupportedProfileChange | Governance/finality transition unsupported |
| Unauthenticated / Denied | Identity or permission check failed |
| AuthorityUnavailable / GrantRevoked / StaleEpoch | Obtain current valid authority; keep effects undispatched |
| MissingArtifact / EvidenceUnavailable | Retrieve required content/evidence |
| EvidenceInvalid / EvidenceExpired | Reject claim or obtain a valid fresh one |
| HeadMismatch / LifecycleConflict | Inspect current head and issue an explicit new proposal |
| Gap / Conflict | Obtain missing history or invoke governing conflict policy |
| ResourceExceeded / Backpressure | Use declared bounded retry or lower workload |
| DeadlineExceeded | Definite only before possible dispatch; otherwise reconcile |
| StorageUnavailable / CorruptHistory | Stop dependent work; repair/recover with evidence |
| CommitUnknown / OutcomeUnknown | Resolve by identity; never infer failure from lost reply |
| HistoryUnavailable / ResyncRequired | Resume from a verified available boundary |
| CancelPending / UpgradeBlocked / RelocationBlocked | Complete the named pending condition |
| RuntimeFault | Inspect recorded fault; resume/retry only under runtime contract |

Errors carry machine-readable codes. Diagnostics may accompany them but cannot
replace a required result or assert a stronger evidence/durability level.

## composition

The baseline composition binds graph/storage, identity, ward and profile
verification before loading executable cells. Cyb and the headless CLI use that
same composition; cyb adds window/chrome and surfaces.

The log, memory, brain and time organs consume graph projections. The engine
MUST be operable without those views or Bevy. The cell extension organ exposes
loading/lifecycle capabilities through cyb; it does not own the semantics of
every organ implemented with a cell-compatible entrypoint.
