//! Battle-local finite effect ownership. An ordinal identifies a record for
//! expiry; its private RuntimeFactSource owns the actual lifetime capability.
mod segmented;
mod structural;
pub(super) use segmented::WallCostPlan;

use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        ability::sustained::{
            compose, BasicAttackOverlayProfile, ChannelAuraDef, ControlKind, ShieldPoolView,
            SurvivalPolicy, SustainedEffectDef, SustainedEffectView, SustainedEffectsSnapshot,
            SustainedEndReason, SustainedEndReceipt,
        },
        battle::{enums::BattleEvent, ids::UnitInstanceId, runtime_facts::RuntimeFactSource},
        behavior::GameError,
        stats::{StatModifier, UnitStats},
    },
};
use std::{collections::BTreeMap, sync::Arc};
fn overflow(site: &'static str) -> GameError {
    crate::game::battle::runtime_control::reject(
        crate::game::battle::runtime_control::BattleRuntimeFault {
            code: crate::game::battle::runtime_control::RuntimeFaultCode::CounterOverflow,
            site,
            limit: None,
            requested: None,
        },
    )
}

#[derive(Debug, Clone)]
struct ShieldRechargeRecord {
    activation: u64,
    time_ms: u64,
    before: u32,
    after: u32,
    source: Arc<RuntimeFactSource>,
}
crate::authoring::clone_shape::struct_shape!(ShieldRechargeRecord {
    activation,
    time_ms,
    before,
    after,
    source
});

