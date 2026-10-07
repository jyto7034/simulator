# R04 code-informed review — current facts와 구현 인계

2026-09-11 KST. Input `WR-CARF-P2G20-002`, source plan `RSP-CARF-P2G20-004`.
이 body는 P2G-20의 static review다. 아래 제안은 아직 구현된 API가 아니다. 승인 target/20개 acceptance의 canonical owner는 [P2G-10](P2G_10_POLICY_001.json)의 5 decided revisions와 Decisions이며 본문은 이를 변경하지 않는다.

## 1. 조사 범위와 현재성

[Source evidence](P2G_20_SOURCE_EVIDENCE_001.json)는 49개 파일의 raw hash, 실제 읽은 행 범위, 6개 caller/constructor 검색과 관련 dirty를 보존한다. 전체 파일 hash는 전체 행의 의미 검토를 뜻하지 않는다. HEAD는 `f318414fb0156a67e04c5479000c2a2683fc0c09`, 실제 기준은 보존된 dirty workspace다. Plan001의 추정 locator는 관찰 전 [correction](P2G_20_PLAN_CORRECTION_001.json)으로 Plan002에 교정했으며 predecessor를 덮어쓰지 않았다.

R03 actual audit002의 inputs119 318개, audit outputs 101개와 release binary 3개가 그대로임을 확인했다. 실제 과거 full 1327 tests, authoring focused 59 tests 및 12개 selected build/API/CLI/cost/source receipts를 source basis와 함께 재사용한다. P2G에서 새 runtime test를 실행하지 않았고 R04 acceptance PASS를 주장하지 않는다. R01/R02는 R03에서 실제 검증한 non-impact 관계를 인수했으며 모든 과거 파일이 unchanged라고 주장하지 않는다. GEX에서 source가 바뀌면 affected predecessor와 direct consumer를 재검증한다. 이전 실패 aggregate를 편집하거나 selected PASS로 전체 guard 성공을 대체하지 않는다.

RQ01–07은 required workspace 경계다. RQ08 외부 server/Unity/transport와 RQ09 released/deployed actual은 승인 Scope O01/O05와 AUTORUN §§1/3/9에 따라 N/A다. Local release 검증은 외부 배포 호환 증거가 아니다. 개인 저장 내용, 미래 Flow/boss adapter와 이후 R 전체 구현은 조사 완료 claim에 포함하지 않는다.

## 2. Current Actual

### B01 — mutation과 reaction 순서 (RQ01/02)

`damage_runtime.rs:1–649`의 committed damage는 BasicAttack만 기존 OnAttack/OnHit activation plan을 수식 전에 모으고 실제 HP loss를 계산한다. 기본·추가 damage component의 합은 한 기본공격 발생에 속한다. HP 적용·사망 정리 뒤 준비한 반응 batch가 실행된다. 실제 손실 기반 gate, 공명 획득, RNG/기존 log 순번은 현재 의미다. 일반 skill damage를 BasicAttack trigger로 바꾸면 안 된다.

`commands.rs:246–420,538–700`의 Heal은 signed flat/max-HP-percent를 계산·clamp한다. 음수 Heal은 기존 HP 감소와 actual damage summary에 반영되며 HP Cost가 아니다. Rest/field work도 실제 HP mutation 경로를 사용한다. stat modifier에 따른 maxHP 변화가 lethal이면 `record_runtime_stat_hp_change`가 처리한다. 초기화/재배치의 HP 설정을 Heal로 새로 기록할 이유는 없다.

`death_runtime.rs:1–215`는 movement 정리, wave epoch 처리, buff 제거, graveyard/log 기록, OnDeath/OnKill/정렬된 OnAllyDeath 반응과 reconcile 경계를 가진다. `summon_runtime.rs:610–680`의 Kill은 직접 death finalization에 들어가며 Despawn은 withdraw/Despawned 경로다. Damage만 계측하면 직접 kill 사실을 빠뜨린다. optional proc 억제가 필수 death/wave 처리까지 건너뛰어서는 안 된다.

