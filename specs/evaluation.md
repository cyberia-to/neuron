---
title: agent evaluation
tags: cell, soma, hermes, spec
status: draft
spec-version: "0.2"
---
# agent evaluation

An architecture comparison motivates work. A superiority claim requires a
working agent and reproducible comparative evidence. The following gates apply
to releases claiming the [agent composition profile](agent.md).

## three separate claims

| Gate | Evidence required |
|---|---|
| Cell conformance | Applicable C-series checks pass for every advertised combination |
| Agent parity | Complete reference capability inventory with supported, adapted or missing behavior and end-to-end checks |
| Agent superiority | Predeclared comparative quality, recovery and learning results with cost/latency and critical-regression controls |

Adapted behavior must accomplish the same user outcome and disclose migration
cost. Renamed or deliberately unsupported features remain visible. Full parity
requires closing every in-scope gap; exclusions require an explicit scope
decision and narrow the claim. A convenient subset cannot stand for full Hermes.

## reference and reproducibility

The design review inspected Hermes revision
d15ed4445207dda418b984e8bda0f68f48b8c6f3 on 2026-09-11.
Each implementation comparison pins its chosen reference revision anew. The
review's source map is in [the audit](../docs/foundations-agent-review.md).

Before measurements, publish a manifest containing both commits, configuration,
tool/adapter inventory, model/provider revision, prompts, runtime versions,
hardware, storage, network conditions, privacy grants and task corpus hash.
Record cold and warm starts separately. Use the same model, tool access and
token/time/cost ceilings for the primary matched comparison. Also report a
separate best-available-system comparison if optimizations differ.

The baseline receives its documented setup and supported durable/recovery
features. Alternative context plugins are declared, not silently excluded when
making claims about extensibility. Record tunings symmetrically. Freeze test
tasks after development; distinguish training, tuning and held-out sets.

## scenario families

| ID | User outcome and stress | Required observation |
|---|---|---|
| A01 | Fresh local ask and source-gate extension | Time to useful result, setup actions, artifacts and authority correctness |
| A02 | Long coding/research task across compaction | Constraint preservation, source recall, correctness and actual task completion |
| A03 | Restart at every tool/commit boundary | Lost acknowledged work, duplicate effects, explicit uncertainty and recovery effort |
| A04 | Parallel delegated task across parent/child restart | Correct join, child ownership, cancellation and bounded total exposure |
| A05 | Resume on another compatible machine | Same task/context, complete artifacts, fencing and credential rebinding |
| A06 | Learn a skill, then solve held-out related tasks | Improvement over no-learning ablation, failures, provenance and contamination checks |
| A07 | Malicious tool output or imported skill | Task continuation within authority, data-disclosure boundaries and useful completion |
| A08 | Concurrent file edits and rollback | User edits preserved, conflicts visible and intended workspace result recovered |
| A09 | Scheduled work during provider/channel failure | Correct catch-up, one logical occurrence and delivery retry without task rerun |
| A10 | Provider/model change and offline operation | Preserved constraints, disclosure policy, budget and honest capability degradation |
| A11 | Many idle cells and long retained history | Memory, update cost, startup/recovery latency and bounded query behavior |
| A12 | CLI, cyb, messenger and media workflow | Equivalent task state, correct artifacts, reconnection and privacy |

Each family expands into executable cases with deterministic outcome oracles
where possible. Human quality evaluation is blind to system identity; model
judges declare their rubric and receive human calibration. Fault tests inject
the same failure points and accepted external outcomes into both systems.
Report unsupported scenarios as such, without inventing reference behavior.

## measurement and decision rules

Predeclare primary metrics, sample size/repetitions, seeds, success thresholds,
non-inferiority margins, critical failures and statistical method before running
the frozen corpus. Publish all attempts, timeouts, crashes and setup failures in
the denominator. Aggregate at task level, with paired confidence intervals and
correction for multiple primary comparisons. Insufficient power is inconclusive.

Measure authorized successful outcomes first, then elapsed/p95 latency, tokens,
provider charges, CPU/GPU time, peak/idle memory, graph I/O, storage growth and
recovery/user intervention. Energy measurements identify hardware/meters and
error; missing energy data is reported as unavailable. Successful tasks per
unit cost/energy complement raw throughput.

A superiority report MUST show its predeclared meaningful gain and confidence
bound in each area claimed, satisfy its non-inferiority bounds for ordinary
daily tasks and publish all critical failures. Unauthorized effects, false
success/durability or silent data loss block release. Absence in a finite suite
is evidence for that suite, rather than a universal safety proof.

Broadly claiming to surpass Hermes requires parity plus demonstrated gains in
task quality, durable recovery/portability and transferable learning under the
declared budget. A win restricted to restart handling is reported at that scope.
Rust, a graph store, proof vocabulary or feature counts establish no such win.

## evidence artifact

Publish machine-readable case outcomes, capability inventory, frozen manifests,
reproduction commands and scoped traces with the report. Private fixtures may
have redacted public equivalents; disclose what independent reviewers cannot
reproduce. Raw traces retain context, attempt and result identities so failures
can be investigated through the same graph history.

Current status: specification only. No comparative benchmark has run and no
parity, quality, performance or superiority result is asserted.
