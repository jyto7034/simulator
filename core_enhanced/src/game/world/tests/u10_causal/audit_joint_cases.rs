//! Joint producer/consumer evidence for U10-AUD-F01 and F02.
//! Relics, copies, effective plans, fragments and deployment are normal World
//! inputs. Stationary statistics and runtime equipment placement are explicit
//! native preparation. Only the additional Death/AllyDeath comparison changes
//! two prepared employee sides; it is not a public PvE configuration claim.
use super::*;
use crate::game::{
    ability::{DeliveryDef, ProjectileHitPolicy},
    battle::{
        core::{relic_causal_tests as native, BattleCore},
        damage::{DamageSource, DamageType},
        event_log::BattleLogEvent,
        runtime_facts::DeathCause,
    },
    data::equipment_data::EquipmentMetadata,
};

const COUNTER_GEAR: u128 = 0xda010;
const DEATH_GEAR: u128 = 0xda020;

struct JointWorld {
    core: GameCore,
    player: Uuid,
    employees: Vec<Uuid>,
    actors: Vec<UnitInstanceId>,
    enemies: Vec<UnitInstanceId>,
    copies: Vec<Uuid>,
    optional_programs: std::collections::BTreeMap<String, String>,
}

impl JointWorld {
    fn battle(&self) -> &BattleCore {
        &self.core.state.active_battle.as_ref().unwrap().battle
    }
    fn battle_mut(&mut self) -> &mut BattleCore {
        &mut self.core.state.active_battle.as_mut().unwrap().battle
    }
    fn advance(&mut self, delta: u64) {
        self.core
            .advance_active_battle_for_server_tick(delta)
            .unwrap();
    }
}

fn hp(core: &GameCore, id: UnitInstanceId) -> u32 {
    core.state.active_battle.as_ref().unwrap().battle.units[&id]
        .stats
        .current_health
}

fn counter_gear() -> EquipmentMetadata {
    ron::de::from_str(r#"EquipmentMetadata(
        id:"u10_joint_counter",uuid:"00000000-0000-0000-0000-0000000da010",
        name:"joint counter",equipment_type:Accessory,rarity:ZAYIN,price:0,
        triggered_effects:{
            OnAttack:[(target:SelfUnit,effect:Modifier((stat:AttackIntervalMs,kind:Flat,value:17)))],
            OnHit:[
                (target:SelfUnit,effect:Modifier((stat:AttackIntervalMs,kind:Flat,value:23))),
                (target:CounterpartUnit,effect:BonusDamage(flat:8,percent:0,damage_type:True)),
                (target:CounterpartUnit,effect:BonusDamage(flat:0,percent:50,damage_type:True)),
            ],
        })"#).unwrap()
}

fn death_gear() -> EquipmentMetadata {
    ron::de::from_str(
        r#"EquipmentMetadata(
        id:"u10_joint_death",uuid:"00000000-0000-0000-0000-0000000da020",
        name:"joint death reactions",equipment_type:Accessory,rarity:ZAYIN,price:0,
        triggered_effects:{
            OnDeath:[(target:AllEnemies,effect:Heal(flat:2,percent:0))],
            OnKill:[(target:AllAllies,effect:Heal(flat:3,percent:0))],
            OnAllyDeath:[(target:AllEnemies,effect:Heal(flat:5,percent:0))],
        })"#,
    )
    .unwrap()
}

