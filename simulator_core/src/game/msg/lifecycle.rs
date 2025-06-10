use actix::{Context, Handler, Message};
use tracing::info;

use crate::{card::types::PlayerKind, exception::{GameError, GameplayError}, game::GameActor, game::GameConfig};

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct InitializeGame(pub GameConfig);

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RemovePlayerActor {
    pub player_kind: PlayerKind,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerReady(pub PlayerKind);

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct CheckReEntry {
    pub player_type: PlayerKind,
}

impl Handler<InitializeGame> for GameActor {
    type Result = Result<(), GameError>;

    fn handle(&mut self, msg: InitializeGame, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

impl Handler<RemovePlayerActor> for GameActor {
    type Result = Result<(), GameError>;

    fn handle(&mut self, msg: RemovePlayerActor, _: &mut Self::Context) -> Self::Result {
        info!("Removing player actor: {:?}", msg.player_kind);
        let player_identity = self
            .get_player_identity_by_kind(msg.player_kind)
            .cloned()
            .ok_or_else(|| GameError::Gameplay(GameplayError::ResourceNotFound { kind: "player_identity", id: format!("{:?}", msg.player_kind) }))?;
        if let None = self.players.remove(&player_identity) {
            return Err(GameError::Gameplay(GameplayError::ResourceNotFound { kind: "player_identity", id: format!("{:?}", msg.player_kind) }));
        }
        Ok(())
    }
}

impl Handler<PlayerReady> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: PlayerReady, _: &mut Self::Context) -> Self::Result {}
}

impl Handler<CheckReEntry> for GameActor {
    type Result = Result<(), GameError>;

    fn handle(&mut self, msg: CheckReEntry, _: &mut Context<Self>) -> Self::Result {
        todo!()
    }
}
