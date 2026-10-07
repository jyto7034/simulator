# R06 WP02 D07-B 실제 평가 문맥·수치·계산 과정 — 단계검토 인계

상태: **D07-B 부분 구현·자체 검증 종료 / 단계검토 target 동결 / 독립 검토 요청 대기**.

시작점은 D07-A 독립 PASS target manifest SHA-256 `ae6609c1e75b711a27cacf8467ac0c42eaa99635a0fc19c5ae486ecd949bb1ac`, 338개 source digest `5279b8333c6be8f90992ac94a47f09beb083bf82bfcdf02ebbe3e0d179c8f530`다. 구현 전 D07-A `freeze.py verify`를 통과했고, 338개 바이트와 신규 두 파일의 부재를 `baseline/entry_001`에 보존했다.

최종 확장 source는 340개, digest `ad8021359785e365c3f9560268bcef2a95205495f9e33a22314c3f97743480c4`다. 제품 source 336개 digest는 `6b5e0291b1b6f936c277e2138bc2c2adbbeaaa2b3df5c1b4d5bf31b5acff6807`다. 단계검토 target은 `target/d07_runtime_amount_001/MANIFEST.json`이 소유한다.

## 구현 결과

### 실제 소비 기록과 공통 계산

- `TrialLimits.amount_observation`을 기본 false인 신뢰 host opt-in으로 추가하고 `CoreApplication → GameCore → world combat → BattleCore`에 전달했다. optional NDJSON trace와 독립적으로 동작한다.
- runtime의 generation/site/fingerprint/action/delivery/target 검증 및 Snapshot/Live frame 선택 뒤 `evaluate_compiled_amount`가 `ExprMeter → evaluate → narrow`를 한 번 수행한다. 실제 소비와 query replay가 이 동일 helper를 사용한다.
- 관측 on인 legacy Damage/Heal은 catalog가 만든 context-free plan과 빈 exact frame을 공통 evaluator에 넣고 그 amount를 실제 command가 소비한다. catalog 없는 programmatic effect와 관측 off 일반 gameplay는 기존 literal 경로를 유지한다. production authored DamageExpr/HealExpr 차단은 해제하지 않았다.
- 대상별 실제 평가에서 한 개의 최신 candidate만 유지하고 command 처리, projection settle, action prefix settle 성공 뒤 observer에 commit한다. 0 amount도 평가 사실로 기록한다. 무대상 D04-F01 경로는 평가·ordinal 증가·가짜 결과가 없다.

### 수명·identity·rollback·비용

- private `RecordedAmountEvaluation`은 content generation, battle key, checked ordinal, EffectSite/fingerprint/kind/provenance/moment, 실제 source/장비 copy/action/delivery, release/consume target·time, effect index, immutable plan/frame, typed result/narrowed amount/trace/usage를 소유한다.
- active 전투마다 `Arc` 최근 1건만 유지한다. 새 기록이 앞 기록을 대체하며 map/ring/history Vec는 없다. 종료 시 기존 last-terminal capsule이 동일 Arc를 보관하고 action/delivery 정리 뒤에도 재생된다. checkpoint에는 넣지 않았다.
- effect/target/timestamp/receipt 실패는 기존 core/world candidate rollback으로 gameplay, record와 ordinal을 함께 복원한다. 전역 collector, mutex, thread-local record 저장소는 없다.
- `EvalFrame::retained_extent`, source/site 문자열, plan/frame/view/trace/member payload, typed value를 checked 계산한다. 교체 admission은 old+new가 공존하는 peak를 allocation 전에 검사하며 runtime fact retained extent와 clone preflight에도 record payload를 포함한다. controlled clone과 timestamp scratch는 immutable Arc를 공유한다.

### 조회와 CLI

- 별도 닫힌 `AmountQuery`는 `Latest { Active | LastTerminal }`와 `Exact { battle_key, evaluation_ordinal }`만 받는다. public DTO에는 readonly identity/value/trace/usage/extent만 투영하며 plan/frame/source capability를 반환하지 않는다.
- `TrialSession::preview_amount`와 `explain_amount`는 독립 Inspection scope에서 exact record를 검증·재평가한다. runtime 결과와 replay가 다르면 typed invariant error다. disabled/no battle/not yet evaluated/not retained는 서로 다른 unavailable이며 0 성공값으로 바꾸지 않는다.
- explain은 D07-A 정적 plan을 추가하고 `evaluation.performed=false`를 유지한다. 미래 시전 수치는 제공하지 않는다. target usefulness는 expression 또는 mixed future amount를 `Indeterminate`로 유지하면서 기존 hostile eligibility를 보존한다.
- CLI `simulate --amount-observation`은 scenario receipt 뒤 active/last-terminal preview/explain 최대 4개 typed NDJSON 문서를 출력한다. `--trace-output`과 독립이며 중복·unknown·비 simulate 사용은 거부한다. 중간 `serde_json::Value` 변환 없이 typed encoder를 쓴다.

