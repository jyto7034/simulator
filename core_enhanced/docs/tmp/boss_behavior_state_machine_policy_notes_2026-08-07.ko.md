# 보스 행동 상태 머신 정책 논의 임시 기록 — 2026-08-07

- 상태: 정책 논의 진행 중 / 구현 미착수
- 문서 등급: temporary design notes / non-canonical
- 대상: `DefenseRoute`의 Elite, NormalBoss, FinalBoss를 중심으로 한 opt-in 보스 행동
- 다음 단계: 핵심 정책 논의 → read-only 코드 영향 조사 → v1 정책 확정 → 선행 기반 보강 → 첫 보스 패턴 수직 구현

## 문서 책임

이 문서는 2026-08-07에 진행한 보스 이동 AI와 상위 행동 상태 머신 논의를 임시로 보존한다.
현재 runtime, live RON, Unity wire 계약이나 구현 완료를 주장하지 않는다.

- 현재 gameplay 규칙은 [`archive/game_rulebook_legacy_2026-10-04.md`](../archive/game_rulebook_legacy_2026-10-04.md)를 우선한다.
- 현재 runtime 불변식은 [`core_runtime_contract.md`](../core_runtime_contract.md)를 우선한다.
- 현재 live RON authoring과 validation은 [`data_loading_contract.md`](../data_loading_contract.md)를
  우선한다.
- 이 기록을 근거로 production code/data/canonical을 선행 변경하지 않는다.
- 정책 논의는 한 번에 한 항목씩 진행하고, 사용자가 `다음.`이라고 한 뒤 다음 항목으로 넘어간다.

## 문제와 목표

현재 route enemy는 battlefield RON에서 확정된 authored route를 따라 이동한다. 이 구조는 플레이어가
route와 진입 방향을 읽고 배치하는 `DefenseRoute`의 기본 전술성을 만든다. 반면 Elite, Boss와
FinalBoss까지 항상 같은 방식으로 route만 따라가면 보스 기믹과 패턴을 표현하기 어렵고 전투가
정적으로 느껴질 수 있다.

목표는 authored route를 제거하는 것이 아니다.

- 일반 침식 직원은 기본적으로 기존 authored route만 따른다.
- opt-in한 적은 field condition 또는 battle trigger에 따라 특정 player unit, defense object,
  named battlefield region 등을 향하는 별도 이동 의도를 가질 수 있다.
- 핵심 사용자는 Elite, NormalBoss, FinalBoss지만 runtime type으로 사용을 강제 제한하지 않는다.
- 장기적으로는 이동 연쇄만이 아니라 telegraph, skill, field gimmick과 phase 전환을 구성하는 보스
  패턴 기반이 필요하다.

## 확인한 기존 게임 경계

- 공식 전투 모드는 `DefenseRoute` 하나이며 보스도 같은 전장, 배치, wave와 transport 기반을 쓴다.
- authored route는 단순 선이 아니라 적 진행, 보호 대상 도달, block 우선순위, briefing과 Unity route
  표시의 기반이다.
- 이동과 충돌은 2D 연속좌표, 배치와 range 후보는 tile 기반이다.
- sticky engagement, Hard CC, Pause, ranged reposition과 released delivery lifecycle은 이미 별도
  전투 계약을 가진다.
- Core가 전투 상태와 판단의 authority이며 Unity는 route, target 또는 pattern state를 재계산하지
  않는다.
- active battle checkpoint/resync는 전투 상태를 재현하지만 active battle은 durable SavePoint run
  payload가 아니다.

## 승인된 정책

### BP-01 — Opt-in 적용 범위

- 보스 행동 AI는 RON의 명시적인 opt-in이다.
- AI 설정이 없으면 현재와 동일하게 authored route만 따른다.
- 첫 주 사용 대상은 Elite, NormalBoss와 FinalBoss다.
- enemy classification으로 사용을 강제하지 않는다. 필요하면 다른 route enemy도 같은 기반을 사용할
  수 있다.
- runtime admission은 `Boss인가`가 아니라 유효한 행동 설정이 있는지를 기준으로 한다.

### BP-02 — Route 복귀와 미래 objective repath

행동 종료 뒤 movement disposition은 다음 두 의미를 구분한다.

`RejoinForwardRoute`:

- 현재 authored route와 진행 authority를 보존한다.
- 현재 progress보다 뒤쪽 route cell로 되돌아가지 않는다.
- 현재 위치에서 도달 가능한 전방 route 지점으로 합류한 뒤 authored route를 계속 진행한다.
- 현재 기본 동작이자 RON field 생략 시 적용할 기본값이다.

`RepathToProtectedObjective`:

- 현재 위치에서 보호 오브젝트까지 새 최단 경로를 만들고 남은 이동 route를 교체하는 의미다.
- 보스 phase 전환이나 최종 돌격 같은 명시적인 영구 재경로에 사용할 미래 확장이다.
- 첫 구현 범위에는 포함하지 않는다.
- Schema가 future disposition을 식별하더라도 현재 live RON에서 사용하면 publication validation이
  명시적으로 거부해야 한다.
- 미구현 `RepathToProtectedObjective`를 `RejoinForwardRoute`로 silent fallback하지 않는다.

### BP-03 — 이동 연쇄의 소유권

- `next_movement`를 movement engine이 직접 소유하는 단순 이동 chain으로 만들지 않는다.
- 이동의 완료·실패·중단은 typed outcome으로 상위 보스 행동 runtime에 보고한다.
- 연속 이동, phase, skill과 field gimmick의 다음 단계 선택은 상위 pattern/state transition이 소유한다.
- Movement는 현재 state가 요청한 단일 movement intent를 기존 공간 규칙으로 수행하는 하위 실행기다.
- 아직 구현하지 않은 future action을 현재 RON이 사용하면 validation에서 거부하며 임의 no-op 또는
  fallback하지 않는다.

### BP-04 — 항상 활성화된 최상위 오케스트레이터

보스 상태 머신은 특수 패턴 때만 켜지는 보조 기능이 아니라 opt-in 보스의 고수준 행동을 항상 소유하는
최상위 오케스트레이터다. 그러나 전투 mechanics를 직접 재구현하지 않는다.

상위 오케스트레이션 runtime이 소유할 방향:

- current phase
- selected pattern과 current state
- transition 선택
- pattern이 고정한 target과 결정 결과
- pattern timer와 pattern-owned RNG 결정
- 현재 state가 허용하거나 요청한 typed action/movement intent

기존 전투 subsystem이 계속 소유할 권위:

- pathfinding과 실제 movement/collision
- block과 sticky engagement
- 기본 공격 eligibility, targeting, windup과 release
- skill cast, effect와 projectile/area lifecycle
- HP, damage, buff, Hard CC와 death
- battle objective와 종료 판정

평상시 route 전투도 상태 머신 밖의 암묵적 fallback으로 두지 않고 `RouteCombat`에 해당하는 명시적
상태 또는 동등한 baseline behavior로 표현한다. 특수 pattern state는 이동, 기본 공격, skill과 gimmick
중 무엇을 허용·요청·정지할지를 명시하고, 실제 수행은 각 기존 subsystem에 위임한다.

요약하면 상위 상태 머신은 `무엇을 언제 할지`를 결정하고 하위 subsystem은 `어떻게 유효하게 수행할지`를
결정한다.

### BP-05 — Encounter와 actor behavior의 두 계층

최종보스 세 종의 기획 골격을 대조한 결과, 최상위 orchestration을 보스 unit 하나의 component로만
표현하지 않는다.

`BossEncounterRuntime`:

- encounter phase, objective와 전역 transition을 소유한다.
- encounter clock, wave와 전역 mechanic scheduling을 조율한다.
- 구속구, 억제 디바이스, 알, 랜턴과 악장처럼 보스 unit 밖의 encounter-local mechanic을 연결한다.
- phase 전환에서 pending encounter action의 중단·보존과 다음 objective activation을 결정한다.

`BossActorBehaviorRuntime`:

