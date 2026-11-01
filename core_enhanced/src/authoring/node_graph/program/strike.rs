use crate::game::{
    ability::action::ActionPolicy,
    battle::{damage::DamageType, tile_range::TileRangePattern},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrikeProgram {
    pub acquisition: TileRangePattern,
    pub include_air: bool,
    pub parts: Vec<StrikePart>,
    pub policy: ActionPolicy,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StrikePart {
    Wait {
        duration_ms: u32,
    },
    Strike {
        id: String,
        range: TileRangePattern,
        damage_type: DamageType,
        numerator: u64,
        denominator: u64,
        missing_hp: Option<(u32, u32)>,
        after: Vec<StrikeStatus>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StrikeStatus {
    Pull {
        distance: u32,
    },
    Stun {
        definition: String,
        duration_ms: u32,
    },
}
impl StrikeProgram {
    pub fn validate(&self) -> Result<(), String> {
        self.acquisition.validate()?;
        if self.parts.is_empty() || self.parts.len() > 128 {
            return Err("Strike plan requires 1..128 parts".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut strikes = 0;
        for p in &self.parts {
            if let StrikePart::Strike {
                id,
                range,
                numerator,
                denominator,
                missing_hp,
                after,
                ..
            } = p
            {
                strikes += 1;
                if id.is_empty()
                    || id.len() > 128
                    || !ids.insert(id)
                    || *numerator == 0
                    || *denominator == 0
                {
                    return Err("Invalid strike identity or coefficient".into());
                }
                range.validate()?;
                if missing_hp.is_some_and(|(p, c)| p == 0 || c == 0) {
                    return Err("Missing HP scaling requires positive rate and cap".into());
                }
                if after.len() > 16 {
                    return Err("Too many hit statuses".into());
                }
                for s in after {
                    match s {
                        StrikeStatus::Pull { distance } if *distance == 0 => {
                            return Err("Pull requires positive distance".into())
                        }
                        StrikeStatus::Stun {
                            definition,
                            duration_ms,
                        } if definition.is_empty()
                            || definition.len() > 256
                            || *duration_ms == 0 =>
                        {
                            return Err("Stun requires definition and duration".into())
                        }
                        _ => (),
                    }
                }
            }
        }
        if strikes == 0 {
            return Err("Strike plan has no strikes".into());
        }
        Ok(())
    }
}
