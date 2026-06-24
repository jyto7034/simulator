use crate::{
    game::battle::event_log::{BattleEventLog, BattleLogEvent, SkillCastTarget},
    game::resources::Position,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct BattlefieldSize {
    pub(super) width: u8,
    pub(super) height: u8,
}

pub(super) fn extract_battlefield_size(event_log: &BattleEventLog) -> Option<BattlefieldSize> {
    for entry in &event_log.entries {
        if let BattleLogEvent::BattleStart { width, height } = entry.event {
            return Some(BattlefieldSize { width, height });
        }
    }
    None
}

pub(super) fn position_in_bounds(pos: Position, width: u8, height: u8) -> bool {
    pos.x >= 0 && pos.y >= 0 && pos.x < width as i32 && pos.y < height as i32
}

pub(super) fn positions_from_event(event: &BattleLogEvent) -> Vec<(Position, &'static str)> {
    match event {
        BattleLogEvent::AutoCastStart {
            target: Some(SkillCastTarget::Tile { position }),
            ..
        } => vec![(*position, "AutoCastStart.target(tile)")],
        _ => Vec::new(),
    }
}
