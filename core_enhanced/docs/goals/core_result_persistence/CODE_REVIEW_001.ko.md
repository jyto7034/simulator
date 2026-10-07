# U04 P2G-20 코드 검토

Input: `WR-CRRP-002`, `PIS-CRRP-001`, `DR-CRRP-01..15-001`.
P2G-10의 statement·acceptance를 그대로 검토한다. 이 문서는 구현 완료/실행 증거가 아니다.
exact bytes와 dirty state는 source index/archive와 Review Evidence에 결속한다.
선행 U03의 완료된 소스와 실제 검증은 currentness 확인 후 baseline으로만 재사용한다.

## Current Actual

| Requirement | 실제 producer → owner → consumer와 차이 |
| --- | --- |
| RQ01 | `world/combat.rs::transition_finished_live_battle_to_result`는 terminal observation/debug trace 뒤 `CombatBattle`와 `CombatResult`만 게시한다. `handle_complete_combat_result_inner`가 typed Run 자원, staged employee, consumable, grant, encounter progression, node completion, compendium을 처리한다. 따라서 ack 전 종료에 정산이 없다. 패배 분기에는 `attempts_exhausted`에 따른 재도전이 남아 있다. victory planner는 공통 `EconomyGrantState`/`GrantExecutionContext`를 쓰지만 ChooseOne을 거절한다. `data/validation.rs`도 live ChooseOne을 거절한다. `combat_setup/rewards.rs`가 실제 후보 리스트를 authored UUID/ID에서 한 번 만든다. 현재 요청 후보수 필드는 없으며 고정 후보 리스트의 실제 길이를 사용한다. 후보 소유/희귀도 제한을 만드는 근거는 없다. |
| RQ02 | `ConfirmEnterNode`는 combat/boss에서만 BattleEntrySnapshot을 먼저 잡고 map enter, omen confirmed, VN, content/전투 진입을 실행한다. Event 전투는 이미 진행한 Event state에서 별도 전투 진입 snapshot을 잡는다. `RetreatBattle`은 snapshot을 복원하며 entry 뒤 증가시킨 attempts도 되돌려 현재 횟수 제한을 보장하지 못한다. `RunCheckpointPayload`는 Run/economy/UUID/rewind VN 상태를 capture하지만 phase·pending result/choice·run recovery restriction을 보존하지 않는다. `export_active_run_checkpoint`는 SavePoint만 V9로 export한다. SavePoint가 없으면 Continue할 payload도 없다. 일반 payload validator는 content/seed/power/map/economy/ownership/progression을 검증하며 SavePoint 노드에만 한정된 validator로 오인하지 않는다. restore가 map phase/빈 current content로 고정돼 있다는 점을 전환해야 한다. |
| RQ03 | `GameCore::prepare_application`은 current context/native clone 한도 → sole draft → operation/reconcile/poll/context → prepared를 만들고 infallible commit으로 게시한다. public `PreparedNonBattleApplication`은 exclusive borrow와 prepared 결과/기존 checkpoint를 제공하지만 active battle은 거절한다. public elapsed/tick/bucket/manual active 명령은 현재 먼저 commit한다. U03는 actual runtime error의 최초 typed fault를 application 밖에 남기며 projection/transport 오류를 정상 domain rollback으로 바꾸지 않는다. observation의 prepared/commit 구분도 유지해야 한다. |
| RQ04 | 실제 ws → LoadBalanceActor → PlayerGameActor가 입력을 실행한다. actor는 공유 `Arc<Mutex<ActiveRunStore>>`와 profile persistence를 가진다. 현재 file lock은 process store open 충돌만 막으며 actor별 stale write를 막는 revision이 없다. 명령 handler는 일부 nonbattle battle-entry만 prepare-before-publish하고 나머지는 commit 이후 checkpoint diff를 저장/rollback한다. tick은 전투 update와 snapshot만 내보내며 결과 저장을 하지 않는다. ResetActiveRun은 현재 모든 load 오류에서 invalidate한다. detach는 살아 있는 전투를 보존·정지하고 새 actor Continue와 별개다. |
| RQ05 | active V4는 core bytes/base64+loads/peak+checksum이며 파일 sync→backup rename→current rename→directory sync를 사용한다. publish는 current read 오류를 `.ok()`로 버리며 load는 unsupported current도 backup으로 바꿀 수 있다. invalidate는 marker 뒤 파일을 삭제한다. profile V3는 영구 기록 union이고 unsupported current를 load에서 거절하지만 publish/reset 원본 보호도 점검해야 한다. 기존 Standard completion journal은 profile→active invalidation의 recovery intent를 저장한다. 현재 journal은 일반 결과/선택을 담지 못한다. 기본 dev/prod 설정은 local `persistent_data`; workspace root와 game_server 아래 실제 디렉터리는 조사 시 없었다. 원격 운영 저장은 제공되지 않았으며 운영 무손실 이행을 관찰했다고 하지 않는다. |
| RQ06 | Core capability는 phase와 local completion/attempt 상태를 계산한다. `can_complete_combat_result_locally`는 아직 정산 가능한 원시 전투를 검사하고 snapshot의 can_retreat는 단순 `!finished`다. SelectedEventSnapshot의 CombatBattle summary를 실제 host가 사용한다. debug/test event log 압축은 별도 첨부다. 따라서 ack/choice/retreat 표시와 실행을 같이 전환하고 Continue 결과에 full event log를 직렬화하지 않는다. |
| RQ07 | 기존 combat/node session/checkpoint tests, 실제 persistence tempdir tests, public Application/host fault probes와 socket consumer가 실행 seam이다. 현재 기대값 중 ack 정산, 패배 재도전, SavePoint-only Continue/V9/V4는 의도된 전환 대상이다. 단순히 old test를 삭제하여 통과시키지 않고 새로운 관측 기대와 원본 전후 diff를 남긴다. 각 I/O 단계 process crash와 경쟁 choice는 새 시험이 필요하다. |
| RQ08 | U03 audit003은 Complete/High로 닫혔고 closure publication은 상태 문서만 바꿨다. U03 12개와 이전100개를 합친112개 조건, 별도 R06 semantic/transitive31개 적용성을 인수한다. prior source 및 receipts/원본 감사는 수정하지 않고 새 영향을 추적한다. 후속 구현의 성공을 이 baseline으로 대신하지 않는다. |

