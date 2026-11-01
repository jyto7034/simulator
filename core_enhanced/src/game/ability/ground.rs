//! Event subscriptions and independently owned contributions of a fixed ground area.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroundStatusKind {
    Slow {
        percent: u32,
    },
    /// Integer percentage floor of the linear elapsed/duration ramp.
    RampingSlow {
        duration_ms: u32,
        max_percent: u32,
        rate_numerator: u64,
        rate_denominator: u64,
    },
    Stun,
    DamageTaken {
        percent: u32,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroundOverlap {
    Sum,
    Strongest,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroundStatusLifetime {
    UntilExit,
    UntilAreaEnd,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundStatusDef {
    pub definition: String,
    pub kind: GroundStatusKind,
    pub overlap: GroundOverlap,
    pub lifetime: GroundStatusLifetime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vulnerability: Option<super::sustained::HitVulnerability>,
}
crate::authoring::clone_shape::struct_shape!(GroundStatusDef {
    definition,
    kind,
    overlap,
    lifetime,
    vulnerability
});
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroundTrigger {
    Enter,
    Dwell { duration_ms: u32 },
    Exit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroundStatusAction {
    Apply,
    Clear,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundStatusRule {
    pub trigger: GroundTrigger,
    pub status: usize,
    pub action: GroundStatusAction,
}
crate::authoring::clone_shape::struct_shape!(GroundStatusRule {
    trigger,
    status,
    action
});
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundStatusProgram {
    pub statuses: Vec<GroundStatusDef>,
    pub rules: Vec<GroundStatusRule>,
}
crate::authoring::clone_shape::struct_shape!(GroundStatusProgram { statuses, rules });
impl GroundStatusProgram {
    pub fn validate(&self) -> Result<(), String> {
        if self.statuses.is_empty()
            || self.statuses.len() > 64
            || self.rules.is_empty()
            || self.rules.len() > 128
        {
            return Err("ground status requires 1..64 definitions and 1..128 subscriptions".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for status in &self.statuses {
            if let Some(v) = &status.vulnerability {
                if status.kind != GroundStatusKind::Stun
                    || v.definition.is_empty()
                    || v.definition.len() > 512
                    || v.percent == 0
                    || v.percent > i32::MAX as u32
                    || v.duration_ms == 0
                {
                    return Err("ground control vulnerability requires stun and a finite positive native modifier".into());
                }
            }
            if status.definition.is_empty()
                || status.definition.len() > 256
                || !ids.insert(&status.definition)
            {
                return Err(
                    "ground status definition identities must be unique and bounded".into(),
                );
            }
            if let GroundStatusKind::RampingSlow {
                duration_ms,
                max_percent,
                rate_numerator,
                rate_denominator,
            } = status.kind
            {
                if duration_ms == 0
                    || rate_numerator == 0
                    || rate_denominator == 0
                    || max_percent > 100
                    || status.lifetime != GroundStatusLifetime::UntilExit
                    || status.overlap != GroundOverlap::Strongest
                {
                    return Err("ramping slow requires positive duration, 0..100 percent and strongest membership lifetime".into());
                }
            }
            if matches!(status.kind, GroundStatusKind::Slow { percent } | GroundStatusKind::DamageTaken { percent } if percent > i32::MAX as u32)
            {
                return Err(
                    "ground numeric status exceeds the native signed percentage range".into(),
                );
            }
        }
        for rule in &self.rules {
            if rule.status >= self.statuses.len()
                || matches!(rule.trigger, GroundTrigger::Dwell { duration_ms: 0 })
                || (rule.trigger == GroundTrigger::Exit && rule.action == GroundStatusAction::Apply)
            {
                return Err(
                    "invalid ground status subscription or exited relation application".into(),
                );
            }
        }
        Ok(())
    }
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        self.statuses.iter().try_fold(
            self.statuses
                .capacity()
                .checked_mul(std::mem::size_of::<GroundStatusDef>())?
                .checked_add(
                    self.rules
                        .capacity()
                        .checked_mul(std::mem::size_of::<GroundStatusRule>())?,
                )?,
            |sum, status| {
                let extra = status.vulnerability.as_ref().map_or(Some(0), |v| {
                    v.definition
                        .capacity()
                        .checked_add(v.modifier_source.as_ref().map_or(0, |s| {
                            s.definition.capacity().saturating_add(s.rule.capacity())
                        }))
                })?;
                sum.checked_add(status.definition.capacity())?
                    .checked_add(extra)
            },
        )
    }
}

crate::authoring::clone_shape::copy_shape!(
    GroundStatusKind,
    GroundOverlap,
    GroundStatusLifetime,
    GroundTrigger,
    GroundStatusAction
);
