# Core Runtime Contract

이 문서는 core runtime 구현자가 지켜야 하는 source-of-truth, 결정론, 관측/전송 경계 계약을 정리한다.

## 문서 책임

| 구분 | 내용 |
| --- | --- |
| 문서 등급 | Runtime canonical |
| 소유 범위 | 결정론과 RNG, runtime state와 원자적 commit, event/checkpoint/result lifecycle, projection ownership, recovery, VN runtime, actor identity |
| 소유하지 않는 범위 | 플레이어-facing 규칙의 의도, 타겟팅·범위 알고리즘, live RON authoring/loading, Unity wire의 정확한 JSON shape와 소비 순서 |
| 함께 확인할 문서 | [game_rulebook.md](game_rulebook.md), [skill_target_contract.md](skill_target_contract.md), [data_loading_contract.md](data_loading_contract.md), 외부 `unity_core_contract.md`, 외부 `core_unity_battle_transport_contract.md` |

[game_rulebook.md](game_rulebook.md)는 플레이 규칙을 설명하고, 이 문서는 그 규칙을 runtime code, event log, snapshot, Unity-facing DTO로 구현할 때의 금지/허용 경계를 설명한다. 문서보다 실제 runtime code와 live RON/data가 우선이며, 문서와 코드가 충돌하면 코드를 먼저 확인하고 정책 판단이 필요한 부분은 사용자와 의논한다.

## Runtime Foundations

이 장은 다른 runtime 계약이 전제로 삼는 결정론과 검증된 data의 진입 경계를 소유한다.

### Determinism And RNG

전투 판정 RNG는 event log 관측 순서와 분리한다.

- 치명타, proc, 명중, 회피, 특수 발동처럼 gameplay 결과를 바꾸는 모든 랜덤 판정은 `event_log_seq`나 로그 기록 순서를 seed 재료로 사용하지 않는다.
- 새 랜덤 판정을 추가할 때는 공격 원천, caster/target, projectile/cast/step/hit identity, battle time 같은 안정적인 gameplay identity에서 seed를 파생한다.
- `BattleEventLog`는 결과를 기록하고 Unity presentation에 전달하는 관측 채널이며, 판정 source of truth가 아니다.
- 로그 이벤트를 추가, 삭제, 분할, 병합하는 작업이 이미 존재하는 전투 판정 결과를 바꾸면 안 된다.
- proc 성공 횟수, cooldown 상태, max trigger count 같은 runtime state는 발동 제한과 디버그에 사용할 수 있지만, proc roll identity의 주재료로 사용하지 않는다.

맵 생성, 보상 후보 생성처럼 생성 시점에 결과가 run state로 고정되는 절차적 생성은 seeded `StdRng` 순차 draw를 사용할 수 있다. 단, 생성 결과를 저장하지 않고 seed만으로 재생성하는 구조를 도입할 때는 별도 계약과 재현성 테스트가 필요하다.

### Data Authoring And Loading

Live RON authoring strictness, embedded loader ownership, fixture 경계, validation gate의 canonical 계약은 [data_loading_contract.md](data_loading_contract.md)를 따른다. 이 문서는 검증된 data가 runtime에 들어온 이후의 동작 계약만 소유한다.

## Active Battle Lifecycle

이 장은 active battle의 관측 로그, setup/checkpoint presentation과 CombatResult 종료 경계를
소유한다. 플레이어-facing 성공·실패 의미는 [game_rulebook.md](game_rulebook.md), 정확한 wire
payload와 message order는 외부 Unity canonical을 따른다.

### Battle Event Log Boundary

`BattleEventLog`는 live `DefenseRoute` 전투의 append-only 관측 로그다. 전투 전체를 미리 산출하는 replay source가 아니며, gameplay 판정을 되돌려 계산하는 입력도 아니다.

- `battle_update.events_delta`는 아직 클라이언트가 presentation event log에 적용하지 않은 event log entry 구간이다.
- 전투 중 정확한 연출 타이밍 source는 `battle_update.events_delta`다.
- 전투 중 현재 상태/복구 source는 `battle_update.checkpoint`다.
- Unity는 event로 현재 상태를 계산하지 않고, checkpoint로 연출 타이밍을 만들지 않는다.
- `checkpoint.at_seq`는 그 seq까지의 event를 presentation timeline에서 처리한 뒤 reconcile할 권위 상태를 뜻한다.
- seq가 부여된 live presentation event는 이후 수정하지 않는다.

### Battle Setup And Movement Presentation

전투 시작 scene construction source는 `battle_setup_snapshot`이다. 전투 중 이동 연출 source는 checkpoint position이 아니라 movement event다.

- 유닛 이동 연출은 `MovementSegmentStarted`/`MovementStopped` event를 기준으로 한다.
- 실제 위치 변화가 있는 movement tick은 Unity-facing event로 노출된다.
- `checkpoint.units[*].world_position`은 이동 연출 source가 아니라 event 처리 후 보정할 reconcile target이다.
- 유닛 철수/사망 연출 위치는 `UnitWithdrawn`/`UnitDied` event의 `world_position`과 projected tile `position`을 기준으로 한다.
- 철수/사망한 유닛은 official checkpoint `units`에서 빠질 수 있으므로, Unity는 checkpoint에서 inactive unit을 찾아 연출 위치를 추론하지 않는다.
- `battle_update.checkpoint.units`는 official gameplay presentation/reconcile source이며 Active unit만 포함한다.
- inactive runtime unit 조회가 필요하면 debug/admin/replay 전용 query를 사용하고, 해당 query의 결과를 gameplay 표시 source로 사용하지 않는다.

#### Route And Battlefield Projection

- `BattlefieldRoute.cells`는 적이 실제로 따라가는 route path다. Unity는 `start`/`end`만 보고 경로를
  재계산하지 않고 setup snapshot의 `routes[*].cells`를 scene route 표시와 이동 연출 기준으로 사용한다.
- route cells는 유효 타일 위에 있어야 하며 static obstacle과 void tile을 지나면 안 된다. 연속된
  cell은 상하좌우 cardinal-adjacent여야 한다.
- authored route가 없는 Defense 템플릿은 core가 deterministic generated route를 만들 수 있다.
  Generated route도 같은 cell 조건을 만족해야 하며 `[start, end]`만 가진 대각선/직선 fallback은
  공식 계약이 아니다.
