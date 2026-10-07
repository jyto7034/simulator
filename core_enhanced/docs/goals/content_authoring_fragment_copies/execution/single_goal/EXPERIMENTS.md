# R07 조사·명령·검증 기록

2026-09-29: 신규 Topic 존재 여부 확인 후 WORKING_POLICY/P2G_POLICY_AND_REVIEW/P2G30/40/50/PLAN_HANDOFF 작성. P2G40 첫 식별자 오류 실패는 candidate 원문과 실패 코드 보존, 실제 generator와 read-only validator 재실행 성공. P2G50 quality contract2: 6 elements/1 Phase/2 execution nodes/1 Goal/12 integration rows/10 dimensions, failures empty. 구현 PASS 아님.

R06 경계 대기 중 실제 skill_fragment, data/skill_fragment_data, grant_state/reward/uuid_manager, maintenance/helpers/employee/snapshot, active_run, stat_pipeline/player_spawns, runtime source/resource owner 경로 재확인. 공통 progress 제거는 reward, combat death recovery, checkpoint, public DTO와 동시에 해야 한다. R07 product 수정 미착수.

### GEX-10 schema cutover 001

R06 경계 manifest `0b6938d77cc6cded598b2b01e51d41bf7e651dd66e8acf2f695a1cf3b5fd44f1` 인수 완료. OwnedFragment UUID map/선택·해금·owner와 legacy module, checked FRAGCOPY namespace, ActiveSkill.forms/recipe, base/form/recipe validation, catalog schema4와 공식2개 active 정의를 전환하기 시작했다. Stat producer에 owner와 source capsule을 연결하고 Certified input의 shared progress를 exact copy 입력으로 교체했다.

`schema_check_001`은 전환 중 lib compile 진단이다. 기존 command/DTO/save/recovery 경로의 제거된 progress/definition-loadout 참조가 남아 실패하는 중간 상태이며 완료 주장이 아니다. 기존 skill_fragment 단위시험은 baseline archive에 보존했고, 공통 강화 의미의 시험은 새 F01–F12 사본·형태·rollback 시험으로 전환한다. 다른 테스트를 skip/cfg 제거해 성공시키지 않는다.

### GEX-10 owner/consumer cutover 002–003

`schema_check_002` exit101: 9 errors remaining in stale result payload encoding and an overly broad profile-initializer edit. Corrected only those locations; no test skip. Replaced world maintenance operations with staged five-root transactions, paired exact-copy equip/unequip/transfer, separate unlock/select and checked aggregated explicit recipe consumption/yields. Inventory/employee/maintenance views now enumerate exact copies and expose independent form eligibility; generic enhancement disabled as removed. Checkpoint graph validation uses paired bindings and selected-form compatibility; death recovery releases exact copies without deleting unlocks.

Added immutable fragment source capsule to runtime action sources and runtime units, exact Arc source binding, and R05 FragmentCopy/SourceFragment owner routing through existing lifetime store. Extended actual owner reconciliation, preserving UUID through transfer and removing a dismantled copy's state through the central reconciliation path. `schema_check_003` records lib compilation of this integrated intermediate source; tests/data fixture migration and V8 conversion remain in progress.

### Migration regression 001

`schema_check_005`: all targets + test-fixtures check exit0. `migration_lib_001`: 기본 병렬 1510 tests, 1494 passed/16 failed/0 ignored. 실패 원인: 새 action/result 개수 기대값, R06 공개 scenario의 옛 grant pointer, schema/version/field-path 기대값, V7 변환의 HashMap 출력 순서, Maintenance 진입 결과가 session 설치 전에 form eligibility를 조회하던 순서, 기존 placeholder에 분쇄 recipe를 가정하던 sweep 시험. raw V7는 변조하지 않았다. V8 writer를 canonical JSON 출력으로 교정하여 동일 변환의 bytes도 결정적으로 만든다. Maintenance는 후보 transaction 내부에 session을 먼저 설치하고 preview를 만든다. 새 F01–06/11/12 integration tests를 추가한다.

