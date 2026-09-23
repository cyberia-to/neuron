---
title: neuron specification
tags: neuron, soft3, spec
status: accepted
spec-version: "0.3"
---
# Neuron specification

Neuron is the protocol subject with its own profile-defined identity and optional
durable program execution. One named robot attaches several neurons for its keys,
networks and devices. One neuron can execute multiple progs and tasks. Their data
IDs and lifecycles add no second signing subject. Identity-only/watch-only use
requires no running VM. The specification adopts the approved
[convergence decision](../../soft3/roadmap/neuron-cell-convergence.md).

MUST/MUST NOT define requirements for a profile claiming this contract. Accepted
architecture does not make every runtime, proof backend, remote executor or agent
feature implemented. Each profile declares the actual schemas, authority,
resources, storage, evidence and failure behavior it supports. Observed results
belong in [audit](../../soft3/audit/neuron-cell/implementation.md).

| Contract | Defines |
|---|---|
| [Foundations](foundations.md) | Cyb meanings, ownership, locality and immediate execution |
| [Identity](identity.md) | Native/foreign subjects, bindings, key and device changes |
| [Model](model.md) | Neuron/prog/invocation records and distinct orderings |
| [Data](data.md) | Canonical values, exact suites, IDs and bounded decoding |
| [Runtime v1](runtime-v1.md) | Implemented multi-prog/task transitions and resource accounting |
| [Lifecycle](lifecycle.md) | Program management, task cancellation and host placement |
| [Execution](execution.md) | Runtime/checkpoint/act/observation contracts |
| [Authority](authority.md) | Current host-bound grants and independent publication verification |
| [Local authority](local-authority.md) | Existing H(pubkey) and exact NSIG1 profile |
| [Worker dispatch](worker-dispatch.md) | Existing worker selection, generations and guarded handoff |
| [History](history.md) | Shared graph/BBG persistence, receipts, closure and retention |
| [Communication](communication.md) | Typed delivery, read/write/trade, durable cursors |
| [Action envelope](action-envelope.md) | Signed native external action/receipt boundary |
| [Evidence](evidence.md) | Observation, execution proof and network finality distinctions |
| [Evolution](evolution.md) | Upgrade, relocation, backup and explicit forks |
| [Migration](migration.md) | Preserved old origins, resumable import and fencing |
| [API](api.md), [CLI](cli.md) | Ports, operators and headless composition |
| [Navigation](navigation.md) | Subject/prog/view/particle references without side effects |
| [Query projection](query-projection.md) | Immutable execution relations owned here, read by inf |
| [Integration](integration.md) | Crate DAG, owners, product and domain profiles |
| [Agent](agent.md), [evaluation](evaluation.md) | Full-agent requirements and measurable comparative gates |
| [Conformance](conformance.md) | Behavioral, authority and persistence-failure checks |
| [Legacy local runtime](local-runtime.md) | Retained original wire/reader compatibility |

Cybergraph owns history, BBG supplies durability, and cyb Log renders it. Foculus
owns signal ordering/finality; neuron runtime commits are a separate application
projection over the same storage. Ward authorizes and vault manages secrets.
Rune/nox execute; workers place computation. Soma owns cognition, goals and
learning; progs are its execution mechanism, not a new identity hierarchy.

The old birth-based subject exists only as a legacy origin in migration records
and immutable old codecs. Neither an activation particle nor a prog/checkpoint
hash is a NeuronId. Existing keys, addresses, signatures, claims and historical
record bytes survive convergence; unsupported continuation/evidence profiles
remain inspectable and suspended rather than silently fabricated or rerun.
