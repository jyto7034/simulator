# GrantExperience Per-Recipient Policy Implementation Goal

Status: complete
Implementation Complete: true
Goal Type: single
Execution Mode: implement_and_verify
Production Mutation Authorized: yes; limited to the approved per-recipient XP contract
Repository Root: `/mnt/f/work/simulator`
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Created: 2026-08-13
Activated: 2026-08-13
Completed: 2026-08-13

## Objective

`GrantExperience.amount`를 total pool 균등분배가 아니라 target이 선택한 직원 한 명당 지급량으로
구현한다. 네 target의 대상 선택 의미를 유지하면서 모든 대상에게 authored amount 전액을 지급하고,
Core/live description/canonical/test를 한 계약으로 동기화한다.

## User/Contract-Visible Problem

확정 정책에서 `amount: 20`이고 대상 직원이 셋이면 각자 20 XP, 총 60 XP를 받아야 한다. 현재
`GrantExecutor::grant_experience`는 amount를 대상 수로 나누고 remainder를 낮은 UUID부터 배분하므로
`amount: 7`, 대상 세 명이 `[3, 2, 2]`를 받는다. Live `experience_reward` 설명과 기존 회귀 테스트도 이
legacy 의미를 반영한다. 새 reward/bonus 콘텐츠를 authoring하기 전에 중앙 지급기와 모든 관측 evidence를
승인 정책에 맞춰야 한다.

## Audit Finding ID

없음. 사용자와의 명시적 정책 확정에서 유래한 correctness correction이다.

## Activation Baseline Runtime And Live Data Evidence

Goal 초기화 전 승인된 구현 검토에서 확인한 입력이다. Production 수정 전 current hunk와 baseline을 다시
기록한다.

- `core_enhanced/src/game/reward.rs::GrantExecutor::grant_experience`가 네 target의 최종 XP mutation authority다.
- 활성화 시점 구현은 대상 UUID를 sort/dedupe/membership filter한 뒤 quotient/remainder를 분배했다.
- `world/tests/node_sessions.rs`의 AliveRoster 회귀는 amount 7을 `[3, 2, 2]`로 기대하고 preview diff 합계
  7을 고정한다.
- `game_resources/data/events/rewards/base.ron`의 `experience_reward`는 `amount: 200`, `AliveRoster`이고
  description은 분배 의미다. 현재 live encounter와 `default_treasures` pool은 이 entry를 참조하지 않는다.
- 전투 생존 XP와 mastered Elite sweep은 직원별 `SelectedEmployee` grant를 사용해 이미 대상자별 전액을
  지급한다.
- DTO와 game_server는 `EmployeeExperienceDiffDto`를 전달할 뿐 amount 의미를 재계산하지 않는다.
- 활성화 시점 canonical은 승인 목표와 runtime legacy 불일치를 분리해 기록했다.
- Current worktree는 Goal 대상 code/test/data/canonical을 포함해 광범위하게 dirty하다. 기존 hunk는 사용자
  소유로 보존해야 한다.

## Source Of Truth And Ownership

- Gameplay 의미: `core_enhanced/docs/game_rulebook.md`의 대상자별 전액 정책과 사용자 승인.
- Target 선택과 XP mutation: Core `RewardEffect`, `ExperienceTargetPolicy`, `GrantExecutionContext`,
  `GrantExecutor::grant_experience`.
- Level/잔여 XP/tier: `Employee::add_experience_with_policy`와 live `RunPolicyData`.
- Live 표시와 authored amount/target: `game_resources/data/events/rewards/base.ron`.
- Result observation: `EmployeeExperienceDiffDto`와 기존 Core behavior result; game_server는 transport only.
- Runtime canonical: `core_enhanced/docs/core_runtime_contract.md`.
- Goal 문서는 실행 범위와 evidence를 소유하며 durable 정책을 대신하지 않는다.

## Invariants

- `amount`는 각 최종 대상에게 그대로 적용한다. 대상 수에 따른 quotient/remainder가 없다.
- `AliveRoster`는 physical damage 6 미만인 모든 직원이다. Trauma, combat eligibility, 배치, 무장과 roster
  slot을 추가 조건으로 사용하지 않는다.
- `CombatParticipants`는 explicit context 참가자, `SelectedEmployee`는 explicit context 직원,
  `AllRoster`는 사망자를 포함한 roster 전체다.
- Target UUID의 deterministic sort, dedupe와 현재 membership 처리 의미는 유지한다.
- Missing participant/selected context는 기존 `InvalidAction`이며 state를 변경하지 않는다.
- Amount 0 validation, XP curve, multi-level, current-level remainder, tier derivation과 saturation은 유지한다.
- `EmployeeExperienceDiffDto.amount`와 before/after는 실제 직원별 mutation을 정확히 표현한다.
- Reward preview/apply와 reward/combat transaction은 모든 effect를 함께 stage하고 실패 시 부분 commit하지
  않는다.
