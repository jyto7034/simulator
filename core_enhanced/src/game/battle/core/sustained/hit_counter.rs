//! Finite per-source-copy counters that consume a threshold into ordinary CC.
use super::*;
use crate::game::ability::sustained::{
    HitCounterOutcome, HitStatusKind, HitStatusProfile, ThresholdControlKind,
};

pub(super) struct PendingHitCounter {
    pub outcome: HitCounterOutcome,
    control_profile: HitStatusProfile,
    control_ms: u32,
}

impl BattleCore {
    pub(super) fn merge_threshold_hit_status(
        &mut self,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        profile: &mut HitStatusProfile,
        duration_ms: u32,
        now: u64,
    ) -> Result<Option<PendingHitCounter>, GameError> {
        let HitStatusKind::ThresholdControl {
            stacks: added,
            threshold,
            control: kind,
            control_ms,
        } = profile.kind
        else {
            return Ok(None);
        };
        control::charge(self.sustained.records.len() as u64, "hit_counter_merge")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.sustained
                .records
                .len()
                .checked_mul(std::mem::size_of::<u64>()),
            "hit_counter_replacements",
        )?;
        let source = &self.runtime_fact_source;
        let mut prior = Vec::with_capacity(self.sustained.records.len());
        let mut before = 0u32;
        for (id, record) in &self.sustained.records {
            if record.caster != caster
                || record.target != target
                || record.expires_at <= now
                || record.source.equipment != source.equipment
                || record.source.fragment.as_ref().map(|s| s.copy_uuid)
                    != source.fragment.as_ref().map(|s| s.copy_uuid)
            {
                continue;
            }
            let Some(old) = &record.hit_status else {
                continue;
            };
            if old.definition != profile.definition {
                continue;
            }
            let HitStatusKind::ThresholdControl {
                stacks,
                threshold: old_threshold,
                control: old_kind,
                control_ms: old_ms,
            } = old.kind
            else {
                continue;
            };
            if old_threshold != threshold
                || old_kind != kind
                || old_ms != control_ms
                || stacks >= threshold
                || !prior.is_empty()
            {
                return Err(GameError::InvalidAction);
            }
            before = stacks;
            prior.push(*id);
        }
        let total = before
            .checked_add(added)
            .ok_or_else(|| overflow("hit_counter_total"))?;
        let triggered = total >= threshold;
        let after = if triggered { total - threshold } else { total };
        let deadline = if after > 0 {
            Some(
                now.checked_add(u64::from(duration_ms))
                    .ok_or_else(|| overflow("hit_counter_deadline"))?,
            )
        } else {
            None
        };
        let mut control_profile = crate::authoring::clone_shape::controlled_clone(profile)?;
        control_profile.kind = match kind {
            ThresholdControlKind::Stun => HitStatusKind::Stun,
            ThresholdControlKind::Root => HitStatusKind::Root,
        };
        for id in prior {
            self.end_sustained(id, now, SustainedEndReason::Replaced)?;
        }
        profile.kind = HitStatusKind::ThresholdControl {
            stacks: after,
            threshold,
            control: kind,
            control_ms,
        };
        control::poll("hit_counter_consumed")?;
        Ok(Some(PendingHitCounter {
            outcome: HitCounterOutcome {
                before,
                added,
                after,
                threshold,
                control: kind,
                triggered,
                control_applied: false,
                counter_expires_at_ms: deadline,
                control_expires_at_ms: None,
            },
            control_profile,
            control_ms,
        }))
    }

    pub(super) fn finish_hit_counter(
        &mut self,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        mut pending: PendingHitCounter,
        now: u64,
        side: Option<crate::game::enums::Side>,
        area_source: Option<Arc<super::super::area_observation::AreaEffectSource>>,
    ) -> Result<(), GameError> {
        let source = crate::authoring::clone_shape::controlled_clone(&self.runtime_fact_source)?;
        if pending.outcome.triggered {
            let prior_activation = self.sustained.next;
            self.begin_sustained_admitted(
                caster,
                target,
                SustainedEffectDef::HitStatus {
                    duration_ms: pending.control_ms,
                    profile: pending.control_profile,
                },
                now,
                None,
                side,
                area_source.clone(),
                None,
                true,
                false,
            )?;
            if self.sustained.next != prior_activation {
                pending.outcome.control_applied = true;
                pending.outcome.control_expires_at_ms = self
                    .sustained
                    .records
                    .get(&self.sustained.next)
                    .map(|r| r.expires_at);
            }
        }
        self.observe_sustained_change(
            &source,
            area_source.as_ref(),
            caster,
            target,
            now,
            super::super::authoring_observation::AppliedChange::HitCounter {
                outcome: pending.outcome,
            },
        )?;
        control::poll("hit_counter_finished")
    }
}
