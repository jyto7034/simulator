//! Conditional changes to an authored finite action. Actual equipped copies
//! resolve these declarations in the immutable employee plan before acceptance.
use crate::game::{battle::tile_range::TileRangePattern, relic::RelicVariant};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionPolicy {
    #[serde(default)]
    pub external_interrupt_protected: bool,
    #[serde(default)]
    pub modifiers: Vec<ActionModifier>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionModifier {
    pub id: String,
    pub relic_definition: String,
    pub variant: RelicVariant,
    pub change: ActionChange,
}
impl ActionModifier {
    /// The provider position qualifies this authored rule inside one effective
    /// plan. It is provenance, not a transferable execution capability.
    pub(crate) fn contribution_rule_id(&self, provider: usize) -> String {
        format!("action-{provider}-{}", self.id)
    }

    /// Static rule correspondence only. The occurrence's actual provider and
    /// copy must come from the captured runtime plan, not this name check.
    pub(crate) fn matches_contribution_rule_id(&self, value: &str) -> bool {
        let Some((provider, authored)) = value.strip_prefix("action-")
            .and_then(|tail| tail.split_once('-')) else { return false; };
        let Ok(index) = provider.parse::<usize>() else { return false; };
        provider == index.to_string() && authored == self.id
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ActionChange {
    Area { step: String, change: AreaChange },
    SupportRange { step: String, effect: String, pattern: TileRangePattern },
    SupportRadius { step: String, effect: String, percent: u32 },
    TauntDamage { step: String, effect: String, taken_percent: u32, basic_percent: u32 },
    Range {
        steps: Vec<String>,
        pattern: TileRangePattern,
    },
    Duration {
        step: String,
        effect: String,
        percent: u32,
    },
    ControlVulnerability {
        step: String,
        effect: String,
        percent: u32,
        duration_ms: u32,
    },
    Multihit {
        steps: Vec<String>,
        increment_percent: u32,
        cap_percent: u32,
    },
    ProtectInterruption,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AreaChange {
    Duration { percent: u32 },
    RampRate { status: usize, reached_rules: Vec<usize>, numerator: u64, denominator: u64 },
    ControlVulnerability { status: usize, percent: u32, duration_ms: u32 },
}
impl AreaChange {
    pub fn validate(&self, area: &super::SkillTileAreaDeliveryDef) -> Result<(), String> {
        use super::ground::*;
        if !matches!(area.mode, super::SkillTileAreaMode::ChannelGround { .. }) { return Err("Area modifier requires a channel-owned area".into()); }
        match self {
            Self::Duration { percent } if *percent > 0 && *percent <= i32::MAX as u32 => Ok(()),
            Self::RampRate { status, reached_rules, numerator, denominator } => {
                if *numerator < *denominator || *denominator == 0 || reached_rules.len() > 128 { return Err("Invalid ramp progress ratio or rule count".into()); }
                let p = area.ground_status.as_ref().ok_or("Ramp requires ground status program")?;
                let Some(GroundStatusDef { kind: GroundStatusKind::RampingSlow { duration_ms, rate_numerator, rate_denominator, .. }, .. }) = p.statuses.get(*status) else { return Err("Ramp modifier requires a ramping slow definition".into()); };
                let arrival = u32::try_from((u128::from(*duration_ms) * u128::from(*rate_denominator)).div_ceil(u128::from(*rate_numerator))).map_err(|_| "Ramp arrival exceeds native duration")?;
                let mut unique = std::collections::BTreeSet::new();
                for rule in reached_rules {
                    if !unique.insert(rule) || !p.rules.get(*rule).is_some_and(|r| r.trigger == GroundTrigger::Dwell { duration_ms: arrival }) { return Err("Ramp arrival rule does not share its progress duration".into()); }
                }
                Ok(())
            }
            Self::ControlVulnerability { status, percent, duration_ms } if *percent > 0 && *percent <= i32::MAX as u32 && *duration_ms > 0 => {
                if !area.ground_status.as_ref().and_then(|p| p.statuses.get(*status)).is_some_and(|s| s.kind == GroundStatusKind::Stun) { return Err("Area vulnerability requires a stun definition".into()); }
                Ok(())
            }
            _ => Err("Area modifier requires a positive bounded value".into()),
        }
    }
}
impl ActionPolicy {
    pub fn validate(&self, skill: &super::SkillDef) -> Result<(), String> {
        if self.modifiers.len() > 128 {
            return Err("Too many action modifiers".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for m in &self.modifiers {
            for id in [&m.id, &m.relic_definition] {
                if id.is_empty() || id.trim() != id || id.len() > 256 {
                    return Err("Invalid action modifier identity".into());
                }
            }
            if !ids.insert(&m.id) {
                return Err("Duplicate action modifier".into());
            }
            let step = |id: &str| {
                skill
                    .steps
                    .iter()
                    .find(|s| s.id == id)
                    .ok_or_else(|| "Unknown action modifier step".to_string())
            };
            match &m.change {
                ActionChange::SupportRadius { step: id, effect, percent } => {
                    if *percent == 0 || *percent > i32::MAX as u32 {
                        return Err("Support radius percent outside positive native range".into());
                    }
                    if !step(id)?.effects.iter().any(|e| &e.id == effect && support_radius(&e.op).is_some()) {
                        return Err("Support radius requires an authored circular group support effect".into());
                    }
                }
                ActionChange::SupportRange { step: id, effect, pattern } => {
                    pattern.validate()?;
                    if !step(id)?.effects.iter().any(|e| &e.id == effect && matches!(&e.op, super::SkillEffectDef::Sustained { definition: super::sustained::SustainedEffectDef::ChannelAura { aura, .. } } if aura.support.is_some())) {
                        return Err("Support range requires a channel support footprint".into());
                    }
                }
                ActionChange::Area { step: id, change } => {
                    let super::DeliveryDef::TileArea { area, .. } = &step(id)?.delivery else { return Err("Area modifier requires tile area delivery".into()); };
                    change.validate(area)?;
                }
                ActionChange::TauntDamage { step: id, effect, taken_percent, basic_percent } => {
                    if *taken_percent > 100 || *basic_percent > i32::MAX as u32 || (*taken_percent == 0 && *basic_percent == 0) {
                        return Err("Taunt modifier requires a valid positive relation coefficient".into());
                    }
                    if !step(id)?.effects.iter().any(|e| &e.id == effect && matches!(e.op, super::SkillEffectDef::Sustained {
                        definition: super::sustained::SustainedEffectDef::Control { kind: super::sustained::ControlKind::ApproachTaunt { .. }, .. }
                    })) { return Err("Taunt modifier requires an approach taunt effect".into()); }
                }
                ActionChange::Range { steps, pattern } => {
                    pattern.validate()?;
                    if steps.is_empty() || steps.len() > 64 {
                        return Err("Range requires finite target steps".into());
                    }
                    for id in steps {
                        if !matches!(step(id)?.delivery, super::DeliveryDef::TileArea { .. }) {
                            return Err("Range requires tile area steps".into());
                        }
                    }
                }
                ActionChange::Multihit {
                    steps,
                    increment_percent,
                    cap_percent,
                } => {
                    if steps.is_empty()
                        || steps.len() > 64
                        || *increment_percent == 0
                        || *cap_percent == 0
                    {
                        return Err("Multihit requires steps and positive rates".into());
                    }
                    for id in steps {
                        if !step(id)?.effects.iter().any(|e| {
                            matches!(
                                e.op,
                                super::SkillEffectDef::Damage { .. }
                                    | super::SkillEffectDef::DamageExpr { .. }
                            )
                        }) {
                            return Err("Multihit requires damage".into());
                        }
                    }
                }
                ActionChange::Duration {
                    step: id,
                    effect,
                    percent,
                }
                | ActionChange::ControlVulnerability {
                    step: id,
                    effect,
                    percent,
                    ..
                } => {
                    if *percent == 0 || *percent > i32::MAX as u32 {
                        return Err("Modifier percent outside positive native range".into());
                    }
                    let Some(e) = step(id)?.effects.iter().find(|e| &e.id == effect) else {
                        return Err("Unknown modifier effect".into());
                    };
                    let hit = matches!(e.op, super::SkillEffectDef::Sustained { definition: super::sustained::SustainedEffectDef::HitStatus { .. } });
                    let control_duration = matches!(m.change, ActionChange::Duration { .. }) && matches!(e.op, super::SkillEffectDef::Sustained { definition: super::sustained::SustainedEffectDef::Control { .. } | super::sustained::SustainedEffectDef::ChannelAura { .. } });
                    if !hit && !control_duration { return Err("Modifier requires an authored timed status".into()); }
                    if let ActionChange::ControlVulnerability { duration_ms, .. } = &m.change {
                        if *duration_ms == 0 {
                            return Err("Vulnerability requires positive duration".into());
                        }
                        if !matches!(&e.op, super::SkillEffectDef::Sustained { definition: super::sustained::SustainedEffectDef::HitStatus { profile, .. } } if profile.kind.is_hard_control() && profile.vulnerability.is_none()) {
                            return Err("Vulnerability requires a stun or freeze without an existing vulnerability".into());
                        }
                    }
                }
                ActionChange::ProtectInterruption => (),
            }
        }
        Ok(())
    }
}

/// Only group footprints are scalable. Converted selected-target reach is a
/// separate policy, and a tile mask has no implied continuous radius.
pub(crate) fn support_radius(effect: &super::SkillEffectDef) -> Option<u32> {
    if crate::game::relic::structural::support::policy(effect)
        .and_then(|p| p.range.as_ref()).is_some() { return None; }
    let radius = match effect {
        super::SkillEffectDef::Sustained { definition: super::sustained::SustainedEffectDef::ChannelAura { aura, .. } } => aura.radius_units,
        super::SkillEffectDef::PersonalShieldGrant { definition } => match definition.recipients {
            crate::game::relic::structural::authoring::PersonalShieldRecipientsDef::PlayerEmployeesAtRelease { radius_units } => radius_units,
            _ => return None,
        },
        _ => return None,
    };
    (radius > 0).then_some(radius)
}
