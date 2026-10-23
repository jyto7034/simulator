//! Ownership for released objects, separate from employee and encounter slots.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    battle::{
        ids::UnitInstanceId,
        runtime_facts::{ActionOrdinal, RuntimeFactSource},
    },
    behavior::GameError,
    data::independent_object_data::IndependentObjectId,
    enums::Side,
    resources::Position,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ObjectFamilyKey {
    pub creator: UnitInstanceId,
    pub fragment_copy: Option<Uuid>,
    pub equipment_copy: Option<Uuid>,
    pub family: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct ObjectCounter {
    pub alive: u32,
    pub total: u32,
}

#[derive(Debug, Clone)]
pub(super) struct IndependentObject {
    pub id: UnitInstanceId,
    pub definition: IndependentObjectId,
    pub family: ObjectFamilyKey,
    pub side: Side,
    pub position: Position,
    pub epoch: u64,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub source: RuntimeFactSource,
    pub creation_action: Option<ActionOrdinal>,
    pub source_attack: u32,
    pub source_defense: i32,
    pub source_magic_resistance: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectRemovalReason {
    Expired,
    SourceRemoved,
    AttackConsumed,
    Destroyed,
    Despawned,
    BattleEnded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectRemovalReceipt {
    pub object_id: UnitInstanceId,
    pub creator: UnitInstanceId,
    pub source_actor: Option<UnitInstanceId>,
    pub definition: IndependentObjectId,
    pub epoch: u64,
    pub time_ms: u64,
    pub reason: ObjectRemovalReason,
}

#[derive(Debug, Clone, Default)]
pub(super) struct IndependentObjectState {
    pub objects: BTreeMap<UnitInstanceId, IndependentObject>,
    pub counters: BTreeMap<ObjectFamilyKey, ObjectCounter>,
    pub next_id: u64,
    pub last_removal: Option<ObjectRemovalReceipt>,
}

crate::authoring::clone_shape::struct_shape!(ObjectFamilyKey {
    creator,
    fragment_copy,
    equipment_copy,
    family
});
crate::authoring::clone_shape::copy_shape!(ObjectCounter, ObjectRemovalReason);
crate::authoring::clone_shape::struct_shape!(IndependentObject {
    id,
    definition,
    family,
    side,
    position,
    epoch,
    created_at_ms,
    expires_at_ms,
    source,
    creation_action,
    source_attack,
    source_defense,
    source_magic_resistance
});
crate::authoring::clone_shape::struct_shape!(ObjectRemovalReceipt {
    object_id,
    creator,
    source_actor,
    definition,
    epoch,
    time_ms,
    reason
});
crate::authoring::clone_shape::struct_shape!(IndependentObjectState {
    objects,
    counters,
    next_id,
    last_removal
});

impl ObjectFamilyKey {
    pub(super) fn retained_extent(&self) -> Option<usize> {
        std::mem::size_of::<Self>().checked_add(self.family.capacity())
    }
}

impl IndependentObject {
    pub(super) fn retained_extent(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(self.definition.0.capacity())?
            .checked_add(self.family.retained_extent()?)?
            .checked_add(self.source.retained_extent()?)
    }
}

impl IndependentObjectState {
    pub(super) fn entity_count(&self) -> usize {
        self.objects
            .len()
            .saturating_add(self.counters.len())
            .saturating_add(usize::from(self.last_removal.is_some()))
    }

    pub(super) fn retained_extent(&self) -> Option<usize> {
        // One entry may own a whole BTree node. Include sixteen key/value
        // slots and child pointers per entry, even at minimum occupancy.
        let mut bytes = std::mem::size_of::<Self>();
        for object in self.objects.values() {
            let node = std::mem::size_of::<(UnitInstanceId, IndependentObject)>()
                .checked_add(16)?
                .checked_mul(16)?;
            bytes = bytes.checked_add(object.retained_extent()?.checked_add(node)?)?;
        }
        for key in self.counters.keys() {
            let node = std::mem::size_of::<(ObjectFamilyKey, ObjectCounter)>()
                .checked_add(16)?
                .checked_mul(16)?;
            bytes = bytes.checked_add(key.retained_extent()?.checked_add(node)?)?;
        }
        if let Some(receipt) = &self.last_removal {
            bytes = bytes.checked_add(receipt.definition.0.capacity())?;
        }
        Some(bytes)
    }
}

impl super::BattleCore {
    pub(super) fn independent_action_origin(
        &self,
        id: UnitInstanceId,
    ) -> Result<
        Option<std::sync::Arc<crate::game::battle::runtime_facts::IndependentObjectOriginSnapshot>>,
        GameError,
    > {
        use crate::game::battle::runtime_facts::IndependentObjectOriginSnapshot;
        let Some(object) = self.independent_objects.objects.get(&id) else {
            return Ok(None);
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            object
                .source
                .retained_extent()
                .and_then(|n| n.checked_add(object.definition.0.len()))
                .and_then(|n| {
                    n.checked_add(std::mem::size_of::<IndependentObjectOriginSnapshot>() + 64)
                }),
            "independent_action_origin",
        )?;
        let origin = IndependentObjectOriginSnapshot {
            object_id: id,
            definition_id: crate::authoring::clone_shape::controlled_clone(&object.definition)?,
            creator: object.family.creator,
            creation_action: object.creation_action,
            creation_cast: object.source.cast.clone(),
            creation_fragment: object.source.fragment.clone(),
            creation_equipment: object.source.equipment,
            creation_authored: object.source.authored.clone(),
        };
        Ok(Some(std::sync::Arc::new(origin)))
    }
    /// Cleanup may run for a paused input; it must never trigger an attack.
    pub(super) fn reconcile_independent_object_removals(
        &mut self,
        now_ms: u64,
    ) -> Result<(), GameError> {
        use crate::game::data::summoned_unit_data::SummonerRemovalPolicyData;
        use std::ops::Bound::{Excluded, Unbounded};
        loop {
            let mut previous = None;
            let mut removed = false;
            loop {
                let next = match previous {
                    None => self.independent_objects.objects.first_key_value(),
                    Some(id) => self
                        .independent_objects
                        .objects
                        .range((Excluded(id), Unbounded))
                        .next(),
                };
                let Some((&id, object)) = next else {
                    break;
                };
                previous = Some(id);
                crate::authoring::control::charge(1, "independent_lifecycle_scan")?;
                let definition = self
                    .game_data
                    .boss_runtime_data
                    .independent_objects()
                    .get(&object.definition)
                    .ok_or(GameError::InvalidAction)?;
                let reason = if now_ms >= object.expires_at_ms {
                    Some(ObjectRemovalReason::Expired)
                } else if definition.definition.on_source_removed
                    == SummonerRemovalPolicyData::Despawn
                    && !self
                        .units
                        .get(&object.family.creator)
                        .is_some_and(|u| u.is_active())
                {
                    Some(ObjectRemovalReason::SourceRemoved)
                } else {
                    None
                };
                if let Some(reason) = reason {
                    let epoch = object.epoch;
                    self.retire_independent_object(id, epoch, now_ms, reason)?;
                    removed = true;
                }
            }
            // A removed actor can be another object's source. Settle that graph
            // before attacks, without recursion or an unmetered scratch list.
            if !removed {
                break;
            }
        }
        Ok(())
    }

    /// Attacks run only at a settled timestamp, after expiry and source cleanup.
    pub(super) fn reconcile_independent_objects(&mut self, now_ms: u64) -> Result<(), GameError> {
        use std::ops::Bound::{Excluded, Unbounded};
        self.reconcile_independent_object_removals(now_ms)?;
        let mut previous = None;
        loop {
            let next = match previous {
                None => self.independent_objects.objects.first_key_value(),
                Some(id) => self
                    .independent_objects
                    .objects
                    .range((Excluded(id), Unbounded))
                    .next(),
            };
            let Some((&id, object)) = next else {
                break;
            };
            previous = Some(id);
            crate::authoring::control::charge(1, "independent_trigger_scan")?;
            let definition = self
                .game_data
                .boss_runtime_data
                .independent_objects()
                .get(&object.definition)
                .cloned()
                .ok_or(GameError::InvalidAction)?;
            if let crate::game::data::independent_object_data::IndependentObjectKind::Trap(trap) =
                &definition.definition.kind
            {
                self.trigger_independent_trap(id, trap, now_ms)?;
                self.reconcile_independent_object_removals(now_ms)?;
            }
        }
        Ok(())
    }

    pub(super) fn clear_independent_objects(&mut self, now_ms: u64) -> Result<(), GameError> {
        while let Some((&id, object)) = self.independent_objects.objects.first_key_value() {
            let epoch = object.epoch;
            self.retire_independent_object(id, epoch, now_ms, ObjectRemovalReason::BattleEnded)?;
        }
        Ok(())
    }

    pub(super) fn expire_independent_object(
        &mut self,
        id: UnitInstanceId,
        epoch: u64,
        now_ms: u64,
    ) -> Result<(), GameError> {
        if !self
            .independent_objects
            .objects
            .get(&id)
            .is_some_and(|object| object.epoch == epoch && object.expires_at_ms <= now_ms)
        {
            return Ok(());
        }
        self.retire_independent_object(id, epoch, now_ms, ObjectRemovalReason::Expired)?;
        Ok(())
    }

    pub(super) fn retire_independent_object(
        &mut self,
        id: UnitInstanceId,
        epoch: u64,
        now_ms: u64,
        reason: ObjectRemovalReason,
    ) -> Result<bool, GameError> {
        if !self
            .independent_objects
            .objects
            .get(&id)
            .is_some_and(|object| object.epoch == epoch)
        {
            return Ok(false);
        }
        if self.units.get(&id).is_some_and(|unit| unit.is_active()) {
            self.withdraw_unit(id, now_ms)?;
        }
        self.event_queue.discard_channel_periodic_starts(id)?;
        self.event_queue.discard_independent_object_expiry(id)?;
        self.units.remove(&id);
        let references = self.scenario_runtime.spawned.unit_refs.len();
        crate::authoring::control::charge(references as u64, "object_adapter_reference_remove")?;
        self.scenario_runtime
            .spawned_for_write()?
            .unit_refs
            .retain(|_, unit| *unit != id);
        self.remove_independent_object_owner(id, epoch, now_ms, reason)
    }

    /// Called inside the common spawn candidate, after the complete authored
    /// request has been validated. All fallible admission precedes insertion.
    pub(super) fn register_independent_object(
        &mut self,
        object: IndependentObject,
        max_alive: u32,
        max_total: u32,
    ) -> Result<(), GameError> {
        use crate::game::battle::runtime_control;
        runtime_control::charge(1, "independent_object_register")?;
        if object.epoch == 0
            || object.expires_at_ms <= object.created_at_ms
            || self.independent_objects.objects.contains_key(&object.id)
            || max_alive == 0
            || max_total < max_alive
            || object.source.action.is_some()
            || object.source.use_control
            || object.source.delivery.is_some()
            || object.source.resource_capabilities.is_some()
            || object.source.allows_finite_children
            || object.source.flow.is_some()
            || object.source.reinput().is_some()
            || object.source.reaction
        {
            return Err(GameError::InvalidAction);
        }
        let old = self
            .independent_objects
            .counters
            .get(&object.family)
            .copied()
            .unwrap_or_default();
        let new = ObjectCounter {
            alive: old.alive.checked_add(1).ok_or(GameError::InvalidAction)?,
            total: old.total.checked_add(1).ok_or(GameError::InvalidAction)?,
        };
        if new.alive > max_alive || new.total > max_total {
            return Err(GameError::InvalidAction);
        }
        let new_family = !self
            .independent_objects
            .counters
            .contains_key(&object.family);
        self.admit_authoring_runtime_growth(
            1 + usize::from(new_family),
            "independent_object_owner",
        )?;
        let current = self.retained_runtime_fact_extent()?;
        let node = std::mem::size_of::<(UnitInstanceId, IndependentObject)>()
            .checked_add(16)
            .and_then(|n| n.checked_mul(16));
        let extra = object
            .retained_extent()
            .and_then(|n| n.checked_add(node?))
            .and_then(|n| {
                if new_family {
                    let counter_node = std::mem::size_of::<(ObjectFamilyKey, ObjectCounter)>()
                        .checked_add(16)?
                        .checked_mul(16)?;
                    n.checked_add(object.family.retained_extent()?.checked_add(counter_node)?)
                } else {
                    Some(n)
                }
            });
        runtime_control::admit_retained_growth(
            current,
            extra.and_then(|n| current.checked_add(n)),
            "independent_object_owner",
        )?;
        let key = crate::authoring::clone_shape::controlled_clone(&object.family)?;
        self.record_lab_object(&object.definition.0,false)?;
        self.independent_objects.counters.insert(key, new);
        self.independent_objects.objects.insert(object.id, object);
        Ok(())
    }

    /// Removes only the logical owner. The lifecycle caller retires its adapter
    /// and queue in the same candidate; a stale epoch has no authority.
    pub(super) fn remove_independent_object_owner(
        &mut self,
        id: UnitInstanceId,
        epoch: u64,
        time_ms: u64,
        reason: ObjectRemovalReason,
    ) -> Result<bool, GameError> {
        use crate::game::battle::runtime_control;
        runtime_control::charge(1, "independent_object_remove")?;
        let Some(object) = self
            .independent_objects
            .objects
            .get(&id)
            .filter(|o| o.epoch == epoch)
        else {
            return Ok(false);
        };
        let counter = self
            .independent_objects
            .counters
            .get(&object.family)
            .filter(|c| c.alive > 0)
            .ok_or(GameError::InvalidAction)?;
        let alive = counter.alive - 1;
        let current = self.retained_runtime_fact_extent()?;
        runtime_control::admit_retained_growth(
            current,
            current
                .checked_add(std::mem::size_of::<ObjectRemovalReceipt>())
                .and_then(|n| n.checked_add(object.definition.0.len())),
            "independent_object_removal_receipt",
        )?;
        self.admit_authoring_runtime_growth(
            usize::from(self.independent_objects.last_removal.is_none()),
            "independent_object_removal_receipt",
        )?;
        let creator = object.family.creator;
        let definition = crate::authoring::clone_shape::controlled_clone(&object.definition)?;
        let family = crate::authoring::clone_shape::controlled_clone(&object.family)?;
        self.record_lab_object(&definition.0,true)?;
        self.independent_objects.objects.remove(&id);
        self.independent_objects
            .counters
            .get_mut(&family)
            .expect("validated counter")
            .alive = alive;
        self.independent_objects.last_removal = Some(ObjectRemovalReceipt {
            object_id: id,
            creator,
            source_actor: self.runtime_fact_source.actor,
            definition,
            epoch,
            time_ms,
            reason,
        });
        let receipt = crate::authoring::clone_shape::controlled_clone(
            self.independent_objects
                .last_removal
                .as_ref()
                .expect("just recorded"),
        )?;
        self.record_event_log(
            time_ms,
            crate::game::battle::event_log::BattleLogEvent::IndependentObjectRemoved { receipt },
        );
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        battle::{
            runtime_facts::{ActionHandle, ActionToken, AttemptToken},
            scenario::BattleScenario,
        },
        data::GameDataBuilder,
    };
    use std::sync::Arc;

    fn core() -> super::super::BattleCore {
        super::super::BattleCore::new_from_scenario(
            BattleScenario::empty((4, 4)),
            Arc::new(GameDataBuilder::empty().build()),
            7,
        )
    }
    fn object(id: u128) -> IndependentObject {
        IndependentObject {
            id: UnitInstanceId(Uuid::from_u128(id)),
            definition: IndependentObjectId::new("friend"),
            family: ObjectFamilyKey {
                creator: UnitInstanceId(Uuid::from_u128(100)),
                fragment_copy: Some(Uuid::from_u128(200)),
                equipment_copy: None,
                family: "friends".into(),
            },
            side: Side::Player,
            position: Position::new(1, 1),
            epoch: 1,
            created_at_ms: 0,
            expires_at_ms: 1_000,
            source: RuntimeFactSource::default(),
            creation_action: None,
            source_attack: 20,
            source_defense: 10,
            source_magic_resistance: 5,
        }
    }

    #[test]
    fn u07_object_origin_releases_parent_credentials() {
        let source = RuntimeFactSource {
            actor: Some(UnitInstanceId(Uuid::from_u128(100))),
            action: Some(ActionHandle {
                validity: Arc::new(ActionToken {
                    attempt: Arc::new(AttemptToken),
                    ordinal: ActionOrdinal(7),
                }),
            }),
            use_control: true,
            allows_finite_children: true,
            reaction: true,
            flow: Some(Arc::new(
                crate::game::battle::runtime_facts::FlowOccurrence {
                    node_index: 2,
                    occurrence: 3,
                },
            )),
            ..Default::default()
        };
        let child = source.independent_origin().unwrap();
        assert_eq!(child.actor, source.actor);
        assert_eq!(source.action_ordinal(), Some(ActionOrdinal(7)));
        assert!(
            child.action.is_none()
                && !child.use_control
                && !child.allows_finite_children
                && !child.reaction
                && child.flow.is_none()
        );
        assert!(source.action.is_some() && source.use_control);
        let mut invalid = object(1);
        invalid.source = source;
        assert!(core().register_independent_object(invalid, 1, 2).is_err());
    }

    #[test]
    fn u07_object_owner_counter_clone_and_stale_removal() {
        let mut core = core();
        let before = core.retained_runtime_fact_extent().unwrap();
        core.register_independent_object(object(1), 1, 2).unwrap();
        assert!(core.retained_runtime_fact_extent().unwrap() > before);
        assert!(core.register_independent_object(object(2), 1, 2).is_err());
        let mut branch = crate::authoring::clone_shape::controlled_clone(&core).unwrap();
        let id = object(1).id;
        assert!(!branch
            .remove_independent_object_owner(id, 2, 1_000, ObjectRemovalReason::Expired)
            .unwrap());
        assert!(branch
            .remove_independent_object_owner(id, 1, 1_000, ObjectRemovalReason::Expired)
            .unwrap());
        assert!(!branch
            .remove_independent_object_owner(id, 1, 1_000, ObjectRemovalReason::Destroyed)
            .unwrap());
        assert_eq!(core.independent_objects.objects.len(), 1);
        assert!(core.independent_objects.last_removal.is_none());
        assert_eq!(
            branch.independent_objects.counters[&object(1).family],
            ObjectCounter { alive: 0, total: 1 }
        );
        branch.register_independent_object(object(2), 1, 2).unwrap();
        branch
            .remove_independent_object_owner(object(2).id, 1, 1_000, ObjectRemovalReason::Expired)
            .unwrap();
        assert!(branch.register_independent_object(object(3), 1, 2).is_err());
        let mut other_copy = object(3);
        other_copy.family.fragment_copy = Some(Uuid::from_u128(201));
        branch
            .register_independent_object(other_copy, 1, 2)
            .unwrap();
        assert_eq!(branch.independent_objects.counters.len(), 2);
    }
}
