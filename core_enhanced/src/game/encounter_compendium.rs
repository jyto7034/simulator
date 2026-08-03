use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::game::{
    battle::{
        event_log::{BattleEventLog, BattleLogEvent},
        types::BattleUnitSourceIdentity,
    },
    data::GameDataBase,
    enums::Side,
};

/// Persisted game-global encounter knowledge.
///
/// Only canonical IDs are stored. Display metadata is always resolved from the
/// current strict live catalogs when a snapshot is projected. Persisted IDs
/// may be dormant (temporarily absent from the current catalog).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterCompendiumProfileState {
    pub discovered_abnormality_ids: BTreeSet<String>,
    pub defeated_abnormality_ids: BTreeSet<String>,
    pub discovered_corrosion_profile_ids: BTreeSet<String>,
    pub defeated_corrosion_profile_ids: BTreeSet<String>,
}

impl EncounterCompendiumProfileState {
    pub fn from_consumed_battle_result(
        event_log: &BattleEventLog,
        player_victory: bool,
        primary_abnormality_id: Option<&str>,
    ) -> Self {
        let mut committed = Self::default();
        let mut opponent_sources = HashMap::new();
        for entry in &event_log.entries {
            let BattleLogEvent::UnitSpawned {
                unit_instance_id,
                owner,
                unit_source,
                ..
            } = &entry.event
            else {
                continue;
            };
            if *owner != Side::Opponent {
                continue;
            }
            match unit_source {
                BattleUnitSourceIdentity::Abnormality { abnormality_id, .. } => {
                    committed
                        .discovered_abnormality_ids
                        .insert(abnormality_id.clone());
                    opponent_sources.insert(*unit_instance_id, unit_source.clone());
                }
                BattleUnitSourceIdentity::CorrodedEmployee { profile_id, .. } => {
                    committed
                        .discovered_corrosion_profile_ids
                        .insert(profile_id.clone());
                    opponent_sources.insert(*unit_instance_id, unit_source.clone());
                }
                BattleUnitSourceIdentity::Employee { .. }
                | BattleUnitSourceIdentity::FacilityEntity { .. }
                | BattleUnitSourceIdentity::DefenseObject { .. }
                | BattleUnitSourceIdentity::TestFixture { .. } => {}
            }
        }

        if !player_victory {
            return committed;
        }

        if let Some(primary_abnormality_id) = primary_abnormality_id {
            // A consumed primary victory is itself authoritative encounter
            // evidence even when an older battle fixture omitted UnitSpawned.
            committed
                .discovered_abnormality_ids
                .insert(primary_abnormality_id.to_string());
            committed
                .defeated_abnormality_ids
                .insert(primary_abnormality_id.to_string());
        }
        for entry in &event_log.entries {
            let BattleLogEvent::UnitDied {
                unit_instance_id,
                owner,
                ..
            } = &entry.event
            else {
                continue;
            };
            if *owner != Side::Opponent {
                continue;
            }
            match opponent_sources.get(unit_instance_id) {
                Some(BattleUnitSourceIdentity::Abnormality { abnormality_id, .. }) => {
                    committed
                        .defeated_abnormality_ids
                        .insert(abnormality_id.clone());
                }
                Some(BattleUnitSourceIdentity::CorrodedEmployee { profile_id, .. }) => {
                    committed
                        .defeated_corrosion_profile_ids
                        .insert(profile_id.clone());
                }
                Some(
                    BattleUnitSourceIdentity::Employee { .. }
                    | BattleUnitSourceIdentity::FacilityEntity { .. }
                    | BattleUnitSourceIdentity::DefenseObject { .. }
                    | BattleUnitSourceIdentity::TestFixture { .. },
                )
                | None => {}
            }
        }
        committed
    }

