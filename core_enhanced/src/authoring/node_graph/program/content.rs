//! Native content companions are explicit catalog inputs, never skill effects.
use super::*;
use crate::game::{
    data::equipment_data::{EquipmentDatabase, EquipmentType},
    resource::ResourceCatalog,
};

pub struct NativeGraphContent {
    pub skills: SkillDatabase,
    pub equipment: Vec<WeaponGrowthBinding>,
    pub mark_weapons: Vec<WeaponMarkBinding>,
}
pub struct NativeGraphCatalogs {
    pub skills: SkillDatabase,
    pub equipment: EquipmentDatabase,
    pub resources: ResourceCatalog,
}
impl NativeGraphContent {
    /// Pure merge; callers publish the resulting catalogs through ordinary
    /// GameData validation. Conflicts never overwrite existing declarations.
    pub fn bind_catalogs(
        &self,
        skills: &SkillDatabase,
        equipment: &EquipmentDatabase,
        resources: &ResourceCatalog,
    ) -> Result<NativeGraphCatalogs, String> {
        resources.validate().map_err(|e| e.to_string())?;
        crate::authoring::control::charge_serialized(
            &(skills, equipment, resources, &self.skills, &self.equipment, &self.mark_weapons),
            "graph_catalog_binding",
        )
        .map_err(|e| format!("{e:?}"))?;
        let mut merged_skills = skills.skills.clone();
        for skill in &self.skills.skills {
            if merged_skills.iter().any(|s| s.id == skill.id) {
                return Err(format!("Skill {} already exists", skill.id));
            }
            merged_skills.push(skill.clone());
        }
        let mut merged_equipment = equipment.clone();
        let mut merged_resources = resources.clone();
        for binding in &self.equipment {
            let item = merged_equipment
                .items
                .iter_mut()
                .find(|m| m.id == binding.weapon)
                .ok_or_else(|| format!("Growth weapon {} is unavailable", binding.weapon))?;
            if item.equipment_type != EquipmentType::Weapon {
                return Err("Growth binding requires a weapon".into());
            }
            let profile = item
                .weapon_profile
                .as_mut()
                .ok_or("Growth weapon has no combat profile")?;
            if profile
                .kill_growth
                .as_ref()
                .is_some_and(|old| old != &binding.growth)
            {
                return Err("Conflicting weapon growth declarations".into());
            }
            profile.kill_growth = Some(binding.growth.clone());
            let definition = binding.growth.resource_definition();
            if let Some(old) = merged_resources
                .resources
                .iter()
                .find(|d| d.id == definition.id)
            {
                if *old != definition {
                    return Err("Conflicting growth resource declarations".into());
                }
            } else {
                merged_resources.resources.push(definition);
            }
            binding.growth.validate_catalog(&merged_resources)?;
        }
        for binding in &self.mark_weapons {
            binding.definition.validate()?;
            let item = merged_equipment.items.iter_mut().find(|m|m.id==binding.weapon).ok_or("Mark weapon is unavailable")?;
            if item.equipment_type != EquipmentType::Weapon { return Err("Mark binding requires a weapon".into()); }
            let profile = item.weapon_profile.as_mut().ok_or("Mark weapon has no combat profile")?;
            if profile.mark_weapon.as_ref().is_some_and(|old|old != &binding.definition) { return Err("Conflicting weapon mark/evolution declarations".into()); }
            profile.mark_weapon = Some(binding.definition.clone());
        }
        merged_resources.validate().map_err(|e| e.to_string())?;
        // Preserve range presets; the loader rebinds sources and validates the
        // complete catalog rather than trusting cloned compiled indexes.
        #[derive(Serialize)]
        struct Skills<'a> {
            range_presets: &'a Vec<crate::game::data::skill_data::SkillRangePresetDef>,
            skills: Vec<crate::game::ability::SkillDef>,
        }
        let input = ron::ser::to_string(&Skills {
            range_presets: &skills.range_presets,
            skills: merged_skills,
        })
        .map_err(|e| e.to_string())?;
        let merged_skills = SkillDatabase::from_ron("compiled:graph-catalogs", &input)
            .map_err(|e| e.to_string())?;
        Ok(NativeGraphCatalogs {
            skills: merged_skills,
            equipment: merged_equipment,
            resources: merged_resources,
        })
    }
}
