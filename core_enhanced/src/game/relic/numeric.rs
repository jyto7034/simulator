//! One checked additive relic class, shared by execution and immutable replay.
//! This module performs arithmetic; it does not grant runtime plan authority.
use serde::Serialize;
use uuid::Uuid;

use super::RelicVariant;
use crate::game::{ability::authoring::EffectSite, battle::ids::UnitInstanceId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RelicNumericTarget {
    SkillDamage {
        site: EffectSite,
    },
    BasicAttackDamage {
        actor: UnitInstanceId,
        attack_epoch: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelicDamageContribution {
    pub copy_uuid: Uuid,
    pub definition_id: String,
    pub variant: RelicVariant,
    pub effect_id: String,
    pub flat: i64,
    pub percent: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelicNumericFaultCode {
    Overflow,
    InvalidCoefficient,
}

/// The base has already passed its original typed and range guards. No relic
/// can rescue an invalid base; this function performs the final guard too.
pub(crate) fn compose_damage(
    base: u32,
    flat: i128,
    percent: i128,
) -> Result<u32, RelicNumericFaultCode> {
    if flat < 0 || percent < 0 {
        return Err(RelicNumericFaultCode::InvalidCoefficient);
    }
    let effective = i128::from(base)
        .checked_add(flat)
        .and_then(|value| {
            percent
                .checked_add(100)
                .and_then(|coefficient| value.checked_mul(coefficient))
        })
        .map(|value| value / 100)
        .ok_or(RelicNumericFaultCode::Overflow)?;
    u32::try_from(effective).map_err(|_| RelicNumericFaultCode::Overflow)
}

pub(crate) fn contribution_totals(
    values: &[RelicDamageContribution],
) -> Result<(i128, i128), RelicNumericFaultCode> {
    values
        .iter()
        .try_fold((0i128, 0i128), |(flat, percent), value| {
            if value.flat < 0 || value.percent < 0 {
                return Err(RelicNumericFaultCode::InvalidCoefficient);
            }
            Ok((
                flat.checked_add(i128::from(value.flat))
                    .ok_or(RelicNumericFaultCode::Overflow)?,
                percent
                    .checked_add(i128::from(value.percent))
                    .ok_or(RelicNumericFaultCode::Overflow)?,
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relic_numeric_once_additive_fraction_and_final_narrow() {
        assert_eq!(compose_damage(100, 20, 20 + 30), Ok(180));
        assert_eq!(compose_damage(3, 0, 50), Ok(4));
        assert_eq!(compose_damage(110, 20, 50), Ok(195));
        assert_eq!(compose_damage(u32::MAX, 0, 0), Ok(u32::MAX));
        assert_eq!(
            compose_damage(u32::MAX, 1, 0),
            Err(RelicNumericFaultCode::Overflow)
        );
        assert_eq!(
            compose_damage(1, i128::MAX, 0),
            Err(RelicNumericFaultCode::Overflow)
        );
        assert_eq!(
            compose_damage(1, 0, i128::MAX),
            Err(RelicNumericFaultCode::Overflow)
        );
        assert_eq!(
            compose_damage(10, -1, 0),
            Err(RelicNumericFaultCode::InvalidCoefficient)
        );
    }
}
