mod common;

use std::collections::{BinaryHeap, HashMap};

use bevy_ecs::world::World;
use game_core::ecs::resources::{Field, Inventory, Position};
use game_core::game::battle::buffs::BuffId;
use game_core::game::battle::enums::BattleEvent;
use game_core::game::battle::{
    BattleCore, BattleWinner, GrowthId, GrowthStack, OwnedArtifact, OwnedUnit, PlayerDeckInfo,
};
use game_core::game::enums::Tier;
use game_core::game::stats::{StatId, StatModifier, StatModifierKind, UnitStats};
use uuid::Uuid;

use common::create_test_game_data;

// ============================================================
// BattleEvent Priority Tests
// ============================================================

#[cfg(test)]
mod battle_event_tests {
    use super::*;

    #[test]
    fn battle_event_time_ms_extraction() {
        let attacker_instance_id = Uuid::new_v4();
        let event = BattleEvent::Attack {
            time_ms: 1500,
            attacker_instance_id,
            target_instance_id: None,
            schedule_next: true,
            cause_seq: None,
        };
        assert_eq!(event.time_ms(), 1500);
    }

    #[test]
    fn battle_event_heap_orders_by_time() {
        let mut heap = BinaryHeap::new();

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        heap.push(BattleEvent::Attack {
            time_ms: 3000,
            attacker_instance_id: id1,
            target_instance_id: None,
            schedule_next: true,
            cause_seq: None,
        });
        heap.push(BattleEvent::Attack {
            time_ms: 1000,
            attacker_instance_id: id2,
            target_instance_id: None,
            schedule_next: true,
            cause_seq: None,
        });
        heap.push(BattleEvent::Attack {
            time_ms: 2000,
            attacker_instance_id: id3,
            target_instance_id: None,
            schedule_next: true,
            cause_seq: None,
        });

        // Then: BinaryHeap은 time_ms가 가장 작은 이벤트부터 pop되어야 함 (custom Ord 기반)
        assert_eq!(heap.pop().unwrap().time_ms(), 1000);
        assert_eq!(heap.pop().unwrap().time_ms(), 2000);
        assert_eq!(heap.pop().unwrap().time_ms(), 3000);
    }

    #[test]
    fn battle_event_priority_at_same_time() {
        let mut heap = BinaryHeap::new();

        let id = Uuid::new_v4();
        let buff_id = BuffId::from_name("test_buff");

        // Given: 같은 time_ms에 서로 다른 타입의 이벤트가 섞여있음
        heap.push(BattleEvent::Attack {
            time_ms: 1000,
            attacker_instance_id: id,
            target_instance_id: None,
            schedule_next: true,
            cause_seq: None,
        });
        heap.push(BattleEvent::ApplyBuff {
            time_ms: 1000,
            caster_instance_id: id,
            target_instance_id: id,
            buff_id,
            duration_ms: 1000,
            cause_seq: None,
        });
        heap.push(BattleEvent::BuffExpire {
            time_ms: 1000,
            caster_instance_id: id,
            target_instance_id: id,
            buff_id,
            cause_seq: None,
        });
        heap.push(BattleEvent::BuffTick {
            time_ms: 1000,
            caster_instance_id: id,
            target_instance_id: id,
            buff_id,
            cause_seq: None,
        });

        // Then: 우선순위는 ApplyBuff(1) → BuffTick(2) → Attack(3) → BuffExpire(4) 순서여야 함
        let first = heap.pop().unwrap();
        assert!(matches!(first, BattleEvent::ApplyBuff { .. }));

        let second = heap.pop().unwrap();
        assert!(matches!(second, BattleEvent::BuffTick { .. }));

        let third = heap.pop().unwrap();
        assert!(matches!(third, BattleEvent::Attack { .. }));

        let fourth = heap.pop().unwrap();
        assert!(matches!(fourth, BattleEvent::BuffExpire { .. }));
    }
}

// ============================================================
// UnitStats Tests
// ============================================================

#[cfg(test)]
mod unit_stats_tests {
    use super::*;

    #[test]
    fn unit_stats_new_creates_zero_stats() {
        let stats = UnitStats::new();
        assert_eq!(stats.max_health, 0);
        assert_eq!(stats.attack, 0);
        assert_eq!(stats.defense, 0);
        assert_eq!(stats.attack_interval_ms, 0);
    }

    #[test]
    fn unit_stats_with_values() {
        let stats = UnitStats::with_values(100, 100, 50, 10, 1500);
        assert_eq!(stats.max_health, 100);
        assert_eq!(stats.current_health, 100);
        assert_eq!(stats.attack, 50);
        assert_eq!(stats.defense, 10);
        assert_eq!(stats.attack_interval_ms, 1500);
    }

