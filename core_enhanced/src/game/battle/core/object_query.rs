//! Read-only projections of the object owner and existing request ledger.
use super::{BattleCore, ObjectRemovalReceipt, SpawnRequestSettlement};
use crate::game::{
    ability::{authoring::EffectSite, SkillId},
    battle::{
        ids::UnitInstanceId,
        runtime_facts::{ActionOrdinal, RuntimeFactSource},
    },
    behavior::GameError,
    data::independent_object_data::{IndependentObjectId, IndependentObjectKind},
    enums::Side,
    resources::Position,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKindView {
    Actor,
    Trap,
    Wall,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectSourceView {
    pub creator: UnitInstanceId,
    pub creation_action: Option<u64>,
    pub skill_id: Option<SkillId>,
    pub cast_sequence: Option<u64>,
    pub fragment_copy_uuid: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fragment_definition_id: Option<crate::game::data::skill_fragment_data::SkillFragmentId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fragment_form_id: Option<String>,
    pub equipment_runtime_uuid: Option<Uuid>,
    pub equipment_definition_uuid: Option<Uuid>,
    pub equipment_copy_uuid: Option<Uuid>,
    pub authored_site: Option<EffectSite>,
}
impl ObjectSourceView {
    pub(super) fn from_runtime(
        creator: UnitInstanceId,
        action: Option<ActionOrdinal>,
        source: &RuntimeFactSource,
    ) -> Result<Self, GameError> {
        source.admit_owner("object_source_query")?;
        Ok(Self {
            creator,
            creation_action: action.map(|id| id.0),
            skill_id: source
                .cast
                .as_ref()
                .map(|cast| crate::authoring::clone_shape::controlled_clone(&cast.skill_id))
                .transpose()?,
            cast_sequence: source.cast.as_ref().map(|cast| cast.cast_seq),
            fragment_copy_uuid: source.fragment.as_ref().map(|fragment| fragment.copy_uuid),
            fragment_definition_id: source
                .fragment
                .as_ref()
                .map(|fragment| {
                    crate::authoring::clone_shape::controlled_clone(&fragment.definition_id)
                })
                .transpose()?,
            fragment_form_id: source
                .fragment
                .as_ref()
                .map(|fragment| crate::authoring::clone_shape::controlled_clone(&fragment.form_id))
                .transpose()?,
            equipment_runtime_uuid: source.equipment.map(|e| e.runtime_item_instance_id),
            equipment_definition_uuid: source.equipment.map(|e| e.definition_uuid),
            equipment_copy_uuid: source.equipment.and_then(|e| e.persistent_copy_uuid),
            authored_site: source
                .authored
                .as_ref()
                .map(|a| crate::authoring::clone_shape::controlled_clone(&a.site))
                .transpose()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndependentObjectView {
    pub object_id: UnitInstanceId,
    pub definition_id: IndependentObjectId,
    pub kind: ObjectKindView,
    pub side: Side,
    pub position: Position,
    pub epoch: u64,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub remaining_ms: u64,
    pub source: ObjectSourceView,
    pub creation_attack: u32,
    pub current_health: Option<u32>,
    pub max_health: Option<u32>,
    pub defense: Option<i32>,
    pub magic_resistance: Option<i32>,
    pub can_move: bool,
    pub can_attack: bool,
    pub can_receive_heal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectCreationReceipt {
    pub request_id: String,
    pub definition_id: String,
    pub side: Side,
    pub time_ms: u64,
    pub positions: Vec<Position>,
    pub source: ObjectSourceView,
    pub settlement: Option<SpawnRequestSettlement>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndependentObjectsSnapshot {
    pub active: Vec<IndependentObjectView>,
    pub creations: Vec<ObjectCreationReceipt>,
    pub last_removal: Option<ObjectRemovalReceipt>,
}

impl BattleCore {
    pub fn independent_objects_snapshot(
        &self,
        now_ms: u64,
    ) -> Result<IndependentObjectsSnapshot, GameError> {
        use crate::game::battle::runtime_control;
        let count = self.independent_objects.objects.len();
        crate::authoring::control::charge(count as u64, "object_query")?;
        runtime_control::admit_retained_growth(
            0,
            self.independent_objects.retained_extent().and_then(|n| {
                n.checked_add(count.checked_mul(std::mem::size_of::<IndependentObjectView>())?)
            }),
            "object_query_storage",
        )?;
        let mut active = Vec::with_capacity(count);
        for object in self.independent_objects.objects.values() {
            let definition = self
                .game_data
                .boss_runtime_data
                .independent_objects()
                .get(&object.definition)
                .ok_or(GameError::InvalidAction)?;
            let kind = match definition.definition.kind {
                IndependentObjectKind::Actor(_) => ObjectKindView::Actor,
                IndependentObjectKind::Trap(_) => ObjectKindView::Trap,
                IndependentObjectKind::Wall(_) => ObjectKindView::Wall,
            };
            let unit = self.units.get(&object.id);
            active.push(IndependentObjectView {
                object_id: object.id,
                definition_id: crate::authoring::clone_shape::controlled_clone(&object.definition)?,
                kind,
                side: object.side,
                position: unit.map_or(object.position, |unit| unit.body.projected_tile()),
                epoch: object.epoch,
                created_at_ms: object.created_at_ms,
                expires_at_ms: object.expires_at_ms,
                remaining_ms: object.expires_at_ms.saturating_sub(now_ms),
                source: ObjectSourceView::from_runtime(
                    object.family.creator,
                    object.creation_action,
                    &object.source,
                )?,
                creation_attack: object.source_attack,
                current_health: unit.map(|u| u.stats.current_health),
                max_health: unit.map(|u| u.stats.max_health),
                defense: unit.map(|u| u.stats.defense),
                magic_resistance: unit.map(|u| u.stats.magic_resist),
                can_move: unit.is_some_and(|u| u.can_move()),
                can_attack: kind == ObjectKindView::Trap
                    || unit.is_some_and(|u| u.can_basic_attack()),
                can_receive_heal: kind == ObjectKindView::Actor,
            });
        }
        Ok(IndependentObjectsSnapshot {
            active,
            creations: self.spawn_kernel.independent_request_snapshots()?,
            last_removal: crate::authoring::clone_shape::controlled_clone(
                &self.independent_objects.last_removal,
            )?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentWallCostPreview {
    pub activation: u64,
    pub pool_id: String,
    pub before: u32,
    pub detached: u32,
    pub after: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectPlacementPreview {
    pub definition_id: IndependentObjectId,
    pub result_definition_id: IndependentObjectId,
    pub position: Position,
    pub evolves_object_id: Option<UnitInstanceId>,
    pub expected_health: Option<u32>,
    pub expected_max_health: Option<u32>,
    pub duration_ms: u32,
    pub health_cost: u32,
    pub segment_cost: Option<SegmentWallCostPreview>,
}

impl BattleCore {
    /// A read-only forecast at the current state. Acceptance and release each
    /// validate again; this DTO carries no action/resource/spawn authority.
    pub fn object_creation_preview(
        &self,
        now_ms: u64,
        creator: UnitInstanceId,
        skill_id: &SkillId,
        target: Option<crate::game::battle::event_log::SkillCastTarget>,
    ) -> Result<Vec<ObjectPlacementPreview>, GameError> {
        use super::sustained::WallCostPlan;
        use crate::game::ability::SkillEffectDef;
        let skill = self
            .game_data
            .skill_data
            .get_by_id(skill_id.as_str())
            .ok_or(GameError::InvalidAction)?;
        self.validate_skill_object_placements(now_ms, creator, skill, target)?;
        let count =
            skill
                .steps
                .iter()
                .flat_map(|step| &step.effects)
                .try_fold(0_usize, |sum, effect| {
                    let count = match &effect.op {
                        SkillEffectDef::CreateObject { definition } => definition.offsets.len(),
                        _ => 0,
                    };
                    sum.checked_add(count)
                });
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            count.and_then(|n| n.checked_mul(std::mem::size_of::<ObjectPlacementPreview>())),
            "object_placement_preview",
        )?;
        let count = count.ok_or(GameError::InvalidAction)?;
        let mut meter = crate::authoring::control::CloneMeter::default();
        meter.scan_storage(Some(count))?;
        meter.finish("object_placement_preview", "object_placement_preview_copy")?;
        let mut previews = Vec::with_capacity(count);
        for effect in skill.steps.iter().flat_map(|step| &step.effects) {
            let SkillEffectDef::CreateObject { definition } = &effect.op else {
                continue;
            };
            let object = self
                .game_data
                .boss_runtime_data
                .independent_objects()
                .get(&definition.definition_id)
                .ok_or(GameError::InvalidAction)?;
            let key = self.object_preview_family(creator, skill, object)?;
            let payment = definition
                .wall_cost
                .as_ref()
                .map(|cost| {
                    self.plan_object_wall_cost(
                        creator,
                        cost,
                        key.fragment_copy,
                        key.equipment_copy,
                        now_ms,
                    )
                })
                .transpose()?;
            let positions = self.object_creation_positions(creator, definition, target)?;
            for position in positions {
                let evolution = self.independent_evolution_candidate(
                    object,
                    &key,
                    position,
                    definition.allow_same_source_evolution,
                )?;
                let resolved = evolution
                    .as_ref()
                    .map_or(object.as_ref(), |(_, next)| next.as_ref());
                let max_health = payment
                    .map(|plan| plan.wall_health())
                    .or_else(|| match &resolved.definition.kind {
                        IndependentObjectKind::Actor(actor) => Some(actor.max_health),
                        IndependentObjectKind::Wall(wall) => Some(wall.max_health),
                        IndependentObjectKind::Trap(_) => None,
                    });
                let health = max_health.map(|max| {
                    evolution
                        .as_ref()
                        .and_then(|(id, _)| self.units.get(id))
                        .map_or(max, |unit| unit.stats.current_health.min(max))
                });
                let segment_cost = if let Some(WallCostPlan::Segment {
                    activation,
                    before,
                    detached,
                }) = payment
                {
                    let pool = &definition
                        .wall_cost
                        .as_ref()
                        .ok_or(GameError::InvalidAction)?
                        .pool_id;
                    crate::game::battle::runtime_control::admit_retained_growth(
                        0,
                        Some(pool.len()),
                        "object_segment_cost_preview",
                    )?;
                    Some(SegmentWallCostPreview {
                        activation,
                        pool_id: crate::authoring::clone_shape::controlled_clone(pool)?,
                        before,
                        detached,
                        after: before - detached,
                    })
                } else {
                    None
                };
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    definition
                        .definition_id
                        .0
                        .len()
                        .checked_add(resolved.definition.id.0.len()),
                    "object_preview_definitions",
                )?;
                previews.push(ObjectPlacementPreview {
                    definition_id: crate::authoring::clone_shape::controlled_clone(
                        &definition.definition_id,
                    )?,
                    result_definition_id: crate::authoring::clone_shape::controlled_clone(
                        &resolved.definition.id,
                    )?,
                    position,
                    evolves_object_id: evolution.as_ref().map(|(id, _)| *id),
                    expected_health: health,
                    expected_max_health: max_health,
                    duration_ms: resolved.definition.ttl_ms,
                    health_cost: match payment {
                        Some(WallCostPlan::Health { amount, .. }) => amount,
                        _ => 0,
                    },
                    segment_cost,
                });
            }
        }
        Ok(previews)
    }
}
