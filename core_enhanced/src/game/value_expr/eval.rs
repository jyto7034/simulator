use std::collections::BTreeMap;
use std::mem::size_of;

use crate::game::ability::authoring::EffectSite;

use super::compile::{bounded_effect_site, bounded_value_unit, divide_rounded};
use super::{
    aggregate_member_scan_work, aggregate_sort_work, text_work_units, typed_integer_dynamic_bytes,
    value_ref_dynamic_bytes, value_unit_dynamic_bytes, AuthoredMember, CompiledAmountPlan,
    CompiledOp, EvalFrame, EvalMember, EvaluatedAmount, ExprMeter, FiniteSource, GroupOwner,
    GroupOwnerDomain, Grouping, ProjectedTile, Reduction, ResolvedOwnerRelation, TraceEntry,
    TypedInteger, ValueExprFault, ValueExprFaultCode, ValueExprPhase, ValueRef, ValueUnit,
};

#[derive(Clone, Copy)]
enum MemberRef<'a> {
    Authored(&'a AuthoredMember),
    Resolved(&'a EvalMember),
}

enum MemberSlice<'a> {
    Authored(&'a [AuthoredMember]),
    Resolved(&'a [EvalMember]),
}

impl<'a> MemberSlice<'a> {
    fn len(&self) -> usize {
        match self {
            Self::Authored(members) => members.len(),
            Self::Resolved(members) => members.len(),
        }
    }

    fn member(&self, index: usize) -> MemberRef<'a> {
        match self {
            Self::Authored(members) => MemberRef::Authored(&members[index]),
            Self::Resolved(members) => MemberRef::Resolved(&members[index]),
        }
    }

    fn extend_refs(&self, output: &mut Vec<MemberRef<'a>>) {
        match self {
            Self::Authored(members) => output.extend(members.iter().map(MemberRef::Authored)),
            Self::Resolved(members) => output.extend(members.iter().map(MemberRef::Resolved)),
        }
    }
}

impl<'a> MemberRef<'a> {
    fn identity(self) -> &'a str {
        match self {
            Self::Authored(member) => &member.identity,
            Self::Resolved(member) => &member.identity,
        }
    }

    fn value(self) -> Option<&'a TypedInteger> {
        match self {
            Self::Authored(member) => Some(&member.value),
            Self::Resolved(member) => member.value.as_ref(),
        }
    }

    fn owner(self) -> Option<&'a GroupOwner> {
        match self {
            Self::Authored(member) => member.owner.as_ref(),
            Self::Resolved(member) => member.owner.as_ref(),
        }
    }

    fn projected_tile(self) -> Option<ProjectedTile> {
        match self {
            Self::Authored(member) => member.projected_tile,
            Self::Resolved(member) => member.projected_tile,
        }
    }

    fn values(self) -> Option<&'a BTreeMap<ValueRef, TypedInteger>> {
        match self {
            Self::Authored(_) => None,
            Self::Resolved(member) => Some(&member.values),
        }
    }
}

struct SegmentTask<'a> {
    segment: usize,
    pc: usize,
    values: Vec<TypedInteger>,
    member: Option<MemberRef<'a>>,
}

struct AggregateTask<'a> {
    members: Vec<MemberRef<'a>>,
    next_member: usize,
    awaiting_member: bool,
    member_segment: usize,
    grouping: Grouping,
    reduction: Reduction,
    outer_reduction: Option<Reduction>,
    member_unit: ValueUnit,
    result_unit: ValueUnit,
    node_index: usize,
    groups: BTreeMap<GroupKey<'a>, Vec<TypedInteger>>,
}

