use crate::game::battle::scenario::{
    ScenarioAction, ScenarioEvent, ScenarioEventId, ScenarioSpawnGroup, ScenarioTrigger,
};

pub(crate) fn push_start_spawn_group(
    groups: &mut Vec<ScenarioSpawnGroup>,
    events: &mut Vec<ScenarioEvent>,
    group: ScenarioSpawnGroup,
) {
    push_spawn_group_with_trigger(groups, events, group, ScenarioTrigger::AtBattleStart);
}

pub(crate) fn push_timed_spawn_group(
    groups: &mut Vec<ScenarioSpawnGroup>,
    events: &mut Vec<ScenarioEvent>,
    group: ScenarioSpawnGroup,
    time_ms: u32,
) {
    let trigger = if time_ms == 0 {
        ScenarioTrigger::AtBattleStart
    } else {
        ScenarioTrigger::AtTimeMs(u64::from(time_ms))
    };
    push_spawn_group_with_trigger(groups, events, group, trigger);
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
        once: true,
    });
    groups.push(group);
}
