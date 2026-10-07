use super::{
    compiled::{ActorView, EncounterView},
    output,
    query::request_error,
    AuthoringContent, AuthoringError,
};
use crate::game::data::{
    boss_runtime_data::{BossActorBehaviorId, BossEncounterRuntimeId, BossPatternId, BossSignalId},
    content_pack::{
        ContentIdentity, DefinitionKey, DefinitionKind, DefinitionSource, DefinitionWriter,
    },
    summoned_unit_data::SummonedUnitDefinitionId,
};
use serde::Serialize;

#[derive(Serialize)]
struct Explanation<'a, T: Serialize> {
    identity: &'a ContentIdentity,
    key: &'a DefinitionKey,
    present: bool,
    source: Option<&'a DefinitionSource>,
    writers: &'a [&'a DefinitionWriter],
    representation: &'static str,
    index_space: Option<&'static str>,
    value: Option<T>,
}

fn encode<T: Serialize>(
    content: &AuthoringContent,
    key: &DefinitionKey,
    source: Option<&DefinitionSource>,
    writers: &[&DefinitionWriter],
    representation: &'static str,
    value: Option<T>,
) -> Result<String, AuthoringError> {
    output::encode_result(&Explanation {
        identity: content.token.identity(),
        key,
        present: source.is_some(),
        source,
        writers,
        representation,
        index_space: (representation == "compiled_graph").then_some("compiled"),
        value,
    })
}

