pub mod ability_executor;
pub mod buffs;
pub mod damage;
pub mod death;
pub mod enums;
pub mod timeline;
pub mod validation;
pub mod replay;

mod core;
mod types;

pub use core::BattleCore;
pub use timeline::{HpChangeReason, Timeline, TimelineEntry, TimelineEvent, TIMELINE_VERSION};
pub use types::{
    BattleResult, BattleWinner, Event, GrowthId, GrowthStack, OwnedArtifact, OwnedItem, OwnedUnit,
    PlayerDeckInfo,
};
