# U06 실행·검증

P2G40 구조와 P2G50 quality2 표준검증 PASS, 의미검토/P2G60 인계완료.
GEX00 실제 dirty baseline archive 추출hash확인. 초기U05 baseline은현재성재사용; U06 productmutation/새acceptance실행없음.

### BASIC early validation

- `../../evidence/basic_compile_001.json`: wrong cargo package selector `core_enhanced`; no Rust compile/test ran. Tool invocation error, preserved raw.
- `../../evidence/basic_compile_002.json`: corrected `game_core`, cargo check --tests PASS on 11 changed product files including new basic_heal module. Source unchanged during check.
- `../../evidence/basic_heal_focused_001.json`: FAIL at compile, 7 errors: added native admission called wrong control namespace; six test harness API/access/comparison mistakes. No behavior tests ran. Corrected native admission with checked physical extent, public fact view, real BuffId/RON constructor and serialized log equality; no oracle removed.
- `basic_heal_focused_002` running after correction/format. Seven tests cover strict action/delivery, ratio/distance/ID/self targeting, fixed target death, release-time area center/ally filter, actual capped HP with no damage metadata, clone + finite source settlement, transaction abort/epoch cancel, and ordering. Completion remains pending until actual receipt.
- `../../evidence/basic_heal_focused_002.json`: PASS 7/7 (0 skipped). `basic_queue_risk_001`: PASS 4/4. `basic_clone_risk_001`: PASS 46/46 native budget/clone/retained/rollback tests, including U05 use-state failure/retry and existing timestamp range-row reuse. Every receipt has unchanged source identity. These are early subsystem checks, not U06 completion.


### Ally 연결 및 중지 시 증거 현재성 — 2026-10-05

- `../../evidence/ally_compile_001.json`: cargo check 실패. shared skill reach helper의 private 가시성 오류1개. Core 내부 가시성으로 수정.
- `../../evidence/ally_focused_001.json`: 테스트 enum 이름 Air/Airborne 오류1개. 행동 시험 미실행, 원래 입력/oracle 유지 후 수정.
- `../../evidence/ally_focused_002.json`: `cargo test -p game_core --lib basic_heal_tests` PASS9/9, 0 skipped.
  기본7건과 ally 선택/사망 고정/범위/공중2건. source_unchanged=true이며 중지 시 제품369개와 exact 일치한다.
- 제품 sha256: `0e070ffb8a3b223285b9aae2f700b91b38cf14847b052d7572ae4090124873c2`.
- 앞선 queue4/native46은 Ally 및 enum pair 추가 이전 소스에 묶인 증거다. 최종 전체 PASS로 확대하지 않는다.
- 사용자 중지 후 새 제품 시험은 실행하지 않았다. 통합/feature/host/public/native 최종 검증과 독립 감사는 남아 있다.

### 재개 BASIC 잔여
- basic_residual_001: 시험 fixture의 ron::from_str 경로 오류2개로 compile 실패, 동작 시험 미실행. 기존 ron::de::from_str API로 교정하며 원본 로그/receipt를 보존한다.
- basic_residual_002: PASS11/11, 0 ignored, source unchanged.
- combat_modifiers_compile_001: 새 CombatModifiers library는 컴파일됐지만 이전 AllySingle의 integration test ron_loading exhaustive 소비자 누락1개로 실패. 선택 이름 ally_single 분기를 추가해 기존 oracle을 유지했다.
- combat_modifiers_focused_001: PASS6/6, public sustained 감소/흡혈·반열린만료·clone/rollback/strict 입력·Heal overlay 접수 거절과 기존 ally2건. source unchanged.
- starter_compile_001: UnitCombatProfile에 없는 Default 호출1개로 compile 실패. 기존 for_employee 생성자와 실제 정의의 stats/speed를 사용하도록 교정했다. 행동 검증 미실행.
- starter_data_load_001: 정상 disk RON 로더/통합 직원정의 검증 PASS1/1. source unchanged. strict6스킬/6파편/5무기와12세트 참조가 실제 로더를 통과했다.
- ron_loading의 catalog 고정집합은 승인된 새5무기(30→35: Gun+1/Shield+1/Staff+2/Sword+1),6스킬과12세트로 명시 갱신한다. 원래 무기 수치/assertion·기존 skill선택 전부 유지; 모든 starter 동일검 정책만 새6역할로 대체한다. 이전 테스트 원문은 U06 baseline archive에 보존했다.
- starter_grant_focused_001: 신규 시험의 Employee import 누락으로 compile 실패1개. 실제 기존 타입 경로를 import하여 교정; 행동 실행 전 실패를 보존한다.
- starter_grant_focused_002: PASS2/2, 정상 여섯 starter를 두 원정으로 모두 선택하여 장비/파편 사본·양방향 binding과 fresh Continue 동일성을 검증했다. 실제 채용과 지급 완료 후 주입 실패의 전체 상태 복원·정상 재시도도 통과. source unchanged.
- starter_native_risk_001: 45PASS/1FAIL. terminal_nonempty_result_enforces_native_and_authoring_budgets_before_commit의 resource fixture가 공식 starter Heal 전환으로 OnAttack 자원 소비 입력을 잃어 종료 후 active None에 도달했다. fixture의 원래6검+baseline/빈HQ/스캐너를 명시 복원했으며 native한도/oracle은 수정하지 않았다. 공식 새 세트 시험과 구분하여 재검증한다.
- starter_native_risk_002: PASS46/46, 0 ignored. 자원 fixture 입력복원으로 원래 native/rollback 한도와 수치를 유지했다. 새 공용 생산코드의 현재성 source unchanged.
- 콘텐츠 목록의 기존 첫 원소를 읽는 fixture를 보존하기 위해 새 무기/파편/스킬은 각 기존 목록 끝에 추가했다. 기존 항목 순서/내용과 새 정의 의미를 바꾸지 않는다. 최종 live catalog 검사에서 재검증한다.

