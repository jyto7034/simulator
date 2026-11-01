use crate::game::{
    battle::{
        cooldown::SourcedEffect,
        core::{
            commands::RuntimeCommandBatch, triggers::TriggerOccurrenceContext,
            types::CommandExecutionSummary, BattleCore,
        },
        damage::{
            apply_damage_to_unit, CombatRollIdentity, CombatRollKind,
            DamageBonusSnapshot, DamageContext, DamageModifiers, DamageResult, DamageSource,
            DamageSourceSnapshot,
        },
        event_log::HpChangeReason,
        ids::UnitInstanceId,
    },
    determinism,
    stats::{Effect, TriggerType},
};

#[derive(Debug, Clone, Copy)]
pub(in crate::game::battle::core) enum CommittedDamagePolicy {
    BasicAttack {
        occurrence_id: uuid::Uuid,
        occurrence_time_ms: u64,
    },
    GeneralCommand,
    BypassShields,
}

#[derive(Debug, Clone)]
pub(in crate::game::battle::core) struct CommittedDamageInput {
    pub(in crate::game::battle::core) target_id: UnitInstanceId,
    pub(in crate::game::battle::core) source_snapshot: DamageSourceSnapshot,
    pub(in crate::game::battle::core) committed_at_ms: u64,
    pub(in crate::game::battle::core) policy: CommittedDamagePolicy,
}

impl BattleCore {
    fn damage_effect_snapshot_from_triggers(
        effects: Vec<SourcedEffect>,
    ) -> Result<
        (
            DamageModifiers,
            Vec<DamageBonusSnapshot>,
            Vec<crate::game::battle::damage::DamageModifierSourceSnapshot>,
        ),
        crate::game::behavior::GameError,
    > {
        crate::game::battle::runtime_control::admit_retained_growth(0,
            effects.len().checked_mul(std::mem::size_of::<DamageBonusSnapshot>() + std::mem::size_of::<crate::game::battle::damage::DamageModifierSourceSnapshot>()),
            "damage_trigger_source_vectors")?;
        let mut modifiers = DamageModifiers::default();
        let mut bonus_damage = Vec::new();
        let mut modifier_sources = Vec::new();
        for sourced in effects {
            match sourced.effect {
                Effect::ModifyDamage(effect_modifiers) => {
                    modifiers = modifiers.merge(effect_modifiers);
                    modifier_sources.push(
                        crate::game::battle::damage::DamageModifierSourceSnapshot {
                            source: sourced.source,
                            equipment_source: sourced.equipment_source,
                        },
                    );
                }
                Effect::BonusDamage {
                    flat,
                    percent,
                    damage_type,
                } => {
                    bonus_damage.push(DamageBonusSnapshot {
                        source: sourced.source,
                        equipment_source: sourced.equipment_source,
                        flat,
                        percent,
                        damage_type,
                    });
                }
                Effect::BasicLifesteal { .. }
                | Effect::Modifier(_)
                | Effect::Heal { .. }
                | Effect::ApplyBuff { .. } => {}
            }
        }
        Ok((modifiers, bonus_damage, modifier_sources))
    }

    pub(in crate::game::battle::core) fn damage_effects_from_source_snapshot(
        snapshot: &DamageSourceSnapshot,
    ) -> Vec<SourcedEffect> {
        let mut effects = Vec::new();
        if snapshot.on_attack_modifiers != DamageModifiers::default() {
            effects.push(SourcedEffect {
                equipment_source: None,
                source: crate::game::battle::cooldown::CooldownSource::Unit {
                    unit_instance_id: snapshot.source_id,
                },
                target: crate::game::stats::TriggerEffectTarget::SelfUnit,
                effect: Effect::ModifyDamage(snapshot.on_attack_modifiers),
            });
        }
        effects.extend(
            snapshot
                .on_attack_bonus_damage
                .iter()
                .map(|bonus| SourcedEffect {
                    source: bonus.source,
                    equipment_source: bonus.equipment_source,
                    target: crate::game::stats::TriggerEffectTarget::SelfUnit,
                    effect: Effect::BonusDamage {
                        flat: bonus.flat,
                        percent: bonus.percent,
                        damage_type: bonus.damage_type,
                    },
                }),
        );
        effects
    }

    pub(in crate::game::battle::core) fn damage_source_snapshot_for_unit(
        &mut self,
        source_id: UnitInstanceId,
        target_id: UnitInstanceId,
        damage_source: DamageSource,
        damage_type: crate::game::battle::damage::DamageType,
        base_damage: u32,
        modifiers: DamageModifiers,
        minimum_damage: u32,
        committed_at_ms: u64,
        include_on_attack_damage_effects: bool,
    ) -> Option<DamageSourceSnapshot> {
        let mut snapshot = self.damage_source_snapshot_template_for_unit(
            source_id,
            damage_source,
            damage_type,
            base_damage,
            modifiers,
            minimum_damage,
            committed_at_ms,
            include_on_attack_damage_effects,
        )?;
        snapshot = self.materialize_damage_source_snapshot_for_target(snapshot, target_id);
        Some(snapshot)
    }

    pub(in crate::game::battle::core) fn damage_source_snapshot_for_regular_occurrence(
        &mut self,
        source_id: UnitInstanceId,
        target_id: UnitInstanceId,
        damage_source: DamageSource,
        damage_type: crate::game::battle::damage::DamageType,
        base_damage: u32,
        modifiers: DamageModifiers,
        minimum_damage: u32,
        committed_at_ms: u64,
        include_on_attack_damage_effects: bool,
        regular: bool,
    ) -> Option<DamageSourceSnapshot> {
        let snapshot = self.damage_source_snapshot_template_for_occurrence(
            source_id,
            damage_source,
            damage_type,
            base_damage,
            modifiers,
            minimum_damage,
            committed_at_ms,
            include_on_attack_damage_effects,
            regular,
        )?;
        Some(self.materialize_damage_source_snapshot_for_target(snapshot, target_id))
    }

    pub(in crate::game::battle::core) fn damage_source_snapshot_template_for_unit(
        &mut self,
        source_id: UnitInstanceId,
        damage_source: DamageSource,
        damage_type: crate::game::battle::damage::DamageType,
        base_damage: u32,
        modifiers: DamageModifiers,
        minimum_damage: u32,
        committed_at_ms: u64,
        include_on_attack_damage_effects: bool,
    ) -> Option<DamageSourceSnapshot> {
        self.damage_source_snapshot_template_for_occurrence(
            source_id,
            damage_source,
            damage_type,
            base_damage,
            modifiers,
            minimum_damage,
            committed_at_ms,
            include_on_attack_damage_effects,
            false,
        )
    }