## Proposed Implementation

아래는 승인 의미를 보존하는 technical decision이다. 구체 Rust 이름·helper 분할은 GEX 상세 설계에서
같은 owner/실패/검증 경계를 유지하며 정한다. 정책 의미가 바뀌면 P2G-10으로 돌아간다.

### TD01 — 공통 전투 결과 정산과 확인 정보

기존 combat result planner/staged grant/node-completion을 유일 정산 owner로 유지한다. 실제 terminal transition에서
이 경로를 한 번 실행하여 직원/typed resource/consumable/XP/보상/진행/profile을 같은 prepared root에 넣는다.
패배 retry 분기를 제거하고 현재 encounter failure/monolith/normal-boss 소비 owner에 즉시 failure completion을 전달한다.
정상 승패 우선 판정은 기존 BattleVerdict producer를 사용하며 UI나 host에 별도 판정기를 만들지 않는다.

확정 결과 receipt는 battle identity, 표시 summary, 이미 계산한 acknowledgement outcome, 다음 phase,
필요한 선택 권리를 소유한다. CombatResult 화면은 receipt를 보여주고 ack는 receipt 소비/다음 phase 이동만 한다.
node completion/failure가 node session/content를 정리한 뒤에도 receipt의 phase/context 불변식을 명시적으로 검증한다.
terminal run의 영속 처리는 receipt 화면 유무가 아니라 실제 settlement의 terminal disposition을 따른다.
이미 확정된 RunComplete/RunFailed/Abandoned를 Continue 가능한 게임으로 재해석하지 않는다.

