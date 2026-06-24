# Experiments

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-22 | Goal setup | Defined validation/test harness implementation scope. | Not started. | Start after early subgoals identify touched validation surfaces. |
| 2026-06-22 | Timeline version validation | Removed normal acceptance of timeline/log versions 6/7 and added a focused regression test. | Initial test compile failed because the new test called a non-existent `TimelineValidatorConfig::lenient()` helper. | Use the existing local lenient test config helper and rerun focused validation. |
| 2026-06-22 | Live content audit manifest | Moved live skill delivery/coverage expected lists from Rust arrays to `docs/audit/live_skill_catalog_manifest.ron`; tests now load that versioned manifest and compare live RON against it. | Success. `cargo test --test live_skill_catalog_audit -- --test-threads=1` passed. | Continue timeline validator focused retest. |
| 2026-06-22 | Timeline version validation retry | Re-ran the focused legacy-version validator test after switching to `buff_only_config()`. | Success. Legacy versions 6/7 now fail normal validation with `TimelineVersionMismatch`. Residual grep found no remaining normal acceptance strings. | Run final cross-check and broad validation. |
| 2026-06-22 | Final policy coverage cross-check | Added `POLICY_COVERAGE.md` mapping every confirmed policy heading to implementation/validation evidence, then ran broad validation. | Success. No undocumented drift or new policy-decision report was found. | Complete subgoal and master goal. |

## Failed Approaches

- Timeline validator focused test initially failed to compile because `TimelineValidatorConfig::lenient()` does not exist. Fixed the test to use the existing `buff_only_config()` helper.

## Validation Commands

- `cargo test --test live_skill_catalog_audit -- --test-threads=1` - passed, 3 tests.
- `cargo test --lib game::battle::validation::validator::tests::normal_validation_rejects_legacy_timeline_versions -- --test-threads=1` - failed to compile before test helper correction.
- `cargo test --lib game::battle::validation::validator::tests::normal_validation_rejects_legacy_timeline_versions -- --test-threads=1` - passed after helper correction.
- `rg -n "supported=\\{6, 7|timeline\\.version != TIMELINE_VERSION &&|version != 6|version != 7|version 6|version 7" src tests docs -S` - no matches after legacy acceptance removal.
- `rg -n "expected_spatial_skills|expected_intentional_instant_skills|abnormality_skill_coverage|item_and_artifact_skill_coverage|skill_fragment_skill_coverage|set_of\\(" tests/live_skill_catalog_audit.rs -S` - no matches after manifest migration.
- `cargo test --lib -- --test-threads=1` - passed, 515 tests.
- `cargo test --test ron_loading -- --test-threads=1` - passed, 18 tests.
- `cargo check -p game_server` - passed.
