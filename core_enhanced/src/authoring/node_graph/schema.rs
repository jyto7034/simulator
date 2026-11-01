use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub format: String,
    pub version: u32,
    pub id: String,
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub frames: Vec<Frame>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub position: Position,
    pub values: BTreeMap<String, serde_json::Value>,
    pub note: String,
    #[serde(deserialize_with = "required_optional")]
    pub frame: Option<String>,
}
fn required_optional<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub id: String,
    pub title: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub id: String,
    pub source: Endpoint,
    pub target: Endpoint,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    pub node: String,
    pub port: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Location {
    pub node: Option<String>,
    pub port: Option<String>,
    pub edge: Option<String>,
    pub field: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub phase: String,
    pub document: Option<String>,
    pub digest: String,
    pub primary: Location,
    pub related: Vec<Location>,
    pub reason: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldValue {
    Number(f64),
    Text(String),
    Boolean(bool),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Origin {
    pub source: Endpoint,
    /// Edges from consumer toward producer, including every collapsed reroute.
    pub path: Vec<String>,
    pub reroutes: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizedNode {
    pub kind: String,
    pub values: BTreeMap<String, FieldValue>,
    pub inputs: BTreeMap<String, Vec<Origin>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizedDocument {
    pub document: String,
    pub digest: String,
    pub semantic_version: String,
    pub semantic_fingerprint: String,
    pub nodes: BTreeMap<String, NormalizedNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Catalog {
    pub schema_version: u32,
    pub semantic_version: String,
    pub definitions: Vec<Definition>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Definition {
    pub kind: String,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub fields: Vec<Field>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Port {
    pub id: String,
    pub value_type: String,
    pub multiple: bool,
    pub required: bool,
    pub active_when: Option<Vec<BTreeMap<String, serde_json::Value>>>,
    pub required_when: Option<Vec<BTreeMap<String, serde_json::Value>>>,
}
impl Port {
    pub fn active(&self, values: &BTreeMap<String, serde_json::Value>) -> bool {
        self.active_when.as_ref().map_or(true, |cases| {
            cases.iter().any(|case| {
                case.iter()
                    .all(|(key, value)| values.get(key) == Some(value))
            })
        })
    }
    pub fn required(&self, values: &BTreeMap<String, serde_json::Value>) -> bool {
        self.required_when.as_ref().map_or(self.required, |cases| {
            cases.iter().any(|case| {
                case.iter()
                    .all(|(key, value)| values.get(key) == Some(value))
            })
        })
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Field {
    pub id: String,
    pub value_type: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    #[serde(default)]
    pub integer: bool,
    pub unit: Option<String>,
    pub options: Option<Vec<String>>,
    pub active_when: Option<Vec<BTreeMap<String, serde_json::Value>>>,
}
impl Field {
    pub fn active(&self, values: &BTreeMap<String, serde_json::Value>) -> bool {
        self.active_when.as_ref().map_or(true, |cases| {
            cases.iter().any(|case| {
                case.iter()
                    .all(|(key, value)| values.get(key) == Some(value))
            })
        })
    }
}
