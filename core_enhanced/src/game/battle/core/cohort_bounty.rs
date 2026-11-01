//! Cohort kills produce Attempt currency. Only terminal victory promotes it
//! into the copy's Run balance; all stat readers use the shared stat composer.
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        ability::{
            equipment_attack::CompiledEquipmentAttack, hostile_cohort::HostileCohortProvider,
        },
        battle::{
            damage::{DamageSource, DamageSourceSnapshot},
            ids::UnitInstanceId,
            runtime_control,
            runtime_facts::{EquipmentSourceSnapshot, RuntimeFactSource},
            types::{BattleUnitSourceIdentity, BattleVerdict},
        },
        behavior::GameError,
        enums::Side,
        resource::{ResourceMutation, ResourceOperation, ResourceOwner, ResourceOwnerSelector},
        stats::{StatId, StatModifier, StatModifierKind},
    },
};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum BountyOperation {
    Earned,
    Confirmed,
    Discarded,
    Equipped,
}
crate::authoring::clone_shape::copy_shape!(BountyOperation);
#[derive(Debug, Clone, serde::Serialize)]
pub struct BountyWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: Option<UnitInstanceId>,
    pub equipment_copy: uuid::Uuid,
    pub operation: BountyOperation,
    pub pending: u32,
    pub confirmed: u32,
    pub tier_attack: u32,
    pub next_threshold: Option<u32>,
    pub actual_attack: Option<u32>,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(BountyWitness {
    generation,
    time_ms,
    actor,
    equipment_copy,
    operation,
    pending,
    confirmed,
    tier_attack,
    next_threshold,
    actual_attack,
    definition_id,
    source_file,
    field_path
});
impl BountyWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = std::mem::size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?;
        if let Some(g) = &self.generation {
            n = n
                .checked_add(std::mem::size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(n)
    }
}

