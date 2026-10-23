//! Battle-lifetime stacks owned by an actual weapon copy, independent of deployment.
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        ability::{marks::MarkRatio, weapon_marks::MarkWeaponDef},
        battle::{
            damage::{DamageSource, DamageSourceSnapshot},
            ids::UnitInstanceId,
            runtime_facts::{ActionHandle, EquipmentSourceSnapshot},
        },
        behavior::GameError,
    },
};
use std::{collections::BTreeMap, sync::Arc};
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WeaponMarkCapture {
    equipment: EquipmentSourceSnapshot,
    definition: MarkWeaponDef,
    chance: MarkRatio,
    amount: u64,
}
crate::authoring::clone_shape::struct_shape!(WeaponMarkCapture {
    equipment,
    definition,
    chance,
    amount
});
impl WeaponMarkCapture {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(32)?
            .checked_add(self.definition.retained_dynamic_extent()?)
    }
    fn key(&self) -> uuid::Uuid {
        self.equipment
            .persistent_copy_uuid
            .unwrap_or(self.equipment.runtime_item_instance_id)
    }
}
#[derive(Debug, Clone, Default)]
pub(super) struct WeaponStackPool {
    pub units: BTreeMap<UnitInstanceId, u64>,
    pub tiles: BTreeMap<(i32, i32), u64>,
}
crate::authoring::clone_shape::struct_shape!(WeaponStackPool { units, tiles });
#[derive(Debug, Clone, Default)]
pub(super) struct WeaponMarkState {
    pub pools: BTreeMap<(uuid::Uuid, String), WeaponStackPool>,
    attempts: Vec<(ActionHandle, UnitInstanceId, uuid::Uuid)>,
}
crate::authoring::clone_shape::struct_shape!(WeaponMarkState { pools, attempts });
impl WeaponMarkState {
    pub(super) fn retained_extent(&self) -> Option<usize> {
        let mut n = self.attempts.capacity().checked_mul(std::mem::size_of::<(
            ActionHandle,
            UnitInstanceId,
            uuid::Uuid,
        )>())?;
        for ((_, key), pool) in &self.pools {
            n = n
                .checked_add(160)?
                .checked_add(key.capacity())?
                .checked_add(pool.units.len().checked_mul(96)?)?
                .checked_add(pool.tiles.len().checked_mul(80)?)?;
        }
        Some(n)
    }
    pub(super) fn entity_count(&self) -> usize {
        self.attempts.len()
            + self.pools.len()
            + self
                .pools
                .values()
                .map(|p| p.units.len() + p.tiles.len())
                .sum::<usize>()
    }
}
impl BattleCore {
    pub(super) fn bind_weapon_marks(
        &self,
        snapshot: &mut DamageSourceSnapshot,
    ) -> Result<(), GameError> {
        if snapshot.source != DamageSource::BasicAttack || snapshot.fact_source.reaction {
            return Ok(());
        }
        if !self
            .units
            .get(&snapshot.source_id)
            .is_some_and(|u| u.is_active() && !u.corroded)
        {
            return Ok(());
        }
        control::charge(self.items.len() as u64, "weapon_marks_provider_scan")?;
        let mut bound = None;
        for item in self
            .items
            .values()
            .filter(|i| i.owner_unit_instance == snapshot.source_id)
        {
            let Some(def) = self
                .game_data
                .equipment_data
                .get_by_uuid(&item.base_uuid)
                .and_then(|m| m.weapon_profile.as_ref())
                .and_then(|p| p.mark_weapon.as_ref())
            else {
                continue;
            };
            if bound.is_some() {
                return Err(GameError::InvalidAction);
            }
            let hit = def.hit(item.weapon_stage);
            let mut factor = hit.multiplier;
            for m in &def.modifiers {
                if self
                    .equipped_normal_relic_copy(snapshot.source_id, &m.relic_definition)
                    .is_some()
                {
                    factor = super::marks::ratio_product(factor, m.multiplier)?;
                }
            }
            let amount = u128::from(hit.amount)
                .checked_mul(u128::from(factor.numerator))
                .ok_or(GameError::InvalidAction)?;
            // A stack is indivisible; defer rounding until all actual equipment factors apply.
            let amount = u64::try_from(amount.div_ceil(u128::from(factor.denominator)))
                .map_err(|_| GameError::InvalidAction)?;
            control::charge_serialized(def, "weapon_marks_capture")?;
            let capture = WeaponMarkCapture {
                equipment: item.source_snapshot(),
                definition: def.clone(),
                chance: hit.chance_percent,
                amount,
            };
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                capture.retained_extent(),
                "weapon_marks_capture",
            )?;
            bound = Some(Arc::new(capture));
        }
        snapshot.weapon_mark_capture = bound;
        Ok(())
    }
    pub(super) fn weapon_mark_hit(
        &mut self,
        snapshot: &DamageSourceSnapshot,
        target: UnitInstanceId,
    ) -> Result<(), GameError> {
        if snapshot.source != DamageSource::BasicAttack || snapshot.fact_source.reaction {
            return Ok(());
        }
        let (Some(c), Some(action)) = (&snapshot.weapon_mark_capture, &snapshot.fact_source.action)
        else {
            return Ok(());
        };
        if !self.mark_target_eligible(snapshot.source_side, c.definition.filter, target) {
            return Ok(());
        }
        let copy = c.key();
        let state = &mut self.sustained.weapon_marks;
        control::charge(state.attempts.len() as u64, "weapon_marks_dedup")?;
        state
            .attempts
            .retain(|(a, _, _)| self.runtime_facts.action_lifetime(a).is_some());
        if state
            .attempts
            .iter()
            .any(|(a, t, w)| a == action && *t == target && *w == copy)
        {
            return Ok(());
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(512 + c.definition.key.len()),
            "weapon_mark_grant",
        )?;
        let mut seed = self.seed ^ action.ordinal().0.rotate_left(23);
        for b in copy.as_bytes().iter().chain(target.as_bytes()) {
            seed = seed.wrapping_mul(0x100000001b3) ^ u64::from(*b);
        }
        seed = (seed ^ (seed >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        seed = (seed ^ (seed >> 27)).wrapping_mul(0x94d049bb133111eb);
        seed ^= seed >> 31;
        if u128::from(seed)
            >= (u128::from(c.chance.numerator) * (u128::from(u64::MAX) + 1))
                / (100 * u128::from(c.chance.denominator))
        {
            state.attempts.push((action.clone(), target, copy));
            return Ok(());
        }
        let old = state
            .pools
            .get(&(copy, c.definition.key.clone()))
            .and_then(|p| p.units.get(&target))
            .copied()
            .unwrap_or(0);
        let next = old.checked_add(c.amount).ok_or_else(stack_overflow)?;
        state.attempts.push((action.clone(), target, copy));
        let pool = state
            .pools
            .entry((copy, c.definition.key.clone()))
            .or_default();
        let value = pool.units.entry(target).or_default();
        *value = next;
        control::poll("weapon_mark_granted")
    }
    pub(super) fn weapon_mark_death(&mut self, target: UnitInstanceId, now: u64) -> Result<(), GameError> {
        let Some(tile) = self.sample_unit_body_at(target, now).map(|b| b.projected_tile()) else {
            return Ok(());
        };
        control::charge(
            self.sustained.weapon_marks.pools.len() as u64,
            "weapon_mark_death_scan",
        )?;
        for pool in self.sustained.weapon_marks.pools.values_mut() {
            if let Some(amount) = pool.units.get(&target).copied() {
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    Some(80),
                    "weapon_mark_death_tile",
                )?;
                let value = pool.tiles.entry((tile.x, tile.y)).or_default();
                *value = value.checked_add(amount).ok_or_else(stack_overflow)?;
                pool.units.remove(&target);
            }
        }
        control::poll("weapon_mark_death_transferred")
    }
}

