use sqlx::PgPool;

// --- AppState: 서버 전체에서 공유될 상태 ---
// 액터 주소와 DB 커넥션 풀을 포함합니다.
#[derive(Clone)]
pub struct AppState {
    pub http_client: reqwest::Client,
    pub db_pool: PgPool,
    pub steam_web_api_key: String,
    pub app_id: u32,
    pub expected_identity: String,
    pub jwt_secret: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct SteamApiResponse {
    pub response: SteamApiResponseDetails,
}

#[derive(serde::Deserialize, Debug)]
pub struct SteamApiResponseDetails {
    pub params: Option<SteamApiResponseParams>,
    pub error: Option<SteamApiError>,
}

#[derive(serde::Deserialize, Debug)]
pub struct SteamApiResponseParams {
    pub result: String,
    pub steamid: String,
    pub ownersteamid: String,
    pub vacbanned: bool,
    pub publisherbanned: bool,
}

#[derive(serde::Deserialize, Debug)]
pub struct SteamApiError {
    pub errorcode: i32,
    pub errordesc: String,
}

// --- JWT Claims ---
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Claims {
    pub sub: String, // Subject (user's steam_id)
    pub iat: usize,  // Issued at (timestamp)
    pub exp: usize,  // Expiration time (timestamp)
}
