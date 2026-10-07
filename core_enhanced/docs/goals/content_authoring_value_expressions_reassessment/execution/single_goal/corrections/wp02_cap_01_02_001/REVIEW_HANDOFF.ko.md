# R06 WP02 CAP-01·CAP-02 교정 단계검토 인계

상태: **교정·자체 검증 종료 / 단계검토 타깃 동결 / 재검토 요청 대기**. 이 문서는 재검토 PASS 또는 WP02·Phase·Goal 완료를 선언하지 않는다.

## 기준과 변경 경계

- 직전 [F02-R01 검토](../../reviews/wp02_f02_r01_correction_002_review_001/REVIEW.md)의 PASS와 F03 교정을 유지했다. 직전 [타깃 manifest](../../corrections/wp02_f02_r01_001/target/f02_r01_correction_002/MANIFEST.json)의 SHA-256은 `ae2c540157c9caa769febb56e2664da7be8b5807b917e4ecfcb685103aa5f0f7`이다.
- 수정 전 실제 source 333개와 고유 입력 3개의 bytes를 [새 baseline](baseline/entry_001/MANIFEST.json)과 [before archive](baseline/entry_001/SOURCE_AND_INPUTS_BEFORE_CORRECTION.tar.gz)에 고정했다. baseline manifest SHA-256은 `21f63a5f801cd40e51488876ed1acac7faca851a4d439240d0dfbbbf2ad87b5c`이다. 기존 dirty/untracked, 이전 baseline·target·probe·로그는 변경하지 않았다.
- 제품 변경은 5개 파일이다. `authoring/control.rs`의 남은 work·container 상한 조회, `battle/runtime_control.rs`의 공통 성장 승인, `battle/core/runtime_facts.rs`의 CAP-01과 actor extent/COW, `battle/core/queue.rs`의 CAP-02와 timestamp bucket, `battle/core/value_expr_runtime.rs`의 action–cast 제품 회귀 시험이다. 새 공개 API·직렬화 필드·전역 registry·gameplay 예산 숫자는 없다.

## 비용 소유권과 선택

성장 배수는 **1.5배, 첫 예약 최소 2칸**으로 택했다. checked `len+1`과 곱셈, 기존 actor/pending/bucket 원소 상한, 남은 retained 및 work 허용량으로 후보를 제한한다. 여유 예약만 불가능하면 최소 필요량으로 낮추고, 그조차 불가능하면 기존 첫 fault를 반환한다. 매번 새 버퍼 전체 요청량을 실제 `reserve_exact` 전에 한 번 승인한다. 2배 후보는 R05 원본 동시 peak 한도를 재현성 있게 넘었으므로 [실패 자료](verification/r05_guard_current_quiet.json)를 보존하고 1.5배로 줄였다.

| 대상 | 기존 비용 소유권 | 이번 성장 후 계산 |
| --- | --- | --- |
| actor sorted rows 삽입 | authoring shift work; native retained의 새 버퍼 전체와 authoring work `ceil(bytes/8)` | 동일 소유권. 필요 최소·배수 후보 중 승인 가능한 capacity를 선택하고 전체 새 배열+64를 선청구. 기존 버퍼 해제는 같은 operation의 누적 청구를 환급하지 않음 |
| actor COW 및 다음 operation | COW는 살아 있는 row 복사·scan; live owner는 retained extent에 포함 | COW는 source spare를 복사 비용에 넣지 않음. live owner extent는 실제 `rows.capacity()`를 반영하고, 다음 operation seed도 이를 사용 |
| heap/bucket slot | enqueue/resource work와 native retained의 새 slot 버퍼 전체 | `BinaryHeap`/timestamp `Vec`의 전체 새 capacity+64를 선청구. heap comparator·기존 배열 순서·동일 key 순서는 그대로. pop/clear 후에도 heap이 보유한 capacity를 다음 seed에 포함 |
| 이벤트 payload/metadata | 큰 `BattleEvent`, fact source, Snapshot, completion/source owner 별도 | slot 예약과 분리된 기존 선청구를 유지. 실패 latch는 바깥 operation에서 드러나며 실패 enqueue를 성공으로 처리하지 않음 |

`preflight_runtime_fact_clone`의 heap scan은 실제 `len+1`칸을 포함한다. clone은 살아 있는 원소만 복사하고 원본의 여유 capacity를 새 버퍼 복사량으로 중복 청구하지 않는다. ActorSummaries의 ID 정렬·caused/received와 기존 child/proc/delivery, cast 없는 source, source 퇴장 후 Snapshot, F01·F03 경계는 유지했다.

## 직접 측정과 회귀

원본 [PC fixture](../../../../../tmp/r06_pc_baseline_20260926/probe.rs)를 변경하지 않고, 수정 전 baseline source와 수정 후 현재 source를 별도 `/tmp/r06-cap-before-source`·`/tmp/r06-cap-after-source`, 별도 release build target에서 실행했다. 계측 전용 `baseline_usage` adapter와 test hook만 격리 사본에 추가했고 [333파일 대조](verification/pc_source_identity.json)가 확인한다. [원본 before](verification/pc_before_raw.json), [최종 after](verification/pc_after_raw.json), [36조건 비교](verification/pc_comparison.json), [실행 로그](verification/pc_after.log)를 보존했다. 각 조건은 시간 11회·allocation 3회와 모드별 warmup 1회를 같은 구간에서 측정했다.

