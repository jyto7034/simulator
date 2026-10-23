use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};

use crate::game::{
    data::{build_string_index, once_lock_with, DataValidationError},
    employee::EmployeeDefinition,
};

pub const STARTER_DEFINITION_MIN_COUNT: usize = 6;
pub const STARTER_DEFINITION_MAX_COUNT: usize = 6;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmployeeDefinitionDatabase {
    pub definitions: Vec<EmployeeDefinition>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
}

impl EmployeeDefinitionDatabase {
    // Only assemble here: final cross-catalog validation must precede indexes.
    pub(crate) fn from_raw_parts(definitions: Vec<EmployeeDefinition>) -> Self {
        Self {
            definitions,
            by_id: OnceLock::new(),
        }
    }

    pub fn new(definitions: Vec<EmployeeDefinition>) -> Self {
        let by_id = once_lock_with(build_string_index(
            &definitions,
            "employee definition id",
            |definition| &definition.employee_definition_id,
        ));
        Self { definitions, by_id }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id.get_or_init(|| {
            build_string_index(&self.definitions, "employee definition id", |definition| {
                &definition.employee_definition_id
            })
        })
    }

    pub(crate) fn validate_contract(&self) -> Result<(), DataValidationError> {
        let mut seen_ids = std::collections::HashSet::new();
        for (index, definition) in self.definitions.iter().enumerate() {
            let definition_id = definition.employee_definition_id.as_str();
            if definition_id.trim().is_empty() {
                return Err(DataValidationError::at(
                    "employee_definitions",
                    format!("definitions[{index}].employee_definition_id"),
                    "employee_definition_id must not be empty",
                ));
            }
            if !seen_ids.insert(definition_id) {
                return Err(DataValidationError::at(
                    definition_id,
                    format!("definitions[{index}].employee_definition_id"),
                    format!("duplicate employee_definition_id '{definition_id}'"),
                ));
            }
            for (field, value) in [
                ("name", definition.name.as_str()),
                ("profile.role", definition.profile.role.as_str()),
                ("profile.background", definition.profile.background.as_str()),
            ] {
                if value.trim().is_empty() {
                    return Err(DataValidationError::at(
                        definition_id,
                        format!("definitions[{index}].{field}"),
                        format!("{field} must not be empty"),
                    ));
                }
            }
            if definition.profile.base_combat_stats.max_hp == 0 {
                return Err(DataValidationError::at(
                    definition_id,
                    format!("definitions[{index}].profile.base_combat_stats.max_hp"),
                    "max_hp must be > 0",
                ));
            }
            if definition.profile.basic_attack_speed_percent == 0 {
                return Err(DataValidationError::at(
                    definition_id,
                    format!("definitions[{index}].profile.basic_attack_speed_percent"),
                    "basic_attack_speed_percent must be > 0",
                ));
            }
        }

        let starter_count = self
            .definitions
            .iter()
            .filter(|definition| definition.starter_eligible)
            .count();
        if !self.definitions.is_empty()
            && !(STARTER_DEFINITION_MIN_COUNT..=STARTER_DEFINITION_MAX_COUNT)
                .contains(&starter_count)
        {
            return Err(DataValidationError::at(
                "employee_definitions",
                "definitions.starter_eligible",
                format!(
                    "starter-eligible definition count must be between {STARTER_DEFINITION_MIN_COUNT} and {STARTER_DEFINITION_MAX_COUNT}, got {starter_count}"
                ),
            ));
        }
        if !self.definitions.is_empty() {
            for role in [crate::game::employee::EmployeeRole::Dealer,
                crate::game::employee::EmployeeRole::Tank,
                crate::game::employee::EmployeeRole::Healer] {
                if self.definitions.iter().filter(|definition|
                    definition.starter_eligible && definition.role == role).count() != 2 {
                    return Err(DataValidationError::at("employee_definitions",
                        "definitions.role", "starter roles must have exactly two candidates each"));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate_indexes(&self) {
        self.validate_contract()
            .unwrap_or_else(|error| panic!("invalid employee definition data: {error:?}"));
        let _ = self.by_id();
    }

    pub fn get_by_id(&self, employee_definition_id: &str) -> Option<&EmployeeDefinition> {
        self.by_id()
            .get(employee_definition_id)
            .and_then(|&index| self.definitions.get(index))
    }

    pub fn starter_definitions(&self) -> Vec<EmployeeDefinition> {
        self.definitions
            .iter()
            .filter(|definition| definition.starter_eligible)
            .cloned()
            .collect()
    }

    pub fn recruitment_definitions(&self) -> Vec<EmployeeDefinition> {
        self.definitions
            .iter()
            .filter(|definition| !definition.starter_eligible)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition_ron(id: &str, starter_eligible: bool) -> String {
        let role = match id.strip_prefix("starter_").and_then(|value| value.parse::<usize>().ok()).unwrap_or(0) % 3 {
            0 => "Dealer", 1 => "Tank", _ => "Healer",
        };
        format!(
            r#"(
                employee_definition_id: "{id}",
                role: {role},
                name: "Employee {id}",
                starter_eligible: {starter_eligible},
                initial_loadout: (equipment_ids: [], baseline_skill_fragment_ids: []),
                profile: (
                    role: "Role",
                    background: "Background",
                    personality: (risk: Cautious, social: Obedient),
                    combat_aptitude: Balanced,
                    event_aptitude: Maintenance,
                    base_combat_stats: (max_hp: 100, attack: 10, defense: 0, magic_resist: 0),
                    basic_attack_speed_percent: 100,
                ),
            )"#
        )
    }

    #[test]
    fn employee_definitions_deserialize_from_one_strict_catalog() {
        let definitions = (0..6)
            .map(|index| definition_ron(&format!("starter_{index}"), true))
            .chain(std::iter::once(definition_ron("recruitment_0", false)))
            .collect::<Vec<_>>()
            .join(",");
        let db: EmployeeDefinitionDatabase =
            ron::de::from_str(&format!("(definitions: [{definitions}])"))
                .expect("employee definitions should deserialize");

        db.validate_indexes();
        assert_eq!(db.starter_definitions().len(), 6);
        assert_eq!(db.recruitment_definitions().len(), 1);
        assert_eq!(
            db.get_by_id("recruitment_0").unwrap().name,
            "Employee recruitment_0"
        );
    }

    #[test]
    fn duplicate_definition_id_is_rejected() {
        let definitions = (0..6)
            .map(|index| definition_ron(&format!("starter_{index}"), true))
            .chain(std::iter::once(definition_ron("starter_0", false)))
            .collect::<Vec<_>>()
            .join(",");
        let db: EmployeeDefinitionDatabase =
            ron::de::from_str(&format!("(definitions: [{definitions}])")).unwrap();

        let error = db.validate_contract().unwrap_err();
        assert!(format!("{error:?}").contains("duplicate employee_definition_id"));
    }

    #[test]
    fn zero_attack_speed_is_rejected() {
        let mut definitions = (0..6)
            .map(|index| definition_ron(&format!("starter_{index}"), true))
            .collect::<Vec<_>>()
            .join(",");
        definitions = definitions.replacen(
            "basic_attack_speed_percent: 100",
            "basic_attack_speed_percent: 0",
            1,
        );
        let db: EmployeeDefinitionDatabase =
            ron::de::from_str(&format!("(definitions: [{definitions}])")).unwrap();

        let error = db.validate_contract().unwrap_err();
        assert!(format!("{error:?}").contains("basic_attack_speed_percent must be > 0"));
    }

    #[test]
    fn missing_required_profile_is_rejected_during_deserialization() {
        let error = ron::de::from_str::<EmployeeDefinitionDatabase>(
            r#"(
                definitions: [(
                    employee_definition_id: "missing_profile",
                    role: Dealer,
                    name: "Missing Profile",
                    starter_eligible: false,
                    initial_loadout: (equipment_ids: [], baseline_skill_fragment_ids: []),
                )],
            )"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("profile"));
    }

    #[test]
    fn u12_starter_count_and_each_role_count_are_independent_contracts() {
        let definitions = (0..6)
            .map(|index| definition_ron(&format!("starter_{index}"), true))
            .collect::<Vec<_>>().join(",");
        let db: EmployeeDefinitionDatabase =
            ron::de::from_str(&format!("(definitions: [{definitions}])")).unwrap();
        db.validate_contract().unwrap();
        for count in [5, 7] {
            let definitions = (0..count)
                .map(|index| definition_ron(&format!("starter_{index}"), true))
                .collect::<Vec<_>>().join(",");
            let bad: EmployeeDefinitionDatabase =
                ron::de::from_str(&format!("(definitions: [{definitions}])")).unwrap();
            assert!(bad.validate_contract().is_err(), "starter count {count}");
        }
        let imbalanced = definitions.replacen("role: Healer", "role: Dealer", 1);
        assert_ne!(imbalanced, definitions);
        let bad: EmployeeDefinitionDatabase =
            ron::de::from_str(&format!("(definitions: [{imbalanced}])")).unwrap();
        assert_eq!(bad.starter_definitions().len(), 6);
        assert!(bad.validate_contract().is_err(), "six total cannot hide 3/2/1 roles");
    }

    #[test]
    fn u12_authored_role_is_required_and_does_not_use_profile_text_as_fallback() {
        let valid = definition_ron("starter_0", true);
        let definition: EmployeeDefinition = ron::de::from_str(&valid).unwrap();
        assert_eq!(definition.role, crate::game::employee::EmployeeRole::Dealer);
        assert_eq!(definition.profile.role, "Role");
        for bad in [valid.replacen("role: Dealer,", "", 1),
            valid.replacen("role: Dealer,", "role: Unknown,", 1)] {
            assert_ne!(bad, valid);
            assert!(ron::de::from_str::<EmployeeDefinition>(&bad).is_err());
        }
    }
}
