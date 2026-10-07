use crate::game::battle::{core::BattleCore, ids::UnitInstanceId};

fn overflow(site: &'static str) -> crate::game::behavior::GameError {
    use crate::game::battle::runtime_control::{reject, BattleRuntimeFault, RuntimeFaultCode};
    reject(BattleRuntimeFault {
        code: RuntimeFaultCode::CounterOverflow,
        site,
        limit: None,
        requested: None,
    })
}

#[derive(Default)]
pub(super) struct ResonanceGainBlocks {
    hard_cc: bool,
    skill_use: bool,
    autonomous_recovery: bool,
}
impl ResonanceGainBlocks {
    pub fn blocked(&self) -> bool {
        self.hard_cc || self.skill_use || self.autonomous_recovery
    }
}

impl BattleCore {
    // Basic attacks grant resonance separately for release, dealt damage, and received damage.
    pub(in crate::game::battle::core) fn grant_basic_attack_release_resonance(
        &mut self,
        attacker_id: UnitInstanceId,
        time_ms: u64,
    ) {
        if self
            .units
            .get(&attacker_id)
            .is_some_and(Self::is_employee_resonance_unit)
        {
            return;
        }
        self.add_resonance(attacker_id, 10, time_ms, true);
    }

    pub(in crate::game::battle::core) fn grant_basic_attack_damage_dealt_resonance(
        &mut self,
        attacker_id: UnitInstanceId,
        damage_dealt: u32,
        time_ms: u64,
    ) {
        if self
            .units
            .get(&attacker_id)
            .is_some_and(Self::is_employee_resonance_unit)
        {
            return;
        }
        let gained = damage_dealt / 10;
        if gained > 0 {
            self.add_resonance(attacker_id, gained, time_ms, true);
        }
    }

    pub(in crate::game::battle::core) fn grant_basic_attack_damage_received_resonance(
        &mut self,
        target_id: UnitInstanceId,
        damage_taken: u32,
        time_ms: u64,
        target_survived: bool,
    ) {
        if self
            .units
            .get(&target_id)
            .is_some_and(Self::is_employee_resonance_unit)
        {
            return;
        }
        let gained = damage_taken / 10;
        if gained > 0 {
            self.add_resonance(target_id, gained, time_ms, target_survived);
        }
    }
}

// Employee resonance owns admission and actual use. It does not own the
// lifetime of every projectile, area or other independently released child.
impl BattleCore {
    pub(super) fn is_employee_resonance_unit(unit: &super::types::RuntimeUnit) -> bool {
        unit.owner == crate::game::enums::Side::Player
            && matches!(
                unit.source_identity,
                crate::game::battle::types::BattleUnitSourceIdentity::Employee { .. }
            )
    }

    pub(super) fn admit_employee_resonance_use(
        &mut self,
        actor: UnitInstanceId,
        skill: &crate::game::ability::SkillId,
        action: &crate::game::battle::runtime_facts::ActionHandle,
        now: u64,
    ) -> Result<bool, crate::game::behavior::GameError> {
        use crate::game::battle::runtime_facts::{
            CostReason, FactResource, ResourceOwner, RuntimeFactSource,
        };
        let unit = self
            .units
            .get(&actor)
            .ok_or(crate::game::behavior::GameError::UnitNotFound)?;
        if !Self::is_employee_resonance_unit(unit) {
            return Ok(false);
        }
        if unit.resonance_current < unit.resonance_max.max(1)
            || self.resonance_use_has_obligations(actor)
        {
            return Err(crate::game::behavior::GameError::InvalidAction);
        }
        let spent = unit.resonance_current;
        let maximum = unit.resonance_max;
        let source = RuntimeFactSource {
            actor: Some(actor),
            action: Some(action.clone()),
            fragment: unit.fragment_source.clone(),
            ..Default::default()
        };
        let reason = if self.corrosion_ready(actor, skill, now) {
            CostReason::Corrosion
        } else {
            CostReason::CastReset
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            std::mem::size_of::<super::types::ResonanceUse>()
                .checked_add(skill.as_str().len())
                .and_then(|n| n.checked_add(64)),
            "resonance_use_admission",
        )?;
        let unit = self.units.get_mut(&actor).unwrap();
        unit.resonance_current = 0;
        unit.resonance_use = Some(super::types::ResonanceUse {
            action: action.clone(),
            skill_id: skill.clone(),
            consumed: spent,
            unreleased_attacks: 0,
            control_cancelled: false,
        });
        self.record_resource_fact(
            ResourceOwner::Unit(actor),
            FactResource::Resonance,
            Some(reason),
            source,
            -i64::from(spent),
            spent,
            0,
            now,
        )?;
        self.record_event_log(
            now,
            crate::game::battle::event_log::BattleLogEvent::ResonanceChanged {
                unit_instance_id: actor,
                before: spent,
                after: 0,
                max: maximum,
            },
        );
        Ok(true)
    }

