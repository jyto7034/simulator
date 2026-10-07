# U06 코드 검토001 — 정책 의미를 보존한 구현 검토

Input: WR-CBST-002, PE-CBST-01–20. 정책 의미/acceptance는 P2G-10 소유이며 이 문서가 변경하지 않는다.
Source plan003은 source plan001/002의 발견된 정상 grant/clone/command 소비자를 보완한 같은 정책 basis다.
제품 변경·새 시험 실행 없음. U05의 Complete/High와 이후 운영 문서 delta는 WORKFLOW_INTAKE_001.json 참조.
아래는 실제 읽은 코드 사실과 구현 제안을 구분한다. complete batch 발행 시 exact149 source/hash와20요소별 coverage를 검증한다. 실행 결과는 GEX에서 별도로 입증한다.

## Current Actual

| 요구 | 확인한 현재 경로 | 변경 필요 |
| --- | --- | --- |
| RQ01 | `data/abnormality_data.rs:92–226` BasicAttackDef와 strict AuthoredBasicAttackDef는 damage_type/targeting/range/interval/windup/delivery뿐이다. `equipment_data.rs:70–164,705–800` WeaponCombatProfile은 적성/타일효과/범위/주기/기본저지 검증을 소유한다. `battle/types.rs:581–605`가 장비 profile을 실제 기본행동에 적용한다. | 기본 회복의 명시 typed schema와 runtime 전파·검증·clone/extent·작성 view를 함께 전환한다. 기존 피해 profile과 특수 총 경로를 유지한다. |
| RQ01/RQ03 | employees/definitions.ron의 starter6명은 모두 standard_training_sword와 direct BasicAttackModifier baseline이다. hq6명은 loadout이 비어 있다. `Employee::from_definition:510`은 baseline IDs만 복사한다. `SkillFragmentLoadout::apply`는 baseline active를 오류로 거절한다. | 정상 active 사본 grant+equip과 장비 grant를 시작/채용 양쪽에서 공통화해야 한다. baseline list에 active ID만 넣는 수정은 실행되지 않는다. |
| RQ02 | `basic_attack.rs`는 적생존/usefulness/range로 target을 고르고 windup 뒤 DamageSourceSnapshot을 담은 AttackResolve를 예약한다. `commands.rs:346`는 같은편을 거절하고 committed damage와 proc/overlay를 처리한다. ApplyHeal(:1068)는 생존 검사와 HP 실제 변화를 이미 공통 처리한다. | 치유는 피해 snapshot/crit/proc 경로를 통과시키지 않는다. 별도 heal resolve carrier를 같은 기본 행동 lifecycle과 공통 HP owner에 연결한다. |
| RQ02 | `ability.rs:145` SkillTarget은 SelfUnit/EnemySingle/CastTarget뿐이다. `skill_data.rs:1064`는 PlayerUnit/AutomaticUnit과 EnemySingle만 허용한다. `sim.rs:730–820`의 explicit direct-target/start gate, `skill_runtime/cast.rs:430`의 선택, `target_usefulness.rs:100`의 실제 delivery usefulness 모두 적 중심이다. | active 단일 치유용 AllySingle과 명시 아군 selector를 입력→컴파일→접수/step→query까지 도입한다. 기본/스킬의 공간 투영은 재사용하고 적대 usefulness를 아군에 강제하지 않는다. |
| RQ02 | `enums.rs:399`의 queue priority는 피해 projectile/skill/AttackResolve(10) 이후 FieldWork(11), Rest(12)다. BasicAttack 시작에서 action source와 finite delivery를 만들고 resolve가 release/dispatch/close를 기록한다. | 기본 heal resolve는 피해 band 뒤에서 실행해야 하며 no-proc와 action/source 정리도 필요하다. 기존 스킬 내부 순서를 무관하게 재작성하지 않는다. |
| RQ03 | `world/helpers.rs:591–704` starter3을 검증하고 장비 grant/placeholder preflight/EquipmentLoadoutTransition을 사용한다. `headquarters.rs:34–83`는 Employee만 roster에 넣고 장비 grant를 하지 않는다. `skill_fragment.rs`는 OwnedFragment grant와 transition::equip_copy의 양방향 binding을 제공한다. | 하나의 정상 initial-loadout owner가 equipment와 active fragment 사본을 부여/장착해야 한다. 기존 UUID/실패 원자성 및 이전/슬롯 자격을 재사용한다. |
| RQ03 | `world/combat.rs:1950`는 condition부터 검사한 뒤 zone/profile/TP/재배치 eligibility를 판정하고 staged live command로 배치한다. state::deployment_candidate_eligibility는 이미배치/개인대기/TP만 검사하며 동시8 검사는 없다. 철수(:2236)는 kernel 철수 후 고정 HP policy와30초 lock을 만든다. | 동시8 상한을 query/최종 실행 공통 gate에 추가하고 누적 participant 수와 분리한다. 출전불가 우선/철수·재배치 owner는 유지한다. roster10은 기존 채용/조회 경계에서 함께 확인한다. |
| RQ04 | tactical_relocation::disabled_reason은 Defense만 허용한다. destination/self occupancy/no-op/profile/CC/windup/pending/cooldown 검증이 존재한다. swap은 양쪽 preflight 후 한 candidate에서 두 위치를 바꾸고 World가 TP5를 한 번 지불한다. | DefenseRequired만 제거하고 같은 실체 보존·최종 점유 원자 경로를 재사용한다. 강제 Defense1500ms와 자동Attack/timer query를 추가한다. |
| RQ04 | UnitTacticalState는 일반 stance/relocation cooldown과 rest epoch만 가진다. tactical_stance::stance_change_disabled_reason은 channel_active에서 ActionLocked를 반환한다. 일반 태세 대기1500ms와 before/after 효과 차이 및 Rest 주기 보존은 존재한다. | 새 강제방어 이유/종료 시각·epoch를 관리한다. 일반 태세는 채널 자체로 거절하지 않되 채널 고유 행동 제한을 유지한다. stale timer는 생존/동일 상태에서만 소비한다. |
| RQ04 | field_work는 작업물별 accumulated work, contributor count/generation을 소유한다. 완료는 status를 먼저 Completed로 바꾸고 fullHP에도 완료 event를 기록한다. transition_field_work_activity(:354)는 입장/퇴장 모두 Defense로 설정한다. | 입장시 강제방어 취소와 완료/이탈시Attack을 적용하고 이동 후 강제방어와 작업 전이를 순서대로 일치시킨다. 누적/CC/no-bank/1회 완료는 기존 경로 검증으로 재사용한다. |
| RQ05 | movement/blocking::refresh는 기존 관계를 먼저 보존하고 route progress desc→spawn→ID, 신규 blocker는 거리→ID로 고른다. 기존 관계에 실제 반경 이탈 검사가 없다. engine::apply_continuous_movement_outputs는 backend의 to를 그대로 적용해 접촉점 sweep이 없다. body/projected tile은 실제 좌표의 투영이며 blocker 타일로 강제 치환하지 않는다. | segment 최초접촉 clamp·용량 순서와 반경이탈 해제를 추가한다. 이동 backend/실제 log/active segment를 같은 최종좌표로 맞춘다. 신규 후보는 기존 유효 관계를 밀어내지 않는다. |
| RQ05 | targeting::is_basic_attack_target_in_range는 모든 기본 공격에 tile reach를 요구한다. basic_attack 선택의 blocker-first도 같은 range filter를 통과한다. | 저지 관계에 한해 기본 공격 range 예외를 공통 함수에 추가한다. skill/tile area/제3공격자/치유에는 확장하지 않는다. |
| RQ06 | ActiveBattleSession은 playback/remainder와 skill 전용 tactical_focus만 갖는다. BeginSkillTargeting은 기존 focus를 거절하고 restore_playback을 저장한다. cancel/activate는 직접 restore한다. SetBattleSpeed는 focus 중 복원값을 수정한다. 일반 focus/배치 선택/settings는 없다. | 최초복원값을 소유하는 공통 selection/time owner가 일반focus와 skill 하위선택/배치선택을 함께 처리한다. 선택중speed를 거절하고 settings pause/닫기·disconnect·terminal precedence를 연결한다. |
| RQ06 | host는 CoreApplication의 kind/owner exhaustive route와 result kind로 실제 battle payload를 추출한다. normal RequestBattleState는 authoritative snapshot을 보낸다. `world/application.rs:265`는 corrosion forced pause 시 tick/disconnect/read만 허용한다. `combat.rs:1516`은 playback 이전에 forced host elapsed를 처리한다. | 새 command/result의 routing/capability/host classification을 모두 전환한다. 기존 침식 host시계 예외와 excess delta 폐기는 유지한다. Unity 렌더링은 인수 대상이 아니다. |
| RQ07 | BattleCore/ActiveBattleSession은 controlled CloneShape와 copy-on-write candidate rollback을 쓴다. UnitTacticalState struct_shape, RuntimeUnit clone_from의 BasicAttackDef 수동 재사용, queue::PackedEvent/payload extent, runtime_facts::retained_runtime_fact_extent가 비용 경계다. Continue는 live session을 disk에 그대로 저장하지 않고 U04 pre-entry/confirmed owner를 따른다. | 새 동적 heal profile/선택 상태와 queue/event를 clone/retained/work/관측5모드에서 먼저 확인한다. 실제 크기 증가는 명시하고 과거 성능 상한/oracle은 유지한다. 새 live-battle disk 포맷은 만들지 않는다. |
| RQ08 | U05 audit002의141조건과 별도31 R06, product368/PC576/host/일반 public 소비자 근거가 있다. 운영 문서 delta 외 입력 불변을 intake에서 확인했다. 기존 world/combat 및 kernel/strict tests와 public adapter는 원형을 보존한 successor에서 사용할 수 있다. | 새6starter 데이터는 legacy numerical fixture의 입력과 구분해야 한다. frozen baseline helper를 explicit하게 유지하는 adapter가 필요한 경우 값/이유/영향을 기록하고 실제 실행한다. 새 행동과 적 저지 변경을 과거 oracle 성공으로 덮지 않는다. |

