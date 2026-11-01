//! Immutable battle facts and source identities, independent of event logging.
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::{ids::UnitInstanceId, skill_source::SkillCastExecutionSource};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ActionOrdinal(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct StrikeOrdinal(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub(crate) struct DeliveryOrdinal(pub u64);

#[derive(Debug)]
pub(crate) struct AttemptToken;
#[derive(Debug)]
pub(crate) struct ActionToken {
    pub(crate) attempt: Arc<AttemptToken>,
    pub(crate) ordinal: ActionOrdinal,
}

/// A clone-relative capability. Serialization exposes only a diagnostic ordinal;
/// it cannot reconstruct the immutable attempt and occurrence validity tokens.
#[derive(Clone)]
pub struct ActionHandle {
    pub(crate) validity: Arc<ActionToken>,
}

impl ActionHandle {
    pub fn ordinal(&self) -> ActionOrdinal {
        self.validity.ordinal
    }
}
impl std::fmt::Debug for ActionHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionHandle")
            .field("ordinal", &self.validity.ordinal)
            .finish_non_exhaustive()
    }
}
impl PartialEq for ActionHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.validity, &other.validity)
    }
}
impl Eq for ActionHandle {}
impl Serialize for ActionHandle {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.validity.ordinal.serialize(serializer)
    }
}
crate::authoring::clone_shape::struct_shape!(ActionHandle { validity });
crate::authoring::clone_shape::copy_shape!(ActionOrdinal, StrikeOrdinal, DeliveryOrdinal);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeliveryTicket {
    pub action: ActionHandle,
    pub ordinal: DeliveryOrdinal,
    pub resource: Option<Arc<crate::game::resource::runtime::ResourceDelivery>>,
}
crate::authoring::clone_shape::struct_shape!(DeliveryTicket {
    action,
    ordinal,
    resource
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionQueryError {
    ForcedPause,
    DifferentAttempt,
    Unavailable,
    StaleOccurrence,
    NotSettled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionCancelReason {
    CasterDied,
    Withdrawn,
    HardCrowdControl,
    Interrupted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionFailureReason {
    CasterUnavailable,
    DefinitionUnavailable,
    OriginUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "reason", rename_all = "snake_case")]
pub enum ActionTerminalKind {
    Finished,
    Cancelled(ActionCancelReason),
    Failed(ActionFailureReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ActionTerminal {
    pub time_ms: u64,
    pub kind: ActionTerminalKind,
}
crate::authoring::clone_shape::copy_shape!(
    ActionQueryError,
    ActionCancelReason,
    ActionFailureReason,
    ActionTerminalKind,
    ActionTerminal
);

/// The actual provider captured while its runtime item is still available.
/// Scenario-created equipment may have no persistent inventory copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct EquipmentSourceSnapshot {
    pub definition_uuid: Uuid,
    pub runtime_item_instance_id: Uuid,
    pub persistent_copy_uuid: Option<Uuid>,
}

crate::authoring::clone_shape::struct_shape!(EquipmentSourceSnapshot {
    definition_uuid,
    runtime_item_instance_id,
    persistent_copy_uuid,
});

/// Independently known source slots. Missing authoring metadata does not erase
/// a known cast; missing inventory identity does not erase a runtime provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResourceSourceCapabilities {
    context: Option<Arc<crate::game::resource::runtime::ResourceContext>>,
    delivery: Option<Arc<crate::game::resource::runtime::ResourceDelivery>>,
}
crate::authoring::clone_shape::struct_shape!(ResourceSourceCapabilities { context, delivery });

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FlowOccurrence {
    pub node_index: usize,
    pub occurrence: usize,
}
crate::authoring::clone_shape::struct_shape!(FlowOccurrence {
    node_index,
    occurrence
});

/// Captured only by the validated current sustained activation at reinput commit.
/// A fixed duration remains fixed for Snapshot and Live evaluations of Goodbye.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct CommittedReinputContext {
    pub activation: u64,
    pub committed_at_ms: u64,
    pub remaining_ms: u64,
}
crate::authoring::clone_shape::copy_shape!(CommittedReinputContext);

/// Creation provenance has no execution credentials and cannot contain another
/// source graph. The object's own action/cast remains in RuntimeFactSource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndependentObjectOriginSnapshot {
    pub object_id: UnitInstanceId,
    pub definition_id: crate::game::data::independent_object_data::IndependentObjectId,
    pub creator: UnitInstanceId,
    pub creation_action: Option<ActionOrdinal>,
    pub creation_cast: Option<Arc<SkillCastExecutionSource>>,
    pub creation_fragment: Option<Arc<crate::game::skill_fragment::FragmentSourceSnapshot>>,
    pub creation_equipment: Option<EquipmentSourceSnapshot>,
    pub creation_authored: Option<Arc<crate::game::ability::authoring::EffectSource>>,
}
crate::authoring::clone_shape::struct_shape!(IndependentObjectOriginSnapshot {
    object_id,
    definition_id,
    creator,
    creation_action,
    creation_cast,
    creation_fragment,
    creation_equipment,
    creation_authored
});
impl IndependentObjectOriginSnapshot {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let observations = RuntimeFactSource {
            cast: self.creation_cast.clone(),
            fragment: self.creation_fragment.clone(),
            equipment: self.creation_equipment,
            authored: self.creation_authored.clone(),
            ..Default::default()
        };
        std::mem::size_of::<Self>()
            .checked_add(self.definition_id.0.capacity())?
            .checked_add(observations.retained_extent()?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusPredecessor {
    pub activation: u64,
    pub owner: UnitInstanceId,
    pub fragment: Arc<crate::game::skill_fragment::FragmentSourceSnapshot>,
    pub definition: String,
}
crate::authoring::clone_shape::struct_shape!(StatusPredecessor {
    activation,
    owner,
    fragment,
    definition
});
impl StatusPredecessor {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(self.definition.capacity())?
            .checked_add(
                RuntimeFactSource {
                    fragment: Some(self.fragment.clone()),
                    ..Default::default()
                }
                .retained_extent()?,
            )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusTransitionOrigin {
    pub previous: Vec<StatusPredecessor>,
}
crate::authoring::clone_shape::struct_shape!(StatusTransitionOrigin { previous });
impl StatusTransitionOrigin {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = std::mem::size_of::<Self>().checked_add(
            self.previous
                .capacity()
                .checked_mul(std::mem::size_of::<StatusPredecessor>())?,
        )?;
        for p in &self.previous {
            n = n.checked_add(p.retained_extent()?)?;
        }
        Some(n)
    }
}
/// Rare observation slots share one optional allocation. Ordinary sources keep
/// the same inline footprint as before independent objects were introduced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MobileEffectOrigin {
    pub motion: crate::game::ability::mobile_effect::MobileMotion,
    pub death_origin: Option<Arc<crate::game::ability::mobile_effect::MobileDeathOrigin>>,
    pub seed_origin: Option<Arc<crate::game::ability::mobile_effect::MobileSeedOrigin>>,
    pub activation: u64,
    pub owner: UnitInstanceId,
    pub copy: Uuid,
    pub definition: String,
}
crate::authoring::clone_shape::struct_shape!(MobileEffectOrigin {
    motion,
    death_origin,
    seed_origin,
    activation,
    owner,
    copy,
    definition
});
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ExtendedSourceObservation {
    independent_object: Option<Arc<IndependentObjectOriginSnapshot>>,
    reinput: Option<Arc<CommittedReinputContext>>,
    relic_numeric: Option<Arc<super::core::relic_numeric::RelicNumericBinding>>,
    relic_effective: Option<Arc<super::core::effective_skill::EffectiveSkillBinding>>,
    regular_support: Option<Arc<super::damage::RegularBasicSupportSnapshot>>,
    status_transition: Option<Arc<StatusTransitionOrigin>>,
    guard_release: Option<Arc<crate::game::ability::guard_charge::GuardReleaseOrigin>>,
    mobile_effect: Option<Arc<MobileEffectOrigin>>,
    causal: Option<Arc<super::core::relic_causal::CausalSourceData>>,
    cast_damage: Option<Arc<super::skill_source::CastDamageSnapshot>>,
}
crate::authoring::clone_shape::struct_shape!(ExtendedSourceObservation {
    independent_object,
    reinput,
    relic_numeric,
    relic_effective,
    regular_support,
    status_transition,
    guard_release,
    mobile_effect,
    causal,
    cast_damage
});

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeFactSource {
    pub(crate) extended_observation: Option<Arc<ExtendedSourceObservation>>,
    /// A scheduled, still-unreleased attack belonging to an admitted skill use.
    pub(crate) use_control: bool,
    /// Runtime-only provenance; authored data cannot forge a reaction.
    pub(crate) reaction: bool,
    /// Native chain permission; the actual copy is already in `equipment`.
    /// Keep rare provenance flags in the source's existing inline padding.
    pub(crate) death_burst: bool,
    pub flow: Option<Arc<FlowOccurrence>>,
    pub actor: Option<UnitInstanceId>,
    pub equipment: Option<EquipmentSourceSnapshot>,
    pub fragment: Option<Arc<crate::game::skill_fragment::FragmentSourceSnapshot>>,
    pub cast: Option<Arc<SkillCastExecutionSource>>,
    pub authored: Option<Arc<crate::game::ability::authoring::EffectSource>>,
    pub(crate) action: Option<ActionHandle>,
    pub(crate) delivery: Option<Arc<DeliveryTicket>>,
    pub(crate) allows_finite_children: bool,
    pub(crate) resource_capabilities: Option<Arc<ResourceSourceCapabilities>>,
}

crate::authoring::clone_shape::struct_shape!(RuntimeFactSource {
    extended_observation,
    use_control,
    reaction,
    death_burst,
    flow,
    actor,
    equipment,
    fragment,
    cast,
    authored,
    action,
    delivery,
    allows_finite_children,
    resource_capabilities,
});

impl Serialize for RuntimeFactSource {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Preserve the existing public fields, omission rules and order. The
        // storage wrapper is private and grants no execution authority.
        #[derive(Serialize)]
        struct Observation<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            independent_object: Option<&'a Arc<IndependentObjectOriginSnapshot>>,
            #[serde(skip_serializing_if = "std::ops::Not::not")]
            reaction: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            death_burst: Option<Uuid>,
            #[serde(skip_serializing_if = "Option::is_none")]
            reinput: Option<&'a Arc<CommittedReinputContext>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            regular_support: Option<&'a Arc<super::damage::RegularBasicSupportSnapshot>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            status_transition: Option<&'a Arc<StatusTransitionOrigin>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            guard_release: Option<&'a Arc<crate::game::ability::guard_charge::GuardReleaseOrigin>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            mobile_effect: Option<&'a Arc<MobileEffectOrigin>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            flow: &'a Option<Arc<FlowOccurrence>>,
            actor: &'a Option<UnitInstanceId>,
            equipment: &'a Option<EquipmentSourceSnapshot>,
            #[serde(skip_serializing_if = "Option::is_none")]
            fragment: &'a Option<Arc<crate::game::skill_fragment::FragmentSourceSnapshot>>,
            cast: &'a Option<Arc<SkillCastExecutionSource>>,
            authored: &'a Option<Arc<crate::game::ability::authoring::EffectSource>>,
            action_ordinal: &'a Option<ActionHandle>,
            #[serde(skip_serializing_if = "Option::is_none")]
            relic_numeric: Option<&'a Arc<super::core::relic_numeric::RelicNumericBinding>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            relic_effective: Option<&'a Arc<super::core::effective_skill::EffectiveSkillBinding>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            causal: Option<&'a Arc<super::core::relic_causal::CausalContext>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            causal_cause: Option<&'a Arc<super::core::relic_causal::CausalCauseObservation>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            cast_damage: Option<&'a Arc<super::skill_source::CastDamageSnapshot>>,
        }
        Observation {
            independent_object: self.independent_object(),
            reaction: self.reaction,
            death_burst: self.death_burst_copy(),
            reinput: self.reinput(),
            regular_support: self.regular_support(),
            status_transition: self.status_transition(),
            guard_release: self.guard_release(),
            mobile_effect: self.mobile_effect(),
            flow: &self.flow,
            actor: &self.actor,
            equipment: &self.equipment,
            fragment: &self.fragment,
            cast: &self.cast,
            authored: &self.authored,
            action_ordinal: &self.action,
            relic_numeric: self.relic_numeric(),
            relic_effective: self.relic_effective(),
            causal: self.causal().and_then(|c| c.context.as_ref()),
            causal_cause: self.causal().and_then(|c| c.cause.as_ref()),
            cast_damage: self.cast_damage(),
        }
        .serialize(serializer)
    }
}

