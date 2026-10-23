# U12 코드 기반 전환 검토

입력은 WR-CEXL-007 / RSP-CEXL-002다. 메인 Astra high의 직접 코드 검토와 별도 담당자의 현재 source 보고를 묶는다. 아래 설계는 구현 제안이며 현재 지원 API나 기능 통과를 뜻하지 않는다. 정책 의미와 수용 문장은 P2G-10의 exact member가 소유한다. formal refs와 source bytes는 P2G-20 complete result가 발행할 때 결속한다.

## 현재 경계와 차이

| 요소 | 현재 코드 사실 | 필요한 전환과 검증 seam |
| --- | --- | --- |
| 01 | `world.rs:857–900`의 Endless는 profile tutorial 완료를 요구하며 Standard/Endless가 모드다. 시작은 일부 필드만 비운다. | Tutorial/Expedition 제품 모드·즉시 시작·skip/반복을 공개 입력/phase/query에 함께 연결한다. fresh profile 공개 시작으로 확인하고 튜토리얼이 원정 profile에 쓰지 못하게 한다. |
| 02 | fixed tutorial 배치와 공용 전투·노드 실행기가 존재하지만 일반 진행/profile/보스 omen과 공유된다. | 고정3층·사전 설정 직원/장비·정상 실제 전투·활동을 사용한다. 튜토리얼만 마지막 일반 보스에서 완료하며 모든 grant/도감/기록/종료/복원에서 격리한다. 작성된 적 값과 별도 약화 actor만 적용하고 peak 적 스케일링을 사용하지 않는다. |
| 03 | `employee_data.rs:12–13,126`은 5–7 eligible 허용, `helpers.rs:591–735`는 선택 검증 뒤 UUID와 공통 grant로 기본 세트를 만든다. | 역할별2/총6과 선택3을 실제 등록·로더·공개 query에서 보장한다. starter/paid recruit 모두 기존 grant와 배치8/보유10 경계를 소비한다. 부적합 선택은 draft 전체 거절한다. |
| 04 | `recruitment.rs:35–159` seed 큐·offer·roster exact ownership이 있으나 한 명 채용이 세션을 닫고 나머지를 tail로 보낸다. `headquarters.rs:33–94`에 가격 차감이 없다. | 작성된 유료 가격/최대2회 세션·깊이1–4 자격을 연결한다. 큐 순서/잔여 offer/횟수/가격·wallet·기본 grant가 한 draft로 움직인다. 다른 HQ 서비스가 남고 깊이5 이상은 채용만 제공하지 않는다. |
| 05 | 직원 definition의 profile과 instance condition은 별도지만 `UuidManager`는 run_seed/counters만 있고 같은 seed 재초기화가 같은 UUID를 만든다. | scenario seed와 원정 instance identity를 분리한다. 같은 원정 복원은 identity·성격·condition을 보존하고 새 원정은 새 인원·사본 namespace를 갖는다. 두 성격 축을 입력 정의에서 가져오며 적성은 표시 정보로 유지한다. |
| 06 | `world.rs:928–970`은 starter 선택 때 기존 잔액에 예산을 더한다. 새 시작은 inventory/enkephalin/fragment/relic을 전체 초기화하지 않는다. | 새 원정 출발 draft의 전체 run state 초기화 후 authored 기본 예산을 한 번 지급한다. title/VN 재진입·중복 selection·Continue·SavePoint는 새 지급 경로가 아니다. |
| 07 | `encounter_compendium.rs:23–66`의 profile은 OR/max 지식만 있으며 vault가 없다. equipment/fragment inventory는 사본 UUID·장착·강화/형태를 이미 가진다. | profile vault와 active run inventory를 분리하고 선택한 총0–2사본의 소유를 이동한다. 장비 종류별 quota를 만들지 않는다. canonical definition과 사본 ID/개화는 보존하며 보관품을 definition history로 다시 발급하지 않는다. |
| 08 | `node_flow.rs:607–650,706–776`은 Gate 입장 직후 자동 다음 층 전환이다. RunProgression은 floor index/mode이며 별도 exhaustion 선택은 없다. | 현재 authored depth≥3의 도달한 Gate에 귀환/진행을 둔다. 최소 실제 최초 입장 소모/0 전이와 활동 전 소진 선택을 연결한다. Gate/횡단은 소모하지 않고 0 유지/깊이3 도달로 선택이 재발생하지 않는다. 일반 보스 보상은 원정 자동 귀환이 아니다. |
| 09 | 실제 반출 owner가 없고 현재 run inventory와 profile 사이 이동이 없다. | 실제 성공 귀환 draft만 현재 보유 장비/fragment를 vault로 이동하고 장착 소유를 해제한다. 일반 무기 enhancement를 0으로, fragment unlocked/selected form은 유지한다. 실패/포기는 active 사본만 잃는다. |
| 10 | `combat.rs:1019–1320`은 살아 있음/무기 장착/condition·소지 치료와 selectable SavePoint/Gate만 판단한다. allowed node에는 실제로 도움 없는 노드도 먼저 들어간다. | 실제 행동과 같은 eligibility 계산을 공유하여 치료·장비 재편·유료 채용·즉시 도달 SavePoint/Gate·실제 귀환을 평가한다. 조건 없는 category·돈 없는 채용·없는 기본 무기·깊이2 귀환은 회복 근거가 아니다. 실패 후보는 기존 checkpoint 횟수/복구 결정을 거쳐 확정한다. |
| 11 | `commit_run_failure`는 phase만 바꾸고 `handle_abandon_run`은 state를 새로 만들며 profile만 보존한다. 성공 종료 인력 처분/반출 순서가 없다. | 성공 정산→인력 종료→story pending을 단일 draft/accepted commit에서 확정한다. 실패와 포기도 공통 terminal owner를 사용하되 성공 자산을 실패 손실로 다시 처리하지 않는다. 후퇴/중단/복구는 terminal이 아니다. |
| 12 | profile에 최초 귀환 이야기 상태가 없다. VN trigger의 run-local consumed를 영구 story로 사용할 수 없다. | profile에 never/pending/consumed 상태를 둔다. 첫 실제 성공 귀환 후 pending, play/skip ack는 동일한 단조 소비 전환이다. query가 재시작 후 권위 상태를 제공하고 중복 ack는 자산/직원을 재정산하지 않는다. |
| 13 | `PreparedApplication`은 draft/checkpoint/profile을 미리 만들고 exclusive borrow 후 commit으로 live root를 한 번 교체한다. server는 approved/unbound 모두 journal CAS를 사용하지만 profile을 반복 merge한다. | 이 경계를 재사용해 vault와 active/terminal/story를 함께 확정한다. public commands/DTO·Continue/checkpoint·profile/store·U11 승인/identity·fixture를 같은 전환에 넣는다. 원격 배포/Unity 호환은 주장하지 않는다. |
| PS01/02 | psychology module·Employee member/default·clone shape·snapshot DTO·node advance가 있고 Employee nested decode가 unknown field를 묵인한다. 공개 V7/V8 converter도 있다. | 모델/생성/저장/query/전용 입력·capability와 실제 current fixture를 함께 제거한다. nested current/SavePoint old field와 옛 mode/save를 명시 거절하며 원본과 거절 전 state를 보호한다. condition/성격/영구 기록은 유지한다. |

