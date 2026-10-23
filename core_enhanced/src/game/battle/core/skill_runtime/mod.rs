use crate::game::{
    ability::{SkillEffectDef, SkillStepDef},
    battle::damage::DamageModifiers,
};

pub mod area;
pub mod cast;
pub mod projectile;

pub(super) mod flow;

// Every damage effect uses the whole step's modifiers, folded in authored order.
pub(super) fn skill_step_damage_modifiers(step: &SkillStepDef) -> DamageModifiers {
    step.effects
        .iter()
        .filter_map(|effect| match &effect.op {
            SkillEffectDef::ModifyDamage { modifiers } => Some(*modifiers),
            _ => None,
        })
        .fold(DamageModifiers::default(), DamageModifiers::merge)
}
