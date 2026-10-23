use crate::authoring::control;
use crate::game::ability::AbilityActivationDef;
use crate::game::battle::cooldown::{CooldownSource, SourcedAbilityActivation, SourcedEffect};
use crate::game::battle::damage::{
    bonus_damage_raw_from_source_snapshot, BattleCommand, DamageSource, DamageSourceSnapshot,
};
use crate::game::battle::ids::UnitInstanceId;
use crate::game::stats::{
    classify_triggered_effect_execution, Effect, TriggerType, TriggeredEffectExecutionKind,
};
use uuid::Uuid;

use super::{
    commands::{RuntimeCommandBatch, TriggerAbilityContext},
    BattleCore, RuntimeItem,
};

#[derive(Debug, Clone, Copy)]
pub(in crate::game::battle::core) struct TriggerOccurrenceContext<'a> {
    pub(in crate::game::battle::core) trigger_type: TriggerType,
    pub(in crate::game::battle::core) trigger_unit_id: UnitInstanceId,
    pub(in crate::game::battle::core) counterpart_unit_id: Option<UnitInstanceId>,
    pub(in crate::game::battle::core) target_id: Option<UnitInstanceId>,
    pub(in crate::game::battle::core) occurrence_id: Uuid,
    pub(in crate::game::battle::core) occurrence_index: u32,
    pub(in crate::game::battle::core) allow_dead_caster: bool,
    pub(in crate::game::battle::core) committed_hit: Option<&'a DamageSourceSnapshot>,
    pub(in crate::game::battle::core) time_ms: u64,
}

#[derive(Debug, Default)]
pub(in crate::game::battle::core) struct TriggerOccurrencePlan {
    pub(in crate::game::battle::core) current_hit_effects: Vec<SourcedEffect>,
    pub(in crate::game::battle::core) effect_batches: Vec<RuntimeCommandBatch>,
    pub(in crate::game::battle::core) activation_commands: Vec<BattleCommand>,
}

impl TriggerOccurrencePlan {
    #[cfg(test)]
    pub(in crate::game::battle::core) fn into_commands(self) -> Vec<BattleCommand> {
        let mut commands: Vec<_> = self
            .effect_batches
            .into_iter()
            .flat_map(|batch| batch.commands)
            .collect();
        commands.extend(self.activation_commands);
        commands
    }

    pub(in crate::game::battle::core) fn into_effect_batches_and_activations(
        self,
    ) -> (Vec<RuntimeCommandBatch>, Vec<BattleCommand>) {
        (self.effect_batches, self.activation_commands)
    }

    pub(in crate::game::battle::core) fn into_command_batches(self) -> Vec<RuntimeCommandBatch> {
        let (mut batches, activation_commands) = self.into_effect_batches_and_activations();
        if !activation_commands.is_empty() {
            let batch = RuntimeCommandBatch::unsourced(activation_commands);
            if batch
                .fact_source
                .admit_owner("trigger_activation_batch")
                .is_err()
            {
                return Vec::new();
            }
            batches.push(batch);
        }
        batches
    }
}

#[derive(Debug, Default)]
struct ItemTriggerPayload {
    effects: Vec<SourcedEffect>,
    activations: Vec<SourcedAbilityActivation>,
}