impl RuntimeFactSource {
    pub fn cast_damage(&self) -> Option<&Arc<super::skill_source::CastDamageSnapshot>> {
        self.extended_observation.as_ref().and_then(|s| s.cast_damage.as_ref())
    }
    pub(crate) fn with_cast_damage(&self, capture: Option<Arc<super::skill_source::CastDamageSnapshot>>) -> Result<Self, crate::game::behavior::GameError> {
        if capture.is_none() && self.cast_damage().is_none() { return Ok(self.clone()); }
        super::runtime_control::admit_retained_growth(0,
            Some(std::mem::size_of::<ExtendedSourceObservation>() + 64), "cast_damage_source")?;
        let mut value = self.clone();
        let mut extended = self.extended_observation.as_deref().cloned().unwrap_or_default();
        extended.cast_damage = capture;
        value.extended_observation = Some(Arc::new(extended));
        Ok(value)
    }
    pub(crate) fn death_burst_copy(&self) -> Option<Uuid> {
        self.death_burst
            .then(|| self.equipment.and_then(|e| e.persistent_copy_uuid))
            .flatten()
    }
    pub(crate) fn extended_observation(
        independent_object: Option<Arc<IndependentObjectOriginSnapshot>>,
        reinput: Option<Arc<CommittedReinputContext>>,
    ) -> Result<Option<Arc<ExtendedSourceObservation>>, crate::game::behavior::GameError> {
        if independent_object.is_none() && reinput.is_none() {
            return Ok(None);
        }
        super::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<ExtendedSourceObservation>() + 64),
            "extended_source_observation",
        )?;
        Ok(Some(Arc::new(ExtendedSourceObservation {
            independent_object,
            reinput,
            relic_numeric: None,
            relic_effective: None,
            regular_support: None,
            status_transition: None,
            guard_release: None,
            mobile_effect: None,
            causal: None,
            cast_damage: None,
        })))
    }

    pub(crate) fn causal(&self) -> Option<&Arc<super::core::relic_causal::CausalSourceData>> {
        self.extended_observation
            .as_ref()
            .and_then(|s| s.causal.as_ref())
    }
    pub(crate) fn causal_program_source(&self) -> Result<Self, crate::game::behavior::GameError> {
        super::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<ExtendedSourceObservation>() + 64),
            "causal_program_observation",
        )?;
        let mut source = self.clone();
        source.extended_observation = Some(Arc::new(ExtendedSourceObservation {
            independent_object: self.independent_object().cloned(),
            reinput: None,
            relic_numeric: None,
            relic_effective: None,
            regular_support: None,
            status_transition: self.status_transition().cloned(),
            guard_release: self.guard_release().cloned(),
            mobile_effect: self.mobile_effect().cloned(),
            causal: self.causal().cloned(),
            cast_damage: self.cast_damage().cloned(),
        }));
        Ok(source)
    }
    /// Reattribution may select a new native program/provider; only the causal
    /// path and suppression travel across that boundary, never the old program.
    pub(crate) fn inherit_causal_context(
        &self,
        parent: &Self,
    ) -> Result<Self, crate::game::behavior::GameError> {
        let Some(context) = parent.causal().and_then(|c| c.context.clone()) else {
            return Ok(self.clone());
        };
        if self.causal().and_then(|c| c.context.as_ref()).is_some() {
            return Ok(self.clone());
        }
        super::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<super::core::relic_causal::CausalSourceData>() + 64),
            "derived_causal_source",
        )?;
        self.with_causal(Some(Arc::new(
            super::core::relic_causal::CausalSourceData {
                context: Some(context),
                program: self.causal().and_then(|c| c.program.clone()),
                // This is the child's already accepted amount capture. The
                // parent contributes ancestry, never a replacement amount.
                amount: self.causal().and_then(|c| c.amount.clone()),
                cause: self
                    .causal()
                    .and_then(|c| c.cause.clone())
                    .or_else(|| parent.causal().and_then(|c| c.cause.clone())),
            },
        )))
    }
    pub(crate) fn with_causal(
        &self,
        causal: Option<Arc<super::core::relic_causal::CausalSourceData>>,
    ) -> Result<Self, crate::game::behavior::GameError> {
        super::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<ExtendedSourceObservation>() + 64),
            "causal_source_observation",
        )?;
        let mut source = self.clone();
        source.extended_observation = Some(Arc::new(ExtendedSourceObservation {
            independent_object: self.independent_object().cloned(),
            reinput: self.reinput().cloned(),
            relic_numeric: self.relic_numeric().cloned(),
            relic_effective: self.relic_effective().cloned(),
            regular_support: self.regular_support().cloned(),
            status_transition: self.status_transition().cloned(),
            guard_release: self.guard_release().cloned(),
            mobile_effect: self.mobile_effect().cloned(),
            causal,
            cast_damage: self.cast_damage().cloned(),
        }));
        Ok(source)
    }
    pub(crate) fn relic_effective(
        &self,
    ) -> Option<&Arc<super::core::effective_skill::EffectiveSkillBinding>> {
        self.extended_observation
            .as_ref()
            .and_then(|value| value.relic_effective.as_ref())
    }
    pub(crate) fn with_relic_effective(
        &self,
        binding: Arc<super::core::effective_skill::EffectiveSkillBinding>,
    ) -> Result<Self, crate::game::behavior::GameError> {
        super::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<ExtendedSourceObservation>() + 64),
            "effective_source_observation",
        )?;
        let mut value = self.clone();
        value.extended_observation = Some(Arc::new(ExtendedSourceObservation {
            independent_object: self.independent_object().cloned(),
            reinput: self.reinput().cloned(),
            relic_numeric: self.relic_numeric().cloned(),
            relic_effective: Some(binding),
            regular_support: self.regular_support().cloned(),
            status_transition: self.status_transition().cloned(),
            guard_release: self.guard_release().cloned(),
            mobile_effect: self.mobile_effect().cloned(),
            causal: self.causal().cloned(),
            cast_damage: self.cast_damage().cloned(),
        }));
        Ok(value)
    }
    pub fn guard_release(&self) -> Option<&Arc<crate::game::ability::guard_charge::GuardReleaseOrigin>> {
        self.extended_observation.as_ref().and_then(|s| s.guard_release.as_ref())
    }
    pub(crate) fn with_guard_release(&self, origin: crate::game::ability::guard_charge::GuardReleaseOrigin) -> Result<Self, crate::game::behavior::GameError> {
        super::runtime_control::admit_retained_growth(0, std::mem::size_of::<ExtendedSourceObservation>().checked_add(std::mem::size_of_val(&origin)+128).and_then(|n|n.checked_add(origin.definition.capacity())).and_then(|n|n.checked_add(origin.relic_definition.capacity())), "guard_release_origin")?;
        let mut value=self.clone();
        value.extended_observation=Some(Arc::new(ExtendedSourceObservation {
            independent_object:self.independent_object().cloned(),reinput:self.reinput().cloned(),relic_numeric:self.relic_numeric().cloned(),relic_effective:self.relic_effective().cloned(),regular_support:self.regular_support().cloned(),status_transition:self.status_transition().cloned(),mobile_effect:self.mobile_effect().cloned(),guard_release:Some(Arc::new(origin)),causal:self.causal().cloned(),cast_damage:self.cast_damage().cloned(),
        }));
        Ok(value)
    }
    pub fn status_transition(&self) -> Option<&Arc<StatusTransitionOrigin>> {
        self.extended_observation
            .as_ref()
            .and_then(|s| s.status_transition.as_ref())
    }
    pub(crate) fn with_status_transition(
        &self,
        origin: StatusTransitionOrigin,
    ) -> Result<Self, crate::game::behavior::GameError> {
        super::runtime_control::admit_retained_growth(
            0,
            origin
                .retained_extent()
                .and_then(|n| n.checked_add(std::mem::size_of::<ExtendedSourceObservation>() + 64)),
            "status_transition_origin",
        )?;
        let mut value = crate::authoring::clone_shape::controlled_clone(self)?;
        value.extended_observation = Some(Arc::new(ExtendedSourceObservation {
            independent_object: self.independent_object().cloned(),
            reinput: self.reinput().cloned(),
            relic_numeric: self.relic_numeric().cloned(),
            relic_effective: self.relic_effective().cloned(),
            regular_support: self.regular_support().cloned(),
            mobile_effect: self.mobile_effect().cloned(),
            status_transition: Some(Arc::new(origin)),
            guard_release: self.guard_release().cloned(),
            causal: self.causal().cloned(),
            cast_damage: self.cast_damage().cloned(),
        }));
        Ok(value)
    }
    pub fn mobile_effect(&self) -> Option<&Arc<MobileEffectOrigin>> {
        self.extended_observation
            .as_ref()
            .and_then(|s| s.mobile_effect.as_ref())
    }
    pub(crate) fn with_mobile_effect(
        &self,
        origin: MobileEffectOrigin,
    ) -> Result<Self, crate::game::behavior::GameError> {
        super::runtime_control::admit_retained_growth(
            0,
            origin
                .death_origin
                .as_ref()
                .map_or(Some(0), |d| d.retained_extent()),
            "mobile_death_origin",
        )?;
        super::runtime_control::admit_retained_growth(
            0,
            std::mem::size_of::<ExtendedSourceObservation>()
                .checked_add(std::mem::size_of::<MobileEffectOrigin>() + 128)
                .and_then(|n| n.checked_add(origin.definition.capacity()))
                .and_then(|n| {
                    n.checked_add(
                        origin
                            .seed_origin
                            .as_ref()
                            .map_or(Some(0), |s| s.retained_extent())?,
                    )
                }),
            "mobile_effect_origin",
        )?;
        let mut value = self.clone();
        value.extended_observation = Some(Arc::new(ExtendedSourceObservation {
            independent_object: self.independent_object().cloned(),
            reinput: self.reinput().cloned(),
            relic_numeric: self.relic_numeric().cloned(),
            relic_effective: self.relic_effective().cloned(),
            regular_support: self.regular_support().cloned(),
            status_transition: self.status_transition().cloned(),
            guard_release: self.guard_release().cloned(),
            mobile_effect: Some(Arc::new(origin)),
            causal: self.causal().cloned(),
            cast_damage: self.cast_damage().cloned(),
        }));
        Ok(value)
    }
    pub(crate) fn regular_support(
        &self,
    ) -> Option<&Arc<super::damage::RegularBasicSupportSnapshot>> {
        self.extended_observation
            .as_ref()
            .and_then(|value| value.regular_support.as_ref())
    }
    pub(crate) fn with_regular_support(
        &self,
        support: Option<Arc<super::damage::RegularBasicSupportSnapshot>>,
    ) -> Result<Self, crate::game::behavior::GameError> {
        if support.is_none() && self.regular_support().is_none() {
            return Ok(self.clone());
        }
        super::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<ExtendedSourceObservation>() + 64),
            "regular_support_source",
        )?;
        let mut value = self.clone();
        value.extended_observation = Some(Arc::new(ExtendedSourceObservation {
            independent_object: self.independent_object().cloned(),
            reinput: self.reinput().cloned(),
            relic_numeric: self.relic_numeric().cloned(),
            relic_effective: self.relic_effective().cloned(),
            regular_support: support,
            status_transition: self.status_transition().cloned(),
            guard_release: self.guard_release().cloned(),
            mobile_effect: self.mobile_effect().cloned(),
            causal: self.causal().cloned(),
            cast_damage: self.cast_damage().cloned(),
        }));
        Ok(value)
    }
    pub(crate) fn relic_numeric(
        &self,
    ) -> Option<&Arc<super::core::relic_numeric::RelicNumericBinding>> {
        self.extended_observation
            .as_ref()
            .and_then(|value| value.relic_numeric.as_ref())
    }

    /// Private producer only. Observed/serialized provenance has no decoder
    /// that can reconstruct this immutable execution binding.
    pub(crate) fn with_relic_numeric(
        &self,
        binding: Arc<super::core::relic_numeric::RelicNumericBinding>,
    ) -> Result<Self, crate::game::behavior::GameError> {
        super::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<ExtendedSourceObservation>() + 64),
            "relic_source_observation",
        )?;
        let mut value = self.clone();
        value.extended_observation = Some(Arc::new(ExtendedSourceObservation {
            independent_object: self.independent_object().cloned(),
            reinput: self.reinput().cloned(),
            relic_numeric: Some(binding),
            relic_effective: self.relic_effective().cloned(),
            regular_support: self.regular_support().cloned(),
            status_transition: self.status_transition().cloned(),
            guard_release: self.guard_release().cloned(),
            mobile_effect: self.mobile_effect().cloned(),
            causal: self.causal().cloned(),
            cast_damage: self.cast_damage().cloned(),
        }));
        Ok(value)
    }

    /// Rebind the same execution source with the capture's exact numeric slot.
    /// Other cold provenance belongs to the incoming delivery and stays intact.
    pub(crate) fn with_exact_relic_numeric(
        &self,
        binding: Option<&Arc<super::core::relic_numeric::RelicNumericBinding>>,
    ) -> Result<Self, crate::game::behavior::GameError> {
        if let Some(binding) = binding {
            return self.with_relic_numeric(Arc::clone(binding));
        }
        if self.relic_numeric().is_none() {
            return Ok(self.clone());
        }
        // Replace only numeric ownership; all other observations survive.
        let mut value = self.clone();
        let has_other = self.independent_object().is_some() || self.reinput().is_some()
            || self.relic_effective().is_some() || self.regular_support().is_some()
            || self.status_transition().is_some() || self.mobile_effect().is_some()
            || self.guard_release().is_some() || self.causal().is_some() || self.cast_damage().is_some();
        value.extended_observation = if has_other {
            super::runtime_control::admit_retained_growth(0, Some(std::mem::size_of::<ExtendedSourceObservation>()+64), "extended_source_observation")?;
            Some(Arc::new(ExtendedSourceObservation {
                independent_object:self.independent_object().cloned(),reinput:self.reinput().cloned(),relic_numeric:None,relic_effective:self.relic_effective().cloned(),regular_support:self.regular_support().cloned(),status_transition:self.status_transition().cloned(),mobile_effect:self.mobile_effect().cloned(),guard_release:self.guard_release().cloned(),causal:self.causal().cloned(),cast_damage:self.cast_damage().cloned(),
            }))
        } else { None };
        Ok(value)
    }

    /// Remove only this observation of the same execution source. Immutable
    /// numeric ownership and independent creation provenance survive together.
    pub(crate) fn without_reinput(&self) -> Result<Self, crate::game::behavior::GameError> {
        let mut value = self.clone();
        if self.reinput().is_none() {
            return Ok(value);
        }
        if self.independent_object().is_none()
            && self.relic_numeric().is_none()
            && self.relic_effective().is_none()
            && self.regular_support().is_none()
            && self.status_transition().is_none()
            && self.mobile_effect().is_none()
            && self.guard_release().is_none()
            && self.causal().is_none()
            && self.cast_damage().is_none()
        {
            value.extended_observation = None;
            return Ok(value);
        }
        super::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<ExtendedSourceObservation>() + 64),
            "extended_source_observation",
        )?;
        value.extended_observation = Some(Arc::new(ExtendedSourceObservation {
            independent_object: self.independent_object().cloned(),
            reinput: None,
            relic_numeric: self.relic_numeric().cloned(),
            relic_effective: self.relic_effective().cloned(),
            regular_support: self.regular_support().cloned(),
            status_transition: self.status_transition().cloned(),
            guard_release: self.guard_release().cloned(),
            mobile_effect: self.mobile_effect().cloned(),
            causal: self.causal().cloned(),
            cast_damage: self.cast_damage().cloned(),
        }));
        Ok(value)
    }

    pub fn independent_object(&self) -> Option<&Arc<IndependentObjectOriginSnapshot>> {
        self.extended_observation
            .as_ref()
            .and_then(|value| value.independent_object.as_ref())
    }

    pub(crate) fn reinput(&self) -> Option<&Arc<CommittedReinputContext>> {
        self.extended_observation
            .as_ref()
            .and_then(|value| value.reinput.as_ref())
    }

    pub(crate) fn resource_capabilities(
        context: Option<Arc<crate::game::resource::runtime::ResourceContext>>,
        delivery: Option<Arc<crate::game::resource::runtime::ResourceDelivery>>,
    ) -> Result<Option<Arc<ResourceSourceCapabilities>>, crate::game::behavior::GameError> {
        if context.is_none() && delivery.is_none() {
            return Ok(None);
        }
        super::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<ResourceSourceCapabilities>() + 64),
            "resource_source_capabilities",
        )?;
        Ok(Some(Arc::new(ResourceSourceCapabilities {
            context,
            delivery,
        })))
    }

    pub(crate) fn resource_context(
        &self,
    ) -> Option<&Arc<crate::game::resource::runtime::ResourceContext>> {
        self.resource_capabilities
            .as_ref()
            .and_then(|value| value.context.as_ref())
    }

    pub(crate) fn resource_delivery(
        &self,
    ) -> Option<&Arc<crate::game::resource::runtime::ResourceDelivery>> {
        self.resource_capabilities
            .as_ref()
            .and_then(|value| value.delivery.as_ref())
    }

    pub fn action_ordinal(&self) -> Option<ActionOrdinal> {
        self.action.as_ref().map(ActionHandle::ordinal)
    }

    /// A conservative retained extent per owner, including shared pointees.
    /// It does not treat an Arc handle as the size of its dynamic payload.
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = std::mem::size_of::<Self>();
        if let Some(origin)=self.guard_release() {
            bytes=bytes.checked_add(std::mem::size_of_val(origin.as_ref())+64)?.checked_add(origin.definition.capacity())?.checked_add(origin.relic_definition.capacity())?;
        }
        if let Some(origin) = self.status_transition() {
            bytes = bytes.checked_add(origin.retained_extent()?)?;
        }
        if let Some(origin) = self.mobile_effect() {
            bytes = bytes.checked_add(
                origin
                    .death_origin
                    .as_ref()
                    .map_or(Some(0), |d| d.retained_extent())?,
            )?;
            bytes = bytes
                .checked_add(std::mem::size_of::<MobileEffectOrigin>() + 64)?
                .checked_add(origin.definition.capacity())?
                .checked_add(
                    origin
                        .seed_origin
                        .as_ref()
                        .map_or(Some(0), |s| s.retained_extent())?,
                )?;
        }
        if self.extended_observation.is_some() {
            bytes = bytes.checked_add(std::mem::size_of::<ExtendedSourceObservation>() + 64)?;
        }
        if let Some(causal) = self.causal() {
            bytes = bytes.checked_add(causal.retained_extent()?)?;
        }
        if let Some(binding) = self.relic_numeric() {
            bytes = bytes.checked_add(binding.retained_extent()?)?;
        }
        if let Some(binding) = self.relic_effective() {
            bytes = bytes.checked_add(binding.retained_extent()?)?;
        }
        if let Some(origin) = self.independent_object() {
            bytes = bytes.checked_add(origin.retained_extent()?.checked_add(64)?)?;
        }
        if let Some(fragment) = &self.fragment {
            bytes = bytes.checked_add(fragment.retained_extent()?.checked_add(64)?)?;
        }
        // The compact handle's one immutable token owns its attempt identity
        // and ordinal. Count that shared allocation conservatively per owner.
        let token_bytes = std::mem::size_of::<ActionToken>().checked_add(64)?;
        if self.action.is_some() {
            bytes = bytes.checked_add(token_bytes)?;
        }
        if self.delivery.is_some() {
            bytes = bytes
                .checked_add(token_bytes)?
                .checked_add(std::mem::size_of::<DeliveryTicket>() + 64)?;
        }
        if self.reinput().is_some() {
            bytes = bytes.checked_add(std::mem::size_of::<CommittedReinputContext>() + 64)?;
        }
        if self.flow.is_some() {
            bytes = bytes.checked_add(std::mem::size_of::<FlowOccurrence>() + 64)?;
        }
        if self.resource_capabilities.is_some() {
            bytes = bytes.checked_add(std::mem::size_of::<ResourceSourceCapabilities>() + 64)?;
        }
        if self.resource_context().is_some() {
            bytes = bytes.checked_add(
                128 + std::mem::size_of::<crate::game::resource::runtime::ResourceContext>() + 64,
            )?;
        }
        if self.resource_delivery().is_some() {
            bytes = bytes.checked_add(
                256 + std::mem::size_of::<crate::game::resource::runtime::ResourceDelivery>() + 64,
            )?;
        }
        if self
            .delivery
            .as_ref()
            .is_some_and(|ticket| ticket.resource.is_some())
        {
            bytes = bytes.checked_add(
                256 + std::mem::size_of::<crate::game::resource::runtime::ResourceDelivery>() + 64,
            )?;
        }
        if let Some(support) = self.regular_support() {
            bytes = bytes.checked_add(support.retained_extent()?)?;
        }
        if let Some(capture) = self.cast_damage() {
            bytes = bytes.checked_add(capture.retained_extent()?)?;
        }
        if let Some(cast) = &self.cast {
            bytes = bytes
                .checked_add(std::mem::size_of::<SkillCastExecutionSource>())?
                .checked_add(cast.skill_id.as_str().len())?;
        }
        if let Some(authored) = &self.authored {
            bytes = bytes.checked_add(std::mem::size_of_val(authored.as_ref()))?;
            for text in [
                authored.source_file.as_str(),
                authored.site.action.definition_id.as_str(),
                authored.site.action.action_id.as_str(),
                authored.site.effect_id.as_str(),
                authored.field_path.as_str(),
                authored.origin_definition_id.as_str(),
            ] {
                bytes = bytes.checked_add(text.len())?;
            }
            if let crate::game::ability::authoring::EffectOrigin::Added {
                source_definition_id,
            } = &authored.origin
            {
                bytes = bytes.checked_add(source_definition_id.as_str().len())?;
            }
        }
        Some(bytes)
    }

    pub(crate) fn admit_owner(
        &self,
        site: &'static str,
    ) -> Result<(), crate::game::behavior::GameError> {
        super::runtime_control::admit_retained_growth(0, self.retained_extent(), site)
    }

    pub(crate) fn observation(&self) -> Self {
        let mut value = self.clone();
        value.delivery = None;
        value.allows_finite_children = false;
        value.resource_capabilities = None;
        value
    }

    /// Provenance for a new independent actor. Unlike a released delivery,
    /// this owner must not borrow the creating action's execution capability.
    pub(crate) fn independent_origin(&self) -> Result<Self, crate::game::behavior::GameError> {
        let mut value = self.observation();
        value.action = None;
        value.use_control = false;
        if value.reinput().is_some() {
            value = value.without_reinput()?;
        }
        value.reaction = false;
        value.death_burst = false;
        if value.cast_damage().is_some() { value = value.with_cast_damage(None)?; }
        value.flow = None;
        Ok(value)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DamageSegmentFact {
    pub ordinal: u32,
    pub damage_type: super::damage::DamageType,
    pub raw_damage: u32,
    pub calculated_damage: u32,
    pub source: Arc<RuntimeFactSource>,
}
crate::authoring::clone_shape::struct_shape!(DamageSegmentFact {
    ordinal,
    damage_type,
    raw_damage,
    calculated_damage,
    source
});

#[derive(Debug, Clone, Serialize)]
pub struct ShieldAbsorptionFact {
    /// Local calculation component; absent preserves aggregate legacy receipts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_index: Option<u32>,
    pub activation: u64,
    pub absorbed: u32,
    pub remaining: u32,
    pub source: Arc<RuntimeFactSource>,
}
crate::authoring::clone_shape::struct_shape!(ShieldAbsorptionFact {
    component_index,    activation,
    absorbed,
    remaining,
    source
});

#[derive(Debug, Clone, Serialize)]
pub struct SurvivalPreventionFact {
    pub activation: u64,
    pub policy: crate::game::ability::sustained::SurvivalPolicy,
    pub source: Arc<RuntimeFactSource>,
}
crate::authoring::clone_shape::struct_shape!(SurvivalPreventionFact {
    activation,
    policy,
    source
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ReflectionOutcome {
    NotEligible,
    Missed,
    RoundedToZero,
    OwnerDefeated,
    AttackerUnavailable,
    Executed,
}
crate::authoring::clone_shape::copy_shape!(ReflectionOutcome);
#[derive(Debug, Clone, Serialize)]
pub struct PassiveGuardFact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge: Option<crate::game::ability::guard_charge::GuardChargeReceipt>,
    pub policy: crate::game::data::skill_fragment_data::ParryGuardDef,
    pub reduction_suppressed: bool,
    pub roll_percent: Option<u8>,
    pub reflected_base: u32,
    pub reflection: ReflectionOutcome,
    pub source: Arc<RuntimeFactSource>,
}
crate::authoring::clone_shape::struct_shape!(PassiveGuardFact {
    charge,
    policy,
    reduction_suppressed,
    roll_percent,
    reflected_base,
    reflection,
    source
});

fn is_zero_u32(value: &u32) -> bool { *value == 0 }

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProtectionReceipt {
    pub mitigated_damage: u32,
    /// HP damage rescheduled after shields; not absorbed or permanently prevented.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub deferred: u32,
    pub absorbed: u32,
    pub prevented: u32,
    #[serde(flatten)]
    pub(crate) details: Option<Box<ProtectionDetails>>,
}
crate::authoring::clone_shape::struct_shape!(ProtectionReceipt {
    mitigated_damage,
    deferred,
    absorbed,
    prevented,
    details
});

/// A charged equipment copy prevented part of a post-shield HP packet.
#[derive(Debug, Clone, Serialize)]
pub struct IdleHpGuardFact {
    pub source: RuntimeFactSource,
    pub charged_since_ms: u64,
    pub recharge_ms: u32,
    pub reduction_percent: u8,
    pub prevented: u32,
}
crate::authoring::clone_shape::struct_shape!(IdleHpGuardFact {
    source, charged_since_ms, recharge_ms, reduction_percent, prevented
});

/// Optional protection payload; ordinary hits allocate no detail record.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ProtectionDetails {
    pub shields: Vec<ShieldAbsorptionFact>,
    pub survival: Option<Box<SurvivalPreventionFact>>,
    pub deflection: Option<Box<DeflectionFact>>,
    pub passive_guard: Option<Box<PassiveGuardFact>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_hp_guard: Option<Box<IdleHpGuardFact>>,
}
crate::authoring::clone_shape::struct_shape!(ProtectionDetails {
    shields,
    survival,
    deflection,
    passive_guard,
    idle_hp_guard
});
impl std::ops::Deref for ProtectionReceipt {
    type Target = ProtectionDetails;
    fn deref(&self) -> &Self::Target {
        static EMPTY: ProtectionDetails = ProtectionDetails {
            shields: Vec::new(),
            survival: None,
            deflection: None,
            passive_guard: None,
            idle_hp_guard: None,
        };
        self.details.as_deref().unwrap_or(&EMPTY)
    }
}
impl ProtectionReceipt {
    pub(crate) fn primary_absorbed(&self) -> u32 {
        self.shields.iter().filter(|s| s.component_index.is_none_or(|i| i == 0))
            .fold(0, |sum: u32, s| sum.saturating_add(s.absorbed))
    }
    pub(crate) fn details_mut(
        &mut self,
    ) -> Result<&mut ProtectionDetails, crate::game::behavior::GameError> {
        if self.details.is_none() {
            super::runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<ProtectionDetails>() + 64),
                "protection_detail_storage",
            )?;
            self.details = Some(Box::default());
        }
        Ok(self.details.as_deref_mut().unwrap())
    }
    // Preserve the established ordinary-damage wire shape. Protected strikes
    // retain their complete receipt; a plain hit has no protection outcome.
    fn is_plain(&self) -> bool {
        self.deferred == 0
            && self.absorbed == 0
            && self.prevented == 0
            && self.shields.is_empty()
            && self.survival.is_none()
            && self.deflection.is_none()
            && self.passive_guard.is_none()
            && self.idle_hp_guard.is_none()
    }
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = self
            .shields
            .capacity()
            .checked_mul(std::mem::size_of::<ShieldAbsorptionFact>())?;
        if self.details.is_some() {
            bytes = bytes.checked_add(std::mem::size_of::<ProtectionDetails>() + 64)?;
        }
        for shield in &self.shields {
            bytes = bytes
                .checked_add(shield.source.retained_extent()?)?
                .checked_add(64)?;
        }
        if let Some(guard) = &self.idle_hp_guard {
            bytes = bytes.checked_add(guard.source.retained_extent()?)?
                .checked_add(std::mem::size_of::<IdleHpGuardFact>() + 64)?;
        }
        if let Some(guard) = &self.passive_guard {
            bytes = bytes
                .checked_add(guard.source.retained_extent()?)?
                .checked_add(std::mem::size_of::<PassiveGuardFact>() + 64)?;
        }
        if let Some(deflection) = &self.deflection {
            bytes = bytes
                .checked_add(deflection.source.retained_extent()?)?
                .checked_add(std::mem::size_of::<DeflectionFact>() + 64)?;
        }
        if let Some(survival) = &self.survival {
            bytes = bytes
                .checked_add(survival.source.retained_extent()?)?
                .checked_add(std::mem::size_of::<SurvivalPreventionFact>() + 64)?;
        }
        Some(bytes)
    }
}

