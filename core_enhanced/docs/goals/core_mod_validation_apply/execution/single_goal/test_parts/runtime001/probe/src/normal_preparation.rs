//! Independent normal bootstrap and bounded fixture discovery. Every mutation
//! uses a public command or WorldLabPreparation; no private state is installed.
//! Original U11 author work is preserved. Additions: /root/u11_tests_resume.
use game_core::{application::{ApplicationCommand,ApplicationSnapshotKind,CoreApplication},
    authoring::{CancellationToken,LabEmployeeGroup,LabPlacement,LabPreparation,
        LabRelicLoadout,ScenarioLimits,TrialLimits,TrialScenario,TrialSession,WorldLabPreparation},
    game::{behavior::{GameStateContextDto,RunSnapshotDto},battle::tile_range::FacingDirection,
        map::MapNodeCategory,resources::Position},mod_validation::Candidate};
use serde::Serialize;
use serde_json::{json,Value};
use std::{error::Error,fs::{self,OpenOptions},io::{Read,Write},path::Path,sync::Arc};
use uuid::Uuid;
use game_core::runtime_safety::{RuntimeSafetySettings,EMBEDDED_RUNTIME_SAFETY_RON};
use sha2::{Digest,Sha256};

pub const SEED:u64=77;
pub const PLAYER:Uuid=Uuid::from_u128(0xc0de1100000000000000000000000901);
pub fn save(path:&Path,value:&impl Serialize)->Result<(),Box<dyn Error>> {
    let mut bytes=serde_json::to_vec_pretty(value)?;bytes.push(b'\n');
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;Ok(())
}
/// Explicit trusted profile input; omission retains the exact embedded baseline.
pub fn runtime_profile(path:Option<&Path>,out:&Path)->Result<(Arc<RuntimeSafetySettings>,Vec<u8>),Box<dyn Error>> {
    let bytes=if let Some(path)=path {
        let mut bytes=Vec::new();fs::File::open(path)?.take(65_537).read_to_end(&mut bytes)?;
        if bytes.len()>65_536{return Err("runtime profile exceeds finite read bound".into());}bytes
    }else{EMBEDDED_RUNTIME_SAFETY_RON.to_vec()};
    let settings=Arc::new(RuntimeSafetySettings::from_ron("actual independent diagnostic profile",&bytes)?);
    save(out,&json!({"source_path":path,"embedded_default":path.is_none(),
        "raw_sha256":format!("{:x}",Sha256::digest(&bytes)),"effective_sha256":settings.effective_sha256(),
        "raw_bytes":bytes,"authoring_execution":settings.authoring_execution(),
        "criterion_selected":false,"default_limit_change_claimed":false}))?;
    Ok((settings,bytes))
}
/// Explicit trusted content input; supplied failures never fall back to official.
pub fn trusted_selection(path:Option<&Path>,out:&Path)->Result<game_core::game::data::content_pack::ContentSelection,Box<dyn Error>> {
    use game_core::game::data::content_pack::ContentSelection;
    let (selection,raw)=if let Some(path)=path {
        let mut bytes=Vec::new();fs::File::open(path)?.take(65_537).read_to_end(&mut bytes)?;
        if bytes.len()>65_536{return Err("trusted selection exceeds finite read bound".into());}
        (serde_json::from_slice::<ContentSelection>(&bytes)?,Some(bytes))
    }else{(ContentSelection{packs:vec!["official".into()],winners:vec![]},None)};
    save(out,&json!({"source_path":path,"official_default":path.is_none(),
        "raw_sha256":raw.as_ref().map(|bytes|format!("{:x}",Sha256::digest(bytes))),
        "raw_bytes":raw,"selection":selection}))?;
    Ok(selection)
}
pub fn checked(document:&Value)->Result<TrialScenario,Box<dyn Error>> {
    Ok(TrialScenario::from_json_slice(&serde_json::to_vec(document)?,&ScenarioLimits::default())?)
}
pub fn limits(candidate:&Candidate)->Result<TrialLimits,Box<dyn Error>> {
    let mut limits=TrialLimits::with_runtime_safety(candidate.generation().runtime_safety());
    limits.coverage=Some(candidate.coverage_plan()?);Ok(limits)
}
pub fn prepare(candidate:&Candidate,document:&Value,recipe:&LabPreparation)
    ->Result<TrialSession,Box<dyn Error>> {
    WorldLabPreparation::prepare(candidate.authoring_content(),checked(document)?,recipe.clone(),
        limits(candidate)?,&CancellationToken::default()).map_err(|e|format!("normal World preparation: {e:?}").into())
}
fn bind(step:usize,pointer:&str,to:&str)->Value {
    json!({"from":{"step":step,"pointer":pointer},"to":to,"value_type":"string"})
}

