# R06 WP02 D04-OWNER-F01 교정 — 독립 재검토 인계

상태: **교정 구현·자체 검증 종료 / successor target 동결 / 독립 재검토 요청 대기**.

시작점은 owner relation target manifest SHA-256 `c2e703c2a08e1fc38141d3d070e052d8207cd7aaf9790f650c685429728a2a9f`, 확장 337개 source digest `eb9bd8fb3927f8a9f3c5b35908012300e4fad9c4e8d7a03a9e612e44b0dc804a`다. 구현 전에 predecessor `freeze.py verify`로 currentness와 격리 양방향 복원을 확인하고, 그 실제 바이트를 `baseline/entry_001`에 보존했다.

최종 확장 337개 digest는 `8aa03d6bd04f8d524fa61b465a9bd7a4171062a1190f615b794e20f915190f04`, 제품 333개 digest는 `8f6a58059254d4db4dc726bc8aeb2a6c6ec348e9aef685052837df903fd94bcd`다. 제품 변경은 5개 파일뿐이다.

## 교정 결과

- `RuntimeEquipmentSlot`이 `BattleEquipmentBinding`과 정확한 `runtime_item_instance_id`를 한 entry에 함께 보관한다. inventory/RON/draft/public binding 계약은 확장하지 않았다.
- batch `spawn_scenario_group`과 Wave/shared kernel의 `spawn_scenario_group_member` 모두 unit instance ID를 먼저 계산한다. known layout projection은 원 `equipped_items`의 enumerate 순번으로 runtime item ID를 한 번 만들고, 같은 ID를 RuntimeItem과 provider 등록에 사용한다. 실제 item/provider loop 순서는 유지했다.
- Armor-only ByType의 Armor 칸이 index 1이어도 ID salt는 원 목록 순번 0이다. Any3도 주소 대신 원 목록 순번을 사용한다. legacy layout `None`은 기존 enumerate 생성 경로를 유지한다.
- `make_item_instance_id`는 기존 `equipment UUID(16) + owner UUID(16) + side(1) + big-endian salt(4)`의 정확한 37 bytes를 `[u8; 37]`에 구성한다. namespace, stable seed, 최종 UUID는 바꾸지 않았다.
- `resolved_slot_binding`에서 `items.values()` 전역 순회를 제거했다. 슬롯 entry의 ID로 `items.get(id)`를 최대 한 번 호출하고 map key/internal instance ID, owner unit, base definition, persistent copy를 모두 대조한다. nil ID, 누락 item, 다른 unit ID, 내부 instance/owner/base/copy 손상은 모두 `InvalidOwner`이며 다른 item을 찾는 fallback은 없다.
- 정상 빈 슬롯은 predicate false, unknown layout은 `MissingContext`, copy 미제공 legacy binding의 `HasEquippedSlot` 의미와 `EquippedSlot` 거부는 기존대로다.

## 작업량·retained·clone 근거

준비식의 `2 * predicates + 2 * relation_views`는 count pass와 view construction pass에서 각 predicate/relation을 대상당 최대 한 번 부르는 구조와 대응한다. 슬롯 호출 하나가 고정 metadata 검사와 직접 map lookup 최대 한 번이므로 실제 조회 수는 `2 * N * (P + R)` 이하이며 기존 논리 work 상한 안에 있다. checked 계산, `limits.max_work`, 준비 전 authoring/native charge 및 evaluator meter를 유지했다.

C01 fixture에서 집계 밖 실제 직원 장비를 0/100/1,000개 추가했을 때 전체 items는 3/103/1,003개였고 Weapon predicate+relation 조회는 모두 정확히 4회였다. C02는 N=2에서 predicate만 4, relation만 4, 둘 다 8회였고, Weapon+Armor의 서로 다른 node는 12회로 상한 16 이하였다. 관측 counter는 실제 `items.get` 직전에 둔 test 전용 thread-local이다. 코드 대조도 `resolved_slot_binding` 내부 item scan 0, 직접 ID lookup 1을 확인했다.

x86_64 최종 artifact에서 `BattleEquipmentSlots` payload는 이전 모양 106 bytes에서 154 bytes로 48 bytes 늘었다. Arc header를 포함한 `retained_extent()`는 122에서 170 bytes가 됐다. C07은 새 extent보다 1 byte 부족할 때 `RetainedByteLimit`, 정확한 한도에서 성공하는 경계를 확인했다. RuntimeUnit clone과 `reuse_timestamp_value`는 같은 Arc payload를 공유한다.

## C01–C09 대응

