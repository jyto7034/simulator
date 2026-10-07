# EEP-F01~F03 교정 판단 및 비용 소유권

## 기준선

- 원 검토: `reviews/wp02_effect_execution_prefix_001_review_001/REVIEW.md`, CORRECTION_REQUIRED.
- Before target SHA-256: `f8d854b8ea257a2ea19045caf61434d0592ee2333cfe827edfb5eaac7530b52f`.
- 새 actual-byte baseline: `baseline/entry_001/MANIFEST.json`, SHA-256 `3f981d8cdc662e264248899682631cb719e515726308d38b4777e8f127514706`.
- 제품 수정 전에 predecessor hash와 333개 source 일치를 확인하고 `SOURCE_BEFORE.tar.gz`를 생성했다. dirty worktree의 기존 변경은 보존했다. Git HEAD를 before로 대체하지 않았다.

## F01: 작업보다 먼저 청구

정책: 기존 native/authoring 한도와 kernel 수식·rounding·fingerprint는 변경하지 않는다. 아래 byte 값은 누적 admission의 보수적 상계이며 RSS/현재 살아 있는 heap과 동일하지 않다. 임시 버퍼도 무료로 취급하거나 해제 시 환급하지 않는다.

| 할당 | 비용 소유자 / 청구 위치 | 처리 |
| --- | --- | --- |
| release target 임시 `Vec<Option<UnitInstanceId>>` | 없음 | 삭제. 기존 slice의 index를 borrow하여 empty일 때만 None 1회 |
| carrier rows, capture Arc slots | `value_expr_delivery_carrier` | 기존 checked structural admission 후 예약 |
| effect ID Vec slots 및 문자열, action site 문자열 | carrier structural admission | 누락된 String slots·action site bytes 추가. clone은 청구 뒤 |
| carrier/ID Arc 헤더 | carrier `size_of` + 기존 256-byte envelope | 기존 여유분으로 포함; 따로 중복 청구하지 않음 |
| resource ID Vec slots·문자열 | `value_expr_result_projection` | borrowed prepass로 모든 result-resource reference 상계 계산 후 `with_capacity`/clone. 중복 문자열까지 보수적으로 청구하고 capacity는 dedup 후 유지 |
| resource ID 정렬 | resource sort work | 문자 bytes+slots 기반 checked work를 먼저 청구, `sort_unstable`으로 별도 sort heap 제거. scan/clone 반복에 work·취소 polling |
| prefix settled slots | `value_expr_prefix_witness` | effect 수 × Option projection 크기, 할당 전 |
| prefix cumulative resource Vec·action site clone | 동일 prefix admission | resource 수 × i128 + site 문자열. 아직 local 내부 bookkeeping이며 action-wide 공개 값으로 사용하지 않음 |
| effect별 projection resource Vec | `value_expr_effect_projection` | 해당 effect 시작 때 한 번. carrier/prefix에서 미래 effect 배열을 중복 예약하던 항목 삭제 |
| literal projection의 빈 resource Arc | 동일 effect projection envelope | 기존에는 호출자가 먼저 만들었으나, 이제 Option을 넘기고 admission 후 생성 |
| target별 evaluated amount Vec | `value_expr_evaluated_amounts` | target 수 × RuntimeAmount 크기를 checked 계산해 `with_capacity` 전 청구 |
| EvalCapture 및 Live/Snapshot frame | 기존 capture/evaluator owner | 기존 경계를 유지. carrier가 같은 captures를 별도 버퍼처럼 중복 청구하지 않음 |

새 iterator에는 대상 수를 줄이거나 중복 대상을 제거하는 의미 변경이 없다. resource ID는 deterministic sort/dedup 의미를 유지한다. effect frontier 탐색/settle에도 작업 청구를 추가했다. resource ID의 원소/문자열 상계는 clone 전에 계산하며 overflow는 오류다.

## F02: delivery 집계와 effect 집계의 분리

`build_ordered_skill_effect_commands`는 per-effect 결과에 대상 수를 넣지 않는다. `execute_ordered_skill_effects`의 aggregate가 targets.len()을 한 번 기록한다. `SkillStepResult::merge`는 수정하지 않아 별도 소비/반복 실행 결과의 합산이 유지된다. 실제 damage 대상·kill·applied effect 결과도 기존대로 합산한다.

제품 회귀: literal 두 효과 × 두 대상 × 두 번 소비 → 각 소비 2, 합계 4. empty target → 0. Damage + EarlierEffect HealExpr → 대상 1, 실제 HP 원상복귀. 기존 전체 시험은 modifier/signal/literal 경로도 포함하나, 이번 새 probe가 가능한 모든 조합을 검증했다고 주장하지 않는다.

## F03: 미지원 범위를 불완전한 정상값으로 숨기지 않기

현재 witness의 소유자는 delivery consumption이지 action 전체가 아니다. 따라서 local cumulative 3을 앞 step의 7까지 포함하는 action prefix로 반환하지 않는다. `CurrentActionPrefix`에 앞 완료 frontier가 없으면 NotSettled, 있더라도 action-wide witness가 없으므로 Unavailable이다. 단일 step 예외를 추론하지 않는다.

`EarlierEffect`는 source/action/delivery/site 결속, 미래·자기 참조 구분을 유지하고, 모든 앞 effect 완료 및 현재/미래 effect 미완료라는 exact frontier를 추가 확인한다. 유효한 완료 결과 0은 정상값이고 witness 부재는 0이 아니다.

제품 회귀는 step 0의 실제 damage 7을 먼저 확정한 후, step 1의 damage 3 + CurrentActionPrefix heal을 candidate transaction으로 실행한다. Unavailable 시 현재 candidate 전체를 rollback하고 이전 확정 HP 73 / 누적 damage 7 / heal 0을 보존한다. 완전한 action-wide prefix 자체는 다음 별도 구간의 구현 대상이다.

## 종료 조건

교정만 구현했고 독립 재검토를 기다린다. publication 차단, 기존 예산, WPs의 미완료 상태를 유지한다. 최종 manifest는 source 333개, 변경 제품 2개, 실제 bytes archive와 양방향 delta, handoff/실행 증거의 hash를 묶는다.
