//! Restricted YAML 1.2 input. Event admission bounds nesting before constructing
//! values, and rejects extensions instead of silently normalizing their meaning.
use super::{compile_json, Artifact, Diagnostic, DiagnosticCode, SourceTrace, MAX_INPUT_BYTES};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use yaml_rust2::{
    parser::{Event, Parser},
    scanner::TScalarStyle,
    Yaml,
};

const MAX_DEPTH: usize = 64;
const MAX_VALUES: usize = 100_000;
const SAFE_INTEGER: i64 = (1i64 << 53) - 1;

enum Container {
    Map(Map<String, Value>, Option<String>),
    Sequence(Vec<Value>),
}
struct Frame {
    value: Container,
    path: String,
}
fn next_path(stack: &[Frame]) -> String {
    match stack.last() {
        Some(Frame {
            value: Container::Map(_, Some(key)),
            path,
        }) => {
            if path.is_empty() {
                key.clone()
            } else {
                format!("{path}.{key}")
            }
        }
        Some(Frame {
            value: Container::Sequence(values),
            path,
        }) => format!("{path}[{}]", values.len()),
        Some(frame) => frame.path.clone(),
        None => String::new(),
    }
}
fn insert(stack: &mut [Frame], root: &mut Option<Value>, value: Value) -> Result<(), String> {
    match stack.last_mut() {
        Some(Frame {
            value: Container::Map(map, pending),
            ..
        }) => {
            if let Some(key) = pending.take() {
                map.insert(key, value);
            } else {
                let Value::String(key) = value else {
                    return Err("mapping keys must be strings".into());
                };
                if key == "<<" {
                    return Err("YAML merge is not supported".into());
                }
                if map.contains_key(&key) {
                    return Err(format!("duplicate mapping key: {key}"));
                }
                *pending = Some(key);
            }
        }
        Some(Frame {
            value: Container::Sequence(values),
            ..
        }) => values.push(value),
        None => {
            if root.is_some() {
                return Err("exactly one YAML document is required".into());
            }
            *root = Some(value);
        }
    }
    Ok(())
}
fn scalar(text: String, style: TScalarStyle) -> Result<Value, String> {
    if style != TScalarStyle::Plain {
        return Ok(Value::String(text));
    }
    if text.is_empty() {
        return Err("empty value: use explicit null for a null value".into());
    }
    if matches!(text.as_str(), "Null" | "NULL") {
        return Ok(Value::Null);
    }
    match Yaml::from_str(&text) {
        Yaml::Null => Ok(Value::Null),
        Yaml::Boolean(value) => Ok(Value::Bool(value)),
        Yaml::String(value) => Ok(Value::String(value)),
        Yaml::Integer(value) if (-SAFE_INTEGER..=SAFE_INTEGER).contains(&value) => Ok(value.into()),
        Yaml::Integer(_) => Err("integer exceeds the editor's exact numeric range".into()),
        Yaml::Real(value) => {
            let number = value
                .parse::<f64>()
                .map_err(|_| "non-finite or invalid number")?;
            if !number.is_finite() || (number.fract() == 0.0 && number.abs() > SAFE_INTEGER as f64)
            {
                return Err("non-finite or inexact integer is not supported".into());
            }
            // JS has one numeric type; canonicalize integral floats identically.
            if number.fract() == 0.0 {
                Ok((number as i64).into())
            } else {
                Ok(Value::Number(
                    Number::from_f64(number).ok_or("invalid number")?,
                ))
            }
        }
        _ => Err("unsupported scalar".into()),
    }
}

/// Also used by parity tests: this is the actual compiler's input adapter.
pub fn parse_yaml(input: &str) -> Result<Value, Diagnostic> {
    parse_yaml_with_nesting(input, MAX_DEPTH)
}

