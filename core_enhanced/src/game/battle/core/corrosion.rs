use super::BattleCore;
use crate::game::{
    ability::SkillId,
    battle::{
        ids::UnitInstanceId,
        runtime_facts::{ActionHandle, ActionTerminalKind, DeathCause, RuntimeFactSource},
        types::BattleUnitSourceIdentity,
    },
    behavior::{ActionDisabledReasonCode, ActionKind, GameError},
    data::skill_fragment_data::FragmentCorrosionClass,
    enums::Side,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct CorrosionFreeze {
    pub remaining_ms: u64,
    pub started_at_battle_ms: u64,
    pub actor: UnitInstanceId,
}
crate::authoring::clone_shape::copy_shape!(CorrosionFreeze);

impl BattleCore {
    pub fn forced_pause_remaining_ms(&self) -> Option<u64> {
        self.corrosion_freeze
            .map(|freeze| freeze.remaining_ms)
            .into_iter()
            .chain(self.sustained.tile_bursts.forced_pause_remaining())
            .max()
    }
    pub fn forced_pause_reason(&self) -> ActionDisabledReasonCode {
        if self.corrosion_freeze.is_some() {
            ActionDisabledReasonCode::CorrosionForcedPause
        } else {
            ActionDisabledReasonCode::SkillPresentationPause
        }
    }
    pub(super) fn ensure_not_forced_paused(&self) -> Result<(), GameError> {
        if self.forced_pause_remaining_ms().is_some() {
            return Err(GameError::ActionUnavailable {
                action: ActionKind::ActivateSkill,
                reason: self.forced_pause_reason(),
            });
        }
        Ok(())
    }
    /// Host time, deliberately independent from battle time and playback speed.
    /// The entire excess delta is discarded on the expiry call.
    pub fn advance_forced_pause_host_elapsed(
        &mut self,
        elapsed_ms: u64,
    ) -> Result<bool, GameError> {
        if self.forced_pause_remaining_ms().is_none() {
            return Ok(false);
        }
        self.runtime_transaction(|core| {
            core.advance_tile_burst_host(elapsed_ms)?;
            let Some(freeze) = core.corrosion_freeze.as_mut() else {
                return Ok(true);
            };
            freeze.remaining_ms = freeze.remaining_ms.saturating_sub(elapsed_ms);
            if freeze.remaining_ms == 0 {
                let ended = *freeze;
                core.corrosion_freeze = None;
                core.record_event_log(
                    ended.started_at_battle_ms,
                    crate::game::battle::event_log::BattleLogEvent::CorrosionEnded {
                        actor: ended.actor,
                    },
                );
            }
            Ok(true)
        })
    }
    fn corrosion_fragment_is_excluded(
        &self,
        fragment: &crate::game::data::skill_fragment_data::SkillFragmentMetadata,
    ) -> bool {
        use crate::game::data::{
            pve_data::PveEncounterClass, skill_fragment_data::SkillFragmentOrigin,
        };
        if fragment.corrosion_class == FragmentCorrosionClass::ExcludedFinalBoss {
            return true;
        }
        if crate::authoring::control::charge(
            self.game_data.pve_data.encounters.len() as u64,
            "corrosion_final_boss_classification",
        )
        .is_err()
        {
            return true;
        }
        self.game_data.pve_data.encounters.iter().filter(|encounter| matches!(encounter.encounter_class, PveEncounterClass::FinalBoss | PveEncounterClass::ReferenceFinalBoss)).any(|encounter| {
            encounter.primary_abnormality_id.as_deref().is_some_and(|id| {
                matches!(&fragment.origin, Some(SkillFragmentOrigin::Abnormality { abnormality_id }) if abnormality_id == id)
                    || self.game_data.abnormality_data.get_by_id(id).is_some_and(|abnormality| abnormality.response_complete_skill_fragment_id.as_ref() == Some(&fragment.id))
            })
        })
    }
    pub fn corrosion_ready(&self, actor: UnitInstanceId, skill: &SkillId, time_ms: u64) -> bool {
        self.corrosion_eligible(actor, skill, time_ms, false)
    }
    fn corrosion_eligible(
        &self,
        actor: UnitInstanceId,
        skill: &SkillId,
        time_ms: u64,
        admitted: bool,
    ) -> bool {
        let Some(unit) = self.units.get(&actor) else {
            return false;
        };
        let Some(source) = unit
            .fragment_source
            .as_ref()
            .filter(|source| self.fragment_owns_skill(&source.skill_id, skill))
        else {
            return false;
        };
        self.forced_pause_remaining_ms().is_none() && unit.owner == Side::Player && unit.is_active() && !unit.corroded
            && matches!(unit.source_identity, BattleUnitSourceIdentity::Employee { .. })
            && unit.employee_trauma.is_some_and(|trauma| trauma >= self.game_data.run_policy.corrosion.trauma_threshold && trauma < 100)
            && (admitted || (unit.resonance_current >= unit.resonance_max.max(1)
                && self.current_skill_view(skill, actor, &Default::default()).is_some_and(|definition|
                    self.resonance_admission_payment(actor, definition) == Some(unit.resonance_current))))
            && (if admitted { self.active_hard_cc_release_time_ms(actor, time_ms).is_none() }
                else { self.skill_start_disabled_reason(actor, time_ms).is_none() })
            && self.game_data.skill_fragment_data.get_by_id(&source.definition_id).is_some_and(|fragment| !self.corrosion_fragment_is_excluded(fragment) && matches!(&fragment.effect, crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill { forms, .. } if forms.get(&source.form_id).is_some_and(|form| form.skill_id == source.skill_id)))
    }
    pub(super) fn apply_corrosion_percent_damage(
        &mut self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        use crate::game::battle::event_log::{BattleLogEvent, HpChangeReason};
        if !self.single_target_can_target_unit(target, true)
            || self.sustained_invulnerable(target, now)?
        {
            return Ok(());
        }
        let before = self.units[&target].stats.current_health;
        let maximum = self.units[&target].stats.max_health;
        // Round upward once, independently of armor and ordinary amplification.
        let damage = (u64::from(maximum)
            * u64::from(
                self.game_data
                    .run_policy
                    .corrosion
                    .boss_max_health_damage_percent,
            )
            / 100) as u32;
        let protection = self.absorb_sustained_shields(target, damage, now)?;
        let floor = self.boss_runtime.as_ref().map_or(0, |runtime| {
            runtime.corrosion_health_floor(target, before, maximum)
        });
        let after = before
            .saturating_sub(
                damage
                    .saturating_sub(protection.absorbed)
                    .saturating_sub(protection.prevented),
            )
            .max(floor);
        let prepared = if after == 0 {
            self.prepare_death_fact(Some(actor), target, now)?
        } else {
            None
        };
        let reservation = self.reserve_authoring_observation()?;
        self.units.get_mut(&target).unwrap().stats.current_health = after;
        self.commit_authoring_observation(
            reservation,
            now,
            Some(actor),
            target,
            super::authoring_observation::AppliedChange::Hp {
                before,
                after,
                reason: HpChangeReason::Corrosion,
            },
        );
        self.record_event_log(
            now,
            BattleLogEvent::HpChanged {
                source_instance_id: Some(actor),
                target_instance_id: target,
                delta: -(before.saturating_sub(after).min(i32::MAX as u32) as i32),
                hp_before: before,
                hp_after: after,
                reason: HpChangeReason::Corrosion,
                damage_source: None,
                damage_type: None,
                raw_damage: Some(damage),
                final_damage: Some(before - after),
                damage_breakdown: None,
                critical: Some(false),
                feedback_tags: vec![],
            },
        );
        if let Some(mut prepared) = prepared {
            prepared.cause = DeathCause::Corrosion;
            self.finalize_prepared_death_with_dispatch(
                Some(actor),
                target,
                Side::Opponent,
                now,
                prepared,
                None,
                false,
            );
        }
        crate::authoring::control::poll("corrosion_percent_damage_complete")?;
        crate::game::battle::runtime_control::poll()
    }

    pub(super) fn try_start_corrosion(
        &mut self,
        actor: UnitInstanceId,
        skill: &SkillId,
        action: &ActionHandle,
        time_ms: u64,
    ) -> Result<bool, GameError> {
        if !self.has_resonance_use_credential(actor, skill, action)
            || !self
                .units
                .get(&actor)
                .and_then(|unit| unit.resonance_use.as_ref())
                .is_some_and(|use_state| use_state.full_payment)
            || !self.corrosion_eligible(actor, skill, time_ms, true)
        {
            return Ok(false);
        }
        self.ensure_not_forced_paused()?;
        let count = self
            .units
            .values()
            .filter(|unit| unit.owner == Side::Opponent && unit.is_active())
            .count();
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            count.checked_mul(std::mem::size_of::<UnitInstanceId>()),
            "corrosion_enemy_capture",
        )?;
        crate::authoring::control::charge(self.units.len() as u64, "corrosion_enemy_capture")?;
        let mut enemies = Vec::with_capacity(count);
        enemies.extend(
            self.units
                .iter()
                .filter(|(_, unit)| unit.owner == Side::Opponent && unit.is_active())
                .map(|(id, _)| *id),
        );
        enemies.sort_by_key(|id| (self.units[id].spawn_order, *id));
        crate::authoring::control::poll("corrosion_capture_complete")?;
        let source = RuntimeFactSource {
            actor: Some(actor),
            action: Some(action.clone()),
            fragment: self.units[&actor].fragment_source.clone(),
            ..Default::default()
        };
        let source = if self.runtime_facts.causal.is_some() {
            let source = source.inherit_causal_context(&self.runtime_fact_source)?;
            self.causal_root_source(&source, true)?
        } else {
            source
        };
        let previous_source = std::mem::replace(&mut self.runtime_fact_source, source.clone());
        // Store the employee obligation before any death can make the battle terminal.
        self.units.get_mut(&actor).unwrap().corroded = true;
        self.units.get_mut(&actor).unwrap().employee_trauma = Some(100);
        self.units.get_mut(&actor).unwrap().pending_skill_cast = None;
        self.withdraw_unit(actor, time_ms)?;
        crate::authoring::control::poll("corrosion_employee_cost_complete")?;
        for enemy in enemies {
            use crate::game::battle::types::BattleUnitThreatClass;
            let unit = &self.units[&enemy];
            let ordinary = unit.threat_class == BattleUnitThreatClass::Normal
                && matches!(
                    unit.source_identity,
                    BattleUnitSourceIdentity::CorrodedEmployee { .. }
                        | BattleUnitSourceIdentity::SummonedUnit { .. }
                );
            if !ordinary {
                if unit.threat_class != BattleUnitThreatClass::Normal
                    || matches!(
                        unit.source_identity,
                        BattleUnitSourceIdentity::Abnormality { .. }
                    )
                {
                    self.apply_corrosion_percent_damage(actor, enemy, time_ms)?;
                }
                continue;
            }
            let Some(mut prepared) = self.prepare_death_fact(Some(actor), enemy, time_ms)? else {
                continue;
            };
            prepared.cause = DeathCause::Corrosion;
            self.finalize_prepared_death_with_dispatch(
                Some(actor),
                enemy,
                Side::Opponent,
                time_ms,
                prepared,
                None,
                false,
            );
            crate::authoring::control::poll("corrosion_special_death_complete")?;
        }
        self.runtime_facts.release(action, 0, time_ms)?;
        self.runtime_facts
            .terminal(action, ActionTerminalKind::Finished, time_ms)?;
        self.corrosion_freeze = Some(CorrosionFreeze {
            remaining_ms: self.game_data.run_policy.corrosion.presentation_duration_ms,
            started_at_battle_ms: time_ms,
            actor,
        });
        self.record_event_log(
            time_ms,
            crate::game::battle::event_log::BattleLogEvent::CorrosionStarted {
                actor,
                duration_ms: self.game_data.run_policy.corrosion.presentation_duration_ms,
            },
        );
        self.event_queue
            .push(crate::game::battle::enums::BattleEvent::CorrosionBoundary { time_ms });
        self.runtime_fact_source = previous_source;
        crate::authoring::control::poll("corrosion_presentation_admitted")?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        battle::{
            core::{
                sim::{BattleExecutionState, BattleStepOutcome},
                tests::native_action_fixture,
            },
            event_log::BattleLogEvent,
            runtime_facts::ActionQueryError,
        },
        data::{
            skill_data::SkillDatabase, skill_fragment_data::SkillFragmentDatabase, GameDataBuilder,
        },
        skill_fragment::FragmentSourceSnapshot,
    };
    const SKILLS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/r08/skills.ron"
    ));
    const FRAGMENTS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/r09/fragments.ron"
    ));
    fn fixture(
        trauma: u32,
        excluded: bool,
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let skills = SkillDatabase::from_ron("r09/skills.ron", SKILLS).unwrap();
        let skill = skills.get_by_id("r08_strike").unwrap().clone();
        let (mut core, state, actor, enemy, _) = native_action_fixture(skill, Default::default());
        let fragments: SkillFragmentDatabase = ron::de::from_str(&FRAGMENTS.replace(
            "corrosion_class: Ordinary",
            if excluded {
                "corrosion_class: ExcludedFinalBoss"
            } else {
                "corrosion_class: Ordinary"
            },
        ))
        .unwrap();
        core.game_data = GameDataBuilder::empty()
            .with_skills(skills)
            .with_skill_fragments(fragments)
            .build_arc();
        let unit = core.units.get_mut(&actor).unwrap();
        unit.source_identity = BattleUnitSourceIdentity::Employee {
            employee_uuid: actor.as_uuid(),
            base_uuid: unit.base_uuid,
        };
        unit.employee_trauma = Some(trauma);
        unit.fragment_source = Some(
            FragmentSourceSnapshot {
                copy_uuid: uuid::Uuid::from_u128(91),
                definition_id: "r09_fragment".into(),
                form_id: "base".into(),
                skill_id: "r08_strike".into(),
            }
            .into_shared()
            .unwrap(),
        );
        let target = core.units.get_mut(&enemy).unwrap();
        target.source_identity = BattleUnitSourceIdentity::CorrodedEmployee {
            profile_id: "r09_ordinary".into(),
            base_uuid: target.base_uuid,
        };
        target.threat_class = crate::game::battle::types::BattleUnitThreatClass::Normal;
        (core, state, actor, enemy)
    }
    #[test]
    fn corrosion_classifies_targets_and_percent_damage_bypasses_ordinary_reductions() {
        use crate::game::battle::types::BattleUnitThreatClass;
        for (threat, maximum, current, expected) in [
            (BattleUnitThreatClass::Normal, 1001, 1001, 0),
            (BattleUnitThreatClass::Elite, 1001, 1001, 750),
            (BattleUnitThreatClass::Boss, 1001, 1001, 750),
            (BattleUnitThreatClass::Boss, 1001, 250, 0),
        ] {
            let (mut core, mut state, actor, enemy) = fixture(80, false);
            let target = core.units.get_mut(&enemy).unwrap();
            target.threat_class = threat;
            target.stats.max_health = maximum;
            target.stats.current_health = current;
            core.request_skill_action(0, actor, "r08_strike".into(), None)
                .unwrap();
            let target = core.units.get_mut(&enemy).unwrap();
            target.stats.defense = 10000;
            target.incoming_damage_modifiers.damage_reduction_percent = 100;
            core.step_battle_execution_until(&mut state, 0).unwrap();
            assert_eq!(core.units[&enemy].stats.current_health, expected);
            assert_eq!(core.units[&actor].employee_trauma, Some(100));
            assert_eq!(core.runtime_fact_summary().resource_spent, 100);
            assert_eq!(
                core.runtime_fact_summary().damage_hp_lost,
                0,
                "special corrosion cannot become an ordinary damage-chain trigger"
            );
            if expected == 0 {
                assert_eq!(
                    core.runtime_fact_summary()
                        .latest_death
                        .as_ref()
                        .unwrap()
                        .cause,
                    DeathCause::Corrosion
                );
            }
        }
    }

    #[test]
    fn corrosion_percent_respects_shield_survival_and_existing_invulnerability() {
        use crate::game::ability::sustained::{SurvivalPolicy, SustainedEffectDef};
        use crate::game::battle::types::BattleUnitThreatClass;
        for (shield, survival, invulnerable, expected) in [
            (100, false, false, 849),
            (2000, false, false, 1000),
            (0, true, false, 1),
            (0, false, true, 1000),
        ] {
            let (mut core, mut state, actor, enemy) = fixture(80, false);
            let target = core.units.get_mut(&enemy).unwrap();
            target.threat_class = BattleUnitThreatClass::Boss;
            target.stats.max_health = 1001;
            target.stats.current_health = if survival { 100 } else { 1000 };
            if shield > 0 {
                core.begin_sustained(
                    enemy,
                    enemy,
                    SustainedEffectDef::Shield {
                        duration_ms: 1000,
                        amount: shield,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    0,
                )
                .unwrap();
            }
            if survival {
                core.begin_sustained(
                    enemy,
                    enemy,
                    SustainedEffectDef::Survival {
                        duration_ms: 1000,
                        policy: SurvivalPolicy::FloorOne,
                    },
                    0,
                )
                .unwrap();
            }
            if invulnerable {
                super::super::relic_causal_tests::prepare_existing_invulnerability(
                    &mut core, actor, enemy,
                );
            }
            core.request_skill_action(0, actor, "r08_strike".into(), None)
                .unwrap();
            core.step_battle_execution_until(&mut state, 0).unwrap();
            assert_eq!(core.units[&enemy].stats.current_health, expected);
            assert!(core.runtime_fact_summary().latest_death.is_none());
        }
    }

    #[test]
    fn r09_public_ron_start_replaces_cast_and_host_time_controls_freeze() {
        for trauma in [79, 80, 99] {
            for excluded in [false, true] {
                let (mut core, mut state, actor, enemy) = fixture(trauma, excluded);
                let expected = trauma >= 80 && !excluded;
                assert_eq!(
                    core.live_skill_readiness(0, actor).unwrap().corrosion_ready,
                    expected
                );
                let handle = core
                    .request_skill_action(0, actor, "r08_strike".into(), None)
                    .unwrap();
                assert!(
                    !core.units[&actor].corroded,
                    "admission is not actual execution"
                );
                let outcome = core.step_battle_execution_until(&mut state, 0).unwrap();
                assert!(matches!(outcome, BattleStepOutcome::Running));
                assert_eq!(core.units[&actor].corroded, expected);
                if expected {
                    assert_eq!(core.units[&actor].employee_trauma, Some(100));
                    assert!(!core.units[&actor].is_active());
                    assert!(!core.units[&enemy].is_active());
                    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 0);
                    assert_eq!(core.runtime_fact_summary().resource_spent, 100);
                    assert_eq!(core.forced_pause_remaining_ms(), Some(1000));
                    assert!(core.step_battle_execution_until(&mut state, 300).is_err());
                    assert!(matches!(
                        core.take_settled_action_result(&handle),
                        Err(ActionQueryError::ForcedPause)
                    ));
                    assert!(core.advance_forced_pause_host_elapsed(999).unwrap());
                    assert_eq!(core.forced_pause_remaining_ms(), Some(1));
                    assert!(core.advance_forced_pause_host_elapsed(5000).unwrap());
                    assert_eq!(core.forced_pause_remaining_ms(), None);
                    let action = core.take_settled_action_result(&handle).unwrap();
                    assert_eq!(action.action.released_at_ms, Some(0));
                    assert_eq!(action.action.facts.resource_spent, 100);
                    assert_eq!(action.action.facts.damage_hp_lost, 0);
                    assert_eq!(
                        core.event_log
                            .entries
                            .iter()
                            .filter(|e| matches!(e.event, BattleLogEvent::CorrosionStarted { .. }))
                            .count(),
                        1
                    );
                } else {
                    core.step_battle_execution_until(&mut state, 300).unwrap();
                    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 100);
                }
            }
        }
    }
    #[test]
    fn r09_start_gate_uses_admission_credential_and_ignores_stale_events() {
        let (mut core, mut state, actor, enemy) = fixture(80, false);
        core.request_skill_action(0, actor, "r08_strike".into(), None)
            .unwrap();
        assert_eq!(core.units[&actor].resonance_current, 0);
        let stale = core
            .event_queue
            .iter()
            .find(|event| {
                matches!(
                    event.as_ref(),
                    crate::game::battle::enums::BattleEvent::SkillStartGate { .. }
                )
            })
            .unwrap()
            .into_owned();
        // A subsequent resource mutation cannot revoke the accepted credential.
        core.units.get_mut(&actor).unwrap().resonance_current = 99;
        core.step_battle_execution_until(&mut state, 500).unwrap();
        assert!(core.units[&actor].corroded);
        assert!(core.units[&actor].pending_skill_cast.is_none());
        assert!(!core.units[&enemy].is_active());
        assert_eq!(core.runtime_fact_summary().resource_spent, 100);
        core.advance_forced_pause_host_elapsed(1000).unwrap();
        core.process_event(stale, 0).unwrap();
        assert_eq!(core.runtime_fact_summary().resource_spent, 100);
        assert_eq!(
            core.event_log
                .entries
                .iter()
                .filter(|entry| matches!(entry.event, BattleLogEvent::CorrosionStarted { .. }))
                .count(),
            1
        );
        let (mut core, _, actor, _) = fixture(80, false);
        core.units.get_mut(&actor).unwrap().fragment_source = None;
        assert!(!core.corrosion_ready(actor, &"r08_strike".into(), 0));
    }

    #[test]
    fn r09_corrosion_faults_capture_cost_death_presentation_roll_back_and_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in [
            "corrosion_capture_complete",
            "corrosion_employee_cost_complete",
            "corrosion_special_death_complete",
            "corrosion_percent_damage_complete",
            "corrosion_presentation_admitted",
        ] {
            let (mut core, mut state, actor, enemy) = fixture(80, false);
            if site == "corrosion_percent_damage_complete" {
                core.units.get_mut(&enemy).unwrap().threat_class =
                    crate::game::battle::types::BattleUnitThreatClass::Boss;
                core.begin_sustained(
                    enemy,
                    enemy,
                    crate::game::ability::sustained::SustainedEffectDef::Shield {
                        duration_ms: 1000,
                        amount: 10,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    0,
                )
                .unwrap();
            }
            core.request_skill_action(0, actor, "r08_strike".into(), None)
                .unwrap();
            let mut clean = core.clone();
            let mut clean_state = state.clone();
            let before = (
                core.transaction_debug_snapshot(),
                core.event_queue_debug_snapshot(),
                format!("{state:?}"),
            );
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || core.step_battle_execution_until(&mut state, 0),
                )
            });
            assert!(token.is_cancelled(), "did not reach {site}: {result:?}");
            assert!(result.is_err(), "{site}");
            assert_eq!(
                (
                    core.transaction_debug_snapshot(),
                    core.event_queue_debug_snapshot(),
                    format!("{state:?}")
                ),
                before,
                "{site}"
            );
            core.step_battle_execution_until(&mut state, 0).unwrap();
            clean
                .step_battle_execution_until(&mut clean_state, 0)
                .unwrap();
            assert_eq!(
                core.transaction_debug_snapshot(),
                clean.transaction_debug_snapshot()
            );
            assert_eq!(
                serde_json::to_value(core.runtime_fact_summary()).unwrap(),
                serde_json::to_value(clean.runtime_fact_summary()).unwrap()
            );
        }
    }
    #[test]
    fn r09_auto_uses_same_actual_start_gate() {
        let (mut core, mut state, actor, _) = fixture(80, false);
        core.units.get_mut(&actor).unwrap().skill_activation_mode =
            crate::game::ability::SkillActivationMode::Auto;
        core.enqueue_event(crate::game::battle::enums::BattleEvent::AutoCastStart {
            time_ms: 0,
            caster_instance_id: actor,
            cause: crate::game::battle::event_log::BattleEventCause::Root {
                kind: crate::game::battle::event_log::BattleEventRootCause::Init,
            },
        });
        assert!(!core.units[&actor].corroded);
        core.step_battle_execution_until(&mut state, 0).unwrap();
        assert!(core.units[&actor].corroded);
        assert_eq!(core.runtime_fact_summary().resource_spent, 100);
    }

    #[test]
    fn r09_public_equipment_death_reactions_are_suppressed_only_for_corrosion() {
        use crate::game::{
            battle::core::types::RuntimeItem,
            data::{equipment_data::EquipmentMetadata, ValidatedGameData},
        };
        for special in [false, true] {
            let (mut core, mut state, actor, enemy) = fixture(80, false);
            let gear: EquipmentMetadata = ron::de::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/r09/death_reactions.ron"
            )))
            .unwrap();
            let base = gear.uuid;
            core.game_data =
                GameDataBuilder::from_validated(&ValidatedGameData::from(core.game_data.clone()))
                    .with_equipment(vec![gear])
                    .build_arc();
            let ally: UnitInstanceId = uuid::Uuid::from_u128(0xFA03).into();
            core.units.get_mut(&ally).unwrap().stats.current_health = 50;
            let other: UnitInstanceId = uuid::Uuid::from_u128(0xFA04).into();
            let mut unit = core.units[&enemy].clone();
            unit.instance_id = other;
            unit.spawn_order += 1;
            core.insert_runtime_unit(other, unit);
            for (index, owner) in [actor, enemy, other].into_iter().enumerate() {
                let id = uuid::Uuid::from_u128(0x909000 + index as u128);
                core.items.insert(
                    id,
                    RuntimeItem {
                        weapon_stage: Default::default(),
                        instance_id: id,
                        owner: core.units[&owner].owner,
                        owner_unit_instance: owner,
                        base_uuid: base,
                        persistent_copy_uuid: None,
                    },
                );
            }
            if special {
                let handle = core
                    .request_skill_action(0, actor, "r08_strike".into(), None)
                    .unwrap();
                core.step_battle_execution_until(&mut state, 0).unwrap();
                assert_eq!(core.units[&ally].stats.current_health, 50);
                assert!(!core.units[&enemy].is_active() && !core.units[&other].is_active());
                assert_eq!(core.query_action(&handle).unwrap().facts.deaths, 2);
                assert_eq!(
                    core.runtime_fact_summary()
                        .latest_death
                        .as_ref()
                        .unwrap()
                        .cause,
                    crate::game::battle::runtime_facts::DeathCause::Corrosion
                );
            } else {
                core.finalize_unit_death(Some(actor), enemy, Side::Opponent, 0);
                assert_eq!(
                    core.units[&ally].stats.current_health, 81,
                    "ordinary OnDeath7 + OnKill11 + OnAllyDeath13 positive control"
                );
            }
        }
    }
    #[test]
    fn graph_partial_payment_is_normal_use_even_above_corrosion_threshold() {
        use crate::game::ability::resonance::*;
        let (mut core, mut state, actor, enemy) = fixture(80, false);
        let mut skill = core
            .game_data
            .skill_data
            .get_by_id("r08_strike")
            .unwrap()
            .clone();
        skill.resonance_admission = Some(ResonanceAdmission {
            capacity: 100,
            requirement: ResonanceRequirement::Payable,
            cost: ResonanceCost::Fixed(25),
        });
        core.game_data = GameDataBuilder::empty()
            .with_skills(SkillDatabase::new(vec![skill]))
            .with_skill_fragments((*core.game_data.skill_fragment_data).clone())
            .build_arc();
        assert!(!core.live_skill_readiness(0, actor).unwrap().corrosion_ready);
        let action = core
            .request_skill_action(0, actor, "r08_strike".into(), None)
            .unwrap();
        assert_eq!(core.units[&actor].resonance_current, 75);
        core.step_battle_execution_until(&mut state, 300).unwrap();
        assert!(core.units[&actor].is_active());
        assert!(!core.units[&actor].corroded);
        assert_eq!(core.units[&actor].employee_trauma, Some(80));
        assert!(core.forced_pause_remaining_ms().is_none());
        assert_eq!(core.units[&enemy].stats.current_health, 0);
        assert_eq!(core.query_action(&action).unwrap().facts.resource_spent, 25);
    }
}
