//! New finite personal grant family and fixed channel connection. This family
//! never joins the legacy RechargeShield latest-record selector.
use super::{
    control, overflow, BattleCore, GameError, Record, RuntimeFactSource, SustainedEffectDef,
    SustainedEndReason, UnitInstanceId,
};
use crate::game::{
    battle::runtime_facts::EquipmentSourceSnapshot,
    relic::structural::{
        authoring::{PersonalShieldGrantDef, PersonalShieldRecipientsDef},
        EffectiveEffectKey,
    },
    skill_fragment::FragmentSourceSnapshot,
};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) struct PersonalShieldGroup {
    caster: UnitInstanceId,
    equipment: Option<EquipmentSourceSnapshot>,
    fragment: Option<Arc<FragmentSourceSnapshot>>,
    effect: EffectiveEffectKey,
}
#[derive(Debug, Clone, Copy)]
pub(super) struct FixedRecipientConnection {
    pub recipient: UnitInstanceId,
    pub reach: u32,
    pub terminal: bool,
}
#[derive(Debug, Clone, Default)]
pub(super) struct StructuralRecord {
    pub personal_group: Option<Arc<PersonalShieldGroup>>,
    pub fixed: Option<FixedRecipientConnection>,
}
crate::authoring::clone_shape::copy_shape!(FixedRecipientConnection);
crate::authoring::clone_shape::struct_shape!(StructuralRecord {
    personal_group,
    fixed
});
impl StructuralRecord {
    pub(super) fn personal_view(
        &self,
    ) -> Result<Option<crate::game::ability::sustained::PersonalShieldGrantView>, GameError> {
        let Some(group) = &self.personal_group else {
            return Ok(None);
        };
        control::charge_serialized(&group.effect, "personal_grant_projection")?;
        crate::game::relic::allocation::admit_owned_projection(
            &group.effect,
            "personal_grant_projection",
        )?;
        Ok(Some(
            crate::game::ability::sustained::PersonalShieldGrantView {
                effect: group.effect.clone(),
                fragment_copy_uuid: group.fragment.as_ref().map(|fragment| fragment.copy_uuid),
                equipment_copy_uuid: group
                    .equipment
                    .and_then(|equipment| equipment.persistent_copy_uuid),
                legacy_recharge_supported: false,
            },
        ))
    }
    pub(super) fn extent(&self) -> Option<usize> {
        let mut bytes = std::mem::size_of::<Self>();
        if let Some(group) = &self.personal_group {
            bytes = bytes.checked_add(std::mem::size_of::<PersonalShieldGroup>() + 64)?;
            let site = match &group.effect {
                EffectiveEffectKey::Original { site } => site,
                EffectiveEffectKey::Generated {
                    grant_rule_id,
                    anchor,
                    local_action_id,
                    local_effect_id,
                    ..
                } => {
                    return bytes
                        .checked_add(grant_rule_id.len())?
                        .checked_add(anchor.definition_id.as_str().len())?
                        .checked_add(anchor.action_id.len())?
                        .checked_add(local_action_id.len())?
                        .checked_add(local_effect_id.len());
                }
            };
            bytes = bytes
                .checked_add(site.action.definition_id.as_str().len())?
                .checked_add(site.action.action_id.len())?
                .checked_add(site.effect_id.len())?;
        }
        Some(bytes)
    }
}
impl BattleCore {
    pub(super) fn fixed_connection_for_source(
        &self,
        source: &RuntimeFactSource,
    ) -> Result<Option<Box<StructuralRecord>>, GameError> {
        let Some(binding) = source.relic_effective() else {
            return Ok(None);
        };
        let Some(authored) = &source.authored else {
            return Ok(None);
        };
        let Some(effect) = binding.effect(&authored.site) else {
            return Ok(None);
        };
        if !effect.recipes.iter().any(|recipe| {
            matches!(recipe,
            crate::game::relic::structural::authoring::StructuralRecipeDef::FixedRecipientSupport)
        }) {
            return Ok(None);
        }
        let cast = source.cast.as_ref().ok_or(GameError::InvalidAction)?;
        let owner = self
            .active_skill_casts
            .get(&cast.cast_seq)
            .ok_or(GameError::InvalidAction)?;
        let target = owner
            .flow
            .as_ref()
            .and_then(|flow| flow.accepted_target())
            .ok_or(GameError::InvalidAction)?;
        let crate::game::battle::event_log::SkillCastTarget::Unit {
            unit_instance_id: recipient,
        } = target
        else {
            return Err(GameError::InvalidAction);
        };
        let reach = binding
            .execution()
            .selected_recipient_reach
            .ok_or(GameError::InvalidAction)?;
        if recipient == cast.caster_instance_id
            || !self.selected_player_employee_in_reach(cast.caster_instance_id, recipient, reach)
        {
            return Err(GameError::InvalidAction);
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<StructuralRecord>()),
            "fixed_connection_storage",
        )?;
        Ok(Some(Box::new(StructuralRecord {
            personal_group: None,
            fixed: Some(FixedRecipientConnection {
                recipient,
                reach,
                terminal: false,
            }),
        })))
    }
    pub(super) fn fixed_connection_members(
        &self,
        caster: UnitInstanceId,
        connection: FixedRecipientConnection,
    ) -> Result<(Vec<UnitInstanceId>, bool), GameError> {
        if connection.terminal {
            return Ok((Vec::new(), true));
        }
        let identity_live = self.normal_player_employee(connection.recipient);
        if !identity_live {
            return Ok((Vec::new(), true));
        }
        if !self.selected_player_employee_in_reach(caster, connection.recipient, connection.reach) {
            return Ok((Vec::new(), false));
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<UnitInstanceId>()),
            "fixed_connection_member",
        )?;
        Ok((vec![connection.recipient], false))
    }
    pub(super) fn normal_player_employee(&self, unit: UnitInstanceId) -> bool {
        self.units.get(&unit).is_some_and(|u| {
            u.owner == crate::game::enums::Side::Player
                && u.is_active()
                && u.is_combatant()
                && u.employee_trauma.is_some()
                && matches!(
                    u.source_identity,
                    crate::game::battle::types::BattleUnitSourceIdentity::Employee { .. }
                )
        })
    }
    pub(super) fn personal_source_live(&self, record: &Record) -> bool {
        if !self.normal_player_employee(record.caster) {
            return false;
        }
        let Some(source_actor) = record.source.actor else {
            return false;
        };
        if source_actor != record.caster {
            return false;
        }
        if let Some(equipment) = record.source.equipment {
            return self
                .items
                .get(&equipment.runtime_item_instance_id)
                .is_some_and(|item| {
                    item.owner_unit_instance == record.caster && item.source_snapshot() == equipment
                });
        }
        if let Some(fragment) = &record.source.fragment {
            return self
                .units
                .get(&record.caster)
                .is_some_and(|unit| unit.fragment_source.as_ref() == Some(fragment));
        }
        // Existing native Unit skill provider is legitimate only while that
        // exact unit still owns the same skill. This confers no relic provider.
        record.source.cast.as_ref().is_some_and(|cast| {
            self.units
                .get(&record.caster)
                .is_some_and(|unit| unit.skill_id.as_ref() == Some(&cast.skill_id))
        })
    }
    pub(in crate::game::battle::core) fn begin_personal_shield_grant(
        &mut self,
        caster: UnitInstanceId,
        definition: &PersonalShieldGrantDef,
        base_amount: u32,
        source: &RuntimeFactSource,
        now: u64,
    ) -> Result<usize, GameError> {
        if !self.normal_player_employee(caster) || source.actor != Some(caster) {
            return Err(GameError::InvalidAction);
        }
        let authored = source.authored.as_ref().ok_or(GameError::InvalidAction)?;
        let key = if let Some(effect) = source
            .relic_effective()
            .and_then(|binding| binding.effect(&authored.site))
        {
            control::charge_serialized(&effect.key, "personal_grant_effect_key")?;
            crate::game::relic::allocation::admit_owned_projection(
                &effect.key,
                "personal_grant_effect_key",
            )?;
            effect.key.clone()
        } else {
            control::charge_serialized(&authored.site, "personal_grant_effect_key")?;
            crate::game::relic::allocation::admit_owned_projection(
                &authored.site,
                "personal_grant_effect_key",
            )?;
            EffectiveEffectKey::Original {
                site: authored.site.clone(),
            }
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<PersonalShieldGroup>() + 64),
            "personal_grant_group",
        )?;
        let group = Arc::new(PersonalShieldGroup {
            caster,
            equipment: source.equipment,
            fragment: source.fragment.clone(),
            effect: key,
        });
        let members = match definition.recipients {
            PersonalShieldRecipientsDef::PlayerEmployeesAtRelease { radius_units } => {
                let members = self.employee_members(caster, radius_units, true)?;
                if members
                    .iter()
                    .any(|member| !self.normal_player_employee(*member))
                {
                    return Err(GameError::InvalidAction);
                }
                members
            }
            PersonalShieldRecipientsDef::SelectedPlayerEmployeeAtRelease {
                authored_reach_units,
            } => {
                let cast = source.cast.as_ref().ok_or(GameError::InvalidAction)?;
                let selected = self
                    .active_skill_casts
                    .get(&cast.cast_seq)
                    .and_then(|owner| owner.flow.as_ref())
                    .and_then(|flow| flow.accepted_target())
                    .ok_or(GameError::InvalidAction)?;
                let crate::game::battle::event_log::SkillCastTarget::Unit { unit_instance_id } =
                    selected
                else {
                    return Err(GameError::InvalidAction);
                };
                if !self.selected_player_employee_in_reach(
                    caster,
                    unit_instance_id,
                    authored_reach_units,
                ) {
                    return Err(GameError::InvalidAction);
                }
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    Some(std::mem::size_of::<UnitInstanceId>()),
                    "personal_grant_selected_member",
                )?;
                vec![unit_instance_id]
            }
        };
        // The caster is qualified for authored Area; Single was already checked.
        // An empty set is never promoted to successful source creation.
        if members.is_empty() || base_amount == 0 {
            return Err(GameError::InvalidAction);
        }
        let amount = if let Some(creation) = &definition.source_creation {
            use crate::game::relic::structural::{
                authoring::{EffectOperation, RelicProperty, TypedRelicValue},
                capabilities,
                compose::NumericProgram,
            };
            let descriptor = capabilities::descriptor(
                EffectOperation::PersonalShieldGrant,
                RelicProperty::PersonalShieldInitialAmount,
            )
            .ok_or(GameError::InvalidAction)?;
            let mut program = NumericProgram::default();
            for operation in &creation.operations {
                control::charge(1, "personal_source_creation")?;
                program
                    .include(*operation, descriptor)
                    .map_err(|e| GameError::InvalidStaticData(e.to_string()))?;
            }
            u32::try_from(
                program
                    .apply(TypedRelicValue::Hp(i64::from(base_amount)), descriptor)
                    .map_err(|e| GameError::InvalidStaticData(e.to_string()))?
                    .value(),
            )
            .map_err(|_| overflow("personal_source_creation"))?
        } else {
            base_amount
        };
        control::charge(
            self.sustained.records.len() as u64,
            "personal_grant_replacement_scan",
        )?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.sustained
                .records
                .len()
                .checked_mul(std::mem::size_of::<u64>()),
            "personal_grant_oldset",
        )?;
        let old: Vec<_> = self
            .sustained
            .records
            .iter()
            .filter(|(_, record)| {
                record
                    .structural
                    .as_ref()
                    .and_then(|value| value.personal_group.as_ref())
                    .is_some_and(|old| old.as_ref() == group.as_ref())
            })
            .map(|(id, _)| *id)
            .collect();
        // Admit the simultaneous old/new owner set before ending any old pool.
        self.admit_authoring_runtime_growth(
            members
                .len()
                .checked_mul(2)
                .ok_or_else(|| overflow("personal_grant_count"))?,
            "personal_grant_batch",
        )?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            members.len().checked_mul(
                std::mem::size_of::<Record>() + std::mem::size_of::<StructuralRecord>() + 160,
            ),
            "personal_grant_batch_storage",
        )?;
        let prior = self.runtime_fact_source.clone();
        self.runtime_fact_source = source.independent_origin()?;
        let operation = (|| {
            for member in &members {
                self.begin_sustained_owned(
                    caster,
                    *member,
                    SustainedEffectDef::Shield {
                        duration_ms: definition.duration_ms,
                        amount,
                        priority: definition.priority,
                        shared_radius_units: None,
                    },
                    now,
                    Some(Box::new(StructuralRecord {
                        personal_group: Some(Arc::clone(&group)),
                        fixed: None,
                    })),
                )?;
            }
            for id in old {
                self.end_sustained(id, now, SustainedEndReason::Replaced)?;
            }
            Ok(members.len())
        })();
        self.runtime_fact_source = prior;
        operation
    }
}