## Proposed Implementation — TD01–08; exact GEX 상세화가 남음

1. **기본행동 owner:** 기존 weapon/BasicAttackDef에 Damage 또는 Heal의 typed 표현을 둔다. Heal은 명시량(고정값/주문력 계수의 지원 범위), 단일/footprint, strict Instant delivery와 아군 선정을 제공한다. Damage 기본경로를 그대로 유지하되 순수치유는 damage snapshot/crit/overlay/proc를 만들지 않는다. BasicHealResolve는 기존 windup/epoch/next action scheduling과 fact source를 소유하고 damage 후 priority에서 공통 회복 함수를 호출한다. 큐 carrier/retained/clone/close를 함께 구현한다.
2. **active 아군 대상:** AllySingle의 automatic/player unit 선택을 strict parser/compiler/reach validator/cast/step/final direct-target validity/preview에 연결한다. 기본치유와 active의 대상 선택 의미를 혼동하지 않으며 explicit manual은 U05의 고정 대상/무효취소/무환불을 그대로 소비한다. 범위 active는 기존 PlayerTile+Allies TileArea를 재사용한다.
3. **starter grant:** initial_loadout에 active fragment 사본 ID의 명시 자리를 두고 정상 start/recruit가 공통 atomic grant/equip helper를 호출한다. baseline BasicAttackModifier에 active를 끼워 넣지 않는다. 여섯 역할은 기존6후보 identity/성격을 보존해 데이터로 배정하고 hq 채용도 대응 세트를 작성한다. 대검은 Sword 계열. 콘텐츠 계수는 GEX에 정확한 authored 초기값/관측을 기록한다.
4. **전술 상태:** 같은 UnitTacticalState에 forced defense until/generation을 소유시켜 이동/교대 성공 후 설정한다. 작업 시작은 취소하고 완료/이탈은Attack을 적용한다. 일반 태세와 강제타이머의 경계를 공통 validator/query에 둔다. swap은 기존 두-unit candidate와 World 한 번 지불을 유지한다.
5. **저지:** 기존 capacity/priority owner에 실제 거리 유지 조건과 이동 segment 접촉 처리를 추가한다. 단순 도착 후 snap이나 논리tile 덮기는 쓰지 않는다. 현재 position/block state를 query에 투영하고 range 예외는 기본 적대행동 함수에만 둔다.
6. **시간/선택:** ActiveBattleSession에 공통 선택 복원 owner를 두고 일반 focus, deployment selection, skill focus를 별도로 식별한다. 기존 skill DTO는 사용할 수 있으나 restore authority를 여러 독립 사본으로 두지 않는다. settings는 직전 effective pause/speed를 보존하는 overlay이며 disconnect의 paused는 모든 복원값에 반영한다. 새 입력을 CoreApplication/host/capability에 exhaustive 연결한다.
7. **조회:** 기본행동 종류/예상회복·유효대상과 저지관계/위치, 전술 목적지/교대 payload eligibility를 기존 validator와 공유하는 읽기 전용 query로 제공한다. preview 후 실제 command는 비용·양쪽 자격을 재검사한다. ghost/UI 단계 구조는 넣지 않는다.
8. **검증 순서:** strict loader/공통 fixture 유효성→작은 실제 clone/roundtrip/queue/extent/fault→각 기능과 normal6starter public 명령→필수 최종 Core/host/관측/public/PC/성능 통합→구현 비참여 독립 감사. 관측 없는 zero-test나 문서만으로 동작 PASS를 만들지 않는다.

