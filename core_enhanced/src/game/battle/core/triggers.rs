use crate::game::ability::AbilityActivationDef;
use crate::game::battle::cooldown::{CooldownSource, SourcedAbilityActivation, SourcedEffect};
use crate::game::battle::ids::UnitInstanceId;
use crate::game::stats::TriggerType;

use super::{BattleCore, RuntimeArtifact, RuntimeItem, TriggerSource};

impl BattleCore {
    pub(super) fn collect_triggers(
        &self,
        source: TriggerSource,
        trigger: TriggerType,
    ) -> Vec<SourcedEffect> {
        let mut effects: Vec<SourcedEffect> = Vec::new();

        match source {
            TriggerSource::Artifact { side } => {
                let mut artifacts: Vec<&RuntimeArtifact> = self
                    .artifacts
                    .values()
                    .filter(|a| a.owner == side)
                    .collect();
                artifacts.sort_by(|a, b| a.instance_id.as_bytes().cmp(b.instance_id.as_bytes()));

                for artifact in artifacts {
                    if let Some(metadata) = self
                        .game_data
                        .artifact_data
                        .get_by_uuid(&artifact.base_uuid)
                    {
                        if let Some(triggered) = metadata.triggered_effects.get(&trigger) {
                            effects.extend(triggered.iter().cloned().map(|triggered_effect| {
                                let (target, effect) = triggered_effect.into_parts();
                                SourcedEffect {
                                    source: CooldownSource::Artifact {
                                        artifact_instance_id: artifact.instance_id,
                                    },
                                    target,
                                    effect,
                                }
                            }));
                        }
                    }
                }
            }
            TriggerSource::Item { unit_instance_id } => {
                let mut items: Vec<&RuntimeItem> = self
                    .items
                    .values()
                    .filter(|i| i.owner_unit_instance == unit_instance_id)
                    .collect();
                items.sort_by(|a, b| a.instance_id.as_bytes().cmp(b.instance_id.as_bytes()));

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
        let Some(unit) = self.units.get(&unit_instance_id) else {
            return Vec::new();
        };

        let mut effects =
            self.collect_triggers(TriggerSource::Artifact { side: unit.owner }, trigger);
        effects.extend(self.collect_triggers(TriggerSource::Item { unit_instance_id }, trigger));

        effects
    }

    pub(super) fn collect_trigger_activations(
        &self,
        source: TriggerSource,
        trigger: TriggerType,
    ) -> Vec<SourcedAbilityActivation> {
        let mut activations = Vec::new();

        match source {
            TriggerSource::Artifact { side } => {
                let mut artifacts: Vec<&RuntimeArtifact> = self
                    .artifacts
                    .values()
                    .filter(|a| a.owner == side)
                    .collect();
                artifacts.sort_by(|a, b| a.instance_id.as_bytes().cmp(b.instance_id.as_bytes()));

                for artifact in artifacts {
                    if let Some(metadata) = self
                        .game_data
                        .artifact_data
                        .get_by_uuid(&artifact.base_uuid)
                    {
                        activations.extend(
                            metadata
                                .ability_activations
                                .iter()
                                .enumerate()
                                .filter(|binding| {
                                    matches!(
                                        &binding.1.activation,
                                        AbilityActivationDef::TriggerProc { trigger: activation_trigger, .. }
                                        if *activation_trigger == trigger
                                    )
                                })
                                .map(|(binding_index, binding)| SourcedAbilityActivation {
                                    source: CooldownSource::Artifact {
                                        artifact_instance_id: artifact.instance_id,
                                    },
                                    binding: binding.clone(),
                                    binding_index,
                                }),
                        );
                    }
                }
            }
            TriggerSource::Item { unit_instance_id } => {
                let mut items: Vec<&RuntimeItem> = self
                    .items
                    .values()
                    .filter(|i| i.owner_unit_instance == unit_instance_id)
                    .collect();
                items.sort_by(|a, b| a.instance_id.as_bytes().cmp(b.instance_id.as_bytes()));

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
                                        AbilityActivationDef::TriggerProc { trigger: activation_trigger, .. }
                                        if *activation_trigger == trigger
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
        let Some(unit) = self.units.get(&unit_instance_id) else {
            return Vec::new();
        };

        let mut activations =
            self.collect_trigger_activations(TriggerSource::Artifact { side: unit.owner }, trigger);
        activations.extend(
            self.collect_trigger_activations(TriggerSource::Item { unit_instance_id }, trigger),
        );

        activations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::battle::core::types::{RuntimeArtifact, RuntimeItem, RuntimeUnit};
    use crate::game::battle::timeline::{TimelineCause, TimelineRootCause};
    use crate::game::battle::types::PlayerDeckInfo;
    use crate::game::data::{
        abnormality_data::AbnormalityDatabase, artifact_data::ArtifactDatabase,
        bonus_data::BonusDatabase, equipment_data::EquipmentDatabase, event_pools::EventPhasePool,
        event_pools::EventPoolConfig, pve_data::PveEncounterDatabase,
        random_event_data::RandomEventDatabase, shop_data::ShopDatabase, skill_data::SkillDatabase,
        GameDataBase,
    };
    use crate::game::enums::Side;
    use crate::game::stats::{
        Effect, StatId, StatModifier, StatModifierKind, TriggerEffectTarget, TriggeredEffect,
        UnitStats,
    };
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;
    use uuid::Uuid;

    fn empty_deck() -> PlayerDeckInfo {
        PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        }
    }

    fn game_data_with(
        artifacts: Vec<crate::game::data::artifact_data::ArtifactMetadata>,
        equipments: Vec<crate::game::data::equipment_data::EquipmentMetadata>,
    ) -> Arc<GameDataBase> {
        let pool = EventPhasePool {
            shops: vec![],
            bonuses: vec![],
            random_events: vec![],
        };
        let event_pools = EventPoolConfig {
            dawn: pool.clone(),
            noon: pool.clone(),
            dusk: pool.clone(),
            midnight: pool.clone(),
            white: pool,
        };

        Arc::new(GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![])),
            artifact_data: Arc::new(ArtifactDatabase::new(artifacts)),
            equipment_data: Arc::new(EquipmentDatabase::new(equipments)),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools,
        }))
    }

