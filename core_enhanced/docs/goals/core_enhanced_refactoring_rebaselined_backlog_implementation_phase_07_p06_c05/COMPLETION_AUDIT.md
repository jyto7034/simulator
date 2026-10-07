# Phase 07 CRA-P06-C05 Completion Audit

Audit Mode: phase_exit
Audit Target: Phase 07 — CRA-P06-C05 Use One Saturating MVP Score For Selection And Publication
Current Status At Audit Entry: active
Implementation Complete Claim At Audit Entry: false
Policy Source: no direct policy ID; arithmetic-consistency requirement and preservation boundary in this Child
`PLAN.md`, Master Phase 07 registry and the final exact linear implementation order
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Repository/Commit/Dirty State: `/mnt/f/work/simulator` at
`f318414fb0156a67e04c5479000c2a2683fc0c09`; expanded status moved from 403 entries at activation to 405
entries at exit, final status SHA-256
`94905ae23c2457e227ce1ed8c797decf3824b219322e703090145e0e7093fd8c`

## Completion Evidence Matrix

| ID | 완료 조건 또는 불변식 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| C-01 | 집계→점수→비교→DTO authority가 식별된다 | Child objective와 P07-C01 | `collect_battle_result_stats`가 UUID별 집계 뒤 `select_mvp`를 한 번 호출하고 world가 그 DTO를 CombatResult에 저장한다 | result-stats 7/7, combat-result 6/6 | pass |
| C-02 | accepted signed 극단값의 곱과 총합이 debug/release에서 명시적으로 saturate한다 | preserved invariant 4, arithmetic correction | `metric_score`의 `saturating_mul`; `evaluate_mvp_score`의 authored-order `saturating_add` | pre-fix debug overflow와 release 0/2 red; post-fix debug 2/2 및 release 2/2 | pass |
| C-03 | 선택과 공개 score/title/highlight가 하나의 evaluation을 공유한다 | objective, P07-C04 | `EvaluatedMvpCandidate`가 후보별 `MvpScoreEvaluation` 하나를 보존하고 comparator와 selected DTO construction이 같은 값을 소비한다; 구 `score_total` 제거 | 양/음 극단 테스트가 employee UUID, score, title, highlight를 함께 고정 | pass |
| C-04 | live score, highlight, tie, incapacitation과 redeployment 집계가 보존된다 | preserved invariants 1~3, 5 | non-Score tiebreak와 lower-UUID fallback, highlight replacement, aggregation/eligibility 코드는 변경되지 않았다 | ordinary score 160, UUID 양방향 입력, incapacitated MVP, multi-instance aggregation을 포함한 result-stats 7/7 | pass |
| C-05 | signed weight authoring, live RON과 public/serialized shape가 바뀌지 않는다 | out-of-scope 및 preserved invariant 6 | weight는 계속 `i64`; validator는 기존 non-empty 규칙; `game_resources/data/run/policy.ron`의 MVP 값과 `BattleResultMvpDto` field 불변 | run-policy 18/18, live RON 63/63, complete Core 944/944 | pass |
| C-06 | workspace와 외부 consumer가 Core DTO를 재계산 없이 소비한다 | migration/compatibility boundary | world가 collector 결과를 typed CombatResult에 저장하고 server가 snapshot을 운반한다; Unity canonical은 `selected_event.result_stats`를 1차 source로 지정한다 | combat-result 6/6, server all-target compile pass, external canonical read audit | pass |
| C-07 | canonical과 scoped dirty 경계가 실제 변경과 일치한다 | implementation strategy, P07-C02/P07-C06 | nearest runtime canonical에 shared saturating evaluation을 추가; production source diff는 previously-clean `result_stats.rs` 한 파일 | `cargo fmt --check` pass, `git diff --check` pass, scoped diff/hash audit | pass |
| C-08 | focused/broad 검증과 Phase exit review가 통과한다 | P07-C07/P07-C08 | 현재 dirty tree의 production/data/consumer 경계를 재검증했다 | focused 전부 pass, complete Core 944/944, server all-target compile pass | pass |

## Runtime And State Evidence

- `collect_battle_result_stats`는 event log와 participant result를 직원 UUID별 한 행으로 집계하고 UUID로
  정렬한 뒤 private `select_mvp`에 전달한다. 이 Phase는 upstream 집계, terminal instance 대표성,
  incapacitation eligibility를 수정하지 않았다.
- `metric_score`는 metric의 nonnegative `u64` 값을 `i64`로 변환하되 초과값은 `i64::MAX`로 제한하고 signed
  weight를 `saturating_mul`한다.
- `evaluate_mvp_score`는 policy의 authored metric 순서대로 각 weighted score를 `saturating_add`하고 기존
  weighted-score/value 우선순위로 첫 best highlight를 보존한다.
