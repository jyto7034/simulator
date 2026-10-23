use crate::game::{
    ability::marks::{MarkRatio, PopulationPowerDef},
    behavior::GameError,
};

/// Exact units chosen from the authored rates. Time queries do not mutate the
/// clock; membership changes settle the previous population before replacing it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PowerClock {
    pub units: u128,
    pub scale: u64,
    pub sampled_at: u64,
    pub empty_since: Option<u64>,
}
crate::authoring::clone_shape::struct_shape!(PowerClock {
    units,
    scale,
    sampled_at,
    empty_since
});
impl PowerClock {
    pub fn new(def: &PopulationPowerDef, now: u64) -> Result<Self, GameError> {
        Ok(Self {
            units: 0,
            scale: def.scale().map_err(|_| GameError::InvalidAction)?,
            sampled_at: now,
            empty_since: Some(now),
        })
    }
    pub fn value(
        &self,
        def: &PopulationPowerDef,
        hosts: usize,
        now: u64,
    ) -> Result<u128, GameError> {
        let elapsed = now
            .checked_sub(self.sampled_at)
            .ok_or(GameError::InvalidAction)?;
        let scaled = |ratio: MarkRatio| {
            u128::from(ratio.numerator) * u128::from(self.scale / ratio.denominator)
        };
        let cap = scaled(def.cap);
        let current = self.units.min(cap);
        if hosts > 0 {
            let per_ms = scaled(def.growth_per_second) / 1000;
            let remaining = cap - current;
            let rate = per_ms.checked_mul(hosts as u128);
            let growth = rate
                .and_then(|r| r.checked_mul(u128::from(elapsed)))
                .unwrap_or(u128::MAX);
            return Ok(current + growth.min(remaining));
        }
        if def.prevent_decay {
            return Ok(current);
        }
        let since = self.empty_since.ok_or(GameError::InvalidAction)?;
        let grace_end = since
            .checked_add(u64::from(def.grace_ms))
            .unwrap_or(u64::MAX);
        let decay_time = now.saturating_sub(self.sampled_at.max(grace_end));
        let loss = (scaled(def.decay_per_second) / 1000)
            .checked_mul(u128::from(decay_time))
            .unwrap_or(u128::MAX);
        Ok(current.saturating_sub(loss))
    }
    pub fn settle(
        &mut self,
        def: &PopulationPowerDef,
        hosts: usize,
        now: u64,
    ) -> Result<(), GameError> {
        self.units = self.value(def, hosts, now)?;
        self.sampled_at = now;
        Ok(())
    }
    pub fn membership_changed(&mut self, old: usize, new: usize, now: u64) {
        if new > 0 {
            self.empty_since = None;
        } else if old > 0 {
            self.empty_since = Some(now);
        }
    }
    pub fn amount(&self, units: u128, coefficient: MarkRatio) -> Result<u32, GameError> {
        // Reduce cross-factors before multiplication. Reject a representational
        // overflow instead of silently rounding or saturating a damage request.
        fn gcd(mut a: u128, mut b: u128) -> u128 {
            while b != 0 {
                (a, b) = (b, a % b);
            }
            a
        }
        let mut n = units;
        let mut d = u128::from(self.scale);
        let mut cn = u128::from(coefficient.numerator);
        let mut cd = u128::from(coefficient.denominator);
        let g = gcd(n, cd);
        n /= g;
        cd /= g;
        let g = gcd(cn, d);
        cn /= g;
        d /= g;
        let numerator = n.checked_mul(cn).ok_or(GameError::InvalidAction)?;
        let denominator = d.checked_mul(cd).ok_or(GameError::InvalidAction)?;
        u32::try_from(numerator.div_ceil(denominator))
            .ok()
            .filter(|v| *v <= i32::MAX as u32)
            .ok_or(GameError::InvalidAction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ratio(n: u64, d: u64) -> MarkRatio {
        MarkRatio {
            numerator: n,
            denominator: d,
        }
    }
    fn definition() -> PopulationPowerDef {
        PopulationPowerDef {
            growth_per_second: ratio(3, 2),
            cap: ratio(100, 1),
            decay_per_second: ratio(1, 4),
            grace_ms: 200,
            prevent_decay: false,
        }
    }
    #[test]
    fn mark_power_membership_integrates_exactly_and_empty_grace_is_not_reset_by_queries() {
        let d = definition();
        let mut c = PowerClock::new(&d, 0).unwrap();
        c.membership_changed(0, 2, 0);
        c.settle(&d, 2, 100).unwrap();
        assert_eq!(c.amount(c.units, ratio(10, 1)).unwrap(), 3);
        c.membership_changed(2, 1, 100);
        c.settle(&d, 1, 300).unwrap();
        assert_eq!(c.amount(c.units, ratio(10, 1)).unwrap(), 6);
        c.membership_changed(1, 0, 300);
        assert_eq!(c.value(&d, 0, 500).unwrap(), c.units);
        c.settle(&d, 0, 900).unwrap();
        assert_eq!(c.amount(c.units, ratio(10, 1)).unwrap(), 5);
        c.membership_changed(0, 1, 900);
        c.settle(&d, 1, 1100).unwrap();
        assert_eq!(c.amount(c.units, ratio(10, 1)).unwrap(), 8);
    }
    #[test]
    fn mark_power_cadence_partition_does_not_change_growth_or_decay() {
        let d = definition();
        let mut a = PowerClock::new(&d, 0).unwrap();
        let mut b = a.clone();
        a.membership_changed(0, 3, 0);
        b.membership_changed(0, 3, 0);
        a.settle(&d, 3, 1000).unwrap();
        for t in [1, 100, 333, 999, 1000] {
            b.settle(&d, 3, t).unwrap();
        }
        assert_eq!(a, b);
        a.membership_changed(3, 0, 1000);
        b.membership_changed(3, 0, 1000);
        a.settle(&d, 0, 2000).unwrap();
        for t in [1100, 1200, 1201, 1800, 2000] {
            b.settle(&d, 0, t).unwrap();
        }
        assert_eq!(a, b);
    }
    #[test]
    fn mark_power_cap_and_preservation_do_not_create_growth_without_hosts() {
        let mut d = definition();
        d.prevent_decay = true;
        let mut c = PowerClock::new(&d, 0).unwrap();
        c.membership_changed(0, usize::MAX, 0);
        c.settle(&d, usize::MAX, u64::MAX - 1000).unwrap();
        assert_eq!(c.amount(c.units, ratio(1, 1)).unwrap(), 100);
        c.membership_changed(usize::MAX, 0, u64::MAX - 1000);
        c.settle(&d, 0, u64::MAX).unwrap();
        assert_eq!(c.amount(c.units, ratio(1, 1)).unwrap(), 100);
    }
}
