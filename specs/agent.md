---
title: agent composition profile
tags: neuron, soma, cyb, spec
status: accepted
spec-version: "0.3"
---
# agent composition profile

This application profile defines the cyb agent requirements for a usable
successor to Hermes. It builds on neuron/prog admission and execution contracts;
a subject that only links or controls a book need not implement cognition.

Soma owns cognition, task strategy, context selection and learning. Neuron execution supplies
admission, durable execution, causal identity and recovery. Application schemas
are owned by their organs and pinned in an agent release manifest; names below
are required semantic roles, rather than new neuron kernel types. A release cannot
claim this profile until those schemas and executable checks exist.

## application context

Every agent command Event has a context particle whose application schema binds:

| Binding | Required meaning |
|---|---|
| now | The particle at which the command acts; selected by now/com |
| soul | Immutable configuration version used to form this invocation |
| lineage | Task/goal, parent invocation and conversation references when applicable |
| sources | Relevant graph scopes and their actual selected heads |
| workspace | Root scope and observed revision/artifact manifest, if files are involved |
| constraints | Explicit user requirements and declared budget references |
| disclosure | Data scopes and intended recipients, evaluated through ward |

Fields unrelated to the command are explicitly absent. The manifest is frozen
at admission. Lazy context expansion records additional source heads and a
successor manifest before those inputs influence a committed step. Private
metadata inherits the same access scope as its source.

The UI may change now while a task runs. The task retains its own binding.
New user steering is an authenticated input applied at an explicit safe boundary;
its accepted constraints and successor context remain in history. Soul changes
affect new asks. Applying them to running work requires such a transition.
Ward still applies current revocations immediately at its enforcement boundary.

The now organ is semantic context. Rune's ~now slot remains clock input;
~here/~world and the event payload carry the relevant graph context through the
adapter. Context data and host-held authority_context have distinct identities.

Soma records a context-build manifest: exact selected source particles, ordering,
transform/version, summaries, omissions, model/tokenizer and available window.
Originals needed for pending work remain available under declared retention.
Compression pins user constraints, unresolved effects, child obligations and
their references. Summaries can expand to their sources when those are retained;
unavailable sources are labelled. Retrieval and compression consume budgets.

Static context bundles such as [ctx](../../ctx/README.md) can seed this process.
Their generator version, source revisions, selection method and omissions are
recorded. A packed text export is derived input; current anatomy and explicit
user steering determine organ meanings when an older bundle disagrees.

## tasks, concurrency and interaction

Soma's task schema MUST represent accepted, running, waiting, suspended,
completed, failed and cancelled work, plus outcome uncertainty where applicable.
Neuron lifecycle remains independent: one subject can own multiple progs and
concurrent waiting invocations. Task and prog IDs are data, not signing identities.
A session/conversation is a presentation and input scope; task ownership persists
when a terminal disconnects, a view closes or a model changes.

Durable task records bind user intent, success criteria, inputs, responsible
neuron and prog, current context, plan revision, pending work and terminal artifacts.
Plans are proposals until the responsible policy admits their steps. Cancellation
stops new dispatch, tracks accepted external work and preserves its outcomes.
Steering and progress are correlated with the stable task/invocation identity.

Delegation is bounded concurrent work with a persisted parent-child relation,
input/context disclosure, grant subset, budget reservation and join policy.
Required join policies are all-required, first-success and best-effort with a
declared completion boundary. First-success records the selected result and
cancels or explicitly detaches remaining children under governing policy.

Child admission, progress, terminal result and parent consumption survive host
restart. Parent completion settles each child's obligation or explicitly detaches
it to a named owner. A process-local callback is insufficient. Cross-neuron joins
use messages and receipts; they assume no atomic transaction across peers.
Child results are input under their provenance and authority, never direct writes
to parent state. Parent adoption validates current task/workspace preconditions.

Neuron execution serializes authoritative subject transitions while model/tool
work runs concurrently within quotas. Independent state uses separate progs;
independent work uses invocations/tasks. Placement on another worker or device
alone creates no identity. A different signing authority/network profile requires
an explicit neuron binding; cross-subject work uses messages and receipts.
Task schemas remain in Soma.

## inference and tool execution

A release claiming full daily-agent parity supports local inference and declared
remote provider adapters.
Soma selects models; glia/runtime adapters execute; body places resources; vault
supplies credential operations. Each request pins provider/model identity,
parameters, context manifest, tool schemas and adapter version. Opaque provider
model revisions are reported as such; exact reproducibility is not inferred.

Capability negotiation covers tools, structured output, streaming, context limits
and media. Failover records a new attempt/configuration and preserves task intent,
constraints, disclosure policy and budget. Moving private data to an external
provider requires the corresponding ward grant. An ambiguous accepted request
follows execution.md's uncertainty rules, including possible duplicate billing.
Streaming partial text is distinguishable from a completed response artifact.

Tool discovery is bounded and schema-driven. Tools advertise argument/result
schemas, authority scopes, executor identity, effect/idempotency semantics and
resource estimates. Discovery or a returned instruction cannot expand authority.
Adapters preserve those contracts for local tools, external protocols and dynamic
rune abilities. Unsupported semantics fail before the dependent operation.

