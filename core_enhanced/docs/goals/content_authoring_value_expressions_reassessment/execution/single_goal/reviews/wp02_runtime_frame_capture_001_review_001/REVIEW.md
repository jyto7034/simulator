# R06 WP02 runtime frame/capture 001 단계검토

검토일: 2026-09-26 KST  
판정: **CORRECTION_REQUIRED — 확정 finding 3건, 다음 구현 구간 진입 보류**

대상: `R06_WP02_RUNTIME_FRAME_CAPTURE_TARGET_001`  
Manifest SHA-256: `d3dbf3ba7c3c3be5a584db3092e7059c116d92b1ae812740f1fb68ccb05c2fc3`  
검토 입구: [WP02 REVIEW_HANDOFF](../../segments/wp02_runtime_frame_capture_001/REVIEW_HANDOFF.md)

## 검토 범위와 독립성

GEX §6.4의 부분 단계검토다. WP02 전체 또는 GEX-20/30 완료 감사가 아니다. 동결 delta 10파일, frame/capture/evaluator와 action·resource의 관련 생산자/소비자, content identity 전달, 기존 검증과 보존 원본을 직접 확인했다. 순차 effects/result-prefix/publication 등 명시된 후속 미구현은 이번 결함으로 세지 않는다.

이 reviewer는 기존 R06 논의·인수인계 작성에 참여했고, 동결 후 Unix 테스트 모듈 조건 수정에도 참여했다. 완전한 blind review나 그 테스트 수정에 대한 독립 감사라고 주장하지 않는다. WP02 frame/capture 제품 구현에는 참여하지 않았으며, 이번에는 제품 코드를 교정하지 않았다. 재현용 테스트는 동결 archive의 `/tmp` 사본에만 추가했다.

## WP02-F01 — 호스트 잔여 예산을 평가 전에 적용하지 않음

우선순위: **높음 — 다음 runtime consumer 연결 전 교정 필요**.

위치: [value_expr_runtime.rs:462](/mnt/f/work/simulator/core_enhanced/src/game/battle/core/value_expr_runtime.rs:462), [같은 파일:474](/mnt/f/work/simulator/core_enhanced/src/game/battle/core/value_expr_runtime.rs:474), [frame 생성:503](/mnt/f/work/simulator/core_enhanced/src/game/battle/core/value_expr_runtime.rs:503).

Live frame을 먼저 만들고, `ExprMeter::new(plan.evaluation_limits())`로 컴파일 당시 한도를 그대로 사용한 뒤 `evaluate` 전체가 반환하고 나서 호스트 work를 청구한다. 이 meter에는 현재 native/authoring operation의 더 작은 잔여 예산이 전달되지 않는다. 평가 중 frame/stack/trace의 할당도 native retained admission과 연결되지 않는다. capture 생성 시의 admission은 이후 consumption의 동적 평가 admission을 대신하지 못한다.

직접 재현:

- 실제 catalog의 128개 Source Attack read를 가진 Add 식에서 정상 실행의 요청 allocation은 **53,936 bytes**였다. native `max_work=1`에서도 **54,278 bytes**를 요청한 뒤에야 `value_expr_evaluation`, requested work **130**으로 실패했다. 거부 결과는 맞지만 계산·할당 이전의 제한으로 작동하지 않는다. 이 수치는 요청 allocation 합계이며 RSS/peak memory가 아니다.
- 기존 core와 capture extent를 seed하고 native retained 잔여량을 **1 byte**로 둬도 Source Attack 평가가 성공했다. 반환된 evaluator 사용량 자체가 `retained_bytes: 768`이었다.
- 두 재현은 current catalog의 `capture_value_expr_entry` 경로를 사용했다. 제품 전체의 외부 실행 재현은 아니며 현재 publication gate는 닫혀 있다.

근거: [최종 경계 시험](probe_production_death.log)의 `review_exhausted_work_must_stop_before_full_evaluator_allocation`, `review_native_remaining_retained_limit_must_reject_evaluation`.

교정 방향:

- plan 한도와 현재 operation의 잔여 work/retained/취소 계약을 연결하는 공통 admission 경계를 둔다. frame 조회·동적 복제·stack/trace 할당보다 앞서 적용한다.
- 정적 요구와 실제 동적 비용을 구분하고, 선청구/실제 청구의 누락·중복을 막는다. `reference_node` 조회 등 frame 준비 비용도 검토한다.
- 종료 후 일괄 청구만으로 제한을 입증하지 않는다. 취소와 work는 평가 진행 중에도 필요한 경계에서 확인하도록 연결한다.
- 새 큰 한도, 오류 후 0 반환, 별도 계산기를 해결책으로 쓰지 않는다.

재검증: native와 authoring 각각의 limit−1/limit/limit+1, 잔여 예산이 이미 소진된 상태, Snapshot/Live consumption, 큰 식의 조기 거부·취소 및 실패 후 정상 retry. 값/trace/rounding 의미는 보존한다.

## WP02-F02 — 유효 action과 실제 actor/source-copy의 결속 검사 누락

우선순위: **중간 — 현재 adapter의 identity 보장 미충족**.

위치: [value_expr_runtime.rs:299](/mnt/f/work/simulator/core_enhanced/src/game/battle/core/value_expr_runtime.rs:299), [runtime_facts.rs:483](/mnt/f/work/simulator/core_enhanced/src/game/battle/core/runtime_facts.rs:483).

capture는 authored source와 plan의 일치, action token의 유효성, delivery의 action 소속을 확인한다. 그러나 `source.actor`는 존재 여부만 검사하고, action에 보존된 actor/provider와 전달받은 source actor/equipment 사이의 관계는 확인하지 않는다. 서로 개별적으로 유효한 identity를 조합해도 통과한다.

직접 재현:

- Attack 10인 actor A의 유효 action을 유지하면서 `source.actor`만 Attack 3인 actor B로 바꾸면 capture/evaluation이 성공하고 **Damage(3)**을 반환했다. 동일 current catalog plan을 사용했다.
- action 수락 시 보존한 equipment copy 93을 capture 시 copy 94로 바꾼 source도 받아들였다. 이 추가 사례는 copy 불일치의 admission을 확인한 것이며 실제 타 사본 resource 지출까지 재현한 것은 아니다.

근거: [최종 경계 시험](probe_production_death.log)의 actor/equipment mismatch 2건.

교정 방향:

- action/delivery가 보유한 실제 실행 provenance를 기준으로 actor 및 해당하는 source-copy/provider의 결속을 검사하거나, 검증된 불변 실행 context만 capture가 받도록 한다.
- effect별 authored metadata는 달라질 수 있으므로 source 구조 전체를 무조건 동일 비교하는 방식은 피한다. 합법적인 child/proc/delivery 관계는 기존 R04 계약에 맞춰 구분한다.
- Snapshot의 source 퇴장 후 사용을 금지하는 방식으로 해결하지 않는다. capture 당시의 결속과 consumption 당시 Live 가용성은 별개의 검사다.

재검증: 정상 동일 source, 동일 definition의 서로 다른 copy, 다른 actor와 유효 action 조합, 같은 attempt의 다른 action, 정상 child/delivery, Snapshot 이후 source 퇴장, Live의 owner 부재를 구분한다.

## WP02-F03 — 비활성 unit의 Live resource read가 성공함

우선순위: **중간 — context refusal 의미가 selector별로 불일치**.

위치: [resource_runtime.rs:68](/mnt/f/work/simulator/core_enhanced/src/game/battle/core/resource_runtime.rs:68), [value_expr_runtime.rs:566](/mnt/f/work/simulator/core_enhanced/src/game/battle/core/value_expr_runtime.rs:566), 비교 [같은 파일:654](/mnt/f/work/simulator/core_enhanced/src/game/battle/core/value_expr_runtime.rs:654).

Stat/HP read는 `value_expr_unit`에서 active lifecycle을 확인한다. 반면 Resource read가 쓰는 공통 resolver는 `units.get`과 store lease의 가용성만 확인한다. 사망·퇴장으로 deployment가 끝나도 Attempt/Run resource와 unit 기록은 남을 수 있으므로, 유효 lease만으로 Live unit context의 가용성을 증명할 수 없다.

직접 재현:

