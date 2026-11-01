//! Field-local compatibility: keep existing fragment IDs as plain strings while
//! allowing an explicit absence. Omitting the field is still an authoring error.
use crate::game::data::skill_fragment_data::SkillFragmentId;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

pub(super) fn serialize<S: Serializer>(
    value: &Option<SkillFragmentId>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match value {
        Some(id) => id.serialize(serializer),
        None => serializer.serialize_none(),
    }
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<SkillFragmentId>, D::Error> {
    struct Fragment;
    impl<'de> Visitor<'de> for Fragment {
        type Value = Option<SkillFragmentId>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a fragment ID string or explicit None/null")
        }
        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            Ok(Some(value.into()))
        }
        fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
            Ok(Some(value.into()))
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            // JSON's explicit null is represented by visit_unit.
            Ok(None)
        }
        fn visit_some<D: Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, D::Error> {
            SkillFragmentId::deserialize(deserializer).map(Some)
        }
    }
    deserializer.deserialize_any(Fragment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::pve_data::PveEncounterRewardSource;

    fn source(fragment: &str) -> String {
        format!(
            "(reward_source_id:\"source\",encounter_id:\"encounter\",fragment_id:{fragment},first_equipment_ids:[],repeat_rewards:[],revival:None)"
        )
    }

    #[test]
    fn optional_fragment_legacy_and_explicit_none_roundtrip_without_hidden_default() {
        for (input, expected) in [
            ("\"fragment\"", Some(SkillFragmentId::from("fragment"))),
            (
                "Some(\"fragment\")",
                Some(SkillFragmentId::from("fragment")),
            ),
            ("None", None),
        ] {
            let row: PveEncounterRewardSource = ron::de::from_str(&source(input)).unwrap();
            assert_eq!(row.fragment_id, expected);
            let encoded = ron::ser::to_string(&row).unwrap();
            if expected.is_some() {
                assert!(encoded.contains("fragment_id:\"fragment\""));
                assert!(!encoded.contains("Some(\"fragment\")"));
            } else {
                assert!(encoded.contains("fragment_id:None"));
            }
            let restored: PveEncounterRewardSource = ron::de::from_str(&encoded).unwrap();
            assert_eq!(restored.fragment_id, expected);
            let json = serde_json::to_value(&row).unwrap();
            assert_eq!(
                json["fragment_id"],
                if expected.is_some() {
                    serde_json::json!("fragment")
                } else {
                    serde_json::Value::Null
                }
            );
            let restored: PveEncounterRewardSource = serde_json::from_value(json).unwrap();
            assert_eq!(restored.fragment_id, expected);
        }
        let missing = source("None").replace("fragment_id:None,", "");
        let error = ron::de::from_str::<PveEncounterRewardSource>(&missing).unwrap_err();
        assert!(error.to_string().contains("fragment_id"), "{error}");
        for invalid in ["12", "true", "[\"fragment\"]", "Some(12)"] {
            assert!(
                ron::de::from_str::<PveEncounterRewardSource>(&source(invalid)).is_err(),
                "{invalid}"
            );
        }
        let mut json = serde_json::to_value(
            ron::de::from_str::<PveEncounterRewardSource>(&source("None")).unwrap(),
        )
        .unwrap();
        json.as_object_mut().unwrap().remove("fragment_id");
        assert!(serde_json::from_value::<PveEncounterRewardSource>(json).is_err());
    }
}

#[cfg(test)]
mod catalog_tests {
    use crate::game::data::live_ron::{parse_gameplay_ron_sources, GameplayRonSourceSet};

    #[test]
    fn optional_fragment_cross_catalog_requires_agreement_and_resolves_present_ids() {
        let baseline = || parse_gameplay_ron_sources(GameplayRonSourceSet::embedded()).unwrap();
        let parsed = baseline();
        parsed.validate_domain_catalogs().unwrap();
        let wolf = parsed
            .pve_encounters
            .reward_sources
            .iter()
            .position(|s| s.reward_source_id == "normal_boss_big_bad_wolf")
            .unwrap();
        let abnormality = parsed
            .abnormalities
            .items
            .iter()
            .position(|a| a.id == "f-02-58_big_bad_wolf")
            .unwrap();
        assert!(parsed.pve_encounters.reward_sources[wolf]
            .fragment_id
            .is_none());
        assert!(parsed.abnormalities.items[abnormality]
            .response_complete_skill_fragment_id
            .is_none());
        for case in 0..3 {
            let mut changed = baseline();
            if case != 1 {
                changed.pve_encounters.reward_sources[wolf].fragment_id = Some(
                    if case == 2 {
                        "absent-fragment"
                    } else {
                        "fragment_big_bad_wolf_trace"
                    }
                    .into(),
                );
            }
            if case != 0 {
                changed.abnormalities.items[abnormality].response_complete_skill_fragment_id = Some(
                    if case == 2 {
                        "absent-fragment"
                    } else {
                        "fragment_big_bad_wolf_trace"
                    }
                    .into(),
                );
            }
            let error = changed.validate_domain_catalogs().unwrap_err();
            assert!(
                error
                    .field_path
                    .as_deref()
                    .is_some_and(|p| p.contains("fragment")),
                "{case}: {error}"
            );
        }
        let mut duplicate = baseline();
        let existing = duplicate
            .pve_encounters
            .reward_sources
            .iter()
            .find_map(|s| s.fragment_id.clone())
            .unwrap();
        duplicate.pve_encounters.reward_sources[wolf].fragment_id = Some(existing);
        let error = duplicate.pve_encounters.validate_contract().unwrap_err();
        assert!(
            error.cause.contains("exactly one reward source"),
            "{error:?}"
        );
        let mut several = baseline();
        let second = several
            .pve_encounters
            .reward_sources
            .iter()
            .position(|s| s.reward_source_id == "elite_censored")
            .unwrap();
        several.pve_encounters.reward_sources[second].fragment_id = None;
        several
            .abnormalities
            .items
            .iter_mut()
            .find(|a| a.id == "o-03-89_censored")
            .unwrap()
            .response_complete_skill_fragment_id = None;
        several.validate_domain_catalogs().unwrap();
        // Absence is not a shared fragment identity; two explicit no-fragment sources coexist.
        assert!(several.pve_encounters.reward_sources[wolf]
            .fragment_id
            .is_none());
        assert!(several.pve_encounters.reward_sources[second]
            .fragment_id
            .is_none());
    }
}
