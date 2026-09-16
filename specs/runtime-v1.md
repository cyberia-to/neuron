---
title: neuron runtime records and transitions
tags: neuron, execution, spec
status: accepted
spec-version: "0.3"
---
# Runtime v1

This contract replaces the standalone runtime-cell subject. It reuses the bounded
Rune machine, canonical data/artifact codecs and graph transaction port.
The subject is the existing native NeuronId; all other IDs below address data.
Identity-only and foreign observation APIs remain usable without this runtime.

## Subject and programs

The authoritative graph namespace is NeuronId. Its initial head commits an
activation record containing the subject, supported authority policy and initial
root snapshot. The head's commit particle is independent of the subject ID.
Subsequent commits bind subject, index, previous commit, before/after snapshots,
request/event, changed records and authorization evidence. CAS applies to the
whole subject root; a conflict publishes no success and dispatches no effect.

The root contains network, policy, binding epoch, total budget limit/charged/held,
maps of installed progs and invocations, imported origin mappings, and a fair
dispatch cursor. Network is pinned; actions through another binding are explicit.
The root is bounded and all maps reject duplicate/unsorted keys on decode.

Installing a prog creates a data ID derived from subject and installation nonce.
Its record pins source/code artifact, mutable state artifact and revision,
lifecycle, per-invocation step allowance, concurrency bound and allowed acts.
Two installations of the same code have separate state without separate keys.
Upgrade increments the prog revision and pins replacement code/state artifacts;
live invocations retain their admitted source and compatible continuation. Their
old state revision prevents silent adoption over the replacement state.

## Invocations and state conflicts

Admission binds subject, prog, caller nonce, input, context, authority epoch,
parent/causation and allowance. The request identity binds subject+nonce; the
record binds every admitted field. An exact retry returns its original receipt;
nonce reuse for another prog/input/context conflicts, including after completion.

Each invocation stores its admitted code, input and context, base state revision,
checkpoint, status, used/charged/reserved steps, effect ordinal, optional pending
operation, result/fault and parent/child allocation references. Persisted statuses
are Running, Waiting, Joining, Completed, Failed, Cancelled and Conflict. Queued
is runnable work awaiting a slice; Unknown is a Waiting operation with an
unresolved recorded attempt. Neither is a separate persisted status.

Independent invocations may execute bounded slices concurrently on compatible
workers. Authoritative commits serialize. Result adoption requires the same prog
state revision against which the invocation ran. A stale result is retained as a
conflict, never silently overwrites state and never repeats an external effect.
A caller may explicitly submit new pure work against the new state. Serial
submission remains the simple default for dependent mutations of one prog.

A waiting effect blocks that invocation; other programs/invocations continue.
Fair dispatch skips non-runnable jobs and advances its persisted cursor under
bounded work. Admission limits apply before allocation; terminal history can be
archived as graph records while request claims and outcome receipts remain resolvable.

## Budgets and children

Top-level admission holds its declared allowance from the neuron's finite budget.
Per-slice reservations are persisted before evaluation. Normal completion charges
actual work; a lost settlement charges the entire reservation on recovery.
Charged resources remain charged across restart, archive, cancel and upgrade.
No API resets total usage by recreating the same identity or replaying admission.

A child's allowance transfers from its parent's unspent, undelegated allowance;
it does not reserve the same budget again at the neuron level. Parent bookkeeping
retains delegated allowance while a child runs, and the child's actual charge
after it settles. Unused allowance returns to a live parent; detached obligations
require an explicit new owner and budget transition. A parent result waits for
required child obligations before completion. Soma owns join/strategy policy.

Total charged + held never exceeds the declared neuron budget. Checks use bounded,
checked integer arithmetic. Foreign costs and inference/tool units use declared
resource contracts; CPU steps are not a substitute for monetary accounting.

## Effects and authority

Every operation binds its subject/network/prog/invocation, ordinal and exact
arguments through the containing root and invocation. Attempt records pin the
binding epoch/policy, grant and executor. The native profile supplies its fixed
result codec and local finality/unknown-outcome rules; applications may impose
stronger result, retry and evidence contracts without inventing new wire fields.
An attempt and current ward decision are durable before dispatch. Runtime data
cannot grant authority; deny is the default. The host verifies supported key
ownership and ward/vault enforce scoped use independently of VM evaluation.

After a recorded attempt, a restart reports an unknown outcome until correlated
evidence resolves it. No automatic retry is inferred from a timeout. A matching
outcome retry is idempotent; conflicting value/attempt/operation is rejected.
Consumption is persisted even after the continuation clears its live outbox.
Cancellation does not discard an unknown attempted effect or unsettled children.

Paused programs may retain outcomes but do not start new work. Retirement affects
the prog and requires its obligations to be settled; it does not delete the neuron.
Revocation changes the binding epoch. Admitted old selections remain attributable
and readable but cannot dispatch under a newer epoch without an explicit permitted
rebind transition. Already dispatched attempts retain their original binding.

## Storage and profiles

Cybergraph/BBG store the root, immutable content and request claims in their shared
database. No new log or private canonical database is introduced. Local logical
heads remain separate from foculus SignalChain steps and consensus finality.
The local writer uses the database's exclusive owner and CAS. Multi-device readers
and workers do not gain independent commit/dispatch authority.

The new neuron schema suite is independent of immutable cell v1. Legacy artifact,
checkpoint, birth and operation bytes remain readable by the compatibility codec.
Legacy active state is translated to these records with explicit origin references;
the [migration contract](migration.md) defines activation and fencing.
