# Implementation Boundary Audit

이 문서는 2026-08-12 baseline working tree에서 직접 재검증한 current command/data-to-authority,
persistence, server와 접근 가능한 Unity consumer 경계를 finding별로 누적한다. 룰북의 승인된 목표,
현재 실제 동작과 제안 구현은 서로 분리한다.

## Coverage Registry

| Finding | Bounded unit | Current validity | Boundary status |
| --- | --- | --- | --- |
| `GRB-M-001` | Standard lifecycle/profile | confirmed | traced |
| `GRB-M-002` | Research/fragment economy | confirmed | traced |
| `GRB-M-003` | Standard lifecycle/profile | confirmed | traced |
| `GRB-M-004` | Standard lifecycle/profile | confirmed | traced |
| `GRB-M-005` | Elite/Monolith/Sweep | confirmed | traced |
| `GRB-M-006` | Elite/Monolith/Sweep | confirmed | traced |
| `GRB-M-007` | Elite/Monolith/Sweep | confirmed | traced |
| `GRB-M-008` | Research/fragment economy | confirmed | traced |
| `GRB-M-009` | Research/fragment economy | confirmed | traced |
| `GRB-M-010` | Boss pools/Omen lifecycle | confirmed | traced |
| `GRB-M-011` | Boss pools/Omen lifecycle | confirmed | traced |
| `GRB-M-012` | Boss pools/Omen lifecycle | confirmed | traced |
| `GRB-M-013` | Boss pools/Omen lifecycle | confirmed | traced |
| `GRB-M-014` | Elite/Monolith/Sweep | confirmed | traced |
| `GRB-M-015` | Fragment placeholder conversion | confirmed | traced |

## Bounded Unit A — Standard Lifecycle And Global Profile

### Shared target authority

`docs/game_rulebook.md`의 2026-08-12 승인·구현 대기 주석은 Standard 튜토리얼 재구성,
Endless 자동 완료 폐기와 영구 개인 기록을 승인된 목표 정책으로 명시한다. 세 항목의 current
canonical은 충분히 명확하다. Audit/user resolution은 runnable bootstrap tutorial을 `ST1-NT60`, scripted
fragment를 none, 기존 local-install profile을 bounded V1 preservation으로 닫았다. `ST1-NT60` 값은
production authoring/validation에 쓰는 minimum fixture이지 현재 live 상태나 final content authority가 아니다.

### `GRB-M-001` — Standard fixed tutorial

- **Current validity:** confirmed. 같은 commit의 선행 감사 결론을 복사하지 않고 current
  `GameCore::handle_start_new_game` → starter selection → `generate_current_floor_map`을 다시 읽었다.
- **Current actual path:** `core_enhanced/src/game/world.rs:632`는 `GameMode`를 받아 profile gate 없이
  pre-run flow를 연다. Starter commit은 `RunProgression::new` 뒤
  `world/node_flow.rs:316`의 `generate_current_floor_map`을 호출한다. 이 함수는 Standard에도 certified
  power authority를 초기화/manifest하고 `FacilityMapGenerator::generate_with_requirements`와
  `assign_map_encounters`를 호출한다.
- **Live data/loader:** `game_resources/data/map/node_assignment_profiles.ron`의 Standard schedule은
  Floor별 template pool/profile을 고르지만 완전한 authored node/content sequence가 아니다.
  Floor 1/2의 `terminal_guard`는 둘 다 `combat_elite`이고 Floor 3은 weighted constraints다.
  `facility_generator.rs:83`은 pool candidate와 deterministic assignment를 순회하고
  `assign_combat_difficulties`로 shared difficulty를 합성한다.
- **Terminal encounter:** `map/progression.rs:86-106`은 final Standard floor terminal을 Boss로,
  이전 floor guard를 Elite로 만든다. `world/map_encounters.rs:331-345`는 final Standard Boss를
  `PveEncounterClass::FinalBoss`로 고정한다. Current tests
  `final_standard_floor_boss_node_selects_final_boss_encounter`와
  `boss_completion_advances_standard_floors_until_run_complete`가 이 반대 계약을 assertion한다.
- **Persistence/server/Unity:** generated `RunMap`, difficulty와 encounter payload는 기존 active-run
  checkpoint/DTO를 통과한다. Server/Unity는 assigned map을 소비하며 tutorial authored identity로
  다시 해석하는 별도 authority가 없다.
- **Bootstrap acceptance fixture:** `ST1-NT60`은 Standard Floor 1 Normal, Floor 2 Porcubbus Elite, Floor 3
  Nothing There NormalBoss를 authored fixed map/order/tuning으로 실행한다. F3 boss actor에만 base
  HP/ATK/DEF/MR 60%에서 유도한 exact override `720/53/36/33`을 적용하고 다른 actor/node/reward와 다른
  Nothing There 출현은 100%다. Standard difficulty는
  certified power에 합성되지 않고, Floor 3 승리 transaction이 run 완료와 tutorial flag commit을 함께
  만든다. Scripted fragment reward는 없다.