- `select_mvp`는 각 직원 행을 정확히 한 번 평가해 `EvaluatedMvpCandidate`에 저장한다. Score tiebreak는 저장된
  total만 비교하며, 최종 DTO는 선택된 candidate의 동일 total/highlight를 이동해 score/title/highlight를
  만든다. 선택 뒤 재평가 또는 ordinary `.sum()` 경로는 없다.
- Damage/Kill/Skill/BasicAttack/DeployedTime tiebreak 순서와 마지막 lower-employee-UUID fallback은 변경되지
  않았다. 모든 tiebreak가 같을 때 input order를 뒤집은 테스트도 UUID 1을 선택한다.
- 이 계산은 pure result construction이며 gameplay state mutation, 비용, clock, retry 또는 event ordering을
  추가하지 않는다.

## Data Evidence

- `BattleResultMvpMetricWeight.weight`는 계속 signed `i64`이며 negative, duplicate 또는 extreme weight를 새로
  거부하거나 normalize하지 않는다.
- run-policy validator는 기존처럼 metric weight와 tiebreaker가 비어 있지 않은지만 이 범위에서 검사한다.
- production `game_resources/data/run/policy.ron`의 MVP 값은 Damage `1`, Kill `100`, Skill `25`, Survived
  `50`이고 tie는 Score, Damage, Kill, Skill, time, UUID다. Phase는 이 block을 수정하지 않았다.
- Live RON 63개 검증이 현재 dirty tree에서 통과했다. Fixture만 새 계약을 사용하는 별도 data path는 없다.

## DTO/Transport/Persistence Evidence

- `BattleResultMvpDto`의 `employee_uuid`, `unit_instance_id`, `score`, `title`, `highlighted_metrics`와 highlight
  DTO field/type은 변경되지 않았다. 새 evaluation/candidate type은 module-private이며 serde를 파생하지 않는다.
- world `combat_battle_state_from_live_result`가 collector를 한 번 호출하고 그 typed result를
  `CombatBattleState.result_stats`에 저장한다. Behavior projection은 같은 `BattleResultStatsDto`를 운반한다.
- game_server는 result 통계를 재계산하거나 별도 MVP schema로 mapping하지 않고 Core snapshot JSON을 전송한다.
  all-target compile은 shape compatibility를 확인했고 combat-result test는 실제 typed snapshot을 확인했다.
- Result stats는 active battle/CombatResult 수명이며 durable run save/checkpoint authority가 아니다. 공개 shape와
  lifetime이 변하지 않아 version bump, dual reader/writer, fallback 또는 migration은 필요하지 않다.

## Consumer Evidence

- confirmed workspace consumers: world CombatResult construction/projection, PvE bonus-objective evaluator,
  combat trace diagnostic export와 game_server snapshot transport.
- confirmed external canonical consumer: read-only로 확인한
  `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`와 `unity_core_contract.md`는 Unity가
  `selected_event.result_stats`를 MVP/직원 통계의 1차 source로 사용하고 raw event log에서 재계산하지 않는다고
  명시한다. 기존 JSON example의 MVP shape도 현재 DTO와 일치한다.
- inaccessible/unknown consumer: 없음이 입증된 것은 아니다. 다만 이 Phase는 serialized/public shape를 바꾸지
  않았으므로 알려지지 않은 consumer에 새 rollout 요구를 만들지 않는다.
- explicitly excluded consumer mutation: Unity implementation과 외부 canonical 편집. Read audit만 수행했다.

## Legacy/Compatibility Audit

- `score_total`과 `score_employee` symbol은 production module에서 제거됐고 MVP total에 ordinary `.sum()`이
  남지 않았다.
- comparator/publication을 위한 dual arithmetic, build-mode별 fallback, debug bypass, ignored extreme test 또는
  compatibility alias를 추가하지 않았다.
- 기존 DTO/RON/schema를 유지한 것이 legacy 지원층은 아니다. 같은 shape 안에서 undefined overflow 의미만
  일관된 saturation으로 고쳤다.
- Negative/duplicate/range authoring 판단은 별도 정책 범위로 남겼으며 이 Phase가 숨은 validation policy를
  만들지 않았다.

## Scoped Diff And Dirty Preservation

- Activation 때 `result_stats.rs`는 clean SHA-256
  `9597992ca3f5799f5e94ab9b7a3201104c9345e89c9d5c56b9f3cfeceee0822c`였다. 최종 SHA-256은
  `ea6202a12362ffcc23f556eeb001bc7e125d21907c73bbecd6191966681b2762`이며 diff는 shared evaluator,
  divergent helper 제거와 세 regression test로 제한된다.
- `core_runtime_contract.md`는 activation 전부터 dirty였다. Phase hunk는 Active Battle Result section의 세 줄짜리
  shared-evaluation 계약이며 기존 대규모 사용자 변경과 분리된다.
