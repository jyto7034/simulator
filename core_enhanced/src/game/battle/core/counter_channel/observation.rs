use super::*;
fn preview(
    phase: &CounterPhase,
    consumed: u32,
    tile: Position,
    origin: Position,
    facing: FacingDirection,
) -> Result<CounterPhasePreview, GameError> {
    let range = match &phase.range {
        PhaseRange::Fixed(r) => r,
        PhaseRange::Consumed(v) => {
            &v.iter()
                .find(|(n, _)| *n == consumed)
                .ok_or(GameError::InvalidAction)?
                .1
        }
    };
    let (origin, facing) = if phase.origin == PhaseOrigin::SelectedTile {
        (tile, FacingDirection::Up)
    } else {
        (origin, facing)
    };
    let tiles = range
        .pattern
        .affected_tiles(origin, facing)
        .map_err(GameError::InvalidStaticData)?;
    crate::game::battle::runtime_control::admit_retained_growth(
        0,
        tiles
            .capacity()
            .checked_mul(std::mem::size_of::<Position>())
            .and_then(|n| n.checked_add(phase.definition.len() + 64)),
        "counter_area_preview",
    )?;
    Ok(CounterPhasePreview {
        phase: phase.definition.clone(),
        origin,
        tiles,
    })
}
impl BattleCore {
    pub fn counter_channels_snapshot(&self) -> Result<CounterChannelsSnapshot, GameError> {
        control::charge(
            self.sustained.counters.entity_count() as u64,
            "counter_snapshot",
        )?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.sustained.counters.retained_extent(),
            "counter_snapshot_storage",
        )?;
        let mut owners = Vec::new();
        for (key, o) in &self.sustained.counters.owners {
            let unit = self.units.get(&o.caster).ok_or(GameError::InvalidAction)?;
            let origin = unit.body.projected_tile();
            let tile = o
                .aim
                .filter(|t| self.battlefield.is_valid_tile(*t))
                .unwrap_or(origin);
            let facing =
                FacingDirection::from_delta((tile.x - origin.x) as f32, (tile.y - origin.y) as f32)
                    .or(unit.facing_direction)
                    .unwrap_or(FacingDirection::Up);
            let auto = o
                .program
                .entries
                .iter()
                .find(|e| e.automatic)
                .ok_or(GameError::InvalidAction)?;
            let mut menu = Vec::new();
            for (i, e) in o
                .program
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| !e.automatic)
            {
                let skill_id = o.program.entry_skill_id(&o.fragment.skill_id, i);
                let resonance_eligible = self
                    .current_skill_view(&skill_id, o.caster, &Default::default())
                    .is_some_and(|skill| {
                        self.resonance_admission_payment(o.caster, skill).is_some()
                    });
                menu.push(CounterMenuSnapshot {
                    skill_id,
                    key: e.key.clone(),
                    minimum: e.minimum,
                    maximum: e.maximum,
                    counter_eligible: o.count >= e.minimum && o.count <= e.maximum,
                    resonance_eligible,
                    selection_range: e.selection_range.clone().ok_or(GameError::InvalidAction)?,
                });
            }
            let active = o
                .active
                .as_ref()
                .map(|a| {
                    let e = &o.program.entries[a.admitted.entry];
                    let phase = &e.phases[a.phase];
                    Ok::<_, GameError>(CounterActiveSnapshot {
                        skill_id: o
                            .program
                            .entry_skill_id(&o.fragment.skill_id, a.admitted.entry),
                        phase: phase.definition.clone(),
                        phase_end_ms: a.phase_end,
                        next_tick_ms: a.next_at,
                        consumed: a.admitted.consumed,
                        automatic: e.automatic,
                        cancellable: !e.automatic,
                        damaged_enemy_hp: a.damaged,
                        healed_ally_hp: a.healed,
                        area: preview(
                            phase,
                            a.admitted.consumed,
                            a.admitted.tile,
                            a.admitted.origin,
                            a.admitted.facing,
                        )?,
                    })
                })
                .transpose()?;
            owners.push(CounterOwnerSnapshot {
                source_copy: *key,
                caster: o.caster,
                count: o.count,
                capacity: o.program.capacity,
                next_gain_ms: o.next_gain,
                remembered_tile: o.aim,
                automatic_tile: tile,
                automatic_reserved: o.count >= o.program.capacity
                    && !active.as_ref().is_some_and(|a| a.automatic),
                pending_skill: o
                    .pending
                    .as_ref()
                    .map(|p| o.program.entry_skill_id(&o.fragment.skill_id, p.entry)),
                active,
                menu,
                automatic_areas: auto
                    .phases
                    .iter()
                    .map(|p| preview(p, o.program.capacity, tile, origin, facing))
                    .collect::<Result<_, _>>()?,
                applied_modifiers: o.applied.clone(),
            });
        }
        control::poll("counter_snapshot_complete")?;
        Ok(CounterChannelsSnapshot { owners })
    }
    pub fn counter_channel_preview(
        &self,
        caster: UnitInstanceId,
        skill_id: &SkillId,
        tile: Position,
        now: u64,
    ) -> Result<Option<CounterChannelPreview>, GameError> {
        let Some(o) = self
            .counter_key(caster)
            .and_then(|k| self.sustained.counters.owners.get(&k))
        else {
            return Ok(None);
        };
        let Some((_, _, entry)) = self
            .current_skill_view(skill_id, caster, &Default::default())
            .and_then(Self::counter_declaration)
        else {
            return Ok(None);
        };
        if !self.counter_manual_entry(&o.fragment.skill_id, skill_id)
            || !self.battlefield.is_valid_tile(tile)
        {
            return Err(GameError::InvalidAction);
        }
        let e = &o.program.entries[entry];
        let origin = self.units[&caster].body.projected_tile();
        let facing =
            FacingDirection::from_delta((tile.x - origin.x) as f32, (tile.y - origin.y) as f32)
                .or(self.units[&caster].facing_direction)
                .unwrap_or(FacingDirection::Up);
        let consumed = if e.spend == CounterSpend::AdmissionAll {
            o.count
        } else {
            0
        };
        if o.count < e.minimum || o.count > e.maximum {
            return Err(GameError::InvalidAction);
        }
        let phases = e
            .phases
            .iter()
            .map(|p| preview(p, consumed, tile, origin, facing))
            .collect::<Result<_, _>>()?;
        let end = now
            .checked_add(
                e.phases
                    .iter()
                    .map(|p| u64::from(p.duration_ms))
                    .sum::<u64>(),
            )
            .ok_or(GameError::InvalidAction)?;
        let time_gain = o
            .next_gain
            .filter(|t| *t <= end)
            .map_or(0, |t| 1 + (end - t) / u64::from(o.program.interval_ms));
        let possible = if let Some(award) = &e.completion_award {
            u128::from(o.count)
                + u128::from(program::gain(
                    &o.program,
                    CounterGainTarget::Completion,
                    u64::from(award.base) + u64::from(award.damage_and_healing_bonus),
                )?)
                + u128::from(time_gain)
                    * u128::from(program::gain(
                        &o.program,
                        CounterGainTarget::Periodic,
                        u64::from(o.program.periodic_gain),
                    )?)
                >= u128::from(o.program.capacity)
        } else {
            false
        };
        let possible_automatic_followup = if possible {
            let auto = o
                .program
                .entries
                .iter()
                .find(|e| e.automatic)
                .ok_or(GameError::InvalidAction)?;
            auto.phases
                .iter()
                .map(|p| preview(p, o.program.capacity, tile, origin, facing))
                .collect::<Result<_, _>>()?
        } else {
            Vec::new()
        };
        control::poll("counter_preview_complete")?;
        Ok(Some(CounterChannelPreview {
            phases,
            possible_automatic_followup,
        }))
    }
}
