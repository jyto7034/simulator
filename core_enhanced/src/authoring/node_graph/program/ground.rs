//! A finite independent ground instance. Its lifetime starts at creation,
//! including when its first periodic event is delayed by one interval.
use crate::game::{
    ability::SkillHitTargetFilter,
    battle::{damage::DamageType, tile_range::TileRangePattern},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundProgram {
    pub selection_range: TileRangePattern,
    pub footprint: TileRangePattern,
    pub duration_ms: u32,
    pub interval_ms: u32,
    pub first: GroundFirstTick,
    pub hit_targets: SkillHitTargetFilter,
    pub include_caster: bool,
    pub effect: GroundEffect,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroundFirstTick {
    Immediate,
    AfterInterval,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GroundEffect {
    Damage {
        amount: u32,
        damage_type: DamageType,
    },
    Heal {
        amount: u32,
    },
}
impl GroundProgram {
    pub(crate) fn validate(&self) -> Result<(), String> {
        self.selection_range.validate()?;
        self.footprint.validate()?;
        if self.duration_ms == 0 || self.interval_ms == 0 {
            return Err("Ground lifetime and interval must be positive".into());
        }
        let first = match self.first {
            GroundFirstTick::Immediate => 0,
            GroundFirstTick::AfterInterval => self.interval_ms,
        };
        let ticks = if first < self.duration_ms {
            1 + (self.duration_ms - 1 - first) / self.interval_ms
        } else {
            0
        };
        if ticks > 10_000 {
            return Err("Ground program exceeds 10000 periodic events".into());
        }
        let amount = match self.effect {
            GroundEffect::Damage { amount, .. } | GroundEffect::Heal { amount } => amount,
        };
        if amount > i32::MAX as u32 {
            return Err("Ground amount exceeds native magnitude".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundStatusAreaProgram {
    pub selection_range: TileRangePattern,
    pub footprint: TileRangePattern,
    pub duration_ms: u32,
    pub hit_targets: SkillHitTargetFilter,
    pub include_caster: bool,
    pub status: crate::game::ability::ground::GroundStatusProgram,
}
impl GroundStatusAreaProgram {
    pub(crate) fn validate(&self) -> Result<(), String> {
        self.selection_range.validate()?;
        self.footprint.validate()?;
        if self.duration_ms == 0 {
            return Err("Ground lifetime must be positive".into());
        }
        self.status.validate()
    }
}
