# R06 WP02 D04 유닛 관계·필터·장착 슬롯 — 단계검토 인계

상태: **부분 구현·자체 검증 종료 / 단계검토 타깃 동결 / 독립 검토 요청 대기**.

시작점은 D04-F01 successor manifest SHA-256 `e4f0d77b1bd3066493d605271605d7a4b6e16317a6d540d303909b076b480104`, 제품 source digest `34ceedbebc84b9396d42f92fa31033fe0b831a90e5325c8c4463e0c88f067145`다. 구현 직전 333개 source의 currentness와 격리 양방향 복원을 재검증했다. R05 실행 중 발견한 source consumer와 lock 네 파일은 편집 전에 별도 보존해 동결 집합을 337개로 확장했다.

최종 제품 333개 digest는 `8bde59151f95ad7301c7451f91af7b69c0a5e61c2a0a46239c1da8dd13f2b7b2`, 확장 337개 digest는 `eb9bd8fb3927f8a9f3c5b35908012300e4fad9c4e8d7a03a9e612e44b0dc804a`다. 단계검토 target은 `target/d04_owner_relation_001/MANIFEST.json`이다.

## 구현 결과

### 실제 관계와 명시적 제외

- `FiniteSource::ResolvedTargetsWhere`에 bounded conjunction 조건 `IsActualEmployee`, `HasEquippedSlot(address)`를 추가했다. 조건은 admission 후 고정 순서 정렬·중복 제거하며, 빈 조건은 기존 `ResolvedTargets`로 정규화한다.
- `Grouping::ByResolvedOwner`는 `Side`, `ActualEmployee`, `EquippedSlot(address)`를 명시한다. 실제 key는 각각 현재 `unit.owner`, `BattleUnitSource::Employee`의 employee UUID, 지정 슬롯 binding의 실제 equipment copy UUID다.
- 실제 직원이 아닌 source의 nonnil owned UUID를 직원으로 보지 않는다. 정상 빈 슬롯만 조건에서 false가 되며, unknown layout, 잘못된 주소/layout, copy 부재·nil·중복·stale runtime identity는 typed 오류다. 자동 skip, 0 반환, 첫 장비 선택을 넣지 않았다.
- original resolved target을 UnitId로 정렬·중복 제거하고 존재와 active identity를 먼저 확인한다. 조건을 통과한 member에 대해서만 해당 aggregate의 직접 Attack/HP ref와 관계를 읽는다. 조건은 원 타격 목록이나 완료 대상을 바꾸지 않는다.

### 장착 생산 경로와 node별 view

- `EquipmentSlotAddress`와 `ItemSlot::iter_with_address()`를 추가했다. ByType은 Weapon/Armor/Accessory 고정 주소를, Any3은 관측 배열의 0–2 index를 보존하며 기존 장비 순서를 유지한다.
- 실제 inventory projection 두 경로가 `BattleUnitDraft.equipment_layout`과 각 `BattleEquipmentBinding.slot`을 전달한다. 장비가 없는 정상 layout은 `Some(empty)`로, 출처 불명 legacy fixture는 `None`으로 구분한다.
- 두 runtime spawn 경로가 공통 검증 helper로 `BattleEquipmentSlots`의 읽기 전용 Arc snapshot을 만든다. 기존 RuntimeItem 생성 순서와 actual copy identity를 유지하며 snapshot allocation, RuntimeUnit handle, scenario binding의 retained extent를 admission에 포함했다. clone shape와 scratch 재사용도 새 필드를 전달한다.
- 각 resolved aggregate는 `PreparedResolvedTargetView { node_index, members }`를 따로 가진다. evaluator는 선택한 하나의 slice로 raw count, canonical scan, grouping, reduction을 끝까지 수행한다. view 목록이 있으면 다른 node나 legacy 목록으로 fallback하지 않는다.
- Snapshot은 release의 조건 판정·key·값·member set을 고정하고, Live는 effect mutation 전에 한 번 새 view를 준비한다. 같은 effect의 모든 대상 amount가 성공한 뒤 mutation하는 기존 경계를 유지한다.

