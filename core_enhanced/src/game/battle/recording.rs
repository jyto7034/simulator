use crate::game::battle::{
    core::BattleCore,
    event_log::{BattleEventCause, BattleEventLogEntry, BattleEventRootCause, BattleLogEvent},
};

impl BattleCore {
    /// Append a new entry to the battle event log.
    ///
    /// The method name still uses `event_log` because this is the battle-local
    /// chronological event log. The Unity transport exposes these entries as
    /// `battle_update.events_delta`, not as current-state data.
    #[inline]
    pub(super) fn record_event_log(&mut self, time_ms: u64, event: BattleLogEvent) -> u64 {
        if self.event_log.sequence_exhausted() {
            return u64::MAX;
        }
        let seq = self.event_log_seq;
        let entry = BattleEventLogEntry {
            skill_effect_source: self.recording_skill_effect_source.clone(),
            time_ms,
            seq,
            cause: self.recording_cause().unwrap_or_default(),
            source_command_id: self.recording_source_command_id().map(str::to_owned),
            event,
        };
        if self.result_stats_accumulator.should_observe(&entry.event) {
            self.result_stats_accumulator.observe(&entry);
        }
        self.event_log.append(entry);
        if let Some(next_seq) = self.event_log_seq.checked_add(1) {
            self.event_log_seq = next_seq;
        } else {
            self.event_log.close_for_sequence_exhaustion();
        }
        seq
    }

    pub(super) fn recording_cause(&self) -> Option<BattleEventCause> {
        self.recording_cause_stack.last().copied()
    }

    pub(super) fn recording_source_command_id(&self) -> Option<&str> {
        self.recording_source_command_stack
            .last()
            .map(String::as_str)
    }

    pub(super) fn with_recording_source_command_id<F, R>(
        &mut self,
        source_command_id: &str,
        f: F,
    ) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let stack_len = self.recording_source_command_stack.len();
        self.recording_source_command_stack
            .push(source_command_id.to_owned());
        let result = f(self);
        self.recording_source_command_stack.truncate(stack_len);
        result
    }

    pub(super) fn with_recording_context<F, R>(&mut self, cause: BattleEventCause, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let stack_len = self.recording_cause_stack.len();
        self.recording_cause_stack.push(cause);
        let result = f(self);
        self.recording_cause_stack.truncate(stack_len);
        result
    }

    pub(super) fn with_recording_root<F, R>(&mut self, kind: BattleEventRootCause, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.with_recording_context(BattleEventCause::Root { kind }, f)
    }

    pub(super) fn with_recording_cause<F, R>(&mut self, cause: u64, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.recording_cause_stack
            .push(BattleEventCause::Parent { seq: cause });
        let result = f(self);
        self.recording_cause_stack.pop();
        result
    }
}
