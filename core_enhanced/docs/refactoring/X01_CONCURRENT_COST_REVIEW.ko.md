# X01-P1 동시 전투 부하의 단계별 비용과 설계 판단

작성·갱신일: 2026-10-18

**64-session 부하의 단계별 비용 귀속·설계 검토 완료. 성능 기준 초과는 해결하지 않았다.**
전체 완료를 결정한 마지막 lane에서 step 구간이 wall time의 83.5–90.9%, 중앙값 89.3%를 차지했다.
초기화·worker 시작·로그 직렬화를 주원인으로 보고 구조를 바꿀 근거는 부족하다.
step 내부의 복사·예산 scan·사건 처리 비중은 아직 구분하지 않았으며, 원자성을 없애는 최적화는 선정하지 않는다.

## 1 범위·기준·기존 근거

[README](README.md)의 다음 행동과 [snapshot 조사 §4](S01_SNAPSHOT_COST_REVIEW.ko.md#4-설계-판단후속-작업의-경계)를 이어 수행했다.
[기준](../refactoring_criteria.ko.md) §3.1–3.3·7.4, [실행 지침](../system_refactoring_execution_guide.ko.md) §5.3–6·9,
[runtime 성능 계약](../core_runtime_contract.md#final-core-performance-guard)을 적용한다.
기존 10ms cap·표본수·seed·입력·출력 identity와 실패 의미를 유지한다.

[직접 입력 대조](evidence/concurrent_001/reused-evidence.json)의 도구·설정·runtime·기준 12개가 이전 근거와 일치했다.
[1차 측정](PERFORMANCE_RESOURCE_MEASUREMENT.ko.md)의 concurrent plain 63개·allocator 5개 출력과
[복사·결정론 조사](X01_COPY_DETERMINISM_OBSERVATION_REVIEW.ko.md)의 원자성·buffer 계보 검토를 재사용했다.
과거 frozen P01과 현재 출력 hash가 다른 상태는 유지하며 과거 대비 회귀 여부를 새로 판정하지 않는다.

이번에 바꾼 것은 기존 final_performance의 명시적 `--inspect-concurrent <output.json>` 분기와
[별도 진단 모듈](../../tools/runtime_architecture_final_validation/src/concurrent_inspection.rs)이다.
기본 verifier·기존 `--measure-current`·원래 concurrent_sample·공유 fixture 함수는 변경하지 않았다.
[diff](evidence/concurrent_001/tool.diff)와 [입력/산술 대조](evidence/concurrent_001/validation.json)에서 본문 보존을 확인했다.
제품 소스·공식 데이터·manifest·lock·성능 cap은 이번에 수정하지 않았다.

## 2 진단 방법과 유효 범위

기존 fixture 생성/검증 후 한 publication을 모든 Core에 공유한다.
64 session을 8 lane에 고정 분배하고 lane마다 8개 Core를 생성·start한 뒤,
100회에 걸쳐 각 session을 10ms씩 진행한다. 각 session은 양쪽 8개씩의 unit을 가진 9×9 시나리오다.
coordinator CPU30·lane CPU0/2/4/6/8/10/12/14, plain release 조건을 유지했다.

기존 concurrent_sample 1회를 **출력 reference**로 실행하고, 진단 warm-up 1회 후 21개 표본을 수집했다.
reference 1개는 새로운 latency 분포나 회귀 판정의 근거가 아니다.
event-heavy·allocator·전체 기능 테스트·다른 feature는 이번 질문에 필요하지 않아 반복하지 않았다.

진단 orchestration만 타이머가 추가된 별도 경로이고 fixture·factory·실행 API·step 수·직렬화 방식·session 정렬·digest는 원래 경로와 같다.
worker pool·barrier를 추가하거나 작업을 측정 구간 밖으로 이동하지 않았다.
진단과 계측 없는 reference, 이전 68개 표본의 **11,840 events / 3,881,088 bytes / SHA `86cb93a7…42fbf`**가 일치한다.
이는 현재 부하의 출력 동등성이지 frozen P01 parity 또는 모든 실행 경로의 증명은 아니다.

구간은 lane 진입까지의 offset, affinity 설정, 초기화, step loop, encoding/cleanup, coordinator tail로 나눴다.
초기화 안의 construct/start와 encoding 안의 JSON 직렬화도 따로 관측한다.
encoding/cleanup에서 직렬화 시간을 뺀 값은 Core 소멸뿐 아니라 iterator·수집·타이머 비용을 포함하므로 순수 destructor 비용이라고 하지 않는다.
digest 계산은 원래 부하처럼 elapsed timer 뒤다. fixture 변환도 밖이다.

시간은 CPU 사용량이 아닌 **wall time**이며 scheduling/preemption을 포함한다.
lane들은 서로 겹치므로 여덟 lane의 시간을 합해 전체 지연이라고 하지 않는다.
매 표본에서 `end_offset_ns`가 가장 큰 lane을 고르고 그 경로와 이후 coordinator tail을 연결했다.
spawn loop와 join 대기 구간도 겹치는 작업을 포함한다. join interval을 별도 추가 병목으로 더하지 않는다.

[release build](evidence/concurrent_001/build.log)는 89초·exit 0,
[진단 실행](evidence/concurrent_001/execution.json)은 1.72초·exit 0이었다.
[raw 표본](evidence/concurrent_001/inspection.json)은 `performance_pass_claimed=false`, `baseline_comparison_applicable=false`다.
exit 0은 진단·출력 대조 성공이며 기존 cap 충족이 아니다.

## 3 실제 결과

진단 전체 p95는 **12.099306ms**, median은 10.053616ms였다.
타이머·trace 운반이 추가된 값이므로 기존 plain p95 12.188723 / 11.789399 / 13.473684ms와 비교해 개선/퇴행을 선언하지 않는다.
기존 cap 초과 판단을 지우거나 새 승인 baseline으로 삼지 않는다.

| 마지막 완료 lane의 구간 | 21개 표본 중앙값 | 해석 |
| --- | ---: | --- |
| worker 진입 offset | 0.320166ms | coordinator 시작 이후 해당 worker가 진입하기까지. 순수 thread 생성 시간은 아님 |
| affinity 설정 | 0.033790ms | pin 호출과 해당 구간 scheduling 포함 |
| 8 session 초기화 | 0.174841ms | construct 합 0.058822ms, start 합 0.114895ms. 각각의 중앙값은 단순 합산하지 않음 |
| 8 session × 100회 step | **8.969930ms** | 공개 step API의 예산·후보·사건 처리 및 지연된 최초 spawn 포함 |
| 로그 encoding + Core 정리 | 0.479489ms | 그중 JSON encoding 합의 중앙값 0.381001ms |
| 마지막 lane 이후 coordinator tail | 0.054012ms | thread 종료·join/수집/정렬·scope 반환 등 남은 비용 |

step의 전체 지연 대비 비중은 표본별 83.5226–90.9308%, 중앙값 89.2533%다.
각 행의 중앙값을 더한 수치는 전체 중앙값과 같을 필요가 없다.
coordinator spawn loop 중앙값은 0.173218ms, join interval 9.868567ms, 정렬 0.000942ms다.
join 대부분은 lane 작업과 중첩된 대기이며 별도 9.87ms 작업으로 해석하지 않는다.

합산 가능한 예로 전체 p95에 해당한 **표본 14·마지막 lane 7**은 다음과 같다.

| 동일 표본의 연속 경로 | 시간 |
| --- | ---: |
| worker 진입까지 | 0.375139ms |
| affinity | 0.026646ms |
| 초기화 | 0.150375ms |
| step | 10.846130ms |
| encoding + 정리 | 0.646944ms |
| coordinator tail | 0.054072ms |
| **합계** | **12.099306ms** |

이 표는 별도 구간 p95들의 합이 아니라 같은 실제 표본의 분해다.
모든 표본에서 phase 합·offset 순서·구간 안의 부분 측정·출력 parity를 대조했다.

## 4 실행 경계와 설계 품질

| 경계·후보 | 확인한 사실 | 판정 |
| --- | --- | --- |
| publication·worker 상태 | [factory](../../tools/runtime_architecture_final_validation/src/final_performance.rs)는 한 ValidatedGameData를 공유하고 Core·execution cursor는 session별 소유. worker는 서로 다른 session을 진행 | 유지. 정적 데이터 재검증·공유 mutable Core가 주원인이라는 근거 없음 |
| 초기화·worker pool | construct/start가 마지막 lane에서 중앙값 약 0.175ms. 이 부하는 새 worker를 만드는 진단 harness이며 실제 host scheduling 구조를 측정하지 않음 | 초기화 cache·worker pool을 제품 개선으로 선정하지 않음. pool로 바꾼 수치를 기존 cap 통과로 대체하지 않음 |
| 직렬화 | 전체 로그가 약 3.88MB이지만 encoding/정리는 마지막 lane 중앙값 약 0.479ms. [event_log.rs](../../src/game/battle/event_log.rs)의 owned entries는 Arc 공유, wire는 version/entries | 로그 필드·검사·digest 제거로 기준을 맞추지 않음. serializer 변경을 최우선으로 정할 근거 부족 |
| step 진입 | [sim.rs](../../src/game/battle/core/sim.rs)의 `step_battle_execution_by` → until → run_profile_budget → inner. 사건이 없으면 Core는 borrow하고 execution cursor만 후보로 확정 | 유지. 모든 10ms 호출이 전체 BattleCore를 복사한다는 가정은 틀림 |
| 사건 timestamp | [runtime_facts.rs](../../src/game/battle/core/runtime_facts.rs)의 preflight → 후보 생성/같은 call의 units buffer 재사용 → 사건 실행 → retained 확인/poll → Core와 cursor 동시 commit | 실패 의미와 clone/work admission·순서/계보를 보존. 높은 step 시간만으로 copy가 원인이라고 확정하지 않음 |
| typed ResourceStore | 현재 fixture의 resource catalog는 빈 목록이며 prepare_runtime_resources는 비어 있으면 store를 만들지 않음 | 이 결과로 populated store의 중첩 transaction 비용·제거 필요를 판단하지 않음 |

이번 조사에서 새로운 동작 오류를 재현하거나 step 내부의 특정 구조 결함을 증명한 것은 아니다.
**확인한 원인 범위는 step 단계까지**다. 초기 spawn·사건 처리·clone/scan·retained 확인·큐 처리와 scheduling의 세부 비중은 남아 있다.
같은 call 안의 buffer 재사용을 call 사이로 늘리려면 수명·삽입 이력·실패 폐기·authoring 동작까지 바뀌므로 국소적인 시간 최적화로 취급하지 않는다.

## 5 후속 순서와 완료 검토

현재 실제 내부 변경 후보 중에는 [S01-P2 template lookup](S01_SNAPSHOT_COST_REVIEW.ko.md#s01-p2를-선택할-경우의-구체적-구현-단위)의
불필요한 소유 사본·보존 조건·작은 검증 범위가 이미 확인돼 있다.
**다음은 S01-P2의 borrowed lookup 구현·영향 검증·분리 검토**로 정한다.
이는 snapshot/동시 실행 시간 숫자를 서로 비교한 우선순위가 아니라, 현재 근거와 변경 위험·완결 가능한 내부 책임에 따른 선택이다.
실제 전후 비용은 구현 시 확인하며 이 변경으로 snapshot byte cap 또는 concurrent 10ms가 해결된다고 가정하지 않는다.

X01의 성능 finding은 **X01-P2 step 내부 비용 귀속**으로 열린 채 유지한다.
이후 해당 성능 개선을 선택하면 같은 fixture에서 native 후보 준비·사건 실행·retained/commit 구간을 구분하는 최소 관측을 사용한다.
기존 총량 측정을 다시 반복하거나 연료 카운터를 CPU 시간으로 대체하지 않는다.
실제 비용 owner가 확인된 뒤 최소 구조 변경을 정하고, 기존 실패/계보/결정론 검증과 계측 없는 동일 부하의 전후 비교를 배정한다.
현재 원자성 검사 제거, thread 수/step 수 축소, cap 변경은 선정하지 않았다.

S01-C1 응답/크기 예산과 S01-C2 외부 계약·Unity reader 불일치는 기존 기록대로 유지한다.
이는 내부 borrowed lookup의 선행 조건은 아니며 공개 schema 전환 전에 해결해야 한다.

별도 읽기 전용 pass에서 원래 두 실행 모드 보존, 원시 표본·68개 이전 identity와의 대조,
병렬 시간 해석, 실제 호출·상태 소유와 후속의 범위를 확인했다.
[검토](evidence/concurrent_001/review.json)와 [문서 확인](evidence/concurrent_001/verification.json)에 연결한다.
동일 작성자의 분리 검토이며 비참여 독립 감사가 아니다.

완료는 **단계별 비용 조사·도구 추가·설계 판단** 범위다. 성능 수용·step 내부 원인 확정·제품 개선 완료가 아니다.
추가 전체 테스트·allocator·Unity/서버 실행·모든 콘텐츠/최대·장기 부하 검증을 수행했다고 주장하지 않는다.


### 후속 내부 비용 조사 (2026-10-18)

S01-P2 구현 후 [X01-P2 step 내부 조사](X01_STEP_COST_REVIEW.ko.md)를 수행했다.
임시 계측에서 사건 처리는 step의 79.0%, candidate 복사 구간은 8.8%였고,
root AttackStart dispatch가 step의 34.1%로 가장 컸다. 이는 공통 전후 정리가 포함된 구간이다.
제품·일반 도구 소스와 기존 cap은 유지했다. 다음은 [README](README.md)의 S10-P2 공격 시작/공통 정리 비용 분리다.
성능 수용·S01-P2 지연 판단·snapshot/concurrent 기준 초과는 미완료로 유지한다.
