use std::collections::{BTreeSet, VecDeque};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use super::{FacilityLayout, MapSlotId, MapEdgeDirection, MapNodePayload};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldenBoughLocation { pub room_id: String, pub slot_id: MapSlotId }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldenBoughPlan {
    pub id: String,
    pub weight: u32,
    pub locations: Vec<GoldenBoughLocation>,
    pub deterrence_refill: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldenBoughState {
    pub plan_id: String,
    pub locations: Vec<GoldenBoughLocation>,
    pub collected: BTreeSet<GoldenBoughLocation>,
    pub deterrence_refill: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenBoughMarkerDto {
    pub room_id: String,
    pub slot_id: MapSlotId,
    pub collected: bool,
    pub reachable: bool,
}
crate::authoring::clone_shape::struct_shape!(GoldenBoughLocation { room_id,slot_id });
crate::authoring::clone_shape::struct_shape!(GoldenBoughState { plan_id,locations,collected,deterrence_refill });
impl FacilityLayout {
    /// Uses the same unconditional supported room/edge constraints as room movement.
    /// Forward-only descent deliberately excludes rooms abandoned on another branch.
    pub fn reachable_bough_rooms(&self, from: &str) -> BTreeSet<String> {
        let mut reached = BTreeSet::from([from.to_owned()]);
        let mut queue = VecDeque::from([from.to_owned()]);
        while let Some(id) = queue.pop_front() {
            for edge in &self.connections {
                if edge.condition.is_some() { continue; }
                let next = if edge.from == id { Some(&edge.to) }
                    else if edge.to == id && edge.direction == MapEdgeDirection::Bidirectional { Some(&edge.from) }
                    else { None };
                let Some(next) = next else { continue; };
                let Some(room) = self.room(next) else { continue; };
                if room.depth == 0 || room.access_condition.is_some() || !room.support.is_supported() { continue; }
                if reached.insert(next.clone()) { queue.push_back(next.clone()); }
            }
        }
        reached
    }
    pub fn validate_golden_bough_plans(&self) -> Result<(),String> {
        let fail = |reason: &str| format!("facility '{}' golden bough plans: {reason}",self.id);
        let candidates:Vec<_> = self.rooms.iter().flat_map(|room| room.reservations.iter()
            .filter(|slot| self.activity(&slot.activity_id).is_some_and(|a|a.golden_bough_candidate))
            .map(move |slot|(room,slot))).collect();
        if candidates.is_empty() {
            return if self.golden_bough_plans.is_empty() { Ok(()) } else { Err(fail("no eligible fixed candidates")) };
        }
        if self.golden_bough_plans.len()<2 { return Err(fail("multiple authored plans required")); }
        let reachable = self.reachable_bough_rooms(&self.initial_room_id);
        let wings:BTreeSet<_> = candidates.iter().filter_map(|(r,_)|r.wing.as_deref()).collect();
        for (room,slot) in &candidates {
            let activity=self.activity(&slot.activity_id).expect("candidate exists");
            if room.final_room || !reachable.contains(&room.id) || activity.condition.is_some()
                || !matches!(activity.payload, MapNodePayload::Encounter{..}) {
                return Err(fail("candidate must be a reachable unconditional fixed boss"));
            }
        }
        let mut ids=BTreeSet::new();
        for plan in &self.golden_bough_plans {
            if plan.id.trim().is_empty() || !ids.insert(&plan.id) || plan.weight==0
                || plan.deterrence_refill==0 || plan.locations.is_empty() { return Err(fail("invalid plan identity/weight/refill")); }
            let mut unique=BTreeSet::new();
            let mut covered=BTreeSet::new();
            for location in &plan.locations {
                let Some((room,_))=candidates.iter().find(|(room,slot)|room.id==location.room_id && slot.slot_id==location.slot_id) else {
                    return Err(fail("plan references a noncandidate location"));
                };
                if !unique.insert(location) {return Err(fail("duplicate plan location"));}
                if let Some(wing)=room.wing.as_deref() {covered.insert(wing);}
            }
            if covered!=wings {return Err(fail("plan omits a major descent wing"));}
        }
        Ok(())
    }
    pub(crate) fn choose_golden_bough_plan(&self, seed:u64) -> Result<Option<GoldenBoughState>,String> {
        self.validate_golden_bough_plans()?;
        if self.golden_bough_plans.is_empty() {return Ok(None);}
        let total=self.golden_bough_plans.iter().try_fold(0u64,|sum,p|sum.checked_add(u64::from(p.weight)))
            .ok_or("golden bough weight overflow")?;
        let mut rng=rand::rngs::StdRng::seed_from_u64(seed ^ 0x474F_4C44_504C_414E);
        let mut roll=rng.gen_range(0..total);
        for plan in &self.golden_bough_plans {
            if roll<u64::from(plan.weight) {return Ok(Some(GoldenBoughState {
                plan_id:plan.id.clone(),locations:plan.locations.clone(),collected:Default::default(),
                deterrence_refill:plan.deterrence_refill,
            }));}
            roll-=u64::from(plan.weight);
        }
        Err("no eligible golden bough plan".into())
    }
}
