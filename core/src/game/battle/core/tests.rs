use std::{path::PathBuf, sync::Arc};

use bevy_ecs::world::World;
use uuid::Uuid;

use crate::{
    ecs::resources::{Inventory, Position},
    game::{
        battle::TimelineEvent,
        data::{
            abnormality_data::{AbnormalityDatabase, AbnormalityMetadata},
            artifact_data::ArtifactDatabase,
            bonus_data::BonusDatabase,
            equipment_data::{EquipmentDatabase, EquipmentMetadata, EquipmentType},
            event_pools::{EventPhasePool, EventPoolConfig},
            pve_data::PveEncounterDatabase,
            random_event_data::RandomEventDatabase,
            shop_data::ShopDatabase,
            GameDataBase,
        },
        enums::{RiskLevel, Side, Tier},
        stats::{
            Effect, StatId, StatModifier, StatModifierKind, TriggerType, TriggeredEffects,
            UnitStats,
        },
    },
};

use super::{BattleCore, RuntimeItem, RuntimeUnit};
use crate::game::battle::{HpChangeReason, OwnedUnit, PlayerDeckInfo, Timeline};

fn empty_event_pools() -> EventPoolConfig {
    let empty = EventPhasePool {
        shops: vec![],
        bonuses: vec![],
        random_events: vec![],
    };
    EventPoolConfig {
        dawn: empty.clone(),
        noon: empty.clone(),
        dusk: empty.clone(),
        midnight: empty.clone(),
        white: empty,
    }
}

fn minimal_game_data(
    abnormalities: Vec<AbnormalityMetadata>,
    equipments: Vec<EquipmentMetadata>,
) -> Arc<GameDataBase> {
    let abnormality_data = Arc::new(AbnormalityDatabase::new(abnormalities));
    let artifact_data = Arc::new(ArtifactDatabase::new(vec![]));
    let equipment_data = Arc::new(EquipmentDatabase::new(equipments));
    let shop_data = Arc::new(ShopDatabase::new(vec![]));
    let bonus_data = Arc::new(BonusDatabase::new(vec![]));
    let random_event_data = Arc::new(RandomEventDatabase::new(vec![]));
    let pve_data = Arc::new(PveEncounterDatabase::new(vec![]));
    let event_pools = empty_event_pools();

    Arc::new(GameDataBase::new(
        abnormality_data,
        artifact_data,
        equipment_data,
        shop_data,
        bonus_data,
        random_event_data,
        pve_data,
        event_pools,
    ))
}

#[test]
fn battle_does_not_use_world_inventory_for_artifacts() {
    let mut world = World::new();
    let mut inventory = Inventory::new();

    let artifact = Arc::new(crate::game::data::artifact_data::ArtifactMetadata {
        id: "a".to_string(),
        uuid: Uuid::from_u128(1),
        name: "a".to_string(),
        description: "a".to_string(),
        rarity: RiskLevel::ZAYIN,
        price: 0,
        triggered_effects: TriggeredEffects::default(),
    });
    inventory.artifacts.add_item(artifact).unwrap();
    world.insert_resource(inventory);

    let player_uuid = Uuid::from_u128(10);
    let opponent_uuid = Uuid::from_u128(11);

    let game_data = minimal_game_data(
        vec![
            AbnormalityMetadata {
                id: "p".to_string(),
                uuid: player_uuid,
                name: "p".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 10,
                attack: 1,
                defense: 0,
                attack_interval_ms: 1000,
                abilities: vec![],
            },
            AbnormalityMetadata {
                id: "o".to_string(),
                uuid: opponent_uuid,
                name: "o".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 10,
                attack: 1,
                defense: 0,
                attack_interval_ms: 1000,
                abilities: vec![],
            },
        ],
        vec![],
    );

    let player = PlayerDeckInfo {
        units: vec![OwnedUnit {
            base_uuid: player_uuid,
            level: Tier::I,
            growth_stacks: Default::default(),
            equipped_items: vec![],
        }],
        artifacts: vec![],
        positions: [(player_uuid, Position::new(0, 0))].into_iter().collect(),
    };
    let opponent = PlayerDeckInfo {
        units: vec![OwnedUnit {
            base_uuid: opponent_uuid,
            level: Tier::I,
            growth_stacks: Default::default(),
            equipped_items: vec![],
        }],
        artifacts: vec![],
        positions: [(opponent_uuid, Position::new(1, 0))].into_iter().collect(),
    };

    let mut battle = BattleCore::new(&player, &opponent, game_data, (3, 3));
    let _ = battle.run_battle(&mut world).unwrap();

    assert!(battle.artifacts.is_empty());
}