- `battle_setup_snapshot.routes`, runtime enemy movement plan, movement event/checkpoint는 같은 route
  cells에서 파생되어야 한다.
- 전투 이동과 충돌은 2D 연속좌표 기반이며 static obstacle과 비사각형 전장의 void tile은 이동 충돌
  입력으로 투영된다. 배치 입력과 range 후보 선별은 별도의 타일 기반 계약을 따른다.

### Active Battle Result And Diagnostic Sources

게임은 player-facing 또는 authoritative gameplay 전투 history를 생성하지 않는다.

- 전투 event log, participant result, result stats는 active battle과 현재 CombatResult lifecycle의 소유다. 해당 content가 종료되면 run/save/checkpoint/profile 상태에 history로 보존하지 않는다.
- 선택적 개발자 trace는 실제 `battle_uuid`를 key로 `target/combat_traces/run_<seed>/<battle_uuid>.json`에 쓴다. 이 파일은 save/checkpoint/gameplay state가 아니며 serialization/filesystem 실패는 warning만 남기고 전투 결과 전환을 실패시키지 않는다.
- story document collection은 전투 history와 독립된 시스템이다.
- 전투 결과 화면의 1차 통계 source는 core가 `BattleEventLog`와 `ParticipantBattleResult`에서 전투 종료 시 산출한 `selected_event.result_stats`다.
- 전술적 전투불능은 player `RuntimeUnit` instance별 idempotence ledger로 직원의 attempt-local condition에 정확히 한 번 반영한다. 재배치는 그 working 직원 상태에서 새 draft를 만들며 별도 death HP 비율 override를 사용하지 않는다.
- 전술적 `RuntimeUnit` 사망과 persistent 직원 사망은 같은 상태가 아니다. `apply_incapacitation`은 instance마다 physical damage를 1, 정책 trauma를 한 번 더하고, physical damage 6만 persistent 사망이다. physical damage 5 또는 trauma 100인 직원은 살아 있지만 combat eligibility가 없으므로 새 runtime instance를 만들 수 없다.
- 완료된 결과는 attempt-local condition을 다시 패널티 계산하지 않고 roster에 한 번 커밋한다. 직원 condition, event의 `OnAnyResolution` 효과, 전투 단위 소비 아이템 duration은 하나의 staged result state에서 적용한 뒤 한 번 커밋한다.
- `RetreatBattle`은 전투 진입 전에 캡처한 world snapshot을 소비해 `NodeConfirm`으로 정확히 복원한다. attempt, RNG/UUID continuation, roster/inventory/resource, 소비 아이템, node/session/content, VN/trigger, run checkpoint를 포함하며 CombatResult/node completion을 만들지 않는다.
- 커밋 결과가 `NoLivingEmployees`면 남은 attempt보다 failure 후보 판정이 우선한다. `NoDeployableEmployees`는 실패 node 완료 후 현재 보유 Physical Treatment, 현재 progression에서 즉시 선택 가능한 SavePoint/Gate, runtime candidate catalog가 실제 채용 후보를 제공하는 `HeadquartersContact`를 하나의 evaluator로 계산한다. `combat_recovery_restriction` projection과 `SelectMapNode`/loadout/consumable validation은 같은 제한을 사용하고 허용 행동 완료 후 전체 deployability와 recovery path를 다시 계산한다.
- 강제 BossOmen capstone 승리 completion은 terminal Floor advance를 계획하지 않는다. Authored capstone의 forced boss를 완료/clear하고 생성 때 precommit된 concealed Exit Gate와 connection을 공개해 현재 Floor의 유일한 selectable successor로 staged commit하며, 기존 Gate confirm만 `GateTransition` recovery와 Floor advance를 수행한다.
- 모든 gameplay failure 후보는 결과 transaction을 먼저 커밋한다. 유효하고 `loads_used < 3`인 checkpoint가 있으면 `RunFailed` 전에 `AwaitingRunFailureDecision`으로 전환해 `LoadRunCheckpoint`, `ConfirmRunFailure`만 투영한다. checkpoint가 없거나 loadable하지 않으면 즉시 failure를 확정한다. BossOmen 전용 post-`RunFailed` load 예외는 없다.
- 직원 카드의 화면상 나열 순서는 gameplay 상태가 아니다. Core는 직원 보유·생사·출전 가능 여부와 전투 배치를 계속 소유하지만 표시 순서, 표시 슬롯, 표시 순서 변경 command/capability/result를 저장하거나 투영하지 않는다. 따라서 checkpoint load와 battle retreat는 Unity의 로컬 표시 순서를 되돌리지 않는다.
- `LoadRunCheckpoint`와 `RecoverBattleSetupLoss`는 서로 다른 recovery command다. 전자는 Core가 root capability를 투영한 경우에만 checkpoint를 소비하고, 후자는 active battle의 immutable setup과 현재 revisioned projection을 읽기 전용으로 다시 보내며 revision을 소비하거나 mutation authority를 요구하지 않는다.
- 성공한 `LoadRunCheckpoint`는 checkpoint-owned state 복원과 `loads_used + 1`을 하나의 Core
  transaction으로 적용한다. `loads_used`는 checkpoint payload로 복원하지 않는 run-scoped monotonic
  metadata이며 실패한 load, SavePoint overwrite, restart와 backup recovery에서 감소하지 않는다.
- `power_model_version`과 `current_floor_power_anchor`는 checkpoint payload가 map/node difficulty와 같은
  generation으로 소유하고 load/continue에서 exact 복원한다. `run_peak_certified_power`만 checkpoint 밖
  power high-water다. Core transaction rollback은 peak만 max-merge하고 anchor/model generation은 rollback
  snapshot으로 복원한다.

### Durable active-run host boundary

- Core export는 마지막 완료 SavePoint를 strict core payload version 2 JSON으로 직렬화한다. Payload의
  checkpoint가 model version/current Floor anchor를 소유하며 local encounter compendium과 transport
  `player_id`는 포함하지 않는다. Continue bootstrap은 현재 host identity/profile을 주입한다.
- `game_server`는 local-install scope의 단일 `ActiveRunStore`를 player actor별 파일이 아닌 host
  persistence aggregate로 소유한다. Strict envelope version 3, checksum과 `loads_used`/run peak만 가진
  monotonic metadata, current file,
  검증된 직전 checkpoint generation의 rolling backup, temp file, directory fsync와 process-lifetime
  exclusive lock을 사용한다.
