//! Independent bounded rendering of borrowed failures, after gameplay has failed.

use std::{fmt, io};

use serde::{ser, Serialize};

use super::{AuthoringError, CancellationToken, TrialFailure};

#[derive(Debug, Clone, Copy)]
pub struct DiagnosticLimits {
    pub max_source_bytes: usize,
    pub max_output_bytes: usize,
    pub max_nodes: usize,
    pub max_depth: usize,
    pub max_work: u64,
}
impl Default for DiagnosticLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: 1 << 20,
            max_output_bytes: 1 << 20,
            max_nodes: 16_384,
            max_depth: 32,
            max_work: 1 << 24,
        }
    }
}
impl DiagnosticLimits {
    fn validate(self) -> Result<(), DiagnosticRenderError> {
        if self.max_source_bytes == 0
            || self.max_source_bytes > 8 << 20
            || self.max_output_bytes == 0
            || self.max_output_bytes > 8 << 20
            || self.max_nodes == 0
            || self.max_nodes > 262_144
            || self.max_depth == 0
            || self.max_depth > 64
            || self.max_work == 0
            || self.max_work > 1 << 30
        {
            return Err(DiagnosticRenderError::InvalidLimits);
        }
        Ok(())
    }
}

/// A rendering failure never replaces or owns the original diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticRenderError {
    InvalidLimits,
    Cancelled,
    SourceBytes,
    OutputBytes,
    Nodes,
    Depth,
    Work,
    Allocation,
    UnsupportedSerialization,
    Encoding,
}
impl fmt::Display for DiagnosticRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidLimits => "invalid diagnostic rendering limits",
            Self::Cancelled => "diagnostic rendering cancelled",
            Self::SourceBytes => "diagnostic source byte limit",
            Self::OutputBytes => "diagnostic output byte limit",
            Self::Nodes => "diagnostic node limit",
            Self::Depth => "diagnostic depth limit",
            Self::Work => "diagnostic work limit",
            Self::Allocation => "diagnostic allocation refused",
            Self::UnsupportedSerialization => "unsupported diagnostic serialization",
            Self::Encoding => "diagnostic encoding failed",
        })
    }
}
impl std::error::Error for DiagnosticRenderError {}
impl ser::Error for DiagnosticRenderError {
    fn custom<T: fmt::Display>(_: T) -> Self {
        Self::Encoding
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiagnosticUsage {
    pub work: u64,
    pub nodes: usize,
    pub source_bytes: usize,
    pub output_bytes: usize,
}

#[derive(Debug)]
pub struct RenderedDiagnostic {
    document: String,
    usage: DiagnosticUsage,
}
impl RenderedDiagnostic {
    pub fn document(&self) -> &str {
        &self.document
    }
    pub fn usage(&self) -> DiagnosticUsage {
        self.usage
    }
}

/// Uses its own limits and cancellation, independent of the failed attempt.
pub fn render_error(
    error: &AuthoringError,
    limits: &DiagnosticLimits,
    cancellation: &CancellationToken,
) -> Result<RenderedDiagnostic, DiagnosticRenderError> {
    render(error, limits, cancellation)
}

pub fn render_trial_failure(
    failure: &TrialFailure,
    limits: &DiagnosticLimits,
    cancellation: &CancellationToken,
) -> Result<RenderedDiagnostic, DiagnosticRenderError> {
    render(failure, limits, cancellation)
}

struct Budget<'a> {
    limits: DiagnosticLimits,
    cancellation: &'a CancellationToken,
    usage: DiagnosticUsage,
}
impl Budget<'_> {
    fn charge(&mut self, work: u64) -> Result<(), DiagnosticRenderError> {
        if self.cancellation.is_cancelled() {
            return Err(DiagnosticRenderError::Cancelled);
        }
        let next = self
            .usage
            .work
            .checked_add(work)
            .ok_or(DiagnosticRenderError::Work)?;
        if next > self.limits.max_work {
            return Err(DiagnosticRenderError::Work);
        }
        self.usage.work = next;
        Ok(())
    }
    fn node(&mut self, depth: usize) -> Result<(), DiagnosticRenderError> {
        self.charge(1)?;
        if depth > self.limits.max_depth {
            return Err(DiagnosticRenderError::Depth);
        }
        if self.usage.nodes == self.limits.max_nodes {
            return Err(DiagnosticRenderError::Nodes);
        }
        self.usage.nodes += 1;
        Ok(())
    }
    fn string(&mut self, text: &str) -> Result<(), DiagnosticRenderError> {
        let next = self
            .usage
            .source_bytes
            .checked_add(text.len())
            .ok_or(DiagnosticRenderError::SourceBytes)?;
        if next > self.limits.max_source_bytes {
            return Err(DiagnosticRenderError::SourceBytes);
        }
        self.charge(text.len() as u64)?;
        self.usage.source_bytes = next;
        Ok(())
    }
}