impl BattleCore {
    fn collect_item_trigger_payload(
        &self,
        unit_instance_id: UnitInstanceId,
        trigger: TriggerType,
    ) -> ItemTriggerPayload {
        let mut payload = ItemTriggerPayload::default();
        let count = self.items.len() as u64;
        if control::charge(
            count
                .saturating_mul(u64::from(usize::BITS - self.items.len().leading_zeros()) + 2)
                .saturating_add(1),
            "trigger_item_scan",
        )
        .is_err()
            || self
                .admit_authoring_runtime_growth(0, "trigger_item_storage")
                .is_err()
        {
            return payload;
        }
        let mut items: Vec<&RuntimeItem> = self
            .items
            .values()
            .filter(|item| item.owner_unit_instance == unit_instance_id)
            .collect();
        items.sort_by(|left, right| {
            left.instance_id
                .as_bytes()
                .cmp(right.instance_id.as_bytes())
        });

        for item in items {
            if control::poll("trigger_item").is_err() {
                return ItemTriggerPayload::default();
            }
            let Some(metadata) = self.game_data.equipment_data.get_by_uuid(&item.base_uuid) else {
                continue;
            };
            let source = CooldownSource::Item {
                item_instance_id: item.instance_id,
            };

            let triggered_count = metadata.triggered_effects.get(&trigger).map_or(0, Vec::len);
            if control::admit(
                control::Resource::EffectCommands,
                payload
                    .effects
                    .len()
                    .saturating_add(payload.activations.len()),
                triggered_count.saturating_add(metadata.ability_activations.len()),
                "trigger_payload",
            )
            .is_err()
                || control::charge_serialized(
                    &metadata.triggered_effects.get(&trigger),
                    "trigger_effect_copy",
                )
                .is_err()
                || control::charge_serialized(
                    &metadata.ability_activations,
                    "trigger_activation_copy",
                )
                .is_err()
            {
                return ItemTriggerPayload::default();
            }

            if crate::game::battle::runtime_control::admit_retained_growth(
                0,
                triggered_count
                    .checked_add(metadata.ability_activations.len())
                    .and_then(|n| {
                        n.checked_mul(std::mem::size_of::<
                            Option<crate::game::battle::runtime_facts::EquipmentSourceSnapshot>,
                        >())
                    }),
                "trigger_provider_payload",
            )
            .is_err()
            {
                return ItemTriggerPayload::default();
            }
            if let Some(triggered) = metadata.triggered_effects.get(&trigger) {
                payload
                    .effects
                    .extend(triggered.iter().cloned().map(|triggered_effect| {
                        let (target, effect) = triggered_effect.into_parts();
                        SourcedEffect {
                            equipment_source: Some(item.source_snapshot()),
                            source,
                            target,
                            effect,
                        }
                    }));
            }
            payload.activations.extend(
                metadata
                    .ability_activations
                    .iter()
                    .enumerate()
                    .filter(|binding| {
                        matches!(
                            &binding.1.activation,
                            AbilityActivationDef::TriggerProc {
                                trigger: activation_trigger,
                                ..
                            } if *activation_trigger == trigger
                        )
                    })
                    .map(|(binding_index, binding)| SourcedAbilityActivation {
                        equipment_source: Some(item.source_snapshot()),
                        source,
                        binding: binding.clone(),
                        binding_index,
                    }),
            );
        }

        payload
    }

    pub(super) fn collect_all_triggers(
        &self,
        unit_instance_id: UnitInstanceId,
        trigger: TriggerType,
    ) -> Vec<SourcedEffect> {
        if !self.units.contains_key(&unit_instance_id) {
            return Vec::new();
        }
        self.collect_item_trigger_payload(unit_instance_id, trigger)
            .effects
    }

    pub(super) fn collect_current_hit_triggers(
        &self,
        unit_instance_id: UnitInstanceId,
        trigger: TriggerType,
    ) -> Vec<SourcedEffect> {
        if !self.units.contains_key(&unit_instance_id) {
            return Vec::new();
        }
        self.collect_item_trigger_payload(unit_instance_id, trigger)
            .effects
            .into_iter()
            .filter(|sourced| Self::is_current_hit_effect(trigger, sourced))
            .collect()
    }

    fn is_current_hit_effect(trigger: TriggerType, sourced: &SourcedEffect) -> bool {
        matches!(
            classify_triggered_effect_execution(trigger, sourced.target, &sourced.effect),
            Some(TriggeredEffectExecutionKind::CurrentHitDamage)
        )
    }

