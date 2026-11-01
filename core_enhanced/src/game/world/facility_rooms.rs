use std::collections::{HashSet,VecDeque};
use super::GameCore;
use crate::game::{behavior::{BehaviorResult,GameError},resources::GameState,map::*};

impl GameCore {
    pub(super) fn sealed_golden_bough(&self, id:MapNodeId) -> Option<GoldenBoughLocation> {
        let run=self.state.run.as_ref()?;
        let state=run.facility_run_state.rooms.as_ref()?;
        let node=run.map.node(id)?;
        self.sealed_golden_bough_in_room(&state.current_room_id, node)
    }
    pub(super) fn sealed_golden_bough_in_room(&self, room_id: &str, node: &MapNode) -> Option<GoldenBoughLocation> {
        let gold=self.state.run.as_ref()?.facility_run_state.rooms.as_ref()?.golden_boughs.as_ref()?;
        // Room and slot identify the original seal; reanimated encounters never qualify.
        if node.combat_difficulty.as_ref().is_some_and(|d|d.revival.is_some()) {return None;}
        gold.locations.iter().find(|location|location.room_id==room_id
            && location.slot_id==node.slot_id && !gold.collected.contains(*location)).cloned()
    }
    pub(super) fn commit_golden_bough_victory(&mut self, id:MapNodeId) -> Result<(),GameError> {
        let Some(location)=self.sealed_golden_bough(id) else {return Ok(());};
        let run=self.run_state_mut()?;
        let gold=run.facility_run_state.rooms.as_mut().and_then(|s|s.golden_boughs.as_mut()).ok_or(GameError::InvalidAction)?;
        let next=run.run_progression.deterrence.checked_add(gold.deterrence_refill)
            .ok_or_else(||GameError::InvalidStaticData("golden bough deterrence overflow".into()))?;
        if gold.collected.insert(location) {run.run_progression.deterrence=next;}
        Ok(())
    }
    pub(super) fn facility_layout(&self) -> Option<&FacilityLayout> {
        let rooms = self.state.run.as_ref()?.facility_run_state.rooms.as_ref()?;
        self.game_data.map_catalogs.facility_templates.layouts.iter().find(|l|l.id == rooms.layout_id)
    }
    /// Presentation-only predicate; never erase the authoritative encounter.
    fn facility_encounter_hidden(&self, room_id: &str, slot_id: &MapSlotId) -> bool {
        let Some(layout) = self.facility_layout() else { return false; };
        if self.state.player_profile.facility_knowledge.get(&layout.id)
            .is_some_and(|k| k.completed_rooms.contains(room_id)) { return false; }
        layout.room(room_id).is_some_and(|room| room.reservations.iter()
            .find(|r| &r.slot_id == slot_id)
            .and_then(|r| layout.activity(&r.activity_id))
            .is_some_and(|a| matches!(a.payload, MapNodePayload::Encounter { .. })))
    }
    pub(super) fn current_facility_encounter_hidden(&self, id: MapNodeId) -> bool {
        let Some(run) = &self.state.run else { return false; };
        let Some(rooms) = &run.facility_run_state.rooms else { return false; };
        run.map.node(id).is_some_and(|n| self.facility_encounter_hidden(&rooms.current_room_id, &n.slot_id))
    }
    pub(super) fn conceal_current_facility_encounters(&self, view: &mut MapViewDto) {
        for node in &mut view.nodes {
            if self.current_facility_encounter_hidden(node.id) {
                node.payload = MapNodePayload::Encounter { encounter_id: None };
            }
        }
    }
    pub(super) fn facility_session_view(&self, mut session: NodeSession) -> NodeSession {
        if self.current_facility_encounter_hidden(session.node_id) {
            session.payload = MapNodePayload::Encounter { encounter_id: None };
        }
        session
    }
    /// A previously cleared route may be traversed again without another node entry.
    /// Only completed nodes can connect the player to an already reached Gate.
    pub(super) fn facility_gate_reached(&self) -> bool {
        let Some(run) = &self.state.run else {return false;};
        if run.facility_run_state.rooms.is_none() || run.run_progression.pending_continuation.is_some() {return false;}
        if !matches!(self.state.lifecycle.phase(),GameState::ViewingMap | GameState::InNode {category:MapNodeCategory::Gate,..}) {return false;}
        let Some(start) = run.map_progression.current_node_id else {return false;};
        let Ok(template) = self.facility_template_for_map_ref(&run.map) else {return false;};
        let mut visited = HashSet::new(); let mut queue = VecDeque::from([start]);
        while let Some(id) = queue.pop_front() {
            if !visited.insert(id) {continue;}
            let Some(node) = run.map.node(id) else {continue;};
            if node.state != MapNodeState::Completed {continue;}
            if node.category == MapNodeCategory::Gate {return true;}
            queue.extend(run.map.neighbor_node_ids(template,id));
        }
        false
    }
    pub(super) fn record_facility_room_completion(&mut self) {
        let Some(state) = self.state.run.as_ref().and_then(|r|r.facility_run_state.rooms.as_ref()) else {return;};
        self.state.player_profile.facility_knowledge.entry(state.layout_id.clone()).or_default().completed_rooms.insert(state.current_room_id.clone());
    }
    pub(super) fn facility_node_support(&self, id: MapNodeId) -> Option<FacilityContentSupport> {
        let run = self.state.run.as_ref()?;
        let state = run.facility_run_state.rooms.as_ref()?;
        let metadata = state.activity_metadata.get(&id)?;
        let layout = self.facility_layout()?;
        let activity = layout.activity(&metadata.activity_id)?;
        if !activity.support.is_supported() {return Some(activity.support.clone());}
        if activity.condition.is_some() {
            return Some(FacilityContentSupport::Unsupported { reason:"조우 조건 생산자 미지원".into() });
        }
        if metadata.reward_activity.is_some_and(|kind| !Self::basic_relic_activity_supported(kind)
            && !(kind == crate::game::map::RewardActivityKind::Gambling && layout.dealer.is_some())
            && !(kind == crate::game::map::RewardActivityKind::StressCombat && layout.stress_combat.is_some())) {
            return Some(FacilityContentSupport::Unsupported {reason:"이 유물 활동은 후속 U16 묶음에서 지원합니다".into()});
        }
        Some(FacilityContentSupport::Supported)
    }
    pub(super) fn validate_facility_node_entry(&self,id:MapNodeId) -> Result<(),GameError> {
        if self.facility_node_support(id).is_some_and(|s| !s.is_supported()) {return Err(GameError::NotImplemented("facility activity executor".into()));}
        Ok(())
    }
    pub(super) fn handle_move_to_facility_room(&mut self, room_id:&str) -> Result<BehaviorResult,GameError> {
        if !self.facility_gate_reached() {return Err(GameError::InvalidAction);}
        let layout = self.facility_layout().ok_or(GameError::InvalidAction)?;
        let run = self.run_state()?;
        let rooms = run.facility_run_state.rooms.as_ref().ok_or(GameError::InvalidAction)?;
        let edge = layout.connection(&rooms.current_room_id,room_id).ok_or(GameError::InvalidAction)?;
        let destination = layout.room(room_id).ok_or(GameError::InvalidAction)?;
        // Check execution support even when a visible condition or permanent discovery
        // would otherwise permit the approach. No final executor is fabricated here.
        if !destination.support.is_supported() {return Err(GameError::NotImplemented("facility room executor".into()));}
        if edge.condition.is_some() || destination.access_condition.is_some() {return Err(GameError::InvalidAction);}
        let depth = destination.depth;
        if depth == 0 || depth < run.run_progression.floor_index()+1 {return Err(GameError::InvalidAction);}
        let descending = depth > run.run_progression.floor_index()+1;
        let recovery_restricted=self.has_combat_recovery_restriction();
        let target = room_id.to_owned();
        self.with_state_transaction(|core| {
            let mut facility = crate::authoring::clone_shape::controlled_clone(&core.run_state()?.facility_run_state)?;
            let rooms = facility.rooms.as_mut().ok_or(GameError::InvalidAction)?;
            let previous_id = rooms.current_room_id.clone();
            let revisited = rooms.visited_rooms.remove(&target);
            let old = core.run_state()?;
            rooms.visited_rooms.insert(previous_id,FacilityVisitedRoom {map:crate::authoring::clone_shape::controlled_clone(&old.map)?,progression:crate::authoring::clone_shape::controlled_clone(&old.map_progression)?});
            rooms.current_room_id = target.clone();
            let mut progression = crate::authoring::clone_shape::controlled_clone(&old.run_progression)?;
            progression.mode_state = RunProgressionModeState::Expedition {floor_index:depth-1};
            let mut encounters = crate::authoring::clone_shape::controlled_clone(&old.encounter_progression)?;
            if descending {encounters.handle_gate_advance();}
            let (map,navigation,facility) = if let Some(mut saved) = revisited {
                let template = core.facility_template_for_map(&saved.map)?;
                saved.progression.reenter_room(&mut saved.map,&template).map_err(|_|GameError::InvalidAction)?;
                if let Some(anchor) = saved.map.nodes.iter().find_map(|n|n.combat_difficulty.as_ref().map(|d|d.floor_power_anchor)) {
                    core.state.run_power_authority.current_floor_power_anchor = anchor;
                }
                (saved.map,saved.progression,facility)
            } else {core.generate_current_floor_map(&progression,&facility,&mut encounters)};
            if descending {
                core.apply_staged_recovery_effect(super::facility::StagedRecoveryEffect::GateTransition)?;
                core.run_state_mut()?.record_progress.complete_floor();
            }
            // Commit permanent completion only after all fallible preparation/recovery.
            let completed_layout = core.facility_layout().ok_or(GameError::InvalidAction)?.id.clone();
            let completed_room = core.run_state()?.facility_run_state.rooms.as_ref().ok_or(GameError::InvalidAction)?.current_room_id.clone();
            let run = core.run_state_mut()?;
            run.map = map; run.map_progression = navigation; run.facility_run_state = facility;
            run.run_progression = progression; run.encounter_progression = encounters;
            core.state.node_session = None; core.state.active_node_content = None;
            core.transition_to(GameState::ViewingMap)?;
            if recovery_restricted && core.reassess_combat_recovery()?.is_some() {return Err(GameError::InvalidAction);}
            let result = core.handle_request_map_data()?;
            if descending {
                let streak=core.run_state()?.record_progress.current_deathless_floor_streak;
                core.state.player_profile.record_deathless_streak(streak);
                core.state.player_profile.record_floor_reached(depth);
            }
            core.state.player_profile.facility_knowledge.entry(completed_layout).or_default().completed_rooms.insert(completed_room);
            Ok(result)
        })
    }
    pub(super) fn facility_view(&self) -> Result<Option<FacilityViewDto>,GameError> {
        let Some(layout) = self.facility_layout() else {return Ok(None);};
        let run = self.run_state()?;
        let state = run.facility_run_state.rooms.as_ref().ok_or(GameError::InvalidAction)?;
        let knowledge = self.state.player_profile.facility_knowledge.get(&layout.id);
        let mut views = vec![];
        for room in &layout.rooms {
            let explored = knowledge.is_some_and(|k|k.completed_rooms.contains(&room.id));
            let known_final = explored || knowledge.is_some_and(|k|k.discovered_final_rooms.contains(&room.id));
            if room.final_room && !known_final {continue;}
            let current = room.id == state.current_room_id;
            let visited = current || state.visited_rooms.contains_key(&room.id);
            let adjacent = layout.connection(&state.current_room_id,&room.id).is_some() || layout.connection(&room.id,&state.current_room_id).is_some();
            let named = explored || visited || adjacent || room.depth == 0;
            let template = room.template_id.as_ref().and_then(|id|self.game_data.map_catalogs.facility_templates.template(id));
            let instance = if current {Some((&run.map,&run.map_progression))} else {state.visited_rooms.get(&room.id).map(|r|(&r.map,&r.progression))};
            let mut map = instance.zip(template).map(|((m,p),t)|p.view(m,t));
            let mut activities = vec![];
            if let Some((m,_)) = instance {
                for node in &m.nodes {
                    let Some(meta) = state.activity_metadata.get(&node.id) else {continue;};
                    let Some(activity) = layout.activity(&meta.activity_id) else {continue;};
                    let secret = self.facility_encounter_hidden(&room.id, &node.slot_id);
                    if secret {if let Some(n) = map.as_mut().and_then(|v|v.nodes.iter_mut().find(|n|n.id==node.id)) {n.payload=MapNodePayload::Encounter{encounter_id:None};}}
                    let support = if secret {FacilityContentSupport::Unsupported{reason:"미확인 고유 조우 실행 미지원".into()}} else {self.facility_node_support(node.id).unwrap_or(FacilityContentSupport::Supported)};
                    let entry_preview = self.node_entry_preview_in_room(node, Some(&room.id))?;
                    let cost = entry_preview.charged_cost;
                    if let Some(projected) = map.as_mut().and_then(|view|view.nodes.iter_mut().find(|item| item.id == node.id)) {
                        projected.entry_preview = Some(entry_preview.clone());
                    }
                    activities.push(FacilityActivityViewDto {node_id:node.id,display:room.slot_display.get(&node.slot_id).cloned(),label:(!secret).then(||activity.label.clone()),reward_activity:meta.reward_activity,relic_reward:meta.reward_activity.and_then(|kind|self.relic_reward_entry_view(kind)),condition:(!secret).then(||activity.condition.clone()).flatten(),support,entry_preview,first_entry_cost:cost,exhausts_deterrence:cost>0 && run.run_progression.deterrence>0 && run.run_progression.deterrence<=cost,deterrence_already_exhausted:run.run_progression.deterrence==0});
                }
            }
            views.push(FacilityRoomViewDto {id:room.id.clone(),depth:room.depth,wing:room.wing.clone(),display:room.display.clone(),name:named.then(||room.name.clone()),concept:named.then(||room.concept.clone()),explored,visited_this_expedition:visited,descent_recovery:if layout.connection(&state.current_room_id,&room.id).is_some() && room.depth > run.run_progression.floor_index()+1 {self.trauma_recovery_preview_for_category(MapNodeCategory::Gate)?} else {None},support:named.then(||room.support.clone()),access_condition:if room.final_room {room.access_condition.clone()} else {None},interior:if explored || visited {template.cloned()} else {None},current_expedition_map:map,activities,known_reservations:if explored {
                room.reservations.iter().filter_map(|r|layout.activity(&r.activity_id).map(|a|FacilityReservationViewDto{slot_id:r.slot_id.clone(),label:a.label.clone(),kind_id:a.kind_id.clone(),reward_activity:a.reward_activity,support:a.support.clone(),condition:a.condition.clone()})).collect()
            } else {vec![]}});
        }
        let disclosed:HashSet<_> = views.iter().map(|r|r.id.as_str()).collect();
        let connections = layout.connections.iter().filter(|e|disclosed.contains(e.from.as_str()) && disclosed.contains(e.to.as_str())).cloned().collect();
        let reachable = layout.reachable_bough_rooms(&state.current_room_id);
        let golden_boughs = state.golden_boughs.as_ref().map(|gold|gold.locations.iter().map(|location|GoldenBoughMarkerDto {
            room_id:location.room_id.clone(),slot_id:location.slot_id.clone(),
            collected:gold.collected.contains(location),reachable:reachable.contains(&location.room_id),
        }).collect()).unwrap_or_default();
        Ok(Some(FacilityViewDto {golden_boughs,layout_id:layout.id.clone(),current_room_id:state.current_room_id.clone(),rooms:views,connections,gate_reached:self.facility_gate_reached()}))
    }
}

impl GameCore {
    pub(super) fn validate_facility_checkpoint(&self, payload:&super::state::RunCheckpointPayload) -> Result<(),GameError> {
        let Some(state) = &payload.facility_run_state.rooms else {return Ok(());};
        let invalid = || GameError::InvalidStaticData("saved facility room no longer matches authored layout".into());
        let layout = self.game_data.map_catalogs.facility_templates.layouts.iter().find(|l|l.id == state.layout_id).ok_or_else(invalid)?;
        let room = layout.room(&state.current_room_id).ok_or_else(invalid)?;
        if room.template_id.as_ref() != Some(&payload.map.map_template_id) || room.depth != payload.run_progression.floor_index()+1 {return Err(invalid());}
        for (id,visited) in &state.visited_rooms {
            let room = layout.room(id).ok_or_else(invalid)?;
            if id == &state.current_room_id || room.template_id.as_ref() != Some(&visited.map.map_template_id) {return Err(invalid());}
            let template = self.facility_template_for_map(&visited.map)?;
            visited.map.validate_facility_template_contract(&template).map_err(GameError::InvalidStaticData)?;
            if !visited.progression.validate_checkpoint_state(&visited.map,&template) {return Err(invalid());}
        }
        Ok(())
    }
}
