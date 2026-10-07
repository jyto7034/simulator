# U07 P2G-20 — 독립 생성물·진화·분절 상태 코드 검토

입력 WR-CIOB-002의20개 정책 의미와 acceptance를 그대로 검토한다. Candidate source plan001→002→003은 역방향 소비자 조사에서 추가된 경계를 보존한다. 최종 required RQ01–08/N·A RQ09–10과146개 exact 파일은 `evidence/REVIEW_SOURCE_MANIFEST_003.json`·archive에 결속한다. 파일 수는 capture 집합이며 모든 파일의 모든 행을 읽었다는 뜻이 아니다. 아래 owner/producer/consumer의 bounded sed/rg 정역 조사를 근거로 한다. U06 최종 product `b33a8a019087f3b6766c8cf611636f46246fa3d895464b5a439f6d176694a401`과 현황 delta 인수는 PREDECESSOR_INTAKE_001이 소유한다. 이번 검토는 정적 분석이며 새로운 실행 성공/제품 완료 주장이 아니다.

## Current Actual RQ01–08

| 요구 | 실제 owner·생산/소비 경로 | 구현 차이 |
|---|---|---|
| RQ01 | `data/summoned_unit_data.rs:47–145`는 definitions에 양의 HP/이동/반경과 필수 boss behavior를 갖는 actor만 제공한다. compiled DB는 boss_runtime_data 내부이며 live_ron의 summoned compile→boss compile→skill refs로 발행한다. raw.rs의 CatalogKind/DefinitionKind와 authoring/definition.rs는 폐쇄 집합이다. ability.rs SkillEffectDef에는 범용 생성 opcode가 없다. | 정상 catalog에 별도 object 정의를 추가하고 typed create/evolve consumer·strict kind/reference/수치 검증·compiled/public 설명을 함께 연결해야 한다. 기존 boss 소환 정의 의미를 기본값으로 바꾸지 않는다. |
| RQ01 | skills/base.ron의 laetitia_surprise_gift는 projectile/poison/area NPC 공격이다. fragments/base.ron의 래티씨아는 기본공격 modifier placeholder다. value_expr/catalog.rs:135–163은 amount 효과만 컴파일하고 나머지 op를 명시 제외한다. | 친구 기본/개화의 실제 PlayerTile 스킬과 별도 얼음벽·고기벽 소비를 작성한다. 새 op를 값표현식 효과로 가장하지 않으며 공통 결과/provenance 분기를 갱신한다. |
| RQ02 | spawn_kernel.rs:607–704는 payload identity 검사→정규화→request reserve→candidate clone→전체배치→commit이다. runtime fault는 도메인 거절과 다르다. summon_runtime.rs:303–515는 같은 kernel을 사용하지만 group/unit에 Opponent를 지정하고 boss route/behavior adapter를 연결한다. | 새 source kind와 객체 점유를 같은 kernel에 연결한다. Side는 실제 source에서 얻으며 boss encounter 없이도 actor가 행동해야 한다. 비용과 mutation은 candidate 안에서 함께 처리한다. |
| RQ02 | sim.rs:1250–1380 manual cast는 대상/공명/사용잠금/sustained start를 검증한다. PlayerTile은 target usefulness를 생략한다. skill_runtime/cast.rs:694는 CastTarget 중 Unit만 목록에 넣는다. sim.rs:3352의 ordered effects가 effect별 source/result를 소유한다. | 생성은 tile을 unit target loop로 변환하지 않고 cast anchor를 명시 소비한다. 접수 전 공통 생성 preflight와 실행 시 재검증을 추가한다. empty target list를 생성 성공0개로 삼지 않는다. |
| RQ03 | RuntimeUnit role은 Combatant/DefenseObject이고 can_move/can_basic_attack은 combatant를 검사한다. movement/planner.rs:40–66은 player FixedDefense/opponent route를 선택한다. blocking.rs:271–287의 protected target fallback은 DefenseObject를 고른다. | 독립 actor 이동 capability와 wall HP adapter role을 구분한다. wall을 DefenseObject로 재사용하면 승패 목표에 혼입되므로 금지한다. 함정은 별도 object 상태이며 RuntimeUnit이 필요 없다. |
| RQ03 | engine.rs:419–426은 backend 결과에 U06 contact clamp를 적용한 뒤 body/segment/block을 갱신한다. 기존 blocking은 직원 capacity와 지상 enemy 관계다. targeting/basic_heal은 active·same-side·missing HP/양의 heal/range로 치유 대상을 정한다. | 벽 접촉은 직원 저지 용량과 별도 관계로 실제 segment에서 제약한다. 원래 route를 유지하고 지상 적이 벽을 공격하게 한다. 공중/일반치유/직원명령/시설목표에서 wall을 명시 제외한다. |
| RQ03 | skill_runtime/area.rs는 고정 tile projection, source snapshot, TTL/tick을 이미 소유한다. sim.rs:2488–2603은 같은 timestamp refill을 모두 처리한 뒤 terminal outcome을 계산한다. | field는 existing TileArea executor를 사용하고 query/source를 연결한다. object wake는 stable ID/epoch와 기존 timestamp ordering을 사용하며 terminal 판단 전에 정산한다. |
| RQ04 | sustained.rs:1361–1448은 mitigated 피해를 stable personal/shared priority→activation 순서로 단일 remaining pool에서 흡수한다. damage_runtime.rs:450 이후 잔여만 HP-loss/reflection로 간다. RechargeShield는 caster+source_skill의 최근 activation을 찾는다. | segmented descriptor와 명시 pool identity를 기존 shield owner에 추가한다. stack은 ceil(remaining/H) 파생값이다. 특정 pool 분리와 HP fallback을 재충전의 최근 activation 추정으로 구현하지 않는다. block bonus는 실제 remaining에서 파생한다. |
| RQ04 | HealthCost는 protection을 거치지 않는 비용 fact/HP 변경 경로이며 기존 lethal opt-in이 있다. commands.rs:935–995는 stat→HP→prepared death 순서를 지킨다. stats.rs:275–293은 max 증가에 회복하지 않고 감소에 clamp한다. stat_pipeline은 전투 진입 profile/장비값 조합이고 live mutation owner가 아니다. | wall 생성비용은 nonlethal preflight→같은 candidate에서 HealthCost 또는 partial segment detach→벽 생성이다. wall HP0은 object destruction으로 분리해 직원 death/kill/corpse proc를 만들지 않는다. 기존 unit maxHP ordering을 재사용한다. |
| RQ05 | RuntimeFactSource::observation은 delivery/finite-child/resource 권한을 제거한다. queue는 소유한 source/lease를 dispatch 완료에 닫는다. BattleCore에는 수동 CloneShape·재사용 clone·materialized count·retained fact 계산·fingerprint가 있다. value_expr_runtime은 released capture와 effect result accumulator를 따로 소유한다. | object observation source를 부모 실행 권한으로 사용하지 않는다. 독립 actor의 후속 action과 trap 1회 공격은 자신의 실제 실행 수명으로 시작한다. 새 owner를 모든 clone/count/retained/queue 경계에 추가하고 allocation 전 admission한다. 생성/흡수는 가짜 damage/heal result를 만들지 않는다. |
| RQ06 | world/combat.rs:2767–2828 preview는 validate_manual_skill_activation을 호출하고 mutation하지 않는다. state.rs:1470–1540은 active units와 sustained effects를 checkpoint에 투영한다. host handlers는 CoreApplication 결과/재접속 projection을 직렬화한다. authoring/trial.rs:546 이후 정상 application candidate와 실제 ready receipt를 사용한다. | Core의 공통 생성 eligibility/cost/tiles와 objects/pool query를 DTO로 투영한다. query-only로 원자 비용을 수행하지 않는다. normal ActivateSkill·trial·host roundtrip의 nonzero 결과를 필수 검증한다. |
| RQ07 | active_run은 V12 checkpoint, continuation은 V11 Continue envelope와 checksum/검증을 소유한다. 같은 actor 재접속은 현재 battle projection이며 disconnect pause를 유지한다. Continue는 기존 미확정 전투 입장 전 경계다. | live object/epoch/source/cost 상태는 clone/rollback에 포함한다. authored catalog의 checkpoint/fixture roundtrip reader와 writer를 함께 갱신하고 optional absent 직렬화가 이전 checksum을 불필요하게 바꾸지 않게 한다. live battle disk 저장은 새로 약속하지 않는다. |
| RQ08 | U06 final audit002는 Complete/High, 정확한 source/condition/dimension/31R06 재사용과 fresh Core1794/1794/1803·host92/100+heap1을 확인했다. 기존 spawn_kernel/summon tests는 atomic/idempotence/limits/TTL/Persist·Despawn과 no-victory를 검증한다. commands maxHP decision table, U06 heal·movement·host tests도 있다. | 선행161조건+별도31R06을 exact 의미로 상속하고 새20조건/10차원을 매핑한다. 새 layout/enum/executor가 넓은 영향을 주므로 최종 full suite·public RON/trial·native costs/fault/allocator와 affected persistence는 현재 실행한다. 각 불변 경계만 exact delta 근거로 재사용한다. |

