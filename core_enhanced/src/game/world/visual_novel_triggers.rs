use uuid::Uuid;

use super::{visual_novel_flow::VisualNovelQueuePlacement, GameCore};
use crate::game::{
    behavior::GameError,
    combat_preview::{CombatMissionVariant, CombatNodeType},
    data::visual_novel_trigger_data::{
        VisualNovelTrigger, VisualNovelTriggerCheckpointPolicy, VisualNovelTriggerRepeat,
        VisualNovelTriggerSource,
    },
    map::{GameMode, MapNodeCategory, MapNodeId, MapNodeKindId},
    visual_novel::{
        VisualNovelSession, VisualNovelSource, VisualNovelTriggerConsumedKey,
        VisualNovelTriggerConsumedScope,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum VisualNovelTriggerEvent {
    RunStart {
        game_mode: GameMode,
    },
    MapNodeEntered {
        node_id: MapNodeId,
        category: MapNodeCategory,
        kind_id: MapNodeKindId,
    },
    BattleStarted {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
    },
    BattleTimeReached {
        battle_uuid: Uuid,
        previous_battle_time_ms: u64,
        current_battle_time_ms: u64,
    },
}

impl GameCore {
    pub(super) fn evaluate_visual_novel_triggers(
        &mut self,
        event: VisualNovelTriggerEvent,
    ) -> Result<usize, GameError> {
        let mut matches = self
            .game_data
            .visual_novel_trigger_data
            .triggers
            .iter()
            .filter(|trigger| self.trigger_matches_event(trigger, &event))
            .cloned()
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.id.as_str().cmp(right.id.as_str()))
        });

        let mut activated = 0usize;
        for trigger in matches {
            let Some(consumed_key) = self.consumed_key_for_trigger(&trigger, &event)? else {
                let session = self.visual_novel_session_for_trigger(&trigger, &event)?;
                self.activate_or_queue_visual_novel_session(
                    session,
                    VisualNovelQueuePlacement::Tail,
                );
                activated += 1;
                continue;
            };
            if self.is_visual_novel_trigger_consumed(trigger.checkpoint_policy, &consumed_key) {
                continue;
            }
            let session = self.visual_novel_session_for_trigger(&trigger, &event)?;
            self.activate_or_queue_visual_novel_session(session, VisualNovelQueuePlacement::Tail);
            self.mark_visual_novel_trigger_consumed(trigger.checkpoint_policy, consumed_key);
            activated += 1;
        }

        Ok(activated)
    }

    fn trigger_matches_event(
        &self,
        trigger: &VisualNovelTrigger,
        event: &VisualNovelTriggerEvent,
    ) -> bool {
        match (&trigger.source, event) {
            (
                VisualNovelTriggerSource::RunStart {
                    game_mode: trigger_mode,
                },
                VisualNovelTriggerEvent::RunStart { game_mode },
            ) => trigger_mode == game_mode,
            (
                VisualNovelTriggerSource::MapNodeEntered {
                    category: trigger_category,
                    kind_id: trigger_kind_id,
                },
                VisualNovelTriggerEvent::MapNodeEntered {
                    category, kind_id, ..
                },
            ) => trigger_category == category && trigger_kind_id == kind_id,
            (
                VisualNovelTriggerSource::BattleStarted {
                    encounter_id: trigger_encounter_id,
                    node_type: trigger_node_type,
                    mission_variant: trigger_mission_variant,
                },
                VisualNovelTriggerEvent::BattleStarted {
                    encounter_id,
                    node_type,
                    mission_variant,
                    ..
                },
            ) => {
                trigger_encounter_id == encounter_id
                    && trigger_node_type == node_type
                    && trigger_mission_variant == mission_variant
            }
            (
                VisualNovelTriggerSource::BattleTimeReached { battle_time_ms },
                VisualNovelTriggerEvent::BattleTimeReached {
                    previous_battle_time_ms,
                    current_battle_time_ms,
                    ..
                },
            ) => {
                previous_battle_time_ms < battle_time_ms
                    && *battle_time_ms <= *current_battle_time_ms
            }
            _ => false,
        }
    }

    fn consumed_key_for_trigger(
        &self,
        trigger: &VisualNovelTrigger,
        event: &VisualNovelTriggerEvent,
    ) -> Result<Option<VisualNovelTriggerConsumedKey>, GameError> {
        let scope = match trigger.repeat {
            VisualNovelTriggerRepeat::Always => return Ok(None),
            VisualNovelTriggerRepeat::OncePerRun => VisualNovelTriggerConsumedScope::Run,
            VisualNovelTriggerRepeat::OncePerBattle => {
                let battle_uuid = match event {
                    VisualNovelTriggerEvent::BattleStarted { battle_uuid, .. }
                    | VisualNovelTriggerEvent::BattleTimeReached { battle_uuid, .. } => {
                        *battle_uuid
                    }
                    _ => {
                        return Err(GameError::InvalidStaticData(format!(
                            "visual novel trigger '{}' uses OncePerBattle outside battle source",
                            trigger.id.as_str()
                        )))
                    }
                };
                VisualNovelTriggerConsumedScope::Battle { battle_uuid }
            }
            VisualNovelTriggerRepeat::OncePerNode => {
                let node_id = match event {
                    VisualNovelTriggerEvent::MapNodeEntered { node_id, .. } => *node_id,
                    VisualNovelTriggerEvent::BattleStarted { battle_uuid, .. }
                    | VisualNovelTriggerEvent::BattleTimeReached { battle_uuid, .. } => {
                        MapNodeId(*battle_uuid)
                    }
                    VisualNovelTriggerEvent::RunStart { .. } => {
                        return Err(GameError::InvalidStaticData(format!(
                            "visual novel trigger '{}' uses OncePerNode outside node source",
                            trigger.id.as_str()
                        )))
                    }
                };
                VisualNovelTriggerConsumedScope::Node { node_id }
            }
        };
        Ok(Some(VisualNovelTriggerConsumedKey {
            trigger_id: trigger.id.clone(),
            scope,
        }))
    }

    fn is_visual_novel_trigger_consumed(
        &self,
        checkpoint_policy: VisualNovelTriggerCheckpointPolicy,
        key: &VisualNovelTriggerConsumedKey,
    ) -> bool {
        match checkpoint_policy {
            VisualNovelTriggerCheckpointPolicy::PersistAcrossCheckpointLoad => self
                .state
                .visual_novel_triggers
                .persistent_consumed
                .contains(key),
            VisualNovelTriggerCheckpointPolicy::RewindWithRunCheckpoint => self
                .state
                .visual_novel_triggers
                .checkpoint_rewind_consumed
                .contains(key),
        }
    }

    fn mark_visual_novel_trigger_consumed(
        &mut self,
        checkpoint_policy: VisualNovelTriggerCheckpointPolicy,
        key: VisualNovelTriggerConsumedKey,
    ) {
        match checkpoint_policy {
            VisualNovelTriggerCheckpointPolicy::PersistAcrossCheckpointLoad => {
                self.state
                    .visual_novel_triggers
                    .persistent_consumed
                    .insert(key);
            }
            VisualNovelTriggerCheckpointPolicy::RewindWithRunCheckpoint => {
                self.state
                    .visual_novel_triggers
                    .checkpoint_rewind_consumed
                    .insert(key);
            }
        }
    }

    fn visual_novel_session_for_trigger(
        &mut self,
        trigger: &VisualNovelTrigger,
        event: &VisualNovelTriggerEvent,
    ) -> Result<VisualNovelSession, GameError> {
        let definition =
            self.resolve_visual_novel_session_definition(trigger.vn_id.clone(), || {
                GameError::InvalidStaticData(format!(
                    "visual novel trigger '{}' references missing visual novel '{}'",
                    trigger.id.as_str(),
                    trigger.vn_id.as_str()
                ))
            })?;
        Ok(self.assemble_visual_novel_session(
            definition,
            source_for_trigger_event(event),
            trigger.blocking,
        ))
    }
}

fn source_for_trigger_event(event: &VisualNovelTriggerEvent) -> VisualNovelSource {
    match event {
        VisualNovelTriggerEvent::RunStart { .. } => VisualNovelSource::HqBriefing,
        VisualNovelTriggerEvent::MapNodeEntered { .. } => VisualNovelSource::Map,
        VisualNovelTriggerEvent::BattleStarted { battle_uuid, .. }
        | VisualNovelTriggerEvent::BattleTimeReached { battle_uuid, .. } => {
            VisualNovelSource::Battle {
                battle_uuid: *battle_uuid,
            }
        }
    }
}
