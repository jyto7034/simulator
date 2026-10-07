# R06 WP02 D04-F01 교정 독립 재검토

작성일: 2026-09-26. 대상: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`, WP02 부분 구현.

**D04-F01 기능 교정은 PASS이며 기존 finding은 해결됐다. 다만 R05 비용 검증의 유효한 실패가 남아 있어 이 successor의 단계 전체 PASS는 보류한다.**
새 기능 결함은 이번 교정 범위에서 발견하지 못했다. 아래 R05-E01은 비용 검증 미해결 사항이며, D04-F01이 allocation 증가의 원인이라는 판정은 아니다.

## 1. 대상과 증거 결속

- [교정 인계서](../../corrections/wp02_d04_f01_001/REVIEW_HANDOFF.ko.md)
- [Successor manifest](../../corrections/wp02_d04_f01_001/target/d04_f01_correction_001/MANIFEST.json)
- Target SHA-256: `e4f0d77b1bd3066493d605271605d7a4b6e16317a6d540d303909b076b480104`
- 최종 333개 source digest: `34ceedbebc84b9396d42f92fa31033fe0b831a90e5325c8c4463e0c88f067145`
- 선행 D04 target SHA-256: `24bd3b7e748b94037b2ec57822a7b7ef2b67406b9d2635a3a23a6b3eac7ba8bf`
- [원 finding](../wp02_d04_resolved_target_001_review_001/REVIEW.md), [교정 지시서](../../handoffs/WP02_D04_F01_CORRECTION_REQUEST_20260926.ko.md).

변경된 `core/mod.rs`, `core/sim.rs`, `core/value_expr_runtime.rs`, `data/mod.rs` 4개 파일을 검토했다.
입구·출구 `freeze.py verify`에서 currentness와 양방향 복원을 확인했다.
[verify.py](verify.py)와 [verification.json](verification.json)은 after archive의 정확한 파일 집합·내용,
records/artifacts, 최종 receipt의 source 전후·로그, PC 격리 copied/built/run identity와 R05 선행 대조의 source를 확인한다.
제품 소스·기존 동결본·기존 검토 기록은 수정하지 않았다.

## 2. D04-F01 — 해결 / 기능 PASS

`execute_ordered_skill_effects`는 실제 target이 있을 때만 target-bound capture lookup과 Live member 준비를 수행한다.
빈 set에서는 carrier 및 해당 effect row의 존재를 확인하고 `Some(Vec::new())`를 command builder에 전달한다.
effect command/signal, projection 종료, local prefix/action witness settlement와 step/delivery 완료 경로는 계속 실행된다.

`capture_for`의 matching은 바뀌지 않았다. 실제 target의 identity 검증과 모든 amount를 mutation 전에 계산하는 순서도 유지됐다.
`has_effect_capture`는 empty 경로에서 row 존재를 확인하는 제한된 helper다. 새 fallback capture나 전역 성공 처리는 없다.
자원 fixture를 위한 `with_resources`는 기존 `GameDataBuilder`의 test/debug/test-fixtures 범위 안에 추가됐고,
실제 resource runtime 정책은 바뀌지 않았다.

독립 검토 실행:

| 소스/시험 | 결과 | 증거 |
| --- | --- | --- |
| 그대로 추출한 successor 소스의 신규 제품 회귀 | **9/9 PASS** | [로그](focused_correction_002.log), [receipt](focused_correction_002.json) |
| 이전 독립 검토의 probe 원문을 그대로 연결한 사본 | **4/4 PASS** | [probe](projectile_probes.rs), [로그](original_reproductions_001.log), [receipt](original_reproductions_001.json) |

9건은 removed/inactive/timeout의 empty impact, terminal log·cast/projectile/queue·action 정리,
빈 DamageExpr/HealExpr 뒤 SourceEmployee Grant 1회, step count, capture mismatch,
실제 TargetEmployee InvalidOwner의 candidate rollback과 이전 commit 보존을 확인한다.
이전 실패 4조합(상수식/ResolvedTargets Sum × Snapshot/Live)이 모두 통과해 원 finding의 해결을 확인했다.

최초 독립 빌드는 linker Bus error로 실패했고 시험은 실행되지 않았다. 당시 `/tmp`는 16GiB 중 가용 467MiB였다.
공간 부족이 원인일 가능성을 보고 검토자가 만든 package build cache만 정리한 뒤, 같은 소스·selector에
`CARGO_INCREMENTAL=0`을 사용해 재실행했다. 9건이 통과했으며 최초 실패도 보존했다.
원인의 유일성을 입증한 것은 아니다. [환경 기록](environment_note.json)에 조치와 한계를 구분했다.

## 3. R05-E01 — 필수 비용 검증 미해결 / 단계 수용 보류

최종 소스의 첫 두 R05 실행은 모두 정상 측정 후 원 allocation guard assertion에서 exit 101로 종료됐다.
이 실패는 source가 다른 중간 revision, 빌드 실패 또는 sandbox 오류로 제외할 수 없다.

| 실행 | source | 5개 sample 중 최대 peak | 원 cap 대비 | guard |
| --- | --- | ---: | ---: | --- |
| `r05_guard_final_001` | 최종 교정본 | 9,040,041 | +9,853 | FAIL |
| `r05_guard_repeat_001` | 같은 최종 교정본 | 9,040,546 | +10,358 | FAIL |
| `r05_predecessor_control_002` | 동결 D04 대조 | 9,050,125 | +19,937 | FAIL |
| `r05_guard_after_control_001` | 같은 최종 교정본 | 9,020,761 | −9,427 | PASS |

원 cap은 **9,030,188 bytes**다. [r05_assessment.json](r05_assessment.json)은 summary뿐 아니라 각 raw receipt를
직접 읽어 hash·tool/input baseline·sample peak·guard 결과를 대조한 기록이다.
세 교정본 실행의 전체 15 sample 중 2개가 cap을 넘었다. 이 관찰을 실패 확률 추정으로 확대하지 않는다.

`tools/runtime_architecture_final_validation/src/final_performance.rs`의 `allocation_guard`는
`samples.iter().all(...)`로 allocation 횟수·peak·retained 각각을 baseline 상한과 비교한다.
main은 적용 대상 allocation guard를 assert한다. 평균·중앙값이나 마지막 실행만 채택하는 예외는 없다.
[기존 D08](../../../../CODE_REVIEW_003.ko.md)은 cap을 침범하면 구조를 줄이거나 required evidence blocker로 기록하도록 한다.
따라서 마지막 실행의 PASS는 유효하지만 앞선 동일 최종 소스의 FAIL을 해소하는 근거로 충분하지 않다.

모든 완성 실행에서 allocation operations=370,928, retained delta=4,196,928 bytes와 결과 hash는 같았다.
[공통 관찰](r05_common_observations.json)에 비교 결과를 보존했다.
동시 8개 lane에서 allocation/deallocation이 겹치는 동안 전역 live peak를 관찰하는 도구 구조상,
이 두 수치와 결과 parity가 같아도 peak는 달라질 수 있다. 스케줄 차이의 영향은 가능한 설명이며 단독 원인으로 입증되지는 않았다.
선행 D04 재실행도 초과하므로 **이번 기능 교정이 새 비용 회귀를 만들었다고 단정하지 않는다.**
반대로 그것만으로 현재 상한 충족을 인정할 수도 없다.

구현자가 모든 실패·대조·PASS를 보존한 것은 적절하다. 기록 누락이나 결과 은폐를 지적하는 finding이 아니다.
이번 검토는 이미 결속된 실패 증거를 분석했으며, PASS가 나올 때까지 R05를 추가 반복하지 않았다.

해소 방향:

1. 동일 tool/workload/observer/affinity 조건에서 peak의 동시 live allocation 구성과 변동 원인을 조사한다.
   비교·반복 조건과 결과 채택 기준을 먼저 기록하고 모든 유효 실행을 보존한다.
2. 실제 runtime 비용이라면 원 cap 안의 여유를 확보하는 구조 교정을 수행한다. 평균화·직렬화·부하 축소·cap 확대를 기존 조건 충족으로 대체하지 않는다.
3. 측정 도구의 결함이나 비교 불가능 조건을 발견하면 구체적 재현 근거로 분리한다. 도구를 교정할 경우 원 기록과 workload 의미를 보존하고 선행/후속을 같은 조건에서 다시 비교한다.
4. 이 비용 문제의 근거가 닫힌 뒤 단계 전체 PASS를 재판정한다. 이미 해결된 D04-F01을 다시 미해결 기능 결함으로 되돌리지 않는다.

## 4. 재사용 검증과 판정 범위

[reused_evidence.json](reused_evidence.json)의 최종 source 결속을 확인해 다음 결과를 재사용했다.

- 전체 check, 신규9·value-expression74·정상 target-locked1·area1, 전체 제품 **1,509/1,509**, 지정 allow의 Clippy와 변경 영역 format.
- PC 격리 copied/built/run identity 및 기존 36조건 성공31/typed refusal5, gameplay oracle·상태·오류 code 유지.
- R05의 성공과 실패 전부. latency gate는 미적용이며 allocation bytes는 RSS가 아니다.

이번 검토가 전체 1,509건·PC·R05를 직접 재실행한 것은 아니다. 직접 실행은 위 9건과 이전 probe 4건이다.
각 direct receipt는 실제 격리 source와 workspace의 전후 hash를 기록한다. probe 사본의 유일한 제품 소스 변경은
`core/mod.rs` test module 안의 module hook이며 [test_only_hook.patch](test_only_hook.patch)에 남겼다.

다음 작업은 **R05 비용 변동의 원인·상한 충족 근거를 닫는 것**이다.
현재 판정은 D04-F01 기능 PASS / 비용 증거 미해결이며, D04 구간 전체 수용이나 WP02/R06 완료가 아니다.
잔여 ByOwner·scalar producer·D07·WP03/04와 production publication 차단은 유지한다.