- opt-in boss actor별 baseline behavior, selected pattern과 current state를 소유한다.
- actor target 결정과 movement/combat/gimmick intent를 하위 subsystem에 제출한다.
- 하위 실행기의 completion, failure와 interruption outcome을 받아 actor pattern transition을 진행한다.

보스별 고유 object와 rule은 typed `MechanicRuntime` 또는 기존 subsystem이 소유하고
`BossEncounterRuntime`이 이를 조율한다. 백야의 고해 기록과 전조 chain처럼 active battle 밖의
run progression은 boss runtime으로 옮기지 않는다.

### BP-06 — Phase별 RON pattern selector

- 다음 pattern의 선택 방식은 boss 전체에 고정하지 않고 encounter phase별 RON이 명시한다.
- RON은 arbitrary selection script를 작성하지 않고 Core가 구현·검증하는 closed typed strategy 중
  하나를 선택한다.
- 승인된 strategy vocabulary는 다음 방향을 가진다.
  - `FixedSequence`: authored order로 pattern을 실행하고 명시된 경우 반복한다.
  - `PriorityEligible`: 현재 조건을 만족하는 pattern 중 authored priority가 가장 높은 항목을 고른다.
  - `WeightedEligible`: 현재 조건을 만족하는 pattern 중 authored weight와 authoritative battle RNG로
    하나를 고른다.
- 같은 boss도 phase마다 다른 selector를 사용할 수 있다. 예를 들어 초반은 eligible weighted selection,
  마지막 phase는 authored fixed sequence를 사용할 수 있다.
- RNG가 필요한 선택은 Core battle RNG와 committed runtime state가 authority이며 reconnect/resync가 다시
  추첨하지 않는다.
- 첫 구현은 실제 reference boss가 요구하는 strategy만 지원한다. 알려진 future strategy라도 현재
  미구현이면 RON publication validation에서 거부하고 다른 selector로 fallback하지 않는다.

### BP-07 — State task 병행과 충돌 경계

- 하나의 state는 임의 개수의 closed typed task를 가질 수 있다. 이 task 목록은 서로 경쟁하는 후보가
  아니라 state가 함께 실행하려는 병행 action 묶음이다.
- 각 task type은 자신이 요구하는 intrinsic capability claim을 Core 구현에서 선언한다. RON author가
  임의 claim을 작성하거나 실제 task 권위와 다르게 축소하지 않는다.
- 한 state의 task들은 필요한 모든 claim을 원자적으로 함께 획득할 수 있어야 한다. 동시에 성립할 수
  없는 exclusive claim 조합은 publication validation에서 거부한다.
- 서로 충돌하지 않는 movement, combat action, encounter mechanic, telegraph 등의 task는 같은 state에서
  병행할 수 있다. 고정된 author-facing channel slot 수로 조합을 제한하지 않는다.
- 같은 권위를 두고 경쟁하는 행동은 한 state의 task priority로 중재하지 않는다. 별도 State 또는
  Pattern으로 작성하고 BP-06 selector나 명시적 transition priority가 선택한다.
- 여러 state에서 공통으로 유지할 task는 상위 parent state 또는 `BossEncounterRuntime`에 둔다. 예를
  들어 phase 전체의 darkness/권능 scheduler는 child actor pattern과 독립적으로 유지할 수 있다.
- task별 authored numeric priority로 동적 compatible-set scheduler를 만드는 안은 채택하지 않는다.
  복수 exclusive capability를 요구하는 task가 섞이면 부분 admission, starvation과 불투명한 선택을
  만들고 이미 존재하는 State/Pattern selector와 결정 책임이 중복되기 때문이다.
- 실행 중 state가 바뀔 때 task cancel/suspend와 released action 보존은 별도 interruption lifecycle
  정책에서 결정한다.

### BP-08 — Phase transition timing의 명시적 authoring

- 각 Phase transition은 조건과 target phase뿐 아니라 전환 timing을 RON에 반드시 명시한다.
- 허용 timing은 closed typed vocabulary인 `Immediate`, `AfterCurrentState`,
  `AfterCurrentPattern` 세 가지다.
- `Immediate`는 조건이 확정된 authoritative decision boundary에서 현재 Phase를 즉시 떠난다. 실행 중
  task/action을 어떻게 중단하거나 보존하는지는 별도 interruption lifecycle 정책을 따른다.
- `AfterCurrentState`는 조건 충족 시 target Phase를 pending으로 고정한다. 현재 State가 끝나면 같은
  Pattern의 다음 State를 시작하지 않고 pending Phase로 전환한다.
- `AfterCurrentPattern`은 조건 충족 시 target Phase를 pending으로 고정한다. 현재 Pattern은 끝까지
  진행할 수 있지만 Phase selector가 새 Pattern을 선택하기 전에 pending Phase로 전환한다.
- 조건 충족으로 pending transition이 만들어진 뒤 조건이 다시 거짓이 되더라도 transition을 취소하거나
  target을 재선택하지 않는다.
- `timing` 생략 default, encounter kind에 따른 추론과 unsupported value fallback은 없다. 누락·미지원
  timing은 RON publication validation에서 거부한다.
- 같은 decision boundary에서 여러 Phase transition이 동시에 충족될 때의 선택 우선순위는 별도
  transition arbitration 정책에서 결정한다.

### BP-09 — State exit의 즉시 task 취소

- State가 종료되면 그 State가 소유한 모든 실행 중 task를 즉시 취소한다.
- release 전 movement intent, attack windup과 skill cast는 취소하고 해당 task/action이 소유한 lock과
  state-bound bookkeeping을 정리한다.
- 반복 task는 State exit 뒤 새로운 action/event를 예약하지 않는다.
- State task가 유지하던 state-bound aura와 field effect는 State exit와 함께 제거한다.
- 취소 전에 이미 적용된 damage/effect는 rollback하지 않는다.
- 이미 release된 projectile과 detached delivery/area는 State task와 분리된 기존 battle lifecycle을
  계속한다.
- 상위 Phase, parent State 또는 `BossEncounterRuntime`이 소유한 task는 child State exit로 취소하지
  않는다.
- task별 `on_exit` RON option, `PreserveCurrentAction`, generic suspend/resume와 State 재진입 시 task
  continuation은 제공하지 않는다.
- 현재 action을 정상 완료해야 하는 전환은 task exit option이 아니라 BP-08의 `AfterCurrentState` 또는
  `AfterCurrentPattern` timing으로 authoring한다. `Immediate`는 현재 State task의 즉시 취소를 뜻한다.
- State task 취소, lock 정리, detached lifecycle 판정과 다음 State/Phase admission은 부분 상태가
  노출되지 않도록 하나의 authoritative transition boundary에서 처리한다.

### BP-10 — Pattern은 분기 가능한 named State graph

- Pattern 내부 실행 구조를 authored linear State sequence로 제한하지 않는다.
- 각 Pattern은 stable unique State ID와 명시적인 entry State를 가진 directed graph다.
- State의 typed transition은 다른 State ID 또는 BP-12에서 정의한 terminal Pattern outcome을 가리킬 수
  있다.
- target 도달/소멸, task completion/failure, timer와 encounter condition에 따라 Pattern 내부에서 서로
  다른 State로 분기할 수 있다.
- Graph 선택은 arbitrary expression/script 실행을 허용한다는 뜻이 아니다. State task와 transition
  condition은 Core가 구현·검증하는 closed typed vocabulary를 사용한다.
- missing/duplicate State ID, missing entry와 존재하지 않는 transition target은 RON publication
  validation에서 거부한다.
- Cycle, terminal outcome과 모든 경로의 종료 가능성은 BP-11~BP-14를 따르며, transition arbitration과
  runtime loop guard의 나머지 세부는 후속 정책에서 별도로 확정한다.

### BP-11 — Pattern graph의 bounded loop만 허용

- Pattern graph는 반복 표현을 위해 cycle을 가질 수 있지만 모든 cycle은 명시적인 bounded loop gate를
  통과해야 한다.
- Loop authoring은 stable loop identity와 양수 maximum iteration count를 명시해야 한다. 생략된 limit,
  0과 overflow 가능한 값은 publication validation에서 거부한다.
