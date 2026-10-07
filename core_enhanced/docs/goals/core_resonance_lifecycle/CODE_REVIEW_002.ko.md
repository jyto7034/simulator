# U05 코드 기반 검토 — candidate batch 002

현재 basis는 WR-CRSL-003/PIS-CRSL-002, 정책14개와 RSP-CRSL-002의 required7/N·A2 경계다.
P2G-20 최초 candidate는 초기 충전 효과/적 owner 의미를 발견해 성공 publish 없이 반환했다.
P2G-10 successor가 세 정책을 보완했고 나머지11개는 exact 유지했다. 이 검토는 정적 current actual,
변경 경계와 구현 구조 제안이며 구현·새 시험·release/deployment 성공을 뜻하지 않는다.
정확한 파일 내용은 `evidence/REVIEW_SOURCE_002.tar.gz`와 manifest에 고정한다. 전체 파일의 hash는
모든 행을 읽었다는 주장이 아니다. 아래 claim에 필요한 producer→owner→consumer를 선별 추적했다.

## Current Actual

| Requirement | 확인한 실제 경로와 의미 | 필요한 전환 |
|---|---|---|
| RQ01 | `stat_pipeline.rs:33-48,244-264`: fragment 적용 뒤 InitialSkillCharge를 start에 더함. `skill_fragment.rs:354-418`: 실제 사본/형태를 해석하고 Manual/source를 붙이지만 파편별 최대 공명은 없음. `build.rs:81-84,243-246`: 공통 profile의 start/max/gain_lock으로 양 spawn helper가 초기화. `resonance_runtime.rs`: release10·dealt/received HP÷10; `commands.rs:420,464,532`와 `damage_runtime.rs`가 실제 호출. `tactical_stance.rs:414-443`: Rest HP와 공명 별도 지급. `mod.rs:947-1108`: 모든 명시 양수 ModifyResonance가 add_resonance의 HardCC/시간 lock gate를 통과. 공통 전투초 직원 regen은 없음 | 직원 base20%와 기존 bonus 순서, 파편별 max, 직원 auto pulse·legacy 획득 제거. opponent autonomous 기존 주기 보존. typed use state를 공통 양수 gate에 추가 |
| RQ02 | `world/combat.rs:2703-2955`의 focus/preview/ActivateSkill→kernel candidate는 preview read-only·실패 원자성·성공 focus restore를 이미 공유. `sim.rs:784-1030,1190-1315`: manual base는 full을 검사하지만 start_admitted는 소비하지 않고 start_gate_pending/reset_resonance를 기록. `SkillStartGate:4098-4168`가 실제 잔량 full·silence·최신 target을 재검사. `corrosion.rs:90-204`도 full을 다시 요구하고 실제 시작에서 전량 소비·철수/trauma100·기존 전체 적 특수사망·host시간 강제Pause 실행. `AutoCastStart:4171-4380`은 별도 접수/침식 호출. Manual/AutoCastEnd:4396-4619에서 reset 및 postlock. 직접/타일 target validator 모두 any-step usefulness를 검사 | 유효 접수 한 번 소비+자격, start/corrosion의 재소비 제거와 실행 시점 유지. target kind/empty tile/anchor 검증 분리. auto와 boss explicit caller도 정확한 resource owner에 연결 |
| RQ03 | `skill_runtime/cast.rs:62-109` cleanup은 flow complete+모든 delivery terminal+area 소멸을 기다림. `flow.rs:162-310,428-490`: Released 또는 EffectsSettled로 후속 action 진행, structured root complete에서 action dispatched. `core/runtime_facts.rs:742-803`: control terminal과 delivery settlement 분리. `sustained.rs:172-438,488-680`: finite modifier/overlay/shield/channel 기록·만료/소비/target·source 종료; channel만 HardCC stop. `sim.rs:4662-4965` buff는 caster/target/id key, refresh 최신 expiry, tick와 expiry event. `mod.rs:1192-1277`, commands InterruptCast, build withdrawal/death 정리가 미방출/방출 효과를 구별 | active_skill_casts empty 또는 EffectsSettled를 사용 종료로 재사용하면 느린 projectile/독립 area가 잘못 잠금. 시간제 buff는 실제 lease 종료를 유지. combo/flow 미방출 control 취소는 released carrier와 별개로 닫기 |
| RQ04 | `ability.rs:750-767` SkillDef에는 use lifetime descriptor 없음. strict `RawSkillDef`는 `skill_data.rs:758-779`, 변환은885-908. `ability/flow.rs`의 finite control plan과 `ability/sustained.rs`의 typed finite effects가 공개 제작 경로. `ResonanceDef`에는 start/max/gain_lock, `PlatformRestProfile`에는 HP와 resonance bonus. live `skills/base.ron`25개는 instant/projectile/TileArea/extraattack/permanent ModifyStats/ApplyBuff 조합. Flow·channel·finite modifier 등은 공개 R08–12 fixture에서 확인. live loader/builder는 같은 validation owner를 호출 | public descriptor와 compiled plan/view/clone shape·전종 데이터 동시 전환. 직원 profile과 적 autonomous profile 적용 범위 검증. Rest 제거 필드는 strict parser에서 오류. 파편별 max를 actual selected form/definition 소유로 연결 |
| RQ05 | readiness DTO는 current/max/manual eligibility/target/corrosion만 운반(`behavior.rs:1927-1945`, `sim.rs:1070-1186`), independent gain lock reason 없음. `world/state.rs`가 snapshot/battle_update에 같은 projection을 사용. host handlers는 read-only PreviewSkillTarget을 query로 분류하고 CoreApplication 명령/준비된 tick을 실제 파일 publication과 연결. reconnect는 같은 actor의 active_battle_reconnect를 호출하고 runtime을 재생성하지 않음. host state/messages는 Core DTO를 serialize | additive authoritative use/gain-lock projection과 공개 catalog 의미 갱신. 예외/우선순위를 query에 재구현하지 않고 kernel helper 공유. 실제 host public path에서 observe; Unity 구현은 범위외 |
| RQ06 | U04 `active_run/continuation.rs`는 current confirmed/pre-entry와 SavePoint를 저장; 미완료 battle kernel을 wire serialize하지 않음. `state.rs:637,674,1485` active session/private candidate clone에 live battle/focus가 포함. `core/types.rs` RuntimeUnit/PendingSkillCast/ActiveSkillCast는 controlled clone 대상이며 별도 direct save codec이 아님. File active/profile adapter와 U04 prepared publication이 Continue/rewind 경계를 소유 | 새 runtime state는 transaction rollback/private clone/reconnect에서 보존. 지원 snapshot roundtrip은 기존 지원 surface(정상 query resync/인메모리 snapshot 및 durable confirmed state)로 검증하며 새 live-battle 디스크 저장을 발명하지 않음. employee persistent schema를 바꾸면 reader/writer/구형보호를 동시에 갱신 |
| RQ07 | U04 publication의367제품 hash를 전부 재검증: unchanged. 감사 Complete/High 원본·U01 취소와127조건/별도R06 증거는 인수 baseline. 기존 tests/live_item_skill_activation, skill_runtime_contract, live_skill_catalog_audit, ron_loading, world/tests/combat/checkpoint_validation, host handler/reconnect와 file fixtures에 실행 seam 존재 | 변경된 기대값은 명시 정책으로 교체하고 무관한 회귀는 보존. GEX에서 새 공개 RON+normal command와 actual state/trace를 실행; type/test 존재만으로 통과 처리하지 않음 |