fn parse_yaml_with_nesting(input: &str, max_nesting: usize) -> Result<Value, Diagnostic> {
    let digest = format!("{:x}", Sha256::digest(input.as_bytes()));
    let error = |code, path: String, reason: String| Diagnostic {
        code,
        document: String::new(),
        source_digest: digest.clone(),
        source: SourceTrace {
            definition: if path.is_empty() { "$".into() } else { path },
            calls: vec![],
            inputs: vec![],
        },
        reason,
    };
    if input.len() > MAX_INPUT_BYTES {
        return Err(error(
            DiagnosticCode::Limit,
            String::new(),
            "module byte limit exceeded".into(),
        ));
    }
    // A single fixed schema; directives must not switch interpretation to YAML 1.1.
    if input.lines().any(|line| line.starts_with('%')) {
        return Err(error(
            DiagnosticCode::Parse,
            String::new(),
            "YAML directives are not supported; use the fixed 1.2 schema".into(),
        ));
    }
    let mut parser = Parser::new_from_str(input);
    let mut stack: Vec<Frame> = vec![];
    let mut root = None;
    let mut documents = 0;
    let mut values = 0;
    loop {
        let (event, marker) = parser
            .next_token()
            .map_err(|e| error(DiagnosticCode::Parse, next_path(&stack), e.to_string()))?;
        let path = next_path(&stack);
        let fail = |message: String| {
            error(
                DiagnosticCode::Parse,
                path.clone(),
                format!("line {}: {message}", marker.line()),
            )
        };
        let is_mapping = matches!(&event, Event::MappingStart(..));
        match event {
            Event::StreamStart | Event::DocumentEnd => {}
            Event::StreamEnd => break,
            Event::DocumentStart => {
                documents += 1;
                if documents > 1 {
                    return Err(fail("exactly one YAML document is required".into()));
                }
            }
            Event::Alias(_) => return Err(fail("YAML aliases are not supported".into())),
            Event::MappingStart(anchor, tag) | Event::SequenceStart(anchor, tag) => {
                if anchor != 0 || tag.is_some() {
                    return Err(fail("YAML anchors and tags are not supported".into()));
                }
                if stack.len() >= max_nesting.min(MAX_DEPTH) {
                    return Err(error(
                        DiagnosticCode::Limit,
                        path,
                        "YAML nesting limit exceeded".into(),
                    ));
                }
                if matches!(
                    stack.last(),
                    Some(Frame {
                        value: Container::Map(_, None),
                        ..
                    })
                ) {
                    return Err(fail("mapping keys must be strings".into()));
                }
                values += 1;
                if values > MAX_VALUES {
                    return Err(error(
                        DiagnosticCode::Limit,
                        path,
                        "YAML value limit exceeded".into(),
                    ));
                }
                stack.push(Frame {
                    path,
                    value: if is_mapping {
                        Container::Map(Map::new(), None)
                    } else {
                        Container::Sequence(vec![])
                    },
                });
            }
            Event::MappingEnd | Event::SequenceEnd => {
                let frame = stack
                    .pop()
                    .ok_or_else(|| fail("unbalanced collection".into()))?;
                let value = match frame.value {
                    Container::Map(map, None) => Value::Object(map),
                    Container::Map(_, Some(_)) => {
                        return Err(fail("mapping value is missing".into()))
                    }
                    Container::Sequence(values) => Value::Array(values),
                };
                insert(&mut stack, &mut root, value).map_err(fail)?;
            }
            Event::Scalar(text, style, anchor, tag) => {
                if anchor != 0 || tag.is_some() {
                    return Err(fail("YAML anchors and tags are not supported".into()));
                }
                values += 1;
                if values > MAX_VALUES {
                    return Err(error(
                        DiagnosticCode::Limit,
                        path,
                        "YAML value limit exceeded".into(),
                    ));
                }
                insert(&mut stack, &mut root, scalar(text, style).map_err(&fail)?).map_err(fail)?;
            }
            Event::Nothing => return Err(fail("unexpected parser event".into())),
        }
    }
    root.ok_or_else(|| {
        error(
            DiagnosticCode::Parse,
            String::new(),
            "empty YAML document".into(),
        )
    })
}

pub fn compile_yaml(input: &str) -> Result<Artifact, Diagnostic> {
    compile_yaml_with_nesting(input, MAX_DEPTH)
}

/// A content host may tighten, but never raise, the authoring dialect ceiling.
pub(crate) fn compile_yaml_with_nesting(
    input: &str,
    max_nesting: usize,
) -> Result<Artifact, Diagnostic> {
    let value = parse_yaml_with_nesting(input, max_nesting)?;
    let normalized = serde_json::to_string(&value).expect("finite JSON value");
    let digest = format!("{:x}", Sha256::digest(input.as_bytes()));
    match compile_json(&normalized) {
        Ok(mut artifact) => {
            artifact.source_digest = digest;
            Ok(artifact)
        }
        Err(mut error) => {
            error.source_digest = digest;
            Err(error)
        }
    }
}
