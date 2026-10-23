use super::{Diagnostic, Location, NormalizedDocument, Origin};
use std::collections::{BTreeMap, BTreeSet};

/// Request/permit/receipt ownership checks shared by the graph lowerers. The
/// lowered operation's own validator must still check its additional contracts.
pub fn resonance_scope_diagnostics(doc: &NormalizedDocument) -> Vec<Diagnostic> {
    let mut out = vec![];
    let definitions: BTreeMap<_, _> = super::structure::catalog()
        .definitions
        .iter()
        .map(|d| (d.kind.as_str(), d))
        .collect();
    let flow: BTreeMap<_, Vec<&Origin>> = doc
        .nodes
        .iter()
        .map(|(id, n)| {
            let inputs = definitions
                .get(n.kind.as_str())
                .map(|d| {
                    d.inputs
                        .iter()
                        .filter(|p| p.value_type == "flow")
                        .flat_map(|p| n.inputs.get(&p.id).into_iter().flatten())
                        .collect()
                })
                .unwrap_or_default();
            (id.as_str(), inputs)
        })
        .collect();
    let source = |id: &str, port: &str| -> Option<&str> {
        let origins = doc.nodes.get(id)?.inputs.get(port)?;
        (origins.len() == 1).then(|| origins[0].source.node.as_str())
    };
    let mut issue = |id: &str, port: &str, code: &str, other: Option<&str>, reason: &str| {
        out.push(Diagnostic {
            code: code.into(),
            phase: "semantic".into(),
            document: Some(doc.document.clone()),
            digest: doc.digest.clone(),
            primary: Location {
                node: Some(id.into()),
                port: Some(port.into()),
                ..Default::default()
            },
            related: other
                .map(|id| {
                    vec![Location {
                        node: Some(id.into()),
                        ..Default::default()
                    }]
                })
                .unwrap_or_default(),
            reason: reason.into(),
            expected: None,
            actual: None,
        });
    };
    let mut admissions: BTreeMap<&str, &str> = BTreeMap::new();
    let mut payments: BTreeMap<&str, &str> = BTreeMap::new();
    for (id, n) in &doc.nodes {
        let require_path = |producer: Option<&str>, output: &str| {
            producer.is_some_and(|p| must_pass(id, p, output, &flow))
        };
        match n.kind.as_str() {
            "skill-resonance-qualify" => {
                let request = source(id, "request");
                let read = source(id, "snapshot");
                let owner = read.and_then(|r| source(r, "request"));
                if request.is_some() && owner != request {
                    issue(
                        id,
                        "snapshot",
                        "graph.owner_scope",
                        owner,
                        "Snapshot and qualification belong to different requests.",
                    );
                }
                if !require_path(request, "out") {
                    issue(
                        id,
                        "request",
                        "graph.result_scope",
                        request,
                        "Qualification must execute from its own request.",
                    );
                }
                if let Some(cost) = source(id, "cost") {
                    if source(cost, "snapshot") != read {
                        issue(
                            id,
                            "cost",
                            "graph.owner_scope",
                            Some(cost),
                            "Cost and qualification use different snapshots.",
                        );
                    }
                }
            }
            "skill-resonance-admit" => {
                let permit = source(id, "permit");
                if !require_path(permit, "out") {
                    issue(
                        id,
                        "permit",
                        "graph.result_scope",
                        permit,
                        "Admission must execute on the permit's successful branch.",
                    );
                }
                if let Some(request) = permit.and_then(|p| source(p, "request")) {
                    if let Some(previous) = admissions.insert(request, id) {
                        issue(
                            id,
                            "permit",
                            "graph.duplicate_admission",
                            Some(previous),
                            "A request cannot be admitted twice.",
                        );
                    }
                }
            }
            "skill-resonance-result" => {
                let producer = source(id, "receipt");
                if !require_path(producer, "out") {
                    issue(
                        id,
                        "receipt",
                        "graph.result_scope",
                        producer,
                        "A receipt is only available after successful payment.",
                    );
                }
            }
            "skill-resonance-stage" | "skill-resonance-end" => {
                let owner = source(id, "use");
                if !require_path(owner, "out") {
                    issue(
                        id,
                        "use",
                        "graph.owner_scope",
                        owner,
                        "Use must belong to the successful admission on this flow.",
                    );
                }
                if let Some(end) = doc.nodes.iter().find_map(|(other, node)| {
                    (other != id
                        && node.kind == "skill-resonance-end"
                        && source(other, "use") == owner
                        && reachable(id, other, &flow))
                    .then_some(other)
                }) {
                    issue(
                        id,
                        "use",
                        "graph.use_after_end",
                        Some(end),
                        "This use was already ended on an incoming execution path.",
                    );
                }
            }
            "skill-resonance-pay" => {
                let step = source(id, "step");
                if !require_path(step, "out") {
                    issue(
                        id,
                        "step",
                        "graph.result_scope",
                        step,
                        "Payment must follow the corresponding stage occurrence.",
                    );
                }
                if let Some(step) = step {
                    if let Some(previous) = payments.insert(step, id) {
                        issue(
                            id,
                            "step",
                            "graph.duplicate_payment",
                            Some(previous),
                            "A stage occurrence cannot have two payment sites.",
                        );
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn must_pass(
    node: &str,
    producer: &str,
    port: &str,
    incoming: &BTreeMap<&str, Vec<&Origin>>,
) -> bool {
    // Every reverse path must hit the exact successful output. Memoization is
    // local to this producer/output; node and edge array order has no meaning.
    let mut memo = BTreeMap::new();
    let mut stack = vec![(node, false)];
    let mut visiting = BTreeSet::new();
    while let Some((id, expanded)) = stack.pop() {
        if memo.contains_key(id) {
            continue;
        }
        let edges = incoming.get(id).map(Vec::as_slice).unwrap_or_default();
        if expanded {
            let result = !edges.is_empty()
                && edges.iter().all(|edge| {
                    if edge.source.node == producer {
                        edge.source.port == port
                    } else {
                        memo.get(edge.source.node.as_str())
                            .copied()
                            .unwrap_or(false)
                    }
                });
            memo.insert(id, result);
            visiting.remove(id);
        } else {
            if !visiting.insert(id) {
                return false;
            }
            stack.push((id, true));
            for edge in edges {
                if edge.source.node != producer && !memo.contains_key(edge.source.node.as_str()) {
                    stack.push((&edge.source.node, false));
                }
            }
        }
    }
    memo.get(node).copied().unwrap_or(false)
}
fn reachable(node: &str, ancestor: &str, incoming: &BTreeMap<&str, Vec<&Origin>>) -> bool {
    let mut seen = BTreeSet::new();
    let mut pending = vec![node];
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        for edge in incoming.get(id).into_iter().flatten() {
            if edge.source.node == ancestor {
                return true;
            }
            pending.push(&edge.source.node);
        }
    }
    false
}
