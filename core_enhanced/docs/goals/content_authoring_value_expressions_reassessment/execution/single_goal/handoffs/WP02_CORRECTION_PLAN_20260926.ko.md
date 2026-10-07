# R06 WP02 교정 설계·구현 인계 계획

작성일: 2026-09-26 KST  
상태: **교정 계획 작성 / 제품 구현 미착수 / 단계검토 옵션 유지**

## 1. 목적·권한·현재 상태

기존 WP02 감사 F01~F03와 PC 부하 측정에서 관측한 두 capacity 문제를 함께 다룰 수정 방향이다. 사용자의 이번 요청은 교정 계획 정리이며, 이 문서 작성 자체로 제품 구현·예산 상향·다음 WP 진행을 승인한 것은 아니다. 실제 교정은 사용자가 구현을 지시한 뒤 새 baseline에서 수행한다.

기존 [후속 구현 인수인계](IMPLEMENTATION_RESUME_20260926.ko.md)의 단계검토·before/after 보존 규칙은 유지한다. 그 문서의 작성 당시 “검토 대기” 이후 [WP02 감사](../reviews/wp02_runtime_frame_capture_001_review_001/REVIEW.md)가 `CORRECTION_REQUIRED`를 판정했다. **순차 effects·prefix·WP03보다 이 교정과 재검토가 먼저**다. 과거 계획/manifest의 상태를 소급 수정하지 않는다.

계획 입력:

- [동결 target manifest](../segments/wp02_runtime_frame_capture_001/target/target_001/MANIFEST.json): SHA-256 `d3dbf3ba7c3c3be5a584db3092e7059c116d92b1ae812740f1fb68ccb05c2fc3`.
- [예산 범위 조사](../../../../../tmp/R06_PC_WORKLOAD_AND_BUDGET_AUDIT_20260926.ko.md).
- [수정 전 PC 측정 보고서](../../../../../tmp/r06_pc_baseline_20260926/REPORT.ko.md), [측정 원본 identity](../../../../../tmp/r06_pc_baseline_20260926/source_identity.json), [측정 자료 해시](../../../../../tmp/r06_pc_baseline_20260926/ARTIFACTS.json).
- 측정 자료 `ARTIFACTS.json` SHA-256: `b77a5096e4b67b3775ff29397559c1b160d2fe223ff6b3e7d748e2604cf1b85d`.

PC 대상, 아군 12명·적 수백 명이 기준이다. 모바일은 제외한다. 100/300/500명·25×25 타일은 측정 fixture이며 제품 상한이 아니다. 파고드는 천국의 공간 상한은 생성/점유 가능한 타일 수라는 사용자 설명을 유지한다.

## 2. 교정 항목과 우선 판단

| ID | 문제 | 핵심 방향 | 닫힘을 확인할 증거 |
| --- | --- | --- | --- |
| WP02-F01 | frame/evaluator 작업 후 호스트 비용 청구, 평가 메모리·중간 취소 미연결 | 기존 evaluator의 비용 경계에 호스트 admission을 연결 | 작은 잔여 예산에서 실제 계산·할당 전에 거부, 중간 취소, 누락/중복 없는 계측 |
| WP02-F02 | 유효 action과 actor/source-copy의 결속 누락 | action/delivery 소유자가 가진 실행 provenance를 검증 | actor·equipment copy 바꿔치기 거부, 합법적인 source/child 관계 유지 |
| WP02-F03 | 비활성 unit의 Live resource 읽기 허용 | expression의 시점 가용성 검사와 resource 저장 수명 분리 | 사망·퇴장 후 Live 거부, 적법한 Snapshot 및 영속 자원 유지 |
| CAP-01 | 300명 일괄 피해의 `actor_row_reserve` 거부 | ActorSummaries의 한 칸씩 재할당을 bounded 성장으로 개선 | 같은 피해 fixture의 완료 여부, 누적 요청·extent·COW·rollback 검증 |
| CAP-02 | 500명 동시 생성의 `queued_event_slots` 거부 | 기존 heap 순서를 보존하는 bounded 용량 성장 | 같은 생성 fixture의 완료 여부, queue 순서·한도 경계·실패 전 상태 유지 |

