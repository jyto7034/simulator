use crate::{
    ecs::resources::Position,
    game::battle::timeline::{SkillCastTarget, Timeline, TimelineEvent},
};

#[derive(Debug, Clone, Copy)]
pub(super) struct BattlefieldSize {
    pub(super) width: u8,
    pub(super) height: u8,
}

pub(super) fn extract_battlefield_size(timeline: &Timeline) -> Option<BattlefieldSize> {
    for entry in &timeline.entries {
        if let TimelineEvent::BattleStart { width, height } = entry.event {
            return Some(BattlefieldSize { width, height });
        }
    }
    None
}

pub(super) fn position_in_bounds(pos: Position, width: u8, height: u8) -> bool {
    pos.x >= 0 && pos.y >= 0 && pos.x < width as i32 && pos.y < height as i32
}

pub(super) fn positions_from_event(event: &TimelineEvent) -> Vec<(Position, &'static str)> {
    match event {
        TimelineEvent::UnitSpawned { position, .. } => vec![(*position, "UnitSpawned.position")],
        TimelineEvent::UnitMoved { from, to, .. } => {
            vec![(*from, "UnitMoved.from"), (*to, "UnitMoved.to")]
        }
        TimelineEvent::MovementSegmentStarted { from, to, .. } => {
            vec![
                (*from, "MovementSegmentStarted.from"),
                (*to, "MovementSegmentStarted.to"),
            ]
        }
        TimelineEvent::MovementStopped { position, .. } => {
            vec![(*position, "MovementStopped.position")]
        }
        TimelineEvent::AutoCastStart {
            target: Some(SkillCastTarget::Tile { position }),
            ..
        } => vec![(*position, "AutoCastStart.target(tile)")],
        _ => Vec::new(),
    }
}