receipt는 현재 한 결과에 한정하며 모든 과거 결과를 쌓지 않는다. 전체 BattleCore/runtime facts/EventLog를 새 저장
그래프로 만들지 않는다. 기존 debug trace/선택적 event-log 첨부는 ephemeral이고 없으면 없음으로 표시한다.
저장에 필요한 summary와 grant result는 현재 admitted 직원/자산/후보에 한정한다. native clone/capture accounting을 우회하지 않는다.

대안: ack에서만 정산은 S01 위반, host에서 복제 정산은 sole grant owner 위반, 전체 runtime serialization은
전투 도중 재개 금지/한도 계약과 맞지 않아 채택하지 않는다.

### TD02 — 선택 권리와 재처리 identity

현재 공통 RewardSession/SelectReward/ClaimReward 흐름을 확장하여 combat의 확정 pending choice를 처리한다.
receipt/entitlement에 실제 RewardOption 리스트, stage/result identity, selected/claimed 상태,
combat participant/selected-employee 문맥과 deterministic grant seed basis를 보존한다. 결과 발생 때 공통 무조건 보상과
encounter progression은 정산하고 선택 후보의 effect는 선택 이후 공통 GrantExecutor로 적용한다.
현재 authored reward list를 예정 quota로 부풀리지 않는다. 실제2개/0개와 다시 열기·복원에서도 같은 목록을 검증한다.
0개는 명시 no-reward 표시 후 정상 다음 진행으로 간다. 정적 참조 오류나 resource 한도 오류를 후보 부족으로 삼켜서는 안 된다.

중복 방지는 현재 receipt/entitlement의 소비 상태와 현재 저장 revision에 귀속한다. 오직 battle UUID의 영구 seen set을
두지 않는다. SavePoint rewind는 그 시점의 자산/권리를 되돌리고 load 누적이 다른 복구 분기에서 정당한 새 결과가 가능하다.
같은 actor의 두 claim은 Core 상태가, 같은 저장에서 갈라진 actor의 두 claim은 TD06의 conditional publish가 막는다.
기존 normal reward node의 완료 전 소비는 unfinished-node rollback 대상이며 combat의 확정 pending choice와 구분한다.

### TD03 — 미확정 노드 경계와 stable Continue payload

V9 SavePoint-only codec의 후속 strict Core 형식을 도입한다. 최신 안정 원정 state와 선택적 단일 SavePoint,
현재 확정 result/choice·복구 제한·phase를 명시적으로 보존한다. run/economy/RNG/UUID/content identity의 기존 capture와
validation을 재사용하고 stable current와 explicit checkpoint를 혼동하지 않는다. foreign content, ownership,
중복/누락 identity, 잘못된 pending choice/phase, load>3 등은 import 전에 거절한다.

node entry 전에 모든 현재 node 유형의 복구 경계를 고정한다. 진행 중이면 export는 이 pre-entry root를 사용하고
성공 후퇴 usage overlay만 현재 값을 합친다. 완료/승패가 확정되면 새 stable root로 전환하고 old pre-entry root를 닫는다.
pre-entry capture와 explicit SavePoint는 서로 재귀적으로 복사하지 않으며 immutable Arc 공유와 현재 capture accounting으로
무한 snapshot 중첩/매 tick 전체 직렬화를 피한다. 전투 tick은 실제 durable boundary가 변할 때만 새 bytes를 만든다.

run 시작의 현재 확정 직원 선발/지도와 비전투 준비 단계의 확정 변경도 export할 수 있게 하므로 첫 SavePoint 전 Continue를
지원한다. 시작 전 briefing/직원 후보의 중간 단계는 current lifecycle에 맞는 명시 rollback/없음으로 처리하고, 확정 원정에
새 예산/새 직원 생성이 다시 실행되지 않도록 한다. unfinished Event 안의 소비/부화 상당 기존 상태는 소비와 결과를 함께 복원한다.

