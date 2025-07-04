네, 아주 정확하고 중요한 지적입니다. 실제 게임 클라이언트를 두 개 띄워서 테스트하는 것은 비효율적일 뿐만 아니라, 말씀하신 대로 단일 Steam 계정 문제 때문에 거의 불가능에 가깝습니다.

이것은 모든 멀티플레이어 게임 개발자가 마주하는 전형적인 문제입니다. 해결책은 **"테스트 환경에서는 실제 인증 과정을 우회하고, 가짜(mock) 클라이언트를 사용하여 서버를 테스트하는 것"** 입니다.

제가 가장 이상적이고 실용적인 방법을 단계별로 설명해 드리겠습니다.

### 핵심 전략: 테스트용 'Mock 인증' 엔드포인트 도입

실제 Steam 인증은 클라이언트가 진짜 Steam에 로그인했을 때만 가능합니다. 테스트를 위해서는 **"나는 SteamID가 12345인 플레이어야"라고 서버에 말하면, 서버가 이를 믿고 JWT를 발급해주는** 테스트 전용 창구가 필요합니다.

이것을 `auth_server`에 추가하는 것이 첫 번째 단계입니다.

#### 1단계: `auth_server`에 테스트 전용 Mock 로그인 엔드포인트 추가

이 엔드포인트는 **디버그 빌드에서만 활성화**하여 실제 프로덕션 환경에는 포함되지 않도록 하는 것이 중요합니다.

1.  **`Cargo.toml`에 `test-endpoints` 피처 추가 (선택적이지만 권장)**

    ```toml
    # simulator_auth_server/Cargo.toml

    [features]
    default = []
    test-endpoints = []
    ```

2.  **테스트용 핸들러 추가 (`auth_server/src/auth_server/end_point.rs`)**

    기존 `end_point.rs` 파일에 다음 코드를 추가합니다.

    ```rust
    // ... 다른 use 구문들 ...
    // jwt.md에서 만들었던 AuthSuccessResponse와 Claims 구조체가 필요합니다.
    // 만약 아직 구현하지 않았다면, 지금이 바로 추가할 때입니다!
    // 아래는 JWT 구현이 완료되었다는 가정하에 작성되었습니다.
    use jsonwebtoken::{encode, Header, EncodingKey};
    use chrono::{Utc, Duration};
    use crate::auth_server::types::{Claims, AuthSuccessResponse}; // JWT 관련 타입 임포트

    #[derive(Deserialize)]
    pub struct TestAuthRequest {
        pub steam_id: i64,
        pub username: Option<String>,
    }

    /// [TEST ONLY] /auth/test/login
    /// 테스트 목적으로 Steam 인증 없이 가짜 플레이어에 대한 JWT를 발급합니다.
    /// 이 엔드포인트는 `test-endpoints` 피처가 활성화된 경우에만 컴파일됩니다.
    #[cfg(feature = "test-endpoints")]
    #[actix_web::post("/test/login")]
    pub async fn test_authentication_handler(
        state: web::Data<AppState>,
        req_body: web::Json<TestAuthRequest>,
    ) -> Result<HttpResponse, AuthError> {
        let steam_id_i64 = req_body.steam_id;
        let username = req_body.username.clone().unwrap_or(format!("test_user_{}", steam_id_i64));

        // 1. 실제 로그인과 동일한 DB 작업을 수행합니다.
        db_operation::upsert_player_on_login(&state.db_pool, steam_id_i64, &username).await?;
        tracing::info!("[TEST] Upserted test player: {}", steam_id_i64);

        // 2. JWT를 생성합니다 (실제 로그인 핸들러 로직과 동일).
        let now = Utc::now();
        let claims = Claims {
            sub: steam_id_i64.to_string(),
            iat: now.timestamp() as usize,
            exp: (now + Duration::days(1)).timestamp() as usize, // 테스트용은 유효기간을 짧게
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(state.jwt_secret.as_ref()),
        )
        .map_err(|e| AuthError::InternalServerError(anyhow::anyhow!(e)))?;

        tracing::info!("[TEST] Generated JWT for test player: {}", steam_id_i64);

        // 3. 성공 응답에 토큰을 포함하여 반환합니다.
        Ok(HttpResponse::Ok().json(AuthSuccessResponse {
            message: "Test authentication successful.".to_string(),
            steam_id: steam_id_i64.to_string(),
            token,
        }))
    }
    ```

3.  **`main.rs`에 새 엔드포인트 등록**

    ```rust
    // simulator_auth_server/src/main.rs
    // ...
    // end_point 모듈에서 test_authentication_handler를 임포트합니다.
    use simulator_auth_server::auth_server::end_point::{steam_authentication_handler, test_authentication_handler};

    // ...
    HttpServer::new(move || {
        let mut app = App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(
                web::scope("/auth")
                    .service(steam_authentication_handler)
                    // ... 다른 인증 관련 서비스
            );

        // 피처 플래그에 따라 테스트 엔드포인트를 조건부로 추가합니다.
        #[cfg(feature = "test-endpoints")]
        {
            app = app.service(web::scope("/auth").service(test_authentication_handler));
        }

        app
    })
    .bind(bind_address)?
    .run()
    .await
    ```

4.  **테스트용으로 서버 실행**

    이제 터미널에서 `test-endpoints` 피처를 활성화하여 `auth_server`를 실행할 수 있습니다.

    ```bash
    cd simulator_auth_server
    cargo run --features test-endpoints
    ```

