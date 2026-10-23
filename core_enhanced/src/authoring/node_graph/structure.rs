use super::{schema::*, SEMANTIC_VERSION};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

const MAX_SAFE: f64 = 9_007_199_254_740_991.0;
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(super) fn catalog() -> &'static Catalog {
    static VALUE: OnceLock<Catalog> = OnceLock::new();
    VALUE.get_or_init(|| {
        let value: Catalog =
            serde_json::from_str(include_str!("catalog.json")).expect("generated node contract");
        assert_eq!(value.schema_version, 1);
        assert_eq!(value.semantic_version, SEMANTIC_VERSION);
        value
    })
}
fn at(node: &str, field: Option<&str>) -> Location {
    Location {
        node: Some(node.into()),
        field: field.map(Into::into),
        ..Default::default()
    }
}
struct Check {
    document: Option<String>,
    digest: String,
    errors: Vec<Diagnostic>,
}
impl Check {
    fn issue(&mut self, code: &str, primary: Location, reason: &str) {
        self.errors.push(Diagnostic {
            code: code.into(),
            phase: "structure".into(),
            document: self.document.clone(),
            digest: self.digest.clone(),
            primary,
            related: vec![],
            reason: reason.into(),
            expected: None,
            actual: None,
        });
    }
    fn unique(&mut self, ids: &mut BTreeSet<String>, id: &str, location: Location) {
        if id.is_empty() || id.len() > 128 || !ids.insert(id.into()) {
            self.issue(
                "graph.id",
                location,
                "IDs must be nonempty, unique and at most 128 UTF-8 bytes.",
            );
        }
    }
    fn text(&mut self, text: &str, location: Location) {
        if text.len() > 16_384 {
            self.issue("graph.limit", location, "Text exceeds 16384 UTF-8 bytes.");
        }
    }
    fn finish<T>(&mut self, value: T) -> Result<T, Vec<Diagnostic>> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            self.errors.sort_by(|a, b| {
                (
                    &a.phase,
                    &a.primary.node,
                    &a.primary.port,
                    &a.primary.edge,
                    &a.primary.field,
                    &a.code,
                )
                    .cmp(&(
                        &b.phase,
                        &b.primary.node,
                        &b.primary.port,
                        &b.primary.edge,
                        &b.primary.field,
                        &b.code,
                    ))
            });
            Err(std::mem::take(&mut self.errors))
        }
    }
}