#[test]
fn ability_does_not_execute_with_caster_in_graveyard() {
    let caster_id = Uuid::from_u128(100);
    let target_id = Uuid::from_u128(200);

    let game_data = minimal_game_data(
        vec![
            AbnormalityMetadata {
                id: "c".to_string(),
                uuid: Uuid::from_u128(1),
                name: "c".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 10,
                attack: 1,
                defense: 0,
                attack_interval_ms: 1000,
                abilities: vec![],
            },
            AbnormalityMetadata {
                id: "t".to_string(),
                uuid: Uuid::from_u128(2),
                name: "t".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 10,
                attack: 1,
                defense: 0,
                attack_interval_ms: 1000,
                abilities: vec![],
            },
        ],
        vec![],
    );

    let empty_deck = PlayerDeckInfo {
        units: vec![],
        artifacts: vec![],
        positions: Default::default(),
    };
    let mut battle = BattleCore::new(&empty_deck, &empty_deck, game_data, (3, 3));

    battle.units.insert(
        target_id,
        RuntimeUnit {
            instance_id: target_id,
            owner: Side::Opponent,
            base_uuid: Uuid::from_u128(2),
            stats: UnitStats::with_values(10, 10, 1, 0, 1000),
            position: Position::new(1, 0),
            current_target: None,
        },
    );

    battle.graveyard.insert(
        caster_id,
        super::super::ability_executor::UnitSnapshot {
            id: caster_id,
            owner: Side::Player,
            position: Position::new(0, 0),
            stats: UnitStats::with_values(10, 0, 1, 0, 1000),
        },
    );

    let result = battle.execute_ability_via_executor(
        crate::game::ability::AbilityId::UnknownDistortionStrike,
        caster_id,
        None,
        0,
    );

    assert!(!result.executed);
    assert!(battle.units.contains_key(&target_id));
}

#[test]
fn ability_kill_credits_killer_for_on_kill_triggers() {
    let caster_id = Uuid::from_u128(101);
    let target_id = Uuid::from_u128(201);
    let equipment_uuid = Uuid::from_u128(301);

    let mut triggered_effects = TriggeredEffects::default();
    triggered_effects.insert(
        TriggerType::OnKill,
        vec![Effect::Modifier(StatModifier {
            stat: StatId::Attack,
            kind: StatModifierKind::Flat,
            value: 5,
        })],
    );

    let game_data = minimal_game_data(
        vec![
            AbnormalityMetadata {
                id: "c".to_string(),
                uuid: Uuid::from_u128(1),
                name: "c".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 10,
                attack: 1,
                defense: 0,
                attack_interval_ms: 1000,
                abilities: vec![],
            },
            AbnormalityMetadata {
                id: "t".to_string(),
                uuid: Uuid::from_u128(2),
                name: "t".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 10,
                attack: 1,
                defense: 0,
                attack_interval_ms: 1000,
                abilities: vec![],
            },
        ],
        vec![EquipmentMetadata {
            id: "e".to_string(),
            uuid: equipment_uuid,
            name: "e".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            triggered_effects,
        }],
    );

    let empty_deck = PlayerDeckInfo {
        units: vec![],
        artifacts: vec![],
        positions: Default::default(),
    };
    let mut battle = BattleCore::new(&empty_deck, &empty_deck, game_data, (3, 3));

    battle.units.insert(
        caster_id,
        RuntimeUnit {
            instance_id: caster_id,
            owner: Side::Player,
            base_uuid: Uuid::from_u128(1),
            stats: UnitStats::with_values(10, 10, 1, 0, 1000),
            position: Position::new(0, 0),
            current_target: None,
        },
    );
    battle.items.insert(
        Uuid::from_u128(999),
        RuntimeItem {
            instance_id: Uuid::from_u128(999),
            owner: Side::Player,
            owner_unit_instance: caster_id,
            base_uuid: equipment_uuid,
        },
    );
    battle.units.insert(
        target_id,
        RuntimeUnit {
            instance_id: target_id,
            owner: Side::Opponent,
            base_uuid: Uuid::from_u128(2),
            stats: UnitStats::with_values(10, 10, 1, 0, 1000),
            position: Position::new(1, 0),
            current_target: None,
        },
    );

    let result = battle.execute_ability_via_executor(
        crate::game::ability::AbilityId::UnknownDistortionStrike,
        caster_id,
        None,
        0,
    );

    battle.process_commands(result.commands, 0);

    assert!(!battle.units.contains_key(&target_id));
    assert_eq!(battle.units.get(&caster_id).unwrap().stats.attack, 6);
}