enum Task<'a> {
    Segment(SegmentTask<'a>),
    Aggregate(AggregateTask<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum GroupKey<'a> {
    WholeSet,
    Owner(&'a GroupOwner),
    ProjectedTile(ProjectedTile),
}

#[derive(Default)]
struct RuntimeState {
    stack_values: usize,
    task_frames: usize,
}

pub fn evaluate(
    plan: &CompiledAmountPlan,
    frame: &EvalFrame,
    meter: &mut ExprMeter,
) -> Result<EvaluatedAmount, ValueExprFault> {
    poll_cancelled(plan, meter, 0)?;
    if meter.limits.max_depth > plan.limits.max_depth
        || meter.limits.max_nodes > plan.limits.max_nodes
        || meter.limits.max_stack > plan.limits.max_stack
        || meter.limits.max_dynamic_members > plan.limits.max_dynamic_members
        || meter.limits.max_groups > plan.limits.max_groups
        || meter.limits.max_work > plan.limits.max_work
        || meter.limits.max_retained_bytes > plan.limits.max_retained_bytes
    {
        return Err(fault(
            plan,
            ValueExprFaultCode::BudgetExceeded,
            0,
            None,
            None,
            "evaluation meter cannot exceed the plan admission limits",
            ValueExprPhase::Evaluate,
        ));
    }

    let cost = plan.cost();
    for (required, limit, label) in [
        (
            cost.nodes,
            meter.limits.max_nodes,
            "compiled node requirement",
        ),
        (
            cost.max_depth,
            meter.limits.max_depth,
            "compiled depth requirement",
        ),
        (
            cost.max_stack,
            meter.limits.max_stack,
            "simultaneous value-stack requirement",
        ),
    ] {
        if required > limit {
            return Err(fault(
                plan,
                ValueExprFaultCode::BudgetExceeded,
                0,
                None,
                None,
                format!("{label} {required} exceeds evaluation limit {limit}"),
                ValueExprPhase::Evaluate,
            ));
        }
    }

    admit_frame_values(plan, frame, meter)?;

    let task_storage = cost
        .max_task_frames
        .checked_mul(size_of::<Task<'_>>())
        .and_then(|bytes| bytes.checked_mul(2))
        .ok_or_else(|| accounting_overflow(plan, 0, "task-frame storage"))?;
    let stack_storage = cost
        .stack_storage_slots
        .checked_mul(size_of::<TypedInteger>())
        .and_then(|bytes| bytes.checked_mul(4))
        .ok_or_else(|| accounting_overflow(plan, 0, "value-stack storage"))?;
    charge_retained(plan, meter, 0, task_storage)?;
    charge_retained(plan, meter, 0, stack_storage)?;

    let mut runtime = RuntimeState {
        stack_values: 0,
        task_frames: 1,
    };
    meter.usage.peak_task_frames = meter.usage.peak_task_frames.max(1);
    let mut tasks = Vec::with_capacity(cost.max_task_frames);
    tasks.push(Task::Segment(SegmentTask {
        segment: 0,
        pc: 0,
        values: Vec::new(),
        member: None,
    }));
    let mut returned: Option<TypedInteger> = None;
    let mut trace = Vec::new();

    loop {
        let Some(task) = tasks.last_mut() else {
            let value = returned.ok_or_else(|| {
                fault(
                    plan,
                    ValueExprFaultCode::Unavailable,
                    0,
                    None,
                    None,
                    "evaluator ended without a root value",
                    ValueExprPhase::Evaluate,
                )
            })?;
            if value.unit != *plan.result_unit() {
                return Err(fault(
                    plan,
                    ValueExprFaultCode::UnitMismatch,
                    0,
                    Some(plan.result_unit().clone()),
                    Some(value.unit),
                    "evaluated root unit differs from the compiled result unit",
                    ValueExprPhase::Evaluate,
                ));
            }
            return Ok(EvaluatedAmount { value, trace });
        };

        match task {
            Task::Segment(segment) => {
                if let Some(value) = returned.take() {
                    push_stack_value(plan, meter, &mut runtime, &mut segment.values, value, 0)?;
                }
                let ops = plan.segment(segment.segment).ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::Unavailable,
                        0,
                        None,
                        None,
                        "compiled segment is missing",
                        ValueExprPhase::Evaluate,
                    )
                })?;
                if segment.pc == ops.len() {
                    if segment.values.len() != 1 {
                        return Err(fault(
                            plan,
                            ValueExprFaultCode::Unavailable,
                            0,
                            None,
                            None,
                            "compiled segment did not yield exactly one value",
                            ValueExprPhase::Evaluate,
                        ));
                    }
                    returned = segment.values.pop();
                    consume_stack_values(plan, &mut runtime, 1, 0)?;
                    pop_task(plan, &mut runtime, 0)?;
                    tasks.pop();
                    continue;
                }

                let op = &ops[segment.pc];
                charge_work(plan, meter, op.node_index(), 1)?;
                match op {
                    CompiledOp::Aggregate {
                        source,
                        grouping,
                        reduction,
                        outer_reduction,
                        member_segment,
                        member_unit,
                        result_unit,
                        node_index,
                    } => {
                        let aggregate_members =
                            members(plan, frame, meter, source, *grouping, *node_index)?;
                        let unit_payload = value_unit_dynamic_bytes(member_unit)
                            .checked_add(value_unit_dynamic_bytes(result_unit))
                            .and_then(|bytes| bytes.checked_mul(2))
                            .ok_or_else(|| {
                                accounting_overflow(plan, *node_index, "aggregate task payload")
                            })?;
                        charge_retained(plan, meter, *node_index, unit_payload)?;
                        segment.pc += 1;
                        let aggregate = AggregateTask {
                            members: aggregate_members,
                            next_member: 0,
                            awaiting_member: false,
                            member_segment: *member_segment,
                            grouping: *grouping,
                            reduction: *reduction,
                            outer_reduction: *outer_reduction,
                            member_unit: member_unit.clone(),
                            result_unit: result_unit.clone(),
                            node_index: *node_index,
                            groups: BTreeMap::new(),
                        };
                        push_task(plan, meter, &mut runtime, *node_index)?;
                        tasks.push(Task::Aggregate(aggregate));
                    }
                    _ => {
                        let value =
                            execute_op(plan, frame, meter, &mut runtime, op, segment, &mut trace)?;
                        push_stack_value(
                            plan,
                            meter,
                            &mut runtime,
                            &mut segment.values,
                            value,
                            op.node_index(),
                        )?;
                        segment.pc += 1;
                    }
                }
            }
            Task::Aggregate(aggregate) => {
                if aggregate.awaiting_member {
                    let value = returned.take().ok_or_else(|| {
                        fault(
                            plan,
                            ValueExprFaultCode::Unavailable,
                            aggregate.node_index,
                            Some(aggregate.member_unit.clone()),
                            None,
                            "aggregate member segment did not return a value",
                            ValueExprPhase::Evaluate,
                        )
                    })?;
                    if value.unit != aggregate.member_unit {
                        return Err(fault(
                            plan,
                            ValueExprFaultCode::UnitMismatch,
                            aggregate.node_index,
                            Some(aggregate.member_unit.clone()),
                            Some(value.unit),
                            "aggregate member result unit changed after compilation",
                            ValueExprPhase::Evaluate,
                        ));
                    }
                    let member = aggregate.members[aggregate.next_member];
                    let key = group_key(plan, aggregate.grouping, member, aggregate.node_index)?;
                    if !aggregate.groups.contains_key(&key) {
                        charge_group(plan, meter, aggregate.node_index)?;
                        let storage = group_storage_bytes().ok_or_else(|| {
                            accounting_overflow(plan, aggregate.node_index, "group storage")
                        })?;
                        charge_retained(plan, meter, aggregate.node_index, storage)?;
                    }
                    let value_slot = size_of::<TypedInteger>().checked_mul(4).ok_or_else(|| {
                        accounting_overflow(plan, aggregate.node_index, "aggregate value storage")
                    })?;
                    charge_retained(plan, meter, aggregate.node_index, value_slot)?;
                    aggregate.groups.entry(key).or_default().push(value);
                    aggregate.next_member += 1;
                    aggregate.awaiting_member = false;
                }

                if aggregate.next_member < aggregate.members.len() {
                    let member = aggregate.members[aggregate.next_member];
                    let member_segment = aggregate.member_segment;
                    aggregate.awaiting_member = true;
                    push_task(plan, meter, &mut runtime, aggregate.node_index)?;
                    tasks.push(Task::Segment(SegmentTask {
                        segment: member_segment,
                        pc: 0,
                        values: Vec::new(),
                        member: Some(member),
                    }));
                    continue;
                }

                let value = finish_aggregate(plan, meter, aggregate)?;
                push_trace(plan, meter, &mut trace, aggregate.node_index, None, &value)?;
                returned = Some(value);
                pop_task(plan, &mut runtime, aggregate.node_index)?;
                tasks.pop();
            }
        }
    }
}

