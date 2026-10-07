# U08 PUBLIC 잔여 설계·실행 계약 — 2026-10-06/001

상태: GEX-10 계속. 초기 PC와 numeric 경계 통과는 전체 U08 완료가 아니다.
root Astra/high 계획·조정, 기존 u07_implementation Sol/xhigh 제품·실행, u08_test_author Astra/high 시험·oracle 역할을 유지한다.
최종 감사는 이 작업에 참여하지 않은 별도 Astra/high가 맡는다. 현재 제품/시험/도구 writer는 모두 HOLD이며 실제 API 합의 후 파일별로 인계한다.

## 현재 유효 증거와 보존

인수 source409/6ea1c1d78b9f495cf72040cf8f61ddf1cd2cceb256f43f7b87cebcf5f107f2cb.
초기 PC 576행/36조건,31성공/5기존 typed거절. gameplay와 immediate U07-M 비시간 항목 exact, 허용 비용 delta0.
17개 negative control 모두 거절. historical U06 대비 owner+200은 U07의 IndependentState152+OptionalPlacement48이며 U08 신규 비용이 아니다.
시간 제외는 setup.elapsed_ns/measurement.elapsed_ns만이다. PC parent receipt와 strict proof를 EVIDENCE.json에 exact 연결한다.
focused004 33/33, off001 31/31 source409/eca6c373… 이후 차이는 host observability/tests.rs 한 파일의 raw drain 교정뿐이므로 numeric 비영향 재사용.
relic_host_layout001과 relic_host_heap001은 현재409/6ea1c1…에서 PASS. 최초 fixture/기대값 실패 원본과 과거 Sol 시험 작성 이력 보존.
기능 실행 종료 후 PC를 수행했고 owned session 종료 확인. process 조회는 sandbox namespace 한계가 있다.

## A. 실제 loadout 기반 prospective query와 effective profile

목적: 유물의 active/dormant 수치 기여를 정상 CoreApplication query로 설명한다.
RequestRelicNumericPlan{employee_uuid} 형태의 read-only 행동과 별도 prospective DTO를 추가한다. 정확 이름/field는 구현자·시험자 합의 부록으로 고정한다.
실제 validated loadout/catalog generation과 copyUUID/effectID 정렬, variant, typed SkillDamage(site) 또는 prospective BasicAttackDamage를 표시한다.
실행용 RelicNumericTarget의 actor/attack_epoch를 가짜 값으로 채우지 않고 private RuntimeFactBinding/capability를 발행하지 않는다.
query는 UUID/RNG/save/action/epoch를 바꾸지 않는다. PreviewSkillTarget 기존 의미도 유지한다.
selector는 실행과 같은 판정을 재사용하되 실제 accepted provider/site만 후보로 삼는다. 현재 직원의 모든 weapon/fragment를 각 source에 뭉뚱그려 전달하면 안 된다.
basic은 실제 weapon, fragment 없음, 고유 empty tags를 사용하며 다른 skill tags를 빌리지 않는다. selector의 조건은 같은 효과 안에서 AND다.
누락 문맥은 unresolved reason이며 fake0이 아니다. 외부 문맥 없는 compiled amount는 catalog::can_evaluate_without_external_context와 공통 evaluate_compiled_amount를 활용할 수 있다.
Snapshot/Live 시점을 prospective 값으로 확정하지 않는다. dormant reason과 계산 가능한 base/effective, unresolved/fault를 구별한다.
world/snapshot.rs의 effective profile은 combat_setup/player_spawns.rs BattleUnitDraft.combat_profile 빌드 흐름을 재사용해 유물 Defense/MR penalty를 포함한다.
certified profile/power와 persistent stats는 유지한다. 전체 전투 생성으로 query를 구현하지 않는다.

## B. authored Event 무료 선택·거절·재시도·복원

