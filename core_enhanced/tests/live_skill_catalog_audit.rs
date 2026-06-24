mod common;

use std::collections::BTreeSet;

use game_core::game::ability::DeliveryDef;
use game_core::game::data::skill_fragment_data::{SkillFragmentEffectDef, SkillFragmentId};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct LiveSkillCatalogAuditManifest {
    version: u32,
    delivery: LiveSkillDeliveryAudit,
    behavior_coverage: LiveSkillBehaviorCoverageAudit,
}

#[derive(Debug, Deserialize)]
struct LiveSkillDeliveryAudit {
    spatial_skills: BTreeSet<String>,
    intentional_instant_skills: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
struct LiveSkillBehaviorCoverageAudit {
    abnormality_skills: BTreeSet<String>,
    item_and_artifact_skills: BTreeSet<String>,
    skill_fragment_skills: BTreeSet<String>,
}

fn live_skill_catalog_audit_manifest() -> LiveSkillCatalogAuditManifest {
    let manifest: LiveSkillCatalogAuditManifest = ron::de::from_str(include_str!(
        "../docs/audit/live_skill_catalog_manifest.ron"
    ))
    .expect("live skill catalog audit manifest should deserialize");
    assert_eq!(manifest.version, 1, "unexpected audit manifest version");
    manifest
}

#[test]
fn live_skill_catalog_matches_current_delivery_audit() {
    let game_data = common::load_game_data_from_ron();
    let manifest = live_skill_catalog_audit_manifest();

    let actual_spatial_skills: BTreeSet<String> = game_data
        .skill_data
        .skills
        .iter()
        .filter(|skill| {
            skill
                .steps
                .iter()
                .any(|step| !matches!(step.delivery, DeliveryDef::Instant))
        })
        .map(|skill| skill.id.as_str().to_string())
        .collect();

    assert_eq!(
        actual_spatial_skills, manifest.delivery.spatial_skills,
        "live skill delivery composition changed; re-audit which skills should stay intentional Instant vs spatial delivery"
    );

    let actual_intentional_instant_skills: BTreeSet<String> = game_data
        .skill_data
        .skills
        .iter()
        .filter(|skill| {
            skill
                .steps
                .iter()
                .all(|step| matches!(step.delivery, DeliveryDef::Instant))
        })
        .map(|skill| skill.id.as_str().to_string())
        .collect();

    assert_eq!(
        actual_intentional_instant_skills, manifest.delivery.intentional_instant_skills,
        "live Instant-only skill set changed; re-audit whether any newly added skill should migrate to Projectile/Area"
    );
}

#[test]
fn live_skill_catalog_is_fully_covered_by_behavior_test_manifest() {
    let game_data = common::load_game_data_from_ron();
    let manifest = live_skill_catalog_audit_manifest();
    let actual_live_skill_ids: BTreeSet<String> = game_data
        .skill_data
        .skills
        .iter()
        .map(|skill| skill.id.as_str().to_string())
        .collect();

    let covered_skill_ids: BTreeSet<String> = manifest
        .behavior_coverage
        .abnormality_skills
        .union(&manifest.behavior_coverage.item_and_artifact_skills)
        .cloned()
        .collect::<BTreeSet<_>>()
        .union(&manifest.behavior_coverage.skill_fragment_skills)
        .cloned()
        .collect();

    assert_eq!(
        actual_live_skill_ids, covered_skill_ids,
        "a live skill was added without updating the scenario/item behavior-test coverage manifest"
    );
}

#[test]
fn a_rank_original_and_fragment_skills_are_independent_live_contracts() {
    let game_data = common::load_game_data_from_ron();
    let pairs = [
        (
            "t-02-43_freischutz",
            "freischutz_magic_bullet",
            "fragment_freischutz_black_round",
            "fragment_freischutz_black_round",
        ),
        (
            "t-02-99_funeral_butterfly",
            "funeral_butterfly_eulogy_volley",
            "fragment_funeral_butterfly_eulogy",
            "fragment_funeral_butterfly_eulogy",
        ),
    ];

    for (abnormality_id, original_skill_id, fragment_id, fragment_skill_id) in pairs {
        let abnormality = game_data
            .abnormality_data
            .get_by_id(abnormality_id)
            .unwrap_or_else(|| panic!("missing A-rank abnormality '{abnormality_id}'"));
        assert_eq!(
            abnormality.skill_id.as_ref().map(|id| id.as_str()),
            Some(original_skill_id),
            "A-rank abnormality '{abnormality_id}' must point at its original skill"
        );

        let original_skill = game_data
            .skill_data
            .get_by_id(original_skill_id)
            .unwrap_or_else(|| panic!("missing original skill '{original_skill_id}'"));
        let fragment_skill = game_data
            .skill_data
            .get_by_id(fragment_skill_id)
            .unwrap_or_else(|| panic!("missing fragment skill '{fragment_skill_id}'"));

        assert_ne!(
            original_skill.id.as_str(),
            fragment_skill.id.as_str(),
            "original and fragment skills must never share the same SkillDef id"
        );
        assert_ne!(
            format!("{:?}", original_skill.steps),
            format!("{:?}", fragment_skill.steps),
            "original skill '{original_skill_id}' and fragment skill '{fragment_skill_id}' must differ in their step contracts"
        );

        let fragment = game_data
            .skill_fragment_data
            .get_by_id(&SkillFragmentId::from(fragment_id))
            .unwrap_or_else(|| panic!("missing skill fragment '{fragment_id}'"));
        match &fragment.effect {
            SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id, ..
            } => assert_eq!(
                imitation_skill_id.as_str(),
                fragment_skill_id,
                "fragment '{fragment_id}' must point at its independent imitation skill"
            ),
            SkillFragmentEffectDef::BasicAttackModifier { .. } => {
                panic!("A-rank fragment '{fragment_id}' must be an active skill")
            }
        }
    }
}
