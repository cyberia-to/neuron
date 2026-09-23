---
title: neuron foundations
tags: neuron, cyb, soft3, spec
status: accepted
spec-version: "0.3"
---
# Foundations

Neuron joins protocol identity with optional execution of durable progs. It does
not add an identity between the named robot and its attached subjects. Independent
state, continuation, deployment or task cancellation alone needs no new key.

[Cyb anatomy](../../cyb/anatomy.md) governs organ meanings and
[cyb architecture](../../cyb/specs/architecture.md) fixes subject composition.
[Soft3 execution](../../soft3/specs/execution-model.md) governs machine,
environment, proof profile, network and executor selection. Data/terms belong to
[soft3](../../soft3/specs/terms.md); algorithms and cryptography stay with their
owners. An older document cannot silently restore a birth-based runtime subject.

| Foundation | Required behavior |
|---|---|
| One graph | State, context, transitions and provenance are graph-addressable; BBG provides durability and Log presents history |
| One subject | Actions bind an explicit domain-qualified neuron; programs and placements are data/work |
| Composable data | Existing atom/pair/particle semantics, immutable suites and exact bytes |
| Local autonomy | Supported behavior runs with locally retained dependencies and no mandatory network |
| Immediate entry | A source gate uses the existing evaluator/host; no extra service/account per call |
| User authority | Current ward check at publication/dispatch; vault never gives keys to runtime artifacts |
| Captured context | Agent work pins soul/model/workspace/source revisions; steering is an explicit successor |
| Visible evidence | Observation, valid computation, source provenance and network finality are separate claims |
| Bounded locality | Finite queues, graph access, evaluation slices and declared adapter limits |
| Retained resources | Spent/held allowances survive restart, archive and child joins |
| Continuous lifetime | A surface, task, prog, attachment and device have distinct lifecycles |
| Small foundation | Identity-only consumers do not import execution/storage/GUI/inference |

## Source entry and packaging

Rust composes hosts; Rune supplies loaded behavior. Command, surface, processor,
resolver and companion are trigger conventions, not five identities or VMs.
A local pure read/view uses a fixed snapshot without persistent installation.
Stateful work installs a prog under an explicitly controlled neuron; it never
creates custody implicitly. Create/attach may be a simple user action, but the
host must retain its chosen subject/network and cannot borrow later UI selection.

A supported local source needs no new repository, NFT, network publication,
remote consensus, extra daemon or ahead-of-time build. Missing runtimes/artifacts
return availability errors. Optimization and proof generation may be separate
stages; an effect requiring a proof or finality waits at that declared gate.

GUI and headless bodies use the same owner APIs. Local models/resources remain
explicit capabilities; loss of one dependency blocks dependent work, while other
admitted programs may continue. Untrusted source is bounded by its evaluator;
a trusted native console/tool adapter must state that it lacks hard preemption
rather than advertise VM sandbox guarantees for arbitrary host code.

## Computation, knowledge and resources

An execution proof binds program/input/witness semantics. State verification binds
an external claim to a chosen root/tier. Provenance identifies a source. Recording
a model answer or tool receipt proves neither its truth nor an economic judgment.
Conviction and monetary effects require their actual sigma/ward/network contract;
runtime bookkeeping is not evidence of universal consensus.

Graph retrieval, summaries, skills and model revisions are application policy.
Connectivity does not guarantee optimal model context; cross-model hidden-state
reuse requires a validated mathematical/codec contract. Neuron carries artifacts
without making speculative model algorithms a baseline dependency.

VM steps, wall time, memory, provider tokens, billing and physical energy have
different units and enforcement. Shared child allowances are transferred, never
minted on retry. Unknown external usage/results retain reservations or explicit
unresolved state. A cooperative limit is identified as such. Body owns placement
and telemetry, sigma owns asset accounting, and Soma owns model/tool strategy.

## Domain and repository boundaries

Organs name responsibilities; their count is not a count of repositories,
processes or signing subjects. Runtime and graph libraries remain reusable without
Bevy. A service can run several progs; its governance may authorize a neuron.
A ledger/book and a graph shard retain their protocol rules, independently of
whether a neuron executes maintenance work for them. Neither becomes a runtime
subject merely by holding state.

The [integration map](integration.md) assigns owners. Full daily-agent and
comparative Hermes claims additionally require [agent](agent.md) and
[evaluation](evaluation.md) evidence; passing a counter/runtime suite alone is
insufficient. Architecture adoption is not a measurement or deployment report.
