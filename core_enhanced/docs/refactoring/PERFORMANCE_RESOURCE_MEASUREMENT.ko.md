# 성능·자원 비용 1차 측정과 후속 판단

작성·갱신일: 2026-10-18

**1차 측정·도구 교정·분리 검토 완료. 성능 수용은 미충족이다.**
현재 Run snapshot의 시간·크기와 64-session 전체 구간에서 기존 기준 초과를 확인했다.
게임 실행 코드는 바꾸지 않았으며, 원인 귀속 전 복사 제거·기준 완화는 하지 않았다.
이전 [선정 리팩토링의 통합 완료](FINAL_INTEGRATION_REVIEW.ko.md)와 이번 새 성능 finding을 구분한다.

## 1 범위·기준과 근거 재사용

후속 작업 질문에 제안한 성능 측정을 이번 이어서 수행 요청으로 재개했다.
[기준](../refactoring_criteria.ko.md) §3.2·7.4, [실행 지침](../system_refactoring_execution_guide.ko.md) §5.3–6,
[runtime 성능 계약](../core_runtime_contract.md#final-core-performance-guard)을 적용한다.
기존 Core-owned `tools/runtime_architecture_final_validation`의 public_guards와 final_performance를 재사용했다.

공개 Run snapshot+직렬화, checkpoint export, 1,024/8,192 history query, 128/8,192 rejected staging,
5,000ms event-heavy 및 64 session/8 lane × 100 × 10ms 고정 부하를 측정했다.
후자는 커널 fixture이며 공개 Run·host 저장·실제 사용자 64명의 종단간 부하가 아니다.
최대 이력 개수는 확인했지만 모든 콘텐츠·최대 유닛/자원·장기 실행을 측정한 것은 아니다.

[직접 연결 입력 대조](evidence/performance_001/reused-evidence.json)의 8개 파일은 최종 통합 입력과 일치한다.
기존 기능·실패/복원 검증을 재사용하고 전수 테스트는 반복하지 않았다.
과거 성능 숫자는 현재 통과 증거로 쓰지 않았다. [착수 입력](evidence/performance_001/entry.json),
[최종 도구·설정 입력](evidence/performance_001/final-tool-inputs.json)은 전체 새 source freeze와 구분한다.

환경은 Linux/WSL2 x86_64, release, Core test-fixtures다. 전투 측정은 기존 coordinator CPU30·lane CPU0/2/4/6/8/10/12/14를 유지했다.
public guard는 기존처럼 별도 affinity를 강제하지 않는다. 배포 SLA나 다른 장치의 수치로 확대하지 않는다.
plain 시간과 allocation-observer 할당량을 별도 빌드·순차 실행했다. 계측 빌드의 시간과 plain의 0 allocation 필드는 비용 판정에 사용하지 않는다.

## 2 도구 교정·실패의 처리

[실제 변경 diff](evidence/performance_001/tool-final.diff)는 세 파일이다.

- public_guards: 폐기된 Standard를 typed GameMode::Expedition으로 교정하고 필수 loan_copy_ids를 추가했다.
  현재 공식 초반의 선택 가능한 slot-2 보상을 공개 command로 한 번 종료한 뒤 전투로 진입한다.
  Combat 분류 외에 공개 `entry_preview.stress_combat`도 소비하고 `has_active_battle`을 필수 확인한다.
  예상 밖 콘텐츠를 임의로 건너뛰는 진행기는 추가하지 않았다. 현재 선택은 slot-5 스트레스 전투다.
  측정 연산·표본수·cap·rollback 검사는 유지했다. workload를 `/3`으로 식별하고 측정 구간 밖에서 snapshot 구성별 byte 수를 기록한다.
- final_performance: 기본 P01 검증 본문·부하 함수는 그대로다. 명시적 `--measure-current <output.json>`은 같은 부하의 현재 raw sample만 수집한다.
  `baseline_comparison_applicable=false`, `performance_pass_claimed=false`를 출력한다. frozen baseline을 덮어쓰지 않았다.
- standalone Cargo.lock: 현재 Core의 fs2/hmac 및 전이 의존성 여섯 package를 추가했다. 기존 package 버전은 교체하지 않았고 workspace lock도 유지했다.

최초 offline cache/lock 오류, 전투 선택 실패와 후속 교정은 [finding 기록](evidence/performance_001/findings.json)에 남겼다.
[초기 실행](evidence/performance_001/plain-executions.json)의 public guard는 전투 준비에서,
final_performance는 baseline 도구 SHA 불일치에서 각각 exit 101이었다.
보상 종료 뒤에도 전투를 Combat 분류만으로 찾던 추가 실패는 [원시 로그](evidence/performance_001/public_guards-public003.log)에 보존한다.
최종 public plain은 준비에 성공해 모든 측정 receipt를 쓴 뒤 **실제 snapshot cap 위반으로 exit 101**이다.
이를 도구 실패나 통과로 바꾸지 않았다.

P01 baseline의 도구 SHA는 `64debfd6…c631`, 현재 preservation source는 `2ae536cc…ed14`다.
새 진단 모드 이후에도 [기본 검증 실행](evidence/performance_001/default-after-diagnostic-execution.json)은 같은 guard에서 거절한다.
현재 출력의 event 수·byte 수는 과거와 같지만 SHA도 다르므로 과거 baseline 대비 회귀 유무·원인·통과를 선언하지 않는다.

## 3 실제 측정 결과

### 공개 Run·history

[최종 plain 원자료](evidence/performance_001/public_guards-public007.json)는 각 101개 표본이다.

| 항목 | 현재 결과 | 기존 기준·판정 |
| --- | ---: | --- |
| Run snapshot+직렬화 p95 | 0.728398ms | 0.5ms 초과 |
| Run snapshot compact JSON | 117,922 bytes | 65,536 bytes 초과 |
| checkpoint export p95 / 크기 | 0.000040ms / 21,734 bytes | 0.1ms / 32KiB 이내 |
| history last-10 query p95, 1,024 / 8,192 | 0.220 / 0.220µs | 규모 조건·cap 충족 |
| rejected staging p95, 128 / 8,192 | 19.116 / 17.603µs | 이력 크기 독립 조건 충족, 전후 공개 Run snapshot 동일 |

snapshot 초과를 확인한 같은 측정 구간의 선행 세 회도 0.725923 / 0.716185 / 0.708952ms와 117,922 bytes였다.
마지막 도구는 측정 구간 밖의 구성별 byte 기록만 추가했다. 통과 표본만 고르거나 cap을 높이지 않았다.
구성은 facility 51,821, skill_catalog 32,379, roster 16,684, inventory 6,651, map 5,266 bytes다.
시설+스킬만 **84,200 bytes**라 동일 compact JSON을 유지한 캐싱만으로 64KiB를 달성할 수 없다.
component 값은 해당 JSON value 길이이며 key·구분자·외부 wrapper를 포함한 총량과는 다르다.

[별도 allocator](evidence/performance_001/public_guards-allocation008.json)의 rejected staging은 각 5회다.
128/8,192 모두 **518 allocations / 0 reallocations / 56,033 peak bytes / 0 retained delta**로 동일했다.
할당 guard·공개 snapshot 보존은 통과했지만 allocator exit 0이 plain의 snapshot cap 실패를 해소하지 않는다.

### 고정 전투 부하

[plain 1](evidence/performance_001/final_performance-diagnostic002.json),
[plain 2](evidence/performance_001/final_performance-repeat003.json),
[plain 3](evidence/performance_001/final_performance-repeat004.json)은 각각 warm-up 뒤 21개 표본이다.
첫 concurrent 초과가 반복되는지 보기 위해 두 회를 추가했다.

| 부하 | p95 1회 | p95 2회 | p95 3회 | 현재 관찰 |
| --- | ---: | ---: | ---: | --- |
| event-heavy 5,000ms | 20.852338ms | 17.705238ms | 17.896254ms | 기존 25ms보다 낮음. baseline PASS 아님 |
| 64 session / 8 lane | 12.188723ms | 11.789399ms | 13.473684ms | 세 회 모두 기존 10ms 초과 |

각 부하 63개 plain과 5개 allocator 표본의 event 수·byte 수·SHA가 프로세스·빌드 사이에도 일치했다.
event-heavy는 186 events / 65,720 bytes, concurrent는 11,840 events / 3,881,088 bytes다.
현재 실행의 repeatability이며 P01 parity 통과가 아니다.

[별도 전투 allocator](evidence/performance_001/final_performance-allocation008.json)는 각 5회다.

| 부하 | allocations / reallocations | peak delta bytes | retained delta bytes |
| --- | ---: | ---: | ---: |
| event-heavy | 42,262 / 1,944 | 272,063 | 266,238 |
| concurrent | 364,268 / 6,660 | 9,674,116–9,728,762 | 4,196,928 |

이는 allocator 요청 byte 관측이다. RSS나 실제 copy bytes, 종료 후 누수 검사가 아니다.
event-heavy는 core/log를 보유한 지점, concurrent는 직렬화 결과들을 보유한 지점에서 retained를 측정한다.
과거 baseline과 입력·출력의 동등성이 확보되지 않아 allocation 회귀 guard 통과로 해석하지 않는다.

## 4 설계 판단과 확정 후속

| 항목 | 근거와 처리 |
| --- | --- |
| **S01-P1: snapshot 시간·payload** | 현재 공개 Run에서 기존 기준 초과. `world/snapshot.rs`가 facility·skill catalog·roster·inventory 등을 묶고 `facility_rooms.rs::facility_view`가 방·연결·현재 지도·활동을 조립한다. 다음은 각 필드의 소비 계약·중복·생성/직렬화 비용 대조다. 내부 시간 최적화와 출력 크기/계약 판단을 분리한다. 지금 DTO 필드 삭제·새 schema·cap 증액을 결정하지 않음 |
| **X01-P1: 64-session 전체 비용** | 3회 p95 모두 기존 10ms 초과. `concurrent_sample`은 thread 시작, core 생성/start, 100회 step, log 직렬화, join/정렬을 함께 잰다. 같은 조건에서 단계별 비용을 구분한 뒤 실제 병목의 owner·개선·보존 검증을 정함 |
| history 복사 | 현재 선택한 거절 경로에서는 이력 규모에 따른 할당 증가 없음. Arc 공유/후보 경계를 제거할 근거로 삼지 않음 |
| typed resource 중첩 transaction | 공식 catalog는 빈 목록이며 `prepare_runtime_resources`는 비어 있으면 store를 만들지 않는다. 이 부하로 populated store의 accept/finite/terminal 비용을 설명하지 못함. 해당 부하는 별도 후속이며 아직 병목 판정 안 함 |
| frozen P01 비교 | 기존 도구 source·출력 identity가 달라 비교 유효성 미확보. 기본 guard는 거절 유지. 현재 수치를 새로운 승인 baseline으로 자동 승격하지 않음 |

중첩 복사를 없애면 국소·묶음 원자성뿐 아니라 clone/work/retained 연료와 거절 지점도 달라진다.
측정된 초과만으로 그 복사를 원인으로 단정하거나 앞선 기능 검증의 의미를 약화하지 않는다.

## 5 분리 검토와 현재 종료점

측정 종료 뒤 같은 작성자의 읽기 전용 pass에서 실제 소스·원래 guard 본문·최종 diff·원시 수치·한계와 후속을 대조했다.
[산술·입력 대조](evidence/performance_001/measurement-validation.json), [검토 기록](evidence/performance_001/review.json),
[문서 확인](evidence/performance_001/verification.json)에 연결한다. 비참여 독립 감사가 아니다.
원래 verifier 본문이 정확히 보존됐는지, p95·표본수·출력 동일성·계측 분리·실제 실패가 유지됐는지 확인했다.

**종료점은 현재 비용 기준선·도구 교정·후속 범위 확정이다. 성능 개선이나 모든 기준 통과 완료가 아니다.**
다음은 S01-P1의 snapshot 소비 계약·구성 비용 조사, 이어 X01-P1의 concurrent 비용 귀속이다.
게임 source·테스트·공식 콘텐츠·성능 cap·workspace 설정은 변경하지 않았다. 기능 전수·전체 feature·Unity 검증도 반복하지 않았다.

후속 S01-P1은 [snapshot 소비·비용 조사](S01_SNAPSHOT_COST_REVIEW.ko.md)에서 Core/서버 경계와 생성/직렬화를 분리했다.
이 문서의 수치·실패를 유지하며, 내부 후보·응답/예산 판단·외부 계약 미확인은 후속 기록을 따른다.
[X01-P1 concurrent 단계별 비용 귀속](X01_CONCURRENT_COST_REVIEW.ko.md)도 후속 완료했다.
현재 다음 실행은 S01-P2 template borrowed lookup 구현이며 X01의 step 내부 귀속·성능 기준 초과는 열린 상태다.
