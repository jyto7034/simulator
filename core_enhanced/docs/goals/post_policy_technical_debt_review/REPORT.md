# Post Policy Technical Debt Review Report

## Executive Summary

2026-06 core policy work moved several systems in the right direction: builder `empty()` is no longer secretly live-backed, damage feedback is core-owned, effective combat profile has a shared runtime helper, and Rapier no longer acts as the source of truth for unit collision/blocking.

The main remaining debt is not a single broken runtime path, but boundary drift:

- Some Unity-facing DTO states are documented before runtime lifecycle support exists.
- Some command/snapshot payloads are still assembled with manual JSON strings.
- Some validation that belongs in explicit live-data audit is running inside `GameDataBase::new`.
- Snapshot can hide effective-profile errors that deployment/runtime would treat as real errors.
- Combat preview authoring has a few rough edges: authored route ids can drift from selected battlefield templates, rumor warnings include future-only tags, and generated wave briefing can under-report actual generated enemies.

Post-audit policy discussion resolved several items:

- False threat rumors should stay simple and seed-driven. Use a 20% chance, at most one rumor, selected from warning tags that are not present in the actual resolved spawn waves.
- `Disproved` should be emitted for false rumors on re-entry after retreat. There is no planned observation system; `Observed` should be removed from the active contract instead of reserved.
- Roster snapshot effective profile failures should be surfaced as an `effective_profile_error` style field rather than silently dropped or making the whole snapshot fail.
- Stale resource files should be audited with a deletion-first bias, but external resource deletion still requires confirming no tool depends on them.

No Critical or High issues were found in this audit. The Medium items should be split into follow-up goals rather than fixed opportunistically inside this review.

## Critical Findings

None.

## High Findings

None.

## Medium Findings

### M1. Preview Warning Validation Runs Runtime Generation Inside `GameDataBase::new`

- Severity: Medium
- Area: validation placement and cost
- Files:
  - `src/game/data/mod.rs:1022`
  - `src/game/data/mod.rs:1143`
- Problem: `GameDataBase::new` constructs the database and immediately runs `validate_combat_preview_threat_warning_contract`, which generates combat previews for every PVE encounter across five seeds.
- Evidence: `validate_combat_preview_threat_warning_contract` calls `CombatPreview::generate_for_node` inside nested encounter/seed loops, then `GameDataBase::new` calls that function before returning the database.
- User-visible Risk: static data construction now depends on runtime preview generation policy. A preview-generation refactor, seed policy change, or expensive encounter set can make ordinary data loading fail or slow down even when static references are valid.
- Recommended Direction: move this into an explicit live-data validation/audit path, such as a named `GameDataBase::validate_generated_contracts()` or dedicated live RON test. Keep `GameDataBase::new` focused on structural schema/index/cross-reference validation.
- Needs User Policy Decision: No. This is an engineering-boundary cleanup.

### M2. Threat Warning DTO Still Exposes Unused `Observed` State

