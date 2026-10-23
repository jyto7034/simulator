fn audit2_xp_data(extra_wave: bool, reward_amount: Option<u32>, unrestrained: bool) -> ValidatedGameData {
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
    for (i,round) in db.layouts[0].stress_combat.as_mut().unwrap().rounds.iter_mut().enumerate() {
        round.rewards = if i == 0 { vec![RewardEffect::GrantExperience {
            amount: 2_147_483_647, target: crate::game::reward::ExperienceTargetPolicy::CombatParticipants,
        }] } else { vec![] };
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
    let mut run_policy: crate::game::data::run_policy_data::RunPolicyData = ron::de::from_str(sources.run_policy).unwrap();
    assert_eq!(run_policy.growth.post_battle_survival_xp,10);
    run_policy.growth.xp_required_per_level = vec![1_000_000_000];
    let run_policy_text = ron::ser::to_string(&run_policy).unwrap();
    sources.run_policy = &run_policy_text;
    GameDataBase::load_live_sources(sources).unwrap()
}
fn audit2_xp_start(extra_wave: bool, reward_amount: Option<u32>, unrestrained: bool) -> (GameCore, Uuid, MapNodeId) {
    let p = Uuid::from_u128(16401);
    let mut c = GameCore::new(audit2_xp_data(extra_wave, reward_amount, unrestrained), 16401);
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
fn u164_audit2_xp_quantity_preserves_survival_and_authored_grants() {
    for unrestrained in [false,true] {
        let (mut c,p,_)=audit2_xp_start(false,None,unrestrained);
        for e in c.state.roster.iter() { assert_eq!((e.level,e.experience),(1,0)); }
        deploy(&mut c,p);
        until_choice_or_result(&mut c);
        assert_eq!(view(&c).completed_rounds,1);
        for e in c.state.roster.iter() { assert_eq!((e.level,e.experience),(1,0)); }
        act(&mut c,p,true).unwrap();
        assert!(matches!(c.get_state(),GameState::CombatResult{..}));
        let expected=if unrestrained{4_294_967_314u64}else{2_147_483_657u64};
        for e in c.state.roster.iter() {
            let actual=u64::from(e.level-1)*1_000_000_000 + u64::from(e.experience);
            if e.employee_definition_id=="field_medic" { assert_eq!(actual,0); }
            else {
                println!("AUDIT2_XP unrestrained={unrestrained} employee={} level={} remaining={} actual={actual} expected={expected}",e.employee_definition_id,e.level,e.experience);
                assert_eq!(actual,expected,"survival + authored XP must be conserved");
            }
        }
    }
}
