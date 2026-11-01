//! Completed-plan ownership, not compiler allocation history. All visits are
//! borrowed and charged before inspecting a container/leaf. Shared pointees
//! remain conservatively priced per owning edge; generation/base catalogs use
//! the existing immutable-generation convention, not repeated catalog copies.
use super::{
    authoring::*, dto::*, execution::EffectiveProviderPlan, EffectBody, EffectiveEffect,
    EffectiveEffectKey, EffectiveGrant, EffectivePlanComponents, EffectiveProperty,
    EmployeeEffectivePlan, FaultCode, PlanContribution, ProspectiveProvider, RelicPlanError, Work,
};
use crate::game::{
    ability::{
        authoring::{ActionSite, AuthoredSkillEffect, EffectOrigin, EffectSite, EffectSource},
        flow::{
            CompiledFlowCondition, CompiledFlowNode, CompiledFlowNodeKind, CompiledSideBranch,
            CompiledSkillFlow, FlowCondition, SkillFlowDef,
        },
        objects::{CreateObjectDef, ObjectWallCostDef},
        sustained::{
            BasicAttackOverlayProfile, ChannelAuraDef, SegmentedShieldDef, SustainedEffectDef,
        },
        DeliveryDef, SkillCastTargetingDef, SkillDef, SkillEffectDef, SkillId,
        SkillPresentationDef, SkillStepCondition, SkillStepDef, SkillStepRepeat, SkillUseEffectRef,
        SkillUseLifetime,
    },
    battle::tile_range::TileRangePattern,
    data::{content_pack::ContentIdentity, skill_fragment_data::SkillFragmentId},
    relic::OwnedRelic,
    resource::{ResourceId, ResourceOperation},
    resources::Position,
    skill_fragment::FragmentSourceSnapshot,
    stats::StatModifier,
    value_expr::{
        AmountExpr, CompiledAmountEntry, CompiledAmountPlan, FiniteSource, GroupOwner, ResultField,
        ResultSelector, TypedInteger, ValueExpr, ValueExprCatalog, ValueRef, ValueUnit,
    },
};
use std::{mem::size_of, sync::Arc};

fn overflow() -> RelicPlanError {
    RelicPlanError::new(
        FaultCode::ArithmeticOverflow,
        "effective retained storage overflow",
    )
}
pub(super) struct Extent<'a> {
    bytes: usize,
    work: &'a mut Work,
    shared_pointees: bool,
    comparison: bool,
}
impl Extent<'_> {
    fn scan(&mut self) -> Result<(), RelicPlanError> {
        self.work.charge(1, "effective_retained_scan")
    }
    fn add(&mut self, bytes: Option<usize>) -> Result<(), RelicPlanError> {
        self.bytes = self
            .bytes
            .checked_add(bytes.ok_or_else(overflow)?)
            .ok_or_else(overflow)?;
        Ok(())
    }
    fn allocation(&mut self, bytes: Option<usize>) -> Result<(), RelicPlanError> {
        let bytes = bytes.ok_or_else(overflow)?;
        if bytes != 0 {
            self.add(bytes.checked_add(64))?;
        }
        Ok(())
    }
}
// Inline values are already priced by their enclosing struct/enum/container.
// These visitors add only dynamic allocations reachable from those values.
pub(super) trait Heap {
    fn heap(&self, meter: &mut Extent<'_>) -> Result<(), RelicPlanError>;
}
impl<T: Heap> Heap for Vec<T> {
    fn heap(&self, meter: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        meter.scan()?;
        meter.allocation(self.capacity().checked_mul(size_of::<T>()))?;
        meter
            .work
            .charge(self.len(), "effective_retained_members")?;
        for value in self {
            value.heap(meter)?;
        }
        Ok(())
    }
}
// Borrowed values at a copy boundary are copied into owned containers/DTOs.
impl<T: Heap + ?Sized> Heap for &T {
    fn heap(&self, meter: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        meter.scan()?;
        meter.allocation(Some(std::mem::size_of_val(*self)))?;
        (**self).heap(meter)
    }
}
impl<T: Heap> Heap for [T] {
    fn heap(&self, meter: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        meter
            .work
            .charge(self.len(), "effective_retained_members")?;
        for value in self {
            value.heap(meter)?;
        }
        Ok(())
    }
}
impl Heap for String {
    fn heap(&self, meter: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        meter.scan()?;
        if meter.comparison {
            meter.work.charge(self.len(), "effective_comparison_text")?;
        }
        meter.allocation(Some(self.capacity()))
    }
}
impl<T: Heap> Heap for Arc<T> {
    fn heap(&self, meter: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        meter.scan()?;
        if !meter.shared_pointees {
            return Ok(());
        }
        meter.allocation(Some(size_of::<T>()))?;
        (**self).heap(meter)
    }
}
impl<T: Heap> Heap for Box<T> {
    fn heap(&self, meter: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        meter.scan()?;
        meter.allocation(Some(size_of::<T>()))?;
        (**self).heap(meter)
    }
}
impl<T: Heap> Heap for Option<T> {
    fn heap(&self, meter: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        meter.scan()?;
        if let Some(value) = self {
            value.heap(meter)?;
        }
        Ok(())
    }
}
impl<A: Heap, B: Heap> Heap for (A, B) {
    fn heap(&self, meter: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        meter.scan()?;
        self.0.heap(meter)?;
        self.1.heap(meter)
    }
}
macro_rules! fixed {
    ($($ty:ty),* $(,)?) => {$ (
        impl Heap for $ty {
            fn heap(&self, meter: &mut Extent<'_>) -> Result<(), RelicPlanError> {
                meter.scan()
            }
        }
    )*};
}
fixed!(
    usize,
    u32,
    bool,
    Position,
    StatModifier,
    CompiledSideBranch,
    NumericEditDef,
    crate::game::value_expr::ResolvedTargetPredicate
);
macro_rules! text_id {
    ($($ty:ty),* $(,)?) => {$ (
        impl Heap for $ty {
            fn heap(&self, meter: &mut Extent<'_>) -> Result<(), RelicPlanError> {
                meter.scan()?;
                if meter.comparison { meter.work.charge(self.retained_text_capacity(), "effective_comparison_text")?; }
                meter.allocation(Some(self.retained_text_capacity()))
            }
        }
    )*};
}
text_id!(SkillId, SkillFragmentId, ResourceId);
// Exhaustive destructuring makes a new retained field an explicit visitor
// maintenance requirement, including fields skipped by serde.
macro_rules! fields {
    ($ty:ty; [$($owned:ident),*]; [$($inline:ident),*]) => {
        impl Heap for $ty {
            fn heap(&self, meter: &mut Extent<'_>) -> Result<(), RelicPlanError> {
                meter.scan()?;
                let Self { $($owned,)* $($inline: _,)* } = self;
                $($owned.heap(meter)?;)*
                Ok(())
            }
        }
    };
}
fields!(EffectivePlanComponents; [providers, copies, effects, grants, execution, causal_rules, wedge_rules, action_modifiers, rules, fingerprint];
        [employee_uuid, usage, retained_bytes]);
fixed!(
    CausalEventDef,
    CausalDamagePerspective,
    CausalTargetDef,
    super::causal::CausalPayloadKind
);
fields!(StructuralSelectorDef; [provider, effect]; []);
fields!(ProviderPredicateDef; [equipment_id, fragment_id, form_id]; []);
fields!(EffectPredicateDef; [all_tags]; [op, has_property]);
fields!(crate::game::data::relic_data::RelicDamageSelector; [all_tags, equipment_id, fragment_id, form_id]; [target, damage_type]);
fields!(CausalBranchDef; [id, program]; []);
fields!(CausalProgramDef; [steps, flow]; []);
fields!(CausalStepDef; [id, delivery, effects]; [delay_ms, target]);
fields!(CausalEffectDef; [id, tags, op]; []);
fields!(super::causal::EffectiveCausalProgram; [id, contribution, execution, targets]; [site_kind, site_index, event, retained_bytes]);
fields!(super::causal::EffectiveCausalRule; [contribution, selector, condition, source, payload, programs]; [event, interval_ms, kind]);
fields!(super::causal::EffectiveWedgeRule; [contribution, selector, condition, effect_sites]; [increment_percent, cap_percent]);
fields!(super::action::EffectiveActionModifier; [contribution, change]; [provider]);
impl Heap for RelicConditionDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Always => Ok(()),
            Self::SameEffect(p) => p.heap(m),
            Self::Exists(s) => s.heap(m),
            Self::All(v) | Self::Any(v) => v.heap(m),
            Self::Not(c) => c.heap(m),
        }
    }
}
impl Heap for CausalPayloadDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Program(p) => p.heap(m),
            Self::Fork { branches } => branches.heap(m),
            Self::Join { branches, then } => {
                branches.heap(m)?;
                then.heap(m)
            }
        }
    }
}
impl Heap for CausalPrimitiveDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::DamageExpr { amount, .. } | Self::HealExpr { amount } => amount.heap(m),
            Self::PersonalShieldGrant { definition } => definition.heap(m),
            Self::Resources { operations } => operations.heap(m),
            Self::ApplyBuff { buff_id, .. } => buff_id.heap(m),
            _ => Ok(()),
        }
    }
}
fields!(FragmentSourceSnapshot; [definition_id, form_id, skill_id]; [copy_uuid]);
fields!(OwnedRelic; [definition_id]; [copy_uuid, variant]);
fields!(PlanContribution; [definition_id, rule_id]; [copy_uuid, variant]);
fields!(ActionSite; [definition_id, action_id]; []);
fields!(EffectSite; [action, effect_id]; []);
fields!(EffectiveProperty; [contributions]; [property, base, program, final_value]);
fields!(EffectiveEffect; [key, anchor, tags, delivery, properties, recipes, recipe_contributions, body, amount];
        [provider, operation, amount_capabilities, original, present]);