- SavePoint completion 또는 성공한 명시적 load 뒤 durable publish가 실패하면 actor는 캡처한 Core
  state를 복원하고 command 성공을 보고하지 않는다. Backup은 이전 checkpoint payload와 그 anchor를
  유지하되 `loads_used`와 run peak는 current와 max-merge해 감소시키지 않는다.
- current 손상과 valid backup 조합은 backup을 current로 자동 복구하고 Unity에 한 번의 비차단 상태를
  투영한다. current/backup 모두 손상되거나 version이 unsupported이면 원본을 빈 run으로 대체하지 않고
  `ResetActiveRun` 또는 종료만 허용한다.
- `ContinueRun`은 마지막 완료 SavePoint를 복원하지만 명시적 checkpoint load가 아니므로 횟수를
  소비하지 않는다. `RunComplete`, 확정된 `RunFailed`, `AbandonRun`은 tombstone을 먼저 publish해
  continue source를 무효화하며, 새 run replacement는 명시적 `replace_active_run: true`가 필요하다.
- result stats는 runtime instance event를 보존하면서 직원 UUID별 한 행으로 집계한다. 마지막 배치 instance가 terminal HP/생존을 대표하고, 전투불능 여부와 누적 metric은 같은 직원의 모든 instance에서 합성한다.
- Unity는 MVP, 누적 피해량, 처치 수, 받은 피해량, 기본 공격/스킬 사용 횟수, 배치/철수 횟수, 배치 시간, 최종 HP/생존/전투불능 여부를 raw event log에서 재계산하지 않는다.
- event log는 현재 CombatResult 상세와 비권위 개발자 trace에만 사용하며, player gameplay history로 누적하지 않는다.
- `selected_event.bonus_objectives`는 PvE encounter가 명시한 추가 연구 목표의 결과 표시 source다.
- Unity는 bonus objective를 달성/미달성 표시와 presentation hint에 사용하되, 연구도 증가나 보상 지급을 로컬에서 재계산하지 않는다.
- 실제 연구 진행도는 combat result 완료 처리와 뒤따르는 `abnormality_research` snapshot이 source of truth다.

## Run State And Atomic Commit

이 장은 node resolution과 run state 사이에서 연구, 영구 도감, inventory, shop, reward와 직원
성장을 검증하고 한 번만 commit하는 runtime 경계를 소유한다. 각 기능의 플레이어-visible 의미는
[game_rulebook.md](game_rulebook.md)가 소유한다.

### Root Capability And Parameterized Admission

- Root `action_capabilities`는 현재 context에서 관련 없는 action을 생략하고, 관련 있지만 실행할 수 없는
  action을 stable typed reason으로 disabled한다. Parameterized action은 대응 payload가 하나라도 실행
  가능할 때만 family capability가 enabled되며, 이것이 모든 payload의 실행 가능성을 뜻하지 않는다.
- Maintenance projection은 `maintenance_options`의 Core-owned operation eligibility를 재사용한다.
  Skill-fragment의 upgrade/awaken/dismantle은 각각 `enhance`/`awaken`/`dismantle`, equipment의
  dismantle/enhance는 각각 `dismantle`/`enhance`에 대응한다. Maintenance context에서 대응
  `can_execute`가 하나도 없으면 `NoEligiblePayload`, context 밖이면 irrelevant다.
- Snapshot projection과 common preflight는 같은 action-operation mapping을 사용한다. Family capability가
  enabled여도 command는 선택한 identity와 최신 domain state를 기존 Maintenance validator로 다시
  검증한 뒤에만 mutation하며, 실패는 원자적으로 state를 보존한다.

### Facility Map Authority And Persistence

- Live `FacilityTemplateDatabase`의 template/version/semantic slots/directed connections가 topology의
  단일 authority다. `RunMap`은 선택한 `map_template_id`, `facility_template_version`과 slot별 확정 node,
  derived start/typed terminal identity를 저장하지만 runtime node-ID edge list나 depth/lane 좌표를
  저장하지 않는다.
- Navigation, node completion, manifestation과 recovery-path 평가는 current live template를 ID로
  resolve하고 node `slot_id`를 authored connection에 연결한다. 좌표, node 배열 순서, terminal까지의
  거리 또는 구 procedural graph를 fallback으로 사용하지 않는다.
- Facility generation은 mode/Floor schedule이 정한 profile과 pool에서 deterministic shuffle/cycle을
  소비하고 constrained assignment를 한 번 수행한다. 남은 template order, cycle, 사용 template와
  assignment된 current map은 `FacilityRunState`/`RunState`/checkpoint/active-run payload에 포함되며
  reconnect/load에서 weight로 재생성하지 않는다.
- Checkpoint import는 current catalog에서 template ID/version을 resolve하고 모든 authored slot이 정확히
  한 번 존재하는지, category allowed set, derived start IDs와 typed terminals가 exact 일치하는지 strict
  검증한다. 누락/추가/중복 slot, version drift와 terminal 역할 drift를 silent reset하지 않는다.
- Manifestation target은 Floor 생성 시 node identity/content/seed까지 precommit한다. 마지막 prerequisite
  성공 commit과 target reveal/navigation 갱신은 같은 world transaction이며 monotonic/idempotent다.
  Retreat/failure/cancel rollback과 reconnect는 concealed assignment를 보존한다.
- Generic capstone switch는 typed template/entry input을 모두 검증하고 완전한 map과 progression을 staging한
  뒤 원자 교체한다. BossOmen은 이 seam으로 `boss_omen_capstone`을 사용하며 기존 exploration map에
  node/connection을 append하지 않는다.
- Certified power는 현재 전투 진입 가능한 roster(`alive && physical_damage <= 4 && trauma < 100`)만
  합산한다. 경계 안의 현재 HP 및 physical damage/trauma 크기는 계수가 아니며 Critical, Breakdown,
  Dead는 제외한다. eligible power가 0인 roster의 relative threat는 `VeryHigh`다.
- Facility creation은 certified run peak에서 Floor anchor를 먼저 확정한 뒤 assignment profile의 required
  difficulty profile로 ordinary quota/role, fixed target power와 reward/wave/stat contract를 category/content와
  같은 transaction에서 저장한다. Manifested Combat/Boss에 difficulty authority가 없거나 live model/profile과
  다르면 preview와 실행은 실패하며 Floor 번호나 current roster로 fallback하지 않는다.