- tactics_compile_001: cargo check --tests PASS, source unchanged. timer/상태/query/행동 gate 전체 컴파일.
- tactics_focused_001: 11PASS/1FAIL, 새 타이머/CC/독립clone/Rest주기2건 PASS. 기존 swap 시험의 DefenseRequired 기대1개가 최신 Attack허용 정책과 충돌해 해당 사전검사를 성공 기대로 갱신했다. 나머지 양쪽 원자성/CC/점유 oracle은 유지한다.

- tactics_focused_002: PASS12/12, source unchanged. 공격태세 이동/교대 허용과 stale/CC/clone/Rest 원자성 검증.
- field_work_focused_001: 9PASS/1FAIL. U05 공명 시험의 이동1000ms 후 수동Attack이 최신 강제Defense1500ms에 걸렸다. 공명 수치/시점은 유지하고3000ms 거절 및3500ms 자동Attack을 검증하도록 입력을 현행화한다.
- tactics_queue_001: 5PASS/1FAIL. 모든 variant clone/예산 검증은 PASS; fixture cardinality 상수에 BasicHealResolve+ForcedDefenseEnd 총2개 추가를 반영하지 않아23대22로 실패. 상수23으로 교정하며 실제23쌍의 total-order 검증을 유지한다.

- field_work_focused_002 PASS10/10 및 tactics_queue_002 PASS6/6, source unchanged. 공명 기존 수치/시점 유지와 신규23variant 전체 비교·clone·한도 검증.
- block_focused_001: movement57PASS/2FAIL. 새 fast fixture가 stats만 갱신하고 이미 만들어진 body.move_speed=1을 그대로 둬 segment가 접촉까지 도달하지 않았다. 실제 runtime 이동속도 body를10으로 명시하여 입력을 교정한다. 기존 movement/native 전체와 새 basic range 예외는 PASS.

- block_focused_002: PASS59/59, source unchanged. 실제 Rapier output의4방향 fast접촉/유지·현재위치/용량우선·기본 적대 범위 예외, 기존 movement/native 회귀 통과. 기존 외부반경 drift유지 시험은 승인된 반경이탈해제로 기대와 이름을 명시변경한다.

- time_compile_001: Core와 TIME schema/state/query/route 컴파일 PASS, host test1개가 STARTER optional field의 Rust literal 누락으로 실패. host legacy loadout은 active_skill_fragment_id=None을 명시하여 기존 검+baseline 입력을 보존한다. 새6command host 파싱 fixture와 정상 중첩/설정 테스트를 추가해 재검증한다.

- time_focused_001: shared player-unit fixture 추출 후 기존 시험의 deploy_position 지역변수1개 누락으로 compile 실패. 고정 배치된 caster의 실제 tile을 읽어 동일입력으로 복원; 행동시험 미실행.
- time_focused_002: PASS16/16, source unchanged. BASIC/STARTER/timer/BLOCK과 새3TIME 정상명령 테스트: general/deploy/skill 중첩, 단일restore, 실패·futurecursor fullrollback, 독립clone, 설정입력gate, 중복열기/hosttick불변, disconnectclose정지유지, Pause/0.5hosttick의1499/1500 timer 경계.