#[derive(Debug, Clone)]
struct Record {
    caster: UnitInstanceId,
    target: UnitInstanceId,
    expires_at: u64,
    modifiers: Vec<StatModifier>,
    combat_modifiers: Option<crate::game::ability::sustained::CombatModifierProfile>,
    aura: Option<Arc<ChannelAuraDef>>,
    shield: Option<ShieldPoolView>,
    segmented: Option<crate::game::ability::sustained::SegmentedShieldDef>,
    survival: Option<SurvivalPolicy>,
    control: Option<ControlKind>,
    parry: Option<crate::game::ability::sustained::ParryView>,
    overlay: Option<Arc<BasicAttackOverlayProfile>>,
    next_attack_only: bool,
    goodbye_skill: Option<crate::game::ability::SkillId>,
    members: Vec<UnitInstanceId>,
    source: RuntimeFactSource,
    structural: Option<Box<structural::StructuralRecord>>,
}
crate::authoring::clone_shape::struct_shape!(Record {
    caster,
    target,
    expires_at,
    modifiers,
    combat_modifiers,
    aura,
    shield,
    segmented,
    survival,
    control,
    parry,
    overlay,
    next_attack_only,
    goodbye_skill,
    members,
    source,
    structural
});
#[derive(Debug, Clone, Default)]
pub(super) struct SustainedState {
    next: u64,
    ended: u64,
    latest_end: Option<SustainedEndReceipt>,
    latest_recharge: Option<ShieldRechargeRecord>,
    latest_detachment: Option<crate::game::ability::sustained::SegmentDetachReceipt>,
    segmented_count: usize,
    regular_aura_count: usize,
    records: BTreeMap<u64, Record>,
    bases: BTreeMap<UnitInstanceId, UnitStats>,
    basic_bases: BTreeMap<UnitInstanceId, crate::game::data::abnormality_data::BasicAttackDef>,
}
crate::authoring::clone_shape::struct_shape!(SustainedState {
    next,
    ended,
    latest_end,
    latest_recharge,
    latest_detachment,
    segmented_count,
    regular_aura_count,
    records,
    bases,
    basic_bases
});
impl SustainedState {
    pub(super) fn has_use_effect(
        &self,
        action: &crate::game::battle::runtime_facts::ActionHandle,
        owns: impl Fn(&RuntimeFactSource) -> bool,
    ) -> bool {
        self.records
            .values()
            .any(|record| record.source.action.as_ref() == Some(action) && owns(&record.source))
    }
    pub(super) fn retained_extent(&self) -> Option<usize> {
        let mut size = self
            .bases
            .len()
            .checked_mul(std::mem::size_of::<UnitStats>() + 80)?;
        if let Some(receipt) = &self.latest_recharge {
            size = size
                .checked_add(std::mem::size_of_val(receipt) + 64)?
                .checked_add(receipt.source.retained_extent()?)?;
        }
        for basic in self.basic_bases.values() {
            size = size.checked_add(basic_extent(basic)?)?.checked_add(80)?;
        }
        if let Some(receipt) = &self.latest_detachment {
            size = size
                .checked_add(std::mem::size_of_val(receipt))?
                .checked_add(receipt.pool_id.capacity())?;
        }
        for record in self.records.values() {
            if let Some(policy) = &record.structural {
                size = size.checked_add(policy.extent()?)?;
            }
            if let Some(pool) = &record.segmented {
                size = size.checked_add(pool.pool_id.capacity())?;
            }
            if let Some(parry) = &record.parry {
                size = size.checked_add(parry.counter_skill.as_str().len())?;
            }
            size = size.checked_add(
                record
                    .goodbye_skill
                    .as_ref()
                    .map_or(0, |s| s.as_str().len()),
            )?;
            if let Some(profile) = &record.overlay {
                size = size.checked_add(profile_extent(profile)?)?;
            }
            size = size
                .checked_add(std::mem::size_of::<Record>() + 80)?
                .checked_add(
                    record
                        .modifiers
                        .capacity()
                        .checked_mul(std::mem::size_of::<StatModifier>())?,
                )?
                .checked_add(record.source.retained_extent()?)?
                .checked_add(
                    record
                        .members
                        .capacity()
                        .checked_mul(std::mem::size_of::<UnitInstanceId>())?,
                )?
                .checked_add(record.aura.as_ref().map_or(0, |a| {
                    std::mem::size_of::<ChannelAuraDef>() + a.family.capacity() + 64
                }))?;
        }
        Some(size)
    }
    pub(super) fn entity_count(&self) -> usize {
        self.records
            .len()
            .saturating_add(self.bases.len())
            .saturating_add(usize::from(self.latest_recharge.is_some()))
            .saturating_add(usize::from(self.latest_detachment.is_some()))
            .saturating_add(self.basic_bases.len())
            .saturating_add(
                self.records
                    .values()
                    .map(|r| r.modifiers.len() + r.members.len())
                    .sum::<usize>(),
            )
    }
    pub(super) fn len(&self) -> usize {
        self.records.len()
    }
    fn effective(&self, target: UnitInstanceId, base: UnitStats) -> UnitStats {
        compose(
            base,
            self.records
                .values()
                .filter(|r| r.target == target)
                .flat_map(|r| r.modifiers.iter().copied()),
        )
    }
}
impl BattleCore {
    pub(super) fn sustained_damage_reduction(
        &self,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<i32, GameError> {
        control::charge(
            self.sustained.records.len() as u64,
            "combat_modifier_reduction",
        )?;
        Ok(self
            .sustained
            .records
            .values()
            .filter(|r| {
                r.target == target
                    && r.expires_at > now
                    && self.units.get(&r.caster).is_some_and(|u| u.is_active())
            })
            .filter_map(|r| r.combat_modifiers)
            .fold(0i32, |total, m| {
                total.saturating_add(m.damage_reduction_percent as i32)
            }))
    }

    pub(super) fn append_sustained_lifesteal(
        &self,
        target: UnitInstanceId,
        now: u64,
        snapshots: &mut Vec<crate::game::battle::damage::BasicLifestealSnapshot>,
    ) -> Result<(), GameError> {
        use crate::game::battle::{damage::BasicLifestealSnapshot, runtime_control};
        control::charge(
            self.sustained.records.len() as u64,
            "combat_modifier_lifesteal",
        )?;
        for record in self.sustained.records.values() {
            let Some(modifiers) = record.combat_modifiers else {
                continue;
            };
            if record.target != target
                || record.expires_at <= now
                || modifiers.basic_lifesteal_percent == 0
                || !self
                    .units
                    .get(&record.caster)
                    .is_some_and(|u| u.is_active())
            {
                continue;
            }
            if snapshots.len() == snapshots.capacity() {
                runtime_control::admit_retained_growth(
                    0,
                    snapshots
                        .len()
                        .checked_add(1)
                        .and_then(|n| n.checked_mul(std::mem::size_of::<BasicLifestealSnapshot>())),
                    "sustained_lifesteal_vector",
                )?;
                snapshots.reserve_exact(1);
            }
            control::charge_serialized(&record.source, "sustained_lifesteal_source_copy")?;
            runtime_control::admit_retained_growth(
                0,
                record
                    .source
                    .retained_extent()
                    .and_then(|n| n.checked_add(std::mem::size_of::<RuntimeFactSource>() + 64)),
                "sustained_lifesteal_source",
            )?;
            snapshots.push(BasicLifestealSnapshot {
                percent: modifiers.basic_lifesteal_percent,
                max_heal: modifiers.basic_lifesteal_max_heal,
                equipment: record.source.equipment,
                fact_source: Some(Arc::new(record.source.observation())),
            });
        }
        Ok(())
    }

    pub(super) fn begin_sustained(
        &mut self,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        definition: SustainedEffectDef,
        now: u64,
    ) -> Result<(), GameError> {
        let structural = self.fixed_connection_for_source(&self.runtime_fact_source)?;
        self.begin_sustained_owned(caster, target, definition, now, structural)
    }
    fn begin_sustained_owned(
        &mut self,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        definition: SustainedEffectDef,
        now: u64,
        structural: Option<Box<structural::StructuralRecord>>,
    ) -> Result<(), GameError> {
        definition
            .validate()
            .map_err(GameError::InvalidStaticData)?;
        if !self.units.get(&caster).is_some_and(|u| u.is_active())
            || !self.units.get(&target).is_some_and(|u| u.is_active())
        {
            return Ok(());
        }
        let combat_modifiers = match &definition {
            SustainedEffectDef::CombatModifiers { modifiers, .. } => Some(*modifiers),
            _ => None,
        };
        let parry = match &definition {
            SustainedEffectDef::Parry {
                window_ms,
                invulnerable_ms,
                miss_recovery_ms,
                reduction_percent,
                counter_skill,
                ..
            } => {
                if caster != target {
                    return Err(GameError::InvalidAction);
                }
                let window_until_ms = now
                    .checked_add(u64::from(*window_ms))
                    .ok_or_else(|| overflow("parry_window"))?;
                let miss_recovery_at_ms = window_until_ms
                    .checked_add(u64::from(*miss_recovery_ms))
                    .ok_or_else(|| overflow("parry_recovery"))?;
                control::charge_serialized(counter_skill, "parry_counter_owner")?;
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    Some(counter_skill.as_str().len()),
                    "parry_counter_owner",
                )?;
                Some(crate::game::ability::sustained::ParryView {
                    window_until_ms,
                    miss_recovery_at_ms,
                    invulnerable_ms: *invulnerable_ms,
                    invulnerable_until_ms: None,
                    reduction_percent: *reduction_percent,
                    counter_skill: counter_skill.clone(),
                })
            }
            _ => None,
        };
        let active_control = match &definition {
            SustainedEffectDef::Control { kind, .. } => {
                if self.units[&caster].owner == self.units[&target].owner
                    || !self.units[&target].is_combatant()
                {
                    return Err(crate::game::battle::runtime_control::reject(crate::game::battle::runtime_control::BattleRuntimeFault { code: crate::game::battle::runtime_control::RuntimeFaultCode::InvalidControlTarget, site: "sustained_control_target", limit: None, requested: None }));
                }
                Some(*kind)
            }
            _ => None,
        };
        let survival = match &definition {
            SustainedEffectDef::Survival { policy, .. } => Some(*policy),
            _ => None,
        };
        let segmented = if let SustainedEffectDef::SegmentedShield { pool, .. } = &definition {
            if caster != target {
                return Err(GameError::InvalidAction);
            }
            self.validate_segmented_pool_available(
                caster,
                &pool.pool_id,
                self.runtime_fact_source
                    .fragment
                    .as_ref()
                    .map(|source| source.copy_uuid),
                self.runtime_fact_source
                    .equipment
                    .map(|source| source.runtime_item_instance_id),
                now,
            )?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(pool.pool_id.len()),
                "segmented_descriptor",
            )?;
            Some(crate::authoring::clone_shape::controlled_clone(pool)?)
        } else {
            None
        };
        let has_segments = segmented.is_some();
        let shield = match &definition {
            SustainedEffectDef::SegmentedShield { pool, .. } => Some(ShieldPoolView {
                capacity: pool
                    .segment_health
                    .checked_mul(pool.max_segments)
                    .ok_or(GameError::InvalidAction)?,
                remaining: pool.initial_health,
                priority: pool.priority,
                shared_radius_units: None,
            }),
            SustainedEffectDef::Shield {
                amount,
                priority,
                shared_radius_units,
                ..
            } => {
                if shared_radius_units.is_some() && caster != target {
                    return Err(GameError::InvalidAction);
                }
                Some(ShieldPoolView {
                    capacity: *amount,
                    remaining: *amount,
                    priority: *priority,
                    shared_radius_units: *shared_radius_units,
                })
            }
            _ => None,
        };
        let (duration_ms, modifiers, aura, overlay, next_attack_only, goodbye_skill) =
            match definition {
                SustainedEffectDef::Control { duration_ms, kind } => {
                    let modifiers = if let ControlKind::Slow { percent } = kind {
                        crate::game::battle::runtime_control::admit_retained_growth(
                            0,
                            Some(std::mem::size_of::<StatModifier>()),
                            "slow_modifier",
                        )?;
                        vec![StatModifier {
                            stat: crate::game::stats::StatId::MoveSpeedUnitsPerMs,
                            kind: crate::game::stats::StatModifierKind::Percent,
                            value: -(percent as i32),
                        }]
                    } else {
                        Vec::new()
                    };
                    (duration_ms, modifiers, None, None, false, None)
                }
                SustainedEffectDef::Parry { duration_ms, .. } => {
                    (duration_ms, Vec::new(), None, None, false, None)
                }
                SustainedEffectDef::Survival { duration_ms, .. } => {
                    (duration_ms, Vec::new(), None, None, false, None)
                }
                SustainedEffectDef::Shield { duration_ms, .. }
                | SustainedEffectDef::SegmentedShield { duration_ms, .. } => {
                    (duration_ms, Vec::new(), None, None, false, None)
                }
                SustainedEffectDef::Modifiers {
                    duration_ms,
                    modifiers,
                } => (duration_ms, modifiers, None, None, false, None),
                SustainedEffectDef::CombatModifiers { duration_ms, .. } => {
                    (duration_ms, Vec::new(), None, None, false, None)
                }
                SustainedEffectDef::BasicAttackOverlay {
                    duration_ms,
                    profile,
                    next_attack_only,
                    goodbye_skill,
                } => {
                    if caster != target
                        || !profile.supports_action(&self.units[&target].basic_attack.action)
                    {
                        return Err(GameError::InvalidAction);
                    }
                    control::charge_serialized(&profile, "overlay_profile_copy")?;
                    crate::game::battle::runtime_control::admit_retained_growth(
                        0,
                        profile_extent(&profile),
                        "overlay_profile_storage",
                    )?;
                    (
                        duration_ms,
                        Vec::new(),
                        None,
                        Some(Arc::new(profile)),
                        next_attack_only,
                        goodbye_skill,
                    )
                }
                SustainedEffectDef::ChannelAura { duration_ms, aura } => {
                    if target != caster
                        || self.channel_active(caster)
                        || self.unit_is_field_working(caster)
                        || self.aura_members(caster, &aura)?.is_empty()
                    {
                        return Err(GameError::InvalidAction);
                    }
                    control::charge_serialized(&aura, "aura_definition_owner")?;
                    crate::game::battle::runtime_control::admit_retained_growth(
                        0,
                        std::mem::size_of::<ChannelAuraDef>()
                            .checked_add(aura.family.capacity())
                            .and_then(|n| n.checked_add(64)),
                        "aura_definition_storage",
                    )?;
                    (
                        duration_ms,
                        Vec::new(),
                        Some(Arc::new(aura)),
                        None,
                        false,
                        None,
                    )
                }
            };
        let expires_at = now
            .checked_add(u64::from(duration_ms))
            .ok_or_else(|| overflow("sustained_deadline"))?;
        let activation = self
            .sustained
            .next
            .checked_add(1)
            .ok_or_else(|| overflow("sustained_activation"))?;
        control::charge_serialized(&modifiers, "sustained_record_copy")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            modifiers
                .len()
                .checked_mul(std::mem::size_of::<StatModifier>())
                .and_then(|n| {
                    n.checked_add(
                        std::mem::size_of::<Record>() + std::mem::size_of::<UnitStats>() + 160,
                    )
                }),
            "sustained_record_storage",
        )?;
        self.admit_authoring_runtime_growth(modifiers.len() + 2, "sustained_entities")?;
        let mut effect_source =
            crate::authoring::clone_shape::controlled_clone(&self.runtime_fact_source)?;
        if has_segments {
            // A stored pool survives its producer's action and can be consumed
            // by a later cast. It owns a private finite lifetime, not a use lease.
            effect_source = effect_source.independent_origin()?;
        }
        let source = self.reserve_finite_source(effect_source)?;
        let base = self.units[&target].stats;
        if overlay.is_some() && !self.sustained.basic_bases.contains_key(&target) {
            let original = &self.units[&target].basic_attack;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                basic_extent(original),
                "overlay_base_storage",
            )?;
            let original = crate::authoring::clone_shape::controlled_clone(original)?;
            self.sustained.basic_bases.insert(target, original);
        }
        if !modifiers.is_empty() {
            self.sustained.bases.entry(target).or_insert(base);
        }
        let is_channel = aura.is_some();
        if has_segments {
            self.sustained.segmented_count = self
                .sustained
                .segmented_count
                .checked_add(1)
                .ok_or_else(|| overflow("segmented_pool_count"))?;
        }
        self.sustained.next = activation;
        if let (Some(cast), Some(flow)) = (&source.cast, source.flow.as_ref()) {
            self.begin_skill_step_delivery(cast.cast_seq, flow.occurrence, 1);
        }
        if aura.as_ref().is_some_and(|aura| {
            aura.basic_scope
                == crate::game::ability::sustained::SupportCoefficientScope::RegularOnly
        }) {
            self.sustained.regular_aura_count = self
                .sustained
                .regular_aura_count
                .checked_add(1)
                .ok_or_else(|| overflow("aura_regular_count"))?;
        }
        self.sustained.records.insert(
            activation,
            Record {
                caster,
                target,
                expires_at,
                modifiers,
                combat_modifiers,
                aura,
                shield,
                segmented,
                survival,
                control: active_control,
                parry,
                overlay,
                next_attack_only,
                goodbye_skill,
                members: Vec::new(),
                source,
                structural,
            },
        );
        self.event_queue.push(BattleEvent::SustainedExpiry {
            time_ms: expires_at,
            activation,
        });
        self.refresh_sustained_target(target)?;
        self.refresh_basic_overlay(target)?;
        self.refresh_aura_members()?;
        if active_control == Some(ControlKind::Root) {
            self.active_movement_segments.remove(&target);
            if let Some(unit) = self.units.get_mut(&target) {
                unit.move_epoch = unit.move_epoch.wrapping_add(1);
            }
            self.record_movement_stopped(
                now,
                target,
                crate::game::battle::event_log::MovementStopReason::Rooted,
                Some(expires_at),
            );
        }
        if matches!(active_control, Some(ControlKind::Taunt { .. })) {
            self.retarget_controlled_basic(target, now);
        }
        if is_channel {
            self.event_queue.discard_channel_periodic_starts(caster)?;
            self.interrupt_movement(
                now,
                caster,
                crate::game::battle::event_log::MovementStopReason::CastStarted,
                None,
                super::movement::ActionState::Idle,
            );
            if let Some(unit) = self.units.get_mut(&caster) {
                unit.basic_attack_epoch = unit.basic_attack_epoch.wrapping_add(1);
                unit.pending_basic_attack = false;
            }
        }
        if has_segments {
            self.refresh_segmented_block_state()?;
        }
        control::poll("sustained_started")
    }
    fn refresh_sustained_target(&mut self, target: UnitInstanceId) -> Result<(), GameError> {
        control::charge(
            self.sustained.records.len() as u64 + 1,
            "sustained_composition",
        )?;
        let Some(mut base) = self.sustained.bases.get(&target).copied() else {
            return Ok(());
        };
        if let Some(unit) = self.units.get(&target) {
            base.current_health = unit.stats.current_health;
            // MaxHealth remains owned by permanent stats, damage and healing.
            base.max_health = unit.stats.max_health;
        }
        let effective = self.sustained.effective(target, base);
        if let Some(unit) = self.units.get_mut(&target) {
            unit.stats = effective;
            unit.body.move_speed = effective.move_speed_units_per_ms as f32 * 1000.0
                / super::movement::types::DATA_UNITS_PER_WORLD;
        }
        if !self.sustained.records.values().any(|r| r.target == target) {
            self.sustained.bases.remove(&target);
        }
        Ok(())
    }
    pub(super) fn apply_permanent_with_sustained(
        &mut self,
        target: UnitInstanceId,
        current: UnitStats,
        modifier: StatModifier,
    ) -> UnitStats {
        if control::charge(
            self.sustained.records.len() as u64,
            "permanent_sustained_composition",
        )
        .is_err()
        {
            return current;
        }
        let Some(base) = self.sustained.bases.get_mut(&target) else {
            let mut result = current;
            result.apply_modifier(modifier);
            return result;
        };
        base.current_health = current.current_health;
        base.max_health = current.max_health;
        base.apply_modifier(modifier);
        self.sustained
            .effective(target, *self.sustained.bases.get(&target).unwrap())
    }
    fn end_sustained(
        &mut self,
        activation: u64,
        now: u64,
        reason: SustainedEndReason,
    ) -> Result<(), GameError> {
        let Some(record) = self.sustained.records.remove(&activation) else {
            return Ok(());
        };
        if record.aura.as_ref().is_some_and(|aura| {
            aura.basic_scope
                == crate::game::ability::sustained::SupportCoefficientScope::RegularOnly
        }) {
            self.sustained.regular_aura_count = self
                .sustained
                .regular_aura_count
                .checked_sub(1)
                .ok_or_else(|| overflow("aura_regular_count"))?;
        }
        if record.segmented.is_some() {
            self.sustained.segmented_count = self
                .sustained
                .segmented_count
                .checked_sub(1)
                .ok_or_else(|| overflow("segmented_pool_count"))?;
        }
        self.event_queue.discard_sustained_expiry(activation)?;
        self.close_source_resource_delivery(record.source.resource_delivery().map(|d| d.as_ref()))?;
        if let Some(ticket) = &record.source.delivery {
            self.runtime_facts.close_delivery(ticket, now)?;
        }
        self.refresh_sustained_target(record.target)?;
        self.refresh_basic_overlay(record.target)?;
        if matches!(record.control, Some(ControlKind::Taunt { .. }))
            && reason != SustainedEndReason::Terminal
        {
            self.retarget_controlled_basic(record.target, now);
        }
        if let (Some(cast), Some(flow)) = (&record.source.cast, record.source.flow.as_ref()) {
            self.finish_skill_step_delivery(cast.cast_seq, flow.occurrence, now);
        }
        self.sustained.ended = self
            .sustained
            .ended
            .checked_add(1)
            .ok_or_else(|| overflow("sustained_end_count"))?;
        self.sustained.latest_end = Some(SustainedEndReceipt {
            activation,
            caster: record.caster,
            target: record.target,
            time_ms: now,
            reason,
        });
        if reason != SustainedEndReason::Terminal
            && record.aura.is_some()
            && self
                .units
                .get(&record.caster)
                .is_some_and(|u| u.is_active())
        {
            self.event_queue.push(BattleEvent::AttackStart {
                time_ms: now,
                attacker_instance_id: record.caster,
                target_instance_id: None,
                schedule_next: true,
                cause: Default::default(),
            });
        }
        if record.segmented.is_some() && reason != SustainedEndReason::Terminal {
            self.refresh_segmented_block_state()?;
        }
        control::poll("sustained_ended")
    }
    pub(super) fn clear_sustained(&mut self, now: u64) -> Result<(), GameError> {
        while let Some(id) = self.sustained.records.keys().next().copied() {
            self.end_sustained(id, now, SustainedEndReason::Terminal)?;
        }
        Ok(())
    }
    pub(super) fn expire_sustained(&mut self, activation: u64, now: u64) -> Result<(), GameError> {
        if self
            .sustained
            .records
            .get(&activation)
            .is_some_and(|r| r.expires_at <= now)
        {
            self.end_sustained(activation, now, SustainedEndReason::Expired)?;
        }
        Ok(())
    }
    pub(super) fn reconcile_sustained(&mut self, now: u64) -> Result<(), GameError> {
        control::charge(self.sustained.records.len() as u64, "sustained_reconcile")?;
        loop {
            control::charge(self.sustained.records.len() as u64, "sustained_end_scan")?;
            let expired = self
                .sustained
                .records
                .iter()
                .find(|(_, r)| {
                    r.expires_at <= now
                        || !self.units.get(&r.caster).is_some_and(|u| u.is_active())
                        || !self.units.get(&r.target).is_some_and(|u| u.is_active())
                        || (r
                            .structural
                            .as_ref()
                            .is_some_and(|policy| policy.personal_group.is_some())
                            && (!self.personal_source_live(r)
                                || !self.normal_player_employee(r.target)))
                })
                .map(|(id, _)| *id);
            let Some(id) = expired else {
                break;
            };
            let record = &self.sustained.records[&id];
            let reason = if record.expires_at <= now {
                SustainedEndReason::Expired
            } else if !self
                .units
                .get(&record.caster)
                .is_some_and(|u| u.is_active())
                || (record
                    .structural
                    .as_ref()
                    .is_some_and(|policy| policy.personal_group.is_some())
                    && !self.personal_source_live(record))
            {
                SustainedEndReason::SourceUnavailable
            } else {
                SustainedEndReason::TargetUnavailable
            };
            self.end_sustained(id, now, reason)?;
        }
        self.refresh_aura_members()?;
        Ok(())
    }
}