    pub(in crate::game::battle::core) fn damage_source_snapshot_template_for_occurrence(
        &mut self,
        source_id: UnitInstanceId,
        damage_source: DamageSource,
        damage_type: crate::game::battle::damage::DamageType,
        base_damage: u32,
        modifiers: DamageModifiers,
        minimum_damage: u32,
        committed_at_ms: u64,
        include_on_attack_damage_effects: bool,
        regular_basic: bool,
    ) -> Option<DamageSourceSnapshot> {
        let base_damage = if damage_source == DamageSource::BasicAttack {
            let bonus = self
                .aura_basic_bonus_for_occurrence(source_id, regular_basic)
                .ok()?
                .0;
            let value = u64::from(base_damage) * (100 + u64::from(bonus)) / 100;
            if self.has_regular_support() {
                match u32::try_from(value) {
                    Ok(value) => value,
                    Err(_) => {
                        let _ = crate::game::battle::runtime_control::reject(
                        crate::game::battle::runtime_control::BattleRuntimeFault {
                            code: crate::game::battle::runtime_control::RuntimeFaultCode::CounterOverflow,
                            site: "aura_regular_damage_snapshot", limit: Some(u32::MAX as u64), requested: Some(value),
                        });
                        return None;
                    }
                }
            } else {
                value.min(u64::from(u32::MAX)) as u32
            }
        } else {
            base_damage
        };
        let (source_side, source_attack) = self
            .units
            .get(&source_id)
            .map(|unit| (unit.owner, unit.stats.attack))
            .or_else(|| {
                self.graveyard
                    .get(&source_id)
                    .map(|unit| (unit.owner, unit.stats.attack))
            })?;
        let (on_attack_modifiers, on_attack_bonus_damage, on_attack_modifier_sources) =
            if include_on_attack_damage_effects {
                Self::damage_effect_snapshot_from_triggers(
                    self.collect_current_hit_triggers(source_id, TriggerType::OnAttack),
                )
                .ok()?
            } else {
                (DamageModifiers::default(), Vec::new(), Vec::new())
            };

        let mut basic_lifesteal = Vec::new();
        if damage_source == DamageSource::BasicAttack {
            for effect in self.collect_all_triggers(source_id, TriggerType::OnAttack) {
                if let Effect::BasicLifesteal { percent, max_heal } = effect.effect {
                    crate::game::battle::runtime_control::admit_retained_growth(
                        0,
                        basic_lifesteal.len().checked_add(1).and_then(|n| {
                            n.checked_mul(
                                std::mem::size_of::<
                                    crate::game::battle::damage::BasicLifestealSnapshot,
                                >() * 2,
                            )
                        }),
                        "basic_lifesteal_snapshot",
                    )
                    .ok()?;
                    basic_lifesteal.push(crate::game::battle::damage::BasicLifestealSnapshot {
                        percent,
                        max_heal,
                        equipment: effect.equipment_source,
                        fact_source: None,
                    });
                }
            }
            self.append_sustained_lifesteal(source_id, committed_at_ms, &mut basic_lifesteal)
                .ok()?;
        }
        self.runtime_fact_source
            .admit_owner("damage_snapshot_source")
            .ok()?;
        let fact_source = crate::game::battle::runtime_facts::RuntimeFactSource {
            actor: Some(source_id),
            ..self.runtime_fact_source.clone()
        };
        let (base_damage, fact_source) =
            if damage_source == DamageSource::BasicAttack && self.source_has_relics(&fact_source) {
                self.bind_basic_relic_source(source_id, damage_type, base_damage, &fact_source)
                    .map_err(|error| match error {
                        crate::game::behavior::GameError::BattleRuntimeFault(_) => error,
                        _ => crate::game::battle::runtime_control::invariant("relic_basic_binding"),
                    })
                    .ok()?
            } else {
                (base_damage, fact_source)
            };
        let fact_source = if damage_source == DamageSource::BasicAttack {
            fact_source
                .with_regular_support(
                    self.capture_regular_support(source_id, regular_basic)
                        .ok()?,
                )
                .ok()?
        } else {
            fact_source
        };
        Some(DamageSourceSnapshot {
            fact_source,
            basic_primary: None,
            skill_cast_source: None,
            value_expr_capture: None,
            prepared_basic: None,
            state_basic_capture: None,
            weapon_growth: None,
            equipment_attack: None,
            equipment_hits: None,
            release_gap: None,
            magazine: None,
            presence_capture: None,
            reactive_capture: None,
            mobile_capture: None,
            weapon_mark_capture: None,
            source_id,
            source_side,
            source_attack,
            source: damage_source,
            damage_type,
            base_damage,
            modifiers,
            crit_roll_percent: None,
            crit_roll_identity: CombatRollIdentity::new(
                CombatRollKind::for_damage_source(damage_source),
                Some(source_id),
                self.damage_source_instance_id(
                    source_id,
                    damage_source,
                    damage_type,
                    base_damage,
                    committed_at_ms,
                ),
            ),
            minimum_damage,
            committed_at_ms,
            basic_lifesteal,
            on_attack_modifiers,
            on_attack_bonus_damage,
            on_attack_modifier_sources,
        })
    }

    pub(in crate::game::battle::core) fn execute_committed_damage(
        &mut self,
        input: CommittedDamageInput,
    ) -> Option<CommandExecutionSummary> {
        self.execute_damage_with_consumed_dot(input, false)
    }

    /// Only a producer that consumed its own DOT for this damage may supply
    /// this qualification. It does not confer ownership or equipment rights.
    pub(super) fn execute_damage_with_consumed_dot(
        &mut self,
        input: CommittedDamageInput,
        consumed_owner_dot: bool,
    ) -> Option<CommandExecutionSummary> {
        self.execute_damage_with_observation_origin(input, consumed_owner_dot, None)
    }

    /// Supplies immutable authored provenance only to this damage application.
    /// Reactions, reflected hits and gameplay facts keep their own source.
    pub(super) fn execute_damage_with_observation_origin(
        &mut self,
        input: CommittedDamageInput,
        consumed_owner_dot: bool,
        observation_origin: Option<&crate::game::battle::runtime_facts::RuntimeFactSource>,
    ) -> Option<CommandExecutionSummary> {
        let mut input = input;
        input.source_snapshot.fact_source = input
            .source_snapshot
            .fact_source
            .inherit_causal_context(&self.runtime_fact_source)
            .ok()?;
        let prior = std::mem::replace(
            &mut self.runtime_fact_source,
            input.source_snapshot.fact_source.clone(),
        );
        let result = self.execute_committed_damage_inner(input, consumed_owner_dot, observation_origin);
        self.runtime_fact_source = prior;
        result
    }

