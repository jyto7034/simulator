# 🌍 Simulator 환경 설정 시스템

## 📋 개요

`simulator_env`는 전체 Simulator 프로젝트를 위한 통합 환경 설정 시스템입니다. 하드코딩된 URL들을 제거하고 중앙 집중식으로 모든 설정을 관리합니다.

## 🚀 사용 방법

### 기본 사용법

```rust
use simulator_env::env;

// 환경 설정 초기화
simulator_env::init()?;

// 서버 URL들 가져오기
let match_server_url = env::match_server_url();
let auth_server_url = env::auth_server_url();
let redis_url = env::redis_url();
```

### 테스트 코드 예시

```rust
use simulator_env::env;
use test_client::scenario::TestScenario;

#[actix_web::test]
async fn run_example_test() -> Result<()> {
    // 환경 설정 초기화
    simulator_env::init()?;
    
    // 설정에서 자동으로 URL 가져오기
    let mut scenario = TestScenario::setup_normal_match_test();
    
    let result = scenario.run().await?;
    assert!(result.is_success());
    
    Ok(())
}
```

## 🔧 설정 방법

### 1. 설정 파일 (`simulator.toml`)

```toml
[servers]
[servers.match_server]
host = "127.0.0.1"
port = 8080
use_tls = false

[servers.auth_server]
host = "127.0.0.1"
port = 8081
use_tls = false

[database]
[database.redis]
host = "127.0.0.1"
port = 6379
db = 0

[testing]
timeout_seconds = 30
parallel_tests = 4
```

### 2. 환경 변수 (`.env` 파일)

```bash
# 개발 환경
SIMULATOR_SERVERS_MATCH_SERVER_HOST=localhost
SIMULATOR_SERVERS_MATCH_SERVER_PORT=8080
SIMULATOR_LOGGING_LEVEL=debug

# 프로덕션 환경
SIMULATOR_SERVERS_MATCH_SERVER_HOST=match.example.com
SIMULATOR_SERVERS_MATCH_SERVER_USE_TLS=true
SIMULATOR_DATABASE_REDIS_PASSWORD=your_password
```

### 3. 코드에서 직접 설정

```rust
use simulator_env::SimulatorConfig;

// 개발 환경용
let config = SimulatorConfig::development();

// 프로덕션 환경용
let config = SimulatorConfig::production();

// 테스트 환경용
let config = SimulatorConfig::testing();
```

## 📂 설정 파일 위치

설정 파일은 다음 순서로 검색됩니다:

1. `$XDG_CONFIG_HOME/simulator/simulator.toml`
2. `$HOME/.config/simulator/simulator.toml`
3. `./config/simulator.toml`
4. `./simulator.toml`

## 🛠️ 주요 기능

### 환경별 설정 지원

```rust
// 환경에 따른 자동 설정
match std::env::var("ENVIRONMENT").as_deref() {
    Ok("production") => SimulatorConfig::production(),
    Ok("testing") => SimulatorConfig::testing(),
    _ => SimulatorConfig::development(),
}
```

### URL 생성 헬퍼

```rust
let endpoint = ServerEndpoint {
    host: "example.com".to_string(),
    port: 8080,
    use_tls: true,
};

println!("{}", endpoint.url());      // "https://example.com:8080"
println!("{}", endpoint.ws_url());   // "wss://example.com:8080"
println!("{}", endpoint.address());  // "example.com:8080"
```

### 편리한 헬퍼 함수들

```rust
use simulator_env::env;

// 자주 사용되는 설정들
let match_url = env::match_server_url();
let auth_url = env::auth_server_url();
let redis_url = env::redis_url();
let log_level = env::log_level();
let timeout = env::test_timeout_seconds();
```

## 🎯 장점

1. **중앙 집중식 관리**: 모든 설정을 한 곳에서 관리
2. **환경별 설정**: 개발/테스트/프로덕션 환경 자동 구분
3. **유연한 오버라이드**: 파일 → 환경변수 → 코드 순서로 설정 적용
4. **타입 안전성**: Rust의 타입 시스템으로 설정 검증
5. **편의성**: 헬퍼 함수로 간편한 접근

## 🔄 설정 우선순위

1. **환경 변수** (최우선)
2. **설정 파일** (`simulator.toml`)
3. **기본값** (코드에 정의된 디폴트)

## 📝 예시

### 기존 코드 (하드코딩)
```rust
let match_server_url = "127.0.0.1:8080".to_string();
let scenario = TestScenario::setup_normal_match_test(match_server_url);
```

### 개선된 코드 (환경 설정)
```rust
simulator_env::init()?;
let scenario = TestScenario::setup_normal_match_test();
```

이제 모든 URL과 설정이 중앙에서 관리되며, 환경에 따라 자동으로 적절한 값이 사용됩니다!