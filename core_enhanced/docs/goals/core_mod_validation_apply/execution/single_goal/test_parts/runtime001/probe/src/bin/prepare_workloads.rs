//! Fixed full-window workload proposals. Counts are independent input choices;
//! actual preparation receipts and actors are reported separately. No criterion
//! is selected and no failed scale is silently reduced to a successful one.
use game_core::{authoring::{CancellationToken,LabPlacement,LabPreparation,TrialSession,WorldLabPreparation},
    game::data::content_pack::{ContentSelection,InputLimits},
    mod_validation::{Candidate,LabProfile,LabSuite,TrustedBase},runtime_safety::RuntimeSafetySettings};
use serde_json::{json,Value};
use std::{collections::BTreeMap,error::Error,fs,path::Path,sync::Arc,time::Instant};
use u11_independent_public_probe::{normal_preparation as normal,fixture_coverage as oracle,file_state::FileState};
#[path="../workload_oracle.rs"] mod workload_oracle;
type Result<T>=std::result::Result<T,Box<dyn Error>>;
const CADENCE_LARGE_CELLS:[(i32,i32);8]=[(5,4),(6,4),(7,4),(8,4),(5,6),(6,6),(7,6),(8,6)];

#[derive(Clone,Copy,PartialEq,Eq)]
enum WindowMode { Legacy, FixedCadence }
impl WindowMode {
    fn parse(value:Option<&str>)->Result<Self>{match value {
        None|Some("legacy-single-60000")=>Ok(Self::Legacy),
        Some("fixed-cadence-60x1000")=>Ok(Self::FixedCadence),
        _=>Err("mode must be legacy-single-60000 or fixed-cadence-60x1000".into()),
    }}
    fn name(self)->&'static str {if self==Self::Legacy{"legacy-single-60000"}else{"fixed-cadence-60x1000"}}
    fn revision(self)->&'static str {if self==Self::Legacy{"u11-fixed-window-population-v2"}else{"u11-fixed-cadence-60x1000-v2"}}
    fn operations(self)->usize {if self==Self::Legacy{1}else{60}}
    fn delta_ms(self)->u64 {if self==Self::Legacy{60000}else{1000}}
    fn deployments(self)->[usize;3] {if self==Self::Legacy{[1,8,16]}else{[1,8,8]}}
    fn strict(self)->bool {self==Self::FixedCadence}
    fn large_placement_plan(self)->&'static str {
        if self.strict(){"u11-east-corridor-two-flank-v1"}else{"u11-discovery-row-priority-v1"}
    }
}