## T01 — 원정 소유와 시작

GameCore의 prepared application draft가 시작·선발·대여·예산과 terminal을 소유한다. 제품 모드는 Tutorial/Expedition로 구별하고 옛 모드 variant를 새 의미로 조용히 alias하지 않는다. 정확 Rust variant/field 이름은 제품 담당자가 현재 소비자에 맞춰 정하되 public 입력·문서·fixture·저장 reader/writer를 함께 바꾼다. Standard 난이도 이름은 제품 모드 뜻으로 사용하지 않는다.

새 시작에는 `GameCoreState::new`의 초기화 범위를 재사용하되 manager/player와 authoritative profile만 보존한다. 미확정 시작을 취소/실패했을 때 live state, wallet, vault와 UUID 카운터가 바뀌지 않아야 한다. 기본 세트는 `initialize_employees_with_loadouts`의 preview→grant→정확 사본 장착 경로를 재사용하며 대여품을 grant로 재발급하지 않는다. 같은 seed의 지도·큐 난수와 새 개인 identity 요구는 분리한다.

권장안은 profile에 원정 sequence/instance marker를 두고 새 원정 출발 시 조건부로 한 번 전진시키며 사본/직원 UUID namespace를 해당 instance에 결속하는 방식이다. sequence overflow는 거절한다. Tutorial은 원정 sequence를 포함한 영구 profile을 바꾸지 않으며 별도 mode namespace의 내부 identity만 사용한다. SavePoint/Continue는 저장한 instance를 유지하고 새 원정으로 해석하지 않는다. 무작위 UUID를 gameplay seed에 섞어 매번 queue/지도 결과를 바꾸는 대안은 피한다. 외부 trusted host가 발급하는 instance token을 선택한다면 동일한 durable 일회성·rollback·결정론적 scenario 분리 증거가 필요하다.

