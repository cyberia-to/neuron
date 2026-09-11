//! Canonical record construction for the local profile.
use crate::Error;
use cell_model::{Builder, Head, Particle, Reader, Snapshot, Source, Value::*};
use std::collections::BTreeMap;

pub const RUNTIME: &[u8] = b"rune/bounded-machine/1";
pub const SLICE: u64 = 1000;

pub fn named(b: &mut Builder, name: &str) -> Result<Particle, Error> {
    Ok(b.blob(name.as_bytes().to_vec())?)
}
pub fn authority(b: &mut Builder) -> Result<Particle, Error> {
    named(b, "cell/local-private-database-owner/1")
}
pub fn policy(b: &mut Builder, steps: u64, allowed: &[u64]) -> Result<Particle, Error> {
    let owner = authority(b)?;
    let tags = allowed
        .iter()
        .map(|v| b.uint(*v))
        .collect::<Result<Vec<_>, _>>()?;
    let tags = b.list(&tags)?;
    Ok(b.values("cell/local-policy/1", &[Ref(owner), Uint(steps), Raw(tags)])?)
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
pub fn definition(
    b: &mut Builder,
    source: Vec<u8>,
    policy: Particle,
    steps: u64,
) -> Result<Particle, Error> {
    let runtime = b.blob(RUNTIME.to_vec())?;
    let code = b.artifact(source, "application/x-rune")?;
    let noun = b.schema("rune/noun-artifact/1")?;
    let checkpoint = b.schema("rune/checkpoint/1")?;
    let behavior = b.variant("entry/event", &[])?;
    let entry = b.values(
        "cell/entry/1",
        &[Text("main"), Ref(noun), Ref(noun), Raw(behavior)],
    )?;
    let entries = b.list(&[entry])?;
    let metric = named(b, "compute_steps")?;
    let key = b.reference(metric)?;
    let value = b.uint(steps)?;
    let pair = b.pair(key, value)?;
    let limits = b.list(&[pair])?;
    let overflow = b.variant("overflow/reject", &[])?;
    let resources = b.values(
        "cell/resource-contract/1",
        &[Raw(limits), Uint(SLICE), Raw(overflow)],
    )?;
    Ok(b.values(
        "cell/definition/1",
        &[
            Uint(1),
            Ref(runtime),
            Ref(code),
            Ref(noun),
            Raw(entries),
            Refs(&[code]),
            Ref(policy),
            Ref(resources),
            Optional(Some(checkpoint)),
            Optional(None),
            Refs(&[]),
        ],
    )?)
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
pub fn event(
    b: &mut Builder,
    cell: Particle,
    nonce: [u8; 32],
    entry: &str,
    payload: Particle,
    context: Option<Particle>,
    policy: Particle,
) -> Result<(Particle, Particle), Error> {
    let origin = authority(b)?;
    let nonce = b.nonce(nonce)?;
    let event = b.values(
        "cell/event/1",
        &[
            Ref(origin),
            Raw(nonce),
            Ref(cell),
            Text(entry),
            Ref(payload),
            Optional(context),
            Refs(&[]),
            Optional(None),
            Optional(None),
            Ref(policy),
        ],
    )?;
    let request = b.values("cell/request-id/1", &[Ref(origin), Raw(nonce)])?;
    Ok((event, request))
}
pub fn system_event(
    b: &mut Builder,
    cell: Particle,
    head: Head,
    kind: &str,
    payload: Particle,
    policy: Particle,
) -> Result<(Particle, Particle), Error> {
    let nonce = b.values(
        "cell/system-nonce/1",
        &[Ref(cell), Ref(head.commit), Text(kind)],
    )?;
    event(b, cell, nonce, kind, payload, None, policy)
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
pub fn commit(
    b: &mut Builder,
    cell: Particle,
    head: Head,
    snapshots: (Particle, Particle),
    event: Particle,
    records: &[Particle],
    snapshot: &Snapshot,
) -> Result<Head, Error> {
    let (before, after) = snapshots;
    let index = head.index.checked_add(1).ok_or(Error::Budget)?;
    let id = b.values(
        "cell/commit/1",
        &[
            Ref(cell),
            Uint(index),
            Ref(head.commit),
            Ref(before),
            Ref(after),
            Refs(&[event]),
            Refs(records),
            Ref(snapshot.authority_policy),
            Uint(snapshot.epoch),
            Refs(&[]),
        ],
    )?;
    Ok(Head { index, commit: id })
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
pub fn inbox(
    b: &mut Builder,
    event: Particle,
    status: &str,
    index: u64,
) -> Result<Particle, Error> {
    let status = b.variant(&format!("inbox-status/{status}"), &[])?;
    let entry = b.values(
        "cell/inbox-entry/1",
        &[Ref(event), Raw(status), Optional(Some(event)), Uint(index)],
    )?;
    Ok(b.map(&BTreeMap::from([(event, entry)]))?)
}
pub fn write_live(
    b: &mut Builder,
    snapshot: &mut Snapshot,
    live: &Live,
    head: Head,
) -> Result<Vec<Particle>, Error> {
    let runtime = b.blob(RUNTIME.to_vec())?;
    let schema = b.schema("rune/checkpoint/1")?;
    let trigger = match &live.pending {
        Some(p) => b.variant("trigger/operation-result", &[Ref(p.id)])?,
        None => b.variant("trigger/scheduler-yield", &[])?,
    };
    let base = b.head(head)?;
    let resources = b.values(
        "cell/local-resources/1",
        &[
            Uint(live.charged),
            Uint(live.reserved),
            Uint(live.used),
            Uint(live.limit),
        ],
    )?;
    let c = b.values(
        "cell/continuation/1",
        &[
            Ref(snapshot.definition),
            Ref(runtime),
            Ref(schema),
            Ref(live.checkpoint),
            Raw(trigger),
            Ref(base),
            Ref(live.event),
            Optional(live.context),
            Ref(resources),
        ],
    )?;
    snapshot.continuations = b.map(&BTreeMap::from([(c, c)]))?;
    snapshot.inbox = inbox(
        b,
        live.event,
        "suspended",
        head.index.checked_add(1).ok_or(Error::Budget)?,
    )?;
    let mut records = vec![c];
    snapshot.outbox = if let Some(p) = &live.pending {
        let stage = b.variant(&format!("operation-stage/{}", p.stage), &[])?;
        let attempts: Vec<_> = p.attempt.into_iter().collect();
        let out = b.values(
            "cell/outbox-entry/1",
            &[
                Ref(p.record),
                Raw(stage),
                Refs(&attempts),
                Optional(p.outcome),
                Optional(None),
            ],
        )?;
        records.push(out);
        b.map(&BTreeMap::from([(p.id, out)]))?
    } else {
        b.empty_map()?
    };
    Ok(records)
}
pub fn operation(
    b: &mut Builder,
    cell: Particle,
    head: Head,
    live: &Live,
    snapshot: &Snapshot,
    tag: u64,
    args: Vec<u8>,
) -> Result<Pending, Error> {
    let id = b.values(
        "cell/operation-id/1",
        &[Ref(cell), Ref(head.commit), Ref(live.event), Uint(0)],
    )?;
    let act = b.values("rune/act/1", &[Uint(tag)])?;
    let arguments = b.artifact(args, "application/x-rune-noun")?;
    let target = b.variant("target/local-executor", &[])?;
    let schema = b.schema("rune/noun-artifact/1")?;
    let retry = b.variant("retry/never", &[])?;
    let finality = named(b, "cell/local-authority/1")?;
    let record = b.values(
        "cell/operation/1",
        &[
            Ref(id),
            Ref(live.event),
            Raw(target),
            Ref(act),
            Ref(arguments),
            Ref(snapshot.authority_policy),
            Ref(schema),
            Optional(None),
            Raw(retry),
            Ref(finality),
            Optional(None),
        ],
    )?;
    Ok(Pending {
        record,
        id,
        tag,
        arguments,
        attempt: None,
        outcome: None,
        stage: "pending",
    })
}

/// Retain the result's single consumer even after removing it from live state.
pub fn consumed(b: &mut Builder, live: &Live) -> Result<Option<Particle>, Error> {
    let Some(p) = live.pending.as_ref().filter(|p| p.stage == "resolved") else {
        return Ok(None);
    };
    let stage = b.variant("operation-stage/resolved", &[])?;
    let attempts: Vec<_> = p.attempt.into_iter().collect();
    Ok(Some(b.values(
        "cell/outbox-entry/1",
        &[
            Ref(p.record),
            Raw(stage),
            Refs(&attempts),
            Optional(p.outcome),
            Optional(Some(live.event)),
        ],
    )?))
}