| ID | 최종 source 관측 |
| --- | --- |
| C01 | `equipment_lookup_count_is_constant_when_unrelated_battle_items_grow`: unrelated 0/100/1,000에서 직접 lookup 4/4/4, 전역 scan 0 |
| C02 | `equipment_lookup_count_tracks_predicates_relations_members_and_nodes`: predicate/relation 결합과 여러 slot node의 실제 횟수, 결과와 node별 view 격리 확인 |
| C03 | 기존 `resolved_view_total_limit_and_filtered_scan_work_are_enforced`, `resolved_member_preparation_admits_exact_native_extent_before_copy`, authoring/native work 및 cancellation 회귀가 낮은 한도와 filtered input 비용을 typed 거부 |
| C04 | `equipment_relation_rejects_every_direct_runtime_item_identity_mismatch`가 다른 unit ID, nil ID, map 제거, 내부 instance/owner/base/copy 불일치를 모두 `InvalidOwner`로 확인. 기존 unknown/empty/no-copy/nil/duplicate 시험 유지 |
| C05 | `equipment_projection_keeps_source_order_item_ids_for_by_type_and_any3`: ByType 앞 빈 슬롯, Any3 역주소, 같은 definition의 다른 copy에서 원 목록 salt와 slot=RuntimeItem=provider ID 확인 |
| C06 | 실제 inventory→draft→runtime 시험을 확장해 AtBattleStart와 Wave 양쪽의 layout/address/copy/runtime item ID 연결 확인 |
| C07 | `equipment_projection_retained_extent_and_arc_reuse_include_runtime_ids`: 실제 170-byte retained 경계, binding+ID 보존, clone/scratch Arc 공유 확인 |
| C08 | 기존 Snapshot/Live, ordered effect rollback, capture identity 회귀와 D04-F01 전용 9개 시험 통과 |
| C09 | `stack_item_instance_material_is_byte_identical_to_the_previous_vec_scheme`: Player/Opponent, owner/definition/salt 변화에 예전 Vec 37-byte 방식과 새 stack 방식 UUID 동일 |

기존 O01–O15 구현은 제거하거나 완화하지 않았다. 88개 `value_expr` 집중 시험이 parser/compiler/evaluator/fingerprint, node view, Snapshot/Live, rollback, 한도와 typed 오류를 다시 통과했다. D04-F01 9개를 별도 selector로 재실행했고 전체 최종 discovery 1,526개에도 O01–O15, C01–C09, 실제 inventory 경로와 D04-F01이 포함된다.

## 최종 검증

모든 영수증의 source 전후 digest는 최종 확장 digest `8aa03d6b...90f04`로 같다.

| 검증 | 결과 |
| --- | --- |
| 변경 영역 rustfmt | `changed_region_format_final_002`, exit 0. 수정 hunk와 기존 format debt 겹침 0 |
| `cargo check -p game_core --all-targets --offline --locked` | `cargo_check_all_targets_final`, exit 0 |
| 장비 집중 | `correction_equipment_focused_final`, 43 passed / 0 failed |
| value expression 집중 | `value_expr_focused_final`, 88 passed / 0 failed |
| C09 / C06 | 각각 1/0, 1/0 |
| D04-F01 | `d04_f01_regression_final`, 9 passed / 0 failed |
| 전체 제품 시험 | `cargo_test_all_targets_final`, 1,526 passed / 0 failed |
| 지정 allow의 clippy | `cargo_clippy_all_targets_final`, exit 0; 기존 warning 보존 |
| direct lookup 코드 대조 | `direct_lookup_code_scan_final`, scan 0 / exact get 1 / identity 4종 유지 |
| PC36 | 576 rows, 성공 31 / typed refusal 5. gameplay oracle·완료 상태·오류 코드·요청 상세 차이 0 |
| R05 exact workload | workload 완료 뒤 concurrent peak 단일 경고로 exit 101 |

제품 target은 `/home/blast/.cache/d04-owner-f01-product-target`, PC는 `/home/blast/.cache/d04-owner-f01-pc-target`, R05는 `/home/blast/.cache/d04-owner-f01-r05-target`으로 분리했다. PC 격리본의 복사 전 제품 333개 바이트가 현재 제품과 일치하며, hook을 추가한 built digest와 hook archive를 별도로 결속했다.

R05 event-heavy는 peak/retained 255,558 bytes, p95 13,966,056ns, 25ms cap과 결정성 parity를 통과했다. concurrent retained는 모든 sample 4,196,928 bytes이고 allocation/reallocation은 364,268/6,660으로 같다. peak는 8,874,298 / 9,038,036 / 9,011,969 / 9,041,800 / 8,985,695 bytes이며 최대값은 원 cap 9,030,188보다 11,612 bytes 높다. repeatability, p95 8,050,968ns의 10ms cap, 결정성 parity와 회귀 통계는 통과했다. 정책에 따라 이 peak assertion 하나만 경고이며 다른 hard guard failure는 없다.

모든 실패와 selector 오류는 `FAILURE_HISTORY.md`와 원 receipt/log에 남겼다. 최종 target은 337개 before/after, 변경 5개 경로, 정방향·역방향 patch, 원자료와 인계서를 포함한다. `freeze.py verify`로 workspace currentness와 격리 before→after/after→before byte 복원을 검사한다.

## 남은 범위

D07 전체, WP03 스탯 공식, WP04, 효과·상태 조회, 실험실, 편집기, 안전 초과 시 스킬 자동 취소, 20분 시험은 진행하지 않았다. 전체 expression production publication 차단은 유지한다. 독립 재검토 PASS, D04/WP02/R06 전체 완료를 선언하지 않는다.
