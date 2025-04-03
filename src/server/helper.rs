use std::time::Duration;

use actix_ws::Session;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::{
    card::types::PlayerType,
    exception::{GameError, MessageProcessResult},
    serialize_error,
};

use super::jsons::Message;

/// 에러 메시지 전송 매크로
/// - 세션을 통해 에러 메시지를 전송하고, 전송 성공 여부를 반환합니다.
/// - retry 키워드를 사용하면 최대 재시도 횟수를 지정할 수 있습니다.
#[macro_export]
macro_rules! try_send_error {
    ($session:expr, $error:expr) => {
        if send_error_and_check(&mut $session, $error).await == Some(()) {
            break;
        }
    };

    ($session:expr, $error:expr, retry) => {
        match send_error_and_check(&mut $session, $error).await {
            Some(()) => break,
            None => {
                // 재시도 로직
                let retry_result = send_error_and_check(&mut $session, $error).await;
                if retry_result == Some(()) {
                    break;
                }
            }
        }
    };

    ($session:expr, $error:expr, retry $max_retries:expr) => {{
        let mut retries = 0;
        loop {
            match send_error_and_check(&mut $session, $error).await {
                Some(()) => break,
                None => {
                    retries += 1;
                    if retries >= $max_retries {
                        // 로깅 또는 다른 실패 처리
                        // log::warn!("Failed to send error after {} retries", $max_retries);
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
    }};
}

/// 에러 메시지를 전송하고, 전송 성공 여부를 반환합니다.
/// # return
/// - 성공 시 Some(())
/// - 실패 시 None
pub async fn send_error_and_check(session: &mut Session, error_msg: GameError) -> Option<()> {
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
            self.terminate_session(session, GameError::ParseError, session_id, player_type)
                .await;
            return MessageProcessResult::TerminateSession(GameError::ParseError);
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
            self.terminate_session(session, GameError::InvalidApproach, session_id, player_type)
                .await;
            return MessageProcessResult::TerminateSession(GameError::UnexpectedMessage);
        }

        MessageProcessResult::NeedRetry
    }

    /// 세션 종료 처리
    async fn terminate_session(
        &self,
        session: &mut Session,
        error: GameError,
        session_id: Uuid,
        player_type: PlayerType,
    ) {
        // 에러 메시지 전송 (최대 3회 시도)
        self.send_error_with_retry(session, error).await;
    }

    /// 에러 메시지 전송 (재시도 로직 포함)
    async fn send_error_with_retry(&self, session: &mut Session, error: GameError) -> bool {
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