## Proposed Implementation / Technical Decisions

### TD01 — 작성 정의와 capability

`SummonedUnitCatalogData`의 기존 definitions 옆에 default-empty/skip-empty 독립 object collection을 추가하는 방향을 채택한다. catalog 파일과 resolver/source identity를 재사용하면서 DefinitionKind는 actor/trap/wall 정의를 명시 식별한다. kind별 enum으로 필요 없는 HP/이동/기본공격을 요구하지 않는다. actor combat profile은 기존 컴파일러/validator를 재사용하고 stationary/path 행동은 명시한다. fixed field는 이미 있는 유한 TileArea를 정상 공개 포트로 사용한다. strict cross-reference와 evolution compatible kind/HP-capability, 양수 period/TTL/capacity·상한·overflow 검증을 발행 전 수행한다. 새 capability는 실제 executor가 완성된 뒤에만 광고한다. 모든 생성물을 필수 BossActor로 만드는 대안은 권한/정지 의미가 달라 배제한다. 정확한 Rust 타입/조절값은 GEX에서 기록한다.

### TD02 — 접수·생성·비용 transaction

SkillEffectDef의 typed create/evolve는 accepted cast tile·source occurrence·authored effect identity를 사용한다. 기존 SpawnKernel에 source adapter를 추가하고 객체 occupants도 정규화에 포함한다. occurrence stable request/payload identity로 중복 settlement를 재사용한다. 순서는 definition/source/placement/limits/cost preflight→candidate→비용·ID·spawn/epoch·source·queue/fact→commit이다. 도메인 거절은 명시 결과이며 budget/control fault는 sticky fault로 전파한다. 접수 당시 공명은 U05 경계에 속해 나중 domain rejection에서 환불하지 않는다. TileArea의 적 unit loop에 생성 op를 넣는 대안은 빈 타일과 중복생성이 잘못되어 배제한다.