- Loop counter는 Pattern activation마다 초기화하고 해당 active Pattern runtime이 소유한다. State를
  재진입하거나 reconnect/resync한다고 초기화하지 않는다.
- Maximum iteration에 도달하면 loop edge를 다시 탈 수 없고 authored exhaustion target 또는 terminal
  outcome으로 진행해야 한다. 임의 State에 머물거나 첫 State로 fallback하지 않는다.
- `loop_limit` 없는 graph cycle과 cycle 안에서 빠져나올 수 없는 구성은 publication validation에서
  거부한다.
- 같은 timestamp에 battle time 진행, 외부 event 대기 또는 task completion 없이 instantaneous State만
  순환하는 zero-time cycle은 bounded 횟수와 관계없이 거부한다.
- 무기한 반복 행동은 Pattern graph의 unbounded cycle로 표현하지 않는다. BP-14의 Phase-owned State로
  유지하거나, finite Pattern이 terminal outcome을 낸 뒤 Phase selector가 다음 Pattern을 다시 선택하는
  방식으로 표현한다.
- Loop counter와 현재 traversal state는 authoritative active battle checkpoint/resync가 보존한다.

### BP-12 — Pattern은 authored named outcome을 반환한다

- Pattern은 호출된 하위 동작처럼 실행되고, terminal node에 도달하면 상위 Phase에 하나의 named outcome을
  반환한다.
- Outcome ID는 Pattern의 RON 정의에 stable ID로 명시한다. `Completed`, `TargetEscaped`,
  `CoreDestroyed`처럼 해당 Pattern의 의미를 표현할 수 있으며 전역 고정 `Success`/`Failure` 두 값으로
  제한하지 않는다.
- 모든 terminal node는 선언된 outcome 하나를 명시해야 한다. 선언되지 않은 outcome 참조와 outcome이
  없는 terminal node는 publication validation에서 거부한다.
- Phase는 반환된 outcome에 따라 다음 Pattern 선택, State 전이 또는 Phase 전이를 결정할 수 있다.
  Phase가 관찰할 수 있는 것은 선언된 outcome이며 하위 Pattern 내부 State나 task 구현 세부에 직접
  결합하지 않는다.
- 정상적인 authored outcome과 runtime 실행 오류·상위에 의한 강제 취소는 분리한다. 오류나 취소를
  콘텐츠 outcome으로 위장하거나 임의 `Success`로 fallback하지 않는다.
- Outcome별 상위 처리는 BP-13의 exhaustive mapping을 따르고 Pattern 종료 가능성 검증은 BP-14를
  따른다.

### BP-13 — Pattern outcome은 폐쇄형이며 상위 Phase가 exhaustive하게 처리한다

- 각 Pattern이 선언한 authored outcome 집합은 해당 Pattern 반환 타입의 폐쇄형 variant 집합으로
  취급한다.
- Pattern을 호출·선택하는 상위 Phase는 반환 가능한 모든 authored outcome을 명시적으로 처리해야
  한다. 하나라도 누락되면 RON publication validation에서 거부한다.
- 여러 outcome을 동일한 후속 행동으로 보내는 grouped arm은 허용하지만 `default`, wildcard와
  unknown-outcome fallback은 허용하지 않는다.
- Pattern의 outcome variant를 추가·삭제·이름 변경하면 모든 참조 Phase의 exhaustive mapping을 다시
  검증한다. 기존 호출부가 암묵적으로 새 의미를 받아들이게 하지 않는다.
- Outcome ID 참조는 자유 문자열 비교로 실행하지 않는다. publication 단계에서 정의를 resolve하고
  typed/stable runtime representation으로 변환한 뒤 실행한다.
- Runtime 실행 오류와 상위 강제 취소는 authored outcome variant가 아니며 이 mapping으로 위장해
  처리하지 않는다. 해당 시스템 경로의 전파·복구 정책은 별도로 정의한다.
- 모든 정상 terminal node가 선언된 outcome을 반환하고, 모든 반환 variant가 상위에서 처리된다는 두
  방향의 완전성을 publication validation이 검사한다.

### BP-14 — Pattern은 항상 finite하고 상주 행동은 Phase-owned State가 담당한다

- `Persistent Pattern`이라는 별도 종류를 도입하지 않는다. 모든 Pattern은 유한한 하위 동작이며 정상
  실행 경로에서 반드시 BP-12의 named outcome 하나를 반환한다.
- Pattern graph의 모든 State에서 terminal outcome으로 진행할 수 있어야 한다. 종료 여부가 외부
  event에만 의존하는 wait에는 authoritative timeout과 fallback transition/outcome을 명시해야 한다.
- 무기한 route defense, idle, patrol, trigger 대기와 같은 상주 행동은 장기 수명을 가진 Phase가
  소유하는 State로 표현한다. Pattern과 State 양쪽에 같은 persistent 책임을 만들지 않는다.
- Phase-owned State는 필요할 때 finite Pattern을 호출·선택하고, 반환된 outcome을 BP-13에 따라
  exhaustive하게 처리하여 다음 State, Pattern 또는 Phase 전이를 결정한다.
- Pattern 실행을 위해 상주 State를 떠날 때 State-owned task는 BP-09에 따라 취소한다. Pattern 종료 후
  상주 행동으로 돌아가면 해당 State에 새로 진입하며 generic suspend/resume을 사용하지 않는다.
- `AfterCurrentPattern`은 finite Pattern의 종료 경계를 기다린다. Phase의 `Immediate` 전이가 실행 중인
  Pattern을 취소할 수 있지만, 이 시스템 취소를 authored outcome으로 변환하지 않는다.
- Pattern selector, sequence와 outcome mapping에는 persistent 여부에 따른 예외 규칙이 존재하지
  않는다. Pattern은 호출되면 outcome을 반환하거나 명시적인 상위 취소를 받는다는 단일 lifecycle
  계약을 따른다.

### BP-15 — Phase는 State와 PatternCall로 구성된 상주 control graph를 가진다

- Phase는 장기적인 전투 기믹 국면이며, 하나의 Pattern 종료만으로 자동 종료되지 않는다. authored
  Phase transition condition이 충족되고 BP-08의 timing 정책에 따라 commit될 때만 다음 Phase로
  전환한다.
- 각 Phase는 stable node ID와 명시적인 entry node를 가진 control graph를 소유한다. Actor behavior를
  구동하는 node 종류는 `State`와 `PatternCall`로 제한한다.
- `State` node는 `RouteCombat`, `Guarding`, `Idle`처럼 장시간 유지될 수 있는 행동 모드와 그 task를
  소유한다. Phase 자체가 소유하는 timer, wave와 field mechanic task는 control node 전환과 무관하게
  Phase 수명 동안 유지된다.
- `PatternCall` node는 BP-06의 selector 또는 명시적 Pattern 참조를 통해 하나의 finite Pattern 실행을
  시작하는 호출 경계다. PatternCall 자체는 이동·공격 task를 소유하지 않는다.
- State에서 PatternCall로 진행할 때 기존 State는 종료되고 BP-09에 따라 State-owned task를 취소한다.
  Pattern 반환 뒤 이전 State를 resume하지 않으며, outcome mapping이 지정한 State 또는 다음
  PatternCall node로 진행한다.
- Pattern은 Phase control node로 직접 jump하거나 Phase 내부 상태를 수정하지 않는다. BP-12의 named
  outcome만 반환하고, Phase가 BP-13의 exhaustive mapping으로 다음 control node 또는 Phase 전이를
  결정한다.
- 동일 actor에 대해 Phase State와 Pattern 내부 State를 동시에 활성화하지 않는다. Authoritative actor
  behavior의 active leaf는 언제나 하나이며 control-node 교체와 task cleanup/admission은 하나의 전이
  경계에서 처리한다.
- Phase control graph는 장기 실행을 위해 State↔PatternCall cycle을 가질 수 있지만, 같은 timestamp에
  시간 진행·event 대기·task completion 없이 도는 zero-time cycle은 publication validation에서
  거부한다.
