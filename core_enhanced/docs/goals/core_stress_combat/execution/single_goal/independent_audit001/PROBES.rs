// Audit-local only. Shared setup is adapted from the existing u164 fixture.
fn audit_data(extra_wave: bool, reward_amount: Option<u32>, unrestrained: bool) -> ValidatedGameData {
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
    GameDataBase::load_live_sources(sources).unwrap()
}
fn audit_start(extra_wave: bool, reward_amount: Option<u32>, unrestrained: bool) -> (GameCore, Uuid, MapNodeId) {
    let p = Uuid::from_u128(16401);
    let mut c = GameCore::new(audit_data(extra_wave, reward_amount, unrestrained), 16401);
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
fn u164_independent_multiwave_waits_for_delayed_member() {
    let (mut c,p,_) = audit_start(true,None,false);
    deploy(&mut c,p);
    for _ in 0..80 { c.advance_active_battle_by(250).unwrap(); }
    assert_eq!(view(&c).completed_rounds,0,"first group clear cannot complete the two-wave round");
    assert!(!view(&c).awaiting_choice);
    until_choice_or_result(&mut c);
    assert_eq!(view(&c).completed_rounds,1);
    assert!(view(&c).awaiting_choice);
    assert!(c.state.active_battle.as_ref().unwrap().execution.last_event_time_ms() >= 30000);
    let before=c.state.enkephalin.amount;
    act(&mut c,p,true).unwrap();
    assert_eq!(c.state.enkephalin.amount,before+10);
}
#[test]
fn u164_independent_public_exhaustion_and_cumulative_xp_dust() {
    for unrestrained in [false,true] {
        let (mut c,p,_) = audit_start(false,None,unrestrained);
        let before=c.state.enkephalin.amount;
        let dust=c.state.crafting_resources.equipment_dust;
        let xp:Vec<_>=c.state.roster.iter().map(|e|(e.uuid,e.employee_definition_id.clone(),e.experience)).collect();
        deploy(&mut c,p);
        for round in 1..=5 {
            until_choice_or_result(&mut c);
            if round<5 {
                assert_eq!(view(&c).completed_rounds,round);
                assert_eq!(c.state.enkephalin.amount,before);
                assert_eq!(c.state.crafting_resources.equipment_dust,dust);
                for (id,_,before_xp) in &xp { assert_eq!(c.state.roster.get(id).unwrap().experience,*before_xp); }
                act(&mut c,p,false).unwrap();
            }
        }
        let multiplier=if unrestrained{2}else{1};
        assert_eq!(c.state.enkephalin.amount,before+150*multiplier);
        assert_eq!(c.state.crafting_resources.equipment_dust,dust+15*multiplier);
        for (id,definition,before_xp) in xp {
            assert_eq!(c.state.roster.get(&id).unwrap().experience-before_xp,
                if definition=="field_medic"{0}else{25*multiplier});
        }
    }
}
#[test]
fn u164_independent_pending_then_manual_retreat() {
    let (mut c,p,id)=audit_start(false,None,true);
    let before=assets(&c);
    deploy(&mut c,p);
    until_choice_or_result(&mut c);
    assert_eq!(view(&c).completed_rounds,1);
    act(&mut c,p,false).unwrap();
    execute_current_battle_mutation(&mut c,p,PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(assets(&c),before);
    assert_eq!(c.state.run.as_ref().unwrap().map.node(id).unwrap().state,MapNodeState::Completed);
    let fresh=restore(&c,p);
    assert_eq!(assets(&fresh),before);
    assert_eq!(fresh.state.run.as_ref().unwrap().map.node(id).unwrap().state,MapNodeState::Completed);
}
#[test]
fn u164_independent_cumulative_authored_rewards_are_payable() {
    // Each amount passes the fixed x2 check, but even an empty u32 wallet cannot
    // hold the cumulative five-round award. Supported authoring, no state edits.
    let (mut c,p,_) = audit_start(false,Some(1_000_000_000),false);
    deploy(&mut c,p);
    for round in 1..=5 {
        until_choice_or_result(&mut c);
        if round<5 {
            assert_eq!(view(&c).completed_rounds,round);
            act(&mut c,p,false).unwrap();
        }
    }
    assert!(matches!(c.get_state(),GameState::CombatResult{..}));
}