    fn retaliation_command(
        &mut self,
        context: TriggerOccurrenceContext<'_>,
        sourced: &SourcedEffect,
    ) -> Option<BattleCommand> {
        if !matches!(
            classify_triggered_effect_execution(
                context.trigger_type,
                sourced.target,
                &sourced.effect,
            ),
            Some(TriggeredEffectExecutionKind::RetaliationDamage)
        ) {
            return None;
        }
        let Effect::BonusDamage {
            flat,
            percent,
            damage_type,
        } = sourced.effect
        else {
            return None;
        };
        let counterpart_id = context.counterpart_unit_id?;
        if !self
            .units
            .get(&counterpart_id)
            .is_some_and(|unit| unit.is_active())
        {
            return None;
        }
        let committed_hit = context.committed_hit?;
        let raw_damage =
            bonus_damage_raw_from_source_snapshot(committed_hit, flat, percent, damage_type);
        if raw_damage == 0 {
            return None;
        }
        let mut source_snapshot = self.damage_source_snapshot_for_unit(
            context.trigger_unit_id,
            counterpart_id,
            DamageSource::Environment,
            damage_type,
            raw_damage,
            Default::default(),
            0,
            context.time_ms,
            false,
        )?;
        source_snapshot.fact_source = crate::game::battle::runtime_facts::RuntimeFactSource {
            actor: Some(context.trigger_unit_id),
            equipment: sourced.equipment_source,
            ..Default::default()
        };
        source_snapshot.fact_source = source_snapshot
            .fact_source
            .inherit_causal_context(&self.runtime_fact_source)
            .ok()?;
        Some(BattleCommand::ApplyDamage {
            target_id: counterpart_id,
            source_snapshot,
        })
    }