## B01–B20 및 V17

상세 대응은 `evidence/B01_B20_MATRIX.json`에 있다. 전용 시험은 다음 5개이며 최종 `d07_b_focused_006`에서 5/5 통과했다.

- `d07_b01_b04_legacy_consumer_and_recorded_replay_share_one_numerical_kernel`
- `d07_b06_b09_multi_target_order_and_zero_evaluation_replace_one_record`
- `d07_b10_failed_later_effect_rolls_back_record_and_ordinal_with_gameplay`
- `d07_b09_b14_amount_query_states_are_typed_and_do_not_advance_the_trial`
- `d07_b15_public_trial_amount_preview_explain_and_exact_use_actual_legacy_execution`

`evidence/V17_RUNTIME_QUERY_CORRESPONDENCE.json`은 두 실제 소비를 보존한다.

- Snapshot: 현재 외부 legacy publication의 실제 projectile 소비 amount 42를 last-terminal에 보관했다. CLI preview/explain의 identity, typed amount, trace와 runtime/query usage가 같다. 정적 설명은 `performed=false`, publication은 `legacy_supported`다.
- Live: staged authored dynamic ordered executor가 실제로 소비한 amount 11의 exact frame을 보관했다. 같은 record에서 만든 preview/explain 원문은 typed amount, trace, usage가 같고 publication은 `staged_only`다.

이 증거는 public legacy, staged dynamic, 미지원 미래 예측을 구분한다. D07-C effective stat/Certified input/save와 WP03/04는 구현하지 않았다.

## 최종 검증

| 검증 | 최종 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo check -p game_core --all-targets --offline --locked` | exit 0 |
| `cargo test -p game_core --lib authoring --offline --locked` | 112 passed / 0 failed |
| `cargo test -p game_core --lib value_expr --offline --locked` | 91 passed / 0 failed |
| `cargo test -p game_core --all-targets --offline --locked` | 기본 병렬 1,538 passed / 0 failed |
| 지정 allow의 clippy | exit 0, 기존 및 비차단 warning 보존 |
| 실제 CLI trace on/off | 각각 exit 0, 34 receipt/amount 행; amount 4문서 byte-identical SHA-256 `8d555df63d8410237e0a9796d7cadb038197d5e31c3dcf22f158c8c3bf6ceaa0` |
| CLI 옵션 거부 | duplicate/unknown/validate-option 모두 exit 2 |
| PC36 | 576행, 36조건; 31 완료/5 typed 거부; gameplay/source/hit order/RNG/native usage 차이 0 |
| R05 | raw exit 101; concurrent peak 단일 warning, 다른 hard guard 없음 |

PC36은 final 340개 source를 격리한 뒤 보존 private probe hook만 주입해 실행했다. 관측 off 기존 조건은 선행 raw와 gameplay 및 비용 필드가 같다. 관측 on은 위 5개 전용 시험과 V17, 실제 CLI에서 별도로 검증했다.

R05 event-heavy는 p95 13,651,973ns, cap 25ms, peak/retained 255,558 bytes, 결정성 parity를 통과했다. concurrent는 p95 7,366,165ns, cap 10ms, retained 4,196,928 bytes, allocation/reallocation 364,268/6,660이 매 sample 동일했다. peak samples는 9,018,430 / 9,061,034 / 9,047,976 / 9,020,428 / 9,036,804 bytes다. 최대값은 원 9,030,188 기준을 30,846 bytes 초과했다. 원 도구 assertion과 exit 101을 보존하고, 승인된 정책에 따라 이 **concurrent peak 하나만 warning**으로 분류했다. OOM, parity·반복성·latency·retained·event guard 또는 다른 hard failure는 없다.

실패와 재시도는 `FAILURE_HISTORY.md`, 최종 명령·log SHA는 `evidence/FINAL_VALIDATION.json`, PC/R05 원자료와 분류는 `evidence/PC_SUMMARY.json` 및 `evidence/R05_SUMMARY.json`에 있다.

## 동결·잔여 범위

동결 target은 338개 before/340개 after archive, 신규 파일 부재↔존재, 정방향·역방향 patch, 증거·인계 archive를 묶는다. `freeze.py verify`는 current workspace와 after archive의 byte 일치 및 before→after→before 격리 복원을 확인한다. 기존 dirty/untracked 자료와 선행 target은 정리하거나 덮어쓰지 않았다.

D07-B 구현과 자체 검증까지만 종료했다. D07 전체, D07-C, WP02/R06 완료, WP03/04, production authored publication 허용, 20분 시험, 독립 검토 PASS를 선언하지 않는다. 원 대화에서 이 target을 독립 검토해 달라.