fn execute_op(
    plan: &CompiledAmountPlan,
    frame: &EvalFrame,
    meter: &mut ExprMeter,
    runtime: &mut RuntimeState,
    op: &CompiledOp,
    segment: &mut SegmentTask<'_>,
    trace: &mut Vec<TraceEntry>,
) -> Result<TypedInteger, ValueExprFault> {
    let (node_index, value) = match op {
        CompiledOp::PushConst { value, node_index } => (
            *node_index,
            clone_stack_value(plan, meter, *node_index, value)?,
        ),
        CompiledOp::Read {
            reference,
            node_index,
        } => {
            let value = read_value(plan, frame, meter, segment.member, reference, *node_index)?;
            (
                *node_index,
                clone_stack_value(plan, meter, *node_index, value)?,
            )
        }
        CompiledOp::Add {
            arity,
            unit,
            node_index,
        } => {
            let start = operand_start(plan, &segment.values, *arity, *node_index)?;
            let mut result = 0_i128;
            for value in &segment.values[start..] {
                require_unit(plan, *node_index, unit, value)?;
                result = result.checked_add(value.value).ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::Overflow,
                        *node_index,
                        Some(unit.clone()),
                        Some(unit.clone()),
                        "Add overflowed i128",
                        ValueExprPhase::Evaluate,
                    )
                })?;
            }
            segment.values.truncate(start);
            consume_stack_values(plan, runtime, *arity, *node_index)?;
            (
                *node_index,
                make_stack_value(plan, meter, *node_index, result, unit)?,
            )
        }
        CompiledOp::Mul { unit, node_index } => {
            let start = operand_start(plan, &segment.values, 2, *node_index)?;
            let values = &segment.values[start..];
            let result = values[0]
                .value
                .checked_mul(values[1].value)
                .ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::Overflow,
                        *node_index,
                        Some(unit.clone()),
                        Some(unit.clone()),
                        "Mul overflowed i128",
                        ValueExprPhase::Evaluate,
                    )
                })?;
            segment.values.truncate(start);
            consume_stack_values(plan, runtime, 2, *node_index)?;
            (
                *node_index,
                make_stack_value(plan, meter, *node_index, result, unit)?,
            )
        }
        CompiledOp::Ratio {
            rounding,
            unit,
            node_index,
        } => {
            let start = operand_start(plan, &segment.values, 2, *node_index)?;
            let values = &segment.values[start..];
            if values[1].value == 0 {
                return Err(fault(
                    plan,
                    ValueExprFaultCode::DivideByZero,
                    *node_index,
                    Some(values[1].unit.clone()),
                    Some(values[1].unit.clone()),
                    "Ratio denominator is zero",
                    ValueExprPhase::Evaluate,
                ));
            }
            let result =
                divide_rounded(values[0].value, values[1].value, *rounding).ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::Overflow,
                        *node_index,
                        Some(unit.clone()),
                        Some(unit.clone()),
                        "Ratio overflowed i128",
                        ValueExprPhase::Evaluate,
                    )
                })?;
            segment.values.truncate(start);
            consume_stack_values(plan, runtime, 2, *node_index)?;
            (
                *node_index,
                make_stack_value(plan, meter, *node_index, result, unit)?,
            )
        }
        CompiledOp::Clamp { unit, node_index } => {
            let start = operand_start(plan, &segment.values, 3, *node_index)?;
            let values = &segment.values[start..];
            for value in values {
                require_unit(plan, *node_index, unit, value)?;
            }
            if values[1].value > values[2].value {
                return Err(fault(
                    plan,
                    ValueExprFaultCode::InvalidClamp,
                    *node_index,
                    Some(unit.clone()),
                    Some(unit.clone()),
                    "Clamp min is greater than max",
                    ValueExprPhase::Evaluate,
                ));
            }
            let result = values[0].value.clamp(values[1].value, values[2].value);
            segment.values.truncate(start);
            consume_stack_values(plan, runtime, 3, *node_index)?;
            (
                *node_index,
                make_stack_value(plan, meter, *node_index, result, unit)?,
            )
        }
        CompiledOp::Aggregate { .. } => unreachable!("aggregate is handled by the task machine"),
    };
    push_trace(
        plan,
        meter,
        trace,
        node_index,
        segment.member.map(MemberRef::identity),
        &value,
    )?;
    Ok(value)
}

