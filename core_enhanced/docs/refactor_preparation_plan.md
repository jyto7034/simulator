# 리팩토링 준비 기준

이 문서는 `core_enhanced`를 리팩토링할 때 적용할 공통 판단 기준이다. 과거 구현 기록, 완료된 작업 목록, 낡은 설계 메모는 남기지 않는다. 실제 실행 목표와 실험 기록은 별도 goal 문서에 작성하고, 완료된 goal 문서는 현재 정책 문서로 필요한 지시 사항을 옮긴 뒤 삭제한다.

## 목적

리팩토링의 목적은 더 멋진 구조를 만드는 것이 아니라, 현재 게임 흐름을 더 쉽게 읽고 안전하게 바꾸는 것이다.

우선순위:

- source of truth 개수를 줄인다.
- 같은 정책을 바꾸기 위해 열어야 하는 파일 수를 줄인다.
- live 경로에서 안 쓰는 레거시와 호환 레이어를 제거한다.
- 현재 룰과 코드 계약이 충돌하면 코드 근거를 먼저 확인하고 문서를 갱신한다.
- 테스트는 내부 추상화가 아니라 실제 플레이 흐름과 client-facing 계약을 고정한다.

## 현재 기준선

리팩토링은 다음 현재형 기준을 전제로 한다.

- 공식 전투 흐름은 `DefenseRoute` 실시간 전투다.
- 공식 전투 노드는 사전 배치 replay-only 전투로 시작하지 않는다.
- Unity 클라이언트는 live battle 상태에서 배치, 철수, 수동 스킬, 후퇴, 일시정지, 재생, 배속 조작을 보낸다.
- 전투 결과 확인은 별도 결과 상태에서 완료 액션으로 맵 흐름에 복귀한다.
- 런 중 직원 정렬은 `roster_order`만 사용한다. 구형 `bench`/`field` 기반 TFT식 메타게임 배치 상태와 snapshot 루트 `bench`/`field`는 공식 계약이 아니다.
- `Frontline`, 구형 pre-combat deployment, replay-only 전투 시작, 결과 화면 후퇴 같은 과거 흐름은 되살리지 않는다.
- `Boss`는 아직 최종 전투 방식이 확정되지 않은 TODO다. 구현이 필요해지면 `DefenseRoute` 파생으로 설계하는 방향을 우선 검토한다.

## 범위 판단

기능 추가가 아니라 리팩토링일 때만 이 문서를 기준으로 삼는다.

- 게임 시스템 흐름: `GameState`, node entry/complete, shop/reward/support/headquarters, run failure, reward delivery.
- 전투 조우 연결: map node intent, encounter assignment, combat preview, battle scenario handoff.
- 실시간 전투 연결: live battle command, result state, server push mapping, Unity-facing snapshot/delta.
- 보상/장비/직원 성장/신뢰도처럼 전투 외부에서 런 흐름을 바꾸는 정책.

별도 goal로 분리할 범위:

- 이동 시스템 내부.
- 스킬 실행/타겟팅/투사체/효과 런타임 내부.
- BuffDatabase는 `GameDataBase` source of truth로 통합하는 goal이 진행 중이다. 전투 상태이상 buff와 consumable modifier는 수명주기가 다르므로 같은 runtime으로 합치지 않는다.
- Unity 클라이언트 UI/연출 구현.
- 순수 밸런스 수치 조정.

## 과도한 리팩토링 방지 기준

가장 경계해야 할 실패는 복잡도를 줄이기 위해 시작했는데, 미래 가능성을 핑계로 새 추상화와 파일 경계를 더 많이 만드는 것이다.

과도한 리팩토링 신호:

- 실제 변형이 1~2개뿐인데 `Trait`, `Policy`, `Resolver`, `Strategy` 계층을 먼저 만든다.
- 코드를 읽을 때 게임 규칙보다 추상화 이름을 먼저 이해해야 한다.
- 파일은 쪼갰지만 책임은 그대로 섞여 있어 수정하려면 더 많은 파일을 열어야 한다.
- live RON과 현재 게임 흐름에 없는 미래 기능 때문에 현재 코드가 우회한다.
- 제거 가능한 레거시를 adapter나 compatibility layer로 감싼다.
- 테스트가 게임 흐름보다 새 추상화 모양을 고정한다.
- 단순 함수나 테이블로 충분한 곳에 동적 dispatch, generic, trait object를 도입한다.
- 리팩토링 후 public API, DTO, RON schema가 이유 없이 더 복잡해진다.

