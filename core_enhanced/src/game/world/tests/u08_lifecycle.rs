//! Synthetic battle inputs exercise the real deployment and result routes.
//! No terminal result, death, slot recovery, or relic runtime owner is injected.
use super::*;
use crate::game::{
    battle::{
        damage::DamageType, ids::UnitInstanceId, runtime_facts::DamageFact,
        tile_range::TileRangePattern, types::BattleVerdict,
    },
    behavior::ActionDisabledReasonCode,
    combat_preview::DeploymentZoneKind,
    data::relic_data::RelicDatabase,
    relic::RelicVariant,
    world::AdminCommand,
};

const TIMER_MS: u64 = 50_000;

#[test]
fn actual_retreat_restores_nonempty_entry_relics_and_materials_then_reentry_consumes_fresh_binding()
{
    let (mut core, player, employee, guard, copies) = setup(false);
    let entry_relics = relic_roots(&core);
    let entry_roster = serde_json::to_value(&core.state.roster).unwrap();
    let definitions = serde_json::to_value(&core.game_data.relic_data).unwrap();
    let certified = core.certified_input_fingerprint().unwrap();
    let (position, actor, guard_actor) = enter(&mut core, player, employee, guard);
    let guard_position = core.state.active_battle.as_ref().unwrap().battle.units[&guard_actor]
        .body
        .projected_tile();
    let node = core.state.active_battle.as_ref().unwrap().node_id;
    let before_attempt = core.run_state().unwrap().abnormality_attempt_state(node);
    assert_eq!(before_attempt.retreats_used, 0);
    // ConfirmEnterNode already called start_abnormality_attempt.
    assert_eq!(before_attempt.attempts_started, 1);
    let hit = next_hit(&mut core, actor);
    assert_hit(&core, employee, actor, copies, &hit);
    assert_eq!(hit.actual_hp_loss(), 174);
    let old_fact = serde_json::to_value(&hit).unwrap();
    let progressed_time = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .execution
        .last_event_time_ms();
    assert!(progressed_time > 0);
    assert!(hit.hp_after < hit.hp_before);
    let retreated =
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).unwrap();
    let BehaviorResult::NodePreview {
        node_id,
        combat_preview: Some(_),
        ..
    } = retreated
    else {
        panic!("real retreat must restore preview")
    };
    assert_eq!(node_id, node);
    assert!(matches!(core.get_state(), GameState::NodeConfirm { node_id, .. } if node_id == node));
    assert!(core.state.active_battle.is_none());
    assert_eq!(relic_roots(&core), entry_relics);
    assert_slots(&core, employee, copies);
    assert_eq!(core.state.crafting_resources.relic_corruption_catalysts, 2);
    assert_eq!(
        serde_json::to_value(&core.state.roster).unwrap(),
        entry_roster
    );
    let attempt = core.run_state().unwrap().abnormality_attempt_state(node);
    assert_eq!(attempt.retreats_used, 1);
    assert_eq!(attempt.attempts_started, 1);
    assert_eq!(attempt.max_attempts, before_attempt.max_attempts);
    assert_eq!(core.certified_input_fingerprint().unwrap(), certified);
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.node_id, node);
    assert_eq!(active.execution.last_event_time_ms(), 0);
    assert_eq!(
        core.run_state()
            .unwrap()
            .abnormality_attempt_state(node)
            .attempts_started,
        2
    );
    let redeployed = deploy(&mut core, player, employee, position).unwrap();
    deploy(&mut core, player, guard, guard_position).unwrap();
    let fresh = next_hit(&mut core, redeployed);
    assert_hit(&core, employee, redeployed, copies, &fresh);
    assert_eq!(fresh.actual_hp_loss(), 174);
    assert_eq!(
        fresh.hp_before, hit.hp_before,
        "new attempt must restore enemy HP before the same first hit"
    );
    assert!(!Arc::ptr_eq(
        hit.source.relic_numeric().unwrap(),
        fresh.source.relic_numeric().unwrap()
    ));
    assert_eq!(serde_json::to_value(&hit).unwrap(), old_fact);
    assert_eq!(relic_roots(&core), entry_relics);
    assert_eq!(
        serde_json::to_value(&core.game_data.relic_data).unwrap(),
        definitions
    );
    println!("U08_RETREAT_ENTRY_RESTORE time_before={progressed_time} retreat_count=1 first_hp_loss={} reentry_hp_loss={}",hit.actual_hp_loss(),fresh.actual_hp_loss());
}

