//! Closed portable operations. Deserialization is not execution authority: the
//! loader validates the target and every operation before binding native skills.
mod attack_overlay;
pub use attack_overlay::AttackOverlayProgram;
mod native;
mod channel;
pub use channel::ChannelGroundProgram;
mod taunt;
pub use taunt::{TauntProgram, WeaponGrowthBinding};
mod tile_burst;
pub use tile_burst::{TileBurstProgram, WeaponMarkBinding};
mod content;
pub use content::{NativeGraphContent, NativeGraphCatalogs};
mod strike;
pub use strike::{StrikeProgram, StrikePart, StrikeStatus};
mod ground;
pub use ground::{GroundProgram, GroundFirstTick, GroundEffect, GroundStatusAreaProgram};
mod support;
pub use support::{
    SupportConversionProgram, SupportEffectProgram, SupportProgram, SupportReplacementProgram,
};
mod parry;
pub use parry::ParryProgram;
mod scheduled_area;
pub use scheduled_area::ScheduledAreaProgram;
mod delayed_area;
use super::{Diagnostic, Location};
use crate::game::{
    ability::{resonance::ResonanceAdmission, SkillId},
    data::skill_data::SkillDatabase,
};
pub use delayed_area::DelayedAreaProgram;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const IR_VERSION: u32 = 1;
pub const NATIVE_TARGET: &str = "native-manual-employee-v1";
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GraphOperation {
    ResonanceStatusStrike { program: crate::game::ability::status_strike::StatusStrikeDef },
    ResonanceFusionPlacement { program: crate::game::ability::mobile_effect::FusionPlacementDef },
    ResonancePairedAreas { program: crate::game::ability::paired_areas::PairedAreasDef },
    ResonancePreparedVolley { program: crate::game::ability::prepared_volley::PreparedVolleyDef },
    ResonanceInputPlan { program: crate::game::ability::input_plan::InputPlanDef },
    ResonanceRelationAction { program: crate::game::ability::relation_action::RelationActionDef },
    ResonanceAttackOverlay { admission: ResonanceAdmission, program: AttackOverlayProgram },
    ResonanceCounterChannel { program: crate::game::ability::counter_channel::CounterChannelDef },
    ResonanceTileBurst { admission: ResonanceAdmission, program: TileBurstProgram },
    ResonancePresenceBurst { admission: ResonanceAdmission, program: crate::game::ability::marks::PresenceBurstProgram },
    ResonanceChannelGround { admission: ResonanceAdmission, program: ChannelGroundProgram },
    ResonanceTaunt { admission: ResonanceAdmission, program: TauntProgram },
    ResonanceStrike { admission: ResonanceAdmission, program: StrikeProgram },
    ResonanceGroundStatus { admission: ResonanceAdmission, program: GroundStatusAreaProgram },
    ResonanceGround { admission: ResonanceAdmission, program: GroundProgram },
    ResonanceSupport {
        admission: ResonanceAdmission,
        program: SupportProgram,
    },
    /// Atomic request admission, optional receipt observation, and explicit end.
    /// Further payload families require explicit variants with their own validator.
    ResonanceUse { admission: ResonanceAdmission },
    ResonanceParry {
        admission: ResonanceAdmission,
        program: ParryProgram,
    },
    ResonancePreparedBasic {
        admission: ResonanceAdmission,
        profile: crate::game::ability::sustained::PreparedBasicProfile,
    },
    ResonanceScheduledArea {
        admission: ResonanceAdmission,
        program: ScheduledAreaProgram,
    },
    ResonanceDelayedArea {
        admission: ResonanceAdmission,
        program: DelayedAreaProgram,
    },
    ResonanceReactiveHeal {
        admission: ResonanceAdmission,
        duration_ms: u32,
        profile: crate::game::ability::sustained::ReactiveHealProfile,
        range: crate::game::battle::tile_range::TileRangePattern,
    },
    ResonanceState {
        admission: ResonanceAdmission,
        duration_ms: u32,
        profile: crate::game::ability::sustained::TargetStateProfile,
    },
}
impl GraphOperation {
    pub fn admission(&self) -> ResonanceAdmission {
        match self {
            Self::ResonanceStatusStrike { program } => program.admission,
            Self::ResonanceFusionPlacement { program } => program.admission,
            Self::ResonancePairedAreas { program } => program.admission,
            Self::ResonancePreparedVolley { program } => program.admission,
            Self::ResonanceInputPlan { program } => program.admission,
            Self::ResonanceRelationAction { program } => program.admission,
            Self::ResonanceCounterChannel { program } => program.entries[program.default_entry].admission,
            Self::ResonanceAttackOverlay { admission, .. }
            | Self::ResonanceTileBurst { admission, .. }
            | Self::ResonancePresenceBurst { admission, .. }
            | Self::ResonanceChannelGround { admission, .. }
            | Self::ResonanceTaunt { admission, .. }
            | Self::ResonanceUse { admission }
            | Self::ResonanceStrike { admission, .. }
            | Self::ResonanceGround { admission, .. }
            | Self::ResonanceGroundStatus { admission, .. }
            | Self::ResonanceSupport { admission, .. }
            | Self::ResonanceParry { admission, .. }
            | Self::ResonancePreparedBasic { admission, .. }
            | Self::ResonanceState { admission, .. }
            | Self::ResonanceScheduledArea { admission, .. }
            | Self::ResonanceDelayedArea { admission, .. }
            | Self::ResonanceReactiveHeal { admission, .. } => *admission,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortableGraph {
    pub(crate) format: String,
    pub(crate) version: u32,
    pub(crate) semantic_version: String,
    pub(crate) target: String,
    pub(crate) skill_id: SkillId,
    pub(crate) operation: GraphOperation,
    pub(crate) source_document: String,
    pub(crate) source_digest: String,
    pub(crate) semantic_fingerprint: String,
    pub(crate) source_map: BTreeMap<String, Vec<Location>>,
}
impl PortableGraph {
    pub fn operation(&self) -> &GraphOperation {
        &self.operation
    }
    pub fn source_map(&self) -> &BTreeMap<String, Vec<Location>> {
        &self.source_map
    }
    pub fn to_ron(&self) -> Result<String, String> {
        self.validate()?;
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| e.to_string())
    }
    pub fn from_ron(text: &str) -> Result<Self, String> {
        // Bound before allocating; RON's own recursion limit remains in force.
        if text.len() > 5_000_000 {
            return Err("graph artifact byte limit exceeded".into());
        }
        let graph: Self = ron::de::from_str(text).map_err(|e| e.to_string())?;
        graph.validate()?;
        Ok(graph)
    }
    fn validate(&self) -> Result<(), String> {
        if self.format != "skill-graph-ir"
            || self.version != IR_VERSION
            || self.semantic_version != super::SEMANTIC_VERSION
            || self.target != NATIVE_TARGET
        {
            return Err("unsupported graph artifact version or target".into());
        }
        if self.skill_id.trim().is_empty() || self.skill_id.len() > 128 {
            return Err("invalid graph skill identity".into());
        }
        if self.source_document.is_empty()
            || self.source_document.len() > 128
            || [&self.source_digest, &self.semantic_fingerprint]
                .iter()
                .any(|digest| digest.len() != 64 || !digest.bytes().all(|c| c.is_ascii_hexdigit()))
            || self.source_map.len() > 2000
        {
            return Err("invalid graph provenance".into());
        }
        let mut total = 0usize;
        for (key, locations) in &self.source_map {
            total = total
                .checked_add(locations.len())
                .ok_or("source map overflow")?;
            if key.len() > 256
                || locations.len() > 16000
                || total > 32000
                || locations.iter().any(|loc| {
                    [&loc.node, &loc.port, &loc.edge, &loc.field]
                        .iter()
                        .any(|s| s.as_ref().is_some_and(|s| s.len() > 256))
                })
            {
                return Err("graph source map limit exceeded".into());
            }
        }
        match &self.operation {
            GraphOperation::ResonanceStatusStrike { program } => program.validate(),
            GraphOperation::ResonanceFusionPlacement { program } => program.validate(),
            GraphOperation::ResonancePairedAreas { program } => program.validate(),
            GraphOperation::ResonancePreparedVolley { program } => program.validate(),
            GraphOperation::ResonanceInputPlan { program } => program.validate(),
            GraphOperation::ResonanceRelationAction { program } => program.validate(),
            GraphOperation::ResonanceAttackOverlay { admission, program } => { admission.validate().map_err(str::to_string)?; program.validate() }
            GraphOperation::ResonanceCounterChannel { program } => program.validate(),
            GraphOperation::ResonanceTileBurst { admission, program } => { admission.validate().map_err(str::to_string)?; program.validate() }
            GraphOperation::ResonancePresenceBurst { admission, program } => { admission.validate().map_err(str::to_string)?; program.validate() }
            GraphOperation::ResonanceChannelGround { admission, program } => { admission.validate().map_err(str::to_string)?; program.validate() }
            GraphOperation::ResonanceTaunt { admission, program } => { admission.validate().map_err(str::to_string)?; program.validate() }
            GraphOperation::ResonanceStrike { admission, program } => { admission.validate().map_err(str::to_string)?; program.validate() }
            GraphOperation::ResonanceGroundStatus { admission, program } => {
                admission.validate().map_err(str::to_string)?;
                program.validate()
            }
            GraphOperation::ResonanceGround { admission, program } => {
                admission.validate().map_err(str::to_string)?;
                program.validate()
            }
            GraphOperation::ResonanceSupport { admission, program } => {
                admission.validate().map_err(str::to_string)?;
                program.validate()
            }
            GraphOperation::ResonanceParry { admission, program } => {
                admission.validate().map_err(str::to_string)?;
                program.validate()
            }
            GraphOperation::ResonancePreparedBasic { admission, profile } => {
                admission.validate().map_err(str::to_string)?;
                profile.validate()
            }
            GraphOperation::ResonanceScheduledArea { admission, program } => {
                admission.validate().map_err(str::to_string)?;
                program.validate()
            }
            GraphOperation::ResonanceDelayedArea { admission, program } => {
                admission.validate().map_err(str::to_string)?;
                program.validate()
            }
            GraphOperation::ResonanceReactiveHeal {
                admission,
                duration_ms,
                profile,
                range,
            } => {
                admission.validate().map_err(str::to_string)?;
                if *duration_ms == 0 {
                    return Err("State requires positive duration".into());
                }
                profile.validate()?;
                range.validate()
            }
            GraphOperation::ResonanceUse { admission } => {
                admission.validate().map_err(str::to_string)
            }
            GraphOperation::ResonanceState {
                admission,
                duration_ms,
                profile,
            } => {
                admission.validate().map_err(str::to_string)?;
                if *duration_ms == 0 {
                    return Err("State requires positive duration".into());
                }
                profile.validate()
            }
        }
    }
    /// Uses the production authored-skill loader and its contract/Flow checks.
    /// Source-map entries never grant permission to execute an operation.
    pub fn load_native(&self) -> Result<SkillDatabase, String> {
        let content = self.load_content()?;
        if !content.equipment.is_empty() || !content.mark_weapons.is_empty() { return Err("This graph includes equipment companions; use load_content and bind_catalogs".into()); }
        Ok(content.skills)
    }
    pub fn load_content(&self) -> Result<NativeGraphContent, String> {
        self.validate()?;
        let equipment = match &self.operation {
            GraphOperation::ResonanceTaunt { program, .. } => program.weapon_growth.iter().cloned().collect(),
            _ => Vec::new(),
        };
        let skills = SkillDatabase::from_ron("compiled:node-graph", &self.native_ron()?).map_err(|e| e.to_string())?;
        let mark_weapons = match &self.operation {
            GraphOperation::ResonanceTileBurst { program, .. } => vec![program.weapon.clone()],
            _ => Vec::new(),
        };
        Ok(NativeGraphContent { skills, equipment, mark_weapons })
    }
    pub(crate) fn native_ron(&self) -> Result<String, String> {
        self.validate()?;
        native::bind(&self.operation, &self.skill_id)
    }
}

pub(super) fn issue(
    doc: &super::NormalizedDocument,
    node: Option<&str>,
    port: Option<&str>,
    code: &str,
    reason: &str,
) -> Diagnostic {
    Diagnostic {
        code: code.into(),
        phase: "lowering".into(),
        document: Some(doc.document.clone()),
        digest: doc.digest.clone(),
        primary: Location {
            node: node.map(Into::into),
            port: port.map(Into::into),
            ..Default::default()
        },
        related: vec![],
        reason: reason.into(),
        expected: None,
        actual: None,
    }
}
