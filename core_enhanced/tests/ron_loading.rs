mod common;

use std::collections::BTreeSet;

use game_core::game::ability::{
    DeliveryDef, SkillAreaAnchorSource, SkillCastTargetingDef, SkillTarget, UnitTargetRule,
};
use game_core::game::battle::buffs::{BuffId, BuffKind};
use game_core::game::combat_preview::{
    required_briefing_warning_tags_for_spawn_waves, BattlefieldArchetype, CombatNodeType,
    CombatPreview, EnemyKind, ThreatWarningSource,
};
use game_core::game::data::{
    abnormality_data::AbnormalityDatabase,
    consumable_data::{ConsumableEffect, ConsumableTier},
    equipment_data::{EquipmentType, WeaponArchetype},
    event_data::{EventSceneNext, EventSuccessAction},
    pve_data::{PveEncounterClass, PveWaveEnemyData, PveWaveSource},
    reward_data::RewardGrantKind,
    skill_fragment_data::{SkillFragmentEffectDef, SkillFragmentOrigin},
};
use game_core::game::employee::EventAptitude;
use game_core::game::enums::RewardMode;
use game_core::game::map::{MapNodeCategory, MapNodeId, MapNodePayload};
use game_core::game::reward::{ExperienceTargetPolicy, RewardEffect};
use uuid::Uuid;

#[test]
fn live_abnormality_ron_only_contains_catalog_roster() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../game_resources/data/abnormalities/base.ron");
    let contents = std::fs::read_to_string(&path).expect("abnormalities/base.ron should load");
    let database: AbnormalityDatabase =
        ron::de::from_str(&contents).expect("abnormalities/base.ron should deserialize");

    let expected = BTreeSet::from([
        "d-01-110_clouded_monk",
        "d-03-109_melting_love",
        "d-04-108_parasite_tree",
        "f-01-18_scarecrow",
        "f-01-57_little_red",
        "f-01-87_snow_queen",
        "f-02-49_rudolta",
        "f-02-58_big_bad_wolf",
        "f-02-70_black_swan",
        "f-05-32_warm_hearted_woodsman",
        "o-01-04_queen_of_hatred",
        "o-01-15_nameless_fetus",
        "o-01-45_white_night",
        "o-01-64_king_of_greed",
        "o-01-64_yin",
        "o-01-73_knight_of_despair",
        "o-02-63_apocalypse_bird",
        "o-03-89_censored",
        "o-04-66_porcubbus",
        "o-04-72_burrowing_heaven",
        "o-05-76_schadenfreude",
        "o-05-102_yang",
        "o-06-20_nothing_there",
        "t-01-31_silent_orchestra",
        "t-01-75_mountain",
        "t-02-43_freischutz",
        "t-02-99_funeral_butterfly",
        "t-04-50_queen_bee",
        "t-06-27_moonlit_wail",
        "o-01-67_laetitia",
    ]);
    let actual: BTreeSet<&str> = database.items.iter().map(|abno| abno.id.as_str()).collect();

    assert_eq!(
        actual, expected,
        "live abnormality metadata must contain only the catalog roster"
    );
}

#[test]
fn live_weapon_block_capacity_is_explicit_and_matches_approved_guideline() {
    let game_data = common::load_game_data_from_ron();
    let mut represented_archetypes = BTreeSet::new();

    for item in &game_data.equipment_data.items {
        if item.equipment_type != EquipmentType::Weapon {
            continue;
        }
        let profile = item
            .weapon_profile
            .as_ref()
            .expect("validated live weapon must have a profile");
        let expected = match profile.weapon_archetype {
            WeaponArchetype::Shield => 3,
            WeaponArchetype::Sword | WeaponArchetype::Spear => 1,
            WeaponArchetype::Bow | WeaponArchetype::Gun | WeaponArchetype::Staff => 0,
        };
        assert_eq!(
            profile.block_capacity, expected,
            "live weapon '{}' must explicitly use its approved authoring guideline",
            item.id
        );
        represented_archetypes.insert(format!("{:?}", profile.weapon_archetype));
    }

    assert_eq!(
        represented_archetypes,
        BTreeSet::from([
            "Bow".to_string(),
            "Gun".to_string(),
            "Shield".to_string(),
            "Spear".to_string(),
            "Staff".to_string(),
            "Sword".to_string(),
        ])
    );
}

#[test]
fn load_game_data_from_ron_reads_step_based_skill_schema() {
    let game_data = common::load_game_data_from_ron();

    let white_night = game_data
        .skill_data
        .get_by_id("white_night_pale_benediction")
        .expect("WhiteNight skill should exist in RON data");
    assert_eq!(white_night.steps.len(), 3);
    assert_eq!(white_night.steps[0].id, "ally_salvation");
    assert_eq!(white_night.steps[1].id, "ally_blessing");
    assert_eq!(white_night.steps[2].id, "enemy_judgement");

    let mountain = game_data
        .abnormality_data
        .get_by_id("t-01-75_mountain")
        .expect("Mountain of Smiling Bodies abnormality should exist in RON data");
    assert_eq!(
        mountain.skill_id.as_deref(),
        Some("mountain_mass_consumption")
    );

    let queen = game_data
        .skill_data
        .get_by_id("queen_of_hatred_magical_beam")
        .expect("Queen skill should exist in RON data");
    assert!(matches!(
        &queen.steps[0].delivery,
        DeliveryDef::TileArea { footprint, .. } if !footprint.rows.is_empty()
    ));
    assert!(matches!(
        queen.steps[0].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                ..
            },
            ..
        }
    ));

    let melting_love = game_data
        .skill_data
        .get_by_id("melting_love_slime_infection")
        .expect("Melting Love skill should exist in RON data");
    assert!(matches!(
        &melting_love.steps[1].delivery,
        DeliveryDef::TileArea { footprint, .. } if !footprint.rows.is_empty()
    ));
    assert!(matches!(
        melting_love.steps[1].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::ImpactContext,
                ..
            },
            ..
        }
    ));

    let white_night = game_data
        .skill_data
        .get_by_id("white_night_pale_benediction")
        .expect("WhiteNight skill should exist in RON data");
    assert!(matches!(
        &white_night.steps[0].delivery,
        DeliveryDef::TileArea { footprint, .. } if footprint.include_anchor_tile
    ));
    assert!(matches!(
        white_night.steps[0].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                include_caster: true,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        &white_night.steps[2].delivery,
        DeliveryDef::TileArea { footprint, .. }
            if footprint.include_anchor_tile && footprint.is_rotation_invariant()
    ));
    assert!(matches!(
        white_night.steps[2].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                include_caster: false,
                ..
            },
            ..
        }
    ));

    assert_eq!(
        white_night.steps[0].presentation.impact_vfx_id.as_deref(),
        Some("white_night_pale_benediction_salvation")
    );
    assert_eq!(
        white_night.steps[1].presentation.cast_state.as_deref(),
        Some("Cast")
    );
    assert_eq!(
        white_night.steps[2].presentation.target_anchor.as_deref(),
        Some("Head")
    );

    assert!(
        game_data.skill_data.get_by_id("plague_mass_heal").is_none(),
        "Plague Doctor is WhiteNight's prelude/form, so its standalone skill should not remain in live RON"
    );

    let mountain_skill = game_data
        .skill_data
        .get_by_id("mountain_mass_consumption")
        .expect("Mountain skill should exist in RON data");
    assert!(matches!(
        &mountain_skill.steps[0].delivery,
        DeliveryDef::TileArea { footprint, .. } if !footprint.rows.is_empty()
    ));
    assert!(matches!(
        mountain_skill.steps[0].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                ..
            },
            ..
        }
    ));
}

