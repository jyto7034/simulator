use std::sync::Arc;

use rocket::{get, http::Status, put, request::{FromRequest, Outcome}, serde::json::Json, State};
use tokio::sync::Mutex;

use crate::{card::types::PlayerType, enums::phase::Phase, exception::ServerError};

use super::types::{MulliganCards, Player, SelectedCard, ServerState};

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Player{
    type Error = ServerError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error>{
        let state = request.rocket().state::<ServerState>().expect("error");
        let current_turn = state.game.lock().await.get_turn().current_turn();
        
        let cookies = request.cookies();
        match cookies.get_private("type") {
            Some(cookie) => {
                let player_type = match cookie.value(){
                    "p1" => PlayerType::Player1,
                    "p2" => PlayerType::Player2,
                    _ => return Outcome::Error((
                        Status::BadRequest,
                        ServerError::system_error_default()
                    )),
                };

                if current_turn != player_type{
                    return Outcome::Error((
                        Status::BadRequest,
                        ServerError::bad_request_default()
                    ))
                }
                Outcome::Success(Player { player_type })
            }
            None => Outcome::Error((
                Status::Unauthorized,
                ServerError::not_authenticated_default()
            ))
        }
    }
}

async fn check_phase(phase: Phase, state: &State<Arc<Mutex<ServerState>>>) -> Result<(), ServerError> {
    let current_phase = state.lock().await.get_game().await.get_phase();
    match current_phase == phase {
        true => Ok(()),
        false => Err(ServerError::WrongPhase { current: current_phase, expected: phase })
    }
}

#[get("/get_mulligan_cards")]
pub async fn get_mulligan_cards(state: &State<Arc<Mutex<ServerState>>>) -> Result<Json<MulliganCards>, ServerError>{
    check_phase(Phase::GameStart, state).await?;
    let state = state.lock().await;
    let mut game = state.get_game_mut().await;
    
    Ok(Json(
        MulliganCards{
            uuids: vec![]
        }
    ))
}

#[put("/", data = "<selected_card>")]
pub fn select_mulligan_card(selected_card: Json<SelectedCard>){

}