- `MapViewDto`는 `difficulty_band`, `relative_threat`, `readiness_warnings`만 투영한다. Internal node의
  model/profile/anchor/target과 exact multiplier는 wire에 포함하지 않는다. Relative threat/readiness는
  현재 roster에서 다시 계산할 수 있지만 저장된 node difficulty는 수정하지 않는다.

### Safe Zone Research Delivery Boundary

스킬 파편 연구 완료는 node activity 중 `pending_research_deliveries`에 queue되고, Core가 해결된
node content에서 `ViewingMap` planning으로 복귀하는 권위 transition을 commit할 때만 지급된다.

- `ResearchDeliveryPolicy` category allowlist와 node-entry delivery hook은 존재하지 않는다.
  SavePoint, HeadquartersContact, Shop, Reward, Maintenance, Event 진입은 pending을 읽거나 제거하지
  않으며 각 entry/state result에 `research_deliveries`를 싣지 않는다.
- 실제 지급은 staged `SkillFragmentInventory`에서 모든 pending identity/count를 검증하고 전부
  적용한 뒤에만 inventory를 교체한다. 하나라도 실패하면 node completion과 pending 제거를
  commit하지 않는다.
- planning-return delivery fact는 `NodeCompleted.research_deliveries` 또는
  `FloorAdvanced.research_deliveries`에만 있다. nested `CombatRewardsGranted`,
  `EmployeeRecruited`, `EmergencySuppliesGranted`는 기존 typed `completion` payload 안에서 같은
  완료 계약을 운반한다.
- `CancelSelectedNode`, checkpoint load, snapshot/reconnect, retryable defeat, `RetreatBattle`,
  `RunComplete`, `RunFailed`는 planning-return delivery authority가 아니다. recovery node resolution이
  곧바로 RunFailed가 되는 경우에도 pending은 지급되지 않는다.
- Gate confirm은 새 Floor의 `ViewingMap`에 도착하면서 지급한다. Event-started combat는 Event
  entry/전투 시작이 아니라 owning Event node가 소비되어 planning으로 돌아갈 때 지급한다.
- SavePoint completion은 delivery가 반영된 inventory를 checkpoint에 저장하므로 그 checkpoint를
  다시 load해도 같은 pending을 중복 지급하지 않는다.
- SavePoint completion은 staged recovery, pending delivery, node completion과 process-local checkpoint
  capture를 하나의 world transaction에서 검증·커밋한다. Gate confirm도 staged recovery, pending
  delivery와 Floor transition을 같은 transaction으로 처리한다. 실패하면 employee condition,
  inventory, map/Floor, active node와 checkpoint 어느 것도 부분 적용하지 않는다.
- 두 recovery는 같은 tagged `TraumaRecovery`를 사용한다. `CurrentPercent`는 현재 trauma 기준
  `ceil(trauma * percent / 100)`을 saturating 차감하며 Critical/Breakdown을 포함한 생존 직원만
  대상으로 한다. Dead와 physical damage는 바꾸지 않는다.
- Unity는 successful command result의 exact completion payload만 읽어 알림을 만들며, 뒤따르는
  root snapshot이나 Safe Zone 화면 진입으로 지급을 재추론하거나 command를 보내지 않는다.

### Global Encounter Compendium

영구 조우 지식은 run-local 대응 연구와 다른 game-global profile 상태다.

- 영구 상태 `EncounterCompendiumProfileState`는 abnormality/corrosion 각각의 canonical `discovered`/`defeated` set을 저장하고 `defeated ⊆ discovered`를 strict validation한다. 이름·위험 등급과 authored 순서는 현재 strict catalog에서 snapshot 투영 시 해결한다.
- 수집 source는 완료된 attempt의 `BattleEventLog::UnitSpawned`다. `Side::Opponent` 이면서 `BattleUnitSourceIdentity::Abnormality` 또는 `CorrodedEmployee`인 실제 spawn만 해당한다. preview, authored wave 참조, 시작 전 catalog preload는 발견이 아니다.
- 승리, 도전 횟수 소진 패배, fatal 패배처럼 combat node를 소비하는 결과의 권위 전환이 성공한 뒤 실제 spawn을 발견 union한다. Player 승리는 current primary ID를 발견·격파하고, 승리 event log에서 실제 `UnitDied`한 non-primary opponent source만 격파한다. retryable 패배와 `RetreatBattle`은 union을 호출하지 않는다.
- set union은 idempotent고 항목을 삭제하지 않는다. 새 result commit은 current catalog membership을 검증하지만 persisted profile import는 ID format/subset만 검증해 dormant ID를 보존한다. replacement load를 제공하지 않는다.
- 이 상태는 `RunState`, `RunCheckpointPayload`, `BattleEntrySnapshot`에 포함되지 않는다. checkpoint load,
  이전 run state load, 새 run, RunFailed, retreat와 ordinary non-battle rollback은 이미 커밋된 union을
  교체하거나 줄이지 않는다. Common transaction restore는 rollback snapshot의 빈 profile을 authority로
  사용하지 않고, restore 직전 live profile을 복원 state에 union한다.
- active-run transaction rollback snapshot도 compendium copy를 소유하지 않으며 restore는 같은 방식으로
  현재 monotonic profile을 보존한다. Active-run file publish 실패와 profile publish 실패는 서로의
  gameplay/file commit을 rollback하지 않는다.
- root `encounter_compendium` snapshot은 current catalog 전체를 authored 순서로 투영한다. 각 항목은 canonical ID와 `discovered`, `defeated`를 가지며 미발견 항목은 이름/위험 metadata를 생략한다. Dormant ID는 profile에 남지만 current projection에는 나타나지 않는다. game_server는 이 typed Core 필드를 그대로 전달하고 Unity는 local discovery를 재계산하지 않는다.
- Host는 OS application persistent-data 아래 local-install singleton profile을 version/checksum/current/rolling backup/temp/exclusive lock으로 관리한다. Backup 복구는 dirty 상태와 1회 알림을 만들고, write 실패는 최신 전체 in-memory union을 유지해 mutation/명시 retry/종료 flush 때 재시도한다. `AttachSession`은 readable durable profile을 Core에 merge한 뒤 root snapshot을 만들며, fatal profile은 원본을 보존하고 title gameplay를 retry/확인된 reset/exit 전까지 차단하는 recovery snapshot을 계속 제공한다. Dual corrupt/unsupported version을 빈 profile로 대체하지 않는다.
- developer `combat_traces`는 compendium을 읽거나 수정하지 않는다.