fn read_value<'a>(
    plan: &CompiledAmountPlan,
    frame: &'a EvalFrame,
    meter: &mut ExprMeter,
    member: Option<MemberRef<'a>>,
    reference: &ValueRef,
    node_index: usize,
) -> Result<&'a TypedInteger, ValueExprFault> {
    let diagnostic_reserve = value_ref_dynamic_bytes(reference)
        .ok_or_else(|| accounting_overflow(plan, node_index, "read diagnostic payload"))?
        .checked_mul(4)
        .ok_or_else(|| accounting_overflow(plan, node_index, "read diagnostic payload"))?;
    charge_retained(plan, meter, node_index, diagnostic_reserve)?;
    let value = if let ValueRef::AggregateMember { .. } = reference {
        member.and_then(MemberRef::value)
    } else if reference.is_member_scoped() {
        member.and_then(|member| member.values()?.get(reference))
    } else {
        frame.get(reference)
    }
    .ok_or_else(|| {
        fault(
            plan,
            ValueExprFaultCode::MissingContext,
            node_index,
            Some(reference.unit()),
            None,
            "evaluation frame is missing a required value",
            ValueExprPhase::Evaluate,
        )
    })?;
    require_unit(plan, node_index, &reference.unit(), value)?;
    Ok(value)
}

fn members<'a>(
    plan: &'a CompiledAmountPlan,
    frame: &'a EvalFrame,
    meter: &mut ExprMeter,
    source: &'a FiniteSource,
    grouping: Grouping,
    node_index: usize,
) -> Result<Vec<MemberRef<'a>>, ValueExprFault> {
    let selected = match source {
        FiniteSource::AuthoredMembers(members) => MemberSlice::Authored(members),
        FiniteSource::ResolvedTargets | FiniteSource::ResolvedTargetsWhere(_) => {
            if frame.has_resolved_target_views() {
                let members = frame.resolved_target_view(node_index).ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::MissingContext,
                        node_index,
                        None,
                        None,
                        "resolved target view for this aggregate node was not provided",
                        ValueExprPhase::Evaluate,
                    )
                })?;
                MemberSlice::Resolved(members)
            } else if matches!(source, FiniteSource::ResolvedTargets)
                && !matches!(grouping, Grouping::ByResolvedOwner(_))
            {
                if !frame.has_resolved_targets() {
                    return Err(fault(
                        plan,
                        ValueExprFaultCode::MissingContext,
                        node_index,
                        None,
                        None,
                        "resolved target set was not provided for this evaluation moment",
                        ValueExprPhase::Evaluate,
                    ));
                }
                MemberSlice::Resolved(frame.resolved_targets())
            } else {
                return Err(fault(
                    plan,
                    ValueExprFaultCode::MissingContext,
                    node_index,
                    None,
                    None,
                    "this resolved aggregate requires its node-specific target view",
                    ValueExprPhase::Evaluate,
                ));
            }
        }
    };
    let raw_count = selected.len();
    charge_raw_members(plan, meter, node_index, raw_count)?;

    let mut owner_domain: Option<GroupOwnerDomain> = None;
    let mut identity_bytes = 0_usize;
    let mut group_key_bytes = 0_usize;
    for index in 0..raw_count {
        let member = selected.member(index);
        identity_bytes = identity_bytes
            .checked_add(member.identity().len())
            .ok_or_else(|| accounting_overflow(plan, node_index, "member identity"))?;
        let values_len = member.values().map_or(0, BTreeMap::len);
        let fixed_scan_work = aggregate_member_scan_work(
            member.identity().len(),
            member.value().map_or(0, typed_integer_dynamic_bytes),
            member.owner().map(|owner| owner.identity.len()),
            values_len,
            0,
        )
        .ok_or_else(|| accounting_overflow(plan, node_index, "member scan work"))?;
        charge_work(plan, meter, node_index, fixed_scan_work)?;
        if let Some(values) = member.values() {
            for (reference, value) in values {
                let dynamic_bytes = value_ref_dynamic_bytes(reference)
                    .ok_or_else(|| {
                        accounting_overflow(plan, node_index, "member reference payload")
                    })?
                    .checked_add(typed_integer_dynamic_bytes(value))
                    .ok_or_else(|| accounting_overflow(plan, node_index, "member value payload"))?;
                charge_work(
                    plan,
                    meter,
                    node_index,
                    text_work_units(dynamic_bytes).ok_or_else(|| {
                        accounting_overflow(plan, node_index, "member reference scan work")
                    })?,
                )?;
            }
        }
        if member.identity().trim().is_empty() {
            return Err(fault(
                plan,
                ValueExprFaultCode::InvalidOwner,
                node_index,
                None,
                None,
                "aggregate member identity must not be empty",
                ValueExprPhase::Evaluate,
            ));
        }

        match grouping {
            Grouping::WholeSet => {}
            Grouping::ByOwner => {
                let owner = member.owner().ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::InvalidOwner,
                        node_index,
                        None,
                        None,
                        "aggregate member has no owner grouping key",
                        ValueExprPhase::Evaluate,
                    )
                })?;
                if owner.identity.trim().is_empty() {
                    return Err(fault(
                        plan,
                        ValueExprFaultCode::InvalidOwner,
                        node_index,
                        None,
                        None,
                        "aggregate member has an empty owner identity",
                        ValueExprPhase::Evaluate,
                    ));
                }
                if owner_domain.is_some_and(|domain| domain != owner.domain) {
                    return Err(fault(
                        plan,
                        ValueExprFaultCode::InvalidOwner,
                        node_index,
                        None,
                        None,
                        "one aggregate cannot mix owner grouping domains",
                        ValueExprPhase::Evaluate,
                    ));
                }
                owner_domain = Some(owner.domain);
                group_key_bytes = group_key_bytes
                    .checked_add(owner.identity.len())
                    .ok_or_else(|| accounting_overflow(plan, node_index, "owner grouping key"))?;
            }
            Grouping::ByResolvedOwner(relation) => {
                let owner = member.owner().ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::InvalidOwner,
                        node_index,
                        None,
                        None,
                        "resolved aggregate member has no relation key",
                        ValueExprPhase::Evaluate,
                    )
                })?;
                let expected = resolved_relation_domain(relation);
                if owner.domain != expected || owner.identity.trim().is_empty() {
                    return Err(fault(
                        plan,
                        ValueExprFaultCode::InvalidOwner,
                        node_index,
                        None,
                        None,
                        "resolved aggregate member relation key has the wrong domain or an empty identity",
                        ValueExprPhase::Evaluate,
                    ));
                }
                group_key_bytes = group_key_bytes
                    .checked_add(owner.identity.len())
                    .ok_or_else(|| accounting_overflow(plan, node_index, "owner grouping key"))?;
            }
            Grouping::ByProjectedTile if member.projected_tile().is_none() => {
                return Err(fault(
                    plan,
                    ValueExprFaultCode::MissingContext,
                    node_index,
                    None,
                    None,
                    "aggregate member has no projected-tile grouping key",
                    ValueExprPhase::Evaluate,
                ));
            }
            Grouping::ByProjectedTile => {}
        }
    }

    let sort_work = aggregate_sort_work(
        raw_count,
        identity_bytes,
        grouping_uses_owner_key(grouping).then_some(group_key_bytes),
    )
    .ok_or_else(|| accounting_overflow(plan, node_index, "member sort work"))?;
    charge_work(plan, meter, node_index, sort_work)?;

    let index_storage = raw_count
        .checked_mul(size_of::<MemberRef<'_>>())
        .and_then(|bytes| bytes.checked_mul(2))
        .ok_or_else(|| accounting_overflow(plan, node_index, "member index storage"))?;
    charge_retained(plan, meter, node_index, index_storage)?;
    let mut canonical = Vec::with_capacity(raw_count);
    selected.extend_refs(&mut canonical);
    canonical.sort_by(|left, right| left.identity().cmp(right.identity()));
    for pair in canonical.windows(2) {
        if pair[0].identity() == pair[1].identity() && !same_member(pair[0], pair[1]) {
            return Err(fault(
                plan,
                ValueExprFaultCode::InvalidOwner,
                node_index,
                None,
                None,
                "duplicate member identity has conflicting values",
                ValueExprPhase::Evaluate,
            ));
        }
    }
    canonical.dedup_by(|right, left| left.identity() == right.identity());
    charge_members(plan, meter, node_index, canonical.len())?;
    Ok(canonical)
}