CAP-01/02는 이번 측정의 **capacity 관측 ID**다. 기존 감사 finding을 새 번호로 바꾸거나 두 문제가 이미 독립 감사 PASS/FAIL을 받은 것처럼 표현하지 않는다. 36개 측정 조건 중 6개 거부는 CAP-01의 2개와 CAP-02 공통 setup의 4개다.

고정할 판단:

1. 안전성 교정과 정상 부하 수용력 개선을 구분한다. 빠르게 계산되더라도 F01이 남으면 안전성 미완료다.
2. 현재 기본 한도·계산 의미·취소·첫 fault 유지·transaction 경계를 먼저 유지한다. 숫자 상향으로 실패를 숨기지 않는다.
3. `admit_retained_growth(0, new_array_bytes)`를 단순히 `new-old` 청구로 바꾸는 것을 정답으로 삼지 않는다. 기존 누적 할당 admission 의미가 바뀌는지 먼저 증명해야 한다.
4. 권장 개선은 **실제 재할당 횟수와 불필요한 복사를 줄이는 것**이다. 기록/피해/대상/trace를 생략하거나 가상 수식으로 교체하지 않는다.

## 3. F01 — 비용 경계와 호스트 연결

### 3.1 구조 선택

`ValueExpr`의 단위·산술·집계·반올림은 기존 compiler/evaluator가 계속 소유한다. BattleCore 의존 계산기를 새로 만들지 않는다. evaluator의 비용 검사 지점에 작은 예산 연결 인터페이스(콜백/trait/제어 객체 중 구현자가 선택)를 두고, runtime adapter가 native/authoring host와 연결하는 방향을 권장한다. 순수 kernel 시험은 기존 local meter 방식으로 유지할 수 있어야 한다.

실행은 다음 순서를 만족해야 한다.

1. 살아 있는 바깥 operation의 fault/cancel 상태와 plan 제약을 확인한다. 중첩 호출에서 예산을 새로 시작하지 않는다.
2. frame 준비의 조회·순회·동적 복제 비용을 **해당 작업 전**에 검증/청구한다. `reference_node`의 plan 순회도 무료라고 간주하지 않는다. 읽기 전용 인덱스로 대체한다면 그 구축·보유 비용과 fingerprint 비변경을 확인한다.
3. evaluator의 work·member 준비·정렬·stack/task/trace 할당에 local 및 host 제약을 적용한다. 현재 활성화된 여러 host의 제약을 모두 지키되 단위가 다른 카운터를 단순 `min`으로 합치지 않는다.
4. 정적 요구는 사전 거부 근거로, 동적 비용은 실제 진행 전 admission 근거로 사용한다. plan의 최대 65,536 work를 매 평가마다 일괄 소모하는 식으로 정상 작은 식을 과대 청구하지 않는다.
5. 진행 중에도 bounded 간격으로 실제 호스트 cancellation/fault를 확인한다. 시작/종료 poll만 두거나 local meter의 복사된 bool만 검사하는 것으로 끝내지 않는다.
6. 기존 마지막 `charge(meter.usage().work, ...)`와 새 선청구가 중복되지 않도록 책임을 옮긴다. 실패 경로도 이미 소비한 예산을 환급하지 않는다. 원래 host fault와 ValueExpr site/node 진단을 보존한다.

호스트 scope가 없는 독립 kernel 시험과 실제 native/controlled 실행은 구별한다. 정상 production adapter 경로에서 예산 연결이 빠진 채 no-op 연결로 성공하지 못하게 검증한다. 자의적으로 evaluator 안에 매번 새 outer scope를 만드는 것은 금지한다.

