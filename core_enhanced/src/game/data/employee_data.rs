use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};

use crate::game::{
    data::{build_string_index, once_lock_with},
    employee::StarterEmployeeCandidate,
};

pub const STARTER_CANDIDATE_MIN_COUNT: usize = 5;
pub const STARTER_CANDIDATE_MAX_COUNT: usize = 7;

fn validate_candidate_fields(candidates: &[StarterEmployeeCandidate], context: &str) {
    for candidate in candidates {
        assert!(
            !candidate.name.trim().is_empty(),
            "{context} '{}' has an empty name",
            candidate.id
        );
        assert!(
            !candidate.role.trim().is_empty(),
            "{context} '{}' has an empty role",
            candidate.id
        );
        assert!(
            !candidate.background.trim().is_empty(),
            "{context} '{}' has an empty background",
            candidate.id
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterEmployeeCandidateDatabase {
    pub candidates: Vec<StarterEmployeeCandidate>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
}

impl StarterEmployeeCandidateDatabase {
    pub fn new(candidates: Vec<StarterEmployeeCandidate>) -> Self {
        let by_id = once_lock_with(build_string_index(
            &candidates,
            "starter employee candidate id",
            |candidate| &candidate.id,
        ));
        Self { candidates, by_id }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id.get_or_init(|| {
            build_string_index(
                &self.candidates,
                "starter employee candidate id",
                |candidate| &candidate.id,
            )
        })
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        if self.candidates.is_empty() {
            return;
        }
        assert!(
            (STARTER_CANDIDATE_MIN_COUNT..=STARTER_CANDIDATE_MAX_COUNT)
                .contains(&self.candidates.len()),
            "starter employee candidate count must be between {} and {}, got {}",
            STARTER_CANDIDATE_MIN_COUNT,
            STARTER_CANDIDATE_MAX_COUNT,
            self.candidates.len()
        );
        validate_candidate_fields(&self.candidates, "starter employee candidate");
    }

    pub fn get_by_id(&self, id: &str) -> Option<&StarterEmployeeCandidate> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.candidates.get(index))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecruitmentEmployeeCandidateDatabase {
    pub candidates: Vec<StarterEmployeeCandidate>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
}

impl RecruitmentEmployeeCandidateDatabase {
    pub fn new(candidates: Vec<StarterEmployeeCandidate>) -> Self {
        let by_id = once_lock_with(build_string_index(
            &candidates,
            "recruitment employee candidate id",
            |candidate| &candidate.id,
        ));
        Self { candidates, by_id }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id.get_or_init(|| {
            build_string_index(
                &self.candidates,
                "recruitment employee candidate id",
                |candidate| &candidate.id,
            )
        })
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        validate_candidate_fields(&self.candidates, "recruitment employee candidate");
    }

    pub fn get_by_id(&self, id: &str) -> Option<&StarterEmployeeCandidate> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.candidates.get(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::employee::EmployeeGrade;

    #[test]
    fn starter_employee_candidates_deserialize_from_ron() {
        let db: StarterEmployeeCandidateDatabase = ron::de::from_str(
            r#"(
                candidates: [
                    (id: "a", name: "A", grade: Junior, role: "R", background: "B"),
                    (id: "b", name: "B", grade: Junior, role: "R", background: "B"),
                    (id: "c", name: "C", grade: Junior, role: "R", background: "B"),
                    (id: "d", name: "D", grade: Junior, role: "R", background: "B"),
                    (id: "e", name: "E", grade: Junior, role: "R", background: "B"),
                ],
            )"#,
        )
        .expect("starter candidates should deserialize");

        db.validate_indexes();
        assert_eq!(db.get_by_id("a").unwrap().grade, EmployeeGrade::Junior);
    }

    #[test]
    fn recruitment_employee_candidates_deserialize_from_separate_schema() {
        let db: RecruitmentEmployeeCandidateDatabase = ron::de::from_str(
            r#"(
                candidates: [
                    (id: "relay_guard", name: "Relay Guard", grade: Junior, role: "R", background: "B"),
                ],
            )"#,
        )
        .expect("recruitment candidates should deserialize");

        db.validate_indexes();
        assert_eq!(db.get_by_id("relay_guard").unwrap().name, "Relay Guard");
    }
}
