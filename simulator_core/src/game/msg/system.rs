use actix::{Context, Handler, Message};

use crate::{
    exception::GameError,
    game::msg::helper::{to_card_snapshot, to_private_card_snapshot},
    player::PlayerActor,
    sync::snapshots::PlayerStateSnapshot,
    zone::zone::Zone,
};

#[derive(Message)]
#[rtype(result = "Result<PlayerStateSnapshot, GameError>")]
pub struct GetPlayerStateSnapshot;

impl Handler<GetPlayerStateSnapshot> for PlayerActor {
    type Result = Result<PlayerStateSnapshot, GameError>;

    fn handle(&mut self, _msg: GetPlayerStateSnapshot, _ctx: &mut Context<Self>) -> Self::Result {
        Ok(PlayerStateSnapshot {
            player_kind: self.player_type,
            health: self.health, // 가상의 필드
            mana: self.mana.get_current(),
            mana_max: self.mana.get_max(),
            deck_count: self.deck.len(),
            hand: self
                .hand
                .get_cards()
                .iter()
                .map(to_private_card_snapshot)
                .collect(),
            field: self
                .field
                .get_cards()
                .iter()
                .map(to_card_snapshot)
                .collect(),
            graveyard: self
                .graveyard
                .get_cards()
                .iter()
                .map(to_card_snapshot)
                .collect(),
        })
    }
}