- `run_policy_data.rs`, world combat test와 live RON의 기존 dirty 변경을 보존했고 Phase를 위해 수정하지 않았다.
- Expanded status는 403→405 entries다. 새 tracked production delta 하나 외에는 Child/audit 문서와 기존 dirty
  state이며 reset, restore, clean, stash, file deletion, 이동 또는 broad formatting을 수행하지 않았다.

## Canonical Alignment

- 현재 실제 동작: 후보별 signed saturated evaluation 하나가 selection과 publication을 소유한다.
- 현재 canonical: `core_runtime_contract.md`의 Active Battle Result section이 authored metric order, signed `i64`
  product/total saturation, shared score/title/highlight와 preserved signed weight/tie를 명시한다.
- 승인된 목표: Child objective와 동일하다. Goal 주장만 최신 의미를 소유하지 않으며 runtime/test/canonical이
  일치한다.
- 외부 Unity canonical은 result stats를 Core-owned source로 유지한다. Wire shape가 바뀌지 않아 external text
  correction은 필요하지 않다.

## Validation Results

모든 final 명령은 현재 dirty worktree에서 실행했다.

| 명령 | 실제 결과 |
| --- | --- |
| `cargo test --lib mvp_selection_and_publication_share_` | 2 passed, 0 failed, 0 ignored, 942 filtered |
| `cargo test --release --lib mvp_selection_and_publication_share_` | 2 passed, 0 failed, 0 ignored, 942 filtered |
| `cargo test --lib game::battle::result_stats::tests` | 7 passed, 0 failed, 0 ignored, 937 filtered |
| `cargo test --lib run_policy` | 18 passed, 0 failed, 0 ignored, 925 filtered |
| `cargo test --lib game::pve_bonus_objectives::tests` | 3 passed, 0 failed, 0 ignored, 940 filtered |
| `cargo test --lib combat_result` | 6 passed, 0 failed, 0 ignored, 937 filtered |
| `cargo test --lib live_ron` | 63 passed, 0 failed, 0 ignored, 880 filtered |
| `cargo test --lib` | 944 passed, 0 failed, 0 ignored, 0 filtered |
| game_server `cargo check --all-targets` | compile pass; runner test count not applicable |
| `cargo fmt --check` | pass; formatter reported no diff |
| repository `git diff --check` | pass; no whitespace error |

## Baseline Failures

- Intentional pre-fix debug positive and negative exact cases each selected one test and failed on ordinary signed
  addition overflow before assertion.
- Intentional pre-fix release combined filter executed two tests and failed both: positive selected UUID 2 instead of
  1; negative selected UUID 1 instead of 2. This distinguished release wrapping from the target saturation.
- The baseline result-stats family was 4/4 and run-policy family was 17/17. No unrelated baseline failure was hidden.
- Post-fix validation has no remaining failure or skipped/ignored Phase proof.

## Cross-Component Findings

- No material finding. Core is the only MVP producer; world/server/Unity boundaries consume the typed result rather
  than competing with it.
- PvE bonus objectives read battle summary fields but do not recalculate MVP. Their focused tests and the full package
  pass after the score-owner consolidation.
- No data/save/wire assumption from Phase 07 creates a temporary seam for Phase 08.

## Long-Term Direction Review

| Candidate ID | 작업 유형 | 현재→목표 authority 결과 | 보존 불변식 결과 | 제거·통합 결과 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P06-C05 | correctness correction | split comparator/publication arithmetic → one candidate evaluation | live score/title, signed weights, tie, eligibility, aggregation과 shape 보존 | divergent ordinary total helper 제거; duplicate score execution 제거 | extremes debug/release, result 7/7, Core 944/944 | pass |

- Fit: High
- Improvement class: none
- Reason: private evaluation object 하나가 기존 domain seam 안에서 score와 highlight ownership을 단순화한다.
  새 public abstraction, duplicate state, compatibility layer, special-case build path 또는 future placeholder가 없다.

## Primary Verdict

- Complete
- Confidence: High

## Status Transition

- from: `active`, `Implementation Complete: false`
- to: `complete`, `Implementation Complete: true`
- next Phase activation: allowed; Master가 Phase 07을 `complete — Complete / High`로 기록하고 Phase 08을
  `ready`로 전환한 뒤에만 활성화한다.

## Remaining Risk And Required Action

- Material remaining risk: none within the approved arithmetic-consistency scope.
- Negative weights, duplicate metrics와 authoring bounds의 desirability는 이 correction과 독립된 정책 질문이며
  현재 accepted semantics를 보존했다. Phase 07 완료를 막는 미결정 사항이 아니다.
- Immediate correction: none.
- Required action: Master registry/status를 audit verdict와 동기화하고 Phase 08 dependency gate를 연다.