#[test]
fn live_skills_pin_explicit_cast_selection_independent_from_weapon_profiles() {
    let game_data = common::load_game_data_from_ron();
    let actual = game_data
        .skill_data
        .skills
        .iter()
        .map(|skill| {
            let SkillCastTargetingDef::Explicit { target, .. } = &skill.cast_targeting;
            let selection = match target {
                SkillTarget::SelfUnit => "self",
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                } => "nearest",
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::CurrentTarget,
                } => "current_target",
                SkillTarget::EnemySingle {
                    rule: UnitTargetRule::LowestHealthEnemy,
                } => "lowest_health_enemy",
                SkillTarget::CastTarget => "cast_target",
            };
            (skill.id.to_string(), selection.to_string())
        })
        .collect::<BTreeSet<_>>();
    let expected = BTreeSet::from([
        (
            "unknown_distortion_strike".to_string(),
            "nearest".to_string(),
        ),
        (
            "queen_of_hatred_magical_beam".to_string(),
            "nearest".to_string(),
        ),
        (
            "little_red_hunt_the_prey".to_string(),
            "current_target".to_string(),
        ),
        (
            "mountain_mass_consumption".to_string(),
            "nearest".to_string(),
        ),
        (
            "melting_love_slime_infection".to_string(),
            "lowest_health_enemy".to_string(),
        ),
        (
            "nothing_there_goodbye".to_string(),
            "current_target".to_string(),
        ),
        (
            "white_night_pale_benediction".to_string(),
            "self".to_string(),
        ),
        (
            "freischutz_magic_bullet".to_string(),
            "lowest_health_enemy".to_string(),
        ),
        (
            "funeral_butterfly_eulogy_volley".to_string(),
            "current_target".to_string(),
        ),
        ("laetitia_surprise_gift".to_string(), "nearest".to_string()),
        (
            "snow_queen_crystal_prison".to_string(),
            "current_target".to_string(),
        ),
        ("rudolta_sleigh_charge".to_string(), "nearest".to_string()),
        ("silent_orchestra_finale".to_string(), "nearest".to_string()),
        (
            "censored_mind_break".to_string(),
            "current_target".to_string(),
        ),
        ("yin_black_pulse".to_string(), "nearest".to_string()),
        ("yang_white_resonance".to_string(), "self".to_string()),
        (
            "burrowing_heaven_gaze_punishment".to_string(),
            "current_target".to_string(),
        ),
        (
            "parasite_tree_root_blessing".to_string(),
            "self".to_string(),
        ),
        ("porcubbus_spore_bloom".to_string(), "nearest".to_string()),
        (
            "scarecrow_wisdom_reap".to_string(),
            "lowest_health_enemy".to_string(),
        ),
        (
            "woodsman_heart_rip".to_string(),
            "current_target".to_string(),
        ),
        (
            "paradise_lost_judgement".to_string(),
            "current_target".to_string(),
        ),
        (
            "resonance_pendant_opening_focus".to_string(),
            "self".to_string(),
        ),
        (
            "fragment_freischutz_black_round".to_string(),
            "lowest_health_enemy".to_string(),
        ),
        (
            "fragment_funeral_butterfly_eulogy".to_string(),
            "lowest_health_enemy".to_string(),
        ),
    ]);

    assert_eq!(actual, expected);
}

#[test]
fn load_game_data_from_ron_reads_consumable_schema() {
    let game_data = common::load_game_data_from_ron();

    let ampoule = game_data
        .consumable_data
        .get_by_id("stabilizing_ampoule")
        .expect("stabilizing_ampoule consumable should exist in RON data");
    assert_eq!(ampoule.tier, ConsumableTier::Common);
    assert!(ampoule.live_pool);
    assert!(matches!(
        ampoule.effect,
        ConsumableEffect::TraumaMitigation { percent: 20 }
    ));

    let deployment_anchor = game_data
        .consumable_data
        .get_by_id("deployment_anchor_patch")
        .expect("deployment_anchor_patch consumable should exist in RON data");
    assert!(deployment_anchor.live_pool);
    assert!(matches!(
        deployment_anchor.effect,
        ConsumableEffect::DeployCostReduction { percent: 25 }
    ));

    let resonance_primer = game_data
        .consumable_data
        .get_by_id("resonance_primer")
        .expect("resonance_primer consumable should exist in RON data");
    assert!(resonance_primer.live_pool);
    assert!(matches!(
        resonance_primer.effect,
        ConsumableEffect::InitialSkillCharge { percent: 40 }
    ));

    let forbidden = game_data
        .consumable_data
        .get_by_id("collapse_accelerant")
        .expect("forbidden prototype should exist for schema validation");
    assert_eq!(forbidden.tier, ConsumableTier::Forbidden);
    assert!(!forbidden.live_pool);
}

