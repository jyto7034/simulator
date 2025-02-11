
use actix_web::{web, FromRequest, HttpRequest, HttpResponse};
use actix_ws::{handle, Message};
use futures_util::future::{ready, Ready};
use futures_util::StreamExt;

use crate::enums::COUNT_OF_MULLIGAN_CARDS;
use crate::{card::types::PlayerType, exception::ServerError};

use super::jsons::{serialize_mulligan_complete_json, Action, MulliganMessage};
use super::server_utils::{parse_to_mulligan_msg, serialize_cards_to_mulligan_json};
use super::types::ServerState;

#[derive(Debug, Clone, Copy)]
pub struct AuthPlayer(PlayerType);

impl FromRequest for AuthPlayer {
    type Error = ServerError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        ready(Err(ServerError::NotFound))
    }
}

impl From<AuthPlayer> for PlayerType {
    fn from(value: AuthPlayer) -> Self {
        value.0
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
///     json!
///     ({
///         "step": "mulligan",
///         "action": "deal", // 혹은 "reroll", "complete" 등
///         "payload": {
///             "player": "player",
///             "cards": ["CARD_UUID_1", "CARD_UUID_2", "CARD_UUID_3", "CARD_UUID_4"]
///         }
///     });
/// ```
///
/// 멀리건 카드를 받은 플레이어는 다시 뽑을 카드를 선택하여 서버로 전송합니다.
/// 이 때 플레이어가 서버로 전송하는 json 규격은 아래와 같습니다.
///
/// ```
///     json!
///     ({
///         "step": "mulligan",
///         "action": "reroll-request", // 혹은 "reroll", "complete" 등
///         "payload": {
///             "player": "player",
///             "cards": ["CARD_UUID_1", "CARD_UUID_2"]
///         }
///     });
/// ```
///
/// 다시 뽑을 카드의 갯수만큼 덱에서 뽑고 플레이어에게 전송합니다. 이 때 전송되는 json 규격은 아래와 같습니다.
/// ```
///     json!
///     ({
///         "step": "mulligan",
///         "action": "rerolled", // 혹은 "reroll", "complete" 등
///         "payload": {
///             "player": "player",
///             "cards": ["CARD_UUID_3", "CARD_UUID_4"]
///         }
///     });
/// ```
/// 재추첨 카드들은 덱의 맨 아래에 위치하게 됩니다.
/// 위 일련의 과정이 모두 완료 되면 MulliganState 의 confirm_selection() 함수를 호출하여 선택을 확정합니다.
/// 해당 함수 호출 후, 다른 플레이어의 MulliganState 의 is_ready() 함수를 통해 준비 상태를 확인합니다.
/// 두 플레이어가 모두 준비되면 다음 단계로 넘어갑니다.
/// 
/// ##test
/// 
/// - 스레드를 두 개를 만들어서 동시에 접속하고 바로 confirm 처리하게 될 경우

// TODO: 각 에러 처리 분명히 해야함.
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
        // 새로운 카드를 뽑아서 json 형식으로 변환합니다.
        let new_cards = game.get_mulligan_cards(player, COUNT_OF_MULLIGAN_CARDS)?;
        let mut player = game.get_player_by_type(player).get_mut();
        player
            .get_mulligan_state_mut()
            .get_select_cards()
            .extend(new_cards.iter().cloned());
        let new_cards_json = serialize_cards_to_mulligan_json(new_cards)?;
        
        // 새로운 카드를 클라이언트에게 전송합니다.
        session.text(new_cards_json).await.map_err(|_| return ServerError::InternalServerError)?;
    }

    // 이후, 스레드 내에서 클라이언트와의 상호작용을 계속하기 위해 필요한 state를 클론합니다.
    let state_clone = state.clone();

    // WebSocket 메시지 수신 등 후속 처리는 별도 spawn된 작업에서 진행합니다.
    actix_web::rt::spawn(async move {
        while let Some(data) = stream.next().await {
            match data {
                Ok(Message::Text(json)) => {
                    // 클라이언트에서 받은 메시지를 분석합니다.
                    let msg = parse_to_mulligan_msg(json.to_string());
                    match msg.action {
                        Action::Reroll => {
                            let mut game = state_clone.game.lock().await;
                            
                            // 기존 카드를 덱의 맨 아래로 복구시킨 후, 새로 뽑은 카드를 json 으로 변환시킵니다.
                            let Ok(new_cards) = game.restore_then_reroll_mulligan_cards(msg.payload.player, msg.payload.cards) else { break; };
                            
                            {
                                let mut player = game.get_player_by_type(msg.payload.player).get_mut();
                                player
                                    .get_mulligan_state_mut()
                                    .get_select_cards()
                                    .extend(new_cards.iter().cloned());
                            }

                            let Ok(rerolled_cards) = serialize_cards_to_mulligan_json(new_cards) else { break; };

                            // 새로운 카드를 클라이언트에게 전송합니다.
                            let Err(_) = session.text(rerolled_cards).await else { break; };
                        }
                        Action::Complete => {
                            let game = state_clone.game.lock().await;

                            // 플레이어의 mulligan 단계 확정 작업 등 추가 로직
                            game.get_player_by_type(player)
                                .get_mut()
                                .get_mulligan_state_mut()
                                .confirm_selection();
                            if game
                                .get_player_by_type(player.0.reverse())
                                .get_mut()
                                .get_mulligan_state_mut()
                                .is_ready()
                            {
                                // 멀리건 단계 완료 json 전송
                                let Ok(json) = serialize_mulligan_complete_json() else { break; };
                                let Err(_) = session.text(json).await else { break; };
                                break;
                            }
                        }
                    }

                    // 받은 원본 메시지를 다시 클라이언트에 전송하는 예시
                    if let Err(e) = session.text(json).await {
                        eprintln!("Failed to send text message: {:?}", e);
                        break;
                    }
                }
                Ok(Message::Close(reason)) => {
                    session.close(reason).await.ok();
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(resp)
}
