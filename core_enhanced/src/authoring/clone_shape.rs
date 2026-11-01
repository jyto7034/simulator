//! Borrowed ownership visitors for controlled native clones. This module never
//! clones a source to measure it, resolves definitions, or retains source owners.
//! Native tables must use their storage proof; serde is restricted to explicitly
//! reviewed, entirely owned leaves with no hidden/Arc/table children.

use std::{
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashSet, VecDeque},
    fmt,
    sync::Arc,
};

use serde::{ser, Serialize};

use super::control::CloneMeter;
use crate::{game::behavior::GameError, owned_collections::OwnedHashMap};

pub(crate) trait CloneShape {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError>;
    fn preflight_native_clone(&self) -> Result<(), GameError> {
        Ok(())
    }
}

/// Price this owned copy before executing its unchanged native Clone. The
/// attempt budget remains outside the value, and a failed post-copy poll drops
/// the local candidate. Its caller still owns the enclosing transaction.
pub(crate) fn controlled_clone<T: Clone + CloneShape>(value: &T) -> Result<T, GameError> {
    if crate::game::battle::runtime_control::is_active() {
        value.preflight_native_clone()?;
    }
    if !super::control::is_active() {
        return Ok(value.clone());
    }
    super::control::poll("owned_clone_entry")?;
    let mut meter = CloneMeter::default();
    value.visit_clone_shape(&mut meter)?;
    meter.finish("owned_clone_storage", "owned_clone_copy")?;
    let candidate = value.clone();
    super::control::poll("owned_clone_complete")?;
    Ok(candidate)
}

macro_rules! scalar_shape {
    ($($ty:ty),* $(,)?) => {$ (
        impl CloneShape for $ty where $ty: Copy {
            fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
                meter.scan_storage(Some(1))
            }
        }
    )*};
}
scalar_shape!(
    (),
    bool,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    uuid::Uuid
);

impl CloneShape for String {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
        meter.scan_string(self)
    }
}

impl<T: CloneShape> CloneShape for Vec<T> {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
        // Native Vec::clone copies live elements, not the source's spare capacity.
        meter.scan_storage(self.len().checked_add(1))?;
        for value in self {
            value.visit_clone_shape(meter)?;
        }
        Ok(())
    }
}

impl<T: CloneShape> CloneShape for BinaryHeap<T> {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
        // Rust 1.91.1 BinaryHeap::clone delegates to its Vec::clone, preserving
        // native heap order without comparing/reinserting values or spare capacity.
        meter.scan_storage(self.len().checked_add(1))?;
        for value in self.iter() {
            value.visit_clone_shape(meter)?;
        }
        Ok(())
    }
}

impl<T: CloneShape> CloneShape for VecDeque<T> {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
        // Rust 1.91.1 Clone creates with_capacity(len), then extends its exact
        // front-to-back iterator. This is not clone_from's retained allocation.
        meter.scan_storage(self.len().checked_add(1))?;
        for value in self {
            value.visit_clone_shape(meter)?;
        }
        Ok(())
    }
}

impl<T: CloneShape> CloneShape for Option<T> {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
        meter.scan_storage(Some(1))?;
        if let Some(value) = self {
            value.visit_clone_shape(meter)?;
        }
        Ok(())
    }
}

impl<A: CloneShape, B: CloneShape> CloneShape for (A, B) {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
        meter.scan_storage(Some(1))?;
        self.0.visit_clone_shape(meter)?;
        self.1.visit_clone_shape(meter)
    }
}

impl<A: CloneShape, B: CloneShape, C: CloneShape> CloneShape for (A, B, C) {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
        meter.scan_storage(Some(1))?;
        self.0.visit_clone_shape(meter)?;
        self.1.visit_clone_shape(meter)?;
        self.2.visit_clone_shape(meter)
    }
}

/// Caller must establish the actual fresh/insert-only/native-clone ingress.
/// There is intentionally no blanket CloneShape implementation for HashSet:
/// serde sequence length and arbitrary deleted tables are not bucket proofs.
pub(crate) fn visit_tombstone_free_set<T: CloneShape, S>(
    set: &HashSet<T, S>,
    meter: &mut CloneMeter,
) -> Result<(), GameError> {
    let slots =
        crate::owned_collections::MapCloneHistory::from_tombstone_free_capacity(set.capacity())
            .bucket_bound()
            .and_then(|n| n.checked_add(1));
    meter.scan_storage(slots)?;
    for value in set {
        value.visit_clone_shape(meter)?;
    }
    Ok(())
}

impl<T: ?Sized> CloneShape for Arc<T> {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
        // Arc clone only retains its handle. No pointee CloneShape is required.
        meter.scan_storage(Some(1))
    }
}