    fn execute_committed_damage_inner(
        &mut self,
        input: CommittedDamageInput,
        consumed_owner_dot: bool,
        observation_origin: Option<&crate::game::battle::runtime_facts::RuntimeFactSource>,
    ) -> Option<CommandExecutionSummary> {
        let CommittedDamageInput {
            target_id,
            mut source_snapshot,
            committed_at_ms,
            policy,
        } = input;
        self.reconcile_sustained(committed_at_ms).ok()?;
        let (
            target_owner,
            target_defense,
            target_magic_resist,
            mut target_incoming_modifiers,
            target_current_hp,
            target_max_hp,
        ) = self
            .units
            .get(&target_id)
            .filter(|target| target.is_active())
            .map(|target| {
                (
                    target.owner,
                    target.stats.defense,
                    target.stats.magic_resist,
                    target.incoming_damage_modifiers,
                    target.stats.current_health,
                    target.stats.max_health,
                )
            })?;

        if source_snapshot.source == DamageSource::Ability && source_snapshot.base_damage > 0
            && self.block_equipment_skill(target_id, source_snapshot.source_id,
                Some(source_snapshot.source_side), committed_at_ms).ok()? {
            return Some(CommandExecutionSummary::default());
        }

        let mut strike = match self.admit_damage_strike(target_id) {
            Ok(value) => value,
            Err(_) => return None,
        };

        let taunt_eligible = source_snapshot.source != DamageSource::Environment && !source_snapshot.fact_source.reaction
            && !self.units.get(&source_snapshot.source_id).is_some_and(|u| u.corroded)
            && !self.units.get(&target_id).is_some_and(|u| u.corroded);
        let (taunt_reduction, taunt_bonus) = if taunt_eligible { self.approach_taunt_damage(source_snapshot.source_id, target_id, source_snapshot.source == DamageSource::BasicAttack) } else { (0, 0) };
        target_incoming_modifiers.damage_reduction_percent = target_incoming_modifiers.damage_reduction_percent.saturating_add(taunt_reduction);
        target_incoming_modifiers.damage_amp_percent = target_incoming_modifiers.damage_amp_percent.saturating_add(taunt_bonus).saturating_add(self.shot_cohort_damage_bonus(target_id,&source_snapshot).ok()?);
        target_incoming_modifiers.damage_amp_percent = target_incoming_modifiers.damage_amp_percent.saturating_add(self.ground_status_totals(target_id).ok()?.1).saturating_add(self.strike_hit_damage_taken_amp(&mut strike).ok()?).saturating_add(self.equipment_form_damage_percent(target_id).ok()?);
        target_incoming_modifiers.damage_reduction_percent = target_incoming_modifiers
            .damage_reduction_percent
            .saturating_add(
                self.sustained_damage_reduction(target_id, committed_at_ms)
                    .ok()?,
            )
            .saturating_add(self.parry_reduction(target_id, committed_at_ms).ok()?)
            .saturating_add(self.channel_opening_reduction(target_id, &source_snapshot, committed_at_ms).ok()?);
        let conditional_reduction = self.strike_conditional_reduction(
            &mut strike, &source_snapshot, committed_at_ms).ok()?;
        target_incoming_modifiers.damage_reduction_percent = target_incoming_modifiers
            .damage_reduction_percent.saturating_add(self.strike_wearer_reduction(
                &mut strike, &source_snapshot, committed_at_ms).ok()?);
        let without_passive_guard = target_incoming_modifiers;
        let mut passive_guard = self
            .passive_parry_guard(target_id, committed_at_ms)
            .ok()?;
        if let Some(guard) = &passive_guard {
            if !guard.reduction_suppressed {
                target_incoming_modifiers.damage_reduction_percent = target_incoming_modifiers
                    .damage_reduction_percent
                    .saturating_add(guard.policy.reduction_percent as i32);
            }
        }

        if self.evade_equipment_basic(target_id, &source_snapshot, &mut strike, committed_at_ms).ok()? {
            return Some(CommandExecutionSummary::default());
        }

        source_snapshot.modifiers.base_damage_amp_percent = source_snapshot.modifiers.base_damage_amp_percent
            .saturating_add(self.strike_source_direct_amp(&mut strike, &source_snapshot, committed_at_ms).ok()?);
        source_snapshot.modifiers.damage_amp_percent = source_snapshot.modifiers.damage_amp_percent
            .saturating_add(self.ready_wait_damage_percent(&source_snapshot, committed_at_ms).ok()?);
        source_snapshot.modifiers.base_damage_amp_percent = source_snapshot.modifiers.base_damage_amp_percent
            .saturating_add(self.captured_cast_damage_percent(&source_snapshot));
        source_snapshot.modifiers.damage_amp_percent = source_snapshot.modifiers.damage_amp_percent
            .saturating_add(self.nearby_ally_damage_percent(&source_snapshot, target_id, committed_at_ms).ok()?);
        let conditional_damage = self.strike_conditional_damage_per_type(
            &mut strike, &source_snapshot, committed_at_ms).ok()?;
        let penetration = self.direct_penetration(
            &source_snapshot, target_id, committed_at_ms).ok()?;
        let continuity_damage = self.attack_continuity_damage_percent(
            &source_snapshot, target_id, committed_at_ms).ok()?;
        let shield_beneficiary_percent = self.shield_beneficiary_damage_percent(&source_snapshot, target_id, committed_at_ms).ok()?;
        if let Some(capture) = source_snapshot.release_gap.as_ref().filter(|_| {
            source_snapshot.source == DamageSource::BasicAttack && source_snapshot.basic_primary.is_some()
                && !source_snapshot.fact_source.reaction && source_snapshot.fact_source.guard_release().is_none()
                && source_snapshot.fact_source.cast.as_ref().is_none_or(|c| c.proc_origin.is_none())
        }) {
            source_snapshot.modifiers.base_damage_amp_percent = source_snapshot.modifiers.base_damage_amp_percent
                .saturating_add(capture.percent as i32);
        }
        self.apply_weapon_growth_damage(&mut source_snapshot, target_id).ok()?;
        let equipment_bonus_source = self.apply_equipment_attack_damage(&mut source_snapshot, target_id).ok()?;
        source_snapshot.modifiers.base_damage_amp_percent = source_snapshot.modifiers.base_damage_amp_percent
            .saturating_add(self.equipment_primary_damage_percent(&source_snapshot, target_id, committed_at_ms).ok()?);
        self.apply_wedge(&mut source_snapshot, target_id, strike.ordinal())
            .ok()?;
        let mut deflection = self
            .deflect_direct_strike(target_id, &source_snapshot, committed_at_ms)
            .ok()?;
        if let Some(fact) = &mut deflection {
            use crate::game::battle::runtime_facts::{CounterOutcome, DeflectionKind};
            self.mark_strike_deflected(&mut strike);
            if fact.kind == DeflectionKind::Parry {
                fact.counter = if self
                    .units
                    .get(&source_snapshot.source_id)
                    .is_some_and(|u| u.is_active())
                {
                    CounterOutcome::Executed
                } else {
                    CounterOutcome::AttackerUnavailable
                };
            }
        }
        let accepted_hit = deflection.is_none() && !self.sustained_invulnerable(target_id, committed_at_ms).ok()?;
        if accepted_hit { self.mobile_accepted_hit(&source_snapshot, target_id, committed_at_ms).ok()?; }
        let mut pending_counter = deflection.clone();
        let snapshot_on_attack_effects =
            Self::damage_effects_from_source_snapshot(&source_snapshot);
        let mut command_batches = Vec::new();
        let mut activation_commands = Vec::new();
        let mut on_hit_effects = Vec::new();
        if let CommittedDamagePolicy::BasicAttack {
            occurrence_id,
            occurrence_time_ms,
        } = policy
        {
            if deflection.is_some() {
                on_hit_effects = self.collect_current_hit_triggers(target_id, TriggerType::OnHit);
            } else {
                let source_live = self
                    .units
                    .get(&source_snapshot.source_id)
                    .is_some_and(|unit| unit.is_active());
                let on_attack_plan = source_live.then(|| {
                    self.dispatch_item_trigger_occurrence(TriggerOccurrenceContext {
                        trigger_type: TriggerType::OnAttack,
                        trigger_unit_id: source_snapshot.source_id,
                        counterpart_unit_id: Some(target_id),
                        target_id: Some(target_id),
                        occurrence_id,
                        occurrence_index: 0,
                        allow_dead_caster: false,
                        committed_hit: Some(&source_snapshot),
                        time_ms: occurrence_time_ms,
                    })
                });
                let mut on_hit_plan =
                    self.dispatch_item_trigger_occurrence(TriggerOccurrenceContext {
                        trigger_type: TriggerType::OnHit,
                        trigger_unit_id: target_id,
                        counterpart_unit_id: Some(source_snapshot.source_id),
                        target_id: Some(source_snapshot.source_id),
                        occurrence_id,
                        occurrence_index: 0,
                        allow_dead_caster: false,
                        committed_hit: Some(&source_snapshot),
                        time_ms: occurrence_time_ms,
                    });
                on_hit_effects = std::mem::take(&mut on_hit_plan.current_hit_effects);

                if let Some(plan) = on_attack_plan {
                    let (effect_batches, mut activations) =
                        plan.into_effect_batches_and_activations();
                    command_batches.extend(effect_batches);
                    activation_commands.append(&mut activations);
                }
                let (effect_batches, mut activations) =
                    on_hit_plan.into_effect_batches_and_activations();
                command_batches.extend(effect_batches);
                activation_commands.append(&mut activations);
            }
        }

        let ctx = DamageContext {
            attacker_side: source_snapshot.source_side,
            target_side: target_owner,
            attacker_attack: source_snapshot.source_attack,
            target_armor: target_defense,
            target_magic_resist,
            target_incoming_modifiers,
            target_current_hp,
            target_max_hp,
            on_attack_effects: &snapshot_on_attack_effects,
            on_hit_effects: &on_hit_effects,
        };
        let request = source_snapshot.request(target_id);
        let component_bound = snapshot_on_attack_effects
            .len()
            .checked_add(on_hit_effects.len())
            .and_then(|n| n.checked_add(1));
        if crate::game::battle::runtime_control::limit(
            crate::game::battle::runtime_control::RuntimeFaultCode::StrikeSegmentLimit,
            "damage_formula_components",
            self.runtime_facts.limits.max_strike_segments,
            component_bound,
        )
        .is_err()
        {
            return None;
        }
        if crate::authoring::control::charge(
            component_bound.unwrap_or(0) as u64,
            "damage_formula_components",
        )
        .is_err()
        {
            return None;
        }
        let formula_storage =
            match self
                .runtime_facts
                .reserve_fact_storage(component_bound.and_then(|n| {
                    n.checked_mul(
                        2 * std::mem::size_of::<
                            Option<crate::game::battle::damage::DamageComponentSource>,
                        >(),
                    )
                })) {
                Ok(bytes) => bytes,
                Err(_) => return None,
            };
        let eligible_flat = accepted_hit && self.incoming_flat_eligible(target_id, &source_snapshot);
        if eligible_flat {
            crate::authoring::control::charge(component_bound.unwrap_or(0) as u64,
                "incoming_flat_component_scan").ok()?;
        }
        let eligible_flat = eligible_flat && crate::game::battle::damage::has_attacker_physical_component(&request, &ctx);
        let mut allowance = if eligible_flat { self.strike_incoming_flat(&mut strike).ok()? }
            else { super::equipment_defense::FlatAllowance::default() };
        let flat_before = allowance.remaining;
        let attacker_support = crate::game::battle::damage::AttackerDamageSupport {
            penetration,
            all_types_percent: shield_beneficiary_percent,
            per_type_percent: [
                conditional_damage[0].saturating_add(continuity_damage),
                conditional_damage[1].saturating_add(continuity_damage),
                conditional_damage[2],
            ],
        };
        let mut result = crate::game::battle::damage::calculate_damage_with_source_support(&request, &ctx, &mut allowance.remaining, conditional_reduction, attacker_support);
        if eligible_flat {
            self.commit_incoming_flat(&mut strike, allowance);
            self.observe_incoming_flat(target_id, &source_snapshot, committed_at_ms,
                strike.ordinal().0, allowance, flat_before).ok()?;
            crate::authoring::control::poll("incoming_flat_applied").ok()?;
        }
        if accepted_hit && !source_snapshot.fact_source.reaction && source_snapshot.fact_source.guard_release().is_none() && source_snapshot.source_id != target_id && source_snapshot.source_side != target_owner && matches!(source_snapshot.source,DamageSource::BasicAttack | DamageSource::Ability) {
            if let Some(guard)=passive_guard.as_mut().filter(|g|!g.reduction_suppressed) {
                crate::authoring::control::charge(result.breakdown.len() as u64, "guard_reduction_credit").ok()?;
                let blocked=crate::game::battle::damage::reduction_credit_with_source_support(&request,&ctx,&result,without_passive_guard,flat_before,conditional_reduction,attacker_support);
                guard.charge=self.accumulate_guard_charge(target_id,blocked).ok()?;
            }
        }
        let mut protection = if let Some(fact) = deflection {
            result.triggered_commands.clear();
            let mut receipt = crate::game::battle::runtime_facts::ProtectionReceipt {
                mitigated_damage: result.final_damage,
                prevented: result.final_damage,
                ..Default::default()
            };
            receipt.details_mut().ok()?.deflection = Some(Box::new(fact));
            receipt
        } else {
            self.absorb_shields_before_deferral(target_id, result.final_damage, &result.breakdown, committed_at_ms, matches!(policy, CommittedDamagePolicy::BypassShields))
                .ok()?
        };
        result.final_damage = result
            .final_damage
            .saturating_sub(protection.absorbed)
            .saturating_sub(protection.prevented);
        let idle_guard_percent = self.consume_idle_hp_guard(target_id, result.final_damage,
            committed_at_ms, &source_snapshot.fact_source, &mut protection).ok()?;
        result.final_damage = Self::after_idle_hp_guard(result.final_damage, idle_guard_percent);
        protection.deferred = self.defer_equipment_damage(target_id, &source_snapshot, result.final_damage, committed_at_ms).ok()?;
        result.final_damage -= protection.deferred;
        let prevented_before_survival = protection.prevented;
        if protection.deflection.is_none() {
            self.prevent_sustained_death(target_id, result.final_damage, committed_at_ms, &mut protection).ok()?;
            result.final_damage = result.final_damage.saturating_sub(protection.prevented - prevented_before_survival);
        }
        let prevented_before_revival = protection.prevented;
        let revival = self.prepare_employee_revival(target_id, result.final_damage, committed_at_ms,
            &source_snapshot.fact_source, &mut protection).ok()?;
        result.final_damage = result.final_damage.saturating_sub(protection.prevented - prevented_before_revival);
        let prevented_before_last_will = protection.prevented;
        let last_will = self.prepare_last_will(target_id, result.final_damage, committed_at_ms,
            &source_snapshot.fact_source, &mut protection).ok()?;
        result.final_damage = result.final_damage.saturating_sub(protection.prevented - prevented_before_last_will);
        result.target_remaining_hp = target_current_hp.saturating_sub(result.final_damage);
        result.target_killed = target_current_hp > 0 && result.target_remaining_hp == 0;
        // One roll per target strike, on its first positive HP-loss segment.
        // Shielded/deflected/zero segments do not roll or create reflection.
        let mut reflection_snapshot = None;
        if let Some(guard) = &mut passive_guard {
            use crate::game::battle::runtime_facts::ReflectionOutcome;
            let eligible = !source_snapshot.fact_source.reaction && source_snapshot.fact_source.guard_release().is_none()
                && source_snapshot.source_id != target_id
                && source_snapshot.source_side != target_owner
                && matches!(source_snapshot.source, DamageSource::BasicAttack | DamageSource::Ability);
            if let Some(key) = strike.first_positive_roll_key().filter(|_| eligible && result.target_remaining_hp < target_current_hp) {
                let roll = (determinism::uuid_v4_from_seed(
                    self.seed ^ Self::unit_tag(target_id),
                    0x7061727279677561,
                    key,
                )
                .as_u128()
                    % 100) as u8;
                guard.roll_percent = Some(roll);
                guard.reflected_base = (u64::from(target_current_hp - result.target_remaining_hp)
                    * u64::from(guard.policy.reflection_percent)
                    / 100) as u32;
                guard.reflection = if u32::from(roll) >= guard.policy.reflection_chance_percent {
                    ReflectionOutcome::Missed
                } else if guard.reflected_base == 0 {
                    ReflectionOutcome::RoundedToZero
                } else if result.target_remaining_hp == 0 || last_will.is_some() {
                    ReflectionOutcome::OwnerDefeated
                } else if !self
                    .units
                    .get(&source_snapshot.source_id)
                    .is_some_and(|u| u.is_active())
                {
                    ReflectionOutcome::AttackerUnavailable
                } else {
                    ReflectionOutcome::Executed
                };
                if guard.reflection == ReflectionOutcome::Executed {
                    let prior =
                        std::mem::replace(&mut self.runtime_fact_source, (*guard.source).clone());
                    reflection_snapshot = self.damage_source_snapshot_for_unit(
                        target_id,
                        source_snapshot.source_id,
                        DamageSource::Environment,
                        source_snapshot.damage_type,
                        guard.reflected_base,
                        DamageModifiers::default(),
                        0,
                        committed_at_ms,
                        false,
                    );
                    self.runtime_fact_source = prior;
                }
            }
        }
        if let Some(guard) = passive_guard {
            protection.details_mut().ok()?.passive_guard = Some(Box::new(guard));
        }
        let effective_hit = target_current_hp > result.target_remaining_hp || protection.absorbed > 0;
        // The canonical component order is base, OnAttack, OnHit. Shields
        // consume that order; an appended bonus cannot claim the prepared kill.
        let prepared_direct_kill = target_current_hp > 0 && result.target_remaining_hp == 0
            && result.breakdown.first().is_some_and(|base|
                self.immediate_primary_after_deferral(target_id, Self::after_idle_hp_guard(base.final_damage.saturating_sub(protection.primary_absorbed()), idle_guard_percent), &source_snapshot).ok().is_some_and(|base| base >= target_current_hp));

        let damage_fact = match self.prepare_damage_fact(
            &result,
            protection,
            strike,
            &source_snapshot,
            equipment_bonus_source,
            &on_hit_effects,
            target_incoming_modifiers,
            target_current_hp,
            committed_at_ms,
        ) {
            Ok(value) => value,
            Err(_) => return None,
        };
        let first_target_strike = damage_fact.1;
        let committed_strike = damage_fact.0.strike;
        self.runtime_facts.release_fact_storage(formula_storage);
        let actual_damage = target_current_hp.saturating_sub(result.target_remaining_hp);
        if actual_damage > 0 {
            self.commit_wedge(&source_snapshot, target_id, damage_fact.0.strike)
                .ok()?;
        }
        let killed_target = target_current_hp > 0 && result.target_remaining_hp == 0;

        let equipment_kill = if killed_target || last_will.is_some() {
            self.capture_equipment_kill(
                &source_snapshot, target_id, consumed_owner_dot, prepared_direct_kill, committed_at_ms,
            ).ok()?
        } else { None };
        if matches!(policy, CommittedDamagePolicy::BasicAttack { .. }) {
            self.grant_basic_attack_damage_dealt_resonance(
                source_snapshot.source_id,
                actual_damage,
                committed_at_ms,
            );
        }
        if effective_hit { self.weapon_mark_hit(&source_snapshot, target_id).ok()?; }
        if effective_hit { self.presence_hit(&source_snapshot, target_id, killed_target, committed_at_ms).ok()?; }
        self.volley_damage_mark(&source_snapshot,target_id,actual_damage,committed_at_ms).ok()?;
        let adaptive_type = if accepted_hit && first_target_strike {
            self.adaptive_shield_target_type(&source_snapshot, target_id)
        } else { None };
        self.apply_damage_result_and_record_with_origin(
            Some(source_snapshot.source_id),
            &result,
            damage_fact,
            committed_at_ms,
            match policy {
                CommittedDamagePolicy::BasicAttack { .. } => HpChangeReason::BasicAttack,
                CommittedDamagePolicy::GeneralCommand | CommittedDamagePolicy::BypassShields => HpChangeReason::Command,
            },
            observation_origin,
        );

        let transformed = if let Some(plan) = revival {
            self.resolve_employee_revival(plan, committed_at_ms).ok()?
        } else { false };
        let killed_target = if let Some(plan) = last_will {
            self.resolve_last_will(plan, &source_snapshot, committed_at_ms).ok()?;
            self.units.get(&target_id).is_some_and(|u| u.lifecycle.is_dead())
        } else { killed_target };
        if actual_damage > 0 && !killed_target && !transformed {
            self.release_overheal_reserve(target_id, &source_snapshot.fact_source, committed_at_ms).ok()?;
        }
        if let Some(damage_type) = adaptive_type {
            self.adaptive_shield_after_hit(&source_snapshot, target_id, damage_type, committed_at_ms).ok()?;
        }
        if accepted_hit && first_target_strike && !killed_target && !transformed {
            self.equipment_attack_hit(&source_snapshot, target_id, committed_at_ms, committed_strike).ok()?;
        }
        if killed_target && self.units.get(&target_id).is_some_and(|u| u.lifecycle.is_dead()) {
            if let Some(capture) = equipment_kill {
                self.dispatch_equipment_kill(capture, committed_at_ms).ok()?;
            }
            self.shot_cohort_death(target_id, &source_snapshot, committed_at_ms).ok()?;
            self.equipment_form_kill(&source_snapshot, target_id, committed_at_ms).ok()?;
            self.equipment_death_burst_kill(&source_snapshot, target_id, committed_at_ms).ok()?;
        }
        self.relation_shields_after_damage(target_id, committed_at_ms).ok()?;
        if prepared_direct_kill && matches!(policy, CommittedDamagePolicy::BasicAttack { .. }) {
            self.authored_state_basic_kill(&source_snapshot, target_id, committed_at_ms).ok()?;
            self.prepared_basic_kill(&source_snapshot, target_id, committed_at_ms).ok()?;
            self.weapon_growth_kill(&source_snapshot, target_id, committed_at_ms).ok()?;
            self.equipment_attack_kill(&source_snapshot, target_id, committed_at_ms).ok()?;
        }
        if actual_damage > 0 && !killed_target && !transformed {
            self.owned_damage_status(&source_snapshot,target_id,committed_at_ms).ok()?;
            self.reactive_heal_after_damage(target_id, &source_snapshot, committed_at_ms).ok()?;
            self.authored_state_after_damage(target_id, &source_snapshot, actual_damage, committed_at_ms).ok()?;
        }
        if let Some(fact) = &mut pending_counter {
            self.execute_parry_counter(fact, source_snapshot.source_id, committed_at_ms)
                .ok()?;
        }
        if let Some(reflected) = reflection_snapshot {
            let prior_observation = self.authoring_direct_source.take();
            let prior_effect = self.recording_skill_effect_source.take();
            let prior_scope = self.begin_strike_scope();
            let _ = self.execute_committed_damage(CommittedDamageInput {
                target_id: source_snapshot.source_id,
                source_snapshot: reflected,
                committed_at_ms,
                policy: CommittedDamagePolicy::GeneralCommand,
            });
            self.end_strike_scope(prior_scope);
            self.recording_skill_effect_source = prior_effect;
            self.authoring_direct_source = prior_observation;
            crate::authoring::control::poll("passive_guard_after_reflection").ok()?;
        }
        let formula_commands = std::mem::take(&mut result.triggered_commands);
        if !formula_commands.is_empty() {
            command_batches.push(RuntimeCommandBatch::unsourced(formula_commands));
        }
        if !activation_commands.is_empty() {
            command_batches.push(RuntimeCommandBatch::unsourced(activation_commands));
        }
        if !command_batches.is_empty() {
            self.process_command_batches(command_batches, committed_at_ms);
        }

        Some(CommandExecutionSummary {
            accepted_hit_count: usize::from(accepted_hit),
            actual_hp_loss: u64::from(actual_damage),
            actual_damage_target_count: usize::from(actual_damage > 0),
            effective_hit_count: usize::from(effective_hit),
            killed_target_count: usize::from(killed_target),
        })
    }

