//! Node-graph acquisition and source-preserving normalization.
//!
//! A normalized document is not execution authority. Semantic lowering and the
//! production data loader must approve it before it can become a runtime plan.

mod gaps;
mod lower;
mod program;
mod service;
pub use lower::compile;
pub use program::{NativeGraphContent, NativeGraphCatalogs, GraphOperation, PortableGraph, IR_VERSION, NATIVE_TARGET};
pub use service::{compile_request, CompileOutcome, CompileRequest, CompileResponse};
mod schema;
mod scope;
mod structure;
pub use gaps::authoring_gaps;
pub use scope::resonance_scope_diagnostics;

pub use schema::{
    Diagnostic, Document, Edge, Endpoint, FieldValue, Location, Node, NormalizedDocument,
    NormalizedNode, Origin,
};
pub use structure::read_and_normalize;

pub const SEMANTIC_VERSION: &str = "2026-10-22.2";

#[cfg(test)]
pub(crate) mod tests;
