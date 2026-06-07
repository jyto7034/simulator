# Post Policy Technical Debt Review Notes

## Initial Notes

- This is an audit goal; findings should not be silently fixed in this goal.
- The prior master goal intentionally completed many policy implementations quickly. The highest-risk review areas are source-of-truth drift, snapshot/runtime mismatch, validation cost, Rapier boundary regression, and tests that only prove implementation shape.
- External Unity docs in `/mnt/f/unity projects/ark/docs` are canonical when Unity-facing contracts are involved.

## Source Of Truth Notes

- Runtime code remains the strongest source of truth for this audit. Several docs describe intended lifecycle states, but some of those states are not yet produced by runtime code.
- `GameDataBuilder::empty()` now uses `BuffDatabase::new(vec![])`, and `GameDataBuilder::live_defaults()` explicitly opts into `BuffDatabase::live_default()`. The earlier hidden-live-default concern is resolved for builder semantics.
- `BuffDatabase::live_default()` still uses `include_str!`, but it is now an explicit helper rather than hidden behind `empty()`. This should be kept as a conscious live-default convenience or eventually replaced by an external loader as part of a data-loading cleanup, not treated as an active medium bug.
- External Unity docs previously described `ThreatWarningStatus::Observed` and `ThreatWarningSource::Observed`, but the current policy uses only hardcoded false-rumor `Disproved`; the active contract should remove `Observed`.
- External Unity docs say Unity should trust snapshots after command payloads. That makes snapshot error swallowing especially risky when effective combat profile computation fails.

## Runtime And Snapshot Notes

- `effective_combat_profile_for_employee` is the shared runtime helper for actual battle drafts and snapshot effective combat profile calculation. This is a good source-of-truth direction.
- `src/game/world/snapshot.rs` currently calls that helper and then uses `.ok()`, turning static/profile errors into `null` effective fields. This can hide a broken weapon/profile/fragment path from Unity while deployment later fails.
- `src/game/world/helpers.rs` has a projected item slot compatibility path that manually applies weapon profile data to a copied profile. It is reasonable because it evaluates a hypothetical loadout, but it duplicates the effective-profile construction idea. A pure projection helper would reduce drift.
- `../game_server/src/game/player_game_actor/state.rs` still maps many `BehaviorResult` variants into stringly `json!` payloads. Some newer payloads are typed, but battle/node/result payload shape is still not compiler-protected.

## Validation Notes

- `GameDataBase::new` runs structural index/cross-reference validation and then calls `validate_combat_preview_threat_warning_contract`.
- That validation generates actual `CombatPreview`s for each PVE encounter and five seeds. This catches warning omissions, but it also makes data construction depend on runtime preview generation behavior and seed policy.
- Better long-term boundary: static constructor validates static references and schema invariants; preview generation consistency should live in an explicit live-data audit/test or a named validation phase.

## Movement And Rapier Notes

- Rapier backend is mostly aligned with the new policy:
  - `MovementTerrainPolicy::Airborne` skips static obstacle correction.
  - unit colliders are excluded from `corrected_ground_static_obstacle_translation_for`.
  - board wall colliders are test-only helpers and ignored by runtime correction.
  - focused tests assert no unit depenetration and no same-lane steering around units.
- `steering.rs` still contains separation, side-bias, and congestion code, but defaults are zeroed. This is not currently a behavior bug. It is a cleanup/debt candidate because the dead parameters can invite future accidental reactivation of old anti-overlap behavior.

## Live Data Notes

- Current code and server load equipment from `../game_resources/data/equipments/base.ron`.
- `../game_resources/data/equipments.ron` still exists and appears to be a stale/parallel equipment database with overlapping IDs. It is not referenced by current load paths found in `src`, `tests`, or `../game_server/src`.
- This is a low source-of-truth drift risk: future data edits may update the wrong file.

## Test Debt Notes

- No `#[ignore]` tests were found in `src`, `tests`, or `../game_server/src`.
- Existing goal notes record several test-filter mistakes:
  - `cargo test -p game_core damage_feedback -- --nocapture` ran 0 tests before being corrected.
  - `cargo test -p game_core ron_loading -- --nocapture` and similar integration-test-name filters ran 0 tests before `--test ron_loading` was used.
  - `cargo test -p game_core blocking -- --nocapture` passed while matching only 1-2 tests in some goals.
- This audit should not duplicate `post_policy_test_verification_goal.md`, but the report should call out that successful focused filters are not always broad evidence.
