//! External content pack acquisition, typed resolution and validated publication.

pub(crate) mod canonical;
mod packaging;
mod raw;
mod resolver;
mod schema;
mod snapshot;

pub use packaging::{package_official, PackageReceipt};
pub use raw::{CatalogKind, DefinitionKey, DefinitionKind, DefinitionSource};
pub use resolver::{WriterKind as DefinitionWriteKind, WriterReceipt as DefinitionWriter};
pub(crate) use schema::canonical_key_parts;
pub use schema::{
    CatalogFile, ContentContract, ContentIdentity, ContentSelection, DefinitionOperation,
    InputLimits, OperationKind, PackManifest, PackNamespace, WriterSelection,
};
pub use snapshot::{SourceFileReceipt, SourceReceipt};

use super::{live_ron::LiveRonError, ContentGeneration, GameDataBase, ValidatedGameData};
use crate::authoring::control::{self, ExecutionPhase, Resource};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::Arc};

/// Immutable explanation data from the same resolution that produced a token.
/// These are existing loader records, retained without keeping raw input text
/// or reinterpreting paths after publication.
#[derive(Debug)]
pub struct ContentSourceMetadata {
    definitions: Vec<(DefinitionKey, DefinitionSource)>,
    writers: Vec<DefinitionWriter>,
}

impl ContentSourceMetadata {
    pub fn definitions(&self) -> &[(DefinitionKey, DefinitionSource)] {
        &self.definitions
    }

    pub fn source_for(&self, key: &DefinitionKey) -> Option<&DefinitionSource> {
        self.definitions
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, source)| source)
    }

    /// Includes non-winning and Remove writers. The selected writer need not
    /// have a final definition when its operation is Remove.
    pub fn writers(&self) -> &[DefinitionWriter] {
        &self.writers
    }
}

/// Read exactly the selected external packs and publish a generation only after
/// final reference, compiled domain and generated combat setup validation.
pub fn load(
    trusted_root: &Path,
    selection: &ContentSelection,
    limits: &InputLimits,
) -> Result<ValidatedGameData, LiveRonError> {
    load_with_runtime_safety(
        trusted_root,
        selection,
        limits,
        crate::runtime_safety::RuntimeSafetySettings::embedded_shared(),
    )
}

/// Load under a trusted host profile. Content cannot select or modify this profile.
pub fn load_with_runtime_safety(
    trusted_root: &Path,
    selection: &ContentSelection,
    limits: &InputLimits,
    settings: Arc<crate::runtime_safety::RuntimeSafetySettings>,
) -> Result<ValidatedGameData, LiveRonError> {
    let mut limits = limits.clone();
    let parse = settings.expression_parse();
    limits.value_expr.max_input_bytes =
        limits.value_expr.max_input_bytes.min(parse.max_input_bytes);
    limits.value_expr.max_depth = limits.value_expr.max_depth.min(parse.max_depth);
    limits.value_expr.max_nodes = limits.value_expr.max_nodes.min(parse.max_nodes);
    limits.value_expr.max_authored_members = limits
        .value_expr
        .max_authored_members
        .min(parse.max_authored_members);
    let limits = &limits;
    let _phase = control::phase(ExecutionPhase::Acquisition);
    control::charge(1, "content_acquisition").map_err(control_error)?;
    let snapshot = snapshot::acquire(trusted_root, selection, limits)?;
    control::poll("content_acquired").map_err(control_error)?;
    publish_snapshot_with_runtime_safety(snapshot, selection, limits, settings)
}

fn control_error(error: crate::game::behavior::GameError) -> LiveRonError {
    LiveRonError::validation(
        "authoring_attempt",
        "execution_control",
        None::<String>,
        None::<String>,
        format!("{error:?}"),
    )
}

fn publish_snapshot(
    snapshot: snapshot::SourceSnapshot,
    selection: &ContentSelection,
    limits: &InputLimits,
) -> Result<ValidatedGameData, LiveRonError> {
    publish_snapshot_with_runtime_safety(
        snapshot,
        selection,
        limits,
        crate::runtime_safety::RuntimeSafetySettings::embedded_shared(),
    )
}

