//! Shared pure conditions and percentage plans, evaluated at target-strike admission.
use super::{authoring::EffectSource, condition::ConditionLogic, flow::FlowComparison};
use crate::game::battle::damage::DamageType;
use crate::game::value_expr::{
    self as expr, AmountExpr, CompiledAmountPlan, DeliveryCapabilities, EvalMoment, ExprLimits,
    ValueRef, ValueSubject, ValueUnit,
};
use serde::{Deserialize, Serialize};
use std::{mem::size_of, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentConditionalDamage {
    /// Outgoing damage only. Omission preserves the existing owned-source scope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_filter: Option<ConditionalDamageFilter>,
    pub condition: DamageCondition,
    pub percent: AmountExpr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionalDamageFilter {
    pub damage_types: Vec<DamageType>,
    #[serde(default)]
    pub scope: ConditionalDamageScope,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionalDamageScope {
    #[default]
    Owned,
    /// Normal damage by the wearer, including its own periodic skill effects.
    /// Excludes reaction/proc, guard release and independently acting objects.
    Wearer,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CompiledDamageFilter {
    pub types: [bool; 3],
    pub scope: ConditionalDamageScope,
}
impl Default for CompiledDamageFilter {
    fn default() -> Self {
        Self {
            types: [true; 3],
            scope: ConditionalDamageScope::Owned,
        }
    }
}
pub(crate) fn damage_type_index(kind: DamageType) -> usize {
    match kind {
        DamageType::Physical => 0,
        DamageType::Magic => 1,
        DamageType::True => 2,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DamageCondition {
    MovementImpaired,
    Distance {
        comparison: FlowComparison,
        units: u32,
    },
    Compare {
        left: AmountExpr,
        comparison: FlowComparison,
        right: AmountExpr,
    },
    Logic(ConditionLogic<DamageCondition>),
}

#[derive(Debug)]
pub(crate) enum CompiledDamageCondition {
    MovementImpaired,
    Distance {
        comparison: FlowComparison,
        units: u32,
    },
    Compare {
        left: Arc<CompiledAmountPlan>,
        comparison: FlowComparison,
        right: Arc<CompiledAmountPlan>,
    },
    Logic(ConditionLogic<CompiledDamageCondition>),
}
#[derive(Debug)]
pub(crate) struct CompiledConditionalDamage {
    pub filter: CompiledDamageFilter,
    pub condition: CompiledDamageCondition,
    pub percent: Arc<CompiledAmountPlan>,
}

type Error = (String, String);
struct Compiler<'a> {
    source: &'a EffectSource,
    field_root: &'a str,
    limits: &'a ExprLimits,
    nodes: usize,
    bytes: usize,
    work: u64,
}
impl Compiler<'_> {
    fn fail(&self, path: &str, cause: &str) -> Error {
        (format!("{}{path}", self.field_root), cause.into())
    }
    fn account(&mut self, bytes: usize, work: u64, path: &str) -> Result<(), Error> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or_else(|| self.fail(path, "conditional storage overflow"))?;
        self.work = self
            .work
            .checked_add(work)
            .ok_or_else(|| self.fail(path, "conditional work overflow"))?;
        if self.bytes > self.limits.max_retained_bytes || self.work > self.limits.max_compile_work {
            return Err(self.fail(path, "combined conditional damage budget exceeded"));
        }
        Ok(())
    }
    fn amount(
        &mut self,
        amount: &AmountExpr,
        path: &str,
    ) -> Result<Arc<CompiledAmountPlan>, Error> {
        if amount.moment != EvalMoment::Live {
            return Err(self.fail(
                path,
                "conditional damage requires Live at target-strike admission",
            ));
        }
        let mut source = self.source.clone();
        source.field_path.push_str(path);
        source.site.effect_id = format!("{}{path}", self.source.site.effect_id).into();
        let plan = expr::compile(
            amount,
            &source,
            DeliveryCapabilities {
                target_at_consume: true,
                ..DeliveryCapabilities::constant_only()
            },
            self.limits,
        )
        .map_err(|e| self.fail(path, &e.to_string()))?;
        if plan.requirements().resolved_targets
            || plan.requirements().state_owner_dot_targets
            || plan.requirements().references.iter().any(|r| {
                !matches!(
                    r,
                    ValueRef::HitPoints {
                        subject: ValueSubject::Source | ValueSubject::Target,
                        ..
                    }
                )
            })
        {
            return Err(self.fail(
                path,
                "conditional damage permits only source and target HP references",
            ));
        }
        self.account(
            plan.cost().plan_retained_bytes
                + size_of::<CompiledAmountPlan>()
                + 2 * size_of::<usize>(),
            plan.cost().compile_work,
            path,
        )?;
        Ok(Arc::new(plan))
    }
    fn condition(
        &mut self,
        condition: &DamageCondition,
        path: &str,
        depth: usize,
    ) -> Result<CompiledDamageCondition, Error> {
        self.nodes += 1;
        if depth > 8 || self.nodes > 32 {
            return Err(self.fail(
                path,
                "conditional damage requires at most 32 condition nodes and depth 8",
            ));
        }
        self.account(size_of::<CompiledDamageCondition>(), 1, path)?;
        Ok(match condition {
            DamageCondition::MovementImpaired => CompiledDamageCondition::MovementImpaired,
            DamageCondition::Distance { comparison, units } => CompiledDamageCondition::Distance {
                comparison: *comparison,
                units: *units,
            },
            DamageCondition::Compare {
                left,
                comparison,
                right,
            } => {
                let left = self.amount(left, &format!("{path}.Compare.left"))?;
                let right = self.amount(right, &format!("{path}.Compare.right"))?;
                if left.result_unit() != right.result_unit() {
                    return Err(self.fail(
                        path,
                        "conditional comparison operands require the same unit",
                    ));
                }
                CompiledDamageCondition::Compare {
                    left,
                    comparison: *comparison,
                    right,
                }
            }
            DamageCondition::Logic(logic) => {
                if logic.children().is_empty() {
                    return Err(self.fail(path, "conditional logic groups must not be empty"));
                }
                let logic = logic.try_map(|i, child| {
                    self.condition(
                        child,
                        &format!("{path}.Logic{}", logic.child_suffix(i)),
                        depth + 1,
                    )
                })?;
                CompiledDamageCondition::Logic(logic)
            }
        })
    }
}
impl EquipmentConditionalDamage {
    pub(crate) fn compile(
        &self,
        source: &EffectSource,
        limits: &ExprLimits,
    ) -> Result<(CompiledConditionalDamage, usize, u64), Error> {
        self.compile_at(source, limits, "program.conditional_damage")
    }
    pub(crate) fn compile_at(
        &self,
        source: &EffectSource,
        limits: &ExprLimits,
        field_root: &str,
    ) -> Result<(CompiledConditionalDamage, usize, u64), Error> {
        let mut compiler = Compiler {
            source,
            field_root,
            limits,
            nodes: 0,
            bytes: size_of::<CompiledConditionalDamage>(),
            work: 0,
        };
        let mut filter = CompiledDamageFilter::default();
        if let Some(authored) = &self.damage_filter {
            if authored.damage_types.is_empty() || authored.damage_types.len() > 3 {
                return Err(compiler.fail(
                    ".damage_filter.damage_types",
                    "requires 1..3 distinct damage types",
                ));
            }
            filter.types = [false; 3];
            filter.scope = authored.scope;
            for kind in &authored.damage_types {
                let index = damage_type_index(*kind);
                if std::mem::replace(&mut filter.types[index], true) {
                    return Err(
                        compiler.fail(".damage_filter.damage_types", "duplicate damage type")
                    );
                }
            }
            compiler.account(
                authored.damage_types.capacity() * size_of::<DamageType>(),
                authored.damage_types.len() as u64,
                ".damage_filter",
            )?;
        }
        let condition = compiler.condition(&self.condition, ".condition", 1)?;
        let percent = compiler.amount(&self.percent, ".percent")?;
        if percent.result_unit() != &ValueUnit::Scalar {
            return Err(compiler.fail(".percent", "conditional damage percentage requires Scalar"));
        }
        Ok((
            CompiledConditionalDamage {
                filter,
                condition,
                percent,
            },
            compiler.bytes,
            compiler.work,
        ))
    }
}