fn wide_range() -> TileRangePattern {
    let mut rows = vec!["XXXXXXXXXXXXXXX"; 15];
    rows[7] = "XXXXXXX@XXXXXXX";
    serde_json::from_value(json!({"include_anchor_tile": false, "rows": rows})).unwrap()
}

fn setup(lethal_enemy: bool) -> (GameCore, Uuid, Uuid, Uuid, [Uuid; 3]) {
    let base = GameCore::new(game_data_with_pve_encounters(), 0x808_D2).game_data;
    let mut enemies = base.abnormality_data.items.clone();
    let enemy = enemies
        .iter_mut()
        .find(|row| row.id == "defense_risk_abno")
        .unwrap();
    enemy.max_health = 1_000_000;
    enemy.attack = if lethal_enemy { 1_000 } else { 0 };
    enemy.basic_attack.damage_type = DamageType::True;
    enemy.basic_attack.defense_tile_range = Some(wide_range());
    enemy.basic_attack.interval_ms = 1_000;
    enemy.basic_attack.windup_ms = 10;
    if lethal_enemy {
        // A fixed-route melee enemy only attacks its blocker or route-end
        // target. This authored ranged profile instead selects the victim's
        // actual negative defense through ordinary targeting.
        enemy.basic_attack.range_role = WeaponRangeRole::Ranged;
        enemy.basic_attack.targeting_profile =
            crate::game::data::equipment_data::TargetingProfile::LowDefenseFirst;
    }
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
    weapon.interval_ms = 1_000;
    weapon.windup_ms = 10;
    let mut encounters = base.pve_data.encounters.clone();
    let encounter = encounters
        .iter_mut()
        .find(|row| row.id == "defense_encounter")
        .unwrap();
    encounter.terminal_outcomes = PveEncounter::fixture_terminal_outcomes(Some(TIMER_MS));
    for wave in &mut encounter.waves {
        wave.required_for_victory = false;
    }
    let mut relics: RelicDatabase = ron::de::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/u08/relics.ron"
    )))
    .unwrap();
    let template = relics.relics[0].clone();
    for index in 1..=2 {
        let mut definition = template.clone();
        definition.id = format!("u08_lifecycle_{index}");
        definition.uuid = Uuid::from_u128(0x808_100 + index);
        relics.relics.push(definition);
    }
    let data = GameDataBuilder::from_validated(&base)
        .with_abnormalities(enemies)
        .with_equipment(equipment)
        .with_pve(PveEncounterDatabase::new_with_reward_sources(
            encounters,
            base.pve_data.reward_sources.clone(),
        ))
        .with_relics(relics)
        .build_arc();
    let mut core = GameCore::new(data, 0x808_D2);
    let player = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player);
    let employees = core.state.roster.combat_eligible_employee_ids();
    let employee = employees[0];
    let guard = employees[1];
    // Pre-battle fixture inputs: a sturdy living guard and an ordinary victim.
    core.state
        .roster
        .get_mut(&guard)
        .unwrap()
        .profile
        .base_combat_stats
        .max_hp = 1_000_000;
    // Employees already have a captured combat profile. Set the same starting
    // fixture health there before battle creation; never patch runtime HP.
    let guard_stats = &mut core
        .state
        .roster
        .get_mut(&guard)
        .unwrap()
        .combat_profile
        .battle_profile
        .stats;
    guard_stats.max_health = 1_000_000;
    guard_stats.current_health = 1_000_000;
    let mut copies = [Uuid::nil(); 3];
    for (slot, definition) in ["u08_probe", "u08_lifecycle_1", "u08_lifecycle_2"]
        .into_iter()
        .enumerate()
    {
        core.execute_admin_command(AdminCommand::AdminGrantRelic {
            definition_id: definition.into(),
            count: 1,
        })
        .unwrap();
        copies[slot] = core
            .state
            .relics
            .copies()
            .find(|copy| copy.definition_id == definition)
            .unwrap()
            .copy_uuid;
        core.execute(
            player,
            PlayerBehavior::EquipRelic {
                copy_uuid: copies[slot],
                employee_uuid: employee,
                slot: slot as u8,
            },
        )
        .unwrap();
    }
    core.execute_admin_command(AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 3 })
        .unwrap();
    core.execute(
        player,
        PlayerBehavior::CorruptRelic {
            copy_uuid: copies[0],
        },
    )
    .unwrap();
    (core, player, employee, guard, copies)
}

