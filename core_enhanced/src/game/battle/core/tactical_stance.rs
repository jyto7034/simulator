use crate::game::{
    battle::{
        enums::BattleEvent,
        event_log::{BattleLogEvent, HpChangeReason},
        ids::UnitInstanceId,
        types::{EngagementStance, TacticalTileKind, UnitExclusiveActivity},
    },
    behavior::{ActionDisabledReasonCode, ActionKind, GameError},
    data::equipment_data::WeaponCombatProfile,
    enums::Side,
};

use super::{BattleCore, UnitTacticalState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TacticalTileEffect {
    None,
    DefenseFocus,
    Rest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DerivedTacticalEffects {
    tile_effect: TacticalTileEffect,
    focus_damage_reduction_percent: u32,
    block_capacity: u32,
}

impl DerivedTacticalEffects {
    fn from_state(state: &UnitTacticalState) -> Self {
        if state.activity == UnitExclusiveActivity::FieldWorking {
            return Self {
                tile_effect: TacticalTileEffect::None,
                focus_damage_reduction_percent: 0,
                block_capacity: 0,
            };
        }

        match (state.stance, state.tile_kind) {
            (EngagementStance::Attack, TacticalTileKind::Ground) => Self {
                tile_effect: TacticalTileEffect::None,
                focus_damage_reduction_percent: 0,
                block_capacity: state.base_block_capacity,
            },
            (EngagementStance::Attack, TacticalTileKind::Platform) => Self {
                tile_effect: TacticalTileEffect::None,
                focus_damage_reduction_percent: 0,
                block_capacity: 0,
            },
            (EngagementStance::Defense, TacticalTileKind::Ground) => {
                let focus = state.weapon_profile.ground_defense_focus;
                Self {
                    tile_effect: TacticalTileEffect::DefenseFocus,
                    focus_damage_reduction_percent: focus
                        .map(|profile| profile.incoming_damage_reduction_percent)
                        .unwrap_or(0),
                    block_capacity: state.base_block_capacity.saturating_add(
                        focus
                            .map(|profile| profile.block_capacity_bonus)
                            .unwrap_or(0),
                    ),
                }
            }
            (EngagementStance::Defense, TacticalTileKind::Platform) => Self {
                tile_effect: TacticalTileEffect::Rest,
                focus_damage_reduction_percent: 0,
                block_capacity: 0,
            },
        }
    }

    fn rest_active(self) -> bool {
        self.tile_effect == TacticalTileEffect::Rest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitTacticalSnapshot {
    pub stance: EngagementStance,
    pub tile_kind: TacticalTileKind,
    pub tile_effect: TacticalTileEffect,
    pub stance_cooldown_until_ms: u64,
    pub relocation_cooldown_until_ms: u64,
    pub forced_defense_until_ms: Option<u64>,
    pub forced_defense_remaining_ms: u64,
    pub relocation_disabled_reason: Option<ActionDisabledReasonCode>,
    pub attack_disabled_reason: Option<ActionDisabledReasonCode>,
    pub defense_disabled_reason: Option<ActionDisabledReasonCode>,
    pub base_block_capacity: u32,
    pub effective_block_capacity: u32,
    pub defense_focus_damage_reduction_percent: u32,
    pub next_rest_tick_ms: Option<u64>,
    pub activity: UnitExclusiveActivity,
}

impl BattleCore {
    /// Set the post-move stance after work assignment, before effect reconciliation.
    pub(super) fn begin_relocation_defense(
        &mut self,
        unit_id: UnitInstanceId,
        now: u64,
        changed_tile: bool,
    ) -> Result<(), GameError> {
        let state = self
            .tactical_states
            .get_mut(&unit_id)
            .ok_or(GameError::UnitNotFound)?;
        if state.activity == UnitExclusiveActivity::FieldWorking {
            return Ok(());
        }
        let until = now.checked_add(1500).ok_or_else(|| {
            crate::game::battle::runtime_control::invariant("forced_defense_deadline")
        })?;
        let epoch = state.forced_defense_epoch.checked_add(1).ok_or_else(|| {
            crate::game::battle::runtime_control::invariant("forced_defense_epoch")
        })?;
        let before = state.stance;
        state.stance = EngagementStance::Defense;
        state.forced_defense_until_ms = Some(until);
        state.forced_defense_epoch = epoch;
        state.stance_cooldown_until_ms = state.stance_cooldown_until_ms.max(until);
        if changed_tile {
            state.next_rest_tick_ms = None;
        }
        let cooldown_until_ms = state.stance_cooldown_until_ms;
        self.event_queue.push(BattleEvent::ForcedDefenseEnd {
            time_ms: until,
            unit_instance_id: unit_id,
            epoch,
        });
        if before != EngagementStance::Defense {
            self.record_event_log(
                now,
                BattleLogEvent::StanceChanged {
                    unit_instance_id: unit_id,
                    before,
                    after: EngagementStance::Defense,
                    cooldown_until_ms,
                },
            );
        }
        Ok(())
    }

    pub(super) fn handle_forced_defense_end(
        &mut self,
        now: u64,
        unit_id: UnitInstanceId,
        epoch: u64,
    ) {
        if !self
            .units
            .get(&unit_id)
            .is_some_and(|unit| unit.is_active())
            || !self.tactical_states.get(&unit_id).is_some_and(|state| {
                state.forced_defense_epoch == epoch
                    && state.forced_defense_until_ms == Some(now)
                    && state.activity == UnitExclusiveActivity::Ready
            })
        {
            return;
        }
        let before_effects = self
            .derived_tactical_effects(unit_id)
            .expect("validated tactical state");
        let state = self
            .tactical_states
            .get_mut(&unit_id)
            .expect("validated tactical state");
        let before = state.stance;
        state.stance = EngagementStance::Attack;
        state.forced_defense_until_ms = None;
        let cooldown_until_ms = state.stance_cooldown_until_ms;
        self.reconcile_tactical_effects(unit_id, before_effects, now);
        self.refresh_block_state();
        self.record_event_log(
            now,
            BattleLogEvent::StanceChanged {
                unit_instance_id: unit_id,
                before,
                after: EngagementStance::Attack,
                cooldown_until_ms,
            },
        );
        if let Some(unit) = self.units.get_mut(&unit_id) {
            if unit.skill_activation_mode == crate::game::ability::SkillActivationMode::Auto
                && unit.resonance_current >= unit.resonance_max.max(1)
            {
                unit.pending_cast = true;
            }
        }
        self.try_start_pending_basic_attacks(now);
        self.schedule_pending_autocasts(now);
    }

    pub(in crate::game::battle::core) fn initialize_deployed_tactical_state(
        &mut self,
        unit_instance_id: UnitInstanceId,
        tile_kind: TacticalTileKind,
        weapon_profile: WeaponCombatProfile,
        time_ms: u64,
    ) -> Result<(), GameError> {
        match tile_kind {
            TacticalTileKind::Ground if weapon_profile.ground_defense_focus.is_none() => {
                return Err(GameError::InvalidStaticData(
                    "ground deployment weapon profile is missing ground_defense_focus".to_string(),
                ));
            }
            TacticalTileKind::Platform if weapon_profile.platform_rest.is_none() => {
                return Err(GameError::InvalidStaticData(
                    "platform deployment weapon profile is missing platform_rest".to_string(),
                ));
            }
            _ => {}
        }

        let Some(unit) = self.units.get_mut(&unit_instance_id) else {
            return Err(GameError::UnitNotFound);
        };
        if self
            .independent_objects
            .objects
            .contains_key(&unit_instance_id)
            || unit.owner != Side::Player
            || !unit.is_combatant()
            || !unit.is_active()
        {
            return Err(GameError::ActionUnavailable {
                action: ActionKind::SetEngagementStance,
                reason: ActionDisabledReasonCode::UnitUnavailable,
            });
        }

        let base_block_capacity = weapon_profile.block_capacity;
        unit.block_capacity = match tile_kind {
            TacticalTileKind::Ground => base_block_capacity,
            TacticalTileKind::Platform => 0,
        };

        let stance_cooldown_until_ms = time_ms.saturating_add(
            self.game_data
                .run_policy
                .tactical_stance
                .stance_change_cooldown_ms,
        );
        self.tactical_states.insert(
            unit_instance_id,
            UnitTacticalState {
                stance: EngagementStance::Attack,
                tile_kind,
                weapon_profile,
                base_block_capacity,
                stance_cooldown_until_ms,
                relocation_cooldown_until_ms: 0,
                forced_defense_until_ms: None,
                forced_defense_epoch: 0,
                rest_epoch: 0,
                next_rest_tick_ms: None,
                activity: UnitExclusiveActivity::Ready,
            },
        );
        self.refresh_block_state();
        Ok(())
    }

    pub(super) fn derived_tactical_effects(
        &self,
        unit_instance_id: UnitInstanceId,
    ) -> Option<DerivedTacticalEffects> {
        self.tactical_states
            .get(&unit_instance_id)
            .map(DerivedTacticalEffects::from_state)
    }

    pub(super) fn reconcile_tactical_effects(
        &mut self,
        unit_instance_id: UnitInstanceId,
        before: DerivedTacticalEffects,
        time_ms: u64,
    ) {
        let after = self
            .derived_tactical_effects(unit_instance_id)
            .expect("tactical effect reconciliation requires tactical state");
        let before_focus = i32::try_from(before.focus_damage_reduction_percent).unwrap_or(i32::MAX);
        let after_focus = i32::try_from(after.focus_damage_reduction_percent).unwrap_or(i32::MAX);
        let unit = self
            .units
            .get_mut(&unit_instance_id)
            .expect("tactical effect reconciliation requires live unit");
        if before_focus != after_focus {
            unit.incoming_damage_modifiers.damage_reduction_percent = unit
                .incoming_damage_modifiers
                .damage_reduction_percent
                .saturating_sub(before_focus)
                .saturating_add(after_focus);
        }
        unit.block_capacity = after.block_capacity;

        let rest_changed = before.rest_active() != after.rest_active();
        let rest_interval_ms = self
            .game_data
            .run_policy
            .tactical_stance
            .rest_tick_interval_ms;
        let (rest_epoch, rest_tick) = {
            let state = self
                .tactical_states
                .get_mut(&unit_instance_id)
                .expect("tactical effect reconciliation requires tactical state");
            let restart_rest =
                rest_changed || (after.rest_active() && state.next_rest_tick_ms.is_none());
            if restart_rest {
                state.rest_epoch = state.rest_epoch.wrapping_add(1);
            }
            if after.rest_active() {
                if restart_rest {
                    state.next_rest_tick_ms = Some(time_ms.saturating_add(rest_interval_ms));
                }
            } else {
                state.next_rest_tick_ms = None;
            }
            (
                state.rest_epoch,
                restart_rest.then_some(state.next_rest_tick_ms).flatten(),
            )
        };
        if let Some(rest_tick_ms) = rest_tick {
            self.event_queue.push(BattleEvent::RestTick {
                time_ms: rest_tick_ms,
                unit_instance_id,
                rest_epoch,
            });
        }
    }

    pub fn unit_tactical_snapshot(
        &self,
        unit_instance_id: UnitInstanceId,
        now_ms: u64,
    ) -> Option<UnitTacticalSnapshot> {
        let state = self.tactical_states.get(&unit_instance_id)?;
        let unit = self.units.get(&unit_instance_id)?;
        if !unit.is_active() {
            return None;
        }

        let effects = DerivedTacticalEffects::from_state(state);

        Some(UnitTacticalSnapshot {
            stance: state.stance,
            tile_kind: state.tile_kind,
            tile_effect: effects.tile_effect,
            stance_cooldown_until_ms: state.stance_cooldown_until_ms,
            relocation_cooldown_until_ms: state.relocation_cooldown_until_ms,
            forced_defense_until_ms: state.forced_defense_until_ms,
            forced_defense_remaining_ms: state
                .forced_defense_until_ms
                .map_or(0, |until| until.saturating_sub(now_ms)),
            relocation_disabled_reason: self
                .tactical_relocation_disabled_reason(unit_instance_id, now_ms),
            attack_disabled_reason: self.stance_change_disabled_reason(
                unit_instance_id,
                EngagementStance::Attack,
                now_ms,
            ),
            defense_disabled_reason: self.stance_change_disabled_reason(
                unit_instance_id,
                EngagementStance::Defense,
                now_ms,
            ),
            base_block_capacity: state.base_block_capacity,
            effective_block_capacity: if self
                .unit_control_state(unit_instance_id, now_ms)
                .is_some_and(|control| control.hard_cc_active())
            {
                0
            } else {
                self.effective_block_capacity(unit_instance_id)
            },
            defense_focus_damage_reduction_percent: effects.focus_damage_reduction_percent,
            next_rest_tick_ms: state.next_rest_tick_ms,
            activity: state.activity,
        })
    }

    pub fn stance_change_disabled_reason(
        &self,
        unit_instance_id: UnitInstanceId,
        target_stance: EngagementStance,
        time_ms: u64,
    ) -> Option<ActionDisabledReasonCode> {
        let Some(unit) = self.units.get(&unit_instance_id) else {
            return Some(ActionDisabledReasonCode::UnitUnavailable);
        };
        let Some(state) = self.tactical_states.get(&unit_instance_id) else {
            return Some(ActionDisabledReasonCode::WeaponRequired);
        };
        if self
            .independent_objects
            .objects
            .contains_key(&unit_instance_id)
            || unit.owner != Side::Player
            || !unit.is_combatant()
            || !unit.is_active()
        {
            return Some(ActionDisabledReasonCode::UnitUnavailable);
        }
        if state.activity == UnitExclusiveActivity::FieldWorking {
            return Some(ActionDisabledReasonCode::FieldWorking);
        }
        if state.forced_defense_until_ms.is_some() {
            return Some(ActionDisabledReasonCode::ForcedDefense);
        }
        if state.stance == target_stance {
            return Some(ActionDisabledReasonCode::AlreadyActive);
        }
        if self
            .unit_control_state(unit_instance_id, time_ms)
            .is_some_and(|control| control.hard_cc_active())
        {
            return Some(ActionDisabledReasonCode::HardCrowdControl);
        }
        if time_ms < state.stance_cooldown_until_ms {
            return Some(ActionDisabledReasonCode::StanceCooldown);
        }
        None
    }

    pub fn set_engagement_stance(
        &mut self,
        unit_instance_id: UnitInstanceId,
        target_stance: EngagementStance,
        time_ms: u64,
    ) -> Result<(), GameError> {
        self.runtime_transaction(|core| {
            core.ensure_not_forced_paused()?;
            core.set_engagement_stance_in_candidate(unit_instance_id, target_stance, time_ms)
        })
    }

    pub(in crate::game::battle::core) fn set_engagement_stance_in_candidate(
        &mut self,
        unit_instance_id: UnitInstanceId,
        target_stance: EngagementStance,
        time_ms: u64,
    ) -> Result<(), GameError> {
        if let Some(reason) =
            self.stance_change_disabled_reason(unit_instance_id, target_stance, time_ms)
        {
            return Err(GameError::ActionUnavailable {
                action: ActionKind::SetEngagementStance,
                reason,
            });
        }

        let (before, before_effects) = {
            let state = self
                .tactical_states
                .get(&unit_instance_id)
                .ok_or(GameError::InvalidAction)?;
            (state.stance, DerivedTacticalEffects::from_state(state))
        };

        let cooldown_until_ms = time_ms.saturating_add(
            self.game_data
                .run_policy
                .tactical_stance
                .stance_change_cooldown_ms,
        );
        {
            let state = self
                .tactical_states
                .get_mut(&unit_instance_id)
                .ok_or(GameError::InvalidAction)?;
            state.stance = target_stance;
            state.stance_cooldown_until_ms = cooldown_until_ms;
        }
        self.reconcile_tactical_effects(unit_instance_id, before_effects, time_ms);
        self.refresh_block_state();
        self.record_event_log(
            time_ms,
            BattleLogEvent::StanceChanged {
                unit_instance_id,
                before,
                after: target_stance,
                cooldown_until_ms,
            },
        );

        if target_stance == EngagementStance::Attack {
            if let Some(unit) = self.units.get_mut(&unit_instance_id) {
                if unit.skill_activation_mode == crate::game::ability::SkillActivationMode::Auto
                    && unit.resonance_current >= unit.resonance_max.max(1)
                {
                    unit.pending_cast = true;
                }
            }
            self.try_start_pending_basic_attacks(time_ms);
            self.schedule_pending_autocasts(time_ms);
        }
        Ok(())
    }

    pub(in crate::game::battle::core) fn unit_is_in_defense_stance(
        &self,
        unit_instance_id: UnitInstanceId,
    ) -> bool {
        self.tactical_states
            .get(&unit_instance_id)
            .is_some_and(|state| state.stance == EngagementStance::Defense)
    }

    pub(in crate::game::battle::core) fn handle_rest_tick(
        &mut self,
        time_ms: u64,
        unit_instance_id: UnitInstanceId,
        rest_epoch: u64,
    ) {
        let Some(state) = self.tactical_states.get(&unit_instance_id) else {
            return;
        };
        if state.rest_epoch != rest_epoch
            || state.stance != EngagementStance::Defense
            || state.tile_kind != TacticalTileKind::Platform
            || state.activity != UnitExclusiveActivity::Ready
            || state.next_rest_tick_ms != Some(time_ms)
        {
            return;
        }
        let Some(rest_profile) = state.weapon_profile.platform_rest else {
            return;
        };
        if !self
            .units
            .get(&unit_instance_id)
            .is_some_and(|unit| unit.is_active())
        {
            return;
        }

        let hard_cc_active = self
            .unit_control_state(unit_instance_id, time_ms)
            .is_some_and(|control| control.hard_cc_active());
        if !hard_cc_active {
            if rest_profile.hp_recovery_per_tick > 0
                && self
                    .units
                    .get(&unit_instance_id)
                    .is_some_and(|unit| unit.stats.current_health < unit.stats.max_health)
            {
                self.apply_hp_delta_and_record(
                    None,
                    unit_instance_id,
                    i32::try_from(rest_profile.hp_recovery_per_tick).unwrap_or(i32::MAX),
                    time_ms,
                    HpChangeReason::Rest,
                );
            }
        }

        let interval_ms = self
            .game_data
            .run_policy
            .tactical_stance
            .rest_tick_interval_ms;
        let next_tick_ms = time_ms.saturating_add(interval_ms);
        if let Some(state) = self.tactical_states.get_mut(&unit_instance_id) {
            if state.rest_epoch != rest_epoch
                || state.stance != EngagementStance::Defense
                || state.tile_kind != TacticalTileKind::Platform
            {
                return;
            }
            state.next_rest_tick_ms = Some(next_tick_ms);
        }
        self.event_queue.push(BattleEvent::RestTick {
            time_ms: next_tick_ms,
            unit_instance_id,
            rest_epoch,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::battle::core::movement::types::{ActiveMovementSegment, WorldVec2};
    use crate::game::battle::core::sim::{BattleLiveCommand, BattleLiveCommandOutcome};
    use crate::game::battle::core::types::PendingSkillCast;
    use crate::game::{
        battle::{
            damage::{
                calculate_damage, DamageContext, DamageModifiers, DamageRequest, DamageSource,
                DamageType,
            },
            event_log::BattleEventCause,
            scenario::BattleScenario,
            tile_range::FacingDirection,
            types::{
                BattleUnitDraft, BattleUnitRole, BattleUnitSource, BattleUnitThreatClass,
                DeploymentAffinity, UnitCombatProfile,
            },
        },
        data::{
            equipment_data::{GroundDefenseFocusProfile, PlatformRestProfile},
            GameDataBuilder,
        },
        enums::Tier,
        resources::Position,
    };
    use uuid::Uuid;

    fn weapon_profile(tile_kind: TacticalTileKind) -> WeaponCombatProfile {
        let mut profile = WeaponCombatProfile::default();
        match tile_kind {
            TacticalTileKind::Ground => {
                profile.deployment_affinity = DeploymentAffinity::GroundOnly;
                profile.block_capacity = 2;
                profile.ground_defense_focus = Some(GroundDefenseFocusProfile {
                    block_capacity_bonus: 3,
                    incoming_damage_reduction_percent: 10,
                });
                profile.platform_rest = None;
            }
            TacticalTileKind::Platform => {
                profile.deployment_affinity = DeploymentAffinity::PlatformOnly;
                profile.block_capacity = 0;
                profile.ground_defense_focus = None;
                profile.platform_rest = Some(PlatformRestProfile {
                    hp_recovery_per_tick: 3,
                });
            }
        }
        profile
    }

    fn fixture_draft(owned_uuid: u128, weapon_profile: WeaponCombatProfile) -> BattleUnitDraft {
        let mut combat_profile = UnitCombatProfile::employee_default();
        combat_profile.weapon_profile = Some(weapon_profile.clone());
        combat_profile.block_capacity = weapon_profile.block_capacity;
        BattleUnitDraft {
            owned_uuid: Uuid::from_u128(owned_uuid),
            source: BattleUnitSource::TestFixture {
                base_uuid: Uuid::from_u128(owned_uuid.saturating_add(1)),
                profile: combat_profile,
            },
            threat_class: BattleUnitThreatClass::Normal,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: Default::default(),
            equipment_layout: None,
            equipped_items: Vec::new(),
            equipped_item_enhancements: Vec::new(),
            employee_start_condition: None,
        }
    }

    fn deployed_core(tile_kind: TacticalTileKind) -> (BattleCore, UnitInstanceId) {
        let draft = fixture_draft(0x5101, weapon_profile(tile_kind));
        let mut core = BattleCore::new_from_scenario(
            BattleScenario::empty((4, 4)),
            GameDataBuilder::live_defaults().build_arc(),
            0x5103,
        );
        let outcome = core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft,
                position: Position::new(1, 1),
                tile_kind,
                facing: FacingDirection::Right,
                instance_salt: 1,
                time_ms: 0,
                current_hp_policy: None,
            })
            .expect("fixture deploy must succeed");
        let BattleLiveCommandOutcome::UnitDeployed { unit_id } = outcome else {
            panic!("unexpected deploy outcome: {outcome:?}");
        };
        (core, unit_id)
    }

    fn any_tile_weapon_profile() -> WeaponCombatProfile {
        let mut profile = WeaponCombatProfile::default();
        profile.deployment_affinity = DeploymentAffinity::Any;
        profile.block_capacity = 2;
        profile.ground_defense_focus = Some(GroundDefenseFocusProfile {
            block_capacity_bonus: 3,
            incoming_damage_reduction_percent: 10,
        });
        profile.platform_rest = Some(PlatformRestProfile {
            hp_recovery_per_tick: 3,
        });
        profile
    }

    #[test]
    fn attack_defense_toggle_enforces_cooldown_and_derives_ground_focus() {
        let (mut core, unit_id) = deployed_core(TacticalTileKind::Ground);
        let initial = core.unit_tactical_snapshot(unit_id, 0).unwrap();
        assert_eq!(initial.stance, EngagementStance::Attack);
        assert_eq!(initial.tile_effect, TacticalTileEffect::None);
        assert_eq!(initial.stance_cooldown_until_ms, 1_500);
        assert_eq!(initial.effective_block_capacity, 2);
        assert_eq!(
            initial.defense_disabled_reason,
            Some(ActionDisabledReasonCode::StanceCooldown)
        );

        let before_rejection = core.event_log.entries.len();
        assert!(matches!(
            core.set_engagement_stance(unit_id, EngagementStance::Defense, 1_499),
            Err(GameError::ActionUnavailable {
                action: ActionKind::SetEngagementStance,
                reason: ActionDisabledReasonCode::StanceCooldown,
            })
        ));
        assert_eq!(core.event_log.entries.len(), before_rejection);

        core.set_engagement_stance(unit_id, EngagementStance::Defense, 1_500)
            .unwrap();
        let defense = core.unit_tactical_snapshot(unit_id, 1_500).unwrap();
        assert_eq!(defense.stance, EngagementStance::Defense);
        assert_eq!(defense.tile_effect, TacticalTileEffect::DefenseFocus);
        assert_eq!(defense.effective_block_capacity, 5);
        assert_eq!(defense.defense_focus_damage_reduction_percent, 10);
        assert_eq!(
            core.units[&unit_id]
                .incoming_damage_modifiers
                .damage_reduction_percent,
            10
        );
        core.units
            .get_mut(&unit_id)
            .unwrap()
            .incoming_damage_modifiers
            .physical_damage_reduction_percent = 95;
        let request = DamageRequest {
            source: DamageSource::Ability,
            damage_type: DamageType::Physical,
            modifiers: DamageModifiers::default(),
            crit_roll_percent: None,
            attacker_id: UnitInstanceId::from(Uuid::from_u128(0x51A0)),
            target_id: unit_id,
            base_damage: 100,
            minimum_damage: 0,
            time_ms: 1_500,
        };
        let calculate = |modifiers| {
            calculate_damage(
                &request,
                &DamageContext {
                    attacker_side: Side::Opponent,
                    target_side: Side::Player,
                    attacker_attack: 100,
                    target_armor: 0,
                    target_magic_resist: 0,
                    target_incoming_modifiers: modifiers,
                    target_current_hp: 100,
                    target_max_hp: 100,
                    on_attack_effects: &[],
                    on_hit_effects: &[],
                },
            )
        };
        assert_eq!(
            calculate(core.units[&unit_id].incoming_damage_modifiers).final_damage,
            0,
            "general focus reduction must add to typed reduction and clamp at 100%"
        );
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1_501,
                caster_instance_id: unit_id,
                target_instance_id: unit_id,
                buff_id: crate::game::battle::buffs::BuffId::from_name("freeze"),
                duration_ms: 10,
                cause: BattleEventCause::default(),
            },
            1_501,
        )
        .unwrap();
        assert_eq!(
            core.unit_tactical_snapshot(unit_id, 1_501)
                .unwrap()
                .effective_block_capacity,
            0
        );
        assert_eq!(
            core.units[&unit_id]
                .incoming_damage_modifiers
                .damage_reduction_percent,
            10,
            "Hard CC must preserve DefenseFocus damage reduction"
        );
        core.process_event(
            BattleEvent::BuffExpire {
                time_ms: 1_511,
                caster_instance_id: unit_id,
                target_instance_id: unit_id,
                buff_id: crate::game::battle::buffs::BuffId::from_name("freeze"),
                cause: BattleEventCause::default(),
            },
            1_511,
        )
        .unwrap();

        let before_same = core.event_log.entries.len();
        assert!(matches!(
            core.set_engagement_stance(unit_id, EngagementStance::Defense, 3_000),
            Err(GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::AlreadyActive,
                ..
            })
        ));
        assert_eq!(core.event_log.entries.len(), before_same);

        core.set_engagement_stance(unit_id, EngagementStance::Attack, 3_000)
            .unwrap();
        let attack = core.unit_tactical_snapshot(unit_id, 3_000).unwrap();
        assert_eq!(attack.tile_effect, TacticalTileEffect::None);
        assert_eq!(attack.effective_block_capacity, 2);
        assert_eq!(
            core.units[&unit_id]
                .incoming_damage_modifiers
                .damage_reduction_percent,
            0
        );
        assert_eq!(
            calculate(core.units[&unit_id].incoming_damage_modifiers).final_damage,
            5
        );
    }

    #[test]
    fn platform_rest_uses_full_intervals_clamps_and_skips_without_catch_up() {
        let (mut core, unit_id) = deployed_core(TacticalTileKind::Platform);
        core.set_engagement_stance(unit_id, EngagementStance::Defense, 1_500)
            .unwrap();
        {
            let unit = core.units.get_mut(&unit_id).unwrap();
            unit.stats.current_health = 50;
            unit.resonance_current = 98;
            unit.resonance_max = 100;
        }
        let event_count = core.event_log.entries.len();
        core.handle_rest_tick(2_499, unit_id, 1);
        assert_eq!(core.event_log.entries.len(), event_count);
        assert_eq!(core.units[&unit_id].stats.current_health, 50);

        core.handle_rest_tick(2_500, unit_id, 1);
        assert_eq!(core.units[&unit_id].stats.current_health, 53);
        assert_eq!(core.units[&unit_id].resonance_current, 98);
        assert_eq!(
            core.unit_tactical_snapshot(unit_id, 2_500)
                .unwrap()
                .next_rest_tick_ms,
            Some(3_500)
        );

        {
            let unit = core.units.get_mut(&unit_id).unwrap();
            unit.stats.current_health = unit.stats.max_health;
            unit.resonance_current = unit.resonance_max;
        }
        let full_event_count = core.event_log.entries.len();
        core.handle_rest_tick(3_500, unit_id, 1);
        assert_eq!(core.event_log.entries.len(), full_event_count);

        {
            let unit = core.units.get_mut(&unit_id).unwrap();
            unit.resonance_current = 90;
            unit.action_locks.lock_resonance_gain_until(5_000);
        }
        core.handle_rest_tick(4_500, unit_id, 1);
        assert_eq!(core.units[&unit_id].resonance_current, 90);
        core.handle_rest_tick(5_500, unit_id, 1);
        assert_eq!(core.units[&unit_id].resonance_current, 90);

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 6_000,
                caster_instance_id: unit_id,
                target_instance_id: unit_id,
                buff_id: crate::game::battle::buffs::BuffId::from_name("freeze"),
                duration_ms: 1_000,
                cause: BattleEventCause::default(),
            },
            6_000,
        )
        .unwrap();
        assert!(matches!(
            core.set_engagement_stance(unit_id, EngagementStance::Attack, 6_000),
            Err(GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::HardCrowdControl,
                ..
            })
        ));
        core.handle_rest_tick(6_500, unit_id, 1);
        assert_eq!(core.units[&unit_id].resonance_current, 90);
        core.handle_rest_tick(7_500, unit_id, 1);
        assert_eq!(core.units[&unit_id].resonance_current, 90);

        core.set_engagement_stance(unit_id, EngagementStance::Attack, 7_500)
            .unwrap();
        core.units.get_mut(&unit_id).unwrap().stats.current_health = 50;
        let stale_event_count = core.event_log.entries.len();
        core.handle_rest_tick(8_500, unit_id, 1);
        assert_eq!(core.units[&unit_id].stats.current_health, 50);
        assert_eq!(core.event_log.entries.len(), stale_event_count);
        core.set_engagement_stance(unit_id, EngagementStance::Defense, 9_000)
            .unwrap();
        assert_eq!(
            core.unit_tactical_snapshot(unit_id, 9_000)
                .unwrap()
                .next_rest_tick_ms,
            Some(10_000),
            "regaining Rest must require a fresh full interval"
        );
    }

    #[test]
    fn same_timestamp_damage_precedes_rest_and_death_is_not_revived() {
        let (mut core, unit_id) = deployed_core(TacticalTileKind::Platform);
        core.event_queue.clear();
        core.set_engagement_stance(unit_id, EngagementStance::Defense, 1_500)
            .unwrap();
        core.units.get_mut(&unit_id).unwrap().stats.current_health = 1;
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1_500,
                caster_instance_id: unit_id,
                target_instance_id: unit_id,
                buff_id: crate::game::battle::buffs::BuffId::from_name("poison"),
                duration_ms: 2_000,
                cause: BattleEventCause::default(),
            },
            1_500,
        )
        .unwrap();

        let first_at_tick = core
            .event_queue
            .pop()
            .expect("damage tick must be scheduled");
        assert!(matches!(
            first_at_tick.event.view().into_owned(),
            BattleEvent::BuffTick { time_ms: 2_500, .. }
        ));
        core.process_event(first_at_tick, 2_500).unwrap();
        assert_eq!(core.units[&unit_id].stats.current_health, 0);

        let rest = core
            .event_queue
            .pop()
            .expect("same-time Rest must remain queued");
        assert!(matches!(
            rest.event.view().into_owned(),
            BattleEvent::RestTick { time_ms: 2_500, .. }
        ));
        core.process_event(rest, 2_500).unwrap();
        assert_eq!(core.units[&unit_id].stats.current_health, 0);
        assert!(!core.event_log.entries.iter().any(|entry| {
            entry.time_ms == 2_500
                && matches!(
                    entry.event,
                    BattleLogEvent::HpChanged {
                        reason: HpChangeReason::Rest,
                        ..
                    }
                )
        }));
    }

    #[test]
    fn defense_blocks_new_starts_but_preserves_committed_and_triggered_attacks() {
        let (mut core, unit_id) = deployed_core(TacticalTileKind::Ground);
        let target_outcome = core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft: fixture_draft(0x5201, weapon_profile(TacticalTileKind::Ground)),
                position: Position::new(2, 1),
                tile_kind: TacticalTileKind::Ground,
                facing: FacingDirection::Left,
                instance_salt: 2,
                time_ms: 0,
                current_hp_policy: None,
            })
            .unwrap();
        let BattleLiveCommandOutcome::UnitDeployed { unit_id: target_id } = target_outcome else {
            panic!("unexpected target deploy outcome: {target_outcome:?}");
        };
        core.units.get_mut(&target_id).unwrap().owner = Side::Opponent;
        core.tactical_states.remove(&target_id);
        core.event_queue.clear();

        let target_hp = core.units[&target_id].stats.current_health;
        core.handle_basic_attack_start_event(
            1_500,
            1_500,
            unit_id,
            None,
            true,
            BattleEventCause::default(),
        );
        let committed_resolve = core
            .event_queue
            .iter()
            .find(|event| matches!(event.as_ref(), BattleEvent::AttackResolve { .. }))
            .map(std::borrow::Cow::into_owned)
            .expect("attack begun in Attack stance must commit its resolve");
        core.set_engagement_stance(unit_id, EngagementStance::Defense, 1_500)
            .unwrap();
        core.process_event(committed_resolve.clone(), committed_resolve.time_ms())
            .unwrap();
        assert!(core.units[&target_id].stats.current_health < target_hp);

        core.event_queue.clear();
        core.units.get_mut(&unit_id).unwrap().pending_basic_attack = false;

        core.handle_basic_attack_start_event(
            2_000,
            2_000,
            unit_id,
            None,
            true,
            BattleEventCause::default(),
        );
        assert!(core.units[&unit_id].pending_basic_attack);
        assert_eq!(
            core.skill_start_disabled_reason(unit_id, 2_000),
            Some(ActionDisabledReasonCode::DefenseStance)
        );
        assert!(!core
            .event_queue
            .iter()
            .any(|event| matches!(event.as_ref(), BattleEvent::AttackResolve { .. })));

        core.units.get_mut(&unit_id).unwrap().pending_basic_attack = false;
        core.handle_basic_attack_start_event(
            2_000,
            2_000,
            unit_id,
            Some(target_id),
            false,
            BattleEventCause::default(),
        );
        assert!(!core.units[&unit_id].pending_basic_attack);
        assert!(core
            .event_queue
            .iter()
            .any(|event| matches!(event.as_ref(), BattleEvent::AttackResolve { .. })));
    }

    fn u06_ready_healer() -> (BattleCore, UnitInstanceId) {
        let (mut core, id) = deployed_core(TacticalTileKind::Ground);
        core.event_queue.clear();
        let unit = core.units.get_mut(&id).unwrap();
        unit.basic_attack.action = crate::game::data::abnormality_data::BasicAttackAction::Heal {
            flat: 20,
            spell_power_percent: 0,
            footprint: None,
        };
        unit.basic_attack.windup_ms = 50;
        unit.basic_attack.interval_ms = 5000;
        unit.stats.attack_interval_ms = 5000;
        unit.stats.max_health = 100;
        unit.stats.current_health = 10;
        unit.next_basic_attack_ms = 0;
        unit.pending_basic_attack = false;
        (core, id)
    }

    #[test]
    fn u06_basic_heal_auto_gates_full_hp_defense_cc_and_silence_without_hostile_proc() {
        for mode in [
            "full",
            "defense_before",
            "freeze_before",
            "silence_before",
            "defense_after",
            "freeze_after",
            "silence_after",
        ] {
            let (mut core, id) = u06_ready_healer();
            if mode == "full" {
                core.units.get_mut(&id).unwrap().stats.current_health = 100;
            }
            if mode == "defense_before" {
                core.set_engagement_stance(id, EngagementStance::Defense, 1500)
                    .unwrap();
            }
            if mode.ends_with("_before") && mode != "defense_before" {
                core.process_event(
                    BattleEvent::ApplyBuff {
                        time_ms: 1500,
                        caster_instance_id: id,
                        target_instance_id: id,
                        buff_id: crate::game::battle::buffs::BuffId::from_name(
                            mode.trim_end_matches("_before"),
                        ),
                        duration_ms: 1000,
                        cause: BattleEventCause::default(),
                    },
                    1500,
                )
                .unwrap();
            }
            core.handle_basic_attack_start_event(
                1500,
                1500,
                id,
                None,
                true,
                BattleEventCause::default(),
            );
            let should_start = !matches!(mode, "full" | "defense_before" | "freeze_before");
            assert_eq!(
                core.event_queue
                    .iter()
                    .any(|e| matches!(e.as_ref(), BattleEvent::BasicHealResolve { .. })),
                should_start,
                "{mode}"
            );
            if !should_start {
                continue;
            }
            if mode == "defense_after" {
                core.set_engagement_stance(id, EngagementStance::Defense, 1500)
                    .unwrap();
            }
            if mode == "freeze_after" || mode == "silence_after" {
                core.process_event(
                    BattleEvent::ApplyBuff {
                        time_ms: 1525,
                        caster_instance_id: id,
                        target_instance_id: id,
                        buff_id: crate::game::battle::buffs::BuffId::from_name(
                            mode.trim_end_matches("_after"),
                        ),
                        duration_ms: 1000,
                        cause: BattleEventCause::default(),
                    },
                    1525,
                )
                .unwrap();
            }
            let event = core.event_queue.pop().unwrap();
            assert!(
                matches!(
                    event.event.view().as_ref(),
                    BattleEvent::BasicHealResolve { time_ms: 1550, .. }
                ),
                "{mode}"
            );
            core.process_event(event, 1550).unwrap();
            assert_eq!(
                core.units[&id].stats.current_health,
                if mode == "freeze_after" { 10 } else { 30 },
                "{mode}"
            );
            assert_eq!(core.runtime_fact_summary().damage_hp_lost, 0, "{mode}");
            assert_eq!(
                core.runtime_fact_view().active_actions().count(),
                0,
                "{mode}"
            );
        }
    }

    #[test]
    fn u06_basic_heal_real_queue_resolves_lethal_damage_first_without_reviving() {
        let (mut core, id) = u06_ready_healer();
        core.units.get_mut(&id).unwrap().stats.current_health = 1;
        core.units.get_mut(&id).unwrap().basic_attack.windup_ms = 1000;
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1500,
                caster_instance_id: id,
                target_instance_id: id,
                buff_id: crate::game::battle::buffs::BuffId::from_name("poison"),
                duration_ms: 2000,
                cause: BattleEventCause::default(),
            },
            1500,
        )
        .unwrap();
        core.handle_basic_attack_start_event(
            1500,
            1500,
            id,
            None,
            true,
            BattleEventCause::default(),
        );
        let damage = core.event_queue.pop().unwrap();
        assert!(matches!(
            damage.event.view().as_ref(),
            BattleEvent::BuffTick { time_ms: 2500, .. }
        ));
        core.process_event(damage, 2500).unwrap();
        assert_eq!(core.units[&id].stats.current_health, 0);
        let heal = core.event_queue.pop().unwrap();
        assert!(matches!(
            heal.event.view().as_ref(),
            BattleEvent::BasicHealResolve { time_ms: 2500, .. }
        ));
        core.process_event(heal, 2500).unwrap();
        assert_eq!(core.units[&id].stats.current_health, 0);
        assert_eq!(core.runtime_fact_summary().healing_delta, 0);
        assert_eq!(core.runtime_fact_view().active_actions().count(), 0);
    }

    #[test]
    fn u06_move_from_attack_forces_defense_then_attack_even_under_cc_and_clones_independently() {
        let (mut core, unit_id) = deployed_core(TacticalTileKind::Ground);
        core.tactically_relocate_unit(
            unit_id,
            Position::new(2, 1),
            TacticalTileKind::Ground,
            FacingDirection::Right,
            1_500,
        )
        .unwrap();
        let moved = core.unit_tactical_snapshot(unit_id, 1_500).unwrap();
        assert_eq!(moved.stance, EngagementStance::Defense);
        assert_eq!(moved.forced_defense_until_ms, Some(3_000));
        assert_eq!(moved.forced_defense_remaining_ms, 1_500);
        assert_eq!(
            moved.attack_disabled_reason,
            Some(ActionDisabledReasonCode::ForcedDefense)
        );
        let epoch = core.tactical_states[&unit_id].forced_defense_epoch;
        let mut copy = crate::authoring::clone_shape::controlled_clone(&core).unwrap();
        copy.handle_forced_defense_end(2_999, unit_id, epoch);
        copy.handle_forced_defense_end(3_000, unit_id, epoch + 1);
        assert_eq!(
            copy.unit_tactical_snapshot(unit_id, 2_999).unwrap().stance,
            EngagementStance::Defense
        );
        copy.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 2_999,
                caster_instance_id: unit_id,
                target_instance_id: unit_id,
                buff_id: crate::game::battle::buffs::BuffId::from_name("freeze"),
                duration_ms: 1_000,
                cause: BattleEventCause::default(),
            },
            2_999,
        )
        .unwrap();
        copy.process_event(
            BattleEvent::ForcedDefenseEnd {
                time_ms: 3_000,
                unit_instance_id: unit_id,
                epoch,
            },
            3_000,
        )
        .unwrap();
        let restored = copy.unit_tactical_snapshot(unit_id, 3_000).unwrap();
        assert_eq!(restored.stance, EngagementStance::Attack);
        assert_eq!(restored.forced_defense_until_ms, None);
        assert_eq!(restored.stance_cooldown_until_ms, 3_000);
        assert_eq!(
            restored.effective_block_capacity, 0,
            "Hard CC still suppresses blocking"
        );
        assert!(copy
            .unit_control_state(unit_id, 3_000)
            .unwrap()
            .hard_cc_active());
        assert_eq!(
            core.unit_tactical_snapshot(unit_id, 3_000).unwrap().stance,
            EngagementStance::Defense
        );
        let count = copy.event_log.entries.len();
        copy.handle_forced_defense_end(3_000, unit_id, epoch);
        assert_eq!(
            copy.event_log.entries.len(),
            count,
            "duplicate expiry is inert"
        );
        core.handle_forced_defense_end(3_000, unit_id, epoch);
        assert_eq!(
            core.unit_tactical_snapshot(unit_id, 3_000)
                .unwrap()
                .effective_block_capacity,
            2
        );
    }

    #[test]
    fn u06_platform_move_restarts_rest_but_manual_defense_has_no_forced_timer() {
        let (mut core, unit_id) = deployed_core(TacticalTileKind::Platform);
        core.set_engagement_stance(unit_id, EngagementStance::Defense, 1_500)
            .unwrap();
        assert_eq!(
            core.unit_tactical_snapshot(unit_id, 1_500)
                .unwrap()
                .forced_defense_until_ms,
            None
        );
        core.tactically_relocate_unit(
            unit_id,
            Position::new(2, 1),
            TacticalTileKind::Platform,
            FacingDirection::Right,
            1_900,
        )
        .unwrap();
        assert_eq!(
            core.unit_tactical_snapshot(unit_id, 1_900)
                .unwrap()
                .next_rest_tick_ms,
            Some(2_900)
        );
        core.units.get_mut(&unit_id).unwrap().stats.current_health = 50;
        core.handle_rest_tick(2_500, unit_id, 1);
        assert_eq!(core.units[&unit_id].stats.current_health, 50);
        let epoch = core.tactical_states[&unit_id].rest_epoch;
        core.handle_rest_tick(2_900, unit_id, epoch);
        assert_eq!(core.units[&unit_id].stats.current_health, 53);
        let forced_epoch = core.tactical_states[&unit_id].forced_defense_epoch;
        core.handle_forced_defense_end(3_400, unit_id, forced_epoch);
        assert_eq!(
            core.unit_tactical_snapshot(unit_id, 3_400)
                .unwrap()
                .next_rest_tick_ms,
            None
        );
        core.handle_rest_tick(3_900, unit_id, epoch);
        assert_eq!(core.units[&unit_id].stats.current_health, 53);
    }

    #[test]
    fn tactical_relocation_preserves_runtime_identity_and_migrates_defense_tile_effects() {
        let draft = fixture_draft(0x5301, any_tile_weapon_profile());
        let mut core = BattleCore::new_from_scenario(
            BattleScenario::empty((4, 4)),
            GameDataBuilder::live_defaults().build_arc(),
            0x5303,
        );
        let BattleLiveCommandOutcome::UnitDeployed { unit_id } = core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft,
                position: Position::new(1, 1),
                tile_kind: TacticalTileKind::Ground,
                facing: FacingDirection::Right,
                instance_salt: 1,
                time_ms: 0,
                current_hp_policy: None,
            })
            .unwrap()
        else {
            panic!("expected deployed fixture");
        };
        core.set_engagement_stance(unit_id, EngagementStance::Defense, 1_500)
            .unwrap();
        {
            let unit = core.units.get_mut(&unit_id).unwrap();
            unit.stats.current_health = 41;
            unit.resonance_current = 37;
            unit.next_basic_attack_ms = 8_888;
            unit.ranged_reposition_until_ms = 9_999;
        }
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1_500,
                caster_instance_id: unit_id,
                target_instance_id: unit_id,
                buff_id: crate::game::battle::buffs::BuffId::from_name("silence"),
                duration_ms: 5_000,
                cause: BattleEventCause::default(),
            },
            1_500,
        )
        .unwrap();
        assert!(core.unit_control_state(unit_id, 1_500).unwrap().silenced);

        core.tactically_relocate_unit(
            unit_id,
            Position::new(2, 2),
            TacticalTileKind::Platform,
            FacingDirection::Up,
            1_500,
        )
        .unwrap();

        let unit = &core.units[&unit_id];
        assert_eq!(unit.instance_id, unit_id);
        assert_eq!(unit.body.projected_tile(), Position::new(2, 2));
        assert_eq!(
            unit.body.position,
            WorldVec2::from_tile_center(Position::new(2, 2))
        );
        assert_eq!(unit.facing_direction, Some(FacingDirection::Up));
        assert_eq!(unit.stats.current_health, 41);
        assert_eq!(unit.resonance_current, 37);
        assert_eq!(unit.next_basic_attack_ms, 8_888);
        assert_eq!(unit.ranged_reposition_until_ms, 9_999);
        assert!(core.unit_control_state(unit_id, 1_500).unwrap().silenced);
        assert_eq!(unit.block_capacity, 0);
        assert_eq!(unit.incoming_damage_modifiers.damage_reduction_percent, 0);
        let platform = core.unit_tactical_snapshot(unit_id, 1_500).unwrap();
        assert_eq!(platform.stance, EngagementStance::Defense);
        assert_eq!(platform.tile_kind, TacticalTileKind::Platform);
        assert_eq!(platform.tile_effect, TacticalTileEffect::Rest);
        assert_eq!(platform.stance_cooldown_until_ms, 3_000);
        assert_eq!(platform.relocation_cooldown_until_ms, 11_500);
        assert_eq!(platform.next_rest_tick_ms, Some(2_500));
        assert_eq!(
            platform.relocation_disabled_reason,
            Some(ActionDisabledReasonCode::RelocationCooldown)
        );
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            entry.event,
            BattleLogEvent::UnitTacticallyRelocated {
                unit_instance_id,
                from_position,
                to_position,
                before_facing: FacingDirection::Right,
                after_facing: FacingDirection::Up,
                cooldown_until_ms: 11_500,
                ..
            } if unit_instance_id == unit_id
                && from_position == Position::new(1, 1)
                && to_position == Position::new(2, 2)
        )));

        let log_count = core.event_log.entries.len();
        assert!(matches!(
            core.tactically_relocate_unit(
                unit_id,
                Position::new(3, 2),
                TacticalTileKind::Platform,
                FacingDirection::Left,
                11_499,
            ),
            Err(GameError::ActionUnavailable {
                action: ActionKind::TacticalRelocateUnit,
                reason: ActionDisabledReasonCode::RelocationCooldown,
            })
        ));
        assert_eq!(core.event_log.entries.len(), log_count);
        assert_eq!(core.unit_projected_tile(unit_id), Some(Position::new(2, 2)));

        core.tactically_relocate_unit(
            unit_id,
            Position::new(3, 2),
            TacticalTileKind::Ground,
            FacingDirection::Left,
            11_500,
        )
        .unwrap();
        let ground = core.unit_tactical_snapshot(unit_id, 11_500).unwrap();
        assert_eq!(ground.tile_effect, TacticalTileEffect::DefenseFocus);
        assert_eq!(ground.effective_block_capacity, 5);
        assert_eq!(ground.defense_focus_damage_reduction_percent, 10);
        assert_eq!(ground.next_rest_tick_ms, None);
        assert_eq!(
            core.units[&unit_id]
                .incoming_damage_modifiers
                .damage_reduction_percent,
            10
        );
    }

    #[test]
    fn tactical_relocation_gates_action_and_allied_occupancy_without_partial_mutation() {
        let (mut core, unit_id) = deployed_core(TacticalTileKind::Ground);
        assert_eq!(
            core.tactical_relocation_disabled_reason(unit_id, 1_500),
            None,
            "Attack stance permits tactical relocation"
        );

        let groups_before_occupied_deploy = core.scenario.groups.len();
        let log_before_occupied_deploy = core.event_log.entries.len();
        assert!(matches!(
            core.apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft: fixture_draft(0x5401, weapon_profile(TacticalTileKind::Ground)),
                position: Position::new(1, 1),
                tile_kind: TacticalTileKind::Ground,
                facing: FacingDirection::Left,
                instance_salt: 2,
                time_ms: 0,
                current_hp_policy: None,
            }),
            Err(GameError::PositionOccupied)
        ));
        assert_eq!(core.scenario.groups.len(), groups_before_occupied_deploy);
        assert_eq!(core.event_log.entries.len(), log_before_occupied_deploy);

        let BattleLiveCommandOutcome::UnitDeployed { unit_id: other_id } = core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft: fixture_draft(0x5401, weapon_profile(TacticalTileKind::Ground)),
                position: Position::new(2, 1),
                tile_kind: TacticalTileKind::Ground,
                facing: FacingDirection::Left,
                instance_salt: 2,
                time_ms: 0,
                current_hp_policy: None,
            })
            .unwrap()
        else {
            panic!("expected second fixture deploy");
        };
        core.set_engagement_stance(unit_id, EngagementStance::Defense, 1_500)
            .unwrap();

        let before_noop = core.event_log.entries.len();
        assert!(matches!(
            core.tactically_relocate_unit(
                unit_id,
                Position::new(1, 1),
                TacticalTileKind::Ground,
                FacingDirection::Right,
                1_500,
            ),
            Err(GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::AlreadyActive,
                ..
            })
        ));
        assert_eq!(core.event_log.entries.len(), before_noop);

        core.units
            .get_mut(&unit_id)
            .unwrap()
            .basic_attack_windup_until_ms = Some(1_600);
        assert_eq!(
            core.tactical_relocation_disabled_reason(unit_id, 1_500),
            Some(ActionDisabledReasonCode::ActionInProgress)
        );
        core.units
            .get_mut(&unit_id)
            .unwrap()
            .basic_attack_windup_until_ms = None;
        core.units.get_mut(&unit_id).unwrap().pending_skill_cast = Some(PendingSkillCast {
            start_gate_pending: false,
            reset_resonance: true,
            action: None,
            skill_id: crate::game::ability::SkillId::from("relocation_busy_fixture"),
            cast_target: None,
            accepted_target_tile: None,
            start_seq: 77,
        });
        assert_eq!(
            core.tactical_relocation_disabled_reason(unit_id, 1_500),
            Some(ActionDisabledReasonCode::ActionInProgress)
        );
        core.units.get_mut(&unit_id).unwrap().pending_skill_cast = None;

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1_500,
                caster_instance_id: unit_id,
                target_instance_id: unit_id,
                buff_id: crate::game::battle::buffs::BuffId::from_name("freeze"),
                duration_ms: 10,
                cause: BattleEventCause::default(),
            },
            1_500,
        )
        .unwrap();
        assert_eq!(
            core.tactical_relocation_disabled_reason(unit_id, 1_500),
            Some(ActionDisabledReasonCode::HardCrowdControl)
        );
        core.process_event(
            BattleEvent::BuffExpire {
                time_ms: 1_510,
                caster_instance_id: unit_id,
                target_instance_id: unit_id,
                buff_id: crate::game::battle::buffs::BuffId::from_name("freeze"),
                cause: BattleEventCause::default(),
            },
            1_510,
        )
        .unwrap();

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1_510,
                caster_instance_id: unit_id,
                target_instance_id: unit_id,
                buff_id: crate::game::battle::buffs::BuffId::from_name("silence"),
                duration_ms: 10,
                cause: BattleEventCause::default(),
            },
            1_510,
        )
        .unwrap();
        assert_eq!(
            core.tactical_relocation_disabled_reason(unit_id, 1_510),
            None,
            "Silence must not block tactical relocation"
        );
        core.process_event(
            BattleEvent::BuffExpire {
                time_ms: 1_520,
                caster_instance_id: unit_id,
                target_instance_id: unit_id,
                buff_id: crate::game::battle::buffs::BuffId::from_name("silence"),
                cause: BattleEventCause::default(),
            },
            1_520,
        )
        .unwrap();

        let before_invalid_destination = core.event_log.entries.len();
        assert!(matches!(
            core.tactically_relocate_unit(
                unit_id,
                Position::new(9, 9),
                TacticalTileKind::Ground,
                FacingDirection::Left,
                1_520,
            ),
            Err(GameError::OutOfBounds)
        ));
        core.battlefield
            .add_static_obstacle(Position::new(3, 3))
            .unwrap();
        assert!(matches!(
            core.tactically_relocate_unit(
                unit_id,
                Position::new(3, 3),
                TacticalTileKind::Ground,
                FacingDirection::Left,
                1_520,
            ),
            Err(GameError::StaticObstacleBlocked)
        ));
        core.battlefield.remove_static_obstacle(Position::new(3, 3));
        assert!(matches!(
            core.tactically_relocate_unit(
                unit_id,
                Position::new(3, 3),
                TacticalTileKind::Platform,
                FacingDirection::Left,
                1_520,
            ),
            Err(GameError::InvalidAction)
        ));
        assert_eq!(core.event_log.entries.len(), before_invalid_destination);
        assert_eq!(core.unit_projected_tile(unit_id), Some(Position::new(1, 1)));

        core.units.get_mut(&other_id).unwrap().role = BattleUnitRole::DefenseObject;
        let before_occupied = core.event_log.entries.len();
        assert!(matches!(
            core.tactically_relocate_unit(
                unit_id,
                Position::new(2, 1),
                TacticalTileKind::Ground,
                FacingDirection::Left,
                1_520,
            ),
            Err(GameError::PositionOccupied)
        ));
        assert_eq!(core.unit_projected_tile(unit_id), Some(Position::new(1, 1)));
        assert_eq!(core.event_log.entries.len(), before_occupied);

        core.units.get_mut(&other_id).unwrap().owner = Side::Opponent;
        core.tactical_states.remove(&other_id);
        core.tactically_relocate_unit(
            unit_id,
            Position::new(2, 1),
            TacticalTileKind::Ground,
            FacingDirection::Left,
            1_520,
        )
        .expect("enemy overlap does not occupy a player tactical slot");
        assert_eq!(core.unit_projected_tile(unit_id), Some(Position::new(2, 1)));
        assert_eq!(
            core.unit_projected_tile(other_id),
            Some(Position::new(2, 1))
        );
    }

    #[test]
    fn same_tile_facing_only_relocation_keeps_tile_effect_cadence() {
        let (mut core, unit_id) = deployed_core(TacticalTileKind::Platform);
        core.set_engagement_stance(unit_id, EngagementStance::Defense, 1_500)
            .unwrap();
        let before = core.unit_tactical_snapshot(unit_id, 1_500).unwrap();
        core.tactically_relocate_unit(
            unit_id,
            Position::new(1, 1),
            TacticalTileKind::Platform,
            FacingDirection::Left,
            1_500,
        )
        .unwrap();
        let after = core.unit_tactical_snapshot(unit_id, 1_500).unwrap();
        assert_eq!(after.tile_effect, TacticalTileEffect::Rest);
        assert_eq!(after.next_rest_tick_ms, before.next_rest_tick_ms);
        assert_eq!(after.relocation_cooldown_until_ms, 11_500);
        assert_eq!(
            core.units[&unit_id].facing_direction,
            Some(FacingDirection::Left)
        );
    }

    #[test]
    fn tactical_swap_is_one_batch_and_preserves_each_units_runtime_state() {
        let mut core = BattleCore::new_from_scenario(
            BattleScenario::empty((4, 4)),
            GameDataBuilder::live_defaults().build_arc(),
            0x5703,
        );
        let deploy =
            |core: &mut BattleCore, owned_uuid, position, tile_kind, facing, instance_salt| {
                let outcome = core
                    .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                        draft: fixture_draft(owned_uuid, any_tile_weapon_profile()),
                        position,
                        tile_kind,
                        facing,
                        instance_salt,
                        time_ms: 0,
                        current_hp_policy: None,
                    })
                    .unwrap();
                let BattleLiveCommandOutcome::UnitDeployed { unit_id } = outcome else {
                    panic!("unexpected fixture outcome: {outcome:?}");
                };
                unit_id
            };
        let first_id = deploy(
            &mut core,
            0x5701,
            Position::new(1, 1),
            TacticalTileKind::Ground,
            FacingDirection::Right,
            1,
        );
        let second_id = deploy(
            &mut core,
            0x5801,
            Position::new(2, 1),
            TacticalTileKind::Platform,
            FacingDirection::Left,
            2,
        );
        core.set_engagement_stance(first_id, EngagementStance::Defense, 1_500)
            .unwrap();
        core.set_engagement_stance(second_id, EngagementStance::Defense, 1_500)
            .unwrap();
        {
            let first = core.units.get_mut(&first_id).unwrap();
            first.stats.current_health = 41;
            first.resonance_current = 37;
            first.next_basic_attack_ms = 8_888;
        }
        {
            let second = core.units.get_mut(&second_id).unwrap();
            second.stats.current_health = 53;
            second.resonance_current = 29;
            second.next_basic_attack_ms = 9_999;
        }
        let event_count = core.event_log.entries.len();

        let outcome = core
            .apply_live_command(BattleLiveCommand::TacticalSwapUnits {
                first_unit_id: first_id,
                second_unit_id: second_id,
                first_destination_tile_kind: TacticalTileKind::Platform,
                second_destination_tile_kind: TacticalTileKind::Ground,
                first_facing: FacingDirection::Up,
                second_facing: FacingDirection::Down,
                time_ms: 1_500,
            })
            .unwrap();
        assert_eq!(
            outcome,
            BattleLiveCommandOutcome::UnitsTacticallySwapped {
                first_unit_id: first_id,
                second_unit_id: second_id,
            }
        );
        let first = &core.units[&first_id];
        let second = &core.units[&second_id];
        assert_eq!(first.body.projected_tile(), Position::new(2, 1));
        assert_eq!(second.body.projected_tile(), Position::new(1, 1));
        assert_eq!(first.facing_direction, Some(FacingDirection::Up));
        assert_eq!(second.facing_direction, Some(FacingDirection::Down));
        assert_eq!(
            (first.stats.current_health, first.resonance_current),
            (41, 37)
        );
        assert_eq!(
            (second.stats.current_health, second.resonance_current),
            (53, 29)
        );
        assert_eq!(first.next_basic_attack_ms, 8_888);
        assert_eq!(second.next_basic_attack_ms, 9_999);
        let first_tactical = core.unit_tactical_snapshot(first_id, 1_500).unwrap();
        let second_tactical = core.unit_tactical_snapshot(second_id, 1_500).unwrap();
        assert_eq!(first_tactical.tile_effect, TacticalTileEffect::Rest);
        assert_eq!(first_tactical.next_rest_tick_ms, Some(2_500));
        assert_eq!(
            second_tactical.tile_effect,
            TacticalTileEffect::DefenseFocus
        );
        assert_eq!(second_tactical.next_rest_tick_ms, None);
        assert_eq!(first_tactical.relocation_cooldown_until_ms, 11_500);
        assert_eq!(second_tactical.relocation_cooldown_until_ms, 11_500);
        assert_eq!(core.event_log.entries.len(), event_count + 1);
        assert!(matches!(
            core.event_log.entries.last().map(|entry| &entry.event),
            Some(BattleLogEvent::UnitsTacticallySwapped {
                first_unit_instance_id,
                second_unit_instance_id,
                first_from_position,
                first_to_position,
                second_from_position,
                second_to_position,
                first_after_facing: FacingDirection::Up,
                second_after_facing: FacingDirection::Down,
                cooldown_until_ms: 11_500,
                ..
            }) if *first_unit_instance_id == first_id
                && *second_unit_instance_id == second_id
                && *first_from_position == Position::new(1, 1)
                && *first_to_position == Position::new(2, 1)
                && *second_from_position == Position::new(2, 1)
                && *second_to_position == Position::new(1, 1)
        ));
        let post_swap_event_count = core.event_log.entries.len();
        assert!(matches!(
            core.validate_tactical_swap(
                first_id,
                second_id,
                TacticalTileKind::Ground,
                TacticalTileKind::Platform,
                11_499,
            ),
            Err(GameError::ActionUnavailable {
                action: ActionKind::TacticalSwapUnits,
                reason: ActionDisabledReasonCode::RelocationCooldown,
            })
        ));
        assert_eq!(core.event_log.entries.len(), post_swap_event_count);
    }

    #[test]
    fn tactical_swap_preflights_both_units_and_third_occupancy_without_mutation() {
        let mut core = BattleCore::new_from_scenario(
            BattleScenario::empty((5, 5)),
            GameDataBuilder::live_defaults().build_arc(),
            0x5903,
        );
        let deploy = |core: &mut BattleCore, owned_uuid, position, instance_salt| {
            let BattleLiveCommandOutcome::UnitDeployed { unit_id } = core
                .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                    draft: fixture_draft(owned_uuid, any_tile_weapon_profile()),
                    position,
                    tile_kind: TacticalTileKind::Ground,
                    facing: FacingDirection::Right,
                    instance_salt,
                    time_ms: 0,
                    current_hp_policy: None,
                })
                .unwrap()
            else {
                panic!("expected fixture deployment");
            };
            unit_id
        };
        let first_id = deploy(&mut core, 0x5901, Position::new(1, 1), 1);
        let second_id = deploy(&mut core, 0x5a01, Position::new(2, 1), 2);
        let third_id = deploy(&mut core, 0x5b01, Position::new(3, 1), 3);
        core.set_engagement_stance(first_id, EngagementStance::Defense, 1_500)
            .unwrap();
        core.validate_tactical_swap(
            first_id,
            second_id,
            TacticalTileKind::Ground,
            TacticalTileKind::Ground,
            1_500,
        )
        .expect("both Attack and Defense participants can swap");
        core.set_engagement_stance(second_id, EngagementStance::Defense, 1_500)
            .unwrap();

        let before = (
            core.unit_projected_tile(first_id),
            core.unit_projected_tile(second_id),
            core.units[&first_id].facing_direction,
            core.units[&second_id].facing_direction,
        );
        assert!(matches!(
            core.tactically_swap_units(
                first_id,
                first_id,
                TacticalTileKind::Ground,
                TacticalTileKind::Ground,
                FacingDirection::Left,
                FacingDirection::Left,
                1_500,
            ),
            Err(GameError::ActionUnavailable {
                action: ActionKind::TacticalSwapUnits,
                reason: ActionDisabledReasonCode::AlreadyActive,
            })
        ));

        core.units.get_mut(&second_id).unwrap().pending_skill_cast = Some(PendingSkillCast {
            start_gate_pending: false,
            reset_resonance: true,
            action: None,
            skill_id: crate::game::ability::SkillId::from("swap_busy_fixture"),
            cast_target: None,
            accepted_target_tile: None,
            start_seq: 88,
        });
        assert!(matches!(
            core.validate_tactical_swap(
                first_id,
                second_id,
                TacticalTileKind::Ground,
                TacticalTileKind::Ground,
                1_500,
            ),
            Err(GameError::ActionUnavailable {
                action: ActionKind::TacticalSwapUnits,
                reason: ActionDisabledReasonCode::ActionInProgress,
            })
        ));
        core.units.get_mut(&second_id).unwrap().pending_skill_cast = None;

        core.active_movement_segments.insert(
            second_id,
            ActiveMovementSegment {
                start: WorldVec2::from_tile_center(Position::new(2, 1)),
                target: WorldVec2::from_tile_center(Position::new(2, 2)),
                started_at_ms: 1_500,
                ends_at_ms: 2_500,
            },
        );
        assert!(matches!(
            core.validate_tactical_swap(
                first_id,
                second_id,
                TacticalTileKind::Ground,
                TacticalTileKind::Ground,
                1_500,
            ),
            Err(GameError::ActionUnavailable {
                action: ActionKind::TacticalSwapUnits,
                reason: ActionDisabledReasonCode::ActionInProgress,
            })
        ));
        core.active_movement_segments.remove(&second_id);

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1_500,
                caster_instance_id: first_id,
                target_instance_id: second_id,
                buff_id: crate::game::battle::buffs::BuffId::from_name("freeze"),
                duration_ms: 10,
                cause: BattleEventCause::default(),
            },
            1_500,
        )
        .unwrap();
        assert!(matches!(
            core.validate_tactical_swap(
                first_id,
                second_id,
                TacticalTileKind::Ground,
                TacticalTileKind::Ground,
                1_500,
            ),
            Err(GameError::ActionUnavailable {
                action: ActionKind::TacticalSwapUnits,
                reason: ActionDisabledReasonCode::HardCrowdControl,
            })
        ));
        core.process_event(
            BattleEvent::BuffExpire {
                time_ms: 1_510,
                caster_instance_id: first_id,
                target_instance_id: second_id,
                buff_id: crate::game::battle::buffs::BuffId::from_name("freeze"),
                cause: BattleEventCause::default(),
            },
            1_510,
        )
        .unwrap();

        core.battlefield
            .add_static_obstacle(Position::new(2, 1))
            .unwrap();
        assert!(matches!(
            core.validate_tactical_swap(
                first_id,
                second_id,
                TacticalTileKind::Ground,
                TacticalTileKind::Ground,
                1_510,
            ),
            Err(GameError::StaticObstacleBlocked)
        ));
        core.battlefield.remove_static_obstacle(Position::new(2, 1));

        let occupied_world = WorldVec2::from_tile_center(Position::new(2, 1));
        core.units.get_mut(&third_id).unwrap().body.position = occupied_world;
        core.units.get_mut(&third_id).unwrap().role = BattleUnitRole::DefenseObject;
        let before_occupied_event_count = core.event_log.entries.len();
        assert!(matches!(
            core.tactically_swap_units(
                first_id,
                second_id,
                TacticalTileKind::Ground,
                TacticalTileKind::Ground,
                FacingDirection::Left,
                FacingDirection::Up,
                1_510,
            ),
            Err(GameError::PositionOccupied)
        ));
        assert_eq!(core.unit_projected_tile(first_id), before.0);
        assert_eq!(core.unit_projected_tile(second_id), before.1);
        assert_eq!(core.units[&first_id].facing_direction, before.2);
        assert_eq!(core.units[&second_id].facing_direction, before.3);
        assert_eq!(core.event_log.entries.len(), before_occupied_event_count);

        let third_world = WorldVec2::from_tile_center(Position::new(3, 1));
        core.units.get_mut(&third_id).unwrap().body.position = third_world;
        core.tactically_swap_units(
            first_id,
            second_id,
            TacticalTileKind::Ground,
            TacticalTileKind::Ground,
            FacingDirection::Left,
            FacingDirection::Up,
            1_510,
        )
        .expect("the participant pair itself must be excluded from occupancy");
    }

    #[test]
    fn tactical_relocation_releases_and_deterministically_reacquires_engagement() {
        let (mut core, blocker_id) = deployed_core(TacticalTileKind::Ground);
        let deploy_enemy_fixture = |core: &mut BattleCore, owned_uuid, position, instance_salt| {
            let outcome = core
                .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                    draft: fixture_draft(owned_uuid, weapon_profile(TacticalTileKind::Ground)),
                    position,
                    tile_kind: TacticalTileKind::Ground,
                    facing: FacingDirection::Left,
                    instance_salt,
                    time_ms: 0,
                    current_hp_policy: None,
                })
                .unwrap();
            let BattleLiveCommandOutcome::UnitDeployed { unit_id } = outcome else {
                panic!("unexpected fixture outcome: {outcome:?}");
            };
            core.units.get_mut(&unit_id).unwrap().owner = Side::Opponent;
            core.tactical_states.remove(&unit_id);
            unit_id
        };
        let old_enemy_id = deploy_enemy_fixture(&mut core, 0x5501, Position::new(1, 2), 2);
        let new_enemy_id = deploy_enemy_fixture(&mut core, 0x5601, Position::new(3, 2), 3);
        core.units
            .get_mut(&old_enemy_id)
            .unwrap()
            .set_world_position(WorldVec2::from_tile_center(Position::new(1, 1)));
        core.set_engagement_stance(blocker_id, EngagementStance::Defense, 1_500)
            .unwrap();
        core.refresh_block_state();
        assert_eq!(core.blocked_by(old_enemy_id), Some(blocker_id));
        assert_eq!(core.blocked_by(new_enemy_id), None);

        core.tactically_relocate_unit(
            blocker_id,
            Position::new(3, 2),
            TacticalTileKind::Ground,
            FacingDirection::Left,
            1_500,
        )
        .unwrap();
        assert_eq!(core.blocked_by(old_enemy_id), None);
        assert_eq!(core.blocked_by(new_enemy_id), Some(blocker_id));
        assert_eq!(
            core.units[&new_enemy_id].current_target,
            Some(blocker_id),
            "the newly engaged enemy must target the relocated blocker"
        );
    }
}
