//! Recipient rules share the ordinary typed expression compiler and current equipment ownership.
use super::*;
use crate::game::{
    ability::authoring::{ActionSite, EffectSite, EffectSource},
    value_expr::{
        self as expr, CompiledAmountPlan, DeliveryCapabilities, EvalMoment, ExprLimits, ValueRef,
        ValueSubject, ValueUnit,
    },
};
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ControlDurationKind {
    Stun,
    Freeze,
    Root,
    Slow,
    Silence,
    Taunt,
    BasicLock,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlDurationProgram {
    pub reduction_percent: u8,
    pub kinds: Vec<ControlDurationKind>,
}
impl ControlDurationProgram {
    pub(super) fn validate(&self) -> Result<(), (&'static str, &'static str)> {
        if !(1..=99).contains(&self.reduction_percent) {
            return Err((
                "control_duration.reduction_percent",
                "duration reduction requires 1..99 percent",
            ));
        }
        if self.kinds.is_empty()
            || self.kinds.len() > 7
            || self
                .kinds
                .iter()
                .enumerate()
                .any(|(i, k)| self.kinds[..i].contains(k))
        {
            return Err((
                "control_duration.kinds",
                "duration kinds must be nonempty, unique, and at most seven",
            ));
        }
        Ok(())
    }
}
crate::authoring::clone_shape::copy_shape!(ControlDurationKind);
impl EquipmentSupportBinding {
    pub(crate) fn compile_received_heal(
        &self,
        limits: &ExprLimits,
    ) -> Result<Option<Arc<CompiledAmountPlan>>, (String, String)> {
        let Some(amount) = &self.program.received_heal_percent else {
            return Ok(None);
        };
        let path = "program.received_heal_percent";
        let fail = |message: String| (path.to_owned(), message);
        if amount.moment != EvalMoment::Live {
            return Err(fail(
                "received healing requires Live at each incoming heal".into(),
            ));
        }
        let (file, base) = self.source.clone().unwrap_or_else(|| {
            (
                "typed:equipment-supports".into(),
                "equipment_supports[0]".into(),
            )
        });
        let source = EffectSource {
            source_file: file,
            field_path: format!("{base}.{path}"),
            site: EffectSite {
                action: ActionSite {
                    definition_id: self.equipment.clone().into(),
                    action_id: "received_heal".into(),
                },
                effect_id: "received_heal_percent".into(),
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
        if plan.result_unit() != &ValueUnit::Scalar
            || plan.requirements().resolved_targets
            || plan.requirements().state_owner_dot_targets
            || plan.requirements().references.iter().any(|r| {
                !matches!(
                    r,
                    ValueRef::Stat {
                        subject: ValueSubject::Source,
                        ..
                    } | ValueRef::HitPoints {
                        subject: ValueSubject::Source,
                        ..
                    } | ValueRef::RelationCount {
                        subject: ValueSubject::Source,
                        ..
                    }
                )
            })
        {
            return Err(fail("received healing requires Scalar using only recipient Source stats, HP, or relation counts".into()));
        }
        Ok(Some(Arc::new(plan)))
    }
}
pub(super) fn compiled_storage_bound(
    program: &EquipmentSupportProgram,
    plan: Option<&CompiledAmountPlan>,
) -> Result<usize, String> {
    let mut bytes = program.control_duration.as_ref().map_or(0, |p| {
        p.kinds.len() * std::mem::size_of::<ControlDurationKind>()
    });
    if let Some(plan) = plan {
        crate::authoring::control::charge_serialized(program, "recipient_support_authored_copy")
            .map_err(|e| format!("{e:?}"))?;
        let encoded = serde_json::to_vec(program)
            .map_err(|e| e.to_string())?
            .len();
        bytes = bytes
            .checked_add(plan.cost().plan_retained_bytes)
            .and_then(|n| n.checked_add(std::mem::size_of::<CompiledAmountPlan>() + 64))
            .and_then(|n| {
                n.checked_add(
                    plan.cost()
                        .nodes
                        .checked_mul(std::mem::size_of::<expr::ValueExpr>() + 64)?,
                )
            })
            .and_then(|n| {
                n.checked_add(
                    plan.cost()
                        .authored_members
                        .checked_mul(std::mem::size_of::<expr::AuthoredMember>() + 128)?,
                )
            })
            .and_then(|n| n.checked_add(encoded.checked_mul(2)?))
            .ok_or("recipient support storage overflow")?;
    }
    Ok(bytes)
}
