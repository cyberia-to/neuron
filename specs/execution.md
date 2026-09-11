---
title: execution
tags: cell, soft3, spec
status: draft
spec-version: "0.1"
---
# execution

A runtime evaluates a pinned definition against a pinned Snapshot and explicit
input. The host supplies a separate authority context and finite resource budget.

## runtime interface

The model interface has three operations:

~~~text
start(definition, snapshot, admitted_event, context, budget) → RunStep
resume(definition, checkpoint, recorded_input, context, budget) → RunStep
inspect(definition) → RuntimeSupport
~~~

RunStep is one of:

- Complete(application_state, result, view_artifacts, requested_operations, witnesses).
- AwaitAct(checkpoint, requested_operations, witnesses).
- AwaitEvent(checkpoint, subscription_request, witnesses).
- Yield(checkpoint, used_resources).
- Fault(code, diagnostic_artifact, used_resources).

Each successful step proposes an updated Snapshot and a commit. The engine
validates schema, quota, authority requirements and predecessor before publishing.
Uncommitted requested operations are inert. A continuation resumes only with
input matching its persisted selector and expected schema.

Admission records InboxEntry(admitted) once for the original EventId. Subsequent
commits may reference that Event as causation without admitting it again.
Suspension/resumption changes its existing invocation state; completion records
InboxEntry(completed) and the result artifact matching the entry's output schema.
The host arbitrates resumption so a retained continuation has one successful
consumption transition even if several matching deliveries race.

Complete ends the current invocation. Requested operations may remain pending
only when the entry's governing policy explicitly permits detached work and
provides a result-handler event entry. They retain their OperationId, authority,
budget and outcomes. Otherwise the invocation must AwaitAct until its dependent
operations resolve. A Completed invocation receipt reports its result and any
explicitly detached operation references; it never implies those effects finished.

Read/view entries receive immutable snapshots and a restricted authority context.
Their outputs are artifacts or prysm chunks. They cannot issue a state mutation
or consequential external act. A user interaction with a view creates an Event.

## checkpoints

Continuation = (definition, runtime, checkpoint_schema, artifact, trigger,
base_head, invocation, resources_used).

trigger is a named variant: operation_result(OperationId), event(selector
particle), or scheduler_yield. invocation is the originating EventId.
base_head identifies the state at which the checkpoint was produced; later
management/outcome records may advance the head without changing that checkpoint.
The engine verifies that the current Snapshot still contains the continuation
before resuming it.

The artifact serializes everything required by the adapter to resume, excluding
raw host pointers, runtime-local grant handles and secret material.
Epoch/grants are rebound from current policy. Checkpoint schema and runtime
semantics must match or have an authorized migration.

A runtime can advertise complete-entry-only execution if each invocation is
bounded and restartable before commit. Such a runtime cannot satisfy definitions
requiring durable mid-entry suspension. A native stack address is never a
portable checkpoint.

## operations

Operation = (id, invocation, target, act, arguments, required_authority,
result_schema, deadline, retry, finality_gate, checkpoint).

target identifies a cell, a surface or an executor using a named variant.
act and required_authority are protocol/schema particles; arguments is an
artifact/particle reference; deadline is optional TimePoint.
checkpoint is an optional Continuation reference. finality_gate comes from
the governing profile and may be strengthened by this operation.

retry is one of:
- never;
- deduplicated(executor_contract, maximum_attempts);
- reconcile(executor_contract, maximum_attempts).

An executor contract defines idempotency scope and retention horizon, query by
operation identity, cancellation and result evidence. A generic retry promise
without those semantics fails admission.

Attempt = (id, operation, number, epoch, authority_decision, dispatch_time,
executor_contract).

Outcome = (operation, attempt, disposition, value, evidence, observed_at).
disposition is succeeded, failed_definite, denied, cancelled_definite or unknown.
value and evidence are optional particles; a successful result must match
result_schema. An unknown outcome may later be resolved by a correlated Outcome.
Conflicting terminal outcomes are retained and trigger reconciliation.

## operation progression

| Stage | Persisted fact and permitted next action |
|---|---|
| Pending | Proposed act and continuation committed; authorization may be sought |
| AwaitingGrant | Ward decision pending; effect remains undispatched |
| Authorized | Decision bound to release, arguments, epoch and policy |
| AttemptRecorded | Attempt identity durable; dispatch may happen after rechecking authority |
| Dispatched | Execution observed started; result or reconciliation awaited |
| Resolved | Definite outcome committed; continuation may consume it once |
| Unknown | Result cannot be determined; reconciliation policy applies |

Stages are projections of graph records. A crash after AttemptRecorded can occur
before or after actual dispatch, so recovery treats such an attempt as potentially
executed. It queries the executor or applies the declared idempotency contract.
The same OperationId is used for a deduplicated retry. Blind retry of an
unreconcilable consequential operation is forbidden.

A result can be consumed once per recorded continuation. Re-delivery returns the
existing outcome/consumption receipt. Persisting an outcome and scheduling its
consumer follows the history transaction boundary. Cancellation and deadlines
have the same unknown-outcome rule.

## determinism and observations

Time, randomness, network replies, model outputs, filesystem reads and user input
that affect a committed transition are explicit events/witnesses. Pure replay
consumes recorded values. It MUST NOT repeat completed external acts or draw new
randomness to reproduce an old commit.

A deterministic adapter declares its exact runtime/code semantics. Native or
model-host computation may be trusted under the profile; its evidence identifies
the trusted host and assumptions. Recording a value alone proves no truth about
the external world.

Synchronous rune Host calls must enter the same operation/receipt discipline.
An adapter either yields at the boundary or durably records and resumes the
operation through the host protocol. Returning a fake successful value when an
act is unsupported is forbidden.

## budgets, streaming and scheduling

The host intersects caller, definition, delegated and machine budgets.
Compute/memory limits are enforced by the runtime/placement mechanism before
unbounded execution is admitted. Budget exhaustion yields a valid checkpoint or
records a bounded fault. Recursive cell calls consume a delegated budget with
a bounded call depth; waiting never holds another cell's write lock.

Scheduling is fair among admitted instances according to host policy. Plan owns
future schedules; cell schedules execution of already admitted work.
Timers arrive as events from a clock provider with declared semantics.

Streaming output has stable invocation identity and monotonically numbered
chunks. A chunk advertised as resumable is durably recorded or included in a
durable artifact manifest before acknowledgement. Transient view deltas carry
an explicit transient flag and may be reconstructed or lost. Terminal result
artifacts are sealed and referenced by the completion commit.
Chunk batching is allowed; the advertised cursor never passes durable data.
