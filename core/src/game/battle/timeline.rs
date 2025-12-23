use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};
use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        ability::AbilityId,
        battle::buffs::BuffId,
        enums::Side,
        stats::{StatModifier, UnitStats},
    },
};

pub const TIMELINE_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttackKind {
    /// Scheduled auto attack (attack interval based).
    Auto,
    /// Triggered/one-off attack (e.g., ability extra attack).
    Triggered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub version: u32,
    pub entries: Vec<TimelineEntry>,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            version: TIMELINE_VERSION,
            entries: Vec::new(),
        }
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn to_pretty_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn write_json<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }

    pub fn write_pretty_json<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    pub fn read_json<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let timeline = serde_json::from_reader(reader)?;
        Ok(timeline)
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub time_ms: u64,
    pub seq: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause_seq: Option<u64>,
    pub event: TimelineEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TimelineEvent {
    BattleStart {
        width: u8,
        height: u8,
    },
    ArtifactSpawned {
        artifact_instance_id: Uuid,
        owner: Side,
        base_uuid: Uuid,
    },
    ItemSpawned {
        item_instance_id: Uuid,
        owner: Side,
        owner_unit_instance_id: Uuid,
        base_uuid: Uuid,
    },
    UnitSpawned {
        unit_instance_id: Uuid,
        owner: Side,
        base_uuid: Uuid,
        position: Position,
        stats: UnitStats,
    },
    Attack {
        attacker_instance_id: Uuid,
        target_instance_id: Uuid,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<AttackKind>,
    },
    AutoCastStart {
        caster_instance_id: Uuid,
        ability_id: Option<AbilityId>,
        target_instance_id: Option<Uuid>,
    },
    AutoCastEnd {
        caster_instance_id: Uuid,
    },
    AbilityCast {
        ability_id: AbilityId,
        caster_instance_id: Uuid,
        target_instance_id: Option<Uuid>,
    },
    BuffApplied {
        caster_instance_id: Uuid,
        target_instance_id: Uuid,
        buff_id: BuffId,
        duration_ms: u64,
    },
    BuffTick {
        caster_instance_id: Uuid,
        target_instance_id: Uuid,
        buff_id: BuffId,
    },
    BuffExpired {
        caster_instance_id: Uuid,
        target_instance_id: Uuid,
        buff_id: BuffId,
    },
    HpChanged {
        source_instance_id: Option<Uuid>,
        target_instance_id: Uuid,
        delta: i32,
        hp_before: u32,
        hp_after: u32,
        reason: HpChangeReason,
    },
    StatChanged {
        source_instance_id: Option<Uuid>,
        target_instance_id: Uuid,
        modifier: StatModifier,
        stats_before: UnitStats,
        stats_after: UnitStats,
    },
    UnitDied {
        unit_instance_id: Uuid,
        owner: Side,
        killer_instance_id: Option<Uuid>,
    },
    BattleEnd {
        winner: super::BattleWinner,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HpChangeReason {
    BasicAttack,
    Command,
}
