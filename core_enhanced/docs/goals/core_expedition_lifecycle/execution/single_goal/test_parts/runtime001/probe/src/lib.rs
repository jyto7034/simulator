//! Independent U12 consumers. No test-fixtures feature is enabled here.
#![forbid(unsafe_code)]

use game_core::application::{ApplicationBattleAuthority, ApplicationExecutionContext,
    ApplicationSnapshotKind, CoreApplication};
use serde_json::{json, Value};
use std::{collections::BTreeSet, error::Error, fs, io::Write, path::Path};
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Adapters differ only at command publication and fresh-application restore.
/// This driver never accesses World internals or fabricates a battle outcome.
pub trait Host {
    fn app(&self) -> &CoreApplication;
    fn command(&mut self, document: Value, context: ApplicationExecutionContext) -> Result<Value>;
    fn tick(&mut self, delta_ms: u64, after_seq: u64) -> Result<Option<Value>>;
    fn restart(&mut self) -> Result<()>;
}
pub fn snapshot(host: &impl Host, kind: ApplicationSnapshotKind) -> Result<Value> {
    Ok(host.app().snapshot(kind)?.read_model().as_value().clone())
}
fn check(condition: bool, message: &str) -> Result<()> {
    if condition { Ok(()) } else { Err(message.to_owned().into()) }
}
fn array<'a>(value: &'a Value, name: &str) -> Result<&'a Vec<Value>> {
    value.as_array().ok_or_else(|| format!("missing array: {name}").into())
}
fn cursor(value: &Value, after: &mut u64) {
    if let Some(seq) = value.pointer("/events_delta/to_seq").and_then(Value::as_u64) { *after = (*after).max(seq); }
    match value { Value::Object(fields) => for child in fields.values() { cursor(child, after); },
        Value::Array(items) => for child in items { cursor(child, after); }, _ => {} }
}
struct Trace { file: fs::File, commands: usize, ticks: usize, after: u64, battle_id: Option<Value> }
impl Trace {
    fn observe(&mut self, value:&Value) {
        if let Some(id)=value.pointer("/battle_update/battle_uuid").filter(|id|!id.is_null()) {
            if self.battle_id.as_ref()!=Some(id) {self.battle_id=Some(id.clone());self.after=0;}
        }
        cursor(value,&mut self.after);
    }
    fn record(&mut self, value: Value) -> Result<()> {
        serde_json::to_writer(&mut self.file, &value)?; self.file.write_all(b"\n")?; self.file.flush()?; Ok(())
    }
    fn command(&mut self, host: &mut impl Host, document: Value) -> Result<Value> {
        let run = snapshot(host, ApplicationSnapshotKind::Run)?;
        let authority = if run["game_state"] == "in_battle" {
            let id=&run["game_state_context"]["battle_uuid"];
            if self.battle_id.as_ref()!=Some(id) {self.battle_id=Some(id.clone());self.after=0;}
            Some(serde_json::from_value::<ApplicationBattleAuthority>(json!({
                "battle_uuid":run["game_state_context"]["battle_uuid"],
                "battle_state_revision":run["game_state_context"]["battle_state_revision"]}))?)
        } else { None };
        let context = ApplicationExecutionContext { source_command_id: Some(format!("u12-walk-{}", self.commands)),
            battle_event_after_seq: self.after, expected_battle_authority: authority };
        let result = host.command(document.clone(), context);
        self.commands += 1;
        match result {
            Ok(value) => { self.observe(&value); self.record(json!({"operation":"command", "index":self.commands,
                "input":document,"result":value,"run_after":snapshot(host, ApplicationSnapshotKind::Run)?}))?; Ok(value) }
            Err(error) => { self.record(json!({"operation":"command", "index":self.commands,
                "input":document,"error":error.to_string(),"run_after":snapshot(host, ApplicationSnapshotKind::Run)?}))?; Err(error) }
        }
    }
    fn restart(&mut self, host: &mut impl Host) -> Result<()> {
        let before = snapshot(host, ApplicationSnapshotKind::Run)?;
        let profile = snapshot(host, ApplicationSnapshotKind::PlayerProfile)?;
        let checkpoint = host.app().export_active_run_checkpoint()?.ok_or("restart boundary has no checkpoint")?;
        host.restart()?;
        let checkpoint_after = host.app().export_active_run_checkpoint()?;
        self.record(json!({"operation":"fresh_application_continue_checkpoint_pair",
            "before":{"raw_bytes":checkpoint.core_payload_json(),
                "parsed_json":serde_json::from_slice::<Value>(checkpoint.core_payload_json()).ok(),
                "loads_used":checkpoint.loads_used(),
                "run_peak_certified_power":checkpoint.run_peak_certified_power()},
            "after":checkpoint_after.as_ref().map(|copy| json!({
                "raw_bytes":copy.core_payload_json(),
                "parsed_json":serde_json::from_slice::<Value>(copy.core_payload_json()).ok(),
                "loads_used":copy.loads_used(),
                "run_peak_certified_power":copy.run_peak_certified_power()})),
            "raw_bytes_equal":checkpoint_after.as_ref().map(|copy| copy.core_payload_json())
                == Some(checkpoint.core_payload_json()),
            "loads_used_equal":checkpoint_after.as_ref().map(|copy| copy.loads_used())
                == Some(checkpoint.loads_used()),
            "run_peak_certified_power_equal":checkpoint_after.as_ref().map(|copy| copy.run_peak_certified_power())
                == Some(checkpoint.run_peak_certified_power())}))?;
        check(snapshot(host, ApplicationSnapshotKind::PlayerProfile)? == profile, "Continue changed profile")?;
        let after = snapshot(host, ApplicationSnapshotKind::Run)?;
        check(after["run_progression"] == before["run_progression"], "Continue changed progression")?;
        check(after["roster"] == before["roster"], "Continue changed roster")?;
        check(after["inventory"] == before["inventory"], "Continue changed inventory")?;
        check(host.app().export_active_run_checkpoint()?.as_ref().map(|c| c.core_payload_json()) == Some(checkpoint.core_payload_json()),
            "Continue changed saved checkpoint bytes")?;
        self.record(json!({"operation":"fresh_application_continue","before":before,"after":after}))
    }
}