fn deploy(
    core: &mut GameCore,
    player: Uuid,
    employee: Uuid,
    position: Position,
) -> Result<UnitInstanceId, GameError> {
    match execute_current_battle_mutation(
        core,
        player,
        PlayerBehavior::DeployUnit {
            employee_uuid: employee,
            position,
            facing: FacingDirection::Right,
        },
    )? {
        BehaviorResult::BattleUnitDeployed {
            unit_instance_id, ..
        } => Ok(unit_instance_id),
        other => panic!("expected real deployment: {other:?}"),
    }
}

fn enter(
    core: &mut GameCore,
    player: Uuid,
    employee: Uuid,
    guard: Uuid,
) -> (Position, UnitInstanceId, UnitInstanceId) {
    let node = force_first_available_node(
        core,
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
        panic!("combat preview");
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
    // The preview lists authored zones, including slots occupied by initial
    // allied defenders. Apply the actual player deployment occupancy contract
    // after battle creation; enemy overlap is deliberately legal in that API.
    cells.retain(|cell| {
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .player_tactical_slot_occupant(*cell, None)
            .is_none()
    });
    assert!(cells.len() >= 2);
    let position = cells[0];
    let guard_position = *cells.last().unwrap();
    let actor = deploy(core, player, employee, position).unwrap();
    let guard_actor = deploy(core, player, guard, guard_position).unwrap();
    (position, actor, guard_actor)
}

fn relic_roots(core: &GameCore) -> Value {
    json!({"relics": core.state.relics, "materials": core.state.crafting_resources})
}

fn assert_slots(core: &GameCore, employee: Uuid, copies: [Uuid; 3]) {
    assert_eq!(core.state.relics.employee_slots(employee), copies.map(Some));
    assert_eq!(
        core.state.relics.get(copies[0]).unwrap().variant,
        RelicVariant::Corrupt
    );
    for copy in &copies[1..] {
        assert_eq!(
            core.state.relics.get(*copy).unwrap().variant,
            RelicVariant::Normal
        );
    }
}

fn next_hit(core: &mut GameCore, actor: UnitInstanceId) -> Arc<DamageFact> {
    for _ in 0..6_000 {
        let active = core
            .state
            .active_battle
            .as_ref()
            .expect("hit must precede timer result");
        let facts = active.battle.runtime_fact_snapshot().unwrap();
        if let Some(hit) = facts
            .actors
            .iter()
            .find(|row| row.actor == actor)
            .and_then(|row| row.summary.caused.latest_damage.clone())
        {
            return hit;
        }
        core.advance_active_battle_to_next_event_bucket().unwrap();
    }
    panic!("deployed actor never produced a real damage fact");
}

fn assert_hit(
    core: &GameCore,
    employee: Uuid,
    actor: UnitInstanceId,
    copies: [Uuid; 3],
    hit: &DamageFact,
) {
    let unit = &core.state.active_battle.as_ref().unwrap().battle.units[&actor];
    // One Corrupt (40,+50%) and two Normal (20,+20% each), additive class.
    let expected = ((u64::from(unit.stats.attack) + 80) * 190 / 100) as u32;
    assert_eq!(hit.requested_damage, expected);
    assert_eq!(
        hit.actual_hp_loss(),
        expected,
        "true damage reaches the durable enemy"
    );
    let binding = hit
        .source
        .relic_numeric()
        .expect("real attack must bind equipped relics");
    assert_eq!(binding.employee_uuid, employee);
    assert_eq!(binding.actor, actor);
    assert_eq!(hit.source.actor, Some(actor));
    assert_eq!(binding.contributions.len(), 3);
    for copy in copies {
        let row = binding
            .contributions
            .iter()
            .find(|row| row.copy_uuid == copy)
            .unwrap();
        assert_eq!(row.variant, core.state.relics.get(copy).unwrap().variant);
        assert_eq!(
            (row.flat, row.percent),
            if copy == copies[0] {
                (40, 50)
            } else {
                (20, 20)
            }
        );
    }
}

#[test]
fn all_three_copy_slot_permutations_preserve_actual_damage_penalties_and_certified_base() {
    let (prepared, player, employee, guard, copies) = setup(false);
    let certified = prepared.certified_input_fingerprint().unwrap();
    let roster = serde_json::to_value(&prepared.state.roster).unwrap();
    let definitions = serde_json::to_vec(&prepared.game_data.relic_data).unwrap();
    let mut baseline = None;
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let mut core = prepared.clone();
        for copy in copies {
            core.execute(player, PlayerBehavior::UnequipRelic { copy_uuid: copy })
                .unwrap();
        }
        assert_eq!(core.certified_input_fingerprint().unwrap(), certified);
        for (slot, index) in order.into_iter().enumerate() {
            core.execute(
                player,
                PlayerBehavior::EquipRelic {
                    copy_uuid: copies[index],
                    employee_uuid: employee,
                    slot: slot as u8,
                },
            )
            .unwrap();
        }
        assert_eq!(
            core.state.relics.employee_slots(employee),
            order.map(|index| Some(copies[index]))
        );
        assert_eq!(core.certified_input_fingerprint().unwrap(), certified);
        assert_eq!(serde_json::to_value(&core.state.roster).unwrap(), roster);
        let BehaviorResult::RelicNumericPlan { result: plan } = core
            .execute(
                player,
                PlayerBehavior::RequestRelicNumericPlan {
                    employee_uuid: employee,
                },
            )
            .unwrap()
        else {
            panic!("numeric plan");
        };
        let base = plan.base_battle_stats.unwrap();
        let effective = plan.effective_battle_stats.unwrap();
        assert_eq!(effective.attack, base.attack);
        assert_eq!(effective.defense, base.defense - 15);
        assert_eq!(effective.magic_resist, base.magic_resist - 10);
        let (_, actor, _) = enter(&mut core, player, employee, guard);
        let hit = next_hit(&mut core, actor);
        assert_hit(&core, employee, actor, copies, &hit);
        let actual = &core.state.active_battle.as_ref().unwrap().battle.units[&actor].stats;
        assert_eq!(actual.attack, base.attack);
        assert_eq!(actual.defense, base.defense - 15);
        assert_eq!(actual.magic_resist, base.magic_resist - 10);
        let comparison = json!({"base":base, "effective":effective, "attack":actual.attack,
            "defense":actual.defense, "magic_resist":actual.magic_resist,
            "requested":hit.requested_damage, "hp_loss":hit.actual_hp_loss(),
            "contributions":hit.source.relic_numeric().unwrap().contributions});
        if let Some(expected) = &baseline {
            assert_eq!(&comparison, expected, "{order:?}");
        } else {
            baseline = Some(comparison);
        }
        assert_eq!(core.certified_input_fingerprint().unwrap(), certified);
        assert_eq!(serde_json::to_value(&core.state.roster).unwrap(), roster);
        assert_eq!(
            serde_json::to_vec(&core.game_data.relic_data).unwrap(),
            definitions
        );
        println!(
            "U08_SLOT_PERMUTATION order={order:?} requested={} hp_loss={}",
            hit.requested_damage,
            hit.actual_hp_loss()
        );
    }
}

