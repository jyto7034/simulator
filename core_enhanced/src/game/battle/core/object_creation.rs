//! Shared atomic creation. Public skill effects and preview use this owner.
use std::sync::Arc;

use crate::game::{
    battle::{
        enums::BattleEvent,
        event_log::{BattleEventCause, BattleEventRootCause},
        ids::UnitInstanceId,
        scenario::{
            EnemyMovementPlan, ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef,
            ScenarioUnitSpawn,
        },
        types::{
            BattleUnitDraft, BattleUnitSource, BattleUnitStatScale, BattleUnitThreatClass,
            IndependentUnitKind, UnitCombatProfile,
        },
    },
    behavior::GameError,
    data::independent_object_data::{
        CompiledIndependentObject, IndependentActorMotion, IndependentObjectKind,
    },
    enums::{Side, Tier},
    growth::GrowthStack,
    resources::Position,
};

use super::{
    independent_objects::{IndependentObject, ObjectFamilyKey},
    spawn_kernel::IndependentSpawnIdentity,
    BattleCore, SpawnKernelError, SpawnOccupancyPolicy, SpawnOccupantKind, SpawnPlacement,
    SpawnRejection, SpawnRequest, SpawnRequestId, SpawnRequestSettlement, SpawnRequestSource,
};

fn kernel_error(error: SpawnKernelError) -> GameError {
    // Native/authoring interruption remains sticky in its owning scope. It is
    // never converted into a cached rejection by this caller.
    GameError::InvalidStaticData(format!("independent object Spawn Kernel: {error:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        battle::{
            runtime_facts::RuntimeFactSource,
            scenario::{BattleScenario, IndependentPlacementDomain},
            tile_range::TileRangePattern,
        },
        data::{
            abnormality_data::{BasicAttackDef, MovementDef, ResonanceDef},
            boss_runtime_data::BossRuntimeCatalogData,
            independent_object_data::{
                IndependentActorDef, IndependentObjectDefinition, IndependentObjectId,
                IndependentTrapDef, IndependentWallDef,
            },
            summoned_unit_data::{SummonedUnitCatalogData, SummonerRemovalPolicyData},
            GameDataBuilder,
        },
    };
    use uuid::Uuid;

    fn fixture(side: Side) -> (BattleCore, UnitInstanceId) {
        fixture_with_skills(
            side,
            crate::game::data::skill_data::SkillDatabase::new(vec![]),
        )
    }

    fn fixture_with_skills(
        side: Side,
        skills: crate::game::data::skill_data::SkillDatabase,
    ) -> (BattleCore, UnitInstanceId) {
        fixture_with_policy(side, skills, SummonerRemovalPolicyData::Persist)
    }

    fn fixture_with_policy(
        side: Side,
        skills: crate::game::data::skill_data::SkillDatabase,
        policy: SummonerRemovalPolicyData,
    ) -> (BattleCore, UnitInstanceId) {
        fixture_with_motion(side, skills, policy, IndependentActorMotion::Stationary)
    }

    fn fixture_with_motion(
        side: Side,
        skills: crate::game::data::skill_data::SkillDatabase,
        policy: SummonerRemovalPolicyData,
        motion: IndependentActorMotion,
    ) -> (BattleCore, UnitInstanceId) {
        fixture_with_motion_and_blocking(side, skills, policy, motion, false, false)
    }

    fn fixture_with_motion_and_blocking(
        side: Side,
        skills: crate::game::data::skill_data::SkillDatabase,
        policy: SummonerRemovalPolicyData,
        motion: IndependentActorMotion,
        blocking_weapon: bool,
        airborne_form: bool,
    ) -> (BattleCore, UnitInstanceId) {
        let pattern = TileRangePattern {
            include_anchor_tile: true,
            rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
        };
        let mut trap = IndependentObjectDefinition {
            id: IndependentObjectId::new("friend"),
            base_uuid: Uuid::from_u128(200),
            family: "friends".into(),
            display_name: "Friend".into(),
            ttl_ms: 1_000,
            max_alive: 2,
            max_total: 4,
            on_source_removed: policy,
            evolves_to: None,
            kind: IndependentObjectKind::Trap(IndependentTrapDef {
                trigger_range: pattern.clone(),
                damage_range: pattern,
                damage_flat: 10,
                attack_percent: 100,
                damage_type: crate::game::battle::damage::DamageType::Physical,
                air_capable: false,
                max_targets: None,
            }),
        };
        let mut definitions = vec![trap.clone()];
        trap.id = IndependentObjectId::new("wall");
        trap.base_uuid = Uuid::from_u128(201);
        trap.family = "walls".into();
        trap.kind = IndependentObjectKind::Wall(IndependentWallDef {
            max_health: 50,
            defense: 1,
            magic_resistance: 2,
            inherit_source_defenses: true,
        });
        definitions.push(trap.clone());
        trap.id = IndependentObjectId::new("actor");
        trap.base_uuid = Uuid::from_u128(202);
        trap.family = "actors".into();
        trap.kind = IndependentObjectKind::Actor(IndependentActorDef {
            max_health: 40,
            attack: 12,
            defense: 3,
            magic_resistance: 4,
            movement: MovementDef::default(),
            motion,
            mobility: crate::game::battle::types::MobilityKind::Ground,
            basic_attack: BasicAttackDef::default(),
            resonance: ResonanceDef::default(),
            skill_id: skills
                .get_by_id("actor_pulse")
                .map(|skill| skill.id.clone()),
        });
        definitions.push(trap);
        // Three-step chains exercise form changes without changing the normal
        // create-only fixture behavior or its authored family limits.
        let mut forms = Vec::new();
        for small in &mut definitions {
            let mut large = small.clone();
            large.id = IndependentObjectId::new(format!("{}_large", small.id.as_str()));
            large.base_uuid = Uuid::from_u128(small.base_uuid.as_u128() + 1000);
            large.ttl_ms = 1500;
            match &mut large.kind {
                IndependentObjectKind::Trap(trap) => {
                    trap.damage_flat = 20;
                    trap.attack_percent = 200;
                }
                IndependentObjectKind::Wall(wall) => wall.max_health = 100,
                IndependentObjectKind::Actor(actor) => {
                    actor.max_health = 80;
                    actor.attack = 24;
                    if airborne_form {
                        actor.mobility = crate::game::battle::types::MobilityKind::Airborne;
                    }
                }
            }
            small.evolves_to = Some(large.id.clone());
            let mut tiny = large.clone();
            tiny.id = IndependentObjectId::new(format!("{}_tiny", small.id.as_str()));
            tiny.base_uuid = Uuid::from_u128(small.base_uuid.as_u128() + 2000);
            match &mut tiny.kind {
                IndependentObjectKind::Wall(wall) => wall.max_health = 15,
                IndependentObjectKind::Actor(actor) => actor.max_health = 15,
                _ => {}
            }
            large.evolves_to = Some(tiny.id.clone());
            forms.extend([large, tiny]);
        }
        definitions.extend(forms);
        let boss_data = BossRuntimeCatalogData::default()
            .compile_with_summoned_units(&SummonedUnitCatalogData {
                definitions: vec![],
                objects: definitions,
            })
            .unwrap();
        let employee_creator = side == Side::Player && !skills.skills.is_empty();
        let weapon = crate::game::data::equipment_data::EquipmentMetadata {
            content_family: None,
            id: "creation_fixture_sword".into(),
            uuid: Uuid::from_u128(300),
            name: "Creation fixture sword".into(),
            equipment_type: crate::game::data::equipment_data::EquipmentType::Weapon,
            rarity: crate::game::enums::RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: false,
            bound: false,
            cannot_unequip_reason: String::new(),
            triggered_effects: Default::default(),
            ability_activations: vec![],
            weapon_profile: Some(if blocking_weapon {
                crate::game::data::equipment_data::WeaponCombatProfile {
                    block_capacity: 1,
                    windup_ms: 0,
                    ..Default::default()
                }
            } else {
                Default::default()
            }),
            event_check_modifiers: vec![],
            event_choice_keys: vec![],
        };
        let data = GameDataBuilder::empty()
            .with_boss_runtime_data(Arc::new(boss_data))
            .with_skills(skills)
            .with_equipment(vec![weapon])
            .build_arc();
        let mut scenario = BattleScenario::empty((6, 4));
        scenario.battlefield.object_placement = Some(IndependentPlacementDomain {
            ground: (0..6)
                .flat_map(|x| (0..4).map(move |y| Position::new(x, y)))
                .collect(),
            reserved: vec![Position::new(5, 1)],
        });
        let group = ScenarioSpawnGroup {
            id: ScenarioGroupId::new("creator"),
            side,
            required_for_victory: false,
            enemy_movement_plan: None,
            spawns: vec![ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new("creator"),
                side,
                position: Position::new(0, 1),
                instance_salt: 0,
                draft: BattleUnitDraft {
                    owned_uuid: Uuid::from_u128(100),
                    source: if employee_creator {
                        BattleUnitSource::Employee(UnitCombatProfile::for_employee(
                            100, 30, 7, 8, 100,
                        ))
                    } else {
                        BattleUnitSource::TestFixture {
                            base_uuid: Uuid::from_u128(100),
                            profile: UnitCombatProfile::for_employee(100, 30, 7, 8, 100),
                        }
                    },
                    threat_class: BattleUnitThreatClass::Normal,
                    level: Tier::I,
                    stat_scale: Default::default(),
                    growth_stacks: GrowthStack::new(),
                    equipment_layout: None,
                    equipped_items: if employee_creator {
                        vec![crate::game::battle::types::BattleEquipmentBinding {
            weapon_stage: Default::default(),
                            base_uuid: Uuid::from_u128(300),
                            persistent_copy_uuid: None,
                            slot: None,
                        }]
                    } else {
                        vec![]
                    },
                    equipped_item_enhancements: vec![],
                    employee_start_condition: None,
                },
            }],
        };
        scenario.groups.push(group.clone());
        scenario
            .events
            .push(crate::game::battle::scenario::ScenarioEvent {
                id: crate::game::battle::scenario::ScenarioEventId::new("creator"),
                trigger: crate::game::battle::scenario::ScenarioTrigger::AtBattleStart,
                action: crate::game::battle::scenario::ScenarioAction::SpawnGroup {
                    group_id: group.id.clone(),
                },
            });
        scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(10_000);
        let mut core = BattleCore::new_from_scenario(scenario, data, 7);
        let creator = core.spawn_scenario_group_member(&group, 0).unwrap();
        (core, creator)
    }

    fn input(
        creator: UnitInstanceId,
        definition: &str,
        positions: Vec<Position>,
    ) -> IndependentSpawnIdentity {
        IndependentSpawnIdentity {
            creator,
            definition_id: definition.into(),
            positions,
            allow_same_source_evolution: false,
            wall_cost: None,
            source: RuntimeFactSource {
                actor: Some(creator),
                ..Default::default()
            }
            .independent_origin()
            .unwrap(),
            creation_action: None,
            time_ms: 10,
        }
    }

    include!("tests/u07_block_lifecycle.inc.rs");

    const TILE_CREATION: &str = r#"(skills:[(
        id:"create_friend", name:"Create friend", kind:Targeted, use_lifetime:Action, focus_time_ms:100,
        cast_targeting:Explicit(selection:player_tile,target:CastTarget,reach:global,air_capable:false),
        steps:[(id:"place",delay_ms:0,target:CastTarget,delivery:Instant,effects:[
            (id:"create",op:CreateObject(definition:(definition_id:"friend",anchor:CastTile,offsets:[(x:0,y:0)])))
        ])]
    )])"#;

    #[test]
    fn u07_object_snapshot_damage_overflow_is_rejected_before_payment_and_creation() {
        use crate::game::{
            ability::SkillId, battle::event_log::SkillCastTarget, data::skill_data::SkillDatabase,
        };
        let skills = SkillDatabase::from_ron("u07/overflow.ron", TILE_CREATION).unwrap();
        let skill = skills.get_by_id("create_friend").unwrap().clone();
        let (mut core, creator) = fixture_with_skills(Side::Player, skills);
        core.units.get_mut(&creator).unwrap().stats.attack = u32::MAX;
        let before = core.transaction_debug_snapshot();
        let queue = core.event_queue_debug_snapshot();
        assert!(matches!(
            core.validate_skill_object_placements(
                0,
                creator,
                &skill,
                Some(SkillCastTarget::Tile {
                    position: Position::new(1, 1)
                })
            ),
            Err(GameError::InvalidStaticData(_))
        ));
        assert!(core
            .request_skill_action(
                0,
                creator,
                SkillId::from("create_friend"),
                Some(SkillCastTarget::Tile {
                    position: Position::new(1, 1)
                })
            )
            .is_err());
        assert!(matches!(
            core.create_independent_objects(
                SpawnRequestId::from("overflow"),
                Side::Player,
                input(creator, "friend", vec![Position::new(1, 1)])
            ),
            Err(GameError::InvalidStaticData(_))
        ));
        assert_eq!(core.transaction_debug_snapshot(), before);
        assert_eq!(core.event_queue_debug_snapshot(), queue);
        assert!(core.independent_objects.objects.is_empty());
        assert_eq!(core.spawn_kernel.request_count(), 0);
    }

    #[test]
    fn u07_object_actor_path_is_checked_against_actual_battlefield_before_creation() {
        use crate::game::{battle::event_log::SkillCastTarget, data::skill_data::SkillDatabase};
        let skills = SkillDatabase::from_ron(
            "u07/path.ron",
            &TILE_CREATION.replace("definition_id:\"friend\"", "definition_id:\"actor\""),
        )
        .unwrap();
        let skill = skills.get_by_id("create_friend").unwrap().clone();
        let (mut core, creator) = fixture_with_motion(
            Side::Player,
            skills,
            SummonerRemovalPolicyData::Persist,
            IndependentActorMotion::Path {
                cells: vec![Position::new(1, 1), Position::new(6, 1)],
            },
        );
        let before = core.transaction_debug_snapshot();
        assert!(matches!(
            core.validate_skill_object_placements(
                0,
                creator,
                &skill,
                Some(SkillCastTarget::Tile {
                    position: Position::new(1, 1)
                })
            ),
            Err(GameError::OutOfBounds)
        ));
        assert!(matches!(
            core.create_independent_objects(
                SpawnRequestId::from("bad_path"),
                Side::Player,
                input(creator, "actor", vec![Position::new(1, 1)])
            ),
            Err(GameError::OutOfBounds)
        ));
        assert_eq!(core.transaction_debug_snapshot(), before);
        assert!(core.independent_objects.objects.is_empty());
        assert_eq!(core.spawn_kernel.request_count(), 0);
    }

    #[test]
    fn u07_object_every_offset_must_be_within_cast_reach() {
        use crate::game::{battle::event_log::SkillCastTarget, data::skill_data::SkillDatabase};
        let ron = TILE_CREATION.replace("reach:global", "reach:pattern,target_range:Some((include_anchor_tile:true,rows:[\".X.\",\".@.\",\"...\"]))")
            .replace("offsets:[(x:0,y:0)]", "offsets:[(x:0,y:0),(x:1,y:0)]");
        let skills = SkillDatabase::from_ron("u07/offset_range.ron", &ron).unwrap();
        let skill = skills.get_by_id("create_friend").unwrap().clone();
        let (core, creator) = fixture_with_skills(Side::Player, skills);
        let target = Position::new(1, 1);
        let (_, reach, range, _) = skill.cast_target_definition().unwrap();
        assert!(
            core.is_tile_in_skill_reach(creator, target, reach, range),
            "selected anchor is in range"
        );
        let before = core.transaction_debug_snapshot();
        assert!(matches!(
            core.validate_skill_object_placements(
                0,
                creator,
                &skill,
                Some(SkillCastTarget::Tile { position: target })
            ),
            Err(GameError::OutOfBounds)
        ));
        assert_eq!(core.transaction_debug_snapshot(), before);
    }

    #[test]
    fn u07_object_evolution_public_two_casts_keep_id_and_pay_each_use() {
        use crate::game::{
            ability::{SkillActivationMode, SkillId},
            battle::event_log::SkillCastTarget,
            data::skill_data::SkillDatabase,
        };
        let ron = TILE_CREATION.replace(
            "anchor:CastTile",
            "anchor:CastTile,allow_same_source_evolution:true",
        );
        let skills = SkillDatabase::from_ron("u07/evolution.ron", &ron).unwrap();
        let (mut core, creator) = fixture_with_skills(Side::Player, skills);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution_until(&mut execution, 0).unwrap();
        let caster = core.units.get_mut(&creator).unwrap();
        caster.skill_id = Some(SkillId::from("create_friend"));
        caster.skill_activation_mode = SkillActivationMode::Manual;
        caster.resonance_current = 100;
        let target = Some(SkillCastTarget::Tile {
            position: Position::new(1, 1),
        });
        let first = core
            .request_skill_action(0, creator, SkillId::from("create_friend"), target)
            .unwrap();
        assert_eq!(core.units[&creator].resonance_current, 0);
        core.step_battle_execution_until(&mut execution, 100)
            .unwrap();
        let id = *core.independent_objects.objects.keys().next().unwrap();
        let old_epoch = core.independent_objects.objects[&id].epoch;
        let mut control = core.clone();
        core.units.get_mut(&creator).unwrap().resonance_current = 100;
        let second = core
            .request_skill_action(101, creator, SkillId::from("create_friend"), target)
            .unwrap();
        assert_eq!(core.units[&creator].resonance_current, 0);
        core.step_battle_execution_until(&mut execution, 201)
            .unwrap();
        let view = core.independent_objects_snapshot(201).unwrap();
        assert_eq!(view.active.len(), 1);
        assert_eq!(view.active[0].object_id, id);
        assert_eq!(view.active[0].definition_id.as_str(), "friend_large");
        assert_eq!(view.active[0].epoch, old_epoch + 1);
        assert_eq!(view.active[0].expires_at_ms, 1701);
        assert_eq!(
            view.active[0].source.creation_action,
            Some(second.ordinal().0)
        );
        assert_ne!(first.ordinal(), second.ordinal());
        assert!(
            matches!(&view.creations[1].settlement, Some(SpawnRequestSettlement::Evolved { unit_ids }) if unit_ids == &[id])
        );
        let counter = core.independent_objects.counters.values().next().unwrap();
        assert_eq!(
            (
                counter.alive,
                counter.total,
                core.independent_objects.next_id
            ),
            (1, 1, 1)
        );
        core.expire_independent_object(id, old_epoch, 1100).unwrap();
        assert_eq!(core.independent_objects.objects.len(), 1);
        assert_eq!(
            control.independent_objects.objects[&id].definition.as_str(),
            "friend"
        );
        control
            .expire_independent_object(id, old_epoch, 1100)
            .unwrap();
        assert!(control.independent_objects.objects.is_empty());
        core.units.get_mut(&creator).unwrap().resonance_current = 100;
        let before = core.transaction_debug_snapshot();
        assert!(core
            .request_skill_action(202, creator, SkillId::from("create_friend"), target)
            .is_err());
        assert_eq!(core.transaction_debug_snapshot(), before);
        core.step_battle_execution_until(&mut execution, 1701)
            .unwrap();
        assert!(core.independent_objects.objects.is_empty());
    }

    #[test]
    fn u07_object_evolution_replay_and_foreign_copy_are_atomic() {
        use crate::game::battle::runtime_facts::EquipmentSourceSnapshot;
        let (mut core, creator) = fixture(Side::Player);
        let mut request = input(creator, "friend", vec![Position::new(1, 1)]);
        request.allow_same_source_evolution = true;
        request.source.equipment = Some(EquipmentSourceSnapshot {
            definition_uuid: Uuid::from_u128(99),
            runtime_item_instance_id: Uuid::from_u128(1001),
            persistent_copy_uuid: Some(Uuid::from_u128(1002)),
        });
        let first = core
            .create_independent_objects(
                SpawnRequestId::from("first"),
                Side::Player,
                request.clone(),
            )
            .unwrap();
        let SpawnRequestSettlement::Spawned { unit_ids } = first else {
            panic!()
        };
        let id = unit_ids[0];
        let mut foreign = request.clone();
        foreign
            .source
            .equipment
            .as_mut()
            .unwrap()
            .runtime_item_instance_id = Uuid::from_u128(1003);
        assert!(matches!(
            core.create_independent_objects(SpawnRequestId::from("foreign"), Side::Player, foreign)
                .unwrap(),
            SpawnRequestSettlement::Rejected {
                reason: SpawnRejection::OccupancyRejected(_)
            }
        ));
        assert_eq!(core.independent_objects.objects[&id].epoch, 1);
        request.time_ms = 20;
        let evolved = core
            .create_independent_objects(
                SpawnRequestId::from("evolve"),
                Side::Player,
                request.clone(),
            )
            .unwrap();
        assert_eq!(
            evolved,
            SpawnRequestSettlement::Evolved { unit_ids: vec![id] }
        );
        let before = core.transaction_debug_snapshot();
        let queue = core.event_queue_debug_snapshot();
        assert_eq!(
            core.create_independent_objects(
                SpawnRequestId::from("evolve"),
                Side::Player,
                request.clone()
            )
            .unwrap(),
            evolved
        );
        assert_eq!(core.transaction_debug_snapshot(), before);
        assert_eq!(core.event_queue_debug_snapshot(), queue);
        assert!(matches!(
            core.create_independent_objects(SpawnRequestId::from("third"), Side::Player, request)
                .unwrap(),
            SpawnRequestSettlement::Rejected {
                reason: SpawnRejection::OccupancyRejected(_)
            }
        ));
        assert_eq!(core.independent_objects.objects[&id].epoch, 2);
    }

    #[test]
    fn u07_object_evolution_hp_forms_preserve_health_and_clamp_without_new_slot() {
        for definition in ["wall", "actor"] {
            let (mut core, creator) = fixture(Side::Player);
            let mut request = input(creator, definition, vec![Position::new(2, 2)]);
            request.allow_same_source_evolution = true;
            let SpawnRequestSettlement::Spawned { unit_ids } = core
                .create_independent_objects(
                    SpawnRequestId::from("first"),
                    Side::Player,
                    request.clone(),
                )
                .unwrap()
            else {
                panic!()
            };
            let id = unit_ids[0];
            core.units.get_mut(&id).unwrap().stats.current_health = 20;
            core.units.get_mut(&id).unwrap().resonance_current = 7;
            if definition == "actor" {
                core.begin_sustained(
                    creator,
                    id,
                    crate::game::ability::sustained::SustainedEffectDef::Modifiers {
                        duration_ms: 1000,
                        modifiers: vec![crate::game::stats::StatModifier {
                            stat: crate::game::stats::StatId::Attack,
                            kind: crate::game::stats::StatModifierKind::Flat,
                            value: 10,
                        }],
                    },
                    11,
                )
                .unwrap();
                assert_eq!(core.units[&id].stats.attack, 22);
            }
            let old_attack_epoch = core.units[&id].basic_attack_epoch;
            request.time_ms = 20;
            assert_eq!(
                core.create_independent_objects(
                    SpawnRequestId::from("large"),
                    Side::Player,
                    request.clone()
                )
                .unwrap(),
                SpawnRequestSettlement::Evolved { unit_ids: vec![id] }
            );
            assert_eq!(core.units[&id].stats.current_health, 20);
            assert_eq!(core.units[&id].resonance_current, 7);
            assert!(core.units[&id].stats.max_health >= 80);
            assert!(core.units[&id].basic_attack_epoch > old_attack_epoch);
            if definition == "actor" {
                assert_eq!(core.units[&id].stats.attack, 34);
            }
            request.definition_id = format!("{definition}_large");
            request.time_ms = 30;
            assert_eq!(
                core.create_independent_objects(
                    SpawnRequestId::from("tiny"),
                    Side::Player,
                    request
                )
                .unwrap(),
                SpawnRequestSettlement::Evolved { unit_ids: vec![id] }
            );
            assert_eq!(
                (
                    core.units[&id].stats.current_health,
                    core.units[&id].stats.max_health
                ),
                (15, 15)
            );
            assert_eq!(core.independent_objects.objects[&id].epoch, 3);
            assert_eq!(core.independent_objects.next_id, 1);
            let counter = core.independent_objects.counters.values().next().unwrap();
            assert_eq!((counter.alive, counter.total), (1, 1));
            assert_eq!(core.units.len(), 2);
            let form_index = core.event_log.entries.iter().rposition(|entry| matches!(&entry.event, crate::game::battle::event_log::BattleLogEvent::IndependentObjectEvolved { receipt } if receipt.object_id == id && receipt.max_health == Some(15))).unwrap();
            let hp_index = core.event_log.entries.iter().rposition(|entry| matches!(&entry.event, crate::game::battle::event_log::BattleLogEvent::HpChanged { target_instance_id, hp_before: 20, hp_after: 15, .. } if *target_instance_id == id)).unwrap();
            assert!(form_index < hp_index);
            if definition == "actor" {
                core.reconcile_sustained(1011).unwrap();
                assert_eq!(
                    core.units[&id].stats.attack, 24,
                    "expiry must restore the new form base"
                );
            }
        }
    }

    #[test]
    fn native_u07_object_evolution_late_queue_fault_restores_form_counter_and_retry() {
        use crate::game::battle::runtime_control as native;
        let (mut base, creator) = fixture(Side::Player);
        let mut request = input(creator, "friend", vec![Position::new(1, 1)]);
        request.allow_same_source_evolution = true;
        base.create_independent_objects(
            SpawnRequestId::from("first"),
            Side::Player,
            request.clone(),
        )
        .unwrap();
        request.time_ms = 20;
        let mut probe = base.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.runtime_transaction(|core| {
                core.create_independent_objects(
                    SpawnRequestId::from("evolve"),
                    Side::Player,
                    request.clone(),
                )
            })
        });
        assert!(matches!(
            result.unwrap(),
            SpawnRequestSettlement::Evolved { .. }
        ));
        let short_limit = admissions
            .iter()
            .rfind(|(site, _, _)| *site == "queued_event_payload")
            .unwrap()
            .2
            - 1;
        let mut short = base.clone();
        short.runtime_facts.limits.max_retained_bytes = short_limit;
        let before = short.transaction_debug_snapshot();
        let queue = short.event_queue_debug_snapshot();
        let failure = short
            .runtime_transaction(|core| {
                core.create_independent_objects(
                    SpawnRequestId::from("evolve"),
                    Side::Player,
                    request.clone(),
                )
            })
            .unwrap_err();
        assert!(
            matches!(
                failure,
                GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "queued_event_payload",
                    ..
                })
            ),
            "{failure:?}"
        );
        assert_eq!(short.transaction_debug_snapshot(), before);
        assert_eq!(short.event_queue_debug_snapshot(), queue);
        assert_eq!(
            short
                .independent_objects
                .objects
                .values()
                .next()
                .unwrap()
                .definition
                .as_str(),
            "friend"
        );
        short.runtime_facts.limits = Default::default();
        assert!(matches!(
            short
                .runtime_transaction(|core| core.create_independent_objects(
                    SpawnRequestId::from("evolve"),
                    Side::Player,
                    request.clone()
                ))
                .unwrap(),
            SpawnRequestSettlement::Evolved { .. }
        ));
        assert_eq!(
            short
                .independent_objects
                .objects
                .values()
                .next()
                .unwrap()
                .epoch,
            2
        );
        assert_eq!(
            short
                .independent_objects
                .counters
                .values()
                .next()
                .unwrap()
                .total,
            1
        );
    }

    #[test]
    fn u07_object_public_tile_cast_rejects_before_resonance_and_releases_use_after_creation() {
        use crate::game::{
            ability::{SkillActivationMode, SkillId},
            battle::event_log::SkillCastTarget,
            data::skill_data::SkillDatabase,
        };
        let skills = SkillDatabase::from_ron("u07/skills.ron", TILE_CREATION).unwrap();
        let (mut core, creator) = fixture_with_skills(Side::Player, skills);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution_until(&mut execution, 0).unwrap();
        let caster = core.units.get_mut(&creator).unwrap();
        caster.skill_id = Some(SkillId::from("create_friend"));
        caster.skill_activation_mode = SkillActivationMode::Manual;
        caster.resonance_current = caster.resonance_max;
        let before = core.transaction_debug_snapshot();
        let queue = core.event_queue_debug_snapshot();
        assert!(core
            .request_skill_action(
                0,
                creator,
                SkillId::from("create_friend"),
                Some(SkillCastTarget::Tile {
                    position: Position::new(5, 1)
                })
            )
            .is_err());
        assert_eq!(core.transaction_debug_snapshot(), before);
        assert_eq!(core.event_queue_debug_snapshot(), queue);
        for _ in 0..3 {
            core.validate_manual_skill_activation(
                0,
                creator,
                &SkillId::from("create_friend"),
                Some(SkillCastTarget::Tile {
                    position: Position::new(1, 1),
                }),
            )
            .unwrap();
        }
        assert_eq!(
            core.transaction_debug_snapshot(),
            before,
            "preview must not mutate gameplay"
        );
        let action = core
            .request_skill_action(
                0,
                creator,
                SkillId::from("create_friend"),
                Some(SkillCastTarget::Tile {
                    position: Position::new(1, 1),
                }),
            )
            .unwrap();
        assert_eq!(core.units[&creator].resonance_current, 0);
        assert!(
            core.independent_objects.objects.is_empty(),
            "acceptance is not release"
        );
        core.step_battle_execution_until(&mut execution, 100)
            .unwrap();
        assert_eq!(core.independent_objects.objects.len(), 1);
        let object = core.independent_objects.objects.values().next().unwrap();
        assert_eq!(object.position, Position::new(1, 1));
        assert_eq!(object.source_attack, 30);
        assert_eq!(object.creation_action, Some(action.ordinal()));
        assert!(object.source.action.is_none() && !object.source.use_control);
        let query_before = core.transaction_debug_snapshot();
        let view = core.independent_objects_snapshot(100).unwrap();
        assert_eq!(view.active.len(), 1);
        assert_eq!(view.active[0].current_health, None);
        assert_eq!(
            view.active[0].source.creation_action,
            Some(action.ordinal().0)
        );
        assert_eq!(
            view.active[0].source.skill_id.as_ref().unwrap().as_str(),
            "create_friend"
        );
        assert_eq!(view.creations.len(), 1);
        assert!(
            matches!(&view.creations[0].settlement, Some(SpawnRequestSettlement::Spawned { unit_ids }) if unit_ids.len() == 1)
        );
        let decoded: super::super::IndependentObjectsSnapshot =
            serde_json::from_str(&serde_json::to_string(&view).unwrap()).unwrap();
        assert_eq!(decoded, view);
        assert_eq!(core.independent_objects_snapshot(100).unwrap(), view);
        assert_eq!(core.transaction_debug_snapshot(), query_before);
        let snapshot = core.query_action(&action).unwrap();
        assert!(snapshot.terminal.is_some() && snapshot.effects_settled_at_ms.is_some());
        assert!(
            !core.resonance_use_has_obligations(creator),
            "child TTL must not keep the creation use locked"
        );
        let mut witness = core.scenario.groups[0].clone();
        witness.spawns[0].instance_salt = 1;
        witness.spawns[0].position = Position::new(0, 3);
        witness.spawns[0].unit_ref = ScenarioUnitRef::new("witness");
        core.spawn_scenario_group_member(&witness, 0).unwrap();
        core.withdraw_unit(creator, 100).unwrap();
        core.step_battle_execution_until(&mut execution, 1_099)
            .unwrap();
        assert_eq!(core.independent_objects.objects.len(), 1);
        core.step_battle_execution_until(&mut execution, 1_100)
            .unwrap();
        assert!(core.independent_objects.objects.is_empty());
        let ended = core.independent_objects_snapshot(1_100).unwrap();
        assert!(ended.active.is_empty());
        assert_eq!(ended.creations, view.creations);
        assert_eq!(
            ended.last_removal.as_ref().unwrap().reason,
            super::super::ObjectRemovalReason::Expired
        );
        assert!(core.event_log.entries.iter().any(|entry| matches!(&entry.event,
            crate::game::battle::event_log::BattleLogEvent::IndependentObjectRemoved { receipt } if receipt.creator == creator)));
    }

    #[test]
    fn u07_object_opcode_strict_inputs_and_missing_definition_are_rejected() {
        use crate::game::data::skill_data::SkillDatabase;
        for (old, new) in [
            ("anchor:CastTile", "anchor:SourceTile"),
            ("offsets:[(x:0,y:0)]", "offsets:[]"),
            ("offsets:[(x:0,y:0)]", "offsets:[(x:0,y:0),(x:0,y:0)]"),
            ("delivery:Instant", "delivery:Instant,repeat:Times(count:2)"),
            ("anchor:CastTile", "anchor:CastTile,unknown:true"),
        ] {
            assert!(
                SkillDatabase::from_ron("u07/invalid.ron", &TILE_CREATION.replace(old, new))
                    .is_err(),
                "accepted {new}"
            );
        }
        let skills = SkillDatabase::from_ron("u07/missing.ron", TILE_CREATION).unwrap();
        assert!(BossRuntimeCatalogData::default()
            .compile()
            .unwrap()
            .get_summoned_unit_skill_validation(&skills)
            .is_err());
    }

    #[test]
    fn u07_object_creation_batch_replay_conflict_and_expiry_are_atomic() {
        let (mut core, creator) = fixture(Side::Player);
        let request = input(
            creator,
            "friend",
            vec![Position::new(1, 1), Position::new(2, 1)],
        );
        let settled = core
            .create_independent_objects(
                SpawnRequestId::from("create:1"),
                Side::Player,
                request.clone(),
            )
            .unwrap();
        let SpawnRequestSettlement::Spawned { unit_ids } = &settled else {
            panic!("{settled:?}")
        };
        assert_eq!(unit_ids.len(), 2);
        assert_eq!(
            core.units.len(),
            1,
            "traps must not acquire unit capabilities"
        );
        let before = core.transaction_debug_snapshot();
        let queue = core.event_queue_debug_snapshot();
        assert_eq!(
            core.create_independent_objects(
                SpawnRequestId::from("create:1"),
                Side::Player,
                request.clone()
            )
            .unwrap(),
            settled
        );
        assert_eq!(core.transaction_debug_snapshot(), before);
        assert_eq!(core.event_queue_debug_snapshot(), queue);
        let mut changed = request.clone();
        changed.positions[1] = Position::new(3, 1);
        assert!(core
            .create_independent_objects(SpawnRequestId::from("create:1"), Side::Player, changed)
            .is_err());
        let mut branch = crate::authoring::clone_shape::controlled_clone(&core).unwrap();
        branch
            .expire_independent_object(unit_ids[0], 1, 1_009)
            .unwrap();
        assert_eq!(branch.independent_objects.objects.len(), 2);
        branch
            .expire_independent_object(unit_ids[0], 1, 1_010)
            .unwrap();
        assert_eq!(branch.independent_objects.objects.len(), 1);
        assert_eq!(core.independent_objects.objects.len(), 2);
        assert_eq!(
            branch
                .create_independent_objects(SpawnRequestId::from("create:1"), Side::Player, request)
                .unwrap(),
            settled,
            "replay must not recreate an expired member"
        );

        let (mut rejected, creator) = fixture(Side::Player);
        let before_queue = rejected.event_queue_debug_snapshot();
        let request = input(
            creator,
            "friend",
            vec![Position::new(1, 1), Position::new(5, 1)],
        );
        let result = rejected
            .create_independent_objects(
                SpawnRequestId::from("reserved"),
                Side::Player,
                request.clone(),
            )
            .unwrap();
        assert!(matches!(
            result,
            SpawnRequestSettlement::Rejected {
                reason: SpawnRejection::UnwalkablePosition(_)
            }
        ));
        assert!(rejected.independent_objects.objects.is_empty());
        assert!(rejected.independent_objects.counters.is_empty());
        assert_eq!(rejected.independent_objects.next_id, 0);
        assert_eq!(rejected.event_queue_debug_snapshot(), before_queue);
        assert_eq!(
            rejected
                .create_independent_objects(SpawnRequestId::from("reserved"), Side::Player, request)
                .unwrap(),
            result
        );
    }

    #[test]
    fn u07_object_actor_and_wall_adapters_follow_actual_side_and_capabilities() {
        for side in [Side::Player, Side::Opponent] {
            let (mut core, creator) = fixture(side);
            for (name, position) in [
                ("actor", Position::new(1, 1)),
                ("wall", Position::new(2, 1)),
            ] {
                let result = core
                    .create_independent_objects(
                        SpawnRequestId::from(name),
                        side,
                        input(creator, name, vec![position]),
                    )
                    .unwrap();
                let SpawnRequestSettlement::Spawned { unit_ids } = result else {
                    panic!("{result:?}")
                };
                let unit = &core.units[&unit_ids[0]];
                assert_eq!(unit.owner, side);
                assert_eq!(unit.body.projected_tile(), position);
                assert!(!unit.can_move(), "stationary actor and wall must not chase");
                assert_eq!(unit.can_basic_attack(), name == "actor");
                assert!(core.tactical_states.get(&unit_ids[0]).is_none());
                if name == "wall" {
                    assert_eq!(unit.stats.current_health, 50);
                    assert_eq!((unit.stats.defense, unit.stats.magic_resist), (7, 8));
                    assert_eq!(
                        unit.role,
                        crate::game::battle::types::BattleUnitRole::IndependentWall
                    );
                }
            }
        }
    }

    #[test]
    fn u07_object_hp_zero_retires_without_unit_death_and_wall_rejects_heal() {
        use super::super::independent_objects::ObjectRemovalReason;
        use crate::game::battle::event_log::{BattleLogEvent, HpChangeReason};
        for name in ["wall", "actor"] {
            let (mut core, creator) = fixture(Side::Player);
            let result = core
                .create_independent_objects(
                    SpawnRequestId::from(name),
                    Side::Player,
                    input(creator, name, vec![Position::new(2, 1)]),
                )
                .unwrap();
            let SpawnRequestSettlement::Spawned { unit_ids } = result else {
                panic!()
            };
            let id = unit_ids[0];
            core.apply_hp_delta_and_record(Some(creator), id, -10, 20, HpChangeReason::Command);
            let hp = core.units[&id].stats.current_health;
            core.apply_hp_delta_and_record(Some(creator), id, 10, 21, HpChangeReason::Command);
            assert_eq!(
                core.units[&id].stats.current_health,
                hp + if name == "actor" { 10 } else { 0 }
            );
            core.apply_hp_delta_and_record(Some(creator), id, -100, 22, HpChangeReason::Command);
            assert!(!core.units.contains_key(&id));
            assert!(core.independent_objects.objects.is_empty());
            assert_eq!(
                core.independent_objects
                    .last_removal
                    .as_ref()
                    .unwrap()
                    .reason,
                ObjectRemovalReason::Destroyed
            );
            assert!(!core.event_log.entries.iter().any(|entry| matches!(entry.event, BattleLogEvent::UnitDied { unit_instance_id, .. } if unit_instance_id == id)));
            assert!(!core
                .retire_independent_object(id, 1, 23, ObjectRemovalReason::Destroyed)
                .unwrap());
            assert!(core
                .independent_objects
                .counters
                .values()
                .all(|c| c.alive == 0 && c.total == 1));
        }
    }

    #[test]
    fn u07_object_source_removal_policy_reconciles_all_kinds() {
        use super::super::independent_objects::ObjectRemovalReason;
        for policy in [
            SummonerRemovalPolicyData::Persist,
            SummonerRemovalPolicyData::Despawn,
        ] {
            for name in ["friend", "wall", "actor"] {
                let (mut core, creator) = fixture_with_policy(
                    Side::Player,
                    crate::game::data::skill_data::SkillDatabase::new(vec![]),
                    policy,
                );
                core.create_independent_objects(
                    SpawnRequestId::from(name),
                    Side::Player,
                    input(creator, name, vec![Position::new(2, 1)]),
                )
                .unwrap();
                core.withdraw_unit(creator, 20).unwrap();
                core.reconcile_independent_objects(20).unwrap();
                let persists = policy == SummonerRemovalPolicyData::Persist;
                assert_eq!(
                    core.independent_objects.objects.len(),
                    usize::from(persists)
                );
                if !persists {
                    assert_eq!(
                        core.independent_objects
                            .last_removal
                            .as_ref()
                            .unwrap()
                            .reason,
                        ObjectRemovalReason::SourceRemoved
                    );
                }
                core.reconcile_independent_objects(1_010).unwrap();
                assert!(core.independent_objects.objects.is_empty());
            }
        }
    }

    #[test]
    fn u07_object_trap_attacks_after_source_removal_and_consumes_only_on_hit() {
        use super::super::independent_objects::ObjectRemovalReason;
        use crate::game::battle::event_log::BattleLogEvent;
        let (mut core, creator) = fixture(Side::Player);
        let result = core
            .create_independent_objects(
                SpawnRequestId::from("trap"),
                Side::Player,
                input(creator, "friend", vec![Position::new(2, 1)]),
            )
            .unwrap();
        let SpawnRequestSettlement::Spawned { unit_ids } = result else {
            panic!()
        };
        let trap_id = unit_ids[0];
        core.reconcile_independent_objects(20).unwrap();
        assert!(
            core.independent_objects.objects.contains_key(&trap_id),
            "no enemy means no consumption"
        );
        core.withdraw_unit(creator, 21).unwrap();
        let mut enemy = core.scenario.groups[0].clone();
        enemy.id = ScenarioGroupId::new("enemy");
        enemy.side = Side::Opponent;
        enemy.spawns[0].side = Side::Opponent;
        enemy.spawns[0].instance_salt = 1;
        enemy.spawns[0].unit_ref = ScenarioUnitRef::new("enemy");
        enemy.spawns[0].position = Position::new(3, 1);
        enemy.spawns[0].draft.source = BattleUnitSource::TestFixture {
            base_uuid: Uuid::from_u128(100),
            profile: UnitCombatProfile::for_employee(200, 0, 0, 0, 100),
        };
        let target = core.spawn_scenario_group_member(&enemy, 0).unwrap();
        // Cleanup-only input reconciliation never attacks while paused.
        core.reconcile_independent_object_removals(22).unwrap();
        assert_eq!(core.units[&target].stats.current_health, 200);
        core.reconcile_independent_objects(22).unwrap();
        assert_eq!(core.units[&target].stats.current_health, 160);
        assert!(!core.independent_objects.objects.contains_key(&trap_id));
        assert_eq!(
            core.independent_objects
                .last_removal
                .as_ref()
                .unwrap()
                .reason,
            ObjectRemovalReason::AttackConsumed
        );
        assert!(core.event_log.entries.iter().any(|entry| matches!(entry.event,
            BattleLogEvent::HpChanged { source_instance_id: Some(source), target_instance_id, delta: -40, .. }
                if source == trap_id && target_instance_id == target)));
        let facts = core.runtime_fact_snapshot().unwrap();
        let source = &facts.summary.latest_damage.as_ref().unwrap().source;
        assert_eq!(source.actor, Some(trap_id));
        assert!(
            source.cast.is_none(),
            "creation cast is not the trap's current cast"
        );
        let origin = source.independent_object().unwrap();
        assert_eq!((origin.object_id, origin.creator), (trap_id, creator));
        assert_eq!(origin.definition_id.as_str(), "friend");
        assert_eq!(
            facts
                .latest_settled_action
                .as_ref()
                .unwrap()
                .skill_id
                .as_ref()
                .unwrap()
                .as_str(),
            "$independent_trap"
        );
        core.reconcile_independent_objects(23).unwrap();
        assert_eq!(
            core.units[&target].stats.current_health, 160,
            "one shot only"
        );
    }

    #[test]
    fn u07_object_wall_stops_actual_segment_then_basic_attack_destroys_and_route_resumes() {
        use crate::game::battle::types::MobilityKind;
        for airborne in [false, true] {
            let (mut core, creator) = fixture(Side::Player);
            let result = core
                .create_independent_objects(
                    SpawnRequestId::from("wall"),
                    Side::Player,
                    input(creator, "wall", vec![Position::new(2, 1)]),
                )
                .unwrap();
            let SpawnRequestSettlement::Spawned { unit_ids } = result else {
                panic!()
            };
            let wall = unit_ids[0];
            assert!(core
                .validate_player_tactical_destination(Position::new(2, 1), None)
                .is_err());
            let mut enemy = core.scenario.groups[0].clone();
            enemy.id = ScenarioGroupId::new("runner");
            enemy.side = Side::Opponent;
            enemy.spawns[0].side = Side::Opponent;
            enemy.spawns[0].instance_salt = 1;
            enemy.spawns[0].unit_ref = ScenarioUnitRef::new("runner");
            enemy.spawns[0].position = Position::new(0, 1);
            enemy.enemy_movement_plan = Some(EnemyMovementPlan::PathAlongCells {
                cells: vec![Position::new(4, 1)],
            });
            let mut profile = UnitCombatProfile::for_employee(200, 100, 0, 0, 100);
            profile.movement.speed_units_per_ms = 10_000_000;
            profile.stats.move_speed_units_per_ms = 10_000_000;
            profile.blockable = false;
            if airborne {
                profile.mobility_kind = MobilityKind::Airborne;
            }
            enemy.spawns[0].draft.source = BattleUnitSource::TestFixture {
                base_uuid: Uuid::from_u128(100),
                profile,
            };
            let target = core.spawn_scenario_group_member(&enemy, 0).unwrap();
            core.withdraw_unit(creator, 19).unwrap();
            core.run_continuous_attack_movement_tick(20, 100);
            if airborne {
                assert_eq!(core.blocked_by(target), None);
                assert!(core.units[&target].body.position.x > 3.0);
                continue;
            }
            assert_eq!(
                core.blocked_by(target),
                Some(wall),
                "runner={:?}",
                core.units[&target]
            );
            assert!(core.units[&target].body.position.x < 2.0);
            assert_eq!(
                core.select_basic_attack_target(target, None, None),
                Some(wall)
            );
            core.handle_basic_attack_start_event(
                20,
                20,
                target,
                Some(wall),
                false,
                BattleEventCause::Root {
                    kind: BattleEventRootCause::Period,
                },
            );
            while core.event_queue.peek_time_ms().is_some_and(|t| t < 1_010) {
                let event = core.event_queue.pop().unwrap();
                let now = event.event.view().time_ms();
                core.process_event(event, now).unwrap();
            }
            assert!(
                !core.units.contains_key(&wall),
                "actual basic attack must destroy wall before expiry"
            );
            core.run_continuous_attack_movement_tick(900, 100);
            assert_eq!(core.blocked_by(target), None);
            assert!(
                core.units[&target].body.position.x > 3.0,
                "original route resumes"
            );
        }
    }

    #[test]
    fn u07_object_actor_basic_attack_has_own_action_and_creation_observation() {
        let (mut core, creator) = fixture(Side::Player);
        let result = core
            .create_independent_objects(
                SpawnRequestId::from("actor"),
                Side::Player,
                input(creator, "actor", vec![Position::new(2, 2)]),
            )
            .unwrap();
        let SpawnRequestSettlement::Spawned { unit_ids } = result else {
            panic!()
        };
        let actor = unit_ids[0];
        let mut enemy = core.scenario.groups[0].clone();
        enemy.id = ScenarioGroupId::new("target");
        enemy.side = Side::Opponent;
        enemy.spawns[0].side = Side::Opponent;
        enemy.spawns[0].instance_salt = 1;
        enemy.spawns[0].unit_ref = ScenarioUnitRef::new("target");
        enemy.spawns[0].position = Position::new(3, 2);
        enemy.spawns[0].draft.source = BattleUnitSource::TestFixture {
            base_uuid: Uuid::from_u128(100),
            profile: UnitCombatProfile::for_employee(200, 0, 0, 0, 100),
        };
        let target = core.spawn_scenario_group_member(&enemy, 0).unwrap();
        core.withdraw_unit(creator, 9).unwrap();
        while core.event_queue.peek_time_ms().is_some_and(|t| t < 1_010) {
            let event = core.event_queue.pop().unwrap();
            let now = event.event.view().time_ms();
            core.process_event(event, now).unwrap();
        }
        assert!(core.units[&target].stats.current_health < 200);
        let facts = core.runtime_fact_snapshot().unwrap();
        let source = &facts.summary.latest_damage.as_ref().unwrap().source;
        assert_eq!(source.actor, Some(actor));
        assert!(source.cast.is_none());
        let origin = source.independent_object().unwrap();
        assert_eq!((origin.object_id, origin.creator), (actor, creator));
        assert_eq!(origin.definition_id.as_str(), "actor");
    }

    #[test]
    fn u07_object_actor_auto_cast_binds_own_cast_and_keeps_creation_cast() {
        use crate::game::{
            ability::{SkillActivationMode, SkillId},
            battle::event_log::SkillCastTarget,
            data::skill_data::SkillDatabase,
        };
        let authored = TILE_CREATION.replace("definition_id:\"friend\"", "definition_id:\"actor\"");
        let mut skills = SkillDatabase::from_ron("u07/create_actor.ron", &authored).unwrap();
        let pulse = SkillDatabase::from_ron("u07/actor_pulse.ron", r#"(skills:[(
            id:"actor_pulse", name:"Pulse", kind:Targeted, use_lifetime:Action, focus_time_ms:1,
            cast_targeting:Explicit(selection:automatic_unit,target:EnemySingle(rule:Nearest),reach:global,air_capable:false),
            steps:[(id:"hit",delay_ms:0,target:CastTarget,delivery:Instant,effects:[(id:"damage",op:Damage(amount:7,damage_type:True))])]
        )])"#).unwrap();
        skills.skills.extend(pulse.skills);
        let (mut core, creator) = fixture_with_skills(Side::Player, skills);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution_until(&mut execution, 0).unwrap();
        let unit = core.units.get_mut(&creator).unwrap();
        unit.skill_id = Some(SkillId::from("create_friend"));
        unit.skill_activation_mode = SkillActivationMode::Manual;
        unit.resonance_current = unit.resonance_max;
        let creation = core
            .request_skill_action(
                0,
                creator,
                SkillId::from("create_friend"),
                Some(SkillCastTarget::Tile {
                    position: Position::new(1, 1),
                }),
            )
            .unwrap();
        core.step_battle_execution_until(&mut execution, 100)
            .unwrap();
        let actor = *core.independent_objects.objects.keys().next().unwrap();
        let mut enemy = core.scenario.groups[0].clone();
        enemy.id = ScenarioGroupId::new("target");
        enemy.side = Side::Opponent;
        enemy.spawns[0].side = Side::Opponent;
        enemy.spawns[0].instance_salt = 1;
        enemy.spawns[0].unit_ref = ScenarioUnitRef::new("target");
        enemy.spawns[0].position = Position::new(3, 1);
        enemy.spawns[0].draft.equipped_items.clear();
        enemy.spawns[0].draft.source = BattleUnitSource::TestFixture {
            base_uuid: Uuid::from_u128(100),
            profile: UnitCombatProfile::for_employee(200, 0, 0, 0, 100),
        };
        let target = core.spawn_scenario_group_member(&enemy, 0).unwrap();
        core.withdraw_unit(creator, 100).unwrap();
        let unit = core.units.get_mut(&actor).unwrap();
        unit.resonance_current = unit.resonance_max;
        assert!(matches!(
            core.request_skill_action(100, actor, SkillId::from("actor_pulse"), None),
            Err(GameError::ActionUnavailable {
                reason: crate::game::behavior::ActionDisabledReasonCode::UnitUnavailable,
                ..
            })
        ));
        // A second wrapper admission occurs after the object's own action was
        // accepted/released and its cast sequence/provenance were bound. The
        // sticky native fault must discard that entire candidate.
        {
            use crate::game::battle::runtime_control as native;
            let retained = core.retained_runtime_fact_extent().unwrap();
            let mut probe = core.clone();
            let (result, admissions) = native::observe_retained_admissions(|| {
                native::run(probe.runtime_facts.limits, || {
                    native::seed_retained_extent(retained)?;
                    probe.runtime_transaction(|candidate| {
                        candidate.invoke_ability(
                            101,
                            actor,
                            "actor_pulse",
                            None,
                            Default::default(),
                            false,
                        );
                        Ok(())
                    })
                })
            });
            result.unwrap();
            let wrappers: Vec<_> = admissions
                .iter()
                .filter(|(site, _, _)| *site == "extended_source_observation")
                .collect();
            assert_eq!(wrappers.len(), 2);
            core.runtime_facts.limits.max_retained_bytes = wrappers[1].2 - 1;
            let before = (
                core.transaction_debug_snapshot(),
                core.event_queue_debug_snapshot(),
                core.skill_cast_seq,
                core.runtime_fact_source.clone(),
            );
            let failure = core
                .runtime_transaction(|candidate| {
                    native::seed_retained_extent(retained)?;
                    candidate.invoke_ability(
                        101,
                        actor,
                        "actor_pulse",
                        None,
                        Default::default(),
                        false,
                    );
                    Ok(())
                })
                .unwrap_err();
            assert!(
                matches!(
                    failure,
                    GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                        code: native::RuntimeFaultCode::RetainedByteLimit,
                        site: "extended_source_observation",
                        ..
                    })
                ),
                "{failure:?}"
            );
            assert_eq!(
                (
                    core.transaction_debug_snapshot(),
                    core.event_queue_debug_snapshot(),
                    core.skill_cast_seq,
                    core.runtime_fact_source.clone()
                ),
                before
            );
            assert_eq!(core.units[&target].stats.current_health, 200);
            assert_eq!(
                core.independent_objects.objects[&actor].creation_action,
                Some(creation.ordinal())
            );
            core.runtime_facts.limits = Default::default();
        }
        core.process_event(
            BattleEvent::AutoCastStart {
                time_ms: 101,
                caster_instance_id: actor,
                cause: BattleEventCause::Root {
                    kind: BattleEventRootCause::Period,
                },
            },
            101,
        )
        .unwrap();
        core.step_battle_execution_until(&mut execution, 200)
            .unwrap();
        assert_eq!(core.units[&target].stats.current_health, 193);
        let facts = core.runtime_fact_snapshot().unwrap();
        let source = &facts.summary.latest_damage.as_ref().unwrap().source;
        assert_eq!(source.actor, Some(actor));
        let cast = source.cast.as_ref().unwrap();
        assert_eq!(cast.caster_instance_id, actor);
        assert_eq!(cast.skill_id.as_str(), "actor_pulse");
        let origin = source.independent_object().unwrap();
        assert_eq!(origin.creation_action, Some(creation.ordinal()));
        assert_eq!(
            origin.creation_cast.as_ref().unwrap().caster_instance_id,
            creator
        );
        assert_eq!(
            origin.creation_cast.as_ref().unwrap().skill_id.as_str(),
            "create_friend"
        );
        assert_eq!(
            core.query_action(&creation).unwrap().facts.damage_mutations,
            0
        );
    }

    #[test]
    fn native_u07_object_late_retained_fault_rolls_back_batch_and_retries() {
        use crate::game::battle::runtime_control::{
            self as native, RuntimeFactLimits, RuntimeFaultCode,
        };
        let (base, creator) = fixture(Side::Player);
        let request = input(
            creator,
            "friend",
            vec![Position::new(1, 1), Position::new(2, 1)],
        );
        let before = base.retained_runtime_fact_extent().unwrap();
        let mut probe = base.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            native::run(RuntimeFactLimits::default(), || {
                native::seed_retained_extent(before)?;
                probe.create_independent_objects(
                    SpawnRequestId::from("native:1"),
                    Side::Player,
                    request.clone(),
                )
            })
        });
        assert!(matches!(
            result.unwrap(),
            SpawnRequestSettlement::Spawned { .. }
        ));
        let owner_admissions = admissions
            .iter()
            .filter(|(site, _, _)| *site == "independent_object_owner")
            .collect::<Vec<_>>();
        assert_eq!(owner_admissions.len(), 2);
        let short_limit = owner_admissions[1].2 - 1;
        let mut short = base.clone();
        let queue = short.event_queue_debug_snapshot();
        let failure = native::run(
            RuntimeFactLimits {
                max_retained_bytes: short_limit,
                ..Default::default()
            },
            || {
                native::seed_retained_extent(before)?;
                short.create_independent_objects(
                    SpawnRequestId::from("native:1"),
                    Side::Player,
                    request.clone(),
                )
            },
        )
        .unwrap_err();
        assert!(
            matches!(
                failure,
                GameError::BattleRuntimeFault(
                    crate::game::battle::runtime_control::BattleRuntimeFault {
                        code: RuntimeFaultCode::RetainedByteLimit,
                        ..
                    }
                )
            ),
            "{failure:?}"
        );
        assert!(short.independent_objects.objects.is_empty());
        assert!(short.independent_objects.counters.is_empty());
        assert_eq!(short.independent_objects.next_id, 0);
        assert_eq!(short.units.len(), 1);
        assert_eq!(short.event_queue_debug_snapshot(), queue);
        assert!(
            short
                .replay_independent_request(
                    &SpawnRequestId::from("native:1"),
                    Side::Player,
                    &request
                )
                .unwrap()
                .is_none(),
            "interruption must not become a cached gameplay rejection"
        );
        assert!(
            matches!(short.create_independent_objects(SpawnRequestId::from("native:1"), Side::Player, request).unwrap(), SpawnRequestSettlement::Spawned { unit_ids } if unit_ids.len() == 2)
        );
    }
    include!("tests/u07_segmented.inc.rs");
    include!("tests/u07_official.inc.rs");
    include!("tests/u07_edges.inc.rs");
}