- 개념적으로는 하나의 큰 행동 그래프이며, Pattern은 재사용과 typed outcome 경계를 위해 분리한 finite
  subgraph다. 재귀적인 Phase 호출이나 Pattern을 Phase처럼 상주시켜 이 계층을 우회하지 않는다.

### BP-16 — 같은 timestamp의 cross-scope 전이는 상위 lifecycle owner가 우선한다

- 동일 authoritative timestamp에 여러 계층의 전이가 함께 eligible해지면 lifecycle 범위가 큰 상위
  owner가 하위 행동 전이보다 우선권을 가진다.
- 고정 우선순위는 `battle terminal/death/despawn > Phase transition > Phase control transition >
  Pattern 내부 State transition > new task admission`이다. 콘텐츠 RON은 이 cross-scope 순서를
  override할 수 없다.
- `Immediate` Phase transition은 현재 Phase State 또는 Pattern을 BP-09의 경계에서 취소하고 하위
  transition target과 새 task를 admission하지 않는다.
- `AfterCurrentState`와 `AfterCurrentPattern`은 조건 충족 시 pending transition의 우선권을 즉시
  latch하되, BP-08이 허용한 현재 State 또는 Pattern 종료 경계까지만 실행한다. 해당 경계에서 발생한
  completion/outcome은 authoritative event로 기록할 수 있지만 그 mapping을 따라 다음 control node,
  Pattern State 또는 task를 시작하지 않고 pending Phase transition을 commit한다.
- 하위 completion과 transition candidate를 계산한 사실은 상위 전이에 의해 rollback되는 부작용으로
  취급하지 않는다. 다만 stage 단계에서 얻은 하위 candidate가 상위 lifecycle 결정 뒤 commit 권한을
  잃는 것으로 처리한다.
- 이 정책은 서로 다른 scope 사이의 arbitration만 정한다. 동일 graph node에서 여러 transition이 함께
  eligible한 경우의 authoring 및 tie-break는 별도 정책으로 확정한다.

### BP-17 — 동일 node의 transition priority는 마지막 중재 장치다

- 동일 graph node에서 여러 transition condition이 같은 authoritative timestamp에 eligible할 수 있으므로
  각 transition은 stable transition ID와 명시적인 priority를 가져야 한다. 누락된 priority는 허용하지
  않는다.
- Priority는 해당 node의 transition 사이에서 고유해야 한다. 같은 priority가 둘 이상이면 publication
  validation에서 거부하며 RON 선언 순서, 문자열 ID 또는 container iteration order로 tie-break하지
  않는다.
- Runtime은 같은 pre-transition snapshot에서 해당 node의 모든 condition을 평가하고, eligible candidate
  중 priority가 가장 높은 transition 하나만 commit한다. 먼저 평가된 condition의 side effect가 다음
  condition 결과에 영향을 주지 않는다.
- 이 authored priority는 동일 node 안에서만 유효하며 BP-16의 cross-scope lifecycle 우선순위를
  override할 수 없다.
- Priority는 조건 의미를 대신하는 일반 행동 선택기나 task scheduler가 아니다. 가능한 경우 transition
  condition을 의도에 맞게 구체화하고, priority는 실제 동시 충족이 발생했을 때 결정론적으로 한 경로를
  선택하는 마지막 중재 장치로만 사용한다.
- 복수 candidate가 동시에 eligible했던 사실과 최종 winner는 후속 observability 정책에 따라
  authoritative 기록 대상으로 삼는다.

### BP-18 — Target은 기본적으로 lock하고 명시적 decision point에서만 재평가한다

- Target acquisition이 성공하면 기본 정책 `LockUntilInvalid`에 따라 선택된 target identity를 현재
  Pattern activation 동안 유지한다. 거리나 점수가 조금 변했다는 이유로 매 tick 또는 임의 event마다
  더 좋은 target으로 교체하지 않는다.
- Target lock은 authoritative runtime state이며 checkpoint/resync에서 동일 identity로 복원한다.
  Reconnect나 Unity 재표시를 이유로 target selection을 다시 실행하지 않는다.
- 콘텐츠가 동적인 재선택을 요구하면 RON에서 `ReevaluateAtDecisionPoint`를 명시할 수 있다. 재평가는
  State entry, task completion 또는 typed timer/event처럼 authored되고 검증된 decision point에서만
  수행한다.
- 재평가 시에도 동일 pre-decision snapshot과 후속 deterministic target ordering 정책을 사용한다.
  Unity frame, container iteration order와 wall clock은 후보 및 선택 결과에 관여하지 않는다.
- 공격 telegraph/windup 중 target 변경은 해당 Pattern이 별도의 decision point를 명시하지 않는 한
  허용하지 않는다. Target이 invalid해진 경우에는 후속 target-loss 정책으로 처리하며 암묵적으로 다음
  후보를 고르지 않는다.
- `LockUntilInvalid`가 기본값이고 재평가 정책을 생략한 기존/신규 콘텐츠는 이 의미를 따른다.

### BP-19 — Target invalidation 뒤의 행동은 RON graph가 명시한다

- 엔진은 locked target이 사망, 철수, despawn 또는 적대 actor lifecycle 이탈로 더 이상 존재하는 유효
  target identity가 아님을 authoritative하게 판정하고 `TargetInvalidated` event를 발행한다.
- 엔진은 invalidation을 이유로 후보 목록의 다음 target을 자동 선택하거나 이전 acquisition query를
  암묵적으로 재실행하지 않는다.
- 현재 Pattern graph는 `TargetInvalidated`에 대해 명시적으로 `AcquireTarget` State로 전이하여 재획득을
  시도하거나, 선언된 `TargetLost` 계열 named outcome을 반환하는 경로를 가져야 한다. 다른 authored
  recovery State로 진행하는 것도 typed transition으로만 허용한다.
- Pattern이 target-dependent task를 포함하면서 invalidation 경로를 처리하지 않으면 publication
  validation에서 거부한다. Runtime generic fallback, no-op과 임의 기본 target은 허용하지 않는다.
- 재획득 State에 진입한 경우 BP-18의 decision-point acquisition으로 새 target을 선택하며, 기존
  target identity와 telegraph를 조용히 덮어쓰지 않는다.
- Target invalidation의 판정 범위는 BP-20을 따르며, invalidation 이후 행동의 소유자는 엔진이 아니라
  RON graph다.

### BP-20 — Temporary tracking loss와 Grace를 현재 모델에 도입하지 않는다

- 현재 `DefenseRoute`에는 line-of-sight, stealth, 동적으로 열리고 닫히는 navigation obstacle 또는
  보행 중인 player target처럼 일반적인 temporary tracking loss를 만드는 정규 규칙이 없다. 따라서
  generic `Immediate | Grace(duration)` target-loss policy를 도입하지 않는다.
- Target validity는 actor identity와 active hostile lifecycle의 유효성이다. 사망, 철수, despawn과 같은
  lifecycle 이탈은 BP-19의 즉시 `TargetInvalidated`이며 grace 대상이 아니다.
- `TacticalRelocateUnit`으로 target position이 바뀌어도 같은 runtime identity가 active한 동안 target
  lock은 유지한다. 위치 변경을 새 target acquisition으로 취급하지 않는다.
- Path를 찾을 수 없음, 공격 사거리 이탈, 현재 skill/effect의 usefulness 또는 air eligibility 불충족은
  target invalidation이 아니다. 각각 `PathUnavailable`, `OutOfRange`, `ActionIneligible` 계열의 typed
  task result로 Pattern graph가 처리한다.
- Target identity 유지, movement feasibility와 action eligibility를 서로 다른 검증 축으로 유지한다.
  한 축의 실패를 다른 축의 implicit invalidation이나 자동 재탐색으로 변환하지 않는다.
- 미래에 stealth, line-of-sight, dynamic door/obstacle처럼 실제 temporary loss 규칙이 추가되면 별도
  정책 승인 뒤 typed loss condition과 grace semantics를 확장한다. 구현 전 RON에 해당 future variant를
  작성하면 publication validation에서 거부한다.

