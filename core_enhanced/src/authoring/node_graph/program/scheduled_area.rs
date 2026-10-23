use super::DelayedAreaProgram;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduledAreaProgram {
    pub opportunities: u32,
    pub interval_ms: u32,
    pub impact: DelayedAreaProgram,
}
impl ScheduledAreaProgram {
    pub(crate) fn duration_ms(&self) -> Result<u32, String> {
        self.opportunities
            .checked_sub(1)
            .and_then(|n| n.checked_mul(self.interval_ms))
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| "Scheduled duration overflow".into())
    }
    pub(crate) fn validate(&self) -> Result<(), String> {
        self.impact.validate()?;
        if self.opportunities == 0
            || self.opportunities > 10_000
            || self.interval_ms == 0
            || self.impact.prepare_ms != 0
            || self.impact.on_hit_heal_percent.is_some()
        {
            return Err(
                "Schedule requires 1..10000 opportunities, a positive interval and a direct impact"
                    .into(),
            );
        }
        self.duration_ms()?;
        Ok(())
    }
}
