use std::time::Duration;

use actix_ws::Session;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::{
    card::{insert::TopInsert, types::PlayerType},
    enums::UUID,
    exception::{MessageProcessResult, MulliganError, ServerError},
    game::Game,
    serialize_error,
    zone::zone::Zone,
};

use super::jsons::Message;

pub fn process_mulligan_completion<T: Into<PlayerType> + Copy>(
    game: &mut Game,
    player_type: T,
) -> Result<Vec<UUID>, ServerError> {
    let selected_cards = game
        .get_player_by_type(player_type.into())
        .get()
        .get_mulligan_state_mut()
        .get_select_cards();
    let cards = game.get_cards_by_uuid(selected_cards.clone());
    game.get_player_by_type(player_type.into())
        .get()
        .get_hand_mut()
        .add_card(cards, Box::new(TopInsert))
        .map_err(|_| ServerError::InternalServerError)?;
    game.get_player_by_type(player_type.into())
        .get()
        .get_mulligan_state_mut()
        .confirm_selection();
    Ok(selected_cards)
}

/// 에러 메시지를 전송하고, 전송 성공 여부를 반환합니다.
/// # return
/// - 성공 시 Some(())
/// - 실패 시 None
pub async fn send_error_and_check(session: &mut Session, error_msg: MulliganError) -> Option<()> {
    // 에러 메시지 직렬화
    let Ok(error_json) = serialize_error!(error_msg) else {
        return None; // 직렬화 실패
    };

    // 메시지 전송
    match session.text(error_json).await {
        Ok(_) => Some(()),
        Err(_) => None,
    }
}

pub struct MessageHandler {
    unexpected_msg_count: usize,
    parsing_error_count: usize,
}

impl MessageHandler {
    pub fn new() -> Self {
        Self {
            unexpected_msg_count: 0,
            parsing_error_count: 0,
        }
    }

    /// 메시지 처리 시도 및 에러 카운트 관리
    pub async fn process_message<T: Message + DeserializeOwned>(
        &mut self,
        session: &mut Session,
        json: &str,
        session_id: Uuid,
        player_type: PlayerType,
    ) -> MessageProcessResult<T> {
        // 반환 값이 Ok면 계속, Err면 함수를 종료해야 함
        let parse_result = serde_json::from_str::<serde_json::Value>(json);
        if let Err(e) = parse_result {
            // JSON 구문 자체가 잘못됨 (파싱 실패)
            return self
                .handle_parse_error(session, json, e, session_id, player_type)
                .await;
        }

        match serde_json::from_str::<T>(json) {
            Ok(data) => {
                // 성공 시 카운터 초기화
                self.reset_counters();
                MessageProcessResult::Success(data)
            }
            Err(e) => {
                self.handle_unexpected_message(session, session_id, player_type)
                    .await
            }
        }
    }

    /// 파싱 에러 처리
    async fn handle_parse_error<T>(
        &mut self,
        session: &mut Session,
        json: &str,
        error: serde_json::Error,
        session_id: Uuid,
        player_type: PlayerType,
    ) -> MessageProcessResult<T> {
        self.parsing_error_count += 1;

        // 로그 출력
        // log::warn!(
        //     "JSON parsing error: {}, attempt: {}/3, received: {}",
        //     error, self.parsing_error_count, json
        // );

        if self.parsing_error_count >= 3 {
            self.terminate_session(session, MulliganError::ParseError, session_id, player_type)
                .await;
            return MessageProcessResult::TerminateSession(ServerError::ParseError(
                "JSON parsing error".to_string(),
            ));
        }

        MessageProcessResult::NeedRetry
    }

    /// 예상치 못한 메시지 처리
    async fn handle_unexpected_message<T>(
        &mut self,
        session: &mut Session,
        session_id: Uuid,
        player_type: PlayerType,
    ) -> MessageProcessResult<T> {
        self.unexpected_msg_count += 1;

        if self.unexpected_msg_count >= 3 {
            self.terminate_session(
                session,
                MulliganError::InvalidApproach,
                session_id,
                player_type,
            )
            .await;
            return MessageProcessResult::TerminateSession(ServerError::UnexpectedMessage);
        }

        MessageProcessResult::NeedRetry
    }

    /// 세션 종료 처리
    async fn terminate_session(
        &self,
        session: &mut Session,
        error: MulliganError,
        session_id: Uuid,
        player_type: PlayerType,
    ) {
        // 에러 메시지 전송 (최대 3회 시도)
        self.send_error_with_retry(session, error).await;
    }

    /// 에러 메시지 전송 (재시도 로직 포함)
    async fn send_error_with_retry(&self, session: &mut Session, error: MulliganError) -> bool {
        if let Ok(error_json) = serialize_error!(error) {
            for attempt in 0..3 {
                match session.text(error_json.clone()).await {
                    Ok(_) => return true,
                    Err(err) if attempt < 2 => {
                        // log::warn!("Failed to send error, retry {}/3: {}", attempt + 1, err);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    Err(err) => {
                        // log::error!("Failed to send error after 3 attempts: {}", err);
                        return false;
                    }
                }
            }
        }
        false
    }

    /// 카운터 초기화
    fn reset_counters(&mut self) {
        self.unexpected_msg_count = 0;
        self.parsing_error_count = 0;
    }
}