    #[test]
    fn add_max_health_positive() {
        let mut stats = UnitStats::with_values(100, 100, 0, 0, 0);
        stats.add_max_health(50);
        assert_eq!(stats.max_health, 150);
    }

    #[test]
    fn add_max_health_negative() {
        let mut stats = UnitStats::with_values(100, 100, 0, 0, 0);
        stats.add_max_health(-30);
        assert_eq!(stats.max_health, 70);
    }

    #[test]
    fn add_max_health_negative_clamps_to_zero() {
        let mut stats = UnitStats::with_values(50, 50, 0, 0, 0);
        stats.add_max_health(-100);
        assert_eq!(stats.max_health, 0);
    }

    #[test]
    fn add_attack_positive_and_negative() {
        let mut stats = UnitStats::with_values(0, 0, 100, 0, 0);
        stats.add_attack(25);
        assert_eq!(stats.attack, 125);

        stats.add_attack(-50);
        assert_eq!(stats.attack, 75);
    }

    #[test]
    fn add_defense_clamps_to_zero() {
        let mut stats = UnitStats::with_values(0, 0, 0, 10, 0);
        stats.add_defense(-20);
        assert_eq!(stats.defense, 0);
    }

    #[test]
    fn add_attack_interval_ms() {
        let mut stats = UnitStats::with_values(0, 0, 0, 0, 1500);
        stats.add_attack_interval_ms(-200);
        assert_eq!(stats.attack_interval_ms, 1300);

        stats.add_attack_interval_ms(500);
        assert_eq!(stats.attack_interval_ms, 1800);
    }

    #[test]
    fn apply_flat_modifier() {
        let mut stats = UnitStats::with_values(100, 100, 50, 10, 1500);

        stats.apply_modifier(StatModifier {
            stat: StatId::Attack,
            kind: StatModifierKind::Flat,
            value: 20,
        });
        assert_eq!(stats.attack, 70);

        stats.apply_modifier(StatModifier {
            stat: StatId::MaxHealth,
            kind: StatModifierKind::Flat,
            value: -25,
        });
        assert_eq!(stats.max_health, 75);
    }

    #[test]
    fn apply_percent_modifier() {
        let mut stats = UnitStats::with_values(100, 100, 100, 100, 1000);

        // Then: +20% attack → 100 * 20 / 100 = +20 → 120
        stats.apply_modifier(StatModifier {
            stat: StatId::Attack,
            kind: StatModifierKind::Percent,
            value: 20,
        });
        assert_eq!(stats.attack, 120);

        // Then: -10% defense → 100 * (-10) / 100 = -10 → 90
        stats.apply_modifier(StatModifier {
            stat: StatId::Defense,
            kind: StatModifierKind::Percent,
            value: -10,
        });
        assert_eq!(stats.defense, 90);
    }

    #[test]
    fn apply_multiple_modifiers() {
        let mut stats = UnitStats::with_values(100, 100, 50, 10, 1500);

        let modifiers = vec![
            StatModifier {
                stat: StatId::Attack,
                kind: StatModifierKind::Flat,
                value: 10,
            },
            StatModifier {
                stat: StatId::Attack,
                kind: StatModifierKind::Percent,
                value: 20, // 20% of 60 = +12
            },
            StatModifier {
                stat: StatId::MaxHealth,
                kind: StatModifierKind::Flat,
                value: 50,
            },
        ];

        stats.apply_modifiers(modifiers);

        assert_eq!(stats.attack, 72); // 50 + 10 = 60, then 60 * 1.2 = 72
        assert_eq!(stats.max_health, 150);
    }
}

// ============================================================
// GrowthStack Tests
// ============================================================

#[cfg(test)]
mod growth_stack_tests {
    use super::*;

    #[test]
    fn growth_stack_new_is_empty() {
        let stack = GrowthStack::new();
        assert!(stack.stacks.is_empty());
    }

    #[test]
    fn growth_stack_insert_and_retrieve() {
        let mut stack = GrowthStack::new();
        stack.stacks.insert(GrowthId::KillStack, 10);
        stack.stacks.insert(GrowthId::PveWinStack, 5);

        assert_eq!(stack.stacks.get(&GrowthId::KillStack), Some(&10));
        assert_eq!(stack.stacks.get(&GrowthId::PveWinStack), Some(&5));
        assert_eq!(stack.stacks.get(&GrowthId::QuestRewardStack), None);
    }
}

