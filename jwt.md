네, 아주 좋은 질문입니다. JWT(JSON Web Token) 도입은 "로그인 성공" 이후의 상태 관리를 위한 완벽한 다음 단계입니다. 현재 시스템은 1회성 인증만 처리하므로, JWT를 통해 "인증된 세션"을 만들어 클라이언트가 후속 요청에서 자신을 증명할 수 있게 됩니다.

제가 단계별로 아주 자세하게 설명해 드릴게요.

### **JWT 도입의 핵심 개념: 새로운 인증 흐름**

현재:

1.  **클라이언트** -> **서버**: "이 스팀 티켓 좀 봐줘."
2.  **서버** -> **스팀**: "이 티켓 유효해?"
3.  **스팀** -> **서버**: "응, 유효하고 SteamID는 OOO야."
4.  **서버** -> **클라이언트**: "인증 성공했어."
5.  (끝. 다음 요청 시 클라이언트가 누군지 알 수 없음)

JWT 도입 후:

1.  **클라이언트** -> **서버**: "이 스팀 티켓 좀 봐줘."
2.  **서버** -> **스팀**: "이 티켓 유효해?"
3.  **스팀** -> **서버**: "응, 유효하고 SteamID는 OOO야."
4.  **서버**: (DB에 플레이어 정보 저장 후) **"이 플레이어를 위한 JWT를 발급해야겠다!"**
5.  **서버**: JWT 생성 (내용: `playerId: OOO`, `만료시간: 7일 뒤`) 및 비밀 키로 서명.
6.  **서버** -> **클라이언트**: "인증 성공했어. 앞으로 이걸 신분증처럼 써. **(JWT 전달)**"
7.  --- (이후 모든 요청) ---
8.  **클라이언트**: (예: 내 프로필 정보 요청) "내 프로필 정보 줘. 내 신분증은 이거야. (`Authorization: Bearer [JWT]` 헤더에 추가)"
9.  **서버**: 클라이언트가 보낸 JWT를 자신의 비밀 키로 검증.
10. **서버**: "음, 서명이 유효하고 만료되지 않았군. 요청한 사람은 SteamID가 OOO인 플레이어야. 프로필 정보를 찾아서 줘야겠다."

자, 이제 이걸 코드로 구현해 보겠습니다.

---

### **1단계: JWT 라이브러리 추가**

가장 널리 쓰이는 `jsonwebtoken` 라이브러리를 `Cargo.toml`에 추가합니다.

```toml
[dependencies]
# ... 기존 의존성들
jsonwebtoken = "9.3.0"
chrono = { version = "0.4.41", features = ["serde"] } # chrono는 이미 있지만 serde 기능이 필요합니다.
```

### **2단계: JWT 비밀 키 설정**

JWT는 서버만 아는 비밀 키로 서명되어야 합니다. 이 키가 유출되면 누구나 유효한 토큰을 만들 수 있으므로, **절대로 코드에 하드코딩하면 안 됩니다.** `.env` 파일을 사용합시다.

1.  **.env 파일에 비밀 키 추가:**

    ```dotenv
    # .env
    DATABASE_URL=...
    STEAM_WEB_API_KEY=...
    EXPECTED_IDENTITY=...

    # 새로 추가
    JWT_SECRET="your-super-secret-and-long-key-that-no-one-can-guess"
    ```

    (실제로는 더 복잡하고 긴 문자열을 사용하세요.)

2.  **AppState에 JWT 비밀 키 추가:**
    `main.rs`에서 비밀 키를 로드하고, `AppState`를 수정하여 키를 저장합니다.

    `src/auth_server/types.rs`:

    ```rust
    // src/auth_server/types.rs

    #[derive(Clone)]
    pub struct AppState {
        pub http_client: reqwest::Client,
        pub db_pool: PgPool,
        pub steam_web_api_key: String,
        pub app_id: u32,
        pub expected_identity: String,
        pub jwt_secret: String, // 새로 추가
    }
    // ...
    ```

    `src/main.rs`:

    ```rust
    // src/main.rs
    async fn main() -> std::io::Result<()> {
        // ...
        let steam_web_api_key =
            std::env::var("STEAM_WEB_API_KEY").expect("STEAM_WEB_API_KEY must be set in .env file");

        // 새로 추가
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file");

        // ...
        let app_state = AppState {
            http_client: reqwest::Client::new(),
            db_pool,
            steam_web_api_key: steam_web_api_key.clone(),
            app_id: 480,
            expected_identity: std::env::var("EXPECTED_IDENTITY")
                .expect("EXPECTED_IDENTITY must be set in .env file"),
            jwt_secret, // 새로 추가
        };
        // ...
    }
    ```

### **3단계: JWT 페이로드(Claims) 정의**

