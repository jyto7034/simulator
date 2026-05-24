# 리팩토링 준비 계획

이 문서는 지금까지 빠르게 구현된 `core_enhanced`를 본격 리팩토링하기 전에, 다른 AI나 개발자가 같은 기준으로 코드를 읽고 기술 부채를 정리할 수 있게 만든 실행 계획이다.

목표는 즉시 코드를 갈아엎는 것이 아니라, 범주별로 책임 경계를 확인하고 중복 설계, 임시 호환 경로, 의미가 약한 코드, 테스트 부채를 식별한 뒤 안전한 순서로 리팩토링하는 것이다.

## 현재 판단

최근 구현은 게임 흐름을 빠르게 세우는 데 성공했지만, 여러 정책이 빠르게 추가되면서 다음 위험이 커졌다.

- `world.rs`, `events/combat.rs`, `combat_preview.rs`, `battle/core`가 각각 게임 정책, 전투 미리보기, 시나리오 변환, 런타임 처리를 동시에 들고 있다.
- 같은 개념이 서로 다른 이름으로 여러 층에 존재한다. 예: `CombatNodeType`, `BattlefieldArchetype`, `CombatMissionRisk`, `WinCondition`, `BattleObjective`, `Pve*Data`.
- 과거 목적의 타입이 일부 데이터/테스트 호환으로 남아 있을 수 있다. live 경로에 남아 있는 경우 우선 제거한다. 예: deprecated mission, phase-era event pool naming.
- 테스트가 많지만 “현재 게임 흐름을 보장하는 테스트”와 “과거 구조를 묶어두는 테스트”가 섞여 있다.
- 문서가 많아졌고, 일부 문서는 설계 기록인지 현재 룰인지 인수인계인지 역할이 겹친다.

따라서 리팩토링은 “파일 크기 축소”보다 “source of truth를 한 곳으로 모으는 것”을 우선한다.

## 리팩토링 원칙

- 레거시는 기능 보존 목적이 명확할 때만 남긴다. live 경로에서 사용하지 않는 과거 호환 코드는 deprecated 표시만으로 오래 보존하지 않는다.
- 새 fallback을 추가하지 않는다. 미리보기와 실제 전투 입력이 어긋나면 fallback이 아니라 데이터 계약 또는 변환 경계를 고친다.
- RON authoring 타입과 runtime 타입을 혼동하지 않는다. RON은 작성 편의, `BattleScenario`는 전투 시작 입력, `BattleCore`는 실행만 담당한다.
- 전투 중 직접 조작보다 전투 전 정보/배치가 중요하다는 게임 목적을 흐리는 코드는 제거 또는 격리한다.
- 수치 밸런스는 쉽게 바꿀 수 있게 두되, 아직 재미 검증이 필요한 수치는 리팩토링의 핵심 판단 기준으로 삼지 않는다.
- 문서는 현재 룰과 실행 계획을 분리한다. `game_rulebook.md`는 룰, `current_handoff.md`는 현재 상태, 이 문서는 리팩토링 준비/점검표다.
- 문서의 계획을 무조건 따르지 않는다. 실제 코드를 읽고 더 적은 변경으로 더 명확한 책임 경계와 더 나은 장기 구조를 만들 수 있다면, 코드 근거가 있는 개선안을 우선 적용한다.
- 궁금한 점이나 게임 정책이 모호한 부분은 임의로 확정하지 않는다. 합리적인 기본값으로 진행 가능한 구현 세부사항이 아니라 룰/플레이 감각/보상 구조에 영향을 주는 결정이라면, 먼저 사용자와 의논해 정책을 확정한 뒤 문서와 코드를 수정한다.

## 문서와 코드 판단 우선순위

이 문서는 다음 작업자가 빠르게 방향을 잡기 위한 준비 계획이지, 반드시 그대로 따라야 하는 고정 명령이 아니다. 실제 코드를 읽었을 때 문서의 추천 순서보다 더 작고 안전한 개선점, 더 명확한 책임 경계, 더 먼저 제거해야 할 레거시가 보이면 코드 근거를 우선한다.

작업 기준:

- 문서의 “다음 단계”는 기본 후보로 본다.
- 실제 코드를 읽고 더 나은 개선점이 확인되면 그 작업을 선택한다.
- 단, 선택 이유를 문서나 최종 보고에 남긴다.
- 문서와 코드가 충돌하면 코드의 현재 동작과 테스트 계약을 먼저 신뢰한다.
- 문서가 틀렸거나 낡았다고 판단되면 코드 수정과 함께 문서를 갱신한다.
- “더 나은 개선점”은 구조가 멋진 것이 아니라, 더 적은 변경으로 더 많은 중복/혼동/레거시를 줄이는 것이다.
- 정책이 모호할 때는 구현 편의로 임의 결정하지 않는다. 게임 룰, 노드 흐름, 보상, 직원 성장, 전투 실패/성공 판정에 영향을 주는 내용은 의논 후 확정한다.
- 코드 정리 중 궁금한 점이 생겼을 때, 단순 구현 세부사항은 합리적으로 진행하되 플레이 감각이나 세계관 정책을 바꾸는 질문은 사용자와 먼저 의논한 뒤 수정한다.

판단 예시:

- 문서는 `enemy draft 변환` 분리를 다음 후보로 적었지만, 실제로는 `BattleScenario` 조립 흐름이 더 큰 혼선을 만든다면 scenario builder 쪽을 먼저 본다.
- 문서는 모듈 분리를 추천하지만, 코드상 단순 함수 이동만으로 충분하면 새 모듈을 만들지 않는다.
- 문서가 남겨둔 deprecated 경로가 live/test 어디에서도 쓰이지 않는 것이 확인되면 wrapper를 추가하지 말고 제거를 우선한다.

