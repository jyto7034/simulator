//! Genuine public Return seeds and approved filesystem consumers. The explicit
//! current-copy absence test below additionally uses a declared checkpoint
//! boundary fixture; it is not evidence of a public dismantling route.
use super::support::*;
use crate::mod_admission::{AdmissionSession, ApprovedPersistence};
use game_core::{
    application::{
        ApplicationCommand, ApplicationExecutionContext, ApplicationSnapshotKind, CoreApplication,
    },
    mod_validation::RequiredConfiguration,
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};
use uuid::Uuid;
#[path = "../probe/src/lib.rs"]
mod public_driver;

const CHILD: &str =
    "mod_admission::u12_normal_host_tests::return_cycle::u12_normal_return_cycle_child";
const COMMITTED: [&str; 4] = [
    "player_profile.json",
    "player_profile.backup.json",
    "active_run.json",
    "active_run.backup.json",
];
fn env_path(name: &str) -> Result<PathBuf> {
    Ok(std::env::var_os(name)
        .ok_or_else(|| format!("explicit {name} required"))?
        .into())
}
fn array(value: &Value) -> Result<&Vec<Value>> {
    value.as_array().ok_or("array required".into())
}
fn object(value: &Value) -> Result<&serde_json::Map<String, Value>> {
    value.as_object().ok_or("object required".into())
}
fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn profile(app: &CoreApplication) -> Result<Value> {
    snapshot(app, ApplicationSnapshotKind::PlayerProfile)
}
fn run(app: &CoreApplication) -> Result<Value> {
    snapshot(app, ApplicationSnapshotKind::Run)
}
fn committed(root: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    Ok(file_bytes(root)?
        .into_iter()
        .filter(|(name, _)| COMMITTED.contains(&name.as_str()))
        .collect())
}
fn copy_save(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir(destination)?;
    for name in COMMITTED {
        let path = source.join(name);
        if path.exists() {
            assert!(fs::symlink_metadata(&path)?.file_type().is_file());
            fs::copy(path, destination.join(name))?;
        }
    }
    assert!(destination.join("player_profile.json").is_file());
    assert!(destination.join("active_run.json").is_file());
    Ok(())
}
fn checkpoint(app: &CoreApplication) -> Result<Value> {
    Ok(app
        .export_active_run_checkpoint()?
        .map(|c| {
            json!({"bytes":c.core_payload_json(),
        "loads_used":c.loads_used(),"run_peak_certified_power":c.run_peak_certified_power()})
        })
        .unwrap_or(Value::Null))
}
fn save_seed(out: &Path, root: &Path, app: &CoreApplication) -> Result<()> {
    fs::create_dir(out)?;
    copy_save(root, &out.join("save"))?;
    save(
        &out.join("state.json"),
        &json!({"profile":profile(app)?,"run":run(app)?,"checkpoint":checkpoint(app)?}),
    )
}
fn ids(run: &Value) -> Result<BTreeSet<String>> {
    let mut result = BTreeSet::new();
    for value in array(&run["inventory"]["equipments"])? {
        assert!(result.insert(
            value["instance_uuid"]
                .as_str()
                .ok_or("equipment UUID")?
                .to_owned()
        ));
    }
    for value in array(&run["inventory"]["skill_fragments"])? {
        assert!(result.insert(
            value["copy"]["copy_uuid"]
                .as_str()
                .ok_or("fragment UUID")?
                .to_owned()
        ));
    }
    Ok(result)
}
fn vault_ids(profile: &Value) -> Result<BTreeSet<String>> {
    let vault = &profile["lifecycle"]["vault"];
    let mut ids = BTreeSet::new();
    for field in ["equipment", "fragments"] {
        for key in object(&vault[field])?.keys() {
            assert!(ids.insert(key.clone()));
        }
    }
    Ok(ids)
}
// Independent exact settlement oracle from the returned public inventory. Every
// insertion must be new; comparing only lengths could conceal a duplicate copy.
fn returned_profile(run: &Value, profile: &Value) -> Result<Value> {
    let mut expected = profile.clone();
    let vault = &mut expected["lifecycle"]["vault"];
    for item in array(&run["inventory"]["equipments"])? {
        let id = item["instance_uuid"].as_str().ok_or("equipment UUID")?;
        let data = &item["item"];
        let copy = json!({"copy_uuid":id,"definition_id":data["definition_id"],"equipment_type":data["equipment_type"],
            "enhancement_level":if data["equipment_type"]=="Weapon" {json!(0)} else {data["enhancement_level"].clone()}});
        assert!(vault["equipment"]
            .as_object_mut()
            .ok_or("vault equipment")?
            .insert(id.to_owned(), copy)
            .is_none());
    }
    for item in array(&run["inventory"]["skill_fragments"])? {
        let mut copy = item["copy"].clone();
        let id = copy["copy_uuid"]
            .as_str()
            .ok_or("fragment UUID")?
            .to_owned();
        copy.as_object_mut()
            .ok_or("fragment copy")?
            .remove("equipped_to");
        assert!(vault["fragments"]
            .as_object_mut()
            .ok_or("vault fragments")?
            .insert(id, copy)
            .is_none());
    }
    if expected["lifecycle"]["first_return_story"] == "Never" {
        expected["lifecycle"]["first_return_story"] = json!("Pending");
    }
    expected["lifecycle"]["active_expedition"] = Value::Null;
    Ok(expected)
}