### BP-21 — Target acquisition은 typed lexicographic ordering을 사용한다

- `AcquireTarget`은 target kind, closed typed filter 목록과 하나 이상의 의미 있는 `order_by` key를 RON에
  명시해야 한다. 엔진은 target 역할이나 unit 분류에서 primary ordering을 추론하거나 숨은 기본 점수식을
  적용하지 않는다.
- 모든 후보와 ordering metric은 같은 authoritative pre-acquisition snapshot에서 계산한다. 앞 후보의
  평가나 선택 시도가 뒤 후보의 metric과 eligibility를 변경하지 않는다.
- `order_by`는 `Min(PathCost)`, `Min(HpRatio)`처럼 Core가 구현·검증하는 closed typed key를 순서대로
  나열하며 lexicographic comparison을 수행한다. Arbitrary expression, weighted score sum과 floating-point
  epsilon tie-break는 허용하지 않는다.
- Metric의 전제조건도 typed validation한다. 예를 들어 `Min(PathCost)`가 실제 도달 가능한 후보만을
  요구한다면 query가 해당 movement profile의 reachability filter를 명시해야 하며, path 없음에 임의의
  무한대 점수를 부여해 후보를 조용히 뒤로 미루지 않는다.
- Authored key가 모두 동률이면 Core가 target kind별 canonical final key를 적용한다. Unit은 battle spawn
  sequence 뒤 runtime unit ID, object는 stable authored object ID, tile/field는 canonical coordinate
  ordering을 사용한다. RON은 이 final tie-break를 생략·재정의하지 못한다.
- Container iteration order, RON declaration order와 Unity 표시 순서는 target 선택에 관여하지 않는다.
  Random target selection이 필요하면 별도의 명시적 authoritative-RNG strategy로 확장하며 implicit
  tie-break로 RNG를 소비하지 않는다.

### BP-22 — Condition은 기존 tick/event pipeline 안에서 state와 event source를 구분한다

- Boss Runtime을 위해 Unity frame polling이나 별도의 독립 update loop를 만들지 않는다. Server tick이
  simulation time을 전진시키고 BattleCore가 timestamp event bucket을 처리하는 기존 authoritative
  pipeline 안에서 condition을 평가한다.
- RON condition source는 최소한 `StatePredicate`와 `EventOccurred`를 서로 다른 closed typed variant로
  구분한다. 둘을 자유 문자열이나 동일 boolean callback으로 뭉개지 않는다.
- `StatePredicate`는 현재 authoritative battle/encounter state의 level을 검사한다. Node entry 시점에
  이미 참이면 즉시 eligible하며, 관련 state mutation이 commit된 event 경계에서 다시 평가한다.
- `EventOccurred`는 현재 node activation 뒤 발생한 명시적 typed event edge만 관찰한다. 과거 event log를
  검색해 재발생시거나 현재 state가 같다는 이유로 event가 발생했다고 간주하지 않는다.
- Node activation event sequence/cursor는 active runtime state다. 같은 timestamp에서도 activation보다
  뒤의 event만 `EventOccurred` 후보가 되며 checkpoint/resync가 cursor를 보존한다.
- 과거부터 유지된 사실이 필요하면 `StatePredicate` 또는 별도로 정의된 persistent encounter state를
  사용한다. Timer condition은 wall-clock polling이 아니라 authoritative battle-time event로 예약한다.
- 같은 timestamp 안의 condition candidate와 transition arbitration은 BP-16·BP-17 및 후속 atomic commit
  정책을 따른다. 이 정책은 기존 event ordering을 우회하는 새 실행 순서를 만들지 않는다.

### BP-23 — Event memory는 명시적인 Pulse 또는 State-local latch다

- 각 `EventOccurred` condition은 `Pulse`와 `UntilStateExit` 중 memory policy 하나를 RON에 반드시
  명시해야 한다. 기본값과 event 종류에서의 암묵적 추론은 허용하지 않는다.
- `Pulse`는 해당 typed event가 관찰된 authoritative event 경계에서만 참이다. 그 경계에서 선택되지
  않았거나 후속 guard를 만족하지 못한 event를 다음 tick/bucket에서 재사용하지 않는다.
- `UntilStateExit`는 현재 State가 활성화된 뒤 event가 한 번이라도 발생하면 State-local boolean latch를
  세운다. 현재 task가 계속 실행된 뒤 task completion 같은 후속 조건과 함께 분기하는 용도다.
- State-local latch는 State entry 때 비어 있고 State exit/cancellation 때 반드시 폐기한다. 같은 State
  definition에 재진입해도 이전 activation의 latch를 복원하지 않는다.
- 활성 State의 latch 값은 authoritative runtime state이며 checkpoint/resync가 보존한다. Reconnect로
  event를 다시 발행하거나 latch를 초기화하지 않는다.
- Generic Pattern-wide/Phase-wide event latch는 제공하지 않는다. State를 넘어 유지할 사실은 명시적
  State 전이, Pattern outcome 또는 typed encounter/mechanic state로 표현한다.
- Event 발생 횟수나 누적량이 필요한 기믹은 boolean latch를 반복 사용하지 않고 별도의 typed counter를
  사용한다. Latch는 “현재 State 동안 한 번이라도 발생했는가”만 표현한다.

### BP-24 — Condition과 Transition은 generic cooldown을 소유하지 않는다

- Condition은 authoritative fact/event의 충족 여부를 표현하고 Transition은 다음 경로 선택을 표현한다.
  둘 모두 generic `cooldown_ms`나 재진입 억제 timer를 소유하지 않는다.
- 참인 level condition이 graph 재진입 직후 다시 transition을 일으키는 것은 condition의 정상 의미다.
  이를 transition cooldown으로 숨기지 않고 condition, State 구조 또는 persistent mechanic state를
  올바르게 작성한다.
- State가 최소 시간 유지되어야 하면 명시적인 battle-time timer condition/task로 표현한다. Timer는
  BP-22의 예약 event이며 wall clock이나 마지막 transition 시각의 숨은 비교를 사용하지 않는다.
- 공격 Pattern의 재사용 제한은 Pattern selector admission/cooldown, Skill과 Action의 재사용 제한은 기존
  실행 subsystem의 cooldown이 각각 소유한다. Transition ID를 실행 cooldown identity로 재사용하지 않는다.
- 즉시 왕복과 무한 전이는 graph publication validation, BP-11·BP-15의 zero-time cycle 금지 및 runtime
  loop guard로 막는다. Cooldown을 graph correctness 보완책으로 사용하지 않는다.
- Cooldown 상태는 실제 소유 실행 단위의 authoritative runtime/checkpoint에만 저장하며 condition 평가기가
  별도의 숨은 cooldown state를 만들지 않는다.

### BP-25 — Finite Task 결과는 폐쇄형이며 State가 exhaustive하게 처리한다

- 각 finite Task type은 Core가 정의하는 closed typed result variant 집합을 가진다. 예를 들어 movement
  task는 `Reached`, `PathUnavailable`, `TargetInvalidated`, `TimedOut`처럼 해당 실행 domain에서 예상
  가능한 정상·실패 결과를 명시한다.
- State는 소유한 finite Task가 반환할 수 있는 모든 result variant를 RON에서 처리해야 한다. 누락,
  `default`, wildcard와 unknown-result fallback은 publication validation에서 거부한다.
- 각 result arm은 다른 State/control node로의 typed transition, Pattern named outcome 반환 또는 명시적
  `ContinueState` disposition 중 하나를 선택한다. 여러 result를 같은 arm으로 묶는 것은 허용한다.
- 엔진은 실패 result를 이유로 task를 자동 재시도하거나 target 변경, route 복귀와 대체 action을
  실행하지 않는다. 필요한 recovery는 result arm에서 RON graph가 명시한다.
- State/Pattern/Phase exit가 소유 task를 BP-09에 따라 취소한 것은 authored Task result가 아니다. 취소로
  result mapping을 실행하거나 취소를 `Completed`/`Failed` variant로 위장하지 않는다.
