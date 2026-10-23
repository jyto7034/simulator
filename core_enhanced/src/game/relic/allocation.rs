//! Allocation-free encoded-shape admission for continuation and owned projections.
//! Continue admissions apply with or without relics; the module path is historical.
//! This measures serialization, not native clone shape or process allocator bytes.
use crate::game::{battle::runtime_control as native, behavior::GameError};
use serde::Serialize;
use serde_json::Value;
use std::{io, mem::size_of};

struct Shape {
    encoded: usize,
    string_bytes: usize,
    members: usize,
    objects: usize,
    object_nodes: usize,
    separators: usize,
    containers: usize,
    in_string: bool,
    escaped: bool,
    work_site: Option<&'static str>,
    // Same nesting ceiling as serde_json's supported decoder. A fixed stack
    // keeps this sizing pass allocation-free, including chunked input scans.
    object_members: [usize; 128],
    depth: usize,
}
impl Default for Shape {
    fn default() -> Self {
        Self { encoded: 0, string_bytes: 0, members: 0, objects: 0,
            object_nodes: 0, separators: 0, containers: 0, in_string: false,
            escaped: false, work_site: None, object_members: [0; 128], depth: 0 }
    }
}
fn add(value: &mut usize, delta: usize) -> io::Result<()> {
    *value = value
        .checked_add(delta)
        .ok_or_else(|| io::Error::from(io::ErrorKind::OutOfMemory))?;
    Ok(())
}
impl io::Write for Shape {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        crate::authoring::control::charge(
            bytes.len() as u64,
            self.work_site.unwrap_or("relic_serialized_shape"),
        )
        // write_all retries Interrupted. A sticky gameplay fault must
        // terminate serialization and reach control::poll instead.
        .map_err(|_| io::Error::from(io::ErrorKind::Other))?;
        add(&mut self.encoded, bytes.len())?;
        for byte in bytes {
            if self.in_string {
                add(&mut self.string_bytes, 1)?;
                if self.escaped {
                    self.escaped = false;
                } else if *byte == b'\\' {
                    self.escaped = true;
                } else if *byte == b'"' {
                    self.in_string = false;
                }
            } else {
                match byte {
                    b'"' => {
                        self.in_string = true;
                        add(&mut self.string_bytes, 1)?;
                    }
                    b':' => {
                        add(&mut self.members, 1)?;
                        let depth = self.depth.checked_sub(1)
                            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?;
                        add(&mut self.object_members[depth], 1)?;
                    }
                    b',' => add(&mut self.separators, 1)?,
                    b'[' | b'{' => {
                        add(&mut self.containers, 1)?;
                        if *byte == b'{' { add(&mut self.objects, 1)?; }
                        let frame = self.object_members.get_mut(self.depth)
                            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?;
                        *frame = 0;
                        self.depth += 1;
                    }
                    b']' | b'}' => {
                        self.depth = self.depth.checked_sub(1)
                            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?;
                        if *byte == b'}' {
                            let members = self.object_members[self.depth];
                            // A root holds up to eleven entries without a split.
                            // Larger trees have >=5 entries per non-root node.
                            let nodes = if members <= 11 { 1 } else { 1 + (members - 1) / 5 };
                            add(&mut self.object_nodes, nodes)?;
                        }
                    }
                    _ => (),
                }
            }
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn shape(value: &impl Serialize) -> Result<Shape, GameError> {
    shape_at(
        value,
        "relic_serialized_shape",
        "relic_serialized_shape_complete",
    )
}
fn shape_at(
    value: &impl Serialize,
    work_site: &'static str,
    poll_site: &'static str,
) -> Result<Shape, GameError> {
    let mut shape = Shape {
        work_site: Some(work_site),
        ..Shape::default()
    };
    let result = serde_json::to_writer(&mut shape, value);
    crate::authoring::control::poll(poll_site)?;
    result.map_err(|error| GameError::InvalidStaticData(error.to_string()))?;
    Ok(shape)
}
fn overflow(site: &'static str) -> GameError {
    native::reject(native::BattleRuntimeFault {
        code: native::RuntimeFaultCode::CounterOverflow,
        site,
        limit: None,
        requested: None,
    })
}
fn admit(bytes: Option<usize>, site: &'static str) -> Result<usize, GameError> {
    let bytes = bytes.ok_or_else(|| overflow(site))?;
    native::admit_retained_growth(0, Some(bytes), site)?;
    Ok(bytes)
}
pub(super) fn map_node_bound() -> Option<usize> {
    size_of::<(String, Value)>()
        .checked_mul(11)?
        .checked_add(224)
}
/// Reserve the closed continuation's independent JSON Value tree. Each object
/// reserves its own B-tree shape, including the unsplit eleven-entry root;
/// commas plus container starts overcount its array elements. The producer's derived String/UUID
/// serializers own strings directly, so quoted/escaped byte counts bound them.
/// Normalization key buffers are reserved only at their actual sort boundary;
/// canonical encoded Vec and Arc allocations have separate admissions below.
pub(crate) fn admit_json_value(value: &impl Serialize) -> Result<(), GameError> {
    admit_continue_storage(shape(value)?, false, 0)
}
fn admit_continue_storage(s: Shape, decoding: bool, run_entries: usize) -> Result<(), GameError> {
    let site = if decoding { "continue_typed_decode" } else { "relic_continue_value" };
    let slots = s.separators.checked_add(s.containers);
    // Count each object's nodes independently. Summing all members before
    // dividing invents extra nodes for thousands of small, unsplit objects.
    // The closed typed decoder also fits this bound: fixed structs/enums and
    // map entries are covered by their object storage, sequences by slots,
    // and strings/scratch by their separate terms. It does not materialize
    // one B-tree node for every struct field. Generic projection guards below
    // intentionally keep their older, broader bound.
    // Toolchain/dependency changes must revalidate this layout assumption,
    // already shared by map_node_bound's eleven-entry node capacity.
    // RunResourceEntry has one record object and one externally tagged owner
    // object on the wire, but both decode into a single fixed Rust entry, not
    // two B-tree nodes. Count only these schema-known records specially; all
    // other types retain the conservative generic object bound. Vec growth is
    // bounded by twice the live entry count; strings/scratch stay below.
    let nodes = run_entries.checked_mul(2).and_then(|n| s.object_nodes.checked_sub(n));
    let bytes = nodes
        .and_then(|n| n.checked_mul(map_node_bound()?))
        .and_then(|n| n.checked_add(run_entries.checked_mul(std::mem::size_of::<crate::game::resource::durable::RunResourceEntry>() * 2)?))
        .and_then(|n| n.checked_add(s.string_bytes))
        .and_then(|n| n.checked_add(slots?.checked_mul(size_of::<Value>() * 2)?))
        .and_then(|n| {
            if decoding {
                // Preserve the whole-typed-decode scratch reserve, including
                // escaped-string scratch/geometric growth. Decoding creates no
                // normalization keys or canonical encoded output buffer.
                n.checked_add(s.encoded.checked_mul(4)?)
            } else {
                Some(n)
            }
        });
    admit(bytes, site)?;
    Ok(())
}
// This generic typed projection guard retains its original conservative terms.
// It is deliberately independent of the continuation-only Value/decode guards.
fn admit_shape_value(s: Shape, site: &'static str) -> Result<(), GameError> {
    let slots = s.separators.checked_add(s.containers);
    let bytes = s
        .members
        .checked_mul(map_node_bound().ok_or_else(|| overflow(site))?)
        .and_then(|n| n.checked_add(s.string_bytes))
        .and_then(|n| n.checked_add(slots?.checked_mul(size_of::<Value>() * 2)?))
        .and_then(|n| {
            n.checked_add(slots?.checked_mul((size_of::<Vec<u8>>() + size_of::<usize>()) * 2)?)
        })
        .and_then(|n| n.checked_add(slots?.checked_mul(128 * 2)?))
        .and_then(|n| n.checked_add(s.encoded.checked_mul(4)?));
    admit(bytes, site)?;
    Ok(())
}
/// The final profile producer makes typed owned clones. The same lexical bound
/// reserves more than their String/Vec/map storage before that producer runs;
/// it creates no JSON Value or buffer, and charges each producer invocation.
pub(crate) fn admit_profile_projection(value: &impl Serialize) -> Result<(), GameError> {
    admit_shape_value(shape(value)?, "relic_numeric_profile")
}

/// Opt-in owned DTO/session clone bound, before any independent allocation.
pub(crate) fn admit_owned_projection(
    value: &impl Serialize,
    site: &'static str,
) -> Result<(), GameError> {
    admit_shape_value(shape(value)?, site)
}

/// Reserve typed decoding before materializing any run
/// owner. Keep the original closed-checkpoint map/sequence/string/scratch
/// reserve, excluding normalization sort allocations that decoding never runs.
/// This admission does not copy the input or seed the operation a second time.
pub(crate) fn admit_json_input(bytes: &[u8], run_entries: usize) -> Result<(), GameError> {
    let mut s = Shape::default();
    let result = io::Write::write_all(&mut s, bytes);
    crate::authoring::control::poll("relic_continue_decode_shape")?;
    result.map_err(|error| GameError::InvalidStaticData(error.to_string()))?;
    admit_continue_storage(s, true, run_entries)
}

/// Reserve the actual cached-key sort workspace before calling it. The tuple
/// bound uses 2x len; each child to_vec starts at 128 bytes and cumulative
/// geometric capacities are bounded by 2x128 + 4x its encoded length. All child
/// keys coexist while sorting. Their allocation-free shape scan replaces the
/// existing continue_set_key scan, with shared-to-native forwarding once.
pub(crate) fn admit_normalization_keys(values: &[Value]) -> Result<(), GameError> {
    let site = "relic_continue_normalization";
    let mut bytes = values
        .len()
        .checked_mul(size_of::<Vec<u8>>() + size_of::<usize>())
        .and_then(|n| n.checked_mul(2))
        .ok_or_else(|| overflow(site))?;
    for value in values {
        let encoded = shape_at(value, "continue_set_key", "continue_set_key")?.encoded;
        let key_bytes = encoded
            .checked_mul(4)
            .and_then(|n| n.checked_add(128 * 2))
            .ok_or_else(|| overflow(site))?;
        bytes = bytes.checked_add(key_bytes).ok_or_else(|| overflow(site))?;
    }
    crate::authoring::control::poll("continue_set_key")?;
    admit(Some(bytes), site)?;
    Ok(())
}

/// Reserve an exact-capacity encoded Vec. The supported serde_json BTreeMap
/// iterator already orders object keys; the writer allocates no key vectors.
/// Canonical sorting changes order, not the JSON encoded byte count.
pub(crate) fn admit_json_buffer(value: &Value) -> Result<usize, GameError> {
    let s = shape(value)?;
    admit(Some(s.encoded), "relic_continue_bytes")?;
    Ok(s.encoded)
}
pub(crate) fn admit_arc(bytes: usize, site: &'static str) -> Result<(), GameError> {
    admit(bytes.checked_add(64), site)?;
    Ok(())
}
pub(crate) fn admit_codec_fields(
    keys: &[&str],
    extra_string_bytes: usize,
) -> Result<(), GameError> {
    let strings = keys
        .iter()
        .try_fold(extra_string_bytes, |n, key| n.checked_add(key.len()));
    admit(
        keys.len()
            .checked_mul(map_node_bound().ok_or_else(|| overflow("relic_continue_envelope"))?)
            .and_then(|n| n.checked_add(strings?)),
        "relic_continue_envelope",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encoded_shape_handles_escaped_quotes_punctuation_and_unicode_across_chunks() {
        let value = serde_json::json!({"a": ["α,:{}[]\\\"é", "\\", 12], "b": {"x:y": "中"}});
        let bytes = serde_json::to_vec(&value).unwrap();
        let normal = shape(&value).unwrap();
        assert_eq!(normal.encoded, bytes.len());
        assert_eq!(normal.members, 3);
        assert_eq!(normal.objects, 2);
        assert_eq!(normal.object_nodes, 2);
        let mut split = Shape::default();
        for byte in &bytes {
            io::Write::write_all(&mut split, &[*byte]).unwrap();
        }
        assert_eq!(split.string_bytes, normal.string_bytes);
        assert_eq!(split.members, normal.members);
        assert_eq!(split.objects, normal.objects);
        assert_eq!(split.object_nodes, normal.object_nodes);
        assert_eq!(split.separators, normal.separators);
        assert!(!split.in_string);
    }
    #[test]
    fn independent_small_objects_do_not_invent_split_nodes() {
        let small = serde_json::json!({"a": 1, "b": 2, "c": 3, "d": 4});
        let many = Value::Array(vec![small; 3500]);
        let measured = shape(&many).unwrap();
        assert_eq!(measured.members, 14000);
        assert_eq!(measured.object_nodes, 3500, "each four-field object needs one unsplit root");
        for (members, nodes) in [(0, 1), (11, 1), (12, 3)] {
            let object = Value::Object((0..members).map(|i| (i.to_string(), Value::Null)).collect());
            assert_eq!(shape(&object).unwrap().object_nodes, nodes);
        }
    }
    #[test]
    fn checked_shape_failure_precedes_any_owned_output() {
        let mut s = Shape {
            encoded: usize::MAX,
            ..Default::default()
        };
        assert!(io::Write::write(&mut s, b"x").is_err());
        let failure = native::run(
            native::RuntimeFactLimits {
                max_work: 1,
                ..Default::default()
            },
            || admit_json_value(&serde_json::json!({"x": "y"})),
        )
        .unwrap_err();
        assert!(matches!(
            failure,
            GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                code: native::RuntimeFaultCode::WorkLimit,
                ..
            })
        ));
    }
}
