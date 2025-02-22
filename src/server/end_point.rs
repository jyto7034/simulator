use actix_web::{get, web, FromRequest, HttpRequest, HttpResponse};
use actix_ws::{handle, Message};
use futures_util::future::{ready, Ready};
use futures_util::StreamExt;

use crate::card::insert::{Insert, TopInsert};
use crate::enums::COUNT_OF_MULLIGAN_CARDS;
use crate::zone::zone::Zone;
use crate::{card::types::PlayerType, exception::ServerError};

use super::jsons::{
    serialize_complete_message, serialize_deal_message, serialize_reroll_anwser_message,
    MulliganMessage,
};
use super::types::ServerState;

#[derive(Debug, Clone, Copy)]
pub struct AuthPlayer(PlayerType);

// impl FromRequest for AuthPlayer {
//     type Error = ServerError;
//     type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

//     fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
//         let Some(cookie) = req.cookie("user_id")else {
//             return Box::pin(async { Err(ServerError::InternalServerError) });
//         };
//         if let Some(state) = req.app_data::<web::Data<ServerState>>() {
//             Box::pin(async move {
//                 let game = state.opponent_cookie;

//                 Ok::<AuthPlayer, ServerError>(AuthPlayer(PlayerType::Player1))
//             });
//             Box::pin(async { Err(ServerError::InternalServerError) })
//         } else {
//             Box::pin(async { Err(ServerError::InternalServerError) })
//         }
//     }
// }

impl FromRequest for AuthPlayer {
    type Error = ServerError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let Some(cookie) = req.cookie("user_id") else {
            return ready(Err(ServerError::CookieNotFound));
        };
        let cookie = cookie.to_string().replace("user_id=", "");
        if let Some(state) = req.app_data::<web::Data<ServerState>>() {
            let cookie_str = cookie.to_string();
            let p1_key = state.player_cookie.0.as_str();
            let p2_key = state.opponent_cookie.0.as_str();

            // println!("cookie: {}", cookie_str.as_str());
            // println!("p1: {}", p1_key);
            // println!("p2: {}\n", p2_key);

            match cookie_str.as_str() {
                key if key == p1_key => ready(Ok(AuthPlayer(PlayerType::Player1))),
                key if key == p2_key => ready(Ok(AuthPlayer(PlayerType::Player2))),
                _ => ready(Err(ServerError::InternalServerError)),
            }
        } else {
            ready(Err(ServerError::ServerStateNotFound))
        }
    }
}

impl From<AuthPlayer> for PlayerType {
    fn from(value: AuthPlayer) -> Self {
        value.0
    }
}

impl From<AuthPlayer> for String {
    fn from(value: AuthPlayer) -> Self {
        value.0.into()
    }
}

/// mulligan 단계를 처리하는 end point 입니다.
///
/// AuthPlayer Request Guard 을 통해 접근을 제한합니다.
///
/// 각 플레이어는 게임 시작 시 해당 end point 에 접속하여 WebSocket 연결을 수립하게 됩니다.
/// WebSocket 연결이 성공적으로 수립되면 서버측에서 get_mulligan_cards 함수를 통해 플레이어에게 멀리건 카드를 전송합니다.
/// 이 때 서버측에서 플레이어에게 전송하는 json 규격은 아래와 같습니다
///
/// ```
///     use serde_json::json;
///     json!
///     ({
///         "action": "deal",
///         "payload": {
///             "player": "player",
///             "cards": ["CARD_UUID_1", "CARD_UUID_2", "CARD_UUID_3", "CARD_UUID_4"]
///         }
///     });
/// ```
///
///
/// 멀리건 카드를 받은 플레이어는 다시 뽑을 카드를 선택하여 서버로 전송합니다.
/// 이 때 플레이어가 서버로 전송하는 json 규격은 아래와 같습니다.
///
/// ```
///     use serde_json::json;
///     json!
///     ({
///         "action": "reroll-request",
///         "payload": {
///             "player": "player",
///             "cards": ["CARD_UUID_1", "CARD_UUID_2"]
///         }
///     });
/// ```
///
///
/// 다시 뽑을 카드의 갯수만큼 덱에서 뽑고 플레이어에게 전송합니다. 이 때 전송되는 json 규격은 아래와 같습니다.
/// ```
///     use serde_json::json;
///     json!
///     ({
///         "action": "reroll-answer",
///         "payload": {
///             "player": "player",
///             "cards": ["CARD_UUID_3", "CARD_UUID_4"]
///         }
///     });
/// ```
///
/// 재추첨 카드들은 덱의 맨 아래에 위치하게 됩니다.
/// 위 일련의 과정이 모두 완료 되면 MulliganState 의 confirm_selection() 함수를 호출하여 선택을 확정합니다.
/// 해당 함수 호출 후, 다른 플레이어의 MulliganState 의 is_) 함수를 통해 준비 상태를 확인합니다.
/// 두 플레이어가 모두 준비되면 다음 단계로 넘어갑니다.