## 과도한 리팩토링 방지 기준

가장 경계해야 할 리팩토링 실패는 복잡도를 줄이기 위해 시작했는데, 미래 가능성을 핑계로 새 추상화와 파일 경계를 더 많이 만드는 것이다. 이 프로젝트의 리팩토링은 “더 멋진 구조”가 아니라 “현재 게임 흐름을 더 쉽게 읽고 안전하게 바꾸는 구조”를 목표로 한다.

과도한 리팩토링 신호:

- 아직 실제 변형이 1~2개뿐인데 `Trait`, `Policy`, `Resolver`, `Strategy` 계층을 먼저 만든다.
- 코드를 읽을 때 게임 규칙보다 추상화 이름을 먼저 이해해야 한다.
- 파일은 쪼갰지만 책임은 그대로 섞여 있어, 수정하려면 더 많은 파일을 열어야 한다.
- live RON과 현재 게임 흐름에 없는 미래 기능 때문에 현재 코드가 우회한다.
- 제거 가능한 레거시를 삭제하지 않고 adapter나 compatibility layer로 감싼다.
- 테스트가 “게임이 맞게 도는지”보다 “새 추상화 모양이 유지되는지”를 고정한다.
- 단순 함수나 테이블로 충분한 곳에 동적 dispatch, generic, trait object를 도입한다.
- 리팩토링 후 public API, DTO, RON schema가 이유 없이 더 복잡해진다.

좋은 리팩토링 기준:

- source of truth 개수가 줄어든다.
- 같은 변경을 위해 열어야 하는 파일 수가 줄어든다.
- live 경로에서 안 쓰는 코드가 삭제된다.
- 현재 정책을 한 곳에서 읽을 수 있다.
- 테스트는 내부 구조보다 사용자/게임 흐름 계약을 더 많이 고정한다.
- 새 추상화는 최소 두세 개 이상의 실제 호출부나 변형이 확인된 뒤 도입한다.

판단 규칙:

```text
1. 먼저 삭제할 수 있는가?
2. 삭제할 수 없다면 단순 함수로 모을 수 있는가?
3. 단순 함수가 반복되기 시작했는가?
4. 반복의 축이 실제 게임 정책으로 확인됐는가?
5. 그때만 새 타입/정책/trait를 고려한다.
```

따라서 첫 리팩토링도 큰 아키텍처 재설계가 아니라, 이미 존재하는 중복 정책을 단순한 모듈과 함수로 모으는 수준에서 시작한다. 예를 들어 `CombatNodeType -> 기본 전술/승리 조건/보상 정체성/위험도` 정리는 처음부터 trait 기반 framework로 만들지 말고, 읽기 쉬운 table/function 기반 `combat_mission_policy`로 시작한다.

## Phase-Era 레거시 제거 기준

현재 게임 흐름의 source of truth는 노드맵이다. 과거의 `Ordeal -> Phase -> PhaseEvent -> EventSelection` 진행 모델은 더 이상 live runtime 정책이 아니다.

제거된 live 경로:

- `GameState::WaitingPhaseRequest`
- `GameState::SelectingEvent`
- `PlayerBehavior::RequestPhaseData`
- `PlayerBehavior::SelectEvent`
- `BehaviorResult::AdvancePhase`
- `BehaviorResult::RandomEventState`
- `CurrentPhaseEvents`
- `GameProgression`
- phase advance / ordeal scheduler / phase resolver / event manager 계열 모듈

재사용했던 것:

- 상점 구매/판매/리롤의 검증과 실행 로직
- 보상 세션 선택/수령 로직
- 랜덤 이벤트 메타데이터가 Shop/Reward/Suppress 중 무엇을 가리키는지 해석하는 데이터 구조는 임시 잔재다. 현재 공식 노드 흐름에는 맞지 않으므로 제거 대상이다.

현재 배치:

- 노드맵 진입은 `ViewingMap -> NodeConfirm -> ConfirmEnterNode`가 기준이다.
- Shop/Reward 노드는 phase selection을 거치지 않고 즉시 `InShop` 또는 `InReward` 세션으로 진입한다.
- 기존 Event 노드는 Shop/Reward 래퍼일 뿐이므로 공식 live node에서 제거 완료했다.
- 보상 선택은 `SelectReward`만 사용한다. 이벤트 선택이라는 이름은 더 이상 사용하지 않는다.
- 전투 보상/보너스/상점 종료는 노드 세션이 있을 때만 노드 완료로 이어진다. phase fallback은 없다.

남은 주의점:

- `MapNodeCategory::Event`, `MapNodePayload::Event`, `RandomEventDatabase` live loading, `event_random`, `event_abnormality_room`, Event 관련 테스트는 제거 완료했다. 기존 RON은 legacy 이름으로만 격리하고 런타임/테스트에서는 읽지 않는다.
- 추후 랜덤 이벤트가 필요하면 현재 Event 래퍼를 유지하지 말고, `상황 설명 -> 2~3개 선택지 -> 비용/리스크/보상 적용 -> 결과 -> 맵 복귀`를 갖는 새 `RandomEvent` 노드로 다시 작성한다.
- 단, 데이터 로딩 테스트와 fixture를 대량으로 깨는 이름 변경은 “삭제 가능한 live 레거시”가 아니라 별도 data schema migration으로 다룬다.