판단 순서:

```text
1. 먼저 삭제할 수 있는가?
2. 삭제할 수 없다면 단순 함수로 모을 수 있는가?
3. 단순 함수가 반복되기 시작했는가?
4. 반복의 축이 실제 게임 정책으로 확인됐는가?
5. 그때만 새 타입/정책/trait를 고려한다.
```

## 작업 방식

- 문서를 무조건 따르지 않는다. 실제 코드를 읽고 더 나은 개선점이 있으면 코드 근거를 제시하고 그 방향을 우선한다.
- 정책이 모호하면 임의 확정하지 않는다. 게임 룰, 노드 흐름, 보상, 직원 성장, 전투 실패/성공 판정에 영향을 주는 결정은 사용자와 의논한다.
- 레거시는 과감하게 제거한다. 단, live 기능인지 테스트 fixture인지 먼저 확인한다.
- 현재 공식 플레이 흐름에 없는 compatibility layer, adapter, legacy test는 보존보다 삭제를 우선한다.
- 과거 설계는 필요한 아이디어만 현재 계약으로 재작성한다. 타입, fixture, 테스트 이름을 되살려 호환 계층으로 유지하지 않는다.
- 문서가 코드보다 앞서가거나 낡았으면 문서를 따르지 않는다. 실제 코드와 live RON/API를 읽어 더 단순하고 안전한 개선안을 찾고, 정책 판단이 필요한 경우 사용자에게 근거를 설명한 뒤 진행한다.
- 리팩토링 단위는 작게 잡는다. 한 번의 goal에서 source of truth 하나 또는 연결된 경계 하나만 정리한다.
- 각 단계는 `PLAN`, `EXPERIMENTS`, `NOTES` 또는 동등한 Markdown 기록을 남긴다.
- 검증은 빠른 focused test부터 시작하고, 마무리에는 관련 crate check/test를 실행한다.

## 현재 우선 점검축

대형 변경 이후 리팩토링 goal을 세울 때는 다음 축을 우선 확인한다.

- 상태 전이: `GameState`, allowed actions, snapshot `flow`, server request/result mapping이 같은 계약을 말하는가?
- 전투 진입: map node, node preview, authored encounter, battle scenario가 같은 전투 목적을 공유하는가?
- 실시간 조작: `Deploy`, `Withdraw`, `ActivateSkill`, `Retreat`, `Pause`, `Resume`, `SetBattleSpeed`, `RequestBattleState`가 core와 server에서 같은 의미인가?
- 보상/결과: 전투 성공, 전투 실패, 후퇴, 런 실패, 노드 소비, 보상 지급이 한 곳의 정책으로 설명되는가?
- 데이터 계약: live RON이 현재 runtime이 요구하는 필드를 직접 표현하고, runtime이 없는 미래 기능을 위해 우회하지 않는가?

## 문서 정리 기준

docs에는 현재 의사결정과 client-facing 계약만 남긴다.

- 완료된 goal 문서는 보관하지 않는다.
- 완료된 실험 로그, 감사 로그, 구현 후기, 과거 작업 목록은 현재 지시 사항을 옮긴 뒤 삭제한다.
- 지시 사항은 `game_rulebook.md`, `unity_core_contract.md`, `skill_target_contract.md`, `refactor_preparation_plan.md` 중 가장 가까운 source-of-truth 문서로 합친다.
- 새 goal 문서는 작업 중에만 둔다. goal이 끝나면 완료 결과를 현재 문서에 요약하고 goal 문서는 제거한다.
- "나중에 가능할지도 모르는 아이디어"는 현재 구현 지시처럼 남기지 않는다. 필요한 경우 TODO 또는 보류 정책으로 한 문단만 남긴다.

## Goal 문서 작성 기준

이 문서는 특정 시점의 추천 작업 목록을 보관하지 않는다. 리팩토링 대상이 정해지면 별도 goal 문서에 목적, 범위, 종료 조건, 실험 기록을 작성한다.

Goal 문서에는 다음을 포함한다.

- 실제 코드 판독에서 확인한 문제와 근거 파일.
- source of truth를 줄이는 구체적 종료 조건.
- 삭제 가능한 레거시와 보존해야 하는 live 계약의 구분.
- focused test, `cargo check` 같은 빠른 검증 루프.
- 문서와 코드가 충돌할 때 코드 근거를 우선하고, 정책이 모호하면 사용자와 의논한다는 기준.
- 완료 조건에 "사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다"를 포함한다.
