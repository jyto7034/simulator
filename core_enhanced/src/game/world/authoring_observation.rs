//! Immutable terminal observation ownership, separate from saved gameplay.
use super::GameCore;
use crate::{
    authoring::{
        clone_shape::controlled_clone,
        control,
        trace::{OptionalTrace, TraceOptions, TraceView},
    },
    game::{
        battle::{
            core::authoring_observation::{MandatoryObservation, ObservationKey},
            types::{BattleOutcomeId, BattleVerdict},
        },
        behavior::GameError,
    },
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TerminalCapsule {
    #[serde(skip)]
    retained_extent: usize,
    pub runtime_facts: Arc<crate::game::battle::runtime_facts::RuntimeFactSnapshot>,
    pub key: ObservationKey,
    pub actual_time_ms: u64,
    pub outcome_id: BattleOutcomeId,
    pub verdict: BattleVerdict,
    pub mandatory: MandatoryObservation,
    #[serde(skip)]
    pub trace: OptionalTrace,
}
crate::authoring::clone_shape::struct_shape!(TerminalCapsule {
    retained_extent,
    runtime_facts,
    key,
    actual_time_ms,
    outcome_id,
    verdict,
    mandatory,
    trace
});

#[derive(Debug, Clone, Default)]
pub(super) struct TrialObservation {
    pub(super) trace_options: TraceOptions,
    pub(super) amount_observation_enabled: bool,
    last_terminal: Option<Arc<TerminalCapsule>>,
    pub(super) coverage_plan:Option<Arc<crate::mod_validation::CoveragePlan>>,
    coverage_total:Option<crate::mod_validation::CoverageReport>,
}
crate::authoring::clone_shape::struct_shape!(TrialObservation {
    trace_options,
    amount_observation_enabled,
    last_terminal,
    coverage_plan,
    coverage_total
});

pub(crate) struct TrialTraceView<'a> {
    pub active: Option<(ObservationKey, TraceView<'a>)>,
    pub last_terminal: Option<(ObservationKey, TraceView<'a>)>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum StoredAmountWindow {
    Active,
    LastTerminal,
}

pub(crate) struct StoredAmountView<'a> {
    pub key: ObservationKey,
    pub record:
        Option<&'a Arc<crate::game::battle::core::amount_observation::RecordedAmountEvaluation>>,
}

#[derive(Serialize)]
pub(crate) struct TrialObservationView<'a> {
    pub run_resources: Option<&'a crate::game::resource::RunResourceState>,
    pub active_typed_resources: Option<crate::game::resource::ResourceView<'a>>,
    pub active_runtime_facts: Option<crate::game::battle::runtime_facts::RuntimeFactView<'a>>,
    pub active: Option<&'a MandatoryObservation>,
    pub last_terminal: Option<&'a TerminalCapsule>,
}

fn invalid() -> GameError {
    GameError::InvalidStaticData("invalid authoring observation owner".into())
}

impl GameCore {
    pub(super) fn record_lab_definition(&mut self, kind:&str,id:&str,family:&str)->Result<(),GameError>{
        if let Some(report)=self.authoring_observation.as_mut().and_then(|o|o.coverage_total.as_mut()) { report.record_definition(kind,id,family,crate::mod_validation::CoverageStage::ConsumedOrEffect)?; }
        Ok(())
    }
    pub(crate) fn set_lab_coverage(&mut self,plan:Option<Arc<crate::mod_validation::CoveragePlan>>)->Result<(),GameError>{
        let owner=self.authoring_observation.as_mut().ok_or_else(invalid)?;
        owner.coverage_total=plan.as_ref().map(|p|controlled_clone(&p.report)).transpose()?;
        owner.coverage_plan=plan;Ok(())
    }
    pub(crate) fn lab_coverage(&self)->Result<Option<crate::mod_validation::CoverageReport>,GameError>{
        let owner=self.authoring_observation.as_ref().ok_or_else(invalid)?;
        let Some(total)=&owner.coverage_total else{return Ok(None)};
        let mut report=controlled_clone(total)?;
        if let Some(active)=self.state.active_battle.as_ref() {if let Some(coverage)=active.battle.authoring_observation()?.and_then(|m|m.coverage.as_ref()) {report.merge(&coverage.report)?;}}
        Ok(Some(report))
    }
    pub(crate) fn set_authoring_trace_options(&mut self, options: TraceOptions) {
        self.authoring_observation
            .as_mut()
            .expect("new authoring owner")
            .trace_options = options;
    }

