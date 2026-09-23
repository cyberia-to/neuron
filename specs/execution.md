---
title: execution
tags: neuron, prog, soft3, spec
status: accepted
spec-version: "0.3"
---
# Execution

A neuron executes admitted progs through the existing Rune machine. Work pins
code, state revision, input, context and a finite allowance. Ward supplies current
host authority independently of those data. The exact persisted fields belong to
[data](data.md) and [runtime v1](runtime-v1.md); the interfaces below describe the
native adapter rather than a second VM or a hypothetical wire format.

## Runtime boundary

`RuntimePort` validates values/checkpoints, starts a `RuntimeInput`, and advances a
checkpoint with an optional recorded reply and bounded slice. `RuntimeInput`
contains source/state/event bytes, optional context particle and step limit.
`start` returns serializable checkpoint bytes. `step` returns:

| Result | Meaning |
|---|---|
| `Done { result, used }` | Candidate result/state; adoption still checks prog revision |
| `Yield { checkpoint, used }` | Runnable continuation after a bounded slice |
| `Act { tag, arguments, checkpoint, used }` | Suspend for an explicitly gated host act |
| `Event { tag, selector, checkpoint, used }` | Suspend for a matching recorded event |
| Error | Retained bounded failure; no successful state adoption |

`used` is cumulative runtime steps for this invocation, not the slice delta.
The engine checks monotonicity and that its increase fits the reserved slice.
A malformed result or accounting report fails the invocation. The adapter's
checkpoint/value validators and artifact limits run before publication.

Native invocation status is Running, Waiting, Joining, Completed, Failed,
Cancelled or Conflict. Queued means admitted runnable work; unknown means a
Waiting invocation has a recorded attempt without a definite outcome. These are
scheduler/operation projections, not additional persisted enum values.

A completed result becomes prog state only against its admitted revision.
Concurrent pure evaluation may proceed; one CAS orders authoritative publication.
A stale result is retained as Conflict. Waiting on a tool, child, event or model
never holds a mutable graph transaction. Read-only views return artifacts/prysm
output; a UI action is a separately admitted request with captured authority.

## Checkpoints and replay

A checkpoint binds the admitted source and invocation, runtime semantics,
application context, used steps and suspension point. The current Rune codec
serializes the machine and continuation stack. Host pointers, live grant handles
and secrets cannot become portable checkpoint state. Operation/event metadata
lives in the invocation record and determines which reply can resume it.

Resume uses retained code/context and the correlated outcome. Current authority
is rebound separately and may deny the next transition. Incompatible legacy
checkpoints remain inspectable and paused; an authorized migration must validate
the replacement before execution. A native stack address or reinference of an
old response cannot substitute for a checkpoint.

Time, randomness, file contents, model output and network results that influence
state enter as retained input or observations. Replay consumes those records.
It must never repeat a completed act, select a newer model implicitly or invent
new randomness to reconstruct an old commit. Content integrity establishes the
recorded bytes; external truth depends on their evidence and trust profile.

## Host operations

A suspended act receives an operation ID derived from its invocation and ordinal.
The record pins arguments, result contract and original context. A durable
attempt binds the operation, exact authority statement, executor and generation
before physical dispatch. [Worker dispatch](worker-dispatch.md) specifies the
one-use permit, durable dispatch claim and current grant at the handoff.

| Phase | Permitted action |
|---|---|
| Proposed | Validate allowed act and obtain current authorization |
| Attempt recorded | Claim dispatch once under the selected worker generation |
| Attempt unresolved | Report Unknown; obtain correlated observation/reconciliation |
| Outcome recorded | Consume the exact result/failure once into the continuation |
| Consumed | Return its retained receipt on exact duplicate delivery |

A crash after an attempt may precede or follow the external effect. Recovery
therefore cannot infer that it is safe to execute again. The native generic act
profile has no automatic retry. An adapter that supports reconciliation defines
its own exact lookup/idempotency scope, retention, destination and evidence;
[native actions](action-envelope.md) are one such separate contract. A timeout,
cancellation or transport disconnection never proves that an effect did not run.
Conflicting outcomes are rejected; exact retries return the retained result.

The current invocation has one pending operation. Detached effects require a
separate admitted invocation/owner and allowance. Completing a parent cannot
silently abandon unsettled children or an unknown effect. Observation and
cancellation use current management scope and retain attribution to the original
attempt, including after tool authority was revoked.

## Resources and scheduling

Admission reserves from the neuron's aggregate finite allowance. Before runtime
evaluation the engine persists a slice reservation, currently at most 1000 steps.
Successful accounting charges the actual delta. A failed or lost settlement
charges the reservation conservatively. Charged plus held never exceeds the
limit, including after restart, archive, cancellation and upgrade.

Child admission transfers an available parent allowance rather than reserving it
twice at the neuron root. Parent/child maps, depth and outstanding work are
bounded. Settlement returns only unused allowance and preserves actual charge.
Soma owns task strategies, join policy and schedule occurrence identity; the
engine owns bounded invocation progression and durable accounting. A future
cross-subject delegation protocol must record ownership and reservations at
both subjects without assuming a distributed atomic transaction.

CPU steps are distinct from model tokens, wall time, bytes, GPU work and money.
Each host adapter declares its additional limits. Current in-process native
adapters are trusted code; cooperative inference limits do not claim hard GPU
preemption. A console's process capabilities do not make it an agent tool grant.
Untrusted native execution requires an enforced process/device boundary.

Fair polling advances a persisted cursor and skips work that cannot run. Parked
progs retain references and require no resident VM or private model copy. Graph
reads, decoding, queues and output sizes have explicit bounds; the owning profile
must reject excess before admitting an effect or claiming durable success.

## Streaming

Task/invocation/operation/attempt identity correlates output across selection
changes. Terminal artifacts are durable before completion is reported. Current
local model token deltas are bounded transient presentation events; reconnect
reads the retained task result, rather than claiming token-by-token replay.
A profile advertising resumable streaming must retain acknowledged chunks or a
sealed manifest and advance its cursor only through durable content.
