//! Immutable v1 record readers and original profile identities.
use crate::Error;
use neuron_model::{Builder, Head, Particle, Reader, Snapshot, Source, Value::*};

pub fn named(b: &mut Builder, name: &str) -> Result<Particle, Error> {
    Ok(b.blob(name.as_bytes().to_vec())?)
}

pub fn authority(b: &mut Builder) -> Result<Particle, Error> {
    named(b, "cell/local-private-database-owner/1")
}

pub fn policy_values<S: Source>(
    r: &mut Reader<'_, S>,
    policy: Particle,
) -> Result<(u64, Vec<u64>), Error> {
    let f = r.record(policy, "cell/local-policy/1", 3)?;
    let tags = r
        .list(f[2], 256)?
        .into_iter()
        .map(|v| r.uint(v))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((r.uint(f[1])?, tags))
}

pub fn profile(b: &mut Builder) -> Result<Particle, Error> {
    let kind = b.variant("profile/runtime", &[])?;
    let rule = named(
        b,
        "cell/local-runtime/1:exclusive-private-store;local-durable;host-observed;full-history",
    )?;
    Ok(b.values(
        "cell/profile/1",
        &[
            Raw(kind),
            Ref(rule),
            Ref(rule),
            Ref(rule),
            Ref(rule),
            Ref(rule),
            Ref(rule),
            Ref(rule),
            Refs(&[]),
        ],
    )?)
}

pub fn snapshot_at<S: Source>(
    r: &mut Reader<'_, S>,
    head: Head,
) -> Result<(Particle, Snapshot), Error> {
    let fields = if head.index == 0 {
        r.record(head.commit, "cell/birth/1", 7)?
    } else {
        r.record(head.commit, "cell/commit/1", 10)?
    };
    let id = r.reference(fields[if head.index == 0 { 5 } else { 4 }])?;
    Ok((id, Snapshot::read(r, id)?))
}

#[derive(Debug, Clone)]
pub struct Pending {
    pub record: Particle,
    pub id: Particle,
    pub tag: u64,
    pub arguments: Particle,
    pub attempt: Option<Particle>,
    pub outcome: Option<Particle>,
    pub stage: &'static str,
}

#[derive(Debug, Clone)]
pub struct Live {
    pub event: Particle,
    pub context: Option<Particle>,
    pub checkpoint: Particle,
    pub charged: u64,
    pub reserved: u64,
    pub used: u64,
    pub limit: u64,
    pub pending: Option<Pending>,
}

pub fn read_live<S: Source>(
    r: &mut Reader<'_, S>,
    snapshot: &Snapshot,
) -> Result<Option<Live>, Error> {
    let entries = r.map(snapshot.continuations, 1)?;
    let Some((_, id)) = entries.first() else {
        return Ok(None);
    };
    let c = r.record(*id, "cell/continuation/1", 9)?;
    let resources = r.reference(c[8])?;
    let budget = r.record(resources, "cell/local-resources/1", 4)?;
    let pending = match r.map(snapshot.outbox, 1)?.first() {
        None => None,
        Some((id, entry)) => {
            let out = r.record(*entry, "cell/outbox-entry/1", 5)?;
            let record = r.reference(out[0])?;
            let op = r.record(record, "cell/operation/1", 11)?;
            let tag = r.reference(op[3])?;
            let tag = r.record(tag, "rune/act/1", 1)?;
            let tag = r.uint(tag[0])?;
            let stage = ["pending", "attempt-recorded", "resolved"]
                .into_iter()
                .find(|s| {
                    Builder::new()
                        .variant(&format!("operation-stage/{s}"), &[])
                        .ok()
                        == Some(out[1])
                })
                .ok_or(Error::Unsupported)?;
            let attempts = r.list(out[2], 1)?;
            Some(Pending {
                record,
                id: *id,
                tag,
                arguments: r.reference(op[4])?,
                attempt: attempts.first().map(|v| r.reference(*v)).transpose()?,
                outcome: r.optional_ref(out[3])?,
                stage,
            })
        }
    };
    Ok(Some(Live {
        event: r.reference(c[6])?,
        context: r.optional_ref(c[7])?,
        checkpoint: r.reference(c[3])?,
        charged: r.uint(budget[0])?,
        reserved: r.uint(budget[1])?,
        used: r.uint(budget[2])?,
        limit: r.uint(budget[3])?,
        pending,
    }))
}
