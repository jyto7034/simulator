use super::*;
fn bj(h: &[u8], d: &[u8]) -> BlackjackRound {
    BlackjackRound {
        shoe: vec![],
        hands: vec![h.to_vec(), vec![10, 7], vec![10, 8], d.to_vec()],
        wager_factor: 1,
        done: true,
    }
}
#[test]
fn u163_blackjack_aces_naturals_push_and_bust() {
    assert_eq!(blackjack_total(&[1, 1, 9]), 21);
    assert_eq!(blackjack_total(&[1, 1, 13]), 12);
    assert_eq!(blackjack_total(&[1, 13, 12]), 21);
    assert_eq!(bj(&[1, 10], &[7, 7, 7]).delta_for(0, 10), 15);
    assert_eq!(bj(&[1, 10], &[1, 13]).delta_for(0, 10), 0);
    assert_eq!(bj(&[7, 7, 7], &[1, 10]).delta_for(0, 10), -10);
    assert_eq!(bj(&[10, 8], &[10, 8]).delta_for(0, 10), 0);
    assert_eq!(bj(&[10, 12, 2], &[10, 12, 3]).delta_for(0, 10), -10);
    assert_eq!(bj(&[10, 8], &[10, 12, 3]).delta_for(0, 10), 10);
    assert_eq!(
        bj(&[1, 10], &[10, 8]).delta_for(0, 3),
        4,
        "declared fractional-point rounding"
    );
}
#[test]
fn u163_blackjack_s17_double_and_private_hole() {
    let mut b = bj(&[5, 6], &[1, 6]);
    b.done = false;
    b.shoe = vec![2, 2, 2, 2];
    let mut random = GameRandom::new(1);
    let before = b.view();
    b.hands[3][1] = 9;
    assert_eq!(b.view(), before, "hole card not projected");
    b.hands[3][1] = 6;
    b.act(MiniAction::Double, &mut random).unwrap();
    assert!(b.done);
    assert_eq!(b.wager_factor, 2);
    assert_eq!(b.hands[0], [5, 6, 2]);
    assert_eq!(b.hands[3], [1, 6]);
    assert_eq!(b.delta_for(0, 20), -20);
    assert!(b.act(MiniAction::Hit, &mut random).is_err());
    let mut later = bj(&[2, 3, 4], &[10, 8]);
    later.done = false;
    assert!(later.act(MiniAction::Double, &mut random).is_err());
}
#[test]
fn u163_dice_categories_kickers_and_straights() {
    let hands = [
        [1, 2, 3, 4, 6],
        [1, 1, 2, 3, 4],
        [1, 1, 2, 2, 6],
        [2, 2, 2, 3, 5],
        [1, 2, 3, 4, 5],
        [2, 2, 2, 3, 3],
        [3, 3, 3, 3, 1],
        [4, 4, 4, 4, 4],
    ];
    for pair in hands.windows(2) {
        assert!(dice_rank(&pair[0]) < dice_rank(&pair[1]));
    }
    assert!(dice_rank(&[2, 3, 4, 5, 6]) > dice_rank(&[1, 2, 3, 4, 5]));
    assert!(dice_rank(&[3, 3, 6, 5, 4]) > dice_rank(&[3, 3, 6, 5, 2]));
    assert!(dice_rank(&[5, 5, 2, 2, 1]) > dice_rank(&[4, 4, 3, 3, 6]));
    assert_eq!(dice_rank(&[6, 3, 6, 2, 1]), dice_rank(&[2, 6, 1, 3, 6]));
}
#[test]
fn u163_dice_holds_three_roll_limit_and_tie() {
    let mut r = GameRandom::new(13);
    let mut d = DiceRound::new(&mut r).unwrap();
    let old = d.hands[0].clone();
    d.act(
        MiniAction::Roll {
            held: [true, false, true, false, true],
        },
        &mut r,
    )
    .unwrap();
    for i in [0, 2, 4] {
        assert_eq!(d.hands[0][i], old[i]);
    }
    d.act(MiniAction::Roll { held: [true; 5] }, &mut r).unwrap();
    assert!(d.done);
    assert_eq!(d.rolls[0], 3);
    assert!(d.act(MiniAction::Stand, &mut r).is_err());
    let round = MiniRound::DicePoker(DiceRound {
        hands: vec![vec![5, 5, 2, 2, 1]; 3],
        rolls: vec![3; 3],
        stopped: vec![true; 3],
        done: true,
    });
    assert_eq!(round.base_delta(10, 999).unwrap(), 0);
}
fn fp(numbers: &[u8], modifiers: &[u8], busted: bool) -> FlipPlayer {
    FlipPlayer {
        numbers: numbers.to_vec(),
        modifiers: modifiers.to_vec(),
        busted,
        ..Default::default()
    }
}
fn flip(players: Vec<FlipPlayer>) -> FlipRound {
    FlipRound {
        players,
        turn: 0,
        tasks: vec![],
        pending: None,
        done: false,
        used: vec![0; 22],
        discarded: false,
    }
}
#[test]
fn u163_flip_deck_composition_and_scoring_order() {
    let shoe = FlipShoe::default();
    assert_eq!(
        shoe.remaining.iter().map(|n| u32::from(*n)).sum::<u32>(),
        94
    );
    assert_eq!(&shoe.remaining[19..], [3, 3, 3]);
    assert_eq!(fp(&[3, 11, 5, 7, 10], &[18, 17], false).score(), 82); //36*2+10, bonus not doubled
    assert_eq!(fp(&[3, 11, 5, 7, 10, 9, 4], &[18, 17], false).score(), 123); //49*2+10+15
    assert_eq!(fp(&[0], &[15], false).score(), 6);
    assert_eq!(fp(&[], &[18], false).score(), 0);
    assert_eq!(fp(&[12, 11], &[17, 18], true).score(), 0);
}
#[test]
fn u163_flip_tied_first_rounding_and_bust_exclusion() {
    let mut f = flip(vec![
        fp(&[5], &[], false),
        fp(&[5], &[], false),
        fp(&[5], &[], false),
    ]);
    f.done = true;
    assert_eq!(f.delta(10, 20), 11);
    f.players[2].busted = true;
    assert_eq!(f.delta(10, 21), 15);
    f.players[0].busted = true;
    assert_eq!(f.delta(10, 999), -10);
    let f = flip(vec![
        fp(&[0], &[], false),
        fp(&[0], &[], true),
        fp(&[0], &[], true),
    ]);
    assert_eq!(f.delta(10, 20), 20);
}
#[test]
fn u163_flip_second_chance_is_consumed_and_freeze_not_prevented() {
    let mut f = flip(vec![
        fp(&[12], &[], false),
        fp(&[], &[], false),
        fp(&[], &[], false),
    ]);
    f.players[0].second_chance = true;
    f.used[12] = 1;
    f.used[21] = 1;
    let mut shoe = FlipShoe {
        remaining: vec![0; 22],
        discarded: vec![0; 22],
    };
    shoe.remaining[12] = 1;
    f.tasks.push(FlipTask::Draw {
        seat: 0,
        left: 1,
        deferred: vec![],
    });
    f.pump(&mut GameRandom::new(1), &mut shoe).unwrap();
    assert!(!f.players[0].busted && !f.players[0].second_chance);
    assert_eq!(f.players[0].numbers, [12]);
    assert_eq!(shoe.discarded[12], 1);
    assert_eq!(shoe.discarded[21], 1);
    f.players[0].second_chance = true;
    f.apply_target(PendingTarget { owner: 1, card: 19 }, 0)
        .unwrap();
    assert!(f.players[0].stayed && f.players[0].second_chance);
}
#[test]
fn u163_flip_three_defers_actions_and_seven_ends_immediately() {
    let mut f = flip(vec![
        fp(&[], &[], false),
        fp(&[], &[], false),
        fp(&[], &[], false),
    ]);
    let mut shoe = FlipShoe {
        remaining: vec![0; 22],
        discarded: vec![0; 22],
    };
    for i in [1, 2, 19] {
        shoe.remaining[i] = 1;
    }
    f.tasks.push(FlipTask::Draw {
        seat: 0,
        left: 3,
        deferred: vec![],
    });
    f.pump(&mut GameRandom::new(1), &mut shoe).unwrap();
    assert_eq!(f.players[0].numbers.len(), 2);
    assert_eq!(shoe.remaining.iter().sum::<u8>(), 0);
    assert_eq!(
        f.pending.unwrap().card,
        19,
        "Freeze is resolved only after all three cards"
    );
    let mut f = flip(vec![
        fp(&[0, 1, 2, 3, 4, 5], &[], false),
        fp(&[12], &[], false),
        fp(&[11], &[], false),
    ]);
    let mut shoe = FlipShoe {
        remaining: vec![0; 22],
        discarded: vec![0; 22],
    };
    shoe.remaining[6] = 1;
    f.tasks.push(FlipTask::Draw {
        seat: 0,
        left: 3,
        deferred: vec![19, 20],
    });
    f.pump(&mut GameRandom::new(1), &mut shoe).unwrap();
    assert!(f.done && f.pending.is_none() && f.tasks.is_empty());
    assert_eq!(f.players[0].score(), 36);
}
#[test]
fn u163_flip_bust_cancels_deferred_and_targets_only_active() {
    let mut f = flip(vec![
        fp(&[9], &[], false),
        fp(&[8], &[], false),
        fp(&[7], &[], false),
    ]);
    let mut shoe = FlipShoe {
        remaining: vec![0; 22],
        discarded: vec![0; 22],
    };
    shoe.remaining[9] = 1;
    f.tasks.push(FlipTask::Draw {
        seat: 0,
        left: 1,
        deferred: vec![19, 20],
    });
    f.pump(&mut GameRandom::new(9), &mut shoe).unwrap();
    assert!(f.players[0].busted && f.pending.is_none());
    f.players[1].stayed = true;
    assert_eq!(f.targets(PendingTarget { owner: 2, card: 19 }), [2]);
    assert!(f
        .apply_target(PendingTarget { owner: 2, card: 19 }, 0)
        .is_err());
    assert_eq!(f.npc_target(PendingTarget { owner: 2, card: 20 }), Some(2));
}
#[test]
fn u163_flip_extra_second_chance_and_round_shoe_conservation() {
    let mut f = flip(vec![
        fp(&[], &[], false),
        fp(&[], &[], false),
        fp(&[], &[], false),
    ]);
    f.players[0].second_chance = true;
    assert_eq!(f.targets(PendingTarget { owner: 0, card: 21 }), [1, 2]);
    f.apply_target(PendingTarget { owner: 0, card: 21 }, 1)
        .unwrap();
    assert!(f.players[1].second_chance);
    f.players[2].second_chance = true;
    assert!(f.targets(PendingTarget { owner: 0, card: 21 }).is_empty());
    let mut rng = GameRandom::new(163);
    let mut shoe = FlipShoe::default();
    for _ in 0..40 {
        let mut round = FlipRound::new(&mut rng, &mut shoe).unwrap();
        while !round.done {
            let a = if let Some(p) = round.pending {
                MiniAction::Target {
                    seat: round.targets(p)[0],
                }
            } else if round.players[0].has_card() {
                MiniAction::Stand
            } else {
                MiniAction::Hit
            };
            round.act(a, &mut rng, &mut shoe).unwrap();
        }
        assert_eq!(
            shoe.remaining
                .iter()
                .chain(shoe.discarded.iter())
                .map(|n| u32::from(*n))
                .sum::<u32>(),
            94
        );
    }
}
