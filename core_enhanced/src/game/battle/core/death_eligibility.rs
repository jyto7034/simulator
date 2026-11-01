use super::BattleCore;
use crate::game::{
    battle::{
        ids::UnitInstanceId,
        types::{BattleUnitRole, BattleUnitSourceIdentity},
    },
    behavior::GameError,
    enums::Side,
};

impl BattleCore {
    /// The scenario's admitted spawn identities are the encounter authority.
    /// Temporary objects cannot gain eligibility by copying an enemy base ID.
    pub(super) fn eligible_encounter_death(
        &self,
        victim: UnitInstanceId,
        source_side: Side,
    ) -> Result<bool, GameError> {
        let Some(unit) = self.units.get(&victim) else {
            return Ok(false);
        };
        if !unit.lifecycle.is_dead()
            || unit.owner == source_side
            || unit.role != BattleUnitRole::Combatant
            || matches!(
                unit.source_identity,
                BattleUnitSourceIdentity::IndependentObject { .. }
                    | BattleUnitSourceIdentity::SummonedUnit { .. }
                    | BattleUnitSourceIdentity::DefenseObject { .. }
            )
        {
            return Ok(false);
        }
        crate::authoring::control::charge(
            self.scenario_runtime.spawned.unit_refs.len() as u64,
            "encounter_kill_source",
        )?;
        Ok(self
            .scenario_runtime
            .spawned
            .unit_refs
            .values()
            .any(|id| *id == victim))
    }
}