EventSceneNext::RelicCorruption{next_scene_id}를 추가하고 기존 DAG cycle/target/reachability 검증에 연결한다. EmployeeCheck 최대1은 기존대로 유지하되 새 selection 장면 수를1로 제한하지 않는다.
선택 command는 node/event/sourceScene과 Option<copyUUID>를 받아 실제 현재 장면/선택 권한을 확인한다. None은 거절이다.
cost0/nonroll이며 preview나 client free flag는 권한이 아니다. 일반 catalyst 오염은 기존 cost1 흐름을 유지한다.
EventSession에 skip-None Option<Vec<CommittedRelicCorruptionSelection>>를 둔다. entry는 source_scene_id/copy_uuid Option을 보유하며 실제 선택된 장면 수가 자연 상한이다.
variant 변경+receipt 추가+scene advance는 원 staged transaction에서 atomic하게 commit한다. 거절도 선택 권한을 소비한다.
같은 역사 sourceScene+같은 payload 재시도는 현재 state를 반환한다. 재소비/target재검증/scene rewind를 하지 않는다. 다른 payload retry는 거절한다.
복원은 Some(empty), 중복 scene, 중복 Some copy, 없는/비선택 scene, 잘못된 순서/branch/current path를 거절한다.
entry에서 current까지 기존 committed_check outcome과 selection receipts를 함께 따라 실제 선택 경로를 검증한다. 기존 node_flow에는 이 검증이 없으므로 실제 구현한다.
역사 선택 copy가 현재 있으면 Corrupt여야 한다. 나중에 정상 dismantle한 absent copy는 허용한다. receipt는 현재 copy 존재를 영구 요구하지 않는다.
Decline-only는 copy/material/RELICCOP counter 없이도 새 heap owner다. has_relic_owner, next-op seed, retained extent, CloneShape에 포함한다.
active session/run-map checkpoint/entry/Continue 독립 owner를 각각 계량하고 공유 Arc는 deep-count를 반복하지 않는다.
Vec/string allocation 이전 admission, checked 합산, sticky typed failure/rollback을 보존한다. ContinueRelicCheckpointHint는 decode allocation 전에 receipt 존재를 감지한다.
None인 old VERSION11 canonical checksum을 보존한다. durable validator/restore/clone 모두 같은 상태 의미를 사용한다.

## C. 판매와 수명주기

정상 shop의 qty1 유물 Sell은 normal/corrupt·equipped/unequipped 모두 명시 RelicNotSellable로 거절한다. unknown UUID의 기존 OfferUnavailable은 유지한다.
판매불가 표시는 derived snapshot이며 persistent OwnedRelic에 중복 권한을 넣지 않는다. Shop buy/pool/U16 범위는 추가하지 않는다.
죽음은 world/application.rs의 postbattle grant 뒤 reconcile_relic_loadouts 흐름을 활용한다. copy/variant를 보존하고 dead employee slot만 idempotent 회수한다.
normal postbattle, withdrawal, living redeploy, Continue/reset의 실제 경로 시험을 남긴다. private helper만으로 완료하지 않는다.
StartNewGame NotStarted gate와 AbandonRun GameCoreState::new root reset을 존중하고 inline clear 부재만으로 새 복구 로직을 추가하지 않는다.

## 파일 소유·순서·완료 기준

Sol 제품 후보: game/relic.rs 및 신규 query 모듈, world/relic_runtime.rs, behavior.rs, application.rs, world.rs, world/application.rs, world/action_capability.rs,
world/snapshot.rs, combat_setup/player_spawns.rs, data/event_data.rs, resources/selection.rs, world/event_node.rs, world/node_flow.rs, world/shop.rs,
필요한 active_run/continuation·checkpoint·clone/native owner plumbing. inline 시험은 기존 bytes를 유지하고 시험 교정은 Astra에게 반환한다.
Astra는 별도 tests 파일/fixture/probe/comparator를 소유한다. 모듈 등록을 위해 제품 파일을 편집할 때도 전체 파일 lock을 순차 인계한다.
이전 역할기록의 value_expr_runtime/tests.rs 예시는 잘못된 locator다. 실제 등록 위치는 core/value_expr_runtime.rs의 inline cfg(test)이며 과거 record는 소급 수정하지 않는다.
1) 공개 API/상태 의미 합의 2) 제품 파일/분리 시험 파일 작성 3) 전 writer HOLD 4) Sol fmt/작은 compile/집중 검증 5) receipt 회수·실패 분류 6) 담당 교정과 새label.
기능과 성능은 겹치지 않는다. 테스트 통과로 전체 설계의 타당성 또는 U08 완료를 대신하지 않는다.
이후 실제 newRON/CLI/CoreApplication 공개 소비, queued/projectile/area 지연 소비, 전체 영향 통합/R05/current PC 결속과 191+31 선행조건을 증거에 연결한다.
evidence selection→predecessor impact→adaptation current binding→coverage→자체검증→PLAN/NOTES→exact freeze/정역복원→독립감사 순서다.
기존 정책16군/10차원과 승인 범위는 유지한다. API 의미/owner/완료기준 변경은 root에 근거와 대안을 반환하고 일반 내부 선택은 담당자가 자율 처리한다.
