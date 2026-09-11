---
title: local runtime implementation profile
tags: cell, rust, implementation, spec
status: draft
spec-version: "0.2"
---
# local runtime implementation profile

This experimental implementation profile instantiates a subset of the generic
contracts for one private local host. It is not the conforming runtime baseline:
per-neuron publication, the complete resource metric contract, upgrades and
subscription semantics remain release gates. Those requirements are unchanged.
The exclusively opened graph database and its process are the local authority.
Receipts state LocalDurable and LocalAuthority, with host-observed computation.
Remote authorship, network SignalChain publication and consensus are unsupported
by this profile. Publication adapters retain their upstream chain ordering.

The runtime is rune's bounded machine/checkpoint revision 1. A source expression
sees a mounted Subject whose mem is current application state. The admitted
input is the lexical `event` binding; here carries the optional context particle,
and clock/world default to explicit empty values. Its final noun is
the next application state and the invocation result. Nouns use rune's bounded
checkpoint codec as blob artifacts, pinned by the runtime ABI.

The initial host admits one live invocation per instance, with at most one
pending act. Other instances have independent state. Completed inbox/outbox
entries leave the live snapshot; their graph records and nonce indexes remain.
Repeated admission returns the original event receipt. Changed data with the
same origin/nonce conflicts. Paused cells retain pending work and accept outcomes;
retirement requires resolution of pending obligations.

Schema manifests are ordinary Model B data: pair(atom 0x53434831,
fields(name, contract_particle)), with fields a zero-terminated right-cons list.
Name uses the data.md UTF-8 representation; contract_particle identifies the
exact bytes of [schema suite 1](../model/schema-suite-v1.txt), hashed by the
hemera byte-artifact codec. That file is immutable after this implementation
version; changing documentation does not change schema identity. A suite change
requires an explicit reader/migration and new manifest identities.
Records are pair(particle_value(manifest), ordered fields). Particle values use
the stack's balanced four-field shape [[h0 h1] [h2 h3]]. All used manifests and
contract artifacts are retained. Name aliases are never used as schema identity.

Blob artifacts use hemera byte hashing, with an explicit codec particle. Data
nodes use nox Model B structural identity. uint remains [high32 low32]. Collections
use canonical sorted maps bounded by the profile's one-live-invocation quota;
the unbounded history/dedup indexes use BBG's persistent B-tree path.

The local-resources record holds charged steps, reserved steps, last checkpoint's
used steps and the invocation limit, all uint. Local policy binds owner, step
limit and a sorted unique list of permitted act tags. Lifecycle, inbox status,
operation stage, trigger, target, retry and outcome variants use names under
cell/{domain}/{variant}/1 with the owning contract's fields. The initial adapter
rejects event subscriptions and remote executors explicitly.

The engine records admission/checkpoint, pending Operation, Attempt and Outcome
before each corresponding transition or dispatch. The host resolves allow/deny
using a current local policy; automatic executors advertise their idempotency
contract. An attempted effect without a retained outcome restores as unknown and
requires a correlated outcome or executor reconciliation. It is never silently
dispatched twice. Recorded results resume the machine once.

WardPort receives the selected policy, operation, cell, act and epoch outside
the interpreter. Engine::new denies every act. The node's LocalWard adapter
enforces the private owner's stored allow-list; this is bootstrap authorization,
not an implementation of ward's delegated grants, secret routing or revocation
service. Denials are retained and leave the operation undispatched. Successful
and definite-failure outcomes bind operation/attempt IDs; changed terminal
results conflict. Definite failure faults the invocation and preserves its
application state. Completed/closed continuations retain consumed_by records.

The automatic emit adapter accepts a view into graph history and returns zero;
its CLI rendering is transient. It promises no exactly-once terminal delivery.
host(request) passes the complete noun to a manual executor. Query/link/seal and
subscribe tags have no automatic adapters here; granting a tag does not create
one. Reactive Event requests fail explicitly. Actor clocks are empty, not wall
time; external times/results must arrive as explicit inputs.

Each runtime slice reserves its maximum permitted steps before evaluation. A
recorded result settles actual usage; an interrupted slice consumes its reserved
exposure before retrying the saved checkpoint. Yield retains used steps and the
new checkpoint. Inputs, context particles, code and checkpoints share the graph
commit's availability boundary.

Limits enforced by this implementation: source 8192 bytes / 512 tokens, parser
nesting 64, lowering nesting 128 and 2048 expansions; runtime 65536 logical noun
nodes, depth 128, 256 frames/values and at most 1000000 steps per invocation.
Noun decoding is bounded; graph batches admit at most 131072 content entries /
16 MiB, individual model blobs at most 8 MiB. History pages hold at most 4096
heads. Readers bound record visits. These are logical and per-request limits;
body-enforced RSS, aggregate graph-read bytes/deadlines, storage quotas and all
remaining required ResourceContract metrics are not yet implemented. No hard
physical process-memory or complete baseline resource guarantee is advertised.

Unsupported lifecycle/profile/codec/runtime transitions return typed errors.
Implementation coverage is documented with executable tests in docs/implementation.md.
