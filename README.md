---
title: neuron
tags: neuron, soft3
status: implementation
---
# neuron

Neuron is the protocol subject and its optional durable execution model.
One robot attaches neurons for different keys, networks and devices. One neuron
can run several programs and tasks. Programs have data IDs and independent state;
they do not introduce another signing identity.

The identity-only library requires neither a VM nor a renderer. The runtime uses
Rune for bounded evaluation, cybergraph/BBG for history and atomic publication,
and host ward/vault adapters for authorization and signing. Cyb Log renders that
history.

Start with [identity](specs/identity.md), [runtime](specs/runtime-v1.md),
[authority](specs/local-authority.md), [migration](specs/migration.md) and
[CLI](specs/cli.md). The [complete specification](specs/README.md) follows the
accepted one-subject model. Remaining cross-repository work is tracked by the
[roadmap](../soft3/roadmap/neuron-cell-convergence.md).
The [convergence explanation](docs/cell-convergence.md) describes the accepted
model and links the preserved original design evidence.

## Run

Build with companion repositories alongside this one under ~/cyber:

```nu
cd ~/cyber/neuron
cargo build -p neuron-cli
./target/debug/neuron keygen demo.key
let opts = [--store demo.bbg --key-file demo.key]
let owner = (./target/debug/neuron ...$opts activate | from json)
let prog = (./target/debug/neuron ...$opts install $owner.neuron examples/counter.rune | from json)
./target/debug/neuron ...$opts submit $owner.neuron $prog.prog 7
./target/debug/neuron ...$opts submit $owner.neuron $prog.prog 5
./target/debug/neuron --store demo.bbg inspect $owner.neuron --prog $prog.prog
./target/debug/neuron --store demo.bbg history $owner.neuron
```

The counter's state is 12. Each command opens the same graph in a new process.
The local signing profile preserves mudra's H(compressed public key) identity
and existing ADR-036 bytes. Read-only inspection needs no signing key.
A key is created only by the explicit keygen command, which refuses an existing file.

For encrypted custody, select an existing Vault with two configured replicas:

```nu
let opts = [--store demo.bbg --vault-home /path/to/vault --vault-root ENTRY_ID]
./target/debug/neuron ...$opts activate
```

Vault prompts for its unlock password. Add `--vault-domain example.test` for a
domain-root entry; a spell uses the existing Cosmos path. Neuron receives public
credentials and signatures; Vault retains the root and performs `sign` after
authorization and durable replication. The `--key-file` path above remains an
explicit legacy adapter, mutually exclusive with encrypted custody. Neither mode
is selected as a fallback when the other fails. See [local authority](specs/local-authority.md).

Install another program under the same neuron to get independent program state.
Use --inflight to admit parallel jobs. A result based on an outdated program
revision is retained as a conflict; it cannot overwrite a newer state.
Parent/child allowances transfer within the same finite neuron budget.
Archived jobs release space, preserving spent resources and original request claims.

For [the tool example](examples/tool.rune), supply --grant-act host to the host
and --allow host to install. Submit returns an invocation and operation.
take NEURON INVOCATION records a signed attempt before an executor may act.
outcome NEURON INVOCATION OPERATION ATTEMPT VALUE supplies its correlated result.
After a crash, an attempted operation stays unknown until reconciliation.
A waiting task does not prevent other programs from running.
emit has a local display adapter; other effects require their owning adapters.

pause/resume/retire apply to a program; cancel/archive apply to an invocation.
Program retirement preserves the neuron. Source upgrade preserves admitted
continuations and uses the same state revision conflict rule.

## Legacy data

The immutable cell v1 schema suite and historical hashes remain unchanged.
The read-only legacy APIs identify a birth hash as an origin particle.
They do not cast it into a key identity.

```nu
./target/debug/neuron --store bbg legacy-inspect OLD_ORIGIN
./target/debug/neuron --store bbg legacy-history OLD_ORIGIN
./target/debug/neuron --store bbg --key-file owner.key --grant-act host import NEURON OLD_ORIGIN=INSTALL_NONCE --nonce IMPORT_NONCE
```

IDs and nonces above are 64 hexadecimal characters. Import several mappings
together to converge several origins into one neuron. Activation checks all
source/target heads and atomically fences the original namespaces. An exact
rerun returns the same receipt. History, completed request claims, checkpoints,
unknown attempts and resource charges retain their original provenance.
The original binary refuses a store after semantic activation.

An existing implicit cell.redb prevents accidentally creating an empty default
BBG store. Backend conversion is separate: build with --features
legacy-redb-migration, then run neuron migrate-redb SOURCE DESTINATION before
semantic import. Conversion keeps the source; subsequent work selects the
destination explicitly.

## Packages and validation

- id: dependency-free common NeuronId, preserving native wire bytes.
- model: identity/binding/navigation; optional canonical runtime records.
- engine: programs, invocations, budgets, effects, recovery and migration.
- rune: bounded source/checkpoint adapter.
- node: shared graph storage and supported ward/vault composition.
- cli: the same composition without a GUI.

The local profile provides durable local records and supported key authorization.
Network proof/finality and the full agent composition have their own conformance
gates; this workspace does not infer those guarantees from local execution.

Run cargo test --workspace --offline and cargo clippy --workspace --all-targets
--offline --no-deps -- -D warnings. Format only these packages with cargo fmt
-p neuron-id -p neuron-model -p neuron-engine -p neuron-rune -p neuron-node
-p neuron-cli; --all also traverses companion repositories.

The [implementation ledger](../soft3/audit/neuron-cell/implementation.md) records
actual validation and the remaining cross-repository work.