Released Actual(RQ08), Deployed Actual(RQ09)는 승인한 workspace Core 구현/실제 local host source 검토의
완료 claim에 필요하지 않다. 외부 revision을 추정하지 않고 N/A 이유를 plan/observation에 고정한다.
향후 로컬 release build·headless 실행은 GEX의 검증 의무이며 이 N/A로 제외하지 않는다.

## Proposed Implementation

모두 구현 제안이다. 정확한 타입/field spelling은 GEX 상세 설계에서 선택하며 다음 owner와 의미는 보존한다.

- TD01 직원 resource domain: 실제 Employee source identity와 player ownership으로 새 자원을 적용한다.
  실제 장착 fragment form/definition에 양수 max를 작성하고 profile composition에서 base floor20%를 만든 후
  기존 consumable bonus를 적용한다. arbitrary employee start/postcast values는 더 이상 직원 authoring authority가
  아니다. opponent/summoned profile의 기존 autonomous 초기량·획득·주기는 별도 유효 owner로 유지한다.
  공통 struct를 그대로 두고 field를 직원에서 조용히 무시하는 구현은 피한다. 필요하면 typed profile 분리와
  validation을 함께 바꾼다. 저장된 직원 base profile이 영향을 받으면 CF-D01의 원본보호를 지킨다.
- TD02 regen: event-driven kernel에 battle-time 1000ms cadence를 둔다. 전체 대형 scan을 매 ms 수행하지 않고
  살아 있는 직원의 기존 scheduling/queue owner로 pulse를 관리한다. 차단된 pulse는 버리고 backlog를 만들지
  않는다. Pause/forced Pause에는 battle clock이 흐르지 않는다. 모든 explicit 양수는 같은 gain gate로 들어간다.
  직원 attack/damage gain과 Rest bonus 호출/작성 필드를 제거하되 적 autonomous gain path는 보존한다.
