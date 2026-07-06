# PVE Encounter Classification And Floor Scaling Contract Notes

## User Decisions Captured

- Current RON has legacy structure from earlier Bazaar/TFT-like designs and should be reshaped for the current facility-exploration plus DefenseRoute flow.
- `PveEncounter` should become a clear encounter scenario, not a difficulty bucket.
- Remove `PveEncounter.difficulty`.
- Do not replace difficulty with another numeric encounter-authored difficulty field.
- Add explicit encounter classification:
  - `Normal`;
  - `Elite`;
  - `NormalBoss`;
  - `FinalBoss`.
- `Normal` means ordinary combat with only corroded employees and may have no primary abnormality.
- `Elite` means corroded employees plus an elite abnormality monster and requires a primary abnormality.
- `NormalBoss` means corroded employees plus a non-final boss abnormality monster and requires a primary abnormality.
- `FinalBoss` means corroded employees plus a final boss abnormality monster and requires a primary abnormality.
- `primary_abnormality_id = None` is valid for `Normal` only and means no abnormality response research, no response completion, and no abnormality fragment reward.
- `Normal` encounters must not author abnormality suppression research or abnormality-targeted bonus objectives. Extra rewards for pure corroded-employee battles are ordinary encounter rewards.
- `Elite`, `NormalBoss`, and `FinalBoss` use exactly one primary abnormality target in the current implementation. Multi-primary encounters are deferred.
- Separate "what appears" from "how strong it is":
  - encounter classification and waves decide what appears;
  - current Floor and scaling policy decide strength.
- Floor scaling policy should live in RON.
- First live Floor scaling table is 2-Floor stages, four stages total:
  - Floor 1-2;
  - Floor 3-4;
  - Floor 5-6;
  - Floor 7-8.
- Floor 9+ clamps to the final stage.
- Authoring uses human-facing Floor numbers starting at 1; runtime `floor_index` can remain zero-based.
- Floor stat scaling initially applies only:
  - max health;
  - attack;
  - defense.
- Do not scale magic resistance, movement speed, attack interval, windup, resonance, skill cooldown, deployment cost, or rewards in this goal.
- Generated corroded waves can receive budget scaling.
- Generated wave budget scaling uses ceiling and minimum 1.
- Manual waves do not receive count/budget scaling.
- Manual wave units still receive stat scaling.
- All waves are not automatically repeated.
- Extra waves are explicit and stage-authored.
- Boss gimmick/phase/objective waves are not implicitly cloned or repeated.

## Known Runtime Evidence Before Implementation

- `PveEncounter` currently has `id`, `abnormality_id`, `difficulty`, `risk_level`, `node_type`, `mission_variant`, `survive_timer_ms`, rewards, suppression research, battlefield, tactical plan, win condition, waves, and static obstacles.
- Current `pve/encounters.ron` uses `difficulty` and often uses `abnormality_id` even when actual waves spawn only corroded employees.
- `map_encounters.rs` currently sorts and filters candidates by `PveEncounter.difficulty`.
- `RunProgression::difficulty_floor_index()` currently exists to support old difficulty selection.
- Live PVE encounter difficulty values currently span `1..=7`, so unbounded Endless Floors can exceed the authored difficulty range if selection remains difficulty-window based.
- Current map encounter assignment has silent fallback behavior when difficulty-compatible candidates are empty. This is a policy bug: role-compatible pools should fail loudly or be reported as blockers instead of falling back to all encounters.
- `CorrodedWavePreset.difficulty` exists, but previous audit found generated corroded wave resolution uses preset id, budget, count range, role mix, and seed salt rather than this field as the active runtime selection source.
- Live RON loading currently passes before this goal begins, but the structure is policy-stale rather than syntactically broken.

## Implementation Guidance

- Prefer explicit schema fields and validation failures over inference from naming conventions.
- Avoid deriving encounter class from `id` prefixes such as `suppress_`.
- Avoid deriving encounter class from wave contents alone; waves describe spawned units, not node/encounter role.
- Avoid deriving final boss from "highest difficulty" or "highest risk".
- If `node_type` remains necessary, keep it as mission/runtime mode information, not encounter class.
- If `primary_abnormality_id` rename is too broad, stop and ask before keeping `abnormality_id` as a transition field.
- If preview generation and battle construction diverge, prefer a shared resolver instead of duplicating scaling logic.

## 2026-06-27 Startup Audit Notes

The first implementation audit found that the `primary_abnormality_id` policy is the first hard decision point.

Current runtime shape:

- `PveEncounter` requires `abnormality_id: String`.
- `node_flow.rs` reads `encounter.abnormality_id` before starting combat.
- `ActiveBattleSession` stores `abnormality_id: String`.
- `CombatBattleState` stores `abnormality_id: String`.
- `selected_event.type == "combat_battle"` exposes `abnormality_id`.
- Battle record JSON exports `abnormality_id`.
- Endless research reward application uses the finished battle's `abnormality_id`.

Current live data shape:

- Every live PVE encounter has an `abnormality_id`.
- The current live encounter list is suppression-themed.
- Normal combat nodes currently depend on the old difficulty-window selector to reuse suppression encounters.
- There is no explicit generic `Normal` encounter pool with `primary_abnormality_id = None`.

This meant Phase 4A could not be implemented as a purely mechanical field rename without a policy choice:

- true `Normal` encounters require no-primary battle/result/research behavior;
- keeping mandatory abnormality identity on all encounters conflicts with the planned `Normal` semantics;
- classifying existing suppression encounters as `Normal` would silently change content meaning and research identity.

Resolved long-term direction:

- Add true generic `Normal` corroded-employee encounter entries for ordinary combat nodes.
- Rename encounter identity to `primary_abnormality_id: Option<String>`.
- Only `Elite`, `NormalBoss`, and `FinalBoss` encounters may drive abnormality response research.
- Normal battle result/record surfaces should expose no primary abnormality instead of inventing a placeholder.
- Update Unity-facing/result docs and probes if this changes public JSON shape.

## Follow-Up Candidates

- Rename or remove `CorrodedWavePreset.difficulty`; possible replacement names include `pressure_tier`, `wave_intensity`, or `budget_tier`.
- Add reward scaling after combat pacing is stable.
- Add encounter-specific scaling overrides if global stages are too coarse.
- Add boss omen chain integration after base classification and scaling are stable.
- Add authored facility-map templates beyond the current generated depth/lane template.
- Add content authoring tooling to preview Floor stage results.
