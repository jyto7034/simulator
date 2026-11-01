use super::super::combat::battle_advance_atomic_snapshot;
use super::*;
use crate::game::data::{
    content_pack::{self, package_official, tests::ExternalFixture, ContentSelection},
    skill_fragment_data::{SkillFragmentDatabase, SkillFragmentEffectDef, SkillFragmentOrigin},
};
use crate::game::revival::RevivalVariantPolicy;

const VARIANT: &str = "hatch_variant_fixture";
const FRAGMENT: &str = "hatch_variant_fragment";
const ENCOUNTER: &str = "hatch_variant_encounter";
const GEAR: [&str; 3] = [
    "placeholder_funeral_butterfly_weapon",
    "placeholder_funeral_butterfly_armor",
    "placeholder_funeral_butterfly_accessory",
];

fn data() -> (ExternalFixture, ValidatedGameData) {
    let mut kept = None;
    let data = authored_route_with_source_consumer(false, None, false, false, |sources| {
        let fixture = ExternalFixture::official();
        for (path, text) in sources.source_inventory() {
            fixture.write(&format!("official/{path}"), text);
        }
        fixture.write("official/data/bundle.ron", sources.manifest);
        let path = "official/data/abnormalities/base.ron";
        let mut abnormalities: AbnormalityDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let mut variant = abnormalities
            .items
            .iter()
            .find(|a| a.id == "f-02-58_big_bad_wolf")
            .unwrap()
            .clone();
        variant.id = VARIANT.into();
        variant.uuid = Uuid::from_u128(0x12001);
        variant.name = "Authored hatch variant".into();
        variant.response_complete_skill_fragment_id = Some(FRAGMENT.into());
        abnormalities.items.push(variant);
        fixture.write(path, ron::ser::to_string(&abnormalities).unwrap());
        let path = "official/data/skill_fragments/base.ron";
        let mut fragments: SkillFragmentDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        let mut fragment = fragments
            .fragments
            .iter()
            .find(|f| f.id.as_str() == "fragment_big_bad_wolf_trace")
            .unwrap()
            .clone();
        fragment.id = FRAGMENT.into();
        fragment.uuid = Uuid::from_u128(0x12002);
        fragment.origin = Some(SkillFragmentOrigin::Abnormality {
            abnormality_id: VARIANT.into(),
        });
        fragment.sources.clear();
        fragment.dependencies.clear();
        fragment.effect = SkillFragmentEffectDef::BasicAttackModifier {
            attack_bonus: 1,
            attack_interval_ms_reduction: 0,
        };
        fragments.fragments.push(fragment);
        fixture.write(path, ron::ser::to_string(&fragments).unwrap());
        let path = "official/data/pve/encounters.ron";
        let mut pve: PveEncounterDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        let mut encounter = pve
            .encounters
            .iter()
            .find(|e| e.id == "suppress_big_bad_wolf")
            .unwrap()
            .clone();
        encounter.id = ENCOUNTER.into();
        encounter.primary_abnormality_id = Some(VARIANT.into());
        for wave in &mut encounter.waves {
            if let crate::game::data::pve_data::PveWaveSource::Manual(enemies) = &mut wave.source {
                for enemy in enemies {
                    if let crate::game::data::pve_data::PveWaveEnemyData::Abnormality {
                        abnormality_id,
                        ..
                    } = enemy
                    {
                        if abnormality_id == "f-02-58_big_bad_wolf" {
                            *abnormality_id = VARIANT.into();
                        }
                    }
                }
            }
        }
        pve.encounters.push(encounter);
        let source = pve
            .reward_sources
            .iter_mut()
            .find(|s| s.reward_source_id == "normal_boss_big_bad_wolf")
            .unwrap();
        source.repeat_rewards.clear();
        let policy = source.revival.as_mut().unwrap();
        assert!(
            serde_json::to_value(&policy)
                .unwrap()
                .get("variant")
                .is_none(),
            "legacy policy stays omitted"
        );
        policy.variant = Some(RevivalVariantPolicy {
            abnormality_id: VARIANT.into(),
            fragment_id: FRAGMENT.into(),
            equipment_ids: GEAR.iter().map(|s| (*s).into()).collect(),
        });
        for stage in [&mut policy.stage_2, &mut policy.stage_3] {
            stage.encounter_id = ENCOUNTER.into();
            stage.equipment_choices.clear();
        }
        fixture.write(path, ron::ser::to_string(&pve).unwrap());
        // An existing omen consumer must recognize the first-hatch source too.
        // This is a technical reachability fixture, not final-boss content work.
        let path = "official/data/boss_omen/chains.ron";
        let mut omens: crate::game::data::boss_omen_data::BossOmenChainDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        omens.chains[0].required_fragment = Some(
            crate::game::data::boss_omen_data::BossOmenRequiredFragment {
                fragment_id: FRAGMENT.into(),
                awakening_required: false,
            },
        );
        fixture.write(path, ron::ser::to_string(&omens).unwrap());
        let receipt = package_official(
            &fixture.root().join("official"),
            &fixture.root().join("published"),
            &Default::default(),
        )
        .unwrap();
        let loaded = content_pack::load(
            fixture.root(),
            &ContentSelection {
                packs: vec!["published".into()],
                winners: vec![],
            },
            &Default::default(),
        )
        .unwrap();
        assert_eq!(loaded.identity(), &receipt.content_identity);
        kept = Some(fixture);
        loaded
    });
    (kept.unwrap(), data)
}

