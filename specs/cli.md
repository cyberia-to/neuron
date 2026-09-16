# Neuron CLI

Automatic local emit uses the guarded embedded worker path: explicit supported
descriptor, fresh placement generation, affine attempt token and one-use durable
dispatch claim. Its device/boot label identifies this disposable local process;
it makes no physical-device or remote-attestation claim. External/manual `take`
never constructs a worker token. Running a no-longer-retained invocation reports
`not-retained`; retrying its admission still returns its preserved original receipt.

The executable is `neuron`. Mutations require an explicitly selected host key;
inspection/history and legacy inspection need no VM authority. The supported
local key file holds one secp256k1 scalar, created only by explicit `keygen FILE`
with exclusive creation and owner-only permissions. Identity is the existing
mudra H(compressed_pubkey), never a program installation hash.

`--key-file FILE` selects vault custody. `--network ID` pins a network explicitly;
otherwise an existing subject's stored network is used. New local activation
uses the versioned local network identifier. `--grant-act NAME` supplies current
host rights. A program's `install --allow NAME` requests a subset of those rights;
its manifest cannot issue rights. No key bytes are emitted in JSON or graph data.

Commands: keygen, identity, activate, install, submit, run, inspect, history,
take, outcome, fail, wake, pause, resume, cancel, retire, upgrade, archive, import,
legacy-inspect, legacy-history. Programs and invocations are explicit arguments
under the neuron subject. Admission defaults to the program's bounded allowance;
parent allocations use `--parent` and `--steps`. `--queue-only` persists admission
without evaluation. Stable `--nonce` returns the original receipt or conflicts.

`take` durably records an attempt and returns bound subject/network/prog/invocation/
operation/attempt/authorization. A repeated take is unknown, not permission to
redispatch. `outcome` and `fail` correlate exact operation/attempt identities.
Paused programs retain outcomes and resume only after explicit activation.
`wake` delivers a correlated event result using its own stable nonce.

`import NEURON ORIGIN=INSTALL_NONCE... --nonce NONCE` migrates inspected legacy
namespaces in the selected shared store and fences them atomically. An exact
rerun returns the original mapping. `legacy-inspect` and `legacy-history` retain
explicit provenance; normal subject APIs do not accept a birth hash as identity.
Backend redb migration remains a separate feature-gated conversion command.

For a separate SSD source, first activate the authenticated target in `--store`,
then use `stage-import NEURON --source LEGACY_DIR --nonce TRANSFER_KEY --pages N`.
The source must be closed by other processes. This seals its ordinary writer
entry point and copies bounded pages. JSON reports the manifest, copied row/byte
counts and `complete`; rerun with the same key until complete. Interrupted pages
do not advance the cursor. The source remains retained and sealed. Then run
`import` with explicit origin-to-prog mapping and granted acts. Staging alone
creates no invocation and cannot dispatch. A changed transfer key/target conflicts.

An existing implicit `cell.redb` prevents accidentally opening an empty default
BBG store. Explicit `--store` selects a store; existing files are never replaced
by an empty directory. The CLI reports local durability and local authority,
without claiming network finality or unimplemented proof verification.


`legacy-source-inspect SOURCE [--destination PATH]` opens the source explicitly,
including after a seal. It reports validated source heads, counts/digest, programs,
resources, live/unknown operations, checkpoint compatibility and sizing. It needs
no key, target root or execution grant and never seals an unsealed source.

`legacy-export SOURCE DESTINATION --target NEURON --nonce TRANSFER_KEY --pages N`
uses the same bounded BBG logical transfer without activating a subject. The
source is sealed and retained; the destination holds inert staged namespaces.
A page-limited result reports `complete=false`; rerun the exact command until
complete. The intended target and nonce are public cutover metadata, not custody.
Then explicitly activate that subject under its key and run semantic `import`
with the required mappings/grants, or stage the retained source into its already
active shared store. Export creates no key and executes no pending invocation.
