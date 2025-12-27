use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::game::ability::{SkillDef, SkillId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDatabase {
    pub skills: Vec<SkillDef>,

    #[serde(skip)]
    by_id: HashMap<SkillId, SkillDef>,
}

impl SkillDatabase {
    pub fn new(skills: Vec<SkillDef>) -> Self {
        let by_id = skills
            .iter()
            .map(|s| (s.id.clone(), s.clone()))
            .collect::<HashMap<_, _>>();
        Self { skills, by_id }
    }

    pub fn init_map(&mut self) {
        self.by_id = self
            .skills
            .iter()
            .map(|s| (s.id.clone(), s.clone()))
            .collect();
    }

    pub fn get_by_id(&self, id: &str) -> Option<&SkillDef> {
        self.by_id.get(id)
    }
}

