use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::exception::GameError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputRequest {
    Dig { options: Vec<Uuid> },
    SelectTarget { options: Vec<Uuid> },
}

pub struct InputHandler {
    pending_inputs: HashMap<Uuid, oneshot::Sender<Vec<Uuid>>>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            pending_inputs: HashMap::new(),
        }
    }

    /// 비동기적으로 사용자 입력을 기다립니다
    pub async fn wait_for_input(&mut self, request: InputRequest) -> Result<Vec<Uuid>, GameError> {
        let request_id = Uuid::new_v4();
        let (tx, rx) = oneshot::channel();

        // 요청 저장
        self.pending_inputs.insert(request_id, tx);

        // JSON 생성 및 전송 로직은 외부에서 처리

        // 비동기적으로 응답 대기
        match rx.await {
            Ok(selection) => Ok(selection),
            Err(_) => Err(GameError::InputRequestCancelled),
        }
    }

    /// 클라이언트로부터 받은 입력을 처리합니다
    pub fn handle_input(
        &mut self,
        request_id: Uuid,
        selection: Vec<Uuid>,
    ) -> Result<(), GameError> {
        if let Some(tx) = self.pending_inputs.remove(&request_id) {
            // 선택 결과 전송 - wait_for_input이 재개됨
            let _ = tx.send(selection);
            Ok(())
        } else {
            Err(GameError::InvalidInputRequest)
        }
    }

    /// 모든 대기 중인 요청을 취소합니다
    pub fn cancel_all(&mut self) {
        self.pending_inputs.clear();
    }
}