/// Exact canonical decimal seconds, with no rounded binary-float multiplication.
pub(super) fn milliseconds(value: &Value) -> Option<u64> {
    let n = value.as_f64()?;
    if n < 0.0 || !n.is_finite() {
        return None;
    }
    if n == 0.0 {
        return Some(0);
    }
    let text = n.to_string();
    let (coefficient, exponent) = text.split_once(['e', 'E']).unwrap_or((&text, "0"));
    let (whole, fraction) = coefficient.split_once('.').unwrap_or((coefficient, ""));
    let digits = format!("{whole}{fraction}").parse::<u128>().ok()?;
    let shift = exponent
        .parse::<i32>()
        .ok()?
        .checked_add(3)?
        .checked_sub(fraction.len() as i32)?;
    if !(0..=38).contains(&shift) {
        return None;
    }
    let result = digits.checked_mul(10u128.checked_pow(shift as u32)?)?;
    (result <= MAX_SAFE as u128).then_some(result as u64)
}
fn valid_pattern(value: &str) -> bool {
    if value.is_empty() {
        return true;
    }
    let rows: Vec<_> = value.split('\n').collect();
    let width = rows[0].len();
    width > 0
        && value.bytes().filter(|b| *b == b'@').count() == 1
        && rows
            .iter()
            .all(|row| row.len() == width && row.bytes().all(|b| matches!(b, b'X' | b'.' | b'@')))
}
fn field(check: &mut Check, node: &Node, field: &Field) {
    let Some(value) = node.values.get(&field.id) else {
        return;
    };
    let location = at(&node.id, Some(&field.id));
    let valid_type = match field.value_type.as_str() {
        "optional-number" => value.as_str() == Some("") || value.is_number(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        _ => value.is_string(),
    };
    if !valid_type {
        check.issue(
            "graph.field_type",
            location,
            "Value does not match the authored field type.",
        );
        return;
    }
    if let Some(text) = value.as_str() {
        check.text(text, location.clone());
        if field.value_type == "tile-pattern" && !valid_pattern(text) {
            check.issue(
                "graph.tile_pattern",
                location.clone(),
                "Expected equal-width X/. rows and exactly one @ anchor.",
            );
        }
        if field
            .options
            .as_ref()
            .is_some_and(|options| !options.iter().any(|option| option == text))
        {
            check.issue(
                "graph.field_choice",
                location.clone(),
                "Unknown authored choice.",
            );
        }
    }
    if let Some(number) = value.as_f64() {
        if !number.is_finite()
            || field.min.is_some_and(|min| number < min)
            || field.max.is_some_and(|max| number > max)
        {
            check.issue(
                "graph.field_range",
                location.clone(),
                "Value is outside its field limits.",
            );
        }
        if field.integer && (number.fract() != 0.0 || number.abs() > MAX_SAFE) {
            check.issue(
                "graph.integer_required",
                location.clone(),
                "Expected an exact safe integer.",
            );
        }
        if field.unit.as_deref() == Some("seconds") && milliseconds(value).is_none() {
            check.issue(
                "graph.time_precision",
                location,
                "Expected nonnegative whole milliseconds within the safe integer range.",
            );
        }
    }
}

/// Read the editor's exact v1 snapshot and remove presentation/reroute machinery.
/// Missing gameplay values remain visible as empty values. This operation does
/// not assert semantic validity, backend support, external references or execution.
pub fn read_and_normalize(json: &str) -> Result<NormalizedDocument, Vec<Diagnostic>> {
    let mut check = Check {
        document: None,
        digest: hash(json.as_bytes()),
        errors: vec![],
    };
    if json.len() > 5_000_000 {
        check.issue(
            "graph.limit",
            Location::default(),
            "Graph input exceeds 5MB.",
        );
        return check.finish(NormalizedDocument::empty());
    }
    let document: Document = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(error) => {
            check.issue(
                "graph.schema",
                Location {
                    field: Some(format!("line {} column {}", error.line(), error.column())),
                    ..Default::default()
                },
                &error.to_string(),
            );
            check.errors.last_mut().unwrap().phase = "parse".into();
            return check.finish(NormalizedDocument::empty());
        }
    };
    check.document = Some(document.id.clone());
    if document.format != "node-graph" || document.version != 1 {
        check.issue(
            "graph.version",
            Location::default(),
            "Only node-graph version 1 is supported.",
        );
    }
    if document.nodes.len() > 2000 || document.edges.len() > 8000 || document.frames.len() > 200 {
        check.issue(
            "graph.limit",
            Location::default(),
            "Maximum: 2000 nodes, 8000 edges and 200 frames.",
        );
        return check.finish(NormalizedDocument::empty());
    }
    let definitions: BTreeMap<_, _> = catalog()
        .definitions
        .iter()
        .map(|d| (d.kind.as_str(), d))
        .collect();
    let mut ids = BTreeSet::new();
    check.unique(&mut ids, &document.id, Location::default());
    check.text(
        &document.name,
        Location {
            field: Some("name".into()),
            ..Default::default()
        },
    );
    let frames: BTreeSet<_> = document.frames.iter().map(|f| f.id.as_str()).collect();
    for frame in &document.frames {
        let loc = Location {
            field: Some(format!("frames.{}", frame.id)),
            ..Default::default()
        };
        check.unique(&mut ids, &frame.id, loc.clone());
        check.text(&frame.title, loc);
        if !document
            .nodes
            .iter()
            .any(|n| n.frame.as_ref() == Some(&frame.id))
        {
            check.issue(
                "graph.empty_frame",
                Location::default(),
                "Each frame must contain a node.",
            );
        }
    }
    let nodes: BTreeMap<_, _> = document.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    for node in &document.nodes {
        check.unique(&mut ids, &node.id, at(&node.id, None));
        check.text(&node.title, at(&node.id, Some("title")));
        check.text(&node.note, at(&node.id, Some("note")));
        if [node.position.x, node.position.y]
            .iter()
            .any(|x| !x.is_finite() || x.abs() > 1_000_000.0)
        {
            check.issue(
                "graph.position",
                at(&node.id, Some("position")),
                "Coordinates exceed the editor bounds.",
            );
        }
        if node
            .frame
            .as_ref()
            .is_some_and(|f| !frames.contains(f.as_str()))
        {
            check.issue(
                "graph.reference",
                at(&node.id, Some("frame")),
                "Frame does not exist.",
            );
        }
        let Some(def) = definitions.get(node.kind.as_str()) else {
            check.issue(
                "graph.unknown_kind",
                at(&node.id, Some("kind")),
                "Node kind is not in the semantic contract.",
            );
            continue;
        };
        // Previous area-damage drafts carried no damage-type field. Keep their
        // stored representation; the semantic pass asks for an explicit type.
        let legacy_area_damage = node.kind == "skill-area-damage" && node.values.is_empty();
        if !legacy_area_damage && (node.values.len() != def.fields.len()
            || node
                .values
                .keys()
                .any(|key| !def.fields.iter().any(|f| &f.id == key)))
        {
            check.issue(
                "graph.fields",
                at(&node.id, Some("values")),
                "Missing or unsupported node fields.",
            );
        }
        for f in &def.fields {
            if legacy_area_damage { continue; }
            field(&mut check, node, f);
        }
    }
    let mut inputs = BTreeSet::new();
    let mut outputs = BTreeSet::new();
    let mut pairs = BTreeSet::new();
    for edge in &document.edges {
        let loc = Location {
            node: Some(edge.target.node.clone()),
            port: Some(edge.target.port.clone()),
            edge: Some(edge.id.clone()),
            field: None,
        };
        check.unique(&mut ids, &edge.id, loc.clone());
        let a = nodes
            .get(edge.source.node.as_str())
            .and_then(|n| definitions.get(n.kind.as_str()))
            .and_then(|d| d.outputs.iter().find(|p| p.id == edge.source.port));
        let b = nodes
            .get(edge.target.node.as_str())
            .and_then(|n| definitions.get(n.kind.as_str()))
            .and_then(|d| d.inputs.iter().find(|p| p.id == edge.target.port));
        let (Some(a), Some(b)) = (a, b) else {
            check.issue(
                "graph.reference",
                loc,
                "Source/output or target/input does not exist.",
            );
            continue;
        };
        if a.value_type != b.value_type {
            check.issue(
                "graph.port_type",
                loc.clone(),
                "Ports must have exactly the same type.",
            );
            let diagnostic = check.errors.last_mut().unwrap();
            diagnostic.expected = Some(b.value_type.clone());
            diagnostic.actual = Some(a.value_type.clone());
            diagnostic.related.push(Location {
                node: Some(edge.source.node.clone()),
                port: Some(edge.source.port.clone()),
                ..Default::default()
            });
        }
        if !pairs.insert((&edge.source, &edge.target)) {
            check.issue(
                "graph.duplicate_link",
                loc.clone(),
                "Duplicate edge endpoints.",
            );
        }
        if (!a.multiple && outputs.contains(&edge.source))
            || (!b.multiple && inputs.contains(&edge.target))
        {
            check.issue(
                "graph.port_capacity",
                loc,
                "A single connection port has multiple edges.",
            );
        }
        outputs.insert(edge.source.clone());
        inputs.insert(edge.target.clone());
    }
    check.finish(())?;
    let mut incoming: BTreeMap<Endpoint, Vec<&Edge>> = BTreeMap::new();
    let mut followers: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut degree: BTreeMap<&str, usize> = nodes.keys().map(|id| (*id, 0)).collect();
    for edge in &document.edges {
        incoming.entry(edge.target.clone()).or_default().push(edge);
        let target = nodes[edge.target.node.as_str()];
        let def = definitions[target.kind.as_str()];
        if !def
            .inputs
            .iter()
            .find(|p| p.id == edge.target.port)
            .unwrap()
            .active(&target.values)
        {
            continue;
        }
        followers
            .entry(&edge.source.node)
            .or_default()
            .push(&edge.target.node);
        *degree.get_mut(edge.target.node.as_str()).unwrap() += 1;
    }
    let mut ready: BTreeSet<_> = degree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(id, _)| *id)
        .collect();
    let mut visited = 0;
    while let Some(id) = ready.pop_first() {
        visited += 1;
        for target in followers.get(id).into_iter().flatten() {
            let count = degree.get_mut(target).unwrap();
            *count -= 1;
            if *count == 0 {
                ready.insert(*target);
            }
        }
    }
    if visited != nodes.len() {
        // Trace predecessors in the remaining subgraph so the diagnostic names
        // the cycle itself, rather than every innocent downstream consumer.
        let mut cursor = *degree.iter().find(|(_, count)| **count > 0).unwrap().0;
        let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
        let mut chain: Vec<&str> = vec![];
        loop {
            if let Some(start) = seen.get(cursor) {
                let cycle: Vec<_> = chain[*start..].iter().map(|id| at(id, None)).collect();
                check.issue(
                    "graph.cycle",
                    cycle[0].clone(),
                    "A dependency or execution cycle prevents normalization.",
                );
                check.errors.last_mut().unwrap().related = cycle[1..].to_vec();
                break;
            }
            seen.insert(cursor, chain.len());
            chain.push(cursor);
            let target = nodes[cursor];
            let def = definitions[target.kind.as_str()];
            let edge = document
                .edges
                .iter()
                .find(|e| {
                    e.target.node == cursor
                        && degree[e.source.node.as_str()] > 0
                        && def
                            .inputs
                            .iter()
                            .find(|p| p.id == e.target.port)
                            .unwrap()
                            .active(&target.values)
                })
                .expect("positive remaining indegree");
            cursor = &edge.source.node;
        }
        return check.finish(NormalizedDocument::empty());
    }
    let mut result = NormalizedDocument {
        document: document.id,
        digest: check.digest.clone(),
        semantic_version: SEMANTIC_VERSION.into(),
        semantic_fingerprint: String::new(),
        nodes: BTreeMap::new(),
    };
    for node in &document.nodes {
        if node.kind.starts_with("reroute-") || node.kind == "note" {
            continue;
        }
        let def = definitions[node.kind.as_str()];
        let values = node
            .values
            .iter()
            .filter(|(key, _)| {
                def.fields
                    .iter()
                    .find(|f| &f.id == *key)
                    .unwrap()
                    .active(&node.values)
            })
            .map(|(key, value)| {
                let value = match value {
                    Value::Bool(v) => FieldValue::Boolean(*v),
                    Value::Number(v) => FieldValue::Number(v.as_f64().unwrap()),
                    Value::String(v) => FieldValue::Text(v.clone()),
                    _ => unreachable!("validated fields"),
                };
                (key.clone(), value)
            })
            .collect();
        let mut normalized = NormalizedNode {
            kind: node.kind.clone(),
            values,
            inputs: BTreeMap::new(),
        };
        for port in &def.inputs {
            if !port.active(&node.values) {
                continue;
            }
            let endpoint = Endpoint {
                node: node.id.clone(),
                port: port.id.clone(),
            };
            let mut sources = vec![];
            for edge in incoming.get(&endpoint).into_iter().flatten() {
                let mut origin = Origin {
                    source: edge.source.clone(),
                    path: vec![edge.id.clone()],
                    reroutes: vec![],
                };
                while nodes[origin.source.node.as_str()]
                    .kind
                    .starts_with("reroute-")
                {
                    let route = origin.source.node.clone();
                    origin.reroutes.push(route.clone());
                    let input = Endpoint {
                        node: route,
                        port: "in".into(),
                    };
                    let Some(edges) = incoming.get(&input) else {
                        check.issue(
                            "graph.reroute_input",
                            loc_port(&origin.source),
                            "Reroute has no producer.",
                        );
                        break;
                    };
                    if edges.len() != 1 {
                        check.issue(
                            "graph.reroute_merge",
                            loc_port(&origin.source),
                            "Reroute cannot infer a multi-source join.",
                        );
                        break;
                    }
                    origin.path.push(edges[0].id.clone());
                    origin.source = edges[0].source.clone();
                }
                sources.push(origin);
            }
            sources.sort_by(|a, b| a.source.cmp(&b.source));
            if !sources.is_empty() {
                normalized.inputs.insert(port.id.clone(), sources);
            }
        }
        result.nodes.insert(node.id.clone(), normalized);
    }
    // Fingerprint deliberately excludes source-map paths, document identity,
    // titles, notes, positions, frames, reroutes and all input array ordering.
    let semantics: BTreeMap<_, _> = result
        .nodes
        .iter()
        .map(|(id, node)| {
            let inputs: BTreeMap<_, _> = node
                .inputs
                .iter()
                .map(|(port, values)| {
                    (
                        port,
                        values
                            .iter()
                            .map(|origin| &origin.source)
                            .collect::<Vec<_>>(),
                    )
                })
                .collect();
            (id, (&node.kind, &node.values, inputs))
        })
        .collect();
    result.semantic_fingerprint =
        hash(&serde_json::to_vec(&(SEMANTIC_VERSION, semantics)).expect("finite checked graph"));
    check.finish(result)
}
fn loc_port(endpoint: &Endpoint) -> Location {
    Location {
        node: Some(endpoint.node.clone()),
        port: Some(endpoint.port.clone()),
        ..Default::default()
    }
}
impl NormalizedDocument {
    fn empty() -> Self {
        Self {
            document: String::new(),
            digest: String::new(),
            semantic_version: String::new(),
            semantic_fingerprint: String::new(),
            nodes: BTreeMap::new(),
        }
    }
}