fn xy(cell: &Value) -> (i64, i64) { (cell["x"].as_i64().unwrap_or(0), cell["y"].as_i64().unwrap_or(0)) }
fn distance(a: &Value, b: &Value) -> i64 { let (x,y)=xy(a); let (u,v)=xy(b); (x-u).abs()+(y-v).abs() }
#[derive(Clone, Copy, PartialEq, Eq)]
enum PlacementAction { Deploy, Relocate, Stay, SwapMember }
impl PlacementAction {
    fn name(self) -> &'static str {
        match self { Self::Deploy => "deploy", Self::Relocate => "relocate", Self::Stay => "stay", Self::SwapMember => "swap_member" }
    }
}
#[derive(Clone)]
struct DeploymentPlacement {
    employee_uuid: Value,
    role: &'static str,
    position: Value,
    facing: String,
    basic_attack: Value,
    planning_eligibility: Value,
    action: PlacementAction,
    cost: u64,
    relocation_cooldown_ms: Option<u64>,
    forced_defense_ms: Option<u64>,
}
struct JointDeploymentPlan {
    placements: Vec<DeploymentPlacement>,
    contact: Value,
    target_source: &'static str,
    order: Vec<usize>,
    contact_attackers: u8,
    swap: Option<FormationSwap>,
}
struct FormationSwap { first: usize, second: usize, cost: u64, preview: Value, completed: bool }
fn contact_mask(placements: &[DeploymentPlacement], target: &Value) -> u8 {
    u8::from(covers(&placements[0].basic_attack,target))
        | (u8::from(covers(&placements[1].basic_attack,target)) << 1)
}
fn requires_contact(mask: u8, index: usize) -> bool { index < 2 && mask & (1 << index) != 0 }
fn formation_support(placements: &[DeploymentPlacement], mask: u8) -> bool {
    covers(&placements[2].basic_attack,&placements[0].position)
        && (mask != 2 || covers(&placements[2].basic_attack,&placements[1].position))
}
fn plan_reaches(plan: &JointDeploymentPlan, target: &Value) -> bool {
    contact_mask(&plan.placements,target) & plan.contact_attackers == plan.contact_attackers
}
fn geometry_eligible(eligibility: &Value) -> bool {
    eligibility["enabled"] == true || (eligibility["enabled"] == false
        && eligibility["disabled_reason_code"] == "insufficient_resources")
}
fn relocation_geometry_eligible(eligibility: &Value) -> bool {
    eligibility["enabled"] == true || (eligibility["enabled"] == false
        && matches!(eligibility["disabled_reason_code"].as_str(), Some(
            "insufficient_resources" | "relocation_cooldown" | "action_in_progress"
            | "action_locked" | "hard_crowd_control")))
}
fn covers(basic: &Value, position: &Value) -> bool {
    basic["available"] == true && basic["cells"].as_array().is_some_and(|cells| cells.contains(position))
}
fn living(unit: &Value, owner: &str) -> bool {
    unit["owner"] == owner && unit["role"] == "combatant"
        && unit["stats"]["current_health"].as_u64().is_some_and(|hp| hp > 0)
}
fn actor<'a>(checkpoint: &'a Value, employee: &Value) -> Option<&'a Value> {
    checkpoint["units"].as_array()?.iter().find(|unit| living(unit,"Player")
        && unit["unit_source"]["employee_uuid"] == *employee)
}
fn deployed<'a>(checkpoint: &'a Value, employee: &Value) -> Option<&'a Value> {
    checkpoint["deployment"]["deployed_units"].as_array()?.iter()
        .find(|unit| unit["employee_uuid"] == *employee)
}
fn live_targets(checkpoint: &Value) -> Result<Vec<Value>> {
    let mut targets = Vec::new();
    for unit in array(&checkpoint["units"], "live target units")?.iter().filter(|unit| living(unit,"Opponent")) {
        if !targets.contains(&unit["position"]) { targets.push(unit["position"].clone()); }
    }
    Ok(targets)
}
fn slot_occupied(checkpoint: &Value, cell: &Value, employee: &Value) -> bool {
    // Tactical slots exclude the moving employee. Opponent overlap is legal in
    // the public relocation contract; enemy positions are not allied occupants.
    checkpoint["units"].as_array().is_some_and(|units| units.iter().any(|unit|
        unit["owner"] == "Player" && unit["position"] == *cell
        && unit["unit_source"]["employee_uuid"] != *employee
        && unit["stats"]["current_health"].as_u64().is_some_and(|hp| hp > 0)
        && matches!(unit["role"].as_str(),Some("combatant"|"defense_object"|"independent_wall"))))
        || checkpoint["independent_objects"]["active"].as_array()
            .is_some_and(|objects| objects.iter().any(|object| object["position"] == *cell))
}
fn formation_order(placements: &[DeploymentPlacement], checkpoint: &Value, mask: u8) -> Option<Vec<usize>> {
    // This is a conservative driver strategy, not a product eligibility rule.
    // Existing Tank support must survive each chosen intermediate relocation;
    // a sole Dealer attacker also keeps healing coverage along that sequence.
    // Before the initial Healer arrives, normal initial deployment is allowed.
    for order in [[0,1,2],[0,2,1],[1,0,2],[1,2,0],[2,0,1],[2,1,0]] {
        let mut current: Vec<Option<(Value,Value)>> = placements.iter().map(|placement|
            actor(checkpoint,&placement.employee_uuid).map(|unit|
                (unit["position"].clone(),unit["range_previews"]["basic_attack"].clone()))).collect();
        let mut valid = true;
        for index in order {
            let placement = &placements[index];
            if placement.action == PlacementAction::Stay { continue; }
            current[index] = Some((placement.position.clone(),placement.basic_attack.clone()));
            if let (Some((tank,_)),Some((_,healing))) = (&current[0],&current[2]) {
                if !covers(healing,tank) { valid = false; break; }
            }
            if mask == 2 {
                if let (Some((dealer,_)),Some((_,healing))) = (&current[1],&current[2]) {
                    if !covers(healing,dealer) { valid = false; break; }
                }
            }
        }
        if valid { return Some(order.to_vec()); }
    }
    None
}
fn joint_deployment_plan(
    host: &mut impl Host, trace: &mut Trace, employees: &[&Value], setup: &Value,
    checkpoint: &Value, targets: &[Value], target_source: &'static str,
) -> Result<Option<JointDeploymentPlan>> {
    check(!targets.is_empty(), "joint deployment needs an authoritative contact")?;
    let zones = array(&setup["deployment_zones"],"formation deployment zones")?;
    let costs = array(&checkpoint["deployment"]["unit_deploy_costs"], "planning costs")?;
    let mut candidates = Vec::new();
    let mut role_eligibility = Vec::new();
    for role in ["Tank", "Dealer", "Healer"] {
        let employee = employees.iter().find(|employee| employee["role"] == role)
            .ok_or_else(|| format!("joint deployment missing role {role}"))?;
        let employee_uuid = &employee["uuid"];
        let live = actor(checkpoint,employee_uuid);
        let current = deployed(checkpoint,employee_uuid);
        // Deployed employees are absent from unit_deploy_costs. Their tactical
        // availability and cost come from deployed state and relocation preview.
        let cost = if live.is_none() {
            Some(costs.iter().find(|cost| cost["employee_uuid"] == *employee_uuid)
                .ok_or("unplaced employee has no returned deployment cost")?)
        } else { None };
        let eligibility = if live.is_some() {
            &current.ok_or("living employee lacks public deployed record")?["tactical_state"]["relocation_eligibility"]
        } else { &cost.unwrap()["eligibility"] };
        role_eligibility.push(json!({"employee_uuid":employee_uuid,"role":role,"eligibility":eligibility}));
        let mut placements = Vec::new();
        if let Some(unit) = live {
            let basic = &unit["range_previews"]["basic_attack"];
            if basic["available"] == true {
                placements.push(DeploymentPlacement { employee_uuid:employee_uuid.clone(), role,
                    position:unit["position"].clone(), facing:current.unwrap()["facing"].as_str().ok_or("current facing")?.to_owned(),
                    basic_attack:basic.clone(), planning_eligibility:Value::Null,
                    action:PlacementAction::Stay, cost:0, relocation_cooldown_ms:None, forced_defense_ms:None });
            }
        }
        let can_plan = if live.is_some() { relocation_geometry_eligible(eligibility) } else { geometry_eligible(eligibility) };
        if can_plan {
            let affinity = employee["combat_profile"]["effective_deployment_affinity"].as_str().ok_or("planning affinity")?;
            let mut cells: Vec<_> = zones.iter().filter(|zone| matches!((affinity,zone["kind"].as_str()),
                ("any",Some("Ground"|"Platform"))|("ground_only",Some("Ground"))|("platform_only",Some("Platform"))))
                .filter_map(|zone| zone["cells"].as_array()).flatten()
                .filter(|cell| !slot_occupied(checkpoint,cell,employee_uuid))
                .filter(|cell| !setup["battlefield"]["static_obstacles"].as_array().is_some_and(|blocked| blocked.contains(cell)))
                .collect();
            cells.sort_by_key(|cell| (distance(cell,&targets[0]), -xy(cell).1, xy(cell).0));
            cells.dedup();
            for cell in cells {
                if live.is_some() {
                    for facing in ["up","right","down","left"] {
                        if live.unwrap()["position"] == *cell && current.unwrap()["facing"] == facing { continue; }
                        let preview = trace.command(host,json!({"type":"preview_tactical_command","command":{
                            "type":"relocate","employee_uuid":employee_uuid,"destination":cell,"facing":facing}}))?;
                        if !relocation_geometry_eligible(&preview["eligibility"]) { continue; }
                        let projected = array(&preview["units"],"tactical preview units")?.first().ok_or("tactical preview actor")?;
                        check(projected["employee_uuid"] == *employee_uuid && projected["destination"] == *cell
                            && projected["facing"] == facing,"tactical preview does not match requested placement")?;
                        let basic = &projected["range_previews"]["basic_attack"];
                        if basic["available"] != true || array(&basic["cells"],"tactical planning range")?.is_empty() { continue; }
                        placements.push(DeploymentPlacement { employee_uuid:employee_uuid.clone(), role,
                            position:cell.clone(), facing:facing.to_owned(), basic_attack:basic.clone(),
                            planning_eligibility:preview["eligibility"].clone(), action:PlacementAction::Relocate,
                            cost:preview["cost"].as_u64().ok_or("tactical preview cost")?,
                            relocation_cooldown_ms:Some(preview["relocation_cooldown_ms"].as_u64().ok_or("relocation cooldown")?),
                            forced_defense_ms:Some(projected["forced_defense_ms"].as_u64().ok_or("forced defense duration")?) });
                    }
                } else {
                    let preview = trace.command(host,json!({"type":"request_deployment_range_preview",
                        "employee_uuid":employee_uuid,"position":cell}))?;
                    if !geometry_eligible(&preview["deploy_eligibility"]) { continue; }
                    for facing in ["up", "right", "down", "left"] {
                        let basic = &preview["facings"][facing]["range_previews"]["basic_attack"];
                        if basic["available"] != true || array(&basic["cells"], "planning basic range")?.is_empty() { continue; }
                        placements.push(DeploymentPlacement { employee_uuid:employee_uuid.clone(), role,
                            position:cell.clone(), facing:facing.to_owned(), basic_attack:basic.clone(),
                            planning_eligibility:preview["deploy_eligibility"].clone(), action:PlacementAction::Deploy,
                            cost:cost.unwrap()["effective_deploy_cost"].as_u64().ok_or("planning deploy cost")?,
                            relocation_cooldown_ms:None, forced_defense_ms:None });
                    }
                }
            }
        }
        candidates.push(placements);
    }
    // Prefer the original shared contact. If none exists, one actual attacker
    // may engage while the other keeps a legal supported position. This is a
    // driver strategy; the battle's terminal contract remains unchanged.
    let mut best = None;
    let mut best_order = None;
    let mut best_mask = 0;
    for attacker_count in [2,1] {
        for (target_index,target) in targets.iter().enumerate() {
            for (tank_index,tank) in candidates[0].iter().enumerate() {
                for (dealer_index,dealer) in candidates[1].iter().enumerate().filter(|(_,p)|p.position!=tank.position) {
                    let mask=u8::from(covers(&tank.basic_attack,target)) | (u8::from(covers(&dealer.basic_attack,target))<<1);
                    if mask.count_ones()!=attacker_count { continue; }
                    for (healer_index,healer) in candidates[2].iter().enumerate().filter(|(_,p)|
                        p.position!=tank.position && p.position!=dealer.position
                        && covers(&p.basic_attack,&tank.position)
                        && (mask!=2 || covers(&p.basic_attack,&dealer.position))) {
                        let proposed=[tank.clone(),dealer.clone(),healer.clone()];
                        if !formation_support(&proposed,mask) { continue; }
                        let Some(order)=formation_order(&proposed,checkpoint,mask) else { continue; };
                        let changes=proposed.iter().filter(|p|p.action!=PlacementAction::Stay).count();
                        let score=(if target_source=="returned_route" {target_index} else {0},changes,
                            !covers(&healer.basic_attack,&dealer.position),
                            distance(&tank.position,target)+distance(&dealer.position,target)+distance(&healer.position,&tank.position),
                            target_index,tank_index,dealer_index,healer_index);
                        if best.as_ref().is_none_or(|previous|score<*previous) {
                            best=Some(score);best_order=Some(order);best_mask=mask;
                        }
                    }
                }
            }
        }
        if best.is_some() { break; }
    }
    if let Some((_,_,_,_,target_index,tank,dealer,healer))=best {
        let plan=JointDeploymentPlan { placements:vec![candidates[0][tank].clone(),candidates[1][dealer].clone(),candidates[2][healer].clone()],
            contact:targets[target_index].clone(),target_source,order:best_order.unwrap(),contact_attackers:best_mask,swap:None };
        record_formation_plan(trace,&plan)?;
        return Ok(Some(plan));
    }
    // A vacatable allied slot has no valid relocation preview: the product
    // returns PositionOccupied. Only its real atomic Swap preview can authorize
    // the two new poses without inventing a range for an occupied destination.
    if target_source=="live_opponent" {
        if let Some(plan)=swap_deployment_plan(host,trace,employees,checkpoint,targets)? {
            record_formation_plan(trace,&plan)?;
            return Ok(Some(plan));
        }
    }
    trace.record(json!({"operation":"joint_formation_unavailable","target_source":target_source,"targets":targets,
        "geometry_candidates":candidates.iter().map(Vec::len).collect::<Vec<_>>(),"role_eligibility":role_eligibility,
        "driver_strategy":"shared contact first, supported single attacker second, actual allied Swap preview last"}))?;
    Ok(None)
}
fn record_formation_plan(trace: &mut Trace, plan: &JointDeploymentPlan) -> Result<()> {
    trace.record(json!({"operation":"joint_deployment_plan","contact":plan.contact,"target_source":plan.target_source,
        "contact_attackers":plan.placements.iter().enumerate().filter(|(index,_)|requires_contact(plan.contact_attackers,*index))
            .map(|(_,p)|p.role).collect::<Vec<_>>(),
        "healer_covers_tank":covers(&plan.placements[2].basic_attack,&plan.placements[0].position),
        "healer_covers_dealer":covers(&plan.placements[2].basic_attack,&plan.placements[1].position),
        "commands_required":if plan.swap.is_some() {1} else {plan.placements.iter().filter(|p|p.action!=PlacementAction::Stay).count()},
        "execution_order":plan.order,"swap_preview":plan.swap.as_ref().map(|swap|&swap.preview),
        "placements":plan.placements.iter().map(|p|json!({"employee_uuid":p.employee_uuid,"role":p.role,
            "position":p.position,"facing":p.facing,"basic_attack":p.basic_attack,"action":p.action.name(),
            "planning_eligibility":p.planning_eligibility,"cost":p.cost,"relocation_cooldown_ms":p.relocation_cooldown_ms,
            "forced_defense_ms":p.forced_defense_ms})).collect::<Vec<_>>()}))
}
fn swap_deployment_plan(host: &mut impl Host, trace: &mut Trace, employees: &[&Value],
    checkpoint: &Value, targets: &[Value]) -> Result<Option<JointDeploymentPlan>> {
    let mut current=Vec::new();
    let mut affinities=Vec::new();
    let mut tile_kinds=Vec::new();
    for role in ["Tank","Dealer","Healer"] {
        let employee=employees.iter().find(|employee|employee["role"]==role).ok_or("swap role employee")?;
        let Some(unit)=actor(checkpoint,&employee["uuid"]) else { return Ok(None); };
        let Some(deployment)=deployed(checkpoint,&employee["uuid"]) else { return Ok(None); };
        let basic=&unit["range_previews"]["basic_attack"];
        if basic["available"]!=true { return Ok(None); }
        current.push(DeploymentPlacement { employee_uuid:employee["uuid"].clone(),role,position:unit["position"].clone(),
            facing:deployment["facing"].as_str().ok_or("swap current facing")?.to_owned(),basic_attack:basic.clone(),
            planning_eligibility:deployment["tactical_state"]["relocation_eligibility"].clone(),
            action:PlacementAction::Stay,cost:0,relocation_cooldown_ms:None,forced_defense_ms:None });
        affinities.push(employee["combat_profile"]["effective_deployment_affinity"].as_str().ok_or("swap affinity")?);
        tile_kinds.push(unit["tactical_state"]["tile_kind"].as_str().ok_or("swap current tile kind")?);
    }
    let compatible=|index:usize,destination:usize|matches!((affinities[index],tile_kinds[destination]),
        ("any","ground"|"platform")|("ground_only","ground")|("platform_only","platform"));
    let mut best=None;
    let mut best_score=None;
    for (first,second) in [(0,1),(0,2),(1,2)] {
        if !compatible(first,second) || !compatible(second,first)
            || !relocation_geometry_eligible(&current[first].planning_eligibility)
            || !relocation_geometry_eligible(&current[second].planning_eligibility) { continue; }
        for first_facing in ["up","right","down","left"] {
            for second_facing in ["up","right","down","left"] {
                let command=json!({"type":"swap","first_employee_uuid":current[first].employee_uuid,
                    "second_employee_uuid":current[second].employee_uuid,"first_facing":first_facing,"second_facing":second_facing});
                let preview=trace.command(host,json!({"type":"preview_tactical_command","command":command}))?;
                if !relocation_geometry_eligible(&preview["eligibility"]) { continue; }
                check(preview["command"]==command,"swap preview command identity")?;
                let projected=array(&preview["units"],"swap projected units")?;
                check(projected.len()==2,"swap must project exactly two employees")?;
                let mut proposed=current.clone();
                for (index,other,facing) in [(first,second,first_facing),(second,first,second_facing)] {
                    let unit=projected.iter().find(|unit|unit["employee_uuid"]==current[index].employee_uuid).ok_or("swap projected employee")?;
                    check(unit["destination"]==current[other].position && unit["facing"]==facing,"swap projected pose")?;
                    proposed[index].position=unit["destination"].clone();proposed[index].facing=facing.to_owned();
                    proposed[index].basic_attack=unit["range_previews"]["basic_attack"].clone();
                    proposed[index].planning_eligibility=preview["eligibility"].clone();proposed[index].action=PlacementAction::SwapMember;
                    proposed[index].cost=preview["cost"].as_u64().ok_or("swap cost")?;
                    proposed[index].relocation_cooldown_ms=Some(preview["relocation_cooldown_ms"].as_u64().ok_or("swap cooldown")?);
                    proposed[index].forced_defense_ms=Some(unit["forced_defense_ms"].as_u64().ok_or("swap forced defense")?);
                }
                for (target_index,target) in targets.iter().enumerate() {
                    let mask=contact_mask(&proposed,target);
                    if mask==0 || !formation_support(&proposed,mask) { continue; }
                    let score=(2-mask.count_ones(),!covers(&proposed[2].basic_attack,&proposed[1].position),
                        target_index,first,second,first_facing,second_facing);
                    if best_score.as_ref().is_none_or(|previous|score<*previous) {
                        best_score=Some(score);
                        best=Some(JointDeploymentPlan { placements:proposed.clone(),contact:target.clone(),target_source:"live_opponent",
                            order:vec![0,1,2],contact_attackers:mask,swap:Some(FormationSwap { first,second,
                                cost:preview["cost"].as_u64().ok_or("swap cost")?,preview:preview.clone(),completed:false }) });
                    }
                }
            }
        }
    }
    Ok(best)
}

