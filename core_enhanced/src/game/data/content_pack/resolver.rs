use std::collections::{BTreeMap, BTreeSet};

use crate::authoring::control;
use serde::Serialize;

use super::super::live_ron::{LiveRonError, LiveRonPhase, LiveRonSource};
use super::raw::{KeyScope, LocatedDefinition};
use super::schema::{canonical_key_parts, ContentContract, PackNamespace};
use super::{
    DefinitionKey, DefinitionSource, InputLimits, OperationKind, PackManifest, WriterSelection,
};

pub(super) struct ParsedPack {
    pub manifest: PackManifest,
    pub manifest_source: String,
    pub definitions: Vec<LocatedDefinition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum WriterKind {
    Add,
    Replace,
    Remove,
}

struct Writer {
    kind: WriterKind,
    after: BTreeSet<String>,
    payload: Option<LocatedDefinition>,
    source: DefinitionSource,
}

#[derive(Debug, Serialize)]
struct PackIdentity {
    pack_id: String,
    pack_version: String,
    namespace: PackNamespace,
    contract: ContentContract,
    required_capabilities: BTreeSet<String>,
    dependencies: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WriterReceipt {
    pub target: DefinitionKey,
    pub pack_id: String,
    pub operation: WriterKind,
    pub after: BTreeSet<String>,
    pub selected: bool,
}

pub(super) struct ResolvedContent {
    pub definitions: Vec<LocatedDefinition>,
    packs: Vec<PackIdentity>,
    pub(super) writers: Vec<WriterReceipt>,
}

impl ResolvedContent {
    pub(super) fn semantic_bytes(&self) -> Result<Vec<u8>, LiveRonError> {
        let definitions: Vec<_> = self.definitions.iter().map(|d| &d.definition).collect();
        super::canonical::bytes(&(
            ContentContract::current(),
            &self.packs,
            &self.writers,
            definitions,
        ))
        .map_err(|cause| error("resolved_content", None, "fingerprint", cause.to_string()))
    }
}

fn error(
    source: &str,
    target: Option<&DefinitionKey>,
    field: &str,
    cause: impl Into<String>,
) -> LiveRonError {
    let error = LiveRonError::validation(
        source,
        "content_resolver",
        target.map(|t| t.id.clone()),
        Some(field),
        cause,
    )
    .with_phase(LiveRonPhase::Resolution);
    match target {
        Some(target) => error.with_definition(target.kind),
        None => error,
    }
}

fn writer_error(writer: &Writer, target: &DefinitionKey, cause: impl Into<String>) -> LiveRonError {
    error(
        &writer.source.source_file,
        Some(target),
        &writer.source.field_path,
        cause,
    )
    .with_pack(&writer.source.pack_id)
}

fn writers_error<'a>(
    target: &DefinitionKey,
    field: &str,
    cause: &str,
    writers: impl IntoIterator<Item = &'a Writer>,
) -> LiveRonError {
    let mut diagnostic = error("selection", Some(target), field, cause);
    // One writer per pack and target, visited in deterministic pack-ID order.
    // Keep the aggregate primary unowned; each contributor has an actual source.
    diagnostic.related_sources = writers
        .into_iter()
        .map(|writer| LiveRonSource {
            source_file: writer.source.source_file.clone(),
            object_id: Some(target.id.clone()),
            field_path: Some(writer.source.field_path.clone()),
            pack_id: Some(writer.source.pack_id.clone()),
            catalog_kind: Some(target.kind.catalog()),
            definition_kind: Some(target.kind),
        })
        .collect();
    diagnostic
}

/// Resolve explicit writer graphs, then return final typed payloads in the base
/// Add's execution slots. No domain references or database indexes are evaluated
/// until after this function succeeds.
pub(super) fn resolve(
    packs: Vec<ParsedPack>,
    selections: &[WriterSelection],
    limits: &InputLimits,
) -> Result<ResolvedContent, LiveRonError> {
    if packs.is_empty() || packs.len() > limits.max_packs {
        return Err(error(
            "selection",
            None,
            "packs",
            "empty selection or pack count limit exceeded",
        ));
    }
    let mut by_id = BTreeMap::new();
    let mut edge_count = 0_usize;
    for pack in packs {
        control::charge(1, "resolver_pack").map_err(super::control_error)?;
        pack.manifest.validate(&pack.manifest_source)?;
        for edges in std::iter::once(pack.manifest.dependencies.len())
            .chain(pack.manifest.operations.iter().map(|op| op.after.len()))
        {
            control::charge(edges as u64, "resolver_dependency_edges")
                .map_err(super::control_error)?;
            edge_count = edge_count.checked_add(edges).ok_or_else(|| {
                error(
                    &pack.manifest_source,
                    None,
                    "dependencies",
                    "dependency edge count overflow",
                )
            })?;
            if edge_count > limits.max_dependency_edges {
                return Err(error(
                    &pack.manifest_source,
                    None,
                    "dependencies",
                    "dependency/writer edge limit exceeded",
                ));
            }
        }
        let pack_id = pack.manifest.pack_id.clone();
        if by_id.insert(pack_id.clone(), pack).is_some() {
            return Err(error(
                "selection",
                None,
                "packs",
                format!("duplicate pack ID '{pack_id}'"),
            ));
        }
    }

    // Availability graph; its transitive closure checks authorization for an
    // explicit writer edge. It never selects or orders competing writers.
    let mut dependencies = BTreeMap::new();
    for (id, pack) in &by_id {
        control::charge(
            pack.manifest.dependencies.len() as u64 + 1,
            "resolver_dependencies",
        )
        .map_err(super::control_error)?;
        let mut direct = BTreeSet::new();
        for dependency in &pack.manifest.dependencies {
            if dependency == id
                || !by_id.contains_key(dependency)
                || !direct.insert(dependency.clone())
            {
                return Err(error(
                    &pack.manifest_source,
                    None,
                    "dependencies",
                    format!("self, missing or duplicate dependency '{dependency}'"),
                ));
            }
        }
        dependencies.insert(id.clone(), direct);
    }
    let mut ancestors = BTreeMap::<String, BTreeSet<String>>::new();
    while ancestors.len() != by_id.len() {
        control::charge(by_id.len() as u64, "resolver_ancestor_search")
            .map_err(super::control_error)?;
        let next = dependencies.iter().find(|(id, deps)| {
            !ancestors.contains_key(*id) && deps.iter().all(|d| ancestors.contains_key(d))
        });
        let Some((id, direct)) = next else {
            return Err(error("selection", None, "dependencies", "dependency cycle"));
        };
        let mut closure = direct.clone();
        for dependency in direct {
            control::charge(
                ancestors[dependency].len() as u64,
                "resolver_ancestor_closure",
            )
            .map_err(super::control_error)?;
            closure.extend(ancestors[dependency].iter().cloned());
        }
        ancestors.insert(id.clone(), closure);
    }

    let mut targets = BTreeMap::<DefinitionKey, BTreeMap<String, Writer>>::new();
    let mut identities = Vec::new();
    for (id, pack) in by_id {
        let mut payloads = BTreeMap::new();
        for definition in pack.definitions {
            control::charge(1, "resolver_payload").map_err(super::control_error)?;
            let key = definition.definition.key();
            if definition.source.pack_id != id {
                return Err(error(
                    &definition.source.source_file,
                    Some(&key),
                    &definition.source.field_path,
                    "payload source does not belong to manifest pack",
                ));
            }
            if key.kind.scope() != KeyScope::FixedKey && canonical_key_parts(&key.id).is_none() {
                return Err(error(
                    &definition.source.source_file,
                    Some(&key),
                    &definition.source.field_path,
                    "definition key must be exact global or namespace::local",
                ));
            }
            if payloads.insert(key.clone(), definition).is_some() {
                return Err(error(
                    &pack.manifest_source,
                    Some(&key),
                    "files",
                    "duplicate payload for the same pack/definition key",
                ));
            }
        }
        let mut operated = BTreeSet::new();
        for (index, op) in pack.manifest.operations.iter().enumerate() {
            control::charge(op.after.len() as u64 + 1, "resolver_operation")
                .map_err(super::control_error)?;
            let field = format!("operations[{index}]");
            if !operated.insert(op.target.clone()) {
                return Err(error(
                    &pack.manifest_source,
                    Some(&op.target),
                    &field,
                    "duplicate operation for the same pack/definition key",
                ));
            }
            let payload = payloads.remove(&op.target);
            let kind =
                match (op.operation, payload.is_some()) {
                    (OperationKind::Replace, true) => WriterKind::Replace,
                    (OperationKind::Remove, false) => WriterKind::Remove,
                    _ => return Err(error(
                        &pack.manifest_source,
                        Some(&op.target),
                        &field,
                        "Replace requires exactly one same-key payload; Remove forbids a payload",
                    )),
                };
            let after: BTreeSet<_> = op.after.iter().cloned().collect();
            if after.is_empty() || after.len() != op.after.len() {
                return Err(error(
                    &pack.manifest_source,
                    Some(&op.target),
                    &field,
                    "after must name nonempty unique predecessor writers",
                ));
            }
            for predecessor in &after {
                if !ancestors[&id].contains(predecessor) {
                    return Err(error(
                        &pack.manifest_source,
                        Some(&op.target),
                        &field,
                        format!("writer '{predecessor}' is not a declared dependency ancestor"),
                    ));
                }
            }
            targets.entry(op.target.clone()).or_default().insert(
                id.clone(),
                Writer {
                    kind,
                    after,
                    payload,
                    source: DefinitionSource {
                        pack_id: id.clone(),
                        source_file: pack.manifest_source.clone(),
                        field_path: field,
                    },
                },
            );
        }
        for (key, payload) in payloads {
            control::charge(1, "resolver_add").map_err(super::control_error)?;
            if key.kind.scope() == KeyScope::IndependentString {
                let (namespace, _) =
                    canonical_key_parts(&key.id).expect("canonical key checked above");
                let expected = match &pack.manifest.namespace {
                    PackNamespace::Global => None,
                    PackNamespace::Named(value) => Some(value.as_str()),
                };
                if namespace != expected {
                    return Err(error(
                        &payload.source.source_file,
                        Some(&key),
                        &payload.source.field_path,
                        "Add key does not explicitly belong to the declared namespace",
                    ));
                }
            }
            let source = payload.source.clone();
            targets.entry(key).or_default().insert(
                id.clone(),
                Writer {
                    kind: WriterKind::Add,
                    after: BTreeSet::new(),
                    payload: Some(payload),
                    source,
                },
            );
        }
        identities.push(PackIdentity {
            pack_id: id.clone(),
            pack_version: pack.manifest.pack_version,
            namespace: pack.manifest.namespace,
            contract: pack.manifest.contract,
            required_capabilities: pack.manifest.required_capabilities.into_iter().collect(),
            dependencies: dependencies
                .remove(&id)
                .expect("dependency graph contains every selected pack"),
        });
    }

    let mut explicit = BTreeMap::new();
    for selection in selections {
        control::charge(1, "resolver_selection").map_err(super::control_error)?;
        if !targets.contains_key(&selection.target)
            || explicit
                .insert(selection.target.clone(), selection.pack_id.clone())
                .is_some()
        {
            return Err(error(
                "selection",
                Some(&selection.target),
                "winners",
                "unknown target or duplicate winner selection",
            ));
        }
    }
    let mut final_slots = Vec::new();
    let mut receipts = Vec::new();
    for (target, mut writers) in targets {
        control::charge(writers.len() as u64, "resolver_target_writers")
            .map_err(super::control_error)?;
        let bases = writers
            .iter()
            .filter(|(_, writer)| writer.kind == WriterKind::Add)
            .collect::<Vec<_>>();
        if bases.len() != 1 {
            return Err(writers_error(
                &target,
                "operations",
                "target requires exactly one base Add; duplicate Add and missing targets are errors",
                writers.values().filter(|writer| bases.is_empty() || writer.kind == WriterKind::Add),
            ));
        }
        let (base_id, base) = bases[0];
        let base_payload = base.payload.as_ref().expect("Add always has payload");
        let slot = (
            target.kind.catalog(),
            base_id.clone(),
            base_payload.source.source_file.clone(),
            base_payload.source_order,
        );
        let mut referenced = BTreeSet::new();
        for writer in writers.values() {
            control::charge(writer.after.len() as u64 + 1, "resolver_writer_edges")
                .map_err(super::control_error)?;
            for predecessor in &writer.after {
                let previous = writers.get(predecessor).ok_or_else(|| {
                    writer_error(
                        writer,
                        &target,
                        format!("pack '{predecessor}' has no writer for this target"),
                    )
                })?;
                if previous.kind == WriterKind::Remove {
                    return Err(writer_error(
                        writer,
                        &target,
                        "operation follows a removed target",
                    ));
                }
                referenced.insert(predecessor.clone());
            }
        }
        let mut reached = BTreeSet::from([base_id.clone()]);
        while reached.len() != writers.len() {
            control::charge(writers.len() as u64, "resolver_writer_search")
                .map_err(super::control_error)?;
            let next = writers.iter().find(|(id, writer)| {
                !reached.contains(*id)
                    && !writer.after.is_empty()
                    && writer.after.iter().all(|id| reached.contains(id))
            });
            let Some((id, _)) = next else {
                return Err(writers_error(
                    &target,
                    "operations",
                    "writer cycle or writer disconnected from base Add",
                    writers.values(),
                ));
            };
            reached.insert(id.clone());
        }
        let heads: Vec<_> = writers
            .keys()
            .filter(|id| !referenced.contains(*id))
            .cloned()
            .collect();
        let winner = match explicit.remove(&target) {
            Some(id) if heads.contains(&id) => id,
            Some(_) => {
                return Err(writers_error(
                    &target,
                    "winners",
                    "selected writer is absent or non-maximal",
                    writers.values(),
                ))
            }
            None if heads.len() == 1 => heads[0].clone(),
            None => {
                return Err(writers_error(
                    &target,
                    "winners",
                    "ambiguous writers require an explicit winner or explicit chain",
                    heads.iter().filter_map(|id| writers.get(id)),
                ))
            }
        };
        for (id, writer) in &writers {
            receipts.push(WriterReceipt {
                target: target.clone(),
                pack_id: id.clone(),
                operation: writer.kind,
                after: writer.after.clone(),
                selected: id == &winner,
            });
        }
        if let Some(payload) = writers
            .remove(&winner)
            .expect("selected existing head")
            .payload
        {
            final_slots.push((slot, payload));
        }
    }
    final_slots.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(ResolvedContent {
        definitions: final_slots
            .into_iter()
            .map(|(_, payload)| payload)
            .collect(),
        packs: identities,
        writers: receipts,
    })
}

#[cfg(test)]
mod tests;
