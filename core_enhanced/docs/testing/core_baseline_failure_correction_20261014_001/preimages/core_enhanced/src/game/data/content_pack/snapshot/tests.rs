use super::super::{CatalogFile, CatalogKind, ContentContract, PackNamespace};
use super::*;

#[test]
fn parser_budget_counts_syntax_without_counting_quoted_or_commented_delimiters() {
    for input in [
        r#"["[[[[[[", '\'', "escaped \" ] ] "]"#,
        "[/* [[[[[[ */ 1, // ]]]]]\n2]",
        r####"[r###"[[["#]]]"###]"####,
    ] {
        check_nesting("fixture.ron", input, 3).unwrap();
        ron::de::from_str::<serde::de::IgnoredAny>(input).unwrap();
    }
    check_nesting("fixture.ron", "[[[0]]]", 3).unwrap();
    assert!(check_nesting("fixture.ron", "[[[[0]]]]", 3)
        .unwrap_err()
        .cause
        .contains("nesting limit"));
    assert!(check_nesting("fixture.ron", "/* /* /* /* x */ */ */ */", 3).is_err());
    assert!(check_nesting("fixture.ron", r#####"r####"x"####"#####, 3).is_err());
    for input in ["[)", "[", "/* x", "\"x", "'x", "r#\"x"] {
        assert!(check_nesting("fixture.ron", input, 3).is_err(), "{input}");
    }
}

#[test]
fn relative_paths_reject_platform_escapes_and_non_normal_components() {
    for path in [
        "", ".", "..", "../x", "a/../x", "a/./x", "/x", "a//x", "a/", "a\\x", "C:/x", "x\ny",
    ] {
        assert!(relative_path(Path::new(path)).is_err(), "{path:?}");
    }
    assert_eq!(
        relative_path(Path::new("pack/data/한글 file.ron")).unwrap(),
        "pack/data/한글 file.ron"
    );
    assert!(validate_limits(&InputLimits {
        max_nesting: 129,
        ..InputLimits::default()
    })
    .is_err());
    assert!(validate_limits(&InputLimits {
        max_nesting: 0,
        ..InputLimits::default()
    })
    .is_err());
    let overflow = validate_limits(&InputLimits {
        max_file_bytes: usize::MAX,
        ..InputLimits::default()
    })
    .unwrap_err();
    assert_eq!(overflow.source_file, "host_limits");
    assert!(overflow.cause.contains("invalid trusted input limits"));
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::{
        fs,
        os::unix::{fs::symlink, net::UnixListener},
    };

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("capg-snapshot-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
        fn write(&self, path: &str, bytes: impl AsRef<[u8]>) {
            let path = self.0.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }
        fn manifest(&self, paths: &[&str]) -> PackManifest {
            let manifest = PackManifest {
                pack_id: "base".into(),
                pack_version: "1".into(),
                namespace: PackNamespace::Global,
                contract: ContentContract::current(),
                required_capabilities: vec!["literal_effect_authoring_v1".into()],
                dependencies: vec![],
                files: paths
                    .iter()
                    .map(|path| CatalogFile {
                        kind: CatalogKind::FacilityTemplates,
                        path: (*path).into(),
                    })
                    .collect(),
                operations: vec![],
            };
            self.write("base/pack.ron", ron::ser::to_string(&manifest).unwrap());
            manifest
        }
        fn selection(&self) -> ContentSelection {
            ContentSelection {
                packs: vec!["base".into()],
                winners: vec![],
            }
        }
        fn load(&self) -> Result<SourceSnapshot, LiveRonError> {
            acquire(&self.0, &self.selection(), &InputLimits::default())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const DATA: &str = "(tag_catalog: [], templates: [], pools: [])";

    #[test]
    fn owned_snapshot_and_byte_receipt_survive_later_source_edits() {
        let f = Fixture::new();
        f.manifest(&["data.ron"]);
        f.write("base/data.ron", DATA);
        let snapshot = f.load().unwrap();
        assert_eq!(snapshot.packs.len(), 1);
        assert_eq!(snapshot.packs[0].manifest.pack_id, "base");
        assert_eq!(snapshot.packs[0].manifest_source, "base/pack.ron");
        assert_eq!(snapshot.receipt.files.len(), 2);
        assert!(snapshot.receipt.files.iter().all(|r| r.pack_id == "base"));
        let receipt = snapshot
            .receipt
            .files
            .iter()
            .find(|r| r.path.ends_with("data.ron"))
            .unwrap();
        assert_eq!(
            receipt.sha256,
            format!("{:x}", Sha256::digest(DATA.as_bytes()))
        );
        f.write("base/data.ron", "(changed_after_snapshot: true)");
        assert_eq!(snapshot.packs[0].catalogs[0].2, DATA);
        assert_eq!(receipt.byte_count, DATA.len());
    }

    #[test]
    fn exact_file_and_aggregate_byte_limits_include_manifest_bytes() {
        let f = Fixture::new();
        let manifest = f.manifest(&["data.ron"]);
        f.write("base/data.ron", DATA);
        let manifest_len = ron::ser::to_string(&manifest).unwrap().len();
        let exact = InputLimits {
            max_packs: 1,
            max_files: 2,
            max_file_bytes: manifest_len.max(DATA.len()),
            max_total_bytes: manifest_len + DATA.len(),
            ..InputLimits::default()
        };
        acquire(&f.0, &f.selection(), &exact).unwrap();
        let mut too_many = f.selection();
        too_many.packs.push("second".into());
        let over = acquire(&f.0, &too_many, &exact).err().unwrap();
        assert_eq!(over.source_file, "selection");
        assert!(over.cause.contains("pack count limit exceeded"));
        for limits in [
            InputLimits {
                max_files: 1,
                ..exact.clone()
            },
            InputLimits {
                max_file_bytes: exact.max_file_bytes - 1,
                ..exact.clone()
            },
            InputLimits {
                max_total_bytes: exact.max_total_bytes - 1,
                ..exact.clone()
            },
        ] {
            assert!(acquire(&f.0, &f.selection(), &limits).is_err());
        }
    }

    #[test]
    fn duplicate_inventory_hardlinks_and_case_colliding_directories_are_rejected() {
        let f = Fixture::new();
        f.manifest(&["data.ron", "data.ron"]);
        f.write("base/data.ron", DATA);
        assert!(f
            .load()
            .err()
            .unwrap()
            .cause
            .contains("duplicate inventory"));
        f.manifest(&["data.ron", "alias.ron"]);
        fs::hard_link(f.0.join("base/data.ron"), f.0.join("base/alias.ron")).unwrap();
        assert!(f
            .load()
            .err()
            .unwrap()
            .cause
            .contains("duplicate file identity"));
        f.manifest(&["Dir/one.ron", "dir/two.ron"]);
        f.write("base/Dir/one.ron", DATA);
        f.write("base/dir/two.ron", DATA);
        assert!(f.load().err().unwrap().cause.contains("case-colliding"));
    }

    #[test]
    fn symlinks_in_file_directory_and_root_are_never_followed() {
        let f = Fixture::new();
        let outside = Fixture::new();
        outside.write("secret.ron", DATA);
        f.manifest(&["link.ron"]);
        symlink(outside.0.join("secret.ron"), f.0.join("base/link.ron")).unwrap();
        assert!(f.load().err().unwrap().cause.contains("no-follow"));
        f.manifest(&["dir/secret.ron"]);
        symlink(&outside.0, f.0.join("base/dir")).unwrap();
        assert!(f.load().err().unwrap().cause.contains("no-follow"));
        symlink(&f.0, outside.0.join("root_link")).unwrap();
        assert!(acquire(
            &outside.0.join("root_link"),
            &f.selection(),
            &InputLimits::default()
        )
        .is_err());
    }

    #[test]
    fn invalid_utf8_directories_and_socket_inputs_return_errors() {
        let f = Fixture::new();
        f.manifest(&["data.ron"]);
        f.write("base/data.ron", [0xff]);
        assert!(f.load().err().unwrap().cause.contains("UTF-8"));
        f.manifest(&["directory"]);
        fs::create_dir(f.0.join("base/directory")).unwrap();
        assert!(f.load().err().unwrap().cause.contains("regular file"));
        f.manifest(&["socket"]);
        let _socket = UnixListener::bind(f.0.join("base/socket")).unwrap();
        assert!(f.load().is_err());
    }

    #[test]
    fn all_input_files_are_rechecked_after_acquisition() {
        let f = Fixture::new();
        let mut manifest = f.manifest(&["data.ron"]);
        manifest.pack_id = "decoded_owner".into();
        f.write("base/pack.ron", ron::ser::to_string(&manifest).unwrap());
        let check = |error: LiveRonError| {
            assert!(error.cause.contains("changed during snapshot"));
            assert_eq!(error.phase, LiveRonPhase::Acquisition);
            assert_eq!(error.code, "content_acquisition");
            assert_eq!(error.pack_id.as_deref(), Some("decoded_owner"));
            assert_eq!(error.source_file, "base/data.ron");
            assert_eq!(error.catalog_kind, Some(CatalogKind::FacilityTemplates));
        };
        f.write("base/data.ron", DATA);
        let result = acquire_with_hook(&f.0, &f.selection(), &InputLimits::default(), || {
            f.write(
                "base/data.ron",
                "(tag_catalog: [], templates: [], pools: [ ])",
            )
        });
        check(result.err().unwrap());
        f.write("base/data.ron", DATA);
        let result = acquire_with_hook(&f.0, &f.selection(), &InputLimits::default(), || {
            let original = fs::read(f.0.join("base/data.ron")).unwrap();
            f.write("base/new.ron", original);
            fs::rename(f.0.join("base/new.ron"), f.0.join("base/data.ron")).unwrap();
        });
        check(result.err().unwrap());
    }

    #[test]
    fn replacing_pack_directory_with_symlink_between_passes_fails() {
        let f = Fixture::new();
        let outside = Fixture::new();
        f.manifest(&["data.ron"]);
        f.write("base/data.ron", DATA);
        outside.write("data.ron", DATA);
        outside.write("pack.ron", fs::read(f.0.join("base/pack.ron")).unwrap());
        let result = acquire_with_hook(&f.0, &f.selection(), &InputLimits::default(), || {
            fs::rename(f.0.join("base"), f.0.join("old_base")).unwrap();
            symlink(&outside.0, f.0.join("base")).unwrap();
        });
        let error = result.err().unwrap();
        assert!(error.cause.contains("no-follow"));
        assert_eq!(error.phase, LiveRonPhase::Acquisition);
        assert_eq!(error.pack_id.as_deref(), Some("base"));
        assert_eq!(error.source_file, "base/pack.ron");
        assert_eq!(error.catalog_kind, None);
    }

    #[test]
    fn obsolete_contract_unsupported_capability_and_unknown_fields_fail_in_real_manifest() {
        let f = Fixture::new();
        let current = f.manifest(&[]);
        let mut old_schema = current.clone();
        old_schema.contract.catalog_schema -= 1;
        let mut old_pack = current.clone();
        old_pack.contract.pack_format -= 1;
        let mut old_fingerprint = current.clone();
        old_fingerprint.contract.fingerprint -= 1;
        for invalid in [old_schema, old_pack, old_fingerprint] {
            f.write("base/pack.ron", ron::ser::to_string(&invalid).unwrap());
            assert!(f
                .load()
                .err()
                .unwrap()
                .cause
                .contains("current complete content contract"));
        }
        let mut invalid = current.clone();
        invalid.contract.authoring += 1;
        f.write("base/pack.ron", ron::ser::to_string(&invalid).unwrap());
        assert!(f.load().is_err());
        invalid = current.clone();
        invalid.contract.execution += 1;
        f.write("base/pack.ron", ron::ser::to_string(&invalid).unwrap());
        assert!(f.load().is_err());
        invalid = current.clone();
        invalid
            .required_capabilities
            .push("future_unimplemented_vm".into());
        f.write("base/pack.ron", ron::ser::to_string(&invalid).unwrap());
        assert!(f
            .load()
            .err()
            .unwrap()
            .cause
            .contains("unsupported authoring/execution capability"));
        let mut text = ron::ser::to_string(&current).unwrap();
        let end = text.rfind(')').unwrap();
        text.insert_str(end, ", unknown_permission: true");
        f.write("base/pack.ron", text);
        assert!(f.load().is_err());
    }

    #[test]
    fn nested_external_catalog_and_empty_or_missing_selection_never_fall_back() {
        let f = Fixture::new();
        f.manifest(&["data.ron"]);
        f.write(
            "base/data.ron",
            format!("{}0{}", "[".repeat(129), "]".repeat(129)),
        );
        assert!(f.load().err().unwrap().cause.contains("nesting limit"));
        assert!(acquire(
            &f.0,
            &ContentSelection {
                packs: vec![],
                winners: vec![]
            },
            &InputLimits::default()
        )
        .is_err());
        assert!(acquire(
            &f.0,
            &ContentSelection {
                packs: vec!["missing".into()],
                winners: vec![]
            },
            &InputLimits::default()
        )
        .is_err());
        assert!(acquire(
            &f.0,
            &ContentSelection {
                packs: vec!["../escape".into()],
                winners: vec![]
            },
            &InputLimits::default()
        )
        .is_err());
    }
}