impl BattleCore {
    pub fn weapon_marks_snapshot(
        &self,
    ) -> Result<crate::game::ability::weapon_marks::WeaponMarksSnapshot, GameError> {
        use crate::game::ability::weapon_marks::{WeaponMarkStoreSnapshot, WeaponMarksSnapshot};
        control::charge(
            self.sustained.weapon_marks.entity_count() as u64,
            "weapon_marks_snapshot",
        )?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.sustained.weapon_marks.retained_extent(),
            "weapon_marks_snapshot_storage",
        )?;
        Ok(WeaponMarksSnapshot {
            stores: self
                .sustained
                .weapon_marks
                .pools
                .iter()
                .map(|((copy, key), p)| WeaponMarkStoreSnapshot {
                    weapon_copy: *copy,
                    key: key.clone(),
                    units: p.units.iter().map(|(id, n)| (*id, *n)).collect(),
                    tiles: p
                        .tiles
                        .iter()
                        .map(|((x, y), n)| (crate::game::resources::Position::new(*x, *y), *n))
                        .collect(),
                })
                .collect(),
        })
    }
    pub(super) fn reconcile_weapon_marks(&mut self) -> Result<(), GameError> {
        control::charge(
            self.sustained.weapon_marks.entity_count() as u64,
            "weapon_marks_reconcile",
        )?;
        self.sustained
            .weapon_marks
            .attempts
            .retain(|(a, _, _)| self.runtime_facts.action_lifetime(a).is_some());
        // Escape/despawn removes the inaccessible unit's marks. A real death
        // has already transferred them at its canonical death boundary.
        for pool in self.sustained.weapon_marks.pools.values_mut() {
            pool.units
                .retain(|id, _| self.units.get(id).is_some_and(|u| u.is_active()));
        }
        Ok(())
    }
}

fn stack_overflow() -> GameError {
    crate::game::battle::runtime_control::reject(
        crate::game::battle::runtime_control::BattleRuntimeFault {
            code: crate::game::battle::runtime_control::RuntimeFaultCode::CounterOverflow,
            site: "weapon_stack_overflow",
            limit: None,
            requested: None,
        },
    )
}
