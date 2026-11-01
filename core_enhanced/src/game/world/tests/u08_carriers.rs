//! Internal pre-battle catalog/ownership preparation; E1 separately proves
//! normal authored acquisition. Every cast, release, withdrawal and hit below
//! uses ordinary world commands and scheduler events, with observation off.
use super::*;
use crate::game::{
    battle::{
        damage::DamageType, ids::UnitInstanceId, runtime_facts::DamageFact,
        tile_range::TileRangePattern,
    },
    combat_preview::DeploymentZoneKind,
    data::relic_data::RelicDatabase,
    enums::Side,
    world::AdminCommand,
};

const SOURCE: &str = "tests/fixtures/u08/carrier001/skills.ron";
const LITERAL: &str = "u08_carrier_literal";

#[derive(Clone)]
struct Fixture {
    core: GameCore,
    player: Uuid,
    employees: [Uuid; 2],
    actors: [UnitInstanceId; 2],
    relics: [Uuid; 2],
    fragments: [Uuid; 2],
}

#[derive(Clone)]
struct PreparedFixture {
    core: GameCore,
    player: Uuid,
    employees: [Uuid; 2],
    relics: [Uuid; 2],
    fragments: [Uuid; 2],
}

fn wide_range() -> TileRangePattern {
    let mut rows = vec!["XXXXXXXXXXXXXXX"; 15];
    rows[7] = "XXXXXXX@XXXXXXX";
    serde_json::from_value(json!({"include_anchor_tile": false, "rows": rows})).unwrap()
}

fn fixture(skill: &str) -> Fixture {
    fixture_with_basic(skill, false, 100)
}

fn fixture_with_basic(skill: &str, ranged: bool, windup_ms: u32) -> Fixture {
    enter_prepared(prepare_with_catalog(skill, ranged, windup_ms, |data| data))
}

fn prepare_with_catalog(
    skill: &str,
    ranged: bool,
    windup_ms: u32,
    edit: impl FnOnce(crate::game::data::ValidatedGameData) -> crate::game::data::ValidatedGameData,
) -> PreparedFixture {
    let base = GameCore::new(game_data_with_pve_encounters(), 0xe208).game_data;
    assert!(base.skill_data.skills.is_empty());
    let skills = SkillDatabase::from_ron(
        SOURCE,
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/u08/carrier001/skills.ron"
        )),
    )
    .unwrap();
    let extra: SkillFragmentDatabase = ron::de::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/u08/carrier001/fragments.ron"
    )))
    .unwrap();
    let mut fragments = base.skill_fragment_data.fragments.clone();
    fragments.extend(extra.fragments);
    let relics: RelicDatabase = ron::de::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/u08/relics.ron"
    )))
    .unwrap();
    let mut enemies = base.abnormality_data.items.clone();
    let enemy = enemies
        .iter_mut()
        .find(|row| row.id == "defense_risk_abno")
        .unwrap();
    enemy.max_health = 1_000_000;
    enemy.attack = 0;
    let mut equipment = base.equipment_data.items.clone();
    let weapon = equipment
        .iter_mut()
        .find(|row| row.id == "standard_training_sword")
        .unwrap()
        .weapon_profile
        .as_mut()
        .unwrap();
    weapon.damage_type = DamageType::True;
    weapon.defense_tile_range = wide_range();
    weapon.interval_ms = 1500;
    weapon.windup_ms = windup_ms;
    if ranged {
        weapon.range_role = crate::game::data::equipment_data::WeaponRangeRole::Ranged;
        weapon.ranged_reposition_ms = 0;
        weapon.delivery =
            ron::de::from_str("Projectile(speed_units_per_ms: 1000, hit_policy: TargetLocked)")
                .unwrap();
    }
    let mut encounters = base.pve_data.encounters.clone();
    let encounter = encounters
        .iter_mut()
        .find(|row| row.id == "defense_encounter")
        .unwrap();
    encounter.terminal_outcomes = PveEncounter::fixture_terminal_outcomes(Some(50_000));
    for wave in &mut encounter.waves {
        wave.required_for_victory = false;
    }
    let data = GameDataBuilder::from_validated(&base)
        .with_abnormalities(enemies)
        .with_equipment_data(Arc::new(crate::game::data::equipment_data::EquipmentDatabase::with_all(
            equipment,
            base.equipment_data.materials.clone(),
            base.equipment_data.dismantle_recipes.clone(),
            base.equipment_data.enhancement_recipes.clone(),
        )))
        .with_skills(skills)
        .with_skill_fragments(SkillFragmentDatabase::new(fragments))
        .with_relics(relics)
        .with_pve(PveEncounterDatabase::new_with_reward_sources(
            encounters,
            base.pve_data.reward_sources.clone(),
        ))
        .build_arc();
    let mut core = GameCore::new(edit(data.into()), 0xe208);
    let player = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player);
    let available = core.state.roster.combat_eligible_employee_ids();
    let employees = [available[0], available[1]];
    // Internal setup grants only; runtime owners are captured by real battle
    // creation/deployment, never installed or patched by this test.
    core.execute_admin_command(AdminCommand::AdminGrantRelic {
        definition_id: "u08_probe".into(),
        count: 2,
    })
    .unwrap();
    let copies: Vec<_> = core
        .state
        .relics
        .copies()
        .map(|copy| copy.copy_uuid)
        .collect();
    let relics = [copies[0], copies[1]];
    core.execute_admin_command(AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 1 })
        .unwrap();
    core.execute(
        player,
        PlayerBehavior::CorruptRelic {
            copy_uuid: relics[1],
        },
    )
    .unwrap();
    let fragment_id = format!("{skill}_fragment");
    core.execute_admin_command(AdminCommand::AdminGrantSkillFragment {
        fragment_id: fragment_id.clone().into(),
        count: 2,
    })
    .unwrap();
    let copies: Vec<_> = core
        .state
        .skill_fragments
        .copies()
        .filter(|copy| copy.definition_id.as_str() == fragment_id)
        .map(|copy| copy.copy_uuid)
        .collect();
    let fragments = [copies[0], copies[1]];
    for index in 0..2 {
        core.execute(
            player,
            PlayerBehavior::EquipRelic {
                copy_uuid: relics[index],
                employee_uuid: employees[index],
                slot: 2,
            },
        )
        .unwrap();
        core.execute(
            player,
            PlayerBehavior::EquipSkillFragment {
                copy_uuid: fragments[index],
                employee_uuid: employees[index],
            },
        )
        .unwrap();
    }
    PreparedFixture {
        core,
        player,
        employees,
        relics,
        fragments,
    }
}

fn enter_prepared(prepared: PreparedFixture) -> Fixture {
    enter_prepared_with_observation(prepared, false)
}

fn enter_prepared_with_observation(prepared: PreparedFixture, observed: bool) -> Fixture {
    let PreparedFixture {
        mut core,
        player,
        employees,
        relics,
        fragments,
    } = prepared;
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        MapNodePayload::Encounter {
            encounter_id: Some("defense_encounter".into()),
        },
    );
    let BehaviorResult::NodePreview {
        combat_preview: Some(preview),
        ..
    } = core
        .execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap()
    else {
        panic!("preview")
    };
    let route_start = preview
        .routes
        .iter()
        .find(|route| route.id == "defense_main")
        .unwrap()
        .start;
    let mut cells: Vec<_> = preview
        .deployment_zones
        .iter()
        .filter(|zone| zone.kind == DeploymentZoneKind::Ground)
        .flat_map(|zone| zone.cells.iter().copied())
        .collect();
    cells.sort_by_key(|cell| (cell.manhattan(&route_start), cell.y, cell.x));
    cells.dedup();
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    cells.retain(|cell| {
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .player_tactical_slot_occupant(*cell, None)
            .is_none()
    });
    let positions = [cells[0], cells[1]];
    let mut actors = Vec::new();
    for index in 0..2 {
        let result = execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::DeployUnit {
                employee_uuid: employees[index],
                position: positions[index],
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
        let BehaviorResult::BattleUnitDeployed {
            unit_instance_id, ..
        } = result
        else {
            panic!("deployment")
        };
        actors.push(unit_instance_id);
    }
    if observed {
        assert!(core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .authoring_observation()
            .unwrap()
            .is_some());
    } else {
        assert_observation_off(&core);
    }
    Fixture {
        core,
        player,
        employees,
        actors: [actors[0], actors[1]],
        relics,
        fragments,
    }
}