### 3.2 메모리의 세 가지 의미

| 구분 | 유지할 의미 | 주의 |
| --- | --- | --- |
| per-expression retained | 한 평가에서 신규 저장/예약을 보수적으로 제한 | 기존 1 MiB를 제거하지 않음. 미리 보유한 frame·재사용 버퍼가 별도 owner에서 계산되는지도 확인 |
| operation cumulative admission | 해당 작업이 허용받은 새 할당/성장 비용의 누계 | 후보 폐기·해제로 allowance를 돌려주지 않음 |
| live owner extent | 현재 남아 있는 source/capture/frame/buffer/trace 등의 보유량 | 실제 할당된 spare capacity와 Arc 공유/COW 수명을 누락하지 않음 |

“평가 메모리 사용량” 한 숫자를 세 카운터에 그대로 더하는 방식은 금지한다. 먼저 무엇을 누가 생성하고 보유하며 반환 후 누가 해제하는지 적고, 실제 할당·예약과 검증용 상한을 구분해 계산한다.

같은 allocation을 frame/capture 생성 때와 evaluator가 읽을 때 중복 청구하지 않는다. 반면 함수 반환 후 살아 있는 `EvaluatedAmount.trace`를 이미 해제된 scratch처럼 취급해서는 안 된다. 새 할당이 없는 재사용이라면 실제 재사용이 성립하는 수명/소유권 증거가 있어야 한다.

반복 평가의 임시 버퍼 재사용은 필요할 때 고려하되, 기존 budget 의미를 바꾸지 않는 bounded owner 안에서만 한다. 서로 다른 action/source/target/generation의 값을 재사용하거나 전역·thread-local gameplay cache로 상태를 숨기지 않는다. clone·rollback·실패 후 재시도에서 오래된 frame/trace가 섞이지 않는 회귀가 필요하다. scratch 최적화만을 위해 다음 WP consumer를 먼저 연결하지 않는다.

이미 예산이 없는 상태에서도 진단 생성 자체가 큰 할당을 하지 않도록 bounded fault 경로를 확인한다. checked arithmetic과 작업 전 거부는 새 연결 코드에도 적용한다.

## 4. F02/F03 — identity와 시점 가용성

### F02

`runtime_facts` owner가 action의 actor·provider 정보를 기준으로 capture source를 검증하거나, 그 owner가 검증한 불변 context를 반환하도록 한다. handle이 유효하다는 사실과 handle/source가 서로 결속돼 있다는 사실을 별도로 확인한다.

- actor, 필요한 equipment definition/runtime instance/persistent copy, action/delivery 소속을 해당 실행 provenance와 대조한다. 같은 definition의 다른 copy는 같은 소유자로 취급하지 않는다.
- `persistent_copy_uuid=None`인 합법적인 scenario equipment를 무조건 invalid로 만들지 않는다. 기존 R04의 실제 identity 규칙을 따른다.
- effect별 authored metadata가 다를 수 있으므로 `RuntimeFactSource` 전체 동등 비교로 막지 않는다. 실제로 허용된 child/proc/delivery 관계만 인정하고 새로운 관계를 만들어 통과시키지 않는다.
- 검증용으로 mutable 소유권을 중복 저장하지 않는다. 공개 fact API·직렬화·action token의 신뢰 경계도 확장하지 않는다.
- Snapshot은 capture 당시 결속을 검사한다. 이후 source 퇴장 때문에 적법한 snapshot 값 사용까지 일괄 금지하지 않는다.

### F03

expression이 요구하는 Source/Target의 해당 시점 가용성을 **frame 읽기 경계**에서 검사한다. `SourceEmployee`, `TargetEmployee`, `SourceEquipment`의 주체를 정확히 해석하고, 실제 active unit context와 lease/owner/lifetime 검사를 모두 만족시킨다.

