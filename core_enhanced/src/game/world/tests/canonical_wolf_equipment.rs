use super::*;
use crate::authoring::composition::canonical_wolf_equipment_tests as canonical;
use crate::game::data::content_pack::{
    self, package_official_with_skill_files, tests::ExternalFixture, ContentSelection,
};
use crate::game::data::equipment_data::EquipmentDatabase;

fn data() -> (ExternalFixture, ValidatedGameData) {
    let mut kept = None;
    let loaded = authored_route_with_source_consumer(false, None, false, false, |sources| {
        let fixture = ExternalFixture::official();
        for (path, text) in sources.source_inventory() {
            fixture.write(&format!("official/{path}"), text);
        }
        fixture.write("official/data/bundle.ron", sources.manifest);
        let path = "official/data/equipments/base.ron";
        let mut equipment: EquipmentDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        let metadata = canonical::catalog();
        let old = equipment
            .items
            .iter_mut()
            .find(|i| i.id == metadata.id)
            .unwrap();
        assert_eq!(old.uuid, metadata.uuid);
        *old = metadata;
        fixture.write(path, ron::ser::to_string(&equipment).unwrap());
        // The last authored combat observes the final form's actual stats and cap.
        let path = "official/data/map/facility_templates.ron";
        let mut maps: FacilityTemplateDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
        // This fixed route reserves its shop and uses the former maintenance
        // slot for the final observation combat; do not request another support.
        maps.layouts[0].allocation.one_reserved_support_counts = [(1, 100)].into();
        let normal = maps.layouts[0].allocation.normal_activity_id.clone();
        maps.layouts[0]
            .rooms
            .iter_mut()
            .find(|r| r.id == "B1-01")
            .unwrap()
            .reservations
            .iter_mut()
            .find(|s| s.slot_id.as_str() == "9")
            .unwrap()
            .activity_id = normal;
        fixture.write(path, ron::ser::to_string(&maps).unwrap());
        let source = "data/skills/big-bad-wolf-equipment.skill.yaml";
        fixture.write(&format!("official/{source}"), canonical::ORIGINAL);
        let files = vec![source.into()];
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
        fixture.write(
            &format!("official/{source}"),
            canonical::resolved().to_string(),
        );
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
fn item(core: &GameCore, copy: Uuid) -> crate::game::resources::EquipmentItemDto {
    core.get_inventory_snapshot_dto()
        .unwrap()
        .equipments
        .into_iter()
        .find(|i| i.instance_uuid == copy)
        .unwrap()
        .item
}
fn claim_mask(core: &mut GameCore, player: Uuid) -> Uuid {
    let rewards = core
        .state
        .active_node_content
        .as_ref()
        .unwrap()
        .as_reward()
        .unwrap()
        .rewards
        .clone();
    assert_eq!(rewards.len(), 1);
    let before: Vec<_> = core
        .state
        .inventory
        .equipments
        .iter()
        .map(|i| i.instance_uuid)
        .collect();
    core.execute(
        player,
        PlayerBehavior::SelectReward {
            reward_id: rewards[0].uuid,
        },
    )
    .unwrap();
    core.execute(player, PlayerBehavior::ClaimReward).unwrap();
    let copy = core
        .state
        .inventory
        .equipments
        .iter()
        .find(|i| !before.contains(&i.instance_uuid))
        .unwrap();
    assert_eq!(copy.meta.id, "placeholder_big_bad_wolf_accessory");
    let copy = copy.instance_uuid;
    let inventory = core.get_inventory_snapshot_json().unwrap();
    assert!(core.execute(player, PlayerBehavior::ClaimReward).is_err());
    assert_eq!(core.get_inventory_snapshot_json().unwrap(), inventory);
    core.execute(player, PlayerBehavior::ExitReward).unwrap();
    copy
}
#[test]
fn canonical_wolf_equipment_real_revived_copy_four_forms_and_continue() {
    let (fixture, data) = data();
    let mut core = GameCore::new(data, 12077);
    let player = Uuid::from_u128(0xc0de1200000000000000000000001001);
    start(&mut core, player);
    let initial_fragments = serde_json::to_value(&core.state.skill_fragments).unwrap();
    enter(&mut core, player, "1");
    win(&mut core, player);
    assert_eq!(
        serde_json::to_value(&core.state.skill_fragments).unwrap(),
        initial_fragments
    );
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    buy_catalysts(&mut core, player);
    let node = hatch_stage(&mut core, player, "6", RevivalStage::Stage2);
    assert_eq!(
        core.revival_snapshot().unwrap().hatched_nodes[&node].equipment_choices,
        vec!["placeholder_big_bad_wolf_accessory".to_string()]
    );
    confirm(&mut core, player);
    win(&mut core, player);
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let copy = claim_mask(&mut core, player);
    assert_eq!(
        serde_json::to_value(&core.state.skill_fragments).unwrap(),
        initial_fragments
    );
    let employee = core
        .state
        .roster
        .iter()
        .find(|e| e.employee_definition_id == "scout_runner")
        .unwrap()
        .uuid;
    core.execute(
        player,
        PlayerBehavior::EquipItem {
            item_uuid: copy,
            target_unit: employee,
        },
    )
    .unwrap();
    assert_eq!(item(&core, copy).form.unwrap().index, 0);
    let mut witnesses = vec![];
    // Form effects before each real qualified victory: +HP, +Attack twice,
    // incoming damage only, then final +Attack. The fifth win stays capped.
    for (index, (slot, form_index, max_hp, attack, next_index, progress)) in [
        ("7", 0, 280, 80, 1, 0),
        ("4", 1, 180, 90, 1, 1),
        ("5", 1, 180, 90, 2, 0),
        ("8", 2, 180, 80, 3, 0),
        ("9", 3, 180, 120, 3, 0),
    ]
    .into_iter()
    .enumerate()
    {
        if index == 0 {
            let node = hatch_stage(&mut core, player, slot, RevivalStage::Stage3);
            assert_eq!(
                core.revival_snapshot().unwrap().hatched_nodes[&node]
                    .equipment_choices
                    .len(),
                1
            );
            confirm(&mut core, player);
        } else {
            enter(&mut core, player, slot);
        }
        core.state
            .active_battle
            .as_mut()
            .unwrap()
            .battle
            .enable_authoring_observation();
        win_with_deployed_consumer(&mut core, player, |core| {
            let active = core.state.active_battle.as_ref().unwrap();
            let actor = active
                .battle
                .units
                .values()
                .find(|u| u.source_owned_uuid == employee && u.is_active())
                .unwrap();
            assert_eq!(
                (actor.stats.max_health, actor.stats.attack),
                (max_hp, attack),
                "form {form_index}"
            );
            let observation = active.battle.authoring_observation().unwrap().unwrap();
            let w = observation.latest_equipment_form.as_ref().unwrap();
            assert_eq!((w.equipment_copy, w.form.index), (copy, form_index));
            assert_eq!(
                w.source_file,
                "published/data/skills/big-bad-wolf-equipment.skill.yaml"
            );
        });
        let settled = item(&core, copy);
        assert_eq!(
            (
                settled.form.as_ref().unwrap().index,
                settled.form.as_ref().unwrap().progress
            ),
            (next_index, progress)
        );
        witnesses.push(json!({"slot":slot,"copy":copy,"before_form":form_index,"max_hp":max_hp,"attack":attack,"settled":settled}));
        core.execute(player, PlayerBehavior::CompleteCombatResult)
            .unwrap();
        if index == 0 {
            let second = claim_mask(&mut core, player);
            assert_ne!(second, copy);
            assert_eq!(
                item(&core, second).form.unwrap().index,
                0,
                "new copy does not inherit first copy progress"
            );
        }
        if index == 1 {
            let expected = core.get_inventory_snapshot_json().unwrap();
            let saved = core.export_active_run_checkpoint().unwrap().unwrap();
            let mut fresh = core_with_profile_for_continue(&core, 12078);
            fresh
                .continue_from_active_run_checkpoint(
                    player,
                    &saved.core_payload_json,
                    saved.loads_used,
                    saved.run_peak_certified_power,
                )
                .unwrap();
            assert_eq!(fresh.get_inventory_snapshot_json().unwrap(), expected);
            core = fresh;
            assert_eq!(
                (
                    item(&core, copy).form.as_ref().unwrap().index,
                    item(&core, copy).form.as_ref().unwrap().progress
                ),
                (1, 1)
            );
        }
    }
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("part_big_bad_wolf"),
        6
    );
    assert!(!core.state.inventory.equipments.iter().any(|i| matches!(
        i.meta.id.as_str(),
        "placeholder_big_bad_wolf_weapon" | "placeholder_big_bad_wolf_armor"
    )));
    eprintln!("canonical wolf forms World {}", json!(witnesses));
    if let Some(root) = std::env::var_os("WOLF_GEAR_WORLD_EXPORT_DIR") {
        let root = std::path::PathBuf::from(root);
        assert!(!root.exists());
        fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
            std::fs::create_dir_all(to).unwrap();
            for entry in std::fs::read_dir(from).unwrap() {
                let e = entry.unwrap();
                let dest = to.join(e.file_name());
                if e.file_type().unwrap().is_dir() {
                    copy_tree(&e.path(), &dest);
                } else {
                    std::fs::copy(e.path(), dest).unwrap();
                }
            }
        }
        copy_tree(fixture.root(), &root.join("content"));
        std::fs::write(
            root.join("witnesses.json"),
            serde_json::to_vec_pretty(&witnesses).unwrap(),
        )
        .unwrap();
    }
}
