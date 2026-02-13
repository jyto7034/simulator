use crate::game::{ability::SkillId, data::GameDataBase};

pub trait SkillFocusTimeProvider {
    fn focus_time_ms(&self, skill_id: &SkillId) -> Option<u32>;
}

impl SkillFocusTimeProvider for GameDataBase {
    fn focus_time_ms(&self, skill_id: &SkillId) -> Option<u32> {
        self.skill_data
            .get_by_id(skill_id.as_str())
            .map(|s| s.focus_time_ms)
    }
}
