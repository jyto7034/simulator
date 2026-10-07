# R05 acceptance evidence — current implementation

Projection of original exact33 acceptances/12IVA/10 dimensions; implementation evidence index, not a completion verdict. Original P2G artifacts are immutable.

Final source: [input](evidence/IMPLEMENTATION_INPUT_001.json), [dirty-baseline attribution](evidence/ATTRIBUTION_001.json), [alltarget103](evidence/resource_core_alltargets_103.json). Actual1410 passed,0failed,0ignored. Required audits remain pending.

## Direct witness index

| Key | Actual witness |
|---|---|
| W01 | [external_ron_executes_each_lifetime_with_actual_employee_and_copy_sources](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W02 | [external_ron_second_cost_shortage_preserves_actual_timestamp_candidate](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W03 | [actual_withdraw_redeploy_resets_deployment_but_preserves_attempt_and_run](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W04 | [actual_retreat_restores_run_resource_entry_timeline_and_attempt_identity](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W05 | [actual_run_result_failure_retry_and_fresh_public_checkpoint_preserve_nonempty_values](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W06 | [actual_same_copy_transfer_enhance_sale_and_dismantle_preserve_or_discard_only_that_run_key](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W07 | [external_resource_reference_and_selector_negatives_keep_exact_authored_source](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W08 | [two_real_same_definition_copies_and_target_employee_selector_do_not_alias](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W09 | [released_fixed_area_keeps_delivery_and_chain_resources_after_controller_withdrawal](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W10 | [released_payload_cannot_reuse_closed_activation_and_failure_keeps_committed_withdrawal](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W11 | [released_projectile_completion_retains_delivery_after_motion_and_controller_end](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W12 | [extra_attack_child_cast_preserves_its_actual_causal_chain](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W13 | [nonempty_run_survives_repeat_retreat_second_result_floor_and_ends_at_new_run](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W14 | [actual_event_defeat_commits_nonempty_resources_once_and_retry_uses_committed_basis](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W15 | [actual_admin_acquisition_recruitment_and_employee_status_preserve_owner_identity](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W16 | [actual_shop_and_reward_acquisitions_create_distinct_spendable_copy_owners](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| W17 | [pending_nonempty_result_enforces_native_and_authoring_budgets_before_commit](../../../../../src/game/world/tests/resource_lifetimes.rs) |
| S01 | [each_lifetime_closes_its_own_entries_and_rejects_old_lease](../../../../../src/game/resource/store/tests.rs) |
| S02 | [unknown_or_wrong_owner_and_lifetime_do_not_create_state](../../../../../src/game/resource/store/tests.rs) |
| S03 | [discarded_candidate_and_new_store_cannot_reuse_old_ordinal_capability](../../../../../src/game/resource/store/tests.rs) |
| S04 | [chain_waits_for_finite_children_and_duplicate_release_cannot_close_a_new_child](../../../../../src/game/resource/store/tests.rs) |
| S05 | [activation_end_does_not_close_released_delivery_or_independent_chain](../../../../../src/game/resource/store/tests.rs) |
| S06 | [multi_cost_shortage_and_duplicate_key_aggregation_preserve_every_field](../../../../../src/game/resource/store/tests.rs) |
| S07 | [competing_reservations_lock_available_and_abort_never_grants_currency](../../../../../src/game/resource/store/tests.rs) |
| S08 | [stale_reservation_revision_and_rolled_back_handle_cannot_spend_new_reservation](../../../../../src/game/resource/store/tests.rs) |
| S09 | [owner_retention_preserves_values_and_removal_releases_other_cost_locks_without_refund](../../../../../src/game/resource/store/tests.rs) |
| S10 | [closing_one_lifetime_aborts_whole_reservation_and_retains_other_balance](../../../../../src/game/resource/store/tests.rs) |
| S11 | [grant_decay_and_consume_report_actual_changes_at_arithmetic_boundaries](../../../../../src/game/resource/store/tests.rs) |
| S12 | [failures_after_local_mutation_restore_entries_ids_and_revisions](../../../../../src/game/resource/store/tests.rs) |
| S13 | [hold_retention_and_revision_overflow_reject_without_publishing_ids](../../../../../src/game/resource/store/tests.rs) |
| S14 | [runtime_work_failure_rejects_before_store_commit](../../../../../src/game/resource/store/tests.rs) |
| S15 | [native_clone_budget_prices_store_before_copying_or_mutating](../../../../../src/game/resource/store/tests.rs) |
| S16 | [catalog_validation_and_new_run_projection_are_admitted_before_work_and_copy](../../../../../src/game/resource/store/tests.rs) |
| S17 | [deterministic_order_and_sparse_initial_survive_run_projection_and_new_store](../../../../../src/game/resource/store/tests.rs) |
| S18 | [run_result_rejects_duplicate_wrong_basis_and_pending_reservations](../../../../../src/game/resource/store/tests.rs) |
| S19 | [durable_missing_fields_unknown_owner_definition_cap_and_duplicates_are_rejected](../../../../../src/game/resource/store/tests.rs) |
| S20 | [durable_owner_removal_does_not_refund_or_close_surviving_employee_run_values](../../../../../src/game/resource/store/tests.rs) |
| S21 | [resource_fault_is_typed_and_latched_when_a_cleanup_caller_ignores_it](../../../../../src/game/resource/store/tests.rs) |
| S22 | [authoring_work_budget_also_prices_resource_walks_and_preserves_values](../../../../../src/game/resource/store/tests.rs) |
| S23 | [retained_byte_bound_includes_sparse_map_nodes_and_rejects_growth_atomically](../../../../../src/game/resource/store/tests.rs) |
| P15 | [15 actual default-release API/CLI/ordinary CoreApplication cases: six lifetimes, value mutations/new ID, independent roots, same-definition copies,32/256 resources; trace parity;256 long-clock refusal and explicit-clock completion](evidence/TYPED_RESOURCE_EXTERNAL_006/RESULT.json) |
| N13 | [13 typed invalid packs, repeated API/CLI exact source/phase diagnostics; existing valid session/replay and original files preserved](evidence/TYPED_RESOURCE_INVALID_002/RESULT.json) |
| L08 | [8 exact-minus-one receipt/work/queue/clone refusals at first cost, terminal and Complete; repeat faults preserve accepted prefix and normal replay](evidence/TYPED_RESOURCE_LIMITS_003.json) |
| ORDER | [2 resolved pack orders identical; declaration order gives distinct identity under existing resolver slots but same gameplay; second cost Consume99 rolls back first cost and matches public CLI/native failure](evidence/TYPED_RESOURCE_ORDER_003/RESULT.json) |
| LEGACY | [13 original consumer groups: CLI/privacy, API/privacy, Damage18/36, resonance17/31, Buff, generic producer invalid13, delivery, preview, R04 facts/source, replacement, byte-source receipts, ordinary application](evidence/PREDECESSOR_CONSUMERS_001/RESULT.json) |
| DIAG | [10 original generated policy/boss valid/invalid/restoration commands with repeated source-aware failures and no output/token](evidence/PREDECESSOR_DIAGNOSTIC_001/receipt.json) |
| GUARD | [Original plain/allocator public/kernel guards and native67 search/default/3 exact-one-below dimensions; original concurrent peak cap9,030,188, actual9,025,562](evidence/ORDINARY_GUARDS_003/RESULT.json) |
| COST | [Final default-release plain1025/allocator615 samples;99 zero-allocation typed reads;12 independent-generation drops per input. Requested bytes not RSS; fact-only bound is not typed-store bound](evidence/COST_SUMMARY_002.json) |
| AUDIT | Required fresh GEX20 and predecessor restoration; no positive verdict prefilled |

## Exact33 acceptance projection

| Member | Approved acceptance | Current witnesses |
|---|---|---|
| `PER-CARL-E01-REVIEWED-001[1]` | 같은 정의의 장비 두 사본과 서로 다른 직원의 값·사용기록이 분리된다. 같은 사본의 서로 다른 배치도 같은 영구 owner로 추적된다. | W08, W03, P15 |
| `PER-CARL-E01-REVIEWED-001[2]` | 자원 변경·비용 결과가 실제 source owner로 귀속되고 definition ID 또는 transient unit ID로 다른 사본과 합쳐지지 않는다. | W01, W08, P15 |
| `PER-CARL-E01-REVIEWED-001[3]` | 실재하지 않거나 서로 맞지 않는 owner/binding은 명시적으로 거부되며 다른 소유자의 상태를 변경하지 않는다. | W07, S02, S03, W05, N13 |
| `PER-CARL-E02-REVIEWED-001[1]` | 같은 사본의 Deployment 상태는 철수·재배치로 종료/초기화되고 Attempt 기록은 같은 전투 도전 안에서 유지된다. Run 상태는 Attempt·Deployment 종료만으로 초기화되지 않는다. | W03, W13, W14, P15 |
| `PER-CARL-E02-REVIEWED-001[2]` | Activation/Delivery/Chain 각각의 실제 시작·종료·취소 경계에서 상태와 예약/lease가 누출되지 않으며 stale handle/timer와 중복 종료는 새 인스턴스에 영향을 주지 않는다. | S01, S03, S04, S05, S10, W09, W10, W11, W12 |
| `PER-CARL-E02-REVIEWED-001[3]` | 이미 방출한 전달체는 현재 승인된 독립 수명을 지킨다. controller 취소가 모든 released payload의 무조건 취소나 미생성 후속 효과의 존속으로 바뀌지 않는다. | W09, W10, W11, LEGACY |
| `PER-CARL-E02-REVIEWED-001[4]` | 여섯 수명 중 실제 consumer와 종료 검증이 없는 수명을 구현 완료로 표시하지 않는다. 후속 full Flow·branch/join 기능을 R05 완료의 대용으로 만들지 않는다. | W01, W03, W09, W10, W11, W12, W13, W14, P15 |
| `PER-CARL-E03-REVIEWED-001[1]` | 유효 초기값은 0 이상 상한 이하이며 잘못된 타입·범위·누락·비유한 값·미지원 수명/owner 조합은 설명 가능한 거부다. 누락을 0 또는 무제한으로 메우지 않는다. | W07, S19, N13 |
| `PER-CARL-E03-REVIEWED-001[2]` | grant/decay 결과는 요청량과 실제 변화량을 구별한다. 상한 초과분이나 없는 잔량을 환불·부채·다른 자원으로 변환하지 않는다. | S11, W01, P15 |
| `PER-CARL-E03-REVIEWED-001[3]` | 상한보다 큰 잔량, 산술 overflow 또는 잘못된 비용 입력이 성공으로 게시되지 않는다. 실행 안전 예산과 authored gameplay cap을 구별한다. | S11, S13, S14, S15, S16, S23, W02, W17, L08, P15 |
| `PER-CARL-E03-REVIEWED-001[4]` | 기존 공명/전술 자원 동작은 보존한다. 시험 입력의 초기량·상한·소비량은 공식 밸런스 결정이 아니다. | LEGACY, GUARD, W06 |
| `PER-CARL-E04-REVIEWED-001[1]` | 여러 비용 중 하나가 부족하거나 중간 처리가 실패하면 부분 지출·다른 사본 지출·성공 receipt·ID/RNG/queue 누출이 없다. | W02, W17, S06, S12, ORDER, L08 |
| `PER-CARL-E04-REVIEWED-001[2]` | 동일 reservation의 중복 commit/abort 또는 stale revision/epoch는 추가 지출·환불·재생성을 일으키지 않는다. 예약된 잔량을 다른 소비가 다시 지출할 수 없다. | S07, S08, S10, S18 |
| `PER-CARL-E04-REVIEWED-001[3]` | Cost/ResourceChange 결과는 실제 before/after와 소유자·자원을 나타내며 로그 보존 여부가 원자성/권한/상태의 authority가 되지 않는다. | W01, W08, P15, COST |
| `PER-CARL-E04-REVIEWED-001[4]` | 이미 완료된 이전 timestamp 전체를 rollback하는 것으로 과장하지 않고 현재 공개 transaction의 실패 경계를 검증한다. | W10, W17, L08, ORDER, P15 |
| `PER-CARL-E05-REVIEWED-001[1]` | 동일 item UUID의 해제·다른 직원 장착·강화 전후에 Run 자원 잔량이 같다. 다른 직원/장비의 자원과 섞이지 않는다. | W06, W08 |
| `PER-CARL-E05-REVIEWED-001[2]` | 정상 판매·분해가 성공할 때 그 사본 전용 자원과 binding이 정리된다. 거부된 판매·분해에서는 사본과 자원이 모두 보존된다. | W06, S09, S20 |
| `PER-CARL-E05-REVIEWED-001[3]` | 자원 잔량 때문에 판매/분해 보상량이 늘거나 다른 사본에 자원이 지급되지 않는다. 기존 판매/분해 정책의 보상·비용은 그대로다. | W06, W16 |
| `PER-CARL-E06-REVIEWED-001[1]` | 자원이 도입되는 시점에 save writer/reader/validator, entry capture/restore, result delta와 rollback이 함께 작동한다. 별도 후속 R을 기다려야 하는 저장 공백이 없다. | W05, W13, W17 |
| `PER-CARL-E06-REVIEWED-001[2]` | 동일 owner의 전투 중 변화가 정상 승리·패배 결과 확정에서 한 번만 적용되고 같은 결과의 재적용은 자원 증감 중복을 만들지 않는다. | W05, W14, S18 |
| `PER-CARL-E06-REVIEWED-001[3]` | retreat/retry 시 자원·attempt·RNG/UUID 등 기존 rollback 범위를 복원한다. RuntimeUnit 철수·재배치는 Run 자원을 되감지 않는다. | W03, W04, W13, W14 |
| `PER-CARL-E06-REVIEWED-001[4]` | 체크포인트 round-trip은 owner identity/잔량/세대 결속을 보존한다. 기존 loads_used·run peak·개인 profile 등 rollback 예외를 새 자원 복원에 섞어 되감지 않는다. | W05 |
| `PER-CARL-E06-REVIEWED-001[5]` | 직원 상태 변화가 직원 identity를 제거하지 않는 한 해당 직원 Run 상태를 임의 소멸시키지 않는다. 다른 사본·독립 진행·보상 기록을 함께 삭제하지 않는다. | W15, W16, W06 |
| `PER-CARL-E07-REVIEWED-001[1]` | 새 application에서 같은 유효 콘텐츠·계약의 새 Run 저장을 읽어 동일한 자원 owner/값/진행을 얻는다. compiled index/pointer가 영구 identity가 되지 않는다. | W05, S17, S19 |
| `PER-CARL-E07-REVIEWED-001[2]` | 동일 정의 ID라도 fingerprint/자원 계약이 다르거나 저장 owner가 실재하지 않으면 기존 런·profile·입력 파일을 바꾸지 않고 실패한다. | W05, S19 |
| `PER-CARL-E07-REVIEWED-001[3]` | 기존 R02 latest-only 정책의 미지원 버전 거부를 유지하고 이전 저장을 암묵적으로 0 자원으로 변환하지 않는다. | W05, LEGACY |
| `PER-CARL-E08-REVIEWED-001[1]` | 같은 엔진에서 새 ID/같은 정의 두 사본의 외부 팩을 읽고 실제 authoring trial과 게임 실행으로 값·비용·수명 결과를 관찰한다. | P15, W08 |
| `PER-CARL-E08-REVIEWED-001[2]` | 지원하지 않는 자원 kind/owner/lifetime/operation 또는 잘못된 참조는 public validator에서 사실대로 거부되며 빈 executor/성공 fallback이 없다. | N13, W07, S02 |
| `PER-CARL-E08-REVIEWED-001[3]` | 새 자원 결과는 직접 소비자·설명 포트에 일관되게 전달된다. 기존 공식 RON·schema 소비자와 같은 최신 경로를 사용한다. | P15, LEGACY, DIAG |
| `PER-CARL-E09-REVIEWED-001[1]` | owner/container/pack 순회 순서와 진단 on/off가 승인된 게임 의미·RNG 권한을 바꾸지 않는다. stale/중복/부족 비용·취소·실패/restore를 실제 nonzero 시험으로 검증한다. | S03, S07, S08, S17, W03, W10, W12, W13, P15, ORDER |
| `PER-CARL-E09-REVIEWED-001[2]` | 새 state/clone/receipt/queue 경계의 현실 입력과 adversarial 입력을 측정하고 경계 초과 시 typed fault와 transaction 무변경을 검증한다. 0 tests/skip/검증기 통과를 게임 성공으로 세지 않는다. | S13, S14, S15, S16, S21, S22, S23, W17, P15, L08, GUARD, COST |
| `PER-CARL-E09-REVIEWED-001[3]` | T05/T20/T23/T29 및 R05 카드의 여섯 수명·실제 owner/저장 전환을 요구별 근거로 대조한다. 기존 단위 PASS나 작은 내부 store test만으로 전체 완료를 주장하지 않는다. | P15, LEGACY, DIAG, GUARD, COST |
| `PER-CARL-E09-REVIEWED-001[4]` | 구현을 멈춘 별도 fresh 감사 pass와 적용 reviewer 조건을 충족한다. 구현에 참여하지 않은 reviewer의 감사 기록을 남기고 필수 미완료를 complete로 쓰지 않는다. | AUDIT |

## Twelve integration seams

| Original requirement | Direct witnesses |
|---|---|
| IVA-CARL-01 | W01, W03, W07, W08, P15, N13 |
| IVA-CARL-02 | W01, W03, W04, W13, W14, S01 |
| IVA-CARL-03 | W09, W10, W11, S01, S03, S05 |
| IVA-CARL-04 | W12, W09, S04, S10, P15 |
| IVA-CARL-05 | W01, W07, S11, S13, S19, N13, P15, LEGACY |
| IVA-CARL-06 | W02, W10, W17, S06, S07, S08, S12, S18, L08, ORDER |
| IVA-CARL-07 | W06, W15, W16, S09, S20 |
| IVA-CARL-08 | W04, W05, W13, W14, W17, S18 |
| IVA-CARL-09 | W05, S17, S19 |
| IVA-CARL-10 | P15, N13, LEGACY, DIAG |
| IVA-CARL-11 | S13, S14, S15, S16, S17, S21, S22, S23, W17, P15, L08, ORDER, GUARD, COST |
| IVA-CARL-12 | LEGACY, DIAG, GUARD, COST, AUDIT |

## Ten required dimensions

| Original requirement | Direct witnesses |
|---|---|
| success | W01, W03, W05, W06, W08, W09, W11, W12, W13, W14, W15, W16, P15 |
| invalid_input | W07, S02, S19, N13, DIAG |
| failure_rollback | W02, W05, W10, W17, S12, S14, S15, L08, ORDER |
| retry_idempotency | S03, S04, S07, S08, S18, W05, W13, W14, L08 |
| ordering_concurrency | S07, S17, W08, W12, P15, ORDER |
| boundary_empty_max | S11, S13, S16, S19, S23, W17, N13, L08 |
| disconnect_restart_persistence | W03, W04, W05, W06, W13, W14, LEGACY, COST |
| live_data | P15, N13, LEGACY, DIAG |
| producer_consumer | W01, W05, W08, W15, W16, P15, LEGACY, DIAG |
| legacy_fallback_absence | W07, S02, S03, S19, S21, N13, LEGACY, AUDIT |

## Completion conditions

| Original requirement | Direct witnesses |
|---|---|
| C01 | W08, W07, S02, P15, N13 |
| C02 | W01, W03, W09, W10, W11, W12, W13, W14, S01, S03, S04 |
| C03 | S06, S07, S08, S11, S12, S18, W02, ORDER |
| C04 | W06, W15, W16, S20 |
| C05 | W04, W05, W13, W14, W17 |
| C06 | P15, N13, LEGACY, DIAG |
| C07 | L08, ORDER, GUARD, COST, W17, S13, S14, S15, S16, S21, S22, S23 |
| C08 | LEGACY, DIAG, GUARD, COST, AUDIT |

## R05 card and roadmap contract

| Applicable requirement | Scope and evidence |
|---|---|
| T05 | Actual copy ownership, Attempt persistence, Deployment reset, atomic costs and stale capability blocking; W01, W03, W08, S01, S03, S06, S07, W02, P15, ORDER |
| T20 | Current V7 identity/owner/save, exactly-once result, retry/retreat distinction, readonly reconnect and independent profile preservation; W04, W05, W06, W13, W14, LEGACY, COST |
| T23 | Compile/native/runtime/output finite admission, typed rollback and deterministic source/owner/pack behavior; original performance thresholds preserved; S14, S15, S16, S21, S22, S23, W17, P15, L08, ORDER, GUARD, COST |
| T29 | Strict raw/final validation, valid Replace/invalid references, acquisition/source receipts, failed replacement, fingerprint refusal and actual public atomic prefix; no rollback of earlier committed transactions; W05, W10, P15, N13, L08, ORDER, LEGACY, DIAG |
| R05_card_AT05 | All six actual lifetimes, persistent employee/copy binding, store and reserve/commit/abort, Run entry/result/checkpoint/floor integrated at introduction; no future full Flow/relic/fragment-copy implementation assumed; W01, W03, W05, W06, W08, W09, W10, W11, W12, W13, W14, W15, W16, W17, P15, AUDIT |

## Current limits and remaining work

- Official Resources catalog stays empty; external initial/cap/operation counts are test values, not official balance
- 256-resource long clock has explicit atomic clone-limit refusal; completion is separately proven using explicit bounded clock commands
- System allocator requested bytes are not RSS; RuntimeFactUsage does not bound the whole typed store
- Default release public consumers are distinct from test-fixtures native guard harness
- No server/Unity/deployment/host personal-store migration or future R06+ work is claimed
- Required fresh R01-R04 restoration after R05 shared-source/schema changes
- Separate implementation-nonparticipant R05 GEX20
- Final target/evidence closure and parent R05 reconciliation
