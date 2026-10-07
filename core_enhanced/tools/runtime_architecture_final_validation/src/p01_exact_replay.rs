// Compile the immutable accepted P01 observer verbatim against the final Core.
// The included source owns `main`; this wrapper intentionally adds no workload
// branch, input normalization, timing allowance, or output transformation.
include!("../../runtime_architecture_preservation/src/main.rs");
