//! Battle-copy relationships and an admitted finite action plan. Display names
//! carry no execution behavior; references address validated authored definitions.
use super::{marks::MarkRatio, resonance::ResonanceAdmission};
use crate::game::battle::{damage::DamageType, tile_range::TileRangePattern};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ActionCount {
    Constant(u32),
    StartStock,
    Paid,
    Quotient {
        count: Box<ActionCount>,
        divisor: u32,
        minimum: u32,
    },
    Sum {
        first: Box<ActionCount>,
        second: Box<ActionCount>,
    },
}
impl ActionCount {
    pub fn evaluate(&self, stock: u32, paid: u32) -> Option<u32> {
        match self {
            Self::Constant(n) => Some(*n),
            Self::StartStock => Some(stock),
            Self::Paid => Some(paid),
            Self::Quotient {
                count,
                divisor,
                minimum,
            } => Some((count.evaluate(stock, paid)?.checked_div(*divisor)?).max(*minimum)),
            Self::Sum { first, second } => first
                .evaluate(stock, paid)?
                .checked_add(second.evaluate(stock, paid)?),
        }
    }
    fn validate(&self, paid: bool, depth: usize) -> Result<(), String> {
        if depth > 32 {
            return Err("action count nesting limit".into());
        }
        match self {
            Self::Paid if !paid => Err("paid count outside a spending step".into()),
            Self::Quotient { count, divisor, .. } => {
                if *divisor == 0 {
                    return Err("zero count divisor".into());
                }
                count.validate(paid, depth + 1)
            }
            Self::Sum { first, second } => {
                first.validate(paid, depth + 1)?;
                second.validate(paid, depth + 1)
            }
            _ => Ok(()),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanTarget {
    Primary,
    SelectedArea,
    ForwardArea,
    SelectedDirectionArea,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanStatusKind {
    Stun,
    Root,
    BasicLock,
    Vulnerability,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanStatus {
    pub definition: String,
    pub kind: PlanStatusKind,
    pub duration_ms: u32,
    pub count: ActionCount,
    pub percent: MarkRatio,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanRepeatTiming {
    Interval { delayed_first: bool },
    FixedTotal,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanRange {
    pub definition: String,
    pub pattern: TileRangePattern,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ActionPlan {
    Strike {
        target: PlanTarget,
        power: u32,
        damage_type: DamageType,
        range: Option<PlanRange>,
        after: Vec<PlanStatus>,
        ramp: MarkRatio,
    },
    Sequence {
        first: usize,
        next: usize,
    },
    Wait {
        duration_ms: u32,
    },
    Repeat {
        body: usize,
        count: ActionCount,
        time_ms: u32,
        timing: PlanRepeatTiming,
    },
    Spend {
        amount: Option<ActionCount>,
        body: usize,
        after: Option<PlanStatus>,
    },
    Stance {
        duration_ms: u32,
        reduction: MarkRatio,
    },
    Timed {
        body: usize,
        base_ms: u32,
        per_count_ms: u32,
        count: ActionCount,
        interval_ms: u32,
        delayed_first: bool,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedActionPlan {
    pub definition: String,
    pub op: ActionPlan,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationActionEntry {
    pub definition: String,
    pub plan: usize,
    pub next_basic: bool,
    pub interrupt_on_silence: bool,
    pub protected: bool,
    pub selector: Option<PlanRange>,
    pub self_damage: Option<(u32, MarkRatio, DamageType)>,
    pub manual_stop: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RelationActionChange {
    Period {
        definition: String,
        negative: bool,
        percent: MarkRatio,
    },
    UnlimitedStock,
    Gain {
        factor: MarkRatio,
    },
    Shield {
        percent: MarkRatio,
    },
    StatusCount {
        definition: String,
        factor: MarkRatio,
    },
    Range {
        definition: String,
        pattern: TileRangePattern,
    },
    PrimaryArea {
        definition: String,
    },
    Protection,
    Multihit {
        entry: String,
        step: MarkRatio,
        cap: MarkRatio,
    },
    Retarget {
        entry: String,
        distance_units: u32,
    },
    ControlVulnerability {
        definition: String,
        percent: u32,
        duration_ms: u32,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationActionModifier {
    pub id: String,
    pub relic_definition: String,
    pub change: RelationActionChange,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationActionDef {
    pub definition: String,
    pub stock_definition: String,
    pub relation_definition: String,
    pub mode_definition: String,
    pub menu_entry: String,
    pub admission: ResonanceAdmission,
    pub selection_range: Option<TileRangePattern>,
    pub initial_stock: u32,
    pub stock_capacity: Option<u32>,
    pub period_ms: u32,
    pub periodic_gain: u32,
    pub reduction: MarkRatio,
    pub shield_threshold: u32,
    pub shield_percent: MarkRatio,
    pub shield_duration_ms: u32,
    pub mode_damage_percent: MarkRatio,
    pub normal: RelationActionEntry,
    pub active: RelationActionEntry,
    pub plans: Vec<NamedActionPlan>,
    pub modifiers: Vec<RelationActionModifier>,
}
fn identity(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 128
}
fn ratio(r: MarkRatio) -> bool {
    r.denominator > 0 && r.numerator <= i64::MAX as u64 && r.denominator <= i64::MAX as u64
}
fn percent(r: MarkRatio) -> bool {
    ratio(r) && u128::from(r.numerator) <= 100 * u128::from(r.denominator)
}
impl RelationActionDef {
    pub fn validate(&self) -> Result<(), String> {
        self.admission.validate()?;
        if ![
            &self.definition,
            &self.stock_definition,
            &self.relation_definition,
            &self.mode_definition,
            &self.menu_entry,
        ]
        .iter()
        .all(|s| identity(s))
            || self.period_ms == 0
            || self.periodic_gain == 0
            || self.shield_duration_ms == 0
            || self
                .stock_capacity
                .is_some_and(|c| c == 0 || self.initial_stock > c)
            || !percent(self.reduction)
            || self.shield_threshold > 100
            || !ratio(self.shield_percent)
            || !ratio(self.mode_damage_percent)
            || self.plans.is_empty()
            || self.plans.len() > 2000
            || self.modifiers.len() > 256
        {
            return Err("invalid relationship action declaration".into());
        }
        if let Some(r) = &self.selection_range {
            r.validate()?;
        }
        let mut definitions = std::collections::BTreeSet::new();
        for p in &self.plans {
            if !identity(&p.definition) || !definitions.insert(&p.definition) {
                return Err("duplicate plan identity".into());
            }
        }
        for e in [&self.normal, &self.active] {
            if !identity(&e.definition)
                || (e.next_basic && (self.selection_range.is_some() || e.selector.is_some()))
            {
                return Err("invalid action entry".into());
            }
            if let Some(r) = &e.selector {
                r.pattern.validate()?;
            }
            if let Some((interval, value, _)) = e.self_damage {
                if interval == 0 || !percent(value) {
                    return Err("invalid periodic self damage".into());
                }
            }
            self.validate_plan(
                e.plan,
                false,
                &mut Vec::new(),
                &mut std::collections::BTreeSet::new(),
            )?;
            let mut pending = vec![e.plan];
            let mut seen = std::collections::BTreeSet::new();
            while let Some(i) = pending.pop() {
                if !seen.insert(i) {
                    continue;
                }
                match &self.plans[i].op {
                    ActionPlan::Strike {
                        target: PlanTarget::Primary,
                        ..
                    } if !e.next_basic && e.selector.is_none() => {
                        return Err("Primary action requires a basic target or selector".into())
                    }
                    ActionPlan::Strike {
                        target: PlanTarget::SelectedArea | PlanTarget::SelectedDirectionArea,
                        ..
                    } if self.selection_range.is_none() => {
                        return Err("Selected area requires explicit tile selection".into())
                    }
                    ActionPlan::Sequence { first, next } => {
                        pending.push(*first);
                        pending.push(*next);
                    }
                    ActionPlan::Spend { body, .. }
                    | ActionPlan::Repeat { body, .. }
                    | ActionPlan::Timed { body, .. } => pending.push(*body),
                    _ => (),
                }
            }
            if (e.self_damage.is_some() || e.manual_stop)
                && !matches!(self.plans[e.plan].op, ActionPlan::Timed { .. })
            {
                return Err("Periodic self damage and stop input require a timed root plan".into());
            }
        }
        let statuses: Vec<_> = self
            .plans
            .iter()
            .flat_map(|p| match &p.op {
                ActionPlan::Strike { after, .. } => after.iter().collect::<Vec<_>>(),
                ActionPlan::Spend { after, .. } => after.iter().collect(),
                _ => vec![],
            })
            .collect();
        let mut modifier_ids = std::collections::BTreeSet::new();
        for m in &self.modifiers {
            if !modifier_ids.insert(&m.id) {
                return Err("Duplicate relationship modifier identity".into());
            }
            let valid_target=match &m.change{
                RelationActionChange::StatusCount{definition,..}=>statuses.iter().any(|s|&s.definition==definition),
                RelationActionChange::ControlVulnerability{definition,..}=>statuses.iter().any(|s|&s.definition==definition&&s.kind==PlanStatusKind::Stun),
                RelationActionChange::Range{definition,..}=>self.plans.iter().any(|p|matches!(&p.op,ActionPlan::Strike{range:Some(r),..} if &r.definition==definition)),
                RelationActionChange::PrimaryArea{definition}=>self.plans.iter().any(|p|&p.definition==definition&&matches!(p.op,ActionPlan::Strike{target:PlanTarget::Primary,..})),
                RelationActionChange::Multihit{entry,..}|RelationActionChange::Retarget{entry,..}=>entry==&self.normal.definition||entry==&self.active.definition,
                RelationActionChange::Period{definition,..}=>identity(definition),_=>true,
            };
            if !valid_target {
                return Err("Modifier references an unrelated definition".into());
            }
            if !identity(&m.id) || !identity(&m.relic_definition) {
                return Err("invalid modifier identity".into());
            }
            match &m.change {
                RelationActionChange::Period {
                    percent: p,
                    negative,
                    ..
                } => {
                    if !ratio(*p)
                        || (*negative && u128::from(p.numerator) >= 100 * u128::from(p.denominator))
                    {
                        return Err("invalid period modifier".into());
                    }
                }
                RelationActionChange::Gain { factor }
                | RelationActionChange::StatusCount { factor, .. } => {
                    if !ratio(*factor) || factor.numerator == 0 {
                        return Err("invalid count factor".into());
                    }
                }
                RelationActionChange::Shield { percent: p } => {
                    if !ratio(*p) {
                        return Err("invalid shield modifier".into());
                    }
                }
                RelationActionChange::Range { pattern, .. } => pattern.validate()?,
                RelationActionChange::Multihit { step, cap, .. } => {
                    if !ratio(*step) || !ratio(*cap) {
                        return Err("invalid multihit modifier".into());
                    }
                }
                RelationActionChange::Retarget { distance_units, .. } => {
                    if *distance_units == 0 {
                        return Err("invalid retarget radius".into());
                    }
                }
                RelationActionChange::ControlVulnerability {
                    percent,
                    duration_ms,
                    ..
                } => {
                    if *duration_ms == 0 || *percent > i32::MAX as u32 {
                        return Err("invalid control followup".into());
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }
    fn validate_status(s: &PlanStatus, paid: bool) -> Result<(), String> {
        if !identity(&s.definition) || s.duration_ms == 0 || !ratio(s.percent) {
            return Err("invalid plan status".into());
        }
        s.count.validate(paid, 0)
    }
    fn validate_plan(
        &self,
        i: usize,
        paid: bool,
        path: &mut Vec<usize>,
        checked: &mut std::collections::BTreeSet<(usize, bool)>,
    ) -> Result<(), String> {
        if path.len() >= 64 || path.contains(&i) {
            return Err("cyclic or overly deep action plan".into());
        }
        if checked.contains(&(i, paid)) {
            return Ok(());
        }
        let p = &self.plans.get(i).ok_or("missing action plan")?.op;
        path.push(i);
        match p {
            ActionPlan::Strike {
                target,
                power,
                range,
                after,
                ramp,
                ..
            } => {
                if *power > i32::MAX as u32
                    || !ratio(*ramp)
                    || after.len() > 32
                    || (*target != PlanTarget::Primary && range.is_none())
                {
                    return Err("invalid strike plan".into());
                }
                if let Some(r) = range {
                    if !identity(&r.definition) {
                        return Err("invalid range identity".into());
                    }
                    r.pattern.validate()?;
                }
                for s in after {
                    Self::validate_status(s, paid)?;
                }
            }
            ActionPlan::Sequence { first, next } => {
                self.validate_plan(*first, paid, path, checked)?;
                self.validate_plan(*next, paid, path, checked)?;
            }
            ActionPlan::Wait { duration_ms } => {
                if *duration_ms == 0 {
                    return Err("zero wait".into());
                }
            }
            ActionPlan::Stance {
                duration_ms,
                reduction,
            } => {
                if *duration_ms == 0 || !percent(*reduction) {
                    return Err("invalid stance".into());
                }
            }
            ActionPlan::Repeat {
                body,
                count,
                time_ms,
                ..
            } => {
                if *time_ms == 0
                    || !matches!(
                        self.plans.get(*body).map(|p| &p.op),
                        Some(ActionPlan::Strike { .. })
                    )
                {
                    return Err("repeat requires one immediate strike".into());
                }
                count.validate(paid, 0)?;
                self.validate_plan(*body, paid, path, checked)?;
            }
            ActionPlan::Spend {
                amount,
                body,
                after,
            } => {
                if let Some(c) = amount {
                    c.validate(paid, 0)?;
                }
                self.validate_plan(*body, true, path, checked)?;
                if let Some(s) = after {
                    Self::validate_status(s, true)?;
                }
            }
            ActionPlan::Timed {
                body,
                base_ms,
                per_count_ms,
                count,
                interval_ms,
                ..
            } => {
                if *base_ms == 0
                    || *per_count_ms == 0
                    || *interval_ms == 0
                    || !matches!(
                        self.plans.get(*body).map(|p| &p.op),
                        Some(ActionPlan::Strike { .. })
                    )
                {
                    return Err("invalid timed strike plan".into());
                }
                count.validate(paid, 0)?;
                self.validate_plan(*body, paid, path, checked)?;
            }
        }
        path.pop();
        checked.insert((i, paid));
        Ok(())
    }
}

crate::authoring::clone_shape::copy_shape!(PlanTarget);
crate::authoring::clone_shape::copy_shape!(PlanStatusKind);
crate::authoring::clone_shape::copy_shape!(PlanRepeatTiming);
crate::authoring::clone_shape::struct_shape!(PlanStatus {
    definition,
    kind,
    duration_ms,
    count,
    percent
});
crate::authoring::clone_shape::struct_shape!(PlanRange {
    definition,
    pattern
});
crate::authoring::clone_shape::struct_shape!(NamedActionPlan { definition, op });
crate::authoring::clone_shape::struct_shape!(RelationActionEntry {
    definition,
    plan,
    next_basic,
    interrupt_on_silence,
    protected,
    selector,
    self_damage,
    manual_stop
});
crate::authoring::clone_shape::struct_shape!(RelationActionModifier {
    id,
    relic_definition,
    change
});
crate::authoring::clone_shape::struct_shape!(RelationActionDef {
    definition,
    stock_definition,
    relation_definition,
    mode_definition,
    menu_entry,
    admission,
    selection_range,
    initial_stock,
    stock_capacity,
    period_ms,
    periodic_gain,
    reduction,
    shield_threshold,
    shield_percent,
    shield_duration_ms,
    mode_damage_percent,
    normal,
    active,
    plans,
    modifiers
});
impl crate::authoring::clone_shape::CloneShape for ActionCount {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        m.scan_storage(Some(1))?;
        match self {
            Self::Quotient { count, .. } => count.visit_clone_shape(m),
            Self::Sum { first, second } => {
                first.visit_clone_shape(m)?;
                second.visit_clone_shape(m)
            }
            _ => Ok(()),
        }
    }
}
impl crate::authoring::clone_shape::CloneShape for ActionPlan {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        m.scan_storage(Some(1))?;
        match self {
            Self::Strike { range, after, .. } => {
                range.visit_clone_shape(m)?;
                after.visit_clone_shape(m)
            }
            Self::Repeat { count, .. } | Self::Timed { count, .. } => count.visit_clone_shape(m),
            Self::Spend { amount, after, .. } => {
                amount.visit_clone_shape(m)?;
                after.visit_clone_shape(m)
            }
            _ => Ok(()),
        }
    }
}
impl crate::authoring::clone_shape::CloneShape for RelationActionChange {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        m.scan_storage(Some(1))?;
        match self {
            Self::Range {
                definition,
                pattern,
            } => {
                definition.visit_clone_shape(m)?;
                pattern.visit_clone_shape(m)
            }
            Self::Period { definition, .. }
            | Self::StatusCount { definition, .. }
            | Self::PrimaryArea { definition }
            | Self::ControlVulnerability { definition, .. } => definition.visit_clone_shape(m),
            Self::Multihit { entry, .. } | Self::Retarget { entry, .. } => {
                entry.visit_clone_shape(m)
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationActionsSnapshot {
    pub owners: Vec<RelationOwnerSnapshot>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationOwnerSnapshot {
    pub source_copy: uuid::Uuid,
    pub caster: crate::game::battle::ids::UnitInstanceId,
    pub stock: u32,
    pub capacity: Option<u32>,
    pub active_mode: bool,
    pub mode_reserved: bool,
    pub shield_used: bool,
    pub target: Option<crate::game::battle::ids::UnitInstanceId>,
    pub generation: Option<u64>,
    pub next_gain_ms: Option<u64>,
    pub menu_entry: String,
    pub menu_available: bool,
    pub eligible_targets: Vec<crate::game::battle::ids::UnitInstanceId>,
    pub pending: bool,
    pub active: Option<RelationExecutionSnapshot>,
    pub applied_modifiers: Vec<super::marks::AppliedMarkModifier>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationExecutionSnapshot {
    pub activation: u64,
    pub entry: String,
    pub waiting_basic: bool,
    pub primary: Option<crate::game::battle::ids::UnitInstanceId>,
    pub start_stock: u32,
    pub next_tick_ms: Option<u64>,
    pub next_self_damage_ms: Option<u64>,
    pub remaining_plans: Vec<String>,
    pub target_hits: Vec<(crate::game::battle::ids::UnitInstanceId, u32)>,
    pub target_ramps: Vec<(crate::game::battle::ids::UnitInstanceId, MarkRatio)>,
}
