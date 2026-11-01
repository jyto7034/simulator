//! Released tile explosions. Stack capture is an admission transaction; hit
//! recipients are queried independently at each subsequent explosion round.
use super::marks::{ExplosionDedup, MarkChange, MarkModifier, MarkRatio};
use crate::game::battle::{damage::DamageType, tile_range::TileRangePattern};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrbConversion {
    pub relic_definition: String,
    pub threshold: u64,
    pub base_power: u32,
    pub power_per_stack: MarkRatio,
    pub range: TileRangePattern,
    pub presentation_ms: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TileBurstSource {
    WalkablePath,
    WeaponStacks {
        weapon: String,
        key: String,
        divisor: u32,
        rank_bonus_percent: MarkRatio,
        interval_ms: u32,
        conversion: Option<OrbConversion>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TileBurstDef {
    pub source: TileBurstSource,
    pub power: u32,
    pub delay_ms: u32,
    pub range: TileRangePattern,
    pub damage_type: DamageType,
    pub bonus_percent: MarkRatio,
    pub dedup: ExplosionDedup,
    pub modifiers: Vec<MarkModifier>,
}
impl TileBurstDef {
    pub fn validate(&self) -> Result<(), String> {
        self.range.validate()?;
        self.bonus_percent.validate(true)?;
        if self.power == 0 || self.power > i32::MAX as u32 {
            return Err("Tile burst requires positive native damage power".into());
        }
        if let TileBurstSource::WeaponStacks {
            weapon,
            key,
            divisor,
            rank_bonus_percent,
            interval_ms,
            conversion,
        } = &self.source
        {
            if !super::weapon_marks::valid_id(weapon)
                || !super::weapon_marks::valid_id(key)
                || *divisor == 0
                || *interval_ms == 0
            {
                return Err("Invalid weapon stack burst source or cadence".into());
            }
            rank_bonus_percent.validate(true)?;
            if let Some(c) = conversion {
                c.range.validate()?;
                c.power_per_stack.validate(true)?;
                if !super::weapon_marks::valid_id(&c.relic_definition)
                    || c.threshold == 0
                    || c.base_power > i32::MAX as u32
                    || c.presentation_ms == 0
                {
                    return Err("Orb conversion requires actual relic identity, threshold, explicit range and host presentation time".into());
                }
            }
        }
        let mut ids = std::collections::BTreeSet::new();
        if self.modifiers.len() > 64 {
            return Err("Too many tile burst modifiers".into());
        }
        for m in &self.modifiers {
            if !super::weapon_marks::valid_id(&m.id)
                || !super::weapon_marks::valid_id(&m.relic_definition)
                || !ids.insert(&m.id)
            {
                return Err("Invalid tile burst modifier identity".into());
            }
            match &m.change {
                MarkChange::Range { pattern } => pattern.validate()?,
                MarkChange::ExplosionPower { percent } => percent.validate(true)?,
                MarkChange::ExplosionOverlap => {}
                _ => return Err("Modifier cannot target this tile burst".into()),
            }
        }
        Ok(())
    }
}
crate::authoring::clone_shape::struct_shape!(OrbConversion {
    relic_definition,
    threshold,
    base_power,
    power_per_stack,
    range,
    presentation_ms
});
crate::authoring::clone_shape::struct_shape!(TileBurstDef {
    source,
    power,
    delay_ms,
    range,
    damage_type,
    bonus_percent,
    dedup,
    modifiers
});
impl crate::authoring::clone_shape::CloneShape for TileBurstSource {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        m.scan_storage(Some(1))?;
        match self {
            Self::WalkablePath => Ok(()),
            Self::WeaponStacks {
                weapon,
                key,
                rank_bonus_percent,
                conversion,
                ..
            } => {
                weapon.visit_clone_shape(m)?;
                key.visit_clone_shape(m)?;
                rank_bonus_percent.visit_clone_shape(m)?;
                conversion.visit_clone_shape(m)
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileBurstsSnapshot {
    pub pending_casts: usize,
    pub released: Vec<ReleasedTileBurstSnapshot>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleasedTileBurstSnapshot {
    pub id: u64,
    pub caster: crate::game::battle::ids::UnitInstanceId,
    pub action: u64,
    pub round: u64,
    pub next_battle_time_ms: u64,
    pub presentation_remaining_ms: Option<u64>,
    pub orb_relic_copy: Option<uuid::Uuid>,
    pub centers: Vec<TileBurstCenterSnapshot>,
    pub applied_modifiers: Vec<super::marks::AppliedMarkModifier>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileBurstCenterSnapshot {
    pub tile: crate::game::resources::Position,
    pub remaining_rounds: u64,
    pub damage: u32,
}
