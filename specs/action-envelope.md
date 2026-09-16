# Public action envelope for host and network adapters

ActionRequest is public data, without custody or dispatch capability. Its fields
in canonical compact JSON order are `request, attachment, context, kind, payload`.
References and payload use JSON byte arrays. Context uses the existing identity
module's subject/network representations and field order:
`subject,network,binding_revision,prog,invocation,policy,grant`.
Kind is 1..128 printable ASCII bytes. Payload is at most 8 MiB before encoding;
the enclosing record/transport imposes its additional total encoded-size bound.

The signing statement is H(canonical compact JSON of the two-element array
`["cyb/robot-action/1", action]`). This domain string is retained from the first
host implementation so its existing signed records do not change identity.
It names a codec version, not another signing subject. The NSIG1 envelope over
that statement is owned by mudra; neuron-model only encodes public records.

A network envelope's canonical fields are `schema, action, evidence`, where schema
is `neuron/signed-action/1` and evidence is the exact 102-byte NSIG1 array.
The entire encoded envelope is bounded at 8 MiB. Decode rejects unknown fields,
noncanonical serialization, invalid identity/context references, inconsistent
prog/invocation presence, wrong evidence size, oversized payload or kind.
The network then verifies the signature, pinned destination, supported operation
kind and its own admission rules. Parsing never grants authority by itself.

Action data/serde consumers depend on neuron-id and optional serde/serde_json,
without an engine, Rune, storage, inference or UI. Signature verification uses
mudra without its optional mnemonic/proving features. Disabling all model default
features and serde retains the ID-only dependency boundary.
