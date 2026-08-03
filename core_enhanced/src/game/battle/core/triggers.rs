use crate::game::ability::AbilityActivationDef;
use crate::game::battle::cooldown::{CooldownSource, SourcedAbilityActivation, SourcedEffect};
use crate::game::battle::ids::UnitInstanceId;
use crate::game::stats::TriggerType;

use super::{BattleCore, RuntimeItem, TriggerSource};

impl BattleCore {
    pub(super) fn collect_triggers(
        &self,
        source: TriggerSource,
        trigger: TriggerType,
    ) -> Vec<SourcedEffect> {
        let mut effects = Vec::new();
        match source {
            TriggerSource::Item { unit_instance_id } => {
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
                    if let Some(metadata) =
                        self.game_data.equipment_data.get_by_uuid(&item.base_uuid)
                    {
                        if let Some(triggered) = metadata.triggered_effects.get(&trigger) {
                            effects.extend(triggered.iter().cloned().map(|triggered_effect| {
                                let (target, effect) = triggered_effect.into_parts();
                                SourcedEffect {
                                    source: CooldownSource::Item {
                                        item_instance_id: item.instance_id,
                                    },
                                    target,
                                    effect,
                                }
                            }));
                        }
                    }
                }
            }
        }
        effects
    }

    pub(super) fn collect_all_triggers(
        &self,
        unit_instance_id: UnitInstanceId,
        trigger: TriggerType,
    ) -> Vec<SourcedEffect> {
        if !self.units.contains_key(&unit_instance_id) {
            return Vec::new();
        }
        self.collect_triggers(TriggerSource::Item { unit_instance_id }, trigger)
    }

    pub(super) fn collect_trigger_activations(
        &self,
        source: TriggerSource,
        trigger: TriggerType,
    ) -> Vec<SourcedAbilityActivation> {
        let mut activations = Vec::new();
        match source {
            TriggerSource::Item { unit_instance_id } => {
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
                    if let Some(metadata) =
                        self.game_data.equipment_data.get_by_uuid(&item.base_uuid)
                    {
                        activations.extend(
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
                                    source: CooldownSource::Item {
                                        item_instance_id: item.instance_id,
                                    },
                                    binding: binding.clone(),
                                    binding_index,
                                }),
                        );
                    }
                }
            }
        }
        activations
    }

    pub(super) fn collect_all_trigger_activations(
        &self,
        unit_instance_id: UnitInstanceId,
        trigger: TriggerType,
    ) -> Vec<SourcedAbilityActivation> {
        if !self.units.contains_key(&unit_instance_id) {
            return Vec::new();
        }
        self.collect_trigger_activations(TriggerSource::Item { unit_instance_id }, trigger)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::*;
    use crate::game::{
        battle::scenario::BattleScenario,
        data::{
            equipment_data::{EquipmentMetadata, EquipmentType},
            GameDataBuilder,
        },
        enums::{RiskLevel, Side},
        stats::{Effect, TriggerEffectTarget, TriggeredEffect},
    };

    #[test]
    fn collect_triggers_sorts_equipment_instances_by_runtime_identity() {
        let base_uuid = Uuid::from_u128(200);
        let mut triggered = HashMap::new();
        triggered.insert(
            TriggerType::OnHit,
            vec![TriggeredEffect::legacy(Effect::ApplyBuff {
                buff_id: "on_hit_buff".to_string(),
                duration_ms: 50,
            })],
        );
        let equipment = EquipmentMetadata {
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
            ability_activations: vec![],
            weapon_profile: Some(Default::default()),
            event_check_modifiers: vec![],
            event_choice_keys: vec![],
        };
        let game_data = GameDataBuilder::empty()
            .with_equipment(vec![equipment])
            .build_arc();
        let mut core = BattleCore::new_from_scenario(BattleScenario::empty((4, 4)), game_data, 123);
        let unit_instance_id = UnitInstanceId::from(Uuid::from_u128(10));
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        for instance_id in [second, first] {
            core.items.insert(
                instance_id,
                RuntimeItem {
                    instance_id,
                    owner: Side::Player,
                    owner_unit_instance: unit_instance_id,
                    base_uuid,
                },
            );
        }

        let effects =
            core.collect_triggers(TriggerSource::Item { unit_instance_id }, TriggerType::OnHit);

        assert_eq!(effects.len(), 2);
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
    }
}