#[test]
fn simultaneous_deaths_do_not_trigger_on_ally_death_from_dead_units() {
    let ally_a = Uuid::from_u128(10);
    let ally_b = Uuid::from_u128(11);
    let enemy = Uuid::from_u128(20);
    let equipment_uuid = Uuid::from_u128(30);

    let mut triggered_effects = TriggeredEffects::default();
    triggered_effects.insert(
        TriggerType::OnAllyDeath,
        vec![Effect::Ability(
            crate::game::ability::AbilityId::UnknownDistortionStrike,
        )],
    );

    let game_data = minimal_game_data(
        vec![
            AbnormalityMetadata {
                id: "a".to_string(),
                uuid: Uuid::from_u128(1),
                name: "a".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 10,
                attack: 1,
                defense: 0,
                attack_interval_ms: 1000,
                abilities: vec![],
            },
            AbnormalityMetadata {
                id: "b".to_string(),
                uuid: Uuid::from_u128(2),
                name: "b".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 10,
                attack: 1,
                defense: 0,
                attack_interval_ms: 1000,
                abilities: vec![],
            },
            AbnormalityMetadata {
                id: "e".to_string(),
                uuid: Uuid::from_u128(3),
                name: "e".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 10,
                attack: 1,
                defense: 0,
                attack_interval_ms: 1000,
                abilities: vec![],
            },
        ],
        vec![EquipmentMetadata {
            id: "skill_item".to_string(),
            uuid: equipment_uuid,
            name: "skill_item".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            triggered_effects,
        }],
    );

    let empty_deck = PlayerDeckInfo {
        units: vec![],
        artifacts: vec![],
        positions: Default::default(),
    };
    let mut battle = BattleCore::new(&empty_deck, &empty_deck, game_data, (3, 3));

    battle.units.insert(
        ally_a,
        RuntimeUnit {
            instance_id: ally_a,
            owner: Side::Player,
            base_uuid: Uuid::from_u128(1),
            stats: UnitStats::with_values(10, 10, 1, 0, 1000),
            position: Position::new(0, 0),
            current_target: None,
        },
    );
    battle.units.insert(
        ally_b,
        RuntimeUnit {
            instance_id: ally_b,
            owner: Side::Player,
            base_uuid: Uuid::from_u128(2),
            stats: UnitStats::with_values(10, 10, 1, 0, 1000),
            position: Position::new(1, 0),
            current_target: None,
        },
    );
    battle.items.insert(
        Uuid::from_u128(999),
        RuntimeItem {
            instance_id: Uuid::from_u128(999),
            owner: Side::Player,
            owner_unit_instance: ally_b,
            base_uuid: equipment_uuid,
        },
    );
    battle.units.insert(
        enemy,
        RuntimeUnit {
            instance_id: enemy,
            owner: Side::Opponent,
            base_uuid: Uuid::from_u128(3),
            stats: UnitStats::with_values(10, 10, 1, 0, 1000),
            position: Position::new(2, 0),
            current_target: None,
        },
    );

    battle.process_commands(
        vec![
            crate::game::battle::damage::BattleCommand::ApplyHeal {
                target_id: ally_a,
                flat: -999,
                percent: 0,
                source_id: Some(enemy),
            },
            crate::game::battle::damage::BattleCommand::ApplyHeal {
                target_id: ally_b,
                flat: -999,
                percent: 0,
                source_id: Some(enemy),
            },
        ],
        0,
    );

    // Then: 동시 사망한 유닛(ally_b)은 침묵해야 하므로 OnAllyDeath로 스킬이 발동하지 않아야 한다
    assert!(battle.units.contains_key(&enemy));
    assert!(!battle
        .timeline
        .entries
        .iter()
        .any(|e| matches!(e.event, TimelineEvent::AbilityCast { .. })));
}