/// A separate fixture keeps the old 41 cases' builder/helper bytes untouched.
fn prepare_joint(rules: [&str; 3], gear: EquipmentMetadata, fragment: bool) -> JointWorld {
    let skills =
        SkillDatabase::from_ron("u10/joint_skills.ron", &skill_input(&[(0, &[1])])).unwrap();
    let fragments = if fragment {
        vec![active_skill_fragment("u10_fragment", 0xda001, "u10_strike")]
    } else {
        vec![]
    };
    let base = GameCore::new(
        game_data_with_pve_equipment_and_active_skill_fragments(
            vec![gear],
            skills.skills.clone(),
            fragments,
        ),
        0xda002,
    );
    let mut enemies = base.game_data.abnormality_data.items.clone();
    for enemy in &mut enemies {
        enemy.max_health = 10_000;
        enemy.attack = 0;
        enemy.defense = 0;
        enemy.magic_resist = 0;
    }
    let relic_rows = rules
        .iter()
        .enumerate()
        .filter(|(_, rules)| !rules.is_empty())
        .map(|(index, rules)| {
            format!(
                r#"(
        id:"joint_{index}",uuid:"00000000-0000-0000-0000-{:012x}",
        name:"joint {index}",description:"independent joint input",rarity:"probe",
        allow_duplicates:true,effects:[],dismantle_equipment_dust:1,rules:[{rules}])"#,
                0xda100 + index
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let relics: RelicDatabase = ron::de::from_str(&format!("(relics:[{relic_rows}])")).unwrap();
    let mut pve = (*base.game_data.pve_data).clone();
    let encounter = pve
        .encounters
        .iter_mut()
        .find(|entry| entry.id == "defense_encounter")
        .unwrap();
    // A future required wave keeps the public Corrosion comparison observable
    // after the current enemies disappear and the actor has withdrawn.
    let mut future = encounter.waves[0].clone();
    future.id = "joint_future".into();
    future.activation =
        crate::game::data::pve_data::PveWaveActivationData::at_time("joint_future", 500);
    encounter.waves.push(future);
    let buffs = ron::de::from_str(
        r#"BuffDatabase(buffs:[(name:"joint_released_dot",
        kind:PeriodicDamage(damage_per_tick:19,damage_type:True),tick_interval_ms:40,
        max_stacks:1,reapply_policy:RefreshDuration)])"#,
    )
    .unwrap();
    let data = GameDataBuilder::from_validated(&base.game_data)
        .with_skills(skills)
        .with_relics(relics)
        .with_abnormalities(enemies)
        .with_pve(pve)
        .with_buffs(buffs)
        .build_with_runtime_safety(crate::runtime_safety::RuntimeSafetySettings::embedded_shared())
        .expect("normal complete joint catalog publication");
    let mut core = GameCore::new(Arc::new(data), 0xda002);
    let player = Uuid::from_u128(0xda002);
    start_new_game_with_default_starters(&mut core, player);
    let employees = core.roster().unwrap().combat_eligible_employee_ids();
    assert_eq!(employees.len(), 3);
    if fragment {
        let serum = consumable_meta(
            0xda003,
            "joint_initial_charge",
            ConsumableTier::Critical,
            ConsumableEffect::InitialSkillCharge { percent: 100 },
        );
        let employee = core.roster_mut().unwrap().get_mut(&employees[0]).unwrap();
        employee.condition.trauma = 80;
        employee.apply_consumable_modifier(Uuid::from_u128(0xda004), &serum);
        let copy = core
            .apply_grant_effects(&[RewardEffect::GrantSkillFragment {
                fragment_id: "u10_fragment".into(),
            }])
            .unwrap()
            .skill_fragment_diffs[0]
            .copy_uuid;
        core.execute(
            player,
            PlayerBehavior::EquipSkillFragment {
                employee_uuid: employees[0],
                copy_uuid: copy,
            },
        )
        .unwrap();
    }
    let mut copies = Vec::new();
    for (index, employee) in employees.iter().enumerate() {
        // Basic-only cases publish/equip just the participating owner's relic.
        // An empty role is an ordinary employee, not an invalid empty relic.
        if rules[index].is_empty() {
            continue;
        }
        let before: BTreeSet<_> = core
            .state
            .relics
            .copies()
            .map(|copy| copy.copy_uuid)
            .collect();
        core.execute_admin_command(super::super::super::AdminCommand::AdminGrantRelic {
            definition_id: format!("joint_{index}"),
            count: 1,
        })
        .unwrap();
        let added = core
            .state
            .relics
            .copies()
            .find(|copy| !before.contains(&copy.copy_uuid))
            .unwrap()
            .copy_uuid;
        core.execute(
            player,
            PlayerBehavior::EquipRelic {
                employee_uuid: *employee,
                copy_uuid: added,
                slot: 0,
            },
        )
        .unwrap();
        copies.push(added);
    }
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        MapNodePayload::Encounter {
            encounter_id: Some("defense_encounter".into()),
        },
    );
    let preview = core
        .execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    let BehaviorResult::NodePreview {
        combat_preview: Some(preview),
        ..
    } = preview
    else {
        panic!("normal preview")
    };
    let positions = preview
        .deployment_zones
        .iter()
        .find(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
        .unwrap()
        .cells
        .clone();
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    for (index, employee) in employees.iter().enumerate() {
        core.state
            .active_battle
            .as_mut()
            .unwrap()
            .tactical_resource
            .set_current_for_test(100);
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::DeployUnit {
                employee_uuid: *employee,
                position: positions[index],
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
    }
    let battle = &mut core.state.active_battle.as_mut().unwrap().battle;
    let actors = employees
        .iter()
        .map(|employee| {
            battle
                .units
                .values()
                .find(|unit| unit.source_owned_uuid == *employee)
                .unwrap()
                .instance_id
        })
        .collect::<Vec<_>>();
    let mut enemy_ids = battle
        .units
        .values()
        .filter(|unit| unit.owner == Side::Opponent)
        .map(|unit| unit.instance_id)
        .collect::<Vec<_>>();
    enemy_ids.sort();
    assert!(!enemy_ids.is_empty());
    // The generated ChokePoint has real static obstacles. Pick walkable
    // adjacent tiles for the basic producer/defender and distinct walkable
    // tiles for other actors, without altering the published battlefield.
    let walkable = (0..battle.battlefield.height())
        .flat_map(|y| {
            (0..battle.battlefield.width()).map(move |x| Position::new(x as i32, y as i32))
        })
        .filter(|position| battle.battlefield.is_walkable_tile(*position))
        .collect::<Vec<_>>();
    let (actor_position, defender_position) = walkable
        .iter()
        .find_map(|position| {
            let right = Position::new(position.x + 1, position.y);
            battle
                .battlefield
                .is_walkable_tile(right)
                .then_some((*position, right))
        })
        .expect("published battlefield has an adjacent walkable pair");
    let mut remaining_positions = walkable
        .into_iter()
        .filter(|position| *position != actor_position && *position != defender_position);
    for (index, actor) in actors.iter().enumerate() {
        let unit = battle.units.get_mut(actor).unwrap();
        unit.stats.max_health = 10_000;
        unit.stats.current_health = 4_000;
        unit.stats.attack = 0;
        unit.stats.defense = 0;
        unit.stats.magic_resist = 0;
        unit.stats.attack_interval_ms = 2_000;
        unit.basic_attack.damage_type = DamageType::True;
        let position = if index == 0 {
            actor_position
        } else {
            remaining_positions.next().unwrap()
        };
        existing_native_probe::place_stationary(battle, *actor, position.x, position.y);
    }
    for (index, enemy) in enemy_ids.iter().enumerate() {
        battle
            .units
            .get_mut(enemy)
            .unwrap()
            .stats
            .attack_interval_ms = 2_000;
        let position = if index == 0 {
            defender_position
        } else {
            remaining_positions.next().unwrap()
        };
        existing_native_probe::place_stationary(battle, *enemy, position.x, position.y);
    }
    assert!(
        native::joint_causal_owner_present(battle),
        "published active rules allocate the real causal owner"
    );
    let owner = core.capture_structural_battle_owner().unwrap().unwrap();
    let optional_programs = owner
        .plans
        .values()
        .flat_map(|plan| plan.causal_rules.iter())
        .filter(|rule| {
            matches!(
                rule.contribution.rule_id.as_str(),
                "joint_death" | "joint_kill" | "joint_ally_death"
            )
        })
        .flat_map(|rule| {
            rule.programs.iter().map(|program| {
                (
                    program.execution.skill.id.as_str().to_owned(),
                    rule.contribution.rule_id.clone(),
                )
            })
        })
        .collect();
    JointWorld {
        core,
        player,
        employees,
        actors,
        enemies: enemy_ids,
        copies,
        optional_programs,
    }
}

fn damages(
    battle: &BattleCore,
) -> Vec<(
    Option<UnitInstanceId>,
    UnitInstanceId,
    i32,
    Option<DamageSource>,
)> {
    battle
        .event_log
        .entries
        .iter()
        .filter_map(|entry| match entry.event {
            BattleLogEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                delta,
                damage_source,
                ..
            } if delta < 0 => Some((source_instance_id, target_instance_id, delta, damage_source)),
            _ => None,
        })
        .collect()
}

