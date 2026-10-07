# R06 WP02 effect execution / prefix segment 001 재검토

## 판정: CORRECTION_REQUIRED

대상: `segments/wp02_effect_execution_prefix_001/target/effect_execution_prefix_001/MANIFEST.json`

SHA-256: `f8d854b8ea257a2ea19045caf61434d0592ee2333cfe827edfb5eaac7530b52f`

새 수정 요구는 **3건, 모두 P2**다. 실제 코드와 변경 diff를 검토하고 격리 소스에서 재현했다. 제품 코드·기존 target·baseline은 수정하지 않았다. cross-step witness/ResolvedTargets 등의 전체 구현을 이번 checkpoint에 강제하지 않지만, 미지원 의미를 불완전한 정상값으로 반환해서는 안 된다.

## EEP-F01 [P2] 새 capture 임시 버퍼가 선청구보다 먼저 할당됨

주 위치: `core_enhanced/src/game/battle/core/value_expr_runtime.rs:910`.

`build_value_expr_delivery_capture`의 structural admission은 rows와 `Arc<EvalCapture>` slots 등을 계산하지만, 각 expression에서 `release_targets`를 `Vec<Option<UnitInstanceId>>`로 복사하는 임시 버퍼는 포함하지 않는다. 이 복사는 첫 `capture_value_expr_entry`의 admission보다 먼저 실행된다.

독립 probe는 512개의 서로 다른 active unit을 fixture에 넣고, 현재 owner extent 58,702 bytes를 seed한 뒤 남은 allowance를 6,000 bytes로 설정했다. 전체 native cap은 64,702 bytes다. 최종 거부 전 요청 allocation 합계는 13,296 bytes, 단일 최대 요청은 **8,704 bytes**였다. 이 요청 하나가 남은 allowance 전체보다 크다. 이후 `value_expr_capture`에서 requested 65,459 / limit 64,702로 거부됐지만 이미 수행한 임시 allocation을 막지는 못했다.

이는 “6KB가 현대 PC에 적합한 상한인가”의 문제가 아니다. 작은 한도로 검사해 **작업 전 청구 계약의 누락**을 드러낸 것이다. 전체 PC workload가 이 크기에서 실패한다거나 실제 RSS가 이 수치라고 주장하지 않는다.

수정 방향:

- 가능한 경우 release target slice를 borrow/iterator로 순회해 임시 복사를 없앤다. 복사가 필요하면 실제 예약량을 checked 계산해 allocation 전에 청구한다.
- 동일 책임의 신규 버퍼도 함께 비용 소유권을 점검한다. 특히 resource ID 수집은 push/clone 후 `resource_extent`를 청구하고 있고, `sim.rs:2884`의 evaluated amount Vec도 직접 선청구가 없다. carrier의 effect-id Vec slots, projection/cumulative vector까지 포함한 표로 누락·중복을 구분한다. 이 주변 항목은 코드에서 확인한 점검 범위이며 모두에 별도 allocator 재현을 했다는 뜻은 아니다.
- 기존 한도를 늘리거나 임시 버퍼라는 이유로 제외하지 않는다. byte/work/cancellation admission을 실제 작업 앞으로 둔다.
- 남은 allowance의 경계/1 byte 부족, 많은 대상·resource ID, 거부 전 allocation을 회귀로 남긴다.

증거: `probes_expanded.log`의 `review_prefix_capture_scratch_must_fit_admitted_bytes`.

## EEP-F02 [P2] resolved target 수가 효과 수만큼 중복 집계됨

위치: `core_enhanced/src/game/battle/core/sim.rs:2580`, `sim.rs:2962`.

각 effect command builder가 `resolved_target_count = targets.len()`으로 시작하고, 바깥 executor가 매 effect 결과를 `aggregate.merge`한다. 따라서 같은 delivery의 같은 target 목록을 effect 수만큼 다시 센다.

literal Damage 1 + Heal 1, 대상 1명으로 비교했다. 보존된 기존 command builder는 **1**, 새 순차 executor는 **2**를 반환했다. expression이 없는 기존 literal 스킬에서도 생기는 회귀다. 이 결과는 cast progress 및 `SkillStepDebugResult`로 전달된다. 현재 이것이 전투 피해량 자체를 배수로 만든다는 주장과는 구분한다.

수정 방향:

- 한 delivery consumption에서 resolved target 수는 한 번만 기록한다.
- effect별 실제 피해/사망/적용/예약 결과는 각각 정해진 의미로 누적한다.
- `SkillStepResult::merge`를 무조건 max나 overwrite로 바꾸지 않는다. 반복 실행·서로 다른 delivery 결과의 합산은 별도 책임이다.
- literal 다중 효과, expression 혼합, modifier/signal 효과, 반복 delivery의 집계 의미를 회귀로 확인한다.

증거: `probes_expanded.log`의 `review_prefix_literal_resolved_target_count_is_not_effect_count`.

## EEP-F03 [P2] delivery-local 합계를 CurrentActionPrefix로 정상 반환함

위치: `core_enhanced/src/game/battle/core/value_expr_runtime.rs:321`, `value_expr_runtime.rs:451`, `core_enhanced/src/game/battle/core/sim.rs:2863` 부근.

executor는 consumption마다 새 `EffectResultPrefix`를 만들고 cumulative를 0으로 시작한다. 그런데 `CurrentActionPrefix`는 이 local cumulative를 그대로 반환한다. 동일 action의 앞 step/delivery 결과가 누락됐는지 검사하거나 미지원으로 거부하지 않는다.

