mod common;

use game_core::game::ability::{DeliveryDef, SkillAreaAnchorSource};
use game_core::game::battle::buffs::{BuffId, BuffKind};
use game_core::game::battle::types::MobilityKind;
use game_core::game::combat_preview::{
    required_briefing_warning_tags_for_spawn_waves, BattlefieldArchetype, CombatNodeType,
    CombatPreview, DeploymentZoneKind, EnemyKind, SpawnZoneKind, ThreatWarningSource,
};
use game_core::game::data::{
    abnormality_data::AbnormalityDatabase,
    consumable_data::{ConsumableEffect, ConsumableTier},
    pve_data::PveWaveSource,
    reward_data::RewardTag,
    skill_fragment_data::{SkillFragmentEffectDef, SkillFragmentOrigin},
};
use game_core::game::map::{MapNodeCategory, MapNodeDefinitionDatabase, MapNodeId};
use game_core::game::resources::Position;
use game_core::game::reward::RewardEffect;
use uuid::Uuid;

#[test]
fn live_abnormality_ron_reads_airborne_mobility_kind() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../game_resources/data/abnormalities/base.ron");
    let contents = std::fs::read_to_string(&path).expect("abnormalities/base.ron should load");
    let database: AbnormalityDatabase =
        ron::de::from_str(&contents).expect("abnormalities/base.ron should deserialize");

    let punishing_bird = database
        .get_by_id("o-02-56_punishing_bird")
        .expect("Punishing Bird should exist in live abnormality RON");

    assert_eq!(punishing_bird.mobility_kind, MobilityKind::Airborne);
    assert!(!punishing_bird
        .target_traits
        .contains(&game_core::game::battle::types::UnitTargetTrait::Airborne));
}