Cost producer는 `sim.rs:2300–2445`의 Auto/ManualCastEnd 공명 소비와 `world/combat.rs:1925–1993,2334–2374,2490–2531`의 deploy/relocation/swap tactical resource pay다. `world/state.rs:651–729`는 pay 자격과 차감을 소유한다. 음수 ModifyResonance/ModifyTacticalResource는 임의 resource change로서 자동으로 Cost가 되지 않는다.

### B02 — provenance와 occurrence (RQ02/04)

`skill_source.rs:1–35`는 skill ID/caster/cast sequence/ability log sequence/cause/proc origin을 보유한다. `ability/authoring.rs:1–56`과 `skill_data.rs:17–109,175–223`는 실제 effect file/site/field/origin을 제공한다. `process_skill_commands`는 batch 실행에 recording과 authoring source를 설정하지만 nested/legacy trigger batch는 modifier unit만 남는 지점이 있다. Queue source도 authoring 수집 여부에 연결돼 있어 gameplay 사실의 완전한 권위로 쓸 수 없다.

`triggers.rs:1–355`, `queue.rs:86–150`, `sim.rs`의 BuffApply/BuffTick/기본공격 예약과 현재 skill projectile/area를 함께 전환해야 한다. 기존 queue ordering은 BattleEvent를 기준으로 하며 출처를 추가한다고 정렬·RNG를 바꾸지 않는다. BuffTick의 실제 예약 producer와 지속 status source를 보존할 수 있으나 여러 stack contributor의 true join/root 정책까지 승인된 것은 아니다.

### B03 — 현재 action lifetime (RQ03/05)

`sim.rs:1690–1814,1858–2182`의 manual request는 target 검증 뒤 focus/예약 상태를 만들고 CastEnd에서 별도 cast sequence로 invoke한다. pending identity에 log sequence를 사용하는 부분이 있다. `skill_runtime/cast.rs:1–320`의 제어 상태는 모든 step이 끝나고 관련 area/deferred 상태가 없어지면 제거된다. 그러나 withdraw 뒤 독립 projectile/GroundFixed area는 보존한 source로 실행할 수 있다. 제어 상태의 존재만으로 모든 효과 정산을 판정할 수 없다.

HardCC는 pending/basic windup을 중단하지만 방출 뒤 예약 skill step의 기존 동작을 일괄 취소하지 않는다(`core/mod.rs:6407–6508`). Silence는 새 cast 자격을 막으며 pending을 일괄 취소하지 않는다. InterruptCast는 현재 pending-only/no-refund다. Withdrawal은 pending/active cast와 caster-owned Follow area를 정리하고 released projectile/GroundFixed를 보존한다. Death는 movement/lifecycle/wave/buff/graveyard/reaction을 처리하지만 pending/active cast/Follow area를 일괄 제거하지 않는다(`death_runtime.rs:88–170`, `movement/lifecycle.rs:8–30`). Matching pending CastEnd가 death 후에도 호출돼, dead-caster invocation이 no-op인 뒤 recovery/resonance reset이 실행될 수 있다(`sim.rs:2998–3160`, `407–425`, `1110–1136`). 이 실제 Cost와 failed/no-release 관찰을 구별하며 OnDeath allow-dead invocation은 별도 자격을 보존한다. 하나의 무차별 취소 규칙으로 합칠 수 없다.

### B04 — 실제 장비 사본 (RQ04)

`resources/item_slot.rs:1–39`, `inventory.rs:297–358,579–617`는 실제 inventory instance UUID와 장착 참조/소유 검증을 가진다. `player_spawns.rs:125–210,232–278`의 두 player draft 경로가 이를 definition UUID 목록으로 축소한다. `core/build.rs:120–166,252–348,435–537`의 두 RuntimeItem 생성 경로는 base UUID/index 기반 runtime ID를 만든다. 이 ID는 RNG/cooldown의 현재 identity로서 보존해야 한다. `stat_pipeline.rs:100–166`은 definition lookup을 소비한다. `skill_fragment.rs:1–65`는 종류별 수량/진행도이며 존재하지 않는 fragment copy ID를 만들 수 없다.

