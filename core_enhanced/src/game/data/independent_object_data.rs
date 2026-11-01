//! Independent battle objects. Authored capabilities do not imply employee,
//! encounter-slot, objective, reward, or corpse ownership.
use std::{collections::BTreeMap, sync::Arc};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    ability::{SkillActivationMode, SkillId},
    battle::{
        damage::{DamageModifiers, DamageType},
        tile_range::TileRangePattern,
        types::{DeploymentAffinity, MobilityKind, UnitCombatProfile},
    },
    resources::Position,
    stats::UnitStats,
};

use super::{
    abnormality_data::{BasicAttackDef, MovementDef, ResonanceDef},
    skill_data::SkillDatabase,
    summoned_unit_data::{validate_stable_id, SummonerRemovalPolicyData},
    DataValidationError,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IndependentObjectId(pub String);
impl IndependentObjectId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl std::borrow::Borrow<str> for IndependentObjectId {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}
crate::authoring::clone_shape::newtype_shape!(IndependentObjectId);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum IndependentActorMotion {
    Stationary,
    Path { cells: Vec<Position> },
}
crate::authoring::clone_shape::enum_shape!(IndependentActorMotion {
    units: [Stationary],
    tuples: [],
    records: [Path { cells }]
});

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndependentActorDef {
    pub max_health: u32,
    pub attack: u32,
    pub defense: i32,
    pub magic_resistance: i32,
    pub movement: MovementDef,
    pub motion: IndependentActorMotion,
    pub mobility: MobilityKind,
    pub basic_attack: BasicAttackDef,
    pub resonance: ResonanceDef,
    pub skill_id: Option<SkillId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndependentTrapDef {
    pub trigger_range: TileRangePattern,
    pub damage_range: TileRangePattern,
    pub damage_flat: u32,
    pub attack_percent: u32,
    pub damage_type: DamageType,
    pub air_capable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_targets: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndependentWallDef {
    pub max_health: u32,
    pub defense: i32,
    pub magic_resistance: i32,
    pub inherit_source_defenses: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum IndependentObjectKind {
    Actor(IndependentActorDef),
    Trap(IndependentTrapDef),
    Wall(IndependentWallDef),
}
impl IndependentObjectKind {
    pub fn has_health(&self) -> bool {
        !matches!(self, Self::Trap(_))
    }
    fn kind_tag(&self) -> u8 {
        match self {
            Self::Actor(_) => 0,
            Self::Trap(_) => 1,
            Self::Wall(_) => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndependentObjectDefinition {
    pub id: IndependentObjectId,
    pub base_uuid: Uuid,
    pub family: String,
    pub display_name: String,
    pub ttl_ms: u32,
    pub max_alive: u32,
    pub max_total: u32,
    pub on_source_removed: SummonerRemovalPolicyData,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evolves_to: Option<IndependentObjectId>,
    pub kind: IndependentObjectKind,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompiledIndependentObject {
    pub definition: IndependentObjectDefinition,
    pub actor_profile: Option<UnitCombatProfile>,
}
crate::authoring::clone_shape::struct_shape!(MovementDef {
    speed_units_per_ms,
    radius_units
});
crate::authoring::clone_shape::struct_shape!(ResonanceDef {
    start,
    max,
    gain_lock_ms
});
crate::authoring::clone_shape::struct_shape!(IndependentActorDef {
    max_health,
    attack,
    defense,
    magic_resistance,
    movement,
    motion,
    mobility,
    basic_attack,
    resonance,
    skill_id
});
crate::authoring::clone_shape::struct_shape!(IndependentTrapDef {
    trigger_range,
    damage_range,
    damage_flat,
    attack_percent,
    damage_type,
    air_capable,
    max_targets
});
crate::authoring::clone_shape::struct_shape!(IndependentWallDef {
    max_health,
    defense,
    magic_resistance,
    inherit_source_defenses
});
crate::authoring::clone_shape::enum_shape!(IndependentObjectKind {
    units: [],
    tuples: [Actor(value), Trap(value), Wall(value)],
    records: []
});
crate::authoring::clone_shape::struct_shape!(IndependentObjectDefinition {
    id,
    base_uuid,
    family,
    display_name,
    ttl_ms,
    max_alive,
    max_total,
    on_source_removed,
    evolves_to,
    kind
});
crate::authoring::clone_shape::struct_shape!(CompiledIndependentObject {
    definition,
    actor_profile
});

#[derive(Debug, Clone, Default)]
pub struct IndependentObjectDatabase {
    definitions: BTreeMap<IndependentObjectId, Arc<CompiledIndependentObject>>,
}
impl IndependentObjectDatabase {
    pub fn get(&self, id: &IndependentObjectId) -> Option<&Arc<CompiledIndependentObject>> {
        self.definitions.get(id)
    }
    pub fn get_by_str(&self, id: &str) -> Option<&Arc<CompiledIndependentObject>> {
        self.definitions.get(id)
    }
    pub fn len(&self) -> usize {
        self.definitions.len()
    }
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
    pub fn definitions(&self) -> impl Iterator<Item = &Arc<CompiledIndependentObject>> {
        self.definitions.values()
    }
    pub(crate) fn validate_skill_references(
        &self,
        skills: &SkillDatabase,
    ) -> Result<(), DataValidationError> {
        for compiled in self.definitions.values() {
            if let Some(skill) = compiled
                .actor_profile
                .as_ref()
                .and_then(|p| p.skill_id.as_ref())
            {
                if skills.get_by_id(skill.as_str()).is_none() {
                    return Err(DataValidationError::at(
                        compiled.definition.id.as_str(),
                        "kind.Actor.skill_id",
                        format!("missing skill '{}'", skill.as_str()),
                    ));
                }
            }
        }
        for skill in &skills.skills {
            for step in &skill.steps {
                for effect in &step.effects {
                    if let crate::game::ability::SkillEffectDef::CreateObject { definition } =
                        &effect.op
                    {
                        let object = self.get(&definition.definition_id).ok_or_else(|| {
                            DataValidationError::at(
                                skill.id.as_str(),
                                format!(
                                    "steps.{}.effects.{}.op.CreateObject.definition_id",
                                    step.id, effect.id
                                ),
                                format!(
                                    "unknown independent object '{}'",
                                    definition.definition_id.as_str()
                                ),
                            )
                        })?;
                        if definition.wall_cost.is_some()
                            && !matches!(object.definition.kind, IndependentObjectKind::Wall(_))
                        {
                            return Err(DataValidationError::at(
                                skill.id.as_str(),
                                format!(
                                    "steps.{}.effects.{}.op.CreateObject.wall_cost",
                                    step.id, effect.id
                                ),
                                "wall_cost requires an HP wall definition",
                            ));
                        }
                        if definition.allow_same_source_evolution
                            && object.definition.evolves_to.is_none()
                        {
                            return Err(DataValidationError::at(skill.id.as_str(), format!("steps.{}.effects.{}.op.CreateObject.allow_same_source_evolution", step.id, effect.id), "evolution requires an authored evolves_to target"));
                        }
                        if definition.offsets.len() > object.definition.max_alive as usize {
                            return Err(DataValidationError::at(
                                skill.id.as_str(),
                                format!(
                                    "steps.{}.effects.{}.op.CreateObject.offsets",
                                    step.id, effect.id
                                ),
                                "batch exceeds authored max_alive",
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    pub(crate) fn compile(
        input: &[IndependentObjectDefinition],
    ) -> Result<Self, DataValidationError> {
        let mut definitions = BTreeMap::new();
        let mut uuids = BTreeMap::new();
        let mut families = BTreeMap::new();
        for definition in input {
            let id = definition.id.as_str();
            let error = |field: &str, reason: &str| DataValidationError::at(id, field, reason);
            // Admit the owned definition/profile copies and index nodes before allocation.
            // The existing outer loader retains the typed sticky interruption.
            use crate::authoring::{clone_shape::CloneShape, control};
            let admission = (|| {
                control::admit(
                    control::Resource::Definitions,
                    definitions.len(),
                    1,
                    "independent_object_definitions",
                )?;
                let mut meter = control::CloneMeter::default();
                definition.visit_clone_shape(&mut meter)?;
                definition.visit_clone_shape(&mut meter)?;
                meter.scan_storage(Some(16))?;
                meter.finish(
                    "independent_object_compile_storage",
                    "independent_object_compile_copy",
                )
            })();
            admission.map_err(|failure| {
                error(
                    "definition",
                    &format!("compilation admission failed: {failure:?}"),
                )
            })?;
            validate_stable_id(id, id, "id")?;
            validate_stable_id(&definition.family, id, "family")?;
            if definition.display_name.trim().is_empty() {
                return Err(error("display_name", "must not be empty"));
            }
            if definition.ttl_ms == 0 {
                return Err(error("ttl_ms", "must be positive"));
            }
            if definition.max_alive == 0 || definition.max_total < definition.max_alive {
                return Err(error("limits", "requires 0 < max_alive <= max_total"));
            }
            if definitions.contains_key(&definition.id) {
                return Err(error("id", "duplicate independent object id"));
            }
            if uuids.insert(definition.base_uuid, id).is_some() {
                return Err(error("base_uuid", "duplicate independent object UUID"));
            }
            let limits = (definition.max_alive, definition.max_total);
            if families
                .insert(&definition.family, limits)
                .is_some_and(|prior| prior != limits)
            {
                return Err(error(
                    "family",
                    "all forms in a family must declare the same creation limits",
                ));
            }
            let profile = match &definition.kind {
                IndependentObjectKind::Actor(actor) => {
                    if actor.max_health == 0
                        || actor.max_health > i32::MAX as u32
                        || actor.attack > i32::MAX as u32
                    {
                        return Err(error(
                            "kind.Actor",
                            "HP must be positive and HP/attack fit the signed runtime range",
                        ));
                    }
                    if actor.movement.speed_units_per_ms == 0 || actor.movement.radius_units == 0 {
                        return Err(error("kind.Actor.movement", "speed and radius must be positive; use Stationary motion to remain still"));
                    }
                    if matches!(&actor.motion, IndependentActorMotion::Path { cells } if cells.is_empty())
                    {
                        return Err(error("kind.Actor.motion", "path must contain cells"));
                    }
                    if actor.resonance.max == 0 || actor.resonance.start > actor.resonance.max {
                        return Err(error(
                            "kind.Actor.resonance",
                            "requires positive maximum and start <= maximum",
                        ));
                    }
                    actor
                        .basic_attack
                        .try_validate_runtime_contract()
                        .map_err(|e| {
                            error(
                                &format!("kind.Actor.basic_attack{}", e.field_suffix),
                                &e.cause,
                            )
                        })?;
                    let mut stats = UnitStats::with_values(
                        actor.max_health,
                        actor.max_health,
                        actor.attack,
                        actor.defense,
                        actor.basic_attack.interval_ms,
                    );
                    stats.magic_resist = actor.magic_resistance;
                    stats.move_speed_units_per_ms = actor.movement.speed_units_per_ms;
                    Some(UnitCombatProfile {
                        stats,
                        basic_attack: actor.basic_attack.clone(),
                        basic_attack_speed_percent: 100,
                        weapon_profile: None,
                        movement: actor.movement.clone(),
                        resonance: actor.resonance.clone(),
                        skill_id: actor.skill_id.clone(),
                        fragment_source: None,
                        skill_activation_mode: SkillActivationMode::Auto,
                        deployment_affinity: DeploymentAffinity::GroundOnly,
                        block_capacity: 0,
                        block_radius_units: 0.0,
                        blockable: !actor.mobility.is_airborne(),
                        mobility_kind: actor.mobility,
                        target_traits: if actor.mobility.is_airborne() {
                            vec![crate::game::battle::types::UnitTargetTrait::Airborne]
                        } else {
                            Vec::new()
                        },
                        incoming_damage_modifiers: DamageModifiers::default(),
                    })
                }
                IndependentObjectKind::Trap(trap) => {
                    if trap.max_targets == Some(0) {
                        return Err(error(
                            "kind.Trap.max_targets",
                            "must be positive when present",
                        ));
                    }
                    trap.trigger_range
                        .validate()
                        .map_err(|e| error("kind.Trap.trigger_range", &e))?;
                    trap.damage_range
                        .validate()
                        .map_err(|e| error("kind.Trap.damage_range", &e))?;
                    if trap.damage_flat > i32::MAX as u32
                        || trap.damage_flat == 0 && trap.attack_percent == 0
                    {
                        return Err(error(
                            "kind.Trap.damage",
                            "damage must have a positive coefficient and flat amount must fit i32",
                        ));
                    }
                    None
                }
                IndependentObjectKind::Wall(wall) => {
                    if wall.max_health == 0 || wall.max_health > i32::MAX as u32 {
                        return Err(error(
                            "kind.Wall.max_health",
                            "must be positive and fit i32",
                        ));
                    }
                    None
                }
            };
            definitions.insert(
                definition.id.clone(),
                Arc::new(CompiledIndependentObject {
                    definition: definition.clone(),
                    actor_profile: profile,
                }),
            );
        }
        for compiled in definitions.values() {
            let original = &compiled.definition;
            let mut cursor = original;
            let mut seen = std::collections::BTreeSet::new();
            while let Some(next) = &cursor.evolves_to {
                crate::authoring::control::charge(1, "independent_object_evolution_graph")
                    .map_err(|failure| {
                        DataValidationError::at(
                            original.id.as_str(),
                            "evolves_to",
                            format!("graph admission failed: {failure:?}"),
                        )
                    })?;
                if !seen.insert(cursor.id.clone()) {
                    return Err(DataValidationError::at(
                        original.id.as_str(),
                        "evolves_to",
                        "evolution cycle is not supported",
                    ));
                }
                let successor = definitions.get(next).ok_or_else(|| {
                    DataValidationError::at(
                        original.id.as_str(),
                        "evolves_to",
                        format!("missing object '{}'", next.as_str()),
                    )
                })?;
                let next = &successor.definition;
                if next.family != original.family
                    || next.kind.kind_tag() != original.kind.kind_tag()
                {
                    return Err(DataValidationError::at(
                        original.id.as_str(),
                        "evolves_to",
                        "evolution must preserve family and kind/HP capability",
                    ));
                }
                cursor = next;
            }
        }
        Ok(Self { definitions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::summoned_unit_data::SummonedUnitCatalogData;

    fn wall(id: &str, uuid: u128) -> IndependentObjectDefinition {
        IndependentObjectDefinition {
            id: IndependentObjectId::new(id),
            base_uuid: Uuid::from_u128(uuid),
            family: "wall_family".into(),
            display_name: id.into(),
            ttl_ms: 10_000,
            max_alive: 2,
            max_total: 4,
            on_source_removed: SummonerRemovalPolicyData::Persist,
            evolves_to: None,
            kind: IndependentObjectKind::Wall(IndependentWallDef {
                max_health: 100,
                defense: 4,
                magic_resistance: 7,
                inherit_source_defenses: true,
            }),
        }
    }

    #[test]
    fn u07_object_schema_preserves_absent_catalog_bytes_and_strict_roundtrip() {
        let old = "(definitions:[])";
        let parsed: SummonedUnitCatalogData = ron::de::from_str(old).unwrap();
        assert!(parsed.objects.is_empty());
        // RON permits an optional trailing comma. Persisted identity uses the
        // typed JSON/canonical representation, whose absent field must not change.
        #[derive(Serialize)]
        struct LegacyCatalog {
            definitions: Vec<super::super::summoned_unit_data::SummonedUnitDefinitionData>,
        }
        let legacy = LegacyCatalog {
            definitions: Vec::new(),
        };
        assert_eq!(
            serde_json::to_vec(&parsed).unwrap(),
            serde_json::to_vec(&legacy).unwrap()
        );

        let with_objects = SummonedUnitCatalogData {
            definitions: Vec::new(),
            objects: vec![wall("ice_wall", 1)],
        };
        let encoded = ron::ser::to_string(&with_objects).unwrap();
        let restored: SummonedUnitCatalogData = ron::de::from_str(&encoded).unwrap();
        let compiled = restored.compile().unwrap();
        let result = compiled
            .objects
            .get(&IndependentObjectId::new("ice_wall"))
            .unwrap();
        assert!(
            result.actor_profile.is_none(),
            "an HP object does not become an actor profile"
        );
        assert!(result.definition.kind.has_health());
        let bad = encoded.replace(
            "inherit_source_defenses:true",
            "inherit_source_defenses:true,unknown_capability:true",
        );
        assert_ne!(bad, encoded);
        assert!(ron::de::from_str::<SummonedUnitCatalogData>(&bad).is_err());
        assert!(ron::de::from_str::<SummonedUnitCatalogData>(
            &encoded.replace("max_health:100", "max_health:-1")
        )
        .is_err());
    }

    #[test]
    fn u07_object_evolution_rejects_cycle_foreign_family_kind_and_limit_drift() {
        let mut a = wall("small_wall", 1);
        let b = wall("big_wall", 2);
        a.evolves_to = Some(b.id.clone());
        assert_eq!(
            IndependentObjectDatabase::compile(&[a.clone(), b.clone()])
                .unwrap()
                .len(),
            2
        );
        let mut cycle = b.clone();
        cycle.evolves_to = Some(a.id.clone());
        assert!(IndependentObjectDatabase::compile(&[a.clone(), cycle])
            .unwrap_err()
            .cause
            .contains("cycle"));
        assert!(IndependentObjectDatabase::compile(&[a.clone()])
            .unwrap_err()
            .cause
            .contains("missing object"));
        let mut foreign = b.clone();
        foreign.family = "another_family".into();
        assert!(IndependentObjectDatabase::compile(&[a.clone(), foreign])
            .unwrap_err()
            .cause
            .contains("preserve family"));
        let mut limits = b.clone();
        limits.max_total += 1;
        assert!(IndependentObjectDatabase::compile(&[a.clone(), limits])
            .unwrap_err()
            .cause
            .contains("same creation limits"));
        let mut kind = b;
        kind.kind = IndependentObjectKind::Trap(IndependentTrapDef {
            trigger_range: TileRangePattern {
                rows: vec!["@".into()],
                include_anchor_tile: true,
            },
            damage_range: TileRangePattern {
                rows: vec!["@".into()],
                include_anchor_tile: true,
            },
            damage_flat: 1,
            attack_percent: 0,
            damage_type: DamageType::Physical,
            air_capable: false,
            max_targets: None,
        });
        let error = IndependentObjectDatabase::compile(&[a, kind]).unwrap_err();
        assert!(error.cause.contains("kind/HP capability"), "{error:?}");
    }

    #[test]
    fn u07_object_positive_limits_hp_and_duplicate_uuid_are_not_normalized() {
        let original = wall("wall", 1);
        let mut zero = original.clone();
        zero.ttl_ms = 0;
        assert_eq!(
            IndependentObjectDatabase::compile(&[zero])
                .unwrap_err()
                .field_path,
            Some("ttl_ms".into())
        );
        let mut limits = original.clone();
        limits.max_alive = 5;
        assert!(IndependentObjectDatabase::compile(&[limits])
            .unwrap_err()
            .cause
            .contains("max_alive"));
        let mut hp = original.clone();
        let IndependentObjectKind::Wall(value) = &mut hp.kind else {
            unreachable!()
        };
        value.max_health = 0;
        assert!(IndependentObjectDatabase::compile(&[hp])
            .unwrap_err()
            .cause
            .contains("positive"));
        let duplicate = wall("another_wall", 1);
        assert!(IndependentObjectDatabase::compile(&[original, duplicate])
            .unwrap_err()
            .cause
            .contains("UUID"));
    }

    #[test]
    fn u07_object_stationary_actor_has_real_profile_and_explicit_motion() {
        let mut definition = wall("ally_turret", 1);
        definition.kind = IndependentObjectKind::Actor(IndependentActorDef {
            max_health: 50,
            attack: 12,
            defense: 3,
            magic_resistance: 4,
            movement: MovementDef::default(),
            motion: IndependentActorMotion::Stationary,
            mobility: MobilityKind::Ground,
            basic_attack: BasicAttackDef::default(),
            resonance: ResonanceDef::default(),
            skill_id: None,
        });
        let compiled = IndependentObjectDatabase::compile(&[definition.clone()]).unwrap();
        let actor = compiled.get(&definition.id).unwrap();
        assert_eq!(actor.actor_profile.as_ref().unwrap().stats.attack, 12);
        assert!(
            actor
                .actor_profile
                .as_ref()
                .unwrap()
                .stats
                .move_speed_units_per_ms
                > 0
        );
        let IndependentObjectKind::Actor(actor) = &mut definition.kind else {
            unreachable!()
        };
        actor.motion = IndependentActorMotion::Path { cells: Vec::new() };
        assert!(IndependentObjectDatabase::compile(&[definition])
            .unwrap_err()
            .cause
            .contains("path must"));
    }

    #[test]
    fn u07_object_compiled_clone_is_independent_and_admitted_before_copy() {
        use crate::authoring::{
            clone_shape::{controlled_clone, CloneShape},
            control,
        };
        let definition = wall("wall", 1);
        let db = IndependentObjectDatabase::compile(&[definition.clone()]).unwrap();
        let original = db.get(&definition.id).unwrap();
        let mut copy = controlled_clone(original.as_ref()).unwrap();
        copy.definition.family = "changed".into();
        assert_eq!(original.definition.family, "wall_family");
        let mut meter = control::CloneMeter::default();
        original.as_ref().visit_clone_shape(&mut meter).unwrap();
        assert!(meter.native_scan_slots() > 0);
        let mut limits = control::ExecutionLimits::default();
        limits.max_clone_slots = 1;
        let outcome = control::run(
            &limits,
            &control::CancellationToken::default(),
            control::ExecutionPhase::FinalValidation,
            || controlled_clone(original.as_ref()),
        );
        assert!(
            outcome.is_err(),
            "a one-slot limit must refuse before cloning the definition"
        );
        assert_eq!(original.definition.family, "wall_family");
    }
}
