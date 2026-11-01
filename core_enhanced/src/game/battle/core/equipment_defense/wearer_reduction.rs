//! The current wearer owns this percentage; no hostile actor or timer is needed.
use super::*;
use crate::game::{
    battle::{damage::DamageSourceSnapshot, types::BattleUnitSourceIdentity},
    value_expr as expr,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct WearerReductionWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub target: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub time_ms: u64,
    pub strike: u64,
    pub current_hp: u32,
    pub max_hp: u32,
    pub percent: u32,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub incoming: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(WearerReductionWitness {
    generation,
    target,
    equipment,
    time_ms,
    strike,
    current_hp,
    max_hp,
    percent,
    definition_id,
    source_file,
    field_path,
    incoming
});
impl WearerReductionWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?
            .checked_add(self.incoming.retained_extent()?)?;
        if let Some(generation) = &self.generation {
            bytes = bytes
                .checked_add(std::mem::size_of_val(generation.as_ref()))?
                .checked_add(generation.semantic_sha256.capacity())?;
        }
        Some(bytes)
    }
}

impl BattleCore {
    pub(in crate::game::battle::core) fn evaluate_wearer_reduction(
        &mut self,
        target: UnitInstanceId,
        incoming: &DamageSourceSnapshot,
        now: u64,
        strike: u64,
    ) -> Result<i32, GameError> {
        if self.sustained.closed
            || !self
                .game_data
                .equipment_defense_catalog()
                .has_wearer_reduction
            || !self.units.get(&target).is_some_and(|u| {
                u.is_active()
                    && u.is_combatant()
                    && !u.corroded
                    && u.owner == Side::Player
                    && u.stats.current_health > 0
                    && matches!(u.source_identity, BattleUnitSourceIdentity::Employee { .. })
            })
        {
            return Ok(0);
        }
        // Compilation permits only Source HP reads. Reuse the common pure HP
        // expression evaluator without depending on an incoming actor's life.
        let evaluation =
            super::super::conditional_damage::Evaluation::new(self, target, target, now)?;
        let mut selected: Option<(EquipmentSourceSnapshot, u32)> = None;
        let mut result = Ok(());
        self.visit_equipment_defenses(target, |equipment, program| {
            let Some(plan) = &program.wearer_reduction else {
                return;
            };
            if result.is_err() {
                return;
            }
            result = (|| {
                let value = evaluation.amount(plan)?;
                if !(0..=99).contains(&value) {
                    return Err(runtime_control::reject_expression(expr::ValueExprFault {
                        code: expr::ValueExprFaultCode::Overflow,
                        site: plan.source().site.clone(),
                        node_index: 0,
                        expected_unit: Some(expr::ValueUnit::Scalar),
                        actual_unit: Some(expr::ValueUnit::Scalar),
                        phase: expr::ValueExprPhase::Narrow,
                        cause: "wearer damage reduction must be from 0 to 99 percent".into(),
                    }));
                }
                let percent = value as u32;
                if selected.is_none_or(|(old, amount)| {
                    percent > amount
                        || (percent == amount
                            && equipment.runtime_item_instance_id < old.runtime_item_instance_id)
                }) {
                    selected = Some((equipment, percent));
                }
                Ok(())
            })();
        })?;
        result?;
        control::poll("wearer_reduction_evaluated")?;
        let Some((equipment, percent)) = selected else {
            return Ok(0);
        };
        if self.authoring_observation.is_some() {
            let plan = self
                .game_data
                .equipment_defense_catalog()
                .by_equipment
                .get(&equipment.definition_uuid)
                .and_then(|p| p.wearer_reduction.as_ref())
                .ok_or_else(|| runtime_control::invariant("wearer_reduction_origin"))?;
            let source = plan.source();
            let extent = incoming.fact_source.retained_extent().and_then(|n| {
                n.checked_add(
                    size_of::<WearerReductionWitness>()
                        + source.site.action.definition_id.as_str().len()
                        + source.source_file.len()
                        + source.field_path.len(),
                )
            });
            runtime_control::admit_retained_growth(0, extent, "wearer_reduction_witness_copy")?;
            control::charge(
                extent.ok_or(GameError::InvalidAction)? as u64,
                "wearer_reduction_witness_copy",
            )?;
            let witness = WearerReductionWitness {
                generation: self.content_identity.clone(),
                target,
                equipment,
                time_ms: now,
                strike,
                current_hp: evaluation.owner_hp,
                max_hp: evaluation.owner_max,
                percent,
                definition_id: source.site.action.definition_id.to_string(),
                source_file: source.source_file.clone(),
                field_path: source.field_path.clone(),
                incoming: incoming.fact_source.observation(),
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "wearer_reduction_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_wearer_reduction = Some(witness);
        }
        Ok(percent as i32)
    }
}