// This private serializer visits only the two closed public diagnostic graphs.
// It never formats values or builds a temporary Value/string/collection.
struct Scan<'a, 'b> {
    budget: &'a mut Budget<'b>,
    depth: usize,
}
impl Scan<'_, '_> {
    fn scalar(self) -> Result<(), DiagnosticRenderError> {
        self.budget.node(self.depth)
    }
    fn container(self, len: Option<usize>) -> Result<Self, DiagnosticRenderError> {
        self.budget.node(self.depth)?;
        let len = len.ok_or(DiagnosticRenderError::UnsupportedSerialization)?;
        if len > self.budget.limits.max_nodes - self.budget.usage.nodes {
            return Err(DiagnosticRenderError::Nodes);
        }
        Ok(self)
    }
    fn child<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), DiagnosticRenderError> {
        value.serialize(Scan {
            budget: self.budget,
            depth: self.depth + 1,
        })
    }
}
macro_rules! scalars {
    ($($method:ident($ty:ty)),* $(,)?) => {$(
        fn $method(self, _: $ty) -> Result<(), DiagnosticRenderError> { self.scalar() }
    )*};
}
impl<'a, 'b> ser::Serializer for Scan<'a, 'b> {
    type Ok = ();
    type Error = DiagnosticRenderError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;
    scalars!(
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),
        serialize_f32(f32),
        serialize_f64(f64),
        serialize_char(char)
    );
    fn serialize_str(self, value: &str) -> Result<(), Self::Error> {
        self.budget.node(self.depth)?;
        self.budget.string(value)
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), Self::Error> {
        Err(Self::Error::UnsupportedSerialization)
    }
    fn serialize_none(self) -> Result<(), Self::Error> {
        self.scalar()
    }
    fn serialize_some<T: ?Sized + Serialize>(mut self, value: &T) -> Result<(), Self::Error> {
        self.budget.node(self.depth)?;
        self.child(value)
    }
    fn serialize_unit(self) -> Result<(), Self::Error> {
        self.scalar()
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Self::Error> {
        self.scalar()
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<(), Self::Error> {
        self.serialize_str(variant)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        mut self,
        _: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.budget.node(self.depth)?;
        self.child(value)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        mut self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.budget.node(self.depth)?;
        self.budget.string(variant)?;
        self.child(value)
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self, Self::Error> {
        self.container(len)
    }
    fn serialize_tuple(self, len: usize) -> Result<Self, Self::Error> {
        self.container(Some(len))
    }
    fn serialize_tuple_struct(self, _: &'static str, len: usize) -> Result<Self, Self::Error> {
        self.container(Some(len))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self, Self::Error> {
        self.budget.string(variant)?;
        self.container(Some(len))
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self, Self::Error> {
        self.container(len.and_then(|n| n.checked_mul(2)))
    }
    fn serialize_struct(self, _: &'static str, len: usize) -> Result<Self, Self::Error> {
        self.container(len.checked_mul(2))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self, Self::Error> {
        self.budget.string(variant)?;
        self.container(len.checked_mul(2))
    }
    fn collect_str<T: ?Sized + fmt::Display>(self, _: &T) -> Result<(), Self::Error> {
        Err(Self::Error::UnsupportedSerialization)
    }
}
macro_rules! sequence {
    ($trait:ident, $method:ident) => {
        impl ser::$trait for Scan<'_, '_> {
            type Ok = ();
            type Error = DiagnosticRenderError;
            fn $method<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
                self.child(value)
            }
            fn end(self) -> Result<(), Self::Error> {
                Ok(())
            }
        }
    };
}
sequence!(SerializeSeq, serialize_element);
sequence!(SerializeTuple, serialize_element);
sequence!(SerializeTupleStruct, serialize_field);
sequence!(SerializeTupleVariant, serialize_field);
impl ser::SerializeMap for Scan<'_, '_> {
    type Ok = ();
    type Error = DiagnosticRenderError;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.child(value)
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.child(value)
    }
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}
macro_rules! structure {
    ($trait:ident) => {
        impl ser::$trait for Scan<'_, '_> {
            type Ok = ();
            type Error = DiagnosticRenderError;
            fn serialize_field<T: ?Sized + Serialize>(
                &mut self,
                key: &'static str,
                value: &T,
            ) -> Result<(), Self::Error> {
                self.child(key)?;
                self.child(value)
            }
            fn end(self) -> Result<(), Self::Error> {
                Ok(())
            }
        }
    };
}
structure!(SerializeStruct);
structure!(SerializeStructVariant);

struct Writer<'a, 'b> {
    budget: &'a mut Budget<'b>,
    bytes: Vec<u8>,
    failure: Option<DiagnosticRenderError>,
}
impl Writer<'_, '_> {
    fn append(&mut self, chunk: &[u8]) -> Result<(), DiagnosticRenderError> {
        let next = self
            .bytes
            .len()
            .checked_add(chunk.len())
            .ok_or(DiagnosticRenderError::OutputBytes)?;
        if next > self.budget.limits.max_output_bytes {
            return Err(DiagnosticRenderError::OutputBytes);
        }
        self.budget.charge(chunk.len() as u64)?;
        if next > self.bytes.capacity() {
            let capacity = next
                .max(self.bytes.len().saturating_mul(2))
                .max(64)
                .min(self.budget.limits.max_output_bytes);
            self.budget.charge((capacity + self.bytes.len()) as u64)?;
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(capacity)
                .map_err(|_| DiagnosticRenderError::Allocation)?;
            replacement.extend_from_slice(&self.bytes);
            self.bytes = replacement;
        }
        self.bytes.extend_from_slice(chunk);
        self.budget.usage.output_bytes = next;
        Ok(())
    }
}
impl io::Write for Writer<'_, '_> {
    fn write(&mut self, chunk: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(io::ErrorKind::Other.into());
        }
        match self.append(chunk) {
            Ok(()) => Ok(chunk.len()),
            Err(error) => {
                self.failure = Some(error);
                Err(io::ErrorKind::Other.into())
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn render(
    value: &impl Serialize,
    limits: &DiagnosticLimits,
    cancellation: &CancellationToken,
) -> Result<RenderedDiagnostic, DiagnosticRenderError> {
    limits.validate()?;
    let mut budget = Budget {
        limits: *limits,
        cancellation,
        usage: DiagnosticUsage::default(),
    };
    budget.charge(0)?;
    value.serialize(Scan {
        budget: &mut budget,
        depth: 0,
    })?;
    let mut writer = Writer {
        budget: &mut budget,
        bytes: Vec::new(),
        failure: None,
    };
    if serde_json::to_writer(&mut writer, value).is_err() {
        return Err(writer.failure.unwrap_or(DiagnosticRenderError::Encoding));
    }
    // UTF-8 verification reads the completed bytes once, without copying them.
    writer.budget.charge(writer.bytes.len() as u64)?;
    let document = String::from_utf8(writer.bytes).map_err(|_| DiagnosticRenderError::Encoding)?;
    budget.charge(0)?;
    Ok(RenderedDiagnostic {
        document,
        usage: budget.usage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        authoring::{ExecutionFailure, ExecutionPhase, RequestFailure},
        game::data::live_ron::{LiveRonError, LiveRonSource},
    };
    use serde_json::{json, Value};

    fn content_error() -> AuthoringError {
        let mut diagnostic = LiveRonError::parse(
            "pack/기술.ron",
            "skills",
            Some("actions[0]".into()),
            "bad\n\"effect\"",
        );
        diagnostic.expected = Some(json!({"nested":[1,true,null,"é\n"]}));
        diagnostic.actual = Some(json!([{"key":"value"}]));
        diagnostic.related_sources = vec![LiveRonSource {
            source_file: "other.ron".into(),
            ..Default::default()
        }];
        AuthoringError::Content { diagnostic }
    }

    #[test]
    fn diagnostic_renderer_preserves_all_error_variants_and_original_cancelled_failure() {
        let cancel = CancellationToken::default();
        let application =
            crate::application::ApplicationCommand::from_json_slice(b"{}").unwrap_err();
        for error in [
            content_error(),
            AuthoringError::Request {
                diagnostic: RequestFailure {
                    code: "request",
                    phase: ExecutionPhase::TrialPreparation,
                    field_path: "steps",
                    reason: "bad é\n".into(),
                },
            },
            AuthoringError::Execution {
                diagnostic: ExecutionFailure {
                    code: "authoring_cancelled",
                    phase: ExecutionPhase::Receipt,
                    site: "final_poll",
                    limit: None,
                    requested: None,
                },
            },
            AuthoringError::Application {
                phase: ExecutionPhase::Inspection,
                diagnostic: application,
            },
        ] {
            let original = serde_json::to_value(&error).unwrap();
            let rendered = render_error(&error, &DiagnosticLimits::default(), &cancel).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(rendered.document()).unwrap(),
                original
            );
            assert_eq!(rendered.usage().output_bytes, rendered.document().len());
            let cancelled = CancellationToken::default();
            cancelled.cancel();
            assert_eq!(
                render_error(&error, &DiagnosticLimits::default(), &cancelled).unwrap_err(),
                DiagnosticRenderError::Cancelled
            );
            assert_eq!(serde_json::to_value(&error).unwrap(), original);
            assert_eq!(
                render_error(&error, &DiagnosticLimits::default(), &cancel)
                    .unwrap()
                    .document(),
                rendered.document()
            );
        }
    }

    #[test]
    fn diagnostic_renderer_enforces_exact_source_output_work_and_structural_limits() {
        let cancel = CancellationToken::default();
        let error = content_error();
        let defaults = DiagnosticLimits::default();
        let complete = render_error(&error, &defaults, &cancel).unwrap();
        let mut limits = defaults;
        limits.max_source_bytes = complete.usage().source_bytes;
        assert!(render_error(&error, &limits, &cancel).is_ok());
        limits.max_source_bytes -= 1;
        assert_eq!(
            render_error(&error, &limits, &cancel).unwrap_err(),
            DiagnosticRenderError::SourceBytes
        );
        limits = defaults;
        limits.max_output_bytes = complete.document().len();
        let exact = render_error(&error, &limits, &cancel).unwrap();
        assert_eq!(exact.document(), complete.document());
        limits.max_work = exact.usage().work;
        assert!(render_error(&error, &limits, &cancel).is_ok());
        limits.max_work -= 1;
        assert_eq!(
            render_error(&error, &limits, &cancel).unwrap_err(),
            DiagnosticRenderError::Work
        );
        limits.max_work = defaults.max_work;
        limits.max_output_bytes -= 1;
        assert_eq!(
            render_error(&error, &limits, &cancel).unwrap_err(),
            DiagnosticRenderError::OutputBytes
        );
        limits = defaults;
        limits.max_nodes = complete.usage().nodes;
        assert!(render_error(&error, &limits, &cancel).is_ok());
        limits.max_nodes -= 1;
        assert_eq!(
            render_error(&error, &limits, &cancel).unwrap_err(),
            DiagnosticRenderError::Nodes
        );
        let mut nested = Value::Null;
        for _ in 0..80 {
            nested = json!([nested]);
        }
        let AuthoringError::Content { mut diagnostic } = content_error() else {
            unreachable!()
        };
        diagnostic.expected = Some(nested);
        assert_eq!(
            render_error(&AuthoringError::Content { diagnostic }, &defaults, &cancel).unwrap_err(),
            DiagnosticRenderError::Depth
        );
        let AuthoringError::Content { mut diagnostic } = content_error() else {
            unreachable!()
        };
        diagnostic.related_sources = vec![LiveRonSource::default(); defaults.max_nodes + 1];
        assert_eq!(
            render_error(&AuthoringError::Content { diagnostic }, &defaults, &cancel).unwrap_err(),
            DiagnosticRenderError::Nodes
        );
        limits = defaults;
        limits.max_depth = 65;
        assert_eq!(
            render_error(&error, &limits, &cancel).unwrap_err(),
            DiagnosticRenderError::InvalidLimits
        );
    }

    #[test]
    fn diagnostic_renderer_preflight_never_formats_custom_display() {
        struct Unreviewed;
        impl fmt::Display for Unreviewed {
            fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                panic!("must not format");
            }
        }
        impl Serialize for Unreviewed {
            fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.collect_str(self)
            }
        }
        assert_eq!(
            render(
                &Unreviewed,
                &DiagnosticLimits::default(),
                &CancellationToken::default()
            )
            .unwrap_err(),
            DiagnosticRenderError::UnsupportedSerialization
        );
    }
}
