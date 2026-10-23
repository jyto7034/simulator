//! Integer admission rules shared by portable graph lowering and native casts.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ResonanceRequirement {
    Full,
    Minimum(u32),
    Payable,
    None,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ResonanceCost {
    All,
    Fixed(u32),
    Zero,
    MaximumPercent { percent: u8, minimum: u32 },
    CurrentPercent { percent: u8, minimum: u32 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResonanceAdmission {
    pub capacity: u32,
    pub requirement: ResonanceRequirement,
    pub cost: ResonanceCost,
}
impl ResonanceAdmission {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.capacity == 0 {
            return Err("resonance capacity must be positive");
        }
        if matches!(self.requirement, ResonanceRequirement::Minimum(n) if n > self.capacity) {
            return Err("minimum requirement exceeds capacity");
        }
        match self.cost {
            ResonanceCost::Fixed(n) if n > self.capacity => {
                return Err("fixed cost exceeds capacity")
            }
            ResonanceCost::MaximumPercent { percent, minimum }
            | ResonanceCost::CurrentPercent { percent, minimum }
                if percent > 100 || minimum > self.capacity =>
            {
                return Err("invalid percentage cost")
            }
            _ => (),
        }
        Ok(())
    }
    /// Recomputed immediately before mutation; no floating point or rounded snapshots.
    pub fn payment(&self, current: u32, maximum: u32) -> Option<u32> {
        self.validate().ok()?;
        if maximum != self.capacity || current > maximum {
            return None;
        }
        let required = match self.requirement {
            ResonanceRequirement::Full => maximum,
            ResonanceRequirement::Minimum(n) => n,
            ResonanceRequirement::Payable | ResonanceRequirement::None => 0,
        };
        if current < required {
            return None;
        }
        let proportional = |base: u32, percent: u8, minimum: u32| {
            ((u64::from(base) * u64::from(percent)).div_ceil(100) as u32).max(minimum)
        };
        let cost = match self.cost {
            ResonanceCost::All => current,
            ResonanceCost::Fixed(n) => n,
            ResonanceCost::Zero => 0,
            ResonanceCost::MaximumPercent { percent, minimum } => {
                proportional(maximum, percent, minimum)
            }
            ResonanceCost::CurrentPercent { percent, minimum } => {
                proportional(current, percent, minimum)
            }
        };
        (current >= cost).then_some(cost)
    }
}

crate::authoring::clone_shape::copy_shape!(ResonanceAdmission);
