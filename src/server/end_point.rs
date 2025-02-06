use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, FromRequest};
use actix_ws::{handle, Message};
use futures_util::StreamExt;
use futures_util::future::{ready, Ready};

use crate::{card::types::PlayerType, exception::ServerError};

use super::types::ServerState;

#[derive(Debug)]
pub struct AuthPlayer(PlayerType);

impl FromRequest for AuthPlayer{
    type Error = ServerError;
    type Future = Ready<Result<Self, Self::Error>>;
    
    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        
        ready(Err(ServerError::NotFound))
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
///         "player_type": "player",
///         "operation": "get_mulligan_cards",    
///         "cards": ["CARD_UUID_1", "CARD_UUID_2", "CARD_UUID_3", "CARD_UUID_4"]
///     });
/// ```
/// 
/// 멀리건 카드를 받은 플레이어는 다시 뽑을 카드를 선택하여 서버로 전송합니다.
/// 이 때 플레이어가 서버로 전송하는 json 규격은 아래와 같습니다.
/// 
/// ```
///     json!
///     ({
///         "player_type": "player",
///         "operation": "reroll_mulligan_cards",    
///         "cards": ["CARD_UUID_1", "CARD_UUID_3"]
///     });
/// ```
/// 
/// TODO: operation 수정 해야함.
/// 다시 뽑을 카드의 갯수만큼 덱에서 뽑고 플레이어에게 전송합니다. 이 때 전송되는 json 규격은 아래와 같습니다.
/// ```
///     json!
///     ({
///         "player_type": "player",
///         "operation": "rerolled_mulligan_cards",    
///         "cards": ["CARD_UUID_6" "CARD_UUID_7"]
///     });
/// ```
/// 재추첨 카드들은 덱의 맨 아래에 위치하게 됩니다.
/// 위 일련의 과정이 모두 완료 되면 MulliganState 의 confirm_selection() 함수를 호출하여 선택을 확정합니다.
/// 해당 함수 호출 후, 다른 플레이어의 MulliganState 의 is_ready() 함수를 통해 준비 상태를 확인합니다.
/// 두 플레이어가 모두 준비되면 다음 단계로 넘어갑니다.
  
pub async fn handle_mulligan_cards(player: AuthPlayer, req: HttpRequest, payload: web::Payload, state: web::Data<ServerState>) -> Result<HttpResponse, ServerError> {
    let mut game = state.game.lock().await;
    let cards_uuid = game.get_mulligan_cards();
    
    // Http 업그레이드
    let (resp, mut session, mut stream) = handle(&req, payload).map_err(|_| return ServerError::HandleFailed).unwrap();

    // Http 스레드 생성
    actix_web::rt::spawn(async move{
        while let Some(data) = stream.next().await {
            match data {
                Ok(Message::Text(json)) => {
                    /*
                        // json_result 는 enum 형태로써, mulligan 의 상세 단계 정보를 표현한다.
                        let json_result = parsing(json);
                        match json_result{
                            ReRoll(player_type, cards: Vec<UUID>) => {
                                // 기존 카드를 넣고 새로운 카드를 뽑습니다
                                let cards: Vec<UUID> = game.reroll_mulligan_cards(player_type, cards);
                                let cards_json = cards_to_json(cards);

                                // 새로운 카드를 플레이어에게 전송합니다
                                if let Err(e) = session.text(cards_json).await {
                                    eprintln!("Failed to send text message: {:?}", e);
                                    break;
                                }
                                
                                // 그런 뒤, 해당 Player 의 mulligan 단계를 확정짓고 다른 플레이어의 확정을 확인 후 대기 합니다.
                                game.get_player().get_mulligan_state().confirm_selection();
                                if game.get_oppoent().get_mulligan_state().is_ready() {
                                    // Mulligan 단계 종료
                                }
                            }
                        }
                    */
                    if let Err(e) = session.text(json).await {
                        eprintln!("Failed to send text message: {:?}", e);
                        break;
                    }
                }
                Ok(Message::Close(reason)) => {
                    session.close(reason).await.ok();
                    break;
                },
                _ => {}
            }
        }
    });
    Ok(resp)
}