# S01-P2 시설 template 조회의 소유권 정리

작성·갱신일: 2026-10-18

**지도 조회·게이트 도달 판정의 불필요한 template 사본을 제거했다. 구현·동작/구조 검토 완료, 성능 수용은 미완료다.**
공개 snapshot은 변경 전후 117,922 bytes 전체가 같고 기존 영향 테스트 7개가 통과했다.
기존 snapshot 시간·크기 기준과 X01 concurrent 성능 finding은 열린 상태다.

## 1 범위·재사용 근거

[README](README.md)가 지정한 [S01-P2 구현 단위](S01_SNAPSHOT_COST_REVIEW.ko.md#s01-p2를-선택할-경우의-구체적-구현-단위)와
[X01의 우선순위 판단](X01_CONCURRENT_COST_REVIEW.ko.md#5-후속-순서와-완료-검토)에 따라 진행했다.
[기준](../refactoring_criteria.ko.md) §6–8과 [실행 지침](../system_refactoring_execution_guide.ko.md) §5–6을 적용했다.

[기준선](evidence/snapshot_borrow_001/baseline.json)의 생산자 입력 4개와 public_guards 도구 hash가 이전 근거와 일치했다.
S01-P1의 공개 결과·소비 계약 조사와 X01-P1의 비용 귀속을 재사용하고 변경한 두 호출 경계만 다시 검토했다.
변경 전 release 바이너리도 S01-P1에서 기록한 hash와 일치했다. 이 바이너리로 변경 직전 snapshot 진단을 한 번 실행했다.
이전 전체 기능 검증을 반복하지 않고, 이번 소스에 영향받는 기존 시설 테스트를 선택했다.
테스트·도구·DTO·manifest·상한은 이번에 변경하지 않았다.

## 2 구현과 설계 검토

변경은 [node_flow.rs](../../src/game/world/node_flow.rs)와 [facility_rooms.rs](../../src/game/world/facility_rooms.rs) 두 파일이다.
[이번 작업만의 diff](evidence/snapshot_borrow_001/source.diff)는 작업 시작 시 보관한 소스와 비교하므로 기존 사용자 변경을 포함하지 않는다.

| 경계 | 변경 전 | 변경 후와 보존 조건 |
| --- | --- | --- |
| template resolver | publication에서 ID 조회·version 검증 후 전체 clone | `facility_template_for_map_ref`가 같은 검증 후 `&FacilityTemplate` 반환. 참조 수명은 `self`에 묶임 |
| 소유 사본이 필요한 기존 호출 | `facility_template_for_map` 사용 | 같은 이름의 owned wrapper가 borrowed resolver의 성공 결과만 `.cloned()`. 전이 호출의 ownership 유지 |
| `current_map_view` | template을 복사한 뒤 읽어서 DTO 생성 | 참조를 직접 사용. 은닉·위협도·readiness·entry preview와 반환 DTO의 소유권 유지 |
| `facility_gate_reached` | template을 복사한 뒤 완료 노드 BFS | 참조로 같은 이웃 조회. 조기 거절·게이트 조건·방문 순서·오류 시 false 유지 |

lookup·version 검사와 오류 문자열의 권위는 한 곳이다. borrowed/owned helper에 정책을 중복 구현하지 않았다.
새 캐시·무효화 책임·공유 mutable 상태는 생기지 않는다. 읽기 호출은 publication의 데이터를 빌리고 결과 DTO는 기존처럼 소유한다.
두 읽기 경로에서 전체 template clone이 제거됐다는 구조적 효과와 실제 지연 시간은 별도로 판단한다.
snapshot 한 번당 clone 횟수가 항상 정확히 두 번이라고 주장하지 않는다. 게이트 판정의 호출 빈도와 조기 반환에 따라 달라진다.
기존 owned 호출을 일괄 전환하거나 최상위 지도와 시설 내부 지도를 합치지 않았다.

## 3 영향 검증과 최초 성능 관측

[명령](evidence/snapshot_borrow_001/test-command.json)·[로그](evidence/snapshot_borrow_001/tests.log)·[결과](evidence/snapshot_borrow_001/test-result.json)에
기존 테스트 **7 passed / 0 failed**를 기록했다. 공개·미지원 방, 실제 게이트 왕복과 Continue,
하강/SavePoint·영구 탐사, 목적지 witness, 예산 거부·취소·재시도, 조우 완료 후 공개를 검증한다.
테스트는 수정하지 않았다. 새 테스트·전체 기능·allocator 실행도 없다.

[plain release build](evidence/snapshot_borrow_001/build-result.json)는 exit 0이었다.
[전](evidence/snapshot_borrow_001/before-inspection.json)·[후](evidence/snapshot_borrow_001/after-inspection.json) 진단은 같은 도구·fixture의 각 101개 표본이다.
SavePoint 완료 후 ViewingMap snapshot 전체와 모든 구성의 byte 수가 같고, 이전 S01-P1 결과와도 일치한다.
SHA-256은 `9e413d3e35c6f1a40ea861248c44d65a1c2b91747207bdc8dbb7faf51ddaa4aa`다.

| 최초 순차 관측 | 변경 전 median / p95 | 변경 후 median / p95 |
| --- | ---: | ---: |
| 생성 + Value 구성 | 0.616666 / 0.742194ms | 0.647946 / 0.821093ms |
| compact 직렬화 | 0.104527 / 0.155193ms | 0.104056 / 0.158490ms |
| 같은 표본의 두 구간 합 | 0.722086 / 0.887579ms | 0.755720 / 1.041971ms |

첫 관측은 후자가 더 느렸다. 이 결과를 숨기거나 단일 측정으로 합격·회귀 확정하지 않는다.
다른 시점·CPU scheduling·cache 영향을 충분히 분리하지 못했으므로 아래의 제한된 교차 비교를 추가했다.

기본 [public guard raw](evidence/snapshot_borrow_001/after-public-guard.json)·[실행](evidence/snapshot_borrow_001/after-public-guard-execution.json)은
snapshot p95 **1.021362ms**, **117,922 bytes**로 기존 `<0.5ms / <65,536 bytes`를 넘었고 exit 101이었다.
receipt를 저장한 뒤 snapshot 시간 assert에서 종료했다. 별도 기록된 checkpoint·history·staging 조건은 통과했다.
checkpoint 21,734 bytes, history 1,024/8,192 entries 조회, staging 128/8,192 entries 실패 rollback 동일성도 보존됐다.
이 실패는 도구 오류나 성능 통과로 바꾸지 않는다. 이후 진단 비교로 기존 cap을 면제하지 않는다.

## 4 지연 증가 관측의 제한된 후속 비교

[실행 코드](evidence/snapshot_borrow_001/compare.py)는 같은 임시 소스·build 경로에서 전후 바이너리를 만들고,
같은 CPU 30에서 warm-up 뒤 순서를 번갈아 6쌍만 실행했다. 통과한 기능 테스트와 전체 성능 부하는 반복하지 않았다.
[입력 대조](evidence/snapshot_borrow_001/comparison-inputs.json)에서 복사한 소스 406개가 두 파일의 전후 버전 외에는 같음을 확인했다.
두 빌드와 14회 관측은 모두 exit 0이다. [명령·binary hash·순서](evidence/snapshot_borrow_001/comparison-execution.json),
[대조와 쌍별 값](evidence/snapshot_borrow_001/comparison-validation.json)을 보존한다. warm-up 두 개는 아래 집계에서 제외했다.

| 6회 각각의 통계량을 비교 | 변경 전 실행 간 범위 | 변경 후 실행 간 범위 | 변경 후가 낮은 쌍 |
| --- | ---: | ---: | ---: |
| 생성 + Value 구성 median | 0.666029–0.736574ms | 0.683653–0.733388ms | 3 / 6 |
| 전체 median | 0.775376–0.865558ms | 0.797579–0.856521ms | 4 / 6 |
| 전체 p95 | 1.019819–1.398825ms | 1.141882–1.500759ms | 2 / 6 |

전체 median의 실행 간 중앙값은 전 0.819670ms / 후 0.826293ms,
전체 p95의 실행 간 중앙값은 전 1.149211ms / 후 1.291704ms다. 이것은 표본을 합친 전체 p95가 아니다.
각 표본의 생성·직렬화 시간 합과 출력 동일성을 별도로 검증했다.
변경 후 p95가 4쌍에서 더 높다는 관측을 유지한다. 실행 간 분산과 쌍별 방향이 섞여 있어 지속적인 개선이나 코드 변경에 의한
퇴행을 확정하지 못했다. 단순한 일률적 허용 비율로 비퇴행 합격을 만들지 않는다.
**구조적 clone 제거는 확인했지만 성능 개선·비퇴행 수용은 주장하지 않는다.**
정식 성능 수용에는 더 안정적인 환경의 비교가 필요하며 기존 시간·크기 cap 미충족도 그대로다.
이는 최대 방문방·전체 콘텐츠·장기 부하·Unity/서버 측정이나 과거 P01 기준선 비교를 대체하지 않는다.

## 5 분리 검토·다음 행동

소스와 실행 근거를 고정한 뒤 읽기 전용 pass에서 lookup·오류의 단일성, 두 읽기 호출의 clone 제거,
공개 결과·저장·실패/취소 의미, 실제 측정 실패·한계와 후속 분리를 대조했다.
[검증](evidence/snapshot_borrow_001/validation.json)과 [분리 검토](evidence/snapshot_borrow_001/review.json)가 결과를 소유한다.
동일 작성자의 분리 검토이며 비참여 독립 감사가 아니다.

이번 완료는 **내부 소유권 구현·동작/구조 검토** 범위다. 성능 수용 전체가 완료됐다는 뜻이 아니다.
성능 회귀 여부 미확정과 snapshot 시간·크기 초과를 열린 항목으로 남긴다.
S01-C1 응답/예산·S01-C2 외부 계약/Unity reader도 기존 미해결로 유지한다.

다음은 **X01-P2 step 내부 비용 귀속**이다. [X01 §4–5](X01_CONCURRENT_COST_REVIEW.ko.md#4-실행-경계와-설계-품질)의
`step_by` → `step_until_budgeted_inner` → `runtime_transaction` / `runtime_timestamp_reusing`에서
native 후보 준비·사건 처리·retained/commit 비용을 최소 계측으로 분리한다. 기존 89.3% step 비중만으로 특정 clone을 원인으로 단정하지 않는다.
원자성·실패·계보·결정론과 동일 fixture를 보존하고, 앞서 확인한 단계 총량·전체 테스트를 이유 없이 다시 실행하지 않는다.
S01-P2의 지연 분포와 S01-P3 서버 중간 표현은 각각 미확정 성능 판단·별도 후보로 유지한다.
