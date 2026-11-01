//! Incomplete source is preserved by the source adapter, never lowered by filling
//! native values. This failure-only lookup gives an explicit hole its diagnostic.
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};
use serde_path_to_error::{Path, Segment};
use std::fmt;

pub(super) fn unresolved_at(input: &str, path: &Path) -> Option<(String, String, String)> {
    // Validate the entire JSON, including trailing input and duplicate keys, before
    // interpreting a marker. Value's ordinary decoder would silently keep the last
    // duplicate and could report a hole at a different value than Module decoded.
    let StrictValue(root) = serde_json::from_str(input).ok()?;
    let mut value = &root;
    let segment_count = path.iter().len();
    for (index, segment) in path.iter().enumerate() {
        // A denied field/variant reports the marker's reserved key, whereas a
        // scalar mismatch reports its containing value. Both identify the SAME
        // exact singleton marker; never search ancestors or unrelated leaves.
        let reserved = matches!(segment, Segment::Map { key } if key == "Unresolved")
            || matches!(segment, Segment::Enum { variant } if variant == "Unresolved");
        if index + 1 == segment_count && reserved {
            if let Some(reason) = marker_reason(value) {
                let display = path.to_string();
                let parent = display
                    .strip_suffix(".Unresolved")
                    .unwrap_or("$")
                    .to_owned();
                return Some((document_id(&root), reason.into(), parent));
            }
        }
        value = match segment {
            Segment::Seq { index } => value.as_array()?.get(*index)?,
            Segment::Map { key } => value.as_object()?.get(key)?,
            Segment::Enum { variant } => value.as_object()?.get(variant)?,
            Segment::Unknown => return None,
        };
    }
    Some((
        document_id(&root),
        marker_reason(value)?.into(),
        path.to_string(),
    ))
}

fn document_id(root: &Value) -> String {
    root.get("id").and_then(Value::as_str).unwrap_or("").into()
}

fn marker_reason(value: &Value) -> Option<&str> {
    let marker = value.as_object()?;
    if marker.len() != 1 {
        return None;
    }
    marker
        .get("Unresolved")?
        .as_str()
        .filter(|reason| !reason.trim().is_empty())
}

struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        struct StrictVisitor;
        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictValue;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("JSON without duplicate object keys")
            }

            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }

            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(StrictValue(value.into()))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(StrictValue(value.into()))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(StrictValue(value.into()))
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(value)
                    .map(|number| StrictValue(Value::Number(number)))
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(StrictValue(value.into()))
            }

            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(StrictValue(value.into()))
            }

            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(StrictValue(value)) = sequence.next_element()? {
                    values.push(value);
                }
                Ok(StrictValue(Value::Array(values)))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut object: A) -> Result<Self::Value, A::Error> {
                let mut values = Map::new();
                while let Some(key) = object.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom(format!("duplicate object key: {key}")));
                    }
                    let StrictValue(value) = object.next_value()?;
                    values.insert(key, value);
                }
                Ok(StrictValue(Value::Object(values)))
            }
        }
        decoder.deserialize_any(StrictVisitor)
    }
}