// TODO: 각 에러 처리 분명히 해야함.
#[get("/mulligan_step")]
pub async fn handle_mulligan_cards(
    player: AuthPlayer,
    req: HttpRequest,
    payload: web::Payload,
    state: web::Data<ServerState>,
) -> Result<HttpResponse, ServerError> {
    // Http 업그레이드: 이때 session과 stream이 반환됩니다.
    let (resp, mut session, mut stream) =
        handle(&req, payload).map_err(|_| ServerError::HandleFailed)?;
    {
        let mut game = state.game.lock().await;

        // Mulligan deal 단계 수행 코드입니다.
        // 새로운 카드를 뽑아서 player 의 mulligan cards 에 저장 한 뒤, json 형태로 변환하여 전송합니다.
        let new_cards = game.get_mulligan_cards(player, COUNT_OF_MULLIGAN_CARDS)?;
        let mut _player = game.get_player_by_type(player).get();
        _player
            .get_mulligan_state_mut()
            .get_select_cards()
            .extend(new_cards.iter().cloned());
        let new_cards_json = serialize_deal_message(player, new_cards)?;
        session
            .text(new_cards_json)
            .await
            .map_err(|_| return ServerError::InternalServerError)?;
    }

    // 이후, 스레드 내에서 클라이언트와의 상호작용을 계속하기 위해 필요한 state를 클론합니다.

    // WebSocket 메시지 수신 등 후속 처리는 별도 spawn된 작업에서 진행합니다.
    actix_web::rt::spawn(async move {
        while let Some(data) = stream.next().await {
            match data {
                Ok(Message::Text(json)) => {
                    // 클라이언트에서 받은 메시지를 분석합니다.
                    let msg = match serde_json::from_str::<MulliganMessage>(&json) {
                        Ok(data) => data,
                        Err(e) => {
                            // TODO 재시도 혹은 기타 처리
                            // 받은 json 이 MulliganMessage 타입이 아닌 경우.
                            eprintln!("error {}", e);
                            break;
                        }
                    };

                    match msg {
                        MulliganMessage::RerollRequest(payload) => {
                            let player_type = AuthPlayer(payload.player.into());

                            let mut game = state.game.lock().await;
                            // 기존 카드를 덱의 최하단에 위치 시킨 뒤, 새로운 카드를 뽑아서 player 의 mulligan cards 에 저장하고 json 으로 변환하여 전송합니다.
                            let Ok(rerolled_card) = game.restore_then_reroll_mulligan_cards(
                                player_type,
                                payload.cards,
                            ) else {
                                // TODO 재시도 혹은 기타 처리
                                break;
                            };

                            let mut player = game.get_player_by_type(player_type).get();
                            player
                                .get_mulligan_state_mut()
                                .get_select_cards()
                                .extend(rerolled_card.iter().cloned());
                            let Ok(rerolled_cards_json) = serialize_reroll_anwser_message(
                                player.get_player_type(),
                                rerolled_card,
                            ) else {
                                // TODO 재시도 혹은 기타 처리
                                break;
                            };

                            let Ok(_) = session.text(rerolled_cards_json).await else {
                                // TODO 재시도 혹은 기타 처리
                                break;
                            };
                        }
                        MulliganMessage::Complete(payload) => {
                            let game = state.game.lock().await;

                            // player 의 mulligan 상태를 완료 상태로 변경 후 상대의 mulligan 상태를 확인합니다.
                            // 만약 상대도 완료 상태이라면, mulligan step 을 종료하고 다음 step 으로 진행합니다.
                            let player_type = AuthPlayer(payload.player.into());

                            game.get_player_by_type(player_type)
                                .get()
                                .get_mulligan_state_mut()
                                .confirm_selection();
                            
                            if game
                                .get_player_by_type(player.0.reverse())
                                .get()
                                .get_mulligan_state_mut()
                                .is_ready()
                            {
                                let selected_cards = game.get_player_by_type(player_type).get().get_mulligan_state_mut().get_select_cards();
                                let cards = game.get_cards_by_uuid(selected_cards);
                                game.get_player_by_type(player_type).get().get_hand_mut().add_card(cards, Box::new(TopInsert)).unwrap();
                                
                                let Ok(_) = serialize_complete_message(player) else {
                                    // TODO 재시도 혹은 기타 처리
                                    break;
                                };
                                let Ok(_) = session.text(json).await else {
                                    // TODO 재시도 혹은 기타 처리
                                    break;
                                };
                            }
                        }
                        _ => todo!(),
                    }
                }
                Ok(Message::Close(reason)) => {
                    // TODO 종료 처리 확실히.
                    session.close(reason).await.ok();
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(resp)
}
