use super::*;
impl BattleCore {
    pub(in crate::game::battle::core) fn selected_relation_target(
        &self,
        caster: UnitInstanceId,
    ) -> Option<UnitInstanceId> {
        self.relation_key(caster)
            .and_then(|k| self.sustained.relations.owners.get(&k))
            .and_then(|o| o.relation.as_ref().map(|r| r.target))
    }
    pub fn relation_actions_snapshot(&self) -> Result<RelationActionsSnapshot, GameError> {
        control::charge(
            self.sustained.relations.entity_count() as u64,
            "relation_snapshot",
        )?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.sustained.relations.retained_extent(),
            "relation_snapshot_storage",
        )?;
        let mut owners = Vec::new();
        for (key, o) in &self.sustained.relations.owners {
            let menu_available = self.relation_live(o)
                && !o.mode
                && self.normal_player_employee(o.caster)
                && self
                    .unit_control_state(o.caster, self.sustained_observed_time())
                    .is_some_and(|s| !s.hard_cc_active() && !s.silenced)
                && !self.unit_is_field_working(o.caster)
                && !self.unit_is_in_defense_stance(o.caster);
            control::charge(self.units.len() as u64, "relation_menu_targets")?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                self.units
                    .len()
                    .checked_mul(std::mem::size_of::<UnitInstanceId>()),
                "relation_menu_storage",
            )?;
            let eligible_targets = if menu_available {
                self.units
                    .values()
                    .filter(|u| self.normal_player_employee(u.instance_id))
                    .map(|u| u.instance_id)
                    .collect()
            } else {
                Vec::new()
            };
            let active = o.active.as_ref().map(|a| RelationExecutionSnapshot {
                activation: a.id,
                entry: Self::relation_entry(&o.program, &a.admitted)
                    .definition
                    .clone(),
                waiting_basic: a.waiting_basic,
                primary: a.admitted.primary,
                start_stock: a.start_stock,
                next_tick_ms: a.next_at,
                next_self_damage_ms: a.next_self,
                remaining_plans: a
                    .continuations
                    .iter()
                    .rev()
                    .filter_map(|c| match c {
                        Continuation::Plan { index, .. } => Some(*index),
                        Continuation::Repeat { plan, .. }
                        | Continuation::Timed { plan, .. }
                        | Continuation::AfterSpend { plan, .. } => Some(*plan),
                        _ => None,
                    })
                    .map(|i| o.program.plans[i].definition.clone())
                    .collect(),
                target_hits: a.hits.iter().map(|(t, n)| (*t, *n)).collect(),
                target_ramps: a.ramps.iter().map(|(t, n)| (*t, *n)).collect(),
            });
            owners.push(RelationOwnerSnapshot {
                source_copy: *key,
                caster: o.caster,
                stock: o.stock,
                capacity: o.program.stock_capacity,
                active_mode: o.mode,
                mode_reserved: o.reserved_mode,
                shield_used: o.shield_used,
                target: o.relation.as_ref().map(|r| r.target),
                generation: o.relation.as_ref().map(|r| r.generation),
                next_gain_ms: o.relation.as_ref().map(|r| r.next_gain),
                menu_entry: o.program.menu_entry.clone(),
                menu_available,
                eligible_targets,
                pending: o.pending.is_some(),
                active,
                applied_modifiers: o.applied.clone(),
            });
        }
        Ok(RelationActionsSnapshot { owners })
    }
}
