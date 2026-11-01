use super::*;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct PackageReceipt {
    pub output_directory: PathBuf,
    pub source_receipt: SourceReceipt,
    pub content_identity: ContentIdentity,
    pub pack_receipt: SourceReceipt,
}

/// Package the explicit source root's `data/bundle.ron` and the official catalogs
/// into a new directory outside that source tree. Catalog bytes are preserved;
/// only `pack.ron` is generated. Existing output is never merged or overwritten.
pub fn package_official(
    source_root: &Path,
    new_pack_directory: &Path,
    limits: &InputLimits,
) -> Result<PackageReceipt, LiveRonError> {
    package_official_with_skill_files(source_root, new_pack_directory, &[], limits)
}

/// Include explicitly selected, root-relative Skills catalogs in the same
/// captured and validated candidate. Source files are copied byte for byte;
/// authored YAML is compiled by the loader before any output is created.
pub fn package_official_with_skill_files(
    source_root: &Path,
    new_pack_directory: &Path,
    skill_files: &[PathBuf],
    limits: &InputLimits,
) -> Result<PackageReceipt, LiveRonError> {
    let manifest = PackManifest {
        pack_id: "official".into(),
        pack_version: "1".into(),
        namespace: PackNamespace::Global,
        contract: ContentContract::current(),
        required_capabilities: PackManifest::SUPPORTED_CAPABILITIES
            .iter()
            .map(|s| (*s).into())
            .collect(),
        dependencies: vec![],
        files: CatalogKind::ALL
            .iter()
            .map(|kind| CatalogFile {
                kind: *kind,
                path: kind.official_path().into(),
            })
            .chain(skill_files.iter().map(|path| CatalogFile {
                kind: CatalogKind::Skills,
                path: path.clone(),
            }))
            .collect(),
        operations: vec![],
    };
    let snapshot = snapshot::acquire_official(source_root, manifest.clone(), limits)?;
    let selection = ContentSelection {
        packs: vec!["official".into()],
        winners: vec![],
    };
    // Reuse exactly the loader's post-acquisition validation before any output.
    let expected = publish_snapshot(snapshot.clone(), &selection, limits)?;
    let mut files: Vec<_> = snapshot
        .packs
        .into_iter()
        .flat_map(|pack| pack.catalogs)
        .map(|(_, path, input)| (path, input))
        .collect();
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let manifest_text = ron::ser::to_string(&manifest).map_err(|cause| {
        LiveRonError::validation(
            "pack.ron",
            "official_packaging",
            None::<String>,
            None::<String>,
            cause.to_string(),
        )
    })?;
    // The manifest is deliberately last: incomplete output is never advertised
    // as a complete inventory. Errors leave only this new output for inspection.
    files.push(("pack.ron".into(), manifest_text));
    let output = snapshot::write_new_pack(source_root, new_pack_directory, &files)?;
    let selection = ContentSelection {
        packs: vec![output
            .file_name()
            .expect("created normal directory name")
            .into()],
        winners: vec![],
    };
    let packed = load(
        output.parent().expect("created directory parent"),
        &selection,
        limits,
    )?;
    let prefix = output.file_name().unwrap().to_str().unwrap();
    for (path, input) in &files {
        let packed_path = format!("{prefix}/{path}");
        let expected_sha256 = format!("{:x}", Sha256::digest(input.as_bytes()));
        if !packed.source_receipt().files.iter().any(|receipt| {
            receipt.path == packed_path
                && receipt.byte_count == input.len()
                && receipt.sha256 == expected_sha256
        }) {
            return Err(LiveRonError::validation(
                packed_path,
                "official_packaging",
                None::<String>,
                None::<String>,
                "packaged output bytes changed from the captured source or generated manifest",
            ));
        }
    }
    if packed.identity() != expected.identity() {
        return Err(LiveRonError::validation(
            output.display().to_string(),
            "official_packaging",
            None::<String>,
            None::<String>,
            "packaged output changed from the validated source snapshot",
        ));
    }
    Ok(PackageReceipt {
        output_directory: output,
        source_receipt: snapshot.receipt,
        content_identity: packed.identity().clone(),
        pack_receipt: packed.source_receipt().clone(),
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::symlink};

    #[path = "authored_tests.rs"]
    mod authored;

    struct SourceFixture(PathBuf);
    impl SourceFixture {
        fn new() -> Self {
            let root =
                std::env::temp_dir().join(format!("capg-packaging-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&root).unwrap();
            let fixture = Self(root);
            let official = Path::new(env!("CARGO_MANIFEST_DIR")).join("../game_resources");
            for path in Self::paths() {
                let target = fixture.source().join(path);
                fs::create_dir_all(target.parent().unwrap()).unwrap();
                fs::copy(official.join(path), target).unwrap();
            }
            fixture
        }
        fn paths() -> Vec<&'static str> {
            std::iter::once("data/bundle.ron")
                .chain(CatalogKind::ALL.iter().map(|k| k.official_path()))
                .collect()
        }
        fn source(&self) -> PathBuf {
            self.0.join("source")
        }
        fn bytes(&self) -> Vec<Vec<u8>> {
            Self::paths()
                .into_iter()
                .map(|path| fs::read(self.source().join(path)).unwrap())
                .collect()
        }
        fn package(&self, output: &Path) -> Result<PackageReceipt, LiveRonError> {
            package_official(&self.source(), output, &InputLimits::default())
        }
    }
    impl Drop for SourceFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn official_packaging_preserves_external_catalog_and_existing_custom_bytes() {
        let fixture = SourceFixture::new();
        let equipment = fixture.source().join("data/equipments/base.ron");
        let input = fs::read_to_string(&equipment).unwrap();
        fs::write(
            &equipment,
            format!("// explicit external source comment\r\n{input}\n"),
        )
        .unwrap();
        let custom = fixture.0.join("user_pack.ron");
        let custom_bytes = b"user-owned arbitrary bytes\0\xff";
        fs::write(&custom, custom_bytes).unwrap();
        let before = fixture.bytes();
        let output = fixture.0.join("official");
        let receipt = fixture.package(&output).unwrap();
        // Twenty-six catalogs plus the source bundle / output pack manifest.
        assert_eq!(receipt.source_receipt.files.len(), 27);
        assert_eq!(receipt.pack_receipt.files.len(), 27);
        for source in [&receipt.source_receipt, &receipt.pack_receipt] {
            assert!(source
                .files
                .iter()
                .any(|file| file.path.ends_with("data/relics/base.ron")));
        }
        assert_eq!(
            fs::read(output.join("data/relics/base.ron")).unwrap(),
            fs::read(fixture.source().join("data/relics/base.ron")).unwrap()
        );
        assert_eq!(fixture.bytes(), before);
        assert_eq!(fs::read(&custom).unwrap(), custom_bytes);
        for kind in CatalogKind::ALL {
            assert_eq!(
                fs::read(output.join(kind.official_path())).unwrap(),
                fs::read(fixture.source().join(kind.official_path())).unwrap()
            );
        }
        assert!(!fixture.source().join("pack.ron").exists());
        assert!(!output.join("data/bundle.ron").exists());
        let manifest_bytes = fs::read(output.join("pack.ron")).unwrap();
        assert!(fixture
            .package(&output)
            .unwrap_err()
            .cause
            .contains("exclusively create"));
        assert_eq!(fs::read(output.join("pack.ron")).unwrap(), manifest_bytes);
        assert_eq!(fixture.bytes(), before);
        assert_eq!(fs::read(&custom).unwrap(), custom_bytes);
    }

    #[test]
    fn official_packaging_rejects_existing_alias_and_source_tree_outputs() {
        let fixture = SourceFixture::new();
        let empty = fixture.0.join("empty");
        fs::create_dir(&empty).unwrap();
        let occupied = fixture.0.join("occupied");
        fs::create_dir(&occupied).unwrap();
        fs::write(occupied.join("sentinel"), b"preserve").unwrap();
        let alias = fixture.0.join("alias");
        symlink(&occupied, &alias).unwrap();
        let before = fixture.bytes();
        for output in [
            &empty,
            &occupied,
            &alias,
            &fixture.source(),
            &fixture.source().join("nested"),
        ] {
            assert!(fixture.package(output).is_err(), "{}", output.display());
        }
        assert_eq!(fixture.bytes(), before);
        assert_eq!(fs::read_dir(&empty).unwrap().count(), 0);
        assert_eq!(fs::read(occupied.join("sentinel")).unwrap(), b"preserve");
        assert!(!fixture.source().join("nested").exists());
        assert!(fs::symlink_metadata(alias)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn official_packaging_rejects_missing_stale_or_invalid_source_before_output() {
        let fixture = SourceFixture::new();
        let output = fixture.0.join("official");
        let bundle = fixture.source().join("data/bundle.ron");
        let original_bundle = fs::read(&bundle).unwrap();
        fs::write(&bundle, "(schema_version: 1)").unwrap();
        let error = fixture.package(&output).unwrap_err();
        assert_eq!(error.source_file, "data/bundle.ron");
        assert!(error.cause.contains("unsupported schema version"));
        assert!(!output.exists());
        fs::write(&bundle, original_bundle).unwrap();
        let catalog = fixture.source().join("data/skills/base.ron");
        let original_catalog = fs::read(&catalog).unwrap();
        fs::remove_file(&catalog).unwrap();
        assert!(fixture.package(&output).is_err());
        assert!(!output.exists());
        fs::write(&catalog, "(skills: [], unsupported: true)").unwrap();
        assert!(fixture.package(&output).is_err());
        assert!(!output.exists());
        fs::write(&catalog, original_catalog).unwrap();
        let missing = fixture.0.join("missing_source");
        assert!(package_official(&missing, &output, &InputLimits::default()).is_err());
        assert!(!output.exists());
    }
}