- **Content authority limit:** 위 identity/tuning/slots는 runtime의 fixed/no-adaptive publication과 terminal
  transaction을 검증하는 최소 fixture다. 별도 content Goal이 이 system invariant를 유지하면서 final
  tutorial topology/encounter/tuning/reward/script/presentation을 정한다.
- **Implementation boundary:** 새/별도 Standard authored template+assignment identity와 strict
  validation, F3 fixed boss actor의 four-axis exact override, generator의 mode-specific fixed publication,
  Standard difficulty bypass, final terminal
  class/commit 변경, explicit no-scripted-fragment authoring과 active-run/save 검증, target test replacement가
  필요하다. Current generic `BattleUnitStatScale`은 magic resistance가 없고 encounter-wide이므로 이
  override의 구현 수단이 아니다. Map DTO shape 자체는 기존 primitive 재사용 가능성이 높지만 profile commit은
  `GRB-M-004`와 coimplementation해야 한다.

### `GRB-M-003` — Endless auto-completion

- **Current validity:** confirmed.
- **Current actual path:** `world/combat.rs:3018-3048`의 player-victory plan은 Endless non-Monolith
  승리에서 legacy research를 갱신하고 `all_response_complete()`를 계산한다. 결과 commit의
  `world/combat.rs:3233-3237`은 recovery restriction이 없고 이 값이 true이면
  `GameState::RunComplete`와 `BehaviorResult::RunComplete`로 전이한다.
- **Live data/consumer:** production abnormality/research policy가 completion set을 제공하고 server의
  일반 `RunComplete` mapping은 mode별로 이를 억제하지 않는다. Existing test
  `endless_final_response_completion_returns_run_complete_without_planning_delivery`는 정확히 이
  behavior를 보호한다.
- **Approved target acceptance:** Endless victory/result commit은 어떤 fragment/response/FinalBoss
  ledger 완성으로도 `RunComplete`를 만들지 않는다. 자발적 포기 또는 확정된 run failure만 run을
  종료하고 정상 node/floor progression을 계속한다.
- **Implementation boundary:** research authority 제거 package에서 `endless_response_complete` 계산,
  return-to-planning 분기와 RunComplete branch/test를 제거해야 한다. Public `RunComplete` wire type은
  Standard의 정상 terminal result에 계속 필요하므로 DTO 자체 제거가 아니라 producer 조건 분리다.

### `GRB-M-004` — Tutorial unlock and permanent records

- **Current validity:** confirmed.
- **Current profile authority:** `core_enhanced/src/game/encounter_compendium.rs:19-26`의 유일한
  game-global gameplay profile은 abnormality/corrosion discovered/defeated set 네 개뿐이다.
  `GameCore::new_with_encounter_compendium_profile`, merge/reset와 combat-result discovery commit만
  존재하며 tutorial flag, highest Floor, deathless streak와 all-FinalBoss record state/mutation은 없다.
- **Persistence:** `game_server/src/persistence/encounter_compendium_store.rs`는 version 1,
  `deny_unknown_fields` envelope로 위 profile만 current/backup에 durable publish한다. Active-run store와
  별도 lifetime이지만 target fields를 보존할 schema/version/migration은 없다.
- **Command/admission:** `GameCore::handle_start_new_game`은 Standard/Endless 모두 같은 entry를
  profile predicate 없이 연다. Core action capability에도 mode별 payload eligibility가 없다.
- **Unity consumer:** `GameCommandBus.StartNewGame`은 두 typed mode를 모두 허용한다.
  `RunFlowScreenComposer.cs:138-166`은 main menu에 Standard와 Endless command를 동시에 추가하고,
  `ArkClientContractTests.cs:6578-6588`은 두 버튼을 assertion한다. Fresh Unity runner는 unavailable이다.
- **Approved target acceptance:** Standard Floor 3 NormalBoss 승리 최종 commit만 tutorial flag를
  단조 true로 만들고, false이면 Endless command를 Core가 거부하며 Unity는 disabled reason을 표시한다.
  Endless의 highest Floor, deathless Floor streak와 one-run all-FinalBoss achievement는 달성 즉시
  active-run/checkpoint와 분리된 local profile에 단조 commit되고 Standard/checkpoint rollback은 이를
  바꾸지 않는다.
- **Implementation boundary:** game-global profile type/store/version, atomic Core-result→host durable
  publish seam, snapshot/capability/disabled reason, server startup/reset/backup behavior와 Unity menu/read
  model을 coordinated cutover해야 한다. 기존 compendium을 같은 profile로 확장할지 별도 store로 둘지와
  이미 존재하는 version-1 compendium 보존 정책은 rollout decision이다. Standard completion flag commit은
  `GRB-M-001`, all-FinalBoss record는 `GRB-M-012`의 cycle/completion identity를 prerequisite로 가진다.

### Unit A evidence limit