## T02 — vault와 정확 publication

profile의 기존 encounter/progress 지식은 단조 합침이 적합하지만 vault 사본 소유는 감소할 수 있다. 전체 profile을 기존 `merge`에 넣으면 안 된다. profile 안에서 지식 영역과 exact lifecycle 영역(vault·현재 원정 instance/sequence·story)을 타입/메서드로 구분하고, accepted application이 검증된 다음 상태를 소유하게 한다.

직접 확인한 모든 경계는 `world.rs:165–218`의 profile capture/merge, `application.rs:967–1007,1306–1338,1756–1775`, server `handlers.rs:548–625,3886–3972`, `mod_admission/persistence.rs:67–121`, `player_profile_store/application_intent.rs:258–364`다. 현재 approved/unbound publish, accept, restart recovery에 merge가 각각 있다. 신규 exact 영역은 이 전부에서 동일 의미로 전환한다. 권장안은 정상 mutation에서 profile→active lock 순서로 기존 expected active revision CAS를 먼저 검증하고, 지식만 합친 뒤 exact lifecycle next state를 채택해 journal에 기록하는 것이다. `changed == false` shortcut이 stale revision 확인을 우회해서는 안 된다. 별도 profile revision을 추가하더라도 active/profile 원자적 수락 한 지점을 유지해야 한다.

host의 persisted profile 재동기화는 authoritative exact 상태를 설치하는 경로로 분리한다. 지식용 merge API나 no-durable fallback을 통해 vault를 합치거나 덮어써 자산을 복제할 수 없어야 한다. 정상 durable host는 계속 기존 approved gate와 accepted intent를 사용한다. 독립적인 두 번째 vault 파일/나중 저장은 profile와 active 사이 부분 반출 위험 때문에 채택하지 않는다.

journal rename 전 실패는 candidate를 폐기하고 live/disk 원본을 보존한다. rename 뒤에는 기존 U04 accepted/recovery_pending 의미대로 Core를 정확히 한 번 commit하고 후속 저장을 막은 채 동일 accepted 상태를 복구한다. `finish_application_intent`의 profile→active 기록과 old/new active/required configuration 결속을 유지한다. 복구 마지막 단계에서도 accepted exact vault를 채택하며 OR/union으로 소실 사본을 부활시키지 않는다. 원본 current/backup/intent의 unsupported format을 writer-capable 복구 전에 거절한다.

vault 장비는 canonical definition ID·copy UUID·필요한 실제 상태, fragment는 definition ID·copy UUID·unlocked/selected form을 보존하는 저장형을 권장한다. metadata 전체를 영구 권위로 고정하거나 장착된 직원 UUID를 다음 원정에 넘기지 않는다. 현재 catalog에 없는 보관품은 원본을 잃지 않고 dormant로 유지하며 대여만 명시 거절한다. 모든 장비 유형을 포함한다. 현재 inventory에서 사라진 사본을 획득 기록으로 복구하지 않는다. profile decode/clone/publication은 기존 byte/work/retained admission을 소비하고 유한한 검증을 적용한다. 새 임의 gameplay 보관 개수 상한을 도입하는 것으로 비용 문제를 숨기지 않는다. 저장/import/대여/반환 validation은 vault와 active 사본 UUID의 합집합에 단일 소유를 요구한다. 서로 다른 종류/definition의 동일 UUID, map key와 embedded ID 불일치, dormant vault와 active 충돌도 거절하고 기존 원본을 유지한다.

## T03 — 귀환·실패·이야기와 복원

한 terminal 정산 메서드가 source cause, instance, 귀환 자격, 현재 보유 사본을 검증한다. 성공 귀환은 사본 이동→일반 무기 강화 reset→장착 해제→직원 종료→최초 story pending→Continue 무효화를 같은 draft에서 처리한다. 확정 실패/자발 포기는 active 사본과 직원만 종료하며 vault 미반입/영구 기록을 유지한다. 실패 후보와 checkpoint 복구는 이 메서드를 아직 호출하지 않는다. terminal phase/instance를 사용해 중복 command가 재정산하지 못하게 한다.

최초 이야기 ack는 terminal 이후 profile-only mutation일 수 있으므로 active checkpoint가 None이어도 조건부 publication이 동작해야 한다. 다음 원정이 pending을 지우거나 다시 만들지 않는다. UI 재생 자체를 Core가 확인한 것처럼 주장하지 않으며 play/skip ack의 소비만 소유한다.

