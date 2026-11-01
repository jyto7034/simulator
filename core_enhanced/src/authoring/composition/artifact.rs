use super::*;
use crate::game::{
    ability::authoring::EffectSource,
    data::{live_ron::LiveRonError, skill_data::SkillDatabase},
    resource::{ResourceCatalog, ResourceDefinition, ResourceDefinitionError},
    value_expr::{ExprLimits, ValueExprCatalog},
};
use std::collections::BTreeMap;

/// Portable native data plus a cold source map. Loading always revalidates the
/// executable data; source metadata is only used for diagnostics/observation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    // Trust is process-local and cannot be asserted by a serialized artifact.
    #[serde(skip)]
    pub(super) original_verified: bool,
    pub(super) version: u32,
    pub(super) format: String,
    pub(super) document: String,
    pub(super) source_digest: String,
    pub(super) native_ron: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) resources: Vec<ResourceDefinition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) equipment_attacks:
        Vec<crate::game::ability::equipment_attack::EquipmentAttackBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) equipment_defenses:
        Vec<crate::game::ability::equipment_defense::EquipmentDefenseBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) equipment_supports:
        Vec<crate::game::ability::equipment_support::EquipmentSupportBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) equipment_states: Vec<crate::game::ability::equipment_state::EquipmentStateBinding>,
    pub(super) sources: BTreeMap<String, SourceTrace>,
}
impl Artifact {
    /// Used by the production pack loader after bounded envelope parsing. The
    /// loader still applies its own expression limits and final cross-catalog
    /// validation before publishing a generation.
    pub(crate) fn validated_native_ron(&self) -> Result<&str, Diagnostic> {
        self.load_native()?;
        Ok(&self.native_ron)
    }
    /// Declarations carried by this artifact; pack publication installs them
    /// through the same definition resolver as standalone resource catalogs.
    pub fn resources(&self) -> &[ResourceDefinition] {
        &self.resources
    }
    pub fn equipment_attacks(
        &self,
    ) -> &[crate::game::ability::equipment_attack::EquipmentAttackBinding] {
        &self.equipment_attacks
    }
    pub fn equipment_defenses(
        &self,
    ) -> &[crate::game::ability::equipment_defense::EquipmentDefenseBinding] {
        &self.equipment_defenses
    }
    pub fn equipment_supports(
        &self,
    ) -> &[crate::game::ability::equipment_support::EquipmentSupportBinding] {
        &self.equipment_supports
    }
    pub fn equipment_states(
        &self,
    ) -> &[crate::game::ability::equipment_state::EquipmentStateBinding] {
        &self.equipment_states
    }
    pub fn source_digest(&self) -> &str {
        &self.source_digest
    }
    /// Declared diagnostic metadata. Use origin_for for verified attribution.
    pub fn sources(&self) -> &BTreeMap<String, SourceTrace> {
        &self.sources
    }
    pub(super) fn source_key(&self) -> String {
        format!("composition:{}:{}", self.document, self.source_digest)
    }
    pub(super) fn diagnostic(
        &self,
        code: DiagnosticCode,
        path: &str,
        reason: impl Into<String>,
    ) -> Diagnostic {
        Diagnostic {
            code,
            document: self.document.clone(),
            source_digest: self.source_digest.clone(),
            source: self.original_verified.then(|| self.source_entry(path)).flatten()
                .map(|(native, source)| {
                    let mut source = source.clone();
                    // Append payloads preserve native field structure. Carry the
                    // diagnostic suffix to the literal instead of selecting its
                    // entire (potentially long) state program.
                    if native.ends_with(".op") || native.ends_with(".program") {
                        source.definition.push_str(&path[native.len()..]);
                    }
                    source
                })
                .unwrap_or(SourceTrace {
                    definition: path.into(),
                    calls: vec![],
                    inputs: vec![],
                }),
            reason: reason.into(),
        }
    }
    pub(crate) fn source_at(&self, path: &str) -> Option<&SourceTrace> {
        self.original_verified.then(|| self.source_entry(path).map(|(_, source)| source)).flatten()
    }
    pub fn is_source_verified(&self) -> bool {
        self.original_verified
    }
    /// Compare imported metadata and executable data with the separately held
    /// original. Native loading alone cannot establish an original location.
    pub fn verify_json_source(&mut self, original: &str) -> Result<(), Diagnostic> {
        self.original_verified = false;
        self.verify_compilation(super::compile_json(original)?)
    }
    pub fn verify_yaml_source(&mut self, original: &str) -> Result<(), Diagnostic> {
        self.original_verified = false;
        self.verify_compilation(super::compile_yaml(original)?)
    }
    fn verify_compilation(&mut self, expected: Artifact) -> Result<(), Diagnostic> {
        let actual = serde_json::to_value(&*self);
        let expected = serde_json::to_value(&expected);
        if !matches!((actual, expected), (Ok(actual), Ok(expected)) if actual == expected) {
            return Err(self.diagnostic(DiagnosticCode::Reference, "artifact",
                "artifact data or source map differs from the supplied original"));
        }
        self.original_verified = true;
        Ok(())
    }
    fn source_entry(&self, path: &str) -> Option<(&String, &SourceTrace)> {
        self.sources
            .iter()
            .filter(|(key, _)| {
                path == key.as_str()
                    || path
                        .strip_prefix(key.as_str())
                        .is_some_and(|tail| tail.starts_with('.') || tail.starts_with('['))
            })
            .max_by_key(|(key, _)| key.len())
    }
    /// Runtime EffectSource carries the original native field path. Its source
    /// identity must match this exact compilation before the map can be used.
    pub fn origin_for(&self, effect: &EffectSource) -> Option<&SourceTrace> {
        self.origin_for_file(effect, &self.source_key())
    }
    pub(super) fn origin_for_file(
        &self,
        effect: &EffectSource,
        source_file: &str,
    ) -> Option<&SourceTrace> {
        if !self.original_verified || effect.source_file != source_file {
            return None;
        }
        let mut native = self.load_native().ok()?;
        native.bind_sources(source_file);
        let resources = ResourceCatalog { resources: self.resources.clone() };
        let expressions = ValueExprCatalog::compile_skills(&native, &resources, &ExprLimits::default()).ok()?;
        super::super::provenance::registered_skill_source(&native, &expressions, effect)
            .then(|| self.source_at(&effect.field_path)).flatten()
    }
    fn native_error(&self, error: LiveRonError) -> Diagnostic {
        self.diagnostic(
            DiagnosticCode::Native,
            error.field_path.as_deref().unwrap_or("skills"),
            error.to_string(),
        )
    }
    /// Validate the complete artifact and return its skills projection.
    /// Resource declarations remain available through `resources()`.
    pub fn load_native(&self) -> Result<SkillDatabase, Diagnostic> {
        if !(1..=VERSION).contains(&self.version) || self.format != "skill-composition-native" {
            return Err(self.diagnostic(
                DiagnosticCode::Version,
                "version",
                "unsupported composition artifact",
            ));
        }
        if self.document.is_empty()
            || self.document.len() > 256
            || self.source_digest.len() != 64
            || !self.source_digest.bytes().all(|b| b.is_ascii_hexdigit())
            || self.native_ron.len() > MAX_ARTIFACT_BYTES
            || self.sources.len() > 65_536
            || self.sources.iter().any(|(path, source)| {
                path.len() > 4096
                    || source.definition.len() > 4096
                    || source.calls.len() > MAX_CALL_DEPTH + 1
                    || source.inputs.len() > 256
                    || source
                        .calls
                        .iter()
                        .chain(&source.inputs)
                        .any(|p| p.len() > 4096)
            })
        {
            return Err(self.diagnostic(
                DiagnosticCode::Limit,
                "artifact",
                "invalid or oversized artifact metadata",
            ));
        }
        let skills = SkillDatabase::from_ron(&self.source_key(), &self.native_ron)
            .map_err(|error| self.native_error(error))?;
        let resources = ResourceCatalog {
            resources: self.resources.clone(),
        };
        resources.validate().map_err(|error| {
            let path = match &error {
                ResourceDefinitionError::InitialAboveCap(id) => self
                    .resources
                    .iter()
                    .position(|r| &r.id == id)
                    .map(|i| format!("resources[{i}].initial")),
                ResourceDefinitionError::DuplicateId(id) => self
                    .resources
                    .iter()
                    .rposition(|r| &r.id == id)
                    .map(|i| format!("resources[{i}].id")),
                _ => None,
            };
            self.diagnostic(
                DiagnosticCode::Native,
                path.as_deref().unwrap_or("resources"),
                error.to_string(),
            )
        })?;
        skills
            .validate_resource_references(&resources)
            .map_err(|error| {
                self.diagnostic(
                    DiagnosticCode::Native,
                    error.field_path.as_deref().unwrap_or("skills"),
                    error.cause,
                )
            })?;
        ValueExprCatalog::compile_skills(&skills, &resources, &ExprLimits::default())
            .map_err(|error| self.native_error(error))?;
        let mut equipment = std::collections::BTreeSet::new();
        if self.equipment_attacks.len() > 128 {
            return Err(self.diagnostic(
                DiagnosticCode::Limit,
                "equipment_attacks",
                "too many equipment bindings",
            ));
        }
        for (index, binding) in self.equipment_attacks.iter().enumerate() {
            let path = format!("equipment_attacks[{index}]");
            if !equipment.insert(&binding.equipment) {
                return Err(self.diagnostic(
                    DiagnosticCode::Reference,
                    &format!("{path}.equipment"),
                    "duplicate equipment binding",
                ));
            }
            let mut binding = binding.clone();
            binding.source = Some((self.source_key(), path.clone()));
            binding
                .compile(&resources, &ExprLimits::default())
                .map_err(|(p, reason)| {
                    self.diagnostic(DiagnosticCode::Native, &format!("{path}.{p}"), reason)
                })?;
        }
        let mut equipment = std::collections::BTreeSet::new();
        if self.equipment_defenses.len() > 128 {
            return Err(self.diagnostic(
                DiagnosticCode::Limit,
                "equipment_defenses",
                "too many equipment bindings",
            ));
        }
        for (index, binding) in self.equipment_defenses.iter().enumerate() {
            let path = format!("equipment_defenses[{index}]");
            if binding.equipment.is_empty()
                || binding.equipment.len() > 128
                || binding.equipment.trim() != binding.equipment
                || binding.equipment.chars().any(char::is_control)
                || !equipment.insert(&binding.equipment)
            {
                return Err(self.diagnostic(
                    DiagnosticCode::Reference,
                    &format!("{path}.equipment"),
                    "invalid or duplicate equipment binding",
                ));
            }
            binding.compile_wearer_amounts(&ExprLimits::default()).map_err(|(p, reason)|
                self.diagnostic(DiagnosticCode::Native, &format!("{path}.{p}"), reason))?;
            binding.program.validate().map_err(|(p, reason)| {
                self.diagnostic(
                    DiagnosticCode::Native,
                    &format!("{path}.program.{p}"),
                    reason,
                )
            })?;
        }
        let mut equipment = std::collections::BTreeSet::new();
        if self.equipment_supports.len() > 128 {
            return Err(self.diagnostic(
                DiagnosticCode::Limit,
                "equipment_supports",
                "too many equipment bindings",
            ));
        }
        for (index, binding) in self.equipment_supports.iter().enumerate() {
            let path = format!("equipment_supports[{index}]");
            if binding.equipment.is_empty()
                || binding.equipment.len() > 128
                || binding.equipment.trim() != binding.equipment
                || binding.equipment.chars().any(char::is_control)
                || !equipment.insert(&binding.equipment)
            {
                return Err(self.diagnostic(
                    DiagnosticCode::Reference,
                    &format!("{path}.equipment"),
                    "invalid or duplicate equipment binding",
                ));
            }
            binding.compile_received_heal(&ExprLimits::default()).map_err(|(p,reason)|self.diagnostic(DiagnosticCode::Native,&format!("{path}.{p}"),reason))?;
            binding.program.validate().map_err(|(p, reason)| {
                self.diagnostic(
                    DiagnosticCode::Native,
                    &format!("{path}.program.{p}"),
                    reason,
                )
            })?;
        }
        let mut equipment = std::collections::BTreeSet::new();
        if self.equipment_states.len() > 128 {
            return Err(self.diagnostic(
                DiagnosticCode::Limit,
                "equipment_states",
                "too many equipment bindings",
            ));
        }
        for (index, binding) in self.equipment_states.iter().enumerate() {
            let path = format!("equipment_states[{index}]");
            if binding.equipment.is_empty()
                || binding.equipment.len() > 128
                || binding.equipment.trim() != binding.equipment
                || binding.equipment.chars().any(char::is_control)
                || !equipment.insert(&binding.equipment)
            {
                return Err(self.diagnostic(
                    DiagnosticCode::Reference,
                    &format!("{path}.equipment"),
                    "invalid or duplicate equipment binding",
                ));
            }
            binding
                .program
                .compile_equipped(&resources, None, &ExprLimits::default())
                .map_err(|reason| {
                    self.diagnostic(DiagnosticCode::Native, &format!("{path}.program"), reason)
                })?;
        }
        Ok(skills)
    }
    pub fn to_ron(&self) -> Result<String, Diagnostic> {
        self.load_native()?;
        let text = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| self.diagnostic(DiagnosticCode::Native, "artifact", e.to_string()))?;
        if text.len() > MAX_ARTIFACT_BYTES {
            return Err(self.diagnostic(
                DiagnosticCode::Limit,
                "artifact",
                "artifact byte limit exceeded",
            ));
        }
        Ok(text)
    }
    pub fn from_ron(text: &str) -> Result<Self, String> {
        if text.len() > MAX_ARTIFACT_BYTES {
            return Err("composition artifact byte limit exceeded".into());
        }
        let artifact: Self = ron::de::from_str(text).map_err(|e| e.to_string())?;
        artifact.load_native().map_err(|e| e.to_string())?;
        Ok(artifact)
    }
}
