//! Public setup only: external pack loading, command parsing, and read models.
use game_core::{
    application::{
        ApplicationAdminCommand, ApplicationCommand, ApplicationError, ApplicationResult,
        ApplicationSnapshotKind, CoreApplication,
    },
    game::{
        behavior::{
            EmployeeRosterSnapshotDto, GameStateContextDto, InventorySnapshotDto, RunSnapshotDto,
        },
        data::{
            content_pack::{
                self, CatalogFile, CatalogKind, ContentContract, ContentSelection, InputLimits,
                PackManifest, PackNamespace,
            },
            ValidatedGameData,
        },
        relic::RelicStateDto,
    },
};
use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::OnceLock,
};
use uuid::Uuid;

pub const PLAYER: Uuid = Uuid::from_u128(0xa5080001);

pub struct PublicContent {
    pub root: PathBuf,
    pub selection: ContentSelection,
    pub data: ValidatedGameData,
}

pub fn content() -> &'static PublicContent {
    static CONTENT: OnceLock<PublicContent> = OnceLock::new();
    CONTENT.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("u08-public-{}", Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        content_pack::package_official(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../game_resources"),
            &root.join("official"),
            &InputLimits::default(),
        )
        .unwrap();
        let probe = root.join("u08_public");
        fs::create_dir(&probe).unwrap();
        let catalogs = [
            (
                CatalogKind::Relics,
                "relics.ron",
                include_str!("../fixtures/u08/public001/relics.ron"),
            ),
            (
                CatalogKind::Skills,
                "skills.ron",
                include_str!("../fixtures/u08/public001/skills.ron"),
            ),
            (
                CatalogKind::SkillFragments,
                "fragments.ron",
                include_str!("../fixtures/u08/public001/fragments.ron"),
            ),
            (
                CatalogKind::Equipment,
                "equipment.ron",
                include_str!("../fixtures/u08/public001/equipment.ron"),
            ),
            (
                CatalogKind::Events,
                "events.ron",
                include_str!("../fixtures/u08/public001/events.ron"),
            ),
            (
                CatalogKind::Rewards,
                "rewards.ron",
                include_str!("../fixtures/u08/public001/rewards.ron"),
            ),
            (
                CatalogKind::Shops,
                "shops.ron",
                include_str!("../fixtures/u08/public001/shops.ron"),
            ),
        ];
        for (_, name, input) in &catalogs {
            write_new(&probe.join(name), input.as_bytes());
        }
        let manifest = PackManifest {
            pack_id: "u08_public".into(),
            pack_version: "1".into(),
            namespace: PackNamespace::Global,
            contract: ContentContract::current(),
            required_capabilities: vec![
                "gameplay_catalogs_v2".into(),
                "literal_effect_authoring_v1".into(),
            ],
            dependencies: vec!["official".into()],
            files: catalogs
                .iter()
                .map(|(kind, name, _)| CatalogFile {
                    kind: *kind,
                    path: (*name).into(),
                })
                .collect(),
            operations: vec![],
        };
        write_new(
            &probe.join("pack.ron"),
            ron::ser::to_string(&manifest).unwrap().as_bytes(),
        );
        let selection = ContentSelection {
            packs: vec!["official".into(), "u08_public".into()],
            winners: vec![],
        };
        write_new(
            &root.join("selection.json"),
            &serde_json::to_vec_pretty(&selection).unwrap(),
        );
        let data = content_pack::load(&root, &selection, &InputLimits::default()).unwrap();
        println!(
            "U08_PUBLIC_CONTENT root={} generation={}",
            root.display(),
            data.identity().semantic_sha256
        );
        PublicContent {
            root,
            selection,
            data,
        }
    })
}

fn write_new(path: &Path, bytes: &[u8]) {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap()
        .write_all(bytes)
        .unwrap();
}

pub fn execute(
    app: &mut CoreApplication,
    command: Value,
) -> Result<ApplicationResult, ApplicationError> {
    let command = ApplicationCommand::from_document(command.into())?;
    app.execute(PLAYER, command, Default::default())
}

