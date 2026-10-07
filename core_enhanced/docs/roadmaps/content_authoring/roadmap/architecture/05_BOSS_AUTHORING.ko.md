# A05 — 커뮤니티 보스 제작 규격과 런타임 연결

**보스 제작은 이번 로드맵의 핵심 경로다.** 아래는 권장 구조와 계약이며 기존 코드의 지원 목록이나 이미 승인된 최종 runtime schema가 아니다. CA16 구현 AI는 관련 현재 코드를 읽고 상세 설계를 완성한다. RON 조합기 UI는 만들지 않지만 조합기가 표시할 작성 단위·입출력·오류·실행 추적을 제공한다.

## 1. 제어 계층: 기존 계층형 상태·작업 모델을 확장

```text
EncounterDefinition
 ├─ ActorSlot bindings ── scenario의 wave/spawn identity
 ├─ Phase graph ── 여러 actor의 상위 전이 조정
 ├─ Blackboard declarations ── encounter 공유 사실/카운터
 └─ PhaseActorBehavior
      └─ Behavior graph
           ├─ Resident State (지속 작업과 전이)
           └─ PatternCall + Selector
                └─ Pattern (typed parameters, outcomes)
                     └─ State (entry 작업, 동시 tasks, 결과 전이)
                          └─ TaskRequest → DomainActionHandle
```

Pattern은 재사용 가능한 작업 묶음이다. Behavior는 무엇을 언제 선택하는지, Phase는 전투의 상위 형태·다중 actor 전환을 소유한다. 스킬 피해를 보스 graph에 복사해 넣지 않는다. 보스 Task에서 검증된 스킬·효과 프로그램을 호출한다.

이 계층을 그대로 새 타입명으로 전면 복제하지 않는다. 현재 `BossRuntimeCatalogData`, compiled pattern/state/task, activation ID가 이미 책임을 수행하면 확장한다. 이번에 재확인한 raw task 목록은 `data/boss_runtime_data.rs:164–230`; 기존 Task의 종료·권한은 구현 AI가 compiler/runtime에서 다시 확인한다.

## 2. 제작자가 선택할 보스 전용 구성 요소

| 종류 | 작성 내용 | engine와 연결되는 의미 |
| --- | --- | --- |
| Phase | 진입/종료, HP·사망·사건 조건, 전이 시점 | actor identity 보존, 상위 중재 |
| Selector | 고정 순서, 조건 우선, 가중 eligible | eligibility→쿨다운→선택·기록 |
| Target query | 적/아군/오브젝트, 필터, 정렬, lock/re-evaluate | 공통 query의 보스 consumer 허용 범위 |
| Movement task | 경로 전투, 추격, 복귀, 방향, 필요한 지정 위치 | 기존 movement/path/collision authority |
| Cast/Invoke task | skill/effect 참조, target/aim, wait milestone, 실패 경로 | ActionHandle과 progress/result |
| State effects | profile overlay, targetability, damage gate, aura | source lease와 종료 처리 |
| Counter/timer | HP·피해·공격 연속·작업 사실을 집계 | typed blackboard/resource + committed event |
| Spawn/object | 소환 정의·위치·batch·수명·실패 정책 | 공통 생성 경계, actor/field object 구분 |
| Scenario interaction | 작업 완료, 목표 도달, 소환 batch 정리 | typed ScenarioSignal·binding |
| Telegraph | 준비 시점·방향·대상 위치·예고 footprint | core 사실/preview 제공; 실제 렌더링은 비범위 |

광선·돌진·반복 폭격 같은 패턴은 가능하면 공통 flow/전달/이동의 조합으로 작성한다. `HelloTask`, `PorccubusSuppressTask` 같은 캐릭터별 enum을 추가하는 방식으로 해결하지 않는다.

## 3. BossTask → ActionService 포트

다음은 **권장 인터페이스 의사코드**다. 타입 정의와 위치는 CA16-D에서 결정한다.

```text
StartAction {
  operation_ref: compiled skill / action ref,
  actor: UnitInstanceId,
  input: TargetAndAim,
  source: SourceContext,
  owner_scope: BossStateActivationId,
  completion: Released | FlowFinished | EffectsSettled,
  on_state_exit: CancelOwnedWork | DetachReleased,
}
 → Rejected(Reason) | Accepted(ActionHandle)

ActionProgress(handle, milestone, receipt)
ActionTerminal(handle, Completed(summary) | Failed(reason) | Canceled(reason))
```

`Accepted`는 요청 응답이지 기본 completion이 아니다. 각 Task는 선택한 milestone에 도달했을 때 `Completed` 결과를 한 번 얻는다. 예를 들어 사격 후 이동은 Released까지 기다리면 되고, 피해 결과를 보고 다음 행동을 고르려면 EffectsSettled 또는 개별 result port를 기다린다. 지속 aura·무기한 소환까지 포함해 항상 EffectsSettled를 기다리면 종료되지 않을 수 있으므로 finite 검사·timeout 또는 별도 수명 소유가 필요하다.