Static runtime, live RON, server persistence와 Unity source/assertion으로 boundary를 닫았다. Fresh
runtime probe는 current semantics를 바꿀 불확실성이 없어 실행하지 않았다. Unity runner는 unavailable이며
Unity behavior를 fresh test로 검증했다고 주장하지 않는다.

## Bounded Unit B — Research Removal And Encounter Fragment Economy

### Shared target authority

룰북은 research/pending delivery를 전부 폐기하고, Endless Elite/NormalBoss/FinalBoss 승리 source가
정확한 `fragment_id` 또는 이후 `repeat_reward`를 결과 commit에서 직접 지급하도록 승인했다. Mastered
Elite 소탕만 owned count가 0일 때 `repeat_reward` 대신 한 copy를 재획득하는 예외를 가진다.
정확한 fragment identity와 reward quantity는 live RON authoring에 위임되지만 source eligibility,
run-local dedupe와 checkpoint rollback 의미는 현재 gameplay target이다.

### `GRB-M-002` — Legacy research and pending delivery cutover

- **Current validity:** confirmed.
- **Run authority:** `world/state.rs:1193-1207`의 `RunState`는
  `RunAbnormalityResearchState`와 encounter history를 소유하고 Floor transition에 복제한다.
  `abnormality_research.rs:11-157`은 환상체별 point/required/response/suppression win/completion/
  unique-fragment state와 victory mutation, `all_response_complete`를 구현한다.
- **Fragment inventory authority:** `skill_fragment.rs`의 `SkillFragmentInventory`는 owned stack과
  upgrade/awakening state 외에 `research_progress`, `research_completion_count`와
  `pending_research_deliveries`를 저장한다. Completion threshold를 넘으면 pending을 증가시키고
  `deliver_pending_research_with_policy`가 planning return에서 owned copies로 바꾼다.
- **Reachable transaction:** `world/combat.rs:3018-3118`은 승리에서 research를 stage하고 old
  abnormality fragment를 grant한다. `world/node_flow.rs:210-225`, `778-805`, `992-1025`는 node/floor
  planning return마다 pending delivery를 stage/commit한다.
- **Live data:** `game_resources/data/run/policy.ron:16-24`에 required/gain/repeat dust가 있고,
  `abnormalities/base.ron`의 모든 current abnormality가
  `response_complete_skill_fragment_id`를 가진다. `events/rewards/base.ron`에는 여러
  `GrantSkillFragmentResearch` effect가 production pool에 있다.
- **Save/checkpoint:** `RunCheckpointPayload`는 `RunState` research fields와 whole
  `SkillFragmentInventory`를 `deny_unknown_fields`로 저장한다. `world/active_run.rs`의 current strict
  active-run payload version은 3이며 old version을 reject한다. 따라서 research field 제거는 단순
  in-memory cleanup이 아니라 active-run/checkpoint serialized cutover다.
- **DTO/server/Unity:** root snapshot은 `abnormality_research`; inventory snapshot은 research progress와
  pending deliveries; reward results는 `skill_fragment_research_diffs`; planning-return results는
  `research_deliveries`를 직렬화한다. Server test
  `planning_return_preserves_research_deliveries_before_trailing_snapshot`이 ordering을 보호한다. Unity
  `ResearchDeliveryNotificationModel`은 `NodeCompleted`/`FloorAdvanced` exact field set과 sorted delivery
  entries를 strict parse하고 EditMode test/fixture가 이를 보호한다.
- **Approved target acceptance:** 새 run에 research state가 없고 승리/Reward/HQ/planning return이
  research나 pending delivery를 만들지 않는다. Inventory/DTO/save/server/Unity에 old fields/readers가
  없고 Endless completion이 research에 의존하지 않는다. Fragment upgrade/awakening/dust와 owned stack은
  그대로 유지한다.
- **Implementation boundary:** RunPolicy/abnormality/reward schema와 live RON, RunState,
  SkillFragmentInventory의 research-only fields/methods, combat/node-flow branches, snapshot/result DTO,
  checkpoint/active-run version, server mapping/fixture와 Unity reader/tests를 같은 strict cutover에서
  제거해야 한다. Encounter fragment ledger(`GRB-M-008`)가 먼저 또는 coimplementation되어야 old unique
  fragment source를 제거해도 획득 기능이 사라지지 않는다.

### `GRB-M-008` — Encounter-grade acquisition and repeat ledger

- **Current validity:** confirmed.
- **Current schema/data:** `data/pve_data.rs:212-244`의 strict `PveEncounter`는 class,
  primary abnormality, ordinary `reward_uuids`, optional old `suppression_research`와 combat setup만
  가진다. `fragment_id`, `repeat_reward` 또는 reward-source identity가 없다. Production PVE에는
  Normal 3, Elite 13, NormalBoss 12, FinalBoss 5 definitions가 있으며 target fields는 0건이다.
