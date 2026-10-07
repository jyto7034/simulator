# R06 WP02 비용 회귀 조사

작성일: 2026-09-27 KST. 대상: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`, WP02.

**현재 D04-F01 기능 PASS는 유지한다. 비용 상한 충족 근거는 미해결이며, peak를 만드는 allocation 소유자는 아직 특정하지 못했다.**
이번 조사는 기존 원시 기록과 측정·제품 코드를 읽은 결과다. 새 workload 실행이나 제품 교정은 하지 않았다.
기존 R05에서 확정한 guard를 사용해 R06 변경의 비용을 확인하는 작업이며, R05 기능 개발을 재개하는 것이 아니다.

## 대상과 판정

- [현재 독립 검토](../../reviews/wp02_d04_f01_correction_001_review_001/REVIEW.md): D04-F01 해결, R05-E01 비용 증거 미해결.
- [현재 manifest](../../corrections/wp02_d04_f01_001/target/d04_f01_correction_001/MANIFEST.json): SHA-256 `e4f0d77b1bd3066493d605271605d7a4b6e16317a6d540d303909b076b480104`.
- Source 333개 digest: `34ceedbebc84b9396d42f92fa31033fe0b831a90e5325c8c4463e0c88f067145`.
- [재분석 코드](analyze.py), [원시 기록별 수치·hash](analysis.json), [현재성·양방향 복원 확인](currentness.log).

## 확인한 사실

원 guard의 concurrent cap은 **peak_delta_bytes 9,030,188 bytes**다. RSS 또는 절대 peak_live_bytes가 아니다.
`final_performance.rs::allocation_guard`는 각 sample의 allocation+reallocation 횟수, peak delta, retained delta가
각 baseline 최대값 이내인지 모두 검사한다. 마지막 성공 실행이나 평균으로 실패를 대체하는 규칙은 없다.

| 실행 | 최대 peak delta | cap 대비 | 결과 |
| --- | ---: | ---: | --- |
| 동결 D04 최초 | 8,997,016 | −33,172 | PASS |
| D04-F01 최종본 첫 실행 | 9,040,041 | +9,853 | FAIL |
| 동일 최종본 재실행 | 9,040,546 | +10,358 | FAIL |
| 동결 D04 대조 실행 | 9,050,125 | +19,937 | FAIL |
| 동일 최종본 마지막 실행 | 9,020,761 | −9,427 | PASS |

위 완성 실행들의 concurrent allocation=364,268, reallocation=6,660, retained delta=4,196,928과
event hash가 같다. D04 대조도 실패하므로 D04-F01 교정이 원인이라고 단정할 수 없다.
현재 최종본의 15 sample 중 2개가 초과했다는 관찰을 실패 확률 추정으로 해석하지 않는다.

### peak에 포함되는 구간

측정은 `concurrent_sample`의 thread spawn 전부터 시작한다. 8개 lane이 각각 8개 session을 만들고
100회의 step을 진행한 다음 각 event log를 `serde_json::to_vec`로 직렬화한다.
lane별 setup·step·serialization을 맞추는 barrier는 없다. 한 lane이 직렬화하는 동안 다른 lane의 core가
살아 있을 수 있고, 직렬화 중 같은 lane의 나머지 core와 이미 만든 byte buffer도 공존한다.
모든 lane을 join·정렬하고 hash를 계산한 뒤, 직렬화 결과를 소유한 상태에서 측정을 끝낸다.

따라서 global peak에는 전투 중 상태뿐 아니라 thread/container 비용과 직렬화 buffer의 공존도 들어간다.
동일 결과·동일 allocation 횟수·동일 마지막 retained여도 공존 시점이 다르면 peak는 달라질 수 있다.
**이는 코드상 가능한 변동 메커니즘이다. 실제 최고점이 어느 단계·소유자에서 발생했는지는 기존 receipt로 알 수 없다.**
observer는 요청된 heap bytes를 세며 RSS/allocator arena를 관찰하지 않는다. 현재 조사에서 observer 결함을 입증하지 않았다.
`repeatable`은 결과 parity 검사이며 메모리 peak의 동일성을 뜻하지 않는다.

### 재할당 차이는 CAP 교정 구간부터 확인된다

[CAP 인계서](../../corrections/wp02_cap_01_02_001/REVIEW_HANDOFF.ko.md)와 원시 receipt를 대조했다.

| 과거 구간 | allocation | reallocation | 최대 peak delta |
| --- | ---: | ---: | ---: |
| R05 교정128의 기록 | 364,268 | 5,900 | 9,006,984 |
| R06 CAP 직전 direct_002 | 364,268 | 5,892 | 9,003,133 |
| CAP 2배 성장 후보 quiet | 364,268 | 4,548 | 9,052,494 |
| CAP 최종 1.5배 성장 final_exact | 364,268 | 6,660 | 9,010,868 |
| 현재 D04-F01 첫 실행 | 364,268 | 6,660 | 9,040,041 |

같은 tool/input hash와 결과 hash를 갖는 기존 기록의 비교다. 실행 환경·source가 다른 과거 기록으로 인과 효과를 계산하지 않는다.
R05 교정128 대비 +760회, CAP 직전 대비 +768회이며 D04-F01에서 새로 생긴 증가가 아니다.
CAP은 actor rows와 queue/bucket의 성장·용량 선청구를 고쳐 PC 36조건의 거부를 11→5로 줄인 작업이다.
재할당이 더 적은 2배 후보가 peak 상한을 넘었으므로 **reallocation 횟수 최소화가 이번 교정의 목표는 아니다.**
기존 CAP 교정을 되돌리거나 성장 배수만 무작정 바꾸면 이미 해결한 work/retained 거부를 되살릴 수 있다.

## 다음 측정과 설계 방향

우선순위는 아래와 같다. 모두 원인 후보이며 확정된 결함 목록은 아니다.

1. actor rows, event queue, timestamp bucket의 live len/capacity와 COW 전후 공존량을 측정한다.
   CAP의 실제 capacity 선청구·최소 합법 용량 fallback·정렬·동일 priority 순서를 보존한다.
2. R06의 `ActiveAction.bound_cast`와 optional boxed result, cast/delivery capture가 기존 literal workload에서
   차지하는 inline 크기·공유/복제 비용을 측정한다. witness는 표현식 요구가 없으면 설치하지 않는 경로가 이미 있다.
   기존 lazy/Box 설계를 중복 구현하거나 증거 없이 모든 필드를 Box로 바꾸지 않는다.
3. peak가 serialization 전후 어디에 있는지 진단한다. tool의 직렬화·lane 수·측정 경계를 바꾼 결과를
   원 guard 통과로 제출하지 않는다. tool 비용으로 보이더라도 기존 cap 계약은 자동으로 면제되지 않는다.

진단용 계측은 격리 사본에서 수행하고 원 acceptance binary와 구분한다. phase tag, 고정 크기 counter/trace로
계측 자체의 allocation과 재진입을 피한다. counter 읽기의 동시성·계측 오버헤드도 결과의 한계에 적는다.
측정에서 실제 중복 소유·불필요한 복제/여유 용량이 확인되면 해당 owner만 줄인다. Snapshot identity와
release 고정값, action 결과 누적, 오류·rollback, resource authority 및 work/retained admission을 유지해야 한다.

구체적인 실행 순서·검증·동결 조건은 [후속 구현 지시서](../../handoffs/WP02_COST_REGRESSION_IMPLEMENTATION_REQUEST_20260927.ko.md)에 정했다.
이 조사는 R05-E01을 닫거나 D04 단계 전체/WP02/R06 완료를 선언하지 않는다.
