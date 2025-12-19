use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::game::{
    battle::timeline::TimelineEvent,
    enums::Side,
    stats::{Effect, TriggerType},
};

use super::BattleCore;

impl BattleCore {
    pub(super) fn process_pending_deaths(&mut self, current_time_ms: u64) {
        let pending_death_info: HashMap<Uuid, (Option<Uuid>, Side)> = self
            .death_handler
            .pending_deaths
            .iter()
            .map(|d| (d.unit_id, (d.killer_id, d.owner)))
            .collect();

        let pending_unit_ids: Vec<Uuid> = self
            .death_handler
            .pending_deaths
            .iter()
            .map(|d| d.unit_id)
            .collect();
        let pending_unit_set: HashSet<Uuid> = pending_unit_ids.iter().copied().collect();

        let on_death_effects: HashMap<Uuid, Vec<Effect>> = pending_unit_ids
            .iter()
            .map(|&id| (id, self.collect_all_triggers(id, TriggerType::OnDeath)))
            .collect();

        let killer_ids: Vec<Uuid> = self
            .death_handler
            .pending_deaths
            .iter()
            .filter_map(|d| d.killer_id)
            .collect();
        let on_kill_effects: HashMap<Uuid, Vec<Effect>> = killer_ids
            .iter()
            .map(|&id| (id, self.collect_all_triggers(id, TriggerType::OnKill)))
            .collect();

        let all_unit_ids: Vec<Uuid> = self.units.keys().copied().collect();
        let on_ally_death_effects: HashMap<Uuid, Vec<Effect>> = all_unit_ids
            .iter()
            .map(|&id| (id, self.collect_all_triggers(id, TriggerType::OnAllyDeath)))
            .collect();

        let alive_unit_ids: HashSet<Uuid> = self
            .units
            .iter()
            .filter_map(|(id, u)| {
                if u.stats.current_health > 0 {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        let unit_owner: HashMap<Uuid, Side> =
            self.units.iter().map(|(id, u)| (*id, u.owner)).collect();

        let result = self.death_handler.process_all_deaths(
            |unit_id| on_death_effects.get(&unit_id).cloned().unwrap_or_default(),
            |unit_id| on_kill_effects.get(&unit_id).cloned().unwrap_or_default(),
            |unit_id| {
                on_ally_death_effects
                    .get(&unit_id)
                    .cloned()
                    .unwrap_or_default()
            },
            |dead_unit_id, dead_unit_side| {
                let mut allies: Vec<Uuid> = unit_owner
                    .iter()
                    .filter(|(id, owner)| {
                        **id != dead_unit_id
                            && **owner == dead_unit_side
                            && alive_unit_ids.contains(*id)
                            && !pending_unit_set.contains(*id)
                    })
                    .map(|(id, _)| *id)
                    .collect();
                allies.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
                allies
            },
        );

        for unit_id in &result.units_to_remove {
            if let Some(unit) = self.units.get(unit_id) {
                self.graveyard.insert(*unit_id, unit.to_snapshot());
            }

            self.buffs
                .retain(|key, _| key.target_instance_id != *unit_id);

            let (killer_id, owner) = pending_death_info
                .get(unit_id)
                .copied()
                .unwrap_or((None, Side::Opponent));
            self.record_timeline(
                current_time_ms,
                TimelineEvent::UnitDied {
                    unit_instance_id: *unit_id,
                    owner,
                    killer_instance_id: killer_id,
                },
            );

            self.units.remove(unit_id);
            self.runtime_field.remove(*unit_id);

            for unit in self.units.values_mut() {
                if unit.current_target == Some(*unit_id) {
                    unit.current_target = None;
                }
            }
        }

        if !result.commands.is_empty() {
            self.process_commands(result.commands, current_time_ms);
        }
    }
}