pub(super) fn document(
    content: &AuthoringContent,
    key: &DefinitionKey,
    source: Option<&DefinitionSource>,
    writers: &[&DefinitionWriter],
) -> Result<String, AuthoringError> {
    if source.is_none() {
        return encode::<()>(content, key, source, writers, "removed", None);
    }
    let db = content.token.database();
    let id = key.id.as_str();
    macro_rules! emit {
        ($value:expr) => {
            emit!("resolved_definition", $value)
        };
        ($representation:expr, $value:expr) => {{
            let value = ($value).ok_or_else(|| {
                request_error(
                    "authoring_definition_projection_missing",
                    "key",
                    "Final source exists but its compiled database record is absent.",
                )
            })?;
            encode(content, key, source, writers, $representation, Some(value))
        }};
    }
    use DefinitionKind::*;
    match key.kind {
        Relic => emit!(db.relic_data.get_by_id(id)),
        Resource => {
            super::control::charge(
                db.resource_data.resources.len() as u64,
                "resource_definition_projection",
            )
            .map_err(super::query::control_error)?;
            emit!(db
                .resource_data
                .resources
                .iter()
                .find(|value| value.id.as_str() == id))
        }
        ShopPreset => emit!(db.shop_data.preset_by_id(id)),
        Shop => emit!(db.shop_data.get_by_id(id)),
        ShopPool => emit!(db.shop_data.pools.iter().find(|v| v.id == id)),
        Reward => emit!(db.reward_data.get_by_id(id)),
        RewardPool => emit!(db.reward_data.pools.iter().find(|v| v.id == id)),
        EquipmentRewardPool => emit!(db.reward_data.equipment_pools.iter().find(|v| v.id == id)),
        Event => emit!(db.event_data.get_by_id(id)),
        VisualNovel => emit!(db.visual_novel_data.get_by_id(id)),
        VisualNovelTrigger => emit!(db.visual_novel_trigger_data.get_by_id(id)),
        BossOmenChain => emit!(db.boss_omen_data.get_by_id(id)),
        BossPattern => emit!(
            "compiled_graph",
            db.boss_runtime_data.get_pattern(&BossPatternId::new(id))
        ),
        BossActorBehavior => emit!(
            "compiled_graph",
            db.boss_runtime_data
                .get_actor_behavior(&BossActorBehaviorId::new(id))
                .map(|v| ActorView::new(v))
        ),
        BossEncounter => emit!(
            "compiled_graph",
            db.boss_runtime_data
                .get_encounter(&BossEncounterRuntimeId::new(id))
                .map(|v| EncounterView::new(v))
        ),
        BossSignal => {
            #[derive(Serialize)]
            struct Signal<'a> {
                id: &'a str,
            }
            emit!(
                "compiled_declaration",
                db.boss_runtime_data
                    .declares_signal(&BossSignalId::new(id))
                    .then_some(Signal { id })
            )
        }
        IndependentObject => emit!(
            "compiled_definition",
            db.boss_runtime_data
                .independent_objects()
                .get(&crate::game::data::independent_object_data::IndependentObjectId::new(id))
        ),
        SummonedUnit => emit!(
            "compiled_definition",
            db.boss_runtime_data
                .get_summoned_unit(&SummonedUnitDefinitionId::new(id))
        ),
        Abnormality => emit!(db.abnormality_data.get_by_id(id)),
        CorrodedRangePreset => emit!(db
            .corroded_employee_data
            .range_presets
            .iter()
            .find(|v| v.id == id)),
        CorrodedProfile => emit!(db.corroded_employee_data.get_by_id(id)),
        CorrodedWavePreset => emit!(db.corroded_wave_data.get_by_id(id)),
        Employee => emit!(db.employee_definition_data.get_by_id(id)),
        Equipment => emit!(db.equipment_data.get_by_id(id)),
        EquipmentMaterial => emit!(db.equipment_data.get_material_by_id(id)),
        DismantleRecipe => emit!(db.equipment_data.get_dismantle_recipe_by_equipment_id(id)),
        EnhancementRecipe => emit!(db.equipment_data.get_enhancement_recipe_by_equipment_id(id)),
        Consumable => emit!(db.consumable_data.get_by_id(id)),
        Buff => emit!(db.buff_data.buffs.iter().find(|v| v.name == id)),
        SkillRangePreset => emit!(db.skill_data.range_presets.iter().find(|v| v.id == id)),
        Skill => emit!(db.skill_data.get_by_id(id)),
        SkillFragment => emit!(db.skill_fragment_data.get_by_id_str(id)),
        PveEncounter => emit!(db.pve_data.get_by_id(id)),
        PveRewardSource => emit!(db
            .pve_data
            .reward_sources
            .iter()
            .find(|v| v.reward_source_id == id)),
        RunPolicy => emit!((id == "run_policy").then_some(db.run_policy.as_ref())),
        MapNode => emit!(db
            .map_node_definitions()
            .nodes
            .iter()
            .find(|v| v.kind_id.as_str() == id)),
        FacilityTags => {
            emit!((id == "tag_catalog").then_some(&db.facility_templates().tag_catalog))
        }
        FacilityTemplate => emit!(db
            .facility_templates()
            .templates
            .iter()
            .find(|v| v.id.as_str() == id)),
        FacilityPool => emit!(db.facility_templates().pools.iter().find(|v| v.id == id)),
        NodeAssignmentProfile => emit!(db
            .node_assignment_profiles()
            .profiles
            .iter()
            .find(|v| v.id == id)),
        StandardSchedule => emit!(db
            .node_assignment_profiles()
            .standard_schedule
            .iter()
            .find(|v| v.floor.to_string() == id)),
        EndlessSchedule => {
            emit!((id == "endless_stages").then_some(&db.node_assignment_profiles().endless_stages))
        }
        BattlefieldSize => emit!(db
            .map_catalogs
            .battlefield_archetypes
            .size_classes
            .iter()
            .find(|v| format!("{:?}", v.size_class) == id)),
        BattlefieldArchetype => emit!(db
            .map_catalogs
            .battlefield_archetypes
            .archetypes
            .iter()
            .find(|v| format!("{:?}", v.archetype) == id)),
        BattlefieldTemplate => emit!(db
            .map_catalogs
            .battlefield_templates
            .templates
            .iter()
            .find(|v| v.id == id)),
    }
}

pub(super) fn skill_flow_document(
    content: &AuthoringContent,
    key: &DefinitionKey,
    source: Option<&DefinitionSource>,
    writers: &[&DefinitionWriter],
) -> Result<String, AuthoringError> {
    let plan = content
        .token
        .database()
        .skill_data
        .compiled_flow(&key.id)
        .map_err(|cause| {
            request_error("authoring_definition_projection_missing", "flow", &cause)
        })?;
    encode(content, key, source, writers, "compiled_graph", Some(plan))
}
