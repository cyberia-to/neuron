# Legacy cutover operator guide

The original cell namespaces remain readable provenance. Explicit mapping
converges them into progs of one authenticated neuron. Every migration keeps
the source; no source deletion or automatic replay is part of cutover.

1. Stop the source writer and retain its directory and separately held custody.
   Use `neuron legacy-source-inspect SOURCE --destination DEST`. Inspect all
   origins, heads, resource totals, pending attempts and checkpoint diagnostics.
   Record the logical digest. The command needs no signing key and also reads a
   sealed source. A busy source, missing object, malformed receipt or exceeded
   inspection bound is an error; repair or choose a supported source first.
2. Select the actual target subject from its existing key. If creating custody,
   use explicit `neuron keygen KEYFILE`, then `neuron --key-file KEYFILE identity`.
   Keep the key outside both stores. A birth hash is an origin identifier.
3. For an inert export, run `neuron legacy-export SOURCE DEST --target ID
   --nonce TRANSFER_KEY --pages 32` repeatedly with identical arguments until
   `complete=true`. It first validates the entire source, then seals it and
   writes bounded pages into the BBG destination. No custody is needed for this
   phase. The target ID and nonce are public, pinned cutover metadata.
4. Activate the target explicitly: `neuron --store DEST --key-file KEYFILE
   activate`. Grant the required supported host actions explicitly. Then use
   `neuron --store DEST --key-file KEYFILE import ID ORIGIN=INSTALL_NONCE ...
   --nonce IMPORT_NONCE`. Every origin needs its own stable installation nonce.
   Import validates current authority and publishes mappings plus origin fences
   atomically. An exact retry returns the original mapping.
5. Compare the resulting progs, history, states, charges, held reservations,
   claims and unknown attempts against inspection. Use legacy-history for the
   original authorship and inspect/history for the target execution records.
   Reopen both stores and inspect the sealed source digest again.
6. Reconcile an unknown attempt only with evidence for its exact operation and
   attempt. A pending side effect has no automatic retry. An incompatible
   checkpoint remains paused with preserved bytes until an explicit compatible
   runtime or upgrade is chosen. It must never become an empty continuation.

For an already active target, `stage-import ID --source SOURCE --nonce KEY
--pages N` performs the same retained-source transfer under current authority;
then semantic import uses the explicit mapping. Same-store import directly uses
the mapping without a transfer. A source seal is permanent for normal writers.
Changing the destination subject or transfer key after sealing conflicts. A
sealed retained source can resume the same pinned transfer after interruption.

The inspector limits are 1,000,000 logical rows, 8 GiB logical bytes and 256
application namespaces. `--max-rows` and `--max-bytes` can reduce these bounds.
The initial source scan is complete before sealing; each copy page is bounded
by 512 rows and the BBG transaction byte ceiling. `--pages` is 1–4096 per call.
This profile accepts application-only SSD stores; native/shared graph domains
and redb conversion use their separate owner contracts.

Destination sizing reports available bytes at the nearest existing ancestor and
a planning allowance of four times logical bytes plus 64 MiB. It reserves no
disk space and cannot predict backend overhead exactly. Before-seal destination
open errors leave a fresh source writable. After-seal failures retain the source
and the last atomic destination cursor. Reopen and retry the same transfer;
never remove the seal or interpret a partial target as an activated neuron.

Contracts: [migration](../specs/migration.md), [CLI](../specs/cli.md),
[BBG transfer](../../bbg/specs/application-transfer.md).
Observed boundary/fault coverage lives in the
[cutover audit](../../soft3/audit/neuron-cell/legacy-cutover.md).