### Ordinary Inventory And Quantity Boundary

- `Inventory`의 ordinary slot occupancy는 owned equipment instance 수와 서로 다른 consumable
  definition stack 수의 합이다. Equipment material, skill fragment, fragment dust와 currency는
  포함하지 않는다.
- Equipment는 owned instance UUID로 유지한다. Consumable은 `definition_id` key의 단일
  `quantity: u32` stack이며 metadata UUID가 wire selector다. 별도 owned consumable UUID stream은
  없다.
- ordinary storage에는 fixed equipment/consumable cap과 capacity admission preflight가 없다.
  `InventoryFull`은 ordinary inventory grant 결과가 아니다.
- 저장된 `reserved_slot_count`는 50에서 시작해 성공한 새-slot commit이 경계를 넘을 때 최소
  10단위로 증가한다. 수량-only mutation과 제거는 값을 변경하지 않으며 collection memory
  capacity나 overflow storage로 사용하지 않는다.
- root inventory snapshot은 `reserved_slot_count`, `occupied_slot_count`, equipment instance
  배열, definition-id 정렬 consumable stack 배열을 투영한다. 각 consumable은 full post-state
  `quantity`를 필수로 가진다.
- 첫 consumable grant는 `inventory_diff.added`, 기존 stack grant와 수량이 남은 사용/판매는
  `updated`, 수량 0은 metadata UUID를 `removed`에 보낸다. Equipment `removed` UUID는 owned
  instance identity이므로 item kind가 identity 의미를 결정한다.
- `GrantConsumable`은 explicit positive `quantity`를 authoring/runtime contract로 가진다.
  Reward, Event, Shop, admin, combat result는 같은 `GrantExecutor`와 cloned `Inventory` stage를
  사용하므로 quantity와 reserve는 전체 transaction이 성공할 때만 함께 commit된다.
- `RunCheckpointPayload`와 `BattleEntrySnapshot`은 `Inventory` 자체를 clone하므로 quantity와
  reserve high-water를 정확히 저장/복원한다.

### Shop Session, Quote, And Transaction Boundary

- `ShopDatabase`는 reusable `merchandise_presets`, Shop별 preset composition/local entries,
  stable `offer_count`, default buy percentage, optional refresh/sell policy, encounter-level Shop
  pools를 가진다. Prototype `ShopType`, fixed visible/hidden stock, `can_reroll` schema는 없다.
- Initial page와 refresh page는 authoritative run/Floor/node/Shop/page state에서 weighted
  sampling without replacement로 생성된다. 후보는 typed merchandise identity로 정렬된 뒤
  sampling되며 page 안에 duplicate가 없다.
- `ShopSessionState`는 current offers, deterministic offer identity, page index, purchased identity
  set, refresh uses와 policy를 소유한다. Root snapshot/reconnect는 저장된 offer를 읽을 뿐 다시
  추첨하지 않는다. Floor-local `RunState.shop_sessions`와 `RunCheckpointPayload.shop_sessions`가
  page/purchase/refresh continuation을 정확히 저장·복원한다.
- Purchase payload는 definition UUID가 아니라 current `offer_id`를 보낸다. Core는 offer의 exact
  total quote와 funds를 확인하고 공통 `GrantExecutor` stage, currency deduction, offer removal,
  purchased history를 하나의 commit으로 교체한다.
- `RefreshShop`은 `RerollShop`을 대체한다. Optional policy, remaining uses, funds, full-page candidate
  sufficiency를 같은 evaluator로 확인한 뒤 cost/use/page를 한 번에 commit한다. 실패 시 셋 모두
  불변이다.
- Sale payload는 `{ item_uuid, quantity }`다. Equipment identity는 owned instance UUID이고 quantity
  1만 가능하다. Consumable identity는 static metadata UUID이고 requested quantity만 해당 consumable definition stack에서
  제거한다. Intrinsic removability, tag acceptance, checked quote와 currency overflow를 commit 전에
  검증한다.
- Buy quote는 checked `ceil(base_price * buy_percent / 100)`, sale unit quote는 checked
  `floor(base_price * sell_percent / 100)`다. Startup cross-catalog validation은 모든 reachable
  merchandise에 `max_live_sell_quote < min_live_buy_quote`를 강제한다.
- `selected_event.type == "shop"`은 full display offer, purchase eligibility, refresh status/reason,
  sale candidates/quotes를 투영한다. Context capability는 실행 가능한 payload 존재 여부를 나타내고
  commit은 payload projection과 같은 evaluator를 다시 실행한다.

### Monolith Run State And Conversion Boundary

- `MonolithRunState`는 inventory와 분리된 abnormality-keyed run ledger다. 자연 Elite outcome, victories,
  purchase/uses, active converted node, residual sequence, exchange state, one-shot Shop guarantee와 dormant
  corrupted key를 소유한다.
- 이 state는 `RunState`, `RunCheckpointPayload`, active-run payload와 `BattleEntrySnapshot`에 포함된다.
  Checkpoint load는 함께 되감지만 `loads_used`와 local encounter compendium profile은 되감지 않는다.
- Endless natural Elite assignment는 해결된 identity와 같은-Floor duplicate를 hard-exclude한다. 전체
  후보 소진 시 soft constraint를 reset하지 않고 authored Normal encounter와 ordinary Standard
  difficulty로 node content를 강등한다.
- Floor 4+ pooled Shop resolution만 RunPolicy가 지정한 wandering merchant로 교체한다. Standard의 Shop
  pool은 별도 shop ID를 사용한다. First page는 typed `ShopMerchandise::Monolith`를 최대 둘 포함하고,
  refresh는 item-only다. Monolith purchase는 inventory grant 없이 currency/offer/ledger를 clone-stage한다.
- `game_state_context.monolith_conversion`은 NodeConfirm의 canonical payload projection이다.
  Option의 `enabled`와 선택적 `disabled_reason`도 Core authority이며, 진행 중인 identity는 다른 사용 가능
  option과 함께 투영될 때 `진행 중: <slot>` 사유를 가진다.
  `ConvertMapNodeWithMonolith { node_id, abnormality_id }`는 같은 projection을 재검증하고 Available Normal
  Combat의 topology를 보존한 채 encounter/difficulty와 one-use consumption을 transaction으로 commit한다.