fields!(EffectiveGrant; [contribution, anchor, body]; [provider]);
fields!(RelicRuleStateDto; [contribution, targets]; [state]);
fields!(EffectiveEffectAddressDto; [effect]; [provider_index]);
fields!(EffectiveProviderPlan; [skill, flow, amounts, effects, selected_recipient_reach];
        [selected_recipient_excludes_caster]);
fields!(GrantedEffectDef; [id, tags, op]; []);
fields!(GrantedFlowStepDef; [id, delivery, effects]; [delay_ms, target]);
fields!(PersonalShieldGrantDef; [amount, source_creation, support]; [duration_ms, priority, recipients, replace_only_recipients]);
fields!(ShieldCreationModifierDef; [operations]; []);
fields!(SkillDef; [action_policy, use_lifetime, flow, id, name, cast_targeting, steps];
        [kind, focus_time_ms, focus_permissions, resonance_admission]);
fields!(SkillUseEffectRef; [step, effect]; []);
fields!(SkillStepDef; [id, when, repeat, delivery, effects, presentation];
        [delay_ms, target, targeting]);
fields!(SkillPresentationDef; [cast_state, projectile_vfx_id, impact_vfx_id, target_anchor]; []);
fields!(AuthoredSkillEffect; [id, tags, origin, attached_to, op, source]; []);
fields!(EffectSource; [source_file, site, field_path, origin_definition_id, origin]; []);
fields!(TileRangePattern; [rows]; [include_anchor_tile]);
fields!(BasicAttackOverlayProfile; [defense_tile_range, delivery, line]; [range_policy, followup_on_input]);
fields!(crate::game::ability::sustained::BasicLineProfile; [piercing]; [length_units, width_units, power, weapon_effects, piercing_relics]);
fields!(crate::game::ability::sustained::BasicPiercingRule; [relic_definition]; [increment_percent, cap_percent]);
fields!(ChannelAuraDef; [family, support]; [priority, prefer_strongest, radius_units, basic_damage_percent, basic_speed_points, basic_scope, health_upkeep]);
fields!(super::support::SupportPolicyDef; [range, conversion, conversions]; []);
fields!(super::support::SupportConversionDef; [relic_definition, selection_range, replacement]; [variant]);
impl Heap for super::support::SupportReplacementDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::PersonalShield { amount } => amount.heap(m),
            Self::RegularBasic { .. } => Ok(()),
        }
    }
}
impl Heap for super::support::SelectedSupportReach {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Pattern { range, .. } => range.heap(m),
            Self::Radius(_) => Ok(()),
        }
    }
}
fields!(SegmentedShieldDef; [pool_id]; [segment_health, max_segments, initial_health, block_per_segment, priority]);
fields!(ObjectWallCostDef; [pool_id]; [pool_source, health_cost_percent, wall_health_percent]);
fields!(ResourceOperation; [resource]; [owner, operation]);
fields!(AmountExpr; [root]; [moment, provenance]);
fields!(TypedInteger; [unit]; [value]);
fields!(crate::game::value_expr::AuthoredMember; [identity, value, owner]; [projected_tile]);
fields!(GroupOwner; [identity]; [domain]);
fields!(CompiledSkillFlow; [nodes, side_branches, settled_effect_prefix, source, step_ids];
        [root, legacy, authored_delay_bound_ms]);
fields!(CompiledFlowNode; [id, kind, result_scope];
        [result_available_on_completion, guaranteed_effect_prefix]);