The daily-agent capability inventory MUST cover terminal/process and file tools,
web/browser access, tool protocols including MCP, external agent integration
including ACP, search/retrieval, model/provider selection, configuration, skills,
scheduled/background work, CLI and messenger use, plus media capabilities.
Each has concrete adapters, tested operations and explicit limitations in the
release manifest. Missing adapters remain parity gaps; names alone prove nothing.

## workspace and artifacts

Memory presents graph particles as files. Files on an external filesystem use
versioned observations through an adapter; their authority and consistency
boundary is explicit. Task artifacts include source revision, resulting revision,
tool/attempt identity and test/evaluation results where applicable.

A write/patch checks its declared base content and path scope at execution.
Conflicting user edits produce reconciliation, preserving both versions.
Multi-file atomicity is claimed only with an executor implementing that contract.
Otherwise partial application is recorded with a recovery manifest.

Workspace restoration is an authorized compensating operation with expected
current revisions. It preserves unrelated user edits by default and records any
unresolved conflict. An invocation checkpoint restores execution state; it cannot rewind
an arbitrary filesystem, sent message or remote transaction. External effects
need their own cancellation/reconciliation/compensation contracts.

## memory and learning

Soma publishes retrievable episodes and skill candidates as graph particles with
source task, context, observations, attempts, outcomes and evaluation evidence.
Instructional skills are versioned data; executable abilities are pinned prog code releases.
Both specify prerequisites, scope, dependencies and provenance.

Candidate, evaluated, active, superseded and rejected states are explicit under
soma's promotion policy. A candidate becomes active only after the configured
evaluation and authority gates. Evaluation records successful and failed cases,
reference versions and criteria; self-description by the model is insufficient.
Promotion may be automatic within existing authorization. Code/permission changes
use the corresponding neuron/ward contracts. A skill cannot grant itself rights.

Retrieval returns supporting sources, scope, freshness and contradictory evidence
when known. User corrections become attributable revisions; obsolete candidates
remain traceable under retention policy. Memory/brain/log/time present the same
graph. Search indexes and model caches are disposable derived accelerators.

Reuse across tasks or robots requires compatible context, dependency versions,
disclosure permission and recipient evaluation. Shared bytes convey no automatic
trust or authority. Improvement is measured on held-out tasks under evaluation.md.

## standing orders and delivery

Plan owns schedule/timezone/catch-up semantics and produces a stable trigger
identity for each intended occurrence. Neuron execution admits it idempotently; soma performs
the work. Sense owns recipient/channel delivery, with radio providing transport.

Trigger, task execution and delivery each retain their identity and status.
Retrying failed delivery reuses the completed artifact; it MUST NOT rerun the
task merely to resend its result. Delivery acceptance/acknowledgement and unknown
external outcomes follow the channel executor contract. A reconnect resumes the
appropriate cursor. Persistent local scheduling works without an external cron
service, subject to host uptime and plan's declared catch-up policy.

## budgets and observability

Task budgets cover elapsed deadline, runtime/graph resources, model tokens,
provider charges and child allocations. Reservation precedes dispatch; usage and
uncertainty survive restart. Aggregate child exposure fits the parent's remaining
budget. Hard provider cost ceilings require enforceable provider/prepaid limits;
otherwise the release declares its reservation assumptions and possible overrun.
Sigma owns monetary accounting, body physical resources, ward enforcement and
soma task allocation. Energy reports include the measurement method.

Com/sense show accepted work, progress, needed decisions, unresolved effects and
terminal results. Log/time can explain the causal path from intent through
context, tools, children and learned skill, including failures and missing data.
History is queryable through CLI and cyb with the same privacy/evidence semantics.

## release requirement

A concrete agent release pins all participating organ/runtime/adapter/schema
versions and a complete capability manifest. [Evaluation](evaluation.md) defines
feature parity and superiority claims. Neuron conformance is necessary for this
composition, and cannot establish the agent's task competence by itself.

## Local native composition and parity scope

The migration's local composition is specified by Soma's
[neuron tasks](../../soma/specs/neuron-tasks.md) and
[local provider](../../soma/specs/local-provider.md): common Host/Registry custody,
bounded Rune execution, pinned local model, retained context/observations,
controls, child joins, schedules and learning proposals. Task data and execution
statuses are interpreted through those exact schemas. Their wire enums need not
literally use every semantic status label in this broader requirement document.

The convenience workspace revision is an explicit root label, not a snapshot of
all files. The bounded read tool can check an expected content hash. Current
stream deltas are transient and terminal task artifacts are durable. Native
model/tool adapters are trusted local code with their declared cooperative limits.

Remote providers, additional write/browser/media tools, MCP/ACP, messenger
channels and broad autonomous learning retain their individual parity gates in
the subsequent full-agent roadmap. This scope follows the accepted
[convergence roadmap §11](../../soft3/roadmap/neuron-cell-convergence.md).
Completing identity/execution migration establishes neither full Hermes parity
nor a comparative superiority result. All required agent features above remain
visible in the full-agent inventory rather than being silently dropped.