#[test]
fn u10_audit_f01_real_basic_counter_and_published_subscriber_share_one_battle() {
    let rule = damage_subscriber("basic_echo", 7, 10).replace(
        "perspective:Dealt),",
        "perspective:Dealt),source:(target:BasicAttackDamage),",
    );
    let mut fixture = prepare_joint([&rule, "", ""], counter_gear(), false);
    let actor = fixture.actors[0];
    let defender = fixture.enemies[0];
    for (index, unit) in [actor, defender].into_iter().enumerate() {
        native::joint_install_published_equipment(
            fixture.battle_mut(),
            unit,
            Uuid::from_u128(COUNTER_GEAR),
            Uuid::from_u128(0xda011 + index as u128),
        );
    }
    let target_before = hp(&fixture.core, defender);
    fixture
        .battle_mut()
        .units
        .get_mut(&actor)
        .unwrap()
        .stats
        .attack = 10;
    native::actual_basic_attack_transaction(fixture.battle_mut(), actor, defender, 0).unwrap();
    assert_eq!(
        target_before - hp(&fixture.core, defender),
        10,
        "counter8+5 must not amplify the incoming10"
    );
    assert_eq!(
        hp(&fixture.core, actor),
        3987,
        "defender counter flat8 plus half of original10"
    );
    assert_eq!(
        damages(fixture.battle()),
        vec![
            (Some(actor), defender, -10, Some(DamageSource::BasicAttack)),
            (Some(defender), actor, -8, Some(DamageSource::Environment)),
            (Some(defender), actor, -5, Some(DamageSource::Environment)),
        ]
    );
    let counter = &fixture
        .battle()
        .runtime_fact_summary()
        .latest_damage
        .as_ref()
        .unwrap()
        .source;
    assert_eq!(counter.actor, Some(defender));
    assert_eq!(
        counter.equipment.as_ref().unwrap().definition_uuid,
        Uuid::from_u128(COUNTER_GEAR)
    );
    assert_eq!(
        counter.equipment.as_ref().unwrap().runtime_item_instance_id,
        Uuid::from_u128(0xda012)
    );
    fixture.advance(11);
    assert_eq!(target_before - hp(&fixture.core, defender), 17);
    assert_eq!(hp(&fixture.core, actor), 3987);
    let rows = damages(fixture.battle());
    assert_eq!(
        rows.len(),
        4,
        "one original, two counters, one delayed child; no recursive reflection"
    );
    assert_eq!(
        rows[3],
        (Some(actor), defender, -7, Some(DamageSource::Ability))
    );
    let child = serde_json::to_value(
        &fixture
            .battle()
            .runtime_fact_summary()
            .latest_damage
            .as_ref()
            .unwrap()
            .source,
    )
    .unwrap();
    assert_eq!(
        fixture
            .battle()
            .runtime_fact_summary()
            .latest_damage
            .as_ref()
            .unwrap()
            .target,
        defender
    );
    assert_eq!(fixture.battle().runtime_fact_summary().damage_hp_lost, 30);
    assert_eq!(child["actor"], json!(actor));
    assert_eq!(child["causal_cause"]["owner"], json!(actor));
    assert_eq!(child["causal_cause"]["event_target"], json!(defender));
    assert_eq!(child["causal_cause"]["copy_uuid"], json!(fixture.copies[0]));
    assert_eq!(child["causal_cause"]["rule_id"], "basic_echo");
    assert_eq!(child["causal_cause"]["root_source"]["actor"], json!(actor));
    assert_eq!(
        child["causal_cause"]["triggering_source"]["actor"],
        json!(actor)
    );
    assert_eq!(
        fixture.battle().units[&actor].stats.attack_interval_ms,
        2017
    );
    assert_eq!(
        fixture.battle().units[&defender].stats.attack_interval_ms,
        2023
    );
    assert_eq!(
        fixture
            .battle()
            .event_log
            .entries
            .iter()
            .filter(|entry| matches!(
                entry.event,
                BattleLogEvent::StatChanged {
                    modifier: StatModifier {
                        stat: StatId::AttackIntervalMs,
                        ..
                    },
                    ..
                }
            ))
            .count(),
        2,
        "derived child and counters recreate neither attack nor hit item occurrences"
    );
    fixture.advance(100);
    assert_eq!(damages(fixture.battle()), rows);
}