## 전체 작업 순서

1. 문서와 코드의 source of truth를 확정한다.
2. 게임 시스템 흐름을 읽고 state/action/result 경계를 정리한다.
3. 전투 시나리오와 조우 데이터 변환 경계를 읽는다.
4. 이동/전술 AI를 전투 런타임 안에서 별도 하위 시스템으로 검토한다.
5. 스킬/효과 런타임을 전투 core에서 분리 가능한 단위로 검토한다.
6. 보상/장비/스킬 파편/연구 정책을 게임 시스템 계층에서 검토한다.
7. 테스트를 “현재 계약 보장”과 “레거시 고정”으로 나눈다.
8. 실제 리팩토링 순서를 작게 쪼개고 각 단계마다 `cargo test -p game_core`로 고정한다.

## 범주 1. 게임 시스템 흐름

### 읽을 파일

- `src/game/world.rs`
- `src/game/world/helpers.rs`
- `src/game/world/snapshot.rs`
- `src/game/world/support.rs`
- `src/game/world/state.rs`
- `src/game/behavior.rs`
- `src/game/managers/action_scheduler.rs`
- `src/game/resources/*.rs`
- `src/game/map/*.rs`

### 확인할 질문

- `GameCore::execute`가 너무 많은 세부 정책을 직접 알고 있지 않은가?
- `GameState`, `SelectedEventState`, `NodeSession`, `BehaviorResult`가 같은 상태를 중복 표현하지 않는가?
- `NodePreview -> NodeConfirm -> ConfirmEnterNode -> NodeResolved` 흐름이 한 곳에서 추적 가능한가?
- 지원 노드, 정비 노드, 연구 배송, 배치 변경이 각각 action gate와 state gate를 중복 구현하지 않는가?
- 런 실패 조건이 `world.rs`, helper, post-battle resolution, node selection validation에 흩어져 있지 않은가?
- 스냅샷 DTO가 내부 상태 구조에 너무 강하게 결합되어 있지 않은가?

### 부채 후보

- `world.rs`가 2500줄 이상으로, 실행 dispatch와 각 도메인 정책이 계속 누적되고 있다.
- `handle_confirm_enter_node` 계열은 노드 소비, 연구 배송, 배치 검증, 세션 진입, 전투 시작을 동시에 다룰 가능성이 높다.
- `ActionScheduler`와 `GameCore` validation이 같은 action 가능 여부를 서로 다른 기준으로 판단할 위험이 있다.
- 지원 노드가 현재는 단순하지만, Medical/Rest/Maintenance 행동이 늘어나면 `world/support.rs`만으로 감당하기 어렵다.

### 리팩토링 방향

- `GameCore`는 orchestrator로 남기고, 노드 진입/완료를 `NodeFlowService` 성격의 내부 모듈로 분리한다.
- 전투 노드 전용 흐름은 `CombatNodeSessionService`로 분리한다. 미리보기, 배치 검증, 전투 시작, 결과 반영의 책임을 묶는다.
- 지원 노드 반복 행동은 `SupportSessionService`로 분리한다.
- 런 실패 판정은 `RunFailurePolicy` 또는 `RunContinuationPolicy`로 모아 한 곳에서만 판단한다.

## 범주 2. 전투 시나리오와 조우 데이터

### 읽을 파일

- `src/game/combat_preview.rs`
- `src/game/events/combat.rs`
- `src/game/battle/scenario.rs`
- `src/game/data/pve_data.rs`
- `src/game/data/corroded_wave_data.rs`
- `src/game/data/corroded_employee_data.rs`
- `src/game/combat_mission_policy.rs`
- `src/game/reward_policy.rs`
- `game_resources/data/pve/encounters.ron`
- `game_resources/data/map/battlefield_templates.ron`
- `game_resources/data/map/battlefield_archetypes.ron`

### 확인할 질문

- `PveEncounter -> CombatPreview -> BattleScenario -> BattleCore` 변환 경계가 명확한가?
- `CombatNodeType`은 임무 목적, `BattlefieldArchetype`은 필드 형태, `CombatMissionRisk`는 임무 위험도라는 분리가 코드에서 유지되는가?
- `events/combat.rs`가 combat node adapter인지, scenario builder인지, reward resolver인지 역할이 섞여 있지 않은가?
- 기본 전술/승리 조건이 `CombatNodeType`별로 한 곳에 모여 있는가?
- RON에서 authoring 가능한 필드와 runtime에서만 계산되는 필드가 섞여 있지 않은가?
- live RON에만 적용돼야 하는 제약이 test fixture까지 불필요하게 압박하지 않는가?

### 부채 후보

- `combat_preview.rs`가 2000줄 이상이며, 타입 정의, RON 로딩, ASCII 템플릿 파싱, 생성기, 검증, 테스트를 모두 포함한다.
- `events/combat.rs`가 2000줄 이상이며, roster 변환, enemy 변환, scenario 생성, 기본 전술, 보상 해석, 테스트를 함께 담고 있다.
- `PveWinConditionData`와 `WinCondition`, `PveBattleObjectiveData`와 `BattleObjective`가 비슷한 구조를 반복한다.
- 완료된 정리: deprecated `DefendPoint`, 이전 회수형 `RecoverAndExtract`는 live/runtime/test 계약에서 제거하고, 현재 방어/회수 계약은 `ProtectUnit`과 `RecoverHoldAndExtract`로 정리한다.
- 완료된 정리: `CombatNodeIntentPolicy` thin wrapper는 제거하고, 맵 조우 배정/preview fallback/live RON 검증은 `CombatMissionPolicy`를 직접 참조한다.