`r07_copies_001`은 새 시험의 EquipmentMaterialCost import 누락으로 compile 실패; import 교정. `r07_copies_002` 실제7 tests passed: 정상 grant/UUID rollback, copy 독립·occupied/이전 rollback, form cost/locked/no-op/query, confirmed 분쇄/산출 실패 rollback, 사망 회수 반복, copy cap/duplicate/UUID overflow, immutable V7 bytes의 명시 변환. `r07_public_001`은 ApplicationCheckpoint private field 사용 compile 실패; 새 공개 시험을 public constructor/getter로 교정했다. runtime amount query와 mandatory trace에 fragment capsule을 투영하고 retained/clone/work 계산도 추가했다.


`r07_public_002–004`의 compile/fixture 실패 원자료를 보존했다. RON serializer의 i128 미지원은 fixture에서 원문 RON 추가로 해결했고, map slot을 기존 combat slot 앞에 추가해 UUID-derived 배치가 바뀌는 시험 입력 문제는 기존 slot 순서를 보존해 교정했다. `r07_public_005` 실제 공개 시험1개 PASS: Snapshot/Live 각각 trace off/on, 정상 재료 지급/개화/선택, 새 subprocess save/restore. `r07_acceptance_001` 12개 중10 PASS/2 fail: V7 raw에 V8 typed checksum helper를 잘못 적용, bounded latest-cost DTO에 모든6자원 이름이 남는다고 가정한 시험 oracle 오류. 원본 checksum을 canonical raw에서 계산하고 실제 atomic cost mutation 수와 typed resource store를 확인하도록 수정했다. 추가로 형태별 weapon 이전 rollback, retained source payload exact/short/retry 시험은 이미 PASS.

`r07_acceptance_002` 10/13 PASS, 추가 oracle3개 교정: resource 단위를 Damage로 narrowing한 시험을 typed snapshot frame 조회로 교정; awakened old state에 required available 플래그 추가; terminal battle은 live resource view를 제공하지 않으므로 bounded facts의 실제6 typed cost + SP cost mutation7과 latest Chain source를 확인. `r07_acceptance_003` 12/13 뒤 최종 oracle 교정. `r07_acceptance_004` 14/14 PASS, ignored0. 공개 query의 work/metadata/result exact/short/retry, pre-cancel와 `trial_step_record` 중간 취소 후 receipt/work/state 재시도 포함.
마지막 Run resource exact owner 이전/해체 rollback 시험을 추가하고 cargo fmt 실행. `integrated_all_001`부터 기본 병렬 전체 검증을 실행하며 동일 source identity의 feature/check/event/release/clippy/fmt를 순차 수행한다.

`integrated_all_001`: 1581 passed/0 failed/0 ignored,167.1s. `integrated_feature_001`:1581/0/0,236.7s. 동일 source367개/digest d1df7e73f94be9ef3c6231cd42c7b52d4708e6b971f23a0f1719608d02218b15, 기본 병렬/RUST_TEST_THREADS 미지정. R07 Run owner transfer/confirmed dismantle/뒤 grant 실패 rollback 포함15개 r07 selector가 전체 실행에서 통과했다.

PC integrated001 측정 자체 exit0/31success/5typed refusals. 비교는 final_facts192행 차이 때문에 exit1. PC_NULL_FIELD_DIAGNOSIS.json은 모든 차이가 새 absent fragment:null뿐임을 전체576rows에서 확인한다. 기대값/비교기를 약화하지 않고 RuntimeFactSource와 ApplicationWitness의 optional fragment None을 serialize 생략해 기존 무파편 출력 호환을 복구한다. 실제 Some 출처는 그대로 노출한다. 최종 검증을002로 재실행하고001 원자료/요약을 보존한다.002 PC 원자료는 같은 측정 코드에서 /tmp 새 경로로 기록한 bytes를 evidence로 복사해 다수 작은 파일 write의 mounted-drive 비용을 피하며 계산/샘플/한도를 바꾸지 않는다.

`pc_integrated_002`: exit0, 실제576rows/36conditions/31success/5typed refusals. `PC_SUMMARY_002.json`에서 모든 gameplay/state/오류코드 차이0. requested-cost detail80행은 source 구조의 계량 변화로 보존하며 비교 제외 필드는 기존 측정 계약 그대로다. 첫 비교 실패의 terminal exit1은 PC_COMPARISON_001_FAILURE.json에도 기록했다. 최종 후보 source digest56cc75e37a8cd3356ce48fb90517580fd9bbff41a5da0054f19cbfc51d2bf436.