- Live Stat/HP와 Live Resource가 같은 가용성 원칙을 따르게 한다. inactive/missing이면 기존 typed fault로 거부하고 0으로 대체하지 않는다.
- Snapshot은 적법하게 capture한 값을 유지한다. Run/Attempt 자원을 사망 시 삭제해서 문제를 가리지 않는다.
- 공유 resource mutation resolver 전체를 active-only로 바꾸지 않는다. R05의 finite delivery·사망 이후 자원 보존/소비 정책과 분리한다.
- read 전후 resource revision·balance·reservation이 바뀌지 않아야 한다.

## 5. CAP-01/02 — 한 칸씩 확장을 bounded 성장으로

### 공통 권장안

현재 컨테이너 표현과 순서를 최대한 유지하면서, `len == capacity`에서 한 칸씩 확장하던 부분을 **checked·상한 있는 단계적 용량 성장**으로 바꾸는 방향을 우선 검토한다. 예를 들어 작은 시작 용량에서 배수로 성장시키되 합법적인 원소 상한과 남은 allocation allowance를 넘는 과잉 예약은 하지 않는다. 배수·helper 이름·정확한 구현 형태는 담당자가 근거를 기록해 선택한다.

- 과잉 예약만 한도를 초과하지만 꼭 필요한 최소 용량은 들어가는 경우를 구분한다. 실제 allocation 이전에 작은 합법 용량을 선택할 수 있어야 한다. 이미 fault를 latch한 뒤 무시하고 재시도하지 않는다.
- 새 buffer의 보수적 전체 요청량을 작업 전에 admission하는 기존 의미는 유지하는 방향으로 시작한다. old/new buffer 전환과 COW 비용을 숨기지 않는다.
- capacity 증가 후의 owner extent, clone scan/copy, 다음 operation의 seed, clear/release를 함께 갱신한다. **spare capacity가 늘어났는데 len만 계산하는 계측을 그대로 두지 않는다.**
- 반대로 clone이 실제 복사하지 않는 원본 spare capacity까지 매 clone 비용으로 청구하지 않는다. 원본의 live extent와 복제본의 실제 예약/복사 비용은 별개다.
- capacity/bytes/work 계산은 checked arithmetic. source·plan·trace·fact payload의 기존 비용을 누락하지 않는다. 한 번에 최대 허용량을 전부 예약하는 방식도 피한다.
- old/new allocation 요청·보유·clone이 예산에서 각각 어떻게 표현되는지 짧은 회계 표를 구현 기록에 남긴다. helper만 바꾸고 owner bookkeeping을 미루지 않는다.

### CAP-01: ActorSummaries

`core/runtime_facts.rs`의 정렬 rows 구조와 ID 순서는 우선 유지한다. capacity 성장 비용을 줄이되 insert shift work, 새로운 actor 한도, caused/received summary, Arc COW 및 latest fact 보유는 그대로 계산한다. 특히 `writable_actor_summaries`, native retained extent 및 clone_shape를 함께 점검한다. 다른 자료구조로 전면 교체하는 것은 첫 선택이 아니며 필요하다면 추가 설계로 반환한다.

### CAP-02: EventQueue

`core/queue.rs`의 기존 BinaryHeap과 comparator·동일 key 처리 순서를 유지한다. 성장 최적화를 위해 이벤트를 정렬·재삽입하거나 tie-break를 바꾸지 않는다. source/fact-source·delivery completion metadata, large payload admission, 첫 fault latch를 유지한다. 허용 실패 시 조용히 이벤트를 잃고 바깥 호출을 성공시키지 않는다.

queue slot extent와 live payload extent를 구분하고, enqueue/pop/clear/clone 및 timestamp candidate 교체에서 검증한다. pending/bucket의 authoring 제한과 native work/retained 제한도 각각 확인한다. native/controlled 한도가 같다고 가정하지 않는다.

## 6. 구현 범위·순서·중단