impl BattleCore {
    pub(super) fn sustained_flow_children(&self, cast_seq: u64, step: usize) -> usize {
        if control::charge(
            self.sustained.records.len() as u64,
            "sustained_flow_children",
        )
        .is_err()
        {
            return 0;
        }
        self.sustained
            .records
            .values()
            .filter(|r| {
                r.source
                    .cast
                    .as_ref()
                    .is_some_and(|c| c.cast_seq == cast_seq)
                    && r.source.flow.as_ref().is_some_and(|f| f.occurrence == step)
            })
            .count()
    }
    pub(super) fn channel_active(&self, caster: UnitInstanceId) -> bool {
        if control::charge(self.sustained.records.len() as u64, "channel_lease_lookup").is_err() {
            return true;
        }
        self.sustained
            .records
            .values()
            .any(|r| r.caster == caster && r.aura.is_some())
    }
    pub(super) fn channel_reinput(
        &self,
        caster: UnitInstanceId,
        skill: &crate::game::ability::SkillId,
    ) -> Option<u64> {
        if !self
            .units
            .get(&caster)
            .is_some_and(|u| u.is_active() && u.owner == crate::game::enums::Side::Player)
        {
            return None;
        }
        if control::charge(
            self.sustained.records.len() as u64,
            "channel_reinput_lookup",
        )
        .is_err()
        {
            return None;
        }
        self.sustained
            .records
            .iter()
            .find(|(_, r)| {
                r.caster == caster
                    && r.aura.is_some()
                    && r.source.cast.as_ref().is_some_and(|c| &c.skill_id == skill)
            })
            .map(|(id, _)| *id)
    }
    pub(super) fn stop_channel(
        &mut self,
        caster: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        self.stop_channel_with_reason(caster, now, SustainedEndReason::Stopped)
    }
    pub(super) fn stop_channel_with_reason(
        &mut self,
        caster: UnitInstanceId,
        now: u64,
        reason: SustainedEndReason,
    ) -> Result<(), GameError> {
        loop {
            control::charge(self.sustained.records.len() as u64, "channel_stop_lookup")?;
            let Some(id) = self
                .sustained
                .records
                .iter()
                .find(|(_, r)| r.caster == caster && r.aura.is_some())
                .map(|(id, _)| *id)
            else {
                break;
            };
            self.end_sustained(id, now, reason)?;
        }
        Ok(())
    }
    fn aura_members(
        &self,
        caster: UnitInstanceId,
        aura: &ChannelAuraDef,
    ) -> Result<Vec<UnitInstanceId>, GameError> {
        self.employee_members(caster, aura.radius_units, false)
    }
    fn employee_members(
        &self,
        caster: UnitInstanceId,
        radius_units: u32,
        include_caster: bool,
    ) -> Result<Vec<UnitInstanceId>, GameError> {
        control::charge(self.units.len() as u64, "aura_eligibility_scan")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.units
                .len()
                .checked_mul(std::mem::size_of::<UnitInstanceId>()),
            "aura_member_candidate",
        )?;
        let Some(owner) = self.units.get(&caster) else {
            return Ok(Vec::new());
        };
        let radius =
            f64::from(radius_units) / f64::from(super::movement::types::DATA_UNITS_PER_WORLD);
        let mut members: Vec<_> = self
            .units
            .values()
            .filter(|u| {
                (include_caster || u.instance_id != caster)
                    && u.owner == owner.owner
                    && u.is_active()
                    && u.is_combatant()
                    && u.employee_trauma.is_some()
                    && matches!(
                        u.source_identity,
                        crate::game::battle::types::BattleUnitSourceIdentity::Employee { .. }
                    )
                    && {
                        let dx = f64::from(u.body.position.x - owner.body.position.x);
                        let dy = f64::from(u.body.position.y - owner.body.position.y);
                        dx * dx + dy * dy <= radius * radius
                    }
            })
            .map(|u| u.instance_id)
            .collect();
        members.sort_unstable();
        Ok(members)
    }
    fn refresh_aura_members(&mut self) -> Result<(), GameError> {
        let mut after = None;
        loop {
            let next = match after {
                Some(id) => self
                    .sustained
                    .records
                    .range((std::ops::Bound::Excluded(id), std::ops::Bound::Unbounded))
                    .next(),
                None => self.sustained.records.iter().next(),
            };
            let Some((&id, record)) = next else {
                break;
            };
            after = Some(id);
            let fixed = record.structural.as_ref().and_then(|policy| policy.fixed);
            let (fixed_members, terminal) = if let Some(fixed) = fixed {
                let (members, terminal) = self.fixed_connection_members(record.caster, fixed)?;
                (Some(members), terminal)
            } else {
                (None, false)
            };
            let members = if let Some(members) = fixed_members {
                members
            } else if let Some(aura) = record.aura.as_ref() {
                self.aura_members(record.caster, aura)?
            } else if let Some(shield) = record.shield {
                if let Some(radius) = shield.shared_radius_units {
                    self.employee_members(record.caster, radius, true)?
                } else {
                    crate::game::battle::runtime_control::admit_retained_growth(
                        0,
                        Some(std::mem::size_of::<UnitInstanceId>()),
                        "shield_personal_member",
                    )?;
                    vec![record.target]
                }
            } else {
                continue;
            };
            let old_len = record.members.len();
            let changed = members != record.members;
            if changed {
                self.admit_authoring_runtime_replacement(
                    old_len,
                    members.len(),
                    "aura_membership",
                )?;
            }
            if terminal || changed {
                let record = self
                    .sustained
                    .records
                    .get_mut(&id)
                    .expect("selected sustained record");
                if terminal {
                    if let Some(fixed) = record
                        .structural
                        .as_mut()
                        .and_then(|policy| policy.fixed.as_mut())
                    {
                        fixed.terminal = true;
                    }
                }
                if changed {
                    record.members = members;
                }
            }
        }
        Ok(())
    }
    pub(super) fn aura_basic_bonus(&self, unit: UnitInstanceId) -> (u32, u32) {
        let count = self.sustained.records.len() as u64;
        if control::charge(count.saturating_mul(count + 1), "aura_family_resolution").is_err() {
            return (0, 0);
        }
        let mut damage = 0u32;
        let mut speed = 0u32;
        for (id, r) in &self.sustained.records {
            let Some(aura) = &r.aura else {
                continue;
            };
            if r.members.binary_search(&unit).is_err() {
                continue;
            }
            let overridden = self.sustained.records.iter().any(|(other_id, other)| {
                other.members.binary_search(&unit).is_ok()
                    && other.aura.as_ref().is_some_and(|a| {
                        a.family == aura.family
                            && (a.priority > aura.priority
                                || (a.priority == aura.priority && other_id < id))
                    })
            });
            if !overridden {
                damage = damage.saturating_add(aura.basic_damage_percent);
                speed = speed.saturating_add(aura.basic_speed_points);
            }
        }
        (damage, speed)
    }
    pub(super) fn aura_basic_bonus_for_occurrence(
        &self,
        unit: UnitInstanceId,
        regular: bool,
    ) -> Result<(u32, u32), GameError> {
        use crate::game::ability::sustained::SupportCoefficientScope;
        let eligible = |aura: &ChannelAuraDef| {
            regular || aura.basic_scope == SupportCoefficientScope::LegacyBasic
        };
        if self.sustained.regular_aura_count == 0 {
            return Ok(self.aura_basic_bonus(unit));
        }
        let count = self.sustained.records.len();
        control::charge(
            u64::try_from(
                count
                    .checked_mul(count + 1)
                    .ok_or_else(|| overflow("aura_scope_work"))?,
            )
            .map_err(|_| overflow("aura_scope_work"))?,
            "aura_scoped_family_resolution",
        )?;
        let mut damage = 0u32;
        let mut speed = 0u32;
        for (id, record) in &self.sustained.records {
            let Some(aura) = &record.aura else { continue };
            if !eligible(aura) || record.members.binary_search(&unit).is_err() {
                continue;
            }
            let overridden = self.sustained.records.iter().any(|(other_id, other)| {
                other.members.binary_search(&unit).is_ok()
                    && other.aura.as_ref().is_some_and(|other_aura| {
                        eligible(other_aura)
                            && other_aura.family == aura.family
                            && (other_aura.priority > aura.priority
                                || (other_aura.priority == aura.priority && other_id < id))
                    })
            });
            if !overridden {
                damage = damage
                    .checked_add(aura.basic_damage_percent)
                    .ok_or_else(|| overflow("aura_regular_damage"))?;
                speed = speed
                    .checked_add(aura.basic_speed_points)
                    .ok_or_else(|| overflow("aura_regular_speed"))?;
            }
        }
        Ok((damage, speed))
    }
    pub(super) fn capture_regular_support(
        &self,
        unit: UnitInstanceId,
        regular: bool,
    ) -> Result<Option<Arc<crate::game::battle::damage::RegularBasicSupportSnapshot>>, GameError>
    {
        if self.sustained.regular_aura_count == 0 {
            return Ok(None);
        }
        use crate::game::{
            ability::sustained::SupportCoefficientScope,
            battle::damage::{RegularBasicSupportContributor, RegularBasicSupportSnapshot},
        };
        let eligible = |aura: &ChannelAuraDef| {
            regular || aura.basic_scope == SupportCoefficientScope::LegacyBasic
        };
        let count = self.sustained.records.len();
        control::charge(
            (count as u64)
                .checked_mul(count as u64 + 1)
                .ok_or_else(|| overflow("aura_snapshot_work"))?,
            "aura_snapshot_winners",
        )?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            count
                .checked_mul(std::mem::size_of::<RegularBasicSupportContributor>())
                .and_then(|bytes| {
                    bytes.checked_add(std::mem::size_of::<RegularBasicSupportSnapshot>() + 64)
                }),
            "aura_release_snapshot",
        )?;
        let mut contributors = Vec::with_capacity(count);
        for (id, record) in &self.sustained.records {
            let Some(aura) = &record.aura else { continue };
            if !eligible(aura) || record.members.binary_search(&unit).is_err() {
                continue;
            }
            let overridden = self.sustained.records.iter().any(|(other_id, other)| {
                other.members.binary_search(&unit).is_ok()
                    && other.aura.as_ref().is_some_and(|other_aura| {
                        eligible(other_aura)
                            && other_aura.family == aura.family
                            && (other_aura.priority > aura.priority
                                || (other_aura.priority == aura.priority && other_id < id))
                    })
            });
            if !overridden {
                let source = record
                    .source
                    .with_regular_support(None)?
                    .independent_origin()?;
                source.admit_owner("aura_release_contributor")?;
                contributors.push(RegularBasicSupportContributor {
                    source,
                    damage_percent: aura.basic_damage_percent,
                    speed_points: aura.basic_speed_points,
                });
            }
        }
        Ok(Some(Arc::new(RegularBasicSupportSnapshot { contributors })))
    }
    pub(super) fn has_regular_support(&self) -> bool {
        self.sustained.regular_aura_count != 0
    }
    pub(super) fn sustained_basic_interval(&self, unit: UnitInstanceId, interval: u64) -> u64 {
        let speed = self
            .units
            .get(&unit)
            .map_or(100, |u| u.basic_attack_speed_percent)
            .max(1);
        let points = match self.aura_basic_bonus_for_occurrence(unit, true) {
            Ok(value) => value.1,
            Err(_) => return interval,
        };
        (u128::from(interval) * u128::from(speed)
            / u128::from(if self.sustained.regular_aura_count == 0 {
                speed.saturating_add(points)
            } else {
                match speed.checked_add(points) {
                    Some(value) => value,
                    None => {
                        let _ = overflow("aura_regular_interval");
                        return interval;
                    }
                }
            }))
        .clamp(1, u128::from(u64::MAX)) as u64
    }
    pub(super) fn validate_sustained_start(
        &self,
        caster: UnitInstanceId,
        skill: &crate::game::ability::SkillDef,
    ) -> Result<(), GameError> {
        control::charge(self.sustained.records.len() as u64, "hello_active_guard")?;
        if self.sustained.records.values().any(|r| {
            r.caster == caster
                && r.goodbye_skill.is_some()
                && r.source
                    .cast
                    .as_ref()
                    .is_some_and(|c| c.skill_id == skill.id)
        }) {
            return Err(GameError::InvalidAction);
        }
        for step in &skill.steps {
            for effect in &step.effects {
                if let crate::game::ability::SkillEffectDef::Sustained {
                    definition: SustainedEffectDef::SegmentedShield { pool, .. },
                } = &effect.op
                {
                    let fragment = if self.runtime_fact_source.reinput().is_some()
                        || self.runtime_fact_source.reaction
                    {
                        self.runtime_fact_source
                            .fragment
                            .as_ref()
                            .map(|source| source.copy_uuid)
                    } else {
                        self.units
                            .get(&caster)
                            .and_then(|unit| unit.fragment_source.as_ref())
                            .filter(|source| {
                                source.skill_id == skill.id
                                    && self.runtime_fact_source.equipment.is_none()
                            })
                            .map(|source| source.copy_uuid)
                    };
                    self.validate_segmented_pool_available(
                        caster,
                        &pool.pool_id,
                        fragment,
                        self.runtime_fact_source
                            .equipment
                            .map(|source| source.runtime_item_instance_id),
                        0,
                    )?;
                }
                if let crate::game::ability::SkillEffectDef::Sustained {
                    definition: SustainedEffectDef::BasicAttackOverlay { profile, .. },
                } = &effect.op
                {
                    if self
                        .units
                        .get(&caster)
                        .is_some_and(|unit| !profile.supports_action(&unit.basic_attack.action))
                    {
                        return Err(GameError::ActionUnavailable {
                            action: crate::game::behavior::ActionKind::ActivateSkill,
                            reason:
                                crate::game::behavior::ActionDisabledReasonCode::NoEligiblePayload,
                        });
                    }
                }
                if let crate::game::ability::SkillEffectDef::Sustained {
                    definition: SustainedEffectDef::ChannelAura { aura, .. },
                } = &effect.op
                {
                    if self.aura_members(caster, aura)?.is_empty() {
                        return Err(GameError::ActionUnavailable {
                            action: crate::game::behavior::ActionKind::ActivateSkill,
                            reason:
                                crate::game::behavior::ActionDisabledReasonCode::NoEligiblePayload,
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

impl BattleCore {
    pub fn sustained_effect_snapshot(&self) -> Result<SustainedEffectsSnapshot, GameError> {
        control::charge(self.sustained.entity_count() as u64, "sustained_query")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.sustained.retained_extent().and_then(|n| {
                n.checked_add(
                    self.sustained
                        .records
                        .len()
                        .checked_mul(std::mem::size_of::<SustainedEffectView>())?,
                )
            }),
            "sustained_query_storage",
        )?;
        if let Some(receipt) = &self.sustained.latest_recharge {
            control::charge_serialized(receipt.source.as_ref(), "shield_recharge_query")?;
        }
        let mut active = Vec::with_capacity(self.sustained.records.len());
        for (id, record) in &self.sustained.records {
            let source = &record.source;
            control::charge_serialized(source, "sustained_query_source")?;
            active.push(SustainedEffectView {
                activation: *id,
                caster: record.caster,
                target: record.target,
                expires_at_ms: record.expires_at,
                is_channel: record.aura.is_some(),
                shield: record.shield,
                personal_grant: record
                    .structural
                    .as_ref()
                    .map(|policy| policy.personal_view())
                    .transpose()?
                    .flatten(),
                fixed_connection: record
                    .structural
                    .as_ref()
                    .and_then(|policy| policy.fixed)
                    .map(|fixed| {
                        use crate::game::ability::sustained::{
                            FixedSupportConnectionState, FixedSupportConnectionView,
                        };
                        FixedSupportConnectionView {
                            recipient: fixed.recipient,
                            authored_reach_units: fixed.reach,
                            state: if fixed.terminal {
                                FixedSupportConnectionState::TerminalInvalid
                            } else if record.members.binary_search(&fixed.recipient).is_ok() {
                                FixedSupportConnectionState::InRange
                            } else {
                                FixedSupportConnectionState::OutOfRange
                            },
                        }
                    }),
                segmented: self.segmented_pool_view(record)?,
                survival: record.survival,
                control: record.control,
                parry: record.parry.clone(),
                overlay: record.overlay.as_deref().cloned(),
                next_attack_only: record.next_attack_only,
                goodbye_skill: record.goodbye_skill.clone(),
                modifiers: record.modifiers.clone(),
                combat_modifiers: record.combat_modifiers,
                members: record.members.clone(),
                authored_site: source.authored.as_ref().map(|a| a.site.clone()),
                skill_id: source.cast.as_ref().map(|c| c.skill_id.clone()),
                cast_sequence: source.cast.as_ref().map(|c| c.cast_seq),
                flow_occurrence: source.flow.as_ref().map(|f| (f.node_index, f.occurrence)),
                fragment_copy_uuid: source.fragment.as_ref().map(|f| f.copy_uuid),
            });
        }
        use crate::game::{
            ability::sustained::PassiveGuardView, data::skill_fragment_data::SkillFragmentEffectDef,
        };
        control::charge(self.units.len() as u64, "passive_guard_query")?;
        let mut passive_guards = Vec::new();
        for (unit_id, unit) in &self.units {
            if !unit.is_active() {
                continue;
            }
            let Some(fragment) = unit.fragment_source.as_ref() else {
                continue;
            };
            let Some(metadata) = self
                .game_data
                .skill_fragment_data
                .get_by_id(&fragment.definition_id)
            else {
                continue;
            };
            let SkillFragmentEffectDef::ActiveSkill {
                forms,
                passive_guard: Some(policy),
                ..
            } = &metadata.effect
            else {
                continue;
            };
            if !forms
                .get(&fragment.form_id)
                .is_some_and(|f| f.skill_id == fragment.skill_id)
            {
                continue;
            }
            control::charge(
                self.sustained.records.len() as u64,
                "passive_guard_query_windows",
            )?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                self.sustained
                    .records
                    .len()
                    .checked_mul(std::mem::size_of::<(u64, u64)>()),
                "passive_guard_query_intervals",
            )?;
            let mut intervals: Vec<_> = self
                .sustained
                .records
                .values()
                .filter(|r| r.target == *unit_id && r.source.fragment.as_deref() == Some(fragment))
                .filter_map(|r| r.parry.as_ref())
                .filter(|p| p.invulnerable_until_ms.is_none())
                .map(|p| (p.window_until_ms, p.miss_recovery_at_ms))
                .collect();
            intervals.sort_unstable();
            control::charge_serialized(fragment, "passive_guard_query_source")?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                fragment
                    .retained_extent()
                    .and_then(|n| n.checked_add(std::mem::size_of::<PassiveGuardView>())),
                "passive_guard_query_storage",
            )?;
            passive_guards.reserve_exact(1);
            passive_guards.push(PassiveGuardView {
                unit: *unit_id,
                fragment: (**fragment).clone(),
                policy: *policy,
                reduction_suppressed_intervals_ms: intervals,
            });
        }
        passive_guards.sort_by_key(|v| v.unit);
        Ok(SustainedEffectsSnapshot {
            passive_guards,
            started: self.sustained.next,
            ended: self.sustained.ended,
            latest_end: self.sustained.latest_end,
            latest_recharge: self.sustained.latest_recharge.as_ref().map(|r| {
                crate::game::ability::sustained::ShieldRechargeReceipt {
                    activation: r.activation,
                    time_ms: r.time_ms,
                    before: r.before,
                    after: r.after,
                    caster: r.source.actor,
                    skill_id: r.source.cast.as_ref().map(|c| c.skill_id.clone()),
                    cast_sequence: r.source.cast.as_ref().map(|c| c.cast_seq),
                    authored_site: r.source.authored.as_ref().map(|a| a.site.clone()),
                    fragment_copy_uuid: r.source.fragment.as_ref().map(|f| f.copy_uuid),
                    equipment_copy_uuid: r.source.equipment.and_then(|e| e.persistent_copy_uuid),
                }
            }),
            latest_detachment: crate::authoring::clone_shape::controlled_clone(
                &self.sustained.latest_detachment,
            )?,
            active,
        })
    }
}

fn pattern_extent(value: &crate::game::battle::tile_range::TileRangePattern) -> Option<usize> {
    let mut bytes = value
        .rows
        .capacity()
        .checked_mul(std::mem::size_of::<String>())?;
    for row in &value.rows {
        bytes = bytes.checked_add(row.capacity())?;
    }
    Some(bytes)
}
fn delivery_extent(value: &crate::game::ability::DeliveryDef) -> Option<usize> {
    match value {
        crate::game::ability::DeliveryDef::TileArea { footprint, .. } => pattern_extent(footprint),
        _ => Some(0),
    }
}
fn profile_extent(value: &BasicAttackOverlayProfile) -> Option<usize> {
    std::mem::size_of_val(value)
        .checked_add(64)?
        .checked_add(
            value
                .defense_tile_range
                .as_ref()
                .map_or(Some(0), pattern_extent)?,
        )?
        .checked_add(delivery_extent(&value.delivery)?)
}
pub(super) fn basic_extent(
    value: &crate::game::data::abnormality_data::BasicAttackDef,
) -> Option<usize> {
    std::mem::size_of_val(value)
        .checked_add(80)?
        .checked_add(
            value
                .defense_tile_range
                .as_ref()
                .map_or(Some(0), pattern_extent)?,
        )?
        .checked_add(delivery_extent(&value.delivery)?)?
        .checked_add(match &value.action {
            crate::game::data::abnormality_data::BasicAttackAction::Heal {
                footprint: Some(footprint),
                ..
            } => pattern_extent(footprint)?,
            _ => 0,
        })
}

#[cfg(test)]
mod basic_heal_extent_tests {
    use super::*;
    use crate::game::{
        battle::{runtime_control, tile_range::TileRangePattern},
        data::abnormality_data::{BasicAttackAction, BasicAttackDef},
    };

    #[test]
    fn basic_heal_footprint_is_owned_and_admitted_before_overlay_base_clone() {
        let mut basic = BasicAttackDef::default();
        let empty = basic_extent(&basic).unwrap();
        let mut row = String::with_capacity(4096);
        row.push('@');
        let mut rows = Vec::with_capacity(8);
        rows.push(row);
        let dynamic = rows.capacity() * std::mem::size_of::<String>() + rows[0].capacity();
        basic.action = BasicAttackAction::Heal {
            flat: 1,
            spell_power_percent: 0,
            footprint: Some(TileRangePattern {
                include_anchor_tile: true,
                rows,
            }),
        };
        assert_eq!(basic_extent(&basic), Some(empty + dynamic));
        let mut state = SustainedState::default();
        state
            .basic_bases
            .insert(uuid::Uuid::from_u128(1).into(), basic);
        assert_eq!(state.retained_extent(), Some(empty + dynamic + 80));
        let limits = runtime_control::RuntimeFactLimits {
            max_retained_bytes: empty + dynamic - 1,
            ..Default::default()
        };
        let result = runtime_control::run(limits, || {
            runtime_control::admit_retained_growth(
                0,
                basic_extent(state.basic_bases.values().next().unwrap()),
                "overlay_base_storage",
            )
        });
        assert!(matches!(result, Err(GameError::BattleRuntimeFault(f))
            if f.code == runtime_control::RuntimeFaultCode::RetainedByteLimit));
        assert_eq!(state.basic_bases.len(), 1);
    }
}

impl BattleCore {
    pub(super) fn validate_independent_form_overlays(
        &self,
        target: UnitInstanceId,
        basic: &crate::game::data::abnormality_data::BasicAttackDef,
    ) -> Result<(), GameError> {
        control::charge(
            self.sustained.records.len() as u64,
            "object_form_overlay_scan",
        )?;
        if self.sustained.records.values().any(|record| {
            record.target == target
                && record
                    .overlay
                    .as_ref()
                    .is_some_and(|overlay| !overlay.supports_action(&basic.action))
        }) {
            return Err(GameError::InvalidAction);
        }
        Ok(())
    }

    pub(super) fn rebase_independent_form_sustained(
        &mut self,
        target: UnitInstanceId,
        profile: &crate::game::battle::types::UnitCombatProfile,
    ) -> Result<(), GameError> {
        if let Some(base) = self.sustained.bases.get_mut(&target) {
            *base = profile.stats;
        }
        if self.sustained.basic_bases.contains_key(&target) {
            let basic = crate::authoring::clone_shape::controlled_clone(&profile.basic_attack)?;
            self.sustained.basic_bases.insert(target, basic);
        }
        let unit = self
            .units
            .get_mut(&target)
            .ok_or(GameError::InvalidAction)?;
        unit.stats = profile.stats;
        unit.basic_attack = crate::authoring::clone_shape::controlled_clone(&profile.basic_attack)?;
        self.refresh_sustained_target(target)?;
        self.refresh_basic_overlay(target)
    }

    fn refresh_basic_overlay(&mut self, target: UnitInstanceId) -> Result<(), GameError> {
        control::charge(self.sustained.records.len() as u64, "overlay_composition")?;
        let Some(base) = self.sustained.basic_bases.get(&target) else {
            return Ok(());
        };
        let mut basic = crate::authoring::clone_shape::controlled_clone(base)?;
        let winner = self
            .sustained
            .records
            .values()
            .rev()
            .find_map(|r| (r.target == target).then_some(r.overlay.as_ref()).flatten());
        if let Some(profile) = winner {
            control::charge_serialized(profile.as_ref(), "overlay_effective_copy")?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                profile_extent(profile.as_ref()),
                "overlay_effective_storage",
            )?;
            profile.apply(&mut basic);
        } else {
            self.sustained.basic_bases.remove(&target);
        }
        if let Some(unit) = self.units.get_mut(&target) {
            unit.basic_attack = basic;
        }
        Ok(())
    }
    pub(super) fn consume_basic_overlay(
        &mut self,
        caster: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        control::charge(
            self.sustained.records.len() as u64,
            "overlay_consume_lookup",
        )?;
        let winner = self
            .sustained
            .records
            .iter()
            .rev()
            .find(|(_, r)| r.target == caster && r.overlay.is_some());
        if let Some((&id, record)) = winner {
            if record.next_attack_only {
                self.end_sustained(id, now, SustainedEndReason::AttackReleased)?;
            }
        }
        Ok(())
    }
}

impl BattleCore {
    pub(super) fn reinput_sustained_skill(
        &mut self,
        caster: UnitInstanceId,
        skill: &crate::game::ability::SkillId,
        activation: u64,
        target: Option<crate::game::battle::event_log::SkillCastTarget>,
        now: u64,
    ) -> Result<(), GameError> {
        self.reconcile_sustained(now)?;
        let unit = self.units.get(&caster).ok_or(GameError::UnitNotFound)?;
        let record = self
            .sustained
            .records
            .get(&activation)
            .ok_or(GameError::InvalidAction)?;
        if !unit.is_active()
            || unit.owner != crate::game::enums::Side::Player
            || record.caster != caster
            || now >= record.expires_at
            || !record
                .source
                .cast
                .as_ref()
                .is_some_and(|c| &c.skill_id == skill)
        {
            return Err(GameError::InvalidAction);
        }
        if record.aura.is_some() {
            if target.is_some() {
                return Err(GameError::InvalidAction);
            }
            return self.end_sustained(activation, now, SustainedEndReason::Stopped);
        }
        let goodbye = record
            .goodbye_skill
            .as_ref()
            .ok_or(GameError::InvalidAction)?
            .clone();
        if unit.pending_skill_cast.is_some() {
            return Err(GameError::InvalidAction);
        }
        let child = self
            .cloned_skill_definition(goodbye.as_str())?
            .ok_or_else(|| {
                crate::game::battle::runtime_control::invariant("reinput_goodbye_definition")
            })?;
        let cast_target = self.reinput_cast_target(now, caster, &child, target)?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(
                std::mem::size_of::<crate::game::battle::runtime_facts::CommittedReinputContext>()
                    + 64,
            ),
            "reinput_context_storage",
        )?;
        let source = RuntimeFactSource {
            actor: Some(caster),
            fragment: record.source.fragment.clone(),
            extended_observation: RuntimeFactSource::extended_observation(
                None,
                Some(Arc::new(
                    crate::game::battle::runtime_facts::CommittedReinputContext {
                        activation,
                        committed_at_ms: now,
                        remaining_ms: record.expires_at - now,
                    },
                )),
            )?,
            resource_capabilities: RuntimeFactSource::resource_capabilities(
                self.resource_context_for_source(&record.source),
                None,
            )?,
            ..Default::default()
        };
        source.admit_owner("reinput_source_owner")?;
        self.end_sustained(activation, now, SustainedEndReason::Reinput)?;
        control::poll("reinput_after_consume")?;
        let cast_seq = self.skill_cast_seq;
        let prior = std::mem::replace(&mut self.runtime_fact_source, source);
        self.invoke_ability(
            now,
            caster,
            goodbye.as_str(),
            cast_target,
            Default::default(),
            false,
        );
        self.runtime_fact_source = prior;
        control::poll("reinput_after_invoke")?;
        if self.skill_cast_seq == cast_seq {
            return Err(crate::game::battle::runtime_control::invariant(
                "reinput_committed_child",
            ));
        }
        let event = self
            .event_queue
            .take_reinput_step(cast_seq, now)?
            .ok_or_else(|| {
                crate::game::battle::runtime_control::invariant("reinput_committed_child")
            })?;
        self.process_event(event, now)?;
        control::poll("reinput_after_execution")?;
        Ok(())
    }
}

impl BattleCore {
    /// Selection scans are stable and metered. Only the owner record changes;
    /// membership refresh cannot duplicate or recharge its balance.
    pub(super) fn absorb_sustained_shields(
        &mut self,
        target: UnitInstanceId,
        damage: u32,
        now: u64,
    ) -> Result<crate::game::battle::runtime_facts::ProtectionReceipt, GameError> {
        use crate::game::battle::runtime_facts::{ProtectionReceipt, ShieldAbsorptionFact};
        self.reconcile_sustained(now)?;
        control::charge(self.sustained.records.len() as u64, "shield_receipt_bound")?;
        let count = self
            .sustained
            .records
            .values()
            .filter(|r| r.shield.is_some() && r.members.binary_search(&target).is_ok())
            .count();
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            count.checked_mul(std::mem::size_of::<ShieldAbsorptionFact>()),
            "shield_receipt_vector",
        )?;
        let mut receipt = ProtectionReceipt {
            mitigated_damage: damage,
            ..Default::default()
        };
        if count > 0 {
            receipt.details_mut()?.shields = Vec::with_capacity(count);
        }
        let mut pending = damage;
        let mut segmented_changed = false;
        while pending > 0 {
            control::charge(self.sustained.records.len() as u64, "shield_selection")?;
            let next = self
                .sustained
                .records
                .iter()
                .filter_map(|(id, record)| {
                    let shield = record.shield?;
                    (shield.remaining > 0 && record.members.binary_search(&target).is_ok())
                        .then_some((
                            (
                                shield.shared_radius_units.is_some(),
                                std::cmp::Reverse(shield.priority),
                                *id,
                            ),
                            shield.remaining,
                        ))
                })
                .min_by_key(|v| v.0);
            let Some(((_, _, id), remaining)) = next else {
                break;
            };
            let absorbed = pending.min(remaining);
            segmented_changed |= self.sustained.records[&id].segmented.is_some();
            control::admit(
                control::Resource::EffectCommands,
                receipt.shields.len(),
                1,
                "shield_receipt_members",
            )?;
            let source = &self.sustained.records[&id].source;
            source.admit_owner("shield_receipt_source")?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                source
                    .retained_extent()
                    .and_then(|n| n.checked_add(std::mem::size_of::<ShieldAbsorptionFact>() + 64)),
                "shield_receipt_storage",
            )?;
            let source = Arc::new(source.observation());
            receipt.details_mut()?.shields.push(ShieldAbsorptionFact {
                activation: id,
                absorbed,
                remaining: remaining - absorbed,
                source,
            });
            self.sustained
                .records
                .get_mut(&id)
                .unwrap()
                .shield
                .as_mut()
                .unwrap()
                .remaining -= absorbed;
            receipt.absorbed += absorbed;
            pending -= absorbed;
            control::poll("shield_after_absorption")?;
            if absorbed == remaining {
                self.end_sustained(id, now, SustainedEndReason::Depleted)?;
            }
        }
        let hp = self
            .units
            .get(&target)
            .map_or(0, |u| u.stats.current_health);
        if hp > 0 && pending >= hp {
            control::charge(self.sustained.records.len() as u64, "survival_selection")?;
            let selected = self
                .sustained
                .records
                .iter()
                .filter_map(|(id, r)| {
                    let policy = r.survival?;
                    (r.target == target).then_some((
                        policy != SurvivalPolicy::FloorOne,
                        *id,
                        policy,
                    ))
                })
                .min_by_key(|v| (v.0, v.1));
            if let Some((_, id, policy)) = selected {
                let source = &self.sustained.records[&id].source;
                source.admit_owner("survival_receipt_source")?;
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    source.retained_extent().and_then(|n| {
                        n.checked_add(
                            std::mem::size_of::<
                                crate::game::battle::runtime_facts::SurvivalPreventionFact,
                            >() + 64,
                        )
                    }),
                    "survival_receipt_storage",
                )?;
                receipt.details_mut()?.survival = Some(Box::new(
                    crate::game::battle::runtime_facts::SurvivalPreventionFact {
                        activation: id,
                        policy,
                        source: Arc::new(source.observation()),
                    },
                ));
                receipt.prevented = pending - (hp - 1);
                if policy == SurvivalPolicy::PreventDeathOnce {
                    self.end_sustained(id, now, SustainedEndReason::Triggered)?;
                }
                control::poll("survival_after_prevention")?;
            }
        }
        if segmented_changed {
            self.refresh_segmented_block_state()?;
        }
        Ok(receipt)
    }
}

