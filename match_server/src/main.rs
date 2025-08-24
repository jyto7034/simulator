use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use match_server::{extract_client_ip, session::ws_session::Session, AppState};
use std::time::Duration;
use tracing::error;

#[get("/ws/")]
async fn matchmaking_ws_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let client_ip = extract_client_ip(&req).ok_or_else(|| {
        error!("Failed to extract client IP - rejecting connection");
        actix_web::error::ErrorBadRequest("Unable to determine client IP")
    })?;

    let session = Session::new(
        state.matchmaker_addr.clone(),
        state.sub_manager_addr.clone(),
        Duration::from_secs(state.settings.matchmaking.heartbeat_interval_seconds),
        Duration::from_secs(state.settings.matchmaking.client_timeout_seconds),
        state.clone(),
        client_ip,
    );

    ws::start(session, &req, stream)
}

fn main() {}