impl Heap for (String, u32, bool, bool) {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        self.0.heap(m)
    }
}
impl Heap for ProspectiveProvider {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Fragment(value) => value.heap(m),
            Self::Equipment {
                definition_id,
                copy_uuid: _,
                skill_id,
            } => {
                definition_id.heap(m)?;
                skill_id.heap(m)
            }
        }
    }
}
impl Heap for EffectiveEffectKey {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Original { site } => site.heap(m),
            Self::Generated {
                relic_copy_uuid: _,
                grant_rule_id,
                anchor,
                local_action_id,
                local_effect_id,
            } => {
                grant_rule_id.heap(m)?;
                anchor.heap(m)?;
                local_action_id.heap(m)?;
                local_effect_id.heap(m)
            }
        }
    }
}
impl Heap for EffectBody {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Original {
                skill_id,
                step: _,
                effect: _,
            } => skill_id.heap(m),
            Self::Granted(value) => value.heap(m),
        }
    }
}
impl Heap for StructuralRecipeDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::SupportSelectionPattern { range, .. } => range.heap(m),
            Self::ChannelHealthUpkeep { .. } => Ok(()),
            Self::DamageFootprint { footprint } => footprint.heap(m),
            Self::PersonalShieldSingle { initial_amount } => initial_amount.heap(m),
            Self::FixedRecipientSupport
            | Self::RegularBasicSupport {
                damage_percent: _,
                speed_points: _,
            } => Ok(()),
        }
    }
}
impl Heap for GrantBodyDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::InlineEffects { action_id, effects } => {
                action_id.heap(m)?;
                effects.heap(m)
            }
            Self::FiniteFlow {
                anchor_action_id,
                steps,
                flow,
            } => {
                anchor_action_id.heap(m)?;
                steps.heap(m)?;
                flow.heap(m)
            }
        }
    }
}
impl Heap for GrantPrimitiveDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::DamageExpr {
                amount,
                damage_type: _,
            }
            | Self::HealExpr { amount } => amount.heap(m),
            Self::PersonalShieldGrant { definition } => definition.heap(m),
            Self::Damage {
                amount: _,
                damage_type: _,
            }
            | Self::Heal { amount: _ }
            | Self::HealthCost {
                amount: _,
                allow_lethal: _,
            } => Ok(()),
        }
    }
}
impl Heap for SkillUseLifetime {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Action => Ok(()),
            Self::ActionAndEffects { effects } => effects.heap(m),
        }
    }
}
impl Heap for SkillCastTargetingDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Explicit {
                selection: _,
                target: _,
                reach: _,
                target_range,
                air_capable: _,
            } => target_range.heap(m),
        }
    }
}
impl Heap for SkillStepCondition {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Always | Self::IfPreviousStepDealtDamage => Ok(()),
            Self::IfCasterHasBuff {
                buff_id,
                min_stacks: _,
            } => buff_id.heap(m),
        }
    }
}
impl Heap for SkillStepRepeat {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Once | Self::Times { count: _ } => Ok(()),
            Self::ByBuffStacks {
                unit: _,
                buff_id,
                max: _,
            } => buff_id.heap(m),
        }
    }
}
impl Heap for EffectOrigin {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Original => Ok(()),
            Self::Added {
                source_definition_id,
            } => source_definition_id.heap(m),
        }
    }
}
impl Heap for DeliveryDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Instant
            | Self::Projectile {
                speed_units_per_ms: _,
                hit_policy: _,
                allow_targetless_cast: _,
                max_range_tiles: _,
                max_lifetime_ms: _,
                max_kills: _,
                max_pierces: _,
                collision: _,
            } => Ok(()),
            Self::TileArea { footprint, area: _ } => footprint.heap(m),
        }
    }
}
impl Heap for CreateObjectDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        let Self {
            definition_id,
            anchor: _,
            offsets,
            allow_same_source_evolution: _,
            wall_cost,
        } = self;
        definition_id.0.heap(m)?;
        offsets.heap(m)?;
        wall_cost.heap(m)
    }
}
impl Heap for SustainedEffectDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::State { program, .. } | Self::PreparedState { program, .. } => program.heap(m),
            Self::PreparedBasic { profile } => {
                profile.definition.heap(m)?;
                if let Some(reward) = &profile.kill_reward {
                    reward.profile.definition.heap(m)?;
                }
                Ok(())
            }
            Self::IndependentShield { .. } => Ok(()),
            Self::HitStatus { profile, .. } => {
                profile.definition.heap(m)?;
                profile.modifier_source.heap(m)?;
                if let Some(v) = &profile.vulnerability {
                    v.definition.heap(m)?;
                    v.modifier_source.heap(m)?;
                }
                Ok(())
            }
            Self::TargetState { profile, .. } => profile.definition.heap(m),
            Self::ReactiveHeal { profile, .. } => profile.definition.heap(m),
            Self::Parry {
                duration_ms: _,
                window_ms: _,
                invulnerable_ms: _,
                miss_recovery_ms: _,
                reduction_percent: _,
                counter_skill,
                guard_charge,
                ..
            } => {
                counter_skill.heap(m)?;
                guard_charge.heap(m)
            }
            Self::SegmentedShield {
                duration_ms: _,
                pool,
            } => pool.heap(m),
            Self::BasicAttackOverlay {
                duration_ms: _,
                next_attack_only: _,
                goodbye_skill,
                profile,
            } => {
                goodbye_skill.heap(m)?;
                profile.heap(m)
            }
            Self::ChannelAura {
                duration_ms: _,
                aura,
            } => aura.heap(m),
            Self::Modifiers {
                duration_ms: _,
                modifiers,
            } => modifiers.heap(m),
            Self::Control {
                duration_ms: _,
                kind: _,
            }
            | Self::Survival {
                duration_ms: _,
                policy: _,
            }
            | Self::Shield {
                duration_ms: _,
                amount: _,
                priority: _,
                shared_radius_units: _,
            }
            | Self::CombatModifiers {
                duration_ms: _,
                modifiers: _,
            } => Ok(()),
        }
    }
}
impl Heap for crate::game::ability::state_program::StateProgram {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        self.id.heap(m)?;
        self.basic_attack.heap(m)?;
        self.fields.heap(m)?;
        self.handlers.heap(m)
    }
}
impl Heap for crate::game::ability::state_program::StateField {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        self.name.heap(m)?;
        match &self.value {
            crate::game::ability::state_program::StateFieldDef::Integer { unit, .. } => {
                unit.heap(m)
            }
            crate::game::ability::state_program::StateFieldDef::Bool(_) => Ok(()),
        }
    }
}
impl Heap for crate::game::ability::state_program::StateHandler {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        self.id.heap(m)?;
        self.body.heap(m)
    }
}
impl Heap for crate::game::ability::state_program::StateOp {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        use crate::game::ability::state_program::StateOp;
        m.scan()?;
        match self {
            StateOp::Set { field, value } => {
                field.heap(m)?;
                value.heap(m)
            }
            StateOp::SetBool { field, .. } => field.heap(m),
            StateOp::AddCastDamage { percent } => percent.heap(m),
            StateOp::Chance { percent, then_ops, else_ops } => {
                percent.heap(m)?; then_ops.heap(m)?; else_ops.heap(m)
            },
            StateOp::HealOwner { amount } | StateOp::GrantOwnerResonance { amount } => amount.heap(m),
            StateOp::ApplyOwnerState {
                duration, profile, ..
            } => {
                duration.heap(m)?;
                profile.definition.heap(m)
            }
            StateOp::ApplyEventTargetStatus { duration, profile } => {
                duration.heap(m)?;
                profile.definition.heap(m)?;
                profile.modifier_source.heap(m)?;
                profile.vulnerability.heap(m)
            }
            StateOp::Resources { operations } => operations.heap(m),
            StateOp::End => Ok(()),
            StateOp::EndAndInvoke { skill } => skill.heap(m),
            StateOp::If {
                condition,
                then_ops,
                else_ops,
            } => {
                condition.heap(m)?;
                then_ops.heap(m)?;
                else_ops.heap(m)
            }
        }
    }
}
impl Heap for SkillEffectDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::GuardCounter { program } => program.heap(m),
            Self::StatusStrike { program } => program.heap(m),
            Self::FusionPlacement { program } => program.heap(m),
            Self::PairedAreas { program } => program.heap(m),
            Self::PreparedVolley { program } => program.heap(m),
            Self::InputPlan { program } => program.heap(m),
            Self::RelationAction { program } => program.heap(m),
            Self::CounterChannel { program, root, .. } => {
                root.heap(m)?;
                program.heap(m)
            }
            Self::TileBurst { program } => {
                m.allocation(Some(std::mem::size_of_val(program.as_ref())))?;
                program.range.heap(m)?;
                if let crate::game::ability::tile_burst::TileBurstSource::WeaponStacks {
                    weapon,
                    key,
                    conversion,
                    ..
                } = &program.source
                {
                    weapon.heap(m)?;
                    key.heap(m)?;
                    if let Some(c) = conversion {
                        c.relic_definition.heap(m)?;
                        c.range.heap(m)?;
                    }
                }
                m.allocation(
                    program
                        .modifiers
                        .capacity()
                        .checked_mul(
                            std::mem::size_of::<crate::game::ability::marks::MarkModifier>(),
                        ),
                )?;
                for modifier in &program.modifiers {
                    modifier.id.heap(m)?;
                    modifier.relic_definition.heap(m)?;
                    if let crate::game::ability::marks::MarkChange::Range { pattern } =
                        &modifier.change
                    {
                        pattern.heap(m)?;
                    }
                }
                Ok(())
            }
            Self::PresenceBurst { program } => {
                m.allocation(Some(std::mem::size_of_val(program.as_ref())))?;
                program.marks.definition.heap(m)?;
                program.explosion.range.heap(m)?;
                m.allocation(
                    program
                        .modifiers
                        .capacity()
                        .checked_mul(
                            std::mem::size_of::<crate::game::ability::marks::MarkModifier>(),
                        ),
                )?;
                for modifier in &program.modifiers {
                    modifier.id.heap(m)?;
                    modifier.relic_definition.heap(m)?;
                    if let crate::game::ability::marks::MarkChange::Range { pattern } =
                        &modifier.change
                    {
                        pattern.heap(m)?;
                    }
                }
                Ok(())
            }
            Self::PersonalShieldGrant { definition } => definition.heap(m),
            Self::CreateObject { definition } => definition.heap(m),
            Self::RechargeShield {
                source_skill,
                amount: _,
            } => source_skill.heap(m),
            Self::ShieldExpr {
                amount,
                duration_ms: _,
                priority: _,
                shared_radius_units: _,
            }
            | Self::DamageExpr {
                amount,
                damage_type: _,
            }
            | Self::HealExpr { amount } => amount.heap(m),
            Self::Sustained { definition } => definition.heap(m),
            Self::Resources { operations } => operations.heap(m),
            Self::ApplyBuff {
                buff_id,
                duration_ms: _,
            } => buff_id.heap(m),
            Self::Pull {
                max_distance_tiles: _,
            }
            | Self::HealthCost {
                amount: _,
                allow_lethal: _,
            }
            | Self::Damage {
                amount: _,
                damage_type: _,
            }
            | Self::ModifyDamage { modifiers: _ }
            | Self::Heal { amount: _ }
            | Self::ModifyResonance { amount: _ }
            | Self::ModifyTacticalResource { amount: _ }
            | Self::ModifyStats { modifier: _ }
            | Self::InterruptCast
            | Self::ExtraAttack { count: _ } => Ok(()),
        }
    }
}
impl Heap for FlowCondition {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Predicate(call) => call.heap(m),
            Self::Logic(logic) => logic.heap(m),
            Self::Always(_) => Ok(()),
            Self::Compare {
                node_id,
                metric: _,
                comparison: _,
                value: _,
            } => node_id.heap(m),
            Self::EffectCompare {
                node_id,
                effect_id,
                metric: _,
                comparison: _,
                value: _,
            } => {
                node_id.heap(m)?;
                effect_id.heap(m)
            }
        }
    }
}
impl Heap for SkillFlowDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Action {
                id,
                step_id,
                wait: _,
            } => {
                id.heap(m)?;
                step_id.heap(m)
            }
            Self::Wait { id, duration_ms: _ } => id.heap(m),
            Self::Sequence { id, children } => {
                id.heap(m)?;
                children.heap(m)
            }
            Self::Parallel { id, branches } => {
                id.heap(m)?;
                branches.heap(m)
            }
            Self::Branch {
                id,
                condition,
                then_node,
                else_node,
            } => {
                id.heap(m)?;
                condition.heap(m)?;
                then_node.heap(m)?;
                else_node.heap(m)
            }
            Self::Repeat {
                id,
                count: _,
                interval_ms: _,
                body,
            } => {
                id.heap(m)?;
                body.heap(m)
            }
        }
    }
}
impl Heap for CompiledFlowCondition {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Logic(logic) => logic.heap(m),
            Self::Always(_)
            | Self::Compare {
                producer: _,
                metric: _,
                comparison: _,
                value: _,
            } => Ok(()),
            Self::EffectCompare {
                producer: _,
                effect,
                metric: _,
                comparison: _,
                value: _,
            } => effect.heap(m),
        }
    }
}
impl Heap for CompiledFlowNodeKind {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Action {
                step_index: _,
                wait: _,
            }
            | Self::Wait { duration_ms: _ }
            | Self::Repeat {
                count: _,
                interval_ms: _,
                body: _,
            } => Ok(()),
            Self::Sequence { children } => children.heap(m),
            Self::Parallel { branches } => branches.heap(m),
            Self::Branch {
                condition,
                then_node: _,
                else_node: _,
            } => condition.heap(m),
        }
    }
}
impl Heap for ValueUnit {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Resource(id) => id.heap(m),
            Self::Magnitude
            | Self::Scalar
            | Self::Count
            | Self::Milliseconds
            | Self::AttackSpeedPoints => Ok(()),
        }
    }
}
impl Heap for ResultSelector {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::EarlierEffect(id) => id.heap(m),
            Self::CurrentActionPrefix => Ok(()),
        }
    }
}
impl Heap for ResultField {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::ResourceChangeDelta(id) => id.heap(m),
            Self::DamageHpLost
            | Self::HealingDelta
            | Self::DamageStrikes
            | Self::DamagingStrikes
            | Self::Count => Ok(()),
        }
    }
}
impl Heap for ValueRef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::StateField { name, unit } => {
                name.heap(m)?;
                unit.heap(m)
            }
            Self::EventHpLost | Self::RelationCount { .. } => Ok(()),
            Self::Stat {
                subject: _,
                stat: _,
            }
            | Self::HitPoints {
                subject: _,
                field: _,
            }
            | Self::Time(_) => Ok(()),
            Self::Resource {
                resource,
                owner: _,
                field: _,
            } => resource.heap(m),
            Self::Result { selector, field } => {
                selector.heap(m)?;
                field.heap(m)
            }
            Self::AggregateMember { unit } => unit.heap(m),
        }
    }
}
impl Heap for FiniteSource {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::AuthoredMembers(values) => values.heap(m),
            Self::ResolvedTargetsWhere(predicates) => predicates.heap(m),
            Self::ResolvedTargets | Self::StateOwnerDotTargets => Ok(()),
        }
    }
}
impl Heap for ValueExpr {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Const(value) => value.heap(m),
            Self::Read(value) => value.heap(m),
            Self::Add(values) => values.heap(m),
            Self::Mul { value, scalar } => {
                value.heap(m)?;
                scalar.heap(m)
            }
            Self::Ratio {
                numerator,
                denominator,
                rounding: _,
            } => {
                numerator.heap(m)?;
                denominator.heap(m)
            }
            Self::Clamp { value, min, max } => {
                value.heap(m)?;
                min.heap(m)?;
                max.heap(m)
            }
            Self::Aggregate {
                source,
                grouping: _,
                reduction: _,
                outer_reduction: _,
                member,
            } => {
                source.heap(m)?;
                member.heap(m)
            }
        }
    }
}
impl Heap for CompiledAmountPlan {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        // Existing checked final-plan proof covers boxed segments/op payload,
        // source text and requirements. It excludes compile scratch and hash
        // scratch. The Arc visitor separately includes this fixed plan header.
        m.add(Some(self.cost().plan_retained_bytes))
    }
}
impl Heap for ValueExprCatalog {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        // Same conservative Rust B-tree node/spare-slot proof as native fact
        // tables, including one empty root. No len-only live-row estimate.
        let nodes = 1_usize.checked_add(self.len().saturating_sub(1) / 5);
        let node_bytes = size_of::<(EffectSite, CompiledAmountEntry)>()
            .checked_mul(11)
            .and_then(|n| n.checked_add(12 * size_of::<usize>() + 128));
        m.add(nodes.and_then(|count| node_bytes.and_then(|bytes| count.checked_mul(bytes))))?;
        m.work
            .charge(self.len(), "effective_retained_catalog_entries")?;
        for (site, entry) in self.entries() {
            site.heap(m)?;
            entry.plan().heap(m)?;
        }
        let states = self.states();
        if !states.is_empty() {
            let nodes = 1_usize.checked_add(states.len().saturating_sub(1) / 5);
            let bytes = size_of::<(
                EffectSite,
                Arc<crate::game::ability::state_program::CompiledStateProgram>,
            )>()
            .checked_mul(11)
            .and_then(|n| n.checked_add(12 * size_of::<usize>() + 128));
            m.add(nodes.and_then(|n| bytes.and_then(|b| n.checked_mul(b))))?;
            for (site, program) in states {
                m.scan()?;
                site.heap(m)?;
                m.allocation(program.retained_bytes.checked_add(2 * size_of::<usize>()))?;
            }
        }
        Ok(())
    }
}
pub(super) fn measure_causal_program(
    program: &super::causal::EffectiveCausalProgram,
    work: &mut Work,
) -> Result<usize, RelicPlanError> {
    let mut meter = Extent {
        bytes: 0,
        work,
        shared_pointees: true,
        comparison: false,
    };
    meter.scan()?;
    meter.allocation(Some(size_of::<super::causal::EffectiveCausalProgram>()))?;
    program.heap(&mut meter)?;
    Ok(meter.bytes)
}

