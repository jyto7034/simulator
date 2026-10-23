# 선정 대표 조사·개선 범위의 최종 통합 검토

작성·갱신일: 2026-10-18

**판정: 완료.** S01–S11·X01/X02에서 선정한 대표 흐름의 조사·필수 개선·연결 검토를 완료했다.
이번 범위의 필수 미해결은 없다. 모든 Core branch·콘텐츠 조합·실행 환경의 정확성을 보증하는 판정은 아니다.
현재 구조의 유지 판단과 비필수 제한은 아래 범위와 함께 유효하다.

## 1 대상과 검토 방법

[README](README.md) §4의 완료 조건, [판단 기준](../refactoring_criteria.ko.md) §3·완료 전 조건,
[실행 지침](../system_refactoring_execution_guide.ko.md) §6–7을 적용했다.
[통합 준비 기록](INTEGRATION_READINESS_REVIEW.ko.md) §2–3의 선정 후보와 다섯 연결 조건을 대상으로,
현재 source의 생산자·소비자·실패 경계, 후속 diff, 원시 실행 근거를 별도 읽기 전용 pass에서 대조했다.
준비 보고서의 완료 요약만으로 통과시키지 않았다. 같은 작업자의 검토이며 비참여 독립 감사는 아니다.

준비 단계의 458개 입력 목록과 변경 귀속을 재사용했다. 최신 변경 대상과 직접 읽은 연결 경계
**40개 입력**은 [선택 대조](evidence/integration_final_001/review-inputs.json)에서 기록된 최종 상태와 일치했다.
X02 event 생성기의 교정 후 hash를 적용하며, 교정 전 baseline을 현재 값으로 잘못 사용하지 않았다.
전체 입력 재해시·전면 재조사·새 build/test 실행은 하지 않았다. 제품·테스트·fixture·설정 변경도 없다.

