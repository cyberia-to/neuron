# Local worker dispatch and fencing

This contract instantiates soft3's worker/job model for embedded host-act
adapters. A worker is an executor placement, never another signing subject.
The existing native neuron key authorizes placement and operations. Remote
execution or partition-safe takeover requires a separate verified profile.

## Selection and placement

A versioned worker descriptor pins the five execution-model selections:
machine/ABI, environment/ABI, proof profile, network and executor (warrior,
version, backend, worker, device and boot identifiers). The local host-act
adapter advertises its actual native function environment and proof=none;
it refuses other machine/environment/proof combinations before admission.
The descriptor also bounds accepted act tags and argument bytes. Worker and
boot identifiers locate work and supply fencing evidence; they grant no rights.

The neuron root retains a monotonic placement generation and an optional
worker descriptor. Rebinding or releasing placement is an authorized CAS
transition. A different boot never silently inherits an existing lease.
Replacement invalidates the predecessor's undispatched tokens. Previously
attempted effects remain attached to their old context and require reconciliation.
Local recovery may replace a boot only under the exclusive local database owner;
it does not assert that an unreachable remote executor has stopped.

## Attempt, dispatch and outcome

1. Preparation validates program lifecycle, current epoch, allowed acts, argument
   bound, selected profile and exact placement. It records the immutable attempt
   and worker descriptor before returning an opaque, non-Clone dispatch token.
2. The enforcing executor accepts only that token, consumes it by value and
   verifies the current subject/network/policy/epoch/placement and attempt.
   Public Dispatch metadata or historical signatures cannot construct a token.
3. A separate durable dispatch claim binds operation, attempt, placement
   generation and exact worker contract. CAS permits one claim. The host holds
   the current ward guard through claim publication and the immediate synchronous
   adapter call. This is the local dispatch linearization boundary; a revocation
   or replacement ordered before it rejects the operation, and one ordered after
   it cannot undo a possibly started effect.
4. The grant guard is released before accepting the adapter's outcome through
   the normal neuron transition path. Outcome acceptance pins original operation
   and attempt; new policy may permit reconciliation without permitting execution.

Reconcile, cancel, archive, pause/retire and reservation recovery require a
current subject/network/policy/prog-scoped grant, but do not require the prog's
now-revoked act set. Fully disabled grants still reject writes. Rebinding future
execution requires the current act set and cannot change an unknown attempt.
An explicitly reconciled result may survive an epoch change and then be rebound;
its original operation/attempt evidence remains immutable. Cancellation or budget
exhaustion consumes the retained result without executing another effect.

A crash before/after the claim or during the adapter call leaves an unknown
attempt and never recreates a token. Recovering the task, retrying admission or
cloning metadata cannot redispatch. A definite result can be reconciled once;
conflicting results fail. Failure after a successful external action remains
unknown until an executor receipt or explicit owner reconciliation resolves it.

Manual `take` remains an explicit external/manual-reconciliation interface. It
creates a potentially executed attempt, supplies no enforcing worker token and
cannot enter the embedded executor. It must not be advertised as automatic
worker fencing. Structured CLI/headless automatic adapters use the guarded path.

## Supported limits

The initial worker invokes synchronous trusted host functions. It bounds the
input and permits only registered acts. It does not claim to preempt arbitrary
native code or constrain arbitrary external provider billing. Profiles requesting
unsupported hard CPU/time/memory guarantees fail before dispatch. Program VM
step reservation remains the existing durable engine accounting contract.

## Acceptance

Executable scenarios must cover: revoke after attempt/before claim; old boot
after replacement; two workers racing; wrong subject/network/profile; duplicate
metadata/token attempt; crash after claim; outcome after policy change; unknown
legacy attempt; and a complete tool round-trip alongside an independent prog.
No secret appears in descriptors, graph records, tokens' public metadata or logs.