### 리팩토링 방향

- `combat_preview.rs`를 `preview/types.rs`, `preview/template.rs`, `preview/generator.rs`, `preview/validation.rs`로 분리한다.
- `events/combat.rs`의 scenario 생성 책임을 `combat/scenario_builder.rs`로 이동한다.
- `CombatNodeType`별 기본 전술/승리 조건/보상 정체성은 하나의 `CombatMissionPolicy` 계층으로 묶는다.
- deprecated win condition은 live 경로에서 완전히 제거 가능한지 먼저 테스트로 확인한 뒤 삭제한다.
- RON 변환 타입은 `data/pve_data.rs`에 남기되 runtime 타입과 1:1로 맞추려 하지 않는다. 작성 편의용 enum과 runtime enum은 의도적으로 분리한다.

### PveEncounter -> CombatPreview -> BattleScenario Source Of Truth

전투 시작 파이프라인은 한 방향이어야 한다. `PveEncounter`는 RON authoring 계약, `CombatPreview`는 선택된 노드의 확정된 전장/웨이브 입력, `BattleScenario`는 전투 런타임 시작 입력이다.

| 필드/개념 | Source of truth | Preview 변환 | Scenario 변환 |
|---|---|---|---|
| 전투 목적 | `PveEncounter.node_type` | `CombatPreview.node_type`에 복사. authored encounter가 없을 때만 archetype fallback 허용 | `CombatExecutor`가 preview node type으로 기본 tactical plan/win condition 선택 |
| 전장 형태 | `PveEncounter.battlefield` 또는 map/category seed | `BattlefieldGenerator`가 ASCII template을 선택해 크기, 유효 타일, 배치/스폰 구역, 장애물 확정 | `BattleStartPlan`이 preview battlefield를 `BattleFieldSpec`으로 복사 |
| 적 웨이브 | `PveEncounter.waves` | `CombatPreview.spawn_waves`로 확정. GeneratedCorroded도 이 시점에 고정 | `CombatExecutor`는 preview spawn waves만 `ScenarioSpawnGroup`/`SpawnGroup` 이벤트로 변환 |
| 보상 정체성 | `PveEncounter.reward_uuids` + `CombatNodeType` | preview에는 직접 보상 목록을 싣지 않음 | 전투 종료 후 active `CombatNodeType`으로 `CombatRewardPolicy` 검증 |
| 기본 전술/승패 | `CombatMissionPolicy` | preview의 node type, mission risk, battlefield 정보를 기준으로 결정 가능 | authored override가 없으면 `CombatMissionPolicy` 기본값 사용 |
| authored 전술/승패 override | `PveEncounter.tactical_plan`, `PveEncounter.win_condition` | preview에는 표시용 전장/웨이브만 확정 | `CombatExecutor`가 default plan 위에 authored override를 적용 |

현재 코드 계약:

- 전투 시작 시 `CombatPreview.encounter_id`는 요청한 `encounter_id`와 일치해야 한다.
- authored `PveEncounter.node_type`이 있으면 `CombatPreview.node_type`과 반드시 일치해야 한다.
- `CombatPreview.spawn_waves`가 비어 있으면 `PveEncounter.waves`로 다시 추론하지 않고 실패한다.
- live RON은 `node_type`을 반드시 작성해야 하며, `SplitOperation`처럼 아직 live 지원되지 않는 목적은 사용할 수 없다.

## 범주 3. 전투 런타임

### 읽을 파일

- `src/game/battle/core/mod.rs`
- `src/game/battle/core/sim.rs`
- `src/game/battle/core/build.rs`
- `src/game/battle/core/commands.rs`
- `src/game/battle/core/types.rs`
- `src/game/battle/core/triggers.rs`
- `src/game/battle/timeline.rs`
- `src/game/battle/replay/*.rs`
- `src/game/battle/validation/*.rs`

### 확인할 질문

- `BattleCore`가 전투 루프, 이벤트 큐, 스킬 실행, 이동 계획, 승패 판정, 타임라인 기록을 모두 직접 들고 있지 않은가?
- `compute_winner`가 점점 mission-specific branch로 커지고 있지 않은가?
- timeline event가 클라이언트 계약인지, 테스트 convenience인지, 내부 debug log인지 역할이 섞여 있지 않은가?
- replay validation이 현재 battle runtime을 검증하는지, 과거 타임라인 구조를 유지하기 위한 테스트인지 구분되는가?
- runtime unit 상태와 scenario runtime 매핑이 한 곳에서만 갱신되는가?

### 부채 후보

- `battle/core/mod.rs`, `sim.rs`, `commands.rs`가 각각 2000줄 이상이다.
- `WinCondition`별 판정이 `sim.rs` 안에 직접 분기되어 mission 추가 때마다 core가 커진다.
- 전투 결과가 `BattleWinner`만으로 표현되어 node 실패/성공/부분 보상/런 실패 연결을 `world`가 추가 해석한다.
- timeline export 테스트와 replay validation 테스트가 많아졌으나, 어떤 것이 클라이언트 계약인지 불명확할 수 있다.

### 리팩토링 방향

- 승패 판정은 `WinConditionEvaluator`로 분리한다. `BattleCore`는 evaluator를 호출하고 state 변경만 반영한다.
- timeline 기록은 `TimelineRecorder` 경계로 정리한다. 클라이언트 계약 이벤트와 내부 디버그 이벤트를 분리한다.
- scenario runtime 매핑은 `ScenarioRuntime` 모듈로 분리해 spawn/ref/group 상태를 한 곳에서 관리한다.
- battle validation은 `BattleScenario::validate()`와 runtime invariant validation으로 계층을 나눈다.

