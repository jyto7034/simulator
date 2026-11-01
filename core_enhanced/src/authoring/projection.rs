//! Admission before the first owned public JSON projection. Source DTO
//! construction and its native copies remain owned by their producers.

use super::control::{self, Resource};
use serde::{ser::SerializeMap, Serialize, Serializer};
use serde_json::Value;

struct Counter(usize);

fn interrupted(_: crate::game::behavior::GameError) -> std::io::Error {
    // The owning control scope retains the typed first failure and phase.
    std::io::Error::other("authoring public projection interrupted")
}

impl std::io::Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        control::admit(
            Resource::ResultBytes,
            self.0,
            bytes.len(),
            "public_projection_size",
        )
        .and_then(|()| control::charge(bytes.len() as u64, "public_projection_scan"))
        .map_err(interrupted)?;
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("public projection size overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) fn to_value<T: Serialize>(value: T) -> serde_json::Result<Value> {
    if !control::is_active() {
        return serde_json::to_value(value);
    }
    control::poll("public_projection_entry").map_err(|e| serde_json::Error::io(interrupted(e)))?;
    let mut counter = Counter(0);
    serde_json::to_writer(&mut counter, &value)?;
    control::charge(counter.0 as u64, "public_projection_value")
        .map_err(|e| serde_json::Error::io(interrupted(e)))?;
    // Keep native Number/map/Option serialization semantics; no JSON parse.
    let result = serde_json::to_value(value)?;
    control::poll("public_projection_complete")
        .map_err(|e| serde_json::Error::io(interrupted(e)))?;
    Ok(result)
}

/// Formats a diagnostic without first building an unbounded owned message.
pub(crate) fn display(value: impl std::fmt::Display) -> serde_json::Result<String> {
    if !control::is_active() {
        return Ok(value.to_string());
    }
    control::poll("diagnostic_message_entry").map_err(|e| serde_json::Error::io(interrupted(e)))?;
    struct Message(String);
    impl std::fmt::Write for Message {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            control::admit(
                Resource::ResultBytes,
                self.0.len(),
                text.len(),
                "diagnostic_message_size",
            )
            .and_then(|()| control::charge(text.len() as u64, "diagnostic_message_copy"))
            .map_err(|_| std::fmt::Error)?;
            self.0.push_str(text);
            Ok(())
        }
    }
    let mut message = Message(String::new());
    std::fmt::write(&mut message, format_args!("{value}")).map_err(|_| {
        serde_json::Error::io(std::io::Error::other("diagnostic formatting failed"))
    })?;
    control::poll("diagnostic_message_complete")
        .map_err(|e| serde_json::Error::io(interrupted(e)))?;
    Ok(message.0)
}

pub(crate) trait Fields {
    const LEN: usize;
    fn entries<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error>;
}

impl Fields for () {
    const LEN: usize = 0;
    fn entries<M: SerializeMap>(&self, _: &mut M) -> Result<(), M::Error> {
        Ok(())
    }
}

impl<T: Serialize, Tail: Fields> Fields for (&'static str, T, Tail) {
    const LEN: usize = 1 + Tail::LEN;
    fn entries<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
        map.serialize_entry(self.0, &self.1)?;
        self.2.entries(map)
    }
}

pub(crate) struct Object<F>(pub F);

impl<F: Fields> Serialize for Object<F> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(F::LEN))?;
        self.0.entries(&mut map)?;
        map.end()
    }
}

macro_rules! object {
    (@fields) => { () };
    (@fields $key:literal : $value:expr $(, $rest_key:literal : $rest_value:expr)* $(,)?) => {
        ($key, &$value, $crate::authoring::projection::object!(@fields $($rest_key : $rest_value),*))
    };
    ({ $($key:literal : $value:expr),* $(,)? }) => {
        $crate::authoring::projection::to_value(&$crate::authoring::projection::Object(
            $crate::authoring::projection::object!(@fields $($key : $value),*)
        ))
    };
}
pub(crate) use object;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
    use std::cell::Cell;

    struct Seen<'a> {
        calls: &'a Cell<usize>,
        text: &'a str,
    }
    impl Serialize for Seen<'_> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.calls.set(self.calls.get() + 1);
            serializer.serialize_str(self.text)
        }
    }

    #[test]
    fn public_projection_counts_before_first_value_and_preserves_native_numbers() {
        let calls = Cell::new(0);
        let source = Seen {
            calls: &calls,
            text: "borrowed\nvalue",
        };
        let expected = serde_json::to_value(&source).unwrap();
        let bytes = serde_json::to_vec(&expected).unwrap().len();
        for (limit, work, success) in [
            (bytes - 1, 10_000, false),
            (bytes, 2 * bytes as u64 - 1, false),
            (bytes, 2 * bytes as u64, true),
        ] {
            calls.set(0);
            let mut limits = ExecutionLimits::default();
            limits.max_result_bytes = limit;
            limits.max_work = work;
            let result = control::run(
                &limits,
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || to_value(&source),
            );
            if success {
                let (value, usage) = result.unwrap();
                assert_eq!(value.unwrap(), expected);
                assert_eq!(usage.result_bytes_peak, bytes);
                assert_eq!(usage.work, 2 * bytes as u64);
                assert_eq!(calls.get(), 2);
            } else {
                let error = result.unwrap_err();
                assert_eq!(
                    error.code,
                    if limit < bytes {
                        "authoring_result_limit"
                    } else {
                        "authoring_work_limit"
                    }
                );
                assert_eq!(calls.get(), 1, "native Value serialization did not run");
            }
        }
        calls.set(0);
        assert_eq!(to_value(&source).unwrap(), expected);
        assert_eq!(calls.get(), 1, "ordinary native path visits once");
        let cases = [
            0.1_f64,
            -0.0,
            f64::MIN_POSITIVE,
            f64::MAX,
            f64::NAN,
            f64::INFINITY,
        ];
        let (value, _) = control::run(&ExecutionLimits::default(), &CancellationToken::default(), ExecutionPhase::TrialExecution, || {
            object!({"numbers":cases, "large":u64::MAX, "optional":Option::<u64>::None, "nested":object!({"value":f32::MIN_POSITIVE})?})
        }).unwrap();
        assert_eq!(
            value.unwrap(),
            serde_json::json!({"numbers":cases, "large":u64::MAX, "optional":Option::<u64>::None, "nested":{"value":f32::MIN_POSITIVE}})
        );
    }
}
