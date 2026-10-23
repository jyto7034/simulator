//! Typed, finite amount-expression kernel.
//!
//! The kernel is intentionally independent from authored RON and battle state.
//! A later integration layer must use bounded deserialization and construct an
//! [`EvalFrame`] from capabilities that the runtime already owns.

mod catalog;
mod compile;
mod eval;
mod parse;

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::game::ability::authoring::EffectSite;
use crate::game::resource::{ResourceId, ResourceOwnerSelector};
use crate::game::resources::item_slot::EquipmentSlotAddress;

pub(crate) use catalog::can_evaluate_without_external_context;
pub(crate) use catalog::delivery_capabilities;
pub use catalog::{AmountPlanKind, CompiledAmountEntry, ValueExprCatalog};
pub use compile::{compile, CompiledAmountPlan, CompiledOp};
pub use eval::{evaluate, narrow_damage, narrow_heal};
pub(crate) use parse::{current_parse_limits, with_parse_limits};
pub use parse::{validate_expression_input, ExprInputError, ExprParseLimits};

pub const VALUE_EXPR_COMPILER_VERSION: u32 = 1;
pub(crate) const VALUE_EXPR_TEXT_WORK_GRANULE: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EvalMoment {
    Snapshot,
    Live,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum AmountProvenance {
    AuthoredExpression,
    LegacyDamageLiteral,
    LegacyHealLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AmountExpr {
    pub moment: EvalMoment,
    pub root: ValueExpr,
    #[serde(skip)]
    pub provenance: AmountProvenance,
}

impl AmountExpr {
    pub fn authored(moment: EvalMoment, root: ValueExpr) -> Self {
        Self {
            moment,
            root,
            provenance: AmountProvenance::AuthoredExpression,
        }
    }

    pub fn legacy_damage(amount: i32) -> Self {
        Self {
            moment: EvalMoment::Snapshot,
            root: ValueExpr::Const(TypedInteger::new(
                i128::from(amount).unsigned_abs() as i128,
                ValueUnit::Magnitude,
            )),
            provenance: AmountProvenance::LegacyDamageLiteral,
        }
    }

    pub fn legacy_heal(amount: i32) -> Self {
        Self {
            moment: EvalMoment::Snapshot,
            root: ValueExpr::Const(TypedInteger::new(i128::from(amount), ValueUnit::Magnitude)),
            provenance: AmountProvenance::LegacyHealLiteral,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ValueExpr {
    Const(TypedInteger),
    Read(ValueRef),
    Add(Vec<ValueExpr>),
    Mul {
        value: Box<ValueExpr>,
        scalar: Box<ValueExpr>,
    },
    Ratio {
        numerator: Box<ValueExpr>,
        denominator: Box<ValueExpr>,
        rounding: Rounding,
    },
    Clamp {
        value: Box<ValueExpr>,
        min: Box<ValueExpr>,
        max: Box<ValueExpr>,
    },
    Aggregate {
        source: FiniteSource,
        grouping: Grouping,
        reduction: Reduction,
        outer_reduction: Option<Reduction>,
        member: Box<ValueExpr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Rounding {
    TowardZero,
    Floor,
    Ceil,
    NearestTiesEven,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ValueUnit {
    Magnitude,
    Scalar,
    Count,
    Milliseconds,
    AttackSpeedPoints,
    Resource(ResourceId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TypedInteger {
    #[serde(serialize_with = "parse::serialize_integer")]
    pub value: i128,
    pub unit: ValueUnit,
}

impl TypedInteger {
    pub fn new(value: i128, unit: ValueUnit) -> Self {
        Self { value, unit }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ValueSubject {
    Source,
    Target,
    AggregateMember,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StatSelector {
    Attack,
    SpellPower,
    AttackSpeedPoints,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HitPointField {
    Current,
    Max,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ResourceValueField {
    Balance,
    Reserved,
    Available,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TimeValue {
    BattleElapsed,
    RemainingTime,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ResultSelector {
    EarlierEffect(String),
    CurrentActionPrefix,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ResultField {
    DamageHpLost,
    HealingDelta,
    ResourceChangeDelta(ResourceId),
    DamageStrikes,
    DamagingStrikes,
    Count,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ValueRef {
    Stat {
        subject: ValueSubject,
        stat: StatSelector,
    },
    HitPoints {
        subject: ValueSubject,
        field: HitPointField,
    },
    Resource {
        resource: ResourceId,
        owner: ResourceOwnerSelector,
        field: ResourceValueField,
    },
    Time(TimeValue),
    Result {
        selector: ResultSelector,
        field: ResultField,
    },
    AggregateMember {
        unit: ValueUnit,
    },
}

impl ValueRef {
    pub fn unit(&self) -> ValueUnit {
        match self {
            Self::Stat {
                stat: StatSelector::Attack | StatSelector::SpellPower,
                ..
            }
            | Self::HitPoints { .. } => ValueUnit::Magnitude,
            Self::Stat {
                stat: StatSelector::AttackSpeedPoints,
                ..
            } => ValueUnit::AttackSpeedPoints,
            Self::Resource { resource, .. } => ValueUnit::Resource(resource.clone()),
            Self::Time(_) => ValueUnit::Milliseconds,
            Self::Result { field, .. } => match field {
                ResultField::DamageHpLost | ResultField::HealingDelta => ValueUnit::Magnitude,
                ResultField::ResourceChangeDelta(resource) => ValueUnit::Resource(resource.clone()),
                ResultField::DamageStrikes | ResultField::DamagingStrikes | ResultField::Count => {
                    ValueUnit::Count
                }
            },
            Self::AggregateMember { unit } => unit.clone(),
        }
    }

    pub fn is_member_scoped(&self) -> bool {
        matches!(
            self,
            Self::Stat {
                subject: ValueSubject::AggregateMember,
                ..
            } | Self::HitPoints {
                subject: ValueSubject::AggregateMember,
                ..
            } | Self::AggregateMember { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredMember {
    pub identity: String,
    pub value: TypedInteger,
    pub owner: Option<GroupOwner>,
    pub projected_tile: Option<ProjectedTile>,
}

impl AuthoredMember {
    pub fn scalar(identity: impl Into<String>, value: TypedInteger) -> Self {
        Self {
            identity: identity.into(),
            value,
            owner: None,
            projected_tile: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum FiniteSource {
    AuthoredMembers(Vec<AuthoredMember>),
    ResolvedTargets,
    ResolvedTargetsWhere(Vec<ResolvedTargetPredicate>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Grouping {
    WholeSet,
    ByOwner,
    ByProjectedTile,
    ByResolvedOwner(ResolvedOwnerRelation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ResolvedOwnerRelation {
    Side,
    ActualEmployee,
    EquippedSlot(EquipmentSlotAddress),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ResolvedTargetPredicate {
    IsActualEmployee,
    HasEquippedSlot(EquipmentSlotAddress),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GroupOwnerDomain {
    Side,
    Employee,
    EquipmentCopy,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupOwner {
    pub domain: GroupOwnerDomain,
    pub identity: String,
}

impl GroupOwner {
    pub fn new(
        domain: GroupOwnerDomain,
        identity: impl Into<String>,
    ) -> Result<Self, &'static str> {
        let identity = identity.into();
        if identity.trim().is_empty() {
            return Err("group owner identity must not be empty");
        }
        Ok(Self { domain, identity })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Reduction {
    Count,
    Sum,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectedTile {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExprLimits {
    /// Maximum authored expression depth admitted by the compiler. Evaluation
    /// compares a smaller meter limit with the depth recorded in the plan.
    pub max_depth: usize,
    /// Maximum authored expression nodes admitted by the compiler. Evaluation
    /// compares a smaller meter limit with the node count recorded in the plan.
    pub max_nodes: usize,
    /// Maximum simultaneously live value slots across nested evaluation segments.
    pub max_stack: usize,
    /// Maximum canonical members evaluated after duplicate identity removal.
    /// Raw member inspection is charged separately to `max_work`.
    pub max_dynamic_members: usize,
    pub max_groups: usize,
    /// Evaluation, including raw member preparation, uses this ceiling.
    pub max_work: u64,
    /// Compiler traversal, payload inspection, normalization, and fingerprint
    /// generation use this separate ceiling. This keeps `max_work`'s existing
    /// evaluation meaning while ensuring rejected authored input is bounded.
    pub max_compile_work: u64,
    /// Conservative ceiling for storage newly retained or reserved by the
    /// compiler/evaluator. Memory already owned by an input AST/frame is excluded.
    pub max_retained_bytes: usize,
}

impl Default for ExprLimits {
    fn default() -> Self {
        crate::runtime_safety::RuntimeSafetySettings::embedded().expression()
    }
}

impl ExprLimits {
    pub fn validate(self) -> Result<Self, &'static str> {
        if self.max_depth == 0
            || self.max_nodes == 0
            || self.max_stack == 0
            || self.max_dynamic_members == 0
            || self.max_groups == 0
            || self.max_work == 0
            || self.max_compile_work == 0
            || self.max_retained_bytes == 0
        {
            return Err("all expression limits must be non-zero");
        }
        Ok(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeliveryCapabilities {
    pub snapshot: bool,
    pub live: bool,
    pub target_at_release: bool,
    pub target_at_consume: bool,
    pub resolved_targets_at_release: bool,
    pub resolved_targets_at_consume: bool,
    pub resources: bool,
    pub result_prefix: bool,
    pub deadline: bool,
    pub owner_grouping: bool,
    pub projected_tile_grouping: bool,
}

impl DeliveryCapabilities {
    pub const fn constant_only() -> Self {
        Self {
            snapshot: true,
            live: true,
            target_at_release: false,
            target_at_consume: false,
            resolved_targets_at_release: false,
            resolved_targets_at_consume: false,
            resources: false,
            result_prefix: false,
            deadline: false,
            owner_grouping: false,
            projected_tile_grouping: false,
        }
    }

    #[cfg(test)]
    pub(crate) const fn all() -> Self {
        Self {
            snapshot: true,
            live: true,
            target_at_release: true,
            target_at_consume: true,
            resolved_targets_at_release: true,
            resolved_targets_at_consume: true,
            resources: true,
            result_prefix: true,
            deadline: true,
            owner_grouping: true,
            projected_tile_grouping: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextRequirements {
    pub references: Vec<ValueRef>,
    pub resolved_targets: bool,
    pub owner_grouping: bool,
    pub projected_tile_grouping: bool,
    pub resolved_owner_relations: Vec<ResolvedOwnerRelation>,
    pub resolved_target_predicates: Vec<ResolvedTargetPredicate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PlanCost {
    pub nodes: usize,
    pub max_depth: usize,
    pub max_stack: usize,
    pub stack_storage_slots: usize,
    pub max_task_frames: usize,
    /// Work determined from the compiled operations and authored aggregate
    /// inputs. This is a complete upper bound when `runtime_dynamic_work` is
    /// false; otherwise it excludes frame/resolved payload that is unavailable
    /// to the compiler and remains protected by the runtime meter.
    pub static_work: u64,
    /// True when frame values or resolved members can add work that cannot be
    /// bounded from the authored expression alone.
    pub runtime_dynamic_work: bool,
    /// Complete admission ceiling. For a fully static plan this equals
    /// `static_work`; for a runtime-dynamic plan this is the admitted
    /// `max_work` ceiling rather than a mislabeled structural minimum.
    pub worst_case_work: u64,
    pub compile_work: u64,
    pub compile_reserved_bytes: usize,
    pub plan_retained_bytes: usize,
    pub authored_members: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueExprFaultCode {
    UnitMismatch,
    MissingRounding,
    DivideByZero,
    Overflow,
    InvalidClamp,
    MissingContext,
    UnsupportedMoment,
    InvalidOwner,
    StaleLease,
    DifferentAttempt,
    NotSettled,
    FutureResult,
    Unavailable,
    BudgetExceeded,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueExprPhase {
    Compile,
    Evaluate,
    Narrow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExprDiagnostic {
    pub code: ValueExprFaultCode,
    pub site: EffectSite,
    pub node_index: usize,
    pub expected_unit: Option<ValueUnit>,
    pub actual_unit: Option<ValueUnit>,
    pub phase: ValueExprPhase,
    pub cause: String,
}

impl fmt::Display for ExprDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "value expression compile error {:?} at {}/{}/{} node {}: {}",
            self.code,
            self.site.action.definition_id,
            self.site.action.action_id,
            self.site.effect_id,
            self.node_index,
            self.cause
        )
    }
}

impl std::error::Error for ExprDiagnostic {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValueExprFault {
    pub code: ValueExprFaultCode,
    pub site: EffectSite,
    pub node_index: usize,
    pub expected_unit: Option<ValueUnit>,
    pub actual_unit: Option<ValueUnit>,
    pub phase: ValueExprPhase,
    pub cause: String,
}

impl fmt::Display for ValueExprFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "value expression fault {:?} at {}/{}/{} node {}: {}",
            self.code,
            self.site.action.definition_id,
            self.site.action.action_id,
            self.site.effect_id,
            self.node_index,
            self.cause
        )
    }
}

impl std::error::Error for ValueExprFault {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalMember {
    pub identity: String,
    /// Authored scalar only. A resolved unit has no implicit scalar producer.
    pub value: Option<TypedInteger>,
    pub owner: Option<GroupOwner>,
    pub projected_tile: Option<ProjectedTile>,
    pub values: BTreeMap<ValueRef, TypedInteger>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedResolvedTargetView {
    pub node_index: usize,
    pub members: Arc<Vec<EvalMember>>,
}

impl From<AuthoredMember> for EvalMember {
    fn from(member: AuthoredMember) -> Self {
        Self {
            identity: member.identity,
            value: Some(member.value),
            owner: member.owner,
            projected_tile: member.projected_tile,
            values: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EvalFrame {
    // Sorted for deterministic iteration and logarithmic repeated reads. The
    // battle adapter reserves its full known cardinality before inserting.
    values: Vec<(ValueRef, TypedInteger)>,
    resolved_targets: Option<Arc<Vec<EvalMember>>>,
    resolved_target_views: Option<Arc<Vec<PreparedResolvedTargetView>>>,
}

impl EvalFrame {
    pub(crate) fn with_value_capacity(count: usize) -> Self {
        Self {
            values: Vec::with_capacity(count),
            resolved_targets: None,
            resolved_target_views: None,
        }
    }

    pub fn insert(&mut self, reference: ValueRef, value: TypedInteger) -> Result<(), &'static str> {
        if reference.unit() != value.unit {
            return Err("frame value unit does not match its reference");
        }
        match self.values.binary_search_by(|(key, _)| key.cmp(&reference)) {
            Ok(index) => self.values[index].1 = value,
            Err(index) => self.values.insert(index, (reference, value)),
        }
        Ok(())
    }

    pub(crate) fn insert_bounded(
        &mut self,
        reference: ValueRef,
        value: TypedInteger,
        limit: usize,
    ) -> Result<(), &'static str> {
        if self.values.len() >= limit
            && self
                .values
                .binary_search_by(|(key, _)| key.cmp(&reference))
                .is_err()
        {
            return Err("frame value capacity exhausted");
        }
        self.insert(reference, value)
    }

    pub fn set_resolved_targets(&mut self, members: Vec<EvalMember>) {
        self.resolved_targets = Some(Arc::new(members));
    }

    pub(crate) fn set_resolved_targets_shared(&mut self, members: Arc<Vec<EvalMember>>) {
        self.resolved_targets = Some(members);
    }

    pub fn set_resolved_target_views(&mut self, mut views: Vec<PreparedResolvedTargetView>) {
        views.sort_by_key(|view| view.node_index);
        self.resolved_target_views = Some(Arc::new(views));
    }

    pub(crate) fn set_resolved_target_views_shared(
        &mut self,
        views: Arc<Vec<PreparedResolvedTargetView>>,
    ) {
        self.resolved_target_views = Some(views);
    }

    pub(crate) fn get(&self, reference: &ValueRef) -> Option<&TypedInteger> {
        self.values
            .binary_search_by(|(key, _)| key.cmp(reference))
            .ok()
            .map(|index| &self.values[index].1)
    }

    pub(crate) fn contains_key(&self, reference: &ValueRef) -> bool {
        self.get(reference).is_some()
    }

    pub(crate) fn resolved_targets(&self) -> &[EvalMember] {
        self.resolved_targets.as_deref().map_or(&[], Vec::as_slice)
    }

    pub(crate) fn has_resolved_targets(&self) -> bool {
        self.resolved_targets.is_some()
    }

    pub(crate) fn resolved_target_view(&self, node_index: usize) -> Option<&[EvalMember]> {
        let views = self.resolved_target_views.as_deref()?;
        views
            .binary_search_by_key(&node_index, |view| view.node_index)
            .ok()
            .map(|index| views[index].members.as_slice())
    }

    pub(crate) fn has_resolved_target_views(&self) -> bool {
        self.resolved_target_views.is_some()
    }

    /// Conservative bytes retained by an immutable recorded frame. Shared
    /// member views are charged per owner so a surviving record never counts
    /// only its Arc handles.
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        fn member_extent(member: &EvalMember) -> Option<usize> {
            let mut bytes = std::mem::size_of::<EvalMember>()
                .checked_add(member.identity.len())?
                .checked_add(member.values.len().checked_mul(
                    std::mem::size_of::<(ValueRef, TypedInteger)>().checked_add(64)?,
                )?)?;
            if let Some(value) = &member.value {
                bytes = bytes.checked_add(typed_integer_dynamic_bytes(value))?;
            }
            for (reference, value) in &member.values {
                bytes = bytes
                    .checked_add(value_ref_dynamic_bytes(reference)?)?
                    .checked_add(typed_integer_dynamic_bytes(value))?;
            }
            Some(bytes)
        }

        let mut bytes = std::mem::size_of::<Self>().checked_add(
            self.values
                .capacity()
                .checked_mul(std::mem::size_of::<(ValueRef, TypedInteger)>())?,
        )?;
        for (reference, value) in &self.values {
            bytes = bytes
                .checked_add(value_ref_dynamic_bytes(reference)?)?
                .checked_add(typed_integer_dynamic_bytes(value))?;
        }
        if let Some(members) = &self.resolved_targets {
            bytes = bytes
                .checked_add(std::mem::size_of::<Vec<EvalMember>>())?
                .checked_add(
                    members
                        .capacity()
                        .checked_mul(std::mem::size_of::<EvalMember>())?,
                )?;
            for member in members.iter() {
                bytes = bytes.checked_add(member_extent(member)?)?;
            }
        }
        if let Some(views) = &self.resolved_target_views {
            bytes = bytes
                .checked_add(std::mem::size_of::<Vec<PreparedResolvedTargetView>>())?
                .checked_add(
                    views
                        .capacity()
                        .checked_mul(std::mem::size_of::<PreparedResolvedTargetView>())?,
                )?;
            for view in views.iter() {
                bytes = bytes
                    .checked_add(std::mem::size_of::<Vec<EvalMember>>())?
                    .checked_add(
                        view.members
                            .capacity()
                            .checked_mul(std::mem::size_of::<EvalMember>())?,
                    )?;
                for member in view.members.iter() {
                    bytes = bytes.checked_add(member_extent(member)?)?;
                }
            }
        }
        Some(bytes)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct ExprUsage {
    pub work: u64,
    pub raw_members: usize,
    pub dynamic_members: usize,
    pub groups: usize,
    pub retained_bytes: usize,
    pub peak_stack_values: usize,
    pub peak_task_frames: usize,
}

#[derive(Debug, Clone)]
pub struct ExprMeter {
    limits: ExprLimits,
    usage: ExprUsage,
    cancelled: bool,
    host_admission:
        Option<fn(&CompiledAmountPlan, usize, u64, usize) -> Result<(), ValueExprFault>>,
}

impl ExprMeter {
    pub fn new(limits: ExprLimits) -> Result<Self, &'static str> {
        Ok(Self {
            limits: limits.validate()?,
            usage: ExprUsage::default(),
            cancelled: false,
            host_admission: None,
        })
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    pub fn usage(&self) -> ExprUsage {
        self.usage
    }

    /// The BattleCore adapter attaches the live operation's admission owner.
    /// Standalone compiler/evaluator users retain the local meter contract.
    pub(crate) fn with_host_admission(
        mut self,
        admission: fn(&CompiledAmountPlan, usize, u64, usize) -> Result<(), ValueExprFault>,
    ) -> Self {
        self.host_admission = Some(admission);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TraceEntry {
    pub node_index: usize,
    pub member_identity: Option<String>,
    pub value: TypedInteger,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EvaluatedAmount {
    pub value: TypedInteger,
    pub trace: Vec<TraceEntry>,
}

pub(crate) fn text_work_units(bytes: usize) -> Option<u64> {
    let chunks =
        bytes.checked_add(VALUE_EXPR_TEXT_WORK_GRANULE - 1)? / VALUE_EXPR_TEXT_WORK_GRANULE;
    u64::try_from(chunks.max(1)).ok()
}

pub(crate) fn aggregate_member_scan_work(
    identity_bytes: usize,
    primary_value_dynamic_bytes: usize,
    owner_identity_bytes: Option<usize>,
    associated_value_count: usize,
    associated_value_payload_work: u64,
) -> Option<u64> {
    let mut work = text_work_units(identity_bytes)?
        .checked_add(u64::try_from(associated_value_count).ok()?)?
        .checked_add(text_work_units(primary_value_dynamic_bytes)?)?
        .checked_add(associated_value_payload_work)?;
    if let Some(owner_bytes) = owner_identity_bytes {
        work = work.checked_add(text_work_units(owner_bytes)?)?;
    }
    Some(work)
}

pub(crate) fn aggregate_sort_work(
    count: usize,
    identity_bytes: usize,
    owner_key_bytes: Option<usize>,
) -> Option<u64> {
    let levels = if count <= 1 {
        0
    } else {
        usize::BITS - (count - 1).leading_zeros()
    };
    let levels_usize = usize::try_from(levels).ok()?;
    let comparison_count = count
        .checked_mul(levels_usize)?
        .checked_add(count.checked_mul(2)?)?
        .checked_mul(2)?;
    let comparison_work = text_work_units(comparison_count)?;
    let levels_work = u64::from(levels);
    let mut work = text_work_units(identity_bytes)?
        .checked_mul(levels_work)?
        .checked_add(comparison_work)?;
    if let Some(owner_bytes) = owner_key_bytes {
        work = work.checked_add(
            text_work_units(owner_bytes)?
                .checked_mul(levels_work)?
                .checked_add(comparison_work)?,
        )?;
    }
    Some(work)
}

pub(crate) fn value_unit_dynamic_bytes(unit: &ValueUnit) -> usize {
    match unit {
        ValueUnit::Resource(resource) => resource.as_str().len(),
        _ => 0,
    }
}

pub(crate) fn typed_integer_dynamic_bytes(value: &TypedInteger) -> usize {
    value_unit_dynamic_bytes(&value.unit)
}

pub(crate) fn value_ref_dynamic_bytes(reference: &ValueRef) -> Option<usize> {
    let mut bytes = 0_usize;
    match reference {
        ValueRef::Resource { resource, .. } => {
            bytes = bytes.checked_add(resource.as_str().len())?;
        }
        ValueRef::Result { selector, field } => {
            if let ResultSelector::EarlierEffect(effect_id) = selector {
                bytes = bytes.checked_add(effect_id.len())?;
            }
            if let ResultField::ResourceChangeDelta(resource) = field {
                bytes = bytes.checked_add(resource.as_str().len())?;
            }
        }
        ValueRef::AggregateMember { unit } => {
            bytes = bytes.checked_add(value_unit_dynamic_bytes(unit))?;
        }
        ValueRef::Stat { .. } | ValueRef::HitPoints { .. } | ValueRef::Time(_) => {}
    }
    Some(bytes)
}

#[cfg(test)]
mod tests;