enum FormationProgress { Complete, Waiting(Value), Replan(Value) }
fn advance_formation(host: &mut impl Host, trace: &mut Trace, plan: &mut JointDeploymentPlan,
    deployed_employees: &mut BTreeSet<String>) -> Result<FormationProgress> {
    if plan.swap.as_ref().is_some_and(|swap|!swap.completed) {
        match advance_formation_swap(host,trace,plan)? {
            FormationProgress::Complete=>{},
            other=>return Ok(other),
        }
    }
    for &index in &plan.order {
        if !host.app().has_active_battle() { return Ok(FormationProgress::Complete); }
        let payload = host.app().active_battle_reconnect(trace.after)?.payload().as_value().clone();
        let checkpoint = &payload["battle_update"]["checkpoint"];
        let targets = live_targets(checkpoint)?;
        if plan.target_source == "live_opponent" {
            if targets.is_empty() { return Ok(FormationProgress::Waiting(json!({"reason":"no_live_opponent"}))); }
            let contact = targets.iter().find(|target| plan_reaches(plan,target));
            let Some(contact) = contact else {
                return Ok(FormationProgress::Replan(json!({"reason":"live_contact_drift","actual_targets":targets,"prior_contact":plan.contact})));
            };
            plan.contact = contact.clone();
        }
        let placement = &plan.placements[index];
        let live = actor(checkpoint,&placement.employee_uuid);
        let current = deployed(checkpoint,&placement.employee_uuid);
        if live.is_some_and(|unit| unit["position"] == placement.position)
            && current.is_some_and(|unit| unit["facing"] == placement.facing) { continue; }
        if matches!(placement.action,PlacementAction::Stay|PlacementAction::SwapMember)
            || (placement.action == PlacementAction::Relocate && live.is_none()) {
            return Ok(FormationProgress::Replan(json!({"reason":"current_actor_changed","employee_uuid":placement.employee_uuid})));
        }
        if slot_occupied(checkpoint,&placement.position,&placement.employee_uuid) {
            return Ok(FormationProgress::Replan(json!({"reason":"destination_now_occupied","employee_uuid":placement.employee_uuid,"position":placement.position})));
        }
        let action = if live.is_some() { PlacementAction::Relocate } else { PlacementAction::Deploy };
        let (eligibility,cost) = if action == PlacementAction::Relocate {
            (&current.ok_or("relocation deployed state")?["tactical_state"]["relocation_eligibility"],placement.cost)
        } else {
            let cost = array(&checkpoint["deployment"]["unit_deploy_costs"],"latest deployment costs")?.iter()
                .find(|cost| cost["employee_uuid"] == placement.employee_uuid).ok_or("latest deploy cost")?;
            (&cost["eligibility"],cost["effective_deploy_cost"].as_u64().ok_or("latest deploy cost value")?)
        };
        let resource = checkpoint["tactical_resource"]["current"].as_u64().ok_or("latest tactical resource")?;
        if eligibility["enabled"] != true || resource < cost {
            return Ok(FormationProgress::Waiting(json!({"reason":"current_action_unavailable","employee_uuid":placement.employee_uuid,
                "action":action.name(),"eligibility":eligibility,"current_resource":resource,"required_cost":cost,
                "tactical_state":current.map(|unit|&unit["tactical_state"])})));
        }
        let preview = if action == PlacementAction::Relocate {
            trace.command(host,json!({"type":"preview_tactical_command","command":{"type":"relocate",
                "employee_uuid":placement.employee_uuid,"destination":placement.position,"facing":placement.facing}}))?
        } else {
            trace.command(host,json!({"type":"request_deployment_range_preview",
                "employee_uuid":placement.employee_uuid,"position":placement.position}))?
        };
        let (enabled,basic) = if action == PlacementAction::Relocate {
            let projected = array(&preview["units"],"fresh tactical units")?.first().ok_or("fresh tactical actor")?;
            check(projected["employee_uuid"] == placement.employee_uuid && projected["destination"] == placement.position
                && projected["facing"] == placement.facing,"fresh tactical placement identity")?;
            (&preview["eligibility"],&projected["range_previews"]["basic_attack"])
        } else { (&preview["deploy_eligibility"],&preview["facings"][&placement.facing]["range_previews"]["basic_attack"]) };
        if enabled["enabled"] != true {
            return Ok(FormationProgress::Waiting(json!({"reason":"fresh_preview_unavailable","employee_uuid":placement.employee_uuid,"preview":preview})));
        }
        let tank = actor(checkpoint,&plan.placements[0].employee_uuid);
        let healer = actor(checkpoint,&plan.placements[2].employee_uuid);
        let target = if placement.role == "Healer" {
            tank.map(|unit|unit["position"].clone()).unwrap_or_else(||plan.placements[0].position.clone())
        } else { plan.contact.clone() };
        let dealer = actor(checkpoint,&plan.placements[1].employee_uuid);
        let needs_contact = requires_contact(plan.contact_attackers,index);
        let needs_healing = index==0 || (index==1 && plan.contact_attackers==2);
        let healer_supports_dealer = placement.role!="Healer" || plan.contact_attackers!=2
            || covers(basic,&dealer.map(|unit|unit["position"].clone()).unwrap_or_else(||plan.placements[1].position.clone()));
        if ((placement.role=="Healer" || needs_contact) && !covers(basic,&target))
            || !healer_supports_dealer || (needs_healing && healer.is_some_and(|unit|
                !covers(&unit["range_previews"]["basic_attack"],&placement.position))) {
            return Ok(FormationProgress::Replan(json!({"reason":"fresh_range_or_support_changed","employee_uuid":placement.employee_uuid,
                "target":target,"preview":preview})));
        }
        trace.record(json!({"operation":if action == PlacementAction::Deploy {"deployment_selection"} else {"tactical_relocation_selection"},
            "employee_uuid":placement.employee_uuid,"role":placement.role,"position":placement.position,"facing":placement.facing,
            "target":target,"contact_required":needs_contact,
            "target_source":if placement.role == "Healer" {"live_ally_or_planned_initial_tank"} else {plan.target_source},
            "basic_attack":basic,"actual_preview":preview,"planned_execution_order":plan.order}))?;
        trace.command(host,if action == PlacementAction::Relocate {
            json!({"type":"tactical_relocate_unit","employee_uuid":placement.employee_uuid,"destination":placement.position,"facing":placement.facing})
        } else { json!({"type":"deploy_unit","employee_uuid":placement.employee_uuid,"position":placement.position,"facing":placement.facing}) })?;
        if action == PlacementAction::Deploy {
            deployed_employees.insert(placement.employee_uuid.as_str().ok_or("employee UUID")?.to_owned());
        }
        // Every following command fetches the new checkpoint and Trace.command
        // reads the latest Run authority, including after same-cell turns.
    }
    if !host.app().has_active_battle() { return Ok(FormationProgress::Complete); }
    let payload = host.app().active_battle_reconnect(trace.after)?.payload().as_value().clone();
    let checkpoint = &payload["battle_update"]["checkpoint"];
    let Some(tank) = actor(checkpoint,&plan.placements[0].employee_uuid) else {
        return Ok(FormationProgress::Replan(json!({"reason":"final_tank_unavailable"})));
    };
    let Some(dealer) = actor(checkpoint,&plan.placements[1].employee_uuid) else {
        return Ok(FormationProgress::Replan(json!({"reason":"final_dealer_unavailable"})));
    };
    let Some(healer) = actor(checkpoint,&plan.placements[2].employee_uuid) else {
        return Ok(FormationProgress::Replan(json!({"reason":"final_healer_unavailable"})));
    };
    if (requires_contact(plan.contact_attackers,0) && !covers(&tank["range_previews"]["basic_attack"],&plan.contact))
        || (requires_contact(plan.contact_attackers,1) && !covers(&dealer["range_previews"]["basic_attack"],&plan.contact))
        || !covers(&healer["range_previews"]["basic_attack"],&tank["position"])
        || (plan.contact_attackers==2 && !covers(&healer["range_previews"]["basic_attack"],&dealer["position"])) {
        return Ok(FormationProgress::Replan(json!({"reason":"final_live_joint_ranges_changed","contact":plan.contact})));
    }
    Ok(FormationProgress::Complete)
}
fn advance_formation_swap(host: &mut impl Host, trace: &mut Trace, plan: &mut JointDeploymentPlan) -> Result<FormationProgress> {
    if !host.app().has_active_battle() { return Ok(FormationProgress::Complete); }
    let swap=plan.swap.as_ref().ok_or("missing formation swap")?;
    let (first,second,cost)=(swap.first,swap.second,swap.cost);
    let payload=host.app().active_battle_reconnect(trace.after)?.payload().as_value().clone();
    let checkpoint=&payload["battle_update"]["checkpoint"];
    let targets=live_targets(checkpoint)?;
    let Some(contact)=targets.iter().find(|target|plan_reaches(plan,target)) else {
        return Ok(FormationProgress::Replan(json!({"reason":"swap_contact_drift","actual_targets":targets})));
    };
    plan.contact=contact.clone();
    let mut current=plan.placements.clone();
    for (index,placement) in current.iter_mut().enumerate() {
        let Some(unit)=actor(checkpoint,&placement.employee_uuid) else {
            return Ok(FormationProgress::Replan(json!({"reason":"swap_requires_three_living_employees"})));
        };
        let deployment=deployed(checkpoint,&placement.employee_uuid).ok_or("swap current deployment")?;
        placement.position=unit["position"].clone();placement.basic_attack=unit["range_previews"]["basic_attack"].clone();
        placement.facing=deployment["facing"].as_str().ok_or("swap current facing")?.to_owned();
        if index==first || index==second {
            let eligibility=&deployment["tactical_state"]["relocation_eligibility"];
            if eligibility["enabled"]!=true {
                return Ok(FormationProgress::Waiting(json!({"reason":"current_swap_actor_unavailable",
                    "employee_uuid":placement.employee_uuid,"eligibility":eligibility})));
            }
        }
    }
    // An atomic swap is meaningful only while both source slots still match
    // the reviewed pair; a changed actor or position requires a new plan.
    if current[first].position!=plan.placements[second].position || current[second].position!=plan.placements[first].position {
        return Ok(FormationProgress::Replan(json!({"reason":"swap_source_positions_changed"})));
    }
    let resource=checkpoint["tactical_resource"]["current"].as_u64().ok_or("swap tactical resource")?;
    if resource<cost { return Ok(FormationProgress::Waiting(json!({"reason":"current_swap_resources",
        "current_resource":resource,"required_cost":cost}))); }
    let command=json!({"type":"swap","first_employee_uuid":plan.placements[first].employee_uuid,
        "second_employee_uuid":plan.placements[second].employee_uuid,"first_facing":plan.placements[first].facing,
        "second_facing":plan.placements[second].facing});
    let preview=trace.command(host,json!({"type":"preview_tactical_command","command":command}))?;
    check(preview["command"]==command,"fresh swap command identity")?;
    if preview["eligibility"]["enabled"]!=true {
        return Ok(FormationProgress::Waiting(json!({"reason":"fresh_preview_unavailable","preview":preview})));
    }
    let actual_cost=preview["cost"].as_u64().ok_or("fresh swap cost")?;
    if resource<actual_cost { return Ok(FormationProgress::Waiting(json!({"reason":"current_swap_resources",
        "current_resource":resource,"required_cost":actual_cost}))); }
    let projected=array(&preview["units"],"fresh swap projected units")?;
    check(projected.len()==2,"fresh swap has exactly two units")?;
    for index in [first,second] {
        let unit=projected.iter().find(|unit|unit["employee_uuid"]==plan.placements[index].employee_uuid).ok_or("fresh swap employee")?;
        let actor_id=&actor(checkpoint,&plan.placements[index].employee_uuid).ok_or("fresh swap living actor")?["unit_instance_id"];
        check(unit["unit_instance_id"]==*actor_id && unit["destination"]==plan.placements[index].position
            && unit["facing"]==plan.placements[index].facing,"fresh swap actor and pose identity")?;
        current[index].position=unit["destination"].clone();current[index].facing=unit["facing"].as_str().ok_or("fresh swap facing")?.to_owned();
        current[index].basic_attack=unit["range_previews"]["basic_attack"].clone();
    }
    if contact_mask(&current,&plan.contact)&plan.contact_attackers!=plan.contact_attackers
        || !formation_support(&current,plan.contact_attackers) {
        return Ok(FormationProgress::Replan(json!({"reason":"fresh_swap_contact_or_support_changed","preview":preview})));
    }
    trace.record(json!({"operation":"tactical_swap_selection","contact":plan.contact,"contact_attackers":plan.contact_attackers,
        "actual_preview":preview,"three_living_employees":true,"atomic_supported_destinations":true}))?;
    trace.command(host,json!({"type":"tactical_swap_units","first_employee_uuid":plan.placements[first].employee_uuid,
        "second_employee_uuid":plan.placements[second].employee_uuid,"first_facing":plan.placements[first].facing,
        "second_facing":plan.placements[second].facing}))?;
    plan.swap.as_mut().unwrap().completed=true;
    Ok(FormationProgress::Complete)
}
fn formation_fingerprint(checkpoint: &Value, plan: &JointDeploymentPlan) -> Result<String> {
    // Cooldowns, resources and the clock cannot authorize an identical loop.
    // A changed opponent, tile or HP is fresh evidence for the same formation.
    let opponents:Vec<_>=array(&checkpoint["units"],"formation fingerprint units")?.iter().filter(|unit|living(unit,"Opponent"))
        .map(|unit|json!({"id":unit["unit_instance_id"],"position":unit["position"],"health":unit["stats"]["current_health"]})).collect();
    Ok(serde_json::to_string(&json!({"opponents":opponents,"contact_attackers":plan.contact_attackers,
        "placements":plan.placements.iter().map(|p|json!({"employee_uuid":p.employee_uuid,"position":p.position,"facing":p.facing})).collect::<Vec<_>>()}))?)
}
fn planning_key(checkpoint: &Value, targets: &[Value], source: &str) -> Result<String> {
    let actors: Vec<_> = array(&checkpoint["units"],"planning fingerprint units")?.iter()
        .filter(|unit|unit["owner"] == "Player" && unit["stats"]["current_health"].as_u64().is_some_and(|hp|hp>0))
        .map(|unit| json!({"id":unit["unit_instance_id"],"position":unit["position"],
            "employee_uuid":unit["unit_source"]["employee_uuid"],"basic_attack":unit["range_previews"]["basic_attack"]})).collect();
    let objects: Vec<_> = checkpoint["independent_objects"]["active"].as_array().into_iter().flatten()
        .map(|object|json!({"id":object["object_id"],"position":object["position"]})).collect();
    let deployment_availability: Vec<_> = checkpoint["deployment"]["unit_deploy_costs"].as_array().into_iter().flatten()
        .map(|cost|json!({"employee_uuid":cost["employee_uuid"],"eligibility":cost["eligibility"]})).collect();
    let relocation_availability: Vec<_> = checkpoint["deployment"]["deployed_units"].as_array().into_iter().flatten()
        .map(|unit|json!({"employee_uuid":unit["employee_uuid"],"facing":unit["facing"],
            "eligibility":unit["tactical_state"]["relocation_eligibility"]})).collect();
    let opponents:Vec<_>=array(&checkpoint["units"],"planning opponent state")?.iter().filter(|unit|living(unit,"Opponent"))
        .map(|unit|json!({"id":unit["unit_instance_id"],"position":unit["position"],"health":unit["stats"]["current_health"]})).collect();
    Ok(serde_json::to_string(&json!({"targets":targets,"source":source,"actors":actors,"objects":objects,"opponents":opponents,
        "deployment_availability":deployment_availability,"relocation_availability":relocation_availability}))?)
}
fn close_driver_skill_focus(host: &mut impl Host, trace: &mut Trace) -> Result<()> {
    if !host.app().has_active_battle() { return Ok(()); }
    let payload=host.app().active_battle_reconnect(trace.after)?.payload().as_value().clone();
    if !payload["battle_update"]["checkpoint"]["tactical_focus"].is_null() {
        trace.command(host,json!({"type":"cancel_skill_targeting"}))?;
    }
    Ok(())
}
fn use_ready_skills(host: &mut impl Host, trace: &mut Trace,
    attempted_targets: &mut std::collections::BTreeMap<String,Value>) -> Result<()> {
    if !host.app().has_active_battle() { return Ok(()); }
    let payload=host.app().active_battle_reconnect(trace.after)?.payload().as_value().clone();
    let employee_ids: Vec<_> = array(&payload["battle_update"]["checkpoint"]["deployment"]["deployed_units"],"skill deployment")?
        .iter().map(|unit|unit["employee_uuid"].clone()).collect();
    for employee in employee_ids {
        if !host.app().has_active_battle() { break; }
        let payload=host.app().active_battle_reconnect(trace.after)?.payload().as_value().clone();
        let checkpoint=&payload["battle_update"]["checkpoint"];
        check(checkpoint["tactical_focus"].is_null(),"driver cannot leave a skill focus across host ticks")?;
        let Some(caster)=actor(checkpoint,&employee) else { continue; };
        let Some(deployment)=deployed(checkpoint,&employee) else { continue; };
        let readiness=&deployment["skill_readiness"];
        if readiness["activation_mode"]!="manual" || readiness["manual_activation_allowed"]!=true
            || readiness["eligibility"]["enabled"]!=true || readiness["target_available"]!=true { continue; }
        let skill=readiness["skill_id"].as_str().ok_or("ready skill id")?;
        let units=array(&checkpoint["units"],"skill live units")?;
        let contact=units.iter().any(|attacker|living(attacker,"Player") && attacker["basic_action"]["kind"]=="damage"
            && units.iter().filter(|unit|living(unit,"Opponent"))
                .any(|target|covers(&attacker["range_previews"]["basic_attack"],&target["position"])));
        if !contact { continue; }
        match readiness["selection"].as_str() {
            Some("self"|"automatic_unit") => {
                // For these selections, returned eligibility already performs
                // the exact manual activation validation with no explicit target.
                trace.record(json!({"operation":"skill_activation_selection","employee_uuid":employee,
                    "skill_id":skill,"readiness":readiness,"selection_source":"current_skill_readiness","actual_combat_contact":true}))?;
                trace.command(host,json!({"type":"activate_skill","employee_uuid":employee,"skill_id":skill,"target":null}))?;
            }
            Some("player_unit") => {
                let run=snapshot(host,ApplicationSnapshotKind::Run)?;
                let catalog=array(&run["skill_catalog"]["skills"],"public skill catalog")?.iter()
                    .find(|definition|definition["skill_id"]==skill).ok_or("ready skill not in public catalog")?;
                // The currently required basic manual support case has an
                // explicit public ally policy. Unknown policies and tile casts
                // are not guessed from skill IDs or translated display names.
                if catalog["cast_target"]["target_policy"]!="AllySingle" { continue; }
                let range=&caster["range_previews"]["active_skill"];
                if range["available"]!=true || range["skill_id"]!=skill { continue; }
                let cells=array(&range["cast_cells"],"actual manual skill cast cells")?;
                let mut candidates: Vec<_> = units.iter().filter(|unit|living(unit,"Player")
                    && unit["stats"]["current_health"].as_u64().zip(unit["stats"]["max_health"].as_u64())
                        .is_some_and(|(hp,max)|hp<max)
                    && cells.contains(&unit["position"])).collect();
                candidates.sort_by(|a,b| {
                    let ratio=(a["stats"]["current_health"].as_u64().unwrap()*b["stats"]["max_health"].as_u64().unwrap())
                        .cmp(&(b["stats"]["current_health"].as_u64().unwrap()*a["stats"]["max_health"].as_u64().unwrap()));
                    ratio.then_with(||a["unit_instance_id"].as_str().cmp(&b["unit_instance_id"].as_str()))
                });
                if candidates.is_empty() { continue; }
                let fingerprint=json!({"skill_id":skill,"caster_position":caster["position"],"readiness":readiness,
                    "range":range,"candidates":candidates.iter().map(|unit|json!({"id":unit["unit_instance_id"],
                        "position":unit["position"],"health":unit["stats"]["current_health"]})).collect::<Vec<_>>()});
                let key=employee.as_str().ok_or("skill employee UUID")?.to_owned();
                if attempted_targets.get(&key)==Some(&fingerprint) { continue; }
                attempted_targets.insert(key,fingerprint);
                trace.command(host,json!({"type":"begin_skill_targeting","employee_uuid":employee,"skill_id":skill}))?;
                let mut selected=false;
                for target_unit in candidates {
                    let target=json!({"type":"Unit","unit_instance_id":target_unit["unit_instance_id"]});
                    let preview=match trace.command(host,json!({"type":"preview_skill_target","employee_uuid":employee,
                        "skill_id":skill,"target":target})) {
                        Ok(preview)=>preview,
                        Err(error)=>{ let cleanup=close_driver_skill_focus(host,trace);
                            return Err(format!("manual skill preview failed: {error}; focus cleanup error: {:?}",cleanup.err().map(|e|e.to_string())).into()); }
                    };
                    if preview["eligibility"]["enabled"]!=true { continue; }
                    if preview["employee_uuid"]!=employee || preview["skill_id"]!=skill || preview["target"]!=target {
                        let cleanup=close_driver_skill_focus(host,trace);
                        return Err(format!("manual target preview identity mismatch; focus cleanup error: {:?}",
                            cleanup.err().map(|e|e.to_string())).into());
                    }
                    trace.record(json!({"operation":"skill_activation_selection","employee_uuid":employee,"skill_id":skill,
                        "readiness":readiness,"public_cast_target":catalog["cast_target"],"actual_cast_range":range,
                        "target_unit":target_unit,"actual_preview":preview}))?;
                    if let Err(error)=trace.command(host,json!({"type":"activate_skill","employee_uuid":employee,"skill_id":skill,"target":target})) {
                        let cleanup=close_driver_skill_focus(host,trace);
                        return Err(format!("manual skill activation failed: {error}; focus cleanup error: {:?}",cleanup.err().map(|e|e.to_string())).into());
                    }
                    selected=true; break;
                }
                close_driver_skill_focus(host,trace)?;
                if !selected { trace.record(json!({"operation":"skill_targeting_no_eligible_candidate",
                    "employee_uuid":employee,"skill_id":skill,"focus_closed":true}))?; }
            }
            _=>{},
        }
    }
    // BeginSkillTargeting pauses through the product's focus contract. Every
    // successful path has activated or canceled before the next unchanged tick.
    if host.app().has_active_battle() {
        let payload=host.app().active_battle_reconnect(trace.after)?.payload().as_value().clone();
        check(payload["battle_update"]["checkpoint"]["tactical_focus"].is_null(),"skill focus must be closed before host tick")?;
    }
    Ok(())
}

