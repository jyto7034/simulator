use crate::game::{
    battle::types::DeploymentAffinity,
    combat_mission_policy::CombatMissionPolicy,
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
    pub time_ms: u32,
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
}

#[derive(Debug, Clone, Copy)]
pub struct BattlefieldGenerationRequest<'a> {
    pub category: MapNodeCategory,
    pub encounter_id: Option<&'a str>,
    pub seed: u64,
}

pub struct BattlefieldGenerator;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatPreview {
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
}