impl BattleCore {
    pub(super) fn recharge_sustained_shield(
        &mut self,
        caster: UnitInstanceId,
        source_skill: &crate::game::ability::SkillId,
        amount: u32,
        now: u64,
    ) -> Result<(), GameError> {
        self.reconcile_sustained(now)?;
        control::charge(
            self.sustained.records.len() as u64,
            "shield_recharge_lookup",
        )?;
        let id = self.sustained.records.iter().rev().find(|(_, r)| r.caster == caster && r.shield.is_some() && r.segmented.is_none() && r.structural.as_ref().is_none_or(|policy| policy.personal_group.is_none()) && r.source.cast.as_ref().is_some_and(|c| &c.skill_id == source_skill)).map(|(id, _)| *id)
            .ok_or_else(|| crate::game::battle::runtime_control::reject(crate::game::battle::runtime_control::BattleRuntimeFault { code: crate::game::battle::runtime_control::RuntimeFaultCode::ShieldUnavailable, site: "shield_recharge_owner", limit: None, requested: None }))?;
        let source = &self.runtime_fact_source;
        source.admit_owner("shield_recharge_source")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            source
                .retained_extent()
                .and_then(|n| n.checked_add(std::mem::size_of::<ShieldRechargeRecord>() + 64)),
            "shield_recharge_receipt",
        )?;
        self.admit_authoring_runtime_replacement(
            usize::from(self.sustained.latest_recharge.is_some()),
            1,
            "shield_recharge_entity",
        )?;
        let source = Arc::new(self.runtime_fact_source.observation());
        let pool = self
            .sustained
            .records
            .get_mut(&id)
            .unwrap()
            .shield
            .as_mut()
            .unwrap();
        let before = pool.remaining;
        pool.remaining = pool.remaining.saturating_add(amount).min(pool.capacity);
        self.sustained.latest_recharge = Some(ShieldRechargeRecord {
            activation: id,
            time_ms: now,
            before,
            after: pool.remaining,
            source,
        });
        control::poll("shield_after_recharge")
    }
}

