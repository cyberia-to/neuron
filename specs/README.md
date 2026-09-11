---
title: cell specification
tags: cell, soft3, spec
status: draft
spec-version: "0.2"
date: 2026-09-11
---
# cell specification 0.2

This is the complete candidate model for review. MUST, MUST NOT, SHOULD and MAY
express requirements on an implementation claiming this version. Every document
in this directory is normative except explicitly identified source-status notes.
Draft means review is pending; it describes no implementation as complete.

The model covers definition, birth, execution, authority, history, communication,
evidence, lifecycle, evolution, hosting, interfaces and conformance. The agent
composition profile adds cyb context, soma tasks/learning and comparative release
criteria while preserving application ownership. Ledger
economics, consensus algorithms and spectral division are supplied by their
own protocols, with explicit requirements at the cell boundary.

## reading order

| Contract | Defines |
|---|---|
| [Foundations](foundations.md) | Cyb/soft3 invariants, instant start, locality and evidence boundaries |
| [Model](model.md) | Entities, identities, state, records, ordering |
| [Data](data.md) | Canonical schema representation and identity derivation |
| [Lifecycle](lifecycle.md) | Installation, activation, suspension and retirement |
| [Execution](execution.md) | Runtime steps, acts, continuations, resource limits |
| [Authority](authority.md) | Host-bound grants, delegation and revocation |
| [History](history.md) | Cybergraph persistence, transactions, replay and retention |
| [Communication](communication.md) | Read/write/trade, delivery, cursors and failures |
| [Evidence](evidence.md) | Claims, trust anchors, finality and cell profiles |
| [Evolution](evolution.md) | Upgrade, relocation, backup, fork and division |
| [API](api.md) | Host operations, ports, results and errors |
| [Integration](integration.md) | Repository structure, owners, extraction and gaps |
| [Agent composition](agent.md) | Context, tasks, tools, learning, workspace, schedules and delivery |
| [Conformance](conformance.md) | Required behavioral and failure checks |
| [Evaluation](evaluation.md) | Full-agent parity inventory and measurable superiority gates |
| [Experimental local runtime](local-runtime.md) | Implemented subset, exact wire suite, limits and remaining baseline gates |

## governing decisions

1. Cyb's organ definitions are authoritative. Cell is its extension organ.
2. One cell repository serves the protocol ladder and reusable local hosting.
3. Definition, instance, replica and host have distinct identities/lifecycles.
4. Cybergraph records history; bbg/storage makes it durable; cyb log presents it.
5. Cell state is a projection of authenticated graph history. Its application
   data, continuations and artifacts are referenced particles.
6. The baseline uses existing per-neuron SignalChains through one append
   coordinator per writer. Cell-local commit ordering is an explicit graph
   overlay; independent domain consensus requires the profile's named protocol.
7. Authority is bound outside mutable runtime data and enforced by ward.
8. Runtime/native observations carry explicit evidence and assumptions.
9. Every consequential act follows a durable pending-act record and the required
   finality gate. Unknown external outcomes remain explicit.
10. Instance identity survives an authorized upgrade or relocation; an
    independent fork receives a new birth identity.
11. Local gates start through the existing host; stronger profiles add explicit
    gates. State transitions and graph access have bounded incremental costs.
12. Agent invocations pin context; soma owns cognition and evaluated learning.
    Durable children and consumed budgets survive task recovery.
13. Agent parity and superiority require complete capability mapping and
    reproducible outcomes beyond cell conformance.

The [0.2 review](../docs/foundations-agent-review.md) records the rationale.
This revision changes draft Event/Continuation/Query fields, RunStep resource
reporting and schema identity binding.
The experimental local implementation pins [schema suite 1](../model/schema-suite-v1.txt).
The generic /1 names remain aliases; a reader checks immutable manifest identity.
This local suite is not a registered network wire release.

## completeness and dependency status

These contracts select baseline behavior and rejection rules. Support for a
profile is advertised only when its runtime, storage, authorization, verification
and finality requirements pass conformance. Unsupported requirements produce a
typed error before activation or admission.

The [integration gaps](integration.md#required-upstream-work) identify missing
dependencies in the present stack. No alternate history service or substitute
proof/consensus implementation is implied. Wire registration and stack codec
compatibility must be verified before a network-compatible release.

A complete model specifies how extensions are admitted and what evidence they
must provide; each extension owns its domain algorithm and conformance suite.
