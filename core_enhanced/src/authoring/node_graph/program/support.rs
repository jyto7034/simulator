use crate::game::battle::tile_range::TileRangePattern;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportProgram {
    pub family: String,
    pub duration_ms: u32,
    pub range: TileRangePattern,
    pub effect: SupportEffectProgram,
    pub conversion: Option<SupportConversionProgram>,
    #[serde(default)]
    pub policy: crate::game::ability::action::ActionPolicy,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SupportEffectProgram {
    PersonalShield {
        max_health_percent: u32,
    },
    Channel {
        damage_percent: u32,
        speed_points: u32,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupportConversionProgram {
    pub relic_definition: String,
    pub selection_range: TileRangePattern,
    pub replacement: SupportReplacementProgram,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SupportReplacementProgram {
    PersonalShield {
        numerator: u64,
        denominator: u64,
    },
    Channel {
        damage_percent: u32,
        speed_points: u32,
    },
}
impl SupportProgram {
    pub fn validate(&self) -> Result<(), String> {
        if self.family.is_empty() || self.family.len() > 128 || self.duration_ms == 0 {
            return Err("Support requires a bounded family and positive duration".into());
        }
        self.range.validate()?;
        match self.effect {
            SupportEffectProgram::PersonalShield { max_health_percent }
                if (1..=10000).contains(&max_health_percent) =>
            {
                ()
            }
            SupportEffectProgram::Channel {
                damage_percent,
                speed_points,
            } if damage_percent <= 10000
                && speed_points <= 10000
                && (damage_percent > 0 || speed_points > 0) =>
            {
                ()
            }
            _ => return Err("Support values are out of range".into()),
        }
        if let Some(conversion) = &self.conversion {
            if conversion.relic_definition.is_empty()
                || conversion.relic_definition.len() > 128
                || !conversion
                    .relic_definition
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
            {
                return Err("Support conversion requires a bounded relic definition".into());
            }
            conversion.selection_range.validate()?;
            match (&self.effect, &conversion.replacement) {
                (
                    SupportEffectProgram::PersonalShield { .. },
                    SupportReplacementProgram::PersonalShield {
                        numerator,
                        denominator,
                    },
                ) if *numerator > 0
                    && *denominator > 0
                    && *numerator <= 9_007_199_254_740_991
                    && *denominator <= 9_007_199_254_740_991 =>
                {
                    ()
                }
                (
                    SupportEffectProgram::Channel { .. },
                    SupportReplacementProgram::Channel {
                        damage_percent,
                        speed_points,
                    },
                ) if *damage_percent <= 10000 && *speed_points <= 10000 => (),
                _ => {
                    return Err(
                        "Support conversion has incompatible or unbounded replacement".into(),
                    )
                }
            }
        }
        Ok(())
    }
}
