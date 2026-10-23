fn audit2_data(extra_wave: bool, reward_amount: Option<u32>, unrestrained: bool) -> ValidatedGameData {
    let mut sources = GameplayRonSourceSet::embedded();
    let mut db = FacilityTemplateDatabase::from_ron_str(sources.facility_templates).unwrap();
    let l = &mut db.layouts[0];
    l.golden_bough_plans.clear();
    for a in &mut l.activities {
        a.golden_bough_candidate = false;
    }
    l.rooms.retain(|r| r.id == "ENTRY" || r.id == "B1-01");
    l.connections
        .retain(|c| c.from == "ENTRY" && c.to == "B1-01");
    let mut activity = l.activity("early_reward").unwrap().clone();
    activity.id = "stress".into();
    activity.reward_activity = Some(RewardActivityKind::StressCombat);
    activity.first_entry_cost = Some(1);
    l.activities.push(activity);
    let r = l.rooms.iter_mut().find(|r| r.id == "B1-01").unwrap();
    r.reservations = [("2", "stress"), ("5", "save"), ("8", "stress")]
        .into_iter()
        .map(|(slot, id)| FacilityReservation {
            slot_id: MapSlotId::new(slot),
            activity_id: id.into(),
        })
        .collect();
    r.elite_slots.clear();
    r.omen_slot = None;
    for a in &mut l.allocation.sizes {
        for p in a.elite_percent_by_depth.values_mut() {
            *p = 0;
        }
    }
    if unrestrained {
        db.layouts[0].activities.iter_mut().find(|a| a.id == "stress").unwrap().first_entry_cost = Some(12);
    }
    let policy = db.layouts[0].stress_combat.as_mut().unwrap();
    for (i, round) in policy.rounds.iter_mut().enumerate() {
        round.rewards = vec![
            RewardEffect::GrantEnkephalin { amount: reward_amount.unwrap_or(10 * (i as u32 + 1)) },
            RewardEffect::GrantExperience { amount: i as u32 + 1, target: crate::game::reward::ExperienceTargetPolicy::CombatParticipants },
            RewardEffect::GrantEquipmentDust { amount: i as u32 + 1 },
        ];
    }
    if reward_amount.is_some() {
        // Four rounds of429,496,729 and the last429,496,731 sum to2,147,483,647.
        policy.rounds[4].rewards[0] = RewardEffect::GrantEnkephalin { amount: 429_496_731 };
    }
    if extra_wave { policy.rounds[0].wave_ids.push("audit_delayed_wave".into()); }
    let mut encounters: crate::game::data::pve_data::PveEncounterDatabase = ron::de::from_str(sources.pve_encounters).unwrap();
    if extra_wave {
        let e = encounters.encounters.iter_mut().find(|e|e.id == "stress_defense_v1").unwrap();
        let mut wave = e.waves[0].clone();
        wave.id = "audit_delayed_wave".into();
        wave.activation.trigger_id = wave.id.clone();
        wave.activation.primary = crate::game::data::pve_data::PveWavePrimaryTriggerData::AtTimeMs { time_ms: 30000 };
        e.waves.push(wave);
    }
    // Official catalog is empty; use a supported authored relic fixture.
    sources.relics = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/u08/relics.ron"));
    let relic_db: crate::game::data::relic_data::RelicDatabase = ron::de::from_str(sources.relics).unwrap();
    let relic_id = relic_db.relics.first().unwrap().id.clone();
    let mut equipment: crate::game::data::equipment_data::EquipmentDatabase = ron::de::from_str(sources.equipment).unwrap();
    let mut part = equipment.materials.iter().find(|m| m.id == "revival_catalyst").unwrap().clone();
    part.id = "audit2_part".into();
    part.uuid = Uuid::from_u128(164_002);
    part.material_type = crate::game::data::equipment_data::EquipmentMaterialType::AbnormalityPart;
    equipment.materials.push(part);
    let equipment_text = ron::ser::to_string(&equipment).unwrap();
    sources.equipment = &equipment_text;
    for (i, round) in db.layouts[0].stress_combat.as_mut().unwrap().rounds.iter_mut().enumerate() {
        let amount = if i == 4 { 429_496_731 } else { 429_496_729 };
        round.rewards = vec![
            if i % 2 == 0 { RewardEffect::GrantEquipmentDust { amount } }
            else { RewardEffect::GrantEquipmentMaterial { material_id: "equipment_dust".into(), amount } },
            RewardEffect::GrantEquipmentMaterial { material_id: "audit2_part".into(), amount },
        ];
        if i == 4 { round.rewards.push(RewardEffect::GrantRelic { relic_id: relic_id.clone() }); }
    }
    let encounters_text = ron::ser::to_string(&encounters).unwrap();
    sources.pve_encounters = &encounters_text;
    let text = ron::ser::to_string(&db).unwrap();
    sources.facility_templates = &text;
    // A supported authored veteran profile exercises final-round success;
    // first-round extraction separately retains the unchanged starter stats.
    let definitions = sources.employee_definitions.replace(
        "attack: 10",
        "attack: 80",
    );
    sources.employee_definitions = &definitions;
    let run_policy = if reward_amount.is_some() {
        sources.run_policy.replace("starter_enkephalin: 500", "starter_enkephalin: 0")
    } else { sources.run_policy.to_string() };
    sources.run_policy = &run_policy;
    GameDataBase::load_live_sources(sources).unwrap()
}
fn audit2_start(extra_wave: bool, reward_amount: Option<u32>, unrestrained: bool) -> (GameCore, Uuid, MapNodeId) {
    let p = Uuid::from_u128(16401);
    let mut c = GameCore::new(audit2_data(extra_wave, reward_amount, unrestrained), 16401);
    c.execute(
        p,
        PlayerBehavior::StartNewGame {
            game_mode: GameMode::Expedition,
            replace_active_run: false,
        },
    )
    .unwrap();
    for _ in 0..16 {
        let Some(vn) = c.get_run_snapshot_dto().unwrap().visual_novel else {
            break;
        };
        c.execute(
            p,
            PlayerBehavior::AdvanceVisualNovel {
                session_id: vn.session_id,
                current_scene_id: VisualNovelSceneId::new(vn.current_scene_id),
            },
        )
        .unwrap();
    }
    c.execute(
        p,
        PlayerBehavior::SelectStarterEmployees {
            employee_definition_ids: vec![
                "scout_runner".into(),
                "breach_guard".into(),
                "field_medic".into(),
            ],
            loan_copy_ids: vec![],
        },
    )
    .unwrap();
    let id = c
        .state
        .run
        .as_ref()
        .unwrap()
        .map
        .nodes
        .iter()
        .find(|n| n.slot_id.as_str() == "2")
        .unwrap()
        .id;
    let preview = c
        .current_map_view()
        .unwrap()
        .nodes
        .into_iter()
        .find(|n| n.id == id)
        .unwrap()
        .entry_preview
        .unwrap()
        .stress_combat
        .unwrap();
    assert_eq!(
        (preview.total_rounds, preview.quantity_multiplier),
        (5, if unrestrained { 2 } else { 1 })
    );
    c.execute(p, PlayerBehavior::SelectMapNode { node_id: id })
        .unwrap();
    c.execute(p, PlayerBehavior::ConfirmEnterNode).unwrap();
    if unrestrained {
        assert!(c.state.active_battle.is_none());
        assert_eq!(c.state.run.as_ref().unwrap().run_progression.deterrence, 0);
        c.execute(p, PlayerBehavior::ChooseExpeditionContinuation { choice: crate::game::expedition_lifecycle::ExpeditionContinuationChoice::Continue }).unwrap();
    }
    assert!(matches!(c.get_state(), GameState::InBattle { .. }));
    (c, p, id)
}

