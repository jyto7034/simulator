use super::*;
use crate::game::{
    ability::sustained::{PreparedBasicProfile, PreparedBasicView},
    battle::{
        damage::{DamageSource, DamageSourceSnapshot, PreparedBasicSnapshot},
        runtime_facts::ActionHandle,
    },
};

#[derive(Debug, Clone)]
pub(super) struct PreparedRecord {
    caster: UnitInstanceId,
    profile: PreparedBasicProfile,
    pub(super) source: RuntimeFactSource,
    bound: Option<ActionHandle>,
    epoch: u64,
    released: bool,
    rewarded: bool,
    superseded: bool,
}
crate::authoring::clone_shape::struct_shape!(PreparedRecord {
    caster,
    profile,
    source,
    bound,
    epoch,
    released,
    rewarded,
    superseded
});
impl PreparedRecord {
    pub(super) fn retained_extent(&self) -> Option<usize> {
        self.source
            .retained_extent()?
            .checked_add(std::mem::size_of::<Self>() + 160)?
            .checked_add(self.profile.definition.capacity())?
            .checked_add(
                self.profile
                    .kill_reward
                    .as_ref()
                    .map_or(0, |r| r.profile.definition.capacity()),
            )
    }
}
impl BattleCore {
    pub(super) fn begin_prepared_basic(
        &mut self,
        caster: UnitInstanceId,
        profile: PreparedBasicProfile,
        now: u64,
    ) -> Result<(), GameError> {
        self.reconcile_prepared_basics(now)?;
        let source = crate::authoring::clone_shape::controlled_clone(&self.runtime_fact_source)?;
        let id = self
            .sustained
            .next
            .checked_add(1)
            .ok_or_else(|| overflow("prepared_basic_activation"))?;
        let record = PreparedRecord {
            caster,
            profile,
            source,
            bound: None,
            epoch: 0,
            released: false,
            rewarded: false,
            superseded: false,
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            record.retained_extent(),
            "prepared_basic_record",
        )?;
        self.admit_authoring_runtime_growth(1, "prepared_basic_entity")?;
        // A new preparation supersedes only its own authored source family.
        // A bound attack finishes with its old capture; cancellation cannot resurrect it.
        let mut cursor = 0;
        loop {
            control::charge(
                self.sustained.prepared.len() as u64,
                "prepared_basic_replace",
            )?;
            let old = self
                .sustained
                .prepared
                .range((
                    std::ops::Bound::Excluded(cursor),
                    std::ops::Bound::Unbounded,
                ))
                .find(|(_, r)| {
                    r.caster == caster
                        && r.profile.definition == record.profile.definition
                        && r.source.fragment.as_ref().map(|f| f.copy_uuid)
                            == record.source.fragment.as_ref().map(|f| f.copy_uuid)
                        && r.source.equipment == record.source.equipment
                        && !r.released
                })
                .map(|(id, r)| (*id, r.bound.is_some()));
            let Some((old, bound)) = old else {
                break;
            };
            cursor = old;
            if bound {
                self.sustained.prepared.get_mut(&old).unwrap().superseded = true;
            } else {
                self.end_prepared_basic(old, now, SustainedEndReason::Replaced)?;
            }
        }
        let source = self.reserve_finite_source(record.source)?;
        if let (Some(cast), Some(flow)) = (&source.cast, &source.flow) {
            self.begin_skill_step_delivery(cast.cast_seq, flow.occurrence, 1);
        }
        self.sustained.next = id;
        self.sustained
            .prepared
            .insert(id, PreparedRecord { source, ..record });
        control::poll("prepared_basic_granted")
    }
    pub(super) fn end_prepared_basic(
        &mut self,
        id: u64,
        now: u64,
        reason: SustainedEndReason,
    ) -> Result<(), GameError> {
        let Some(record) = self.sustained.prepared.remove(&id) else {
            return Ok(());
        };
        self.close_source_resource_delivery(record.source.resource_delivery().map(|d| d.as_ref()))?;
        if let Some(ticket) = &record.source.delivery {
            self.runtime_facts.close_delivery(ticket, now)?;
        }
        if let (Some(cast), Some(flow)) = (&record.source.cast, &record.source.flow) {
            self.finish_skill_step_delivery(cast.cast_seq, flow.occurrence, now);
        }
        self.sustained.ended = self
            .sustained
            .ended
            .checked_add(1)
            .ok_or_else(|| overflow("prepared_basic_end"))?;
        self.sustained.latest_end = Some(SustainedEndReceipt {
            activation: id,
            caster: record.caster,
            target: record.caster,
            time_ms: now,
            reason,
        });
        control::poll("prepared_basic_ended")
    }
    pub(in crate::game::battle::core) fn reconcile_prepared_basics(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        let mut cursor = 0;
        loop {
            control::charge(1, "prepared_basic_reconcile")?;
            let Some((&id, record)) = self
                .sustained
                .prepared
                .range((
                    std::ops::Bound::Excluded(cursor),
                    std::ops::Bound::Unbounded,
                ))
                .next()
            else {
                break;
            };
            cursor = id;
            let unit = self.units.get(&record.caster).filter(|u| u.is_active());
            let life = record
                .bound
                .as_ref()
                .and_then(|a| self.runtime_facts.action_lifetime(a));
            let cancelled = record.bound.is_some()
                && !record.released
                && (unit.is_some_and(|u| u.basic_attack_epoch != record.epoch)
                    || life.is_none_or(|(_, terminal, _)| terminal));
            let reason = if unit.is_none() {
                Some(SustainedEndReason::SourceUnavailable)
            } else if record.released
                && life.is_none_or(|(_, terminal, outstanding)| terminal && outstanding == 0)
            {
                Some(SustainedEndReason::AttackReleased)
            } else if cancelled && record.superseded {
                Some(SustainedEndReason::Replaced)
            } else {
                None
            };
            if let Some(reason) = reason {
                self.end_prepared_basic(id, now, reason)?;
            } else if cancelled {
                self.sustained.prepared.get_mut(&id).unwrap().bound = None;
            }
        }
        Ok(())
    }
    pub(in crate::game::battle::core) fn bind_prepared_basic(
        &mut self,
        snapshot: &mut DamageSourceSnapshot,
        epoch: u64,
        now: u64,
    ) -> Result<(), GameError> {
        self.reconcile_prepared_basics(now)?;
        if snapshot.source != DamageSource::BasicAttack || snapshot.fact_source.reaction {
            return Ok(());
        }
        let Some(action) = snapshot.fact_source.action.as_ref() else {
            return Ok(());
        };
        control::charge(
            self.sustained.prepared.len() as u64,
            "prepared_basic_select",
        )?;
        // Multiple authored families queue deterministically; one preparation per attack.
        let Some((&id, record)) =
            self.sustained.prepared.iter().find(|(_, r)| {
                r.caster == snapshot.source_id && r.bound.is_none() && !r.superseded
            })
        else {
            return Ok(());
        };
        record.source.admit_owner("prepared_basic_capture_source")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<PreparedBasicSnapshot>() + 64),
            "prepared_basic_capture",
        )?;
        let capture = Arc::new(PreparedBasicSnapshot {
            activation: id,
            numerator: record.profile.numerator,
            denominator: record.profile.denominator,
            source: record.source.observation(),
        });
        snapshot.base_damage = capture.apply(snapshot.base_damage)?;
        snapshot.prepared_basic = Some(capture);
        let record = self.sustained.prepared.get_mut(&id).unwrap();
        record.bound = Some(action.clone());
        record.epoch = epoch;
        control::poll("prepared_basic_bound")
    }
    pub(in crate::game::battle::core) fn release_prepared_basic(
        &mut self,
        snapshot: &DamageSourceSnapshot,
    ) -> Result<(), GameError> {
        let Some(capture) = &snapshot.prepared_basic else {
            return Ok(());
        };
        let record = self
            .sustained
            .prepared
            .get_mut(&capture.activation)
            .ok_or(GameError::InvalidAction)?;
        if record.bound.as_ref() != snapshot.fact_source.action.as_ref() || record.released {
            return Err(GameError::InvalidAction);
        }
        record.released = true;
        control::poll("prepared_basic_released")
    }
    pub(in crate::game::battle::core) fn prepared_basic_kill(
        &mut self,
        snapshot: &DamageSourceSnapshot,
        now: u64,
    ) -> Result<(), GameError> {
        if snapshot.source != DamageSource::BasicAttack
            || snapshot.fact_source.reaction
            || !self
                .units
                .get(&snapshot.source_id)
                .is_some_and(|u| u.is_active())
        {
            return Ok(());
        }
        let Some(capture) = &snapshot.prepared_basic else {
            return Ok(());
        };
        let Some(record) = self.sustained.prepared.get_mut(&capture.activation) else {
            return Ok(());
        };
        if !record.released
            || record.rewarded
            || record.caster != snapshot.source_id
            || record.bound.as_ref() != snapshot.fact_source.action.as_ref()
        {
            return Ok(());
        }
        let Some(reward) = record.profile.kill_reward.as_ref() else {
            return Ok(());
        };
        control::charge_serialized(reward, "prepared_basic_reward_copy")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(reward.profile.definition.len()),
            "prepared_basic_reward_profile",
        )?;
        let reward = reward.clone();
        let source = crate::authoring::clone_shape::controlled_clone(&record.source)?;
        record.rewarded = true;
        let prior = std::mem::replace(&mut self.runtime_fact_source, source);
        let result = self.begin_sustained(
            snapshot.source_id,
            snapshot.source_id,
            SustainedEffectDef::TargetState {
                duration_ms: reward.duration_ms,
                profile: reward.profile,
            },
            now,
        );
        self.runtime_fact_source = prior;
        result?;
        control::poll("prepared_basic_rewarded")
    }
    pub(super) fn prepared_basic_views(&self) -> Result<Vec<PreparedBasicView>, GameError> {
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.sustained
                .prepared
                .len()
                .checked_mul(std::mem::size_of::<PreparedBasicView>()),
            "prepared_basic_views",
        )?;
        self.sustained
            .prepared
            .iter()
            .map(|(id, r)| {
                Ok(PreparedBasicView {
                    activation: *id,
                    caster: r.caster,
                    profile: crate::authoring::clone_shape::controlled_clone(&r.profile)?,
                    bound_action: r.bound.as_ref().map(|a| a.ordinal().0),
                    released: r.released,
                    rewarded: r.rewarded,
                    superseded: r.superseded,
                    fragment_copy_uuid: r.source.fragment.as_ref().map(|f| f.copy_uuid),
                })
            })
            .collect()
    }
}