- TargetEmployee의 Attempt resource `meter=7`을 정상 상태에서 읽는 대조군은 성공한다.
- 기존 production `finalize_unit_death`를 호출하고 실제 target lifecycle이 Dead임을 확인한 뒤에도 동일 Live capture가 **Damage(7)**을 반환했다.
- 이 resource 시험은 기존 구현자 fixture와 같은 방식으로 resource setup 후 계획을 컴파일해 private capture seam에 주입했다. 외부 pack 전체 경로 성공으로 표시하지 않는다. 사망 상태는 수동 플래그만 바꾼 것이 아니라 실제 death producer로 만들었다.

근거: [최종 경계 시험](probe_production_death.log)의 `review_live_resource_of_inactive_target_must_be_rejected`.

교정 방향:

- ValueExpr frame이 요구하는 source/target의 해당 시점 가용성을 resource read에도 검사한다. store owner/lease의 유효성과 battle unit의 가용성을 구분한다.
- 공유 mutation resolver를 무조건 active-only로 바꿔 R05 finite delivery·사망 후 유지되는 Run/Attempt resource 규칙을 깨지 않는다. expression 읽기 경계와 기존 저장·지출 정책의 책임을 분리한다.
- 이미 적법하게 캡처된 Snapshot resource 값은 유지한다. 문제를 가리기 위해 사망 시 영속 자원을 삭제하거나 0으로 대체하지 않는다.

재검증: SourceEmployee/TargetEmployee/SourceEquipment, active/dead/withdrawn/removed, Attempt/Run/Deployment, Snapshot/Live, 기존 finite delivery와 read 전후 revision/reservation 불변.

## 현재성·복원 및 동결 이후 delta

- 원본 manifest와 detached hash, baseline/predecessor 참조, source/record/evidence archive의 구성·각 payload, 17 validation receipts의 결속을 검사했다.
- before archive에 기존 binary-capable patch를 적용해 after 314개 파일의 hash와 파일 집합을 복원 검증했다.
- 현재 working source는 312/314 일치. `world/tests/combat.rs`의 Unix guard와 `world/tests/resource_lifetimes.rs` 포맷만 다르다. execution records 6/6, evidence 37/37은 일치한다.
- [원본 verifier](original_currentness.log)는 예상대로 `combat.rs` mismatch로 실패했다. 이를 PASS로 바꾸지 않았다. 별도 [검증 코드](verify_review_target.py)는 이 두 current hash를 명시적으로 검사하면서 나머지 원본 검사와 archive 검증을 수행한다. 결과 명칭은 `FROZEN_ARTIFACTS_AND_EXPLICIT_CURRENT_DELTAS_VERIFIED`이며 unchanged currentness가 아니다.
- 진입 [target_entry](target_entry.log), 중간 종료 [target_exit](target_exit.log), 마지막 [target_final_exit](target_final_exit.log)에 해당 비교를 보존했다. 두 테스트 delta는 이번 frozen production finding의 원인이 아니다. Unix guard는 플랫폼별 테스트 포함 범위를 바꾸므로 Windows 검증 완료라고 해석하지 않는다.

## 직접 검증과 재사용의 구분

| 직접 실행 | 결과 | 실행 대상 |
| --- | --- | --- |
| 기존 frame/capture adapter | [7/7 PASS](existing_adapter.log) | frozen archive + reviewer test include 사본; 새 review tests 제외 |
| ValueExpr | [36/36 PASS](value_expr_regression.log) | 같은 사본 |
| resource 관련 회귀 | [37/37 PASS](resource_regression.log) | 같은 사본; `game::resource` selector는 `game::resources`도 포함 |
| runtime facts | [11/11 PASS](facts_regression.log) | 같은 사본 |
| reviewer 경계 시험 | [1 PASS / 5 FAIL](probe_production_death.log) | 정상 대조군 1, finding 재현 5; 3개 finding으로 분류 |
| current all-target check | [exit 0](current_all_target_check.log) | 실제 작업 폴더, 기존 dead-code warning 보존 |
| current resource lifetimes | [17/17 PASS](current_resource_lifetimes_rebuilt.log) | `CARGO_INCREMENTAL=0`, 실제 root 소스 재컴파일·별도 바이너리 확인 |
| current adapter / 기본 빌드 캐시 복원 | [7/7 PASS](current_adapter_default_cache_rebuilt.log) | 실제 root 소스 기본 설정 재컴파일; 앞의 adapter 7건과 중복 coverage |

