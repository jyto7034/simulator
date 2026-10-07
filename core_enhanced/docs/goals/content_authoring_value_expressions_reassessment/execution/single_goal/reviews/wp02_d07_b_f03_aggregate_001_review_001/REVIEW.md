# D07-B F03 C06 successor 단계검토

2026-09-28 · **PARTIAL_SCOPE_PASS — C06 집계 소비·기록 재평가 범위 통과**.

진영·실제 직원·지정 장비 사본별 node view, 명시적 필터, 복수 node 식이 실제 command 소비와 Snapshot/Live 기록 재평가에 연결됨을 확인했다. 이번 범위에서 추가 교정을 요구하는 finding은 없다. C07–C15 및 C03-OBS01은 미완료다.

## 대상과 검토 경계

- [구현 인계서](../../corrections/wp02_d07_b_f03_aggregate_001/REVIEW_HANDOFF.ko.md), [동결 manifest](../../corrections/wp02_d07_b_f03_aggregate_001/target/d07_b_f03_aggregate_001/MANIFEST.json).
- Manifest SHA-256: `0aed6d9be20ece81d8daee39d0dc26314115deac8a8396073152244c8bef354b`.
- before 343개 digest: `f9883de09a463ad9de856e6cbc3022abc2c27a0b5101cc71f9df9e7c488a7e51`.
- after 345개 digest: `fa40803901eb1d1f1df670d85617eb69de0bcbc4603f667db0d94e14966a53a2`.
- C06 변경: `value_expr_runtime.rs`의 cfg(test) 하위 module 선언 1줄과 새 `tests/aggregate_replay.rs`. 선행 342개 파일 byte 동일.
- 별도 입력: `core_enhanced/tests/battle_showcase.rs`가 동결 after에 추가됨. C06 작성 변경과 구분한다.

사본 대화에서 구현 delta·실제 코드 경로·시험과 동결 근거를 검토했다. 설계를 공유한 단계검토이며 설계까지 독립된 감사는 아니다. 시연 테스트의 최초 작성은 이 대화에서 사용자가 별도로 요청한 작업이다. 이번 판정은 그 파일의 새 독립 품질 감사가 아니라 C06 검증 입력으로서의 identity·최종 실행 포함 여부를 확인한다. 이번 검토 중 제품 소스와 동결 자료는 수정하지 않았다.

## 코드와 시험 판단

fixture는 실제 catalog의 authored plan으로 action과 ObservationKey를 연결하고, finite delivery의 실제 release capture를 만든다. 입력 변경 후 기존 transaction 안의 ordered executor/command 소비 경계를 실행하며, source template과 runtime_fact_source를 같은 delivery에 연결한다. record/frame을 시험이 직접 만들거나 release 직전 값을 consume 때 다시 캡처하지 않는다.

복수 node 식의 독립 기대값은 다음과 같다.

| node의 관계 | Snapshot | Live |
| --- | --- | --- |
| Side별 Max의 합 | 18 | 19 |
| 실제 직원별 Max의 합 | 8 | 13 |
| 직원·첫 슬롯 필터 후 장비 사본별 합 | 8 | 4 |
| 직원·둘째 슬롯 필터 후 장비 사본별 합 | 3 | 4 |
| 전체 Add | **37** | **40** |

ByType의 Weapon/Armor와 Any3의 0/1 슬롯을 각각 Snapshot/Live로 실행한다. 초기 공격력 3/5/11/7을 release 뒤 4/9/13/6으로 바꾸고 B의 첫 슬롯을 제거한다. 실제 command 4건은 모두 37 또는 40이고 네 대상 HP는 63 또는 60이다. 집계 필터가 피해 대상 자체를 줄이지 않는다는 점도 검증한다.

각 node의 member identity, owner domain과 실제 직원/장비 사본 key, 공격력, canonical 순서를 별도로 검사한다. predicate가 사용하는 ID를 포함한 aggregate/read 번호 `(1,2)/(3,5)/(6,9)/(10,13)`와 최종 Add를 따라 전체 trace 기대 벡터를 만든다. Snapshot 14개 / Live 13개 trace가 일치한다. 값만 같은 두 query를 비교하는 것으로 node view 혼합을 놓치지 않는다.

Snapshot은 release frame의 같은 Arc, Live는 snapshot frame 없는 capture에서 소비 시점 frame을 사용한다. 소비 후 공격력과 A의 두 슬롯을 다시 바꿔도 frame·trace와 문서가 보존된다. 반복 조회 bytes·usage와 gameplay/queue/facts/event log/record Arc/순번 불변을 확인한다.

