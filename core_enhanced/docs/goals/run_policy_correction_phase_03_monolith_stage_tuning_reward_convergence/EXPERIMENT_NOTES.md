# Monolith Stage Tuning And Reward Convergence Experiment Notes

Status: complete notes

Implementation complete: true

Created: 2026-07-28

## 1. Dependency

- Master P03; direct prerequisites P00-P02 correction `Complete / High`.
- P04 independently reaudits this phase and all predecessors before Master completion.

## 2. Approved Boundary

- Typed stage 2/3 is difficulty authority and must match monolith ledger.
- Stage affects target/stat/reward, not wave budget.
- Stat and reward combine role+stage once before applying/rounding once.
- Unity consumes qualitative reward tier, never exact internal percentage.
- Strict P02 save schema preserves stage; no migration or missing-field fallback.

## 3. Confirmed Pre-Change Defects

- Conversion scales only `target_power`; preview/setup use the same role stat tuning for stage 2 and 3.
- Reward applies role and stage in separate rounding passes (`1 -> 3 -> 4`) instead of the approved
  one-pass composite (`1 * 313% -> 3`).
- DTO, Unity reader and presenter expose exact `reward_multiplier_percent`.
- Difficulty has no typed stage, so ledger and combat difficulty cannot be cross-validated directly.

## 4. Dirty Worktree

- Core/Unity target files overlap broad existing campaign edits and several are untracked.
- Read current file/diff first; modify only stage authority, resolver, DTO consumer and direct tests.

## 5. Policy Questions

현재 없음. New stage labels, wave changes, reward pool changes or save compatibility needs require
`policy_decision_required`.

## 6. Out-Of-Scope Candidates

없음. P04 record cleanup/final review를 이 child에서 선행하지 않는다.

## 7. Confirmed Final Contract

- Difficulty의 required typed `monolith_stage`와 Monolith ledger가 단일 stage authority다.
- Role+stage resolver의 최종값은 Stage2 `220/165/145/145/200/250`, Stage3
  `275/207/182/182/200/313` 순서(target/HP/attack/defense/wave/reward)다.
- Stage는 target/stat/reward에만 영향을 주고 generated-wave budget에는 영향을 주지 않는다.
- Reward scaling은 original quantity에 composite percent를 한 번만 nearest/minimum-one 적용한다.
- Wire/UI reward 정보는 qualitative tier뿐이며 legacy exact percent는 mixed payload에서도 거부된다.
- Typed stage는 checkpoint export, continue, checkpoint load와 transaction rollback에서 보존되고
  missing/mismatch authority는 fail closed한다.

## 8. Completion Evidence

- Core focused: monolith 17/17, live RON 23/23.
- Rust broad: workspace all-targets 전부 통과, Core lib 798/798.
- Unity: focused 3/3, EditMode 260/260, PlayMode 62 passed + 4 explicit live-server skips.
- Completion review: `Complete / High` on 2026-07-30.