#[test]
fn apply_heal_percent_affects_current_health() {
    let unit_id = Uuid::from_u128(10);

    let game_data = minimal_game_data(
        vec![AbnormalityMetadata {
            id: "u".to_string(),
            uuid: Uuid::from_u128(1),
            name: "u".to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 0,
            max_health: 20,
            attack: 1,
            defense: 0,
            attack_interval_ms: 1000,
            abilities: vec![],
        }],
        vec![],
    );

    let empty_deck = PlayerDeckInfo {
        units: vec![],
        artifacts: vec![],
        positions: Default::default(),
    };
    let mut battle = BattleCore::new(&empty_deck, &empty_deck, game_data, (3, 3));

    battle.units.insert(
        unit_id,
        RuntimeUnit {
            instance_id: unit_id,
            owner: Side::Player,
            base_uuid: Uuid::from_u128(1),
            stats: UnitStats::with_values(20, 10, 1, 0, 1000),
            position: Position::new(0, 0),
            current_target: None,
        },
    );

    battle.process_commands(
        vec![crate::game::battle::damage::BattleCommand::ApplyHeal {
            target_id: unit_id,
            flat: 0,
            percent: 50,
            source_id: None,
        }],
        0,
    );

    assert_eq!(battle.units.get(&unit_id).unwrap().stats.current_health, 20);
}

#[test]
fn poison_buff_ticks_as_command_damage() {
    let mut world = World::new();

    let player_uuid = Uuid::from_u128(1000);
    let opponent_uuid = Uuid::from_u128(1001);
    let equipment_uuid = Uuid::from_u128(2000);

    let mut triggered_effects = TriggeredEffects::default();
    triggered_effects.insert(
        TriggerType::OnAttack,
        vec![Effect::ApplyBuff {
            buff_id: "poison".to_string(),
            duration_ms: 3500,
        }],
    );

    let game_data = minimal_game_data(
        vec![
            AbnormalityMetadata {
                id: "p".to_string(),
                uuid: player_uuid,
                name: "p".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 100,
                attack: 1,
                defense: 0,
                attack_interval_ms: 1000,
                abilities: vec![],
            },
            AbnormalityMetadata {
                id: "o".to_string(),
                uuid: opponent_uuid,
                name: "o".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 100,
                attack: 1,
                defense: 0,
                attack_interval_ms: 1000,
                abilities: vec![],
            },
        ],
        vec![EquipmentMetadata {
            id: "poison_weapon".to_string(),
            uuid: equipment_uuid,
            name: "poison_weapon".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            triggered_effects,
        }],
    );

    let player = PlayerDeckInfo {
        units: vec![OwnedUnit {
            base_uuid: player_uuid,
            level: Tier::I,
            growth_stacks: Default::default(),
            equipped_items: vec![equipment_uuid],
        }],
        artifacts: vec![],
        positions: [(player_uuid, Position::new(0, 0))].into_iter().collect(),
    };
    let opponent = PlayerDeckInfo {
        units: vec![OwnedUnit {
            base_uuid: opponent_uuid,
            level: Tier::I,
            growth_stacks: Default::default(),
            equipped_items: vec![],
        }],
        artifacts: vec![],
        positions: [(opponent_uuid, Position::new(1, 0))].into_iter().collect(),
    };

    let mut battle = BattleCore::new(&player, &opponent, game_data, (3, 3));
    let result = battle.run_battle(&mut world).unwrap();

    let has_poison_tick = result
        .timeline
        .entries
        .iter()
        .any(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                delta,
                reason: HpChangeReason::Command,
                ..
            } => *delta == -2,
            _ => false,
        });

    assert!(has_poison_tick);
}

