# Audit-local verification history

The first delta reconstruction assertion failed because the auditor used `a/path` for absent-baseline file headers, while the retained unified patch uses `/dev/null`. Direct comparison showed header-only differences. The verifier was corrected to the actual format and rerun; this was an audit-tool formatting error, not a source or validation failure. No implementation evidence was changed.

The second verification stopped at the initial overly broad expectation that INPUT115 had no changed member. Direct enumeration showed only the documented standalone `final_performance.rs` adaptation, just as for INPUT114. The verifier now records that exact difference for both checkpoints; public guards and core input reuse exclude the changed kernel harness.

Supplemental reconstruction initially expected an extra blank line after the feature declaration. Exact bytes showed one terminal newline; the removed segment now matches its recorded hash, and reconstructs the old manifest exactly. No source finding resulted.
