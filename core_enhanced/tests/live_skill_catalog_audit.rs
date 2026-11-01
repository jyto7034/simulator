mod common;

use std::collections::BTreeSet;

use game_core::game::ability::DeliveryDef;
use game_core::game::data::skill_fragment_data::{
    SkillFragmentEffectDef, SkillFragmentId, SkillFragmentOrigin,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveSkillCatalogAuditManifest {
    version: u32,
    delivery: LiveSkillDeliveryAudit,
    reference_ownership: LiveSkillReferenceOwnershipAudit,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveSkillDeliveryAudit {
    spatial_skills: Vec<String>,
    intentional_instant_skills: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveSkillReferenceOwnershipAudit {
    abnormality_skills: Vec<String>,
    equipment_activation_skills: Vec<String>,
    active_skill_fragment_skills: Vec<String>,
    unreferenced_skills: Vec<String>,
}

fn unique_manifest_set(label: &str, values: &[String]) -> BTreeSet<String> {
    let set = values.iter().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        set.len(),
        values.len(),
        "duplicate id in live skill audit manifest field {label}"
    );
    set
}

fn live_skill_catalog_audit_manifest() -> LiveSkillCatalogAuditManifest {
    let manifest: LiveSkillCatalogAuditManifest = ron::de::from_str(include_str!(
        "../docs/audit/live_skill_catalog_manifest.ron"
    ))
    .expect("live skill catalog audit manifest should deserialize");
    assert_eq!(manifest.version, 2, "unexpected audit manifest version");
    manifest
}

#[test]
fn live_skill_catalog_matches_current_delivery_audit() {
    let game_data = common::load_game_data_from_ron();
    let manifest = live_skill_catalog_audit_manifest();
    let expected_spatial_skills =
        unique_manifest_set("delivery.spatial_skills", &manifest.delivery.spatial_skills);
    let expected_intentional_instant_skills = unique_manifest_set(
        "delivery.intentional_instant_skills",
        &manifest.delivery.intentional_instant_skills,
    );

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
        actual_spatial_skills, expected_spatial_skills,
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
        actual_intentional_instant_skills, expected_intentional_instant_skills,
        "live Instant-only skill set changed; re-audit whether any newly added skill should migrate to Projectile/Area"
    );
}

#[test]
fn live_skill_catalog_has_exact_reference_ownership() {
    let game_data = common::load_game_data_from_ron();
    let manifest = live_skill_catalog_audit_manifest();
    let expected_abnormality_skills = unique_manifest_set(
        "reference_ownership.abnormality_skills",
        &manifest.reference_ownership.abnormality_skills,
    );
    let expected_equipment_activation_skills = unique_manifest_set(
        "reference_ownership.equipment_activation_skills",
        &manifest.reference_ownership.equipment_activation_skills,
    );
    let expected_active_skill_fragment_skills = unique_manifest_set(
        "reference_ownership.active_skill_fragment_skills",
        &manifest.reference_ownership.active_skill_fragment_skills,
    );
    let expected_unreferenced_skills = unique_manifest_set(
        "reference_ownership.unreferenced_skills",
        &manifest.reference_ownership.unreferenced_skills,
    );
    let actual_live_skill_ids: BTreeSet<String> = game_data
        .skill_data
        .skills
        .iter()
        .map(|skill| skill.id.as_str().to_string())
        .collect();

    let actual_abnormality_skills = game_data
        .abnormality_data
        .items
        .iter()
        .filter_map(|abnormality| abnormality.skill_id.as_ref())
        .map(|skill_id| skill_id.as_str().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual_abnormality_skills, expected_abnormality_skills,
        "live abnormality skill references changed; update the reference-ownership audit"
    );

    let actual_equipment_activation_skills = game_data
        .equipment_data
        .items
        .iter()
        .flat_map(|equipment| equipment.ability_activations.iter())
        .map(|binding| binding.ability_id.as_str().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual_equipment_activation_skills, expected_equipment_activation_skills,
        "live equipment skill activations changed; update the reference-ownership audit"
    );

    let actual_active_skill_fragment_skills = game_data
        .skill_fragment_data
        .fragments
        .iter()
        .flat_map(|fragment| match &fragment.effect {
            SkillFragmentEffectDef::ActiveSkill { forms, .. } => forms
                .values()
                .map(|form| &form.skill_id)
                .collect::<Vec<_>>(),
            SkillFragmentEffectDef::BasicAttackModifier { .. } => Vec::new(),
        })
        .map(|skill_id| skill_id.as_str().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual_active_skill_fragment_skills, expected_active_skill_fragment_skills,
        "live active skill fragment references changed; update the reference-ownership audit"
    );

    let referenced_skill_ids: BTreeSet<String> = expected_abnormality_skills
        .union(&expected_equipment_activation_skills)
        .cloned()
        .collect::<BTreeSet<_>>()
        .union(&expected_active_skill_fragment_skills)
        .cloned()
        .collect();

    let actual_unreferenced_skill_ids = actual_live_skill_ids
        .difference(&referenced_skill_ids)
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual_unreferenced_skill_ids,
        expected_unreferenced_skills,
        "the live unreferenced-skill set changed; add a real owner or update the explicit audit set"
    );

    let accounted_skill_ids = referenced_skill_ids
        .union(&expected_unreferenced_skills)
        .cloned()
        .collect::<BTreeSet<_>>();

    assert_eq!(
        actual_live_skill_ids, accounted_skill_ids,
        "a live skill is not accounted for by actual reference ownership or the explicit unreferenced set; this does not assert behavior coverage"
    );
}

#[test]
fn live_content_index_lists_every_abnormality_derived_fragment() {
    let game_data = common::load_game_data_from_ron();
    let reference_abnormality_ids = game_data
        .pve_data
        .encounters
        .iter()
        .filter(|encounter| encounter.is_reference_only())
        .filter_map(|encounter| encounter.primary_abnormality_id())
        .collect::<BTreeSet<_>>();
    let actual_fragment_ids = game_data
        .skill_fragment_data
        .fragments
        .iter()
        .filter(|fragment| {
            matches!(
                &fragment.origin,
                Some(SkillFragmentOrigin::Abnormality { abnormality_id })
                    if !reference_abnormality_ids.contains(abnormality_id.as_str())
            )
        })
        .map(|fragment| fragment.id.as_str().to_string())
        .collect::<BTreeSet<_>>();

    let documented_fragment_id_list = include_str!("../docs/skills/live_content_index.md")
        .split('`')
        .filter(|token| token.starts_with("fragment_"))
        .map(str::to_string)
        .collect::<Vec<_>>();
    let documented_fragment_ids = documented_fragment_id_list
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    assert_eq!(actual_fragment_ids.len(), 30);
    assert_eq!(
        documented_fragment_ids.len(),
        documented_fragment_id_list.len(),
        "the derived live content index contains a duplicate fragment id"
    );
    assert_eq!(
        actual_fragment_ids, documented_fragment_ids,
        "the derived live content index must list every abnormality-origin fragment exactly"
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
            SkillFragmentEffectDef::ActiveSkill { forms, .. } => assert_eq!(
                forms["base"].skill_id.as_str(),
                fragment_skill_id,
                "fragment '{fragment_id}' must point at its independent imitation skill"
            ),
            SkillFragmentEffectDef::BasicAttackModifier { .. } => {
                panic!("A-rank fragment '{fragment_id}' must be an active skill")
            }
        }
    }
}
