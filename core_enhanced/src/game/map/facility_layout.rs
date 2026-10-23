//! Authored whole-facility topology and expedition-local room state.
//! Display coordinates never participate in navigation or random seeds.
use std::collections::{BTreeMap, HashSet, VecDeque};
use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use super::{FacilityTemplateDatabase, FacilitySlotRole, MapEdgeDirection, MapNodeId,
    MapNodeKindId, MapNodePayload, MapSlotId, MapTemplateId, MapProgression, RunMap,
    GoldenBoughPlan, GoldenBoughState, GoldenBoughMarkerDto};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityLayout {
    pub id: String,
    pub expedition_default: bool,
    pub entrance_room_id: String,
    pub initial_room_id: String,
    pub difficulty_profile_id: String,
    pub rooms: Vec<FacilityRoom>,
    pub connections: Vec<FacilityRoomConnection>,
    pub activities: Vec<FacilityActivityDefinition>,
    pub allocation: FacilityAllocationPolicy,
    #[serde(default)]
    pub golden_bough_plans: Vec<GoldenBoughPlan>,
    #[serde(default)]
    pub relic_rewards: crate::game::relic_reward::BasicRelicRewardPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dealer: Option<crate::game::dealer::DealerPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stress_combat: Option<crate::game::stress_combat::StressCombatPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityRoom {
    pub id: String,
    pub depth: u32,
    pub wing: Option<String>,
    pub name: String,
    pub concept: String,
    pub display: FacilityDisplay,
    pub template_id: Option<MapTemplateId>,
    pub final_room: bool,
    pub support: FacilityContentSupport,
    pub access_condition: Option<String>,
    #[serde(default)]
    pub exclusive_approach_room_id: Option<String>,
    pub reservations: Vec<FacilityReservation>,
    pub omen_slot: Option<MapSlotId>,
    pub elite_slots: Vec<MapSlotId>,
    pub safe_access_slots: Vec<MapSlotId>,
    pub slot_display: BTreeMap<MapSlotId, FacilityDisplay>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityDisplay {
    pub x: i32,
    pub y: i32,
    pub asset_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FacilityContentSupport {
    Supported,
    Unsupported { reason: String },
}
impl FacilityContentSupport {
    pub fn is_supported(&self) -> bool { matches!(self, Self::Supported) }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityRoomConnection {
    pub from: String,
    pub to: String,
    pub direction: MapEdgeDirection,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityReservation {
    pub slot_id: MapSlotId,
    pub activity_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityActivityDefinition {
    pub id: String,
    /// Overrides the node-kind cost for this authored activity, including zero.
    #[serde(default)]
    pub first_entry_cost: Option<u32>,
    #[serde(default)]
    pub golden_bough_candidate: bool,
    pub label: String,
    pub kind_id: MapNodeKindId,
    pub payload: MapNodePayload,
    pub support: FacilityContentSupport,
    pub condition: Option<String>,
    pub reward_activity: Option<RewardActivityKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewardActivityKind { SimpleChoice, ResourceInvestment, Gambling, RelicGacha, StressCombat }
impl RewardActivityKind {
    pub const ALL: [Self; 5] = [Self::SimpleChoice, Self::ResourceInvestment, Self::Gambling, Self::RelicGacha, Self::StressCombat];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityAllocationPolicy {
    pub support_counts: BTreeMap<u32, u32>,
    pub one_reserved_support_counts: BTreeMap<u32, u32>,
    pub support_activities: BTreeMap<String, u32>,
    pub sizes: Vec<FacilitySizeAllocation>,
    pub normal_activity_id: String,
    pub elite_activity_id: String,
    pub event_activity_id: String,
    pub reward_activity_id: String,
    pub gate_activity_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilitySizeAllocation {
    pub size: u32,
    pub event_counts: BTreeMap<u32, u32>,
    pub reward_counts: BTreeMap<u32, u32>,
    pub elite_percent_by_depth: BTreeMap<u32, u32>,
}

/// The current room's map remains owned by RunState. Only other rooms are stored here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityExplorationState {
    pub layout_id: String,
    pub current_room_id: String,
    pub visited_rooms: BTreeMap<String, FacilityVisitedRoom>,
    pub activity_metadata: BTreeMap<MapNodeId, FacilityNodeMetadata>,
    pub golden_boughs: Option<GoldenBoughState>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityVisitedRoom {
    pub map: RunMap,
    pub progression: MapProgression,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityNodeMetadata {
    pub activity_id: String,
    pub reward_activity: Option<RewardActivityKind>,
    pub omen_compatible: bool,
}

impl FacilityLayout {
    pub fn room(&self, id: &str) -> Option<&FacilityRoom> { self.rooms.iter().find(|r| r.id == id) }
    pub fn activity(&self, id: &str) -> Option<&FacilityActivityDefinition> { self.activities.iter().find(|a| a.id == id) }
    pub fn connection(&self, from: &str, to: &str) -> Option<&FacilityRoomConnection> {
        self.connections.iter().find(|e| e.from == from && e.to == to || e.direction == MapEdgeDirection::Bidirectional && e.to == from && e.from == to)
    }
    pub fn validate(&self, templates: &FacilityTemplateDatabase) -> Result<(), String> {
        let fail = |s: &str| format!("facility layout '{}': {s}", self.id);
        if self.id.trim().is_empty() || self.rooms.is_empty() { return Err(fail("empty layout identity/rooms")); }
        let mut ids = HashSet::new();
        let mut activities = HashSet::new();
        for a in &self.activities {
            if a.id.trim().is_empty() || !activities.insert(&a.id) { return Err(fail("empty/duplicate activity identity")); }
            validate_support(&a.support)?;
            if a.condition.as_ref().is_some_and(|s| s.trim().is_empty()) { return Err(fail("empty activity condition")); }
        }
        let pol = &self.allocation;
        for id in [&pol.normal_activity_id, &pol.elite_activity_id, &pol.event_activity_id, &pol.reward_activity_id, &pol.gate_activity_id].into_iter().chain(pol.support_activities.keys()) {
            if self.activity(id).is_none() { return Err(fail(&format!("unknown allocation activity '{id}'"))); }
        }
        validate_weights(&pol.support_counts, 2)?;
        validate_weights(&pol.one_reserved_support_counts, 2)?;
        if pol.support_counts.keys().any(|n| *n > 2) || pol.one_reserved_support_counts.keys().any(|n| !(1..=2).contains(n)) { return Err(fail("invalid support total")); }
        if pol.support_activities.is_empty() || pol.support_activities.values().any(|w| *w == 0 || *w > 100) { return Err(fail("invalid support activity weights")); }
        let mut sizes = HashSet::new();
        for s in &pol.sizes {
            if s.size == 0 || !sizes.insert(s.size) { return Err(fail("empty/duplicate allocation size")); }
            validate_weights(&s.event_counts, s.size)?; validate_weights(&s.reward_counts, s.size)?;
            if s.elite_percent_by_depth.values().any(|v| *v > 100) { return Err(fail("invalid elite percentage")); }
        }
        for room in &self.rooms {
            if room.id.trim().is_empty() || !ids.insert(&room.id) { return Err(fail("empty/duplicate room identity")); }
            validate_support(&room.support)?;
            if room.depth == 0 {
                if room.id != self.entrance_room_id || room.template_id.is_some() || !room.reservations.is_empty() { return Err(fail("invalid entrance")); }
                continue;
            }
            let template = room.template_id.as_ref().and_then(|id| templates.template(id)).ok_or_else(|| fail(&format!("room '{}' missing template", room.id)))?;
            if template.slots.iter().any(|s| !matches!(s.role,FacilitySlotRole::Start|FacilitySlotRole::Ordinary|FacilitySlotRole::Terminal(super::FacilityTerminalResolution::FloorGate))) {
                return Err(fail("room template needs Start, activity slots and FloorGate terminal"));
            }
            let slots: HashSet<_> = template.slots.iter().filter(|s| matches!(s.role, FacilitySlotRole::Ordinary)).map(|s| &s.id).collect();
            let mut reserved = HashSet::new();
            for r in &room.reservations {
                if !slots.contains(&r.slot_id) || !reserved.insert(&r.slot_id) || self.activity(&r.activity_id).is_none() { return Err(fail(&format!("room '{}' invalid reservation", room.id))); }
            }
            let mut elites = HashSet::new();
            for slot in &room.elite_slots {
                if !slots.contains(slot) || reserved.contains(slot) || room.omen_slot.as_ref() == Some(slot) || !elites.insert(slot) { return Err(fail("invalid/duplicate elite candidate")); }
            }
            if (3..=7).contains(&room.depth) && !room.final_room && room.omen_slot.is_none() { return Err(fail("missing omen-compatible slot")); }
            if let Some(omen) = &room.omen_slot {
                if !(3..=7).contains(&room.depth) || room.final_room || !slots.contains(omen) || reserved.contains(omen) { return Err(fail("invalid omen-compatible slot")); }
            }
            if room.safe_access_slots.iter().any(|s| !slots.contains(s) || elites.contains(s)) { return Err(fail("invalid safe access/elite overlap")); }
            if room.slot_display.keys().any(|s| !template.slots.iter().any(|t| t.id == *s)) { return Err(fail("unknown displayed slot")); }
            // Fixed encounters must be optional: all other activities and the Gate
            // remain reachable when those encounters are excluded.
            let blocked: HashSet<_> = room.reservations.iter().filter(|r| self.activity(&r.activity_id).is_some_and(|a| matches!(a.payload, MapNodePayload::Encounter { .. }))).map(|r| r.slot_id.clone()).collect();
            if !room.final_room && !blocked.is_empty() {
                let reached = reachable_slots(template, &blocked);
                if template.slots.iter().any(|s| !blocked.contains(&s.id) && !reached.contains(&s.id)) { return Err(fail("fixed encounter blocks Gate or another activity")); }
            }
            if !room.final_room {
                let size = u32::try_from(slots.len()).map_err(|_| fail("room size overflow"))?;
                let settings = pol.sizes.iter().find(|s| s.size == size).ok_or_else(|| fail("room lacks allocation size"))?;
                let fixed_support = self.reserved_support_count(room);
                if fixed_support > 2 { return Err(fail("reserved support exceeds two")); }
                for (count, _) in self.support_distribution(fixed_support) {
                    if self.count_combinations(room, settings, count)?.is_empty() { return Err(fail(&format!("room '{}' cannot realize support total {count}", room.id))); }
                }
            } else if room.reservations.len() != slots.len() { return Err(fail("final room must author every activity")); }
        }
        if self.room(&self.entrance_room_id).is_none_or(|r| r.depth != 0) || self.room(&self.initial_room_id).is_none_or(|r| r.depth != 1 || !r.support.is_supported()) { return Err(fail("invalid initial room")); }
        let mut edges = HashSet::new();
        for e in &self.connections {
            let from = self.room(&e.from).ok_or_else(|| fail("missing edge source"))?;
            let to = self.room(&e.to).ok_or_else(|| fail("missing edge target"))?;
            let key = if e.direction == MapEdgeDirection::Bidirectional && e.from > e.to { (&e.to, &e.from) } else { (&e.from, &e.to) };
            if e.from == e.to || !edges.insert(key) { return Err(fail("self/duplicate edge")); }
            if from.depth == to.depth {
                if e.direction != MapEdgeDirection::Bidirectional { return Err(fail("same-depth edge must be bidirectional")); }
                if from.depth >= 4 && from.wing != to.wing { return Err(fail("deep cross-wing edge")); }
            } else if Some(to.depth) != from.depth.checked_add(1) || e.direction != MapEdgeDirection::ForwardOnly {
                return Err(fail("floor edge must descend exactly one depth"));
            }
            if to.final_room && to.depth == 8 && (from.depth != 7 || from.wing != to.wing) { return Err(fail("invalid final-floor approach")); }
            if from.final_room && from.depth < 8 || to.final_room && to.depth < 8 {
                let (hidden, other) = if from.final_room { (from, to) } else { (to, from) };
                if hidden.exclusive_approach_room_id.as_deref() != Some(other.id.as_str()) || hidden.access_condition.is_none() || other.final_room || hidden.wing != other.wing || hidden.depth != other.depth { return Err(fail("invalid hidden final approach")); }
            }
        }
        if self.connection(&self.entrance_room_id, &self.initial_room_id).is_none() { return Err(fail("entrance is not connected to initial room")); }
        for r in self.rooms.iter().filter(|r| r.depth == 7 && !r.final_room) {
            if !self.connections.iter().any(|e| e.from == r.id && self.room(&e.to).is_some_and(|t| t.depth == 8 && t.wing == r.wing && t.final_room)) { return Err(fail("seventh-floor room lacks its final-floor connection")); }
        }
        let mut reached = HashSet::from([self.entrance_room_id.as_str()]);
        let mut pending = VecDeque::from([self.entrance_room_id.as_str()]);
        while let Some(id) = pending.pop_front() {
            for e in &self.connections {
                let next = if e.from == id { Some(e.to.as_str()) } else if e.to == id && e.direction == MapEdgeDirection::Bidirectional { Some(e.from.as_str()) } else { None };
                if let Some(next) = next { if reached.insert(next) { pending.push_back(next); } }
            }
        }
        if reached.len() != self.rooms.len() { return Err(fail("unreachable room")); }
        self.validate_golden_bough_plans()?;
        Ok(())
    }
    pub fn reserved_support_count(&self, room: &FacilityRoom) -> u32 {
        room.reservations.iter().filter(|r| self.activity(&r.activity_id).is_some_and(|a| matches!(a.payload, MapNodePayload::Maintenance | MapNodePayload::Shop {..} | MapNodePayload::HeadquartersContact {..}))).count() as u32
    }
    fn support_distribution(&self, reserved: u32) -> BTreeMap<u32, u32> {
        match reserved { 0 => self.allocation.support_counts.clone(), 1 => self.allocation.one_reserved_support_counts.clone(), _ => BTreeMap::from([(2, 1)]) }
    }
    fn count_combinations(&self, room: &FacilityRoom, size: &FacilitySizeAllocation, support: u32) -> Result<Vec<((u32,u32,u32),u64)>,String> {
        let fixed_support = self.reserved_support_count(room);
        let reserved = u32::try_from(room.reservations.len()).map_err(|_| "too many reservations")?;
        let minimum_normal = if room.depth >= 3 { 3 } else { 2 };
        let elite_percent = *size.elite_percent_by_depth.get(&room.depth).ok_or("missing depth elite percentage")?;
        if room.depth == 1 && elite_percent != 0 { return Err("first depth cannot have elite encounters".into()); }
        let mut result = vec![];
        for (&events, &ew) in &size.event_counts {
            for (&rewards, &rw) in &size.reward_counts {
                for (elite, lw) in [(0,100-elite_percent),(1,elite_percent)] {
                    if lw == 0 || elite == 1 && room.elite_slots.is_empty() { continue; }
                    let used = reserved + support.saturating_sub(fixed_support) + events + rewards + elite;
                    if used + minimum_normal <= size.size {
                        result.push(((events,rewards,elite),u64::from(ew)*u64::from(rw)*u64::from(lw)));
                    }
                }
            }
        }
        Ok(result)
    }
    /// Support totals are sampled first. Conditioning other counts never changes
    /// that marginal distribution, including rooms with fixed HQ/maintenance.
    pub(crate) fn sample_activities(&self, room: &FacilityRoom, template: &super::FacilityTemplate, rng: &mut impl Rng) -> Result<BTreeMap<MapSlotId,FacilityNodeMetadata>,String> {
        let mut result = BTreeMap::new();
        for r in &room.reservations {
            let selected = if room.final_room && self.allocation.support_activities.contains_key(&r.activity_id) {
                weighted(self.allocation.support_activities.iter().map(|(id,w)|(id.clone(),u64::from(*w))).collect(),rng)?
            } else {r.activity_id.clone()};
            let a = self.activity(&selected).ok_or("unknown reserved activity")?;
            result.insert(r.slot_id.clone(), FacilityNodeMetadata { activity_id: a.id.clone(), reward_activity: a.reward_activity, omen_compatible: false });
        }
        if !room.final_room {
            let mut free: Vec<_> = template.slots.iter().filter(|s| matches!(s.role,FacilitySlotRole::Ordinary) && !result.contains_key(&s.id)).map(|s| s.id.clone()).collect();
            let size = (free.len()+result.len()) as u32;
            let settings = self.allocation.sizes.iter().find(|s| s.size == size).ok_or("missing size allocation")?;
            let fixed_support = self.reserved_support_count(room);
            let support = weighted(self.support_distribution(fixed_support).into_iter().map(|(n,w)|(n,u64::from(w))).collect(),rng)?;
            let (events,rewards,elites) = weighted(self.count_combinations(room,settings,support)?,rng)?;
            // The compatible slot stays ordinary until the U20 producer chooses it.
            if let Some(slot) = &room.omen_slot {
                free.retain(|s| s != slot);
                result.insert(slot.clone(),FacilityNodeMetadata {activity_id:self.allocation.normal_activity_id.clone(),reward_activity:None,omen_compatible:true});
            }
            if elites > 0 {
                let slot = room.elite_slots.choose(rng).ok_or("no elite candidate")?.clone();
                free.retain(|s| *s != slot);
                result.insert(slot,FacilityNodeMetadata{activity_id:self.allocation.elite_activity_id.clone(),reward_activity:None,omen_compatible:false});
            }
            free.shuffle(rng);
            let mut selected = Vec::new();
            for _ in fixed_support..support {
                selected.push((weighted(self.allocation.support_activities.iter().map(|(id,w)|(id.clone(),u64::from(*w))).collect(),rng)?,None));
            }
            for _ in 0..events {selected.push((self.allocation.event_activity_id.clone(),None));}
            for _ in 0..rewards {selected.push((self.allocation.reward_activity_id.clone(),Some(RewardActivityKind::ALL[rng.gen_range(0..5)])));}
            if selected.len() > free.len() { return Err("allocation exceeds eligible positions".into()); }
            for slot in free {
                let (activity_id,reward_activity) = selected.pop().unwrap_or_else(|| (self.allocation.normal_activity_id.clone(),None));
                result.insert(slot,FacilityNodeMetadata{activity_id,reward_activity,omen_compatible:false});
            }
        }
        for s in &template.slots {
            if matches!(s.role,FacilitySlotRole::Terminal(_)) {
                result.insert(s.id.clone(),FacilityNodeMetadata{activity_id:self.allocation.gate_activity_id.clone(),reward_activity:None,omen_compatible:false});
            }
        }
        Ok(result)
    }
}
fn validate_support(s: &FacilityContentSupport) -> Result<(),String> {
    if matches!(s,FacilityContentSupport::Unsupported{reason} if reason.trim().is_empty()) { Err("unsupported content needs a reason".into()) } else {Ok(())}
}
fn validate_weights(weights: &BTreeMap<u32,u32>, max: u32) -> Result<(),String> {
    if weights.is_empty() || weights.iter().any(|(n,w)| *n > max || *w == 0 || *w > 100) { Err("invalid allocation weights/counts".into()) } else {Ok(())}
}
fn weighted<T>(values: Vec<(T,u64)>, rng: &mut impl Rng) -> Result<T,String> {
    let total = values.iter().try_fold(0u64,|s,(_,w)|s.checked_add(*w)).ok_or("allocation weight overflow")?;
    if total == 0 {return Err("no feasible allocation".into());}
    let mut choice = rng.gen_range(0..total);
    for (v,w) in values {if choice < w {return Ok(v);} choice -= w;}
    unreachable!("choice is less than total")
}
fn reachable_slots(template: &super::FacilityTemplate, blocked: &HashSet<MapSlotId>) -> HashSet<MapSlotId> {
    let mut reached: HashSet<_> = template.slots.iter().filter(|s| s.role == FacilitySlotRole::Start).map(|s|s.id.clone()).collect();
    let mut queue: VecDeque<_> = reached.iter().cloned().collect();
    while let Some(id) = queue.pop_front() {
        for e in &template.connections {
            let next = if e.from_slot_id == id {Some(&e.to_slot_id)} else if e.to_slot_id == id && e.direction == MapEdgeDirection::Bidirectional {Some(&e.from_slot_id)} else {None};
            if let Some(next) = next {if !blocked.contains(next) && reached.insert(next.clone()) {queue.push_back(next.clone());}}
        }
    }
    reached
}

crate::authoring::clone_shape::struct_shape!(FacilityExplorationState { layout_id,current_room_id,visited_rooms,activity_metadata,golden_boughs });
crate::authoring::clone_shape::struct_shape!(FacilityVisitedRoom { map,progression });
crate::authoring::clone_shape::struct_shape!(FacilityNodeMetadata { activity_id,reward_activity,omen_compatible });


#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacilityKnowledge {
    pub completed_rooms: std::collections::BTreeSet<String>,
    pub discovered_final_rooms: std::collections::BTreeSet<String>,
}
crate::authoring::clone_shape::struct_shape!(FacilityKnowledge {completed_rooms,discovered_final_rooms});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacilityViewDto {
    pub layout_id: String,
    pub current_room_id: String,
    pub rooms: Vec<FacilityRoomViewDto>,
    pub connections: Vec<FacilityRoomConnection>,
    pub gate_reached: bool,
    pub golden_boughs: Vec<GoldenBoughMarkerDto>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacilityRoomViewDto {
    pub id: String,
    pub depth: u32,
    pub wing: Option<String>,
    pub display: FacilityDisplay,
    pub name: Option<String>,
    pub concept: Option<String>,
    pub explored: bool,
    pub visited_this_expedition: bool,
    pub descent_recovery: Option<crate::game::behavior::TraumaRecoveryPreviewDto>,
    pub support: Option<FacilityContentSupport>,
    pub access_condition: Option<String>,
    pub interior: Option<super::FacilityTemplate>,
    pub current_expedition_map: Option<super::MapViewDto>,
    pub activities: Vec<FacilityActivityViewDto>,
    pub known_reservations: Vec<FacilityReservationViewDto>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacilityReservationViewDto {
    pub slot_id: MapSlotId,
    pub label: String,
    pub kind_id: MapNodeKindId,
    pub reward_activity: Option<RewardActivityKind>,
    pub support: FacilityContentSupport,
    pub condition: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacilityActivityViewDto {
    pub node_id: MapNodeId,
    pub entry_preview: super::NodeEntryPreviewDto,
    pub display: Option<FacilityDisplay>,
    pub label: Option<String>,
    pub reward_activity: Option<RewardActivityKind>,
    pub relic_reward: Option<crate::game::relic_reward::RelicRewardEntryView>,
    pub condition: Option<String>,
    pub support: FacilityContentSupport,
    pub first_entry_cost: u32,
    pub exhausts_deterrence: bool,
    pub deterrence_already_exhausted: bool,
}
impl super::FacilityRunState {
    pub(crate) fn node<'a>(&'a self, current: &'a RunMap, id: MapNodeId) -> Option<&'a super::MapNode> {
        current.node(id).or_else(|| self.rooms.as_ref()?.visited_rooms.values().find_map(|r|r.map.node(id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng,rngs::StdRng};
    fn catalogs() -> crate::game::data::live_ron::GameplayMapCatalogs {crate::game::data::live_ron::embedded_map_catalogs_for_fixtures()}
    #[test]
    fn u13_official_graph_counts_and_authored_errors() {
        let catalogs = catalogs();
        let db = &catalogs.facility_templates;
        let layout = &db.layouts[0];
        assert_eq!(layout.rooms.len(),89);
        assert_eq!(layout.rooms.iter().filter(|r|r.final_room).count(),4);
        let activity_count:usize = layout.rooms.iter().filter_map(|r|r.template_id.as_ref()).map(|id|db.template(id).unwrap().slots.iter().filter(|s|s.role == FacilitySlotRole::Ordinary).count()).sum();
        assert_eq!(activity_count,846);
        assert!(layout.connection("A6-05","A6-H").is_some());
        assert!(layout.connection("A6-04","A6-H").is_none());
        let mut wrong = layout.clone();
        wrong.connections.push(FacilityRoomConnection{from:"A4-01".into(),to:"B4-01".into(),direction:MapEdgeDirection::Bidirectional,condition:None});
        assert!(wrong.validate(db).unwrap_err().contains("deep cross-wing"));
        let mut wrong = layout.clone();wrong.rooms[1].id = wrong.rooms[2].id.clone();
        assert!(wrong.validate(db).is_err());
        let mut wrong = layout.clone();wrong.rooms.iter_mut().find(|r|r.id=="A3-01").unwrap().omen_slot=Some(MapSlotId::new("missing"));
        assert!(wrong.validate(db).unwrap_err().contains("omen-compatible"));
        let mut wrong=layout.clone();let e=wrong.connections.iter_mut().find(|e|e.from=="A6-05" && e.to=="A6-H").unwrap();e.from="A6-04".into();
        assert!(wrong.validate(db).unwrap_err().contains("hidden final approach"));
    }
    #[test]
    fn u13_assignment_counts_reservations_seed_and_coordinate_independence() {
        let catalogs=catalogs();let db=&catalogs.facility_templates;let layout=&db.layouts[0];
        for room in layout.rooms.iter().filter(|r|r.depth>0 && !r.final_room) {
            let template=db.template(room.template_id.as_ref().unwrap()).unwrap();
            for seed in 0..64 {
                let first=layout.sample_activities(room,template,&mut StdRng::seed_from_u64(seed)).unwrap();
                let mut moved=room.clone();moved.display.x+=777;moved.display.y-=111;moved.slot_display.clear();
                let second=layout.sample_activities(&moved,template,&mut StdRng::seed_from_u64(seed)).unwrap();
                assert_eq!(first,second);
                let ordinary=first.values().filter(|m|m.activity_id=="ordinary").count();
                assert!(ordinary >= if room.depth<=2 {2}else{3},"{} seed{seed}",room.id);
                let support=first.values().filter(|m|matches!(m.activity_id.as_str(),"maintenance"|"hq"|"shop")).count();
                assert!(support<=2);
                for reservation in &room.reservations {assert_eq!(first[&reservation.slot_id].activity_id,reservation.activity_id);}
                let elites:Vec<_>=first.iter().filter(|(_,m)|m.activity_id=="elite").collect();
                assert!(elites.len()<=usize::from(room.depth>1));
                for (slot,_) in elites {assert!(room.elite_slots.contains(slot));assert!(!room.safe_access_slots.contains(slot));}
                if let Some(slot)=&room.omen_slot {assert_eq!(first[slot].activity_id,"ordinary");assert!(first[slot].omen_compatible);}
            }
        }
    }
    #[test]
    fn u13_support_marginal_and_five_reward_types_are_not_balanced_bags() {
        let catalogs=catalogs();let db=&catalogs.facility_templates;let layout=&db.layouts[0];
        let room=layout.room("A1-01").unwrap();let template=db.template(room.template_id.as_ref().unwrap()).unwrap();
        let mut support=[0usize;3];let mut rewards=[0usize;5];
        for seed in 0..5000 {
            let sampled=layout.sample_activities(room,template,&mut StdRng::seed_from_u64(seed)).unwrap();
            support[sampled.values().filter(|m|matches!(m.activity_id.as_str(),"maintenance"|"hq"|"shop")).count()]+=1;
            for m in sampled.values() {if let Some(kind)=m.reward_activity {rewards[RewardActivityKind::ALL.iter().position(|k|*k==kind).unwrap()]+=1;}}
        }
        // Independent policy frequencies, with a fixed, generous finite-sample margin.
        for (actual,expected) in support.into_iter().zip([2000usize,2250,750]) {assert!(actual.abs_diff(expected)<150,"{actual} versus{expected}");}
        let count:usize=rewards.iter().sum();assert!(count>1000);
        for actual in rewards {assert!(actual.abs_diff(count/5)<count/15);}
        let early=layout.room("B1-01").unwrap();let template=db.template(early.template_id.as_ref().unwrap()).unwrap();
        let sampled=layout.sample_activities(early,template,&mut StdRng::seed_from_u64(99)).unwrap();
        assert_eq!(sampled[&MapSlotId::new("2")].reward_activity,Some(RewardActivityKind::SimpleChoice));
    }
}