    fn new_core(game_data: Arc<GameDataBase>) -> BattleCore {
        let deck = empty_deck();
        BattleCore::new(&deck, &deck, game_data, (4, 4), 123)
    }

    fn runtime_unit(id: u128, owner: Side) -> RuntimeUnit {
        RuntimeUnit {
            instance_id: UnitInstanceId::from(Uuid::from_u128(id)),
            owner,
            base_uuid: Uuid::nil(),
            stats: UnitStats::with_values(10, 10, 1, 0, 1),
            body: Default::default(),
            move_epoch: 0,
            action_state: crate::game::battle::core::movement::ActionState::Idle,
            action_locks: Default::default(),
            current_target: None,
            next_basic_attack_ms: 0,
            pending_basic_attack: false,
            resonance_current: 0,
            resonance_max: 100,
            resonance_lock_ms: 0,
            next_action_time: 0,
            pending_cast: false,
            pending_cast_cause: None,
            pending_skill_cast: None,
        }
    }

    #[test]
    fn collect_triggers_sorts_artifacts_by_instance_id_and_includes_sources() {
        let base_a = Uuid::from_u128(100);
        let base_b = Uuid::from_u128(101);

        let mut effects_a = HashMap::new();
        effects_a.insert(
            TriggerType::OnAttack,
            vec![TriggeredEffect::legacy(Effect::Modifier(StatModifier {
                stat: StatId::Attack,
                kind: StatModifierKind::Flat,
                value: 1,
            }))],
        );

        let mut effects_b = HashMap::new();
        effects_b.insert(
            TriggerType::OnAttack,
            vec![TriggeredEffect::legacy(Effect::ApplyBuff {
                buff_id: "skill_b".to_string(),
                duration_ms: 100,
            })],
        );

        let game_data = game_data_with(
            vec![
                crate::game::data::artifact_data::ArtifactMetadata {
                    id: "a".to_string(),
                    uuid: base_a,
                    name: "A".to_string(),
                    description: "".to_string(),
                    rarity: crate::game::enums::RiskLevel::ZAYIN,
                    price: 0,
                    triggered_effects: effects_a,
                    ability_activations: vec![],
                },
                crate::game::data::artifact_data::ArtifactMetadata {
                    id: "b".to_string(),
                    uuid: base_b,
                    name: "B".to_string(),
                    description: "".to_string(),
                    rarity: crate::game::enums::RiskLevel::ZAYIN,
                    price: 0,
                    triggered_effects: effects_b,
                    ability_activations: vec![],
                },
            ],
            vec![],
        );

        let mut core = new_core(game_data);

        let instance_small = Uuid::from_u128(1);
        let instance_large = Uuid::from_u128(2);
        core.artifacts.insert(
            instance_large,
            RuntimeArtifact {
                instance_id: instance_large,
                owner: Side::Player,
                base_uuid: base_b,
            },
        );
        core.artifacts.insert(
            instance_small,
            RuntimeArtifact {
                instance_id: instance_small,
                owner: Side::Player,
                base_uuid: base_a,
            },
        );
        core.artifacts.insert(
            Uuid::from_u128(3),
            RuntimeArtifact {
                instance_id: Uuid::from_u128(3),
                owner: Side::Opponent,
                base_uuid: base_a,
            },
        );

        let out = core.collect_triggers(
            TriggerSource::Artifact { side: Side::Player },
            TriggerType::OnAttack,
        );
        assert_eq!(out.len(), 2);

        assert!(matches!(
            out[0],
            SourcedEffect {
                source: CooldownSource::Artifact {
                    artifact_instance_id,
                },
                target: TriggerEffectTarget::SelfUnit,
                effect: Effect::Modifier(_),
            } if artifact_instance_id == instance_small
        ));
        assert!(matches!(
            out[1],
            SourcedEffect {
                source: CooldownSource::Artifact {
                    artifact_instance_id,
                },
                target: TriggerEffectTarget::SelfUnit,
                effect: Effect::ApplyBuff {
                    ref buff_id,
                    duration_ms,
                },
            } if artifact_instance_id == instance_large
                && buff_id == "skill_b"
                && duration_ms == 100
        ));
    }

