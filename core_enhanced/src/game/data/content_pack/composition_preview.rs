//! An ephemeral authoring overlay. All captured catalogs still pass the normal
//! resolver and publication checks; no source files or live generations change.
use super::*;
use crate::authoring::composition::Artifact;
use std::collections::BTreeSet;

pub(crate) struct CompositionPublication {
    pub token: ValidatedGameData,
    pub artifact_path: String,
    pub artifact_digest: String,
}
pub(crate) struct CompositionBase {
    snapshot: snapshot::SourceSnapshot,
    token: ValidatedGameData,
}

pub(crate) fn capture_composition_base(
    root: &Path,
    selection: &ContentSelection,
    limits: &InputLimits,
    settings: Arc<crate::runtime_safety::RuntimeSafetySettings>,
) -> Result<CompositionBase, LiveRonError> {
    let limits = effective_input_limits(limits, &settings);
    let snapshot = snapshot::acquire(root, selection, &limits)?;
    for pack in &snapshot.packs {
        control::charge_serialized(&(&pack.manifest, &pack.catalogs), "composition_base_clone")
            .map_err(control_error)?;
    }
    let token =
        publish_snapshot_with_runtime_safety(snapshot.clone(), selection, &limits, settings)?;
    Ok(CompositionBase { snapshot, token })
}

pub(crate) fn load_composition_preview(
    base: CompositionBase,
    selection: &ContentSelection,
    artifact: &Artifact,
    limits: &InputLimits,
    settings: Arc<crate::runtime_safety::RuntimeSafetySettings>,
) -> Result<CompositionPublication, LiveRonError> {
    let error = |cause: String| {
        LiveRonError::validation(
            "composition_preview",
            "composition_preview",
            None::<String>,
            None::<String>,
            cause,
        )
        .with_code("composition_preview_invalid")
    };
    let limits = effective_input_limits(limits, &settings);
    // The separately bounded base validation owns these exact captured bytes.
    // The candidate never reopens files or silently repairs an invalid base.
    let CompositionBase {
        mut snapshot,
        token: base,
    } = base;
    let artifact_text = artifact.to_ron().map_err(|e| error(e.to_string()))?;
    let artifact_digest = format!("{:x}", Sha256::digest(artifact_text.as_bytes()));
    let pack_id = format!("composition-preview-{artifact_digest}");
    let manifest_source = format!("{pack_id}/pack.ron");
    let relative_artifact = "data/skills/preview.skill-composition.ron";
    let artifact_path = format!("{pack_id}/{relative_artifact}");
    if snapshot
        .packs
        .iter()
        .any(|p| p.manifest.pack_id == pack_id || p.manifest_source == manifest_source)
    {
        return Err(error(
            "preview pack identity collides with selected external content".into(),
        ));
    }
    let definitions =
        CatalogKind::Skills.parse(&pack_id, &artifact_path, &artifact_text, &limits)?;
    let targets: BTreeSet<_> = definitions.iter().map(|d| d.definition.key()).collect();
    let writers = base
        .source_metadata()
        .expect("external publication has provenance")
        .writers();
    let mut operations = Vec::new();
    let mut preview_selection = selection.clone();
    preview_selection
        .winners
        .retain(|writer| !targets.contains(&writer.target));
    for target in &targets {
        let after: BTreeSet<_> = writers
            .iter()
            .filter(|writer| &writer.target == target)
            .map(|writer| writer.pack_id.clone())
            .collect();
        if !after.is_empty() {
            operations.push(DefinitionOperation {
                target: target.clone(),
                operation: OperationKind::Replace,
                after: after.into_iter().collect(),
            });
            preview_selection.winners.push(WriterSelection {
                target: target.clone(),
                pack_id: pack_id.clone(),
            });
        }
    }
    let manifest = PackManifest {
        pack_id: pack_id.clone(),
        pack_version: artifact_digest.clone(),
        namespace: PackNamespace::Global,
        contract: ContentContract::current(),
        required_capabilities: vec!["structured_skill_flow_v1".into()],
        dependencies: snapshot
            .packs
            .iter()
            .map(|p| p.manifest.pack_id.clone())
            .collect(),
        files: vec![CatalogFile {
            kind: CatalogKind::Skills,
            path: relative_artifact.into(),
        }],
        operations,
    };
    manifest.validate(&manifest_source)?;
    let manifest_text = ron::ser::to_string(&manifest).map_err(|e| error(e.to_string()))?;
    let extra_bytes = artifact_text
        .len()
        .checked_add(manifest_text.len())
        .ok_or_else(|| error("preview byte count overflow".into()))?;
    let captured_bytes = snapshot
        .receipt
        .files
        .iter()
        .try_fold(0usize, |sum, file| sum.checked_add(file.byte_count))
        .ok_or_else(|| error("captured byte count overflow".into()))?;
    if snapshot.packs.len() >= limits.max_packs
        || snapshot.receipt.files.len().saturating_add(2) > limits.max_files
        || artifact_text.len() > limits.max_file_bytes
        || manifest_text.len() > limits.max_file_bytes
        || captured_bytes
            .checked_add(extra_bytes)
            .is_none_or(|sum| sum > limits.max_total_bytes)
    {
        return Err(error(
            "preview overlay exceeds the selected content input limits".into(),
        ));
    }
    control::charge(extra_bytes as u64, "composition_overlay_bytes").map_err(control_error)?;
    for (path, text) in [
        (&manifest_source, &manifest_text),
        (&artifact_path, &artifact_text),
    ] {
        snapshot.receipt.files.push(SourceFileReceipt {
            pack_id: pack_id.clone(),
            path: path.clone(),
            byte_count: text.len(),
            sha256: format!("{:x}", Sha256::digest(text.as_bytes())),
        });
    }
    snapshot.receipt.files.sort_by(|a, b| a.path.cmp(&b.path));
    snapshot.packs.push(snapshot::PackSnapshot {
        manifest,
        manifest_text,
        manifest_source,
        catalogs: vec![(CatalogKind::Skills, artifact_path.clone(), artifact_text)],
    });
    preview_selection.packs.push(pack_id.into());
    let token =
        publish_snapshot_with_runtime_safety(snapshot, &preview_selection, &limits, settings)?;
    Ok(CompositionPublication {
        token,
        artifact_path,
        artifact_digest,
    })
}