fn clock_window(trial:&mut TrialSession,mode:WindowMode,directory:&Path)->Result<()> {
    let initial_index=trial.next_step_index();
    let initial_work=trial.accepted_work();
    let initial_lifecycle=serde_json::to_value(trial.lifecycle())?;
    let mut operations=Vec::new();
    let mut peaks=BTreeMap::<String,u64>::new();
    let mut total_duration_ns=0_u64;
    let mut total_receipt_work=0_u64;
    let window_start=Instant::now();
    let execution:Result<()>=(||{
        if mode.strict()&&(initial_lifecycle["total_elapsed_ms"]!=0
            ||initial_lifecycle["current"]["last_battle_time_ms"]!=0
            ||initial_lifecycle["current"]["status"]["state"]!="running"){
            return Err("fixed cadence must begin at actual running battle time zero".into());
        }
        while !trial.is_complete(){
            let before_work=trial.accepted_work();
            let started=Instant::now();
            let advanced=trial.advance(&CancellationToken::default());
            let duration_ns=u64::try_from(started.elapsed().as_nanos())?;
            total_duration_ns=total_duration_ns.checked_add(duration_ns).ok_or("clock duration overflow")?;
            let (index,public_result,receipt)=match advanced {
                Ok(step)=>(step.index(),step.public_result().cloned(),serde_json::from_str::<Value>(step.receipt().document())?),
                Err(error)=>{
                    normal::save(&directory.join("failure_state.json"),&json!({"error":format!("{error:?}"),
                        "lifecycle":trial.lifecycle(),"accepted_work":trial.accepted_work(),"coverage":trial.lab_coverage()?,
                        "failed_operation_duration_ns":duration_ns,"failed_attempt_usage":error.attempt_usage(),
                        "accepted_clock_operations":operations.len(),"execution_mode":mode.name()}))?;
                    return Err(format!("fixed full-window execution: {error:?}").into());
                }
            };
            let ordinal=operations.len()+1;
            let lifecycle=serde_json::to_value(trial.lifecycle())?;
            // Preserve the actual accepted receipt before asserting its contract.
            normal::save(&directory.join(format!("clock_step_{}.json",trial.next_step_index())),
                &json!({"public_result":public_result,"receipt":receipt,"lifecycle":lifecycle}))?;
            let usage=receipt["usage"].as_object().ok_or("actual StepView receipt usage absent")?;
            let work=usage["work"].as_u64().ok_or("actual step work absent")?;
            let accepted_delta=trial.accepted_work().checked_sub(before_work).ok_or("accepted work decreased")?;
            if work!=accepted_delta{return Err("StepView usage and accepted-work delta disagree".into());}
            total_receipt_work=total_receipt_work.checked_add(work).ok_or("clock work overflow")?;
            for (key,value) in usage {
                let value=value.as_u64().ok_or("actual usage counter is not unsigned")?;
                let peak=peaks.entry(key.clone()).or_default();*peak=(*peak).max(value);
            }
            let expected_ms=ordinal as u64*mode.delta_ms();
            let input_matches=receipt["payload"]["request"]==json!({"kind":"clock","clock":{
                "mode":"elapsed","delta_ms":mode.delta_ms(),"after_seq":0}});
            let checkpoint_ms=public_result.as_ref().and_then(|v|v.pointer("/payload/battle_update/checkpoint/battle_time_ms")).and_then(Value::as_u64);
            let running_checkpoint=lifecycle["current"]["status"]["state"]=="running"
                &&lifecycle["current"]["last_battle_time_ms"]==expected_ms
                &&lifecycle["total_elapsed_ms"]==expected_ms&&checkpoint_ms==Some(expected_ms)
                &&lifecycle["current"]["battle_uuid"]==initial_lifecycle["current"]["battle_uuid"]
                &&lifecycle["entries"]==initial_lifecycle["entries"];
            let operation=json!({"ordinal":ordinal,"step_index":index,"duration_ns":duration_ns,
                "authoring_usage":usage,"accepted_work_delta":accepted_delta,"expected_battle_time_ms":expected_ms,
                "actual_checkpoint_time_ms":checkpoint_ms,"input_matches_fixed_contract":input_matches,
                "running_checkpoint_matches_fixed_contract":running_checkpoint,"lifecycle":lifecycle});
            operations.push(operation);
            if !input_matches||index!=initial_index+ordinal-1||ordinal>mode.operations()
                ||(mode.strict()&&!running_checkpoint){
                return Err("actual clock operation violates the precommitted cadence contract; raw receipt preserved".into());
            }
        }
        if operations.len()!=mode.operations(){return Err("actual accepted clock operation count differs from fixed input".into());}
        if trial.accepted_work().checked_sub(initial_work)!=Some(total_receipt_work){
            return Err("whole clock accepted work differs from the sum of actual step receipts".into());
        }
        Ok(())
    })();
    let observed_window_duration_ns=u64::try_from(window_start.elapsed().as_nanos())?;
    normal::save(&directory.join("actual_clock_metrics.json"),&json!({
        "execution_mode":mode.name(),"measurement_revision":mode.revision(),"completed":execution.is_ok(),
        "error":execution.as_ref().err().map(ToString::to_string),"requested_operations":mode.operations(),
        "accepted_operations":operations.len(),"fixed_delta_ms":mode.delta_ms(),"requested_total_clock_ms":60000,
        "accepted_input_clock_ms":operations.len() as u64*mode.delta_ms(),"initial_step_index":initial_index,
        "initial_authoring_work":initial_work,"final_authoring_work":trial.accepted_work(),
        "total_accepted_clock_authoring_work":total_receipt_work,"per_operation_authoring_peaks":peaks,
        "total_advance_call_duration_ns_including_failed_attempt":total_duration_ns,
        "maximum_accepted_advance_call_duration_ns":operations.iter().filter_map(|op|op["duration_ns"].as_u64()).max(),
        "observed_window_duration_ns_including_probe_evidence_io":observed_window_duration_ns,
        "paired_worker_execute_window_remains_authoritative_for_calibration":true,
        "authoring_usage_is_not_an_exact_native_fuel_measurement":true,
        "native_and_trial_limits_unchanged":true,"after_seq_zero_delivery_cost_included":true,
        "initial_lifecycle":initial_lifecycle,"final_lifecycle":trial.lifecycle(),"operations":operations}))?;
    execution
}