두 step의 실제 RON을 catalog에 넣고 **동일 ActionHandle**로 실행했다.

1. step 0: 실제 피해 7.
2. step 1: 실제 피해 3 → `CurrentActionPrefix(DamageHpLost)`만큼 회복.
3. action facts의 누적 피해는 10인데 회복은 **3**, 남은 HP는 73이었다. typed refusal도 없었다.

현재 action-level witness를 다음 구간으로 남기는 것은 허용된다. 하지만 그 상태에서 local 3을 action prefix 값으로 표시하는 것은 다른 의미다. 완전한 값 10을 제공하지 못하면 지원되지 않는 조합을 명시적으로 거부해야 한다.

또한 runtime `value()`는 `CurrentActionPrefix`에서 settled frontier를 확인하지 않는다. 별도 seam probe에서 모든 row가 미완료인 witness를 전달해도 정상 0을 읽었다. 이 probe는 private 상태를 의도적으로 구성한 방어 경계 시험이다. 일반 executor가 항상 그 상태에 도달한다는 뜻이 아니며, catalog는 첫 effect의 CurrentActionPrefix를 이미 거부한다. 따라서 이 보조 결과를 외부 입력으로 즉시 가능한 별도 취약점으로 세지 않는다.

수정 방향:

- 이번 scope를 유지한다면 action-level 완전성을 증명할 수 없는 CurrentActionPrefix 사용을 typed unsupported/unavailable로 차단한다. 전부 지원하는 척하거나 delivery-local prefix로 조용히 의미를 축소하지 않는다.
- 지원 가능한 단일 consumption을 유지하려면 어떤 owner가 action frontier의 완전성을 보증하는지 명확히 한다. 보증이 없다면 해당 selector 지원 자체를 다음 구간으로 남겨도 된다.
- runtime read는 현재 effect 앞의 정확한 완료 frontier와 witness 존재를 확인한다. 미완료/미래/자기/다른 occurrence를 정상 0이나 누적값으로 처리하지 않는다.
- cross-step/cross-delivery 전체 구현을 이번 교정의 필수 범위로 확대하지 않는다. 이를 구현할 경우에는 별도 구현 범위와 검토 타깃으로 다룬다.

증거: `review_prefix_cross_step_is_complete_or_explicitly_unsupported`, `review_prefix_runtime_rejects_missing_current_frontier`.

## 확인된 장점과 유지할 부분

- 같은 effect의 모든 target amount를 적용 전에 평가하는 구조, 다음 effect에서 변경된 Live 상태를 읽는 구조는 분리돼 있다.
- 실제 mitigation 이후 피해 7/4를 회복 7/4로 사용하는 제품 시험은 독립 실행에서도 통과했다.
- immediate/projectile/area의 주요 Result 전파와 context 복원 경로를 검토했다. 관련 제품 오류·rollback 및 Snapshot 수명 시험이 통과했다.
- `ResolvedTargets`는 명시적 UnsupportedMoment로 남고, publication 차단도 유지된다.
- 따라서 전체 구조를 폐기할 필요는 없다. 선청구 소유권, delivery 집계, action prefix 지원 경계를 교정한다.

## 직접 검증 및 증거

격리 소스 `/tmp/r06-prefix-review.WcaQXG`: frozen archive에서 추출했다. private test-module hook 한 곳과 추가 probe 파일 외 **333개 source bytes 일치**를 검증했다. allocator는 검토 binary에서만 사용했다.

| 직접 검증 | 결과 |
| --- | --- |
| manifest hash/current source | 333개 일치 |
| 동결 evidence | 54개 hash 일치 |
| before → forward delta → after 복원 | 333개 일치 |
| after → reverse delta → before 복원 | 333개 일치 |
| 기존 battle core 제품 시험 | 394/394 |
| ValueExpr kernel/catalog/parser | 37/37 |
| resource 모듈 | 27/27 |
| 기존 제품 시험 합계 | **458/458** |
| 최종 검토자 추가 probe | **1 통과 / 4 실패**, 위 3 findings에 대응 |

`probes_expanded.log`가 최종 probe 결과다. 최초 `probes.log`는 더 작은 초기 seam fixture의 결과로 보존했다. 최종 allocation probe는 유효 active unit 512개와 실제 owner extent seed를 추가해 다시 검증했다. 이 두 실행의 시험 수를 합쳐 서로 다른 coverage인 것처럼 계산하지 않는다. 통과한 추가 probe는 기존 actual-damage test를 호출한 control이며 별도 새로운 기능 coverage로 과장하지 않는다.

전체 all-target 1,484개, check, Clippy, changed-region rustfmt는 구현자 frozen 증거를 재사용했다. 이번 검토에서 이 전체 명령을 다시 실행한 것은 아니다. PC/R05도 재실측하지 않았고, 동결 comparison/receipt에서 36조건 중 31완료/5거부와 concurrent peak 8,920,677 / cap 9,030,188을 확인했다. 이 workload의 성공이 이번 신규 경계 시험을 대신하지는 않는다.

실행기 `run.mjs`, 최종 `probes.rs`, `isolated_identity.json`, 독립 복원기 `verify.py`, `verification.json`, 직접 실행 로그/receipt를 이 디렉터리에 남겼다.

## 다음 조치

EEP-F01~F03을 교정하고 현재 actual bytes를 별도 before baseline으로 보존한 뒤 새 correction target으로 재검토를 요청한다. 이전 타깃·실패 증거를 덮어쓰지 않는다. 예산 상향, publication 해제, 다음 WP 진입, WP02 완료는 이 검토로 승인하지 않는다.