struct Host<'a> {
    app: CoreApplication,
    required: RequiredConfiguration,
    revision: Option<Uuid>,
    admission: &'a AdmissionSession,
    persistence: &'a ApprovedPersistence,
    root: PathBuf,
    return_seed: Option<PathBuf>,
    hire_evidence: Option<PathBuf>,
    invariant_profile: Option<Value>,
}
impl<'a> Host<'a> {
    fn restore(
        admission: &'a AdmissionSession,
        persistence: &'a ApprovedPersistence,
        root: &Path,
    ) -> Result<Self> {
        let (app, required, revision) = restore(admission, persistence)?;
        Ok(Self {
            app,
            required,
            revision,
            admission,
            persistence,
            root: root.to_owned(),
            return_seed: None,
            hire_evidence: None,
            invariant_profile: None,
        })
    }
    fn check_profile(&self) -> Result<()> {
        if let Some(expected) = &self.invariant_profile {
            assert_eq!(&profile(&self.app)?, expected);
            assert_eq!(
                &serde_json::to_value(self.persistence.player_profile()?)?,
                expected
            );
        }
        Ok(())
    }
    fn execute(&mut self, document: Value, context: ApplicationExecutionContext) -> Result<Value> {
        let value = execute_with_context(
            self.admission,
            self.persistence,
            &mut self.app,
            &mut self.required,
            &mut self.revision,
            document,
            context,
        )?;
        self.check_profile()?;
        Ok(value["payload"].clone())
    }
    fn command(&mut self, document: Value) -> Result<Value> {
        self.execute(document, Default::default())
    }
    fn reject(&mut self, document: Value, evidence: &Path) -> Result<()> {
        let before = roots(&self.app)?;
        let files = file_bytes(&self.root)?;
        let revision = self.revision;
        let error = self
            .command(document.clone())
            .expect_err("negative must reject");
        assert_eq!(roots(&self.app)?, before);
        assert_eq!(file_bytes(&self.root)?, files);
        assert_eq!(self.revision, revision);
        save(
            evidence,
            &json!({"command":document,"error":error.to_string(),"roots_before":before,"unchanged":true}),
        )
    }
    fn paid_hire(&mut self, evidence: &Path) -> Result<()> {
        fs::create_dir(evidence)?;
        let before = run(&self.app)?;
        let profile_before = profile(&self.app)?;
        let offered = array(&before["selected_event"]["recruitment_candidates"])?;
        let candidate = offered
            .first()
            .ok_or("actual affordable HQ offer required")?
            .clone();
        let command = json!({"type":"recruit_employee","employee_definition_id":candidate["employee_definition_id"]});
        let result = self.command(command.clone())?;
        let after = run(&self.app)?;
        // Official run/policy.ron declares the positive paid price, 100.
        assert_eq!(
            before["resources"]["enkephalin"]
                .as_u64()
                .ok_or("wallet before")?
                .checked_sub(100)
                .ok_or("paid HQ wallet")?,
            after["resources"]["enkephalin"]
                .as_u64()
                .ok_or("wallet after")?
        );
        let old = array(&before["roster"]["employees"])?;
        let new = array(&after["roster"]["employees"])?;
        assert_eq!(new.len(), old.len() + 1);
        for employee in old {
            let retained = new
                .iter()
                .find(|copy| copy["uuid"] == employee["uuid"])
                .ok_or("retained employee")?;
            for field in [
                "uuid",
                "employee_definition_id",
                "role",
                "profile",
                "condition",
                "level",
                "experience",
                "equipped_items",
            ] {
                assert_eq!(retained[field], employee[field]);
            }
            for field in ["baseline_ids", "active_copy_uuid"] {
                assert_eq!(
                    retained["skill_fragments"][field],
                    employee["skill_fragments"][field]
                );
            }
        }
        let hired: Vec<_> = new
            .iter()
            .filter(|e| !old.iter().any(|prior| prior["uuid"] == e["uuid"]))
            .collect();
        assert_eq!(hired.len(), 1);
        let hired = hired[0];
        assert_eq!(
            hired["employee_definition_id"],
            candidate["employee_definition_id"]
        );
        let expected_equipment: BTreeSet<_> =
            array(&candidate["initial_loadout"]["equipment_ids"])?
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect();
        let equipped = array(&hired["equipped_items"])?;
        let actual_equipment: BTreeSet<_> = equipped
            .iter()
            .map(|v| v["definition_id"].as_str().unwrap())
            .collect();
        assert_eq!(actual_equipment, expected_equipment);
        assert_eq!(equipped.len(), expected_equipment.len());
        assert_eq!(
            hired["skill_fragments"]["baseline_ids"],
            candidate["initial_loadout"]["baseline_skill_fragment_ids"]
        );
        let fragment = array(&after["inventory"]["skill_fragments"])?
            .iter()
            .find(|f| f["copy"]["copy_uuid"] == hired["skill_fragments"]["active_copy_uuid"])
            .ok_or("hired active fragment")?;
        assert_eq!(
            fragment["copy"]["definition_id"],
            candidate["initial_loadout"]["active_skill_fragment_id"]
        );
        let old_ids = ids(&before)?;
        let new_ids = ids(&after)?;
        assert!(
            old_ids.is_subset(&new_ids),
            "loans and all preceding inventory remain owned"
        );
        for copy in array(&before["inventory"]["equipments"])? {
            assert!(array(&after["inventory"]["equipments"])?.contains(copy));
        }
        for copy in array(&before["inventory"]["skill_fragments"])? {
            assert!(array(&after["inventory"]["skill_fragments"])?
                .iter()
                .any(|after| after["copy"] == copy["copy"]));
        }
        assert_eq!(
            array(&after["inventory"]["equipments"])?.len(),
            array(&before["inventory"]["equipments"])?.len() + equipped.len()
        );
        assert_eq!(
            array(&after["inventory"]["skill_fragments"])?.len(),
            array(&before["inventory"]["skill_fragments"])?.len() + 1
        );
        let mut acquired = BTreeSet::new();
        for item in equipped {
            assert!(!old_ids.contains(item["instance_uuid"].as_str().unwrap()));
            acquired.insert(item["instance_uuid"].as_str().unwrap().to_owned());
        }
        assert!(!old_ids.contains(fragment["copy"]["copy_uuid"].as_str().unwrap()));
        acquired.insert(fragment["copy"]["copy_uuid"].as_str().unwrap().to_owned());
        assert_eq!(
            new_ids
                .difference(&old_ids)
                .cloned()
                .collect::<BTreeSet<_>>(),
            acquired
        );
        assert_eq!(profile(&self.app)?, profile_before);
        assert!(!array(&after["selected_event"]["recruitment_candidates"])?
            .iter()
            .any(|offer| offer["employee_definition_id"] == candidate["employee_definition_id"]));
        save(
            &evidence.join("paid_command.json"),
            &json!({"command":command,"result":result,"offered":candidate,
            "before":before,"after":after,"exact_price":100,"profile_unchanged":true}),
        )?;
        self.reject(command, &evidence.join("retry_rejected.json"))
    }
}
impl public_driver::Host for Host<'_> {
    fn app(&self) -> &CoreApplication {
        &self.app
    }
    fn command(&mut self, document: Value, context: ApplicationExecutionContext) -> Result<Value> {
        if document["type"] == "request_emergency_supplies" {
            if let Some(out) = self.hire_evidence.take() {
                self.paid_hire(&out)?;
            }
        }
        if document["type"] == "choose_expedition_continuation" && document["choice"] == "Return" {
            let before = run(&self.app)?;
            let expected = returned_profile(&before, &profile(&self.app)?)?;
            if let Some(seed) = self.return_seed.take() {
                save_seed(&seed, &self.root, &self.app)?;
                save(&seed.join("expected_return.json"), &expected)?;
            }
            let result = self.execute(document, context)?;
            assert_eq!(profile(&self.app)?, expected);
            assert!(self.app.export_active_run_checkpoint()?.is_none());
            assert!(self.persistence.snapshot()?.checkpoint.is_none());
            assert!(array(&run(&self.app)?["roster"]["employees"])?.is_empty());
            return Ok(result);
        }
        self.execute(document, context)
    }
    fn tick(&mut self, delta_ms: u64, after_seq: u64) -> Result<Option<Value>> {
        let before = self.app.export_active_run_checkpoint()?;
        let prepared = self
            .app
            .prepare_active_battle_for_host_tick(delta_ms, after_seq)?;
        let (result, pending) = self.persistence.publish(
            prepared,
            &self.required,
            &mut self.revision,
            before.as_ref(),
            false,
        )?;
        assert!(!pending);
        self.check_profile()?;
        Ok(result.map(|r| r.payload().as_value().clone()))
    }
    fn restart(&mut self) -> Result<()> {
        let (app, required, revision) = restore(self.admission, self.persistence)?;
        self.app = app;
        self.required = required;
        self.revision = revision;
        self.check_profile()
    }
}
fn to_selection(host: &mut Host<'_>, out: &Path) -> Result<Vec<Value>> {
    fs::create_dir(out)?;
    let command =
        json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false});
    let result = host.command(command.clone())?;
    save(
        &out.join("start.json"),
        &json!({"command":command,"result":result}),
    )?;
    for step in 0..128 {
        let state = run(&host.app)?;
        if state["game_state"] == "selecting_starter_employees" {
            let candidates = array(&state["game_state_context"]["candidates"])?;
            assert_eq!(candidates.len(), 6);
            return ["Dealer", "Tank", "Healer"]
                .iter()
                .map(|role| {
                    candidates
                        .iter()
                        .find(|c| c["role"] == *role)
                        .map(|c| c["employee_definition_id"].clone())
                        .ok_or("one candidate per role required".into())
                })
                .collect();
        }
        let vn = &state["visual_novel"];
        assert!(vn.is_object());
        let command = if let Some(choice) = array(&vn["choices"])?
            .iter()
            .find(|c| c["selectable"] == true)
        {
            json!({"type":"select_visual_novel_choice","session_id":vn["session_id"],"current_scene_id":vn["current_scene_id"],"choice_id":choice["choice_id"]})
        } else {
            json!({"type":"advance_visual_novel","session_id":vn["session_id"],"current_scene_id":vn["current_scene_id"]})
        };
        let result = host.command(command.clone())?;
        save(
            &out.join(format!("vn_{step:03}.json")),
            &json!({"command":command,"result":result}),
        )?;
    }
    Err("actual starter briefing exceeded finite bound".into())
}
fn loan_command(starters: &[Value], copies: &[Value]) -> Value {
    json!({"type":"select_starter_employees","employee_definition_ids":starters,"loan_copy_ids":copies})
}
fn ack() -> Value {
    json!({"type":"acknowledge_first_return_story"})
}
fn return_command() -> Value {
    json!({"type":"choose_expedition_continuation","choice":"Return"})
}