#[test]
fn u10_audit_f01_released_basic_skips_counter_for_dead_or_missing_attacker_with_causal_owner() {
    let rule = damage_subscriber("basic_echo", 7, 10).replace(
        "perspective:Dealt),",
        "perspective:Dealt),source:(target:BasicAttackDamage),",
    );
    for missing in [false, true] {
        let mut fixture = prepare_joint([&rule, "", ""], counter_gear(), false);
        let actor = fixture.actors[0];
        let defender = fixture.enemies[0];
        native::joint_install_published_equipment(
            fixture.battle_mut(),
            defender,
            Uuid::from_u128(COUNTER_GEAR),
            Uuid::from_u128(0xda013),
        );
        let unit = fixture.battle_mut().units.get_mut(&actor).unwrap();
        unit.stats.attack = 10;
        unit.stats.current_health = 1;
        unit.basic_attack.delivery = DeliveryDef::Projectile {
            // Authored speed uses 1,000,000 data units per world tile.
            // The prepared adjacent target has a 10ms center flight time.
            speed_units_per_ms: crate::game::battle::core::movement::types::DATA_UNITS_PER_WORLD
                as u32
                / 10,
            hit_policy: ProjectileHitPolicy::TargetLocked,
            allow_targetless_cast: false,
            max_range_tiles: None,
            max_lifetime_ms: None,
            max_kills: None,
            max_pierces: None,
            collision: Default::default(),
        };
        let before = hp(&fixture.core, defender);
        native::actual_basic_attack_transaction(fixture.battle_mut(), actor, defender, 0).unwrap();
        let launches = fixture
            .battle()
            .event_log
            .entries
            .iter()
            .filter_map(|entry| match entry.event {
                BattleLogEvent::BasicAttackProjectileLaunched {
                    projectile_id,
                    attacker_instance_id,
                    target_instance_id,
                    fired_at_ms,
                    expected_impact_time_ms,
                    ..
                } if attacker_instance_id == actor && target_instance_id == defender => {
                    Some((projectile_id, fired_at_ms, expected_impact_time_ms))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(launches.len(), 1, "one actual released projectile");
        let (projectile, fired_at, expected_impact) = launches[0];
        assert_eq!((fired_at, expected_impact), (0, 10));
        assert_eq!(
            hp(&fixture.core, defender),
            before,
            "real projectile released before arrival"
        );
        // Actual committed lethal damage supplies the dead-attacker branch.
        // Removal after that committed death is explicit missing-source setup;
        // the released projectile/context itself is never synthesized/edited.
        native::joint_committed_damage(fixture.battle_mut(), defender, actor, 1, 0);
        assert!(!fixture.battle().units[&actor].is_active());
        if missing {
            fixture.battle_mut().units.remove(&actor);
        }
        let start = damages(fixture.battle()).len();
        if missing {
            // Removing a deployed employee is explicit native preparation.
            // World correctly rejects its now-dangling deployment mapping;
            // exercise the native scheduler with the original execution state.
            let active = fixture.core.state.active_battle.as_mut().unwrap();
            active
                .battle
                .step_battle_execution_by(&mut active.execution, 100)
                .expect("missing original attacker: native timestamp advance");
        } else {
            fixture
                .core
                .advance_active_battle_for_server_tick(100)
                .expect("dead original attacker: World advance");
        }
        let impacts = fixture
            .battle()
            .event_log
            .entries
            .iter()
            .filter_map(|entry| match entry.event {
                BattleLogEvent::BasicAttackProjectileImpacted {
                    projectile_id,
                    attacker_instance_id,
                    target_instance_id,
                    hit,
                    ..
                } if projectile_id == projectile => {
                    Some((attacker_instance_id, target_instance_id, hit, entry.time_ms))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            impacts.len(),
            1,
            "dead/missing={missing}: one actual arrival"
        );
        assert_eq!(
            (impacts[0].0, impacts[0].1, impacts[0].2),
            (actor, defender, true)
        );
        assert!(impacts[0].3 > fired_at && impacts[0].3 <= expected_impact);
        assert_eq!(
            before - hp(&fixture.core, defender),
            10,
            "dead/missing={missing}: original released hit remains10"
        );
        assert_eq!(
            &damages(fixture.battle())[start..],
            &[(Some(actor), defender, -10, Some(DamageSource::BasicAttack)),],
            "dead/missing={missing}: no counter, amplification, recreated child or reflection loop"
        );
        assert_eq!(fixture.battle().units[&defender].stats.attack_interval_ms,2023,
            "the actual incoming basic still performs the defender's single ordinary hit occurrence");
        assert!(native::joint_causal_owner_present(fixture.battle()));
        eprintln!(
            "u10_joint_guard_completed missing={missing} boundary={} original_loss=10 additional_damage_rows=0 hit_markers=1",
            if missing { "native_timestamp" } else { "World" }
        );
    }
}

const KILL_RULE: &str = r#"(id:"joint_kill",apply:Subscribe((event:Kill,
    payload:Program((steps:[(id:"kill",delay_ms:20,target:Owner,
        effects:[(id:"kill_heal",op:Heal(amount:11))])])))))"#;
const DEATH_RULE: &str = r#"(id:"joint_death",apply:Subscribe((event:Death,
    payload:Program((steps:[(id:"death",delay_ms:10,target:Killer,
        effects:[(id:"death_heal",op:Heal(amount:7))])])))))"#;