impl BattleCore {
    pub(super) fn equipped_attack_providers(
        &self,
        actor: UnitInstanceId,
        accepts: fn(&CompiledEquipmentAttack) -> bool,
    ) -> Result<Vec<(EquipmentSourceSnapshot, Arc<CompiledEquipmentAttack>)>, GameError> {
        let mut providers = Vec::new();
        let Some(unit) = self.units.get(&actor).filter(|u| {
            u.is_active()
                && !u.corroded
                && u.owner == Side::Player
                && matches!(u.source_identity, BattleUnitSourceIdentity::Employee { .. })
        }) else {
            return Ok(providers);
        };
        let slots = unit.equipment_slots.as_ref();
        control::charge(
            if slots.is_some() { 3 } else { self.items.len() } as u64,
            "bounty_providers",
        )?;
        let actual = slots
            .into_iter()
            .flat_map(|s| s.slots.iter().flatten())
            .map(|slot| {
                self.items
                    .get(&slot.runtime_item_instance_id)
                    .ok_or(GameError::InvalidAction)
            });
        let legacy = self
            .items
            .values()
            .take(if slots.is_some() { 0 } else { self.items.len() })
            .filter(|i| i.owner_unit_instance == actor)
            .map(Ok);
        for item in actual.chain(legacy) {
            let item = item?;
            if item.owner_unit_instance != actor {
                return Err(GameError::InvalidAction);
            }
            let Some(program) = self
                .game_data
                .equipment_attack_catalog()
                .by_equipment
                .get(&item.base_uuid)
                .filter(|p| accepts(p))
            else {
                continue;
            };
            if item.persistent_copy_uuid.is_none_or(|id| id.is_nil()) {
                return Err(GameError::InvalidStaticData(
                    "equipment passive requires an actual persistent equipment copy".into(),
                ));
            }
            runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<(
                    EquipmentSourceSnapshot,
                    Arc<CompiledEquipmentAttack>,
                )>()),
                "bounty_provider_capture",
            )?;
            providers.reserve_exact(1);
            providers.push((item.source_snapshot(), Arc::clone(program)));
        }
        Ok(providers)
    }

    fn bounty_providers(&self, actor: UnitInstanceId) -> Result<Vec<(EquipmentSourceSnapshot, Arc<CompiledEquipmentAttack>)>, GameError> {
        self.equipped_attack_providers(actor, |p| p.cohort.as_ref().is_some_and(|c| c.bounty.is_some()))
    }

    pub(super) fn reward_cohort_bounty(
        &mut self,
        incoming: &DamageSourceSnapshot,
        definition: &str,
        now: u64,
    ) -> Result<(), GameError> {
        if incoming.fact_source.reaction
            || incoming.fact_source.guard_release().is_some()
            || incoming
                .fact_source
                .cast
                .as_ref()
                .is_some_and(|c| c.proc_origin.is_some())
            || !matches!(
                incoming.source,
                DamageSource::BasicAttack | DamageSource::Ability
            )
        {
            return Ok(());
        }
        for (equipment, program) in self.bounty_providers(incoming.source_id)? {
            let cohort = program.cohort.as_ref().unwrap();
            if cohort.spawn.definition != definition {
                continue;
            }
            let bounty = cohort.bounty.as_ref().unwrap();
            let mut source = incoming.fact_source.clone();
            source.actor = Some(incoming.source_id);
            source.equipment = Some(equipment);
            source.authored = Some(Arc::clone(&program.cohort_source));
            let prior = std::mem::replace(&mut self.runtime_fact_source, source);
            let result = self.apply_typed_resource_operations(
                &[ResourceOperation {
                    resource: bounty.pending.clone(),
                    owner: ResourceOwnerSelector::SourceEquipment,
                    operation: ResourceMutation::Grant(1),
                }],
                &[],
                now,
            );
            self.runtime_fact_source = prior;
            result?;
            self.observe_bounty(
                equipment.persistent_copy_uuid.unwrap(),
                &program,
                Some(incoming.source_id),
                now,
                BountyOperation::Earned,
            )?;
        }
        control::poll("cohort_bounty_earned")
    }

    pub(super) fn settle_cohort_bounties(
        &mut self,
        verdict: BattleVerdict,
        now: u64,
    ) -> Result<(), GameError> {
        // Providers outlive the wearer's deployment. They are registration
        // identity only; balances remain exclusively in the resource store.
        let count: usize = self
            .sustained
            .shared_cohorts
            .owners
            .values()
            .map(|o| o.providers.len())
            .sum();
        control::charge(count as u64, "bounty_settlement_providers")?;
        runtime_control::admit_retained_growth(
            0,
            count.checked_mul(std::mem::size_of::<(
                uuid::Uuid,
                uuid::Uuid,
                Arc<CompiledEquipmentAttack>,
            )>()),
            "bounty_settlement_capture",
        )?;
        let mut providers = Vec::with_capacity(count);
        for owner in self.sustained.shared_cohorts.owners.values() {
            for provider in owner.providers.values() {
                let HostileCohortProvider::Equipment { copy, definition } = provider else {
                    continue;
                };
                if let Some(program) = self
                    .game_data
                    .equipment_attack_catalog()
                    .by_equipment
                    .get(definition)
                    .filter(|p| p.cohort.as_ref().is_some_and(|c| c.bounty.is_some()))
                {
                    providers.push((owner.employee, *copy, Arc::clone(program)));
                }
            }
        }
        for (employee, copy, program) in providers {
            let bounty = program.cohort.as_ref().unwrap().bounty.as_ref().unwrap();
            let runtime = self
                .runtime_facts
                .resources
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            let pending = runtime
                .store
                .bind(
                    &bounty.pending,
                    ResourceOwner::EquipmentCopy(copy),
                    &runtime.attempt,
                )
                .map_err(|e| e.into_game_error())?;
            let amount = runtime
                .store
                .value(&pending)
                .map_err(|e| e.into_game_error())?
                .balance;
            if amount == 0 {
                continue;
            }
            let mut operations = vec![(pending, ResourceMutation::Consume(amount))];
            if verdict == BattleVerdict::Victory {
                let run = runtime.run.as_ref().ok_or(GameError::InvalidAction)?;
                let confirmed = runtime
                    .store
                    .bind(
                        &bounty.confirmed,
                        ResourceOwner::EquipmentCopy(copy),
                        &run.scope,
                    )
                    .map_err(|e| e.into_game_error())?;
                operations.push((confirmed, ResourceMutation::Grant(amount)));
            }
            let source = RuntimeFactSource {
                actor: self
                    .units
                    .values()
                    .find(|u| u.source_owned_uuid == employee)
                    .map(|u| u.instance_id),
                equipment: self
                    .items
                    .values()
                    .find(|i| i.persistent_copy_uuid == Some(copy))
                    .map(|i| i.source_snapshot()),
                authored: Some(Arc::clone(&program.cohort_source)),
                ..Default::default()
            };
            self.apply_typed_resource_batch(&operations, &source, now)?;
            self.observe_bounty(
                copy,
                &program,
                source.actor,
                now,
                if verdict == BattleVerdict::Victory {
                    BountyOperation::Confirmed
                } else {
                    BountyOperation::Discarded
                },
            )?;
        }
        control::poll("cohort_bounty_settled")
    }

    pub(super) fn bounty_attack_modifier(
        &self,
        actor: UnitInstanceId,
    ) -> Result<StatModifier, GameError> {
        let mut total = 0i32;
        if !self.sustained.closed {
            if let Some(runtime) = self
                .runtime_facts
                .resources
                .as_ref()
                .filter(|r| r.run.is_some())
            {
                for (equipment, program) in self.bounty_providers(actor)? {
                    let bounty = program.cohort.as_ref().unwrap().bounty.as_ref().unwrap();
                    let lease = runtime
                        .store
                        .bind(
                            &bounty.confirmed,
                            ResourceOwner::EquipmentCopy(equipment.persistent_copy_uuid.unwrap()),
                            &runtime.run.as_ref().unwrap().scope,
                        )
                        .map_err(|e| e.into_game_error())?;
                    let balance = runtime
                        .store
                        .value(&lease)
                        .map_err(|e| e.into_game_error())?
                        .balance;
                    total = total
                        .checked_add(bounty.attack(balance) as i32)
                        .ok_or_else(|| runtime_control::invariant("bounty_attack_overflow"))?;
                }
            }
        }
        Ok(StatModifier {
            stat: StatId::Attack,
            kind: StatModifierKind::Flat,
            value: total,
        })
    }

    pub(super) fn reconcile_bounty_stats(&mut self) -> Result<(), GameError> {
        let catalog = self.game_data.equipment_attack_catalog();
        control::charge(catalog.by_equipment.len() as u64, "bounty_stat_catalog")?;
        if !catalog
            .by_equipment
            .values()
            .any(|p| p.cohort.as_ref().is_some_and(|c| c.bounty.is_some()))
        {
            return Ok(());
        }
        control::charge(self.units.len() as u64, "bounty_stat_units")?;
        runtime_control::admit_retained_growth(
            0,
            self.units
                .len()
                .checked_mul(std::mem::size_of::<UnitInstanceId>()),
            "bounty_stat_targets",
        )?;
        let targets: Vec<_> = self.units.keys().copied().collect();
        for target in targets {
            let modifier = self.bounty_attack_modifier(target)?;
            let before = self.units[&target].stats.attack;
            if modifier.value != 0 {
                self.ensure_passive_stat_base(target)?;
            }
            if modifier.value != 0 || self.has_sustained_stat_base(target) {
                self.refresh_sustained_target(target)?;
            }
            if !self.sustained.closed && self.units[&target].stats.attack != before {
                for (equipment, program) in self.bounty_providers(target)? {
                    self.observe_bounty(
                        equipment.persistent_copy_uuid.unwrap(),
                        &program,
                        Some(target),
                        self.sustained_observed_time(),
                        BountyOperation::Equipped,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn observe_bounty(
        &mut self,
        copy: uuid::Uuid,
        program: &CompiledEquipmentAttack,
        actor: Option<UnitInstanceId>,
        time_ms: u64,
        operation: BountyOperation,
    ) -> Result<(), GameError> {
        if self.authoring_observation.is_none() {
            return Ok(());
        }
        let bounty = program.cohort.as_ref().unwrap().bounty.as_ref().unwrap();
        let runtime = self
            .runtime_facts
            .resources
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let owner = ResourceOwner::EquipmentCopy(copy);
        let read = |id, scope| {
            let lease = runtime
                .store
                .bind(id, owner, scope)
                .map_err(|e| e.into_game_error())?;
            runtime
                .store
                .value(&lease)
                .map(|v| v.balance)
                .map_err(|e| e.into_game_error())
        };
        let pending = read(&bounty.pending, &runtime.attempt)?;
        let confirmed = read(
            &bounty.confirmed,
            &runtime.run.as_ref().ok_or(GameError::InvalidAction)?.scope,
        )?;
        let origin = &program.cohort_source;
        use crate::authoring::clone_shape::controlled_clone;
        runtime_control::admit_retained_growth(
            0,
            std::mem::size_of::<BountyWitness>()
                .checked_add(origin.site.action.definition_id.as_str().len())
                .and_then(|n| n.checked_add(origin.source_file.len()))
                .and_then(|n| n.checked_add(origin.field_path.len())),
            "bounty_witness_copy",
        )?;
        let witness = BountyWitness {
            generation: self.content_identity.clone(),
            time_ms,
            actor,
            equipment_copy: copy,
            operation,
            pending,
            confirmed,
            tier_attack: bounty.attack(confirmed),
            next_threshold: bounty
                .tiers
                .iter()
                .find(|t| t.threshold > confirmed)
                .map(|t| t.threshold),
            actual_attack: actor.and_then(|id| {
                self.units
                    .get(&id)
                    .filter(|u| u.is_active())
                    .map(|u| u.stats.attack)
            }),
            definition_id: controlled_clone(&origin.site.action.definition_id.to_string())?,
            source_file: controlled_clone(&origin.source_file)?,
            field_path: controlled_clone(&origin.field_path)?,
        };
        runtime_control::admit_retained_growth(0, witness.retained_extent(), "bounty_observation")?;
        if let Some(observation) = &mut self.authoring_observation {
            observation.latest_bounty = Some(witness);
        }
        Ok(())
    }
}