이번에 full suite, Clippy, rustfmt, CLI/save/authoring 전체를 새로 실행한 것은 아니다. 기존 target의 해당 receipt/log는 hash와 source binding을 확인해 과거 자체 검증 증거로만 참조했다. 현재 미연결 publication/순차 effects/rollback 전체의 PASS로 확대하지 않는다.

### 시험 구성·실패 기록

- [최종 probe source](probes.rs), [실행·receipt 생성기](run.mjs), [격리 사본 identity](harness_identity_retry.log)를 보존했다. 동결 archive의 제품 source를 그대로 사용하고 `value_expr_runtime.rs`의 private tests에 `include!` 한 줄만 추가했음을 314개 파일 hash로 확인했다. 제품 로직의 assertion을 완화하지 않았다.
- archive에 없는 workspace member manifest `game_server/Cargo.toml`은 현재 파일을 보충했다. 그 SHA-256 `19b41cd19c6e60818698b8ecaa9d9cc0c44ed071bff2cfbb7296d92dbeee722f`는 [동결 HEAD의 파일](workspace_support_history.log)과 일치한다. 다음 target에는 이 build 입력도 명시적으로 연결하는 것이 좋다. 과거 archive를 수정하지 않았다.
- 최초 resource probe는 fixture builder가 resources 설치 전에 expression을 compile하면서 실패했다. [최초 로그](probe_initial.log)/[최초 source](probes_initial.rs)를 보존했고 이 실패는 finding 증거로 세지 않았다. 기존 fixture와 같은 setup 순서로 수정한 [중간 source](probes_before_production_death.rs)/[중간 결과](probe_final.log), 실제 death producer를 사용한 최종 결과를 구분한다.
- allocator는 격리 테스트 바이너리에만 설치했고 현재 thread의 요청 allocation 합계를 측정한다. 값 계산을 바꾸거나 RSS/peak allocation을 측정한 것으로 표시하지 않는다.
- harness identity의 첫 실행은 Node `spawnSync rustc`의 sandbox EPERM으로 실패했다. [실패](harness_identity.log)와 승인된 동일 검증 [재실행](harness_identity_retry.log)을 함께 보존했다.
- root와 사본이 같은 build directory를 사용해 최초 `current_resource_lifetimes` invocation이 사본의 reviewer 테스트를 포함한 바이너리를 재사용한 정황(전체 1,413개)을 확인했다. [그 실행](current_resource_lifetimes.log)은 17개 성공이어도 current 소스 재빌드 근거에서 제외한다. `CARGO_INCREMENTAL=0`으로 실제 root를 재컴파일한 뒤 별도 바이너리에서 **17 PASS / 1,390 filtered = 원래 1,407개**임을 확인했다. 재현 probe는 사본에서 실제 재컴파일됐으며 이 출처 문제와 구분한다.
- 검토가 만든 기본 설정 테스트 바이너리는 `/tmp/r06-wp02-review.E34TyB/frozen_review_test_binary`로 이동·보존하고, root의 기본 설정 바이너리도 재컴파일했다. 마지막 adapter 실행에서 **7 PASS / 1,400 filtered = 원래 1,407개**를 확인했다. reviewer test가 추가된 바이너리를 작업 폴더의 기본 실행물로 남기지 않았다. 후속 probe는 별도 target directory를 권장한다.

## 반환 범위

다음 작업은 **WP02-F01/F02/F03 교정 설계 확인 → 명시적 교정 요청 → 새 baseline에서 교정·자체 검증 → correction target 동결 → 재검토**다. 이 보고서는 수정 실행이나 다음 구현 승인 자체가 아니다.

순차 effects/result-prefix/WP03를 함께 구현해 findings를 덮지 않는다. 기존 publication 차단·커널 계산 의미·R04/R05 lifetime·literal/V7 계약을 유지한다. 이 부분 검토가 통과한 후에만 사용자의 다음 구현 요청으로 이어간다. WP/Phase/Goal 완료는 선언하지 않는다.