#[test]
#[ignore = "requires actual same-executable U12_NORMAL_CONFIG and exclusive U12_NORMAL_OUTPUT"]
fn u12_normal_public_return_loan_cycle_preserves_exact_copies_and_instances() {
    let test = || -> Result<()> {
        let config = env_path("U12_NORMAL_CONFIG")?;
        let out = env_path("U12_NORMAL_OUTPUT")?.join("public_return_cycle");
        fs::create_dir(&out)?;
        let admission = open(&config)?;
        let root = out.join("save");
        let persistence = ApprovedPersistence::open(&root, Arc::clone(&admission.host))?;
        let mut host = Host::restore(&admission, &persistence, &root)?;
        assert_eq!(profile(&host.app)?["lifecycle"]["expedition_sequence"], 0);
        host.return_seed = Some(out.join("first_pending_return"));
        let first = public_driver::walk(
            &mut host,
            "Expedition",
            "complete",
            &out.join("first.trace.jsonl"),
        )?;
        assert_eq!(first["full_lifecycle_complete"], true);
        assert_eq!(first["battles"], 3);
        let first_profile = profile(&host.app)?;
        assert_eq!(first_profile["lifecycle"]["expedition_sequence"], 1);
        assert_eq!(first_profile["lifecycle"]["first_return_story"], "Pending");
        assert_eq!(
            object(&first_profile["lifecycle"]["vault"]["equipment"])?.len(),
            6
        );
        assert_eq!(
            object(&first_profile["lifecycle"]["vault"]["fragments"])?.len(),
            3
        );
        save_seed(&out.join("first_return_pending"), &root, &host.app)?;
        let old = read(&out.join("first_pending_return/state.json"))?;
        let old_checkpoint = game_core::application::ApplicationCheckpoint::new(
            serde_json::from_value(old["checkpoint"]["bytes"].clone())?,
            serde_json::from_value(old["checkpoint"]["loads_used"].clone())?,
            serde_json::from_value(old["checkpoint"]["run_peak_certified_power"].clone())?,
        );
        let starters = to_selection(&mut host, &out.join("second_briefing"))?;
        let equipment = object(&first_profile["lifecycle"]["vault"]["equipment"])?
            .keys()
            .next()
            .unwrap()
            .clone();
        let fragment = object(&first_profile["lifecycle"]["vault"]["fragments"])?
            .keys()
            .next()
            .unwrap()
            .clone();
        let third = object(&first_profile["lifecycle"]["vault"]["equipment"])?
            .keys()
            .find(|id| **id != equipment)
            .unwrap()
            .clone();
        let copies = vec![json!(equipment), json!(fragment)];
        host.reject(
            loan_command(&starters, &[copies[0].clone(), copies[0].clone()]),
            &out.join("duplicate_loan_rejected.json"),
        )?;
        host.reject(
            loan_command(
                &starters,
                &[copies[0].clone(), copies[1].clone(), json!(third)],
            ),
            &out.join("third_loan_rejected.json"),
        )?;
        let command = loan_command(&starters, &copies);
        let result = host.command(command.clone())?;
        save(
            &out.join("loan_command.json"),
            &json!({"command":command,"result":result}),
        )?;
        let second_run = run(&host.app)?;
        let second_profile = profile(&host.app)?;
        assert_eq!(second_profile["lifecycle"]["expedition_sequence"], 2);
        assert_ne!(
            second_profile["lifecycle"]["active_expedition"],
            old["profile"]["lifecycle"]["active_expedition"]
        );
        let mut expected_vault = first_profile["lifecycle"]["vault"].clone();
        expected_vault["equipment"]
            .as_object_mut()
            .unwrap()
            .remove(&equipment);
        expected_vault["fragments"]
            .as_object_mut()
            .unwrap()
            .remove(&fragment);
        assert_eq!(second_profile["lifecycle"]["vault"], expected_vault);
        let mut expected_profile = first_profile.clone();
        expected_profile["lifecycle"]["vault"] = expected_vault;
        expected_profile["lifecycle"]["expedition_sequence"] = json!(2);
        expected_profile["lifecycle"]["active_expedition"] =
            second_run["run_progression"]["expedition_instance"].clone();
        assert_eq!(second_profile, expected_profile);
        let owned = ids(&second_run)?;
        assert!(owned.contains(&equipment) && owned.contains(&fragment));
        assert!(owned.is_disjoint(&vault_ids(&second_profile)?));
        let equipment_copy = array(&second_run["inventory"]["equipments"])?
            .iter()
            .find(|copy| copy["instance_uuid"] == equipment)
            .ok_or("exact loaned equipment")?;
        let equipment_source = &first_profile["lifecycle"]["vault"]["equipment"][&equipment];
        assert_eq!(
            equipment_copy["item"]["definition_id"],
            equipment_source["definition_id"]
        );
        assert_eq!(
            equipment_copy["item"]["equipment_type"],
            equipment_source["equipment_type"]
        );
        assert_eq!(
            equipment_copy["item"]["enhancement_level"],
            equipment_source["enhancement_level"]
        );
        assert!(equipment_copy["equipped_to"].is_null());
        let fragment_copy = array(&second_run["inventory"]["skill_fragments"])?
            .iter()
            .find(|copy| copy["copy"]["copy_uuid"] == fragment)
            .ok_or("exact loaned fragment")?;
        let mut exact_fragment = fragment_copy["copy"].clone();
        assert!(exact_fragment["equipped_to"].is_null());
        exact_fragment
            .as_object_mut()
            .unwrap()
            .remove("equipped_to");
        assert_eq!(
            exact_fragment,
            first_profile["lifecycle"]["vault"]["fragments"][&fragment]
        );
        let old_actors: BTreeSet<_> = array(&old["run"]["roster"]["employees"])?
            .iter()
            .map(|e| e["uuid"].clone().to_string())
            .collect();
        assert!(array(&second_run["roster"]["employees"])?
            .iter()
            .all(|e| !old_actors.contains(&e["uuid"].to_string())));
        let first_owned = ids(&old["run"])?;
        let preserved: BTreeSet<_> = [equipment.clone(), fragment.clone()].into_iter().collect();
        assert_eq!(
            owned
                .intersection(&first_owned)
                .cloned()
                .collect::<BTreeSet<_>>(),
            preserved
        );
        save_seed(&out.join("second_departed"), &root, &host.app)?;
        save(
            &out.join("loans.json"),
            &json!({"copies":copies,"command":command,"profile":second_profile,"run":second_run,"checkpoint":checkpoint(&host.app)?}),
        )?;
        let (mut stale_reader, _, _) = new_application(&admission, &persistence)?;
        let roots_before = roots(&stale_reader)?;
        let files_before = file_bytes(&root)?;
        assert!(stale_reader
            .continue_from_active_run_checkpoint(PLAYER, &old_checkpoint)
            .is_err());
        assert_eq!(roots(&stale_reader)?, roots_before);
        assert_eq!(file_bytes(&root)?, files_before);
        let (mut current_reader, _, _) = new_application(&admission, &persistence)?;
        current_reader.continue_from_active_run_checkpoint(
            PLAYER,
            &host
                .app
                .export_active_run_checkpoint()?
                .ok_or("second instance checkpoint")?,
        )?;
        assert_eq!(checkpoint(&current_reader)?, checkpoint(&host.app)?);
        assert_eq!(file_bytes(&root)?, files_before);
        save(
            &out.join("prior_instance_rejected.json"),
            &json!({"old_checkpoint":old["checkpoint"],"current_profile_roots":roots_before,
            "current_checkpoint_positive":checkpoint(&current_reader)?,"unchanged":true}),
        )?;
        host.hire_evidence = Some(out.join("second_paid_hq"));
        let savepoint = public_driver::walk(
            &mut host,
            "Expedition",
            "save",
            &out.join("second_to_save.trace.jsonl"),
        )?;
        assert_eq!(savepoint["save_points"], 1);
        assert!(host.hire_evidence.is_none());
        let before_ack = profile(&host.app)?;
        let result = host.command(ack())?;
        let mut consumed = before_ack.clone();
        consumed["lifecycle"]["first_return_story"] = json!("Consumed");
        assert_eq!(profile(&host.app)?, consumed);
        save(
            &out.join("savepoint_ack.json"),
            &json!({"command":ack(),"result":result,"profile_before":before_ack,"profile_after":consumed}),
        )?;
        let command = json!({"type":"load_run_checkpoint"});
        let result = host.command(command.clone())?;
        assert_eq!(profile(&host.app)?, consumed);
        assert!(ids(&run(&host.app)?)?.contains(&equipment));
        save(
            &out.join("load_savepoint.json"),
            &json!({"command":command,"result":result,"profile":profile(&host.app)?}),
        )?;
        host.return_seed = Some(out.join("second_pending_return"));
        let second = public_driver::walk(
            &mut host,
            "Expedition",
            "complete",
            &out.join("second_to_return.trace.jsonl"),
        )?;
        assert_eq!(second["terminal_reached"], true);
        let final_profile = profile(&host.app)?;
        assert_eq!(final_profile["lifecycle"]["first_return_story"], "Consumed");
        assert_eq!(final_profile["lifecycle"]["expedition_sequence"], 2);
        assert!(host.app.export_active_run_checkpoint()?.is_none());
        save_seed(&out.join("second_return_consumed"), &root, &host.app)?;
        drop(host);
        drop(persistence);
        let reopened = ApprovedPersistence::open(&root, Arc::clone(&admission.host))?;
        assert_eq!(
            serde_json::to_value(reopened.player_profile()?)?,
            final_profile
        );
        assert!(reopened.snapshot()?.checkpoint.is_none());
        save(
            &out.join("RESULT.json"),
            &json!({"passed":true,"first":first,"second":second,"profile":final_profile,
            "exact_seeds":["first_pending_return","first_return_pending","second_departed","second_pending_return","second_return_consumed"]}),
        )?;
        Ok(())
    };
    test().unwrap();
}

