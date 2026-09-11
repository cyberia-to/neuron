# cell implementation

Authorized on 2026-09-11 after review of draft 0.2.

## executable boundaries

1. BBG owns atomic persistent application records/content/head storage. Cybergraph
   owns validation, content identity, conditional history and queries. Cell uses
   this public path; local execution advertises local durability only.
2. Rune owns bounded reduction and serializable deep continuations. Authority is
   supplied by the host, independent of mutable subject and checkpoint bytes.
3. Cell model owns canonical records and schema manifests. Engine owns admission,
   lifecycle, outbox and recovery. Node composes the production adapters. CLI
   exercises source loading, instances, events, inspection and recovery.
4. The integration suite uses distinct births, conflicting/duplicate requests,
   nested acts, restart at effect boundaries, denied operations, exhausted
   budgets and durable state/artifact recovery.
5. Update conformance coverage and remaining integration work from executable
   evidence. Wider soma/agent, distributed finality and remote placement remain
   separate advertised profiles with their own integration gates.

## verification

Each owning repository runs its focused tests, formatting/lints for changed code,
then downstream integration tests. CLI scenarios run in fresh temporary stores
and reopen the same database in a new process. No live user graph is migrated by
these tests. Commits remain local on feature branches.

## progress

- Source boundaries inspected; cell, bbg, cybergraph and rune feature branches created.
- Local slice implemented in model, engine, rune, node and CLI.
- BBG atomic application storage and cybergraph application API implemented.
- Rune authority binding, bounded source lowering and deep checkpoints implemented.
- Model/node/recovery/new-process CLI tests and strict cell Clippy pass.
- Upstream focused tests and the rune workspace pass. Existing BBG/cybergraph
  strict Clippy warnings are recorded in docs/implementation.md.
- Remaining baseline and agent gates are explicit in docs/conformance-local.md;
  this slice advertises only local-experimental/1.
