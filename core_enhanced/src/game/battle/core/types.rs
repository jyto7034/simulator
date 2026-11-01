use std::{collections::BTreeMap, sync::Arc};

use uuid::Uuid;

use crate::{
    game::resources::Position,
    game::{
        ability::{
            SkillAreaAnchorSource, SkillAreaTickPolicy, SkillAreaTracking, SkillHitTargetFilter,
            SkillId, SkillProjectileCollisionDef, SkillTileAreaOrigin,
        },
        battle::{
            buffs::BuffId,
            cooldown::CooldownSource,
            core::movement::{
                types::{UnitBody, WorldVec2},
                ActionState,
            },
            damage::{DamageModifiers, DamageSourceSnapshot, DamageType},
            event_log::{BattleEventCause, SkillCastTarget},
            ids::UnitInstanceId,
            tile_range::{FacingDirection, TileRangePattern},
            types::{
                BattleEquipmentBinding, BattleUnitRole, BattleUnitSourceIdentity, EngagementStance,
                MobilityKind, TacticalTileKind, UnitExclusiveActivity, UnitSnapshot,
            },
        },
        data::{abnormality_data::BasicAttackDef, equipment_data::WeaponCombatProfile},
        enums::Side,
        resources::item_slot::SlotLayoutKind,
        stats::UnitStats,
    },
};

use crate::game::battle::types::UnitTargetTrait;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileGuidance {
    Homing,
    Fixed,
}

pub type SkillDeliveryId = Uuid;
pub type AreaInstanceId = Uuid;

