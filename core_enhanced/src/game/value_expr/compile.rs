use std::collections::BTreeSet;
use std::mem::size_of;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::game::ability::authoring::{ActionSite, EffectOrigin, EffectSite, EffectSource};
use crate::game::ability::SkillId;
use crate::game::resource::{ResourceId, ResourceOwnerSelector};

use super::{
    aggregate_member_scan_work, aggregate_sort_work, text_work_units, typed_integer_dynamic_bytes,
    value_ref_dynamic_bytes, value_unit_dynamic_bytes, AmountExpr, AmountProvenance,
    AuthoredMember, ContextRequirements, DeliveryCapabilities, EvalMoment, ExprDiagnostic,
    ExprLimits, FiniteSource, GroupOwnerDomain, Grouping, HitPointField, PlanCost, Reduction,
    ResolvedOwnerRelation, ResolvedTargetPredicate, ResourceValueField, ResultField,
    ResultSelector, Rounding, StatSelector, TimeValue, TypedInteger, ValueExpr, ValueExprFaultCode,
    ValueExprPhase, ValueRef, ValueSubject, ValueUnit, VALUE_EXPR_COMPILER_VERSION,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum CompiledOp {
    PushConst {
        value: TypedInteger,
        node_index: usize,
    },
    Read {
        reference: ValueRef,
        node_index: usize,
    },
    Add {
        arity: usize,
        unit: ValueUnit,
        node_index: usize,
    },
    Mul {
        unit: ValueUnit,
        node_index: usize,
    },
    Ratio {
        rounding: Rounding,
        unit: ValueUnit,
        node_index: usize,
    },
    Clamp {
        unit: ValueUnit,
        node_index: usize,
    },
    Aggregate {
        source: FiniteSource,
        grouping: Grouping,
        reduction: Reduction,
        outer_reduction: Option<Reduction>,
        member_segment: usize,
        member_unit: ValueUnit,
        result_unit: ValueUnit,
        node_index: usize,
    },
}

impl CompiledOp {
    pub fn node_index(&self) -> usize {
        match self {
            Self::PushConst { node_index, .. }
            | Self::Read { node_index, .. }
            | Self::Add { node_index, .. }
            | Self::Mul { node_index, .. }
            | Self::Ratio { node_index, .. }
            | Self::Clamp { node_index, .. }
            | Self::Aggregate { node_index, .. } => *node_index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompiledAmountPlan {
    pub(crate) segments: Box<[Box<[CompiledOp]>]>,
    pub(crate) limits: ExprLimits,
    pub(crate) source: EffectSource,
    result_unit: ValueUnit,
    moment: EvalMoment,
    provenance: AmountProvenance,
    requirements: ContextRequirements,
    cost: PlanCost,
    semantic_fingerprint: [u8; 32],
}

impl CompiledAmountPlan {
    pub fn root_ops(&self) -> &[CompiledOp] {
        &self.segments[0]
    }

    pub fn segment(&self, index: usize) -> Option<&[CompiledOp]> {
        self.segments.get(index).map(AsRef::as_ref)
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    pub fn result_unit(&self) -> &ValueUnit {
        &self.result_unit
    }

    pub fn moment(&self) -> EvalMoment {
        self.moment
    }

    pub fn provenance(&self) -> AmountProvenance {
        self.provenance
    }

    pub fn requirements(&self) -> &ContextRequirements {
        &self.requirements
    }

    pub(crate) fn evaluation_limits(&self) -> ExprLimits {
        self.limits
    }

    pub fn cost(&self) -> PlanCost {
        self.cost
    }

    pub fn source(&self) -> &EffectSource {
        &self.source
    }

    pub fn semantic_fingerprint(&self) -> [u8; 32] {
        self.semantic_fingerprint
    }

    pub fn semantic_fingerprint_hex(&self) -> String {
        let mut result = String::with_capacity(64);
        for byte in self.semantic_fingerprint {
            use std::fmt::Write;
            write!(&mut result, "{byte:02x}").expect("writing to a String is infallible");
        }
        result
    }
}

#[derive(Debug, Clone)]
struct CompileValue {
    unit: ValueUnit,
    constant: Option<i128>,
    work: u64,
}

enum Task<'a> {
    Enter {
        expr: &'a ValueExpr,
        depth: usize,
        segment: usize,
        inside_aggregate: bool,
    },
    ContinueAdd {
        values: &'a [ValueExpr],
        next_child: usize,
        depth: usize,
        segment: usize,
        inside_aggregate: bool,
        start: usize,
        node_index: usize,
    },
    ExitMul {
        segment: usize,
        start: usize,
        node_index: usize,
    },
    ExitRatio {
        segment: usize,
        start: usize,
        rounding: Rounding,
        node_index: usize,
    },
    ExitClamp {
        segment: usize,
        start: usize,
        node_index: usize,
    },
    ExitAggregate {
        parent_segment: usize,
        member_segment: usize,
        source: FiniteSource,
        grouping: Grouping,
        reduction: Reduction,
        outer_reduction: Option<Reduction>,
        static_preprocessing_work: Option<u64>,
        node_index: usize,
    },
}

struct Compiler<'a> {
    expression: &'a AmountExpr,
    source: &'a EffectSource,
    capabilities: DeliveryCapabilities,
    limits: ExprLimits,
    segments: Vec<Vec<CompiledOp>>,
    values: Vec<CompileValue>,
    requirements: BTreeSet<ValueRef>,
    resolved_targets: bool,
    state_owner_dot_targets: bool,
    owner_grouping: bool,
    projected_tile_grouping: bool,
    resolved_owner_relations: BTreeSet<ResolvedOwnerRelation>,
    resolved_target_predicates: BTreeSet<ResolvedTargetPredicate>,
    admitted_nodes: usize,
    nodes: usize,
    max_depth_seen: usize,
    authored_members: usize,
    compile_work: u64,
    compile_reserved_bytes: usize,
    plan_retained_bytes: usize,
    runtime_dynamic_work: bool,
    resolved_aggregate_count: usize,
}

pub fn compile(
    expression: &AmountExpr,
    source: &EffectSource,
    capabilities: DeliveryCapabilities,
    limits: &ExprLimits,
) -> Result<CompiledAmountPlan, ExprDiagnostic> {
    let limits = limits.validate().map_err(|cause| ExprDiagnostic {
        code: ValueExprFaultCode::BudgetExceeded,
        site: bounded_effect_site(&source.site),
        node_index: 0,
        expected_unit: None,
        actual_unit: None,
        phase: ValueExprPhase::Compile,
        cause: cause.to_string(),
    })?;
    Compiler {
        expression,
        source,
        capabilities,
        limits,
        segments: Vec::new(),
        values: Vec::new(),
        requirements: BTreeSet::new(),
        resolved_targets: false,
        state_owner_dot_targets: false,
        owner_grouping: false,
        projected_tile_grouping: false,
        resolved_owner_relations: BTreeSet::new(),
        resolved_target_predicates: BTreeSet::new(),
        admitted_nodes: 0,
        nodes: 0,
        max_depth_seen: 0,
        authored_members: 0,
        compile_work: 0,
        compile_reserved_bytes: size_of::<Compiler<'_>>()
            .checked_add(size_of::<Vec<CompiledOp>>() * 2)
            .ok_or_else(|| ExprDiagnostic {
                code: ValueExprFaultCode::Overflow,
                site: bounded_effect_site(&source.site),
                node_index: 0,
                expected_unit: None,
                actual_unit: None,
                phase: ValueExprPhase::Compile,
                cause: "compiler base storage accounting overflowed".to_string(),
            })?,
        plan_retained_bytes: 0,
        runtime_dynamic_work: false,
        resolved_aggregate_count: 0,
    }
    .compile()
}

impl<'a> Compiler<'a> {
    fn compile(mut self) -> Result<CompiledAmountPlan, ExprDiagnostic> {
        self.admit_source_storage()?;
        self.ensure_compile_storage(0)?;
        self.admit_plan_storage(
            size_of::<Vec<CompiledOp>>().checked_mul(4).ok_or_else(|| {
                self.error(
                    ValueExprFaultCode::Overflow,
                    0,
                    None,
                    None,
                    "root segment storage accounting overflowed",
                )
            })?,
            0,
            "root segment storage",
        )?;
        self.segments.push(Vec::new());
        match self.expression.moment {
            EvalMoment::Snapshot if !self.capabilities.snapshot => {
                return Err(self.error(
                    ValueExprFaultCode::UnsupportedMoment,
                    0,
                    None,
                    None,
                    "this delivery does not support Snapshot evaluation",
                ));
            }
            EvalMoment::Live if !self.capabilities.live => {
                return Err(self.error(
                    ValueExprFaultCode::UnsupportedMoment,
                    0,
                    None,
                    None,
                    "this delivery does not support Live evaluation",
                ));
            }
            _ => {}
        }

        let mut tasks = Vec::new();
        self.schedule(
            &mut tasks,
            Task::Enter {
                expr: &self.expression.root,
                depth: 1,
                segment: 0,
                inside_aggregate: false,
            },
            0,
        )?;
        while let Some(task) = tasks.pop() {
            match task {
                Task::Enter {
                    expr,
                    depth,
                    segment,
                    inside_aggregate,
                } => self.enter(expr, depth, segment, inside_aggregate, &mut tasks)?,
                Task::ContinueAdd {
                    values,
                    next_child,
                    depth,
                    segment,
                    inside_aggregate,
                    start,
                    node_index,
                } => {
                    if next_child == values.len() {
                        self.exit_add(segment, start, values.len(), node_index)?;
                    } else {
                        self.schedule(
                            &mut tasks,
                            Task::ContinueAdd {
                                values,
                                next_child: next_child + 1,
                                depth,
                                segment,
                                inside_aggregate,
                                start,
                                node_index,
                            },
                            node_index,
                        )?;
                        self.schedule(
                            &mut tasks,
                            Task::Enter {
                                expr: &values[next_child],
                                depth: depth + 1,
                                segment,
                                inside_aggregate,
                            },
                            node_index,
                        )?;
                    }
                }
                Task::ExitMul {
                    segment,
                    start,
                    node_index,
                } => self.exit_mul(segment, start, node_index)?,
                Task::ExitRatio {
                    segment,
                    start,
                    rounding,
                    node_index,
                } => self.exit_ratio(segment, start, rounding, node_index)?,
                Task::ExitClamp {
                    segment,
                    start,
                    node_index,
                } => self.exit_clamp(segment, start, node_index)?,
                Task::ExitAggregate {
                    parent_segment,
                    member_segment,
                    source,
                    grouping,
                    reduction,
                    outer_reduction,
                    static_preprocessing_work,
                    node_index,
                } => self.exit_aggregate(
                    parent_segment,
                    member_segment,
                    source,
                    grouping,
                    reduction,
                    outer_reduction,
                    static_preprocessing_work,
                    node_index,
                )?,
            }
        }

        if self.values.len() != 1 {
            return Err(self.error(
                ValueExprFaultCode::Unavailable,
                0,
                None,
                None,
                "compiler did not produce exactly one root value",
            ));
        }
        let root = self.values.pop().expect("length checked");
        self.admit_working_storage(
            self.segments
                .len()
                .checked_mul(size_of::<Option<ExecutionShape>>())
                .and_then(|bytes| bytes.checked_mul(2))
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        0,
                        None,
                        None,
                        "execution-shape scratch accounting overflowed",
                    )
                })?,
            0,
            "execution-shape scratch",
        )?;
        let execution_shape = self.execution_shape()?;
        let max_stack = execution_shape.max_stack;
        if max_stack > self.limits.max_stack {
            return Err(self.error(
                ValueExprFaultCode::BudgetExceeded,
                0,
                None,
                None,
                format!(
                    "compiled stack {max_stack} exceeds limit {}",
                    self.limits.max_stack
                ),
            ));
        }
        // Each resolved aggregate is conservatively priced as though it alone
        // consumed the dynamic member limit. Multiple node views instead share
        // one runtime member limit, and preparation plus evaluation meter their
        // actual combined work before gameplay mutation.
        if root.work > self.limits.max_work && self.resolved_aggregate_count < 2 {
            return Err(self.error(
                ValueExprFaultCode::BudgetExceeded,
                0,
                None,
                None,
                format!(
                    "worst-case work {} exceeds limit {}",
                    root.work, self.limits.max_work
                ),
            ));
        }

        self.charge_text_work(self.plan_retained_bytes, 0, "semantic fingerprint input")?;
        self.admit_working_storage(
            size_of::<Sha256>().checked_mul(2).ok_or_else(|| {
                self.error(
                    ValueExprFaultCode::Overflow,
                    0,
                    None,
                    None,
                    "semantic fingerprint state accounting overflowed",
                )
            })?,
            0,
            "semantic fingerprint state",
        )?;
        let semantic_fingerprint = canonical_plan_digest(
            self.expression.moment,
            self.expression.provenance,
            &self.source.site,
            &self.segments,
            &root.unit,
        );
        let worst_case_work = if self.runtime_dynamic_work {
            self.limits.max_work
        } else {
            root.work
        };
        let static_work = if self.runtime_dynamic_work {
            root.work.min(self.limits.max_work)
        } else {
            root.work
        };
        let segments = self
            .segments
            .into_iter()
            .map(|ops| ops.into_boxed_slice())
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Ok(CompiledAmountPlan {
            segments,
            limits: self.limits,
            source: self.source.clone(),
            result_unit: root.unit,
            moment: self.expression.moment,
            provenance: self.expression.provenance,
            requirements: ContextRequirements {
                references: self.requirements.into_iter().collect(),
                resolved_targets: self.resolved_targets,
                state_owner_dot_targets: self.state_owner_dot_targets,
                owner_grouping: self.owner_grouping,
                projected_tile_grouping: self.projected_tile_grouping,
                resolved_owner_relations: self.resolved_owner_relations.into_iter().collect(),
                resolved_target_predicates: self.resolved_target_predicates.into_iter().collect(),
            },
            cost: PlanCost {
                nodes: self.admitted_nodes,
                max_depth: self.max_depth_seen,
                max_stack,
                stack_storage_slots: execution_shape.stack_storage_slots,
                max_task_frames: execution_shape.max_task_frames,
                static_work,
                runtime_dynamic_work: self.runtime_dynamic_work,
                worst_case_work,
                compile_work: self.compile_work,
                compile_reserved_bytes: self.compile_reserved_bytes,
                plan_retained_bytes: self.plan_retained_bytes,
                authored_members: self.authored_members,
            },
            semantic_fingerprint,
        })
    }

    fn schedule(
        &mut self,
        tasks: &mut Vec<Task<'a>>,
        task: Task<'a>,
        node_index: usize,
    ) -> Result<(), ExprDiagnostic> {
        let bytes = size_of::<Task<'a>>().checked_mul(4).ok_or_else(|| {
            self.error(
                ValueExprFaultCode::Overflow,
                node_index,
                None,
                None,
                "compiler task storage accounting overflowed",
            )
        })?;
        self.admit_working_storage(bytes, node_index, "compiler task storage")?;
        tasks.push(task);
        Ok(())
    }

    fn admit_source_storage(&mut self) -> Result<(), ExprDiagnostic> {
        let mut dynamic = 0_usize;
        for bytes in [
            self.source.source_file.len(),
            self.source.site.action.definition_id.as_str().len(),
            self.source.site.action.action_id.len(),
            self.source.site.effect_id.len(),
            self.source.field_path.len(),
            self.source.origin_definition_id.as_str().len(),
        ] {
            dynamic = dynamic.checked_add(bytes).ok_or_else(|| {
                self.error(
                    ValueExprFaultCode::Overflow,
                    0,
                    None,
                    None,
                    "source payload accounting overflowed",
                )
            })?;
        }
        if let EffectOrigin::Added {
            source_definition_id,
        } = &self.source.origin
        {
            dynamic = dynamic
                .checked_add(source_definition_id.as_str().len())
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        0,
                        None,
                        None,
                        "source origin payload accounting overflowed",
                    )
                })?;
        }
        self.charge_text_work(dynamic, 0, "source payload scan")?;
        let retained = size_of::<EffectSource>()
            .checked_mul(2)
            .and_then(|bytes| bytes.checked_add(dynamic.checked_mul(2)?))
            .ok_or_else(|| {
                self.error(
                    ValueExprFaultCode::Overflow,
                    0,
                    None,
                    None,
                    "source retained storage accounting overflowed",
                )
            })?;
        self.admit_plan_storage(retained, 0, "compiled source payload")
    }

    fn admit_dynamic_payload(
        &mut self,
        bytes: usize,
        copies: usize,
        node_index: usize,
        label: &'static str,
    ) -> Result<(), ExprDiagnostic> {
        self.charge_text_work(bytes, node_index, label)?;
        let retained = bytes.checked_mul(copies).ok_or_else(|| {
            self.error(
                ValueExprFaultCode::Overflow,
                node_index,
                None,
                None,
                "dynamic payload storage accounting overflowed",
            )
        })?;
        self.admit_plan_storage(retained, node_index, label)
    }

    fn charge_text_work(
        &mut self,
        bytes: usize,
        node_index: usize,
        label: &'static str,
    ) -> Result<(), ExprDiagnostic> {
        if bytes == 0 {
            return Ok(());
        }
        let work = text_work_units(bytes).ok_or_else(|| {
            self.error(
                ValueExprFaultCode::Overflow,
                node_index,
                None,
                None,
                "text work accounting overflowed",
            )
        })?;
        self.charge_compile_work(work, node_index, label)
    }

    fn charge_compile_work(
        &mut self,
        amount: u64,
        node_index: usize,
        label: &'static str,
    ) -> Result<(), ExprDiagnostic> {
        self.compile_work = self.compile_work.checked_add(amount).ok_or_else(|| {
            self.error(
                ValueExprFaultCode::Overflow,
                node_index,
                None,
                None,
                "compile work accounting overflowed",
            )
        })?;
        if self.compile_work > self.limits.max_compile_work {
            return Err(self.error(
                ValueExprFaultCode::BudgetExceeded,
                node_index,
                None,
                None,
                format!(
                    "{label} raised compile work {} above limit {}",
                    self.compile_work, self.limits.max_compile_work
                ),
            ));
        }
        Ok(())
    }

    fn admit_plan_storage(
        &mut self,
        amount: usize,
        node_index: usize,
        label: &'static str,
    ) -> Result<(), ExprDiagnostic> {
        self.plan_retained_bytes =
            self.plan_retained_bytes
                .checked_add(amount)
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "plan retained storage accounting overflowed",
                    )
                })?;
        self.admit_working_storage(amount, node_index, label)
    }

    fn admit_working_storage(
        &mut self,
        amount: usize,
        node_index: usize,
        label: &'static str,
    ) -> Result<(), ExprDiagnostic> {
        self.compile_reserved_bytes =
            self.compile_reserved_bytes
                .checked_add(amount)
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "compiler storage accounting overflowed",
                    )
                })?;
        self.ensure_compile_storage_with_label(node_index, label)
    }

    fn ensure_compile_storage(&self, node_index: usize) -> Result<(), ExprDiagnostic> {
        self.ensure_compile_storage_with_label(node_index, "compiler base storage")
    }

    fn ensure_compile_storage_with_label(
        &self,
        node_index: usize,
        label: &'static str,
    ) -> Result<(), ExprDiagnostic> {
        if self.compile_reserved_bytes > self.limits.max_retained_bytes {
            return Err(self.error(
                ValueExprFaultCode::BudgetExceeded,
                node_index,
                None,
                None,
                format!(
                    "{label} raised compiler reserved bytes {} above limit {}",
                    self.compile_reserved_bytes, self.limits.max_retained_bytes
                ),
            ));
        }
        Ok(())
    }

    fn enter(
        &mut self,
        expr: &'a ValueExpr,
        depth: usize,
        segment: usize,
        inside_aggregate: bool,
        tasks: &mut Vec<Task<'a>>,
    ) -> Result<(), ExprDiagnostic> {
        if depth > self.limits.max_depth {
            return Err(self.error(
                ValueExprFaultCode::BudgetExceeded,
                self.nodes,
                None,
                None,
                format!(
                    "expression depth {depth} exceeds limit {}",
                    self.limits.max_depth
                ),
            ));
        }
        if self.admitted_nodes >= self.limits.max_nodes {
            return Err(self.error(
                ValueExprFaultCode::BudgetExceeded,
                self.nodes,
                None,
                None,
                format!(
                    "expression node count exceeds limit {}",
                    self.limits.max_nodes
                ),
            ));
        }
        let node_index = self.nodes;
        self.nodes += 1;
        self.admitted_nodes += 1;
        self.max_depth_seen = self.max_depth_seen.max(depth);
        self.charge_compile_work(1, node_index, "expression node scan")?;
        self.admit_plan_storage(
            size_of::<CompiledOp>()
                .checked_mul(4)
                .and_then(|bytes| bytes.checked_add(size_of::<CompileValue>() * 4))
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "compiled node storage accounting overflowed",
                    )
                })?,
            node_index,
            "compiled node storage",
        )?;
        let start = self.segments[segment].len();
        match expr {
            ValueExpr::Const(value) => {
                self.admit_dynamic_payload(
                    typed_integer_dynamic_bytes(value),
                    2,
                    node_index,
                    "constant unit payload",
                )?;
                self.segments[segment].push(CompiledOp::PushConst {
                    value: value.clone(),
                    node_index,
                });
                self.values.push(CompileValue {
                    unit: value.unit.clone(),
                    constant: Some(value.value),
                    work: 1,
                });
            }
            ValueExpr::Read(reference) => {
                if !reference.is_member_scoped() {
                    self.runtime_dynamic_work = true;
                }
                let dynamic = value_ref_dynamic_bytes(reference).ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "reference payload accounting overflowed",
                    )
                })?;
                self.admit_dynamic_payload(dynamic, 4, node_index, "reference payload")?;
                self.validate_reference(reference, inside_aggregate, node_index)?;
                self.admit_plan_storage(
                    size_of::<ValueRef>()
                        .checked_mul(16)
                        .and_then(|bytes| bytes.checked_add(size_of::<usize>() * 16))
                        .ok_or_else(|| {
                            self.error(
                                ValueExprFaultCode::Overflow,
                                node_index,
                                None,
                                None,
                                "reference container accounting overflowed",
                            )
                        })?,
                    node_index,
                    "compiled reference containers",
                )?;
                self.requirements.insert(reference.clone());
                self.segments[segment].push(CompiledOp::Read {
                    reference: reference.clone(),
                    node_index,
                });
                self.values.push(CompileValue {
                    unit: reference.unit(),
                    constant: None,
                    work: 1,
                });
            }
            ValueExpr::Add(values) => {
                if values.len() < 2 {
                    return Err(self.error(
                        ValueExprFaultCode::Unavailable,
                        node_index,
                        None,
                        None,
                        "Add requires at least two operands",
                    ));
                }
                let remaining_nodes = self.limits.max_nodes - self.admitted_nodes;
                if values.len() > remaining_nodes {
                    return Err(self.error(
                        ValueExprFaultCode::BudgetExceeded,
                        node_index,
                        None,
                        None,
                        "Add direct children exceed the remaining node budget",
                    ));
                }
                self.schedule(
                    tasks,
                    Task::ContinueAdd {
                        values,
                        next_child: 0,
                        depth,
                        segment,
                        inside_aggregate,
                        start,
                        node_index,
                    },
                    node_index,
                )?;
            }
            ValueExpr::Mul { value, scalar } => {
                self.schedule(
                    tasks,
                    Task::ExitMul {
                        segment,
                        start,
                        node_index,
                    },
                    node_index,
                )?;
                self.schedule(
                    tasks,
                    Task::Enter {
                        expr: scalar,
                        depth: depth + 1,
                        segment,
                        inside_aggregate,
                    },
                    node_index,
                )?;
                self.schedule(
                    tasks,
                    Task::Enter {
                        expr: value,
                        depth: depth + 1,
                        segment,
                        inside_aggregate,
                    },
                    node_index,
                )?;
            }
            ValueExpr::Ratio {
                numerator,
                denominator,
                rounding,
            } => {
                self.schedule(
                    tasks,
                    Task::ExitRatio {
                        segment,
                        start,
                        rounding: *rounding,
                        node_index,
                    },
                    node_index,
                )?;
                self.schedule(
                    tasks,
                    Task::Enter {
                        expr: denominator,
                        depth: depth + 1,
                        segment,
                        inside_aggregate,
                    },
                    node_index,
                )?;
                self.schedule(
                    tasks,
                    Task::Enter {
                        expr: numerator,
                        depth: depth + 1,
                        segment,
                        inside_aggregate,
                    },
                    node_index,
                )?;
            }
            ValueExpr::Clamp { value, min, max } => {
                self.schedule(
                    tasks,
                    Task::ExitClamp {
                        segment,
                        start,
                        node_index,
                    },
                    node_index,
                )?;
                self.schedule(
                    tasks,
                    Task::Enter {
                        expr: max,
                        depth: depth + 1,
                        segment,
                        inside_aggregate,
                    },
                    node_index,
                )?;
                self.schedule(
                    tasks,
                    Task::Enter {
                        expr: min,
                        depth: depth + 1,
                        segment,
                        inside_aggregate,
                    },
                    node_index,
                )?;
                self.schedule(
                    tasks,
                    Task::Enter {
                        expr: value,
                        depth: depth + 1,
                        segment,
                        inside_aggregate,
                    },
                    node_index,
                )?;
            }
            ValueExpr::Aggregate {
                source,
                grouping,
                reduction,
                outer_reduction,
                member,
            } => {
                let static_preprocessing_work =
                    self.admit_finite_source(source, *grouping, depth + 1, node_index)?;
                if matches!(source, FiniteSource::StateOwnerDotTargets) {
                    self.runtime_dynamic_work = true;
                }
                if is_resolved_source(source) {
                    self.runtime_dynamic_work = true;
                    self.resolved_aggregate_count = self
                        .resolved_aggregate_count
                        .checked_add(1)
                        .ok_or_else(|| {
                            self.error(
                                ValueExprFaultCode::Overflow,
                                node_index,
                                None,
                                None,
                                "resolved aggregate count overflowed",
                            )
                        })?;
                }
                self.validate_aggregate_capabilities(source, *grouping, node_index)?;
                let source = self.normalize_source(source, node_index)?;
                let member_segment = self.segments.len();
                self.admit_plan_storage(
                    size_of::<Vec<CompiledOp>>().checked_mul(2).ok_or_else(|| {
                        self.error(
                            ValueExprFaultCode::Overflow,
                            node_index,
                            None,
                            None,
                            "compiled segment storage accounting overflowed",
                        )
                    })?,
                    node_index,
                    "compiled segment storage",
                )?;
                self.segments.push(Vec::new());
                self.schedule(
                    tasks,
                    Task::ExitAggregate {
                        parent_segment: segment,
                        member_segment,
                        source,
                        grouping: *grouping,
                        reduction: *reduction,
                        outer_reduction: *outer_reduction,
                        static_preprocessing_work,
                        node_index,
                    },
                    node_index,
                )?;
                self.schedule(
                    tasks,
                    Task::Enter {
                        expr: member,
                        depth: depth + 1,
                        segment: member_segment,
                        inside_aggregate: true,
                    },
                    node_index,
                )?;
            }
        }
        Ok(())
    }

    fn exit_add(
        &mut self,
        segment: usize,
        start: usize,
        arity: usize,
        node_index: usize,
    ) -> Result<(), ExprDiagnostic> {
        let values = self.take_values(arity, node_index)?;
        let unit = values[0].unit.clone();
        if let Some(actual) = values.iter().find(|value| value.unit != unit) {
            return Err(self.error(
                ValueExprFaultCode::UnitMismatch,
                node_index,
                Some(unit),
                Some(actual.unit.clone()),
                "Add operands must have the same unit",
            ));
        }
        let constant = if values.iter().all(|value| value.constant.is_some()) {
            let mut result = 0_i128;
            for value in &values {
                result = result
                    .checked_add(value.constant.expect("all constants checked"))
                    .ok_or_else(|| {
                        self.error(
                            ValueExprFaultCode::Overflow,
                            node_index,
                            Some(unit.clone()),
                            Some(unit.clone()),
                            "constant Add overflowed i128",
                        )
                    })?;
            }
            Some(result)
        } else {
            None
        };
        let work = self.operation_work(&values, constant.is_some(), node_index)?;
        self.finish_op(segment, start, node_index, unit, constant, work, |unit| {
            CompiledOp::Add {
                arity,
                unit,
                node_index,
            }
        });
        Ok(())
    }

    fn exit_mul(
        &mut self,
        segment: usize,
        start: usize,
        node_index: usize,
    ) -> Result<(), ExprDiagnostic> {
        let values = self.take_values(2, node_index)?;
        let left = &values[0];
        let right = &values[1];
        let unit = match (&left.unit, &right.unit) {
            (ValueUnit::Scalar, unit) | (unit, ValueUnit::Scalar) => unit.clone(),
            _ => {
                return Err(self.error(
                    ValueExprFaultCode::UnitMismatch,
                    node_index,
                    Some(ValueUnit::Scalar),
                    Some(right.unit.clone()),
                    "Mul requires exactly one Scalar operand unless both operands are Scalar",
                ));
            }
        };
        let constant = match (left.constant, right.constant) {
            (Some(left), Some(right)) => Some(left.checked_mul(right).ok_or_else(|| {
                self.error(
                    ValueExprFaultCode::Overflow,
                    node_index,
                    Some(unit.clone()),
                    Some(unit.clone()),
                    "constant Mul overflowed i128",
                )
            })?),
            _ => None,
        };
        let work = self.operation_work(&values, constant.is_some(), node_index)?;
        self.finish_op(segment, start, node_index, unit, constant, work, |unit| {
            CompiledOp::Mul { unit, node_index }
        });
        Ok(())
    }

    fn exit_ratio(
        &mut self,
        segment: usize,
        start: usize,
        rounding: Rounding,
        node_index: usize,
    ) -> Result<(), ExprDiagnostic> {
        let values = self.take_values(2, node_index)?;
        let numerator = &values[0];
        let denominator = &values[1];
        let unit = if numerator.unit == denominator.unit {
            ValueUnit::Scalar
        } else if denominator.unit == ValueUnit::Scalar {
            numerator.unit.clone()
        } else {
            return Err(self.error(
                ValueExprFaultCode::UnitMismatch,
                node_index,
                Some(numerator.unit.clone()),
                Some(denominator.unit.clone()),
                "Ratio requires equal units or a Scalar denominator",
            ));
        };
        let constant = match (numerator.constant, denominator.constant) {
            (_, Some(0)) => {
                return Err(self.error(
                    ValueExprFaultCode::DivideByZero,
                    node_index,
                    Some(denominator.unit.clone()),
                    Some(denominator.unit.clone()),
                    "constant Ratio denominator is zero",
                ));
            }
            (Some(numerator), Some(denominator)) => Some(
                divide_rounded(numerator, denominator, rounding).ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        Some(unit.clone()),
                        Some(unit.clone()),
                        "constant Ratio overflowed i128",
                    )
                })?,
            ),
            _ => None,
        };
        let work = self.operation_work(&values, constant.is_some(), node_index)?;
        self.finish_op(segment, start, node_index, unit, constant, work, |unit| {
            CompiledOp::Ratio {
                rounding,
                unit,
                node_index,
            }
        });
        Ok(())
    }

    fn exit_clamp(
        &mut self,
        segment: usize,
        start: usize,
        node_index: usize,
    ) -> Result<(), ExprDiagnostic> {
        let values = self.take_values(3, node_index)?;
        let unit = values[0].unit.clone();
        if let Some(actual) = values.iter().find(|value| value.unit != unit) {
            return Err(self.error(
                ValueExprFaultCode::UnitMismatch,
                node_index,
                Some(unit),
                Some(actual.unit.clone()),
                "Clamp value, min and max must have the same unit",
            ));
        }
        if let (Some(min), Some(max)) = (values[1].constant, values[2].constant) {
            if min > max {
                return Err(self.error(
                    ValueExprFaultCode::InvalidClamp,
                    node_index,
                    Some(unit.clone()),
                    Some(unit.clone()),
                    "Clamp min is greater than max",
                ));
            }
        }
        let constant = match (values[0].constant, values[1].constant, values[2].constant) {
            (Some(value), Some(min), Some(max)) => Some(value.clamp(min, max)),
            _ => None,
        };
        let work = self.operation_work(&values, constant.is_some(), node_index)?;
        self.finish_op(segment, start, node_index, unit, constant, work, |unit| {
            CompiledOp::Clamp { unit, node_index }
        });
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn exit_aggregate(
        &mut self,
        parent_segment: usize,
        member_segment: usize,
        source: FiniteSource,
        grouping: Grouping,
        reduction: Reduction,
        outer_reduction: Option<Reduction>,
        static_preprocessing_work: Option<u64>,
        node_index: usize,
    ) -> Result<(), ExprDiagnostic> {
        let member = self.take_values(1, node_index)?.pop().expect("one value");
        match grouping {
            Grouping::WholeSet if outer_reduction.is_some() => {
                return Err(self.error(
                    ValueExprFaultCode::Unavailable,
                    node_index,
                    None,
                    None,
                    "WholeSet aggregate must not declare an outer reduction",
                ));
            }
            Grouping::ByOwner | Grouping::ByProjectedTile | Grouping::ByResolvedOwner(_)
                if outer_reduction.is_none() =>
            {
                return Err(self.error(
                    ValueExprFaultCode::Unavailable,
                    node_index,
                    None,
                    None,
                    "grouped aggregate requires an explicit outer reduction",
                ));
            }
            _ => {}
        }
        self.validate_authored_member_units(&source, member_segment, node_index)?;
        let inner_unit = reduction_unit(reduction, &member.unit);
        let result_unit = outer_reduction
            .map(|outer| reduction_unit(outer, &inner_unit))
            .unwrap_or_else(|| inner_unit.clone());
        let member_count_bound = match &source {
            FiniteSource::AuthoredMembers(members) => members.len(),
            FiniteSource::ResolvedTargets | FiniteSource::ResolvedTargetsWhere(_) | FiniteSource::StateOwnerDotTargets => {
                self.limits.max_dynamic_members
            }
        };
        let member_count = u64::try_from(member_count_bound).map_err(|_| {
            self.error(
                ValueExprFaultCode::Overflow,
                node_index,
                None,
                None,
                "aggregate member bound does not fit the work counter",
            )
        })?;
        let preprocessing = match static_preprocessing_work {
            Some(work) => work,
            None => {
                let one_member = aggregate_member_scan_work(
                    0,
                    0,
                    grouping_uses_owner_key(grouping).then_some(0),
                    0,
                    0,
                )
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "resolved member structural work overflowed",
                    )
                })?
                .checked_add(u64::try_from(source_predicate_count(&source)).map_err(|_| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "resolved predicate count does not fit the work counter",
                    )
                })?)
                .and_then(|work| {
                    work.checked_add(u64::from(matches!(grouping, Grouping::ByResolvedOwner(_))))
                })
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "resolved predicate/relation work overflowed",
                    )
                })?;
                member_count
                    .checked_add(one_member.checked_mul(member_count).ok_or_else(|| {
                        self.error(
                            ValueExprFaultCode::Overflow,
                            node_index,
                            None,
                            None,
                            "resolved member scan work overflowed",
                        )
                    })?)
                    .and_then(|work| {
                        work.checked_add(aggregate_sort_work(
                            member_count_bound,
                            0,
                            grouping_uses_owner_key(grouping).then_some(0),
                        )?)
                    })
                    .ok_or_else(|| {
                        self.error(
                            ValueExprFaultCode::Overflow,
                            node_index,
                            None,
                            None,
                            "resolved aggregate structural work overflowed",
                        )
                    })?
            }
        };
        let reduction_passes = if outer_reduction.is_some() {
            2_u64
        } else {
            1_u64
        };
        let work = member
            .work
            .checked_mul(member_count)
            .and_then(|work| work.checked_add(preprocessing))
            .and_then(|work| work.checked_add(reduction_passes.checked_mul(member_count)?))
            .and_then(|work| work.checked_add(1))
            .ok_or_else(|| {
                self.error(
                    ValueExprFaultCode::Overflow,
                    node_index,
                    None,
                    None,
                    "nested aggregate worst-case work overflowed",
                )
            })?;
        if work > self.limits.max_work {
            return Err(self.error(
                ValueExprFaultCode::BudgetExceeded,
                node_index,
                None,
                None,
                format!(
                    "aggregate worst-case work {work} exceeds limit {}",
                    self.limits.max_work
                ),
            ));
        }
        self.segments[parent_segment].push(CompiledOp::Aggregate {
            source,
            grouping,
            reduction,
            outer_reduction,
            member_segment,
            member_unit: member.unit,
            result_unit: result_unit.clone(),
            node_index,
        });
        self.values.push(CompileValue {
            unit: result_unit,
            constant: None,
            work,
        });
        Ok(())
    }

    fn validate_authored_member_units(
        &mut self,
        source: &FiniteSource,
        member_segment: usize,
        node_index: usize,
    ) -> Result<(), ExprDiagnostic> {
        let FiniteSource::AuthoredMembers(members) = source else {
            return Ok(());
        };
        let Some(first_member) = members.first() else {
            return Ok(());
        };
        let operation_count = self.segments[member_segment].len();
        let scan_work = u64::try_from(operation_count).map_err(|_| {
            self.error(
                ValueExprFaultCode::Overflow,
                node_index,
                None,
                None,
                "aggregate member unit scan does not fit the work counter",
            )
        })?;
        self.charge_compile_work(scan_work, node_index, "aggregate member unit scan")?;

        // Child aggregate reads belong to their own member segment. Checking the
        // first payload against every direct read also detects conflicting read
        // units without a read-by-member scan or a collection of cloned units.
        let mut witness: Option<usize> = None;
        for operation_index in 0..operation_count {
            let Some((_, read_node_index)) =
                self.direct_member_read(member_segment, operation_index)
            else {
                continue;
            };
            if witness.is_none() {
                self.charge_compile_work(1, read_node_index, "authored member unit scan")?;
            }
            let (unit, _) = self
                .direct_member_read(member_segment, operation_index)
                .expect("selected direct member read");
            let comparison_bytes = value_unit_dynamic_bytes(unit)
                .checked_add(typed_integer_dynamic_bytes(&first_member.value))
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        read_node_index,
                        None,
                        None,
                        "authored member unit comparison accounting overflowed",
                    )
                })?;
            self.charge_text_work(
                comparison_bytes,
                read_node_index,
                "authored member unit comparison",
            )?;
            let (unit, _) = self
                .direct_member_read(member_segment, operation_index)
                .expect("selected direct member read");
            if unit != &first_member.value.unit {
                return Err(self.error(
                    ValueExprFaultCode::UnitMismatch,
                    read_node_index,
                    Some(bounded_value_unit(unit)),
                    Some(bounded_value_unit(&first_member.value.unit)),
                    "authored aggregate payload unit differs from its direct member read",
                ));
            }
            witness.get_or_insert(operation_index);
        }
        let Some(witness) = witness else {
            // The member expression does not read its authored payload.
            return Ok(());
        };
        let (_, read_node_index) = self
            .direct_member_read(member_segment, witness)
            .expect("selected direct member read");
        for member in members.iter().skip(1) {
            self.charge_compile_work(1, read_node_index, "authored member unit scan")?;
            let (unit, _) = self
                .direct_member_read(member_segment, witness)
                .expect("selected direct member read");
            let comparison_bytes = value_unit_dynamic_bytes(unit)
                .checked_add(typed_integer_dynamic_bytes(&member.value))
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        read_node_index,
                        None,
                        None,
                        "authored member unit comparison accounting overflowed",
                    )
                })?;
            self.charge_text_work(
                comparison_bytes,
                read_node_index,
                "authored member unit comparison",
            )?;
            let (unit, _) = self
                .direct_member_read(member_segment, witness)
                .expect("selected direct member read");
            if unit != &member.value.unit {
                return Err(self.error(
                    ValueExprFaultCode::UnitMismatch,
                    read_node_index,
                    Some(bounded_value_unit(unit)),
                    Some(bounded_value_unit(&member.value.unit)),
                    "authored aggregate payload unit differs from its direct member read",
                ));
            }
        }
        Ok(())
    }

    fn direct_member_read(
        &self,
        member_segment: usize,
        operation_index: usize,
    ) -> Option<(&ValueUnit, usize)> {
        match &self.segments[member_segment][operation_index] {
            CompiledOp::Read {
                reference: ValueRef::AggregateMember { unit },
                node_index,
            } => Some((unit, *node_index)),
            _ => None,
        }
    }

    fn validate_reference(
        &self,
        reference: &ValueRef,
        inside_aggregate: bool,
        node_index: usize,
    ) -> Result<(), ExprDiagnostic> {
        if matches!(reference, ValueRef::StateField { .. } | ValueRef::EventHpLost)
            && !self.capabilities.state_context
        {
            return Err(self.error(
                ValueExprFaultCode::MissingContext, node_index, None,
                Some(reference.unit()), "state-program context is unavailable",
            ));
        }
        if matches!(reference, ValueRef::RelationCount { subject: ValueSubject::AggregateMember, .. }) {
            return Err(self.error(ValueExprFaultCode::MissingContext, node_index, None, Some(reference.unit()), "relation counts require Source or Target, not aggregate members"));
        }
        if reference.is_member_scoped() && !inside_aggregate {
            return Err(self.error(
                ValueExprFaultCode::MissingContext,
                node_index,
                None,
                Some(reference.unit()),
                "aggregate-member reference is outside an Aggregate member expression",
            ));
        }
        if matches!(reference, ValueRef::Resource { .. }) && !self.capabilities.resources {
            return Err(self.error(
                ValueExprFaultCode::MissingContext,
                node_index,
                None,
                Some(reference.unit()),
                "resource read capability is unavailable",
            ));
        }
        let requires_target = matches!(
            reference,
            ValueRef::RelationCount { subject: ValueSubject::Target, .. } |
            ValueRef::Stat {
                subject: ValueSubject::Target,
                ..
            } | ValueRef::HitPoints {
                subject: ValueSubject::Target,
                ..
            } | ValueRef::Resource {
                owner: ResourceOwnerSelector::TargetEmployee,
                ..
            }
        );
        if requires_target {
            let supported = match self.expression.moment {
                EvalMoment::Snapshot => self.capabilities.target_at_release,
                EvalMoment::Live => self.capabilities.target_at_consume,
            };
            if !supported {
                return Err(self.error(
                    ValueExprFaultCode::UnsupportedMoment,
                    node_index,
                    None,
                    Some(reference.unit()),
                    "target context is unavailable at the selected evaluation moment",
                ));
            }
        }
        match reference {
            ValueRef::Result { .. } if !self.capabilities.result_prefix => {
                return Err(self.error(
                    ValueExprFaultCode::NotSettled,
                    node_index,
                    None,
                    Some(reference.unit()),
                    "completed-effect prefix capability is unavailable",
                ));
            }
            ValueRef::Time(super::TimeValue::RemainingTime) if !self.capabilities.deadline => {
                return Err(self.error(
                    ValueExprFaultCode::MissingContext,
                    node_index,
                    None,
                    Some(ValueUnit::Milliseconds),
                    "RemainingTime requires an explicit deadline capability",
                ));
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_aggregate_capabilities(
        &mut self,
        source: &FiniteSource,
        grouping: Grouping,
        node_index: usize,
    ) -> Result<(), ExprDiagnostic> {
        if matches!(source, FiniteSource::StateOwnerDotTargets) {
            if !self.capabilities.state_context || self.expression.moment != EvalMoment::Live {
                return Err(self.error(ValueExprFaultCode::UnsupportedMoment, node_index, None, None,
                    "StateOwnerDotTargets requires a Live State context"));
            }
            if grouping != Grouping::WholeSet {
                return Err(self.error(ValueExprFaultCode::MissingContext, node_index, None, None,
                    "StateOwnerDotTargets supports WholeSet membership only"));
            }
            self.state_owner_dot_targets = true;
        }
        if is_resolved_source(source) {
            let supported = match self.expression.moment {
                EvalMoment::Snapshot => self.capabilities.resolved_targets_at_release,
                EvalMoment::Live => self.capabilities.resolved_targets_at_consume,
            };
            if !supported {
                return Err(self.error(
                    ValueExprFaultCode::UnsupportedMoment,
                    node_index,
                    None,
                    None,
                    "resolved targets are unavailable at the selected evaluation moment",
                ));
            }
            self.resolved_targets = true;
        }
        if let FiniteSource::AuthoredMembers(members) = source {
            match grouping {
                Grouping::ByOwner
                    if members.iter().any(|member| {
                        member
                            .owner
                            .as_ref()
                            .is_none_or(|owner| owner.identity.trim().is_empty())
                    }) =>
                {
                    return Err(self.error(
                        ValueExprFaultCode::InvalidOwner,
                        node_index,
                        None,
                        None,
                        "every authored member requires an owner for ByOwner grouping",
                    ));
                }
                Grouping::ByOwner if mixed_owner_domains(members) => {
                    return Err(self.error(
                        ValueExprFaultCode::InvalidOwner,
                        node_index,
                        None,
                        None,
                        "one aggregate cannot mix owner grouping domains",
                    ));
                }
                Grouping::ByProjectedTile
                    if members.iter().any(|member| member.projected_tile.is_none()) =>
                {
                    return Err(self.error(
                        ValueExprFaultCode::MissingContext,
                        node_index,
                        None,
                        None,
                        "every authored member requires a projected tile for ByProjectedTile grouping",
                    ));
                }
                Grouping::ByResolvedOwner(_) => {
                    return Err(self.error(
                        ValueExprFaultCode::InvalidOwner,
                        node_index,
                        None,
                        None,
                        "ByResolvedOwner requires a resolved-target source",
                    ));
                }
                _ => {}
            }
        }
        match grouping {
            Grouping::WholeSet => {}
            Grouping::ByOwner if self.capabilities.owner_grouping => {
                self.owner_grouping = true;
            }
            Grouping::ByProjectedTile if self.capabilities.projected_tile_grouping => {
                self.projected_tile_grouping = true;
            }
            Grouping::ByResolvedOwner(relation) if self.capabilities.owner_grouping => {
                validate_resolved_relation(relation).map_err(|cause| {
                    self.error(
                        ValueExprFaultCode::InvalidOwner,
                        node_index,
                        None,
                        None,
                        cause,
                    )
                })?;
                self.owner_grouping = true;
                self.resolved_owner_relations.insert(relation);
            }
            Grouping::ByOwner => {
                return Err(self.error(
                    ValueExprFaultCode::MissingContext,
                    node_index,
                    None,
                    None,
                    "owner grouping capability is unavailable",
                ));
            }
            Grouping::ByProjectedTile => {
                return Err(self.error(
                    ValueExprFaultCode::MissingContext,
                    node_index,
                    None,
                    None,
                    "projected-tile grouping capability is unavailable",
                ));
            }
            Grouping::ByResolvedOwner(_) => {
                return Err(self.error(
                    ValueExprFaultCode::MissingContext,
                    node_index,
                    None,
                    None,
                    "resolved-owner grouping capability is unavailable",
                ));
            }
        }
        Ok(())
    }

    fn admit_finite_source(
        &mut self,
        source: &FiniteSource,
        grouping: Grouping,
        predicate_depth: usize,
        node_index: usize,
    ) -> Result<Option<u64>, ExprDiagnostic> {
        if let FiniteSource::ResolvedTargetsWhere(predicates) = source {
            if predicate_depth > self.limits.max_depth {
                return Err(self.error(
                    ValueExprFaultCode::BudgetExceeded,
                    node_index,
                    None,
                    None,
                    format!(
                        "resolved predicate depth {predicate_depth} exceeds limit {}",
                        self.limits.max_depth
                    ),
                ));
            }
            let next_admitted_nodes = self
                .admitted_nodes
                .checked_add(predicates.len())
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "resolved predicate node count overflowed",
                    )
                })?;
            if next_admitted_nodes > self.limits.max_nodes {
                return Err(self.error(
                    ValueExprFaultCode::BudgetExceeded,
                    node_index,
                    None,
                    None,
                    format!(
                        "expression node count {next_admitted_nodes} exceeds limit {}",
                        self.limits.max_nodes
                    ),
                ));
            }
            for predicate in predicates {
                validate_resolved_predicate(*predicate).map_err(|cause| {
                    self.error(
                        ValueExprFaultCode::InvalidOwner,
                        node_index,
                        None,
                        None,
                        cause,
                    )
                })?;
                self.resolved_target_predicates.insert(*predicate);
            }
            self.admitted_nodes = next_admitted_nodes;
            let canonical_predicate_count = predicates
                .iter()
                .enumerate()
                .filter(|(index, predicate)| !predicates[..*index].contains(predicate))
                .count();
            self.nodes = self
                .nodes
                .checked_add(canonical_predicate_count)
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "canonical resolved predicate node count overflowed",
                    )
                })?;
            if !predicates.is_empty() {
                self.max_depth_seen = self.max_depth_seen.max(predicate_depth);
            }
            let count = u64::try_from(predicates.len()).map_err(|_| {
                self.error(
                    ValueExprFaultCode::Overflow,
                    node_index,
                    None,
                    None,
                    "resolved predicate count does not fit the work counter",
                )
            })?;
            self.charge_compile_work(count, node_index, "resolved predicate scan")?;
            let storage = predicates
                .len()
                .checked_mul(size_of::<ResolvedTargetPredicate>())
                .and_then(|bytes| bytes.checked_mul(2))
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "resolved predicate storage accounting overflowed",
                    )
                })?;
            self.admit_plan_storage(storage, node_index, "compiled resolved predicates")?;
            return Ok(None);
        }
        let FiniteSource::AuthoredMembers(members) = source else {
            return Ok(None);
        };
        let count = members.len();
        self.authored_members = self.authored_members.checked_add(count).ok_or_else(|| {
            self.error(
                ValueExprFaultCode::Overflow,
                node_index,
                None,
                None,
                "authored member count overflowed",
            )
        })?;
        if self.authored_members > self.limits.max_dynamic_members {
            return Err(self.error(
                ValueExprFaultCode::BudgetExceeded,
                node_index,
                None,
                None,
                format!(
                    "authored members {} exceed limit {}",
                    self.authored_members, self.limits.max_dynamic_members
                ),
            ));
        }

        let mut preprocessing_work = u64::try_from(count).map_err(|_| {
            self.error(
                ValueExprFaultCode::Overflow,
                node_index,
                None,
                None,
                "authored member scan count does not fit the work counter",
            )
        })?;

        let mut identity_bytes = 0_usize;
        let mut owner_key_bytes = 0_usize;
        let mut dynamic_bytes = 0_usize;
        for member in members {
            identity_bytes = identity_bytes
                .checked_add(member.identity.len())
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "authored identity payload accounting overflowed",
                    )
                })?;
            dynamic_bytes = dynamic_bytes
                .checked_add(member.identity.len())
                .and_then(|bytes| {
                    bytes.checked_add(
                        member
                            .owner
                            .as_ref()
                            .map_or(0, |owner| owner.identity.len()),
                    )
                })
                .and_then(|bytes| bytes.checked_add(typed_integer_dynamic_bytes(&member.value)))
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "authored member payload accounting overflowed",
                    )
                })?;
            preprocessing_work = preprocessing_work
                .checked_add(
                    aggregate_member_scan_work(
                        member.identity.len(),
                        typed_integer_dynamic_bytes(&member.value),
                        member.owner.as_ref().map(|owner| owner.identity.len()),
                        0,
                        0,
                    )
                    .ok_or_else(|| {
                        self.error(
                            ValueExprFaultCode::Overflow,
                            node_index,
                            None,
                            None,
                            "authored member work accounting overflowed",
                        )
                    })?,
                )
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "authored member work accounting overflowed",
                    )
                })?;
            if grouping == Grouping::ByOwner {
                owner_key_bytes = owner_key_bytes
                    .checked_add(
                        member
                            .owner
                            .as_ref()
                            .map_or(0, |owner| owner.identity.len()),
                    )
                    .ok_or_else(|| {
                        self.error(
                            ValueExprFaultCode::Overflow,
                            node_index,
                            None,
                            None,
                            "authored owner-key payload accounting overflowed",
                        )
                    })?;
            }
        }
        preprocessing_work = preprocessing_work
            .checked_add(
                aggregate_sort_work(
                    count,
                    identity_bytes,
                    (grouping == Grouping::ByOwner).then_some(owner_key_bytes),
                )
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "authored member sort work accounting overflowed",
                    )
                })?,
            )
            .ok_or_else(|| {
                self.error(
                    ValueExprFaultCode::Overflow,
                    node_index,
                    None,
                    None,
                    "authored preprocessing work accounting overflowed",
                )
            })?;
        self.charge_compile_work(
            preprocessing_work,
            node_index,
            "authored source preprocessing",
        )?;
        for member in members {
            if member.identity.trim().is_empty() {
                return Err(self.error(
                    ValueExprFaultCode::InvalidOwner,
                    node_index,
                    None,
                    None,
                    "authored aggregate member identity must not be empty",
                ));
            }
        }

        let member_slots = count
            .checked_mul(size_of::<AuthoredMember>())
            .and_then(|bytes| bytes.checked_mul(2))
            .ok_or_else(|| {
                self.error(
                    ValueExprFaultCode::Overflow,
                    node_index,
                    None,
                    None,
                    "authored member container accounting overflowed",
                )
            })?;
        let payload = dynamic_bytes.checked_mul(2).ok_or_else(|| {
            self.error(
                ValueExprFaultCode::Overflow,
                node_index,
                None,
                None,
                "authored member retained payload accounting overflowed",
            )
        })?;
        self.admit_plan_storage(
            member_slots.checked_add(payload).ok_or_else(|| {
                self.error(
                    ValueExprFaultCode::Overflow,
                    node_index,
                    None,
                    None,
                    "authored source retained storage accounting overflowed",
                )
            })?,
            node_index,
            "compiled authored source",
        )?;
        Ok(Some(preprocessing_work))
    }

    fn normalize_source(
        &mut self,
        source: &FiniteSource,
        node_index: usize,
    ) -> Result<FiniteSource, ExprDiagnostic> {
        match source {
            FiniteSource::StateOwnerDotTargets => Ok(FiniteSource::StateOwnerDotTargets),
            FiniteSource::ResolvedTargets => Ok(FiniteSource::ResolvedTargets),
            FiniteSource::ResolvedTargetsWhere(predicates) => {
                let mut normalized = predicates.clone();
                normalized.sort_unstable();
                normalized.dedup();
                if normalized.is_empty() {
                    Ok(FiniteSource::ResolvedTargets)
                } else {
                    Ok(FiniteSource::ResolvedTargetsWhere(normalized))
                }
            }
            FiniteSource::AuthoredMembers(members) => {
                let mut normalized = members.clone();
                normalized.sort_by(|left, right| left.identity.cmp(&right.identity));
                if normalized
                    .windows(2)
                    .any(|pair| pair[0].identity == pair[1].identity)
                {
                    return Err(self.error(
                        ValueExprFaultCode::InvalidOwner,
                        node_index,
                        None,
                        None,
                        "authored aggregate member identities must be unique",
                    ));
                }
                Ok(FiniteSource::AuthoredMembers(normalized))
            }
        }
    }

    fn take_values(
        &mut self,
        count: usize,
        node_index: usize,
    ) -> Result<Vec<CompileValue>, ExprDiagnostic> {
        let start = self.values.len().checked_sub(count).ok_or_else(|| {
            self.error(
                ValueExprFaultCode::Unavailable,
                node_index,
                None,
                None,
                "compiler value stack underflow",
            )
        })?;
        self.admit_working_storage(
            count
                .checked_mul(size_of::<CompileValue>())
                .and_then(|bytes| bytes.checked_mul(2))
                .ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        node_index,
                        None,
                        None,
                        "compiler operand scratch accounting overflowed",
                    )
                })?,
            node_index,
            "compiler operand scratch",
        )?;
        Ok(self.values.split_off(start))
    }

    fn operation_work(
        &self,
        values: &[CompileValue],
        folded: bool,
        node_index: usize,
    ) -> Result<u64, ExprDiagnostic> {
        if folded {
            return Ok(1);
        }
        let mut work = 1_u64;
        for value in values {
            work = work.checked_add(value.work).ok_or_else(|| {
                self.error(
                    ValueExprFaultCode::Overflow,
                    node_index,
                    None,
                    None,
                    "expression worst-case work overflowed",
                )
            })?;
        }
        Ok(work)
    }

    fn finish_op(
        &mut self,
        segment: usize,
        start: usize,
        node_index: usize,
        unit: ValueUnit,
        constant: Option<i128>,
        work: u64,
        dynamic: impl FnOnce(ValueUnit) -> CompiledOp,
    ) {
        if let Some(value) = constant {
            self.segments[segment].truncate(start);
            self.segments[segment].push(CompiledOp::PushConst {
                value: TypedInteger::new(value, unit.clone()),
                node_index,
            });
        } else {
            self.segments[segment].push(dynamic(unit.clone()));
        }
        self.values.push(CompileValue {
            unit,
            constant,
            work,
        });
    }

    fn execution_shape(&self) -> Result<ExecutionShape, ExprDiagnostic> {
        if self.segments.is_empty() {
            return Err(self.error(
                ValueExprFaultCode::Unavailable,
                0,
                None,
                None,
                "compiled aggregate segment graph is invalid",
            ));
        }
        let mut shapes: Vec<Option<ExecutionShape>> = vec![None; self.segments.len()];
        for segment_index in (0..self.segments.len()).rev() {
            let mut local_depth = 0_usize;
            let mut local_maximum = 0_usize;
            let mut maximum = 0_usize;
            let mut max_task_frames = 1_usize;
            let mut child_storage_slots = 0_usize;
            for op in &self.segments[segment_index] {
                let (pop, push) = match op {
                    CompiledOp::PushConst { .. } | CompiledOp::Read { .. } => (0, 1),
                    CompiledOp::Add { arity, .. } => (*arity, 1),
                    CompiledOp::Mul { .. } | CompiledOp::Ratio { .. } => (2, 1),
                    CompiledOp::Clamp { .. } => (3, 1),
                    CompiledOp::Aggregate { member_segment, .. } => {
                        let child = shapes
                            .get(*member_segment)
                            .and_then(|shape| *shape)
                            .ok_or_else(|| {
                                self.error(
                                    ValueExprFaultCode::Unavailable,
                                    op.node_index(),
                                    None,
                                    None,
                                    "compiled aggregate segment dependency is invalid",
                                )
                            })?;
                        maximum = maximum.max(
                            local_depth.checked_add(child.max_stack).ok_or_else(|| {
                                self.error(
                                    ValueExprFaultCode::Overflow,
                                    op.node_index(),
                                    None,
                                    None,
                                    "nested compiled stack requirement overflowed",
                                )
                            })?,
                        );
                        max_task_frames = max_task_frames.max(
                            child.max_task_frames.checked_add(2).ok_or_else(|| {
                                self.error(
                                    ValueExprFaultCode::Overflow,
                                    op.node_index(),
                                    None,
                                    None,
                                    "nested task-frame requirement overflowed",
                                )
                            })?,
                        );
                        child_storage_slots = child_storage_slots.max(child.stack_storage_slots);
                        (0, 1)
                    }
                };
                local_depth = local_depth.checked_sub(pop).ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Unavailable,
                        op.node_index(),
                        None,
                        None,
                        "compiled segment stack underflow",
                    )
                })?;
                local_depth = local_depth.checked_add(push).ok_or_else(|| {
                    self.error(
                        ValueExprFaultCode::Overflow,
                        op.node_index(),
                        None,
                        None,
                        "compiled segment stack overflow",
                    )
                })?;
                local_maximum = local_maximum.max(local_depth);
                maximum = maximum.max(local_depth);
            }
            if local_depth != 1 {
                return Err(self.error(
                    ValueExprFaultCode::Unavailable,
                    0,
                    None,
                    None,
                    "compiled segment does not leave exactly one value",
                ));
            }
            shapes[segment_index] = Some(ExecutionShape {
                max_stack: maximum,
                stack_storage_slots: local_maximum.checked_add(child_storage_slots).ok_or_else(
                    || {
                        self.error(
                            ValueExprFaultCode::Overflow,
                            0,
                            None,
                            None,
                            "stack storage requirement overflowed",
                        )
                    },
                )?,
                max_task_frames,
            });
        }
        shapes[0].ok_or_else(|| {
            self.error(
                ValueExprFaultCode::Unavailable,
                0,
                None,
                None,
                "compiled root segment shape is unavailable",
            )
        })
    }

    fn error(
        &self,
        code: ValueExprFaultCode,
        node_index: usize,
        expected_unit: Option<ValueUnit>,
        actual_unit: Option<ValueUnit>,
        cause: impl Into<String>,
    ) -> ExprDiagnostic {
        ExprDiagnostic {
            code,
            site: bounded_effect_site(&self.source.site),
            node_index,
            expected_unit: expected_unit.as_ref().map(bounded_value_unit),
            actual_unit: actual_unit.as_ref().map(bounded_value_unit),
            phase: ValueExprPhase::Compile,
            cause: cause.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ExecutionShape {
    max_stack: usize,
    stack_storage_slots: usize,
    max_task_frames: usize,
}

const DIAGNOSTIC_TEXT_LIMIT: usize = 128;

pub(crate) fn bounded_text(value: &str) -> String {
    if value.len() <= DIAGNOSTIC_TEXT_LIMIT {
        value.to_string()
    } else {
        let mut end = (DIAGNOSTIC_TEXT_LIMIT / 2).min(value.len());
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        let mut bounded = value[..end].to_string();
        bounded.push_str("...[bounded]");
        bounded
    }
}

pub(crate) fn bounded_value_unit(unit: &ValueUnit) -> ValueUnit {
    match unit {
        ValueUnit::Resource(resource) => ValueUnit::Resource(
            ResourceId::new(bounded_text(resource.as_str()))
                .expect("a bounded valid resource id remains valid"),
        ),
        _ => unit.clone(),
    }
}

pub(crate) fn bounded_effect_site(site: &EffectSite) -> EffectSite {
    EffectSite {
        action: ActionSite {
            definition_id: SkillId::new(bounded_text(site.action.definition_id.as_str())),
            action_id: bounded_text(&site.action.action_id),
        },
        effect_id: bounded_text(&site.effect_id),
    }
}

fn reduction_unit(reduction: Reduction, member_unit: &ValueUnit) -> ValueUnit {
    match reduction {
        Reduction::Count => ValueUnit::Count,
        Reduction::Sum | Reduction::Max => member_unit.clone(),
    }
}

pub(crate) fn divide_rounded(
    numerator: i128,
    denominator: i128,
    rounding: Rounding,
) -> Option<i128> {
    let quotient = numerator.checked_div(denominator)?;
    let remainder = numerator.checked_rem(denominator)?;
    if remainder == 0 || matches!(rounding, Rounding::TowardZero) {
        return Some(quotient);
    }
    let direction = if (numerator < 0) == (denominator < 0) {
        1_i128
    } else {
        -1_i128
    };
    match rounding {
        Rounding::TowardZero => Some(quotient),
        Rounding::Floor if direction < 0 => quotient.checked_sub(1),
        Rounding::Floor => Some(quotient),
        Rounding::Ceil if direction > 0 => quotient.checked_add(1),
        Rounding::Ceil => Some(quotient),
        Rounding::NearestTiesEven => {
            let remainder_abs = remainder.unsigned_abs();
            let denominator_abs = denominator.unsigned_abs();
            let other_half = denominator_abs.checked_sub(remainder_abs)?;
            if remainder_abs < other_half
                || (remainder_abs == other_half && quotient.rem_euclid(2) == 0)
            {
                Some(quotient)
            } else if direction > 0 {
                quotient.checked_add(1)
            } else {
                quotient.checked_sub(1)
            }
        }
    }
}

fn canonical_plan_digest(
    moment: EvalMoment,
    provenance: AmountProvenance,
    site: &EffectSite,
    segments: &[Vec<CompiledOp>],
    result_unit: &ValueUnit,
) -> [u8; 32] {
    let mut output = Sha256::new();
    output.update(b"game_core.value_expr.plan\0");
    put_u64(&mut output, u64::from(VALUE_EXPR_COMPILER_VERSION));
    put_u8(&mut output, moment as u8);
    put_u8(&mut output, provenance as u8);
    put_str(&mut output, site.action.definition_id.as_str());
    put_str(&mut output, &site.action.action_id);
    put_str(&mut output, &site.effect_id);
    put_unit(&mut output, result_unit);
    put_u64(&mut output, segments.len() as u64);
    for segment in segments {
        put_u64(&mut output, segment.len() as u64);
        for op in segment {
            put_op(&mut output, op);
        }
    }
    output.finalize().into()
}

fn put_op(output: &mut Sha256, op: &CompiledOp) {
    match op {
        CompiledOp::PushConst { value, node_index } => {
            put_u8(output, 0);
            put_typed(output, value);
            put_u64(output, *node_index as u64);
        }
        CompiledOp::Read {
            reference,
            node_index,
        } => {
            put_u8(output, 1);
            put_reference(output, reference);
            put_u64(output, *node_index as u64);
        }
        CompiledOp::Add {
            arity,
            unit,
            node_index,
        } => {
            put_u8(output, 2);
            put_u64(output, *arity as u64);
            put_unit(output, unit);
            put_u64(output, *node_index as u64);
        }
        CompiledOp::Mul { unit, node_index } => {
            put_u8(output, 3);
            put_unit(output, unit);
            put_u64(output, *node_index as u64);
        }
        CompiledOp::Ratio {
            rounding,
            unit,
            node_index,
        } => {
            put_u8(output, 4);
            put_u8(output, *rounding as u8);
            put_unit(output, unit);
            put_u64(output, *node_index as u64);
        }
        CompiledOp::Clamp { unit, node_index } => {
            put_u8(output, 5);
            put_unit(output, unit);
            put_u64(output, *node_index as u64);
        }
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
            put_u8(output, 6);
            put_source(output, source);
            put_grouping(output, *grouping);
            put_u8(output, *reduction as u8);
            put_option_tag(output, outer_reduction.is_some());
            if let Some(reduction) = outer_reduction {
                put_u8(output, *reduction as u8);
            }
            put_u64(output, *member_segment as u64);
            put_unit(output, member_unit);
            put_unit(output, result_unit);
            put_u64(output, *node_index as u64);
        }
    }
}

fn put_source(output: &mut Sha256, source: &FiniteSource) {
    match source {
        FiniteSource::AuthoredMembers(members) => {
            put_u8(output, 0);
            put_u64(output, members.len() as u64);
            for member in members {
                put_member(output, member);
            }
        }
        FiniteSource::ResolvedTargets => put_u8(output, 1),
        FiniteSource::StateOwnerDotTargets => put_u8(output, 3),
        FiniteSource::ResolvedTargetsWhere(predicates) => {
            put_u8(output, 2);
            put_u64(output, predicates.len() as u64);
            for predicate in predicates {
                put_predicate(output, *predicate);
            }
        }
    }
}

fn put_grouping(output: &mut Sha256, grouping: Grouping) {
    match grouping {
        Grouping::WholeSet => put_u8(output, 0),
        Grouping::ByOwner => put_u8(output, 1),
        Grouping::ByProjectedTile => put_u8(output, 2),
        Grouping::ByResolvedOwner(relation) => {
            put_u8(output, 3);
            put_resolved_relation(output, relation);
        }
    }
}

fn put_resolved_relation(output: &mut Sha256, relation: ResolvedOwnerRelation) {
    match relation {
        ResolvedOwnerRelation::Side => put_u8(output, 0),
        ResolvedOwnerRelation::ActualEmployee => put_u8(output, 1),
        ResolvedOwnerRelation::EquippedSlot(address) => {
            put_u8(output, 2);
            put_slot_address(output, address);
        }
    }
}

fn put_predicate(output: &mut Sha256, predicate: ResolvedTargetPredicate) {
    match predicate {
        ResolvedTargetPredicate::IsActualEmployee => put_u8(output, 0),
        ResolvedTargetPredicate::HasEquippedSlot(address) => {
            put_u8(output, 1);
            put_slot_address(output, address);
        }
    }
}

fn put_slot_address(
    output: &mut Sha256,
    address: crate::game::resources::item_slot::EquipmentSlotAddress,
) {
    use crate::game::resources::item_slot::EquipmentSlotAddress;
    match address {
        EquipmentSlotAddress::Weapon => put_u8(output, 0),
        EquipmentSlotAddress::Armor => put_u8(output, 1),
        EquipmentSlotAddress::Accessory => put_u8(output, 2),
        EquipmentSlotAddress::Any3(index) => {
            put_u8(output, 3);
            put_u8(output, index);
        }
    }
}

fn put_member(output: &mut Sha256, member: &AuthoredMember) {
    put_str(output, &member.identity);
    put_typed(output, &member.value);
    put_option_tag(output, member.owner.is_some());
    if let Some(owner) = &member.owner {
        put_u8(
            output,
            match owner.domain {
                GroupOwnerDomain::Side => 0,
                GroupOwnerDomain::Employee => 1,
                GroupOwnerDomain::EquipmentCopy => 2,
            },
        );
        put_str(output, &owner.identity);
    }
    put_option_tag(output, member.projected_tile.is_some());
    if let Some(tile) = member.projected_tile {
        output.update(tile.x.to_be_bytes());
        output.update(tile.y.to_be_bytes());
    }
}

fn mixed_owner_domains(members: &[AuthoredMember]) -> bool {
    let mut domains = members
        .iter()
        .filter_map(|member| member.owner.as_ref().map(|owner| owner.domain));
    let Some(first) = domains.next() else {
        return false;
    };
    domains.any(|domain| domain != first)
}

fn is_resolved_source(source: &FiniteSource) -> bool {
    matches!(
        source,
        FiniteSource::ResolvedTargets | FiniteSource::ResolvedTargetsWhere(_)
    )
}

fn source_predicate_count(source: &FiniteSource) -> usize {
    match source {
        FiniteSource::ResolvedTargetsWhere(predicates) => predicates.len(),
        FiniteSource::AuthoredMembers(_) | FiniteSource::ResolvedTargets | FiniteSource::StateOwnerDotTargets => 0,
    }
}

fn grouping_uses_owner_key(grouping: Grouping) -> bool {
    matches!(grouping, Grouping::ByOwner | Grouping::ByResolvedOwner(_))
}

fn validate_resolved_relation(relation: ResolvedOwnerRelation) -> Result<(), &'static str> {
    match relation {
        ResolvedOwnerRelation::EquippedSlot(address) if !address.is_valid() => {
            Err("resolved owner relation has an out-of-range Any3 slot address")
        }
        ResolvedOwnerRelation::Side
        | ResolvedOwnerRelation::ActualEmployee
        | ResolvedOwnerRelation::EquippedSlot(_) => Ok(()),
    }
}

