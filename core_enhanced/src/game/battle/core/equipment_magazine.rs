//! A weapon copy owns ammunition; a released projectile owns only its round.
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        ability::{equipment_attack::CompiledEquipmentAttack, DeliveryDef, ProjectileHitPolicy},
        battle::{
            damage::{DamageSource, DamageSourceSnapshot},
            enums::BattleEvent,
            ids::UnitInstanceId,
            runtime_control,
            runtime_facts::EquipmentSourceSnapshot,
        },
        behavior::GameError,
        data::abnormality_data::BasicAttackAction,
    },
};
use std::{collections::BTreeMap, mem::size_of, sync::Arc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MagazineKey {
    Persistent(Uuid),
    Legacy { employee: Uuid, definition: Uuid },
}
crate::authoring::clone_shape::copy_shape!(MagazineKey);
impl MagazineKey {
    pub(super) fn for_equipment(employee: Uuid, equipment: EquipmentSourceSnapshot) -> Self {
        equipment
            .persistent_copy_uuid
            .map(Self::Persistent)
            .unwrap_or(Self::Legacy {
                employee,
                definition: equipment.definition_uuid,
            })
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Magazine {
    pub(super) owner: UnitInstanceId,
    pub(super) employee: Uuid,
    pub(super) equipment: EquipmentSourceSnapshot,
    pub(super) remaining: u32,
    pub(super) enhanced: bool,
    pub(super) reload_at: Option<u64>,
    pub(super) waiting: bool,
}
crate::authoring::clone_shape::copy_shape!(Magazine);
#[derive(Debug, Clone, Default)]
pub(super) struct EquipmentAttackState {
    pub(super) release_gaps: BTreeMap<Uuid, super::release_gap::ReleaseGapClock>,
    pub(super) continuity: BTreeMap<Uuid, super::attack_continuity::AttackClock>,
    pub(super) magazines: BTreeMap<MagazineKey, Magazine>,
}
crate::authoring::clone_shape::struct_shape!(EquipmentAttackState { magazines, continuity, release_gaps });
impl EquipmentAttackState {
    pub(super) fn remove_employee(&mut self, employee: Uuid) {
        if control::charge(self.magazines.len() as u64, "magazine_remove_employee").is_err() {
            return;
        }
        self.magazines.retain(|_, m| m.employee != employee);
        if control::charge(self.continuity.len() as u64, "attack_continuity_remove_employee").is_err() { return; }
        self.continuity.retain(|_, c| c.employee != employee);
        if control::charge(self.release_gaps.len() as u64, "release_gap_remove_employee").is_err() { return; }
        self.release_gaps.retain(|_, c| c.employee != employee);
    }

    pub(super) fn retained_extent(&self) -> Option<usize> {
        size_of::<Self>().checked_add(self.release_gaps.len().checked_mul(size_of::<super::release_gap::ReleaseGapClock>() + 128)?)?.checked_add(self.continuity.len().checked_mul(size_of::<super::attack_continuity::AttackClock>() + 128)?)?.checked_add(
            self.magazines
                .len()
                .checked_mul(size_of::<Magazine>() + 128)?,
        )
    }
}

#[derive(Debug)]
pub(crate) struct MagazineCapture {
    pub equipment: EquipmentSourceSnapshot,
    pub program: Arc<CompiledEquipmentAttack>,
    pub enhanced: bool,
}
impl PartialEq for MagazineCapture {
    fn eq(&self, other: &Self) -> bool {
        self.equipment == other.equipment
            && self.program.fingerprint == other.program.fingerprint
            && self.enhanced == other.enhanced
    }
}
impl Eq for MagazineCapture {}
impl MagazineCapture {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        (size_of::<Self>() + 64).checked_add(self.program.retained_bytes)
    }
}
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum MagazineOperation {
    Fired,
    Reloaded,
    Reward,
}
crate::authoring::clone_shape::copy_shape!(MagazineOperation);
#[derive(Debug, Clone, serde::Serialize)]
pub struct MagazineWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub actor: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub time_ms: u64,
    pub operation: MagazineOperation,
    pub enhanced: bool,
    pub remaining: u32,
    pub capacity: u32,
    pub reload_at_ms: Option<u64>,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(MagazineWitness {
    generation,
    actor,
    equipment,
    time_ms,
    operation,
    enhanced,
    remaining,
    capacity,
    reload_at_ms,
    definition_id,
    source_file,
    field_path
});
impl MagazineWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = size_of::<Self>()
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
    pub(super) fn reconcile_equipment_magazines(&mut self) -> Result<(), GameError> {
        control::charge(
            (self.equipment_attack_state.magazines.len() as u64)
                .saturating_mul((self.units.len() + self.items.len() + 1) as u64),
            "magazine_reconcile",
        )?;
        let units = &self.units;
        let items = &self.items;
        self.equipment_attack_state.magazines.retain(|key, m| {
            let Some(old) = units.get(&m.owner) else {
                return false;
            };
            if old.lifecycle.is_dead() {
                return false;
            }
            let active = units
                .values()
                .find(|u| u.source_owned_uuid == m.employee && u.is_active());
            if let Some(u) = active {
                let item = items.values().find(|i| {
                    i.owner_unit_instance == u.instance_id
                        && MagazineKey::for_equipment(m.employee, i.source_snapshot()) == *key
                        && i.base_uuid == m.equipment.definition_uuid
                        && u.equipment_slots.as_ref().is_none_or(|s| {
                            s.slots
                                .iter()
                                .flatten()
                                .any(|slot| slot.runtime_item_instance_id == i.instance_id)
                        })
                });
                let Some(item) = item else {
                    return false;
                };
                m.owner = u.instance_id;
                m.equipment = item.source_snapshot();
                true
            } else {
                // Withdrawal intentionally removes runtime items. The battle
                // copy survives and rebinds to the next deployment of this employee.
                matches!(old.lifecycle, super::types::RuntimeUnitLifecycle::Withdrawn)
            }
        });
        Ok(())
    }
    /// Weapon membership is authority. Corrosion disables reward, not the
    /// mechanical magazine of a gun that still fires its ordinary profile.
    fn magazine_provider(
        &self,
        actor: UnitInstanceId,
    ) -> Result<Option<(EquipmentSourceSnapshot, Arc<CompiledEquipmentAttack>)>, GameError> {
        let Some(unit) = self.units.get(&actor) else {
            return Ok(None);
        };
        let catalog = self.game_data.equipment_attack_catalog();
        let slots = unit.equipment_slots.as_ref();
        control::charge(
            if slots.is_some() { 3 } else { self.items.len() } as u64,
            "magazine_provider",
        )?;
        let equipped = slots
            .into_iter()
            .flat_map(|s| s.slots.iter().flatten())
            .map(|s| {
                self.items
                    .get(&s.runtime_item_instance_id)
                    .ok_or_else(|| runtime_control::invariant("magazine_slot"))
            });
        let legacy = self
            .items
            .values()
            .take(if slots.is_some() { 0 } else { self.items.len() })
            .filter(|i| i.owner_unit_instance == actor)
            .map(Ok);
        let mut found = None;
        for item in equipped.chain(legacy) {
            let item = item?;
            if item.owner_unit_instance != actor {
                return Err(runtime_control::invariant("magazine_owner"));
            }
            let Some(p) = catalog
                .by_equipment
                .get(&item.base_uuid)
                .filter(|p| p.magazine.is_some())
            else {
                continue;
            };
            if found.is_some() {
                return Err(runtime_control::invariant("multiple_magazine_weapons"));
            }
            found = Some((item.source_snapshot(), Arc::clone(p)));
        }
        Ok(found)
    }
    fn magazine_profile_active(&self, actor: UnitInstanceId) -> bool {
        self.units.get(&actor).is_some_and(|u| {
            u.is_active()
                && matches!(u.basic_attack.action, BasicAttackAction::Damage | BasicAttackAction::DamageArea { .. })
                && self.basic_line_profile(actor).is_none()
                && !self.relation_replaces_next_basic(actor)
                && matches!(
                    u.basic_attack.delivery,
                    DeliveryDef::Projectile {
                        hit_policy: ProjectileHitPolicy::DirectionalCollision | ProjectileHitPolicy::TargetLocked,
                        ..
                    }
                )
        })
    }
    fn ensure_magazine(
        &mut self,
        actor: UnitInstanceId,
        equipment: EquipmentSourceSnapshot,
        plan: &CompiledEquipmentAttack,
    ) -> Result<MagazineKey, GameError> {
        let employee = self
            .units
            .get(&actor)
            .ok_or(GameError::UnitNotFound)?
            .source_owned_uuid;
        let copy = MagazineKey::for_equipment(employee, equipment);
        if let Some(m) = self.equipment_attack_state.magazines.get_mut(&copy) {
            if m.employee == employee && m.equipment.definition_uuid == equipment.definition_uuid {
                m.owner = actor;
                m.equipment = equipment;
                return Ok(copy);
            }
            self.equipment_attack_state.magazines.remove(&copy);
        }
        self.admit_authoring_runtime_growth(1, "magazine_entity")?;
        let before = self.equipment_attack_state.retained_extent();
        runtime_control::admit_retained_growth(
            before.unwrap_or(usize::MAX),
            before.and_then(|n| n.checked_add(size_of::<Magazine>() + 128)),
            "magazine_storage",
        )?;
        self.equipment_attack_state.magazines.insert(
            copy,
            Magazine {
                owner: actor,
                employee,
                equipment,
                remaining: plan.magazine.as_ref().unwrap().capacity,
                enhanced: false,
                reload_at: None,
                waiting: false,
            },
        );
        Ok(copy)
    }
    /// Called before accepting a new basic action. Reload owns the wake-up;
    /// no duplicate AttackStart chain is parked at an obsolete reload deadline.
    pub(super) fn equipment_magazine_ready(
        &mut self,
        actor: UnitInstanceId,
        now: u64,
        automatic: bool,
    ) -> Result<bool, GameError> {
        if !self.magazine_profile_active(actor) {
            return Ok(true);
        }
        let Some((equipment, plan)) = self.magazine_provider(actor)? else {
            return Ok(true);
        };
        self.reconcile_equipment_magazines()?;
        let copy = self.ensure_magazine(actor, equipment, &plan)?;
        if let Some(due) = self.equipment_attack_state.magazines[&copy]
            .reload_at
            .filter(|d| *d <= now)
        {
            self.finish_equipment_reload(copy, due)?;
        }
        let m = self
            .equipment_attack_state
            .magazines
            .get_mut(&copy)
            .unwrap();
        if m.remaining > 0 {
            return Ok(true);
        }
        if automatic {
            m.waiting = true;
        }
        if let Some(u) = self.units.get_mut(&actor) {
            u.pending_basic_attack = false;
        }
        control::poll("magazine_wait")?;
        Ok(false)
    }
    /// After target/range checks, before release. Ammunition is fixed once and
    /// then carried through the existing basic projectile collision path.
    pub(super) fn release_equipment_magazine(
        &mut self,
        source: &mut DamageSourceSnapshot,
        now: u64,
    ) -> Result<bool, GameError> {
        if source.source != DamageSource::BasicAttack
            || !self.magazine_profile_active(source.source_id)
        {
            return Ok(true);
        }
        let Some((equipment, plan)) = self.magazine_provider(source.source_id)? else {
            return Ok(true);
        };
        self.reconcile_equipment_magazines()?;
        let copy = self.ensure_magazine(source.source_id, equipment, &plan)?;
        let current = self.equipment_attack_state.magazines[&copy];
        if current.remaining == 0 {
            return Ok(false);
        }
        let capture = MagazineCapture {
            equipment,
            program: Arc::clone(&plan),
            enhanced: current.enhanced,
        };
        runtime_control::admit_retained_growth(0, capture.retained_extent(), "magazine_capture")?;
        let m = self
            .equipment_attack_state
            .magazines
            .get_mut(&copy)
            .unwrap();
        m.remaining -= 1;
        if m.remaining == 0 {
            let at = now
                .checked_add(u64::from(plan.magazine.as_ref().unwrap().reload_ms))
                .ok_or_else(|| runtime_control::invariant("magazine_reload_overflow"))?;
            m.reload_at = Some(at);
            self.event_queue.push(BattleEvent::MagazineReload {
                time_ms: at,
                equipment: copy,
            });
        }
        // A magazine-only program has no ordinary bonus capture to name its
        // provider. Preserve the action/cast identity while attributing this
        // actual weapon projectile (including cohort weakness) to its copy.
        source.fact_source.equipment = Some(equipment);
        source.magazine = Some(Arc::new(capture));
        self.observe_magazine(copy, &plan, now, MagazineOperation::Fired)?;
        control::poll("magazine_fired")?;
        Ok(true)
    }
    fn wake_magazine_attack(&mut self, m: Magazine, now: u64) {
        if !m.waiting {
            return;
        }
        let Some(u) = self.units.get(&m.owner).filter(|u| u.is_active()) else {
            return;
        };
        self.event_queue.push(BattleEvent::AttackStart {
            time_ms: now.max(u.next_basic_attack_ms),
            attacker_instance_id: m.owner,
            target_instance_id: None,
            schedule_next: true,
            cause: Default::default(),
        });
    }
    pub(super) fn finish_equipment_reload(
        &mut self,
        copy: MagazineKey,
        now: u64,
    ) -> Result<(), GameError> {
        self.reconcile_equipment_magazines()?;
        let Some(old) = self
            .equipment_attack_state
            .magazines
            .get(&copy)
            .copied()
            .filter(|m| m.reload_at == Some(now))
        else {
            return Ok(());
        };
        let Some(plan) = self
            .game_data
            .equipment_attack_catalog()
            .by_equipment
            .get(&old.equipment.definition_uuid)
            .cloned()
        else {
            return Ok(());
        };
        let m = self
            .equipment_attack_state
            .magazines
            .get_mut(&copy)
            .unwrap();
        m.remaining = plan.magazine.as_ref().unwrap().capacity;
        m.enhanced = false;
        m.reload_at = None;
        m.waiting = false;
        self.wake_magazine_attack(old, now);
        self.observe_magazine(copy, &plan, now, MagazineOperation::Reloaded)?;
        control::poll("magazine_reloaded")
    }
    pub(super) fn reward_cohort_magazine(
        &mut self,
        actor: UnitInstanceId,
        family: &str,
        now: u64,
    ) -> Result<(), GameError> {
        if !self
            .units
            .get(&actor)
            .is_some_and(|u| u.is_active() && !u.corroded && u.is_combatant())
        {
            return Ok(());
        }
        let Some((equipment, plan)) = self.magazine_provider(actor)? else {
            return Ok(());
        };
        let definition = plan.magazine.as_ref().unwrap();
        if definition.reward_family.as_deref() != Some(family) {
            return Ok(());
        }
        self.reconcile_equipment_magazines()?;
        let copy = self.ensure_magazine(actor, equipment, &plan)?;
        let old = self.equipment_attack_state.magazines[&copy];
        let m = self
            .equipment_attack_state
            .magazines
            .get_mut(&copy)
            .unwrap();
        m.remaining = definition.capacity;
        m.enhanced = true;
        m.reload_at = None;
        m.waiting = false;
        self.wake_magazine_attack(old, now);
        self.observe_magazine(copy, &plan, now, MagazineOperation::Reward)?;
        control::poll("magazine_rewarded")
    }
    pub(super) fn scale_magazine_collision(source: &mut DamageSourceSnapshot, first: bool) {
        let Some(capture) = &source.magazine else {
            return;
        };
        let m = capture.program.magazine.as_ref().unwrap();
        let percent = if !capture.enhanced {
            m.normal_percent
        } else if first {
            m.enhanced_primary_percent.expect("validated enhanced round")
        } else {
            m.enhanced_secondary_percent.expect("validated enhanced round")
        };
        source.base_damage = (u64::from(source.base_damage) * u64::from(percent) / 100)
            .min(u64::from(u32::MAX)) as u32;
    }
    fn observe_magazine(
        &mut self,
        copy: MagazineKey,
        plan: &CompiledEquipmentAttack,
        now: u64,
        operation: MagazineOperation,
    ) -> Result<(), GameError> {
        if self.authoring_observation.is_none() {
            return Ok(());
        }
        let m = self.equipment_attack_state.magazines[&copy];
        let origin = &plan.magazine_source;
        runtime_control::admit_retained_growth(
            0,
            Some(
                size_of::<MagazineWitness>()
                    + origin.site.action.definition_id.as_str().len()
                    + origin.source_file.len()
                    + origin.field_path.len(),
            ),
            "magazine_witness_copy",
        )?;
        let w = MagazineWitness {
            generation: self.content_identity.clone(),
            actor: m.owner,
            equipment: m.equipment,
            time_ms: now,
            operation,
            enhanced: m.enhanced,
            remaining: m.remaining,
            capacity: plan.magazine.as_ref().unwrap().capacity,
            reload_at_ms: m.reload_at,
            definition_id: origin.site.action.definition_id.to_string(),
            source_file: origin.source_file.clone(),
            field_path: origin.field_path.clone(),
        };
        runtime_control::admit_retained_growth(0, w.retained_extent(), "magazine_observation")?;
        if let Some(o) = &mut self.authoring_observation {
            o.latest_magazine = Some(w);
        }
        Ok(())
    }
}
