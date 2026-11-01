//! Current nearby opponents contribute flat defense through the shared stat base.
use super::*;
use crate::game::{
    battle::{
        tile_range::{TileRangePolicy, TileReachQuery},
        types::BattleUnitSourceIdentity,
    },
    stats::{StatId, StatModifier, StatModifierKind},
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct NearbyDefenseContribution {
    pub equipment: EquipmentSourceSnapshot,
    pub count: u32,
    pub per_enemy: u32,
    pub max_bonus: u32,
    pub bonus: u32,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(NearbyDefenseContribution {
    equipment,
    count,
    per_enemy,
    max_bonus,
    bonus,
    definition_id,
    source_file,
    field_path
});
#[derive(Debug, Clone, serde::Serialize)]
pub struct NearbyDefenseWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub bonus: i32,
    pub actual_defense: i32,
    pub contributors: Vec<NearbyDefenseContribution>,
}
crate::authoring::clone_shape::struct_shape!(NearbyDefenseWitness {
    generation,
    time_ms,
    actor,
    bonus,
    actual_defense,
    contributors
});
impl NearbyDefenseWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>().checked_add(
            self.contributors
                .capacity()
                .checked_mul(size_of::<NearbyDefenseContribution>())?,
        )?;
        if let Some(g) = &self.generation {
            bytes = bytes
                .checked_add(std::mem::size_of_val(g.as_ref()))?
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
    pub(in crate::game::battle::core) fn nearby_defense_modifier(
        &self,
        actor: UnitInstanceId,
    ) -> Result<StatModifier, GameError> {
        let (value, _) = self.nearby_defense_values(actor, false)?;
        Ok(StatModifier {
            stat: StatId::Defense,
            kind: StatModifierKind::Flat,
            value,
        })
    }

    fn nearby_defense_values(
        &self,
        actor: UnitInstanceId,
        capture: bool,
    ) -> Result<(i32, Vec<NearbyDefenseContribution>), GameError> {
        let mut total = 0i32;
        let mut contributors = Vec::new();
        if self.sustained.closed
            || !self
                .game_data
                .equipment_defense_catalog()
                .has_nearby_defense
        {
            return Ok((total, contributors));
        }
        let Some(wearer) = self.units.get(&actor).filter(|u| {
            u.is_active()
                && !u.corroded
                && u.owner == Side::Player
                && matches!(u.source_identity, BattleUnitSourceIdentity::Employee { .. })
        }) else {
            return Ok((total, contributors));
        };
        let now = self.sustained_observed_time();
        let anchor = self
            .sample_unit_world_position_at(actor, now)
            .map(|p| p.project_to_tile());
        let mut result = Ok(());
        self.visit_equipment_defenses(actor, |equipment, program| {
            if result.is_err() {
                return;
            }
            result = (|| {
                let EquipmentDefenseProgram::NearbyHostileDefense {
                    range,
                    per_enemy,
                    max_bonus,
                } = &program.program
                else {
                    return Ok(());
                };
                if equipment.persistent_copy_uuid.is_none_or(|id| id.is_nil()) {
                    return Err(GameError::InvalidStaticData(
                        "nearby defense requires an actual persistent equipment copy".into(),
                    ));
                }
                Self::charge_authoring_tile_pattern(range, self.units.len() as u64 + 1)?;
                control::charge(self.units.len() as u64, "nearby_defense_candidates")?;
                let query = TileReachQuery::try_new(
                    &self.battlefield,
                    TileRangePolicy::Pattern,
                    Some(range),
                    anchor,
                    wearer.facing_direction,
                )
                .map_err(|_| runtime_control::invariant("nearby_defense_range"))?;
                let mut count = 0u32;
                for (id, target) in &self.units {
                    if *id != actor
                        && target.owner != wearer.owner
                        && target.is_active()
                        && target.is_combatant()
                        && target.stats.current_health > 0
                        && self
                            .sample_unit_world_position_at(*id, now)
                            .is_some_and(|p| query.contains(p.project_to_tile()))
                    {
                        count = count
                            .checked_add(1)
                            .ok_or_else(|| runtime_control::invariant("nearby_defense_count"))?;
                    }
                }
                let bonus =
                    (u64::from(count) * u64::from(*per_enemy)).min(u64::from(*max_bonus)) as u32;
                total = total
                    .checked_add(bonus as i32)
                    .ok_or_else(|| runtime_control::invariant("nearby_defense_sum"))?;
                if capture {
                    let bytes = size_of::<NearbyDefenseContribution>()
                        + program.definition_id.len()
                        + program.source_file.len()
                        + program.field_path.len();
                    runtime_control::admit_retained_growth(
                        0,
                        Some(bytes),
                        "nearby_defense_contribution",
                    )?;
                    control::charge(bytes as u64, "nearby_defense_source_copy")?;
                    contributors.reserve_exact(1);
                    contributors.push(NearbyDefenseContribution {
                        equipment,
                        count,
                        per_enemy: *per_enemy,
                        max_bonus: *max_bonus,
                        bonus,
                        definition_id: program.definition_id.clone(),
                        source_file: program.source_file.clone(),
                        field_path: program.field_path.clone(),
                    });
                }
                Ok(())
            })();
        })?;
        result?;
        Ok((total, contributors))
    }

    pub(in crate::game::battle::core) fn reconcile_nearby_defense(
        &mut self,
    ) -> Result<(), GameError> {
        if !self
            .game_data
            .equipment_defense_catalog()
            .has_nearby_defense
        {
            return Ok(());
        }
        control::charge(self.units.len() as u64, "nearby_defense_wearers")?;
        runtime_control::admit_retained_growth(
            0,
            self.units.len().checked_mul(size_of::<UnitInstanceId>()),
            "nearby_defense_wearer_list",
        )?;
        let actors: Vec<_> = self.units.keys().copied().collect();
        for actor in actors {
            let (bonus, contributors) =
                self.nearby_defense_values(actor, self.authoring_observation.is_some())?;
            if bonus != 0 {
                self.ensure_passive_stat_base(actor)?;
            }
            if self.has_sustained_stat_base(actor) {
                self.refresh_sustained_target(actor)?;
            }
            if !contributors.is_empty() {
                let witness = NearbyDefenseWitness {
                    generation: self.content_identity.clone(),
                    time_ms: self.sustained_observed_time(),
                    actor,
                    bonus,
                    actual_defense: self.units[&actor].stats.defense,
                    contributors,
                };
                runtime_control::admit_retained_growth(
                    0,
                    witness.retained_extent(),
                    "nearby_defense_observation",
                )?;
                self.authoring_observation
                    .as_mut()
                    .unwrap()
                    .latest_nearby_defense = Some(witness);
            }
        }
        control::poll("nearby_defense_reconciled")
    }
}