SavePoint와 전투 후퇴는 run-local 자원·직원·사본·선택을 복원하고 vault 전체 snapshot을 되돌리지 않는다. 같은 instance의 대여 사본은 run-local 복구에 포함될 수 있으나 동시에 vault에도 존재할 수 없다. 종료된 run의 오래된 Continue를 새 원정에 적용하지 못하도록 instance와 authoritative profile/active pairing을 확인한다. profile/active의 immutable accepted boundary와 필요한 binding을 runtime validation에 포함한다. pairing은 같은 instance의 합법 복원을 profile 전체 revision equality로 막지 않아야 한다. 같은 원정에서 지식이 추가되거나 profile-only story ack가 확정된 후 이전 SavePoint를 복원하는 양성은 유지하면서 최신 vault/story를 되감지 않는다. checkpoint None인 ack도 stale no-change CAS 검사에 포함한다.

## T04 — 최소 실제 지도와 소진 선택

U12는 U13 전체 시설을 선구현하지 않는다. 현재 정상 fixed facility 경로에서 실제 전투/활동과 깊이1/2/3 이상 Gate가 있는 지원 개발 원정 profile을 사용한다. authored 현재 깊이를 return eligibility source로 두며 방문 횟수·목적 깊이·미래 Gate를 사용하지 않는다. Gate 확인은 먼저 선택 phase를 만들고 계속 선택 때에만 기존 하강/회복 경로를 호출한다. Gate 경로의 귀환은 실제 현재 Gate를 선택한 상태에서만 허용한다.

소진은 정상 노드 최초 입장에 한 번 기록되는 최소 authored 비용/state로 연결한다. 입장 확인→최초 소모/입장 receipt→0 전이 선택→활동 시작의 순서를 분리하며 취소/후퇴/재입장/Continue에서도 중복 소모나 선택 재발생을 막는다. 소진 선택은 현재 깊이1–2에서 계속/포기, 깊이3 이상에서 귀환/계속을 제공한다. 이 소진 귀환은 비Gate 활동 노드에서도 허용되며 Gate 자격과 별개의 source cause로 검증한다. 계속 선택 이후에는 저장한 pending activity를 실행하고 귀환이면 활동 보상/완료를 만들지 않는다. 초기량/비용은 정책이 위임한 authored tuning이며 실제 준비된 정상 경로와 고정 workload로 확인한다. U14 전체 Breakdown/황금가지/무억제 배율·U13 전체 graph·개별 최종보스는 현재 완료 주장 밖이다. 기존 공용 전투 능력치 계산을 넓게 바꾸지 않고 tutorial fixed profile의 적 수치만 필요한 경계에서 적용한다.

## T05 — 채용·실제 회복과 튜토리얼

기존 queue/offer/roster 단일 소유 검증을 확장해 세션 내 잔여 제안과 최대2명 채용 횟수를 저장한다. 실제 command와 query/recovery evaluator가 동일 eligibility predicate(깊이, offer, roster<10, wallet≥price, definition/loadout 유효)를 공유한다. 한 draft 안에서 가격 차감과 grant를 수행한다. 미채용 제안은 현재 순서대로 tail에 반환하고 다른 HQ 서비스를 선택해도 같은 규칙을 쓴다. UI-only price나 category 존재만으로 회복 가능을 표시하지 않는다.

회복 evaluator는 현재 inventory의 장비 재배치 가능성과 직원 condition/무기, 실제 보유 치료량, 당장 선택 가능한 노드, 합법적인 paid recruit 및 귀환을 함께 반환한다. 반환한 각 reason/action은 실제 명령으로 실행 가능해야 하며 수행 뒤 다시 평가한다. 치료 후 장비 재편처럼 여러 합법 행동으로 출전 가능해지는 경우도 허용하고, 실행 가능하지만 실현 가능한 연쇄 끝에도 출전/합법 귀환을 만들지 못하는 무의미한 치료·SavePoint는 회복 근거로 세지 않는다. 현재 `CombatRecoveryRestriction.allowed_node_ids`에 행동 종류/이유를 연결하고 불필요한 전투/소비/성장 행동을 허용하지 않는다. 모든 직원 사망 상태는 단순 future recruit category로 회복 가능 판정을 우회하지 않는다. existing checkpoint failure-decision count semantics를 유지한다.

