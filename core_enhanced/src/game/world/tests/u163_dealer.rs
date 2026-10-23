//! Domain expectations from RULES and the U16-3 acceptance. Setup grants are internal;
//! normal authored pack / fresh-process approved host is separate evidence.
use super::*;
use crate::game::{
    data::{live_ron::GameplayRonSourceSet, relic_data::RelicDatabase, ValidatedGameData},
    dealer::*,
    dealer_games::*,
    map::*,
};
fn data(edit: impl FnOnce(&mut DealerPolicy)) -> ValidatedGameData {
    data_with_rarity(None, edit)
}
fn data_with_rarity(
    extra: Option<&str>,
    edit: impl FnOnce(&mut DealerPolicy),
) -> ValidatedGameData {
    let mut sources = GameplayRonSourceSet::embedded();
    let mut relics: RelicDatabase = ron::de::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/u08/relics.ron"
    )))
    .unwrap();
    let original = relics.relics.remove(0);
    relics.relics.clear();
    for (g, rarity) in ["common", "rare", "special"].iter().enumerate() {
        for i in 0..4 {
            let mut d = original.clone();
            d.id = format!("{rarity}_{i}");
            d.uuid = Uuid::from_u128(163000 + g as u128 * 10 + i);
            d.rarity = rarity.to_string();
            relics.relics.push(d);
        }
    }
    if let Some(rarity) = extra {
        relics
            .relics
            .iter_mut()
            .find(|d| d.id == "special_3")
            .unwrap()
            .rarity = rarity.to_string();
    }
    let mut catalog = FacilityTemplateDatabase::from_ron_str(sources.facility_templates).unwrap();
    let l = &mut catalog.layouts[0];
    l.id = "u163_authored".into();
    l.golden_bough_plans.clear();
    for a in &mut l.activities {
        a.golden_bough_candidate = false;
    }
    l.rooms.retain(|r| r.id == "ENTRY" || r.id == "B1-01");
    l.connections
        .retain(|c| c.from == "ENTRY" && c.to == "B1-01");
    let mut dealer = l.activity("early_reward").unwrap().clone();
    dealer.id = "dealer".into();
    dealer.reward_activity = Some(RewardActivityKind::Gambling);
    l.activities.push(dealer);
    let r = l.rooms.iter_mut().find(|r| r.id == "B1-01").unwrap();
    r.reservations = [("2", "save"), ("5", "dealer"), ("8", "dealer")]
        .into_iter()
        .map(|(s, a)| FacilityReservation {
            slot_id: MapSlotId::new(s),
            activity_id: a.into(),
        })
        .collect();
    r.elite_slots.clear();
    r.omen_slot = None;
    for size in &mut l.allocation.sizes {
        for w in size.elite_percent_by_depth.values_mut() {
            *w = 0;
        }
    }
    let p = l.dealer.as_mut().unwrap();
    p.gambit_weights = [0, 1];
    p.offers = vec![
        DealerOfferPolicy {
            id: "cheap".into(),
            relic_ids: vec!["common_0".into()],
            price: 2,
            stock: 2,
        },
        DealerOfferPolicy {
            id: "bundle".into(),
            relic_ids: vec!["common_1".into(), "common_2".into()],
            price: 5,
            stock: 1,
        },
        DealerOfferPolicy {
            id: "rare".into(),
            relic_ids: vec!["rare_0".into()],
            price: 15,
            stock: 1,
        },
        DealerOfferPolicy {
            id: "special".into(),
            relic_ids: vec!["special_0".into()],
            price: 30,
            stock: 1,
        },
    ];
    edit(p);
    let text = ron::ser::to_string(&catalog).unwrap();
    let defs = ron::ser::to_string(&relics).unwrap();
    sources.facility_templates = &text;
    sources.relics = &defs;
    GameDataBase::load_live_sources(sources).unwrap()
}
fn start() -> (GameCore, Uuid) {
    u13_facility::start(data(|_| {}), 16301)
}
fn node(c: &GameCore, slot: &str) -> MapNodeId {
    c.state
        .run
        .as_ref()
        .unwrap()
        .map
        .nodes
        .iter()
        .find(|n| n.slot_id.as_str() == slot)
        .unwrap()
        .id
}
fn enter(c: &mut GameCore, p: Uuid, slot: &str) {
    let id = node(c, slot);
    c.execute(p, PlayerBehavior::SelectMapNode { node_id: id })
        .unwrap();
    c.execute(p, PlayerBehavior::ConfirmEnterNode).unwrap();
}
fn session(c: &GameCore) -> &DealerSession {
    c.state
        .active_node_content
        .as_ref()
        .unwrap()
        .as_reward()
        .unwrap()
        .dealer
        .as_ref()
        .unwrap()
}
fn view(c: &GameCore) -> Value {
    c.get_selected_event_snapshot_json().unwrap().unwrap()["dealer"].clone()
}
fn roots(c: &GameCore) -> Value {
    json!({"state":non_battle_state_without_profile_fingerprint(c),"relics":c.state.relics,"crafting":c.state.crafting_resources,"uuid":c.state.uuid_manager,"save":c.export_active_run_checkpoint().unwrap().map(|s|s.core_payload_json)})
}
fn act(c: &mut GameCore, p: Uuid, a: DealerAction) {
    let revision = session(c).revision;
    c.execute(
        p,
        PlayerBehavior::DealerAct {
            revision,
            action: a,
        },
    )
    .unwrap();
}
fn reject(c: &mut GameCore, p: Uuid, a: DealerAction) {
    let before = roots(c);
    let revision = session(c).revision;
    assert!(c
        .execute(
            p,
            PlayerBehavior::DealerAct {
                revision,
                action: a
            }
        )
        .is_err());
    assert_eq!(roots(c), before);
}
fn vn(c: &mut GameCore, p: Uuid, choice: &str) {
    let v = c.get_run_snapshot_dto().unwrap().visual_novel.unwrap();
    c.execute(
        p,
        PlayerBehavior::SelectVisualNovelChoice {
            session_id: v.session_id,
            current_scene_id: VisualNovelSceneId::new(v.current_scene_id),
            choice_id: crate::game::data::visual_novel_data::VisualNovelChoiceId::new(choice),
        },
    )
    .unwrap();
}
fn choose(c: &mut GameCore, p: Uuid, game: MiniGame, wager: i64, gambit: bool) {
    act(c, p, DealerAction::ChooseGame { game, wager });
    if session(c).phase == DealerPhase::Gambit {
        vn(c, p, if gambit { "yes" } else { "no" });
    }
    assert_eq!(session(c).phase, DealerPhase::Ready);
}
fn give(c: &mut GameCore, ids: &[&str]) -> Vec<Uuid> {
    c.apply_grant_effects(
        &ids.iter()
            .map(|x| RewardEffect::GrantRelic {
                relic_id: x.to_string(),
            })
            .collect::<Vec<_>>(),
    )
    .unwrap()
    .relic_diffs
    .iter()
    .map(|c| c.copy_uuid)
    .collect()
}
fn restore(c: &GameCore, p: Uuid) -> GameCore {
    let s = c.export_active_run_checkpoint().unwrap().unwrap();
    let mut out = GameCore::new(c.game_data.clone(), 999);
    out.install_authoritative_player_profile(c.state.player_profile.clone())
        .unwrap();
    out.continue_from_active_run_checkpoint(
        p,
        &s.core_payload_json,
        s.loads_used,
        s.run_peak_certified_power,
    )
    .unwrap();
    out
}
#[test]
fn u163_offer_vn_decline_repeat_and_next_expedition_reset() {
    let (mut c, p) = start();
    enter(&mut c, p, "2");
    c.execute(p, PlayerBehavior::CompleteNode).unwrap();
    enter(&mut c, p, "5");
    let first = view(&c);
    assert_eq!(first["points"], 30);
    assert_eq!(first["first_explanation"], true);
    assert_eq!(first["assigned_multiplier"], 3);
    let before = roots(&c);
    assert_eq!(view(&c), first);
    assert_eq!(roots(&c), before);
    reject(&mut c, p, DealerAction::Accept);
    let v = c.get_run_snapshot_dto().unwrap().visual_novel.unwrap();
    assert_eq!(v.vn_id, "dealer_first_offer");
    vn(&mut c, p, "decline");
    assert_eq!(
        c.state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(node(&c, "5"))
            .unwrap()
            .state,
        MapNodeState::Completed
    );
    assert!(c.state.relics.is_empty());
    enter(&mut c, p, "8");
    assert_eq!(view(&c)["first_explanation"], false);
    assert_eq!(
        c.get_run_snapshot_dto()
            .unwrap()
            .visual_novel
            .unwrap()
            .vn_id,
        "dealer_repeat_offer"
    );
    vn(&mut c, p, "decline");
    let (mut fresh, p) = start();
    enter(&mut fresh, p, "2");
    fresh.execute(p, PlayerBehavior::CompleteNode).unwrap();
    enter(&mut fresh, p, "5");
    assert_eq!(view(&fresh)["first_explanation"], true);
}
#[test]
fn u163_gambit_cancel_repeat_negative_and_no_regrant() {
    let (mut c, p) = start();
    enter(&mut c, p, "2");
    c.execute(p, PlayerBehavior::CompleteNode).unwrap();
    enter(&mut c, p, "5");
    vn(&mut c, p, "accept");
    reject(&mut c, p, DealerAction::ChooseGambit { enabled: true });
    choose(&mut c, p, MiniGame::DicePoker, 30, true);
    act(&mut c, p, DealerAction::CancelGame);
    assert_eq!(session(&c).points, 30);
    assert_eq!(session(&c).gambit, Some(true));
    choose(&mut c, p, MiniGame::DicePoker, 30, false);
    let rev = session(&c).revision;
    act(&mut c, p, DealerAction::StartRound);
    assert_eq!(session(&c).phase, DealerPhase::Playing);
    let before = roots(&c);
    assert!(c
        .execute(
            p,
            PlayerBehavior::DealerAct {
                revision: rev,
                action: DealerAction::StartRound
            }
        )
        .is_err());
    assert_eq!(roots(&c), before);
    act(&mut c, p, DealerAction::Forfeit);
    assert_eq!(session(&c).points, -60);
    assert_eq!(session(&c).last_result.unwrap().delta, -90);
    reject(&mut c, p, DealerAction::Forfeit);
    reject(&mut c, p, DealerAction::Continue);
    act(&mut c, p, DealerAction::Settle);
    assert_eq!(session(&c).phase, DealerPhase::Shop);
    assert!(view(&c)["eligible_actions"]
        .as_array()
        .unwrap()
        .contains(&json!("leave")));
    reject(
        &mut c,
        p,
        DealerAction::Purchase {
            offer_id: "cheap".into(),
        },
    );
    act(&mut c, p, DealerAction::Leave);
}
#[test]
fn u163_all_three_games_normal_routes_and_public_round_results() {
    let (mut c, p) = start();
    enter(&mut c, p, "2");
    c.execute(p, PlayerBehavior::CompleteNode).unwrap();
    enter(&mut c, p, "5");
    vn(&mut c, p, "accept");
    let rules = view(&c)["game_rules"].clone();
    assert_eq!(rules["Blackjack"]["natural_ratio"], json!([3, 2]));
    assert_eq!(rules["DicePoker"]["max_rolls"], 3);
    assert_eq!(rules["Flip7"]["seven_bonus"], 15);
    for game in [MiniGame::Blackjack, MiniGame::DicePoker, MiniGame::Flip7] {
        choose(
            &mut c,
            p,
            game,
            if game == MiniGame::Flip7 { 0 } else { 1 },
            false,
        );
        act(&mut c, p, DealerAction::StartRound);
        let mut steps = 0;
        while session(&c).phase == DealerPhase::Playing {
            let public = view(&c);
            let action = if game == MiniGame::Flip7 {
                if let Some(target) = public["round"]["pending_action"]["targets"]
                    .as_array()
                    .and_then(|v| v.first())
                    .and_then(|v| v.as_u64())
                {
                    MiniAction::Target { seat: target as u8 }
                } else {
                    let q = &public["round"]["players"][0];
                    if q["numbers"].as_array().unwrap().is_empty()
                        && q["modifiers"].as_array().unwrap().is_empty()
                        && !q["second_chance"].as_bool().unwrap()
                        && q["actions"].as_array().unwrap().is_empty()
                    {
                        MiniAction::Hit
                    } else {
                        MiniAction::Stand
                    }
                }
            } else {
                MiniAction::Stand
            };
            act(&mut c, p, DealerAction::Play { action });
            steps += 1;
            assert!(steps < 100, "finite current-round strategy");
        }
        let r = session(&c).last_result.unwrap();
        assert_eq!(r.game, game);
        assert_eq!(r.multiplier, 1);
        assert_eq!(r.delta, r.base_delta);
        let public = view(&c);
        assert!(public["round"].get("random").is_none());
        assert!(public["round"].get("shoe").is_none());
        act(&mut c, p, DealerAction::Continue);
    }
    assert_eq!(session(&c).round_number, 3);
    act(&mut c, p, DealerAction::Settle);
    act(&mut c, p, DealerAction::Leave);
}
#[test]
fn u163_collateral_lock_partial_redeem_corrupt_preservation_and_shop() {
    let (mut c, p) = start();
    let ids = give(&mut c, &["common_0", "common_1", "rare_1", "special_1"]);
    c.apply_grant_effects(&[RewardEffect::GrantRelicCorruptionCatalyst { amount: 1 }])
        .unwrap();
    c.execute(p, PlayerBehavior::CorruptRelic { copy_uuid: ids[0] })
        .unwrap();
    let original = c.state.relics.get(ids[0]).unwrap().clone();
    enter(&mut c, p, "2");
    c.execute(p, PlayerBehavior::CompleteNode).unwrap();
    enter(&mut c, p, "5");
    vn(&mut c, p, "accept");
    choose(&mut c, p, MiniGame::DicePoker, 10, true);
    act(&mut c, p, DealerAction::StartRound);
    act(&mut c, p, DealerAction::Forfeit);
    assert_eq!(session(&c).points, 0);
    reject(&mut c, p, DealerAction::Continue);
    act(&mut c, p, DealerAction::OpenCollateral);
    reject(
        &mut c,
        p,
        DealerAction::SelectCollateral {
            copies: vec![ids[0], ids[0]],
        },
    );
    reject(
        &mut c,
        p,
        DealerAction::SelectCollateral {
            copies: vec![Uuid::nil()],
        },
    );
    act(
        &mut c,
        p,
        DealerAction::SelectCollateral {
            copies: vec![ids[0], ids[1]],
        },
    );
    assert_eq!(view(&c)["points_after_selection"], 20);
    act(&mut c, p, DealerAction::CancelCollateral);
    assert_eq!(c.state.relics.copies().count(), 4);
    assert_eq!(session(&c).points, 0);
    act(&mut c, p, DealerAction::OpenCollateral);
    act(
        &mut c,
        p,
        DealerAction::SelectCollateral {
            copies: vec![ids[1], ids[0]],
        },
    );
    act(&mut c, p, DealerAction::ConfirmCollateral);
    assert_eq!(session(&c).points, 20);
    assert_eq!(session(&c).escrow.len(), 2);
    assert!(c.state.relics.get(ids[0]).is_none());
    let owner = c.state.roster.iter().next().unwrap().uuid;
    let before = roots(&c);
    for a in [
        PlayerBehavior::EquipRelic {
            copy_uuid: ids[0],
            employee_uuid: owner,
            slot: 0,
        },
        PlayerBehavior::CorruptRelic { copy_uuid: ids[0] },
        PlayerBehavior::ExitReward,
        PlayerBehavior::ClaimReward,
        PlayerBehavior::CompleteNode,
    ] {
        assert!(c.execute(p, a).is_err());
        assert_eq!(roots(&c), before);
    }
    act(&mut c, p, DealerAction::Settle);
    reject(&mut c, p, DealerAction::Continue);
    reject(&mut c, p, DealerAction::OpenCollateral);
    act(
        &mut c,
        p,
        DealerAction::SelectRedemption {
            copies: vec![ids[0]],
        },
    );
    let v = view(&c);
    assert_eq!(v["selected_principal"], 10);
    assert_eq!(v["points_after_selection"], 10);
    assert_eq!(v["forfeited_copies"], json!([ids[1]]));
    act(&mut c, p, DealerAction::ConfirmRedemption);
    assert_eq!(c.state.relics.get(ids[0]), Some(&original));
    assert!(c.state.relics.get(ids[1]).is_none());
    assert_eq!(session(&c).points, 10);
    for offer in ["bundle", "cheap", "cheap"] {
        act(
            &mut c,
            p,
            DealerAction::Purchase {
                offer_id: offer.into(),
            },
        );
    }
    assert_eq!(session(&c).points, 1);
    reject(
        &mut c,
        p,
        DealerAction::Purchase {
            offer_id: "cheap".into(),
        },
    );
    reject(
        &mut c,
        p,
        DealerAction::Purchase {
            offer_id: "rare".into(),
        },
    );
    assert_eq!(c.state.relics.copies().count(), 7);
    let final_relics = c.state.relics.clone();
    act(&mut c, p, DealerAction::Leave);
    let fresh = restore(&c, p);
    assert_eq!(fresh.state.relics, final_relics);
    assert!(
        fresh
            .state
            .run
            .as_ref()
            .unwrap()
            .run_progression
            .dealer_introduced
    );
}
#[test]
fn u163_equipped_negative_collateral_positive_boundary_and_forfeit_all() {
    let (mut c, p) = start();
    let ids = give(&mut c, &["special_0", "common_0", "rare_0"]);
    let owner = c.state.roster.iter().next().unwrap().uuid;
    c.execute(
        p,
        PlayerBehavior::EquipRelic {
            copy_uuid: ids[2],
            employee_uuid: owner,
            slot: 0,
        },
    )
    .unwrap();
    enter(&mut c, p, "2");
    c.execute(p, PlayerBehavior::CompleteNode).unwrap();
    enter(&mut c, p, "5");
    vn(&mut c, p, "accept");
    choose(&mut c, p, MiniGame::DicePoker, 30, true);
    act(&mut c, p, DealerAction::StartRound);
    act(&mut c, p, DealerAction::Forfeit);
    act(&mut c, p, DealerAction::OpenCollateral);
    reject(
        &mut c,
        p,
        DealerAction::SelectCollateral {
            copies: vec![ids[2]],
        },
    );
    act(
        &mut c,
        p,
        DealerAction::SelectCollateral {
            copies: vec![ids[0]],
        },
    );
    assert_eq!(view(&c)["points_after_selection"], 0);
    reject(&mut c, p, DealerAction::ConfirmCollateral);
    act(
        &mut c,
        p,
        DealerAction::SelectCollateral {
            copies: vec![ids[0], ids[1]],
        },
    );
    act(&mut c, p, DealerAction::ConfirmCollateral);
    assert_eq!(session(&c).points, 10);
    act(&mut c, p, DealerAction::Settle);
    act(
        &mut c,
        p,
        DealerAction::SelectRedemption {
            copies: vec![ids[0]],
        },
    );
    assert_eq!(view(&c)["can_confirm_selection"], false);
    reject(&mut c, p, DealerAction::ConfirmRedemption);
    act(&mut c, p, DealerAction::SelectRedemption { copies: vec![] });
    act(&mut c, p, DealerAction::ConfirmRedemption);
    assert_eq!(session(&c).points, 10);
    assert_eq!(c.state.relics.copies().count(), 1);
    assert_eq!(c.state.relics.employee_slots(owner)[0], Some(ids[2]));
    act(&mut c, p, DealerAction::Leave);
}
#[test]
fn u163_incomplete_restore_rewinds_game_escrow_redemption_purchase_together() {
    let (mut c, p) = start();
    let ids = give(&mut c, &["common_0", "common_1"]);
    enter(&mut c, p, "2");
    c.execute(p, PlayerBehavior::CompleteNode).unwrap();
    let before = c.state.relics.clone();
    enter(&mut c, p, "5");
    let initial = view(&c);
    vn(&mut c, p, "accept");
    choose(&mut c, p, MiniGame::DicePoker, 1, true);
    act(&mut c, p, DealerAction::StartRound);
    let game = view(&c);
    let mut rewind = restore(&c, p);
    assert_eq!(rewind.state.relics, before);
    enter(&mut rewind, p, "5");
    assert_eq!(view(&rewind), initial);
    vn(&mut rewind, p, "accept");
    choose(&mut rewind, p, MiniGame::DicePoker, 1, true);
    act(&mut rewind, p, DealerAction::StartRound);
    assert_eq!(view(&rewind), game);
    act(&mut c, p, DealerAction::Forfeit);
    act(&mut c, p, DealerAction::OpenCollateral);
    act(
        &mut c,
        p,
        DealerAction::SelectCollateral {
            copies: ids.clone(),
        },
    );
    act(&mut c, p, DealerAction::ConfirmCollateral);
    assert!(c.state.relics.is_empty());
    assert_eq!(restore(&c, p).state.relics, before);
    act(&mut c, p, DealerAction::Settle);
    act(
        &mut c,
        p,
        DealerAction::SelectRedemption {
            copies: vec![ids[0]],
        },
    );
    act(&mut c, p, DealerAction::ConfirmRedemption);
    act(
        &mut c,
        p,
        DealerAction::Purchase {
            offer_id: "bundle".into(),
        },
    );
    assert_eq!(restore(&c, p).state.relics, before);
    act(&mut c, p, DealerAction::Leave);
    assert_eq!(restore(&c, p).state.relics, c.state.relics);
    c.execute(p, PlayerBehavior::LoadRunCheckpoint).unwrap();
    assert_eq!(c.state.relics, before);
    assert!(
        !c.state
            .run
            .as_ref()
            .unwrap()
            .run_progression
            .dealer_introduced
    );
}