- **Current result path:** ordinary victory applies encounter `reward_uuids` and legacy abnormality
  research. Monolith Stage 3만 `monolith.rs`의 `unique_fragment_granted`를 갱신한 뒤
  `world/combat.rs:2952-2986`에서 abnormality의 old single response-complete fragment를 직접 grant하는
  partial special case다. NormalBoss/FinalBoss direct grant와 repeat reward transaction은 없다.
- **State/persistence gap:** target Elite third-victory/mastery, source completion/repeat and boss grant
  state를 하나의 typed run-local ledger로 보존하는 authority가 없다. Existing V3 checkpoint가 old
  research/monolith special state를 저장하므로 ledger introduction과 legacy removal은 save shape를
  함께 바꾼다.
- **DTO/consumer gap:** existing `skill_fragment_diffs`는 최종 stack 변화만 표현하고 source identity,
  first-vs-repeat, trigger/mastery를 표현하지 않는다. Accessible Unity source에는 이 target result의
  dedicated reader/presentation이 없다.
- **Approved target acceptance:** eligible victory commit은 exact authored source를 기준으로 first grant
  또는 authored repeat reward 중 정확히 하나를 node/reward/XP/condition mutation과 원자 commit한다.
  Defeat/retreat/failed commit은 ledger나 inventory를 바꾸지 않는다. Dismantle 뒤에도 first-grant ledger를
  reset하지 않는다. Mastered Elite 소탕은 count 0이면 one-copy reacquisition, count 1 이상이면 repeat
  reward를 상호 배타적으로 적용하며 분쇄→재획득 반복을 허용한다. 재획득은 identity별 upgrade/awakening
  progress를 보존한다. Checkpoint load는 ledger/inventory/reward를 함께 rewind한다.
- **Implementation boundary:** typed acquisition definition과 source identity, PVE/live validation,
  run/checkpoint ledger, one victory-result transaction shared by Elite/NormalBoss/FinalBoss/BossOmen,
  existing `GrantExecutionResult` integration, result/snapshot projection와 consumer presentation/test가
  필요하다. Exact live IDs/quantities가 작성되기 전 framework-only placeholder publication을 완료로
  보지 않는다.

### `GRB-M-009` — Ordinary Reward source exclusion

- **Current validity:** confirmed.
- **Reachable live path:** `map/node_definitions.ron:43-46`의 `reward_treasure`가 production
  `default_treasures` pool을 사용한다. 이 pool은 direct fragment reward 두 개와 research reward 세
  family를 포함한다. `world/reward.rs`는 selected Reward option을 staged economy grant에 넘기고
  `reward.rs:369-424`는 `GrantSkillFragment`와 `GrantSkillFragmentResearch`를 source category 검사 없이
  실행한다.
- **Validation/test:** generic reward reference validation은 ID 존재를 확인하지만 ordinary Reward-node
  pool의 fragment source eligibility를 reject하지 않는다. `live_rewards_can_grant_skill_fragments_from_ron`
  과 `reward_session_reports_skill_fragment_and_research_diffs`가 current allowance를 보호한다.
- **Approved target acceptance:** ordinary Reward node/pool publication은 direct fragment와 research
  effect를 포함하면 load-time fail-closed한다. Existing live pool에서는 해당 entries를 제거하고
  encounter-grade 또는 explicit Standard script만 fragment를 지급한다.
- **Implementation boundary:** live pool membership correction만으로 현재 data는 고칠 수 있지만
  재도입 방지를 위해 node/pool source-aware validation과 invalid-data test가 필요하다. Generic
  `GrantSkillFragment` primitive는 encounter/Standard source에서 재사용하므로 제거 대상이 아니다.
  Research effect variant 자체는 `GRB-M-002` cutover에서 제거한다.

### Unit B evidence limit

Static trace가 current reachability, strict save shape와 consumer contract를 직접 닫아 probe는 실행하지
않았다. Exact future fragment IDs/repeat quantities는 아직 authored되지 않았으므로 availability가 아니라
implementation content dependency로 남긴다. Unity runner는 unavailable이다.

## Bounded Unit C — Elite Pool, Monolith And Sweep

### Shared target authority

룰북은 Elite를 미숙련 자연 출현 → 3승 숙련 → 숙련 deterministic repeat cycle로 바꾸되, 각 Floor의
Elite slot은 distinct unmastered를 먼저 채우고 부족분만 mastered pool로 보충한다. Monolith Stage 2/3
stat 배율 초기값은 150%/200%이며 개발 중 RON tuning으로 조절할 수 있고 exhausted 재현 실패는 영구
실패로 만든다. 숙련 Elite 소탕은 실제 전투를 만들지 않되 같은 base reward, 재획득 또는 repeat reward와
생존 roster XP를 지급하고
아이템 duration을 소비하는 synthetic combat-result transaction이다. 이 동작 방향은 승인됐고 exact
encounter fragment/repeat content만 `GRB-M-008` authoring 입력에 의존한다.

### `GRB-M-005` — Elite natural/mastered pool lifecycle