fn battle(host: &mut impl Host, trace: &mut Trace) -> Result<()> {
    let run=snapshot(host,ApplicationSnapshotKind::Run)?;
    let id=&run["game_state_context"]["battle_uuid"];
    if trace.battle_id.as_ref()!=Some(id) {trace.battle_id=Some(id.clone());trace.after=0;}
    let payload = host.app().active_battle_reconnect(trace.after)?.payload().as_value().clone();
    let setup = &payload["battle_setup_snapshot"];
    trace.record(json!({"operation":"battle_entry","payload":payload}))?;
    let routes = array(&setup["routes"], "battle routes")?;
    let mut route_targets = Vec::new();
    for route in routes {
        let endpoint = &route["end"];
        check(endpoint["x"].is_i64() && endpoint["y"].is_i64(), "route endpoint coordinates")?;
        if !route_targets.contains(endpoint) { route_targets.push(endpoint.clone()); }
    }
    for route in routes {
        for cell in array(&route["cells"], "route cells")?.iter().rev() {
            if !route_targets.contains(cell) { route_targets.push(cell.clone()); }
        }
    }
    let roster = snapshot(host, ApplicationSnapshotKind::EmployeeRoster)?;
    let mut employees: Vec<_> = array(&roster["employees"], "employees")?.iter().collect();
    employees.sort_by_key(|e| match e["role"].as_str() {Some("Tank")=>0,Some("Dealer")=>1,_=>2});
    let mut plan: Option<JointDeploymentPlan> = None;
    let mut deployed_employees = BTreeSet::new();
    let automatic_players=payload["battle_update"]["checkpoint"]["units"].as_array()
        .map(|units|units.iter().filter(|unit|unit["owner"]=="Player" && unit["role"]=="combatant").count()).unwrap_or(0);
    let manual_deployment=!payload["battle_update"]["checkpoint"]["deployment"].is_null();
    let mut formation_incomplete = true;
    let mut live_mode = route_targets.is_empty();
    let mut last_search_key = None;
    let mut next_search_tick = 0;
    let mut next_action_tick = 0;
    let mut attempted_skill_targets = std::collections::BTreeMap::new();
    let mut attempted_formations = BTreeSet::new();
    let mut previous_opponents = Value::Null;
    let mut stable_ticks = 0usize;
    let mut last_diagnostic = json!({"reason":"awaiting_initial_formation"});
    let mut recorded_diagnostic = Value::Null;
    // Observation cadence limits read-only search traffic. It does not change
    // elapsed host ticks, playback, product cooldowns, or the diagnostic ceiling.
    const OBSERVATION_TICKS: usize = 10;
    for index in 0..3000 {
        if !host.app().has_active_battle() {
            check(deployed_employees.len()+automatic_players >= 3, "battle ended before three real player combatants were deployed")?;
            return Ok(());
        }
        if manual_deployment {
            let payload = host.app().active_battle_reconnect(trace.after)?.payload().as_value().clone();
            let checkpoint = &payload["battle_update"]["checkpoint"];
            let units = array(&checkpoint["units"],"current tactical units")?;
            let opponents = json!(units.iter().filter(|unit|living(unit,"Opponent")).map(|unit|json!({
                "id":unit["unit_instance_id"],"position":unit["position"],"world_position":unit["world_position"],
                "health":unit["stats"]["current_health"]})).collect::<Vec<_>>());
            if opponents == previous_opponents { stable_ticks=stable_ticks.saturating_add(1); }
            else { stable_ticks=0; previous_opponents=opponents; }
            let targets = live_targets(checkpoint)?;
            let attackers: Vec<_> = employees.iter().filter(|employee|matches!(employee["role"].as_str(),Some("Tank"|"Dealer")))
                .filter_map(|employee|actor(checkpoint,&employee["uuid"])).collect();
            let outside_all_attackers = !targets.is_empty() && !attackers.iter()
                .any(|unit|targets.iter().any(|target|covers(&unit["range_previews"]["basic_attack"],target)));
            let forced_defense = units.iter().filter(|unit|living(unit,"Player")).any(|unit|
                unit["tactical_state"]["forced_defense_remaining_ms"].as_u64().is_some_and(|remaining|remaining>0));
            if !formation_incomplete && outside_all_attackers && !forced_defense && stable_ticks >= OBSERVATION_TICKS {
                formation_incomplete=true; live_mode=true;
                last_diagnostic=json!({"reason":"stationary_live_opponents_outside_actual_attacker_ranges","opponents":previous_opponents,
                    "stable_observation_ticks":stable_ticks});
            }
            if formation_incomplete && plan.is_none() && index >= next_search_tick {
                let (contacts,source) = if live_mode {(targets,"live_opponent")} else {(route_targets.clone(),"returned_route")};
                if contacts.is_empty() {
                    last_diagnostic=json!({"reason":"waiting_for_actual_delayed_opponents"});
                } else {
                    let key=planning_key(checkpoint,&contacts,source)?;
                    if last_search_key.as_ref()!=Some(&key) {
                        plan=joint_deployment_plan(host,trace,&employees,setup,checkpoint,&contacts,source)?;
                        if let Some(candidate)=&plan {
                            if !attempted_formations.insert(formation_fingerprint(checkpoint,candidate)?) {
                                trace.record(json!({"operation":"repeated_formation_rejected","contact":candidate.contact,
                                    "reason":"same_opponent_state_and_final_poses_already_attempted"}))?;
                                plan=None;
                            }
                        }
                        next_search_tick=index+OBSERVATION_TICKS;
                        last_search_key=Some(key);
                        if plan.is_none() { last_diagnostic=json!({"reason":"no_joint_formation_for_current_public_state","targets":contacts,
                            "target_source":source,"details_operation":"joint_formation_unavailable"}); }
                    }
                }
            }
            if index >= next_action_tick {
              if let Some(current) = &mut plan {
                match advance_formation(host,trace,current,&mut deployed_employees)? {
                    FormationProgress::Complete => {
                        last_diagnostic=json!({"reason":"joint_formation_complete","contact":current.contact,"target_source":current.target_source});
                        formation_incomplete=false; plan=None;
                    }
                    FormationProgress::Waiting(reason) => {
                        // Current-state eligibility/resource checks issue no
                        // commands. A disabled final preview is retried only at
                        // the observation cadence, never on every host tick.
                        if reason["reason"] == "fresh_preview_unavailable" { next_action_tick=index+OBSERVATION_TICKS; }
                        last_diagnostic=reason;
                    }
                    FormationProgress::Replan(reason) => { last_diagnostic=reason; plan=None; }
                }
              }
            }
            if last_diagnostic!=recorded_diagnostic {
                trace.record(json!({"operation":"formation_progress","battle_tick":index,"diagnostic":last_diagnostic}))?;
                recorded_diagnostic=last_diagnostic.clone();
            }
            if !formation_incomplete {
                use_ready_skills(host,trace,&mut attempted_skill_targets)?;
            }
        }
        // A diagnostic ceiling, not a balance claim: 300 simulated seconds
        // per battle. Every actual tick is retained; unresolved battles fail.
        let result = host.tick(100, trace.after);
        trace.ticks += 1;
        match result {
            Ok(value) => { if let Some(value)=&value { trace.observe(value); }
                trace.record(json!({"operation":"host_tick","battle_tick":index,"delta_ms":100,"result":value}))?; }
            Err(error) => { trace.record(json!({"operation":"host_tick","battle_tick":index,"error":error.to_string()}))?; return Err(error); }
        }
    }
    trace.record(json!({"operation":"battle_diagnostic_ceiling","last_formation_diagnostic":last_diagnostic,
        "formation_incomplete":formation_incomplete,"actual_deployed_employees":deployed_employees}))?;
    Err(format!("battle exceeded diagnostic ceiling without a terminal outcome; last formation diagnostic: {last_diagnostic}").into())
}

