//! Authored dealer terms and encounter-owned state. Prices and principal are never gambit-scaled.
use crate::game::{
    behavior::GameError,
    data::{
        relic_data::RelicDatabase,
        visual_novel_data::{VisualNovelChoiceNext, VisualNovelDatabase, VisualNovelSceneNext},
    },
    dealer_games::*,
    relic::OwnedRelic,
    relic_reward::RelicRewardGrade,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DealerOfferPolicy {
    pub id: String,
    pub relic_ids: Vec<String>,
    pub price: u32,
    pub stock: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DealerPolicy {
    pub basic_points: u32,
    pub gambit_weights: [u32; 2],
    pub minimum_wager: u32,
    pub flip_loss: u32,
    pub flip_first_bonus: u32,
    pub pledge_values: [u32; 3],
    #[serde(default = "default_pledge_value")]
    pub default_pledge_value: u32,
    pub offers: Vec<DealerOfferPolicy>,
    pub first_vn: String,
    pub repeat_vn: String,
    pub gambit_vn: String,
}
fn default_pledge_value() -> u32 {
    10
}
impl Default for DealerPolicy {
    fn default() -> Self {
        Self {
            basic_points: 30,
            gambit_weights: [1, 1],
            minimum_wager: 1,
            flip_loss: 10,
            flip_first_bonus: 20,
            pledge_values: [10, 30, 60],
            default_pledge_value: 10,
            offers: vec![],
            first_vn: "dealer_first_offer".into(),
            repeat_vn: "dealer_repeat_offer".into(),
            gambit_vn: "dealer_gambit".into(),
        }
    }
}
impl DealerPolicy {
    pub fn validate(
        &self,
        relics: &RelicDatabase,
        vns: &VisualNovelDatabase,
    ) -> Result<(), String> {
        if self.basic_points == 0
            || self.minimum_wager == 0
            || self.flip_loss == 0
            || self.pledge_values.contains(&0)
            || self.default_pledge_value == 0
            || self
                .gambit_weights
                .iter()
                .try_fold(0u32, |n, w| n.checked_add(*w))
                .is_none_or(|n| n == 0)
        {
            return Err("dealer invalid points/wager/principal/gambit weights".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for offer in &self.offers {
            crate::authoring::control::charge(1, "dealer_authoring_offer")
                .map_err(|e| format!("{e:?}"))?;
            if offer.id.trim().is_empty()
                || !ids.insert(&offer.id)
                || offer.relic_ids.is_empty()
                || offer.price == 0
                || offer.stock == 0
            {
                return Err("dealer offer id/items/price/stock invalid".into());
            }
            for id in &offer.relic_ids {
                crate::authoring::control::charge(
                    relics.relics.len() as u64 + 1,
                    "dealer_authoring_relic",
                )
                .map_err(|e| format!("{e:?}"))?;
                if relics.get_by_id(id).is_none() {
                    return Err(format!("dealer unknown relic '{id}'"));
                }
            }
        }
        for (id, choices) in [
            (&self.first_vn, ["accept", "decline"]),
            (&self.repeat_vn, ["accept", "decline"]),
            (&self.gambit_vn, ["yes", "no"]),
        ] {
            let vn = vns
                .get_by_id(id)
                .ok_or_else(|| format!("dealer unknown visual novel '{id}'"))?;
            let scene = vn
                .scene_by_id(vn.entry_scene_id.as_str())
                .ok_or("dealer VN entry missing")?;
            let VisualNovelSceneNext::Choices { choices: found } = &scene.next else {
                return Err("dealer VN must offer direct terminal choices".into());
            };
            if found.len() != 2
                || !choices.iter().all(|id| {
                    found.iter().any(|c| {
                        c.id.as_str() == *id && matches!(c.next, VisualNovelChoiceNext::End)
                    })
                })
            {
                return Err("dealer VN terminal choice contract mismatch".into());
            }
        }
        Ok(())
    }
    pub fn principal(&self, rarity: &str) -> Option<u32> {
        Some(
            RelicRewardGrade::from_rarity(rarity)
                .map_or(self.default_pledge_value, |g| self.pledge_values[g.index()]),
        )
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DealerPhase {
    Offer,
    GameSelection,
    Gambit,
    Ready,
    Playing,
    Between,
    Collateral,
    Redemption,
    Shop,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum DealerAction {
    Accept,
    Decline,
    ChooseGame { game: MiniGame, wager: i64 },
    ChooseGambit { enabled: bool },
    CancelGame,
    StartRound,
    Play { action: MiniAction },
    Forfeit,
    Continue,
    OpenCollateral,
    SelectCollateral { copies: Vec<Uuid> },
    CancelCollateral,
    ConfirmCollateral,
    Settle,
    SelectRedemption { copies: Vec<Uuid> },
    ConfirmRedemption,
    Purchase { offer_id: String },
    Leave,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PledgedRelic {
    pub copy: OwnedRelic,
    pub principal: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DealerOffer {
    pub policy: DealerOfferPolicy,
    pub remaining: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DealerRoundResult {
    pub game: MiniGame,
    pub round: u64,
    pub base_delta: i64,
    pub multiplier: u8,
    pub delta: i64,
    pub points: i64,
    pub forfeited: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DealerSession {
    pub phase: DealerPhase,
    pub revision: u64,
    pub points: i64,
    pub first_explanation: bool,
    pub assigned_multiplier: u8,
    pub gambit: Option<bool>,
    pub selected_game: Option<MiniGame>,
    pub wager: i64,
    pub round_number: u64,
    pub game: Option<MiniRound>,
    pub last_result: Option<DealerRoundResult>,
    pub escrow: Vec<PledgedRelic>,
    pub selected_copies: Vec<Uuid>,
    pub offers: Vec<DealerOffer>,
    pub random: GameRandom,
    pub flip_shoe: FlipShoe,
}
impl DealerSession {
    pub fn new(policy: &DealerPolicy, first: bool, seed: u64) -> Result<Self, GameError> {
        use rand::{Rng, SeedableRng};
        let total = policy.gambit_weights[0]
            .checked_add(policy.gambit_weights[1])
            .ok_or(GameError::InvalidAction)?;
        if total == 0 {
            return Err(GameError::InvalidAction);
        }
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let assigned_multiplier = if rng.gen_range(0..total) < policy.gambit_weights[0] {
            2
        } else {
            3
        };
        Ok(Self {
            phase: DealerPhase::Offer,
            revision: 0,
            points: i64::from(policy.basic_points),
            first_explanation: first,
            assigned_multiplier,
            gambit: None,
            selected_game: None,
            wager: 0,
            round_number: 0,
            game: None,
            last_result: None,
            escrow: vec![],
            selected_copies: vec![],
            offers: policy
                .offers
                .iter()
                .map(|p| DealerOffer {
                    policy: p.clone(),
                    remaining: p.stock,
                })
                .collect(),
            random: GameRandom::new(seed ^ 0x4445414c45524741),
            flip_shoe: FlipShoe::default(),
        })
    }
    pub fn multiplier(&self) -> u8 {
        if self.gambit == Some(true) {
            self.assigned_multiplier
        } else {
            1
        }
    }
    pub fn finish_round(&mut self, base: i64, forfeited: bool) -> Result<(), GameError> {
        let multiplier = self.multiplier();
        let delta = base
            .checked_mul(i64::from(multiplier))
            .ok_or(GameError::InvalidAction)?;
        let points = self
            .points
            .checked_add(delta)
            .ok_or(GameError::InvalidAction)?;
        self.last_result = Some(DealerRoundResult {
            game: self.selected_game.ok_or(GameError::InvalidAction)?,
            round: self.round_number,
            base_delta: base,
            multiplier,
            delta,
            points,
            forfeited,
        });
        self.points = points;
        self.phase = DealerPhase::Between;
        Ok(())
    }
    pub fn extent(&self) -> Result<usize, GameError> {
        // Shape-based serializer counting includes all variable strings, copies and bounded card/task owners.
        struct Counter(usize);
        impl std::io::Write for Counter {
            fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                self.0 = self
                    .0
                    .checked_add(b.len())
                    .ok_or_else(|| std::io::Error::other("dealer extent overflow"))?;
                Ok(b.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut c = Counter(0);
        serde_json::to_writer(&mut c, self).map_err(|_| GameError::InvalidAction)?;
        crate::authoring::control::charge(c.0 as u64, "dealer_extent")?;
        c.0.checked_mul(4)
            .and_then(|n| n.checked_add(std::mem::size_of::<Self>()))
            .ok_or(GameError::InvalidAction)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealerCopyView {
    pub copy: OwnedRelic,
    pub principal: Option<u32>,
    pub eligible: bool,
    pub reason: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealerOfferView {
    pub id: String,
    pub price: u32,
    pub remaining: u32,
    pub relics: Vec<crate::game::data::relic_data::RelicDefinition>,
    pub variant: crate::game::relic::RelicVariant,
    pub can_purchase: bool,
    pub reason: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealerView {
    pub phase: DealerPhase,
    pub game_rules: serde_json::Value,
    pub revision: u64,
    pub points: i64,
    pub first_explanation: bool,
    pub assigned_multiplier: u8,
    pub gambit: Option<bool>,
    pub applied_multiplier: u8,
    pub selected_game: Option<MiniGame>,
    pub wager: i64,
    pub maximum_loss: i64,
    pub double_maximum_loss: Option<i64>,
    pub minimum_wager: u32,
    pub flip_loss: u32,
    pub flip_first_bonus: u32,
    pub round_number: u64,
    pub round: Option<serde_json::Value>,
    pub last_result: Option<DealerRoundResult>,
    pub eligible_actions: Vec<String>,
    pub copies: Vec<DealerCopyView>,
    pub collateral: Vec<PledgedRelic>,
    pub selected_copies: Vec<Uuid>,
    pub selected_principal: i64,
    pub points_after_selection: i64,
    pub can_confirm_selection: bool,
    pub selection_reason: Option<String>,
    pub forfeited_copies: Vec<Uuid>,
    pub offers: Vec<DealerOfferView>,
    pub points_expire_on_leave: bool,
}
crate::authoring::clone_shape::copy_shape!(DealerPhase, DealerRoundResult);
crate::authoring::clone_shape::struct_shape!(DealerOfferPolicy {
    id,
    relic_ids,
    price,
    stock
});
crate::authoring::clone_shape::struct_shape!(DealerOffer { policy, remaining });
crate::authoring::clone_shape::struct_shape!(PledgedRelic { copy, principal });
crate::authoring::clone_shape::struct_shape!(DealerSession {
    phase,
    revision,
    points,
    first_explanation,
    assigned_multiplier,
    gambit,
    selected_game,
    wager,
    round_number,
    game,
    last_result,
    escrow,
    selected_copies,
    offers,
    random,
    flip_shoe
});