/// One actual HP mutation. Components are calculations, not invented shares of
/// the effective HP loss when their combined result overkills a target.
#[derive(Debug, Clone, Serialize)]
pub struct DamageFact {
    #[serde(skip)]
    pub(crate) retained_extent: usize,
    pub ordinal: u64,
    pub time_ms: u64,
    pub strike: StrikeOrdinal,
    pub target: UnitInstanceId,
    pub source: Arc<RuntimeFactSource>,
    pub requested_damage: u32,
    #[serde(skip_serializing_if = "ProtectionReceipt::is_plain")]
    pub protection: ProtectionReceipt,
    pub hp_before: u32,
    pub hp_after: u32,
    pub segments: Vec<DamageSegmentFact>,
    pub modifier_sources: Vec<RuntimeFactSource>,
    pub unattributed_modifiers: super::damage::DamageModifiers,
}
impl DamageFact {
    pub fn actual_hp_loss(&self) -> u32 {
        self.hp_before.saturating_sub(self.hp_after)
    }
}
crate::authoring::clone_shape::struct_shape!(DamageFact {
    retained_extent,
    ordinal,
    time_ms,
    strike,
    target,
    source,
    requested_damage,
    protection,
    hp_before,
    hp_after,
    segments,
    modifier_sources,
    unattributed_modifiers
});