    fn unit_tag(unit_id: UnitInstanceId) -> u64 {
        let mut high = [0u8; 8];
        let mut low = [0u8; 8];
        high.copy_from_slice(&unit_id.as_bytes()[..8]);
        low.copy_from_slice(&unit_id.as_bytes()[8..]);
        u64::from_be_bytes(high) ^ u64::from_be_bytes(low).rotate_left(17)
    }

    fn uuid_tag(uuid: uuid::Uuid) -> u64 {
        let mut high = [0u8; 8];
        let mut low = [0u8; 8];
        high.copy_from_slice(&uuid.as_bytes()[..8]);
        low.copy_from_slice(&uuid.as_bytes()[8..]);
        u64::from_be_bytes(high) ^ u64::from_be_bytes(low).rotate_left(31)
    }

    fn damage_source_tag(source: DamageSource) -> u64 {
        match source {
            DamageSource::BasicAttack => 1,
            DamageSource::Ability => 2,
            DamageSource::BuffTick => 3,
            DamageSource::Environment => 4,
        }
    }

    fn damage_type_tag(damage_type: crate::game::battle::damage::DamageType) -> u64 {
        match damage_type {
            crate::game::battle::damage::DamageType::Physical => 1,
            crate::game::battle::damage::DamageType::Magic => 2,
            crate::game::battle::damage::DamageType::True => 3,
        }
    }