#[test]
fn repeated_battle_delayed_capture_and_terminal_cache_release_old_relic_owners() {
    // Three actual battle lifecycles distinguish the current terminal cache
    // from accumulation of older battle owners or delayed payloads.
    const ROUNDS: usize = 3;
    const SKILL: &str = "u08_carrier_projectile";
    let mut prepared = prepare_with_catalog(SKILL, false, 100, |data| data);
    let retained_data = prepared.core.game_data.clone();
    let mut authoring = GameCore::new_for_authoring(retained_data.clone(), 0xe208);
    authoring.state = prepared.core.state;
    authoring.set_amount_observation_enabled(true);
    prepared.core = authoring;
    let mut owners = Vec::new();
    let mut terminal_facts: Vec<
        std::sync::Weak<crate::game::battle::runtime_facts::RuntimeFactSnapshot>,
    > = Vec::new();
    let mut bindings = Vec::new();
    let mut generations = Vec::new();
    let mut plans = Vec::new();
    let mut measured_extents = Vec::new();
    for round in 0..ROUNDS {
        let mut fixture = enter_prepared_with_observation(prepared, true);
        for owner in &owners {
            assert!(std::sync::Weak::upgrade(owner).is_none());
        }
        if let Some(previous) = terminal_facts.last() {
            assert!(
                previous.upgrade().is_some(),
                "last_terminal is intentionally retained across new entry"
            );
        }
        let (owner, owner_extent) = fixture
            .core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .u08_relic_owner_lifetime()
            .unwrap();
        assert!(owner.upgrade().is_some());
        assert!(owner_extent > 0);
        cast(&mut fixture, 1, SKILL);
        wait_projectile(&mut fixture, 1, SKILL);
        let deliveries = fixture
            .core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .u08_delayed_capture_lifetimes();
        assert_eq!(deliveries.len(), 1);
        let (delivery, delivery_extent) = &deliveries[0];
        let evaluations = delivery.upgrade().unwrap().u08_eval_capture_lifetimes();
        assert_eq!(evaluations.len(), 1);
        let (evaluation, evaluation_extent) = &evaluations[0];
        let (source, generation, plan, binding) =
            evaluation.upgrade().unwrap().u08_capture_link_lifetimes();
        assert!(source.upgrade().is_some());
        assert!(binding.upgrade().is_some());
        assert!(*delivery_extent > 0 && *evaluation_extent > 0);
        measured_extents.push((owner_extent, *delivery_extent, *evaluation_extent));
        assert!(
            measured_extents
                .iter()
                .all(|extent| *extent == measured_extents[0]),
            "same structural release owners have stable individual retained extents"
        );
        let withdrawn = withdraw(&mut fixture, 1);
        let hit = next_skill_hit(&mut fixture, 1, SKILL);
        assert_eq!(hit.requested_damage, 210);
        assert_eq!(hit.actual_hp_loss(), 210);
        assert!(hit.time_ms > withdrawn);
        assert!(Arc::ptr_eq(
            hit.source.relic_numeric().unwrap(),
            &binding.upgrade().unwrap()
        ));
        drop(hit);
        for _ in 0..6000 {
            if fixture.core.state.active_battle.is_none() {
                break;
            }
            fixture
                .core
                .advance_active_battle_to_next_event_bucket()
                .unwrap();
        }
        assert!(fixture.core.state.active_battle.is_none());
        assert!(
            owner.upgrade().is_none(),
            "a terminal observation must not retain the battle relic owner"
        );
        assert!(
            delivery.upgrade().is_none(),
            "consumed delayed delivery must have no strong owner"
        );
        assert!(
            evaluation.upgrade().is_none(),
            "consumed EvalCapture must have no strong owner"
        );
        assert!(
            source.upgrade().is_none(),
            "the release capture source Arc has no remaining owner"
        );
        for previous in &terminal_facts {
            assert!(
                previous.upgrade().is_none(),
                "next terminal replaces older last_terminal facts"
            );
        }
        for previous in &bindings {
            assert!(
                std::sync::Weak::upgrade(previous).is_none(),
                "older binding must not accumulate after cache replacement"
            );
        }
        let (facts, facts_extent) = {
            let observation = fixture.core.authoring_observation().unwrap();
            let terminal = observation.last_terminal.unwrap();
            let receipt = fixture.core.state.confirmed_combat_result.as_ref().unwrap();
            assert!(Arc::ptr_eq(
                &terminal.runtime_facts,
                receipt.runtime_facts.as_ref().unwrap()
            ));
            (
                Arc::downgrade(&terminal.runtime_facts),
                terminal.runtime_facts.retained_extent,
            )
        };
        assert!(facts_extent > 0);
        assert!(
            binding.upgrade().is_some(),
            "latest terminal facts/amount retain the allowed binding"
        );
        fixture
            .core
            .execute(fixture.player, PlayerBehavior::CompleteCombatResult)
            .unwrap();
        assert!(fixture.core.state.confirmed_combat_result.is_none());
        assert!(
            facts.upgrade().is_some(),
            "Complete acknowledges receipt; last_terminal remains cached"
        );
        assert!(binding.upgrade().is_some());
        owners.push(owner);
        terminal_facts.push(facts);
        bindings.push(binding);
        generations.push(generation);
        plans.push(plan);
        println!("U08_OWNER_LIFETIME round={} owner_extent={owner_extent} delivery_extent={delivery_extent} eval_extent={evaluation_extent} terminal_fact_extent={facts_extent} active_owner_released=true current_terminal_cached=true",round+1);
        prepared = PreparedFixture {
            core: fixture.core,
            player: fixture.player,
            employees: fixture.employees,
            relics: fixture.relics,
            fragments: fixture.fragments,
        };
    }
    drop(prepared);
    assert!(owners.iter().all(|weak| weak.upgrade().is_none()));
    assert!(terminal_facts.iter().all(|weak| weak.upgrade().is_none()));
    assert!(bindings.iter().all(|weak| weak.upgrade().is_none()));
    // The retained validated catalog owns these independently of any battle.
    // Weak::upgrade=None proves the strong owner and nested payload are dropped;
    // backing storage may remain allocated until the Weak handles also drop.
    assert!(generations.iter().all(|weak| weak.upgrade().is_some()));
    assert!(plans.iter().all(|weak| weak.upgrade().is_some()));
    assert_eq!(
        retained_data.identity(),
        &*generations[0].upgrade().unwrap()
    );
}

fn assert_observation_off(core: &GameCore) {
    assert!(core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .authoring_observation()
        .unwrap()
        .is_none());
}

fn ready_target(fixture: &mut Fixture, index: usize) -> UnitInstanceId {
    for _ in 0..6000 {
        let active = fixture.core.state.active_battle.as_ref().unwrap();
        let ready = active
            .battle
            .live_skill_readiness(active.execution.last_event_time_ms(), fixture.actors[index])
            .map(|value| serde_json::to_value(value).unwrap());
        let target = active
            .battle
            .units
            .iter()
            .find(|(_, unit)| unit.owner == Side::Opponent && unit.is_active())
            .map(|(id, _)| *id);
        if ready.as_ref().is_some_and(|value| {
            value["eligibility"]["enabled"] == true && value["target_available"] == true
        }) {
            return target.unwrap();
        }
        fixture
            .core
            .advance_active_battle_to_next_event_bucket()
            .unwrap();
    }
    panic!("ordinary scheduler did not produce a ready actor and target");
}