### B05 — transaction·save·bounds·공개 소비자 (RQ05–07)

`CoreApplication.execute` → `world/application.rs:130–287`는 controlled clone에서 operation/lifecycle/observation을 준비한 뒤 GameCore 전체를 commit한다. bare BattleCore public operation은 이 outer transaction이 항상 감싸는 것이 아니다. 새 fallible kernel port의 실패 무변경은 그 실제 공개 경계에서도 보장해야 한다.

`world/state.rs:542–639`의 BattleEntrySnapshot restore는 active battle을 없애고 UUID stream을 복구한다. 따라서 retreat/retry 뒤 battle UUID가 재사용될 수 있다. `authoring_lifecycle.rs:1–241`의 entry 관찰 identity는 이미 이를 고려하지만 새 ActionHandle의 validity를 자동으로 보장하지 않는다. `state.rs`, `active_run.rs`, `node_flow.rs`의 durable V6/entry/result 소유와 live observation 경계는 별개다. 현재 inventory의 영구 ID 저장 계약을 바꿀 필요가 없다.

`authoring/control.rs:1–154`의 finite work/event/bucket/effect/entity/clone 한계와 `core/authoring_observation.rs:225–397`의 admission은 authoring scope를 전제로 한다. 새 native runtime 상태는 scope가 꺼져도 유한해야 한다. 현재 mandatory observation은 latest/latest-nonzero/counter 중심의 제한된 projection이며 모든 gameplay history store가 아니다. `authoring/trial.rs:610–640,845–941`, `query.rs:101–169`, CLI `author_content.rs:132–184`가 같은 production API/receipt 경계를 소비한다.

## 3. Proposed Implementation와 Technical Decision Log

아래 P01–P07은 `technical_decision`이다. 승인 의미를 보존하는 내부 권장 구조이며 구현 owner는 후속 R04 GEX다. 정확한 Rust 명칭·모듈 배치·동등한 자료구조는 owner가 결정한다. 의미·수명·실패 경계는 임의 재량이 아니다.

### P01 — 확정 mutation owner에서 사실 생성

BattleCore의 공통 fact state가 action/strike/segment 식별과 bounded result summary를 소유하고 실제 mutation helper가 Damage/Heal/Cost/Death 결과를 낸다. World가 소유한 tactical pay는 성공한 실제 소비를 같은 battle fact boundary에 전달한다. 요청량, actual HP delta/effective heal/resource consumed와 death를 분리한다. 실패를 zero-success로 바꾸지 않는다. Damage component별 계산 수치와 하나의 직접 strike를 구별하고 기존 actual-damage gate를 그대로 소비한다.

필수 death finalization과 wave bookkeeping은 optional reaction dispatch와 명시적으로 분리한다. victim source/status/position 중 필요한 bounded snapshot을 cleanup 전에 확보하여 lethal mutation과 연결한다. snapshot 읽기 시점을 기록하고 HP 변화 전후를 혼동하지 않는다. 기존 수식/HP 적용/사망/반응 순서는 유지한다. 새 콘텐츠의 predeath proc나 생존 규칙은 추가하지 않는다. 직접 summon Kill과 stat-HP lethal도 포함하고 Despawn은 death로 위장하지 않는다.

대안인 log replay 기반 receipt는 보관량/disabled log에 따라 의미가 사라지므로 제외한다. 모든 HP assignment를 heal로 감싸는 방식도 init/redeploy 의미를 잘못 바꾸므로 제외한다.

### P02 — 원천 snapshot과 distinct occurrence

실제 source에는 definition/effect origin, actor/provider, transient item ID와 알려진 persistent copy를 구분해 보존한다. action/cast, carrier step iteration/target의 direct strike, effect/component segment는 각각 다른 key다. strike key는 수식 전에 만들어 한 strike의 다중 segment가 여러 hit로 부풀지 않게 한다. hit 자격은 기존 HP-loss gate를 보존한다.

