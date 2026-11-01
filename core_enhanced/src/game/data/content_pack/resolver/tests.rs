use super::*;
use crate::game::{
    data::content_pack::{raw::RawDefinition, DefinitionKind},
    map::{FacilityTemplatePool, MapTemplateId},
};

fn key(id: &str) -> DefinitionKey {
    DefinitionKey {
        kind: DefinitionKind::FacilityPool,
        id: id.into(),
    }
}

fn pack(
    id: &str,
    definitions: &[(&str, &str)],
    dependencies: &[&str],
    operations: Vec<super::super::DefinitionOperation>,
) -> ParsedPack {
    ParsedPack {
        manifest: PackManifest {
            pack_id: id.into(),
            pack_version: "1".into(),
            namespace: PackNamespace::Global,
            contract: ContentContract::current(),
            required_capabilities: vec![],
            dependencies: dependencies.iter().map(|d| (*d).into()).collect(),
            files: vec![],
            operations,
        },
        manifest_source: format!("{id}/pack.ron"),
        definitions: definitions
            .iter()
            .enumerate()
            .map(|(index, (name, marker))| LocatedDefinition {
                definition: RawDefinition::FacilityPool(FacilityTemplatePool {
                    id: (*name).into(),
                    template_ids: vec![MapTemplateId::new(*marker)],
                }),
                source: DefinitionSource {
                    pack_id: id.into(),
                    source_file: format!("{id}/pools.ron"),
                    field_path: format!("pools[{index}]"),
                },
                source_order: index,
            })
            .collect(),
    }
}

fn op(id: &str, operation: OperationKind, after: &[&str]) -> super::super::DefinitionOperation {
    super::super::DefinitionOperation {
        target: key(id),
        operation,
        after: after.iter().map(|s| (*s).into()).collect(),
    }
}

fn replacement(id: &str, marker: &str, after: &[&str]) -> ParsedPack {
    pack(
        id,
        &[("pool", marker)],
        after,
        vec![op("pool", OperationKind::Replace, after)],
    )
}

fn failure(packs: Vec<ParsedPack>, winners: &[WriterSelection]) -> LiveRonError {
    resolve(packs, winners, &InputLimits::default())
        .err()
        .expect("must reject unresolved structural input")
}

fn marker(content: &ResolvedContent) -> &str {
    match &content.definitions[0].definition {
        RawDefinition::FacilityPool(pool) => pool.template_ids[0].as_str(),
        _ => unreachable!(),
    }
}

#[test]
fn sibling_remove_and_replace_require_explicit_head_and_only_selected_branch_survives() {
    let inputs = || {
        vec![
            pack("base", &[("pool", "original")], &[], vec![]),
            pack(
                "remove",
                &[],
                &["base"],
                vec![op("pool", OperationKind::Remove, &["base"])],
            ),
            replacement("replace", "changed", &["base"]),
        ]
    };
    assert!(failure(inputs(), &[]).cause.contains("ambiguous"));
    let choose = |id: &str| {
        vec![WriterSelection {
            target: key("pool"),
            pack_id: id.into(),
        }]
    };
    let changed = resolve(inputs(), &choose("replace"), &InputLimits::default()).unwrap();
    assert_eq!(marker(&changed), "changed");
    let removed = resolve(inputs(), &choose("remove"), &InputLimits::default()).unwrap();
    assert!(removed.definitions.is_empty());
    assert_ne!(
        changed.semantic_bytes().unwrap(),
        removed.semantic_bytes().unwrap()
    );
    assert!(failure(inputs(), &choose("base"))
        .cause
        .contains("non-maximal"));
}

#[test]
fn tombstone_succession_is_rejected_while_full_definition_diamond_merge_is_explicit() {
    let removed = vec![
        pack("base", &[("pool", "a")], &[], vec![]),
        pack(
            "remove",
            &[],
            &["base"],
            vec![op("pool", OperationKind::Remove, &["base"])],
        ),
        replacement("replace", "b", &["remove"]),
    ];
    assert!(failure(removed, &[]).cause.contains("removed target"));
    let diamond = || {
        vec![
            pack("base", &[("pool", "a")], &[], vec![]),
            replacement("left", "b", &["base"]),
            replacement("right", "c", &["base"]),
            replacement("merge", "d", &["left", "right"]),
        ]
    };
    assert_eq!(
        marker(&resolve(diamond(), &[], &InputLimits::default()).unwrap()),
        "d"
    );
    let mut competing = diamond();
    competing.push(replacement("sibling", "e", &["base"]));
    assert!(failure(competing, &[]).cause.contains("ambiguous"));
}