#[test]
fn load_game_data_from_ron_reads_step_based_skill_schema() {
    let game_data = common::load_game_data_from_ron();

    let skill = game_data
        .skill_data
        .get_by_id("scorched_explosion")
        .expect("scorched_explosion skill should exist in RON data");

    assert_eq!(skill.id.as_str(), "scorched_explosion");
    assert_eq!(skill.steps.len(), 1);
    assert_eq!(skill.steps[0].id, "explode");
    assert_eq!(skill.steps[0].range_units, 2.0);

    let abno = game_data
        .abnormality_data
        .get_by_id("f-01-02")
        .expect("Scorched Girl abnormality should exist in RON data");
    assert_eq!(abno.skill_id.as_deref(), Some("scorched_explosion"));

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

    let one_sin = game_data
        .abnormality_data
        .get_by_id("o-03-03_one_sin")
        .expect("One Sin abnormality should exist in RON data");
    assert_eq!(one_sin.skill_id.as_deref(), Some("one_sin_penitence"));

    let one_sin_skill = game_data
        .skill_data
        .get_by_id("one_sin_penitence")
        .expect("One Sin skill should exist in RON data");
    assert_eq!(one_sin_skill.steps.len(), 2);
    assert_eq!(one_sin_skill.steps[0].id, "penitence_judgement");
    assert_eq!(one_sin_skill.steps[1].id, "penitence_absolution");
    assert_eq!(
        one_sin_skill.steps[0]
            .presentation
            .projectile_vfx_id
            .as_deref(),
        Some("one_sin_penitence_judgement")
    );
    assert_eq!(
        one_sin_skill.steps[1].presentation.impact_vfx_id.as_deref(),
        Some("one_sin_penitence_absolution")
    );
    assert_eq!(
        one_sin_skill.steps[0].presentation.target_anchor.as_deref(),
        Some("Head")
    );

    let one_sin_fragment_skill = game_data
        .skill_data
        .get_by_id("fragment_one_sin_penitence")
        .expect("One Sin fragment imitation skill should exist in RON data");
    assert_eq!(one_sin_fragment_skill.steps.len(), 2);
    assert_eq!(
        one_sin_fragment_skill.steps[0].id,
        "fragment_penitence_judgement"
    );
    assert_eq!(
        one_sin_fragment_skill.steps[1].id,
        "fragment_penitence_absolution"
    );

    let queen = game_data
        .skill_data
        .get_by_id("queen_of_hatred_magical_beam")
        .expect("Queen skill should exist in RON data");
    assert!(queen.steps[0].defense_tile_range.is_some());
    assert!(matches!(
        queen.steps[0].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                ..
            }
        }
    ));

    let melting_love = game_data
        .skill_data
        .get_by_id("melting_love_slime_infection")
        .expect("Melting Love skill should exist in RON data");
    assert!(melting_love.steps[1].defense_tile_range.is_some());
    assert!(matches!(
        melting_love.steps[1].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::ImpactContext,
                ..
            }
        }
    ));

    let white_night = game_data
        .skill_data
        .get_by_id("white_night_pale_benediction")
        .expect("WhiteNight skill should exist in RON data");
    assert!(white_night.steps[0].defense_tile_range.is_some());
    assert!(matches!(
        white_night.steps[0].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                include_caster: true,
                ..
            }
        }
    ));
    assert!(white_night.steps[2].defense_tile_range.is_some());
    assert!(matches!(
        white_night.steps[2].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                include_caster: false,
                ..
            }
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

    let plague = game_data
        .skill_data
        .get_by_id("plague_mass_heal")
        .expect("Plague skill should exist in RON data");
    assert!(plague.steps[0].defense_tile_range.is_some());
    assert!(matches!(
        plague.steps[0].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                include_caster: true,
                ..
            }
        }
    ));

    let fragment = game_data
        .skill_data
        .get_by_id("fragment_universe_nova")
        .expect("Fragment skill should exist in RON data");
    assert!(fragment.steps[0].defense_tile_range.is_some());
    assert!(matches!(
        fragment.steps[0].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                ..
            }
        }
    ));

    let fairy = game_data
        .skill_data
        .get_by_id("fairy_festival_blessing")
        .expect("Fairy skill should exist in RON data");
    assert!(fairy.steps[0].defense_tile_range.is_some());
    assert!(matches!(
        fairy.steps[0].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                include_caster: true,
                ..
            }
        }
    ));

    let dark_lamp = game_data
        .skill_data
        .get_by_id("big_bird_dark_lamp")
        .expect("Dark Lamp skill should exist in RON data");
    assert!(dark_lamp.steps[0].defense_tile_range.is_some());
    assert!(matches!(
        dark_lamp.steps[0].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                ..
            }
        }
    ));
    assert!(dark_lamp.steps[1].defense_tile_range.is_some());
    assert!(matches!(
        dark_lamp.steps[1].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                ..
            }
        }
    ));

    let mountain_skill = game_data
        .skill_data
        .get_by_id("mountain_mass_consumption")
        .expect("Mountain skill should exist in RON data");
    assert!(mountain_skill.steps[0].defense_tile_range.is_some());
    assert!(matches!(
        mountain_skill.steps[0].delivery,
        DeliveryDef::TileArea {
            area: game_core::game::ability::SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                ..
            }
        }
    ));
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
        .get_by_id_str("fragment_one_sin_penitence")
        .expect("One Sin skill fragment should exist in RON data");

    assert!(matches!(
        &fragment.origin,
        Some(SkillFragmentOrigin::Abnormality { abnormality_id })
            if abnormality_id == "o-03-03_one_sin"
    ));
    assert!(matches!(
        &fragment.effect,
        SkillFragmentEffectDef::ActiveSkill {
            imitation_skill_id, ..
        }
            if imitation_skill_id.as_str() == "fragment_one_sin_penitence"
    ));
    assert!(
        game_data
            .skill_data
            .get_by_id("fragment_one_sin_penitence")
            .is_some(),
        "fragment imitation skill must be an independent SkillDef"
    );

    let original_skill = game_data
        .abnormality_data
        .get_by_id("o-03-03_one_sin")
        .and_then(|abnormality| abnormality.skill_id.as_ref())
        .expect("origin abnormality should keep its original skill");
    assert_eq!(original_skill.as_str(), "one_sin_penitence");
}

#[test]
fn load_game_data_from_ron_reads_starter_employee_candidates() {
    let game_data = common::load_game_data_from_ron();

    let candidates = &game_data.starter_employee_data.candidates;
    assert_eq!(candidates.len(), 6);
    assert!(game_data
        .starter_employee_data
        .get_by_id("field_medic")
        .is_some());
    assert!(candidates
        .iter()
        .all(|candidate| !candidate.background.trim().is_empty()));
}

#[test]
fn load_game_data_from_ron_reads_separate_recruitment_candidates() {
    let game_data = common::load_game_data_from_ron();

    let candidates = &game_data.recruitment_employee_data.candidates;
    assert!(!candidates.is_empty());
    assert!(game_data
        .recruitment_employee_data
        .get_by_id("hq_field_medic")
        .is_some());
    assert!(game_data
        .starter_employee_data
        .get_by_id("hq_field_medic")
        .is_none());
}

