//! A finite basic action owns its homing members and one aggregate settlement.
use super::{basic_attack_projectile::ProjectileLaunch, types::ProjectileGuidance, BattleCore};
use crate::{
    authoring::{clone_shape::controlled_clone, control},
    game::{
        battle::{damage::DamageSourceSnapshot, ids::UnitInstanceId, runtime_control},
        behavior::GameError,
    },
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub(super) struct BasicProjectileBatch {
    pending: u32,
    actual_hp_loss: u64,
    snapshot: DamageSourceSnapshot,
}
crate::authoring::clone_shape::struct_shape!(BasicProjectileBatch {
    pending,
    actual_hp_loss,
    snapshot,
});

// Same conservative eleven-slot/five-key node bound as the runtime fact trees.
// Include an empty root after the final removal, so churn never hides capacity.
pub(super) fn batch_tree_extent(len: usize) -> Option<usize> {
    let nodes = 1usize.checked_add(len.saturating_sub(1) / 5)?;
    nodes.checked_mul(
        std::mem::size_of::<(Uuid, BasicProjectileBatch)>()
            .checked_mul(11)?
            .checked_add(12 * std::mem::size_of::<usize>())?
            .checked_add(128)?,
    )
}
impl BasicProjectileBatch {
    pub(super) fn retained_extent(&self) -> Option<usize> {
        self.snapshot.fact_retained_extent()
    }
}

impl BattleCore {
    pub(super) fn release_basic_projectile_targets(
        &mut self,
        actor: UnitInstanceId,
        primary: UnitInstanceId,
        max_targets: u32,
        air_capable: bool,
        speed_units_per_ms: u32,
        snapshot: DamageSourceSnapshot,
        now: u64,
    ) -> Result<bool, GameError> {
        let Some(unit) = self.units.get(&actor) else {
            return Ok(false);
        };
        let profile = unit.basic_attack.targeting_profile;
        let owner = unit.owner;
        let forced = self.taunt_source(actor);
        runtime_control::admit_retained_growth(
            0,
            self.units
                .len()
                .checked_mul(std::mem::size_of::<UnitInstanceId>()),
            "basic_projectile_target_storage",
        )?;
        let mut targets = Vec::with_capacity(self.units.len());
        targets.extend(
            self.authoring_target_candidates()?
                .filter(|u| u.is_active() && u.owner != owner)
                .map(|u| u.instance_id)
                .filter(|id| forced.is_none_or(|forced| forced == *id))
                .filter(|id| self.is_basic_attack_target_in_range_at(actor, *id, now))
                .filter(|id| self.is_basic_attack_useful_target(actor, *id)),
        );
        control::charge(
            (targets.len() as u64)
                .saturating_mul(u64::from(usize::BITS - targets.len().leading_zeros()) + 2),
            "basic_projectile_target_order",
        )?;
        targets.sort_unstable_by(|a, b| {
            (*b == primary)
                .cmp(&(*a == primary))
                .then_with(|| {
                    self.equipment_prioritizes_cohort(actor, *b)
                        .cmp(&self.equipment_prioritizes_cohort(actor, *a))
                })
                .then_with(|| self.compare_targeting_profile_candidate(profile, *a, *b))
        });
        targets.truncate(max_targets as usize);
        if targets.first() != Some(&primary) {
            return Ok(false);
        }
        let Some(origin) = self.sample_unit_world_position_at(actor, now) else {
            return Ok(false);
        };
        let batch_id = snapshot.crit_roll_identity.source_instance_id;
        let batch = BasicProjectileBatch {
            pending: targets.len() as u32,
            actual_hp_loss: 0,
            snapshot: controlled_clone(&snapshot)?,
        };
        runtime_control::admit_retained_growth(
            0,
            batch.retained_extent().and_then(|payload| {
                batch_tree_extent(self.basic_projectile_batches.len().checked_add(1)?)?
                    .checked_sub(batch_tree_extent(self.basic_projectile_batches.len())?)?
                    .checked_add(payload)
            }),
            "basic_projectile_batch",
        )?;
        if self.basic_projectile_batches.contains_key(&batch_id) {
            return Err(runtime_control::invariant(
                "duplicate_basic_projectile_batch",
            ));
        }
        self.basic_projectile_batches.insert(batch_id, batch);
        for target in targets {
            control::charge(1, "basic_projectile_member")?;
            let aim = self
                .sample_unit_world_position_at(target, now)
                .ok_or_else(|| runtime_control::invariant("basic_projectile_target_position"))?;
            self.spawn_basic_attack_projectile(ProjectileLaunch {
                area: None,
                batch_id: Some(batch_id),
                spatial: None,
                fired_at_ms: now,
                attacker_instance_id: actor,
                attacker_owner_at_launch: owner,
                air_capable_at_launch: air_capable,
                target_instance_id: target,
                attacker_origin: origin,
                target_aim: aim,
                speed_units_per_ms,
                guidance: ProjectileGuidance::Homing,
                damage_type: snapshot.damage_type,
                source_snapshot: controlled_clone(&snapshot)?,
            });
            control::poll("basic_projectile_member_released")?;
        }
        Ok(true)
    }

    pub(super) fn settle_basic_projectile_member(
        &mut self,
        id: Uuid,
        actual_hp_loss: u64,
        now: u64,
    ) -> Result<(), GameError> {
        let batch = self
            .basic_projectile_batches
            .get_mut(&id)
            .ok_or_else(|| runtime_control::invariant("basic_projectile_batch_missing"))?;
        Self::add_basic_hp_loss(&mut batch.actual_hp_loss, actual_hp_loss)?;
        batch.pending = batch
            .pending
            .checked_sub(1)
            .ok_or_else(|| runtime_control::invariant("basic_projectile_batch_underflow"))?;
        if batch.pending != 0 {
            return Ok(());
        }
        let batch = self.basic_projectile_batches.remove(&id).unwrap();
        self.finish_basic_lifesteal(&batch.snapshot, batch.actual_hp_loss, now)?;
        if let Some(ticket) = batch.snapshot.fact_source.delivery {
            self.runtime_facts.close_delivery(&ticket, now)?;
        }
        control::poll("basic_projectile_batch_settled")
    }

    pub(super) fn clear_basic_projectile_batches(&mut self, now: u64) -> Result<(), GameError> {
        // Terminal cleanup never converts unfinished damage into lifesteal.
        runtime_control::admit_retained_growth(
            0,
            self.basic_projectile_batches
                .len()
                .checked_mul(std::mem::size_of::<Uuid>()),
            "basic_projectile_batch_clear_storage",
        )?;
        let mut ids: Vec<_> = self.basic_projectile_batches.keys().copied().collect();
        ids.sort_unstable();
        for id in ids {
            control::charge(1, "basic_projectile_batch_clear")?;
            let batch = self.basic_projectile_batches.remove(&id).unwrap();
            if let Some(ticket) = batch.snapshot.fact_source.delivery {
                self.runtime_facts.close_delivery(&ticket, now)?;
            }
        }
        self.projectiles.retain(|_, p| p.batch_id.is_none());
        Ok(())
    }
}