const ALLY_RULE: &str = r#"(id:"joint_ally_death",apply:Subscribe((event:AllyDeath,
    payload:Program((steps:[(id:"ally",delay_ms:30,target:Owner,
        effects:[(id:"ally_heal",op:Heal(amount:13))])])))))"#;
const RELEASED_RULE: &str = r#"(id:"joint_unrelated",apply:Subscribe((
    event:DamageHpLost(perspective:Dealt),source:(target:BasicAttackDamage),
    payload:Program((steps:[(id:"release",target:Owner,
        effects:[(id:"released_dot",op:ApplyBuff(buff_id:"joint_released_dot",duration_ms:90))])])))))"#;

fn optional_casts(
    battle: &BattleCore,
    programs: &std::collections::BTreeMap<String, String>,
) -> Vec<(String, UnitInstanceId)> {
    battle
        .event_log
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            BattleLogEvent::AbilityCast {
                skill_id,
                caster_instance_id,
                ..
            } => programs
                .get(skill_id.as_str())
                .map(|rule| (rule.clone(), *caster_instance_id)),
            _ => None,
        })
        .collect()
}

fn configure_death_equipment(fixture: &mut JointWorld) {
    let actors = fixture.actors.clone();
    for (index, actor) in actors.into_iter().enumerate() {
        native::joint_install_published_equipment(
            fixture.battle_mut(),
            actor,
            Uuid::from_u128(DEATH_GEAR),
            Uuid::from_u128(0xda021 + index as u128),
        );
    }
}

fn assert_reaction_heal(
    battle: &BattleCore,
    rule: &str,
    owner: UnitInstanceId,
    target: UnitInstanceId,
    amount: i64,
    killer: UnitInstanceId,
    victim: UnitInstanceId,
    copy: Uuid,
) {
    let fact = battle.runtime_fact_summary().latest_heal.as_ref().unwrap();
    assert_eq!(fact.target, target);
    assert_eq!(fact.actual_delta(), amount);
    assert_eq!(fact.source.actor, Some(owner));
    let cause = fact.source.causal().unwrap().cause.as_ref().unwrap();
    assert_eq!(cause.rule_id, rule);
    assert_eq!(cause.owner, owner);
    assert_eq!(cause.actual_killer, Some(killer));
    assert_eq!(cause.event_target, Some(victim));
    assert_eq!(cause.copy_uuid, copy);
    assert_eq!(cause.root_source.actor, Some(killer));
    assert_eq!(cause.triggering_source.actor, Some(killer));
}

