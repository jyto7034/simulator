# U03 code-informed 구현 검토

2026-10-04 · P2G-20 · 입력 `WR-CRFI-002` · 제품 변경 없음.
승인 의미와 acceptance는 [P2G-10](P2G_10_POLICY_001.json)의 12개 current element를 그대로 입력으로 사용한다.
이 문서는 구현 가능성·변경 경계·검증 seam의 검토다. 구현 완료 증거는 아니다.
정확한 파일 bytes와 source group은 source plan, source index와 complete Review Batch에서 결속한다.

## Current Actual

### RQ01 — 오류 생산과 application 원자성

`runtime_control.rs::RuntimeFaultCode`에는 실제 hard guard/Invariant뿐 아니라
HealthCostUnavailable, ShieldUnavailable, InvalidControlTarget도 있다. `ResourceFaultCode`에도
Insufficient/ReservationConflict와 scope/lease/invariant 실패가 함께 있다. 모든 enum member를
일괄 fatal로 취급하면 기존 정상 거절 의미가 바뀐다. `ValueExprFault`는 닫힌 code/phase와
node index 외에 동적 site/cause 문자열을 소유하므로 전체 clone을 영속 fault로 보관하지 않는다.
실제 typed expression failure는 `boss_action_tests.rs`의
`r11_hello_actual_child_expression_error_restores_live_overlay_and_all_facts`에서 생산된다.
이 private rollback 시험은 새 공개 application 수명의 대체 증거가 아니다.

`world/application.rs::prepare_application`은 native/deferred scope, context/resource 검증,
preflight clone, controlled draft, operation/reconcile/poll을 실행하고 sole `commit_application`만
live root를 바꾼다. 실패 candidate는 버려지지만 현재 공개 `CoreApplication`에는 별도 fault 상태가 없다.
`application.rs::execute_internal`, elapsed/host_tick/next_event/pause는 raw GameError를
즉시 ApplicationError로 바꾼다. `from_domain`의 진단 projection은 다시 실패하거나 outer authoring
first-failure를 반환할 수 있으므로 이 변환 전에 최소 원인을 잡아야 한다.
`ApplicationResult::from_behavior`는 domain 호출 성공 다음의 별도 public projection이다.
그 실패를 domain 실행 오류로 되돌려 분류해서는 안 된다.

`InvalidStaticData`는 profile 입력 검증, 조회 직렬화, 실제 battle 내부 데이터 불일치에도 사용된다.
따라서 enum뿐 아니라 작업 origin을 함께 구분해야 한다. active battle의 실제 gameplay mutation에서
trusted runtime 데이터/불변식 오류를 terminal로 분류하되 profile/admin 입력 오류와 read-only 조회는
같은 문자열/variant라는 이유만으로 전투를 종료하지 않는다. 실제 runtime hard fault의 원래 typed code는 유지한다.

### RQ02 — 실제 host 수명과 reachable 소비자

`PlayerGameSession` websocket auth는 `GetOrCreatePlayerActor`→`AttachSession`→snapshot/side messages
후 EnsureLiveBattleTick을 보낸다. player/admin 명령도 성공하면 Ensure를 보낸다.
actor tick의 현재 오류 처리는 Error를 보내고 timer를 취소하는 것뿐이다. Ensure는 socket/active battle만
검사하므로 다시 예약할 수 있다. 수동 명령의 일반 Err는 timer 취소 없이 빠져나온다.

Attach는 세션 교체 시 pause, profile sync, snapshot 후 active battle reconnect/setup을 생성한다.
Detach는 pause 후 active battle이면 TTL을 설치하지 않는다. 이 기존 active-battle 보존 수명을
fault에도 유지해야 재접속으로 새 정상 owner가 만들어지지 않는다. host는 별도 오류 정책을 복제하지 않고
Core의 fault query를 소비해야 한다. admin 및 profile 관리 명령은 일반 execute 이전에 처리되므로
중앙 entry gate가 필요하다. session authority 검사는 gate 앞에 유지한다.

### RQ03 — 실제 저장 reader/writer와 복구

player handler는 profile 관리→profile sync→rollback/checkpoint capture→Continue 또는
prepare/persist/commit/execute→active/profile 저장→transport 순이다. S1 F02의 prepared battle entry는
모든 fallible Core 준비와 checkpoint projection을 먼저 끝내고 host 저장 뒤 한 번만 commit한다.
미게시 후보 실패를 live battle fault로 승격하지 않아야 이 계약과 재시도가 유지된다.