#[derive(Debug, Clone)]
pub(super) struct UnitTacticalState {
    pub(super) stance: EngagementStance,
    pub(super) tile_kind: TacticalTileKind,
    pub(super) weapon_profile: WeaponCombatProfile,
    pub(super) base_block_capacity: u32,
    pub(super) stance_cooldown_until_ms: u64,
    pub(super) relocation_cooldown_until_ms: u64,
    pub(super) forced_defense_until_ms: Option<u64>,
    pub(super) forced_defense_epoch: u64,
    pub(super) rest_epoch: u64,
    pub(super) next_rest_tick_ms: Option<u64>,
    pub(super) activity: UnitExclusiveActivity,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicSpatialProjectile {
    pub actual_hp_loss: u64,
    pub range_tiles: u32,
    pub lifetime_ms: u64,
    pub collision: crate::game::ability::SkillProjectileCollisionDef,
    pub max_kills: Option<u32>,
    pub max_pierces: Option<u32>,
    pub kills: u32,
    pub hit_unit_ids: Vec<UnitInstanceId>,
}
crate::authoring::clone_shape::struct_shape!(BasicSpatialProjectile {
    actual_hp_loss,
    range_tiles,
    lifetime_ms,
    collision,
    max_kills,
    max_pierces,
    kills,
    hit_unit_ids
});
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectileRecord {
    pub area: Option<std::sync::Arc<super::basic_projectile_area::BasicProjectileArea>>,
    pub batch_id: Option<Uuid>,
    pub spatial: Option<BasicSpatialProjectile>,
    pub fired_at_ms: u64,
    pub last_reevaluation_ms: u64,
    pub attacker_instance_id: UnitInstanceId,
    pub attacker_owner_at_launch: Side,
    pub air_capable_at_launch: bool,
    pub target_instance_id: UnitInstanceId,
    pub start: WorldVec2,
    pub current_position: WorldVec2,
    pub aim: WorldVec2,
    pub speed_units_per_ms: u32,
    pub guidance: ProjectileGuidance,
    pub damage_type: DamageType,
    pub source_snapshot: DamageSourceSnapshot,
    pub max_travel_ms: u64,
    pub cause: BattleEventCause,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveProjectileRuntime {
    pub delivery_id: SkillDeliveryId,
    pub cast_seq: u64,
    pub step_index: usize,
    pub skill_id: SkillId,
    pub step_id: String,
    pub caster_instance_id: UnitInstanceId,
    pub caster_owner: Side,
    pub spawned_at_ms: u64,
    pub start: WorldVec2,
    pub current_position: WorldVec2,
    pub aim: WorldVec2,
    pub speed_units_per_ms: u32,
    pub guidance: ProjectileGuidance,
    pub target_unit_id: Option<UnitInstanceId>,
    pub source_snapshot: DamageSourceSnapshot,
    pub collision: SkillProjectileCollisionDef,
    pub hit_unit_ids: Vec<UnitInstanceId>,
    pub killed_unit_count: u32,
    pub max_kills: Option<u32>,
    pub impact_vfx_id: Option<String>,
    pub last_reevaluation_ms: Option<u64>,
    pub next_reevaluation_ms: u64,
    pub max_travel_ms: u64,
    pub cause: BattleEventCause,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkillImpactContext {
    pub delivery_id: SkillDeliveryId,
    pub impact_time_ms: u64,
    pub impact_position: WorldVec2,
    pub direction_hint: Option<WorldVec2>,
    pub first_hit_unit_id: Option<UnitInstanceId>,
    pub hit_unit_ids: Vec<UnitInstanceId>,
    pub spawned_area_id: Option<AreaInstanceId>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct EffectHitHistory {
    pub hit_unit_ids: Vec<UnitInstanceId>,
    pub previous_tick_unit_ids: Vec<UnitInstanceId>,
}
crate::authoring::clone_shape::struct_shape!(EffectHitHistory {
    hit_unit_ids,
    previous_tick_unit_ids
});
#[derive(Debug, Clone, PartialEq)]
pub struct AreaRuntime {
    pub(in crate::game::battle::core) channel_started_at: Option<u64>,
    pub(crate) source_generation: Option<std::sync::Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub(in crate::game::battle::core) mode: crate::game::ability::SkillTileAreaMode,
    pub(in crate::game::battle::core) ground_status: Option<Box<super::skill_runtime::area::ground_status::GroundAreaState>>,
    pub emission: Option<crate::game::ability::SkillAreaEmissionDef>,
    pub area_id: AreaInstanceId,
    pub cast_seq: u64,
    pub step_index: usize,
    pub skill_id: SkillId,
    pub step_id: String,
    pub caster_instance_id: UnitInstanceId,
    pub caster_owner: Side,
    pub source_snapshot: DamageSourceSnapshot,
    pub anchor: SkillAreaAnchorSource,
    pub tile_origin: SkillTileAreaOrigin,
    pub tracking: SkillAreaTracking,
    pub origin: WorldVec2,
    pub center: WorldVec2,
    pub direction_hint: WorldVec2,
    pub facing: FacingDirection,
    pub tile_range: TileRangePattern,
    pub tile_anchor: Position,
    pub hit_targets: SkillHitTargetFilter,
    pub include_caster: bool,
    pub tick_policy: SkillAreaTickPolicy,
    pub duration_ms: u32,
    pub spawned_at_ms: u64,
    pub expires_at_ms: u64,
    pub tick_interval_ms: Option<u32>,
    pub next_tick_ms: Option<u64>,
    pub step_target: Option<SkillCastTarget>,
    pub hit_unit_ids: Vec<UnitInstanceId>,
    pub previous_tick_unit_ids: Vec<UnitInstanceId>,
    pub(crate) effective_history: Option<Box<std::collections::BTreeMap<usize, EffectHitHistory>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActionLocks {
    pub movement_until_ms: u64,
    pub basic_attack_until_ms: u64,
    pub resonance_gain_until_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnitControlState {
    pub hard_cc_release_time_ms: Option<u64>,
    pub silenced: bool,
}

impl UnitControlState {
    pub fn hard_cc_active(self) -> bool {
        self.hard_cc_release_time_ms.is_some()
    }
}

impl ActionLocks {
    pub fn can_move(&self, now_ms: u64) -> bool {
        now_ms >= self.movement_until_ms
    }

    pub fn can_basic_attack(&self, now_ms: u64) -> bool {
        now_ms >= self.basic_attack_until_ms
    }

    pub fn can_gain_resonance(&self, now_ms: u64) -> bool {
        now_ms >= self.resonance_gain_until_ms
    }

    pub fn lock_movement_until(&mut self, until_ms: u64) {
        self.movement_until_ms = self.movement_until_ms.max(until_ms);
    }

    pub fn lock_basic_attack_until(&mut self, until_ms: u64) {
        self.basic_attack_until_ms = self.basic_attack_until_ms.max(until_ms);
    }

    pub fn lock_resonance_gain_until(&mut self, until_ms: u64) {
        self.resonance_gain_until_ms = self.resonance_gain_until_ms.max(until_ms);
    }
}

/// 전투 중 사용되는 장비 런타임 표현
#[derive(Debug, Clone)]
pub(super) struct RuntimeItem {
    pub(super) weapon_stage: crate::game::ability::weapon_marks::WeaponStage,
    pub(super) instance_id: Uuid,
    pub(super) owner: Side,
    pub(super) owner_unit_instance: UnitInstanceId,
    pub(super) base_uuid: Uuid,
    pub(super) persistent_copy_uuid: Option<Uuid>,
}

impl RuntimeItem {
    pub(super) fn source_snapshot(
        &self,
    ) -> crate::game::battle::runtime_facts::EquipmentSourceSnapshot {
        crate::game::battle::runtime_facts::EquipmentSourceSnapshot {
            definition_uuid: self.base_uuid,
            runtime_item_instance_id: self.instance_id,
            persistent_copy_uuid: self.persistent_copy_uuid,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct PendingSkillCast {
    pub(super) start_gate_pending: bool,
    pub(super) reset_resonance: bool,
    pub(super) action: Option<crate::game::battle::runtime_facts::ActionHandle>,
    pub(super) skill_id: SkillId,
    pub(super) cast_target: Option<SkillCastTarget>,
    pub(super) accepted_target_tile: Option<Position>,
    pub(super) accepted_facing: Option<crate::game::battle::tile_range::FacingDirection>,
    pub(super) start_seq: u64,
}

#[derive(Debug, Clone)]
pub(super) struct ResonanceUse {
    pub action: crate::game::battle::runtime_facts::ActionHandle,
    pub skill_id: SkillId,
    pub consumed: u32,
    pub full_payment: bool,
    pub unreleased_attacks: u32,
    pub control_cancelled: bool,
}
crate::authoring::clone_shape::struct_shape!(ResonanceUse {
    action,
    skill_id,
    consumed,
    full_payment,
    unreleased_attacks,
    control_cancelled
});

#[derive(Debug, Clone, Default)]
pub(super) struct SkillStepResult {
    pub(super) actual_hp_loss: u64,
    pub(super) resolved_target_count: usize,
    pub(super) actual_damage_target_count: usize,
    pub(super) effective_hit_count: usize,
    pub(super) killed_target_count: usize,
    pub(super) applied_effect_count: usize,
    pub(super) scheduled_attack_count: usize,
}

impl SkillStepResult {
    pub(super) fn merge(&mut self, other: &Self) {
        self.actual_hp_loss = self.actual_hp_loss.saturating_add(other.actual_hp_loss);
        self.resolved_target_count = self
            .resolved_target_count
            .saturating_add(other.resolved_target_count);
        self.actual_damage_target_count = self
            .actual_damage_target_count
            .saturating_add(other.actual_damage_target_count);
        self.effective_hit_count = self.effective_hit_count.saturating_add(other.effective_hit_count);
        self.killed_target_count = self
            .killed_target_count
            .saturating_add(other.killed_target_count);
        self.applied_effect_count = self
            .applied_effect_count
            .saturating_add(other.applied_effect_count);
        self.scheduled_attack_count = self
            .scheduled_attack_count
            .saturating_add(other.scheduled_attack_count);
    }

    pub(super) fn dealt_damage(&self) -> bool {
        self.actual_damage_target_count > 0
    }
}

#[derive(Debug, Clone)]
pub(super) struct SkillStepProgress {
    pub(super) use_owned: bool,
    pub(super) result: SkillStepResult,
    pub(super) started: bool,
    pub(super) pending_delivery_count: usize,
}

impl SkillStepProgress {
    pub(super) fn is_terminal(&self) -> bool {
        self.started && self.pending_delivery_count == 0
    }
}

#[derive(Debug, Clone)]
pub(super) struct DeferredSkillStep {
    pub(super) caster_instance_id: UnitInstanceId,
    pub(super) skill_id: SkillId,
    pub(super) step_id: String,
    pub(super) cast_target: Option<SkillCastTarget>,
    pub(super) cause: BattleEventCause,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct CommandExecutionSummary {
    /// A valid direct contact, including zero damage; excludes deflection and invulnerability.
    pub(super) accepted_hit_count: usize,
    pub(super) actual_hp_loss: u64,
    pub(super) actual_damage_target_count: usize,
    pub(super) effective_hit_count: usize,
    pub(super) killed_target_count: usize,
}

pub(super) type SkillCommand = (
    crate::game::battle::damage::BattleCommand,
    Option<std::sync::Arc<crate::game::ability::authoring::EffectSource>>,
);

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkillStepDebugResult {
    pub resolved_target_count: usize,
    pub actual_damage_target_count: usize,
    pub applied_effect_count: usize,
    pub scheduled_attack_count: usize,
}

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
impl From<&SkillStepResult> for SkillStepDebugResult {
    fn from(value: &SkillStepResult) -> Self {
        Self {
            resolved_target_count: value.resolved_target_count,
            actual_damage_target_count: value.actual_damage_target_count,
            applied_effect_count: value.applied_effect_count,
            scheduled_attack_count: value.scheduled_attack_count,
        }
    }
}

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillStepDebugProgress {
    pub result: SkillStepDebugResult,
    pub started: bool,
    pub pending_delivery_count: usize,
    pub terminal: bool,
}

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
impl From<&SkillStepProgress> for SkillStepDebugProgress {
    fn from(value: &SkillStepProgress) -> Self {
        Self {
            result: SkillStepDebugResult::from(&value.result),
            started: value.started,
            pending_delivery_count: value.pending_delivery_count,
            terminal: value.is_terminal(),
        }
    }
}

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveSkillCastDebugState {
    pub total_steps: usize,
    pub step_progress: BTreeMap<usize, SkillStepDebugProgress>,
    pub deferred_step_indices: Vec<usize>,
    pub has_impact_context: bool,
    pub active_area_count: usize,
}

#[derive(Debug, Clone)]
pub(super) struct ActiveSkillCast {
    pub(super) control_cancelled: bool,
    pub(super) flow: Option<super::skill_runtime::flow::FlowState>,
    pub(super) fact_source: crate::game::battle::runtime_facts::RuntimeFactSource,
    pub(super) execution_source:
        Option<std::sync::Arc<crate::game::battle::skill_source::SkillCastExecutionSource>>,
    #[allow(dead_code)]
    pub(super) caster_instance_id: UnitInstanceId,
    pub(super) ability_event_seq: u64,
    pub(super) skill_id: SkillId,
    pub(super) caster_owner: Side,
    pub(super) anchor_position: Position,
    pub(super) accepted_facing: Option<crate::game::battle::tile_range::FacingDirection>,
    pub(super) cast_target_anchor_position: Option<Position>,
    pub(super) cast_target_anchor_world_position: Option<WorldVec2>,
    pub(super) allow_dead_caster: bool,
    pub(super) total_steps: usize,
    pub(super) step_progress: BTreeMap<usize, SkillStepProgress>,
    pub(super) deferred_steps: BTreeMap<usize, DeferredSkillStep>,
    pub(super) impact_contexts_by_step: BTreeMap<usize, SkillImpactContext>,
    // Latest impact copy retained for storage accounting and debug projection.
    // Prior-step anchors are selected from impact_contexts_by_step.
    #[allow(dead_code)]
    pub(super) last_impact_context: Option<SkillImpactContext>,
    // Registered persistent areas keep the cast alive until their expiry cleanup.
    #[allow(dead_code)]
    pub(super) active_area_ids: Vec<AreaInstanceId>,
}

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
impl From<&ActiveSkillCast> for ActiveSkillCastDebugState {
    fn from(value: &ActiveSkillCast) -> Self {
        Self {
            total_steps: value.total_steps,
            step_progress: value
                .step_progress
                .iter()
                .map(|(index, progress)| (*index, SkillStepDebugProgress::from(progress)))
                .collect(),
            deferred_step_indices: value.deferred_steps.keys().copied().collect(),
            has_impact_context: !value.impact_contexts_by_step.is_empty()
                || value.last_impact_context.is_some(),
            active_area_count: value.active_area_ids.len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct AbilityProcKey {
    pub(super) source: CooldownSource,
    pub(super) ability_id: SkillId,
    pub(super) binding_index: usize,
}

#[derive(Debug, Clone, Default)]
pub(super) struct AbilityProcState {
    pub(super) trigger_count: u32,
    pub(super) next_ready_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeUnitLifecycle {
    Active,
    Withdrawn,
    Despawned,
    Dead,
    PermanentlyDead,
}

/// Immutable equipment layout projected at the unit's battle entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeEquipmentSlot {
    pub(crate) binding: BattleEquipmentBinding,
    pub(crate) runtime_item_instance_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleEquipmentSlots {
    pub(crate) layout: SlotLayoutKind,
    pub(crate) slots: [Option<RuntimeEquipmentSlot>; 3],
}

impl BattleEquipmentSlots {
    pub(crate) const fn retained_extent() -> usize {
        std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>()
    }
}

impl RuntimeUnitLifecycle {
    pub fn is_active(self) -> bool {
        matches!(self, RuntimeUnitLifecycle::Active)
    }

    pub fn is_dead(self) -> bool {
        matches!(self, RuntimeUnitLifecycle::Dead | RuntimeUnitLifecycle::PermanentlyDead)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeUnit {
    pub employee_trauma: Option<u32>,
    pub corroded: bool,
    pub instance_id: UnitInstanceId,
    pub lifecycle: RuntimeUnitLifecycle,
    pub spawn_order: u64,
    pub source_owned_uuid: Uuid,
    pub owner: Side,
    pub role: BattleUnitRole,
    pub threat_class: crate::game::battle::types::BattleUnitThreatClass,
    pub base_uuid: Uuid,
    pub source_identity: BattleUnitSourceIdentity,
    pub(crate) equipment_slots: Option<Arc<BattleEquipmentSlots>>,
    pub stats: UnitStats,
    pub basic_attack_speed_percent: u32,
    pub incoming_damage_modifiers: DamageModifiers,
    pub basic_attack: BasicAttackDef,
    pub fragment_source: Option<Arc<crate::game::skill_fragment::FragmentSourceSnapshot>>,
    pub skill_id: Option<SkillId>,
    pub skill_activation_mode: crate::game::ability::SkillActivationMode,
    pub body: UnitBody,
    pub tactical_anchor: Option<WorldVec2>,
    pub enemy_movement_plan: Option<crate::game::battle::scenario::EnemyMovementPlan>,
    pub block_capacity: u32,
    pub block_radius_units: f32,
    pub blockable: bool,
    pub mobility_kind: MobilityKind,
    pub target_traits: Vec<UnitTargetTrait>,
    pub facing_direction: Option<FacingDirection>,
    pub move_epoch: u32,
    pub action_state: ActionState,
    pub action_locks: ActionLocks,
    pub current_target: Option<UnitInstanceId>,
    pub next_basic_attack_ms: u64,
    pub pending_basic_attack: bool,
    pub(super) basic_attack_epoch: u64,
    pub(super) basic_attack_windup_until_ms: Option<u64>,
    pub ranged_reposition_until_ms: u64,
    pub resonance_current: u32,
    pub resonance_max: u32,
    pub resonance_lock_ms: u64,
    pub(super) resonance_use: Option<ResonanceUse>,
    pub(super) next_resonance_pulse_ms: Option<u64>,
    pub(super) equipment_resonance_gain: Option<super::equipment_resonance_gain::ResonanceGainState>,
    pub next_action_time: u64,
    pub pending_cast: bool,
    pub pending_cast_cause: Option<BattleEventCause>,
    pub(super) pending_skill_cast: Option<PendingSkillCast>,
}

impl RuntimeUnit {
    pub(super) fn dynamic_profile_retained_extent(&self) -> Option<usize> {
        let (BattleUnitSourceIdentity::IndependentObject { definition_id, .. }
            | BattleUnitSourceIdentity::SummonedUnit { definition_id, .. }) = &self.source_identity
        else {
            return Some(0);
        };
        let path = match &self.enemy_movement_plan {
            Some(crate::game::battle::scenario::EnemyMovementPlan::PathAlongCells { cells }) => {
                cells
                    .capacity()
                    .checked_mul(std::mem::size_of::<crate::game::resources::Position>())?
            }
            _ => 0,
        };
        let bytes = std::mem::size_of::<Self>()
            .checked_add(super::sustained::basic_extent(&self.basic_attack)?)?
            .checked_add(definition_id.capacity())?
            .checked_add(self.skill_id.as_ref().map_or(0, |id| id.as_str().len()))?
            .checked_add(path)?;
        let bytes = bytes.checked_add(
            self.target_traits
                .capacity()
                .checked_mul(std::mem::size_of::<UnitTargetTrait>())?,
        )?;
        Some(bytes)
    }

    pub(crate) const fn fact_slot_extent() -> usize {
        96 + std::mem::size_of::<Option<Arc<crate::game::skill_fragment::FragmentSourceSnapshot>>>()
            + std::mem::size_of::<Option<Arc<BattleEquipmentSlots>>>()
            + std::mem::size_of::<u32>()
            + std::mem::size_of::<Option<ResonanceUse>>()
            + std::mem::size_of::<Option<super::equipment_resonance_gain::ResonanceGainState>>()
            + std::mem::size_of::<Option<u64>>()
    }

    /// Only used for a timestamp scratch map with the same proven native
    /// bucket lineage. Value replacement reuses owned range rows and strings.
    pub(super) fn reuse_timestamp_value(&mut self, source: &Self) {
        fn reuse_basic_attack(target: &mut BasicAttackDef, source: &BasicAttackDef) {
            let BasicAttackDef {
                action,
                range_policy,
                defense_tile_range,
                damage_type,
                targeting_profile,
                air_capable,
                range_role,
                interval_ms,
                windup_ms,
                ranged_reposition_ms,
                delivery,
            } = source;
            target.action.clone_from(action);
            target.range_policy.clone_from(range_policy);
            match (&mut target.defense_tile_range, defense_tile_range) {
                (Some(target), Some(source)) => {
                    let crate::game::battle::tile_range::TileRangePattern {
                        include_anchor_tile,
                        rows,
                    } = source;
                    target.include_anchor_tile = *include_anchor_tile;
                    target.rows.clone_from(rows);
                }
                (target, source) => target.clone_from(source),
            }
            target.damage_type.clone_from(damage_type);
            target.targeting_profile.clone_from(targeting_profile);
            target.air_capable.clone_from(air_capable);
            target.range_role.clone_from(range_role);
            target.interval_ms.clone_from(interval_ms);
            target.windup_ms.clone_from(windup_ms);
            target.ranged_reposition_ms.clone_from(ranged_reposition_ms);
            target.delivery.clone_from(delivery);
        }
        let Self {
            instance_id,
            lifecycle,
            spawn_order,
            employee_trauma,
            corroded,
            source_owned_uuid,
            owner,
            role,
            threat_class,
            base_uuid,
            source_identity,
            equipment_slots,
            fragment_source,
            stats,
            basic_attack_speed_percent,
            incoming_damage_modifiers,
            basic_attack,
            skill_id,
            skill_activation_mode,
            body,
            tactical_anchor,
            enemy_movement_plan,
            block_capacity,
            block_radius_units,
            blockable,
            mobility_kind,
            target_traits,
            facing_direction,
            move_epoch,
            action_state,
            action_locks,
            current_target,
            next_basic_attack_ms,
            pending_basic_attack,
            basic_attack_epoch,
            basic_attack_windup_until_ms,
            ranged_reposition_until_ms,
            resonance_current,
            resonance_max,
            resonance_lock_ms,
            resonance_use,
            next_resonance_pulse_ms,
            equipment_resonance_gain,
            next_action_time,
            pending_cast,
            pending_cast_cause,
            pending_skill_cast,
        } = source;
        self.instance_id.clone_from(instance_id);
        self.lifecycle.clone_from(lifecycle);
        self.spawn_order.clone_from(spawn_order);
        self.employee_trauma = *employee_trauma;
        self.corroded = *corroded;
        self.source_owned_uuid.clone_from(source_owned_uuid);
        self.owner.clone_from(owner);
        self.role.clone_from(role);
        self.threat_class.clone_from(threat_class);
        self.base_uuid.clone_from(base_uuid);
        if &self.source_identity != source_identity {
            self.source_identity.clone_from(source_identity);
        }
        self.equipment_slots.clone_from(equipment_slots);
        self.fragment_source.clone_from(fragment_source);
        self.stats.clone_from(stats);
        self.basic_attack_speed_percent = *basic_attack_speed_percent;
        self.incoming_damage_modifiers
            .clone_from(incoming_damage_modifiers);
        reuse_basic_attack(&mut self.basic_attack, basic_attack);
        if &self.skill_id != skill_id {
            self.skill_id.clone_from(skill_id);
        }
        self.skill_activation_mode.clone_from(skill_activation_mode);
        self.body.clone_from(body);
        self.tactical_anchor.clone_from(tactical_anchor);
        self.enemy_movement_plan.clone_from(enemy_movement_plan);
        self.block_capacity.clone_from(block_capacity);
        self.block_radius_units.clone_from(block_radius_units);
        self.blockable.clone_from(blockable);
        self.mobility_kind.clone_from(mobility_kind);
        self.target_traits.clone_from(target_traits);
        self.facing_direction.clone_from(facing_direction);
        self.move_epoch.clone_from(move_epoch);
        self.action_state.clone_from(action_state);
        self.action_locks.clone_from(action_locks);
        self.current_target.clone_from(current_target);
        self.next_basic_attack_ms.clone_from(next_basic_attack_ms);
        self.pending_basic_attack.clone_from(pending_basic_attack);
        self.basic_attack_epoch.clone_from(basic_attack_epoch);
        self.basic_attack_windup_until_ms
            .clone_from(basic_attack_windup_until_ms);
        self.ranged_reposition_until_ms
            .clone_from(ranged_reposition_until_ms);
        self.resonance_current.clone_from(resonance_current);
        self.resonance_max.clone_from(resonance_max);
        self.resonance_lock_ms.clone_from(resonance_lock_ms);
        self.resonance_use.clone_from(resonance_use);
        self.equipment_resonance_gain.clone_from(equipment_resonance_gain);
        self.next_resonance_pulse_ms
            .clone_from(next_resonance_pulse_ms);
        self.next_action_time.clone_from(next_action_time);
        self.pending_cast.clone_from(pending_cast);
        self.pending_cast_cause.clone_from(pending_cast_cause);
        self.pending_skill_cast.clone_from(pending_skill_cast);
    }

    /// UnitSnapshot 생성
    pub(super) fn to_snapshot(&self, position: Position) -> UnitSnapshot {
        UnitSnapshot {
            id: self.instance_id,
            owner: self.owner,
            role: self.role,
            threat_class: self.threat_class,
            mobility_kind: self.mobility_kind,
            position,
            world_position: self.body.position,
            stats: self.stats,
        }
    }

    pub fn world_position(&self) -> WorldVec2 {
        self.body.position
    }

    pub fn movement_body_view(&self) -> UnitBody {
        self.body
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn set_world_position(&mut self, position: WorldVec2) {
        self.body.position = position;
    }

    pub fn is_dead(&self) -> bool {
        self.lifecycle.is_dead()
    }

    pub fn is_active(&self) -> bool {
        self.lifecycle.is_active()
    }

    pub fn is_combatant(&self) -> bool {
        self.role == BattleUnitRole::Combatant
    }

    pub fn is_airborne(&self) -> bool {
        self.mobility_kind.is_airborne()
    }

    pub fn can_basic_attack(&self) -> bool {
        self.is_active() && self.is_combatant()
    }

    pub fn can_move(&self) -> bool {
        self.is_active()
            && self.is_combatant()
            && self.stats.move_speed_units_per_ms > 0
            && (!matches!(
                self.source_identity,
                BattleUnitSourceIdentity::IndependentObject { .. }
            ) || self.enemy_movement_plan.is_some())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct BuffInstanceKey {
    pub(super) caster_instance_id: UnitInstanceId,
    pub(super) target_instance_id: UnitInstanceId,
    pub(super) buff_id: BuffId,
}

#[derive(Debug, Clone)]
pub(super) struct ActiveBuff {
    pub(super) latest_application_source: crate::game::battle::runtime_facts::RuntimeFactSource,
    pub(super) stacks: u8,
    pub(super) expires_at_ms: u64,
    pub(super) next_tick_ms: Option<u64>,
}

crate::authoring::clone_shape::copy_shape!(
    ProjectileGuidance,
    ActionLocks,
    UnitControlState,
    RuntimeUnitLifecycle,
    BuffInstanceKey
);
crate::authoring::clone_shape::struct_shape!(UnitTacticalState {
    stance,
    tile_kind,
    weapon_profile,
    base_block_capacity,
    stance_cooldown_until_ms,
    relocation_cooldown_until_ms,
    forced_defense_until_ms,
    forced_defense_epoch,
    rest_epoch,
    next_rest_tick_ms,
    activity,
});
crate::authoring::clone_shape::struct_shape!(ProjectileRecord {
    area,
    batch_id,
    spatial,
    fired_at_ms,
    last_reevaluation_ms,
    attacker_instance_id,
    attacker_owner_at_launch,
    air_capable_at_launch,
    target_instance_id,
    start,
    current_position,
    aim,
    speed_units_per_ms,
    guidance,
    damage_type,
    source_snapshot,
    max_travel_ms,
    cause,
});
crate::authoring::clone_shape::struct_shape!(ActiveProjectileRuntime {
    delivery_id,
    cast_seq,
    step_index,
    skill_id,
    step_id,
    caster_instance_id,
    caster_owner,
    spawned_at_ms,
    start,
    current_position,
    aim,
    speed_units_per_ms,
    guidance,
    target_unit_id,
    source_snapshot,
    collision,
    hit_unit_ids,
    killed_unit_count,
    max_kills,
    impact_vfx_id,
    last_reevaluation_ms,
    next_reevaluation_ms,
    max_travel_ms,
    cause,
});
crate::authoring::clone_shape::struct_shape!(SkillImpactContext {
    delivery_id,
    impact_time_ms,
    impact_position,
    direction_hint,
    first_hit_unit_id,
    hit_unit_ids,
    spawned_area_id,
});
crate::authoring::clone_shape::struct_shape!(AreaRuntime {
    channel_started_at,
    source_generation,
    mode,
    ground_status,
    emission,
    area_id,
    cast_seq,
    step_index,
    skill_id,
    step_id,
    caster_instance_id,
    caster_owner,
    source_snapshot,
    anchor,
    tile_origin,
    tracking,
    origin,
    center,
    direction_hint,
    facing,
    tile_range,
    tile_anchor,
    hit_targets,
    include_caster,
    tick_policy,
    duration_ms,
    spawned_at_ms,
    expires_at_ms,
    tick_interval_ms,
    next_tick_ms,
    step_target,
    hit_unit_ids,
    previous_tick_unit_ids,
    effective_history,
});
crate::authoring::clone_shape::struct_shape!(RuntimeItem {
    weapon_stage,
    instance_id,
    owner,
    owner_unit_instance,
    base_uuid,
    persistent_copy_uuid,
});
crate::authoring::clone_shape::struct_shape!(PendingSkillCast {
    start_gate_pending,
    reset_resonance,
    action,
    skill_id,
    cast_target,
    accepted_target_tile,
    accepted_facing,
    start_seq,
});
crate::authoring::clone_shape::struct_shape!(SkillStepResult {
    actual_hp_loss,
    resolved_target_count,
    actual_damage_target_count,
    effective_hit_count,
    killed_target_count,
    applied_effect_count,
    scheduled_attack_count,
});
crate::authoring::clone_shape::struct_shape!(SkillStepProgress {
    use_owned,
    result,
    started,
    pending_delivery_count,
});
crate::authoring::clone_shape::struct_shape!(DeferredSkillStep {
    caster_instance_id,
    skill_id,
    step_id,
    cast_target,
    cause,
});
crate::authoring::clone_shape::struct_shape!(ActiveSkillCast {
    control_cancelled,
    flow,
    fact_source,
    execution_source,
    caster_instance_id,
    ability_event_seq,
    skill_id,
    caster_owner,
    anchor_position,
    accepted_facing,
    cast_target_anchor_position,
    cast_target_anchor_world_position,
    allow_dead_caster,
    total_steps,
    step_progress,
    deferred_steps,
    impact_contexts_by_step,
    last_impact_context,
    active_area_ids,
});
crate::authoring::clone_shape::struct_shape!(AbilityProcKey {
    source,
    ability_id,
    binding_index
});
crate::authoring::clone_shape::struct_shape!(AbilityProcState {
    trigger_count,
    next_ready_ms
});
crate::authoring::clone_shape::struct_shape!(ActiveBuff {
    latest_application_source,
    stacks,
    expires_at_ms,
    next_tick_ms
});
crate::authoring::clone_shape::struct_shape!(RuntimeUnit {
    employee_trauma,
    corroded,
    instance_id,
    lifecycle,
    spawn_order,
    source_owned_uuid,
    owner,
    role,
    threat_class,
    base_uuid,
    source_identity,
    equipment_slots,
    fragment_source,
    stats,
    basic_attack_speed_percent,
    incoming_damage_modifiers,
    basic_attack,
    skill_id,
    skill_activation_mode,
    body,
    tactical_anchor,
    enemy_movement_plan,
    block_capacity,
    block_radius_units,
    blockable,
    mobility_kind,
    target_traits,
    facing_direction,
    move_epoch,
    action_state,
    action_locks,
    current_target,
    next_basic_attack_ms,
    pending_basic_attack,
    basic_attack_epoch,
    basic_attack_windup_until_ms,
    ranged_reposition_until_ms,
    resonance_current,
    resonance_max,
    resonance_lock_ms,
    resonance_use,
    next_resonance_pulse_ms,
    equipment_resonance_gain,
    next_action_time,
    pending_cast,
    pending_cast_cause,
    pending_skill_cast,
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_locks_lock_methods_take_max_and_gate_actions() {
        let mut locks = ActionLocks::default();
        assert!(locks.can_move(0));
        assert!(locks.can_basic_attack(0));
        assert!(locks.can_gain_resonance(0));

        locks.lock_movement_until(10);
        locks.lock_movement_until(5);
        assert!(!locks.can_move(9));
        assert!(locks.can_move(10));

        locks.lock_basic_attack_until(20);
        locks.lock_basic_attack_until(15);
        assert!(!locks.can_basic_attack(19));
        assert!(locks.can_basic_attack(20));

        locks.lock_resonance_gain_until(30);
        locks.lock_resonance_gain_until(25);
        assert!(!locks.can_gain_resonance(29));
        assert!(locks.can_gain_resonance(30));
    }

    #[test]
    fn skill_impact_context_preserves_first_hit_and_spawned_area() {
        let delivery_id = Uuid::from_u128(0xAA);
        let first_hit_unit_id: UnitInstanceId = Uuid::from_u128(0xBB).into();
        let area_id = Uuid::from_u128(0xCC);
        let context = SkillImpactContext {
            delivery_id,
            impact_time_ms: 123,
            impact_position: WorldVec2::new(0.00001, 0.00002),
            direction_hint: Some(WorldVec2::new(0.00003, 0.00004)),
            first_hit_unit_id: Some(first_hit_unit_id),
            hit_unit_ids: vec![first_hit_unit_id],
            spawned_area_id: Some(area_id),
        };

        assert_eq!(context.delivery_id, delivery_id);
        assert_eq!(context.first_hit_unit_id, Some(first_hit_unit_id));
        assert_eq!(context.spawned_area_id, Some(area_id));
        assert_eq!(
            context.direction_hint,
            Some(WorldVec2::new(0.00003, 0.00004))
        );
    }
}
