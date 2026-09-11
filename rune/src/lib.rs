//! Rune source/runtime adapter for the local cell profile.
use cell_engine::{Error, RuntimeInput, RuntimePort, RuntimeStep};
use rune_ast::{Expr, Noun};
use rune_interp::{
    codec,
    machine::{Limits, Machine, Step},
};

#[derive(Default)]
pub struct Rune;
fn runtime(e: impl std::fmt::Debug) -> Error {
    Error::Runtime(format!("{e:?}"))
}
fn limits(steps: u64) -> Limits {
    Limits {
        steps,
        ..Limits::default()
    }
}
pub fn encode(value: &Noun) -> Result<Vec<u8>, Error> {
    codec::encode(value, 65_536, 128).map_err(runtime)
}
pub fn decode(bytes: &[u8]) -> Result<Noun, Error> {
    codec::decode(bytes, 65_536, 128).map_err(runtime)
}
pub fn display(bytes: &[u8]) -> Result<String, Error> {
    fn write(value: &Noun, out: &mut String) {
        match value {
            Noun::Atom(n) => out.push_str(&n.to_string()),
            Noun::Cell(h, t) => {
                out.push('[');
                write(h, out);
                out.push(' ');
                write(t, out);
                out.push(']');
            }
        }
    }
    let mut out = String::new();
    write(&decode(bytes)?, &mut out);
    Ok(out)
}
fn expression(source: &[u8]) -> Result<Expr, Error> {
    if source.len() > 8192 {
        return Err(Error::Budget);
    }
    let source = std::str::from_utf8(source).map_err(runtime)?;
    rune_parse::parse_bounded(source, 512).map_err(runtime)
}
fn literal(value: Noun) -> Expr {
    match value {
        Noun::Atom(n) => Expr::Atom(n),
        Noun::Cell(h, t) => Expr::Cell(Box::new(literal(*h)), Box::new(literal(*t))),
    }
}
impl RuntimePort for Rune {
    fn validate_value(&self, bytes: &[u8]) -> Result<(), Error> {
        decode(bytes).map(|_| ())
    }
    fn start(&self, input: RuntimeInput<'_>) -> Result<Vec<u8>, Error> {
        let mut subject = rune_subject::Subject::minimal();
        subject.mem = decode(input.state)?;
        if let Some(context) = input.context {
            let mut values = Vec::new();
            for bytes in context.chunks_exact(8) {
                values.push(Noun::Atom(u64::from_le_bytes(
                    bytes.try_into().map_err(runtime)?,
                )));
            }
            subject.here = Noun::cell(
                Noun::cell(values[0].clone(), values[1].clone()),
                Noun::cell(values[2].clone(), values[3].clone()),
            );
        }
        let body = Expr::Let {
            name: "event".into(),
            mold: None,
            value: Box::new(literal(decode(input.event)?)),
            body: Box::new(expression(input.source)?),
        };
        let formula = rune_lower::lower_bounded(body, 2048).map_err(runtime)?;
        Machine::new(subject.to_noun(), formula, limits(input.step_limit))
            .map_err(runtime)?
            .checkpoint()
            .map_err(runtime)
    }
    fn step(
        &self,
        checkpoint: &[u8],
        reply: Option<&[u8]>,
        slice: u64,
        total: u64,
    ) -> Result<RuntimeStep, Error> {
        let mut machine = Machine::restore(checkpoint, limits(total)).map_err(runtime)?;
        if let Some(bytes) = reply {
            machine.resume(decode(bytes)?).map_err(runtime)?;
        }
        let step = machine.run(slice).map_err(runtime)?;
        let used = machine.used_steps();
        Ok(match step {
            Step::Done(value) => RuntimeStep::Done {
                result: encode(&value)?,
                used,
            },
            Step::Yield => RuntimeStep::Yield {
                checkpoint: machine.checkpoint().map_err(runtime)?,
                used,
            },
            Step::Act { tag, arguments } => RuntimeStep::Act {
                tag,
                arguments: encode(&arguments)?,
                checkpoint: machine.checkpoint().map_err(runtime)?,
                used,
            },
            Step::Event { tag, selector } => RuntimeStep::Event {
                tag,
                selector: encode(&selector)?,
                checkpoint: machine.checkpoint().map_err(runtime)?,
                used,
            },
        })
    }
}
pub fn value(source: &str) -> Result<Vec<u8>, Error> {
    let formula =
        rune_lower::lower_bounded(expression(source.as_bytes())?, 2048).map_err(runtime)?;
    let mut machine = Machine::new(Noun::Atom(0), formula, limits(10_000)).map_err(runtime)?;
    match machine.run(10_001).map_err(runtime)? {
        Step::Done(value) => encode(&value),
        _ => Err(Error::Denied),
    }
}