fn cast(fixture: &mut Fixture, index: usize, skill: &str) {
    let target = ready_target(fixture, index);
    execute_current_battle_mutation(
        &mut fixture.core,
        fixture.player,
        PlayerBehavior::BeginSkillTargeting {
            employee_uuid: fixture.employees[index],
            skill_id: skill.into(),
        },
    )
    .unwrap();
    execute_current_battle_mutation(
        &mut fixture.core,
        fixture.player,
        PlayerBehavior::ActivateSkill {
            employee_uuid: fixture.employees[index],
            skill_id: skill.into(),
            target: Some(SkillCastTarget::Unit {
                unit_instance_id: target,
            }),
        },
    )
    .unwrap();
}

fn next_skill_hit(fixture: &mut Fixture, index: usize, skill: &str) -> Arc<DamageFact> {
    for _ in 0..6000 {
        let active = fixture.core.state.active_battle.as_ref().unwrap();
        let facts = active.battle.runtime_fact_snapshot().unwrap();
        if let Some(hit) = facts
            .actors
            .iter()
            .find(|row| row.actor == fixture.actors[index])
            .and_then(|row| row.summary.caused.latest_damage.clone())
            .filter(|hit| {
                hit.source
                    .authored
                    .as_ref()
                    .is_some_and(|source| source.site.action.definition_id.as_str() == skill)
            })
        {
            return hit;
        }
        fixture
            .core
            .advance_active_battle_to_next_event_bucket()
            .unwrap();
    }
    panic!("normal cast did not produce authored skill damage");
}

fn assert_skill_hit(fixture: &Fixture, index: usize, skill: &str, expected: u32, hit: &DamageFact) {
    assert_observation_off(&fixture.core);
    assert_eq!(hit.requested_damage, expected);
    assert!(hit.hp_before > expected);
    assert_eq!(hit.actual_hp_loss(), expected);
    let binding = hit.source.relic_numeric().unwrap();
    assert_eq!(binding.actor, fixture.actors[index]);
    assert_eq!(binding.employee_uuid, fixture.employees[index]);
    assert_eq!(
        binding.generation.as_ref(),
        fixture.core.game_data.identity()
    );
    assert_eq!(binding.contributions.len(), 1);
    assert_eq!(binding.contributions[0].copy_uuid, fixture.relics[index]);
    assert_eq!(
        (
            binding.contributions[0].flat,
            binding.contributions[0].percent
        ),
        if index == 0 { (20, 20) } else { (40, 50) }
    );
    let fragment = hit.source.fragment.as_ref().unwrap();
    assert_eq!(fragment.copy_uuid, fixture.fragments[index]);
    assert_eq!(fragment.definition_id.as_str(), format!("{skill}_fragment"));
    assert_eq!(fragment.skill_id.as_str(), skill);
    assert_eq!(fragment.form_id, "base");
    let authored = hit.source.authored.as_ref().unwrap();
    assert_eq!(authored.source_file, SOURCE);
    assert_eq!(authored.site.action.definition_id.as_str(), skill);
    assert_eq!(authored.origin_definition_id.as_str(), skill);
    assert_eq!(serde_json::to_value(&authored.origin).unwrap(), "Original");
    let effect = fixture
        .core
        .game_data
        .skill_data
        .effect(&authored.site)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&effect.tags).unwrap(),
        json!(["carrier", "shared"])
    );
    assert_eq!(
        serde_json::to_value(effect.source.as_ref().unwrap()).unwrap(),
        serde_json::to_value(authored).unwrap()
    );
}

#[test]
fn original_authored_effect_add_remove_rebuilds_fresh_battle_damage_without_rewriting_catalogs() {
    let original = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/u08/carrier001/skills.ron"
    ));
    let ending = "op: Damage(amount: 100, damage_type: True))])]),";
    let added = original.replacen(ending,
        "op: Damage(amount: 100, damage_type: True))]), (id: \"added_step\", delay_ms: 73, target: CastTarget, delivery: Instant, effects: [(id: \"added_damage\", tags: [\"carrier\", \"shared\"], op: Damage(amount: 50, damage_type: True))])]),", 1);
    assert_ne!(added, original);
    let mut generations = Vec::new();
    let mut retained = Vec::new();
    for (label, authored, has_added) in [
        ("original", original, false),
        ("added", added.as_str(), true),
        ("removed", original, false),
    ] {
        let prepared = prepare_with_catalog(LITERAL, false, 100, |data| {
            GameDataBuilder::from_validated(&data)
                .with_skills(SkillDatabase::from_ron(SOURCE, authored).unwrap())
                .build_arc()
                .into()
        });
        generations.push(prepared.core.game_data.identity().clone());
        let source_before = serde_json::to_vec(
            &prepared
                .core
                .game_data
                .skill_data
                .get_by_id(LITERAL)
                .unwrap(),
        )
        .unwrap();
        let mut battle = enter_prepared(prepared);
        cast(&mut battle, 0, LITERAL);
        let first = next_skill_hit(&mut battle, 0, LITERAL);
        assert_skill_hit(&battle, 0, LITERAL, 144, &first);
        assert_eq!(
            first.source.authored.as_ref().unwrap().site.effect_id,
            "damage"
        );
        let mut loss = first.actual_hp_loss();
        if has_added {
            let second = wait_caused_hit(&mut battle, 0, |hit| {
                hit.ordinal > first.ordinal
                    && hit
                        .source
                        .authored
                        .as_ref()
                        .is_some_and(|source| source.site.effect_id == "added_damage")
            });
            // This is another Original catalog effect, not U09 created closure.
            assert_skill_hit(&battle, 0, LITERAL, 84, &second);
            assert_eq!(
                second
                    .source
                    .authored
                    .as_ref()
                    .unwrap()
                    .site
                    .action
                    .action_id,
                "added_step"
            );
            loss += second.actual_hp_loss();
        }
        assert_eq!(loss, if has_added { 228 } else { 144 });
        // Pass the added Instant step's 73 ms release/impact horizon even in
        // original/removed generations. Count only this actor's Ability HP
        // changes, so ordinary weapon attacks cannot hide a stale effect.
        while battle
            .core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .execution
            .last_event_time_ms()
            <= first.time_ms + 200
        {
            battle
                .core
                .advance_active_battle_to_next_event_bucket()
                .unwrap();
        }
        let entries = &battle
            .core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .event_log
            .entries;
        let skill_losses: Vec<_> = entries
            .iter()
            .filter_map(|entry| match &entry.event {
                crate::game::battle::event_log::BattleLogEvent::HpChanged {
                    source_instance_id: Some(actor),
                    damage_source: Some(crate::game::battle::damage::DamageSource::Ability),
                    hp_before,
                    hp_after,
                    ..
                } if *actor == battle.actors[0] => Some(hp_before.saturating_sub(*hp_after)),
                _ => None,
            })
            .collect();
        assert_eq!(
            skill_losses,
            if has_added { vec![144, 84] } else { vec![144] }
        );
        let added_releases = entries.iter().filter(|entry| matches!(&entry.event,
            crate::game::battle::event_log::BattleLogEvent::AbilityStepTriggered { skill_id, step_id, caster_instance_id, .. }
            if skill_id.as_str() == LITERAL && step_id == "added_step" && *caster_instance_id == battle.actors[0]
        )).count();
        assert_eq!(added_releases, usize::from(has_added));
        let sites = &battle
            .core
            .game_data
            .skill_data
            .get_by_id(LITERAL)
            .unwrap()
            .steps;
        assert_eq!(sites.len(), if has_added { 2 } else { 1 });
        assert_eq!(
            serde_json::to_vec(&battle.core.game_data.skill_data.get_by_id(LITERAL).unwrap())
                .unwrap(),
            source_before
        );
        retained.push((first.clone(), serde_json::to_value(&first).unwrap()));
        println!("U08_ORIGINAL_EFFECT_RECONFIG label={label} actual_hp_loss={loss}");
    }
    assert_ne!(generations[0], generations[1]);
    assert_eq!(generations[0], generations[2]);
    for (fact, before) in retained {
        assert_eq!(serde_json::to_value(fact).unwrap(), before);
    }
}

