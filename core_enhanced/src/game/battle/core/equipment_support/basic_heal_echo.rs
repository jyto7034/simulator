//! One self-heal per equipped provider after a normal basic healing action.
use super::*;

#[derive(Debug, Clone, serde::Serialize)]
pub struct BasicHealEchoWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub actual_other_healed: u64,
    pub actual_heal_percent: u32,
    pub max_hp_cap_percent: u8,
    pub max_hp: u32,
    pub requested_self_heal: u32,
    pub actual_self_healed: u32,
    pub healing_source: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(BasicHealEchoWitness {
    generation,
    time_ms,
    actor,
    equipment,
    definition_id,
    source_file,
    field_path,
    actual_other_healed,
    actual_heal_percent,
    max_hp_cap_percent,
    max_hp,
    requested_self_heal,
    actual_self_healed,
    healing_source
});
impl BasicHealEchoWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?
            .checked_add(self.healing_source.retained_extent()?)?;
        if let Some(g) = &self.generation {
            bytes = bytes
                .checked_add(size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(bytes)
    }
}
impl BattleCore {
    pub(in crate::game::battle::core) fn basic_heal_echo_action_eligible(
        &self,
        actor: UnitInstanceId,
    ) -> bool {
        let source = &self.runtime_fact_source;
        self.game_data
            .equipment_support_catalog()
            .has_basic_heal_echo
            && source.actor == Some(actor)
            && !source.reaction
            && source.reinput().is_none()
            && source.fragment.is_none()
            && source.cast.is_none()
            && source
                .action
                .as_ref()
                .is_some_and(|a| self.runtime_facts.is_basic_action(a))
    }

    pub(in crate::game::battle::core) fn basic_heal_echo_after_action(
        &mut self,
        actor: UnitInstanceId,
        actual: u64,
        now: u64,
    ) -> Result<(), GameError> {
        if actual == 0 || !self.basic_heal_echo_action_eligible(actor) {
            return Ok(());
        }
        let data = Arc::clone(&self.game_data);
        let catalog = data.equipment_support_catalog();
        let mut cursor = None;
        loop {
            let Some(unit) = self
                .units
                .get(&actor)
                .filter(|u| u.is_active() && !u.corroded && u.is_combatant())
            else {
                break;
            };
            let slots = unit.equipment_slots.as_ref();
            control::charge(
                if slots.is_some() { 3 } else { self.items.len() } as u64,
                "basic_heal_echo_providers",
            )?;
            let equipped = slots
                .into_iter()
                .flat_map(|s| s.slots.iter().flatten())
                .map(|s| {
                    self.items
                        .get(&s.runtime_item_instance_id)
                        .ok_or_else(|| runtime_control::invariant("basic_heal_echo_slot"))
                });
            let legacy = self
                .items
                .values()
                .take(if slots.is_some() { 0 } else { self.items.len() })
                .filter(|i| i.owner_unit_instance == actor)
                .map(Ok);
            let mut selected = None;
            for item in equipped.chain(legacy) {
                let item = item?;
                if item.owner_unit_instance != actor {
                    return Err(runtime_control::invariant("basic_heal_echo_owner"));
                }
                let equipment = item.source_snapshot();
                let id = equipment.runtime_item_instance_id;
                if cursor.is_some_and(|c| id <= c) {
                    continue;
                }
                let Some(plan) = catalog
                    .by_equipment
                    .get(&item.base_uuid)
                    .filter(|p| p.program.basic_heal_echo.is_some())
                else {
                    continue;
                };
                if selected.as_ref().is_none_or(|(old, _, _)| id < *old) {
                    selected = Some((id, equipment, Arc::clone(plan)));
                }
            }
            let Some((id, equipment, plan)) = selected else {
                break;
            };
            cursor = Some(id);
            let program = plan
                .program
                .basic_heal_echo
                .expect("selected echo provider");
            let max_hp = unit.stats.max_health;
            let amount = (u128::from(actual) * u128::from(program.actual_heal_percent) / 100)
                .min(u128::from(max_hp) * u128::from(program.max_hp_cap_percent) / 100)
                .min(i32::MAX as u128) as u32;
            if amount == 0 {
                continue;
            }
            let hp_before = unit.stats.current_health;
            self.runtime_fact_source
                .admit_owner("basic_heal_echo_source")?;
            control::charge(
                self.runtime_fact_source
                    .retained_extent()
                    .unwrap_or(usize::MAX) as u64,
                "basic_heal_echo_source_copy",
            )?;
            let mut source = self.runtime_fact_source.observation();
            source.equipment = Some(equipment);
            source.authored = plan.echo_source.clone();
            source.reaction = true;
            source.use_control = false;
            source.admit_owner("basic_heal_echo_source")?;
            let prior = std::mem::replace(&mut self.runtime_fact_source, source);
            let result = self.apply_direct_heal_and_record(Some(actor), actor, amount as i32, now);
            self.runtime_fact_source = prior;
            result?;
            let healed = self
                .units
                .get(&actor)
                .map_or(0, |u| u.stats.current_health.saturating_sub(hp_before));
            if self.authoring_observation.is_some() {
                let path = plan.echo_source.as_ref().expect("echo source");
                let bytes = size_of::<BasicHealEchoWitness>()
                    .checked_add(plan.definition_id.len())
                    .and_then(|n| n.checked_add(path.source_file.len()))
                    .and_then(|n| n.checked_add(path.field_path.len()))
                    .and_then(|n| {
                        self.runtime_fact_source
                            .retained_extent()
                            .and_then(|b| n.checked_add(b))
                    });
                runtime_control::admit_retained_growth(0, bytes, "basic_heal_echo_observation")?;
                control::charge(
                    bytes.unwrap_or(usize::MAX) as u64,
                    "basic_heal_echo_observation_copy",
                )?;
                let witness = BasicHealEchoWitness {
                    generation: self.content_identity.clone(),
                    time_ms: now,
                    actor,
                    equipment,
                    definition_id: plan.definition_id.clone(),
                    source_file: path.source_file.clone(),
                    field_path: path.field_path.clone(),
                    actual_other_healed: actual,
                    actual_heal_percent: program.actual_heal_percent,
                    max_hp_cap_percent: program.max_hp_cap_percent,
                    max_hp,
                    requested_self_heal: amount,
                    actual_self_healed: healed,
                    healing_source: self.runtime_fact_source.observation(),
                };
                runtime_control::admit_retained_growth(
                    0,
                    witness.retained_extent(),
                    "basic_heal_echo_observation",
                )?;
                self.authoring_observation
                    .as_mut()
                    .unwrap()
                    .latest_basic_heal_echo = Some(witness);
            }
            control::poll("basic_heal_echo_applied")?;
        }
        Ok(())
    }
}
