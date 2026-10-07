
#[cfg(test)]
impl BattleExecutionState {
    pub(crate) fn audit_u03_exhaust_simulation_time(&mut self) {
        self.last_event_time_ms = u64::MAX;
    }
}