fn wait_caused_hit(
    fixture: &mut Fixture,
    index: usize,
    matches: impl Fn(&DamageFact) -> bool,
) -> Arc<DamageFact> {
    for _ in 0..6000 {
        if let Some(hit) = latest_hit(fixture, index).filter(|hit| matches(hit)) {
            return hit;
        }
        fixture
            .core
            .advance_active_battle_to_next_event_bucket()
            .unwrap();
    }
    panic!("normal battle did not produce expected caused damage");
}

fn change_form_in_maintenance(prepared: &mut PreparedFixture, index: usize) {
    let node_id = force_first_available_node(
        &mut prepared.core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    prepared
        .core
        .execute(prepared.player, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    prepared
        .core
        .execute(prepared.player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    prepared
        .core
        .execute_admin_command(AdminCommand::AdminSetEnkephalin { amount: 100 })
        .unwrap();
    assert!(prepared
        .core
        .fragment_form_unavailable_reason(prepared.fragments[index], "alternate", false)
        .is_none());
    prepared
        .core
        .execute(
            prepared.player,
            PlayerBehavior::AwakenSkillFragment {
                copy_uuid: prepared.fragments[index],
                form_id: "alternate".into(),
            },
        )
        .unwrap();
    assert_eq!(prepared.core.state.enkephalin.amount, 99);
    prepared
        .core
        .execute(
            prepared.player,
            PlayerBehavior::SelectSkillFragmentForm {
                copy_uuid: prepared.fragments[index],
                form_id: "alternate".into(),
            },
        )
        .unwrap();
    prepared
        .core
        .execute(prepared.player, PlayerBehavior::CompleteNode)
        .unwrap();
}

#[test]
fn typed_form_and_weapon_changes_rebind_actual_skill_and_basic_damage_in_fresh_battles() {
    use crate::game::data::relic_data::{
        RelicDamageEffect, RelicDamageSelector, RelicDamageTarget,
    };
    let prepared = prepare_with_catalog(LITERAL, false, 100, |data| {
        let mut fragments = data.skill_fragment_data.fragments.clone();
        let fragment = fragments
            .iter_mut()
            .find(|row| row.id.as_str() == "u08_carrier_literal_fragment")
            .unwrap();
        let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
            forms,
            ..
        } = &mut fragment.effect
        else {
            panic!("active fragment");
        };
        forms.insert("alternate".into(), ron::de::from_str("(skill_id: \"u08_carrier_literal\", resonance_max: 1, bloom_recipe: Some((enkephalin: 1, materials: [])))").unwrap());
        let mut equipment = data.equipment_data.items.clone();
        let mut alternate = equipment
            .iter()
            .find(|row| row.id == "standard_training_sword")
            .unwrap()
            .clone();
        alternate.id = "u08_alternate_weapon".into();
        alternate.uuid = Uuid::from_u128(0xe208_100);
        equipment.push(alternate);
        let mut relics = (*data.relic_data).clone();
        let effects = |flat, percent| {
            vec![
                RelicDamageEffect {
                    id: "exact_form".into(),
                    selector: RelicDamageSelector {
                        target: RelicDamageTarget::SkillDamage,
                        fragment_id: Some("u08_carrier_literal_fragment".into()),
                        form_id: Some("base".into()),
                        ..Default::default()
                    },
                    flat,
                    percent,
                },
                RelicDamageEffect {
                    id: "exact_weapon".into(),
                    selector: RelicDamageSelector {
                        target: RelicDamageTarget::BasicAttackDamage,
                        equipment_id: Some("standard_training_sword".into()),
                        ..Default::default()
                    },
                    flat,
                    percent,
                },
            ]
        };
        relics.relics[0].effects = effects(20, 20);
        relics.relics[0].corruption.as_mut().unwrap().effects = effects(40, 50);
        GameDataBuilder::from_validated(&data)
            .with_skill_fragments(SkillFragmentDatabase::new(fragments))
            .with_equipment_data(Arc::new(crate::game::data::equipment_data::EquipmentDatabase::with_all(
            equipment,
            data.equipment_data.materials.clone(),
            data.equipment_data.dismantle_recipes.clone(),
            data.equipment_data.enhancement_recipes.clone(),
        )))
            .with_relics(relics)
            .build_arc()
            .into()
    });
    let definitions = serde_json::to_vec(&prepared.core.game_data.relic_data).unwrap();
    let mut original_facts = Vec::new();
    for (form_changed, weapon_changed) in [(false, false), (true, false), (false, true)] {
        let mut branch = prepared.clone();
        if form_changed {
            change_form_in_maintenance(&mut branch, 0);
        }
        let weapon = if weapon_changed {
            branch
                .core
                .execute_admin_command(AdminCommand::AdminGrantEquipment {
                    definition_id: "u08_alternate_weapon".into(),
                    count: 1,
                })
                .unwrap();
            let copy = branch
                .core
                .state
                .inventory
                .equipments
                .iter()
                .find(|item| item.meta.id == "u08_alternate_weapon")
                .unwrap()
                .instance_uuid;
            branch
                .core
                .execute(
                    branch.player,
                    PlayerBehavior::EquipItem {
                        item_uuid: copy,
                        target_unit: branch.employees[0],
                    },
                )
                .unwrap();
            copy
        } else {
            branch
                .core
                .state
                .roster
                .get(&branch.employees[0])
                .unwrap()
                .loadout
                .item_slot
                .iter()
                .find(|item| {
                    branch
                        .core
                        .game_data
                        .equipment_data
                        .get_by_uuid(&item.base_uuid)
                        .unwrap()
                        .id
                        == "standard_training_sword"
                })
                .unwrap()
                .instance_uuid
        };
        let relic_before = serde_json::to_value(&branch.core.state.relics).unwrap();
        let mut battle = enter_prepared(branch);
        let raw_attack = battle
            .core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .units[&battle.actors[0]]
            .stats
            .attack;
        cast(&mut battle, 0, LITERAL);
        let hit = next_skill_hit(&mut battle, 0, LITERAL);
        let expected = if form_changed { 100 } else { 144 };
        assert_eq!(hit.requested_damage, expected);
        assert_eq!(hit.actual_hp_loss(), expected);
        assert_eq!(
            hit.source.fragment.as_ref().unwrap().copy_uuid,
            battle.fragments[0]
        );
        assert_eq!(
            hit.source.fragment.as_ref().unwrap().form_id,
            if form_changed { "alternate" } else { "base" }
        );
        assert!(
            hit.source.equipment.is_none(),
            "fragment skill must not borrow current weapon provider"
        );
        assert_eq!(
            hit.source
                .relic_numeric()
                .map_or(0, |binding| binding.contributions.len()),
            usize::from(!form_changed)
        );
        let basic = wait_caused_hit(&mut battle, 0, |candidate| {
            candidate.ordinal > hit.ordinal
                && candidate.source.authored.is_none()
                && candidate.source.fragment.is_none()
        });
        let expected_basic = if weapon_changed {
            raw_attack
        } else {
            (raw_attack + 20) * 120 / 100
        };
        assert_eq!(basic.requested_damage, expected_basic);
        assert_eq!(basic.actual_hp_loss(), expected_basic);
        assert_eq!(basic.source.actor, Some(battle.actors[0]));
        let unit = &battle
            .core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .units[&battle.actors[0]];
        let slot = unit
            .equipment_slots
            .as_ref()
            .unwrap()
            .slots
            .iter()
            .flatten()
            .find(|slot| slot.binding.persistent_copy_uuid == Some(weapon))
            .unwrap();
        let expected_weapon = if weapon_changed {
            "u08_alternate_weapon"
        } else {
            "standard_training_sword"
        };
        assert_eq!(
            battle
                .core
                .game_data
                .equipment_data
                .get_by_uuid(&slot.binding.base_uuid)
                .unwrap()
                .id,
            expected_weapon
        );
        if !weapon_changed {
            assert_eq!(
                basic
                    .source
                    .relic_numeric()
                    .unwrap()
                    .equipment
                    .unwrap()
                    .persistent_copy_uuid,
                Some(weapon)
            );
        }
        assert_eq!(
            basic
                .source
                .relic_numeric()
                .map_or(0, |binding| binding.contributions.len()),
            usize::from(!weapon_changed)
        );
        assert_eq!(
            serde_json::to_value(&battle.core.state.relics).unwrap(),
            relic_before
        );
        assert_eq!(
            serde_json::to_vec(&battle.core.game_data.relic_data).unwrap(),
            definitions
        );
        original_facts.push((hit.clone(), serde_json::to_value(&hit).unwrap()));
        println!("U08_TYPED_PROVIDER_RECONFIG form={form_changed} weapon={weapon_changed} skill={expected} basic={expected_basic}");
    }
    for (fact, before) in original_facts {
        assert_eq!(serde_json::to_value(fact).unwrap(), before);
    }
}

fn wait_received_hit(fixture: &mut Fixture, index: usize, after_ms: u64) -> Arc<DamageFact> {
    for _ in 0..6000 {
        let active = fixture.core.state.active_battle.as_ref().unwrap();
        let snapshot = active.battle.runtime_fact_snapshot().unwrap();
        if let Some(hit) = snapshot
            .actors
            .iter()
            .find(|actor| actor.actor == fixture.actors[index])
            .and_then(|actor| actor.summary.received.latest_damage.clone())
            .filter(|hit| hit.time_ms > after_ms)
        {
            return hit;
        }
        fixture
            .core
            .advance_active_battle_to_next_event_bucket()
            .unwrap();
    }
    panic!("ordinary enemy basic attack did not hit the intended employee");
}

#[test]
fn physical_magic_modifiers_and_resistance_consume_relic_once_and_negative_penalties_in_real_hp() {
    for (damage_type, expected, unpenalized_loss) in [
        (DamageType::Physical, [85, 126], 90),
        (DamageType::Magic, [108, 157], 95),
    ] {
        for equipment_reduction in [0, 10] {
            let mut prepared = prepare_with_catalog(LITERAL, false, 100, |data| {
                let authored = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/u08/carrier001/skills.ron"))
                .replacen("damage_type: True", &format!("damage_type: {damage_type:?}"), 1)
                .replacen("effects: [(id: \"damage\"", "effects: [(id: \"legacy_modifier\", op: ModifyDamage(modifiers: (damage_amp_percent: 50))), (id: \"damage\"", 1);
                let mut enemies = data.abnormality_data.items.clone();
                let enemy = enemies
                    .iter_mut()
                    .find(|row| row.id == "defense_risk_abno")
                    .unwrap();
                enemy.defense = 150;
                enemy.magic_resist = 100;
                enemy.attack = 100;
                enemy.basic_attack.damage_type = damage_type;
                enemy.basic_attack.range_role = WeaponRangeRole::Ranged;
                enemy.basic_attack.targeting_profile =
                    crate::game::data::equipment_data::TargetingProfile::LowDefenseFirst;
                enemy.basic_attack.defense_tile_range = Some(wide_range());
                enemy.basic_attack.interval_ms = 1000;
                enemy.basic_attack.windup_ms = 10;
                let mut equipment = data.equipment_data.items.clone();
                equipment
                    .iter_mut()
                    .find(|row| row.id == "standard_training_sword")
                    .unwrap()
                    .weapon_profile
                    .as_mut()
                    .unwrap()
                    .ground_defense_focus
                    .as_mut()
                    .unwrap()
                    .incoming_damage_reduction_percent = equipment_reduction;
                GameDataBuilder::from_validated(&data)
                    .with_skills(SkillDatabase::from_ron(SOURCE, &authored).unwrap())
                    .with_abnormalities(enemies)
                    .with_equipment_data(Arc::new(crate::game::data::equipment_data::EquipmentDatabase::with_all(
            equipment,
            data.equipment_data.materials.clone(),
            data.equipment_data.dismantle_recipes.clone(),
            data.equipment_data.enhancement_recipes.clone(),
        )))
                    .build_arc()
                    .into()
            });
            // Synthetic pre-battle base inputs only; deployment builds the runtime
            // stats. The vulnerable Corrupt employee has negative final resistance.
            for index in 0..2 {
                let employee = prepared
                    .core
                    .state
                    .roster
                    .get_mut(&prepared.employees[index])
                    .unwrap();
                employee.profile.base_combat_stats.max_hp = 1_000_000;
                employee.profile.base_combat_stats.defense = if index == 0 { 200 } else { 10 };
                employee.profile.base_combat_stats.magic_resist = if index == 0 { 200 } else { 5 };
                let stats = &mut employee.combat_profile.battle_profile.stats;
                stats.max_health = 1_000_000;
                stats.current_health = 1_000_000;
                stats.defense = if index == 0 { 200 } else { 10 };
                stats.magic_resist = if index == 0 { 200 } else { 5 };
            }
            let mut plain = prepared.clone();
            plain
                .core
                .execute(
                    plain.player,
                    PlayerBehavior::UnequipRelic {
                        copy_uuid: plain.relics[1],
                    },
                )
                .unwrap();
            let mut battle = enter_prepared(prepared);
            for index in 0..2 {
                cast(&mut battle, index, LITERAL);
                let hit = next_skill_hit(&mut battle, index, LITERAL);
                assert_eq!(hit.requested_damage, if index == 0 { 144 } else { 210 });
                assert_eq!(hit.actual_hp_loss(), expected[index]);
                assert_eq!(hit.segments.len(), 1);
                assert_eq!(hit.segments[0].damage_type, damage_type);
                assert_eq!(hit.segments[0].calculated_damage, expected[index]);
                assert_eq!(hit.unattributed_modifiers.damage_amp_percent, 50);
                assert_eq!(hit.source.relic_numeric().unwrap().contributions.len(), 1);
                let target = &battle
                    .core
                    .state
                    .active_battle
                    .as_ref()
                    .unwrap()
                    .battle
                    .units[&hit.target];
                assert_eq!(target.stats.defense, 150);
                assert_eq!(target.stats.magic_resist, 100);
            }
            // Skills require Attack stance. Finish both casts before the
            // typed Defense command activates the equipment incoming modifier.
            let defense_at = set_ground_defense(&mut battle, 1);
            let victim = &battle
                .core
                .state
                .active_battle
                .as_ref()
                .unwrap()
                .battle
                .units[&battle.actors[1]];
            assert_eq!((victim.stats.defense, victim.stats.magic_resist), (-5, -5));
            let penalized = wait_received_hit(&mut battle, 1, defense_at);
            assert_eq!(penalized.requested_damage, 100);
            // Resistance is floored before equipment reduction: 105*90/100=94.
            let penalized_loss = if equipment_reduction == 0 { 105 } else { 94 };
            assert_eq!(penalized.actual_hp_loss(), penalized_loss);
            assert_ground_focus_damage(&battle, 1, equipment_reduction, &penalized);
            let mut plain = enter_prepared(plain);
            let defense_at = set_ground_defense(&mut plain, 1);
            let victim = &plain
                .core
                .state
                .active_battle
                .as_ref()
                .unwrap()
                .battle
                .units[&plain.actors[1]];
            assert_eq!((victim.stats.defense, victim.stats.magic_resist), (10, 5));
            let unpenalized = wait_received_hit(&mut plain, 1, defense_at);
            assert_eq!(unpenalized.requested_damage, 100);
            let plain_loss = if equipment_reduction == 0 {
                unpenalized_loss
            } else {
                // Physical floor(10000/110)=90 then 81; Magic floor(10000/105)=95 then 85.
                match damage_type {
                    DamageType::Physical => 81,
                    DamageType::Magic => 85,
                    _ => unreachable!(),
                }
            };
            assert_eq!(unpenalized.actual_hp_loss(), plain_loss);
            assert_ground_focus_damage(&plain, 1, equipment_reduction, &unpenalized);
            println!("U08_TYPED_MITIGATION type={damage_type:?} equipment_reduction={equipment_reduction} normal={} corrupt={} incoming_plain={plain_loss} incoming_corrupt={penalized_loss}",expected[0],expected[1]);
        }
    }
}

fn set_ground_defense(fixture: &mut Fixture, index: usize) -> u64 {
    fixture.core.advance_active_battle_by(1_500).unwrap();
    execute_current_battle_mutation(
        &mut fixture.core,
        fixture.player,
        PlayerBehavior::SetEngagementStance {
            employee_uuid: fixture.employees[index],
            stance: crate::game::battle::types::EngagementStance::Defense,
        },
    )
    .unwrap();
    time(&fixture.core)
}

fn assert_ground_focus_damage(fixture: &Fixture, index: usize, reduction: u32, hit: &DamageFact) {
    let active = fixture.core.state.active_battle.as_ref().unwrap();
    let tactical = active
        .battle
        .unit_tactical_snapshot(fixture.actors[index], hit.time_ms)
        .unwrap();
    assert_eq!(format!("{:?}", tactical.stance), "Defense");
    assert_eq!(format!("{:?}", tactical.tile_kind), "Ground");
    assert_eq!(format!("{:?}", tactical.tile_effect), "DefenseFocus");
    assert_eq!(tactical.defense_focus_damage_reduction_percent, reduction);
    // Ground focus enters the legacy incoming modifier, not a new Relic or
    // equipment effect receipt. Tie it to the deployed employee's real weapon.
    assert_eq!(
        hit.unattributed_modifiers.damage_reduction_percent,
        reduction as i32
    );
    let owned = fixture
        .core
        .state
        .roster
        .get(&fixture.employees[index])
        .unwrap()
        .loadout
        .item_slot
        .iter()
        .find(|item| {
            fixture
                .core
                .game_data
                .equipment_data
                .get_by_uuid(&item.base_uuid)
                .unwrap()
                .id
                == "standard_training_sword"
        })
        .unwrap();
    let authored = fixture
        .core
        .game_data
        .equipment_data
        .get_by_uuid(&owned.base_uuid)
        .unwrap();
    assert_eq!(authored.id, "standard_training_sword");
    assert_eq!(
        authored
            .weapon_profile
            .as_ref()
            .unwrap()
            .ground_defense_focus
            .unwrap()
            .incoming_damage_reduction_percent,
        reduction
    );
}

#[test]
fn literal_observation_off_same_skill_two_employees_keep_distinct_actual_bindings() {
    let mut fixture = fixture(LITERAL);
    let original = serde_json::to_value(
        fixture
            .core
            .game_data
            .skill_data
            .get_by_id(LITERAL)
            .unwrap(),
    )
    .unwrap();
    cast(&mut fixture, 0, LITERAL);
    let first = next_skill_hit(&mut fixture, 0, LITERAL);
    assert_skill_hit(&fixture, 0, LITERAL, 144, &first);
    let retained = serde_json::to_value(&first).unwrap();
    cast(&mut fixture, 1, LITERAL);
    let second = next_skill_hit(&mut fixture, 1, LITERAL);
    assert_skill_hit(&fixture, 1, LITERAL, 210, &second);
    assert!(!Arc::ptr_eq(
        first.source.relic_numeric().unwrap(),
        second.source.relic_numeric().unwrap()
    ));
    assert_ne!(
        first.source.fragment.as_ref().unwrap().copy_uuid,
        second.source.fragment.as_ref().unwrap().copy_uuid
    );
    assert_eq!(serde_json::to_value(&first).unwrap(), retained);
    assert_eq!(
        serde_json::to_value(
            fixture
                .core
                .game_data
                .skill_data
                .get_by_id(LITERAL)
                .unwrap()
        )
        .unwrap(),
        original
    );
    println!(
        "U08_LITERAL_OBSERVATION_OFF normal={} corrupt={}",
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&second).unwrap()
    );
}

fn time(core: &GameCore) -> u64 {
    core.state
        .active_battle
        .as_ref()
        .unwrap()
        .execution
        .last_event_time_ms()
}

fn latest_hit(fixture: &Fixture, index: usize) -> Option<Arc<DamageFact>> {
    fixture
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .runtime_fact_snapshot()
        .unwrap()
        .actors
        .iter()
        .find(|row| row.actor == fixture.actors[index])
        .and_then(|row| row.summary.caused.latest_damage.clone())
}

fn damage_count(fixture: &Fixture, index: usize) -> u64 {
    fixture
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .runtime_fact_snapshot()
        .unwrap()
        .actors
        .iter()
        .find(|row| row.actor == fixture.actors[index])
        .map_or(0, |row| row.summary.caused.damage_mutations)
}

fn withdraw(fixture: &mut Fixture, index: usize) -> u64 {
    execute_current_battle_mutation(
        &mut fixture.core,
        fixture.player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: fixture.employees[index],
        },
    )
    .unwrap();
    assert!(!fixture
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .units[&fixture.actors[index]]
        .is_active());
    time(&fixture.core)
}

