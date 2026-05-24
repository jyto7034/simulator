# 코어 시스템 Goal 계획

이 문서는 `using-codex-goals-effectively-ko.md`의 원칙을 이 프로젝트에 맞게 적용한 goal 문서다.

목표는 "게임 컨텐츠를 더 넣기"가 아니라, 현재 구현된 코어 시스템 흐름이 실제 플레이 루프를 끝까지 안전하게 지탱하도록 만드는 것이다. 이동 시스템, 전장 미세 조정, 밸런스 수치 조정, 신규 적/스킬/장비 컨텐츠 추가는 이 문서의 범위에서 제외한다.

## Goal 작성 기준

Goal mode에 적합한 작업은 아래 조건을 만족해야 한다.

- 종료 조건이 명확해야 한다.
- 빠르게 검증할 수 있는 테스트 또는 검색 명령이 있어야 한다.
- "더 좋은 구조"가 아니라 "현재 플레이 흐름을 더 안전하게 바꾸는 구조"여야 한다.
- 새 추상화는 실제 중복 또는 변경 지점이 확인된 경우에만 도입한다.
- 정책이 모호한 항목은 코드 수정 전에 사용자와 의논한다.
- 현재 공식 플레이 흐름에 없는 레거시 호환 경로는 감싸서 보존하지 않고 제거한다.
- 오래된 경로를 남기려면 현재 live RON 또는 공식 플레이 루프에서 실제로 사용된다는 근거가 있어야 한다.

## 현재 판단

코어 시스템의 큰 축은 이미 연결되어 있다.

- 시작 직원 선택 후 런 맵 생성.
- 노드 선택과 `NodeConfirm`.
- 전투 노드별 `CombatPreview`, 배치, 전투 시작.
- `BattleScenario` 기반 전투 실행과 리플레이.
- 직원 Run HP, 트라우마, 사망, 경험치 일부 반영.
- 보상 자동 지급, 연구 진행도, pending 연구 수령.
- Support, Shop, Reward, Maintenance 노드 진입과 완료.
- 스킬 파편 장착, 강화, 개화, 분쇄.
- 장비 구매, 장착, 분해, 복원, 강화.

하지만 "한 런을 실제 플레이 흐름으로 완전히 닫는다"는 기준에서는 아래 시스템 갭이 남아 있다.

## G1. 전투 실패 후 회복 경로 기반 런 실패 판정

진행 상태: 완료.

구현 근거:

- `src/game/world/combat.rs`
  - 비보스 전투 실패 시 `NoDeployableEmployees`만 노드 완료 후 다시 런 실패를 판정한다.
  - `NoLivingEmployees`, `CombatTeamUnavailable`, `BossDefeated`는 회복 노드 예외 없이 즉시 런 실패를 유지한다.
- `src/game/world/tests.rs`
  - `failed_combat_opens_outgoing_medical_before_no_deployable_run_failure`
  - `failed_combat_without_outgoing_medical_fails_when_no_deployable_employee_remains`

### 해결한 문제

문서 정책은 "출전 가능한 직원이 없어도 다음 선택 가능한 노드 중 Medical 같은 회복 가능 노드가 있으면 런은 지속된다"이다. 이전 전투 리플레이 종료 흐름은 실패한 노드를 완료해 다음 노드를 열기 전에 `fail_run_if_needed()`를 먼저 호출했다.

이전 코드 흐름:

```text
FinishCombatReplay
-> apply_post_battle_resolution
-> boss 패배 검사
-> fail_run_if_needed
-> 실패 전투면 handle_complete_node
```

이 구조에서는 실패한 전투의 outgoing 노드에 Medical이 있더라도, 아직 `available_node_ids`가 갱신되지 않았기 때문에 회복 경로로 인정되지 않을 수 있었다.

### 코드 근거

- `src/game/world/combat.rs`
  - `handle_finish_combat_replay()`는 이제 비보스 실패 전투의 `NoDeployableEmployees` 판정에서 `handle_complete_node()`를 먼저 호출한다.
  - `has_available_recovery_map_node()`는 현재 `map_progression.available_node_ids`만 본다.