fn same_member(left: MemberRef<'_>, right: MemberRef<'_>) -> bool {
    left.identity() == right.identity()
        && left.value() == right.value()
        && left.owner() == right.owner()
        && left.projected_tile() == right.projected_tile()
        && match (left.values(), right.values()) {
            (None, None) => true,
            (Some(left), Some(right)) => left == right,
            (None, Some(values)) | (Some(values), None) => values.is_empty(),
        }
}

fn group_key<'a>(
    plan: &CompiledAmountPlan,
    grouping: Grouping,
    member: MemberRef<'a>,
    node_index: usize,
) -> Result<GroupKey<'a>, ValueExprFault> {
    match grouping {
        Grouping::WholeSet => Ok(GroupKey::WholeSet),
        Grouping::ByOwner => member.owner().map(GroupKey::Owner).ok_or_else(|| {
            fault(
                plan,
                ValueExprFaultCode::InvalidOwner,
                node_index,
                None,
                None,
                "aggregate member has no owner grouping key",
                ValueExprPhase::Evaluate,
            )
        }),
        Grouping::ByResolvedOwner(relation) => {
            let owner = member.owner().ok_or_else(|| {
                fault(
                    plan,
                    ValueExprFaultCode::InvalidOwner,
                    node_index,
                    None,
                    None,
                    "resolved aggregate member has no relation key",
                    ValueExprPhase::Evaluate,
                )
            })?;
            if owner.domain != resolved_relation_domain(relation) {
                return Err(fault(
                    plan,
                    ValueExprFaultCode::InvalidOwner,
                    node_index,
                    None,
                    None,
                    "resolved aggregate member relation key has the wrong domain",
                    ValueExprPhase::Evaluate,
                ));
            }
            Ok(GroupKey::Owner(owner))
        }
        Grouping::ByProjectedTile => member
            .projected_tile()
            .map(GroupKey::ProjectedTile)
            .ok_or_else(|| {
                fault(
                    plan,
                    ValueExprFaultCode::MissingContext,
                    node_index,
                    None,
                    None,
                    "aggregate member has no projected-tile grouping key",
                    ValueExprPhase::Evaluate,
                )
            }),
    }
}

fn grouping_uses_owner_key(grouping: Grouping) -> bool {
    matches!(grouping, Grouping::ByOwner | Grouping::ByResolvedOwner(_))
}

fn resolved_relation_domain(relation: ResolvedOwnerRelation) -> GroupOwnerDomain {
    match relation {
        ResolvedOwnerRelation::Side => GroupOwnerDomain::Side,
        ResolvedOwnerRelation::ActualEmployee => GroupOwnerDomain::Employee,
        ResolvedOwnerRelation::EquippedSlot(_) => GroupOwnerDomain::EquipmentCopy,
    }
}

fn finish_aggregate(
    plan: &CompiledAmountPlan,
    meter: &mut ExprMeter,
    aggregate: &AggregateTask<'_>,
) -> Result<TypedInteger, ValueExprFault> {
    let result_slots = if aggregate.groups.is_empty() && aggregate.grouping == Grouping::WholeSet {
        1
    } else {
        aggregate.groups.len()
    };
    let result_storage = result_slots
        .checked_mul(size_of::<TypedInteger>())
        .and_then(|bytes| bytes.checked_mul(2))
        .ok_or_else(|| accounting_overflow(plan, aggregate.node_index, "group result storage"))?;
    charge_retained(plan, meter, aggregate.node_index, result_storage)?;
    let mut group_results = Vec::with_capacity(result_slots);
    if aggregate.groups.is_empty() && aggregate.grouping == Grouping::WholeSet {
        group_results.push(reduce(
            plan,
            meter,
            aggregate.node_index,
            aggregate.reduction,
            &aggregate.member_unit,
            &[],
        )?);
    } else {
        for values in aggregate.groups.values() {
            group_results.push(reduce(
                plan,
                meter,
                aggregate.node_index,
                aggregate.reduction,
                &aggregate.member_unit,
                values,
            )?);
        }
    }
    let value = match aggregate.outer_reduction {
        Some(outer) => reduce(
            plan,
            meter,
            aggregate.node_index,
            outer,
            &reduction_unit(aggregate.reduction, &aggregate.member_unit),
            &group_results,
        )?,
        None => group_results.into_iter().next().ok_or_else(|| {
            fault(
                plan,
                ValueExprFaultCode::Unavailable,
                aggregate.node_index,
                Some(aggregate.result_unit.clone()),
                None,
                "ungrouped aggregate did not produce a value",
                ValueExprPhase::Evaluate,
            )
        })?,
    };
    require_unit(plan, aggregate.node_index, &aggregate.result_unit, &value)?;
    Ok(value)
}