- 변환된 NodeConfirm의 `game_state_context.monolith_remanifestation`은 identity/name/risk/stage/max attempts와
  qualitative `reward_tier`(`enhanced|final_enhanced`)를 명시한다. Exact multiplier는 public DTO가 아니며
  Unity는 combat preview나 local catalog로 이를 역산하지 않는다.
- 변환은 difficulty에 required typed `monolith_stage`를 기록하고 ledger stage와 cross-validation한다.
  P04 `MonolithRemanifestation` role과 RunPolicy stage multiplier는 단일 resolver에서 target, 적 최대
  HP/공격/방어, reward에 먼저 합성한다. 합성값은 한 번 적용·반올림하며 generated wave budget과
  extra waves는 stage로 바꾸지 않는다. Missing/legacy stage, role-stage 불일치, ledger 불일치는
  checkpoint/continue와 live preview/reward에서 fail closed한다.
- Remanifestation victory/failure resolution, unique fragment grant, residual creation과 exchange는 모두
  cloned state 또는 world transaction을 먼저 검증한다. Invalid/stale payload는 active mapping, currency,
  uses, reward와 residual을 부분 변경하지 않는다.
- Residual threshold의 next-Floor guarantee는 facility generation requirement다. Shop-tag slot을 정확히
  하나 고정하고 새 Floor commit 시 소비하며 같은 run에서 다시 발행하지 않는다. Floor를 떠날 때
  unresolved node-local mapping은 제거하지만 commit된 use는 환불하지 않는다. Exchange VN은 merchant Shop session보다 앞선 blocking
  overlay이며 accept는 oldest four를 원자 소비하고 decline은 terminal state다.
- Corrupted key에는 현재 consumer가 없다. FinalBoss classification, Endless completion, hidden capstone과
  hidden-boss content는 이 runtime 계약의 현재 구현 범위가 아니다.

### Reward And Employee Growth Boundary

Reward definition은 typed `RewardEffect` 목록이고 실제 지급은 `GrantExecutor` 하나가 수행한다.

- Reward session과 combat result는 inventory, skill fragment, roster, UUID manager, Enkephalin을 clone/stage한 뒤 모든 effect가 성공한 경우에만 한 번 교체한다. 중간 effect 실패는 앞선 XP/재화/item mutation을 commit하지 않는다.
- Live combat encounter는 `ClaimAll`만 허용한다. CombatResult의 `ChooseOne`은 validation/runtime error이며 Core가 자동 지급 reward를 로컬 선택 UI로 바꾸지 않는다.
- Map Reward node는 authored pool에서 `Experience` grant kind를 제외하고 node seed로 definition 하나를 고른 뒤 `ClaimAll` session으로 고정한다. `GrantEquipmentFromPool`도 stage/reward/effect identity에서 파생한 deterministic seed를 사용한다.
- `GrantExperience.amount`는 total pool이다. 대상 UUID를 오름차순 정렬, dedupe, roster membership filter한 뒤 quotient를 분배하고 remainder는 낮은 UUID부터 1씩 지급한다. `CombatParticipants`와 `SelectedEmployee`는 explicit `GrantExecutionContext`가 없으면 실패한다.
- `AliveRoster`는 physical damage가 6 미만인 직원만 검사한다. combat eligibility, trauma, roster slot은 별도 조건으로 암묵 추가하지 않는다. `AllRoster`는 condition과 무관하다.
- `EmployeeExperienceDiffDto`는 authored reward effect의 exact per-employee amount와 level/experience before/after다. 성공/일반 Reward result는 이 diff를 transport하고, game_server는 의미를 재해석하지 않는다. 현재 Unity에는 전용 XP diff reader가 없으며 root roster snapshot이 최종 상태 source다.
- 커밋된 전투 결과의 survival XP는 `result_stats.employees`의 employee-level `survived`와 staged roster의 `Alive`를 함께 검사하고 각 직원에게 `post_battle_survival_xp`를 `SelectedEmployee` effect로 한 번 적용한다. 승패 모두 같은 post-battle stage를 사용하고 `RetreatBattle`은 entry snapshot 복원으로 이 mutation을 폐기한다.
- Survival XP는 authored combat reward의 `employee_experience_diffs`에 합쳐지지 않는다. `NodeOutcomeEmployeeChange.experience_before/after`와 후속 roster snapshot이 그 최종 변화를 표현한다.
- `Employee.experience`는 current level 안의 잔여 XP다. `add_experience_with_policy`는 saturating add 후 `xp_required_for_next_level(level)`을 반복 차감하므로 한 grant가 여러 level을 넘을 수 있다. authored curve를 넘으면 마지막 requirement를 반복한다.
- `Employee.level`이 persistent 성장 source이고 `battle_tier_by_level`은 level에서 파생한다. 현재 `BattleUnitDraft.level`/snapshot `battle_tier`는 분류 projection이며 stat pipeline의 HP/attack/defense multiplier가 아니다.
- `GrowthStack`은 level/XP와 별도 내부 draft state다. 현재 `KillStack`만 final stat pipeline에서 flat attack으로 적용되고, production gameplay에는 stack producer가 없다. Level-up은 GrowthStack을 변경하지 않는다.

현재 live 수치는 `game_resources/data/run/policy.ron`이 소유한다. Canonical 문서는 수치의 현재 의미를 설명하지만 runtime constant나 Unity-local 계산으로 복제하지 않는다.

## Visual Novel Runtime

이 장은 top-level VN state, pre-run flow와 trigger 평가·반복·checkpoint 정책의 runtime ownership을
소유한다. Scene의 player-facing 콘텐츠 의미와 정확한 Unity wire shape는 각 인접 canonical을 따른다.

### Presentation And Pre-Run Flow

Visual Novel runtime은 route나 run 내부 상태가 아니라 core 최상위 runtime 상태다.

- Canonical owner는 `GameCoreState.visual_novel`이다.
- `RunState`와 run checkpoint는 active VN session을 소유하거나 복원하지 않는다.
- Root `state_snapshot.visual_novel`과 `battle_update.visual_novel`은 같은 top-level VN projection에서 나온다.
- Unity는 VN 표시 여부를 `selected_event`나 현재 route로 추론하지 않고, public snapshot의 `visual_novel != null`만 본다.
- root snapshot의 Event content는 `{ type: "event", node_id, event }`이고 `EventState` command
  result는 같은 `event` inner DTO를 직렬화한다. game_server와 Unity는 별도 Event envelope
  alias나 JSON repair를 만들지 않는다.
