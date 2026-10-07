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
/// Passive resource gain, cast reset and other effect kinds are not included.
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
    pub fn latest_nonzero_sourced_hp(self) -> Option<ApplicationView<'a>> {
        self.0
            .latest_nonzero_sourced_hp
            .as_ref()
            .map(ApplicationView)
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