- **Current validity:** confirmed.
- **Current authority:** `monolith.rs:20-63`의 abnormality별 `MonolithProgress`는 natural outcome,
  victories, purchased/use와 old unique-fragment flag를 함께 가진다. `is_naturally_excluded`는 승리와
  실패 모두 true로 취급한다.
- **Reachable selection:** `world/map_encounters.rs:52-75`는 Endless Elite 후보에서 natural outcome이
  한 번이라도 resolved된 abnormality를 영구 제외하고, 비면 authored ordinary combat으로 치환한다.
  별도의 unmastered/mastered pool, mastery threshold, deterministic cycle cursor가 없다.
  이후 selection도 legacy research completion weight와 recent-floor soft exclusion을 사용한다
  (`map_encounters.rs:215-297`, live `run/policy.ron:25-30`).
- **Existing tests:** `natural_resolution_hard_excludes_and_only_victory_unlocks_purchase`,
  `endless_resolved_elite_pool_falls_back_to_authored_ordinary_combat`와 repeat-weighting tests가 current
  반대 계약을 보호한다.
- **Approved target acceptance:** 자연 Elite는 3번째 eligible victory까지 unmastered pool에 남고,
  Stage 3 mastery 뒤에만 mastered repeat pool에 진입한다. 현재 Floor의 Elite slots는 distinct unmastered
  candidates를 우선 배정하고, 수가 부족할 때만 나머지를 mastered cycle의 stable authored identity와
  run-local cursor로 결정론적으로 채운다. 양쪽을 합쳐도 unique candidate가 부족한 slot은 Normal combat으로
  downgrade한다. 실패/retreat는 victory나 cursor를 올리지 않는다.
- **Implementation boundary:** Monolith progression에서 natural outcome과 mastery/reward authority를
  분리하거나 replacement encounter ledger에 통합하고, map assignment의 hard exclusion/research weighting을
  unmastered-first/mastered-cycle selector로 교체해야 한다. Ledger/cursor, strict checkpoint validation,
  pool-empty fallback, deterministic reload tests와 live encounter identity 검증이 필요하다. 이 finding은
  `GRB-M-008`의 third-victory fragment grant와 같은 commit boundary를 공유한다.

### `GRB-M-006` — Monolith Stage 2/3 stat multipliers

- **Current validity:** confirmed.
- **Current live data:** `game_resources/data/run/policy.ron:41-44`는 Stage 2/3 stat과 reward multiplier를
  각각 100/125로 둔다. `run_policy_data.rs:860-880`은 Monolith role일 때 stat multiplier를 target power,
  health, attack, defense 등 combat tuning에 실제 적용한다.
- **Validation:** loader는 값이 양수이고 Stage 3이 Stage 2보다 큰지만 검증한다
  (`run_policy_data.rs:542-592`). 승인된 150/200 initial stat 값이나 reward 정책과의 독립성을 고정하지
  않는다.
- **Approved target acceptance:** Stage 2와 3 combat stat multiplier의 initial live 값은 각각 150%와
  200%이고 target power 및 모든 적용 stat이 같은 validated multiplier에서 파생된다. 두 값은 positive,
  ordered, safe-range invariant 안에서 개발 중 RON tuning으로 조절할 수 있다. 룰북은 reward multiplier를
  이번 finding의 변경 대상으로 승인하지 않았으므로 별도 authoring 없이 현 값을 유지한다.
- **Implementation boundary:** live two-value correction, invariant validation과 initial baseline/Stage별
  derived tuning tests가 핵심이다. Initial 값을 loader constant로 영구 고정하지 않는다. Wire/save shape
  변화는 없고 current policy application seam을 재사용할 수 있다.

### `GRB-M-007` — Mastered Elite sweep vertical slice

- **Current validity:** confirmed absence. Gameplay 의미의 `Sweep`/`sweep`/`소탕`을 Core, server,
  live data와 접근 가능한 Unity production source 전체에서 검색한 결과 action/command/result/consumer
  hit가 0건이었다. 유일한 `continuous-sweep` 문자열은 battle movement unit-test fixture 이름이다.
- **Reusable current seam:** combat 승리 결과는 `world/combat.rs:635-723`의 staged economy/roster 상태,
  survival XP와 consumable duration decrement를 거쳐 `CombatRewardsGranted`에 node completion을
  감싼다. 그러나 이 함수는 실제 `CombatBattleState`/event log/result stats를 전제로 하므로 그대로
  가짜 battle을 조립하는 것이 target synthetic authority는 아니다. `behavior.rs` ActionKind와
  `PlayerBehavior`, action capability에도 sweep admission이 없다.
- **Approved target acceptance:** mastered Elite와 유효한 sweep item을 명시적으로 선택할 때 전투 상태,
  battle record, bonus objective 또는 employee condition을 만들거나 바꾸지 않는다. 같은 authored base
  reward에 더해 source fragment count가 0이면 한 copy를 재획득하고 1 이상이면 `GRB-M-008` repeat reward를
  지급한다. 두 branch는 상호 배타적이며 분쇄→재획득 cycle은 제한하지 않는다. 현재 살아 있는 roster
  전체의 post-battle XP, item duration 감소, mastered cycle advancement와 node completion을 하나의
  staged/atomic result로 commit한다.
