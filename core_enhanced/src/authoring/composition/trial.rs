//! A running composition owns one validated content generation and source map.
//! Editing or compiling another source cannot replace that generation in place.
use super::{compile_yaml, Artifact, Diagnostic, SourceTrace};
use crate::authoring::{
    self, AuthoringContent, AuthoringError, CancellationToken, ExecutionPhase, LabPreparation,
    StepView, TrialFailure, TrialLimits, TrialScenario, TrialSession,
};
use crate::game::{
    ability::authoring::EffectSource,
    data::content_pack::{self, ContentIdentity, ContentSelection, InputLimits},
};
use crate::authoring::provenance::{EquipmentOriginClaim, EquipmentOriginRole};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
#[serde(tag = "phase", content = "diagnostic", rename_all = "snake_case")]
pub enum CompositionTrialFailure {
    Compile(Diagnostic),
    Content(AuthoringError),
    Trial(TrialFailure),
}
impl std::fmt::Display for CompositionTrialFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compile(e) => e.fmt(f),
            Self::Content(e) => e.fmt(f),
            Self::Trial(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for CompositionTrialFailure {}

#[derive(Debug, Clone, Serialize)]
pub struct CompositionRunIdentity {
    pub run_id: uuid::Uuid,
    pub source_digest: String,
    pub artifact_digest: String,
    pub artifact_path: String,
    pub content: ContentIdentity,
    pub scenario_digest: String,
}

pub struct CompositionTrial {
    identity: CompositionRunIdentity,
    artifact: Artifact,
    session: TrialSession,
    base_validation_usage: authoring::ExecutionUsage,
}
impl CompositionTrial {
    pub fn prepare(
        root: &Path,
        selection: &ContentSelection,
        source: &str,
        scenario: TrialScenario,
        lab: Option<LabPreparation>,
        limits: TrialLimits,
        cancel: &CancellationToken,
    ) -> Result<Self, CompositionTrialFailure> {
        let artifact = compile_yaml(source).map_err(CompositionTrialFailure::Compile)?;
        Self::prepare_artifact(root, selection, artifact, scenario, lab, limits, cancel)
    }
    pub(super) fn prepare_artifact(
        root: &Path,
        selection: &ContentSelection,
        artifact: Artifact,
        scenario: TrialScenario,
        lab: Option<LabPreparation>,
        limits: TrialLimits,
        cancel: &CancellationToken,
    ) -> Result<Self, CompositionTrialFailure> {
        Self::prepare_artifact_with_runtime_safety(
            root, selection, artifact, scenario, lab, limits, cancel,
            crate::runtime_safety::RuntimeSafetySettings::embedded_shared(),
        )
    }
    /// Select trusted host settings for catalog publication and the resulting
    /// runtime. Source compilation retains its own bounded composition contract;
    /// per-step trial limits remain explicit and do not grow automatically.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_with_runtime_safety(
        root: &Path,
        selection: &ContentSelection,
        source: &str,
        scenario: TrialScenario,
        lab: Option<LabPreparation>,
        limits: TrialLimits,
        cancel: &CancellationToken,
        settings: std::sync::Arc<crate::runtime_safety::RuntimeSafetySettings>,
    ) -> Result<Self, CompositionTrialFailure> {
        let artifact = compile_yaml(source).map_err(CompositionTrialFailure::Compile)?;
        Self::prepare_artifact_with_runtime_safety(
            root, selection, artifact, scenario, lab, limits, cancel, settings,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn prepare_artifact_with_runtime_safety(
        root: &Path,
        selection: &ContentSelection,
        artifact: Artifact,
        scenario: TrialScenario,
        lab: Option<LabPreparation>,
        limits: TrialLimits,
        cancel: &CancellationToken,
        settings: std::sync::Arc<crate::runtime_safety::RuntimeSafetySettings>,
    ) -> Result<Self, CompositionTrialFailure> {
        let (base, base_validation_usage) = authoring::control::run(
            &settings.authoring_execution(),
            cancel,
            ExecutionPhase::Acquisition,
            || {
                content_pack::capture_composition_base(
                    root,
                    selection,
                    &InputLimits::default(),
                    settings.clone(),
                )
            },
        )
        .map_err(|diagnostic| {
            CompositionTrialFailure::Content(AuthoringError::Execution { diagnostic })
        })?;
        let base = base.map_err(|diagnostic| {
            CompositionTrialFailure::Content(AuthoringError::Content { diagnostic })
        })?;
        let (publication, usage) = authoring::control::run(
            &settings.authoring_execution(),
            cancel,
            ExecutionPhase::Acquisition,
            || {
                content_pack::load_composition_preview(
                    base,
                    selection,
                    &artifact,
                    &InputLimits::default(),
                    settings.clone(),
                )
            },
        )
        .map_err(|diagnostic| {
            CompositionTrialFailure::Content(AuthoringError::Execution { diagnostic })
        })?;
        let content_pack::CompositionPublication {
            token,
            artifact_path,
            artifact_digest,
        } = publication.map_err(|diagnostic| {
            CompositionTrialFailure::Content(AuthoringError::Content { diagnostic })
        })?;
        let content = AuthoringContent::from_candidate(token, usage);
        let session = match lab {
            Some(recipe) => {
                TrialSession::prepare_with_lab(content, scenario, recipe, limits, cancel)
            }
            None => TrialSession::prepare(content, scenario, limits, cancel),
        }
        .map_err(CompositionTrialFailure::Trial)?;
        Ok(Self {
            identity: CompositionRunIdentity {
                run_id: uuid::Uuid::new_v4(),
                source_digest: artifact.source_digest().into(),
                artifact_digest,
                artifact_path,
                content: session.content_identity().clone(),
                scenario_digest: session.input_sha256().into(),
            },
            artifact,
            session,
            base_validation_usage,
        })
    }
    pub fn identity(&self) -> &CompositionRunIdentity {
        &self.identity
    }
    pub fn base_validation_usage(&self) -> &authoring::ExecutionUsage {
        &self.base_validation_usage
    }
    /// Read-only access preserves the source/generation association. In
    /// particular, callers cannot use TrialSession::replace through this view.
    pub fn session(&self) -> &TrialSession {
        &self.session
    }
    pub fn advance(&mut self, cancel: &CancellationToken) -> Result<StepView<'_>, TrialFailure> {
        self.session.advance(cancel)
    }
    pub fn run_to_end(&mut self, cancel: &CancellationToken) -> Result<(), TrialFailure> {
        self.session.run_to_end(cancel)
    }
    pub fn origin_for(&self, source: &EffectSource) -> Option<&SourceTrace> {
        if source.source_file != self.identity.artifact_path {
            return None;
        }
        self.session.composition_origin(&self.artifact, source)
    }
    pub fn origin_for_observed(
        &self,
        source: crate::authoring::AuthoredEffectView<'_>,
    ) -> Option<&SourceTrace> {
        self.origin_for(source.source())
    }
    /// Resolve the actual finite effect's producer even after its area has ended.
    pub fn latest_sustained_area_origin(&self) -> Option<(String, String, SourceTrace)> {
        let observation = self.session.observation().ok()?;
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let producer = observed.latest_sourced_sustained()?.area_source()?;
        self.observed_area_origin(producer)
    }
    pub fn latest_hit_counter_area_origin(&self) -> Option<(String, String, SourceTrace)> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        self.observed_area_origin(observed.latest_sourced_hit_counter()?.area_source()?)
    }
    // Internal claim validation is available to verifier tests. Public callers
    // resolve only this session's actual observations through latest_* methods.
    pub(crate) fn observed_area_origin(&self, producer: &crate::authoring::view::AreaEffectSource) -> Option<(String, String, SourceTrace)> {
        let result = self.session.composition_area_effect_origin(&self.artifact, producer)?;
        (result.0 == self.identity.artifact_path).then_some(result)
    }
    /// Resolve only this session's actual latest witness. Accepting an arbitrary
    /// witness would confuse equal semantic content with equal authoring source.
    pub fn latest_area_origin(&self) -> Option<(String, String, SourceTrace)> {
        let observation = self.session.observation().ok()?;
        let observed = observation
            .active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let area = observed.latest_area()?;
        if area.generation.as_deref() != Some(&self.identity.content) {
            return None;
        }
        let result = self.session.composition_delivery_origin(
            &self.artifact, area.skill_id.as_str(), &area.step_id)?;
        (result.0 == self.identity.artifact_path).then_some(result)
    }
    pub fn latest_adaptive_shield_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_adaptive_shield()?;
        if witness.generation.as_deref() != Some(&self.identity.content)
            || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::AdaptiveShield,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub(crate) fn observed_equipment_origin(
        &self, claim: &EquipmentOriginClaim<'_>,
    ) -> Option<SourceTrace> {
        if claim.generation != Some(&self.identity.content)
            || claim.source_file != self.identity.artifact_path { return None; }
        self.session.composition_equipment_origin(&self.artifact, claim).cloned()
    }
    pub fn latest_channel_opening_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_channel_opening()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::ChannelOpening,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_idle_hp_guard_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_idle_hp_guard()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::IdleHpGuard,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_incoming_flat_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_incoming_flat()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::IncomingFlat,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_wearer_reduction_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_wearer_reduction()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::WearerReduction,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_attack_continuity_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_attack_continuity()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::AttackContinuity,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_release_gap_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_release_gap()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::ReleaseGap,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_control_duration_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_control_duration()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::ControlDuration(witness.kind),
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_shield_beneficiary_damage_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_shield_beneficiary_damage()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::ShieldBeneficiaryDamage,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_received_heal_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_received_heal()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::ReceivedHeal,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_skill_barrier_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_skill_barrier()?;
        if witness.generation.as_deref() != Some(&self.identity.content)
            || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::SkillBarrier,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_basic_evasion_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_basic_evasion()?;
        if witness.generation.as_deref() != Some(&self.identity.content)
            || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::BasicEvasion,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_ready_wait_origins(&self) -> Option<Vec<Option<SourceTrace>>> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_ready_wait()?;
        if witness.generation.as_deref() != Some(&self.identity.content) { return None; }
        Some(witness.contributors.iter().map(|contributor| {
            self.observed_equipment_origin(&EquipmentOriginClaim {
                generation: witness.generation.as_deref(),
                role: EquipmentOriginRole::ReadyWait,
                equipment_uuid: Some(contributor.equipment.definition_uuid),
                definition_id: Some(&contributor.definition_id),
                source_file: &contributor.source_file,
                field_path: &contributor.field_path,
            })
        }).collect())
    }
    pub fn latest_nearby_defense_origins(&self) -> Option<Vec<Option<SourceTrace>>> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_nearby_defense()?;
        if witness.generation.as_deref() != Some(&self.identity.content) { return None; }
        Some(witness.contributors.iter().map(|contributor| {
            self.observed_equipment_origin(&EquipmentOriginClaim {
                generation: witness.generation.as_deref(),
                role: EquipmentOriginRole::NearbyDefense,
                equipment_uuid: Some(contributor.equipment.definition_uuid),
                definition_id: Some(&contributor.definition_id),
                source_file: &contributor.source_file,
                field_path: &contributor.field_path,
            })
        }).collect())
    }
    pub fn latest_primary_damage_origins(&self) -> Option<Vec<Option<SourceTrace>>> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_primary_damage()?;
        if witness.generation.as_deref() != Some(&self.identity.content) { return None; }
        Some(witness.contributors.iter().map(|contributor| {
            self.observed_equipment_origin(&EquipmentOriginClaim {
                generation: witness.generation.as_deref(),
                role: EquipmentOriginRole::PrimaryDamage,
                equipment_uuid: Some(contributor.equipment.definition_uuid),
                definition_id: Some(&contributor.definition_id),
                source_file: &contributor.source_file,
                field_path: &contributor.field_path,
            })
        }).collect())
    }
    pub fn latest_area_damage_origins(&self) -> Option<Vec<Option<SourceTrace>>> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_area_damage()?;
        if witness.generation.as_deref() != Some(&self.identity.content) { return None; }
        Some(witness.contributors.iter().map(|contributor| {
            self.observed_equipment_origin(&EquipmentOriginClaim {
                generation: witness.generation.as_deref(),
                role: EquipmentOriginRole::AreaDamage,
                equipment_uuid: Some(contributor.equipment.definition_uuid),
                definition_id: Some(&contributor.definition_id),
                source_file: &contributor.source_file,
                field_path: &contributor.field_path,
            })
        }).collect())
    }
    pub fn latest_nearby_ally_damage_origins(&self) -> Option<Vec<Option<SourceTrace>>> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_nearby_ally_damage()?;
        if witness.generation.as_deref() != Some(&self.identity.content) { return None; }
        Some(witness.contributors.iter().map(|contributor| {
            self.observed_equipment_origin(&EquipmentOriginClaim {
                generation: witness.generation.as_deref(),
                role: EquipmentOriginRole::NearbyAllyDamage,
                equipment_uuid: Some(contributor.equipment.definition_uuid),
                definition_id: Some(&contributor.definition_id),
                source_file: &contributor.source_file,
                field_path: &contributor.field_path,
            })
        }).collect())
    }
    pub fn latest_conditional_damage_origins(&self) -> Option<Vec<Option<SourceTrace>>> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_conditional_damage()?;
        if witness.generation.as_deref() != Some(&self.identity.content) { return None; }
        Some(witness.contributors.iter().map(|contributor| {
            self.observed_equipment_origin(&EquipmentOriginClaim {
                generation: witness.generation.as_deref(),
                role: EquipmentOriginRole::ConditionalDamage,
                equipment_uuid: Some(contributor.equipment.definition_uuid),
                definition_id: Some(&contributor.definition_id),
                source_file: &contributor.source_file,
                field_path: &contributor.field_path,
            })
        }).collect())
    }
    pub fn latest_direct_penetration_origins(&self) -> Option<Vec<Option<SourceTrace>>> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_direct_penetration()?;
        if witness.generation.as_deref() != Some(&self.identity.content) { return None; }
        Some(witness.contributors.iter().map(|contributor| {
            self.observed_equipment_origin(&EquipmentOriginClaim {
                generation: witness.generation.as_deref(),
                role: EquipmentOriginRole::DirectPenetration(contributor.damage_type),
                equipment_uuid: Some(contributor.equipment.definition_uuid),
                definition_id: Some(&contributor.definition_id),
                source_file: &contributor.source_file,
                field_path: &contributor.field_path,
            })
        }).collect())
    }
    pub fn latest_conditional_reduction_origins(&self) -> Option<Vec<Option<SourceTrace>>> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = &observed.latest_conditional_reduction()?.evaluation;
        if witness.generation.as_deref() != Some(&self.identity.content) { return None; }
        Some(witness.contributors.iter().map(|contributor| {
            self.observed_equipment_origin(&EquipmentOriginClaim {
                generation: witness.generation.as_deref(),
                role: EquipmentOriginRole::ConditionalReduction,
                equipment_uuid: Some(contributor.equipment.definition_uuid),
                definition_id: Some(&contributor.definition_id),
                source_file: &contributor.source_file,
                field_path: &contributor.field_path,
            })
        }).collect())
    }
    pub fn latest_actual_heal_reaction_origins(&self) -> Option<Vec<Option<SourceTrace>>> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_actual_heal_reaction()?;
        if witness.generation.as_deref() != Some(&self.identity.content) { return None; }
        Some(witness.contributors.iter().map(|contributor| {
            self.observed_equipment_origin(&EquipmentOriginClaim {
                generation: witness.generation.as_deref(),
                role: EquipmentOriginRole::ActualHealReaction,
                equipment_uuid: Some(contributor.equipment.definition_uuid),
                definition_id: Some(&contributor.definition_id),
                source_file: &contributor.source_file,
                field_path: &contributor.field_path,
            })
        }).collect())
    }
    pub fn latest_resonance_gain_origins(&self) -> Option<Vec<Option<SourceTrace>>> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_resonance_gain()?;
        if witness.generation.as_deref() != Some(&self.identity.content) { return None; }
        Some(witness.contributors.iter().map(|contributor| {
            self.observed_equipment_origin(&EquipmentOriginClaim {
                generation: witness.generation.as_deref(),
                role: EquipmentOriginRole::ResonanceGain,
                equipment_uuid: Some(contributor.equipment.definition_uuid),
                definition_id: Some(&contributor.definition_id),
                source_file: &contributor.source_file,
                field_path: &contributor.field_path,
            })
        }).collect())
    }
    pub fn latest_equipment_speed_origins(&self) -> Option<Vec<Option<SourceTrace>>> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_equipment_speed()?;
        if witness.generation.as_deref() != Some(&self.identity.content) { return None; }
        Some(witness.contributors.iter().map(|contributor| {
            self.observed_equipment_origin(&EquipmentOriginClaim {
                generation: witness.generation.as_deref(),
                role: EquipmentOriginRole::EquipmentSpeed,
                equipment_uuid: Some(contributor.equipment.definition_uuid),
                definition_id: Some(&contributor.definition_id),
                source_file: &contributor.source_file,
                field_path: &contributor.field_path,
            })
        }).collect())
    }
    pub fn latest_equipment_support_origins(&self) -> Option<Vec<Option<SourceTrace>>> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active()
            .or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_equipment_support()?;
        if witness.generation.as_deref() != Some(&self.identity.content) { return None; }
        Some(witness.contributors.iter().map(|contributor| {
            self.observed_equipment_origin(&EquipmentOriginClaim {
                generation: witness.generation.as_deref(),
                role: EquipmentOriginRole::EquipmentSupport(witness.channel),
                equipment_uuid: Some(contributor.equipment.definition_uuid),
                definition_id: Some(&contributor.definition_id),
                source_file: &contributor.source_file,
                field_path: &contributor.field_path,
            })
        }).collect())
    }
    pub fn latest_heal_barrier_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_heal_barrier()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::HealBarrier,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_overheal_reserve_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_overheal_reserve()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::OverhealReserve,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_shared_cohort_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_shared_cohort()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file.as_deref() != Some(&self.identity.artifact_path) { return None; }
        witness.equipment_copy?;
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::SharedCohort,
            equipment_uuid: None,
            definition_id: Some(&witness.definition),
            source_file: witness.source_file.as_deref()?,
            field_path: witness.field_path.as_deref()?,
        })
    }
    pub fn latest_employee_transformation_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_employee_transformation()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::EmployeeTransformation,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_last_will_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_last_will()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::LastWill,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_weapon_stage_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_weapon_stage()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::WeaponStage(witness.stage),
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_equipment_death_burst_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_equipment_death_burst()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::EquipmentDeathBurst,
            equipment_uuid: None,
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_equipment_form_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_equipment_form()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::EquipmentForm,
            equipment_uuid: None,
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_bounty_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_bounty()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::Bounty,
            equipment_uuid: None,
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_magazine_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_magazine()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::Magazine,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_deferred_damage_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_deferred_damage()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::DeferredDamage,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }
    pub fn latest_basic_heal_echo_origin(&self) -> Option<SourceTrace> {
        let observation = self.session.observation().ok()?;
        let observed = observation.active().or_else(|| observation.last_terminal().map(|t| t.mandatory()))?;
        let witness = observed.latest_basic_heal_echo()?;
        if witness.generation.as_deref() != Some(&self.identity.content) || witness.source_file != self.identity.artifact_path { return None; }
        self.observed_equipment_origin(&EquipmentOriginClaim {
            generation: witness.generation.as_deref(),
            role: EquipmentOriginRole::BasicHealEcho,
            equipment_uuid: Some(witness.equipment.definition_uuid),
            definition_id: Some(&witness.definition_id),
            source_file: &witness.source_file,
            field_path: &witness.field_path,
        })
    }

}