    pub fn validate_persisted(&self) -> Result<(), String> {
        for abnormality_id in self
            .discovered_abnormality_ids
            .iter()
            .chain(&self.defeated_abnormality_ids)
        {
            validate_profile_id("abnormality", abnormality_id)?;
        }
        for profile_id in self
            .discovered_corrosion_profile_ids
            .iter()
            .chain(&self.defeated_corrosion_profile_ids)
        {
            validate_profile_id("corrosion profile", profile_id)?;
        }
        if !self
            .defeated_abnormality_ids
            .is_subset(&self.discovered_abnormality_ids)
        {
            return Err(
                "encounter compendium defeated abnormality ids must be discovered".to_string(),
            );
        }
        if !self
            .defeated_corrosion_profile_ids
            .is_subset(&self.discovered_corrosion_profile_ids)
        {
            return Err(
                "encounter compendium defeated corrosion profile ids must be discovered"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub fn validate_current_catalog_commit(&self, game_data: &GameDataBase) -> Result<(), String> {
        self.validate_persisted()?;
        for abnormality_id in &self.discovered_abnormality_ids {
            if game_data
                .abnormality_data
                .get_by_id(abnormality_id)
                .is_none()
            {
                return Err(format!(
                    "encounter compendium references missing abnormality '{abnormality_id}'"
                ));
            }
        }
        for profile_id in &self.discovered_corrosion_profile_ids {
            if game_data
                .corroded_employee_data
                .get_by_id(profile_id)
                .is_none()
            {
                return Err(format!(
                    "encounter compendium references missing corrosion profile '{profile_id}'"
                ));
            }
        }
        Ok(())
    }

    pub fn merge(&mut self, committed: Self) {
        self.discovered_abnormality_ids
            .extend(committed.discovered_abnormality_ids);
        self.defeated_abnormality_ids
            .extend(committed.defeated_abnormality_ids);
        self.discovered_corrosion_profile_ids
            .extend(committed.discovered_corrosion_profile_ids);
        self.defeated_corrosion_profile_ids
            .extend(committed.defeated_corrosion_profile_ids);
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
    use crate::game::{
        battle::{
            core::movement::types::EventLogVec2,
            event_log::{BattleEventCause, BattleEventLogEntry},
            ids::UnitInstanceId,
            types::{BattleUnitRole, BattleUnitThreatClass, MobilityKind},
        },
        stats::UnitStats,
    };

    fn spawned(
        seq: u64,
        owner: Side,
        unit_source: BattleUnitSourceIdentity,
    ) -> BattleEventLogEntry {
        let unit_uuid = Uuid::from_u128(u128::from(seq));
        BattleEventLogEntry {
            time_ms: seq,
            seq,
            cause: BattleEventCause::default(),
            source_command_id: None,
            event: BattleLogEvent::UnitSpawned {
                unit_instance_id: UnitInstanceId::from(unit_uuid),
                owner,
                role: BattleUnitRole::Combatant,
                threat_class: BattleUnitThreatClass::Normal,
                mobility_kind: MobilityKind::Ground,
                base_uuid: unit_source.base_uuid(),
                unit_source,
                world_position: EventLogVec2::default(),
                stats: UnitStats::default(),
            },
        }
    }

    #[test]
    fn consumed_victory_collects_discovery_and_actual_deaths() {
        let abnormality_uuid = Uuid::from_u128(0xA1);
        let corrosion_uuid = Uuid::from_u128(0xC1);
        let ignored_uuid = Uuid::from_u128(0xF1);
        let mut event_log = BattleEventLog::new();
        event_log.entries = vec![
            spawned(
                1,
                Side::Opponent,
                BattleUnitSourceIdentity::Abnormality {
                    abnormality_id: "z_abnormality".to_string(),
                    base_uuid: abnormality_uuid,
                },
            ),
            spawned(
                2,
                Side::Opponent,
                BattleUnitSourceIdentity::CorrodedEmployee {
                    profile_id: "corroded_guard".to_string(),
                    base_uuid: corrosion_uuid,
                },
            ),
            spawned(
                3,
                Side::Player,
                BattleUnitSourceIdentity::Abnormality {
                    abnormality_id: "player_side_ignored".to_string(),
                    base_uuid: ignored_uuid,
                },
            ),
            spawned(
                4,
                Side::Opponent,
                BattleUnitSourceIdentity::FacilityEntity {
                    profile_id: "facility_ignored".to_string(),
                    base_uuid: ignored_uuid,
                },
            ),
            spawned(
                5,
                Side::Opponent,
                BattleUnitSourceIdentity::Abnormality {
                    abnormality_id: "z_abnormality".to_string(),
                    base_uuid: abnormality_uuid,
                },
            ),
        ];

        event_log.entries.push(BattleEventLogEntry {
            time_ms: 6,
            seq: 6,
            cause: BattleEventCause::default(),
            source_command_id: None,
            event: BattleLogEvent::UnitDied {
                unit_instance_id: UnitInstanceId::from(Uuid::from_u128(2)),
                owner: Side::Opponent,
                killer_instance_id: None,
                world_position: EventLogVec2::default(),
                position: crate::game::resources::board::Position { x: 0, y: 0 },
            },
        });

        let committed = EncounterCompendiumProfileState::from_consumed_battle_result(
            &event_log,
            true,
            Some("z_abnormality"),
        );

        assert_eq!(
            committed
                .discovered_abnormality_ids
                .into_iter()
                .collect::<Vec<_>>(),
            vec!["z_abnormality"]
        );
        assert_eq!(
            committed
                .discovered_corrosion_profile_ids
                .into_iter()
                .collect::<Vec<_>>(),
            vec!["corroded_guard"]
        );
        assert_eq!(
            committed.defeated_abnormality_ids,
            BTreeSet::from(["z_abnormality".to_string()])
        );
        assert_eq!(
            committed.defeated_corrosion_profile_ids,
            BTreeSet::from(["corroded_guard".to_string()])
        );
    }

    #[test]
    fn victory_does_not_mark_a_surviving_non_primary_encounter_defeated() {
        let mut event_log = BattleEventLog::new();
        event_log.entries = vec![
            spawned(
                1,
                Side::Opponent,
                BattleUnitSourceIdentity::Abnormality {
                    abnormality_id: "primary_abnormality".to_string(),
                    base_uuid: Uuid::from_u128(0xA1),
                },
            ),
            spawned(
                2,
                Side::Opponent,
                BattleUnitSourceIdentity::CorrodedEmployee {
                    profile_id: "surviving_corrosion".to_string(),
                    base_uuid: Uuid::from_u128(0xC1),
                },
            ),
        ];

        let committed = EncounterCompendiumProfileState::from_consumed_battle_result(
            &event_log,
            true,
            Some("primary_abnormality"),
        );

        assert_eq!(
            committed.defeated_abnormality_ids,
            BTreeSet::from(["primary_abnormality".to_string()])
        );
        assert!(committed.defeated_corrosion_profile_ids.is_empty());
        assert!(committed
            .discovered_corrosion_profile_ids
            .contains("surviving_corrosion"));
    }

    #[test]
    fn profile_schema_rejects_unknown_fields() {
        let error = serde_json::from_str::<EncounterCompendiumProfileState>(
            r#"{
                "discovered_abnormality_ids": [],
                "defeated_abnormality_ids": [],
                "discovered_corrosion_profile_ids": [],
                "defeated_corrosion_profile_ids": [],
                "battle_records": []
            }"#,
        )
        .unwrap_err();
        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn persisted_profile_preserves_dormant_ids_but_enforces_subset_and_format() {
        let dormant = EncounterCompendiumProfileState {
            discovered_abnormality_ids: BTreeSet::from(["dormant_abnormality".to_string()]),
            defeated_abnormality_ids: BTreeSet::from(["dormant_abnormality".to_string()]),
            discovered_corrosion_profile_ids: BTreeSet::from(["dormant_corrosion".to_string()]),
            defeated_corrosion_profile_ids: BTreeSet::new(),
        };
        dormant.validate_persisted().unwrap();

        let invalid_subset = EncounterCompendiumProfileState {
            defeated_abnormality_ids: BTreeSet::from(["not_discovered".to_string()]),
            ..Default::default()
        };
        assert!(invalid_subset.validate_persisted().is_err());

        let invalid_id = EncounterCompendiumProfileState {
            discovered_abnormality_ids: BTreeSet::from([" contains space ".to_string()]),
            ..Default::default()
        };
        assert!(invalid_id.validate_persisted().is_err());
    }
}