## 범주 4. 이동과 전술 AI

### 읽을 파일

- `src/game/battle/core/movement/engine.rs`
- `src/game/battle/core/movement/planner.rs`
- `src/game/battle/core/movement/rapier_backend.rs`
- `src/game/battle/core/movement/steering.rs`
- `src/game/battle/core/movement/types.rs`
- `src/game/battle/core/spatial.rs`
- `src/game/battle/battlefield/*.rs`
- `tests/battle_rapier_movement.rs`
- `tests/movement_timeline_exports.rs`

### 확인할 질문

- 전술 목표(`TacticalPlan`, `TacticalGroupPlan`)와 실제 이동 명령 생성 책임이 분리되어 있는가?
- 개별 유닛 행동, 그룹 앵커, waypoint, leash, collision avoidance가 같은 planner에 과도하게 섞여 있지 않은가?
- ASCII map의 void tile, static obstacle, deployment cell, spawn zone이 같은 좌표계로 끝까지 유지되는가?
- 방어형/포위형처럼 “최대한 움직이지 않는 전투”와 전선형/회수형처럼 “목표로 이동하는 전투”가 같은 규칙으로 억지 처리되고 있지 않은가?
- Rapier backend가 core domain 타입을 너무 많이 알고 있지 않은가?

### 부채 후보

- `movement/planner.rs`가 1600줄 이상이며, tactical objective와 movement fallback 처리가 계속 늘고 있다.
- 그룹 이동은 기초가 생겼지만, formation slot, anchor advancement, reconnect, waypoint가 장기적으로 별도 policy가 될 가능성이 높다.
- `fallback`이라는 이름의 동작이 실제로 의도된 안정화인지, 과거 무제한 추격의 잔재인지 구분해야 한다.

### 리팩토링 방향

- 이동은 세 층으로 나눈다.
- `TacticalIntentResolver`: mission/tactical plan에서 목표 anchor와 allowed radius를 만든다.
- `MovementPlanner`: 목표 위치를 유닛별 desired movement로 바꾼다.
- `MovementBackend`: Rapier/연속 좌표 충돌 처리만 담당한다.
- 방어형, 포위형, 전선형, 회수형의 이동 목적은 planner 안의 분기가 아니라 `TacticalPlan` 데이터로 최대한 표현한다.

## 범주 5. 스킬과 효과 런타임

### 읽을 파일

- `src/game/ability.rs`
- `src/game/battle/core/skill_runtime/*.rs`
- `src/game/battle/core/commands.rs`
- `src/game/battle/core/targeting.rs`
- `src/game/battle/damage.rs`
- `src/game/battle/buffs.rs`
- `src/game/data/skill_data.rs`
- `src/game/data/skill_fragment_data.rs`
- `tests/skill_refactor_validation.rs`
- `tests/skill_test/**/*.rs`

### 확인할 질문

- 스킬 데이터 스키마(`SkillDef`)와 런타임 실행기(`skill_runtime`)가 분리되어 있는가?
- targeting, delivery, damage, buff, projectile, area가 모두 `commands.rs`에 모여 있지 않은가?
- 환상체 원본 스킬과 직원 파편 모방 스킬이 데이터적으로 독립이라는 정책이 코드/테스트에서 보장되는가?
- `SkillAreaTickPolicy`, projectile collision, conditional step 같은 확장점이 battle event loop와 과도하게 결합되어 있지 않은가?
- 스킬 테스트가 개별 스킬 데이터를 검증하는지, 런타임 일반 계약을 검증하는지 구분되는가?

### 부채 후보

- `ability.rs`가 1000줄 이상이고 스키마, 기본값, RON 테스트를 함께 가진다.
- `commands.rs`가 스킬/공격/피해 처리까지 크게 들고 있어, 스킬 확장이 계속될수록 전투 core가 비대해진다.
- 스킬 테스트가 많지만 반복 fixture가 많아 새 스킬 추가 비용이 커질 수 있다.

### 리팩토링 방향

- `ability.rs`를 `ability/schema.rs`, `ability/targeting.rs`, `ability/delivery.rs`, `ability/effects.rs` 성격으로 분리한다.
- `SkillRuntime`은 `CastResolver`, `TargetResolver`, `DeliveryRuntime`, `EffectApplier`로 나눈다.
- 스킬 테스트 helper를 공용 scenario fixture 중심으로 정리하고, 개별 스킬 테스트는 “데이터가 의도한 표현을 한다”에 집중한다.

## 범주 6. 보상, 장비, 스킬 파편, 직원 성장

### 읽을 파일

- `src/game/reward.rs`
- `src/game/reward_policy.rs`
- `src/game/skill_fragment.rs`
- `src/game/employee.rs`
- `src/game/employee_trust.rs`
- `src/game/growth.rs`
- `src/game/resources/inventory.rs`
- `src/game/data/reward_data.rs`
- `src/game/data/equipment_data.rs`
- `src/game/data/employee_data.rs`
- `src/game/data/skill_fragment_data.rs`

### 확인할 질문