- query_compile_001: Core/Host --tests 전체 컴파일 PASS, source unchanged. 공통 interaction,7개 새 공개command, 기본행동/관계/query 및 tactical preview producer/consumer가 같은트리에서 컴파일됐다.

- query_focused_001: 16PASS/1FAIL. 새 출전불가 행 시험이 physical_damage=100(사망)을 중상으로 기대했다. 기존 MAX_PHYSICAL_DAMAGE-1 경계를 사용하도록 시험 입력만 교정하며 condition>TP 우선 oracle을 유지한다.
- query_transport_001 PASS1/1(51개 result 전체 변환 계약), host_query_parse_001 PASS1/1(기존+새7command 및 swap/relocate payload). 모두 source unchanged.

- starter_boundary_001 PASS18/18, source unchanged. 실제 채용7회로 roster10과 정상6역할 사본을 검증하고11번째 채용의 full-state 불변 거절을 확인했다. 중상입력 교정 후 query 조건우선 시험도 PASS.

- deploy_boundary_compile_001: 신규 시험2곳에서 private checkpoint helper를 호출해 compile 실패했다. 기존 공개 battle-update projection으로 교정하고 가시성은 확장하지 않았다. 행동시험 미실행.

- starter_public_001: 기존3시험 PASS(로스터10/동시8/철수슬롯 포함), 새6active 시험은 혼합 적대 돌파의 유효 적이 없는 입력에서 NoValidTarget으로 실패했다. 실제 첫 wave 적을 범위 안에 배치하는 명시 fixture를 추가하고 두 manual heal의 필수 targeting focus를 정상 명령으로 연다. 기존 적대 usefulness gate는 유지한다.
- starter_public_002: 교정 script의 형식 match assertion 실패 뒤 동일입력 재실행으로 동일3PASS/1FAIL. 새 성공 근거가 아니며 원본 보존. 003에서 교정된 입력을 검증한다.

- starter_public_003 PASS4/4, source unchanged. 정상6starter 실제 RON/사본/배치를 사용한 CoreApplication 발동에서 공격력·주기/흡혈/피해감소/보호막/단일60·영역25 치유를 검증했다. 공명100/HP10 및 돌파용 첫wave 적 위치는 명시 시험 입력이다. cap시험은 정상채용10→11거절, 동시8→9거절 전체 projection불변, 철수 후 새직원 배치로 슬롯반환까지 PASS.
- live_ron 선행 두 negative fixture는 기존baseline[0]과 첫직원 검을 가정했다. missingbaseline ID는 빈vec에 명시추가하고 무기제거는 현재 초기무기를 제거하여 같은 missing-reference/no-weapon oracle을 유지한다. 새 active missing/wrong-kind/incompatible/baseline오용 negative를 별도로 추가했다.

- starter_host_compile_001 PASS Core+Host --tests. integrated_core_default_001은1694PASS/37FAIL, integrated_host_default_001은 example checkpoint interaction 누락으로 compile 실패. 양쪽 source unchanged, 원본 보존.
- 전체 실패 분류/교정: 새7command의 기존59 cardinality를66으로 갱신; preview query를 기존 corrosion read-only allowlist 시험에도 추가; Host example 새필드 보완; AllySingle 포함 shared 진단 문자열 갱신.
- 앞서 RON 목록 이동 도구가 기존 첫 skill unknown_distortion_strike까지 끝으로 옮긴 오류를 발견했다. 초기 archive와 모든 기존26개 skill ID 순서를 직접 대조해 정확히 복원하고 새6개만 끝에 남겼다. 따라서 first-skill 수치60 및 projectile provenance index 기존 oracle은 변경하지 않는다.
- 과거 수치/채널/Continue public 시험의 검+baseline/빈active 입력은 명시 legacy fixture로 보존한다. official 로더/새6역할 시험은 원본 현재 콘텐츠를 계속 사용한다. 고정 v7 inventory 비교는 신규 명시 default action=Damage만 original projection에 보충하며 다른값은 전부 비교한다.
- 채용 branch 시험은 기존 인벤토리 전체 값 불변+정확한 새장비수/소유권을 검증하도록 확장한다. 이는 공식 지급변경이다. 출전불가 직원 행은 숨김 기대에서 typed불가 사유·기존lock보존 기대로 바꾸고 이동 DefenseRequired는 현재 TP/허용 기대를 반영한다.

