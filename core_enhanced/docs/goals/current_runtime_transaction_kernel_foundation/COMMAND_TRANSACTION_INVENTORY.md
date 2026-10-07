# Command Transaction Inventory

- 상태: `prepared / preliminary`
- 기준일: 2026-08-22
- 기준 표면: `src/game/behavior.rs`의 `ActionKind::ALL`
- 최종화 시점: PLAN Phase 1

## 1. 목적

이 문서는 모든 player action의 권위 상태, mutation 방식, target transaction class를 전수 관리한다. 특정 결함 일곱 개만 고치고 나머지 mutation을 암묵적으로 남기지 않기 위한 inventory다.

준비 시점의 분류는 감사와 현재 descriptor를 토대로 한 후보값이다. 실행 시 production code를 추적해 1 action 1 row로 확정한다. `preliminary` 또는 `unclassified`가 남아 있으면 Phase 1을 종료할 수 없다.

## 2. Transaction classes

| Class | 의미 | 현재 Goal 구현 범위 |
|---|---|---|
| `ReadOnly` | authority state를 변경하지 않는 query | 분류 및 non-mutation 검증 |
| `BattleMutation` | active battle authority와 projection을 변경 | 대상 여섯 명령 migration; 기존 tactical path는 positive control |
| `RunMutation` | non-battle run/session authority를 변경 | 분류만; migration은 원칙적으로 제외 |
| `ProfileMutation` | profile/persistent authority를 변경 | 분류만; migration 제외 |
| `FullSessionMutation` | 여러 root를 함께 초기화·전환 | `StartNewGame` migration |
| `ContextualMutation` | 현재 phase/context에 따라 authority가 달라지는 변경 | 실행 중 실제 root를 확인해 더 구체적으로 재분류 |
| `HostOwned` | host가 continuation/recovery authority를 소유 | core mutation 경계 확인만 |

`ContextualMutation`은 영구 target abstraction이 아니라 조사 표시다. Phase 1 종료 전 각 row에 실제 authority roots와 enforcement 방식을 기록해야 한다.

## 3. Required row schema

실행 시 각 action row는 최소 다음 정보를 가져야 한다.

- action name
- current `ActionRoute` / `ActionScope`
- battle authority requirement
- authoritative roots
- current write mechanism
- write 뒤 남는 fallible operation
- target transaction class
- migration disposition
- oracle/conformance test
- monotonic rejection exception
- evidence path

## 4. Preliminary exhaustive inventory

아래 표의 action 목록은 준비 당시 `ActionKind::ALL` 58개를 전수 옮긴 것이다. 실행 시작 시 코드와 exact-set 비교를 수행한다.

