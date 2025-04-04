use crate::exception::GameError;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::result::Result;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum InputRequest {
    Dig {
        source_card: Uuid,
        source_effect_uuid: Uuid,
        potential_cards: Vec<Uuid>,
    },
}

#[derive(Debug, Clone)]
pub enum InputAnswer {
    Dig(Vec<Uuid>),
}

#[derive(Debug)]
pub struct PendingInput {
    request: InputRequest,
    waker: Option<Waker>,
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

    pub async fn wait_for_input(&self, request: InputRequest) -> Result<InputAnswer, GameError> {
        let request_id = Uuid::new_v4();

        {
            // 잠금 획득
            let mut state = self.state.lock().await;

            // 요청 상태를 추가
            state.insert(
                request_id,
                PendingInput {
                    request: request.clone(),
                    waker: None,
                    response: None,
                },
            );
        }

        // 요청을 처리하는 비동기 작업을 시작
        let future = InputFuture {
            state: self.state.clone(),
            request_id,
        };

        // 비동기 작업의 결과를 기다림
        let result = future.await;

        {
            // 잠금 획득 및, 처리된 요청을 삭제
            let mut state = self.state.lock().await;
            state.remove(&request_id);
        }

        // 결과 반환.
        result
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
        selections: InputAnswer,
    ) -> Result<(), GameError> {
        let mut state = self.state.lock().await;

        if let Some(pending) = state.get_mut(&request_id) {
            pending.response = Some(selections);

            // waker가 있으면 깨우기
            if let Some(waker) = pending.waker.take() {
                waker.wake();
            }

            Ok(())
        } else {
            Err(GameError::InvalidRequestId)
        }
    }
}

struct InputFuture {
    state: Arc<Mutex<HashMap<Uuid, PendingInput>>>,
    request_id: Uuid,
}

impl Future for InputFuture {
    type Output = Result<InputAnswer, GameError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let future = self.get_mut();

        // state 락 획득 시도
        match future.state.try_lock() {
            Ok(mut state) => {
                // 요청에 해당하는 ID 를 찾음
                if let Some(pending) = state.get_mut(&future.request_id) {
                    // 요청이 존재하면 응답을 확인
                    if let Some(response) = pending.response.take() {
                        // 응답이 있으면 완료
                        Poll::Ready(Ok(response))
                    } else {
                        // 응답이 없으면 waker 등록 후 Pending
                        pending.waker = Some(cx.waker().clone());
                        Poll::Pending
                    }
                } else {
                    // 요청이 없으면 에러
                    Poll::Ready(Err(GameError::InvalidRequestId))
                }
            }
            Err(_) => {
                // 락을 획득하지 못했으면 Pending
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}