pub(super) fn measure(
    plan: &EffectivePlanComponents,
    generation: &ContentIdentity,
    work: &mut Work,
) -> Result<usize, RelicPlanError> {
    let mut meter = Extent {
        bytes: 0,
        work,
        shared_pointees: true,
        comparison: false,
    };
    meter.scan()?;
    meter.allocation(Some(size_of::<EmployeeEffectivePlan>()))?;
    // Preserve the existing identity/shared-canonical-generation convention.
    // Dynamic generation-local overlays above are fully priced, not omitted.
    meter.allocation(Some(size_of::<ContentIdentity>()))?;
    generation.semantic_sha256.heap(&mut meter)?;
    plan.heap(&mut meter)?;
    Ok(meter.bytes)
}

// Extents belong to the immutable originals held by PreparedProviders. They
// price the complete pointee on EVERY owning edge; reuse avoids walking that
// same immutable owner again, not charging less retained memory for sharing.
pub(super) struct PreparedStorage {
    providers: usize,
    effects: Vec<usize>,
    execution: Vec<usize>,
}
impl PreparedStorage {
    pub(super) fn new(
        providers: &Arc<Vec<ProspectiveProvider>>,
        effects: &[super::SharedEffect],
        execution: &[EffectiveProviderPlan],
        work: &mut Work,
    ) -> Result<Self, RelicPlanError> {
        work.reserve(
            effects
                .len()
                .checked_add(execution.len())
                .and_then(|n| n.checked_mul(size_of::<usize>()))
                .and_then(|n| n.checked_add(size_of::<Self>() + 128)),
            "publication_storage_proof",
        )?;
        fn heap_bytes(value: &impl Heap, work: &mut Work) -> Result<usize, RelicPlanError> {
            let mut meter = Extent {
                bytes: 0,
                work,
                shared_pointees: true,
                comparison: false,
            };
            value.heap(&mut meter)?;
            Ok(meter.bytes)
        }
        let mut effect_extents = Vec::with_capacity(effects.len());
        for effect in effects {
            effect_extents.push(heap_bytes(effect, work)?);
        }
        let mut execution_extents = Vec::with_capacity(execution.len());
        for provider in execution {
            execution_extents.push(heap_bytes(provider, work)?);
        }
        Ok(Self {
            providers: heap_bytes(providers, work)?,
            effects: effect_extents,
            execution: execution_extents,
        })
    }