- Engine invariant 위반, corrupt runtime과 구현 오류도 콘텐츠가 분기할 수 있는 Task result가 아니다.
  Expected domain failure와 system failure의 error channel을 분리한다.
- 같은 timestamp에 여러 finite Task result가 서로 다른 transition을 요청하면 같은 snapshot에서
  candidate를 만들고 BP-16·BP-17의 scope/priority arbitration을 따른다.

### BP-26 — Finite Task timeout은 admission부터 흐르는 battle-time deadline이다

- Core가 실행 특성상 외부 상태 때문에 종료가 지연될 수 있다고 선언한 finite Task type은 RON에서 양수
  `timeout_ms`를 필수로 가져야 한다. 누락, 0과 overflow 가능한 값은 publication validation에서
  거부하며 generic default timeout을 적용하지 않는다.
- Timeout은 State entry transaction에서 Task admission이 성공적으로 commit된 authoritative battle
  time부터 계산한다. Selection, validation 또는 stage 중에는 clock을 시작하지 않는다.
- Runtime은 `admitted_at_ms + timeout_ms`의 checked absolute deadline과 activation identity/epoch를
  저장한다. Checkpoint/resync는 남은 시간을 재계산하지 않고 이 상태를 그대로 보존한다.
- Pause는 battle time 자체를 멈추므로 timeout도 멈춘다. Hard CC, movement stall과 target까지의 진행
  정체는 battle time을 멈추지 않으므로 deadline은 계속 다가오며 별도의 active-task-time clock을 만들지
  않는다.
- State/Pattern/Phase exit로 Task가 취소되면 해당 timeout은 result를 만들지 않는다. Queue에 남은 stale
  timeout event는 activation identity/epoch 불일치로 no-op 처리한다.
- Task의 정상 completion 또는 더 구체적인 expected domain failure와 timeout deadline이 같은 timestamp면
  domain result를 timeout보다 먼저 판정한다. 이미 terminal result가 commit된 Task에 `TimedOut`을 중복
  생성하지 않는다.
- Deadline까지 terminal result가 없으면 BP-25의 closed `TimedOut` result를 반환하고 RON의 exhaustive
  arm이 처리한다.
- 고정 duration wait처럼 Core가 admission 시점에 정상 종료 시각을 완전히 결정하는 Task에는 중복
  watchdog timeout을 요구하지 않는다. Task type metadata가 timeout 필요 여부를 결정하며 RON author가
  임의로 면제하지 못한다.

### BP-27 — Pattern cooldown은 selector admission이 소유하고 Pattern 시작 때 소비한다

- Cooldown은 재사용 가능한 Pattern definition의 내부 상태가 아니라 이를 선택하는 selector candidate의
  admission policy다. 동일 Pattern을 참조하는 서로 다른 호출부가 사용 빈도를 명시적으로 결정한다.
- Candidate는 cooldown 없음 또는 양수 duration과 stable `cooldown_group_id`를 가진 battle-scoped
  cooldown을 명시한다. 같은 actor에서 같은 group ID를 공유하는 정의는 duration 의미도 일치해야 하며
  충돌하면 publication validation에서 거부한다.
- Eligible candidate가 선택되고 Pattern admission이 commit된 authoritative battle time에 cooldown의
  absolute ready time을 설정한다. Selection stage 실패나 admission 거부는 cooldown을 시작하지 않는다.
- 시작 뒤 Pattern이 named outcome으로 끝나거나 상위 전이로 취소돼도 cooldown을 환불·초기화하지 않는다.
  Phase 전환, State/PatternCall 재진입과 reconnect도 ready time을 바꾸지 않는다.
- Cooldown은 actor별 active battle runtime/checkpoint에 보존하고 battle 종료 시 폐기한다. 다른 actor의
  같은 group ID와 공유하지 않는다.
- Selector는 cooldown 중인 candidate를 eligibility 단계에서 제외한 뒤에만 priority/weight와 RNG를
  평가한다. Cooling candidate 때문에 RNG를 소비하거나 재굴림하지 않는다.
- 모든 candidate가 ineligible이면 엔진이 임의 Pattern을 선택하거나 가장 이른 cooldown까지 숨게 기다리지
  않는다. PatternCall이 명시한 `no_eligible` control path로 진행한다.
- Pattern 종료 뒤 추가 recovery 시간이 필요하면 Pattern 내부 Recovery State 또는 Phase control timer로
  표현한다. Cooldown 시작 시점을 completion으로 바꾸는 옵션은 제공하지 않는다.

### BP-28 — Boss Runtime RON은 별도 domain catalog가 소유한다

- Boss Runtime의 Pattern, actor behavior와 encounter runtime graph를 기존 abnormality metadata 또는 PvE
  encounter body에 직접 embed하지 않는다. 별도 `boss_runtime` domain catalog가 stable ID definition으로
  소유한다.
- Abnormality catalog는 actor의 정체성·combat stat·기본 공격·skill metadata를, PvE encounter catalog는
  전장·목표·wave·보상을 계속 소유한다. Encounter는 optional `boss_runtime_id`와 typed actor binding으로
  별도 catalog를 참조한다.
- Boss Runtime catalog 내부에서도 Pattern definition, actor behavior definition과 encounter runtime
  definition은 서로 다른 logical collection/schema로 구분하고 stable ID reference로 연결한다. 실제 파일
  분할 단위는 loader/tooling 구현 시 정하되 이 소유 경계를 합치지 않는다.
- 전용 authoring program이 RON을 생성하더라도 그 출력은 trusted input이 아니다. Core loader가 unknown
  field, duplicate ID, dangling/cyclic-invalid reference, typed outcome/result 완전성과 구현되지 않은
  variant를 publication 전에 다시 검증한다.
- 물리적 파일 분리만으로 runtime fault isolation을 주장하지 않는다. Boss Runtime과 기존 catalog의 모든
  cross-reference를 포함한 versioned bundle이 전부 유효할 때만 한 generation으로 atomic publication한다.
  일부 catalog만 새 generation으로 섞는 partial publication은 허용하지 않는다.
- 새 bundle 검증 실패 시 이미 publication된 last-known-good generation과 이를 pin한 active battle은
  변경하지 않는다. 유효한 이전 generation이 없는 startup에서는 fail closed한다.
- 이 분리는 human readability보다 schema ownership, generator/editor의 검증 범위, 손상 진단과 정의 재사용을
  위한 것이다. 전용 프로그램이 내부 참조를 자동 생성해도 stable ID와 strict Core validation을 생략하지
  않는다.

### BP-29 — 재사용 정의는 deep override가 아니라 typed parameter binding을 사용한다

- Pattern과 actor behavior의 재사용은 inheritance, partial patch와 deep merge가 아니라 definition이
  선언한 closed typed parameter signature와 call-site argument binding으로 표현한다.
- Call site는 모든 required parameter를 정확히 한 번 바인딩해야 한다. 누락, unknown/duplicate argument,
  type mismatch와 허용 범위 밖 값은 publication validation에서 거부하며 임의 기본값이나 caller field
  이름 추론을 사용하지 않는다.
- Parameter type은 Core가 검증할 수 있는 `DurationMs`, `TargetQueryRef`, `SkillRef` 같은 closed domain
  type과 bounded scalar/value로 제한한다. Arbitrary RON subtree, expression, script와 untyped map을
  argument로 전달하지 않는다.
- Referencing encounter/behavior는 Pattern graph node, transition, task list 또는 outcome set을 부분
  교체·추가·삭제할 수 없다. 구조 변경이 필요하면 새로운 stable definition ID를 작성한다.
- 동일 ID의 재정의와 source/file order에 따른 last-wins override를 금지한다. Duplicate ID는 bundle
  publication을 거부한다.
- Definition의 parameter signature가 추가·삭제·변경되면 모든 call site를 다시 type-check한다. Pattern의
  outcome signature는 BP-13, finite Task result는 BP-25와 함께 호출부에서 검증한다.
