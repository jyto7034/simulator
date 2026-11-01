//! Paid preparation, a serial impact barrier and copy-owned inter-battle credit.
use super::{marks::MarkRatio, relation_action::PlanStatus, resonance::ResonanceAdmission};
use crate::game::battle::{damage::DamageType, tile_range::TileRangePattern};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreditLedgerDef {
    pub definition: String,
    pub grant: u32,
    pub credit_limit: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparationPolicy {
    pub maximum: u32,
    pub first_ms: u32,
    pub last_ms: u32,
    pub hold_ms: u32,
    pub deadline_ms: u32,
    pub decay_ms: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HomingImpactDef {
    pub definition: String,
    pub power: u32,
    pub damage_type: DamageType,
    pub speed: MarkRatio,
    pub range: MarkRatio,
    pub width: MarkRatio,
    pub lifetime_ms: u32,
    pub area: TileRangePattern,
    pub air: bool,
    pub obstacle_stop: bool,
    pub after: Vec<PlanStatus>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum VolleyChange {
    MarkDuration {
        negative: bool,
        percent: MarkRatio,
    },
    Area {
        pattern: TileRangePattern,
    },
    Recovery {
        factor: MarkRatio,
    },
    Protection,
    Multihit {
        step: MarkRatio,
        cap: MarkRatio,
    },
    Status {
        status: PlanStatus,
    },
    Continuous {
        percent: MarkRatio,
        cost_interval_ms: u32,
        interval_ms: u32,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VolleyModifier {
    pub id: String,
    pub relic_definition: String,
    pub change: VolleyChange,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedVolleyDef {
    pub definition: String,
    pub admission: ResonanceAdmission,
    pub ledger: CreditLedgerDef,
    pub mark_definition: String,
    pub mark_duration_ms: u32,
    pub recovery: u32,
    pub policy: PreparationPolicy,
    pub selection_range: TileRangePattern,
    pub selection_air: bool,
    pub projectile: HomingImpactDef,
    pub modifiers: Vec<VolleyModifier>,
}
impl PreparedVolleyDef {
    pub fn validate(&self) -> Result<(), String> {
        let valid = |s: &str| !s.trim().is_empty();
        self.admission.validate().map_err(str::to_owned)?;
        if !valid(&self.definition)
            || !valid(&self.ledger.definition)
            || !valid(&self.mark_definition)
            || self.ledger.grant == 0
            || self.ledger.credit_limit == 0
            || self.ledger.grant > i32::MAX as u32
            || self.ledger.credit_limit > i32::MAX as u32
            || self.mark_duration_ms == 0
            || self.recovery == 0
        {
            return Err("invalid prepared volley ledger or recovery".into());
        }
        let p = &self.policy;
        if p.maximum == 0
            || p.first_ms == 0
            || p.last_ms == 0
            || p.last_ms > p.first_ms
            || p.hold_ms == 0
            || p.deadline_ms == 0
            || p.decay_ms == 0
        {
            return Err("invalid accelerated preparation policy".into());
        }
        self.selection_range.validate()?;
        self.projectile.area.validate()?;
        let p = &self.projectile;
        if !valid(&p.definition) || p.lifetime_ms == 0 {
            return Err("invalid homing impact definition".into());
        }
        for v in [p.speed, p.range, p.width] {
            v.validate(false)?;
        }
        let status = |s: &PlanStatus| -> Result<(), String> {
            if !valid(&s.definition)
                || s.duration_ms == 0
                || !matches!(s.count,super::relation_action::ActionCount::Constant(n) if n>0)
            {
                return Err("invalid volley hit status".into());
            }
            s.percent.validate(true)
        };
        for s in &p.after {
            status(s)?;
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut continuous = 0;
        for m in &self.modifiers {
            if !valid(&m.id) || !valid(&m.relic_definition) || !ids.insert(&m.id) {
                return Err("invalid volley modifier".into());
            }
            match &m.change {
                VolleyChange::MarkDuration { negative, percent } => {
                    percent.validate(true)?;
                    if *negative
                        && u128::from(percent.numerator) >= 100 * u128::from(percent.denominator)
                    {
                        return Err("mark duration decrease must preserve positive duration".into());
                    }
                }
                VolleyChange::Area { pattern } => pattern.validate()?,
                VolleyChange::Recovery { factor } => factor.validate(false)?,
                VolleyChange::Protection => (),
                VolleyChange::Multihit { step, cap } => {
                    step.validate(true)?;
                    cap.validate(true)?;
                }
                VolleyChange::Status { status: s } => status(s)?,
                VolleyChange::Continuous {
                    percent,
                    cost_interval_ms,
                    interval_ms,
                } => {
                    continuous += 1;
                    percent.validate(false)?;
                    if !percent.at_most(100) || *cost_interval_ms == 0 || *interval_ms == 0 {
                        return Err("invalid continuous volley cost or cadence".into());
                    }
                }
            }
        }
        if continuous > 1 {
            return Err("conflicting continuous volley replacements".into());
        }
        Ok(())
    }
}
crate::authoring::clone_shape::struct_shape!(CreditLedgerDef {
    definition,
    grant,
    credit_limit
});
crate::authoring::clone_shape::struct_shape!(PreparationPolicy {
    maximum,
    first_ms,
    last_ms,
    hold_ms,
    deadline_ms,
    decay_ms
});
crate::authoring::clone_shape::struct_shape!(HomingImpactDef {
    definition,
    power,
    damage_type,
    speed,
    range,
    width,
    lifetime_ms,
    area,
    air,
    obstacle_stop,
    after
});
crate::authoring::clone_shape::struct_shape!(VolleyModifier {
    id,
    relic_definition,
    change
});
crate::authoring::clone_shape::struct_shape!(PreparedVolleyDef {
    definition,
    admission,
    ledger,
    mark_definition,
    mark_duration_ms,
    recovery,
    policy,
    selection_range,
    selection_air,
    projectile,
    modifiers
});
impl crate::authoring::clone_shape::CloneShape for VolleyChange {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        match self {
            Self::Area { pattern } => pattern.visit_clone_shape(m),
            Self::Status { status } => status.visit_clone_shape(m),
            _ => m.scan_storage(Some(1)),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolleyPhase {
    Preparing,
    Decaying,
    Firing,
    Continuous,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedVolleyOwnerView {
    pub copy: uuid::Uuid,
    pub employee: uuid::Uuid,
    pub caster: Option<crate::game::battle::ids::UnitInstanceId>,
    pub balance: i64,
    pub can_borrow: bool,
    pub borrowed: bool,
    pub activation: Option<u64>,
    pub phase: Option<VolleyPhase>,
    pub prepared: u32,
    pub next_at_ms: Option<u64>,
    pub next_cost_ms: Option<u64>,
    pub deadline_ms: Option<u64>,
    pub flight: Option<u64>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolleyFlightEnd {
    Impact,
    TargetLost,
    Range,
    Lifetime,
    Obstacle,
    Boundary,
    BattleEnded,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VolleyFlightResult {
    pub id: u64,
    pub activation: u64,
    pub copy: uuid::Uuid,
    pub target: crate::game::battle::ids::UnitInstanceId,
    pub released_at_ms: u64,
    pub ended_at_ms: u64,
    pub end: VolleyFlightEnd,
    pub hits: Vec<(crate::game::battle::ids::UnitInstanceId, u64)>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedVolleysSnapshot {
    pub owners: Vec<PreparedVolleyOwnerView>,
    pub flights: Vec<u64>,
    pub settled: Vec<VolleyFlightResult>,
    pub marks: Vec<(uuid::Uuid, crate::game::battle::ids::UnitInstanceId, u64)>,
}

crate::authoring::clone_shape::struct_shape!(VolleyFlightResult {
    id,
    activation,
    copy,
    target,
    released_at_ms,
    ended_at_ms,
    end,
    hits
});

crate::authoring::clone_shape::copy_shape!(VolleyPhase, VolleyFlightEnd);
