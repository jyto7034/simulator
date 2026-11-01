use super::{grant_state::EconomyGrantState, GameCore};
use crate::game::{
    behavior::{BehaviorResult, GameError},
    data::visual_novel_data::{VisualNovelChoiceId, VisualNovelId},
    dealer::*,
    dealer_games::*,
    enums::RewardMode,
    map::MapNodeId,
    resources::{ActiveNodeContent, GameState, RewardSessionState},
    reward::{GrantExecutionContext, RewardEffect},
    visual_novel::VisualNovelSource,
};
use uuid::Uuid;
fn invalid() -> GameError {
    GameError::InvalidAction
}
impl GameCore {
    fn dealer_policy(&self) -> Result<&DealerPolicy, GameError> {
        self.facility_layout()
            .and_then(|p| p.dealer.as_ref())
            .ok_or_else(invalid)
    }
    fn dealer_session(&self) -> Result<&RewardSessionState, GameError> {
        if !matches!(self.state.lifecycle.phase(), GameState::InReward { .. }) {
            return Err(invalid());
        }
        let r = self
            .state
            .active_node_content
            .as_ref()
            .ok_or_else(invalid)?
            .as_reward()?;
        if r.dealer.is_none() {
            return Err(invalid());
        }
        Ok(r)
    }
    pub(super) fn resolve_dealer_reward(
        &self,
        node: MapNodeId,
    ) -> Result<RewardSessionState, GameError> {
        self.ensure_relic_operation()?;
        let policy = self.dealer_policy()?;
        crate::game::relic::allocation::admit_owned_projection(policy, "dealer_entry_policy")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(65536),
            "dealer_fixed_round_workspace",
        )?;
        let first = !self.run_state()?.run_progression.dealer_introduced;
        let dealer = DealerSession::new(policy, first, self.node_seed(node, 0x5544313633444541))?;
        let mut reward = self.build_reward_session(node.0, RewardMode::ChooseOne, vec![], false);
        reward.dealer = Some(Box::new(dealer));
        Ok(reward)
    }
    pub(super) fn start_dealer_dialogue(&mut self) -> Result<(), GameError> {
        let Some(reward) = self
            .state
            .active_node_content
            .as_ref()
            .and_then(|c| c.as_reward().ok())
        else {
            return Ok(());
        };
        let Some(s) = &reward.dealer else {
            return Ok(());
        };
        let policy = self.dealer_policy()?;
        let id = match s.phase {
            DealerPhase::Offer => {
                if s.first_explanation {
                    &policy.first_vn
                } else {
                    &policy.repeat_vn
                }
            }
            DealerPhase::Gambit => &policy.gambit_vn,
            _ => return Ok(()),
        }
        .clone();
        let node_id = MapNodeId(reward.stage_uuid);
        let definition =
            self.resolve_visual_novel_session_definition(VisualNovelId::new(id), invalid)?;
        let session = self.assemble_visual_novel_session(
            definition,
            VisualNovelSource::EventNode { node_id },
            true,
        );
        self.activate_or_queue_visual_novel_session(
            session,
            super::visual_novel_flow::VisualNovelQueuePlacement::FrontOfQueue,
        );
        Ok(())
    }
    pub(super) fn dealer_visual_novel_choice(
        &self,
        session_id: &str,
        choice: &VisualNovelChoiceId,
    ) -> Result<Option<(u64, DealerAction)>, GameError> {
        let Some(r) = self
            .state
            .active_node_content
            .as_ref()
            .and_then(|c| c.as_reward().ok())
        else {
            return Ok(None);
        };
        let Some(s) = &r.dealer else {
            return Ok(None);
        };
        let vn = self
            .state
            .visual_novel
            .active
            .as_ref()
            .ok_or_else(invalid)?;
        if vn.session_id != session_id
            || !matches!(vn.source,VisualNovelSource::EventNode{node_id} if node_id.0==r.stage_uuid)
        {
            return Ok(None);
        }
        let policy = self.dealer_policy()?;
        let action = match s.phase {
            DealerPhase::Offer
                if vn.vn_id.as_str()
                    == if s.first_explanation {
                        &policy.first_vn
                    } else {
                        &policy.repeat_vn
                    } =>
            {
                match choice.as_str() {
                    "accept" => DealerAction::Accept,
                    "decline" => DealerAction::Decline,
                    _ => return Err(invalid()),
                }
            }
            DealerPhase::Gambit if vn.vn_id.as_str() == policy.gambit_vn => match choice.as_str() {
                "yes" => DealerAction::ChooseGambit { enabled: true },
                "no" => DealerAction::ChooseGambit { enabled: false },
                _ => return Err(invalid()),
            },
            _ => return Ok(None),
        };
        Ok(Some((s.revision, action)))
    }
    fn dealer_principal(&self, id: Uuid, policy: &DealerPolicy) -> Result<u32, GameError> {
        let copy = self.state.relics.get(id).ok_or_else(invalid)?;
        if self.state.relics.equipped_to(id)?.is_some() {
            return Err(invalid());
        }
        crate::authoring::control::charge(
            self.game_data.relic_data.relics.len() as u64 + 1,
            "dealer_copy_definition",
        )?;
        let d = self
            .game_data
            .relic_data
            .get_by_id(&copy.definition_id)
            .ok_or_else(invalid)?;
        policy.principal(&d.rarity).ok_or_else(invalid)
    }
    fn dealer_copy_selection(
        &self,
        ids: &[Uuid],
        s: &DealerSession,
        policy: &DealerPolicy,
        redeem: bool,
    ) -> Result<i64, GameError> {
        crate::authoring::control::charge(ids.len() as u64, "dealer_selection_scan")?;
        // Reject impossible length before allocating an index; supported payload errors are metered.
        let available = if redeem {
            s.escrow.len()
        } else {
            self.state.relics.copy_count()
        };
        if ids.len() > available {
            return Err(invalid());
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            ids.len().checked_mul(96),
            "dealer_selection_index",
        )?;
        let mut seen = std::collections::BTreeSet::new();
        let mut sum = 0i64;
        for id in ids {
            if !seen.insert(*id) {
                return Err(invalid());
            }
            let value = if redeem {
                crate::authoring::control::charge(s.escrow.len() as u64, "dealer_redemption_scan")?;
                s.escrow
                    .iter()
                    .find(|p| p.copy.copy_uuid == *id)
                    .ok_or_else(invalid)?
                    .principal
            } else {
                self.dealer_principal(*id, policy)?
            };
            sum = sum.checked_add(i64::from(value)).ok_or_else(invalid)?;
        }
        Ok(sum)
    }
    pub(super) fn handle_dealer_action(
        &mut self,
        revision: u64,
        action: DealerAction,
    ) -> Result<BehaviorResult, GameError> {
        self.with_state_transaction(move |core| core.handle_dealer_action_inner(revision, action))
    }
    fn handle_dealer_action_inner(
        &mut self,
        revision: u64,
        action: DealerAction,
    ) -> Result<BehaviorResult, GameError> {
        use DealerAction as A;
        use DealerPhase as P;
        self.ensure_relic_operation()?;
        let current = self.dealer_session()?;
        if current.dealer.as_ref().unwrap().revision != revision {
            return Err(invalid());
        }
        current.preflight_relic_activity_clone()?;
        let mut next = crate::authoring::clone_shape::controlled_clone(current)?;
        let node = MapNodeId(next.stage_uuid);
        let s = next.dealer.as_mut().unwrap();
        crate::game::relic::allocation::admit_owned_projection(
            self.dealer_policy()?,
            "dealer_policy_clone",
        )?;
        let policy = self.dealer_policy()?.clone();
        // A finite current round and bounded engine tasks; no accumulating encounter history.
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(65536),
            "dealer_round_workspace",
        )?;
        let mut complete = false;
        let mut mark_introduced = false;
        match action {
            A::Accept if s.phase == P::Offer => {
                s.phase = P::GameSelection;
                mark_introduced = true;
            }
            A::Decline if s.phase == P::Offer => {
                complete = true;
                mark_introduced = true;
                s.points = 0;
            }
            A::ChooseGame { game, wager } if s.phase == P::GameSelection && s.points > 0 => {
                let amount = if game == MiniGame::Flip7 {
                    if wager != 0 {
                        return Err(invalid());
                    }
                    i64::from(policy.flip_loss)
                } else {
                    if wager < i64::from(policy.minimum_wager) || wager > s.points {
                        return Err(invalid());
                    }
                    wager
                };
                s.selected_game = Some(game);
                s.wager = amount;
                s.game = None;
                s.phase = if s.gambit.is_none() {
                    P::Gambit
                } else {
                    P::Ready
                };
            }
            A::ChooseGambit { enabled } if s.phase == P::Gambit && s.gambit.is_none() => {
                s.gambit = Some(enabled);
                s.phase = P::Ready;
            }
            A::CancelGame if s.phase == P::Ready => {
                s.phase = P::GameSelection;
                s.selected_game = None;
                s.wager = 0;
            }
            A::StartRound if s.phase == P::Ready && s.points > 0 => {
                let kind = s.selected_game.ok_or_else(invalid)?;
                if kind != MiniGame::Flip7 && s.wager > s.points {
                    return Err(invalid());
                }
                s.round_number = s.round_number.checked_add(1).ok_or_else(invalid)?;
                s.game = Some(MiniRound::new(kind, &mut s.random, &mut s.flip_shoe)?);
                s.phase = P::Playing;
                if s.game.as_ref().unwrap().done() {
                    let delta = s
                        .game
                        .as_ref()
                        .unwrap()
                        .base_delta(s.wager, policy.flip_first_bonus)?;
                    s.finish_round(delta, false)?;
                }
            }
            A::Play { action } if s.phase == P::Playing => {
                if matches!(action, MiniAction::Double)
                    && (!s.game.as_ref().is_some_and(|g| g.can_double())
                        || s.wager.checked_mul(2).is_none_or(|w| w > s.points)
                        || s.wager
                            .checked_mul(2)
                            .and_then(|w| w.checked_mul(i64::from(s.multiplier())))
                            .is_none())
                {
                    return Err(invalid());
                }
                let game = s.game.as_mut().ok_or_else(invalid)?;
                game.act(action, &mut s.random, &mut s.flip_shoe)?;
                if game.done() {
                    let delta = game.base_delta(s.wager, policy.flip_first_bonus)?;
                    s.finish_round(delta, false)?;
                }
            }
            A::Forfeit if s.phase == P::Playing => {
                let game = s.game.as_mut().ok_or_else(invalid)?;
                let loss = -s.wager * i64::from(game.wager_factor());
                game.forfeit(&mut s.flip_shoe);
                s.finish_round(loss, true)?;
            }
            A::Continue if s.phase == P::Between && s.points > 0 => {
                s.phase = P::GameSelection;
                s.game = None;
                s.selected_game = None;
                s.wager = 0;
            }
            A::OpenCollateral if s.phase == P::Between => {
                s.phase = P::Collateral;
                s.selected_copies.clear();
            }
            A::SelectCollateral { mut copies } if s.phase == P::Collateral => {
                self.dealer_copy_selection(&copies, s, &policy, false)?;
                copies.sort();
                s.selected_copies = copies;
            }
            A::CancelCollateral if s.phase == P::Collateral => {
                s.selected_copies.clear();
                s.phase = P::Between;
            }
            A::ConfirmCollateral if s.phase == P::Collateral => {
                let sum = self.dealer_copy_selection(&s.selected_copies, s, &policy, false)?;
                let points = s.points.checked_add(sum).ok_or_else(invalid)?;
                if s.selected_copies.is_empty() || points <= 0 {
                    return Err(invalid());
                }
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    s.selected_copies
                        .len()
                        .checked_mul(std::mem::size_of::<PledgedRelic>()),
                    "dealer_pledged_vec",
                )?;
                let mut assets = EconomyGrantState::capture(&self.state)?;
                for id in &s.selected_copies {
                    let principal = self.dealer_principal(*id, &policy)?;
                    let copy = assets.relics.remove(*id)?;
                    s.escrow.push(PledgedRelic { copy, principal });
                }
                assets.commit_into(&mut self.state);
                s.points = points;
                s.selected_copies.clear();
                s.phase = P::GameSelection;
                s.game = None;
                s.selected_game = None;
                s.wager = 0;
            }
            A::Settle if matches!(s.phase, P::Between | P::GameSelection | P::Ready) => {
                s.selected_copies.clear();
                s.game = None;
                s.phase = if s.escrow.is_empty() {
                    P::Shop
                } else {
                    P::Redemption
                };
            }
            A::SelectRedemption { mut copies } if s.phase == P::Redemption => {
                self.dealer_copy_selection(&copies, s, &policy, true)?;
                copies.sort();
                s.selected_copies = copies;
            }
            A::ConfirmRedemption if s.phase == P::Redemption => {
                let sum = self.dealer_copy_selection(&s.selected_copies, s, &policy, true)?;
                if sum > 0 && s.points < sum {
                    return Err(invalid());
                }
                let mut assets = EconomyGrantState::capture(&self.state)?;
                for pledged in s.escrow.drain(..) {
                    if s.selected_copies.contains(&pledged.copy.copy_uuid) {
                        assets.relics.return_dealer_escrow(pledged.copy)?;
                    }
                }
                assets.commit_into(&mut self.state);
                s.points = s.points.checked_sub(sum).ok_or_else(invalid)?;
                s.selected_copies.clear();
                s.phase = P::Shop;
            }
            A::Purchase { offer_id } if s.phase == P::Shop => {
                crate::authoring::control::charge(s.offers.len() as u64, "dealer_offer_scan")?;
                let offer = s
                    .offers
                    .iter_mut()
                    .find(|o| o.policy.id == offer_id)
                    .ok_or_else(invalid)?;
                if offer.remaining == 0 || s.points < i64::from(offer.policy.price) {
                    return Err(invalid());
                }
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    offer
                        .policy
                        .relic_ids
                        .len()
                        .checked_mul(std::mem::size_of::<RewardEffect>()),
                    "dealer_offer_effects",
                )?;
                let effects = offer
                    .policy
                    .relic_ids
                    .iter()
                    .map(|id| RewardEffect::GrantRelic {
                        relic_id: id.clone(),
                    })
                    .collect::<Vec<_>>();
                self.ensure_relic_grant(&effects)?;
                let mut assets = EconomyGrantState::capture(&self.state)?;
                assets.grant_effects(
                    &self.game_data,
                    &self.state.skill_fragment_policy,
                    &GrantExecutionContext::default(),
                    &effects,
                    self.node_seed(node, revision),
                )?;
                assets.commit_into(&mut self.state);
                s.points -= i64::from(offer.policy.price);
                offer.remaining -= 1;
            }
            A::Leave if s.phase == P::Shop && s.escrow.is_empty() => {
                s.points = 0;
                complete = true;
            }
            _ => return Err(invalid()),
        }
        s.revision = s.revision.checked_add(1).ok_or_else(invalid)?;
        if mark_introduced {
            self.run_state_mut()?.run_progression.dealer_introduced = true;
        }
        self.state.active_node_content = Some(ActiveNodeContent::Reward(next));
        if complete {
            return self.handle_complete_node();
        }
        self.start_dealer_dialogue()?;
        let reward = self.dealer_session()?;
        Ok(BehaviorResult::DealerState {
            dealer: self.dealer_view(reward)?.ok_or_else(invalid)?,
        })
    }
    pub(super) fn dealer_view(
        &self,
        reward: &RewardSessionState,
    ) -> Result<Option<DealerView>, GameError> {
        let Some(s) = &reward.dealer else {
            return Ok(None);
        };
        if !crate::game::battle::runtime_control::has_operation_scope() {
            return crate::game::battle::runtime_control::run_deferred(|| self.dealer_view(reward));
        }
        self.ensure_relic_operation()?;
        let policy = self.dealer_policy()?;
        crate::game::relic::allocation::admit_owned_projection(s.as_ref(), "dealer_view_state")?;
        let total = self.dealer_copy_selection(
            &s.selected_copies,
            s,
            policy,
            s.phase == DealerPhase::Redemption,
        )?;
        let after = if s.phase == DealerPhase::Redemption {
            s.points.checked_sub(total)
        } else {
            s.points.checked_add(total)
        }
        .ok_or_else(invalid)?;
        let confirm = match s.phase {
            DealerPhase::Collateral => !s.selected_copies.is_empty() && after > 0,
            DealerPhase::Redemption => total == 0 || s.points >= total,
            _ => false,
        };
        let reason = match s.phase {
            DealerPhase::Collateral if s.selected_copies.is_empty() => {
                Some("select_unequipped_copies")
            }
            DealerPhase::Collateral if after <= 0 => Some("balance_must_be_positive"),
            DealerPhase::Redemption if !confirm => Some("insufficient_points"),
            _ => None,
        }
        .map(str::to_owned);
        let mut copies = Vec::new();
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.state
                .relics
                .copy_count()
                .checked_mul(std::mem::size_of::<DealerCopyView>()),
            "dealer_copy_view_vec",
        )?;
        for copy in self.state.relics.copies() {
            crate::game::relic::allocation::admit_owned_projection(copy, "dealer_copy_view")?;
            let equipped = self.state.relics.equipped_to(copy.copy_uuid)?.is_some();
            crate::authoring::control::charge(
                self.game_data.relic_data.relics.len() as u64,
                "dealer_copy_view_lookup",
            )?;
            let principal = self
                .game_data
                .relic_data
                .get_by_id(&copy.definition_id)
                .and_then(|d| policy.principal(&d.rarity));
            let reason = if equipped {
                Some("equipped_copy")
            } else if principal.is_none() {
                Some("unsupported_grade")
            } else {
                None
            };
            copies.push(DealerCopyView {
                copy: crate::authoring::clone_shape::controlled_clone(copy)?,
                principal,
                eligible: reason.is_none(),
                reason: reason.map(str::to_owned),
            });
        }
        let mut offers = Vec::new();
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            s.offers
                .len()
                .checked_mul(std::mem::size_of::<DealerOfferView>()),
            "dealer_offer_view_vec",
        )?;
        for offer in &s.offers {
            let mut relics = Vec::new();
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                offer
                    .policy
                    .relic_ids
                    .len()
                    .checked_mul(std::mem::size_of::<
                        crate::game::data::relic_data::RelicDefinition,
                    >()),
                "dealer_offer_relic_vec",
            )?;
            for id in &offer.policy.relic_ids {
                crate::authoring::control::charge(
                    self.game_data.relic_data.relics.len() as u64,
                    "dealer_offer_view_lookup",
                )?;
                let definition = self
                    .game_data
                    .relic_data
                    .get_by_id(id)
                    .ok_or_else(invalid)?;
                crate::game::relic::allocation::admit_owned_projection(
                    definition,
                    "dealer_offer_relic_view",
                )?;
                relics.push(crate::authoring::clone_shape::controlled_clone(definition)?);
            }
            let reason = if s.phase != DealerPhase::Shop {
                Some("settlement_required")
            } else if offer.remaining == 0 {
                Some("out_of_stock")
            } else if s.points < i64::from(offer.policy.price) {
                Some("insufficient_points")
            } else {
                None
            };
            offers.push(DealerOfferView {
                id: offer.policy.id.clone(),
                price: offer.policy.price,
                remaining: offer.remaining,
                relics,
                variant: crate::game::relic::RelicVariant::Normal,
                can_purchase: reason.is_none(),
                reason: reason.map(str::to_owned),
            });
        }
        let actions = match s.phase {
            DealerPhase::Offer | DealerPhase::Gambit => vec![],
            DealerPhase::GameSelection => {
                if s.points > 0 {
                    vec!["choose_game", "settle"]
                } else {
                    vec!["settle"]
                }
            }
            DealerPhase::Ready => vec!["start_round", "cancel_game", "settle"],
            DealerPhase::Playing => vec!["play", "forfeit"],
            DealerPhase::Between => {
                if s.points > 0 {
                    vec!["continue", "open_collateral", "settle"]
                } else {
                    vec!["open_collateral", "settle"]
                }
            }
            DealerPhase::Collateral => vec!["select_collateral", "cancel_collateral"],
            DealerPhase::Redemption => vec!["select_redemption"],
            DealerPhase::Shop => vec!["leave"],
        };
        let mut eligible_actions = actions.into_iter().map(str::to_owned).collect::<Vec<_>>();
        if confirm {
            eligible_actions.push(
                if s.phase == DealerPhase::Collateral {
                    "confirm_collateral"
                } else {
                    "confirm_redemption"
                }
                .into(),
            );
        }
        if offers.iter().any(|o| o.can_purchase) {
            eligible_actions.push("purchase".into());
        }
        let forfeited_copies = if s.phase == DealerPhase::Redemption {
            s.escrow
                .iter()
                .filter(|p| !s.selected_copies.contains(&p.copy.copy_uuid))
                .map(|p| p.copy.copy_uuid)
                .collect()
        } else {
            vec![]
        };
        let maximum_loss = s
            .wager
            .checked_mul(i64::from(s.multiplier()))
            .and_then(|n| n.checked_mul(i64::from(s.game.as_ref().map_or(1, |g| g.wager_factor()))))
            .ok_or_else(invalid)?;
        let double_maximum_loss = if s.game.as_ref().is_some_and(|g| g.can_double())
            && s.wager.checked_mul(2).is_some_and(|n| n <= s.points)
        {
            maximum_loss.checked_mul(2)
        } else {
            None
        };
        let mut round = s.game.as_ref().map(|g| g.view());
        if let Some(value) = round.as_mut().filter(|v| v.get("can_double").is_some()) {
            value["can_double"] = serde_json::Value::Bool(double_maximum_loss.is_some());
            if double_maximum_loss.is_none() {
                if let Some(actions) = value["allowed_actions"].as_array_mut() {
                    actions.retain(|a| a != "double");
                }
            }
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(65536),
            "dealer_rules_projection",
        )?;
        let game_rules = serde_json::json!({
            "Blackjack":{"ruleset":"single_deck_s17","cards":52,"ace_values":[1,11],"dealer_stands_at":17,"win_ratio":[1,1],"natural_ratio":[3,2],"fractional_points":"floor","tie_delta":0,"split":false,"insurance":false,"double_first_two_only":true},
            "DicePoker":{"dice":5,"sides":6,"max_rolls":3,"rank_order":["FiveKind","FourKind","FullHouse","Straight","ThreeKind","TwoPair","Pair","HighCard"],"win_ratio":[1,1],"tie_delta":0},
            "Flip7":{"ruleset":"base_3_1_single_round","cards":94,"seven_bonus":15,"score_scale":1,"first_bonus":policy.flip_first_bonus,"bust_loss":policy.flip_loss,"tied_first_bonus":"equal_integer_shares_remainder_discarded","busted_can_win_bonus":false},
            "participants":3,"npc_styles":["conservative","aggressive"],"gambit_applies_to":"round_net_delta_only"
        });
        Ok(Some(DealerView {
            phase: s.phase,
            game_rules,
            revision: s.revision,
            points: s.points,
            first_explanation: s.first_explanation,
            assigned_multiplier: s.assigned_multiplier,
            gambit: s.gambit,
            applied_multiplier: s.multiplier(),
            selected_game: s.selected_game,
            wager: s.wager,
            maximum_loss,
            double_maximum_loss,
            minimum_wager: policy.minimum_wager,
            flip_loss: policy.flip_loss,
            flip_first_bonus: policy.flip_first_bonus,
            round_number: s.round_number,
            round,
            last_result: s.last_result,
            eligible_actions,
            copies,
            collateral: crate::authoring::clone_shape::controlled_clone(&s.escrow)?,
            selected_copies: crate::authoring::clone_shape::controlled_clone(&s.selected_copies)?,
            selected_principal: total,
            points_after_selection: after,
            can_confirm_selection: confirm,
            selection_reason: reason,
            forfeited_copies,
            offers,
            points_expire_on_leave: true,
        }))
    }
}
