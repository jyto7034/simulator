use super::*;
impl BattleCore {
    pub fn input_plans_snapshot(&self) -> Result<InputPlansSnapshot, GameError> {
        control::charge(
            self.sustained.input_plans.entity_count() as u64,
            "input_plan_observation",
        )?;
        let state = &self.sustained.input_plans;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            state.retained_extent(),
            "input_plan_snapshot_storage",
        )?;
        let owners = state
            .owners
            .iter()
            .map(|(copy, o)| {
                Ok(ShotOwnerView {
                    copy: *copy,
                    employee: o.employee,
                    caster: o.caster,
                    normal: o.normal,
                    enhanced: o.enhanced,
                    next_stock_ms: o.next_stock,
                    next_hostile_ms: o.next_cohort,
                    memory: controlled_clone(&o.memory)?,
                    applied: controlled_clone(&o.applied)?.into_iter().collect(),
                })
            })
            .collect::<Result<_, GameError>>()?;
        let preparation = state
            .preparation
            .as_ref()
            .map(|p| {
                let o = &state.owners[&p.copy];
                let u = &self.units[&p.caster];
                Ok(ShotPreparationView {
                    id: p.id,
                    copy: p.copy,
                    caster: p.caster,
                    started_at_ms: p.started,
                    normal: p.normal,
                    enhanced: p.enhanced,
                    remembered_layout_valid: self.validate_shot_layout(p, &o.memory).is_ok(),
                    placement_tiles: o
                        .program
                        .placement_range
                        .affected_tiles(
                            u.world_position().project_to_tile(),
                            u.facing_direction
                                .unwrap_or(crate::game::battle::tile_range::FacingDirection::Up),
                        )
                        .map_err(GameError::InvalidStaticData)?
                        .into_iter()
                        .filter(|t| self.projectile_tile_valid(*t))
                        .collect(),
                })
            })
            .transpose()?;
        Ok(InputPlansSnapshot {
            owners,
            preparation,
            disconnected: state.disconnected,
            receipts: state.receipts.values().copied().collect(),
            active_projectiles: state.flights.keys().copied().collect(),
            settled_projectiles: state
                .settled
                .values()
                .map(controlled_clone)
                .collect::<Result<_, _>>()?,
            cohort_units: state.cohorts.iter().map(|(a, b)| (*a, *b)).collect(),
            wounds: state
                .wounds
                .iter()
                .map(|((a, b, c), w)| Ok((*a, controlled_clone(b)?, *c, w.count, w.expires)))
                .collect::<Result<_, GameError>>()?,
        })
    }
}
