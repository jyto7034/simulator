//! A released basic attack owns one immutable tile until its finite impact.
use super::{
    damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
    BattleCore,
};
use crate::{
    authoring::{clone_shape::controlled_clone, control},
    game::{
        ability::SkillHitTargetFilter,
        battle::{
            damage::DamageSourceSnapshot, enums::BattleEvent, event_log::BattleLogEvent,
            ids::UnitInstanceId, runtime_control,
        },
        behavior::GameError,
        resources::Position,
    },
};
use uuid::Uuid;

impl BattleCore {
    pub(super) fn clear_basic_tile_impacts(&mut self, now: u64) -> Result<(), GameError> {
        while let Some(queued) = self.event_queue.take_basic_tile_impact()? {
            if queued.close_delivery_on_dispatch {
                let source = queued.fact_source.into_owned()?;
                self.close_source_resource_delivery(
                    source.resource_delivery().map(std::sync::Arc::as_ref),
                )?;
                if let Some(ticket) = source.delivery {
                    self.runtime_facts.close_delivery(&ticket, now)?;
                }
            }
        }
        control::poll("basic_tile_terminal_cleared")
    }

    pub(super) fn release_basic_delayed_tile(
        &mut self,
        target: UnitInstanceId,
        mut snapshot: DamageSourceSnapshot,
        delay_ms: u32,
        air_capable: bool,
        now: u64,
    ) -> Result<bool, GameError> {
        let Some(tile) = self
            .sample_unit_world_position_at(target, now)
            .map(|p| p.project_to_tile())
        else {
            return Ok(false);
        };
        let impact_at_ms = now
            .checked_add(u64::from(delay_ms))
            .ok_or_else(|| runtime_control::invariant("basic_delayed_tile_time_overflow"))?;
        let actor = snapshot.source_id;
        let release_id = snapshot.crit_roll_identity.source_instance_id;
        self.grant_basic_attack_release_resonance(actor, now);
        self.release_prepared_basic(&snapshot)?;
        self.consume_basic_overlay(actor, now)?;
        // The windup delivery is closed by the common release handler. This
        // child keeps action settlement and resource authority until impact.
        let source = self.reserve_finite_source(snapshot.fact_source.clone())?;
        snapshot.fact_source = source.clone();
        let release_seq = self.record_event_log(
            now,
            BattleLogEvent::BasicAttackTileReleased {
                release_id,
                attacker_instance_id: actor,
                tile,
                impact_at_ms,
            },
        );
        self.with_recording_cause(release_seq, |core| {
            core.event_queue.push_completion(
                BattleEvent::BasicAttackTileImpact {
                    time_ms: impact_at_ms,
                    release_id,
                    source_snapshot: snapshot,
                    tile,
                    air_capable,
                    cause: core.recording_cause().unwrap_or_default(),
                },
                None,
                source,
            );
        });
        control::poll("basic_delayed_tile_released")?;
        self.schedule_pending_autocasts(now);
        Ok(true)
    }

    pub(super) fn impact_basic_delayed_tile(
        &mut self,
        release_id: Uuid,
        mut snapshot: DamageSourceSnapshot,
        tile: Position,
        air_capable: bool,
        now: u64,
    ) -> Result<(), GameError> {
        runtime_control::admit_retained_growth(
            0,
            self.units
                .len()
                .checked_mul(std::mem::size_of::<UnitInstanceId>()),
            "basic_delayed_tile_targets",
        )?;
        let mut targets = self.collect_tile_area_targets_at(
            now,
            snapshot.source_side,
            snapshot.source_id,
            &[tile],
            SkillHitTargetFilter::Enemies,
            false,
        );
        targets.retain(|id| self.single_target_can_target_unit(*id, air_capable));
        control::poll("basic_delayed_tile_targets")?;
        let percent = self.area_damage_percent(snapshot.source_id, snapshot.source_side,
            &snapshot.fact_source, &targets, now)?;
        snapshot.modifiers.damage_amp_percent = snapshot.modifiers.damage_amp_percent.saturating_add(percent);
        let target_count = u32::try_from(targets.len())
            .map_err(|_| runtime_control::invariant("basic_delayed_tile_count_overflow"))?;
        let impact_seq = self.record_event_log(
            now,
            BattleLogEvent::BasicAttackTileImpacted {
                release_id,
                attacker_instance_id: snapshot.source_id,
                tile,
                target_count,
            },
        );
        self.with_recording_cause(impact_seq, |core| -> Result<(), GameError> {
            let mut actual_hp_loss = 0;
            for target_id in targets {
                control::charge(1, "basic_delayed_tile_hit")?;
                // A prior target's synchronous reaction can remove another
                // captured occupant. It cannot create a new target for this hit.
                if !core
                    .units
                    .get(&target_id)
                    .is_some_and(|unit| unit.is_active())
                {
                    continue;
                }
                let Some(summary) = core.execute_committed_damage(CommittedDamageInput {
                    target_id,
                    source_snapshot: controlled_clone(&snapshot)?,
                    committed_at_ms: now,
                    policy: CommittedDamagePolicy::BasicAttack {
                        occurrence_id: release_id,
                        occurrence_time_ms: now,
                    },
                }) else {
                    control::poll("basic_delayed_tile_damage")?;
                    return Err(runtime_control::invariant(
                        "basic_delayed_tile_damage_unavailable",
                    ));
                };
                Self::add_basic_hp_loss(&mut actual_hp_loss, summary.actual_hp_loss)?;
            }
            core.finish_basic_lifesteal(&snapshot, actual_hp_loss, now)?;
            core.schedule_pending_autocasts(now);
            control::poll("basic_delayed_tile_impacted")
        })
    }
}
