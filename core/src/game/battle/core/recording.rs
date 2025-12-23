use super::BattleCore;

use crate::game::battle::timeline::{TimelineEntry, TimelineEvent};

impl BattleCore {
    pub(super) fn record_timeline(&mut self, time_ms: u64, event: TimelineEvent) -> u64 {
        let seq = self.timeline_seq;
        self.timeline.entries.push(TimelineEntry {
            time_ms,
            seq,
            cause_seq: self.recording_cause_seq(),
            event,
        });
        self.timeline_seq += 1;
        seq
    }

    pub(super) fn recording_cause_seq(&self) -> Option<u64> {
        self.recording_cause_stack.last().copied()
    }

    pub(super) fn with_recording_parent<F, R>(&mut self, parent: Option<u64>, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let stack_len = self.recording_cause_stack.len();
        if let Some(parent) = parent {
            self.recording_cause_stack.push(parent);
        }
        let result = f(self);
        self.recording_cause_stack.truncate(stack_len);
        result
    }

    pub(super) fn with_recording_cause<F, R>(&mut self, cause: u64, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.recording_cause_stack.push(cause);
        let result = f(self);
        self.recording_cause_stack.pop();
        result
    }
}