## Owner·순서·실패·대안

TD01/02는 strict schema→compiled/profile→start/release→공통 HP→관측/정리 순서다. Heal을 damage_type의 가짜 값이나 음수 damage로 표현하는 대안은 적대 proc과 source accounting을 혼동하므로 기각한다. 기존 피해 표현과 기본 시전 수명은 보존한다. pureheal payload는 typed이며 지원하지 않는 delivery를 load 시 거절한다. target 소실은 공통 취소/주기 규칙을 따르며 새 적대 snapshot이나 arbitrary retarget을 만들지 않는다.
TD03은 grant/equip의 현재 전이 owner를 시작/채용 모두 사용한다. content definition만 바꾸면 실제 사본이 없으므로 불충분하다. start/recruit 후보 전체를 기존 World candidate 안에서 검증한 뒤 게시하며 실패 시 UUID/roster/bag/binding도 되돌린다. roster10과 deployed8은 서로 다른 gate다.
TD04는 stance owner의 강제 시각/epoch를 relocation/swap에서 설정하고 작업 transition에서 취소한다. 일반 stance cooldown 재활용만으로는 자동Attack 이유와 stale callback을 구별할 수 없다. movement final positions→work reconcile→forced stance for non-workers→effects/block/query 순서로 최종 상태를 일치시킨다. 두-unit swap은 한 candidate다.
TD05는 실제 이동 segment의 최초 접촉과 현재 capacity를 판정한다. backend 다음좌표를 먼저 게시한 뒤 blocker tile로 snap하는 대안은 실제 path/target/past-position 계약을 깨므로 기각한다. 유효 기존 관계를 먼저 유지하고 신규 후보를 결정 순서로 처리한다. endpoint도 판정반경 밖이면 관계를 유지하지 않는다.
TD06은 selection 복원 owner 한 개와 settings overlay를 사용한다. 독립 focus별 속도 복사본은 중첩 해제에 stale 복원을 일으키므로 기각한다. command preflight→candidate owner 변경→authoritative query→host result 게시 순서다. 종료/후퇴/불능/disconnect가 복원보다 우선한다. 기존 API의 X0.5/X3 enum 자체를 제거하는 것은 이 정책의 요청이 아니므로 보존하되 선택 중 변경 거절과 최초 restore의 기존 값 보존을 적용한다. UI 기본1/2 입력과 별도 existing elapsed fixture port는 권한을 혼동하지 않는다.
TD07은 기존 Core validator를 호출하는 조회다. UI 전용 shadow rules는 불일치 위험 때문에 기각한다. 예상 target/양/이동/교대 판정은 읽기 전용이며 실제 접수는 다시 검증한다. preview는 실행 성공 보장이 아니다.
TD08은 새 필드·dynamic payload를 작은 clone/shape/queue/rollback 입력부터 검사한다. 파서/selector/tool failure는 raw를 보존하고 제품 failure와 분리한다. 기존 fixture가 starter 변경에 묶이면 원래 수치/무기/passive 입력을 explicit helper로 보존하며 제품 새 starter grant를 검증하는 별도 정상 경로를 둔다. 자료만 갱신하거나 oracle을 완화해 통과시키지 않는다.

