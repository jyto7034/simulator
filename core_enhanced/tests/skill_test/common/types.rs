use game_core::{
    game::data::abnormality_data::{
        AbnormalityMetadata, BasicAttackDef, MovementDef, ResonanceDef,
    },
    game::resources::Position,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PlacedUnitKind {
    SkillDummy,
    Abnormality(&'static str),
    BaseUuid(Uuid),
    Custom(AbnormalityMetadata),
}

#[derive(Debug, Clone, Default)]
pub struct StaticUnitPatch {
    pub max_health: Option<u32>,
    pub attack: Option<u32>,
    pub defense: Option<i32>,
    pub movement: Option<MovementDef>,
    pub basic_attack: Option<BasicAttackDef>,
    pub resonance: Option<ResonanceDef>,
}

impl StaticUnitPatch {
    pub fn is_empty(&self) -> bool {
        self.max_health.is_none()
            && self.attack.is_none()
            && self.defense.is_none()
            && self.movement.is_none()
            && self.basic_attack.is_none()
            && self.resonance.is_none()
    }

    pub(crate) fn merge_from(&mut self, other: &Self) {
        if other.max_health.is_some() {
            self.max_health = other.max_health;
        }
        if other.attack.is_some() {
            self.attack = other.attack;
        }
        if other.defense.is_some() {
            self.defense = other.defense;
        }
        if other.movement.is_some() {
            self.movement = other.movement.clone();
        }
        if other.basic_attack.is_some() {
            self.basic_attack = other.basic_attack.clone();
        }
        if other.resonance.is_some() {
            self.resonance = other.resonance.clone();
        }
    }

    pub(crate) fn apply_to_metadata(&self, abnormality: &mut AbnormalityMetadata) {
        if let Some(max_health) = self.max_health {
            abnormality.max_health = max_health;
        }
        if let Some(attack) = self.attack {
            abnormality.attack = attack;
        }
        if let Some(defense) = self.defense {
            abnormality.defense = defense;
        }
        if let Some(movement) = &self.movement {
            abnormality.movement = movement.clone();
        }
        if let Some(basic_attack) = &self.basic_attack {
            abnormality.basic_attack = basic_attack.clone();
        }
        if let Some(resonance) = &self.resonance {
            abnormality.resonance = resonance.clone();
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeStartPatch {
    pub current_health: Option<u32>,
    pub resonance_current: Option<u32>,
    pub pending_cast: Option<bool>,
    pub current_target_position: Option<Position>,
    pub movement_lock_until_ms: Option<u64>,
    pub basic_attack_lock_until_ms: Option<u64>,
    pub resonance_gain_lock_until_ms: Option<u64>,
}

impl RuntimeStartPatch {
    pub fn is_empty(&self) -> bool {
        self.current_health.is_none()
            && self.resonance_current.is_none()
            && self.pending_cast.is_none()
            && self.current_target_position.is_none()
            && self.movement_lock_until_ms.is_none()
            && self.basic_attack_lock_until_ms.is_none()
            && self.resonance_gain_lock_until_ms.is_none()
    }

    pub(crate) fn merge_from(&mut self, other: &Self) {
        if other.current_health.is_some() {
            self.current_health = other.current_health;
        }
        if other.resonance_current.is_some() {
            self.resonance_current = other.resonance_current;
        }
        if other.pending_cast.is_some() {
            self.pending_cast = other.pending_cast;
        }
        if other.current_target_position.is_some() {
            self.current_target_position = other.current_target_position;
        }
        if other.movement_lock_until_ms.is_some() {
            self.movement_lock_until_ms = other.movement_lock_until_ms;
        }
        if other.basic_attack_lock_until_ms.is_some() {
            self.basic_attack_lock_until_ms = other.basic_attack_lock_until_ms;
        }
        if other.resonance_gain_lock_until_ms.is_some() {
            self.resonance_gain_lock_until_ms = other.resonance_gain_lock_until_ms;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UnitPatch {
    pub static_patch: StaticUnitPatch,
    pub runtime_start: RuntimeStartPatch,
}

impl UnitPatch {
    pub fn is_empty(&self) -> bool {
        self.static_patch.is_empty() && self.runtime_start.is_empty()
    }

    pub(crate) fn merge_from(&mut self, other: &Self) {
        self.static_patch.merge_from(&other.static_patch);
        self.runtime_start.merge_from(&other.runtime_start);
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestUnitOverrides {
    pub max_health: u32,
    pub attack: u32,
    pub defense: i32,
    pub movement: MovementDef,
    pub basic_attack: BasicAttackDef,
    pub resonance: ResonanceDef,
    pub current_health_after_start: Option<u32>,
}

#[allow(dead_code)]
impl TestUnitOverrides {
    pub fn from_abnormality(abnormality: AbnormalityMetadata) -> Self {
        Self {
            max_health: abnormality.max_health,
            attack: abnormality.attack,
            defense: abnormality.defense,
            movement: abnormality.movement,
            basic_attack: abnormality.basic_attack,
            resonance: abnormality.resonance,
            current_health_after_start: None,
        }
    }

    pub fn skill_dummy_defaults() -> Self {
        Self::from_abnormality(crate::skill_test::common::scenario::skill_test_dummy_metadata())
    }

    pub fn with_max_health(mut self, max_health: u32) -> Self {
        self.max_health = max_health;
        self
    }

    pub fn with_current_health_after_start(mut self, current_health: u32) -> Self {
        self.current_health_after_start = Some(current_health);
        self
    }

    pub fn with_attack(mut self, attack: u32) -> Self {
        self.attack = attack;
        self
    }

    pub fn with_defense(mut self, defense: i32) -> Self {
        self.defense = defense;
        self
    }

    pub fn with_movement(mut self, movement: MovementDef) -> Self {
        self.movement = movement;
        self
    }

    pub fn with_basic_attack(mut self, basic_attack: BasicAttackDef) -> Self {
        self.basic_attack = basic_attack;
        self
    }

    pub fn with_resonance(mut self, resonance: ResonanceDef) -> Self {
        self.resonance = resonance;
        self
    }
}

impl From<TestUnitOverrides> for UnitPatch {
    fn from(value: TestUnitOverrides) -> Self {
        Self {
            static_patch: StaticUnitPatch {
                max_health: Some(value.max_health),
                attack: Some(value.attack),
                defense: Some(value.defense),
                movement: Some(value.movement),
                basic_attack: Some(value.basic_attack),
                resonance: Some(value.resonance),
            },
            runtime_start: RuntimeStartPatch {
                current_health: value.current_health_after_start,
                ..Default::default()
            },
        }
    }
}

pub fn passive_dummy_patch(max_health: u32, current_health_after_start: Option<u32>) -> UnitPatch {
    let mut overrides = TestUnitOverrides::skill_dummy_defaults()
        .with_max_health(max_health)
        .with_attack(1)
        .with_movement(MovementDef {
            speed_units_per_ms: 0,
            radius_units: 350_000,
        })
        .with_basic_attack(BasicAttackDef {
            range_units: 1.0,
            defense_tile_range: None,
            interval_ms: 5_000,
            windup_ms: 0,
            delivery: game_core::game::ability::DeliveryDef::Instant,
            ..BasicAttackDef::default()
        });

    if let Some(current_health) = current_health_after_start {
        overrides = overrides.with_current_health_after_start(current_health);
    }

    overrides.into()
}