impl BattleCore {
    pub(super) fn rooted(&self, target: UnitInstanceId) -> bool {
        if control::charge(self.sustained.records.len() as u64, "root_capability").is_err() {
            return false;
        }
        self.sustained
            .records
            .values()
            .any(|r| r.target == target && r.control == Some(ControlKind::Root))
    }
    pub(super) fn taunt_source(&self, target: UnitInstanceId) -> Option<UnitInstanceId> {
        control::charge(self.sustained.records.len() as u64, "taunt_target").ok()?;
        self.sustained
            .records
            .iter()
            .filter_map(|(id, r)| {
                let Some(ControlKind::Taunt { priority }) = r.control else {
                    return None;
                };
                (r.target == target && self.units.get(&r.caster).is_some_and(|u| u.is_active()))
                    .then_some((std::cmp::Reverse(priority), *id, r.caster))
            })
            .min_by_key(|v| (v.0, v.1))
            .map(|v| v.2)
    }
    fn retarget_controlled_basic(&mut self, target: UnitInstanceId, now: u64) {
        let Some(unit) = self.units.get_mut(&target).filter(|u| u.is_active()) else {
            return;
        };
        unit.current_target = None;
        unit.basic_attack_epoch = unit.basic_attack_epoch.wrapping_add(1);
        unit.basic_attack_windup_until_ms = None;
        unit.pending_basic_attack = false;
        self.event_queue.push(BattleEvent::AttackStart {
            time_ms: now,
            attacker_instance_id: target,
            target_instance_id: None,
            schedule_next: true,
            cause: Default::default(),
        });
    }