#[derive(Debug, Clone, Serialize)]
pub struct HealFact {
    #[serde(skip)]
    pub(crate) retained_extent: usize,
    pub ordinal: u64,
    pub time_ms: u64,
    pub target: UnitInstanceId,
    pub source: RuntimeFactSource,
    pub requested_delta: i64,
    pub hp_before: u32,
    pub hp_after: u32,
    pub reason: super::event_log::HpChangeReason,
}
impl HealFact {
    pub fn actual_delta(&self) -> i64 {
        i64::from(self.hp_after) - i64::from(self.hp_before)
    }
}
crate::authoring::clone_shape::struct_shape!(HealFact {
    retained_extent,
    ordinal,
    time_ms,
    target,
    source,
    requested_delta,
    hp_before,
    hp_after,
    reason
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FactResource {
    Health,
    Resonance,
    TacticalPoints,
    Typed {
        id: Arc<crate::game::resource::ResourceId>,
        lifetime: crate::game::resource::ResourceLifetime,
        scope: u64,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum ResourceOwner {
    Unit(UnitInstanceId),
    Side(crate::game::enums::Side),
    Employee(Uuid),
    EquipmentCopy(Uuid),
    FragmentCopy(Uuid),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CostReason {
    AuthoredHealth,
    Corrosion,
    AuthoredResource,
    CastReset,
    TacticalDeploy,
    TacticalRelocation,
    TacticalSwap,
}
crate::authoring::clone_shape::copy_shape!(
    ResourceOwner,
    CostReason,
    super::event_log::HpChangeReason
);
crate::authoring::clone_shape::enum_shape!(FactResource {
    units: [Health, Resonance, TacticalPoints],
    tuples: [],
    records: [Typed {
        id,
        lifetime,
        scope
    }]
});

#[derive(Debug, Clone, Serialize)]
pub struct CostFact {
    #[serde(skip)]
    pub(crate) retained_extent: usize,
    pub ordinal: u64,
    pub time_ms: u64,
    pub owner: ResourceOwner,
    pub resource: FactResource,
    pub reason: CostReason,
    pub source: RuntimeFactSource,
    pub requested_delta: i64,
    pub before: u32,
    pub after: u32,
}
impl CostFact {
    pub fn actual_spent(&self) -> i64 {
        i64::from(self.before) - i64::from(self.after)
    }
}
crate::authoring::clone_shape::struct_shape!(CostFact {
    retained_extent,
    ordinal,
    time_ms,
    owner,
    resource,
    reason,
    source,
    requested_delta,
    before,
    after
});

#[derive(Debug, Clone, Serialize)]
pub struct ResourceChangeFact {
    #[serde(skip)]
    pub(crate) retained_extent: usize,
    pub ordinal: u64,
    pub time_ms: u64,
    pub owner: ResourceOwner,
    pub resource: FactResource,
    pub source: RuntimeFactSource,
    pub requested_delta: i64,
    pub before: u32,
    pub after: u32,
}
impl ResourceChangeFact {
    pub fn actual_delta(&self) -> i64 {
        i64::from(self.after) - i64::from(self.before)
    }
}
crate::authoring::clone_shape::struct_shape!(ResourceChangeFact {
    retained_extent,
    ordinal,
    time_ms,
    owner,
    resource,
    source,
    requested_delta,
    before,
    after
});

#[derive(Debug, Clone, Serialize)]
pub struct PredeathBuffSnapshot {
    pub buff_id: super::buffs::BuffId,
    pub caster: UnitInstanceId,
    pub stacks: u8,
    pub expires_at_ms: u64,
    pub latest_application_source: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(PredeathBuffSnapshot {
    buff_id,
    caster,
    stacks,
    expires_at_ms,
    latest_application_source
});

#[derive(Debug, Clone, Serialize)]
pub struct PredeathSnapshot {
    pub victim: UnitInstanceId,
    pub owner: crate::game::enums::Side,
    /// The blocking authority at death preparation, before death cleanup can
    /// release the engagement. This is independent of the lethal attacker.
    pub blocker: Option<UnitInstanceId>,
    pub hp_before_lethal: u32,
    pub position: Option<crate::game::resources::Position>,
    pub buffs: Vec<PredeathBuffSnapshot>,
}
crate::authoring::clone_shape::struct_shape!(PredeathSnapshot {
    victim,
    owner,
    blocker,
    hp_before_lethal,
    position,
    buffs
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeathCause {
    Transformation,
    Ordinary,
    Corrosion,
}
crate::authoring::clone_shape::copy_shape!(DeathCause);

#[derive(Debug, Clone, Serialize)]
pub struct DeathFact {
    pub cause: DeathCause,
    #[serde(skip)]
    pub(crate) retained_extent: usize,
    pub ordinal: u64,
    pub time_ms: u64,
    pub source: RuntimeFactSource,
    pub predeath: PredeathSnapshot,
    pub lethal_damage: Option<Arc<DamageFact>>,
}
crate::authoring::clone_shape::struct_shape!(DeathFact {
    cause,
    retained_extent,
    ordinal,
    time_ms,
    source,
    predeath,
    lethal_damage
});

/// Fixed witness cardinality; no attempt-long mutation history is retained.
#[derive(Debug, Clone, Default, Serialize)]
pub struct FactSummary {
    pub damage_mutations: u64,
    pub damage_strikes: u64,
    pub damaging_strikes: u64,
    pub damage_hp_lost: u64,
    pub heal_mutations: u64,
    pub healing_delta: i64,
    pub resource_change_mutations: u64,
    pub resource_change_delta: i64,
    pub cost_mutations: u64,
    pub resource_spent: i64,
    pub deaths: u64,
    pub latest_damage: Option<Arc<DamageFact>>,
    pub latest_nonzero_damage: Option<Arc<DamageFact>>,
    pub latest_heal: Option<Arc<HealFact>>,
    pub latest_nonzero_heal: Option<Arc<HealFact>>,
    pub latest_resource_change: Option<Arc<ResourceChangeFact>>,
    pub latest_cost: Option<Arc<CostFact>>,
    pub latest_nonzero_cost: Option<Arc<CostFact>>,
    pub latest_death: Option<Arc<DeathFact>>,
}
crate::authoring::clone_shape::struct_shape!(FactSummary {
    damage_mutations,
    damage_strikes,
    damaging_strikes,
    damage_hp_lost,
    heal_mutations,
    healing_delta,
    resource_change_mutations,
    resource_change_delta,
    cost_mutations,
    resource_spent,
    deaths,
    latest_damage,
    latest_nonzero_damage,
    latest_heal,
    latest_nonzero_heal,
    latest_resource_change,
    latest_cost,
    latest_nonzero_cost,
    latest_death
});

#[derive(Debug, Clone, Default, Serialize)]
pub struct ActorFactSummary {
    pub latest_settled_action: Option<Arc<ActionSnapshot>>,
    pub caused: FactSummary,
    pub received: FactSummary,
}
crate::authoring::clone_shape::struct_shape!(ActorFactSummary {
    latest_settled_action,
    caused,
    received
});

/// Provider component totals do not assign a share of combined overkill HP loss.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ProviderFactSummary {
    pub actor: Option<UnitInstanceId>,
    pub damage_segments: u64,
    pub calculated_damage: u64,
    pub modifier_contributions: u64,
    pub latest_damage: Option<Arc<DamageFact>>,
    pub latest_heal: Option<Arc<HealFact>>,
    pub latest_resource_change: Option<Arc<ResourceChangeFact>>,
    pub latest_cost: Option<Arc<CostFact>>,
    pub latest_death: Option<Arc<DeathFact>>,
}
crate::authoring::clone_shape::struct_shape!(ProviderFactSummary {
    actor,
    damage_segments,
    calculated_damage,
    modifier_contributions,
    latest_damage,
    latest_heal,
    latest_resource_change,
    latest_cost,
    latest_death
});

#[derive(Debug, Clone, Serialize)]
pub struct ActionSnapshot {
    pub ordinal: ActionOrdinal,
    pub actor: UnitInstanceId,
    pub skill_id: Option<Arc<crate::game::ability::SkillId>>,
    pub source: Arc<RuntimeFactSource>,
    pub accepted_at_ms: u64,
    pub released_at_ms: Option<u64>,
    pub terminal: Option<ActionTerminal>,
    pub effects_settled_at_ms: Option<u64>,
    pub pending_control_steps: usize,
    pub outstanding_deliveries: usize,
    pub facts: FactSummary,
}
crate::authoring::clone_shape::struct_shape!(ActionSnapshot {
    ordinal,
    actor,
    skill_id,
    source,
    accepted_at_ms,
    released_at_ms,
    terminal,
    effects_settled_at_ms,
    pending_control_steps,
    outstanding_deliveries,
    facts
});

#[derive(Debug, Clone, Serialize)]
pub struct SettledActionReceipt {
    pub action: Arc<ActionSnapshot>,
}
crate::authoring::clone_shape::struct_shape!(SettledActionReceipt { action });

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct RuntimeFactUsage {
    pub active_actions: usize,
    pub public_result_reservations: usize,
    pub completed_results: usize,
    pub outstanding_deliveries: usize,
    pub actor_summaries: usize,
    pub provider_summaries: usize,
    pub retained_bytes_bound: usize,
    pub accepted_actions: u64,
    pub settled_actions: u64,
}
crate::authoring::clone_shape::copy_shape!(RuntimeFactUsage);

#[derive(Debug, Clone, Serialize)]
pub struct ActorFactObservation {
    pub actor: UnitInstanceId,
    pub summary: ActorFactSummary,
}
crate::authoring::clone_shape::struct_shape!(ActorFactObservation { actor, summary });
#[derive(Debug, Clone, Serialize)]
pub struct ProviderFactObservation {
    pub provider: EquipmentSourceSnapshot,
    pub summary: ProviderFactSummary,
}
crate::authoring::clone_shape::struct_shape!(ProviderFactObservation { provider, summary });

/// A bounded historical value. It carries no mutable engine authority and is
/// not a saved active battle or a way to reconstruct a valid action handle.
#[derive(Debug, Clone, Serialize)]
pub struct RuntimeFactSnapshot {
    #[serde(skip)]
    pub(crate) retained_extent: usize,
    pub summary: FactSummary,
    pub latest_settled_action: Option<Arc<ActionSnapshot>>,
    pub active_actions: Vec<ActionSnapshot>,
    pub actors: Vec<ActorFactObservation>,
    pub providers: Vec<ProviderFactObservation>,
    pub usage: RuntimeFactUsage,
}
crate::authoring::clone_shape::struct_shape!(RuntimeFactSnapshot {
    retained_extent,
    summary,
    latest_settled_action,
    active_actions,
    actors,
    providers,
    usage
});

/// Borrowed observation of either the live authority or its immutable terminal
/// value. Serialization visits ordered bounded rows without building a history.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeFactView<'a> {
    pub(crate) live: Option<&'a super::core::runtime_facts::RuntimeFactState>,
    pub(crate) snapshot: Option<&'a RuntimeFactSnapshot>,
}
impl RuntimeFactSnapshot {
    pub fn view(&self) -> RuntimeFactView<'_> {
        RuntimeFactView {
            live: None,
            snapshot: Some(self),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DeflectionKind {
    Parry,
    Invulnerability,
}
crate::authoring::clone_shape::copy_shape!(DeflectionKind);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CounterOutcome {
    NotRequested,
    Executed,
    AttackerUnavailable,
}
crate::authoring::clone_shape::copy_shape!(CounterOutcome);
#[derive(Debug, Clone, Serialize)]
pub struct DeflectionFact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard_charge: Option<crate::game::ability::guard_charge::GuardChargeReceipt>,
    pub activation: u64,
    pub kind: DeflectionKind,
    pub counter: CounterOutcome,
    pub source: Arc<RuntimeFactSource>,
}
crate::authoring::clone_shape::struct_shape!(DeflectionFact {
    guard_charge,
    activation,
    kind,
    counter,
    source
});

#[cfg(test)]
mod u07_source_storage_tests {
    use super::*;
    use crate::game::{
        ability::{
            authoring::{ActionSite, EffectOrigin, EffectSite, EffectSource},
            SkillId,
        },
        battle::{
            event_log::BattleEventCause,
            runtime_control::{self as native, RuntimeFactLimits, RuntimeFaultCode},
        },
        behavior::GameError,
        data::{
            independent_object_data::IndependentObjectId, skill_fragment_data::SkillFragmentId,
        },
        skill_fragment::FragmentSourceSnapshot,
    };

    fn fixture() -> RuntimeFactSource {
        let cast = Arc::new(SkillCastExecutionSource {
            skill_id: SkillId::from("own"),
            caster_instance_id: UnitInstanceId(Uuid::from_u128(1)),
            cast_seq: 3,
            ability_event_seq: 4,
            cause: BattleEventCause::default(),
            proc_origin: None,
        });
        let action = ActionHandle {
            validity: Arc::new(ActionToken {
                attempt: Arc::new(AttemptToken),
                ordinal: ActionOrdinal(7),
            }),
        };
        RuntimeFactSource {
            extended_observation: RuntimeFactSource::extended_observation(
                Some(Arc::new(IndependentObjectOriginSnapshot {
                    object_id: UnitInstanceId(Uuid::from_u128(1)),
                    definition_id: IndependentObjectId::new("friend"),
                    creator: UnitInstanceId(Uuid::from_u128(2)),
                    creation_action: Some(ActionOrdinal(5)),
                    creation_cast: None,
                    creation_fragment: None,
                    creation_equipment: None,
                    creation_authored: None,
                })),
                Some(Arc::new(CommittedReinputContext {
                    activation: 8,
                    committed_at_ms: 9,
                    remaining_ms: 10,
                })),
            )
            .unwrap(),
            use_control: true,
            reaction: true,
            flow: Some(Arc::new(FlowOccurrence {
                node_index: 2,
                occurrence: 3,
            })),
            actor: Some(UnitInstanceId(Uuid::from_u128(1))),
            equipment: Some(EquipmentSourceSnapshot {
                definition_uuid: Uuid::from_u128(2),
                runtime_item_instance_id: Uuid::from_u128(3),
                persistent_copy_uuid: Some(Uuid::from_u128(4)),
            }),
            fragment: Some(Arc::new(FragmentSourceSnapshot {
                copy_uuid: Uuid::from_u128(5),
                definition_id: SkillFragmentId::from("f"),
                form_id: "base".into(),
                skill_id: SkillId::from("own"),
            })),
            cast: Some(cast),
            authored: Some(Arc::new(EffectSource {
                source_file: "s.ron".into(),
                site: EffectSite {
                    action: ActionSite {
                        definition_id: SkillId::from("own"),
                        action_id: "a".into(),
                    },
                    effect_id: "e".into(),
                },
                field_path: "amount".into(),
                origin_definition_id: SkillId::from("own"),
                origin: EffectOrigin::Original,
            })),
            delivery: Some(Arc::new(DeliveryTicket {
                action: action.clone(),
                ordinal: DeliveryOrdinal(6),
                resource: None,
            })),
            action: Some(action),
            allows_finite_children: true,
            ..Default::default()
        }
    }

    #[test]
    fn u07_source_layout_and_all_optional_json_combinations_preserve_literal_contract() {
        assert_eq!(std::mem::size_of::<RuntimeFactSource>(), 136);
        assert_eq!(
            serde_json::to_string(&RuntimeFactSource::default()).unwrap(),
            r#"{"actor":null,"equipment":null,"cast":null,"authored":null,"action_ordinal":null}"#
        );
        let base = fixture();
        let literals = [
            (
                "independent_object",
                r#"{"object_id":"00000000-0000-0000-0000-000000000001","definition_id":"friend","creator":"00000000-0000-0000-0000-000000000002","creation_action":5,"creation_cast":null,"creation_fragment":null,"creation_equipment":null,"creation_authored":null}"#,
            ),
            ("reaction", "true"),
            (
                "reinput",
                r#"{"activation":8,"committed_at_ms":9,"remaining_ms":10}"#,
            ),
            ("flow", r#"{"node_index":2,"occurrence":3}"#),
            ("actor", r#""00000000-0000-0000-0000-000000000001""#),
            (
                "equipment",
                r#"{"definition_uuid":"00000000-0000-0000-0000-000000000002","runtime_item_instance_id":"00000000-0000-0000-0000-000000000003","persistent_copy_uuid":"00000000-0000-0000-0000-000000000004"}"#,
            ),
            (
                "fragment",
                r#"{"copy_uuid":"00000000-0000-0000-0000-000000000005","definition_id":"f","form_id":"base","skill_id":"own"}"#,
            ),
            (
                "cast",
                r#"{"skill_id":"own","caster_instance_id":"00000000-0000-0000-0000-000000000001","cast_seq":3,"proc_origin":null}"#,
            ),
            (
                "authored",
                r#"{"source_file":"s.ron","site":{"action":{"definition_id":"own","action_id":"a"},"effect_id":"e"},"field_path":"amount","origin_definition_id":"own","origin":"Original"}"#,
            ),
            ("action_ordinal", "7"),
        ];
        for mask in 0..1024u16 {
            let enabled = |bit: usize| mask & (1 << bit) != 0;
            let mut source = base.clone();
            source.extended_observation = RuntimeFactSource::extended_observation(
                enabled(0).then(|| base.independent_object().unwrap().clone()),
                enabled(2).then(|| base.reinput().unwrap().clone()),
            )
            .unwrap();
            source.reaction = enabled(1);
            if !enabled(3) {
                source.flow = None;
            }
            if !enabled(4) {
                source.actor = None;
            }
            if !enabled(5) {
                source.equipment = None;
            }
            if !enabled(6) {
                source.fragment = None;
            }
            if !enabled(7) {
                source.cast = None;
            }
            if !enabled(8) {
                source.authored = None;
            }
            if !enabled(9) {
                source.action = None;
            }
            let expected: Vec<_> = literals
                .iter()
                .enumerate()
                .filter_map(|(bit, (name, value))| {
                    if enabled(bit) {
                        Some(format!("\"{name}\":{value}"))
                    } else if matches!(bit, 4 | 5 | 7 | 8 | 9) {
                        Some(format!("\"{name}\":null"))
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(
                serde_json::to_string(&source).unwrap(),
                format!("{{{}}}", expected.join(",")),
                "optional mask {mask}"
            );
        }
    }

    #[test]
    fn runtime_facts_death_burst_permission_reuses_equipment_copy_and_clears_for_new_actor() {
        let mut source = fixture();
        source.reaction = true;
        source.death_burst = true;
        let copy = Uuid::from_u128(4);
        assert_eq!(source.death_burst_copy(), Some(copy));
        assert_eq!(serde_json::to_value(&source).unwrap()["death_burst"], copy.to_string());
        let cloned = crate::authoring::clone_shape::controlled_clone(&source).unwrap();
        assert_eq!(cloned.death_burst_copy(), Some(copy));
        assert_eq!(source.without_reinput().unwrap().death_burst_copy(), Some(copy));
        let independent = source.independent_origin().unwrap();
        assert_eq!(independent.equipment, source.equipment);
        assert_eq!(independent.death_burst_copy(), None);
        assert!(!independent.reaction);
        assert!(serde_json::to_value(&independent).unwrap().get("death_burst").is_none());
    }

    #[test]
    fn u07_source_origin_keeps_creation_observations_and_drops_all_credentials() {
        let mut source = fixture();
        let mut resources = crate::game::resource::runtime::RuntimeResources::new(Arc::new(
            crate::game::resource::ResourceCatalog { resources: vec![] },
        ))
        .unwrap();
        resources
            .accept(source.action.as_ref().unwrap(), None)
            .unwrap();
        source.resource_capabilities = RuntimeFactSource::resource_capabilities(
            resources.context(source.action.as_ref().unwrap()).cloned(),
            None,
        )
        .unwrap();
        let mut origin = source.independent_object().unwrap().as_ref().clone();
        origin.creation_cast = source.cast.clone();
        origin.creation_fragment = source.fragment.clone();
        origin.creation_equipment = source.equipment;
        origin.creation_authored = source.authored.clone();
        source.extended_observation = RuntimeFactSource::extended_observation(
            Some(Arc::new(origin)),
            source.reinput().cloned(),
        )
        .unwrap();
        let original = source.clone();
        let original_json = serde_json::to_string(&source).unwrap();
        let child = source.independent_origin().unwrap();
        assert!(Arc::ptr_eq(
            source.independent_object().unwrap(),
            child.independent_object().unwrap()
        ));
        assert_eq!(
            child.independent_object().unwrap().creation_cast,
            source.cast
        );
        assert_eq!(
            child.independent_object().unwrap().creation_fragment,
            source.fragment
        );
        assert_eq!(
            child.independent_object().unwrap().creation_equipment,
            source.equipment
        );
        assert_eq!(
            child.independent_object().unwrap().creation_authored,
            source.authored
        );
        assert!(
            child.action.is_none()
                && child.delivery.is_none()
                && child.resource_capabilities.is_none()
        );
        assert!(
            !child.use_control
                && !child.allows_finite_children
                && !child.reaction
                && child.flow.is_none()
                && child.reinput().is_none()
        );
        assert_eq!(source, original);
        assert_eq!(serde_json::to_string(&source).unwrap(), original_json);
        assert!(Arc::ptr_eq(
            source.extended_observation.as_ref().unwrap(),
            original.extended_observation.as_ref().unwrap()
        ));
        let child_again = child.independent_origin().unwrap();
        assert!(Arc::ptr_eq(
            child.extended_observation.as_ref().unwrap(),
            child_again.extended_observation.as_ref().unwrap()
        ));
        let reinput_only = RuntimeFactSource {
            extended_observation: RuntimeFactSource::extended_observation(
                None,
                source.reinput().cloned(),
            )
            .unwrap(),
            ..Default::default()
        };
        assert!(reinput_only
            .independent_origin()
            .unwrap()
            .extended_observation
            .is_none());
    }

    #[test]
    fn native_u07_source_wrapper_exact_admission_refuses_before_replacing_origin_or_clone() {
        let source = fixture();
        let original = source.clone();
        let before = serde_json::to_string(&source).unwrap();
        let wrapper_bytes = std::mem::size_of::<ExtendedSourceObservation>() + 64;
        let (child, admissions) = native::observe_retained_admissions(|| {
            native::run(
                RuntimeFactLimits {
                    max_retained_bytes: wrapper_bytes,
                    ..Default::default()
                },
                || source.independent_origin(),
            )
        });
        assert!(child.unwrap().reinput().is_none());
        assert_eq!(
            admissions,
            vec![("extended_source_observation", 0, wrapper_bytes)]
        );
        let source_owners = Arc::strong_count(source.extended_observation.as_ref().unwrap());
        let origin_owners = Arc::strong_count(source.independent_object().unwrap());
        let failure = native::run(
            RuntimeFactLimits {
                max_retained_bytes: wrapper_bytes - 1,
                ..Default::default()
            },
            || source.independent_origin(),
        )
        .unwrap_err();
        assert!(
            matches!(failure, GameError::BattleRuntimeFault(native::BattleRuntimeFault { code: RuntimeFaultCode::RetainedByteLimit, site: "extended_source_observation", requested: Some(n), .. }) if n == wrapper_bytes as u64)
        );
        assert_eq!(source, original);
        assert_eq!(serde_json::to_string(&source).unwrap(), before);
        assert_eq!(
            Arc::strong_count(source.extended_observation.as_ref().unwrap()),
            source_owners
        );
        assert_eq!(
            Arc::strong_count(source.independent_object().unwrap()),
            origin_owners
        );
        assert!(Arc::ptr_eq(
            source.extended_observation.as_ref().unwrap(),
            original.extended_observation.as_ref().unwrap()
        ));
        let empty = native::run(
            RuntimeFactLimits {
                max_retained_bytes: 1,
                ..Default::default()
            },
            || RuntimeFactSource::extended_observation(None, None),
        )
        .unwrap();
        assert!(empty.is_none());
    }

    #[test]
    fn native_u07_source_retained_payload_is_priced_while_clone_shares_wrapper() {
        use crate::authoring::clone_shape::controlled_clone;
        let source = fixture();
        let empty = RuntimeFactSource::default().retained_extent().unwrap();
        let cold = RuntimeFactSource {
            extended_observation: source.extended_observation.clone(),
            ..Default::default()
        };
        let payload = std::mem::size_of::<ExtendedSourceObservation>()
            + 64
            + source
                .independent_object()
                .unwrap()
                .retained_extent()
                .unwrap()
            + 64
            + std::mem::size_of::<CommittedReinputContext>()
            + 64;
        assert_eq!(cold.retained_extent().unwrap(), empty + payload);
        let retained = cold.retained_extent().unwrap();
        native::run(
            RuntimeFactLimits {
                max_retained_bytes: retained,
                ..Default::default()
            },
            || cold.admit_owner("u07_source_owner"),
        )
        .unwrap();
        let failure = native::run(
            RuntimeFactLimits {
                max_retained_bytes: retained - 1,
                ..Default::default()
            },
            || cold.admit_owner("u07_source_owner"),
        )
        .unwrap_err();
        assert!(
            matches!(failure, GameError::BattleRuntimeFault(native::BattleRuntimeFault { code: RuntimeFaultCode::RetainedByteLimit, site: "u07_source_owner", requested: Some(n), .. }) if n == retained as u64)
        );
        let (copy, admissions) = native::observe_retained_admissions(|| {
            native::run(Default::default(), || controlled_clone(&source))
        });
        let copy = copy.unwrap();
        assert!(
            admissions.is_empty(),
            "Arc clone allocates no wrapper or payload"
        );
        assert!(Arc::ptr_eq(
            source.extended_observation.as_ref().unwrap(),
            copy.extended_observation.as_ref().unwrap()
        ));
        assert_eq!(copy.retained_extent(), source.retained_extent());
    }
}

#[cfg(test)]
mod cold_observation_tests {
    use super::*;
    #[test]
    fn runtime_facts_guard_status_mobile_lineage_survives_numeric_clear_and_independent_origin() {
        use crate::game::{ability::{guard_charge::GuardReleaseOrigin,mobile_effect::MobileMotion},data::content_pack::{ContentIdentity,ContentContract},relic::numeric::RelicNumericTarget};
        let actor=UnitInstanceId(Uuid::from_u128(41));
        let source=RuntimeFactSource {extended_observation:RuntimeFactSource::extended_observation(None,Some(Arc::new(CommittedReinputContext{activation:1,committed_at_ms:0,remaining_ms:5}))).unwrap(),..Default::default()};
        let source=source.with_status_transition(StatusTransitionOrigin{previous:vec![]}).unwrap()
            .with_mobile_effect(MobileEffectOrigin{motion:MobileMotion::ChaseDelay,death_origin:None,seed_origin:None,activation:7,owner:actor,copy:Uuid::from_u128(42),definition:"mobile".into()}).unwrap()
            .with_guard_release(GuardReleaseOrigin{relic_copy:Uuid::from_u128(43),relic_definition:"charge-relic".into(),definition:"charge".into(),consumed:30,added_power:60,primary:true}).unwrap();
        let binding=Arc::new(crate::game::battle::core::relic_numeric::RelicNumericBinding {
            generation:Arc::new(ContentIdentity{contract:ContentContract::current(),semantic_sha256:"fixture".into()}),employee_uuid:Uuid::from_u128(44),actor,target:RelicNumericTarget::BasicAttackDamage{actor,attack_epoch:1},action_ordinal:None,equipment:None,fragment:None,damage_type:crate::game::battle::damage::DamageType::Magic,basic_profile_fingerprint:None,contributions:vec![],
        });
        let bound=source.with_relic_numeric(binding).unwrap();
        for result in [bound.with_exact_relic_numeric(None).unwrap(),source.without_reinput().unwrap(),source.independent_origin().unwrap()] {
            assert!(result.relic_numeric().is_none());
            assert_eq!(result.status_transition(),source.status_transition());
            assert_eq!(result.mobile_effect(),source.mobile_effect());
            assert_eq!(result.guard_release(),source.guard_release());
        }
        let only=RuntimeFactSource{extended_observation:RuntimeFactSource::extended_observation(None,Some(Arc::new(CommittedReinputContext{activation:2,committed_at_ms:0,remaining_ms:5}))).unwrap(),..Default::default()}.with_guard_release(source.guard_release().unwrap().as_ref().clone()).unwrap();
        assert_eq!(only.without_reinput().unwrap().guard_release(),source.guard_release());
    }
}