    pub(super) fn has_resonance_use_credential(
        &self,
        actor: UnitInstanceId,
        skill: &crate::game::ability::SkillId,
        action: &crate::game::battle::runtime_facts::ActionHandle,
    ) -> bool {
        self.units.get(&actor).is_some_and(|unit| {
            unit.resonance_use.as_ref().is_some_and(|use_state| {
                &use_state.action == action
                    && &use_state.skill_id == skill
                    && !use_state.control_cancelled
                    && use_state.consumed >= unit.resonance_max.max(1)
            })
        })
    }

    pub(super) fn schedule_resonance_pulse(&mut self, actor: UnitInstanceId, now: u64) {
        let Some(unit) = self.units.get_mut(&actor) else {
            return;
        };
        if !Self::is_employee_resonance_unit(unit)
            || !unit.is_active()
            || unit.next_resonance_pulse_ms.is_some()
        {
            return;
        }
        let Some(next) = (now / 1000)
            .checked_add(1)
            .and_then(|n| n.checked_mul(1000))
        else {
            let _ = overflow("resonance_pulse_deadline");
            return;
        };
        unit.next_resonance_pulse_ms = Some(next);
        self.event_queue
            .push(crate::game::battle::enums::BattleEvent::ResonancePulse {
                time_ms: next,
                unit_instance_id: actor,
                spawn_order: unit.spawn_order,
            });
    }

    pub(super) fn handle_resonance_pulse(
        &mut self,
        actor: UnitInstanceId,
        generation: u64,
        now: u64,
    ) {
        let Some(unit) = self.units.get_mut(&actor) else {
            return;
        };
        if unit.spawn_order != generation || unit.next_resonance_pulse_ms != Some(now) {
            return;
        }
        unit.next_resonance_pulse_ms = None;
        if !unit.is_active() || !Self::is_employee_resonance_unit(unit) {
            return;
        }
        self.add_resonance(actor, 1, now, false);
        self.schedule_resonance_pulse(actor, now);
    }

    pub(super) fn cancel_uses_with_unavailable_targets(
        &mut self,
        now: u64,
    ) -> Result<(), crate::game::behavior::GameError> {
        loop {
            let per_actor = self.active_skill_casts.values().fold(1usize, |sum, cast| {
                sum.saturating_add(1)
                    .saturating_add(cast.flow.as_ref().map_or(0, |flow| flow.nodes.len()))
            });
            let work = self.units.len().saturating_mul(per_actor).saturating_add(
                self.units
                    .values()
                    .filter_map(|unit| unit.resonance_use.as_ref())
                    .filter_map(|use_state| {
                        self.game_data
                            .skill_data
                            .get_by_id(use_state.skill_id.as_str())
                    })
                    .map(|skill| skill.steps.len())
                    .sum::<usize>(),
            );
            crate::authoring::control::charge(work as u64, "resonance_target_lifecycle")?;
            crate::game::battle::runtime_control::charge(
                work as u64,
                "resonance_target_lifecycle",
            )?;
            let actor = self
                .units
                .iter()
                .filter_map(|(actor, unit)| {
                    let use_state = unit
                        .resonance_use
                        .as_ref()
                        .filter(|use_state| !use_state.control_cancelled)?;
                    // The current action finishes applying its effect before its
                    // remaining authored continuation is checked.
                    if self.runtime_fact_source.action.as_ref() == Some(&use_state.action) {
                        return None;
                    }
                    let skill = self
                        .game_data
                        .skill_data
                        .get_by_id(use_state.skill_id.as_str())?;
                    let pending_target = unit
                        .pending_skill_cast
                        .as_ref()
                        .map(|pending| pending.cast_target);
                    let cast_target = self.active_skill_casts.values().find_map(|cast| {
                        (cast.fact_source.action.as_ref() == Some(&use_state.action)
                            && !cast.control_cancelled)
                            .then(|| {
                                cast.flow
                                    .as_ref()
                                    .filter(|flow| flow.has_use_control())
                                    .map(|flow| flow.accepted_target())
                            })
                            .flatten()
                    });
                    pending_target
                        .or(cast_target)
                        .filter(|target| {
                            !self.admitted_skill_target_is_valid(*actor, skill, *target)
                        })
                        .map(|_| *actor)
                })
                .min();
            let Some(actor) = actor else {
                return Ok(());
            };
            self.cancel_resonance_use_control(actor);
            self.cancel_boss_owned_work(now, actor, true, false);
        }
    }