#[test]
fn u164_audit2_mixed_dust_parts_maximum_settlement_and_relic_quantity() {
    for unrestrained in [false,true] {
        let (mut c,p,id)=audit2_start(false,None,unrestrained);
        let battle_uuid=view(&c).battle_uuid;
        let balance=|c:&GameCore| (
            c.state.crafting_resources.equipment_dust,
            crate::game::material_economy::carried_amount(&c.state.inventory,&c.state.crafting_resources,"audit2_part"),
            c.state.relics.copies().count());
        assert_eq!(balance(&c),(0,0,0));
        deploy(&mut c,p);
        for round in 1..=5 {
            until_choice_or_result(&mut c);
            if round<5 {
                let v=view(&c);
                assert_eq!((v.battle_uuid,v.completed_rounds,v.awaiting_choice),(battle_uuid,round,true));
                assert_eq!(balance(&c),(0,0,0),"pending rewards must not be paid");
                act(&mut c,p,false).unwrap();
            }
        }
        assert!(matches!(c.get_state(),GameState::CombatResult{..}));
        let expected=if unrestrained{4_294_967_294}else{2_147_483_647};
        assert_eq!(balance(&c),(expected,expected,1));
        assert_eq!(c.state.run.as_ref().unwrap().map.node(id).unwrap().state,MapNodeState::Completed);
        let before=assets(&c);
        assert!(c.execute(p,PlayerBehavior::StressCombatAct { battle_uuid, completed_rounds:5, extract:true }).is_err());
        assert_eq!(assets(&c),before);
        let restored=restore(&c,p);
        assert_eq!(balance(&restored),(expected,expected,1));
        assert_eq!(json!(restored.state.relics),json!(c.state.relics));
        println!("AUDIT2_SETTLEMENT unrestrained={unrestrained} dust={expected} parts={expected} relics=1 checkpoint_equal=true duplicate_rejected=true");
    }
}