토큰에 어떤 정보를 담을지 정의합니다. 필수 정보는 **누구인지(`sub`)**와 **언제까지 유효한지(`exp`)** 입니다.

`src/auth_server/types.rs` 에 추가하세요.

```rust
// src/auth_server/types.rs
use serde::{Deserialize, Serialize};

// ... AppState ...

// JWT Claims 구조체 (토큰의 내용)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (여기서는 Player's SteamID)
    pub exp: usize,  // Expiration time (timestamp)
    pub iat: usize,  // Issued at (timestamp)
}
// ...
```

### **4단계: 로그인 핸들러에서 JWT 생성 및 반환 (핵심!)**

이제 "로그인 성공 후 뭘 해야할지"에 대한 직접적인 답변입니다. `steam_authentication_handler`에서 인증이 성공하고 DB 작업이 끝나면, JWT를 생성해서 클라이언트에게 돌려줍니다.

`src/auth_server/end_point.rs`를 수정합니다.

```rust
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};

use crate::auth_server::{
    db_operation,
    errors::AuthError,
    types::{AppState, SteamApiResponse, Claims}, // Claims 추가
};

// ... SteamAuthRequest ...

#[derive(Deserialize, Serialize)]
struct AuthSuccessResponse {
    message: String,
    steam_id: String,
    token: String, // 새로 추가: JWT 토큰
}

// ...

#[actix_web::post("/steam")]
pub async fn steam_authentication_handler(
    state: web::Data<AppState>,
    req_body: web::Json<SteamAuthRequest>,
) -> Result<HttpResponse, AuthError> {
    // ... 기존 스팀 인증 로직 ...

    if let Some(params) = steam_response.response.params {
        if params.result == "OK" {
            let steam_id_u64 = params.steamid.parse::<u64>().map_err(|_| {
                AuthError::InternalServerError(anyhow::anyhow!("Steam returned invalid SteamID"))
            })?;

            tracing::info!(
                "Steam Web API authentication successful for SteamID: {}",
                steam_id_u64
            );

            // =========================================================
            // <<<<< 여기가 바로 JWT를 생성할 지점입니다! >>>>>
            // 1. (주석 해제) DB에 플레이어 정보 저장/업데이트
            let temp_username = format!("user_{}", steam_id_u64); // 실제로는 스팀에서 닉네임을 가져와야 함
            db_operation::upsert_player_on_login(
                &state.db_pool,
                steam_id_u64 as i64,
                &temp_username,
            )
            .await?;

            // 2. JWT Claims 생성
            let now = Utc::now();
            let iat = now.timestamp() as usize;
            let exp = (now + Duration::days(7)).timestamp() as usize; // 7일 유효기간
            let claims = Claims {
                sub: steam_id_u64.to_string(),
                iat,
                exp,
            };

            // 3. JWT 토큰 생성
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(state.jwt_secret.as_ref()),
            )
            .map_err(|e| AuthError::InternalServerError(anyhow::anyhow!(e)))?;

            // 4. 성공 응답에 토큰 포함하여 반환
            Ok(HttpResponse::Ok().json(AuthSuccessResponse {
                message: "Steam Web API authentication successful.".to_string(),
                steam_id: steam_id_u64.to_string(),
                token, // 생성된 토큰을 응답에 포함
            }))
            // =========================================================

        } else {
            // ... 기존 에러 처리 ...
            Err(AuthError::Unauthorized(format!(
                "Steam validation failed with result: {}",
                params.result
            )))
        }
    } else if let Some(error) = steam_response.response.error {
        // ... 기존 에러 처리 ...
        Err(AuthError::Unauthorized(format!(
            "Steam API Error {}: {}",
            error.errorcode, error.errordesc
        )))
    } else {
        // ... 기존 에러 처리 ...
        Err(AuthError::InternalServerError(anyhow::anyhow!(
            "Invalid response structure from Steam API"
        )))
    }
}
```

이제 로그인에 성공한 클라이언트는 `token` 필드에 담긴 JWT를 받게 됩니다.

### **5단계: 보호된 엔드포인트와 JWT 검증 미들웨어**

이제 JWT를 사용하는 방법을 만들어야 합니다. "내 프로필 정보 가져오기"와 같이 **로그인한 사용자만 접근할 수 있는 엔드포인트**를 보호해야 합니다.

가장 깔끔한 방법은 Actix-web의 `Extractor`를 만드는 것입니다. 요청이 핸들러에 도달하기 전에 헤더에서 JWT를 꺼내 검증하는 역할을 합니다.