    pub(in crate::game::battle::core) fn dispatch_item_trigger_occurrence(
        &mut self,
        context: TriggerOccurrenceContext<'_>,
    ) -> TriggerOccurrencePlan {
        if !self.units.contains_key(&context.trigger_unit_id) {
            return TriggerOccurrencePlan::default();
        }
        let payload =
            self.collect_item_trigger_payload(context.trigger_unit_id, context.trigger_type);
        let mut plan = TriggerOccurrencePlan::default();
        let effect_context = super::commands::TriggerEffectContext {
            trigger_unit_id: context.trigger_unit_id,
            counterpart_unit_id: context.counterpart_unit_id,
        };

        for sourced in payload.effects {
            let equipment = sourced.equipment_source;
            let commands = match classify_triggered_effect_execution(
                context.trigger_type,
                sourced.target,
                &sourced.effect,
            ) {
                Some(TriggeredEffectExecutionKind::CurrentHitDamage) => {
                    plan.current_hit_effects.push(sourced);
                    continue;
                }
                Some(TriggeredEffectExecutionKind::RetaliationDamage) => self
                    .retaliation_command(context, &sourced)
                    .into_iter()
                    .collect(),
                Some(TriggeredEffectExecutionKind::TargetedCommand) => {
                    self.trigger_commands_from_effects(vec![sourced], effect_context)
                }
                Some(
                    TriggeredEffectExecutionKind::BasicLifesteal
                    | TriggeredEffectExecutionKind::PermanentModifier,
                )
                | None => continue,
            };
            if !commands.is_empty() {
                let mut batch =
                    RuntimeCommandBatch::with_modifier_source(context.trigger_unit_id, commands);
                batch.fact_source.equipment = equipment;
                if batch
                    .fact_source
                    .admit_owner("trigger_command_batch")
                    .is_err()
                {
                    return TriggerOccurrencePlan::default();
                }
                plan.effect_batches.push(batch);
            }
        }

        plan.activation_commands = Self::activation_commands_from_bindings(
            payload.activations,
            context.trigger_unit_id,
            TriggerAbilityContext {
                trigger_type: context.trigger_type,
                trigger_unit_id: context.trigger_unit_id,
                counterpart_unit_id: context.counterpart_unit_id,
                target_id: context.target_id,
                occurrence_id: context.occurrence_id,
                occurrence_index: context.occurrence_index,
            },
        );
        if context.allow_dead_caster {
            for command in &mut plan.activation_commands {
                if let BattleCommand::TriggerAbility {
                    allow_dead_caster, ..
                } = command
                {
                    *allow_dead_caster = true;
                }
            }
        }

        plan
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::*;
    use crate::game::{
        ability::AbilityActivationBinding,
        battle::scenario::BattleScenario,
        data::{
            equipment_data::{EquipmentMetadata, EquipmentType},
            GameDataBuilder,
        },
        enums::{RiskLevel, Side},
        stats::{Effect, TriggerEffectTarget, TriggeredEffect},
    };

    #[test]
    fn authoring_control_trigger_payload_preserves_runtime_order_and_bounds_combined_copy() {
        let base_uuid = Uuid::from_u128(200);
        let live = crate::game::data::ValidatedGameData::load_live_embedded()
            .expect("official data must provide valid trigger fixture references");
        let buff_name = live.buff_data.buffs[0].name.clone();
        let ability_id = live.skill_data.skills[0].id.clone();
        let mut triggered = HashMap::new();
        triggered.insert(
            TriggerType::OnHit,
            vec![TriggeredEffect::legacy(Effect::ApplyBuff {
                buff_id: buff_name,
                duration_ms: 50,
            })],
        );
        let equipment = EquipmentMetadata {
            content_family: None,
            id: "trigger_item".to_string(),
            uuid: base_uuid,
            name: "Trigger Item".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 1,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: triggered,
            ability_activations: vec![AbilityActivationBinding {
                ability_id,
                activation: AbilityActivationDef::TriggerProc {
                    trigger: TriggerType::OnHit,
                    proc_chance_percent: 100,
                    internal_cooldown_ms: 0,
                    max_triggers_per_battle: None,
                },
            }],
            weapon_profile: Some(Default::default()),
            event_check_modifiers: vec![],
            event_choice_keys: vec![],
        };
        let game_data = GameDataBuilder::empty()
            .with_equipment(vec![equipment])
            .with_buff_data(std::sync::Arc::clone(&live.buff_data))
            .with_skill_data(std::sync::Arc::clone(&live.skill_data))
            .with_boss_runtime_data(std::sync::Arc::clone(&live.boss_runtime_data))
            .build_arc();
        let mut core = BattleCore::new_from_scenario(BattleScenario::empty((4, 4)), game_data, 123);
        let unit_instance_id = UnitInstanceId::from(Uuid::from_u128(10));
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        for instance_id in [second, first] {
            core.items.insert(
                instance_id,
                RuntimeItem {
            weapon_stage: Default::default(),
                    persistent_copy_uuid: Some(Uuid::from_u128(instance_id.as_u128() + 100)),
                    instance_id,
                    owner: Side::Player,
                    owner_unit_instance: unit_instance_id,
                    base_uuid,
                },
            );
        }

        let payload = core.collect_item_trigger_payload(unit_instance_id, TriggerType::OnHit);
        use crate::authoring::control::{CancellationToken, ExecutionLimits, ExecutionPhase};
        let exact = ExecutionLimits {
            max_effect_commands: 4,
            ..Default::default()
        };
        let (controlled, usage) = control::run(
            &exact,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || core.collect_item_trigger_payload(unit_instance_id, TriggerType::OnHit),
        )
        .unwrap();
        assert_eq!(format!("{controlled:?}"), format!("{payload:?}"));
        assert_eq!(usage.effect_commands_peak, 4);
        let error = control::run(
            &ExecutionLimits {
                max_effect_commands: 3,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                let rejected =
                    core.collect_item_trigger_payload(unit_instance_id, TriggerType::OnHit);
                assert!(rejected.effects.is_empty());
                assert!(rejected.activations.is_empty());
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "authoring_effect_command_limit");
        assert_eq!(error.site, "trigger_payload");
        assert_eq!(error.requested, Some(4));
        assert_eq!(core.items.len(), 2);
        let effects = payload.effects;

        assert_eq!(effects.len(), 2);
        for (index, runtime_id) in [first, second].into_iter().enumerate() {
            let expected = crate::game::battle::runtime_facts::EquipmentSourceSnapshot {
                definition_uuid: base_uuid,
                runtime_item_instance_id: runtime_id,
                persistent_copy_uuid: Some(Uuid::from_u128(runtime_id.as_u128() + 100)),
            };
            assert_eq!(effects[index].equipment_source, Some(expected));
            assert_eq!(payload.activations[index].equipment_source, Some(expected));
        }
        assert!(matches!(
            effects.as_slice(),
            [
                SourcedEffect {
                    source: CooldownSource::Item { item_instance_id: left },
                    target: TriggerEffectTarget::SelfUnit,
                    ..
                },
                SourcedEffect {
                    source: CooldownSource::Item { item_instance_id: right },
                    target: TriggerEffectTarget::SelfUnit,
                    ..
                }
            ] if *left == first && *right == second
        ));
        assert!(matches!(
            payload.activations.as_slice(),
            [
                SourcedAbilityActivation {
                    source: CooldownSource::Item { item_instance_id: left },
                    binding_index: 0,
                    ..
                },
                SourcedAbilityActivation {
                    source: CooldownSource::Item { item_instance_id: right },
                    binding_index: 0,
                    ..
                }
            ] if *left == first && *right == second
        ));
    }
}
