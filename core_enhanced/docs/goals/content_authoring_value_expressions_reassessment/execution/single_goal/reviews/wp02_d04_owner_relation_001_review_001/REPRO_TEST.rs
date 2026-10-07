    fn review_owner_fixture(unrelated: usize, skills: Option<SkillDatabase>) -> OwnerFixture {
        let weapon_base = Uuid::from_u128(0x5101);
        let armor_base = Uuid::from_u128(0x5102);
        let weapon_x = Uuid::from_u128(0x5201);
        let armor_y = Uuid::from_u128(0x5202);
        let weapon_z = Uuid::from_u128(0x5203);
        let mut builder = GameDataBuilder::empty().with_equipment(vec![
            equipment_metadata("owner_weapon", weapon_base, EquipmentType::Weapon),
            equipment_metadata("owner_armor", armor_base, EquipmentType::Armor),
        ]);
        if let Some(skills) = skills {
            builder = builder.with_skills(skills);
        }
        let database = builder.build();
        let mut scenario = BattleScenario::empty((128, 128));
        let player_group = ScenarioGroupId::new("owner_players");
        scenario.groups.push(ScenarioSpawnGroup {
            id: player_group.clone(),
            side: Side::Player,
            required_for_victory: false,
            enemy_movement_plan: None,
            spawns: vec![
                ScenarioUnitSpawn {
                    unit_ref: ScenarioUnitRef::new("employee_a"),
                    side: Side::Player,
                    draft: employee_draft(
                        0x5001,
                        100,
                        3,
                        vec![
                            BattleEquipmentBinding {
                                base_uuid: weapon_base,
                                persistent_copy_uuid: Some(weapon_x),
                                slot: Some(EquipmentSlotAddress::Weapon),
                            },
                            BattleEquipmentBinding {
                                base_uuid: armor_base,
                                persistent_copy_uuid: Some(armor_y),
                                slot: Some(EquipmentSlotAddress::Armor),
                            },
                        ],
                    ),
                    position: Position::new(1, 1),
                    instance_salt: 0,
                },
                ScenarioUnitSpawn {
                    unit_ref: ScenarioUnitRef::new("employee_b"),
                    side: Side::Player,
                    draft: employee_draft(
                        0x5002,
                        100,
                        5,
                        vec![BattleEquipmentBinding {
                            base_uuid: weapon_base,
                            persistent_copy_uuid: Some(weapon_z),
                            slot: Some(EquipmentSlotAddress::Weapon),
                        }],
                    ),
                    position: Position::new(2, 1),
                    instance_salt: 1,
                },
                ScenarioUnitSpawn {
                    unit_ref: ScenarioUnitRef::new("summon"),
                    side: Side::Player,
                    draft: draft(0x5003, 0x5004, 100, 11),
                    position: Position::new(3, 1),
                    instance_salt: 2,
                },
            ],
        });
        let enemy_group = ScenarioGroupId::new("owner_enemies");
        scenario.groups.push(ScenarioSpawnGroup {
            id: enemy_group.clone(),
            side: Side::Opponent,
            required_for_victory: false,
            enemy_movement_plan: None,
            spawns: vec![ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new("enemy"),
                side: Side::Opponent,
                draft: draft(0x5005, 0x5006, 100, 7),
                position: Position::new(4, 1),
                instance_salt: 0,
            }],
        });
        for index in 0..unrelated {
            scenario.groups[0].spawns.push(ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new(format!("review_extra_{index}")),
                side: Side::Player,
                draft: employee_draft(0x100000 + index as u128, 100, 3, vec![BattleEquipmentBinding {
                    base_uuid: weapon_base,
                    persistent_copy_uuid: Some(Uuid::from_u128(0x200000 + index as u128)),
                    slot: Some(EquipmentSlotAddress::Weapon),
                }]),
                position: Position::new((index % 100) as i32, (10 + index / 100) as i32),
                instance_salt: (index + 10) as u32,
            });
        }
        let mut core = BattleCore::new_from_scenario(
            scenario,
            ValidatedGameData::from(Arc::new(database)),
            19,
        );
        let players = core.spawn_scenario_group(&player_group).unwrap();
        let enemies = core.spawn_scenario_group(&enemy_group).unwrap();
        OwnerFixture {
            core,
            employee_a: players[0],
            employee_b: players[1],
            summon: players[2],
            enemy: enemies[0],
            weapon_x,
            armor_y,
            weapon_z,
        }
    }


    pub(super) static REVIEW_SLOT_VISITS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    #[test]
    fn review_equipment_scan_work_is_not_charged() {
        use std::sync::atomic::Ordering;
        let mut observed_work = None;
        for unrelated in [0, 100, 1000] {
            let fixture = review_owner_fixture(unrelated, None);
            let plan = plan(EvalMoment::Live, "review_equipment_scan", resolved_attack_from(
                FiniteSource::ResolvedTargetsWhere(vec![ResolvedTargetPredicate::HasEquippedSlot(EquipmentSlotAddress::Weapon)]),
                Grouping::ByResolvedOwner(ResolvedOwnerRelation::EquippedSlot(EquipmentSlotAddress::Weapon)),
                Reduction::Max, Some(Reduction::Sum)));
            REVIEW_SLOT_VISITS.store(0, Ordering::Relaxed);
            let (result, usage) = control::run(
                &ExecutionLimits { max_work: 256, ..Default::default() },
                &CancellationToken::default(), ExecutionPhase::TrialExecution,
                || fixture.core.prepare_resolved_target_members(&plan, Some(&[fixture.employee_a]))
            ).unwrap();
            assert!(result.unwrap().is_some());
            let visits = REVIEW_SLOT_VISITS.load(Ordering::Relaxed);
            assert_eq!(visits, fixture.core.items.len() * 4);
            if let Some(previous) = observed_work { assert_eq!(usage.work, previous); }
            observed_work = Some(usage.work);
            println!("REVIEW_SCAN unrelated_units={} items={} target_count=1 item_visits={} charged_work={} max_work=256 status=accepted", unrelated, fixture.core.items.len(), visits, usage.work);
            if unrelated > 0 { assert!(visits > 256); }
        }
    }
