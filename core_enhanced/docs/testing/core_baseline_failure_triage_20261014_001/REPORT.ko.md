# Core 기존 실패 138개 진단 및 처분 판단

현재 실패 원인은 **테스트 준비/기대값 노후화 137개, 실행 환경 1개**다. 제품 결함으로 확정된 사례는 0개다. 이는 제품에 결함이 없다는 선언이 아니다. 격리 복사본의 시험 준비·기대값만 보정하여 65개는 본래 assertion 전체를 통과했고, 나머지 테스트 측 72개는 현재 실패 원인을 설명했지만 모든 후속 동작의 통과까지 확인하지 않았다. 원본 테스트는 여전히 138개 실패다.

이번 범위는 원인·가치·기존 검증 중복·유지/제거 판단이다. 원본 제품 코드와 테스트는 수정하거나 삭제하지 않았다. 검증 대상 동결 source 713개는 작업 종료 시 모두 동일하다.

| 원인 | 개수 | 판정 |
| --- | ---: | --- |
| 준비 입력 노후화 | 109 | 장비 역할, 필수 명령 필드, 전투 난이도/활동, 현재 시작·지도·저장 경로와 불일치 |
| 기대값 노후화 | 28 | 버전/명령 수/진단 문자열/채용 완료 시점/보존 로그/시간 가정이 구형 |
| 환경 | 1 | Unix socket 생성이 EPERM. 제품 loader 호출 전 실패 |

| 권고 | 테스트 수 | 실행 여부 |
| --- | ---: | --- |
| 유지하고 현행 계약으로 보정 | 135 | 미적용 |
| 기존 검사에 잔여 항목을 합친 뒤 중복 본문 제거 | 1 | 미적용 |
| 정상 경로 유지, 저장 위조 전용 분기 제거 | 1 | 미적용 |
| 환경 조건을 분리하여 유지 | 1 | 미적용 |

전체 테스트를 즉시 삭제할 대상으로 확정한 것은 없다. 단순히 이름에 legacy/v12/Standard가 있거나 같은 모듈에 새 테스트가 있다는 이유로 삭제하지 않는다.

## 제거·통합 판단