// A losing public strategy is separate from the three-combatant victory driver.
// Deploy one actual starter dealer facing away from the authored route, then let the
// unchanged simulation decide the result. No outcome or condition is injected.
fn defeat_then_abandon_and_depart(host: &mut impl Host, trace: &mut Trace) -> Result<Value> {
    let before = snapshot(host, ApplicationSnapshotKind::PlayerProfile)?;
    check(before["lifecycle"]["active_expedition"].is_object()
        && before["lifecycle"]["vault"].is_object()
        && before["lifecycle"]["first_return_story"].is_string(), "actual active lifecycle required")?;
    let old_run = snapshot(host, ApplicationSnapshotKind::Run)?;
    let old_employees: BTreeSet<_> = array(&old_run["roster"]["employees"], "old roster")?.iter()
        .map(|e| e["uuid"].as_str().map(str::to_owned).ok_or("old employee UUID"))
        .collect::<std::result::Result<_, _>>()?;
    check(old_employees.len() == 3, "defeat route requires three selected starters")?;
    let id = &old_run["game_state_context"]["battle_uuid"];
    if trace.battle_id.as_ref() != Some(id) { trace.battle_id = Some(id.clone()); trace.after = 0; }
    let payload = host.app().active_battle_reconnect(trace.after)?.payload().as_value().clone();
    trace.record(json!({"operation":"defeat_battle_entry", "payload":payload}))?;
    let setup = &payload["battle_setup_snapshot"];
    let checkpoint = &payload["battle_update"]["checkpoint"];
    let roster = snapshot(host, ApplicationSnapshotKind::EmployeeRoster)?;
    let employee = array(&roster["employees"], "defeat starters")?.iter()
        .find(|e| e["role"] == "Dealer").ok_or("actual starter employee")?;
    let affinity = employee["combat_profile"]["effective_deployment_affinity"].as_str().ok_or("employee affinity")?;
    let endpoint = &array(&setup["routes"], "defeat routes")?.first().ok_or("authored enemy route")?["end"];
    let route_cells: Vec<_> = array(&setup["routes"], "defeat routes")?.iter()
        .map(|route| array(&route["cells"], "authored route cells"))
        .collect::<Result<Vec<_>>>()?.into_iter().flatten().collect();
    check(!route_cells.is_empty(), "nonempty authored enemy route required")?;
    let mut cells: Vec<_> = array(&setup["deployment_zones"], "defeat zones")?.iter()
        .filter(|zone| matches!((affinity, zone["kind"].as_str()),
            ("any", Some("Ground"|"Platform")) | ("ground_only", Some("Ground")) | ("platform_only", Some("Platform"))))
        .filter_map(|zone| zone["cells"].as_array()).flatten()
        .filter(|cell| !slot_occupied(checkpoint, cell, &employee["uuid"]))
        .collect();
    cells.sort_by_key(|cell| (distance(cell, endpoint), xy(cell)));
    cells.dedup();
    let mut placement = None;
    for cell in cells {
        let preview = trace.command(host, json!({"type":"request_deployment_range_preview",
            "employee_uuid":employee["uuid"], "position":cell}))?;
        if preview["deploy_eligibility"]["enabled"] != true { continue; }
        if let Some(facing) = ["up", "right", "down", "left"].into_iter().find(|facing|
            preview["facings"][*facing]["range_previews"]["basic_attack"]["available"] == true
                && !covers(&preview["facings"][*facing]["range_previews"]["basic_attack"], endpoint)
                && !route_cells.iter()
                    .any(|cell| covers(&preview["facings"][*facing]["range_previews"]["basic_attack"], cell))) {
            placement = Some((cell.clone(), facing));
            trace.record(json!({"operation":"defeat_deployment_selection", "employee_uuid":employee["uuid"],
                "role":"Dealer", "position":cell, "facing":facing, "endpoint":endpoint, "preview":preview}))?;
            break;
        }
    }
    let (position, facing) = placement.ok_or("no currently legal public employee deployment")?;
    trace.command(host, json!({"type":"deploy_unit", "employee_uuid":employee["uuid"], "position":position, "facing":facing}))?;
    for index in 0..3000 {
        if !host.app().has_active_battle() { break; }
        let result = host.tick(100, trace.after);
        trace.ticks += 1;
        match result {
            Ok(value) => {
                if let Some(value) = &value { trace.observe(value); }
                trace.record(json!({"operation":"defeat_host_tick", "battle_tick":index, "delta_ms":100, "result":value}))?;
            }
            Err(error) => {
                trace.record(json!({"operation":"defeat_host_tick", "battle_tick":index, "error":error.to_string()}))?;
                return Err(error);
            }
        }
    }
    check(!host.app().has_active_battle(), "losing strategy exceeded unchanged 300-second diagnostic ceiling")?;
    let defeated = snapshot(host, ApplicationSnapshotKind::Run)?;
    check(defeated["game_state"] == "combat_result" && defeated["selected_event"]["verdict"] == "defeat",
        "actual defeat result required")?;
    trace.command(host, json!({"type":"complete_combat_result"}))?;
    let after_ack = snapshot(host, ApplicationSnapshotKind::Run)?;
    let explicit_abandon = !after_ack["run_progression"].is_null();
    if explicit_abandon { trace.command(host, json!({"type":"abandon_run"}))?; }
    let terminal = snapshot(host, ApplicationSnapshotKind::PlayerProfile)?;
    check(host.app().export_active_run_checkpoint()?.is_none(), "defeat/abandon retained Continue")?;
    check(terminal["lifecycle"]["active_expedition"].is_null(), "terminal retained active expedition")?;
    for field in ["vault", "first_return_story", "expedition_sequence"] {
        check(terminal["lifecycle"][field] == before["lifecycle"][field], "loss/abandon changed persistent lifecycle field")?;
    }
    trace.record(json!({"operation":"actual_defeat_terminal", "explicit_abandon":explicit_abandon,
        "after_defeat_ack":after_ack, "profile_before":before, "profile_after":terminal}))?;
    trace.command(host, json!({"type":"start_new_game", "game_mode":"Expedition", "replace_active_run":false}))?;
    for _ in 0..128 {
        let run = snapshot(host, ApplicationSnapshotKind::Run)?;
        let vn = &run["visual_novel"];
        if !vn.is_null() {
            let command = if let Some(choice) = vn["choices"].as_array().and_then(|cs| cs.iter().find(|c| c["selectable"] == true)) {
                json!({"type":"select_visual_novel_choice", "session_id":vn["session_id"], "current_scene_id":vn["current_scene_id"], "choice_id":choice["choice_id"]})
            } else { json!({"type":"advance_visual_novel", "session_id":vn["session_id"], "current_scene_id":vn["current_scene_id"]}) };
            trace.command(host, command)?;
        } else if run["game_state"] == "selecting_starter_employees" {
            let candidates = array(&run["game_state_context"]["candidates"], "new starter candidates")?;
            check(candidates.len() == 6, "next expedition requires six fresh candidates")?;
            let ids: Vec<_> = ["Dealer", "Tank", "Healer"].iter().map(|role| candidates.iter().find(|c| c["role"] == *role)
                .map(|c| c["employee_definition_id"].clone()).ok_or("next starter role"))
                .collect::<std::result::Result<_, _>>()?;
            trace.command(host, json!({"type":"select_starter_employees", "employee_definition_ids":ids, "loan_copy_ids":[]}))?;
        } else {
            check(run["game_state"] == "viewing_map", "next expedition must reach playable map")?;
            let employees = array(&run["roster"]["employees"], "new roster")?;
            check(employees.len() == 3 && employees.iter().all(|e| e["uuid"].as_str().is_some_and(|id| !old_employees.contains(id))),
                "next expedition reused old employees")?;
            let profile = snapshot(host, ApplicationSnapshotKind::PlayerProfile)?;
            check(profile["lifecycle"]["expedition_sequence"].as_u64()
                == before["lifecycle"]["expedition_sequence"].as_u64().and_then(|n| n.checked_add(1)), "next sequence")?;
            check(profile["lifecycle"]["active_expedition"].is_object()
                && profile["lifecycle"]["active_expedition"] != before["lifecycle"]["active_expedition"], "next instance identity")?;
            for field in ["vault", "first_return_story"] {
                check(profile["lifecycle"][field] == terminal["lifecycle"][field], "next departure changed no-loan lifecycle")?;
            }
            trace.restart(host)?;
            return Ok(json!({"boundary":"defeat_then_abandon", "actual_defeat":true, "explicit_abandon":explicit_abandon,
                "next_expedition_playable":true, "next_profile":profile, "full_lifecycle_complete":false}));
        }
    }
    Err("next departure exceeded finite briefing bound".into())
}