#[test]
fn dependency_and_writer_order_never_supply_an_implicit_override() {
    let inputs = || {
        vec![
            pack("base", &[("pool", "a")], &[], vec![]),
            pack("custom", &[("pool", "b")], &["base"], vec![]),
        ]
    };
    assert!(failure(inputs(), &[]).cause.contains("duplicate Add"));
    let explicit = [WriterSelection {
        target: key("pool"),
        pack_id: "custom".into(),
    }];
    assert!(failure(inputs(), &explicit).cause.contains("duplicate Add"));
    let input = || {
        vec![
            pack("base", &[("pool", "a")], &[], vec![]),
            replacement("custom", "b", &["base"]),
        ]
    };
    let a = resolve(input(), &[], &InputLimits::default()).unwrap();
    let mut reverse = input();
    reverse.reverse();
    let b = resolve(reverse, &[], &InputLimits::default()).unwrap();
    assert_eq!(a.semantic_bytes().unwrap(), b.semantic_bytes().unwrap());
    assert_eq!(marker(&b), "b");
}

#[test]
fn structural_errors_in_losers_cannot_be_hidden_by_host_winner() {
    let inputs = || {
        vec![
            pack("base", &[("pool", "a")], &[], vec![]),
            replacement("good", "b", &["base"]),
        ]
    };
    let winner = [WriterSelection {
        target: key("pool"),
        pack_id: "good".into(),
    }];
    let mut missing_dependency = inputs();
    missing_dependency.push(pack("bad", &[], &["absent"], vec![]));
    assert!(failure(missing_dependency, &winner)
        .cause
        .contains("dependency"));
    let mut missing_writer = inputs();
    missing_writer.push(pack("empty", &[], &["base"], vec![]));
    missing_writer.push(replacement("bad", "c", &["empty"]));
    assert!(failure(missing_writer, &winner)
        .cause
        .contains("no writer for this target"));
    let mut duplicate = inputs();
    duplicate.push(pack("bad", &[("extra", "c"), ("extra", "d")], &[], vec![]));
    assert!(failure(duplicate, &winner)
        .cause
        .contains("duplicate payload"));
}

#[test]
fn missing_and_duplicate_dependencies_cycles_and_edge_budgets_are_errors() {
    assert!(failure(
        vec![
            pack("a", &[], &["b"], vec![]),
            pack("b", &[], &["a"], vec![])
        ],
        &[]
    )
    .cause
    .contains("cycle"));
    assert!(failure(
        vec![
            pack("a", &[], &[], vec![]),
            pack("b", &[], &["a", "a"], vec![])
        ],
        &[]
    )
    .cause
    .contains("duplicate dependency"));
    let input = || {
        vec![
            pack("base", &[("pool", "a")], &[], vec![]),
            replacement("custom", "b", &["base"]),
        ]
    };
    let mut limits = InputLimits {
        max_dependency_edges: 2,
        ..InputLimits::default()
    };
    assert!(resolve(input(), &[], &limits).is_ok());
    limits.max_dependency_edges = 1;
    assert!(resolve(input(), &[], &limits)
        .err()
        .unwrap()
        .cause
        .contains("edge limit"));
    assert!(failure(vec![], &[]).cause.contains("empty"));
    assert!(failure(
        vec![pack("a", &[], &[], vec![]), pack("a", &[], &[], vec![])],
        &[]
    )
    .cause
    .contains("duplicate pack"));
}

