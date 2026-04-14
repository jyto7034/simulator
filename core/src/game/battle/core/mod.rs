use std::{
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};

use uuid::Uuid;

use crate::game::{
    battle::{
        battlefield::Battlefield,
        enums::BattleEvent,
        ids::UnitInstanceId,
        timeline::{Timeline, TimelineCause, TimelineEvent},
        types::{PlayerDeckInfo, UnitSnapshot},
    },
    data::GameDataBase,
};

pub mod build;
pub mod commands;
pub mod ids;
pub mod movement;
pub mod sim;
pub mod spatial;
pub mod triggers;
pub mod types;

use self::types::{
    AbilityProcKey, AbilityProcState, ActiveBuff, ActiveSkillCast, BuffInstanceKey,
    ProjectileRecord, RuntimeArtifact, RuntimeItem, RuntimeUnit, TriggerSource,
};

pub struct BattleCore {
    event_queue: BinaryHeap<BattleEvent>,

    player_info: PlayerDeckInfo,
    opponent_info: PlayerDeckInfo,

    pub units: HashMap<UnitInstanceId, RuntimeUnit>,
    artifacts: HashMap<Uuid, RuntimeArtifact>,
    items: HashMap<Uuid, RuntimeItem>,
    graveyard: HashMap<UnitInstanceId, UnitSnapshot>,

    buffs: HashMap<BuffInstanceKey, ActiveBuff>,
    active_skill_casts: HashMap<u64, ActiveSkillCast>,
    ability_proc_states: HashMap<AbilityProcKey, AbilityProcState>,

    projectiles: HashMap<Uuid, ProjectileRecord>,
    pub battlefield: Battlefield,

    pub game_data: Arc<GameDataBase>,

    pub timeline: Timeline,
    pub timeline_seq: u64,
    pub projectile_seq: u64,
    pub seed: u64,
    pub recording_cause_stack: Vec<TimelineCause>,
}

impl BattleCore {
    pub fn new(
        player: &PlayerDeckInfo,
        opponent: &PlayerDeckInfo,
        game_data: Arc<GameDataBase>,
        field_size: (u8, u8),
        seed: u64,
    ) -> Self {
        Self {
            event_queue: BinaryHeap::new(),
            player_info: player.clone(),
            opponent_info: opponent.clone(),
            units: HashMap::new(),
            artifacts: HashMap::new(),
            items: HashMap::new(),
            graveyard: HashMap::new(),
            buffs: HashMap::new(),
            active_skill_casts: HashMap::new(),
            ability_proc_states: HashMap::new(),
            projectiles: HashMap::new(),
            battlefield: Battlefield::new(field_size.0, field_size.1),
            game_data,
            timeline: Timeline::new(),
            timeline_seq: 0,
            projectile_seq: 0,
            seed,
            recording_cause_stack: Vec::new(),
        }
    }

    fn can_gain_resonance(unit: &RuntimeUnit, now_ms: u64) -> bool {
        unit.action_locks.can_gain_resonance(now_ms)
    }

    fn can_start_autocast(unit: &RuntimeUnit, now_ms: u64) -> bool {
        if now_ms < unit.next_action_time {
            return false;
        }
        if unit.is_dead() {
            return false;
        }
        true
    }

    fn add_resonance(
        &mut self,
        unit_instance_id: UnitInstanceId,
        amount: u32,
        now_ms: u64,
        allow_autocast_when_full: bool,
    ) {
        if amount == 0 {
            return;
        }

        let (before, after, max) = {
            let Some(unit) = self.units.get_mut(&unit_instance_id) else {
                return;
            };

            if unit.is_dead() {
                return;
            }

            if !Self::can_gain_resonance(unit, now_ms) {
                return;
            }

            let max = unit.resonance_max.max(1);
            let before = unit.resonance_current.min(max);
            let after = before.saturating_add(amount).min(max);
            if before == after {
                return;
            }

            unit.resonance_current = after;

            if allow_autocast_when_full && before < max && after == max {
                unit.pending_cast = true;
            }

            (before, after, max)
        };

        self.record_timeline(
            now_ms,
            TimelineEvent::ResonanceChanged {
                unit_instance_id,
                before,
                after,
                max,
            },
        );
    }

    fn modify_resonance(
        &mut self,
        unit_instance_id: UnitInstanceId,
        delta: i32,
        now_ms: u64,
        allow_autocast_when_full: bool,
    ) {
        if delta == 0 {
            return;
        }

        if delta > 0 {
            self.add_resonance(
                unit_instance_id,
                delta as u32,
                now_ms,
                allow_autocast_when_full,
            );
            return;
        }

        let (before, after, max) = {
            let Some(unit) = self.units.get_mut(&unit_instance_id) else {
                return;
            };

            if unit.is_dead() {
                return;
            }

            let max = unit.resonance_max.max(1);
            let before = unit.resonance_current.min(max);
            let dec = delta.unsigned_abs().min(before);
            let after = before.saturating_sub(dec);
            if before == after {
                return;
            }

            unit.resonance_current = after;
            (before, after, max)
        };

        self.record_timeline(
            now_ms,
            TimelineEvent::ResonanceChanged {
                unit_instance_id,
                before,
                after,
                max,
            },
        );
    }

    fn has_buff_kind(
        &self,
        unit_instance_id: UnitInstanceId,
        kind: crate::game::battle::buffs::BuffKind,
    ) -> bool {
        self.buffs.iter().any(|(key, _)| {
            key.target_instance_id == unit_instance_id
                && crate::game::battle::buffs::get(key.buff_id).is_some_and(|def| def.kind == kind)
        })
    }

    fn schedule_pending_autocast_for(&mut self, caster_instance_id: UnitInstanceId, now_ms: u64) {
        let fallback = self.recording_cause().unwrap_or_default();

        if self.has_buff_kind(
            caster_instance_id,
            crate::game::battle::buffs::BuffKind::Silence,
        ) {
            return;
        }

        let Some((cause, should_schedule)) = self.units.get_mut(&caster_instance_id).map(|unit| {
            if !unit.pending_cast {
                return (fallback, false);
            }
            if !Self::can_start_autocast(unit, now_ms) {
                return (fallback, false);
            }

            unit.pending_cast = false;
            let cause = unit.pending_cast_cause.take().unwrap_or(fallback);
            (cause, true)
        }) else {
            return;
        };

        if !should_schedule {
            return;
        }

        self.event_queue.push(BattleEvent::AutoCastStart {
            time_ms: now_ms,
            caster_instance_id,
            cause,
        });
    }