// Separate losing strategy: all three selected non-healers are actually defeated.
// The first-battle boundary observes this prerequisite before the longer failure route.
fn lose_all_selected_starters(host: &mut impl Host, trace: &mut Trace) -> Result<()> {
    let run = snapshot(host, ApplicationSnapshotKind::Run)?;
    let id = &run["game_state_context"]["battle_uuid"];
    if trace.battle_id.as_ref() != Some(id) { trace.battle_id = Some(id.clone()); trace.after = 0; }
    let roster = snapshot(host, ApplicationSnapshotKind::EmployeeRoster)?;
    let employees = array(&roster["employees"], "failure starters")?;
    check(employees.len() == 3 && employees.iter().all(|e| e["role"] != "Healer"),
        "failure route requires three real non-healer starters")?;
    let mut payload = host.app().active_battle_reconnect(trace.after)?.payload().as_value().clone();
    trace.record(json!({"operation":"failure_battle_entry", "payload":payload, "roster":roster}))?;
    let costs = array(&payload["battle_update"]["checkpoint"]["deployment"]["unit_deploy_costs"], "actual deployment costs")?;
    let required_tp = employees.iter().try_fold(0u64, |sum, employee| -> Result<u64> {
        let cost = costs.iter().find(|cost| cost["employee_uuid"] == employee["uuid"])
            .and_then(|cost| cost["effective_deploy_cost"].as_u64()).ok_or("actual starter deployment cost")?;
        sum.checked_add(cost).ok_or_else(|| "deployment cost overflow".into())
    })?;
    let mut elapsed_ticks = 0usize;
    while payload["battle_update"]["checkpoint"]["tactical_resource"]["current"].as_u64().ok_or("current TP")? < required_tp {
        check(elapsed_ticks < 3000, "failure preparation exceeded existing battle ceiling")?;
        let value = host.tick(100, trace.after)?;
        trace.ticks += 1; elapsed_ticks += 1;
        if let Some(value) = &value { trace.observe(value); }
        trace.record(json!({"operation":"failure_tp_tick", "delta_ms":100, "result":value}))?;
        payload = host.app().active_battle_reconnect(trace.after)?.payload().as_value().clone();
    }
    let setup = payload["battle_setup_snapshot"].clone();
    let endpoint = &array(&setup["routes"], "failure routes")?.first().ok_or("authored enemy route")?["end"];
    let route_cells: Vec<_> = array(&setup["routes"], "failure routes")?.iter()
        .map(|route| array(&route["cells"], "authored route cells"))
        .collect::<Result<Vec<_>>>()?.into_iter().flatten().collect();
    check(!route_cells.is_empty(), "nonempty authored enemy route required")?;
    for employee in employees {
        let payload = host.app().active_battle_reconnect(trace.after)?.payload().as_value().clone();
        let checkpoint = &payload["battle_update"]["checkpoint"];
        let affinity = employee["combat_profile"]["effective_deployment_affinity"].as_str().ok_or("employee affinity")?;
        let mut cells: Vec<_> = array(&setup["deployment_zones"], "failure zones")?.iter()
            .filter(|zone| matches!((affinity, zone["kind"].as_str()),
                ("any", Some("Ground"|"Platform")) | ("ground_only", Some("Ground")) | ("platform_only", Some("Platform"))))
            .filter_map(|zone| zone["cells"].as_array()).flatten()
            .filter(|cell| !slot_occupied(checkpoint, cell, &employee["uuid"]) && !route_cells.contains(cell) && *cell != endpoint)
            .collect();
        cells.sort_by_key(|cell| (distance(cell, endpoint), xy(cell)));
        cells.dedup();
        let mut placement = None;
        for cell in cells {
            let preview = trace.command(host, json!({"type":"request_deployment_range_preview",
                "employee_uuid":employee["uuid"], "position":cell}))?;
            if preview["deploy_eligibility"]["enabled"] != true { continue; }
            if let Some(facing) = ["up", "right", "down", "left"].into_iter().find(|facing|
                preview["facings"][*facing]["range_previews"]["basic_attack"]["available"] == true
                    && !covers(&preview["facings"][*facing]["range_previews"]["basic_attack"], endpoint)
                    && !route_cells.iter().any(|cell| covers(&preview["facings"][*facing]["range_previews"]["basic_attack"], cell))) {
                placement = Some((cell.clone(), facing));
                trace.record(json!({"operation":"failure_deployment_selection", "employee_uuid":employee["uuid"],
                    "role":employee["role"], "position":cell, "facing":facing, "endpoint":endpoint, "preview":preview}))?;
                break;
            }
        }
        let (position, facing) = placement.ok_or("no legal losing placement for a selected starter")?;
        trace.command(host, json!({"type":"deploy_unit", "employee_uuid":employee["uuid"], "position":position, "facing":facing}))?;
    }
    for index in elapsed_ticks..3000 {
        if !host.app().has_active_battle() { break; }
        let result = host.tick(100, trace.after);
        trace.ticks += 1;
        match result {
            Ok(value) => {
                if let Some(value) = &value { trace.observe(value); }
                trace.record(json!({"operation":"failure_host_tick", "battle_tick":index, "delta_ms":100, "result":value}))?;
            }
            Err(error) => {
                trace.record(json!({"operation":"failure_host_tick", "battle_tick":index, "error":error.to_string()}))?;
                return Err(error);
            }
        }
    }
    check(!host.app().has_active_battle(), "losing strategy exceeded unchanged 300-second diagnostic ceiling")?;
    let defeated = snapshot(host, ApplicationSnapshotKind::Run)?;
    check(defeated["game_state"] == "combat_result" && defeated["selected_event"]["verdict"] == "defeat",
        "actual three-starter defeat result required")?;
    trace.command(host, json!({"type":"complete_combat_result"}))?;
    let after = snapshot(host, ApplicationSnapshotKind::EmployeeRoster)?;
    let survivors = array(&after["employees"], "post-defeat employees")?;
    if !survivors.is_empty() {
        check(survivors.len() == employees.len(), "defeat changed roster membership before terminal")?;
        for employee in employees {
            let observed = survivors.iter().find(|e| e["uuid"] == employee["uuid"]).ok_or("same injured employee")?;
            let previous_damage = employee["condition"]["physical_damage"].as_u64().ok_or("previous physical damage")?;
            let current_damage = observed["condition"]["physical_damage"].as_u64().ok_or("current physical damage")?;
            check(current_damage > previous_damage,
                "every selected employee must actually incur incapacitation damage")?;
        }
    } else {
        check(snapshot(host, ApplicationSnapshotKind::Run)?["game_state"] == "run_failed",
            "only committed run failure may clear the defeated roster")?;
    }
    trace.record(json!({"operation":"actual_three_starter_defeat", "before":roster, "after":after,
        "run":snapshot(host, ApplicationSnapshotKind::Run)?}))?;
    Ok(())
}

