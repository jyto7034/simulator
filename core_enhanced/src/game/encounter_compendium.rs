use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::game::{
    battle::core::CompendiumCandidate, boss_omen::BossOmenRunState, data::GameDataBase,
};

/// Persisted game-global encounter knowledge.
///
/// Only canonical IDs are stored. Display metadata is always resolved from the
/// current strict live catalogs when a snapshot is projected. Persisted IDs
/// may be dormant (temporarily absent from the current catalog).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterCompendiumProfileState {
    pub registered_ids: BTreeSet<String>,
}

/// One local-install permanent profile. Run checkpoints never contain this state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerProfileState {
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub facility_knowledge: std::collections::BTreeMap<String, crate::game::map::FacilityKnowledge>,
    pub encounter_compendium: EncounterCompendiumProfileState,
    pub highest_floor_reached: u32,
    pub longest_deathless_floor_streak: u32,
    pub all_live_final_bosses_defeated: bool,
    pub lifecycle: crate::game::expedition_lifecycle::ExpeditionLifecycle,
}

crate::authoring::clone_shape::struct_shape!(EncounterCompendiumProfileState { registered_ids });
crate::authoring::clone_shape::struct_shape!(PlayerProfileState {
    facility_knowledge,
    encounter_compendium,
    highest_floor_reached,
    longest_deathless_floor_streak,
    all_live_final_bosses_defeated,
    lifecycle,
});

impl PlayerProfileState {
    pub fn validate_persisted(&self) -> Result<(), String> {
        self.encounter_compendium.validate_persisted()?;
        self.lifecycle.validate()
    }

    /// Knowledge-only import must never silently ignore different exact assets.
    pub fn merge(&mut self, committed: impl Into<Self>) -> Result<(), String> {
        let committed = committed.into();
        committed.validate_persisted()?;
        if self.lifecycle != committed.lifecycle {
            return Err("profile lifecycle requires authoritative installation or accepted publication".into());
        }
        self.merge_knowledge(&committed);
        Ok(())
    }

    pub fn merge_knowledge(&mut self, committed: &Self) {
        for (layout, knowledge) in &committed.facility_knowledge {
            let current = self.facility_knowledge.entry(layout.clone()).or_default();
            current.completed_rooms.extend(knowledge.completed_rooms.iter().cloned());
            current.discovered_final_rooms.extend(knowledge.discovered_final_rooms.iter().cloned());
        }
        self.encounter_compendium
            .merge(committed.encounter_compendium.clone());
        self.highest_floor_reached = self
            .highest_floor_reached
            .max(committed.highest_floor_reached);
        self.longest_deathless_floor_streak = self
            .longest_deathless_floor_streak
            .max(committed.longest_deathless_floor_streak);
        self.all_live_final_bosses_defeated |= committed.all_live_final_bosses_defeated;
    }

    /// Only an accepted paired publication may adopt decreasing copy ownership.
    pub fn adopt_accepted(&mut self, mut committed: Self) -> Result<(), String> {
        committed.validate_persisted()?;
        committed.merge_knowledge(self);
        *self = committed;
        Ok(())
    }

    pub fn record_floor_reached(&mut self, floor_number: u32) {
        self.highest_floor_reached = self.highest_floor_reached.max(floor_number);
    }

    pub fn record_deathless_streak(&mut self, completed_floor_streak: u32) {
        self.longest_deathless_floor_streak = self
            .longest_deathless_floor_streak
            .max(completed_floor_streak);
    }

    pub fn refresh_all_live_final_bosses_defeated(
        &mut self,
        game_data: &GameDataBase,
        boss_omen: &BossOmenRunState,
    ) {
        if self.all_live_final_bosses_defeated {
            return;
        }
        let live_final_bosses = game_data
            .boss_omen_data
            .chains
            .iter()
            .map(|chain| chain.id.clone())
            .collect::<BTreeSet<_>>();
        self.all_live_final_bosses_defeated = !live_final_bosses.is_empty()
            && live_final_bosses
                .iter()
                .all(|chain_id| boss_omen.completed_chain_ids.contains(chain_id));
    }
}

impl From<EncounterCompendiumProfileState> for PlayerProfileState {
    fn from(encounter_compendium: EncounterCompendiumProfileState) -> Self {
        Self {
            encounter_compendium,
            ..Self::default()
        }
    }
}

impl EncounterCompendiumProfileState {
    pub fn from_victory_candidates(candidates: &[CompendiumCandidate]) -> Self {
        Self {
            registered_ids: candidates
                .iter()
                .map(|candidate| candidate.id.clone())
                .collect(),
        }
    }

    pub fn validate_persisted(&self) -> Result<(), String> {
        for id in &self.registered_ids {
            validate_profile_id("registered", id)?;
        }
        Ok(())
    }

    pub fn validate_current_catalog_commit(&self, game_data: &GameDataBase) -> Result<(), String> {
        self.validate_persisted()?;
        for id in &self.registered_ids {
            let is_abnormality = game_data.abnormality_data.get_by_id(id).is_some();
            let is_corrosion_profile = game_data.corroded_employee_data.get_by_id(id).is_some();
            if !is_abnormality && !is_corrosion_profile {
                return Err(format!(
                    "encounter compendium references missing canonical ID '{id}'"
                ));
            }
        }
        Ok(())
    }

    pub fn merge(&mut self, committed: Self) {
        self.registered_ids.extend(committed.registered_ids);
    }
}