- Runtime은 call admission 시 resolve된 immutable argument frame을 activation identity와 함께 캡처한다.
  실행 중 다른 catalog definition을 다시 조회해 값이 바뀌거나 새 generation과 섞이지 않는다.
- 전용 authoring program은 parameter 입력과 참조 관리를 도울 수 있지만 Core의 signature/type/reference
  검증을 대신하지 않는다.

### BP-30 — Boss presentation은 기존 event replay stream에 완결된 projection으로 제공한다

- Unity에 별도 Boss 전용 transport나 내부 graph replay protocol을 추가하지 않는다. Player-visible boss
  presentation은 기존 `battle_update.events_delta`의 연속된 presentation stream에 typed event로
  투영하고, Unity는 `seq`/`time_ms` 순서로 재생한 뒤 `checkpoint.at_seq`에서 reconcile한다.
- Internal Phase/State/Pattern/Task ID와 transition candidate는 gameplay wire DTO가 아니다. 공개할 phase
  label, telegraph와 objective presentation은 별도의 stable presentation ID와 typed payload를 사용하며
  전체 내부 graph 위치는 debug/admin projection에서만 제공한다.
- 이동, skill cast/area, projectile, 피해와 사망은 기존 `Movement*`, skill/projectile, `HpChanged`,
  `UnitDied` presentation event를 우선 재사용한다. 같은 의미를 Boss wrapper event로 중복 발행하지 않는다.
- 기존 leaf event로 표현할 수 없는 공개 phase, 특수 pattern warning, field-wide mechanic와 objective
  presentation에만 새 boss/encounter presentation event variant를 추가한다.
- Duration/cancellable presentation은 stable `presentation_instance_id`를 가진 `Started`, 필요한 경우
  immutable `Updated`, 그리고 `Ended` lifecycle로 완결한다. `Ended`는 `Resolved`, `Cancelled`,
  `Interrupted` 같은 typed reason과 authoritative end time을 제공한다.
- Start/Update event는 Unity가 mutable RON이나 현재 AI state를 조회하지 않고 replay할 수 있도록 actor,
  target/anchor, Core가 확정한 affected tiles와 absolute timing 등 필요한 resolved payload를 포함한다.
  Unity-local asset 선택만 stable presentation ID에 매핑할 수 있다.
- 모든 새 presentation event는 기존 stream의 continuous seq, authoritative `time_ms`, same-timestamp seq
  order, stream-valid `cause.seq`, immutable payload와 duplicate/idempotency 계약을 그대로 따른다.
- Unity의 duration event parser/readiness marker는 새 event의 authoritative end time을 인식해야 한다.
  Start/Update/End matching, seq gap resync, cancellation cleanup과 setup-loss full replay를 contract test한다.
- Unity-facing checkpoint는 현재 공개 phase label/objective/HUD처럼 durable한 UI state만 보존한다.
  Telegraph 시작 animation이나 hit timing을 checkpoint에서 새로 만들지 않는다. 현재 setup-loss recovery가
  presentation stream을 seq 0부터 다시 제공하므로 active telegraph 복원도 event replay가 담당한다.
- Core-only active battle checkpoint는 Phase control node, Pattern State, active Task, latch/loop counter,
  target lock, cooldown/deadline와 immutable argument frame을 보존한다. 이를 Unity gameplay checkpoint에
  그대로 직렬화하지 않는다.
- 미래에 active event log pruning을 도입하면 prune boundary 이전부터 활성인 presentation의 복구 계약을
  별도 승인한다. 현재 정책에 pruning 전제를 미리 넣어 active-presentation snapshot을 중복하지 않는다.

### BP-31 — Control transition은 관련 변경 전체를 하나의 원자적 commit으로 적용한다

- Transition winner는 BP-16과 BP-17의 동일한 pre-transition snapshot에서 결정한다. winner를 고른 뒤
  기존 runtime state를 즉시 변경하지 않고 하나의 transition plan을 먼저 stage한다.
- Staged plan에는 기존 State/PatternCall의 종료와 소유 Task 취소, target lock과 movement intent 정리,
  다음 control node 진입, 새 Task admission, 진입 시점에 계약된 resource reservation/consumption과 cooldown
  시작, 그리고 그 결과로 발행할 semantic/presentation event가 함께 포함된다.
- Core는 staged plan의 참조, capability claim, entry prerequisite, resource/cooldown 조건과 다음 runtime
  invariant를 모두 검증한 뒤 한 번에 commit한다. 일부 mutation, cancellation 또는 event만 먼저 공개하지
  않는다.
- Commit 성공 시 runtime revision/checkpoint state와 event stream을 같은 논리적 경계에서 갱신한다. 새
  event의 연속된 `seq`, authoritative `time_ms`와 `cause`도 이 commit에서 확정하므로 Unity replay에서
  중간 control state가 관찰되지 않는다.
- 예상 가능한 entry prerequisite와 admission 불가는 transition condition/selector eligibility에서 미리
  제외한다. 동일 snapshot을 사용하므로 후보 선택 뒤 그 조건을 다시 다른 시점의 값으로 판정하지 않는다.
- Stage 검증이 예기치 않게 실패하면 plan 전체를 폐기하고 기존 control node와 Task를 그대로 유지한다.
  이를 콘텐츠용 암묵적 fallback transition으로 바꾸지 않으며 invariant error로 보고한다.
- Commit 이후 Task 실행 중 발생하는 `PathUnavailable`, `TargetInvalidated`, `TimedOut` 같은 정상적인
  gameplay 실패는 transition rollback 사유가 아니다. BP-25의 강타입 `TaskResult`로 현재 State가
  exhaustive하게 처리한다.
- 이전 commit에서 이미 적용된 피해, 발사된 projectile, 분리된 area 같은 외부 효과는 새 transition의
  rollback 범위가 아니다. 원자성은 현재 authoritative snapshot에서 시작한 이번 transition plan의 변경에
  한정한다.

### BP-32 — 같은 active node의 모든 transition source는 하나의 명시적 priority 집합에서 경쟁한다

- 한 arbitration boundary에서 `TaskResult`, timer, `EventOccurred`, `StatePredicate` 등 서로 다른
  source가 만든 transition을 하나의 candidate set으로 합친다. source 종류 자체에는 엔진 고정 우선순위를
  부여하지 않는다.
- 모든 candidate는 같은 pre-transition snapshot과 같은 수집 완료 event boundary에서 평가한다. 앞서
  평가한 condition의 side effect나 file/list iteration order가 뒤 candidate의 eligibility를 바꾸지 않는다.
- 물리적으로 trigger 목록이 나뉘어 있더라도 합쳐지는 모든 transition은 BP-17의 stable transition ID와
  명시적·고유한 priority 규칙을 공유한다. priority 동률, 누락 또는 source별 별도 priority namespace는
  publication validation에서 거절한다.
- Phase control graph에서는 현재 active control node, Pattern graph에서는 현재 active State에서 나가는
  transition만 이 candidate set에 들어온다. inactive node의 condition을 polling하거나 그 transition을
  예비 후보로 축적하지 않는다.
- Battle terminal/death/despawn, Phase transition, Phase control transition, Pattern State transition처럼
  scope가 다른 후보 사이에는 BP-16의 고정 순서를 먼저 적용한다. 이 정책은 같은 scope와 active node 안의
  source 종류에 숨은 precedence를 추가하지 않는다는 계약이다.
- 승자가 정해지기 전에는 어떤 candidate도 Task 취소, cooldown/resource 소비, movement intent 변경 또는
  event 발행을 하지 않는다. winner만 BP-31의 atomic transition plan을 만들고 losing candidate는 아무런
  side effect를 남기지 않는다.
- losing condition이 다음 boundary에도 후보가 되는지는 condition의 기존 의미를 따른다. level predicate와
  `UntilStateExit` latch는 여전히 참이면 다시 평가할 수 있고, `Pulse`는 BP-23의 해당 event boundary가
  끝나면 소멸한다.

### BP-33 — Phase 재진입은 이전 activation을 resume하지 않고 새로 시작한다