### TD03 — object owner와 actor/HP adapter

BattleCore의 독립 상태 owner는 logical object ID, definition/form, source/copy family, epoch, TTL, bounded active counters와 자신의 source snapshot을 갖는다. trap은 이 map만 사용한다. actor는 기존 RuntimeUnit의 전투 행동/피해 executor를 adapter로 사용하되 employee roster/tactics/required wave와 별개다. wall도 기존 HP/damage 계산을 사용하기 위한 명시 role/source adapter를 둘 수 있지만 일반 combatant·시설 목표·회복/죽음 proc 자격을 부여하지 않는다. wall HP는 adapter의 한 상태만 truth로 사용하고 map에 이중 balance를 저장하지 않는다. 전용 wall 범위 예외는 실제 접촉 상대 basic damage에만 적용한다. 지상 segment의 최초 벽 접촉을 계산→정지/공격→제거 후 원래 path 재개하며 backend obstacle로 길을 재탐색하지 않는다.

### TD04 — 친구·진화·유한 field

친구는 source Attack 포함 피해 snapshot과 authored trigger/attack footprint를 갖고 실제 적격 target이 있을 때만 one-shot ability damage를 실행한다. stable target order, effect-result 성공 후 소모, no BasicAttack proc를 적용한다. 같은 source/copy 작은 친구만 같은 logical ID로 진화하며 새 source payload·형태 TTL·epoch를 교체한다. 이전 callback/link는 epoch mismatch로 무해하다. HP형 전환은 old HP를 new max로 clamp하고 HP-capability 혼합은 strict 거절한다. field는 기존 fixed TileArea source/lease/tick/TTL을 그대로 실행하여 이중 tick owner를 만들지 않는다. 원래 NPC 감염 스킬을 재명명하는 대안은 공식 파편 동작과 달라 배제한다.