impl BattleCore {
    fn validate_independent_creation_snapshot(
        &self,
        definition: &CompiledIndependentObject,
        source_attack: u32,
    ) -> Result<(), GameError> {
        match &definition.definition.kind {
            IndependentObjectKind::Trap(trap) => {
                super::object_attack::trap_base_damage(trap, source_attack)?;
            }
            IndependentObjectKind::Actor(actor) => {
                if let IndependentActorMotion::Path { cells } = &actor.motion {
                    crate::authoring::control::charge(
                        cells.len() as u64,
                        "object_path_validation",
                    )?;
                    if cells
                        .iter()
                        .any(|cell| !self.battlefield.is_walkable_tile(*cell))
                    {
                        return Err(GameError::OutOfBounds);
                    }
                }
            }
            IndependentObjectKind::Wall(_) => {}
        }
        Ok(())
    }

    pub(super) fn object_creation_positions(
        &self,
        creator: UnitInstanceId,
        definition: &crate::game::ability::objects::CreateObjectDef,
        target: Option<crate::game::battle::event_log::SkillCastTarget>,
    ) -> Result<Vec<Position>, GameError> {
        use crate::game::{
            ability::objects::ObjectPlacementAnchor, battle::event_log::SkillCastTarget,
        };
        let unit = self
            .units
            .get(&creator)
            .filter(|unit| unit.is_active())
            .ok_or(GameError::UnitNotFound)?;
        let anchor = match (definition.anchor, target) {
            (ObjectPlacementAnchor::SourceTile, _) => unit.body.projected_tile(),
            (ObjectPlacementAnchor::CastTile, Some(SkillCastTarget::Tile { position })) => position,
            _ => return Err(GameError::InvalidAction),
        };
        crate::authoring::control::charge(
            definition.offsets.len() as u64,
            "object_placement_offsets",
        )?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            definition
                .offsets
                .len()
                .checked_mul(std::mem::size_of::<Position>()),
            "object_placement_offsets",
        )?;
        definition
            .offsets
            .iter()
            .map(|offset| {
                Ok(Position::new(
                    anchor
                        .x
                        .checked_add(offset.x)
                        .ok_or(GameError::OutOfBounds)?,
                    anchor
                        .y
                        .checked_add(offset.y)
                        .ok_or(GameError::OutOfBounds)?,
                ))
            })
            .collect()
    }

    pub(super) fn object_preview_family(
        &self,
        creator: UnitInstanceId,
        skill: &crate::game::ability::SkillDef,
        object: &CompiledIndependentObject,
    ) -> Result<ObjectFamilyKey, GameError> {
        let unit = self
            .units
            .get(&creator)
            .filter(|unit| unit.is_active())
            .ok_or(GameError::UnitNotFound)?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(object.definition.family.len()),
            "object_preview_family",
        )?;
        Ok(ObjectFamilyKey {
            creator,
            fragment_copy: if self.runtime_fact_source.reinput().is_some()
                || self.runtime_fact_source.reaction
            {
                self.runtime_fact_source
                    .fragment
                    .as_ref()
                    .map(|source| source.copy_uuid)
            } else {
                unit.fragment_source
                    .as_ref()
                    .filter(|source| {
                        source.skill_id == skill.id && self.runtime_fact_source.equipment.is_none()
                    })
                    .map(|source| source.copy_uuid)
            },
            equipment_copy: self
                .runtime_fact_source
                .equipment
                .map(|source| source.runtime_item_instance_id),
            family: crate::authoring::clone_shape::controlled_clone(&object.definition.family)?,
        })
    }

    pub(super) fn validate_skill_object_placements(
        &self,
        now: u64,
        creator: UnitInstanceId,
        skill: &crate::game::ability::SkillDef,
        target: Option<crate::game::battle::event_log::SkillCastTarget>,
    ) -> Result<(), GameError> {
        crate::authoring::control::charge(skill.steps.len() as u64, "object_skill_scan")?;
        for step in &skill.steps {
            crate::authoring::control::charge(step.effects.len() as u64, "object_effect_scan")?;
            for effect in &step.effects {
                let crate::game::ability::SkillEffectDef::CreateObject { definition } = &effect.op
                else {
                    continue;
                };
                let unit = self
                    .units
                    .get(&creator)
                    .filter(|unit| unit.is_active())
                    .ok_or(GameError::UnitNotFound)?;
                let object = self
                    .game_data
                    .boss_runtime_data
                    .independent_objects()
                    .get(&definition.definition_id)
                    .ok_or(GameError::InvalidAction)?;
                let key = self.object_preview_family(creator, skill, object)?;
                if let Some(cost) = &definition.wall_cost {
                    self.plan_object_wall_cost(
                        creator,
                        cost,
                        key.fragment_copy,
                        key.equipment_copy,
                        now,
                    )?;
                }
                self.validate_independent_creation_snapshot(object, unit.stats.attack)?;
                let positions = self.object_creation_positions(creator, definition, target)?;
                let ground_only =
                    !matches!(object.definition.kind, IndependentObjectKind::Actor(_));
                self.validate_object_creation_reach(creator, skill, &positions)?;
                let mut new_count = 0_u32;
                for position in &positions {
                    let evolution = self.independent_evolution_candidate(
                        object,
                        &key,
                        *position,
                        definition.allow_same_source_evolution,
                    )?;
                    if let Some((_, next)) = &evolution {
                        self.validate_independent_creation_snapshot(next, unit.stats.attack)?;
                    } else {
                        new_count = new_count.checked_add(1).ok_or(GameError::InvalidAction)?;
                    }
                    if !self.independent_object_tile_allowed(*position, ground_only)? {
                        return Err(GameError::OutOfBounds);
                    }
                    if !self
                        .independent_object_occupants(
                            *position,
                            creator,
                            unit.owner,
                            evolution.as_ref().map(|(id, _)| *id),
                        )?
                        .is_empty()
                    {
                        return Err(GameError::PositionOccupied);
                    }
                }
                let count = new_count;
                let current = self
                    .independent_objects
                    .counters
                    .get(&key)
                    .copied()
                    .unwrap_or_default();
                if current
                    .alive
                    .checked_add(count)
                    .is_none_or(|n| n > object.definition.max_alive)
                    || current
                        .total
                        .checked_add(count)
                        .is_none_or(|n| n > object.definition.max_total)
                {
                    return Err(GameError::InvalidAction);
                }
            }
        }
        Ok(())
    }

    fn validate_object_creation_reach(
        &self,
        creator: UnitInstanceId,
        skill: &crate::game::ability::SkillDef,
        positions: &[Position],
    ) -> Result<(), GameError> {
        let (_, reach, range, _) = skill
            .cast_target_definition()
            .ok_or(GameError::InvalidAction)?;
        // Shared charge reaches both authoring and native budgets, including
        // global reach. Pattern membership is otherwise charged only inside an
        // active authoring scope, so native-only callers preprice its scans here.
        crate::authoring::control::charge(positions.len() as u64, "object_reach_work")?;
        if !crate::authoring::control::is_active()
            && matches!(reach, crate::game::ability::SkillTargetReach::Pattern)
        {
            if let Some(pattern) = range {
                crate::game::battle::runtime_control::charge(
                    pattern.rows.len() as u64,
                    "object_reach_pattern_rows",
                )?;
                let work = pattern
                    .rows
                    .iter()
                    .try_fold(0_u64, |sum, row| sum.checked_add(row.len() as u64 + 1))
                    .and_then(|cells| cells.checked_mul(6))
                    .and_then(|cells| cells.checked_mul(positions.len() as u64))
                    .ok_or_else(|| {
                        crate::game::battle::runtime_control::invariant("object_reach_pattern_work")
                    })?;
                crate::game::battle::runtime_control::charge(work, "object_reach_pattern_work")?;
            }
        }
        for position in positions {
            if !self.is_tile_in_skill_reach(creator, *position, reach, range) {
                return Err(GameError::OutOfBounds);
            }
        }
        Ok(())
    }

    pub(super) fn execute_object_creation_effect(
        &mut self,
        creator: UnitInstanceId,
        step_id: &str,
        effect: &crate::game::ability::authoring::AuthoredSkillEffect,
        definition: &crate::game::ability::objects::CreateObjectDef,
        cast: &crate::game::battle::skill_source::SkillCastExecutionSource,
        source: &crate::game::battle::runtime_facts::RuntimeFactSource,
        now_ms: u64,
    ) -> Result<super::types::SkillStepResult, GameError> {
        let side = self
            .units
            .get(&creator)
            .filter(|unit| unit.is_active())
            .ok_or(GameError::UnitNotFound)?
            .owner;
        let target = self
            .active_skill_casts
            .get(&cast.cast_seq)
            .and_then(|cast| cast.cast_target_anchor_position)
            .map(|position| crate::game::battle::event_log::SkillCastTarget::Tile { position });
        let positions = self.object_creation_positions(creator, definition, target)?;
        source.admit_owner("independent_effect_origin")?;
        let mut origin = source.independent_origin()?;
        origin.actor = Some(creator);
        origin.authored = effect.source().cloned();
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            step_id
                .len()
                .checked_add(effect.id.len())
                .and_then(|n| n.checked_add(128)),
            "independent_request_key",
        )?;
        let request_id = SpawnRequestId(format!(
            "independent:{}:{}:{}:{}:{}",
            cast.cast_seq,
            step_id.len(),
            step_id,
            effect.id.len(),
            effect.id
        ));
        if let Some(cost) = &definition.wall_cost {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(cost.pool_id.len()),
                "object_wall_policy",
            )?;
        }
        let input = IndependentSpawnIdentity {
            creator,
            definition_id: crate::authoring::clone_shape::controlled_clone(
                &definition.definition_id.0,
            )?,
            positions,
            allow_same_source_evolution: definition.allow_same_source_evolution,
            wall_cost: crate::authoring::clone_shape::controlled_clone(&definition.wall_cost)?,
            source: origin,
            creation_action: source.action_ordinal(),
            time_ms: now_ms,
        };
        let settlement = self.create_independent_objects(request_id, side, input)?;
        Ok(super::types::SkillStepResult {
            applied_effect_count: match settlement {
                SpawnRequestSettlement::Spawned { unit_ids }
                | SpawnRequestSettlement::Evolved { unit_ids } => unit_ids.len(),
                _ => 0,
            },
            ..Default::default()
        })
    }

    pub(super) fn independent_object_tile_allowed(
        &self,
        position: Position,
        ground_only: bool,
    ) -> Result<bool, GameError> {
        let domain_work = self
            .scenario
            .battlefield
            .object_placement
            .as_ref()
            .map_or(0, |domain| {
                domain.reserved.len().saturating_add(if ground_only {
                    domain.ground.len()
                } else {
                    0
                })
            });
        let work = domain_work.saturating_add(self.scenario.tactical_plan.field_work_objects.len());
        crate::authoring::control::charge(work as u64, "object_domain_scan")?;
        if !self.battlefield.is_walkable_tile(position) {
            return Ok(false);
        }
        if self
            .scenario
            .tactical_plan
            .field_work_objects
            .iter()
            .any(|object| object.position == position)
        {
            return Ok(false);
        }
        if let Some(domain) = &self.scenario.battlefield.object_placement {
            if domain.reserved.contains(&position) {
                return Ok(false);
            }
            if ground_only && !domain.ground.contains(&position) {
                return Ok(false);
            }
        } else if ground_only {
            return Ok(false);
        }
        Ok(true)
    }

    fn independent_object_occupants(
        &self,
        position: Position,
        creator: UnitInstanceId,
        side: Side,
        except: Option<UnitInstanceId>,
    ) -> Result<Vec<SpawnOccupantKind>, GameError> {
        crate::authoring::control::charge(
            (self.units.len() + self.independent_objects.objects.len()) as u64,
            "object_occupancy_scan",
        )?;
        // RequireUnoccupied only needs a witness; this bounded list does not
        // allocate a duplicate of every unit sharing a cell.
        let occupant = self
            .units
            .values()
            .find(|unit| {
                Some(unit.instance_id) != except
                    && unit.is_active()
                    && unit.body.projected_tile() == position
            })
            .map(|unit| {
                if unit.instance_id == creator {
                    SpawnOccupantKind::Summoner
                } else if !unit.is_combatant() {
                    SpawnOccupantKind::StaticDefenseObject
                } else if unit.owner == side {
                    SpawnOccupantKind::SameFaction
                } else {
                    SpawnOccupantKind::OpposingCombatant
                }
            })
            .or_else(|| {
                self.independent_objects
                    .objects
                    .values()
                    .any(|object| {
                        Some(object.id) != except
                            && !self.units.contains_key(&object.id)
                            && object.position == position
                    })
                    .then_some(SpawnOccupantKind::StaticDefenseObject)
            });
        if let Some(occupant) = occupant {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<SpawnOccupantKind>()),
                "object_occupancy_witness",
            )?;
            Ok(vec![occupant])
        } else {
            Ok(Vec::new())
        }
    }

    pub(super) fn create_independent_objects(
        &mut self,
        request_id: SpawnRequestId,
        side: Side,
        input: IndependentSpawnIdentity,
    ) -> Result<SpawnRequestSettlement, GameError> {
        if let Some(settlement) = self
            .replay_independent_request(&request_id, side, &input)
            .map_err(kernel_error)?
        {
            return Ok(settlement);
        }
        self.admit_authoring_runtime_growth(
            input.positions.len().saturating_mul(4).saturating_add(1),
            "independent_request_entities",
        )?;
        let definition = self
            .game_data
            .boss_runtime_data
            .independent_objects()
            .get_by_str(&input.definition_id)
            .cloned()
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "unknown independent object '{}'",
                    input.definition_id
                ))
            })?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(definition.definition.family.len()),
            "object_family_key",
        )?;
        let family = ObjectFamilyKey {
            creator: input.creator,
            fragment_copy: input
                .source
                .fragment
                .as_ref()
                .map(|source| source.copy_uuid),
            equipment_copy: input
                .source
                .equipment
                .map(|source| source.runtime_item_instance_id),
            family: crate::authoring::clone_shape::controlled_clone(&definition.definition.family)?,
        };
        if input.allow_same_source_evolution && input.positions.len() != 1 {
            return Err(GameError::InvalidAction);
        }
        let evolution = if let Some(position) = input.positions.first() {
            self.independent_evolution_candidate(
                &definition,
                &family,
                *position,
                input.allow_same_source_evolution,
            )?
        } else {
            None
        };
        let ground_only = !matches!(definition.definition.kind, IndependentObjectKind::Actor(_));
        crate::authoring::control::charge(input.positions.len() as u64, "object_placement_batch")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            input
                .positions
                .len()
                .checked_mul(std::mem::size_of::<SpawnPlacement>()),
            "object_placement_batch",
        )?;
        let reach_valid = if let Some(cast) = &input.source.cast {
            let skill = self
                .game_data
                .skill_data
                .get_by_id(cast.skill_id.as_str())
                .ok_or(GameError::InvalidAction)?;
            match self.validate_object_creation_reach(input.creator, skill, &input.positions) {
                Ok(()) => true,
                Err(GameError::OutOfBounds) => {
                    crate::authoring::control::poll("object_release_reach")?;
                    crate::game::battle::runtime_control::poll()?;
                    false
                }
                Err(error) => return Err(error),
            }
        } else {
            true
        };
        let mut placements = Vec::with_capacity(input.positions.len());
        for (index, position) in input.positions.iter().copied().enumerate() {
            placements.push(SpawnPlacement {
                order: u32::try_from(index).map_err(|_| GameError::InvalidAction)?,
                position,
                terrain_walkable: reach_valid
                    && self.independent_object_tile_allowed(position, ground_only)?,
                occupants: self.independent_object_occupants(
                    position,
                    input.creator,
                    side,
                    evolution.as_ref().map(|(id, _)| *id),
                )?,
            });
        }
        input.source.admit_owner("object_request_origin")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<IndependentSpawnIdentity>() + 64),
            "object_request_identity",
        )?;
        let input = Arc::new(input);
        let request = SpawnRequest {
            id: request_id,
            owner: side,
            occupancy: SpawnOccupancyPolicy::RequireUnoccupied,
            source: SpawnRequestSource::Independent(input.clone()),
            placements,
        };
        let Some(stats) = self
            .units
            .get(&input.creator)
            .filter(|unit| unit.is_active() && unit.owner == side)
            .map(|unit| unit.stats)
        else {
            return self
                .reject_spawn_request(
                    request,
                    SpawnRejection::MutationFailed {
                        game_error_code: "object_creator_unavailable".into(),
                    },
                )
                .map_err(kernel_error);
        };
        self.validate_independent_creation_snapshot(
            evolution
                .as_ref()
                .map_or(definition.as_ref(), |(_, next)| next.as_ref()),
            stats.attack,
        )?;
        if input.source.actor != Some(input.creator)
            || input.source.action.is_some()
            || input.source.use_control
            || input.source.delivery.is_some()
            || input.source.resource_capabilities.is_some()
            || input.source.allows_finite_children
            || input.source.flow.is_some()
            || input.source.reinput().is_some()
            || input.source.reaction
        {
            return self
                .reject_spawn_request(
                    request,
                    SpawnRejection::MutationFailed {
                        game_error_code: "object_origin_has_invalid_authority".into(),
                    },
                )
                .map_err(kernel_error);
        }
        if input.wall_cost.is_some()
            && (input.positions.len() != 1
                || input.allow_same_source_evolution
                || !matches!(definition.definition.kind, IndependentObjectKind::Wall(_)))
        {
            return Err(GameError::InvalidAction);
        }
        let wall_payment = match input
            .wall_cost
            .as_ref()
            .map(|cost| {
                self.plan_object_wall_cost(
                    input.creator,
                    cost,
                    family.fragment_copy,
                    family.equipment_copy,
                    input.time_ms,
                )
            })
            .transpose()
        {
            Ok(plan) => plan,
            Err(GameError::InvalidAction) => {
                return self
                    .reject_spawn_request(
                        request,
                        SpawnRejection::MutationFailed {
                            game_error_code: "object_wall_cost_unavailable".into(),
                        },
                    )
                    .map_err(kernel_error)
            }
            Err(error) => return Err(error),
        };
        let count = if evolution.is_some() {
            0
        } else {
            u32::try_from(input.positions.len()).map_err(|_| GameError::InvalidAction)?
        };
        let counter = self
            .independent_objects
            .counters
            .get(&family)
            .copied()
            .unwrap_or_default();
        if counter
            .alive
            .checked_add(count)
            .is_none_or(|n| n > definition.definition.max_alive)
            || counter
                .total
                .checked_add(count)
                .is_none_or(|n| n > definition.definition.max_total)
        {
            return self
                .reject_spawn_request(
                    request,
                    SpawnRejection::LimitReached {
                        max_alive: definition.definition.max_alive,
                        max_total: definition.definition.max_total,
                    },
                )
                .map_err(kernel_error);
        }
        self.execute_spawn_request_with_evolution(
            request,
            evolution.is_some(),
            |core, placement| {
                if let Some((id, next)) = &evolution {
                    core.evolve_independent_object(*id, next, &input, stats)?;
                    return Ok(*id);
                }
                let wall_health = wall_payment
                    .map(|plan| core.pay_object_wall_cost(input.creator, plan, input.time_ms))
                    .transpose()?;
                let next = core
                    .independent_objects
                    .next_id
                    .checked_add(1)
                    .ok_or(GameError::InvalidAction)?;
                let salt = u32::try_from(next).map_err(|_| GameError::InvalidAction)?;
                let id = Self::make_instance_id(definition.definition.base_uuid, side, salt);
                if core.units.contains_key(&id)
                    || core.independent_objects.objects.contains_key(&id)
                {
                    return Err(GameError::InvalidAction);
                }
                let expires_at_ms = input
                    .time_ms
                    .checked_add(u64::from(definition.definition.ttl_ms))
                    .ok_or(GameError::InvalidAction)?;
                let origin = crate::authoring::clone_shape::controlled_clone(&input.source)?;
                let object = IndependentObject {
                    id,
                    definition: crate::authoring::clone_shape::controlled_clone(
                        &definition.definition.id,
                    )?,
                    family: crate::authoring::clone_shape::controlled_clone(&family)?,
                    side,
                    position: placement.position,
                    epoch: 1,
                    created_at_ms: input.time_ms,
                    expires_at_ms,
                    source: origin,
                    creation_action: input.creation_action,
                    source_attack: stats.attack,
                    source_defense: stats.defense,
                    source_magic_resistance: stats.magic_resist,
                };
                core.register_independent_object(
                    object,
                    definition.definition.max_alive,
                    definition.definition.max_total,
                )?;
                if !matches!(definition.definition.kind, IndependentObjectKind::Trap(_)) {
                    let actual = core.spawn_independent_adapter(
                        &definition,
                        input.creator,
                        side,
                        placement.position,
                        salt,
                        stats.defense,
                        stats.magic_resist,
                        wall_health,
                    )?;
                    if actual != id {
                        return Err(GameError::InvalidAction);
                    }
                    core.record_spawned_units(
                        input.time_ms,
                        BattleEventCause::Root {
                            kind: BattleEventRootCause::Period,
                        },
                        &[id],
                    )?;
                    core.schedule_initial_attack_for_unit(id, input.time_ms);
                }
                core.event_queue.push(BattleEvent::IndependentObjectExpiry {
                    time_ms: expires_at_ms,
                    object_id: id,
                    epoch: 1,
                });
                crate::authoring::control::poll("independent_object_creation_complete")?;
                crate::game::battle::runtime_control::poll()?;
                core.independent_objects.next_id = next;
                Ok(id)
            },
        )
        .map_err(kernel_error)
    }

    pub(super) fn independent_adapter_profile(
        &self,
        definition: &CompiledIndependentObject,
        defense: i32,
        resistance: i32,
    ) -> Result<
        (
            UnitCombatProfile,
            IndependentUnitKind,
            Option<EnemyMovementPlan>,
        ),
        GameError,
    > {
        // Price every adapter scratch/profile/path copy before native constructors.
        // Scenario draft resolution copies the profile twice, then retains its
        // basic action and source identity in the runtime unit.
        let profile_bytes = definition.actor_profile.as_ref().map_or(
            Some(
                std::mem::size_of::<UnitCombatProfile>()
                    + 3 * std::mem::size_of::<String>()
                    + 9
                    + 80,
            ),
            |profile| {
                super::sustained::basic_extent(&profile.basic_attack)
                    .and_then(|n| n.checked_add(std::mem::size_of::<UnitCombatProfile>()))
                    .and_then(|n| {
                        n.checked_add(profile.skill_id.as_ref().map_or(0, |id| id.as_str().len()))
                    })
                    .and_then(|n| {
                        profile
                            .target_traits
                            .len()
                            .checked_mul(std::mem::size_of::<
                                crate::game::battle::types::UnitTargetTrait,
                            >())
                            .and_then(|traits| n.checked_add(traits))
                    })
            },
        );
        let path_cells = match &definition.definition.kind {
            IndependentObjectKind::Actor(actor) => match &actor.motion {
                IndependentActorMotion::Path { cells } => cells.len(),
                _ => 0,
            },
            _ => 0,
        };
        let bytes = profile_bytes
            .and_then(|n| n.checked_mul(3))
            .and_then(|n| {
                path_cells
                    .checked_mul(2 * std::mem::size_of::<Position>())
                    .and_then(|path| n.checked_add(path))
            })
            .and_then(|n| {
                definition
                    .definition
                    .id
                    .0
                    .len()
                    .checked_mul(2)
                    .and_then(|id| n.checked_add(id))
            })
            .and_then(|n| {
                n.checked_add(
                    std::mem::size_of::<ScenarioSpawnGroup>()
                        + std::mem::size_of::<ScenarioUnitSpawn>()
                        + 3 * 64,
                )
            })
            .and_then(|n| {
                n.checked_add(16 * (std::mem::size_of::<(ScenarioUnitRef, UnitInstanceId)>() + 16))
            });
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            bytes,
            "independent_adapter_storage",
        )?;
        crate::game::battle::runtime_control::charge(
            bytes.unwrap_or(usize::MAX) as u64,
            "independent_adapter_copy",
        )?;
        let mut meter = crate::authoring::control::CloneMeter::default();
        meter.scan_storage(path_cells.checked_mul(2).and_then(|n| n.checked_add(32)))?;
        if let Some(profile) = &definition.actor_profile {
            use crate::authoring::clone_shape::CloneShape;
            for _ in 0..3 {
                profile.visit_clone_shape(&mut meter)?;
            }
        }
        meter.finish("independent_adapter_storage", "independent_adapter_copy")?;
        let (profile, kind, movement) = match &definition.definition.kind {
            IndependentObjectKind::Actor(actor) => (
                crate::authoring::clone_shape::controlled_clone(
                    definition
                        .actor_profile
                        .as_ref()
                        .ok_or(GameError::InvalidAction)?,
                )?,
                IndependentUnitKind::Actor,
                match &actor.motion {
                    IndependentActorMotion::Stationary => None,
                    IndependentActorMotion::Path { cells } => {
                        Some(EnemyMovementPlan::PathAlongCells {
                            cells: crate::authoring::clone_shape::controlled_clone(cells)?,
                        })
                    }
                },
            ),
            IndependentObjectKind::Wall(wall) => {
                // The adapter has neutral unused attack fields; IndependentWall
                // grants HP reception only and never enables basic/skill work.
                let profile = UnitCombatProfile::for_employee(
                    wall.max_health,
                    0,
                    if wall.inherit_source_defenses {
                        defense
                    } else {
                        wall.defense
                    },
                    if wall.inherit_source_defenses {
                        resistance
                    } else {
                        wall.magic_resistance
                    },
                    100,
                );
                (profile, IndependentUnitKind::Wall, None)
            }
            IndependentObjectKind::Trap(_) => return Err(GameError::InvalidAction),
        };
        Ok((profile, kind, movement))
    }

    fn spawn_independent_adapter(
        &mut self,
        definition: &CompiledIndependentObject,
        creator: UnitInstanceId,
        side: Side,
        position: Position,
        salt: u32,
        defense: i32,
        resistance: i32,
        wall_health: Option<u32>,
    ) -> Result<UnitInstanceId, GameError> {
        self.admit_authoring_runtime_growth(2, "independent_adapter")?;
        let (mut profile, kind, movement) =
            self.independent_adapter_profile(definition, defense, resistance)?;
        if let Some(health) = wall_health {
            if kind != IndependentUnitKind::Wall || health == 0 || health > i32::MAX as u32 {
                return Err(GameError::InvalidAction);
            }
            profile.stats.max_health = health;
            profile.stats.current_health = health;
        }
        let owned_uuid = crate::game::determinism::uuid_v4_from_seed(
            self.seed,
            0x494e_444f_424a_4543,
            u64::from(salt),
        );
        let spawn = ScenarioUnitSpawn {
            unit_ref: ScenarioUnitRef::new(format!("independent:{owned_uuid}")),
            side,
            position,
            instance_salt: salt,
            draft: BattleUnitDraft {
                owned_uuid,
                source: BattleUnitSource::IndependentObject {
                    definition_id: definition.definition.id.as_str().into(),
                    base_uuid: definition.definition.base_uuid,
                    profile,
                    creator_instance_id: creator,
                    kind,
                },
                threat_class: BattleUnitThreatClass::Normal,
                level: Tier::I,
                stat_scale: BattleUnitStatScale::default(),
                growth_stacks: GrowthStack::new(),
                equipment_layout: None,
                equipped_items: Vec::new(),
                equipped_item_enhancements: Vec::new(),
                employee_start_condition: None,
            },
        };
        let group = ScenarioSpawnGroup {
            id: ScenarioGroupId::new(format!("independent:{owned_uuid}")),
            side,
            required_for_victory: false,
            enemy_movement_plan: movement,
            spawns: vec![spawn],
        };
        self.spawn_scenario_group_member(&group, 0)
    }
}