### parser·compiler·fingerprint·비용

- parser predicate visitor는 각 요소를 할당하기 전에 공유 node limit을 확인한다. 프로그램으로 만든 AST도 compiler에서 predicate node/depth/work/storage limit을 다시 적용한다.
- compiler capability, context requirements, source/group 조합, dynamic work와 retained 계산을 새 variant에 연결했다. 여러 resolved aggregate의 전체 view member 수와 준비 작업·scratch·final allocation을 checked arithmetic과 기존 native/authoring admission으로 제한한다. 모두 필터링돼도 raw scan과 predicate 비용은 발생한다.
- semantic fingerprint는 기존 source 태그 0/1과 grouping 태그 0/1/2를 보존하고 새 source=2, grouping=3 및 relation/predicate/slot payload를 명시적으로 encoding한다. 조건 순열·중복은 같은 digest, 다른 slot 의미는 다른 digest다. compiler version은 바꾸지 않았다.
- bare AggregateMember scalar, WP03 stat ref, 효과·상태 조회는 typed 미지원으로 남겼다. 전체 expression의 production publication 차단도 유지했다.

## O01–O15 검증 대응

| ID | 최종 source에서 확인한 관찰 결과 |
| --- | --- |
| O01 | `actual_units_group_by_side_employee_and_equipment_copy_without_view_contamination`: A 3/5, B 7의 Side 그룹 Max 합 12. 순서 변경·중복 UnitId에도 12 |
| O02 | 같은 시험과 `employee_filter_changes_only_aggregate_input_and_relation_fault_rolls_back`: 무필터 Employee 관계는 소환물에서 InvalidOwner, 직원 조건은 A/B만 포함, 부분 mutation 없음 |
| O03 | TestFixture에 nonnil owned UUID를 둔 부정 case가 Employee가 아님을 확인 |
| O04 | 같은 definition의 실제 무기 copy X/Z와 갑옷 Y를 별도 key로 확인하고 weapon/armor view의 실제 owner key를 assertion |
| O05 | `addressed_iteration_preserves_by_type_holes_and_item_order`, `addressed_iteration_uses_observed_any3_indices`, layout mismatch case로 빈 앞 슬롯·Any3·잘못된 주소를 구별 |
| O06 | `equipment_relation_distinguishes_empty_unknown_missing_copy_and_stale_identity`, `battle_spawn_rejects_invalid_slot_and_actual_copy_projections_before_insertion`로 unknown/empty/copy 없음/stale/nil/중복을 구별 |
| O07 | 한 plan의 Side+Employee+Weapon+Armor view가 각각 4/2/2/1 member이고 결과 37임을 확인. exact node view 부재는 fallback 없이 오류 |
| O08 | 직원 조건 뒤 빈 Count/Sum은 0, Max는 현행 `Unavailable`. 원 타격 목록은 유지 |
| O09 | release Snapshot view를 만든 뒤 별도 Live frame을 준비해도 Snapshot member/key/value가 불변임을 확인 |
| O10 | 실제 ordered DamageExpr에서 직원 aggregate amount 8이 원래 직원 A/B/소환물 대상 모두에 적용되고 대상 목록이 유지됨을 확인. 기존 target-locked projectile Snapshot/Live와 area tick Live 시험도 전체 회귀에서 통과 |
| O11 | 첫 literal effect 뒤 관계 오류가 난 candidate의 damage/result/RNG/queue/log를 rollback하는 시험과 기존 overflow·예산 rollback 회귀 통과 |
| O12 | 전체 view member limit 6 경계의 5 성공/초과 거부, 전부 필터링된 입력의 낮은 authoring work 거부, exact native extent admission을 확인 |
| O13 | 실제 RON parse, serde roundtrip, source×relation 및 Any3(3) 진단, descriptor 요구 조건, 기존 authored ByOwner/production 차단 회귀 통과 |
| O14 | predicate 순열·중복의 fingerprint 동일, slot 의미 차이의 fingerprint 분리, 빈 필터와 legacy source 동일, 기존 literal digest 보존 시험 통과 |
| O15 | D04-F01 무대상 projectile 4조합과 후속 effect/step/delivery 완료, 실제 대상 capture identity 엄격 검사 회귀 통과 |