- TD03 single admission: 기존 public command transaction 안에서 latest validate→action accept→consume-all fact/log→
  typed use credential→pending/event를 함께 commit한다. credential은 actor/skill/action identity에 묶이고 해당
  use가 유효할 때만 start를 통과시킨다. 오래된 queue seq와 중복 input은 새 사용을 만들지 않는다. rollback은
  소비/fact/자격/event를 함께 되돌린다. 후속 castend에서 reset·임의 postlock을 다시 적용하지 않는다.
- TD04 corrosion: readiness의 입력 전 full 검사는 유지하지만 실제 try_start에는 admitted credential을 전달한다.
  현재 actual start/first unpaused boundary와 host presentation pause, trauma·withdrawal·기존 특수 death는 그대로다.
  이미 기록한 소비를 corrosion에서 다시 차감하거나 spent0 비용으로 덮지 않는다. action provenance와 cost reason을
  실제 접수/실행에 맞게 관측하며 불필요한 새 피해/분류 정책을 도입하지 않는다.
- TD05 actual use state: runtime action 관측 identity를 연결하되 별도 private use owner가 pending control,
  실제 사용 동작, 선택된 finite buff/channel 의무를 추적한다. action resource Activation/Delivery/Chain과
  공명 잠금 lease는 서로 다른 의미다. input pending→cast→action dispatch/last attack→ended/cancelled와
  applied timed lease를 식별한다. independent projectile/ground area/summon/DoT는 생성 이후 잠금 owner가 아니다.
  cleanup_finished_skill_cast/action EffectsSettled에 잠금 해제를 단순 결속하지 않는다.
- TD06 flow: legacy delayed step/extraattack과 structured Sequence/Parallel/Branch/Repeat/Wait의 미래 미방출 동작을
  추적한다. 마지막 leaf가 projectile EffectsSettled를 기다린다는 사실만으로 명중까지 잠그지 않는다.
  아직 뒤 action/branch/repeat 의무가 있으면 중간 발사 후 조기 해제하지 않는다. 필요한 미발사 control은
  HardCC/InterruptCast에서 취소하고 released carrier를 유지한다. 명시 descriptor/compiled summary가 모든
  public 조합에서 계산 가능해야 하며 지원 못 하는 결합은 정확한 publication error로 거절한다.
- TD07 timed effects: public use descriptor가 실제 finite buff/channel owner를 식별하게 하며 source action/use를
  실제 적용 record에 연결한다. expiry만 예약한 상수 lock 대신 실제 early removal/overlay consumption/shield end,
  channel reinput/CC, refresh/replacement와 no-op application을 반영한다. 관련 의무가 복수면 마지막 의무가
  끝날 때 해제한다. permanent ModifyStats와 hostile DoT를 일괄 timed-buff 잠금으로 세지 않는다.
- TD08 target: PlayerTile는 valid tile/reach/kind로 admission하고 occupant usefulness 요구만 제거한다.
  PlayerUnit/automatic usefulness는 유지. final acceptance anchor를 pending→cast에 운반하고 direct target의
  필수 lifecycle 상실은 취소/no refund. area anchor의 원래 unit 생존·현재 좌표를 재검사해 접수 geometry를
  바꾸지 않는다. 이미 접수된 use의 start gate에 full/silence 검사를 재사용하지 않는다.
- TD09 observation/restore: typed use phase/lock reasons를 single helper에서 readiness/HUD 또는 적절한 existing DTO로
  투영하고 host serialize/canonical을 함께 갱신한다. HardCC와 use lock의 중첩을 표현한다. 새 owner를 controlled
  clone, retained extent, fault rollback, checkpoint/currentness fixture에 포함한다. 지속형 저장에 runtime pending을
  새로 넣지 않으며 same actor reconnect와 새 process Continue를 구분한다.
- TD10 atomic cutover: raw/compiled schema, official25개와 current fixtures, admission/corrosion, use completion,
  query/canonical, 원자 복원/회귀 검증을 하나의 실행 Goal로 묶는다. 내부 WP는 순차 작업일 뿐 독립 완료·감사
  통과를 가장하지 않는다. 부수적인 source 이동/공통 재설계는 필요 없으면 하지 않는다.

