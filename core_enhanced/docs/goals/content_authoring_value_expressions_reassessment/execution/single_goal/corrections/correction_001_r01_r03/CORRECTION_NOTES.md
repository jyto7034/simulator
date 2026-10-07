# R06 correction 001 R01–R03 교정 기록

State: `자체 검증 완료 / correction_002 동결 입력`

## 변경 파일

제품 source 변경은 다음 네 파일로 한정했다.

- `core_enhanced/src/game/value_expr/mod.rs`
- `core_enhanced/src/game/value_expr/compile.rs`
- `core_enhanced/src/game/value_expr/eval.rs`
- `core_enhanced/src/game/value_expr/tests.rs`

보존·실행 기록은 이 correction 디렉터리와 같은 goal의 `authoring_scripts/r06_correction_002_*`, `r06_run_correction_002_validation.py`, `r06_recover_historical_correction_plan.py`에만 추가했다. 기존 checkpoint, correction 001 baseline/target, review evidence는 덮어쓰지 않았다.

## R01

재감사 원인은 compiler가 authored aggregate의 identity/owner/value payload scan 비용을 plan work에 모두 넣지 않았으나 evaluator는 그 비용을 부과한 점이다. 1-member plan은 6으로 승인되고 실제 7을 사용했다. 이 불일치를 다음과 같이 교정했다.

- member scan과 canonical sort 비용을 공용 checked helper로 정의해 compiler와 evaluator가 같은 식을 쓴다.
- compiler는 authored payload를 clone/sort하기 전에 raw count와 payload work를 승인하고, aggregate plan work에도 같은 preprocessing 값을 넣는다.
- outer aggregate는 child work를 member 수와 checked multiplication으로 합성한다.
- compile 시 알 수 없는 frame/resolved payload가 있으면 `runtime_dynamic_work=true`로 기록한다. `static_work`는 알려진 부분이고 `worst_case_work`는 해당 plan이 승인된 `max_work` ceiling이다.
- 정적 plan은 `runtime_dynamic_work=false`, `static_work == worst_case_work`이며 evaluator의 실제 work가 이 상한을 넘지 않는다.

정상 실행의 member sort, grouping, reduction과 operand 순서는 바꾸지 않았다. dynamic input 처리와 취소 지점의 runtime charge도 유지했다.

## R02

`segment_shape`는 aggregate segment마다 Rust call stack을 하나씩 사용했다. AST 파괴를 막아도 depth 2,048/worker stack 2 MiB에서 컴파일 중 abort했다.

segment 생성 규칙상 aggregate child segment는 parent보다 뒤에 있다. 따라서 segment 배열을 뒤에서 앞으로 한 번 순회하며 child `ExecutionShape`를 읽는 bottom-up 분석으로 바꿨다. dependency가 아직 계산되지 않았거나 root가 없으면 typed compile error를 반환한다. scratch vector 저장 공간은 `Option<ExecutionShape>` 크기의 보수적 두 배를 allocation 전에 admission한다.

지원 depth에 새 cap을 넣지 않았다. 기본 limit 64는 64를 승인하고 65를 typed `BudgetExceeded`로 거부한다. 사용자 확장 limit 513은 depth 512 중첩을 정상 컴파일·평가한다. 재감사 depth 2,048 probe도 같은 2 MiB worker에서 정상 반환했다.

## R03

초기 exact-file, predecessor archive, Git object 검색에서는 4,460-byte PLAN이 나오지 않았다. 이어 local Codex session 기록을 검색해 다음 원본을 찾았다.

- session: `/home/acr0tic/.codex/sessions/2026/09/13/rollout-2026-09-13T15-28-44-01a09974-34e9-7cd1-ae6e-9274421f9c45.jsonl`
- line: `1075`
- JSON pointer: `/payload/item/changes//mnt/f/work/simulator/core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/checkpoint_001_f01_f04/PLAN.md/content`
- recovered: `evidence/r03/HISTORICAL_PLAN_PRE_FIX_001.md`, 4,460 bytes, SHA-256 `661dd456d923e323cfd5704dd520e322251ba9a603303a4b73c295e6c13412c7`
- raw provenance: `evidence/r03/PROVENANCE_APPLY_PATCH_EVENT.jsonl`, 11,485 bytes, SHA-256 `7d5e1a6e3ca15eef397f95717ca0db0f1454884e5e57c3f0b5123b4632f97e79`

`R03_HISTORICAL_PLAN_RECOVERY.json`은 predecessor baseline을 바꾸지 않고 successor가 실제 bytes를 보완했다는 세대 관계를 기록한다. pre-fix-002 baseline의 `historical_r03_plan_bytes_preserved=false`도 캡처 당시의 사실로 보존한다. 따라서 이전 baseline 자체의 보존 결함은 소급 PASS로 바뀌지 않으며, 현재 target verifier는 recovered historical snapshot과 current live record를 별도 대상으로 검증한다.

## 실패와 재실행

- `pre_fix_reaudit_probe`: exit 101. R01 두 테스트가 예상대로 실패했고 나머지는 8 pass / 1 ignored였다.
- `pre_fix_deep_shape_probe`: subprocess exit -6. AST를 `ManuallyDrop`한 상태에서 native stack overflow/abort를 재현했다.
- `intermediate_unit_invalid_resource_fixture`: exit 101. 새 경계 테스트가 128-byte `ResourceId` 자체 계약을 넘는 fixture를 만들었다. 제품 동작 실패가 아니므로 identifier를 유효한 100 bytes로 고친 뒤 동일 unit과 전체 검증을 통과했다. 이 실패 log/receipt는 삭제하지 않았다.
- 이 pass에서는 sandbox 전체 테스트를 일부러 실패시키지 않았다. predecessor evidence의 Unix socket EPERM 환경 제약을 재사용하고, 전체 타깃 테스트는 승인된 환경에서 직접 실행해 통과시켰다.

## 남은 제약

- `runtime_dynamic_work=true` plan의 실제 frame/resolved payload는 compile 시 알 수 없으므로 runtime meter가 계속 제한한다. `worst_case_work`는 구조적 최소값이 아니라 승인된 실행 ceiling이다.
- 메모리 모델은 kernel이 추가로 확보하는 저장 공간의 보수적 누적량이며 RSS 보장이 아니다.
- R03의 원본은 복구되었지만 predecessor baseline 생성 당시 실제 bytes를 archive하지 않았다는 역사적 사실은 남는다. 새 baseline/target은 변경 문서의 actual-byte archive로 절차를 보강했다.
- 실제 lease 검증과 다음 R06 기능은 이 correction 범위 밖이다.
