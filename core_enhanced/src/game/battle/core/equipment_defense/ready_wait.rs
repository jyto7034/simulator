//! Actual equipped-copy waiting and fixed outgoing damage windows. Admission
//! owns readiness; the observation is only the last successful consumption.
use super::*;
use crate::game::{
    ability::SkillId,
    battle::{
        damage::{DamageSource, DamageSourceSnapshot},
        types::BattleUnitSourceIdentity,
    },
};

#[derive(Debug, Clone)]
pub(super) struct ReadyWaitState {
    pub(super) owner: UnitInstanceId,
    equipment: EquipmentSourceSnapshot,
    program: Arc<CompiledEquipmentDefense>,
    pub(super) ready_skill: Option<SkillId>,
    ready_since: Option<u64>,
    percent: u32,
    until: u64,
}
crate::authoring::clone_shape::struct_shape!(ReadyWaitState {
    owner,
    equipment,
    program,
    ready_skill,
    ready_since,
    percent,
    until
});

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReadyWaitContribution {
    pub equipment: EquipmentSourceSnapshot,
    pub waited_ms: u64,
    pub per_second_percent: u32,
    pub max_percent: u32,
    pub percent: u32,
    pub until_ms: u64,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(ReadyWaitContribution {
    equipment,
    waited_ms,
    per_second_percent,
    max_percent,
    percent,
    until_ms,
    definition_id,
    source_file,
    field_path
});
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReadyWaitWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub skill_id: SkillId,
    pub percent: i32,
    pub contributors: Vec<ReadyWaitContribution>,
}
crate::authoring::clone_shape::struct_shape!(ReadyWaitWitness {
    generation,
    time_ms,
    actor,
    skill_id,
    percent,
    contributors
});
impl ReadyWaitWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = size_of::<Self>()
            .checked_add(self.skill_id.as_str().len())?
            .checked_add(
                self.contributors
                    .capacity()
                    .checked_mul(size_of::<ReadyWaitContribution>())?,
            )?;
        if let Some(g) = &self.generation {
            n = n
                .checked_add(std::mem::size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        for c in &self.contributors {
            n = n
                .checked_add(c.definition_id.capacity())?
                .checked_add(c.source_file.capacity())?
                .checked_add(c.field_path.capacity())?;
        }
        Some(n)
    }
}

impl BattleCore {
    pub(in crate::game::battle::core) fn reconcile_ready_wait(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        if !self.game_data.equipment_defense_catalog().has_ready_wait {
            return Ok(());
        }
        if self.sustained.closed {
            self.equipment_defense.ready_wait.clear();
            return Ok(());
        }
        control::charge(self.units.len() as u64, "ready_wait_wearers")?;
        let mut providers = Vec::new();
        for (actor, wearer) in &self.units {
            if !wearer.is_active()
                || wearer.corroded
                || wearer.owner != Side::Player
                || !matches!(
                    wearer.source_identity,
                    BattleUnitSourceIdentity::Employee { .. }
                )
            {
                continue;
            }
            let mut result = Ok(());
            self.visit_equipment_defenses(*actor, |equipment, program| {
                if result.is_err()
                    || !matches!(
                        program.program,
                        EquipmentDefenseProgram::ReadyWaitDamage { .. }
                    )
                {
                    return;
                }
                result = (|| {
                    if equipment.persistent_copy_uuid.is_none_or(|id| id.is_nil()) {
                        return Err(GameError::InvalidStaticData(
                            "ready-wait damage requires an actual persistent equipment copy".into(),
                        ));
                    }
                    runtime_control::admit_retained_growth(
                        0,
                        Some(
                            size_of::<(
                                UnitInstanceId,
                                EquipmentSourceSnapshot,
                                Arc<CompiledEquipmentDefense>,
                            )>() + 128,
                        ),
                        "ready_wait_provider",
                    )?;
                    providers.reserve_exact(1);
                    providers.push((*actor, equipment, Arc::clone(program)));
                    Ok(())
                })();
            })?;
            result?;
        }
        let mut live = std::collections::BTreeSet::new();
        let mut last_owner = None;
        let mut ready = None;
        for (owner, equipment, program) in providers {
            control::charge(1, "ready_wait_provider")?;
            let key = equipment.runtime_item_instance_id;
            live.insert(key);
            if last_owner != Some(owner) {
                ready = self.ready_wait_eligible_skill(now, owner)?;
                last_owner = Some(owner);
            }
            let replacing = self
                .equipment_defense
                .ready_wait
                .get(&key)
                .is_none_or(|r| r.owner != owner || r.equipment != equipment);
            if replacing {
                runtime_control::admit_retained_growth(
                    0,
                    Some(size_of::<ReadyWaitState>() + size_of::<uuid::Uuid>() + 96),
                    "ready_wait_state",
                )?;
                self.equipment_defense.ready_wait.insert(
                    key,
                    ReadyWaitState {
                        owner,
                        equipment,
                        program,
                        ready_skill: None,
                        ready_since: None,
                        percent: 0,
                        until: 0,
                    },
                );
            }
            let state = self.equipment_defense.ready_wait.get_mut(&key).unwrap();
            if state.ready_skill != ready {
                if let Some(id) = &ready {
                    runtime_control::admit_retained_growth(
                        0,
                        Some(id.as_str().len()),
                        "ready_wait_skill",
                    )?;
                    control::charge(id.as_str().len() as u64, "ready_wait_skill")?;
                }
                state.ready_skill = ready.clone();
                state.ready_since = ready.as_ref().map(|_| now);
            }
            if now >= state.until {
                state.percent = 0;
                state.until = 0;
            }
        }
        control::charge(
            self.equipment_defense.ready_wait.len() as u64,
            "ready_wait_cleanup",
        )?;
        self.equipment_defense
            .ready_wait
            .retain(|key, _| live.contains(key));
        control::poll("ready_wait_reconciled")
    }

