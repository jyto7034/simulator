
#[cfg(test)]
impl BattleCore {
    pub(crate) fn audit_u03_missing_goodbye_definition(&mut self) {
        let record = self.sustained.records.values_mut().find(|r| r.goodbye_skill.is_some()).unwrap();
        record.goodbye_skill = Some(crate::game::ability::SkillId::new("audit_missing_trusted_goodbye"));
    }
}
