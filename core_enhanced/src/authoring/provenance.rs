//! Cold source lookup checks the immutable registered producers. It does not
//! authenticate a runtime occurrence or grant execution permission.
use crate::game::{
    ability::{
        authoring::EffectSource,
        conditional_damage::{CompiledConditionalDamage, CompiledDamageCondition},
        state_program::{CompiledStateCondition, CompiledStateOp, CompiledStateProgram},
    },
    data::GameDataBase,
};

pub(super) fn registered_equipment_source(data: &GameDataBase, source: &EffectSource) -> bool {
    for state in data.equipment_state_catalog().by_equipment.values() {
        if state.source.as_ref() == source || state_program(&state.program, source) {
            return true;
        }
    }
    for support in data.equipment_support_catalog().by_equipment.values() {
        if [
            &support.reaction_source,
            &support.echo_source,
            &support.barrier_source,
        ]
        .into_iter()
        .flatten()
        .any(|s| s.as_ref() == source)
            || support
                .received_heal
                .as_ref()
                .is_some_and(|p| p.source() == source)
        {
            return true;
        }
    }
    for defense in data.equipment_defense_catalog().by_equipment.values() {
        if [
            &defense.revival_source,
            &defense.transformation_source,
            &defense.last_will_source,
            &defense.deferred_source,
            &defense.reserve_source,
            &defense.idle_guard_source,
            &defense.adaptive_source,
        ]
        .into_iter()
        .flatten()
        .any(|s| s.as_ref() == source)
            || [&defense.incoming_flat, &defense.wearer_reduction]
                .into_iter()
                .flatten()
                .any(|p| p.source() == source)
            || defense
                .conditional_reduction
                .as_ref()
                .is_some_and(|p| conditional(p, source))
        {
            return true;
        }
    }
    for attack in data.equipment_attack_catalog().by_equipment.values() {
        if [
            (attack.continuity.is_some(), &attack.continuity_source),
            (attack.release_gap.is_some(), &attack.release_gap_source),
            (attack.death_burst.is_some(), &attack.death_burst_source),
            (attack.victory_forms.is_some(), &attack.forms_source),
            (attack.cohort.is_some(), &attack.cohort_source),
            (attack.magazine.is_some(), &attack.magazine_source),
        ]
        .into_iter()
        .any(|(active, s)| active && s.as_ref() == source)
            || [
                &attack.bonus,
                &attack.basic_speed_points,
                &attack.primary_damage_percent,
                &attack.area_damage_percent,
            ]
            .into_iter()
            .flatten()
            .any(|p| p.source() == source)
            || attack
                .direct_penetration
                .iter()
                .any(|p| p.source.as_ref() == source)
            || attack
                .basic_lifesteal
                .as_ref()
                .is_some_and(|p| p.source.as_ref() == source)
            || attack
                .on_hit_status
                .as_ref()
                .is_some_and(|p| p.source.as_ref() == source)
            || attack
                .nearby_ally_damage
                .as_ref()
                .is_some_and(|p| p.percent.source() == source)
            || attack
                .conditional_damage
                .as_ref()
                .is_some_and(|p| conditional(p, source))
            || attack.weapon_stages.as_ref().is_some_and(|p| {
                p.base_source.as_ref() == source || p.upgraded_source.as_ref() == source
            })
            || attack
                .reactions
                .iter()
                .any(|p| state_program(&p.program, source))
        {
            return true;
        }
    }
    false
}