    fn roll_kind_tag(kind: CombatRollKind) -> u64 {
        match kind {
            CombatRollKind::BasicAttackCrit => 1,
            CombatRollKind::SkillCrit => 2,
            CombatRollKind::StatusProc => 3,
            CombatRollKind::EnvironmentCrit => 4,
        }
    }

    fn damage_source_instance_id(
        &mut self,
        source_id: UnitInstanceId,
        source: DamageSource,
        damage_type: crate::game::battle::damage::DamageType,
        base_damage: u32,
        committed_at_ms: u64,
    ) -> uuid::Uuid {
        const DAMAGE_SOURCE_INSTANCE_NS: u64 = 0x444D_4753_5243_4944u64; // "DMGSRCID"

        let seed = self.seed
            ^ Self::unit_tag(source_id).rotate_left(11)
            ^ Self::damage_source_tag(source).rotate_left(23)
            ^ Self::damage_type_tag(damage_type).rotate_left(31)
            ^ u64::from(base_damage).rotate_left(41)
            ^ committed_at_ms.wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ self.damage_source_seq.rotate_left(53);

        let source_instance_id =
            determinism::uuid_v4_from_seed(seed, DAMAGE_SOURCE_INSTANCE_NS, self.damage_source_seq);
        self.damage_source_seq = self.damage_source_seq.wrapping_add(1);
        source_instance_id
    }

