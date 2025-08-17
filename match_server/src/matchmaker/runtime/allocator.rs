use actix::prelude::Addr;
use uuid::Uuid;

use crate::provider::{DedicatedServerProvider, FindAvailableServer};

#[derive(Debug, Clone)]
pub struct CreateSessionResp {
    pub server_address: String,
    pub session_id: Uuid,
}

#[derive(Debug, Clone)]
pub enum CreateError {
    Provider,
    Mailbox,
    HttpTimeout,
    HttpError(u16),
    HttpOther(String),
    ResponseParse,
}

#[derive(serde::Serialize)]
struct CreateSessionReq {
    players: Vec<Uuid>,
}

pub async fn find_and_create(
    http: &reqwest::Client,
    provider: &Addr<DedicatedServerProvider>,
    timeout_secs: u64,
    _game_mode: &str,
    players: &[String],
) -> Result<CreateSessionResp, CreateError> {
    // Local load-test helper: bypass real provider/dedicated and return a fake session
    // Enable by setting env var SIM_FAKE_DEDICATED=1 (or "true").
    if matches!(
        std::env::var("SIM_FAKE_DEDICATED").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    ) {
        let session_id = Uuid::new_v4();
        // Use a stable fake address; test clients do not connect to it in current flows.
        let server_address = format!("ws://fake-dedicated/game?session_id={}", session_id);
        tracing::warn!(
            "SIM_FAKE_DEDICATED enabled: returning fake dedicated allocation for players: {:?}",
            players
        );
        return Ok(CreateSessionResp {
            server_address,
            session_id,
        });
    }

    // 1) Find available server via provider
    let server = match provider.send(FindAvailableServer).await {
        Ok(Ok(srv)) => srv,
        Ok(Err(_)) => return Err(CreateError::Provider),
        Err(_) => return Err(CreateError::Mailbox),
    };

    // 2) Compose request body
    let req_body = CreateSessionReq {
        players: players
            .iter()
            .filter_map(|id| Uuid::parse_str(id).ok())
            .collect(),
    };

    // 3) POST /session/create
    let url = format!("http://{}/session/create", server.address);
    let resp = match http
        .post(&url)
        .json(&req_body)
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            if e.is_timeout() {
                return Err(CreateError::HttpTimeout);
            }
            return Err(CreateError::HttpOther(e.to_string()));
        }
    };

    if !resp.status().is_success() {
        return Err(CreateError::HttpError(resp.status().as_u16()));
    }

    #[derive(serde::Deserialize)]
    struct CreateSessionRespWire {
        server_address: String,
        session_id: Uuid,
    }

    match resp.json::<CreateSessionRespWire>().await {
        Ok(w) => Ok(CreateSessionResp {
            server_address: w.server_address,
            session_id: w.session_id,
        }),
        Err(_) => Err(CreateError::ResponseParse),
    }
}