fn until(fixture: &mut Fixture, end: u64) {
    for _ in 0..6000 {
        if time(&fixture.core) >= end {
            return;
        }
        fixture
            .core
            .advance_active_battle_to_next_event_bucket()
            .unwrap();
    }
    panic!("scheduler did not reach {end}");
}

fn wait_projectile(fixture: &mut Fixture, index: usize, skill: &str) {
    for _ in 0..6000 {
        if fixture.core.state.active_battle.as_ref().unwrap().battle.event_log.entries.iter().any(|entry|
            matches!(&entry.event, crate::game::battle::event_log::BattleLogEvent::SkillProjectileLaunched {
                skill_id, caster_instance_id, ..
            } if skill_id.as_str() == skill && *caster_instance_id == fixture.actors[index])) { return; }
        fixture
            .core
            .advance_active_battle_to_next_event_bucket()
            .unwrap();
    }
    panic!("real scheduler did not release projectile");
}

fn assert_pending_cancelled(fixture: &Fixture, index: usize, skill: &str) {
    assert!(!fixture.core.state.active_battle.as_ref().unwrap().battle.event_log.entries.iter().any(|entry|
        matches!(&entry.event, crate::game::battle::event_log::BattleLogEvent::AbilityStepTriggered {
            skill_id, step_id, caster_instance_id, ..
        } if skill_id.as_str() == skill && *caster_instance_id == fixture.actors[index] && step_id == "must_not_release")));
    assert_observation_off(&fixture.core);
}