#[test]
fn load_game_data_from_ron_reads_buff_schema() {
    let game_data = common::load_game_data_from_ron();

    let poison = game_data
        .buff_data
        .get(BuffId::from_name("poison"))
        .expect("poison buff should exist in RON data");
    assert_eq!(poison.max_stacks, 10);
    assert_eq!(poison.tick_interval_ms, 1000);
    assert!(matches!(poison.kind, BuffKind::PeriodicDamage { .. }));

    let stun = game_data
        .buff_data
        .get(BuffId::from_name("stun"))
        .expect("stun buff should exist in RON data");
    assert_eq!(stun.max_stacks, 1);
    assert!(matches!(stun.kind, BuffKind::Stun));
}

#[test]
fn load_game_data_from_ron_reads_skill_fragment_schema() {
    let game_data = common::load_game_data_from_ron();

    let fragment = game_data
        .skill_fragment_data
        .get_by_id_str("fragment_freischutz_black_round")
        .expect("Der Freischutz skill fragment should exist in RON data");

    assert!(matches!(
        &fragment.origin,
        Some(SkillFragmentOrigin::Abnormality { abnormality_id })
            if abnormality_id == "t-02-43_freischutz"
    ));
    assert!(matches!(
        &fragment.effect,
        SkillFragmentEffectDef::ActiveSkill {
            imitation_skill_id, ..
        }
            if imitation_skill_id.as_str() == "fragment_freischutz_black_round"
    ));
    assert!(
        game_data
            .skill_data
            .get_by_id("fragment_freischutz_black_round")
            .is_some(),
        "fragment imitation skill must be an independent SkillDef"
    );

    let original_skill = game_data
        .abnormality_data
        .get_by_id("t-02-43_freischutz")
        .and_then(|abnormality| abnormality.skill_id.as_ref())
        .expect("origin abnormality should keep its original skill");
    assert_eq!(original_skill.as_str(), "freischutz_magic_bullet");
}

#[test]
fn load_game_data_from_ron_reads_unified_employee_definitions() {
    let game_data = common::load_game_data_from_ron();

    let definitions = &game_data.employee_definition_data.definitions;
    assert_eq!(definitions.len(), 12);
    assert!(game_data
        .employee_definition_data
        .get_by_id("field_medic")
        .is_some());
    assert_eq!(
        game_data
            .employee_definition_data
            .starter_definitions()
            .len(),
        6
    );
    assert!(definitions
        .iter()
        .all(|definition| !definition.profile.background.trim().is_empty()));

    let expected_aptitudes = [
        ("field_medic", EventAptitude::Medical),
        ("hq_field_medic", EventAptitude::Medical),
        ("breach_guard", EventAptitude::Security),
        ("hq_security_guard", EventAptitude::Security),
        ("records_clerk", EventAptitude::Analysis),
        ("hq_records_observer", EventAptitude::Analysis),
        ("scout_runner", EventAptitude::Reconnaissance),
        ("hq_recovery_runner", EventAptitude::Reconnaissance),
        ("containment_trainee", EventAptitude::Containment),
        ("hq_containment_aide", EventAptitude::Containment),
        ("maintenance_tech", EventAptitude::Maintenance),
        ("hq_maintenance_aide", EventAptitude::Maintenance),
    ];
    for (definition_id, expected) in expected_aptitudes {
        assert_eq!(
            game_data
                .employee_definition_data
                .get_by_id(definition_id)
                .expect("approved employee definition should exist")
                .profile
                .event_aptitude,
            expected,
            "unexpected EventAptitude for {definition_id}"
        );
    }
}

#[test]
fn live_archive_scanner_authors_event_modifier_key_and_owner_path() {
    let game_data = common::load_game_data_from_ron();
    let scanner = game_data
        .equipment_data
        .get_by_id("archive_scanner")
        .expect("archive_scanner should exist");

    assert_eq!(scanner.event_check_modifiers.len(), 1);
    assert_eq!(
        scanner.event_check_modifiers[0].aptitude,
        EventAptitude::Analysis
    );
    assert_eq!(scanner.event_check_modifiers[0].amount, 2);
    assert_eq!(scanner.event_choice_keys.len(), 1);
    assert_eq!(
        scanner.event_choice_keys[0].as_str(),
        "sealed_archive_access"
    );

    let maintenance = game_data
        .employee_definition_data
        .get_by_id("maintenance_tech")
        .expect("maintenance_tech should exist");
    assert!(maintenance
        .initial_loadout
        .equipment_ids
        .iter()
        .any(|equipment_id| equipment_id == "archive_scanner"));
}

#[test]
fn load_game_data_from_ron_reads_run_policy() {
    let game_data = common::load_game_data_from_ron();

    assert_eq!(game_data.run_policy.setup.standard_floor_count, 3);
    assert_eq!(game_data.run_policy.setup.starter_employee_count, 3);
    assert_eq!(game_data.run_policy.setup.starter_enkephalin, 500);
    assert_eq!(game_data.run_policy.live_deployment.initial_cost, 20);
    assert_eq!(game_data.run_policy.live_deployment.base_deploy_cost, 10);
    assert_eq!(game_data.run_policy.growth.post_battle_survival_xp, 10);
}

#[test]
fn live_experience_rewards_declare_target_policy() {
    let game_data = common::load_game_data_from_ron();
    let reward = game_data
        .reward_data
        .get_by_id("experience_reward")
        .expect("experience_reward should exist in live reward data");

    assert!(reward.effects.iter().any(|effect| matches!(
        effect,
        RewardEffect::GrantExperience {
            amount: 200,
            target: ExperienceTargetPolicy::AliveRoster,
        }
    )));
}

#[test]
fn live_pve_rewards_use_fixed_current_grant_set() {
    let game_data = common::load_game_data_from_ron();
    let mut observed_effects = BTreeSet::new();

    for encounter in &game_data.pve_data.encounters {
        assert_eq!(
            encounter.reward_mode,
            RewardMode::ClaimAll,
            "live combat rewards must be automatically granted"
        );
        assert!(
            !encounter.reward_uuids.is_empty(),
            "live encounter '{}' must author fixed rewards",
            encounter.id
        );
        for reward_uuid in &encounter.reward_uuids {
            let reward = game_data
                .reward_data
                .get_by_uuid(reward_uuid)
                .expect("validated live encounter reward reference");
            for effect in &reward.effects {
                let kind = match effect {
                    RewardEffect::GrantEnkephalin { .. } => "enkephalin",
                    RewardEffect::GrantEquipmentMaterial { .. } => "equipment_material",
                    RewardEffect::GrantSkillFragment { .. } => "skill_fragment",
                    RewardEffect::GrantSkillFragmentResearch { .. } => "research_progress",
                    other => panic!(
                        "live PVE reward '{}' uses unsupported current effect: {:?}",
                        reward.id, other
                    ),
                };
                observed_effects.insert(kind);
            }
        }
    }

    assert_eq!(
        observed_effects,
        BTreeSet::from([
            "enkephalin",
            "equipment_material",
            "research_progress",
            "skill_fragment",
        ])
    );
}

