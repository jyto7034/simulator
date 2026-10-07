use super::*;
use crate::application::{
    ApplicationBattleAuthority, ApplicationCommand, ApplicationDocument,
    ApplicationExecutionContext, CoreApplication,
};
use crate::game::behavior::{LiveBattleUpdateDto, SkillTargetPreviewResultDto};
use crate::game::reward::RewardEffect;

fn update(app: &mut CoreApplication) -> LiveBattleUpdateDto {
    let result = app.active_battle_reconnect(0).unwrap();
    serde_json::from_value(result.payload().as_value()["battle_update"].clone()).unwrap()
}
fn command(
    app: &mut CoreApplication,
    player: Uuid,
    behavior: PlayerBehavior,
) -> crate::application::ApplicationResult {
    let current = update(app);
    app.execute(
        player,
        ApplicationCommand::from_document(ApplicationDocument::new(
            serde_json::to_value(behavior).unwrap(),
        ))
        .unwrap(),
        ApplicationExecutionContext {
            expected_battle_authority: Some(ApplicationBattleAuthority {
                battle_uuid: current.battle_uuid,
                battle_state_revision: current.battle_state_revision,
            }),
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn u07_public_official_owned_forms_preview_cast_paused_reconnect_clone_and_continue() {
    for (fragment_id, awakened, skill, object_id, employee_definition) in [
        (
            "fragment_laetitia_trace",
            false,
            "laetitia_place_friend",
            "laetitia_small_friend",
            "field_medic",
        ),
        (
            "fragment_laetitia_trace",
            true,
            "laetitia_place_great_friend",
            "laetitia_big_friend",
            "field_medic",
        ),
        (
            "u07_ice_consumer",
            false,
            "common_place_ice_wall",
            "common_ice_wall",
            "breach_guard",
        ),
        (
            "u07_meat_consumer",
            false,
            "melting_love_place_meat_wall",
            "melting_love_meat_wall",
            "breach_guard",
        ),
    ] {
        let live = crate::game::data::ValidatedGameData::load_live_embedded().unwrap();
        let mut fragments = live.skill_fragment_data.fragments.clone();
        // Supply standalone official wall skills through explicit owned test
        // fragments. No unfinished live reward/encounter policy is authored.
        for (id, uuid, skill) in [
            ("u07_ice_consumer", 0xC00701, "common_place_ice_wall"),
            (
                "u07_meat_consumer",
                0xC00702,
                "melting_love_place_meat_wall",
            ),
        ] {
            let mut fragment = active_skill_fragment(id, uuid, skill);
            fragment.compatibility.allowed_weapon_archetypes =
                vec![crate::game::data::equipment_data::WeaponArchetype::Shield];
            fragments.push(fragment);
        }
        let data = GameDataBuilder::from_validated(&live)
            .with_skill_fragments(SkillFragmentDatabase::new(fragments))
            .build_arc();
        let (mut core, player) = super::u06_starter::start_selected_with_data(
            &["field_medic", "breach_guard", "records_clerk"],
            data.clone(),
        );
        let employee = core
            .state
            .roster
            .iter()
            .find(|e| e.employee_definition_id == employee_definition)
            .unwrap()
            .uuid;
        let old = core
            .state
            .roster
            .get(&employee)
            .unwrap()
            .skill_fragments
            .active_copy_uuid()
            .unwrap();
        core.execute(
            player,
            PlayerBehavior::UnequipSkillFragment {
                employee_uuid: employee,
                copy_uuid: old,
            },
        )
        .unwrap();
        let copy = core
            .apply_grant_effects(&[RewardEffect::GrantSkillFragment {
                fragment_id: fragment_id.into(),
            }])
            .unwrap()
            .skill_fragment_diffs[0]
            .copy_uuid;
        core.execute(
            player,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid: employee,
                copy_uuid: copy,
            },
        )
        .unwrap();
        if awakened {
            // Explicit pre-unlocked ownership fixture: official bloom recipe is
            // still unavailable. Normal maintenance selection owns the change.
            let mut inventory = serde_json::to_value(&core.state.skill_fragments).unwrap();
            inventory["copies"][copy.to_string()]["unlocked_forms"] = json!(["base", "awakened"]);
            core.state.skill_fragments = serde_json::from_value(inventory).unwrap();
            let maintenance = force_first_available_node(
                &mut core,
                MapNodeCategory::Maintenance,
                "maintenance",
                MapNodePayload::Maintenance,
            );
            select_and_confirm_map_node(&mut core, player, maintenance);
            core.execute(
                player,
                PlayerBehavior::SelectSkillFragmentForm {
                    copy_uuid: copy,
                    form_id: "awakened".into(),
                },
            )
            .unwrap();
            core.execute(player, PlayerBehavior::CompleteNode).unwrap();
        }
        let node = force_first_available_node(
            &mut core,
            MapNodeCategory::Combat,
            "combat_monster",
            MapNodePayload::Encounter {
                encounter_id: Some("suppress_burrowing_heaven".into()),
            },
        );
        let expected_copies = serde_json::to_value(&core.state.skill_fragments).unwrap();
        let expected_inventory = serde_json::to_value(&core.state.inventory).unwrap();
        select_and_confirm_map_node(&mut core, player, node);
        let profile =
            crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee(
                core.roster().unwrap(),
                core.inventory().unwrap(),
                &core.state.skill_fragments,
                &core.game_data,
                employee,
            )
            .unwrap();
        let position = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .combat_preview
            .deployment_zones
            .iter()
            .find(|z| z.kind.supports_affinity(profile.deployment_affinity))
            .unwrap()
            .cells[0];
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::DeployUnit {
                employee_uuid: employee,
                position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
        let active = core.state.active_battle.as_mut().unwrap();
        let unit_id =
            active.live_deployment.as_ref().unwrap().deployed_units[&employee].unit_instance_id;
        // Readiness fixture avoids an unrelated 80-second charge/wave wait.
        let caster = active.battle.units.get_mut(&unit_id).unwrap();
        caster.resonance_current = caster.resonance_max;
        let health_before = caster.stats.current_health;
        let time = active.execution.last_event_time_ms();
        let target = active
            .combat_preview
            .valid_tiles
            .iter()
            .copied()
            .find(|tile| {
                active
                    .battle
                    .object_creation_preview(
                        time,
                        unit_id,
                        &SkillId::from(skill),
                        Some(SkillCastTarget::Tile { position: *tile }),
                    )
                    .is_ok()
            })
            .expect("eligible authored object tile");
        let target = SkillCastTarget::Tile { position: target };
        let mut app = CoreApplication::from_core(core);
        command(
            &mut app,
            player,
            PlayerBehavior::BeginSkillTargeting {
                employee_uuid: employee,
                skill_id: skill.into(),
            },
        );
        let before = update(&mut app);
        let preview_result = command(
            &mut app,
            player,
            PlayerBehavior::PreviewSkillTarget {
                employee_uuid: employee,
                skill_id: skill.into(),
                target,
            },
        );
        let preview: SkillTargetPreviewResultDto =
            serde_json::from_value(preview_result.payload().as_value().clone()).unwrap();
        assert!(preview.eligibility.enabled, "{skill}");
        assert_eq!(preview.object_placements.len(), 1);
        let after_query = update(&mut app);
        assert_eq!(
            after_query.battle_state_revision,
            before.battle_state_revision
        );
        assert_eq!(
            serde_json::to_value(&after_query.checkpoint).unwrap(),
            serde_json::to_value(&before.checkpoint).unwrap()
        );
        command(
            &mut app,
            player,
            PlayerBehavior::ActivateSkill {
                employee_uuid: employee,
                skill_id: skill.into(),
                target: Some(target),
            },
        );
        app.advance_active_battle_by(500, 0).unwrap();
        let created = update(&mut app);
        let object = &created.checkpoint.independent_objects.active[0];
        assert_eq!(object.definition_id.as_str(), object_id);
        assert_eq!(object.source.fragment_copy_uuid.unwrap(), copy);
        assert_eq!(
            object.source.fragment_form_id.as_deref().unwrap(),
            if awakened { "awakened" } else { "base" }
        );
        let caster = created
            .checkpoint
            .units
            .iter()
            .find(|u| u.unit_instance_id == unit_id)
            .unwrap();
        assert_eq!(caster.hud.resonance_current, Some(0));
        if skill == "melting_love_place_meat_wall" {
            let actual_cost = health_before - caster.stats.current_health;
            assert!(actual_cost > 0);
            assert_eq!(object.max_health, Some(actual_cost * 2));
            assert_eq!(preview.object_placements[0].health_cost, actual_cost);
        }
        command(&mut app, player, PlayerBehavior::PauseBattle);
        let paused = update(&mut app);
        let mut independent = app.fork_for_authoring().unwrap();
        assert!(app
            .advance_active_battle_for_host_tick(5000, 0)
            .unwrap()
            .is_none());
        assert_eq!(
            serde_json::to_value(update(&mut app).checkpoint).unwrap(),
            serde_json::to_value(&paused.checkpoint).unwrap()
        );
        command(&mut independent, player, PlayerBehavior::ResumeBattle);
        independent
            .advance_active_battle_for_host_tick(1000, 0)
            .unwrap();
        assert!(
            update(&mut independent).checkpoint.battle_time_ms > paused.checkpoint.battle_time_ms
        );
        assert_eq!(
            serde_json::to_value(update(&mut app).checkpoint).unwrap(),
            serde_json::to_value(&paused.checkpoint).unwrap()
        );
        command(
            &mut app,
            player,
            PlayerBehavior::WithdrawUnit {
                employee_uuid: employee,
            },
        );
        assert_eq!(
            update(&mut app).checkpoint.independent_objects.active.len(),
            1,
            "Persist after withdraw while paused"
        );
        let saved = app.export_active_run_checkpoint().unwrap().unwrap();
        let mut restarted = GameCore::new(data, 0xC007DEAD);
        restarted
            .continue_from_active_run_checkpoint(
                player,
                saved.core_payload_json(),
                saved.loads_used(),
                saved.run_peak_certified_power(),
            )
            .unwrap();
        assert_eq!(restarted.get_state(), GameState::ViewingMap);
        assert!(restarted.state.active_battle.is_none());
        assert_eq!(
            serde_json::to_value(&restarted.state.skill_fragments).unwrap(),
            expected_copies
        );
        assert_eq!(
            serde_json::to_value(&restarted.state.inventory).unwrap(),
            expected_inventory
        );
    }
}