#[test]
fn live_rewards_can_grant_skill_fragments_from_ron() {
    let game_data = common::load_game_data_from_ron();

    for (reward_id, fragment_id) in [
        (
            "one_sin_penitence_fragment_reward",
            "fragment_one_sin_penitence",
        ),
        ("scorched_spark_fragment_reward", "fragment_scorched_spark"),
        (
            "red_shoes_impulse_fragment_reward",
            "fragment_red_shoes_impulse",
        ),
        (
            "freischutz_black_round_fragment_reward",
            "fragment_freischutz_black_round",
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
            reward.resolved_tags().contains(&RewardTag::SkillFragment),
            "reward `{reward_id}` should be tagged as a skill fragment reward"
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
        .get_by_id("scorched_spark_research_reward")
        .expect("skill fragment research reward should exist in RON data");
    assert!(
        research_reward
            .resolved_tags()
            .contains(&RewardTag::ResearchProgress),
        "skill fragment research reward should carry the ResearchProgress tag"
    );
    assert!(research_reward.effects.iter().any(|effect| matches!(
        effect,
        RewardEffect::GrantSkillFragmentResearch {
            fragment_id,
            amount: 3,
        } if fragment_id.as_str() == "fragment_scorched_spark"
    )));

    let equipment_material_reward = game_data
        .reward_data
        .get_by_id("equipment_dust_reward")
        .expect("equipment material reward should exist in RON data");
    assert!(
        equipment_material_reward
            .resolved_tags()
            .contains(&RewardTag::Equipment),
        "equipment material reward should carry the Equipment tag"
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
        .resolved_tags()
        .contains(&RewardTag::Equipment));
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
        .resolved_tags()
        .contains(&RewardTag::ResearchProgress));
    assert!(high_risk_research_reward
        .effects
        .iter()
        .any(|effect| matches!(
            effect,
            RewardEffect::GrantSkillFragmentResearch {
                fragment_id,
                amount: 2,
            } if fragment_id.as_str() == "fragment_freischutz_black_round"
        )));

    let dismantle_recipe = game_data
        .equipment_data
        .get_dismantle_recipe_by_equipment_id("fourth_match")
        .expect("live equipment dismantle recipe should exist");
    assert!(dismantle_recipe
        .yields
        .iter()
        .any(|material| { material.material_id == "equipment_dust" && material.amount == 1 }));
    let enhancement_recipe = game_data
        .equipment_data
        .get_enhancement_recipe_by_equipment_id("fourth_match")
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

    for artifact in &game_data.artifact_data.items {
        for binding in &artifact.ability_activations {
            assert!(
                game_data
                    .skill_data
                    .get_by_id(&binding.ability_id)
                    .is_some(),
                "artifact `{}` references missing skill `{}`",
                artifact.id,
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
            game_core::game::combat_mission_policy::CombatMissionPolicy::is_supported_encounter_node_type(node_type),
            "live pve encounter `{}` must not use deferred combat node type {:?}",
            encounter.id,
            node_type
        );
        assert!(
            game_data
                .abnormality_data
                .get_by_id(&encounter.abnormality_id)
                .is_some(),
            "pve encounter `{}` references missing primary abnormality `{}`",
            encounter.id,
            encounter.abnormality_id
        );
        for wave in encounter.wave_definitions() {
            if let Some(PveWaveSource::GeneratedCorroded { preset_id, .. }) = &wave.source {
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
                match enemy.kind {
                    EnemyKind::Abnormality => assert!(
                        game_data
                            .abnormality_data
                            .get_by_id(&enemy.abnormality_id)
                            .is_some(),
                        "pve encounter `{}` wave `{}` references missing enemy abnormality `{}`",
                        encounter.id,
                        wave.id,
                        enemy.abnormality_id
                    ),
                    EnemyKind::CorrodedEmployee => {
                        let profile_id = enemy
                            .profile_id
                            .as_deref()
                            .expect("corroded employee wave enemy should specify profile_id");
                        assert!(
                            game_data
                                .corroded_employee_data
                                .get_by_id(profile_id)
                                .is_some(),
                            "pve encounter `{}` wave `{}` references missing corroded employee profile `{}`",
                            encounter.id,
                            wave.id,
                            profile_id
                        );
                    }
                    EnemyKind::FacilityEntity => {
                        panic!("FacilityEntity pve enemy is not implemented yet");
                    }
                }
            }
        }
        for reward_uuid in &encounter.reward_uuids {
            let reward = game_data
                .reward_data
                .get_by_uuid(reward_uuid)
                .unwrap_or_else(|| {
                    panic!(
                        "pve encounter `{}` references missing reward uuid {}",
                        encounter.id, reward_uuid
                    )
                });
            assert!(
                !reward.resolved_tags().contains(&RewardTag::Forbidden),
                "pve encounter `{}` must not offer forbidden legacy reward `{}`",
                encounter.id,
                reward.id
            );
        }
    }
}

#[test]
fn live_map_content_pools_are_safe_and_resolve() {
    let game_data = common::load_game_data_from_ron();
    let map_nodes = MapNodeDefinitionDatabase::builtin();

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
                    game_data.recruitment_employee_data.candidates.len() >= *candidate_count,
                    "recruitment_candidates.ron must cover headquarters candidate_count"
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
                    let reward = game_data
                        .reward_data
                        .get_by_id(reward_id)
                        .expect("reward pool entries should resolve");
                    assert!(
                        !reward.resolved_tags().contains(&RewardTag::Forbidden),
                        "map reward pool `{}` must not contain forbidden reward `{}`",
                        pool.id,
                        reward.id
                    );
                }
            }
            _ => {}
        }
    }
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
                !reward.resolved_tags().contains(&RewardTag::Forbidden),
                "pve encounter `{}` must not offer forbidden legacy reward `{}`",
                encounter.id,
                reward.id
            );
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

    let scorched = game_data
        .pve_data
        .get_by_id("suppress_scorched_girl")
        .expect("scorched scenario should exist");
    assert_eq!(scorched.node_type, Some(CombatNodeType::Defense));
    assert!(matches!(
        scorched
            .battlefield
            .as_ref()
            .and_then(|battlefield| battlefield.archetype),
        Some(BattlefieldArchetype::OpenHall)
    ));
    assert!(matches!(
        scorched.authored_win_condition(),
        Some(game_core::game::battle::scenario::WinCondition::ProtectUnit { .. })
    ));
    assert_eq!(scorched.waves[0].spawn_zone_ids, ["north_west_entry"]);
    assert!(scorched.reward_uuids.iter().any(|uuid| {
        game_data
            .reward_data
            .get_by_uuid(uuid)
            .is_some_and(|reward| reward.id == "scorched_spark_fragment_reward")
    }));

    let red_shoes = game_data
        .pve_data
        .get_by_id("suppress_red_shoes")
        .expect("red shoes scenario should exist");
    assert!(red_shoes.reward_uuids.iter().any(|uuid| {
        game_data
            .reward_data
            .get_by_uuid(uuid)
            .is_some_and(|reward| reward.id == "red_shoes_impulse_fragment_reward")
    }));

    let freischutz = game_data
        .pve_data
        .get_by_id("suppress_freischutz")
        .expect("freischutz scenario should exist");
    assert_eq!(freischutz.node_type, Some(CombatNodeType::Defense));
    assert!(matches!(
        freischutz
            .battlefield
            .as_ref()
            .and_then(|battlefield| battlefield.archetype),
        Some(BattlefieldArchetype::Ambush)
    ));
    assert_eq!(freischutz.waves.len(), 2);
    assert_eq!(freischutz.waves[1].time_ms, 6000);
    assert_eq!(freischutz.waves[1].spawn_zone_ids, ["side_ambush"]);
    assert!(matches!(
        freischutz.waves[1].enemies[0].kind,
        EnemyKind::CorrodedEmployee
    ));
    assert_eq!(
        freischutz.waves[1].enemies[0].profile_id.as_deref(),
        Some("corroded_marksman")
    );
    assert!(freischutz.reward_uuids.iter().any(|uuid| {
        game_data
            .reward_data
            .get_by_uuid(uuid)
            .is_some_and(|reward| reward.id == "freischutz_black_round_fragment_reward")
    }));

    let freischutz_preview = CombatPreview::generate_for_node(
        MapNodeId::new(Uuid::from_u128(0xF43)),
        MapNodeCategory::Combat,
        Some("suppress_freischutz"),
        game_data.as_ref(),
        99,
    );
    assert_eq!(freischutz_preview.archetype, BattlefieldArchetype::Ambush);
    assert_eq!(freischutz_preview.node_type, CombatNodeType::Defense);
    assert_eq!(freischutz_preview.spawn_waves.len(), 2);
    assert_eq!(freischutz_preview.spawn_waves[1].time_ms, 6000);
    assert_eq!(
        freischutz_preview.spawn_waves[1].spawn_zone_ids,
        ["side_ambush"]
    );
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
        .get_by_id("black_box_archive_defense")
        .expect("black box defense scenario should exist");
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
    assert_eq!(defense_archive.waves.len(), 2);
    assert_eq!(defense_archive.waves[0].spawn_zone_ids, ["north_entry"]);
    assert!(!defense_archive.waves[0].required_for_victory);
    assert!(defense_archive
        .waves
        .iter()
        .all(|wave| matches!(wave.source, Some(PveWaveSource::GeneratedCorroded { .. }))));
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
        Some("black_box_archive_defense"),
        game_data.as_ref(),
        17,
    );
    assert_eq!(defense_archive_preview.node_type, CombatNodeType::Defense);
    assert_eq!(
        defense_archive_preview.archetype,
        BattlefieldArchetype::Corridor
    );
    assert_eq!(defense_archive_preview.spawn_waves.len(), 2);
    assert_eq!(
        defense_archive_preview.spawn_waves[0].spawn_zone_ids,
        ["north_entry"]
    );
    assert!(defense_archive_preview
        .spawn_waves
        .iter()
        .flat_map(|wave| wave.enemy_entries.iter())
        .all(|entry| matches!(entry.kind, EnemyKind::CorrodedEmployee)));

    let defense = game_data
        .pve_data
        .get_by_id("defend_black_box_relay")
        .expect("black box defense scenario should exist");
    assert_eq!(defense.node_type, Some(CombatNodeType::Defense));
    assert!(defense.tactical_plan.is_none());
    assert!(defense.authored_win_condition().is_none());
    assert!(matches!(
        defense
            .battlefield
            .as_ref()
            .and_then(|battlefield| battlefield.archetype),
        Some(BattlefieldArchetype::ChokePoint)
    ));
    assert_eq!(defense.waves.len(), 2);
    assert!(defense.waves.iter().all(|wave| {
        matches!(
            wave.source,
            Some(PveWaveSource::GeneratedCorroded { ref preset_id, .. })
                if preset_id.starts_with("black_box_breach_")
        )
    }));
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
            .is_some_and(|reward| reward.id == "early_abnormality_research_reward")
    }));

    let defense_preview = CombatPreview::generate_for_node(
        MapNodeId::new(Uuid::from_u128(0xD3F3)),
        MapNodeCategory::Combat,
        Some("defend_black_box_relay"),
        game_data.as_ref(),
        41,
    );
    assert_eq!(defense_preview.node_type, CombatNodeType::Defense);
    assert_eq!(defense_preview.archetype, BattlefieldArchetype::ChokePoint);
    let route = defense_preview
        .routes
        .iter()
        .find(|route| route.id == "black_box_breach_main")
        .expect("defense preview should expose black box breach route");
    assert_eq!(route.start, Position::new(4, 0));
    assert_eq!(route.end, Position::new(4, 5));
    assert!(route.cells.contains(&route.start));
    assert!(route.cells.contains(&route.end));
    assert!(route
        .cells
        .iter()
        .all(|cell| defense_preview.valid_tiles.contains(cell)));
    assert!(defense_preview.spawn_zones.iter().any(|zone| {
        zone.kind == SpawnZoneKind::Entry
            && zone.id == "north_entry"
            && zone.cells.contains(&route.start)
    }));
    assert!(defense_preview.deployment_zones.iter().any(|zone| {
        zone.kind == DeploymentZoneKind::Ground && zone.cells.contains(&Position::new(4, 6))
    }));
    assert_eq!(defense_preview.spawn_waves.len(), 2);
    assert_eq!(
        defense_preview.spawn_waves[0].spawn_zone_ids,
        ["north_entry"]
    );
    assert!(defense_preview.spawn_waves.iter().all(|wave| {
        wave.route_id.as_deref() == Some("black_box_breach_main")
            && wave.spawn_zone_ids == ["north_entry"]
            && wave.required_for_victory
            && wave
                .enemy_entries
                .iter()
                .all(|entry| matches!(entry.kind, EnemyKind::CorrodedEmployee))
    }));

    let plague = game_data
        .pve_data
        .get_by_id("suppress_plague_doctor")
        .expect("plague doctor scenario should exist");
    assert_eq!(plague.node_type, Some(CombatNodeType::Boss));
    assert!(matches!(
        plague
            .battlefield
            .as_ref()
            .and_then(|battlefield| battlefield.archetype),
        Some(BattlefieldArchetype::BossArena)
    ));
    let mut tactical_plan = game_core::game::battle::scenario::TacticalPlan::default();
    plague.apply_authored_tactical_plan(&mut tactical_plan);
    assert_eq!(tactical_plan.points.len(), 1);
    assert!(matches!(
        tactical_plan.objective,
        game_core::game::battle::scenario::BattleObjective::DefeatBoss { ref unit_ref }
            if unit_ref.0 == "enemy_wave_0_0"
    ));
}

#[test]
fn live_combat_previews_include_required_ad_ap_threat_warning_tags() {
    let game_data = common::load_game_data_from_ron();
    game_data.validate_generated_combat_preview_contracts();
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