    pub(super) fn measure(
        &self,
        plan: &EffectivePlanComponents,
        generation: &ContentIdentity,
        work: &mut Work,
        original_providers: &Arc<Vec<ProspectiveProvider>>,
        original_effects: &[super::SharedEffect],
        original_execution: &[EffectiveProviderPlan],
    ) -> Result<usize, RelicPlanError> {
        let mut meter = Extent {
            bytes: 0,
            work,
            shared_pointees: true,
            comparison: false,
        };
        meter.scan()?;
        meter.allocation(Some(size_of::<EmployeeEffectivePlan>()))?;
        meter.allocation(Some(size_of::<ContentIdentity>()))?;
        generation.semantic_sha256.heap(&mut meter)?;
        // Keep this exhaustive, like Heap's declaration, including serde-skipped
        // owners. A newly retained field must be priced in both paths.
        let EffectivePlanComponents {
            employee_uuid: _,
            providers,
            copies,
            effects,
            grants,
            execution,
            causal_rules,
            wedge_rules,
            action_modifiers,
            rules,
            fingerprint,
            usage: _,
            retained_bytes: _,
        } = plan;
        meter.scan()?;
        meter.work.charge(1, "publication_storage_identity")?;
        if Arc::ptr_eq(providers, original_providers) {
            meter.add(Some(self.providers))?;
        } else {
            providers.heap(&mut meter)?;
        }
        copies.heap(&mut meter)?;
        meter.scan()?;
        meter.allocation(
            effects
                .capacity()
                .checked_mul(size_of::<super::SharedEffect>()),
        )?;
        meter
            .work
            .charge(effects.len(), "effective_retained_members")?;
        for (index, effect) in effects.iter().enumerate() {
            meter.work.charge(1, "publication_storage_identity")?;
            if original_effects
                .get(index)
                .is_some_and(|old| Arc::ptr_eq(old, effect))
            {
                meter.add(Some(*self.effects.get(index).ok_or_else(overflow)?))?;
            } else {
                effect.heap(&mut meter)?;
            }
        }
        grants.heap(&mut meter)?;
        meter.scan()?;
        meter.allocation(
            execution
                .capacity()
                .checked_mul(size_of::<EffectiveProviderPlan>()),
        )?;
        meter
            .work
            .charge(execution.len(), "effective_retained_members")?;
        for (index, provider) in execution.iter().enumerate() {
            meter.work.charge(1, "publication_storage_identity")?;
            if original_execution
                .get(index)
                .is_some_and(|old| same_provider_storage(old, provider))
            {
                meter.add(Some(*self.execution.get(index).ok_or_else(overflow)?))?;
            } else {
                provider.heap(&mut meter)?;
            }
        }
        causal_rules.heap(&mut meter)?;
        wedge_rules.heap(&mut meter)?;
        action_modifiers.heap(&mut meter)?;
        rules.heap(&mut meter)?;
        fingerprint.heap(&mut meter)?;
        Ok(meter.bytes)
    }
}
fn same_provider_storage(a: &EffectiveProviderPlan, b: &EffectiveProviderPlan) -> bool {
    use super::support::SelectedSupportReach;
    let EffectiveProviderPlan {
        skill,
        flow,
        amounts,
        effects,
        selected_recipient_reach,
        selected_recipient_excludes_caster,
    } = a;
    Arc::ptr_eq(skill, &b.skill)
        && Arc::ptr_eq(flow, &b.flow)
        && Arc::ptr_eq(amounts, &b.amounts)
        && Arc::ptr_eq(effects, &b.effects)
        && *selected_recipient_excludes_caster == b.selected_recipient_excludes_caster
        && match (selected_recipient_reach, &b.selected_recipient_reach) {
            (None, None) => true,
            (Some(SelectedSupportReach::Radius(a)), Some(SelectedSupportReach::Radius(b))) => {
                a == b
            }
            (
                Some(SelectedSupportReach::Pattern {
                    range: a,
                    end_on_exit: x,
                }),
                Some(SelectedSupportReach::Pattern {
                    range: b,
                    end_on_exit: y,
                }),
            ) => Arc::ptr_eq(a, b) && x == y,
            _ => false,
        }
}

