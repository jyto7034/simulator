use actix_web::{get, web, HttpResponse, Responder};
use crate::AppState;

#[get("/admin/test/reset")]
pub async fn test_reset(query: web::Query<ResetQuery>, state: web::Data<AppState>) -> impl Responder {
    let run_id = query.run_id.clone();
    if let Ok(mut guard) = state.current_run_id.write() {
        *guard = Some(run_id.clone());
    }
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "run_id": run_id,
        "ts": chrono::Utc::now().to_rfc3339(),
    }))
}

#[derive(serde::Deserialize)]
pub struct ResetQuery { pub run_id: String }