#[test]
fn living_withdraw_and_real_cooldown_redeploy_keep_three_slots_and_rebind_actual_damage() {
    let (mut core, player, employee, guard, copies) = setup(false);
    let BehaviorResult::RelicNumericPlan { result: plan } = core
        .execute(
            player,
            PlayerBehavior::RequestRelicNumericPlan {
                employee_uuid: employee,
            },
        )
        .unwrap()
    else {
        panic!("numeric plan");
    };
    let base_stats = plan.base_battle_stats.unwrap();
    let effective_stats = plan.effective_battle_stats.unwrap();
    assert_eq!(effective_stats.defense, base_stats.defense - 15);
    assert_eq!(effective_stats.magic_resist, base_stats.magic_resist - 10);
    let before = relic_roots(&core);
    let (position, actor, _) = enter(&mut core, player, employee, guard);
    let hit = next_hit(&mut core, actor);
    assert_hit(&core, employee, actor, copies, &hit);
    let stats = core.state.active_battle.as_ref().unwrap().battle.units[&actor]
        .stats
        .clone();
    assert_eq!(stats.defense, effective_stats.defense);
    assert_eq!(stats.magic_resist, effective_stats.magic_resist);
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: employee,
        },
    )
    .unwrap();
    assert_slots(&core, employee, copies);
    assert_eq!(relic_roots(&core), before);
    let ready_at_ms = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .live_deployment
        .as_ref()
        .unwrap()
        .redeploy_locks[&employee]
        .ready_at_ms;
    let withdrawn_time = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .execution
        .last_event_time_ms();
    assert!(ready_at_ms > withdrawn_time);
    let mut redeployed = None;
    let mut refusals = 0;
    for _ in 0..6_000 {
        match deploy(&mut core, player, employee, position) {
            Ok(actor) => {
                redeployed = Some(actor);
                break;
            }
            Err(GameError::ActionUnavailable {
                reason: ActionDisabledReasonCode::RedeployCooldown,
                ..
            }) => {
                refusals += 1;
                assert_eq!(relic_roots(&core), before);
                core.advance_active_battle_to_next_event_bucket().unwrap();
            }
            Err(error) => panic!("unexpected redeploy refusal: {error:?}"),
        }
    }
    let new_actor = redeployed.expect("real clock must unlock redeployment before timer Victory");
    assert!(refusals > 0);
    assert_ne!(new_actor, actor);
    assert!(
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .execution
            .last_event_time_ms()
            >= ready_at_ms
    );
    let new_stats = &core.state.active_battle.as_ref().unwrap().battle.units[&new_actor].stats;
    assert_eq!(new_stats.defense, stats.defense);
    assert_eq!(new_stats.magic_resist, stats.magic_resist);
    let new_hit = next_hit(&mut core, new_actor);
    assert_hit(&core, employee, new_actor, copies, &new_hit);
    assert_ne!(
        hit.source.relic_numeric().unwrap().actor,
        new_hit.source.relic_numeric().unwrap().actor
    );
    assert_slots(&core, employee, copies);
    assert_eq!(relic_roots(&core), before);
}