#[test]
fn load_game_data_from_ron_filters_recruitment_definitions() {
    let game_data = common::load_game_data_from_ron();

    let definitions = game_data.employee_definition_data.recruitment_definitions();
    assert!(!definitions.is_empty());
    assert!(game_data
        .employee_definition_data
        .get_by_id("hq_field_medic")
        .is_some());
    assert!(
        !game_data
            .employee_definition_data
            .get_by_id("hq_field_medic")
            .unwrap()
            .starter_eligible
    );
}

#[test]
fn live_rewards_can_grant_skill_fragments_from_ron() {
    let game_data = common::load_game_data_from_ron();

    for (reward_id, fragment_id) in [
        (
            "freischutz_black_round_fragment_reward",
            "fragment_freischutz_black_round",
        ),
        (
            "funeral_butterfly_eulogy_fragment_reward",
            "fragment_funeral_butterfly_eulogy",
        ),
    ] {
        let reward = game_data
            .reward_data
            .get_by_id(reward_id)
            .expect("skill fragment reward should exist in RON data");

        assert!(reward.effects.iter().any(|effect| matches!(
            effect,
            RewardEffect::GrantSkillFragment { fragment_id: granted_fragment_id }
                if granted_fragment_id.as_str() == fragment_id
        )));
        assert!(
            reward
                .grant_kinds()
                .contains(&RewardGrantKind::SkillFragment),
            "reward `{reward_id}` should resolve as a skill fragment reward"
        );
        assert!(
            game_data
                .skill_fragment_data
                .get_by_id_str(fragment_id)
                .is_some(),
            "reward `{reward_id}` grants missing skill fragment `{fragment_id}`"
        );
    }

    let research_reward = game_data
        .reward_data
        .get_by_id("early_abnormality_research_reward")
        .expect("skill fragment research reward should exist in RON data");
    assert!(
        research_reward
            .grant_kinds()
            .contains(&RewardGrantKind::ResearchProgress),
        "skill fragment research reward should resolve ResearchProgress"
    );
    assert!(research_reward.effects.iter().any(|effect| matches!(
        effect,
        RewardEffect::GrantSkillFragmentResearch {
            fragment_id,
            amount: 1,
        } if fragment_id.as_str() == "fragment_freischutz_black_round"
    )));

    let equipment_material_reward = game_data
        .reward_data
        .get_by_id("equipment_dust_reward")
        .expect("equipment material reward should exist in RON data");
    assert!(
        equipment_material_reward
            .grant_kinds()
            .contains(&RewardGrantKind::Equipment),
        "equipment material reward should resolve Equipment"
    );
    assert!(equipment_material_reward
        .effects
        .iter()
        .any(|effect| matches!(
            effect,
            RewardEffect::GrantEquipmentMaterial {
                material_id,
                amount: 3,
            } if material_id == "equipment_dust"
        )));
    assert!(
        game_data
            .equipment_data
            .get_material_by_id("equipment_dust")
            .is_some(),
        "equipment material reward grants missing material"
    );

    let field_salvage_reward = game_data
        .reward_data
        .get_by_id("field_equipment_salvage_reward")
        .expect("field salvage reward should exist in RON data");
    assert!(field_salvage_reward
        .grant_kinds()
        .contains(&RewardGrantKind::Equipment));
    assert!(field_salvage_reward.effects.iter().any(|effect| matches!(
        effect,
        RewardEffect::GrantEquipmentMaterial {
            material_id,
            amount: 4,
        } if material_id == "equipment_dust"
    )));

    let high_risk_research_reward = game_data
        .reward_data
        .get_by_id("high_risk_abnormality_research_reward")
        .expect("high-risk research reward should exist in RON data");
    assert!(high_risk_research_reward
        .grant_kinds()
        .contains(&RewardGrantKind::ResearchProgress));
    assert!(high_risk_research_reward
        .effects
        .iter()
        .any(|effect| matches!(
            effect,
            RewardEffect::GrantSkillFragmentResearch {
                fragment_id,
                amount: 2,
            } if fragment_id.as_str() == "fragment_melting_love_trace"
        )));

    let dismantle_recipe = game_data
        .equipment_data
        .get_dismantle_recipe_by_equipment_id("standard_armor")
        .expect("live equipment dismantle recipe should exist");
    assert!(dismantle_recipe
        .yields
        .iter()
        .any(|material| { material.material_id == "equipment_dust" && material.amount == 1 }));
    let enhancement_recipe = game_data
        .equipment_data
        .get_enhancement_recipe_by_equipment_id("standard_armor")
        .expect("live equipment enhancement recipe should exist");
    assert_eq!(enhancement_recipe.max_level, 5);
    assert!(enhancement_recipe
        .costs_per_level
        .iter()
        .any(|cost| { cost.material_id == "equipment_dust" && cost.amount == 2 }));
}

#[test]
fn projectile_vfx_steps_use_projectile_delivery() {
    let game_data = common::load_game_data_from_ron();

    for skill in game_data.skill_data.skills.iter() {
        for step in &skill.steps {
            if step.presentation.projectile_vfx_id.is_some() {
                assert!(
                    matches!(step.delivery, DeliveryDef::Projectile { .. }),
                    "skill `{}` step `{}` declares projectile_vfx_id but uses {:?}",
                    skill.id,
                    step.id,
                    step.delivery
                );
            }
        }
    }
}

#[test]
fn remaining_instant_steps_are_intentionally_non_spatial() {
    let game_data = common::load_game_data_from_ron();

    for skill in game_data.skill_data.skills.iter() {
        for step in &skill.steps {
            if matches!(step.delivery, DeliveryDef::Instant) {
                assert!(
                    step.presentation.projectile_vfx_id.is_none(),
                    "skill `{}` step `{}` still uses Instant despite projectile_vfx_id",
                    skill.id,
                    step.id
                );
            }
        }
    }
}

