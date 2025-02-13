use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod app;
pub mod card;
pub mod card_gen;
pub mod enums;
pub mod exception;
pub mod game;
pub mod resource;
pub mod selector;
pub mod server;
pub mod test;
pub mod unit;
pub mod utils;
pub mod zone;

#[macro_use]
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

    // 가져오기 메서드들
    pub fn get_mut(&self) -> MutexGuard<T> {
        self.0.as_ref().unwrap().get_mut()
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

    pub fn get_mut(&self) -> MutexGuard<T> {
        self.0.lock().unwrap()
    }

    pub fn get(&self) -> MutexGuard<T> {
        self.0.lock().unwrap()
    }
}