새 시험은 `value_expr_final_003`의 83개 집중 시험과 `all_targets_final_003`의 전체 1,520개에 포함된다. 실제 world inventory의 같은-definition copy가 draft의 layout/address/actual copy를 지나 battle start 및 wave spawn의 RuntimeUnit/RuntimeItem snapshot까지 유지되는 시험도 전체 회귀에 포함된다.

## 최종 검증과 비용 원자료

모든 아래 영수증의 source 전후 digest는 동일한 최종 확장 digest `eb9bd8...804a`다.

| 검증 | 결과 |
| --- | --- |
| 변경 영역 rustfmt | `changed_region_format_final_003`, exit 0 |
| `cargo check -p game_core --all-targets --offline --locked` | `check_all_targets_final_004`, exit 0 |
| `cargo test -p game_core --lib value_expr --offline --locked` | `value_expr_final_003`, 83 passed / 0 failed |
| `cargo test -p game_core --all-targets --offline --locked` | `all_targets_final_003`, 1,520 passed / 0 failed |
| 지정 allow의 clippy | `clippy_all_targets_final_003`, exit 0; 기존 warning 보존 |
| PC 36조건 | `pc_36_final_002`, 576 rows; 성공 31 / typed refusal 5; gameplay oracle·완료 상태·오류 코드 차이 0 |
| R05 exact 도구·입력 | `r05_guard_final_004`, workload 완료 후 concurrent peak 단일 경고로 exit 101 |

R05 원자료는 `evidence/r05_raw_final_004.json`, 판정 요약은 `evidence/R05_SUMMARY.json`이다. event-heavy는 peak/retained 255,558 bytes, 결정성 parity와 25ms cap을 통과했다. concurrent retained는 각 sample 4,196,928 bytes, allocation/reallocation은 364,268/6,660으로 동일하고, peak samples는 8,990,636 / 9,033,904 / 9,022,752 / 9,049,432 / 8,956,881 bytes다. 최대값은 기존 cap 9,030,188보다 19,244 bytes 높다. 정책에 따라 이 **concurrent peak 단일 초과만 경고**로 분류했다. 결과 parity, 반복성, 10ms cap, 회귀 통계, event allocation guard에는 다른 hard failure가 없다. latency gate는 적용 대상이 아니다.

PC 원자료 `pc_raw_final_002.json`에는 측정·native usage·retained extent가 포함된다. 격리 빌드가 최종 333개 제품 바이트와 일치했음은 `pc_isolate_final_002.json`에 기록했다. R05 첫 compile 실패, 보존 도구 SHA 거부, 형식 검사와 PC 비교 도구 실패는 `FAILURE_HISTORY.md` 및 동명 원 로그에 남겼다.

## 동결·잔여 범위

target은 337개 before/after archive, 변경 32개 경로, 정방향·역방향 patch, 모든 증거·인계 archive를 묶는다. `freeze.py verify`가 현재 workspace bytes와 target archive의 일치, before→after 및 after→before의 격리 byte 복원을 확인한다. 기존 dirty/untracked 자료와 선행 frozen target은 정리하거나 덮어쓰지 않았다.

D04의 이번 유닛 관계 구간은 구현·자체 검증을 마쳤다. D07 전체, WP03 stat 공식, WP04 전체, 효과·상태 조회, 실험실·편집기, 안전 초과 시 스킬 자동 취소, 20분 시험은 남아 있다. production publication 차단은 유지한다. R05 보존 baseline 생성기 `main.rs`는 exact workload identity로 동결된 입력이므로 새 `BattleUnitDraft` literal에 맞춰 갱신한 상태로 남기지 않았다. 현재 제품에서 사용하는 최종 R05 검증 실행기는 새 필드를 명시해 정상 빌드·실행했다.

독립 검토 PASS, WP02 완료, R06 완료를 선언하지 않는다. 원 대화에서 target의 변경·비용 경고·복원 증거를 독립 검토해 달라.