이 기록의 완료 범위는 시스템별로 선정했던 대표 흐름과 확정 개선이다.
지도에서 파일에 담당자를 배정했다는 사실을 모든 함수의 의미 조사나 검증으로 바꾸지 않는다.
각 시스템의 상세 범위와 한계는 [준비 기록의 종결 표](INTEGRATION_READINESS_REVIEW.ko.md#2-선정-후보의-종결과-설계-효과) 및 연결된 원 보고서가 소유한다.

## 2 다섯 통합 연결의 판정

| 연결·판정 | 현재 소스에서 확인한 조건 | 실행 근거와 제한 |
| --- | --- | --- |
| **I1 콘텐츠 → 실행: 충족** | world는 원래 root의 gate 이후 새 generation을 draft에 설치한다. publication은 draft와 output을 함께 교체한다. Battle은 전달된 data를 유지하며 수치식 capture는 generation·authored source와 exact Arc plan의 결합을 검사한다 | 외부 B의 실제 battle/resource/export와 A 유지, B의 늦은 준비 실패 후 A·seed·profile 보존. B 실패 사례는 loader가 거절할 invalid fixture 주입이며 정상 외부 입력의 오류 재현으로 해석하지 않음 |
| **I2 사본/장착 → 전투/효과: 충족** | player scenario는 실제 employee·inventory·loadout의 사본 ID를 전달한다. 생성 helper를 공유해도 batch와 단일 member의 deployment 시점은 각 wrapper가 유지한다. resource 소비와 공개 witness는 실제 employee/copy owner를 사용한다 | 외부 RON의 여섯 resource lifetime에서 실제 비용 12→10·source/copy 일치·reservation 종료. R2 이후 실제 철수/재배치에서 Deployment 초기화와 Run/Attempt 유지. 모든 장비·유물 조합은 미포함 |
| **I3 전투 → 정산/보상/terminal: 충족** | timestamp candidate와 cursor를 함께 확정한다. 최종 보상 수량을 summary/entitlement 전에 한 번 계산한다. terminal 전환이 먼저 정산·receipt를 만들고 ACK는 receipt를 소비한다. claim/Exit·node completion·영구 이관은 별도 단계이며 outer application draft가 후속 실패를 보호한다 | B1의 1배/2배·ChooseOne/ClaimAll·난이도 125% 공개 경로, live terminal/ACK/Continue, 비어 있지 않은 Run 자원 정산 실패·재시도, S09 death event 순서, S10 실제 HP·timestamp retry. 하나의 공식 자연 플레이 실행으로 합쳐 주장하지 않음 |
| **I4 저장 → 복원/다음 행동: 충족** | prepared 결과·checkpoint·profile은 commit 전에 준비된다. host가 durable intent를 accept한 후 Core를 한 번 commit하며 recovery-pending을 구분한다. Continue는 빈 run과 지원 payload·identity·보상 비교를 검사한 draft에서 복원한다 | S04 accepted 전후 I/O 실패·16지점 자식 프로세스 종료, Core 새 인스턴스의 RNG/다음 gate·원본 checkpoint 소비와 B1 재수령 거절. host opaque payload 시험은 Core codec 호환성·장치 전원 손실 시험이 아님 |
| **I5 제작/관측 → 제품 실행: 충족** | Trial은 fork한 CoreApplication의 실제 command/clock/snapshot을 사용하고 control 성공 뒤 accepted aggregate를 교체한다. prepared observation은 반환 전 TLS를 분리하며 실제 commit 뒤만 gauge를 게시한다. world 진단 export 오류는 정산 실패로 전파하지 않는다 | Trial trace 동일성, queue 포화/collector 종료·prepare 폐기, trace I/O 실패 후 정산의 기존 결과. X02의 교정된 실제 event 생성·네 generator 출력 조건 재사용. Unity 소비·서버 exporter 검증으로 확대하지 않음 |

실제 연결 위치는 `world/application.rs::prepare_application_scoped/commit_application`,
`world.rs::execute_behavior_draft`, `core/value_expr_runtime.rs::capture_value_expr_plan_with_prefix`,
`combat_setup/player_spawns.rs`, `core/build.rs`, `core/runtime_facts.rs::runtime_timestamp_reusing`,
`world/combat.rs::transition_finished_live_battle_to_result/handle_complete_combat_result`,
`world/reward.rs`, `world/expedition_lifecycle.rs`, `world/active_run/continuation.rs`,
host `publish_prepared_application`, `authoring/trial.rs::advance/prepare_step`에서 대조했다.

원시 출력의 [대표 성공 18건](evidence/integration_final_001/reused-tests.json)을 정확한 로그·줄로 연결했다.
현재 source 읽기와 변경 영향 대조에 사용하는 **과거 실행 근거**이며 이번에 테스트 18개를 새로 통과한 것이 아니다.
X02의 최초 exit 101과 교정 후 exit 0도 원 기록에 함께 보존한다.

## 3 설계 개선의 보존과 후속 변경 영향

| 설계 항목 | 최종 판정 |
| --- | --- |
| 책임·상태 소유권·실패 | application root의 게시, Battle timestamp/cursor, Run 정산 receipt, host accepted intent, Trial accepted aggregate가 각기 다른 확정 경계를 소유한다. 중복 mutable 원본을 추가하거나 서로 다른 rollback 의미를 하나로 합치지 않음 |
| 규칙 중복·변경 비용 | S07 승리 후처리의 기존 helper 재사용, S08 생성/정리 본문 통합, S10 whole-step 보정 합산의 단일 정의·네 호출이 현재도 유지됨. 동일 규칙을 여러 곳에서 수동 변경하던 부담이 줄어듦 |
| 수명·결정론 | S08 자원 deployment·buff 정리 순서, S09 단일 death epoch/stop event와 실제 lock/segment, source/copy/lease와 checkpoint ID stream 구분을 유지함. 관측 sequence를 gameplay RNG 원본으로 만들지 않음 |
| 의존 방향·공개 범위 | 공통 helper는 내부 책임에 있고 새 public DTO·schema·registry·범용 adapter를 만들지 않음. target usefulness가 실제 executor 규칙의 독립 owner가 되는 의존을 제거함 |
| 데이터·오류·관측 | validated generation·immutable plan·실행 권위·관측 기록을 구분한다. S01/S04/S02/S08/S09/S10 D1은 설명을 실제 owner·수명·게시 순서에 맞췄으며 새 gameplay 정책으로 사용하지 않음 |
| 검증 가능성 | B1 공개 command/원본 저장 복원, S09 실제 event 순서, S10 실제 HP, X02 실제 main 실행이 원래 문제와 연결됨. helper 호출 횟수만 고정한 새 시험으로 대체하지 않음 |
| 성능·자원 | bounded budget·retained lifetime·clone admission의 기존 소유권 유지. 이번 국소 추출/삭제에서 시간·할당량 개선을 주장하지 않으며 측정 없는 복사 제거는 비필수 보류 |

후속 변경의 누적 영향도 대조했다.

- S08 R2는 생성 R1 주변 본문을 보존하며 death/withdraw 정리를 공통화했다.
- S09는 항상 false였던 helper를 제거하면서 death의 기존 항상 실행되던 fallback을 같은 위치에 남겼다.
  R2 정리 순서와 실제 stop event·epoch 의미는 보강 시험으로 확인됐다.
- S10은 `sim.rs`의 서로 다른 두 합산 위치를 같은 순수 함수 호출로 바꿨다. S09가 보존한 cast lock/취소 순서를 바꾸지 않았다.
  `mod.rs`의 후속 변경은 실제 효과 테스트 추가이며 앞선 정리 helper를 바꾸지 않았다.
- X01은 실행 코드 변경 없이 소비·실패·관측 경계를 확인했다. X02-B1은 generator의 command 입력 세 지점만 교정했다.
  게임 실행·상태 schema에 새 경로를 추가하지 않았다.

[누적 대상](evidence/integration_ready_001/change-chain.json), S08 R2·S09 R1·S10 R1의 구조 대조,
현재 helper 정의·실제 호출 및 교정된 generator를 함께 읽었다. 새 중복·임시 전환 경로나 선행 개선의 회귀는 확인하지 못했다.

## 4 검증 재사용의 범위

기본 Core 전수 **2,236개 통과·기존 ignore 6**은 S08 R1 이후, R2/S09/S10 이전의 결과다.
후속 영향 검증 **R2 18개 / S09 17개 / S10 13개**, 관측 feature **13개**,
계약 fixture **4개 생성 결과**는 각 해당 변경 시점의 근거로 구분한다.
테스트 간 중복을 합산한 새 통과 수나 최신 source의 전수 통과로 표기하지 않는다.

현재 검토에서 변경되지 않은 연결과 이미 확인한 구조적 동등성에 근거해 재사용했다.
입력·설정·build 의미·지원 경로가 달라졌거나 이 연결에서 실제 모순이 드러났다면 좁은 재검증이 필요하지만,
이번 선택 대조에서는 그런 원인을 확인하지 않았다. 따라서 전수·release·전체 feature 반복은 배정하지 않았다.
전체 공식 데이터·전이 의존성·외부 환경을 새로 동결한 결과가 아니며, 이후 다른 입력으로 실행할 때의 통과를 보장하지 않는다.

## 5 README 완료 조건과 잔여

| 완료 조건 | 판정·근거 |
| --- | --- |
| 시스템·공유 경계·포함/제외 연결 | 충족. 지도, 원 시스템 조사, 준비 §4의 X01/X02 보완과 이 기록 I1–I5 |
| 선정 흐름의 결과·근거 | 충족. 실제 생산자/소비자와 지원된 실패·복원을 대조. 내부 fixture·공개 경로·host 모의 crash를 구분 |
| 확정 후보의 처리 | 충족. S07 B1/R1, S01 D1, S04 D1, S02/S11 D1, S08 R1/R2/D1, S09 R1/D1, S10 R1/D1, X02-B1 종결. 나머지 대표 구조 유지 |
| 필수 개선의 보존·효과·검증·검토 | 충족. 원 문제와 실제 변경·전후 근거·집중 검증·분리 검토가 연결됨 |
| 후속 변경과 최종 통합 | 충족. §2–3의 연결과 누적 변경 대조, 선택 40개 입력 일치 |
| 필수 잔여와 비필수 제한 구분 | 충족. 선정 범위의 필수 미해결 없음. 아래 제한은 보증 범위를 정하며 숨겨진 완료 조건으로 넘기지 않음 |

남기는 제한과 재개 조건은 다음과 같다.

| 제한·비필수 항목 | 재개 조건 |
| --- | --- |
| 중첩 store/transaction 복사, 장기·최대 규모 비용 | 대표 workload의 측정 요구나 실제 비용 문제 발생 시 성능 owner에서 실패 보존 조건과 함께 평가 |
| comparator fixture 24쌍/31 variant와 모든 효과·활동·성장·저장 조합 미포괄 | 새 variant/순서/해당 도메인 변경 또는 구체적인 미검증 연결이 생기면 그 범위부터 보완 |
| 전체 release/feature 동등성·Unity 소비·서버 exporter | 해당 배포·feature·외부 계약 작업의 실제 실행 환경에서 검증 |
| 승인 cache/worker/calibration A/B matrix·기존 ignore 6개 | 원 관리 환경과 입력을 갖춘 승인 검증 작업에서 처리. 이번 통과에 포함하지 않음 |
| B1 이전 불일치 저장 migration·장치 전원 손실 | 구체적인 저장 복구/운영 요구와 해당 입력이 있을 때 별도 범위로 판단 |
| 임의 내부 상태 변조 방어·전면 API 비공개화·crate 분리 | 현재 지원 경로 또는 구체적인 의존/변경 문제로 필요성이 확인될 때 판단 |

## 6 최종 상태와 재개

[최종 검토 기록](evidence/integration_final_001/review.json)과
[문서·근거 확인](evidence/integration_final_001/verification.json)에 판정 대상을 연결한다.
이번 **선정 대표 조사·개선 범위는 종료**한다. 추가로 자동 배정할 필수 구현·검증은 없다.
새 기능·변경·실패 재현·측정 요구가 들어오면 해당 owner와 영향받는 연결부터 재개하며, 이 완료를 새로운 입력에 자동 적용하지 않는다.
프로젝트에 더 이상 개선할 곳이 없다는 선언이나 전체 외부 제품의 배포 승인으로 사용하지 않는다.

후속 실행 요청으로 [성능·자원 1차 측정](PERFORMANCE_RESOURCE_MEASUREMENT.ko.md)을 재개했다.
이 기록의 선정 기능 범위 완료는 유지하며 새 성능 초과 finding·다음 행동은 최신 README와 측정 기록을 따른다.
