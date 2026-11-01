//! Bind the shared pure condition compiler to incoming damage.
use super::*;
use crate::game::{
    ability::{
        authoring::{ActionSite, EffectSite, EffectSource},
        conditional_damage::{CompiledConditionalDamage, CompiledDamageCondition, DamageCondition},
    },
    value_expr::ExprLimits,
};
impl EquipmentDefenseBinding {
    pub(crate) fn compile_conditional_reduction(
        &self,
        limits: &ExprLimits,
    ) -> Result<Option<(CompiledConditionalDamage, usize, u64)>, (String, String)> {
        let EquipmentDefenseProgram::ConditionalReduction(policy) = &self.program else {
            return Ok(None);
        };
        let (file, base) = self.source.clone().unwrap_or_else(|| {
            (
                "typed:equipment-defenses".into(),
                "equipment_defenses[0]".into(),
            )
        });
        let field_root = "program.ConditionalReduction";
        if policy.damage_filter.is_some() {
            return Err((format!("{field_root}.damage_filter"), "damage_filter is an outgoing damage contract; incoming conditional reduction does not support it".into()));
        }

        let source = EffectSource {
            source_file: file,
            field_path: format!("{base}.{field_root}"),
            site: EffectSite {
                action: ActionSite {
                    definition_id: self.equipment.clone().into(),
                    action_id: "incoming_conditional".into(),
                },
                effect_id: "ConditionalReduction".into(),
            },
            origin_definition_id: self.equipment.clone().into(),
            origin: Default::default(),
        };
        let (compiled, compiled_bytes, work) = policy.compile_at(&source, limits, field_root)?;
        // The catalog also retains the authored program. Account container/node
        // storage explicitly; serialized length bounds owned text, not nodes.
        fn plan_bytes(plan: &crate::game::value_expr::CompiledAmountPlan) -> Option<usize> {
            plan.cost()
                .nodes
                .checked_mul(std::mem::size_of::<crate::game::value_expr::ValueExpr>() + 64)?
                .checked_add(plan.cost().authored_members.checked_mul(
                    std::mem::size_of::<crate::game::value_expr::AuthoredMember>() + 128,
                )?)
        }
        fn condition_bytes(condition: &CompiledDamageCondition) -> Option<usize> {
            let extra = match condition {
                CompiledDamageCondition::Compare { left, right, .. } => {
                    plan_bytes(left)?.checked_add(plan_bytes(right)?)?
                }
                CompiledDamageCondition::Logic(logic) => logic
                    .children()
                    .iter()
                    .try_fold(0usize, |n, child| n.checked_add(condition_bytes(child)?))?,
                _ => 0,
            };
            (std::mem::size_of::<DamageCondition>() + 64).checked_add(extra)
        }
        let bytes = compiled_bytes
            .checked_add(condition_bytes(&compiled.condition).ok_or_else(|| {
                (
                    field_root.into(),
                    "conditional authored storage overflow".into(),
                )
            })?)
            .and_then(|n| n.checked_add(plan_bytes(&compiled.percent)?))
            .ok_or_else(|| {
                (
                    field_root.into(),
                    "conditional authored storage overflow".into(),
                )
            })?;
        Ok(Some((compiled, bytes, work)))
    }
}