대안: 단일 lock-until timestamp는 실제 취소/만료/조건부 종료를 설명하지 못한다. action terminal만 사용하면
finite buff가 너무 일찍 끝나고 EffectsSettled만 사용하면 projectile/독립 child가 너무 늦게 끝난다.
별도 player AI/Unity 계산으로 보완하면 정상 Core query와 불일치한다. 위 구조는 기존 kernel transaction과
source identity를 재사용해 이 차이를 한 owner에서 처리한다. 구현자는 exact storage/container/enum과 pulse
같은 시각 우선순위를 기존 deterministic queue 규칙 아래 선택하고 관련 경계 시험을 기록한다.

## Element Coverage / Validation Seams

| 정책 | Current actual/TD | GEX 판정 예제 |
|---|---|---|
|01|RQ01/04,TD01|최대1/4/5/99/100 기본값, max100+InitialSkillCharge40%=60, 재배치/이동 구별·fragment max |
|02|RQ01/04,TD01/02|공격 없는 직원·Attack/Defense/Work/Silence, Pause/cap/CC missed pulse, Rest HP 유지·enemy cadence |
|03|RQ01/03,TD02/05|모든 ModifyResonance 양수, overlapping HardCC/use, 음수/초기화 구분 |
|04|RQ02/03/05/06,TD03/04|Pause 소비 즉시·resume1회·중복·잔량0 credential·fault rollback·stale pending |
|05|RQ03/04,TD05–07|instant/slow projectile/combo/timed modifier/channel/area/summon/DoT/permanent, 혼합 의무 |
|06|RQ02/03/06,TD05–07|HardCC cancellation·released preservation·timed buff 유지·silence before/after·withdraw/death |
|07|RQ02/03,TD08|cast중 직접 target death/withdraw/faction/air 무효→cancel/no refund, area anchor 유지 |
|08|RQ02/04/05,TD08|wrong/missing kind/full/range·공중/관계/usefulness, manual empty tile vs invalid tile |
|09|RQ02/05/06,TD03/08|stale preview 재검사·failure state/revision/focus/playback unchanged·cancel restore |
|10|RQ02/04,TD08|manual anchor movement/death, manual retarget strict error, automatic retarget 유지 |
|11|RQ02/03/06,TD03/04|현재 corrosion input vs start, consumed0 정상 실행, cost once, trauma/withdraw/death suppression·forced Pause |
|12|RQ03/04/05/06,TD05–10|25official+공개 typed fixtures 전종 descriptor, removed field error, catalog/readiness/host wire, effects-unsettled+use-ended |
|13|RQ05/06,TD09|same actor paused reconnect·clone rollback, Continue pre-entry와 SavePoint/U04 assets, 구형bytes 보호 |
|14|RQ06/07,TD10|127선행+별도R06 영향표·실제 공개경로·안전성/resource caps/observer parity·독립 AstraHigh 감사 |

검증은 기존 의미가 바뀐 지점의 old assertion을 이유와 함께 바꾸고, 새 behavior를 직접 관찰하는 경계 예제를
추가한다. full Core/host suites, fmt·clippy·release public surface와 선행 byte/currentness 확인을 실행 범위에
포함한다. 새 semantic test의 증거를 단순 snapshot/type 존재로 대체하지 않는다.

## Findings / Limits

- IF01 implementation finding: current reset/start full recheck conflicts with already approved immediate consume.
  정책 변경 없이 TD03/04로 수정. PE04/11, RQ02/03.
- IF02 implementation finding: no current use descriptor/regen; release/control/settled already distinct.
  정책 변경 없이 TD02/05–07. PE02/03/05/06/12, RQ01/03/04.
- IF03 implementation finding: current PlayerTile hostile occupant gate conflicts with explicit newer P03.
  이미 P2G-10에서 결정한 차이이며 TD08 전환. PE08/09/10, RQ02/05.
- TD11 technical clarification: 기존 지원 저장이 live battle를 serialize하지 않으므로 이를 새 요구로 추론하지
  않는다. same actor/인메모리 snapshot은 pending을 보존하고 durable Continue는 U04 pre-entry로 복원한다.
  PE13의 '지원 snapshot' 범위 해석이며 의미 변경/새 저장 권한이 아니다. RQ06.
- ED01 evidence limit: lexical live inventory25개는 parser/실행 증거가 아니다. GEX에서 실제 loader와 전종
  lifecycle matrix로 보강한다. 정적 review claim에는 필수 gap이 없으며 production success는 pending.
- 선행 original evidence와 제품367개 exact unchanged만 현재 재사용한다. 새 구현 후 invalidated 조건은
  재검증해야 한다. remote/Windows crash/general power-loss/Unity 구현을 성공했다고 주장하지 않는다.

현재 새 material finding0, 필수 source gap0. 좁힌 exact state·enum·validation의 준비 점검은 GEX에 인계한다.
