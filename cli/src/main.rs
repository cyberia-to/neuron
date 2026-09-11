use cell_engine::{Config, Engine, GraphPort, Progress};
use cell_model::{Lifecycle, Particle, Reader};
use cell_node::{Graph, Rune};
use clap::{Parser, Subcommand};
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "cell",
    version,
    about = "Local stateful rune abilities with durable graph history"
)]
struct Args {
    #[arg(long, default_value = "cell.redb", global = true)]
    store: PathBuf,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Create {
        source: PathBuf,
        #[arg(long, default_value = "0")]
        initial: String,
        #[arg(long)]
        nonce: Option<String>,
        #[arg(long, default_value_t = 1_000_000)]
        steps: u64,
        #[arg(long)]
        allow: Vec<String>,
    },
    Submit {
        cell: String,
        input: String,
        #[arg(long)]
        nonce: Option<String>,
        #[arg(long)]
        context: Option<String>,
        #[arg(long)]
        queue_only: bool,
    },
    Run {
        cell: String,
    },
    Inspect {
        cell: String,
    },
    History {
        cell: String,
        #[arg(long)]
        after: Option<u64>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    Take {
        cell: String,
        operation: String,
    },
    Outcome {
        cell: String,
        operation: String,
        attempt: String,
        value: String,
    },
    Fail {
        cell: String,
        operation: String,
        attempt: String,
        reason: String,
    },
    Pause {
        cell: String,
    },
    Resume {
        cell: String,
    },
    Cancel {
        cell: String,
    },
    Retire {
        cell: String,
    },
}
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn particle(text: &str) -> Result<Particle> {
    if text.len() != 64 || !text.is_ascii() {
        return Err("expected 64 hexadecimal characters".into());
    }
    let mut id = [0; 32];
    for (i, byte) in id.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16)?;
    }
    Ok(id)
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
fn act(name: &str) -> Result<u64> {
    let ordinal = match name {
        "emit" => 1,
        "query" => 2,
        "link" => 3,
        "seal" => 4,
        "subscribe" => 5,
        "host" => 6,
        _ => return Err("unknown act; use emit/query/link/seal/subscribe/host".into()),
    };
    Ok(0xAC75_0000_0000_0000 + ordinal)
}
fn inspection(engine: &Engine<Graph, Rune>, cell: Particle) -> Result<Value> {
    let view = engine.inspect(cell)?;
    Ok(
        json!({ "cell": hex(cell), "head": {"index": view.head.index, "commit": hex(view.head.commit)},
        "lifecycle": view.snapshot.lifecycle.name(), "state": cell_rune::display(&view.state)?,
        "durability": "LocalDurable", "finality": "LocalAuthority",
        "profile": "local-experimental/1", "fault": view.fault,
        "invocation": view.live.as_ref().map(|v| json!({"event": hex(v.event), "context": v.context.map(hex),
            "charged_steps": v.charged, "reserved_steps": v.reserved, "step_limit": v.limit,
            "pending": v.pending.as_ref().map(|p| json!({"operation": hex(p.id), "tag": p.tag, "stage": if p.stage == "attempt-recorded" {"unknown"} else {p.stage}})) })) }),
    )
}
fn run(engine: &Engine<Graph, Rune>, cell: Particle) -> Result<Value> {
    let mut emitted = Vec::new();
    for _ in 0..2000 {
        match engine.tick(cell)? {
            Progress::Advanced(_) => (),
            Progress::Complete(_) | Progress::Idle => {
                let mut result = inspection(engine, cell)?;
                result["emitted"] = json!(emitted);
                return Ok(result);
            }
            Progress::Awaiting {
                operation,
                tag: 0xAC75_0000_0000_0001,
                ..
            } => {
                let receipt = engine.begin_attempt(cell, operation)?;
                engine.record_outcome(cell, operation, receipt.attempt, cell_rune::value("0")?)?;
                emitted.push(json!({"operation": hex(operation), "value": cell_rune::display(&receipt.arguments)?}));
            }
            Progress::Awaiting {
                operation,
                tag,
                arguments,
            } => {
                return Ok(
                    json!({"cell": hex(cell), "status": "awaiting-executor", "operation": hex(operation), "tag": tag, "arguments": cell_rune::display(&arguments)?}),
                );
            }
            Progress::Unknown { operation, attempt } => {
                return Ok(
                    json!({"cell": hex(cell), "status": "unknown-outcome", "operation": hex(operation), "attempt": hex(attempt)}),
                );
            }
        }
    }
    Err("host slice limit reached; run the same cell again to continue".into())
}
fn execute(args: Args) -> Result<Value> {
    let graph = Graph::open(&args.store)?;
    let engine = Engine::with_ward(graph, Rune, cell_node::LocalWard);
    match args.command {
        Command::Create {
            source,
            initial,
            nonce: n,
            steps,
            allow,
        } => {
            if std::fs::metadata(&source)?.len() > 8192 {
                return Err("source exceeds 8192 bytes".into());
            }
            let nonce = nonce(n)?;
            let config = Config {
                step_limit: steps,
                allowed_acts: allow.iter().map(|v| act(v)).collect::<Result<Vec<_>>>()?,
            };
            let cell = engine.create(
                std::fs::read(source)?,
                cell_rune::value(&initial)?,
                nonce,
                config,
            )?;
            let mut result = inspection(&engine, cell)?;
            result["birth_nonce"] = json!(hex(nonce));
            Ok(result)
        }
        Command::Submit {
            cell,
            input,
            nonce: n,
            context,
            queue_only,
        } => {
            let cell = particle(&cell)?;
            let nonce = nonce(n)?;
            let context = context.map(|v| particle(&v)).transpose()?;
            engine.submit(cell, nonce, cell_rune::value(&input)?, context)?;
            let mut result = if queue_only {
                inspection(&engine, cell)?
            } else {
                run(&engine, cell)?
            };
            result["request_nonce"] = json!(hex(nonce));
            Ok(result)
        }
        Command::Run { cell } => run(&engine, particle(&cell)?),
        Command::Inspect { cell } => inspection(&engine, particle(&cell)?),
        Command::History { cell, after, limit } => {
            let cell = particle(&cell)?;
            let mut entries = Vec::new();
            for head in engine.graph.history(cell, after, limit)? {
                let mut events = Vec::new();
                if head.index > 0 {
                    let mut r = Reader::new(&engine.graph, 20_000);
                    let c = r.record(head.commit, "cell/commit/1", 10)?;
                    for event in r.list(c[5], 256)? {
                        let id = r.reference(event)?;
                        let fields = r.record(id, "cell/event/1", 10)?;
                        events.push(json!({"event": hex(id), "entry": r.text(fields[3])?, "payload": hex(r.reference(fields[4])?), "context": r.optional_ref(fields[5])?.map(hex)}));
                    }
                }
                entries.push(
                    json!({"index": head.index, "commit": hex(head.commit), "events": events}),
                );
            }
            Ok(json!({"cell": hex(cell), "history": entries}))
        }
        Command::Take { cell, operation } => {
            let receipt = engine.begin_attempt(particle(&cell)?, particle(&operation)?)?;
            Ok(
                json!({"operation": hex(receipt.operation), "attempt": hex(receipt.attempt), "tag": receipt.tag, "arguments": cell_rune::display(&receipt.arguments)?}),
            )
        }
        Command::Outcome {
            cell,
            operation,
            attempt,
            value,
        } => {
            let cell = particle(&cell)?;
            engine.record_outcome(
                cell,
                particle(&operation)?,
                particle(&attempt)?,
                cell_rune::value(&value)?,
            )?;
            if engine.inspect(cell)?.snapshot.lifecycle == Lifecycle::Active {
                run(&engine, cell)
            } else {
                inspection(&engine, cell)
            }
        }
        Command::Fail {
            cell,
            operation,
            attempt,
            reason,
        } => {
            let cell = particle(&cell)?;
            engine.record_failure(cell, particle(&operation)?, particle(&attempt)?, reason)?;
            inspection(&engine, cell)
        }
        Command::Pause { cell } => {
            let cell = particle(&cell)?;
            engine.manage(cell, Lifecycle::Paused)?;
            inspection(&engine, cell)
        }
        Command::Resume { cell } => {
            let cell = particle(&cell)?;
            engine.manage(cell, Lifecycle::Active)?;
            run(&engine, cell)
        }
        Command::Cancel { cell } => {
            let cell = particle(&cell)?;
            engine.cancel(cell)?;
            inspection(&engine, cell)
        }
        Command::Retire { cell } => {
            let cell = particle(&cell)?;
            engine.manage(cell, Lifecycle::Retiring)?;
            engine.manage(cell, Lifecycle::Retired)?;
            inspection(&engine, cell)
        }
    }
}
fn main() {
    match execute(Args::parse()) {
        Ok(value) => println!("{value}"),
        Err(error) => {
            eprintln!("{}", json!({"error": error.to_string()}));
            std::process::exit(1);
        }
    }
}
