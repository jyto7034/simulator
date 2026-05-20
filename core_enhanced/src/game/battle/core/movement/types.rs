use serde::{Deserialize, Serialize};

use crate::{game::battle::ids::UnitInstanceId, game::resources::Position};

/// Canonical world-space scale for the continuous movement rewrite.
///
/// One board tile maps to one world unit. Tile coordinates are still useful for
/// formation input and debug projection, but runtime movement should use this
/// coordinate space as the source of truth.
pub const WORLD_UNITS_PER_TILE: f32 = 1.0;

/// Default fixed movement step for the first continuous movement implementation.
pub const DEFAULT_MOVEMENT_TICK_MS: u64 = 50;

/// Quantization scale for timeline/debug output when exact JSON stability is
/// more important than writing raw floating point values.
pub const TIMELINE_POSITION_QUANTIZATION: f32 = 1_000.0;

/// Fixed-point data scale used by authored ranges, projectile speeds, and area sizes.
///
/// Runtime movement uses world units. Authored combat data can keep integer
/// precision by expressing one world unit as this many data units.
pub const DATA_UNITS_PER_WORLD: f32 = 1_000_000.0;

/// Conservative starting body radius. Final tuning should live in unit data.
pub const DEFAULT_UNIT_RADIUS: f32 = 0.35;

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldVec2 {
    pub x: f32,
    pub y: f32,
}

impl WorldVec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
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

    pub fn quantized_milli(self) -> TimelineVec2 {
        TimelineVec2 {
            x_milli: (self.x * TIMELINE_POSITION_QUANTIZATION).round() as i32,
            y_milli: (self.y * TIMELINE_POSITION_QUANTIZATION).round() as i32,
        }
    }

    pub fn to_data_units(self) -> (i64, i64) {
        (
            (self.x * DATA_UNITS_PER_WORLD).round() as i64,
            (self.y * DATA_UNITS_PER_WORLD).round() as i64,
        )
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineVec2 {
    pub x_milli: i32,
    pub y_milli: i32,
}

impl TimelineVec2 {
    pub fn to_world(self) -> WorldVec2 {
        WorldVec2::new(
            self.x_milli as f32 / TIMELINE_POSITION_QUANTIZATION,
            self.y_milli as f32 / TIMELINE_POSITION_QUANTIZATION,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementMode {
    Idle,
    Moving,
    MovementLocked { until_ms: u64 },
    Dead,
}

impl Default for MovementMode {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MovementGoal {
    AttackUnit {
        target_id: UnitInstanceId,
        desired_range: f32,
        approach_point: Option<WorldVec2>,
    },
    MoveToPoint {
        point: WorldVec2,
        stop_radius: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitPhysicsHandle {
    pub raw: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitBody {
    pub position: WorldVec2,
    pub previous_position: WorldVec2,
    pub velocity: WorldVec2,
    pub radius: f32,
    pub move_speed: f32,
    pub mode: MovementMode,
    pub goal: Option<MovementGoal>,
    pub physics_handle: Option<UnitPhysicsHandle>,
}

impl Default for UnitBody {
    fn default() -> Self {
        Self::new_at(WorldVec2::ZERO, DEFAULT_UNIT_RADIUS, 0.0)
    }
}

impl UnitBody {
    pub fn new_at(position: WorldVec2, radius: f32, move_speed: f32) -> Self {
        Self {
            position,
            previous_position: position,
            velocity: WorldVec2::ZERO,
            radius,
            move_speed,
            mode: MovementMode::Idle,
            goal: None,
            physics_handle: None,
        }
    }

    pub fn from_tile_center(tile: Position, radius: f32, move_speed: f32) -> Self {
        Self::new_at(WorldVec2::from_tile_center(tile), radius, move_speed)
    }

    pub fn from_data_units(x_units: i64, y_units: i64, radius: f32, move_speed: f32) -> Self {
        Self::new_at(
            WorldVec2::from_data_units(x_units, y_units),
            radius,
            move_speed,
        )
    }

    pub fn projected_tile(&self) -> Position {
        self.position.project_to_tile()
    }

    pub fn distance_to(&self, other: &Self) -> f32 {
        self.position.distance(other.position)
    }

    pub fn can_reach(&self, other: &Self, range_units: f32) -> bool {
        self.distance_to(other) <= range_units + self.radius + other.radius
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
    fn unit_body_reach_includes_both_radii() {
        let a = UnitBody::new_at(WorldVec2::new(0.0, 0.0), 0.3, 1.0);
        let b = UnitBody::new_at(WorldVec2::new(1.5, 0.0), 0.3, 1.0);

        assert!(a.can_reach(&b, 0.9));
        assert!(!a.can_reach(&b, 0.8));
    }

    #[test]
    fn data_unit_conversion_round_trips_through_world_space() {
        let pos = WorldVec2::from_data_units(1_250_000, -500_000);
        assert_eq!(pos, WorldVec2::new(1.25, -0.5));
        assert_eq!(pos.to_data_units(), (1_250_000, -500_000));
    }

    #[test]
    fn timeline_quantization_round_trips_to_world_scale() {
        let point = WorldVec2::new(1.2345, 6.7894).quantized_milli();

        assert_eq!(
            point,
            TimelineVec2 {
                x_milli: 1235,
                y_milli: 6789
            }
        );
        assert_eq!(point.to_world(), WorldVec2::new(1.235, 6.789));
    }
}