#[test]
fn full_payload_operations_and_host_winner_are_bijective() {
    let base = || pack("base", &[("pool", "a")], &[], vec![]);
    let no_payload = pack(
        "custom",
        &[],
        &["base"],
        vec![op("pool", OperationKind::Replace, &["base"])],
    );
    assert!(failure(vec![base(), no_payload], &[])
        .cause
        .contains("requires exactly one"));
    let remove_payload = pack(
        "custom",
        &[("pool", "b")],
        &["base"],
        vec![op("pool", OperationKind::Remove, &["base"])],
    );
    assert!(failure(vec![base(), remove_payload], &[])
        .cause
        .contains("Remove forbids"));
    let duplicate_op = pack(
        "custom",
        &[("pool", "b")],
        &["base"],
        vec![
            op("pool", OperationKind::Replace, &["base"]),
            op("pool", OperationKind::Remove, &["base"]),
        ],
    );
    assert!(failure(vec![base(), duplicate_op], &[])
        .cause
        .contains("duplicate operation"));
    let absent_target = pack(
        "custom",
        &[],
        &["base"],
        vec![op("absent", OperationKind::Remove, &["base"])],
    );
    assert!(failure(vec![base(), absent_target], &[])
        .cause
        .contains("missing targets"));
    let winner = WriterSelection {
        target: key("pool"),
        pack_id: "base".into(),
    };
    assert!(failure(vec![base()], &[winner.clone(), winner])
        .cause
        .contains("duplicate winner"));
    assert!(failure(
        vec![base()],
        &[WriterSelection {
            target: key("absent"),
            pack_id: "base".into()
        }]
    )
    .cause
    .contains("unknown target"));
}

#[test]
fn numeric_authored_order_and_replacement_origin_have_distinct_coordinates() {
    let names: Vec<_> = (0..12).map(|i| format!("pool_{i}")).collect();
    let values: Vec<_> = names.iter().map(|n| (n.as_str(), "original")).collect();
    let input = || {
        vec![
            pack("base", &values, &[], vec![]),
            pack(
                "custom",
                &[("pool_10", "changed")],
                &["base"],
                vec![op("pool_10", OperationKind::Replace, &["base"])],
            ),
        ]
    };
    let result = resolve(input(), &[], &InputLimits::default()).unwrap();
    assert_eq!(
        result
            .definitions
            .iter()
            .map(|d| d.definition.key().id)
            .collect::<Vec<_>>(),
        names
    );
    assert_eq!(result.definitions[10].source.field_path, "pools[0]");
    assert_eq!(
        result.definitions[10].source.source_file,
        "custom/pools.ron"
    );
    let mut changed = input();
    changed[0].definitions.swap(2, 10);
    for (index, d) in changed[0].definitions.iter_mut().enumerate() {
        d.source_order = index;
    }
    assert_ne!(
        result.semantic_bytes().unwrap(),
        resolve(changed, &[], &InputLimits::default())
            .unwrap()
            .semantic_bytes()
            .unwrap()
    );
}

#[test]
fn independent_namespaces_are_explicit_and_recipe_targets_are_not_prefixed() {
    let mut a = pack("a", &[("a::pool", "one")], &[], vec![]);
    a.manifest.namespace = PackNamespace::Named("a".into());
    let mut b = pack("b", &[("b::pool", "two")], &[], vec![]);
    b.manifest.namespace = PackNamespace::Named("b".into());
    assert_eq!(
        resolve(vec![a, b], &[], &InputLimits::default())
            .unwrap()
            .definitions
            .len(),
        2
    );
    let mut implicit = pack("a", &[("pool", "one")], &[], vec![]);
    implicit.manifest.namespace = PackNamespace::Named("a".into());
    assert!(failure(vec![implicit], &[])
        .cause
        .contains("declared namespace"));
    let mut extension = pack("custom", &[], &[], vec![]);
    extension.manifest.namespace = PackNamespace::Named("custom".into());
    extension.definitions.push(LocatedDefinition {
        definition: RawDefinition::DismantleRecipe(
            crate::game::data::equipment_data::EquipmentDismantleRecipeMetadata {
                equipment_id: "official_equipment".into(),
                yields: vec![],
            },
        ),
        source: DefinitionSource {
            pack_id: "custom".into(),
            source_file: "custom/recipes.ron".into(),
            field_path: "dismantle_recipes[0]".into(),
        },
        source_order: 0,
    });
    let result = resolve(vec![extension], &[], &InputLimits::default()).unwrap();
    assert_eq!(
        result.definitions[0].definition.key().id,
        "official_equipment"
    );
}
