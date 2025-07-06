//! Matchmaker 모듈은 매칭 관련 로직을 담당합니다.
//!
//! - `actor`: 매칭 대기열을 관리하고 실제 매칭을 수행하는 액터입니다.

pub mod actor;

// 다른 모듈에서 쉽게 사용할 수 있도록 공개합니다.
pub use actor::Matchmaker;