impl<T: CloneShape> CloneShape for Box<T> {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
        meter.scan_storage(Some(1))?;
        (**self).visit_clone_shape(meter)
    }
}

impl<K: CloneShape, V: CloneShape, S> CloneShape for OwnedHashMap<K, V, S> {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
        meter.scan_storage(self.clone_bucket_bound().and_then(|n| n.checked_add(1)))?;
        for (key, value) in self.iter() {
            key.visit_clone_shape(meter)?;
            value.visit_clone_shape(meter)?;
        }
        Ok(())
    }
}

fn btree_clone_slots(len: usize) -> Option<usize> {
    // Rust 1.91.1 BTreeMap: each non-root node has >=5 keys; a nonempty
    // root has >=1. Native Clone preserves shape, but clones an empty source
    // into root=None even if the source retains its final empty leaf.
    // Each node owns <=37 typed header/key/value/edge slots; use 40 plus one
    // outer collection record. This is not an allocator/RSS byte measurement.
    let nodes = if len == 0 { 0 } else { 1 + (len - 1) / 5 };
    nodes.checked_mul(40)?.checked_add(1)
}

impl<K: CloneShape, V: CloneShape> CloneShape for BTreeMap<K, V> {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
        meter.scan_storage(btree_clone_slots(self.len()))?;
        for (key, value) in self {
            key.visit_clone_shape(meter)?;
            value.visit_clone_shape(meter)?;
        }
        Ok(())
    }
}

impl<T: CloneShape> CloneShape for BTreeSet<T> {
    fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
        meter.scan_storage(btree_clone_slots(self.len()))?;
        for key in self {
            key.visit_clone_shape(meter)?;
        }
        Ok(())
    }
}