// The same exhaustive visitor prices copy admission and retained ownership.
// Shared pointees remain conservatively priced per edge, even for Arc clones.
pub(super) fn admit_owned_copy<T: Heap + ?Sized>(
    value: &T,
    work: &mut Work,
) -> Result<(), RelicPlanError> {
    let mut meter = Extent {
        bytes: 0,
        work,
        shared_pointees: true,
        comparison: false,
    };
    meter.allocation(Some(std::mem::size_of_val(value)))?;
    value.heap(&mut meter)?;
    let bytes = meter.bytes;
    work.reserve(Some(bytes), "relic_structural_owned_copy")
}

// A derived Clone duplicates owned containers but only increments Arc handles.
// This admission is for that operation alone. Later COW mutations must admit
// their pointee before detaching; completed-plan measurement still traverses
// every shared owning edge, including serde-skipped data.
pub(super) fn admit_shared_clone<T: Heap + ?Sized>(
    value: &T,
    work: &mut Work,
) -> Result<(), RelicPlanError> {
    let mut meter = Extent {
        bytes: 0,
        work,
        shared_pointees: false,
        comparison: false,
    };
    meter.allocation(Some(std::mem::size_of_val(value)))?;
    value.heap(&mut meter)?;
    let bytes = meter.bytes;
    work.reserve(Some(bytes), "relic_structural_shared_clone")
}

fields!(crate::game::ability::action::ActionPolicy; [modifiers]; [external_interrupt_protected]);
fields!(crate::game::ability::action::ActionModifier; [id, relic_definition, change]; [variant]);
impl Heap for crate::game::ability::action::ActionChange {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        use crate::game::ability::action::ActionChange::*;
        m.scan()?;
        match self {
            Area { step, change } => {
                step.heap(m)?;
                match change {
                    crate::game::ability::action::AreaChange::RampRate {
                        reached_rules, ..
                    } => reached_rules.heap(m),
                    _ => Ok(()),
                }
            }
            SupportRange {
                step,
                effect,
                pattern,
            } => {
                step.heap(m)?;
                effect.heap(m)?;
                pattern.heap(m)
            }
            Range { steps, pattern } => {
                steps.heap(m)?;
                pattern.heap(m)
            }
            Multihit { steps, .. } => steps.heap(m),
            SupportRadius { step, effect, .. }
            | TauntDamage { step, effect, .. }
            | Duration { step, effect, .. }
            | ControlVulnerability { step, effect, .. } => {
                step.heap(m)?;
                effect.heap(m)
            }
            ProtectInterruption => Ok(()),
        }
    }
}