    /// Pull uses bounded cardinal cells toward the source, stopping before an
    /// obstacle or occupied segment. No autonomous player movement is enabled.
    pub(super) fn pull_enemy_toward_source(
        &mut self,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        distance: u32,
        now: u64,
    ) -> Result<(), GameError> {
        use crate::game::{
            battle::{
                core::movement::{types::WorldVec2, ActionState},
                damage::BattleCommand,
                event_log::{BattleLogEvent, MovementStopReason},
                runtime_control::{reject, BattleRuntimeFault, RuntimeFaultCode},
            },
            enums::Side,
            resources::Position,
        };
        let invalid = || {
            reject(BattleRuntimeFault {
                code: RuntimeFaultCode::InvalidControlTarget,
                site: "pull_actor_or_distance",
                limit: Some(64),
                requested: Some(u64::from(distance)),
            })
        };
        let source = self
            .units
            .get(&caster)
            .filter(|u| u.is_active())
            .ok_or_else(invalid)?;
        let unit = self
            .units
            .get(&target)
            .filter(|u| u.is_active())
            .ok_or_else(invalid)?;
        if distance == 0
            || distance > 64
            || unit.owner != Side::Opponent
            || unit.owner == source.owner
            || !unit.is_combatant()
            || unit.is_airborne()
            || unit.body.radius > 0.5
        {
            return Err(invalid());
        }
        let anchor = source.body.projected_tile();
        let from = unit.body.position;
        let radius = unit.body.radius;
        let mut tile = unit.body.projected_tile();
        let mut world = from;
        for _ in 0..distance {
            control::charge(self.units.len() as u64 + 1, "pull_cell")?;
            let dx = anchor.x - tile.x;
            let dy = anchor.y - tile.y;
            if dx == 0 && dy == 0 {
                break;
            }
            let next = if dx != 0 && dx.abs() >= dy.abs() {
                Position::new(tile.x + dx.signum(), tile.y)
            } else {
                Position::new(tile.x, tile.y + dy.signum())
            };
            if !self.battlefield.is_walkable_tile(next) {
                break;
            }
            let end = WorldVec2::from_tile_center(next);
            let vx = end.x - world.x;
            let vy = end.y - world.y;
            let length_squared = vx * vx + vy * vy;
            let occupied = self
                .units
                .values()
                .filter(|u| u.instance_id != target && u.is_active())
                .any(|u| {
                    let t = if length_squared > 0.0 {
                        (((u.body.position.x - world.x) * vx + (u.body.position.y - world.y) * vy)
                            / length_squared)
                            .clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let dx = u.body.position.x - (world.x + t * vx);
                    let dy = u.body.position.y - (world.y + t * vy);
                    let reach = radius + u.body.radius;
                    dx * dx + dy * dy < reach * reach
                });
            if occupied {
                break;
            }
            tile = next;
            world = end;
        }
        if world == from {
            return Ok(());
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<BattleCommand>()),
            "pull_interrupt_command",
        )?;
        self.process_commands_with_modifier_source(
            vec![BattleCommand::InterruptCast {
                source_id: caster,
                target_id: target,
            }],
            now,
            Some(caster),
        );
        crate::game::battle::runtime_control::poll()?;
        self.release_tactical_engagements(&[target]);
        self.active_movement_segments.remove(&target);
        self.retarget_controlled_basic(target, now);
        let unit = self.units.get_mut(&target).unwrap();
        unit.body.position = world;
        unit.move_epoch = unit.move_epoch.wrapping_add(1);
        unit.action_state = ActionState::Idle;
        self.reconcile_sustained(now)?;
        self.refresh_block_state();
        self.record_event_log(
            now,
            BattleLogEvent::UnitDisplaced {
                source_instance_id: caster,
                unit_instance_id: target,
                from_position: from.project_to_tile(),
                to_position: tile,
                from_world_position: from.quantized_milli(),
                to_world_position: world.quantized_milli(),
            },
        );
        self.record_movement_stopped(now, target, MovementStopReason::Displaced, None);
        control::poll("pull_after_displacement")
    }
}

