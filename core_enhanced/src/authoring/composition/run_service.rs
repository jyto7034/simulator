use super::{compile_yaml, CompositionRunIdentity, CompositionTrial, SourceTrace};
use crate::authoring::{
    CancellationToken, LabPreparation, ScenarioLimits, TrialLimits, TrialScenario,
};
use crate::game::data::content_pack::ContentSelection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

pub const MAX_RUN_REQUEST_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompositionRunRequest {
    pub source: String,
    pub epoch: u64,
    pub revision: u64,
    pub request_id: u64,
    pub artifact_digest: String,
    pub root: PathBuf,
    pub selection: ContentSelection,
    pub scenario: String,
    #[serde(default)]
    pub lab: Option<LabPreparation>,
}
#[derive(Debug, Serialize)]
pub struct CompositionRunFailure {
    pub stage: &'static str,
    pub diagnostic: Value,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedEffect {
    pub time_ms: u64,
    pub skill_id: Option<String>,
    pub definition_id: String,
    pub equipment: Option<crate::game::battle::runtime_facts::EquipmentSourceSnapshot>,
    pub source_file: String,
    pub field_path: String,
    pub target_unit: String,
    pub hp_delta: i64,
    pub hp_reason: Option<crate::game::battle::event_log::HpChangeReason>,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedResonance {
    pub time_ms: u64,
    pub battle_terminal: bool,
    pub skill_id: Option<String>,
    pub definition_id: String,
    pub equipment: Option<crate::game::battle::runtime_facts::EquipmentSourceSnapshot>,
    pub source_file: String,
    pub field_path: String,
    pub target_unit: String,
    pub before: u32,
    pub after: u32,
    pub max: u32,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedSustained {
    pub equipment: Option<crate::game::battle::runtime_facts::EquipmentSourceSnapshot>,
    pub area_source: Option<crate::authoring::view::AreaEffectSource>,
    pub time_ms: u64,
    pub battle_terminal: bool,
    pub definition_id: String,
    pub skill_id: Option<String>,
    pub cast_sequence: Option<u64>,
    pub source_file: String,
    pub field_path: String,
    pub target_unit: String,
    pub activation: u64,
    pub ended: Option<crate::game::ability::sustained::SustainedEndReason>,
    pub expires_at_ms: u64,
    pub target_state: Option<crate::game::ability::sustained::TargetStateProperties>,
    pub basic_speed_reduction_percent: Option<u32>,
        pub damage_taken_increase_percent: Option<u32>,
    pub before: Option<crate::game::stats::UnitStats>,
    pub after: Option<crate::game::stats::UnitStats>,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedHitCounter {
    pub equipment: Option<crate::game::battle::runtime_facts::EquipmentSourceSnapshot>,
    pub area_source: Option<crate::authoring::view::AreaEffectSource>,
    pub time_ms: u64,
    pub battle_terminal: bool,
    pub definition_id: String,
    pub skill_id: Option<String>,
    pub cast_sequence: Option<u64>,
    pub source_file: String,
    pub field_path: String,
    pub target_unit: String,
    #[serde(flatten)]
    pub outcome: crate::game::ability::sustained::HitCounterOutcome,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedArea {
    #[serde(flatten)]
    pub area: crate::authoring::view::AreaWitness,
    pub battle_terminal: bool,
    pub source_file: Option<String>,
    pub field_path: Option<String>,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedReadyWait {
    #[serde(flatten)]
    pub waiting: crate::authoring::view::ReadyWaitWitness,
    pub battle_terminal: bool,
    pub origins: Vec<Option<SourceTrace>>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedNearbyDefense {
    #[serde(flatten)]
    pub defense: crate::authoring::view::NearbyDefenseWitness,
    pub battle_terminal: bool,
    pub origins: Vec<Option<SourceTrace>>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedAreaDamage {
    #[serde(flatten)]
    pub damage: crate::authoring::view::AreaDamageWitness,
    pub battle_terminal: bool,
    pub origins: Vec<Option<SourceTrace>>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedNearbyAllyDamage {
    #[serde(flatten)]
    pub damage: crate::authoring::view::NearbyAllyDamageWitness,
    pub battle_terminal: bool,
    pub origins: Vec<Option<SourceTrace>>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedConditionalDamage {
    #[serde(flatten)]
    pub damage: crate::authoring::view::ConditionalDamageWitness,
    pub battle_terminal: bool,
    pub origins: Vec<Option<SourceTrace>>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedConditionalReduction {
    #[serde(flatten)]
    pub reduction: crate::authoring::view::IncomingConditionalWitness,
    pub battle_terminal: bool,
    pub origins: Vec<Option<SourceTrace>>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedActualHealReaction {
    #[serde(flatten)]
    pub reaction: crate::authoring::view::ActualHealReactionWitness,
    pub battle_terminal: bool,
    pub origins: Vec<Option<SourceTrace>>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedResonanceGain {
    #[serde(flatten)]
    pub gain: crate::authoring::view::ResonanceGainWitness,
    pub battle_terminal: bool,
    pub origins: Vec<Option<SourceTrace>>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedPrimaryDamage {
    #[serde(flatten)]
    pub damage: crate::authoring::view::PrimaryDamageWitness,
    pub battle_terminal: bool,
    pub origins: Vec<Option<SourceTrace>>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedDirectPenetration {
    #[serde(flatten)]
    pub penetration: crate::authoring::view::DirectPenetrationWitness,
    pub battle_terminal: bool,
    pub origins: Vec<Option<SourceTrace>>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedEquipmentSpeed {
    #[serde(flatten)]
    pub timing: crate::authoring::view::EquipmentSpeedWitness,
    pub battle_terminal: bool,
    pub origins: Vec<Option<SourceTrace>>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedHealBarrier {
    #[serde(flatten)]
    pub barrier: crate::authoring::view::HealBarrierWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedOverhealReserve {
    #[serde(flatten)]
    pub reserve: crate::authoring::view::OverhealReserveWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedMagazine {
    #[serde(flatten)]
    pub magazine: crate::authoring::view::MagazineWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Clone, Serialize)]
pub struct CompositionObservedSharedCohort {
    #[serde(flatten)]
    pub cohort: crate::authoring::view::SharedCohortWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedWeaponStage {
    #[serde(flatten)]
    pub shot: crate::authoring::view::WeaponStageWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedEmployeeTransformation {
    #[serde(flatten)]
    pub transformation: crate::authoring::view::EmployeeTransformationWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedLastWill {
    #[serde(flatten)]
    pub will: crate::authoring::view::LastWillWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedDeathBurst {
    #[serde(flatten)]
    pub burst: crate::authoring::view::DeathBurstWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedEquipmentForm {
    #[serde(flatten)]
    pub form: crate::authoring::view::FormWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedBounty {
    #[serde(flatten)]
    pub bounty: crate::authoring::view::BountyWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedDeferredDamage {
    #[serde(flatten)]
    pub deferred: crate::authoring::view::DeferredDamageWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedBasicHealEcho {
    #[serde(flatten)]
    pub echo: crate::authoring::view::BasicHealEchoWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedEquipmentSupport {
    #[serde(flatten)]
    pub support: crate::authoring::view::EquipmentSupportWitness,
    pub battle_terminal: bool,
    pub origins: Vec<Option<SourceTrace>>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedAdaptiveShield {
    #[serde(flatten)]
    pub shield: crate::authoring::view::AdaptiveShieldWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedIdleHpGuard {
    #[serde(flatten)]
    pub protection: crate::authoring::view::IdleHpGuardWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedChannelOpening {
    #[serde(flatten)]
    pub protection: crate::authoring::view::ChannelOpeningWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedIncomingFlat {
    #[serde(flatten)]
    pub protection: crate::authoring::view::IncomingFlatWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedWearerReduction {
    #[serde(flatten)]
    pub protection: crate::authoring::view::WearerReductionWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedAttackContinuity {
    #[serde(flatten)]
    pub protection: crate::authoring::view::AttackContinuityWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedReleaseGap {
    #[serde(flatten)]
    pub protection: crate::authoring::view::ReleaseGapWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedControlDuration {
    #[serde(flatten)]
    pub protection: crate::authoring::view::ControlDurationWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedShieldBeneficiaryDamage {
    #[serde(flatten)]
    pub protection: crate::authoring::view::ShieldBeneficiaryDamageWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedReceivedHeal {
    #[serde(flatten)]
    pub protection: crate::authoring::view::ReceivedHealWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedSkillBarrier {
    #[serde(flatten)]
    pub barrier: crate::authoring::view::SkillBarrierWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionObservedBasicEvasion {
    #[serde(flatten)]
    pub evasion: crate::authoring::view::BasicEvasionWitness,
    pub battle_terminal: bool,
    pub origin: Option<SourceTrace>,
}
#[derive(Debug, Serialize)]
pub struct CompositionRunResponse {
    pub epoch: u64,
    pub revision: u64,
    pub request_id: u64,
    pub source_digest: String,
    pub artifact_digest: String,
    pub status: &'static str,
    pub identity: Option<CompositionRunIdentity>,
    pub accepted_steps: usize,
    pub latest_effect: Option<CompositionObservedEffect>,
    pub latest_resonance: Option<CompositionObservedResonance>,
    pub latest_sustained: Option<CompositionObservedSustained>,
    pub latest_hit_counter: Option<CompositionObservedHitCounter>,
    pub latest_area: Option<CompositionObservedArea>,
    pub latest_shared_cohort: Option<CompositionObservedSharedCohort>,
    pub latest_employee_transformation: Option<CompositionObservedEmployeeTransformation>,
    pub latest_last_will: Option<CompositionObservedLastWill>,
    pub latest_weapon_stage: Option<CompositionObservedWeaponStage>,
    pub latest_equipment_death_burst: Option<CompositionObservedDeathBurst>,
    pub latest_equipment_form: Option<CompositionObservedEquipmentForm>,
    pub latest_bounty: Option<CompositionObservedBounty>,
    pub latest_magazine: Option<CompositionObservedMagazine>,
    pub latest_deferred_damage: Option<CompositionObservedDeferredDamage>,
    pub latest_overheal_reserve: Option<CompositionObservedOverhealReserve>,
    pub latest_basic_heal_echo: Option<CompositionObservedBasicHealEcho>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_heal_barrier: Option<CompositionObservedHealBarrier>,
    pub latest_equipment_support: Option<CompositionObservedEquipmentSupport>,
    pub latest_ready_wait: Option<CompositionObservedReadyWait>,
    pub latest_nearby_defense: Option<CompositionObservedNearbyDefense>,
    pub latest_primary_damage: Option<CompositionObservedPrimaryDamage>,
    pub latest_area_damage: Option<CompositionObservedAreaDamage>,
    pub latest_nearby_ally_damage: Option<CompositionObservedNearbyAllyDamage>,
    pub latest_conditional_damage: Option<CompositionObservedConditionalDamage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_direct_penetration: Option<CompositionObservedDirectPenetration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_conditional_reduction: Option<CompositionObservedConditionalReduction>,
    pub latest_actual_heal_reaction: Option<CompositionObservedActualHealReaction>,
    pub latest_resonance_gain: Option<CompositionObservedResonanceGain>,
    pub latest_equipment_speed: Option<CompositionObservedEquipmentSpeed>,
    pub latest_adaptive_shield: Option<CompositionObservedAdaptiveShield>,
    pub latest_channel_opening: Option<CompositionObservedChannelOpening>,
    pub latest_idle_hp_guard: Option<CompositionObservedIdleHpGuard>,
    pub latest_incoming_flat: Option<CompositionObservedIncomingFlat>,
    pub latest_attack_continuity: Option<CompositionObservedAttackContinuity>,
    pub latest_release_gap: Option<CompositionObservedReleaseGap>,
    pub latest_wearer_reduction: Option<CompositionObservedWearerReduction>,
    pub latest_control_duration: Option<CompositionObservedControlDuration>,
    pub latest_shield_beneficiary_damage: Option<CompositionObservedShieldBeneficiaryDamage>,
    pub latest_received_heal: Option<CompositionObservedReceivedHeal>,
    pub latest_skill_barrier: Option<CompositionObservedSkillBarrier>,
    pub latest_basic_evasion: Option<CompositionObservedBasicEvasion>,
    pub failure: Option<CompositionRunFailure>,
}
impl CompositionRunResponse {
    fn reject(mut self, stage: &'static str, error: impl Serialize) -> Result<Self, String> {
        self.failure = Some(CompositionRunFailure {
            stage,
            diagnostic: serde_json::to_value(error).map_err(|e| e.to_string())?,
        });
        Ok(self)
    }
}

// Keep response projection shared with the verifier's deliberately inconsistent
// observation tests; resolution still belongs to the current trial/catalog.
pub(crate) fn hit_counter_response(
    application: crate::authoring::ApplicationView<'_>,
    battle_terminal: bool,
    area_origin: Option<(String, String, SourceTrace)>,
    resolve_authored: impl FnOnce(crate::authoring::AuthoredEffectView<'_>) -> Option<SourceTrace>,
) -> Option<CompositionObservedHitCounter> {
    let crate::authoring::ApplicationChange::HitCounter { outcome } = application.change() else {
        return None;
    };
    let authored = application.authored_source();
    let area = application.area_source();
    Some(CompositionObservedHitCounter {
        equipment: application.equipment_source(), area_source: area.cloned(),
        time_ms: application.time_ms(), battle_terminal,
        definition_id: authored.map(|s| s.definition_id().into()).or_else(|| area.map(|a| a.definition.clone())).unwrap_or_default(),
        skill_id: application.source().map(|s| s.skill_id().into()).or_else(|| area.map(|a| a.skill_id.as_str().into())),
        cast_sequence: application.source().map(|s| s.cast_seq()).or_else(|| area.map(|a| a.cast_sequence)),
        source_file: authored.map(|s| s.source_file().into()).or_else(|| area_origin.as_ref().map(|s| s.0.clone())).unwrap_or_default(),
        field_path: authored.map(|s| s.field_path().into()).or_else(|| area_origin.as_ref().map(|s| s.1.clone())).unwrap_or_default(),
        target_unit: application.target_unit().to_string(), outcome,
        origin: match authored {
            Some(source) => resolve_authored(source),
            None => area_origin.map(|s| s.2),
        },
    })
}

/// Execute only the submitted finite scenario over an isolated real World.
/// A result identifies the pinned source, generated definition and actual run;
/// no existing trial, content file, player profile or checkpoint is replaced.
pub fn run_composition(
    request: CompositionRunRequest,
    cancel: &CancellationToken,
) -> Result<CompositionRunResponse, String> {
    if [request.epoch, request.revision, request.request_id]
        .iter()
        .any(|n| *n > (1u64 << 53) - 1)
        || request.root.as_os_str().len() > 4096
        || request.scenario.len() > ScenarioLimits::default().max_input_bytes
    {
        return Err("invalid run request identity, root path or scenario size".into());
    }
    let response = CompositionRunResponse {
        epoch: request.epoch,
        revision: request.revision,
        request_id: request.request_id,
        source_digest: format!("{:x}", Sha256::digest(request.source.as_bytes())),
        artifact_digest: request.artifact_digest.clone(),
        status: "rejected",
        identity: None,
        accepted_steps: 0,
        latest_effect: None,
        latest_resonance: None,
        latest_sustained: None,
        latest_hit_counter: None,
        latest_area: None,
        latest_ready_wait: None,
        latest_nearby_defense: None,
        latest_primary_damage: None,
        latest_area_damage: None,
        latest_nearby_ally_damage: None,
        latest_conditional_damage: None,
        latest_direct_penetration: None,
        latest_conditional_reduction: None,
        latest_resonance_gain: None,
        latest_actual_heal_reaction: None,
        latest_equipment_speed: None,
        latest_equipment_support: None,
        latest_heal_barrier: None,
        latest_basic_heal_echo: None,
        latest_deferred_damage: None,
        latest_shared_cohort: None,
        latest_bounty: None,
        latest_employee_transformation: None,
        latest_last_will: None,
        latest_weapon_stage: None,
        latest_equipment_death_burst: None,
        latest_equipment_form: None,
        latest_magazine: None,
        latest_overheal_reserve: None,
        latest_adaptive_shield: None,
        latest_channel_opening: None,
        latest_idle_hp_guard: None,
        latest_incoming_flat: None,
        latest_attack_continuity: None,
        latest_release_gap: None,
        latest_wearer_reduction: None,
        latest_control_duration: None,
        latest_shield_beneficiary_damage: None,
        latest_received_heal: None,
        latest_skill_barrier: None,
        latest_basic_evasion: None,
        failure: None,
    };
    let artifact = match compile_yaml(&request.source) {
        Ok(artifact) => artifact,
        Err(error) => return response.reject("compile", error),
    };
    let artifact_text = artifact.to_ron().map_err(|e| e.to_string())?;
    let artifact_digest = format!("{:x}", Sha256::digest(artifact_text.as_bytes()));
    if artifact_digest != request.artifact_digest {
        return response.reject(
            "definition",
            "compiled artifact does not match the requested definition",
        );
    }
    let scenario = match TrialScenario::from_json_slice(
        request.scenario.as_bytes(),
        &ScenarioLimits::default(),
    ) {
        Ok(scenario) => scenario,
        Err(error) => return response.reject("scenario", error),
    };
    let mut trial = match CompositionTrial::prepare_artifact(
        &request.root,
        &request.selection,
        artifact,
        scenario,
        request.lab,
        TrialLimits::default(),
        cancel,
    ) {
        Ok(trial) => trial,
        Err(error) => return response.reject("prepare", error),
    };
    let result = trial.run_to_end(cancel);
    let mut response = CompositionRunResponse {
        status: if result.is_ok() {
            "completed"
        } else {
            "failed"
        },
        identity: Some(trial.identity().clone()),
        accepted_steps: trial.session().next_step_index(),
        ..response
    };
    match trial.session().observation() {
        Ok(observation) => {
            let observed = observation
                .active()
                .or_else(|| observation.last_terminal().map(|t| t.mandatory()));
            if let Some(shield) = observed.and_then(|o| o.latest_adaptive_shield()) {
                response.latest_adaptive_shield = Some(CompositionObservedAdaptiveShield {
                    shield: shield.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_adaptive_shield_origin(),
                });
            }
            if let Some(protection) = observed.and_then(|o| o.latest_channel_opening()) {
                response.latest_channel_opening = Some(CompositionObservedChannelOpening {
                    protection: protection.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_channel_opening_origin(),
                });
            }
            if let Some(protection) = observed.and_then(|o| o.latest_idle_hp_guard()) {
                response.latest_idle_hp_guard = Some(CompositionObservedIdleHpGuard {
                    protection: protection.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_idle_hp_guard_origin(),
                });
            }
            if let Some(protection) = observed.and_then(|o| o.latest_incoming_flat()) {
                response.latest_incoming_flat = Some(CompositionObservedIncomingFlat {
                    protection: protection.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_incoming_flat_origin(),
                });
            }
            if let Some(protection) = observed.and_then(|o| o.latest_wearer_reduction()) {
                response.latest_wearer_reduction = Some(CompositionObservedWearerReduction {
                    protection: protection.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_wearer_reduction_origin(),
                });
            }
            if let Some(protection) = observed.and_then(|o| o.latest_attack_continuity()) {
                response.latest_attack_continuity = Some(CompositionObservedAttackContinuity {
                    protection: protection.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_attack_continuity_origin(),
                });
            }
            if let Some(protection) = observed.and_then(|o| o.latest_release_gap()) {
                response.latest_release_gap = Some(CompositionObservedReleaseGap {
                    protection: protection.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_release_gap_origin(),
                });
            }
            if let Some(protection) = observed.and_then(|o| o.latest_control_duration()) {
                response.latest_control_duration = Some(CompositionObservedControlDuration {
                    protection: protection.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_control_duration_origin(),
                });
            }
            if let Some(protection) = observed.and_then(|o| o.latest_shield_beneficiary_damage()) {
                response.latest_shield_beneficiary_damage = Some(CompositionObservedShieldBeneficiaryDamage {
                    protection: protection.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_shield_beneficiary_damage_origin(),
                });
            }
            if let Some(protection) = observed.and_then(|o| o.latest_received_heal()) {
                response.latest_received_heal = Some(CompositionObservedReceivedHeal {
                    protection: protection.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_received_heal_origin(),
                });
            }
            if let Some(barrier) = observed.and_then(|o| o.latest_skill_barrier()) {
                response.latest_skill_barrier = Some(CompositionObservedSkillBarrier {
                    barrier: barrier.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_skill_barrier_origin(),
                });
            }
            if let Some(reserve) = observed.and_then(|o| o.latest_overheal_reserve()) {
                response.latest_overheal_reserve = Some(CompositionObservedOverhealReserve {
                    reserve: reserve.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_overheal_reserve_origin(),
                });
            }
            if let Some(cohort) = observed.and_then(|o| o.latest_shared_cohort()) {
                response.latest_shared_cohort = Some(CompositionObservedSharedCohort {
                    cohort: cohort.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_shared_cohort_origin(),
                });
            }
            if let Some(shot) = observed.and_then(|o| o.latest_weapon_stage()) {
                response.latest_weapon_stage = Some(CompositionObservedWeaponStage {
                    shot: shot.clone(), origin: trial.latest_weapon_stage_origin(),
                    battle_terminal: observation.active().is_none(),
                });
            }
            if let Some(transformation) = observed.and_then(|o| o.latest_employee_transformation()) {
                response.latest_employee_transformation = Some(CompositionObservedEmployeeTransformation {
                    transformation: transformation.clone(), origin: trial.latest_employee_transformation_origin(),
                    battle_terminal: observation.active().is_none(),
                });
            }
            if let Some(will) = observed.and_then(|o| o.latest_last_will()) {
                response.latest_last_will = Some(CompositionObservedLastWill {
                    will: will.clone(),
                    origin: trial.latest_last_will_origin(),
                    battle_terminal: observation.active().is_none(),
                });
            }
            if let Some(burst) = observed.and_then(|o| o.latest_equipment_death_burst()) {
                response.latest_equipment_death_burst = Some(CompositionObservedDeathBurst {
                    burst: burst.clone(),
                    origin: trial.latest_equipment_death_burst_origin(),
                    battle_terminal: observation.active().is_none(),
                });
            }
            if let Some(equipment_form) = observed.and_then(|o| o.latest_equipment_form()) {
                response.latest_equipment_form = Some(CompositionObservedEquipmentForm {
                    form: equipment_form.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_equipment_form_origin(),
                });
            }
            if let Some(bounty) = observed.and_then(|o| o.latest_bounty()) {
                response.latest_bounty = Some(CompositionObservedBounty {
                    bounty: bounty.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_bounty_origin(),
                });
            }
            if let Some(magazine) = observed.and_then(|o| o.latest_magazine()) {
                response.latest_magazine = Some(CompositionObservedMagazine {
                    magazine: magazine.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_magazine_origin(),
                });
            }
            if let Some(deferred) = observed.and_then(|o| o.latest_deferred_damage()) {
                response.latest_deferred_damage = Some(CompositionObservedDeferredDamage {
                    deferred: deferred.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_deferred_damage_origin(),
                });
            }
            if let Some(echo) = observed.and_then(|o| o.latest_basic_heal_echo()) {
                response.latest_basic_heal_echo = Some(CompositionObservedBasicHealEcho {
                    echo: echo.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_basic_heal_echo_origin(),
                });
            }
            if let Some(evasion) = observed.and_then(|o| o.latest_basic_evasion()) {
                response.latest_basic_evasion = Some(CompositionObservedBasicEvasion {
                    evasion: evasion.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_basic_evasion_origin(),
                });
            }
            if let Some(waiting) = observed.and_then(|o| o.latest_ready_wait()) {
                response.latest_ready_wait = Some(CompositionObservedReadyWait {
                    waiting: waiting.clone(), battle_terminal: observation.active().is_none(),
                    origins: trial.latest_ready_wait_origins().unwrap_or_else(|| vec![None; waiting.contributors.len()]),
                });
            }
            if let Some(defense) = observed.and_then(|o| o.latest_nearby_defense()) {
                response.latest_nearby_defense = Some(CompositionObservedNearbyDefense {
                    defense: defense.clone(), battle_terminal: observation.active().is_none(),
                    origins: trial.latest_nearby_defense_origins().unwrap_or_else(|| vec![None; defense.contributors.len()]),
                });
            }
            if let Some(damage) = observed.and_then(|o| o.latest_primary_damage()) {
                response.latest_primary_damage = Some(CompositionObservedPrimaryDamage {
                    damage: damage.clone(), battle_terminal: observation.active().is_none(),
                    origins: trial.latest_primary_damage_origins()
                        .unwrap_or_else(|| vec![None; damage.contributors.len()]),
                });
            }
            if let Some(penetration) = observed.and_then(|o| o.latest_direct_penetration()) {
                response.latest_direct_penetration = Some(CompositionObservedDirectPenetration {
                    penetration: penetration.clone(), battle_terminal: observation.active().is_none(),
                    origins: trial.latest_direct_penetration_origins()
                        .unwrap_or_else(|| vec![None; penetration.contributors.len()]),
                });
            }
            if let Some(damage) = observed.and_then(|o| o.latest_area_damage()) {
                response.latest_area_damage = Some(CompositionObservedAreaDamage {
                    damage: damage.clone(), battle_terminal: observation.active().is_none(),
                    origins: trial.latest_area_damage_origins()
                        .unwrap_or_else(|| vec![None; damage.contributors.len()]),
                });
            }
            if let Some(damage) = observed.and_then(|o| o.latest_nearby_ally_damage()) {
                response.latest_nearby_ally_damage = Some(CompositionObservedNearbyAllyDamage {
                    damage: damage.clone(), battle_terminal: observation.active().is_none(),
                    origins: trial.latest_nearby_ally_damage_origins()
                        .unwrap_or_else(|| vec![None; damage.contributors.len()]),
                });
            }
            if let Some(damage) = observed.and_then(|o| o.latest_conditional_damage()) {
                response.latest_conditional_damage = Some(CompositionObservedConditionalDamage {
                    damage: damage.clone(), battle_terminal: observation.active().is_none(),
                    origins: trial.latest_conditional_damage_origins()
                        .unwrap_or_else(|| vec![None; damage.contributors.len()]),
                });
            }
            if let Some(reduction) = observed.and_then(|o| o.latest_conditional_reduction()) {
                response.latest_conditional_reduction = Some(CompositionObservedConditionalReduction {
                    reduction: reduction.clone(), battle_terminal: observation.active().is_none(),
                    origins: trial.latest_conditional_reduction_origins()
                        .unwrap_or_else(|| vec![None; reduction.evaluation.contributors.len()]),
                });
            }
            if let Some(gain) = observed.and_then(|o| o.latest_actual_heal_reaction()) {
                response.latest_actual_heal_reaction = Some(CompositionObservedActualHealReaction {
                    reaction: gain.clone(), battle_terminal: observation.active().is_none(),
                    origins: trial.latest_actual_heal_reaction_origins()
                        .unwrap_or_else(|| vec![None; gain.contributors.len()]),
                });
            }
            if let Some(gain) = observed.and_then(|o| o.latest_resonance_gain()) {
                response.latest_resonance_gain = Some(CompositionObservedResonanceGain {
                    gain: gain.clone(), battle_terminal: observation.active().is_none(),
                    origins: trial.latest_resonance_gain_origins()
                        .unwrap_or_else(|| vec![None; gain.contributors.len()]),
                });
            }
            if let Some(timing) = observed.and_then(|o| o.latest_equipment_speed()) {
                response.latest_equipment_speed = Some(CompositionObservedEquipmentSpeed {
                    timing: timing.clone(), battle_terminal: observation.active().is_none(),
                    origins: trial.latest_equipment_speed_origins()
                        .unwrap_or_else(|| vec![None; timing.contributors.len()]),
                });
            }
            if let Some(barrier) = observed.and_then(|o| o.latest_heal_barrier()) {
                response.latest_heal_barrier = Some(CompositionObservedHealBarrier {
                    barrier: barrier.clone(), battle_terminal: observation.active().is_none(),
                    origin: trial.latest_heal_barrier_origin(),
                });
            }
            if let Some(support) = observed.and_then(|o| o.latest_equipment_support()) {
                response.latest_equipment_support = Some(CompositionObservedEquipmentSupport {
                    support: support.clone(), battle_terminal: observation.active().is_none(),
                    origins: trial.latest_equipment_support_origins()
                        .unwrap_or_else(|| vec![None; support.contributors.len()]),
                });
            }
            if let Some(area) = observed.and_then(|o| o.latest_area()) {
                let source = trial.latest_area_origin();
                response.latest_area = Some(CompositionObservedArea {
                    area: area.clone(),
                    battle_terminal: observation.active().is_none(),
                    source_file: source.as_ref().map(|s| s.0.clone()),
                    field_path: source.as_ref().map(|s| s.1.clone()),
                    origin: source.map(|s| s.2),
                });
            }
            if let Some(application) = observed.and_then(|o| o.latest_sourced_hit_counter()) {
                response.latest_hit_counter = hit_counter_response(
                    application,
                    observation.active().is_none(),
                    trial.latest_hit_counter_area_origin(),
                    |source| trial.origin_for_observed(source).cloned(),
                );
            }
            if let Some(application) = observed.and_then(|o| o.latest_sourced_sustained()) {
                if let (
                    Some(authored),
                    crate::authoring::ApplicationChange::Sustained {
                        activation,
                        ended,
                        expires_at_ms,
                        target_state,
                        basic_speed_reduction_percent,
                        damage_taken_increase_percent,
                        before,
                        after,
                    },
                ) = (application.authored_source(), application.change())
                {
                    response.latest_sustained = Some(CompositionObservedSustained {
                        equipment: application.equipment_source(),
                        area_source: None,
                        time_ms: application.time_ms(),
                        battle_terminal: observation.active().is_none(),
                        definition_id: authored.definition_id().into(),
                        skill_id: application.source().map(|s| s.skill_id().into()),
                        cast_sequence: application.source().map(|s| s.cast_seq()),
                        source_file: authored.source_file().into(),
                        field_path: authored.field_path().into(),
                        target_unit: application.target_unit().to_string(),
                        activation,
                        ended,
                        expires_at_ms,
                        target_state,
                        basic_speed_reduction_percent,
                        damage_taken_increase_percent,
                        before,
                        after,
                        origin: trial.origin_for_observed(authored).cloned(),
                    });
                } else if let (
                    Some(producer),
                    crate::authoring::ApplicationChange::Sustained {
                        activation,
                        ended,
                        expires_at_ms,
                        target_state,
                        basic_speed_reduction_percent,
                        damage_taken_increase_percent,
                        before,
                        after,
                    },
                ) = (application.area_source(), application.change())
                {
                    let source = trial.latest_sustained_area_origin();
                    response.latest_sustained = Some(CompositionObservedSustained {
                        equipment: application.equipment_source(),
                        area_source: Some(producer.clone()),
                        time_ms: application.time_ms(),
                        battle_terminal: observation.active().is_none(),
                        definition_id: producer.definition.clone(),
                        skill_id: Some(producer.skill_id.as_str().into()),
                        cast_sequence: Some(producer.cast_sequence),
                        source_file: source.as_ref().map_or_else(String::new, |s| s.0.clone()),
                        field_path: source.as_ref().map_or_else(String::new, |s| s.1.clone()),
                        target_unit: application.target_unit().to_string(),
                        activation,
                        ended,
                        expires_at_ms,
                        target_state,
                        basic_speed_reduction_percent,
                        damage_taken_increase_percent,
                        before,
                        after,
                        origin: source.map(|s| s.2),
                    });
                }
            }
            if let Some(application) = observed.and_then(|o| o.latest_sourced_resonance()) {
                if let (Some(authored), crate::authoring::ApplicationChange::Resonance { before, after, max })
                    = (application.authored_source(), application.change()) {
                    response.latest_resonance = Some(CompositionObservedResonance {
                        time_ms: application.time_ms(),
                        battle_terminal: observation.active().is_none(),
                        skill_id: application.source().map(|s| s.skill_id().into()),
                        definition_id: authored.definition_id().into(),
                        equipment: application.equipment_source(),
                        source_file: authored.source_file().into(),
                        field_path: authored.field_path().into(),
                        target_unit: application.target_unit().to_string(),
                        before, after, max,
                        origin: trial.origin_for_observed(authored).cloned(),
                    });
                }
            }
            if let Some(application) = observed.and_then(|o| o.latest_nonzero_sourced_hp()) {
                if let Some(authored) = application.authored_source() {
                    response.latest_effect = Some(CompositionObservedEffect {
                        time_ms: application.time_ms(),
                        skill_id: application.source().map(|s| s.skill_id().into()),
                        definition_id: authored.definition_id().into(),
                        equipment: application.equipment_source(),
                        source_file: authored.source_file().into(),
                        field_path: authored.field_path().into(),
                        target_unit: application.target_unit().to_string(),
                        hp_delta: application.change().hp_delta(),
                        hp_reason: match application.change() {
                            crate::authoring::ApplicationChange::Hp { reason, .. } => Some(reason), _ => None,
                        },
                        origin: trial.origin_for_observed(authored).cloned(),
                    });
                }
            }
        }
        Err(error) if result.is_ok() => {
            response.status = "failed";
            return response.reject("observation", error);
        }
        Err(_) => {} // Preserve the original execution diagnostic below.
    }
    match result {
        Ok(()) => Ok(response),
        Err(error) => response.reject("execution", error),
    }
}