    fn damage_roll_percent(&self, identity: CombatRollIdentity) -> u8 {
        const DAMAGE_ROLL_NS: u64 = 0x444D_4752_4F4C_4C53u64; // "DMGROLLS"

        let mut seed = self.seed
            ^ Self::roll_kind_tag(identity.roll_kind).rotate_left(5)
            ^ Self::uuid_tag(identity.source_instance_id).rotate_left(17)
            ^ u64::from(identity.hit_index).rotate_left(47);

        if let Some(source_unit_id) = identity.source_unit_id {
            seed ^= Self::unit_tag(source_unit_id).rotate_left(23);
        }
        if let Some(target_unit_id) = identity.target_unit_id {
            seed ^= Self::unit_tag(target_unit_id).rotate_left(37);
        }

        determinism::uuid_v4_from_seed(seed, DAMAGE_ROLL_NS, u64::from(identity.hit_index))
            .as_bytes()[0]
            % 100
    }

    pub(in crate::game::battle::core) fn materialize_damage_source_snapshot_for_target(
        &self,
        snapshot: DamageSourceSnapshot,
        target_id: UnitInstanceId,
    ) -> DamageSourceSnapshot {
        self.materialize_damage_source_snapshot_for_target_hit(snapshot, target_id, 0)
    }

    pub(in crate::game::battle::core) fn materialize_damage_source_snapshot_for_target_hit(
        &self,
        mut snapshot: DamageSourceSnapshot,
        target_id: UnitInstanceId,
        hit_index: u32,
    ) -> DamageSourceSnapshot {
        snapshot.crit_roll_identity = snapshot
            .crit_roll_identity
            .with_target(target_id)
            .with_hit_index(hit_index);
        if snapshot.crit_roll_percent.is_none() {
            snapshot.crit_roll_percent =
                Some(self.damage_roll_percent(snapshot.crit_roll_identity));
        }
        snapshot
    }

    pub(in crate::game::battle::core) fn apply_damage_result_and_record(
        &mut self,
        source_instance_id: Option<UnitInstanceId>,
        result: &DamageResult,
        damage_fact: (
            std::sync::Arc<crate::game::battle::runtime_facts::DamageFact>,
            bool,
            bool,
        ),
        time_ms: u64,
        reason: HpChangeReason,
    ) {
        self.apply_damage_result_and_record_with_origin(
            source_instance_id, result, damage_fact, time_ms, reason, None,
        );
    }