fields!(crate::game::ability::sustained::HitModifierSource; [definition, rule]; [copy_uuid]);

impl Heap for crate::game::ability::counter_channel::CounterChannelDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        use crate::game::ability::counter_channel::*;
        fn range(r: &CounterRange, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
            r.definition.heap(m)?;
            r.pattern.heap(m)
        }
        self.definition.heap(m)?;
        m.allocation(
            self.entries
                .capacity()
                .checked_mul(size_of::<CounterEntry>()),
        )?;
        for e in &self.entries {
            m.scan()?;
            e.key.heap(m)?;
            e.selection_range.heap(m)?;
            m.allocation(e.phases.capacity().checked_mul(size_of::<CounterPhase>()))?;
            for p in &e.phases {
                m.scan()?;
                p.definition.heap(m)?;
                match &p.range {
                    PhaseRange::Fixed(r) => range(r, m)?,
                    PhaseRange::Consumed(v) => {
                        m.allocation(v.capacity().checked_mul(size_of::<(u32, CounterRange)>()))?;
                        for (_, r) in v {
                            m.scan()?;
                            range(r, m)?;
                        }
                    }
                }
                m.allocation(p.queries.capacity().checked_mul(size_of::<PhaseQuery>()))?;
                for q in &p.queries {
                    m.scan()?;
                    q.definition.heap(m)?;
                    if let PhaseEffect::Damage { amount, .. } | PhaseEffect::Heal { amount } =
                        &q.effect
                    {
                        amount.definition.heap(m)?;
                    }
                }
            }
        }
        m.allocation(
            self.modifiers
                .capacity()
                .checked_mul(size_of::<CounterModifier>()),
        )?;
        for modifier in &self.modifiers {
            m.scan()?;
            modifier.id.heap(m)?;
            modifier.relic_definition.heap(m)?;
            match &modifier.change {
                CounterChange::Range {
                    definition,
                    pattern,
                } => {
                    definition.heap(m)?;
                    pattern.heap(m)?;
                }
                CounterChange::Healing { definition, .. } => definition.heap(m)?,
                CounterChange::HealBuff {
                    definition,
                    profile,
                    ..
                } => {
                    definition.heap(m)?;
                    profile.definition.heap(m)?;
                }
                CounterChange::Protection { entry } => entry.heap(m)?,
                _ => {}
            }
        }
        Ok(())
    }
}

fields!(crate::game::ability::relation_action::PlanRange; [definition,pattern]; []);
fields!(crate::game::ability::relation_action::PlanStatus; [definition,count]; [kind,duration_ms,percent]);
fields!(crate::game::ability::relation_action::NamedActionPlan; [definition,op]; []);
fields!(crate::game::ability::relation_action::RelationActionEntry; [definition,selector]; [plan,next_basic,interrupt_on_silence,protected,self_damage,manual_stop]);
fields!(crate::game::ability::relation_action::RelationActionModifier; [id,relic_definition,change]; []);
fields!(crate::game::ability::relation_action::RelationActionDef; [definition,stock_definition,relation_definition,mode_definition,menu_entry,selection_range,normal,active,plans,modifiers]; [admission,initial_stock,stock_capacity,period_ms,periodic_gain,reduction,shield_threshold,shield_percent,shield_duration_ms,mode_damage_percent]);
impl Heap for crate::game::ability::relation_action::ActionCount {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        use crate::game::ability::relation_action::ActionCount::*;
        match self {
            Quotient { count, .. } => count.heap(m),
            Sum { first, second } => {
                first.heap(m)?;
                second.heap(m)
            }
            _ => Ok(()),
        }
    }
}
impl Heap for crate::game::ability::relation_action::ActionPlan {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        use crate::game::ability::relation_action::ActionPlan::*;
        match self {
            Strike { range, after, .. } => {
                range.heap(m)?;
                after.heap(m)
            }
            Repeat { count, .. } | Timed { count, .. } => count.heap(m),
            Spend { amount, after, .. } => {
                amount.heap(m)?;
                after.heap(m)
            }
            _ => Ok(()),
        }
    }
}
impl Heap for crate::game::ability::relation_action::RelationActionChange {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        use crate::game::ability::relation_action::RelationActionChange::*;
        match self {
            Range {
                definition,
                pattern,
            } => {
                definition.heap(m)?;
                pattern.heap(m)
            }
            Period { definition, .. }
            | StatusCount { definition, .. }
            | PrimaryArea { definition }
            | ControlVulnerability { definition, .. } => definition.heap(m),
            Multihit { entry, .. } | Retarget { entry, .. } => entry.heap(m),
            _ => Ok(()),
        }
    }
}

fields!(crate::game::ability::input_plan::ShotStockDef; [definition]; [initial,capacity]);
fields!(crate::game::ability::input_plan::HostileCohortDef; [definition,attack_range,weakness_family]; [interval_ms,hp,power,speed,physical_defense,magic_defense,block,leak,damage_type,attack_interval_ms,weakness_percent,kill_gain]);
fields!(crate::game::ability::input_plan::ShotProjectileDef; [definition,after]; [power,damage_type,speed,range,width,lifetime_ms,max_hits,air,obstacle_stop]);
fields!(crate::game::ability::input_plan::ShotModifier; [id,relic_definition,requires,change]; []);
fields!(crate::game::ability::input_plan::InputPlanDef; [definition,normal,enhanced,cohort,placement_range,normal_projectile,enhanced_projectile,modifiers]; [admission,stock_interval_ms,stock_gain,allow_duplicates]);
impl Heap for crate::game::ability::input_plan::ShotChange {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        use crate::game::ability::input_plan::ShotChange::*;
        match self {
            KillGain { .. } => Ok(()),
            Wound { profiles, .. } => profiles.heap(m),
            WoundGrant { definition, .. } | ControlVulnerability { definition, .. } => {
                definition.heap(m)
            }
            Status { profile, status } => {
                profile.heap(m)?;
                status.heap(m)
            }
            Push { profile, .. } => profile.heap(m),
            Explosion { profile, range, .. } => {
                profile.heap(m)?;
                range.heap(m)
            }
        }
    }
}

