use std::collections::BTreeMap;

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
                BattleUnitRole, BattleUnitSourceIdentity, MobilityKind, UnitSnapshot,
                UnitTargetTrait,
            },
        },
        data::abnormality_data::BasicAttackDef,
        enums::Side,
        stats::UnitStats,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileGuidance {
    Homing,
    Fixed,
}

pub type SkillDeliveryId = Uuid;
pub type AreaInstanceId = Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectileRecord {
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

#[derive(Debug, Clone, PartialEq)]
pub struct AreaRuntime {
    pub area_id: AreaInstanceId,
    pub cast_seq: u64,
    pub step_index: usize,
    pub skill_id: SkillId,
    pub step_id: String,
    pub caster_instance_id: UnitInstanceId,
    pub caster_owner: Side,
    pub anchor: SkillAreaAnchorSource,
    pub tile_origin: SkillTileAreaOrigin,
    pub tracking: SkillAreaTracking,
    pub origin: WorldVec2,
    pub center: WorldVec2,
    pub direction_hint: WorldVec2,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActionLocks {
    pub movement_until_ms: u64,
    pub basic_attack_until_ms: u64,
    pub resonance_gain_until_ms: u64,
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

/// 전투 중 사용되는 아티팩트 런타임 표현
#[derive(Debug, Clone)]
pub(super) struct RuntimeArtifact {
    pub(super) instance_id: Uuid,
    pub(super) owner: Side,
    pub(super) base_uuid: Uuid,
}

/// 전투 중 사용되는 장비 런타임 표현
#[derive(Debug, Clone)]
pub(super) struct RuntimeItem {
    pub(super) instance_id: Uuid,
    pub(super) owner: Side,
    pub(super) owner_unit_instance: UnitInstanceId,
    pub(super) base_uuid: Uuid,
}

/// 트리거 수집 시 소스 구분
#[derive(Debug, Clone, Copy)]
pub(super) enum TriggerSource {
    Artifact { side: Side },
    Item { unit_instance_id: UnitInstanceId },
}

#[derive(Debug, Clone)]
pub(super) struct PendingSkillCast {
    pub(super) skill_id: SkillId,
    pub(super) cast_target: Option<SkillCastTarget>,
    pub(super) start_seq: u64,
}

#[derive(Debug, Clone, Default)]
pub(super) struct SkillStepResult {
    pub(super) resolved_target_count: usize,
    pub(super) actual_damage_target_count: usize,
    pub(super) applied_effect_count: usize,
    pub(super) scheduled_attack_count: usize,
}

impl SkillStepResult {
    pub(super) fn merge(&mut self, other: &Self) {
        self.resolved_target_count = self
            .resolved_target_count
            .saturating_add(other.resolved_target_count);
        self.actual_damage_target_count = self
            .actual_damage_target_count
            .saturating_add(other.actual_damage_target_count);
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
    pub(super) actual_damage_target_count: usize,
    pub(super) killed_target_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkillStepDebugResult {
    pub resolved_target_count: usize,
    pub actual_damage_target_count: usize,
    pub applied_effect_count: usize,
    pub scheduled_attack_count: usize,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillStepDebugProgress {
    pub result: SkillStepDebugResult,
    pub started: bool,
    pub pending_delivery_count: usize,
    pub terminal: bool,
}

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
    #[allow(dead_code)]
    pub(super) caster_instance_id: UnitInstanceId,
    pub(super) skill_id: SkillId,
    pub(super) caster_owner: Side,
    pub(super) anchor_position: Position,
    pub(super) cast_target_anchor_position: Option<Position>,
    pub(super) cast_target_anchor_world_position: Option<WorldVec2>,
    pub(super) allow_dead_caster: bool,
    pub(super) total_steps: usize,
    pub(super) step_progress: BTreeMap<usize, SkillStepProgress>,
    pub(super) deferred_steps: BTreeMap<usize, DeferredSkillStep>,
    pub(super) impact_contexts_by_step: BTreeMap<usize, SkillImpactContext>,
    // Reserved for the spatial delivery runtime migration.
    #[allow(dead_code)]
    pub(super) last_impact_context: Option<SkillImpactContext>,
    // Reserved for the spatial delivery runtime migration.
    #[allow(dead_code)]
    pub(super) active_area_ids: Vec<AreaInstanceId>,
}

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
    Dead,
}

impl RuntimeUnitLifecycle {
    pub fn is_active(self) -> bool {
        matches!(self, RuntimeUnitLifecycle::Active)
    }

    pub fn is_dead(self) -> bool {
        matches!(self, RuntimeUnitLifecycle::Dead)
    }
}

pub struct RuntimeUnit {
    pub instance_id: UnitInstanceId,
    pub lifecycle: RuntimeUnitLifecycle,
    pub spawn_order: u64,
    pub source_owned_uuid: Uuid,
    pub owner: Side,
    pub role: BattleUnitRole,
    pub threat_class: crate::game::battle::types::BattleUnitThreatClass,
    pub base_uuid: Uuid,
    pub source_identity: BattleUnitSourceIdentity,
    pub stats: UnitStats,
    pub incoming_damage_modifiers: DamageModifiers,
    pub basic_attack: BasicAttackDef,
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
    pub ranged_reposition_until_ms: u64,
    pub resonance_current: u32,
    pub resonance_max: u32,
    pub resonance_lock_ms: u64,
    pub next_action_time: u64,
    pub pending_cast: bool,
    pub pending_cast_cause: Option<BattleEventCause>,
    pub(super) pending_skill_cast: Option<PendingSkillCast>,
}

impl RuntimeUnit {
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
        self.body.clone()
    }

    pub fn apply_movement_body_position(&mut self, body: &UnitBody) {
        self.body = body.clone();
    }

    pub fn set_world_position(&mut self, position: WorldVec2) {
        self.body.previous_position = self.body.position;
        self.body.position = position;
        self.body.velocity = WorldVec2::ZERO;
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
        self.is_active() && self.is_combatant() && self.stats.move_speed_units_per_ms > 0
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
    pub(super) stacks: u8,
    pub(super) expires_at_ms: u64,
    pub(super) next_tick_ms: Option<u64>,
}

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
