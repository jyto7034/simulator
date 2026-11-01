//! Mandatory, bounded-cardinality observations from actual mutation producers.
//! This is neither an event-log history nor optional trace storage.

use serde::{Serialize, Serializer};
use std::sync::Arc;
use uuid::Uuid;

use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        battle::{
            buffs::{BuffId, BuffKind},
            event_log::HpChangeReason,
            ids::UnitInstanceId,
            skill_source::SkillEffectExecutionSource,
        },
        behavior::GameError,
        stats::{StatModifier, UnitStats},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct ObservationKey {
    pub entry_ordinal: u64,
    pub battle_uuid: Uuid,
}
crate::authoring::clone_shape::copy_shape!(ObservationKey);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BuffApplicationState {
    pub stacks: u8,
    pub expires_at_ms: u64,
    pub next_tick_ms: Option<u64>,
}
crate::authoring::clone_shape::copy_shape!(BuffApplicationState);
impl From<&super::types::ActiveBuff> for BuffApplicationState {
    fn from(value: &super::types::ActiveBuff) -> Self {
        Self {
            stacks: value.stacks,
            expires_at_ms: value.expires_at_ms,
            next_tick_ms: value.next_tick_ms,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(tag = "channel", rename_all = "snake_case")]
pub enum AppliedChange {
    HitCounter { outcome: crate::game::ability::sustained::HitCounterOutcome },
    /// An actual accepted Buff application/reapplication, after scheduling its cadence.
    Buff {
        buff_id: BuffId,
        kind: BuffKind,
        before: Option<BuffApplicationState>,
        after: BuffApplicationState,
    },
    /// Actual ModifyResonance command mutation; excludes passive gain/cast reset.
    Resonance { before: u32, after: u32, max: u32 },
    Hp {
        before: u32,
        after: u32,
        reason: HpChangeReason,
    },
    Stats {
        modifier: StatModifier,
        before: UnitStats,
        after: UnitStats,
    },
    /// An accepted finite sustained activation or its actual removal. This is
    /// a lifecycle witness, not an HP assignment or a history of all effects.
    Sustained {
        activation: u64,
        ended: Option<crate::game::ability::sustained::SustainedEndReason>,
        expires_at_ms: u64,
        target_state: Option<crate::game::ability::sustained::TargetStateProperties>,
        basic_speed_reduction_percent: Option<u32>,
        damage_taken_increase_percent: Option<u32>,
        before: Option<UnitStats>,
        after: Option<UnitStats>,
    },
}
crate::authoring::clone_shape::copy_shape!(AppliedChange);

impl AppliedChange {
    pub fn hp_delta(&self) -> i64 {
        let (before, after) = match self {
            Self::Resonance { .. } | Self::Buff { .. } | Self::Sustained { .. } | Self::HitCounter { .. } => return 0,
            Self::Hp { before, after, .. } => (*before, *after),
            Self::Stats { before, after, .. } => (before.current_health, after.current_health),
        };
        i64::from(after) - i64::from(before)
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApplicationWitness {
    pub ordinal: u64,
    pub time_ms: u64,
    pub source_unit: Option<UnitInstanceId>,
    pub target_unit: UnitInstanceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fragment: Option<Arc<crate::game::skill_fragment::FragmentSourceSnapshot>>,
    pub change: AppliedChange,
    #[serde(serialize_with = "serialize_source")]
    pub source: Option<Arc<SkillEffectExecutionSource>>,
    /// Direct authored callbacks have no skill cast. Preserve their real
    /// definition and equipment attribution without inventing a cast event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authored: Option<Arc<crate::game::ability::authoring::EffectSource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area_source: Option<Arc<super::area_observation::AreaEffectSource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment: Option<crate::game::battle::runtime_facts::EquipmentSourceSnapshot>,
}
crate::authoring::clone_shape::struct_shape!(ApplicationWitness {
    ordinal,
    time_ms,
    source_unit,
    target_unit,
    fragment,
    change,
    source,
    authored,
    area_source,
    equipment
});

impl ApplicationWitness {
    /// Fixed shape plus every owned string visited by serialize_source. Copy
    /// bounds reject an unnoticed dynamic child in the other serialized fields.
    pub(crate) fn trace_scan_work(&self) -> Option<u64> {
        use crate::game::ability::authoring::{ActionSite, EffectOrigin, EffectSite, EffectSource};
        fn scalar<T: Copy>(_: &T) {}
        let Self {
            ordinal,
            time_ms,
            source_unit,
            target_unit,
            fragment,
            change,
            source,
            authored,
            area_source,
            equipment,
        } = self;
        scalar(ordinal);
        scalar(time_ms);
        scalar(source_unit);
        scalar(target_unit);
        scalar(change);
        scalar(equipment);
        let mut work = 256_u64;
        if let Some(area) = area_source { work = work.checked_add(area.retained_extent()? as u64)?; }
        if let Some(authored) = authored {
            let fact = crate::game::battle::runtime_facts::RuntimeFactSource {
                authored: Some(Arc::clone(authored)), ..Default::default()
            };
            work = work.checked_add(fact.retained_extent()? as u64)?;
        }
        if let Some(fragment) = fragment {
            work = work.checked_add(fragment.retained_extent()? as u64)?;
        }
        if let Some(source) = source {
            let SkillEffectExecutionSource { cast, authored } = source.as_ref();
            let EffectSource {
                source_file,
                site,
                field_path,
                origin_definition_id,
                origin,
            } = authored.as_ref();
            let EffectSite { action, effect_id } = site;
            let ActionSite {
                definition_id,
                action_id,
            } = action;
            let crate::game::battle::skill_source::SkillCastExecutionSource {
                skill_id,
                caster_instance_id,
                cast_seq,
                ability_event_seq,
                cause,
                proc_origin,
            } = cast.as_ref();
            scalar(caster_instance_id);
            scalar(cast_seq);
            scalar(ability_event_seq);
            scalar(cause);
            if let Some(origin) = proc_origin {
                let crate::game::battle::skill_source::SkillProcOrigin {
                    event_seq,
                    activation_source,
                    binding_index,
                } = origin;
                scalar(event_seq);
                scalar(activation_source);
                scalar(binding_index);
            }
            for text in [
                source_file.as_str(),
                definition_id.as_str(),
                action_id.as_str(),
                effect_id.as_str(),
                field_path.as_str(),
                origin_definition_id.as_str(),
                skill_id.as_str(),
            ] {
                work = work.checked_add(text.len() as u64)?;
            }
            match origin {
                EffectOrigin::Original => {}
                EffectOrigin::Added {
                    source_definition_id,
                } => {
                    work = work.checked_add(source_definition_id.as_str().len() as u64)?;
                }
            }
        }
        Some(work)
    }
}

fn serialize_source<S: Serializer>(
    source: &Option<Arc<SkillEffectExecutionSource>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use crate::game::battle::{cooldown::CooldownSource, event_log::BattleEventCause};
    #[derive(Serialize)]
    struct Proc<'a> {
        event_seq: u64,
        activation_source: &'a CooldownSource,
        binding_index: usize,
    }
    #[derive(Serialize)]
    struct Source<'a> {
        authored: &'a crate::game::ability::authoring::EffectSource,
        skill_id: &'a crate::game::ability::SkillId,
        caster_instance_id: UnitInstanceId,
        cast_seq: u64,
        ability_event_seq: u64,
        cause: &'a BattleEventCause,
        proc_origin: Option<Proc<'a>>,
    }
    source
        .as_ref()
        .map(|source| Source {
            authored: &source.authored,
            skill_id: &source.cast.skill_id,
            caster_instance_id: source.cast.caster_instance_id,
            cast_seq: source.cast.cast_seq,
            ability_event_seq: source.cast.ability_event_seq,
            cause: &source.cast.cause,
            proc_origin: source.cast.proc_origin.as_ref().map(|origin| Proc {
                event_seq: origin.event_seq,
                activation_source: &origin.activation_source,
                binding_index: origin.binding_index,
            }),
        })
        .serialize(serializer)
}

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct MandatoryObservation {
    pub key: Option<ObservationKey>,
    pub applications: u64,
    pub nonzero_hp_applications: u64,
    pub sourced_applications: u64,
    pub latest: Option<ApplicationWitness>,
    pub latest_nonzero_sourced_hp: Option<ApplicationWitness>,
    pub latest_sourced_sustained: Option<ApplicationWitness>,
    pub latest_sourced_hit_counter: Option<ApplicationWitness>,
    pub latest_sourced_resonance: Option<ApplicationWitness>,
    pub area_changes: u64,
    pub latest_area: Option<super::area_observation::AreaWitness>,
    pub latest_ready_wait: Option<super::equipment_defense::ReadyWaitWitness>,
    pub latest_nearby_defense: Option<super::equipment_defense::NearbyDefenseWitness>,
    pub latest_primary_damage: Option<super::equipment_primary_damage::PrimaryDamageWitness>,
    pub latest_area_damage: Option<super::area_damage::AreaDamageWitness>,
    pub latest_nearby_ally_damage: Option<super::nearby_ally_damage::NearbyAllyDamageWitness>,
    pub latest_conditional_damage: Option<super::conditional_damage::ConditionalDamageWitness>,
    pub latest_direct_penetration: Option<super::direct_penetration::DirectPenetrationWitness>,
    pub latest_conditional_reduction: Option<super::equipment_defense::IncomingConditionalWitness>,
    pub latest_actual_heal_reaction: Option<super::equipment_support::ActualHealReactionWitness>,
    pub latest_resonance_gain: Option<super::equipment_resonance_gain::ResonanceGainWitness>,
    pub latest_attack_continuity: Option<super::attack_continuity::AttackContinuityWitness>,
    pub latest_release_gap: Option<super::release_gap::ReleaseGapWitness>,
    pub latest_equipment_speed: Option<super::equipment_attack::EquipmentSpeedWitness>,
    pub latest_last_will: Option<super::equipment_defense::LastWillWitness>,
    pub latest_employee_transformation: Option<super::equipment_defense::EmployeeTransformationWitness>,
    pub latest_weapon_stage: Option<super::weapon_stages::WeaponStageWitness>,
    pub latest_adaptive_shield: Option<super::equipment_defense::AdaptiveShieldWitness>,
    pub latest_channel_opening: Option<super::equipment_defense::ChannelOpeningWitness>,
    pub latest_idle_hp_guard: Option<super::equipment_defense::IdleHpGuardWitness>,
    pub latest_incoming_flat: Option<super::equipment_defense::IncomingFlatWitness>,
    pub latest_wearer_reduction: Option<super::equipment_defense::WearerReductionWitness>,
    pub latest_shield_beneficiary_damage: Option<super::equipment_support::ShieldBeneficiaryDamageWitness>,
    pub latest_received_heal: Option<super::equipment_support::ReceivedHealWitness>,
    pub latest_control_duration: Option<super::equipment_support::ControlDurationWitness>,
    pub latest_skill_barrier: Option<super::equipment_defense::SkillBarrierWitness>,
    pub latest_shared_cohort: Option<super::shared_cohort::SharedCohortWitness>,
    pub latest_equipment_death_burst: Option<super::equipment_death_burst::DeathBurstWitness>,
    pub latest_equipment_form: Option<super::equipment_forms::FormWitness>,
    pub latest_bounty: Option<super::cohort_bounty::BountyWitness>,
    pub latest_magazine: Option<super::equipment_magazine::MagazineWitness>,
    pub latest_deferred_damage: Option<super::equipment_defense::DeferredDamageWitness>,
    pub latest_overheal_reserve: Option<super::equipment_defense::OverhealReserveWitness>,
    pub latest_basic_heal_echo: Option<super::equipment_support::BasicHealEchoWitness>,
    pub latest_heal_barrier: Option<super::equipment_support::HealBarrierWitness>,
    pub latest_equipment_support: Option<super::equipment_support::EquipmentSupportWitness>,
    pub latest_basic_evasion: Option<super::equipment_defense::BasicEvasionWitness>,
    #[serde(skip)]
    pub(crate) amount_observation_enabled: bool,
    #[serde(skip)]
    pub(crate) amount_evaluations: u64,
    #[serde(skip)]
    pub(crate) latest_amount: Option<Arc<super::amount_observation::RecordedAmountEvaluation>>,
    pub(crate) coverage:Option<Box<crate::mod_validation::coverage::CoverageCollector>>,
    #[serde(skip)]
    failed: bool,
}
crate::authoring::clone_shape::struct_shape!(MandatoryObservation {
    key,
    applications,
    nonzero_hp_applications,
    sourced_applications,
    latest,
    latest_nonzero_sourced_hp,
    latest_sourced_sustained,
    latest_sourced_hit_counter,
    latest_sourced_resonance,
    area_changes,
    latest_area,
    latest_ready_wait,
    latest_nearby_defense,
    latest_primary_damage,
    latest_area_damage,
    latest_nearby_ally_damage,
    latest_conditional_damage,
    latest_direct_penetration,
    latest_conditional_reduction,
    latest_actual_heal_reaction,
    latest_resonance_gain,
    latest_attack_continuity,
    latest_release_gap,
    latest_equipment_speed,
    latest_last_will,
    latest_employee_transformation,
    latest_weapon_stage,
    latest_adaptive_shield,
    latest_channel_opening,
    latest_idle_hp_guard,
    latest_incoming_flat,
    latest_wearer_reduction,
    latest_shield_beneficiary_damage,
    latest_received_heal,
    latest_control_duration,
    latest_skill_barrier,
    latest_basic_evasion,
    latest_equipment_support,
    latest_heal_barrier,
    latest_basic_heal_echo,
    latest_deferred_damage,
    latest_magazine,
    latest_shared_cohort,
    latest_bounty,
    latest_equipment_form,
    latest_equipment_death_burst,
    latest_overheal_reserve,
    amount_observation_enabled,
    amount_evaluations,
    latest_amount,
    coverage,
    failed
});

impl MandatoryObservation {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = std::mem::size_of::<Self>();
        for witness in [&self.latest, &self.latest_nonzero_sourced_hp, &self.latest_sourced_sustained, &self.latest_sourced_resonance, &self.latest_sourced_hit_counter]
            .into_iter()
            .flatten()
        {
            if let Some(area) = &witness.area_source { bytes = bytes.checked_add(area.retained_extent()?)?; }
            if witness.source.is_none() { if let Some(fragment) = &witness.fragment { bytes = bytes.checked_add(fragment.retained_extent()?)?; } }
            if let Some(source) = &witness.source {
                let fact = crate::game::battle::runtime_facts::RuntimeFactSource {
                    actor: Some(source.cast.caster_instance_id),
                    fragment: witness.fragment.clone(),
                    cast: Some(Arc::clone(&source.cast)),
                    authored: Some(Arc::clone(&source.authored)),
                    ..Default::default()
                };
                bytes = bytes.checked_add(fact.retained_extent()?)?;
            } else if let Some(authored) = &witness.authored {
                let fact = crate::game::battle::runtime_facts::RuntimeFactSource {
                    authored: Some(Arc::clone(authored)), equipment: witness.equipment,
                    ..Default::default()
                };
                bytes = bytes.checked_add(fact.retained_extent()?)?;
            }
        }
        bytes = bytes.checked_add(self.latest_area.as_ref().map_or(Some(0), |a| a.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_ready_wait.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_nearby_defense.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_primary_damage.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_area_damage.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_nearby_ally_damage.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_conditional_damage.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_direct_penetration.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_conditional_reduction.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_actual_heal_reaction.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_resonance_gain.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_attack_continuity.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_release_gap.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_equipment_speed.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_employee_transformation.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_last_will.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_weapon_stage.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_adaptive_shield.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_channel_opening.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_idle_hp_guard.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_incoming_flat.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_wearer_reduction.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_shield_beneficiary_damage.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_received_heal.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_control_duration.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_skill_barrier.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_heal_barrier.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_shared_cohort.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_equipment_death_burst.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_equipment_form.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_bounty.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_magazine.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_deferred_damage.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_overheal_reserve.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_basic_heal_echo.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_equipment_support.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes = bytes.checked_add(self.latest_basic_evasion.as_ref().map_or(Some(0), |w| w.retained_extent())?)?;
        bytes=bytes.checked_add(self.coverage.as_ref().map_or(Some(0),|c|c.report.retained_extent())?)?;
        bytes.checked_add(
            self.latest_amount
                .as_ref()
                .map_or(0, |r| r.retained_extent()),
        )
    }

    pub(crate) fn failed(&self) -> bool {
        self.failed
    }
}

pub(super) struct ObservationReservation {
    ordinal: u64,
    next_nonzero: u64,
    next_sourced: u64,
    source: Option<Arc<SkillEffectExecutionSource>>,
    authored: Option<Arc<crate::game::ability::authoring::EffectSource>>,
    area_source: Option<Arc<super::area_observation::AreaEffectSource>>,
    equipment: Option<crate::game::battle::runtime_facts::EquipmentSourceSnapshot>,
    fragment: Option<Arc<crate::game::skill_fragment::FragmentSourceSnapshot>>,
}

impl BattleCore {
    pub(super) fn observe_sustained_change(
        &mut self,
        source: &crate::game::battle::runtime_facts::RuntimeFactSource,
        area_source: Option<&Arc<super::area_observation::AreaEffectSource>>,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        now: u64,
        change: AppliedChange,
    ) -> Result<(), GameError> {
        if self.authoring_observation.is_none() { return Ok(()); }
        let reservation = self.reserve_authoring_observation_for_effect(source, area_source)?;
        self.commit_authoring_observation(reservation, now, Some(caster), target, change);
        control::poll("sustained_observation_committed")
    }

    /// Observation provenance is separate from damage/action authority. An
    /// authored enhancement can remain a normal attack with its original ticket.
    pub(super) fn reserve_authoring_observation_for_effect(
        &mut self,
        source: &crate::game::battle::runtime_facts::RuntimeFactSource,
        area_source: Option<&Arc<super::area_observation::AreaEffectSource>>,
    ) -> Result<Option<ObservationReservation>, GameError> {
        if self.authoring_observation.is_none() { return Ok(None); }
        // Capture the activation's own immutable provenance, including expiry
        // after its caster/action has exited. Never borrow the ambient event.
        control::charge(4, "sustained_observation_source")?;
        if source.cast.is_some() && source.authored.is_some() && area_source.is_none() {
            crate::game::battle::runtime_control::admit_retained_growth(
                0, Some(std::mem::size_of::<SkillEffectExecutionSource>()), "sustained_observation_source",
            )?;
        }
        let effect = source.cast.as_ref().zip(source.authored.as_ref()).filter(|_| area_source.is_none()).map(|(cast, authored)| {
            Arc::new(SkillEffectExecutionSource { cast: Arc::clone(cast), authored: Arc::clone(authored) })
        });
        let mut reservation = Self::reserve_authoring_observation_parts(
            self.authoring_observation.as_mut(), effect.as_ref(), source.fragment.as_ref(), source, area_source,
        )?;
        // A menu cast keeps its own skill ID while its fragment identifies the
        // root definition. Validate that declared ownership; never rewrite either
        // identity or infer ownership from an ID prefix.
        if let (Some(reservation), Some(effect), Some(fragment)) =
            (reservation.as_mut(), effect.as_ref(), source.fragment.as_ref())
        {
            if reservation.fragment.is_none() && area_source.is_none() {
                control::charge(1, "sustained_observation_fragment_owner")?;
                if self.fragment_owns_skill(&fragment.skill_id, &effect.cast.skill_id) {
                    reservation.fragment = Some(Arc::clone(fragment));
                }
            }
        }
        Ok(reservation)
    }
    pub(crate) fn set_lab_coverage(&mut self,plan:&crate::mod_validation::CoveragePlan)->Result<(),GameError>{
        let collector=crate::mod_validation::coverage::CoverageCollector::new(plan)?;
        self.authoring_observation.as_mut().ok_or_else(||control::reject("lab_coverage_missing","lab_coverage_owner"))?.coverage=Some(collector);Ok(())
    }
    pub(super) fn record_lab_source(&mut self,source:&crate::game::battle::runtime_facts::RuntimeFactSource,stage:crate::mod_validation::CoverageStage)->Result<(),GameError>{
        if let Some(coverage)=self.authoring_observation.as_mut().and_then(|o|o.coverage.as_mut()){
            coverage.price_source(source)?;
            coverage.record_source(source,stage);
        }Ok(())
    }
    pub(super) fn record_lab_site(&mut self,source:&crate::game::battle::runtime_facts::RuntimeFactSource,site:Option<&crate::game::ability::authoring::EffectSite>,stage:crate::mod_validation::CoverageStage)->Result<(),GameError>{
        if let Some(coverage)=self.authoring_observation.as_mut().and_then(|o|o.coverage.as_mut()){
            coverage.price_source(source)?;coverage.record_source_at(source,site,stage);
        }Ok(())
    }
    pub(super) fn record_lab_object(&mut self,id:&str,terminal:bool)->Result<(),GameError>{
        if let Some(coverage)=self.authoring_observation.as_mut().and_then(|o|o.coverage.as_mut()){
            coverage.price_record()?;coverage.record_object(id,terminal);
        }Ok(())
    }
    pub(crate) fn enable_authoring_observation(&mut self) {
        self.enable_authoring_observation_with_trace(Default::default());
    }

    pub(crate) fn enable_authoring_observation_with_trace(
        &mut self,
        options: crate::authoring::trace::TraceOptions,
    ) {
        self.enable_authoring_observation_with_options(options, false);
    }

    pub(crate) fn enable_authoring_observation_with_options(
        &mut self,
        trace: crate::authoring::trace::TraceOptions,
        amount_observation_enabled: bool,
    ) {
        self.authoring_observation = Some(MandatoryObservation {
            amount_observation_enabled,
            ..MandatoryObservation::default()
        });
        self.authoring_direct_source = None;
        self.authoring_trace = crate::authoring::trace::OptionalTrace::new(trace);
    }

    pub(crate) fn authoring_observation(&self) -> Result<Option<&MandatoryObservation>, GameError> {
        if self
            .authoring_observation
            .as_ref()
            .is_some_and(|state| state.failed)
        {
            return Err(GameError::InvalidStaticData(
                "authoring_observation_invalid".into(),
            ));
        }
        Ok(self.authoring_observation.as_ref())
    }

    pub(crate) fn bind_authoring_observation(
        &mut self,
        key: ObservationKey,
    ) -> Result<(), GameError> {
        let state = self
            .authoring_observation
            .as_mut()
            .ok_or_else(|| GameError::InvalidStaticData("authoring_observation_missing".into()))?;
        if state.failed || state.key.is_some() {
            return Err(GameError::InvalidStaticData(
                "authoring_observation_invalid".into(),
            ));
        }
        state.key = Some(key);
        Ok(())
    }

    pub(crate) fn take_authoring_observation(&mut self) -> Result<MandatoryObservation, GameError> {
        if self
            .authoring_observation()?
            .and_then(|state| state.key)
            .is_none()
        {
            return Err(GameError::InvalidStaticData(
                "authoring_observation_missing".into(),
            ));
        }
        Ok(self
            .authoring_observation
            .take()
            .expect("validated observer owner"))
    }

    pub(super) fn reserve_authoring_observation(
        &mut self,
    ) -> Result<Option<ObservationReservation>, GameError> {
        Self::reserve_authoring_observation_parts(
            self.authoring_observation.as_mut(),
            self.authoring_direct_source.as_ref(),
            self.runtime_fact_source.fragment.as_ref(),
            &self.runtime_fact_source,
            None,
        )
    }

    pub(super) fn reserve_authoring_observation_with_source(
        &mut self,
        source: Option<&Arc<SkillEffectExecutionSource>>,
    ) -> Result<Option<ObservationReservation>, GameError> {
        Self::reserve_authoring_observation_parts(
            self.authoring_observation.as_mut(),
            source,
            self.runtime_fact_source.fragment.as_ref(),
            &self.runtime_fact_source,
            None,
        )
    }

    fn reserve_authoring_observation_parts(
        state: Option<&mut MandatoryObservation>,
        source: Option<&Arc<SkillEffectExecutionSource>>,
        fragment: Option<&Arc<crate::game::skill_fragment::FragmentSourceSnapshot>>,
        runtime_source: &crate::game::battle::runtime_facts::RuntimeFactSource,
        area_source: Option<&Arc<super::area_observation::AreaEffectSource>>,
    ) -> Result<Option<ObservationReservation>, GameError> {
        let Some(state) = state else {
            return Ok(None);
        };
        // One operation plus bounded immutable source-handle copies.
        control::charge(4, "authoring_observation_reserve")?;
        let authored = (source.is_none() && area_source.is_none()).then_some(runtime_source.authored.as_ref()).flatten();
        if let Some(coverage)=&state.coverage {coverage.price_source(runtime_source)?;}
        let counters = state
            .applications
            .checked_add(1)
            .zip(state.nonzero_hp_applications.checked_add(1))
            .zip(if source.is_some() || authored.is_some() || area_source.is_some() {
                state.sourced_applications.checked_add(1)
            } else {
                Some(state.sourced_applications)
            });
        let Some(((ordinal, next_nonzero), next_sourced)) = counters.filter(|_| !state.failed)
        else {
            state.failed = true;
            return Err(control::reject(
                "authoring_observation_invalid",
                "authoring_observation_counter",
            ));
        };
        // Both root and child effects need the same captured occurrence before
        // borrowing its providers. A matching skill name does not identify a
        // cast, and an explicit source alone cannot authenticate ambient gear.
        let same_cast = source.is_some_and(|effect| {
            runtime_source.cast.as_ref() == Some(&effect.cast)
        });
        let same_child_cast = same_cast && fragment.zip(source).is_some_and(|(fragment, effect)| {
            effect.cast.skill_id != fragment.skill_id
        });
        if same_child_cast {
            control::charge(1, "sustained_observation_fragment_owner")?;
        }
        Ok(Some(ObservationReservation {
            ordinal,
            next_nonzero,
            next_sourced,
            source: source.cloned(),
            authored: authored.cloned(),
            area_source: area_source.cloned(),
            equipment: (same_cast || authored.is_some()).then_some(runtime_source.equipment).flatten(),
            fragment: fragment
                .filter(|fragment| {
                    same_cast
                        || area_source.is_some_and(|area| area.matches_cast(runtime_source.cast.as_ref()))
                        // Intrinsic effects have an authored site but no cast.
                        || authored.is_some_and(|effect| effect.site.action.definition_id == fragment.skill_id)
                })
                .cloned(),
        }))
    }

    pub(super) fn commit_authoring_observation(
        &mut self,
        reservation: Option<ObservationReservation>,
        time_ms: u64,
        source_unit: Option<UnitInstanceId>,
        target_unit: UnitInstanceId,
        change: AppliedChange,
    ) {
        let Some(reservation) = reservation else {
            return;
        };
        let state = self
            .authoring_observation
            .as_mut()
            .expect("reserved observation owner");
        let nonzero = change.hp_delta() != 0;
        let witness = ApplicationWitness {
            ordinal: reservation.ordinal,
            time_ms,
            source_unit,
            target_unit,
            change,
            source: reservation.source,
            authored: reservation.authored,
            area_source: reservation.area_source,
            equipment: reservation.equipment,
            fragment: reservation.fragment,
        };
        state.applications = reservation.ordinal;
        // Sustained lifecycle is additional observation of an existing effect,
        // not another coverage application (especially at delayed expiry).
        if !matches!(change,AppliedChange::Hp{..}|AppliedChange::Sustained{..}|AppliedChange::HitCounter{..}) {
            if let Some(coverage)=&mut state.coverage {
                coverage.record_effect(witness.source.as_deref(),crate::mod_validation::CoverageStage::ConsumedOrEffect);
                coverage.record_source(&self.runtime_fact_source,crate::mod_validation::CoverageStage::ConsumedOrEffect);
                if let AppliedChange::Buff{buff_id,..}=change {coverage.record_buff(buff_id);}
            }
        }
        state.sourced_applications = reservation.next_sourced;
        if matches!(change, AppliedChange::Sustained { .. }) && (witness.source.is_some() || witness.authored.is_some() || witness.area_source.is_some()) {
            state.latest_sourced_sustained = Some(witness.clone());
        }
        if matches!(change, AppliedChange::HitCounter { .. }) && (witness.source.is_some() || witness.authored.is_some() || witness.area_source.is_some()) {
            state.latest_sourced_hit_counter = Some(witness.clone());
        }
        if matches!(change, AppliedChange::Resonance { .. }) && (witness.source.is_some() || witness.authored.is_some()) {
            state.latest_sourced_resonance = Some(witness.clone());
        }
        if nonzero {
            state.nonzero_hp_applications = reservation.next_nonzero;
            if witness.source.is_some() || witness.authored.is_some() {
                state.latest_nonzero_sourced_hp = Some(witness.clone());
            }
        }
        state.latest = Some(witness);
        self.authoring_trace
            .record(state.latest.as_ref().expect("committed witness"));
    }
}

#[cfg(test)]
mod provenance_tests {
    use super::*;
    use crate::game::{
        ability::authoring::{ActionSite, EffectOrigin, EffectSite, EffectSource},
        battle::{runtime_facts::RuntimeFactSource, skill_source::SkillCastExecutionSource},
        skill_fragment::FragmentSourceSnapshot,
    };

    #[test]
    fn child_fragment_observation_requires_the_same_cast_occurrence() {
        let actor = UnitInstanceId::from(Uuid::from_u128(0x126a));
        let cast = Arc::new(SkillCastExecutionSource {
            skill_id: "counter_child".into(),
            caster_instance_id: actor,
            cast_seq: 3,
            ability_event_seq: 5,
            cause: Default::default(),
            proc_origin: None,
        });
        let effect = Arc::new(SkillEffectExecutionSource {
            cast: cast.clone(),
            authored: Arc::new(EffectSource {
                source_file: "fixture".into(),
                site: EffectSite {
                    action: ActionSite {
                        definition_id: cast.skill_id.clone(),
                        action_id: "counter".into(),
                    },
                    effect_id: "damage".into(),
                },
                field_path: "counter.damage".into(),
                origin_definition_id: cast.skill_id.clone(),
                origin: EffectOrigin::Original,
            }),
        });
        let fragment = Arc::new(FragmentSourceSnapshot {
            copy_uuid: Uuid::from_u128(0x126b),
            definition_id: "guard_fragment".into(),
            form_id: "base".into(),
            skill_id: "guard_root".into(),
        });
        // Identical immutable copies are one occurrence. Skill-name equality
        // alone, a different caster, or no occurrence must not borrow ancestry.
        for case in 0..5 {
            let mut captured = (*cast).clone();
            match case {
                1 => captured.cast_seq += 1,
                2 => captured.caster_instance_id = UnitInstanceId::from(Uuid::from_u128(0x126c)),
                3 => captured.skill_id = "other_child".into(),
                _ => {}
            }
            let source = RuntimeFactSource {
                actor: Some(actor),
                cast: (case != 4).then(|| Arc::new(captured)),
                fragment: Some(fragment.clone()),
                ..Default::default()
            };
            let mut state = MandatoryObservation::default();
            let reservation = BattleCore::reserve_authoring_observation_parts(
                Some(&mut state),
                Some(&effect),
                source.fragment.as_ref(),
                &source,
                None,
            )
            .unwrap()
            .unwrap();
            assert_eq!(reservation.fragment.is_some(), case == 0, "case {case}");
            assert_eq!(reservation.source.as_ref().unwrap().cast, cast);
            if let Some(actual) = reservation.fragment {
                assert_eq!(actual.copy_uuid, fragment.copy_uuid);
                assert_eq!(actual.skill_id, fragment.skill_id);
            }
        }
    }

    #[test]
    fn root_fragment_and_equipment_observation_require_same_cast_occurrence() {
        let actor = UnitInstanceId::from(Uuid::from_u128(0x126d));
        let cast = Arc::new(SkillCastExecutionSource {
            skill_id: "same_root".into(), caster_instance_id: actor,
            cast_seq: 2, ability_event_seq: 3, cause: Default::default(), proc_origin: None,
        });
        let effect = Arc::new(SkillEffectExecutionSource {
            cast: cast.clone(),
            authored: Arc::new(EffectSource {
                source_file: "fixture".into(), field_path: "skills[0].steps[0].effects[0]".into(),
                site: EffectSite {
                    action: ActionSite { definition_id: "same_root".into(), action_id: "hit".into() },
                    effect_id: "damage".into(),
                },
                origin_definition_id: "same_root".into(), origin: EffectOrigin::Original,
            }),
        });
        let fragment = Arc::new(FragmentSourceSnapshot {
            copy_uuid: Uuid::from_u128(0x126e), definition_id: "root_fragment".into(),
            form_id: "base".into(), skill_id: "same_root".into(),
        });
        let equipment = crate::game::battle::runtime_facts::EquipmentSourceSnapshot {
            definition_uuid: Uuid::from_u128(0x126f),
            runtime_item_instance_id: Uuid::from_u128(0x1270),
            persistent_copy_uuid: Some(Uuid::from_u128(0x1271)),
        };
        let mut wrongly_attributed = Vec::new();
        for case in 0..6 {
            let mut other = (*cast).clone();
            match case {
                1 => other.cast_seq += 1,
                2 => other.caster_instance_id = UnitInstanceId::from(Uuid::from_u128(0x1272)),
                3 => other.skill_id = "another_root".into(),
                5 => other.ability_event_seq += 1,
                _ => {}
            }
            let source = RuntimeFactSource {
                actor: Some(actor), cast: (case != 4).then(|| Arc::new(other)),
                fragment: Some(fragment.clone()), equipment: Some(equipment),
                ..Default::default()
            };
            let mut state = MandatoryObservation::default();
            let reservation = BattleCore::reserve_authoring_observation_parts(
                Some(&mut state), Some(&effect), source.fragment.as_ref(), &source, None,
            ).unwrap().unwrap();
            assert_eq!(reservation.source.as_ref().unwrap().cast, cast,
                "known effect remains known even when ambient copy linkage is unavailable");
            if case == 0 {
                assert_eq!(reservation.fragment.as_ref().unwrap().copy_uuid, fragment.copy_uuid);
                assert_eq!(reservation.equipment, Some(equipment));
            } else if reservation.fragment.is_some() || reservation.equipment.is_some() {
                wrongly_attributed.push(case);
            }
        }
        eprintln!("root occurrence verifier borrowed unrelated providers in cases {wrongly_attributed:?}");
        // A castless intrinsic authored provider is a normal source, not a
        // missing skill cast to fill with the most recent occurrence.
        let intrinsic = RuntimeFactSource {
            actor: Some(actor), equipment: Some(equipment), authored: Some(effect.authored.clone()),
            fragment: Some(fragment.clone()), ..Default::default()
        };
        let mut state = MandatoryObservation::default();
        let reservation = BattleCore::reserve_authoring_observation_parts(
            Some(&mut state), None, intrinsic.fragment.as_ref(), &intrinsic, None,
        ).unwrap().unwrap();
        assert!(reservation.source.is_none());
        assert_eq!(reservation.authored, intrinsic.authored);
        assert_eq!(reservation.equipment, Some(equipment));
        assert_eq!(reservation.fragment.as_ref().unwrap().copy_uuid, fragment.copy_uuid);
        assert!(wrongly_attributed.is_empty());
    }

    #[test]
    fn area_fragment_observation_requires_the_captured_cast_occurrence() {
        let actor = UnitInstanceId::from(Uuid::from_u128(0xa126));
        let cast = Arc::new(SkillCastExecutionSource {
            skill_id: "same_ground".into(), caster_instance_id: actor,
            cast_seq: 7, ability_event_seq: 11, cause: Default::default(), proc_origin: None,
        });
        let area = Arc::new(super::super::area_observation::AreaEffectSource {
            generation: None, captured_cast: Some(cast.clone()), area_id: Uuid::from_u128(0xa127),
            skill_id: "same_ground".into(), step_id: "ground".into(), cast_sequence: 7,
            status_index: 0, definition: "ground-vulnerability".into(), modifier_source: None,
        });
        let fragment = Arc::new(FragmentSourceSnapshot {
            copy_uuid: Uuid::from_u128(0xa128), definition_id: "ground_fragment".into(),
            form_id: "base".into(), skill_id: "same_ground".into(),
        });
        let mut wrongly_attributed = Vec::new();
        for case in 0..6 {
            let mut other = (*cast).clone();
            match case {
                1 => other.cast_seq += 1,
                2 => other.caster_instance_id = UnitInstanceId::from(Uuid::from_u128(0xa129)),
                3 => other.skill_id = "other_ground".into(),
                5 => other.ability_event_seq += 1,
                _ => {}
            }
            let source = RuntimeFactSource {
                actor: Some(actor), cast: (case != 4).then(|| Arc::new(other)),
                fragment: Some(fragment.clone()), ..Default::default()
            };
            let mut state = MandatoryObservation::default();
            let reservation = BattleCore::reserve_authoring_observation_parts(
                Some(&mut state), None, source.fragment.as_ref(), &source, Some(&area),
            ).unwrap().unwrap();
            assert_eq!(reservation.area_source.as_ref().unwrap().area_id, Uuid::from_u128(0xa127));
            assert!(reservation.source.is_none() && reservation.authored.is_none());
            if case == 0 {
                assert_eq!(reservation.fragment.as_ref().unwrap().copy_uuid, Uuid::from_u128(0xa128));
            } else if reservation.fragment.is_some() {
                wrongly_attributed.push(case);
            }
        }
        eprintln!("area occurrence verifier borrowed unrelated fragments in cases {wrongly_attributed:?}");
        assert!(wrongly_attributed.is_empty());
        // Static metadata remains usable without claiming a runtime occurrence.
        // Inconsistent public occurrence fields must not authenticate providers.
        for case in 0..3 {
            let mut unbound_area = (*area).clone();
            match case {
                0 => unbound_area.captured_cast = None,
                1 => unbound_area.cast_sequence += 1,
                _ => unbound_area.skill_id = "other_ground".into(),
            }
            let source = RuntimeFactSource {
                actor: Some(actor), cast: Some(cast.clone()), fragment: Some(fragment.clone()),
                ..Default::default()
            };
            let mut state = MandatoryObservation::default();
            let unbound_area = Arc::new(unbound_area);
            let reservation = BattleCore::reserve_authoring_observation_parts(
                Some(&mut state), None, source.fragment.as_ref(), &source, Some(&unbound_area),
            ).unwrap().unwrap();
            assert!(reservation.fragment.is_none());
            assert!(reservation.area_source.is_some());
        }
        assert!(serde_json::to_value(area).unwrap().get("captured_cast").is_none(),
            "private occurrence matching must not extend the public area contract");
    }

}
