# U17 후속 저장·복원 리팩토링

2026-10-16 실행 요청: `CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md 문서를 지침 삼아 u17 후속 작업에 착수해줄래?`
추가 지시: 실제 코드에서 로드맵과 다른 의견이 나오면 양쪽을 검토하여 구현한다.

승인 범위는 [로드맵 U17 후속](../../../../roadmaps/core_first/ROADMAP.ko.md#u17-후속--저장복원-계약과-상태-수명-리팩토링)이다.
기존 게임 정책과 V17 wire 계약을 보존하는 후속 구현이며 새 정책·Goal 판정을 만들지 않는다.
기존 P2G/Goal/`single_goal/target/u17_current_001`은 당시의 고정 근거다. 이 디렉터리는 별도 변경·증거를 소유한다.
단독 실행, 중간 검토 중지 없음, 구현·자체 검증·고정/복원 확인 후 감사 요청 대기에서 종료한다. 자동 감사와 U18은 수행하지 않는다.

## 실제 코드와 방향 비교

- 상태 종류 전체를 새로운 타입 계층으로 분할하면 이미 존재하는 exhaustive capture/CloneShape를 중복하고 V17 이행 부담을 만든다. 이번에는 기존 typed checkpoint를 유지하고 **복사/설치와 완료 수명 경계**를 모은다. 보존 정책 자체는 아래 표로 분리한다.
- SavePoint는 wire alias 해소 이후 다시 복사되어 설치된다. 단독 소유 Arc는 이동하고 공유 소유만 사전 검사 후 복사한다. 모든 checkpoint를 Arc로 바꾸는 대규모 변경은 필요하지 않다.
- 유물 유무에 따라 전체 Continue JSON 비용 검사가 달라진다. 공통 저장 계량은 유물과 무관하게 적용하되 기존 유물 native owner·typed resource 보호를 중복 계량하지 않는다. decode/Value/encoded buffer/clone의 계량과 실제 allocator peak를 구분한다. 한도는 유지한다.
- 부활 보상의 옵션 생성 helper는 이미 공유된다. 남은 source→stage 해석을 같은 도메인에 두고 생성/복원에서 소비한다. 매 저장마다 전체 reader를 실행하지 않는다.
- 완료 preview 제거는 이미 교정되어 있다. 새 결함으로 보고하지 않고 완료 owner 정리의 공통 경계로 만든다. 미확정 preview, 유물 Event receipt, 보상 권리는 유지한다.

## 상태별 소유권·복원 정책

| 상태 / owner | Continue | SavePoint | 거래 rollback | 전투 후퇴 |
| --- | --- | --- | --- | --- |
| 런 자산·직원·맵·난수 / RunCheckpointPayload | 미완료 노드 입장 전, 완료는 확정 시점 | checkpoint 시점 | 미수락 거래 전 | 입장 비용 확정 후 전투 전 |
| 창고 자산·debit / profile lifecycle + run ledger | 저장된 debit과 profile 차이 정산 | checkpoint debit 차이 정산 | lifecycle 원본 복원 | 입장 snapshot 범위; 전투 안 창고 지불 없음 |
| 도감·탐사·개인 기록 / profile | authoritative profile 유지 | 유지 | 현재 단조 지식 유지 | 유지 |
| 후퇴 횟수 / abnormality_attempts | 입장 전 root에 확정 후퇴 overlay | 해당 시점 | 거래 전 | 복원 후 1회 확정 |
| SavePoint 사용 횟수 / run_checkpoint + host monotonic | host 값 유지 | 성공 시 +1 | 미수락 시도 원복 | 전투 전 값; 전투 중 load 불가 |
| 사본 UUID·난수 / uuid_manager | 해당 root 복원 | 해당 root, VN session counter 유지 | 거래 전 | 전투 전 |
| 확정 결과·선택권 / confirmed result + reward continuation | 미확인 결과/후보/선택/수령 상태 보존 | checkpoint 당시 자산/진행으로 복원 | 거래 전 | 확정 결과 이후 후퇴 불가 |
| preview·node session / RunState·GameCoreState | 미확정 배정과 필요한 선택만 유지 | 필요한 배정 보존 | 거래 전 | 전투 전 session 복원 |
| 완료 preview·일반 Event session | 완료 시 회수 | 회수 후 캡처 | 거래 전 | 미완료이므로 보존 |
| 유물 Event receipt | 완료 뒤에도 유지 | 해당 시점 | 거래 전 | 해당 시점 |
| 귀환·실패 / lifecycle + host journal | terminal active 무효화 | 정책상 가능한 실패 결정에서만 load | 미수락이면 원복 | 결과 뒤 불가 |

## 구현·검증 순서

1. 기존 공개 시나리오의 저장 경계와 fresh reader 비교를 재사용 가능한 절차로 연결한다. 소규모 fresh application 검사는 동일 콘텐츠/profile로 결과 확인 전→선택→수령 및 다음 명령을 비교한다. 미완료는 live와 비교하지 않고 입장 전 기대 상태·확정 후퇴를 따로 검사한다.
2. 공통 checkpoint 복사/설치, 부활 source 해석, 완료 임시 상태 정리, 저장 비용 단계 계량을 구현한다. V17 문서 형식과 기본 한도는 변경하지 않는다.
3. 관련 Core 기본/native 회귀, 정상/확장 공개 host→fresh process→후속 명령, 실제 durable IO·동일 재시도와 journal 수락 전후 복구, 비용 전후를 확인한다. 동일 입력/비영향 근거가 있는 기존 결과만 재사용한다.
4. 실제 코드·호출·기대값 자체 검토, 변경 source/증거 고정과 정역 복원 검증, 최신 인수인계 갱신 후 감사 요청 대기.

기존 runner는 `../single_goal/test_parts/{full_flow,fresh_readers,io_replay,collect_cost}.py`다. 과거 고정 파일을 수정·덮어쓰지 않고 공통 절차는 새 test_parts에서 재사용한다. 새로운 binary/config/승인은 새로운 출력으로 결속한다.
기본 Cargo 환경은 기존 `/tmp/wp02-cargo-home`, `.cache/u164-review-dev-target`, dev/test debug=0, offline/locked. 공유 target과 issuer host는 순차 실행한다.
컴파일 예상은 기존 2–8분, 최초 진단 60초, 10분 초과 시 CPU/로그/진행 확인. 공개 명령 응답 제한은 기존 180초, 비용 process는 기존 600초. 원인 없는 재실행·한도 상향·전체 회귀 반복은 하지 않는다.

## 현재 상태

구현·자체검증 종료. 기본 Core 유효2160pass(소켓 권한 재실행 포함)/6ignore, 관련 native25·6·45, server persistence29, 정상/확장 각333명령·fresh14·IO17, 이전 writer 저장fresh14, 실제 귀환/Abandon terminal, 비용12process가 통과했다. 중복되는 검사 묶음을 고유 시험 수로 합산하지 않는다. [CURRENT_RESULTS](CURRENT_RESULTS.json), [비용 비교](COST_REVIEW.json), [자체 검토](REVIEW.ko.md), [재현](REPRODUCE.md)을 따른다. 별도 source/evidence와 정역 복원 판정의 위치는 `target/u17_storage_001`이다. 감사·GoalComplete·U18은 미실행이며 새 감사 요청을 기다린다.

### 구현 중 확인한 대안 선택

3500자원 기존 회귀를 계량 없이 통과시키거나 입력/기본 한도를 바꾸는 대신, 자원 lookup 반복·JSON 작은 객체 과대예약·key 재정렬·checksum 전체 버퍼·복원 뒤 재capture/재encode를 각각 제거한다. 전체 타입 분리 없이도 wire를 검증한 reader가 그 immutable continuation을 재사용하는 구조가 현재 문제에 더 직접적이다. 마지막 재capture의 중복을 없애면 SavePoint 단독이동만 최적화하는 초기안과 비용 범위가 달라지므로 실행 기록에 전후 범위를 명시한다.