## Element Coverage — V01–20

각 V 번호는 같은 번호 PE-CBST의 exact acceptance 전체를 소유한다. 아래는 acceptance를 바꾸는 축약 규칙이 아니라 실행 seam과 producer/consumer/restore 매핑이다.

| Element / validation | Required source | Producer→consumer / 실행 경계 |
| --- | --- | --- |
| PE-CBST-01 / V01 전투 목표와 TP | RQ03,RQ08 | World TP command gate→accepted deployment/redeployment/swap→battle second/Pause/end;20/99/ceil and failed cost unchanged |
| PE-CBST-02 / V02 배치 자격과 동시 인원 | RQ03,RQ08 | condition→weapon/zone/occupancy→shared deployed8 and roster10 gate→query/public command; withdraw frees live slot, not participant history |
| PE-CBST-03 / V03 철수와 새 재배치 개체 | RQ03,RQ07,RQ08 | live command withdrawal→kernel cancel/released effects→fixed HP30/lock30or90→fresh new entity/profile; U01/U04/U05 restoration |
| PE-CBST-04 / V04 무기 profile과 작성 자격 | RQ01,RQ02,RQ03,RQ08 | RON WeaponProfile→strict validation→Employee runtime basic profile→fragment eligibility/attack speed→authoring view; negative fields and special gun |
| PE-CBST-05 / V05 순수 기본 치유 | RQ01,RQ02,RQ07,RQ08 | Heal schema→ally acquire→windup release→priority11 HP→fact closure/log;fullHP/death/CC/Silence/Pause, single/AoE actual outcomes and zero hostile proc |
| PE-CBST-06 / V06 여섯 회사 지급 시작 세트 | RQ01,RQ03,RQ08 | starter6/hq6 RON→normal inventory copy grant/equip→start/recruit/public catalog→six actual actives/basic actions→transfer; no baseline-active shortcut |
| PE-CBST-07 / V07 일반 태세와 방어 효과 | RQ02,RQ04,RQ08 | stance preflight→effects/block/rest→new actions versus in-flight action;general1500/CC/channel/Silence/no-op, query disabled reasons |
| PE-CBST-08 / V08 Rest와 제어·동시각 회복 | RQ02,RQ04,RQ05,RQ08 | PlatformDefense→rest eligibility/epoch→periodic common HP;CC no bank/recover full period, Ground/fieldwork/no positive resonance bonus |
| PE-CBST-09 / V09 전술 이동의 자격·보존 | RQ04,RQ08 | relocate shared preflight→same unit final position/facing→cost5/cooldown10000→work/effects/block/query;all old identity/HP/buff/use state retained |
| PE-CBST-10 / V10 이동 후 강제 방어 | RQ04,RQ07,RQ08 | move/swap→forcedDefense until/generation→1499/1500 timer→Attack;Pause/scaled time/CC/stale withdrawal/death/end/work cancellation |
| PE-CBST-11 / V11 교대 전체 원자 실행 | RQ04,RQ08 | both relocation preflights with pair occupancy exclusions→one candidate→two final bodies/oneTP5→two timers/work/block;second failure fully unchanged |
| PE-CBST-12 / V12 현장 작업과 완료 사실 | RQ04,RQ07,RQ08 | work object accumulated/generation→eligible contributor elapsed→once completion/effect→Attack;fullHP repair consumed,CC/Pause/reassignment/clone |
| PE-CBST-13 / V13 저지 위치와 유지/해제 | RQ05,RQ08 | continuous movement segment→first contact capacity priority→actual body/tile/blockrelation→range exit/reacquire;directions,fast crossing,passing,current-position/stable existing |
| PE-CBST-14 / V14 저지 상대 기본 대상 예외 | RQ02,RQ05,RQ08 | block relation→basic hostile range exception→target/usefulness/release;different tile/behind valid,skill/area/third-party/heal unchanged,release loses exception |
| PE-CBST-15 / V15 전술·행동 query와 preview | RQ03,RQ04,RQ05,RQ06,RQ08 | authoritative profile/body/tactics/work→shared preview/valid targets/reasons→normal DTO/host→latest final command;repeated query/stale preview unchanged |
| PE-CBST-16 / V16 공통 포커스와 선택 감속 | RQ06,RQ07,RQ08 | common general/deploy/skill selection→first restore owner→0.5 nesting/change/cancel/dead actor→all closed restore;speed rejection/manualPause/end precedence |
| PE-CBST-17 / V17 Pause·설정과 강제 시계 | RQ06,RQ07,RQ08 | Pause command order→settings overlay preserve/close→host tick conversion→battle timers;duplicate open/actions blocked;old corrosion forced-host elapsed exception |
| PE-CBST-18 / V18 저장·재접속·fault 소유권 | RQ06,RQ07,RQ08 | new live fields/queued carrier→controlled clone/roundtrip/fault→same actor disconnect/query→Continue pre-entry/SavePoint;no duplicate restore/resume/use or original deletion |
| PE-CBST-19 / V19 공개 작성·비용·관측의 완결 | RQ01,RQ02,RQ06,RQ07,RQ08 | actual authoring loader/compile/catalog/CoreApplication/host and size/clone/queue/work/observation/resource failure→required negative/public/native evidence |
| PE-CBST-20 / V20 검증·독립 감사와 후속 경계 | RQ08 | 141 inherited+31 R06 input impact→all20 current acceptance/10dimensions/headless public scenario→source archive/currentness→nonparticipant CompleteHigh audit and corrected findings |