impl BattleCore {
    pub(super) fn parry_reduction(
        &self,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<i32, GameError> {
        control::charge(self.sustained.records.len() as u64, "parry_reduction")?;
        Ok(self
            .sustained
            .records
            .values()
            .filter(|r| r.target == target && r.expires_at > now)
            .filter_map(|r| r.parry.as_ref())
            .filter(|p| {
                p.invulnerable_until_ms.is_some()
                    || now < p.window_until_ms
                    || now >= p.miss_recovery_at_ms
            })
            .fold(0i32, |sum, p| {
                sum.saturating_add(p.reduction_percent as i32)
            }))
    }
    /// Nonzero directly authored enemy damage is eligible; ticks, reaction
    /// damage and self damage never arm a counter or consume the window.
    pub(super) fn deflect_direct_strike(
        &mut self,
        target: UnitInstanceId,
        snapshot: &crate::game::battle::damage::DamageSourceSnapshot,
        now: u64,
    ) -> Result<Option<crate::game::battle::runtime_facts::DeflectionFact>, GameError> {
        use crate::game::battle::{
            damage::DamageSource,
            runtime_facts::{CounterOutcome, DeflectionFact, DeflectionKind},
        };
        self.reconcile_sustained(now)?;
        if snapshot.fact_source.reaction
            || snapshot.source_id == target
            || snapshot.source_side == self.units[&target].owner
            || snapshot.base_damage == 0
            || !matches!(
                snapshot.source,
                DamageSource::BasicAttack | DamageSource::Ability
            )
        {
            return Ok(None);
        }
        control::charge(self.sustained.records.len() as u64, "parry_selection")?;
        let selected = self
            .sustained
            .records
            .iter()
            .filter(|(_, r)| r.target == target)
            .find_map(|(id, r)| {
                let p = r.parry.as_ref()?;
                if p.invulnerable_until_ms.is_some_and(|until| now < until) {
                    Some((*id, DeflectionKind::Invulnerability))
                } else if p.invulnerable_until_ms.is_none() && now < p.window_until_ms {
                    Some((*id, DeflectionKind::Parry))
                } else {
                    None
                }
            });
        let Some((activation, kind)) = selected else {
            return Ok(None);
        };
        let record = &self.sustained.records[&activation];
        record.source.admit_owner("parry_receipt_source")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            record
                .source
                .retained_extent()
                .and_then(|n| n.checked_add(std::mem::size_of::<DeflectionFact>() + 64)),
            "parry_receipt_storage",
        )?;
        let source = Arc::new(record.source.observation());
        if kind == DeflectionKind::Parry {
            let p = self
                .sustained
                .records
                .get_mut(&activation)
                .unwrap()
                .parry
                .as_mut()
                .unwrap();
            p.invulnerable_until_ms = Some(
                now.checked_add(u64::from(p.invulnerable_ms))
                    .ok_or_else(|| overflow("parry_invulnerability"))?,
            );
        }
        control::poll("parry_after_deflection")?;
        Ok(Some(DeflectionFact {
            activation,
            kind,
            counter: CounterOutcome::NotRequested,
            source,
        }))
    }
    pub(super) fn execute_parry_counter(
        &mut self,
        fact: &mut crate::game::battle::runtime_facts::DeflectionFact,
        attacker: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        use crate::game::battle::runtime_facts::{CounterOutcome, DeflectionKind};
        if fact.kind != DeflectionKind::Parry {
            return Ok(());
        }
        let record = self
            .sustained
            .records
            .get(&fact.activation)
            .ok_or_else(|| reaction_invariant("parry_counter_owner"))?;
        let caster = record.target;
        if !self.units.get(&attacker).is_some_and(|u| u.is_active()) {
            fact.counter = CounterOutcome::AttackerUnavailable;
            return Ok(());
        }
        let parry = record
            .parry
            .as_ref()
            .ok_or_else(|| reaction_invariant("parry_counter_owner"))?;
        control::charge_serialized(&parry.counter_skill, "parry_counter_skill")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(parry.counter_skill.as_str().len()),
            "parry_counter_skill_storage",
        )?;
        let skill = parry.counter_skill.clone();
        let source = RuntimeFactSource {
            actor: Some(caster),
            reaction: true,
            equipment: record.source.equipment,
            fragment: record.source.fragment.clone(),
            resource_capabilities: RuntimeFactSource::resource_capabilities(
                self.resource_context_for_source(&record.source),
                None,
            )?,
            ..Default::default()
        };
        source.admit_owner("parry_counter_source")?;
        let cast_seq = self.skill_cast_seq;
        let prior = std::mem::replace(&mut self.runtime_fact_source, source);
        self.invoke_ability(
            now,
            caster,
            skill.as_str(),
            Some(crate::game::battle::event_log::SkillCastTarget::Unit {
                unit_instance_id: attacker,
            }),
            Default::default(),
            false,
        );
        self.runtime_fact_source = prior;
        control::poll("parry_after_counter_invoke")?;
        if self.skill_cast_seq == cast_seq {
            return Err(reaction_invariant("parry_counter_action"));
        }
        let event = self
            .event_queue
            .take_reinput_step(cast_seq, now)?
            .ok_or_else(|| reaction_invariant("parry_counter_owner"))?;
        self.process_event(event, now)?;
        fact.counter = CounterOutcome::Executed;
        control::poll("parry_after_counter_execution")
    }
}