fn conditional(plan: &CompiledConditionalDamage, source: &EffectSource) -> bool {
    plan.percent.source() == source || damage_condition(&plan.condition, source)
}
fn damage_condition(condition: &CompiledDamageCondition, source: &EffectSource) -> bool {
    match condition {
        CompiledDamageCondition::MovementImpaired | CompiledDamageCondition::Distance { .. } => {
            false
        }
        CompiledDamageCondition::Compare { left, right, .. } => {
            left.source() == source || right.source() == source
        }
        CompiledDamageCondition::Logic(logic) => {
            logic.children().iter().any(|c| damage_condition(c, source))
        }
    }
}
fn state_program(program: &CompiledStateProgram, source: &EffectSource) -> bool {
    program
        .handlers
        .iter()
        .any(|handler| state_ops(&handler.body, source))
}
fn state_ops(ops: &[CompiledStateOp], source: &EffectSource) -> bool {
    ops.iter().any(|op| match op {
        CompiledStateOp::Chance {
            percent,
            then_ops,
            else_ops,
            ..
        } => {
            percent.source() == source || state_ops(then_ops, source) || state_ops(else_ops, source)
        }
        CompiledStateOp::AddCastDamage { percent } => percent.source() == source,
        CompiledStateOp::Set { value, .. } => value.source() == source,
        CompiledStateOp::If {
            condition,
            then_ops,
            else_ops,
        } => {
            let found = state_condition(condition, source);
            found || state_ops(then_ops, source) || state_ops(else_ops, source)
        }
        CompiledStateOp::HealOwner { amount } | CompiledStateOp::GrantOwnerResonance { amount } => {
            amount.source() == source
        }
        CompiledStateOp::ApplyOwnerState { duration, .. }
        | CompiledStateOp::ApplyEventTargetStatus { duration, .. } => duration.source() == source,
        CompiledStateOp::Resources {
            source: expected, ..
        } => expected.as_ref() == source,
        CompiledStateOp::SetBool { .. }
        | CompiledStateOp::End
        | CompiledStateOp::EndAndInvoke { .. } => false,
    })
}

fn state_condition(condition: &CompiledStateCondition, source: &EffectSource) -> bool {
    match condition {
        CompiledStateCondition::Compare { left, right, .. } => {
            left.source() == source || right.source() == source
        }
        CompiledStateCondition::Logic(logic) => {
            logic.children().iter().any(|c| state_condition(c, source))
        }
        CompiledStateCondition::Bool { .. }
        | CompiledStateCondition::Always(_)
        | CompiledStateCondition::EventVictimHadOwnerDot => false,
    }
}

pub(super) fn registered_skill_source(
    native: &crate::game::data::skill_data::SkillDatabase,
    expressions: &crate::game::value_expr::ValueExprCatalog,
    source: &EffectSource,
) -> bool {
    let Some(effect) = native.effect(&source.site) else {
        return false;
    };
    let Some(expected) = effect.source() else {
        return false;
    };
    if source.source_file != expected.source_file
        || source.origin_definition_id != expected.origin_definition_id
        || source.origin != expected.origin
    {
        return false;
    }
    let Some(suffix) = source.field_path.strip_prefix(&expected.field_path) else {
        return false;
    };
    if suffix.is_empty() {
        return true;
    }
    // Existence and producer identity are separate checks: a valid sibling
    // field (e.g. the effect's id) is not the source of a runtime application.
    serde_json::to_value(effect)
        .ok()
        .is_some_and(|value| native_descendant(&value, suffix).is_some())
        && expressions
            .state(&source.site)
            .is_some_and(|program| state_program(program, source))
}

/// Follow a native serde field/index address without accepting an ancestor as
/// proof that the requested descendant exists. Null/absent options are not
/// authored effect locations.
fn native_descendant<'a>(
    mut value: &'a serde_json::Value,
    mut path: &str,
) -> Option<&'a serde_json::Value> {
    while !path.is_empty() {
        if let Some(rest) = path.strip_prefix('.') {
            let end = rest.find(['.', '[']).unwrap_or(rest.len());
            if end == 0 {
                return None;
            }
            value = value.as_object()?.get(&rest[..end])?;
            path = &rest[end..];
        } else if let Some(rest) = path.strip_prefix('[') {
            let end = rest.find(']')?;
            let raw = &rest[..end];
            let index: usize = raw.parse().ok()?;
            if index.to_string() != raw {
                return None;
            }
            value = value.as_array()?.get(index)?;
            path = &rest[end + 1..];
        } else {
            return None;
        }
    }
    (!value.is_null()).then_some(value)
}

/// Check the static producer of a finite ground effect. Area/cast/copy IDs are
/// occurrence diagnostics; their runtime truth is checked by execution tests,
/// never by asking whether the wearer still has that copy after expiry.
pub(super) fn registered_area_effect_source(
    native: &crate::game::data::skill_data::SkillDatabase,
    source: &super::view::AreaEffectSource,
) -> bool {
    use crate::game::ability::{action::{ActionChange, AreaChange}, DeliveryDef};
    let Some(skill) = native.get_by_id(source.skill_id.as_str()) else { return false; };
    let Some(step) = skill.steps.iter().find(|step| step.id == source.step_id) else { return false; };
    let DeliveryDef::TileArea { area, .. } = &step.delivery else { return false; };
    let Some(status) = area.ground_status.as_ref().and_then(|p| p.statuses.get(source.status_index)) else {
        return false;
    };
    if let Some(native) = &status.vulnerability {
        return native.definition == source.definition && match (&native.modifier_source, &source.modifier_source) {
            (None, None) => true,
            (Some(a), Some(b)) => a.definition == b.definition && a.rule == b.rule && a.copy_uuid == b.copy_uuid,
            _ => false,
        };
    }
    let Some(modifier) = &source.modifier_source else { return false; };
    skill.action_policy.as_ref().is_some_and(|policy| policy.modifiers.iter().any(|rule| {
        rule.matches_contribution_rule_id(&modifier.rule)
            && rule.relic_definition == modifier.definition
            && source.definition == format!("{}:{}", rule.relic_definition, modifier.rule)
            && matches!(&rule.change, ActionChange::Area {
                step, change: AreaChange::ControlVulnerability { status, .. }
            } if step == &source.step_id && *status == source.status_index)
    }))
}