fn population(trial:&TrialSession)->Result<Value>{
    let reconnect=normal::reconnect(trial)?;
    let units=reconnect.pointer("/payload/battle_update/checkpoint/units").and_then(Value::as_array)
        .ok_or("real battle checkpoint units required")?;
    let observation=trial.observation()?;
    let facts=observation.active_runtime_facts().map(|facts|json!({"usage":facts.usage(),
        "actor_summaries":facts.actors().count(),"provider_summaries":facts.providers().count(),
        "active_actions":facts.active_actions().count(),"summary":facts.summary()}));
    Ok(json!({"receipt":trial.lab_preparation(),"live_battle_units":units.len(),
        "friendly_units":units.iter().filter(|u|u["owner"]=="Player").count(),
        "enemy_units":units.iter().filter(|u|u["owner"]=="Opponent").count(),
        "runtime_facts":facts,"accepted_authoring_work":trial.accepted_work(),
        "lifecycle":trial.lifecycle(),"coverage":trial.lab_coverage()?,"reconnect":reconnect}))
}
fn placements(candidate:&Candidate,scenario:&Value,recipe:&mut LabPreparation,count:usize,out:&Path,mode:WindowMode)->Result<()> {
    let discovery=normal::prepare(candidate,scenario,recipe)?;
    let view=normal::reconnect(&discovery)?;
    normal::save(&out.join("placement_discovery.json"),&view)?;
    let tactical=view.pointer("/payload/battle_update/checkpoint/tactical_resource").ok_or("actual discovery currency absent")?;
    let deployment=view.pointer("/payload/battle_update/checkpoint/deployment").ok_or("actual discovery deployment absent")?;
    let costs=deployment["unit_deploy_costs"].as_array().ok_or("actual deployment prices absent")?;
    normal::save(&out.join("actual_initial_tactical_funding.json"),&json!({"tactical_resource":tactical,"deployment":deployment}))?;
    assert_eq!(tactical["current"],160);assert_eq!(tactical["maximum"],160);assert_eq!(tactical["recovery_per_second"],1);
    assert_eq!(deployment["battle_time_ms"],0);assert_eq!(deployment["deployed_units"],json!([]));
    assert!(!costs.is_empty());
    assert!(costs.iter().all(|row|row["base_deploy_cost"]==10&&row["effective_deploy_cost"]==10));
    let (cells,mut capacity)=normal::unoccupied_ground_cells(&view)?;
    capacity["requested_deployments"]=json!(count);
    normal::save(&out.join("placement_capacity.json"),&capacity)?;
    if recipe.encounter_id=="u11_parallel_patrol" {
        let setup=&view["payload"]["battle_setup_snapshot"];
        let expected_cells:std::collections::BTreeSet<_>=[4,6].into_iter().flat_map(|y|(1..=8).map(move|x|(x,y))).collect();
        let actual_cells:std::collections::BTreeSet<_>=cells.iter().map(|p|(p.x,p.y)).collect();
        let valid=setup["catalog_refs"]["battlefield_template_id"]=="u11_parallel_patrol_large"
            &&setup["encounter_id"]=="u11_parallel_patrol"&&setup["battlefield"]["width"]==11&&setup["battlefield"]["height"]==11
            &&capacity["unique_ground_cell_count"]==17&&actual_cells==expected_cells
            &&capacity["excluded_occupied_ground_cells"]==json!([{"x":10,"y":5}]);
        normal::save(&out.join("actual_custom_geometry.json"),&json!({"valid":valid,"setup":setup,"capacity":capacity}))?;
        if !valid{return Err("actual custom workload geometry differs from reviewed template".into());}
    }
    if cells.len()<count{return Err(format!("requested {count} legal deployments but actual setup provides only {} unoccupied ground cells",cells.len()).into());}
    let two_flank=mode.strict()&&recipe.encounter_id=="u11_parallel_patrol";
    let selected_cells=if two_flank {
        // Static route intersection alone did not keep the original left-hand
        // targets in range through windup. This fixed successor covers columns
        // 5..8 from both legal flanks; gameplay still decides every actual hit.
        let requested:Vec<_>=CADENCE_LARGE_CELLS.into_iter()
            .map(|(x,y)|game_core::game::resources::Position::new(x,y)).collect();
        let valid=count==8&&requested.iter().all(|p|cells.contains(p));
        normal::save(&out.join("fixed_placement_plan.json"),&json!({
            "plan_id":mode.large_placement_plan(),"requested_cells":requested,
            "actual_discovered_free_cells":cells,"all_requested_cells_verified_free":valid,
            "selected_before_gameplay":true,"prior_cadence001_geometry_failure_preserved":true}))?;
        if !valid{return Err("fixed two-flank plan requires all eight exact public free deployment cells".into());}
        requested
    }else{cells.into_iter().take(count).collect()};
    let mut chosen=Vec::new();let mut geometry=Vec::new();let mut intersecting=0usize;
    for (employee_index,position) in selected_cells.into_iter().enumerate() {
        let (facing,intersects,evidence)=normal::route_facing(candidate,recipe,&view,position)?;
        if two_flank {
            use game_core::game::battle::tile_range::FacingDirection;
            let expected=if position.y==4{FacingDirection::Down}else{FacingDirection::Up};
            if facing!=expected||!intersects{return Err("actual public facing geometry does not match the fixed two-flank plan".into());}
        }
        intersecting+=usize::from(intersects);geometry.push(evidence);
        chosen.push(LabPlacement{employee_index,position,facing});
    }
    normal::save(&out.join("placement_facing_geometry.json"),&json!({"placements":geometry,
        "placement_plan_id":if recipe.encounter_id=="u11_parallel_patrol"{mode.large_placement_plan()}else{"normal-discovery-row-priority-v1"},
        "route_intersecting_deployments":intersecting,"requested_deployments":count,
        "nonintersecting_positions_preserved":true}))?;
    recipe.placements=chosen;
    if intersecting!=count{return Err(format!("requested {count} route-intersecting deployments but fixed legal positions provide {intersecting}; requested count preserved").into());}
    Ok(())
}
fn main()->Result<()> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=5&&args.len()!=6{return Err("usage: prepare_workloads FROZEN_LAB_ROOT REGISTRY_JSON INDEPENDENT_CASE RUNTIME_SAFETY_RON NEW_OUTPUT [legacy-single-60000|fixed-cadence-60x1000]".into());}
    let mode=WindowMode::parse(args.get(5).map(|v|v.to_str().ok_or("UTF-8 workload mode required")).transpose()?)?;
    let root=Path::new(&args[0]);let label=args[2].to_str().ok_or("UTF-8 independent case required")?;
    let out=Path::new(&args[4]);fs::create_dir(out)?;
    let exact_profile=fs::read(&args[3])?;
    let settings=Arc::new(RuntimeSafetySettings::from_ron("independent fixed-window workload",&exact_profile)?);
    if label!="host_ab"{return Err("large workload successor requires exact host_ab producer fixture".into());}
    let lab:Value=serde_json::from_slice(&fs::read(root.join("LAB_BASE_ORACLE.json"))?)?;
    if lab["authored_target"]["max_health"]!=10000||lab["authored_target"]["attack"]!=0{return Err("unchanged trusted lab target baseline required".into());}
    let selection:ContentSelection=serde_json::from_slice(&fs::read(root.join("workload_base_selection.json"))?)?;
    if !selection.packs.iter().map(|pack|pack.as_path()).eq([Path::new("official"),Path::new("u11_workload_context")])||!selection.winners.is_empty(){return Err("exact reviewed trusted workload context required".into());}
    normal::save(&out.join("trusted_base_selection.json"),&selection)?;
    normal::save(&out.join("trusted_workload_context.json"),&json!({"selection":selection,"lab_base":lab,
        "fixture_context":serde_json::from_slice::<Value>(&fs::read(root.join("WORKLOAD_CONTEXT.json"))?)?,
        "tactical_funding":serde_json::from_slice::<Value>(&fs::read(root.join("TACTICAL_FUNDING.json"))?)?}))?;
    let base=TrustedBase::acquire(root,selection,
        &InputLimits::default(),settings,&CancellationToken::default())?;
    let candidate=Candidate::acquire(root,&fs::read(&args[1])?,&base,&InputLimits::default(),&CancellationToken::default())?;
    let expected=oracle::expected(label)?;
    normal::save(&out.join("independent_inventory.json"),&json!({"candidate":candidate.view(),"required_sites":expected}))?;
    let mut scenario=normal::prefix(&candidate,&out.join("normal_prefix"))?;
    // The selected complete window is fixed before any execution or measurement.
    // Legacy mode preserves the original one-operation stress proposal, including
    // its illegal 16-deployment case. The explicit successor uses legal 1/8/8.
    scenario["steps"]=json!((0..mode.operations()).map(|_|json!({"kind":"clock","clock":{
        "mode":"elapsed","delta_ms":mode.delta_ms(),"after_seq":0}})).collect::<Vec<_>>());
    normal::checked(&scenario)?;
    normal::save(&out.join("FIXED_CONTRACT.json"),&json!({"execution_mode":mode.name(),
        "measurement_revision":mode.revision(),"large_placement_plan_id":mode.large_placement_plan(),
        "large_placement_cells":if mode.strict(){Some(CADENCE_LARGE_CELLS.map(|(x,y)|json!({"x":x,"y":y})))}else{None},
        "large_placement_plan_scope":"expanded_and_pathological_only","declared_employee_counts":[16,500,1000],
        "declared_deployment_counts":mode.deployments(),"clock_operations":mode.operations(),
        "fixed_delta_ms":mode.delta_ms(),"requested_total_clock_ms":60000,"after_seq":0,
        "strict_running_checkpoints":mode.strict(),"initial_tactical_currency":160,"per_actor_cost":10,
        "expected_remaining_currency":mode.deployments().map(|n|160-n*10),
        "fixture_workload_context_counts_are_historical_proposal":mode.strict(),
        "historical_fixture_deployment_proposal":[1,8,16],"legacy_evidence_preserved":true,
        "legacy_mode_default_unchanged":true,"scenario":scenario}))?;
    let mut cases=Vec::new();let mut all_realized=true;let mut realized_isolated_suites=Vec::new();
    for((class,count),deployed)in [("normal",16_u32),("expanded",500),("pathological",1000)].into_iter().zip(mode.deployments()) {
        let directory=out.join(class);fs::create_dir(&directory)?;
        let mut recipe=normal::recipe(label,count)?;
        if class!="normal"{recipe.encounter_id="u11_parallel_patrol".into();}
        let placement=placements(&candidate,&scenario,&mut recipe,deployed,&directory,mode);
        normal::save(&directory.join("fixed_input.json"),&json!({"class":class,"requested_employee_count":count,
            "requested_deployments":deployed,"full_window_ms":60000,"execution_mode":mode.name(),
            "measurement_revision":mode.revision(),"clock_operations":mode.operations(),"fixed_delta_ms":mode.delta_ms(),
            "placement_plan_id":if class=="normal"{"normal-discovery-row-priority-v1"}else{mode.large_placement_plan()},
            "scenario":scenario,"preparation":recipe}))?;
        let result=placement.and_then(|()|{
            let mut trial=WorldLabPreparation::prepare(candidate.authoring_content(),normal::checked(&scenario)?,recipe.clone(),
                normal::limits(&candidate)?,&CancellationToken::default()).map_err(|e|format!("actual preparation: {e:?}"))?;
            let receipt=trial.lab_preparation().ok_or("actual bulk receipt required")?;
            assert_eq!(receipt.employee_ids.len(),count as usize);assert_eq!(receipt.deployed_count,deployed);
            oracle::inventory(&trial.lab_coverage()?,&candidate.generation().identity().semantic_sha256,&expected)?;
            let prepared=population(&trial)?;
            normal::save(&directory.join("actual_prepared.json"),&prepared)?;
            let checkpoint=prepared.pointer("/reconnect/payload/battle_update/checkpoint").ok_or("actual funded prepared checkpoint absent")?;
            let tactical=&checkpoint["tactical_resource"];
            normal::save(&directory.join("actual_tactical_payment.json"),&json!({"initial":160,
                "unchanged_per_actor_cost":10,"actual_deployed":receipt.deployed_count,
                "expected_paid":deployed*10,"expected_remaining":160-deployed*10,"actual_tactical_resource":tactical}))?;
            assert_eq!(checkpoint["battle_time_ms"],0);
            assert_eq!(tactical["current"],160-deployed*10);assert_eq!(tactical["maximum"],160);
            assert_eq!(tactical["recovery_per_second"],1);

            clock_window(&mut trial,mode,&directory)?;
            let after=population(&trial)?;
            normal::save(&directory.join("actual_after_full_window.json"),&after)?;
            let participation=workload_oracle::observe(&trial,&recipe,&after["reconnect"],class!="normal",mode.strict())?;
            normal::save(&directory.join("actual_participation_and_elapsed.json"),&participation)?;
            if participation["valid"]!=true{return Err("actual workload participation, spawn or elapsed contract not realized; exact raw evidence preserved".into());}
            let report=trial.lab_coverage()?;
            oracle::inventory(&report,&candidate.generation().identity().semantic_sha256,&expected)?;
            if !report.is_complete(){return Err("fixed full window left required independent producer sites unobserved".into());}
            Ok(())
        });
        let realized=result.is_ok();
        match result {
            Ok(())=>normal::save(&directory.join("REALIZATION.json"),&json!({"realized":true,"criterion_or_approval":false}))?,
            Err(error)=>{all_realized=false;normal::save(&directory.join("REALIZATION.json"),&json!({"realized":false,
                "error":error.to_string(),"count_reduced":false,"criterion_or_approval":false}))?;},
        }
        let case=json!({"id":format!("{}_{class}",if mode.strict(){"cadence"}else{"fixed"}),"class":class,"scenario":serde_json::to_vec(&scenario)?,"preparation":recipe});
        if realized {
            // Explicitly scoped diagnostic proposal; a successful normal scale
            // never replaces a failed expanded/pathological combined proposal.
            let isolated:LabSuite=serde_json::from_value(json!({"revision":if mode.strict(){format!("{}-{class}",mode.revision())}else{format!("u11-fixed-window-{class}-v2")},"cases":[case]}))?;
            normal::save(&out.join(format!("isolated_{class}_suite.json")),&isolated)?;
            realized_isolated_suites.push(format!("isolated_{class}_suite.json"));
        }
        cases.push(case);
    }
    if all_realized {
        let suite:LabSuite=serde_json::from_value(json!({"revision":mode.revision(),"cases":cases}))?;
        normal::save(&out.join("proposed_full_window_suite.json"),&suite)?;
    }else{
        // An unavailable placement/failed scale must not become a usable suite
        // with fewer actual actors. Preserve it only as an unrealized proposal.
        normal::save(&out.join("UNREALIZED_CASES.json"),&json!({"unrealized_proposal":true,"cases":cases,
            "declared_employee_counts":[16,500,1000],"declared_deployment_counts":mode.deployments(),
            "execution_mode":mode.name(),"measurement_revision":mode.revision(),
            "combined_suite_exported":false,"realized_isolated_suites":realized_isolated_suites}))?;
    }
    let profile=LabProfile{revision:if mode.strict(){"u11-fixed-cadence-60x1000-diagnostic-v2".into()}else{"u11-fixed-window-diagnostic-v1".into()},repeats:3,timeout_ms:300_000,
        max_output_bytes:16*1024*1024,runtime_safety_ron:exact_profile,criterion:None};
    normal::save(&out.join("diagnostic_profile.json"),&profile)?;
    normal::save(&out.join("RESULT.json"),&json!({"preparation_only":true,"runtime_executed":true,"all_declared_scales_realized":all_realized,
        "preparation_scope":"suite proposal includes actual World setup, complete requested60000ms clock input, per-actor participation and explicit actual lifecycle/terminal duration; no adopted criterion or approval",
        "trusted_fixture_pack":"u11_workload_context","normal_encounter":"normal_corroded_patrol","expanded_pathological_encounter":"u11_parallel_patrol",
        "requested_clock_is_not_claimed_active_combat_duration":true,
        "scale_is_prepared_roster_not_combat_count":true,"full_window_ms":60000,
        "execution_mode":mode.name(),"measurement_revision":mode.revision(),"declared_employee_counts":[16,500,1000],
        "large_placement_plan_id":mode.large_placement_plan(),"prior_cadence001_geometry_failure_preserved":true,
        "declared_deployment_counts":mode.deployments(),"clock_operations":mode.operations(),"fixed_delta_ms":mode.delta_ms(),
        "strict_running_checkpoints":mode.strict(),"legacy_stress_and_policy_failures_not_reclassified":true,"threshold_selected":false,"approval_claimed":false}))?;
    let files:Vec<_>=FileState::capture(out,65_536,256*1024*1024)?.report().entries.into_iter()
        .filter(|e|e.kind=="file").map(|e|json!({"path":e.relative,"bytes":e.bytes,"sha256":e.sha256.expect("actual regular diagnostic file hash")})).collect();
    normal::save(&out.join("DERIVED_INPUTS_RECEIPT.json"),&json!({"schema":"u11.derived-inputs/v1",
        "artifact_kind":"world-preparation-and-window-diagnostic","runtime_executed":true,
        "combined_suite_exported":all_realized,"criterion_selected":false,"approval_claimed":false,"files":files}))?;
    if !all_realized{return Err("one or more declared scales were not realized; preserved raw evidence requires classification".into());}
    Ok(())
}