현재 skill command·trigger batch·queued buff application/tick·ScheduleAttack·projectile/area에 immutable causal context를 전달한다. 지연 효과는 caster/control 상태가 사라져도 필요한 bounded source를 보유한다. 실제 확보하지 못한 legacy origin/root/copy는 명시적인 부재다. log sequence는 narration 연결용으로 보존할 수 있지만 fact 권위 key는 log 보관과 독립이다. arbitrary recursive history 대신 현재 producer를 연결하는 bounded context를 사용한다. full chain visited/true join/periodic root는 해당 후속 owner에 남는다.

### P03 — request와 action control/settlement를 함께 전환

공통 kernel request는 qualification을 먼저 판정하고 typed Result로 거부한다. 성공 admission만 handle을 만들며 Accepted, Released, FlowFinished와 EffectsSettled를 따로 관찰한다. engine-owned action store는 제어 terminal과 남은 released delivery의 settlement를 구분한다. control 성공/취소/실패는 정확히 한 번 terminal로 전이하며 이후 delivery 정산을 두 번째 성공 terminal로 바꾸지 않는다. miss/zero actual 결과도 실제 finite delivery 종료를 따라 정산한다.

취소 helper는 기존 cause별 범위를 표현한다. withdraw/death/InterruptCast/HardCC/Silence가 허용하는 현재 경로만 바꾼다. t50/t150 fixture는 해당 기존 취소 cause로 요청하고 새 player UI/button이나 일괄 취소 gameplay를 추가하지 않는다. 제어 상태가 제거돼도 released projectile/GroundFixed settlement record는 유지한다. ApplyBuff는 실제 application까지 추적하되 적용 후 status/aura/summon의 독립 수명을 action 완료로 무한 기다리지 않는다. 각 기존 carrier의 유한 완료 조건을 GEX 상세 설계에서 전이 표로 고정·검증한다. Piercing projectile은 앞선 hit 뒤 빈 최종 구간에서 만료되면 runtime을 제거하되 기존 terminal impact를 내지 않을 수 있다(`projectile.rs:133–258`). 관찰 정산은 실제 제거와 남은 queued impact/application 완료를 추적한다. 이 보완을 위해 `resolve_skill_step_delivery`를 새로 호출하면 deferred-step release/cleanup(`cast.rs:183–212`)까지 바뀌므로 금지한다. 기존 deferred control이 남으면 FlowFinished/성공 terminal을 만들지 않는다.

handle은 attempt owner와 opaque validity token/epoch, deterministic logical occurrence를 분리한다. clone은 기존 유효 token을 보존하고 abort된 신규 요청에서 탈출한 handle은 복원 뒤 같은 ordinal이 나와도 유효하지 않아야 한다. retry는 새 attempt owner다. token 비교는 validity에만 사용하며 RNG, ordering, semantic receipt identity에 쓰지 않는다. battle UUID/cast sequence만으로 stale 판정하는 대안은 restore/retry에 안전하지 않다. source Arc와 handle token은 외부 mutable runtime backdoor가 아니다.

### P04 — paired equipment binding으로 AT04 전환

BattleUnitDraft의 장비 entry를 definition ID와 optional actual persistent copy를 묶는 값으로 전환하는 방안을 권장한다. 두 player draft producer는 inventory instance UUID를 직접 넘기고 두 build constructor가 RuntimeItem/source에 보존한다. stat pipeline은 definition을 읽고 runtime ID 생성은 기존대로 유지한다. 모든 constructor/test/scenario를 동반 전환하여 별도 배열/추정 lookup 불일치를 막는다. scenario/fragment의 None은 사실상 부재이며 fabricated UUID가 아니다. 기존 inventory serialized ID를 재사용하므로 이 변경 때문에 개인 저장 변환이나 V6 migration을 만들지 않는다.

### P05 — operation atomicity와 attempt 수명