- 전투 생존 XP는 실제 생존 참가자만 지급한다. Mastered Elite sweep은 살아 있는 roster 전체에 전액을
  지급한다. 두 경로의 실효 결과를 바꾸지 않는다.
- P01~P04 game-rulebook Master의 progression, V4/V2, checkpoint/profile, sweep semantics는 변경하지 않는다.

## Refactor Candidate And Behavior-Preservation Boundary

- Candidate ID: `GXP-C01`
- Work type: `correctness_correction` / `policy_or_feature_migration`
- Current authority: 중앙 `GrantExecutor::grant_experience`, total-pool 산술.
- Target authority: 같은 중앙 함수, per-recipient authored amount.
- 제거할 legacy: `eligible_count`, `base_amount`, `remainder` 분배와 이를 고정한 현재 테스트/description/
  runtime mismatch 문구.
- 보존 경계: target membership, ordering/dedupe, context failure, level arithmetic, diff shape, transaction,
  DTO/server/save shape.
- Priority evidence: gameplay impact high, correction locality high, schema/migration risk low. Numeric score는
  산정하지 않았으며 완료 판정에 사용하지 않는다.
- Characterization: 활성화 시점 legacy AliveRoster/preview/live target/rollback focused tests 각 1건 green.
- 정책 또는 외부 계약 결정 필요: 없음. Unity는 명시적 비범위다.

## Approved Policy And Evidence

- 사용자 승인, 2026-08-13:
  - `AliveRoster`: 살아 있는 모든 직원 각각 전액
  - `CombatParticipants`: 지정된 참가자 각각 전액
  - `SelectedEmployee`: 선택 직원에게 전액
  - `AllRoster`: 생사와 무관한 모든 직원 각각 전액
  - 예: amount 20, 대상 세 명이면 각 20, 총 60
- Updated target policy: `game_rulebook.md`.
- Current/target boundary: `core_runtime_contract.md`.

## Open Policy Decisions

없음. Live `experience_reward.amount: 200`의 향후 밸런스 조정은 이 Goal의 정책 결정이 아니며 현재 값을
유지한다. Context에 roster 외 UUID가 들어왔을 때 현재 membership filter를 오류로 강화할지도 별도 validation
정책이므로 이번 Goal에서는 현 동작을 보존한다.

## In Scope

- 대상 production 파일의 current diff/hunk ownership과 focused baseline 기록
- 네 `ExperienceTargetPolicy`의 target-first per-recipient tests
- 중앙 XP 지급기에서 total-pool quotient/remainder 제거
- preview/apply/diff/level-up/context failure와 atomic rollback 회귀
- Live `experience_reward` description을 대상자별 전액 의미로 동기화
- Runtime canonical의 legacy 불일치 문구를 구현 완료 상태로 전환
- Focused Core/live-RON tests, full Core/server regression, formatting/diff and completion audit

## Out Of Scope

- `experience_reward.amount: 200`, XP curve, 생존 XP 10 또는 reward multiplier tuning
- Reward target enum/RON/DTO/JSON/schema 변경
- Context membership filter를 fail-closed error로 바꾸는 별도 validation 정책
- Mastered sweep의 optional AliveRoster 단순화; 현재 정확한 SelectedEmployee loop 유지
- 신규 reward/bonus 콘텐츠, 장비/파편/보스 콘텐츠
- Active Run/PlayerProfile/save/checkpoint migration
- Unity source, runner, presentation 및 live-client probe
- 과거 완료 Goal 문서 재작성과 unrelated dirty cleanup/refactor

## Implementation Scope

```text
RewardEffect::GrantExperience { amount, target }
  -> GrantExecutor target resolver
      -> deterministic sort/dedupe/current membership handling
          -> for each target: add_experience_with_policy(amount)
              -> exact per-employee diff
                  -> existing cloned grant/world transaction commit
```

Target-first tests must fail against the activation-baseline quotient/remainder behavior before production mutation. The code
change should remain centralized; no caller-specific multiplier or compatibility branch is permitted.

## DTO/Data/Save Migration

- DTO: shape unchanged. Only existing diff values change to the approved per-recipient amount.
- Data: schema unchanged. Only the live `experience_reward` description changes; amount 200 and target remain.
- Save/checkpoint/profile: no field, semantic lifetime or version change. Active Run Core/host V4 and PlayerProfile V2
  remain exact.
- Compatibility: no dual behavior, alias, legacy flag or version switch. Current runtime changes atomically.

## Deployment And Compatibility

Development-only Core/live-RON/game_server atomic boundary. No public release gate is required. Because serialized
shape is unchanged, game_server only needs regression verification. Unity is explicitly unverified/out of scope and
no ignored field, fallback or local XP recomputation is introduced for it.

## Focused Validation

