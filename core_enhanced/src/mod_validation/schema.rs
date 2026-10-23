use super::ModValidationError;
use crate::game::data::content_pack::{ContentIdentity, ContentSelection, WriterSelection};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

pub const MAX_REGISTRY_BYTES: usize = 1024 * 1024;
pub const MAX_MODS: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModDefinition {
    pub mod_id: String,
    pub mod_version: String,
    pub packs: Vec<PathBuf>,
    pub dependencies: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModSelection {
    pub mods: Vec<String>,
    pub winners: Vec<WriterSelection>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModRegistry {
    pub schema_version: u32,
    pub mods: Vec<ModDefinition>,
    pub selection: ModSelection,
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn valid_path(p: &std::path::Path) -> bool {
    p.to_str().is_some_and(|s| {
        !s.is_empty()
            && s.len() <= 4096
            && !s.chars().any(char::is_control)
            && !s.contains(['\\', ':'])
            && s.split('/').all(|x| !x.is_empty() && x != "." && x != "..")
    }) && !p.is_absolute()
}
impl ModRegistry {
    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ModValidationError> {
        if bytes.len() > MAX_REGISTRY_BYTES {
            return Err(ModValidationError::invalid(
                "registry",
                "registry byte limit",
            ));
        }
        let registry: Self = serde_json::from_slice(bytes)
            .map_err(|e| ModValidationError::invalid("registry", e.to_string()))?;
        registry.validate()?;
        Ok(registry)
    }
    pub fn validate(&self) -> Result<(), ModValidationError> {
        if self.schema_version != 1
            || self.mods.len() > MAX_MODS
            || self.selection.mods.len() > MAX_MODS
            || self.selection.winners.len() > 4096
        {
            return Err(ModValidationError::invalid(
                "registry",
                "unsupported version or finite count limit",
            ));
        }
        let mut ids = BTreeSet::new();
        let mut packs = BTreeSet::new();
        for m in &self.mods {
            if !valid_id(&m.mod_id)
                || m.mod_version.is_empty()
                || m.mod_version.len() > 128
                || m.mod_version.trim() != m.mod_version
                || m.mod_version.chars().any(char::is_control)
                || !ids.insert(&m.mod_id)
                || m.packs.is_empty()
                || m.packs.len() > 128
                || m.dependencies.len() > MAX_MODS
            {
                return Err(ModValidationError::invalid(
                    format!("mods.{}", m.mod_id),
                    "invalid/duplicate mod or finite member limit",
                ));
            }
            for p in &m.packs {
                if !valid_path(p) || !packs.insert(p) {
                    return Err(ModValidationError::invalid(
                        format!("mods.{}.packs", m.mod_id),
                        "invalid path or duplicate pack ownership",
                    ));
                }
            }
            if m.dependencies.iter().collect::<BTreeSet<_>>().len() != m.dependencies.len() {
                return Err(ModValidationError::invalid(
                    format!("mods.{}.dependencies", m.mod_id),
                    "duplicate dependency",
                ));
            }
        }
        let map: BTreeMap<_, _> = self.mods.iter().map(|m| (m.mod_id.as_str(), m)).collect();
        for m in &self.mods {
            for dependency in &m.dependencies {
                if !map.contains_key(dependency.as_str()) {
                    return Err(ModValidationError::invalid(
                        format!("mods.{}.dependencies", m.mod_id),
                        format!("missing mod {dependency}"),
                    ));
                }
            }
            self.closure_for(&[m.mod_id.clone()])?;
        }
        if self.selection.mods.iter().collect::<BTreeSet<_>>().len() != self.selection.mods.len() {
            return Err(ModValidationError::invalid(
                "selection.mods",
                "duplicate selected mod",
            ));
        }
        self.closure_for(&self.selection.mods)?;
        let mut targets = BTreeSet::new();
        if self
            .selection
            .winners
            .iter()
            .any(|w| !targets.insert(&w.target))
        {
            return Err(ModValidationError::invalid(
                "selection.winners",
                "duplicate explicit writer target",
            ));
        }
        Ok(())
    }
    pub(crate) fn closure_for(
        &self,
        selected: &[String],
    ) -> Result<Vec<ModDefinition>, ModValidationError> {
        fn visit<'a>(
            id: &str,
            map: &BTreeMap<&'a str, &'a ModDefinition>,
            active: &mut BTreeSet<String>,
            done: &mut BTreeSet<String>,
        ) -> Result<(), ModValidationError> {
            if done.contains(id) {
                return Ok(());
            }
            if !active.insert(id.to_owned()) {
                return Err(ModValidationError::invalid(
                    "dependencies",
                    format!("mod cycle at {id}"),
                ));
            }
            let m = map.get(id).ok_or_else(|| {
                ModValidationError::invalid("selection.mods", format!("missing mod {id}"))
            })?;
            for d in &m.dependencies {
                visit(d, map, active, done)?;
            }
            active.remove(id);
            done.insert(id.to_owned());
            Ok(())
        }
        let map = self.mods.iter().map(|m| (m.mod_id.as_str(), m)).collect();
        let mut done = BTreeSet::new();
        let mut active = BTreeSet::new();
        for id in selected {
            visit(id, &map, &mut active, &mut done)?;
        }
        Ok(self
            .mods
            .iter()
            .filter(|m| done.contains(&m.mod_id))
            .cloned()
            .collect())
    }
    pub(crate) fn content_selection(
        &self,
        selected: &[ModDefinition],
        base: &ContentSelection,
    ) -> ContentSelection {
        let packs: BTreeSet<_> = base
            .packs
            .iter()
            .chain(selected.iter().flat_map(|m| m.packs.iter()))
            .cloned()
            .collect();
        let mut winners: BTreeMap<_, _> = base
            .winners
            .iter()
            .map(|w| (w.target.clone(), w.clone()))
            .collect();
        winners.extend(
            self.selection
                .winners
                .iter()
                .map(|w| (w.target.clone(), w.clone())),
        );
        ContentSelection {
            packs: packs.into_iter().collect(),
            winners: winners.into_values().collect(),
        }
    }
}

/// Compatibility requirements, never a player-save approval or cached capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredConfiguration {
    pub schema_version: u32,
    pub content_identity: ContentIdentity,
    pub mods: Vec<ModDefinition>,
    pub packs: Vec<RequiredPack>,
    pub selection: ContentSelection,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredPack {
    pub path: String,
    pub pack_id: String,
    pub pack_version: String,
    pub dependencies: Vec<String>,
}
impl RequiredConfiguration {
    pub fn validate(&self) -> Result<(), ModValidationError> {
        if self.schema_version != 1
            || self.mods.len() > MAX_MODS
            || self.packs.len() > 128
            || self.selection.packs.len() > 128
            || self.selection.winners.len() > 4096
        {
            return Err(ModValidationError::protected(
                "invalid required configuration bounds/version",
            ));
        }
        let registry = ModRegistry {
            schema_version: 1,
            mods: self.mods.clone(),
            selection: ModSelection {
                mods: self.mods.iter().map(|m| m.mod_id.clone()).collect(),
                winners: self.selection.winners.clone(),
            },
        };
        registry
            .validate()
            .map_err(|e| ModValidationError::protected(e.to_string()))?;
        let paths: BTreeSet<_> = self.packs.iter().map(|p| p.path.as_str()).collect();
        if self.selection.packs.iter().collect::<BTreeSet<_>>().len() != self.selection.packs.len()
        {
            return Err(ModValidationError::protected(
                "duplicate required selection pack",
            ));
        }
        let ids: BTreeSet<_> = self.packs.iter().map(|p| p.pack_id.as_str()).collect();
        if paths.len() != self.packs.len()
            || ids.len() != self.packs.len()
            || self.packs.iter().any(|p| {
                !valid_path(std::path::Path::new(&p.path))
                    || !valid_id(&p.pack_id)
                    || p.pack_version.is_empty()
                    || p.dependencies.len() > 128
                    || p.dependencies.iter().any(|d| !ids.contains(d.as_str()))
            })
            || self
                .selection
                .packs
                .iter()
                .any(|p| !paths.contains(p.to_string_lossy().as_ref()))
            || paths.len() != self.selection.packs.len()
            || self
                .mods
                .iter()
                .flat_map(|m| &m.packs)
                .any(|p| !paths.contains(p.to_string_lossy().as_ref()))
        {
            return Err(ModValidationError::protected(
                "invalid required pack membership/dependencies",
            ));
        }
        if self.content_identity.semantic_sha256.len() != 64
            || !self
                .content_identity
                .semantic_sha256
                .bytes()
                .all(|x| x.is_ascii_hexdigit())
        {
            return Err(ModValidationError::protected(
                "invalid full content identity",
            ));
        }
        Ok(())
    }
    pub fn ensure_compatible(&self, required: &Self) -> Result<(), ModValidationError> {
        self.validate()?;
        required.validate()?;
        if self != required {
            return Err(ModValidationError::protected("required mod/pack/writer/full content configuration differs from approved configuration"));
        }
        Ok(())
    }
}