fn validate_resolved_predicate(predicate: ResolvedTargetPredicate) -> Result<(), &'static str> {
    match predicate {
        ResolvedTargetPredicate::HasEquippedSlot(address) if !address.is_valid() => {
            Err("resolved target predicate has an out-of-range Any3 slot address")
        }
        ResolvedTargetPredicate::IsActualEmployee | ResolvedTargetPredicate::HasEquippedSlot(_) => {
            Ok(())
        }
    }
}

fn put_typed(output: &mut Sha256, value: &TypedInteger) {
    output.update(value.value.to_be_bytes());
    put_unit(output, &value.unit);
}

fn put_unit(output: &mut Sha256, unit: &ValueUnit) {
    match unit {
        ValueUnit::Magnitude => put_u8(output, 0),
        ValueUnit::Scalar => put_u8(output, 1),
        ValueUnit::Count => put_u8(output, 2),
        ValueUnit::Milliseconds => put_u8(output, 3),
        ValueUnit::AttackSpeedPoints => put_u8(output, 4),
        ValueUnit::Resource(resource) => {
            put_u8(output, 5);
            put_str(output, resource.as_str());
        }
    }
}

fn put_reference(output: &mut Sha256, reference: &ValueRef) {
    match reference {
        ValueRef::StateField { name, unit } => {
            put_u8(output, 7);
            put_str(output, name);
            put_unit(output, unit);
        }
        ValueRef::EventHpLost => put_u8(output, 8),
        ValueRef::RelationCount { subject, relation } => {
            put_u8(output, 9);
            put_subject(output, *subject);
            match relation { super::RelationCountKind::BlockingHostiles => put_u8(output, 0) }
        }
        ValueRef::Stat { subject, stat } => {
            put_u8(output, 0);
            put_subject(output, *subject);
            put_u8(
                output,
                match stat {
                    StatSelector::Attack => 0,
                    StatSelector::SpellPower => 1,
                    StatSelector::AttackSpeedPoints => 2,
                    StatSelector::Defense => 3,
                },
            );
        }
        ValueRef::HitPoints { subject, field } => {
            put_u8(output, 1);
            put_subject(output, *subject);
            put_u8(
                output,
                match field {
                    HitPointField::Current => 0,
                    HitPointField::Max => 1,
                    HitPointField::Missing => 2,
                },
            );
        }
        ValueRef::Resource {
            resource,
            owner,
            field,
        } => {
            put_u8(output, 2);
            put_str(output, resource.as_str());
            put_u8(
                output,
                match owner {
                    ResourceOwnerSelector::SourceEmployee => 0,
                    ResourceOwnerSelector::SourceEquipment => 1,
                    ResourceOwnerSelector::SourceFragment => 3,
                    ResourceOwnerSelector::TargetEmployee => 2,
                },
            );
            put_u8(
                output,
                match field {
                    ResourceValueField::Balance => 0,
                    ResourceValueField::Reserved => 1,
                    ResourceValueField::Available => 2,
                },
            );
        }
        ValueRef::Time(time) => {
            put_u8(output, 3);
            put_u8(
                output,
                match time {
                    TimeValue::BattleElapsed => 0,
                    TimeValue::RemainingTime => 1,
                },
            );
        }
        ValueRef::Result { selector, field } => {
            put_u8(output, 4);
            match selector {
                ResultSelector::EarlierEffect(effect) => {
                    put_u8(output, 0);
                    put_str(output, effect);
                }
                ResultSelector::CurrentActionPrefix => put_u8(output, 1),
            }
            match field {
                ResultField::DamageHpLost => put_u8(output, 0),
                ResultField::HealingDelta => put_u8(output, 1),
                ResultField::ResourceChangeDelta(resource) => {
                    put_u8(output, 2);
                    put_str(output, resource.as_str());
                }
                ResultField::DamageStrikes => put_u8(output, 3),
                ResultField::DamagingStrikes => put_u8(output, 4),
                ResultField::Count => put_u8(output, 5),
            }
        }
        ValueRef::AggregateMember { unit } => {
            put_u8(output, 5);
            put_unit(output, unit);
        }
    }
}

fn put_subject(output: &mut Sha256, subject: ValueSubject) {
    put_u8(
        output,
        match subject {
            ValueSubject::Source => 0,
            ValueSubject::Target => 1,
            ValueSubject::AggregateMember => 2,
        },
    );
}

fn put_option_tag(output: &mut Sha256, present: bool) {
    put_u8(output, u8::from(present));
}

fn put_str(output: &mut Sha256, value: &str) {
    put_u64(output, value.len() as u64);
    output.update(value.as_bytes());
}

fn put_u8(output: &mut Sha256, value: u8) {
    output.update([value]);
}

fn put_u64(output: &mut Sha256, value: u64) {
    output.update(value.to_be_bytes());
}
