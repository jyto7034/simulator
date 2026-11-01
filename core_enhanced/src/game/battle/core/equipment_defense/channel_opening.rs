//! Opening protection is derived from actual live channel leases. This witness
//! describes an eligible damage calculation, not remaining HP or a live buff.
use super::*;
use crate::game::battle::damage::{DamageSource, DamageSourceSnapshot};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChannelOpeningWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub target: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub time_ms: u64,
    pub started_at_ms: u64,
    pub window_end_ms: u64,
    pub reduction_percent: u8,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub incoming: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(ChannelOpeningWitness {
    generation,
    target,
    equipment,
    time_ms,
    started_at_ms,
    window_end_ms,
    reduction_percent,
    definition_id,
    source_file,
    field_path,
    incoming
});
impl ChannelOpeningWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?
            .checked_add(self.incoming.retained_extent()?)?;
        if let Some(g) = &self.generation {
            n = n
                .checked_add(std::mem::size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(n)
    }
}
impl BattleCore {
    pub(in crate::game::battle::core) fn channel_opening_reduction(
        &mut self,
        target: UnitInstanceId,
        incoming: &DamageSourceSnapshot,
        now: u64,
    ) -> Result<i32, GameError> {
        if !self
            .game_data
            .equipment_defense_catalog()
            .has_channel_opening
            || incoming.base_damage == 0
        {
            return Ok(0);
        }
        let Some(unit) = self
            .units
            .get(&target)
            .filter(|u| u.is_active() && !u.corroded)
        else {
            return Ok(0);
        };
        if incoming.source_id == target
            || incoming.source_side == unit.owner
            || (incoming.source == DamageSource::Environment && !incoming.fact_source.reaction)
        {
            return Ok(0);
        }
        let mut has_guard = false;
        self.visit_equipment_defenses(target, |_, p| {
            has_guard |= matches!(
                p.program,
                EquipmentDefenseProgram::ChannelOpeningGuard { .. }
            );
        })?;
        if !has_guard {
            return Ok(0);
        }
        let Some(start) = self.channel_started_at(target, now)? else {
            return Ok(0);
        };
        let mut chosen: Option<(
            EquipmentSourceSnapshot,
            Arc<CompiledEquipmentDefense>,
            u32,
            u8,
        )> = None;
        self.visit_equipment_defenses(target, |item, p| {
            if let EquipmentDefenseProgram::ChannelOpeningGuard {
                duration_ms,
                reduction_percent,
            } = p.program
            {
                if now >= start
                    && now - start < u64::from(duration_ms)
                    && chosen.as_ref().is_none_or(|(previous, _, _, percent)| {
                        reduction_percent > *percent
                            || (reduction_percent == *percent
                                && item.runtime_item_instance_id
                                    < previous.runtime_item_instance_id)
                    })
                {
                    chosen = Some((item, Arc::clone(p), duration_ms, reduction_percent));
                }
            }
        })?;
        let Some((equipment, program, duration, reduction)) = chosen else {
            return Ok(0);
        };
        if self.authoring_observation.is_some() {
            let extent = incoming.fact_source.retained_extent().and_then(|n| {
                n.checked_add(
                    size_of::<ChannelOpeningWitness>()
                        + program.definition_id.len()
                        + program.source_file.len()
                        + program.field_path.len(),
                )
            });
            runtime_control::admit_retained_growth(0, extent, "channel_opening_witness_copy")?;
            control::charge(
                extent.unwrap_or(usize::MAX) as u64,
                "channel_opening_witness_copy",
            )?;
            let witness = ChannelOpeningWitness {
                generation: self.content_identity.clone(),
                target,
                equipment,
                time_ms: now,
                started_at_ms: start,
                window_end_ms: start.saturating_add(duration.into()),
                reduction_percent: reduction,
                definition_id: program.definition_id.clone(),
                source_file: program.source_file.clone(),
                field_path: program.field_path.clone(),
                incoming: incoming.fact_source.observation(),
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "channel_opening_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_channel_opening = Some(witness);
        }
        control::poll("channel_opening_evaluated")?;
        Ok(i32::from(reduction))
    }
}