fields!(crate::game::ability::prepared_volley::CreditLedgerDef; [definition]; [grant,credit_limit]);
fields!(crate::game::ability::prepared_volley::PreparationPolicy; []; [maximum,first_ms,last_ms,hold_ms,deadline_ms,decay_ms]);
fields!(crate::game::ability::prepared_volley::HomingImpactDef; [definition,area,after]; [power,damage_type,speed,range,width,lifetime_ms,air,obstacle_stop]);
fields!(crate::game::ability::prepared_volley::VolleyModifier; [id,relic_definition,change]; []);
fields!(crate::game::ability::prepared_volley::PreparedVolleyDef; [definition,ledger,mark_definition,policy,selection_range,projectile,modifiers]; [admission,mark_duration_ms,recovery,selection_air]);
impl Heap for crate::game::ability::prepared_volley::VolleyChange {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        use crate::game::ability::prepared_volley::VolleyChange::*;
        match self {
            Area { pattern } => pattern.heap(m),
            Status { status } => status.heap(m),
            _ => Ok(()),
        }
    }
}

fields!(crate::game::ability::paired_areas::PairedAreasDef; [definition,selection_range,first,second,intersection,modifiers]; [admission,duration_ms,interval_ms,healing,transfer_percent]);
fields!(crate::game::ability::paired_areas::PairModifier; [id,relic_definition,change]; []);
impl Heap for crate::game::ability::paired_areas::PairChange {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        use crate::game::ability::paired_areas::PairChange::*;
        match self {
            Range { pattern, .. } | CoincidentBurst { pattern, .. } => pattern.heap(m),
            HealBuff { profile, .. } => profile.definition.heap(m),
            _ => Ok(()),
        }
    }
}

fields!(crate::game::ability::sustained::TargetStateProfile; [definition]; [attack_percent,defense_percent,attack_speed_points,hp_floor_one,damage_invulnerable,basic_attack_locked,active_skill_locked,received_heal_percent,end_on_source_exit]);

fields!(crate::game::ability::mobile_effect::FusionPlacementDef; [definition,selection_range,tiers,seed]; [admission,primary,adjacent,max_units,max_tiles]);
fields!(crate::game::ability::mobile_effect::FusionTier; [profile]; [minimum]);
fields!(crate::game::ability::mobile_effect::FusionSeed; [definition,relic_definition,modifiers]; [duration_ms]);
fields!(crate::game::ability::mobile_effect::SeedDurationModifier; [id,relic_definition]; [negative,percent]);
fields!(crate::game::ability::mobile_effect::MobileProfile; [definition,detection,range,power,modifiers]; [motion,damage_type,speed_units_per_second,delay_ms,lifetime_ms]);
fields!(crate::game::ability::mobile_effect::MobileModifier; [id,relic_definition,change]; []);
impl Heap for crate::game::ability::mobile_effect::MobilePower {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        use crate::game::ability::mobile_effect::MobilePower::*;
        match self {
            Weapon { .. } => Ok(()),
            Add { base, .. } | Fusion { base, .. } => base.heap(m),
        }
    }
}
impl Heap for crate::game::ability::mobile_effect::MobileChange {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        use crate::game::ability::mobile_effect::MobileChange::*;
        match self {
            Range { pattern } => pattern.heap(m),
            _ => Ok(()),
        }
    }
}

fields!(crate::game::ability::status_strike::StatusStrikeDef; [definition,range,range_modifiers,power,from,to,mobile,death]; [admission,damage_type,owned_damage,limit]);
fields!(crate::game::ability::status_strike::ReactiveStatusDef; [definition,modifiers]; [kind,duration_ms,refresh]);
fields!(crate::game::ability::status_strike::ReactiveModifier; [id,relic_definition]; [change]);
fields!(crate::game::ability::status_strike::StatusDeathMobileDef; [definition,relic_definition,states]; []);

fields!(crate::game::ability::guard_charge::GuardChargeDef; [definition,relic_definition,range]; [capacity,primary,splash]);
fields!(crate::game::ability::guard_charge::GuardCounterDef; [charge]; [weapon,damage_type]);

fields!(ContentIdentity; [semantic_sha256]; [contract]);
fields!(crate::game::data::relic_data::RelicDamageEffect; [id, selector]; [flat, percent]);
fields!(RelicRuleDef; [id, selector, when, apply]; []);
fields!(CausalSubscriptionDef; [source, payload]; [event]);
impl Heap for RelicRuleActionDef {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Grant(v) => v.heap(m),
            Self::Subscribe(v) => v.heap(m),
            Self::Periodic { payload, .. } => payload.heap(m),
            Self::Recipe(v) => v.heap(m),
            Self::Wedge { .. } | Self::Edit { .. } => Ok(()),
        }
    }
}

fields!(crate::game::ability::sustained::HitVulnerability; [definition, modifier_source]; [percent, duration_ms]);

// Bound a typed equality check before traversing it. Includes fields skipped
// by serialization, shared pointees and the text contents compared by Eq.
pub(super) fn admit_comparison<T: Heap + ?Sized>(
    value: &T,
    work: &mut Work,
) -> Result<(), RelicPlanError> {
    let mut meter = Extent {
        bytes: 0,
        work,
        shared_pointees: true,
        comparison: true,
    };
    value.heap(&mut meter)
}

impl<T: Heap> Heap for crate::game::ability::condition::ConditionLogic<T> {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::All(items) | Self::Any(items) => items.heap(m),
            Self::Not(item) => item.heap(m),
        }
    }
}
impl Heap for crate::game::ability::state_program::StateCondition {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Always(_) | Self::EventVictimHadOwnerDot => Ok(()),
            Self::Predicate(call) => call.heap(m),
            Self::Compare { left, right, .. } => {
                left.heap(m)?;
                right.heap(m)
            }
            Self::Bool { field, .. } => field.heap(m),
            Self::Logic(logic) => logic.heap(m),
        }
    }
}

impl Heap for crate::game::ability::condition::PredicateCall {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        self.name.heap(m)?;
        self.arguments.heap(m)
    }
}
impl Heap for crate::game::ability::condition::PredicateNamedArgument {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        self.name.heap(m)?;
        self.value.heap(m)
    }
}
impl Heap for crate::game::ability::condition::PredicateArgument {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Literal(value) => value.heap(m),
            Self::Parameter(name) | Self::Unresolved(name) => name.heap(m),
        }
    }
}
impl Heap for crate::game::ability::condition::PredicateValue {
    fn heap(&self, m: &mut Extent<'_>) -> Result<(), RelicPlanError> {
        m.scan()?;
        match self {
            Self::Amount(value) => value.heap(m),
            Self::BoolField(field) => field.heap(m),
            Self::FlowResult(result) => {
                result.node_id.heap(m)?;
                result.effect_id.heap(m)
            }
            Self::FlowLiteral(_) => Ok(()),
        }
    }
}