새 fact/action/source state는 BattleCore/GameCore 소유 clone·candidate commit에 포함한다. bare public BattleCore의 fallible request도 preflight/준비 후 commit 또는 제한된 staged candidate로 보호한다. resource/queue/RNG/ID/log와 새 상태가 함께 commit/abort돼야 한다. 실패 오류의 context는 이전 authoritative state를 바꾸지 않고 반환할 수 있다. 이전 성공 timestamp prefix를 되감거나 기존 recoverable trial을 sticky failure로 바꾸지 않는다.

entry creates attempt; live clone/reconnect reads same attempt state; failed candidate aborts all introduced mutation; battle exit/result projects 필요한 finite summary; retreat/drop/restart reclaim authoritative attempt store and invalidate old handles. 외부 immutable projection이 이미 반환된 경우 그것은 과거 값이며 새 attempt authority가 아니다. durable save는 현재 battle attempt를 영속화하지 않는 실제 writer/loader 의미를 유지하고 그 N/A를 테스트·계약에 명시한다. inventory 사본 ID는 기존 durable 경로를 사용한다. R20까지 상태 소유/복원을 미루지 않는다.

### P06 — finite native state와 관찰 projection

새 active handles, retained summaries, predeath status members, source context depth/cardinality와 in-flight delivery는 native hard bounds 및 authoring work/clone/retention admission을 함께 갖는다. 한계·counter overflow는 typed fault다. 필요한 결과를 silent eviction하지 않는다. full history append 대신 현재 action의 집계/필요한 유한 결과를 보존한다. 정확한 수치는 기존 한계·실제 fixture high-water와 allocator/cost 측정으로 GEX에서 정당화하는 safety parameter이며 gameplay cap/공식 평가 점수가 아니다.

CloneShape와 runtime growth enumeration을 새 field에 맞춰 전환한다. 부정 입력을 이유로 전체 capacity/이력 반복 scan을 허용하지 않는다. admission-before-allocation, inclusive bound/one-under 실패와 반복 복구를 검증한다. R03 mandatory observation/API/CLI에 필요한 fact projection을 연결하되 기존 meaning과 finite output 보장을 보존한다. tracing off 또는 log/diag cap 변경이 runtime fact/RNG/result를 바꾸면 실패다.

### P07 — 최소 공통 cutover와 지원 설명

위 변경은 AT03/AT04 producer부터 direct consumer·state·test·core contract까지 하나의 유효 cutover로 계획한다. 일부 writer만 새 fact/handle을 쓰고 나머지는 log-only인 중간 상태를 완료로 게시하지 않는다. 현재 RON parser/validator/compiler vocabulary를 그대로 사용해 current carrier/fact를 검증할 수 있으므로 새로운 RON opcode는 요구하지 않는다. 필요가 드러나면 같은 Goal에서 parse→validation→compiler→executor/negative fixtures까지 실제 지원 여부를 검토하며 승인 의미를 확대하지 않는다.

describe/explain/public receipt는 실제 제공된 milestones/source/unsupported scope를 정확하게 드러낸다. 후속 boss adapter/full Flow/R05 ResourceStore를 placeholder executor로 채워 R04 dependency처럼 만들지 않는다. 독립 default-release API와 얇은 CLI, 기존 public world ActivateSkill/focus/retreat/deploy 및 직접 BattleCore 테스트가 실제 소비자다.

## 4. Observable acceptance → validation seams

아래 A번호는 P2G-10의 각 element acceptance 배열을 1-based로 선택한다. 문구/의미는 원본 그대로이며 표의 시험은 이를 구현 관찰로 연결한다. 모두 후속 GEX에서 실제 nonzero 실행이 필요하다.