#[test]
fn released_snapshot_projectile_survives_withdraw_and_unreleased_step_is_cancelled() {
    let skill = "u08_carrier_projectile";
    let mut fixture = fixture(skill);
    cast(&mut fixture, 1, skill);
    wait_projectile(&mut fixture, 1, skill);
    assert!(
        latest_hit(&fixture, 1).is_none_or(|hit| hit
            .source
            .authored
            .as_ref()
            .is_none_or(|source| source.site.action.definition_id.as_str() != skill)),
        "release precedes damage"
    );
    let withdrawn = withdraw(&mut fixture, 1);
    let hit = next_skill_hit(&mut fixture, 1, skill);
    assert!(hit.time_ms > withdrawn);
    assert_skill_hit(&fixture, 1, skill, 210, &hit);
    let count = damage_count(&fixture, 1);
    until(&mut fixture, withdrawn + 4000);
    assert_eq!(
        damage_count(&fixture, 1),
        count,
        "released payload hits once"
    );
    assert_pending_cancelled(&fixture, 1, skill);
    println!(
        "U08_RELEASED_PROJECTILE withdrawn={withdrawn} hit={}",
        serde_json::to_string(&hit).unwrap()
    );
}

#[test]
fn released_ground_fixed_snapshot_area_keeps_binding_for_ticks_after_withdraw() {
    let skill = "u08_carrier_fixed";
    let mut fixture = fixture(skill);
    cast(&mut fixture, 1, skill);
    let first = next_skill_hit(&mut fixture, 1, skill);
    assert_skill_hit(&fixture, 1, skill, 210, &first);
    let binding = serde_json::to_value(first.source.relic_numeric().unwrap()).unwrap();
    let withdrawn = withdraw(&mut fixture, 1);
    let initial_count = damage_count(&fixture, 1);
    let mut seen = first.ordinal;
    let mut later = 0;
    for _ in 0..6000 {
        if time(&fixture.core) >= withdrawn + 4000 {
            break;
        }
        fixture
            .core
            .advance_active_battle_to_next_event_bucket()
            .unwrap();
        if let Some(hit) = latest_hit(&fixture, 1).filter(|hit| hit.ordinal != seen) {
            assert!(hit.time_ms > withdrawn);
            assert_skill_hit(&fixture, 1, skill, 210, &hit);
            assert_eq!(
                serde_json::to_value(hit.source.relic_numeric().unwrap()).unwrap(),
                binding
            );
            seen = hit.ordinal;
            later += 1;
        }
    }
    assert!(time(&fixture.core) >= withdrawn + 4000);
    assert!(
        later > 0,
        "released fixed area must actually hit after withdrawal"
    );
    assert_eq!(damage_count(&fixture, 1), initial_count + later);
    assert_pending_cancelled(&fixture, 1, skill);
    println!("U08_FIXED_AREA withdrawn={withdrawn} later_ticks={later} binding={binding}");
}