    pub(in crate::game::battle::core) fn activate_ready_wait(
        &mut self,
        actor: UnitInstanceId,
        skill: &SkillId,
        now: u64,
    ) -> Result<(), GameError> {
        if !self.game_data.equipment_defense_catalog().has_ready_wait {
            return Ok(());
        }
        let capture = self.authoring_observation.is_some();
        let mut contributions = Vec::new();
        let mut total = 0i32;
        control::charge(
            self.equipment_defense.ready_wait.len() as u64,
            "ready_wait_consume",
        )?;
        for state in self
            .equipment_defense
            .ready_wait
            .values_mut()
            .filter(|r| r.owner == actor)
        {
            let EquipmentDefenseProgram::ReadyWaitDamage {
                per_second_percent,
                max_percent,
                duration_ms,
            } = state.program.program
            else {
                return Err(runtime_control::invariant("ready_wait_program"));
            };
            let waited = if state.ready_skill.as_ref() == Some(skill) {
                state
                    .ready_since
                    .map_or(0, |start| now.saturating_sub(start))
            } else {
                0
            };
            let percent = (u128::from(waited) * u128::from(per_second_percent) / 1000)
                .min(u128::from(max_percent)) as u32;
            let until = now
                .checked_add(u64::from(duration_ms))
                .ok_or_else(|| runtime_control::invariant("ready_wait_deadline"))?;
            state.ready_skill = None;
            state.ready_since = None;
            state.percent = percent;
            state.until = until;
            total = total
                .checked_add(percent as i32)
                .ok_or_else(|| runtime_control::invariant("ready_wait_sum"))?;
            if capture {
                let p = &state.program;
                let bytes = size_of::<ReadyWaitContribution>()
                    + p.definition_id.len()
                    + p.source_file.len()
                    + p.field_path.len();
                runtime_control::admit_retained_growth(0, Some(bytes), "ready_wait_contribution")?;
                control::charge(bytes as u64, "ready_wait_source_copy")?;
                contributions.reserve_exact(1);
                contributions.push(ReadyWaitContribution {
                    equipment: state.equipment,
                    waited_ms: waited,
                    per_second_percent,
                    max_percent,
                    percent,
                    until_ms: until,
                    definition_id: p.definition_id.clone(),
                    source_file: p.source_file.clone(),
                    field_path: p.field_path.clone(),
                });
            }
        }
        if !contributions.is_empty() {
            runtime_control::admit_retained_growth(
                0,
                Some(skill.as_str().len()),
                "ready_wait_witness_skill",
            )?;
            let witness = ReadyWaitWitness {
                generation: self.content_identity.clone(),
                time_ms: now,
                actor,
                skill_id: skill.clone(),
                percent: total,
                contributors: contributions,
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "ready_wait_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_ready_wait = Some(witness);
        }
        control::poll("ready_wait_activated")
    }

    pub(in crate::game::battle::core) fn ready_wait_damage_percent(
        &self,
        incoming: &DamageSourceSnapshot,
        now: u64,
    ) -> Result<i32, GameError> {
        // Reflections use Environment internally but retain the actual wearer
        // and reaction source. Unowned environmental damage remains excluded.
        if incoming.source == DamageSource::Environment && !incoming.fact_source.reaction {
            return Ok(0);
        }
        control::charge(
            self.equipment_defense.ready_wait.len() as u64,
            "ready_wait_damage",
        )?;
        self.equipment_defense
            .ready_wait
            .values()
            .filter(|r| r.owner == incoming.source_id && now < r.until)
            .try_fold(0i32, |sum, r| {
                sum.checked_add(r.percent as i32)
                    .ok_or_else(|| runtime_control::invariant("ready_wait_sum"))
            })
    }
}
