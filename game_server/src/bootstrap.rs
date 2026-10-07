//! Host filesystem adapter. Core receives bytes/settings and a retained generation.
use crate::env::Settings;
use game_core::{
    game::data::{
        content_pack::{self, ContentSelection, InputLimits},
        ValidatedGameData,
    },
    runtime_safety::{RuntimeSafetySettings, MAX_SETTINGS_BYTES},
};
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

fn read_bounded(path: &Path, limit: usize) -> io::Result<Vec<u8>> {
    let invalid = |message: String| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: {message}", path.display()),
        )
    };
    let before = std::fs::metadata(path).map_err(|e| invalid(e.to_string()))?;
    if !before.is_file() || before.len() > limit as u64 {
        return Err(invalid(format!(
            "expected a regular file of at most {limit} bytes"
        )));
    }
    let file = File::open(path).map_err(|e| invalid(e.to_string()))?;
    if !file.metadata()?.is_file() {
        return Err(invalid("opened input is not a regular file".into()));
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| invalid(e.to_string()))?;
    if bytes.len() > limit {
        return Err(invalid(format!("input exceeds {limit} bytes")));
    }
    Ok(bytes)
}

pub fn load_runtime_safety(path: Option<&Path>) -> io::Result<Arc<RuntimeSafetySettings>> {
    let Some(path) = path else {
        return Ok(RuntimeSafetySettings::embedded_shared());
    };
    let bytes = read_bounded(path, MAX_SETTINGS_BYTES)
        .map_err(|e| io::Error::new(e.kind(), format!("runtime_safety_path: {e}")))?;
    RuntimeSafetySettings::from_ron(&path.display().to_string(), &bytes)
        .map(Arc::new)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

pub fn load_game_data(settings: &Settings) -> io::Result<ValidatedGameData> {
    // An explicit invalid profile fails before any content parse/publication.
    let profile = load_runtime_safety(settings.runtime_safety_path.as_deref())?;
    let selection_bytes = read_bounded(&settings.content.selection_path, 64 * 1024)?;
    let selection: ContentSelection = serde_json::from_slice(&selection_bytes).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: {e}", settings.content.selection_path.display()),
        )
    })?;
    content_pack::load_with_runtime_safety(
        &settings.content.trusted_root,
        &selection,
        &InputLimits::default(),
        profile,
    )
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_safety_host_file_is_explicit_bounded_and_never_falls_back() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("safety.ron");
        let default = load_runtime_safety(None).unwrap();
        let missing = load_runtime_safety(Some(&path)).unwrap_err().to_string();
        assert!(missing.contains("runtime_safety_path") && missing.contains("safety.ron"));
        std::fs::write(
            &path,
            game_core::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON,
        )
        .unwrap();
        assert_eq!(
            load_runtime_safety(Some(&path)).unwrap().effective_sha256(),
            default.effective_sha256()
        );
        std::fs::write(&path, "(schema_version: 99)").unwrap();
        let invalid = load_runtime_safety(Some(&path)).unwrap_err().to_string();
        assert!(invalid.contains("safety.ron"));
        assert!(load_runtime_safety(Some(root.path())).is_err());
        std::fs::write(&path, vec![b' '; MAX_SETTINGS_BYTES + 1]).unwrap();
        assert!(load_runtime_safety(Some(&path)).is_err());
        assert_eq!(
            load_runtime_safety(None).unwrap().effective_sha256(),
            default.effective_sha256()
        );
    }

    #[test]
    fn real_pack_bootstrap_keeps_concurrent_profiles_separate_and_rejects_settings_first() {
        let root = tempfile::tempdir().unwrap();
        let content = root.path().join("content");
        std::fs::create_dir(&content).unwrap();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../game_resources");
        content_pack::package_official(&source, &content.join("official"), &InputLimits::default())
            .unwrap();
        let selection = root.path().join("selection.json");
        std::fs::write(&selection, r#"{"packs":["official"],"winners":[]}"#).unwrap();
        let profile = root.path().join("profile.ron");
        std::fs::write(
            &profile,
            std::str::from_utf8(game_core::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON)
                .unwrap()
                .replace("max_events: 8192", "max_events: 2"),
        )
        .unwrap();
        let settings = |profile: Option<&Path>| {
            serde_json::from_value::<Settings>(serde_json::json!({
            "logging": {"directory": "logs", "filename": "test"},
            "server": {"bind_address": "127.0.0.1", "port": 0, "log_level": "info"},
            "session": {"heartbeat_interval_seconds": 30, "heartbeat_timeout": 120},
            "retry": {"message_max_elapsed_time_ms": 500, "message_initial_interval_ms": 10, "message_max_interval_ms": 100},
            "persistence": {"active_run_directory": "unused", "player_profile_directory": "unused"},
            "content": {"trusted_root": content, "selection_path": selection},
            "runtime_safety_path": profile
        })).unwrap()
        };
        let original_settings = settings(None);
        let selected_settings = settings(Some(&profile));
        let (original, selected) = std::thread::scope(|scope| {
            let first = scope.spawn(|| load_game_data(&original_settings).unwrap());
            let second = scope.spawn(|| load_game_data(&selected_settings).unwrap());
            (first.join().unwrap(), second.join().unwrap())
        });
        assert_eq!(original.identity(), selected.identity());
        assert_eq!(original.runtime_safety().battle_history().max_events, 8192);
        assert_eq!(selected.runtime_safety().battle_history().max_events, 2);
        let mut invalid = selected_settings;
        invalid.content.selection_path = root.path().join("missing-selection.json");
        std::fs::write(&profile, "invalid settings").unwrap();
        let failure = load_game_data(&invalid).err().unwrap().to_string();
        assert!(failure.contains("profile.ron"), "{failure}");
        assert!(
            !failure.contains("missing-selection"),
            "settings must fail before content acquisition"
        );
        assert_eq!(
            selected.runtime_safety().battle_history().max_events,
            2,
            "a retained generation is immutable after host-file changes"
        );
    }
}
