use super::super::live_ron::{LiveRonError, LiveRonPhase, LIVE_RON_SCHEMA_VERSION};
use super::{CatalogKind, DefinitionKey};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::PathBuf};

/// Versions of the actual parsers/executors/canonical representation required by
/// a generation. They are saved together with the semantic content fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentContract {
    pub pack_format: u32,
    pub catalog_schema: u32,
    pub authoring: u32,
    pub execution: u32,
    pub fingerprint: u32,
}

impl ContentContract {
    pub fn current() -> Self {
        Self {
            pack_format: 1,
            catalog_schema: LIVE_RON_SCHEMA_VERSION,
            authoring: 1,
            execution: 3,
            fingerprint: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentIdentity {
    pub contract: ContentContract,
    pub semantic_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PackNamespace {
    Global,
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogFile {
    pub kind: CatalogKind,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationKind {
    Replace,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefinitionOperation {
    pub target: DefinitionKey,
    pub operation: OperationKind,
    /// Explicit predecessor writers of this target, each identified by pack ID.
    pub after: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackManifest {
    pub pack_id: String,
    pub pack_version: String,
    pub namespace: PackNamespace,
    pub contract: ContentContract,
    pub required_capabilities: Vec<String>,
    pub dependencies: Vec<String>,
    pub files: Vec<CatalogFile>,
    pub operations: Vec<DefinitionOperation>,
}

impl PackManifest {
    pub const SUPPORTED_CAPABILITIES: &'static [&'static str] = &[
        "gameplay_catalogs_v2",
        "optional_encounter_fragment_v1",
        "revival_variant_first_rewards_v1",
        "skill_composition_yaml_v1",
        "expedition_lifecycle_v1",
        "literal_effect_authoring_v1",
        "boss_runtime_v1",
        "summoned_units_v1",
        "typed_resources_v1",
        "structured_skill_flow_v1",
        "source_owned_sustained_effects_v1",
    ];

    pub(super) fn validate(&self, source: &str) -> Result<(), LiveRonError> {
        let error = |field, cause| {
            let mut error = LiveRonError::validation(
                source,
                "pack_manifest",
                Some(&self.pack_id),
                Some(field),
                cause,
            )
            .with_phase(LiveRonPhase::Structure);
            if valid_pack_name(&self.pack_id) {
                error.pack_id = Some(self.pack_id.clone());
            }
            error
        };
        if !valid_pack_name(&self.pack_id) {
            return Err(error("pack_id", "invalid pack ID"));
        }
        if self.pack_version.trim().is_empty()
            || self.pack_version.trim() != self.pack_version
            || self.pack_version.chars().any(char::is_control)
        {
            return Err(error(
                "pack_version",
                "pack version must be a nonempty stable string",
            ));
        }
        if self.contract != ContentContract::current() {
            let mut diagnostic = error(
                "contract",
                "only the current complete content contract is supported",
            );
            diagnostic.code = "content_contract_unsupported";
            diagnostic.expected = Some(serde_json::json!(ContentContract::current()));
            diagnostic.actual = Some(serde_json::json!(self.contract));
            return Err(diagnostic);
        }
        if matches!(&self.namespace, PackNamespace::Named(name) if !valid_namespace(name)) {
            return Err(error(
                "namespace",
                "namespace must be a lowercase ASCII identifier",
            ));
        }
        let mut capabilities = BTreeSet::new();
        for capability in &self.required_capabilities {
            if !Self::SUPPORTED_CAPABILITIES.contains(&capability.as_str()) {
                let mut diagnostic = error(
                    "required_capabilities",
                    "unsupported authoring/execution capability",
                );
                diagnostic.code = "content_capability_unsupported";
                diagnostic.expected = Some(serde_json::json!(Self::SUPPORTED_CAPABILITIES));
                diagnostic.actual = Some(serde_json::json!(capability));
                return Err(diagnostic);
            }
            if !capabilities.insert(capability) {
                return Err(error(
                    "required_capabilities",
                    "duplicate required capability",
                ));
            }
        }
        if self.files.iter().any(|file| {
            file.kind == CatalogKind::Skills
                && file.path.to_string_lossy().ends_with(".skill.yaml")
        }) && !self.required_capabilities.iter().any(|cap| cap == "skill_composition_yaml_v1") {
            return Err(error(
                "required_capabilities",
                "YAML Skills catalogs require skill_composition_yaml_v1",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WriterSelection {
    pub target: DefinitionKey,
    pub pack_id: String,
}

/// Only these relative pack directories are selected. Each contains pack.ron.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentSelection {
    pub packs: Vec<PathBuf>,
    pub winners: Vec<WriterSelection>,
}

/// Trusted host budgets. Pack input cannot deserialize or override these limits.
#[derive(Debug, Clone)]
pub struct InputLimits {
    pub max_packs: usize,
    pub max_files: usize,
    pub max_file_bytes: usize,
    pub max_total_bytes: usize,
    pub max_dependency_edges: usize,
    pub max_nesting: usize,
    pub value_expr: crate::game::value_expr::ExprParseLimits,
}

impl Default for InputLimits {
    fn default() -> Self {
        Self {
            max_packs: 128,
            max_files: 4096,
            max_file_bytes: 8 * 1024 * 1024,
            max_total_bytes: 64 * 1024 * 1024,
            max_dependency_edges: 4096,
            max_nesting: 128,
            value_expr: crate::game::value_expr::ExprParseLimits::default(),
        }
    }
}

pub(super) fn valid_pack_name(value: &str) -> bool {
    value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

pub(super) fn valid_namespace(value: &str) -> bool {
    valid_pack_name(value) && !value.contains('-')
}

pub(crate) fn canonical_key_parts(value: &str) -> Option<(Option<&str>, &str)> {
    let (namespace, local) = match value.split_once("::") {
        Some((namespace, local)) if valid_namespace(namespace) => (Some(namespace), local),
        Some(_) => return None,
        None => (None, value),
    };
    if local.is_empty()
        || local.trim() != local
        || local.contains(':')
        || local.chars().any(char::is_control)
    {
        None
    } else {
        Some((namespace, local))
    }
}