#[test]
fn live_resource_skill_references_resolve() {
    let game_data = common::load_game_data_from_ron();

    for abnormality in &game_data.abnormality_data.items {
        if let Some(skill_id) = abnormality.skill_id.as_deref() {
            assert!(
                game_data.skill_data.get_by_id(skill_id).is_some(),
                "abnormality `{}` references missing skill `{}`",
                abnormality.id,
                skill_id
            );
        }
    }

    for equipment in &game_data.equipment_data.items {
        for binding in &equipment.ability_activations {
            assert!(
                game_data
                    .skill_data
                    .get_by_id(&binding.ability_id)
                    .is_some(),
                "equipment `{}` references missing skill `{}`",
                equipment.id,
                binding.ability_id
            );
        }
    }
}

#[test]
fn live_pve_references_resolve() {
    let game_data = common::load_game_data_from_ron();

    for encounter in &game_data.pve_data.encounters {
        assert!(
            encounter.node_type.is_some(),
            "live pve encounter `{}` must explicitly declare combat node purpose",
            encounter.id
        );
        let node_type = encounter.node_type.expect("checked above");
        assert!(
            game_core::game::combat_setup::mission_policy::CombatMissionPolicy::is_supported_encounter_node_type(node_type),
            "live pve encounter `{}` must not use deferred combat node type {:?}",
            encounter.id,
            node_type
        );
        match encounter.encounter_class {
            game_core::game::data::pve_data::PveEncounterClass::Normal => {
                assert!(
                    encounter.primary_abnormality_id.is_none(),
                    "normal pve encounter `{}` must not define primary_abnormality_id",
                    encounter.id
                );
            }
            game_core::game::data::pve_data::PveEncounterClass::Elite
            | game_core::game::data::pve_data::PveEncounterClass::NormalBoss
            | game_core::game::data::pve_data::PveEncounterClass::FinalBoss => {
                let primary = encounter
                    .primary_abnormality_id
                    .as_deref()
                    .unwrap_or_else(|| {
                        panic!(
                            "{:?} pve encounter `{}` must define primary_abnormality_id",
                            encounter.encounter_class, encounter.id
                        )
                    });
                assert!(
                    game_data.abnormality_data.get_by_id(primary).is_some(),
                    "pve encounter `{}` references missing primary abnormality `{}`",
                    encounter.id,
                    primary
                );
            }
        }
        for wave in encounter.wave_definitions() {
            if let PveWaveSource::GeneratedCorroded { preset_id, .. } = &wave.source {
                let preset = game_data
                    .corroded_wave_data
                    .get_by_id(preset_id)
                    .unwrap_or_else(|| {
                        panic!(
                            "pve encounter `{}` wave `{}` references missing corroded wave preset `{}`",
                            encounter.id, wave.id, preset_id
                        )
                    });
                for role in &preset.role_mix {
                    assert!(
                        game_data
                            .corroded_employee_data
                            .get_by_id(&role.profile_id)
                            .is_some(),
                        "corroded wave preset `{}` references missing corroded employee profile `{}`",
                        preset.id,
                        role.profile_id
                    );
                }
            }

            for enemy in wave.manual_enemies() {
                match enemy {
                    game_core::game::data::pve_data::PveWaveEnemyData::Abnormality {
                        abnormality_id,
                        ..
                    } => assert!(
                        game_data
                            .abnormality_data
                            .get_by_id(abnormality_id)
                            .is_some(),
                        "pve encounter `{}` wave `{}` references missing enemy abnormality `{}`",
                        encounter.id,
                        wave.id,
                        abnormality_id
                    ),
                    game_core::game::data::pve_data::PveWaveEnemyData::CorrodedEmployee {
                        profile_id,
                        ..
                    } => assert!(
                        game_data
                            .corroded_employee_data
                            .get_by_id(profile_id)
                            .is_some(),
                        "pve encounter `{}` wave `{}` references missing corroded employee profile `{}`",
                        encounter.id,
                        wave.id,
                        profile_id
                    ),
                    game_core::game::data::pve_data::PveWaveEnemyData::FacilityEntity { .. } => {
                        panic!("FacilityEntity pve enemy is not implemented yet");
                    }
                }
            }
        }
        for reward_uuid in &encounter.reward_uuids {
            assert!(
                game_data.reward_data.get_by_uuid(reward_uuid).is_some(),
                "pve encounter `{}` references missing reward uuid {}",
                encounter.id,
                reward_uuid
            );
        }
    }
}

#[test]
fn live_map_content_pools_are_safe_and_resolve() {
    let game_data = common::load_game_data_from_ron();
    let map_nodes = game_data.map_node_definitions();
    let facility_templates = game_data.facility_templates();
    let assignment_profiles = game_data.node_assignment_profiles();

    assert!(facility_templates.validate_contract().is_ok());
    assert!(assignment_profiles
        .validate_contract(facility_templates)
        .is_ok());
    for template in &facility_templates.templates {
        for slot in &template.slots {
            for category in &slot.allowed_categories {
                // Start nodes are structural facility anchors synthesized by the
                // generator; they intentionally have no content-pool definition.
                if *category == game_core::game::map::MapNodeCategory::Start {
                    continue;
                }
                assert!(
                    map_nodes
                        .nodes
                        .iter()
                        .any(|definition| definition.category == *category),
                    "facility template `{}` slot `{}` category {:?} must resolve to a node definition",
                    template.id.as_str(),
                    slot.id.as_str(),
                    category
                );
            }
        }
    }

    for definition in &map_nodes.nodes {
        match &definition.payload {
            game_core::game::map::MapNodePayload::Shop {
                shop_pool_id: Some(pool_id),
                ..
            } => {
                let pool = game_data
                    .shop_data
                    .pool_by_id(pool_id)
                    .unwrap_or_else(|| panic!("map shop node references missing pool `{pool_id}`"));
                assert!(
                    !pool.shop_ids.is_empty(),
                    "map shop pool `{}` must not be empty",
                    pool.id
                );
            }
            game_core::game::map::MapNodePayload::HeadquartersContact {
                shop_pool_id: Some(pool_id),
                candidate_count,
            } => {
                let pool = game_data.shop_data.pool_by_id(pool_id).unwrap_or_else(|| {
                    panic!("headquarters contact node references missing shop pool `{pool_id}`")
                });
                assert!(
                    !pool.shop_ids.is_empty(),
                    "headquarters contact shop pool `{}` must not be empty",
                    pool.id
                );
                assert!(
                    *candidate_count > 0,
                    "headquarters contact node must expose at least one recruitment candidate"
                );
                assert!(
                    game_data
                        .employee_definition_data
                        .recruitment_definitions()
                        .len()
                        >= *candidate_count,
                    "employee definitions must cover headquarters candidate_count"
                );
            }
            game_core::game::map::MapNodePayload::Reward {
                reward_pool_id: Some(pool_id),
            } => {
                let pool = game_data
                    .reward_data
                    .pool_by_id(pool_id)
                    .unwrap_or_else(|| {
                        panic!("map reward node references missing pool `{pool_id}`")
                    });
                assert!(
                    !pool.reward_ids.is_empty(),
                    "map reward pool `{}` must not be empty",
                    pool.id
                );
                for reward_id in &pool.reward_ids {
                    assert!(
                        game_data.reward_data.get_by_id(reward_id).is_some(),
                        "reward pool entry `{reward_id}` should resolve"
                    );
                }
            }
            _ => {}
        }
    }
}

