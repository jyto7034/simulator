use std::pin::Pin;
use std::time::Duration;

use actix_web::{get, web, FromRequest, HttpRequest, HttpResponse};
use actix_ws::{handle, Message};
use futures_util::StreamExt;
use std::future::Future;
use uuid::Uuid;

use crate::enums::phase::Phase;
use crate::enums::{COUNT_OF_MULLIGAN_CARDS, TIMEOUT};
use crate::exception::MessageProcessResult;
use crate::server::helper::{process_mulligan_completion, send_error_and_check, MessageHandler};
use crate::server::jsons::mulligan::{
    self, serialize_complete_message, serialize_deal_message, serialize_reroll_answer,
};
use crate::server::jsons::ValidationPayload;
use crate::try_send_error;
use crate::{card::types::PlayerType, exception::ServerError};

use super::types::ServerState;

#[derive(Debug, Clone, Copy)]
pub struct AuthPlayer {
    ptype: PlayerType,
    session_id: Uuid,
}

impl AuthPlayer {
    fn new(ptype: PlayerType, session_id: Uuid) -> Self {
        Self { ptype, session_id }
    }
}

impl AuthPlayer {
    fn reverse(&self) -> PlayerType {
        match self.ptype {
            PlayerType::Player1 => PlayerType::Player2,
            PlayerType::Player2 => PlayerType::Player1,
        }
    }
}

impl FromRequest for AuthPlayer {
    type Error = ServerError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();

        Box::pin(async move {
            let Some(cookie) = req.cookie("user_id") else {
                return Err(ServerError::CookieNotFound);
            };
            let Some(game_step) = req.cookie("game_step") else {
                return Err(ServerError::CookieNotFound);
            };

            let cookie = cookie.to_string().replace("user_id=", "");
            let game_step = game_step.to_string().replace("game_step=", "");

            if let Some(state) = req.app_data::<web::Data<ServerState>>() {
                let game = state.game.lock().await;
                if game.get_phase().as_str().to_lowercase() != game_step {
                    return Err(ServerError::WrongPhase(
                        game.get_phase().as_str().to_string(),
                        game_step,
                    ));
                }

                let cookie_str = cookie.to_string();
                let p1_key = state.player_cookie.0.as_str();
                let p2_key = state.opponent_cookie.0.as_str();

                let player_type = match cookie_str.as_str() {
                    key if key == p1_key => PlayerType::Player1,
                    key if key == p2_key => PlayerType::Player2,
                    _ => return Err(ServerError::InternalServerError),
                };

                // 세션 등록 (새 세션 또는 기존 세션 ID 반환)
                let session_id = state
                    .session_manager
                    .register_session(player_type, Phase::Mulligan)
                    .await;

                // 다른 엔드포인트에 이미 유효한 세션이 있는지 확인
                if !state
                    .session_manager
                    .is_valid_session(player_type, session_id, game_step.into())
                    .await
                {
                    return Err(ServerError::ActiveSessionExists(
                        "Active session exists in another phase".into(),
                    ));
                }

                Ok(AuthPlayer::new(player_type, session_id))
            } else {
                Err(ServerError::ServerStateNotFound)
            }
        })
    }
}

impl From<AuthPlayer> for PlayerType {
    fn from(value: AuthPlayer) -> Self {
        value.ptype
    }
}

