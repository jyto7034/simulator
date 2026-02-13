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
pub mod triggers;
pub mod types;

use self::types::{
    ActiveBuff, BuffInstanceKey, ProjectileRecord, RuntimeArtifact, RuntimeItem, RuntimeUnit,
    TriggerSource,
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

    fn schedule_pending_autocast_for(&mut self, caster_instance_id: UnitInstanceId, now_ms: u64) {
        let fallback = self.recording_cause().unwrap_or_default();

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
    use crate::game::battle::core::movement::ActionState;
    use crate::game::battle::core::types::RuntimeUnit;
    use crate::game::data::{
        abnormality_data::AbnormalityDatabase, artifact_data::ArtifactDatabase,
        bonus_data::BonusDatabase, equipment_data::EquipmentDatabase,
        pve_data::PveEncounterDatabase, random_event_data::RandomEventDatabase,
        shop_data::ShopDatabase, skill_data::SkillDatabase,
    };
    use crate::game::enums::Side;
    use crate::game::stats::UnitStats;
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

        Arc::new(GameDataBase::new(
            Arc::new(AbnormalityDatabase::new(vec![])),
            Arc::new(ArtifactDatabase::new(vec![])),
            Arc::new(EquipmentDatabase::new(vec![])),
            Arc::new(ShopDatabase::new(vec![])),
            Arc::new(BonusDatabase::new(vec![])),
            Arc::new(RandomEventDatabase::new(vec![])),
            Arc::new(PveEncounterDatabase::new(vec![])),
            Arc::new(SkillDatabase::new(vec![])),
            event_pools,
        ))
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
            pending_autocast: None,
        }
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
}
