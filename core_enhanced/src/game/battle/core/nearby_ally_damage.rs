//! Live outgoing amplification derived from other employees near the wearer.
//! It never changes Attack or captures a neighbor count in a released carrier.
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        battle::{
            damage::DamageSourceSnapshot,
            ids::UnitInstanceId,
            runtime_control,
            runtime_facts::EquipmentSourceSnapshot,
            tile_range::{TileRangePolicy, TileReachQuery},
            types::BattleUnitSourceIdentity,
        },
        behavior::GameError,
        enums::Side,
        value_expr::{self as expr, EvalFrame, EvalMember, ExprMeter},
    },
};
use std::{
    collections::BTreeMap,
    mem::{size_of, size_of_val},
    sync::Arc,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct NearbyAllyDamageContribution {
    pub equipment: EquipmentSourceSnapshot,
    pub count: u32,
    pub percent: u32,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(NearbyAllyDamageContribution {
    equipment,
    count,
    percent,
    definition_id,
    source_file,
    field_path
});
#[derive(Debug, Clone, serde::Serialize)]
pub struct NearbyAllyDamageWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub wearer: UnitInstanceId,
    pub target: UnitInstanceId,
    pub percent: u32,
    pub contributors: Vec<NearbyAllyDamageContribution>,
}
crate::authoring::clone_shape::struct_shape!(NearbyAllyDamageWitness {
    generation,
    time_ms,
    actor,
    wearer,
    target,
    percent,
    contributors
});
impl NearbyAllyDamageWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>().checked_add(
            self.contributors
                .capacity()
                .checked_mul(size_of::<NearbyAllyDamageContribution>())?,
        )?;
        if let Some(g) = &self.generation {
            bytes = bytes
                .checked_add(size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        for c in &self.contributors {
            bytes = bytes
                .checked_add(c.definition_id.capacity())?
                .checked_add(c.source_file.capacity())?
                .checked_add(c.field_path.capacity())?;
        }
        Some(bytes)
    }
}
impl BattleCore {
    pub(super) fn nearby_ally_damage_percent(
        &mut self,
        snapshot: &DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<i32, GameError> {
        if !self.game_data.equipment_attack_catalog().has_nearby_ally_damage { return Ok(0); }
        let Some(actor) = self.owned_equipment_damage_source(snapshot, target) else { return Ok(0); };
        let providers =
            self.equipped_attack_providers(actor, |p| p.nearby_ally_damage.is_some())?;
        if providers.is_empty() {
            return Ok(0);
        }
        let wearer = self.units.get(&actor).ok_or(GameError::InvalidAction)?;
        let facing = wearer.facing_direction;
        let anchor = self
            .sample_unit_world_position_at(actor, now)
            .map(|p| p.project_to_tile());
        let mut total = 0u32;
        let mut contributors = Vec::new();
        for (equipment, program) in providers {
            let policy = program.nearby_ally_damage.as_ref().unwrap();
            Self::charge_authoring_tile_pattern(&policy.range, self.units.len() as u64 + 1)?;
            control::charge(self.units.len() as u64, "nearby_ally_damage_candidates")?;
            runtime_control::admit_retained_growth(
                0,
                self.units
                    .len()
                    .checked_mul(size_of::<UnitInstanceId>() + size_of::<EvalMember>() + 128),
                "nearby_ally_damage_frame",
            )?;
            let query = TileReachQuery::try_new(
                &self.battlefield,
                TileRangePolicy::Pattern,
                Some(&policy.range),
                anchor,
                facing,
            )
            .map_err(|_| runtime_control::invariant("nearby_ally_damage_range"))?;
            let mut targets: Vec<_> = self
                .units
                .iter()
                .filter_map(|(id, u)| {
                    (*id != actor
                        && u.owner == Side::Player
                        && u.is_active()
                        && !u.corroded
                        && u.stats.current_health > 0
                        && matches!(u.source_identity, BattleUnitSourceIdentity::Employee { .. })
                        && self
                            .sample_unit_world_position_at(*id, now)
                            .is_some_and(|p| query.contains(p.project_to_tile())))
                    .then_some(*id)
                })
                .collect();
            control::charge(
                (targets.len() as u64)
                    .saturating_mul(u64::from(usize::BITS - targets.len().leading_zeros())),
                "nearby_ally_damage_sort",
            )?;
            targets.sort_unstable();
            let count = u32::try_from(targets.len())
                .map_err(|_| runtime_control::invariant("nearby_ally_damage_count"))?;
            let mut frame = EvalFrame::default();
            frame.set_resolved_targets(
                targets
                    .into_iter()
                    .map(|id| EvalMember {
                        identity: id.to_string(),
                        value: None,
                        owner: None,
                        projected_tile: None,
                        values: BTreeMap::new(),
                    })
                    .collect(),
            );
            let plan = &policy.percent;
            let mut meter = ExprMeter::new(plan.evaluation_limits())
                .map_err(|_| runtime_control::invariant("nearby_ally_damage_meter"))?
                .with_host_admission(super::value_expr_runtime::evaluation_host_admission);
            let value = expr::evaluate(plan, &frame, &mut meter)
                .map_err(runtime_control::reject_expression)?
                .value;
            if !(0..=10_000).contains(&value.value) {
                return Err(runtime_control::reject_expression(expr::ValueExprFault {
                    code: expr::ValueExprFaultCode::Overflow,
                    site: plan.source().site.clone(),
                    node_index: 0,
                    expected_unit: Some(expr::ValueUnit::Scalar),
                    actual_unit: Some(value.unit),
                    phase: expr::ValueExprPhase::Narrow,
                    cause: "nearby ally damage increase must be from 0 to 10000 percent".into(),
                }));
            }
            let percent = value.value as u32;
            total = total
                .checked_add(percent)
                .filter(|n| *n <= i32::MAX as u32)
                .ok_or_else(|| runtime_control::invariant("nearby_ally_damage_sum"))?;
            if self.authoring_observation.is_some() {
                let origin = plan.source();
                let bytes = size_of::<NearbyAllyDamageContribution>()
                    .checked_add(origin.site.action.definition_id.as_str().len())
                    .and_then(|n| n.checked_add(origin.source_file.len()))
                    .and_then(|n| n.checked_add(origin.field_path.len()));
                runtime_control::admit_retained_growth(
                    0,
                    bytes,
                    "nearby_ally_damage_witness_copy",
                )?;
                control::charge(
                    bytes.ok_or(GameError::InvalidAction)? as u64,
                    "nearby_ally_damage_witness_copy",
                )?;
                contributors.reserve_exact(1);
                contributors.push(NearbyAllyDamageContribution {
                    equipment,
                    count,
                    percent,
                    definition_id: origin.site.action.definition_id.to_string(),
                    source_file: origin.source_file.clone(),
                    field_path: origin.field_path.clone(),
                });
            }
        }
        if !contributors.is_empty() {
            let witness = NearbyAllyDamageWitness {
                generation: self.content_identity.clone(),
                time_ms: now,
                actor: snapshot.source_id,
                wearer: actor,
                target,
                percent: total,
                contributors,
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "nearby_ally_damage_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_nearby_ally_damage = Some(witness);
        }
        control::poll("nearby_ally_damage_evaluated")?;
        Ok(total as i32)
    }
}
