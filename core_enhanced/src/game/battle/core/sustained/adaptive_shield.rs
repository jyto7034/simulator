//! Adaptive equipment reuses the ordinary pool's balance, expiry and receipts.
use super::*;
use crate::game::battle::{damage::DamageType, runtime_control};

impl Record {
    pub(super) fn adaptive_source_valid(&self, core: &BattleCore) -> bool {
        self.shield.is_none_or(|s| s.damage_type.is_none())
            || self.source.equipment.is_some_and(|e| {
                core.adaptive_shield_copy_valid(self.caster, e.runtime_item_instance_id)
            })
    }
}
impl BattleCore {
    pub(in crate::game::battle::core) fn has_adaptive_shield(
        &self,
        owner: UnitInstanceId,
        copy: uuid::Uuid,
    ) -> Result<bool, GameError> {
        control::charge(
            self.sustained.records.len() as u64,
            "adaptive_shield_active_lookup",
        )?;
        Ok(self.sustained.records.values().any(|r| {
            r.caster == owner
                && r.target == owner
                && r.shield.is_some_and(|s| s.damage_type.is_some())
                && r.source
                    .equipment
                    .is_some_and(|e| e.runtime_item_instance_id == copy)
        }))
    }
    pub(in crate::game::battle::core) fn begin_adaptive_shield(
        &mut self,
        owner: UnitInstanceId,
        capacity: u32,
        damage_type: DamageType,
        duration_ms: u32,
        priority: i32,
        source: RuntimeFactSource,
        now: u64,
    ) -> Result<u64, GameError> {
        let before = self.sustained.next;
        let prior = std::mem::replace(&mut self.runtime_fact_source, source);
        let result = self.begin_sustained_owned_with_shield(
            owner,
            owner,
            SustainedEffectDef::IndependentShield {
                amount: capacity,
                duration_ms,
                priority,
            },
            now,
            None,
            None,
            None,
            Some(capacity),
        );
        self.runtime_fact_source = prior;
        result?;
        if self.sustained.next == before {
            return Err(runtime_control::invariant("adaptive_shield_start"));
        }
        let activation = self.sustained.next;
        self.sustained
            .records
            .get_mut(&activation)
            .unwrap()
            .shield
            .as_mut()
            .unwrap()
            .damage_type = Some(damage_type);
        Ok(activation)
    }
}
