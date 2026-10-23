use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    game::resources::{
        item_slot::{EquipmentSlotAddress, SlotLayoutKind},
        Position,
    },
    game::{
        ability::{SkillActivationMode, SkillId},
        battle::core::movement::types::WorldVec2,
        battle::damage::DamageModifiers,
        battle::ids::UnitInstanceId,
        battle::result_stats::BattleResultStatsDto,
        battle::tile_range::TileRangePattern,
        behavior::GameError,
        data::{
            abnormality_data::{BasicAttackDef, MovementDef, ResonanceDef},
            equipment_data::{EquipmentType, WeaponCombatProfile, WeaponRangeRole},
            GameDataBase,
        },
        enums::{Side, Tier},
        growth::GrowthStack,
        stats::UnitStats,
    },
};

pub struct BattleResult {
    pub runtime_facts: std::sync::Arc<super::runtime_facts::RuntimeFactSnapshot>,
    pub outcome_id: BattleOutcomeId,
    pub verdict: BattleVerdict,
    pub winner: BattleWinner,
    pub result_stats: BattleResultStatsDto,
    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub event_log: crate::game::battle::event_log::BattleEventLog,
    pub participant_results: Vec<ParticipantBattleResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BattleOutcomeId(pub String);

impl BattleOutcomeId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleVerdict {
    Victory,
    Defeat,
}

impl BattleVerdict {
    pub const fn winner(self) -> BattleWinner {
        match self {
            Self::Victory => BattleWinner::Player,
            Self::Defeat => BattleWinner::Opponent,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParticipantBattleResult {
    pub unit_instance_id: UnitInstanceId,
    pub owned_uuid: Uuid,
    pub side: Side,
    pub survived: bool,
    pub final_hp: u32,
    pub max_hp: u32,
    pub became_incapacitated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleWinner {
    Player,
    Opponent,
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UnitExclusiveActivity {
    #[default]
    Ready,
    FieldWorking,
}

/// 어빌리티 실행에 필요한 유닛 정보
#[derive(Debug, Clone)]
pub struct UnitSnapshot {
    pub id: UnitInstanceId,
    pub owner: Side,
    pub role: BattleUnitRole,
    pub threat_class: BattleUnitThreatClass,
    pub mobility_kind: MobilityKind,
    pub position: Position,
    pub world_position: WorldVec2,
    pub stats: UnitStats,
}

crate::authoring::clone_shape::struct_shape!(UnitSnapshot {
    id,
    owner,
    role,
    threat_class,
    mobility_kind,
    position,
    world_position,
    stats,
});

/// Definition and, when known, the actual inventory copy entering battle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BattleEquipmentBinding {
    pub weapon_stage: crate::game::ability::weapon_marks::WeaponStage,
    pub base_uuid: Uuid,
    pub persistent_copy_uuid: Option<Uuid>,
    pub slot: Option<EquipmentSlotAddress>,
}

crate::authoring::clone_shape::struct_shape!(BattleEquipmentBinding {
    weapon_stage,
    base_uuid,
    persistent_copy_uuid,
    slot,
});

#[derive(Debug, Clone)]
pub struct BattleUnitDraft {
    pub owned_uuid: Uuid,
    pub source: BattleUnitSource,
    pub threat_class: BattleUnitThreatClass,
    pub level: Tier,
    pub stat_scale: BattleUnitStatScale,
    pub growth_stacks: GrowthStack,
    pub equipment_layout: Option<SlotLayoutKind>,
    pub equipped_items: Vec<BattleEquipmentBinding>,
    pub equipped_item_enhancements: Vec<BattleEquipmentEnhancement>,
    pub employee_start_condition: Option<EmployeeBattleStartCondition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmployeeBattleStartCondition {
    pub trauma: u32,
    pub battle_hp_bonus_percent: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BattleUnitStatScale {
    pub max_health_percent: u32,
    pub attack_percent: u32,
    pub defense_percent: u32,
    pub exact_override: Option<BattleUnitExactStatOverride>,
    pub encounter_health_percent: u32,
    pub encounter_attack_percent: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BattleUnitExactStatOverride {
    pub max_health: u32,
    pub attack: u32,
    pub defense: i32,
    pub magic_resistance: i32,
}

impl Default for BattleUnitStatScale {
    fn default() -> Self {
        Self {
            max_health_percent: 100,
            attack_percent: 100,
            defense_percent: 100,
            exact_override: None,
            encounter_health_percent: 100,
            encounter_attack_percent: 100,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BattleEquipmentEnhancement {
    pub base_uuid: Uuid,
    pub persistent_copy_uuid: Option<Uuid>,
    pub enhancement_level: u8,
}

#[derive(Debug, Clone)]
pub enum BattleUnitSource {
    Employee(UnitCombatProfile),
    Abnormality {
        base_uuid: Uuid,
    },
    CorrodedEmployee {
        profile_id: String,
        base_uuid: Uuid,
    },
    TestFixture {
        base_uuid: Uuid,
        profile: UnitCombatProfile,
    },
    DefenseObject {
        base_uuid: Uuid,
        profile: UnitCombatProfile,
    },
    IndependentObject {
        definition_id: String,
        base_uuid: Uuid,
        profile: UnitCombatProfile,
        creator_instance_id: crate::game::battle::ids::UnitInstanceId,
        kind: IndependentUnitKind,
    },
    SummonedUnit {
        definition_id: String,
        base_uuid: Uuid,
        profile: UnitCombatProfile,
        summoner_instance_id: crate::game::battle::ids::UnitInstanceId,
        summon_sequence: u64,
        batch_uuid: Uuid,
    },
}

crate::authoring::clone_shape::struct_shape!(BattleUnitDraft {
    owned_uuid,
    source,
    threat_class,
    level,
    stat_scale,
    growth_stacks,
    equipment_layout,
    equipped_items,
    equipped_item_enhancements,
    employee_start_condition,
});
crate::authoring::clone_shape::copy_shape!(
    EmployeeBattleStartCondition,
    BattleUnitStatScale,
    BattleEquipmentEnhancement,
    crate::game::resources::item_slot::EquipmentSlotAddress,
    crate::game::resources::item_slot::SlotLayoutKind,
    crate::game::enums::Tier,
    BattleVerdict
);
crate::authoring::clone_shape::newtype_shape!(BattleOutcomeId);
crate::authoring::clone_shape::enum_shape!(BattleUnitSource {
    units: [],
    tuples: [Employee(profile)],
    records: [
        Abnormality { base_uuid },
        CorrodedEmployee {
            profile_id,
            base_uuid
        },
        TestFixture { base_uuid, profile },
        DefenseObject { base_uuid, profile },
        IndependentObject {
            definition_id,
            base_uuid,
            profile,
            creator_instance_id,
            kind
        },
        SummonedUnit {
            definition_id,
            base_uuid,
            profile,
            summoner_instance_id,
            summon_sequence,
            batch_uuid
        }
    ]
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BattleUnitSourceIdentity {
    Employee {
        employee_uuid: Uuid,
        base_uuid: Uuid,
    },
    Abnormality {
        abnormality_id: String,
        base_uuid: Uuid,
    },
    CorrodedEmployee {
        profile_id: String,
        base_uuid: Uuid,
    },
    FacilityEntity {
        profile_id: String,
        base_uuid: Uuid,
    },
    DefenseObject {
        base_uuid: Uuid,
    },
    TestFixture {
        base_uuid: Uuid,
    },
    IndependentObject {
        definition_id: String,
        base_uuid: Uuid,
        creator_instance_id: crate::game::battle::ids::UnitInstanceId,
        #[serde(rename = "object_kind")]
        kind: IndependentUnitKind,
    },
    SummonedUnit {
        definition_id: String,
        base_uuid: Uuid,
        summoner_instance_id: crate::game::battle::ids::UnitInstanceId,
        summon_sequence: u64,
        batch_uuid: Uuid,
    },
}

impl BattleUnitSourceIdentity {
    pub fn base_uuid(&self) -> Uuid {
        match self {
            Self::Employee { base_uuid, .. }
            | Self::Abnormality { base_uuid, .. }
            | Self::CorrodedEmployee { base_uuid, .. }
            | Self::FacilityEntity { base_uuid, .. }
            | Self::DefenseObject { base_uuid }
            | Self::TestFixture { base_uuid }
            | Self::IndependentObject { base_uuid, .. }
            | Self::SummonedUnit { base_uuid, .. } => *base_uuid,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndependentUnitKind {
    Actor,
    Wall,
}
crate::authoring::clone_shape::copy_shape!(IndependentUnitKind);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BattleUnitRole {
    #[default]
    Combatant,
    DefenseObject,
    IndependentWall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BattleUnitThreatClass {
    #[default]
    Normal,
    Elite,
    Boss,
}

impl BattleUnitThreatClass {
    pub fn uses_resonance_bar(self) -> bool {
        matches!(self, Self::Elite | Self::Boss)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentAffinity {
    #[default]
    GroundOnly,
    PlatformOnly,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EngagementStance {
    #[default]
    Attack,
    Defense,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TacticalTileKind {
    Ground,
    Platform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MobilityKind {
    #[default]
    Ground,
    Airborne,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitTargetTrait {
    Airborne,
    /// Explicit control immunity; size, height and boss class do not imply it.
    TauntImmune,
    SlowImmune,
    StunImmune,
}

impl MobilityKind {
    pub fn is_airborne(self) -> bool {
        matches!(self, Self::Airborne)
    }

    pub fn can_be_blocked(self) -> bool {
        matches!(self, Self::Ground)
    }
}

impl DeploymentAffinity {
    pub fn allows_ground(self) -> bool {
        matches!(
            self,
            DeploymentAffinity::GroundOnly | DeploymentAffinity::Any
        )
    }

    pub fn allows_platform(self) -> bool {
        matches!(
            self,
            DeploymentAffinity::PlatformOnly | DeploymentAffinity::Any
        )
    }
}

impl BattleUnitSource {
    pub fn base_uuid(&self, owned_uuid: Uuid) -> Uuid {
        match self {
            BattleUnitSource::Employee(_) => owned_uuid,
            BattleUnitSource::Abnormality { base_uuid } => *base_uuid,
            BattleUnitSource::CorrodedEmployee { base_uuid, .. } => *base_uuid,
            BattleUnitSource::TestFixture { base_uuid, .. } => *base_uuid,
            BattleUnitSource::DefenseObject { base_uuid, .. } => *base_uuid,
            BattleUnitSource::IndependentObject { base_uuid, .. }
            | BattleUnitSource::SummonedUnit { base_uuid, .. } => *base_uuid,
        }
    }

    pub fn role(&self) -> BattleUnitRole {
        match self {
            BattleUnitSource::DefenseObject { .. } => BattleUnitRole::DefenseObject,
            BattleUnitSource::IndependentObject {
                kind: IndependentUnitKind::Wall,
                ..
            } => BattleUnitRole::IndependentWall,
            _ => BattleUnitRole::Combatant,
        }
    }

    pub fn source_identity(
        &self,
        owned_uuid: Uuid,
        game_data: &GameDataBase,
    ) -> Result<BattleUnitSourceIdentity, GameError> {
        Ok(match self {
            BattleUnitSource::Employee(_) => BattleUnitSourceIdentity::Employee {
                employee_uuid: owned_uuid,
                base_uuid: owned_uuid,
            },
            BattleUnitSource::Abnormality { base_uuid } => {
                let meta = game_data
                    .abnormality_data
                    .get_by_uuid(base_uuid)
                    .ok_or(GameError::MissingResource("AbnormalityMetadata"))?;
                BattleUnitSourceIdentity::Abnormality {
                    abnormality_id: meta.id.clone(),
                    base_uuid: *base_uuid,
                }
            }
            BattleUnitSource::CorrodedEmployee {
                profile_id,
                base_uuid,
            } => BattleUnitSourceIdentity::CorrodedEmployee {
                profile_id: profile_id.clone(),
                base_uuid: *base_uuid,
            },
            BattleUnitSource::TestFixture { base_uuid, .. } => {
                BattleUnitSourceIdentity::TestFixture {
                    base_uuid: *base_uuid,
                }
            }
            BattleUnitSource::DefenseObject { base_uuid, .. } => {
                BattleUnitSourceIdentity::DefenseObject {
                    base_uuid: *base_uuid,
                }
            }
            BattleUnitSource::IndependentObject {
                definition_id,
                base_uuid,
                creator_instance_id,
                kind,
                ..
            } => BattleUnitSourceIdentity::IndependentObject {
                definition_id: definition_id.clone(),
                base_uuid: *base_uuid,
                creator_instance_id: *creator_instance_id,
                kind: *kind,
            },
            BattleUnitSource::SummonedUnit {
                definition_id,
                base_uuid,
                summoner_instance_id,
                summon_sequence,
                batch_uuid,
                ..
            } => BattleUnitSourceIdentity::SummonedUnit {
                definition_id: definition_id.clone(),
                base_uuid: *base_uuid,
                summoner_instance_id: *summoner_instance_id,
                summon_sequence: *summon_sequence,
                batch_uuid: *batch_uuid,
            },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitCombatProfile {
    pub stats: UnitStats,
    pub basic_attack: BasicAttackDef,
    pub basic_attack_speed_percent: u32,
    pub weapon_profile: Option<WeaponCombatProfile>,
    pub movement: MovementDef,
    pub resonance: ResonanceDef,
    pub skill_id: Option<SkillId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fragment_source: Option<crate::game::skill_fragment::FragmentSourceSnapshot>,
    pub skill_activation_mode: SkillActivationMode,
    pub deployment_affinity: DeploymentAffinity,
    pub block_capacity: u32,
    pub block_radius_units: f32,
    pub blockable: bool,
    pub mobility_kind: MobilityKind,
    pub target_traits: Vec<UnitTargetTrait>,
    pub incoming_damage_modifiers: DamageModifiers,
}

pub fn default_basic_attack_tile_range_for_role(range_role: WeaponRangeRole) -> TileRangePattern {
    match range_role {
        WeaponRangeRole::Melee => TileRangePattern {
            include_anchor_tile: true,
            rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
        },
        WeaponRangeRole::Ranged => TileRangePattern {
            include_anchor_tile: true,
            rows: vec![
                "XXX".to_string(),
                "XXX".to_string(),
                "XXX".to_string(),
                ".@.".to_string(),
            ],
        },
    }
}

pub fn effective_basic_attack_tile_range(basic_attack: &BasicAttackDef) -> TileRangePattern {
    basic_attack
        .defense_tile_range
        .clone()
        .unwrap_or_else(|| default_basic_attack_tile_range_for_role(basic_attack.range_role))
}

impl UnitCombatProfile {
    pub(crate) fn employee_base_resonance() -> ResonanceDef {
        ResonanceDef {
            start: 20,
            max: 100,
            gain_lock_ms: 0,
        }
    }

    pub(crate) fn has_employee_base_resonance(&self) -> bool {
        self.resonance.start == 20 && self.resonance.max == 100 && self.resonance.gain_lock_ms == 0
    }
    pub fn employee_default() -> Self {
        Self::for_employee(100, 10, 0, 0, 100)
    }

    pub fn for_employee(
        max_health: u32,
        attack: u32,
        defense: i32,
        magic_resist: i32,
        basic_attack_speed_percent: u32,
    ) -> Self {
        assert!(basic_attack_speed_percent > 0);
        let basic_attack = BasicAttackDef {
            defense_tile_range: Some(default_basic_attack_tile_range_for_role(
                WeaponRangeRole::Melee,
            )),
            interval_ms: 1500,
            windup_ms: 200,
            delivery: Default::default(),
            ..BasicAttackDef::default()
        };
        let movement = MovementDef::default();
        let interval_ms =
            scale_attack_interval_ms(basic_attack.interval_ms, basic_attack_speed_percent);
        let mut basic_attack = basic_attack;
        basic_attack.interval_ms = interval_ms;
        let mut stats = UnitStats::with_values(
            max_health,
            max_health,
            attack,
            defense,
            basic_attack.interval_ms,
        );
        stats.magic_resist = magic_resist;
        stats.move_speed_units_per_ms = movement.speed_units_per_ms;

        Self {
            stats,
            basic_attack,
            basic_attack_speed_percent,
            weapon_profile: None,
            movement,
            resonance: Self::employee_base_resonance(),
            skill_id: None,
            fragment_source: None,
            skill_activation_mode: SkillActivationMode::Auto,
            deployment_affinity: DeploymentAffinity::GroundOnly,
            block_capacity: 0,
            block_radius_units: 0.75,
            blockable: true,
            mobility_kind: MobilityKind::Ground,
            target_traits: Vec::new(),
            incoming_damage_modifiers: DamageModifiers::default(),
        }
    }

    pub fn apply_weapon_profile(
        &mut self,
        weapon_profile: &WeaponCombatProfile,
    ) -> Result<(), GameError> {
        weapon_profile.validate_runtime_contract("equipped weapon")?;
        let interval_ms = checked_scale_attack_interval_ms(
            weapon_profile.interval_ms,
            self.basic_attack_speed_percent,
        )?;
        self.basic_attack.action.clone_from(&weapon_profile.action);
        self.basic_attack.range_policy = crate::game::battle::tile_range::TileRangePolicy::Pattern;
        self.basic_attack.defense_tile_range = Some(weapon_profile.defense_tile_range.clone());
        self.basic_attack.damage_type = weapon_profile.damage_type;
        self.basic_attack.targeting_profile = weapon_profile.targeting_profile;
        self.basic_attack.air_capable = weapon_profile.air_capable;
        self.basic_attack.range_role = weapon_profile.range_role;
        self.basic_attack.interval_ms = interval_ms;
        self.basic_attack.windup_ms = weapon_profile.windup_ms;
        self.basic_attack.ranged_reposition_ms = weapon_profile.ranged_reposition_ms;
        self.basic_attack.delivery = weapon_profile.delivery.clone();
        self.stats.attack_interval_ms = interval_ms;
        self.deployment_affinity = weapon_profile.deployment_affinity;
        self.block_capacity = weapon_profile.block_capacity;
        self.weapon_profile = Some(weapon_profile.clone());
        Ok(())
    }
}

pub fn checked_scale_attack_interval_ms(
    base_interval_ms: u64,
    speed_percent: u32,
) -> Result<u64, GameError> {
    if base_interval_ms == 0 || speed_percent == 0 {
        return Err(GameError::InvalidStaticData(
            "attack interval and speed points must be > 0".into(),
        ));
    }
    u64::try_from((u128::from(base_interval_ms) * 100).div_ceil(u128::from(speed_percent)))
        .map_err(|_| GameError::InvalidStaticData("scaled attack interval overflows u64".into()))
}

pub fn scale_attack_interval_ms(base_interval_ms: u64, speed_percent: u32) -> u64 {
    checked_scale_attack_interval_ms(base_interval_ms, speed_percent)
        .expect("validated attack cadence")
}

impl BattleUnitDraft {
    pub fn base_uuid(&self) -> Uuid {
        self.source.base_uuid(self.owned_uuid)
    }

    pub fn combat_profile_from_abnormality(
        base_uuid: Uuid,
        game_data: &GameDataBase,
    ) -> Result<UnitCombatProfile, GameError> {
        let meta = game_data
            .abnormality_data
            .get_by_uuid(&base_uuid)
            .ok_or(GameError::MissingResource("AbnormalityMetadata"))?;

        if meta.basic_attack.interval_ms == 0 {
            return Err(GameError::InvalidUnitStats(
                "attack_interval_ms must be > 0",
            ));
        }

        let mut stats = UnitStats::with_values(
            meta.max_health,
            meta.max_health,
            meta.attack,
            meta.defense,
            meta.basic_attack.interval_ms,
        );
        stats.magic_resist = meta.magic_resist;
        stats.move_speed_units_per_ms = meta.movement.speed_units_per_ms;

        Ok(UnitCombatProfile {
            stats,
            basic_attack: meta.basic_attack.clone(),
            basic_attack_speed_percent: 100,
            weapon_profile: None,
            movement: meta.movement.clone(),
            resonance: meta.resonance.clone(),
            skill_id: meta.skill_id.clone(),
            fragment_source: None,
            skill_activation_mode: SkillActivationMode::Auto,
            deployment_affinity: DeploymentAffinity::GroundOnly,
            block_capacity: 0,
            block_radius_units: 0.0,
            blockable: meta.mobility_kind.can_be_blocked(),
            mobility_kind: meta.mobility_kind,
            target_traits: effective_target_traits(meta.mobility_kind, &meta.target_traits),
            incoming_damage_modifiers: DamageModifiers::default(),
        })
    }

    pub fn combat_profile(&self, game_data: &GameDataBase) -> Result<UnitCombatProfile, GameError> {
        let mut profile = match &self.source {
            BattleUnitSource::Employee(profile) => Ok(profile.clone()),
            BattleUnitSource::Abnormality { base_uuid } => {
                Self::combat_profile_from_abnormality(*base_uuid, game_data)
            }
            BattleUnitSource::CorrodedEmployee { profile_id, .. } => game_data
                .corroded_employee_data
                .get_by_id(profile_id)
                .map(|profile| profile.to_combat_profile())
                .ok_or(GameError::MissingResource("CorrodedEmployeeProfile")),
            BattleUnitSource::TestFixture { profile, .. } => Ok(profile.clone()),
            BattleUnitSource::DefenseObject { profile, .. } => Ok(profile.clone()),
            BattleUnitSource::IndependentObject { profile, .. }
            | BattleUnitSource::SummonedUnit { profile, .. } => Ok(profile.clone()),
        }?;

        if matches!(&self.source, BattleUnitSource::Employee(_)) {
            let mut weapon_profile: Option<&WeaponCombatProfile> = None;
            for binding in &self.equipped_items {
                let item = game_data
                    .equipment_data
                    .get_by_uuid(&binding.base_uuid)
                    .ok_or(GameError::MissingResource(""))?;
                if item.equipment_type != EquipmentType::Weapon {
                    continue;
                }
                let next_profile = item.weapon_profile.as_ref().ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "weapon equipment '{}' is missing weapon_profile",
                        item.id
                    ))
                })?;
                if weapon_profile.is_some() {
                    return Err(GameError::InvalidStaticData(
                        "battle unit draft contains multiple equipped weapons".to_string(),
                    ));
                }
                weapon_profile = Some(next_profile);
            }
            let weapon_profile = weapon_profile.ok_or(GameError::WeaponRequired)?;
            profile.apply_weapon_profile(weapon_profile)?;
        }

        if let Some(source) = &profile.fragment_source {
            let metadata = game_data
                .skill_fragment_data
                .get_by_id(&source.definition_id)
                .ok_or(GameError::InvalidAction)?;
            let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
                forms,
                ..
            } = &metadata.effect
            else {
                return Err(GameError::InvalidAction);
            };
            if source.copy_uuid.is_nil()
                || forms
                    .get(&source.form_id)
                    .is_none_or(|form| form.skill_id != source.skill_id)
                || profile.skill_id.as_ref() != Some(&source.skill_id)
            {
                return Err(GameError::InvalidAction);
            }
            let report = metadata
                .form_compatibility_report(&source.form_id, &profile)
                .ok_or(GameError::InvalidAction)?;
            if !report.is_compatible {
                return Err(GameError::SkillFragmentIncompatible {
                    fragment_id: source.definition_id.clone(),
                    failure_codes: report.failure_codes,
                });
            }
        }

        if profile.basic_attack_speed_percent == 0 || profile.stats.attack_interval_ms == 0 {
            return Err(GameError::InvalidStaticData(
                "battle profile cadence must be positive".into(),
            ));
        }
        Ok(profile)
    }

    pub fn effective_stats(&self, game_data: &GameDataBase) -> Result<UnitStats, GameError> {
        crate::game::battle::stat_pipeline::effective_stats_for_draft(self, game_data)
    }
}

fn effective_target_traits(
    mobility_kind: MobilityKind,
    authored_traits: &[UnitTargetTrait],
) -> Vec<UnitTargetTrait> {
    let mut traits = authored_traits.to_vec();
    if mobility_kind.is_airborne() && !traits.contains(&UnitTargetTrait::Airborne) {
        traits.push(UnitTargetTrait::Airborne);
    }
    traits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::ability::DeliveryDef;
    use crate::game::data::{
        abnormality_data::AbnormalityMetadata,
        equipment_data::{
            EquipmentEnhancementRecipeMetadata, EquipmentMaterialCost, EquipmentMaterialMetadata,
            EquipmentMaterialType, EquipmentMetadata, EquipmentType,
        },
        GameDataBuilder,
    };
    use crate::game::enums::RiskLevel;
    use crate::game::growth::GrowthId;
    use crate::game::stats::{
        Effect, StatId, StatModifier, StatModifierKind, TriggerEffectTarget, TriggerType,
        TriggeredEffect,
    };
    use std::collections::HashMap;

    #[test]
    fn employee_default_profile_is_ground_blockable_but_unarmed_capacity_is_zero() {
        let profile = UnitCombatProfile::employee_default();

        assert_eq!(profile.deployment_affinity, DeploymentAffinity::GroundOnly);
        assert_eq!(profile.block_capacity, 0);
        assert!((profile.block_radius_units - 0.75).abs() <= f32::EPSILON);
        assert!(profile.blockable);
    }

    #[test]
    fn employee_attack_speed_scales_default_and_weapon_intervals_with_ceiling() {
        let mut profile = UnitCombatProfile::for_employee(120, 15, 3, 7, 125);
        assert_eq!(profile.stats.max_health, 120);
        assert_eq!(profile.stats.attack, 15);
        assert_eq!(profile.stats.defense, 3);
        assert_eq!(profile.stats.magic_resist, 7);
        assert_eq!(profile.basic_attack.interval_ms, 1200);
        assert_eq!(profile.stats.attack_interval_ms, 1200);

        let mut weapon = WeaponCombatProfile::default();
        weapon.interval_ms = 1001;
        profile.apply_weapon_profile(&weapon).unwrap();

        assert_eq!(profile.basic_attack.interval_ms, 801);
        assert_eq!(profile.stats.attack_interval_ms, 801);
    }

    #[test]
    fn equipped_weapon_profile_is_runtime_basic_attack_delivery_authority() {
        let mut profile = UnitCombatProfile::employee_default();
        assert!(matches!(
            profile.basic_attack.delivery,
            DeliveryDef::Instant
        ));

        let mut weapon = WeaponCombatProfile::default();
        weapon.delivery = DeliveryDef::Projectile {
            speed_units_per_ms: 2_500,
            hit_policy: Default::default(),
            allow_targetless_cast: false,
            max_range_tiles: None,
            max_lifetime_ms: None,
            max_kills: None,
            max_pierces: None,
            collision: Default::default(),
        };
        profile.apply_weapon_profile(&weapon).unwrap();

        assert!(matches!(
            profile.basic_attack.delivery,
            DeliveryDef::Projectile {
                speed_units_per_ms: 2_500,
                ..
            }
        ));
        assert!(matches!(
            profile
                .weapon_profile
                .as_ref()
                .map(|weapon| &weapon.delivery),
            Some(DeliveryDef::Projectile {
                speed_units_per_ms: 2_500,
                ..
            })
        ));
    }

    #[test]
    fn abnormality_profile_defaults_to_blockable_enemy_without_deployment_capacity() {
        let abno_uuid = Uuid::from_u128(0xAB);
        let abno = AbnormalityMetadata {
            id: "abno".to_string(),
            uuid: abno_uuid,
            name: "Abno".to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 0,
            max_health: 100,
            attack: 10,
            defense: 5,
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        };
        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abno])
            .build();

        let profile =
            BattleUnitDraft::combat_profile_from_abnormality(abno_uuid, &game_data).unwrap();

        assert_eq!(profile.deployment_affinity, DeploymentAffinity::GroundOnly);
        assert_eq!(profile.block_capacity, 0);
        assert_eq!(profile.block_radius_units, 0.0);
        assert_eq!(profile.mobility_kind, MobilityKind::Ground);
        assert!(profile.blockable);
    }

    #[test]
    fn abnormality_airborne_profile_derives_unblockable_display_trait() {
        let abno_uuid = Uuid::from_u128(0xAC);
        let abno = AbnormalityMetadata {
            id: "drone".to_string(),
            uuid: abno_uuid,
            name: "Drone".to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 0,
            max_health: 100,
            attack: 10,
            defense: 5,
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
            mobility_kind: MobilityKind::Airborne,
            target_traits: Vec::new(),
        };
        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abno.clone()])
            .build();

        let profile =
            BattleUnitDraft::combat_profile_from_abnormality(abno_uuid, &game_data).unwrap();

        assert_eq!(profile.mobility_kind, MobilityKind::Airborne);
        assert!(!profile.blockable);
        assert!(profile.target_traits.contains(&UnitTargetTrait::Airborne));
    }

    #[test]
    fn effective_stats_applies_growth_and_permanent_item_effects() {
        let abno_uuid = Uuid::from_u128(1);
        let item_uuid = Uuid::from_u128(2);

        let abno = AbnormalityMetadata {
            id: "abno".to_string(),
            uuid: abno_uuid,
            name: "Abno".to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 0,
            max_health: 100,
            attack: 10,
            defense: 5,
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        };

        let mut item_triggers: HashMap<TriggerType, Vec<TriggeredEffect>> = HashMap::new();
        item_triggers.insert(
            TriggerType::Permanent,
            vec![TriggeredEffect::legacy(Effect::Modifier(StatModifier {
                stat: StatId::Attack,
                kind: StatModifierKind::Flat,
                value: 7,
            }))],
        );
        let item = EquipmentMetadata {
            content_family: None,
            id: "item".to_string(),
            uuid: item_uuid,
            name: "Item".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: item_triggers,
            ability_activations: vec![],
            weapon_profile: Some(Default::default()),
            event_check_modifiers: vec![],
            event_choice_keys: vec![],
        };

        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abno])
            .with_equipment(vec![item])
            .build();

        let mut growth = GrowthStack::new();
        growth.add(GrowthId::KillStack, 2);

        let unit = BattleUnitDraft {
            owned_uuid: Uuid::from_u128(10),
            source: BattleUnitSource::Abnormality {
                base_uuid: abno_uuid,
            },
            threat_class: BattleUnitThreatClass::Elite,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: growth,
            equipment_layout: None,
            equipped_items: vec![BattleEquipmentBinding {
            weapon_stage: Default::default(),
                base_uuid: item_uuid,
                persistent_copy_uuid: None,
                slot: None,
            }],
            equipped_item_enhancements: vec![],
            employee_start_condition: None,
        };

        let stats = unit.effective_stats(&game_data).unwrap();
        assert_eq!(stats.max_health, 100);
        assert_eq!(stats.current_health, 100);
        assert_eq!(stats.attack, 10 + 2 + 7);
        assert_eq!(stats.defense, 5);
        assert!(stats.attack_interval_ms > 0);
    }

    #[test]
    fn effective_stats_applies_owned_equipment_enhancement_modifiers() {
        let abno_uuid = Uuid::from_u128(1);
        let item_uuid = Uuid::from_u128(2);

        let abno = AbnormalityMetadata {
            id: "abno".to_string(),
            uuid: abno_uuid,
            name: "Abno".to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 0,
            max_health: 100,
            attack: 10,
            defense: 5,
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        };
        let item = EquipmentMetadata {
            content_family: None,
            id: "enhanced_item".to_string(),
            uuid: item_uuid,
            name: "Enhanced Item".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: HashMap::new(),
            ability_activations: vec![],
            weapon_profile: Some(Default::default()),
            event_check_modifiers: vec![],
            event_choice_keys: vec![],
        };
        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abno])
            .with_equipment_data(std::sync::Arc::new(
                crate::game::data::equipment_data::EquipmentDatabase::with_all(
                    vec![item],
                    vec![EquipmentMaterialMetadata {
                        id: "test_material".to_string(),
                        uuid: Uuid::from_u128(3),
                        name: "Test Material".to_string(),
                        description: String::new(),
                        material_type: EquipmentMaterialType::EquipmentDust,
                        rarity: RiskLevel::ZAYIN,
                        equipment_type: Some(EquipmentType::Weapon),
                    }],
                    vec![],
                    vec![EquipmentEnhancementRecipeMetadata {
                        equipment_id: "enhanced_item".to_string(),
                        max_level: 3,
                        costs_per_level: vec![EquipmentMaterialCost {
                            material_id: "test_material".to_string(),
                            amount: 1,
                        }],
                        modifiers_per_level: vec![StatModifier {
                            stat: StatId::Attack,
                            kind: StatModifierKind::Flat,
                            value: 2,
                        }],
                    }],
                ),
            ))
            .build();

        let unit = BattleUnitDraft {
            owned_uuid: Uuid::from_u128(10),
            source: BattleUnitSource::Abnormality {
                base_uuid: abno_uuid,
            },
            threat_class: BattleUnitThreatClass::Elite,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: GrowthStack::new(),
            equipment_layout: None,
            equipped_items: vec![BattleEquipmentBinding {
            weapon_stage: Default::default(),
                base_uuid: item_uuid,
                persistent_copy_uuid: Some(Uuid::from_u128(0xc001)),
                slot: None,
            }],
            equipped_item_enhancements: vec![BattleEquipmentEnhancement {
                base_uuid: item_uuid,
                persistent_copy_uuid: Some(Uuid::from_u128(0xc001)),
                enhancement_level: 2,
            }],
            employee_start_condition: None,
        };

        let stats = unit.effective_stats(&game_data).unwrap();
        assert_eq!(stats.attack, 14);
    }

    #[test]
    fn effective_stats_errors_when_abnormality_missing() {
        let game_data = GameDataBuilder::empty().build();

        let unit = BattleUnitDraft {
            owned_uuid: Uuid::from_u128(10),
            source: BattleUnitSource::Abnormality {
                base_uuid: Uuid::from_u128(1),
            },
            threat_class: BattleUnitThreatClass::Elite,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: GrowthStack::new(),
            equipment_layout: None,
            equipped_items: vec![],
            equipped_item_enhancements: vec![],
            employee_start_condition: None,
        };

        let err = unit.effective_stats(&game_data).unwrap_err();
        assert!(matches!(
            err,
            GameError::MissingResource("AbnormalityMetadata")
        ));
    }

    #[test]
    fn programmatic_data_publication_rejects_invalid_permanent_targeting() {
        let abno_uuid = Uuid::from_u128(1);
        let item_uuid = Uuid::from_u128(2);

        let abno = AbnormalityMetadata {
            id: "abno".to_string(),
            uuid: abno_uuid,
            name: "Abno".to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 0,
            max_health: 100,
            attack: 10,
            defense: 5,
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        };

        let mut item_triggers: HashMap<TriggerType, Vec<TriggeredEffect>> = HashMap::new();
        item_triggers.insert(
            TriggerType::Permanent,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::CounterpartUnit,
                Effect::Modifier(StatModifier {
                    stat: StatId::Defense,
                    kind: StatModifierKind::Flat,
                    value: 3,
                }),
            )],
        );
        let item = EquipmentMetadata {
            content_family: None,
            id: "item".to_string(),
            uuid: item_uuid,
            name: "Item".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: item_triggers,
            ability_activations: vec![],
            weapon_profile: Some(Default::default()),
            event_check_modifiers: vec![],
            event_choice_keys: vec![],
        };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            GameDataBuilder::empty()
                .with_abnormalities(vec![abno])
                .with_equipment(vec![item])
                .build()
        }));
        let payload = match result {
            Ok(_) => panic!("programmatic publication must reject invalid Permanent targeting"),
            Err(payload) => payload,
        };
        let message = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied())
            .expect("validation panic should contain a string message");
        assert!(message.contains("invalid Equipment effect or activation contract"));
        assert!(message.contains("items[0].triggered_effects.Permanent[0]"));
        assert!(message.contains(
            "unsupported equipment trigger effect: trigger=Permanent, target=CounterpartUnit, effect=Modifier"
        ));
    }
}
