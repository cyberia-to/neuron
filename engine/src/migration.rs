//! Explicit legacy-origin mapping into the authenticated subject's execution state.
use crate::neuron::Scope;
use crate::{Authority, Error, MigrationPort, Neuron, RuntimePort, legacy, neuron::Publication};
use neuron_model::execution::{self as model, Invocation, PendingOperation, Prog, Status};
use neuron_model::{Builder, Head, Lifecycle, NeuronId, Particle, Reader, Value::*};

#[derive(Clone, Copy, Debug)]
pub struct ImportOrigin {
    pub origin: Particle,
    pub install_nonce: [u8; 32],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Imported {
    pub origin: Particle,
    pub prog: Particle,
    pub invocation: Option<Particle>,
    pub incompatibility: Option<String>,
}
pub struct ImportPlan {
    publication: Publication,
    expected: Head,
    sources: Vec<(Particle, Head)>,
    allowed: Vec<u64>,
    pub manifest: Particle,
    pub mappings: Vec<Imported>,
}
#[derive(Debug)]
pub struct ImportReceipt {
    pub head: Head,
    pub manifest: Particle,
    pub mappings: Vec<Imported>,
}
impl<G: MigrationPort, R: RuntimePort, A: Authority> Neuron<G, R, A> {
    pub fn prepare_import(
        &self,
        neuron: NeuronId,
        nonce: [u8; 32],
        mut origins: Vec<ImportOrigin>,
    ) -> Result<ImportPlan, Error> {
        if origins.is_empty() || origins.len() > model::MAX_PROGS {
            return Err(Error::Budget);
        }
        origins.sort_by_key(|o| o.origin);
        if origins.windows(2).any(|w| w[0].origin == w[1].origin) {
            return Err(Error::Conflict);
        }
        let old = self.inspect(neuron)?;
        let mut state = old.state.clone();
        let mut b = Builder::new();
        let request = Self::request(&mut b, neuron, nonce)?;
        if self.graph.resolve(neuron, request)?.is_some() {
            return Err(Error::Conflict);
        }
        let mut sources = Vec::new();
        let mut mappings = Vec::new();
        let mut records = Vec::new();
        for origin in &origins {
            if origin.origin == neuron || state.imports.contains_key(&origin.origin) {
                return Err(Error::Conflict);
            }
            let legacy = legacy::inspect(&self.graph, origin.origin)?;
            let install_nonce = b.nonce(origin.install_nonce)?;
            let prog = b.values("neuron/prog-id/1", &[Ref(neuron), Raw(install_nonce)])?;
            if state.progs.contains_key(&prog) {
                return Err(Error::Conflict);
            }
            let mut program = Prog {
                source: legacy.source,
                state: legacy.snapshot.application_state,
                revision: 0,
                lifecycle: legacy.snapshot.lifecycle,
                step_limit: legacy.allowance,
                max_inflight: 1,
                allowed_acts: legacy.allowed_acts,
            };
            program.allowed_acts.sort_unstable();
            program.allowed_acts.dedup();
            let mut incompatibility = None;
            let invocation = legacy.live.as_ref().map(|live| live.event);
            if let Some(live) = legacy.live {
                let mut r = Reader::new(&self.graph, 100_000);
                let event = r.record(live.event, "cell/event/1", 10)?;
                let input = r.reference(event[4])?;
                let checkpoint = r.artifact(live.checkpoint)?;
                if let Err(e) = self.runtime.validate_checkpoint(&checkpoint, live.limit) {
                    incompatibility = Some(e.to_string());
                    program.lifecycle = Lifecycle::Paused;
                }
                let pending = if let Some(p) = live.pending {
                    let attempt = p
                        .attempt
                        .map(|id| {
                            let f = r.record(id, "cell/attempt/1", 7)?;
                            r.reference(f[0])
                        })
                        .transpose()?;
                    let (value, failed) = if let Some(outcome) = p.outcome {
                        let f = r.record(outcome, "cell/outcome/1", 6)?;
                        if let Some(value) = r.optional_ref(f[3])? {
                            (Some(value), false)
                        } else {
                            (Some(r.optional_ref(f[4])?.ok_or(Error::Conflict)?), true)
                        }
                    } else {
                        (None, false)
                    };
                    let stage = match p.stage {
                        "pending" => 0,
                        "attempt-recorded" => 1,
                        "resolved" => 2,
                        _ => return Err(Error::Unsupported),
                    };
                    let record = if stage == 0 {
                        let executor = b.blob(b"neuron/local-manual-reconciliation/1".to_vec())?;
                        b.values(
                            "neuron/operation/1",
                            &[
                                Ref(neuron),
                                Ref(state.network),
                                Ref(prog),
                                Ref(live.event),
                                Ref(p.id),
                                Uint(p.tag),
                                Ref(p.arguments),
                                Ref(state.policy),
                                Uint(state.epoch),
                                Ref(executor),
                            ],
                        )?
                    } else {
                        p.record
                    };
                    Some(PendingOperation {
                        id: p.id,
                        record,
                        tag: p.tag,
                        arguments: p.arguments,
                        attempt,
                        outcome: p.outcome,
                        stage,
                        value,
                        failed,
                        dispatch: None,
                        attempt_record: None,
                    })
                } else {
                    None
                };
                let job = Invocation {
                    prog,
                    code: legacy.source,
                    input,
                    context: live.context,
                    base_revision: 0,
                    checkpoint: Some(live.checkpoint),
                    status: if pending.is_some() {
                        Status::Waiting
                    } else {
                        Status::Running
                    },
                    limit: live.limit,
                    charged: live.charged,
                    reserved: live.reserved,
                    used: live.used,
                    pending,
                    result: None,
                    fault: None,
                    parent: None,
                    delegated: 0,
                    children: Vec::new(),
                    epoch: state.epoch,
                    ordinal: 1,
                };
                if state.invocations.insert(live.event, job).is_some() {
                    return Err(Error::Conflict);
                }
            }
            state.charged = state
                .charged
                .checked_add(legacy.charged)
                .ok_or(Error::Budget)?;
            state.progs.insert(prog, program);
            sources.push((origin.origin, legacy.head));
            records.push(b.values(
                "neuron/import-origin/1",
                &[
                    Ref(origin.origin),
                    Ref(legacy.head.commit),
                    Uint(legacy.head.index),
                    Ref(prog),
                ],
            )?);
            mappings.push(Imported {
                origin: origin.origin,
                prog,
                invocation,
                incompatibility,
            });
        }
        let manifest = b.values(
            "neuron/import-manifest/1",
            &[
                Ref(neuron),
                Ref(state.network),
                Ref(state.policy),
                Uint(state.epoch),
                Ref(old.head.commit),
                Refs(&records),
            ],
        )?;
        let mut imports = Vec::new();
        for (mapping, (_, head)) in mappings.iter().zip(&sources) {
            let incompatibility = mapping
                .incompatibility
                .as_ref()
                .map(|e| model::artifact(&mut b, e.as_bytes().to_vec(), "text/plain"))
                .transpose()?;
            let record = b.values(
                "neuron/import/1",
                &[
                    Ref(mapping.origin),
                    Ref(head.commit),
                    Uint(head.index),
                    Ref(mapping.prog),
                    Optional(mapping.invocation),
                    Ref(manifest),
                    Optional(incompatibility),
                ],
            )?;
            state.imports.insert(mapping.origin, record);
            imports.push(record);
        }
        // The governing grant must cover every requested imported act as well.
        let allowed = state
            .progs
            .values()
            .flat_map(|p| p.allowed_acts.iter().copied())
            .collect::<std::collections::BTreeSet<_>>();
        let allowed = allowed.into_iter().collect::<Vec<_>>();
        let authorization = self.evidence(
            &mut b,
            &old.state,
            Scope {
                kind: "import",
                prog: None,
                invocation: None,
            },
            None,
            manifest,
            &allowed,
        )?;
        imports.push(authorization);
        let publication = self.prepare(
            &old,
            state,
            b,
            (manifest, request),
            Scope {
                kind: "import",
                prog: None,
                invocation: None,
            },
            &imports,
        )?;
        Ok(ImportPlan {
            publication,
            expected: old.head,
            sources,
            allowed,
            manifest,
            mappings,
        })
    }
    pub fn activate_import(&self, plan: ImportPlan) -> Result<ImportReceipt, Error> {
        let p = plan.publication;
        let current = self.inspect(p.neuron)?;
        if current.head != plan.expected {
            return Err(Error::Conflict);
        }
        let action = crate::Action {
            neuron: p.neuron,
            network: current.state.network,
            policy: current.state.policy,
            epoch: current.state.epoch,
            prog: None,
            invocation: None,
            act: None,
            kind: "import",
            statement: plan.manifest,
            allowed_acts: &plan.allowed,
        };
        let mut write = Some(crate::MigrationWrite {
            neuron: p.neuron,
            request: p.request,
            expected: plan.expected,
            head: p.head,
            content: p.content,
            event: p.event,
            manifest: plan.manifest,
            sources: plan.sources,
        });
        let mut head = None;
        self.authority.with_current(&action, &mut || {
            head = Some(
                self.graph
                    .commit_migration(write.take().ok_or(Error::Conflict)?)?,
            );
            Ok(())
        })?;
        let head = head.ok_or(Error::Denied)?;
        Ok(ImportReceipt {
            head,
            manifest: plan.manifest,
            mappings: plan.mappings,
        })
    }
    /// Request retry is resolved before recomputing a manifest against a later root.
    pub fn import(
        &self,
        neuron: NeuronId,
        nonce: [u8; 32],
        mut origins: Vec<ImportOrigin>,
    ) -> Result<ImportReceipt, Error> {
        let mut b = Builder::new();
        let request = Self::request(&mut b, neuron, nonce)?;
        if let Some(head) = self.graph.resolve(neuron, request)? {
            let manifest = self.committed_event(head)?;
            let mut r = Reader::new(&self.graph, 100_000);
            let f = r.record(manifest, "neuron/import-manifest/1", 6)?;
            if r.reference(f[0])? != neuron {
                return Err(Error::Conflict);
            }
            let records = r.list(f[5], model::MAX_PROGS)?;
            origins.sort_by_key(|o| o.origin);
            if records.len() != origins.len() {
                return Err(Error::Conflict);
            }
            let view = self.inspect(neuron)?;
            let mut mappings = Vec::new();
            for (record, origin) in records.iter().zip(&origins) {
                let id = r.reference(*record)?;
                let f = r.record(id, "neuron/import-origin/1", 4)?;
                let nonce = b.nonce(origin.install_nonce)?;
                let prog = b.values("neuron/prog-id/1", &[Ref(neuron), Raw(nonce)])?;
                if r.reference(f[0])? != origin.origin || r.reference(f[3])? != prog {
                    return Err(Error::Conflict);
                }
                let record = *view
                    .state
                    .imports
                    .get(&origin.origin)
                    .ok_or(Error::Missing)?;
                let f = r.record(record, "neuron/import/1", 7)?;
                if r.reference(f[5])? != manifest {
                    return Err(Error::Conflict);
                }
                let incompatibility = r
                    .optional_ref(f[6])?
                    .map(|id| model::read_artifact(&mut r, id))
                    .transpose()?
                    .map(String::from_utf8)
                    .transpose()
                    .map_err(|_| Error::Conflict)?;
                mappings.push(Imported {
                    origin: origin.origin,
                    prog,
                    invocation: r.optional_ref(f[4])?,
                    incompatibility,
                });
            }
            return Ok(ImportReceipt {
                head,
                manifest,
                mappings,
            });
        }
        self.activate_import(self.prepare_import(neuron, nonce, origins)?)
    }
}