#[test]
fn follow_area_initial_hit_is_positive_then_withdraw_cancels_ticks_and_pending_step() {
    let skill = "u08_carrier_follow";
    let mut fixture = fixture(skill);
    cast(&mut fixture, 1, skill);
    let first = next_skill_hit(&mut fixture, 1, skill);
    assert_skill_hit(&fixture, 1, skill, 210, &first);
    let withdrawn = withdraw(&mut fixture, 1);
    let count = damage_count(&fixture, 1);
    until(&mut fixture, withdrawn + 4000);
    assert_eq!(damage_count(&fixture, 1), count);
    assert_eq!(latest_hit(&fixture, 1).unwrap().ordinal, first.ordinal);
    assert_pending_cancelled(&fixture, 1, skill);
    println!(
        "U08_FOLLOW_CANCEL withdrawn={withdrawn} initial_hit={}",
        serde_json::to_string(&first).unwrap()
    );
}

fn atomic_roots(core: &GameCore) -> Value {
    let active = core.state.active_battle.as_ref().unwrap();
    json!({
        "world": non_battle_state_without_profile_fingerprint(core),
        "battle_time": active.execution.last_event_time_ms(),
        "battle_state": active.battle.transaction_debug_snapshot(),
        "queue": active.battle.event_queue_debug_snapshot(),
        "facts": active.battle.runtime_fact_view(),
        "events": active.battle.event_log.to_json_string().unwrap(),
        "revision": active.battle_state_revision,
        "deployment": format!("{:?}", active.live_deployment),
        "attempt": format!("{:?}", active.employee_attempt),
        "relics": core.state.relics,
        "materials": core.state.crafting_resources,
        "uuid": core.state.uuid_manager,
        "savepoint": core.state.run_checkpoint,
        "continue": core.export_active_run_checkpoint().unwrap().map(|saved| saved.core_payload_json),
    })
}

#[test]
fn ordinary_active_battle_rejects_relic_loadout_mutations_without_changing_any_root() {
    let mut fixture = fixture(LITERAL);
    assert!(matches!(
        fixture.core.get_state(),
        GameState::InBattle { .. }
    ));
    assert!(fixture.core.state.active_battle.is_some());
    let before = atomic_roots(&fixture.core);
    for behavior in [
        PlayerBehavior::EquipRelic {
            copy_uuid: fixture.relics[0],
            employee_uuid: fixture.employees[1],
            slot: 1,
        },
        PlayerBehavior::UnequipRelic {
            copy_uuid: fixture.relics[0],
        },
        PlayerBehavior::CorruptRelic {
            copy_uuid: fixture.relics[0],
        },
    ] {
        let action = behavior.kind();
        assert_eq!(
            action.route(),
            crate::game::behavior::ActionRoute::CoreDomain
        );
        assert!(!action.requires_battle_authority());
        for _ in 0..2 {
            // CoreDomain loadout commands need no battle-authority token and
            // skip common preflight. Their domain context guard rejects them.
            let error = fixture
                .core
                .execute(fixture.player, behavior.clone())
                .unwrap_err();
            assert!(
                matches!(error, GameError::InvalidAction),
                "{action:?}: {error:?}"
            );
            assert_eq!(atomic_roots(&fixture.core), before, "{action:?}");
        }
    }
    assert_observation_off(&fixture.core);
}

#[test]
fn live_source_stat_projectile_is_positive_alive_and_missing_context_after_withdraw_is_atomic() {
    let skill = "u08_carrier_live";
    let mut alive = fixture(skill);
    let attack = alive
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .units[&alive.actors[1]]
        .stats
        .attack;
    // This fixture has no aura, critical or True-damage resistance. Source
    // Attack is the pre-authored unit stat; the relic contributes once after it.
    let expected = ((u64::from(attack) + 40) * 150 / 100) as u32;
    cast(&mut alive, 1, skill);
    let hit = next_skill_hit(&mut alive, 1, skill);
    assert_skill_hit(&alive, 1, skill, expected, &hit);

    let mut removed = fixture(skill);
    assert_eq!(
        removed.core.game_data.identity(),
        alive.core.game_data.identity()
    );
    cast(&mut removed, 1, skill);
    wait_projectile(&mut removed, 1, skill);
    let withdrawn = withdraw(&mut removed, 1);
    let mut refused = false;
    for _ in 0..6000 {
        let before = atomic_roots(&removed.core);
        match removed.core.advance_active_battle_to_next_event_bucket() {
            Ok(_) => {}
            Err(GameError::ValueExpressionFault(fault)) => {
                assert_eq!(
                    fault.code,
                    crate::game::value_expr::ValueExprFaultCode::MissingContext
                );
                assert_eq!(atomic_roots(&removed.core), before);
                assert!(!removed
                    .core
                    .state
                    .active_battle
                    .as_ref()
                    .unwrap()
                    .battle
                    .units[&removed.actors[1]]
                    .is_active());
                println!("U08_LIVE_SOURCE_REFUSAL withdrawn={withdrawn} alive_expected={expected} fault={fault:?}");
                refused = true;
                break;
            }
            Err(other) => panic!("expected MissingContext at released impact, got {other:?}"),
        }
    }
    assert!(refused);
    assert_observation_off(&removed.core);
}

fn queued_basic(fixture: &mut Fixture, index: usize, kind: &str) -> (u64, String) {
    for _ in 0..6000 {
        let battle = &fixture.core.state.active_battle.as_ref().unwrap().battle;
        if let Some(row) = battle.event_queue_debug_snapshot().into_iter().find(|row| {
            row.contains("AttackResolve {")
                && row.contains(&format!(
                    "attacker_instance_id: {:?}",
                    fixture.actors[index]
                ))
                && row.contains(&format!("kind: {kind}"))
        }) {
            let end = row
                .split("AttackResolve { time_ms: ")
                .nth(1)
                .unwrap()
                .split(',')
                .next()
                .unwrap()
                .parse()
                .unwrap();
            return (end, row);
        }
        fixture
            .core
            .advance_active_battle_to_next_event_bucket()
            .unwrap();
    }
    panic!("normal scheduler did not queue {kind} basic snapshot");
}

fn next_basic_hit(
    fixture: &mut Fixture,
    index: usize,
    after: u64,
    authored: Option<&str>,
) -> Arc<DamageFact> {
    for _ in 0..6000 {
        if let Some(hit) = latest_hit(fixture, index).filter(|hit| {
            hit.ordinal > after
                && hit
                    .source
                    .relic_numeric()
                    .is_some_and(|binding| binding.basic_profile_fingerprint.is_some())
                && authored.is_none_or(|skill| {
                    hit.source
                        .authored
                        .as_ref()
                        .is_some_and(|source| source.site.action.definition_id.as_str() == skill)
                })
        }) {
            return hit;
        }
        fixture
            .core
            .advance_active_battle_to_next_event_bucket()
            .unwrap();
    }
    panic!("normal scheduler did not consume basic snapshot");
}

