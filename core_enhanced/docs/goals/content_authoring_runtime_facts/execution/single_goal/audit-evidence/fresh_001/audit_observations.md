# Audit observations

Read-only fresh same-model reviewer `/root/r04_runtime_facts_fresh_audit`; no implementation conversation fork or participation. No new gameplay/Cargo/build executions. Writes restricted to this directory and the final report.

The first evidence reconstruction used `before/<path>` for newly added files; the immutable original patch uses `/dev/null`. All 1,129 other checks passed. After reading the actual patch and owner reconstruction format, the auditor corrected only this audit-script assumption. The initial result is preserved in evidence_review_initial.json; the subsequent result independently reconstructs the exact patch. This was no source/evidence failure or changed implementation oracle.

A first shell metadata read used unavailable `python`; it executed no Python work and was retried with installed `python3`. A read-only search initially used `src/bin/final_performance.rs`; the actual named file is `src/final_performance.rs`, subsequently read directly.

Frozen ACCEPTANCE_EVIDENCE_001.md E01[0] says `skill_runtime_contract25`; the actual full164 log has 15 tests in that integration binary. The map's aggregate 1,361 is correct (1,309 + 4 + 4 + 25 + 15 + 4). This clerical observation will be classified after complete coverage review, without modifying frozen owner evidence.