    pub(super) fn source_owns_use_effect(
        &self,
        source: &crate::game::battle::runtime_facts::RuntimeFactSource,
    ) -> bool {
        let Some(use_state) = source
            .actor
            .and_then(|actor| self.units.get(&actor))
            .and_then(|unit| unit.resonance_use.as_ref())
        else {
            return false;
        };
        if source.action.as_ref() != Some(&use_state.action) {
            return false;
        }
        let Some(authored) = source.authored.as_ref() else {
            return false;
        };
        self.game_data
            .skill_data
            .get_by_id(use_state.skill_id.as_str())
            .is_some_and(|skill| {
                let work = skill
                    .use_lifetime
                    .effects()
                    .iter()
                    .fold(0u64, |total, reference| {
                        total
                            .saturating_add(1)
                            .saturating_add(reference.step.len() as u64)
                            .saturating_add(reference.effect.len() as u64)
                    });
                if crate::authoring::control::charge(work, "resonance_effect_reference").is_err()
                    || crate::game::battle::runtime_control::charge(
                        work,
                        "resonance_effect_reference",
                    )
                    .is_err()
                {
                    return false;
                }
                skill
                    .use_lifetime
                    .owns_effect(&authored.site.action.action_id, &authored.site.effect_id)
            })
    }

    pub(super) fn resonance_use_has_obligations(&self, actor: UnitInstanceId) -> bool {
        let Some(unit) = self.units.get(&actor) else {
            return false;
        };
        let Some(use_state) = unit.resonance_use.as_ref() else {
            return false;
        };
        if unit
            .pending_skill_cast
            .as_ref()
            .and_then(|pending| pending.action.as_ref())
            == Some(&use_state.action)
            || use_state.unreleased_attacks > 0
        {
            return true;
        }
        let work = self
            .active_skill_casts
            .values()
            .fold(0usize, |sum, cast| {
                sum.saturating_add(1)
                    .saturating_add(cast.step_progress.len())
                    .saturating_add(cast.flow.as_ref().map_or(0, |flow| flow.nodes.len()))
            })
            .saturating_add(self.sustained.len());
        if crate::authoring::control::charge(work as u64, "resonance_use_obligations").is_err()
            || crate::game::battle::runtime_control::charge(
                work as u64,
                "resonance_use_obligations",
            )
            .is_err()
        {
            return true;
        }
        if self.active_skill_casts.iter().any(|(cast_seq, cast)| {
            cast.fact_source.action.as_ref() == Some(&use_state.action)
                && ((!use_state.control_cancelled
                    && !cast.control_cancelled
                    && cast
                        .flow
                        .as_ref()
                        .is_some_and(|flow| flow.has_use_control()))
                    || cast.step_progress.iter().any(|(occurrence, progress)| {
                        if !progress.use_owned {
                            return false;
                        }
                        if crate::game::battle::runtime_control::charge(
                            self.sustained.len() as u64,
                            "resonance_sustained_children",
                        )
                        .is_err()
                        {
                            return true;
                        }
                        progress.pending_delivery_count
                            > self.sustained_flow_children(*cast_seq, *occurrence)
                    }))
        }) {
            return true;
        }
        self.sustained.has_use_effect(&use_state.action, |source| {
            self.source_owns_use_effect(source)
        })
    }

    pub(super) fn reconcile_resonance_use(&mut self, actor: UnitInstanceId) {
        if !self.resonance_use_has_obligations(actor) {
            if let Some(unit) = self.units.get_mut(&actor) {
                unit.resonance_use = None;
            }
        }
    }

    pub(super) fn resonance_use_projection(
        &self,
        actor: UnitInstanceId,
    ) -> Option<crate::game::behavior::ResonanceUseDto> {
        use crate::game::behavior::{ResonanceUseDto, ResonanceUsePhaseDto as Phase};
        if !self.resonance_use_has_obligations(actor) {
            return None;
        }
        let unit = self.units.get(&actor)?;
        let use_state = unit.resonance_use.as_ref()?;
        let phase = if let Some(pending) = unit
            .pending_skill_cast
            .as_ref()
            .filter(|pending| pending.action.as_ref() == Some(&use_state.action))
        {
            if pending.start_gate_pending {
                Phase::Accepted
            } else {
                Phase::Casting
            }
        } else if use_state.unreleased_attacks > 0
            || self.active_skill_casts.values().any(|cast| {
                cast.fact_source.action.as_ref() == Some(&use_state.action)
                    && !cast.control_cancelled
                    && cast
                        .flow
                        .as_ref()
                        .is_some_and(|flow| flow.has_use_control())
            })
        {
            Phase::Acting
        } else {
            Phase::Effects
        };
        Some(ResonanceUseDto {
            action_ordinal: use_state.action.ordinal().0,
            skill_id: use_state.skill_id.clone(),
            consumed: use_state.consumed,
            phase,
        })
    }