    fn schedule_pending_autocasts(&mut self, now_ms: u64) {
        let mut casters: Vec<UnitInstanceId> = self
            .units
            .iter()
            .filter_map(|(id, unit)| {
                if unit.pending_cast && !unit.is_dead() {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        casters.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        let cause = self.recording_cause().unwrap_or_default();

        for caster_instance_id in casters {
            if let Some(unit) = self.units.get_mut(&caster_instance_id) {
                if unit.pending_cast && unit.pending_cast_cause.is_none() {
                    unit.pending_cast_cause = Some(cause);
                }
            }
            self.schedule_pending_autocast_for(caster_instance_id, now_ms);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::resources::Position;
    use crate::game::ability::{
        DeliveryDef, SkillArea, SkillCastTargetingDef, SkillDef, SkillEffectDef, SkillKind,
        SkillPresentationDef, SkillStepDef, SkillTarget, StepTargetingMode, UnitTargetRule,
    };
    use crate::game::battle::buffs::BuffId;
    use crate::game::battle::core::movement::{ActionState, TILE_UNITS_PER_TILE};
    use crate::game::battle::core::types::RuntimeUnit;
    use crate::game::battle::damage::BattleCommand;
    use crate::game::battle::enums::BattleEvent;
    use crate::game::battle::timeline::{TimelineCause, TimelineEvent};
    use crate::game::data::{
        abnormality_data::{AbnormalityDatabase, AbnormalityMetadata},
        artifact_data::ArtifactDatabase,
        bonus_data::BonusDatabase,
        equipment_data::EquipmentDatabase,
        pve_data::PveEncounterDatabase,
        random_event_data::RandomEventDatabase,
        shop_data::ShopDatabase,
        skill_data::SkillDatabase,
        GameDataBase,
    };
    use crate::game::enums::Side;
    use crate::game::stats::UnitStats;
    use crate::game::stats::{StatId, StatModifier, StatModifierKind};
    use std::collections::HashMap;

    fn empty_deck() -> PlayerDeckInfo {
        PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        }
    }

    fn empty_game_data() -> Arc<GameDataBase> {
        let pool = crate::game::data::event_pools::EventPhasePool {
            shops: vec![],
            bonuses: vec![],
            random_events: vec![],
        };
        let event_pools = crate::game::data::event_pools::EventPoolConfig {
            dawn: pool.clone(),
            noon: pool.clone(),
            dusk: pool.clone(),
            midnight: pool.clone(),
            white: pool,
        };

        Arc::new(GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools,
        }))
    }

    fn new_core() -> BattleCore {
        let deck = empty_deck();
        BattleCore::new(&deck, &deck, empty_game_data(), (4, 4), 123)
    }

    fn runtime_unit(unit_id: UnitInstanceId, owner: Side) -> RuntimeUnit {
        RuntimeUnit {
            instance_id: unit_id,
            owner,
            base_uuid: Uuid::nil(),
            stats: UnitStats::with_values(10, 10, 1, 0, 1),
            pos_x_units: 0,
            pos_y_units: 0,
            move_epoch: 0,
            action_state: ActionState::Idle,
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

    fn runtime_unit_with_base(
        unit_id: UnitInstanceId,
        owner: Side,
        base_uuid: Uuid,
    ) -> RuntimeUnit {
        let mut unit = runtime_unit(unit_id, owner);
        unit.base_uuid = base_uuid;
        unit
    }

    fn place_unit(core: &mut BattleCore, unit_id: UnitInstanceId, pos: Position) {
        core.battlefield.place(unit_id, pos).unwrap();
        let unit = core.units.get_mut(&unit_id).unwrap();
        unit.pos_x_units = (pos.x as i64) * TILE_UNITS_PER_TILE as i64;
        unit.pos_y_units = (pos.y as i64) * TILE_UNITS_PER_TILE as i64;
    }

    fn abnormality_with_basic_attack(
        base_uuid: Uuid,
        delivery: DeliveryDef,
        range_tiles: u8,
    ) -> AbnormalityMetadata {
        AbnormalityMetadata {
            id: format!("abnormality-{base_uuid}"),
            uuid: base_uuid,
            name: "test".to_string(),
            risk_level: crate::game::enums::RiskLevel::ZAYIN,
            price: 1,
            max_health: 10,
            attack: 1,
            defense: 0,
            movement: Default::default(),
            basic_attack: crate::game::data::abnormality_data::BasicAttackDef {
                range_tiles,
                delivery,
                ..Default::default()
            },
            resonance: Default::default(),
            skill_id: None,
        }
    }

    fn core_with_abnormalities(abnormalities: Vec<AbnormalityMetadata>) -> BattleCore {
        core_with_skill_data(abnormalities, vec![])
    }

    fn single_step_skill(
        id: &str,
        target: SkillTarget,
        range_tiles: u8,
        focus_time_ms: u32,
        delivery: DeliveryDef,
        effects: Vec<SkillEffectDef>,
    ) -> SkillDef {
        SkillDef {
            id: id.to_string(),
            name: id.to_string(),
            kind: SkillKind::Targeted,
            cast_targeting: SkillCastTargetingDef::FirstStepTarget,
            focus_time_ms,
            focus_permissions: Default::default(),
            steps: vec![SkillStepDef {
                id: "step_01".to_string(),
                delay_ms: 0,
                range_tiles,
                target,
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery,
                effects,
                presentation: SkillPresentationDef::default(),
            }],
        }
    }

    fn core_with_skill_data(
        abnormalities: Vec<AbnormalityMetadata>,
        skills: Vec<SkillDef>,
    ) -> BattleCore {
        let deck = empty_deck();
        let pool = crate::game::data::event_pools::EventPhasePool {
            shops: vec![],
            bonuses: vec![],
            random_events: vec![],
        };
        let event_pools = crate::game::data::event_pools::EventPoolConfig {
            dawn: pool.clone(),
            noon: pool.clone(),
            dusk: pool.clone(),
            midnight: pool.clone(),
            white: pool,
        };

        let game_data = Arc::new(GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(abnormalities)),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(skills)),
            event_pools,
        }));

        BattleCore::new(&deck, &deck, game_data, (6, 6), 123)
    }

    #[test]
    fn add_resonance_clamps_and_sets_pending_cast_only_when_allowed() {
        let mut core = new_core();
        let unit_id: UnitInstanceId = Uuid::from_u128(1).into();
        let mut unit = runtime_unit(unit_id, Side::Player);
        unit.resonance_current = 90;
        unit.resonance_max = 100;
        core.units.insert(unit_id, unit);

        core.add_resonance(unit_id, 10, 0, false);
        assert_eq!(core.units.get(&unit_id).unwrap().resonance_current, 100);
        assert!(!core.units.get(&unit_id).unwrap().pending_cast);

        core.units.get_mut(&unit_id).unwrap().resonance_current = 90;
        core.add_resonance(unit_id, 10, 0, true);
        assert_eq!(core.units.get(&unit_id).unwrap().resonance_current, 100);
        assert!(core.units.get(&unit_id).unwrap().pending_cast);

        // Locked resonance gain should prevent any change.
        core.units
            .get_mut(&unit_id)
            .unwrap()
            .action_locks
            .lock_resonance_gain_until(10);
        core.units.get_mut(&unit_id).unwrap().resonance_current = 0;
        core.add_resonance(unit_id, 10, 0, true);
        assert_eq!(core.units.get(&unit_id).unwrap().resonance_current, 0);
    }

    #[test]
    fn schedule_pending_autocasts_is_deterministic_and_consumes_pending_cause() {
        let mut core = new_core();
        let caster_small: UnitInstanceId = Uuid::from_u128(1).into();
        let caster_large: UnitInstanceId = Uuid::from_u128(2).into();
        assert!(caster_small.as_bytes() < caster_large.as_bytes());

        let mut u1 = runtime_unit(caster_small, Side::Player);
        u1.pending_cast = true;
        let mut u2 = runtime_unit(caster_large, Side::Player);
        u2.pending_cast = true;
        core.units.insert(caster_small, u1);
        core.units.insert(caster_large, u2);

        let cause = TimelineCause::Parent { seq: 42 };
        core.recording_cause_stack.push(cause);

        core.schedule_pending_autocasts(100);

        // Pending flags are consumed and causes are taken.
        assert!(!core.units.get(&caster_small).unwrap().pending_cast);
        assert!(core
            .units
            .get(&caster_small)
            .unwrap()
            .pending_cast_cause
            .is_none());
        assert!(!core.units.get(&caster_large).unwrap().pending_cast);
        assert!(core
            .units
            .get(&caster_large)
            .unwrap()
            .pending_cast_cause
            .is_none());

        let first = core.event_queue.pop().unwrap();
        let second = core.event_queue.pop().unwrap();

        assert!(matches!(
            first,
            BattleEvent::AutoCastStart {
                time_ms: 100,
                caster_instance_id,
                cause: TimelineCause::Parent { seq: 42 },
            } if caster_instance_id == caster_small
        ));
        assert!(matches!(
            second,
            BattleEvent::AutoCastStart {
                time_ms: 100,
                caster_instance_id,
                cause: TimelineCause::Parent { seq: 42 },
            } if caster_instance_id == caster_large
        ));
    }