fn depart_after_confirmed_failure(host: &mut impl Host, trace: &mut Trace, before: &Value, old_employees: &BTreeSet<String>) -> Result<Value> {
    let after_ack = snapshot(host, ApplicationSnapshotKind::Run)?;
    check(after_ack["game_state"] == "run_failed", "actual committed RunFailed required")?;
    let terminal = snapshot(host, ApplicationSnapshotKind::PlayerProfile)?;
    check(host.app().export_active_run_checkpoint()?.is_none(), "defeat/abandon retained Continue")?;
    check(terminal["lifecycle"]["active_expedition"].is_null(), "terminal retained active expedition")?;
    for field in ["vault", "first_return_story", "expedition_sequence"] {
        check(terminal["lifecycle"][field] == before["lifecycle"][field], "loss/abandon changed persistent lifecycle field")?;
    }
    trace.record(json!({"operation":"actual_run_failure_terminal", "explicit_abandon":false,
        "after_defeat_ack":after_ack, "profile_before":before, "profile_after":terminal}))?;
    trace.command(host, json!({"type":"start_new_game", "game_mode":"Expedition", "replace_active_run":false}))?;
    for _ in 0..128 {
        let run = snapshot(host, ApplicationSnapshotKind::Run)?;
        let vn = &run["visual_novel"];
        if !vn.is_null() {
            let command = if let Some(choice) = vn["choices"].as_array().and_then(|cs| cs.iter().find(|c| c["selectable"] == true)) {
                json!({"type":"select_visual_novel_choice", "session_id":vn["session_id"], "current_scene_id":vn["current_scene_id"], "choice_id":choice["choice_id"]})
            } else { json!({"type":"advance_visual_novel", "session_id":vn["session_id"], "current_scene_id":vn["current_scene_id"]}) };
            trace.command(host, command)?;
        } else if run["game_state"] == "selecting_starter_employees" {
            let candidates = array(&run["game_state_context"]["candidates"], "new starter candidates")?;
            check(candidates.len() == 6, "next expedition requires six fresh candidates")?;
            let ids: Vec<_> = ["Dealer", "Tank", "Healer"].iter().map(|role| candidates.iter().find(|c| c["role"] == *role)
                .map(|c| c["employee_definition_id"].clone()).ok_or("next starter role"))
                .collect::<std::result::Result<_, _>>()?;
            trace.command(host, json!({"type":"select_starter_employees", "employee_definition_ids":ids, "loan_copy_ids":[]}))?;
        } else {
            check(run["game_state"] == "viewing_map", "next expedition must reach playable map")?;
            let employees = array(&run["roster"]["employees"], "new roster")?;
            check(employees.len() == 3 && employees.iter().all(|e| e["uuid"].as_str().is_some_and(|id| !old_employees.contains(id))),
                "next expedition reused old employees")?;
            let profile = snapshot(host, ApplicationSnapshotKind::PlayerProfile)?;
            check(profile["lifecycle"]["expedition_sequence"].as_u64()
                == before["lifecycle"]["expedition_sequence"].as_u64().and_then(|n| n.checked_add(1)), "next sequence")?;
            check(profile["lifecycle"]["active_expedition"].is_object()
                && profile["lifecycle"]["active_expedition"] != before["lifecycle"]["active_expedition"], "next instance identity")?;
            for field in ["vault", "first_return_story"] {
                check(profile["lifecycle"][field] == terminal["lifecycle"][field], "next departure changed no-loan lifecycle")?;
            }
            trace.restart(host)?;
            return Ok(json!({"boundary":"failure_then_depart", "actual_defeat":true, "actual_run_failure":true, "explicit_abandon":false,
                "next_expedition_playable":true, "next_profile":profile, "full_lifecycle_complete":false}));
        }
    }
    Err("next departure exceeded finite briefing bound".into())
}

fn event_command(selected: &Value) -> Result<Value> {
    let event = &selected["event"];
    let mut command = json!({"node_id":selected["node_id"],"event_id":event["event_id"],"current_scene_id":event["current_scene_id"]});
    match event["interaction"]["type"].as_str() {
        Some("advance") => command["type"] = json!("advance_event_scene"),
        Some("relic_corruption") => { check(event["interaction"]["can_decline"] == true,"event cannot decline")?;
            command["type"]=json!("choose_relic_corruption"); command["copy_uuid"]=Value::Null; }
        Some("employee_check_choices") => {
            let pair = array(&event["interaction"]["choices"],"event choices")?.iter()
                .filter(|c| c["eligibility"]["enabled"]==true)
                .find_map(|c| c["employee_candidates"].as_array().and_then(|employees| employees.iter()
                    .filter(|e| e["eligibility"]["enabled"]==true).max_by_key(|e| e["success_chance_percent"].as_u64()).map(|e|(c,e))));
            let (choice,employee)=pair.ok_or("no eligible event choice")?;
            command["type"]=json!("resolve_event_choice_check"); command["choice_id"]=choice["choice_id"].clone(); command["employee_uuid"]=employee["employee_uuid"].clone();
        }
        _ => return Err("unknown returned event interaction".into()),
    }
    Ok(command)
}

fn expected_return_vault(run: &Value, profile: &Value) -> Result<Value> {
    let mut vault = profile["lifecycle"]["vault"].clone();
    for item in array(&run["inventory"]["equipments"],"inventory equipment")? {
        let data=&item["item"]; let id=item["instance_uuid"].as_str().ok_or("equipment identity")?;
        vault["equipment"][id]=json!({"copy_uuid":id,"definition_id":data["definition_id"],
            "equipment_type":data["equipment_type"],"enhancement_level":if data["equipment_type"]=="Weapon" {json!(0)} else {data["enhancement_level"].clone()}});
    }
    for item in array(&run["inventory"]["skill_fragments"],"inventory fragments")? {
        let mut copy=item["copy"].clone(); let id=copy["copy_uuid"].as_str().ok_or("fragment identity")?.to_owned();
        copy.as_object_mut().ok_or("fragment object")?.remove("equipped_to"); vault["fragments"][id]=copy;
    }
    Ok(vault)
}