- `src/game/map/progression.rs`
  - `complete_current_node()`가 호출되어야 outgoing 노드가 `Available`이 된다.

### 확정 정책

- 비보스 전투 실패는 먼저 노드를 소비하고 다음 노드를 연다.
- 그 다음 열린 노드 중 Medical 가능성이 있으면 `ViewingMap`으로 돌아가 런을 유지한다.
- 열린 노드 중 Medical 가능성이 없으면 `RunFailed(NoDeployableEmployees)`가 된다.
- 보스 패배는 이 예외를 보지 않고 즉시 `RunFailed(BossDefeated)`로 유지한다.

### 종료 조건

- 실패한 비보스 전투 후 전원 출전 불가가 되어도, 완료 후 열린 노드 중 Medical이 있으면 `ViewingMap`으로 돌아온다.
- 같은 상황에서 열린 노드 중 Medical이 없으면 `RunFailed(NoDeployableEmployees)`가 된다.
- 보스 패배는 회복 노드 존재 여부와 무관하게 즉시 런 실패다.

### 빠른 검증

```bash
cargo test -p game_core game::world::tests::combat::
```

추가된 대표 테스트:

- `failed_combat_opens_outgoing_medical_before_no_deployable_run_failure`
- `failed_combat_without_outgoing_medical_fails_when_no_deployable_employee_remains`

## G2. 노드 결과 요약 계약

진행 상태: 완료.

구현 근거:

- `src/game/behavior.rs`
  - `NodeOutcomeSummary`, `CombatOutcomeSummary`, `NodeOutcomeEmployeeChange`를 추가했다.
  - 전투 성공은 `CombatRewardsGranted.outcome`으로 결과 요약을 반환한다.
  - 전투 실패 후 맵으로 돌아오는 경우는 `NodeCompleted.outcome`으로 결과 요약을 반환한다.
  - 전투 실패가 곧바로 런 실패로 이어지는 경우는 `RunFailed.outcome`으로 결과 요약을 반환한다.
- `src/game/world/combat.rs`
  - 전투 후 직원의 Run HP, 트라우마, 생존/전투불능 변화를 before/after DTO로 구성한다.
  - 전투 성공/실패 모두 `mission_success`, `node_type`, `winner`, 직원 변화, 인벤토리 diff, 연구 수령 목록을 같은 요약 구조로 반환한다.
- `src/game/world/tests.rs`
  - 성공 전투, 회복 경로가 열린 실패 전투, 회복 경로가 없는 실패 전투, 보스 실패 전투에서 outcome이 반환되는지 확인한다.

### 해결한 문제

룰북은 노드 종료 후 결과창에서 임무 성공/실패, 직원 상태 변화, 보상, 연구 진행도, pending 연구 완료 항목을 보여줘야 한다고 적고 있다. 그러나 현재 전투 성공은 `CombatRewardsGranted { completion: NodeCompleted }`로 바로 맵 완료까지 진행하고, 전투 실패는 대부분 `NodeCompleted`만 반환한다.

이 구조는 코어 내부 진행에는 충분하지만, 클라이언트가 "이번 노드에서 무엇이 일어났는지"를 단일 DTO로 받기 어려웠다. 결과창 UI가 이미 방향성을 갖고 있으므로, 코어도 결과 요약 계약을 명확히 가져야 했다.

### 코드 근거

- `src/game/behavior.rs`
  - `BehaviorResult::CombatRewardsGranted`는 보상과 노드 완료를 묶고, 이제 `outcome`으로 임무 성공/실패/직원 변화 요약도 반환한다.
  - `BehaviorResult::NodeCompleted`는 전투 실패 후 맵으로 돌아오는 경우 `outcome`을 포함할 수 있다.
  - `BehaviorResult::RunFailed`는 전투 결과로 런이 실패한 경우 `outcome`을 포함할 수 있다.
- `src/game/world/combat.rs`
  - 실패 전투는 보상을 생략하되, 결과 요약을 만든 뒤 `handle_complete_node()` 또는 `RunFailed` 결과에 붙인다.