- **Implementation boundary:** typed command/action capability와 preview eligibility/disabled reason,
  sweep item identity+duration validation, battle-independent synthetic result planner, encounter reward ledger와
  mastered cursor mutation, DTO/server mapping, Unity command/read/presentation과 cross-layer fixtures가 필요하다.
  Existing economy grant/XP/result diff primitives는 재사용하되 battle event-log/condition/discovery commit은
  호출하지 않는 별도 domain operation이어야 한다.

### `GRB-M-014` — Permanent exhausted remanifestation failure

- **Current validity:** confirmed.
- **Current behavior:** conversion은 `monolith.rs:177-204`에서 use를 먼저 소비하고 active node를 만든다.
  battle attempt exhaustion은 `world/combat.rs:2893-2911`에서
  `resolve_remanifestation_failure`를 호출한다. 이 함수는 `monolith.rs:270-292`에서 active node만
  제거하고 victories/stage를 유지하므로 uses가 남으면 같은 `next_stage`를 새 노드로 다시 만들 수 있다.
  `clear_floor_local_remanifestations`도 같은 재사용 가능 상태를 만든다.
- **Persistence/validation:** `MonolithProgress`에 failed stage/terminal failure가 없고 checkpoint validation도
  이를 표현하지 않는다. Live `remanifestation_max_attempts: 2`는 한 battle의 retry count를 제한할 뿐
  같은 stage의 future conversion을 막지 않는다.
- **Approved target acceptance:** remanifestation combat이 두 attempt를 모두 소진해 실패하면 해당
  abnormality/stage가 run 동안 terminal failed가 되고 새 node conversion, fragment, mastery와 residual을
  만들 수 없다. Checkpoint load는 실패 직전/직후 transaction을 일관되게 되감는다.
- **Implementation boundary:** explicit failed-stage terminal state와 transition, `is_usable`/conversion gate,
  strict checkpoint validation/snapshot projection, failure decision/load-checkpoint 및 floor-transition tests가
  필요하다. `GRB-M-005`/`008`의 replacement encounter lifecycle 안에서 coimplement해야 competing flags를
  만들지 않는다.

### Unit C evidence limit

Static trace와 exact production-wide absence search로 current boundary를 닫았다. Sweep은 없는 기능이므로
runtime probe가 추가 current behavior를 밝히지 않는다. Unity runner는 unavailable이고 fresh consumer test
통과를 주장하지 않는다.

## Bounded Unit D — NormalBoss And FinalBoss/BossOmen Cycles

### Shared target authority

룰북은 NormalBoss와 일반 FinalBoss chain을 각각 authored identity cycle로 순환시키고, BossOmen
eligibility를 chain별 optional fragment/awakening condition으로 평가한다. Provisional 첫 source 실패는
cycle 뒤로 requeue하고 active source 실패는 같은 step을 유지한다. 이 gameplay 의미는 승인됐으며 exact
minimum Floor, fragment/awakening requirement와 step content는 chain RON authoring 입력이다.

### `GRB-M-010` — NormalBoss authored cycle

- **Current validity:** confirmed absence.
- **Current selection:** `world/map_encounters.rs:132-173`은 node/floor seed로 compatible encounter에서
  modulo/legacy weighted selection한다. Boss category는 Standard final 특례가 아니면 모두
  `PveEncounterClass::NormalBoss`다 (`map_encounters.rs:331-345`). Production에는 12개 live NormalBoss가
  있지만 cycle membership/cursor/resolved assignment state가 없다. Current content roster source는
  Moonlit Wail을 제외하고 Queen/Mountain을 NormalBoss로 내리므로 target membership candidate는 13개다.
- **State/persistence:** `RunState`/V3 checkpoint에 NormalBoss assigned/visited/resolved 또는 current cycle
  ledger가 없다. 승리·attempt-exhausted 패배·미방문 Gate transition을 구별해 pool에 반영하는 seam도 없다.
- **Approved target acceptance:** 한 cycle에서 모든 authored NormalBoss identity를 한 번씩 deterministic
  배정하고, 승리는 cycle에서 소진하며 first/repeat reward를 원자 지급한다. Exhausted 패배도 현재
  cycle만 소진하고, 미방문 assignment는 pool로 반환한다. Pool 소진 뒤에만 전부 refill한다.
- **Implementation boundary:** stable source identity가 있는 NormalBoss catalog membership, cycle
  remaining/assigned ledger와 deterministic selector, node completion outcome/Gate return transition,
  checkpoint validation과 reload/retry tests가 필요하다. Fragment 지급은 `GRB-M-008` transaction과
  coimplement한다.

### `GRB-M-011` — Chain-specific BossOmen eligibility