`r05_integrated_001`의 raw exit101은 원래 concurrent peak guard assertion이다. `R05_SUMMARY.json` 실제 분석: peak9230083/cap9030188, 단독199895 bytes 초과. event_heavy peak258006/276174, allocation operations43398/70671; concurrent operations370928/389240, retained4196928/4196928.5 allocation samples/21 latency samples, repeatability/parity true, 기타 hard guard0. 합의된 concurrent-peak-only 경고만 적용했다. 이후 same-source integrated002 필수 명령과 authoring/value_expr 집중명령, --list, actual release CLI를 모두 수행한다.

공개 CLI 첫 explain request가 Skill에 boss 전용 compiled_graph query를 요청해 cli_r06_explain_001 exit1/authoring_query_unsupported로 거부됐다. 실행 harness 오류이며 제품 거부는 정상이다. 원래 request/explain log/실패 harness를 보존하고 definition query의 explain_002.json 및 새002 영수증으로 재실행한다. 이미 성공한 export/validate/describe/집중 시험은 정확 command와 원래 영수증을 확인해 재사용하며 입력 디렉터리를 덮어쓰지 않는다. 제품 source는 바뀌지 않았다.

CLI simulate의 전체 stdout은 각 log에 약1.5MiB 이상이며 원문 그대로 보존한다. 반복 console 재출력으로 상위 세션 출력 버퍼가 지연되지 않도록 이후 cli_* 래퍼는 시작 시 log 경로와256KiB 단위 byte 진행 및20초 heartbeat를 출력한다. 로그 저장·비교·receipt·source identity는 동일하고 원래 래퍼도 run_full_stdout_001.py로 보존했다. 결과를 완료까지 숨기는 방식이 아니다.

동일 source의 매 명령 archive 재압축을 줄이기 위해 wrapper source cache를 추가했다. 최초 donor는 before==after==current 영수증과 archive 모든 member/bytes/hash를 대조하고, 각 재사용 시 gzip SHA를 검사한 뒤 새 immutable 이름으로 hard link한다. 매 명령 source before/after 검사는 계속 수행한다. 원래 wrapper는 run_full_archive_002.py에 보존했다. 측정/제품 로직·한도·샘플은 변하지 않는다.

R07 release describe 확인에서 `value_expression_explanation.authored_expression_publication`의 R06 이전 staged-only 문구가 남아 있음을 실제로 발견했다. 실제 공개 expression 허용과 모순이므로 query.rs의 공개 지원 문구와 tests.rs의 해당 계약 기대값을 함께 교정한다. static explain 자체가 runtime 값을 추정하지 않는 기존 문구는 유지한다. 두 파일의 문자열만 바뀌지만 최종 검증 source를 엄격히 일치시키기 위해003 필수 전체/feature/집중/build/PC/R05/CLI를 실행한다. 각 release describe도 새 문구를 명시 assert한다.002 결과와CLI001 결과는 덮어쓰지 않는다.

검증 harness 순회 교정: 이미 source membership에서 제외하는 tools/target 빌드 디렉터리를 rglob가 내부까지 순회하여 명령 사이 지연이 발생했다. os.walk에서 해당 디렉터리를 미리 제외했으며 기존 파일 필터는 유지했다. 최종 all003의367개 경로·크기·SHA 및 통합 digest와 완전히 일치함을 SOURCE_WALK_PRUNING.json에 기록했다. 이전 harness는 run_before_directory_pruning_003.py에 보존했다. 진행 중인 wrapper는 이미 로드한 이전 코드로 종료하고 후속 명령부터 같은 입력의 빠른 순회가 적용된다. 제품 소스·시험·데이터 변경은 없다.

최종 동일 source 검증 {'integrated_all_003': 1581, 'integrated_feature_003': 1581}. Current digest `714195327ec0df7facc60d1aafe721e6e26c1296494b39cff593232c45365e6f`; check/event/release/clippy/fmt 및 공개CLI 성공. PC36 실제576rows/31success/5typed refusals, gameplay/completion/error-code 차이0. R05 warning_concurrent_peak_only, concurrent peak 9214469 bytes, 단독 초과 184281 bytes; 다른hard실패0. 각 명령 before/after/current source 완전 일치. independent audit 미수행. COVERAGE/R06_IMPACT_COVERAGE/VALIDATION_CURRENTNESS에 실제 selector를 검증해 연결했다.
