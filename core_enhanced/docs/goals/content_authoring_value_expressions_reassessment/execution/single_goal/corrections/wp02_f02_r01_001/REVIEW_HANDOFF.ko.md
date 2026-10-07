# R06 WP02 F02-R01 교정 단계검토 인계

상태: **교정·자체 검증 종료 / 단계검토 타깃 동결 / 재검토 요청 대기**. F02-R01만 교정했다. 이전 재검토의 F03 부분 교정 PASS 판단은 소급 변경하지 않는다. 이 문서는 재검토 PASS, WP02·Phase·Goal 완료 또는 publication 해제를 선언하지 않는다.

## 기준과 보존

- 입력: [재검토](../../reviews/wp02_f02_f03_correction_001_review_001/REVIEW.md), 변경하지 않은 [검토자 추가 probe](../../reviews/wp02_f02_f03_correction_001_review_001/additional_probes.rs), [기존 교정 계획](../../handoffs/WP02_CORRECTION_PLAN_20260926.ko.md).
- 직전 타깃 [MANIFEST](../../corrections/wp02_f02_f03_001/target/f02_f03_correction_001/MANIFEST.json)의 SHA-256은 `84a7b1a6f4d40973c5fbc148c00457911e966a20f046e4aa6afbeac609897b23`이다. 현재 파일 333개의 bytes와 그 manifest를 구현 전 대조해 일치했다.
- 수정 전 실제 333개 source와 입력 5개를 [새 baseline](baseline/entry_001/MANIFEST.json), [before archive](baseline/entry_001/SOURCE_AND_INPUTS_BEFORE_CORRECTION.tar.gz)에 고정했다. baseline manifest SHA-256은 `ad2b42af4cc49e0b0077ae175a273fbb38aee3bf18fc53624e9ed79978fe8e5d`다. 계획 문서가 source 집합에도 있어 archive의 338항목 중 고유 경로는 337개이며, 중복 항목의 bytes는 동일하다. 기존 baseline·target·검토 증거는 덮어쓰지 않았다.

## 결속 책임과 변경

`invoke_ability_with_accepted_target_tile`의 순서는 action 수락 및 release → cast 실행 정보 생성 → source/carrier 전달이다. 수락 당시 action snapshot의 `cast=None`은 나중 cast의 허가가 아니다. cast 실행 정보가 만들어진 직후, `RuntimeFactState`의 해당 active action이 actor·skill·유효 action을 확인하고 cast Arc를 한 번 결속한다. 이 관계는 action이 delivery를 보유하는 동안 남고, capture의 `validate_action_source`가 전달된 cast의 Arc 동일성을 확인한다. 같은 actor·skill의 다른 `cast_seq`와 값만 복제한 cast도 통과하지 않는다.

변경 파일은 `src/game/battle/core/runtime_facts.rs`, `src/game/battle/core/sim.rs` 두 개다. 결속은 기존 action owner의 비직렬화 private 필드에 두었으며 전역 registry, 공개 token, 직렬화 필드를 추가하지 않았다. cast 없는 basic/non-cast source는 그대로 허용한다. child/proc/delivery의 authored metadata 동등성 검사도 추가하지 않았다. active cast 목록이 지워져도 남은 action/delivery가 결속을 검증한다. source 퇴장 뒤 이미 만든 Snapshot 소비는 기존 수명 규칙을 따른다. 결속 보유 extent는 기존 runtime fact retained 예산에 청구한다.

## 검증과 증거

| 검증 | 결과 | 증거 |
| --- | --- | --- |
| 수정 전 검토자 두 실패 probe와 정상 생산 경로 대조군 | 2 FAIL / 1 PASS로 결함 재현 | [before 로그](verification/before_reviewer_late_cast_from_other_actor_must_not_bind_to_first_action.log), [같은 actor 로그](verification/before_reviewer_late_cast_from_other_action_same_actor_must_not_bind.log), [대조군 로그](verification/before_reviewer_actual_cast_producer_positive_control.log) |
| 수정 후 원본 검토자 9개 및 새 probe 3개 | 12 PASS | [최종 격리 로그](verification/after_reviewer_probes_exact_source.log), [새 probe](verification/f02_r01_probes.rs) |
| 제품 all-target test, debug/offline/locked | 1,469 PASS | [전체 로그](verification/all_target_test_escalated.log) |
| 제품 all-target check | PASS | [check 로그](verification/all_target_check.log) |
| 기존 allow 조건의 all-target Clippy | PASS | [Clippy 로그](verification/clippy_with_existing_allows.log) |
| focused rustfmt | `runtime_facts.rs` PASS; `sim.rs`의 기존 13개 포맷 차이 유지, 추가 차이 0 | [receipt](verification/focused_rustfmt.json) |

추가 probe는 다른 action, 같은 데이터를 복제한 cast, 다른 attempt를 capture 단계에서 거부하고 gameplay snapshot 불변과 원본 source 재시도 성공을 확인한다. 제어 단계를 종료시키고 active cast를 제거한 뒤 남은 delivery의 capture 성공, source 퇴장 후 Snapshot 소비, cast 없는 source 성공도 확인한다. 기존 actor/provider·scenario equipment·F03 자원/생명주기·F01 선청구/취소·child/delivery 관련 테스트는 전체 제품 suite에 포함돼 통과했다. 원본 검토자 assertion은 수정하거나 삭제하지 않았다.

격리 release 시험은 `/tmp/r06-f02-r01-isolated`와 그 전용 `target`을 사용했다. 전체 제품 검증도 같은 별도 build target을 사용했다. 첫 전체 시험은 release fixture 조건이 달라 컴파일되지 않아 [로그](verification/all_target_test.log)를 보존했고, debug 시험의 Unix socket 1개는 sandbox 권한으로 실패해 [로그](verification/all_target_test_debug.log)를 보존했다. 동일 debug 명령의 승인된 실행은 전부 통과했다. 두 실패를 제품 회귀로 세지 않는다.

동결 증거: [after target](target/f02_r01_correction_002/MANIFEST.json), [after archive](target/f02_r01_correction_002/SOURCE_AFTER_CORRECTION.tar.gz), [복원 delta](target/f02_r01_correction_002/SOURCE_DELTA_FROM_ENTRY.tar.gz), [복원 검증](verification/target_integrity.json). 첫 후보 `f02_r01_correction_001`은 baseline 중복 경로를 거부한 검증 실패와 함께 보존했고 최종 단계검토 타깃으로 사용하지 않는다. 별도 재검토자는 원본 probe와 이번 추가 probe를 재사용해 capture 거부와 정상 대조군을 확인할 수 있다.

미실측: 외부 pack publication, PC 36조건과 CAP-01/02, 다른 플랫폼, 순차 effects/result-prefix, WP03. 이번 probe의 cast controller 제거는 상태 전이를 직접 구성한 수명 시험이며, 제품의 모든 projectile/area 조합을 전수 재현했다는 뜻이 아니다. 전체 제품 suite의 실제 released area 회귀는 실행했다.