#[test]
#[ignore = "requires current approved configuration and genuine U12_CYCLE_SEED_ROOT"]
fn u12_normal_return_does_not_reissue_copies_absent_from_current_but_retained_in_savepoint() {
    use sha2::{Digest, Sha256};
    let test = || -> Result<()> {
        let config = env_path("U12_NORMAL_CONFIG")?;
        let seeds = env_path("U12_CYCLE_SEED_ROOT")?;
        let out = env_path("U12_NORMAL_OUTPUT")?.join("current_copy_absence_return");
        fs::create_dir(&out)?;
        let root = out.join("save");
        copy_save(&seeds.join("second_pending_return/save"), &root)?;
        let admission = open(&config)?;
        let persistence = ApprovedPersistence::open(&root, Arc::clone(&admission.host))?;
        let mut host = Host::restore(&admission, &persistence, &root)?;
        let original = host.app.export_active_run_checkpoint()?.ok_or("pending return checkpoint")?;
        let mut document: Value = serde_json::from_slice(original.core_payload_json())?;
        let copies = read(&seeds.join("loans.json"))?["copies"].clone();
        let equipment = copies[0].as_str().ok_or("loaned equipment")?;
        let fragment = copies[1].as_str().ok_or("loaned fragment")?;
        let savepoint_before = document["continuation"]["savepoint"].clone();
        assert!(savepoint_before["snapshot"]["checkpoint"]["inventory"]["equipments"]["items"].get(equipment).is_some());
        assert!(savepoint_before["snapshot"]["checkpoint"]["skill_fragments"]["copies"].get(fragment).is_some());
        let current = &mut document["continuation"]["current"];
        let removed_equipment = current["inventory"]["equipments"]["items"].as_object_mut().unwrap().remove(equipment).ok_or("carried equipment")?;
        let removed_fragment = current["skill_fragments"]["copies"].as_object_mut().unwrap().remove(fragment).ok_or("carried fragment")?;
        assert!(removed_equipment["equipped_to"].is_null() && removed_fragment["equipped_to"].is_null());
        // I-boundary fixture: current copies have already been consumed/lost.
        // Keep the genuine older SavePoint and all knowledge/history unchanged.
        // Existing dismantling tests own that producer; this test owns return.
        fn canonical(value: &Value) -> Value {
            match value {
                Value::Object(fields) => serde_json::to_value(fields.iter().map(|(key, value)| (key, canonical(value))).collect::<BTreeMap<_, _>>()).unwrap(),
                Value::Array(values) => Value::Array(values.iter().map(canonical).collect()),
                value => value.clone(),
            }
        }
        document["continuation_sha256"] = json!(format!("{:x}", Sha256::digest(serde_json::to_vec(&canonical(&document["continuation"]))?)));
        let fixture = game_core::application::ApplicationCheckpoint::new(
            serde_json::to_vec(&document)?, original.loads_used(), original.run_peak_certified_power(),
        );
        let (mut current_app, _, _) = new_application(&admission, &persistence)?;
        current_app.continue_from_active_run_checkpoint(PLAYER, &fixture)?;
        let imported: Value = serde_json::from_slice(current_app.export_active_run_checkpoint()?.unwrap().core_payload_json())?;
        assert_eq!(imported["continuation"]["savepoint"], savepoint_before);
        assert!(!ids(&run(&current_app)?)?.contains(equipment));
        assert!(!ids(&run(&current_app)?)?.contains(fragment));
        assert_eq!(profile(&current_app)?, profile(&host.app)?);
        host.app = current_app;
        let mut expected = read(&seeds.join("second_pending_return/expected_return.json"))?;
        assert!(expected["lifecycle"]["vault"]["equipment"].as_object_mut().unwrap().remove(equipment).is_some());
        assert!(expected["lifecycle"]["vault"]["fragments"].as_object_mut().unwrap().remove(fragment).is_some());
        let returned = host.command(return_command())?;
        assert_eq!(profile(&host.app)?, expected, "settlement must use current ownership, never the older SavePoint or acquisition history");
        assert!(host.app.export_active_run_checkpoint()?.is_none());
        host.reject(return_command(), &out.join("duplicate_return_rejected.json"))?;
        save(&out.join("RESULT.json"), &json!({"passed":true, "input_class":"I: current-copy absence checkpoint fixture; genuine prior public seeds retained", "removed":[equipment,fragment], "older_savepoint_unchanged":true, "result":returned, "expected_profile":expected}))?;
        drop(host);
        drop(persistence);
        let reopened = ApprovedPersistence::open(&root, Arc::clone(&admission.host))?;
        assert_eq!(serde_json::to_value(reopened.player_profile()?)?, expected);
        assert!(reopened.snapshot()?.checkpoint.is_none());
        Ok(())
    };
    test().unwrap();
}

