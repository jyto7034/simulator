use std::fmt;

use serde::{de, Deserialize, Deserializer, Serialize};

use crate::{game::battle::ids::UnitInstanceId, game::resources::Position};

/// Canonical world-space scale for the continuous movement rewrite.
///
/// One board tile maps to one world unit. Tile coordinates are still useful for
/// formation input and debug projection, but runtime movement should use this
/// coordinate space as the source of truth.
pub const WORLD_UNITS_PER_TILE: f32 = 1.0;

/// Quantization scale for event_log/debug output when exact JSON stability is
/// more important than writing raw floating point values.
pub const EVENT_LOG_POSITION_QUANTIZATION: f32 = 1_000.0;

/// Fixed-point data scale used by authored ranges, projectile speeds, and area sizes.
///
/// Runtime movement uses world units. Authored combat data can keep integer
/// precision by expressing one world unit as this many data units.
pub const DATA_UNITS_PER_WORLD: f32 = 1_000_000.0;

/// Conservative starting body radius. Final tuning should live in unit data.
pub const DEFAULT_UNIT_RADIUS: f32 = 0.35;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiniteBattleValueKind {
    PositionX,
    PositionY,
    Radius,
    Speed,
    TimeSeconds,
}

impl fmt::Display for FiniteBattleValueKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::PositionX => "position.x",
            Self::PositionY => "position.y",
            Self::Radius => "radius",
            Self::Speed => "speed",
            Self::TimeSeconds => "time_seconds",
        };
        formatter.write_str(name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiniteBattleValueError {
    NonFinite { kind: FiniteBattleValueKind },
    Negative { kind: FiniteBattleValueKind },
}

impl fmt::Display for FiniteBattleValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite { kind } => write!(formatter, "{kind} must be finite"),
            Self::Negative { kind } => write!(formatter, "{kind} must be non-negative"),
        }
    }
}

impl std::error::Error for FiniteBattleValueError {}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FiniteWorldRadius(f32);

impl FiniteWorldRadius {
    pub fn try_new(value: f32) -> Result<Self, FiniteBattleValueError> {
        checked_non_negative(value, FiniteBattleValueKind::Radius).map(Self)
    }