fn assert_basic_hit(fixture: &Fixture, index: usize, base: u32, hit: &DamageFact) {
    let expected = if index == 0 {
        (base + 20) * 120 / 100
    } else {
        (base + 40) * 150 / 100
    };
    assert_eq!(hit.requested_damage, expected);
    assert!(hit.hp_before > expected);
    assert_eq!(hit.actual_hp_loss(), expected);
    let binding = hit.source.relic_numeric().unwrap();
    assert_eq!(binding.actor, fixture.actors[index]);
    assert_eq!(binding.employee_uuid, fixture.employees[index]);
    assert!(binding.basic_profile_fingerprint.is_some());
    assert!(
        binding.fragment.is_none(),
        "basic selector does not inherit skill fragment conditions"
    );
    assert_eq!(binding.contributions.len(), 1);
    assert_eq!(binding.contributions[0].copy_uuid, fixture.relics[index]);
    assert_eq!(
        fixture
            .core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .units[&fixture.actors[index]]
            .stats
            .attack,
        base
    );
    assert_observation_off(&fixture.core);
}

#[test]
fn basic_auto_and_authored_extra_melee_projectile_use_final_amount_once_and_keep_raw_attack() {
    use crate::game::battle::event_log::{AttackDelivery, AttackKind, BattleLogEvent};
    const EXTRA: &str = "u08_carrier_extra";
    for ranged in [false, true] {
        let mut fixture = fixture_with_basic(EXTRA, ranged, 100);
        for index in 0..2 {
            let base = fixture
                .core
                .state
                .active_battle
                .as_ref()
                .unwrap()
                .battle
                .units[&fixture.actors[index]]
                .stats
                .attack;
            let expected = if index == 0 {
                (base + 20) * 120 / 100
            } else {
                (base + 40) * 150 / 100
            };
            let before = latest_hit(&fixture, index).map_or(0, |hit| hit.ordinal);
            let (_, snapshot) = queued_basic(&mut fixture, index, "Auto");
            assert!(
                snapshot.contains(&format!("source_attack: {base},")),
                "{snapshot}"
            );
            assert!(
                snapshot.contains(&format!("base_damage: {expected},")),
                "{snapshot}"
            );
            let auto = next_basic_hit(&mut fixture, index, before, None);
            assert_basic_hit(&fixture, index, base, &auto);
            cast(&mut fixture, index, EXTRA);
            let (resolve_time, snapshot) = queued_basic(&mut fixture, index, "Triggered");
            assert!(
                snapshot.contains(&format!("source_attack: {base},")),
                "{snapshot}"
            );
            assert!(
                snapshot.contains(&format!("base_damage: {expected},")),
                "{snapshot}"
            );
            let extra = next_basic_hit(&mut fixture, index, auto.ordinal, Some(EXTRA));
            assert_basic_hit(&fixture, index, base, &extra);
            assert!(fixture.core.state.active_battle.as_ref().unwrap().battle.event_log.entries.iter().any(|entry| entry.time_ms == resolve_time && matches!(entry.event,
                BattleLogEvent::AttackResolve { attacker_instance_id, kind: Some(AttackKind::Triggered), delivery: Some(delivery), .. }
                    if attacker_instance_id == fixture.actors[index] && delivery == if ranged { AttackDelivery::Projectile } else { AttackDelivery::Instant }
            )));
            println!("U08_BASIC ranged={ranged} actor={index} raw_attack={base} expected={expected} auto={} triggered={} queued={snapshot}", serde_json::to_string(&auto).unwrap(), serde_json::to_string(&extra).unwrap());
        }
    }
}

fn observed_basic_epoch(fixture: &Fixture, index: usize) -> u64 {
    // Existing read-only Debug views expose scheduler capture/epoch state;
    // this test never changes either queue entries or runtime unit fields.
    let unit = &fixture
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .units[&fixture.actors[index]];
    format!("{unit:?}")
        .split("basic_attack_epoch: ")
        .nth(1)
        .unwrap()
        .split(',')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
fn basic_stale_auto_epoch_after_expired_normal_channel_cancels_while_actor_stays_active() {
    use crate::game::battle::event_log::{AttackKind, BattleLogEvent};
    const EPOCH: &str = "u08_carrier_epoch";
    let mut controlled = fixture_with_basic(EPOCH, false, 1000);
    ready_target(&mut controlled, 0);
    let base = controlled
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .units[&controlled.actors[0]]
        .stats
        .attack;
    let first = latest_hit(&controlled, 0).unwrap();
    assert_basic_hit(&controlled, 0, base, &first);
    let (resolve_time, queued) = queued_basic(&mut controlled, 0, "Auto");
    let old_epoch = observed_basic_epoch(&controlled, 0);
    assert!(queued.contains(&format!("attack_epoch: {old_epoch},")));
    assert!(time(&controlled.core) < resolve_time);
    let before = damage_count(&controlled, 0);
    let mut untreated = controlled.clone();
    execute_current_battle_mutation(
        &mut controlled.core,
        controlled.player,
        PlayerBehavior::ActivateSkill {
            employee_uuid: controlled.employees[0],
            skill_id: EPOCH.into(),
            target: None,
        },
    )
    .unwrap();
    for _ in 0..6000 {
        if observed_basic_epoch(&controlled, 0) != old_epoch {
            break;
        }
        assert!(
            time(&controlled.core) < resolve_time,
            "normal channel must start during the original windup"
        );
        controlled
            .core
            .advance_active_battle_to_next_event_bucket()
            .unwrap();
    }
    let new_epoch = observed_basic_epoch(&controlled, 0);
    assert_ne!(new_epoch, old_epoch);
    assert!(time(&controlled.core) < resolve_time);
    assert!(controlled
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .units[&controlled.actors[0]]
        .is_active());
    assert!(
        controlled
            .core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .event_queue_debug_snapshot()
            .contains(&queued),
        "the stale Auto event remains queued for the epoch guard"
    );
    let channels = controlled
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .sustained_effect_snapshot()
        .unwrap();
    let channel = channels
        .active
        .iter()
        .find(|effect| effect.caster == controlled.actors[0] && effect.is_channel)
        .unwrap();
    let activation = channel.activation;
    let expires_at = channel.expires_at_ms;
    assert!(expires_at < resolve_time);
    until(&mut controlled, expires_at);
    assert!(time(&controlled.core) < resolve_time);
    let effects = controlled
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .sustained_effect_snapshot()
        .unwrap();
    assert!(!effects
        .active
        .iter()
        .any(|effect| effect.caster == controlled.actors[0] && effect.is_channel));
    let ended = effects.latest_end.unwrap();
    assert_eq!(ended.activation, activation);
    assert_eq!(ended.time_ms, expires_at);
    assert_eq!(
        ended.reason,
        crate::game::ability::sustained::SustainedEndReason::Expired
    );
    assert!(
        controlled
            .core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .event_queue_debug_snapshot()
            .contains(&queued),
        "original captured Auto snapshot survives channel expiry unchanged"
    );
    until(&mut controlled, resolve_time);
    until(&mut untreated, resolve_time);
    assert!(controlled
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .units[&controlled.actors[0]]
        .is_active());
    assert_eq!(damage_count(&controlled, 0), before);
    assert_eq!(damage_count(&untreated, 0), before + 1);
    let positive = latest_hit(&untreated, 0).unwrap();
    assert_eq!(positive.time_ms, resolve_time);
    assert_basic_hit(&untreated, 0, base, &positive);
    let has_resolve = |fixture: &Fixture| {
        fixture.core.state.active_battle.as_ref().unwrap().battle.event_log.entries.iter().any(|entry| entry.time_ms == resolve_time && matches!(entry.event,
        BattleLogEvent::AttackResolve { attacker_instance_id, kind: Some(AttackKind::Auto), .. } if attacker_instance_id == fixture.actors[0]))
    };
    assert!(!has_resolve(&controlled));
    assert!(has_resolve(&untreated));
    assert_observation_off(&controlled.core);
    println!("U08_BASIC_EPOCH old={old_epoch} current={new_epoch} channel_expired={expires_at} old_resolve={resolve_time} before_damage_count={before} controlled_count={} untreated_count={} positive={}", damage_count(&controlled, 0), damage_count(&untreated, 0), serde_json::to_string(&positive).unwrap());
}