impl From<AuthPlayer> for String {
    fn from(value: AuthPlayer) -> Self {
        value.ptype.into()
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
/// 서버는 플레이어가 전송한 카드를 덱의 맨 아래에 위치 시킨 뒤, 새로운 카드를 뽑아서 플레이어에게 전송합니다.
/// 이 때 서버측에서 플레이어에게 전송하는 json 규격은 아래와 같습니다.
///
/// ```
///     use serde_json::json;
///     json!
///     ({
///         "action": "complete",
///         "payload": {
///             "player": "player",
///             "cards": ["CARD_UUID_3", "CARD_UUID_4"]
///         }
///     });
/// ```
///
/// 재추첨을 요청하지 않고 카드 선택을 완료한 경우, 플레이어는 서버에게 complete 메시지를 전송합니다.
/// complete 메세지를 받은 서버는 플레이어에게 Complete json 을 전송합니다.
/// 이 때 서버측에서 플레이어에게 전송하는 json 규격은 아래와 같습니다.
///
/// ```
///     use serde_json::json;
///     json!
///     ({
///         "action": "complete",
///         "payload": {
///             "player": "player",
///             "cards": ["CARD_UUID_3", "CARD_UUID_4"]
///         }
///     });
/// ```
///
/// 재추첨 카드들은 덱의 맨 아래에 위치하게 됩니다.
/// 위 일련의 과정이 모두 완료 되면 MulliganState 의 confirm_selection() 함수를 호출하여 선택을 확정합니다.
/// 해당 함수 호출 후, 다른 플레이어의 MulliganState 의 is_ready 함수를 통해 준비 상태를 확인합니다.
/// 두 플레이어가 모두 준비되면 다음 단계로 넘어갑니다.

// TODO: 각 에러 처리 분명히 해야함.
// TODO: 네트워크 이슈가 발생하여 재연결이 필요한 경우 처리가 필요함.
#[get("/mulligan_step")]
pub async fn handle_mulligan(
    player: AuthPlayer,
    req: HttpRequest,
    payload: web::Payload,
    state: web::Data<ServerState>,
) -> Result<HttpResponse, ServerError> {
    // 멀리건 수행 중 연결이 끊힌 경우, 재진입을 허용해야 하는데, 아직 뚜렷한 방법이 떠오르진 않음.

    // 플레이어가 재진입을 시도하는 경우
    {
        let game = state.game.lock().await;
        if !game
            .get_player_by_type(player.ptype)
            .get()
            .get_mulligan_state_mut()
            .get_select_cards()
            .is_empty()
        {
            return Err(ServerError::InvalidApproach);
        }
    }

    let player_type = player.ptype;

    // Http 업그레이드: 이때 session과 stream이 반환됩니다.
    let (resp, mut session, mut stream) =
        handle(&req, payload).map_err(|_| ServerError::HandleFailed)?;

    // Mulligan deal 단계 수행 코드입니다.
    // 새로운 카드를 뽑아서 player 의 mulligan cards 에 저장 한 뒤, json 형태로 변환하여 전송합니다.
    let new_cards = {
        let mut game = state.game.lock().await;
        let cards = game.get_mulligan_cards(player_type, COUNT_OF_MULLIGAN_CARDS)?;
        let mut player = game.get_player_by_type(player_type).get();
        player
            .get_mulligan_state_mut()
            .add_select_cards(cards.clone());

        cards
    };

    let new_cards_json = serialize_deal_message(player_type, new_cards)?;
    session
        .text(new_cards_json)
        .await
        .map_err(|_| return ServerError::InternalServerError)?;

    let mut session_clone = session.clone();
    let heartbeat_session_id = player.session_id;
    let heartbeat_session_manager = state.session_manager.clone();

    // TODO: 멀리건의 경우 플레이어가 생각하는 시간이 N초 존재하므로, 하트비트의 타임아웃 부분을 수정해야할 듯 함
    // TODO: Heartbeat 타임아웃 시, session 객체를 연결을 종료해야함.
    actix_web::rt::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(TIMEOUT));

        loop {
            interval.tick().await;

            // 세션이 유효한지 확인
            if !heartbeat_session_manager
                .is_valid_session(player_type, heartbeat_session_id, Phase::Mulligan)
                .await
            {
                break;
            }

            // 하트비트 전송
            if let Err(_) = session_clone.ping(b"heartbeat").await {
                break;
            }
        }

        // 하트비트 태스크 종료시 세션 정리
        heartbeat_session_manager
            .end_session(player_type, heartbeat_session_id)
            .await;

        // TODO: 우아하게 종료해야함.
        session_clone.close(None).await.ok();
    });

    let mulligan_session_manager = state.session_manager.clone();
    let mulligan_session_id = player.session_id;

    // 이후, 스레드 내에서 클라이언트와의 상호작용을 계속하기 위해 필요한 state를 클론합니다.
    // WebSocket 메시지 수신 등 후속 처리는 별도 spawn된 작업에서 진행합니다.
    actix_web::rt::spawn(async move {
        let mut handler = MessageHandler::new();
        while let Some(data) = stream.next().await {
            match data {
                // 클라이언트에서 받은 메시지를 분석합니다.
                Ok(Message::Text(json)) => {
                    let result = handler
                        .process_message::<mulligan::ClientMessage>(
                            &mut session,
                            &json,
                            mulligan_session_id,
                            player_type,
                        )
                        .await;

                    match result {
                        MessageProcessResult::Success(msg) => {
                            match msg {
                                mulligan::ClientMessage::RerollRequest(payload) => {
                                    if !matches!(payload.player.as_str(), "player1" | "player2") {
                                        try_send_error!(session, ServerError::InvalidPlayer, retry 3);
                                    }

                                    let mut game = state.game.lock().await;
                                    let player_type = AuthPlayer::new(
                                        payload.player.clone().into(),
                                        mulligan_session_id,
                                    );

                                    // 플레이어가 이미 준비 상태인 경우
                                    if game
                                        .get_player_by_type(player_type)
                                        .get()
                                        .get_mulligan_state_mut()
                                        .is_ready()
                                    {
                                        try_send_error!(session, ServerError::InvalidApproach, retry 3);
                                    }

                                    // 플레이어가 선택한 카드가 유효한지 확인합니다.
                                    if payload.validate(
                                        game.get_player_by_type(player_type).get().get_cards(),
                                    ) == None
                                    {
                                        try_send_error!(session, ServerError::InvalidCards, retry 3);
                                    }

                                    // 기존 카드를 덱의 최하단에 위치 시킨 뒤, 새로운 카드를 뽑아서 player 의 mulligan cards 에 저장하고 json 으로 변환하여 전송합니다.
                                    let Ok(rerolled_card) = game
                                        .restore_then_reroll_mulligan_cards(
                                            player_type,
                                            payload.cards.clone(),
                                        )
                                    else {
                                        // TODO 재시도 혹은 기타 처리
                                        break;
                                    };

                                    // 플레이어가 선택한 카드를 select_cards 에서 삭제하고 Reroll 된 카드를 추가합니다.
                                    game.get_player_by_type(player_type)
                                        .get()
                                        .get_mulligan_state_mut()
                                        .remove_select_cards(payload.cards);

                                    game.get_player_by_type(player_type)
                                        .get()
                                        .get_mulligan_state_mut()
                                        .add_select_cards(rerolled_card.clone());

                                    // 멀리건 완료 단계를 수행합니다.
                                    match process_mulligan_completion(&mut game, player_type) {
                                        Ok(selected_cards) => selected_cards,
                                        Err(_) => break,
                                    };

                                    if game
                                        .get_player_by_type(player_type.reverse())
                                        .get()
                                        .get_mulligan_state_mut()
                                        .is_ready()
                                    {
                                        // TODO: 다음 단계로 넘어가는 코드 작성
                                    }

                                    let selected_cards = game
                                        .get_player_by_type(player_type)
                                        .get()
                                        .get_mulligan_state_mut()
                                        .get_select_cards();

                                    let Ok(selected_cards_json) =
                                        serialize_reroll_answer(player_type, selected_cards)
                                    else {
                                        // TODO 재시도 혹은 기타 처리
                                        break;
                                    };

                                    let Ok(_) = session.text(selected_cards_json).await else {
                                        // TODO 재시도 혹은 기타 처리
                                        break;
                                    };

                                    mulligan_session_manager
                                        .end_session(player_type, mulligan_session_id)
                                        .await;
                                }
                                mulligan::ClientMessage::Complete(payload) => {
                                    if !matches!(payload.player.as_str(), "player1" | "player2") {
                                        try_send_error!(session, ServerError::InvalidPlayer, retry 3);
                                    }
                                    let mut game = state.game.lock().await;
                                    let player_type = AuthPlayer::new(
                                        payload.player.clone().into(),
                                        mulligan_session_id,
                                    );

                                    // 이미 준비가 되어있다면 send_error_and_check 함수를 통해 에러 메시지를 전송하고 종료합니다.
                                    if game
                                        .get_player_by_type(player_type)
                                        .get()
                                        .get_mulligan_state_mut()
                                        .is_ready()
                                    {
                                        try_send_error!(session, ServerError::InvalidApproach, retry 3);
                                    }

                                    // 페이로드의 cards 를 확인하여 유효성 검사를 진행합니다.
                                    if payload.validate(
                                        game.get_player_by_type(player_type).get().get_cards(),
                                    ) == None
                                    {
                                        try_send_error!(session, ServerError::InvalidCards, retry 3);
                                    }

                                    // player 의 mulligan 상태를 완료 상태로 변경 후 상대의 mulligan 상태를 확인합니다.
                                    // 만약 상대도 완료 상태이라면, mulligan step 을 종료하고 다음 step 으로 진행합니다.
                                    let selected_cards =
                                        match process_mulligan_completion(&mut game, player_type) {
                                            Ok(selected_cards) => selected_cards,
                                            Err(_) => break,
                                        };

                                    if game
                                        .get_player_by_type(player.reverse())
                                        .get()
                                        .get_mulligan_state_mut()
                                        .is_ready()
                                    {
                                        // TODO: 다음 단계로 넘어가는 코드 작성
                                    }

                                    let Ok(_) = serialize_complete_message(player, selected_cards)
                                    else {
                                        // TODO 재시도 혹은 기타 처리
                                        break;
                                    };

                                    let Ok(_) = session.text(json).await else {
                                        // TODO 재시도 혹은 기타 처리
                                        break;
                                    };

                                    mulligan_session_manager
                                        .end_session(player_type, mulligan_session_id)
                                        .await;
                                }
                            }
                        }
                        MessageProcessResult::NeedRetry => {
                            // TODO: 코드가 좀 장황함 매크로 작성해야할듯
                            try_send_error!(session, ServerError::InvalidApproach, retry 3);
                            continue;
                        }
                        MessageProcessResult::TerminateSession(server_error) => {
                            mulligan_session_manager
                                .end_session(player_type, heartbeat_session_id)
                                .await;
                        }
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

#[get("/draw_step")]
pub async fn handle_draw(
    player: AuthPlayer,
    state: web::Data<ServerState>,
) -> Result<HttpResponse, ServerError> {
    let player_type = player.ptype;

    // 드로우 카드와 기타 필요한 정보를 얻음
    let drawn_card = {
        let mut game = state.game.lock().await;

        // 플레이어가 이미 카드를 뽑은 경우를 확인함
        if game.phase_state.has_player_completed(player_type) {
            return Err(ServerError::InvalidApproach);
        }
        // 플레이어의 드로우 완료 표시
        game.phase_state.mark_player_completed(player_type);

        // 만약 draw_card 함수가 모종의 이유로 실패한다면 completed mark 를 제거하고 에러를 반환함
        game.draw_card(player_type).inspect_err(|_| {
            game.phase_state.reset_player_completed(player_type);
        })?
    };

    // 원하는 정보를 JSON 형태로 구성
    let response_data = serde_json::json!({});

    // JSON 응답 반환
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(response_data.to_string()))
}