fn counts(core: &GameCore) -> (usize, [usize; 3]) {
    (
        core.state
            .skill_fragments
            .copies()
            .filter(|c| c.definition_id.as_str() == FRAGMENT)
            .count(),
        GEAR.map(|id| {
            core.state
                .inventory
                .equipments
                .iter()
                .filter(|i| i.meta.id == id)
                .count()
        }),
    )
}

#[test]
fn variant_hatch_publication_and_real_first_bundle_cancel_continue_and_second_stage() {
    let (_fixture, data) = data();
    let mut core = GameCore::new(data, 12077);
    let player = Uuid::from_u128(0xc0de1200000000000000000000001001);
    start(&mut core, player);
    assert_eq!(counts(&core), (0, [0; 3]));
    enter(&mut core, player, "1");
    win(&mut core, player);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert_eq!(counts(&core), (0, [0; 3]));
    let option = &core.revival_snapshot().unwrap().hatch_options[0];
    assert_eq!(option.abnormality_id, VARIANT);
    assert_eq!(option.display_name, "Authored hatch variant");
    assert_eq!(option.rewards.len(), 5);
    assert!(option.equipment_choices.is_empty());
    buy_catalysts(&mut core, player);
    let first = hatch_stage(&mut core, player, "6", RevivalStage::Stage2);
    let view = &core.revival_snapshot().unwrap().hatched_nodes[&first];
    assert_eq!(view.encounter_id, ENCOUNTER);
    assert_eq!(view.rewards.len(), 5);
    confirm(&mut core, player);
    let mut cancellation_reached = false;
    win_with_advance_consumer(
        &mut core,
        player,
        |_| (),
        |core| {
            let before = battle_advance_atomic_snapshot(core);
            let token = crate::authoring::control::CancellationToken::default();
            let result = crate::authoring::control::with_test_cancellation(
                "encounter_victory_rewards_staged",
                1,
                || {
                    crate::authoring::control::run(
                        &Default::default(),
                        &token,
                        crate::authoring::control::ExecutionPhase::TrialExecution,
                        || core.advance_active_battle_by(250),
                    )
                },
            );
            if token.is_cancelled() {
                assert!(!cancellation_reached);
                cancellation_reached = true;
                let failure = result.unwrap_err();
                assert_eq!(failure.site, "encounter_victory_rewards_staged");
                assert_eq!(battle_advance_atomic_snapshot(core), before);
                assert_eq!(counts(core), (0, [0; 3]));
                core.advance_active_battle_by(250).unwrap();
            } else {
                result.unwrap().0.unwrap();
            }
        },
    );
    assert!(
        cancellation_reached,
        "cancel actual terminal settlement, before acknowledgement"
    );
    assert_eq!(counts(&core), (1, [1; 3]));
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert_eq!(counts(&core), (1, [1; 3]));
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("part_big_bad_wolf"),
        2
    );
    let after = core.export_active_run_checkpoint().unwrap().unwrap();
    assert!(core
        .execute(player, PlayerBehavior::CompleteCombatResult)
        .is_err());
    assert_eq!(
        core.export_active_run_checkpoint()
            .unwrap()
            .unwrap()
            .core_payload_json,
        after.core_payload_json
    );
    let inventory_before_continue = core.get_inventory_snapshot_json().unwrap();
    let mut restored = core_with_profile_for_continue(&core, 12099);
    restored
        .continue_from_active_run_checkpoint(
            player,
            &after.core_payload_json,
            after.loads_used,
            after.run_peak_certified_power,
        )
        .unwrap();
    core = restored;
    assert_eq!(
        core.get_inventory_snapshot_json().unwrap(),
        inventory_before_continue
    );
    assert_eq!(counts(&core), (1, [1; 3]));
    let next = &core.revival_snapshot().unwrap().hatch_options[0];
    assert_eq!(next.abnormality_id, VARIANT);
    assert_eq!(next.rewards.len(), 1);
    let second = hatch_stage(&mut core, player, "7", RevivalStage::Stage3);
    confirm(&mut core, player);
    win(&mut core, player);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert_eq!(counts(&core), (1, [1; 3]));
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("part_big_bad_wolf"),
        6
    );
    let snapshot = core.revival_snapshot().unwrap();
    assert!(snapshot.eggs.is_empty());
    for node in [first, second] {
        assert!(snapshot.hatched_nodes[&node].encounter.resolved);
    }
}

