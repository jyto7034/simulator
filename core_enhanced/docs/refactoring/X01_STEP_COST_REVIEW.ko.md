# X01-P2 step 내부 비용·사건 종류와 후속 설계

작성·갱신일: 2026-10-18

**선택한 step 내부 비용 조사·설계 판단을 완료했다. 제품 변경과 성능 수용은 별개이며 기존 기준 초과는 남아 있다.**
마지막 완료 lane의 step에서 사건 처리 비중 중앙값은 **79.0%**, 후보 복사 구간은 **8.8%**였다.
사건 종류를 추가로 나누면 AttackStart dispatch가 step의 **34.1%**로 가장 컸다.
다음은 공격 시작 handler와 공통 dispatch 정리의 비용을 구분하는 S10-P2다.

## 1 범위·기존 근거와 실행

[README](README.md)의 X01-P2 지시, [X01-P1 §4–5](X01_CONCURRENT_COST_REVIEW.ko.md#4-실행-경계와-설계-품질),
[기준](../refactoring_criteria.ko.md) §7.3–7.4와 [실행 지침](../system_refactoring_execution_guide.ko.md) §5.3–6을 적용했다.
[S01-P2](S01_TEMPLATE_BORROW_IMPLEMENTATION.ko.md)의 동작/구조 결과와 성능 수용 보류도 유지한다.

[기존 직접 입력 15개](evidence/step_cost_001/reused-evidence.json)가 X01-P1과 일치했다.
동일 64-session / 8-lane / 100×10ms fixture·affinity·출력 identity와 이전 원자성 조사를 재사용했다.
[관련 입력 6개와 기존 테스트 2개](evidence/step_cost_001/reused-ownership-tests.json)는
실제 range row 재사용·삽입 후 순서, 실패 timestamp의 성공 prefix 보존·재시도를 확인한 근거다.
제품 경계가 그대로여서 이 테스트와 전수·allocator·event-heavy 부하를 다시 실행하지 않았다.

제품·일반 도구·manifest·cap을 수정하지 않았다. [임시 복사본 준비](evidence/step_cost_001/prepare.py)에서
Core와 기존 concurrent 진단에만 구간 타이머·고정 크기 TLS 합계를 삽입했다.
[계측 diff](evidence/step_cost_001/instrumentation.diff)·[치환 목록](evidence/step_cost_001/instrumentation-replacements.json)을
역으로 적용하면 원래 본문과 정확히 같고, [고정 입력 411개](evidence/step_cost_001/fixed-inputs.json)도 현재 작업 트리와 일치한다.
계측 코드는 제품 소스나 feature에 설치하지 않았다.

초기 빌드는 도구가 include하는 보존 검증 소스의 임시 복사 누락으로 실패했다.
[원시 실패](evidence/step_cost_001/build-initial.log)와 [교정](evidence/step_cost_001/correction.json)을 보존했다.
이를 수정한 [1단계 실행](evidence/step_cost_001/execution-stage1.json)은 build 84.79s·진단 1.67s·exit 0이었다.
사건 처리 78.1%를 확인한 뒤 그 구간만 사건 종류별로 나눈 [최종 실행](evidence/step_cost_001/execution.json)은
build 82.63s·진단 1.72s·exit 0이었다. 각각 warm-up 후 21개 표본이며 제품 테스트 반복은 없다.

1단계 [raw](evidence/step_cost_001/inspection-stage1.json)·[대조](evidence/step_cost_001/validation-stage1.json)와
최종 [raw](evidence/step_cost_001/inspection.json)·[대조](evidence/step_cost_001/validation.json)를 구분한다.
아래 통계는 최종 계측의 21개 표본이다. 두 계측 사이 시간 차이를 성능 개선으로 비교하지 않는다.
진단 바이너리는 임시 위치에 보관하고 일반 도구 실행 파일은 이전 plain binary와 같은 hash로
[복원 확인](evidence/step_cost_001/binary-restoration.json)했다.

## 2 측정 경계와 동등성

각 worker의 초기화가 끝난 뒤 step loop에서만 수집한다. `runtime_timestamp_reusing`의 연속 구간,
`runtime_transaction`, 사건 없는 clock 갱신을 구분한다. [배열 인덱스·구간 정의](evidence/step_cost_001/phase-schema.json)를 연결한다.
`Span` 소멸 시 합산해 commit 뒤 기존 local cleanup도 담는다. 바깥 scratch buffer 폐기는 step 잔여에 남는다.
이 시간은 **wall time**이며 타이머·TLS·scheduling 비용을 포함한다. CPU 시간·할당량·예산 연료와 동일시하지 않는다.

같은 표본의 마지막 완료 lane에서 timestamp 구간 합 + clock 시간 + 잔여가 step 시간과 정확히 같다.
병렬 lane 시간은 합해 전체 지연으로 쓰지 않는다. 내부 사건 시간도 상위 event_operation에 이미 포함되므로 중복 합산하지 않는다.
중첩 dispatch는 가장 바깥 호출 시간에 포함하고 별도 호출 수만 센다. 실제 관측에서는 중첩 dispatch가 0이었다.

두 진단의 모든 표본에서 기존 **11,840 log events / 3,881,088 bytes / SHA `86cb93a7…42fbf`**를 유지했다.
현재 reference는 X01-P1의 계측 없는 출력이며, 새 latency 기준선이나 frozen P01 parity가 아니다.
최종 계측 전체 p95 **12.819568ms**는 진단값이다. 계측 없는 기본 cap 판정을 대체하거나 통과를 주장하지 않는다.

## 3 비용과 실제 실행 횟수

| 마지막 완료 lane의 step 내부 구간 | 21개 표본 중앙값 | 같은 표본 step 대비 비중의 중앙값 |
| --- | ---: | ---: |
| native preflight: retained extent·clone-shape admission | 0.294405ms | 2.92% |
| lineage 확인 + candidate clone/reuse | 0.920047ms | 8.82% |
| execution cursor clone + resource/log 준비 | 0.006641ms | 0.07% |
| 사건 처리 operation | **8.217298ms** | **78.98%** |
| action 정리 + retained 검사 + poll | 0.107148ms | 1.12% |
| Core/cursor commit + local cleanup | 0.253012ms | 2.47% |
| 사건 없는 clock 갱신 | 0.019712ms | 0.21% |
| 바깥 loop·budget 진입·scratch 폐기·계측 등 잔여 | 0.531248ms | 5.21% |

각 분포의 중앙값은 더하지 않는다. 실제 전체 p95 표본 11·lane 7에서는 위 구간이 차례로
0.325687 + 1.027268 + 0.007248 + 8.955730 + 0.130481 + 0.306955 + 0.022128 + 0.579693
= **11.355190ms step**을 구성한다. 전체 12.819568ms에는 초기화·encoding 등 step 밖 비용도 포함된다.

64 session 전체에서 매 표본 **timestamp 1,344회·clock-only 5,120회**, step 중 `runtime_transaction`은 0회였다.
100회 API 호출마다 Core를 복사한다는 설명은 맞지 않는다. timestamp candidate는 모두 fresh clone 1,344회였고
units buffer 재사용은 0회였다. 기존 buffer의 실패/계보 보장이 잘못됐다는 뜻은 아니며,
이 fixture에서 같은 호출 내 재사용이 발생하지 않았다는 관측이다. start의 transaction은 수집 범위 밖이다.

| root dispatch 종류 | 64 session 전체 호출 수 | 마지막 lane 시간 중앙값 | step 비중 중앙값 |
| --- | ---: | ---: | ---: |
| AttackStart | 6,144 | **3.553498ms** | **34.11%** |
| ContinuousMovementTick | 1,344 | 2.019725ms | 19.88% |
| AttackResolve / BasicHealResolve | 2,560 | 1.905529ms | 18.27% |
| SpawnGroup | 128 | 0.283936ms | 2.74% |
| dispatch 밖 사건 bucket·hooks·정리·계측 잔여 | 해당 없음 | 0.343974ms | 3.30% |

skill/flow·resonance/causal 등 나머지 root 분류는 이 fixture에서 0회였다. dispatch 수와 공개 log event 수는 서로 다른 값이다.
root dispatch 시간에는 `process_owned_event`의 공통 sustained/field-work/causal 정리와 local 소멸도 포함된다.
따라서 AttackStart의 34.11% 전체를 targeting·공격 계산·handler 한 함수에 귀속하지 않는다.
마찬가지로 movement dispatch에는 Rapier backend의 지연된 copy-on-write도 들어갈 수 있다.
**상위 candidate 복사가 8.82%라는 결과는 모든 복사 비용의 합이 8.82%라는 뜻이 아니다.**

## 4 설계 품질 판단과 보존 조건

| 경계 | 판단 |
| --- | --- |
| timestamp 후보·실행 cursor | 유지. preflight와 사건 실행 뒤 검증은 서로 다른 상태를 보호한다. 오류·성공 prefix·계보를 포기할 비용 근거가 없음 |
| no-event 경로 | 유지. Core를 빌리고 cursor만 확정하는 기존 분리는 실제 호출에서도 사용됨 |
| units scratch | 현 수명 유지. `BattleCore.units`는 공개 map이며 caller가 호출 사이 바꿀 수 있음. 호출 사이 cache로 늘리려면 계보·교체·실패 폐기를 새로 검증해야 함 |
| 공통 dispatch | source 교체/복원·delivery 종료·causal/sustained/field-work 정리는 실제 실행 의미. 비용이 여러 사건에 반복된다는 이유만으로 합치거나 생략하지 않음 |
| AttackStart | 다음 비용 owner. handler의 gate·재예약·target 선택·finite action/source·snapshot 생성과 공통 정리를 구분해야 최소 변경을 고를 수 있음 |
| movement backend | [S09](S09_MOVEMENT_BLOCKING_REVIEW.ko.md)의 Arc 공유→쓰기 전 controlled clone과 block/body 반영 순서 유지. 현재 결과로 Rapier clone·physics·planner 각각을 주원인이라고 확정하지 않음 |

[공격 경계 입력](evidence/step_cost_001/attack-owner-inputs.json)과 [이동 owner 4개](evidence/step_cost_001/movement-owner-inputs.json)를 대조했다.
과거 S10 조사 시점 `sim.rs` hash는 이후 R1 변경 전 것이어서 재사용하지 않았고, 현재 X01-P1 hash를 사용했다.
이 범위에서 새로운 gameplay 오류를 재현하거나 필수 제품 수정 후보를 확정하지 않았다.

`try_start_pending_basic_attacks`와 AttackStart의 target 선택은 동일 대상을 반복해도 무조건 중복은 아니다.
예약 뒤 실행 시점 사이 상태가 바뀔 수 있고, `schedule_next`의 target 재평가·잠금/CC/재예약 의미가 다르다.
AttackStart 6,144회와 resolve 2,560회의 차이만으로 남는 호출을 삭제 가능한 중복으로 판단하지 않는다.

## 5 완료 검토와 다음 행동

[분리 검토](evidence/step_cost_001/review.json)는 현재 소스·계측 치환·원시 출력·단계 산술·수명·후속 조건을 대조한
동일 작성자의 읽기 전용 pass다. 비참여 독립 감사가 아니다.
이번 완료는 **step 비용의 내부 귀속과 후속 설계 경계 선정**이다. 제품 성능 개선·성능 수용·전체 콘텐츠 조사 완료가 아니다.
S01-P2 지연 판정 보류, snapshot 시간/크기, concurrent cap 초과와 S01-C1/C2/P3는 열린 채 유지한다.

다음은 **S10-P2 AttackStart 내부·공통 dispatch 비용 분리와 개선 설계**다.

1. [sim.rs](../../src/game/battle/core/sim.rs)의 `process_owned_event`에서 공통 전처리·실제 handler·공통 후처리를 구분한다.
2. [basic_attack.rs](../../src/game/battle/core/basic_attack.rs)의 `handle_basic_attack_start_inner`에서
   gate/재예약, target 선택, finite action/source 준비, damage snapshot·로그/queue 구간을 조사한다.
   실행 횟수와 조기 반환 의미를 함께 보고, 반복 scan·소유 사본 등 실제 원인이 확인된 최소 후보만 선택한다.
3. 현재 fixture·출력 identity와 source/ticket·재예약·target 재평가·실패 원자성을 보존한다.
   이미 확인한 worker/step 총량·전체 기능 검사는 반복하지 않는다. 제품 변경을 선택할 때 기존 S10 영향 검증과 plain 전후 비교를 배정한다.

기존 인계의 `step_until_budgeted_inner` 표기는 실제 함수명 `step_battle_execution_until_inner`로 바로잡는다.
새 범용 transaction 계층·전역 cache·worker pool·tick 수 축소·cap 증액을 선정하지 않았다.
