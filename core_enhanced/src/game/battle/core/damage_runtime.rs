use crate::game::{
    battle::{
        cooldown::SourcedEffect,
        core::{commands::TriggerEffectContext, BattleCore},
        damage::{
            apply_damage_to_unit, calculate_damage, CombatRollIdentity, CombatRollKind,
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
pub(in crate::game::battle::core) struct AttackTargetSnapshot {
    pub(in crate::game::battle::core) instance_id: UnitInstanceId,
    pub(in crate::game::battle::core) owner: crate::game::enums::Side,
    pub(in crate::game::battle::core) defense: i32,
    pub(in crate::game::battle::core) magic_resist: i32,
    pub(in crate::game::battle::core) incoming_damage_modifiers: DamageModifiers,
    pub(in crate::game::battle::core) current_hp: u32,
    pub(in crate::game::battle::core) max_hp: u32,
}

#[derive(Debug, Clone)]
pub(in crate::game::battle::core) struct BasicAttackDamageSnapshot {
    pub(in crate::game::battle::core) source: DamageSourceSnapshot,
    pub(in crate::game::battle::core) target: AttackTargetSnapshot,
}

impl BattleCore {
    fn damage_effect_snapshot_from_triggers(
        effects: Vec<SourcedEffect>,
    ) -> (DamageModifiers, Vec<DamageBonusSnapshot>) {
        let mut modifiers = DamageModifiers::default();
        let mut bonus_damage = Vec::new();
        for sourced in effects {
            match sourced.effect {
                Effect::ModifyDamage(effect_modifiers) => {
                    modifiers = modifiers.merge(effect_modifiers);
                }
                Effect::BonusDamage {
                    flat,
                    percent,
                    damage_type,
                } => {
                    bonus_damage.push(DamageBonusSnapshot {
                        flat,
                        percent,
                        damage_type,
                    });
                }
                Effect::Modifier(_) | Effect::Heal { .. } | Effect::ApplyBuff { .. } => {}
            }
        }
        (modifiers, bonus_damage)
    }

    pub(in crate::game::battle::core) fn damage_effects_from_source_snapshot(
        snapshot: &DamageSourceSnapshot,
    ) -> Vec<SourcedEffect> {
        let mut effects = Vec::new();
        if snapshot.on_attack_modifiers != DamageModifiers::default() {
            effects.push(SourcedEffect {
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
                    source: crate::game::battle::cooldown::CooldownSource::Unit {
                        unit_instance_id: snapshot.source_id,
                    },
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
        let (source_side, source_attack) = self
            .units
            .get(&source_id)
            .map(|unit| (unit.owner, unit.stats.attack))
            .or_else(|| {
                self.graveyard
                    .get(&source_id)
                    .map(|unit| (unit.owner, unit.stats.attack))
            })?;
        let (on_attack_modifiers, on_attack_bonus_damage) = if include_on_attack_damage_effects {
            Self::damage_effect_snapshot_from_triggers(
                self.collect_all_triggers(source_id, TriggerType::OnAttack),
            )
        } else {
            (DamageModifiers::default(), Vec::new())
        };

        Some(DamageSourceSnapshot {
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
            on_attack_modifiers,
            on_attack_bonus_damage,
        })
    }

    pub(in crate::game::battle::core) fn calculate_basic_attack_damage_snapshot(
        &mut self,
        snapshot: BasicAttackDamageSnapshot,
    ) -> DamageResult {
        let on_hit_effects =
            self.collect_all_triggers(snapshot.target.instance_id, TriggerType::OnHit);
        let snapshot_on_attack_effects =
            Self::damage_effects_from_source_snapshot(&snapshot.source);
        let source_live = self
            .units
            .get(&snapshot.source.source_id)
            .is_some_and(|unit| unit.is_active());

        let mut trigger_effect_commands = Vec::new();
        if source_live {
            let on_attack_effects =
                self.collect_all_triggers(snapshot.source.source_id, TriggerType::OnAttack);
            trigger_effect_commands.extend(self.trigger_commands_from_effects(
                on_attack_effects,
                TriggerEffectContext {
                    trigger_unit_id: snapshot.source.source_id,
                    counterpart_unit_id: Some(snapshot.target.instance_id),
                },
                Some(snapshot.target.instance_id),
                false,
            ));
        }
        trigger_effect_commands.extend(self.trigger_commands_from_effects(
            on_hit_effects.clone(),
            TriggerEffectContext {
                trigger_unit_id: snapshot.target.instance_id,
                counterpart_unit_id: Some(snapshot.source.source_id),
            },
            Some(snapshot.source.source_id),
            false,
        ));
        let ctx = DamageContext {
            attacker_side: snapshot.source.source_side,
            target_side: snapshot.target.owner,
            attacker_attack: snapshot.source.source_attack,
            target_armor: snapshot.target.defense,
            target_magic_resist: snapshot.target.magic_resist,
            target_incoming_modifiers: snapshot.target.incoming_damage_modifiers,
            target_current_hp: snapshot.target.current_hp,
            target_max_hp: snapshot.target.max_hp,
            on_attack_effects: &snapshot_on_attack_effects,
            on_hit_effects: &on_hit_effects,
        };

        let request = snapshot.source.request(snapshot.target.instance_id);

        let mut result = calculate_damage(&request, &ctx);
        result
            .triggered_commands
            .splice(0..0, trigger_effect_commands);
        result
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
        time_ms: u64,
        reason: HpChangeReason,
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

        let Some(target) = self.units.get_mut(&target_instance_id) else {
            return;
        };

        apply_damage_to_unit(&mut target.stats, result.final_damage);
        let hp_after = target.stats.current_health;
        let delta = hp_after as i32 - hp_before as i32;

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

        if hp_after == 0 {
            self.finalize_unit_death(
                source_instance_id,
                target_instance_id,
                target_owner,
                time_ms,
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
        if delta == 0 {
            return;
        }

        let (target_owner, hp_before, max_health) = {
            let Some(target) = self.units.get(&target_instance_id) else {
                return;
            };
            if !target.is_active() {
                return;
            }
            (
                target.owner,
                target.stats.current_health,
                target.stats.max_health,
            )
        };

        let Some(target) = self.units.get_mut(&target_instance_id) else {
            return;
        };

        let hp_after = if delta > 0 {
            hp_before
                .saturating_add(delta as u32)
                .min(max_health.max(1))
        } else {
            hp_before.saturating_sub(delta.unsigned_abs())
        };
        target.stats.current_health = hp_after;

        let applied_delta = hp_after as i32 - hp_before as i32;
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

        if hp_after == 0 {
            self.finalize_unit_death(
                source_instance_id,
                target_instance_id,
                target_owner,
                time_ms,
            );
        }
    }
}