`persist_player_profile`, 관리 reset/retry/standard-completion recovery, active publish/invalidate,
actor `stopped()`의 merge_and_persist가 실제 writer다. snapshot용 profile sync도 Core mutation이다.
`ActiveRunStore::load`는 backup 복구 시 파일을 쓸 수 있으므로 fault snapshot/관리 gate에서 이 reader도
호출하지 않는다. current store/profile는 local-install 공유 소유이며 fault actor만 차단해야 한다.

명시적 websocket Quit는 actor 종료→Deregister를 요청한다. 다음 GetOrCreate는 새 CoreApplication을
만들고 저장 profile만 병합한다. Continue는 별도 사용자 명령이며 `active_run.rs`의 기존
checkpoint validation과 `loads_used`를 따른다. Continue는 기존부터 명시적 Load와 구별되어 loads를
증가시키지 않는다. U03이 새 무료 Load 예외를 만들지 않는다. `node_flow.rs::handle_load_run_checkpoint`
의 상태 gate/횟수 제한을 오류 복구용으로 우회하지 않는다. 기존 Continue도 valid checkpoint가 없거나
content/profile 검증이 실패하면 기존 typed 거절을 반환한다.

### RQ04 — 조회·진단과 공개 wire

Core snapshot은 typed domain DTO를 JSON으로 projection하며 host는 RunSnapshotDto로 deserialize한 뒤
host overlay를 합친다. application에만 fault field를 붙이고 host가 그 DTO를 다시 decode하면 새 field가
소실될 수 있다. host도 직접 Core fault query를 받아 자기 snapshot에 명시적으로 포함해야 한다.
fault snapshot은 정상 action capabilities를 광고하지 않고 live battle reconnect projection 대신 오류를 표시한다.
대형 snapshot 구성 자체가 실패해도 최소 fault query/host fault status는 남아야 한다.

기존 diagnostic은 source/output bytes, nodes/depth/work를 제한하고 trace는 rows/bytes/row/work가 유한하다.
최초 capsule은 UUID, 닫힌 code/phase/operation, 정수와 static site만 유지할 수 있다. 동적 cause·content
ID·trace 전체를 복사할 필요가 없다. 오류 상태의 반복 조회는 실패를 다시 실행하거나 이력을 누적하지 않는다.

### RQ05 — private·speculative owner

TrialSession은 `prepare_step`에서 `fork_for_authoring`한 application을 실행하고 mandatory receipt와
usage가 성공해야 candidate를 accepted session에 게시한다. 실패한 fork는 버리고 기존 accepted prefix를
유지한다. query/preview도 별도 candidate/읽기 경계다. private BattleCore의 transactional retry는
그대로 유지한다. 공개 live CoreApplication의 sticky 오류를 이 내부 owner로 소급 전파하지 않는다.
fork는 원본의 이미 존재한 fault를 복사해야 하며 새 owner라는 이유만으로 명시적 fault를 지우는 포트가 아니다.

### RQ06 — 관측 종료

SessionState::publish는 committed snapshot와 함께 commit_count를 증가시킨다.
실패 operation summary는 rejected와 attempt work/peak를 남기지만 latest committed battle은 그대로다.
Worker::snapshot은 current.battle가 Some인 세션만 active 집계에 포함하고 선택 battle가 없어지면
ended row를 남겨 선택을 해제한다. 따라서 fault terminal을 가짜 성공 commit으로 publish하지 않고
latest active identity/usage만 종료 정리하는 별도 내부 경계가 필요하다.
observer/queue를 먼저 버리면 마지막 실패 summary의 수신 owner가 사라질 수 있으므로 lease를 유지한다.
N01의 absent-owner/duplicate 계약과 bounded recorder/exporter는 변경 필요가 없다.

### RQ07 — 실제 검증 진입점

기존 `game_server/examples/observe_runtime.rs`는 공개 validated content loader와 CoreApplication만으로
seed23의 VN/starter/map→ordinary battle을 실행한다. native max_work1000은 entry 후60000ms advance에서
실제 한도를 발생시킨다. 기존 `public_profile_probe.py`는 release server/HTTP readiness/websocket auth,
command/reconnect를 실제 실행한다. host unit fixture도 공개 CoreApplication을 사용한다.
이 입력/driver를 재사용하되 U03의 sticky gate·reconnect·save/origin 보존을 확인하는 실제 실패 driver가
새로 필요하다. production에 임의 오류 주입 branch를 넣어 그 branch만 시험하지 않는다.
동적 수식 오류는 정상 compile/admission을 통과하고 실행 때 실패하는 입력으로 추가한다.

### RQ08 — 선행 증거