| 조건 | 수정 전 | 최종 수정 후 |
| --- | --- | --- |
| 적 300명, 단일 source 피해 | `actor_row_reserve`에서 거부; HP 변화 0 | 312유닛, 300적중·HP 300 감소, 요청 allocation 합계 1,187,088 bytes |
| 적 300명, 12 source 피해 | 같은 첫 source 거부 | 312유닛, 3,600적중·HP 3,600 감소 |
| 적 500명 동시 생성 | setup의 `queued_event_slots`에서 거부 | 512유닛 구성; public 1초 진행 양쪽 분할에서 60적중·HP 60 감소 |
| 적 500명, 1/12 source 피해 | setup 거부 | 512유닛, 각각 500/6,000적중·HP 500/6,000 감소 |

36조건 중 거부는 11→5개다. 남은 5개는 기존 128 Read 반복의 `value_expr_evaluation_retained` 거부이며 정책을 완화하지 않았다. 수정 전·후 모두 완료된 100/300명 전투와 100명 피해의 최종 log hash·HP 결과가 일치했다. 수정 전 거부까지 시간과 수정 후 완료 시간을 성능 배수로 나누지 않았다. 측정 allocation은 alloc/realloc **요청 합계**이다. PC fixture는 동시 heap peak와 RSS를 재지 않았으며 이를 요청 합계로 대체하지 않는다. 이 정적 fixture는 전체 수백 적 공세의 성능 판정이 아니다.

R05 원본 concurrent allocation guard의 cap `9,030,188`을 변경하지 않았다. 동일 입력/도구의 직접 before는 두 번 통과했고, 2배 성장 후보 및 첫 최소 2칸 후보의 실패는 각각 [자료](verification/r05_guard_current_quiet.json), [자료](verification/r05_guard_min2.json)에 남겼다. 최종 1.5배 후보는 세 번 모두 통과(15 표본 최대 `9,013,095` bytes)했으며, 최종 source bytes로 다시 빌드한 [확인](verification/r05_guard_final_exact.json)도 통과(최대 `9,010,868` bytes)했다. 동시 peak는 이 R05 fixture의 프로세스 내 계측값이며 PC fixture 요청 합계나 RSS와 다르다.

제품 회귀 시험에는 다른 actor cast, 같은 actor의 다른 실행 cast를 capture에서 거부하고 정상 source·cast 없는 source·종료된 제어 단계의 delivery·source 퇴장 후 Snapshot을 허용하는 두 시험을 편입했다. CAP 단위 시험은 빈 컨테이너, 단계적 성장, 원소 상한, 최소 필요량 fallback, byte/work 부족, overflow, actor COW 독립성, queue clone/clear/order 및 실패 후 재시도를 확인한다. 기존 F02 actor/provider, F03 resource/lifecycle, F01 선청구·취소는 [전체 제품 시험](verification/all_target_test.log)에 포함해 1,475개가 통과했다. [all-target check](verification/all_target_check.log)와 [기존 allow 조건 Clippy](verification/clippy_with_existing_allows.log)도 종료 코드 0이다. 최초 sandbox 전체 시험의 Unix socket 권한 실패는 제품 결함이 아니며, 같은 최종 코드를 격리 build target과 승인된 권한에서 전체 재실행해 통과했다.

Focused `rustfmt --check`는 [수정 전 37개](verification/focused_rustfmt_baseline_exact.log), [최종 57개](verification/focused_rustfmt.log)의 hunk 차이로 종료 코드 1이었다. 기존 파일 전체 포맷 변경은 이 교정 범위를 늘리므로 적용하지 않았다. 새 hunk 20개는 미해결 스타일 검토 항목이며 기능 시험 PASS로 포맷 PASS를 대체하지 않는다.

| 증거 구분 | 범위 |
| --- | --- |
| 이번에 직접 실행 | 직전 target currentness/restore, 실제 수정 전·후 PC 36조건, 최종 source identity, R05 같은 cap의 직접 before·후보·최종 재실행, 새 CAP/결속 제품 시험, 전체 제품 check/test, Clippy, focused rustfmt |
| 이전 증거 재사용 | F02-R01 단계검토 PASS 판단과 검토자 원본 probe, R05의 입력 fixture·9,030,188 cap, PC 원본 fixture·타이밍 구간. 이전 F01 시점 PC 수치를 이번 진입의 before로 쓰지 않음 |
| 이번에 미실행 | 검토자 원본 probe 별도 실행기 전수 재실행, PC 동시 heap peak·RSS, 다른 플랫폼과 전체 동적 공세, 제외된 WP03·publication·순차 effects/result-prefix |

동결 증거는 [after target](target/cap_01_02_correction_001/MANIFEST.json), [after archive](target/cap_01_02_correction_001/SOURCE_AFTER_CORRECTION.tar.gz), [복원 delta](target/cap_01_02_correction_001/SOURCE_DELTA_FROM_ENTRY.tar.gz), [검증 로그](verification/target_integrity.json)에 연결한다. 미실측/미진입 범위는 PC 동시 peak·RSS 및 전체 수백 적 동적 공세, 다른 플랫폼, 순차 effects/result-prefix, WP03, publication 해제다. 재검토자는 우선 새 포맷 hunk와 비용 경계 및 보존된 36조건·R05 증거를 확인하면 된다.