fn publish_snapshot_with_runtime_safety(
    snapshot: snapshot::SourceSnapshot,
    selection: &ContentSelection,
    limits: &InputLimits,
    settings: Arc<crate::runtime_safety::RuntimeSafetySettings>,
) -> Result<ValidatedGameData, LiveRonError> {
    let _phase = control::phase(ExecutionPhase::Parse);
    let mut packs = Vec::new();
    let mut parsed_count = 0_usize;
    for pack in snapshot.packs {
        control::charge(1, "parse_pack").map_err(control_error)?;
        let mut definitions = Vec::new();
        for (kind, path, input) in pack.catalogs {
            control::charge(input.len() as u64, "parse_catalog_bytes").map_err(control_error)?;
            let parsed = kind
                .parse(&pack.manifest.pack_id, &path, &input, limits)
                .map_err(|error| error.with_pack(&pack.manifest.pack_id).with_catalog(kind))?;
            control::admit(
                Resource::Definitions,
                parsed_count,
                parsed.len(),
                "selected_raw_definitions",
            )
            .map_err(control_error)?;
            parsed_count = parsed_count.saturating_add(parsed.len());
            definitions.extend(parsed);
        }
        packs.push(resolver::ParsedPack {
            manifest: pack.manifest,
            manifest_source: pack.manifest_source,
            definitions,
        });
    }
    let _phase = control::phase(ExecutionPhase::Resolution);
    control::poll("resolve_content").map_err(control_error)?;
    let resolved = resolver::resolve(packs, &selection.winners, limits)
        .map_err(|error| snapshot.receipt.bind_diagnostic(error))?;
    control::admit(
        Resource::Definitions,
        0,
        resolved.definitions.len(),
        "final_definitions",
    )
    .map_err(control_error)?;
    let identity = ContentIdentity {
        contract: ContentContract::current(),
        semantic_sha256: format!("{:x}", Sha256::digest(resolved.semantic_bytes()?)),
    };
    let sources = raw::SourceTable::from_definitions(&resolved.definitions);
    control::metadata_size(&(&sources.0, &resolved.writers)).map_err(control_error)?;
    let _phase = control::phase(ExecutionPhase::FinalValidation);
    let validated = raw::assemble(resolved.definitions)?
        .into_validated()
        .map_err(|error| sources.relocate(error))?;
    let database = Arc::new(
        GameDataBase::from_validated_live_catalogs_with_runtime_safety(validated, settings)
            .map_err(|error| sources.relocate(error))?,
    );
    database
        .value_expr_catalog()
        .ensure_runtime_publishable()
        .map_err(|error| sources.relocate(error))?;
    database
        .try_validate_generated_combat_preview_contracts()
        .map_err(|error| sources.relocate(error))?;
    control::poll("content_publication").map_err(control_error)?;
    Ok(ValidatedGameData(Arc::new(ContentGeneration {
        database,
        identity: Arc::new(identity),
        source_receipt: snapshot.receipt,
        source_metadata: Some(ContentSourceMetadata {
            definitions: sources.0,
            writers: resolved.writers,
        }),
    })))
}

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
pub(super) fn fixture_identity(database: &GameDataBase) -> ContentIdentity {
    let mut parts = std::collections::BTreeMap::new();
    macro_rules! part {
        ($name:literal, $value:expr) => {
            parts.insert(
                $name,
                canonical::bytes($value)
                    .expect("fixture identity requires finite serializable content"),
            );
        };
    }
    part!("resources", database.resource_data.as_ref());
    part!("abnormalities", &database.abnormality_data);
    part!("corroded_employees", &database.corroded_employee_data);
    part!("corroded_waves", &database.corroded_wave_data);
    part!("employees", &database.employee_definition_data);
    part!("consumables", &database.consumable_data);
    part!("equipment", &database.equipment_data);
    part!("shops", &database.shop_data);
    part!("rewards", &database.reward_data);
    part!("events", &database.event_data);
    part!("visual_novels", &database.visual_novel_data);
    part!("visual_novel_triggers", &database.visual_novel_trigger_data);
    part!("pve", &database.pve_data);
    part!("boss_omen", &database.boss_omen_data);
    part!(
        "boss_and_summon",
        &database
            .boss_runtime_data
            .authored_catalogs_for_fixture_identity()
    );
    part!("run_policy", &database.run_policy);
    part!("skills", &database.skill_data);
    part!("buffs", &database.buff_data);
    part!("skill_fragments", &database.skill_fragment_data);
    if !database.relic_data.relics.is_empty() {
        part!("relics", &database.relic_data);
    }
    part!("map_nodes", &database.map_catalogs.map_node_definitions);
    part!(
        "facility_templates",
        &database.map_catalogs.facility_templates
    );
    part!(
        "node_assignment_profiles",
        &database.map_catalogs.node_assignment_profiles
    );
    part!(
        "battlefield_archetypes",
        &database.map_catalogs.battlefield_archetypes
    );
    part!(
        "battlefield_templates",
        &database.map_catalogs.battlefield_templates
    );
    ContentIdentity {
        contract: ContentContract::current(),
        semantic_sha256: format!(
            "{:x}",
            Sha256::digest(
                canonical::bytes(&("explicit_fixture_v1", ContentContract::current(), parts))
                    .expect("fixture identity canonicalization")
            )
        ),
    }
}

#[cfg(all(test, unix))]
pub(crate) mod tests;

#[cfg(all(test, unix))]
mod diagnostic_tests;
