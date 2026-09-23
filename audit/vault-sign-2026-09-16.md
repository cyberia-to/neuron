# Vault signing integration — 2026-09-16

The local CLI can use encrypted Vault custody with `--vault-home PATH
--vault-root ID`, optionally `--vault-domain DOMAIN` or `--vault-path PATH`.
Vault handles protected password input, derives the existing key internally and
signs through durable encrypted receipts and two verified copies. The local
authority holds its action grant throughout `SigningVault::sign`; graph
publication and dispatch retain their independent current-permission checks.
`--key-file` remains explicit legacy custody and cannot be combined with Vault.

The full release workspace suite passed 47 tests. The new library integration
test verifies signatures on real graph transitions and denies publication after
revocation. The new process test activates a Neuron, installs a program, executes
it and retries after restart using encrypted Vault custody, without a raw key
file. CLI/node Clippy passed for all targets with owned-code warnings denied.

This is the existing local-operator profile, not a remote Ward service or isolated
signing process. It is implemented on top of the pre-existing uncommitted
cell-to-neuron convergence; no real keys were migrated. See the
[Vault evidence and isolated integration delta](../../vault/audit/sign-2026-09-16/README.md).
