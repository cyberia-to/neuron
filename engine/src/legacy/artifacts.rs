//! Validate referenced artifacts in every historical snapshot, not only its head.
use super::records;
use crate::Error;
use neuron_model::{Head, Particle, Reader, Source};

pub(super) fn snapshot<S: Source>(r: &mut Reader<'_, S>, head: Head) -> Result<(), Error> {
    let (_, snapshot) = records::snapshot_at(r, head)?;
    r.artifact(snapshot.application_state)?;
    let definition = r.record(snapshot.definition, "cell/definition/1", 11)?;
    let code = r.reference(definition[2])?;
    r.artifact(code)?;
    for value in r.list(definition[5], 4096)? {
        let artifact = r.reference(value)?;
        r.artifact(artifact)?;
    }
    records::policy_values(r, snapshot.authority_policy)?;
    if let Some(live) = records::read_live(r, &snapshot)? {
        r.artifact(live.checkpoint)?;
        event(r, live.event)?;
        if let Some(p) = live.pending {
            r.artifact(p.arguments)?;
            if let Some(outcome) = p.outcome {
                operation(r, outcome)?;
            }
        }
    }
    for (_, id) in r.map(snapshot.inbox, 4096)? {
        let entry = r.record(id, "cell/inbox-entry/1", 4)?;
        let id = r.reference(entry[0])?;
        event(r, id)?;
    }
    Ok(())
}
pub(super) fn event<S: Source>(r: &mut Reader<'_, S>, id: Particle) -> Result<(), Error> {
    let e = r.record(id, "cell/event/1", 10)?;
    let value = r.reference(e[4])?;
    if r.text(e[3])? == "main" {
        r.artifact(value)?;
    } else {
        r.content(value)?;
    }
    if let Some(context) = r.optional_ref(e[5])? {
        r.content(context)?;
    }
    Ok(())
}
pub(super) fn operation<S: Source>(r: &mut Reader<'_, S>, id: Particle) -> Result<(), Error> {
    for (schema, fields, required, optional) in [
        ("cell/operation/1", 11, vec![4], vec![10]),
        ("cell/outcome/1", 6, vec![], vec![3, 4]),
        ("cell/continuation/1", 9, vec![3], vec![]),
        ("cell/local-denial/1", 5, vec![4], vec![]),
    ] {
        match r.record(id, schema, fields) {
            Ok(f) => {
                for i in required {
                    let p = r.reference(f[i])?;
                    r.artifact(p)?;
                }
                for i in optional {
                    if let Some(p) = r.optional_ref(f[i])? {
                        r.artifact(p)?;
                    }
                }
                return Ok(());
            }
            Err(neuron_model::Error::UnsupportedSchema) => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(()) // non-artifact resource/authorization records are checked by their readers
}
