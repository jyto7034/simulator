use super::{
    digest, ModDefinition, ModRegistry, ModValidationError, ModValidationErrorKind,
    RequiredConfiguration, RequiredPack,
};
use crate::{
    authoring::{self, CancellationToken, ExecutionPhase, ExecutionUsage},
    game::data::{
        content_pack::{self, ContentSelection, InputLimits},
        ValidatedGameData,
    },
    runtime_safety::RuntimeSafetySettings,
};
use serde::Serialize;
use std::{collections::BTreeMap, path::Path, sync::Arc};

/// Chosen by the trusted host suite, never by a mod ID convention.
pub struct TrustedBase {
    pub(crate) selection: ContentSelection,
    pub(crate) publication: Arc<content_pack::FrozenPublication>,
    pub(crate) settings: Arc<RuntimeSafetySettings>,
}
impl TrustedBase {
    pub fn acquire(
        root: &Path,
        selection: ContentSelection,
        limits: &InputLimits,
        settings: Arc<RuntimeSafetySettings>,
        cancel: &CancellationToken,
    ) -> Result<Self, ModValidationError> {
        let (result, _) = authoring::control::run(
            &settings.authoring_execution(),
            cancel,
            ExecutionPhase::Acquisition,
            || {
                let frozen =
                    content_pack::FrozenContent::acquire(root, &selection, limits, &settings)?;
                frozen.publish(&selection, limits, settings.clone())
            },
        )
        .map_err(|e| ModValidationError::execution("trusted_base", e))?;
        let publication = result.map_err(|e| {
            ModValidationError::new(
                ModValidationErrorKind::ContentFailed,
                "trusted_base",
                e.to_string(),
            )
        })?;
        Ok(Self {
            selection,
            publication: Arc::new(publication),
            settings,
        })
    }
    pub fn generation(&self) -> &ValidatedGameData {
        &self.publication.token
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateView {
    pub identity: String,
    pub required: RequiredConfiguration,
    pub runtime_safety_sha256: String,
    pub source_receipt: content_pack::SourceReceipt,
    pub changed_definitions: Vec<content_pack::DefinitionKey>,
    pub removed_definitions: Vec<content_pack::DefinitionKey>,
}
pub struct Candidate {
    pub(crate) publication: content_pack::FrozenPublication,
    pub(crate) frozen: Arc<content_pack::FrozenContent>,
    pub(crate) registry: ModRegistry,
    pub(crate) base_selection: ContentSelection,
    base_identity: content_pack::ContentIdentity,
    base_receipt: content_pack::SourceReceipt,
    base_publication: Arc<content_pack::FrozenPublication>,
    pub(crate) limits: InputLimits,
    pub(crate) settings: Arc<RuntimeSafetySettings>,
    pub(crate) usage: ExecutionUsage,
    pub(crate) view: CandidateView,
}
impl Candidate {
    pub fn acquire(
        root: &Path,
        registry_bytes: &[u8],
        base: &TrustedBase,
        limits: &InputLimits,
        cancel: &CancellationToken,
    ) -> Result<Self, ModValidationError> {
        let registry = ModRegistry::from_json_slice(registry_bytes)?;
        Self::acquire_registry(root, registry, base, limits, cancel, false)
    }
    pub fn acquire_individual(
        root: &Path,
        registry_bytes: &[u8],
        mod_id: &str,
        base: &TrustedBase,
        limits: &InputLimits,
        cancel: &CancellationToken,
    ) -> Result<Self, ModValidationError> {
        let mut registry = ModRegistry::from_json_slice(registry_bytes)?;
        registry.selection.mods = vec![mod_id.to_owned()];
        Self::acquire_registry(root, registry, base, limits, cancel, true)
    }
    fn acquire_registry(
        root: &Path,
        mut registry: ModRegistry,
        base: &TrustedBase,
        limits: &InputLimits,
        cancel: &CancellationToken,
        individual: bool,
    ) -> Result<Self, ModValidationError> {
        let mods = registry.closure_for(&registry.selection.mods)?;
        let selection = registry.content_selection(&mods, &base.selection);
        if selection.packs.len() > limits.max_packs {
            return Err(ModValidationError::invalid(
                "selection.packs",
                "combined pack count limit",
            ));
        }
        let settings = base.settings.clone();
        let (result, usage) = authoring::control::run(
            &settings.authoring_execution(),
            cancel,
            ExecutionPhase::Acquisition,
            || {
                let frozen =
                    content_pack::FrozenContent::acquire(root, &selection, limits, &settings)?;
                if individual {
                    project_individual_winners(&mut registry, &frozen, &selection);
                }
                let selection = registry.content_selection(&mods, &base.selection);
                let captured_base_matches = frozen.matches_source_receipt(
                    &base.selection,
                    base.publication.token.source_receipt(),
                )?;
                if !captured_base_matches
                    || settings.effective_sha256()
                        != base.publication.token.runtime_safety().effective_sha256()
                {
                    return Err(crate::game::data::live_ron::LiveRonError::validation(
                        "mod_candidate",
                        "trusted_base",
                        None::<String>,
                        None::<String>,
                        "trusted base bytes changed after host suite capture",
                    ));
                }
                let publication = frozen.publish(&selection, limits, settings.clone())?;
                Ok::<_, crate::game::data::live_ron::LiveRonError>((frozen, publication))
            },
        )
        .map_err(|e| ModValidationError::execution("candidate", e))?;
        let (frozen, publication) = result.map_err(|e| {
            ModValidationError::new(
                ModValidationErrorKind::ContentFailed,
                "candidate",
                e.to_string(),
            )
        })?;
        Self::finish(
            Arc::new(frozen),
            publication,
            registry,
            mods,
            base.selection.clone(),
            &base.publication,
            limits.clone(),
            settings,
            usage,
        )
    }
    fn finish(
        frozen: Arc<content_pack::FrozenContent>,
        publication: content_pack::FrozenPublication,
        registry: ModRegistry,
        mods: Vec<ModDefinition>,
        base_selection: ContentSelection,
        base: &Arc<content_pack::FrozenPublication>,
        limits: InputLimits,
        settings: Arc<RuntimeSafetySettings>,
        usage: ExecutionUsage,
    ) -> Result<Self, ModValidationError> {
        let originals: BTreeMap<_, _> = base
            .definitions
            .iter()
            .map(|d| (&d.key, &d.fingerprint))
            .collect();
        let current: BTreeMap<_, _> = publication
            .definitions
            .iter()
            .map(|d| (&d.key, &d.fingerprint))
            .collect();
        let changed_definitions = publication
            .definitions
            .iter()
            .filter(|d| originals.get(&d.key).copied() != Some(&d.fingerprint))
            .map(|d| d.key.clone())
            .collect();
        let removed_definitions = base
            .definitions
            .iter()
            .filter(|d| !current.contains_key(&d.key))
            .map(|d| d.key.clone())
            .collect();
        let selection = registry.content_selection(&mods, &base_selection);
        let packs = frozen
            .manifests(&selection)
            .into_iter()
            .map(|(path, m)| RequiredPack {
                path,
                pack_id: m.pack_id,
                pack_version: m.pack_version,
                dependencies: m.dependencies,
            })
            .collect();
        let required = RequiredConfiguration {
            schema_version: 1,
            content_identity: publication.token.identity().clone(),
            mods,
            packs,
            selection,
        };
        required.validate()?;
        let identity = digest(&(
            &required,
            publication.token.source_receipt(),
            publication.token.runtime_safety().effective_sha256(),
        ))?;
        let view = CandidateView {
            identity,
            required,
            runtime_safety_sha256: settings.effective_sha256().into(),
            source_receipt: publication.token.source_receipt().clone(),
            changed_definitions,
            removed_definitions,
        };
        Ok(Self {
            publication,
            frozen,
            registry,
            base_selection,
            base_identity: base.token.identity().clone(),
            base_receipt: base.token.source_receipt().clone(),
            base_publication: base.clone(),
            limits,
            settings,
            usage,
            view,
        })
    }
    pub fn view(&self) -> &CandidateView {
        &self.view
    }
    pub fn registry(&self) -> &ModRegistry {
        &self.registry
    }
    pub fn coverage_plan(&self) -> Result<Arc<super::CoveragePlan>, ModValidationError> {
        super::CoveragePlan::for_candidate(self)
    }
    pub fn generation(&self) -> &ValidatedGameData {
        &self.publication.token
    }
    pub fn required_configuration(&self) -> &RequiredConfiguration {
        &self.view.required
    }
    pub fn authoring_content(&self) -> authoring::AuthoringContent {
        authoring::AuthoringContent::from_candidate(
            self.publication.token.clone(),
            self.usage.clone(),
        )
    }
    pub(crate) fn base_authoring_content(
        &self,
        cancel: &CancellationToken,
    ) -> Result<authoring::AuthoringContent, ModValidationError> {
        let (publication, usage) = authoring::control::run(
            &self.settings.authoring_execution(),
            cancel,
            ExecutionPhase::Acquisition,
            || {
                self.frozen
                    .publish(&self.base_selection, &self.limits, self.settings.clone())
            },
        )
        .map_err(|e| ModValidationError::execution("removed.trusted_base", e))?;
        let publication = publication
            .map_err(|e| ModValidationError::incomplete("removed.trusted_base", e.to_string()))?;
        if publication.token.identity() != &self.base_identity
            || publication.token.source_receipt() != &self.base_receipt
        {
            return Err(ModValidationError::incomplete(
                "removed.trusted_base",
                "frozen base correspondence changed",
            ));
        }
        Ok(authoring::AuthoringContent::from_candidate(
            publication.token,
            usage,
        ))
    }
    /// Independent dependency closure from captured original definitions; a
    /// combination failure never rewrites prior independent issuer records.
    pub fn individual(
        &self,
        mod_id: &str,
        base: &TrustedBase,
        cancel: &CancellationToken,
    ) -> Result<Self, ModValidationError> {
        let mods = self.registry.closure_for(&[mod_id.to_owned()])?;
        let mut registry = self.registry.clone();
        registry.selection.mods = vec![mod_id.into()];
        if base.publication.token.identity() != &self.base_identity
            || base.publication.token.source_receipt() != &self.base_receipt
            || base.settings.effective_sha256() != self.settings.effective_sha256()
        {
            return Err(ModValidationError::invalid(
                "individual.trusted_base",
                "foreign trusted base",
            ));
        }
        let selection = registry.content_selection(&mods, &self.base_selection);
        project_individual_winners(&mut registry, &self.frozen, &selection);
        let selection = registry.content_selection(&mods, &self.base_selection);
        let (result, usage) = authoring::control::run(
            &self.settings.authoring_execution(),
            cancel,
            ExecutionPhase::Acquisition,
            || {
                self.frozen
                    .publish(&selection, &self.limits, self.settings.clone())
            },
        )
        .map_err(|e| ModValidationError::execution("individual", e))?;
        let publication = result.map_err(|e| {
            ModValidationError::new(
                ModValidationErrorKind::ContentFailed,
                format!("mods.{mod_id}"),
                e.to_string(),
            )
        })?;
        Self::finish(
            self.frozen.clone(),
            publication,
            registry,
            mods,
            self.base_selection.clone(),
            &base.publication,
            self.limits.clone(),
            self.settings.clone(),
            usage,
        )
    }

    pub fn individual_from_capture(
        &self,
        mod_id: &str,
        cancel: &CancellationToken,
    ) -> Result<Self, ModValidationError> {
        self.selected_from_capture_inner(vec![mod_id.to_owned()], cancel, true)
    }
    /// Re-resolve selected roots from the exact captured registry/base. This
    /// retains original winner/reference rules rather than merging PASS DTOs.
    pub fn selected_from_capture(
        &self,
        selected: Vec<String>,
        cancel: &CancellationToken,
    ) -> Result<Self, ModValidationError> {
        self.selected_from_capture_inner(selected, cancel, false)
    }
    fn selected_from_capture_inner(
        &self,
        selected: Vec<String>,
        cancel: &CancellationToken,
        individual: bool,
    ) -> Result<Self, ModValidationError> {
        let base = TrustedBase {
            selection: self.base_selection.clone(),
            publication: self.base_publication.clone(),
            settings: self.settings.clone(),
        };
        let mods = self.registry.closure_for(&selected)?;
        let mut registry = self.registry.clone();
        registry.selection.mods = selected;
        let selection = registry.content_selection(&mods, &self.base_selection);
        if individual {
            project_individual_winners(&mut registry, &self.frozen, &selection);
        }
        let selection = registry.content_selection(&mods, &self.base_selection);
        let (result, usage) = authoring::control::run(
            &self.settings.authoring_execution(),
            cancel,
            ExecutionPhase::Acquisition,
            || {
                self.frozen
                    .publish(&selection, &self.limits, self.settings.clone())
            },
        )
        .map_err(|e| ModValidationError::execution("selection", e))?;
        let publication = result.map_err(|e| {
            ModValidationError::new(
                ModValidationErrorKind::ContentFailed,
                "selection",
                e.to_string(),
            )
        })?;
        Self::finish(
            self.frozen.clone(),
            publication,
            registry,
            mods,
            self.base_selection.clone(),
            &base.publication,
            self.limits.clone(),
            self.settings.clone(),
            usage,
        )
    }
}
fn project_individual_winners(
    registry: &mut ModRegistry,
    frozen: &content_pack::FrozenContent,
    selection: &ContentSelection,
) {
    // Actual captured manifest IDs, never directory-name guesses or unrelated
    // pack acquisition. Included ambiguous/nonmaximal writers still fail in
    // the unchanged resolver. Trusted-base choices are merged separately.
    let pack_ids: std::collections::BTreeSet<_> = frozen
        .manifests(selection)
        .into_iter()
        .map(|(_, manifest)| manifest.pack_id)
        .collect();
    registry
        .selection
        .winners
        .retain(|winner| pack_ids.contains(&winner.pack_id));
}