튜토리얼은 사전 설정 roster/loadout와 고정3층 콘텐츠를 사용하고 출발 6중3 selection/대여와 별도다. 같은 공용 battle/result/retreat/SavePoint/Continue를 소비하되 모든 permanent writer에서 mode guard가 성립해야 한다. profile 한 시점 snapshot만 덮어써 격리하는 대안은 다른 세션의 확정 지식/자산을 잃으므로 사용하지 않는다. 쓸 수 있는 임시 compendium이 필요하면 run-local tutorial state로 제한한다. 완료/skip/실패/반복 모두 vault/원정 sequence/story에 비영향이어야 한다. 빈 profile뿐 아니라 실제 보관 사본·known IDs·pending/consumed story·비0 sequence를 가진 profile에서도 종료/Continue/SavePoint 전후 불변을 비교한다.

## T06 — 제거와 새 지원 저장의 단일 전환

memory/trust/psychology model·default·node advance·clone shape·DTO/projection·전용 입력과 실제 current fixture를 제거한다. allocator memory와 trusted-host 권한 용어는 별개의 시스템이다. `Employee`와 지원 저장의 nested structs에서 제거된 필드를 unknown-field rejection으로 검출한다. 버전만 바꾸고 nested legacy input을 무시하지 않는다.

Continue11/SavePoint12/profile3와 새 mode/profile shape는 현재 저장과 호환되는 것처럼 보이게 하지 않는다. 새 명시 version을 writer/reader/server preflight/required-content inspection에 동시 적용한다. V7/V8 converter는 원본 보호와 명시 미지원 결과를 유지하는 방향으로 닫으며 제거 시스템을 지원 새 상태로 조용히 변환하지 않는다. runtime reader reject와 converter reject를 별도 검증한다. version과 checksum을 실제 올바르게 갱신한 악성 current/nested SavePoint 입력으로 target rejection을 확인한다.

## T07 — 승인된 콘텐츠 소비와 검증 범위

새 mode/run-policy/employee-role/tutorial/vault lookup 계약은 정상 loader/cross-reference validation과 U11 candidate identity/capability/suite/cache/Apply/required-mod-save에 함께 반영한다. 승인 우회를 만들어 새 공개 시작 시험을 통과시키지 않는다. 기존 U11 완결된 issuer/cache/권한 알고리즘의 비영향 증거는 재사용하되 바뀐 public preparation·StartNewGame·profile/Continue producer는 fresh 증거가 필요하다. current startup부터 실제 전투/귀환/재시작까지 수행하는 공개 probe를 독립 Astra가 작성하고 Sol이 실행한다.

기존 `world/tests/mod.rs`의 완료 tutorial 선설정, 1명으로 축소된 employee fixture, internal tactical setup은 U12 정상 fresh start 증거가 아니다. 제거된 빈 psychology 유지 시험도 제거 완료 증거로 세지 않는다. 관련 condition/identity/failed mutation/rollback/SavePoint/10 roster/8 deployment 회귀는 의미를 보존해 새 공개 계약에 맞춘다. 공유 serialization/clone/entry capture가 변경되므로 영향받는 U03 원자성·U04 durable/복원·U07M 비용 및 U11 승인 저장을 선정한다. 단순 역할 변경/세션 재개 때문에 전체 시험을 실행하지 않는다.

## 구현 묶음과 남은 상세 선택

mode/start/identity/employee schema와 profile/vault/terminal/publication/reader 전환은 서로의 생산자·소비자다. 이 중 일부만 완료로 공개하면 구형 reader와 새 writer, active 사본과 vault 소유가 충돌하므로 하나의 최소 수명 Atomic Group으로 구현한다. live tutorial/개발 원정 profile과 공개 계약·검증도 같은 독립 완료 Phase 안에서 닫는다. 파일 개수만으로 Goal을 나누지 않는다.

제품 담당자는 구체 module 분리·함수명·DTO field·버전 숫자·작성 가격/소모 초기값과 runner 명령을 기존 API/콘텐츠에 맞춰 상세화할 수 있다. exact lifecycle CAS·mode 격리·instance/사본 정체성·고정 성격 보존·허용 행동·원본 보호·accepted intent 의미는 바꿀 수 없다. 의미 변화가 필요하면 메인에게 근거를 반환하고 해당 owner에서 처리한다. runtime code와 test/fixture/oracle는 파일 단위 순차 소유로 작성하며 메인은 제품/시험 코드를 대신 쓰지 않는다.

검토 한계: 아직 U12 제품·시험 실행은 없으며 밸런스/성능·전체 튜토리얼 플레이 통과를 주장하지 않는다. 현재 필수 workspace source는 접근 가능하다. Released/Deployed/Unity는 RSP002의 명시 N/A이며 로컬 server 정상 소비 검증을 대신하지 않는다.
