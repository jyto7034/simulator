//! Ruleset-owned mini-games. RNG/shoe state never appears in the public view.
use crate::game::{behavior::GameError, determinism};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
fn invalid() -> GameError {
    GameError::InvalidAction
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiniGame {
    Blackjack,
    DicePoker,
    Flip7,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum MiniAction {
    Hit,
    Stand,
    Double,
    Roll { held: [bool; 5] },
    Target { seat: u8 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameRandom {
    seed: u64,
    cursor: u64,
}
impl GameRandom {
    pub fn new(seed: u64) -> Self {
        Self { seed, cursor: 0 }
    }
    fn below(&mut self, n: u32) -> Result<u32, GameError> {
        crate::authoring::control::charge(1, "dealer_random")?;
        if n == 0 {
            return Err(invalid());
        }
        let seed = determinism::seed_with_namespace(self.seed, self.cursor);
        self.cursor = self.cursor.checked_add(1).ok_or_else(invalid)?;
        Ok(rand::rngs::StdRng::seed_from_u64(seed).gen_range(0..n))
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlackjackRound {
    shoe: Vec<u8>,
    hands: Vec<Vec<u8>>,
    wager_factor: u8,
    done: bool,
}
pub fn blackjack_total(cards: &[u8]) -> u32 {
    let mut n = cards.iter().map(|c| u32::from((*c).min(10))).sum::<u32>();
    if cards.contains(&1) && n + 10 <= 21 {
        n += 10;
    }
    n
}
fn natural(cards: &[u8]) -> bool {
    cards.len() == 2 && blackjack_total(cards) == 21
}
impl BlackjackRound {
    fn draw(&mut self, rng: &mut GameRandom, seat: usize) -> Result<(), GameError> {
        let i = rng.below(self.shoe.len() as u32)? as usize;
        let card = self.shoe.remove(i);
        self.hands[seat].push(card);
        Ok(())
    }
    fn new(rng: &mut GameRandom) -> Result<Self, GameError> {
        let mut s = Self {
            shoe: (1..=13).flat_map(|n| [n; 4]).collect(),
            hands: vec![vec![]; 4],
            wager_factor: 1,
            done: false,
        };
        for _ in 0..2 {
            for p in 0..4 {
                s.draw(rng, p)?;
            }
        }
        if natural(&s.hands[0]) || natural(&s.hands[3]) {
            s.finish(rng)?;
        }
        Ok(s)
    }
    fn finish(&mut self, rng: &mut GameRandom) -> Result<(), GameError> {
        // Each NPC consults its own public hand only. The hidden dealer card is not a strategy input.
        for (seat, threshold) in [(1, 15), (2, 18)] {
            while blackjack_total(&self.hands[seat]) < threshold {
                self.draw(rng, seat)?;
            }
        }
        while blackjack_total(&self.hands[3]) < 17 {
            self.draw(rng, 3)?;
        }
        self.done = true;
        Ok(())
    }
    fn act(&mut self, act: MiniAction, rng: &mut GameRandom) -> Result<(), GameError> {
        if self.done {
            return Err(invalid());
        }
        match act {
            MiniAction::Hit => {
                self.draw(rng, 0)?;
                if blackjack_total(&self.hands[0]) >= 21 {
                    self.finish(rng)?;
                }
            }
            MiniAction::Stand => self.finish(rng)?,
            MiniAction::Double if self.hands[0].len() == 2 => {
                self.wager_factor = 2;
                self.draw(rng, 0)?;
                self.finish(rng)?;
            }
            _ => return Err(invalid()),
        }
        Ok(())
    }
    fn delta_for(&self, seat: usize, wager: i64) -> i128 {
        let h = &self.hands[seat];
        let d = &self.hands[3];
        let total = blackjack_total(h);
        let dt = blackjack_total(d);
        let wager = i128::from(wager);
        if total > 21 {
            -wager
        } else if natural(h) && !natural(d) {
            wager * 3 / 2
        } else if natural(d) && !natural(h) {
            -wager
        } else if dt > 21 || total > dt {
            wager
        } else if total == dt {
            0
        } else {
            -wager
        }
    }
    fn view(&self) -> Value {
        json!({"kind":"Blackjack","hands":&self.hands[..3],"totals":self.hands[..3].iter().map(|h|blackjack_total(h)).collect::<Vec<_>>(),"dealer_cards":if self.done{self.hands[3].clone()}else{vec![self.hands[3][0]]},"dealer_hidden_count":if self.done{0}else{self.hands[3].len()-1},"outcomes":self.done.then(||(0..3).map(|i|{let delta=self.delta_for(i,2);if delta<0{"lose"}else if delta==0{"push"}else if natural(&self.hands[i]){"blackjack"}else{"win"}}).collect::<Vec<_>>()),"done":self.done,"wager_factor":self.wager_factor,"can_double":!self.done&&self.hands[0].len()==2})
    }
}
/// Category first, then multiplicity/face kickers descending. No random input.
pub fn dice_rank(dice: &[u8]) -> [u8; 6] {
    let mut counts = [0u8; 7];
    for n in dice {
        if *n >= 1 && *n <= 6 {
            counts[*n as usize] += 1;
        }
    }
    let mut groups: Vec<(u8, u8)> = (1..=6)
        .filter(|n| counts[*n as usize] > 0)
        .map(|n| (counts[n as usize], n))
        .collect();
    groups.sort_by(|a, b| b.cmp(a));
    let c = groups.iter().map(|g| g.0).collect::<Vec<_>>();
    let straight = groups.len() == 5 && (counts[1] == 0 || counts[6] == 0);
    let rank = if c[0] == 5 {
        7
    } else if c[0] == 4 {
        6
    } else if c == [3, 2] {
        5
    } else if straight {
        4
    } else if c[0] == 3 {
        3
    } else if c == [2, 2, 1] {
        2
    } else if c[0] == 2 {
        1
    } else {
        0
    };
    let mut key = [0; 6];
    key[0] = rank;
    if straight {
        key[1] = if counts[6] > 0 { 6 } else { 5 };
    } else {
        for (i, (_, v)) in groups.iter().enumerate() {
            key[i + 1] = *v;
        }
    }
    key
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiceRound {
    hands: Vec<Vec<u8>>,
    rolls: Vec<u8>,
    stopped: Vec<bool>,
    done: bool,
}
fn dice_hold(dice: &[u8]) -> [bool; 5] {
    let mut counts = [0u8; 7];
    for n in dice {
        counts[*n as usize] += 1;
    }
    let face = (1..=6).max_by_key(|n| (counts[*n], *n)).unwrap() as u8;
    std::array::from_fn(|i| dice[i] == face)
}
impl DiceRound {
    fn roll(&mut self, p: usize, held: [bool; 5], rng: &mut GameRandom) -> Result<(), GameError> {
        if self.rolls[p] >= 3 || self.stopped[p] {
            return Err(invalid());
        }
        for (i, hold) in held.into_iter().enumerate() {
            if !hold {
                self.hands[p][i] = rng.below(6)? as u8 + 1;
            }
        }
        self.rolls[p] += 1;
        Ok(())
    }
    fn new(rng: &mut GameRandom) -> Result<Self, GameError> {
        let mut s = Self {
            hands: vec![vec![0; 5]; 3],
            rolls: vec![0; 3],
            stopped: vec![false; 3],
            done: false,
        };
        for p in 0..3 {
            s.roll(p, [false; 5], rng)?;
        }
        Ok(s)
    }
    fn npc(&mut self, p: usize, rng: &mut GameRandom) -> Result<(), GameError> {
        if self.stopped[p] || self.rolls[p] >= 3 {
            return Ok(());
        }
        if dice_rank(&self.hands[p])[0] >= if p == 1 { 2 } else { 5 } {
            self.stopped[p] = true;
        } else {
            let held = dice_hold(&self.hands[p]);
            self.roll(p, held, rng)?;
        }
        Ok(())
    }
    fn act(&mut self, act: MiniAction, rng: &mut GameRandom) -> Result<(), GameError> {
        if self.done {
            return Err(invalid());
        }
        match act {
            MiniAction::Roll { held } => self.roll(0, held, rng)?,
            MiniAction::Stand => self.stopped[0] = true,
            _ => return Err(invalid()),
        }
        for p in 1..3 {
            self.npc(p, rng)?;
        }
        if self.stopped[0] || self.rolls[0] >= 3 {
            for p in 1..3 {
                while !self.stopped[p] && self.rolls[p] < 3 {
                    self.npc(p, rng)?;
                }
            }
            self.done = true;
        }
        Ok(())
    }
    fn view(&self) -> Value {
        json!({"kind":"DicePoker","hands":self.hands,"rolls":self.rolls,"stopped":self.stopped,"ranks":self.hands.iter().map(|h|dice_rank(h)).collect::<Vec<_>>(),"done":self.done})
    }
}
// Card encoding:0..12 numbers;13..17 +2,+4,+6,+8,+10;18 x2;19 Freeze;20 FlipThree;21 SecondChance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlipShoe {
    remaining: Vec<u8>,
    discarded: Vec<u8>,
}
impl Default for FlipShoe {
    fn default() -> Self {
        let mut counts = (0u8..=12).map(|n| n.max(1)).collect::<Vec<_>>();
        counts.extend([1; 6]);
        counts.extend([3; 3]);
        Self {
            remaining: counts,
            discarded: vec![0; 22],
        }
    }
}
impl FlipShoe {
    fn draw(&mut self, rng: &mut GameRandom) -> Result<u8, GameError> {
        if self.remaining.iter().all(|n| *n == 0) {
            std::mem::swap(&mut self.remaining, &mut self.discarded);
        }
        let mut roll = rng.below(self.remaining.iter().map(|n| u32::from(*n)).sum())?;
        for (i, n) in self.remaining.iter_mut().enumerate() {
            if roll < u32::from(*n) {
                *n -= 1;
                return Ok(i as u8);
            }
            roll -= u32::from(*n);
        }
        Err(invalid())
    }
    fn discard(&mut self, card: u8) {
        self.discarded[card as usize] += 1;
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FlipPlayer {
    pub numbers: Vec<u8>,
    pub modifiers: Vec<u8>,
    pub second_chance: bool,
    pub stayed: bool,
    pub busted: bool,
    pub actions: Vec<u8>,
}
impl FlipPlayer {
    fn active(&self) -> bool {
        !self.stayed && !self.busted
    }
    pub fn score(&self) -> u32 {
        if self.busted {
            return 0;
        }
        let sum = self.numbers.iter().map(|n| u32::from(*n)).sum::<u32>();
        sum * if self.modifiers.contains(&18) { 2 } else { 1 }
            + self
                .modifiers
                .iter()
                .filter(|n| **n < 18)
                .map(|n| u32::from(*n - 12) * 2)
                .sum::<u32>()
            + if self.numbers.len() == 7 { 15 } else { 0 }
    }
    fn has_card(&self) -> bool {
        !self.numbers.is_empty()
            || !self.modifiers.is_empty()
            || self.second_chance
            || !self.actions.is_empty()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum FlipTask {
    Draw {
        seat: u8,
        left: u8,
        deferred: Vec<u8>,
    },
    Action {
        owner: u8,
        card: u8,
    },
    Turn {
        seat: u8,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct PendingTarget {
    owner: u8,
    card: u8,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlipRound {
    pub players: Vec<FlipPlayer>,
    turn: u8,
    tasks: Vec<FlipTask>,
    pending: Option<PendingTarget>,
    done: bool,
    used: Vec<u8>,
    discarded: bool,
}
impl FlipRound {
    fn new(rng: &mut GameRandom, shoe: &mut FlipShoe) -> Result<Self, GameError> {
        let mut s = Self {
            players: vec![FlipPlayer::default(); 3],
            turn: 0,
            tasks: vec![FlipTask::Turn { seat: 0 }],
            pending: None,
            done: false,
            used: vec![0; 22],
            discarded: false,
        };
        for seat in (0..3).rev() {
            s.tasks.push(FlipTask::Draw {
                seat,
                left: 1,
                deferred: vec![],
            });
        }
        s.pump(rng, shoe)?;
        Ok(s)
    }
    fn targets(&self, p: PendingTarget) -> Vec<u8> {
        (0..3)
            .filter(|i| {
                self.players[*i].active()
                    && (p.card != 21 || (!self.players[*i].second_chance && *i != p.owner as usize))
            })
            .map(|i| i as u8)
            .collect()
    }
    fn apply_target(&mut self, p: PendingTarget, seat: u8) -> Result<(), GameError> {
        if !self.targets(p).contains(&seat) {
            return Err(invalid());
        }
        match p.card {
            19 => {
                self.players[seat as usize].stayed = true;
                self.players[seat as usize].actions.push(19);
            }
            20 => {
                self.players[seat as usize].actions.push(20);
                self.tasks.push(FlipTask::Draw {
                    seat,
                    left: 3,
                    deferred: vec![],
                });
            }
            21 => self.players[seat as usize].second_chance = true,
            _ => return Err(invalid()),
        }
        Ok(())
    }
    fn npc_target(&self, p: PendingTarget) -> Option<u8> {
        // This function has no RNG/shoe input; only public tableau and active status.
        self.targets(p).into_iter().max_by_key(|seat| {
            let v = &self.players[*seat as usize];
            (
                if p.card != 21 && *seat != p.owner {
                    1
                } else {
                    0
                },
                v.numbers.len(),
                v.score(),
                std::cmp::Reverse(*seat),
            )
        })
    }
    fn finish(&mut self, shoe: &mut FlipShoe) {
        self.done = true;
        self.pending = None;
        self.tasks.clear();
        if !self.discarded {
            for (i, n) in self.used.iter().enumerate() {
                for _ in 0..*n {
                    shoe.discard(i as u8);
                }
            }
            self.discarded = true;
        }
    }
    fn ended(&self) -> bool {
        self.players.iter().all(|p| !p.active())
            || self.players.iter().any(|p| p.numbers.len() == 7)
    }
    fn pump(&mut self, rng: &mut GameRandom, shoe: &mut FlipShoe) -> Result<(), GameError> {
        loop {
            crate::authoring::control::charge(1, "dealer_flip7_progression")?;
            if self.ended() {
                self.finish(shoe);
                return Ok(());
            }
            if let Some(p) = self.pending {
                let targets = self.targets(p);
                if targets.is_empty() {
                    self.pending = None;
                    continue;
                }
                if p.owner == 0 {
                    return Ok(());
                }
                let target = self.npc_target(p).ok_or_else(invalid)?;
                self.pending = None;
                self.apply_target(p, target)?;
                continue;
            }
            let Some(task) = self.tasks.pop() else {
                return Ok(());
            };
            match task {
                FlipTask::Turn { seat } => {
                    self.turn = seat;
                    let player = &self.players[seat as usize];
                    if !player.active() {
                        self.tasks.push(FlipTask::Turn {
                            seat: (seat + 1) % 3,
                        });
                        continue;
                    }
                    if seat == 0 {
                        return Ok(());
                    }
                    let (count, score) = if seat == 1 { (3, 20) } else { (5, 35) };
                    let stay = player.has_card()
                        && (player.numbers.len() >= count || player.score() >= score);
                    self.tasks.push(FlipTask::Turn {
                        seat: (seat + 1) % 3,
                    });
                    if stay {
                        self.players[seat as usize].stayed = true;
                    } else {
                        self.tasks.push(FlipTask::Draw {
                            seat,
                            left: 1,
                            deferred: vec![],
                        });
                    }
                }
                FlipTask::Action { owner, card } => {
                    let p = PendingTarget { owner, card };
                    if self.targets(p).is_empty() {
                        if card == 21 {
                            self.used[21] -= 1;
                            shoe.discard(21);
                        }
                        continue;
                    }
                    self.pending = Some(p);
                }
                FlipTask::Draw {
                    seat,
                    left,
                    mut deferred,
                } => {
                    if !self.players[seat as usize].active() {
                        continue;
                    }
                    if left == 0 {
                        for card in deferred.into_iter().rev() {
                            self.tasks.push(FlipTask::Action { owner: seat, card });
                        }
                        continue;
                    }
                    let card = shoe.draw(rng)?;
                    self.used[card as usize] += 1;
                    let p = &mut self.players[seat as usize];
                    let mut extra_second = false;
                    match card {
                        0..=12 => {
                            if p.numbers.contains(&card) {
                                if p.second_chance {
                                    p.second_chance = false;
                                    self.used[card as usize] -= 1;
                                    self.used[21] -= 1;
                                    shoe.discard(card);
                                    shoe.discard(21);
                                } else {
                                    p.busted = true;
                                }
                            } else {
                                p.numbers.push(card);
                            }
                        }
                        13..=18 => p.modifiers.push(card),
                        19 | 20 => deferred.push(card),
                        21 => {
                            if p.second_chance {
                                extra_second = true;
                            } else {
                                p.second_chance = true;
                            }
                        }
                        _ => return Err(invalid()),
                    }
                    self.tasks.push(FlipTask::Draw {
                        seat,
                        left: left - 1,
                        deferred,
                    });
                    if extra_second {
                        self.tasks.push(FlipTask::Action {
                            owner: seat,
                            card: 21,
                        });
                    }
                }
            }
        }
    }
    fn act(
        &mut self,
        act: MiniAction,
        rng: &mut GameRandom,
        shoe: &mut FlipShoe,
    ) -> Result<(), GameError> {
        if self.done {
            return Err(invalid());
        }
        if let Some(p) = self.pending {
            let MiniAction::Target { seat } = act else {
                return Err(invalid());
            };
            if p.owner != 0 {
                return Err(invalid());
            }
            self.apply_target(p, seat)?;
            self.pending = None;
        } else {
            if self.turn != 0 || !self.players[0].active() || !self.tasks.is_empty() {
                return Err(invalid());
            }
            match act {
                MiniAction::Hit => {
                    self.tasks.push(FlipTask::Turn { seat: 1 });
                    self.tasks.push(FlipTask::Draw {
                        seat: 0,
                        left: 1,
                        deferred: vec![],
                    });
                }
                MiniAction::Stand if self.players[0].has_card() => {
                    self.players[0].stayed = true;
                    self.tasks.push(FlipTask::Turn { seat: 1 });
                }
                _ => return Err(invalid()),
            }
        }
        self.pump(rng, shoe)
    }
    fn delta(&self, wager: i64, bonus: u32) -> i64 {
        if self.players[0].busted {
            return -wager;
        }
        let score = self.players[0].score();
        let top = self
            .players
            .iter()
            .filter(|p| !p.busted)
            .map(|p| p.score())
            .max()
            .unwrap_or(0);
        let ties = self
            .players
            .iter()
            .filter(|p| !p.busted && p.score() == top)
            .count() as u32;
        i64::from(score)
            + if score == top && ties > 0 {
                i64::from(bonus / ties)
            } else {
                0
            }
    }
    fn view(&self) -> Value {
        json!({"kind":"Flip7","players":self.players,"scores":self.players.iter().map(|p|p.score()).collect::<Vec<_>>(),"turn":self.turn,"done":self.done,"pending_action":self.pending.map(|p|json!({"owner":p.owner,"card":p.card,"targets":self.targets(p)}))})
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiniRound {
    Blackjack(BlackjackRound),
    DicePoker(DiceRound),
    Flip7(FlipRound),
}
impl MiniRound {
    pub fn new(
        kind: MiniGame,
        rng: &mut GameRandom,
        shoe: &mut FlipShoe,
    ) -> Result<Self, GameError> {
        Ok(match kind {
            MiniGame::Blackjack => Self::Blackjack(BlackjackRound::new(rng)?),
            MiniGame::DicePoker => Self::DicePoker(DiceRound::new(rng)?),
            MiniGame::Flip7 => Self::Flip7(FlipRound::new(rng, shoe)?),
        })
    }
    pub fn done(&self) -> bool {
        match self {
            Self::Blackjack(r) => r.done,
            Self::DicePoker(r) => r.done,
            Self::Flip7(r) => r.done,
        }
    }
    pub fn can_double(&self) -> bool {
        matches!(self,Self::Blackjack(r) if !r.done&&r.hands[0].len()==2)
    }
    pub fn wager_factor(&self) -> u8 {
        match self {
            Self::Blackjack(r) => r.wager_factor,
            _ => 1,
        }
    }
    pub fn act(
        &mut self,
        a: MiniAction,
        rng: &mut GameRandom,
        shoe: &mut FlipShoe,
    ) -> Result<(), GameError> {
        match self {
            Self::Blackjack(r) => r.act(a, rng),
            Self::DicePoker(r) => r.act(a, rng),
            Self::Flip7(r) => r.act(a, rng, shoe),
        }
    }
    pub fn base_delta(&self, wager: i64, bonus: u32) -> Result<i64, GameError> {
        if !self.done() {
            return Err(invalid());
        }
        let wager = wager
            .checked_mul(i64::from(self.wager_factor()))
            .ok_or_else(invalid)?;
        Ok(match self {
            Self::Blackjack(r) => i64::try_from(r.delta_for(0, wager)).map_err(|_| invalid())?,
            Self::DicePoker(r) => {
                let ours = dice_rank(&r.hands[0]);
                let theirs = r.hands[1..].iter().map(|h| dice_rank(h)).max().unwrap();
                if ours > theirs {
                    wager
                } else if ours == theirs {
                    0
                } else {
                    -wager
                }
            }
            Self::Flip7(r) => r.delta(wager, bonus),
        })
    }
    pub fn view(&self) -> Value {
        let (mut view, actions) = match self {
            Self::Blackjack(r) => (
                r.view(),
                if r.done {
                    vec![]
                } else if r.hands[0].len() == 2 {
                    vec!["hit", "stand", "double"]
                } else {
                    vec!["hit", "stand"]
                },
            ),
            Self::DicePoker(r) => (
                r.view(),
                if r.done {
                    vec![]
                } else {
                    vec!["roll", "stand"]
                },
            ),
            Self::Flip7(r) => {
                let actions = if r.done {
                    vec![]
                } else if r.pending.is_some_and(|p| p.owner == 0) {
                    vec!["target"]
                } else if r.turn == 0 && r.players[0].active() && r.tasks.is_empty() {
                    if r.players[0].has_card() {
                        vec!["hit", "stand"]
                    } else {
                        vec!["hit"]
                    }
                } else {
                    vec![]
                };
                (r.view(), actions)
            }
        };
        view["allowed_actions"] = json!(actions);
        view
    }
    pub fn forfeit(&mut self, shoe: &mut FlipShoe) {
        match self {
            Self::Blackjack(r) => r.done = true,
            Self::DicePoker(r) => r.done = true,
            Self::Flip7(r) => {
                r.players[0].busted = true;
                r.finish(shoe);
            }
        }
    }
}
crate::authoring::clone_shape::copy_shape!(MiniGame, MiniAction, GameRandom, PendingTarget);
crate::authoring::clone_shape::struct_shape!(BlackjackRound {
    shoe,
    hands,
    wager_factor,
    done
});
crate::authoring::clone_shape::struct_shape!(DiceRound {
    hands,
    rolls,
    stopped,
    done
});
crate::authoring::clone_shape::struct_shape!(FlipShoe {
    remaining,
    discarded
});
crate::authoring::clone_shape::struct_shape!(FlipPlayer {
    numbers,
    modifiers,
    second_chance,
    stayed,
    busted,
    actions
});
crate::authoring::clone_shape::struct_shape!(FlipRound {
    players,
    turn,
    tasks,
    pending,
    done,
    used,
    discarded
});
crate::authoring::clone_shape::enum_shape!(FlipTask {
    units: [],
    tuples: [],
    records: [
        Draw {
            seat,
            left,
            deferred
        },
        Action { owner, card },
        Turn { seat }
    ]
});
crate::authoring::clone_shape::enum_shape!(MiniRound {
    units: [],
    tuples: [Blackjack(r), DicePoker(r), Flip7(r)],
    records: []
});
#[cfg(test)]
#[path = "dealer_games_tests.rs"]
mod tests;
