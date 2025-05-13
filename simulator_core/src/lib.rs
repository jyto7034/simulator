#![allow(unused_variables, unused_labels, dead_code)]

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use card::types::{PlayerIdentity, PlayerKind};
use exception::GameError;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use uuid::Uuid;

pub mod card;
pub mod card_gen;
pub mod effect;
pub mod enums;
pub mod exception;
pub mod game;
pub mod helper;
pub mod player;
pub mod resource;
pub mod selector;
pub mod server;
pub mod test;
pub mod utils;
pub mod zone;

extern crate lazy_static;

// static INIT: Once = Once::new();
// static mut GUARD: Option<tracing_appender::non_blocking::WorkerGuard> = None;
// pub fn setup_logger() {
//     INIT.call_once(|| {
//         let file_appender = RollingFileAppender::new(Rotation::HOURLY, "logs", "app.log");

//         let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

//         tracing_subscriber::fmt()
//             .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
//             .with_thread_ids(true)
//             .with_ansi(false)
//             .with_thread_names(true)
//             .with_file(true)
//             .with_line_number(true)
//             .with_target(false)
//             .with_writer(non_blocking)
//             .pretty()
//             .init();

//         unsafe {
//             GUARD = Some(_guard);
//         }
//     });
// }

use std::sync::Once;
static INIT: Once = Once::new();
static mut GUARD: Option<tracing_appender::non_blocking::WorkerGuard> = None;
pub fn setup_logger() {
    INIT.call_once(|| {
        // 1. 파일 로거 설정
        let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "app.log");
        let (non_blocking_file_writer, _guard) = tracing_appender::non_blocking(file_appender);

        // 2. 로그 레벨 필터 설정 (환경 변수 또는 기본값 INFO)
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")); // 기본 INFO 레벨

        // 3. 콘솔 출력 레이어 설정
        // let console_layer = fmt::layer()
        //     .with_writer(io::stdout) // 표준 출력으로 설정
        //     .with_ansi(true) // ANSI 색상 코드 사용 (터미널 지원 시)
        //     .with_thread_ids(true) // 스레드 ID 포함
        //     .with_thread_names(true) // 스레드 이름 포함
        //     .with_file(true) // 파일 경로 포함
        //     .with_line_number(true) // 라인 번호 포함
        //     .with_target(false) // target 정보 제외 (선택 사항)
        //     .pretty(); // 사람이 읽기 좋은 포맷

        // 4. 파일 출력 레이어 설정
        let file_layer = fmt::layer()
            .with_writer(non_blocking_file_writer) // Non-blocking 파일 로거 사용
            .with_ansi(false) // 파일에는 ANSI 코드 제외
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .pretty();

        // 5. 레지스트리(Registry)에 필터와 레이어 결합
        tracing_subscriber::registry()
            .with(filter) // 필터를 먼저 적용
            // .with(console_layer) // 콘솔 레이어 추가
            .with(file_layer) // 파일 레이어 추가
            .init(); // 전역 Subscriber로 설정

        unsafe {
            GUARD = Some(_guard);
        }

        tracing::info!("로거 초기화 완료: 콘솔 및 파일(logs/app.log) 출력 활성화.");
    });
}

#[derive(Clone)]
pub struct OptArc<T>(Option<ArcMutex<T>>);

impl<T> OptArc<T> {
    // 생성자들
    pub fn new(value: T) -> Self {
        Self(Some(ArcMutex::new(value)))
    }

    pub fn none() -> Self {
        Self(None)
    }

    pub fn from_option(opt: Option<T>) -> Self {
        opt.map(ArcMutex::new).into()
    }

    pub fn get(&self) -> MutexGuard<T> {
        self.0.as_ref().unwrap().get()
    }

    // Option 관련 메서드들
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    pub fn as_ref(&self) -> Option<&ArcMutex<T>> {
        self.0.as_ref()
    }

    pub fn take(&mut self) -> Option<ArcMutex<T>> {
        self.0.take()
    }

    pub fn replace(&mut self, value: T) -> Option<ArcMutex<T>> {
        self.0.replace(ArcMutex::new(value))
    }
}

impl<T> From<Option<ArcMutex<T>>> for OptArc<T> {
    fn from(opt: Option<ArcMutex<T>>) -> Self {
        Self(opt)
    }
}

impl<T> From<Option<T>> for OptArc<T> {
    fn from(opt: Option<T>) -> Self {
        Self(opt.map(ArcMutex::new))
    }
}

impl<T> From<T> for OptArc<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

// RcRef 구현 (스레드 안전 버전)
#[derive(Clone)]
pub struct ArcMutex<T>(Arc<Mutex<T>>);

impl<T> ArcMutex<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }

    pub fn get(&self) -> MutexGuard<T> {
        self.0.lock().unwrap()
    }
}

pub trait StringUuidExt {
    fn to_uuid(&self) -> Result<Uuid, GameError>;
}

impl StringUuidExt for String {
    fn to_uuid(&self) -> Result<Uuid, GameError> {
        Uuid::parse_str(self).map_err(|_| GameError::ParseError)
    }
}

pub trait VecUuidExt {
    fn to_vec_string(&self) -> Vec<String>;
}

impl VecUuidExt for Vec<Uuid> {
    fn to_vec_string(&self) -> Vec<String> {
        self.iter()
            .map(|uuid| uuid.to_string())
            .collect::<Vec<String>>()
    }
}

pub trait VecStringExt {
    fn to_vec_uuid(&self) -> Result<Vec<Uuid>, GameError>;
}

impl VecStringExt for Vec<String> {
    fn to_vec_uuid(&self) -> Result<Vec<Uuid>, GameError> {
        self.iter()
            .map(|uuid| Uuid::parse_str(uuid).map_err(|_| return GameError::ParseError))
            .collect::<Result<Vec<Uuid>, GameError>>()
    }
}

pub trait LogExt<T, E> {
    fn log_ok(self, f: impl FnOnce()) -> Self;
    fn log_err(self, f: impl FnOnce(&E)) -> Self;
}

impl<T, E> LogExt<T, E> for Result<T, E> {
    fn log_ok(self, f: impl FnOnce()) -> Self {
        if self.is_ok() {
            f()
        }
        self
    }

    fn log_err(self, f: impl FnOnce(&E)) -> Self {
        if let Err(ref e) = self {
            f(e);
        }
        self
    }
}
pub trait PlayerHashMapExt<V> {
    fn get_by_uuid(&self, uuid_key: &Uuid) -> Option<&V>;
    fn get_by_kind(&self, kind_key: PlayerKind) -> Option<&V>;
}

impl<V> PlayerHashMapExt<V> for HashMap<PlayerIdentity, V> {
    fn get_by_uuid(&self, uuid_key: &Uuid) -> Option<&V> {
        self.iter()
            .find(|(player_identity_key, _value)| player_identity_key.id == *uuid_key)
            .map(|(_player_identity_key, value)| value)
    }

    fn get_by_kind(&self, kind_key: PlayerKind) -> Option<&V> {
        self.iter()
            .find(|(player_identity_key, _value)| player_identity_key.kind == kind_key)
            .map(|(_player_identity_key, value)| value)
    }
}