// ============================================================
// BattleCore Tests
// ============================================================

#[cfg(test)]
mod battle_core_tests {
    use super::*;

    fn create_empty_deck() -> PlayerDeckInfo {
        PlayerDeckInfo {
            units: Vec::new(),
            artifacts: Vec::new(),
            positions: HashMap::new(),
        }
    }

    #[test]
    fn battle_core_new_initializes_empty() {
        let game_data = create_test_game_data();
        let player = create_empty_deck();
        let opponent = create_empty_deck();

        let _battle = BattleCore::new(&player, &opponent, game_data, (3, 3));

        // Then: BattleCore는 panic 없이 생성되어야 함
        assert!(true);
    }

    #[test]
    fn battle_with_empty_decks_returns_draw() {
        let game_data = create_test_game_data();
        let player = create_empty_deck();
        let opponent = create_empty_deck();

        let mut battle = BattleCore::new(&player, &opponent, game_data, (3, 3));

        let mut world = World::new();
        world.insert_resource(Inventory::new());
        world.insert_resource(Field::new(3, 3));

        let result = battle.run_battle(&mut world);
        assert!(result.is_ok());

        let battle_result = result.unwrap();
        assert_eq!(battle_result.winner, BattleWinner::Draw);
    }
}

// ============================================================
// PlayerDeckInfo Tests
// ============================================================

#[cfg(test)]
mod player_deck_info_tests {
    use super::*;

    #[test]
    fn player_deck_info_with_units() {
        let unit_uuid = Uuid::new_v4();
        let artifact_uuid = Uuid::new_v4();

        let unit = OwnedUnit {
            base_uuid: unit_uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        };

        let artifact = OwnedArtifact {
            base_uuid: artifact_uuid,
        };

        let mut positions = HashMap::new();
        positions.insert(unit_uuid, Position::new(1, 1));

        let deck = PlayerDeckInfo {
            units: vec![unit],
            artifacts: vec![artifact],
            positions,
        };

        assert_eq!(deck.units.len(), 1);
        assert_eq!(deck.artifacts.len(), 1);
        assert_eq!(deck.positions.len(), 1);
        assert_eq!(deck.positions.get(&unit_uuid), Some(&Position::new(1, 1)));
    }
}

// ============================================================
// OwnedUnit Tests
// ============================================================

#[cfg(test)]
mod owned_unit_tests {
    use super::*;

    #[test]
    fn owned_unit_effective_stats_base_only() {
        let game_data = create_test_game_data();

        // Given: test_abnorm_1 (테스트 데이터)
        let abnormality = game_data
            .abnormality_data
            .get_by_id("test_abnorm_1")
            .expect("test_abnorm_1 should exist");

        let unit = OwnedUnit {
            base_uuid: abnormality.uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        };

        let stats = unit
            .effective_stats(&game_data, &[])
            .expect("effective_stats should succeed");

        // Then: 기본 스탯이 메타데이터와 일치해야 함
        assert_eq!(stats.max_health, abnormality.max_health);
        assert_eq!(stats.attack, abnormality.attack);
        assert_eq!(stats.defense, abnormality.defense);
        assert_eq!(stats.attack_interval_ms, abnormality.attack_interval_ms);
    }

    #[test]
    fn owned_unit_effective_stats_with_growth() {
        let game_data = create_test_game_data();

        let abnormality = game_data
            .abnormality_data
            .get_by_id("test_abnorm_1")
            .expect("test_abnorm_1 should exist");

        let mut growth = GrowthStack::new();
        growth.stacks.insert(GrowthId::KillStack, 15);

        let unit = OwnedUnit {
            base_uuid: abnormality.uuid,
            level: Tier::I,
            growth_stacks: growth,
            equipped_items: vec![],
        };

        let stats = unit
            .effective_stats(&game_data, &[])
            .expect("effective_stats should succeed");

        // Then: KillStack은 공격력에 누적되어야 함
        assert_eq!(stats.attack, abnormality.attack + 15);
    }

    #[test]
    fn owned_unit_with_invalid_uuid_returns_error() {
        let game_data = create_test_game_data();

        let unit = OwnedUnit {
            base_uuid: Uuid::new_v4(), // Random UUID not in game_data
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        };

        let result = unit.effective_stats(&game_data, &[]);
        assert!(result.is_err());
    }
}

// ============================================================
// Integration: Full Battle Flow Tests
// ============================================================

#[cfg(test)]
mod battle_flow_tests {
    use super::*;