- integrated_correction_compile_001 PASS Core/Host all-targets. integrated_host_default_002 PASS92/92, 모든example 컴파일 및 새 실제 actor selection/settings/disconnect/reconnect 정상 명령·revision·Pause복원 PASS. integrated_core_default_002는1730PASS/1FAIL: trace parity 시험 한곳이 직접 embedded starter를 사용해 ground배치 실패. 같은 명시 legacy fixture로 입력을 복원하고 trace/oracle 비교는 그대로 유지한다. source unchanged.

- integrated_core_default_003: 1730PASS/1FAIL. 신규 공개 기본치유 확인에서 영역 staff의 2200ms 주기+250ms resolve 이전에 판정했다. 공개 시계 진행을2500ms로 교정하고 실제 HP증가 oracle을 유지한다. 기존 trace parity 교정은 PASS. source unchanged.

- starter_basic_public_001 PASS1/1: six normal active cases plus both authored basic staff heals through CoreApplication elapsed/reconnect. Area release checked after2200+250ms. source unchanged. integrated_matrix.py004 begins final default/fixtures/observation/host/check set on that source.

- integrated_fmt004 PASS. integrated_core_default004: library1731PASS, CLI4/showcase1/item4 PASS; catalog2PASS/2FAIL. Exact catalog manifest lacked six new active references and delivery membership (two spatial/four Instant). Added only those entries; existing26 skills and assertions unchanged. Failure raw retained. This manifest is included solely by live_skill_catalog_audit integration target; previous successful library results are non-impacting, remaining default integration targets rerun as integrated_core_integration005; fixtures/observation full sets remain required.

- integrated_core_integration005: catalog4PASS, RON24PASS/1FAIL. Legacy selection cardinality26 and exact selector subsets omitted the six additions although target mapping was updated. Set32, add newSelf4/PlayerUnit1/PlayerTile1, preserve originalAutomatic20 and all original IDs. No product behavior change.

- integrated_core_integration006 PASS57/57. Default library1731 PASS from004 is reused only for exact unchanged library inputs; two integration-only deltas and replacement57 are documented in DEFAULT_LIBRARY_REUSE_001.json. Full fixture/observation/host/check matrix006 now begins.

- integrated_core_fixtures006 compile failed before any tests: rustc-LLVM IO failure Cannot allocate memory, while another unrelated Python workload occupied memory. Source unchanged; no product assertion failure. Retry007 uses explicit CARGO_PROFILE_TEST_DEBUG=0/CARGO_PROFILE_DEV_DEBUG=0/CARGO_INCREMENTAL=0 and jobs1. Product source, features, runtime safety limits and test oracles unchanged; unrelated processes untouched.

- integrated_core_fixtures007 PASS1788/1788; integrated_fmt007 PASS. integrated_core_observation007 intentionally interrupted during compilation (SIGINT, rawexit-2) for discovered same-version canonical checksum risk: newly serialized default weapon action=Damage can invalidate old strictV11 checksums. Add original-format checksum/Continue regression before correction; no completed observation claim.

- save_damage_compat_repro_001 intentionally reproduced one failure: a supportedV11 document with implicit Damage weapon action, original canonical checksum refreshed before reading, is refused with SHA-256 mismatch. Correct serialization by omitting only the neutral Damage default in BasicAttackDef/AuthoredBasicAttackDef/WeaponCombatProfile; Heal remains explicit. Remove earlier V7 inventory projection compensation and restore exact original-value assertion. Old default-library reuse and PCprepare001 are now historical because this is a real producer change; final matrix will be rebound. Added actual automatic-heal control/fullHP and same-timestamp lethal-damage queue tests to supplement prior shared-gate and ordering evidence.

- save_heal_edges_001 PASS24/24 (runtime-observation): supported implicit-Damage V11 checksum/Continue/re-export equality; actual basicHeal fullHP/Defense before+after/Silence before+after/HardCC before+after and same-time poison lethal→Heal no revival; prior allU06 selectors including normal6sets PASS. Source unchanged. Final integrated008 restarts on corrected serializers.

