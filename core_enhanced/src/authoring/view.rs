//! Borrowed observations contain no mutable runtime or shared-owner handles.
//! Constructing a view never encodes, copies source strings or charges a trial.
use serde::Serialize;
use uuid::Uuid;

use crate::game::{
    ability::authoring::{EffectOrigin, EffectSource},
    battle::{
        cooldown::CooldownSource,
        core::authoring_observation::{ApplicationWitness, MandatoryObservation, ObservationKey},
        event_log::BattleEventCause,
        ids::UnitInstanceId,
        skill_source::{SkillEffectExecutionSource, SkillProcOrigin},
        types::BattleVerdict,
    },
    world::{
        authoring_lifecycle::{TrialBattleStatus, TrialBattleWitness, TrialLifecycle},
        authoring_observation::{TerminalCapsule, TrialObservationView, TrialTraceView},
    },
};

pub use crate::game::battle::core::area_observation::{AreaEffectSource, AreaWitness, AreaChange, AreaMembership};
pub use crate::game::battle::core::equipment_magazine::MagazineWitness;
pub use crate::game::battle::core::attack_continuity::AttackContinuityWitness;
pub use crate::game::battle::core::release_gap::ReleaseGapWitness;
pub use crate::game::battle::core::shared_cohort::SharedCohortWitness;
pub use crate::game::battle::core::cohort_bounty::BountyWitness;
pub use crate::game::battle::core::equipment_forms::FormWitness;
pub use crate::game::battle::core::equipment_death_burst::DeathBurstWitness;
pub use crate::game::battle::core::equipment_primary_damage::{PrimaryDamageContribution, PrimaryDamageWitness};
pub use crate::game::battle::core::area_damage::{AreaDamageContribution, AreaDamageWitness};
pub use crate::game::battle::core::nearby_ally_damage::NearbyAllyDamageWitness;
pub use crate::game::battle::core::conditional_damage::ConditionalDamageWitness;
pub use crate::game::battle::core::direct_penetration::DirectPenetrationWitness;
pub use crate::game::battle::core::equipment_defense::IncomingConditionalWitness;
pub use crate::game::battle::core::equipment_resonance_gain::ResonanceGainWitness;
pub use crate::game::battle::core::equipment_attack::{EquipmentSpeedContribution, EquipmentSpeedWitness};
pub use crate::game::battle::core::equipment_defense::{OverhealReserveWitness, DeferredDamageWitness, LastWillWitness, EmployeeTransformationWitness};
pub use crate::game::battle::core::weapon_stages::WeaponStageWitness;
pub use crate::game::battle::core::equipment_support::{ShieldBeneficiaryDamageWitness,ReceivedHealWitness,ControlDurationWitness,EquipmentSupportWitness, HealBarrierWitness, BasicHealEchoWitness, ActualHealReactionWitness};
pub use crate::game::battle::core::equipment_defense::{WearerReductionWitness, IncomingFlatWitness, ReadyWaitWitness, NearbyDefenseWitness, ChannelOpeningWitness, IdleHpGuardWitness, SkillBarrierWitness, BasicEvasionWitness, AdaptiveShieldWitness};

pub use crate::game::battle::core::authoring_observation::{
    AppliedChange as ApplicationChange, BuffApplicationState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BattleKey {
    pub entry_ordinal: u64,
    pub battle_uuid: Uuid,
}
impl From<ObservationKey> for BattleKey {
    fn from(key: ObservationKey) -> Self {
        Self {
            entry_ordinal: key.entry_ordinal,
            battle_uuid: key.battle_uuid,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(transparent)]
pub struct LifecycleView(pub(super) TrialLifecycle);
impl LifecycleView {
    pub fn entries(self) -> u64 {
        self.0.entries
    }
    pub fn total_elapsed_ms(self) -> u64 {
        self.0.total_elapsed_ms
    }
    pub fn is_ready(self) -> bool {
        self.0.is_ready()
    }
    pub fn current(self) -> Option<BattleLifecycleView> {
        self.0.current.map(BattleLifecycleView)
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(transparent)]
pub struct BattleLifecycleView(TrialBattleWitness);
impl BattleLifecycleView {
    pub fn key(self) -> BattleKey {
        BattleKey {
            entry_ordinal: self.0.entry_ordinal,
            battle_uuid: self.0.battle_uuid,
        }
    }
    pub fn time_zero_completed(self) -> bool {
        self.0.time_zero_completed
    }
    pub fn last_battle_time_ms(self) -> u64 {
        self.0.last_battle_time_ms
    }
    pub fn status(self) -> &'static str {
        match self.0.status {
            TrialBattleStatus::EntryPending => "entry_pending",
            TrialBattleStatus::Running => "running",
            TrialBattleStatus::Finished => "finished",
            TrialBattleStatus::Exited => "exited",
        }
    }
    pub fn terminal_verdict(self) -> Option<BattleVerdict> {
        self.0.terminal_verdict
    }
}

#[derive(Serialize)]
#[serde(transparent)]
pub struct ObservationView<'a>(pub(super) TrialObservationView<'a>);
impl<'a> ObservationView<'a> {
    pub fn run_resources(&self) -> Option<&'a crate::game::resource::RunResourceState> {
        self.0.run_resources
    }
    pub fn active_typed_resources(&self) -> Option<crate::game::resource::ResourceView<'a>> {
        self.0.active_typed_resources
    }
    pub fn active_runtime_facts(
        &self,
    ) -> Option<crate::game::battle::runtime_facts::RuntimeFactView<'a>> {
        self.0.active_runtime_facts
    }
    pub fn active(&self) -> Option<BattleObservationView<'a>> {
        self.0.active.map(BattleObservationView)
    }
    pub fn last_terminal(&self) -> Option<TerminalObservationView<'a>> {
        self.0.last_terminal.map(TerminalObservationView)
    }
}

