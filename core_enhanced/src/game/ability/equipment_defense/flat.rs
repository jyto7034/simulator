//! Compile wearer-only amounts through the shared expression engine.
use super::*;
use crate::game::{
    ability::authoring::{ActionSite, EffectSite, EffectSource},
    value_expr::{
        self as expr, CompiledAmountPlan, DeliveryCapabilities, EvalMoment, ExprLimits,
        StatSelector, ValueRef, ValueSubject, ValueUnit,
    },
};

impl EquipmentDefenseBinding {
    pub(crate) fn compile_wearer_amounts(
        &self,
        limits: &ExprLimits,
    ) -> Result<
        (
            Option<Arc<CompiledAmountPlan>>,
            Option<Arc<CompiledAmountPlan>>,
        ),
        (String, String),
    > {
        let (amount, flat, effect, unit) = match &self.program {
            EquipmentDefenseProgram::PhysicalBasicFlatReduction { amount } => (
                amount,
                true,
                "PhysicalBasicFlatReduction.amount",
                ValueUnit::Magnitude,
            ),
            EquipmentDefenseProgram::WearerReduction { percent } => {
                (percent, false, "WearerReduction.percent", ValueUnit::Scalar)
            }
            _ => return Ok((None, None)),
        };
        let path = format!("program.{effect}");
        let fail = |reason: String| (path.to_owned(), reason);
        if amount.moment != EvalMoment::Live {
            return Err(fail(
                "wearer amount requires Live at the first eligible target strike segment".into(),
            ));
        }
        let (file, base) = self.source.clone().unwrap_or_else(|| {
            (
                "typed:equipment-defenses".into(),
                "equipment_defenses[0]".into(),
            )
        });
        let source = EffectSource {
            source_file: file,
            field_path: format!("{base}.{path}"),
            site: EffectSite {
                action: ActionSite {
                    definition_id: self.equipment.clone().into(),
                    action_id: if flat {
                        "incoming_basic"
                    } else {
                        "incoming_reduction"
                    }
                    .into(),
                },
                effect_id: effect.into(),
            },
            origin_definition_id: self.equipment.clone().into(),
            origin: Default::default(),
        };
        let plan = expr::compile(
            amount,
            &source,
            DeliveryCapabilities::constant_only(),
            limits,
        )
        .map_err(|e| fail(e.to_string()))?;
        if plan.result_unit() != &unit
            || plan.requirements().resolved_targets
            || plan.requirements().references.iter().any(|reference| {
                if flat {
                    !matches!(
                        reference,
                        ValueRef::Stat {
                            subject: ValueSubject::Source,
                            stat: StatSelector::Defense
                        }
                    )
                } else {
                    !matches!(
                        reference,
                        ValueRef::HitPoints {
                            subject: ValueSubject::Source,
                            ..
                        }
                    )
                }
            })
        {
            return Err(fail(
                if flat {
                    "fixed reduction requires Magnitude and only wearer Source Defense references"
                } else {
                    "wearer reduction requires Scalar and only wearer Source HitPoints references"
                }
                .into(),
            ));
        }
        let plan = Some(Arc::new(plan));
        Ok(if flat { (plan, None) } else { (None, plan) })
    }
}