    #[test]
    fn collect_triggers_sorts_items_by_instance_id_and_includes_sources() {
        let base_item = Uuid::from_u128(200);
        let mut triggered = HashMap::new();
        triggered.insert(
            TriggerType::OnHit,
            vec![TriggeredEffect::legacy(Effect::ApplyBuff {
                buff_id: "on_hit_buff".to_string(),
                duration_ms: 50,
            })],
        );
        let equipment = crate::game::data::equipment_data::EquipmentMetadata {
            id: "e".to_string(),
            uuid: base_item,
            name: "E".to_string(),
            equipment_type: crate::game::data::equipment_data::EquipmentType::Weapon,
            rarity: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            triggered_effects: triggered,
            ability_activations: vec![],
        };

        let game_data = game_data_with(vec![], vec![equipment]);
        let mut core = new_core(game_data);

        let unit_id: UnitInstanceId = Uuid::from_u128(10).into();
        let item_small = Uuid::from_u128(1);
        let item_large = Uuid::from_u128(2);
        core.items.insert(
            item_large,
            RuntimeItem {
                instance_id: item_large,
                owner: Side::Player,
                owner_unit_instance: unit_id,
                base_uuid: base_item,
            },
        );
        core.items.insert(
            item_small,
            RuntimeItem {
                instance_id: item_small,
                owner: Side::Player,
                owner_unit_instance: unit_id,
                base_uuid: base_item,
            },
        );

        let out = core.collect_triggers(
            TriggerSource::Item {
                unit_instance_id: unit_id,
            },
            TriggerType::OnHit,
        );
        assert_eq!(out.len(), 2);

        assert!(matches!(
            out[0],
            SourcedEffect {
                source: CooldownSource::Item { item_instance_id },
                target: TriggerEffectTarget::SelfUnit,
                effect: Effect::ApplyBuff {
                    ref buff_id,
                    duration_ms,
                },
            } if item_instance_id == item_small
                && buff_id == "on_hit_buff"
                && duration_ms == 50
        ));
        assert!(matches!(
            out[1],
            SourcedEffect {
                source: CooldownSource::Item { item_instance_id },
                target: TriggerEffectTarget::SelfUnit,
                effect: Effect::ApplyBuff {
                    ref buff_id,
                    duration_ms,
                },
            } if item_instance_id == item_large
                && buff_id == "on_hit_buff"
                && duration_ms == 50
        ));
    }