- **Current validity:** confirmed.
- **Current schema:** `data/boss_omen_data.rs:50-73`의 strict chain은 id, boss abnormality, min Floor,
  required steps, step source와 boss encounter만 갖는다. Optional required fragment, awakening requirement와
  chain dependency field가 없다. Live data에는 `white_night` chain 하나만 있다.
- **Current gate:** `boss_omen.rs:182-195,295-299`는 inventory에 **어떤** awakened fragment progress라도
  있으면 chain 후보를 연다. `select_provisional_chain`은 min Floor와 skipped 목록만 filter하며 chain별
  trigger를 보지 않는다. Abnormality의 `omen_chain_id`는 boss↔chain reference validation만 제공한다.
- **Consumer gap:** Core node-confirm DTO는 placed omen의 chain/step/source/presentation을 투영하지만
  eligibility condition/disabled reason은 투영하지 않는다. 접근 가능한 Unity production source에는
  이를 typed 해석하는 BossOmen reader가 확인되지 않았다.
- **Approved target acceptance:** 새 Floor 생성 시 각 chain RON의 optional required fragment와 해당
  fragment awakening flag만 평가한다. 잠긴 미완료 chain은 cycle에 남고, 남은 chain이 모두 잠기면 omen을
  생성하지 않는다. Unknown/cyclic/unreachable dependency는 load-time fail-closed한다.
- **Implementation boundary:** chain condition schema, fragment catalog cross-validation,
  dependency DAG/reachability validation, deterministic eligible subset selection, checkpoint-consistent cycle과
  Core/Unity presentation test가 필요하다. Closed PD005에 따라 current one-step WhiteNight는 유일한 live
  development bootstrap이며 이를 최종 WhiteNight content로 승격하지 않는다. 상세 live chains/conditions는
  별도 future Goal의 authoring 범위다.

### `GRB-M-012` — FinalBoss chain cycle and victory resolution

- **Current validity:** confirmed.
- **Current state/selection:** `BossOmenRunState`는 provisional, active, one placed source, forced boss와
  skipped provisional IDs만 저장한다 (`boss_omen.rs:20-68`). Completed chain set/current cycle cursor가 없다.
  후보가 비면 skipped IDs를 무시하고 모든 min-Floor chain을 즉시 다시 후보로 쓴다
  (`boss_omen.rs:301-329`).
- **Current victory:** required step 수가 차면 authored capstone을 force하고, FinalBoss 승리는 exit gate를
  공개한 뒤 `clear_after_forced_boss_victory`로 active/forced state를 지운다. Chain completion, first/repeat
  reward source와 cycle exhaustion은 기록하지 않는다. Live one-chain data에서는 같은 chain이 다음
  eligible Floor에 다시 나올 수 있지만 이것은 validated all-chain cycle authority가 아니다.
- **Persistence/profile:** BossOmen state는 RunState/V3 checkpoint에 포함되나 target completion ledger는
  없다. 모든 live FinalBoss를 한 run에서 한 번씩 이겼는지 계산하는 global profile input도 없어
  `GRB-M-004` record를 안전하게 commit할 수 없다.
- **Approved target acceptance:** eligible uncompleted chain만 선택하고 모든 authored 일반 chain을 한 번씩
  승리한 뒤 cycle을 refill한다. FinalBoss victory가 authored base+first/repeat reward, chain completion,
  exit-gate reveal과 permanent all-boss record candidate를 원자 계획/commit하되 RunComplete/Floor advance를
  만들지 않는다. Hidden bosses는 일반 cycle에서 제외한다.
- **Implementation boundary:** strict chain membership/completion/cycle ledger, selector/refill, FinalBoss
  victory plan과 encounter reward ledger, checkpoint validation, global profile event/output와 capstone
  gate tests가 필요하다. Global profile durable publish는 이 Core commit 이후 host acknowledgement까지
  `GRB-M-004`의 별도 lifetime boundary로 이어진다.

### `GRB-M-013` — BossOmen source failure semantics

- **Current validity:** confirmed.
- **Current generic completion:** Event check 실패는 failure scene으로 진행한 뒤 final advance가 ordinary
  `NodeCompleted`를 만든다 (`world/event_node.rs:308-387`; test
  `event_check_failure_is_persisted_gameplay_state_and_final_advance_completes`). Node completion commit은
  success outcome을 받지 않고 `boss_omen.consume_completed_source(node_id)`를 호출한다
  (`world/node_flow.rs:1093-1130`). 따라서 omen Event 실패도 step 성공으로 소비될 수 있다.
- **Current combat failure:** retry 가능한 defeat는 NodeConfirm으로 돌아가지만 attempts exhausted defeat는
  일반 node completion을 plan/commit한다 (`world/combat.rs:2771-2853`). 같은 outcome-blind source consume가
  provisional/active combat omen step을 진행한다. Forced FinalBoss만 `is_final_boss_node` 특례로 즉시 run
  failure 후보가 되며 이 부분은 target과 일치한다.