- `docs/game_rulebook.md`
  - "노드 결과 처리"는 결과창에 필요한 정보를 명시한다.

### 확정 정책

- 새 `GameState::NodeResult`는 당장 만들지 않는다.
- 우선 `BehaviorResult`에 `NodeOutcomeSummary`를 포함해 클라이언트가 결과창을 만들 수 있게 한다.
- 결과 확인용 별도 액션은 클라이언트가 실제로 필요하다고 판단될 때만 추가한다.

### 종료 조건

- 전투 성공/실패 모두 `mission_success`, `node_type`, `winner`, `employee_changes`, `inventory_diff`, `research_deliveries`, `completion`을 한 번에 확인할 수 있다.
- 실패 전투도 "왜 보상이 없거나 줄었는지"를 DTO로 설명할 수 있다.
- 기존 자동 진행 정책은 유지한다.

### 빠른 검증

```bash
cargo test -p game_core game::world::tests::combat::
cargo test -p game_core game::world::tests::map_flow::
```

## G3. RewardEffect::GrantExperience 실제 적용

진행 상태: 완료.

구현 근거:

- `src/game/world/combat.rs`
  - 전투 보상에 포함된 `GrantExperience` 총량을 계산한다.
  - 해당 전투에 출전했고 생존했으며 전투불능이 아닌 직원에게 경험치를 균등 분배한다.
  - Reward/Support/HeadquartersContact/Shop 같은 비전투 노드에서는 경험치 보상을 지급하지 않는다. 기존 Event 노드는 제거되었으며, 추후 새 RandomEvent를 만들 때 성장 보상 여부를 별도로 결정한다.
- `src/game/behavior.rs`
  - `NodeOutcomeEmployeeChange`에 `experience_before`, `experience_after`를 추가했다.
- `src/game/world/tests.rs`
  - `combat_experience_reward_applies_to_surviving_participants_and_outcome`으로 전투 보상 경험치와 결과 요약을 검증한다.
- `src/game/reward.rs`
  - `GrantExperience`는 더 이상 "미구현 placeholder"가 아니라, 호출자가 대상 정책에 맞게 적용해야 하는 효과로 기록한다.

### 해결한 문제

보상 시스템에는 `GrantExperience`가 있지만 이전에는 로그만 남기고 직원에게 적용하지 않았다. 룰북은 실패 노드에서도 직원 성장 요소 일부를 줄 수 있다고 말한다. 현재 생존 XP는 전투 후처리로 지급되고, 보상 데이터의 경험치 효과는 전투 보상 경로에 연결했다.

### 코드 근거

- `src/game/reward.rs`
  - `RewardEffect::GrantExperience`는 호출자가 대상 정책에 맞게 적용해야 하는 효과로 기록된다.
- `src/game/employee.rs`
  - `Employee::add_experience()`와 레벨/등급 성장 로직은 이미 존재한다.
- `docs/game_rulebook.md`
  - 실패 노드는 핵심 보상은 주지 않더라도 직원 경험치 같은 성장요소 일부는 지급할 수 있다고 되어 있다.

### 적용 정책

- 전투 보상 경험치는 해당 전투에 출전했고 생존한 직원에게 균등 지급한다.
- 전투불능 직원은 후처리 trauma/run hp 페널티를 받으므로 같은 경험치 보상을 받지 않는다.
- Reward/Support/HeadquartersContact/Shop 노드의 경험치 보상은 사용하지 않는다. 직원 경험치는 전투 참여/생존/후처리 중심으로만 증가한다. 기존 Event 노드는 Shop/Reward 래퍼라 제거되었고, 추후 RandomEvent를 새로 만들 때 별도 정책으로 결정한다.

### 종료 조건

- `RewardEffect::GrantExperience`가 더 이상 placeholder가 아니다.
- 전투 보상으로 경험치가 지급되는 대상이 명확하다.
- `BehaviorResult` 또는 결과 요약에서 경험치 변화가 확인 가능하다.

### 빠른 검증

```bash
cargo test -p game_core game::world::tests::combat::
cargo test -p game_core game::reward
```

## G4. NodeConfirm 준비 행동의 source of truth 정리

진행 상태: 완료.

