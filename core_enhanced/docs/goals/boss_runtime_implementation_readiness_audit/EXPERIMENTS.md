# Boss Runtime Implementation Readiness Audit — Experiments

Goal: `boss_runtime_implementation_readiness_audit`
Execution Mode: `investigation_only`
Production Mutation Authorized: no

## 2026-08-08 — Goal initialization

### Hypothesis

사용자가 승인한 `BP-01`~`BP-34`, 감사 대상 저장소, 문제와 완료 조건이 충분히 명확하므로 production
조사 전에 단일 investigation Goal을 안전하게 초기화할 수 있다.

### Performed checks

- `goal_authoring_and_execution_guide.md` 870줄을 처음부터 끝까지 읽었다.
- 위 가이드가 위임한 `engineering_workflow.md` 389줄과 `goal_completion_review_guide.md` 567줄을
  처음부터 끝까지 읽었다.
- Goal 경로가 존재하지 않아 신규 Goal임을 확인했다.
- Simulator/Core/game_server와 Unity repository root 및 start commit을 확인했다.
- 관련 저장소에서 `git status --short --untracked-files=all`을 읽어 기존 dirty/untracked 상태가 큰 사용자
  working tree임을 확인했다.
- Goal 문서 외 production investigation, repository-wide Boss symbol search, formatter와 test runner는
  실행하지 않았다.

### Exact commands

```text
rg --files core_enhanced/docs | rg 'goal_authoring_and_execution_guide\.md$|engineering_workflow\.md$'
wc -l core_enhanced/docs/goal_authoring_and_execution_guide.md core_enhanced/docs/engineering_workflow.md
sed -n '1,300p' core_enhanced/docs/goal_authoring_and_execution_guide.md
sed -n '301,600p' core_enhanced/docs/goal_authoring_and_execution_guide.md
sed -n '601,900p' core_enhanced/docs/goal_authoring_and_execution_guide.md
wc -l core_enhanced/docs/goal_completion_review_guide.md core_enhanced/docs/engineering_workflow.md
sed -n '1,430p' core_enhanced/docs/engineering_workflow.md
sed -n '1,300p' core_enhanced/docs/goal_completion_review_guide.md
sed -n '301,620p' core_enhanced/docs/goal_completion_review_guide.md
ls -ld core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit
git rev-parse HEAD
git rev-parse --show-toplevel
git status --short --untracked-files=all
git status --short --untracked-files=all | wc -l
git status --short --untracked-files=all | sha256sum
git status --short --untracked-files=all -- docs Assets/Scripts/ArkClient Assets/Tests | wc -l
git status --short --untracked-files=all -- docs Assets/Scripts/ArkClient Assets/Tests | sha256sum
mkdir -p core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit
```

각 `git` 명령은 다음 해당 root에서 실행했다.

- Simulator/Core/game_server: `/mnt/f/work/simulator`
- Unity: `/mnt/f/unity projects/ark`

### Changes