#[test]
fn live_sealed_archive_event_and_reachable_node_pin_the_vertical_slice_contract() {
    let game_data = common::load_game_data_from_ron();
    let event = game_data
        .event_data
        .get_by_id("sealed_archive_recovery_01")
        .expect("sealed archive checked Event must be published");
    assert_eq!(event.entry_scene_id.as_str(), "archive_check");

    let checked_scenes = event
        .scenes
        .iter()
        .filter_map(|scene| match &scene.next {
            EventSceneNext::EmployeeCheckChoices { choices } => Some((scene, choices)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(checked_scenes.len(), 1);
    let (check_scene, choices) = checked_scenes[0];
    assert_eq!(check_scene.id.as_str(), "archive_check");
    assert_eq!(choices.len(), 2);

    let decode = choices
        .iter()
        .find(|choice| choice.id.as_str() == "decode_archive_seal")
        .expect("decode choice");
    assert_eq!(decode.check.aptitude, EventAptitude::Analysis);
    assert_eq!(decode.check.dc, 16);
    assert_eq!(
        decode
            .check
            .required_choice_key
            .as_ref()
            .map(|key| key.as_str()),
        Some("sealed_archive_access")
    );
    assert!(matches!(
        decode.success_action.as_ref(),
        Some(EventSuccessAction::Grant { effects })
            if effects == &[RewardEffect::GrantEnkephalin { amount: 80 }]
    ));
    assert_eq!(decode.success_scene_id.as_str(), "archive_decoded");
    assert_eq!(decode.failure_scene_id.as_str(), "archive_seal_rejected");

    let stabilize = choices
        .iter()
        .find(|choice| choice.id.as_str() == "stabilize_archive_power")
        .expect("stabilize choice");
    assert_eq!(stabilize.check.aptitude, EventAptitude::Maintenance);
    assert_eq!(stabilize.check.dc, 12);
    assert!(stabilize.check.required_choice_key.is_none());
    assert!(matches!(
        stabilize.success_action.as_ref(),
        Some(EventSuccessAction::Grant { effects })
            if effects == &[RewardEffect::GrantEnkephalin { amount: 30 }]
    ));
    assert_eq!(
        stabilize.success_scene_id.as_str(),
        "archive_power_stabilized"
    );
    assert_eq!(stabilize.failure_scene_id.as_str(), "archive_power_lost");

    let archive_node = game_data
        .map_node_definitions()
        .nodes
        .iter()
        .find(|definition| definition.kind_id.as_str() == "event_sealed_archive")
        .expect("checked Event must have an explicit generated-map definition");
    assert_eq!(archive_node.category, MapNodeCategory::Event);
    assert!(archive_node.weight > 0);
    assert!(matches!(
        &archive_node.payload,
        MapNodePayload::Event { event_id: Some(event_id) }
            if event_id.as_str() == "sealed_archive_recovery_01"
    ));

    let scanner = game_data
        .equipment_data
        .get_by_id("archive_scanner")
        .expect("archive scanner provider must be published");
    assert!(scanner
        .event_check_modifiers
        .iter()
        .any(|modifier| { modifier.aptitude == EventAptitude::Analysis && modifier.amount == 2 }));
    assert!(scanner
        .event_choice_keys
        .iter()
        .any(|key| key.as_str() == "sealed_archive_access"));

    let technician = game_data
        .employee_definition_data
        .get_by_id("maintenance_tech")
        .expect("maintenance technician must be published");
    assert_eq!(
        technician.profile.event_aptitude,
        EventAptitude::Maintenance
    );
    assert!(technician
        .initial_loadout
        .equipment_ids
        .iter()
        .any(|id| id == "archive_scanner"));
}

#[test]
fn live_pve_reward_contracts_do_not_offer_forbidden_abnormality_materialization() {
    let game_data = common::load_game_data_from_ron();

    for encounter in &game_data.pve_data.encounters {
        for reward_uuid in &encounter.reward_uuids {
            let reward = game_data
                .reward_data
                .get_by_uuid(reward_uuid)
                .expect("pve reward reference should resolve");
            assert!(
                !matches!(
                    reward.id.as_str(),
                    "ego_gift_reward" | "double_ego_gift_reward"
                ),
                "pve encounter `{}` should use material/research rewards instead of generic equipment reward `{}`",
                encounter.id,
                reward.id
            );
        }
    }
}

#[test]
fn live_pve_scenario_authoring_contracts_drive_preview_data() {
    let game_data = common::load_game_data_from_ron();

    for profile_id in [
        "corroded_guard",
        "corroded_rusher",
        "corroded_marksman",
        "corroded_bruiser",
        "corroded_medic",
        "corroded_veteran",
    ] {
        assert!(
            game_data
                .corroded_employee_data
                .get_by_id(profile_id)
                .is_some(),
            "corroded employee profile `{profile_id}` should exist"
        );
    }

    let nameless_fetus = game_data
        .pve_data
        .get_by_id("suppress_nameless_fetus")
        .expect("nameless fetus scenario should exist");
    assert_eq!(nameless_fetus.node_type, Some(CombatNodeType::Defense));
    assert!(matches!(
        nameless_fetus
            .battlefield
            .as_ref()
            .and_then(|battlefield| battlefield.archetype),
        Some(BattlefieldArchetype::OpenHall)
    ));
    assert!(matches!(
        nameless_fetus.authored_win_condition(),
        Some(game_core::game::battle::scenario::WinCondition::ProtectUnit { .. })
    ));
    let nameless_preview = CombatPreview::generate_for_node(
        MapNodeId::new(Uuid::from_u128(0xAC1D)),
        MapNodeCategory::Combat,
        Some("suppress_nameless_fetus"),
        &game_data,
        0,
    );
    let nameless_spawn_zone_ids = nameless_preview
        .spawn_zones
        .iter()
        .map(|zone| zone.id.as_str())
        .collect::<BTreeSet<_>>();
    assert!(
        nameless_preview.spawn_waves[0]
            .spawn_zone_ids
            .iter()
            .all(|zone_id| nameless_spawn_zone_ids.contains(zone_id.as_str())),
        "nameless fetus preview should resolve authored waves to existing spawn zones"
    );
    assert!(nameless_fetus.reward_uuids.iter().any(|uuid| {
        game_data
            .reward_data
            .get_by_uuid(uuid)
            .is_some_and(|reward| reward.id == "early_abnormality_research_reward")
    }));

    let scarecrow = game_data
        .pve_data
        .get_by_id("suppress_scarecrow")
        .expect("scarecrow scenario should exist");
    assert!(scarecrow.reward_uuids.iter().any(|uuid| {
        game_data
            .reward_data
            .get_by_uuid(uuid)
            .is_some_and(|reward| reward.id == "field_equipment_salvage_reward")
    }));

    let freischutz = game_data
        .pve_data
        .get_by_id("suppress_freischutz")
        .expect("freischutz scenario should exist");
    assert_eq!(freischutz.encounter_class, PveEncounterClass::NormalBoss);
    assert_eq!(
        freischutz.primary_abnormality_id.as_deref(),
        Some("t-02-43_freischutz")
    );
    assert_eq!(freischutz.node_type, Some(CombatNodeType::Boss));
    assert!(matches!(
        freischutz
            .battlefield
            .as_ref()
            .and_then(|battlefield| battlefield.archetype),
        Some(BattlefieldArchetype::Ambush)
    ));
    assert_eq!(freischutz.waves.len(), 3);
    assert!(freischutz.waves.iter().all(|wave| wave.time_ms == 5_000));
    let freischutz_ambush = freischutz.waves[1].manual_enemies();
    assert!(matches!(
        freischutz_ambush[0],
        PveWaveEnemyData::CorrodedEmployee { .. }
    ));
    let PveWaveEnemyData::CorrodedEmployee { profile_id, .. } = &freischutz_ambush[0] else {
        panic!("freischutz ambush should use a corroded employee");
    };
    assert_eq!(profile_id, "corroded_marksman");
    assert!(freischutz.reward_uuids.iter().any(|uuid| {
        game_data
            .reward_data
            .get_by_uuid(uuid)
            .is_some_and(|reward| reward.id == "freischutz_black_round_fragment_reward")
    }));

    let freischutz_preview = CombatPreview::generate_for_node(
        MapNodeId::new(Uuid::from_u128(0xF43)),
        MapNodeCategory::Boss,
        Some("suppress_freischutz"),
        game_data.as_ref(),
        99,
    );
    assert_eq!(freischutz_preview.archetype, BattlefieldArchetype::Ambush);
    assert_eq!(freischutz_preview.node_type, CombatNodeType::Boss);
    assert_eq!(freischutz_preview.spawn_waves.len(), 3);
    assert!(freischutz_preview
        .spawn_waves
        .iter()
        .all(|wave| wave.time_ms == 5_000));
    assert!(matches!(
        freischutz_preview.spawn_waves[1].enemy_entries[0].kind,
        EnemyKind::CorrodedEmployee
    ));
    assert_eq!(
        freischutz_preview.spawn_waves[1].enemy_entries[0]
            .profile_id
            .as_deref(),
        Some("corroded_marksman")
    );
    assert_eq!(
        freischutz_preview.spawn_waves[1].enemy_entries[0]
            .appearance_seeds
            .len() as u32,
        freischutz_preview.spawn_waves[1].enemy_entries[0].count
    );
    assert!(freischutz_preview
        .spawn_zones
        .iter()
        .any(|zone| zone.id == "side_ambush"));

    let defense_archive = game_data
        .pve_data
        .get_by_id("suppress_warm_hearted_woodsman")
        .expect("roster corridor defense scenario should exist");
    assert_eq!(defense_archive.encounter_class, PveEncounterClass::Elite);
    assert_eq!(
        defense_archive.primary_abnormality_id.as_deref(),
        Some("f-05-32_warm_hearted_woodsman")
    );
    assert_eq!(defense_archive.node_type, Some(CombatNodeType::Defense));
    assert!(defense_archive.tactical_plan.is_none());
    assert!(defense_archive.authored_win_condition().is_none());
    assert!(matches!(
        defense_archive
            .battlefield
            .as_ref()
            .and_then(|battlefield| battlefield.archetype),
        Some(BattlefieldArchetype::Corridor)
    ));
    assert_eq!(defense_archive.waves.len(), 3);
    assert_eq!(defense_archive.waves[0].spawn_zone_ids, ["north_entry"]);
    assert!(!defense_archive.waves[0].required_for_victory);
    assert!(defense_archive
        .waves
        .iter()
        .take(2)
        .all(|wave| matches!(wave.source, PveWaveSource::GeneratedCorroded { .. })));
    assert!(defense_archive
        .waves
        .iter()
        .any(|wave| matches!(wave.manual_enemies().first(), Some(PveWaveEnemyData::Abnormality { abnormality_id, .. }) if abnormality_id == "f-05-32_warm_hearted_woodsman")));
    assert!(defense_archive.reward_uuids.iter().any(|uuid| {
        game_data
            .reward_data
            .get_by_uuid(uuid)
            .is_some_and(|reward| reward.id == "field_equipment_salvage_reward")
    }));
    assert!(defense_archive.reward_uuids.iter().any(|uuid| {
        game_data
            .reward_data
            .get_by_uuid(uuid)
            .is_some_and(|reward| reward.id == "early_abnormality_research_reward")
    }));

    let defense_archive_preview = CombatPreview::generate_for_node(
        MapNodeId::new(Uuid::from_u128(0xB10C)),
        MapNodeCategory::Combat,
        Some("suppress_warm_hearted_woodsman"),
        game_data.as_ref(),
        17,
    );
    assert_eq!(defense_archive_preview.node_type, CombatNodeType::Defense);
    assert_eq!(
        defense_archive_preview.archetype,
        BattlefieldArchetype::Corridor
    );
    assert_eq!(defense_archive_preview.spawn_waves.len(), 3);
    assert_eq!(
        defense_archive_preview.spawn_waves[0].spawn_zone_ids,
        ["north_entry"]
    );
    assert!(defense_archive_preview
        .spawn_waves
        .iter()
        .take(2)
        .flat_map(|wave| wave.enemy_entries.iter())
        .all(|entry| matches!(entry.kind, EnemyKind::CorrodedEmployee)));
    assert!(defense_archive_preview
        .spawn_waves
        .iter()
        .flat_map(|wave| wave.enemy_entries.iter())
        .any(|entry| matches!(entry.kind, EnemyKind::Abnormality)));

    let defense = game_data
        .pve_data
        .get_by_id("suppress_burrowing_heaven")
        .expect("roster corridor defense scenario should exist");
    assert_eq!(defense.encounter_class, PveEncounterClass::NormalBoss);
    assert_eq!(
        defense.primary_abnormality_id.as_deref(),
        Some("o-04-72_burrowing_heaven")
    );
    assert_eq!(defense.node_type, Some(CombatNodeType::Boss));
    assert!(defense.tactical_plan.is_none());
    assert!(defense.authored_win_condition().is_none());
    assert!(matches!(
        defense
            .battlefield
            .as_ref()
            .and_then(|battlefield| battlefield.archetype),
        Some(BattlefieldArchetype::ChokePoint)
    ));
    assert_eq!(defense.waves.len(), 3);
    assert!(defense.waves.iter().take(2).all(|wave| {
        matches!(
            wave.source,
            PveWaveSource::GeneratedCorroded { ref preset_id, .. }
                if preset_id.starts_with("black_box_breach_")
        )
    }));
    assert!(defense
        .waves
        .iter()
        .any(|wave| matches!(wave.manual_enemies().first(), Some(PveWaveEnemyData::Abnormality { abnormality_id, .. }) if abnormality_id == "o-04-72_burrowing_heaven")));
    assert!(defense.reward_uuids.iter().any(|uuid| {
        game_data
            .reward_data
            .get_by_uuid(uuid)
            .is_some_and(|reward| reward.id == "field_equipment_salvage_reward")
    }));
    assert!(defense.reward_uuids.iter().any(|uuid| {
        game_data
            .reward_data
            .get_by_uuid(uuid)
            .is_some_and(|reward| reward.id == "high_risk_abnormality_research_reward")
    }));

    let defense_preview = CombatPreview::generate_for_node(
        MapNodeId::new(Uuid::from_u128(0xD3F3)),
        MapNodeCategory::Boss,
        Some("suppress_burrowing_heaven"),
        game_data.as_ref(),
        41,
    );
    assert_eq!(defense_preview.node_type, CombatNodeType::Boss);
    assert_eq!(defense_preview.archetype, BattlefieldArchetype::ChokePoint);
    assert_eq!(defense_preview.spawn_waves.len(), 3);
    assert!(defense_preview.spawn_waves.iter().take(2).all(|wave| {
        wave.route_id.is_none()
            && wave.required_for_victory
            && wave
                .enemy_entries
                .iter()
                .all(|entry| matches!(entry.kind, EnemyKind::CorrodedEmployee))
    }));
    assert!(defense_preview
        .spawn_waves
        .iter()
        .flat_map(|wave| wave.enemy_entries.iter())
        .any(|entry| matches!(entry.kind, EnemyKind::Abnormality)));

    assert!(
        game_data.pve_data.get_by_id("suppress_plague_doctor").is_none(),
        "Plague Doctor is WhiteNight's prelude/form and must not exist as a standalone PvE scenario"
    );

    let freischutz = game_data
        .pve_data
        .get_by_id("suppress_freischutz")
        .expect("freischutz scenario should exist");
    assert_eq!(freischutz.encounter_class, PveEncounterClass::NormalBoss);
    assert_eq!(freischutz.node_type, Some(CombatNodeType::Boss));
    assert!(matches!(
        freischutz
            .battlefield
            .as_ref()
            .and_then(|battlefield| battlefield.archetype),
        Some(BattlefieldArchetype::Ambush)
    ));
    let mut tactical_plan = game_core::game::battle::scenario::TacticalPlan::default();
    freischutz.apply_authored_tactical_plan(&mut tactical_plan);
    assert_eq!(tactical_plan.points.len(), 1);
    assert!(matches!(
        tactical_plan.objective,
        game_core::game::battle::scenario::BattleObjective::ProtectUnit { ref unit_ref }
            if unit_ref.0 == "black_box_device"
    ));
}

#[test]
fn live_combat_previews_include_required_ad_ap_threat_warning_tags() {
    let game_data = common::load_game_data_from_ron();
    let seeds = [0, 1, 17, 41, 99];

    for encounter in &game_data.pve_data.encounters {
        for seed in seeds {
            let preview = CombatPreview::generate_for_node(
                MapNodeId::new(Uuid::from_u128(
                    0xADAD_1000_0000_0000_0000_0000_0000_0000_u128 + u128::from(seed),
                )),
                MapNodeCategory::Combat,
                Some(encounter.id.as_str()),
                game_data.as_ref(),
                seed,
            );
            let required = required_briefing_warning_tags_for_spawn_waves(
                &preview.spawn_waves,
                game_data.as_ref(),
            );
            let briefing_tags = preview
                .threat_warnings
                .iter()
                .filter(|warning| warning.source == ThreatWarningSource::Briefing)
                .map(|warning| warning.tag)
                .collect::<std::collections::HashSet<_>>();

            for required_tag in required {
                assert!(
                    briefing_tags.contains(&required_tag),
                    "encounter '{}' seed {} omits required threat warning {:?}",
                    encounter.id,
                    seed,
                    required_tag
                );
            }
        }
    }
}
