use super::{Diagnostic, FieldValue, Location, NormalizedDocument};
use std::collections::BTreeMap;

/// Report unfinished authored fields and active required ports. This is one
/// semantic pass; successful scope checking/lowering is still required.
pub fn authoring_gaps(document: &NormalizedDocument) -> Vec<Diagnostic> {
    let definitions: BTreeMap<_, _> = super::structure::catalog()
        .definitions
        .iter()
        .map(|d| (d.kind.as_str(), d))
        .collect();
    let mut diagnostics = vec![];
    for (id, node) in &document.nodes {
        let Some(def) = definitions.get(node.kind.as_str()) else {
            continue;
        };
        let values: BTreeMap<_, _> = node
            .values
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    match v {
                        FieldValue::Number(v) => serde_json::json!(v),
                        FieldValue::Text(v) => serde_json::json!(v),
                        FieldValue::Boolean(v) => serde_json::json!(v),
                    },
                )
            })
            .collect();
        let mut issue = |code: &str, port: Option<&str>, field: Option<&str>, reason: &str| {
            diagnostics.push(Diagnostic {
                code: code.into(),
                phase: "semantic".into(),
                document: Some(document.document.clone()),
                digest: document.digest.clone(),
                primary: Location {
                    node: Some(id.clone()),
                    port: port.map(Into::into),
                    field: field.map(Into::into),
                    edge: None,
                },
                related: vec![],
                reason: reason.into(),
                expected: None,
                actual: None,
            })
        };
        if node.kind == "skill-area-damage" && !values.contains_key("damageType") {
            issue("graph.policy_unresolved", None, Some("damageType"), "This historical area-damage draft needs an explicit damage type.");
        }
        for field in &def.fields {
            let Some(value) = values.get(&field.id) else {
                continue;
            };
            if ["optional-number", "tile-pattern"].contains(&field.value_type.as_str())
                && value.as_str() == Some("")
            {
                issue(
                    "graph.value_unresolved",
                    None,
                    Some(&field.id),
                    "A required authored value is unfinished.",
                );
            }
            if field.options.is_some() && value.as_str() == Some("unresolved") {
                issue(
                    "graph.policy_unresolved",
                    None,
                    Some(&field.id),
                    "Select an explicit authored policy.",
                );
            }
        }
        for port in &def.inputs {
            if port.active(&values) && port.required(&values) && !node.inputs.contains_key(&port.id)
            {
                issue(
                    "graph.input_required",
                    Some(&port.id),
                    None,
                    "An active required input has no producer.",
                );
            }
        }
    }
    diagnostics
}