- 새 Goal 디렉터리와 필수 문서 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`만 생성했다.
- Production code, live data, tests, canonical, game_server와 Unity 파일은 변경하지 않았다.

### Test result

Test runner는 실행하지 않았다. 따라서 pass/fail/skip 수를 보고하지 않는다. Goal 초기화와 문서 작성은
runtime behavior 검증이 아니며, test 결과를 완료 evidence로 주장하지 않는다.

### Result and decision

- Goal name, 승인 범위, 완료 조건과 대상 경로가 모두 확인됐다.
- 여러 component를 읽지만 하나의 Boss authoring→runtime→replay readiness 결과를 내고 production
  migration을 수행하지 않으므로 단일 Goal로 시작한다.
- Goal Status는 `active`, `Implementation Complete`는 `not_applicable`, execution mode는
  `investigation_only`, production mutation은 `no`다.
- 실제 code/data/test/consumer evidence는 Goal 문서 초기화 뒤 discovery에서 수집한다.

## 2026-08-08 — Initial document verification

### Hypothesis

필수 문서 3개가 Goal 가이드의 metadata와 조사 전용 범위를 갖고, 새 Goal 경로 이외 production/user
dirty 상태를 변경하지 않았다.

### Exact commands

```text
wc -l core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/PLAN.md core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/EXPERIMENTS.md core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/EXPERIMENT_NOTES.md
rg -n '^Status:|^Implementation Complete:|^Goal Type:|^Execution Mode:|^Production Mutation Authorized:|^## (...)$' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/PLAN.md
git status --short --untracked-files=all -- core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit
git status --short --untracked-files=all -- core_enhanced/src core_enhanced/tests core_enhanced/docs game_resources game_server ':(exclude)core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/**' | wc -l
git status --short --untracked-files=all -- core_enhanced/src core_enhanced/tests core_enhanced/docs game_resources game_server ':(exclude)core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/**' | sha256sum
git status --short --untracked-files=all | wc -l
git status --short --untracked-files=all | sha256sum
git status --short --untracked-files=all -- docs Assets/Scripts/ArkClient Assets/Tests | wc -l
git status --short --untracked-files=all -- docs Assets/Scripts/ArkClient Assets/Tests | sha256sum
```

마지막 네 Unity 명령은 `/mnt/f/unity projects/ark`에서 실행했다.

### Actual result

- 문서 line count: `PLAN.md` 448, `EXPERIMENTS.md` 77, `EXPERIMENT_NOTES.md` 156 — verification
  entry를 추가하기 전 runner 출력이다.
- PLAN metadata와 필수 section heading이 모두 검색됐다.
- Goal 경로에는 새 문서 3개만 untracked로 나타났다.
- Goal 경로를 제외한 Simulator production-scoped status는 `211` entries와 초기 SHA-256
  `7358c0145d11bbc1799400d20429174996a1a20ad12461e19ce761a983565cfe`를 유지했다.
- Unity full status는 `2059` entries와 초기 SHA-256
  `55d940dc476ee297b267ed15afffd40570d448289cdccb0f172957bf254c1dcf`를 유지했다.
- Unity supporting scope는 `306` entries와 초기 SHA-256
  `50ab153889caa2437bed4d0518dd67ac50c5b190dde08b7460ff6fcbedaaff14`를 유지했다.

### Test result and decision

Test runner는 실행하지 않았다. 실제 pass/fail/skip 수는 없다. Initial Goal authoring은 검증됐고 Status는
`active`로 유지한다.

## 2026-08-08 — Full game rulebook audit requirement clarification

### Reason

사용자 확인으로 `game_rulebook.md`를 current canonical 목록에 링크하는 것만으로는 전체 재독과 실제
runtime/live data 대조가 완료 조건인지 충분히 명확하지 않음을 확인했다.

### Change

- `PLAN.md`의 baseline/canonical 단계에 current working-tree rulebook을 처음부터 EOF까지 다시 읽는 요구를
  추가했다.
- DefenseRoute, route/objective/wave, movement/blocking, targeting, time/Pause, terminal 및
  event/checkpoint/reconnect를 집중 대조하되 전체 문맥을 생략하지 않도록 명시했다.
- Rulebook과 runtime/live data/test의 일치·drift·target-policy 차이를 별도 기록하도록 했다.
- Completion condition과 evidence matrix에 `BRA-C13` gate를 추가했다.

### Test result and decision

문서 요구사항만 보완했으며 test runner는 실행하지 않았다. 실제 rulebook 재독과 대조 evidence는 Goal
실행 단계에서 기록한다.

## Pending experiment log policy

이후 실제로 수행한 read-only probe와 test만 시간순으로 추가한다. 각 기록에는 hypothesis, exact command,
runner가 보고한 실제 test 수, pass/fail/skip, 실패 원인, artifact와 판단을 기록한다. Source inspection만
수행한 경우 test count를 추측하지 않는다.

## 2026-08-08 — Unity stale baseline and incremental evidence protocol clarification

### Reason

사용자는 Unity 프로젝트가 current Core/game_server보다 stale하다는 사실을 명시하고, 장시간 감사 뒤 대화
compact로 finding이 유실되지 않도록 각 감사 결과를 발견 시점에 문서화할 것을 요구했다.

### Performed checks

- 현재 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, `BOUNDARY_AUDIT.md`의 line count와 내용을 다시
  읽었다.
- `PLAN.md`에서 Unity stale baseline과 즉시 기록 규칙이 기존에 명시되어 있지 않음을 확인했다.
- 기존 `EXPERIMENT_NOTES.md`의 Unity `current canonical` 표현이 새로 확인된 stale baseline과 충돌함을
  확인했다.

### Exact commands

```text
wc -l core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/*.md
git rev-parse HEAD
git status --short
sed -n '1,260p' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/EXPERIMENTS.md
sed -n '1,320p' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/EXPERIMENT_NOTES.md
sed -n '1,420p' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/BOUNDARY_AUDIT.md
rg -n -i 'stale|Unity|그때그때|즉시|발견|기록|작성|discovery|current-tree|current working-tree|접근' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/PLAN.md
```

### Changes and decision

- `PLAN.md`에 Unity를 stale external consumer baseline으로 분류하는 evidence rule과 invariant `BRA-I13`을
  추가했다.
- bounded investigation unit 직후 finding/probe를 Goal artifact에 기록하고 대화-only evidence를 금지하는
  invariant `BRA-I14`, durable protocol과 completion gate `BRA-C14`를 추가했다.
- `EXPERIMENT_NOTES.md`의 기존 Unity 문서 등급 표현을 stale consumer evidence로 정정하고 이 durable
  checkpoint를 남겼다.
- Production code/data/test/canonical, game_server와 Unity artifact는 변경하지 않았다.

### Test result

문서 운영 계약만 수정했다. Test runner는 실행하지 않았으므로 pass/fail/skip 수는 없다.

## 2026-08-08 — Full current game rulebook read

### Hypothesis

Current `game_rulebook.md` 전체를 EOF까지 읽으면 Boss 감사에 직접 관련된 DefenseRoute, authored route,
blocking, attack/skill, Pause/battle time, terminal과 Core/Unity authority 계약뿐 아니라 다른 gameplay 장의
교차 영향을 빠뜨리지 않고 확인할 수 있다.

### Exact commands

```text
wc -l core_enhanced/docs/game_rulebook.md
sed -n '1,340p' core_enhanced/docs/game_rulebook.md
sed -n '341,680p' core_enhanced/docs/game_rulebook.md
sed -n '681,1020p' core_enhanced/docs/game_rulebook.md
```

### Actual result

- Current file은 961줄이며 1~961줄을 순서대로 EOF까지 읽었다.
- `DefenseRoute`가 일반 적과 Boss의 공통 전장/transport/wave 기반이고 authored route, sticky block,
  Hard CC release 경계, Pause battle-time 및 Core-authoritative replay/checkpoint가 current gameplay 계약임을
  재확인했다.
- Rulebook에는 target Boss Phase/Pattern/State/Task runtime이 current behavior로 기재되어 있지 않으므로
  `BP-01`~`BP-34`는 current canonical 설명이 아니라 후속 구현 target이다.
- Rulebook이 외부 Unity 문서를 canonical로 링크하지만 실제 Unity project는 current Core/game_server보다
  stale하다. 이는 gameplay 의미 충돌이 아니라 external consumer 문서 신선도 drift이며, 현재 producer
  계약을 Unity 상태에 맞춰 역추론할 근거가 아니다.

### Test result and decision

Source inspection만 수행했다. Test runner는 실행하지 않았고 pass/fail/skip 수는 없다. Rulebook 전체 재독
gate는 충족했으며 code/live data/test 대조는 각 후속 bounded unit에서 계속한다.

## 2026-08-08 — Current runtime/data/targeting canonical read

### Hypothesis

`core_runtime_contract.md`, `data_loading_contract.md`, `skill_target_contract.md`를 current-tree에서 읽으면
Boss policy가 재사용할 current invariant와 실제 code에서 별도 확인해야 할 canonical claim을 구분할 수 있다.

### Exact commands

```text
wc -l core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md core_enhanced/docs/skill_target_contract.md
sed -n '1,820p' core_enhanced/docs/core_runtime_contract.md
rg -n '^## |^### ' core_enhanced/docs/core_runtime_contract.md
sed -n '285,440p' core_enhanced/docs/core_runtime_contract.md
sed -n '1,340p' core_enhanced/docs/data_loading_contract.md
sed -n '1,420p' core_enhanced/docs/skill_target_contract.md
```

### Actual result

- Runtime canonical은 append-only event seq, battle-time/Pause, failure no-mutation, checkpoint reconcile,
  stale cast/action epoch와 detached delivery 보존을 current invariant로 선언한다.
- Data canonical은 manifest와 21개 gameplay source를 한 candidate DB로 parse/cross-validate한 뒤 startup에서
  한 번 publication하는 fail-closed embedded loader를 선언한다. Hot reload/data-pack API와 failed reload의
  last-known-good generation retention은 current path가 아니다.
- Targeting canonical은 basic/skill별 deterministic candidate ordering과 locked projectile/cast target primitive를
  제공하지만 Boss activation-owned lock, closed lexicographic query list와 path-cost metric을 제공하지 않는다.
- Runtime canonical의 generic failure no-mutation 서술은 world battle advance 실제 code에 대해서는 아직
  증명되지 않았다. 기존 boundary finding의 post-mutation fallible projection/trigger path를 source symbol에서
  다시 확인해야 한다.

### Test result and decision

Source inspection만 수행했으며 runner 결과는 없다. Canonical 문구는 actual code보다 우선하지 않고 다음
bounded source trace의 검증 대상 claim으로 사용한다.

## 2026-08-08 — Live publication and encounter opt-in source trace

### Hypothesis

Current embedded source set과 `PveEncounter` binding을 추적하면 BP-01/BP-28의 opt-in/catalog seam, atomic
publication primitive 및 현재 generated setup validation coverage를 구체적으로 판정할 수 있다.

### Exact commands

```text
rg -n 'LIVE_GAMEPLAY_RON_SOURCE_COUNT|struct GameplayRonSourceSet|struct ParsedGameplayRon|struct GameDataBaseParts|load_live_embedded|validate_generated_combat_preview_contracts|struct PveEncounter|boss_runtime|BossEncounterRuntime|BossActorBehaviorRuntime' core_enhanced/src game_server/src game_resources/data -g '*.rs' -g '*.ron'
nl -ba core_enhanced/src/game/data/mod.rs | sed -n '690,770p'
nl -ba core_enhanced/src/game/data/live_ron.rs | sed -n '100,175p'
nl -ba core_enhanced/src/game/data/live_ron.rs | sed -n '245,330p'
nl -ba core_enhanced/src/game/data/live_ron.rs | sed -n '1140,1268p'
nl -ba core_enhanced/src/game/data/pve_data.rs | sed -n '190,285p'
nl -ba core_enhanced/src/game/data/pve_data.rs | sed -n '450,645p'
nl -ba core_enhanced/src/game/data/validation.rs | sed -n '1155,1235p'
rg -n -C 14 'encounter_class: FinalBoss|encounter_class: NormalBoss' game_resources/data/pve/encounters.ron
```

### Actual result

- `GameplayRonSourceSet`은 manifest 외 22개 gameplay source field를 수동으로 열거하고
  `LIVE_GAMEPLAY_RON_SOURCE_COUNT == 22` test로 개수만 고정한다.
- `load_live_embedded`는 parse→domain/cross-reference validation→candidate DB construction→generated
  preview validation이 모두 성공한 뒤 `Arc<GameDataBase>`를 반환한다. 실패 candidate의 partial publish는 없다.
- Loader는 compile-time `include_str!` 단일 startup 경로이며 hot reload, generation registry와 failed update
  뒤 last-known-good publication owner는 없다.
- `PveEncounter`는 `deny_unknown_fields`이고 class/primary abnormality/wave를 검증하지만
  `boss_runtime_id`나 typed actor behavior binding이 없다. Repository-wide search에서도 target Boss runtime
  type/symbol은 발견되지 않았다.
- Live FinalBoss/NormalBoss entries는 ordinary `PveEncounter`와 manual abnormality wave이며 FinalBoss 일부는
  주석상 placeholder다.
- Generated preview publication gate는 모든 encounter에 대해 floor `0`, category `Combat`, seed
  `[0,1,17,41,99]`의 baseline preview/threat warning만 검사한다. Manifested difficulty role 전체와 eventual
  typed scenario/Boss binding feasibility를 검증하지 않는다.

### Test result and decision

Source inspection만 수행했다. Existing loader는 strict candidate publication primitive로 재사용 가능하지만,
Boss catalog source/DB field/cross-reference와 encounter binding은 foundation과 함께 추가해야 한다. Live
Boss opt-in 전에 generated setup feasibility coverage를 확대하는 prerequisite가 필요하다.

## 2026-08-08 — Preview to scenario and runtime spawn trace

### Hypothesis

Combat preview에서 `BattleScenario`와 `RuntimeUnit`까지 실제 변환을 읽으면 authored route의 current owner와
Boss encounter/actor binding을 삽입할 최소 typed seam을 확정할 수 있다.

### Exact commands

```text
rg -n 'BattleScenario|ScenarioUnitSpawn|ScenarioSpawnGroup|EnemyMovementPlan|PathAlongCells|enemy_movement_plan|build_battle_scenario_from_preview' core_enhanced/src/game/combat_setup core_enhanced/src/game/battle/scenario.rs core_enhanced/src/game/events/combat.rs
nl -ba core_enhanced/src/game/events/combat.rs | sed -n '70,315p'
nl -ba core_enhanced/src/game/battle/scenario.rs | sed -n '70,255p'
nl -ba core_enhanced/src/game/battle/scenario.rs | sed -n '243,420p'
nl -ba core_enhanced/src/game/combat_setup/enemy_spawns.rs | sed -n '1,145p'
nl -ba core_enhanced/src/game/combat_setup/mission_policy.rs | sed -n '1,215p'
nl -ba core_enhanced/src/game/battle/core/mod.rs | sed -n '115,215p'
nl -ba core_enhanced/src/game/battle/core/build.rs | sed -n '1,300p'
nl -ba core_enhanced/src/game/battle/core/sim.rs | sed -n '1685,1765p'
nl -ba core_enhanced/src/game/events/combat.rs | sed -n '900,1180p'
```

### Actual result

- `CombatExecutor::build_battle_scenario_from_preview`가 encounter를 다시 lookup하고 preview identity를
  검증한 뒤 mission plan, defense object, enemy group/event를 조립한다.
- `enemy_spawn_groups_from_preview`는 preview wave의 `route_id`를 exact route cells로 resolve하여
  `ScenarioSpawnGroup.enemy_movement_plan` 하나로 저장한다. Group의 모든 enemy spawn이 같은 plan을 받는다.
- `ScenarioUnitSpawn`에는 behavior definition/argument binding이 없고 `BattleScenario`에는 encounter runtime
  binding이 없다. Boss actor별 immutable resolved binding을 넣을 current typed slot이 없다.
- `start_battle_execution_with_setup`이 scenario structural validation 후 runtime을 reset하고 group spawn을
  실행한다. `spawn_scenario_group`은 group movement plan을 각 `RuntimeUnit.enemy_movement_plan`으로 clone한다.
- Existing `events/combat.rs` tests는 preview wave timing, authored objective override와 preview route cells가
  scenario tactical/group plan으로 동일하게 전달되는 실제 setup function을 assertion한다. Non-opt-in route
  regression의 retain/strengthen 기반이다.

### Test result and decision

Source/test body inspection만 수행했고 runner는 실행하지 않았다. 최소 seam은 publication에서 resolve한
Boss encounter definition을 `BattleScenario`에, actor behavior definition+argument frame을
`ScenarioUnitSpawn` 또는 동등한 per-spawn typed plan에 넣고 `BattleCore` start에서 immutable runtime으로
admit하는 것이다. Ordinary group route field와 fallback은 그대로 유지한다.

## 2026-08-08 — Movement, route progress, blocking and basic-target authority trace

### Hypothesis

Current planner/path/block/target source를 함께 읽으면 Boss intent를 기존 leaf execution 앞에 넣을 seam과
`RejoinForwardRoute`, activation target lock 및 closed Task result에 필요한 새 foundation을 구분할 수 있다.

### Exact commands

```text
nl -ba core_enhanced/src/game/battle/core/movement/planner.rs | sed -n '1,145p'
nl -ba core_enhanced/src/game/battle/core/movement/path.rs | sed -n '1,300p'
nl -ba core_enhanced/src/game/battle/core/movement/types.rs | sed -n '1,330p'
nl -ba core_enhanced/src/game/battle/core/movement/engine.rs | sed -n '60,135p'
nl -ba core_enhanced/src/game/battle/core/movement/engine.rs | sed -n '235,365p'
nl -ba core_enhanced/src/game/battle/core/movement/engine.rs | sed -n '515,715p'
nl -ba core_enhanced/src/game/battle/core/movement/lifecycle.rs | sed -n '1,120p'
nl -ba core_enhanced/src/game/battle/core/movement/blocking.rs | sed -n '125,315p'
nl -ba core_enhanced/src/game/battle/core/targeting.rs | sed -n '1,285p'
nl -ba core_enhanced/src/game/battle/core/targeting.rs | sed -n '535,720p'
nl -ba core_enhanced/src/game/battle/core/basic_attack.rs | sed -n '1,175p'
nl -ba core_enhanced/src/game/battle/core/movement/planner.rs | sed -n '470,565p'
```

### Actual result

- Opponent planner 순서는 block/Hard-CC movement gate 뒤 ranged reposition, airborne/basic target, authored
  route goal이다. 별도 Boss intent owner나 route fallback보다 앞선 explicit opt-in branch가 없다.
- Missing unit plan은 `scenario.tactical_plan.enemy_plan`으로 fallback한다. 이 분기가 non-opt-in parity owner다.
- Route progress는 현재 world position을 모든 route segment에 투영한 nearest progress다. 거리 동률은 더 큰
  progress를 고르지만 actor-owned monotonic cursor가 없어 route 이탈 뒤 더 가까운 과거 segment를 선택할 수
  있다.
- Movement는 `MoveToPoint` direct steering과 collision correction이다. Static obstacle stop reason은 internal
  `MovementOutput`에 존재하지만 apply 단계가 segment만 제거하고 reason을 상위 result/event로 전달하지 않는다.
- `RuntimeUnit.current_target`은 block/basic attack target preference이며 in-range/usefulness에 따라 유지 또는
  교체된다. Pattern activation target lock, decision-point re-evaluation과 target-invalidated result owner가 아니다.
- Current targeting comparator는 closed weapon profiles와 stable spawn order/unit ID fallback을 제공한다.
  Arbitrary Boss filter + ordered key list, path cost/reachability metric은 없다.
- Existing tests는 route waypoint progression, empty-route no-chase, block priority/stickiness, persisted in-range
  target와 deterministic profile ordering을 실제 methods로 보호한다. Monotonic rejoin cursor와 typed movement
  result는 보호하지 않는다.

### Test result and decision

Source/test body inspection만 수행했다. Existing movement/blocking/attack executor는 유지한다. Boss foundation은
planner 앞의 explicit actor intent, actor-owned forward route cursor, deterministic bounded grid/path query,
activation target lock과 engine-output→closed TaskResult adapter를 추가해야 한다. Current
`RuntimeUnit.current_target`를 Boss blackboard로 재사용하지 않는다.

## 2026-08-08 — Same-time scheduler and max-time terminal trace

### Hypothesis

Current event ordering과 두 advance API의 max-time 처리를 함께 읽으면 BP-16/BP-31/BP-32/BP-34를 얹기
전에 닫아야 할 scheduler/terminal prerequisite가 구체적인 source seam으로 확인된다.

### Exact commands

```text
rg -n 'enum BattleEvent|impl Ord for BattleEvent|fn priority|step_battle_execution|finalize_battle_execution|same_timestamp_attack_resolve' core_enhanced/src/game/battle
nl -ba core_enhanced/src/game/battle/enums.rs | sed -n '1,245p'
nl -ba core_enhanced/src/game/battle/enums.rs | sed -n '215,475p'
nl -ba core_enhanced/src/game/battle/enums.rs | sed -n '475,565p'
nl -ba core_enhanced/src/game/battle/core/sim.rs | sed -n '50,95p'
nl -ba core_enhanced/src/game/battle/core/sim.rs | sed -n '1260,1320p'
nl -ba core_enhanced/src/game/battle/core/sim.rs | sed -n '1580,1645p'
nl -ba core_enhanced/src/game/battle/core/sim.rs | sed -n '1760,1910p'
nl -ba core_enhanced/src/game/battle/core/mod.rs | sed -n '2280,2390p'
```

### Actual result

- `BattleEvent::Ord`는 timestamp와 numeric priority 뒤 variant별 field를 일부 비교하지만 catch-all
  `_ => Ordering::Equal`을 사용한다. Priority 8을 공유하는 `AutoCastEnd`와 `ManualCastEnd` 같은 서로 다른
  event kind 및 별도 `ManualCastEnd` 값들이 distinct event인데도 `Equal`이 될 수 있다. 원인/cause도 모든
  branch의 final tie-break가 아니다. 따라서 current order는 total order가 아니다.
- Same-time drain은 한 timestamp bucket을 처리한 뒤 그 처리 중 새로 enqueue된 동일 timestamp event가 있으면
  다시 bucket을 만든다. BP-34의 deterministic microstep 기반으로 재사용 가능하지만 per-timestamp microstep
  counter/limit 또는 zero-time livelock guard는 없다.
- `step_battle_execution`은 다음 event가 `max_battle_time_ms`를 넘으면 Draw로 finalize한다. 반면
  `step_battle_execution_until`은 target을 max-time으로 clamp한 뒤 다음 event가 더 미래이면 time만 target으로
  옮기고 winner가 없을 때 `Running`을 반환한다. 동일 execution이 호출 API에 따라 max-time terminal 의미가
  달라질 수 있다.
- `finalize_battle_execution`의 `finalized` guard는 한 execution 안에서 result/event 중복 생성을 막는
  reusable primitive다.
- Existing `same_timestamp_attack_resolve_is_drained_before_battle_end` test는 attack resolve와 battle end 한
  조합만 검증한다. 모든 variant의 distinct ordering, insertion permutation 및 microstep overflow를 보호하지
  않는다.

### Test result and decision

Source/test body inspection만 수행했고 runner는 실행하지 않았다. Boss transition event를 추가하기 전에
`BattleEvent`에 stable total tie-break를 주고, 동일 timestamp microstep budget의 deterministic failure를
정의하며, 모든 advance entry point가 같은 max-time terminal helper를 사용하도록 해야 한다. 이 세 항목은
Boss behavior 구현과 섞지 않는 `prerequisite_before_boss`다.

## 2026-08-08 — World live-advance atomicity trace

### Hypothesis

`advance_active_battle_*`의 mutation과 fallible projection 순서를 읽으면 runtime canonical의 failure
no-mutation 불변식이 live tick에서도 실제 보장되는지 판정할 수 있다.

### Exact commands

```text
rg -n 'advance_active_battle|active_battle_advanced_result|transition_finished|battle_state_revision|drain_live_signals|trigger' core_enhanced/src/game/world -g '*.rs'
nl -ba core_enhanced/src/game/world/combat.rs | sed -n '45,120p'
nl -ba core_enhanced/src/game/world/combat.rs | sed -n '1170,1390p'
nl -ba core_enhanced/src/game/world/combat.rs | sed -n '1386,1510p'
nl -ba core_enhanced/src/game/world/state.rs | sed -n '560,635p'
nl -ba core_enhanced/src/game/world/state.rs | sed -n '615,985p'
nl -ba core_enhanced/src/game/world/visual_novel_triggers.rs | sed -n '35,90p'
nl -ba core_enhanced/src/game/world/visual_novel_flow.rs | sed -n '185,245p'
rg -n 'InvalidBattleResyncSeq|advance_active_battle_by_after_seq|advance_active_battle_for_server_tick_after_seq' core_enhanced/src game_server/src -g '*.rs'
nl -ba core_enhanced/src/game/world/tests/combat.rs | sed -n '2770,2845p'
nl -ba core_enhanced/src/game/world/tests/combat.rs | sed -n '3540,3620p'
```

### Actual result

- `advance_active_battle_by_internal`은 revision overflow 가능성만 preflight한 뒤 `BattleCore` execution을
  직접 step/finalize한다. 그 다음 별도 `&mut self` 구간에서 fallible VN trigger evaluation을 하고, 다시 active
  session을 빌려 resource refresh, live-signal drain, employee/deployment reconcile, revision commit과 DTO 생성을
  순서대로 수행한다.
- `battle_update_dto_after`는 `after_seq > latest_seq`이면 `InvalidBattleResyncSeq`를 반환한다. 이 검사는
  execution step, trigger 처리, signal/resource/roster reconcile 및 `battle_state_revision` 증가 뒤에 있다.
  따라서 public `advance_active_battle_*_after_seq`에 future cursor가 전달되면 `Err`와 함께 Core time/event/state,
  trigger state 또는 projection revision 중 일부가 이미 바뀔 수 있다.
- `attach_visual_novel_to_battle_result`도 revision commit 뒤 fallible snapshot/capability projection을 수행한다.
  Finished path는 마지막에 active content를 result로 바꾸고 active battle을 제거한 뒤 fallible state transition을
  호출한다. Current flow는 전체 advance를 staging clone에서 검증한 뒤 commit하는 transaction이 아니다.
- `advance_active_battle_to_next_event_bucket`도 같은 post-step trigger/projection 순서를 복제한다.
- Existing future-seq test는 read-only `RequestBattleState` rejection만 확인한다. Revision-overflow test는 playback
  command의 preflight no-mutation만 확인한다. Live advance의 post-step error에서 time/event/resource/revision/
  trigger가 모두 unchanged임을 assertion하는 test는 검색 범위에서 발견되지 않았다.
- `game_server` tick handler가 `advance_active_battle_for_server_tick_after_seq`를 직접 호출하므로 이 seam은
  단순 dead helper가 아니다.

### Test result and decision

Source/test inspection으로 failure-after-mutation 경로가 구체적으로 확인되어 별도 runner는 실행하지 않았다.
Boss BP-31 atomic transition을 world에 통합하기 전에 advance 전체를 staged clone/plan에서 실행해 trigger,
projection, cursor와 terminal transition까지 validate한 뒤 한 번 commit하거나, 동등한 rollback 보장을 제공해야
한다. Future cursor는 가능하면 step 전 preflight하되 그것만으로 뒤쪽의 다른 fallible projection을 모두 닫았다고
간주하지 않는다. Disposition은 `prerequisite_before_boss`다.

## 2026-08-08 — Active runtime versus public checkpoint trace

### Hypothesis

Current active session, run checkpoint와 live checkpoint DTO를 분리해 읽으면 Boss graph state의 durable owner와
public reconnect projection 경계를 save migration을 추측하지 않고 정할 수 있다.

### Exact commands

```text
rg -n 'struct ActiveBattleSession|BattleEntrySnapshot|RunCheckpointPayload|LiveBattleStateCheckpointDto|active_battle_reconnect_projection|live_checkpoint_dto' core_enhanced/src/game -g '*.rs'
nl -ba core_enhanced/src/game/world/state.rs | sed -n '100,370p'
nl -ba core_enhanced/src/game/world/state.rs | sed -n '967,1065p'
nl -ba core_enhanced/src/game/behavior.rs | sed -n '1790,1860p'
nl -ba core_enhanced/src/game/behavior.rs | sed -n '2780,2885p'
nl -ba core_enhanced/src/game/world.rs | sed -n '170,235p'
nl -ba core_enhanced/src/game/world/snapshot.rs | sed -n '560,610p'
nl -ba core_enhanced/src/game/battle/core/mod.rs | sed -n '45,135p'
rg -n 'active battle|checkpoint|reconnect|save migration' core_enhanced/docs/core_runtime_contract.md
```

### Actual result

- `ActiveBattleSession` owns the live `BattleCore` and `BattleExecutionState` in memory. It is neither part of
  `RunCheckpointPayload` nor cloned by `GameCoreState::clone_non_battle_transaction_state`; that clone explicitly
  declines while a battle is active. `BattleEntrySnapshot` captures the pre-battle world and restores with
  `active_battle = None`, so it is retreat rollback, not an active-battle save.
- `core_runtime_contract.md` explicitly says reconnect/resync reads the same in-memory active session and active battle
  is not persisted in run checkpoint/save. Boss v1 therefore needs no run-save schema migration unless scope is
  deliberately expanded to process-restart battle resume.
- `LiveBattleStateCheckpointDto` is a public projection generated from the active runtime at a chosen `at_seq`. It
  carries battle time, playback, tactical focus, field-work state, active units, resource and deployment. Per unit it
  carries ID/side/role/HUD/control/tactical state/range previews/mobility/source/position/world position/stats.
- The public unit checkpoint has no authoritative facing. It also has no Boss public phase/objective or active
  duration-presentation lifecycle state. Internal Pattern/State/Task IDs and activation epoch should remain absent.
- Same-battle setup-loss recovery is read-only setup + current revisioned update/checkpoint from the same active
  session. This is consumer reconstruction, not restoration of Core execution from the DTO.
- `BattleCore` itself is not `Clone`; this also confirms that the world advance atomicity fix cannot silently rely on
  an existing full active-session transaction clone.

### Test result and decision

Source/canonical inspection만 수행했다. Boss authoritative graph state belongs inside `BattleCore`/active session and
must survive ordinary in-memory ticks, Pause and reconnect naturally. The public checkpoint adds only gameplay-
observable durable presentation state required to reconcile a connected consumer—at minimum any continuing public
phase/objective/duration lifecycle and facing if the first slice depends on it. Save/checkpoint migration remains
`not_applicable` for the current no-active-battle-persistence scope; restart resume would be a separate policy change.

## 2026-08-08 — game_server battle transport and cursor trace

### Hypothesis

Core DTO→actor result mapping→WebSocket message→delivery cursor를 읽으면 Boss public fields가 자동 전파되는
위치와 server 동시 수정이 필요한 위치를 정확히 나눌 수 있다.

### Exact commands

```text
rg -n 'BattleAdvanced|BattleState|battle_update|battle_setup|events_delta|checkpoint|battle_event_after_seq|advance_active_battle_for_server_tick_after_seq' game_server/src/game/player_game_actor -g '*.rs'
nl -ba game_server/src/game/player_game_actor/messages.rs | sed -n '90,205p'
nl -ba game_server/src/game/player_game_actor/messages.rs | sed -n '300,570p'
nl -ba game_server/src/game/player_game_actor/state.rs | sed -n '225,370p'
nl -ba game_server/src/game/player_game_actor/state.rs | sed -n '480,535p'
nl -ba game_server/src/game/player_game_actor/handlers.rs | sed -n '50,105p'
nl -ba game_server/src/game/player_game_actor/handlers.rs | sed -n '785,960p'
nl -ba game_server/src/game/player_game_actor/handlers.rs | sed -n '2200,2350p'
rg -n 'to_value\((update|setup)|json.*parity|parity' game_server/src/game/player_game_actor -g '*.rs'
```

### Actual result

- Battle command results are forced onto side-message transport; legacy command payload serialization rejects them.
  Start emits setup then update, tick emits update, authority rejection emits `battle_resync`, and final tick emits the
  final update before the top-level result snapshot. Existing actor tests protect these coarse ordering/cursor rules.
- `BattleResync { update: LiveBattleUpdateDto }` nests the typed Core update intact. In contrast normal
  `BattleUpdate` manually enumerates/destructures every top-level update field, and `BattleSetupSnapshot` manually
  enumerates every setup field into a server-owned enum variant.
- Nested `events_delta`, `checkpoint` and `catalog_refs` are passed as whole typed Core values, so new event variants or
  fields inside those nested DTOs reach serialization after the server compiles. A new top-level Core update/setup
  field can be silently omitted by the manual mapper because field access does not exhaustively destructure the Core
  struct.
- Existing serialization tests assert selected shape values, not exact normalized JSON parity between a Core DTO and
  the corresponding server envelope. Thus they do not catch every silently dropped future top-level field.
- Actor `battle_event_cursor` records `to_seq` when it enqueues a `BattleUpdate`/`BattleResync` to the active socket and
  feeds that cursor into the next Core tick. Attach/reconnect deliberately requests from 0 and emits immutable setup
  plus current update. The actor does not recompute event ordering or checkpoint gameplay.
- The server tick directly calls the after-seq Core advance API; therefore the previously recorded Core future-cursor
  post-mutation failure can stop ticking after partial authoritative mutation.

### Test result and decision

Source/test body inspection만 수행했다. First Boss public cut should prefer existing typed nested event/checkpoint and
catalog-ref surfaces where semantically correct, but schema placement must not be distorted merely to avoid server
work. Any new top-level Core setup/update field requires same-cut server mapper changes and an exact JSON parity test.
Resync and normal update must be tested against the same Boss event/checkpoint generation. Disposition is
`coimplement_with_first_wire_change`, while the Core advance atomicity issue remains a prior prerequisite.

## 2026-08-08 — Stale Unity battle replay, reconcile and cleanup trace

### Hypothesis

Unity를 current producer authority로 보지 않고 실제 store/model/renderer/tests를 읽으면 first Boss slice가 existing
events만으로 표시 가능한 범위와 새 lifecycle consumer가 필요한 범위를 구분할 수 있다.

### Exact commands

```text
rg --files '/mnt/f/unity projects/ark' | rg 'BattleRuntimeState|GameSnapshotStore|Battle.*(Message|Event|Checkpoint|Setup)|Tests?.*Battle'
nl -ba '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/BattleRuntimeState.cs' | sed -n '1,215p'
nl -ba '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/BattleRuntimeState.cs' | sed -n '403,829p'
nl -ba '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameSnapshotStore.cs' | sed -n '80,256p'
nl -ba '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldFieldModel.cs' | sed -n '240,450p'
nl -ba '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldFieldModel.cs' | sed -n '560,720p'
nl -ba '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldFieldModel.cs' | sed -n '750,790p'
nl -ba '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldFieldModel.cs' | sed -n '985,1075p'
nl -ba '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldFieldAssembler.cs' | sed -n '1,250p'
nl -ba '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldFieldAssembler.cs' | sed -n '580,635p'
nl -ba '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldFieldAssembler.cs' | sed -n '1040,1185p'
nl -ba '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldUnitActorView.cs' | sed -n '1,340p'
rg -n 'SpineUnitVisual|UnitVisualActionRequest|UnitVisualFactory' '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient' -g '*.cs' | rg -v '/SpinePresentation/(Runtime|Sandbox)/'
rg -n 'BattleRuntimeState|battle_resync|MovementSegmentStarted|PresentationClock|ReconciledCheckpoint' '/mnt/f/unity projects/ark/Assets/Tests' -g '*.cs'
```

### Actual result

- Baseline status remains explicitly stale relative to current Core/game_server. Findings below describe consumer
  readiness/debt and never override current producer contracts.
- `BattleRuntimeState` validates positive revision, contiguous seq, `checkpoint.at_seq == to_seq`, whole-projection
  eligibility, duplicate/stale revision and resync envelope identity. It stores raw event envelopes, delays a pending
  checkpoint until presentation seq advances, and owns a buffered battle-time clock. Existing EditMode tests cover
  representative revision/seq/resync, movement timing, projectile marker and reconcile cases.
- Actual battlefield consumption is narrower than the Unity transport document's aspirational list. Field model and
  actor view consume `UnitSpawned` identity, `MovementSegmentStarted`/`MovementStopped` and interpolate movement;
  damage feedback consumes `HpChanged`. No runtime usage of the Spine action API was found outside the isolated
  SpinePresentation runtime/sandbox.
- Projectile and skill-area event names appear only in `BattleRuntimeState` readiness-marker logic. This delays
  checkpoint reconcile to authored end times but does not render attack, projectile, skill or area VFX. Tests cover
  the basic-attack projectile marker, not every hard-coded marker kind.
- Unknown events are accepted as raw JSON but get no readiness marker and are marked presented immediately. Because
  official checkpoints contain active units only, `UnitDied` can allow checkpoint reconcile and actor destruction
  without a battle-wired death animation. There is no stable presentation-instance registry or cancel/interrupt/end
  lifecycle for continuing Boss telegraphs/mechanics.
- `ApplySetupSnapshot` calls `Reset()` only when `battle_uuid` changes. A standalone setup for the same battle simply
  replaces the setup object; it does not discard event queue/presentation markers/checkpoint state as the stale Unity
  transport document says setup-loss recovery should. An ordinary same-revision update is then ignored. This is a
  concrete same-battle reset/rebuild gap, although a brand-new client store receiving attach recovery from seq 0 takes
  the new-battle/empty-state path successfully.
- The renderer destroys actors/HUDs when they disappear from the reconciled active-unit checkpoint and clears the
  whole field on null/new signature. Movement stop/end is timed and tested; generic action cancellation and death/
  Boss duration cleanup are not battle-wired.
- Checkpoint parser reads optional `facing` and defaults the actor to right when absent. Current Core public unit
  checkpoint does not emit facing, so authoritative direction is not actually reconstructed.

### Test result and decision

Static source/test inspection only; no stale Unity runner was executed. An internal Core tracking slice using current
movement/damage events can proceed without a new Boss-specific consumer event. Any continuing telegraph/public
phase/objective, attack/skill/death visual, or cancelable Boss duration is `coimplement_with_first_wire_change` across
Core event/checkpoint, server parity and Unity instance-owned presentation. Same-battle setup-loss reset is existing
consumer debt and must be fixed/covered before claiming reconnect-safe Boss presentation. Unity remains a downstream
sync gate, not a producer-side kernel blocker.

## 2026-08-08 — Existing refactor candidate source correlation

### Hypothesis

2026-08-07 final/phase catalog의 named candidates를 원문으로 읽으면 Boss audit가 기존 후보를 과도하게 선행
조건으로 끌어들이거나 이미 발견된 correctness를 새 Boss 문제로 중복 소유하는 것을 막을 수 있다.

### Exact commands

```text
rg -n 'CRA-P01-C01|CRA-P01-C02|CRA-P05-C02|CRA-P05-C04|CRA-P06-C01|CRA-P06-C02|CRA-P06-C03|CRA-P07-C01|CRA-P07-C02|CRA-P07-C03|CRA-P07-C04|CRA-P08-C01|CRA-P08-C05|CRA-P09-C03|CRA-P09-C05|CRA-P09-C06|CRA-P10-C02|CRA-P10-C03|CRA-P10-C04' core_enhanced/docs/goals/core_enhanced_refactoring_audit_* -g '*.md'
nl -ba core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_01_live_data_loading_and_validation/CANDIDATE_CATALOG.md | sed -n '1,80p'
nl -ba core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_05_combat_preview_and_setup/CANDIDATE_CATALOG.md | sed -n '40,155p'
nl -ba core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_06_battle_lifecycle_and_result/CANDIDATE_CATALOG.md | sed -n '1,100p'
nl -ba core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_07_battle_spatial_and_tactical/CANDIDATE_CATALOG.md | sed -n '1,140p'
nl -ba core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_08_battle_effects_and_stats/CANDIDATE_CATALOG.md | sed -n '1,140p'
nl -ba core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/CANDIDATE_CATALOG.md | sed -n '45,155p'
nl -ba core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_10_command_projection_and_server_seam/CANDIDATE_CATALOG.md | sed -n '25,125p'
```

### Actual result

- P06-C01/C02/C03 원문은 이번 source trace와 정확히 일치한다: max-time API divergence, unequal event `Equal`,
  post-step cursor/projection failure. 이 셋은 중복 Boss 구현이 아니라 existing correctness wave를 먼저 닫아야
  하는 scheduler/world prerequisites다.
- P01-C02/P05-C02는 live opt-in publication 이전 gate다. Isolated Boss kernel/typed schema tests를 막지는 않지만
  실제 Boss encounter를 official live bundle에 넣기 전 manifested difficulty + full setup/Boss binding feasibility
  matrix가 반드시 필요하다. P01-C01/P05-C04는 Boss catalog와 immutable setup binding을 추가할 때 함께 하면
  change amplification을 줄인다.
- P07-C01/C02는 existing tactical command correctness이고 Boss graph owner와 합치지 않는다. Shared early cursor/
  transaction primitive interface만 P06-C03과 조율한다. P07-C03/C04 cleanup은 새 Boss intent/result seam을 먼저
  관찰한 뒤 수행해야 premature removal을 피한다.
- P08-C01/C05와 P09-C03/C05/C06은 leaf trigger/damage/skill/projectile/tile-reach concerns다. Boss runtime은 public
  or typed leaf entrypoints만 호출하고 lifecycle owner를 흡수하지 않는다. P09-C03의 multi-step usefulness policy는
  Boss activation target query와 별개이며 Boss를 gate하지 않는다; P09-C06 tile reach도 navigation/path cost가 아니다.
- P10-C02/C04는 command/root snapshot refactors로 독립적이다. P10-C03의 exact DTO reuse는 first internal slice가
  existing wire만 쓸 때 선행 조건이 아니며, first public Boss Wire change에서 Core/server parity와 함께 실행한다.

### Test result and decision

Artifact/source correlation only; no runner. `BOUNDARY_AUDIT.md` classification을 named candidate 19개에 대해
확정하고 P09-C03을 independent wave, P10-C03을 first-Wire까지 defer로 정밀화한다. 전체 refactor catalog를
Boss 선행 조건으로 확장하지 않는다.

## 2026-08-08 — Final-boss concept stress test

### Hypothesis

백야/고요한 오케스트라/종말새를 requirement가 아니라 stress input으로 대조하면 v1 kernel이 future mechanic을
하드코딩하지 않고 확장점을 제공하는지 판정할 수 있다.

### Exact commands

```text
nl -ba core_enhanced/docs/skills/abnormality_skill_design_notes.ko.md | sed -n '1,225p'
nl -ba core_enhanced/docs/skills/abnormality_skill_design_notes.ko.md | sed -n '196,275p'
nl -ba core_enhanced/docs/skills/abnormality_skill_design_notes.ko.md | sed -n '265,505p'
```

### Actual result

- Source itself labels these as future concepts, not current runtime/data/wire contracts. Exact skills, timings,
  numbers and several mechanic choices remain open and must not be invented by this readiness audit.
- WhiteNight stresses encounter-owned objectives/phases, multiple actor/zone bindings, authored threshold waves,
  terminal interception into a VN question, partial encounter reset, field work and roster/run-local history. Generic
  phase/mechanic extension points fit; third-side apostles, resurrection/reset and question orchestration are later
  approved mechanics, not v1 kernel primitives.
- Silent Orchestra stresses battle-time phase deadlines, phase-specific immunity/target usefulness, wave scheduling,
  end-of-phase global outcomes and a finite encounter resource choice. These map cleanly to encounter state,
  conditions/events, phase entry/exit and typed leaf effect hooks without authoring an Orchestra-specific state kind.
- Apocalypse Bird stresses multiple persistent objectives, irreversible power shutdown, concurrent scheduler hazards,
  phase transition cancellation, a large board, fog/visibility, tactical deployables and camera/minimap UX. The kernel
  can own objective/phase/mechanic state and cancellation epochs; visibility/nav/deployable/camera systems remain
  separate future owners.
- Across all three, the required architectural property is an encounter runtime plus per-actor runtime, typed finite
  graph, explicit event/condition adapters and mechanic extension interface. It is not a universal script VM or a
  catalog of final-boss-specific enums.

### Test result and decision

Concept comparison only. v1 acceptance is that each concept can be decomposed into existing leaf execution plus
explicit future mechanic adapters without changing phase/pattern/task fundamentals. Implementing any concept,
inventing open policy, fog/navigation/camera/VN-reset systems or exposing internal graph IDs is out of scope.

## 2026-08-08 — BP-01~BP-34 exact coverage verification

### Hypothesis

`POLICY_COVERAGE_MATRIX.md`가 승인된 34개 정책을 정확히 한 번씩 포함하고 각 행에 허용된 primary
disposition 하나를 두면 compact 이후에도 누락·중복이나 암묵적 정책 결론을 다시 추론할 필요가 없다.

### Exact commands

```text
rg -n '^\| BP-[0-9]{2} \|' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/POLICY_COVERAGE_MATRIX.md
rg '^\| BP-[0-9]{2} \|' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/POLICY_COVERAGE_MATRIX.md | wc -l
rg '^\| BP-[0-9]{2} \|' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/POLICY_COVERAGE_MATRIX.md | sed -E 's/^\| (BP-[0-9]{2}) .*/\1/' | sort | uniq -d
comm -3 <(seq -w 1 34 | sed 's/^/BP-/') <(rg '^\| BP-[0-9]{2} \|' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/POLICY_COVERAGE_MATRIX.md | sed -E 's/^\| (BP-[0-9]{2}) .*/\1/' | sort)
rg '^\| BP-[0-9]{2} \|' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/POLICY_COVERAGE_MATRIX.md | rg -o '`(existing_primitive_sufficient|existing_primitive_needs_correction|new_boss_runtime_foundation_required|coordinate_with_existing_refactor|policy_conflict|unverified_external|not_applicable_to_v1)`' | sort | uniq -c
```

### Actual result

- Row count: exactly 34.
- Duplicate-ID output: empty.
- Expected `BP-01`~`BP-34` versus actual symmetric difference: empty.
- Primary disposition counts: `coordinate_with_existing_refactor` 4,
  `existing_primitive_needs_correction` 2, `new_boss_runtime_foundation_required` 28.
- No row needs `policy_conflict`, `unverified_external`, or a whole-policy `not_applicable_to_v1` verdict. The future
  subfeatures under BP-02/BP-28 have explicit reject/future-owner handling inside those rows.

### Test result and decision

Artifact integrity check passed. Every row also names current code/canonical evidence, the missing authority/seam,
implementation owner/sequence and a test-first acceptance condition. This closes the exact-coverage investigation
unit; implementation tests remain future work because this Goal authorizes no production mutation.

## 2026-08-08 — Implementation readiness synthesis integrity check

### Hypothesis

누적 boundary/policy/refactor evidence를 implementation handoff로 종합하되 readiness와 live publication을
분리하면 후속 담당자가 대화 기억 없이 최소 dependency와 slice 범위를 재현할 수 있다.

### Exact commands

```text
rg -n '^## (Verdict|Required dependency order|G1|G2|Test-first safety net|G3a|G3b|Final-boss stress result|Handoff boundary)' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/IMPLEMENTATION_READINESS_REPORT.md
rg -o 'CRA-P[0-9]{2}-C[0-9]{2}' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/IMPLEMENTATION_READINESS_REPORT.md | sort | uniq
rg -n 'public Wire|stale Unity|no production|authorizes no production|정책 결정' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/IMPLEMENTATION_READINESS_REPORT.md
```

### Actual result

- Report contains explicit verdict, dependency order, G1 foundation, G2 tracking slice, test-first matrix, G3a live
  publication, G3b public presentation, final-boss stress and handoff boundaries.
- Referenced named dependencies are P01-C01/C02, P05-C02/C04, P06-C01/C02/C03 and P10-C03. Other correlated
  candidates remain in `BOUNDARY_AUDIT.md` with their independent/deferred classifications rather than becoming gates.
- Readiness text explicitly distinguishes stale Unity/public Wire, says no additional user policy decision remains,
  and repeats the production no-mutation/authorization boundary.

### Test result and decision

Document structure/source-reference check passed. Verdict is `conditionally_ready`: G0 correctness can be separately
authorized now; G1/G2 follows it; official live opt-in and public presentation remain distinct gated follow-ups.
No runtime runner was executed because the target runtime is not implemented and this synthesis introduces no
production diff.

## 2026-08-08 — Concurrent Unity change isolation and Spine boundary re-audit

### Hypothesis

Final dirty-state drift may be concurrent user/environment work rather than this Goal, but a new Spine presentation
surface could invalidate the stale-consumer finding and therefore must be re-read before completion.

### Exact commands

```text
git status --short --untracked-files=all -- core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit
git status --short --untracked-files=all -- core_enhanced/src core_enhanced/tests core_enhanced/docs game_resources game_server ':(exclude)core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/**' | wc -l
git status --short --untracked-files=all -- core_enhanced/src core_enhanced/tests core_enhanced/docs game_resources game_server ':(exclude)core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/**' | sha256sum
git -C '/mnt/f/unity projects/ark' status --short --untracked-files=all | wc -l
git -C '/mnt/f/unity projects/ark' status --short --untracked-files=all | sha256sum
git -C '/mnt/f/unity projects/ark' status --short --untracked-files=all -- docs Assets/Scripts/ArkClient Assets/Tests | wc -l
git -C '/mnt/f/unity projects/ark' status --short --untracked-files=all -- docs Assets/Scripts/ArkClient Assets/Tests | sha256sum
stat -c '%n|birth=%w|mtime=%y' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/PLAN.md core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/EXPERIMENTS.md
find '/mnt/f/unity projects/ark/docs' '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient' '/mnt/f/unity projects/ark/Assets/Tests' -type f -newermt '2026-08-08 22:42:39 +0900' -printf '%TY-%Tm-%Td %TH:%TM:%TS %p\n' | sort | wc -l
find '/mnt/f/unity projects/ark/docs' '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient' '/mnt/f/unity projects/ark/Assets/Tests' -type f -newermt '2026-08-08 22:42:39 +0900' -printf '%TY-%Tm-%Td %TH:%TM:%TS %p\n' | sort | tail -n 100
rg -n "SpinePresentation|SpineUnitVisual|BattleRuntimeState|UnitSpawned|MovementSegmentStarted|HpChanged|events_delta|BattleEvent" '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient' '/mnt/f/unity projects/ark/Assets/Tests' -g '*.cs' -g '*.asmdef'
sed -n '1,260p' '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/SpinePresentation/Runtime/SpineUnitVisual.cs'
sed -n '1,280p' '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/SpinePresentation/Sandbox/SpinePresentationSandboxController.cs'
sed -n '1,260p' '/mnt/f/unity projects/ark/Assets/Tests/EditMode/SpinePresentation/SpinePresentationContractTests.cs'
sed -n '1,300p' '/mnt/f/unity projects/ark/Assets/Tests/PlayMode/SpinePresentation/SpinePresentationRenderTests.cs'
rg -n "ArkClient\\.SpinePresentation|SpineUnitVisual|IUnitVisual" '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient' -g '*.cs' -g '*.asmdef' | rg -v '/SpinePresentation/' || true
```

### Actual result

- Simulator Goal output consists only of its six current Markdown artifacts; Goal-excluded production scope remains
  exactly `211` entries and initial hash `7358c0145d11bbc1799400d20429174996a1a20ad12461e19ce761a983565cfe`.
- Unity full/supporting scope is now `2108`/`350`, hashes
  `8803df459017a04f9c627a17c5e0f0af01695adc6d3eb4f09c04f2fed4715876` and
  `77ab49c4d3ca0fdf178e6ccc3b1314e1dfe7fd5188144d050e086d236cd755b3`.
- 22 files are newer than Goal start, concentrated in the concurrent `spine_character_presentation_core` campaign:
  Spine runtime/editor/sandbox, its tests, screenshots and Goal docs. Timestamps span 22:49~23:13 after the Goal's
  22:42 start.
- The Spine runtime asmdef references only `spine-unity`/`spine-csharp`; sandbox references only that runtime. Its
  contract tests explicitly reject `ArkClient.Runtime`, JSON and WebSocket dependencies, and sandbox text says
  `semantic local input only`.
- Reverse reference search outside `/SpinePresentation/` returned no output. Current battlefield/runtime still owns
  only spawn/movement/damage presentation and does not map battle events to the new visual primitive.

### Test result and decision

Source/timestamp/status audit only; the concurrent Unity tests were not run or claimed. External status drift is
preserved and attributed to an independently active Spine campaign, not reverted. The Boss audit finding is refined,
not invalidated: G3b may reuse this leaf abstraction, but still needs Core semantic lifecycle, server parity and a
battle-event/seq/checkpoint-aware Unity adapter.

## 2026-08-08 — Goal-exit completion validation

### Hypothesis

Seven required artifacts, 14 completion rows, 14 invariant rows, exact policy/candidate coverage and final scoped
status must all agree before PLAN Status can move from `active` to `complete`.

### Exact commands

```text
for f in PLAN.md EXPERIMENTS.md EXPERIMENT_NOTES.md BOUNDARY_AUDIT.md POLICY_COVERAGE_MATRIX.md IMPLEMENTATION_READINESS_REPORT.md COMPLETION_AUDIT.md; do test -s "core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/$f" || echo "missing:$f"; done
rg '^\| BRA-C[0-9]{2} \|' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/COMPLETION_AUDIT.md | wc -l
rg '^\| BRA-I[0-9]{2} \|' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/COMPLETION_AUDIT.md | wc -l
rg -n '^## (Primary verdict|Status transition|Remaining risk and required action)$|`Complete / High`|^Open policy questions: none\.$' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/COMPLETION_AUDIT.md
rg -n '^\|.*\| fail \|$' core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/COMPLETION_AUDIT.md || true
git status --short --untracked-files=all -- core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit
git status --short --untracked-files=all -- core_enhanced/src core_enhanced/tests core_enhanced/docs game_resources game_server ':(exclude)core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/**' | wc -l
git status --short --untracked-files=all -- core_enhanced/src core_enhanced/tests core_enhanced/docs game_resources game_server ':(exclude)core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/**' | sha256sum
git -C '/mnt/f/unity projects/ark' status --short --untracked-files=all | wc -l
git -C '/mnt/f/unity projects/ark' status --short --untracked-files=all | sha256sum
git -C '/mnt/f/unity projects/ark' status --short --untracked-files=all -- docs Assets/Scripts/ArkClient Assets/Tests | wc -l
git -C '/mnt/f/unity projects/ark' status --short --untracked-files=all -- docs Assets/Scripts/ArkClient Assets/Tests | sha256sum
```

### Actual result

- Missing artifact output: empty; all seven required Markdown files are non-empty.
- Completion rows: 14. Invariant rows: 14. `fail` row output: empty.
- Completion artifact exposes `Complete / High`, Primary verdict, Status transition, Remaining risk and `Open policy
  questions: none`.
- Goal path status contains exactly the seven expected untracked Markdown artifacts.
- Simulator Goal-excluded production scope remains 211 entries with initial hash
  `7358c0145d11bbc1799400d20429174996a1a20ad12461e19ce761a983565cfe`.
- Unity status remains stable at the already isolated concurrent state: full 2108 / supporting 350, hashes
  `8803df459017a04f9c627a17c5e0f0af01695adc6d3eb4f09c04f2fed4715876` /
  `77ab49c4d3ca0fdf178e6ccc3b1314e1dfe7fd5188144d050e086d236cd755b3`.

### Test result and decision

Goal-exit artifact/traceability/dirty-preservation validation passed. No runtime test runner was invoked and no
pass/fail/skip test count is claimed. `COMPLETION_AUDIT.md` independently returns `Complete / High`; PLAN may now move
to `complete` while `Implementation Complete` remains `not_applicable` and readiness remains `conditionally_ready`.