Master002 Complete/High와 END 종료 후 U02 현황 두 문서만 변경됐다.
[baseline 인수](evidence/PREDECESSOR_BASELINE_001.json)는 제품 입력359개의 bytes가 동일함을 확인한다.
따라서 U02 final Core1701/1701/1707, host77/84, 별도 allocator와 공개 소비자 증거를 U03 변경 전 기준으로
재사용한다. 이를 U03 변경 후 PASS로 부르지 않는다. U02 PE12개와 U01/R06/R07의88개 조건을 조건별로
매핑하고 실제 변경 이후 영향을 받는 조건을 새 source 증거에 다시 결속한다. R06의31 original acceptance
매핑도 유지한다. R05 raw101의 기존 concurrent peak 단독 경고, OBS001과20분 제외는 그대로다.

## Proposed Implementation

| ID | 권장 구현·공유 계약 | 대안·이유와 영향 |
| --- | --- | --- |
| TD01 | CoreApplication에 고정 크기 최초 battle fault capsule을 두고 raw domain Err를 공개 진단으로 바꾸기 전에 기록. query accessor와 일정한 `battle_faulted` 후속 거절 제공 | GameCore draft 안의 flag는 rollback으로 사라지고 host-only flag는 embedding이 우회하므로 제외. GameCore gameplay root는 마지막 commit 유지 |
| TD02 | 작업 origin을 battle mutation/read-only/admin-profile 경계로 구분. typed native hard/invariant/resource structural fault, 실제 runtime ValueExprFault와 trusted battle internal failure를 분류. health/shield/target의 정상 거절, resource insufficient/conflict, 일반 GameError authority/action/input/cursor는 비terminal | 모든 Err fatal이나 문자열 matching은 기존 의미를 깨뜨림. 일반 profile/admin InvalidStaticData·query 오류는 fatal로 승격하지 않음. domain Ok 뒤 ApplicationResult projection Err는 latching 밖 |
| TD03 | execute/generation/advance/tick/next/pause/admin/continue/merge/reset/rollback/prepare와 fixture 공개 mutation에 fault gate. public reset의 void→Result로 실제 checked 실패 반환, 유일 host caller 처리 | 임의 panic/무시를 유지하지 않음. capture/export도 fault 상태에서 새 저장 자료를 제공하지 않음. private prepared commit은 exclusive borrow로 중간 fault 발생 불가능 |
| TD04 | Run/action-capability query는 fault 표시와 실행 capability 제거. host snapshot에도 typed fault 포함, fault-only 최소 응답 fallback. active_battle_reconnect는 fault일 때 live setup 반환 안 함 | domain saved Run schema에 fault를 넣으면 checkpoint 수명/형식 변경이 되므로 application 수명에 둠. 정상 None일 때 기존 JSON shape 유지 |
| TD05 | player/admin entry의 session 검사 후 fault gate, 최초 Err에서 timer 즉시 취소. Ensure/tick/queued callback에 guard. Attach/Detach는 fault일 때 pause/profile mutation/live projection 건너뛰고 Core query 소비 | 동일 actor와 fault를 유지해 reconnect 자동 재개 차단. 정상 reconnect/authority 거절 기능 유지 |
| TD06 | 관리 명령·persist helper·stopped writer를 fault gate로 닫음. fault snapshot에서 store.load/recovery/profile sync를 실행하지 않음. Quit는 그대로 허용하고 새 owner의 기존 Continue만 복구로 노출 | 전역 store lockout은 다른 player를 막으므로 제외. 원래 파일/loads/content identity/backup 규칙은 새로 고치지 않음 |
| TD07 | SessionObserver의 terminal 정리는 current battle/usage를 비우되 commit_count를 올리지 않음. 기존 operation guard가 첫 실패 summary를 마무리하고 lease는 유지 | observer drop이나 가짜 commit은 마지막 실패/집계를 왜곡. worker의 existing None cleanup 사용. safety ledger/labels/한도 변경 없음 |
| TD08 | 새 focused Core/host 시험과 release public driver, 전체 필요한 feature matrix 및 조건별 predecessor impact로 검증. 단일 Goal에서 Core+host+save+관측을 함께 완료 | Core-only 완료를 먼저 외부에 게시하면 host 재개/저장 우회가 남으므로 하나의 원자적 전달 단위 권장. implementation/verification 작업은 내부 순서로 나눌 수 있음 |

TD02의 코드 분류는 닫힌 enum match와 origin으로 구현한다. ValueExpr/runtime capsule은 동적 String을 소유하지
않는다. 최초 public 오류의 기존 kind/code는 가능한 그대로 반환하고 후속 `battle_faulted`의 details/query로
동일 first fault를 제공한다. 진단 생성 실패는 그 capsule을 지우거나 정상 상태로 돌리지 않는다.
read-only 명령의 실패를 새 terminal로 만들지는 않지만 이미 fault 상태에서는 일반 command 진입을 닫고
명시적 snapshot/fault query를 사용한다. 이것은 PE05/06의 조회 허용과 mutation gate를 함께 만족한다.