/// The observation role is independent of its diagnostic field path.
#[derive(Debug, Clone, Copy)]
pub(crate) enum EquipmentOriginRole {
    AdaptiveShield, ChannelOpening, IdleHpGuard, IncomingFlat, WearerReduction,
    AttackContinuity, ReleaseGap,
    ControlDuration(crate::game::ability::equipment_support::ControlDurationKind),
    ShieldBeneficiaryDamage, ReceivedHeal, SkillBarrier, BasicEvasion,
    ReadyWait, NearbyDefense, PrimaryDamage, AreaDamage, NearbyAllyDamage,
    ConditionalDamage,
    DirectPenetration(crate::game::battle::damage::DamageType),
    ConditionalReduction, ActualHealReaction, ResonanceGain, EquipmentSpeed,
    EquipmentSupport(crate::game::battle::core::equipment_support::SupportChannel),
    HealBarrier, OverhealReserve, SharedCohort, EmployeeTransformation, LastWill,
    WeaponStage(crate::game::ability::weapon_marks::WeaponStage),
    EquipmentDeathBurst, EquipmentForm, Bounty, Magazine, DeferredDamage, BasicHealEcho,
}

/// A static producer claim, not proof that a copy or event existed.
pub(crate) struct EquipmentOriginClaim<'a> {
    pub generation: Option<&'a crate::game::data::content_pack::ContentIdentity>,
    pub role: EquipmentOriginRole,
    pub equipment_uuid: Option<uuid::Uuid>,
    pub definition_id: Option<&'a str>,
    pub source_file: &'a str,
    pub field_path: &'a str,
}

