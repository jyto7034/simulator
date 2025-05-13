use crate::exception::GameError;
use std::collections::HashMap;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum InputRequest {
    Dig {
        source_card: Uuid,
        source_effect_uuid: Uuid,
        potential_cards: Vec<Uuid>,
    },
    SelectEffect {
        source_card: Uuid,
        potential_effects: Vec<Uuid>,
    },
}

#[derive(Debug, Clone)]
pub enum InputAnswer {
    Dig(Vec<Uuid>),
    SelectEffect(Uuid),
}

#[derive(Debug)]
pub struct PendingInput {
    request: InputRequest,
    response: Option<InputAnswer>,
}

#[derive(Clone)]
pub struct InputWaiter {
    state: Arc<Mutex<HashMap<Uuid, PendingInput>>>,
}

impl InputWaiter {
    pub fn new() -> Self {
        InputWaiter {
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn wait_for_input(
        &self,
        request: InputRequest,
    ) -> Result<oneshot::Receiver<InputAnswer>, GameError> {
        let request_id = Uuid::new_v4();
        let (tx, rx) = oneshot::channel();

        {
            // 잠금 획득
            let mut state: tokio::sync::MutexGuard<'_, HashMap<Uuid, PendingInput>> =
                self.state.lock().await;

            // 요청 상태를 추가
            state.insert(
                request_id,
                PendingInput {
                    request: request.clone(),
                    response: None,
                },
            );
        }

        Ok(rx)
    }

    // 엔드포인트에서 대기 중인 입력 요청 확인
    pub async fn get_pending_requests(&self) -> Vec<(Uuid, InputRequest)> {
        let state = self.state.lock().await;
        state
            .iter()
            .map(|(id, pending)| (*id, pending.request.clone()))
            .collect()
    }

    // 엔드포인트에서 입력 응답 처리
    pub async fn submit_input(
        &self,
        request_id: Uuid,
        response: InputAnswer,
    ) -> Result<(), GameError> {
        let mut state = self.state.lock().await;

        if let Some(pending) = state.get_mut(&request_id) {
            pending.response = Some(response);

            Ok(())
        } else {
            Err(GameError::InvalidRequestId)
        }
    }
}