## Validation Seams

| ID | 확인할 실행·판정 | 적용 PE |
| --- | --- | --- |
| V01 | 공개 live application의 실제 native 한도 최초 실패, 마지막 root/RNG/event 보존, 후속 모든 mutation 거절과 first identity 불변 | 1,2,3,6 |
| V02 | 실제 동적 수식/typed invariant 실패와 정상 authority/action/resource 거절의 대조. read-only/profile/admin 입력 실패, 미게시 battle entry 실패는 비terminal | 1,3,4,12 |
| V03 | public projection/postcommit 전달 오류와 precommit runtime fault를 구분. 진단 취소/한도 실패 후에도 bounded fault query와 gate 유지 | 4,5 |
| V04 | fault Run/capability/reconnect projection; 최소 fault fallback; 대형 cause/반복 조회의 bounded metadata와 기존 trace 제한 | 2,5,6 |
| V05 | 실제 actor 수동·tick 최초 실패, Ensure/late callback/중복 명령·admin·관리 포트의 재개 차단 | 2,6,7 |
| V06 | detach/reconnect/session replacement 후 같은 오류. stale session은 권한 거절, live setup/timer 자동 복귀 없음 | 5,7 |
| V07 | 유효 active/profile files의 전체 bytes·write 경계, 첫 fault→반복 관리 명령→reconnect→stop까지 보호. 정상 준비 저장 실패 retry도 유지 | 3,6,8,12 |
| V08 | 실제 Quit→새 actor→기존 Continue 성공/부재/거절. old fault identity 재개 없음, loads/save-content 규칙 보존 | 8,9 |
| V09 | 동일 host 두 player, 한 actor fault와 다른 actor의 tick/명령/정상 저장 진행. shared store 소유권 유지 | 8,10 |
| V10 | observer failure/work 유지, commit_count 불변, current/active/selected cleanup, 반복 gate의 첫 원인 중복 실행 부재. off/on/수집기 부재·포화 parity | 2,3,11 |
| V11 | private transactional retry, Trial accepted-prefix rollback/cancellation/preview 읽기 전용, public opaque prepared/rollback 권한 유지 | 3,6,12 |
| V12 | source-bound 전체 Core/host 적정 feature matrix, 실제 release 공개 API/CLI/headless host,100 predecessor 조건/31 R06 mapping과 기존 경고 보존 | 12 |

모든 seam은 구현 후 실제 nonzero 실행과 source identity를 필요로 한다. pure classifier tests는 public host
증거를 대신하지 않는다. 새 wire field의 실제 서버 소비와 정상 payload 호환을 확인하며 Unity/원격 배포는
현재 consumer claim이 아니다. full process crash, 데이터 삭제/자동 변환, U04 normal result/save 전환도 제외다.

## Findings and Unknowns

- F01 `implementation_finding`: Core에 sticky 상태 없음, host 재예약·관리/stopped 저장 우회 존재.
  TD01/03/05/06 및 V01/05–09로 구현·검증한다. 현재 안전 완료로 표시하지 않는다.
- F02 `technical_decision`: raw typed 원인·작업 origin과 postcommit public projection을 분리하고 고정 크기
  capsule에 결속한다. TD01/02/04, V02–04가 owner다. 기존 승인 의미의 구현이며 새 정책 변경 아님.
- F03 `technical_decision`: reset의 infallible panic 경계와 observer cleanup을 typed Result/별도 terminal
  update로 바꾼다. TD03/07, V01/10/12가 owner다. actual sole host caller를 함께 바꾼다.
- F04 `evidence_gap` for implementation only: 새 fault wire·실제 host negative fixture·동시 세션 종료 결과는
  아직 실행하지 않았다. V01–12를 단일 Goal의 필수 completion evidence로 넘긴다. 구현 가능성/현재 source
  owner를 모른다는 뜻이 아니며 현 단계에서 새 동작 PASS를 주장하지 않는다.
- F05 `out_of_scope`: 정상 결과·SavePoint policy 전환은 U04, error UI는 U26-C, mod 등록은 U11.
  새 스킬 자동 취소·전체 Run rollback·게임play cap·20분 시험을 추가하지 않는다.

현재 source/owner가 접근 불가인 필수 경계나 미결 material choice는 없다. P2G-10의12개 acceptance 의미를
그대로 보존한다. 실제 구현 중 다른 의미가 필요해지면 해당 owner의 successor 절차로 돌아간다.