후속 명시적 구현 요청을 받으면 다음 순서로 접근한다. 이는 **미리 정한 강제 checkpoint 개수나 빌드 지점이 아니다**.

1. 진입 currentness와 actual-byte baseline 보존. 원본 감사 probe와 부하 baseline을 재현 가능한지 확인하고, 새 실행 PLAN에 F01~F03/CAP-01~02 및 변경 예상 파일을 기록한다.
2. 공통 admission·소유권의 회계 표를 정리한 뒤 F01 연결을 구현한다. F02/F03는 의미를 섞지 않는 독립 변경으로 교정한다.
3. CAP-01/02의 저장소 성장과 해당 extent/COW 계산을 교정한다. 공통 helper가 필요하면 F01과 같은 예산 의미를 사용하되 불필요한 전체 자료구조 프레임워크로 넓히지 않는다.
4. 의미 있는 책임이 닫히고 빌드·관련 검증이 가능한 지점에서 before/after와 미완료 항목을 고정하고 **최종 응답으로 중단**한다. 중간 단계검토가 필요하면 나머지 항목을 미완료로 명시하며, 다 끝나야만 멈출 수 있다고 해석하지 않는다.
5. 교정 타깃 재검토와 사용자의 다음 구현 요청 전에는 production consumer/prefix 연결로 넘어가지 않는다.

예상 변경 경계: `game/value_expr/{mod,eval,compile}.rs` 중 필요한 부분, `battle/core/{value_expr_runtime,runtime_facts,queue}.rs`, `battle/runtime_control.rs`, 필요 시 `authoring/{control,clone_shape}.rs` 및 직접 관련 회귀 테스트. `resource_runtime.rs`는 F03의 읽기/변이 책임 분리에 꼭 필요한 경우만 검토한다. 경계 밖 변경이 필요하면 근거를 먼저 기록하고 의미 확장에는 승인을 요청한다.

제외: 게임 스킬 재설계, 적/타일 제한 신설, 새 전투 scheduler, R07 이후 기능, 전체 컨테이너 교체, 서버/Unity/wrapper/Windows 이식, publication 해제, GEX-20/30 및 WP/Phase/Goal 완료 선언.

## 7. 재검증 조건

| 항목 | 필수 확인 |
| --- | --- |
| F01 안전 경계 | native/authoring 각각과 중첩 활성 상태, 양수 예산을 소진한 잔여 0, limit−1/limit/limit+1, frame 이전/평가 중 취소, 실패 후 다음 operation 정상 재시도 |
| F01 실제 작업량 | 기존 128 Read 조기 거부 probe; 요청 allocation이 전체 평가 후 거부 패턴으로 남지 않음. retained 1 byte 상태에서 비인가 평가 성공 금지. diagnostic 최소 비용과 실제 본 작업 비용 구분 |
| F01 의미 | 값·단위·rounding·overflow·grouping·필수 trace·fingerprint 및 literal 결과 유지. 비용 진단 변화는 계산 결과 변화와 분리 |
| F02 identity | actor 바꿔치기, 동일 definition의 다른 copy, 유효하지만 다른 action/delivery, 합법적 child 및 persistent copy 없는 합법 사례 |
| F03 lifetime | SourceEmployee/TargetEmployee/SourceEquipment × active/dead/withdrawn/missing × Snapshot/Live; Run/Attempt/Deployment 및 기존 finite delivery 회귀 |
| capacity | 0/초기/성장 직전/직후/원소 상한, tight byte/work limit, overflow, spare capacity, COW 첫 write, clone·drop·retry, 동일 key queue 순서 |
| rollback | 실패 시 state/HP/resource/RNG/identity/queue/log/prefix와 scope fault 보존. BattleCore timestamp prefix와 GameCore application 경계를 구분 |
| 비용 회귀 | 보존된 R05 fixture/cap 9,030,188·기존 실패 기록 그대로 비교; 새 PC baseline은 별도 series. requested sum·동시 allocation peak·RSS 혼동 금지 |
| 전체 확인 | 실제 변경 범위 unit/integration, all-target check/test, focused rustfmt, 기존 allow 조건을 명시한 Clippy. 실행/재사용/미실행을 분리 |

