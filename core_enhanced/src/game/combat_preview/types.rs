use crate::game::{
    battle::types::{BattleUnitStatScale, DeploymentAffinity},
    combat_setup::mission_policy::CombatMissionPolicy,
    data::pve_data::PveWaveActivationData,
    enums::{RiskLevel, Tier},
    map::{MapNodeCategory, MapNodeId},
    resources::Position,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BattlefieldArchetype {
    OpenHall,
    Corridor,
    ChokePoint,
    Ambush,
    Surrounded,
    SplitRoom,
    ObstacleRoom,
    BossArena,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CombatNodeType {
    Defense,
    Boss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CombatMissionVariant {
    Defense,
    Boss,
}

impl CombatMissionVariant {
    pub fn default_for_node_type(node_type: CombatNodeType) -> Self {
        match node_type {
            CombatNodeType::Defense => Self::Defense,
            CombatNodeType::Boss => Self::Boss,
        }
    }

    pub fn is_compatible_with(self, node_type: CombatNodeType) -> bool {
        matches!(
            (node_type, self),
            (CombatNodeType::Defense, Self::Defense) | (CombatNodeType::Boss, Self::Boss)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CombatMissionRisk {
    Controlled,
    Unstable,
    Collapse,
}

impl CombatMissionRisk {
    pub fn from_risk_level(risk_level: RiskLevel) -> Self {
        CombatMissionPolicy::mission_risk_from_risk_level(risk_level)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BattlefieldSizeClass {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZoneConfidence {
    Confirmed,
    Likely,
    Suspected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpawnZoneKind {
    Entry,
    Interior,
    BossAnchor,
    Ambush,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BattlefieldTileKind {
    Ground,
    Platform,
    Obstacle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattlefieldTile {
    pub position: Position,
    pub kind: BattlefieldTileKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnemyKind {
    CorrodedEmployee,
    Abnormality,
    FacilityEntity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentZone {
    pub id: String,
    pub label: String,
    pub kind: DeploymentZoneKind,
    pub cells: Vec<Position>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeploymentZoneKind {
    Ground,
    Platform,
}

impl DeploymentZoneKind {
    pub fn supports_affinity(self, affinity: DeploymentAffinity) -> bool {
        match self {
            DeploymentZoneKind::Ground => affinity.allows_ground(),
            DeploymentZoneKind::Platform => affinity.allows_platform(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnZone {
    pub id: String,
    pub label: String,
    pub kind: SpawnZoneKind,
    pub confidence: ZoneConfidence,
    pub cells: Vec<Position>,
    pub revealed_details: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyBriefing {
    pub kind: EnemyKind,
    pub abnormality_id: String,
    pub display_name: String,
    pub risk_level: RiskLevel,
    pub role: String,
    pub count_hint: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatWarningTag {
    ArmoredEnemyPossible,
    HighMagicResistEnemyPossible,
    AirEnemyPossible,
    HardToBlockEnemyPossible,
    ShieldedEnemyPossible,
    RegeneratingEnemyPossible,
    FastBreakthroughEnemyPossible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatWarningStatus {
    Unverified,
    Disproved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatWarningSource {
    Briefing,
    Rumor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreatWarning {
    pub tag: ThreatWarningTag,
    pub status: ThreatWarningStatus,
    pub source: ThreatWarningSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnWaveEnemyEntry {
    pub kind: EnemyKind,
    pub profile_id: Option<String>,
    pub abnormality_id: String,
    pub tier: Tier,
    pub count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub appearance_seeds: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnWave {
    pub id: String,
    pub activation: PveWaveActivationData,
    pub spawn_zone_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_id: Option<String>,
    pub enemy_entries: Vec<SpawnWaveEnemyEntry>,
    pub required_for_victory: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattlefieldRoute {
    pub id: String,
    pub start: Position,
    pub end: Position,
    pub cells: Vec<Position>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattlefieldInstance {
    pub battlefield_template_id: String,
    pub node_type: CombatNodeType,
    pub mission_variant: CombatMissionVariant,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub survive_timer_ms: Option<u64>,
    pub mission_risk: CombatMissionRisk,
    pub archetype: BattlefieldArchetype,
    pub size_class: BattlefieldSizeClass,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<BattlefieldTile>,
    pub valid_tiles: Vec<Position>,
    pub deployment_zones: Vec<DeploymentZone>,
    pub spawn_zones: Vec<SpawnZone>,
    pub routes: Vec<BattlefieldRoute>,
    pub spawn_waves: Vec<SpawnWave>,
    pub obstacles: Vec<Position>,
    pub enemy_briefing: Vec<EnemyBriefing>,
    pub threat_warnings: Vec<ThreatWarning>,
    /// Internal battle setup scale. Serialized previews are UI hints and must
    /// not be round-tripped into battle setup as the scaling source of truth.
    #[serde(skip)]
    pub enemy_stat_scale: BattleUnitStatScale,
}

#[derive(Debug, Clone, Copy)]
pub struct BattlefieldGenerationRequest<'a> {
    pub category: MapNodeCategory,
    pub encounter_id: Option<&'a str>,
    pub seed: u64,
    /// Retained for public source compatibility. Battlefield generation deliberately
    /// derives combat tuning from manifested difficulty, never from the current Floor.
    pub floor_index: u32,
}

pub struct BattlefieldGenerator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleDeterrenceSnapshot {
    pub unrestrained: bool,
    pub seal_released: bool,
    pub enemy_health_percent: u32,
    pub enemy_attack_percent: u32,
    pub victory_quantity_multiplier: u32,
}
impl Default for BattleDeterrenceSnapshot {
    fn default() -> Self {
        Self { unrestrained: false, seal_released: false, enemy_health_percent: 100,
            enemy_attack_percent: 100, victory_quantity_multiplier: 1 }
    }
}
impl BattleDeterrenceSnapshot {
    pub(crate) fn quantity_multiplier(unrestrained: bool) -> u32 {
        if unrestrained { 2 } else { 1 }
    }

    pub(crate) fn for_entry(tuning: crate::game::data::pve_data::PveDeterrenceTuning, unrestrained: bool, sealed: bool) -> Self {
        Self {
            unrestrained, seal_released: sealed,
            enemy_health_percent: if unrestrained { tuning.unrestrained_health_percent } else if sealed { tuning.seal_health_percent } else { 100 },
            enemy_attack_percent: if unrestrained { tuning.unrestrained_attack_percent } else if sealed { tuning.seal_attack_percent } else { 100 },
            victory_quantity_multiplier: Self::quantity_multiplier(unrestrained),
        }
    }
}
crate::authoring::clone_shape::struct_shape!(BattleDeterrenceSnapshot {
    unrestrained, seal_released, enemy_health_percent, enemy_attack_percent, victory_quantity_multiplier
});

/// Unity-facing pre-battle preview DTO and battle setup seed.
///
/// This struct is intentionally serialized in `game_state_context.combat_preview`
/// for node confirmation and preview UI. Internal-only setup fields must stay
/// out of the wire contract, or be explicitly marked as skipped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatPreview {
    #[serde(default)]
    pub deterrence: BattleDeterrenceSnapshot,
    pub node_id: MapNodeId,
    pub encounter_id: Option<String>,
    pub battlefield_template_id: String,
    pub node_type: CombatNodeType,
    pub mission_variant: CombatMissionVariant,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub survive_timer_ms: Option<u64>,
    pub mission_risk: CombatMissionRisk,
    pub archetype: BattlefieldArchetype,
    pub size_class: BattlefieldSizeClass,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<BattlefieldTile>,
    pub valid_tiles: Vec<Position>,
    pub deployment_zones: Vec<DeploymentZone>,
    pub spawn_zones: Vec<SpawnZone>,
    pub routes: Vec<BattlefieldRoute>,
    pub spawn_waves: Vec<SpawnWave>,
    pub obstacles: Vec<Position>,
    pub enemy_briefing: Vec<EnemyBriefing>,
    #[serde(default)]
    pub threat_warnings: Vec<ThreatWarning>,
    /// Internal battle setup scale. Serialized previews are UI hints and must
    /// not be round-tripped into battle setup as the scaling source of truth.
    #[serde(skip)]
    pub enemy_stat_scale: BattleUnitStatScale,
}

crate::authoring::clone_shape::struct_shape!(CombatPreview {
    deterrence,
    node_id,
    encounter_id,
    battlefield_template_id,
    node_type,
    mission_variant,
    survive_timer_ms,
    mission_risk,
    archetype,
    size_class,
    width,
    height,
    tiles,
    valid_tiles,
    deployment_zones,
    spawn_zones,
    routes,
    spawn_waves,
    obstacles,
    enemy_briefing,
    threat_warnings,
    enemy_stat_scale
});