- `RewardEffect`가 모든 보상 실행을 직접 알고 있어 계속 커질 구조인가?
- `CombatRewardPolicy`는 검증만 담당하고 실제 지급은 다른 계층에서 하는 현재 경계가 유지되는가?
- 스킬 파편 정책이 `SkillFragmentPolicy`에 충분히 모여 있는가, 아니면 world/support/reward에도 중복되어 있는가?
- 연구 완료 배송 정책이 노드 카테고리 정책으로만 처리되는가?
- 직원의 Run HP/Battle HP, trauma, trust, injury, growth 반영 순서가 한 곳에서 설명 가능한가?
- 장비 복원/분해/강화가 Maintenance node gate와 inventory mutation을 중복 구현하지 않는가?

### 부채 후보

- `SkillFragmentPolicy::default()`와 `default_run_policy()` 사용이 섞이면 정책 source of truth가 흔들릴 수 있다.
- `RewardEffect`가 보상 타입 증가와 함께 god enum이 될 가능성이 있다.
- 직원 상태 변경은 전투 후처리, 지원 노드, 성장, 신뢰도에서 각각 일어나므로 이벤트 기반 감사 로그가 없으면 추적이 어려워진다.

### 리팩토링 방향

- 보상 실행은 `RewardExecutor`를 유지하되, effect별 handler로 분리한다.
- 스킬 파편 inventory mutation은 반드시 `SkillFragmentPolicy`를 인자로 받는 경로로 통일한다.
- 직원 상태 변경은 `EmployeeStateChange` 또는 `RosterMutation` 결과를 반환해 snapshot/result에서 재사용한다.

## 범주 7. 데이터/RON 계약

### 읽을 파일

- `src/game/data/mod.rs`
- `src/game/data/*.rs`
- `tests/ron_loading.rs`
- `game_resources/data/**/*.ron`

### 확인할 질문

- live RON 검증이 데이터베이스 로딩 시점에 충분히 실패하는가?
- test-only fixture와 live-authored-data 제약이 같은 validation에 묶여 있지 않은가?
- `include_str!` 경로와 shared `game_resources` 경로가 장기적으로 유지 가능한가?
- RON authoring이 늘어날수록 사람이 실수하기 쉬운 필드가 무엇인가?

### 부채 후보

- `data/mod.rs`가 live database 전체 validation을 많이 들고 있다.
- `ron_loading.rs`는 중요한 계약을 많이 보장하지만, 파일이 커지면 실패 원인 파악이 어려워진다.
- live 데이터 제약과 밸런스 제약이 같은 테스트에 들어가면 수치 조정 때 리팩토링 안정성이 떨어진다.

### 리팩토링 방향

- live data validation을 domain별 validator로 나눈다.
- `ron_loading.rs`를 skill/reward/pve/map/enemy/equipment 단위로 분리한다.
- RON schema 변경 시 필요한 migration checklist를 문서에 추가한다.

## 범주 8. 테스트와 문서

### 읽을 파일

- `tests/*.rs`
- `tests/skill_test/**/*.rs`
- `src/game/world/tests.rs`
- `docs/*.md`

### 확인할 질문

- 현재 테스트가 새 게임 흐름을 검증하는가, 아니면 제거된 레거시 API를 우회적으로 붙잡고 있는가?
- e2e 성격의 테스트가 너무 많은 내부 세부 구현을 assert하지 않는가?
- timeline export 테스트가 클라이언트 계약을 안정적으로 표현하는가?
- 문서 중 현재 룰과 과거 논의가 섞인 문서가 있는가?

### 부채 후보

- `src/game/world/tests.rs`가 3000줄 이상이다.
- skill test suite는 가치가 높지만 반복 fixture와 개별 스킬 특화 assert가 많아 유지 비용이 커질 수 있다.
- `current_handoff.md`는 인수인계 문서라 계속 길어질 수밖에 없다. 오래된 항목은 rulebook 또는 refactor plan으로 이동해야 한다.

### 리팩토링 방향

- 테스트를 다음 네 종류로 명명한다.
- `contract`: 클라이언트/데이터/외부 경계 보장.
- `flow`: 실제 게임 흐름 보장.
- `runtime`: battle/skill/movement 내부 불변식 보장.
- `legacy_guard`: 의도적으로 남긴 하위 호환만 보장. 오래 유지하지 않는다.
- 문서는 `game_rulebook.md`, `gameplay_flow_example.md`, `current_handoff.md`, 이 문서 네 개를 우선 source로 삼고, 특수 주제 문서는 링크만 유지한다.

## 1차 감사 체크리스트

아래 체크리스트는 실제 코드 수정 전에 완료한다.

- [ ] `world.rs`의 public/private handler 목록을 작성하고 도메인별로 묶는다.
- [ ] `BehaviorResult`가 클라이언트 계약인지 내부 흐름 결과인지 구분한다.
- [ ] `GameState`와 `SelectedEventState`의 중복 상태를 표로 정리한다.
- [ ] `CombatPreview` 생성부터 `BattleScenario` 생성까지 데이터 필드별 source of truth를 표로 정리한다.
- [ ] `CombatNodeType`별 기본 tactical plan, win condition, reward policy를 한 표로 정리한다.
- [x] deprecated `DefendPoint`, `RecoverAndExtract`가 실제 live 경로에 남아 있는지 확인하고 제거한다.
- [ ] `BattleCore::compute_winner`와 mission-specific runtime state를 분리할 수 있는지 확인한다.
- [ ] movement planner의 fallback 동작을 “의도된 안정화”와 “레거시 무제한 추격 방지”로 분류한다.
- [ ] skill runtime에서 targeting/delivery/effect/buff/projectile 책임이 섞인 함수를 표시한다.
- [ ] `SkillFragmentPolicy::default()` 호출부가 run policy와 충돌하지 않는지 확인한다.
- [ ] live RON validation 테스트를 도메인별로 분리할 수 있는지 확인한다.
- [ ] 제거 가능한 legacy test와 유지해야 하는 contract test를 분류한다.