    #[test]
    fn collect_all_triggers_includes_owner_artifacts_and_unit_items() {
        let base_art = Uuid::from_u128(300);
        let base_item = Uuid::from_u128(400);

        let mut art_effects = HashMap::new();
        art_effects.insert(
            TriggerType::OnBattleStart,
            vec![TriggeredEffect::legacy(Effect::ApplyBuff {
                buff_id: "art_start".to_string(),
                duration_ms: 1,
            })],
        );
        let mut item_effects = HashMap::new();
        item_effects.insert(
            TriggerType::OnBattleStart,
            vec![TriggeredEffect::legacy(Effect::ApplyBuff {
                buff_id: "item_start".to_string(),
                duration_ms: 1,
            })],
        );

        let game_data = game_data_with(
            vec![crate::game::data::artifact_data::ArtifactMetadata {
                id: "a".to_string(),
                uuid: base_art,
                name: "A".to_string(),
                description: "".to_string(),
                rarity: crate::game::enums::RiskLevel::ZAYIN,
                price: 0,
                triggered_effects: art_effects,
                ability_activations: vec![],
            }],
            vec![crate::game::data::equipment_data::EquipmentMetadata {
                id: "e".to_string(),
                uuid: base_item,
                name: "E".to_string(),
                equipment_type: crate::game::data::equipment_data::EquipmentType::Weapon,
                rarity: crate::game::enums::RiskLevel::ZAYIN,
                price: 0,
                allow_duplicate_equip: true,
                triggered_effects: item_effects,
                ability_activations: vec![],
            }],
        );

        let mut core = new_core(game_data);

        let unit_id: UnitInstanceId = Uuid::from_u128(1).into();
        core.units.insert(unit_id, runtime_unit(1, Side::Player));

        core.artifacts.insert(
            Uuid::from_u128(10),
            RuntimeArtifact {
                instance_id: Uuid::from_u128(10),
                owner: Side::Player,
                base_uuid: base_art,
            },
        );
        core.items.insert(
            Uuid::from_u128(11),
            RuntimeItem {
                instance_id: Uuid::from_u128(11),
                owner: Side::Player,
                owner_unit_instance: unit_id,
                base_uuid: base_item,
            },
        );

        core.recording_cause_stack.push(TimelineCause::Root {
            kind: TimelineRootCause::System,
        });
        let out = core.collect_all_triggers(unit_id, TriggerType::OnBattleStart);
        assert_eq!(out.len(), 2);

        let mut seen = HashSet::new();
        for effect in out {
            match effect {
                SourcedEffect {
                    source: CooldownSource::Artifact { .. },
                    target: TriggerEffectTarget::SelfUnit,
                    effect:
                        Effect::ApplyBuff {
                            buff_id: id,
                            duration_ms: 1,
                        },
                } => {
                    assert_eq!(id, "art_start");
                    seen.insert("art");
                }
                SourcedEffect {
                    source: CooldownSource::Item { .. },
                    target: TriggerEffectTarget::SelfUnit,
                    effect:
                        Effect::ApplyBuff {
                            buff_id: id,
                            duration_ms: 1,
                        },
                } => {
                    assert_eq!(id, "item_start");
                    seen.insert("item");
                }
                other => panic!("unexpected effect: {other:?}"),
            }
        }
        assert_eq!(seen.len(), 2);
    }
}