/// Record a normal StartNewGame/VN/starter prefix. Runtime IDs are result
/// bindings in the replay recipe, never copied from this discovery application.
/// Static employee definition IDs are real published catalog selectors.
pub fn prefix(candidate:&Candidate,out:&Path)->Result<Value,Box<dyn Error>> {
    fs::create_dir(out)?;
    let mut app=CoreApplication::new(candidate.generation().clone(),SEED);
    let mut steps=Vec::new();
    let mut command=json!({"type":"start_new_game","game_mode":"Standard","replace_active_run":false});
    let mut template=command.clone();let mut bindings=Vec::<Value>::new();
    for iteration in 0..64 {
        let parsed=ApplicationCommand::from_document(command.clone().into()).map_err(|e|format!("bootstrap command: {e:?}"))?;
        let result=app.execute(PLAYER,parsed,Default::default()).map_err(|e|format!("bootstrap execution: {e:?}"))?;
        steps.push(json!({"kind":"command","command":template,"bindings":bindings}));
        let snapshot=app.snapshot(ApplicationSnapshotKind::Run).map_err(|e|format!("bootstrap snapshot: {e:?}"))?;
        let raw=snapshot.read_model().as_value().clone();
        save(&out.join(format!("{iteration:03}.json")),&json!({"command":command,"result":result,"run":raw}))?;
        let snapshot_index=steps.len();steps.push(json!({"kind":"snapshot","snapshot":"run"}));
        let run:RunSnapshotDto=serde_json::from_value(raw)?;
        if let Some(vn)=run.visual_novel {
            let mut b=vec![bind(snapshot_index,"/read_model/visual_novel/session_id","/command/session_id"),
                bind(snapshot_index,"/read_model/visual_novel/current_scene_id","/command/current_scene_id")];
            if let Some((index,choice))=vn.choices.iter().enumerate().find(|(_,c)|c.selectable) {
                command=json!({"type":"select_visual_novel_choice","session_id":vn.session_id,
                    "current_scene_id":vn.current_scene_id,"choice_id":choice.choice_id});
                template=json!({"type":"select_visual_novel_choice","session_id":null,"current_scene_id":null,"choice_id":null});
                b.push(bind(snapshot_index,&format!("/read_model/visual_novel/choices/{index}/choice_id"),"/command/choice_id"));
            }else{
                command=json!({"type":"advance_visual_novel","session_id":vn.session_id,"current_scene_id":vn.current_scene_id});
                template=json!({"type":"advance_visual_novel","session_id":null,"current_scene_id":null});
            }
            bindings=b;continue;
        }
        if let GameStateContextDto::SelectingStarterEmployees{required_count,candidates}=run.game_state_context {
            let ids:Vec<_>=candidates.iter().take(required_count).map(|c|c.employee_definition_id.clone()).collect();
            if ids.len()!=required_count{return Err("insufficient published normal starter candidates".into());}
            command=json!({"type":"select_starter_employees","employee_definition_ids":ids});
            template=command.clone();bindings.clear();continue;
        }
        if run.roster.combat_eligible_employee_ids.len()<2{return Err("normal prefix has no starter roster".into());}
        let document=json!({"initial":{"player_id":PLAYER,"run_seed":SEED,"profile":"default_memory"},
            "preparation_steps":steps,"steps":[]});
        save(&out.join("replay_prefix.json"),&document)?;return Ok(document);
    }
    Err("normal bootstrap exceeded independent finite command bound".into())
}

/// Literal selectors and expected sites come from the independent RON fixtures.
pub fn recipe(case:&str,count:u32)->Result<LabPreparation,Box<dyn Error>> {
    let (equipment,relic)=match case {
        "host_ab"|"host_a"|"host_b"|"host_load"=>(vec!["standard_training_sword","standard_armor"],None),
        "ab"=>(vec!["u11_weapon_a","u11_armor_b"],None),
        "a"=>(vec!["u11_weapon_a","standard_armor"],None),
        "b"=>(vec!["standard_training_sword","u11_armor_b"],None),
        "noop"=>(vec!["u11_heal_weapon","u11_zero_armor"],None),
        "terminal"=>(vec!["u11_terminal_weapon","standard_armor"],None),
        "recipe_manual"=>(vec!["standard_training_sword","standard_armor"],Some("u11_recipe_relic")),
        "structural"=>(vec!["u11_structural_weapon","standard_armor"],Some("u11_structural_relic")),
        "causal"=>(vec!["u11_causal_weapon","standard_armor"],Some("u11_causal_relic")),
        "resources"=>(vec!["u11_resources_weapon","standard_armor"],Some("u11_resource_relic")),
        "recipe"=>(vec!["u11_recipe_weapon","standard_armor"],Some("u11_recipe_relic")),
        "objects"=>(vec!["u11_object_weapon","standard_armor"],None),
        _ if case.starts_with("host_load_")=>(vec!["standard_training_sword","standard_armor"],None),
        _=>return Err("unknown independent fixture case".into()),
    };
    Ok(LabPreparation{schema_version:1,employees:vec![LabEmployeeGroup{
        definition_id:"records_clerk".into(),count,equipment_ids:Some(equipment.into_iter().map(str::to_owned).collect()),
        active_fragment_id:if case=="recipe_manual"{Some("u11_recipe_fragment".into())}else{None},relics:relic.into_iter().map(|id|LabRelicLoadout{definition_id:id.into(),corrupt:false}).collect()}],
        encounter_id:"normal_corroded_patrol".into(),category:MapNodeCategory::Combat,placements:vec![]})
}