정책 변경:

- 클라이언트도 새 흐름으로 재작성 중이므로 구형 Field 기반 배치 호환을 유지하지 않는다.
- 런 전체에 유지되는 TFT식 Field 배치는 공식 플레이 흐름에서 제거한다.
- `MoveUnit`은 전투/Boss `NodeConfirm` 상태에서 선택한 노드의 `CombatDeployment`를 수정할 때만 유효하다.
- `TransferUnit`과 `ZoneType`은 구형 Field <-> Inventory 이동 계약이므로 제거한다.
- 로스터 정리는 `MoveBenchUnit`만 사용한다.

### 문제

전투 노드에서는 `CombatDeployment`가 전투 배치의 source of truth다. 이전에는 `ViewingMap`에서도 구형 메타게임 `Field` 기반 `MoveUnit`/`TransferUnit` 흐름이 노출되어 "런 공용 필드 배치"와 "노드별 전투 배치"가 함께 보였다. 이 혼동을 없애기 위해 공식 액션 계약에서 구형 Field 배치 의미를 제거했다.

### 코드 근거

- `src/game/world.rs`
  - `handle_move_unit()`은 `NodeConfirm` 전투/Boss 상태에서만 `CombatDeployment`를 수정한다.
  - 그 외 상태의 `MoveUnit`은 `InvalidAction`이다.
- `src/game/world/combat.rs`
  - 전투 시작은 `run.combat_deployments[node_id]`만 사용한다.
- `src/game/managers/action_scheduler.rs`
  - `ViewingMap`에서는 `MoveUnit`/`TransferUnit`을 노출하지 않는다.
- `src/game/world/snapshot.rs`
  - 직원 로스터 스냅샷에서 구형 `field_position`을 노출하지 않는다.

### 확정 정책

- 전투 배치는 노드별 `CombatDeployment`가 유일한 source of truth다.
- `MoveUnit`은 전투 노드 준비 상태에서만 배치를 의미한다.
- 맵 화면에서 직원 순서를 바꿔야 할 때는 `MoveBenchUnit`을 사용한다.
- 구형 Field 기반 배치 호환 API와 관련 테스트는 유지하지 않는다.

### 종료 조건

- `CombatDeployment`가 전투 배치의 유일한 source of truth임이 코드와 테스트에서 명확하다.
- 구형 `Field` 배치는 player-facing 액션과 로스터 스냅샷에 노출되지 않는다.
- 클라이언트가 호출할 액션 이름에서 노드별 배치와 벤치 정리가 혼동되지 않는다.

### 빠른 검증

```bash
cargo test -p game_core game::world::tests::combat::combat_map_node_uses_explicit_node_deployment_instead_of_legacy_field
cargo test -p game_core game::world::tests::placement::
```

## G5. 코어 정책 수치의 설정화

진행 상태: 완료.

구현 근거:

- `src/game/world.rs`
  - `RunSystemPolicy`와 `RUN_SYSTEM_POLICY`를 추가해 런 시스템 수치를 한 곳으로 모았다.
  - 시작 직원 수, 최대 Act 수, 시작 Enkephalin, 전투 생존 XP, 전투불능 트라우마/Run HP 손실률, Medical/Rest 회복량을 같은 정책 구조에서 읽는다.
- `src/game/world/combat.rs`
  - 전투 후 생존 XP와 전투불능 페널티가 `RUN_SYSTEM_POLICY`를 참조한다.
- `src/game/world/support.rs`
  - Medical/Rest 회복량이 `RUN_SYSTEM_POLICY`를 참조한다.
- `src/game/world/helpers.rs`, `src/game/world/snapshot.rs`, `src/game/world/tests.rs`
  - 시작 직원 수와 최대 Act 수 검증도 같은 정책 구조를 기준으로 한다.

### 해결한 문제

밸런스 조정은 나중에 하더라도, 핵심 시스템 수치가 코드 상수에 흩어져 있으면 클라이언트 완성 후 반복 조정이 어렵다. 현재 일부 런 정책 수치는 `GameCore`와 `support.rs`의 상수로 박혀 있다.

예시:

- 시작 직원 수.
- 최대 Act 수.
- 전투 생존 XP.
- 전투불능 트라우마.
- 전투불능 Run HP 감소율.
- Medical/Rest 회복량.
- 시작 Enkephalin.

### 코드 근거

- `src/game/world.rs`
  - `RunSystemPolicy`가 시작 직원 수, 최대 Act 수, 전투 생존 XP, 전투불능 트라우마/Run HP 손실률, Medical/Rest 회복량, 시작 Enkephalin의 source of truth다.
- `src/game/world/support.rs`
  - 지원 노드 회복량은 `RUN_SYSTEM_POLICY`를 참조한다.

### 적용 정책

- 콘텐츠 데이터가 아니라 런 시스템 정책이므로 전용 `RunSystemPolicy`로 묶었다.
- TOML/RON 로더 확장은 아직 하지 않는다. 현재 단계에서는 과도한 설정 계층보다 source of truth 축소가 우선이다.

### 종료 조건

- 위 수치들이 한 곳에서 읽힌다.
- 기존 테스트 결과가 변하지 않는다.
- 테스트에서는 기본 정책과 일부 override 정책을 빠르게 검증할 수 있다.

### 빠른 검증

```bash
cargo test -p game_core game::world::tests::support::
cargo test -p game_core game::world::tests::combat::
```

## G6. 공식 세로 슬라이스 테스트

진행 상태: 완료.

구현 근거:

- `src/game/world/tests.rs`
  - `system_flow::official_run_slice_covers_starter_selection_combat_result_and_safe_node_delivery`를 추가했다.
  - 테스트 내부에서 전투 노드와 다음 안전 노드를 고정하므로 live RON 전체 밸런스에 의존하지 않는다.
  - 시작 직원 선택, 전투 노드 선택, `CombatDeployment` 기반 `MoveUnit`, 전투 시작, `FinishCombatReplay`, 안전 노드 진입, 연구 수령, 다음 맵 진행 확인을 하나의 흐름으로 검증한다.

### 해결한 문제

현재 테스트는 기능별로 촘촘하지만, "시작 직원 선택 -> 노드맵 -> 전투 전 배치 -> 전투 -> 결과 처리 -> 안전 노드 수령/정비 -> 보스/런 종료"를 하나의 공식 루프로 고정하는 세로 슬라이스 테스트가 부족하다.

이 테스트는 게임 재미나 밸런스를 검증하는 테스트가 아니라, 코어 시스템 흐름이 끊기지 않는지 확인하는 빠른 피드백 루프다.

### 코드 근거

- `src/game/world/tests.rs`
  - map flow, combat, support, equipment, maintenance 테스트는 존재한다.
  - 전체 런을 한 번 관통하는 테스트는 제한적이며, 보상/연구/정비/전투 후 실패 정책을 하나로 엮는 계약은 약하다.
- `docs/gameplay_flow_example.md`
  - 플레이 기록 예시는 있으나 대응되는 공식 테스트가 없다.

### 종료 조건

- 최소 1개의 deterministic 세로 슬라이스 테스트가 있다.
- 테스트는 live RON 전체 밸런스에 과하게 의존하지 않고, 필요한 노드/조우를 명시적으로 고정한다.
- 아래 흐름을 포함한다.

```text
StartNewGame
-> SelectStarterEmployees
-> Select combat node
-> MoveUnit in CombatDeployment
-> ConfirmEnterNode
-> FinishCombatReplay
-> safe node entry 또는 Maintenance
-> research delivery 또는 maintenance action
-> next node unlock 확인
```

### 빠른 검증

```bash
cargo test -p game_core game::world::tests::system_flow::
```

## G7. 문서와 코드의 용어 정렬

진행 상태: 완료.

구현 근거:

- 구 보상 세션 상태명을 `InReward`, `InRewardClaimed`로 변경했다.
- 구 보상 세션 행동명을 `ClaimReward`, `ExitReward`로 변경했다.
- 구 보상 수령 결과명을 `RewardGranted`로 변경했다.
- 구 보상 세션 내부 행동명을 `RewardAction`으로 변경했다.
- 기존 랜덤 이벤트의 보상 타겟 래퍼는 제거되었다. 추후 랜덤 이벤트가 필요하면 선택형 `RandomEvent` 노드로 새로 작성한다.
- 전투 효과의 `BonusDamage`처럼 보상 세션과 무관한 일반 게임 용어는 변경하지 않는다.

