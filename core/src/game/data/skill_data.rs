use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};

use crate::game::{
    ability::{SkillDef, SkillId},
    data::{build_string_index, once_lock_with},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDatabase {
    pub skills: Vec<SkillDef>,

    #[serde(skip)]
    by_id: OnceLock<HashMap<SkillId, usize>>,
}

impl SkillDatabase {
    pub fn new(skills: Vec<SkillDef>) -> Self {
        let by_id = once_lock_with(build_string_index(&skills, "skill id", |skill| &skill.id));

        Self { skills, by_id }
    }

    fn by_id(&self) -> &HashMap<SkillId, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.skills, "skill id", |skill| &skill.id))
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
    }

    pub fn get_by_id(&self, id: &str) -> Option<&SkillDef> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.skills.get(index))
    }
}
