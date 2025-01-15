use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

pub mod app;
pub mod card;
pub mod card_gen;
pub mod enums;
pub mod exception;
pub mod game;
pub mod procedure;
pub mod server;
pub mod test;
pub mod unit;
pub mod utils;
pub mod zone;

#[macro_use]
extern crate lazy_static;

#[derive(Clone)]
pub struct OptRcRef<T>(Option<RcRef<T>>);

impl<T> OptRcRef<T> {
    // 생성자들
    pub fn new(value: T) -> Self {
        Self(Some(RcRef::new(value)))
    }

    pub fn none() -> Self {
        Self(None)
    }

    pub fn from_option(opt: Option<T>) -> Self {
        opt.map(RcRef::new).into()
    }

    // 가져오기 메서드들
    pub fn get_mut(&self) -> RefMut<T> {
        self.0.as_ref().unwrap().get_mut()
    }

    pub fn get(&self) -> Ref<T> {
        self.0.as_ref().unwrap().get()
    }

    // Option 관련 메서드들
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    pub fn as_ref(&self) -> Option<&RcRef<T>> {
        self.0.as_ref()
    }

    pub fn take(&mut self) -> Option<RcRef<T>> {
        self.0.take()
    }

    pub fn replace(&mut self, value: T) -> Option<RcRef<T>> {
        self.0.replace(RcRef::new(value))
    }
}

impl<T> From<Option<RcRef<T>>> for OptRcRef<T> {
    fn from(opt: Option<RcRef<T>>) -> Self {
        Self(opt)
    }
}

impl<T> From<Option<T>> for OptRcRef<T> {
    fn from(opt: Option<T>) -> Self {
        Self(opt.map(RcRef::new))
    }
}

impl<T> From<T> for OptRcRef<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

// RcRef 구현 (이전과 동일)
#[derive(Clone)]
pub struct RcRef<T>(Rc<RefCell<T>>);

impl<T> RcRef<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn get_mut(&self) -> RefMut<T> {
        self.0.as_ref().borrow_mut()
    }

    pub fn get(&self) -> Ref<T> {
        self.0.as_ref().borrow()
    }
}