    fn apply_damage_result_and_record_with_origin(
        &mut self,
        source_instance_id: Option<UnitInstanceId>,
        result: &DamageResult,
        damage_fact: (
            std::sync::Arc<crate::game::battle::runtime_facts::DamageFact>,
            bool,
            bool,
        ),
        time_ms: u64,
        reason: HpChangeReason,
        observation_origin: Option<&crate::game::battle::runtime_facts::RuntimeFactSource>,
    ) {
        let target_instance_id = result.target_id;
        let (target_owner, hp_before) = {
            let Some(target) = self.units.get(&target_instance_id) else {
                return;
            };
            if !target.is_active() {
                return;
            }
            (target.owner, target.stats.current_health)
        };

        if hp_before > 0 && result.final_damage > 0 {
            if self.reset_idle_hp_guard_damage(target_instance_id, time_ms).is_err() { return; }
        }
        let prepared_death = if hp_before.saturating_sub(result.final_damage) == 0 {
            match self.prepare_death_fact(source_instance_id, target_instance_id, time_ms) {
                Ok(value) => value,
                Err(_) => return,
            }
        } else {
            None
        };
        let reserved = match observation_origin {
            Some(origin) => self.reserve_authoring_observation_for_effect(origin, None),
            None => self.reserve_authoring_observation(),
        };
        let reservation = match reserved {
            Ok(reservation) => reservation,
            Err(_) => return,
        };
        let player_attack = source_instance_id.and_then(|id|self.units.get(&id))
            .is_some_and(|source|source.owner == crate::game::enums::Side::Player);
        let Some(target) = self.units.get_mut(&target_instance_id) else {
            return;
        };

        apply_damage_to_unit(&mut target.stats, result.final_damage);
        let hp_after = target.stats.current_health;
        self.performance_bonus.observe(
            player_attack && target.owner == crate::game::enums::Side::Opponent
                && Some(target.base_uuid) == self.performance_bonus.primary,
            result.damage_source, hp_before.saturating_sub(hp_after), target.stats.max_health);
        let (fact, first, first_positive) = damage_fact;
        let extent = fact.retained_extent;
        if self
            .runtime_facts
            .record_fact(super::runtime_facts::FactUpdate::Damage(
                fact.clone(),
                first,
                first_positive,
            ))
            .is_err()
        {
            return;
        }
        if self.record_lab_source(&fact.source,if fact.actual_hp_loss()>0 {crate::mod_validation::CoverageStage::ConsumedOrEffect}else{crate::mod_validation::CoverageStage::LegalNoop}).is_err(){return;}
        self.observe_effect_damage_result(
            fact.source.as_ref(),
            fact.actual_hp_loss(),
            first,
            first_positive,
        );
        if self
            .observe_causal_damage(&fact, result.damage_source, result.damage_type)
            .is_err()
        {
            return;
        }
        self.runtime_facts.release_fact_storage(extent);
        let delta = hp_after as i32 - hp_before as i32;
        self.commit_authoring_observation(
            reservation,
            time_ms,
            source_instance_id,
            target_instance_id,
            super::authoring_observation::AppliedChange::Hp {
                before: hp_before,
                after: hp_after,
                reason,
            },
        );

        self.record_event_log(
            time_ms,
            crate::game::battle::event_log::BattleLogEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                delta,
                hp_before,
                hp_after,
                reason,
                damage_source: Some(result.damage_source),
                damage_type: Some(result.damage_type),
                raw_damage: Some(result.raw_damage),
                final_damage: Some(result.final_damage),
                damage_breakdown: Some(result.breakdown.clone()),
                critical: Some(result.critical),
                feedback_tags: result.feedback_tags.clone(),
            },
        );

        if hp_after < hp_before && matches!(reason, HpChangeReason::BasicAttack) {
            self.grant_basic_attack_damage_received_resonance(
                target_instance_id,
                hp_before - hp_after,
                time_ms,
                hp_after > 0,
            );
        }

        if hp_after == 0
            && self
                .independent_objects
                .objects
                .contains_key(&target_instance_id)
        {
            self.finalize_unit_death(
                source_instance_id,
                target_instance_id,
                target_owner,
                time_ms,
            );
            return;
        }

        if let Some(prepared) = prepared_death {
            self.finalize_prepared_death(
                source_instance_id,
                target_instance_id,
                target_owner,
                time_ms,
                prepared,
                Some(fact),
            );
        }
    }

    pub(in crate::game::battle::core) fn apply_hp_delta_and_record(
        &mut self,
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        delta: i32,
        time_ms: u64,
        reason: HpChangeReason,
    ) {
        self.apply_hp_delta_with_reserve(source_instance_id, target_instance_id, delta, time_ms, reason, true);
    }

    pub(in crate::game::battle::core) fn apply_hp_delta_with_reserve(
        &mut self,
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        delta: i32,
        time_ms: u64,
        reason: HpChangeReason,
        store_overheal: bool,
    ) {
        self.apply_hp_delta_with_observation_origin(source_instance_id, target_instance_id,
            delta, time_ms, reason, store_overheal, None);
    }

    pub(super) fn apply_hp_delta_with_observation_origin(
        &mut self,
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        delta: i32,
        time_ms: u64,
        reason: HpChangeReason,
        store_overheal: bool,
        observation_origin: Option<&crate::game::battle::runtime_facts::RuntimeFactSource>,
    ) {
        let (target_owner, hp_before, max_health) = {
            let Some(target) = self.units.get(&target_instance_id) else {
                return;
            };
            if delta > 0
                && target.role == crate::game::battle::types::BattleUnitRole::IndependentWall
            {
                return;
            }
            if !target.is_active() {
                return;
            }
            (
                target.owner,
                target.stats.current_health,
                target.stats.max_health,
            )
        };

        let reservation = if delta == 0 {
            None
        } else {
            let reserved = match observation_origin {
                Some(origin) => self.reserve_authoring_observation_for_effect(origin, None),
                None => self.reserve_authoring_observation(),
            };
            match reserved {
                Ok(value) => value,
                Err(_) => return,
            }
        };
        let hp_after = if delta > 0 {
            hp_before
                .saturating_add(delta as u32)
                .min(max_health.max(1))
        } else {
            hp_before.saturating_sub(delta.unsigned_abs())
        };
        let revival = if hp_after == 0 {
            let incoming = self.runtime_fact_source.clone();
            match self.prepare_employee_revival(target_instance_id, delta.unsigned_abs(), time_ms,
                &incoming, &mut Default::default()) {
                Ok(plan) => plan,
                Err(_) => return,
            }
        } else { None };
        let prepared_death = if hp_after == 0 && revival.is_none() {
            match self.prepare_death_fact(source_instance_id, target_instance_id, time_ms) {
                Ok(value) => value,
                Err(_) => return,
            }
        } else {
            None
        };
        if self
            .record_heal_fact(
                source_instance_id,
                target_instance_id,
                i64::from(delta),
                hp_before,
                hp_after,
                time_ms,
                reason,
            )
            .is_err()
        {
            return;
        }
        if delta == 0 {
            return;
        }

        let Some(target) = self.units.get_mut(&target_instance_id) else {
            return;
        };

        target.stats.current_health = hp_after;
        if delta > 0 && store_overheal {
            let overheal = (delta as u32).saturating_sub(hp_after.saturating_sub(hp_before));
            if self.store_overheal_reserve(target_instance_id, overheal, time_ms).is_err() { return; }
        }

        let applied_delta = hp_after as i32 - hp_before as i32;
        self.commit_authoring_observation(
            reservation,
            time_ms,
            source_instance_id,
            target_instance_id,
            super::authoring_observation::AppliedChange::Hp {
                before: hp_before,
                after: hp_after,
                reason,
            },
        );
        self.record_event_log(
            time_ms,
            crate::game::battle::event_log::BattleLogEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                delta: applied_delta,
                hp_before,
                hp_after,
                reason,
                damage_source: None,
                damage_type: None,
                raw_damage: None,
                final_damage: None,
                damage_breakdown: None,
                critical: None,
                feedback_tags: Vec::new(),
            },
        );

        if hp_after < hp_before && matches!(reason, HpChangeReason::BasicAttack) {
            self.grant_basic_attack_damage_received_resonance(
                target_instance_id,
                hp_before - hp_after,
                time_ms,
                hp_after > 0,
            );
        }

        if let Some(plan) = revival {
            if self.resolve_employee_revival(plan, time_ms).is_err() { return; }
            return;
        }
        if hp_after == 0
            && self
                .independent_objects
                .objects
                .contains_key(&target_instance_id)
        {
            self.finalize_unit_death(
                source_instance_id,
                target_instance_id,
                target_owner,
                time_ms,
            );
            return;
        }

        if let Some(prepared) = prepared_death {
            self.finalize_prepared_death(
                source_instance_id,
                target_instance_id,
                target_owner,
                time_ms,
                prepared,
                None,
            );
        }
    }

    pub(in crate::game::battle::core) fn record_runtime_stat_hp_change_and_finalize(
        &mut self,
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        hp_before: u32,
        hp_after: u32,
        time_ms: u64,
        prepared_death: Option<super::runtime_facts::PreparedDeath>,
    ) -> bool {
        if self.reconcile_overheal_reserves().is_err() { return false; }
        if hp_before == hp_after {
            return false;
        }
        let Some(target_owner) = self
            .units
            .get(&target_instance_id)
            .filter(|target| target.is_active())
            .map(|target| target.owner)
        else {
            return false;
        };

        self.record_event_log(
            time_ms,
            crate::game::battle::event_log::BattleLogEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                delta: hp_after as i32 - hp_before as i32,
                hp_before,
                hp_after,
                reason: HpChangeReason::Command,
                damage_source: None,
                damage_type: None,
                raw_damage: None,
                final_damage: None,
                damage_breakdown: None,
                critical: None,
                feedback_tags: Vec::new(),
            },
        );

        if hp_after == 0
            && self
                .independent_objects
                .objects
                .contains_key(&target_instance_id)
        {
            self.finalize_unit_death(
                source_instance_id,
                target_instance_id,
                target_owner,
                time_ms,
            );
            return true;
        }

        if let Some(prepared) = prepared_death {
            self.finalize_prepared_death(
                source_instance_id,
                target_instance_id,
                target_owner,
                time_ms,
                prepared,
                None,
            );
            return true;
        }
        false
    }
}