#[test]
fn u10_audit_f02_public_corrosion_suppresses_active_kill_and_preserves_released_effect_and_wave() {
    let third = format!("{ALLY_RULE},{RELEASED_RULE}");
    let mut fixture = prepare_joint([KILL_RULE, DEATH_RULE, &third], death_gear(), true);
    configure_death_equipment(&mut fixture);
    let actor = fixture.actors[0];
    let observer = fixture.actors[2];
    let enemy = fixture.enemies[0];
    fixture
        .battle_mut()
        .units
        .get_mut(&observer)
        .unwrap()
        .stats
        .attack = 1;
    native::actual_basic_attack_transaction(fixture.battle_mut(), observer, enemy, 0).unwrap();
    fixture.advance(1);
    assert!(
        fixture
            .battle()
            .event_log
            .entries
            .iter()
            .any(|entry| matches!(entry.event,
        BattleLogEvent::BuffApplied {caster_instance_id,target_instance_id,..}
            if caster_instance_id==observer && target_instance_id==observer)),
        "unrelated causal child already released its real DOT on a surviving player"
    );
    fixture
        .battle_mut()
        .units
        .get_mut(&enemy)
        .unwrap()
        .stats
        .current_health = 1;
    assert!(fixture
        .battle()
        .corrosion_ready(actor, &"u10_strike".into(), 1));
    let mut ordinary = fixture.core.clone();
    native::joint_committed_damage(
        &mut ordinary.state.active_battle.as_mut().unwrap().battle,
        actor,
        enemy,
        1,
        1,
    );
    assert_eq!(
        hp(&ordinary, actor),
        4003,
        "ordinary kill equipment heals before delayed relic reaction"
    );
    assert!(native::joint_pending_causal_rules(
        &ordinary.state.active_battle.as_ref().unwrap().battle
    )
    .iter()
    .any(|rule| rule == "joint_kill"));
    ordinary.advance_active_battle_for_server_tick(30).unwrap();
    assert_eq!(
        hp(&ordinary, actor),
        4014,
        "same published Kill rule delivers delayed11"
    );
    assert_reaction_heal(
        &ordinary.state.active_battle.as_ref().unwrap().battle,
        "joint_kill",
        actor,
        actor,
        11,
        actor,
        enemy,
        fixture.copies[0],
    );
    assert_eq!(
        optional_casts(
            &ordinary.state.active_battle.as_ref().unwrap().battle,
            &fixture.optional_programs
        ),
        vec![("joint_kill".into(), actor)]
    );
    let cost_before = fixture.battle().runtime_fact_summary().resource_spent;
    let damage_before = fixture.battle().runtime_fact_summary().damage_hp_lost;
    let death_before = fixture.battle().runtime_fact_summary().deaths;
    let victims = fixture
        .battle()
        .units
        .values()
        .filter(|u| u.owner == Side::Opponent && u.is_active())
        .map(|u| u.instance_id)
        .collect::<BTreeSet<_>>();
    execute_current_battle_mutation(
        &mut fixture.core,
        fixture.player,
        PlayerBehavior::ActivateSkill {
            employee_uuid: fixture.employees[0],
            skill_id: "u10_strike".into(),
            target: None,
        },
    )
    .unwrap();
    fixture.advance(1);
    let battle = fixture.battle();
    assert!(native::joint_causal_owner_present(battle));
    assert!(battle.units[&actor].corroded && !battle.units[&actor].is_active());
    assert_eq!(battle.units[&actor].employee_trauma, Some(100));
    let facts = battle.runtime_fact_summary();
    assert_eq!(facts.resource_spent - cost_before, 100);
    assert_eq!(
        facts.damage_hp_lost, damage_before,
        "Corrosion is special death, not invented HP-loss damage"
    );
    assert_eq!(facts.deaths - death_before, victims.len() as u64);
    let latest = facts.latest_death.as_ref().unwrap();
    assert_eq!(latest.cause, DeathCause::Corrosion);
    assert_eq!(latest.source.actor, Some(actor));
    assert!(
        latest
            .source
            .causal()
            .unwrap()
            .context
            .as_ref()
            .unwrap()
            .suppress_reactions,
        "actual Corrosion recorded the suppression-carrying source with causal state present"
    );
    let deaths = battle
        .event_log
        .entries
        .iter()
        .filter_map(|entry| match entry.event {
            BattleLogEvent::UnitDied {
                unit_instance_id,
                killer_instance_id,
                ..
            } => Some((unit_instance_id, killer_instance_id)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(deaths.len(), victims.len());
    assert!(deaths
        .iter()
        .all(|(victim, killer)| victims.contains(victim) && *killer == Some(actor)));
    assert!(optional_casts(battle, &fixture.optional_programs).is_empty());
    assert!(native::joint_pending_causal_rules(battle)
        .iter()
        .all(|rule| rule == "joint_unrelated"));
    assert_eq!(
        hp(&fixture.core, observer),
        4000,
        "no optional equipment heal reached living ally"
    );
    assert_eq!(battle.forced_pause_remaining_ms(), Some(1000));
    fixture.advance(1000);
    fixture.advance(100);
    assert_eq!(
        hp(&fixture.core, observer),
        3962,
        "two released DOT ticks19 survive Corrosion on unrelated living owner"
    );
    let dot_rows = damages(fixture.battle())
        .into_iter()
        .filter(|row| row.3 == Some(DamageSource::BuffTick))
        .collect::<Vec<_>>();
    assert_eq!(
        dot_rows,
        vec![
            (Some(observer), observer, -19, Some(DamageSource::BuffTick)),
            (Some(observer), observer, -19, Some(DamageSource::BuffTick)),
        ]
    );
    assert_eq!(
        fixture
            .battle()
            .event_log
            .entries
            .iter()
            .filter(|entry| matches!(entry.event,
        BattleLogEvent::BuffTick {caster_instance_id,target_instance_id,..}
            if caster_instance_id==observer && target_instance_id==observer))
            .count(),
        2
    );
    assert!(optional_casts(fixture.battle(), &fixture.optional_programs).is_empty());
    assert!(native::joint_pending_causal_rules(fixture.battle()).is_empty());
    let dot = &fixture
        .battle()
        .runtime_fact_summary()
        .latest_damage
        .as_ref()
        .unwrap()
        .source;
    assert_eq!(dot.actor, Some(observer));
    assert_eq!(
        dot.causal().unwrap().cause.as_ref().unwrap().rule_id,
        "joint_unrelated"
    );
    assert!(
        !dot.causal()
            .unwrap()
            .context
            .as_ref()
            .unwrap()
            .suppress_reactions,
        "suppression belongs to the Corrosion ancestry, not the whole battle"
    );
    fixture.advance(500);
    assert!(
        fixture
            .battle()
            .units
            .values()
            .any(|unit| unit.owner == Side::Opponent
                && unit.is_active()
                && !victims.contains(&unit.instance_id)),
        "required future wave still spawns"
    );
    assert_eq!(
        fixture.battle().runtime_fact_summary().deaths - death_before,
        victims.len() as u64
    );
}

#[test]
fn u10_audit_f02_prepared_opponent_death_and_allydeath_share_actual_corrosion_suppression() {
    let mut fixture = prepare_joint([KILL_RULE, DEATH_RULE, ALLY_RULE], death_gear(), true);
    let actor = fixture.actors[0];
    let victim = fixture.actors[1];
    let observer = fixture.actors[2];
    // The ordinary PvE case above covers reachable player Kill. These two
    // published employee identities retain their actual World-produced plans;
    // only native side preparation makes Death/AllyDeath eligible opponents.
    fixture.battle_mut().units.get_mut(&victim).unwrap().owner = Side::Opponent;
    fixture.battle_mut().units.get_mut(&observer).unwrap().owner = Side::Opponent;
    configure_death_equipment(&mut fixture);
    fixture
        .battle_mut()
        .units
        .get_mut(&victim)
        .unwrap()
        .stats
        .current_health = 1;
    let mut ordinary = fixture.battle().clone();
    let mut special = fixture.battle().clone();
    let mut special_state = fixture
        .core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .execution
        .clone();
    assert!(
        native::joint_causal_owner_present(&ordinary)
            && native::joint_causal_owner_present(&special)
    );
    native::joint_committed_damage(&mut ordinary, actor, victim, 1, 0);
    assert_eq!(
        ordinary
            .runtime_fact_summary()
            .latest_death
            .as_ref()
            .unwrap()
            .cause,
        DeathCause::Ordinary
    );
    assert_eq!(
        ordinary.units[&actor].stats.current_health, 4010,
        "equipment normal death2 + kill3 + allydeath5 positive control"
    );
    let pending = native::joint_pending_causal_rules(&ordinary);
    for rule in ["joint_death", "joint_kill", "joint_ally_death"] {
        assert!(
            pending.iter().any(|actual| actual == rule),
            "normal actual kill must queue {rule}"
        );
    }
    native::joint_drain_finite_reactions(&mut ordinary, 10);
    assert_reaction_heal(
        &ordinary,
        "joint_death",
        victim,
        actor,
        7,
        actor,
        victim,
        fixture.copies[1],
    );
    assert_eq!(ordinary.units[&actor].stats.current_health, 4017);
    native::joint_drain_finite_reactions(&mut ordinary, 20);
    assert_reaction_heal(
        &ordinary,
        "joint_kill",
        actor,
        actor,
        11,
        actor,
        victim,
        fixture.copies[0],
    );
    assert_eq!(
        ordinary.units[&actor].stats.current_health, 4028,
        "dead-owner Death7 and live-killer Kill11 are delivered after the equipment10"
    );
    native::joint_drain_finite_reactions(&mut ordinary, 30);
    assert_reaction_heal(
        &ordinary,
        "joint_ally_death",
        observer,
        observer,
        13,
        actor,
        victim,
        fixture.copies[2],
    );
    assert_eq!(
        ordinary.units[&observer].stats.current_health, 4013,
        "living opponent ally receives delayed13"
    );
    let casts = optional_casts(&ordinary, &fixture.optional_programs);
    assert_eq!(casts.len(), 3);
    for (rule, owner) in [
        ("joint_death", victim),
        ("joint_kill", actor),
        ("joint_ally_death", observer),
    ] {
        assert_eq!(
            casts
                .iter()
                .filter(|(id, actual)| id == rule && *actual == owner)
                .count(),
            1
        );
    }
    assert_eq!(
        ordinary
            .runtime_fact_summary()
            .latest_death
            .as_ref()
            .unwrap()
            .source
            .actor,
        Some(actor)
    );
    assert_eq!(
        ordinary
            .runtime_fact_summary()
            .latest_death
            .as_ref()
            .unwrap()
            .predeath
            .victim,
        victim
    );
    let captured = ordinary
        .runtime_fact_summary()
        .latest_heal
        .as_ref()
        .unwrap()
        .source
        .causal()
        .unwrap()
        .cause
        .as_ref()
        .unwrap();
    assert_eq!(captured.rule_id, "joint_ally_death");
    assert_eq!(captured.owner, observer);
    assert_eq!(captured.actual_killer, Some(actor));
    assert_eq!(captured.event_target, Some(victim));
    assert_eq!(captured.copy_uuid, fixture.copies[2]);
    assert_eq!(captured.triggering_source.actor, Some(actor));
    let captured_enemies = special
        .units
        .values()
        .filter(|u| u.owner == Side::Opponent && u.is_active())
        .map(|u| u.instance_id)
        .collect::<BTreeSet<_>>();
    // These opponents include abnormality identities. Percentage corrosion is
    // lethal at one HP and must still suppress all optional death reactions.
    for enemy in &captured_enemies { let unit = special.units.get_mut(enemy).unwrap(); unit.stats.current_health = 1; unit.threat_class = crate::game::battle::types::BattleUnitThreatClass::Boss; }
    let handle = special
        .request_skill_action(0, actor, "u10_strike".into(), None)
        .unwrap();
    special
        .step_battle_execution_until(&mut special_state, 0)
        .unwrap();
    assert!(special.units[&actor].corroded && !special.units[&actor].is_active());
    assert_eq!(special.units[&actor].employee_trauma, Some(100));
    assert!(captured_enemies
        .iter()
        .all(|enemy| !special.units[enemy].is_active()));
    let action = special.query_action(&handle).unwrap();
    assert_eq!(action.released_at_ms, Some(0));
    assert_eq!(action.facts.resource_spent, 100);
    assert_eq!(action.facts.damage_hp_lost, 0);
    assert_eq!(action.facts.deaths, captured_enemies.len() as u64);
    let latest = special
        .runtime_fact_summary()
        .latest_death
        .as_ref()
        .unwrap();
    assert_eq!(latest.cause, DeathCause::Corrosion);
    assert_eq!(latest.source.actor, Some(actor));
    assert!(
        latest
            .source
            .causal()
            .unwrap()
            .context
            .as_ref()
            .unwrap()
            .suppress_reactions
    );
    assert!(native::joint_pending_causal_rules(&special).is_empty(),
        "none of the otherwise eligible Death/Kill/AllyDeath producers may enqueue an optional child");
    assert!(optional_casts(&special, &fixture.optional_programs).is_empty());
    assert_eq!(
        special.units[&actor].stats.current_health, 4000,
        "no optional equipment or relic heal"
    );
    let deaths = special
        .event_log
        .entries
        .iter()
        .filter_map(|entry| match entry.event {
            BattleLogEvent::UnitDied {
                unit_instance_id,
                killer_instance_id,
                ..
            } => Some((unit_instance_id, killer_instance_id)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(deaths.len(), captured_enemies.len());
    assert!(deaths
        .iter()
        .all(|(victim, killer)| captured_enemies.contains(victim) && *killer == Some(actor)));
    special.advance_forced_pause_host_elapsed(1000).unwrap();
    native::joint_drain_finite_reactions(&mut special, 100);
    assert!(native::joint_pending_causal_rules(&special).is_empty());
    assert!(optional_casts(&special, &fixture.optional_programs).is_empty());
    assert_eq!(
        special.runtime_fact_summary().deaths,
        captured_enemies.len() as u64
    );
    assert_eq!(special.runtime_fact_summary().damage_hp_lost, 0);
}