### TD05 — 분절 pool·detach·fallback

기존 sustained shield record에 optional segmented metadata/pool ID를 결속한다. remaining만 truth이고 stack/top 계산은 공통 pure helper다. 일반 shield priority·감쇄→흡수→초과HP 순서는 유지하며 segmented block bonus는 remaining 변화 후 즉시 재평가한다. detach는 자기 명시 pool ID·source/copy를 확인해 최상단 partial HP를 제거하고 같은 candidate에 벽HP로 이전한다. 없으면 현재 maxHP의 authored percent/양을 계산해 nonlethal HealthCost를 사용한다. 비용0/overflow/lethal/잘못된 pool reference는 명시 거절한다. 기존 Heal/Regen/Lifesteal은 pool writer가 아니며 일반 RechargeShield와 segmented 생산/보충 규칙을 모호하게 합치지 않는다. full 감염/시체 생산자는 U18/U21 소유로 유지한다.

### TD06 — 독립 provenance·종료와 자원

관측 source는 actual actor/copy/authored site를 보존하되 부모 delivery/action/resource lease를 실행 권한으로 복사하지 않는다. 생성 직후 부모 action의 유한 obligation을 child TTL과 분리한다. actor 후속 action은 실제 actor의 새로운 occurrence다. trap/field의 필요한 private lifetime은 방출 시 명시 소유하고 부모 제거 때 Persist/Despawn 정의에 따라 처리한다. 첫 종료 원인이 active record를 제거하고 stale callbacks는 재결과를 만들지 않는다. HP0 destruction과 Despawn/TTL/consumed 원인은 구분한다. terminal/withdraw/reset에서 active state·queue·leases를 정리하고 bounded fact/log만 남긴다. 모든 owner를 CloneShape/retained/entity/work/allocator에 연결하고 예산 실패를 무관측 성공으로 숨기지 않는다.

### TD07 — query·clock·restore

공통 read-only eligibility가 accepted input과 preview를 함께 판정한다. DTO에 kind/form/source/copy/HP/remaining/derived stack/TTL/epoch와 생성 거절/비용/타일을 제공하고 실제 종료는 bounded result/log로 제공한다. settings/selection/Pause는 U06 owner를 재사용한다. 같은 timestamp bucket 내부와 refill을 다 처리한 후 victory 판정하는 순서를 유지한다. live clone·fault rollback은 ID/source/epoch/counters/queue를 독립 복사하며 same-process reconnect는 pause 상태로 투영한다. checkpoint의 catalog/compiled reader-writer만 필요한 만큼 전환하고 Continue/SavePoint 의미와 기존 원본보호를 보존한다.

### TD08 — 검증·cutover·유효 증거

초기 작은 native layout/strict input/clone/atomic fault 검사→actor/trap/wall/pool focused→정상 CoreApplication/authoring/host→최종 integrated/native public cost 순으로 진행한다. catalog counts·serde match·CloneShape 등 의도된 변화는 원본 oracle/입력을 보존하고 명시적 후속 비교로만 갱신한다. PC 숫자·bounds는 완화하지 않는다. R05의 상속 peak-only 경고와20분stress 제외 외 새 예외를 만들지 않는다. old boss summon, old shield/HealthCost, actual HP-loss/kill, heal, movement, resource/lifetime·restore가 영향 회귀다. 공식 data cutover는 실제 runtime 지원 이후 수행한다. 원자적 single_phase Goal로 전체 U07을 감사하고 그 이전 U08 종속 구현은 금지한다.