1. `start_new_game_creates_starter_employee_roster_and_equipment`: U06의 `u06_starter_six_normal_sets_grant_exact_copies_and_continue_bindings`가 여섯 역할별 장비·파편과 역방향 소유·Continue를 더 넓게 검사한다. 기존 테스트에 남은 combat eligibility 수, inventory ledger consistency, run snapshot의 `roster_order` 비노출 assertion을 적절한 현행 검사에 합친 뒤 중복 본문을 제거하는 것이 좋다. 지금 그대로 삭제하면 잔여 검증을 잃는다.
2. `durable_continue_recomputes_battle_setup_instead_of_trusting_serialized_preview_cache`: 정상 Continue 후 전투 setup 재계산은 유지한다. 저장 JSON의 enemy를 바꾸고 checksum을 재계산하는 후반 분기는 [로드맵 §1.9](../../roadmaps/core_first/ROADMAP.ko.md#19-검증-범위-제한--외부-임의-변조-방어-제외)의 제외 범위다. 이 분기를 살리기 위한 교정은 하지 않는다. U14 preview refresh와 부분 중복하지만 새 인스턴스 Continue→실제 전투 시작까지 완전히 같지는 않아 정상 분기를 즉시 삭제하지 않는다.
3. 명령 수 검사 3개는 각각 application/domain bijection, route/scope/authority, 단일 application owner를 검사한다. 숫자가 같아도 대체 관계가 아니다.
4. U10 인과 체인 39개와 외부 RON resource lifetime 18개는 유지 가치가 높다. 최근 통합 검사나 하위 단위 검사가 각 복제본·합류·지연·회수·실제 public commit 경계를 모두 대체하지 않는다.
5. 정상 저장 version 검사 8개도 유지한다. U12의 current/old-version 검사는 정확한 keyset, reservation, 외부 generation 및 독립 owner 보존을 전부 대신하지 못한다.

## 실행 근거와 한계

- 원본 `current138_001`: 0 PASS / 138 FAIL / 0 ignored / 1965 filtered, test 18.63초. 과거 실패 목록과 정확히 같은 138개를 현재 source에서 재현했다.
- `diagnostic138_001`: 격리 복사본의 docs include fixture 9개가 누락되어 빌드 실패했다. 시험 실행 0이며 PASS 증거가 아니다. 정확한 현재 bytes를 복사하고 [추가입력](DIAGNOSTIC_EXTRA_INPUTS.json)을 보존했다.
- `diagnostic138_002`: 61 PASS / 77 FAIL / 0 ignored / 1965 filtered, test 39.24초. 제품 실행 코드는 그대로이며 [시험 변경](DIAGNOSTIC_001.diff)만 사용했다. 39개 U10 인과 체인·8개 정상 codec·7개 전투 준비·3개 명령 수·definition count·site·resolver·member-scope 검사가 통과했다.
- `diagnostic5_003`: 진단 코드가 sibling private method를 호출하여 빌드 실패했다. 시험 실행 0. 진단을 공개된 effective profile의 affinity 관측으로 바꾸고 [후속 변경](DIAGNOSTIC_004.diff)을 보존했다.
- `diagnostic5_004`: 4 PASS / 1 FAIL / 0 ignored / 2098 filtered, test 9.88초. Ground와 맞는 직원 선택으로 배치 2개, 3000의 시험 전용 work 한도로 fault gate 1개, 100.4초 이내 유한 관측으로 보호막/parry→정상 종료→fresh restore 1개가 통과했다. 제품 기본 한도나 종료 조건은 바꾸지 않았다.
- 남은 로그 사례는 실제 종료 후 보존 buffer만 검사한다. 관측 floor_seq=540, 780 entries, 첫 잔류 시각 12,150ms로 초기 spawn이 제거된 상태다. `UnitSpawned`를 초기/진행 events_delta에서 검사하도록 보정해야 한다.
- Trial 쪽은 장비 준비를 고치자 다음 Standard 모드 오류가 드러났다. R07은 시설 graph connection fixture 불일치가 드러났고, resource lifetime 18개는 자체 장비 덮어쓰기가 남았다. 따라서 공통 helper 한 줄 변경으로 138개를 해결했다고 주장하지 않는다.
- resolver 한도 검사는 필수 빈 loan list를 추가하면 JSON node 수도 늘어난다. old 51/50 경계를 그대로 두고 통과시키면 안 되며 current exact N/N−1을 계산해야 한다.
- [개별 원문·후속 실패·대체 범위](MATRIX.json), [현재 실패 원문](current138_001.stdout.log), [격리 실험 원문](diagnostic138_002.stdout.log), [추가 관측](diagnostic5_004.stderr.log), [실행별 종료 코드](EXECUTION_INDEX.json), [source 동일성](SOURCE_RECHECK.json). 기존 coverage의 존재와 검사 범위는 소스를 확인했다. 아래 추가 재실행이 명시된 사례 외에는 이번 실행의 PASS로 승격하지 않았다.

원본 코드에서 대체 후보 12개를 별도로 재실행하여 **12 PASS / 0 FAIL / 0 ignored**를 확인했다(test 11.84초). U06 역할별 세트·채용 3개, U12 codec 4개와 시작 계약 2개, U13 방/미지원 활동 2개, U14 preview 1개다. [정확한 선택](current_alternatives_selection.json)과 [실행 원문](current_alternatives12_001.stdout.log)을 보존했다. 통과해도 위에서 지적한 고유 assertion까지 자동으로 대체되지는 않는다.

## 교정 순서

1. 현행 작성된 전투 노드의 난이도·지원 activity·실제 방 상태를 함께 보존하는 공통 준비를 정리한다. 격리 실험의 slot 우선 선택은 원인 확인용이며 공식 경로를 구형 Tutorial 지도 설치로 치환하는 최종안이 아니다.
2. 외부 RON의 역할별 시작 세트와 resource fixture 자체의 장비 덮어쓰기를 교정한다. 각 lifetime/skill/source 계약이 유지되는 장비를 선택한다.
3. Trial은 현행 시작과 시설 진행을 명시적인 공개 명령으로 준비하고 binding index·none slot·receipt 한도·clock 관측을 함께 갱신한다.
4. 버전·명령 집합·지원 metadata·채용 session·snapshot 기대값을 현행 계약으로 맞춘다. 이전 저장 거절과 현재 정상 저장을 혼동하지 않는다.
5. 개별 배치·로그·전투 관측·fault 입력을 보정하고 위 두 통합/부분제거 권고를 적용한다. 보정 후 처음 도달하는 assertion에서 실패가 생기면 제품 결함 여부를 새로 판정한다. 현재 분류를 근거로 자동으로 기대값을 바꾸지 않는다.

## 원인별 근거

| ID | 수 | 설명 | 소스 근거 |
| --- | ---: | --- | --- |
| E01 | 1 | UnixListener::bind가 OS EPERM으로 실패하며 socket 입력을 loader에 전달하기 전 중단된다. | `core_enhanced/src/game/data/content_pack/snapshot/tests.rs:232` |
| F01 | 41 | 시험 fixture가 Healer/Tank를 포함한 모든 시작 직원을 sword 세트로 덮어써 현재 역할-무기 검증에서 거절된다. | `core_enhanced/src/game/data/content_pack/tests.rs:44`<br>`core_enhanced/src/game/data/validation.rs:517` |
| F02 | 42 | 공통 helper가 첫 허용 slot을 Combat으로 바꾸지만 원래 SavePoint 등 비전투 node의 combat_difficulty를 만들지 않는다. | `core_enhanced/src/game/world/tests/mod.rs:1628`<br>`core_enhanced/src/game/world/tests/u10_causal.rs:227` |
| F03 | 8 | Trial preparation이 제거된 Standard 모드를 명령 JSON으로 사용한다. | `core_enhanced/src/authoring/trial.rs:1110`<br>`core_enhanced/src/game/world/tests/u12_lifecycle/start_identity.rs:47` |
| F04 | 2 | 현행 명령의 필수 loan_copy_ids가 테스트 JSON에 없다. | `core_enhanced/src/authoring/resolve.rs:946`<br>`core_enhanced/src/game/world/tests/u12_lifecycle/start_identity.rs:75` |
| F05 | 1 | unknown field를 검사하려는 RON이 현재 필수 필드도 누락하여 앞선 parser 오류로 종료한다. | `core_enhanced/src/game/data/run_policy_data.rs:1407` |
| F06 | 4 | 공통 helper가 노드 kind/payload만 바꾸고 시설 activity의 지원 상태를 맞추지 않아 실제 guard에서 거절된다. | `core_enhanced/src/game/world/tests/mod.rs:1628`<br>`core_enhanced/src/game/world/facility_rooms.rs:80` |
| F07 | 3 | 시험용 장비 catalog가 현재 candidate_2의 test_starter_shield를 누락한다. | `core_enhanced/src/game/data/mod.rs:1143` |
| F08 | 1 | 현행 map에서 구형 standard_branching slot 이름을 찾는다. 후반에는 저장 preview를 위조하고 checksum을 재계산한다. | `core_enhanced/src/game/world/tests/combat.rs:937`<br>`core_enhanced/src/game/world/tests/combat.rs:1077`<br>`core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md:345` |
| F09 | 1 | 선발 직후 Available Combat이 있다는 가정으로 position().unwrap()한다. 현행 Expedition은 선행 SavePoint/시설 진행이 있다. | `core_enhanced/src/game/data/content_pack/tests.rs:1572` |
| F10 | 1 | 임의 event node/VN roots를 주입하고 내부 save helper만 호출한다. 현재 공개 LoadRunCheckpoint 자격이 없어 목표 graph 복원에 도달하지 않는다. | `core_enhanced/src/game/world/tests/snapshots_and_start.rs:814`<br>`core_enhanced/src/game/world/node_flow.rs:1405` |
| F11 | 1 | helper를 반복 호출하면 같은 첫 slot/node를 Maintenance로 덮어써 event와 maintenance가 동일해진다. | `core_enhanced/src/game/world/tests/u08_event_owner.rs:307`<br>`core_enhanced/src/game/world/tests/mod.rs:1628` |
| F12 | 1 | run floor_index만 3으로 바꾸고 실제 방 layout/map은 이전 상태로 남겨 저장 layout 검증에서 거절된다. | `core_enhanced/src/game/world/tests/node_sessions.rs:1046` |
| F13 | 1 | 시험 전용 max_work=1000이 battle entry snapshot도 수용하지 못해 의도한 Elapsed fault 주입 전에 실패한다. 시험 한도 3000에서 입장→실제 60초 Elapsed work fault→모든 gate/first cause/roots assertion이 통과했다. | `core_enhanced/src/application/fault_tests.rs:113` |
| F14 | 2 | 관측에서 첫 직원 field_medic의 Ground affinity=false였다. 같은 준비에서 Ground-compatible 직원을 선택하면 원래 paused/raw actor assertion 전체가 통과한다. | `core_enhanced/src/game/world/combat.rs:228` |
| X01 | 8 | 현재 writer version 15를 정상 저장하면서 positive test가 12를 기대한다. | `core_enhanced/src/game/world/active_run.rs:39` |
| X02 | 3 | 현행 명령 80개에 대해 이전 79개를 고정했다. | `core_enhanced/src/application.rs:1846` |
| X03 | 1 | 지원 definition kind가 46개인데 45개를 기대한다. | `core_enhanced/src/authoring/tests.rs:233` |
| X04 | 1 | 현행 admission site는 effective_skill_definition_copy이며 테스트 진단 문자열만 구형이다. | `core_enhanced/src/game/battle/core/commands.rs` |
| X05 | 1 | 현재 LIVE_RON_SCHEMA_VERSION은 6인데 정상 입력에 5, 진단 기대에 exactly 5를 사용한다. | `core_enhanced/src/game/data/live_ron.rs:2584` |
| X06 | 4 | 기대 capability 목록에 현행 RequestRelicEffectivePlan 및 terminal StartNewGame 항목이 빠져 있다. | `core_enhanced/src/game/world/action_capability.rs` |
| X07 | 3 | 한 명 채용 후 즉시 session 종료·잔여 후보 queue 반환을 기대한다. 현행 규칙은 최대 두 명 채용 또는 명시 종료다. | `core_enhanced/src/game/recruitment.rs`<br>`core_enhanced/docs/game_rules/run-and-map.md:185` |
| X08 | 1 | 모든 시작 직원이 sword를 받는다는 전제가 현재 역할별 세트와 충돌한다. | `core_enhanced/src/game/world/tests/snapshots_and_start.rs:1079`<br>`core_enhanced/src/game/world/tests/u06_starter.rs:48` |
| X09 | 4 | 현재 Expedition/facility를 준비하면서 Standard 또는 구형 template ID를 기대한다. | `core_enhanced/src/game/world/tests/mod.rs:550`<br>`core_enhanced/src/game/world/tests/u13_facility.rs:93` |
| X10 | 1 | 장기 전투 종료 후 bounded event log에 초기 UnitSpawned가 영구 잔류한다고 기대한다. 전투는 실제 종료했다. 진단에서 floor_seq=540, retained entries=780, 첫 잔류 시각=12,150ms를 확인했다. | `core_enhanced/src/game/world/tests/combat.rs:2781`<br>`core_enhanced/src/game/battle/event_log.rs:783` |
| X11 | 1 | 보호막 소비·parry가 통과한 뒤 구형 10.4초 관측 상한에서만 중단됐다. 100.4초 이내의 유한 관측으로 원래 정상 종료·결과 처리·fresh restore assertion 전부 통과했다. | `core_enhanced/src/game/world/tests/r09_combat.inc.rs:334` |

## 기존 검증과 대체 가능성

기존 검증을 발견했다는 사실만으로 완전 대체로 판정하지 않았다. 각 테스트의 정확한 source/후속 실패/권고는 MATRIX에 연결된다.

| 계열 | 기존 검증 위치 | 판단 |
| --- | --- | --- |
| trial | `core_enhanced/src/authoring/tests.rs:313`<br>`core_enhanced/src/authoring/tests.rs:639`<br>`core_enhanced/src/game/data/content_pack/tests.rs:703` | 부분 중복. explain/metadata 단위 검사는 실제 TrialSession 접수·실행·영수증·clock·fresh process 및 원본 세션 보존을 대체하지 않는다. |
| causal | `core_enhanced/src/game/world/tests/u10_causal/audit_joint_cases.rs:391`<br>`core_enhanced/src/game/world/tests/u10_causal/audit_joint_cases.rs:732`<br>`core_enhanced/src/game/battle/core/relic_causal_tests.rs:257` | 부분 중복. 실제 기본공격/장비반격/침식 조합 및 하위 BattleCore 검사는 이 39개의 각 합류·반복·복제본·지연 수명 oracle를 대체하지 않는다. |
| resource | `core_enhanced/src/game/world/tests/resource_lifetimes.rs:1201`<br>`core_enhanced/src/game/world/tests/u12_lifecycle/codec.rs:175`<br>`core_enhanced/src/game/world/tests/u06_starter.rs:751` | 부분 중복. 작성 오류·대여 복제본·장비 호환성 검사는 실제 비어 있지 않은 typed resource의 attempt/run/delivery/chain 수명 및 회수·교환·retry를 대체하지 않는다. |
| codec | `core_enhanced/src/game/world/tests/u12_lifecycle/codec.rs:119`<br>`core_enhanced/src/game/world/tests/u12_lifecycle/codec.rs:175`<br>`core_enhanced/src/game/world/tests/u12_lifecycle/codec.rs:213`<br>`core_enhanced/src/game/world/tests/u12_lifecycle/codec.rs:297` | 부분 중복. current Continue·profile·구형 거절은 존재하지만 정확한 envelope/keyset/checksum, reservation 및 외부 generation/서로 다른 소유자의 보존 검사를 모두 대체하지 않는다. |
| starter | `core_enhanced/src/game/world/tests/u06_starter.rs:122`<br>`core_enhanced/src/game/world/tests/u06_starter.rs:159`<br>`core_enhanced/src/game/world/tests/u06_starter.rs:208` | 높은 부분 중복. 6개 역할별 세트/복제본 역방향 소유/Continue, 채용 rollback/10명 상한은 존재한다. 기존 queue의 순서·세션 유지·완료 시점이나 snapshot 필드 검사는 별도 확인이 필요하다. |
| facility | `core_enhanced/src/game/world/tests/u13_facility.rs:93`<br>`core_enhanced/src/game/world/tests/u13_facility.rs:140`<br>`core_enhanced/src/game/world/tests/u13_facility.rs:165`<br>`core_enhanced/src/game/world/tests/u14_deterrence.rs:81` | 부분 중복. 정상 방 이동·SavePoint·미지원 활동 거절·preview refresh는 존재한다. 개별 event history/monolith/전체 snapshot 보존은 완전 대체되지 않는다. |
| combat | `core_enhanced/src/game/world/tests/combat.rs:3577`<br>`core_enhanced/src/game/world/tests/combat.rs:6031`<br>`core_enhanced/src/application/fault_tests.rs:389` | 부분 중복. cursor/retained history/fault 검사는 각각의 paused deploy·raw authored actor·VN·field work·종료 결과 oracle와 다른 경계다. |
| fault | `core_enhanced/src/application/fault_tests.rs:288`<br>`core_enhanced/src/application/fault_tests.rs:346`<br>`core_enhanced/src/application/fault_tests.rs:389`<br>`core_enhanced/src/application/fault_tests.rs:836` | 부분 중복. 다른 fault origin·prepared host tick은 존재하지만 Elapsed work fault 이후 64회 모든 공개 mutation gate/first cause/roots를 검사하는 본 사례를 완전히 대체하지 않는다. |
| resolver | `core_enhanced/src/authoring/resolve.rs:646`<br>`core_enhanced/src/authoring/resolve.rs:946`<br>`core_enhanced/src/game/world/tests/u12_lifecycle/start_identity.rs:75` | 서로 보완. mandatory loan list의 명령 거절과 JSON pointer 불변성/aggregate node exact-limit은 다른 계약이다. |
| api | `core_enhanced/src/application.rs:1846`<br>`core_enhanced/src/game/world/application.rs`<br>`core_enhanced/src/game/behavior.rs` | 숫자는 같아도 application↔domain bijection, route/scope/authority, 단일 application owner를 각각 검사한다. 단순 중복 삭제 대상이 아니다. |
| content | `core_enhanced/src/authoring/tests.rs:193`<br>`core_enhanced/src/game/data/content_pack/tests.rs:866`<br>`core_enhanced/src/game/data/content_pack/tests.rs:1102` | 부분 중복. production diagnostic/외부 generation 보존은 존재하지만 schema 정확성·unknown field·개별 시작 실패 경계나 파일 입력 종류를 모두 대체하지 않는다. |

## 138개 개별 판정

격리 통과는 원본에 교정을 적용했다는 뜻이 아니다. “원인 확인”은 최초 실패의 원인을 확인했다는 뜻이며 모든 후속 assertion의 통과를 뜻하지 않는다.

| ID | 테스트 | 원인 | 권고 | 검증 깊이 |
| --- | --- | --- | --- | --- |
| T001 | `application::fault_tests::native_fault_preserves_last_root_first_cause_and_every_public_mutation_gate` | F13 | 유지·보정 | 격리 전체 통과 |
| T002 | `application::tests::command_kind_is_exactly_bijective_with_all_domain_actions` | X02 | 유지·보정 | 격리 전체 통과 |
| T003 | `authoring::resolve::tests::prior_values_resolve_to_real_command_and_context_without_mutating_the_basis` | F04 | 유지·보정 | 격리 전체 통과 |
| T004 | `authoring::resolve::tests::whole_resolved_node_count_includes_template_and_all_inserted_values` | F04 | 유지·보정 | 현재 실패 원인 확인 |
| T005 | `authoring::tests::describe_uses_current_support_and_immutable_source_with_exact_output_limit` | X03 | 유지·보정 | 격리 전체 통과 |
| T006 | `authoring::trial::tests::amount_completion::r06_public_authored_member_scope_preserves_valid_inputs` | F01 | 유지·보정 | 격리 전체 통과 |
| T007 | `authoring::trial::tests::amount_completion::r06_public_dynamic_query_limits_identity_lifecycle_and_trace_independence` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T008 | `authoring::trial::tests::amount_completion::r06_public_literal_expression_delivery_matrix_and_actual_damage_prefix` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T009 | `authoring::trial::tests::amount_completion::r06_public_operator_units_reject_before_publication_and_preserve_held_run` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T010 | `authoring::trial::tests::amount_completion::r06_public_zero_points_and_weapon_interval_refuse_and_preserve_held_run` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T011 | `authoring::trial::tests::amount_completion::r10_public_flow_amount_generation_query_trace_and_cli_export` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T012 | `authoring::trial::tests::amount_completion::r12_public_boss_flow_projectile_area_milestones_and_cli_export` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T013 | `authoring::trial::tests::amount_completion::r12_public_sustained_protection_amount_trace_and_cli_export` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T014 | `authoring::trial::tests::amount_order::d07_f03_c07_public_replacement_rejects_actual_old_exact_and_retains_zero` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T015 | `authoring::trial::tests::d07_b09_b14_amount_query_states_are_typed_and_do_not_advance_the_trial` | F03 | 유지·보정 | 현재 실패 원인 확인 |
| T016 | `authoring::trial::tests::d07_b15_public_trial_amount_preview_explain_and_exact_use_actual_legacy_execution` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T017 | `authoring::trial::tests::optional_trace_private_trial_keeps_complete_receipts_and_public_results_identical` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T018 | `authoring::trial::tests::private_trial_prefix_keeps_public_slots_and_survives_domain_and_control_failure` | F03 | 유지·보정 | 현재 실패 원인 확인 |
| T019 | `authoring::trial::tests::private_trial_preparation_and_execution_enforce_actual_clock_and_final_readiness` | F03 | 유지·보정 | 현재 실패 원인 확인 |
| T020 | `authoring::trial::tests::private_trial_preparation_uses_one_scope_keeps_none_slots_and_rejects_late_output` | F03 | 유지·보정 | 현재 실패 원인 확인 |
| T021 | `authoring::trial::tests::private_trial_step_receipt_failure_discards_real_start_and_retries_identically` | F03 | 유지·보정 | 현재 실패 원인 확인 |
| T022 | `authoring::trial::tests::public_trial_failures_keep_preparation_position_and_partial_simulation_owner` | F03 | 유지·보정 | 현재 실패 원인 확인 |
| T023 | `authoring::trial::tests::public_trial_preview_is_bounded_and_preserves_the_accepted_session` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T024 | `authoring::trial::tests::public_trial_session_preserves_receipt_order_none_slots_and_finite_completion` | F03 | 유지·보정 | 현재 실패 원인 확인 |
| T025 | `authoring::trial::tests::r07_copies::r07_f07_f08_f09_public_ron_recipe_forms_sources_resources_and_fresh_process` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T026 | `authoring::trial::tests::u07_objects::u07_public_external_ron_trial_creates_actor_traps_walls_and_detaches_pool` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T027 | `authoring::trial::tests::u08_relics::u08_external_changed_or_missing_relic_generation_rejects_original_nonempty_save_atomically` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T028 | `authoring::trial::tests::u08_relics::u08_external_relic_invalid_catalogs_preserve_published_token_and_nonempty_run` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T029 | `authoring::trial::tests::u08_relics::u08_external_ron_normal_grants_savepoint_event_replay_and_fresh_process` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T030 | `authoring::trial::tests::u08_relics::u08_external_ron_normal_reward_and_declined_corruption_consume_authored_damage_in_real_hp` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T031 | `authoring::trial::tests::u09_structural::amount_cases::rv004_external_structural_amount_active_exact_preview_explain_are_read_only` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T032 | `authoring::trial::tests::u09_structural::families::rv003_external_representatives_and_cross_relic_closure_consume_normal_public_commands` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T033 | `authoring::trial::tests::u09_structural::rv003_external_structural_grants_queries_consumption_and_fresh_continue` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T034 | `authoring::trial::tests::u10_causal::u10_p_external_causal_join_periodic_wedge_and_fresh_continue` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T035 | `authoring::trial::tests::u10_causal::u10_p_invalid_causal_external_candidates_preserve_the_held_generation` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T036 | `authoring::trial::tests::u11_lab_continuation::world_lab_preparation_retreat_preserves_bulk_loadouts_and_normal_continue_rejects_duplicates` | F03 | 유지·보정 | 현재 실패 원인 확인 |
| T037 | `game::battle::core::commands::tests::authoring_control_cast_admission_precedes_identity_allocation_and_context_replacement_is_bounded` | X04 | 유지·보정 | 격리 전체 통과 |
| T038 | `game::behavior::tests::every_action_kind_has_one_characterized_route_scope_and_authority_contract` | X02 | 유지·보정 | 격리 전체 통과 |
| T039 | `game::data::content_pack::snapshot::tests::unix::invalid_utf8_directories_and_socket_inputs_return_errors` | E01 | 유지·환경 분리 | 환경 실패 |
| T040 | `game::data::content_pack::tests::authoring_generation_lifetime::real_trial_replacement_failures_and_last_pin_reclaim_generations` | F09 | 유지·보정 | 현재 실패 원인 확인 |
| T041 | `game::data::live_ron::tests::manifest_requires_exact_supported_schema_version` | X05 | 유지·보정 | 현재 실패 원인 확인 |
| T042 | `game::data::run_policy_data::tests::run_policy_rejects_legacy_or_unknown_fields` | F05 | 유지·보정 | 현재 실패 원인 확인 |
| T043 | `game::recruitment::tests::session_is_fixed_and_recruitment_returns_unselected_candidates_to_tail` | X07 | 유지·보정 | 현재 실패 원인 확인 |
| T044 | `game::world::admin::tests::admin_save_point_fixture_uses_authored_slot_and_produces_a_valid_checkpoint` | X09 | 유지·보정 | 현재 실패 원인 확인 |
| T045 | `game::world::application::tests::every_behavior_kind_has_exactly_one_application_owner` | X02 | 유지·보정 | 격리 전체 통과 |
| T046 | `game::world::tests::checkpoint_validation::current_v12_checkpoint_continue_then_v12_capture_is_semantically_identical` | X01 | 유지·보정 | 격리 전체 통과 |
| T047 | `game::world::tests::checkpoint_validation::current_v12_checkpoint_document_has_exact_envelope_and_checksum` | X01 | 유지·보정 | 격리 전체 통과 |
| T048 | `game::world::tests::checkpoint_validation::current_v12_checkpoint_document_has_exact_flat_semantic_field_set_and_values` | X01 | 유지·보정 | 격리 전체 통과 |
| T049 | `game::world::tests::checkpoint_validation::current_v12_checkpoint_round_trips_exact_inventory_reservation_boundary` | X01 | 유지·보정 | 격리 전체 통과 |
| T050 | `game::world::tests::checkpoint_validation::external_generations_bind_save_continue_explicit_load_and_cached_export` | X01 | 유지·보정 | 격리 전체 통과 |
| T051 | `game::world::tests::checkpoint_validation::public_application_checkpoint_port_round_trips_into_a_fresh_application` | X01 | 유지·보정 | 격리 전체 통과 |
| T052 | `game::world::tests::combat::battle_revision_overflow_rejects_mutation_without_changing_playback` | F06 | 유지·보정 | 격리 전체 통과 |
| T053 | `game::world::tests::combat::battle_setup_snapshot_live_defense_state_request_returns_battle_update_without_advancing_cursor` | X06 | 유지·보정 | 현재 실패 원인 확인 |
| T054 | `game::world::tests::combat::battle_time_trigger_fires_once_when_threshold_is_crossed` | F02 | 유지·보정 | 격리 전체 통과 |
| T055 | `game::world::tests::combat::blocking_visual_novel_execution_preflight_rejects_every_omitted_battle_action_without_mutation` | F06 | 유지·보정 | 격리 전체 통과 |
| T056 | `game::world::tests::combat::combat_map_node_fails_run_when_no_employee_can_be_deployed` | X06 | 유지·보정 | 현재 실패 원인 확인 |
| T057 | `game::world::tests::combat::durable_continue_recomputes_battle_setup_instead_of_trusting_serialized_preview_cache` | F08 | 정상 경로 유지·변조 분기 제거 | 현재 실패 원인 확인 |
| T058 | `game::world::tests::combat::elapsed_and_bucket_advance_reject_revision_overflow_without_any_state_change` | F06 | 유지·보정 | 격리 전체 통과 |
| T059 | `game::world::tests::combat::finishing_advance_rejects_future_cursor_without_consuming_terminal_state` | F02 | 유지·보정 | 격리 전체 통과 |
| T060 | `game::world::tests::combat::forced_boss_omen_defeat_immediately_fails_endless_run` | X06 | 유지·보정 | 현재 실패 원인 확인 |
| T061 | `game::world::tests::combat::live_freischutz_field_work_setup_progress_and_recovery_are_authoritative` | F06 | 유지·보정 | 격리 전체 통과 |
| T062 | `game::world::tests::combat::live_little_red_timeout_uses_battle_time_and_has_an_exhaustive_rejoin_exit` | F02 | 유지·보정 | 격리 전체 통과 |
| T063 | `game::world::tests::combat::live_ron_defense_route_playable_path_runs_to_combat_result` | X10 | 유지·보정 | 현재 실패 원인 확인 |
| T064 | `game::world::tests::combat::paused_live_battle_commits_deploy_and_withdraw_without_advancing_time` | F14 | 유지·보정 | 격리 전체 통과 |
| T065 | `game::world::tests::combat::r12_world_two_employees_shared_personal_parry_cost_result_and_fresh_restore` | X11 | 유지·보정 | 격리 전체 통과 |
| T066 | `game::world::tests::combat::resource_lifetimes::actual_admin_acquisition_recruitment_and_employee_status_preserve_owner_identity` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T067 | `game::world::tests::combat::resource_lifetimes::actual_event_defeat_commits_nonempty_resources_once_before_ack_and_restart` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T068 | `game::world::tests::combat::resource_lifetimes::actual_retreat_restores_run_resource_entry_timeline_and_attempt_identity` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T069 | `game::world::tests::combat::resource_lifetimes::actual_run_result_failure_retry_and_fresh_public_checkpoint_preserve_nonempty_values` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T070 | `game::world::tests::combat::resource_lifetimes::actual_same_copy_transfer_enhance_sale_and_dismantle_preserve_or_discard_only_that_run_key` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T071 | `game::world::tests::combat::resource_lifetimes::actual_shop_and_reward_acquisitions_create_distinct_spendable_copy_owners` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T072 | `game::world::tests::combat::resource_lifetimes::actual_withdraw_redeploy_resets_deployment_but_preserves_attempt_and_run` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T073 | `game::world::tests::combat::resource_lifetimes::cmrp_external_world_active_terminal_and_pending_keep_effective_owner_limits` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T074 | `game::world::tests::combat::resource_lifetimes::external_ron_executes_each_lifetime_with_actual_employee_and_copy_sources` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T075 | `game::world::tests::combat::resource_lifetimes::external_ron_second_cost_shortage_preserves_actual_timestamp_candidate` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T076 | `game::world::tests::combat::resource_lifetimes::extra_attack_child_cast_preserves_its_actual_causal_chain` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T077 | `game::world::tests::combat::resource_lifetimes::nonempty_run_survives_repeat_retreat_second_result_floor_and_ends_at_new_run` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T078 | `game::world::tests::combat::resource_lifetimes::released_fixed_area_keeps_delivery_and_chain_resources_after_controller_withdrawal` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T079 | `game::world::tests::combat::resource_lifetimes::released_payload_cannot_reuse_closed_activation_and_failure_keeps_committed_withdrawal` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T080 | `game::world::tests::combat::resource_lifetimes::released_projectile_completion_retains_delivery_after_motion_and_controller_end` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T081 | `game::world::tests::combat::resource_lifetimes::runtime_safety_checkpoint_restore_uses_current_host_profile_and_preserves_original` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T082 | `game::world::tests::combat::resource_lifetimes::terminal_nonempty_result_enforces_native_and_authoring_budgets_before_commit` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T083 | `game::world::tests::combat::resource_lifetimes::two_real_same_definition_copies_and_target_employee_selector_do_not_alias` | F01 | 유지·보정 | 현재 실패 원인 확인 |
| T084 | `game::world::tests::combat::track_and_rejoin_raw_live_bundle_reaches_typed_world_setup_identity` | F14 | 유지·보정 | 격리 전체 통과 |
| T085 | `game::world::tests::content_generation::fixture_b_late_briefing_preparation_failure_preserves_complete_a_authority_on_retry` | F07 | 유지·보정 | 현재 실패 원인 확인 |
| T086 | `game::world::tests::facility::active_run_checkpoint_round_trip_preserves_monotonic_loads_without_local_profile` | X01 | 유지·보정 | 격리 전체 통과 |
| T087 | `game::world::tests::facility::load_run_checkpoint_restores_visible_state_and_anchor_but_not_load_count_or_peak` | X09 | 유지·보정 | 현재 실패 원인 확인 |
| T088 | `game::world::tests::map_flow::combat_node_confirm_starts_live_battle` | X06 | 유지·보정 | 현재 실패 원인 확인 |
| T089 | `game::world::tests::node_sessions::headquarters_recruit_employee_completes_node_and_adds_roster_member` | X07 | 유지·보정 | 현재 실패 원인 확인 |
| T090 | `game::world::tests::node_sessions::headquarters_session_is_fixed_and_recruitment_moves_exactly_one_definition_to_roster` | X07 | 유지·보정 | 현재 실패 원인 확인 |
| T091 | `game::world::tests::node_sessions::live_shop_pool_separates_standard_shop_from_endless_floor_four_wandering_merchant` | F12 | 유지·보정 | 현재 실패 원인 확인 |
| T092 | `game::world::tests::snapshots_and_start::missing_run_start_visual_novel_trigger_fails_start_new_game` | F07 | 유지·보정 | 현재 실패 원인 확인 |
| T093 | `game::world::tests::snapshots_and_start::run_checkpoint_preserves_encounter_progression_graph` | F10 | 유지·보정 | 현재 실패 원인 확인 |
| T094 | `game::world::tests::snapshots_and_start::run_snapshot_exposes_current_flow_after_start` | X09 | 유지·보정 | 현재 실패 원인 확인 |
| T095 | `game::world::tests::snapshots_and_start::run_snapshot_exposes_current_node_session_after_node_entry` | X09 | 유지·보정 | 현재 실패 원인 확인 |
| T096 | `game::world::tests::snapshots_and_start::run_start_session_blocking_is_owned_only_by_the_matching_trigger` | F07 | 유지·보정 | 현재 실패 원인 확인 |
| T097 | `game::world::tests::snapshots_and_start::start_new_game_creates_starter_employee_roster_and_equipment` | X08 | 통합 후 중복 제거 | 현재 실패 원인 확인 |
| T098 | `game::world::tests::u08_event_owner::relic_event_history_survives_a_later_legal_dismantle_of_its_selected_copy` | F11 | 유지·보정 | 현재 실패 원인 확인 |
| T099 | `game::world::tests::u08_relics::durable_continue_and_savepoint_keep_independent_variant_cost_and_slot_owners` | X01 | 유지·보정 | 격리 전체 통과 |
| T100 | `game::world::tests::u10_causal::admission_cases::u10_h_actual_fact_subscriber_admission_has_exact_limits_and_atomic_retry` | F02 | 유지·보정 | 격리 전체 통과 |
| T101 | `game::world::tests::u10_causal::admission_cases::u10_h_join_final_receipt_and_wedge_commit_preserve_prior_prefix_on_refusal` | F02 | 유지·보정 | 격리 전체 통과 |
| T102 | `game::world::tests::u10_causal::ancestry_cases::u10_a_basic_attack_subscription_without_fragment_is_active_and_queries_are_pure` | F02 | 유지·보정 | 격리 전체 통과 |
| T103 | `game::world::tests::u10_causal::ancestry_cases::u10_a_derived_delayed_damage_keeps_the_visited_identity` | F02 | 유지·보정 | 격리 전체 통과 |
| T104 | `game::world::tests::u10_causal::ancestry_cases::u10_a_equal_definition_copies_and_distinct_rule_ids_are_independent_visits` | F02 | 유지·보정 | 격리 전체 통과 |
| T105 | `game::world::tests::u10_causal::ancestry_cases::u10_a_fact_selector_and_same_effect_conditions_cannot_mix_segment_witnesses` | F02 | 유지·보정 | 격리 전체 통과 |
| T106 | `game::world::tests::u10_causal::ancestry_cases::u10_a_recompiled_owner_and_plan_do_not_reset_same_copy_rule_visits` | F02 | 유지·보정 | 격리 전체 통과 |
| T107 | `game::world::tests::u10_causal::ancestry_cases::u10_a_same_copy_effect_revisit_stops_but_next_independent_cast_runs` | F02 | 유지·보정 | 격리 전체 통과 |
| T108 | `game::world::tests::u10_causal::ancestry_cases::u10_a_same_copy_rule_survives_form_and_fingerprint_replacement` | F02 | 유지·보정 | 격리 전체 통과 |
| T109 | `game::world::tests::u10_causal::ancestry_cases::u10_a_same_copy_rule_visit_survives_rebinding_to_another_employee` | F02 | 유지·보정 | 격리 전체 통과 |
| T110 | `game::world::tests::u10_causal::ancestry_cases::u10_a_static_dormancy_and_event_conditions_preserve_no_fragment_active_consumers` | F02 | 유지·보정 | 격리 전체 통과 |
| T111 | `game::world::tests::u10_causal::ancestry_cases::u10_e_each_positive_damage_fact_can_subscribe_even_within_one_strike` | F02 | 유지·보정 | 격리 전체 통과 |
| T112 | `game::world::tests::u10_causal::ancestry_cases::u10_e_full_absorption_and_zero_damage_do_not_emit_hp_loss_subscriptions` | F02 | 유지·보정 | 격리 전체 통과 |
| T113 | `game::world::tests::u10_causal::ancestry_cases::u10_e_healing_and_health_cost_are_not_damage_hp_loss_events` | F02 | 유지·보정 | 격리 전체 통과 |
| T114 | `game::world::tests::u10_causal::ancestry_cases::u10_e_no_fragment_dead_owner_death_subscriber_uses_actual_killer_once` | F02 | 유지·보정 | 격리 전체 통과 |
| T115 | `game::world::tests::u10_causal::ancestry_cases::u10_e_overkill_commits_one_actual_death_and_keeps_damage_and_kill_facts_distinct` | F02 | 유지·보정 | 격리 전체 통과 |
| T116 | `game::world::tests::u10_causal::ancestry_cases::u10_e_taken_delayed_child_preserves_public_original_source_and_actual_killer_after_log_eviction` | F02 | 유지·보정 | 격리 전체 통과 |
| T117 | `game::world::tests::u10_causal::join_cases::u10_b_authored_then_branch_and_join_continuation_have_distinct_program_and_amount_identity` | F02 | 유지·보정 | 격리 전체 통과 |
| T118 | `game::world::tests::u10_causal::join_cases::u10_b_cancelled_incomplete_participant_never_commits_joined_payload` | F02 | 유지·보정 | 격리 전체 통과 |
| T119 | `game::world::tests::u10_causal::join_cases::u10_b_join_excludes_same_clock_nonparticipant_and_clones_waiting_state` | F02 | 유지·보정 | 격리 전체 통과 |
| T120 | `game::world::tests::u10_causal::join_cases::u10_b_local_bucket_descendants_hold_join_until_their_actual_visits_complete` | F02 | 유지·보정 | 격리 전체 통과 |
| T121 | `game::world::tests::u10_causal::join_cases::u10_b_real_fork_siblings_inherit_a_but_each_may_visit_b_once` | F02 | 유지·보정 | 격리 전체 통과 |
| T122 | `game::world::tests::u10_causal::join_cases::u10_b_real_join_waits_for_descendants_then_uses_participant_union` | F02 | 유지·보정 | 격리 전체 통과 |
| T123 | `game::world::tests::u10_causal::periodic_lifetime_cases::u10_c_derived_dot_ticks_keep_ancestry_and_do_not_refresh_their_own_source` | F02 | 유지·보정 | 격리 전체 통과 |
| T124 | `game::world::tests::u10_causal::periodic_lifetime_cases::u10_c_detached_periodic_keeps_accepted_child_program_and_numeric_owner_only` | F02 | 유지·보정 | 격리 전체 통과 |
| T125 | `game::world::tests::u10_causal::periodic_lifetime_cases::u10_c_periodic_occurrences_are_independent_roots_without_active_fragment` | F02 | 유지·보정 | 격리 전체 통과 |
| T126 | `game::world::tests::u10_causal::periodic_lifetime_cases::u10_d_released_subscriber_projectile_keeps_ancestry_after_action_end` | F02 | 유지·보정 | 격리 전체 통과 |
| T127 | `game::world::tests::u10_causal::periodic_lifetime_cases::u10_d_trace_modes_do_not_change_delayed_causal_damage_or_clone_progress` | F02 | 유지·보정 | 격리 전체 통과 |
| T128 | `game::world::tests::u10_causal::resonance_cases::u10_e_queued_and_periodic_positive_resonance_use_the_existing_gate_without_banking` | F02 | 유지·보정 | 격리 전체 통과 |
| T129 | `game::world::tests::u10_causal::wedge_cases::u10_g_invulnerable_first_strike_does_not_consume_wedge_count` | F02 | 유지·보정 | 격리 전체 통과 |
| T130 | `game::world::tests::u10_causal::wedge_cases::u10_g_other_relic_child_damage_does_not_advance_original_cast_wedge` | F02 | 유지·보정 | 격리 전체 통과 |
| T131 | `game::world::tests::u10_causal::wedge_cases::u10_g_same_strike_segments_share_candidate_but_same_time_strikes_do_not` | F02 | 유지·보정 | 격리 전체 통과 |
| T132 | `game::world::tests::u10_causal::wedge_cases::u10_g_spatial_deliveries_within_one_cast_count_each_actual_target_separately` | F02 | 유지·보정 | 격리 전체 통과 |
| T133 | `game::world::tests::u10_causal::wedge_cases::u10_g_structural_guards_and_rounding_precede_numeric_and_wedge_composition` | F02 | 유지·보정 | 격리 전체 통과 |
| T134 | `game::world::tests::u10_causal::wedge_cases::u10_g_two_copy_percent_contributions_add_before_damage_composition` | F02 | 유지·보정 | 격리 전체 통과 |
| T135 | `game::world::tests::u10_causal::wedge_cases::u10_g_wedge_first_strike_counts_and_new_cast_resets` | F02 | 유지·보정 | 격리 전체 통과 |
| T136 | `game::world::tests::u10_causal::wedge_cases::u10_g_wedge_percent_adds_to_existing_numeric_class_after_flat` | F02 | 유지·보정 | 격리 전체 통과 |
| T137 | `game::world::tests::u10_causal::wedge_cases::u10_g_wedge_zero_full_and_partial_absorption_use_actual_positive_loss` | F02 | 유지·보정 | 격리 전체 통과 |
| T138 | `game::world::tests::u10_causal::wedge_cases::u10_g_withdrawal_cancels_unreleased_steps_but_released_projectiles_keep_wedge` | F02 | 유지·보정 | 격리 전체 통과 |