PC baseline 비교 원칙:

- [기존 probe](../../../../../tmp/r06_pc_baseline_20260926/probe.rs)의 입력/반복 수/타이밍 구간을 유지한다. 내부 API 조정이 필요하면 계측 adapter 변경만 별도 delta로 보존하고 workload 변경 여부를 검증한다. 서로 다른 target directory와 새 결과 경로를 사용한다.
- CAP의 우선 목표는 **기본 한도에서 300명 단일 source 피해와 500명 동시 생성**의 해당 확장 거부를 해소하는 것이다. 성공할 때 실제 유닛/적중/HP 변화 수를 검사한다. 12 source 반복도 재측정하되 다른 정당한 한도가 드러나면 별도 결과로 기록한다.
- 이전에 끝까지 수행하지 못한 조건에는 처리 완료 시간의 before/after 배수를 만들지 않는다. “거부까지 시간”과 “완료 시간”을 분리한다.
- F01 교정으로 기존 부하 일부가 새로 거부될 수 있다. 특히 128 Read × 6,000회의 큰 스트레스 입력은 기존 성공이 예산 우회에 의존할 수 있으므로 **무조건 성공 보존을 요구하지 않는다**. 어떤 비용이 새로 반영됐는지, 간단한 정상 부하까지 과잉 청구하는지 구분한다.
- 합성 kernel, 동일 capture 반복, 실제 private 피해 처리, public step을 서로 대체하지 않는다. 새로운 성공도 전체 수백 적 공세의 성능 PASS로 표시하지 않는다.
- wall-time 임의 합격선을 새로 정하지 않는다. 같은 환경에서 실제 처리량/결과/할당/회계의 비교를 우선하고, 큰 지연 증가는 원인을 설명한다.

## 8. 증거 보존과 구현 담당자에게 전달할 문구

구현 시작 시 git HEAD/status 외 dirty/untracked를 포함한 actual bytes와 해시를 새 baseline으로 보존한다. 과거 target과 PC 측정의 파일 집합은 다르므로 한쪽이 다른 쪽을 대체하지 않는다. Unix 테스트 guard 등의 기존 delta를 제거하지 말고 현재 bytes를 기준으로 출처를 기록한다. 종료 target은 수정 source·기록·모든 시험 입력/출력·실패 로그·before/after archive·복원 delta를 결속한다. 원본 probe/log/report/ARTIFACTS는 덮어쓰지 않는다.

사용자가 실제 교정을 시작할 때 전달할 지시:

> 이 교정 계획을 기준으로 R06 WP02-F01/F02/F03 및 CAP-01/CAP-02를 교정해줘. 먼저 원본 감사와 PC baseline을 확인하고 현재 actual bytes를 새 baseline으로 고정해. 예산 숫자·계산 의미·누적 admission의 미환급 계약은 임의 변경하지 마. 구현 상세는 책임 경계 안에서 정하되 비용의 소유권·선청구·재사용 근거를 기록해. 단계검토 옵션을 유지하고 구현 중 빌드·관련 검증이 가능한 의미 있는 지점에서 새 target으로 동결한 뒤 중단해. 미완료 항목과 새로 드러난 거부는 숨기지 말고 보고해. 재검토 PASS나 다음 WP 진입은 자체 선언하지 마.

계획 작성 검증: 기존 source 294개 currentness와 WP02 target manifest hash를 재확인했다. 제품 수정, 부하 재측정, 새 감사 probe 및 전체 회귀 시험은 이번 계획 작성에서 실행하지 않았다. 이 문서는 교정 완료 증거가 아니다.