현재 `MapProgression::validate_checkpoint_state`는 완료 노드뿐 아니라 일부 활성 unavailable node를 검증한다.
새 resume-state validation은 existing economy/map validators를 공통으로 사용하되 허용 phase와 required context를 추가한다.
checkpoint restore의 기존 VN session counter exception과 영구 profile merge는 유지한다. 새 저장 reader/writer를 같은 변경으로 낸다.

대안: 마지막 SavePoint만 계속 export는 S04/S05 위반, 매 tick live state 직렬화는 전투 중간 재개와 성능 계약 위반이다.

### TD04 — 후퇴와 명시 load

현재 `AbnormalityAttemptState`의 진입 횟수와 성공 후퇴 usage를 구분한다. authoritative 사용량은 노드별 성공한 후퇴2회로
설정하고 admission/capability/snapshot을 같은 helper에서 계산한다. 현재 monolith 재현의 max attempts2는 후퇴2 규칙의
별도 예외가 아니므로 현행 원시 재도전 수치로 새로운 종료 복구 금지를 만들지 않는다. 승패는 언제나 즉시 소비한다.

BattleEntrySnapshot을 복원하기 전에 node identity/사용량을 확보하고 복원 뒤 증가한 후퇴값을 반영한다. snapshot 실패·거절은
차감하지 않는다. Event combat의 전투 전 snapshot과 전체 Event node 입장 전 종료 복구는 별도다. 미확정 종료는 현재 사용량을
pre-entry root에 overlay하되 explicit SavePoint load는 overlay하지 않고 당시 값을 복원한다. load 사용량은 root 밖 monotonic
host metadata와 함께 게시한다. profile/기존 peak는 현재 후속 owner 전까지 기존 monotonic 규칙을 유지한다.

### TD05 — 기존 prepared owner를 확장한 durable publication

기존 exclusive prepared root를 command와 host tick에도 제공한다. prepare는 원래 route, authority/generation, U03 fault
classification, observer guard를 유지하고 projection·durable export·profile snapshot까지 fallible 준비를 끝낸다.
dropping candidate는 domain 상태를 게시하지 않는다. host는 TD06으로 intent를 durable하게 접수한 뒤 infallible Core commit을
소비한다. command/tick가 서로 다른 사후 rollback 엔진을 갖지 않는다. 비영속 Core public execute/elapsed/bucket도 같은
정산 owner를 소비하며 기존 정상 public 호출을 유지한다.

native runtime fault는 기존 application 첫 fault를 남긴다. persistence preparation/IO 오류는 U03 gameplay fault가 아니다.
intent 접수 전 실패는 candidate를 폐기하고 명시 저장 오류를 반환한다. 접수 뒤에는 Core를 이전 root로 되돌리지 않고
accepted/recovery-pending으로 남겨 재처리한다. 이때 새 mutation/tick는 복구 전 막고 읽기/종료/복구 안내를 제공한다.
projection/전송 실패는 이미 접수한 결과를 취소하지 않는다. observation commit count는 실제 Core commit 한 번에만 증가한다.

매 live tick에서 immutable resume bytes/profile이 바뀌지 않으면 IO는 필요 없다. 실제 active→result 또는 성공 retreat,
nonbattle confirmed mutation/load만 새 durable intent를 만든다. last-valid root/실패 work ledger/cancel rollback는 선행 계약대로다.

### TD06 — 실제 파일 journal·revision과 복구

현재 profile Standard completion journal의 sync/replace/directory sync 수단을 공통 application commit intent로 확장한다.
intent는 version/checksum, expected active revision, 새 고유 revision, active payload 또는 terminal tombstone, merged profile을
함께 담는다. 기존 단일 active local-install 저장을 유지하며 player별 DB를 추가하지 않는다. host는 같은 lock 순서를 사용해
active/profile 현재성 확인과 intent 접수를 직렬화한다. profile은 기존 영구 union을 반영하고 다른 actor의 확정 기록을 소실하지 않는다.

