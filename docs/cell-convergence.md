---
title: neuron convergence
tags: neuron, prog, cyb, soft3
date: 2026-09-13
status: accepted model
---
# One subject, many programs

Neuron is the protocol subject. Its identity binds authored actions to the
selected key and identity profile. A robot attaches neurons for different keys,
networks and devices. Those attachments express the useful decomposition the
user identified: one named robot can work through several independently owned
or observed identities.

The useful part of the former cell runtime is now an execution capability of
that subject: installed programs, persistent state, bounded invocations,
continuations, effect attempts and recovery. `ProgId`, invocation, operation and
checkpoint particles identify data and work. They acquire no key or independent
signing identity. Creating or retiring a prog leaves the neuron's identity intact.

## Why this is the smaller model

Two installations can have independent program state and lifecycle under one
neuron. A suspended tool invocation can coexist with an unrelated runnable job.
These require separate program and job records, not a second subject hierarchy.
Jobs share the neuron's finite budget and current host authority. Concurrent
results use a program revision check: a stale result is retained as a conflict
instead of overwriting newer state.

A second neuron is appropriate when authorship or control actually differs:
another key, a qualified foreign account, a separately attributed service, or an
independent governance participant. A device is an execution placement; moving
between permitted devices need not create a new subject. A node process can
observe many neurons through one shared GraphSession. Its storage does not imply
custody over the subjects whose history it contains.

| Concrete situation | Representation |
|---|---|
| One robot, Bostrom and native accounts | Two explicitly attached qualified subjects/networks; existing address/signing profiles retained |
| One subject, counter plus agent task | Two progs and their invocations; one neuron and aggregate resource account |
| Several model/tool steps in a task | Task/child/invocation data with captured subject, network and authority |
| A viewed shard with many authors | Shared graph/session and a selected shard; authors retain their own identities |
| Building application or organization | Service/view plus its progs and explicitly configured governance subjects |
| Asset ledger | Book, issuer and settlement/bridge policy, with the existing conservation/finality duties |
| UI page or plugin view | Typed destination and presentation state; navigation creates no account |

## Ownership through the stack

The minimal `neuron-id` crate carries the common native 32-byte identifier. The
model exposes native and domain/network-qualified foreign references without
pulling VM or GUI dependencies into identity consumers. Existing native identity
remains H(compressed public key). Foreign bytes keep their own derivation and
signature contracts.

Neuron-engine coordinates state transitions through graph, runtime and authority
ports. Rune owns evaluation and continuation mechanics; warriors/workers bind
the VM family, deployment profile and device placement. Ward decisions come
from current host grants and vault custody. A program manifest requests acts;
its contents cannot authorize their execution. A command captures its attachment
and network before asynchronous work, and revocation is checked at publication
and dispatch boundaries.

Cybergraph owns history/write interfaces; BBG owns the durable representation.
The Log organ renders that history. GraphSession is the multi-author graph owner
used by CLI and shell. Soma composes the neuron execution model into durable
agent tasks, model/tool turns, children, schedules and user controls. Prysm
renders typed destinations and correlated results. It owns no runtime signer.

## Migration preserves the original subject claims

The former Cell v1 birth hash identifies a legacy origin, rather than a native
key identity. Multiple origins map explicitly to distinct prog installations of
one authenticated target neuron. Original content hashes, schema manifests,
history, completed request claims, charges, held resources, continuations and
unknown attempts remain available with provenance. Normal old writers are
fenced when cutover becomes durable. An unknown attempt is reconciled with
evidence; importing or restarting never automatically repeats its side effect.

The [operator guide](legacy-cutover.md) covers same-store import, bounded export,
sealed inspection and resumable transfer. The complete contracts are in
[specs](../specs/README.md); actual executed evidence belongs to the
[implementation ledger](../../soft3/audit/neuron-cell/implementation.md).

## Agent scope

The local Soma composition executes durable tasks through the same program,
authority and history owners used by the standalone runtime. It includes real
local model inference, supported tool adapters, cancellation, steering, model
and context changes, parent/child joins and stable schedule occurrences.
Provider/channel/tool breadth and measured Hermes parity are separate capability
gates in [agent](../specs/agent.md) and [evaluation](../specs/evaluation.md).
Runtime convergence establishes their execution foundation; a rename alone
would not establish a functional agent.

The [original cell proposal](../audit/legacy-cell-convergence.md) and
[foundations review](../audit/legacy-foundations-agent-review.md) are preserved as
dated evidence. The current [domain ladder](../../cyber/specs/domain-ladder.md)
replaces the earlier attempt to use cell for runtime, host, shard, ledger and
building roles. Biological cells, Noun::Cell, memory/table cells and immutable
legacy wire tags retain their established meanings.
