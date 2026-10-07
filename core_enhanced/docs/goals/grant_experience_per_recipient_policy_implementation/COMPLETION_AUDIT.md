# GrantExperience Per-Recipient Policy Implementation Completion Audit

Audit Mode: goal_exit
Audit Target: `grant_experience_per_recipient_policy_implementation`
Current Status At Audit Entry: active
Implementation Complete Claim At Audit Entry: false
Policy Source: user approval on 2026-08-13 and `docs/game_rulebook.md` Reward/employee growth policy
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Repository/Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Dirty State: 304 status entries at final audit; the repository was broadly dirty before Goal activation

## Completion Evidence Matrix

| ID | 완료 조건 또는 불변식 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| C1 | 수정 전 hunk와 baseline 기록 | PLAN C1 | `EXPERIMENTS.md` pre-mutation inventory | four fresh legacy commands, 4/0/0 | pass |
| C2 | 모든 대상이 authored amount 전액 수령 | approved amount 20 × 3 = 60 | `reward.rs:423-480` passes `amount` unchanged per target | `grant_experience_`: 7/0/0 after 4 expected TDD failures | pass |
| C3 | 네 target의 대상 집합 유지 | approved target matrix | `reward.rs:434-456` | Alive/CombatParticipants/SelectedEmployee/AllRoster matrix | pass |
| C4 | 정렬·dedupe·membership·context 실패 유지 | PLAN invariants | `reward.rs:434-458` | participant order/duplicate/nonmember and error/no-op test | pass |
| C5 | zero/empty/growth/tier/saturation 유지 | PLAN invariants | `reward.rs:430-469`; unchanged `employee.rs:574-579` growth reducer | error/no-op matrix, selected amount 250 → level 3/XP 50, full Core | pass |
| C6 | preview/apply/diff가 직원별 정확한 값 노출 | gameplay/runtime canonical | `reward.rs:470-477`; `world/reward.rs:169-201`; `behavior.rs:4148-4178` | preview/apply 1/0/0 and reward session exact amounts | pass |
| C7 | late failure와 retry가 부분 XP를 남기지 않음 | runtime canonical staging | staged grant commit and combat plan/commit boundary | combat retry rollback 1/0/0 | pass |
| C8 | 생존 XP와 숙련 Elite 소탕 실효 동작 유지 | game rulebook/runtime canonical | SelectedEmployee loops in combat/sweep paths | survival 1/0/0; sweep XP 1/0/0 | pass |
| C9 | live amount 200/AliveRoster와 전액 설명 | approved live-data boundary | `game_resources/.../rewards/base.ron:21-28` | live RON policy/description 1/0/0 | pass |
| C10 | DTO/server/save shape와 version 불변 | PLAN migration boundary | existing DTO/projection; Active Run V4 and Profile V2 unchanged | game_server 72/0/0; static hunk audit | pass |
| C11 | focused 및 전체 Core/server 검증 | PLAN validation | current dirty tree binaries | focused 13/0/0; Core 1080/0/0; server 72/0/0 | pass |
| C12 | canonical 일치와 legacy 제거 | game rulebook + runtime canonical | both canonical sections describe per-recipient full amount | scoped legacy search: 0 matches | pass |
| C13 | 기존 dirty 변경과 범위 보존 | PLAN scope/stop conditions | pre/post hunk inventory; narrow Goal seams only | scoped status/diff, fmt and diff check | pass |
| C14 | 완료 감사 `Complete / High` | completion review guide | this audit | all matrix rows pass; no material finding | pass |

## Runtime And State Evidence

- `GrantExecutor::grant_experience` remains the single mutation authority. It resolves the four target variants,
  sorts UUIDs, deduplicates them, preserves the existing roster-membership filter and returns the existing empty
  result for an empty final target set.
- The removed code is limited to `eligible_count`, quotient, remainder and zero-share skipping. The loop now calls
  `Employee::add_experience_with_policy(amount, ...)` and records that exact authored amount for every resolved
  employee.
- `amount == 0`, missing participant context and missing selected context still return `InvalidAction` before the
  staged state is committed. Target tests compare all five serialized grant roots before and after each failure.
- `EconomyGrantState` remains the reward staging authority. Preview mutates only a captured state; apply commits only
  after every effect succeeds. Combat completion continues to plan the complete grant/result transition before its
  single commit.
- The employee growth reducer is unchanged and still uses saturating addition, repeated level threshold subtraction,
  saturating level increment and policy-derived tier. Both a direct employee unit test and the new SelectedEmployee
  integration case prove amount 250 becomes level 3 with 50 current-level XP.

## Data Evidence

- Live `experience_reward` still authors exactly `amount: 200` and `target: AliveRoster`.
- Only its description changed, from shared/divided wording to explicit full payment for every target employee.
- The live integration loader deserializes the actual production RON and asserts ID, amount, target and description.
- No RON enum, schema, default, alias or reward-pool connection was added. Amount 200 balance remains intentionally
  unchanged and outside this correctness correction.

## DTO/Transport/Persistence Evidence

- `EmployeeExperienceDiffDto` retains the same six fields. Only runtime values now reflect the approved semantics.
- `BehaviorResult` projection still serializes `employee_experience_diffs` directly for regular reward, combat reward
  and mastered sweep results. No server-side division, multiplication or target reconstruction exists.
- The Core/server broad suites pass on the current tree. The Goal did not edit server mapping, request ordering,
  checkpoint payloads, save readers or writers.