Failure 결과는 모든 domain error를 문자열로 뭉개지 않는다. TargetUnavailable, ActionBlocked, NoSpawnSpace 등 해당 task의 닫힌 result schema를 제공한다. 실행 예산 초과나 내부 invariant 파손은 평범한 “다른 패턴 선택”으로 숨기는 게임상 실패가 아니라 상위 runtime fault다.

보스 작업의 명시 시전과 ordinary autocast/basic attack은 시작 권한을 구분한다. 현재 RouteCombat gate가 보스 제어 state의 시전까지 막는지 조사한다. 새 task는 현재 actor/state가 소유한 combat capability·ActionLease·시전 자격을 검증한 요청으로 들어가야 한다. gate를 전역 해제하거나 `invoke_ability`를 직접 불러 모든 일반 차단 조건을 우회하는 식으로 구현하지 않는다.

## 4. 런타임 상태의 권장 shape

| 상태 | owner / 주요 내용 |
| --- | --- |
| EncounterRuntime | encounter activation, phase, actor bindings, encounter-scoped blackboard |
| ActorBehaviorRuntime | unit identity, behavior/node, route cursor, actor-scoped cooldowns |
| PatternActivation | pattern instance, typed argument frame, local counters, outcome |
| StateActivation | state generation, task handles, leases, subscriptions, event memory |
| TaskRuntime | request ID, ActionHandle, expected milestone, absolute battle deadline, result consumption |
| BlackboardValue | declared key, scalar/ref type, scope, writer 권한, reset rule |

Blackboard는 자유로운 global string→any 저장소가 아니다. 정의 시 type·scope·writers가 검증되고 런타임에는 state index로 접근한다. 임계치 카운터와 타이머는 공통 Resource/State store를 재사용할 수 있지만 boss가 world inventory를 직접 수정하는 통로로 만들지 않는다.

## 5. 상태 진입·전환·취소 알고리즘

1. 정해진 battle boundary에서 committed facts를 읽는다. 아직 피해 해결이 끝나지 않은 speculative HP를 phase 조건으로 쓰지 않는다.
2. terminal/death 제약과 phase 전이 후보를 중재하고, 해당 actor의 control/pattern 전이를 계산한다. 같은 state의 고유 priority와 tie 규칙은 compile에서 검증한다.
3. leave 계획에 owner task 취소·subscription 제거·state modifier/lease 정리를 기록한다. 이미 release된 독립 전달은 자신의 수명을 따른다.
4. 새 state generation, entry tasks, typed parameters와 deadlines를 stage한다. capability 충돌·비용·대상 자격을 검증한다.
5. 승인된 transaction boundary에서 일괄 commit한다. 내부 이벤트·RNG·cooldown도 같은 확정과 함께 변경한다.
6. 이전 state activation으로 온 늦은 완료·timeout은 소비하지 않는다. 새 state에 동일 task ID가 있어도 activation이 다르면 무효다.

한 boundary에서 즉시 전이가 여러 번 발생할 수 있으므로 정적 zero-time cycle 검증과 runtime microstep budget을 함께 둔다. `AfterCurrentState/AfterCurrentPattern`은 어떤 actor들의 어떤 완료를 기다리는지와 waiting actor의 새 작업 금지를 명시한다. 정확한 전역 order는 기존 계약과 PG03의 조사 결과로 확정한다.

## 6. 동시 작업과 selector의 명시 규칙

Task descriptor는 movement/combat_action/target_lock 등 사용하는 capability와 공유 여부를 선언한다. 조합기에 “움직이기와 공격하기”를 함께 표시했다고 실제 동시 사용이 성립하는 것은 아니다. 이동하며 공격하는 행동은 그 허용이 선언된 action/capability 조합으로만 가능하다.

동시 task state에는 완료·실패·취소 시 다른 task를 어떻게 처리하는지 명시한다. Any/All join과 실패 시 cancel siblings는 서로 다른 의미다. finite task의 각 정상 result에는 transition 또는 명시적 catch-all route가 있어야 한다. runtime fault를 catch-all 정상 route로 삼지 않는다.

Priority/Weighted selection은 이름만 enum에 두지 않는다. eligible 조건·cooldown scope·가중치 양수·RNG identity·no-eligible route를 함께 구현한다. 선택 candidate를 검토했다는 이유만으로 cooldown을 소비하지 않으며 admission commit 때 소비하는 기존 방향을 우선 재사용한다. exact 결과는 compiler/runtime 조사로 확정한다.

## 7. phase와 외형/스탯 형태는 분리한다