impl BattleCore {
    /// A payment changes HP and emits CostFact directly. It never enters damage,
    /// healing, shield, parry or HP-floor arbitration. Employee revival may
    /// restore HP after the full lethal payment; it never discounts that cost.
    pub(super) fn pay_health_cost(
        &mut self,
        caster: UnitInstanceId,
        amount: u32,
        allow_lethal: bool,
        now: u64,
    ) -> Result<(), crate::game::behavior::GameError> {
        use crate::game::battle::{
            runtime_control::{reject, BattleRuntimeFault, RuntimeFaultCode},
            runtime_facts::{CostReason, FactResource, ResourceOwner},
        };
        let (owner, before) = self
            .units
            .get(&caster)
            .filter(|u| u.is_active())
            .map(|u| (u.owner, u.stats.current_health))
            .ok_or_else(|| {
                reject(BattleRuntimeFault {
                    code: RuntimeFaultCode::HealthCostUnavailable,
                    site: "health_cost_owner",
                    limit: None,
                    requested: Some(u64::from(amount)),
                })
            })?;
        let available = before.saturating_sub(u32::from(!allow_lethal));
        if amount > available || amount > i32::MAX as u32 {
            return Err(reject(BattleRuntimeFault {
                code: RuntimeFaultCode::HealthCostUnavailable,
                site: "health_cost_balance",
                limit: Some(u64::from(available)),
                requested: Some(u64::from(amount)),
            }));
        }
        let after = before - amount;
        let reservation = self.reserve_authoring_observation()?;
        let revival = if after == 0 {
            let incoming = self.runtime_fact_source.clone();
            self.prepare_employee_revival(caster, amount, now, &incoming, &mut Default::default())?
        } else { None };
        let prepared = if after == 0 && revival.is_none() {
            self.prepare_death_fact(Some(caster), caster, now)?
        } else {
            None
        };
        let source = crate::authoring::clone_shape::controlled_clone(&self.runtime_fact_source)?;
        self.record_resource_fact(
            ResourceOwner::Unit(caster),
            FactResource::Health,
            Some(CostReason::AuthoredHealth),
            source,
            -i64::from(amount),
            before,
            after,
            now,
        )?;
        self.units.get_mut(&caster).unwrap().stats.current_health = after;
        self.commit_authoring_observation(
            reservation,
            now,
            Some(caster),
            caster,
            super::authoring_observation::AppliedChange::Hp {
                before,
                after,
                reason: HpChangeReason::HealthCost,
            },
        );
        self.record_event_log(
            now,
            crate::game::battle::event_log::BattleLogEvent::HpChanged {
                source_instance_id: Some(caster),
                target_instance_id: caster,
                delta: -(amount as i32),
                hp_before: before,
                hp_after: after,
                reason: HpChangeReason::HealthCost,
                damage_source: None,
                damage_type: None,
                raw_damage: None,
                final_damage: None,
                damage_breakdown: None,
                critical: None,
                feedback_tags: Vec::new(),
            },
        );
        if let Some(plan) = revival { self.resolve_employee_revival(plan, now)?; }
        if let Some(prepared) = prepared {
            self.finalize_prepared_death(Some(caster), caster, owner, now, prepared, None);
        }
        crate::authoring::control::poll("health_cost_after_payment")?;
        crate::game::battle::runtime_control::poll()
    }
}

impl BattleCore {
    pub(super) fn add_basic_hp_loss(
        total: &mut u64,
        loss: u64,
    ) -> Result<(), crate::game::behavior::GameError> {
        *total = total.checked_add(loss).ok_or_else(|| {
            crate::game::battle::runtime_control::reject(
                crate::game::battle::runtime_control::BattleRuntimeFault {
                    code: crate::game::battle::runtime_control::RuntimeFaultCode::CounterOverflow,
                    site: "basic_actual_hp_sum",
                    limit: None,
                    requested: None,
                },
            )
        })?;
        Ok(())
    }
    pub(super) fn finish_basic_lifesteal(
        &mut self,
        snapshot: &DamageSourceSnapshot,
        actual_hp_loss: u64,
        now: u64,
    ) -> Result<(), crate::game::behavior::GameError> {
        if actual_hp_loss == 0
            || snapshot.source != DamageSource::BasicAttack
            || !self
                .units
                .get(&snapshot.source_id)
                .is_some_and(|u| u.is_active())
        {
            return Ok(());
        }
        let authored = self.equipment_basic_lifesteal(snapshot)?;
        crate::authoring::control::charge(
            snapshot.basic_lifesteal.len() as u64 + u64::from(authored.is_some()),
            "basic_lifesteal_settlement",
        )?;
        for lifesteal in snapshot.basic_lifesteal.iter().chain(authored.as_ref()) {
            let heal = ((u128::from(actual_hp_loss) * u128::from(lifesteal.percent) / 100)
                .min(u128::from(lifesteal.max_heal))) as u32;
            if heal == 0 {
                continue;
            }
            let mut source = crate::game::battle::runtime_facts::RuntimeFactSource {
                actor: Some(snapshot.source_id),
                equipment: lifesteal.equipment,
                ..Default::default()
            };
            if let Some(origin) = &lifesteal.fact_source {
                origin.admit_owner("sustained_lifesteal_heal_source")?;
                source = origin.observation();
            }
            source.admit_owner("basic_lifesteal_heal_source")?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<
                    crate::game::battle::damage::BattleCommand,
                >()),
                "basic_lifesteal_command",
            )?;
            let prior = std::mem::replace(&mut self.runtime_fact_source, source);
            self.process_commands(
                vec![crate::game::battle::damage::BattleCommand::ApplyHeal {
                    target_id: snapshot.source_id,
                    flat: heal as i32,
                    percent: 0,
                    source_id: Some(snapshot.source_id),
                }],
                now,
            );
            self.runtime_fact_source = prior;
            crate::authoring::control::poll("basic_lifesteal_after_heal")?;
        }
        Ok(())
    }
}
