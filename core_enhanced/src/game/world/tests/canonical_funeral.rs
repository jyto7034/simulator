use super::*;
use crate::authoring::composition::canonical_funeral_tests as canonical;
use crate::game::{
    ability::weapon_marks::WeaponStage,
    data::{
        content_pack::{
            self, package_official_with_skill_files, tests::ExternalFixture, ContentSelection,
        },
        equipment_data::EquipmentDatabase,
        relic_data::RelicDatabase,
        skill_fragment_data::SkillFragmentDatabase,
    },
    revival::RevivalVariantPolicy,
};
const ORIGINAL: &str = "t-02-99_funeral_butterfly";
const SOURCE: &str = "normal_boss_funeral_butterfly";
const VARIANT_ENCOUNTER: &str = "suppress_funeral_dying_butterflies";

// This technical encounter fixture proves catalog/reward/copy consumers. The
// old original battle is not the planned failure-free funeral procession, and
// the variant's combat AI is not claimed as the current narrative encounter.
fn data() -> (ExternalFixture, ValidatedGameData) {
    let mut kept = None;
    let loaded = authored_route_with_source_consumer(false, None, false, false, |sources| {
        let fixture = ExternalFixture::official();
        for (path, text) in sources.source_inventory() {
            fixture.write(&format!("official/{path}"), text);
        }
        fixture.write("official/data/bundle.ron", sources.manifest);
        let path = "official/data/map/facility_templates.ron";
        let mut maps: FacilityTemplateDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        maps.layouts[0]
            .activities
            .iter_mut()
            .find(|a| a.id == "u17_boss")
            .unwrap()
            .payload = MapNodePayload::Encounter {
            encounter_id: Some("suppress_funeral_butterfly".into()),
        };
        fixture.write(path, ron::ser::to_string(&maps).unwrap());
        let path = "official/data/abnormalities/base.ron";
        let mut abnormalities: AbnormalityDatabase =
            ron::de::from_str(&fixture.read(path)).unwrap();
        let original = abnormalities
            .items
            .iter_mut()
            .find(|a| a.id == ORIGINAL)
            .unwrap();
        original.response_complete_skill_fragment_id = None;
        original.max_health = 70;
        original.attack = 1;
        original.defense = 0;
        original.magic_resist = 0;
        original.skill_id = None;
        let mut variant = original.clone();
        variant.id = canonical::FAMILY.into();
        variant.uuid = Uuid::from_u128(0x12101);
        variant.name = "죽을 나비의 장례 (전투 입력 시험)".into();
        variant.response_complete_skill_fragment_id = Some(canonical::FRAGMENT.into());
        abnormalities.items.push(variant);
        // Follow-up fights leave enough time for real gun shots and a cast.
        abnormalities
            .items
            .iter_mut()
            .find(|a| a.id == "o-03-89_censored")
            .unwrap()
            .max_health = 1000;
        fixture.write(path, ron::ser::to_string(&abnormalities).unwrap());
        let path = "official/data/skill_fragments/base.ron";
        let mut fragments: SkillFragmentDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        let mut fragment = canonical::fragment();
        if let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
            forms,
            ..
        } = &mut fragment.effect
        {
            for form in forms.values_mut() {
                form.resonance_max = 1;
            }
        }
        *fragments
            .fragments
            .iter_mut()
            .find(|f| f.id.as_str() == canonical::FRAGMENT)
            .unwrap() = fragment;
        fixture.write(path, ron::ser::to_string(&fragments).unwrap());
        let path = "official/data/equipments/base.ron";
        let mut equipment: EquipmentDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        let gear = canonical::equipment();
        for metadata in &gear {
            let previous = equipment
                .items
                .iter_mut()
                .find(|e| e.id == metadata.id)
                .unwrap();
            assert_eq!(previous.uuid, metadata.uuid);
            *previous = metadata.clone();
        }
        fixture.write(path, ron::ser::to_string(&equipment).unwrap());
        let path = "official/data/relics/base.ron";
        let mut relics: RelicDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        for relic in canonical::relics().relics {
            assert!(relics.get_by_id(&relic.id).is_none());
            relics.relics.push(relic);
        }
        fixture.write(path, ron::ser::to_string(&relics).unwrap());
        let path = "official/data/pve/encounters.ron";
        let mut pve: PveEncounterDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        let original = pve
            .encounters
            .iter_mut()
            .find(|e| e.id == "suppress_funeral_butterfly")
            .unwrap();
        for policy in &mut original.outcome_policies {
            policy.reward_uuids.clear();
            policy.grant_node_bonus = false;
        }
        let mut variant = original.clone();
        variant.id = VARIANT_ENCOUNTER.into();
        variant.primary_abnormality_id = Some(canonical::FAMILY.into());
        for wave in &mut variant.waves {
            if let crate::game::data::pve_data::PveWaveSource::Manual(enemies) = &mut wave.source {
                for enemy in enemies {
                    if let crate::game::data::pve_data::PveWaveEnemyData::Abnormality {
                        abnormality_id,
                        ..
                    } = enemy
                    {
                        if abnormality_id == ORIGINAL {
                            *abnormality_id = canonical::FAMILY.into();
                        }
                    }
                }
            }
        }
        pve.encounters.push(variant);
        let source = pve
            .reward_sources
            .iter_mut()
            .find(|s| s.reward_source_id == SOURCE)
            .unwrap();
        source.fragment_id = None;
        source.first_equipment_ids.clear();
        source.repeat_rewards.clear();
        let policy = source.revival.as_mut().unwrap();
        policy.variant = Some(RevivalVariantPolicy {
            abnormality_id: canonical::FAMILY.into(),
            fragment_id: canonical::FRAGMENT.into(),
            equipment_ids: gear.iter().map(|g| g.id.clone()).collect(),
        });
        for stage in [&mut policy.stage_2, &mut policy.stage_3] {
            stage.encounter_id = VARIANT_ENCOUNTER.into();
            stage.equipment_choices.clear();
        }
        fixture.write(path, ron::ser::to_string(&pve).unwrap());
        let mut files = vec![];
        for form in ["base", "bloom", "equipment"] {
            let file = format!("data/skills/funeral-{form}.skill.yaml");
            fixture.write(&format!("official/{file}"), canonical::original(form));
            files.push(file.into());
        }
        let rejected = fixture.root().join("unresolved");
        let error = package_official_with_skill_files(
            &fixture.root().join("official"),
            &rejected,
            &files,
            &Default::default(),
        )
        .unwrap_err();
        assert!(format!("{error:?}").contains("Unresolved"));
        assert!(!rejected.exists());
        for form in ["base", "bloom", "equipment"] {
            let mut d = canonical::resolved(form);
            if form != "equipment" {
                d["skills"][0]["admission"]["capacity"] = json!(1);
            }
            fixture.write(
                &format!("official/data/skills/funeral-{form}.skill.yaml"),
                d.to_string(),
            );
        }
        let receipt = package_official_with_skill_files(
            &fixture.root().join("official"),
            &fixture.root().join("published"),
            &files,
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
    (kept.unwrap(), loaded)
}

fn fight(
    core: &mut GameCore,
    player: Uuid,
    employee: Uuid,
    weapon: Uuid,
    upgraded: bool,
) -> serde_json::Value {
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .battle
        .enable_authoring_observation();
    let mut shot = None;
    let mut cast = None;
    let mut damage = None;
    win_with_advance_consumer(
        core,
        player,
        |core| {
            let unit = core
                .state
                .active_battle
                .as_ref()
                .unwrap()
                .battle
                .units
                .values()
                .find(|u| u.source_owned_uuid == employee)
                .unwrap();
            assert_eq!(
                unit.basic_attack.interval_ms,
                if upgraded { 250 } else { 1000 }
            );
            assert_eq!(unit.block_capacity, 0);
        },
        |core| {
            let active = core.state.active_battle.as_ref().unwrap();
            if let Some(w) = active
                .battle
                .authoring_observation()
                .unwrap()
                .and_then(|o| o.latest_weapon_stage.as_ref())
            {
                if w.equipment.persistent_copy_uuid == Some(weapon) {
                    assert_eq!(
                        w.stage,
                        if upgraded {
                            WeaponStage::Upgraded
                        } else {
                            WeaponStage::Base
                        }
                    );
                    assert!(w.actual_hp_loss > 0);
                    assert_eq!(w.definition_id, format!("equipment:{}", canonical::WEAPON));
                    assert!(
                        w.source_file
                            .ends_with("published/data/skills/funeral-equipment.skill.yaml"),
                        "{w:?}"
                    );
                    shot = Some(serde_json::to_value(w).unwrap());
                }
            }
            if let Some(w) = active
                .battle
                .authoring_observation()
                .unwrap()
                .and_then(|o| o.latest_nonzero_sourced_hp.as_ref())
            {
                if w.source.is_some()
                    && w.fragment
                        .as_ref()
                        .is_some_and(|f| f.definition_id.as_str() == canonical::FRAGMENT)
                    && w.change.hp_delta() < 0
                {
                    damage = Some(serde_json::to_value(w).unwrap());
                }
            }
            let unit = active
                .battle
                .units
                .values()
                .find(|u| u.source_owned_uuid == employee && u.is_active())
                .unwrap();
            let ready = unit.resonance_current == 1;
            let marks = active.battle.weapon_marks_snapshot().unwrap();
            let total: u64 = marks
                .stores
                .iter()
                .filter(|s| s.weapon_copy == weapon)
                .flat_map(|s| {
                    s.units
                        .iter()
                        .map(|(_, n)| n)
                        .chain(s.tiles.iter().map(|(_, n)| n))
                })
                .sum();
            let path_target = active.battle.units.values().any(|u| {
                u.owner == crate::game::enums::Side::Opponent
                    && u.is_active()
                    && active
                        .combat_preview
                        .routes
                        .iter()
                        .any(|r| r.cells.contains(&u.body.projected_tile()))
            });
            if cast.is_none() && ready && (if upgraded { total > 0 } else { path_target }) {
                let skill = format!(
                    "fragment_funeral_butterfly_{}",
                    if upgraded { "bloom" } else { "base" }
                );
                execute_current_battle_mutation(
                    core,
                    player,
                    PlayerBehavior::ActivateSkill {
                        employee_uuid: employee,
                        skill_id: skill.into(),
                        target: None,
                    },
                )
                .unwrap();
                let active = core.state.active_battle.as_ref().unwrap();
                let bursts = active.battle.tile_bursts_snapshot().unwrap();
                assert!(bursts.pending_casts + bursts.released.len() > 0);
                if upgraded {
                    assert!(active
                        .battle
                        .weapon_marks_snapshot()
                        .unwrap()
                        .stores
                        .iter()
                        .filter(|s| s.weapon_copy == weapon)
                        .all(|s| s.units.is_empty() && s.tiles.is_empty()));
                }
                cast = Some(json!({"before_stacks":total,"bursts":bursts}));
            }
            core.advance_active_battle_by(250).unwrap();
        },
    );
    assert!(shot.is_some(), "actual published equipment shot");
    assert!(cast.is_some(), "actual published fragment cast");
    assert!(
        damage.is_some(),
        "the published fragment must actually damage a live enemy, not only admit a cast"
    );
    json!({"upgraded":upgraded,"shot":shot,"cast":cast,"damage":damage})
}

#[test]
fn canonical_funeral_published_variant_bundle_real_equipping_casts_and_permanent_evolution() {
    let (fixture, data) = data();
    let mut core = GameCore::new(data, 12077);
    let player = Uuid::from_u128(0xc0de1200000000000000000000001001);
    start(&mut core, player);
    enter(&mut core, player, "1");
    win(&mut core, player);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert_eq!(
        core.state
            .skill_fragments
            .copies()
            .filter(|c| c.definition_id.as_str() == canonical::FRAGMENT)
            .count(),
        0
    );
    assert!(!core
        .state
        .inventory
        .equipments
        .iter()
        .any(|i| i.meta.id == canonical::WEAPON));
    let option = &core.revival_snapshot().unwrap().hatch_options[0];
    assert_eq!(option.abnormality_id, canonical::FAMILY);
    buy_catalysts(&mut core, player);
    let node_id = node(&core, "6");
    core.execute(player, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(
        player,
        PlayerBehavior::HatchEncounter {
            node_id,
            reward_source_id: SOURCE.into(),
            stage: RevivalStage::Stage2,
            payment: vec![MaterialPayment {
                material_id: "revival_catalyst".into(),
                carried: 1,
                warehouse: 0,
            }],
        },
    )
    .unwrap();
    confirm(&mut core, player);
    win(&mut core, player);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let fragment = core
        .state
        .skill_fragments
        .copies()
        .find(|c| c.definition_id.as_str() == canonical::FRAGMENT)
        .unwrap()
        .copy_uuid;
    let employee = core
        .state
        .roster
        .iter()
        .find(|e| e.employee_definition_id == "scout_runner")
        .unwrap()
        .uuid;
    let gear: Vec<_> = canonical::equipment()
        .iter()
        .map(|g| {
            let copies: Vec<_> = core
                .state
                .inventory
                .equipments
                .iter()
                .filter(|i| i.meta.id == g.id)
                .collect();
            assert_eq!(copies.len(), 1);
            copies[0].instance_uuid
        })
        .collect();
    for copy in &gear {
        core.execute(
            player,
            PlayerBehavior::EquipItem {
                item_uuid: *copy,
                target_unit: employee,
            },
        )
        .unwrap();
    }
    if let Some(previous) = core
        .state
        .roster
        .get(&employee)
        .unwrap()
        .skill_fragments
        .active_copy_uuid()
    {
        core.execute(
            player,
            PlayerBehavior::UnequipSkillFragment {
                employee_uuid: employee,
                copy_uuid: previous,
            },
        )
        .unwrap();
    }
    core.execute(
        player,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: employee,
            copy_uuid: fragment,
        },
    )
    .unwrap();
    enter(&mut core, player, "4");
    let base = fight(&mut core, player, employee, gear[0], false);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    enter(&mut core, player, "9");
    let before = core.get_inventory_snapshot_json().unwrap();
    assert!(core
        .execute(
            player,
            PlayerBehavior::AwakenSkillFragmentWithWeapon {
                copy_uuid: fragment,
                form_id: "bloom".into(),
                weapon_copy_uuid: gear[1],
                materials: None
            }
        )
        .is_err());
    assert_eq!(core.get_inventory_snapshot_json().unwrap(), before);
    core.execute(
        player,
        PlayerBehavior::AwakenSkillFragmentWithWeapon {
            copy_uuid: fragment,
            form_id: "bloom".into(),
            weapon_copy_uuid: gear[0],
            materials: Some(vec![MaterialPayment {
                material_id: "part_funeral_butterfly".into(),
                carried: 2,
                warehouse: 0,
            }]),
        },
    )
    .unwrap();
    for form in ["bloom", "base", "bloom"] {
        core.execute(
            player,
            PlayerBehavior::SelectSkillFragmentForm {
                copy_uuid: fragment,
                form_id: form.into(),
            },
        )
        .unwrap();
        assert_eq!(
            core.state
                .inventory
                .equipments
                .get_item(&gear[0])
                .unwrap()
                .weapon_stage,
            WeaponStage::Upgraded
        );
    }
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
    let saved = core.export_active_run_checkpoint().unwrap().unwrap();
    let inventory = core.get_inventory_snapshot_json().unwrap();
    let mut restored = core_with_profile_for_continue(&core, 12199);
    restored
        .continue_from_active_run_checkpoint(
            player,
            &saved.core_payload_json,
            saved.loads_used,
            saved.run_peak_certified_power,
        )
        .unwrap();
    assert_eq!(restored.get_inventory_snapshot_json().unwrap(), inventory);
    core = restored;
    assert_eq!(
        core.state
            .inventory
            .equipments
            .get_item(&gear[0])
            .unwrap()
            .weapon_stage,
        WeaponStage::Upgraded
    );
    enter(&mut core, player, "5");
    let bloom = fight(&mut core, player, employee, gear[0], true);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let after = core.get_inventory_snapshot_json().unwrap();
    assert!(
        core.execute(
            player,
            PlayerBehavior::SelectSkillFragmentForm {
                copy_uuid: fragment,
                form_id: "base".into()
            }
        )
        .is_err(),
        "form selection requires maintenance context"
    );
    assert_eq!(core.get_inventory_snapshot_json().unwrap(), after);
    core.execute(
        player,
        PlayerBehavior::UnequipSkillFragment {
            employee_uuid: employee,
            copy_uuid: fragment,
        },
    )
    .unwrap();
    assert_eq!(
        core.state
            .inventory
            .equipments
            .get_item(&gear[0])
            .unwrap()
            .weapon_stage,
        WeaponStage::Upgraded
    );
    if let Some(root) = std::env::var_os("FUNERAL_WORLD_EXPORT_DIR") {
        let root = std::path::PathBuf::from(root);
        assert!(!root.exists());
        fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
            std::fs::create_dir_all(to).unwrap();
            for entry in std::fs::read_dir(from).unwrap() {
                let entry = entry.unwrap();
                let target = to.join(entry.file_name());
                if entry.file_type().unwrap().is_dir() {
                    copy_tree(&entry.path(), &target);
                } else {
                    std::fs::copy(entry.path(), target).unwrap();
                }
            }
        }
        copy_tree(fixture.root(), &root.join("content"));
        std::fs::write(
            root.join("witnesses.json"),
            serde_json::to_vec_pretty(
                &json!({"fragment":fragment,"gear":gear,"base":base,"bloom":bloom}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    eprintln!(
        "canonical funeral published World {}",
        json!({"fragment":fragment,"gear":gear,"base":base,"bloom":bloom})
    );
}