- Event inner DTO는 `event_id`, `current_scene_id`, ordered
  `presentation.panels[] { image_id, text_ids[] }`, tagged `interaction`, optional
  `committed_check`로 구성된다. `interaction`은 `advance` 또는
  `employee_check_choices { choices[] }`다. 각 choice가 `choice_id`, `label_id`, `aptitude`, `dc`,
  optional `required_choice_key`, eligibility와 해당 choice의 `employee_candidates[]`를 소유한다.
  Candidate는 identity/name/eligibility, employee/equipment/total modifier, typed
  `modifier_components`, optional `choice_key_evidence`와 exact chance를 포함한다. Scene-common
  `required_roll`과 global employee candidate list는 없다.
- Core는 stored Profile aptitude, employee eligibility, equipped-owned equipment/key 평가, 단 한 번의
  node-seeded d20, chance enumeration, committed performer/result, success action과 failure scene의
  유일한 source of truth다. `committed_check`는 source scene/choice/employee, aptitude/DC/raw,
  typed components/key evidence, modifier/total과 outcome을 저장·직렬화한다. Unity는 panel/text
  cursor와 resolve 전 choice/employee selection만 로컬로 소유한다.
- Event 진행 명령은 scene-qualified `AdvanceEventScene`과
  `ResolveEventChoiceCheck(node_id,event_id,current_scene_id,choice_id,employee_uuid)`뿐이다.
  exact committed identity의 duplicate resolve는 결과 장면에서 capability가 노출되지 않더라도
  저장된 동일 결과를 반환하며 reroll/regrant하지 않는다. 하나라도 다른 replay는 거부한다.
- starter와 본사 채용의 authored identity는 `employee_definition_id`, roster instance identity는
  `roster.employees[*].uuid`다. 본사 채용 명령은 `employee_definition_id`, 시작 선택 명령은
  `employee_definition_ids`를 사용한다. candidate DTO와 roster DTO는 같은 authored Profile을
  투영하며 UUID나 trust state에서 personality를 재생성하지 않는다. reward option instance identity는
  `rewards[*].uuid`, 전투 결과의 대표 환상체 identity는 `primary_abnormality_id`다.
- authored personality는 Profile에만 있다. run-local `EmployeePsychology`는 -100~100 trust score,
  derived band, semantic memory와 bounded recent event receipt를 소유한다. Psychology event는 typed
  authoritative source identity로 직원별 idempotent하게 적용하고, 같은 kind/subject 기억은 occurrence와
  first/last source를 병합한다. checkpoint와 battle-entry rollback은 roster clone으로 이 aggregate를
  정확히 보존하며 사망 recovery도 roster record를 제거하지 않는다.
- 현재 production Psychology mutation은 없다. SavePoint/Gate recovery는 Psychology event를
  생산하지 않는다. dormant high-risk refusal, dialogue cue, combat/trauma modifier policy는
  runtime/public DTO 계약이 아니다.
- `RecruitmentState`는 run-seeded ordered queue와 현재 HQ active session의 definition IDs를 유일하게 소유한다. 시작 미선택 starter와 non-starter definitions를 queue에 포함하고, roster와 합쳐 unified catalog의 각 definition을 정확히 한 번 소유해야 한다.
- HQ snapshot은 active session ID를 unified catalog에 투영한다. 채용은 선택하지 않은 IDs를 session order로 queue tail에 반환하고 선택 ID만 roster로 이동한다. 응급지원과 HQ shop은 session 전체를 tail에 반환한다. checkpoint와 Floor carry-forward는 aggregate 순서를 그대로 보존한다.
- `UseConsumableItem` request의 직원 key는 `target_employee_uuid`이며 active modifier의 표시
  이름은 snapshot의 `active_consumable_modifier.name`이다.
- Physical Treatment는 살아 있는 직원이면 physical damage 0에서도 structurally eligible하며 성공한
  사용은 실효 회복량과 무관하게 stack을 소비한다. 다른 지속 소비 아이템은 기존 active modifier를
  원자적으로 교체하고 `replaced_modifier`를 결과에 기록하며 별도 pre-confirm을 요구하지 않는다.
- `visual_novel.source.type`은 origin metadata다. HQ briefing, event node, battle, map, loadout, tutorial 같은 source는 overlay 표시 여부가 아니라 command 구성과 완료 hook을 위한 정보다.
- `vn_id`, `background_id`, `script_id`, `speaker_id`, `portrait_id`, choice `label_id`, hint id는 Unity-local resource/localization id다. Core는 실제 대사 본문, 이미지, 사운드, cue stream을 내려주지 않는다.
- `speakers`는 배열이다. Core는 scene initial speaker/portrait/slot/active 상태를 내려주고, Unity는 local VN script resource를 통해 세부 cue와 line progression을 처리한다.
- VN definition은 scene/presentation graph만 소유한다. Session `blocking`은 matching
  `VisualNovelTrigger.blocking` 하나에서만 오며 definition-level default/fallback은 없다.

Pre-run HQ briefing은 새 게임 시작 flow의 일부다.

- `StartNewGame(game_mode)`는 run을 즉시 만들지 않고 run-start flow를 시작한다.
- 성공 결과는 `RunStartFlowStarted { game_mode, visual_novel }`이다.
- `StartNewGame` 직후 core state는 `PreRunBriefing { game_mode }`이며, snapshot에는 run/map이 없다.
- `PreRunBriefing` 중 blocking VN이 active이면 Unity는 VN progression command만 노출한다.
- HQ briefing VN이 완료되면 core가 unified EmployeeDefinition catalog의 `starter_eligible` definition을 준비하고 `SelectingStarterEmployees`로 전환한다.
- 실제 `RunState`는 `SelectStarterEmployees` 성공 시 `pending_game_mode`로 생성된다.
- mode-specific HQ VN은 live `VisualNovelTrigger` catalog의 `RunStart { game_mode }` trigger로 시작한다. Core는 run-start VN id를 하드코딩하지 않는다.
- mode-specific HQ VN trigger 또는 trigger가 참조하는 VN id가 live catalog에 없으면 invalid live data다. Core는 briefing을 조용히 skip하거나 다른 VN으로 fallback하지 않는다.

### Trigger Runtime

VN trigger는 core 내부 runtime condition을 live-authored VN session으로 연결하는 data-driven layer다.