#[test]
fn variant_hatch_catalog_rejects_wrong_kind_bundle_and_ordinary_fragment_route() {
    let (fixture, accepted) = data();
    let path = "official/data/pve/encounters.ron";
    let original: PveEncounterDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    let mut duplicate = original.clone();
    let source = duplicate
        .reward_sources
        .iter_mut()
        .find(|s| s.reward_source_id == "normal_boss_big_bad_wolf")
        .unwrap();
    source.fragment_id = Some(FRAGMENT.into());
    let error = duplicate.validate_contract().unwrap_err();
    assert!(
        error
            .field_path
            .as_deref()
            .is_some_and(|p| p.ends_with(".revival.variant.fragment_id")),
        "{error:?}"
    );
    assert!(
        error.cause.contains("exactly one reward source"),
        "{error:?}"
    );

    for fault in [
        "missing-kind",
        "same-kind",
        "wrong-stage",
        "wrong-fragment",
        "duplicate-gear",
        "missing-gear",
        "original-grant",
    ] {
        let mut pve = original.clone();
        let source = pve
            .reward_sources
            .iter_mut()
            .find(|s| s.reward_source_id == "normal_boss_big_bad_wolf")
            .unwrap();
        let policy = source.revival.as_mut().unwrap();
        let variant = policy.variant.as_mut().unwrap();
        match fault {
            "missing-kind" => variant.abnormality_id = "absent".into(),
            "same-kind" => variant.abnormality_id = "f-02-58_big_bad_wolf".into(),
            "wrong-stage" => policy.stage_3.encounter_id = "suppress_big_bad_wolf".into(),
            "wrong-fragment" => variant.fragment_id = "fragment_funeral_butterfly_eulogy".into(),
            "duplicate-gear" => variant.equipment_ids.push(GEAR[0].into()),
            "missing-gear" => variant.equipment_ids[0] = "absent".into(),
            _ => source.first_equipment_ids.push("standard_armor".into()),
        }
        fixture.write(path, ron::ser::to_string(&pve).unwrap());
        let output = fixture.root().join(fault);
        assert!(
            package_official(
                &fixture.root().join("official"),
                &output,
                &Default::default()
            )
            .is_err(),
            "{fault}"
        );
        assert!(!output.exists());
    }
    fixture.write(path, ron::ser::to_string(&original).unwrap());
    let path = "official/data/events/rewards/base.ron";
    let mut rewards: crate::game::data::reward_data::RewardDatabase =
        ron::de::from_str(&fixture.read(path)).unwrap();
    rewards.rewards[0]
        .effects
        .push(RewardEffect::GrantSkillFragment {
            fragment_id: FRAGMENT.into(),
        });
    fixture.write(path, ron::ser::to_string(&rewards).unwrap());
    let output = fixture.root().join("ordinary-fragment");
    let error = package_official(
        &fixture.root().join("official"),
        &output,
        &Default::default(),
    )
    .unwrap_err();
    assert!(
        error
            .cause
            .contains("ordinary Reward must not grant encounter-source fragment"),
        "{error:?}"
    );
    assert!(!output.exists());
    let retained = content_pack::load(
        fixture.root(),
        &ContentSelection {
            packs: vec!["published".into()],
            winners: vec![],
        },
        &Default::default(),
    )
    .unwrap();
    assert_eq!(retained.identity(), accepted.identity());
}
