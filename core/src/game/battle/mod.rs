pub mod ability_executor;
pub mod buffs;
pub mod cooldown;
pub mod damage;
pub mod death;
pub mod enums;
pub mod replay;
pub mod timeline;
pub mod validation;

mod core;
mod types;

pub use cooldown::{CooldownSource, SourcedEffect};
pub use core::BattleCore;
pub use timeline::{HpChangeReason, Timeline, TimelineEntry, TimelineEvent, TIMELINE_VERSION};
pub use types::{
    BattleResult, BattleWinner, Event, GrowthId, GrowthStack, OwnedArtifact, OwnedItem, OwnedUnit,
    PlayerDeckInfo,
};
