# Typed destinations

Status: accepted. `neuron-model::navigation` is an identity-only adapter; it
performs no key operation, network request, code loading, or attachment change.

`Route { destination: Destination, network: Option<NetworkRef> }` separates
`View`, `Particle`, `Neuron(SubjectRef)` and `Prog { subject, prog }`. A network
qualifies a subject/prog destination and must match its native/foreign domain.
Absence of a network means unqualified inspection, never the current network.

Canonical addresses (hex IDs are lowercase; foreign address bytes stay exact):

- `cyb://view/NAME`
- `cyb://particle/HEX32`
- `cyb://neuron/native/HEX32[?network=native/HEX32]`
- `cyb://neuron/foreign/DOMAIN/ADDRESS_HEX[?network=foreign/DOMAIN/NETWORK_HEX]`
- `cyb://prog/native/HEX32/PROG_HEX32[?network=native/HEX32]`
- `cyb://prog/foreign/DOMAIN/ADDRESS_HEX/PROG_HEX32[?network=foreign/DOMAIN/NETWORK_HEX]`

Domain and view segments use percent-encoded UTF-8. Parsing is bounded to 4096
bytes and validates all decoded references. Unknown query fields, path tails,
fragments, malformed bytes and mismatched domains fail. URI normalization never
hashes an address into a new identity. Display encoding (for example Bech32) is
an independent profile over the retained native or foreign reference.

Legacy `cell://ID` requires an explicit resolver mapping that exact ID to a
validated Route. Cyb's built-in mapping is only `landing` → View(`robot`). Old
runtime origin IDs require an import mapping; an arbitrary name or hash is never
interpreted as executable code or a new neuron. Legacy `cyb://brain` style view
aliases are a bounded UI compatibility table and normalize to `cyb://view/...`.

Navigating to a neuron/prog opens inspection without selecting, attaching,
creating, signing, or running it. A subsequent authored action uses a separately
captured attachment and current grant. Launcher items, pins and context resources
carry Route values; their labels and icons carry no authority.