#[test]
fn real_lethal_hit_then_terminal_settlement_clears_slots_and_complete_only_acknowledges_once() {
    let (mut core, player, employee, guard, copies) = setup(true);
    let before = relic_roots(&core);
    let owned_before: Vec<_> = core.state.relics.copies().cloned().collect();
    let (_, actor, guard_actor) = enter(&mut core, player, employee, guard);
    // Only the attempt's starting condition is injected, after legal deployment.
    // 5 is Critical physical damage; actual incapacitation must add 1 to reach 6.
    // Trauma is independent (100 means Breakdown, not physical death).
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .employee_attempt
        .roster
        .get_mut(&employee)
        .unwrap()
        .condition
        .physical_damage = MAX_PHYSICAL_DAMAGE - 1;
    assert!(!core
        .state
        .roster
        .get(&employee)
        .unwrap()
        .condition
        .is_dead());
    let mut lethal = None;
    for _ in 0..6_000 {
        core.advance_active_battle_to_next_event_bucket().unwrap();
        let active = core.state.active_battle.as_ref().unwrap_or_else(|| {
            panic!(
                "actual death must precede terminal result; state={:?}; result={:?}",
                core.get_state(),
                core.state
                    .confirmed_combat_result
                    .as_ref()
                    .map(|receipt| &receipt.summary)
            );
        });
        if active
            .employee_attempt
            .roster
            .get(&employee)
            .unwrap()
            .condition
            .is_dead()
        {
            let facts = active.battle.runtime_fact_snapshot().unwrap();
            let received = &facts
                .actors
                .iter()
                .find(|row| row.actor == actor)
                .unwrap()
                .summary
                .received;
            lethal = received.latest_damage.clone();
            assert_eq!(
                active
                    .employee_attempt
                    .roster
                    .get(&employee)
                    .unwrap()
                    .condition
                    .physical_damage,
                6
            );
            assert!(active.battle.units[&guard_actor].is_active());
            break;
        }
    }
    let lethal = lethal.expect("enemy hit must produce the real new death");
    assert_eq!(lethal.target, actor);
    assert!(lethal.hp_before > 0);
    assert_eq!(lethal.hp_after, 0);
    assert!(lethal.actual_hp_loss() > 0);
    assert_ne!(lethal.source.actor, Some(actor));
    let attacker = lethal
        .source
        .actor
        .expect("lethal damage has an actual enemy actor");
    assert_eq!(
        core.state.active_battle.as_ref().unwrap().battle.units[&attacker].owner,
        crate::game::enums::Side::Opponent
    );
    assert!(
        !core
            .state
            .roster
            .get(&employee)
            .unwrap()
            .condition
            .is_dead(),
        "persistent roster remains pre-result"
    );
    assert_slots(&core, employee, copies);
    assert_eq!(relic_roots(&core), before);
    for _ in 0..6_000 {
        if core.state.active_battle.is_none() {
            break;
        }
        assert!(!core
            .state
            .roster
            .get(&employee)
            .unwrap()
            .condition
            .is_dead());
        assert_slots(&core, employee, copies);
        core.advance_active_battle_to_next_event_bucket().unwrap();
    }
    assert!(
        core.state.active_battle.is_none(),
        "normal clock must reach terminal Victory"
    );
    assert!(matches!(core.get_state(), GameState::CombatResult { .. }));
    let receipt = core.state.confirmed_combat_result.as_ref().unwrap();
    let crate::game::behavior::SelectedEventSnapshotDto::CombatBattle {
        verdict,
        result_stats,
        ..
    } = &receipt.summary
    else {
        panic!("confirmed combat result summary");
    };
    assert_eq!(*verdict, BattleVerdict::Victory);
    assert!(result_stats
        .employees
        .iter()
        .any(|row| row.employee_uuid == guard && row.survived));
    assert!(result_stats
        .employees
        .iter()
        .any(|row| row.employee_uuid == employee && !row.survived && row.became_incapacitated));
    // The terminal application route already settled the result and reconciled
    // relic slots. CompleteCombatResult acknowledges its immutable receipt.
    assert!(core
        .state
        .roster
        .get(&employee)
        .unwrap()
        .condition
        .is_dead());
    assert_eq!(core.state.relics.employee_slots(employee), [None; 3]);
    let settled_roots = relic_roots(&core);
    let settled_roster = serde_json::to_value(&core.state.roster).unwrap();
    core.execute(player, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert!(core.state.confirmed_combat_result.is_none());
    assert_eq!(relic_roots(&core), settled_roots);
    assert_eq!(
        serde_json::to_value(&core.state.roster).unwrap(),
        settled_roster
    );
    assert!(core
        .state
        .roster
        .get(&employee)
        .unwrap()
        .condition
        .is_dead());
    assert!(core.state.roster.get(&guard).unwrap().is_alive());
    assert!(
        core.state.run.is_some(),
        "surviving roster keeps this run active"
    );
    assert_eq!(core.state.relics.employee_slots(employee), [None; 3]);
    assert_eq!(core.state.relics.employee_slots(guard), [None; 3]);
    // Compare the actual shared copies and materials separately from slot removal.
    let after = relic_roots(&core);
    assert_eq!(after["materials"], before["materials"]);
    for copy in copies {
        let retained = core
            .state
            .relics
            .get(copy)
            .expect("dead employee's copy stays in shared inventory");
        assert_eq!(
            retained.variant,
            if copy == copies[0] {
                RelicVariant::Corrupt
            } else {
                RelicVariant::Normal
            }
        );
    }
    assert_eq!(
        core.state.relics.copies().cloned().collect::<Vec<_>>(),
        owned_before
    );
    let committed = core.get_run_snapshot_json().unwrap();
    let roots = relic_roots(&core);
    assert!(core
        .execute(player, PlayerBehavior::CompleteCombatResult)
        .is_err());
    assert_eq!(core.get_run_snapshot_json().unwrap(), committed);
    assert_eq!(relic_roots(&core), roots);
}
