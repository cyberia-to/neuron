---
title: neuron program and task lifecycle
tags: neuron, soft3, spec
status: accepted
spec-version: "0.3"
---
# Lifecycle

Neuron identity, robot attachment, installed prog, invocation and host placement
have distinct lifecycles. Closing a view changes its surface; stopping a prog
never deletes the subject or its other work. Detaching/revoking a binding changes
access through that robot without rewriting protocol history.

## Program state

| Persisted state | Allowed behavior | Supported next state |
|---|---|---|
| Installed | Inspect and negotiate/validate activation; retained by compatibility profiles | Active or Retiring |
| Active | Admit and run bounded work; record outcomes and management | Paused or Retiring |
| Paused | Retain outcomes, inspect, authorized upgrade/rebind/resume | Active or Retiring |
| Retiring | Inspect settled obligations; no new work | Retired |
| Retired | Historical read/export under retention/access rules | Terminal |

The native local install validates code/state and commits Active directly under
an existing activated neuron. Caching source alone installs nothing. Native
activation binds actual subject/network/policy and cumulative budget; it does
not create a second birth identity. Repeating an exact installation nonce returns
the same prog; changed code/state/config under that nonce conflicts.

Current `manage` refuses Retiring/Retired while any invocation of that prog is
nonterminal. It does not hide unknown operations in retired history or detach
unsettled children. A profile with asynchronous retirement must define retained
obligation ownership and settlement before advertising that feature.

Activation/installation requires available validated artifacts, compatible
runtime/checkpoint semantics, supported resource/evidence profiles and a current
ward grant. Failed admission publishes no successful state. Pure views can use
retained data without installing a program or obtaining signing custody.

## Invocation state and cancellation

An admitted invocation is running/waiting/joining or terminal completed,
cancelled, failed/conflict. Unknown attempted effects are a waiting condition
with the original operation/attempt retained. Independent progs continue running;
shared state adoption checks base revision. An invocation's source/context never
silently follows a new selection during resume.

Before an attempted effect, cancellation can settle charged/reserved work and
publish a definite cancelled result. An attempted unknown returns UnknownOutcome;
a live child returns Busy until child obligations settle. A higher-level Soma
cancel can persist intent and drive that tree to safe boundaries. It does not
claim an already handed-off provider/tool stopped. Definite observed outcomes
can be consumed during cancellation without another external call.

Terminal top-level task trees may be archived from bounded live maps. Original
admission/result claims, charges, provenance and artifacts remain graph history.
Archiving is not a budget reset. A program's retirement has no effect on other
programs or the robot's ability to observe the subject.

## Pause, upgrade and authority

Pause stops new scheduling/dispatch for that prog and preserves checkpoints and
outcomes. Resume requires current grant/epoch, compatible code and resources.
A result may be recorded while paused; consuming it waits for permitted execution.
Upgrades increment the prog state revision and retain each admitted source and
continuation. Older results conflict with incompatible new state rather than
silently overwriting it or repeating its effects.

Revocation is monotonic and independent of lifecycle. Loading a snapshot cannot
revive an old grant. Explicit rebind may authorize future work only under the
current profile and allowed acts; it cannot alter an unknown attempted action's
original subject/network/arguments. Fully disabled grants still refuse new writes;
read-only inspection needs no signer. Original evidence remains attributable.

## Host placement and faults

Restoring/ready/running/waiting/unavailable/quarantined/detached describe host
observations, not another durable subject enum. Each observation pins a head and
reason where available. Restart verifies committed state, settles lost compute
reservations conservatively and checks current authority/worker generation before
new dispatch. Missing code leaves data inspectable and work suspended.

A worker/device/boot change requires an authorized generation transition. Local
exclusive ownership fences the supported profile; remote partition-safe takeover
needs its own proof/lease contract. CRDT synchronization does not establish it.
A native stack pointer is never a portable checkpoint.

Runtime faults retain actual resource charges and diagnostic artifacts. Failure
to persist the fault is reported as storage/unavailable/unknown, never a durable
success. Management and work share subject-head CAS, so concurrent changes cannot
both commit against one predecessor. Removing a local replica changes availability,
not the subject's historical identity or remote facts.