- `integrated_core_default008`: 최종 Damage implicit serialization 수정 기준 source 불변, 1,734 lib + 57 integration = 1,791 PASS. `integrated_fmt008` PASS. fixture/observation/host/check/heap 및 공개 호환성 consumer 검증은 진행 중이며 아직 완료 근거로 취급하지 않는다.

- `integrated_core_fixtures008` 1,791 PASS, `integrated_core_observation008` 1,800 PASS, 각 source 불변. 20개 acceptance의 Core selector fragment가 모두 실제 ok 테스트에 연결됨을 확인했다. Host 및 public/performance gate는 별도로 진행한다.
- `compat_pc_prepare_003` PASS: 측정 전 실제 compiler layout만 출력하는 보조 관측을 추가한 새 격리 사본. workload, warmup, samples, 원본 비직원 fixture0/100/1000, raw 행과 수치 상한은 그대로다. 001/002 격리 사본·원본 receipt는 보존하며 최종 PC는003으로 측정한다.

- `public_starter_schema_001` raw exit1 보존: 8개 중 positive 및 6개 부정 입력은 기대대로 통과했으며 `flat:-1`도 reader가 실제로 거절했으나 harness가 `u32` 문구를 잘못 기대했다. 실제 JSON은 ron_parse, items[91].weapon_profile.action.Heal.flat, ExpectedInteger. 제품/입력/한계를 바꾸지 않고 정확한 필드 경로·파서 코드와 Expected integer를 함께 검증하도록 고쳐002로 재실행한다.

- 최종 native matrix008 전체 PASS: Core default1791 / fixtures1791 / observation1800; host default92 / observation100 + separately executed heap1. fmt/check/event_log/clippy_core/clippy_host exit0, source 불변. heap samples1024512, held_start=held_end781267, idle766515, deferred_bound3551232. 기존 lint allow 두 항목 이외 기준 변경 없음.
- 현재 공개 consumers PASS: host failure/retry, privacy8+13, 15개 원본 pack validation, defaults38, frozen cancel/oracle5 + trace5, CLI 비교, profile24명령/6hosts, externalAPI21명령(13CLI/6negative compile), starter0028, 기존 schema 부정 검사, independent area, Continue/rewind 각각7process, fault5mode exact parity+실제 WebSocket. 001 negative 문구 불일치 원본은 보존한다. PC/R05는 이 통합/공개 실행 종료 뒤 별도 순차 측정한다.

- PC `compat_pc_001` PASS576 raw행,36조건31성공/5typed refusal; 실행138.44초, compile포함492.66초, source 불변. `compat_pc_gameplay_001` 원래v32/v31-envelope gameplay/error-code oracle PASS. 실제 compiler layout: BasicAttackAction40, BasicAttackDef168, RuntimeUnit824, UnitTacticalState272bytes.
- `compat_pc_cost_001` rawexit1 보존: RuntimeUnit HashMap bucket의40byte 증가는 설명했지만 setup의 ScenarioSpawnGroup.spawns clone에 포함되는 UnitCombatProfile의 기본공격/optional 무기 프로필 두 inline action(80bytes/element)을 누락했다. source build.rs의 group.cloned 및 실제 선언을 U05archive와 대조했다. 이를 명시한 `compat_pc_cost_002`는576행의 시간 외 모든 필드를 완전 비교해 PASS. 실제 할당량은 증가했으며 할당횟수/native usage/error text/게임플레이 필드는 동일하다. cap·입력·raw행 변경 없음. 기존 수치 상한은 별도R05검증으로 유지한다.

- `compat_r05_002` raw101은 원래 동시 실행 peak 경고만 해당한다. max9,635,797 / 원 cap9,030,188 (+605,609bytes); 다른 hard guard 실패 없음. event-heavy operations44,194/70,671, retained265,798/276,174, peak271,367/276,174; concurrent operations370,928/389,240, retained4,196,928/4,196,928. 원래 allocation 계측의 latency_gate_applicable=false를 유지하며 raw 지연값도 보존한다. `compat_r05_summary_001` PASS.
- 최종 증거 조립 중 X09/X10 및 PE-CRSO01–12는 원래 개별 selector가 비어 있어 전체 실행/public 기록을 소비하는 조건임을 확인했다. 최초 impact 조립의 비어 있지 않아야 한다는 assertion 실패와 그에 따른 미생성 impact를 읽은 coverage preflight 실패는 제품 시험 실패가 아니다. X09/X10에는 전체 실행 및 실제 forward/reverse target refs를, PE-CRSO에는 원문 의미에 맞는 현재 safety/observation selector와 public/heap/service refs를 직접 결속했다. 기존 EVIDENCE_SELECTION은 EVIDENCE_SELECTION_001.json으로 보존했다.
- 이 선행 기록 대조에서 공개 수치 검사 두 구성도 최종 선정에 명시해야 함을 확인하여 실행했다. `public_guards001`, `public_guards_allocator001` 모두 source 불변 exit0; 원래 public_guards 도구/입력/상한 유지. 최종 selection은5suites+13checks+23public과 별도 rawR05를 결속한다.