| # | ActionKind | Current route/scope | Candidate class | 현재 Goal disposition |
|---:|---|---|---|---|
| 1 | `StartNewGame` | CoreCommon / Gameplay | `FullSessionMutation` | **must migrate** (`REM-TX-003`) |
| 2 | `ContinueRun` | HostRecovery / Recovery | `HostOwned` | boundary control only |
| 3 | `ResetActiveRun` | HostRecovery / Recovery | `HostOwned` | boundary control only |
| 4 | `RetryPlayerProfile` | HostRecovery / Recovery | `HostOwned` | boundary control only |
| 5 | `ResetPlayerProfile` | HostRecovery / Recovery | `HostOwned` | boundary control only |
| 6 | `SelectStarterEmployees` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 7 | `UnEquipItem` | CoreDomain / Gameplay | `RunMutation` or `ProfileMutation` | confirm authority; classify only |
| 8 | `EquipItem` | CoreDomain / Gameplay | `RunMutation` or `ProfileMutation` | confirm authority; classify only |
| 9 | `UseConsumableItem` | CoreDomain / Gameplay | `RunMutation` | classify only |
| 10 | `EquipSkillFragment` | CoreDomain / Gameplay | `RunMutation` | classify only |
| 11 | `UnequipSkillFragment` | CoreDomain / Gameplay | `RunMutation` | classify only |
| 12 | `UpgradeSkillFragment` | CoreCommon / Gameplay | `RunMutation` or `ProfileMutation` | confirm authority; classify only |
| 13 | `AwakenSkillFragment` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 14 | `DismantleSkillFragment` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 15 | `DismantleEquipment` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 16 | `EnhanceEquipment` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 17 | `RequestMapData` | CoreCommon / Query | `ReadOnly` | classify + non-mutation control |
| 18 | `SelectMapNode` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 19 | `ConvertMapNodeWithMonolith` | CoreDomain / Gameplay | `RunMutation` | classify only |
| 20 | `ConfirmEnterNode` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 21 | `CancelSelectedNode` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 22 | `CompleteNode` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 23 | `LoadRunCheckpoint` | CoreCommon / Recovery | `FullSessionMutation` | classify only |
| 24 | `ConfirmRunFailure` | CoreCommon / Gameplay | `FullSessionMutation` or `ProfileMutation` | confirm roots; classify only |
| 25 | `AbandonRun` | CoreCommon / Gameplay | `FullSessionMutation` or `ProfileMutation` | confirm roots; classify only |
| 26 | `RecruitEmployee` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 27 | `RequestEmergencySupplies` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 28 | `OpenHeadquartersShop` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 29 | `SelectReward` | CoreCommon / Gameplay | `RunMutation` | existing staged pattern; positive control |
| 30 | `PurchaseItem` | CoreDomain / Gameplay | `RunMutation` | existing staged pattern; positive control |
| 31 | `SellItem` | CoreDomain / Gameplay | `RunMutation` | existing staged pattern; positive control |
| 32 | `RefreshShop` | CoreDomain / Gameplay | `RunMutation` | existing staged pattern; positive control |
| 33 | `ExitShop` | CoreDomain / Gameplay | `RunMutation` | classify only |
| 34 | `ClaimReward` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 35 | `ExitReward` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 36 | `AdvanceEventScene` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 37 | `ResolveEventChoiceCheck` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 38 | `AdvanceVisualNovel` | CoreContextual / Gameplay | `ContextualMutation` | resolve authority; classify only |
| 39 | `SelectVisualNovelChoice` | CoreContextual / Gameplay | `ContextualMutation` | resolve authority; classify only |
| 40 | `SweepMasteredElite` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 41 | `CompleteMasteredEliteSweep` | CoreCommon / Gameplay | `RunMutation` | classify only |
| 42 | `CompleteCombatResult` | CoreCommon / Gameplay | `ContextualMutation` | resolve battle/run handoff; classify only |
| 43 | `RequestBattleState` | CoreBattle / Query | `ReadOnly` | classify + non-mutation control |
| 44 | `RecoverBattleSetupLoss` | CoreBattle / Recovery | `BattleMutation` | trace recovery authority; classify only |
| 45 | `RequestDeploymentRangePreview` | CoreBattle / Query | `ReadOnly` | classify + non-mutation control |
| 46 | `DeployUnit` | CoreBattle / Gameplay | `BattleMutation` | existing staged tactical positive control |
| 47 | `WithdrawUnit` | CoreBattle / Gameplay | `BattleMutation` | existing staged tactical positive control |
| 48 | `SetEngagementStance` | CoreBattle / Gameplay | `BattleMutation` | existing staged tactical positive control |
| 49 | `TacticalRelocateUnit` | CoreBattle / Gameplay | `BattleMutation` | existing staged tactical positive control |
| 50 | `TacticalSwapUnits` | CoreBattle / Gameplay | `BattleMutation` | existing staged tactical positive control |
| 51 | `BeginSkillTargeting` | CoreBattle / Gameplay | `BattleMutation` | **must migrate** (`REM-TX-002`) |
| 52 | `CancelSkillTargeting` | CoreBattle / Gameplay | `BattleMutation` | **must migrate** (`REM-TX-002`) |
| 53 | `PreviewSkillTarget` | CoreBattle / Query | `ReadOnly` | classify + non-mutation control |
| 54 | `ActivateSkill` | CoreBattle / Gameplay | `BattleMutation` | **must migrate** (`REM-TX-002`) |
| 55 | `RetreatBattle` | CoreBattle / Gameplay | `BattleMutation` | trace current staged path; classify only |
| 56 | `PauseBattle` | CoreBattle / Gameplay | `BattleMutation` | **must migrate** (`REM-TX-002`) |
| 57 | `ResumeBattle` | CoreBattle / Gameplay | `BattleMutation` | **must migrate** (`REM-TX-002`) |
| 58 | `SetBattleSpeed` | CoreBattle / Gameplay | `BattleMutation` | **must migrate** (`REM-TX-002`) |

현재 route/scope는 준비 시점의 코드 판독 결과다. 실행 시 불일치가 있으면 code를 우선하고 이 표와 `SOURCE_LOG.md`에 차이를 기록한다.

## 5. Target migration detail

| Remediation | Actions | Known defect | Target boundary | Required proof |
|---|---|---|---|---|
| `REM-TX-002-A` | Pause, Resume, Speed | live session commit 뒤 cursor-dependent DTO 실패 가능 | staged battle session + staged projection + one publish | failure equality, retry, success revision/projection parity |
| `REM-TX-002-B` | Begin/Cancel targeting | focus state commit 뒤 projection 실패 가능 | staged focus mutation + projection + one publish | focus equality on failure, retry, success parity |
| `REM-TX-002-C` | ActivateSkill | skill/event mutation commit 뒤 projection 실패 가능 | staged activation/event generation + projection + one publish | battle state/event equality, determinism, retry |
| `REM-TX-003` | StartNewGame | multi-root mutation 뒤 RunStart trigger 실패 가능 | staged full-session construction + trigger/preflight + one publish | full authority equality, retry, save/new-game parity |

## 6. Positive and negative controls

### Positive controls

- staged tactical commands: deploy, withdraw, stance, relocate, swap
- staged reward/economy paths where audit found plan/preflight/commit separation

이들은 공통 불변식을 추출할 근거다. 단순히 구현을 복사하거나 하나의 trait로 강제 통합하지 않는다.

### Negative controls

- query commands must remain authority-neutral
- host recovery must not be silently pulled into core gameplay commit
- rejected shop overflow finding must not create speculative remediation

## 7. Phase 1 completion checklist

- [ ] production `ActionKind::ALL` count와 이 표 count 일치
- [ ] 누락/중복 action 없음
- [ ] 모든 row의 current route/scope가 code와 일치
- [ ] 모든 gameplay action에 authority roots 기록
- [ ] 모든 gameplay action에 target transaction class 확정
- [ ] 대상 일곱 action에 before/after fallible boundary 기록
- [ ] 대상 일곱 action에 oracle 및 rejection comparator 연결
- [ ] monotonic exception이 없으면 명시적으로 `none` 기록