pub fn command(app: &mut CoreApplication, value: Value) -> Value {
    execute(app, value.clone())
        .unwrap_or_else(|error| panic!("command {value}: {error:?}"))
        .payload()
        .as_value()
        .clone()
}

pub fn admin(app: &mut CoreApplication, value: Value) -> Value {
    app.execute_admin(ApplicationAdminCommand::from_document(value.clone().into()).unwrap())
        .unwrap_or_else(|error| panic!("admin command {value}: {error:?}"))
        .payload()
        .as_value()
        .clone()
}

pub fn snapshot(app: &CoreApplication, kind: ApplicationSnapshotKind) -> Value {
    app.snapshot(kind).unwrap().read_model().as_value().clone()
}

pub fn roster(app: &CoreApplication) -> EmployeeRosterSnapshotDto {
    serde_json::from_value(snapshot(app, ApplicationSnapshotKind::EmployeeRoster)).unwrap()
}

pub fn inventory(app: &CoreApplication) -> InventorySnapshotDto {
    serde_json::from_value(snapshot(app, ApplicationSnapshotKind::Inventory)).unwrap()
}

pub fn app() -> (CoreApplication, Vec<Uuid>) {
    let mut app = CoreApplication::new(content().data.clone(), 0xa508);
    command(
        &mut app,
        json!({"type":"start_new_game", "game_mode":"Expedition", "replace_active_run":false}),
    );
    for _ in 0..128 {
        let run: RunSnapshotDto =
            serde_json::from_value(snapshot(&app, ApplicationSnapshotKind::Run)).unwrap();
        if let Some(vn) = run.visual_novel {
            let value = if let Some(choice) = vn.choices.iter().find(|choice| choice.selectable) {
                json!({"type":"select_visual_novel_choice", "session_id":vn.session_id, "current_scene_id":vn.current_scene_id, "choice_id":choice.choice_id})
            } else {
                json!({"type":"advance_visual_novel", "session_id":vn.session_id, "current_scene_id":vn.current_scene_id})
            };
            command(&mut app, value);
            continue;
        }
        if let GameStateContextDto::SelectingStarterEmployees {
            required_count,
            candidates,
        } = run.game_state_context
        {
            let ids: Vec<_> = candidates
                .into_iter()
                .take(required_count)
                .map(|row| row.employee_definition_id)
                .collect();
            command(
                &mut app,
                json!({"type":"select_starter_employees", "employee_definition_ids":ids,"loan_copy_ids":[]}),
            );
            continue;
        }
        let employees = roster(&app).combat_eligible_employee_ids;
        assert!(
            employees.len() >= 2,
            "two normal starter employees are required"
        );
        return (app, employees);
    }
    panic!("public run-start flow exceeded its fixture bound");
}

pub fn relic_state(app: &mut CoreApplication) -> RelicStateDto {
    serde_json::from_value(command(app, json!({"type":"request_relic_state"}))).unwrap()
}

pub fn grant_relic(app: &mut CoreApplication, definition: &str) -> Uuid {
    let before: Vec<_> = relic_state(app)
        .copies
        .into_iter()
        .map(|copy| copy.copy_uuid)
        .collect();
    admin(
        app,
        json!({"type":"admin_grant_relic", "definition_id":definition, "count":1}),
    );
    relic_state(app)
        .copies
        .into_iter()
        .find(|copy| copy.definition_id == definition && !before.contains(&copy.copy_uuid))
        .unwrap()
        .copy_uuid
}

pub fn equip_relic(app: &mut CoreApplication, copy: Uuid, employee: Uuid, slot: u8) {
    command(
        app,
        json!({"type":"equip_relic", "copy_uuid":copy, "employee_uuid":employee, "slot":slot}),
    );
}