| Element/member | Seam | 구체 입력과 판정 |
| --- | --- | --- |
| E01/A1 | V01 damage/basic regression | 동일 seed/입력의 formula·OnAttack/OnHit·공명·actual HP-loss gate를 비교; 일반 skill은 basic trigger 미발동. 기존 hit-gated self-heal/barrage integration 보존. |
| E01/A2 | V01 actual mutation | overkill·overheal·음수 Heal: request와 실제 HP 변화 각각 확인; 음수 Heal old damage summary 유지, Cost로 바뀌지 않음. |
| E01/A3 | V02 cost | manual/auto 공명 reset과 tactical pay 성공의 actual consumed; 부족/무대상 거부는 성공 receipt 없음·state 불변. unsupported HP cost 등 성공 claim 없음. |
| E01/A4 | V03 death | lethal·direct Kill·stat-HP lethal, optional dispatch 억제: death/wave 정확히 한 번; 기존 OnDeath/OnKill/AllyDeath 순서. Despawn 분리. |
| E02/A1 | V04 occurrence | 한 carrier strike의 여러 damage segment와 서로 다른 iteration/target/cast 비교; 수식 전 strike ID와 actual loss 구분. |
| E02/A2 | V03 predeath | victim에게 caster A의 유효 buff, killer B: cleanup 전 status/source와 actual lethal 결과 연결; 신규 proc 없이 기존 순서 유지. |
| E02/A3 | V04 source | basic/skill/proc/buff tick/projectile/area, source 사망 후 delivery; log off/eviction에서도 origin/correlation 유지, unknown 명시. |
| E02/A4 | V04 public fact | current 실제 producer→core/API receipt 관찰; 미래 true join/root/zero-HP eligibility를 describe에서 지원으로 내놓지 않음. |
| E03/A1 | V05 request rejection | 무대상/불가 cast/예산 실패의 typed error와 handle 부재; bare BattleCore/public world 각각 rollback 검사. |
| E03/A2 | V05 milestones | prototype t0 accept/t100 release/t300 hit: 각 시점 조회; FlowFinished와 EffectsSettled가 실제 carrier 진행과 일치. |
| E03/A3 | V06 cancellation | t50/t150의 기존 withdraw 등 cause로 취소; released projectile/GroundFixed 지속, unreleased step 미부활; Follow area와 HardCC/Silence 별도 회귀. |
| E03/A4 | V06 identity | terminal exactly-once, late delivery settlement와 cancel 구별; duplicate/stale/old attempt handle 거부, limit fault는 silent eviction 아님. |
| E04/A1 | V07 equipment | 동일 정의의 실제 inventory 사본 둘을 서로 다른 actor에 장착해 두 source copy ID가 실제 보유 ID와 일치. |
| E04/A2 | V07 constructors | 두 player draft/build 및 deploy 경로 전체, loadout validation·stat/cooldown/RNG 보존·durable inventory ID roundtrip. |
| E04/A3 | V07 absence | scenario/fragment source persistent copy None, runtime ID와 구분; R07 완료 claim 없음. |
| E05/A1 | V08 transaction | request/advance 실패 전후 새 state와 queue/resource/RNG/ID/log bit-exact 비교; 이전 성공 timestamp prefix 보존·정상 재시도. |
| E05/A2 | V08 lifecycle | live clone/snapshot abort/reconnect 관찰; retreat/re-entry에서 UUID 재사용이어도 이전 handle invalid; result/drop 회수. |
| E05/A3 | V08 save | current V6 writer/loader 및 BattleEntrySnapshot 적용성 검증; 새 attempt state durable N/A, inventory identity 저장 유지. |
| E05/A4 | V09 bounds/determinism | native+authoring 한계 inclusive/one-under, predeath/source/fanout/clone high-water·allocator cost; seed/input/pack/container order와 log/diag cap별 semantic 결과 비교. |
| E05/A5 | V10 integrated acceptance | T07/T23/T26/T28 R04 부분, current R01–R03/direct API/CLI 및 관련 core tests/release, 고정 source 뒤 fresh 감사. 미래 R을 대신 세지 않음. |

GEX는 focused 실패/교정 후 관련 시험부터 재검증한다. 최종 안정화 때 full core lib/tests, all-targets 및 default/release public consumers를 수행한다. 실제 명령·toolchain·fixture·source/dirty hash·nonzero count·expected/actual output을 남긴다. 구조 검증, build 성공, 과거 PASS는 위 신규 behavior의 실행 PASS가 아니다.

## 5. Finding registry·evidence gaps·owner routing