    pub(super) fn resonance_gain_block_reasons(
        &self,
        actor: UnitInstanceId,
        now: u64,
    ) -> Vec<crate::game::behavior::ResonanceGainBlockReasonDto> {
        use crate::game::behavior::ResonanceGainBlockReasonDto as Reason;
        let blocks = self.resonance_gain_blocks(actor, now);
        let mut reasons = Vec::new();
        if blocks.hard_cc {
            reasons.push(Reason::HardCrowdControl);
        }
        if blocks.skill_use {
            reasons.push(Reason::SkillUse);
        }
        if blocks.autonomous_recovery {
            reasons.push(Reason::AutonomousRecovery);
        }
        reasons
    }

    pub(super) fn resonance_gain_blocks(
        &self,
        actor: UnitInstanceId,
        now: u64,
    ) -> ResonanceGainBlocks {
        ResonanceGainBlocks {
            hard_cc: self.active_hard_cc_release_time_ms(actor, now).is_some(),
            skill_use: self.resonance_use_has_obligations(actor),
            autonomous_recovery: self.units.get(&actor).is_some_and(|unit| {
                !Self::is_employee_resonance_unit(unit)
                    && !unit.action_locks.can_gain_resonance(now)
            }),
        }
    }

    pub(super) fn cancel_resonance_use_control(&mut self, actor: UnitInstanceId) {
        if let Some(use_state) = self
            .units
            .get_mut(&actor)
            .and_then(|unit| unit.resonance_use.as_mut())
        {
            use_state.control_cancelled = true;
            use_state.unreleased_attacks = 0;
        }
    }

    pub(super) fn use_attack_can_execute(
        &self,
        source: &crate::game::battle::runtime_facts::RuntimeFactSource,
    ) -> bool {
        !source.use_control
            || source
                .actor
                .and_then(|actor| self.units.get(&actor))
                .and_then(|unit| unit.resonance_use.as_ref())
                .is_some_and(|use_state| {
                    !use_state.control_cancelled
                        && source.action.as_ref() == Some(&use_state.action)
                })
    }

    pub(super) fn reserve_use_attack(
        &mut self,
        source: &crate::game::battle::runtime_facts::RuntimeFactSource,
    ) -> Result<bool, crate::game::behavior::GameError> {
        let Some(use_state) = source
            .actor
            .and_then(|actor| self.units.get(&actor))
            .and_then(|unit| unit.resonance_use.as_ref())
            .filter(|use_state| {
                source.action.as_ref() == Some(&use_state.action) && !use_state.control_cancelled
            })
        else {
            return Ok(false);
        };
        let Some(authored) = source
            .authored
            .as_ref()
            .filter(|authored| authored.site.action.definition_id == use_state.skill_id)
        else {
            return Ok(false);
        };
        let Some(skill) = self
            .game_data
            .skill_data
            .get_by_id(use_state.skill_id.as_str())
        else {
            return Ok(false);
        };
        let work = skill.steps.iter().fold(0u64, |work, step| {
            work.saturating_add(1).saturating_add(step.id.len() as u64)
        });
        crate::authoring::control::charge(work, "resonance_attack_producer")?;
        crate::game::battle::runtime_control::charge(work, "resonance_attack_producer")?;
        let Some(step) = skill
            .steps
            .iter()
            .find(|step| step.id == authored.site.action.action_id)
        else {
            return Ok(false);
        };
        // An independent area owns every attack it creates, including its first
        // tick. A retained action attribution must not reopen the creator's use.
        if matches!(&step.delivery, crate::game::ability::DeliveryDef::TileArea { area, .. } if area.duration_ms > 0)
        {
            return Ok(false);
        }
        let use_state = self
            .units
            .get_mut(&source.actor.unwrap())
            .unwrap()
            .resonance_use
            .as_mut()
            .unwrap();
        use_state.unreleased_attacks = use_state
            .unreleased_attacks
            .checked_add(1)
            .ok_or_else(|| overflow("resonance_use_attacks"))?;
        Ok(true)
    }

    pub(super) fn finish_use_attack(
        &mut self,
        source: &crate::game::battle::runtime_facts::RuntimeFactSource,
    ) {
        if !source.use_control {
            return;
        }
        if let Some(use_state) = source
            .actor
            .and_then(|actor| self.units.get_mut(&actor))
            .and_then(|unit| unit.resonance_use.as_mut())
            .filter(|use_state| source.action.as_ref() == Some(&use_state.action))
        {
            use_state.unreleased_attacks = use_state.unreleased_attacks.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests;