#[test]
fn basic_attack_kills_and_player_wins() {
    let mut world = World::new();

    let player_base_uuid = Uuid::from_u128(10);
    let opponent_base_uuid = Uuid::from_u128(11);

    let game_data = minimal_game_data(
        vec![
            AbnormalityMetadata {
                id: "player".to_string(),
                uuid: player_base_uuid,
                name: "player".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 10,
                attack: 100,
                defense: 0,
                attack_interval_ms: 1,
                abilities: vec![],
            },
            AbnormalityMetadata {
                id: "opponent".to_string(),
                uuid: opponent_base_uuid,
                name: "opponent".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 50,
                attack: 0,
                defense: 0,
                attack_interval_ms: 1000,
                abilities: vec![],
            },
        ],
        vec![],
    );

    let player = PlayerDeckInfo {
        units: vec![OwnedUnit {
            base_uuid: player_base_uuid,
            level: Tier::I,
            growth_stacks: Default::default(),
            equipped_items: vec![],
        }],
        artifacts: vec![],
        positions: [(player_base_uuid, Position::new(0, 0))]
            .into_iter()
            .collect(),
    };
    let opponent = PlayerDeckInfo {
        units: vec![OwnedUnit {
            base_uuid: opponent_base_uuid,
            level: Tier::I,
            growth_stacks: Default::default(),
            equipped_items: vec![],
        }],
        artifacts: vec![],
        positions: [(opponent_base_uuid, Position::new(1, 0))]
            .into_iter()
            .collect(),
    };

    let mut battle = BattleCore::new(&player, &opponent, game_data, (3, 3));
    let result = battle.run_battle(&mut world).unwrap();

    assert_eq!(result.winner, crate::game::battle::BattleWinner::Player);
    assert!(result
        .timeline
        .entries
        .iter()
        .any(|e| matches!(e.event, TimelineEvent::BattleStart { .. })));
    assert_eq!(
        result
            .timeline
            .entries
            .iter()
            .filter(|e| matches!(e.event, TimelineEvent::UnitSpawned { .. }))
            .count(),
        2
    );
    assert!(result
        .timeline
        .entries
        .iter()
        .any(|e| matches!(e.event, TimelineEvent::Attack { .. })));
    assert!(result
        .timeline
        .entries
        .iter()
        .any(|e| matches!(e.event, TimelineEvent::HpChanged { .. })));
    assert!(result
        .timeline
        .entries
        .iter()
        .any(|e| matches!(e.event, TimelineEvent::UnitDied { .. })));
    assert!(result.timeline.entries.iter().any(|e| {
        matches!(
            e.event,
            TimelineEvent::BattleEnd {
                winner: crate::game::battle::BattleWinner::Player
            }
        )
    }));

    let opponent_instance_id = BattleCore::make_instance_id(opponent_base_uuid, Side::Opponent, 0);
    let player_instance_id = BattleCore::make_instance_id(player_base_uuid, Side::Player, 0);

    assert!(battle.units.contains_key(&player_instance_id));
    assert!(!battle.units.contains_key(&opponent_instance_id));
    assert!(battle.graveyard.contains_key(&opponent_instance_id));
}