- Phase transition은 떠나는 Phase activation의 완전한 종료다. descendant State/PatternCall과 그 소유
  Task, target lock, movement intent, phase-local latch/counter/timer를 BP-09와 BP-31의 경계에서 정리한다.
- Phase graph가 나중에 같은 stable Phase ID로 돌아오더라도 이전 active control node나 실행 문맥을
  복원하지 않는다. 새 activation epoch와 immutable argument frame을 만들고 해당 Phase의 명시적 entry
  control node부터 다시 진입한다.
- 새 activation에서는 phase-local State와 latch/counter/timer를 초기화한다. BP-27의 battle-scoped
  selector cooldown처럼 수명이 전투 전체로 이미 정해진 값과 encounter 상위 scope가 소유한 상태는 Phase
  재진입을 이유로 초기화하지 않는다.
- Phase를 유지한 채 유한 행동을 수행하고 원래 흐름으로 돌아오는 요구는 Phase suspend/resume가 아니라
  BP-15의 `PatternCall`과 named Pattern outcome으로 표현한다.
- 종료된 Phase activation을 checkpoint에 dormant context로 쌓아 두지 않는다. Core checkpoint에는 현재
  active activation만 보존하며, 이전 activation에서 예약된 event는 activation epoch 불일치로 stale/no-op
  처리한다.
- RON에는 Phase resume, suspended Phase stack 또는 재진입 시 임의 checkpoint를 선택하는 옵션을 제공하지
  않는다. 그러한 요구가 실제 콘텐츠에서 확인되면 별도 lifecycle 정책으로 승인한다.

### BP-34 — Phase, Pattern과 State activation은 단일 소유권과 비재진입 lifecycle을 따른다

- 각 Phase, Pattern과 State 진입은 stable runtime activation identity/epoch를 정확히 하나 만들고, 각
  activation은 정확히 한 번 종료된다. 같은 definition ID 재진입은 기존 activation 재사용이 아니라 새
  identity 생성이다.
- Entry는 parent에서 child 순서로 stage한다. Phase activation과 phase-owned Task 및 entry control node,
  PatternCall과 Pattern activation 및 entry State, State와 그 Task admission을 각각 BP-31의 한 atomic
  plan에서 검증한다.
- Exit cleanup은 child에서 parent 순서로 계산한다. 하위 State/Pattern Task와 lock/intent를 정리한 뒤
  Pattern, control node, Phase 소유 상태를 종료하지만 외부에서는 BP-31의 단일 commit만 관찰된다.
- Entry/exit 처리 중 transition evaluator를 재귀 호출하지 않는다. Entry snapshot에서 이미 참인 predicate,
  admission 직후 즉시 끝나는 Task와 같은 후속 사실은 commit 뒤 새 simulation microstep에서 평가한다.
  필요한 연쇄는 동일 `time_ms` 안에서 순서대로 여러 atomic commit을 수행하며 server tick 하나의 지연을
  암묵적으로 삽입하지 않는다.
- 동일 시각 microstep 연쇄도 매 단계 새 authoritative snapshot, BP-16/17/32 arbitration과 BP-31 commit을
  거친다. Publication의 zero-time cycle 거부가 주 방어선이고 runtime loop guard는 invariant fail-safe로만
  사용한다.
- State는 Task 목록이 비었거나 모든 finite Task가 결과를 냈다는 이유만으로 자동 완료되지 않는다. 다음
  control transition 또는 Pattern terminal outcome은 exhaustive result arm, timer/event/predicate 등 RON의
  명시적 edge로만 발생한다.
- Pattern activation은 정상 경로에서 BP-12의 named outcome을 정확히 한 번 반환한다. 상위 lifecycle의
  cancellation과 invariant error는 outcome으로 위장하지 않으며 종료된 Pattern의 outcome mapping이나
  Task result를 뒤늦게 실행하지 않는다.
- Phase는 Pattern outcome이나 child State 종료만으로 자동 완료되지 않는다. BP-08의 authored Phase
  transition 또는 battle terminal lifecycle만 Phase를 종료한다.
- 종료된 activation을 참조하는 queued timer, Task completion과 event cursor는 identity/epoch 불일치로
  stale/no-op 처리한다. Stale event가 새 activation의 condition이나 exhaustive result arm을 만족시키지
  않는다.

## 외부 설계 조사 요약

정책 방향을 정할 때 다음 실제 게임·공식 엔진·연구 설계를 참고했다.

- Double Fine의 *Psychonauts 2* 보스 시스템은 phase별 AI 선택과 재사용 가능한 Maneuver/Maneuver
  Chain을 분리하고 telegraph, attack, recovery를 데이터 기반으로 조립했다.
  [GDC 2023 발표 자료](https://media.gdcvault.com/gdc2023/Slides/CraftingEpicBoss_Vessal_Beca.pdf)
- Bungie의 *Halo 2* AI는 hierarchical FSM/behavior DAG에서 비-leaf의 결정과 leaf behavior의 실행을
  분리하고 coherence, transparency와 rapid dithering 방지를 강조했다.
  [GDC 2005 proceeding](https://www.gamedeveloper.com/programming/gdc-2005-proceeding-handling-complexity-in-the-i-halo-2-i-ai)
- Unreal StateTree는 hierarchical state/transition과 task 실행을 분리하고 completion, failure와 event로
  state selection을 구동한다.
  [Epic StateTree 공식 문서](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-state-tree-in-unreal-engine)
- AIIDE의 boss programming model은 FSM behavior graph와 physics/health component system을 분리하고,
  여러 graph의 nested/parallel 실행으로 Zelda, Castlevania와 Sonic 계열 보스를 재현했다.
  [AIIDE 논문](https://www.algorithmicallyanimated.com/assets/papers/exag2016_boss.pdf)

이 사례를 현재 프로젝트에 그대로 복제하지 않는다. 현재 Core는 server-authoritative event simulation과
엄격한 결정론/atomicity 계약을 가지므로, 임의 tick polling과 unrestricted task concurrency 대신 typed
event, 명시적 transition 우선순위와 제한된 실행 권위를 사용해야 한다.

## 구현 순서에 대한 잠정 결론

모든 감사 리팩터링을 먼저 수행하지 않는다. 반대로 정책 없이 보스 상태 머신을 바로 구현하지도 않는다.

```text
구조를 좌우하는 핵심 정책 초안
→ read-only 코드 영향 조사
→ 구현 제약을 반영한 v1 정책 확정
→ 관련 선행 correctness/refactor
→ 실제 보스 pattern 하나를 end-to-end 구현
→ 후속 action과 pattern 확장
```

선행 기반 후보:

- 기존 route, blocking, ranged reposition과 Hard CC의 characterization test
- 같은 timestamp battle event ordering의 total/deterministic order
- battle advance/state transition의 `stage → validate → commit` 경계
- live trigger가 effect와 state transition을 누락하지 않는 실행 경로
- authored route 실행과 상위 movement intent를 분리하는 behavior-preserving seam

명령 metadata 통합, 결과 DTO 대규모 통합과 package cleanup 같은 전체 감사 리팩터링은 이 기능의 선행
조건이 아니다.

## 이후 논의가 필요한 정책 목록

현재 v1 기반 설계에서 추가로 사용자 판단이 필요한 핵심 정책은 없다. 구현 영향 조사에서 기존 runtime
계약과 충돌하거나 첫 vertical slice가 새 표현 요구를 드러내면 이 목록을 다시 연다.

## 비범위와 금지된 선행 확장

- 미래의 모든 보스 skill, summon, invulnerability와 field gimmick을 지금 일반화하지 않는다.
- arbitrary script, expression VM, unrestricted generic task framework를 선행하지 않는다.
- 구현되지 않은 future enum/action을 no-op으로 허용하지 않는다.
- 일반 침식 직원의 authored route 동작을 AI 도입을 이유로 변경하지 않는다.
- Unity가 pattern, target, route 또는 transition을 재계산하게 하지 않는다.
- 이 임시 기록이 있는 것만으로 구현 착수나 canonical 변경이 승인된 것으로 해석하지 않는다.