fn reduce(
    plan: &CompiledAmountPlan,
    meter: &mut ExprMeter,
    node_index: usize,
    reduction: Reduction,
    member_unit: &ValueUnit,
    values: &[TypedInteger],
) -> Result<TypedInteger, ValueExprFault> {
    charge_work(
        plan,
        meter,
        node_index,
        u64::try_from(values.len())
            .map_err(|_| accounting_overflow(plan, node_index, "reduction work"))?,
    )?;
    for value in values {
        require_unit(plan, node_index, member_unit, value)?;
    }
    match reduction {
        Reduction::Count => make_stack_value(
            plan,
            meter,
            node_index,
            values.len() as i128,
            &ValueUnit::Count,
        ),
        Reduction::Sum => {
            let mut result = 0_i128;
            for value in values {
                result = result.checked_add(value.value).ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::Overflow,
                        node_index,
                        Some(member_unit.clone()),
                        Some(member_unit.clone()),
                        "aggregate Sum overflowed i128",
                        ValueExprPhase::Evaluate,
                    )
                })?;
            }
            make_stack_value(plan, meter, node_index, result, member_unit)
        }
        Reduction::Max => {
            let value = values
                .iter()
                .max_by_key(|value| value.value)
                .ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::Unavailable,
                        node_index,
                        Some(member_unit.clone()),
                        None,
                        "Max is undefined for an empty finite aggregate",
                        ValueExprPhase::Evaluate,
                    )
                })?;
            clone_stack_value(plan, meter, node_index, value)
        }
    }
}

fn reduction_unit(reduction: Reduction, member_unit: &ValueUnit) -> ValueUnit {
    match reduction {
        Reduction::Count => ValueUnit::Count,
        Reduction::Sum | Reduction::Max => member_unit.clone(),
    }
}

fn operand_start(
    plan: &CompiledAmountPlan,
    values: &[TypedInteger],
    count: usize,
    node_index: usize,
) -> Result<usize, ValueExprFault> {
    values.len().checked_sub(count).ok_or_else(|| {
        fault(
            plan,
            ValueExprFaultCode::Unavailable,
            node_index,
            None,
            None,
            "evaluation stack underflow",
            ValueExprPhase::Evaluate,
        )
    })
}

fn make_stack_value(
    plan: &CompiledAmountPlan,
    meter: &mut ExprMeter,
    node_index: usize,
    value: i128,
    unit: &ValueUnit,
) -> Result<TypedInteger, ValueExprFault> {
    let dynamic = value_unit_dynamic_bytes(unit)
        .checked_mul(2)
        .ok_or_else(|| accounting_overflow(plan, node_index, "value unit payload"))?;
    charge_retained(plan, meter, node_index, dynamic)?;
    Ok(TypedInteger::new(value, unit.clone()))
}

fn clone_stack_value(
    plan: &CompiledAmountPlan,
    meter: &mut ExprMeter,
    node_index: usize,
    value: &TypedInteger,
) -> Result<TypedInteger, ValueExprFault> {
    let dynamic = typed_integer_dynamic_bytes(value)
        .checked_mul(2)
        .ok_or_else(|| accounting_overflow(plan, node_index, "value payload"))?;
    charge_retained(plan, meter, node_index, dynamic)?;
    Ok(value.clone())
}

fn push_stack_value(
    plan: &CompiledAmountPlan,
    meter: &mut ExprMeter,
    runtime: &mut RuntimeState,
    values: &mut Vec<TypedInteger>,
    value: TypedInteger,
    node_index: usize,
) -> Result<(), ValueExprFault> {
    let next = runtime
        .stack_values
        .checked_add(1)
        .ok_or_else(|| accounting_overflow(plan, node_index, "simultaneous value-stack usage"))?;
    if next > meter.limits.max_stack || next > plan.cost().max_stack {
        return Err(fault(
            plan,
            ValueExprFaultCode::BudgetExceeded,
            node_index,
            None,
            None,
            format!(
                "simultaneous value-stack usage {next} exceeds evaluation limit {}",
                meter.limits.max_stack
            ),
            ValueExprPhase::Evaluate,
        ));
    }
    runtime.stack_values = next;
    meter.usage.peak_stack_values = meter.usage.peak_stack_values.max(next);
    values.push(value);
    Ok(())
}

fn consume_stack_values(
    plan: &CompiledAmountPlan,
    runtime: &mut RuntimeState,
    count: usize,
    node_index: usize,
) -> Result<(), ValueExprFault> {
    runtime.stack_values = runtime.stack_values.checked_sub(count).ok_or_else(|| {
        fault(
            plan,
            ValueExprFaultCode::Unavailable,
            node_index,
            None,
            None,
            "simultaneous value-stack accounting underflowed",
            ValueExprPhase::Evaluate,
        )
    })?;
    Ok(())
}