| ID | Primary classification | 근거/영향 | Disposition |
| --- | --- | --- | --- |
| F01 | implementation_finding | HP/log 중심 출력은 actual/segment/predeath/source 계약에 부족(B01/02) | P01/02와 V01–04로 R04 구현; 현재 기능 완료 아님. |
| F02 | implementation_finding | pending/log identity와 active cast lifetime만으로 stale/settlement 부족(B03/05) | P03/05와 V05/06/08 동반 전환. |
| F03 | implementation_finding | draft가 실제 equipment copy를 소실(B04) | P04/V07 AT04; 기존 runtime ID 보존. |
| F04 | implementation_finding | 새 native state/clone/receipt는 기존 authoring bounds만으로 보장되지 않음(B05) | P05/06/V08/09에서 도입 때 닫음. |
| F05 | technical_decision | 복원 뒤 UUID 재사용, source가 없는 legacy 경계, signed Heal/real tactical pay를 실제 source로 확인 | P01–07의 선택/대안이 승인 acceptance 보존; scope/Decision 수정 없음. |
| F06 | out_of_scope_finding | true join/periodic root, zero-HP new hit/새 predeath reaction, shield/HP1/HPcost/parry/special kill 의미는 후속 정책 owner | P2G-10의 C06/O02/O03·PG02–04에 route; R04 current producer를 위해 새 의미를 고르지 않음. 새 필요가 확인되면 P2G-10 반환. |
| F07 | evidence_gap | 외부 배포·server/Unity와 개인 저장 내용 미조사 | RQ08/09 승인 N/A; core save/API required는 B05/P05/V08에 포함. required completeness blocker 없음. |
| F08 | implementation_finding | 지금 source에는 R04 제안 코드/신규 시험이 없음 | 정상 implementation delta. P2G 성공과 구현 완료를 구분하고 GEX acceptance 전 R04 완료 금지. |

본 검토에서 새 in-scope material policy 또는 필수 source unknown은 발견하지 않았다. 이것은 구현 중 발견할 가능성의 부정이 아니다. GEX에서 current premise 오류/새 observable 의미/필수 source gap을 찾으면 owning P2G stage와 영향 closure로 반환한다. 내부 field/함수/자료구조 선택은 상세 설계에 남기되 owner·수명·취소·실패·저장 의미와 위 검증 의무를 축소하지 않는다.

## 6. Technical successor / GF-CARF-001·002

Predecessor `CRA-CARF-P2G20-001`; correction owner P2G-20 under `PFC-CARF-001 / COD-CARF-002`. [Exact finding](GEX_FINDING_001.json). Source bytes are unchanged. Five existing code_reviewed revisions, Decisions, all20acceptance, Phase/Goal scope remain unchanged. Updated B03/P03 correct Workspace Actual and observational implementation boundaries; current gameplay is preserved, including its incomplete gate bookkeeping. No new source/runtime test has executed.

V01/02/05/06/08/09 additionally require actual pending→death→CastEnd (resource reset without false release), death-after-release queued step/Follow behavior, piercing earlier-hit→empty-final-interval expiry and remaining queued effects. Distinguish finite delivery completion from still unresolved deferred control; no synthetic terminal or gate success. Assert operation rollback, duplicate settlement and bounded retention. Existing t50/t150 withdrawal fixtures use actual existing cancellation causes. These are concrete oracles within existing acceptance/IVA ownership, not added policy acceptance.

Separate same-model read-only reviewer `r04_p2g20_semantic_review` independently read the source and exact Freeze/Goal criteria and confirmed this technical scope; this is planning review only, not artifact publication review or runtime/completion audit. Original source evidence is positively reused after all49files SHA256 check, with fresh current-input plan/observation/binding envelopes.


Owner entry successor WR002 inherits all5PER/5DR/20acceptance from WR001 and adds exact Change/COD002 basis under §8.7. Previous same-WR review002 was a failed repair candidate as recorded in REPAIR_CANDIDATE_DISPOSITION_001.json. Technical source findings remain unchanged. This review binds all observations and existing code_reviewed revisions to WR002, with no new policy revision.