- Active Run remains version 4 and PlayerProfile remains version 2. `GrantExperience.amount` is authored effect
  semantics, not persisted state shape, so no version bump or migration branch is required.

## Consumer Evidence

- Confirmed workspace consumers: Core reward/session/combat/sweep paths and game_server transport.
- Production data consumer: the real RON loader used by `tests/ron_loading.rs`.
- Explicitly excluded consumer: Unity source, runners and presentation, per the user's development-tree direction.
- No inaccessible consumer is required for this Goal because the serialized DTO shape did not change.

## Legacy/Compatibility Audit

The scoped search covered Core production/tests, live data, gameplay canonical and runtime canonical for the former
division expressions, `[3, 2, 2]`, divided-payment Korean wording and current-runtime total-pool mismatch wording.
It returned zero matches. Intentional canonical phrases saying quotient/remainder distribution does not exist remain.

No per-recipient feature flag, legacy mode, version switch, fallback, alias, dual schema, ignored test or caller-level
amount multiplier was introduced. Historical completed Goal evidence was not rewritten.

The existing `combat_setup::apply_combat_reward_multiplier` may scale the authored XP amount with the same fixed
combat reward percentage used for other quantity effects before grant execution. It is unchanged, does not inspect
recipient count and is not a competing distribution authority; the central reducer still defines what that final
authored amount means for every target.

## Scoped Diff And Dirty Preservation

- Pre-mutation inventory identified broad existing user changes in all adjacent large files. In particular,
  skill-fragment-research removal in `reward.rs`, unrelated node/combat tests, reward catalog removal/pool changes,
  canonical expansion and server/persistence work were already present and remain untouched.
- Goal-owned production/test seams are limited to the XP arithmetic block, two existing XP expectations, five
  adjacent target-policy tests, one live-loader assertion, one live description line and one runtime-canonical
  legacy sentence removal.
- `game_rulebook.md` already contained the approved target policy before Goal execution and required no new edit.
- No reset, restore, stash, deletion, broad formatting rewrite, Unity mutation or balance/content tuning was used.

## Canonical Alignment

- `game_rulebook.md:720-731` defines authored amount as per-recipient and the four target meanings.
- `core_runtime_contract.md:744-760` now describes the implemented resolver, staging, diff and growth behavior without
  the prior implementation-mismatch warning.
- Runtime, live description and both canonical sources now agree at field and recipient-set level.

## Validation Results

All commands ran in the current dirty worktree on 2026-08-13.

```text
Pre-mutation characterization:
cargo test -p game_core reward_session_grant_experience_uses_explicit_alive_roster_target_and_reports_diffs -- --quiet
  1 passed; 0 failed; 0 ignored
cargo test -p game_core grant_preview_matches_apply_without_mutating_any_of_the_five_live_roots -- --quiet
  1 passed; 0 failed; 0 ignored
cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry -- --quiet
  1 passed; 0 failed; 0 ignored
cargo test -p game_core live_experience_rewards_declare_target_policy -- --quiet
  1 passed; 0 failed; 0 ignored

Target-first RED evidence before production mutation:
cargo test -p game_core grant_experience_ -- --quiet
  2 passed; 4 failed; 0 ignored; failures were the four multi-recipient full-amount assertions
cargo test -p game_core live_experience_rewards_declare_target_policy -- --quiet
  0 passed; 1 failed; 0 ignored; old divided-payment description

Focused GREEN evidence after implementation:
cargo test -p game_core grant_experience_ -- --quiet
  7 passed; 0 failed; 0 ignored
six independent focused commands for preview, rollback, survival XP, mastered-Elite capability,
mastered-Elite XP and live RON
  6 passed; 0 failed; 0 ignored

Broad validation:
cargo test -p game_core -- --quiet
  lib 1029 + live_item 4 + live_catalog 4 + RON 25 + skill contract 14 + skill suite 4
  = 1080 passed; 0 failed; 0 ignored
cargo test -p game_server -- --quiet
  72 passed; 0 failed; 0 ignored
cargo fmt --all -- --check
  exit 0
git diff --check
  exit 0
```

## Baseline Failures

- No unexplained baseline failure exists.
- The five RED failures were deliberately introduced target-policy assertions and disappeared after the central code
  and live-description correction. No expectation was weakened to obtain GREEN.

## Cross-Component Findings

- Core producer, live RON and game_server transport use one final contract.
- No competing XP arithmetic authority, schema mismatch, persistence migration or temporary compatibility shim was
  found.
- Material findings: none.

## Long-Term Direction Review

| Candidate ID | 작업 유형 | 현재→목표 authority 결과 | 보존 불변식 결과 | 제거·통합 결과 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- | --- |
| GXP-C01 | correctness correction / policy migration | same central reducer now owns per-recipient amount | target/context/growth/atomicity preserved | quotient/remainder legacy removed without parallel path | focused 13 + broad 1152 tests | pass |

- Fit: High
- Improvement class: none
- Reason: the correction removes arithmetic from the existing single authority, adds no abstraction or compatibility
  state, and protects contract-visible target/diff behavior through staged world paths and live data.

## Primary Verdict

- Complete
- Long-term fit: High

## Status Transition

- from: active / `Implementation Complete: false`
- to: complete / `Implementation Complete: true`
- next Phase activation: not applicable; this is a single Goal

## Remaining Risk And Required Action

- No action is required for this Goal.
- Live amount 200 balance/playtesting, new reward content and Unity integration remain explicitly out of scope. These
  are follow-up content/integration concerns, not missing evidence for the unchanged DTO boundary.
