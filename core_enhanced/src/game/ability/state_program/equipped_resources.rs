//! Equipped callbacks prepare fields and effects before publication. Resource
//! reads must precede writes, and writes precede effects that can re-enter the
//! battle. Reject ambiguous authored ordering instead of reading stale data.
use super::*;
use crate::game::resource::{ResourceCatalog, ResourceId, ResourceLifetime, ResourceOwnerSelector};

#[derive(Clone, Copy, Default)]
struct Phase {
    written: bool,
    emitted: bool,
}

pub(super) fn validate_equipped_resources(
    program: &CompiledStateProgram,
    catalog: &ResourceCatalog,
) -> Result<(), String> {
    fn resource(
        catalog: &ResourceCatalog,
        id: &ResourceId,
        owner: ResourceOwnerSelector,
        path: &str,
    ) -> Result<(), String> {
        crate::authoring::control::charge(
            catalog.resources.len() as u64 + 1,
            "equipment_state_resource_catalog",
        )
        .map_err(|e| format!("{path}: {e:?}"))?;
        let definition = catalog
            .resources
            .iter()
            .find(|d| d.id == *id)
            .ok_or_else(|| format!("{path}: unknown resource '{}'", id.as_str()))?;
        definition
            .validate_selector(owner)
            .map_err(|e| format!("{path}: {e}"))?;
        if owner != ResourceOwnerSelector::SourceEquipment
            || definition.lifetime != ResourceLifetime::Attempt
        {
            return Err(format!(
                "{path}: equipped state resources require SourceEquipment and Attempt lifetime"
            ));
        }
        Ok(())
    }
    fn amount(
        catalog: &ResourceCatalog,
        plan: &CompiledAmountPlan,
        phase: Phase,
    ) -> Result<(), String> {
        for reference in &plan.requirements().references {
            if let ValueRef::Resource {
                resource: id,
                owner,
                ..
            } = reference
            {
                let path = &plan.source().field_path;
                resource(catalog, id, *owner, path)?;
                if phase.written || phase.emitted {
                    return Err(format!("{path}: resource reads must precede resource writes and externally applied effects within an equipped handler"));
                }
            }
        }
        Ok(())
    }
    fn condition(
        catalog: &ResourceCatalog,
        c: &CompiledStateCondition,
        phase: Phase,
    ) -> Result<(), String> {
        match c {
            CompiledStateCondition::Compare { left, right, .. } => {
                amount(catalog, left, phase)?;
                amount(catalog, right, phase)
            }
            CompiledStateCondition::Logic(logic) => {
                for child in logic.children() {
                    condition(catalog, child, phase)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    fn branches(
        catalog: &ResourceCatalog,
        a: &[CompiledStateOp],
        b: &[CompiledStateOp],
        phase: Phase,
    ) -> Result<Phase, String> {
        let a = block(catalog, a, phase)?;
        let b = block(catalog, b, phase)?;
        Ok(Phase {
            written: a.written || b.written,
            emitted: a.emitted || b.emitted,
        })
    }
    fn block(
        catalog: &ResourceCatalog,
        ops: &[CompiledStateOp],
        mut phase: Phase,
    ) -> Result<Phase, String> {
        for op in ops {
            crate::authoring::control::charge(1, "equipment_state_resource_order")
                .map_err(|e| format!("equipment state resource order: {e:?}"))?;
            match op {
                CompiledStateOp::Resources { operations, source } => {
                    if phase.emitted {
                        return Err(format!("{}: resource writes must precede externally applied effects within an equipped handler", source.field_path));
                    }
                    for op in operations.iter() {
                        resource(catalog, &op.resource, op.owner, &source.field_path)?;
                    }
                    phase.written = true;
                }
                CompiledStateOp::Set { value, .. }
                | CompiledStateOp::AddCastDamage { percent: value } => {
                    amount(catalog, value, phase)?
                }
                CompiledStateOp::HealOwner { amount: value }
                | CompiledStateOp::GrantOwnerResonance { amount: value } => {
                    amount(catalog, value, phase)?;
                    phase.emitted = true;
                }
                CompiledStateOp::ApplyOwnerState { duration, .. }
                | CompiledStateOp::ApplyEventTargetStatus { duration, .. } => {
                    amount(catalog, duration, phase)?;
                    phase.emitted = true;
                }
                CompiledStateOp::If {
                    condition: c,
                    then_ops,
                    else_ops,
                } => {
                    condition(catalog, c, phase)?;
                    phase = branches(catalog, then_ops, else_ops, phase)?;
                }
                CompiledStateOp::Chance {
                    percent,
                    then_ops,
                    else_ops,
                    ..
                } => {
                    amount(catalog, percent, phase)?;
                    phase = branches(catalog, then_ops, else_ops, phase)?;
                }
                CompiledStateOp::SetBool { .. } => (),
                CompiledStateOp::End | CompiledStateOp::EndAndInvoke { .. } => (),
            }
        }
        Ok(phase)
    }
    for handler in &program.handlers {
        block(catalog, &handler.body, Phase::default())?;
    }
    Ok(())
}