fn expected_case(seeds: &Path, operation: &str) -> Result<Value> {
    match operation {
        "return" => Ok(
            json!({"profile":read(&seeds.join("first_pending_return/expected_return.json"))?,"checkpoint":null}),
        ),
        "loan" => {
            let loan = read(&seeds.join("loans.json"))?;
            Ok(json!({"profile":loan["profile"],"checkpoint":loan["checkpoint"]}))
        }
        "story" => {
            let mut state = read(&seeds.join("first_return_pending/state.json"))?;
            state["profile"]["lifecycle"]["first_return_story"] = json!("Consumed");
            Ok(json!({"profile":state["profile"],"checkpoint":null}))
        }
        _ => Err("unknown actual seed operation".into()),
    }
}
fn perform_seed_operation(
    host: &mut Host<'_>,
    operation: &str,
    seeds: &Path,
    out: &Path,
    crash: bool,
) -> Result<()> {
    let document = match operation {
        "return" => return_command(),
        "story" => ack(),
        "loan" => {
            let starters = to_selection(host, &out.join("briefing"))?;
            let document = read(&seeds.join("loans.json"))?["command"].clone();
            assert_eq!(document["employee_definition_ids"], json!(starters));
            document
        }
        _ => return Err("unknown operation".into()),
    };
    save(
        &out.join("before.json"),
        &json!({"command":document,"profile":profile(&host.app)?,
        "checkpoint":checkpoint(&host.app)?,"roots":roots(&host.app)?,"committed":committed(&host.root)?}),
    )?;
    if crash {
        std::env::set_var("U04_DURABLE_CRASH_AT", std::env::var("U12_CYCLE_STAGE")?);
    }
    let result = host.command(document.clone())?;
    assert!(!crash, "real durable crash stage was not reached");
    save(
        &out.join("operation.json"),
        &json!({"command":document,"result":result}),
    )?;
    let expected = expected_case(seeds, operation)?;
    assert_eq!(profile(&host.app)?, expected["profile"]);
    assert_eq!(checkpoint(&host.app)?, expected["checkpoint"]);
    Ok(())
}