빈 필터 결과는 실제 resolved target 4명을 유지하면서 command 없이 0 평가를 기록한다. 두 moment 모두 HP100·ordinal4·빈 view·단일 0 trace를 확인한다. 부적격 관계 4종은 먼저 성공한 record A를 만든 뒤, 다음 transaction의 선행 Damage1이 후보에서 실제 소비된 것을 tap으로 확인하고 후속 관계 오류를 발생시킨다. InvalidOwner 3종 / MissingContext 1종 거부 후 accepted 상태와 동일 Arc A·ordinal1·두 query가 보존된다. 이 시험이 C10의 모든 실패·취소 경우를 대신하지는 않는다.

장비 변화는 private fixture 조작이고 조회는 `encode_record_for_test`를 통한 실제 인코더 재평가다. 실제 장비 교체 UI, 투사체 이동·충돌 queue 전체, 공개 Latest/Exact selector·권한·terminal·수명·비용 한도까지 새로 검증했다고 확대하지 않는다.

## 검증·외부 입력·재사용

- 직접 실행: `cargo test -p game_core --lib d07_f03_c06_ --offline --locked -- --nocapture --test-threads=1`, **3 passed / 0 failed**. 출력 대조를 위해 집중 시험만 한 스레드로 실행했다.
- 직접 원문 대조: 10개 시나리오, preview/explain 18쌍, typed refusal 4종. aggregate 결과·node별 member 수/정렬·trace 길이·ordinal·시각 및 상태 변경 전후 문서 동일성을 확인했다.
- 최종 영수증 7건의 source 전후 digest·log hash·test summary 확인. 전체 **1,556개**, authoring 112개, value_expr 104개 및 check·Clippy·변경 format 통과. 전체 suite는 구현자의 동결 근거를 대조했으며 검토자가 다시 실행하지 않았다.
- `WORKSPACE_ADDITION.json`의 시연 파일 4,752 bytes / SHA `bb580d624ebac8e075231dd73a27e7eb4dde48d7f9f38bd14708c7a3a3c669ae`가 현재 파일과 manifest after에 일치한다. 최종 전체 로그에 현재 함수 `test_nothing_there_first_cast`의 1개 통과가 포함된다. **1,552 + C06 3 + 시연 1 = 1,556**이다.
- 초기 344개 입력으로 생성된 영수증은 외부 파일을 결속하지 못한 이전 시도로 보존하며 최종 근거로 쓰지 않는다. 최종 7건 모두 345개 입력 기준임을 확인했다. before를 소급 수정하지 않고 두 추가 파일을 격리 정·역 patch에 포함한 방식도 타당하다.
- currentness: source membership 345개, 동결 기록 472개, 격리 343→345→343 byte 복원과 predecessor source row 일치 확인.
- PC36/R05는 선행 F02 측정 재사용이다. parent에서 cfg(test) module 선언 한 줄만 제거하면 predecessor와 byte 동일하고 신규 두 파일은 시험 입력이다. 선행 측정 자료 hash·재사용 연결을 확인했다. 새 benchmark나 관측 on 비용 보장이 아니다.
- PC36 성공31 / typed 거부5, R05 raw exit101 및 peak9,049,037 bytes / cap 초과18,849 bytes 단일 경고를 유지한다.

근거: [집중 실행 로그](FOCUSED_RECHECK.log), [영수증·출력 대조](SCOPE_AND_RECEIPTS_CHECK.json), [시작 확인](ENTRY_CURRENTNESS_AND_RESTORATION.json), [종료 확인](EXIT_CURRENTNESS_AND_RESTORATION.json), [검토 결속](REVIEW_VALIDATION.json).

## 후속

선행 F01·F02·C03 runtime·C04/C05의 부분 PASS를 유지한다. 다음 후보는 [기존 교정 지시서](../../handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md)의 **C07**이다. 복수 effect/target·같은 timestamp·piercing delivery의 기록 순번, 실제 A→B 교체 후 A Exact의 not-retained, zero와 무대상 impact의 차이를 실제 선택 경로로 확인한다.

후속은 사용자 재개 요청 후 원본 구현 대화에서 진행한다. C07–C15와 C03-OBS01(RON MIN 입력/직렬화 문제)의 미해결 의무를 유지한다. C16 확인은 이번 checkpoint에만 적용하며 F03·D07-B·WP02·R06 전체 완료는 아니다.
