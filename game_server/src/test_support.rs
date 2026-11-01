//! Test input adapters through the public application wire contract.
use game_core::application::*;
use game_core::game::{
    behavior::RunSnapshotDto, encounter_compendium::EncounterCompendiumProfileState,
};
use serde_json::{json, Value};
use uuid::Uuid;

pub(crate) fn snake_name(name: &str) -> String {
    name.chars()
        .enumerate()
        .fold(String::new(), |mut out, (index, ch)| {
            if ch.is_uppercase() && index > 0 {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
            out
        })
}

macro_rules! fields {
    ($map:ident;) => {};
    ($map:ident; $key:ident : $value:expr $(, $($tail:tt)*)?) => {
        $map.insert(stringify!($key).to_owned(), serde_json::to_value($value).unwrap());
        $crate::test_support::fields!($map; $($($tail)*)?);
    };
    ($map:ident; $key:ident $(, $($tail:tt)*)?) => {
        $map.insert(stringify!($key).to_owned(), serde_json::to_value($key).unwrap());
        $crate::test_support::fields!($map; $($($tail)*)?);
    };
}
pub(crate) use fields;
macro_rules! command {
    ($kind:ident { $($fields:tt)* }) => {{
        let mut fields = serde_json::Map::new();
        $crate::test_support::fields!(fields; $($fields)*);
        fields.insert("type".into(), serde_json::Value::String($crate::test_support::snake_name(stringify!($kind))));
        game_core::application::ApplicationCommand::from_document(serde_json::Value::Object(fields).into()).unwrap()
    }};
    ($kind:ident) => { $crate::test_support::command!($kind {}) };
}
pub(crate) use command;

// Explicit serialized public-result fixtures for transport-only mapping tests.
// Production integration tests produce their results through CoreApplication.
pub(crate) fn result_envelope(
    kind: ApplicationResultKind,
    mut payload: Value,
) -> ApplicationResult {
    if matches!(
        kind,
        ApplicationResultKind::DeploymentRangePreview | ApplicationResultKind::SkillTargetPreview
    ) {
        payload = payload["result"].take();
    }
    if kind == ApplicationResultKind::RunComplete {
        payload = payload["map"].take();
    }
    let disposition = if matches!(
        kind,
        ApplicationResultKind::DeploymentRangePreview
            | ApplicationResultKind::SkillTargetPreview
            | ApplicationResultKind::BattleState
    ) {
        "read_only"
    } else {
        "accepted"
    };
    serde_json::from_value(json!({ "kind": kind, "meaning": kind.meaning(), "disposition": disposition, "payload": payload })).unwrap()
}
macro_rules! result {
    ($kind:ident { $($fields:tt)* }) => {{
        let mut fields = serde_json::Map::new();
        $crate::test_support::fields!(fields; $($fields)*);
        $crate::test_support::result_envelope(game_core::application::ApplicationResultKind::$kind, serde_json::Value::Object(fields))
    }};
}
pub(crate) use result;

pub(crate) trait ApplicationFixture {
    fn execute_fixture(
        &mut self,
        player: Uuid,
        command: ApplicationCommand,
    ) -> Result<ApplicationResult, ApplicationError>;
    fn run_snapshot_fixture(&self) -> RunSnapshotDto;
    fn merge_compendium_fixture(
        &mut self,
        profile: EncounterCompendiumProfileState,
    ) -> Result<(), ApplicationError>;
}
impl ApplicationFixture for CoreApplication {
    fn execute_fixture(
        &mut self,
        player: Uuid,
        command: ApplicationCommand,
    ) -> Result<ApplicationResult, ApplicationError> {
        self.execute(player, command, ApplicationExecutionContext::default())
    }
    fn run_snapshot_fixture(&self) -> RunSnapshotDto {
        serde_json::from_value(
            self.snapshot(ApplicationSnapshotKind::Run)
                .unwrap()
                .read_model()
                .as_value()
                .clone(),
        )
        .unwrap()
    }
    fn merge_compendium_fixture(
        &mut self,
        profile: EncounterCompendiumProfileState,
    ) -> Result<(), ApplicationError> {
        self.merge_encounter_compendium_profile(serde_json::to_value(profile).unwrap().into())
    }
}

pub(crate) fn error_envelope(kind: ApplicationErrorKind, details: Value) -> ApplicationError {
    let code = if kind == ApplicationErrorKind::ActionUnavailable {
        details["reason"].as_str().unwrap().to_owned()
    } else {
        serde_json::to_value(kind)
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned()
    };
    serde_json::from_value(json!({"kind": kind, "code": code, "details": details})).unwrap()
}
macro_rules! error {
    ($kind:ident { $($fields:tt)* }) => {{
        let mut fields = serde_json::Map::new();
        $crate::test_support::fields!(fields; $($fields)*);
        $crate::test_support::error_envelope(game_core::application::ApplicationErrorKind::$kind, serde_json::Value::Object(fields))
    }};
    ($kind:ident) => { $crate::test_support::error_envelope(game_core::application::ApplicationErrorKind::$kind, serde_json::Value::Null) };
}
pub(crate) use error;