## Findings / Unknowns

- IF01 implementation: pure basic heal, AllySingle and normal active copy grant are absent. TD01–03, V04–06/19 own correction; this does not reopen policy meaning.
- IF02 implementation: relocation Defense-only, channel stance gate, work exit Defense and missing forced timer differ from decided policy. TD04, V07–12 own correction.
- IF03 implementation: blocking lacks first-contact sweep/actual-distance maintenance/basic-only range exception. TD05, V13–14 own correction.
- IF04 implementation: selection/settings/common restore/query payload and live8 gate are incomplete. TD03/06/07, V02/15–18 own correction.
- TD01–08 technical decisions: explicit owner/order/cutover/failure and rejected alternatives above. Exact Rust names/container layout/authored coefficients/queue ordering details are bounded GEX work; numerical gameplay defaults must be documented and measured.
- ED01 evidence limitation: this review is static local workspace evidence. U05 Complete/High plus proven operating-document delta is inherited; no new runtime success, remote deployment or Unity rendering claim is made.

No new material policy finding or required source gap remains. Required8 groups cover20 elements; Released/Deployed Actual are explicitly N/A with local host still required. Review complete publication is conditional on exact sources and canonical refs passing the builder checks. Root alone implements; independent GPT-6 Astra/High audits only after self-validation.
