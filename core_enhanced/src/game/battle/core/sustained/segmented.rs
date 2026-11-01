//! Segments are a view over one shield balance, never a second stack counter.
use super::*;
use crate::game::{
    ability::{
        objects::ObjectWallCostDef,
        sustained::{SegmentDetachReceipt, SegmentedShieldView},
    },
    battle::runtime_control as native,
};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::game::battle::core) enum WallCostPlan {
    Segment {
        activation: u64,
        before: u32,
        detached: u32,
    },
    Health {
        amount: u32,
        wall_health: u32,
    },
}
impl WallCostPlan {
    pub(in crate::game::battle::core) fn wall_health(self) -> u32 {
        match self {
            Self::Segment { detached, .. } => detached,
            Self::Health { wall_health, .. } => wall_health,
        }
    }
}
fn matches_source(record: &Record, fragment: Option<Uuid>, equipment: Option<Uuid>) -> bool {
    record
        .source
        .fragment
        .as_ref()
        .map(|source| source.copy_uuid)
        == fragment
        && record
            .source
            .equipment
            .map(|source| source.runtime_item_instance_id)
            == equipment
}
impl BattleCore {
    fn segmented_pool_id(
        &self,
        owner: UnitInstanceId,
        pool_id: &str,
        fragment: Option<Uuid>,
        equipment: Option<Uuid>,
        now: u64,
    ) -> Result<Option<u64>, GameError> {
        let work = (self.sustained.records.len() as u64).saturating_mul(pool_id.len() as u64 + 1);
        control::charge(work, "segmented_pool_lookup")?;
        let mut found = None;
        for (id, record) in &self.sustained.records {
            if record.caster == owner
                && record.target == owner
                && record.expires_at > now
                && matches_source(record, fragment, equipment)
                && record
                    .segmented
                    .as_ref()
                    .is_some_and(|pool| pool.pool_id == pool_id)
                && record.shield.is_some_and(|pool| pool.remaining > 0)
            {
                if found.replace(*id).is_some() {
                    return Err(native::invariant("segmented_pool_identity"));
                }
            }
        }
        Ok(found)
    }
    pub(in crate::game::battle::core) fn validate_segmented_pool_available(
        &self,
        owner: UnitInstanceId,
        pool_id: &str,
        fragment: Option<Uuid>,
        equipment: Option<Uuid>,
        now: u64,
    ) -> Result<(), GameError> {
        if self
            .segmented_pool_id(owner, pool_id, fragment, equipment, now)?
            .is_some()
        {
            Err(GameError::InvalidAction)
        } else {
            Ok(())
        }
    }
    pub(super) fn segmented_pool_view(
        &self,
        record: &Record,
    ) -> Result<Option<SegmentedShieldView>, GameError> {
        let Some(definition) = &record.segmented else {
            return Ok(None);
        };
        let remaining = record
            .shield
            .ok_or_else(|| native::invariant("segmented_pool_balance"))?
            .remaining;
        let stacks = definition.stacks(remaining);
        Ok(Some(SegmentedShieldView {
            pool_id: crate::authoring::clone_shape::controlled_clone(&definition.pool_id)?,
            segment_health: definition.segment_health,
            max_segments: definition.max_segments,
            remaining_health: remaining,
            stacks,
            top_health: definition.top_health(remaining),
            block_bonus: stacks
                .checked_mul(definition.block_per_segment)
                .ok_or_else(|| overflow("segmented_block_bonus"))?,
        }))
    }
    pub(in crate::game::battle::core) fn effective_block_capacity(
        &self,
        owner: UnitInstanceId,
    ) -> u32 {
        if self.tactical_states.get(&owner).is_some_and(|state| {
            state.tile_kind == crate::game::battle::types::TacticalTileKind::Platform
                || state.activity == crate::game::battle::types::UnitExclusiveActivity::FieldWorking
        }) {
            return 0;
        }

        if self.sustained.segmented_count == 0 {
            return self.units.get(&owner).map_or(0, |unit| unit.block_capacity);
        }
        let calculate = || -> Result<u32, GameError> {
            let base = self.units.get(&owner).map_or(0, |unit| unit.block_capacity);
            control::charge(
                self.sustained.records.len() as u64,
                "segmented_block_capacity",
            )?;
            self.sustained
                .records
                .values()
                .filter(|record| record.target == owner)
                .try_fold(base, |sum, record| {
                    let Some(policy) = &record.segmented else {
                        return Ok(sum);
                    };
                    let pool = record
                        .shield
                        .ok_or_else(|| native::invariant("segmented_pool_balance"))?;
                    policy
                        .stacks(pool.remaining)
                        .checked_mul(policy.block_per_segment)
                        .and_then(|bonus| sum.checked_add(bonus))
                        .ok_or_else(|| overflow("segmented_block_capacity"))
                })
        };
        calculate().unwrap_or(0) // A latched limit/overflow aborts the enclosing candidate.
    }
    pub(in crate::game::battle::core) fn refresh_segmented_block_state(
        &mut self,
    ) -> Result<(), GameError> {
        let count = self.units.len();
        let work = count
            .checked_mul(count)
            .and_then(|n| n.checked_add(count))
            .ok_or_else(|| overflow("segmented_block_work"))?;
        control::charge(work as u64, "segmented_block_refresh")?;
        native::admit_retained_growth(0, count.checked_mul(1024), "segmented_block_buffers")?;
        let mut meter = crate::authoring::control::CloneMeter::default();
        meter.scan_storage(count.checked_mul(32))?;
        meter.finish("segmented_block_buffers", "segmented_block_buffer_copy")?;
        self.refresh_block_state();
        native::poll()?;
        control::poll("segmented_block_refreshed")
    }
    pub(in crate::game::battle::core) fn plan_object_wall_cost(
        &self,
        owner: UnitInstanceId,
        cost: &ObjectWallCostDef,
        fragment: Option<Uuid>,
        equipment: Option<Uuid>,
        now: u64,
    ) -> Result<WallCostPlan, GameError> {
        let (fragment, equipment) = match cost.pool_source {
            crate::game::ability::objects::ObjectPoolSource::CurrentEffect => (fragment, equipment),
            crate::game::ability::objects::ObjectPoolSource::EquippedWeapon => {
                let work = self.items.len() as u64;
                control::charge(work, "wall_cost_equipped_weapon")?;
                let mut weapon = None;
                for item in self
                    .items
                    .values()
                    .filter(|item| item.owner_unit_instance == owner)
                {
                    let definition = self
                        .game_data
                        .equipment_data
                        .get_by_uuid(&item.base_uuid)
                        .ok_or(GameError::InvalidAction)?;
                    if definition.equipment_type
                        == crate::game::data::equipment_data::EquipmentType::Weapon
                        && weapon.replace(item.instance_id).is_some()
                    {
                        return Err(GameError::InvalidAction);
                    }
                }
                // Absence means no weapon-owned pool, never a match to an
                // unattributed pool. Continue with the normal HP fallback.
                if weapon.is_none() {
                    return self.plan_object_wall_health_cost(owner, cost);
                }
                (None, weapon)
            }
        };
        if let Some(activation) =
            self.segmented_pool_id(owner, &cost.pool_id, fragment, equipment, now)?
        {
            let record = &self.sustained.records[&activation];
            let before = record.shield.ok_or(GameError::InvalidAction)?.remaining;
            let detached = record
                .segmented
                .as_ref()
                .ok_or(GameError::InvalidAction)?
                .top_health(before);
            return Ok(WallCostPlan::Segment {
                activation,
                before,
                detached,
            });
        }
        self.plan_object_wall_health_cost(owner, cost)
    }
    fn plan_object_wall_health_cost(
        &self,
        owner: UnitInstanceId,
        cost: &ObjectWallCostDef,
    ) -> Result<WallCostPlan, GameError> {
        let unit = self
            .units
            .get(&owner)
            .filter(|unit| unit.is_active())
            .ok_or(GameError::UnitNotFound)?;
        let amount = u32::try_from(
            u64::from(unit.stats.max_health) * u64::from(cost.health_cost_percent) / 100,
        )
        .map_err(|_| GameError::InvalidAction)?;
        let wall_health =
            u32::try_from(u64::from(amount) * u64::from(cost.wall_health_percent) / 100)
                .map_err(|_| GameError::InvalidAction)?;
        if amount == 0
            || amount >= unit.stats.current_health
            || amount > i32::MAX as u32
            || wall_health == 0
            || wall_health > i32::MAX as u32
        {
            return Err(GameError::InvalidAction);
        }
        Ok(WallCostPlan::Health {
            amount,
            wall_health,
        })
    }
    pub(in crate::game::battle::core) fn pay_object_wall_cost(
        &mut self,
        owner: UnitInstanceId,
        plan: WallCostPlan,
        now: u64,
    ) -> Result<u32, GameError> {
        match plan {
            WallCostPlan::Health {
                amount,
                wall_health,
            } => {
                self.pay_health_cost(owner, amount, false, now)?;
                Ok(wall_health)
            }
            WallCostPlan::Segment {
                activation,
                before,
                detached,
            } => {
                let record = self
                    .sustained
                    .records
                    .get(&activation)
                    .ok_or(GameError::InvalidAction)?;
                let policy = record.segmented.as_ref().ok_or(GameError::InvalidAction)?;
                if record.target != owner
                    || record.caster != owner
                    || record.expires_at <= now
                    || record.shield.is_none_or(|pool| pool.remaining != before)
                    || detached == 0
                    || detached != policy.top_health(before)
                {
                    return Err(GameError::InvalidAction);
                }
                native::admit_retained_growth(
                    0,
                    std::mem::size_of::<SegmentDetachReceipt>().checked_add(policy.pool_id.len()),
                    "segment_detach_receipt",
                )?;
                self.admit_authoring_runtime_replacement(
                    usize::from(self.sustained.latest_detachment.is_some()),
                    1,
                    "segment_detach_receipt",
                )?;
                let receipt = SegmentDetachReceipt {
                    activation,
                    owner,
                    pool_id: crate::authoring::clone_shape::controlled_clone(&policy.pool_id)?,
                    time_ms: now,
                    before,
                    detached,
                    after: before - detached,
                    fragment_copy_uuid: record
                        .source
                        .fragment
                        .as_ref()
                        .map(|source| source.copy_uuid),
                    equipment_runtime_uuid: record
                        .source
                        .equipment
                        .map(|source| source.runtime_item_instance_id),
                };
                self.sustained
                    .records
                    .get_mut(&activation)
                    .unwrap()
                    .shield
                    .as_mut()
                    .unwrap()
                    .remaining -= detached;
                self.sustained.latest_detachment = Some(receipt);
                if before == detached {
                    self.end_sustained(activation, now, SustainedEndReason::Depleted)?;
                }
                self.refresh_segmented_block_state()?;
                Ok(detached)
            }
        }
    }
}
