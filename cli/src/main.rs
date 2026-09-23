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
    #[arg(long, global = true, help = "BBG database directory (default: bbg)")]
    store: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    #[command(flatten)]
    Graph(GraphCommand),
    /// Import an old application redb file into a fresh BBG directory.
    #[cfg(feature = "legacy-redb-migration")]
    MigrateRedb {
        source: PathBuf,
        destination: PathBuf,
    },
}
#[derive(Subcommand)]
enum GraphCommand {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_round_trips_through_hex() {
        let id: Particle = std::array::from_fn(|i| i as u8);
        let text = hex(id);
        assert_eq!(text.len(), 64);
        assert_eq!(particle(&text).unwrap(), id);
        assert_eq!(text, text.to_ascii_lowercase());
    }

    #[test]
    fn particle_rejects_wrong_length() {
        assert!(particle("ab").is_err());
        assert!(particle(&"ab".repeat(31)).is_err());
        assert!(particle(&"ab".repeat(33)).is_err());
    }

    #[test]
    fn particle_rejects_non_hex_and_non_ascii_content() {
        assert!(particle(&"zz".repeat(32)).is_err());
        // a multi-byte UTF-8 character keeps the string's byte length at 64
        // while making every byte-index slice land mid-character; the
        // `is_ascii()` guard must reject it before any slicing runs.
        let mut text = "a".repeat(62);
        text.push('中');
        assert_eq!(text.len(), 65);
        assert!(particle(&text).is_err());
        let mut exact = "a".repeat(62);
        exact.push('é'); // 2-byte char, brings total byte length to 64
        assert_eq!(exact.len(), 64);
        assert!(particle(&exact).is_err());
    }

    #[test]
    fn nonce_uses_the_explicit_value_when_given() {
        let id: Particle = [7; 32];
        let text = hex(id);
        assert_eq!(nonce(Some(text)).unwrap(), id);
        assert!(nonce(Some("not hex".into())).is_err());
    }

    #[test]
    fn nonce_generates_a_fresh_random_particle_when_absent() {
        let a = nonce(None).unwrap();
        let b = nonce(None).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn act_maps_every_known_name_to_a_distinct_ordinal() {
        let names = ["emit", "query", "link", "seal", "subscribe", "host"];
        let values: Vec<u64> = names.iter().map(|n| act(n).unwrap()).collect();
        for v in &values {
            assert_eq!(v & 0xAC75_0000_0000_0000, 0xAC75_0000_0000_0000);
        }
        let mut sorted = values.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), values.len());
    }

    #[test]
    fn act_rejects_an_unknown_name() {
        assert!(act("delete").is_err());
        assert!(act("").is_err());
    }
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
    match args.command {
        Command::Graph(command) => execute_graph(args.store, command),
        #[cfg(feature = "legacy-redb-migration")]
        Command::MigrateRedb {
            source,
            destination,
        } => {
            Graph::migrate_redb(&source, &destination)?;
            Ok(json!({"source": source, "destination": destination, "status": "migrated"}))
        }
    }
}
fn execute_graph(store: Option<PathBuf>, command: GraphCommand) -> Result<Value> {
    let store = match store {
        Some(path) => path,
        None => {
            if std::path::Path::new("cell.redb").try_exists()? {
                return Err("legacy cell.redb exists; migrate it with a legacy-redb-migration build, or select a BBG directory explicitly with --store".into());
            }
            PathBuf::from("bbg")
        }
    };
    let graph = Graph::open(store)?;
    let engine = Engine::with_ward(graph, Rune, cell_node::LocalWard);
    match command {
        GraphCommand::Create {
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
        GraphCommand::Submit {
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
        GraphCommand::Run { cell } => run(&engine, particle(&cell)?),
        GraphCommand::Inspect { cell } => inspection(&engine, particle(&cell)?),
        GraphCommand::History { cell, after, limit } => {
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
        GraphCommand::Take { cell, operation } => {
            let receipt = engine.begin_attempt(particle(&cell)?, particle(&operation)?)?;
            Ok(
                json!({"operation": hex(receipt.operation), "attempt": hex(receipt.attempt), "tag": receipt.tag, "arguments": cell_rune::display(&receipt.arguments)?}),
            )
        }
        GraphCommand::Outcome {
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
        GraphCommand::Fail {
            cell,
            operation,
            attempt,
            reason,
        } => {
            let cell = particle(&cell)?;
            engine.record_failure(cell, particle(&operation)?, particle(&attempt)?, reason)?;
            inspection(&engine, cell)
        }
        GraphCommand::Pause { cell } => {
            let cell = particle(&cell)?;
            engine.manage(cell, Lifecycle::Paused)?;
            inspection(&engine, cell)
        }
        GraphCommand::Resume { cell } => {
            let cell = particle(&cell)?;
            engine.manage(cell, Lifecycle::Active)?;
            run(&engine, cell)
        }
        GraphCommand::Cancel { cell } => {
            let cell = particle(&cell)?;
            engine.cancel(cell)?;
            inspection(&engine, cell)
        }
        GraphCommand::Retire { cell } => {
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