- Trigger live data는 `game_resources/data/visual_novel_triggers/base.ron`에서 로드되고 `GameDataBase::load_live_embedded()` 경로로 검증된다.
- Core가 trigger 조건을 평가한다. Unity는 trigger 조건, priority, repeat history, checkpoint policy를 보거나 재계산하지 않는다.
- Public DTO는 기존 `state_snapshot.visual_novel`과 `battle_update.visual_novel`만 사용한다. Trigger id, priority, repeat policy, checkpoint policy, consumed history는 Unity-facing contract가 아니다.
- `blocking`은 trigger-owned authoring이다. Core는 matching trigger 값을 session/snapshot에
  그대로 commit하며 VN definition에서 재정의하거나 fallback하지 않는다.
- Initial source set은 `RunStart`, `MapNodeEntered`, `BattleStarted`, `BattleTimeReached`다.
- Event node와 VN overlay는 분리된 개념이다. Event scene 진입은 VN trigger source가 아니며
  Event presentation은 typed `selected_event.event` route에서만 진행한다.
- Trigger match ordering은 higher `priority` first, same priority는 trigger id ascending이다.
- Active VN이 있으면 새 VN session은 interrupt하지 않고 `VisualNovelRuntimeState.queue`에 들어간다. Active session 완료 시 queued session이 승격된다.
- Trigger consumed timing은 VN session이 successfully activated 또는 queued 된 시점이다. 단순 match만으로 consumed 처리하지 않고, VN 완료까지 기다리지도 않는다.
- Repeat history는 checkpoint policy별로 분리된다. `PersistAcrossCheckpointLoad` history는 top-level runtime에 남고, `RewindWithRunCheckpoint` history는 run checkpoint payload와 함께 저장/복원된다.
- Battle trigger는 produced battle update 이후 평가하고, 새 VN projection을 같은 `battle_update.visual_novel`에 붙인다. 새 blocking VN은 다음 server tick부터 전투 진행을 멈추며, 이미 산출된 tick을 되돌리지 않는다.
- `BattleTimeReached { battle_time_ms }`는 `previous_battle_time_ms < threshold <= current_battle_time_ms` crossing일 때만 match한다.

## Unity-Facing Projection Boundaries

이 장은 Core가 계산해 Unity에 투영하는 actor identity, HUD 정책, range preview와 combat preview의
생산자 경계를 소유한다. JSON field shape와 소비 순서는 외부 Unity canonical이 소유한다.

### Battle Actor Identity And HUD

전투 유닛 HUD 표시 정책과 actor identity는 core가 내려주는 DTO가 source of truth다.

- Unity는 `checkpoint.units[*].hud.bar_mode`와 `hud.threat_class`를 읽어 HP bar 또는 HP+공명 bar를 그린다.
- Unity는 owner/role/base_uuid/encounter를 조합해 HUD 정책을 추론하지 않는다.
- live actor 정체성은 `UnitSpawned.unit_source`와 `checkpoint.units[*].unit_source`가 제공한다.
- Unity는 선택 패널, actor visual, 디버그 표시, 복구 시 `unit_source`를 사용하고, wave 순서나 `base_uuid` 역조회로 적 종류를 추론하지 않는다.
- `unit_source`는 actor identity source이고, `hud.bar_mode`는 HUD bar source다.
- 침식 직원/환상체/방어 오브젝트 여부를 보고 Unity가 HUD bar 정책을 다시 계산하지 않는다.
- Battle actor HUD는 유닛 아래쪽 world anchor를 `Camera.WorldToScreenPoint`로 투영해 dedicated
  screen-space overlay HUD layer에 표시한다.
- `hud.threat_class`가 높을수록 HUD 크기와 bar 두께가 커지며, HP/공명 색상은 threat class로 바꾸지 않는다.
- HUD 자체는 screen-space UI이므로 화면 기준으로 반듯하다. World-space canvas/mesh 또는
  카메라 billboard/up-vector 정렬을 사용하지 않는다.
- 적군 공격 범위는 플레이어 조작 preview가 아니므로 normal/checkpoint unit DTO에
  `range_previews`를 내려보내지 않는다. 해당 필드는 `owner == Player` unit에만 존재한다.
- 보스 경고, 광역 스킬 telegraph, debug처럼 시각적 경고가 필요한 경우에만 core가 실제 피해/효과가 적용될 타일을 별도 event/DTO로 내려보낸다.

### Range Preview Boundary

플레이어 기본 공격/스킬 범위 overlay는 core가 계산한 `range_previews` 최종 cell이 source of
truth다. 배치 전에는 `DeploymentRangePreview.payload.facings.*`, 배치 후에는
`checkpoint.units[owner == Player]`가 운반한다.

- Unity는 `defense_tile_range`, `effective_weapon_profile`, `effective_basic_attack`, skill catalog range metadata를 조합해 범위를 재계산하지 않는다.
- `owner == Player` checkpoint unit은 `range_previews`가 필수이고, non-player unit은 필드가
  없어야 한다. Unity는 적군 필드를 compatibility로 받아들이거나 빈 preview로 보존하지 않는다.
- 범위 overlay는 이동 가능 tile 표시가 아니다.
- 전장 밖/void/invalid tile은 제외하지만, obstacle/blocked tile은 기본 공격과 스킬 범위 표시에서 제외하지 않는다.
- 장애물은 지상 이동/배치/pathfinding/충돌에 영향을 주는 정보다.
- 실제 피격 가능성은 공격/스킬의 target validation, 공중 대상 가능 여부, 유효 hostile target 규칙이 판단한다.

### Combat Preview Boundary

`CombatPreview`는 node confirm, preview panel, battle setup preparation에 쓰이지만, 모든 내부 계산 필드가 Unity-facing/stored JSON 계약은 아니다.

- `enemy_stat_scale`은 floor scaling을 enemy spawn draft에 적용하기 위한 core 내부 계산값이다.
- `enemy_stat_scale`은 serialized preview JSON에 포함하지 않는다.
- serialized `CombatPreview`를 다시 읽어 battle setup source로 사용하지 않는다.
- Unity 위험도 표시는 raw scale 값을 노출하지 않고 명시적인 표시용 DTO 계약을 사용해야 한다.

Unity-facing DTO shape, command result, battle update cadence, resync 세부 계약은 외부 canonical `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`와 `/mnt/f/unity projects/ark/docs/unity_core_contract.md`를 따른다.