- **Current state limitation:** skipped list는 Floor gate에서 confirm하지 않은 provisional source만 rotate한다.
  Failed provisional/active source를 명시적으로 requeue/retain하는 transition과 reason은 없다.
- **Approved target acceptance:** first provisional Event-started combat failure 또는 Combat source exhausted
  failure는 step/chain completion 없이 candidate order 뒤로 requeue한다. Active chain failure는 active와
  same step을 유지해 다음 Floor에 다시 제안한다. Event node 내부 재시도, fragment/reward/ledger mutation과
  RunComplete를 만들지 않는다.
- **Implementation boundary:** typed source resolution outcome(success/failed/unvisited)과 transaction,
  Event-started battle provenance, provisional queue/active step state, node/Gate completion integration,
  checkpoint validation 및 Event/Combat×provisional/active matrix tests가 필요하다. `GRB-M-012` cycle과
  분리 구현하면 competing queue state가 되므로 같은 run-state cutover에 묶는다.

### Unit D evidence limit

Current Core/live schema, transaction branches와 existing assertions로 경계를 닫았다. Fresh Unity runner는
unavailable이고 BossOmen typed consumer absence는 접근 가능한 production source 검색 범위의 결론이다.

## Bounded Unit E — Skill Fragment Placeholder Conversion

### `GRB-M-015` — 29 temporary BasicAttackModifier fragments

- **Current validity:** confirmed. Production `game_resources/data/skill_fragments/base.ron`에는
  `BasicAttackModifier` 29개와 `ActiveSkill` 2개가 있다. 29개는 concept starter 1개와 abnormality-derived
  placeholder 28개이며 data comment/description도 dedicated skill 전 임시 상태를 명시한다.
- **Existing runtime foundation:** strict `SkillFragmentEffectDef`는 BasicAttackModifier와 ActiveSkill을
  구분하고, ActiveSkill은 imitation skill, upgrade-level variants와 optional awakened skill을 참조한다
  (`data/skill_fragment_data.rs:88-102`). Loader는 skill existence와 cast/area contract를 교차 검증한다
  (`data/validation.rs:460-520`).
- **Equip/combat path:** non-active fragment는 active slot equip을 거부하고 baseline modifier로만 적용한다.
  ActiveSkill은 weapon compatibility와 one-active-slot policy를 거쳐 upgrade/awakening에 따른 skill ID를
  직원 combat profile의 manual skill로 설정한다 (`skill_fragment.rs:780-908`). Core battle targeting,
  activation/range preview와 Unity active-skill transport/presentation seam은 이미 live 2개를 지원한다.
- **Current-wave target disposition:** `future_or_dormant`. 룰북은 dedicated player-selected active skill을
  formal 장기 정책으로 두지만 “이 correction에서는 해당 live data를 변환하지 않는다”고 명시한다.
  따라서 이번 policy implementation wave의 acceptance는 29개를 변환하는 것이 아니라 current valid
  placeholder behavior를 손상시키지 않고 별도 future content Master 경계를 보존하는 것이다.
- **Future campaign acceptance:** 별도 campaign이 승인되면 activated batch의 placeholder가 dedicated
  player-selected active skill로 교체되고 fragment별 imitation/upgrade/awakening behavior, compatibility,
  visuals/presentation와 deterministic battle assertions이 존재해야 한다. 단순 effect tag 전환, 공용
  no-op skill 또는 여러 ID를 같은 generic buff에 연결하는 방식은 future campaign 완료가 아니다.
- **Missing content authority:** 룰북은 dedicated active skill 원칙만 확정했으며 29개의 개별 target,
  range, timing, cost, effect, upgrade/awakening, weapon compatibility, visual/audio와 balance contract를
  제공하지 않는다. 이 correction에서 live data를 변환하지 않는다고 명시한다. 따라서 현재 evidence로
  29개 final implementation patch를 안전하게 산출할 수 없다.
- **Implementation boundary:** runtime framework 증설보다는 29개 skill design+RON publication,
  abnormality/player imitation 분리 검증, per-skill battle tests, Unity presentation/assets와 catalog QA가
  본체다. 기존 framework가 표현하지 못하는 개별 mechanic이 승인된 design에서 발견될 때에만 typed
  runtime/DTO extension을 추가한다. 29개를 한 atomic code cutover로 묶기보다 approved design batches를
  각각 production-complete vertical slice로 내고 마지막에 placeholder count zero를 검증하는 content
  Master/캠페인이 적합하다.
- **Readiness disposition:** `future_or_dormant`. Runtime foundation은 확인됐고 이번 implementation wave에서
  제외한다는 범위도 canonical로 닫혔다. Future campaign difficulty는 content가 확정되기 전 provisional
  3–5 range로만 보존하며 current readiness decision으로 계산하지 않는다.

### Unit E evidence limit

Live count, strict schema, Core runtime과 접근 가능한 Unity consumer를 static inspection했다. Content design이
없으므로 구현 가능하다고 추론하지 않으며 Unity runner는 unavailable이다.
