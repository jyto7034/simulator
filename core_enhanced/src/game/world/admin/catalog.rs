use serde::Serialize;
use serde_json::Value;

use crate::game::{
    behavior::GameError,
    data::skill_fragment_data::{SkillFragmentEffectDef, SkillFragmentId},
};

use super::{AdminCommandOutput, GameCore};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum AdminGrantAmountMode {
    Count,
    Amount,
    Set,
    Single,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum AdminSkillFragmentEffectType {
    BasicAttackModifier,
    ActiveSkill,
}

#[derive(Debug, Clone, Serialize)]
struct AdminEquipmentGrantCatalogEntryDto {
    id: String,
    name: String,
    equipment_type: crate::game::data::equipment_data::EquipmentType,
    rarity: crate::game::enums::RiskLevel,
    grant_command: &'static str,
    amount_mode: AdminGrantAmountMode,
}

#[derive(Debug, Clone, Serialize)]
struct AdminConsumableGrantCatalogEntryDto {
    id: String,
    name: String,
    tier: crate::game::data::consumable_data::ConsumableTier,
    rarity: crate::game::enums::RiskLevel,
    live_pool: bool,
    grant_command: &'static str,
    amount_mode: AdminGrantAmountMode,
}

#[derive(Debug, Clone, Serialize)]
struct AdminArtifactGrantCatalogEntryDto {
    id: String,
    name: String,
    rarity: crate::game::enums::RiskLevel,
    grant_command: &'static str,
    amount_mode: AdminGrantAmountMode,
}

#[derive(Debug, Clone, Serialize)]
struct AdminSkillFragmentGrantCatalogEntryDto {
    id: SkillFragmentId,
    name: String,
    rarity: crate::game::data::skill_fragment_data::SkillFragmentRarity,
    effect_type: AdminSkillFragmentEffectType,
    grant_command: &'static str,
    amount_mode: AdminGrantAmountMode,
}

#[derive(Debug, Clone, Serialize)]
struct AdminEquipmentMaterialGrantCatalogEntryDto {
    id: String,
    name: String,
    material_type: crate::game::data::equipment_data::EquipmentMaterialType,
    rarity: crate::game::enums::RiskLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    equipment_type: Option<crate::game::data::equipment_data::EquipmentType>,
    grant_command: &'static str,
    amount_mode: AdminGrantAmountMode,
}

#[derive(Debug, Clone, Serialize)]
struct AdminResourceGrantCatalogEntryDto {
    id: &'static str,
    name: &'static str,
    grant_command: &'static str,
    amount_mode: AdminGrantAmountMode,
}

#[derive(Debug, Clone, Serialize)]
struct AdminGrantCatalogDto {
    schema_version: u32,
    equipment: Vec<AdminEquipmentGrantCatalogEntryDto>,
    consumables: Vec<AdminConsumableGrantCatalogEntryDto>,
    artifacts: Vec<AdminArtifactGrantCatalogEntryDto>,
    skill_fragments: Vec<AdminSkillFragmentGrantCatalogEntryDto>,
    equipment_materials: Vec<AdminEquipmentMaterialGrantCatalogEntryDto>,
    resources: Vec<AdminResourceGrantCatalogEntryDto>,
}

fn skill_fragment_effect_type(effect: &SkillFragmentEffectDef) -> AdminSkillFragmentEffectType {
    match effect {
        SkillFragmentEffectDef::BasicAttackModifier { .. } => {
            AdminSkillFragmentEffectType::BasicAttackModifier
        }
        SkillFragmentEffectDef::ActiveSkill { .. } => AdminSkillFragmentEffectType::ActiveSkill,
    }
}

impl GameCore {
    pub(super) fn admin_dump_grant_catalog(&self) -> Result<AdminCommandOutput, GameError> {
        let payload: Value = serde_json::to_value(self.admin_grant_catalog()).map_err(|err| {
            GameError::InvalidStaticData(format!("failed to serialize admin grant catalog: {err}"))
        })?;
        Ok(self.admin_output("AdminGrantCatalog", payload))
    }

    fn admin_grant_catalog(&self) -> AdminGrantCatalogDto {
        let mut equipment = self
            .game_data
            .equipment_data
            .items
            .iter()
            .map(|item| AdminEquipmentGrantCatalogEntryDto {
                id: item.id.clone(),
                name: item.name.clone(),
                equipment_type: item.equipment_type,
                rarity: item.rarity,
                grant_command: "admin_grant_equipment",
                amount_mode: AdminGrantAmountMode::Count,
            })
            .collect::<Vec<_>>();
        equipment.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));

        let mut consumables = self
            .game_data
            .consumable_data
            .items
            .iter()
            .map(|item| AdminConsumableGrantCatalogEntryDto {
                id: item.id.clone(),
                name: item.name.clone(),
                tier: item.tier,
                rarity: item.rarity,
                live_pool: item.live_pool,
                grant_command: "admin_grant_consumable",
                amount_mode: AdminGrantAmountMode::Count,
            })
            .collect::<Vec<_>>();
        consumables.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));

        let mut artifacts = self
            .game_data
            .artifact_data
            .items
            .iter()
            .map(|item| AdminArtifactGrantCatalogEntryDto {
                id: item.id.clone(),
                name: item.name.clone(),
                rarity: item.rarity,
                grant_command: "admin_grant_artifact",
                amount_mode: AdminGrantAmountMode::Single,
            })
            .collect::<Vec<_>>();
        artifacts.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));

        let mut skill_fragments = self
            .game_data
            .skill_fragment_data
            .fragments
            .iter()
            .map(|fragment| AdminSkillFragmentGrantCatalogEntryDto {
                id: fragment.id.clone(),
                name: fragment.name.clone(),
                rarity: fragment.rarity.clone(),
                effect_type: skill_fragment_effect_type(&fragment.effect),
                grant_command: "admin_grant_skill_fragment",
                amount_mode: AdminGrantAmountMode::Count,
            })
            .collect::<Vec<_>>();
        skill_fragments.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then(left.id.as_str().cmp(right.id.as_str()))
        });

        let mut equipment_materials = self
            .game_data
            .equipment_data
            .materials
            .iter()
            .map(|material| AdminEquipmentMaterialGrantCatalogEntryDto {
                id: material.id.clone(),
                name: material.name.clone(),
                material_type: material.material_type,
                rarity: material.rarity,
                equipment_type: material.equipment_type,
                grant_command: "admin_grant_equipment_material",
                amount_mode: AdminGrantAmountMode::Amount,
            })
            .collect::<Vec<_>>();
        equipment_materials
            .sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));

        AdminGrantCatalogDto {
            schema_version: 1,
            equipment,
            consumables,
            artifacts,
            skill_fragments,
            equipment_materials,
            resources: vec![
                AdminResourceGrantCatalogEntryDto {
                    id: "fragment_dust",
                    name: "Fragment Dust",
                    grant_command: "admin_grant_fragment_dust",
                    amount_mode: AdminGrantAmountMode::Amount,
                },
                AdminResourceGrantCatalogEntryDto {
                    id: "enkephalin",
                    name: "Enkephalin",
                    grant_command: "admin_set_enkephalin",
                    amount_mode: AdminGrantAmountMode::Set,
                },
            ],
        }
    }
}
