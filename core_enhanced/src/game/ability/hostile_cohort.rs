//! Shared hostile populations are independent of fragment shot resources.
use super::marks::MarkRatio;
use crate::game::battle::{damage::DamageType, ids::UnitInstanceId, tile_range::TileRangePattern};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostileCohortDef {
    pub definition: String,
    pub interval_ms: u32,
    pub hp: u32,
    pub power: u32,
    pub speed: MarkRatio,
    pub physical_defense: u32,
    pub magic_defense: u32,
    pub block: u32,
    pub leak: u32,
    pub damage_type: DamageType,
    pub attack_range: TileRangePattern,
    pub attack_interval_ms: u32,
    pub weakness_family: String,
    pub weakness_percent: MarkRatio,
    pub kill_gain: u32,
}
impl HostileCohortDef {
    pub fn validate(&self) -> Result<(), String> {
        self.validate_fields()
            .map_err(|(field, message)| format!("{field}: {message}"))
    }
    pub(crate) fn validate_fields(&self) -> Result<(), (String, String)> {
        let fail = |field: &str, message: &str| (field.to_owned(), message.to_owned());
        for (field, value) in [
            ("definition", &self.definition),
            ("weakness_family", &self.weakness_family),
        ] {
            if value.trim().is_empty() {
                return Err(fail(field, "hostile identity is required"));
            }
        }
        for (field, value) in [
            ("hp", self.hp),
            ("interval_ms", self.interval_ms),
            ("attack_interval_ms", self.attack_interval_ms),
            ("block", self.block),
            ("leak", self.leak),
            ("kill_gain", self.kill_gain),
        ] {
            if value == 0 {
                return Err(fail(field, "hostile value must be positive"));
            }
        }
        if self.damage_type == DamageType::True {
            return Err(fail(
                "damage_type",
                "hostile attack requires a defended damage type",
            ));
        }
        for (field, value) in [
            ("hp", self.hp),
            ("physical_defense", self.physical_defense),
            ("magic_defense", self.magic_defense),
        ] {
            if value > i32::MAX as u32 {
                return Err(fail(field, "hostile stats exceed native precision"));
            }
        }
        self.speed.validate(false).map_err(|e| fail("speed", &e))?;
        let speed = u128::from(self.speed.numerator) * 1000;
        let native = speed / u128::from(self.speed.denominator);
        if speed % u128::from(self.speed.denominator) != 0
            || native == 0
            || native > u128::from(u32::MAX)
        {
            return Err(fail("speed", "hostile speed exceeds native precision"));
        }
        self.weakness_percent
            .validate(true)
            .map_err(|e| fail("weakness_percent", &e))?;
        if self.weakness_percent.numerator % self.weakness_percent.denominator != 0
            || self.weakness_percent.numerator / self.weakness_percent.denominator > i32::MAX as u64
        {
            return Err(fail(
                "weakness_percent",
                "hostile weakness exceeds native precision",
            ));
        }
        self.attack_range
            .validate()
            .map_err(|e| fail("attack_range", &e))?;
        Ok(())
    }
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(64)?
            .checked_add(self.definition.capacity())?
            .checked_add(self.weakness_family.capacity())?
            .checked_add(crate::game::relic::structural::support::pattern_extent(
                &self.attack_range,
            )?)
    }
}
crate::authoring::clone_shape::struct_shape!(HostileCohortDef {
    definition,
    interval_ms,
    hp,
    power,
    speed,
    physical_defense,
    magic_defense,
    block,
    leak,
    damage_type,
    attack_range,
    attack_interval_ms,
    weakness_family,
    weakness_percent,
    kill_gain
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostileCohortProvider {
    Fragment { copy: Uuid },
    Equipment { copy: Uuid, definition: Uuid },
}
crate::authoring::clone_shape::copy_shape!(HostileCohortProvider);
impl HostileCohortProvider {
    pub(crate) fn copy(self) -> Uuid {
        match self {
            Self::Fragment { copy } | Self::Equipment { copy, .. } => copy,
        }
    }
    pub(crate) fn fragment(self) -> Option<Uuid> {
        match self {
            Self::Fragment { copy } => Some(copy),
            Self::Equipment { .. } => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedCohortOwnerView {
    pub id: Uuid,
    pub employee: Uuid,
    pub definition: String,
    pub providers: Vec<HostileCohortProvider>,
    pub next_spawn_ms: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedCohortUnitView {
    pub unit: UnitInstanceId,
    pub spawner: Uuid,
    pub fragment_recipient: Option<Uuid>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedCohortsSnapshot {
    pub owners: Vec<SharedCohortOwnerView>,
    pub units: Vec<SharedCohortUnitView>,
}
