use super::BattleCore;
use crate::game::{
    battle::{
        damage::{DamageSource, DamageSourceSnapshot},
        ids::UnitInstanceId,
        runtime_facts::EquipmentSourceSnapshot,
        types::{BattleUnitRole, BattleUnitSourceIdentity, BattleUnitThreatClass},
    },
    behavior::GameError,
    resource::{ResourceMutation, ResourceOperation, ResourceOwnerSelector, WeaponKillGrowthDef},
};
use std::sync::Arc;

/// Captured once at the admitted direct basic attack. Secondary hits retain
/// the primary identity and the pre-kill balance, including released carriers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WeaponGrowthCapture {
    pub primary: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub definition: WeaponKillGrowthDef,
    pub balance: u32,
    pub multiplier_copy: Option<uuid::Uuid>,
}
crate::authoring::clone_shape::struct_shape!(WeaponGrowthCapture {
    primary,
    equipment,
    definition,
    balance,
    multiplier_copy
});
impl WeaponGrowthCapture {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(32)?
            .checked_add(self.definition.resource.retained_text_capacity())?
            .checked_add(
                self.definition
                    .multiplier
                    .as_ref()
                    .map_or(0, |m| m.relic_definition.capacity()),
            )
    }
}
impl BattleCore {
    pub(super) fn bind_weapon_growth(
        &mut self,
        snapshot: &mut DamageSourceSnapshot,
        primary: UnitInstanceId,
    ) -> Result<(), GameError> {
        if self.game_data.resource_data.resources.is_empty()
            || snapshot.source != DamageSource::BasicAttack
            || snapshot.fact_source.reaction
        {
            return Ok(());
        }
        let Some(unit) = self.units.get(&snapshot.source_id).filter(|u| !u.corroded) else {
            return Ok(());
        };
        // Ordinary employees have a fixed slot projection. Legacy scenarios
        // can omit it, in which case the bounded scan uses actual runtime items.
        let slots = unit.equipment_slots.as_ref();
        let scan = if slots.is_some() { 3 } else { self.items.len() };
        crate::authoring::control::charge(scan as u64, "weapon_growth_provider_scan")?;
        let slot_items = slots
            .into_iter()
            .flat_map(|s| s.slots.iter().flatten())
            .map(|s| {
                self.items
                    .get(&s.runtime_item_instance_id)
                    .ok_or(GameError::InvalidAction)
            });
        let legacy_items = self
            .items
            .values()
            .take(if slots.is_some() { 0 } else { self.items.len() })
            .filter(|i| i.owner_unit_instance == snapshot.source_id)
            .map(Ok);
        let items = slot_items.chain(legacy_items);
        let mut bound = None;
        for item in items {
            let item = item?;
            if item.owner_unit_instance != snapshot.source_id {
                return Err(GameError::InvalidAction);
            }
            let Some(definition) = self
                .game_data
                .equipment_data
                .get_by_uuid(&item.base_uuid)
                .and_then(|m| m.weapon_profile.as_ref())
                .and_then(|p| p.kill_growth.as_ref())
            else {
                continue;
            };
            if bound.is_some() {
                return Err(GameError::InvalidAction);
            }
            crate::authoring::control::charge_serialized(definition, "weapon_growth_capture")?;
            bound = Some((item.source_snapshot(), definition.clone()));
        }
        let Some((equipment, definition)) = bound else {
            return Ok(());
        };
        self.prepare_runtime_resources()?;
        let mut source = snapshot.fact_source.clone();
        source.equipment = Some(equipment);
        let lease = self
            .resolve_typed_resource_lease(
                &definition.resource,
                ResourceOwnerSelector::SourceEquipment,
                &source,
                None,
            )
            .map_err(|e| e.into_game_error())?;
        let balance = self
            .runtime_facts
            .resources
            .as_ref()
            .ok_or(GameError::InvalidAction)?
            .store
            .value(&lease)
            .map_err(|e| e.into_game_error())?
            .balance;
        let multiplier_copy = definition
            .multiplier
            .as_ref()
            .and_then(|m| self.equipped_normal_relic_copy(snapshot.source_id, &m.relic_definition));
        let capture = WeaponGrowthCapture {
            primary,
            equipment,
            definition,
            balance,
            multiplier_copy,
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            capture.retained_extent(),
            "weapon_growth_capture_storage",
        )?;
        snapshot.weapon_growth = Some(Arc::new(capture));
        crate::authoring::control::poll("weapon_growth_bound")
    }
    pub(super) fn apply_weapon_growth_damage(
        &self,
        snapshot: &mut DamageSourceSnapshot,
        target: UnitInstanceId,
    ) -> Result<(), GameError> {
        let Some(capture) = &snapshot.weapon_growth else {
            return Ok(());
        };
        if snapshot.source != DamageSource::BasicAttack || snapshot.fact_source.reaction {
            return Ok(());
        }
        let percent = if target == capture.primary {
            100
        } else {
            capture.definition.secondary_percent
        };
        let bonus = u128::from(capture.balance)
            * u128::from(capture.definition.damage_per_stack)
            * u128::from(percent)
            / 100;
        snapshot.base_damage =
            (u128::from(snapshot.base_damage) + bonus).min(u128::from(u32::MAX)) as u32;
        crate::authoring::control::poll("weapon_growth_damage")
    }
    pub(super) fn weapon_growth_kill(
        &mut self,
        snapshot: &DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(capture) = &snapshot.weapon_growth else {
            return Ok(());
        };
        if target != capture.primary
            || snapshot.source != DamageSource::BasicAttack
            || snapshot.fact_source.reaction
        {
            return Ok(());
        }
        let Some(victim) = self.units.get(&target) else {
            return Ok(());
        };
        // Scenario membership is the existing encounter spawn authority. An
        // independent/summoned object cannot become eligible by sharing a base ID.
        crate::authoring::control::charge(
            self.scenario_runtime.spawned.unit_refs.len() as u64,
            "weapon_growth_encounter_source",
        )?;
        if !victim.lifecycle.is_dead()
            || victim.owner == snapshot.source_side
            || victim.role != BattleUnitRole::Combatant
            || matches!(
                victim.source_identity,
                BattleUnitSourceIdentity::IndependentObject { .. }
                    | BattleUnitSourceIdentity::SummonedUnit { .. }
                    | BattleUnitSourceIdentity::DefenseObject { .. }
            )
            || !self
                .scenario_runtime
                .spawned
                .unit_refs
                .values()
                .any(|id| *id == target)
        {
            return Ok(());
        }
        let mut amount = match victim.threat_class {
            BattleUnitThreatClass::Boss => capture.definition.abnormality,
            BattleUnitThreatClass::Elite => capture.definition.elite,
            BattleUnitThreatClass::Normal => capture.definition.normal,
        };
        if capture.multiplier_copy.is_some() {
            amount = amount.saturating_mul(
                capture
                    .definition
                    .multiplier
                    .as_ref()
                    .ok_or(GameError::InvalidAction)?
                    .factor,
            );
        }
        let mut source = snapshot.fact_source.clone();
        source.equipment = Some(capture.equipment);
        let prior = std::mem::replace(&mut self.runtime_fact_source, source);
        let result = self.apply_typed_resource_operations(
            &[ResourceOperation {
                resource: capture.definition.resource.clone(),
                owner: ResourceOwnerSelector::SourceEquipment,
                operation: ResourceMutation::Grant(amount),
            }],
            &[],
            now,
        );
        self.runtime_fact_source = prior;
        result?;
        crate::authoring::control::poll("weapon_growth_awarded")
    }
}