fn push_task(
    plan: &CompiledAmountPlan,
    meter: &mut ExprMeter,
    runtime: &mut RuntimeState,
    node_index: usize,
) -> Result<(), ValueExprFault> {
    let next = runtime
        .task_frames
        .checked_add(1)
        .ok_or_else(|| accounting_overflow(plan, node_index, "task-frame usage"))?;
    if next > plan.cost().max_task_frames {
        return Err(fault(
            plan,
            ValueExprFaultCode::BudgetExceeded,
            node_index,
            None,
            None,
            "task-frame usage exceeds the compiled structural requirement",
            ValueExprPhase::Evaluate,
        ));
    }
    runtime.task_frames = next;
    meter.usage.peak_task_frames = meter.usage.peak_task_frames.max(next);
    Ok(())
}

fn pop_task(
    plan: &CompiledAmountPlan,
    runtime: &mut RuntimeState,
    node_index: usize,
) -> Result<(), ValueExprFault> {
    runtime.task_frames = runtime.task_frames.checked_sub(1).ok_or_else(|| {
        fault(
            plan,
            ValueExprFaultCode::Unavailable,
            node_index,
            None,
            None,
            "task-frame accounting underflowed",
            ValueExprPhase::Evaluate,
        )
    })?;
    Ok(())
}

fn require_unit(
    plan: &CompiledAmountPlan,
    node_index: usize,
    expected: &ValueUnit,
    value: &TypedInteger,
) -> Result<(), ValueExprFault> {
    if &value.unit == expected {
        Ok(())
    } else {
        Err(fault(
            plan,
            ValueExprFaultCode::UnitMismatch,
            node_index,
            Some(expected.clone()),
            Some(value.unit.clone()),
            "runtime value unit differs from the compiled unit",
            ValueExprPhase::Evaluate,
        ))
    }
}

fn push_trace(
    plan: &CompiledAmountPlan,
    meter: &mut ExprMeter,
    trace: &mut Vec<TraceEntry>,
    node_index: usize,
    member_identity: Option<&str>,
    value: &TypedInteger,
) -> Result<(), ValueExprFault> {
    #[cfg(test)]
    crate::authoring::control::test_cancellation_site("value_expr_trace");
    let retained = size_of::<TraceEntry>()
        .checked_mul(4)
        .and_then(|bytes| bytes.checked_add(member_identity.map_or(0, str::len).checked_mul(2)?))
        .and_then(|bytes| bytes.checked_add(typed_integer_dynamic_bytes(value).checked_mul(2)?))
        .ok_or_else(|| accounting_overflow(plan, node_index, "trace storage"))?;
    charge_retained(plan, meter, node_index, retained)?;
    trace.push(TraceEntry {
        node_index,
        member_identity: member_identity.map(str::to_owned),
        value: value.clone(),
    });
    Ok(())
}

fn admit_frame_values(
    plan: &CompiledAmountPlan,
    frame: &EvalFrame,
    meter: &mut ExprMeter,
) -> Result<(), ValueExprFault> {
    let references = &plan.requirements().references;
    let relevant_count = references
        .iter()
        .filter(|reference| !reference.is_member_scoped() && frame.contains_key(reference))
        .count();
    if relevant_count == 0 {
        return Ok(());
    }
    charge_work(
        plan,
        meter,
        0,
        u64::try_from(relevant_count)
            .map_err(|_| accounting_overflow(plan, 0, "frame value count"))?,
    )?;
    for reference in references {
        if reference.is_member_scoped() {
            continue;
        }
        let Some(value) = frame.get(reference) else {
            continue;
        };
        let dynamic = value_ref_dynamic_bytes(reference)
            .ok_or_else(|| accounting_overflow(plan, 0, "frame reference payload"))?
            .checked_add(typed_integer_dynamic_bytes(value))
            .ok_or_else(|| accounting_overflow(plan, 0, "frame value payload"))?;
        if dynamic > 0 {
            charge_work(
                plan,
                meter,
                0,
                text_work_units(dynamic)
                    .ok_or_else(|| accounting_overflow(plan, 0, "frame payload work"))?,
            )?;
        }
    }
    Ok(())
}

fn charge_work(
    plan: &CompiledAmountPlan,
    meter: &mut ExprMeter,
    node_index: usize,
    amount: u64,
) -> Result<(), ValueExprFault> {
    poll_cancelled(plan, meter, node_index)?;
    let requested = meter
        .usage
        .work
        .checked_add(amount)
        .ok_or_else(|| accounting_overflow(plan, node_index, "evaluation work"))?;
    if requested > meter.limits.max_work {
        return Err(fault(
            plan,
            ValueExprFaultCode::BudgetExceeded,
            node_index,
            None,
            None,
            format!(
                "evaluation work {} exceeds limit {}",
                requested, meter.limits.max_work
            ),
            ValueExprPhase::Evaluate,
        ));
    }
    meter.usage.work = requested;
    if let Some(admit) = meter.host_admission {
        admit(plan, node_index, amount, 0)?;
    }
    Ok(())
}

fn charge_raw_members(
    plan: &CompiledAmountPlan,
    meter: &mut ExprMeter,
    node_index: usize,
    amount: usize,
) -> Result<(), ValueExprFault> {
    meter.usage.raw_members = meter
        .usage
        .raw_members
        .checked_add(amount)
        .ok_or_else(|| accounting_overflow(plan, node_index, "raw member count"))?;
    charge_work(
        plan,
        meter,
        node_index,
        u64::try_from(amount)
            .map_err(|_| accounting_overflow(plan, node_index, "raw member work"))?,
    )
}

fn charge_members(
    plan: &CompiledAmountPlan,
    meter: &mut ExprMeter,
    node_index: usize,
    amount: usize,
) -> Result<(), ValueExprFault> {
    meter.usage.dynamic_members = meter
        .usage
        .dynamic_members
        .checked_add(amount)
        .ok_or_else(|| accounting_overflow(plan, node_index, "dynamic member count"))?;
    if meter.usage.dynamic_members > meter.limits.max_dynamic_members {
        return Err(fault(
            plan,
            ValueExprFaultCode::BudgetExceeded,
            node_index,
            None,
            None,
            format!(
                "canonical dynamic members {} exceed limit {}",
                meter.usage.dynamic_members, meter.limits.max_dynamic_members
            ),
            ValueExprPhase::Evaluate,
        ));
    }
    Ok(())
}

