# Cross-Layer Phase 04 Persistence Completion Review Experiments

Goal: R04

Updated: 2026-07-14

## Review inputs

- Read the completion review guide and every Phase 4 Child completion review.
- Re-read Phase 4 plan/guardrails and inspected current state, checkpoint, combat completion,
  compendium projection/persistence, research delivery, server actor/pass-through, and Unity reader
  seams over Child summaries.
- Current post-R04-document dirty fingerprints: Core view
  `38e36d105ef98ce13906acbe28affc95711810aa840f7e21ab89390f97633d91`, server view
  `a564aa827f7346c1b58930df2acc509e9137cdd4b341a93790f08a5737b3f0df`, Unity
  `cd6eeed715cde88a4f1e429accbff76ccf3a2d8a75d31f3c0c712ecb2b7db28a`.
- Tracked diff fingerprints: Core/server repository
  `68a951cfc8e1458e6e6a853306eac897f0e3b6dd4b72febd03bdcc517fdf1135`, Unity
  `705344cba78f3aef476002656a2c4c2f7e80125bbbac117bff642c780d0c6c96`.

## Independent focused reruns

- G17 checkpoint/global-VN identity: 1/1.
- G23 explicit AliveRoster XP distribution/result diff: 1/1.
- G16 forced combat-trace export failure remains non-fatal: 1/1.
- G29 compendium source/schema/profile group: 3/3.
- G21 research group: 19/19.
- game_server monotonic actor recreation: 1/1; planning-return result/trailing-snapshot ordering: 1/1.
- Unity combined strict compendium + research-delivery contracts: 9/9.

## Broad evidence reused on unchanged production fingerprints

- Core all-targets: 746 passed, 0 failed, 0 ignored; all-target check and formatting passed.
- game_server all-targets: 30 passed, 0 failed, 0 ignored; all-target check and formatting passed.
- Unity EditMode: 211/211. PlayMode: 52 passed, 0 failed, four explicit live-server environment
  skips. No Phase 4 test is ignored/skipped.
- Rust-generated research fixture exactly matched Unity. G29's Rust-generated compendium fixture
  equality and strict reader remained covered by the current 9-test combined rerun.

## Integrated audits

- Current production/canonical search for `battle_records`, `BattleRecord`, category delivery hook /
  policy names, and serde aliases found only two intentional negative references: the strict
  compendium legacy-field rejection fixture and the runtime contract statement that the removed
  delivery policy does not exist.
- Root `GameCoreState` alone owns `encounter_compendium`; RunState/checkpoint/battle-entry structures
  have no replacement copy. Server persistence merges by union, and Unity reads the required root
  projection strictly.
- Battle gameplay cast/delivery identity uses `skill_cast_seq` and domain occurrence counters;
  `parent_seq` remains observation/cancellation causality only.
- Root snapshot exposes pending research as state, while only typed planning completions expose
  delivery facts. No server/Unity layer derives one from the other.
- Core repository diff check and scoped G21 Unity diff check passed. Full Unity diff check remains
  noisy only from unrelated pre-existing CRLF/whitespace files recorded by G21.
- No live RON was modified in G21/R04, and no dirty worktree was reset, cleaned, restored, stashed,
  or baseline-committed.