fn validate_profile_id(kind: &str, id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.trim() != id
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(format!(
            "encounter compendium contains an invalid {kind} id '{id}'"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::game::battle::core::CompendiumCandidateKind;

    #[test]
    fn victory_candidates_collapse_to_one_registered_set() {
        let candidates = vec![
            CompendiumCandidate {
                id: "abnormality_alpha".to_string(),
                kind: CompendiumCandidateKind::Abnormality,
                base_uuid: Uuid::from_u128(1),
            },
            CompendiumCandidate {
                id: "corrosion_beta".to_string(),
                kind: CompendiumCandidateKind::CorrodedEmployee,
                base_uuid: Uuid::from_u128(2),
            },
            CompendiumCandidate {
                id: "abnormality_alpha".to_string(),
                kind: CompendiumCandidateKind::Abnormality,
                base_uuid: Uuid::from_u128(1),
            },
        ];
        assert_eq!(
            EncounterCompendiumProfileState::from_victory_candidates(&candidates).registered_ids,
            BTreeSet::from([
                "abnormality_alpha".to_string(),
                "corrosion_beta".to_string()
            ])
        );
    }

    #[test]
    fn replacement_schema_rejects_the_retired_four_sets() {
        assert!(serde_json::from_str::<EncounterCompendiumProfileState>(
            r#"{"registered_ids": [], "discovered_abnormality_ids": []}"#
        )
        .unwrap_err()
        .to_string()
        .contains("unknown field"));
    }

    #[test]
    fn persisted_profile_preserves_dormant_ids_and_enforces_format() {
        let dormant = EncounterCompendiumProfileState {
            registered_ids: BTreeSet::from(["dormant_abnormality".to_string()]),
        };
        dormant.validate_persisted().unwrap();

        let invalid_id = EncounterCompendiumProfileState {
            registered_ids: BTreeSet::from([" contains space ".to_string()]),
        };
        assert!(invalid_id.validate_persisted().is_err());
    }

    #[test]
    fn successor_profile_defaults_are_strict_and_merge_is_monotonic() {
        let defaults = PlayerProfileState::default();
        assert_eq!(defaults.lifecycle, Default::default());
        assert_eq!(defaults.highest_floor_reached, 0);
        assert_eq!(defaults.longest_deathless_floor_streak, 0);
        assert!(!defaults.all_live_final_bosses_defeated);
        assert!(serde_json::from_str::<PlayerProfileState>(
            r#"{
                "encounter_compendium": {
                    "registered_ids": []
                },
                "standard_tutorial_completed": false,
                "highest_floor_reached": 0,
                "longest_deathless_floor_streak": 0,
                "all_live_final_bosses_defeated": false,
                "unknown": true
            }"#,
        )
        .unwrap_err()
        .to_string()
        .contains("unknown field"));
        let current = serde_json::to_value(&defaults).unwrap();
        assert_eq!(serde_json::from_value::<PlayerProfileState>(current.clone()).unwrap(), defaults);
        let mut unexpected = current.clone();
        unexpected["unknown"] = serde_json::json!(true);
        assert!(serde_json::from_value::<PlayerProfileState>(unexpected).is_err());
        for old in [false, true] {
            let mut legacy = current.clone();
            legacy["standard_tutorial_completed"] = serde_json::json!(old);
            assert!(serde_json::from_value::<PlayerProfileState>(legacy).is_err());
        }

        let mut profile = PlayerProfileState {
            facility_knowledge: Default::default(),
            encounter_compendium: EncounterCompendiumProfileState {
                registered_ids: BTreeSet::from(["first".to_string()]),
            },
            lifecycle: Default::default(),
            highest_floor_reached: 8,
            longest_deathless_floor_streak: 5,
            all_live_final_bosses_defeated: false,
        };
        profile.merge(PlayerProfileState {
            facility_knowledge: Default::default(),
            encounter_compendium: EncounterCompendiumProfileState {
                registered_ids: BTreeSet::from(["second".to_string()]),
            },
            lifecycle: Default::default(),
            highest_floor_reached: 3,
            longest_deathless_floor_streak: 2,
            all_live_final_bosses_defeated: true,
        }).unwrap();
        assert_eq!(profile.lifecycle, Default::default());
        assert_eq!(profile.highest_floor_reached, 8);
        assert_eq!(profile.longest_deathless_floor_streak, 5);
        assert!(profile.all_live_final_bosses_defeated);
        assert_eq!(
            profile.encounter_compendium.registered_ids,
            BTreeSet::from(["first".to_string(), "second".to_string()])
        );
    }

    #[test]
    fn all_live_final_boss_record_requires_every_current_chain_in_one_run() {
        let game_data = crate::game::data::ValidatedGameData::load_live_embedded()
            .expect("live data must load");
        let live_final_bosses = game_data
            .boss_omen_data
            .chains
            .iter()
            .map(|chain| chain.id.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            live_final_bosses,
            BTreeSet::from([crate::game::data::boss_omen_data::BossOmenChainId::new(
                "white_night"
            )])
        );

        let mut profile = PlayerProfileState::default();
        let mut run = BossOmenRunState::default();
        profile.refresh_all_live_final_bosses_defeated(&game_data, &run);
        assert!(!profile.all_live_final_bosses_defeated);
        for (index, chain_id) in live_final_bosses.iter().enumerate() {
            run.completed_chain_ids.insert(chain_id.clone());
            profile.refresh_all_live_final_bosses_defeated(&game_data, &run);
            assert_eq!(
                profile.all_live_final_bosses_defeated,
                index + 1 == live_final_bosses.len()
            );
        }
    }
}