#[test]
fn on_attack_skill_triggers_and_changes_outcome() {
    let mut world = World::new();

    let player_base_uuid = Uuid::from_u128(20);
    let opponent_base_uuid = Uuid::from_u128(21);
    let equipment_uuid = Uuid::from_u128(22);

    let mut triggered_effects = TriggeredEffects::default();
    triggered_effects.insert(
        TriggerType::OnAttack,
        vec![Effect::Ability(
            crate::game::ability::AbilityId::UnknownDistortionStrike,
        )],
    );
    triggered_effects.insert(
        TriggerType::OnKill,
        vec![Effect::Modifier(StatModifier {
            stat: StatId::Attack,
            kind: StatModifierKind::Flat,
            value: 5,
        })],
    );

    let game_data = minimal_game_data(
        vec![
            AbnormalityMetadata {
                id: "player".to_string(),
                uuid: player_base_uuid,
                name: "player".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 10,
                attack: 1,
                defense: 0,
                attack_interval_ms: 5000,
                abilities: vec![],
            },
            AbnormalityMetadata {
                id: "opponent".to_string(),
                uuid: opponent_base_uuid,
                name: "opponent".to_string(),
                risk_level: RiskLevel::ZAYIN,
                price: 0,
                max_health: 20,
                attack: 1,
                defense: 0,
                attack_interval_ms: 1000,
                abilities: vec![],
            },
        ],
        vec![EquipmentMetadata {
            id: "skill_item".to_string(),
            uuid: equipment_uuid,
            name: "skill_item".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            triggered_effects,
        }],
    );

    let player = PlayerDeckInfo {
        units: vec![OwnedUnit {
            base_uuid: player_base_uuid,
            level: Tier::I,
            growth_stacks: Default::default(),
            equipped_items: vec![equipment_uuid],
        }],
        artifacts: vec![],
        positions: [(player_base_uuid, Position::new(0, 0))]
            .into_iter()
            .collect(),
    };
    let opponent = PlayerDeckInfo {
        units: vec![OwnedUnit {
            base_uuid: opponent_base_uuid,
            level: Tier::I,
            growth_stacks: Default::default(),
            equipped_items: vec![],
        }],
        artifacts: vec![],
        positions: [(opponent_base_uuid, Position::new(1, 0))]
            .into_iter()
            .collect(),
    };

    let mut battle = BattleCore::new(&player, &opponent, game_data, (3, 3));
    let result = battle.run_battle(&mut world).unwrap();

    assert_eq!(result.winner, crate::game::battle::BattleWinner::Player);
    assert!(result.timeline.entries.iter().any(|e| {
        matches!(
            e.event,
            TimelineEvent::AbilityCast {
                ability_id: crate::game::ability::AbilityId::UnknownDistortionStrike,
                ..
            }
        )
    }));

    let player_instance_id = BattleCore::make_instance_id(player_base_uuid, Side::Player, 0);
    let opponent_instance_id = BattleCore::make_instance_id(opponent_base_uuid, Side::Opponent, 0);

    assert!(battle.units.contains_key(&player_instance_id));
    assert!(!battle.units.contains_key(&opponent_instance_id));
    assert!(battle.graveyard.contains_key(&opponent_instance_id));

    assert_eq!(
        battle.units.get(&player_instance_id).unwrap().stats.attack,
        6
    );

    let json = result.timeline.to_json_string().unwrap();

    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test-output");
    std::fs::create_dir_all(&out_dir).unwrap();

    let out_path = out_dir.join(format!("battle-timeline-{}.json", Uuid::new_v4()));
    result.timeline.write_pretty_json(&out_path).unwrap();

    let loaded = Timeline::read_json(&out_path).unwrap();
    let json_roundtrip = loaded.to_json_string().unwrap();
    assert_eq!(json, json_roundtrip);

    let _ = std::fs::remove_file(&out_path);
}