## Element Coverage / Validation Seams

| 정책 | 실제 검증할 producer→consumer와 경계 |
|---|---|
| 01 | 정상 외부 same opcode→양 진영 actor/trap/wall→공격/HP; no employee slot/required wave/reward/compendium authority |
| 02 | same request duplicate/conflict→kernel settlement; late placement/HP/budget fault→전체 ID/HP/pool/source rollback→retry |
| 03 | ground path/spawn/goal/portal/static/occupied/range eligibility; own small evolution 예외와 source A/B·exact/one-over counts |
| 04 | ally/enemy stationary/path actors→적 target/실제 attack·skill; parent removed and no manual employee/roster authority |
| 05 | normal fixed TileArea→positive period/finite TTL/Pause/parent withdrawal→정확한 tick, no HP/occupancy |
| 06 | 공식 기본/개화 fragment→small single/big area→actual damage·성공 소모; no target/air/lost target/동시 후보 |
| 07 | 두 시전/같은 ID/1slot·공명각1회→epoch/TTL reset; third/foreign reject; HP clamp/no heal/clone independence |
| 08 | created source/copy snapshot→Persist/Despawn removal→실제 attribution; parent use lock 종료·재충전, replacement copy 분리 |
| 09 | simultaneous TTL/HP0/consume/remove→first cause once; terminal cleanup/반복 retained·queue 회수 |
| 10 | H100/P150→2stack/top50/pool100/wall50; P100/1/0·multi-break/overkill·block release·no false HP-loss/proc |
| 11 | empty pool→nonlethal HP cost/snapshot max/변환 HP; shields/parry/HP1 bypass; late failure rollback/no resonance preaccept |
| 12 | actual moving ground segment→wall stop/basic damage/destroy/path resume; air passes·no heal/corpse/refund; official ice/flesh consumer |
| 13 | maxHP increase/decrease/0→stat/HP/death-or-destroy order·attribution; direct Kill/special kill/HealthCost regressions |
| 14 | Pause accept→space/HP changed→release rejection without new cost/refund; CC cancellation before/after release and U05 unlock |
| 15 | repeated shared preview/eligible tiles/cost→full fingerprint unchanged; stale command reject/current projection·host serialization |
| 16 | Pause/0.5/settings/host forced freeze·timestamp TTL/damage/evolve/victory→deterministic same seed across order/log modes |
| 17 | live clone/rollback/source/epoch/pool/timer→independence; same actor reconnect; fresh process Continue pre-entry/SavePoint/original protection |
| 18 | strict public pack parse→compiled explain→normal trial/API→nonzero actor/trap/field/wall/evolution/pool/cost; unsupported precise errors |
| 19 | new owner admission/retained/clone/fuel→exact/one-under·five fault modes/observer/allocator/PC/R05 current costs with original oracle |
| 20 | new20+inherited161+separate31R06·10 dimensions→current records/freeze→nonparticipant Astra High whole-goal Complete/High and open0 |

## Findings / Unknowns

- IF01: generic skill creation, separate object schema/owner and actor-side adapter are missing. TD01–03 own the implementation; existing boss summon cannot alone satisfy U07.
- IF02: trap/evolution/wall contact and separate lifecycle/query consumers are missing. TD03/04/06/07 own them; wall heal/death/goal mixing must be explicitly prevented.
- IF03: shield has no segmented identity/detach/derived blocking and creation-cost transaction. TD02/05 own them; recent source_skill recharge is insufficient to choose a pool.
- IF04: new state is not yet in clone/native accounting/persistence projections/public capability and final validation. TD06–08 own the implementation and evidence.
- ED01: source capture and static findings do not prove runtime completion. Old passing tests remain predecessor evidence only; new execution is deferred to GEX.

No material policy change is required by these findings. All20 accepted meanings are implementable within delegated scope; source-required gaps are closed at the above owner boundaries. Concrete container/type details and authored initial balance values remain implementation decisions inside those meanings, not new user decisions. RQ09 Released Actual and RQ10 Deployed Actual are N/A because no external released compatibility/deployed Unity rollout is in scope; local release tooling/host source and final local execution remain required.