## 2차 리팩토링 실행 순서

1. 문서 정리
   - `current_handoff.md`에서 오래된 완료 기록을 압축한다.
   - 현재 룰은 `game_rulebook.md`로, 실행 계획은 이 문서로 이동한다.

2. 게임 흐름 경계 분리
   - `world.rs`에서 node flow, support flow, combat flow를 내부 모듈로 분리한다.
   - 기능 변화 없이 테스트만 통과시키는 순수 이동 작업으로 시작한다.

3. 전투 preview/scenario builder 분리
   - `combat_preview.rs`와 `events/combat.rs`의 생성/변환 책임을 쪼갠다.
   - `PveEncounter -> CombatPreview -> BattleScenario`를 한 방향 파이프라인으로 고정한다.

4. mission policy 통합
   - `CombatRewardPolicy`, `CombatMissionRisk`, default tactical/win condition의 중복 여부를 확인하고, 실제로 같은 변경축일 때만 mission policy 모듈로 묶는다.
   - `CombatNodeType`별 표준 계약을 테스트로 고정한다.

5. BattleCore 내부 책임 분리
   - 승패 판정, scenario runtime, timeline recording을 별도 모듈로 빼낸다.
   - 전투 결과는 node 후처리가 해석하기 쉬운 구조로 보강한다.

6. 이동/스킬 하위 시스템 정리
   - movement planner를 intent/planner/backend로 나눈다.
   - skill runtime을 targeting/delivery/effect application으로 나눈다.

7. 테스트 재분류와 삭제
   - 현재 계약을 보장하지 않는 legacy test를 삭제하거나 `tests_bak` 성격으로 제외한다.
   - live RON, world flow, battle scenario, skill runtime, movement contract 테스트를 분리한다.

## 첫 번째 실제 작업 추천

첫 리팩토링은 `world.rs`나 `battle/core`를 바로 쪼개기보다, 전투 노드 목적 정책을 통합하는 것이 안전하다.

이유:

- 기능 변화 없이 중복 설계를 줄일 수 있다.
- 현재 논의가 가장 많이 누적된 영역이다.
- `CombatNodeType`, `CombatMissionRisk`, reward policy, default tactical plan이 흩어져 있어 장기 부채가 빠르게 커질 수 있다.
- 이 경계를 정리하면 이후 `events/combat.rs`와 `combat_preview.rs` 분리가 쉬워진다.

추천 첫 작업:

```text
src/game/combat_mission_policy.rs 추가
-> CombatNodeType별 mission contract를 한 곳에 정의
-> intent/reward/risk/default objective lookup을 단계적으로 이동
-> 기존 함수는 thin wrapper로 남긴 뒤 호출부를 하나씩 교체
-> 테스트 통과 후 wrapper 제거 여부 판단
```

이 작업에서 기능을 새로 만들지는 않는다. 목적은 “현재 이미 구현된 정책을 한 곳에서 읽히게 만드는 것”이다.

현재 진행 상태:

- 시작됨: `src/game/combat_mission_policy.rs`를 추가해 live 지원 node type, 맵 노드 의도별 선호 node type, archetype fallback, reward tag identity, mission risk duration/cleanup 규칙을 단순 함수 모음으로 통합했다.
- 추가 진행됨: `CombatNodeIntentPolicy` thin wrapper와 모듈을 제거했다. 호출부는 `CombatMissionPolicy`를 직접 사용하므로 전투 목적 정책 source of truth가 줄었다.
- 추가 진행됨: `src/game/events/combat.rs`에 있던 `CombatNodeType`별 기본 tactical plan/win condition 생성 로직을 `CombatMissionPolicy`로 이동했다. 방어 오브젝트의 실제 스폰/프로필 생성은 아직 `CombatExecutor`가 담당한다.
- 추가 진행됨: 방어 오브젝트 스폰 그룹/프로필 생성 책임을 `src/game/combat_defense_object.rs`로 분리했다. 이는 새 builder 계층이 아니라 `DefenseObject` 주입만 담당하는 작은 helper 모듈이다.
- 추가 진행됨: 적 웨이브의 spawn cell 수집, enemy kind별 `BattleUnitDraft` 변환, deterministic enemy owned uuid 생성, `ScenarioSpawnGroup` 조립을 `src/game/combat_enemy_spawns.rs`로 분리했다. `CombatExecutor`는 변환된 enemy group을 scenario event에 연결하는 역할만 남겼다.
- 추가 진행됨: 직원 배치 입력을 player `ScenarioSpawnGroup`과 artifact list로 변환하는 책임을 `src/game/combat_player_spawns.rs`로 분리했다. 직원의 전투 가능 여부, 장비 강화, 장착 파편 기반 전투 프로필 조립은 플레이어 스폰 변환 helper가 담당하고, `CombatExecutor`는 scenario 연결과 실행에 집중한다.
- 추가 진행됨: 전투 보상 UUID 해석, forbidden reward 차단, node type별 `CombatRewardPolicy` 검증을 `src/game/combat_rewards.rs`로 분리했다. `CombatExecutor`는 encounter 조회와 node type 불일치 검증만 수행하고, 보상 계약 검증은 전용 helper가 담당한다.
- 추가 진행됨: `ScenarioSpawnGroup`을 `ScenarioEvent::SpawnGroup`과 함께 등록하는 반복 패턴을 `src/game/combat_scenario_groups.rs`로 분리했다. 새 시나리오 builder 계층은 만들지 않고, group/event 연결 규칙만 작은 helper로 모았다.
- 추가 진행됨: `CombatPreview`에서 전투 시작 battlefield spec을 만드는 `BattleStartPlan`, preview dimension 검증, static obstacle/spawn overlap 검증을 `src/game/combat_battlefield_plan.rs`로 분리했다. `CombatExecutor`는 battlefield 데이터 변환 규칙을 직접 들고 있지 않고 battle start plan을 받아 scenario에 연결한다.
- 추가 진행됨: `world.rs`에 남아 있던 production combat flow를 `src/game/world/combat.rs`로 순수 이동했다. 전투 미리보기/배치, 맵 전투 실행, 전투 리플레이 종료, 자동 전투 보상 지급, 전투 후 직원 상태 반영, 전투 불가 시 런 실패/차단 판정이 한 모듈에 모였다.
- 추가 진행됨: 수동 전투 보상 세션(`InCombatReward`, `InCombatRewardClaimed`, `ClaimCombatReward`, `ExitCombatReward`)은 제거했다. 현재 정책은 `FinishCombatReplay`에서 승리 보상을 자동 지급하고, node session이 있으면 즉시 노드 완료로 이어지는 흐름이다.
- 추가 진행됨: `PveBattleObjectiveData`와 `PveWinConditionData`를 당장 합치지는 않고, 둘 다 작성된 경우 명백히 다른 승리 계약이면 `PveEncounterDatabase::validate_indexes()`가 실패하도록 검증을 추가했다.
- 추가 진행됨: world reward/maintenance/research delivery 경로는 `SkillFragmentPolicy::default()`를 직접 만들지 않고 `GameCoreState.skill_fragment_policy`를 사용한다. 공개 inventory convenience 메서드의 default 호출은 테스트/독립 사용 편의용으로만 남는다.
- 추가 진행됨: 맵 노드 payload를 실제 shop/reward/support/HeadquartersContact 세션으로 라우팅하는 책임을 `src/game/world/map_content.rs`로 순수 이동했다. node seed, 안전 노드 연구 배송, 맵 shop/reward 해석, content node 진입 상태 전환이 한 모듈에 모였다. 기존 random event 래퍼는 live flow에서 제거했다.
- 추가 진행됨: 맵 조회, 노드 선택, recon 사용, 노드 진입 확정, 노드 완료/act 전환 흐름을 `src/game/world/node_flow.rs`로 순수 이동했다. `world.rs`는 action dispatch와 남은 도메인 helper를 보유하고, 노드 진행 state machine은 별도 모듈에서 추적한다.
- 추가 진행됨: 장비 장착/조합, 스킬 파편 장착/강화/각성/분쇄, 장비 복원/분쇄/강화 흐름을 `src/game/world/maintenance.rs`로 순수 이동했다. Maintenance 정책은 기존 validator와 action gate를 그대로 사용하며, 이번 단계에서는 정책 변경 없이 `world.rs`에서 정비 production 기능만 분리했다.
- 추가 진행됨: phase-era 이벤트 생성/선택/advance 흐름은 live core에서 제거했다. map node에 필요한 shop action과 reward claim/select만 `src/game/world/node_rewards.rs`로 재배치했다. 이 모듈은 phase fallback을 갖지 않으며, 노드 세션이 없는 종료 요청은 `InvalidAction`으로 거부한다.
- 검증됨: `cargo check -p game_core`, `cargo test -p game_core` 통과.
- 추가 진행됨: `DefendPoint` live/runtime 의존을 제거했다. 현재 방어형 전투의 source of truth는 `ProtectUnit`과 고정 방어 오브젝트이며, 지점 누수/runner 미션이 필요해지면 deprecated 계약을 되살리지 않고 새 명시적 계약으로 설계한다.
- 추가 진행됨: 구형 `RecoverAndExtract` 명칭과 계약은 제거하고, 최신 Recovery 정책은 `RecoverHoldAndExtract`로 통일했다. 현재 흐름은 `사전 배치 적 섬멸 -> 회수 체크포인트 진입 -> 일정 시간 사수 -> 다음 체크포인트/탈출 지점 이동`이다.

다음 작업의 완료 기준:

- `DefendPoint`가 live RON, `CombatMissionPolicy` 기본 계약, map combat startup, 일반 `BattleCore` 승패 판정 경로에 남아 있지 않음을 코드 근거로 확인한다.
- deprecated runtime branch와 이를 고정하던 legacy test를 삭제한다.
- `RecoverAndExtract` 구형 명칭이 source/test/live RON에 남아 있지 않고, Recovery node 문서와 코드 명칭이 `RecoverHoldAndExtract`로 일치하는지 확인한다.
- 변경 후 `cargo check -p game_core`와 관련 전투/월드 테스트를 통과시킨다.

## 완료 기준

리팩토링 준비 단계는 아래 조건을 만족하면 완료로 본다.

- 각 범주별 source of truth와 중복 후보가 문서화되어 있다.
- 첫 번째 실제 리팩토링 단위가 테스트 가능한 크기로 나뉘어 있다.
- 레거시 제거 후보가 live 경로 영향 여부와 함께 표시되어 있다.
- `cargo test -p game_core`로 현재 기준선을 확인할 수 있다.
- 다음 AI가 이 문서만 읽어도 어디부터 읽고 무엇을 고쳐야 하는지 알 수 있다.
