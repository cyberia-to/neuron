//! Bounded URI adapter over subject/data references. Never authorizes execution.
use crate::{Destination, IdentityError as Error, NetworkRef, SubjectRef};
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Route {
    pub destination: Destination,
    pub network: Option<NetworkRef>,
}
type Result<T> = std::result::Result<T, Error>;
impl Route {
    pub fn view(name: impl Into<String>) -> Result<Self> {
        let route = Self {
            destination: Destination::View(name.into()),
            network: None,
        };
        route.validate()?;
        Ok(route)
    }
    pub fn validate(&self) -> Result<()> {
        self.destination.validate()?;
        if let Some(network) = &self.network {
            network.validate()?;
            let subject = match &self.destination {
                Destination::Neuron(s) | Destination::Prog { subject: s, .. } => s,
                _ => return Err(Error::InvalidReference),
            };
            match (subject, network) {
                (SubjectRef::Native(_), NetworkRef::Native(_)) => {}
                (SubjectRef::Foreign { domain: a, .. }, NetworkRef::Foreign { domain: b, .. })
                    if a == b => {}
                _ => return Err(Error::InvalidReference),
            }
        }
        Ok(())
    }
    /// Only the supplied resolver can give meaning to a retired cell URI.
    pub fn parse(input: &str, legacy: impl FnOnce(&str) -> Option<Self>) -> Result<Self> {
        if input.len() > 4096 || input.contains('#') {
            return Err(Error::InvalidReference);
        }
        if let Some(id) = input.strip_prefix("cell://") {
            if id.is_empty()
                || id.len() > 256
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
            {
                return Err(Error::InvalidReference);
            }
            let route = legacy(id).ok_or(Error::Missing)?;
            route.validate()?;
            return Ok(route);
        }
        let input = input
            .strip_prefix("cyb://")
            .ok_or(Error::InvalidReference)?;
        let (path, query) = match input.split_once('?') {
            Some((p, q)) => (p, Some(q)),
            None => (input, None),
        };
        let parts: Vec<_> = path.split('/').collect();
        let destination = match parts.as_slice() {
            ["view", name] => Destination::View(segment(name)?),
            ["particle", id] => Destination::Particle(id32(id)?),
            ["neuron", kind, tail @ ..] => Destination::Neuron(subject(kind, tail)?),
            ["prog", kind, tail @ ..] if !tail.is_empty() => Destination::Prog {
                subject: subject(kind, &tail[..tail.len() - 1])?,
                prog: id32(tail[tail.len() - 1])?,
            },
            _ => return Err(Error::InvalidReference),
        };
        let network = query
            .map(|q| {
                let parts: Vec<_> = q
                    .strip_prefix("network=")
                    .ok_or(Error::InvalidReference)?
                    .split('/')
                    .collect();
                match parts.as_slice() {
                    ["native", id] => Ok(NetworkRef::Native(id32(id)?)),
                    ["foreign", domain, network] => Ok(NetworkRef::Foreign {
                        domain: segment(domain)?,
                        network: unhex(network)?,
                    }),
                    _ => Err(Error::InvalidReference),
                }
            })
            .transpose()?;
        let route = Self {
            destination,
            network,
        };
        route.validate()?;
        Ok(route)
    }
    pub fn uri(&self) -> Result<String> {
        self.validate()?;
        let path = match &self.destination {
            Destination::View(name) => format!("view/{}", escape(name)),
            Destination::Particle(id) => format!("particle/{}", hex(id)),
            Destination::Neuron(s) => format!("neuron/{}", subject_uri(s)),
            Destination::Prog { subject, prog } => {
                format!("prog/{}/{}", subject_uri(subject), hex(prog))
            }
        };
        let query = match &self.network {
            None => String::new(),
            Some(NetworkRef::Native(id)) => format!("?network=native/{}", hex(id)),
            Some(NetworkRef::Foreign { domain, network }) => {
                format!("?network=foreign/{}/{}", escape(domain), hex(network))
            }
        };
        Ok(format!("cyb://{path}{query}"))
    }
}
fn subject(kind: &str, tail: &[&str]) -> Result<SubjectRef> {
    match (kind, tail) {
        ("native", [id]) => Ok(SubjectRef::Native(id32(id)?)),
        ("foreign", [domain, address]) => Ok(SubjectRef::Foreign {
            domain: segment(domain)?,
            address: unhex(address)?,
        }),
        _ => Err(Error::InvalidReference),
    }
}
fn subject_uri(subject: &SubjectRef) -> String {
    match subject {
        SubjectRef::Native(id) => format!("native/{}", hex(id)),
        SubjectRef::Foreign { domain, address } => {
            format!("foreign/{}/{}", escape(domain), hex(address))
        }
    }
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn nibble(b: u8) -> Result<u8> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(Error::InvalidReference),
    }
}
fn unhex(text: &str) -> Result<Vec<u8>> {
    if text.is_empty() || text.len() > 512 || !text.len().is_multiple_of(2) {
        return Err(Error::InvalidReference);
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|b| Ok(nibble(b[0])? * 16 + nibble(b[1])?))
        .collect()
}
fn id32(text: &str) -> Result<[u8; 32]> {
    unhex(text)?.try_into().map_err(|_| Error::InvalidReference)
}
fn escape(text: &str) -> String {
    text.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}
fn segment(text: &str) -> Result<String> {
    let mut bytes = Vec::new();
    let mut it = text.bytes();
    while let Some(b) = it.next() {
        bytes.push(if b == b'%' {
            nibble(it.next().ok_or(Error::InvalidReference)?)? * 16
                + nibble(it.next().ok_or(Error::InvalidReference)?)?
        } else {
            b
        });
    }
    String::from_utf8(bytes).map_err(|_| Error::InvalidReference)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn destinations_round_trip_without_changing_foreign_bytes_or_making_subjects() {
        let s = SubjectRef::Foreign {
            domain: "cosmos/bostrom".into(),
            address: b"bostrom1unchanged".to_vec(),
        };
        for route in [
            Route::view("robot").unwrap(),
            Route {
                destination: Destination::Particle([7; 32]),
                network: None,
            },
            Route {
                destination: Destination::Neuron(SubjectRef::Native([1; 32])),
                network: Some(NetworkRef::Native([2; 32])),
            },
            Route {
                destination: Destination::Prog {
                    subject: s,
                    prog: [3; 32],
                },
                network: Some(NetworkRef::Foreign {
                    domain: "cosmos/bostrom".into(),
                    network: vec![0, 255],
                }),
            },
        ] {
            assert_eq!(
                Route::parse(&route.uri().unwrap(), |_| panic!("not legacy")).unwrap(),
                route
            );
        }
    }
    #[test]
    fn retired_ids_require_exact_mapping_and_malformed_routes_fail() {
        assert_eq!(
            Route::parse("cell://landing", |id| (id == "landing")
                .then(|| Route::view("robot").unwrap()))
            .unwrap(),
            Route::view("robot").unwrap()
        );
        assert!(Route::parse("cell://unknown", |_| None).is_err());
        for uri in [
            "cell://../landing",
            "cell://landing?run=1",
            "cyb://view/robot?network=native/00",
            "cyb://view/%FF",
            "cyb://particle/0",
            "cyb://view/robot#run",
            "cyb://view/robot/tail",
        ] {
            assert!(Route::parse(uri, |_| None).is_err(), "{uri}");
        }
        let route = Route {
            destination: Destination::Neuron(SubjectRef::Native([1; 32])),
            network: Some(NetworkRef::Foreign {
                domain: "cosmos".into(),
                network: vec![1],
            }),
        };
        assert!(route.uri().is_err());
    }
}