새 active envelope(V4 후속)는 conditional publication revision을 소유한다. tombstone도 revision을 보존해 terminal/reset 뒤
과거의 missing token이 다시 유효해지는 ABA를 막는다. actor가 읽은 expected revision과 현재 값이 다르면 candidate를 접수하지 않고
명시 conflict를 반환한다. startup recovery는 intent의 새 revision이 이미 게시됐는지 검증하여 같은 intent를 idempotent하게 완성한다.
무관한 더 최신 revision이 있으면 덮어쓰지 않고 명시 복구 장애로 남긴다.

파일 순서는 임시 intent write+file sync → intent rename+dir sync → profile/active publish와 각각 file/dir sync →
같은 최신 확정 generation의 recovery copy 완료 → intent 제거+dir sync이다. 접수 뒤 남은 단계의 실패는 journal을 남긴다.
로드 횟수의 최신 값과 결과/권리도 recovery copy에 반영한 뒤 완료하며 오래된 gameplay backup으로 최신 확정 보상을 잃거나
load 횟수를 되돌리는 경로를 만들지 않는다. 기존 rolling-backup 정책의 내부 파일 배치는 이 목적에 맞게 조정한다.
각 게시 지점에 실제 subprocess crash fixture를 연결한다. POSIX rename/sync는 현 Linux에서 실행하고 Windows 분기는
소스/build 계약만 확인하며 Windows crash를 실행했다고 주장하지 않는다.

### TD07 — version·원본 보호와 관리 명령

새 active/Core version은 구 V4/V9를 미지원으로 명시 거절한다. 과거 converter source/API의 역사적 결과를 자동 import하거나
새 payload로 조용히 해석하지 않는다. 현재 영구 profile V3의 필드 의미는 바뀌지 않으므로 그것 자체를 불필요하게 폐기하지 않는다.
미지원 profile version/legacy files는 load뿐 아니라 publish/reset/recovery에서도 먼저 보호한다. 구형 파일을 쓰기 테스트의 대상이나
자동 정리 대상으로 사용하지 않는다. 기존 저장이 있으면 원본을 별도로 보존한 새 저장 위치에서 새 원정을 시작할 수 있으며,
현재 요청에 운영 원본의 이동/변환을 실행할 필요는 없다. reader error를 무조건 invalidate하는 host reset 분기를 제거한다.

새 journal의 미지원/손상도 덮어쓰지 않는다. 이전 Standard completion journal이 있으면 지원 여부를 명시 확인하고 기존 원본을
임의 새 application intent로 재해석하지 않는다. empty configured paths와 isolated fixture에서 새 형식의 전체 경로를 검증한다.

### TD08 — consumer·검증·증거 구조

Core phase/context, can_complete/retreat/claim/load와 실제 실행을 같이 변경한다. host state/notifications는 Core가 확정한
result/choice/durability 상태를 표시한다. summary/event-log absence는 명확히 구별하고 클라이언트에서 승패/노드/후퇴를 다시 계산하지 않는다.
stock ws와 외부 public Core fixture를 실제 호출한다. 기존16 public consumers의 protocol/non-timing 결과는 변화별로 설명하고
새 receipt/후퇴 DTO 차이는 승인한 CF-D07 범위에서 갱신한다.

source-before/after archive는 이번에 변한 실제 source와 필요한 문서를 복원 가능하게 보존한다. prior U03 target의 중첩485MB
archive를 다시 중첩하지 않는다. 변하지 않은 predecessor proof는 원본 path+digest로 external binding하고 actual member 검증을 남긴다.
hash만 남겨 changed-source 복원을 생략하지 않는다. 다음 freeze는 membership/include/exclude/external basis를 명시한다.

## Validation Seams

