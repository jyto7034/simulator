use bevy_ecs::world::World;
use game_core::{
    ecs::resources::Position,
    game::{
        battle::{core::BattleCore, timeline::Timeline, types::BattleResult},
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
    pub fn timeline(&self) -> &Timeline {
        &self.battle_result.timeline
    }

    pub fn caster_instance_id(&self) -> game_core::game::battle::ids::UnitInstanceId {
        self.unit_instance_at(self.board.caster.position, Some(Side::Player))
            .expect("caster unit should exist at board caster position")
    }

    pub fn first_cast_of(
        &self,
        skill_id: &str,
    ) -> &game_core::game::battle::timeline::TimelineEntry {
        find_first_ability_cast(self.timeline(), self.caster_instance_id(), skill_id)
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
    ) -> Vec<&game_core::game::battle::timeline::TimelineEntry> {
        let cast = self.first_cast_of(skill_id);
        step_entries_for_cast(self.timeline(), cast.seq)
    }

    pub fn winner(&self) -> game_core::game::battle::types::BattleWinner {
        self.battle_result.winner
    }

    pub fn unit_instance_at(
        &self,
        position: Position,
        owner: Option<Side>,
    ) -> Option<game_core::game::battle::ids::UnitInstanceId> {
        find_unit_instance_at(self.timeline(), position, owner)
    }
}

fn apply_runtime_start_patch(
    battle: &mut BattleCore,
    position: Position,
    patch: &RuntimeStartPatch,
) -> Result<(), String> {
    let Some(current_health) = patch.current_health else {
        return Ok(());
    };

    let unit_id = battle
        .battlefield
        .occupant(position)
        .map_err(|error| format!("failed to inspect position {:?}: {:?}", position, error))?
        .ok_or_else(|| format!("no unit placed at {:?}", position))?;

    let unit = battle
        .units
        .get_mut(&unit_id)
        .ok_or_else(|| format!("runtime unit missing for {:?}", position))?;

    unit.stats.current_health = current_health.min(unit.stats.max_health);
    Ok(())
}

fn sanitize_timeline_export_name(name: &str) -> String {
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

fn timeline_export_stem(
    abnormality: &AbnormalityMetadata,
    game_data: &game_core::game::data::GameDataBase,
) -> String {
    let mut parts = vec![
        abnormality.id.clone(),
        sanitize_timeline_export_name(&abnormality.name),
    ];

    if let Some(skill_id) = abnormality.skill_id.as_deref() {
        if let Some(skill) = game_data.skill_data.get_by_id(skill_id) {
            let skill_name = sanitize_timeline_export_name(&skill.name);
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

    let mut battle = BattleCore::new(
        &resolved.player_deck,
        &resolved.opponent_deck,
        resolved.game_data.clone(),
        common::BOARD_SIZE,
        10_000,
    );

    let runtime_patches = resolved.runtime_patches.clone();
    let mut world = World::new();
    let battle_result = battle
        .run_battle_with_setup(&mut world, |core| {
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

    common::write_timeline_export(
        &format!(
            "skill_test/{}",
            timeline_export_stem(&resolved.abnormality, resolved.game_data.as_ref())
        ),
        &battle_result.timeline,
    );

    ScenarioRunResult {
        abnormality: resolved.abnormality,
        board,
        battle_result,
    }
}
