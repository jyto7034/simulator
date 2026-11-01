//! Recipient healing and finite control durations; current equipment is read at consumption.
use super::*;
use crate::game::{
    ability::equipment_support::{CompiledEquipmentSupport, ControlDurationKind},
    enums::Side,
    value_expr::{self as expr, ExprMeter, RelationCountKind},
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReceivedHealWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub target: UnitInstanceId,
    pub actor: Option<UnitInstanceId>,
    pub equipment: EquipmentSourceSnapshot,
    pub blocked_hostiles: u32,
    pub percent: u32,
    pub other_percent: u32,
    pub before: u32,
    pub after: u32,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub incoming: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(ReceivedHealWitness {
    generation,
    time_ms,
    target,
    actor,
    equipment,
    blocked_hostiles,
    percent,
    other_percent,
    before,
    after,
    definition_id,
    source_file,
    field_path,
    incoming
});
#[derive(Debug, Clone, serde::Serialize)]
pub struct ControlDurationWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub target: UnitInstanceId,
    pub caster: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub kind: ControlDurationKind,
    pub reduction_percent: u8,
    pub original_ms: u64,
    pub effective_ms: u64,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub incoming: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(ControlDurationWitness {
    generation,
    time_ms,
    target,
    caster,
    equipment,
    kind,
    reduction_percent,
    original_ms,
    effective_ms,
    definition_id,
    source_file,
    field_path,
    incoming
});
macro_rules! witness_extent {
    ($ty:ty) => {
        impl $ty {
            pub(crate) fn retained_extent(&self) -> Option<usize> {
                let mut n = size_of::<Self>()
                    .checked_add(self.definition_id.capacity())?
                    .checked_add(self.source_file.capacity())?
                    .checked_add(self.field_path.capacity())?
                    .checked_add(self.incoming.retained_extent()?)?;
                if let Some(g) = &self.generation {
                    n = n
                        .checked_add(size_of_val(g.as_ref()))?
                        .checked_add(g.semantic_sha256.capacity())?;
                }
                Some(n)
            }
        }
    };
}
witness_extent!(ReceivedHealWitness);
witness_extent!(ControlDurationWitness);

impl BattleCore {
    pub(in crate::game::battle::core) fn visit_recipient_supports(
        &self,
        target: UnitInstanceId,
        mut visit: impl FnMut(
            EquipmentSourceSnapshot,
            &Arc<CompiledEquipmentSupport>,
        ) -> Result<(), GameError>,
    ) -> Result<(), GameError> {
        let Some(unit) = self
            .units
            .get(&target)
            .filter(|u| u.is_active() && u.is_combatant() && !u.corroded)
        else {
            return Ok(());
        };
        let catalog = self.game_data.equipment_support_catalog();
        let slots = unit.equipment_slots.as_ref();
        control::charge(
            if slots.is_some() { 3 } else { self.items.len() } as u64,
            "recipient_support_providers",
        )?;
        let slot_items = slots
            .into_iter()
            .flat_map(|s| s.slots.iter().flatten())
            .map(|s| {
                self.items
                    .get(&s.runtime_item_instance_id)
                    .ok_or_else(|| runtime_control::invariant("recipient_support_slot"))
            });
        let legacy_items = self
            .items
            .values()
            .take(if slots.is_some() { 0 } else { self.items.len() })
            .filter(|i| i.owner_unit_instance == target)
            .map(Ok);
        for item in slot_items.chain(legacy_items) {
            let item = item?;
            if item.owner_unit_instance != target {
                return Err(runtime_control::invariant("recipient_support_owner"));
            }
            if let Some(program) = catalog.by_equipment.get(&item.base_uuid) {
                visit(item.source_snapshot(), program)?;
            }
        }
        Ok(())
    }
    fn selected_received_heal(
        &self,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<Option<(EquipmentSourceSnapshot, Arc<CompiledEquipmentSupport>, u32)>, GameError>
    {
        let mut selected: Option<(EquipmentSourceSnapshot, Arc<CompiledEquipmentSupport>, u32)> =
            None;
        self.visit_recipient_supports(target, |equipment, program| {
            let Some(plan) = &program.received_heal else {
                return Ok(());
            };
            let source = RuntimeFactSource {
                actor: Some(target),
                equipment: Some(equipment),
                ..Default::default()
            };
            let frame = self
                .build_value_expr_frame(plan, &source, Some(target), now, None)
                .map_err(runtime_control::reject_expression)?;
            let mut meter = ExprMeter::new(plan.evaluation_limits())
                .map_err(|_| runtime_control::invariant("received_heal_meter"))?
                .with_host_admission(super::super::value_expr_runtime::evaluation_host_admission);
            let value = expr::evaluate(plan, &frame, &mut meter)
                .map_err(runtime_control::reject_expression)?
                .value;
            if !(0..=10_000).contains(&value.value) {
                return Err(runtime_control::reject_expression(expr::ValueExprFault {
                    code: expr::ValueExprFaultCode::Overflow,
                    site: plan.source().site.clone(),
                    node_index: 0,
                    expected_unit: Some(expr::ValueUnit::Scalar),
                    actual_unit: Some(value.unit),
                    phase: expr::ValueExprPhase::Narrow,
                    cause: "received healing increase must be from 0 to 10000 percent".into(),
                }));
            }
            let percent = value.value as u32;
            if selected.as_ref().is_none_or(|(e, _, p)| {
                percent > *p
                    || (percent == *p
                        && equipment.runtime_item_instance_id < e.runtime_item_instance_id)
            }) {
                selected = Some((equipment, Arc::clone(program), percent));
            }
            Ok(())
        })?;
        control::poll("received_heal_evaluated")?;
        Ok(selected)
    }
    pub(in crate::game::battle::core) fn received_heal_amount(
        &mut self,
        actor: Option<UnitInstanceId>,
        target: UnitInstanceId,
        amount: u32,
        now: u64,
    ) -> Result<u32, GameError> {
        let other_percent = self.target_state_received_heal_percent(target, now)?;
        let selected = if self.game_data.equipment_support_catalog().has_received_heal {
            self.selected_received_heal(target, now)?
        } else {
            None
        };
        let percent = selected.as_ref().map_or(0, |(_, _, p)| *p);
        let after = (u128::from(amount) * (100 + u128::from(other_percent) + u128::from(percent))
            / 100)
            .min(i32::MAX as u128) as u32;
        if let Some((equipment, program, percent)) =
            selected.filter(|_| self.authoring_observation.is_some())
        {
            let blocked_hostiles =
                self.relation_count(target, RelationCountKind::BlockingHostiles)?;
            let suffix = "received_heal_percent";
            self.admit_recipient_witness::<ReceivedHealWitness>(
                &program,
                suffix,
                "received_heal_witness_copy",
            )?;
            let witness = ReceivedHealWitness {
                generation: self.content_identity.clone(),
                time_ms: now,
                target,
                actor,
                equipment,
                blocked_hostiles,
                percent,
                other_percent,
                before: amount,
                after,
                definition_id: program.definition_id.clone(),
                source_file: program.source_file.clone(),
                field_path: format!("{}.{}", program.field_path, suffix),
                incoming: self.runtime_fact_source.observation(),
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "received_heal_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_received_heal = Some(witness);
        }
        Ok(after)
    }
    fn admit_recipient_witness<T>(
        &self,
        program: &CompiledEquipmentSupport,
        suffix: &str,
        site: &'static str,
    ) -> Result<(), GameError> {
        let bytes = self.runtime_fact_source.retained_extent().and_then(|n| {
            n.checked_add(
                size_of::<T>()
                    + program.definition_id.len()
                    + program.source_file.len()
                    + program.field_path.len()
                    + 1
                    + suffix.len(),
            )
        });
        runtime_control::admit_retained_growth(0, bytes, site)?;
        control::charge(bytes.ok_or(GameError::InvalidAction)? as u64, site)?;
        Ok(())
    }
    pub(in crate::game::battle::core) fn received_control_duration(
        &mut self,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        committed_side: Option<Side>,
        kind: ControlDurationKind,
        original_ms: u64,
        now: u64,
    ) -> Result<u64, GameError> {
        if original_ms == 0
            || caster == target
            || !self
                .game_data
                .equipment_support_catalog()
                .has_control_duration
        {
            return Ok(original_ms);
        }
        let side = committed_side.or_else(|| self.units.get(&caster).map(|u| u.owner));
        if !self.units.get(&target).is_some_and(|u| {
            side.is_some_and(|s| s != u.owner) && u.is_active() && u.is_combatant() && !u.corroded
        }) {
            return Ok(original_ms);
        }
        let mut selected: Option<(EquipmentSourceSnapshot, Arc<CompiledEquipmentSupport>, u8)> =
            None;
        self.visit_recipient_supports(target, |equipment, program| {
            let Some(control) = program
                .program
                .control_duration
                .as_ref()
                .filter(|p| p.kinds.contains(&kind))
            else {
                return Ok(());
            };
            let percent = control.reduction_percent;
            if selected.as_ref().is_none_or(|(e, _, p)| {
                percent > *p
                    || (percent == *p
                        && equipment.runtime_item_instance_id < e.runtime_item_instance_id)
            }) {
                selected = Some((equipment, Arc::clone(program), percent));
            }
            Ok(())
        })?;
        let Some((equipment, program, percent)) = selected else {
            return Ok(original_ms);
        };
        let effective_ms = (u128::from(original_ms) * u128::from(100 - percent))
            .div_ceil(100)
            .max(1) as u64;
        control::poll("control_duration_evaluated")?;
        if self.authoring_observation.is_some() {
            let suffix = "control_duration";
            self.admit_recipient_witness::<ControlDurationWitness>(
                &program,
                suffix,
                "control_duration_witness_copy",
            )?;
            let witness = ControlDurationWitness {
                generation: self.content_identity.clone(),
                time_ms: now,
                target,
                caster,
                equipment,
                kind,
                reduction_percent: percent,
                original_ms,
                effective_ms,
                definition_id: program.definition_id.clone(),
                source_file: program.source_file.clone(),
                field_path: format!("{}.{}", program.field_path, suffix),
                incoming: self.runtime_fact_source.observation(),
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "control_duration_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_control_duration = Some(witness);
        }
        Ok(effective_ms)
    }
}

pub(in crate::game::battle::core) fn sustained_control_kind(
    definition: &crate::game::ability::sustained::SustainedEffectDef,
) -> Option<ControlDurationKind> {
    use crate::game::ability::sustained::{
        ControlKind as C, HitStatusKind as H, SustainedEffectDef as D,
    };
    use ControlDurationKind as K;
    match definition {
        D::HitStatus { profile, .. } => match profile.kind {
            H::Stun => Some(K::Stun),
            H::Freeze => Some(K::Freeze),
            H::Root => Some(K::Root),
            H::Slow { .. } => Some(K::Slow),
            H::BasicLock => Some(K::BasicLock),
            _ => None,
        },
        D::Control { kind, .. } => Some(match kind {
            C::Root => K::Root,
            C::Slow { .. } => K::Slow,
            C::Taunt { .. } | C::ApproachTaunt { .. } => K::Taunt,
        }),
        _ => None,
    }
}
pub(in crate::game::battle::core) fn buff_control_kind(
    kind: crate::game::battle::buffs::BuffKind,
) -> Option<ControlDurationKind> {
    use crate::game::battle::buffs::BuffKind as B;
    match kind {
        B::Stun => Some(ControlDurationKind::Stun),
        B::Freeze => Some(ControlDurationKind::Freeze),
        B::Silence => Some(ControlDurationKind::Silence),
        B::PeriodicDamage { .. } => None,
    }
}
