---
title: cell foundations
tags: cell, cyb, soft3, spec
status: draft
spec-version: "0.2"
---
# foundations

Cell gives a loaded ability explicit state, authority, lifetime and causal
history. One implementation serves cyb's extension organ and local hosting;
protocol profiles add their own admission, economics and finality requirements.
Convergence shares the model while preserving each profile's actual guarantees.

## authority of definitions

[Cyb anatomy](../../cyb/anatomy.md) governs organ meanings. The user's decision
assigns authoritative history to cybergraph, persistence to bbg/storage and
presentation to log. Older robot, soma and scripting descriptions are adapted
to these meanings. Implementation findings document present gaps.

[Soft3 types](../../soft3/specs/types.md) and
[terms](../../soft3/specs/terms.md) govern data and composition.
[Cyb philosophy](../../cyb/product/philosophy.md) and
[product specification](../../cyb/product/spec.md) govern product constraints.
Each substrate owns its algorithms, cryptography and protocol versions.

## required properties

| Foundation | Cell obligation |
|---|---|
| One graph | State, context, transitions and provenance are graph-addressable; indexes are projections |
| Composable data | Schemas use existing data/pair/particle semantics and immutable manifest identity |
| Local autonomy | One local host runs supported abilities with locally retained dependencies |
| Instant start | A source gate enters its evaluator through the existing host immediately |
| User authority | Ward checks each effect; vault operations preserve secret ownership |
| Contextual action | Agent invocations bind now, soul and relevant graph/workspace versions |
| Evidence visibility | Receipts distinguish observation, execution evidence, finality and epistemic commitment |
| Bounded locality | Each step declares finite graph access and execution budgets |
| Resource economy | Inactive instances park; shared artifacts and incremental state avoid repeated work |
| Continuous lifetime | View attachment, task lifetime and host placement have independent transitions |
| Small stable foundation | Application policies compose through versioned data and narrow ports |

## instant start and packaging

The Rust host composes the stack; rune supplies loadable behavior and policy.
Command, surface, processor, resolver and companion are trigger conventions over
the same gate, as in [scripting](../../cyb/reference/scripting.md). They do not
require separate schedulers, histories or runtime implementations.

A host MUST accept a local source gate with a derived minimal Definition and
bounded default authority. Its normal local path requires no new repository,
network publication, NFT, remote consensus, separate service installation or
ahead-of-time compilation. Locally absent runtime/artifacts produce explicit
availability errors. Reusing an already installed host is the normal path.

A pure read/view may execute against a retained host snapshot without creating a
new persistent instance for every call. Stateful abilities obtain Birth and
history automatically through the same local host. Consequential acts obey
durable history and ward gates. A local profile uses local persistence/finality;
stronger profile requirements are selected explicitly.

Parsing and lowering proceed directly into evaluation under
[rune instant start](../../rune/specs/execution.md). Optimization and optional
proof generation can follow in the background. An operation needing a proof or
network finality waits at its declared gate; that gate defines its guarantee.

Headless and cyb consumers use the same library contracts and local graph store.
A normal local installation SHOULD provide one executable with embedded local
hosting. Extra runtimes, models and transports are declared capabilities.
Loss of a remote dependency blocks dependent work while independent local work
continues within its policy. A preinstalled local model supports private offline
agent operation; quality and hardware requirements remain measurable properties.

## computation and knowledge

Runtime proofs bind program execution under stated witnesses. State verification
binds the external claim and its tier. Source provenance binds where a statement
came from. These distinctions remain visible in graph data and presentation.

Routine structural/history links use exploratory void valence under the selected
graph protocol. Recording a tool result or learned skill conveys no automatic
true/false conviction. Economic commitment requires ward authorization and sigma
budget under [cyb truth](../../cyb/product/truth.md).

Graph retrieval, learned summaries and compiled models are implementations of
application policy. Graph connectivity alone establishes no general guarantee
of optimal LLM context. Cross-model hidden-state reuse requires a declared
compatible mathematical/runtime contract and validation. The CT-0 compiler's
own conformance predicates remain in [tru](../../tru/specs/ct0.md).
Cell may carry their artifacts and evidence without claiming arbitrary model
answers are correct or making speculative algorithms a baseline dependency.

## repository and organ boundaries

The 21 organs name responsibilities, not mandatory process or instance counts.
Standalone repositories follow reusable contracts, dependency direction and
independent maintenance needs. One soma task may contain many invocations inside
one cell; an independently authorized service may need another cell.

Cell model/engine remains usable without Bevy, a model provider or consensus
node. The [integration map](integration.md) assigns concrete owners. Agent
behavior is specified as a composition in [agent](agent.md), with measurable
release criteria in [evaluation](evaluation.md).