| ID | acceptance / 실제 시험 경계 |
| --- | --- |
| V01 | PE01/02: public Core manual와 host auto tick의 즉시 victory/defeat; ack 전 state/파일; 중복 ack; 동시 승패 우선; 첫 패배 node consume/no retry |
| V02 | PE03: authored ChooseOne 2개/0개, 고정 candidate/seed/context; Continue before/after select/claim; 중복/경쟁 claim 최대1회; no replacement reward |
| V03 | PE04/05: 두 성공 retreat→마지막 도전, 거절 횟수 불변, 전체 snapshot/RNG/자산 복원, Event 전투 경계, 현재 monolith 재현에도 정책 적용 |
| V04 | PE06: combat/Event/shop/maintenance/reward/headquarters/sweep 미확정 중 quit/process restart; 입장 전 소비·결과 동시 복원; 완료 노드 유지 |
| V05 | PE07: 첫 SavePoint 전 stable Continue, latest confirmed after SavePoint, 직원/예산 identity, terminal invalidation, 명시 교체 확인 |
| V06 | PE08/09: SavePoint trauma ceil15% alive/Critical/Breakdown/Dead; 원자 완료; load3 across save/restart/backup; rewind retreat vs monotonic loads/profile |
| V07 | PE10: 실제 incapacitation/redeploy/multiple units의 once injury/trauma, last entity summary, death prevention/equipment/fragment/consumable cleanup, ack/Continue no repeat |
| V08 | PE11: 동일 actor disconnect/reconnect pause, process restart 대비; U03 five faults/late callback/writer protection/Quit recovery 현재성 |
| V09 | PE12: 실제 file adapter subprocess crash at every intent/profile/active/backup/clear stage; retry outcome/IO fault distinction, result ack 전 종료 |
| V10 | PE13: 동일 active revision의 경쟁 choices/other stale actor; terminal/missing ABA; current branch after checkpoint repeated battle identity |
| V11 | PE14: V4/V9/unsupported profile/current+backup+journal fixtures 원본 bytes 보호; reset/newrun/publish/recovery 거절; strict new codec tamper/identity validation |
| V12 | PE01..15: actual stock ws + public Application 소비자/DTO/capability parity; malformed input/authority/fault gating; active/profile 모두 설정된 파일 경로 |
| V13 | PE11/12/15: inherited112 + 별도 R06 semantic31; actual default/all-feature/core/server checks·fmt/clippy/release; failed raw outputs와 approved baseline exception 보존 |
| V14 | PE05/06/09/15: currently introduced state inventory와 every restore-boundary coverage; future owners 명시, 새 미래 schema 부재, readonly/intake/source archive currentness |
| V15 | PE12/15: observation off/on/selected의 result/choice/restore 동등성·retained/commit accounting; 기존 resource limits/cancellation와 정상 PC576 consumer 근거 재사용/영향 재검증 |

## Findings, Unknowns와 적용 한계

| ID | primary classification | disposition |
| --- | --- | --- |
| F01 | implementation_finding | deferred settlement, live ChooseOne rejection, defeat retry, missing retreat limit, SavePoint-only Continue를 TD01–04에서 전환한다. |
| F02 | implementation_finding | active/profile partial write, tick no persistence, stale writer, unsupported reset/publish hazards를 TD05–07에서 함께 전환한다. |
| F03 | technical_decision | single prepared owner + versioned durable intent + conditional revision + compact current receipt; 대안과 실패 의미는 TD01–08. 승인 정책 의미 변경 없음. |
| F04 | out_of_scope | 미래 지도/억제력/창고/도구/유물, sweep 제거, Unity/미합의 고유 콘텐츠. 해당 U14/15/17/20/21/26 owner가 이 복원 계약을 확장한다. |
| F05 | evidence_gap (non-required deployed claim) | 제공된 원격 운영 인스턴스/저장 없음; 실제 deployed behavior/Windows crash는 주장하지 않는다. CF-D01과 local actual adapter 검증이 이번 필수 지원 범위다. |

정식 review의 required workspace/source 경계는 접근 가능하다. 구현 후 실제 안전성·성능·crash 결과는 아직 unknown이며 GEX의
V01–15가 소유한다. old passing tests로 새 의미를 pass 처리하지 않는다. 이 단계에는 새 material-policy finding이나 사용자
답변이 필요한 질문이 없다. actor revision 충돌의 UI 세부와 Rust helper명은 의미를 바꾸지 않는 GEX 상세 설계 범위다.
