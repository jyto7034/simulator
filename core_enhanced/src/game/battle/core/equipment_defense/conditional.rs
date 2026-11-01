//! Current hostile actor conditions; the target strike owns the admitted result.
use super::super::conditional_damage::{
    ConditionalDamageContribution, ConditionalDamageWitness, Evaluation,
};
use super::*;
use crate::game::{
    battle::{
        damage::{DamageSource, DamageSourceSnapshot},
        types::BattleUnitSourceIdentity,
    },
    value_expr as expr,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct IncomingConditionalWitness {
    #[serde(flatten)]
    pub evaluation: ConditionalDamageWitness,
    pub equipment: EquipmentSourceSnapshot,
    pub incoming: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(IncomingConditionalWitness {
    evaluation,
    equipment,
    incoming
});
impl IncomingConditionalWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        size_of::<Self>()
            .checked_add(self.evaluation.retained_extent()?)?
            .checked_add(self.incoming.retained_extent()?)
    }
}
impl BattleCore {
    pub(in crate::game::battle::core) fn incoming_conditional_eligible(
        &self,
        target: UnitInstanceId,
        source: &DamageSourceSnapshot,
    ) -> bool {
        !self.sustained.closed
            && self
                .game_data
                .equipment_defense_catalog()
                .has_conditional_reduction
            && (source.source != DamageSource::Environment || source.fact_source.reaction)
            && source.source_id != target
            && self.units.get(&target).is_some_and(|wearer| {
                wearer.is_active()
                    && wearer.is_combatant()
                    && !wearer.corroded
                    && wearer.owner == Side::Player
                    && wearer.stats.current_health > 0
                    && matches!(
                        wearer.source_identity,
                        BattleUnitSourceIdentity::Employee { .. }
                    )
                    && self.units.get(&source.source_id).is_some_and(|actor| {
                        actor.is_active()
                            && actor.is_combatant()
                            && actor.stats.current_health > 0
                            && actor.owner != wearer.owner
                            && actor.owner == source.source_side
                    })
            })
    }

    pub(in crate::game::battle::core) fn incoming_conditional_percent(
        &mut self,
        target: UnitInstanceId,
        source: &DamageSourceSnapshot,
        now: u64,
    ) -> Result<i32, GameError> {
        let mut evaluation = Evaluation::new(self, target, source.source_id, now)?;
        let observing = self.authoring_observation.is_some();
        let mut contributors = Vec::new();
        let mut selected: Option<(EquipmentSourceSnapshot, u32)> = None;
        let mut result = Ok(());
        self.visit_equipment_defenses(target, |equipment, program| {
            let Some(policy) = &program.conditional_reduction else {
                return;
            };
            if result.is_err() {
                return;
            }
            result = (|| {
                let matched = evaluation.matches(self, &policy.condition)?;
                let raw = if matched {
                    evaluation.amount(&policy.percent)?
                } else {
                    0
                };
                if !(0..=99).contains(&raw) {
                    return Err(runtime_control::reject_expression(expr::ValueExprFault {
                        code: expr::ValueExprFaultCode::Overflow,
                        site: policy.percent.source().site.clone(),
                        node_index: 0,
                        expected_unit: Some(expr::ValueUnit::Scalar),
                        actual_unit: Some(expr::ValueUnit::Scalar),
                        phase: expr::ValueExprPhase::Narrow,
                        cause: "conditional damage reduction must be from 0 to 99 percent".into(),
                    }));
                }
                let percent = raw as u32;
                if selected.is_none_or(|(prior, value)| {
                    percent > value
                        || (percent == value
                            && equipment.runtime_item_instance_id < prior.runtime_item_instance_id)
                }) {
                    selected = Some((equipment, percent));
                }
                if observing {
                    let origin = policy.percent.source();
                    let bytes = size_of::<ConditionalDamageContribution>()
                        .checked_add(origin.site.action.definition_id.as_str().len())
                        .and_then(|n| n.checked_add(origin.source_file.len()))
                        .and_then(|n| n.checked_add(origin.field_path.len()));
                    runtime_control::admit_retained_growth(
                        0,
                        bytes,
                        "conditional_reduction_witness_copy",
                    )?;
                    control::charge(
                        bytes.ok_or(GameError::InvalidAction)? as u64,
                        "conditional_reduction_witness_copy",
                    )?;
                    contributors.reserve_exact(1);
                    contributors.push(ConditionalDamageContribution {
                        equipment,
                        matched,
                        eligible: true,
                        percent,
                        definition_id: origin.site.action.definition_id.to_string(),
                        source_file: origin.source_file.clone(),
                        field_path: origin.field_path.clone(),
                    });
                }
                Ok(())
            })();
        })?;
        result?;
        let Some((equipment, percent)) = selected else {
            return Ok(0);
        };
        if observing {
            runtime_control::admit_retained_growth(
                0,
                source.fact_source.retained_extent(),
                "conditional_reduction_incoming_copy",
            )?;
            let witness = IncomingConditionalWitness {
                evaluation: ConditionalDamageWitness {
                    generation: self.content_identity.clone(),
                    time_ms: now,
                    actor: source.source_id,
                    wearer: target,
                    target: source.source_id,
                    wearer_hp: evaluation.owner_hp,
                    wearer_max_hp: evaluation.owner_max,
                    target_hp: evaluation.target_hp,
                    target_max_hp: evaluation.target_max,
                    target_movement_impaired: evaluation.movement_impaired,
                    distance_squared: evaluation.distance_squared,
                    percent,
                    physical_percent: percent,
                    magic_percent: percent,
                    true_percent: percent,
                    contributors,
                },
                equipment,
                incoming: source.fact_source.observation(),
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "conditional_reduction_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_conditional_reduction = Some(witness);
        }
        control::poll("conditional_reduction_evaluated")?;
        Ok(percent as i32)
    }
}
