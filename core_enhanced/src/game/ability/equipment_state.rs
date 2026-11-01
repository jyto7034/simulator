//! Typed event state whose lifetime belongs to a worn equipment copy.
use super::{authoring::*, state_program::*};
use crate::game::{
    data::{equipment_data::EquipmentDatabase, live_ron::LiveRonError},
    value_expr::ExprLimits,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentStateBinding {
    pub equipment: String,
    pub program: StateProgram,
    #[serde(skip)]
    pub(crate) source: Option<(String, String)>,
}
impl EquipmentStateBinding {
    fn compile_at(
        &self,
        resources: &crate::game::resource::ResourceCatalog,
        file: &str,
        path: &str,
        limits: &ExprLimits,
    ) -> Result<Arc<CompiledEquipmentState>, String> {
        let source = EffectSource {
            source_file: file.into(),
            field_path: format!("{path}.program"),
            site: EffectSite {
                action: ActionSite {
                    definition_id: self.equipment.clone().into(),
                    action_id: "equipment-state".into(),
                },
                effect_id: "program".into(),
            },
            origin_definition_id: self.equipment.clone().into(),
            origin: Default::default(),
        };
        let program = self.program.compile_equipped(resources, Some(&source), limits)?;
        Ok(Arc::new(CompiledEquipmentState {
            program,
            source: Arc::new(source),
        }))
    }
}
#[derive(Debug)]
pub(crate) struct CompiledEquipmentState {
    pub program: Arc<CompiledStateProgram>,
    pub source: Arc<EffectSource>,
}
#[derive(Debug, Default)]
pub(crate) struct EquipmentStateCatalog {
    pub has_periodic: bool,
    pub has_owner_kill: bool,
    pub has_owner_cast: bool,
    pub has_owner_stun_end: bool,
    pub by_equipment: BTreeMap<uuid::Uuid, Arc<CompiledEquipmentState>>,
}
impl EquipmentStateCatalog {
    pub(crate) fn compile(
        equipment: &EquipmentDatabase,
        resources: &crate::game::resource::ResourceCatalog,
        limits: &ExprLimits,
    ) -> Result<Self, LiveRonError> {
        let mut by_equipment = BTreeMap::new();
        let mut retained = 0usize;
        for (index, binding) in equipment.equipment_states.iter().enumerate() {
            let (file, path) = binding.source.clone().unwrap_or_else(|| {
                (
                    "data/equipments/base.ron".into(),
                    format!("equipment_states[{index}]"),
                )
            });
            let error = |field: &str, cause: String| {
                LiveRonError::validation(
                    &file,
                    "equipment_states",
                    Some(binding.equipment.clone()),
                    Some(format!("{path}.{field}")),
                    cause,
                )
            };
            crate::authoring::control::charge(
                equipment.items.len() as u64 + 1,
                "equipment_state_reference",
            )
            .map_err(|e| error("equipment", format!("{e:?}")))?;
            let item = equipment
                .get_by_id(&binding.equipment)
                .ok_or_else(|| error("equipment", "state requires existing equipment".into()))?;
            if by_equipment.contains_key(&item.uuid) {
                return Err(error("equipment", "duplicate state binding".into()));
            }
            let compiled = binding
                .compile_at(resources, &file, &path, limits)
                .map_err(|e| error("program", e))?;
            crate::authoring::control::charge(
                compiled.program.compile_work,
                "equipment_state_compile",
            )
            .map_err(|e| error("program", format!("{e:?}")))?;
            let source_bytes = std::mem::size_of::<EffectSource>()
                + std::mem::size_of::<CompiledEquipmentState>()
                + file.len()
                + path.len()
                + 8
                + binding.equipment.len() * 2
                + "equipment-state".len()
                + "program".len();
            let additional = compiled
                .program
                .retained_bytes
                .checked_add(128 + source_bytes)
                .ok_or_else(|| error("program", "state catalog storage overflow".into()))?;
            crate::authoring::control::admit(
                crate::authoring::control::Resource::MetadataBytes,
                retained,
                additional,
                "equipment_state_catalog_storage",
            )
            .map_err(|e| error("program", format!("{e:?}")))?;
            retained = retained
                .checked_add(additional)
                .ok_or_else(|| error("program", "state catalog storage overflow".into()))?;
            by_equipment.insert(item.uuid, compiled);
        }
        let has_periodic = by_equipment
            .values()
            .any(|c| c.program.periodic_interval_ms.is_some());
        let has_owner_kill = by_equipment.values().any(|c| {
            c.program
                .handlers
                .iter()
                .any(|h| matches!(h.event, StateEvent::OwnerKill | StateEvent::OwnerDirectKill { .. }))
        });
        let has_owner_cast = by_equipment.values().any(|c| c.program.handlers.iter().any(|h| h.event == StateEvent::OwnerSkillCast));
        let has_owner_stun_end = by_equipment.values().any(|c| {
            c.program.handlers.iter().any(|h| h.event == StateEvent::OwnerStunEnded)
        });
        Ok(Self {
            by_equipment,
            has_periodic,
            has_owner_kill,
            has_owner_cast,
            has_owner_stun_end,
        })
    }
}

/// Observation of the wearing relationship; no invented expiry timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentStateView {
    pub activation: u64,
    pub owner: crate::game::battle::ids::UnitInstanceId,
    pub equipment_definition: uuid::Uuid,
    pub runtime_item_instance_id: uuid::Uuid,
    pub persistent_copy_uuid: Option<uuid::Uuid>,
    pub source: EffectSource,
    pub state: StateView,
}
