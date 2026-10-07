use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub logging: LoggingSettings,
    pub server: ServerSettings,
    pub session: SessionSettings,
    pub retry: RetrySettings,
    pub persistence: PersistenceSettings,
    pub content: ContentSettings,
    #[serde(default)]
    pub runtime_safety_path: Option<PathBuf>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        println!("Loading configuration for RUN_MODE: {}", &run_mode);

        let s = Config::builder()
            // Load environment-specific file (e.g., development.toml, production.toml)
            .add_source(
                File::with_name(&format!("config/{}", run_mode))
                    .format(FileFormat::Toml)
                    .required(true),
            )
            // Add environment variables (e.g., APP_SERVER__PORT=8000)
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        s.try_deserialize()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SessionSettings {
    pub heartbeat_interval_seconds: u64,
    pub heartbeat_timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub bind_address: String,
    pub port: u16,
    pub log_level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingSettings {
    pub directory: String,
    pub filename: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RetrySettings {
    pub message_max_elapsed_time_ms: u64,
    pub message_initial_interval_ms: u64,
    pub message_max_interval_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PersistenceSettings {
    /// Host-provided local persistent-data root. This is one local-install
    /// aggregate and is deliberately not keyed by transport player id.
    pub active_run_directory: String,
    /// Separate local-install profile aggregate. It may share a parent root
    /// with active-run files but has its own envelope, lock and transaction.
    pub player_profile_directory: String,
}

impl PersistenceSettings {
    pub fn active_run_directory(&self) -> Result<PathBuf, String> {
        resolve_application_persistent_data_directory(&self.active_run_directory)
    }

    pub fn player_profile_directory(&self) -> Result<PathBuf, String> {
        resolve_application_persistent_data_directory(&self.player_profile_directory)
    }
}

fn resolve_application_persistent_data_directory(configured: &str) -> Result<PathBuf, String> {
    let configured = configured.trim();
    if configured.is_empty() {
        return Err("persistence directory must not be empty".to_string());
    }

    let configured_path = Path::new(configured);
    if configured_path.is_absolute() {
        return Ok(configured_path.to_path_buf());
    }

    application_persistent_data_root().map(|root| root.join(configured_path))
}

fn application_persistent_data_root() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        return std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|root| root.join("Ark"))
            .ok_or_else(|| "LOCALAPPDATA is unavailable for local persistence".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|root| root.join("Library/Application Support/Ark"))
            .ok_or_else(|| "HOME is unavailable for local persistence".to_string());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(root) = std::env::var_os("XDG_DATA_HOME") {
            return Ok(PathBuf::from(root).join("ark"));
        }
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|root| root.join(".local/share/ark"))
            .ok_or_else(|| {
                "XDG_DATA_HOME and HOME are unavailable for local persistence".to_string()
            });
    }

    #[allow(unreachable_code)]
    Err("this platform has no application persistent-data resolver".to_string())
}

#[cfg(test)]
mod persistence_tests {
    use super::resolve_application_persistent_data_directory;
    use std::path::Path;

    #[test]
    fn absolute_persistence_path_is_preserved() {
        let root = std::env::temp_dir().join("ark-explicit-persistence");
        assert_eq!(
            resolve_application_persistent_data_directory(root.to_str().unwrap()).unwrap(),
            root
        );
    }

    #[test]
    fn relative_persistence_path_resolves_outside_the_work_directory() {
        let resolved = resolve_application_persistent_data_directory("persistent_data").unwrap();
        assert!(resolved.is_absolute());
        assert!(resolved.ends_with(Path::new("persistent_data")));
        assert_ne!(resolved, Path::new("persistent_data"));
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ContentSettings {
    /// Explicit production pack root; relative paths are resolved from host cwd.
    pub trusted_root: PathBuf,
    pub selection_path: PathBuf,
}