Phase 전이는 다른 unit을 죽이고 새로 소환하는 것과 다르다. 아무 것도 없는은 연속 HP와 actor identity를 유지한다. 형태 교체는 해당 actor의 profile overlay/action set/표현 상태를 바꾸고, HP 보존 방식은 원문 계약을 따른다.

변신 중에도 피해를 받고, 변신 완료 시 현재 HP를 다시 평가해 도달할 형태를 고른다. 중간 threshold 초과분을 잘라내거나 중간 형태를 강제 유지하지 않는다. 여러 threshold가 한 번에 참일 때 우선순위·timeout·사망이 겹치는 정책은 DG와 PG03에서 표로 확정한다. 보스에 HP floor나 immunity를 붙이는 것은 별도 authored 효과이며 phase 문법의 암묵 기본값이 아니다.

## 8. 전장 기믹과의 연결

FieldWork는 작업 진행을 소유하고 완료 시 `ScenarioSignal`을 발생시킨다. boss의 구독은 신호 kind·object/actor binding·source와 event memory(Pulse/UntilStateExit 등)를 명시한다. 보스는 이를 받아 억제 state로 전이하거나 자신의 aura를 중단한다.

WaveRuntime은 actor HP/death·group clear·지점 등 사실을 읽고 wave를 활성화한다. 보스가 임의로 spawn했다고 required wave가 완료되지 않는다. 소환 batch 정리와 actor death는 구분하고, 단순 despawn이 kill로 인정되지 않게 한다. 보상·도감·승리 권한은 Scenario의 명시 바인딩과 결과 정책에 남긴다.

여러 보스 또는 부속 actor의 공유 기믹은 Encounter scope의 typed signal/counter로 연결할 수 있게 한다. 한 actor의 state-local 값에 다른 actor가 임의 접근하는 방식은 쓰지 않는다. CA00의 공식 원문에서 필요한 multi-actor 결합 범위를 확인해 구현/검증에 배정한다.

## 9. 보스 제작 검증과 추적

정적 검증은 missing entry/target/outcome, task result 미처리, illegal parameter/query, capability 충돌, 종료 보장 없는 wait, zero-time loop, recursive summon 위험, signal 미바인딩, phase/wave dependency를 포함한다.

런타임 설명에는 phase/state/pattern/task activation, 선택된 candidate와 제외 사유, condition 입력, action handle·milestone, 전이 원인·취소 범위, source definition 위치를 제공한다. 이것은 내부 mutable graph를 클라이언트 truth로 공개하라는 뜻이 아니라 도구용 read-only trace다.

정지 보스의 첫 방향성 공격, 스킬을 발사한 직후 phase 전환, wait 중 대상 사망, signal/timeout 동시 발생, 여러 actor phase wait, summon 실패 후 fallback과 stale completion을 필수 시험으로 둔다.

## 10. CA16 내부 실행 묶음

| 묶음 | 구현·설계 계약 | 인수 증거 |
| --- | --- | --- |
| CA16.1 | 작성 graph·task port·result schema·descriptor와 compiler | 잘못된 graph/result/wait 거부 |
| CA16.2 | Cast/Invoke·handle·milestone·cancel·명시 방향 | 사격→release 후 이동, 피해 해결 후 분기, stale 결과 차단 |
| CA16.3 | state effects/profile·targetability·blackboard·selector | 연속 HP 변신/skip·카운터·쿨다운 |
| CA16.4 | field work·signal·wave/summon·다중 actor와 제작 trace | 포큐버스 작업→억제, authority 보존 |

이는 CA16의 인수 가능한 하위 묶음이지 이미 생성된 네 Goal이 아니다. 실제 workflow에서 각각 독립 계약이면 별도 실행 작업으로 분할한다. 보스 계약 설계는 CA01 이후부터 가능하며, 모든 유물 콘텐츠가 완성될 때까지 설계 자체를 기다리지 않는다. 실행 검증은 필요한 공통 계약 구현 뒤에 한다.

## 11. 보스 전용 독립 제작 시험

엔진 동결 뒤 작성자는 공식 보스 이름을 쓰지 않는 새 팩에서 (a) 상태/HP 전이와 둘 이상의 스킬 작업, (b) 소환 또는 작업 신호에 반응하는 기믹, (c) 실패/취소 route를 RON으로 구성한다. 단순 이름·HP 수치 복사는 부족하다. 새 공식 lore·밸런스는 만들 필요 없고 기존 기능을 시험용으로 재조합하면 된다.

이 시험의 data-only diff, validation/compile receipt, 시간순 task trace를 남긴다. runner가 이름을 알고 특수 작업을 대신하면 불합격이다. 지원한다고 했지만 실행되지 않는 조합이 발견되면 공통 기능 또는 문법 계약을 보완한 뒤 다시 동결한다.