- Current legacy characterization before mutation:
  - AliveRoster current split/diff test
  - preview/apply equality
  - live reward target
  - reward rollback
- Target-first matrix:
  - AliveRoster: Normal/Critical/Breakdown included, Dead excluded, each full amount
  - CombatParticipants: out-of-order/duplicate context deduped, each valid member full amount
  - SelectedEmployee: one valid selected employee gets full amount
  - AllRoster: living and Dead each get full amount
  - multi-level amount 250 applied in full to every target
  - missing context error and empty-target current meaning
- Integration regression:
  - reward session and preview/apply exact values
  - combat participant reward and atomic retry failure
  - survival XP once per surviving participant
  - mastered Elite living-only full XP
  - live RON amount/target/description contract

Exact test names and actual pass/fail/ignored counts are recorded in `EXPERIMENTS.md` and `COMPLETION_AUDIT.md`.

## Broad Validation

- `cargo test -p game_core -- --quiet`
- `cargo test -p game_server -- --quiet`
- `cargo fmt --all -- --check`
- `git diff --check`
- Scoped static search proving no reward XP quotient/remainder legacy remains in production/current canonical/live
  description; historical completed Goal evidence is not rewritten.

## Omitted Validation

- Unity source/EditMode/PlayMode/live client: explicitly excluded by user direction; DTO shape does not change.
- Gameplay balance/playtest of amount 200: content tuning is outside this correctness correction.
- Public deployment/migration rehearsal: development-only and no schema/save change.

## Completion Conditions

1. Target file dirty hunks and current focused baseline are recorded before production mutation.
2. Every resolved target receives exactly authored `amount`; no quotient/remainder calculation remains.
3. AliveRoster/CombatParticipants/SelectedEmployee/AllRoster target membership matches the approved contract.
4. Sort/dedupe/current membership handling and missing-context failure remain deterministic and atomic.
5. Amount 0, empty target, multi-level/remainder/tier and saturation semantics remain unchanged except per-target amount.
6. Preview, apply and `EmployeeExperienceDiffDto` expose exact per-employee full amounts and before/after values.
7. Reward/combat late failure still leaves all grant roots unchanged and retry cannot duplicate XP.
8. Survival XP and mastered Elite sweep retain their approved effective recipient/amount behavior.
9. Live `experience_reward` keeps amount 200/AliveRoster but describes per-living-employee full payment.
10. DTO/JSON/server/save/checkpoint versions and shapes remain unchanged; no compatibility branch is added.
11. Focused target matrix, Core integrations, live-RON and full Core/server suites pass with non-zero actual counts.
12. Gameplay/runtime canonical matches implementation and legacy current-split wording is absent outside history.
13. Scoped diff preserves existing user changes and contains no unrelated refactor or balance/content modification.
14. Completion audit under `goal_completion_review_guide.md` receives `Complete / High` before status changes.

## Completion Audit Evidence Matrix

| Condition | Required evidence | Current verdict |
| --- | --- | --- |
| C1 dirty/baseline | target hunk inventory + current focused results | pass — pre-mutation inventory and 4/0/0 baseline |
| C2 amount semantics | central reducer + target-first full-amount tests | pass — central direct amount; 7/0/0 matrix |
| C3 target membership | four-policy matrix | pass — all four policies and condition boundaries |
| C4 determinism/context | order/dedupe/missing-context tests | pass — deterministic/filter/error/no-op coverage |
| C5 growth edges | zero/empty/multi-level/saturation tests | pass — amount 250 and max-level remainder coverage |
| C6 projection | preview/apply and exact diff assertions | pass — exact per-employee diffs and equality |
| C7 atomicity | failure/retry root fingerprint | pass — two retries preserve all grant roots |
| C8 existing XP paths | survival/sweep regression | pass — focused survival and living-only sweep |
| C9 live data | RON load/assertion and description audit | pass — amount 200/AliveRoster/full wording |
| C10 migration/compat | DTO/server/version static/runtime audit | pass — shapes unchanged; server 72/0/0 |
| C11 validation | focused/full actual counts | pass — focused 13; Core 1080; server 72 |
| C12 canonical/legacy | scoped search and canonical comparison | pass — 0 legacy matches |
| C13 dirty preservation | scoped final diff | pass — narrow seams; existing hunks preserved |
| C14 audit | `COMPLETION_AUDIT.md` verdict | pass — `Complete / High` |

## Stop Conditions

- Target code/test/data hunk overlaps an existing user change that cannot be safely preserved.
- Implementing per-recipient XP requires changing target membership, XP curve, amount 200 or another unapproved
  gameplay policy.
- A DTO/schema/save version, compatibility branch or Unity change becomes necessary.
- Target-first tests reveal another competing XP mutation authority or caller-specific total-pool contract.
- Current focused/broad runner has an unexplained failure relevant to this Goal.
- Work expands into content balance, new reward content, hidden boss, WP005 or unrelated cleanup.