pub fn reconnect(trial:&TrialSession)->Result<Value,Box<dyn Error>> {
    let query=trial.reconnect(0,&trial.runtime_safety().authoring_execution(),&CancellationToken::default())?;
    Ok(serde_json::from_str(query.document())?)
}

/// Unique compatible deployment cells minus actual public checkpoint occupancy.
pub fn unoccupied_ground_cells(view:&Value)->Result<(Vec<Position>,Value),Box<dyn Error>> {
    let zones=view.pointer("/payload/battle_setup_snapshot/deployment_zones")
        .and_then(Value::as_array).ok_or("normal reconnect lacks setup deployment zones")?;
    let units=view.pointer("/payload/battle_update/checkpoint/units")
        .and_then(Value::as_array).ok_or("actual checkpoint units required for placement")?;
    if zones.len()>65_536||units.len()>65_536{return Err("finite public placement inventory exceeded".into());}
    let mut occupied=std::collections::BTreeSet::new();
    for unit in units {
        let p:Position=serde_json::from_value(unit.get("position").ok_or("checkpoint unit position absent")?.clone())?;
        occupied.insert((p.y,p.x));
    }
    let mut ground=std::collections::BTreeSet::new();let mut visits=0usize;
    for zone in zones {if zone["kind"]=="Ground" {
        let cells=zone["cells"].as_array().ok_or("ground zone cells absent")?;
        visits=visits.checked_add(cells.len()).ok_or("placement count overflow")?;
        if visits>65_536{return Err("finite public deployment cell inventory exceeded".into());}
        for raw in cells {let p:Position=serde_json::from_value(raw.clone())?;ground.insert((p.y,p.x));}
    }}
    let excluded:Vec<_>=ground.intersection(&occupied).map(|(y,x)|Position::new(*x,*y)).collect();
    let mut available:Vec<_>=ground.difference(&occupied).map(|(y,x)|Position::new(*x,*y)).collect();
    available.sort_by_key(|p|(p.y,(p.x-4).abs(),p.x));
    let report=json!({"unique_ground_cell_count":ground.len(),"excluded_occupied_ground_cells":excluded,
        "available_ground_cell_count":available.len(),"available_ground_cells":available,
        "occupancy_source":"actual public battle_update.checkpoint.units positions"});
    Ok((available,report))
}
/// Select geometry before execution from the public route and published weapon
/// footprint. Direction order is fixed; a cell with no route intersection keeps
/// Up and reports that fact. No elapsed time, target outcome, or witness is read.
pub fn route_facing(candidate:&Candidate,recipe:&LabPreparation,view:&Value,position:Position)
    ->Result<(FacingDirection,bool,Value),Box<dyn Error>> {
    if recipe.employees.len()!=1{return Err("route-facing fixture requires one homogeneous employee group".into());}
    let equipment=recipe.employees[0].equipment_ids.as_ref().ok_or("explicit fixture equipment required")?;
    if equipment.len()>64{return Err("finite fixture equipment bound exceeded".into());}
    let database=candidate.generation().database();
    let weapons:Vec<_>=equipment.iter().filter_map(|id|database.equipment_data.get_by_id(id)
        .and_then(|item|item.weapon_profile.as_ref().map(|profile|(id,profile)))).collect();
    if weapons.len()!=1{return Err("exactly one published fixture weapon profile required".into());}
    let (weapon_id,weapon)=weapons[0];
    let pattern=&weapon.defense_tile_range;
    if pattern.rows.len()>64||pattern.rows.iter().any(|row|row.len()>64){return Err("finite fixture attack footprint bound exceeded".into());}
    let routes=view.pointer("/payload/battle_setup_snapshot/routes").and_then(Value::as_array)
        .ok_or("actual public routes required for facing")?;
    if routes.len()>64{return Err("finite public route bound exceeded".into());}
    let matching:Vec<_>=routes.iter().filter(|route|route["id"]=="defense_main").collect();
    if matching.len()!=1{return Err("one actual defense_main route required".into());}
    let raw=matching[0]["cells"].as_array().ok_or("actual route cells required")?;
    if raw.len()>65_536{return Err("finite public route-cell bound exceeded".into());}
    let route:Vec<Position>=raw.iter().cloned().map(serde_json::from_value).collect::<Result<_,_>>()?;
    let mut observations=Vec::new();let mut selected=None;
    for facing in [FacingDirection::Up,FacingDirection::Right,FacingDirection::Down,FacingDirection::Left] {
        let projected=pattern.affected_tiles(position,facing)?;
        let intersections:Vec<_>=projected.iter().filter(|tile|route.contains(tile)).copied().collect();
        if selected.is_none()&&!intersections.is_empty(){selected=Some(facing);}
        observations.push(json!({"facing":facing,"projected_attack_tiles":projected,"route_intersections":intersections}));
    }
    let facing=selected.unwrap_or(FacingDirection::Up);
    Ok((facing,selected.is_some(),json!({"position":position,"weapon_id":weapon_id,
        "published_weapon_pattern":pattern,"actual_route":matching[0],"fixed_direction_order":["up","right","down","left"],
        "geometry_by_direction":observations,"selected_facing":facing,"route_intersection_found":selected.is_some(),
        "selection_rule":"first direction intersecting actual defense_main route, otherwise Up; chosen before execution"})))
}
/// Discover once, then keep the explicit recipe fixed for subsequent replays.
pub fn place_one(candidate:&Candidate,document:&Value,recipe:&mut LabPreparation,out:&Path)
    ->Result<(),Box<dyn Error>> {
    place_one_with_free_offsets(candidate,document,recipe,&[],out)
}
/// The object fixture additionally requires its authored spawn offsets to be
/// valid unoccupied ground, established before observing any gameplay outcome.
pub fn place_one_with_free_offsets(candidate:&Candidate,document:&Value,recipe:&mut LabPreparation,
    offsets:&[Position],out:&Path)->Result<(),Box<dyn Error>> {
    if offsets.len()>64{return Err("finite fixture placement offset bound exceeded".into());}
    let trial=prepare(candidate,document,recipe)?;
    let view=reconnect(&trial)?;save(out,&view)?;
    let (cells,mut selection)=unoccupied_ground_cells(&view)?;
    let mut free_ground=std::collections::BTreeSet::new();
    if !offsets.is_empty() {
        let tiles=view.pointer("/payload/battle_setup_snapshot/battlefield/tiles").and_then(Value::as_array)
            .ok_or("actual battlefield tiles required for object offsets")?;
        if tiles.len()>65_536{return Err("finite battlefield cell bound exceeded".into());}
        for tile in tiles {if tile["kind"]=="Ground" {
            let p:Position=serde_json::from_value(tile["position"].clone())?;free_ground.insert((p.y,p.x));
        }}
        for field in ["/payload/battle_update/checkpoint/units","/payload/battle_setup_snapshot/battlefield/blocked_tiles"] {
            let entries=view.pointer(field).and_then(Value::as_array).ok_or("actual occupied/blocked positions required")?;
            if entries.len()>65_536{return Err("finite occupied/blocked cell bound exceeded".into());}
            for entry in entries {
                let p:Position=serde_json::from_value(entry.get("position").unwrap_or(entry).clone())?;
                free_ground.remove(&(p.y,p.x));
            }
        }
    }
    let position=cells.into_iter().find(|p|offsets.iter().all(|offset|
        p.x.checked_add(offset.x).zip(p.y.checked_add(offset.y))
            .is_some_and(|(x,y)|free_ground.contains(&(y,x)))))
        .ok_or("normal encounter exposes no unoccupied compatible deployment cell")?;
    let (facing,intersects,geometry)=route_facing(candidate,recipe,&view,position)?;
    selection["selected_position"]=json!(position);selection["required_free_offsets"]=json!(offsets);
    selection["offset_coordinate_semantics"]=json!("SourceTile offsets are absolute world offsets, independent of facing");
    selection["facing_geometry"]=geometry;
    save(&out.with_extension("selection.json"),&selection)?;
    if !intersects{return Err("selected fixture cell has no attack footprint intersecting the actual route".into());}
    recipe.placements=vec![LabPlacement{employee_index:0,position,facing}];
    Ok(())
}