/// Counts describe HP/Stats assignments, actual ModifyResonance changes and Buff applications.
/// Also includes finite sustained activation/removal witnesses. Passive resource
/// gain, cast reset and other effect stores are not included.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(transparent)]
pub struct BattleObservationView<'a>(&'a MandatoryObservation);
impl<'a> BattleObservationView<'a> {
    pub fn key(self) -> Option<BattleKey> {
        self.0.key.map(Into::into)
    }
    pub fn applications(self) -> u64 {
        self.0.applications
    }
    pub fn nonzero_hp_applications(self) -> u64 {
        self.0.nonzero_hp_applications
    }
    pub fn sourced_applications(self) -> u64 {
        self.0.sourced_applications
    }
    pub fn latest(self) -> Option<ApplicationView<'a>> {
        self.0.latest.as_ref().map(ApplicationView)
    }
    pub fn latest_sourced_resonance(self) -> Option<ApplicationView<'a>> {
        self.0.latest_sourced_resonance.as_ref().map(ApplicationView)
    }
    pub fn latest_nonzero_sourced_hp(self) -> Option<ApplicationView<'a>> {
        self.0
            .latest_nonzero_sourced_hp
            .as_ref()
            .map(ApplicationView)
    }
    pub fn latest_area(self) -> Option<&'a AreaWitness> { self.0.latest_area.as_ref() }
    pub fn latest_ready_wait(self) -> Option<&'a ReadyWaitWitness> { self.0.latest_ready_wait.as_ref() }
    pub fn latest_nearby_defense(self) -> Option<&'a NearbyDefenseWitness> { self.0.latest_nearby_defense.as_ref() }
    pub fn latest_primary_damage(self) -> Option<&'a PrimaryDamageWitness> { self.0.latest_primary_damage.as_ref() }
    pub fn latest_area_damage(self) -> Option<&'a AreaDamageWitness> { self.0.latest_area_damage.as_ref() }
    pub fn latest_nearby_ally_damage(self) -> Option<&'a NearbyAllyDamageWitness> { self.0.latest_nearby_ally_damage.as_ref() }
    pub fn latest_conditional_damage(self) -> Option<&'a ConditionalDamageWitness> { self.0.latest_conditional_damage.as_ref() }
    pub fn latest_direct_penetration(self) -> Option<&'a DirectPenetrationWitness> { self.0.latest_direct_penetration.as_ref() }
    pub fn latest_conditional_reduction(self) -> Option<&'a IncomingConditionalWitness> { self.0.latest_conditional_reduction.as_ref() }
    pub fn latest_actual_heal_reaction(self) -> Option<&'a ActualHealReactionWitness> { self.0.latest_actual_heal_reaction.as_ref() }
    pub fn latest_resonance_gain(self) -> Option<&'a ResonanceGainWitness> { self.0.latest_resonance_gain.as_ref() }
    pub fn latest_equipment_speed(self) -> Option<&'a EquipmentSpeedWitness> { self.0.latest_equipment_speed.as_ref() }
    pub fn latest_employee_transformation(self) -> Option<&'a EmployeeTransformationWitness> { self.0.latest_employee_transformation.as_ref() }
    pub fn latest_last_will(self) -> Option<&'a LastWillWitness> { self.0.latest_last_will.as_ref() }
    pub fn latest_weapon_stage(self) -> Option<&'a WeaponStageWitness> { self.0.latest_weapon_stage.as_ref() }
    pub fn latest_adaptive_shield(self) -> Option<&'a AdaptiveShieldWitness> { self.0.latest_adaptive_shield.as_ref() }
    pub fn latest_channel_opening(self) -> Option<&'a ChannelOpeningWitness> { self.0.latest_channel_opening.as_ref() }
    pub fn latest_idle_hp_guard(self) -> Option<&'a IdleHpGuardWitness> { self.0.latest_idle_hp_guard.as_ref() }
    pub fn latest_attack_continuity(self) -> Option<&'a AttackContinuityWitness> { self.0.latest_attack_continuity.as_ref() }
    pub fn latest_release_gap(self) -> Option<&'a ReleaseGapWitness> { self.0.latest_release_gap.as_ref() }
    pub fn latest_wearer_reduction(self) -> Option<&'a WearerReductionWitness> { self.0.latest_wearer_reduction.as_ref() }
    pub fn latest_incoming_flat(self) -> Option<&'a IncomingFlatWitness> { self.0.latest_incoming_flat.as_ref() }
    pub fn latest_shield_beneficiary_damage(self) -> Option<&'a ShieldBeneficiaryDamageWitness> { self.0.latest_shield_beneficiary_damage.as_ref() }
    pub fn latest_received_heal(self) -> Option<&'a ReceivedHealWitness> { self.0.latest_received_heal.as_ref() }
    pub fn latest_control_duration(self) -> Option<&'a ControlDurationWitness> { self.0.latest_control_duration.as_ref() }
    pub fn latest_skill_barrier(self) -> Option<&'a SkillBarrierWitness> { self.0.latest_skill_barrier.as_ref() }
    pub fn latest_equipment_support(self) -> Option<&'a EquipmentSupportWitness> { self.0.latest_equipment_support.as_ref() }
    pub fn latest_heal_barrier(self) -> Option<&'a HealBarrierWitness> { self.0.latest_heal_barrier.as_ref() }
    pub fn latest_shared_cohort(self) -> Option<&'a SharedCohortWitness> { self.0.latest_shared_cohort.as_ref() }
    pub fn latest_equipment_death_burst(self) -> Option<&'a DeathBurstWitness> { self.0.latest_equipment_death_burst.as_ref() }
    pub fn latest_equipment_form(self) -> Option<&'a FormWitness> { self.0.latest_equipment_form.as_ref() }
    pub fn latest_bounty(self) -> Option<&'a BountyWitness> { self.0.latest_bounty.as_ref() }
    pub fn latest_magazine(self) -> Option<&'a MagazineWitness> { self.0.latest_magazine.as_ref() }
    pub fn latest_deferred_damage(self) -> Option<&'a DeferredDamageWitness> { self.0.latest_deferred_damage.as_ref() }
    pub fn latest_overheal_reserve(self) -> Option<&'a OverhealReserveWitness> { self.0.latest_overheal_reserve.as_ref() }
    pub fn latest_basic_heal_echo(self) -> Option<&'a BasicHealEchoWitness> { self.0.latest_basic_heal_echo.as_ref() }
    pub fn latest_basic_evasion(self) -> Option<&'a BasicEvasionWitness> { self.0.latest_basic_evasion.as_ref() }
    pub fn area_changes(self) -> u64 { self.0.area_changes }
    pub fn latest_sourced_hit_counter(self) -> Option<ApplicationView<'a>> {
        self.0.latest_sourced_hit_counter.as_ref().map(ApplicationView)
    }
    pub fn latest_sourced_sustained(self) -> Option<ApplicationView<'a>> {
        self.0.latest_sourced_sustained.as_ref().map(ApplicationView)
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(transparent)]
pub struct TerminalObservationView<'a>(&'a TerminalCapsule);
impl<'a> TerminalObservationView<'a> {
    pub fn runtime_facts(self) -> crate::game::battle::runtime_facts::RuntimeFactView<'a> {
        self.0.runtime_facts.view()
    }
    pub fn key(self) -> BattleKey {
        self.0.key.into()
    }
    pub fn actual_time_ms(self) -> u64 {
        self.0.actual_time_ms
    }
    pub fn outcome_id(self) -> &'a str {
        self.0.outcome_id.as_str()
    }
    pub fn verdict(self) -> BattleVerdict {
        self.0.verdict
    }
    pub fn mandatory(self) -> BattleObservationView<'a> {
        BattleObservationView(&self.0.mandatory)
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(transparent)]
pub struct ApplicationView<'a>(&'a ApplicationWitness);
impl<'a> ApplicationView<'a> {
    pub fn ordinal(self) -> u64 {
        self.0.ordinal
    }
    pub fn time_ms(self) -> u64 {
        self.0.time_ms
    }
    pub fn source_unit(self) -> Option<UnitInstanceId> {
        self.0.source_unit
    }
    pub fn target_unit(self) -> UnitInstanceId {
        self.0.target_unit
    }
    pub fn change(self) -> ApplicationChange {
        self.0.change
    }
    pub fn source(self) -> Option<EffectSourceView<'a>> {
        self.0.source.as_deref().map(EffectSourceView)
    }
    pub fn authored_source(self) -> Option<AuthoredEffectView<'a>> {
        self.0.source.as_ref().map(|s| s.authored.as_ref())
            .or(self.0.authored.as_deref()).map(AuthoredEffectView)
    }
    pub fn area_source(self) -> Option<&'a AreaEffectSource> { self.0.area_source.as_deref() }
    pub fn equipment_source(self) -> Option<crate::game::battle::runtime_facts::EquipmentSourceSnapshot> {
        self.0.equipment
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EffectSourceView<'a>(&'a SkillEffectExecutionSource);
impl<'a> EffectSourceView<'a> {
    pub fn authored(self) -> AuthoredEffectView<'a> {
        AuthoredEffectView(&self.0.authored)
    }
    pub fn skill_id(self) -> &'a str {
        self.0.cast.skill_id.as_str()
    }
    pub fn caster_instance_id(self) -> UnitInstanceId {
        self.0.cast.caster_instance_id
    }
    pub fn cast_seq(self) -> u64 {
        self.0.cast.cast_seq
    }
    pub fn ability_event_seq(self) -> u64 {
        self.0.cast.ability_event_seq
    }
    pub fn cause(self) -> BattleEventCause {
        self.0.cast.cause
    }
    pub fn proc_origin(self) -> Option<ProcOriginView<'a>> {
        self.0.cast.proc_origin.as_ref().map(ProcOriginView)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AuthoredEffectView<'a>(&'a EffectSource);
