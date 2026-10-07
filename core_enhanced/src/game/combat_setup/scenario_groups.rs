//! Small helpers for registering scenario spawn groups with their matching spawn events.
//!
//! Keeping this in one place prevents battle-start and timed-spawn event wiring
//! from diverging between player, defense-object, and enemy groups.

use crate::game::battle::scenario::{
    ScenarioAction, ScenarioEvent, ScenarioEventId, ScenarioGroupId, ScenarioSpawnGroup,
    ScenarioTrigger, ScenarioWaveActivation, ScenarioWavePrimaryTrigger, TacticalPointId,
};
use crate::game::data::pve_data::{PveWaveActivationData, PveWavePrimaryTriggerData};

pub(crate) fn push_start_spawn_group(
    groups: &mut Vec<ScenarioSpawnGroup>,
    events: &mut Vec<ScenarioEvent>,
    group: ScenarioSpawnGroup,
) {
    push_spawn_group_with_trigger(groups, events, group, ScenarioTrigger::AtBattleStart);
}

pub(crate) fn push_wave_spawn_group(
    groups: &mut Vec<ScenarioSpawnGroup>,
    events: &mut Vec<ScenarioEvent>,
    group: ScenarioSpawnGroup,
    activation: PveWaveActivationData,
) {
    let primary = match activation.primary {
        PveWavePrimaryTriggerData::AtTimeMs { time_ms } => {
            ScenarioWavePrimaryTrigger::AtTimeMs { time_ms }
        }
        PveWavePrimaryTriggerData::GroupCleared { wave_id } => {
            ScenarioWavePrimaryTrigger::GroupCleared {
                group_id: ScenarioGroupId::new(format!("enemy_{wave_id}")),
            }
        }
        PveWavePrimaryTriggerData::ActorHealthAtOrBelowPercent {
            actor_slot_id,
            percent,
        } => ScenarioWavePrimaryTrigger::ActorHealthAtOrBelowPercent {
            actor_slot_id,
            percent,
        },
        PveWavePrimaryTriggerData::ActorDefeated { actor_slot_id } => {
            ScenarioWavePrimaryTrigger::ActorDefeated { actor_slot_id }
        }
        PveWavePrimaryTriggerData::ActorReachedPoint {
            actor_slot_id,
            point_id,
            radius_cells,
        } => ScenarioWavePrimaryTrigger::ActorReachedPoint {
            actor_slot_id,
            point_id: TacticalPointId::new(point_id),
            radius_cells,
        },
    };
    push_spawn_group_with_trigger(
        groups,
        events,
        group,
        ScenarioTrigger::Wave(ScenarioWaveActivation {
            trigger_id: activation.trigger_id,
            priority: activation.priority,
            primary,
            deadline_ms: activation.deadline_ms,
        }),
    );
}

fn push_spawn_group_with_trigger(
    groups: &mut Vec<ScenarioSpawnGroup>,
    events: &mut Vec<ScenarioEvent>,
    group: ScenarioSpawnGroup,
    trigger: ScenarioTrigger,
) {
    let group_id = group.id.clone();
    events.push(ScenarioEvent {
        id: ScenarioEventId::new(format!("spawn_{}", group_id.0)),
        trigger,
        action: ScenarioAction::SpawnGroup {
            group_id: group_id.clone(),
        },
    });
    groups.push(group);
}