### 문제

코어에서 과거 `Bonus` 용어가 client-facing 결과와 상태에 남아 있었다. 현재 룰북은 Reward 노드와 자동 보상 중심으로 정리되어 있으므로, 클라이언트 프로토콜에서 `Reward`로 통일했다.

### 코드 근거

- `src/game/resources/state.rs`
  - `GameState::InReward`, `InRewardClaimed`.
- `src/game/behavior.rs`
  - `ClaimReward`, `ExitReward`, `RewardGranted`.
- `src/game/world/snapshot.rs`
  - 스냅샷 상태 문자열은 `in_reward`, `in_reward_claimed`를 사용한다.

### 확정 정책

- client-facing 상태/행동/결과에서는 `Reward`를 사용한다.
- `BonusDamage` 같은 전투 수치 용어는 보상 세션 의미가 아니므로 유지한다.

### 종료 조건

- client-facing 상태/행동/결과에서 Reward와 Bonus가 섞이지 않는다.
- live Reward 노드와 전투 보상 결과가 같은 용어 체계를 사용한다.
- 기존 보상 수령 흐름 테스트가 새 이름으로 통과한다.

### 빠른 검증

```bash
cargo test -p game_core game::world::tests::node_sessions::
rg -n "ClaimReward|ExitReward|RewardGranted|InReward|InRewardClaimed|NotInRewardState" src tests docs
```

## 우선순위

1. `G1`: 런 실패 판정 시점. 실제 플레이 흐름을 잘못 끝낼 수 있으므로 최우선이다.
2. `G2`: 노드 결과 요약 계약. 클라이언트 결과창과 코어 흐름을 연결하는 데 필요하다.
3. `G6`: 공식 세로 슬라이스 테스트. 이후 변경의 빠른 피드백 루프다.
4. `G3`: 경험치 보상 실제 적용. 성장 루프를 닫는다.
5. `G5`: 정책 수치 설정화. 밸런싱 전 준비 작업이다.
6. `G4`: 배치 용어/source of truth 정리 완료. `MoveUnit`은 노드별 전투 배치 전용이며 구형 Field 기반 배치 계약은 제거했다.
7. `G7`: Bonus/Reward 용어 정리 완료. client-facing 보상 세션 용어는 Reward로 통일했다.

## 레거시 제거 원칙

레거시는 과감하게 제거한다. 이 프로젝트에서 레거시 제거는 "예전 코드를 안전하게 감싸는 것"이 아니라, 현재 노드맵 기반 공식 플레이 흐름에 필요 없는 경로를 런타임에서 없애는 것을 뜻한다.

- phase-era 전투, 낡은 덱/이벤트 풀, 과거 보너스/필드 호환 DTO처럼 공식 흐름에서 쓰지 않는 코드는 제거 대상이다.
- live RON 또는 클라이언트 이관 때문에 임시 호환이 꼭 필요하면, 제거 조건과 소유자를 문서에 남긴다.
- 테스트는 레거시 동작을 고정하지 않는다. 현재 게임 흐름, 보상, 런 실패, 노드 소비 계약만 고정한다.
- RON 원본을 참고용으로 남길 필요가 있으면 파일명에 `legacy`를 붙여 보관할 수 있지만, 런타임 코드는 읽지 않아야 한다.

## 확정된 정책 결정

- 전투 실패 후 전원 출전 불가 판정은 비보스 노드를 먼저 소비하고 outgoing 회복 경로를 연 뒤 수행한다.
- 노드 결과창은 별도 코어 상태보다 `BehaviorResult` 결과 요약 DTO를 우선한다.
- 레거시는 compatibility layer로 감싸지 않는다. 현재 공식 플레이 흐름에서 쓰지 않는 경로, 테스트, DTO, 문서 문구는 제거한다.