#[test]
#[ignore = "subprocess-only helper; parent test requires an explicit child result and real exit"]
fn u12_normal_return_cycle_child() {
    let test = || -> Result<()> {
        assert!(
            std::env::var_os("U04_DURABLE_CRASH_AT").is_none(),
            "crash injection begins at the target command only"
        );
        let config = env_path("U12_NORMAL_CONFIG")?;
        let root = env_path("U12_CYCLE_ROOT")?;
        let out = env_path("U12_CYCLE_CHILD_OUTPUT")?;
        let operation = std::env::var("U12_CYCLE_OPERATION")?;
        let mode = std::env::var("U12_CYCLE_CHILD_MODE")?;
        let admission = open(&config)?;
        let persistence = ApprovedPersistence::open(&root, Arc::clone(&admission.host))?;
        let mut host = Host::restore(&admission, &persistence, &root)?;
        match mode.as_str() {
            "crash" | "retry" => perform_seed_operation(
                &mut host,
                &operation,
                &env_path("U12_CYCLE_SEED_ROOT")?,
                &out,
                mode == "crash",
            )?,
            "recover" => {}
            "tutorial" => {
                let expected = profile(&host.app)?;
                assert!(!vault_ids(&expected)?.is_empty());
                assert!(matches!(
                    expected["lifecycle"]["first_return_story"].as_str(),
                    Some("Pending" | "Consumed")
                ));
                host.invariant_profile = Some(expected.clone());
                let result = public_driver::walk(
                    &mut host,
                    "Tutorial",
                    "complete",
                    &out.join("tutorial.trace.jsonl"),
                )?;
                assert_eq!(result["full_lifecycle_complete"], true);
                assert_eq!(result["floors"], json!([0, 1, 2]));
                assert!(result["battles"].as_u64().ok_or("real Tutorial battles")? > 0);
                assert_eq!(profile(&host.app)?, expected);
                assert!(host.app.export_active_run_checkpoint()?.is_none());
                assert!(persistence.snapshot()?.checkpoint.is_none());
                save(
                    &out.join("tutorial.json"),
                    &json!({"result":result,"profile_before":expected,"profile_after":profile(&host.app)?}),
                )?;
            }
            _ => return Err("unknown child mode".into()),
        }
        save(
            &out.join("RESULT.json"),
            &json!({"passed":true,"mode":mode,"operation":operation,
            "profile":profile(&host.app)?,"checkpoint":checkpoint(&host.app)?,"roots":roots(&host.app)?,
            "revision":host.revision,"files":file_bytes(&root)?}),
        )?;
        Ok(())
    };
    test().unwrap();
}
fn child(
    config: &Path,
    root: &Path,
    out: &Path,
    seeds: &Path,
    mode: &str,
    operation: &str,
    stage: Option<&str>,
) -> Result<Option<Value>> {
    fs::create_dir(out)?;
    let mut command = Command::new(std::env::current_exe()?);
    command
        .args([
            "--exact",
            CHILD,
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("U12_NORMAL_CONFIG", config)
        .env("U12_CYCLE_ROOT", root)
        .env("U12_CYCLE_CHILD_OUTPUT", out)
        .env("U12_CYCLE_SEED_ROOT", seeds)
        .env("U12_CYCLE_CHILD_MODE", mode)
        .env("U12_CYCLE_OPERATION", operation)
        .env_remove("U04_DURABLE_CRASH_AT")
        .env_remove("U12_CYCLE_STAGE");
    if let Some(stage) = stage {
        command.env("U12_CYCLE_STAGE", stage);
    }
    let result = command.output()?;
    fs::write(out.join("stdout.txt"), &result.stdout)?;
    fs::write(out.join("stderr.txt"), &result.stderr)?;
    save(
        &out.join("exit.json"),
        &json!({"exit":result.status.code(),"mode":mode,"operation":operation,"stage":stage,"selector":CHILD}),
    )?;
    if mode == "crash" {
        assert_eq!(
            result.status.code(),
            Some(86),
            "actual process crash required: {operation} {stage:?}"
        );
        assert!(!out.join("RESULT.json").exists());
        Ok(None)
    } else {
        assert_eq!(
            result.status.code(),
            Some(0),
            "fresh child failed: {mode} {operation}"
        );
        let observed = read(&out.join("RESULT.json"))?;
        assert_eq!(observed["passed"], true);
        assert_eq!(observed["mode"], mode);
        assert_eq!(observed["operation"], operation);
        Ok(Some(observed))
    }
}

#[test]
#[ignore = "requires explicit genuine U12_CYCLE_SEED_ROOT from public_return_cycle, same-executable config, exclusive output"]
fn u12_normal_return_loan_and_story_crashes_recover_exact_public_seeds() {
    let test = || -> Result<()> {
        let config = env_path("U12_NORMAL_CONFIG")?;
        let seeds = env_path("U12_CYCLE_SEED_ROOT")?;
        assert_eq!(read(&seeds.join("RESULT.json"))?["passed"], true);
        let out = env_path("U12_NORMAL_OUTPUT")?.join("return_loan_story_crashes");
        fs::create_dir(&out)?;
        let all_stages = [
            "intent_temp_synced",
            "intent_renamed",
            "intent_synced",
            "profile_temp_synced",
            "profile_renamed",
            "profile_synced",
            "profile_backup_temp_synced",
            "profile_backup_synced",
            "active_temp_synced",
            "active_renamed",
            "active_synced",
            "active_backup_temp_synced",
            "active_backup_synced",
            "before_intent_clear",
            "intent_removed",
            "intent_clear_synced",
        ];
        let mut crashes = 0;
        for (operation, source, stages) in [
            ("return", "first_pending_return", all_stages.as_slice()),
            (
                "loan",
                "first_return_pending",
                &["intent_temp_synced", "intent_renamed", "profile_renamed"][..],
            ),
            (
                "story",
                "first_return_pending",
                &["intent_temp_synced", "intent_renamed", "profile_renamed"][..],
            ),
        ] {
            for stage in stages {
                let case = out.join(format!("{operation}_{stage}"));
                fs::create_dir(&case)?;
                let root = case.join("save");
                copy_save(&seeds.join(source).join("save"), &root)?;
                child(
                    &config,
                    &root,
                    &case.join("crash"),
                    &seeds,
                    "crash",
                    operation,
                    Some(stage),
                )?;
                crashes += 1;
                let before = read(&case.join("crash/before.json"))?;
                let accepted = *stage != "intent_temp_synced";
                let expected = if accepted {
                    expected_case(&seeds, operation)?
                } else {
                    json!({"profile":before["profile"],"checkpoint":before["checkpoint"]})
                };
                if !accepted {
                    let original: BTreeMap<String, Vec<u8>> =
                        serde_json::from_value(before["committed"].clone())?;
                    assert_eq!(
                        committed(&root)?,
                        original,
                        "unaccepted temporary intent cannot change the four committed files"
                    );
                    assert!(root.join("application_commit.pending.tmp").is_file());
                }
                let mut recovered_files = None;
                for pass in 0..2 {
                    let result = child(
                        &config,
                        &root,
                        &case.join(format!("recover_{pass}")),
                        &seeds,
                        "recover",
                        operation,
                        None,
                    )?
                    .unwrap();
                    assert_eq!(
                        result["profile"], expected["profile"],
                        "{operation} {stage} profile"
                    );
                    assert_eq!(
                        result["checkpoint"], expected["checkpoint"],
                        "{operation} {stage} checkpoint"
                    );
                    if let Some(previous) = &recovered_files {
                        assert_eq!(
                            &result["files"], previous,
                            "second fresh recovery is byte-identical"
                        );
                    } else {
                        recovered_files = Some(result["files"].clone());
                    }
                }
                if !accepted {
                    let retry = child(
                        &config,
                        &root,
                        &case.join("retry"),
                        &seeds,
                        "retry",
                        operation,
                        None,
                    )?
                    .unwrap();
                    let expected = expected_case(&seeds, operation)?;
                    assert_eq!(retry["profile"], expected["profile"]);
                    assert_eq!(retry["checkpoint"], expected["checkpoint"]);
                    let reopened = child(
                        &config,
                        &root,
                        &case.join("retry_reopen"),
                        &seeds,
                        "recover",
                        operation,
                        None,
                    )?
                    .unwrap();
                    assert_eq!(reopened["profile"], retry["profile"]);
                    assert_eq!(reopened["checkpoint"], retry["checkpoint"]);
                    assert_eq!(reopened["files"], retry["files"]);
                }
                save(
                    &case.join("RESULT.json"),
                    &json!({"passed":true,"accepted":accepted,"expected":expected,
                    "two_fresh_recoveries":true,"unaccepted_retry":!accepted}),
                )?;
            }
        }
        assert_eq!(crashes, 22);
        save(
            &out.join("RESULT.json"),
            &json!({"passed":true,"actual_crash_processes":crashes,"fresh_recoveries_per_case":2}),
        )?;
        Ok(())
    };
    test().unwrap();
}

#[test]
#[ignore = "requires genuine U12_CYCLE_SEED_ROOT, reviewed same-executable config, exclusive output"]
fn u12_normal_profile_only_story_ack_rejects_stale_and_preserves_no_change() {
    let test = || -> Result<()> {
        let config = env_path("U12_NORMAL_CONFIG")?;
        let seeds = env_path("U12_CYCLE_SEED_ROOT")?;
        assert_eq!(read(&seeds.join("RESULT.json"))?["passed"], true);
        let out = env_path("U12_NORMAL_OUTPUT")?.join("profile_only_ack_cas");
        fs::create_dir(&out)?;
        let root = out.join("save");
        copy_save(&seeds.join("first_return_pending/save"), &root)?;
        let admission = open(&config)?;
        let persistence = ApprovedPersistence::open(&root, Arc::clone(&admission.host))?;
        // A and B share the normal publication owner, with independently captured
        // Core/profile/revision. This exercises the CAS ahead of no-change logic.
        let mut a = Host::restore(&admission, &persistence, &root)?;
        let mut b = Host::restore(&admission, &persistence, &root)?;
        assert!(checkpoint(&a.app)?.is_null());
        let before_a = roots(&a.app)?;
        let checkpoint_a = a.app.export_active_run_checkpoint()?;
        let (prepared, next) = admission
            .host
            .prepare_with_future_generation(
                &mut a.app,
                &a.required,
                PLAYER,
                ApplicationCommand::from_document(ack().into())?,
                Default::default(),
            )
            .map_err(|e| format!("prepare A: {e:?}"))?;
        let b_result = b.command(ack())?;
        let bytes = file_bytes(&root)?;
        let accepted_revision = b.revision;
        let error = persistence
            .publish(
                prepared,
                &next,
                &mut a.revision,
                checkpoint_a.as_ref(),
                false,
            )
            .expect_err("stale Pending acknowledgement must reject");
        assert!(error.contains("revision conflict"));
        assert_eq!(roots(&a.app)?, before_a);
        assert_eq!(file_bytes(&root)?, bytes);
        assert_eq!(Some(persistence.snapshot()?.revision), accepted_revision);
        save(
            &out.join("pending_stale.json"),
            &json!({"command":ack(),"b_result":b_result,"a_error":error,"b_revision":accepted_revision,"unchanged":true}),
        )?;
        let mut a = Host::restore(&admission, &persistence, &root)?;
        let mut b = Host::restore(&admission, &persistence, &root)?;
        assert_eq!(
            profile(&a.app)?["lifecycle"]["first_return_story"],
            "Consumed"
        );
        let consumed_profile = profile(&a.app)?;
        let before_a = roots(&a.app)?;
        let checkpoint_a = a.app.export_active_run_checkpoint()?;
        assert!(checkpoint_a.is_none());
        let (prepared, next) = admission
            .host
            .prepare_with_future_generation(
                &mut a.app,
                &a.required,
                PLAYER,
                ApplicationCommand::from_document(ack().into())?,
                Default::default(),
            )
            .map_err(|e| format!("prepare consumed A: {e:?}"))?;
        let start =
            json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false});
        let b_result = b.command(start.clone())?;
        assert!(checkpoint(&b.app)?.is_null());
        assert_eq!(profile(&b.app)?, consumed_profile);
        let bytes = file_bytes(&root)?;
        let accepted_revision = b.revision;
        let error = persistence
            .publish(prepared, &next, &mut a.revision, None, false)
            .expect_err("stale no-change acknowledgement must reject before fast path");
        assert!(error.contains("revision conflict"));
        assert_eq!(roots(&a.app)?, before_a);
        assert_eq!(file_bytes(&root)?, bytes);
        assert_eq!(Some(persistence.snapshot()?.revision), accepted_revision);
        save(
            &out.join("consumed_stale.json"),
            &json!({"competing_command":start,"b_result":b_result,"a_command":ack(),"a_error":error,"unchanged":true}),
        )?;
        // B opened a blocking departure briefing to advance the publication
        // revision. Reopen the committed profile-only state before the positive
        // duplicate ACK: that action is not available inside the briefing.
        let mut latest = Host::restore(&admission, &persistence, &root)?;
        assert_eq!(latest.revision, accepted_revision);
        assert!(checkpoint(&latest.app)?.is_null());
        assert_eq!(profile(&latest.app)?, consumed_profile);
        let before_latest = roots(&latest.app)?;
        let result = latest.command(ack())?;
        assert_eq!(roots(&latest.app)?, before_latest);
        assert_eq!(file_bytes(&root)?, bytes);
        assert_eq!(latest.revision, accepted_revision);
        save(
            &out.join("RESULT.json"),
            &json!({"passed":true,"latest_duplicate_ack":{"command":ack(),"result":result},
            "profile_only_pending_cas":true,"profile_only_consumed_no_change_cas":true,"latest_no_change_bytes_and_revision":true}),
        )?;
        Ok(())
    };
    test().unwrap();
}