impl BattleCore {
    /// Derived from the deployed unit's actual equipped copy, never a second
    /// passive slot or a cast-time buff. Resource charge does not gate it.
    pub(super) fn passive_parry_guard(
        &self,
        target: UnitInstanceId,
        snapshot: &crate::game::battle::damage::DamageSourceSnapshot,
        now: u64,
    ) -> Result<Option<crate::game::battle::runtime_facts::PassiveGuardFact>, GameError> {
        use crate::game::{
            battle::{
                damage::DamageSource,
                runtime_facts::{PassiveGuardFact, ReflectionOutcome},
            },
            data::skill_fragment_data::SkillFragmentEffectDef,
        };
        if snapshot.fact_source.reaction
            || snapshot.source_id == target
            || !matches!(
                snapshot.source,
                DamageSource::BasicAttack | DamageSource::Ability
            )
        {
            return Ok(None);
        }
        let Some(unit) = self
            .units
            .get(&target)
            .filter(|u| u.is_active() && u.owner != snapshot.source_side)
        else {
            return Ok(None);
        };
        let Some(fragment) = unit.fragment_source.as_ref() else {
            return Ok(None);
        };
        let Some(metadata) = self
            .game_data
            .skill_fragment_data
            .get_by_id(&fragment.definition_id)
        else {
            return Ok(None);
        };
        let SkillFragmentEffectDef::ActiveSkill {
            forms,
            passive_guard: Some(policy),
            ..
        } = &metadata.effect
        else {
            return Ok(None);
        };
        if !forms
            .get(&fragment.form_id)
            .is_some_and(|f| f.skill_id == fragment.skill_id)
        {
            return Ok(None);
        }
        control::charge(
            self.sustained.records.len() as u64,
            "passive_guard_miss_lookup",
        )?;
        let reduction_suppressed = self.sustained.records.values().any(|r| {
            r.target == target
                && r.expires_at > now
                && r.source.fragment.as_deref() == Some(fragment)
                && r.parry.as_ref().is_some_and(|p| {
                    p.invulnerable_until_ms.is_none()
                        && now >= p.window_until_ms
                        && now < p.miss_recovery_at_ms
                })
        });
        let source = RuntimeFactSource {
            actor: Some(target),
            fragment: Some(fragment.clone()),
            reaction: true,
            ..Default::default()
        };
        source.admit_owner("passive_guard_receipt_source")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            source
                .retained_extent()
                .and_then(|n| n.checked_add(std::mem::size_of::<PassiveGuardFact>() + 64)),
            "passive_guard_receipt_storage",
        )?;
        Ok(Some(PassiveGuardFact {
            policy: *policy,
            reduction_suppressed,
            roll_percent: None,
            reflected_base: 0,
            reflection: ReflectionOutcome::NotEligible,
            source: Arc::new(source.observation()),
        }))
    }
}

fn reaction_invariant(site: &'static str) -> GameError {
    crate::game::battle::runtime_control::reject(
        crate::game::battle::runtime_control::BattleRuntimeFault {
            code: crate::game::battle::runtime_control::RuntimeFaultCode::Invariant,
            site,
            limit: None,
            requested: None,
        },
    )
}

#[cfg(test)]
impl BattleCore {
    pub(crate) fn u03_missing_goodbye_definition_fixture(&mut self) {
        let record = self
            .sustained
            .records
            .values_mut()
            .find(|record| record.goodbye_skill.is_some())
            .unwrap();
        record.goodbye_skill = Some(crate::game::ability::SkillId::new(
            "u03_missing_trusted_goodbye",
        ));
    }
}