/// Explicit field patterns make newly added direct owners a compile error.
/// Every listed child must have its own native/closed-leaf ownership proof.
macro_rules! struct_shape {
    ($ty:ty { $($(#[$attr:meta])* $field:ident),* $(,)? }) => {
        impl $crate::authoring::clone_shape::CloneShape for $ty {
            fn visit_clone_shape(
                &self, meter: &mut $crate::authoring::control::CloneMeter,
            ) -> Result<(), $crate::game::behavior::GameError> {
                let Self { $($(#[$attr])* $field),* } = self;
                meter.scan_storage(Some(1))?;
                $( $(#[$attr])* $crate::authoring::clone_shape::CloneShape::visit_clone_shape($field, meter)?; )*
                Ok(())
            }
        }
    };
}
pub(crate) use struct_shape;

/// The same borrowed field list drives projected admission and native field
/// clones. Complete construction exposes newly added fields to compilation.
/// Fresh declarations are restricted to fixed None/default-empty fields with
/// zero heap allocation. Their own resulting storage is visited, then moved;
/// the omitted source owners are never inspected. No dynamic factory is allowed.
macro_rules! capture_shape {
    ($ty:ident { $($field:ident: $source:expr),* $(,)? }
        sets { $($set:ident: $set_source:expr),* $(,)? }
        fresh { $($fresh:ident: $fresh_value:expr),* $(,)? }
        fresh_sets { $($fresh_set:ident: $fresh_set_value:expr),* $(,)? }) => {{
        $(let $field = $source;)*
        $(let $set = $set_source;)*
        let controlled = $crate::authoring::control::is_active();
        if controlled {
            $crate::authoring::control::poll("owned_capture_entry")?;
        }
        $(let $fresh = $fresh_value;)*
        $(let $fresh_set = $fresh_set_value;)*
        if controlled {
            let mut meter = $crate::authoring::control::CloneMeter::default();
            meter.scan_storage(Some(1))?;
            $( $crate::authoring::clone_shape::CloneShape::visit_clone_shape($field, &mut meter)?; )*
            $( $crate::authoring::clone_shape::visit_tombstone_free_set($set, &mut meter)?; )*
            $( $crate::authoring::clone_shape::CloneShape::visit_clone_shape(&$fresh, &mut meter)?; )*
            $( $crate::authoring::clone_shape::visit_tombstone_free_set(&$fresh_set, &mut meter)?; )*
            meter.finish("owned_capture_storage", "owned_capture_copy")?;
        }
        let candidate = $ty {
            $($field: Clone::clone($field),)*
            $($set: Clone::clone($set),)*
            $($fresh,)*
            $($fresh_set,)*
        };
        if controlled { $crate::authoring::control::poll("owned_capture_complete")?; }
        Ok::<_, $crate::game::behavior::GameError>(candidate)
    }};
}
pub(crate) use capture_shape;

macro_rules! struct_shape_with_sets {
    ($ty:ty { $($field:ident),* $(,)? } sets { $($set:ident),+ $(,)? }) => {
        impl $crate::authoring::clone_shape::CloneShape for $ty {
            fn visit_clone_shape(&self, meter: &mut $crate::authoring::control::CloneMeter)
                -> Result<(), $crate::game::behavior::GameError> {
                let Self { $($field,)* $($set,)+ } = self;
                meter.scan_storage(Some(1))?;
                $( $crate::authoring::clone_shape::CloneShape::visit_clone_shape($field, meter)?; )*
                $( $crate::authoring::clone_shape::visit_tombstone_free_set($set, meter)?; )+
                Ok(())
            }
        }
    };
}
pub(crate) use struct_shape_with_sets;

macro_rules! enum_shape {
    ($ty:ty { units: [$($unit:ident),* $(,)?],
        tuples: [$($tuple:ident($($item:ident),*)),* $(,)?],
        records: [$($record:ident { $($field:ident),* $(,)? }),* $(,)?] }) => {
        impl $crate::authoring::clone_shape::CloneShape for $ty {
            fn visit_clone_shape(&self, meter: &mut $crate::authoring::control::CloneMeter)
                -> Result<(), $crate::game::behavior::GameError> {
                meter.scan_storage(Some(1))?;
                match self {
                    $(Self::$unit => {},)*
                    $(Self::$tuple($($item),*) => {
                        $($crate::authoring::clone_shape::CloneShape::visit_clone_shape($item, meter)?;)*
                    },)*
                    $(Self::$record { $($field),* } => {
                        $($crate::authoring::clone_shape::CloneShape::visit_clone_shape($field, meter)?;)*
                    },)*
                }
                Ok(())
            }
        }
    };
}
pub(crate) use enum_shape;

macro_rules! newtype_shape {
    ($($ty:ty),* $(,)?) => {$ (
        impl $crate::authoring::clone_shape::CloneShape for $ty {
            fn visit_clone_shape(&self, meter: &mut $crate::authoring::control::CloneMeter)
                -> Result<(), $crate::game::behavior::GameError> {
                let Self(value) = self;
                meter.scan_storage(Some(1))?;
                $crate::authoring::clone_shape::CloneShape::visit_clone_shape(value, meter)
            }
        }
    )*};
}
pub(crate) use newtype_shape;

macro_rules! copy_shape {
    ($($ty:ty),* $(,)?) => {$ (
        impl $crate::authoring::clone_shape::CloneShape for $ty where $ty: Copy {
            fn visit_clone_shape(&self, meter: &mut $crate::authoring::control::CloneMeter)
                -> Result<(), $crate::game::behavior::GameError> { meter.scan_storage(Some(1)) }
        }
    )*};
}
pub(crate) use copy_shape;

copy_shape!(
    crate::game::battle::ids::UnitInstanceId,
    crate::game::enums::Side,
    crate::game::resources::Position,
    crate::game::stats::UnitStats,
    crate::game::stats::StatModifier,
    crate::game::battle::types::BattleUnitRole,
    crate::game::battle::types::BattleUnitThreatClass,
    crate::game::battle::types::MobilityKind,
    crate::game::battle::types::EngagementStance,
    crate::game::battle::types::TacticalTileKind,
    crate::game::battle::types::UnitExclusiveActivity,
    crate::game::battle::types::UnitTargetTrait,
    crate::game::battle::damage::DamageModifiers,
    crate::game::battle::damage::DamageType,
    crate::game::battle::damage::DamageSource,
    crate::game::battle::damage::CombatRollIdentity,
    crate::game::battle::damage::DamageBonusSnapshot,
    crate::game::battle::event_log::BattleEventCause,
    crate::game::battle::event_log::SkillCastTarget,
    crate::game::battle::event_log::AttackKind,
    crate::game::battle::event_log::AttackDelivery,
    crate::game::battle::buffs::BuffId,
    crate::game::battle::cooldown::CooldownSource,
    crate::game::battle::core::movement::types::WorldVec2,
    crate::game::battle::core::movement::types::UnitBody,
    crate::game::battle::tile_range::FacingDirection,
    crate::game::ability::SkillActivationMode,
    crate::game::ability::SkillAreaAnchorSource,
    crate::game::ability::SkillTileAreaOrigin,
    crate::game::ability::SkillAreaTracking,
    crate::game::ability::SkillHitTargetFilter,
    crate::game::ability::SkillAreaTickPolicy,
    crate::game::map::MapNodeId,
    crate::game::map::HeadquartersContactOption,
    crate::game::map::GameMode,
    crate::game::enums::RewardMode,
    crate::game::combat_preview::CombatNodeType,
    crate::game::combat_preview::CombatMissionVariant,
    crate::game::data::shop_data::ShopContext,
    crate::game::data::shop_data::ShopRefreshPolicy,
    crate::game::skill_fragment::SkillFragmentPolicy,
    crate::game::behavior::BattlePlaybackState,
    crate::game::battle::core::sim::BattleDeployCurrentHpPolicy,
    crate::game::combat_preview::CombatMissionRisk,
    crate::game::combat_preview::BattlefieldArchetype,
    crate::game::combat_preview::BattlefieldSizeClass,
);

// These closed graphs contain Strings/Vec and fixed records only. Do not add a
// whole Inventory/Employee here: Arc metadata and GrowthStack require native
// owner visitors. No blanket T: Serialize implementation is permitted.
macro_rules! serialized_leaf {
    ($($ty:ty),* $(,)?) => {$ (
        impl CloneShape for $ty {
            fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
                self.serialize(LeafSerializer(meter)).map_err(|error| error.0)
            }
        }
    )*};
}
serialized_leaf!(
    crate::game::employee::EmployeeProfile,
    crate::game::employee::EmployeeDefinition,
    crate::game::employee::EmployeeCondition,
    crate::game::employee::ActiveConsumableModifier,
    crate::game::resources::item_slot::ItemSlot,
    crate::game::skill_fragment::SkillFragmentLoadout,
    crate::game::battle::types::UnitCombatProfile,
    crate::game::data::equipment_data::EquipmentType,
    crate::game::data::skill_fragment_data::SkillFragmentId,
    crate::game::ability::SkillId,
    crate::game::ability::sustained::BasicAttackOverlayProfile,
    crate::game::ability::SkillProjectileCollisionDef,
    crate::game::data::abnormality_data::BasicAttackDef,
    crate::game::data::equipment_data::WeaponCombatProfile,
    crate::game::battle::tile_range::TileRangePattern,
    crate::game::battle::types::BattleUnitSourceIdentity,
    // Reviewed world leaves own String/Vec/fixed data only. Native owner maps,
    // checkpoint state, source Arcs and serde-skipped logs stay outside this list.
    crate::game::resources::GameState,
    crate::game::resources::EventSessionState,
    crate::game::resources::ShopOfferState,
    crate::game::data::shop_data::ShopMerchandise,
    crate::game::data::shop_data::ShopSellPolicy,
    crate::game::resources::Enkephalin,
    crate::game::reward::RewardOption,
    crate::game::reward::RewardEffect,
    crate::game::battle::types::ParticipantBattleResult,
    crate::game::battle::result_stats::BattleResultStatsDto,
    crate::game::map::NodeSession,
    crate::game::visual_novel::VisualNovelSession,
    crate::game::visual_novel::VisualNovelTriggerConsumedKey,
    crate::game::behavior::MasteredEliteSweepResultDto,
    crate::game::behavior::LiveBattleTacticalFocusDto,
    crate::game::map::RunMap,
    crate::game::map::RewardActivityKind,
    crate::game::map::MapViewDto,
    crate::game::map::MapProgression,
    crate::game::map::RunProgressionModeState,
    crate::game::map::MapTemplateId,
    crate::game::map::MapSlotId,
    crate::game::boss_omen::ProvisionalBossOmenState,
    crate::game::boss_omen::ActiveBossOmenState,
    crate::game::boss_omen::PlacedBossOmenSourceState,
    crate::game::boss_omen::ForcedBossOmenNodeState,
    crate::game::data::boss_omen_data::BossOmenChainId,
    crate::game::data::content_pack::ContentIdentity,
    crate::game::recruitment::RecruitmentSessionState,
    crate::game::combat_preview::BattlefieldTile,
    crate::game::combat_preview::DeploymentZone,
    crate::game::combat_preview::SpawnZone,
    crate::game::combat_preview::BattlefieldRoute,
    crate::game::combat_preview::SpawnWave,
    crate::game::combat_preview::EnemyBriefing,
    crate::game::combat_preview::ThreatWarning,
);

#[derive(Debug)]
struct ShapeError(GameError);
impl fmt::Display for ShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl std::error::Error for ShapeError {}
impl ser::Error for ShapeError {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self(GameError::InvalidStaticData(message.to_string()))
    }
}

struct LeafSerializer<'a>(&'a mut CloneMeter);
impl LeafSerializer<'_> {
    fn scalar(&mut self) -> Result<(), ShapeError> {
        self.0.scan_storage(Some(1)).map_err(ShapeError)
    }

    fn container(&mut self, len: Option<usize>) -> Result<(), ShapeError> {
        self.0
            .scan_storage(len.and_then(|n| n.checked_add(1)))
            .map_err(ShapeError)
    }
}

macro_rules! scalar_serializers {
    ($($method:ident($ty:ty)),* $(,)?) => {$ (
        fn $method(mut self, _value: $ty) -> Result<(), ShapeError> { self.scalar() }
    )*};
}

impl<'a> ser::Serializer for LeafSerializer<'a> {
    type Ok = ();
    type Error = ShapeError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = ser::Impossible<(), ShapeError>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    scalar_serializers!(
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),
        serialize_f32(f32),
        serialize_f64(f64),
        serialize_char(char)
    );

    fn serialize_str(self, value: &str) -> Result<(), ShapeError> {
        self.0.scan_string(value).map_err(ShapeError)
    }
    fn serialize_bytes(mut self, value: &[u8]) -> Result<(), ShapeError> {
        self.container(Some(value.len()))
    }
    fn serialize_none(mut self) -> Result<(), ShapeError> {
        self.scalar()
    }
    fn serialize_some<T: ?Sized + Serialize>(mut self, value: &T) -> Result<(), ShapeError> {
        self.scalar()?;
        value.serialize(self)
    }
    fn serialize_unit(mut self) -> Result<(), ShapeError> {
        self.scalar()
    }
    fn serialize_unit_struct(mut self, _name: &'static str) -> Result<(), ShapeError> {
        self.scalar()
    }
    fn serialize_unit_variant(
        mut self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
    ) -> Result<(), ShapeError> {
        self.scalar()
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        mut self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), ShapeError> {
        self.scalar()?;
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        mut self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<(), ShapeError> {
        self.scalar()?;
        value.serialize(self)
    }
    fn serialize_seq(mut self, len: Option<usize>) -> Result<Self, ShapeError> {
        self.container(len)?;
        Ok(self)
    }
    fn serialize_tuple(mut self, len: usize) -> Result<Self, ShapeError> {
        self.container(Some(len))?;
        Ok(self)
    }
    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self, ShapeError> {
        self.serialize_tuple(len)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self, ShapeError> {
        self.serialize_tuple(len)
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, ShapeError> {
        Err(ShapeError(GameError::InvalidStaticData(
            "clone leaf contains an unreviewed native map".to_string(),
        )))
    }
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self, ShapeError> {
        self.serialize_tuple(len)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self, ShapeError> {
        self.serialize_tuple(len)
    }

    fn collect_str<T: ?Sized + fmt::Display>(self, _value: &T) -> Result<(), ShapeError> {
        // Avoid serde's default allocating to_string(). No currently reviewed
        // leaf needs this; a new custom formatting path requires an owner proof.
        Err(ShapeError(GameError::InvalidStaticData(
            "clone leaf contains unreviewed display serialization".to_string(),
        )))
    }
}

macro_rules! sequence_serializer {
    ($trait:ident, $method:ident) => {
        impl ser::$trait for LeafSerializer<'_> {
            type Ok = ();
            type Error = ShapeError;
            fn $method<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), ShapeError> {
                value.serialize(LeafSerializer(self.0))
            }
            fn end(self) -> Result<(), ShapeError> {
                Ok(())
            }
        }
    };
}
sequence_serializer!(SerializeSeq, serialize_element);
sequence_serializer!(SerializeTuple, serialize_element);
sequence_serializer!(SerializeTupleStruct, serialize_field);
sequence_serializer!(SerializeTupleVariant, serialize_field);

macro_rules! struct_serializer {
    ($trait:ident) => {
        impl ser::$trait for LeafSerializer<'_> {
            type Ok = ();
            type Error = ShapeError;
            fn serialize_field<T: ?Sized + Serialize>(
                &mut self,
                _key: &'static str,
                value: &T,
            ) -> Result<(), ShapeError> {
                value.serialize(LeafSerializer(self.0))
            }
            fn end(self) -> Result<(), ShapeError> {
                Ok(())
            }
        }
    };
}
struct_serializer!(SerializeStruct);
struct_serializer!(SerializeStructVariant);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
    use std::cell::Cell;

    fn measured(value: &impl CloneShape) -> control::ExecutionUsage {
        control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                let mut meter = CloneMeter::default();
                value.visit_clone_shape(&mut meter).unwrap();
                meter.finish("test_storage", "test_copy").unwrap();
            },
        )
        .unwrap()
        .1
    }

    #[test]
    fn owned_clone_shape_admits_nested_copy_before_clone_and_ignores_vec_spare_capacity() {
        let value = vec!["first".to_owned(), "longer second".to_owned()];
        let usage = measured(&value);
        let mut reserved = Vec::with_capacity(16_384);
        reserved.extend(value.iter().cloned());
        assert_eq!(measured(&reserved), usage);
        let mut copy = None;
        let limit = ExecutionLimits {
            max_work: usage.work - 1,
            ..Default::default()
        };
        let error = control::run(
            &limit,
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                let mut meter = CloneMeter::default();
                value.visit_clone_shape(&mut meter)?;
                meter.finish("test_storage", "test_copy")?;
                copy = Some(value.clone());
                Ok::<_, GameError>(())
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "authoring_work_limit");
        assert_eq!(error.site, "test_copy");
        assert!(copy.is_none());
        let limit = ExecutionLimits {
            max_work: usage.work,
            max_clone_slots: usage.clone_slots_peak,
            ..Default::default()
        };
        let (result, exact) = control::run(
            &limit,
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                let mut meter = CloneMeter::default();
                value.visit_clone_shape(&mut meter)?;
                meter.finish("test_storage", "test_copy")?;
                Ok::<_, GameError>(value.clone())
            },
        )
        .unwrap();
        assert_eq!(result.unwrap(), value);
        assert_eq!(exact, usage);
    }

    #[test]
    fn owned_clone_shape_container_admission_precedes_child_serialization() {
        struct Child<'a>(&'a Cell<usize>);
        impl Serialize for Child<'_> {
            fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.set(self.0.get() + 1);
                serializer.serialize_u64(1)
            }
        }
        struct Sequence<'a> {
            len: Option<usize>,
            visits: &'a Cell<usize>,
        }
        impl Serialize for Sequence<'_> {
            fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut seq = serializer.serialize_seq(self.len)?;
                ser::SerializeSeq::serialize_element(&mut seq, &Child(self.visits))?;
                ser::SerializeSeq::end(seq)
            }
        }
        for len in [Some(1024), None] {
            let visits = Cell::new(0);
            let failure = control::run(
                &ExecutionLimits {
                    max_clone_slots: 32,
                    ..Default::default()
                },
                &CancellationToken::default(),
                ExecutionPhase::TrialPreparation,
                || {
                    let mut meter = CloneMeter::default();
                    assert!(Sequence {
                        len,
                        visits: &visits
                    }
                    .serialize(LeafSerializer(&mut meter))
                    .is_err());
                },
            )
            .unwrap_err();
            assert_eq!(failure.code, "authoring_clone_limit");
            assert_eq!(visits.get(), 0);
        }
        // A serde map has no native-table proof and cannot enter the leaf path.
        let map = BTreeMap::from([("key", 1)]);
        let mut meter = CloneMeter::default();
        assert!(map.serialize(LeafSerializer(&mut meter)).is_err());
    }

    #[test]
    fn owned_clone_shape_arc_has_no_pointee_requirement_and_btree_copy_is_bounded() {
        struct NoCloneOrSerialize {
            _owned: Vec<String>,
        }
        let shared = Arc::new(NoCloneOrSerialize {
            _owned: vec!["large".repeat(1024)],
        });
        assert_eq!(measured(&shared), measured(&Arc::new(())));
        let mut tree = BTreeMap::new();
        for i in 0..256 {
            tree.insert(i as u64, format!("value_{i}"));
        }
        for i in 0..200 {
            tree.remove(&(i as u64));
        }
        let mut candidate = None;
        let usage = measured(&tree);
        let failure = control::run(
            &ExecutionLimits {
                max_clone_slots: usage.clone_slots_peak - 1,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                let mut meter = CloneMeter::default();
                tree.visit_clone_shape(&mut meter)?;
                meter.finish("test_storage", "test_copy")?;
                candidate = Some(tree.clone());
                Ok::<_, GameError>(())
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_clone_limit");
        assert!(candidate.is_none());
        assert_eq!(measured(&tree.clone()), usage);
        tree.clear();
        assert_eq!(measured(&tree), measured(&BTreeMap::<u64, String>::new()));
    }

    #[test]
    fn owned_clone_shape_employee_counts_owned_profiles_and_keeps_uuid_continuation() {
        use crate::game::{
            employee::{Employee, EmployeeRoster},
            growth::GrowthId,
            managers::uuid_manager::UuidManager,
            skill_fragment::SkillFragmentLoadout,
        };
        let mut uuids = UuidManager::new(73);
        for namespace in 0..32 {
            uuids.next(namespace);
        }
        let mut employee = Employee::new(uuids.next_employee(), "employee_a", "Name");
        employee.skill_fragments =
            SkillFragmentLoadout::with_baseline_ids(vec!["fragment_a".into(), "fragment_b".into()]);
        employee
            .combat_profile
            .growth_stacks
            .add(GrowthId::KillStack, 3);
        let initial = measured(&employee);
        employee.name.push_str(" extended");
        let changed = measured(&employee);
        assert_eq!(changed.clone_slots_peak, initial.clone_slots_peak);
        assert_eq!(changed.work, initial.work + " extended".len() as u64);
        employee
            .combat_profile
            .battle_profile
            .basic_attack
            .defense_tile_range
            .as_mut()
            .unwrap()
            .rows
            .push("...".to_string());
        assert!(measured(&employee).clone_slots_peak > changed.clone_slots_peak);
        let mut roster = EmployeeRoster::new();
        roster.add(employee);
        let roster_json = serde_json::to_value(&roster).unwrap();
        let token = CancellationToken::default();
        token.cancel();
        let mut copied = false;
        let failure = control::run(
            &ExecutionLimits::default(),
            &token,
            ExecutionPhase::TrialPreparation,
            || {
                let mut meter = CloneMeter::default();
                roster.visit_clone_shape(&mut meter)?;
                let _candidate = roster.clone();
                copied = true;
                Ok::<_, GameError>(())
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_cancelled");
        assert!(!copied);
        assert_eq!(serde_json::to_value(&roster).unwrap(), roster_json);
        let (_, usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                let mut meter = CloneMeter::default();
                roster.visit_clone_shape(&mut meter).unwrap();
                uuids.visit_clone_shape(&mut meter).unwrap();
                meter.finish("roster_storage", "roster_copy").unwrap();
                let cloned = roster.clone();
                assert_eq!(serde_json::to_value(&cloned).unwrap(), roster_json);
                let mut next = uuids.clone();
                assert_eq!(next.next_employee(), uuids.peek(UuidManager::NS_EMPLOYEE));
            },
        )
        .unwrap();
        assert!(usage.clone_slots_peak > measured(&roster).clone_slots_peak);
    }

    #[test]
    fn owned_clone_shape_controlled_copy_rejects_before_clone_and_drops_after_cancel() {
        use std::rc::Rc;
        struct Probe {
            text: String,
            copies: Rc<Cell<usize>>,
            drops: Rc<Cell<usize>>,
            cancel: Option<CancellationToken>,
            is_copy: bool,
        }
        impl CloneShape for Probe {
            fn visit_clone_shape(&self, meter: &mut CloneMeter) -> Result<(), GameError> {
                self.text.visit_clone_shape(meter)
            }
        }
        impl Clone for Probe {
            fn clone(&self) -> Self {
                self.copies.set(self.copies.get() + 1);
                if let Some(token) = &self.cancel {
                    token.cancel();
                }
                Self {
                    text: self.text.clone(),
                    copies: self.copies.clone(),
                    drops: self.drops.clone(),
                    cancel: self.cancel.clone(),
                    is_copy: true,
                }
            }
        }
        impl Drop for Probe {
            fn drop(&mut self) {
                if self.is_copy {
                    self.drops.set(self.drops.get() + 1);
                }
            }
        }
        let mut source = Probe {
            text: "owned candidate".repeat(10),
            copies: Rc::new(Cell::new(0)),
            drops: Rc::new(Cell::new(0)),
            cancel: None,
            is_copy: false,
        };
        let usage = measured(&source);
        let mut limits = ExecutionLimits::default();
        limits.max_work = usage.work - 1;
        let failure = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || controlled_clone(&source),
        )
        .err()
        .unwrap();
        assert_eq!(failure.code, "authoring_work_limit");
        assert_eq!(source.copies.get(), 0);
        limits.max_work = usage.work;
        let (copy, actual) = control::run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || controlled_clone(&source),
        )
        .unwrap();
        assert_eq!(copy.as_ref().unwrap().text, source.text);
        assert_eq!(usage, actual);
        drop(copy);
        assert_eq!(source.drops.get(), 1);
        let token = CancellationToken::default();
        source.cancel = Some(token.clone());
        let failure = control::run(&limits, &token, ExecutionPhase::TrialPreparation, || {
            controlled_clone(&source)
        })
        .err()
        .unwrap();
        assert_eq!(failure.code, "authoring_cancelled");
        assert_eq!(failure.site, "owned_clone_complete");
        assert_eq!(source.copies.get(), 2);
        assert_eq!(source.drops.get(), 2);
        assert_eq!(source.text, "owned candidate".repeat(10));
        assert!(!control::is_active());
    }

    #[test]
    fn owned_clone_shape_vecdeque_prices_live_order_without_spare_capacity_or_wrap() {
        let mut wrapped = VecDeque::with_capacity(8);
        wrapped.extend((0..8).map(|i| format!("item {i}")));
        for _ in 0..6 {
            wrapped.pop_front();
        }
        wrapped.extend((8..12).map(|i| format!("item {i}")));
        assert!(!wrapped.as_slices().1.is_empty());
        let mut spare = VecDeque::with_capacity(4096);
        spare.extend(wrapped.iter().cloned());
        assert_eq!(measured(&wrapped), measured(&spare));
        let (copy, _) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || controlled_clone(&wrapped),
        )
        .unwrap();
        let copy = copy.unwrap();
        assert_eq!(copy, wrapped);
        assert_eq!(
            copy.iter().cloned().collect::<Vec<_>>(),
            (6..12).map(|i| format!("item {i}")).collect::<Vec<_>>()
        );
        spare.clear();
        assert_eq!(measured(&spare), measured(&VecDeque::<String>::new()));
    }

    #[test]
    fn owned_clone_shape_monolith_requires_provenance_and_preserves_native_history() {
        use crate::game::{map::MapNodeId, monolith::MonolithRunState};
        use control::NativeStorageProvenance;
        let limits = ExecutionLimits::default();
        let token = CancellationToken::default();
        let phase = ExecutionPhase::TrialPreparation;
        // General scopes do not certify arbitrary public roots, even empty ones.
        let unknown = control::run(&limits, &token, phase, || {
            controlled_clone(&MonolithRunState::default())
        })
        .err()
        .unwrap();
        assert_eq!(unknown.code, "authoring_clone_limit");
        let (source, _, history) = control::run_with_provenance(
            &limits,
            &token,
            phase,
            NativeStorageProvenance::fresh(),
            || {
                let mut source = MonolithRunState::default();
                for i in 0..64 {
                    let id = format!("monolith_{i}");
                    let node = MapNodeId(uuid::Uuid::from_u128(i + 1));
                    source.record_natural_victory(&id).unwrap();
                    source.purchase(&id, 2).unwrap();
                    source
                        .begin_remanifestation(node, &id, "encounter")
                        .unwrap();
                    if i % 2 == 0 {
                        source.resolve_remanifestation_failure(node).unwrap();
                    }
                }
                let bound = control::monolith_clone_bucket_bound().unwrap();
                source.clear_floor_local_remanifestations();
                assert!(source.remanifestation_nodes.is_empty());
                assert_eq!(control::monolith_clone_bucket_bound(), Some(bound));
                source
            },
        )
        .unwrap();
        let (copy, usage, _) =
            control::run_with_provenance(&limits, &token, phase, history, || {
                controlled_clone(&source)
            })
            .unwrap();
        let copy = copy.unwrap();
        assert_eq!(copy, source);
        assert_eq!(
            copy.remanifestation_nodes.capacity(),
            source.remanifestation_nodes.capacity()
        );
        let (_, pair_usage, _) =
            control::run_with_provenance(&limits, &token, phase, history, || {
                let mut meter = CloneMeter::default();
                source.visit_clone_shape(&mut meter).unwrap();
                copy.visit_clone_shape(&mut meter).unwrap();
                meter.finish("pair_storage", "pair_copy").unwrap();
            })
            .unwrap();
        assert_eq!(pair_usage.clone_slots_peak, usage.clone_slots_peak * 2);
        assert_eq!(pair_usage.work, usage.work * 2);
        let mut short = limits.clone();
        short.max_clone_slots = usage.clone_slots_peak - 1;
        let failure = control::run_with_provenance(&short, &token, phase, history, || {
            controlled_clone(&source)
        })
        .err()
        .unwrap();
        assert_eq!(failure.code, "authoring_clone_limit");
        // T=domain Err never implies a commit: discard this candidate ledger.
        let (rejected, _, _) =
            control::run_with_provenance(&limits, &token, phase, history, || {
                control::monolith_imported_fresh(8192);
                Err::<(), _>(GameError::InvalidAction)
            })
            .unwrap();
        assert!(rejected.is_err());
        let (_, retry_usage, _) =
            control::run_with_provenance(&limits, &token, phase, history, || {
                controlled_clone(&source)
            })
            .unwrap();
        assert_eq!(usage, retry_usage);
        // A newly imported table cannot shrink older credit, or turn unknown
        // provenance into an invented zero. Overflow must fail before copying.
        let failure = control::run_with_provenance(&limits, &token, phase, history, || {
            control::monolith_imported_fresh(usize::MAX);
            controlled_clone(&source)
        })
        .err()
        .unwrap();
        assert_eq!(failure.code, "authoring_clone_limit");
    }
}