- Severity: Medium
- Area: Unity-facing contract/runtime lifecycle alignment
- Files:
  - `src/game/combat_preview.rs:181`
  - `src/game/combat_preview.rs:188`
  - `src/game/combat_preview.rs:1615`
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`
- Problem: `ThreatWarningStatus` and `ThreatWarningSource` still include `Observed`, but the agreed policy does not include an observation system. The only lifecycle needed is hardcoded false-rumor `Disproved` on re-entry after retreat.
- Evidence: runtime warning generation sets `status: ThreatWarningStatus::Unverified` and `source: ThreatWarningSource::Briefing` or `Rumor`; no production code found that produces `Observed` or source `Observed`.
- User-visible Risk: Unity or future core code may build around a non-existent observation system. This makes the preview contract look broader than the intended hardcoded rumor/disproof policy.
- Recommended Direction: implement the simple false-rumor lifecycle that was agreed after the audit:
  - keep preview warning generation based on resolved `spawn_waves`;
  - set rumor chance to 20%;
  - choose at most one rumor from warning tags that are absent from the actual resolved spawn waves;
  - after the player enters and retreats, emit that false rumor as `Disproved` in the next preview for the same abnormality attempt;
  - keep real briefing warnings `Unverified` for now;
  - remove `Observed` from the active DTO/Unity contract instead of reserving it.
- Needs User Policy Decision: No.

### M3. Server `BehaviorResult` Payload Mapping Remains Stringly Typed

- Severity: Medium
- Area: Unity-facing command DTO drift
- Files:
  - `../game_server/src/game/player_game_actor/state.rs:180`
  - `../game_server/src/game/player_game_actor/state.rs:532`
- Problem: `BehaviorResult` is typed in core, but the server boundary still maps many variants to payloads with manual `json!` objects and string field names.
- Evidence: `behavior_result_payload` manually constructs `NodePreview`, `BattleAdvanced`, `BattleState`, `BattleUnitDeployed`, `BattleUnitWithdrawn`, `BattleSkillActivated`, and many other payloads with string keys.
- User-visible Risk: field renames or missing fields can compile successfully but break Unity. This is especially risky now that battle payloads carry new policy fields such as combat preview warnings, deployment state, timeline deltas, and mobility/feedback data.
- Recommended Direction: introduce typed serializable payload structs for command results, ideally near the `BehaviorResult` variants or in a core-owned DTO module. The server should convert variants to typed payloads and then serialize, instead of assembling ad-hoc JSON.
- Needs User Policy Decision: No for engineering direction. Yes only if payload shape changes are intentionally bundled.

### M4. Snapshot Effective Combat Profile Silently Drops Errors

- Severity: Medium
- Area: runtime/snapshot alignment
- Files:
  - `src/game/world/snapshot.rs:435`
  - `src/game/combat_player_spawns.rs:82`
- Problem: roster snapshot uses the shared effective-profile helper, but converts any error to `None` with `.ok()`.
- Evidence: `snapshot.rs` calls `effective_combat_profile_for_employee(...).ok()` and then serializes effective fields from the optional profile.
- User-visible Risk: Unity can receive `null` effective weapon/profile fields instead of a clear invalid-data state, while actual deployment may later fail through the same profile path. This can hide live data/schema problems and make the loadout UI misleading.
- Recommended Direction: make snapshot generation surface profile errors explicitly. Options include returning `GameError`, adding an `effective_profile_error` DTO field, or producing a typed invalid-profile report. Avoid silent `None` unless the profile is legitimately absent by policy.
- Needs User Policy Decision: Yes if the Unity-visible error shape changes. The engineering problem is clear, but the DTO shape needs agreement.

### M5. Authored Wave Route Ids Can Drift From Selected Battlefield Templates

- Severity: Medium
- Area: combat preview authoring contract
- Files:
  - `src/game/combat_preview.rs:1348`
  - `src/game/combat_preview.rs:2012`
  - `src/game/events/combat.rs:925`
  - `../game_resources/data/pve/encounters.ron`
- Problem: `PveWaveData.route_id` is copied directly into `SpawnWave.route_id`, but the selected battlefield template may not contain that route id unless the encounter also pins a compatible battlefield/template.
- Evidence: `spawn_waves_for` uses `wave.route_id.clone().or_else(|| fallback_route_id.clone())`; `validate_instance` rejects missing route ids; the current failing test `authored_protect_unit_tactical_plan_overrides_default_defense_contract` hardcodes `black_box_breach_main` while using generated battlefield selection.
- User-visible Risk: data authors can write a plausible route id in RON or fixtures and get preview generation failures depending on selected archetype/size/seed. This is especially brittle for Defense encounters.
- Recommended Direction: make the authoring contract explicit:
  - default encounters should omit `route_id` and let preview generation choose the fallback route;
  - encounters that must reference a named route should also pin a battlefield/template that owns that route;
  - tests should not weaken route validation to pass stale fixtures.
- Needs User Policy Decision: No. This is an authoring-contract cleanup.

## Low Findings

### L1. Projected Skill Fragment Compatibility Duplicates Effective Profile Construction

- Severity: Low
- Area: source-of-truth drift
- Files:
  - `src/game/world/helpers.rs:406`
  - `src/game/combat_player_spawns.rs:82`
- Problem: actual runtime/snapshot effective profile uses `effective_combat_profile_for_employee`, but projected equipment validation manually builds a profile and applies weapon profiles.
- Evidence: `validate_active_skill_fragment_for_projected_item_slot` reconstructs profile logic for hypothetical equipment slots.
- User-visible Risk: low today, because projected loadout needs a custom path. Long term, new weapon/profile modifiers can drift between actual profile and projected validation.
- Recommended Direction: extract a pure helper for "effective combat profile with projected item slot" and have both projected validation and any future preview UI use it.
- Needs User Policy Decision: No.

### L2. Stale Parallel Equipment Data File Remains Beside Active Source

- Severity: Low
- Area: live RON source-of-truth drift
- Files:
  - `../game_resources/data/equipments.ron`
  - `../game_resources/data/equipments/base.ron`
  - `../game_server/src/main.rs:169`
  - `tests/common/mod.rs:365`
- Problem: active code/server/test load paths use `equipments/base.ron`, but `equipments.ron` still exists with overlapping equipment data.
- Evidence: `rg` found active include paths for `equipments/base.ron`; no active code path found for `equipments.ron`.
- User-visible Risk: data authors may update the stale file and see no effect in live runtime, or future tools may accidentally pick the wrong source.
- Recommended Direction: remove or archive stale top-level `equipments.ron` after confirming no external tool still reads it. If external tools need it, document it as generated/exported, not authoritative.
- Needs User Policy Decision: Yes if deleting/replacing external resource files affects authoring workflow.

### L3. Disabled Steering Parameters Leave Old Anti-Overlap Logic In Place

- Severity: Low
- Area: movement/backend cleanup
- Files:
  - `src/game/battle/core/movement/steering.rs:8`
  - `src/game/battle/core/movement/engine.rs:276`
  - `src/game/battle/core/movement/rapier_backend.rs:561`
- Problem: separation, side-bias, and congestion code still exists, but defaults are set to zero/neutral values.
- Evidence: `SteeringParams::default()` sets separation and congestion multipliers to 0 and min speed scale to 1; both Direct and Rapier pass `SteeringParams::default()`.
- User-visible Risk: no current behavior bug found. The risk is future accidental reactivation of unit avoidance that would conflict with the confirmed overlap policy.
- Recommended Direction: either remove the dead anti-overlap steering branch or make it an explicitly named experimental/test-only helper outside the live backend path.
- Needs User Policy Decision: No, unless user wants future soft-avoidance visuals in core.

### L4. Focused Test Filters Have Repeatedly Given Weak Evidence

- Severity: Low
- Area: test debt
- Files:
  - `docs/goals/damage_feedback_dto/EXPERIMENTS.md`
  - `docs/goals/combat_preview_threat_warnings/EXPERIMENTS.md`
  - `docs/goals/buff_database_and_consumable_modifier/EXPERIMENT_NOTES.md`
  - `docs/goals/core_policy_implementation_master/EXPERIMENTS.md`
- Problem: several previous goal records show test commands that passed while running 0 tests, or broad-sounding filters that matched only 1-2 tests.
- Evidence: goal notes record `damage_feedback` and `ron_loading` filter mistakes, plus narrow `blocking` filter coverage.
- User-visible Risk: a green focused command can be mistaken for meaningful coverage. This is mostly process debt and belongs in `post_policy_test_verification_goal.md`.
- Recommended Direction: test-verification goal should record actual test counts for every verification command and prefer `--test <integration_test>` for integration suites.
- Needs User Policy Decision: No.

### L5. Rumor Warning Candidates Include Future-Only Tags

- Severity: Low
- Area: combat preview warning clarity
- Files:
  - `src/game/combat_preview.rs:1672`
  - `src/game/combat_preview.rs:1686`
- Problem: `all_threat_warning_tags()` includes tags that do not currently have real runtime detection, such as hard-to-block, shielded, and regenerating. Because rumors are selected from all tags absent from the real set, these future-only tags can appear as false rumors before their underlying enemy traits exist.
- Evidence: real warning extraction currently detects high defense, high magic resist, fast speed, and airborne. Future-only tags are only available as rumor candidates.
- User-visible Risk: preview text can imply mechanics that do not exist yet. As a rare rumor this is not a runtime bug, but it can teach the player to expect nonexistent traits.
- Recommended Direction: for the 20% false-rumor policy, limit rumor candidates to warning tags with current detection support:
  - armored enemy possible;
  - high magic resist enemy possible;
  - air enemy possible;
  - fast breakthrough enemy possible.
  Add hard-to-block, shielded, and regenerating back only when the corresponding enemy traits/runtime detection exist.
- Needs User Policy Decision: No. User already preferred the simple hardcoded false-warning approach.

### L6. Enemy Briefing Count Hint Can Drift From Generated Wave Source

- Severity: Low
- Area: preview accuracy
- Files:
  - `src/game/combat_preview.rs:1754`
  - `src/game/combat_preview.rs:1431`
  - `src/game/data/pve_data.rs:36`
- Problem: `enemy_briefing` computes `count_hint` from authored `wave.enemies`, while `source: GeneratedCorroded` waves resolve their actual enemy entries later through `resolve_wave_enemy_data`.
- Evidence: warning generation uses resolved `spawn_waves`, but `enemy_briefing` sums the legacy/authored `wave.enemies` collection directly.
- User-visible Risk: generated corroded waves can show a weak or misleading count hint even though the actual battle spawns a different number of enemies.
- Recommended Direction: compute briefing count hints from the same resolved spawn wave entries used by warnings and battle scenario construction.
- Needs User Policy Decision: No.

## Follow-up Candidates

### 바로 고칠 항목

No runtime code should be changed inside this audit. The best immediate fixes are documentation/process only:

- Keep `REPORT.md` as the handoff for the next implementation goals.
- Make future goal reports include actual test counts when using cargo filters.

### 별도 goal로 분리할 항목

- `combat_preview_validation_boundary_goal`: move generated preview warning validation out of `GameDataBase::new` into explicit live-data validation.
- `server_behavior_result_dto_typing_goal`: replace manual server `json!` payload assembly with typed command result DTOs.
- `snapshot_effective_profile_error_surface_goal`: stop silently dropping effective profile errors in roster snapshot.
- `combat_preview_route_authoring_contract_goal`: keep route validation strict, but make route authoring/fallback rules explicit and fix stale route fixtures.
- `threat_warning_false_rumor_lifecycle_goal`: implement 20% false rumor, re-entry `Disproved`, and remove unused `Observed` contract surface.
- `effective_profile_projection_helper_goal`: unify projected item-slot profile computation with runtime/snapshot effective profile helpers.
- `combat_preview_warning_candidate_cleanup_goal`: limit false-rumor candidates to currently detectable warning tags.
- `enemy_briefing_resolved_wave_count_goal`: derive count hints from resolved spawn waves.
- `legacy_resource_file_cleanup_goal`: remove/archive stale top-level resource files such as `equipments.ron`.

### 유보할 항목

- Rapier/backend boundary: no active medium bug found. Current tests and code indicate policy alignment. Keep watching `sync_board_bounds` and steering cleanup, but do not block other work.
- `BuffDatabase::live_default()`: still `include_str!`, but builder `empty()` is no longer hidden-live-backed. Treat as explicit loader cleanup, not an urgent bug.
- Future-only warning tags such as hard-to-block/shielded/regenerating: do not use as false-rumor candidates until the corresponding enemy traits are implemented.

### 정책 논의 필요 항목

- Snapshot effective profile failures should use an `effective_profile_error` style field unless later DTO design finds a better typed equivalent.
- Whether stale data files outside active include paths can be removed now, or whether external authoring tools still depend on them.

## Policy Questions For User

1. `combat_profile.effective_profile_error`의 정확한 DTO shape는 어떻게 할까요?
   - 추천: `{ code: string, message: string }`
2. `../game_resources/data/equipments.ron` 같은 stale top-level resource file은 외부 도구 참조 audit 후 제거해도 될까요?

## Suggested Next Goals

Recommended order:

1. `combat_preview_route_authoring_contract_goal`
2. `threat_warning_false_rumor_lifecycle_goal`
3. `snapshot_effective_profile_error_surface_goal`
4. `server_behavior_result_dto_typing_goal`
5. `combat_preview_validation_boundary_goal`
6. `combat_preview_warning_candidate_cleanup_goal`
7. `enemy_briefing_resolved_wave_count_goal`
8. `legacy_resource_file_cleanup_goal`
9. `effective_profile_projection_helper_goal`

## Verification Commands Run

This audit did not run gameplay tests because it did not change runtime/core/server code.

Read/check commands used as evidence:

- `git status --short`
- `rg` searches over runtime, tests, server, resources, goal notes, and external Unity docs.
- `sed`/`nl` reads of the files listed in each finding.
- `rg -n "#\\[ignore\\]" src tests ../game_server/src`
- `git diff --check -- docs/goals/post_policy_technical_debt_review/PLAN.md docs/goals/post_policy_technical_debt_review/EXPERIMENTS.md docs/goals/post_policy_technical_debt_review/EXPERIMENT_NOTES.md docs/goals/post_policy_technical_debt_review/REPORT.md`
