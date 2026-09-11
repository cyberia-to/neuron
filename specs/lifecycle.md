---
title: lifecycle
tags: cell, soft3, spec
status: draft
spec-version: "0.1"
---
# lifecycle

Lifecycle is durable instance state. Host execution status describes the current
placement and is observed separately. Closing a view detaches its surface.

## durable states

| State | Admitted work | Exit |
|---|---|---|
| Installed | Inspection, verification, grant negotiation, activation, retirement | Active or Retiring |
| Active | Application events, outcomes and management requests | Paused or Retiring |
| Paused | Outcomes, inspection, authorized resume/upgrade/relocation/retirement | Active or Retiring |
| Retiring | Outcomes and cleanup needed to settle already committed work | Retired |
| Retired | Historical reads and export allowed by retention policy | Terminal |

A management request is an authenticated Event. Its result is committed through
the same head comparison as application work. Repeating a request nonce returns
its existing receipt. Unsupported changes fail without a lifecycle transition.
A retired instance can seed a fork with a new birth identity.

## installation and activation

Installation commits Birth, initial state and required artifact closure together.
A release may be cached before this operation; a cache entry alone creates no
instance. Grant negotiation references the pinned release and requested rights.

Activation MUST verify:

1. Definition identity, publication provenance and the governing policy.
2. Input/state/checkpoint schemas and required extension protocols.
3. Availability and integrity of executable and immediate resumption artifacts.
4. A compatible runtime with enforceable resource limits.
5. Ward grants and the required executor epoch/placement authority.
6. Storage durability and profile finality/evidence support.

Successful activation commits Active and the selected definition/grant references.
The host may start scheduling only after the required receipt/finality gate.
Failed activation leaves Installed or Paused and returns a typed cause.

## host states

Host placement states are Restoring, Ready, Running, Waiting, Quarantined,
Unavailable and Detached. They report a pinned durable head where available.

- Restoring reconstructs and verifies committed state.
- Ready can admit/schedule work under policy.
- Running owns a bounded runtime invocation.
- Waiting has a recorded continuation or pending result.
- Quarantined exposes diagnostics and verified history while suspending execution.
- Unavailable reports missing resources, artifacts, authority or supported code.
- Detached holds an observed replica or has released execution placement.

A process restart reconstructs Active instances and resumes eligible work.
It MUST revalidate authority and placement before dispatching effects.
A renderer disappearing does not pause the underlying cell.

## pause and cancellation

Pause stops scheduling new application invocations. It reaches a runtime
checkpoint or abandons an uncommitted pure proposal. Committed acts and outcomes
remain tracked. Their dispatch after pause requires the pause policy explicitly
allowing that operation class; the baseline holds undispatched acts.

Cancellation targets a particular invocation or operation. Before dispatch it can
commit a definite cancellation. After dispatch it sends a cancellation request
to the executor and records the observed result. An unconfirmed external
cancellation remains unknown. Cancellation never rewrites a completed outcome.

Resume commits Active after checking checkpoints, grants, epoch and resources.
A revoked grant cannot be revived by loading an older snapshot.

## faults and retirement

A runtime fault discards uncommitted mutations and requested acts, then records
a fault event through a valid management transition. If recording is unavailable,
the host reports Unavailable and stops execution; it cannot acknowledge a
durable fault record.

Retirement first enters Retiring. Pending operations are completed, definitely
cancelled, or explicitly retained as unresolved history under the governing
policy. Subscriptions stop accepting new application work. Retired records
contain the final head and disposition of pending obligations.

Retention/garbage collection follows history.md. Retirement preserves identity
and the selected historical commitments. Removing local bytes changes replica
availability and does not erase remote history.

## admission under concurrency

Each instance has one authoritative transition at a given predecessor. A host
may compute independent proposals concurrently, but only a successful head
comparison/finality decision permits their state/effects to become authoritative.
Management and application work share this ordering. Receiving an event for an
incompatible lifecycle returns the current head and LifecycleConflict.