fn charge_group(
    plan: &CompiledAmountPlan,
    meter: &mut ExprMeter,
    node_index: usize,
) -> Result<(), ValueExprFault> {
    meter.usage.groups = meter
        .usage
        .groups
        .checked_add(1)
        .ok_or_else(|| accounting_overflow(plan, node_index, "group count"))?;
    if meter.usage.groups > meter.limits.max_groups {
        return Err(fault(
            plan,
            ValueExprFaultCode::BudgetExceeded,
            node_index,
            None,
            None,
            format!(
                "aggregate groups {} exceed limit {}",
                meter.usage.groups, meter.limits.max_groups
            ),
            ValueExprPhase::Evaluate,
        ));
    }
    Ok(())
}

fn charge_retained(
    plan: &CompiledAmountPlan,
    meter: &mut ExprMeter,
    node_index: usize,
    amount: usize,
) -> Result<(), ValueExprFault> {
    let requested = meter
        .usage
        .retained_bytes
        .checked_add(amount)
        .ok_or_else(|| accounting_overflow(plan, node_index, "retained-byte usage"))?;
    if requested > meter.limits.max_retained_bytes {
        return Err(fault(
            plan,
            ValueExprFaultCode::BudgetExceeded,
            node_index,
            None,
            None,
            format!(
                "retained bytes {} exceed limit {}",
                requested, meter.limits.max_retained_bytes
            ),
            ValueExprPhase::Evaluate,
        ));
    }
    meter.usage.retained_bytes = requested;
    if let Some(admit) = meter.host_admission {
        admit(plan, node_index, 0, amount)?;
    }
    Ok(())
}

fn group_storage_bytes() -> Option<usize> {
    // Charge a full node allowance per new key; BTree's branching factor and
    // allocator layout are private implementation details.
    1024_usize.checked_add(
        size_of::<GroupKey<'_>>()
            .checked_add(size_of::<Vec<TypedInteger>>())?
            .checked_mul(2)?,
    )
}

fn accounting_overflow(
    plan: &CompiledAmountPlan,
    node_index: usize,
    label: &'static str,
) -> ValueExprFault {
    fault(
        plan,
        ValueExprFaultCode::Overflow,
        node_index,
        None,
        None,
        format!("{label} accounting overflowed"),
        ValueExprPhase::Evaluate,
    )
}

fn poll_cancelled(
    plan: &CompiledAmountPlan,
    meter: &ExprMeter,
    node_index: usize,
) -> Result<(), ValueExprFault> {
    if let Some(admit) = meter.host_admission {
        admit(plan, node_index, 0, 0)?;
    }
    if meter.cancelled {
        Err(fault(
            plan,
            ValueExprFaultCode::Cancelled,
            node_index,
            None,
            None,
            "expression evaluation was cancelled",
            ValueExprPhase::Evaluate,
        ))
    } else {
        Ok(())
    }
}

pub fn narrow_damage(site: &EffectSite, value: &TypedInteger) -> Result<u32, ValueExprFault> {
    if value.unit != ValueUnit::Magnitude {
        return Err(narrow_fault(
            site,
            ValueExprFaultCode::UnitMismatch,
            Some(ValueUnit::Magnitude),
            Some(bounded_value_unit(&value.unit)),
            "Damage amount must have Magnitude unit",
        ));
    }
    u32::try_from(value.value).map_err(|_| {
        narrow_fault(
            site,
            ValueExprFaultCode::Overflow,
            Some(ValueUnit::Magnitude),
            Some(ValueUnit::Magnitude),
            "Damage amount is outside 0..=u32::MAX",
        )
    })
}

pub fn narrow_heal(site: &EffectSite, value: &TypedInteger) -> Result<i32, ValueExprFault> {
    if value.unit != ValueUnit::Magnitude {
        return Err(narrow_fault(
            site,
            ValueExprFaultCode::UnitMismatch,
            Some(ValueUnit::Magnitude),
            Some(bounded_value_unit(&value.unit)),
            "Heal amount must have Magnitude unit",
        ));
    }
    i32::try_from(value.value).map_err(|_| {
        narrow_fault(
            site,
            ValueExprFaultCode::Overflow,
            Some(ValueUnit::Magnitude),
            Some(ValueUnit::Magnitude),
            "Heal amount is outside i32 range",
        )
    })
}

fn narrow_fault(
    site: &EffectSite,
    code: ValueExprFaultCode,
    expected_unit: Option<ValueUnit>,
    actual_unit: Option<ValueUnit>,
    cause: impl Into<String>,
) -> ValueExprFault {
    ValueExprFault {
        code,
        site: bounded_effect_site(site),
        node_index: 0,
        expected_unit: expected_unit.as_ref().map(bounded_value_unit),
        actual_unit: actual_unit.as_ref().map(bounded_value_unit),
        phase: ValueExprPhase::Narrow,
        cause: cause.into(),
    }
}

fn fault(
    plan: &CompiledAmountPlan,
    code: ValueExprFaultCode,
    node_index: usize,
    expected_unit: Option<ValueUnit>,
    actual_unit: Option<ValueUnit>,
    cause: impl Into<String>,
    phase: ValueExprPhase,
) -> ValueExprFault {
    ValueExprFault {
        code,
        site: bounded_effect_site(&plan.source().site),
        node_index,
        expected_unit: expected_unit.as_ref().map(bounded_value_unit),
        actual_unit: actual_unit.as_ref().map(bounded_value_unit),
        phase,
        cause: cause.into(),
    }
}