pub fn equip_weapon(app: &mut CoreApplication, employee: Uuid, definition: &str) -> Uuid {
    let granted = admin(
        app,
        json!({"type":"admin_grant_equipment", "definition_id":definition, "count":1}),
    );
    let copy: Uuid = serde_json::from_value(granted["item_uuids"][0].clone()).unwrap();
    command(
        app,
        json!({"type":"equip_item", "item_uuid":copy, "target_unit":employee}),
    );
    copy
}

pub fn equip_fragment(app: &mut CoreApplication, employee: Uuid, definition: &str) -> Uuid {
    if let Some(previous) = inventory(app)
        .skill_fragments
        .into_iter()
        .find(|row| row.copy.equipped_to == Some(employee))
    {
        command(
            app,
            json!({"type":"unequip_skill_fragment", "employee_uuid":employee, "copy_uuid":previous.copy.copy_uuid}),
        );
    }
    let before: Vec<_> = inventory(app)
        .skill_fragments
        .into_iter()
        .map(|row| row.copy.copy_uuid)
        .collect();
    admin(
        app,
        json!({"type":"admin_grant_skill_fragment", "fragment_id":definition, "count":1}),
    );
    let copy = inventory(app)
        .skill_fragments
        .into_iter()
        .find(|row| {
            row.copy.definition_id.as_str() == definition && !before.contains(&row.copy.copy_uuid)
        })
        .unwrap()
        .copy
        .copy_uuid;
    command(
        app,
        json!({"type":"equip_skill_fragment", "employee_uuid":employee, "copy_uuid":copy}),
    );
    copy
}

pub fn public_roots(app: &CoreApplication) -> Value {
    let snapshots: Vec<_> = ApplicationSnapshotKind::ALL
        .iter()
        .map(|kind| snapshot(app, *kind))
        .collect();
    let checkpoint = app.export_active_run_checkpoint().unwrap().map(|saved| {
        json!({
            "bytes": saved.core_payload_json(), "loads_used": saved.loads_used(),
            "run_peak_certified_power": saved.run_peak_certified_power(),
        })
    });
    json!({"snapshots":snapshots, "checkpoint":checkpoint})
}

pub fn numeric_plan(app: &mut CoreApplication, employee: Uuid) -> Value {
    command(
        app,
        json!({"type":"request_relic_numeric_plan", "employee_uuid":employee}),
    )
}

pub fn select_form(app: &mut CoreApplication, fragment: Uuid, form: &str) {
    admin(app, json!({"type":"admin_enter_maintenance"}));
    admin(app, json!({"type":"admin_set_enkephalin", "amount":100}));
    command(
        app,
        json!({"type":"awaken_skill_fragment", "copy_uuid":fragment, "form_id":form}),
    );
    command(
        app,
        json!({"type":"select_skill_fragment_form", "copy_uuid":fragment, "form_id":form}),
    );
    command(app, json!({"type":"complete_node"}));
}

/// Admin sets only the Reward node context. ClaimReward performs the authored
/// grant transaction; this helper does not establish a full map traversal.
pub fn claim_relic_reward(app: &mut CoreApplication) -> Uuid {
    let before: Vec<_> = relic_state(app)
        .copies
        .into_iter()
        .map(|copy| copy.copy_uuid)
        .collect();
    admin(
        app,
        json!({"type":"admin_enter_reward", "reward_pool_id":"u08_public_relic_reward_pool", "mode":"ClaimAll", "can_skip":false}),
    );
    command(app, json!({"type":"claim_reward"}));
    let after = relic_state(app);
    let added: Vec<_> = after
        .copies
        .into_iter()
        .filter(|copy| !before.contains(&copy.copy_uuid))
        .collect();
    assert_eq!(added.len(), 1);
    assert_eq!(added[0].definition_id, "u08_public_blend");
    assert_eq!(json!(added[0].variant), "Normal");
    command(app, json!({"type":"exit_reward"}));
    added[0].copy_uuid
}

pub fn selected_event(app: &CoreApplication) -> Value {
    snapshot(app, ApplicationSnapshotKind::SelectedEvent)
}
