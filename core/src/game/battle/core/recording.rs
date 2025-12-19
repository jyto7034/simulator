use super::BattleCore;

use crate::game::battle::timeline::{TimelineEntry, TimelineEvent};

impl BattleCore {
    pub(super) fn record_timeline(&mut self, time_ms: u64, event: TimelineEvent) {
        self.timeline.entries.push(TimelineEntry {
            time_ms,
            seq: self.timeline_seq,
            event,
        });
        self.timeline_seq += 1;
    }
}