    pub(crate) fn set_amount_observation_enabled(&mut self, enabled: bool) {
        self.authoring_observation
            .as_mut()
            .expect("new authoring owner")
            .amount_observation_enabled = enabled;
    }

    pub(crate) fn amount_observation_enabled(&self) -> Result<bool, GameError> {
        Ok(self
            .authoring_observation
            .as_ref()
            .ok_or_else(invalid)?
            .amount_observation_enabled)
    }

    pub(crate) fn stored_amount(
        &self,
        window: StoredAmountWindow,
    ) -> Result<Option<StoredAmountView<'_>>, GameError> {
        let owner = self.authoring_observation.as_ref().ok_or_else(invalid)?;
        match window {
            StoredAmountWindow::Active => {
                let Some(active) = self.state.active_battle.as_ref() else {
                    return Ok(None);
                };
                let mandatory = active.battle.authoring_observation()?.ok_or_else(invalid)?;
                let key = mandatory.key.ok_or_else(invalid)?;
                if key.battle_uuid != active.battle_uuid {
                    return Err(invalid());
                }
                Ok(Some(StoredAmountView {
                    key,
                    record: mandatory.latest_amount.as_ref(),
                }))
            }
            StoredAmountWindow::LastTerminal => {
                Ok(owner
                    .last_terminal
                    .as_deref()
                    .map(|capsule| StoredAmountView {
                        key: capsule.key,
                        record: capsule.mandatory.latest_amount.as_ref(),
                    }))
            }
        }
    }

    pub(crate) fn authoring_trace(&self) -> Result<TrialTraceView<'_>, GameError> {
        let mandatory = self.authoring_observation()?;
        let active = mandatory.active.map(|state| {
            let key = state.key.expect("validated active observation key");
            let battle = &self
                .state
                .active_battle
                .as_ref()
                .expect("validated active battle")
                .battle;
            (key, battle.authoring_trace.view())
        });
        Ok(TrialTraceView {
            active,
            last_terminal: mandatory
                .last_terminal
                .map(|capsule| (capsule.key, capsule.trace.view())),
        })
    }

    pub(crate) fn authoring_observation(&self) -> Result<TrialObservationView<'_>, GameError> {
        let owner = self.authoring_observation.as_ref().ok_or_else(invalid)?;
        let active = self
            .state
            .active_battle
            .as_ref()
            .map(|active| {
                let mandatory = active.battle.authoring_observation()?.ok_or_else(invalid)?;
                let witness = self
                    .authoring_lifecycle
                    .and_then(|state| state.current)
                    .ok_or_else(invalid)?;
                let key = ObservationKey {
                    entry_ordinal: witness.entry_ordinal,
                    battle_uuid: active.battle_uuid,
                };
                if witness.battle_uuid != active.battle_uuid || mandatory.key != Some(key) {
                    return Err(invalid());
                }
                Ok(mandatory)
            })
            .transpose()?;
        Ok(TrialObservationView {
            run_resources: self.run_resources(),
            active_typed_resources: self.battle_typed_resources(),
            active_runtime_facts: self
                .state
                .active_battle
                .as_ref()
                .map(|active| active.battle.runtime_fact_view()),
            active,
            last_terminal: owner.last_terminal.as_deref(),
        })
    }

    pub(super) fn capture_authoring_terminal(
        &mut self,
        battle_uuid: Uuid,
        outcome_id: &BattleOutcomeId,
        verdict: BattleVerdict,
        runtime_facts: Option<Arc<crate::game::battle::runtime_facts::RuntimeFactSnapshot>>,
    ) -> Result<(), GameError> {
        if self.authoring_observation.is_none() {
            return Ok(());
        }
        let runtime_facts = runtime_facts.ok_or_else(invalid)?;
        let key = self
            .authoring_observation()?
            .active
            .and_then(|state| state.key)
            .ok_or_else(invalid)?;
        let active = self.state.active_battle.as_ref().ok_or_else(invalid)?;
        if key.battle_uuid != battle_uuid || !active.execution.is_finished() {
            return Err(invalid());
        }
        let actual_time_ms = active.execution.last_event_time_ms();
        let observation = active.battle.authoring_observation()?.ok_or_else(invalid)?;
        let retained_extent = std::mem::size_of::<TerminalCapsule>()
            .checked_add(64)
            .and_then(|n| n.checked_add(runtime_facts.retained_extent))
            .and_then(|n| n.checked_add(observation.retained_extent()?))
            .and_then(|n| n.checked_add(active.battle.authoring_trace.retained_extent()?))
            .and_then(|n| n.checked_add(outcome_id.as_str().len()))
            .ok_or_else(invalid)?;
        let old_extent = self
            .authoring_observation
            .as_ref()
            .and_then(|owner| owner.last_terminal.as_ref())
            .map_or(0, |capsule| capsule.retained_extent);
        // Existing operation metadata budget covers old/new coexistence before
        // allocating or moving the observer. Clones share the immutable owner.
        admit_terminal_replacement(old_extent, retained_extent)?;
        let outcome_id = controlled_clone(outcome_id)?;
        control::charge(1, "authoring_terminal_owner")?;
        // The unpublished unique Arc exists before taking the real observer.
        let mut capsule = Arc::new(TerminalCapsule {
            retained_extent,
            runtime_facts,
            key,
            actual_time_ms,
            outcome_id,
            verdict,
            mandatory: MandatoryObservation::default(),
            trace: OptionalTrace::default(),
        });
        let battle = &mut self
            .state
            .active_battle
            .as_mut()
            .ok_or_else(invalid)?
            .battle;
        let mandatory = battle.take_authoring_observation()?;
        let trace = std::mem::take(&mut battle.authoring_trace);
        let unpublished = Arc::get_mut(&mut capsule).expect("unpublished terminal owner");
        if let Some(coverage)=&mandatory.coverage {self.authoring_observation.as_mut().and_then(|o|o.coverage_total.as_mut()).ok_or_else(invalid)?.merge(&coverage.report)?;}
        unpublished.mandatory = mandatory;
        unpublished.trace = trace;
        self.authoring_observation
            .as_mut()
            .expect("validated trial owner")
            .last_terminal = Some(capsule);
        Ok(())
    }
}