/// `boundary`: hq, battle, save, floor, complete, defeat_then_abandon, failure_battle, or failure_then_depart. Each early boundary is
/// useful prerequisite evidence and never reports the full lifecycle complete.
pub fn walk(host: &mut impl Host, mode: &str, boundary: &str, trace_path: &Path) -> Result<Value> {
    check(matches!(mode,"Tutorial"|"Expedition"),"current mode required")?;
    check(matches!(boundary,"hq"|"battle"|"save"|"floor"|"complete"|"defeat_then_abandon"|"failure_battle"|"failure_then_depart"),"known boundary required")?;
    check(!matches!(boundary,"defeat_then_abandon"|"failure_battle"|"failure_then_depart") || mode == "Expedition", "loss branch requires Expedition")?;
    let mut trace=Trace {file:fs::OpenOptions::new().create_new(true).write(true).open(trace_path)?,commands:0,ticks:0,after:0,battle_id:None};
    let result=walk_inner(host,mode,boundary,&mut trace);
    trace.record(json!({"operation":"driver_exit","success":result.is_ok(),"error":result.as_ref().err().map(ToString::to_string),
        "requested_mode":mode,"requested_boundary":boundary,"commands":trace.commands,"ticks":trace.ticks,
        "run":snapshot(host,ApplicationSnapshotKind::Run)?,"profile":snapshot(host,ApplicationSnapshotKind::PlayerProfile)?}))?;
    result
}
fn walk_inner(host: &mut impl Host, mode: &str, boundary: &str, trace: &mut Trace) -> Result<Value> {
    let profile_before=snapshot(host,ApplicationSnapshotKind::PlayerProfile)?;
    let mut seen_floors=BTreeSet::new(); let mut battle_count=0usize; let mut save_count=0usize;
    let mut hq_count=0usize; let mut expected_vault=None;
    let failure_route = matches!(boundary,"failure_battle"|"failure_then_depart");
    let mut failure_before = None; let mut failure_employees = BTreeSet::new(); let mut failure_confirmed = false;
    let initial=snapshot(host,ApplicationSnapshotKind::Run)?;
    let started_new=matches!(initial["game_state"].as_str(),Some("not_started"|"run_complete"|"run_failed"|"game_over"));
    let initial_floor=if started_new {0} else {initial["run_progression"]["floor_index"].as_u64().ok_or("active floor")?};
    if started_new {
        trace.command(host,json!({"type":"start_new_game","game_mode":mode,"replace_active_run":false}))?;
    } else {
        check(initial["run_progression"]["game_mode"]==mode,"restored mode differs from requested mode")?;
    }
    for _ in 0..512 {
        let run=snapshot(host,ApplicationSnapshotKind::Run)?;
        if mode=="Tutorial" { check(snapshot(host,ApplicationSnapshotKind::PlayerProfile)?==profile_before,"Tutorial changed persistent profile")?; }
        let state=run["game_state"].as_str().ok_or("game state")?;
        if let Some(floor)=run["run_progression"]["floor_index"].as_u64() {
            if state=="viewing_map" && seen_floors.insert(floor) {
                trace.restart(host)?;
                if floor>initial_floor && boundary=="floor" { return Ok(json!({"boundary":"floor","floor_index":floor,"floors":seen_floors,"full_lifecycle_complete":false})); }
            }
        }
        if state=="run_complete" || (mode=="Expedition" && expected_vault.is_some() && run["run_progression"].is_null()) {
            check(host.app().export_active_run_checkpoint()?.is_none(),"terminal retained Continue checkpoint")?;
            if mode=="Tutorial" { check(seen_floors==(initial_floor..=2).collect(),"Tutorial did not traverse every remaining floor")?; check(battle_count>0,"Tutorial has no real battles")?; }
            if let Some(vault)=&expected_vault { check(snapshot(host,ApplicationSnapshotKind::PlayerProfile)?["lifecycle"]["vault"]==*vault,"return vault differs from exact active copy ledger")?; }
            return Ok(json!({"boundary":"complete","mode":mode,"terminal_reached":true,"initial_floor":initial_floor,"full_lifecycle_complete":started_new && initial_floor==0,
                "floors":seen_floors,"battles":battle_count,"save_points":save_count,"hq_sessions":hq_count}));
        }
        if failure_route && state == "awaiting_run_failure_decision" {
            trace.command(host,json!({"type":"confirm_run_failure"}))?;
            failure_confirmed = true; continue;
        }
        if failure_route && state == "run_failed" {
            let mut result = depart_after_confirmed_failure(host,trace,failure_before.as_ref().ok_or("failure route never entered a real battle")?,&failure_employees)?;
            result["failure_confirmed"] = json!(failure_confirmed);
            result["defeated_battles"] = json!(battle_count);
            return Ok(result);
        }
        check(!matches!(state,"run_failed"|"game_over"|"awaiting_run_failure_decision"),"normal route reached failure; inspect preserved trace")?;
        let vn=&run["visual_novel"];
        let command=if !vn.is_null() {
            if let Some(choice)=vn["choices"].as_array().and_then(|cs|cs.iter().find(|c|c["selectable"]==true)) {
                json!({"type":"select_visual_novel_choice","session_id":vn["session_id"],"current_scene_id":vn["current_scene_id"],"choice_id":choice["choice_id"]})
            } else { json!({"type":"advance_visual_novel","session_id":vn["session_id"],"current_scene_id":vn["current_scene_id"]}) }
        } else if !run["run_progression"]["pending_continuation"].is_null() && run["run_progression"].is_object() {
            let depth=run["run_progression"]["floor_index"].as_u64().ok_or("floor")?+1;
            let returning=depth>=3 && !failure_route;
            if returning { expected_vault=Some(expected_return_vault(&run,&snapshot(host,ApplicationSnapshotKind::PlayerProfile)?)?); }
            json!({"type":"choose_expedition_continuation","choice":if returning {"Return"} else {"Continue"}})
        } else { match state {
            "selecting_starter_employees" => {
                let cs=array(&run["game_state_context"]["candidates"],"starter candidates")?;
                check(cs.len()==6,"six starters required")?;
                let ids:Vec<_> = if failure_route {
                    let dealers:Vec<_> = cs.iter().filter(|c| c["role"]=="Dealer").collect();
                    check(dealers.len()==2,"two official Dealer definitions required")?;
                    let tank=cs.iter().find(|c| c["role"]=="Tank").ok_or("official Tank definition")?;
                    dealers.into_iter().chain(std::iter::once(tank)).map(|c| c["employee_definition_id"].clone()).collect()
                } else {
                    ["Dealer","Tank","Healer"].iter().map(|role| cs.iter().find(|c| c["role"]==*role)
                        .map(|c|c["employee_definition_id"].clone()).ok_or("missing starter role")).collect::<std::result::Result<_,_>>()?
                };
                json!({"type":"select_starter_employees","employee_definition_ids":ids,"loan_copy_ids":[]})
            }
            "viewing_map" => {
                // Failure preparation follows the actual selectable ordinary combat
                // when an endless floor replaces the initial authored route.
                // Boss nodes and unavailable nodes are never used as a shortcut.
                if failure_route {
                    let ids=array(&run["map_navigation"]["selectable_node_ids"],"failure selectable nodes")?;
                    if let Some(node)=array(&run["map"]["nodes"],"failure map nodes")?.iter()
                        .find(|node| node["category"]=="Combat" && node["state"]=="Available" && ids.contains(&node["id"])) {
                        trace.record(json!({"operation":"failure_selectable_combat", "template":run["map"]["map_template_id"], "node":node}))?;
                        trace.command(host,json!({"type":"select_map_node","node_id":node["id"]}))?;
                        continue;
                    }
                }
                let order:&[&str]=match run["map"]["map_template_id"].as_str() {
                    Some("expedition_development_v1")=>&["entry","hq","combat","save","reward","gate"],
                    Some("standard_intro_a")=>&["entry","intake_corridor","supply_annex","containment_guard","floor_gate"],
                    Some("standard_branching_a")=>&["entry","dispatch_hall","save_archive","hq_terminal","crossroads_archive","containment_guard","floor_gate"],
                    Some("standard_final_a")=>&["entry","east_containment","maintenance_core","supply_core","upper_containment","final_approach","final_chamber"],
                    _=>return Err("unexpected official route template".into()),
                };
                let ids=array(&run["map_navigation"]["selectable_node_ids"],"selectable nodes")?;
                let nodes=array(&run["map"]["nodes"],"map nodes")?;
                let node=order.iter().find_map(|slot|nodes.iter().find(|n| n["slot_id"]==*slot && n["state"]=="Available" && ids.contains(&n["id"])))
                    .ok_or("declared official route has no selectable next node")?;
                json!({"type":"select_map_node","node_id":node["id"]})
            }
            "node_confirm"=>json!({"type":"confirm_enter_node"}),
            "in_battle"=>{
                if boundary == "defeat_then_abandon" { return defeat_then_abandon_and_depart(host, trace); }
                if failure_route {
                    if failure_before.is_none() {
                        failure_before=Some(snapshot(host,ApplicationSnapshotKind::PlayerProfile)?);
                        failure_employees=array(&run["roster"]["employees"],"failure roster")?.iter()
                            .map(|e| e["uuid"].as_str().map(str::to_owned).ok_or("failure employee UUID"))
                            .collect::<std::result::Result<_,_>>()?;
                    }
                    lose_all_selected_starters(host,trace)?; battle_count+=1;
                    if boundary=="failure_battle" { return Ok(json!({"boundary":"failure_battle","actual_defeat":true,
                        "defeated_battles":battle_count,"all_three_incapacitated":true,"full_lifecycle_complete":false})); }
                    continue;
                }
                battle(host,trace)?;battle_count+=1;if boundary=="battle" {return Ok(json!({"boundary":"battle","full_lifecycle_complete":false,"battles":battle_count}));}continue;}
            "combat_result"=>{
                check(run["selected_event"]["verdict"]=="victory","actual battle did not win")?;
                json!({"type":"complete_combat_result"})
            }
            "in_reward"=>{let selected=&run["selected_event"];
                if selected["mode"]=="ChooseOne" && selected["selected_reward_uuid"].is_null() {
                    let reward=array(&selected["rewards"],"rewards")?.first().ok_or("no reward candidate")?;
                    json!({"type":"select_reward","reward_id":reward["uuid"]})
                } else {json!({"type":"claim_reward"})}}
            "in_reward_claimed"=>json!({"type":"exit_reward"}),
            "in_shop"=>json!({"type":"exit_shop"}),
            "in_node"=>{match run["selected_event"]["type"].as_str() {
                Some("event")=>event_command(&run["selected_event"] )?,
                Some("headquarters_contact")=>{trace.command(host,json!({"type":"request_emergency_supplies"}))?;hq_count+=1;
                    if boundary=="hq" {return Ok(json!({"boundary":"hq","full_lifecycle_complete":false,"hq_sessions":hq_count,"run":snapshot(host,ApplicationSnapshotKind::Run)?}));} continue;}
                Some("save_point")=>{trace.command(host,json!({"type":"complete_node"}))?;save_count+=1;trace.restart(host)?;
                    if boundary=="save" {return Ok(json!({"boundary":"save","full_lifecycle_complete":false,"save_points":save_count,"battles":battle_count}));}continue;}
                _=>json!({"type":"complete_node"}),
            }}
            _=>return Err(format!("unhandled public phase {state}").into()),
        }};
        trace.command(host,command)?;
    }
    Err("route exceeded 512 public decisions; inspect trace".into())
}