// Static producer validation. Current equipment is not evidence of an earlier
// copy/cast occurrence: delayed effects can outlive their equipped provider.
pub(super) fn registered_equipment_origin(
    data: &GameDataBase, claim: &EquipmentOriginClaim<'_>,
) -> bool {
    use crate::game::ability::equipment_defense::EquipmentDefenseProgram as Defense;
    use EquipmentOriginRole as Role;
    let full_identity = !matches!(claim.role,
        Role::EquipmentDeathBurst | Role::EquipmentForm | Role::Bounty | Role::SharedCohort);
    if full_identity && claim.equipment_uuid.is_none() { return false; }
    let Some(definition) = claim.definition_id else { return false; };
    let source_matches = |source: &EffectSource| {
        source.site.action.definition_id.as_str() == definition
            && source.source_file == claim.source_file && source.field_path == claim.field_path
    };
    let tuple_matches = |id: &str, file: &str, path: &str| {
        id == definition && file == claim.source_file && path == claim.field_path
    };
    for (uuid, plan) in &data.equipment_defense_catalog().by_equipment {
        if claim.equipment_uuid.is_some_and(|expected| expected != *uuid) { continue; }
        let source = match claim.role {
            Role::IncomingFlat => plan.incoming_flat.as_ref().map(|p| p.source()),
            Role::WearerReduction => plan.wearer_reduction.as_ref().map(|p| p.source()),
            Role::ConditionalReduction => plan.conditional_reduction.as_ref().map(|p| p.percent.source()),
            _ => None,
        };
        if source.is_some_and(source_matches) { return true; }
        let active = match (&plan.program, claim.role) {
            (Defense::AdaptiveBasicShield { .. }, Role::AdaptiveShield)
            | (Defense::ChannelOpeningGuard { .. }, Role::ChannelOpening)
            | (Defense::IdleHpGuard { .. }, Role::IdleHpGuard)
            | (Defense::RechargingSkillBarrier { .. }, Role::SkillBarrier)
            | (Defense::BasicAttackEvasion { .. }, Role::BasicEvasion)
            | (Defense::ReadyWaitDamage { .. }, Role::ReadyWait)
            | (Defense::NearbyHostileDefense { .. }, Role::NearbyDefense)
            | (Defense::OverhealReserve { .. }, Role::OverhealReserve)
            | (Defense::LastWill { .. }, Role::LastWill)
            | (Defense::DeferredDamage { .. }, Role::DeferredDamage) => true,
            (Defense::EmployeeRevival { transform_into: Some(_), .. }, Role::EmployeeTransformation) => true,
            _ => false,
        };
        if active && tuple_matches(&plan.definition_id, &plan.source_file, &plan.field_path) { return true; }
    }
    for (uuid, plan) in &data.equipment_support_catalog().by_equipment {
        if claim.equipment_uuid.is_some_and(|expected| expected != *uuid) { continue; }
        let suffix = match claim.role {
            Role::ControlDuration(kind) if plan.program.control_duration.as_ref()
                .is_some_and(|p| p.kinds.contains(&kind)) => Some("control_duration"),
            Role::ShieldBeneficiaryDamage if plan.program.shield_beneficiary_damage_percent
                .is_some_and(|p| p > 0) => Some("shield_beneficiary_damage_percent"),
            Role::ReceivedHeal if plan.received_heal.is_some() => Some("received_heal_percent"),
            Role::ActualHealReaction if plan.reaction_source.is_some() => Some("actual_heal_reaction"),
            Role::ResonanceGain if plan.program.resonance_gain.is_some_and(|p| p.percent > 0) => Some("resonance_gain"),
            Role::EquipmentSupport(crate::game::battle::core::equipment_support::SupportChannel::Heal)
                if plan.program.heal_percent > 0 => Some("heal_percent"),
            Role::EquipmentSupport(crate::game::battle::core::equipment_support::SupportChannel::Shield)
                if plan.program.shield_percent > 0 => Some("shield_percent"),
            Role::HealBarrier if plan.barrier_source.is_some() => Some("direct_heal_barrier"),
            Role::BasicHealEcho if plan.echo_source.is_some() => Some("basic_heal_echo"),
            _ => None,
        };
        if let Some(suffix) = suffix {
            if tuple_matches(&plan.definition_id, &plan.source_file, &format!("{}.{suffix}", plan.field_path)) { return true; }
        }
    }
    for (uuid, plan) in &data.equipment_attack_catalog().by_equipment {
        if claim.equipment_uuid.is_some_and(|expected| expected != *uuid) { continue; }
        let source = match claim.role {
            Role::AttackContinuity if plan.continuity.is_some() => Some(plan.continuity_source.as_ref()),
            Role::ReleaseGap if plan.release_gap.is_some() => Some(plan.release_gap_source.as_ref()),
            Role::PrimaryDamage => plan.primary_damage_percent.as_ref().map(|p| p.source()),
            Role::AreaDamage => plan.area_damage_percent.as_ref().map(|p| p.source()),
            Role::NearbyAllyDamage => plan.nearby_ally_damage.as_ref().map(|p| p.percent.source()),
            Role::ConditionalDamage => plan.conditional_damage.as_ref().map(|p| p.percent.source()),
            Role::EquipmentSpeed => plan.basic_speed_points.as_ref().map(|p| p.source()),
            Role::WeaponStage(stage) => plan.weapon_stages.as_ref().map(|p| p.source(stage).as_ref()),
            Role::EquipmentDeathBurst if plan.death_burst.is_some() => Some(plan.death_burst_source.as_ref()),
            Role::EquipmentForm if plan.victory_forms.is_some() => Some(plan.forms_source.as_ref()),
            Role::Bounty if plan.cohort.as_ref().is_some_and(|p| p.bounty.is_some()) => Some(plan.cohort_source.as_ref()),
            Role::Magazine if plan.magazine.is_some() => Some(plan.magazine_source.as_ref()),
            _ => None,
        };
        if source.is_some_and(source_matches) { return true; }
        if let Role::DirectPenetration(kind) = claim.role {
            if plan.direct_penetration.iter().any(|p| p.damage_type == kind && source_matches(&p.source)) { return true; }
        }
        if matches!(claim.role, Role::SharedCohort) && plan.cohort.as_ref().is_some_and(|p|
            p.spawn.definition == definition && plan.cohort_source.source_file == claim.source_file
                && plan.cohort_source.field_path == claim.field_path) { return true; }
    }
    false
}