impl GameCore {
    pub(crate) fn battle_runtime_facts(
        &self,
    ) -> Option<crate::game::battle::runtime_facts::RuntimeFactView<'_>> {
        if let Some(active) = &self.state.active_battle {
            return Some(active.battle.runtime_fact_view());
        }
        if let Some(receipt) = &self.state.confirmed_combat_result {
            return receipt.runtime_facts.as_deref().map(|facts| facts.view());
        }
        match self.state.active_node_content.as_ref() {
            Some(crate::game::resources::ActiveNodeContent::CombatBattle(battle)) => {
                battle.runtime_facts.as_deref().map(|facts| facts.view())
            }
            _ => None,
        }
    }
}

fn admit_terminal_replacement(old_extent: usize, new_extent: usize) -> Result<(), GameError> {
    control::admit(
        control::Resource::MetadataBytes,
        old_extent,
        new_extent,
        "authoring_terminal_replacement_peak",
    )
}

#[cfg(test)]
pub(crate) fn assert_terminal_storage_contract(core: &GameCore) {
    use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
    let owner = core
        .authoring_observation
        .as_ref()
        .unwrap()
        .last_terminal
        .as_ref()
        .unwrap();
    let extent = owner.retained_extent;
    assert!(extent > std::mem::size_of::<TerminalCapsule>());
    let token = CancellationToken::default();
    let limits = ExecutionLimits::default();
    let (copy, usage) = control::run(&limits, &token, ExecutionPhase::Inspection, || {
        controlled_clone(owner)
    })
    .unwrap();
    assert!(Arc::ptr_eq(owner, &copy.unwrap()));
    assert!(
        usage.clone_slots_peak <= 2,
        "shared immutable owner must not traverse its payload"
    );
    let peak = extent.checked_mul(2).unwrap();
    let mut exact = limits.clone();
    exact.max_metadata_bytes = peak;
    let (result, usage) = control::run(&exact, &token, ExecutionPhase::TrialExecution, || {
        admit_terminal_replacement(extent, extent)
    })
    .unwrap();
    result.unwrap();
    assert_eq!(usage.metadata_bytes_peak, peak);
    exact.max_metadata_bytes -= 1;
    let error = control::run(&exact, &token, ExecutionPhase::TrialExecution, || {
        admit_terminal_replacement(extent, extent)
    })
    .unwrap_err();
    assert_eq!(error.code, "authoring_metadata_limit");
    assert!(
        control::run(&limits, &token, ExecutionPhase::TrialExecution, || {
            admit_terminal_replacement(usize::MAX, 1)
        })
        .is_err()
    );
}
