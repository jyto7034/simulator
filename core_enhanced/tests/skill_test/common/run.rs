use game_core::{
    game::resources::Position,
    game::{
        battle::{
            core::BattleCore,
            enums::BattleEvent,
            event_log::{BattleEventCause, BattleEventLog, BattleEventRootCause},
            types::BattleResult,
        },
        data::abnormality_data::AbnormalityMetadata,
        enums::Side,
    },
};

use crate::common;

use super::{
    find_first_ability_cast, find_unit_instance_at, step_entries_for_cast, BoardScenario,
    RuntimeStartPatch,
};

pub struct ScenarioRunResult {
    pub abnormality: AbnormalityMetadata,
    pub board: BoardScenario,
    pub battle_result: BattleResult,
}

impl ScenarioRunResult {
    pub fn event_log(&self) -> &BattleEventLog {
        &self.battle_result.event_log
    }

    pub fn caster_instance_id(&self) -> game_core::game::battle::ids::UnitInstanceId {
        self.unit_instance_at(self.board.caster.position, Some(Side::Player))
            .expect("caster unit should exist at board caster position")
    }

    pub fn first_cast_of(
        &self,
        skill_id: &str,
    ) -> &game_core::game::battle::event_log::BattleEventLogEntry {
        find_first_ability_cast(self.event_log(), self.caster_instance_id(), skill_id)
            .unwrap_or_else(|| {
                panic!(
                    "{} ({}) never cast skill {} on configured board",
                    self.abnormality.name, self.abnormality.id, skill_id
                )
            })
    }

    pub fn first_cast_steps(
        &self,
        skill_id: &str,
    ) -> Vec<&game_core::game::battle::event_log::BattleEventLogEntry> {
        let cast = self.first_cast_of(skill_id);
        step_entries_for_cast(self.event_log(), cast.seq)
    }

    pub fn winner(&self) -> game_core::game::battle::types::BattleWinner {
        self.battle_result.winner
    }

    pub fn unit_instance_at(
        &self,
        position: Position,
        owner: Option<Side>,
    ) -> Option<game_core::game::battle::ids::UnitInstanceId> {
        find_unit_instance_at(self.event_log(), position, owner)
    }
}

fn apply_runtime_start_patch(
    battle: &mut BattleCore,
    position: Position,
    patch: &RuntimeStartPatch,
) -> Result<(), String> {
    let unit_id = runtime_unit_at(battle, position)
        .ok_or_else(|| format!("no unit placed at {:?}", position))?;
    let current_target_id = if let Some(target_position) = patch.current_target_position {
        Some(runtime_unit_at(battle, target_position).ok_or_else(|| {
            format!(
                "no unit placed at current_target position {:?}",
                target_position
            )
        })?)
    } else {
        None
    };

    let unit = battle
        .units
        .get_mut(&unit_id)
        .ok_or_else(|| format!("runtime unit missing for {:?}", position))?;

    if let Some(current_health) = patch.current_health {
        unit.stats.current_health = current_health.min(unit.stats.max_health);
    }

    if let Some(resonance_current) = patch.resonance_current {
        unit.resonance_current = resonance_current.min(unit.resonance_max.max(1));
    }

    if let Some(pending_cast) = patch.pending_cast {
        unit.pending_cast = pending_cast;
    }

    if let Some(target_id) = current_target_id {
        unit.current_target = Some(target_id);
    }

    if let Some(until_ms) = patch.movement_lock_until_ms {
        unit.action_locks.lock_movement_until(until_ms);
    }

    if let Some(until_ms) = patch.basic_attack_lock_until_ms {
        unit.action_locks.lock_basic_attack_until(until_ms);
    }

    if let Some(until_ms) = patch.resonance_gain_lock_until_ms {
        unit.action_locks.lock_resonance_gain_until(until_ms);
    }

    if patch.pending_cast == Some(true) {
        battle.enqueue_event(BattleEvent::AutoCastStart {
            time_ms: 0,
            caster_instance_id: unit_id,
            cause: BattleEventCause::Root {
                kind: BattleEventRootCause::Period,
            },
        });
    }

    Ok(())
}

fn runtime_unit_at(
    battle: &BattleCore,
    position: Position,
) -> Option<game_core::game::battle::ids::UnitInstanceId> {
    let mut unit_ids = battle
        .units
        .values()
        .filter(|unit| unit.is_active() && unit.body.projected_tile() == position)
        .map(|unit| unit.instance_id)
        .collect::<Vec<_>>();
    unit_ids.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    unit_ids.into_iter().next()
}

fn sanitize_event_log_export_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_was_sep = false;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_was_sep = false;
        } else if !prev_was_sep {
            out.push('_');
            prev_was_sep = true;
        }
    }

    out.trim_matches('_').to_string()
}

fn event_log_export_stem(
    abnormality: &AbnormalityMetadata,
    game_data: &game_core::game::data::GameDataBase,
) -> String {
    let mut parts = vec![
        abnormality.id.clone(),
        sanitize_event_log_export_name(&abnormality.name),
    ];

    if let Some(skill_id) = abnormality.skill_id.as_deref() {
        if let Some(skill) = game_data.skill_data.get_by_id(skill_id) {
            let skill_name = sanitize_event_log_export_name(&skill.name);
            if !skill_name.is_empty() && parts.last().is_none_or(|last| last != &skill_name) {
                parts.push(skill_name);
            }
        }
    }

    parts.join("_")
}

pub fn run_abnormality_scenario(abnormality_id: &str, board: BoardScenario) -> ScenarioRunResult {
    let base_game_data = common::load_game_data_from_ron();
    let resolved =
        super::scenario::resolve_abnormality_scenario(base_game_data, abnormality_id, &board);

    let mut battle = BattleCore::new_from_scenario(
        resolved.battle_scenario.clone(),
        resolved.game_data.clone(),
        10_000,
    );

    let runtime_patches = resolved.runtime_patches.clone();
    let battle_result = battle
        .run_battle_with_post_spawn_setup(|core| {
            for (position, patch) in &runtime_patches {
                apply_runtime_start_patch(core, *position, patch)
                    .unwrap_or_else(|message| panic!("{message}"));
            }
        })
        .unwrap_or_else(|error| {
            panic!(
                "{} ({}) failed to run battle: {:?}",
                resolved.abnormality.name, resolved.abnormality.id, error
            )
        });

    common::write_debug_event_log_export(
        &format!(
            "skill_test/{}",
            event_log_export_stem(&resolved.abnormality, resolved.game_data.as_ref())
        ),
        &battle_result.event_log,
    );

    ScenarioRunResult {
        abnormality: resolved.abnormality,
        board,
        battle_result,
    }
}
