//! One typed table for producer validation, plan explanation and later consumers.
use super::authoring::{EffectOperation, RelicProperty, TypedRelicValue};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RelicUnit {
    Hp,
    PercentPoints,
    SpeedPoints,
    DataUnits,
    TilePattern,
}
#[derive(Debug, Clone, Copy, Serialize)]
pub struct RelicCapabilityDescriptor {
    pub operation: EffectOperation,
    pub property: RelicProperty,
    pub unit: RelicUnit,
    pub minimum: i64,
    pub maximum: i64,
    pub numeric: bool,
    /// The producer and the typed battle consumer use this same contract.
    pub runtime_connected: bool,
}
// Damage/Shield amount expressions narrow through the existing u32 consumer.
// Literal i32 producers remain their own authored representation.
const HP_MAX: i64 = u32::MAX as i64;
const RADIUS_MAX: i64 = u32::MAX as i64;
const fn c(
    op: EffectOperation,
    p: RelicProperty,
    u: RelicUnit,
    lo: i64,
    hi: i64,
    numeric: bool,
) -> RelicCapabilityDescriptor {
    RelicCapabilityDescriptor {
        operation: op,
        property: p,
        unit: u,
        minimum: lo,
        maximum: hi,
        numeric,
        runtime_connected: true,
    }
}
static CAPABILITIES: &[RelicCapabilityDescriptor] = &[
    c(
        EffectOperation::Damage,
        RelicProperty::DamageAmount,
        RelicUnit::Hp,
        0,
        HP_MAX,
        true,
    ),
    c(
        EffectOperation::Damage,
        RelicProperty::DamageFootprint,
        RelicUnit::TilePattern,
        0,
        0,
        false,
    ),
    c(
        EffectOperation::PersonalShieldGrant,
        RelicProperty::PersonalShieldInitialAmount,
        RelicUnit::Hp,
        1,
        HP_MAX,
        true,
    ),
    c(
        EffectOperation::PersonalShieldGrant,
        RelicProperty::SupportAreaRadius,
        RelicUnit::DataUnits,
        1,
        RADIUS_MAX,
        true,
    ),
    c(
        EffectOperation::ChannelAura,
        RelicProperty::SupportAreaRadius,
        RelicUnit::DataUnits,
        1,
        RADIUS_MAX,
        true,
    ),
    c(
        EffectOperation::ChannelAura,
        RelicProperty::RegularBasicDamagePercent,
        RelicUnit::PercentPoints,
        0,
        10000,
        true,
    ),
    c(
        EffectOperation::ChannelAura,
        RelicProperty::RegularBasicSpeedPoints,
        RelicUnit::SpeedPoints,
        0,
        10000,
        true,
    ),
];
pub fn structural_capabilities() -> &'static [RelicCapabilityDescriptor] {
    CAPABILITIES
}
pub(crate) fn descriptor(
    op: EffectOperation,
    p: RelicProperty,
) -> Option<&'static RelicCapabilityDescriptor> {
    CAPABILITIES
        .iter()
        .find(|d| d.operation == op && d.property == p)
}
impl TypedRelicValue {
    pub fn unit(self) -> RelicUnit {
        match self {
            Self::Hp(_) => RelicUnit::Hp,
            Self::PercentPoints(_) => RelicUnit::PercentPoints,
            Self::SpeedPoints(_) => RelicUnit::SpeedPoints,
            Self::DataUnits(_) => RelicUnit::DataUnits,
        }
    }
    pub fn value(self) -> i64 {
        match self {
            Self::Hp(v) | Self::PercentPoints(v) | Self::SpeedPoints(v) | Self::DataUnits(v) => v,
        }
    }
    pub(crate) fn with_value(self, v: i64) -> Self {
        match self {
            Self::Hp(_) => Self::Hp(v),
            Self::PercentPoints(_) => Self::PercentPoints(v),
            Self::SpeedPoints(_) => Self::SpeedPoints(v),
            Self::DataUnits(_) => Self::DataUnits(v),
        }
    }
}