1.  **새로운 파일 `src/auth_server/middleware.rs` 생성:**

    ```rust
    // src/auth_server/middleware.rs

    use actix_web::{dev::Payload, FromRequest, HttpRequest, web};
    use jsonwebtoken::{decode, Validation, DecodingKey};
    use std::future::{ready, Ready};

    use crate::auth_server::{
        errors::AuthError,
        types::{AppState, Claims},
    };

    pub struct AuthenticatedUser {
        pub id: i64, // 파싱된 플레이어 ID
    }

    impl FromRequest for AuthenticatedUser {
        type Error = AuthError;
        type Future = Ready<Result<Self, Self::Error>>;

        fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
            let state = req.app_data::<web::Data<AppState>>().unwrap();

            // 1. Authorization 헤더에서 토큰 추출
            let auth_header = match req.headers().get("Authorization") {
                Some(h) => h.to_str().unwrap_or(""),
                None => return ready(Err(AuthError::Unauthorized("Missing Authorization header".to_string()))),
            };

            if !auth_header.starts_with("Bearer ") {
                return ready(Err(AuthError::Unauthorized("Invalid token format".to_string())));
            }

            let token = &auth_header[7..];

            // 2. 토큰 디코딩 및 검증
            let claims = match decode::<Claims>(
                token,
                &DecodingKey::from_secret(state.jwt_secret.as_ref()),
                &Validation::default(),
            ) {
                Ok(c) => c.claims,
                Err(e) => {
                    let reason = format!("Token validation failed: {}", e);
                    return ready(Err(AuthError::Unauthorized(reason)));
                }
            };

            // 3. Claims에서 사용자 ID 파싱
            let user_id = match claims.sub.parse::<i64>() {
                Ok(id) => id,
                Err(_) => return ready(Err(AuthError::Unauthorized("Invalid user ID in token".to_string()))),
            };

            // 4. 성공 시 AuthenticatedUser 반환
            ready(Ok(AuthenticatedUser { id: user_id }))
        }
    }
    ```

2.  **`src/auth_server/mod.rs` 에 `middleware` 모듈 추가:**

    ```rust
    pub mod db_operation;
    pub mod end_point;
    pub mod errors;
    pub mod model;
    pub mod types;
    pub mod middleware; // 새로 추가
    ```

3.  **보호된 엔드포인트에서 Extractor 사용하기:**
    예시로 "내 프로필 정보"를 가져오는 엔드포인트를 만들어 보겠습니다.

    `src/auth_server/end_point.rs`에 추가:

    ```rust
    // ... 다른 use 구문들
    use crate::auth_server::middleware::AuthenticatedUser; // Extractor 임포트

    // ... 다른 핸들러들 ...

    #[actix_web::get("/me")]
    pub async fn get_my_profile(
        state: web::Data<AppState>,
        auth_user: AuthenticatedUser, // <<-- 여기! 이 인자 하나로 JWT 검증이 끝납니다.
    ) -> Result<HttpResponse, AuthError> {
        // 이 핸들러가 실행되었다는 것은 이미 JWT가 유효하다는 뜻입니다.
        // auth_user.id 에는 인증된 사용자의 SteamID가 들어있습니다.
        let player_id = auth_user.id;

        let profile = db_operation::get_player_profile(&state.db_pool, player_id)
            .await?
            .ok_or_else(|| AuthError::BadRequest(format!("Profile not found for player {}", player_id)))?;

        Ok(HttpResponse::Ok().json(profile))
    }
    ```

4.  **`main.rs`에 새 엔드포인트 등록:**
    ```rust
    // src/main.rs
    use simulator_auth_server::auth_server::{
        end_point::{steam_authentication_handler, get_my_profile}, // get_my_profile 추가
        types::AppState,
    };
    // ...
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(
                web::scope("/auth") // `/auth` 경로 그룹화
                    .service(steam_authentication_handler)
                    .service(get_my_profile) // 새로운 엔드포인트 등록
            )
    })
    // ...
    ```

### **정리**

이제 당신의 서버는 완벽한 JWT 기반 인증 흐름을 갖추게 되었습니다.

1.  클라이언트는 `POST /auth/steam`으로 로그인하고 JWT를 받습니다.
2.  클라이언트는 받은 JWT를 안전한 곳(예: 로컬 저장소)에 저장합니다.
3.  이후 `/auth/me`와 같은 보호된 API를 호출할 때마다, HTTP 요청의 `Authorization` 헤더에 `Bearer [JWT]` 형식으로 토큰을 담아 보냅니다.
4.  서버는 `AuthenticatedUser` Extractor를 통해 자동으로 토큰을 검증하고, 유효한 경우에만 핸들러 로직을 실행합니다.

이제 다른 API 엔드포인트(예: 덱 생성, 카드 목록 조회 등)를 만들 때 핸들러 인자에 `auth_user: AuthenticatedUser`만 추가하면 손쉽게 인증을 적용할 수 있습니다.
