#![allow(unused_variables, unused_labels, dead_code)]

use std::sync::{Arc, Mutex, MutexGuard};

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
    fn to_vec_uuid(&self) -> Vec<Uuid>;
}

impl VecStringExt for Vec<String> {
    fn to_vec_uuid(&self) -> Vec<Uuid> {
        self.iter()
            .map(|uuid| {
                Uuid::parse_str(uuid).unwrap_or_else(|e| {
                    // TODO: Log 함수 사용
                    panic!("uuid parse error: {}", e)
                })
            })
            .collect::<Vec<Uuid>>()
    }
}