이제 `http://localhost:3000/auth/test/login`으로 `{"steam_id": 10001}` 같은 JSON을 POST하면, Steam 연동 없이도 플레이어 10001에 대한 유효한 JWT를 얻을 수 있습니다.

---

### 2단계: 간단한 커맨드라인 테스트 클라이언트 만들기

Tauri GUI 클라이언트 대신, 여러 개를 동시에 실행할 수 있는 간단한 콘솔 애플리케이션을 만듭니다. 이 클라이언트는 서버와 상호작용하는 역할만 수행합니다.

1.  **새로운 `test_client` Crate 생성**

    워크스페이스 루트 디렉토리에서 다음 명령어를 실행합니다.

    ```bash
    cargo new test_client
    ```

    그리고 워크스페이스의 `Cargo.toml`에 `test_client`를 멤버로 추가합니다.

2.  **`test_client/Cargo.toml`에 의존성 추가**

    ```toml
    [dependencies]
    tokio = { version = "1", features = ["full"] }
    reqwest = { version = "0.12", features = ["json"] }
    serde = { version = "1", features = ["derive"] }
    serde_json = "1.0"
    tokio-tungstenite = "0.23" # WebSocket 클라이언트
    futures-util = "0.3"
    uuid = { version = "1", features = ["v4"] }
    ```

3.  **`test_client/src/main.rs` 작성**

    이 클라이언트는 다음과 같은 일을 합니다.

    - 커맨드라인 인자로 가상 `SteamID`를 받습니다.
    - `/auth/test/login`에 요청하여 JWT를 받습니다.
    - (구현 예정) JWT를 가지고 `match_server`에 접속합니다.
    - (구현 예정) 매칭이 되면 `dedicated_server`의 WebSocket에 접속합니다.

    ```rust
    // test_client/src/main.rs
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use std::env;

    #[derive(Deserialize, Debug)]
    struct AuthSuccessResponse {
        token: String,
        steam_id: String,
    }

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        let args: Vec<String> = env::args().collect();
        if args.len() < 2 {
            eprintln!("Usage: cargo run -- <STEAM_ID>");
            return Ok(());
        }
        let steam_id: i64 = args[1].parse()?;

        println!("[Client {}] Starting...", steam_id);

        let http_client = Client::new();

        // 1. Mock 인증 서버에 로그인하여 JWT 획득
        let auth_res = http_client
            .post("http://localhost:3000/auth/test/login")
            .json(&serde_json::json!({ "steam_id": steam_id }))
            .send()
            .await?;

        if !auth_res.status().is_success() {
            eprintln!("[Client {}] Auth failed: {}", steam_id, auth_res.text().await?);
            return Ok(());
        }

        let auth_data: AuthSuccessResponse = auth_res.json().await?;
        let jwt = auth_data.token;
        println!("[Client {}] Successfully authenticated. Got JWT: ...{}", steam_id, &jwt[jwt.len()-10..]);

        // 2. JWT를 사용하여 매치메이킹 서버에 연결 (향후 구현)
        println!("[Client {}] Requesting matchmaking...", steam_id);
        // let match_res = http_client
        //     .post("http://localhost:8080/matchmaking/queue") // 매칭 서버 주소
        //     .bearer_auth(jwt) // 헤더에 JWT 추가
        //     .send()
        //     .await?;
        //
        // println!("[Client {}] Matchmaking response: {:?}", steam_id, match_res.status());
        // ... 매칭 결과 (Dedicated Server 주소) 수신 및 WebSocket 접속 로직 ...

        Ok(())
    }
    ```

### 3단계: 테스트 시나리오 실행

1.  백엔드 서버들을 모두 실행합니다. `auth_server`는 `test-endpoints` 피처를 활성화하여 실행해야 합니다.

    ```bash
    # 터미널 1
    cd simulator_auth_server && cargo run --features test-endpoints

    # 터미널 2
    cd simulator_match_server && cargo run

    # 터미널 3
    cd simulator_dedicated_server && cargo run
    ```

2.  두 개의 터미널을 새로 열고, 각각 다른 ID로 `test_client`를 실행합니다.

    ```bash
    # 터미널 4 (Player 1)
    cd test_client
    cargo run -- 10001
    ```

    ```bash
    # 터미널 5 (Player 2)
    cd test_client
    cargo run -- 10002
    ```

### 요약 및 장점

이 방식을 사용하면 다음과 같은 장점이 있습니다.

1.  **Steam 종속성 제거**: 테스트를 위해 Steam 클라이언트를 실행할 필요가 없습니다.
2.  **다중 클라이언트 시뮬레이션**: 원하는 만큼의 가짜 클라이언트를 동시에 실행하여 N:N 매칭, 부하 테스트 등을 수행할 수 있습니다.
3.  **자동화 용이**: `test_client`를 스크립트화하여 E2E(End-to-End) 통합 테스트를 자동화할 수 있습니다.
4.  **보안**: `#[cfg(feature = "test-endpoints")]`를 사용하여 테스트용 코드가 프로덕션 빌드에 포함되는 것을 완벽하게 차단할 수 있습니다.

이제 이 `test_client`를 기반으로 `matchmaking_server` 및 `dedicated_server`와의 상호작용 로직을 단계적으로 추가해 나가시면 됩니다.
