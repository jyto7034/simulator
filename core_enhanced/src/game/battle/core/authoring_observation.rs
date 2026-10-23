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
}
crate::authoring::clone_shape::copy_shape!(AppliedChange);

impl AppliedChange {
    pub fn hp_delta(&self) -> i64 {
        let (before, after) = match self {
            Self::Resonance { .. } | Self::Buff { .. } => return 0,
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
}
crate::authoring::clone_shape::struct_shape!(ApplicationWitness {
    ordinal,
    time_ms,
    source_unit,
    target_unit,
    fragment,
    change,
    source
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
        } = self;
        scalar(ordinal);
        scalar(time_ms);
        scalar(source_unit);
        scalar(target_unit);
        scalar(change);
        let mut work = 256_u64;
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
    amount_observation_enabled,
    amount_evaluations,
    latest_amount,
    coverage,
    failed
});

impl MandatoryObservation {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = std::mem::size_of::<Self>();
        for witness in [&self.latest, &self.latest_nonzero_sourced_hp]
            .into_iter()
            .flatten()
        {
            if let Some(source) = &witness.source {
                let fact = crate::game::battle::runtime_facts::RuntimeFactSource {
                    actor: Some(source.cast.caster_instance_id),
                    fragment: witness.fragment.clone(),
                    cast: Some(Arc::clone(&source.cast)),
                    authored: Some(Arc::clone(&source.authored)),
                    ..Default::default()
                };
                bytes = bytes.checked_add(fact.retained_extent()?)?;
            }
        }
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
    fragment: Option<Arc<crate::game::skill_fragment::FragmentSourceSnapshot>>,
}

impl BattleCore {
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
        )
    }

    fn reserve_authoring_observation_parts(
        state: Option<&mut MandatoryObservation>,
        source: Option<&Arc<SkillEffectExecutionSource>>,
        fragment: Option<&Arc<crate::game::skill_fragment::FragmentSourceSnapshot>>,
        runtime_source: &crate::game::battle::runtime_facts::RuntimeFactSource,
    ) -> Result<Option<ObservationReservation>, GameError> {
        let Some(state) = state else {
            return Ok(None);
        };
        // One operation plus at most two immutable source-handle copies.
        control::charge(3, "authoring_observation_reserve")?;
        if let Some(coverage)=&state.coverage {coverage.price_source(runtime_source)?;}
        let counters = state
            .applications
            .checked_add(1)
            .zip(state.nonzero_hp_applications.checked_add(1))
            .zip(if source.is_some() {
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
        Ok(Some(ObservationReservation {
            ordinal,
            next_nonzero,
            next_sourced,
            source: source.cloned(),
            fragment: fragment
                .filter(|fragment| {
                    source.is_some_and(|source| source.cast.skill_id == fragment.skill_id)
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
            fragment: reservation.fragment,
        };
        state.applications = reservation.ordinal;
        if !matches!(change,AppliedChange::Hp{..}) {
            if let Some(coverage)=&mut state.coverage {
                coverage.record_effect(witness.source.as_deref(),crate::mod_validation::CoverageStage::ConsumedOrEffect);
                coverage.record_source(&self.runtime_fact_source,crate::mod_validation::CoverageStage::ConsumedOrEffect);
                if let AppliedChange::Buff{buff_id,..}=change {coverage.record_buff(buff_id);}
            }
        }
        state.sourced_applications = reservation.next_sourced;
        if nonzero {
            state.nonzero_hp_applications = reservation.next_nonzero;
            if witness.source.is_some() {
                state.latest_nonzero_sourced_hp = Some(witness.clone());
            }
        }
        state.latest = Some(witness);
        self.authoring_trace
            .record(state.latest.as_ref().expect("committed witness"));
    }
}
