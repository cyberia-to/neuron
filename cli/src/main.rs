mod archive;
use clap::{Parser, Subcommand};
use neuron_engine::{
    Admission, Authority, GraphPort, ImportOrigin, JobProgress, Neuron, ProgramConfig,
};
use neuron_model::{Builder, Lifecycle, Particle, Reader, execution::read_artifact};
use neuron_node::{Grant, GrantHandle, Graph, KeyVault, LocalAuthority, Rune, SigningVault};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(
    name = "neuron",
    version,
    about = "Identity and durable programs in the shared cybergraph"
)]
struct Args {
    #[arg(long, global = true)]
    store: Option<PathBuf>,
    #[arg(long, global = true)]
    key_file: Option<PathBuf>,
    #[arg(long, global = true)]
    network: Option<String>,
    #[arg(long, global = true)]
    grant_act: Vec<String>,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Keygen {
        file: PathBuf,
    },
    Identity,
    Activate {
        #[arg(long, default_value_t = 10_000_000)]
        budget: u64,
    },
    Install {
        neuron: String,
        source: PathBuf,
        #[arg(long, default_value = "0")]
        initial: String,
        #[arg(long)]
        nonce: Option<String>,
        #[arg(long, default_value_t = 1_000_000)]
        steps: u64,
        #[arg(long, default_value_t = 1)]
        inflight: u64,
        #[arg(long)]
        allow: Vec<String>,
    },
    Submit {
        neuron: String,
        prog: String,
        input: String,
        #[arg(long)]
        nonce: Option<String>,
        #[arg(long)]
        context: Option<String>,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        steps: Option<u64>,
        #[arg(long)]
        queue_only: bool,
    },
    Run {
        neuron: String,
        invocation: Option<String>,
    },
    Inspect {
        neuron: String,
        #[arg(long)]
        prog: Option<String>,
    },
    History {
        neuron: String,
        #[arg(long)]
        after: Option<u64>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    Take {
        neuron: String,
        invocation: String,
    },
    Outcome {
        neuron: String,
        invocation: String,
        operation: String,
        attempt: String,
        value: String,
    },
    Fail {
        neuron: String,
        invocation: String,
        operation: String,
        attempt: String,
        reason: String,
    },
    Wake {
        neuron: String,
        invocation: String,
        operation: String,
        value: String,
        #[arg(long)]
        nonce: String,
    },
    Pause {
        neuron: String,
        prog: String,
    },
    Resume {
        neuron: String,
        prog: String,
    },
    Cancel {
        neuron: String,
        invocation: String,
        #[arg(long, default_value = "cancelled by local authority")]
        reason: String,
    },
    Retire {
        neuron: String,
        prog: String,
    },
    Upgrade {
        neuron: String,
        prog: String,
        source: PathBuf,
        #[arg(long)]
        state: String,
    },
    Archive {
        neuron: String,
        invocation: String,
    },
    Import {
        neuron: String,
        #[arg(required = true)]
        origins: Vec<String>,
        #[arg(long)]
        nonce: String,
    },
    LegacyInspect {
        origin: String,
    },
    /// Validate an original or sealed source without mutation or authority.
    LegacySourceInspect {
        source: PathBuf,
        #[arg(long)]
        destination: Option<PathBuf>,
        #[arg(long, default_value_t = neuron_node::archive::MAX_INSPECTION_ROWS)]
        max_rows: u64,
        #[arg(long, default_value_t = neuron_node::archive::MAX_INSPECTION_BYTES)]
        max_bytes: u64,
    },
    /// Seal and logically export to inert BBG staging; no key or target root needed.
    LegacyExport {
        source: PathBuf,
        destination: PathBuf,
        #[arg(long)]
        target: String,
        #[arg(long)]
        nonce: String,
        #[arg(long, default_value_t = 32)]
        pages: usize,
    },
    /// Seal a separate legacy SSD store and copy bounded pages into --store.
    StageImport {
        neuron: String,
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        nonce: String,
        #[arg(long, default_value_t = 32)]
        pages: usize,
    },
    LegacyHistory {
        origin: String,
        #[arg(long)]
        after: Option<u64>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    #[cfg(feature = "legacy-redb-migration")]
    MigrateRedb {
        source: PathBuf,
        destination: PathBuf,
    },
}
impl Command {
    fn subject(&self) -> Option<&str> {
        match self {
            Self::Install { neuron, .. }
            | Self::Submit { neuron, .. }
            | Self::Run { neuron, .. }
            | Self::Inspect { neuron, .. }
            | Self::History { neuron, .. }
            | Self::Take { neuron, .. }
            | Self::Outcome { neuron, .. }
            | Self::Fail { neuron, .. }
            | Self::Wake { neuron, .. }
            | Self::Pause { neuron, .. }
            | Self::Resume { neuron, .. }
            | Self::Cancel { neuron, .. }
            | Self::Retire { neuron, .. }
            | Self::Upgrade { neuron, .. }
            | Self::Archive { neuron, .. }
            | Self::Import { neuron, .. }
            | Self::StageImport { neuron, .. } => Some(neuron),
            _ => None,
        }
    }
}
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
struct CliAuthority(Option<LocalAuthority<KeyVault>>);
impl Authority for CliAuthority {
    fn with_current(
        &self,
        action: &neuron_engine::Action<'_>,
        commit: &mut dyn FnMut() -> std::result::Result<(), neuron_engine::Error>,
    ) -> std::result::Result<(), neuron_engine::Error> {
        self.0
            .as_ref()
            .ok_or(neuron_engine::Error::Denied)?
            .with_current(action, commit)
    }
    fn authorize(
        &self,
        action: &neuron_engine::Action<'_>,
    ) -> std::result::Result<Vec<u8>, neuron_engine::Error> {
        self.0
            .as_ref()
            .ok_or(neuron_engine::Error::Denied)?
            .authorize(action)
    }
}
type Agent = Neuron<Graph, Rune, CliAuthority>;
fn particle(text: &str) -> Result<Particle> {
    if text.len() != 64 || !text.is_ascii() {
        return Err("expected 64 hexadecimal characters".into());
    }
    let mut bytes = [0; 32];
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16)?;
    }
    Ok(bytes)
}
fn hex(id: Particle) -> String {
    id.iter().map(|b| format!("{b:02x}")).collect()
}
fn nonce(value: Option<String>) -> Result<Particle> {
    if let Some(value) = value {
        return particle(&value);
    }
    let mut bytes = [0; 32];
    getrandom::fill(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}
fn named(name: &str) -> Result<Particle> {
    Ok(Builder::new().blob(name.as_bytes().to_vec())?)
}
fn act(name: &str) -> Result<u64> {
    let n = match name {
        "emit" => 1,
        "query" => 2,
        "link" => 3,
        "seal" => 4,
        "subscribe" => 5,
        "host" => 6,
        _ => return Err("unknown act".into()),
    };
    Ok(0xAC75_0000_0000_0000 + n)
}
fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err("file exceeds declared size limit".into());
    }
    Ok(bytes)
}
fn key(path: &Path) -> Result<KeyVault> {
    let mut bytes = read_bounded(path, 32)?;
    if bytes.len() != 32 {
        return Err("key file must contain one 32-byte scalar".into());
    }
    let result = mudra::SigningKey::from_slice(&bytes).map_err(|_| "invalid signing key");
    bytes.fill(0);
    Ok(KeyVault::new(result?))
}
fn keygen(path: &Path) -> Result<Value> {
    let mut bytes = nonce(None)?;
    let signing = mudra::SigningKey::from_slice(&bytes)
        .map_err(|_| "random scalar rejected; retry keygen")?;
    let vault = KeyVault::new(signing);
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    let written = file.write_all(&bytes).and_then(|_| file.sync_all());
    bytes.fill(0);
    written?;
    Ok(json!({"neuron":hex(vault.subject()),"key_file":path,"profile":"native-secp256k1-pubkey/1"}))
}
fn inspection(agent: &Agent, id: Particle, selected: Option<Particle>) -> Result<Value> {
    let view = agent.inspect(id)?;
    let mut progs = Vec::new();
    let mut selected_state = None;
    for (prog, p) in &view.state.progs {
        let state = neuron_rune::display(&agent.state(id, *prog)?)?;
        if selected == Some(*prog) {
            selected_state = Some(state.clone());
        }
        progs.push(json!({"prog":hex(*prog),"revision":p.revision,"lifecycle":p.lifecycle.name(),"state":state}));
    }
    if selected.is_some() && selected_state.is_none() {
        return Err("program is not installed under this neuron".into());
    }
    let mut invocations = Vec::new();
    let mut r = Reader::new(&agent.graph, 2_000_000);
    for (inv, j) in &view.state.invocations {
        let fault = j
            .fault
            .map(|f| read_artifact(&mut r, f))
            .transpose()?
            .map(String::from_utf8)
            .transpose()?;
        invocations.push(json!({"invocation":hex(*inv),"prog":hex(j.prog),"status":j.status.name(),"context":j.context.map(hex),
            "charged_steps":j.charged,"reserved_steps":j.reserved,"allowance":j.limit,"delegated_steps":j.delegated,"epoch":j.epoch,
            "parent":j.parent.map(hex),"fault":fault,"pending":j.pending.as_ref().map(|p|json!({"operation":hex(p.id),"attempt":p.attempt.map(hex),
                "tag":p.tag,"stage":match p.stage{0=>"awaiting-executor",1=>"unknown-outcome",2=>"resolved",3=>"awaiting-event",4=>"event-resolved",_=>"invalid"}}))}));
    }
    Ok(
        json!({"neuron":hex(id),"network":hex(view.state.network),"epoch":view.state.epoch,
        "head":{"index":view.head.index,"commit":hex(view.head.commit)},"progs":progs,"invocations":invocations,
        "budget":{"limit":view.state.limit,"charged":view.state.charged,"held":view.state.held},
        "state":selected_state,"prog":selected.map(hex),"durability":"LocalDurable","finality":"LocalAuthority",
        "profile":"native-secp256k1-local/1","imports":view.state.imports.keys().copied().map(hex).collect::<Vec<_>>()}),
    )
}
fn run(agent: &Agent, id: Particle, selected: Option<Particle>) -> Result<Value> {
    if selected.is_some_and(|inv| {
        agent
            .inspect(id)
            .is_ok_and(|v| !v.state.invocations.contains_key(&inv))
    }) {
        let mut result = inspection(agent, id, None)?;
        result["status"] = json!("not-retained");
        result["invocation"] = json!(selected.map(hex));
        return Ok(result);
    }
    let mut emitted = Vec::new();
    let mut worker = None;
    for _ in 0..2000 {
        let progress = match selected {
            Some(inv) => agent.tick_invocation(id, inv)?,
            None => agent.tick(id)?,
        };
        match progress {
            JobProgress::Yielded { .. } => continue,
            JobProgress::Awaiting {
                invocation,
                tag: 0xAC75_0000_0000_0001,
                ..
            } => {
                if worker.is_none() {
                    let view = agent.inspect(id)?;
                    let boot = nonce(None)?;
                    let worker_id = Builder::new().blob(b"neuron-cli/emit-worker/1".to_vec())?;
                    let descriptor = neuron_node::LocalWorker::descriptor(
                        view.state.network,
                        worker_id,
                        boot,
                        boot,
                        vec![0xAC75_0000_0000_0001],
                        8 * 1024 * 1024,
                    );
                    worker = Some(neuron_node::LocalWorker::bind(
                        agent,
                        id,
                        view.state.writer_generation,
                        descriptor,
                    )?);
                }
                let worker = worker.as_ref().ok_or("missing local worker")?;
                let token = worker.prepare(agent, id, invocation)?;
                let display = neuron_rune::display(&token.metadata().arguments)?;
                let value = neuron_rune::value("0")?;
                let execution = worker.execute(agent, token, |d| {
                    emitted.push(json!({"operation":hex(d.operation),"value":display}));
                    neuron_node::EffectOutcome::Value(value)
                })?;
                execution
                    .reconciliation
                    .ok_or("unknown local emit result")??;
                continue;
            }
            _ => {}
        }
        let view = agent.inspect(id)?;
        let inv = match progress {
            JobProgress::Idle => selected,
            JobProgress::Yielded { invocation, .. }
            | JobProgress::Awaiting { invocation, .. }
            | JobProgress::Unknown { invocation, .. }
            | JobProgress::Event { invocation, .. }
            | JobProgress::Finished { invocation, .. } => Some(invocation),
        };
        let prog = inv
            .and_then(|i| view.state.invocations.get(&i))
            .map(|j| j.prog);
        let mut result = inspection(agent, id, prog)?;
        result["invocation"] = json!(inv.map(hex));
        result["emitted"] = json!(emitted);
        match progress {
            JobProgress::Awaiting { operation, tag, .. } => {
                result["status"] = json!("awaiting-executor");
                result["operation"] = json!(hex(operation));
                result["tag"] = json!(tag);
            }
            JobProgress::Unknown {
                operation, attempt, ..
            } => {
                result["status"] = json!("unknown-outcome");
                result["operation"] = json!(hex(operation));
                result["attempt"] = json!(hex(attempt));
            }
            JobProgress::Event {
                operation,
                tag,
                selector,
                ..
            } => {
                result["status"] = json!("awaiting-event");
                result["operation"] = json!(hex(operation));
                result["tag"] = json!(tag);
                result["selector"] = json!(neuron_rune::display(&selector)?);
            }
            JobProgress::Finished { status, .. } => result["status"] = json!(status.name()),
            _ => result["status"] = json!("idle"),
        }
        return Ok(result);
    }
    Err("host slice limit reached; run the same invocation to continue".into())
}
fn history(
    graph: &Graph,
    id: Particle,
    after: Option<u64>,
    limit: usize,
    legacy: bool,
) -> Result<Value> {
    let mut entries = Vec::new();
    for head in graph.history(id, after, limit)? {
        let mut r = Reader::new(graph, 50_000);
        let mut events = Vec::new();
        if head.index > 0 {
            if legacy {
                let f = r.record(head.commit, "cell/commit/1", 10)?;
                for event in r.list(f[5], 256)? {
                    let event = r.reference(event)?;
                    let f = r.record(event, "cell/event/1", 10)?;
                    events.push(json!({"event":hex(event),"entry":r.text(f[3])?,"legacy_author":hex(r.reference(f[0])?)}));
                }
            } else {
                let f = r.record(head.commit, "neuron/commit/1", 8)?;
                events.push(json!({"event":hex(r.reference(f[5])?),"neuron":hex(id)}));
            }
        }
        entries.push(json!({"index":head.index,"commit":hex(head.commit),"events":events}));
    }
    Ok(
        json!({"namespace":hex(id),"provenance":if legacy{"legacy-cell-v1"}else{"neuron-v1"},"history":entries}),
    )
}
fn execute(args: Args) -> Result<Value> {
    match &args.command {
        Command::LegacySourceInspect {
            source,
            destination,
            max_rows,
            max_bytes,
        } => return archive::inspect(source, destination.as_deref(), *max_rows, *max_bytes),
        Command::LegacyExport {
            source,
            destination,
            target,
            nonce,
            pages,
        } => return archive::export(source, destination, target, nonce, *pages),
        Command::Keygen { file } => return keygen(file),
        Command::Identity => {
            let vault = key(args.key_file.as_deref().ok_or("select --key-file")?)?;
            return Ok(
                json!({"neuron":hex(vault.subject()),"profile":"native-secp256k1-pubkey/1"}),
            );
        }
        #[cfg(feature = "legacy-redb-migration")]
        Command::MigrateRedb {
            source,
            destination,
        } => {
            Graph::migrate_redb(source, destination)?;
            return Ok(json!({"status":"migrated","source":source,"destination":destination}));
        }
        _ => {}
    }
    let store = match args.store {
        Some(path) => path,
        None => {
            if Path::new("cell.redb").try_exists()? {
                return Err(
                    "legacy cell.redb exists; use migrate-redb or explicitly select --store".into(),
                );
            }
            PathBuf::from("bbg")
        }
    };
    let graph = Graph::open(store)?;
    let view = args.command.subject().map(particle).transpose()?.map(|id| {
        // Missing subjects are handled by the command; corruption must propagate.
        (id, graph.head(id))
    });
    let mut root = None;
    if let Some((id, result)) = view
        && result?.is_some()
    {
        root = Some(Neuron::new(&graph, Rune).inspect(id)?.state);
    }
    let network = match args.network {
        Some(n) => particle(&n)?,
        None => root
            .as_ref()
            .map(|s| s.network)
            .unwrap_or(named("cyber:local:network/1")?),
    };
    let policy = root
        .as_ref()
        .map(|s| s.policy)
        .unwrap_or(named("neuron:local-owner-policy/1")?);
    let epoch = root.as_ref().map(|s| s.epoch).unwrap_or(0);
    let mut subject = None;
    let authority = if let Some(path) = args.key_file {
        let vault = key(&path)?;
        subject = Some(vault.subject());
        let grant = GrantHandle::new(Grant {
            neuron: vault.subject(),
            network,
            policy,
            epoch,
            revision: 0,
            enabled: true,
            acts: args
                .grant_act
                .iter()
                .map(|s| act(s))
                .collect::<Result<_>>()?,
            progs: None,
        })?;
        Some(LocalAuthority::new(vault, grant)?)
    } else {
        None
    };
    let agent = Neuron::with_authority(graph, Rune, CliAuthority(authority));
    match args.command {
        Command::Activate { budget } => {
            let id = subject.ok_or("activate requires --key-file")?;
            agent.activate(id, network, policy, budget)?;
            inspection(&agent, id, None)
        }
        Command::Install {
            neuron,
            source,
            initial,
            nonce: n,
            steps,
            inflight,
            allow,
        } => {
            let id = particle(&neuron)?;
            let n = nonce(n)?;
            let prog = agent.install(
                id,
                n,
                read_bounded(&source, 8192)?,
                neuron_rune::value(&initial)?,
                ProgramConfig {
                    step_limit: steps,
                    max_inflight: inflight,
                    allowed_acts: allow.iter().map(|s| act(s)).collect::<Result<_>>()?,
                },
            )?;
            let mut out = inspection(&agent, id, Some(prog))?;
            out["install_nonce"] = json!(hex(n));
            Ok(out)
        }
        Command::Submit {
            neuron,
            prog,
            input,
            nonce: n,
            context,
            parent,
            steps,
            queue_only,
        } => {
            let id = particle(&neuron)?;
            let prog = particle(&prog)?;
            let n = nonce(n)?;
            let view = agent.inspect(id)?;
            let p = view.state.progs.get(&prog).ok_or("program not found")?;
            let receipt = agent.submit(
                id,
                Admission {
                    prog,
                    nonce: n,
                    input: neuron_rune::value(&input)?,
                    context: context.map(|v| particle(&v)).transpose()?,
                    parent: parent.map(|v| particle(&v)).transpose()?,
                    allowance: steps.unwrap_or(p.step_limit),
                },
            )?;
            let mut out = if queue_only {
                inspection(&agent, id, Some(prog))?
            } else {
                run(&agent, id, Some(receipt.invocation))?
            };
            out["invocation"] = json!(hex(receipt.invocation));
            out["request_nonce"] = json!(hex(n));
            out["admission_commit"] = json!(hex(receipt.head.commit));
            Ok(out)
        }
        Command::Run { neuron, invocation } => run(
            &agent,
            particle(&neuron)?,
            invocation.map(|s| particle(&s)).transpose()?,
        ),
        Command::Inspect { neuron, prog } => inspection(
            &agent,
            particle(&neuron)?,
            prog.map(|s| particle(&s)).transpose()?,
        ),
        Command::History {
            neuron,
            after,
            limit,
        } => history(&agent.graph, particle(&neuron)?, after, limit, false),
        Command::Take { neuron, invocation } => {
            let d = agent.begin_attempt(particle(&neuron)?, particle(&invocation)?)?;
            Ok(
                json!({"neuron":hex(d.neuron),"network":hex(d.network),"prog":hex(d.prog),"invocation":hex(d.invocation),
                "operation":hex(d.operation),"attempt":hex(d.attempt),"tag":d.tag,"arguments":neuron_rune::display(&d.arguments)?,
                "authorization":hex(d.authorization),"epoch":d.epoch}),
            )
        }
        Command::Outcome {
            neuron,
            invocation,
            operation,
            attempt,
            value,
        } => {
            let id = particle(&neuron)?;
            let inv = particle(&invocation)?;
            agent.record_outcome(
                id,
                inv,
                particle(&operation)?,
                particle(&attempt)?,
                neuron_rune::value(&value)?,
            )?;
            let view = agent.inspect(id)?;
            let prog = view.state.invocations.get(&inv).map(|j| j.prog);
            if prog.is_some_and(|p| view.state.progs[&p].lifecycle == Lifecycle::Active) {
                run(&agent, id, Some(inv))
            } else {
                inspection(&agent, id, prog)
            }
        }
        Command::Fail {
            neuron,
            invocation,
            operation,
            attempt,
            reason,
        } => {
            let id = particle(&neuron)?;
            agent.record_failure(
                id,
                particle(&invocation)?,
                particle(&operation)?,
                particle(&attempt)?,
                reason,
            )?;
            inspection(&agent, id, None)
        }
        Command::Wake {
            neuron,
            invocation,
            operation,
            value,
            nonce,
        } => {
            let id = particle(&neuron)?;
            let inv = particle(&invocation)?;
            agent.wake(
                id,
                inv,
                particle(&operation)?,
                particle(&nonce)?,
                neuron_rune::value(&value)?,
            )?;
            run(&agent, id, Some(inv))
        }
        Command::Pause { neuron, prog } => {
            let id = particle(&neuron)?;
            let prog = particle(&prog)?;
            agent.manage(id, prog, Lifecycle::Paused)?;
            inspection(&agent, id, Some(prog))
        }
        Command::Resume { neuron, prog } => {
            let id = particle(&neuron)?;
            let prog = particle(&prog)?;
            agent.manage(id, prog, Lifecycle::Active)?;
            inspection(&agent, id, Some(prog))
        }
        Command::Cancel {
            neuron,
            invocation,
            reason,
        } => {
            let id = particle(&neuron)?;
            agent.cancel(id, particle(&invocation)?, reason)?;
            inspection(&agent, id, None)
        }
        Command::Retire { neuron, prog } => {
            let id = particle(&neuron)?;
            let prog = particle(&prog)?;
            agent.manage(id, prog, Lifecycle::Retiring)?;
            agent.manage(id, prog, Lifecycle::Retired)?;
            inspection(&agent, id, Some(prog))
        }
        Command::Upgrade {
            neuron,
            prog,
            source,
            state,
        } => {
            let id = particle(&neuron)?;
            let prog = particle(&prog)?;
            agent.upgrade(
                id,
                prog,
                read_bounded(&source, 8192)?,
                neuron_rune::value(&state)?,
            )?;
            inspection(&agent, id, Some(prog))
        }
        Command::Archive { neuron, invocation } => {
            let id = particle(&neuron)?;
            agent.archive(id, particle(&invocation)?)?;
            inspection(&agent, id, None)
        }
        Command::Import {
            neuron,
            origins,
            nonce,
        } => {
            let mappings = origins
                .iter()
                .map(|s| {
                    let (origin, n) = s
                        .split_once('=')
                        .ok_or("mapping must be ORIGIN=INSTALL_NONCE")?;
                    Ok(ImportOrigin {
                        origin: particle(origin)?,
                        install_nonce: particle(n)?,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let result = agent.import(particle(&neuron)?, particle(&nonce)?, mappings)?;
            Ok(
                json!({"neuron":neuron,"manifest":hex(result.manifest),"commit":hex(result.head.commit),"mappings":result.mappings.iter().map(|m|
                json!({"origin":hex(m.origin),"prog":hex(m.prog),"invocation":m.invocation.map(hex),"incompatibility":m.incompatibility})).collect::<Vec<_>>()}),
            )
        }
        Command::LegacyInspect { origin } => {
            let view = neuron_engine::legacy::inspect(&agent.graph, particle(&origin)?)?;
            let state =
                Reader::new(&agent.graph, 50_000).artifact(view.snapshot.application_state)?;
            Ok(
                json!({"origin":origin,"provenance":"legacy-cell-v1","head":{"index":view.head.index,"commit":hex(view.head.commit)},
                "state":neuron_rune::display(&state)?,"charged":view.charged,"has_continuation":view.live.is_some()}),
            )
        }
        Command::StageImport {
            neuron,
            source,
            nonce,
            pages,
        } => {
            let result = neuron_node::stage_legacy(
                &agent,
                particle(&neuron)?,
                &source,
                particle(&nonce)?,
                pages,
            )?;
            Ok(
                json!({"manifest":hex(result.manifest),"rows":result.rows,"bytes":result.bytes,
                "complete":result.complete,"source_sealed":true,"neuron":neuron}),
            )
        }
        Command::LegacyHistory {
            origin,
            after,
            limit,
        } => history(&agent.graph, particle(&origin)?, after, limit, true),
        _ => Err("command is unavailable in this composition".into()),
    }
}
fn main() {
    match execute(Args::parse()) {
        Ok(value) => println!("{value}"),
        Err(e) => {
            eprintln!("{}", json!({"error":e.to_string()}));
            std::process::exit(1)
        }
    }
}