#[test]
#[ignore = "requires genuine nonempty Pending and Consumed U12_CYCLE_SEED_ROOT, same-executable config, exclusive output"]
fn u12_normal_full_tutorial_preserves_real_pending_and_consumed_profiles() {
    let test = || -> Result<()> {
        let config = env_path("U12_NORMAL_CONFIG")?;
        let seeds = env_path("U12_CYCLE_SEED_ROOT")?;
        assert_eq!(read(&seeds.join("RESULT.json"))?["passed"], true);
        let out = env_path("U12_NORMAL_OUTPUT")?.join("full_tutorial_profiles");
        fs::create_dir(&out)?;
        for source in ["first_return_pending", "second_return_consumed"] {
            let case = out.join(source);
            fs::create_dir(&case)?;
            let expected = read(&seeds.join(source).join("state.json"))?["profile"].clone();
            let root = case.join("save");
            copy_save(&seeds.join(source).join("save"), &root)?;
            let finished = child(
                &config,
                &root,
                &case.join("tutorial"),
                &seeds,
                "tutorial",
                source,
                None,
            )?
            .unwrap();
            assert_eq!(finished["profile"], expected);
            assert!(finished["checkpoint"].is_null());
            let reopened = child(
                &config,
                &root,
                &case.join("fresh_reopen"),
                &seeds,
                "recover",
                source,
                None,
            )?
            .unwrap();
            assert_eq!(reopened["profile"], expected);
            assert!(reopened["checkpoint"].is_null());
            assert_eq!(reopened["files"], finished["files"]);
            save(
                &case.join("RESULT.json"),
                &json!({"passed":true,"profile":expected,"full_actual_tutorial":true,"fresh_reopen":true}),
            )?;
        }
        save(
            &out.join("RESULT.json"),
            &json!({"passed":true,"complete_tutorials":2,"nonempty_actual_return_profiles":true}),
        )?;
        Ok(())
    };
    test().unwrap();
}