#[test]
fn u163_other_authored_rarity_is_pledgeable_and_redeems_same_copy() {
    let (mut c, p) = u13_facility::start(
        data_with_rarity(Some("legendary"), |p| p.default_pledge_value = 17),
        16301,
    );
    let ids = give(&mut c, &["special_3"]);
    let original = c.state.relics.get(ids[0]).unwrap().clone();
    enter(&mut c, p, "2");
    c.execute(p, PlayerBehavior::CompleteNode).unwrap();
    enter(&mut c, p, "5");
    vn(&mut c, p, "accept");
    choose(&mut c, p, MiniGame::DicePoker, 1, false);
    act(&mut c, p, DealerAction::StartRound);
    act(&mut c, p, DealerAction::Forfeit);
    act(&mut c, p, DealerAction::OpenCollateral);
    act(
        &mut c,
        p,
        DealerAction::SelectCollateral {
            copies: ids.clone(),
        },
    );
    assert_eq!(view(&c)["selected_principal"], 17);
    act(&mut c, p, DealerAction::ConfirmCollateral);
    assert_eq!(session(&c).points, 46);
    act(&mut c, p, DealerAction::Settle);
    act(&mut c, p, DealerAction::SelectRedemption { copies: ids });
    act(&mut c, p, DealerAction::ConfirmRedemption);
    assert_eq!(c.state.relics.get(original.copy_uuid), Some(&original));
    assert_eq!(session(&c).points, 29);
}
