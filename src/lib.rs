#![allow(unused_variables, unused_labels, dead_code)]

use std::sync::{Arc, Mutex, MutexGuard};

use exception::GameError;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

pub mod app;
pub mod card;
pub mod card_gen;
pub mod enums;
pub mod exception;
pub mod game;
pub mod helper;
pub mod resource;
pub mod selector;
pub mod server;
pub mod test;
pub mod unit;
pub mod utils;
pub mod zone;

extern crate lazy_static;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct EffectId(Uuid);

impl From<Uuid> for EffectId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<EffectId> for Uuid {
    fn from(effect_id: EffectId) -> Self {
        effect_id.0
    }
}

use std::sync::Once;
static INIT: Once = Once::new();
static mut GUARD: Option<tracing_appender::non_blocking::WorkerGuard> = None;
pub fn setup_logger() {
    INIT.call_once(|| {
        let file_appender = RollingFileAppender::new(Rotation::HOURLY, "logs", "app.log");

        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
            .with_thread_ids(true)
            .with_ansi(false)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .with_writer(non_blocking)
            .pretty()
            .init();

        unsafe {
            GUARD = Some(_guard);
        }
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
