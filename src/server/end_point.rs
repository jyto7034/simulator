use std::pin::Pin;
use std::time::{Duration, Instant};

use actix_web::{get, web, FromRequest, HttpRequest, HttpResponse};
use actix_ws::{handle, CloseCode, CloseReason, Message};
use futures_util::StreamExt;
use serde_json::json;
use std::future::Future;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::enums::{CLIENT_TIMEOUT, COUNT_OF_MULLIGAN_CARDS, HEARTBEAT_INTERVAL};
use crate::exception::MessageProcessResult;
use crate::game::game_step::PlayCardResult;
use crate::server::helper::{send_error_and_check, MessageHandler};
use crate::server::jsons::draw::serialize_draw_answer_message;
use crate::server::jsons::mulligan::{
    self, serialize_complete_message, serialize_deal_message, serialize_reroll_answer,
};
use crate::server::jsons::{main_phase1, ValidationPayload};
use crate::{card::types::PlayerType, exception::GameError};
use crate::{try_send_error, StringUuidExt, VecStringExt};

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
    type Error = GameError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            debug!("AuthPlayer::from_request 시작: 인증 처리 중...");

            let Some(player_name) = req.cookie("user_id") else {
                error!("쿠키 누락: 'user_id' 쿠키를 찾을 수 없음");
                return Err(GameError::CookieNotFound);
            };
            let Some(game_step) = req.cookie("game_step") else {
                error!("쿠키 누락: 'game_step' 쿠키를 찾을 수 없음");
                return Err(GameError::CookieNotFound);
            };

            let player_name = player_name.to_string().replace("user_id=", "");
            let game_step = game_step.to_string().replace("game_step=", "");
            debug!(
                "쿠키 파싱 완료: player_name={}, game_step={}",
                player_name, game_step
            );

            if let Some(state) = req.app_data::<web::Data<ServerState>>() {
                let game = state.game.lock().await;
                let current_phase = game.get_phase().as_str().to_lowercase();

                if current_phase != game_step {
                    warn!(
                        "페이즈 불일치: 요청된 페이즈={}, 현재 게임 페이즈={}",
                        game_step, current_phase
                    );
                    return Err(GameError::WrongPhase);
                }
                debug!("페이즈 검증 성공: {}", current_phase);

                let player_name_str = player_name.to_string();
                let p1_key = state.player_cookie.0.as_str();
                let p2_key = state.opponent_cookie.0.as_str();

                let player_type = match player_name_str.as_str() {
                    key if key == p1_key => {
                        debug!("플레이어1로 인증됨");
                        PlayerType::Player1
                    }
                    key if key == p2_key => {
                        debug!("플레이어2로 인증됨");
                        PlayerType::Player2
                    }
                    _ => {
                        error!("잘못된 플레이어 키: {}", player_name_str);
                        return Err(GameError::InternalServerError);
                    }
                };

                debug!(
                    "세션 등록 시작: player_type={:?}, game_step={}",
                    player_type, game_step
                );
                let session_id = state
                    .session_manager
                    .register_session(player_type, game_step.clone().into())
                    .await;
                debug!("세션 등록 완료: session_id={}", session_id);

                // 다른 엔드포인트에 이미 유효한 세션이 있는지 확인
                if !state
                    .session_manager
                    .is_valid_session(player_type, session_id, game_step.into())
                    .await
                {
                    warn!(
                        "중복 세션 감지: player_type={:?}, session_id={}",
                        player_type, session_id
                    );
                    return Err(GameError::ActiveSessionExists);
                }

                info!(
                    "인증 성공: player_type={:?}, session_id={}",
                    player_type, session_id
                );
                Ok(AuthPlayer::new(player_type, session_id))
            } else {
                error!("서버 상태 객체를 찾을 수 없음");
                Err(GameError::ServerStateNotFound)
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

// 최초로 연결되어야하는 엔드포인트.
#[get("/heartbeat")]
#[instrument(skip(state, req, payload), fields(player_type = ?player.ptype, session_id = ?player.session_id))]
pub async fn heartbeat(
    player: AuthPlayer,
    state: web::Data<ServerState>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse, GameError> {
    let player_type = player.ptype;
    let session_id = player.session_id;

    debug!("WebSocket 연결 업그레이드 시작");

    // WebSocket 연결 설정
    let (response, session, mut stream) = match handle(&req, payload) {
        Ok(result) => {
            info!("WebSocket 연결 성공: player={:?}", player_type);
            result
        }
        Err(e) => {
            error!(
                "WebSocket 핸들링 실패: player={:?}, error={:?}",
                player_type, e
            );
            return Err(GameError::HandleFailed);
        }
    };

    let mut session_clone = session.clone();
    let heartbeat_session_manager = state.session_manager.clone();

    // 하트비트 처리를 위한 별도 태스크 생성
    info!(
        "하트비트 태스크 시작: player={:?}, session_id={}",
        player_type, session_id
    );

    actix_web::rt::spawn(async move {
        debug!("하트비트 인터벌 설정: {}초", HEARTBEAT_INTERVAL);
        let mut interval = tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL));
        let mut last_pong = Instant::now();

        // 클라이언트에게 초기 메시지 전송 (선택사항)
        if let Err(e) = session_clone
            .text(
                json!({
                    "type": "connection_established",
                    "player": format!("{:?}", player_type),
                    "session_id": session_id.to_string(),
                })
                .to_string(),
            )
            .await
        {
            error!(
                "초기 메시지 전송 실패: player={:?}, error={:?}",
                player_type, e
            );
            return;
        }

        loop {
            tokio::select! {
                // 주기적인 핑 전송
                _ = interval.tick() => {
                    // 마지막 pong 으로부터 너무 오랜 시간이 지났으면 연결 종료
                    if last_pong.elapsed() > Duration::from_secs(CLIENT_TIMEOUT) {
                        error!(
                            "클라이언트 타임아웃: player={:?}, 마지막 응답으로부터 {:?}초 경과",
                            player_type, last_pong.elapsed().as_secs()
                        );
                        break;
                    }

                    // 핑 메시지 전송
                    debug!("하트비트 ping 전송: player={:?}", player_type);
                    if let Err(e) = session_clone.ping(b"heartbeat").await {
                        error!("하트비트 ping 실패: player={:?}, error={:?}", player_type, e);
                        break;
                    }
                },

                // 클라이언트로부터 메시지 수신
                Some(msg) = stream.next() => {
                    match msg {
                        Ok(Message::Pong(_)) => {
                            debug!("하트비트 pong 응답 수신: player={:?}", player_type);
                            last_pong = Instant::now();
                        },
                        Ok(Message::Close(reason)) => {
                            info!("클라이언트가 연결 종료 요청: player={:?}, reason={:?}", player_type, reason);
                            break;
                        },
                        Err(e) => {
                            error!("WebSocket 메시지 에러: player={:?}, error={:?}", player_type, e);
                            break;
                        }
                        _ => {
                            info!("클라이언트로부터 다른 메시지 수신: player={:?}", player_type);
                        }
                    }
                }
            }
        }

        // 하트비트 태스크 종료시 세션 정리
        info!(
            "하트비트 태스크 종료, 세션 정리: player={:?}, session_id={}",
            player_type, session_id
        );

        // 세션 종료 처리
        heartbeat_session_manager
            .end_session(player_type, session_id)
            .await;

        info!("세션 정리 성공");

        // 웹소켓 연결 종료
        if let Err(e) = session_clone
            .close(Some(CloseReason {
                code: CloseCode::Normal,
                description: Some("세션 종료".to_string()),
            }))
            .await
        {
            error!("세션 종료 실패: player={:?}, error={:?}", player_type, e);
        } else {
            debug!("세션 종료 성공: player={:?}", player_type);
        }

        info!(
            "하트비트 태스크 종료: player={:?}, session_id={}",
            player_type, session_id
        );
    });

    Ok(response)
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

// TODO: 네트워크 이슈가 발생하여 재연결이 필요한 경우 처리가 필요함.
// TODO: 게임 로직을 end point 가 아니라 Game 내부에다가 구현 해야함.
#[get("/mulligan_phase")]
#[instrument(skip(state, req, payload), fields(player_type = ?player.ptype, session_id = ?player.session_id))]
pub async fn mulligan_phase(
    player: AuthPlayer,
    state: web::Data<ServerState>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse, GameError> {
    info!("멀리건 단계 핸들러 시작: player={:?}", player.ptype);
    // 멀리건 수행 중 연결이 끊힌 경우, 재진입을 허용해야 하는데, 아직 뚜렷한 방법이 떠오르진 않음.

    // 플레이어가 재진입을 시도하는 경우
    {
        let game = state.game.lock().await;
        debug!("게임 상태 잠금 획득: 재진입 확인");

        // TODO: 일관성 있게, PhaseState 를 사용하여 처리하는 것이 좋을 것 같음.
        if !game
            .get_player_by_type(player.ptype)
            .get()
            .get_mulligan_state_mut()
            .get_select_cards()
            .is_empty()
        {
            error!("플레이어가 이미 멀리건을 시작함: player={:?}", player.ptype);
            return Err(GameError::NotAllowedReEntry);
        }
    }

    let player_type = player.ptype;
    debug!("플레이어 타입 설정: {:?}", player_type);

    // Http 업그레이드: 이때 session과 stream이 반환됩니다.
    debug!("WebSocket 연결 업그레이드 시작");
    let (resp, mut session, mut stream) = match handle(&req, payload) {
        Ok(result) => {
            info!("WebSocket 연결 성공: player={:?}", player_type);
            result
        }
        Err(e) => {
            error!(
                "WebSocket 핸들링 실패: player={:?}, error={:?}",
                player_type, e
            );
            return Err(GameError::HandleFailed);
        }
    };

    // Mulligan deal 단계 수행 코드입니다.
    // 새로운 카드를 뽑아서 player 의 mulligan cards 에 저장 한 뒤, json 형태로 변환하여 전송합니다.
    let new_cards = {
        let mut game = state.game.lock().await;
        debug!("게임 상태 잠금 획득: 멀리건 카드 처리");

        info!(
            "멀리건 카드 뽑기 시작: player={:?}, count={}",
            player_type, COUNT_OF_MULLIGAN_CARDS
        );
        let cards = match game.get_mulligan_cards(player_type, COUNT_OF_MULLIGAN_CARDS) {
            Ok(cards) => {
                debug!("멀리건 카드 뽑기 성공: card_count={}", cards.len());
                cards
            }
            Err(e) => {
                error!(
                    "멀리건 카드 뽑기 실패: player={:?}, error={:?}",
                    player_type, e
                );
                return Err(e);
            }
        };

        game.add_select_cards(cards.clone(), player_type);
        debug!("플레이어 멀리건 상태에 선택 카드 추가 완료");

        cards
    };

    debug!("멀리건 딜 메시지 직렬화 시작");
    let new_cards_json = match serialize_deal_message(player_type, new_cards) {
        Ok(json) => {
            debug!("멀리건 딜 메시지 직렬화 성공");
            json
        }
        Err(e) => {
            error!("멀리건 메시지 직렬화 실패: error={:?}", e);
            return Err(e);
        }
    };

    debug!("멀리건 딜 메시지 전송 시작");
    if let Err(e) = session.text(new_cards_json).await {
        error!("멀리건 딜 메시지 전송 실패: error={:?}", e);
        return Err(GameError::InternalServerError);
    }
    info!("멀리건 딜 메시지 전송 완료");

    let mulligan_session_id = player.session_id;
    let mulligan_session_manager = state.session_manager.clone();

    // 이후, 스레드 내에서 클라이언트와의 상호작용을 계속하기 위해 필요한 state를 클론합니다.
    // WebSocket 메시지 수신 등 후속 처리는 별도 spawn된 작업에서 진행합니다.
    info!(
        "멀리건 메시지 처리 태스크 시작: player={:?}, session_id={}",
        player_type, mulligan_session_id
    );
    actix_web::rt::spawn(async move {
        debug!("메시지 핸들러 생성");
        let mut handler = MessageHandler::new();

        while let Some(data) = stream.next().await {
            match data {
                // 클라이언트에서 받은 메시지를 분석합니다.
                Ok(Message::Text(json)) => {
                    debug!("클라이언트 메시지 수신: player={:?}", player_type);

                    let result = handler
                        .process_message::<mulligan::ClientMessage>(
                            &mut session,
                            &json,
                            mulligan_session_id,
                            player_type,
                        )
                        .await;

                    match result {
                        MessageProcessResult::SystemHandled => {
                            // TODO: 작성해야함.
                            debug!("시스템 메시지 처리: player={:?}", player_type);
                        }
                        MessageProcessResult::Success(msg) => {
                            info!(
                                "메시지 처리 성공: player={:?}, message_type={}",
                                player_type,
                                std::any::type_name::<mulligan::ClientMessage>()
                            );

                            match msg {
                                mulligan::ClientMessage::RerollRequest(payload) => {
                                    debug!("리롤 요청 처리: player={:?}", player_type);

                                    if !matches!(payload.player.as_str(), "player1" | "player2") {
                                        error!("유효하지 않은 플레이어: {}", payload.player);
                                        try_send_error!(session, GameError::InvalidPlayer, retry 3);
                                    }

                                    let player_type = AuthPlayer::new(
                                        payload.player.clone().into(),
                                        mulligan_session_id,
                                    );

                                    if let Err(e) = payload.cards.to_vec_uuid() {
                                        error!("카드 UUID 변환 실패: error={:?}", e);
                                        try_send_error!(session, GameError::InvalidCards, retry 3);
                                    }
                                    let payload_cards = payload.cards.to_vec_uuid().unwrap();

                                    debug!(
                                        "리롤 요청 처리: player={:?}, cards={:?}",
                                        player_type, payload_cards
                                    );

                                    let mut game = state.game.lock().await;
                                    debug!("게임 상태 잠금 획득: 리롤 요청 처리");

                                    let result =
                                        game.reroll_request(player_type, payload_cards.clone());

                                    if let Err(e) = &result {
                                        error!(
                                            "리롤 요청 처리 실패: player={:?}, error={:?}",
                                            player_type, e
                                        );
                                        try_send_error!(session, e.clone(), retry 3);
                                    }

                                    let rerolled_cards = result.unwrap();

                                    // 플레이어가 선택한 카드를 select_cards 에서 삭제하고 Reroll 된 카드를 추가합니다.
                                    game.add_reroll_cards(
                                        player_type,
                                        payload_cards.clone(),
                                        rerolled_cards,
                                    );

                                    // 멀리건 완료 단계를 수행합니다.
                                    info!("멀리건 완료 처리: player={:?}", player_type);
                                    let selected_cards =
                                        match game.process_mulligan_completion(player_type) {
                                            Ok(selected_cards) => {
                                                debug!("멀리건 완료 처리 성공");
                                                selected_cards
                                            }
                                            Err(e) => {
                                                error!(
                                                "멀리건 완료 처리 실패: player={:?}, error={:?}",
                                                player_type, e
                                            );
                                                break;
                                            }
                                        };

                                    // 상대 플레이어의 준비 상태 확인

                                    if game.check_player_ready_state(player_type.reverse()) {
                                        info!("양 플레이어 모두 준비 완료: 다음 단계 전환 예정");

                                        // TODO: 다음 단계로 넘어가는 코드 작성
                                        game.move_phase();
                                    }

                                    debug!("리롤 응답 메시지 직렬화 시작");
                                    let selected_cards_json = match serialize_reroll_answer(
                                        player_type,
                                        selected_cards,
                                    ) {
                                        Ok(json) => {
                                            debug!("리롤 응답 메시지 직렬화 성공");
                                            json
                                        }
                                        Err(e) => {
                                            error!("리롤 응답 메시지 직렬화 실패: error={:?}", e);
                                            break;
                                        }
                                    };

                                    debug!("리롤 응답 메시지 전송 시작");
                                    if let Err(e) = session.text(selected_cards_json).await {
                                        error!("리롤 응답 메시지 전송 실패: error={:?}", e);
                                        break;
                                    }
                                    info!("리롤 응답 메시지 전송 완료");

                                    info!(
                                        "멀리건 세션 종료: player={:?}, session_id={}",
                                        player_type, mulligan_session_id
                                    );
                                    mulligan_session_manager
                                        .end_session(player_type, mulligan_session_id)
                                        .await;
                                }
                                mulligan::ClientMessage::Complete(payload) => {
                                    debug!("멀리건 완료 요청 처리: player={:?}", player_type);

                                    if !matches!(payload.player.as_str(), "player1" | "player2") {
                                        error!("유효하지 않은 플레이어: {}", payload.player);
                                        try_send_error!(session, GameError::InvalidPlayer, retry 3);
                                    }

                                    let player_type = AuthPlayer::new(
                                        payload.player.clone().into(),
                                        mulligan_session_id,
                                    );

                                    let mut game = state.game.lock().await;
                                    debug!("게임 상태 잠금 획득: 멀리건 완료 요청 처리");

                                    // 이미 준비가 되어있다면 send_error_and_check 함수를 통해 에러 메시지를 전송하고 종료합니다.
                                    if game
                                        .get_player_by_type(player_type)
                                        .get()
                                        .get_mulligan_state_mut()
                                        .is_ready()
                                    {
                                        warn!(
                                            "플레이어가 이미 준비 상태: player={:?}",
                                            player_type
                                        );
                                        try_send_error!(session, GameError::InvalidApproach, retry 3);
                                    }

                                    // 페이로드의 cards 를 확인하여 유효성 검사를 진행합니다.
                                    debug!("선택한 카드 유효성 검사: player={:?}", player_type);
                                    if payload.validate(
                                        game.get_player_by_type(player_type).get().get_cards(),
                                    ) == None
                                    {
                                        error!("유효하지 않은 카드 선택: player={:?}", player_type);
                                        try_send_error!(session, GameError::InvalidCards, retry 3);
                                    }

                                    // player 의 mulligan 상태를 완료 상태로 변경 후 상대의 mulligan 상태를 확인합니다.
                                    // 만약 상대도 완료 상태이라면, mulligan step 을 종료하고 다음 step 으로 진행합니다.
                                    info!("멀리건 완료 처리: player={:?}", player_type);
                                    let selected_cards =
                                        match game.process_mulligan_completion(player_type) {
                                            Ok(selected_cards) => {
                                                debug!("멀리건 완료 처리 성공");
                                                selected_cards
                                            }
                                            Err(e) => {
                                                error!(
                                                "멀리건 완료 처리 실패: player={:?}, error={:?}",
                                                player_type, e
                                            );
                                                break;
                                            }
                                        };

                                    // 상대 플레이어의 준비 상태 확인
                                    if game.check_player_ready_state(player_type.reverse()) {
                                        info!("양 플레이어 모두 준비 완료: 다음 단계 전환 예정");
                                        // 다음 페이즈로 이동하는 코드
                                        game.move_phase();
                                    }

                                    debug!("완료 메시지 직렬화 시작");
                                    if let Err(e) =
                                        serialize_complete_message(player, selected_cards)
                                    {
                                        error!("완료 메시지 직렬화 실패: error={:?}", e);
                                        break;
                                    }
                                    debug!("완료 메시지 직렬화 성공");

                                    debug!("완료 메시지 전송 시작");
                                    if let Err(e) = session.text(json).await {
                                        error!("완료 메시지 전송 실패: error={:?}", e);
                                        break;
                                    }
                                    info!("완료 메시지 전송 완료");

                                    info!(
                                        "멀리건 세션 종료: player={:?}, session_id={}",
                                        player_type, mulligan_session_id
                                    );
                                    mulligan_session_manager
                                        .end_session(player_type, mulligan_session_id)
                                        .await;
                                }
                            }
                        }
                        MessageProcessResult::NeedRetry => {
                            warn!("메시지 처리 재시도 필요: player={:?}", player_type);
                            try_send_error!(session, GameError::InvalidApproach, retry 3);
                            continue;
                        }
                        MessageProcessResult::TerminateSession(server_error) => {
                            error!(
                                "세션 종료 필요: player={:?}, error={:?}",
                                player_type, server_error
                            );
                            mulligan_session_manager
                                .end_session(player_type, mulligan_session_id)
                                .await;
                        }
                    }
                }
                Ok(Message::Close(reason)) => {
                    info!(
                        "WebSocket 종료 메시지 수신: player={:?}, reason={:?}",
                        player_type, reason
                    );
                    break;
                }
                Ok(msg) => {
                    debug!(
                        "기타 WebSocket 메시지 수신: player={:?}, type={:?}",
                        player_type, msg
                    );
                }
                Err(e) => {
                    error!(
                        "WebSocket 메시지 수신 오류: player={:?}, error={:?}",
                        player_type, e
                    );
                    break;
                }
            }
        }

        if let Err(close_err) = session
            .close(Some(CloseReason {
                code: CloseCode::Normal,
                description: Some("종료".into()),
            }))
            .await
        {
            error!(
                "WebSocket 연결 종료 실패: player={:?}, error={:?}",
                player_type, close_err
            );
            panic!("WebSocket 연결 종료 실패");
        }
        info!("WebSocket 메시지 처리 루프 종료: player={:?}", player_type);
    });

    info!("멀리건 핸들러 완료: player={:?}", player_type);
    Ok(resp)
}

#[get("/draw_phase")]
#[instrument(skip(state), fields(player_type = ?player.ptype))]
pub async fn draw_phase(
    player: AuthPlayer,
    state: web::Data<ServerState>,
) -> Result<HttpResponse, GameError> {
    let player_type = player.ptype;
    info!("드로우 단계 처리 시작: player={:?}", player_type);

    let drawn_card = {
        let mut game = state.game.lock().await;
        debug!("게임 상태 잠금 획득");

        // 플레이어가 이미 카드를 뽑은 경우를 확인함
        if game.phase_state.has_player_completed(player_type) {
            error!("플레이어가 이미 드로우를 완료함: player={:?}", player_type);
            return Err(GameError::NotAllowedReEntry);
        }

        game.phase_state.mark_player_completed(player_type);
        debug!("플레이어 드로우 완료 표시: player={:?}", player_type);

        // 에러가 발생할 경우 mark 를 reset 함.
        let result = game.handle_draw_phase(player_type).inspect_err(|e| {
            game.phase_state.reset_player_completed(player_type);
        })?;

        // 다음 페이즈로 이동
        game.move_phase();

        result
    };

    debug!(
        "응답 JSON 구성 중: player={:?}, card_uuid={}",
        player_type, drawn_card
    );
    let response_data = match serialize_draw_answer_message(player_type, drawn_card) {
        Ok(data) => data,
        Err(e) => {
            error!("JSON 직렬화 실패: player={:?}, error={:?}", player_type, e);
            return Err(e);
        }
    };

    // JSON 응답 반환
    info!(
        "드로우 단계 처리 완료: player={:?}, card_uuid={}",
        player_type, drawn_card
    );
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(response_data.to_string()))
}

#[get("/standby_phase")]
#[instrument(skip(state), fields(player_type = ?player.ptype))]
pub async fn standby_phase(
    player: AuthPlayer,
    state: web::Data<ServerState>,
) -> Result<HttpResponse, GameError> {
    /*
        Standby 페이즈.
        유희왕 룰에 따르면 플레이어가 반드시 실시해야하는 행위는 없음

        해당 페이즈를 구현하기 위해서는 제대로 된 덱이 하나 있어야함.
        때문에 그 전까지는 미구현 상태로 둠.
    */
    todo!()
}

#[get("/main_phase_start_phase")]
#[instrument(skip(state, req, payload), fields(player_type = ?player.ptype))]
pub async fn main_phase_start_phase(
    player: AuthPlayer,
    state: web::Data<ServerState>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse, GameError> {
    info!(
        "메인 페이즈1 개시시 단계 핸들러 시작: player={:?}",
        player.ptype
    );

    let player_type = player.ptype;

    // 플레이어가 재진입을 시도하는 경우
    debug!("게임 상태 잠금 획득: 재진입 확인");
    // TODO: 플레이어가 이미 메인 페이즈1 개시시 단계 진입한 경우 처리가 필요함.

    // Http 업그레이드: 이때 session과 stream이 반환됩니다.
    debug!("WebSocket 연결 업그레이드 시작");
    let (resp, session, stream) = match handle(&req, payload) {
        Ok(result) => {
            info!("WebSocket 연결 성공: player={:?}", player_type);
            result
        }
        Err(e) => {
            error!(
                "WebSocket 핸들링 실패: player={:?}, error={:?}",
                player_type, e
            );
            return Err(GameError::HandleFailed);
        }
    };
    /*
        MainPhaseStart 페이즈.
        메인 페이즈에서 플레이어가 아무 행동도 하지 않은 상태를 의미함.

        이 페이즈에서만 수행 가능한 카드들이 있음.
        [ 메인 페이즈1 개시시 ] 이라는 키워드를 가진 카드들은 해당 페이즈에서 발동 가능함.
        현재 턴의 플레이어의 카드들을 순회해서 해당 효과를 발동시킬건지 플레이어에게 물어보고 입력을 대기해야함.

        이 때문에 WebSocket 연결을 수립해야함.
    */
    todo!()
}

#[get("/main_phase_1_phase")]
#[instrument(skip(state, req, payload), fields(player_type = ?player.ptype))]
pub async fn main_phase_1_phase(
    player: AuthPlayer,
    state: web::Data<ServerState>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse, GameError> {
    /*
        MainPhase1 페이즈.
    */
    info!("메인 페이즈1 페이즈 시작: player={:?}", player.ptype);

    let player_type = player.ptype;
    {
        let mut game = state.game.lock().await;
        debug!("게임 상태 잠금 획득: 재진입 확인");

        if game.phase_state.has_player_completed(player_type) {
            error!(
                "플레이어가 해당 페이즈를 이미 진행중임.: player={:?}",
                player_type
            );
            return Err(GameError::NotAllowedReEntry);
        }

        game.phase_state.mark_player_completed(player_type);
        debug!("플레이어 페이즈 진입 표시: player={:?}", player_type);
    };

    // Http 업그레이드: 이때 session과 stream이 반환됩니다.
    debug!("WebSocket 연결 업그레이드 시작");
    let (resp, mut session, mut stream) = match handle(&req, payload) {
        Ok(result) => {
            info!("WebSocket 연결 성공: player={:?}", player_type);
            result
        }
        Err(e) => {
            error!(
                "WebSocket 핸들링 실패: player={:?}, error={:?}",
                player_type, e
            );
            return Err(GameError::HandleFailed);
        }
    };
    debug!("메시지 핸들러 생성");

    let mut handler = MessageHandler::new();

    let session_id = player.session_id;
    let session_manager = state.session_manager.clone();

    while let Some(data) = stream.next().await {
        match data {
            // 클라이언트에서 받은 메시지를 분석합니다.
            Ok(Message::Text(json)) => {
                debug!("클라이언트 메시지 수신: player={:?}", player_type);

                let result = handler
                    .process_message::<main_phase1::ClientMessage>(
                        &mut session,
                        &json,
                        session_id,
                        player_type,
                    )
                    .await;

                match result {
                    MessageProcessResult::SystemHandled => {
                        // TODO: 작성해야함.
                    }
                    MessageProcessResult::Success(msg) => {
                        info!(
                            "메시지 처리 성공: player={:?}, message_type={}",
                            player_type,
                            std::any::type_name::<mulligan::ClientMessage>()
                        );

                        match msg {
                            main_phase1::ClientMessage::PlayCard(payload) => {
                                debug!("카드 플레이 요청 처리: player={:?}", player_type);

                                if !matches!(payload.player.as_str(), "player1" | "player2") {
                                    error!("유효하지 않은 플레이어: {}", payload.player);
                                    try_send_error!(session, GameError::InvalidPlayer, retry 3);
                                }

                                let player_type =
                                    AuthPlayer::new(payload.player.clone().into(), session_id);

                                if let Err(e) = payload.card.to_uuid() {
                                    error!("카드 UUID 변환 실패: error={:?}", e);
                                    try_send_error!(session, GameError::InvalidCards, retry 3);
                                }
                                let mut game = state.game.lock().await;
                                debug!("게임 상태 잠금 획득: 리롤 요청 처리");

                                let payload_cards_uuid = payload.card.to_uuid().unwrap();
                                let payload_cards =
                                    game.get_cards_by_uuid(payload_cards_uuid.clone())?;

                                // 사용자 입력 대기의 경우
                                let result = game
                                    .proceed_card(player_type, payload_cards_uuid.clone())
                                    .await;
                                if let Ok(inner_result) = result {
                                    match inner_result {
                                        PlayCardResult::Success => break,
                                        PlayCardResult::Fail(game_error) => todo!(),
                                    }
                                } else {
                                    error!(
                                        "카드 플레이 실패: player={:?}, error={:?}",
                                        player_type,
                                        result.unwrap_err()
                                    );
                                    try_send_error!(session, GameError::InvalidCards, retry 3);
                                }
                            }
                        }
                    }
                    MessageProcessResult::NeedRetry => {
                        warn!("메시지 처리 재시도 필요: player={:?}", player_type);
                        try_send_error!(session, GameError::InvalidApproach, retry 3);
                        continue;
                    }
                    MessageProcessResult::TerminateSession(server_error) => {
                        error!(
                            "세션 종료 필요: player={:?}, error={:?}",
                            player_type, server_error
                        );
                        session_manager.end_session(player_type, session_id).await;
                    }
                }
            }
            Ok(Message::Close(reason)) => {
                info!(
                    "WebSocket 종료 메시지 수신: player={:?}, reason={:?}",
                    player_type, reason
                );
                break;
            }
            Ok(msg) => {
                debug!(
                    "기타 WebSocket 메시지 수신: player={:?}, type={:?}",
                    player_type, msg
                );
            }
            Err(e) => {
                error!(
                    "WebSocket 메시지 수신 오류: player={:?}, error={:?}",
                    player_type, e
                );
                break;
            }
        }
    }

    if let Err(close_err) = session
        .close(Some(CloseReason {
            code: CloseCode::Normal,
            description: Some("종료".into()),
        }))
        .await
    {
        error!(
            "WebSocket 연결 종료 실패: player={:?}, error={:?}",
            player_type, close_err
        );
        panic!("WebSocket 연결 종료 실패");
    }
    info!("WebSocket 메시지 처리 루프 종료: player={:?}", player_type);

    Ok(resp)
}