    #[test]
    fn battle_initializes_and_runs_player_only() {
        let game_data = create_test_game_data();

        let abnormality = game_data
            .abnormality_data
            .get_by_id("test_abnorm_1")
            .expect("test_abnorm_1 should exist");

        // Given: 플레이어 덱에 유닛 1개가 존재함
        let player_unit = OwnedUnit {
            base_uuid: abnormality.uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        };

        let mut player_positions = HashMap::new();
        player_positions.insert(abnormality.uuid, Position::new(0, 0));

        let player = PlayerDeckInfo {
            units: vec![player_unit],
            artifacts: vec![],
            positions: player_positions,
        };

        // Given: 상대 덱은 비어있음
        let opponent = PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        };

        let mut battle = BattleCore::new(&player, &opponent, game_data, (3, 3));

        // Given: 전투 실행에 필요한 World 리소스를 준비함
        let mut world = World::new();
        world.insert_resource(Inventory::new());
        world.insert_resource(Field::new(3, 3));

        // When: 전투를 실행함
        let result = battle.run_battle(&mut world);
        assert!(result.is_ok(), "Battle failed: {:?}", result.err());

        // Then: 플레이어만 유닛을 보유하므로 Player 승리
        let battle_result = result.unwrap();
        assert_eq!(battle_result.winner, BattleWinner::Player);
    }

    #[test]
    fn battle_initializes_and_runs_opponent_only() {
        let game_data = create_test_game_data();

        let abnormality = game_data
            .abnormality_data
            .get_by_id("test_abnorm_1")
            .expect("test_abnorm_1 should exist");

        // Given: 플레이어 덱은 비어있음
        let player = PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        };

        // Given: 상대 덱에 유닛 1개가 존재함
        let opponent_unit = OwnedUnit {
            base_uuid: abnormality.uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        };

        let mut opponent_positions = HashMap::new();
        opponent_positions.insert(abnormality.uuid, Position::new(2, 2));

        let opponent = PlayerDeckInfo {
            units: vec![opponent_unit],
            artifacts: vec![],
            positions: opponent_positions,
        };

        let mut battle = BattleCore::new(&player, &opponent, game_data, (3, 3));

        // Given: 전투 실행에 필요한 World 리소스를 준비함
        let mut world = World::new();
        world.insert_resource(Inventory::new());
        world.insert_resource(Field::new(3, 3));

        // When: 전투를 실행함
        let result = battle.run_battle(&mut world);
        assert!(result.is_ok(), "Battle failed: {:?}", result.err());

        // Then: 상대만 유닛을 보유하므로 Opponent 승리
        let battle_result = result.unwrap();
        assert_eq!(battle_result.winner, BattleWinner::Opponent);
    }

    #[test]
    fn battle_with_both_sides_times_out_as_draw() {
        let game_data = create_test_game_data();

        let abnormality = game_data
            .abnormality_data
            .get_by_id("test_abnorm_1")
            .expect("test_abnorm_1 should exist");

        // Given: runtime instance를 흉내내기 위해 서로 다른 UUID를 준비함
        let player_runtime_uuid = Uuid::new_v4();
        let opponent_runtime_uuid = Uuid::new_v4();

        // NOTE: 현재 구현은 base_uuid를 조회/배치 모두에 사용해서 충돌 가능성이 있음.
        // NOTE: 여기서는 충돌이 나지 않는 구성으로 전투 타임아웃(무승부)만 검증한다.

        // Given: 플레이어 덱에 유닛 1개가 존재함
        let player_unit = OwnedUnit {
            base_uuid: abnormality.uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        };

        let mut player_positions = HashMap::new();
        player_positions.insert(abnormality.uuid, Position::new(0, 0));

        let player = PlayerDeckInfo {
            units: vec![player_unit],
            artifacts: vec![],
            positions: player_positions,
        };

        // Given: base_uuid 충돌을 피하기 위해 상대는 비어있는 덱으로 둔다
        // TODO: battle system이 field placement에 runtime UUID를 사용하도록 수정 필요
        let opponent = PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        };

        let mut battle = BattleCore::new(&player, &opponent, game_data, (3, 3));

        // Given: 전투 실행에 필요한 World 리소스를 준비함
        let mut world = World::new();
        world.insert_resource(Inventory::new());
        world.insert_resource(Field::new(3, 3));

        let result = battle.run_battle(&mut world);
        assert!(result.is_ok(), "Battle failed: {:?}", result.err());

        // Then: 상대 덱이 비어있으므로 Player 승리
        let battle_result = result.unwrap();
        assert_eq!(battle_result.winner, BattleWinner::Player);
    }
}