    #[test]
    fn schedule_pending_autocast_respects_next_action_time_and_dead_units() {
        let mut core = new_core();
        let unit_id: UnitInstanceId = Uuid::from_u128(1).into();
        let mut unit = runtime_unit(unit_id, Side::Player);
        unit.pending_cast = true;
        unit.next_action_time = 200;
        core.units.insert(unit_id, unit);

        core.schedule_pending_autocasts(100);
        assert!(core.event_queue.is_empty());
        assert!(core.units.get(&unit_id).unwrap().pending_cast);

        // Dead unit never schedules.
        core.units.get_mut(&unit_id).unwrap().next_action_time = 0;
        core.units.get_mut(&unit_id).unwrap().stats.current_health = 0;
        core.schedule_pending_autocasts(300);
        assert!(core.event_queue.is_empty());
    }

    #[test]
    fn pending_basic_attack_retargets_when_persisted_target_is_out_of_range() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(1).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(2).into();
        let nearer_enemy_id: UnitInstanceId = Uuid::from_u128(3).into();

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.pending_basic_attack = true;
        attacker.current_target = Some(locked_target_id);

        core.units.insert(attacker_id, attacker);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            nearer_enemy_id,
            runtime_unit(nearer_enemy_id, Side::Opponent),
        );

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, locked_target_id, Position::new(3, 0));
        place_unit(&mut core, nearer_enemy_id, Position::new(1, 0));

        core.try_start_pending_basic_attacks(0);

        assert!(!core.units.get(&attacker_id).unwrap().pending_basic_attack);

        let event = core.event_queue.pop().expect("expected attack start");
        assert!(matches!(
            event,
            BattleEvent::AttackStart {
                attacker_instance_id,
                target_instance_id: Some(target_instance_id),
                schedule_next: true,
                ..
            } if attacker_instance_id == attacker_id && target_instance_id == nearer_enemy_id
        ));
        assert!(core.event_queue.is_empty());
    }

    #[test]
    fn pending_basic_attack_retargets_when_persisted_target_is_dead() {
        let attacker_base_uuid = Uuid::from_u128(0xAA03);
        let mut core = core_with_abnormalities(vec![abnormality_with_basic_attack(
            attacker_base_uuid,
            DeliveryDef::Instant,
            2,
        )]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(4).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(5).into();
        let nearer_enemy_id: UnitInstanceId = Uuid::from_u128(6).into();

        let mut attacker = runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid);
        attacker.pending_basic_attack = true;
        attacker.current_target = Some(locked_target_id);

        core.units.insert(attacker_id, attacker);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            nearer_enemy_id,
            runtime_unit(nearer_enemy_id, Side::Opponent),
        );

        core.units
            .get_mut(&locked_target_id)
            .unwrap()
            .stats
            .current_health = 0;

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, locked_target_id, Position::new(1, 0));
        place_unit(&mut core, nearer_enemy_id, Position::new(1, 1));

        core.try_start_pending_basic_attacks(0);

        assert!(!core.units.get(&attacker_id).unwrap().pending_basic_attack);

        let event = core.event_queue.pop().expect("expected attack start");
        assert!(matches!(
            event,
            BattleEvent::AttackStart {
                attacker_instance_id,
                target_instance_id: Some(target_instance_id),
                schedule_next: true,
                ..
            } if attacker_instance_id == attacker_id && target_instance_id == nearer_enemy_id
        ));
        assert!(core.event_queue.is_empty());
    }

    #[test]
    fn movement_intent_clears_out_of_range_target_and_can_pick_new_in_range_target() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(11).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(12).into();
        let nearer_enemy_id: UnitInstanceId = Uuid::from_u128(13).into();

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.current_target = Some(locked_target_id);

        core.units.insert(attacker_id, attacker);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            nearer_enemy_id,
            runtime_unit(nearer_enemy_id, Side::Opponent),
        );

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, locked_target_id, Position::new(3, 0));
        place_unit(&mut core, nearer_enemy_id, Position::new(1, 0));
        core.compute_movement_intents(0);

        let attacker = core.units.get(&attacker_id).unwrap();
        assert_eq!(attacker.current_target, Some(nearer_enemy_id));
        assert!(matches!(attacker.action_state, ActionState::Idle));
    }

    #[test]
    fn movement_intent_retries_from_idle_when_attack_ring_is_currently_blocked() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(14).into();
        let enemy_id: UnitInstanceId = Uuid::from_u128(15).into();

        core.units
            .insert(attacker_id, runtime_unit(attacker_id, Side::Player));
        core.units
            .insert(enemy_id, runtime_unit(enemy_id, Side::Opponent));

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, enemy_id, Position::new(2, 2));

        for raw_id in 16_u128..24 {
            let blocker_id: UnitInstanceId = Uuid::from_u128(raw_id).into();
            core.units
                .insert(blocker_id, runtime_unit(blocker_id, Side::Player));
        }

        let blocker_positions = [
            Position::new(1, 1),
            Position::new(1, 2),
            Position::new(1, 3),
            Position::new(2, 1),
            Position::new(2, 3),
            Position::new(3, 1),
            Position::new(3, 2),
            Position::new(3, 3),
        ];
        for (offset, pos) in blocker_positions.into_iter().enumerate() {
            let blocker_id: UnitInstanceId = Uuid::from_u128(16 + offset as u128).into();
            place_unit(&mut core, blocker_id, pos);
        }
        core.compute_movement_intents(100);
        assert!(matches!(
            core.units.get(&attacker_id).unwrap().action_state,
            ActionState::Blocked { until_ms: 130, .. }
        ));
        assert!(core.event_queue.iter().any(|event| matches!(
            event,
            BattleEvent::MovementIntent { time_ms } if *time_ms == 130
        )));
    }

    #[test]
    fn movement_intent_prefers_forward_step_over_equal_diagonal_option() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(141).into();
        let enemy_id: UnitInstanceId = Uuid::from_u128(142).into();

        core.units
            .insert(attacker_id, runtime_unit(attacker_id, Side::Player));
        core.units
            .insert(enemy_id, runtime_unit(enemy_id, Side::Opponent));

        place_unit(&mut core, attacker_id, Position::new(1, 3));
        place_unit(&mut core, enemy_id, Position::new(0, 1));

        core.compute_movement_intents(0);

        let attacker = core.units.get(&attacker_id).unwrap();
        let ActionState::Moving(state) = &attacker.action_state else {
            panic!("expected movement state, got {:?}", attacker.action_state);
        };

        assert_eq!(state.step_to, Position::new(1, 2));
    }

    #[test]
    fn movement_intent_prefers_straight_follow_up_over_equal_lateral_attack_tile() {
        let mut core = core_with_abnormalities(vec![abnormality_with_basic_attack(
            Uuid::from_u128(0xB001),
            DeliveryDef::Instant,
            1,
        )]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(143).into();
        let enemy_id: UnitInstanceId = Uuid::from_u128(144).into();

        let mut attacker =
            runtime_unit_with_base(attacker_id, Side::Player, Uuid::from_u128(0xB001));
        attacker.current_target = Some(enemy_id);

        core.units.insert(attacker_id, attacker);
        core.units
            .insert(enemy_id, runtime_unit(enemy_id, Side::Opponent));

        place_unit(&mut core, attacker_id, Position::new(1, 4));
        place_unit(&mut core, enemy_id, Position::new(0, 1));

        core.compute_movement_intents(0);

        let attacker = core.units.get(&attacker_id).unwrap();
        let ActionState::Moving(state) = &attacker.action_state else {
            panic!("expected movement state, got {:?}", attacker.action_state);
        };

        assert_eq!(state.step_to, Position::new(1, 3));
        assert_eq!(state.path.get(2).copied(), Some(Position::new(1, 2)));
        assert_eq!(state.reserved_destination, Some(Position::new(1, 2)));
    }

    #[test]
    fn movement_intent_prefers_direct_enemy_tile_engage_approach_when_adjacent_continuous_melee() {
        let attacker_base_uuid = Uuid::from_u128(0xB002);
        let mut core = core_with_abnormalities(vec![abnormality_with_basic_attack(
            attacker_base_uuid,
            DeliveryDef::Instant,
            1,
        )]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(145).into();
        let enemy_id: UnitInstanceId = Uuid::from_u128(146).into();

        let mut attacker = runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid);
        attacker.current_target = Some(enemy_id);

        let enemy = runtime_unit_with_base(enemy_id, Side::Opponent, attacker_base_uuid);

        core.units.insert(attacker_id, attacker);
        core.units.insert(enemy_id, enemy);

        place_unit(&mut core, attacker_id, Position::new(1, 3));
        place_unit(&mut core, enemy_id, Position::new(1, 4));
        core.units.get_mut(&attacker_id).unwrap().pos_y_units = 2_500_000;
        core.units.get_mut(&enemy_id).unwrap().pos_y_units = 4_500_000;

        core.compute_movement_intents(0);

        let attacker = core.units.get(&attacker_id).unwrap();
        let ActionState::Moving(state) = &attacker.action_state else {
            panic!("expected movement state, got {:?}", attacker.action_state);
        };

        assert_eq!(state.step_to, Position::new(1, 4));
        assert_eq!(state.path, vec![Position::new(1, 3), Position::new(1, 4)]);
        assert_eq!(state.reserved_destination, Some(Position::new(1, 4)));
    }

    #[test]
    fn movement_intent_repositions_when_locked_target_only_has_lateral_entry() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(151).into();
        let enemy_id: UnitInstanceId = Uuid::from_u128(152).into();
        let blocker_id: UnitInstanceId = Uuid::from_u128(153).into();

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.current_target = Some(enemy_id);

        core.units.insert(attacker_id, attacker);
        core.units
            .insert(enemy_id, runtime_unit(enemy_id, Side::Opponent));
        core.units
            .insert(blocker_id, runtime_unit(blocker_id, Side::Player));

        place_unit(&mut core, attacker_id, Position::new(1, 3));
        place_unit(&mut core, enemy_id, Position::new(1, 1));
        place_unit(&mut core, blocker_id, Position::new(1, 2));

        core.compute_movement_intents(100);

        let attacker = core.units.get(&attacker_id).unwrap();
        assert_eq!(attacker.current_target, Some(enemy_id));
        let ActionState::Moving(state) = &attacker.action_state else {
            panic!(
                "expected repositioning movement, got {:?}",
                attacker.action_state
            );
        };
        assert_ne!(state.step_to, Position::new(1, 2));
    }

    #[test]
    fn movement_intent_retargets_when_only_other_enemy_is_open() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(161).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(162).into();
        let alternate_enemy_id: UnitInstanceId = Uuid::from_u128(163).into();

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.current_target = Some(locked_target_id);
        core.units.insert(attacker_id, attacker);

        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            alternate_enemy_id,
            runtime_unit(alternate_enemy_id, Side::Opponent),
        );

        place_unit(&mut core, attacker_id, Position::new(1, 3));
        place_unit(&mut core, locked_target_id, Position::new(1, 0));
        place_unit(&mut core, alternate_enemy_id, Position::new(3, 1));

        for (raw_id, pos) in [
            (170_u128, Position::new(0, 0)),
            (171_u128, Position::new(0, 1)),
            (172_u128, Position::new(1, 1)),
            (173_u128, Position::new(2, 0)),
            (174_u128, Position::new(2, 1)),
        ] {
            let blocker_id: UnitInstanceId = Uuid::from_u128(raw_id).into();
            core.units
                .insert(blocker_id, runtime_unit(blocker_id, Side::Player));
            place_unit(&mut core, blocker_id, pos);
        }

        core.compute_movement_intents(100);

        let attacker = core.units.get(&attacker_id).unwrap();
        assert_eq!(attacker.current_target, Some(alternate_enemy_id));
        assert!(
            matches!(attacker.action_state, ActionState::Moving(_)),
            "unexpected attacker state: {:?}",
            attacker.action_state
        );
    }

    #[test]
    fn movement_intent_repositions_when_forward_slot_is_claimed_and_only_lateral_remains() {
        let mut core = new_core();
        let claimer_id: UnitInstanceId = Uuid::from_u128(180).into();
        let attacker_id: UnitInstanceId = Uuid::from_u128(181).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(182).into();
        let claimer_target_id: UnitInstanceId = Uuid::from_u128(183).into();
        let blocker_id: UnitInstanceId = Uuid::from_u128(184).into();

        core.units
            .insert(claimer_id, runtime_unit(claimer_id, Side::Player));
        core.units
            .insert(blocker_id, runtime_unit(blocker_id, Side::Player));

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.current_target = Some(locked_target_id);
        core.units.insert(attacker_id, attacker);

        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            claimer_target_id,
            runtime_unit(claimer_target_id, Side::Opponent),
        );

        place_unit(&mut core, claimer_id, Position::new(2, 3));
        place_unit(&mut core, attacker_id, Position::new(1, 3));
        place_unit(&mut core, locked_target_id, Position::new(1, 0));
        place_unit(&mut core, claimer_target_id, Position::new(1, 1));
        place_unit(&mut core, blocker_id, Position::new(2, 2));

        core.compute_movement_intents(100);

        let claimer = core.units.get(&claimer_id).unwrap();
        let ActionState::Moving(claimer_move) = &claimer.action_state else {
            panic!(
                "expected claimer to move first, got {:?}",
                claimer.action_state
            );
        };
        assert_eq!(claimer_move.step_to, Position::new(1, 2));

        let attacker = core.units.get(&attacker_id).unwrap();
        assert_eq!(attacker.current_target, Some(claimer_target_id));
        let ActionState::Moving(state) = &attacker.action_state else {
            panic!(
                "expected repositioning move after forward slot claim, got {:?}",
                attacker.action_state
            );
        };
        assert_ne!(state.step_to, Position::new(1, 2));
    }

    #[test]
    fn movement_intent_prioritizes_holding_unit_over_fresh_idle_competitor() {
        let mut core = new_core();
        let holder_id: UnitInstanceId = Uuid::from_u128(200).into();
        let idle_competitor_id: UnitInstanceId = Uuid::from_u128(199).into();
        let target_id: UnitInstanceId = Uuid::from_u128(201).into();

        let mut holder = runtime_unit(holder_id, Side::Player);
        holder.current_target = Some(target_id);
        holder.action_state = ActionState::Holding {
            until_ms: 100,
            repath_counter: 2,
        };
        core.units.insert(holder_id, holder);
        core.units.insert(
            idle_competitor_id,
            runtime_unit(idle_competitor_id, Side::Player),
        );
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));

        place_unit(&mut core, holder_id, Position::new(1, 3));
        place_unit(&mut core, idle_competitor_id, Position::new(2, 3));
        place_unit(&mut core, target_id, Position::new(1, 1));

        core.compute_movement_intents(100);

        let holder = core.units.get(&holder_id).unwrap();
        let ActionState::Moving(holder_move) = &holder.action_state else {
            panic!(
                "expected holding unit to move, got {:?}",
                holder.action_state
            );
        };
        assert_eq!(holder_move.step_to, Position::new(1, 2));

        let idle_competitor = core.units.get(&idle_competitor_id).unwrap();
        assert!(matches!(
            idle_competitor.action_state,
            ActionState::Holding { until_ms: 120, .. }
                | ActionState::Yielding { until_ms: 110, .. }
                | ActionState::Moving(_)
        ));
    }

    #[test]
    fn attack_start_keeps_persisted_target_when_it_is_still_in_range() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(31).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(32).into();
        let nearer_enemy_id: UnitInstanceId = Uuid::from_u128(33).into();

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.current_target = Some(locked_target_id);

        core.units.insert(attacker_id, attacker);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            nearer_enemy_id,
            runtime_unit(nearer_enemy_id, Side::Opponent),
        );

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, locked_target_id, Position::new(1, 0));
        place_unit(&mut core, nearer_enemy_id, Position::new(1, 1));

        core.process_event(
            BattleEvent::AttackStart {
                time_ms: 0,
                attacker_instance_id: attacker_id,
                target_instance_id: None,
                schedule_next: true,
                cause: TimelineCause::default(),
            },
            0,
        )
        .unwrap();

        assert_eq!(
            core.units.get(&attacker_id).unwrap().current_target,
            Some(locked_target_id)
        );
    }

    #[test]
    fn attack_start_hint_does_not_override_persisted_target_when_it_is_still_in_range() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(34).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(35).into();
        let hinted_target_id: UnitInstanceId = Uuid::from_u128(36).into();

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.current_target = Some(locked_target_id);

        core.units.insert(attacker_id, attacker);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            hinted_target_id,
            runtime_unit(hinted_target_id, Side::Opponent),
        );

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, locked_target_id, Position::new(1, 0));
        place_unit(&mut core, hinted_target_id, Position::new(1, 1));

        core.process_event(
            BattleEvent::AttackStart {
                time_ms: 0,
                attacker_instance_id: attacker_id,
                target_instance_id: Some(hinted_target_id),
                schedule_next: false,
                cause: TimelineCause::default(),
            },
            0,
        )
        .unwrap();

        assert_eq!(
            core.units.get(&attacker_id).unwrap().current_target,
            Some(locked_target_id)
        );
    }

    #[test]
    fn attack_start_retargets_to_nearest_in_range_when_persisted_target_is_dead() {
        let attacker_base_uuid = Uuid::from_u128(0xAA04);
        let mut core = core_with_abnormalities(vec![abnormality_with_basic_attack(
            attacker_base_uuid,
            DeliveryDef::Instant,
            2,
        )]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(37).into();
        let dead_target_id: UnitInstanceId = Uuid::from_u128(38).into();
        let nearer_enemy_id: UnitInstanceId = Uuid::from_u128(39).into();

        let mut attacker = runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid);
        attacker.current_target = Some(dead_target_id);

        core.units.insert(attacker_id, attacker);
        core.units
            .insert(dead_target_id, runtime_unit(dead_target_id, Side::Opponent));
        core.units.insert(
            nearer_enemy_id,
            runtime_unit(nearer_enemy_id, Side::Opponent),
        );

        core.units
            .get_mut(&dead_target_id)
            .unwrap()
            .stats
            .current_health = 0;

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, dead_target_id, Position::new(1, 0));
        place_unit(&mut core, nearer_enemy_id, Position::new(1, 1));

        core.process_event(
            BattleEvent::AttackStart {
                time_ms: 0,
                attacker_instance_id: attacker_id,
                target_instance_id: None,
                schedule_next: true,
                cause: TimelineCause::default(),
            },
            0,
        )
        .unwrap();

        assert_eq!(
            core.units.get(&attacker_id).unwrap().current_target,
            Some(nearer_enemy_id)
        );
    }

    #[test]
    fn attack_start_prefers_straight_enemy_over_equal_range_diagonal_enemy() {
        let attacker_base_uuid = Uuid::from_u128(0xAA13);
        let enemy_base_uuid = Uuid::from_u128(0xAA14);
        let mut core = core_with_abnormalities(vec![
            abnormality_with_basic_attack(attacker_base_uuid, DeliveryDef::Instant, 1),
            abnormality_with_basic_attack(enemy_base_uuid, DeliveryDef::Instant, 1),
        ]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(40).into();
        let diagonal_enemy_id: UnitInstanceId = Uuid::from_u128(41).into();
        let straight_enemy_id: UnitInstanceId = Uuid::from_u128(42).into();

        core.units.insert(
            attacker_id,
            runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid),
        );
        core.units.insert(
            diagonal_enemy_id,
            runtime_unit_with_base(diagonal_enemy_id, Side::Opponent, enemy_base_uuid),
        );
        core.units.insert(
            straight_enemy_id,
            runtime_unit_with_base(straight_enemy_id, Side::Opponent, enemy_base_uuid),
        );

        place_unit(&mut core, attacker_id, Position::new(2, 2));
        place_unit(&mut core, diagonal_enemy_id, Position::new(1, 1));
        place_unit(&mut core, straight_enemy_id, Position::new(2, 1));

        core.process_event(
            BattleEvent::AttackStart {
                time_ms: 0,
                attacker_instance_id: attacker_id,
                target_instance_id: None,
                schedule_next: true,
                cause: TimelineCause::default(),
            },
            0,
        )
        .unwrap();

        assert_eq!(
            core.units.get(&attacker_id).unwrap().current_target,
            Some(straight_enemy_id)
        );
    }

    #[test]
    fn choose_attack_target_in_range_prefers_melee_enemy_when_distance_is_equal() {
        let attacker_base_uuid = Uuid::from_u128(0xAA05);
        let melee_enemy_base_uuid = Uuid::from_u128(0xAA06);
        let ranged_enemy_base_uuid = Uuid::from_u128(0xAA07);
        let mut core = core_with_abnormalities(vec![
            abnormality_with_basic_attack(
                attacker_base_uuid,
                DeliveryDef::Projectile {
                    speed_units_per_ms: 1_000,
                },
                3,
            ),
            abnormality_with_basic_attack(melee_enemy_base_uuid, DeliveryDef::Instant, 1),
            abnormality_with_basic_attack(
                ranged_enemy_base_uuid,
                DeliveryDef::Projectile {
                    speed_units_per_ms: 1_000,
                },
                3,
            ),
        ]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(80).into();
        let melee_enemy_id: UnitInstanceId = Uuid::from_u128(81).into();
        let ranged_enemy_id: UnitInstanceId = Uuid::from_u128(82).into();

        core.units.insert(
            attacker_id,
            runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid),
        );
        core.units.insert(
            melee_enemy_id,
            runtime_unit_with_base(melee_enemy_id, Side::Opponent, melee_enemy_base_uuid),
        );
        core.units.insert(
            ranged_enemy_id,
            runtime_unit_with_base(ranged_enemy_id, Side::Opponent, ranged_enemy_base_uuid),
        );

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, melee_enemy_id, Position::new(1, 0));
        place_unit(&mut core, ranged_enemy_id, Position::new(0, 1));

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(melee_enemy_id)
        );
    }

    #[test]
    fn choose_attack_target_in_range_prefers_straight_enemy_over_equal_range_diagonal_enemy() {
        let attacker_base_uuid = Uuid::from_u128(0xAA10);
        let enemy_base_uuid = Uuid::from_u128(0xAA11);
        let mut core = core_with_abnormalities(vec![
            abnormality_with_basic_attack(attacker_base_uuid, DeliveryDef::Instant, 1),
            abnormality_with_basic_attack(enemy_base_uuid, DeliveryDef::Instant, 1),
        ]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(91).into();
        let diagonal_enemy_id: UnitInstanceId = Uuid::from_u128(92).into();
        let straight_enemy_id: UnitInstanceId = Uuid::from_u128(93).into();

        core.units.insert(
            attacker_id,
            runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid),
        );
        core.units.insert(
            diagonal_enemy_id,
            runtime_unit_with_base(diagonal_enemy_id, Side::Opponent, enemy_base_uuid),
        );
        core.units.insert(
            straight_enemy_id,
            runtime_unit_with_base(straight_enemy_id, Side::Opponent, enemy_base_uuid),
        );

        place_unit(&mut core, attacker_id, Position::new(2, 2));
        place_unit(&mut core, diagonal_enemy_id, Position::new(1, 1));
        place_unit(&mut core, straight_enemy_id, Position::new(2, 1));

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(straight_enemy_id)
        );
    }

    #[test]
    fn choose_enemy_target_in_tile_range_prefers_melee_enemy_when_distance_is_equal() {
        let melee_enemy_base_uuid = Uuid::from_u128(0xAA08);
        let ranged_enemy_base_uuid = Uuid::from_u128(0xAA09);
        let mut core = core_with_abnormalities(vec![
            abnormality_with_basic_attack(melee_enemy_base_uuid, DeliveryDef::Instant, 1),
            abnormality_with_basic_attack(
                ranged_enemy_base_uuid,
                DeliveryDef::Projectile {
                    speed_units_per_ms: 1_000,
                },
                3,
            ),
        ]);
        let melee_enemy_id: UnitInstanceId = Uuid::from_u128(83).into();
        let ranged_enemy_id: UnitInstanceId = Uuid::from_u128(84).into();

        core.units.insert(
            melee_enemy_id,
            runtime_unit_with_base(melee_enemy_id, Side::Opponent, melee_enemy_base_uuid),
        );
        core.units.insert(
            ranged_enemy_id,
            runtime_unit_with_base(ranged_enemy_id, Side::Opponent, ranged_enemy_base_uuid),
        );

        place_unit(&mut core, melee_enemy_id, Position::new(1, 0));
        place_unit(&mut core, ranged_enemy_id, Position::new(0, 1));

        assert_eq!(
            core.choose_enemy_target_in_tile_range(Side::Player, Position::new(0, 0), 3),
            Some(melee_enemy_id)
        );
    }

    #[test]
    fn choose_enemy_target_in_tile_range_prefers_straight_enemy_over_equal_range_diagonal_enemy() {
        let enemy_base_uuid = Uuid::from_u128(0xAA12);
        let mut core = core_with_abnormalities(vec![abnormality_with_basic_attack(
            enemy_base_uuid,
            DeliveryDef::Instant,
            1,
        )]);
        let diagonal_enemy_id: UnitInstanceId = Uuid::from_u128(94).into();
        let straight_enemy_id: UnitInstanceId = Uuid::from_u128(95).into();

        core.units.insert(
            diagonal_enemy_id,
            runtime_unit_with_base(diagonal_enemy_id, Side::Opponent, enemy_base_uuid),
        );
        core.units.insert(
            straight_enemy_id,
            runtime_unit_with_base(straight_enemy_id, Side::Opponent, enemy_base_uuid),
        );

        place_unit(&mut core, diagonal_enemy_id, Position::new(1, 1));
        place_unit(&mut core, straight_enemy_id, Position::new(2, 1));

        assert_eq!(
            core.choose_enemy_target_in_tile_range(Side::Player, Position::new(2, 2), 1),
            Some(straight_enemy_id)
        );
    }

    #[test]
    fn movement_intent_prefers_melee_enemy_when_chase_distance_is_equal() {
        let attacker_base_uuid = Uuid::from_u128(0xAA0A);
        let melee_enemy_base_uuid = Uuid::from_u128(0xAA0B);
        let ranged_enemy_base_uuid = Uuid::from_u128(0xAA0C);
        let mut core = core_with_abnormalities(vec![
            abnormality_with_basic_attack(attacker_base_uuid, DeliveryDef::Instant, 1),
            abnormality_with_basic_attack(melee_enemy_base_uuid, DeliveryDef::Instant, 1),
            abnormality_with_basic_attack(
                ranged_enemy_base_uuid,
                DeliveryDef::Projectile {
                    speed_units_per_ms: 1_000,
                },
                3,
            ),
        ]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(85).into();
        let melee_enemy_id: UnitInstanceId = Uuid::from_u128(86).into();
        let ranged_enemy_id: UnitInstanceId = Uuid::from_u128(87).into();

        core.units.insert(
            attacker_id,
            runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid),
        );
        core.units.insert(
            melee_enemy_id,
            runtime_unit_with_base(melee_enemy_id, Side::Opponent, melee_enemy_base_uuid),
        );
        core.units.insert(
            ranged_enemy_id,
            runtime_unit_with_base(ranged_enemy_id, Side::Opponent, ranged_enemy_base_uuid),
        );

        place_unit(&mut core, attacker_id, Position::new(1, 3));
        place_unit(&mut core, melee_enemy_id, Position::new(0, 1));
        place_unit(&mut core, ranged_enemy_id, Position::new(2, 1));

        core.compute_movement_intents(100);

        let attacker = core.units.get(&attacker_id).unwrap();
        assert_eq!(attacker.current_target, Some(melee_enemy_id));
    }

    #[test]
    fn movement_intent_prefers_straight_enemy_over_equal_range_diagonal_enemy() {
        let attacker_base_uuid = Uuid::from_u128(0xAA0D);
        let enemy_base_uuid = Uuid::from_u128(0xAA0E);
        let mut core = core_with_abnormalities(vec![
            abnormality_with_basic_attack(attacker_base_uuid, DeliveryDef::Instant, 1),
            abnormality_with_basic_attack(enemy_base_uuid, DeliveryDef::Instant, 1),
        ]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(88).into();
        let diagonal_enemy_id: UnitInstanceId = Uuid::from_u128(89).into();
        let straight_enemy_id: UnitInstanceId = Uuid::from_u128(90).into();

        core.units.insert(
            attacker_id,
            runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid),
        );
        core.units.insert(
            diagonal_enemy_id,
            runtime_unit_with_base(diagonal_enemy_id, Side::Opponent, enemy_base_uuid),
        );
        core.units.insert(
            straight_enemy_id,
            runtime_unit_with_base(straight_enemy_id, Side::Opponent, enemy_base_uuid),
        );

        place_unit(&mut core, attacker_id, Position::new(5, 5));
        place_unit(&mut core, diagonal_enemy_id, Position::new(4, 2));
        place_unit(&mut core, straight_enemy_id, Position::new(5, 2));

        core.compute_movement_intents(100);

        let attacker = core.units.get(&attacker_id).unwrap();
        assert_eq!(attacker.current_target, Some(straight_enemy_id));
        let ActionState::Moving(state) = &attacker.action_state else {
            panic!("expected movement state, got {:?}", attacker.action_state);
        };
        assert_eq!(state.step_to, Position::new(5, 4));
    }

    #[test]
    fn instant_basic_attack_uses_continuous_range_for_selection_and_resolve() {
        let attacker_base_uuid = Uuid::from_u128(0xAA01);
        let mut core = core_with_abnormalities(vec![abnormality_with_basic_attack(
            attacker_base_uuid,
            DeliveryDef::Instant,
            1,
        )]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(61).into();
        let target_id: UnitInstanceId = Uuid::from_u128(62).into();

        core.units.insert(
            attacker_id,
            runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid),
        );
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, target_id, Position::new(1, 1));

        assert_eq!(core.choose_attack_target_in_range(attacker_id), None);
        assert!(!core.resolve_basic_attack(attacker_id, target_id, 0));

        let attacker = core.units.get_mut(&attacker_id).unwrap();
        attacker.pos_x_units = TILE_UNITS_PER_TILE as i64;
        attacker.pos_y_units = TILE_UNITS_PER_TILE as i64;

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(target_id)
        );
        assert!(core.resolve_basic_attack(attacker_id, target_id, 1));
    }

    #[test]
    fn projectile_basic_attack_keeps_tile_range_behavior() {
        let attacker_base_uuid = Uuid::from_u128(0xAA02);
        let mut core = core_with_abnormalities(vec![abnormality_with_basic_attack(
            attacker_base_uuid,
            DeliveryDef::Projectile {
                speed_units_per_ms: 1_000,
            },
            1,
        )]);
        let attacker_id: UnitInstanceId = Uuid::from_u128(71).into();
        let target_id: UnitInstanceId = Uuid::from_u128(72).into();

        core.units.insert(
            attacker_id,
            runtime_unit_with_base(attacker_id, Side::Player, attacker_base_uuid),
        );
        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, target_id, Position::new(1, 1));

        let attacker = core.units.get_mut(&attacker_id).unwrap();
        attacker.pos_x_units = -(TILE_UNITS_PER_TILE as i64);
        attacker.pos_y_units = -(TILE_UNITS_PER_TILE as i64);

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(target_id)
        );
        assert!(core.resolve_basic_attack(attacker_id, target_id, 0));
    }

    #[test]
    fn hard_cc_clears_locked_target_and_reacquires_after_release() {
        let mut core = new_core();
        let attacker_id: UnitInstanceId = Uuid::from_u128(21).into();
        let old_target_id: UnitInstanceId = Uuid::from_u128(22).into();
        let new_target_id: UnitInstanceId = Uuid::from_u128(23).into();

        let mut attacker = runtime_unit(attacker_id, Side::Player);
        attacker.current_target = Some(old_target_id);

        core.units.insert(attacker_id, attacker);
        core.units
            .insert(old_target_id, runtime_unit(old_target_id, Side::Opponent));
        core.units
            .insert(new_target_id, runtime_unit(new_target_id, Side::Opponent));

        place_unit(&mut core, attacker_id, Position::new(0, 0));
        place_unit(&mut core, old_target_id, Position::new(3, 0));
        place_unit(&mut core, new_target_id, Position::new(1, 0));

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 0,
                caster_instance_id: new_target_id,
                target_instance_id: attacker_id,
                buff_id: BuffId::from_name("stun"),
                duration_ms: 50,
                cause: TimelineCause::default(),
            },
            0,
        )
        .unwrap();

        assert_eq!(core.units.get(&attacker_id).unwrap().current_target, None);

        core.process_event(
            BattleEvent::AttackStart {
                time_ms: 51,
                attacker_instance_id: attacker_id,
                target_instance_id: None,
                schedule_next: true,
                cause: TimelineCause::default(),
            },
            51,
        )
        .unwrap();

        assert_eq!(
            core.units.get(&attacker_id).unwrap().current_target,
            Some(new_target_id)
        );
    }

    #[test]
    fn current_target_skill_rule_prefers_locked_target() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(41).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(42).into();
        let other_target_id: UnitInstanceId = Uuid::from_u128(43).into();

        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.current_target = Some(locked_target_id);
        core.units.insert(caster_id, caster);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            other_target_id,
            runtime_unit(other_target_id, Side::Opponent),
        );

        place_unit(&mut core, caster_id, Position::new(0, 0));
        place_unit(&mut core, locked_target_id, Position::new(1, 0));
        place_unit(&mut core, other_target_id, Position::new(1, 1));

        let skill = single_step_skill(
            "current_target",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::CurrentTarget,
            },
            2,
            0,
            DeliveryDef::Instant,
            vec![],
        );

        let target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0));

        assert!(matches!(
            target,
            Some(crate::game::battle::timeline::SkillCastTarget::Unit { unit_instance_id })
                if unit_instance_id == locked_target_id
        ));
    }

    #[test]
    fn lowest_health_enemy_skill_rule_prefers_weaker_target() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(51).into();
        let tank_id: UnitInstanceId = Uuid::from_u128(52).into();
        let weak_id: UnitInstanceId = Uuid::from_u128(53).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        let mut tank = runtime_unit(tank_id, Side::Opponent);
        tank.stats.current_health = 20;
        let mut weak = runtime_unit(weak_id, Side::Opponent);
        weak.stats.current_health = 5;
        core.units.insert(tank_id, tank);
        core.units.insert(weak_id, weak);

        core.battlefield
            .place(caster_id, Position::new(0, 0))
            .unwrap();
        core.battlefield
            .place(tank_id, Position::new(1, 0))
            .unwrap();
        core.battlefield
            .place(weak_id, Position::new(1, 1))
            .unwrap();

        let skill = single_step_skill(
            "lowest_health",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::LowestHealthEnemy,
            },
            2,
            0,
            DeliveryDef::Instant,
            vec![],
        );

        let target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, Position::new(0, 0));

        assert!(matches!(
            target,
            Some(crate::game::battle::timeline::SkillCastTarget::Unit { unit_instance_id })
                if unit_instance_id == weak_id
        ));
    }

    #[test]
    fn line_enemy_area_hits_units_along_anchor_direction() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(61).into();
        let enemy_front_id: UnitInstanceId = Uuid::from_u128(62).into();
        let enemy_back_id: UnitInstanceId = Uuid::from_u128(63).into();
        let enemy_offline_id: UnitInstanceId = Uuid::from_u128(64).into();

        core.units
            .insert(caster_id, runtime_unit(caster_id, Side::Player));
        core.units
            .insert(enemy_front_id, runtime_unit(enemy_front_id, Side::Opponent));
        core.units
            .insert(enemy_back_id, runtime_unit(enemy_back_id, Side::Opponent));
        core.units.insert(
            enemy_offline_id,
            runtime_unit(enemy_offline_id, Side::Opponent),
        );

        let caster_pos = Position::new(1, 1);
        core.battlefield.place(caster_id, caster_pos).unwrap();
        core.battlefield
            .place(enemy_front_id, Position::new(2, 1))
            .unwrap();
        core.battlefield
            .place(enemy_back_id, Position::new(3, 1))
            .unwrap();
        core.battlefield
            .place(enemy_offline_id, Position::new(2, 2))
            .unwrap();

        let skill = single_step_skill(
            "line",
            SkillTarget::Enemies {
                area: SkillArea::Line { length_tiles: 3 },
            },
            3,
            0,
            DeliveryDef::Instant,
            vec![],
        );

        let start_target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, caster_pos);
        let targets = core.resolve_skill_step_targets(
            0,
            caster_id,
            skill.first_step().unwrap(),
            start_target,
        );

        assert!(targets.contains(&enemy_front_id));
        assert!(targets.contains(&enemy_back_id));
        assert!(!targets.contains(&enemy_offline_id));
    }

    #[test]
    fn targeted_execute_keeps_locked_target_when_target_leaves_range() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(65).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(66).into();
        let other_target_id: UnitInstanceId = Uuid::from_u128(67).into();

        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.current_target = Some(locked_target_id);
        core.units.insert(caster_id, caster);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            other_target_id,
            runtime_unit(other_target_id, Side::Opponent),
        );

        let caster_pos = Position::new(0, 0);
        core.battlefield.place(caster_id, caster_pos).unwrap();
        core.battlefield
            .place(locked_target_id, Position::new(1, 0))
            .unwrap();
        core.battlefield
            .place(other_target_id, Position::new(1, 1))
            .unwrap();

        let skill = single_step_skill(
            "locked_target_execute",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::CurrentTarget,
            },
            2,
            50,
            DeliveryDef::Instant,
            vec![],
        );

        let start_target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, caster_pos);
        core.battlefield
            .move_unit(locked_target_id, Position::new(3, 3))
            .unwrap();

        let targets = core.resolve_skill_step_targets(
            0,
            caster_id,
            skill.first_step().unwrap(),
            start_target,
        );
        assert_eq!(targets, vec![locked_target_id]);
    }

    #[test]
    fn targeted_execute_does_not_reacquire_when_locked_target_dies() {
        let mut core = new_core();
        let caster_id: UnitInstanceId = Uuid::from_u128(68).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(69).into();
        let other_target_id: UnitInstanceId = Uuid::from_u128(70).into();

        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.current_target = Some(locked_target_id);
        core.units.insert(caster_id, caster);
        core.units.insert(
            locked_target_id,
            runtime_unit(locked_target_id, Side::Opponent),
        );
        core.units.insert(
            other_target_id,
            runtime_unit(other_target_id, Side::Opponent),
        );

        let caster_pos = Position::new(0, 0);
        core.battlefield.place(caster_id, caster_pos).unwrap();
        core.battlefield
            .place(locked_target_id, Position::new(1, 0))
            .unwrap();
        core.battlefield
            .place(other_target_id, Position::new(1, 1))
            .unwrap();

        let skill = single_step_skill(
            "locked_target_death",
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::CurrentTarget,
            },
            2,
            50,
            DeliveryDef::Instant,
            vec![],
        );

        let start_target =
            core.resolve_skill_cast_target(&skill, caster_id, Side::Player, caster_pos);
        core.battlefield.remove(locked_target_id);
        core.units
            .get_mut(&locked_target_id)
            .unwrap()
            .stats
            .current_health = 0;

        let targets = core.resolve_skill_step_targets(
            0,
            caster_id,
            skill.first_step().unwrap(),
            start_target,
        );
        assert!(targets.is_empty());
    }

    #[test]
    fn process_commands_applies_stat_modifier_and_resonance_delta() {
        let mut core = new_core();
        let target_id: UnitInstanceId = Uuid::from_u128(71).into();
        let mut unit = runtime_unit(target_id, Side::Player);
        unit.resonance_current = 50;
        core.units.insert(target_id, unit);

        core.process_commands(
            vec![
                BattleCommand::ApplyModifier {
                    target_id,
                    modifier: StatModifier {
                        stat: StatId::Attack,
                        kind: StatModifierKind::Flat,
                        value: 7,
                    },
                },
                BattleCommand::ModifyResonance {
                    target_id,
                    amount: -20,
                    allow_autocast_when_full: false,
                },
            ],
            100,
        );

        let unit = core.units.get(&target_id).unwrap();
        assert_eq!(unit.stats.attack, 8);
        assert_eq!(unit.resonance_current, 30);
    }

    #[test]
    fn silence_blocks_pending_autocast_until_expire() {
        let caster_base_uuid = Uuid::from_u128(0x9001);
        let skill = single_step_skill(
            "silence_test_skill",
            SkillTarget::SelfUnit,
            1,
            0,
            DeliveryDef::Instant,
            vec![SkillEffectDef::ModifyResonance { amount: -10 }],
        );
        let abnormality = AbnormalityMetadata {
            id: "caster".to_string(),
            uuid: caster_base_uuid,
            name: "caster".to_string(),
            risk_level: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            max_health: 10,
            attack: 1,
            defense: 0,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: Some(skill.id.clone()),
        };

        let mut core = core_with_skill_data(vec![abnormality], vec![skill]);
        let caster_id: UnitInstanceId = Uuid::from_u128(81).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.base_uuid = caster_base_uuid;
        caster.pending_cast = true;
        core.units.insert(caster_id, caster);

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 0,
                caster_instance_id: caster_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("silence"),
                duration_ms: 10,
                cause: TimelineCause::default(),
            },
            0,
        )
        .unwrap();

        core.schedule_pending_autocasts(0);
        assert!(!core
            .event_queue
            .iter()
            .any(|event| matches!(event, BattleEvent::AutoCastStart { .. })));

        core.process_event(
            BattleEvent::BuffExpire {
                time_ms: 10,
                caster_instance_id: caster_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("silence"),
                cause: TimelineCause::default(),
            },
            10,
        )
        .unwrap();

        assert!(core.event_queue.iter().any(|event| matches!(
            event,
            BattleEvent::AutoCastStart {
                time_ms: 11,
                caster_instance_id,
                ..
            } if *caster_instance_id == caster_id
        )));
    }

    #[test]
    fn hard_cc_delays_autocast_completion_until_lock_release() {
        let caster_base_uuid = Uuid::from_u128(0x9002);
        let skill = single_step_skill(
            "delayed_cast_skill",
            SkillTarget::SelfUnit,
            1,
            5,
            DeliveryDef::Instant,
            vec![SkillEffectDef::ModifyResonance { amount: -10 }],
        );
        let abnormality = AbnormalityMetadata {
            id: "caster".to_string(),
            uuid: caster_base_uuid,
            name: "caster".to_string(),
            risk_level: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            max_health: 10,
            attack: 1,
            defense: 0,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: Some(skill.id.clone()),
        };

        let mut core = core_with_skill_data(vec![abnormality], vec![skill]);
        let caster_id: UnitInstanceId = Uuid::from_u128(82).into();
        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.base_uuid = caster_base_uuid;
        caster.resonance_current = 100;
        core.units.insert(caster_id, caster);
        core.battlefield
            .place(caster_id, Position::new(0, 0))
            .unwrap();

        core.process_event(
            BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id: caster_id,
                cause: TimelineCause::default(),
            },
            0,
        )
        .unwrap();

        let scheduled_end = core
            .event_queue
            .pop()
            .expect("missing scheduled AutoCastEnd");
        let end_cause = match scheduled_end {
            BattleEvent::AutoCastEnd {
                time_ms,
                caster_instance_id,
                cause,
            } => {
                assert_eq!(time_ms, 5);
                assert_eq!(caster_instance_id, caster_id);
                cause
            }
            other => panic!("expected AutoCastEnd, got {other:?}"),
        };

        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1,
                caster_instance_id: caster_id,
                target_instance_id: caster_id,
                buff_id: BuffId::from_name("stun"),
                duration_ms: 10,
                cause: TimelineCause::default(),
            },
            1,
        )
        .unwrap();

        core.process_event(
            BattleEvent::AutoCastEnd {
                time_ms: 5,
                caster_instance_id: caster_id,
                cause: end_cause,
            },
            5,
        )
        .unwrap();

        assert!(!core
            .timeline
            .entries
            .iter()
            .any(|entry| matches!(entry.event, TimelineEvent::AbilityCast { .. })));

        let buff_expire = core.event_queue.pop().expect("missing BuffExpire");
        assert!(matches!(
            buff_expire,
            BattleEvent::BuffExpire { time_ms: 11, .. }
        ));
        core.process_event(buff_expire, 11).unwrap();

        let delayed_end = core.event_queue.pop().expect("missing delayed AutoCastEnd");
        assert!(matches!(
            delayed_end,
            BattleEvent::AutoCastEnd { time_ms: 12, .. }
        ));
        core.process_event(delayed_end, 12).unwrap();

        assert!(core.timeline.entries.iter().any(|entry| {
            entry.time_ms == 12
                && matches!(
                    entry.event,
                    TimelineEvent::AbilityCast {
                        caster_instance_id,
                        ..
                    } if caster_instance_id == caster_id
                )
        }));
    }

    #[test]
    fn autocast_completion_resets_auto_attack_cycle_to_full_interval() {
        let caster_base_uuid = Uuid::from_u128(0x9003);
        let skill = single_step_skill(
            "recovery_cast_skill",
            SkillTarget::SelfUnit,
            1,
            5,
            DeliveryDef::Instant,
            vec![SkillEffectDef::ModifyResonance { amount: -10 }],
        );
        let abnormality = AbnormalityMetadata {
            id: "caster".to_string(),
            uuid: caster_base_uuid,
            name: "caster".to_string(),
            risk_level: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            max_health: 10,
            attack: 1,
            defense: 0,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: Some(skill.id.clone()),
        };

        let mut core = core_with_skill_data(vec![abnormality], vec![skill]);
        let caster_id: UnitInstanceId = Uuid::from_u128(83).into();
        let target_id: UnitInstanceId = Uuid::from_u128(84).into();

        let mut caster = runtime_unit(caster_id, Side::Player);
        caster.base_uuid = caster_base_uuid;
        caster.stats.attack_interval_ms = 10;
        core.units.insert(caster_id, caster);
        core.battlefield
            .place(caster_id, Position::new(0, 0))
            .unwrap();

        core.units
            .insert(target_id, runtime_unit(target_id, Side::Opponent));
        core.battlefield
            .place(target_id, Position::new(1, 0))
            .unwrap();

        core.process_event(
            BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id: caster_id,
                cause: TimelineCause::default(),
            },
            0,
        )
        .unwrap();

        let scheduled_end = core
            .event_queue
            .pop()
            .expect("missing scheduled AutoCastEnd");
        let end_cause = match scheduled_end {
            BattleEvent::AutoCastEnd {
                time_ms,
                caster_instance_id,
                cause,
            } => {
                assert_eq!(time_ms, 5);
                assert_eq!(caster_instance_id, caster_id);
                cause
            }
            other => panic!("expected AutoCastEnd, got {other:?}"),
        };

        core.process_event(
            BattleEvent::AutoCastEnd {
                time_ms: 5,
                caster_instance_id: caster_id,
                cause: end_cause,
            },
            5,
        )
        .unwrap();

        core.process_event(
            BattleEvent::AttackStart {
                time_ms: 5,
                attacker_instance_id: caster_id,
                target_instance_id: Some(target_id),
                schedule_next: true,
                cause: TimelineCause::Root {
                    kind: crate::game::battle::timeline::TimelineRootCause::Period,
                },
            },
            5,
        )
        .unwrap();

        assert!(core.event_queue.iter().any(|event| matches!(
            event,
            BattleEvent::AttackStart {
                time_ms: 15,
                attacker_instance_id,
                schedule_next: true,
                ..
            } if *attacker_instance_id == caster_id
        )));
        assert!(!core.timeline.entries.iter().any(|entry| {
            entry.time_ms == 5
                && matches!(
                    entry.event,
                    TimelineEvent::AttackStart {
                        attacker_instance_id,
                        ..
                    } if attacker_instance_id == caster_id
                )
        }));
    }
}
