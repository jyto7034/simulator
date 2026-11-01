//! The finite aura owns the timer and the ordinary HP-cost path owns payment.
use super::{overflow, BattleCore, BattleEvent, GameError};

impl BattleCore {
    pub(super) fn schedule_channel_upkeep(
        &mut self,
        activation: u64,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(record) = self.sustained.records.get_mut(&activation) else {
            return Ok(());
        };
        let Some(policy) = record.aura.as_ref().and_then(|a| a.health_upkeep) else {
            return Ok(());
        };
        let next = now
            .checked_add(u64::from(policy.interval_ms))
            .ok_or_else(|| overflow("channel_upkeep_deadline"))?;
        record.next_upkeep_at = (next < record.expires_at).then_some(next);
        if let Some(time_ms) = record.next_upkeep_at {
            self.event_queue.push(BattleEvent::ChannelUpkeep {
                time_ms,
                activation,
            });
        }
        Ok(())
    }

    pub(in crate::game::battle::core) fn channel_upkeep(
        &mut self,
        activation: u64,
        now: u64,
    ) -> Result<(), GameError> {
        // Current target/source validity and expiry precede any payment.
        self.reconcile_sustained(now)?;
        let Some(record) = self
            .sustained
            .records
            .get(&activation)
            .filter(|r| r.next_upkeep_at == Some(now))
        else {
            return Ok(());
        };
        let Some(policy) = record.aura.as_ref().and_then(|a| a.health_upkeep) else {
            return Ok(());
        };
        let caster = record.caster;
        let unit = self.units.get(&caster).ok_or(GameError::InvalidAction)?;
        let amount = u32::try_from(
            (u64::from(unit.stats.max_health) * u64::from(policy.max_health_percent)).div_ceil(100),
        )
        .map_err(|_| overflow("channel_upkeep_amount"))?
        .min(unit.stats.current_health);
        let source = crate::authoring::clone_shape::controlled_clone(&record.source)?;
        self.sustained
            .records
            .get_mut(&activation)
            .unwrap()
            .next_upkeep_at = None;
        let prior = std::mem::replace(&mut self.runtime_fact_source, source);
        let result = self.pay_health_cost(caster, amount, true, now);
        self.runtime_fact_source = prior;
        result?;
        self.reconcile_sustained(now)?;
        self.schedule_channel_upkeep(activation, now)
    }
}
