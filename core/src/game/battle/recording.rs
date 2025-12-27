use crate::game::battle::{
    core::BattleCore,
    timeline::{TimelineCause, TimelineEntry, TimelineEvent, TimelineRootCause},
};

impl BattleCore {
    pub(super) fn record_timeline(&mut self, time_ms: u64, event: TimelineEvent) -> u64 {
        let seq = self.timeline_seq;
        self.timeline.entries.push(TimelineEntry {
            time_ms,
            seq,
            cause: self.recording_cause().unwrap_or_default(),
            event,
        });
        self.timeline_seq += 1;
        seq
    }

    pub(super) fn recording_cause(&self) -> Option<TimelineCause> {
        self.recording_cause_stack.last().copied()
    }

    pub(super) fn with_recording_context<F, R>(&mut self, cause: TimelineCause, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let stack_len = self.recording_cause_stack.len();
        self.recording_cause_stack.push(cause);
        let result = f(self);
        self.recording_cause_stack.truncate(stack_len);
        result
    }

    pub(super) fn with_recording_root<F, R>(&mut self, kind: TimelineRootCause, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.with_recording_context(TimelineCause::Root { kind }, f)
    }

    pub(super) fn with_recording_cause<F, R>(&mut self, cause: u64, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.recording_cause_stack
            .push(TimelineCause::Parent { seq: cause });
        let result = f(self);
        self.recording_cause_stack.pop();
        result
    }
}