    pub const fn get(self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FiniteWorldSpeed(f32);

impl FiniteWorldSpeed {
    pub fn try_new(value: f32) -> Result<Self, FiniteBattleValueError> {
        checked_non_negative(value, FiniteBattleValueKind::Speed).map(Self)
    }

    pub const fn get(self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FiniteBattleSeconds(f32);

impl FiniteBattleSeconds {
    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn try_new(value: f32) -> Result<Self, FiniteBattleValueError> {
        checked_non_negative(value, FiniteBattleValueKind::TimeSeconds).map(Self)
    }

    pub fn from_millis(milliseconds: u64) -> Self {
        // Every u64 converts to a finite f32, and division by this non-zero
        // finite constant remains finite.
        Self(milliseconds as f32 / 1_000.0)
    }

    pub const fn get(self) -> f32 {
        self.0
    }
}

fn checked_non_negative(
    value: f32,
    kind: FiniteBattleValueKind,
) -> Result<f32, FiniteBattleValueError> {
    if !value.is_finite() {
        return Err(FiniteBattleValueError::NonFinite { kind });
    }
    if value < 0.0 {
        return Err(FiniteBattleValueError::Negative { kind });
    }
    Ok(value)
}

/// Finite continuous-space position owned by the private battle kernel.
///
/// Raw coordinates are crate-private. External test capability can construct a
/// value from floats only through [`WorldVec2::try_new`], and deserialization
/// applies the same rejection rule.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct WorldVec2 {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl WorldVec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn try_new(x: f32, y: f32) -> Result<Self, FiniteBattleValueError> {
        if !x.is_finite() {
            return Err(FiniteBattleValueError::NonFinite {
                kind: FiniteBattleValueKind::PositionX,
            });
        }
        if !y.is_finite() {
            return Err(FiniteBattleValueError::NonFinite {
                kind: FiniteBattleValueKind::PositionY,
            });
        }
        Ok(Self { x, y })
    }

    pub(crate) const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn x(self) -> f32 {
        self.x
    }

    pub const fn y(self) -> f32 {
        self.y
    }

    pub fn from_tile_center(tile: Position) -> Self {
        Self {
            x: (tile.x as f32 + 0.5) * WORLD_UNITS_PER_TILE,
            y: (tile.y as f32 + 0.5) * WORLD_UNITS_PER_TILE,
        }
    }

    pub fn from_data_units(x_units: i64, y_units: i64) -> Self {
        Self {
            x: x_units as f32 / DATA_UNITS_PER_WORLD,
            y: y_units as f32 / DATA_UNITS_PER_WORLD,
        }
    }

    pub fn project_to_tile(self) -> Position {
        Position::new(
            (self.x / WORLD_UNITS_PER_TILE).floor() as i32,
            (self.y / WORLD_UNITS_PER_TILE).floor() as i32,
        )
    }

    pub fn distance_squared(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    pub fn distance(self, other: Self) -> f32 {
        self.distance_squared(other).sqrt()
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalized_or_zero(self) -> Self {
        let length = self.length();
        if length <= f32::EPSILON {
            Self::ZERO
        } else {
            Self::new(self.x / length, self.y / length)
        }
    }

    pub fn clamp_length(self, max_length: f32) -> Self {
        if max_length <= 0.0 {
            return Self::ZERO;
        }

        let length = self.length();
        if length <= max_length || length <= f32::EPSILON {
            self
        } else {
            let scale = max_length / length;
            Self::new(self.x * scale, self.y * scale)
        }
    }

    pub fn quantized_milli(self) -> EventLogVec2 {
        EventLogVec2 {
            x_milli: (self.x * EVENT_LOG_POSITION_QUANTIZATION).round() as i32,
            y_milli: (self.y * EVENT_LOG_POSITION_QUANTIZATION).round() as i32,
        }
    }

    pub fn to_data_units(self) -> (i64, i64) {
        (
            (self.x * DATA_UNITS_PER_WORLD).round() as i64,
            (self.y * DATA_UNITS_PER_WORLD).round() as i64,
        )
    }
}

impl<'de> Deserialize<'de> for WorldVec2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawWorldVec2 {
            x: f32,
            y: f32,
        }

        let raw = RawWorldVec2::deserialize(deserializer)?;
        Self::try_new(raw.x, raw.y).map_err(de::Error::custom)
    }
}

impl std::ops::Add for WorldVec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::AddAssign for WorldVec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::Sub for WorldVec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Mul<f32> for WorldVec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

#[derive(Debug, Clone, Copy)]
pub(in crate::game::battle::core) struct ActiveMovementSegment {
    pub(in crate::game::battle::core) start: WorldVec2,
    pub(in crate::game::battle::core) target: WorldVec2,
    pub(in crate::game::battle::core) started_at_ms: u64,
    pub(in crate::game::battle::core) ends_at_ms: u64,
}

crate::authoring::clone_shape::copy_shape!(ActiveMovementSegment);

impl ActiveMovementSegment {
    pub(in crate::game::battle::core) fn sample_position_at(&self, time_ms: u64) -> WorldVec2 {
        if time_ms <= self.started_at_ms {
            return self.start;
        }
        if time_ms >= self.ends_at_ms {
            return self.target;
        }

        let duration_ms = self.ends_at_ms.saturating_sub(self.started_at_ms);
        if duration_ms == 0 {
            return self.target;
        }

        let elapsed_ms = time_ms.saturating_sub(self.started_at_ms);
        let t = elapsed_ms as f32 / duration_ms as f32;
        self.start + (self.target - self.start) * t
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventLogVec2 {
    pub x_milli: i32,
    pub y_milli: i32,
}

impl EventLogVec2 {
    pub fn to_world(self) -> WorldVec2 {
        WorldVec2::new(
            self.x_milli as f32 / EVENT_LOG_POSITION_QUANTIZATION,
            self.y_milli as f32 / EVENT_LOG_POSITION_QUANTIZATION,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MovementGoal {
    AttackUnit { target_id: UnitInstanceId },
    MoveToPoint { point: WorldVec2, stop_radius: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UnitBody {
    pub(crate) position: WorldVec2,
    pub(crate) radius: f32,
    pub(crate) move_speed: f32,
}

impl Default for UnitBody {
    fn default() -> Self {
        Self {
            position: WorldVec2::ZERO,
            radius: DEFAULT_UNIT_RADIUS,
            move_speed: 0.0,
        }
    }
}

impl UnitBody {
    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn try_new_at(
        position: WorldVec2,
        radius: f32,
        move_speed: f32,
    ) -> Result<Self, FiniteBattleValueError> {
        let radius = FiniteWorldRadius::try_new(radius)?;
        let move_speed = FiniteWorldSpeed::try_new(move_speed)?;
        Ok(Self::from_finite(position, radius, move_speed))
    }

    pub(crate) const fn from_finite(
        position: WorldVec2,
        radius: FiniteWorldRadius,
        move_speed: FiniteWorldSpeed,
    ) -> Self {
        Self {
            position,
            radius: radius.get(),
            move_speed: move_speed.get(),
        }
    }

    #[cfg(test)]
    pub(crate) fn new_at(position: WorldVec2, radius: f32, move_speed: f32) -> Self {
        Self::try_new_at(position, radius, move_speed)
            .expect("internal unit body inputs must be finite and non-negative")
    }

    pub fn projected_tile(&self) -> Position {
        self.position.project_to_tile()
    }

    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn distance_to(&self, other: &Self) -> f32 {
        self.position.distance(other.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_center_conversion_uses_half_tile_offset() {
        let pos = WorldVec2::from_tile_center(Position::new(2, 3));
        assert_eq!(pos, WorldVec2::new(2.5, 3.5));
        assert_eq!(pos.project_to_tile(), Position::new(2, 3));
    }

    #[test]
    fn data_unit_conversion_round_trips_through_world_space() {
        let pos = WorldVec2::from_data_units(1_250_000, -500_000);
        assert_eq!(pos, WorldVec2::new(1.25, -0.5));
        assert_eq!(pos.to_data_units(), (1_250_000, -500_000));
    }

    #[test]
    fn event_log_quantization_round_trips_to_world_scale() {
        let point = WorldVec2::new(1.2345, 6.7894).quantized_milli();

        assert_eq!(
            point,
            EventLogVec2 {
                x_milli: 1235,
                y_milli: 6789
            }
        );
        assert_eq!(point.to_world(), WorldVec2::new(1.235, 6.789));
    }
}