## U06-F01 correction verification

- First direct formatting invocation failed before cargo because an unquoted PATH contained a Windows directory with spaces (`env: Files/NVIDIA: No such file or directory`, exit127); no product operation occurred. Corrected quoted PATH, cargo fmt PASS.
- `f01_basic_001`: current correction source,16 basic-Heal/ally/overlay/queue/CC/stance focused tests PASS,0fail/0skip;72.82s including compile. Added single+area effective-zero, rounding/current-stat activation, public DTO predicate, release-time stat loss, independent clone/current-stat positive control, fixed-target/no-retarget and action cleanup. Raw receipt/log retained. A subsequent public-consumer test addition requires final-suite binding to the final source; this focused receipt is not misrepresented as final.

- `f01_public_consumer_001`: PASS1 test/8 authored staff cases. Normal grant/equip/deploy + CoreApplication focus/query and2500ms clock: single/area flat0%0,flat0%100 SP0,flat0%1 SP99 remainHP50 and noeligible/selected target; flat0%50 SP40 heals50→90. Explicit injury/current-stat fixtures; authored action comes through changed live equipment data and normal loadout.93.60s including compile.
- `f01_reproduce_001`: exact closed auditor Cargo.toml/lock/main.rs copied byte-for-byte; current source execution PASS. Both ineffective cases now0 automatic starts/0HP events/HP50; unchanged positive control4 starts/4HP events/HP90.58.64s.
- `f01_public_release_build_001`: current release author_content/package_official_content build PASS240.81s. No schema change; public starter003 will exercise original negative boundaries plus0-coefficient authoring acceptance.
- `integrated_core_default009`: current final correction source full default PASS1794(1737lib+57integration),0fail/0skip321.42s including build/waits. Other final configurations in progress.
- `correction/f01_001/EVIDENCE_REUSE.json`: exact product diff is one production file/basic_heal.rs and two test-only deltas.419 old input copies and18 adapters unchanged; existing official Heal flat8/4 alwayspositive.30 old public/performance/fault executions retain their actual old source hashes and are proposed for non-impacting reuse. PC/R05 Damage-only fixtures do not enter the changed owner; layout/CloneShape/native accounting remain exact. New source timing/performance measurement is not claimed. Independent judgment pending.

- `public_starter_schema_003`: fresh release public reader PASS11/11,18.79s. Original8 positive/typed-negative cases preserved; added allzero single, allzero area and SP-dependent single definitions are accepted, leaving current usefulness to the shared runtime predicate. New source receipt and all exact inputs/stdout/stderr retained.

- `integrated_core_fixtures009`: PASS1794(1737lib+57integration),0fail/0skip,350.84s including build. `integrated_core_observation009`: PASS1803(1746lib+57integration),0fail/0skip,336.00s. Both exact final source b33a8a... unchanged; host/static/heap remain in progress.

- Final009 complete: Core1794/1794/1803;host92/100 PASS, observed host ignored heap is separatelyPASS1. fmt/check/event-log/clippyCore+hostPASS; inherited never_loop/eq_op allowances unchanged. Heap1,024,512samples, held_start/end781,267, idle766,515; bounded/retirement assertion passes. No post-focused product edits; all009/selected current receipts bind exactb33a8a019087f3b6766c8cf611636f46246fa3d895464b5a439f6d176694a401.
- `f01_assembly_001`: PASS9.57s.15fresh/30explicitreused executions,20U06/141inherited+31R06 bindings rebuilt, prior canonical5records preserved in correction/f01_001/BEFORE_ASSEMBLY as well as originaltarget archive. Every reused receipt/log/summary was compared to originaltarget RECORDS.json; current native selectors are successful/nonzero. All canonical non-product source documents remain identical except ROADMAP progress preamble; its section0 onward is byte-identical. Independent judgment pending.
