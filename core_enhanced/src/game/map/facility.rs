use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::{GameMode, MapEdgeDirection, MapNodeCategory, MapNodeKindId, MapSlotId, MapTemplateId};

pub const FACILITY_TEMPLATE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityTemplateDatabase {
    pub tag_catalog: Vec<String>,
    pub templates: Vec<FacilityTemplate>,
    pub pools: Vec<FacilityTemplatePool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityTemplate {
    pub id: MapTemplateId,
    pub version: u32,
    pub kind: FacilityTemplateKind,
    #[serde(default)]
    pub tags: Vec<String>,
    pub slots: Vec<FacilityRoomSlot>,
    pub connections: Vec<FacilityConnection>,
    #[serde(default)]
    pub manifestation_rules: Vec<FacilityManifestationRule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FacilityTemplateKind {
    Exploration,
    Puzzle,
    Capstone,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityRoomSlot {
    pub id: MapSlotId,
    pub role: FacilitySlotRole,
    pub allowed_categories: Vec<MapNodeCategory>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FacilitySlotRole {
    Start,
    Entry,
    Ordinary,
    ManifestedBonus,
    Terminal(FacilityTerminalResolution),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FacilityTerminalResolution {
    FloorGate,
    RunFinalBoss,
    BossOmenExitGate,
    SpecialRunCompletion,
}

impl FacilityTerminalResolution {
    pub fn required_category(self) -> MapNodeCategory {
        match self {
            Self::FloorGate | Self::BossOmenExitGate => MapNodeCategory::Gate,
            Self::RunFinalBoss | Self::SpecialRunCompletion => MapNodeCategory::Boss,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityConnection {
    pub from_slot_id: MapSlotId,
    pub to_slot_id: MapSlotId,
    pub direction: MapEdgeDirection,
    #[serde(default)]
    pub reveal_with_slot_id: Option<MapSlotId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityManifestationRule {
    pub id: String,
    pub prerequisite_slot_ids: Vec<MapSlotId>,
    pub manifest_slot_id: MapSlotId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityTemplatePool {
    pub id: String,
    pub template_ids: Vec<MapTemplateId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeAssignmentProfileDatabase {
    pub profiles: Vec<NodeAssignmentProfile>,
    pub standard_schedule: Vec<StandardFacilityScheduleEntry>,
    pub endless_stages: Vec<EndlessFacilityScheduleStage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeAssignmentProfile {
    pub id: String,
    pub difficulty_profile_id: String,
    pub category_constraints: Vec<FacilityCategoryConstraint>,
    pub remaining_category_weights: Vec<FacilityCategoryWeight>,
    #[serde(default)]
    pub tagged_slot_assignments: Vec<TaggedSlotAssignment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityCategoryConstraint {
    pub category: MapNodeCategory,
    #[serde(default)]
    pub min: usize,
    #[serde(default)]
    pub max: Option<usize>,
    #[serde(default)]
    pub exact: Option<usize>,
}

impl FacilityCategoryConstraint {
    pub fn bounds(self) -> Result<(usize, usize), String> {
        if let Some(exact) = self.exact {
            if self.min != 0 || self.max.is_some() {
                return Err(format!(
                    "category {:?} exact cannot be combined with min/max",
                    self.category
                ));
            }
            return Ok((exact, exact));
        }
        let max = self.max.unwrap_or(usize::MAX);
        if self.min > max {
            return Err(format!(
                "category {:?} min {} exceeds max {}",
                self.category, self.min, max
            ));
        }
        Ok((self.min, max))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityCategoryWeight {
    pub category: MapNodeCategory,
    pub weight: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaggedSlotAssignment {
    pub slot_tag: String,
    pub category: MapNodeCategory,
    #[serde(default)]
    pub kind_id: Option<MapNodeKindId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StandardFacilityScheduleEntry {
    pub floor: u8,
    pub pool_id: String,
    pub profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndlessFacilityScheduleStage {
    pub min_floor: u32,
    pub max_floor: Option<u32>,
    pub cycle: Vec<FacilityScheduleCycleEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityScheduleCycleEntry {
    pub pool_id: String,
    pub profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FacilityRunState {
    pub pools: BTreeMap<String, FacilityPoolRunState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FacilityPoolRunState {
    pub cycle: u32,
    pub remaining_template_ids: Vec<MapTemplateId>,
    pub used_template_ids: BTreeSet<MapTemplateId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFacilitySchedule<'a> {
    pub pool: &'a FacilityTemplatePool,
    pub profile: &'a NodeAssignmentProfile,
}

impl FacilityTemplateDatabase {
    pub fn from_ron_str(input: &str) -> Result<Self, String> {
        ron::de::from_str(input).map_err(|error| error.to_string())
    }

    pub fn template(&self, id: &MapTemplateId) -> Option<&FacilityTemplate> {
        self.templates.iter().find(|template| template.id == *id)
    }

    pub fn pool(&self, id: &str) -> Option<&FacilityTemplatePool> {
        self.pools.iter().find(|pool| pool.id == id)
    }

    pub fn validate_contract(&self) -> Result<(), String> {
        let tag_catalog =
            validate_non_empty_unique_strings("facility tag catalog", &self.tag_catalog)?;
        if self.templates.is_empty() {
            return Err("facility templates must not be empty".to_string());
        }
        if self.pools.is_empty() {
            return Err("facility template pools must not be empty".to_string());
        }

        let mut template_ids = HashSet::new();
        for template in &self.templates {
            if template.id.as_str().trim().is_empty() {
                return Err("facility template id must not be empty".to_string());
            }
            if !template_ids.insert(template.id.as_str()) {
                return Err(format!(
                    "duplicate facility template '{}'",
                    template.id.as_str()
                ));
            }
            validate_template(template, &tag_catalog)?;
        }

        let mut pool_ids = HashSet::new();
        for pool in &self.pools {
            if pool.id.trim().is_empty() || !pool_ids.insert(pool.id.as_str()) {
                return Err(format!("invalid or duplicate facility pool '{}'", pool.id));
            }
            if pool.template_ids.is_empty() {
                return Err(format!("facility pool '{}' must not be empty", pool.id));
            }
            let mut members = HashSet::new();
            for template_id in &pool.template_ids {
                if self.template(template_id).is_none() {
                    return Err(format!(
                        "facility pool '{}' references missing template '{}'",
                        pool.id,
                        template_id.as_str()
                    ));
                }
                if !members.insert(template_id.as_str()) {
                    return Err(format!(
                        "facility pool '{}' contains duplicate template '{}'",
                        pool.id,
                        template_id.as_str()
                    ));
                }
            }
        }
        Ok(())
    }
}

impl FacilityTemplate {
    pub fn navigation_start_slot_ids(&self) -> Vec<MapSlotId> {
        let mut slot_ids = BTreeSet::new();
        for slot in &self.slots {
            match slot.role {
                FacilitySlotRole::Entry => {
                    slot_ids.insert(slot.id.clone());
                }
                FacilitySlotRole::Start => {
                    for connection in &self.connections {
                        if connection.from_slot_id == slot.id {
                            slot_ids.insert(connection.to_slot_id.clone());
                        }
                        if connection.direction == MapEdgeDirection::Bidirectional
                            && connection.to_slot_id == slot.id
                        {
                            slot_ids.insert(connection.from_slot_id.clone());
                        }
                    }
                }
                FacilitySlotRole::Ordinary
                | FacilitySlotRole::ManifestedBonus
                | FacilitySlotRole::Terminal(_) => {}
            }
        }
        slot_ids.into_iter().collect()
    }

    pub fn neighbor_slot_ids(&self, slot_id: &MapSlotId) -> Vec<MapSlotId> {
        let mut neighbors = BTreeSet::new();
        for connection in &self.connections {
            if connection.from_slot_id == *slot_id {
                neighbors.insert(connection.to_slot_id.clone());
            }
            if connection.direction == MapEdgeDirection::Bidirectional
                && connection.to_slot_id == *slot_id
            {
                neighbors.insert(connection.from_slot_id.clone());
            }
        }
        neighbors.into_iter().collect()
    }
}

impl NodeAssignmentProfileDatabase {
    pub fn from_ron_str(input: &str) -> Result<Self, String> {
        ron::de::from_str(input).map_err(|error| error.to_string())
    }

    pub fn profile(&self, id: &str) -> Option<&NodeAssignmentProfile> {
        self.profiles.iter().find(|profile| profile.id == id)
    }

    pub fn resolve<'a>(
        &'a self,
        templates: &'a FacilityTemplateDatabase,
        mode: GameMode,
        floor_index: u32,
    ) -> Result<ResolvedFacilitySchedule<'a>, String> {
        let floor = floor_index
            .checked_add(1)
            .ok_or_else(|| "facility floor overflowed u32".to_string())?;
        let (pool_id, profile_id) = match mode {
            GameMode::Standard => {
                let matches = self
                    .standard_schedule
                    .iter()
                    .filter(|entry| u32::from(entry.floor) == floor)
                    .collect::<Vec<_>>();
                if matches.len() != 1 {
                    return Err(format!(
                        "Standard floor {floor} must resolve exactly one facility schedule, got {}",
                        matches.len()
                    ));
                }
                (&matches[0].pool_id, &matches[0].profile_id)
            }
            GameMode::Endless => {
                let stage = self
                    .endless_stages
                    .iter()
                    .find(|stage| {
                        floor >= stage.min_floor
                            && stage.max_floor.is_none_or(|max_floor| floor <= max_floor)
                    })
                    .ok_or_else(|| {
                        format!("Endless floor {floor} has no facility schedule stage")
                    })?;
                let offset = floor - stage.min_floor;
                let cycle_index = usize::try_from(offset)
                    .map_err(|_| "Endless facility cycle index overflowed usize".to_string())?
                    % stage.cycle.len();
                let cycle = &stage.cycle[cycle_index];
                (&cycle.pool_id, &cycle.profile_id)
            }
        };

        Ok(ResolvedFacilitySchedule {
            pool: templates
                .pool(pool_id)
                .ok_or_else(|| format!("facility schedule references missing pool '{pool_id}'"))?,
            profile: self.profile(profile_id).ok_or_else(|| {
                format!("facility schedule references missing profile '{profile_id}'")
            })?,
        })
    }

    pub fn validate_contract(&self, templates: &FacilityTemplateDatabase) -> Result<(), String> {
        if self.profiles.is_empty() {
            return Err("node assignment profiles must not be empty".to_string());
        }
        let mut profile_ids = HashSet::new();
        for profile in &self.profiles {
            validate_assignment_profile(profile)?;
            if !profile_ids.insert(profile.id.as_str()) {
                return Err(format!(
                    "duplicate node assignment profile '{}'",
                    profile.id
                ));
            }
        }

        if self.standard_schedule.is_empty() {
            return Err("Standard facility schedule must not be empty".to_string());
        }
        let mut standard_floors = BTreeSet::new();
        for entry in &self.standard_schedule {
            if entry.floor == 0 || !standard_floors.insert(entry.floor) {
                return Err(format!(
                    "Standard facility schedule has invalid or duplicate floor {}",
                    entry.floor
                ));
            }
            validate_schedule_refs(templates, self, &entry.pool_id, &entry.profile_id)?;
        }
        let expected =
            (1..=*standard_floors.last().expect("non-empty schedule")).collect::<BTreeSet<_>>();
        if standard_floors != expected {
            return Err("Standard facility schedule floors must be contiguous from 1".to_string());
        }

        if self.endless_stages.is_empty() {
            return Err("Endless facility schedule stages must not be empty".to_string());
        }
        let mut expected_min = 1_u32;
        for (index, stage) in self.endless_stages.iter().enumerate() {
            if stage.min_floor != expected_min {
                return Err(format!(
                    "Endless facility stage {index} must start at floor {expected_min}"
                ));
            }
            if stage.cycle.is_empty() {
                return Err(format!(
                    "Endless facility stage {index} cycle must not be empty"
                ));
            }
            for entry in &stage.cycle {
                validate_schedule_refs(templates, self, &entry.pool_id, &entry.profile_id)?;
            }
            match stage.max_floor {
                Some(max_floor) if max_floor >= stage.min_floor => {
                    expected_min = max_floor.checked_add(1).ok_or_else(|| {
                        "Endless facility stage max_floor overflowed u32".to_string()
                    })?;
                }
                Some(_) => {
                    return Err(format!("Endless facility stage {index} has inverted range"))
                }
                None if index + 1 == self.endless_stages.len() => return Ok(()),
                None => {
                    return Err(
                        "only the final Endless facility stage may be open-ended".to_string()
                    )
                }
            }
        }
        Err("final Endless facility stage must be open-ended".to_string())
    }
}

fn validate_template(
    template: &FacilityTemplate,
    tag_catalog: &HashSet<&str>,
) -> Result<(), String> {
    if template.version != FACILITY_TEMPLATE_VERSION {
        return Err(format!(
            "facility template '{}' uses unsupported version {}",
            template.id.as_str(),
            template.version
        ));
    }
    validate_known_tags(
        &format!("facility template '{}'", template.id.as_str()),
        &template.tags,
        tag_catalog,
    )?;
    if template.slots.is_empty() {
        return Err(format!(
            "facility template '{}' has no slots",
            template.id.as_str()
        ));
    }

    let mut slots = HashMap::new();
    let mut start_count = 0;
    let mut terminal_count = 0;
    for slot in &template.slots {
        if slot.id.as_str().trim().is_empty() || slots.insert(slot.id.as_str(), slot).is_some() {
            return Err(format!(
                "facility template '{}' has invalid or duplicate slot '{}'",
                template.id.as_str(),
                slot.id.as_str()
            ));
        }
        if slot.allowed_categories.is_empty() {
            return Err(format!(
                "facility template '{}' slot '{}' has no allowed categories",
                template.id.as_str(),
                slot.id.as_str()
            ));
        }
        if slot
            .allowed_categories
            .iter()
            .copied()
            .collect::<HashSet<_>>()
            .len()
            != slot.allowed_categories.len()
        {
            return Err(format!(
                "facility template '{}' slot '{}' has duplicate allowed category",
                template.id.as_str(),
                slot.id.as_str()
            ));
        }
        validate_known_tags(
            &format!(
                "facility template '{}' slot '{}'",
                template.id.as_str(),
                slot.id.as_str()
            ),
            &slot.tags,
            tag_catalog,
        )?;
        match slot.role {
            FacilitySlotRole::Start => {
                start_count += 1;
                if slot.allowed_categories != [MapNodeCategory::Start] {
                    return Err(format!(
                        "facility template '{}' Start slot '{}' must allow only Start",
                        template.id.as_str(),
                        slot.id.as_str()
                    ));
                }
            }
            FacilitySlotRole::Entry => {
                start_count += 1;
                if slot.allowed_categories.len() != 1
                    || !matches!(
                        slot.allowed_categories[0],
                        MapNodeCategory::Combat | MapNodeCategory::Boss
                    )
                {
                    return Err(format!(
                        "facility template '{}' Entry slot '{}' must allow one Combat or Boss category",
                        template.id.as_str(),
                        slot.id.as_str()
                    ));
                }
            }
            FacilitySlotRole::Terminal(resolution) => {
                terminal_count += 1;
                if slot.allowed_categories != [resolution.required_category()] {
                    return Err(format!(
                        "facility template '{}' terminal slot '{}' must allow only {:?}",
                        template.id.as_str(),
                        slot.id.as_str(),
                        resolution.required_category()
                    ));
                }
            }
            FacilitySlotRole::Ordinary | FacilitySlotRole::ManifestedBonus => {
                if slot.allowed_categories.iter().any(|category| {
                    matches!(
                        category,
                        MapNodeCategory::Start | MapNodeCategory::Gate | MapNodeCategory::Boss
                    )
                }) {
                    return Err(format!(
                        "facility template '{}' ordinary slot '{}' allows reserved category",
                        template.id.as_str(),
                        slot.id.as_str()
                    ));
                }
            }
        }
    }
    if start_count == 0 || terminal_count == 0 {
        return Err(format!(
            "facility template '{}' requires at least one Start and terminal slot",
            template.id.as_str()
        ));
    }

    validate_connections(template, &slots)?;
    validate_manifestations(template, &slots)?;
    validate_directed_terminal_reachability(template, &slots)?;
    Ok(())
}

fn validate_connections(
    template: &FacilityTemplate,
    slots: &HashMap<&str, &FacilityRoomSlot>,
) -> Result<(), String> {
    let mut connections = HashSet::new();
    for connection in &template.connections {
        let from = connection.from_slot_id.as_str();
        let to = connection.to_slot_id.as_str();
        if from == to || !slots.contains_key(from) || !slots.contains_key(to) {
            return Err(format!(
                "facility template '{}' has invalid connection '{}' -> '{}'",
                template.id.as_str(),
                from,
                to
            ));
        }
        if !connections.insert((from, to)) {
            return Err(format!(
                "facility template '{}' has duplicate connection '{}' -> '{}'",
                template.id.as_str(),
                from,
                to
            ));
        }
        if let Some(reveal_slot_id) = &connection.reveal_with_slot_id {
            let Some(reveal_slot) = slots.get(reveal_slot_id.as_str()) else {
                return Err(format!(
                    "facility template '{}' connection reveal references missing slot '{}'",
                    template.id.as_str(),
                    reveal_slot_id.as_str()
                ));
            };
            let capstone_exit = template.kind == FacilityTemplateKind::Capstone
                && matches!(
                    reveal_slot.role,
                    FacilitySlotRole::Terminal(FacilityTerminalResolution::BossOmenExitGate)
                );
            if reveal_slot.role != FacilitySlotRole::ManifestedBonus && !capstone_exit {
                return Err(format!(
                    "facility template '{}' connection reveal slot '{}' is not manifested",
                    template.id.as_str(),
                    reveal_slot_id.as_str()
                ));
            }
        }
    }
    Ok(())
}

fn validate_manifestations(
    template: &FacilityTemplate,
    slots: &HashMap<&str, &FacilityRoomSlot>,
) -> Result<(), String> {
    let mut rule_ids = HashSet::new();
    let mut targets = HashSet::new();
    for rule in &template.manifestation_rules {
        if rule.id.trim().is_empty() || !rule_ids.insert(rule.id.as_str()) {
            return Err(format!(
                "facility template '{}' has invalid or duplicate manifestation rule '{}'",
                template.id.as_str(),
                rule.id
            ));
        }
        let Some(target) = slots.get(rule.manifest_slot_id.as_str()) else {
            return Err(format!(
                "facility template '{}' manifestation '{}' references missing target '{}'",
                template.id.as_str(),
                rule.id,
                rule.manifest_slot_id.as_str()
            ));
        };
        if target.role != FacilitySlotRole::ManifestedBonus
            || !targets.insert(rule.manifest_slot_id.as_str())
        {
            return Err(format!(
                "facility template '{}' manifestation target '{}' must be unique ManifestedBonus",
                template.id.as_str(),
                rule.manifest_slot_id.as_str()
            ));
        }
        if target.allowed_categories.iter().any(|category| {
            matches!(
                category,
                MapNodeCategory::Start
                    | MapNodeCategory::SavePoint
                    | MapNodeCategory::Boss
                    | MapNodeCategory::Gate
            )
        }) {
            return Err(format!(
                "facility template '{}' manifestation target '{}' allows mandatory category",
                template.id.as_str(),
                rule.manifest_slot_id.as_str()
            ));
        }
        if rule.prerequisite_slot_ids.is_empty() {
            return Err(format!(
                "facility template '{}' manifestation '{}' has no prerequisites",
                template.id.as_str(),
                rule.id
            ));
        }
        let mut prerequisites = HashSet::new();
        for prerequisite in &rule.prerequisite_slot_ids {
            let Some(slot) = slots.get(prerequisite.as_str()) else {
                return Err(format!(
                    "facility template '{}' manifestation '{}' references missing prerequisite '{}'",
                    template.id.as_str(),
                    rule.id,
                    prerequisite.as_str()
                ));
            };
            if slot.role == FacilitySlotRole::ManifestedBonus
                || prerequisite == &rule.manifest_slot_id
                || !prerequisites.insert(prerequisite.as_str())
            {
                return Err(format!(
                    "facility template '{}' manifestation '{}' has invalid prerequisite '{}'",
                    template.id.as_str(),
                    rule.id,
                    prerequisite.as_str()
                ));
            }
        }
    }
    if template
        .slots
        .iter()
        .filter(|slot| slot.role == FacilitySlotRole::ManifestedBonus)
        .any(|slot| !targets.contains(slot.id.as_str()))
    {
        return Err(format!(
            "facility template '{}' has manifested slot without one rule",
            template.id.as_str()
        ));
    }
    Ok(())
}

fn validate_directed_terminal_reachability(
    template: &FacilityTemplate,
    slots: &HashMap<&str, &FacilityRoomSlot>,
) -> Result<(), String> {
    let starts = template
        .slots
        .iter()
        .filter(|slot| matches!(slot.role, FacilitySlotRole::Start | FacilitySlotRole::Entry))
        .map(|slot| slot.id.as_str())
        .collect::<Vec<_>>();
    let terminals = template
        .slots
        .iter()
        .filter(|slot| matches!(slot.role, FacilitySlotRole::Terminal(_)))
        .map(|slot| slot.id.as_str())
        .collect::<HashSet<_>>();

    let adjacency = directed_adjacency(template);
    let reachable_from_start = traverse_many(&starts, &adjacency);
    for slot in &template.slots {
        if slot.role != FacilitySlotRole::ManifestedBonus
            && !reachable_from_start.contains(slot.id.as_str())
        {
            return Err(format!(
                "facility template '{}' slot '{}' is not reachable from Start",
                template.id.as_str(),
                slot.id.as_str()
            ));
        }
    }

    for slot in slots.values() {
        if matches!(slot.role, FacilitySlotRole::Terminal(_)) {
            continue;
        }
        let reachable = traverse_many(&[slot.id.as_str()], &adjacency);
        if reachable.is_disjoint(&terminals) {
            return Err(format!(
                "facility template '{}' slot '{}' cannot reach a terminal",
                template.id.as_str(),
                slot.id.as_str()
            ));
        }
    }
    Ok(())
}

fn directed_adjacency<'a>(template: &'a FacilityTemplate) -> HashMap<&'a str, Vec<&'a str>> {
    let mut adjacency = HashMap::<&str, Vec<&str>>::new();
    for connection in &template.connections {
        adjacency
            .entry(connection.from_slot_id.as_str())
            .or_default()
            .push(connection.to_slot_id.as_str());
        if connection.direction == MapEdgeDirection::Bidirectional {
            adjacency
                .entry(connection.to_slot_id.as_str())
                .or_default()
                .push(connection.from_slot_id.as_str());
        }
    }
    adjacency
}

fn traverse_many<'a>(
    starts: &[&'a str],
    adjacency: &HashMap<&'a str, Vec<&'a str>>,
) -> HashSet<&'a str> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::from(starts.to_vec());
    while let Some(slot_id) = queue.pop_front() {
        if !visited.insert(slot_id) {
            continue;
        }
        if let Some(next) = adjacency.get(slot_id) {
            queue.extend(next.iter().copied());
        }
    }
    visited
}

fn validate_assignment_profile(profile: &NodeAssignmentProfile) -> Result<(), String> {
    if profile.id.trim().is_empty() {
        return Err("node assignment profile id must not be empty".to_string());
    }
    if profile.difficulty_profile_id.trim().is_empty() {
        return Err(format!(
            "node assignment profile '{}' difficulty_profile_id must not be empty",
            profile.id
        ));
    }
    let mut constrained = HashSet::new();
    for constraint in &profile.category_constraints {
        constraint.bounds()?;
        if !constrained.insert(constraint.category) {
            return Err(format!(
                "node assignment profile '{}' duplicates {:?} constraint",
                profile.id, constraint.category
            ));
        }
    }
    let mut weighted = HashSet::new();
    for weight in &profile.remaining_category_weights {
        if weight.weight == 0 || !weighted.insert(weight.category) {
            return Err(format!(
                "node assignment profile '{}' has zero or duplicate {:?} weight",
                profile.id, weight.category
            ));
        }
        if matches!(
            weight.category,
            MapNodeCategory::Start | MapNodeCategory::Boss | MapNodeCategory::Gate
        ) {
            return Err(format!(
                "node assignment profile '{}' weights reserved category {:?}",
                profile.id, weight.category
            ));
        }
    }
    let mut tags = HashSet::new();
    for assignment in &profile.tagged_slot_assignments {
        if assignment.slot_tag.trim().is_empty() || !tags.insert(assignment.slot_tag.as_str()) {
            return Err(format!(
                "node assignment profile '{}' has invalid or duplicate tagged assignment '{}'",
                profile.id, assignment.slot_tag
            ));
        }
        if matches!(
            assignment.category,
            MapNodeCategory::Start | MapNodeCategory::Boss | MapNodeCategory::Gate
        ) {
            return Err(format!(
                "node assignment profile '{}' tagged assignment uses reserved category",
                profile.id
            ));
        }
    }
    Ok(())
}

fn validate_schedule_refs(
    templates: &FacilityTemplateDatabase,
    profiles: &NodeAssignmentProfileDatabase,
    pool_id: &str,
    profile_id: &str,
) -> Result<(), String> {
    if templates.pool(pool_id).is_none() {
        return Err(format!(
            "facility schedule references missing pool '{pool_id}'"
        ));
    }
    if profiles.profile(profile_id).is_none() {
        return Err(format!(
            "facility schedule references missing profile '{profile_id}'"
        ));
    }
    Ok(())
}

fn validate_non_empty_unique_strings<'a>(
    name: &str,
    values: &'a [String],
) -> Result<HashSet<&'a str>, String> {
    let mut seen = HashSet::new();
    for value in values {
        if value.trim().is_empty() || !seen.insert(value.as_str()) {
            return Err(format!(
                "{name} contains empty or duplicate value '{value}'"
            ));
        }
    }
    Ok(seen)
}

fn validate_known_tags(
    owner: &str,
    tags: &[String],
    tag_catalog: &HashSet<&str>,
) -> Result<(), String> {
    let mut seen = HashSet::new();
    for tag in tags {
        if !tag_catalog.contains(tag.as_str()) || !seen.insert(tag.as_str()) {
            return Err(format!("{owner} contains unknown or duplicate tag '{tag}'"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(id: &str, role: FacilitySlotRole, category: MapNodeCategory) -> FacilityRoomSlot {
        FacilityRoomSlot {
            id: MapSlotId::new(id),
            role,
            allowed_categories: vec![category],
            tags: vec![],
        }
    }

    fn minimal_template() -> FacilityTemplate {
        FacilityTemplate {
            id: MapTemplateId::new("minimal"),
            version: FACILITY_TEMPLATE_VERSION,
            kind: FacilityTemplateKind::Exploration,
            tags: vec![],
            slots: vec![
                slot("start", FacilitySlotRole::Start, MapNodeCategory::Start),
                slot("room", FacilitySlotRole::Ordinary, MapNodeCategory::Combat),
                slot(
                    "gate",
                    FacilitySlotRole::Terminal(FacilityTerminalResolution::FloorGate),
                    MapNodeCategory::Gate,
                ),
            ],
            connections: vec![
                FacilityConnection {
                    from_slot_id: MapSlotId::new("start"),
                    to_slot_id: MapSlotId::new("room"),
                    direction: MapEdgeDirection::Bidirectional,
                    reveal_with_slot_id: None,
                },
                FacilityConnection {
                    from_slot_id: MapSlotId::new("room"),
                    to_slot_id: MapSlotId::new("gate"),
                    direction: MapEdgeDirection::Bidirectional,
                    reveal_with_slot_id: None,
                },
            ],
            manifestation_rules: vec![],
        }
    }

    #[test]
    fn template_accepts_authored_semantic_slots_and_typed_terminal() {
        let database = FacilityTemplateDatabase {
            tag_catalog: vec![],
            templates: vec![minimal_template()],
            pools: vec![FacilityTemplatePool {
                id: "pool".to_string(),
                template_ids: vec![MapTemplateId::new("minimal")],
            }],
        };
        assert_eq!(database.validate_contract(), Ok(()));
    }

    #[test]
    fn directed_dead_end_before_terminal_is_rejected() {
        let mut template = minimal_template();
        template.connections[1] = FacilityConnection {
            from_slot_id: MapSlotId::new("gate"),
            to_slot_id: MapSlotId::new("room"),
            direction: MapEdgeDirection::ForwardOnly,
            reveal_with_slot_id: None,
        };
        let error = validate_template(&template, &HashSet::new()).unwrap_err();
        assert!(error.contains("not reachable from Start"));
    }

    #[test]
    fn manifested_mandatory_category_is_rejected() {
        let mut template = minimal_template();
        template.slots.push(slot(
            "secret",
            FacilitySlotRole::ManifestedBonus,
            MapNodeCategory::SavePoint,
        ));
        template
            .manifestation_rules
            .push(FacilityManifestationRule {
                id: "secret".to_string(),
                prerequisite_slot_ids: vec![MapSlotId::new("room")],
                manifest_slot_id: MapSlotId::new("secret"),
            });
        let error = validate_template(&template, &HashSet::new()).unwrap_err();
        assert!(error.contains("mandatory category"));
    }
}