impl<'a> AuthoredEffectView<'a> {
    pub(crate) fn source(self) -> &'a EffectSource {
        self.0
    }
    pub fn source_file(self) -> &'a str {
        &self.0.source_file
    }
    pub fn definition_id(self) -> &'a str {
        self.0.site.action.definition_id.as_str()
    }
    pub fn action_id(self) -> &'a str {
        &self.0.site.action.action_id
    }
    pub fn effect_id(self) -> &'a str {
        &self.0.site.effect_id
    }
    pub fn field_path(self) -> &'a str {
        &self.0.field_path
    }
    pub fn origin_definition_id(self) -> &'a str {
        self.0.origin_definition_id.as_str()
    }
    pub fn added_by(self) -> Option<&'a str> {
        match &self.0.origin {
            EffectOrigin::Original => None,
            EffectOrigin::Added {
                source_definition_id,
            } => Some(source_definition_id.as_str()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProcOriginView<'a>(&'a SkillProcOrigin);
impl ProcOriginView<'_> {
    pub fn event_seq(self) -> u64 {
        self.0.event_seq
    }
    pub fn activation_source(self) -> CooldownSource {
        self.0.activation_source
    }
    pub fn binding_index(self) -> usize {
        self.0.binding_index
    }
}

/// Optional trace is deliberately separate from serializable mandatory observations.
pub struct SessionTraceView<'a>(pub(super) TrialTraceView<'a>);
impl<'a> SessionTraceView<'a> {
    pub fn active(&self) -> Option<BattleTraceView<'a>> {
        self.0.active.map(|(key, trace)| BattleTraceView {
            key: key.into(),
            trace,
        })
    }
    pub fn last_terminal(&self) -> Option<BattleTraceView<'a>> {
        self.0.last_terminal.map(|(key, trace)| BattleTraceView {
            key: key.into(),
            trace,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BattleTraceView<'a> {
    key: BattleKey,
    trace: super::trace::TraceView<'a>,
}
impl<'a> BattleTraceView<'a> {
    pub fn key(self) -> BattleKey {
        self.key
    }
    pub fn enabled(self) -> bool {
        self.trace.enabled
    }
    pub fn ndjson(self) -> &'a [u8] {
        self.trace.ndjson
    }
    pub fn retained_rows(self) -> u64 {
        self.trace.retained_rows
    }
    pub fn last_retained_ordinal(self) -> Option<u64> {
        self.trace.last_retained_ordinal
    }
    pub fn gap(self) -> Option<super::TraceGap> {
        self.trace.gap
    }
}
