# Game Rulebook

이 문서는 현재 게임 규칙의 최상위 룰북이다. 플레이어가 어떤 흐름으로 런을 진행하고, 각 노드와 전투가 어떤 의미를 가지며, 보상과 손실이 언제 확정되는지를 설명한다.

세부 구현 계약은 이 문서에 모두 풀어 쓰지 않는다. 타겟팅, Unity 통신, 리팩토링 기준처럼 별도 source of truth가 있는 주제는 아래 문서를 우선한다.

- 스킬/평타 타겟팅, 범위, 자동 시전, 유효 적대 대상 판정: `docs/skill_target_contract.md`
- 전투 runtime source-of-truth, 결정론/RNG, event log, checkpoint, battle record, actor identity 구현 계약: `docs/core_runtime_contract.md`
- Unity WebSocket, snapshot, command/result 통합 계약: 외부 canonical `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- 전투 시작, live update, checkpoint, resync, 전투 종료 snapshot 흐름: 외부 canonical `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- 리팩토링, 레거시 제거, debug 산출물 분류: `docs/refactor_preparation_plan.md`
- 문서 지도와 source-of-truth 순서: `docs/README.md`

문서보다 실제 runtime code와 live RON/data가 우선이다. 문서와 코드가 충돌하면 코드를 먼저 확인하고, 게임 정책 판단이 필요한 부분은 사용자와 의논한다.

## 게임 정체성

이 게임은 환상체를 소유 기물로 쓰는 게임이 아니라, 소수의 직원을 이끌고 봉쇄된 시설을 탐사하는 로그라이크 전술 전투 게임이다.

핵심 판단은 아래 네 가지다.

- 어떤 직원과 장비, 스킬 파편으로 다음 노드에 들어갈지 정한다.
- 노드 브리핑과 위협 경고를 읽고 배치와 전투 계획을 세운다.
- `DefenseRoute` 전투 중 안정화치 기반 배치, 철수, 수동 스킬, 후퇴, 일시정지, 배속을 조작한다.
- 보상, 연구 진행도, 장비/파편 성장, 직원 손실 위험을 관리하며 다음 노드를 고른다.

현재 공식 전투 축은 명일방주식 실시간 배치 방어인 `DefenseRoute` 하나다. 보스 전투도 별도 전투 장르를 되살리기보다 `DefenseRoute` 파생으로 설계하는 방향을 우선 검토한다.

## 게임 모드

게임은 상위 구조로 `일반 모드`와 `끝없는 탐사 모드`를 가진다. 두 모드는 같은 직원, 장비, 스킬 파편, 노드, `DefenseRoute` 전투 규칙을 공유하지만, 런 길이와 보스 체인 정책이 다르다.

Runtime은 런 생성 시점부터 `GameMode`를 명시적으로 저장해야 한다. `GameMode`는 `StartNewGame` command의 필수 입력이며, 직원 선발 완료 후 생성되는 `RunProgression`에 그대로 저장된다. `GameMode`는 노드맵 생성, `Gate` 배치, 다음 Floor 전환, 최종 클리어/실패 조건의 source of truth다. 모드별 규칙을 generator나 UI 추론으로 흩어 두지 않는다.

일반 모드:

- 기본 모드이자 튜토리얼 성향의 짧은 모드다.
- 플레이어가 런 시작, 직원 선발, Floor 맵, 노드 선택, Safezone, `DefenseRoute` 전투, 후퇴/실패/보상, 장비/스킬 파편 성장 흐름을 익히는 것이 목적이다.
- 일반 모드는 3 Floor 고정 구조다.
- Floor 1~2는 엘리트 노드 클리어 후 열리는 `Gate` 노드를 통해 다음 Floor로 이동한다.
- Floor 3은 최종 보스가 최종 관문으로 등장하며, 최종 보스를 클리어하면 런 클리어다.
- 최종 보스의 패배 판정은 즉시 런 실패다. 엘리트와 일반 보스 실패는 런 실패가 아니라 해당 노드 실패로 처리한다.
- 백야, 고요한 오케스트라 같은 장기 보스 체인과 특수 보스 파편 리스크는 기본적으로 일반 모드의 핵심 콘텐츠가 아니다.
- 일반 모드에서도 스킬 파편, 장비, 연구 진행도는 사용할 수 있지만, 끝없는 탐사용 장기 징조 체인을 끝까지 밀어 파편을 얻는 구조는 다루지 않는다.

끝없는 탐사 모드:

- 장기 생존과 심화 콘텐츠를 담당하는 모드다.
- Floor가 계속 이어지며, 플레이어는 누적되는 손실과 보상을 관리한다.
- 특정 보스가 `provisional_boss`로 떠오를 수 있고, 기존 노드 위에 보스 징조 source가 얹힐 수 있다.
- 플레이어가 같은 보스 계열의 징조 source를 반복 선택하면 보스 체인이 확정되거나 진행도가 누적될 수 있다.
- 조건이 완성되면 보스 방이 생성되거나, 일부 체인은 다음 노드 선택지를 잠그는 강제 보스 방으로 이어질 수 있다.
- 다음 층 이동 source는 `Gate` 노드다. 끝없는 탐사에서는 `Gate`의 부모가 엘리트, 보스, 일반 노드 중 무엇이든 될 수 있다.
- 기본적으로 한 층에는 `Gate`가 하나지만, 특수 층에서는 여러 `Gate` 후보를 허용할 수 있다.
- 조건부/Locked Gate, 비밀 Gate, 완전 숨김 Gate는 시스템상 미래 확장 후보지만 기본 live data에 임의로 작성하지 않는다. 그런 노드는 명시 정책이 있을 때만 추가한다.
- 백야 체인, 고요한 오케스트라 같은 장기 보스 기믹, 특수 보스 파편, 고위험 보상, 누적 리스크는 끝없는 탐사 모드의 콘텐츠로 본다.

## 핵심 플레이 루프

기본 런 흐름은 아래 순서다.

```text
런 시작
-> 시작 직원 선발
-> Floor 맵 생성
-> 노드 선택
-> 노드 진입 전 확인과 준비
-> 전투/지원/정비/상점/보상/본사 연락 처리
-> 노드 결과 반영
-> 다음 노드 선택
-> 모드별 종료 조건 확인
```

일반 모드에서는 Floor 1~2의 `Gate`를 통한 다음 Floor 이동과 Floor 3 최종 보스 클리어가 핵심 진행/종료 조건이다. 끝없는 탐사 모드에서는 `Gate`를 통한 다음 Floor 이동, 보스 징조 체인, 누적 손실 관리가 핵심 진행/종료 조건을 만든다.

전투 노드는 한 번의 전투로 끝나지 않을 수 있다. 일반 전투, 엘리트, 일반 보스, 최종 보스는 같은 이상현상 시도 횟수 정책을 따르되, 최종 보스는 패배 판정이 즉시 런 실패라는 예외를 가진다. 패배 판정 전 후퇴와 재진입은 남은 시도가 있을 때 가능하다.

## 시작 직원 선발

런 시작 시 플레이어는 시작 직원 후보 중 출전할 직원을 고른다.

- 시작 직원은 이후 런 전체의 핵심 자원이다.
- 직원은 장비, 스킬 파편, 신뢰도, HP, 트라우마를 통해 장기적으로 관리된다.
- 출전 가능한 직원이 없고 회복 가능 노드도 남아 있지 않으면 런 실패다.

## Floor 맵과 노드 선택

Floor 맵은 추상적인 세로 트리라기보다, 봉쇄된 회사 내부의 방과 복도를 탐사하는 시설형 지도다. 플레이어는 보이는 노드 중 아무 곳이나 순간이동하지 않고, core가 현재 탐사 위치와 연결 관계를 기준으로 열어 준 인접/도달 가능 노드만 선택한다.

- 노드는 전투, 지원, Maintenance, 상점, 보상, 본사 연락, 보스, Gate 같은 계열을 가진다.
- 노드 선택은 단순 이동이 아니라 위험, 보상, 회복 가능성, 성장 루트를 고르는 판단이다.
- core는 run graph, node state, visibility, edge direction, 현재 탐사 위치, 선택 가능성의 source of truth다.
- Unity는 core가 내려준 node와 slot을 로컬 facility template 위에 렌더링한다. Unity가 node 순서, 이름, kind/category로 위치나 연결 가능성을 추측하지 않는다.
- 입구는 일반 노드로 표현되며, 시작 시 `Completed` 상태이자 `map_navigation.current_node_id`의 초기 위치다.
- `map_navigation.current_node_id`는 현재 탐사 위치다. 현재 위치를 `map.nodes[*].state == "Current"` 같은 별도 node state로 표현하지 않는다.
- 전투/보스 노드에 진입하기 전에는 `NodeConfirm` 단계에서 출전 직원, 장비, 스킬 파편, 소비 아이템, 예상 위협을 확인한다.
- `NodeConfirm`은 선택한 노드의 preview/confirmation 단계이며, 이 단계만으로 현재 탐사 위치를 이동시키지 않는다.
- 실제 이동과 노드 소비는 `ConfirmEnterNode` 성공 이후 core 상태 전환으로 확정된다.
- `ConfirmEnterNode`는 최소 한 명 이상의 출전 가능한 직원을 요구한다.
- 전투 시작 전 배치 가능 여부, 안정화치 비용, 배치 타입, 점유 상태, 방향은 `DeployUnit` 시점에 core가 검증한다.

Gate 노드:

- `Gate`는 다음 Floor로 이동하기 위한 독립 노드다. 엘리트나 보스 노드가 `Gate` 역할을 겸하지 않는다.
- 첫 gate 표현은 계단으로 한다.
- 각 노드는 단 하나의 category/성질만 가진다. 예를 들어 엘리트 전투 노드는 엘리트 전투 노드이고, 다음 층 이동은 그 뒤에 열린 별도 `Gate` 노드가 담당한다.
- `Gate`가 어떤 노드의 자식으로 열리는지가 Floor 구조를 결정한다. 일반 모드 Floor 1~2에서는 엘리트 노드의 자식으로 `Gate`가 열린다.
- `Gate`는 기본적으로 `Revealed`이며, `Revealed`는 보인다는 뜻일 뿐 즉시 입장 가능하다는 뜻이 아니다.
- `Gate`에 들어갈 수 있으려면 다른 노드와 마찬가지로 core가 현재 위치 기준 선택 가능성 검증을 통과시켜야 한다.
- `Gate` 확인 UI는 Unity-local UX다. Unity가 다음 Floor 이동 확인 팝업을 처리하고, 취소하면 core command를 보내지 않는다. 확인하면 core에 확정된 Gate 진입/전환 command를 보낸다.
- core는 확인 여부를 별도 상태로 저장하지 않는다. core는 확정 command를 받았을 때 `Gate`가 현재 선택 가능한지 검증하고, 전환 보상을 적용한 뒤 다음 Floor로 이동한다.
- `Gate`는 세이브 포인트가 아니다.
- Gate 전환 보상은 생존 중인 모든 직원의 현재 trauma를 `gate_transition_trauma_recovery_percent`만큼 퍼센트 감소시키는 것이다. 기본값은 10%이며 RON 정책값으로 조절 가능해야 한다.
- Gate 전환 보상은 HP를 회복하지 않고, SavePoint/checkpoint 저장 효과를 주지 않는다. 사망/실종/전투불능 직원은 대상이 아니다.
- Floor 전환 후에도 직원, HP/트라우마 같은 장기 상태, 장비, 스킬 파편, 소비 아이템, 리소스는 유지된다. Gate 전환 보상은 이 상태 유지 위에 별도로 적용된다.
- 일부 노드는 아이콘/종류만 가릴 수 있다. 이 경우 노드 자체를 숨기는 `Concealed`가 아니라 `Obscured`를 사용한다. 단, 기본 live data는 사용자가 명시하지 않는 한 `Revealed`로 작성한다.

노드 진행 상태:

- `Available`: 선택하고 진입할 수 있는 노드다.
- `Completed`: 이미 기능을 다한 노드다. 경유 지점으로는 사용할 수 있지만 같은 콘텐츠를 재실행하지 않는다.
- `Locked`: 조건이 충족되기 전까지 진입할 수 없는 노드다.
- `Unavailable`: 현재 위치와 진행 상태 기준으로 아직 선택할 수 없는 노드다.

노드 visibility:

- `Revealed`: 종류, category, state, preview 정보를 정상 표시할 수 있다.
- `Obscured`: 알 수 없는 방으로 표시할 수 있다. 종류/보상/상세는 숨길 수 있지만, `Available`이고 `map_navigation.selectable_node_ids`에 포함되면 선택할 수 있다.
- `Concealed`: 데이터에는 존재하지만 Unity에서 숨겨도 되는 노드다. 선택할 수 없다.

기본 Node Map은 모든 노드를 `Revealed`로 내려준다. `Concealed`는 비밀방, 조건부 출현 방, 특수 기믹처럼 숨김이 명시된 경우에만 사용한다. `Obscured`도 기본 discovery 상태가 아니라, 종류/보상/위협 정보만 의도적으로 가릴 때 사용하는 선택적 표현이다.

선택 가능성은 core가 계산해 Unity-facing snapshot의 `map_navigation.selectable_node_ids`로 내려준다. Unity는 `map.nodes`, `state`, `visibility`, edge 정보로 선택 가능성을 재계산하지 않는다.

```text
목적지 node.state == "Available"
목적지 node.visibility != "Concealed"
현재 위치에서 목적지까지 graph path 존재
중간 경유 노드는 모두 Completed
Locked / Concealed / Unavailable / 아직 해결되지 않은 Available 노드는 중간 경유 불가
ForwardOnly edge는 방향을 지킴
Bidirectional edge는 양방향 허용
```

`map_progression.available_node_ids`나 `completed_node_ids` 같은 중복 projection은 Unity-facing 선택 기준으로 쓰지 않는다. core 내부에서 진행 계산용 캐시를 둘 수는 있지만, public snapshot의 선택 source는 `map_navigation.selectable_node_ids`다.

연결과 이동:

- core 내부 graph/edges가 노드 사이의 연결 관계와 진행 방향을 정의한다.
- edge direction은 명시적이어야 하며, 현재 값은 `Bidirectional` 또는 `ForwardOnly`다.
- `Bidirectional`은 양방향 이동/진행을 허용한다.
- `ForwardOnly`는 `from_node_id -> to_node_id` 방향만 허용한다.
- Unity-facing Node Map DTO는 일반 런타임 UI용으로 `map.edges`와 `travel_path`를 내려주지 않는다.
- Unity는 edge를 시각화하지 않고, 경유 이동 연출도 하지 않는다. 사용자가 선택 가능한 노드를 클릭하면 해당 노드로 즉시 전환하는 연출을 사용한다.
- 완료된 노드는 transit으로 사용할 수 있다. Completed 노드는 전투, 보상, SavePoint, Rest, Maintenance, HeadquartersContact, Shop, Reward, Gate 같은 본래 기능을 다시 실행하지 않는다.
- 플레이어는 현재 위치에서 완료된 transit node를 통해 도달 가능한 `Available` 목적지를 한 번에 선택할 수 있다.
- locked node는 조건 충족 전까지 선택할 수 없으며, 원칙적으로 뒤에 자식/진행 노드가 없는 leaf room이어야 한다.

시설 지도 presentation:

- core는 `map_template_id`와 모든 node의 `slot_id`를 내려준다.
- core는 `map_navigation.current_node_id`와 `map_navigation.selectable_node_ids`를 내려준다.
- 런/Floor 시작 시 Start는 실제 node로 존재하며 `Completed + Revealed` 상태다. `map_navigation.current_node_id`는 Start node id이고, Start에서 도달 가능한 첫 Available 노드들이 `selectable_node_ids`에 포함된다.
- 현재 첫 구현은 `map_template_id = "act_01_floor_a"`와 `slot_id = depth_XX_lane_YY`를 사용한다.
- 현재 기본 snapshot에서는 모든 node visibility가 `Revealed`다.
- `entrance_lab`, `archive_room` 같은 의미형 authored slot 이름은 미래 template-authoring 후보이며 현재 live 계약이 아니다.
- Unity는 `map_template_id + slot_id`로 로컬 템플릿 좌표를 찾고, 누락된 slot을 임의 좌표로 fallback하지 않는다.
- blocked/contaminated zone, sealed corridor, 경고 overlay 같은 분위기 표현은 기본적으로 core node가 아니라 Unity template/art 표현이다. gameplay에 영향을 주는 선택/진행 대상이 될 때만 run graph node로 승격한다.

## Safe Node와 Safezone

`Safe Node`와 `Safezone`은 다른 개념이다.

`Safe Node`:

- 맵 위에서 선택하고 소비하는 비전투 노드 계열이다.
- 현재 계열은 `SavePoint`, `Rest`, `Maintenance`, `HeadquartersContact`, `Shop`, `Reward`다.
- `Support Node`는 Safe Node 중 core의 `SupportState`를 쓰는 하위 계열이며, 현재 `SavePoint`, `Rest`만 포함한다.
- `Maintenance`는 `SupportState`가 아니라 독립 `MaintenanceState`와 `selected_event.type == "maintenance"`를 쓰는 정비 노드다.

`Safezone`:

- 노드 사이에서 전투 진입을 준비하는 비소비 UX 구간이다.
- 노드를 소비하지 않고 별도 보상, Rest, Maintenance 효과를 지급하지 않는다.
- Safezone에는 `노드 탐색`, `아이템 사용`, `장비 장착/해제` 장면이 있다.
- Safezone Loadout에서는 장비와 스킬 파편을 준비할 수 있다.
- Safezone에서는 파편 강화/개화/분쇄, 장비 분쇄/강화를 실행하지 않는다. 그런 작업은 `Maintenance` 노드의 책임이다.

## Event Node

`Event` node는 비주얼 노벨식 상황, 대사, 선택지, 분기 결과를 담당하는 독립 node category다.

- Event node는 기존 Combat, Reward, Support, Shop 노드에 억지로 끼워 넣지 않는다.
- Standard와 Endless 모두 기본 Floor에는 최소 1개 이상의 Event node가 생성된다.
- Event node는 scene graph를 가진다. Core는 `event_id`, `current_scene_id`, presentation resource id, 선택지 id, 선택 고정 상태, 진행 상태의 source of truth다.
- Unity는 배경, 대사 본문, 초상, 사운드, 장면 전환 같은 presentation asset을 로컬 카탈로그에서 읽는다. Core는 asset 본문을 직접 내려주지 않고 id만 내려준다.
- Unity는 선택지 효과를 추론하지 않는다. 선택 command를 보내고, 실제 보상/패널티/전투 시작/전조 진행은 core가 실행한다.
- 선택지가 없는 scene 진행은 Unity가 `advance_event_scene` command를 보내서 진행한다. Unity는 다음 scene을 임의 계산하지 않는다.
- 선택지가 있는 scene에서는 `select_event_choice` command를 사용한다.
- 선택지를 확정하면 같은 Event node에 다시 들어가도 다른 선택지로 바꿀 수 없고, Event를 처음부터 다시 시작하지 않는다.
- 선택지 효과는 부분 적용을 허용하지 않는다. Core는 가능한 검증을 먼저 수행하고, 선택지 resolution은 전부 성공하거나 전부 실패해야 한다.
- 제작 데이터 참조 누락, event graph 불변식 위반, encounter 생성 실패 같은 core/data invariant 오류는 gameplay rejection이 아니라 panic/fail-fast 대상이다.

Event-started combat:

- Event 선택지는 `StartCombat` 효과로 기존 live battle pipeline을 시작할 수 있다.
- 이 전투는 Event node 전용 전투 시스템을 갖지 않고, 기존 combat preview, battle setup, battle update, combat result 흐름을 재사용한다.
- Event node의 category/payload를 Combat node처럼 위장하지 않는다. Core는 Event choice effect에 명시된 encounter id로 전투를 시작한다.
- Event 선택지로 시작된 전투는 일반 전투 노드와 같은 도전 규칙을 따른다.
- 승리하면 Event 선택지는 해결되고 Event node는 `Completed`가 된다.
- 패배하거나 퇴각해도 도전 횟수가 남아 있으면 Event node는 아직 `Completed`가 아니다. 선택지와 pending combat 상태는 유지된다.
- 같은 Event node로 돌아오면 이미 확정한 선택지의 후속 전투를 이어간다.
- 다른 선택지를 고르거나 Event를 처음부터 다시 시작할 수 없다.
- 패배/퇴각으로 도전 횟수를 모두 소진하면 Event node는 `Completed`가 되고 더 이상 재도전할 수 없다.
- BossOmen source가 Event-started combat을 사용해도 같은 규칙을 따른다. BossOmen step은 Event node가 `Completed`될 때만 소비된다.

## 소비 아이템

소비 아이템은 런 중 보유하는 장기 자원이다.

- 사용 가능 시점은 `ViewingMap`과 `NodeConfirm`이다.
- 전투 중, 전투 결과 처리 중, Safe Node 내부, Shop, Reward에서는 사용할 수 없다.
- 효과 적용에 대상 선택이 필요하면 core가 대상 유효성을 검증한다.
- 사용 시 duration 또는 stack이 감소한다.
- duration은 전투/보스 노드가 해결될 때만 감소한다. SavePoint, Rest, Maintenance, HeadquartersContact, Shop, Reward로는 감소하지 않는다.
- duration이 0이 되면 아이템은 인벤토리에서 제거된다.
- `consumable_use_preview`는 사용 가능 여부, 대상 후보, 효과 요약, disabled reason을 제공한다.
- Unity는 preview를 표시하되 최종 소모와 효과 판정의 source of truth는 core다.

일반 장비와 소비 아이템은 서로 다르다. 일반 장비는 장착/해제가 가능하지만, 사용된 소비 아이템은 되돌릴 수 없다.

## 노드 진입 전 브리핑

전투 노드 미리보기는 제한 자원을 소비해 업그레이드하지 않는다. 기본 브리핑은 항상 전투 진입에 필요한 최소 정보를 제공해야 한다.

기본 브리핑에는 아래 정보가 포함된다.

- 전투 모드와 임무 변형
- 전장 템플릿과 배치 가능 구역
- 주요 route 또는 예상 진입 방향
- 위험도와 보상 후보
- 주요 적 또는 핵심 위협 요약
- 위협 경고와 그 신뢰도

정확한 웨이브 수, 모든 등장 타이밍, 숨은 기믹의 세부 값은 기본적으로 전투 중 관찰과 리트라이 학습 영역으로 둔다. 단, 플레이어가 납득 가능한 배치를 할 수 없을 정도로 정보가 부족하면 별도 소비 자원 없이 기본 브리핑 자체를 보강한다.

`EnemyBriefing`은 플레이어에게 보여줄 정보이며 실제 스폰 수량/종류의 source of truth가 아니다. 실제 전투 스폰은 `SpawnWave.enemy_entries`가 담당한다.

## 전투 모드

현재 live 작성 가능 전투 모드는 `DefenseRoute` 하나다.

`DefenseRoute`:

- 명일방주식 실시간 배치 방어다.
- 플레이어 유닛은 배치 위치에 고정되고, route를 따라 들어오는 적을 저지/공격한다.
- 저지는 일시적인 거리 판정이 아니라 지속 engagement다. 한 번 저지한 적은 사망, 철수, 전투 종료, unblockable/airborne 전환, 강제 이동/텔레포트/넉백/특수 기믹의 명시 해제 전까지 유지된다.
- 일반 route 이동 중 거리 이탈만으로 저지가 해제되지 않는다. 기존 engagement는 우선 보존하고, 남은 `block_capacity`에만 새 적을 붙이며, capacity 초과 적은 통과할 수 있다.
- 철수하거나 전투불능으로 필드에서 제거된 배치 유닛은 `deployed_units`에서 빠지고 재배치 cooldown 상태로 이동한다. 철수 재배치 cooldown은 30초, 전투불능 재배치 cooldown은 90초다.
- 재배치는 기존 runtime unit을 되살리지 않고 새 `RuntimeUnit`과 새 `UnitInstanceId`를 만든다. 기존 withdrawn/dead runtime unit은 debug/admin/replay 확인을 위해 inactive 상태로 보존될 수 있지만, gameplay checkpoint actor source가 아니다.
- 철수 재배치 HP는 철수 시점의 현재 HP에 최대 HP의 30%를 더한 뒤 최대 HP로 cap한 값이다. 이 값은 `LiveBattleRedeployState`의 다음 배치 HP 정책으로 저장된다.
- 전투불능 재배치 HP는 새 runtime unit의 최종 최대 HP를 계산한 뒤 그 60%로 설정한다. 최소 1, 최대 max HP 범위로 clamp한다.
- 재배치 유닛은 이전 전투 버프, pending cast, movement goal, action lock, target, cooldown/runtime state를 상속하지 않는다.
- 철수는 상대가 발사한 incoming hostile projectile을 피하는 의미가 강한 defensive/evasion action이다. 철수하려는 유닛을 향해 이미 날아오던 hostile projectile은 target이 `Withdrawn` 상태가 되면 `BasicAttackProjectileImpacted { hit: false }` 또는 스킬 projectile no-hit으로 끝나며, `HpChanged`를 만들거나 old runtime HP/redeploy HP lock을 갱신하지 않는다.
- 성공은 모든 웨이브 종료, 필수 적 전멸, 보호 오브젝트 생존으로 판정한다.
- 실패는 보호 오브젝트 파괴 또는 시나리오가 정한 실패 조건으로 판정한다.
- 일시정지, 재생, 배속, 후퇴를 서버 시뮬레이션 기준으로 지원한다.

현재 기본 live 계약은 `DefenseRoute` 파생이다. 일반 전투는 `DefenseRoute / Defense`, 보스 전투는 `Boss / Boss` 조합으로 시작하되 같은 live battle transport와 전장/배치/웨이브 모델을 사용한다. `mission_variant`는 전투 런타임 종류를 무분별하게 늘리는 축이 아니며, 전장 아키타입은 배치/스폰/장애물/route 형태를 고르는 데이터 분류다. `Surrounded`, `Ambush`, `SplitRoom` 같은 아키타입은 자동으로 별도 전투 모드를 만들지 않는다.

생존형 전투는 `Encirclement` 같은 별도 mission variant가 아니라 `DefenseRoute / Defense`에 `survive_timer_ms`를 명시한 timer objective다. 이 경우 보호 대상이 살아 있는 상태로 `survive_timer_ms`에 도달하면 남은 적이 있어도 즉시 승리한다. 타이머 동안 발생하는 적 wave는 별도 모드 로직이 아니라 조우의 wave data와 wave pool data가 결정한다.

제거된 전투 정책:

- 과거 `DefendPoint` 누수형 방어 계약은 제거됐다.
- `Encirclement`는 공식 live mission variant가 아니다. 포위형 생존 전투가 필요하면 `DefenseRoute / Defense + survive_timer_ms`와 전장/웨이브 데이터로 표현한다.
- `SplitRoom`은 공식 live mission variant가 아니다. 분리된 방 형태가 필요하면 우선 battlefield archetype/template로 표현하고, split-squad UI나 별도 기믹이 필요해지면 새 정책으로 설계한다.
- `Recovery`/`DefendAndEscape`는 공식 전투 모드에서 내린다.
- `TacticalGroupPlan`/`GroupObjective`/포메이션 기반 group 이동 AI는 최신 공식 전투 모드에서 제거됐다.
- 필요한 콘셉트가 다시 생기면 기존 레거시를 되살리지 말고, 실제 플레이 목적과 UI가 확정된 뒤 새 계약으로 설계한다.

## 전투 시작과 Unity 계약

공식 전투 흐름은 replay-only event-log export가 아니라 live battle state를 기준으로 한다.

- 전투 시작 scene construction source는 `battle_setup_snapshot`이다.
- 전투 중 정확한 연출 타이밍 source는 `battle_update.events_delta`다.
- 전투 중 현재 상태/복구 source는 `battle_update.checkpoint`다.
- 완료된 전투의 event log는 run-local 환상체 도감/관찰용 전투 기록으로 저장한다.
- 전투 결과 화면의 1차 통계 source는 core가 산출한 `selected_event.result_stats`다.
- 전투 유닛 HUD 표시 정책과 live actor 정체성은 core가 내려주는 DTO가 source of truth다.
- 기본 공격/스킬 범위 overlay는 core가 계산한 `range_previews` 최종 cell이 source of truth다. Unity는 `defense_tile_range`, `effective_weapon_profile`, `effective_basic_attack`, skill catalog range metadata를 조합해 범위를 재계산하지 않는다.
- 범위 overlay는 이동 가능 tile 표시가 아니다. 전장 밖/void/invalid tile은 제외하지만, obstacle/blocked tile은 기본 공격과 스킬 범위 표시에서 제외하지 않는다. 장애물은 지상 이동/배치/pathfinding/충돌에 영향을 주는 정보이며, 실제 피격 가능성은 공격/스킬의 target validation, 공중 대상 가능 여부, 유효 hostile target 규칙이 판단한다.
- 침식 직원의 기본 공격 범위는 침식 직원 프로필 RON이 basic attack range preset pool에서 선택한 `TileRangePattern`이 source of truth다. core는 침식 직원이 근거리/원거리인지, `range_units`가 얼마인지, projectile을 쓰는지만 보고 범위를 임의 추론하지 않는다.
- 초기 침식 직원 range preset은 `melee_front_1`, `ranged_center_3x3`, `ranged_center_5x5`이다. `melee_front_1`은 자기 타일과 전방 1칸을 포함한다. 원거리 preset은 자기 위치 타일 중심 정사각형 범위이며 facing을 사용하지 않는다. 범위 후보는 valid tile과 교차한 결과만 사용하고, obstacle/blocked tile은 공격 범위에서 제외하지 않는다.
- 기본 공격을 위해 이동을 멈출지 여부도 최종 타일 범위와 blocker-first 규칙으로 결정한다. route enemy는 target이 tile attack range 안에 없으면 route 이동을 계속하고, `range_units`/연속좌표 거리/body radius로 기본 공격 정지 위치를 계산하지 않는다. 저지 중이면 자신을 저지한 blocker를 우선 공격하며, blocker가 사라지면 일반 tile range targeting 또는 route movement로 돌아간다.
- 적군 원거리 route 유닛은 route를 따라 강제 전진하다가 타겟을 포착하면 멈춰 공격한다. 공격 release, 즉 피해 적용 또는 투사체 생성 직후부터 기본 `ranged_reposition_ms = 1000` 동안 route 재이동을 시도한 뒤 기존 타겟을 재검증한다. 재이동 중에는 새 원거리 타겟을 탐색하지 않는다. 기존 타겟이 여전히 유효하고 범위 안이면 새 타겟 탐색 없이 다시 공격하고, 유효하지 않으면 현재 타겟팅 규칙으로 새 타겟을 찾는다. 자신을 저지하는 유닛이 있으면 재이동/기존 원거리 타겟/방어 목표보다 blocker를 우선 공격한다. `ranged_reposition_ms`는 profile/RON에서 0 이상으로 override할 수 있으며, 0이면 재이동 없이 즉시 기존 타겟을 재검증한다.
- `WholeFieldValidTiles`는 기본 공격과 스킬이 모두 사용할 수 있는 공용 range policy다. 이 정책은 route 유무와 직접 결합되지 않으며, route-following 유닛도 유닛/profile/skill이 명시적으로 허용하면 사용할 수 있다. 보스, 특수 침식 직원, 저격수 같은 특수 유닛이 사용할 수 있으며, 일반 `profile_role: Normal` 침식 직원은 사용할 수 없다. 전체 범위는 후보 타일 범위만 전체라는 뜻이며, 기본 공격은 유닛/basic attack의 `targeting_profile`, 스킬은 명시 skill targeting rule, 그리고 alive/hostile/air_capable/untargetable/유효 hostile target validation을 따른다. `WholeFieldValidTiles` 스킬에 명시 targeting rule이 없으면 validation failure다. 전장 광역 스킬은 `WholeFieldValidTiles` + `TileArea`로 표현할 수 있지만 data에 전장 광역 스킬임이 명시되어야 한다.
- 침식 직원 profile은 `profile_role` metadata를 가진다. `Normal`은 일반 wave용이며 `WholeFieldValidTiles`를 사용할 수 없다. `Special`은 저격수/특수 침식 직원용, `LegacyEcho`는 죽은 직원 이스터에그/잔향 정예용이며 고유 범위와 스킬을 가질 수 있다.
- 적군 공격 범위는 플레이어 조작 preview가 아니므로 기본 DTO로 매번 내려보내지 않는다. 보스 경고, 광역 스킬 telegraph, debug처럼 시각적 경고가 필요한 경우에만 core가 실제 피해/효과가 적용될 타일을 별도 event/DTO로 내려보낸다.
- 환상체의 `threat_class`는 RON metadata가 source of truth다. 환상체는 `Normal`이 될 수 없고 최소 `Elite`, 보스 환상체만 `Boss`다. 직원, 침식 직원, 방어 오브젝트 같은 비환상체 기본 전투 단위는 `Normal`을 사용한다.

전투 runtime source-of-truth, 결정론/RNG, event log, checkpoint, battle record, actor identity 세부 구현 계약은 `docs/core_runtime_contract.md`를 따른다. Unity-facing DTO shape, command result, battle update cadence, resync 세부 계약은 외부 canonical 문서를 따른다. 이 룰북은 플레이 규칙 요약만 유지한다.

## 전장과 배치

전투는 고정된 TFT식 상하 전선만 사용하지 않는다.

- 노드마다 전장 템플릿, route, 장애물, 배치 가능 구역, 적 출현 구역이 달라질 수 있다.
- 전투 시작 후 Unity가 scene을 구성할 때의 source of truth는 `battle_setup_snapshot.battlefield`, `routes`, `deployment_zones`, `spawn_zones`다.
- `combat_preview`는 전투 전 브리핑/미리보기 source이고, 전투 scene construction source가 아니다.
- live 전장 데이터의 authoring source는 shared RON `game_resources/data/map/battlefield_templates.ron`의 ASCII `rows`와 선택적 `routes.overlay`다.
- ASCII row의 공백은 전장 밖이다. 전장은 사각형일 필요가 없다.
- `width`는 가장 긴 row 길이, `height`는 row 개수로 계산한다.
- 공백이 아닌 타일은 `valid_tiles`가 되고, `tiles`에는 좌표와 `Ground`/`Platform`/`Obstacle` kind가 함께 노출된다.
- `P`는 지상 배치, `T`는 플랫폼 배치, `#`은 장애물, `N/L/Q/A/B/W/R`은 적 출현 구역, `X/Y/Z`는 route endpoint/전술 marker로 해석된다.
- route가 필요한 템플릿은 terrain과 같은 크기의 `routes.overlay`를 작성한다. overlay는 route의 시작/종료 marker와 방향 화살표를 표현하며, 유효 타일 위에만 놓을 수 있다.
- `BattlefieldRoute.cells`는 적이 실제로 따라가는 route path다. Unity는 `start`/`end`만 보고 경로를 재계산하지 않고, setup snapshot의 `routes[*].cells`를 그대로 scene route 표시와 이동 연출 기준으로 사용한다.
- route cells는 유효 타일 위에 있어야 하며, static obstacle과 void tile을 지나면 안 되고, 연속된 cell은 상하좌우 cardinal-adjacent여야 한다.
- authored route가 없는 Defense 템플릿은 core가 deterministic generated route를 만들 수 있다. 이 generated route도 위의 route cell 조건을 만족해야 하며, `[start, end]`만 가진 대각선/직선 fallback은 공식 계약이 아니다.
- `battle_setup_snapshot.routes`, runtime enemy movement plan, movement event/checkpoint는 같은 route cells에서 파생되어야 한다.
- 배치 입력과 사거리/스킬 범위 후보 선별은 타일 기반이다.
- 전투 이동과 충돌은 2D 연속좌표 기반이며, static obstacle과 비사각형 전장의 void tile은 이동 충돌 입력으로 투영된다.
- 전장 크기는 초기 기준으로 `Small`, `Medium`, `Large`를 사용한다.
- 보스 전장은 기본적으로 `Large`로 취급한다.

전장 아키타입은 고정 룰이 아니라 데이터 분류다.

- `OpenHall`: 넓은 홀.
- `Corridor`: 외길 복도.
- `ChokePoint`: 병목 지형.
- `Ambush`: 측면 또는 후방 출현이 있는 지형.
- `Surrounded`: 여러 방향에서 압박받는 지형.
- `SplitRoom`: 분리된 방. 현재 live 조우 배정 대상은 아니다.
- `ObstacleRoom`: 장애물이 많은 공간.
- `BossArena`: 보스 전용 공간.

전장 아키타입과 크기 기본값은 `game_resources/data/map/battlefield_archetypes.ron`에서 관리한다. 실제 배치/스폰/장애물/route 형태는 `game_resources/data/map/battlefield_templates.ron`의 템플릿과 이를 파싱한 runtime `BattlefieldInstance`가 source of truth다.

## 적 구성과 웨이브

탐사 중 만나는 적은 전부 환상체일 필요가 없다. 일반 전투와 대규모 웨이브의 주력 적은 침식된 전 탐사 직원이다.

적 분류 방향:

- `CorrodedEmployee`: 일반 웨이브, 매복, 증원에 사용하는 침식 직원.
- `Abnormality`: 보스, 정예, 특수 조우의 중심 위협.
- `FacilityEntity`: 시설 장치나 비인간형 개체. 아직 구현 대상이 아니다.

PVE encounter 분류:

- `Normal`: 순수 침식 직원 전투. `primary_abnormality_id`가 없으며 환상체 연구, 대응 완료, 환상체 파편 보상을 발생시키지 않는다.
- `Elite`: 침식 직원과 엘리트 환상체가 함께 등장하는 전투. `primary_abnormality_id`가 필요하다.
- `NormalBoss`: 침식 직원과 일반 보스급 환상체가 함께 등장하는 전투. `primary_abnormality_id`가 필요하다.
- `FinalBoss`: 침식 직원과 최종 보스급 환상체가 함께 등장하는 전투. `primary_abnormality_id`가 필요하며 Standard 최종 Floor의 run-ending 보스에 사용한다.

`primary_abnormality_id`는 조우의 연구/보너스 목표/반복 등장 기록에서 사용하는 주 환상체 식별자다. 실제로 어떤 적이 스폰되는지는 여전히 웨이브 데이터가 source of truth다.

`Normal` 조우는 `primary_abnormality_id`가 없으므로 환상체 제압 연구, 대응 완료, 환상체 대상 보너스 목표를 가질 수 없다. 순수 침식 직원 전투에 추가 보상을 주고 싶다면 환상체 연구가 아니라 일반 조우 보상으로 작성한다.

현재 구현 단위에서 `Elite`, `NormalBoss`, `FinalBoss`는 정확히 하나의 주 환상체를 가진다. 여러 주 환상체가 동시에 등장하는 조우는 추후 별도 정책으로 다룬다.

웨이브 규칙:

- 적 등장은 `SpawnWave` 기반으로 관리한다.
- live RON의 기본 전투 웨이브는 `time_ms = 5000`으로 맞춘다. 즉 필드 적은 전투 시작 직후가 아니라 5초 뒤 처음 등장한다.
- 증원, 매복, 보스 패턴은 이후 필요에 따라 다른 `time_ms`를 부여해 표현할 수 있지만, 현재 live encounter는 모든 authored wave를 5초 출현으로 통일한다.
- 현재 공식 전투 이벤트는 전투 시작 또는 `time_ms` 기반으로 발생한다.
- 특정 유닛 HP, 특정 유닛 사망, 웨이브 전멸, 지점 도달 같은 조건부 이벤트는 아직 정규 플레이 규칙이 아니다.
- `PveWaveData.required_for_victory`가 `false`인 웨이브는 승리 조건에 필수로 포함되지 않는다. 기본값은 `true`다.
- live RON의 `PveWaveData`는 `source`를 단일 적 구성 source로 사용한다.
- 수동 웨이브는 `source: Manual([...])` 안에 enemy variant를 적는다.
- 침식 직원 entry는 `CorrodedEmployee(profile_id: "...", tier: ..., count: ...)`로 표현하며 `abnormality_id`를 사용하지 않는다.
- 침식 직원 `profile_id`는 외형/스탯/타겟팅/기본 공격 범위 preset을 포함한 침식 직원 profile을 가리킨다.
- 환상체 entry는 `Abnormality(abnormality_id: "...", tier: ..., count: ...)`로 표현한다.
- `FacilityEntity`는 아직 구현 대상이 아니므로 live 웨이브에 사용하지 않는다.

침식 직원 웨이브는 수동 나열과 preset 기반 생성을 모두 허용한다. 생성 결과는 `CombatPreview` 생성 시점에 확정되어 `SpawnWave.enemy_entries`에 저장되고, 전투 중 재생성하지 않는다.

## 기본 공격과 스킬 타겟팅

타겟팅의 상위 원칙은 단순하다.

- 어떤 공격이든 deterministic expected final damage가 0보다 크면 유효 후보로 본다.
- 피해가 0이어도 선택한 적 또는 AoE 안의 적에게 적용되는 적대적 target-applied effect가 있으면 유효 후보로 본다.
- 피해도 없고 적대적 대상 효과도 없으면 자동 평타/스킬 후보에서 제외한다.
- 후보가 여럿이면 `targeting_profile`과 해당 profile의 fallback 규칙(route progress, threat, deterministic tie-breaker 등)을 따른다.
- 피해 가능 대상과 효과만 가능한 대상 사이에 별도 우선순위는 두지 않는다.

적대적 target-applied effect는 적 유닛에게 상태 변화를 시도하는 효과다. debuff, control/status, forced movement, DoT, stat/defense/resistance reduction, vulnerability, mark, aggro/threat manipulation, beneficial-effect removal, `InterruptCast`가 여기에 속한다.

타겟팅 단계에서는 효과가 이미 적용되어 있는지, 갱신/중첩 가능한지, 저항/면역되는지는 따지지 않는다. 단, 그런 제한이 스킬 target filter에 이미 표현되어 있다면 그 필터는 따른다.

스킬/평타 범위, cast target, `RetargetOnStep`, `TileArea`, 자동 적대 타겟 유용성의 세부 source of truth는 `docs/skill_target_contract.md`다.

## 피해 타입과 피드백

`Physical`, `Magic`, `True` 피해는 빌드 색깔과 효율 차이를 만드는 축이다. UI/기획 표현에서 AD는 `Physical`, AP는 `Magic`에 대응한다.

- 특정 일반/정예 적이 한 피해 타입을 영구 무효화해 반대 타입 딜러 보유 여부만 검사하는 구조는 피한다.
- 방어력/마법 저항이 높은 적은 등장할 수 있지만, 대응 수단은 반대 타입 딜러 하나로 잠그지 않는다.
- 관통, 저항 감소, 고정 피해, 제어, 저지, 소비 아이템, 배치/스킬 타이밍 같은 우회 수단을 열어둔다.
- 보스는 일시 보호막, 기믹 방어, 특정 타이밍 무효를 가질 수 있다.
- 보스도 영구적인 단일 타입 면역으로 빌드 전체를 부정하지 않는다.

core는 피해 결과를 계산한 뒤 Unity-facing battle event에 `feedback_tags`를 내려준다. Unity는 방어력/마법 저항 세부 수치를 재계산하거나 추론하지 않는다.

초기 표시 태그:

| tag | 의미 |
| --- | --- |
| `critical` | 치명타 |
| `mitigated` | 큰 폭으로 감소한 피해 |
| `fixed_damage` | 고정 피해 |
| `immune` | 피해 또는 상태가 완전히 막힘 |

`piercing`, `shield`, `blocked`, `resisted_status`는 공식 피해 피드백 태그가 아니다. 관통 projectile, Shield 무기 아키타입, 이동 저지 같은 별도 시스템 용어와 피해 숫자 피드백 태그를 섞지 않는다.

피해량, 사망 여부, HP 변화의 source of truth는 `HpChanged.delta`, `hp_before`, `hp_after`, `final_damage`다.

## 보상과 성장

전투 목적은 어떤 보상 태그를 강조할지 결정하고, 실제 지급은 보상 효과가 담당한다.

기본 보상 태그:

- `Currency`
- `Equipment`
- `SkillFragment`
- `ResearchProgress`

모든 전투 보상은 현재 기본 정책에서 자동 지급한다. live combat encounter의 `reward_mode`는 `ClaimAll`이어야 하며, `ChooseOne` 전투 보상은 validation failure다. 선택형 보상은 아직 정규 전투 결과 흐름에 넣지 않는다.

PVE 전투 보상은 낡은 무작위 완성 장비 보상에 의존하지 않는다. 전투 조우는 장비 가루, E.G.O 잔재, 연구 진행도, 낮은 확률의 스킬 파편 후보를 조합해 보상 풀을 구성한다.

환상체 진압 보상은 환상체 본체를 지급하지 않고, 해당 환상체에서 파생된 독립 스킬 파편, E.G.O 잔재, 설계도, 연구도, 낮은 확률의 완제품 후보를 조합한다.

## 연구 완료와 본사 송신

스킬 파편 연구도 보상은 파편 자체를 즉시 지급하지 않고, 해당 파편의 `research_progress`를 올린다. 연구도는 개화 진행도와 별개의 본부 분석/블랙박스 회수 진행도다.

기본 정책:

- 연구 진행도가 완료 기준에 도달하면 pending 연구 완료 항목으로 등록한다.
- pending 항목은 다음 Safe Node 진입 또는 노드 결과 처리 시 본사 송신으로 자동 수령한다.
- 다음 노드가 전투/보스라면 수령하지 않고 계속 보류한다.
- pending 연구 완료 수령은 Safe Node의 행동권을 소모하지 않는다.
- 인벤토리 스냅샷은 보유 파편 목록과 별도로 `skill_fragment_progress`, `pending_research_deliveries`를 노출한다.

이 정책은 추후 변경 가능해야 한다. 예를 들어 “Support에서만 수령”, “Maintenance에서만 수령”, “Shop 제외” 같은 변경은 데이터/정책 레이어에서 처리하고, 연구 진행도와 파편 인벤토리 구조를 다시 바꾸지 않는다.

## 끝없는 탐사 환상체 대응 연구

끝없는 탐사에는 스킬 파편 연구도와 별개의 런 단위 환상체 대응 연구가 있다.

환상체 대응 연구는 계정 영구 도감이 아니라 현재 런의 진행 상태다. 저장/체크포인트가 런 상태를 보존할 때 함께 보존되며, 끝없는 탐사의 완료 조건과 반복 조우 보상 판단에 사용한다.

대상:

- 엘리트 환상체.
- 일반 보스 환상체.
- 최종 보스 환상체.

비대상:

- 도구형 환상체.
- 침식 직원.
- 시설 entity.
- 일반 웨이브 적.

기본 규칙:

- 각 대상 환상체는 `research_points`, `research_required`, `response_complete`, `suppression_wins`, `last_encountered_floor`, `completed_at_floor`, `unique_fragment_granted`를 가진다.
- runtime source는 `research_points`와 `research_required`다. 퍼센트 값은 UI 표시용 파생값이며, 저장/판정 source로 쓰지 않는다.
- 승리 시 live RON `run/policy.ron`의 환상체 연구 정책에 따라 기본 연구도가 증가한다.
- 모든 밸런스 수치는 live RON에서 조절한다. 코드에 고정된 수치로 보상량, 연구 요구량, 반복 등장 가중치, bonus objective 보상을 결정하지 않는다.
- 현재 기본값은 `default_research_required = 100`, `victory_research_gain = 20`, 반복 제압 파편 가루 `elite = 5`, `boss = 10`, `final_boss = 20`이다.
- 연구도는 요구량을 넘지 않고 cap 된다.
- 처음 대응 완료가 되는 순간 환상체 metadata의 `response_complete_skill_fragment_id`가 가리키는 스킬 파편을 1회 지급한다.
- 이미 대응 완료된 동일 환상체를 다시 제압하면 고유 스킬 파편을 중복 지급하지 않고, threat class별 live RON 수치에 따라 파편 가루를 지급한다.
- 끝없는 탐사는 모든 대응 대상 환상체가 `response_complete`가 되면 완료된다.

`RunSnapshotDto`는 현재 런의 대응 연구 상태를 `abnormality_research`로 노출한다. Unity는 이 필드를 진행도/도감/결과 UI 표시용으로 읽을 수 있지만, 보상 지급 여부는 core command/result 처리 결과를 따른다.

반복 조우:

- 동일 환상체는 같은 Endless run 안에서 다시 등장할 수 있다. 승리는 그 조우에서 몰아낸 것이지, 해당 환상체가 세계에서 영구 제거됐다는 뜻이 아니다.
- 일반 non-omen 후보 선택에서는 최근 `1 Floor` 안에 등장한 동일 환상체를 기본 후보에서 제외한다.
- 최근 `1 Floor` 제외는 해당 환상체가 대응 완료 상태인지 여부와 무관하게 적용한다.
- 대응 완료된 환상체는 일반 non-omen 후보 가중치를 낮춘다. 현재 live RON 기본값은 `response_complete_weight_multiplier_percent = 25`다.
- 미완료 환상체는 일반 후보 가중치를 유지한다. 현재 live RON 기본값은 `incomplete_weight_multiplier_percent = 100`이다.
- soft repeat 조건 때문에 후보 pool이 비면 `empty_pool_fallback = ResetSoftRepeatConstraints` 정책으로 최근 등장 제외, 대응 완료 가중치 감소, 반복 등장 억제만 리셋한다. hard constraint를 깨거나 존재하지 않는 encounter를 만들지 않는다.
- Boss omen chain, 강화/각성 버전, 최종 보스 조건은 일반 non-omen 반복 가중치를 의도적으로 우회할 수 있다.
- `last_encountered_floor`는 조우가 실제로 연구/결과 처리에 반영된 시점의 기록이다. 단순 후보 선정이나 미입장 노드 preview만으로 등장 이력을 갱신하지 않는다.

보너스 목표:

- `PveEncounter.suppression_research.bonus_objectives`가 명시된 encounter만 bonus objective를 가진다. 전역 fallback 목표를 암묵적으로 붙이지 않는다.
- bonus objective가 있는 encounter는 연구 대상 `primary_abnormality_id`가 있어야 한다. `Normal` encounter는 `primary_abnormality_id`가 없으므로 대응 연구와 bonus objective 대상이 아니다.
- bonus objective는 승리한 전투 결과에서만 평가한다. 퇴각, 패배, 노드 실패는 bonus objective 연구 보상을 지급하지 않는다.
- 각 objective는 전투당 최대 1회만 만족 처리한다. 여러 objective가 만족되면 연구 보상은 합산된다.
- 평가 시점은 전투 중 mutable state가 아니라 완료된 battle event log와 battle result stats를 기반으로 combat result 생성 시점에 수행한다.
- 총 연구 증가량은 `victory_research_gain + sum(satisfied_bonus_objectives[*].research_bonus)`다. bonus objective가 없으면 기본 승리 연구도만 적용한다.
- 환상체 부위는 별도 inventory item이 아니다. 강한 제압/특수 조건을 표현하는 presentation fiction이며, 실제 효과는 연구 진행도 증가다.

`ClearWithin(time_ms)`:

- 전투 시작 시간을 기준으로 `victory_battle_time_ms - battle_start_ms <= time_ms`이면 만족한다.
- 보스가 늦게 등장하는 encounter에는 이 목표를 넣지 않는다. 늦게 등장하는 보스용 시간 조건이 필요하면 별도 objective를 새로 설계한다.

`DecisiveDamage(minimum_damage_percent_of_max_hp)`:

- 한 공격 원천의 실효 피해가 대상 최대 체력의 지정 비율 이상이면 만족한다.
- 판정은 정수 비교로 한다. 공식은 `single_source_effective_damage * 100 >= target_max_hp * minimum_damage_percent_of_max_hp`다.
- `single_source_effective_damage`는 한 번의 기본 공격 impact, 스킬 step impact, projectile impact, hitscan impact, 단일 비-DoT tile area impact처럼 하나의 공격 원천에서 발생한 실효 피해다.
- DoT 피해는 제외한다. 작은 반복 tick을 합산해 DecisiveDamage로 만들지 않는다.
- primary abnormality에게 들어간 피해만 계산한다. 잡몹, 시설 entity, 방어 오브젝트 피해는 이 목표를 만족시키지 않는다.
- 마지막 일격일 필요는 없다.

## 장비

장비는 일반 장비와 E.G.O 장비로 나뉜다.

장비 슬롯:

- 무기
- 방어구
- 악세서리

무기는 기본 공격 타입, 사거리, 공격 범위 패턴, 공중 공격 가능 여부, 기본 저지 보정, 기본 `targeting_profile`을 제공한다. 방어구는 생존력과 저지 역할을 강화한다. 악세서리는 보조 스탯, 유틸리티, 특수 조건부 효과를 제공한다.

장비 강화:

- 장비 강화는 장비 가루와 강화 자원을 소비해 장비 성능을 올린다.
- 강화 레벨은 장비 정의가 아니라 `OwnedEquipment` 인스턴스에 저장한다.
- 같은 장비 id라도 개별 인스턴스마다 강화 상태가 다를 수 있다.
- 전투 시작 시 직원이 장착한 장비 인스턴스의 강화 레벨을 `BattleScenario` 유닛 draft에 스냅샷으로 복사한다.
- 전투 중에는 그 스냅샷을 기준으로 스탯을 계산한다.
- 장비 제작은 현재 live Maintenance action이 아니다. 필요하면 `CraftEquipment` 같은 별도 개념으로 새로 설계한다.

스킬 파편 경제와 장비 경제는 분리한다. 스킬 파편을 분쇄해 얻은 파편 가루로 장비를 강화할 수 없고, 장비를 분쇄해 얻은 장비 가루로 스킬 파편을 강화할 수 없다.

## 스킬 파편

기본 직원은 고유 액티브 스킬을 갖지 않는다. 직원은 스킬 파편을 장착해야 대표 액티브 스킬을 사용할 수 있다.

기본 정책:

- 액티브 스킬 파편 슬롯은 기본적으로 1개를 전제로 한다.
- 환상체가 쓰는 원본 스킬과, 직원이 파편으로 모방하는 스킬은 별도 스킬 데이터다.
- 사용하지 않는 파편은 강화, 개화, 분쇄, 연구, 제한 조합 같은 자원 순환으로 활용한다.
- 파편 분쇄는 `Maintenance`에서 수행한다.
- 스킬 파편은 패시브 장비가 아니다.
- 평타 강화, 일시 스탯 증가, 조건부 버프, 공명 변화는 상시 효과가 아니라 액티브 스킬 발동 결과로 표현한다.
- 전투, 전투 결과 처리, 보상 처리 중에는 전투 결과를 보고 즉석으로 파편을 바꾸지 않는다.

스킬 파편 장착 조건:

- 스킬 파편은 직원의 고정 직업명을 요구하지 않는다.
- 스킬 파편은 장착 중인 무기가 만든 전투 프로필을 기준으로 장착 가능 여부를 판단한다.
- 근거리 전용 파편과 원거리 전용 파편은 허용한다.
- 모든 파편을 모든 무기 아키타입에 억지로 대응시키지 않는다.
- 강한 환상체 정체성을 가진 파편일수록 요구 조건을 좁게 둔다.
- 범용 강화/보조 파편은 요구 조건을 느슨하게 둔다.
- 장착 가능 여부의 최종 source of truth는 core validation이다.

콘텐츠 설계와 live placeholder 목록은 `docs/skills/` 아래 문서를 함께 본다.

## 지원 노드

지원 노드는 Safe Node의 하위 계열이다. core에서는 `SupportState`와 `selected_event.type == "support"`로 표현한다.

현재 사용하는 지원 효과:

- `SavePoint`
- `Rest`

지원 노드 공통 규칙:

- 한 지원 노드의 최종 효과는 하나다.
- `Random` 지원 노드는 사용하지 않는다.
- `Supply`, `Communications`, `Containment`, `Intel`은 현재 지원 노드 정규 효과로 사용하지 않는다.
- 대상이 없거나 효과가 없어도 노드는 소비된다.
- 지원 노드는 Safezone이 아니다.

SavePoint:

- `SavePoint`는 런 체크포인트 / 세이브 포인트 노드다.
- 과도한 스트레스를 줄이는 완충 장치지만, 만능 복구 장치가 아니다.
- 별도 전용 화면 없이 노드 완료 후 Safezone으로 복귀한다.
- 완료 시 현재 런 체크포인트를 자동 저장한다.
- 저장 슬롯은 런당 단 하나이며, 새 SavePoint checkpoint가 이전 checkpoint를 덮어쓴다.
- 불러오기 시점은 마지막으로 SavePoint를 통과한 시점이다.
- 플레이어는 임의로 checkpoint를 불러올 수 있다.
- 런당 불러오기 횟수는 최대 3회다.
- 불러오기는 Safezone `Config` / 설정 메뉴에서 제공한다.
- 불러오기 command는 `load_run_checkpoint`이며, snapshot의 `allowed_actions`에 `LoadRunCheckpoint`가 있을 때만 실행 가능하다.
- snapshot의 `run_checkpoint`는 checkpoint 존재 여부, 사용 횟수, 최대 횟수, 남은 횟수, 현재 load 가능 여부를 제공한다.
- 완료 시 생존 직원만 트라우마가 자동 감소한다.
- 트라우마 감소량은 10~20% 범위이며, 1차 기본값은 15%다.
- 사망/실종/전투불능 직원은 트라우마 감소 대상이 아니다.
- HP 회복은 SavePoint의 목적이 아니다.

Rest:

- 살아있는 직원 전체에게 적용된다.
- 트라우마를 소량 회복한다.
- HP는 회복하지 않는다.
- `Rest` 효과는 노드 내부에서 `complete_node`를 선택할 때 적용된다.
- Safezone의 휴식풍 연출이나 다음 노드 준비 화면은 이 `Rest` 효과를 자동 적용하지 않는다.

## Maintenance 노드

Maintenance는 지원 노드가 아니라 독립 Safe Node다.

역할:

- 정비 작업 공간이다.
- 주 기능은 스킬 파편/장비의 분쇄, 강화, 개화다.
- 하단 작업 버튼은 `분쇄`, `강화`, `개화` 3개만 둔다.
- 스킬 파편을 분쇄하면 파편 가루를 획득한다.
- 무기, 방어구, 악세서리를 분쇄하면 장비 가루를 획득한다.
- 스킬 파편 강화는 파편 가루를 소비한다.
- 무기, 방어구, 악세서리 강화는 장비 가루를 소비한다.
- 개화는 1차 정책에서 스킬 파편 전용 작업이다. 무기, 방어구, 악세서리 선택 시 개화는 disabled 처리한다.

운영 규칙:

- 장착 중인 스킬 파편과 장착 중인 장비도 분쇄할 수 있다.
- Unity가 경고창/확인 단계를 제공하고, core는 확인된 command로 보고 자동 해제 후 분쇄한다.
- Maintenance 내부에서도 장비/스킬 파편 장착 교체를 할 수 있다.
- 이는 주 기능이 아니라 정비 중 편의를 위한 부가 기능이며, Loadout과 동일한 호환/교체 validation을 따른다.
- 플레이어가 노드 완료를 선택하기 전까지 여러 정비 행동을 반복할 수 있다.
- 노드 완료를 선택하면 Maintenance 노드는 소비되고 다음 맵 진행으로 돌아간다.
- Safezone의 Loadout과 구분한다.

Unity가 비용, 결과, 가능 여부를 직접 계산하지 않도록 Maintenance snapshot은 선택 가능 대상별 작업 가능 여부, disabled reason, 비용 preview, 획득 preview, before/after preview, 확인 필요 여부, 자동 해제 여부를 제공해야 한다.

## 본사 연락 노드

본사 연락 노드는 런 중 본부와 연결되는 별도 Safe Node다.

- 기존 `SavePoint`, `Rest`, `Maintenance`와 역할이 다르므로 별도 노드 계열로 다룬다.
- 연구 완료 파편 자동 수령은 본사 연락 행동권을 소모하지 않는다.
- 본사 연락은 추후 정보 확인, 연구 송신, 특수 지시 수령 같은 별도 선택지로 확장할 수 있다.

## 노드 결과 처리

전투, 방어, 회수 노드는 성공/실패와 별개로 하나의 결과 처리 화면을 가진다.

권장 결과 처리 순서:

```text
노드 종료
-> 전투 결과 산출
-> 직원 HP / 트라우마 / 전투불능 / 사망 위험 반영
-> 자동 보상 지급
-> 연구 진행도 증가 반영
-> 연구 완료 항목 pending 등록
-> 결과창 표시
-> 플레이어 확인
-> 맵으로 복귀
```

결과창은 최소한 아래 정보를 보여준다.

- 임무 성공/실패.
- 직원별 HP, 트라우마, 전투불능, 부상/출혈 등 상태 변화.
- 주요 전투 기록.
- 자동 지급 보상.
- 연구 진행도 변화.
- pending 연구 완료 항목.

전투 결과 확인은 `CompleteCombatResult` 하나의 성공 단위로 처리한다. 직원 사후 상태 반영, 전투 소비 아이템 duration 감소, 자동 보상 지급, 연구 진행도/pending 등록, 현재 맵 노드 완료, 세션 정리, `ViewingMap` 복귀는 모두 함께 성공하거나 모두 커밋되지 않아야 한다. 중간 검증 실패나 local invariant 불일치가 발생하면 XP, 재화, 장비, 파편, 연구 진행도, 소비 아이템, 맵 진행도 같은 user-visible state를 부분 적용하지 않는다.

`combat_result` snapshot에서 `CompleteCombatResult`는 단순히 `game_state_context.type == "combat_result"`라는 이유만으로 노출하지 않는다. core가 현재 전투 결과 세션, active combat content, battle UUID, current map node, map progression 완료 가능성을 로컬에서 확인할 수 있을 때만 `allowed_actions`에 포함한다. Unity는 결과 확인 버튼 활성화 source로 `allowed_actions`를 사용하고, 버튼을 눌렀을 때 반환되는 `invalid_action`을 정상적인 결과 완료 흐름으로 기대하지 않는다.

전투/방어/회수 실패는 곧바로 런 실패가 아니다. 실패한 노드는 연구 진행도와 핵심 보상을 지급하지 않는다. 단, 전투에 참여한 직원 경험치처럼 직원 성장 요소 일부는 지급할 수 있다.

전투 계열 노드 도전 정책:

- 일반 전투, 엘리트, 일반 보스, 최종 보스는 첫 진입을 포함해 총 3번까지 도전할 수 있다.
- 전투 진입 시 도전 횟수 1회를 소모한다. 실패/퇴각 후 재도전 기회는 최대 2번이다.
- 일반 전투, 엘리트, 일반 보스는 실패하거나 퇴각해도 남은 도전 횟수가 있으면 같은 노드를 재도전할 수 있다.
- 일반 전투, 엘리트, 일반 보스가 3번의 도전을 모두 소진하면 해당 노드는 `Completed` 처리된다. 이후 어떤 방법으로도 재도전할 수 없고, 시설 맵에서는 이동 경유 노드로만 남는다.
- 실패/퇴각으로 소진 완료된 전투 노드는 성공 보상, 핵심 보상, 스킬 파편 연구 진행도를 지급하지 않는다.
- 최종 보스는 패배 판정이 발생하는 순간 런이 완전 실패한다.
- 최종 보스도 패배 판정 전 퇴각은 가능하다. 남은 도전 횟수가 있으면 재도전할 수 있고, 퇴각으로 3번의 도전을 모두 소진하면 런 실패다.

## 후퇴

후퇴는 전투를 이득으로 바꾸는 버튼이 아니라, 배치 실수나 전력 부족을 전투 중 확인했을 때 장기 손실을 줄이는 손절 수단이다.

기본 정책:

- 후퇴는 `BattleEnd` 또는 패배 판정이 발생하기 전까지 전투 계열 노드에서 가능하다.
- 특정 강제 이벤트 전투는 명시 정책으로 후퇴를 막을 수 있다.
- 후퇴 가능 시점은 `BattleEnd`가 발생하기 전까지다.
- 일시정지 중, 스킬 시전 중, 웨이브 진행 중에도 후퇴할 수 있다.
- 후퇴를 선택하면 전투는 즉시 중단된다. 도전 횟수는 이미 전투 진입 시점에 소모된 것으로 본다.
- 남은 시도 횟수가 있으면 노드는 소비되지 않는다.
- 플레이어는 Safezone/NodeConfirm으로 돌아가 로드아웃, 소비 아이템, 출전 직원, 배치 계획을 조정한 뒤 재진입할 수 있다.
- 일반 전투, 엘리트, 일반 보스에서 후퇴로 3번째 시도까지 모두 사용되면 해당 노드는 `Completed` 처리되고 이동 경유 노드로만 남는다.
- 최종 보스에서 후퇴로 3번째 시도까지 모두 사용되면 런 실패다.
- 후퇴한 진입 시도는 핵심 보상, 스킬 파편 연구 진행도, 임무 성공 보상을 지급하지 않는다.
- 후퇴 자체는 직원을 전투불능 처리하지 않는다.

## HP, 전투불능, 런 실패

HP 정책:

- `Run HP`는 런 전체에 유지되는 직원의 실제 체력이다.
- `Battle HP`는 전투 노드에서만 쓰는 전투용 체력이다.
- 전투 시작 시 `기본 전투 컨디션 60% + Run HP / Max HP 비율 40%`를 기준으로 `Battle HP`를 산출하고, 여기에 트라우마 임계치까지 남은 비율을 곱한다.
- 전투 종료 후 생존자의 남은 `Battle HP`는 `Run HP`에 그대로 되쓰지 않는다.
- 전투불능 시에만 조정 가능한 정책값에 따라 `Run HP`가 감소하고 트라우마가 증가한다.

런 실패:

- 살아있는 직원이 한 명도 없으면 런 실패다.
- 출전 가능한 직원이 없고, 선택 가능한 회복 가능 노드도 없으면 런 실패다.
- 출전 가능한 직원이 없어도 마지막 run checkpoint를 불러올 수 있는 횟수가 남아 있으면 플레이어는 checkpoint 복구를 선택할 수 있다.
- `SavePoint`는 HP를 회복해 출전 불가 직원을 되살리는 노드가 아니다.
- 출전 가능한 직원이 없는 상태에서 전투/보스 노드 진입을 확정하려 하면 진입이 차단된다.
- `Rest`는 HP를 회복하지 않으므로 HP 0 직원을 출전 가능 상태로 되돌리는 회복 노드로 보지 않는다.
- `Maintenance`는 정비 노드이므로 HP 회복 가능성으로 보지 않는다.
- 최종 보스 전투 패배는 남은 회복 경로와 무관하게 런 실패다.

## 직원 신뢰도

신뢰도는 기본적으로 전투 수치보다 기억, 대사, 판단, 반응의 기준이다.

- 신뢰도는 선택형 부가 시스템이 아니라 직원 운용과 감정선을 구성하는 핵심 시스템이다.
- 일반 전투 조작을 자주 방해하지 않는다.
- 위험 선택, 조기 개화, 위험 파편, 부상 재출전 같은 고위험 상황에서만 강하게 반응한다.
- 기본 구현 방향은 내러티브 중심이다.
- 명령 거부와 전투 수치 보정은 별도 정책으로 제한한다.
- 신뢰도 변화 계산은 중앙 정책/Resolver에서 처리한다.
- 이벤트, 지원 노드, 스킬 파편 코드가 직접 점수를 중복 계산하지 않는다.

## 고급/후속 정책

아래 항목은 현재 룰북에 방향만 남기고, 구현 시 별도 정책/계약으로 확정한다.

끝없는 탐사 보스 전조:

- 보스 전조 체인은 끝없는 탐사 모드에서 최종 보스급 조우를 예고하고 확정하는 run-local 장기 신호다.
- Standard mode는 보스 전조 체인을 핵심 진행 구조로 사용하지 않는다.
- 동시에 활성화될 수 있는 보스 전조 체인은 하나다.
- 보스 전조 source는 현재 mode가 Endless이고, 현재 Floor가 live RON의 최소 Floor 이상이며, 플레이어가 개화/awakened 상태의 skill fragment를 1개 이상 보유하고, 활성 체인이 없을 때 등장 후보가 된다.
- 전조 후보는 threat class만으로 자동 결정하지 않는다. 환상체 metadata는 `omen_chain_id`로 사용할 BossOmenChain을 명시할 수 있고, `omen_chain_id`가 있는 보스/환상체만 전조 후보가 된다.
- `omen_chain_id`는 boolean flag가 아니라 별도 BossOmenChain 정의를 가리키는 데이터 연결 키다.
- BossOmen step은 고정 node category가 아니다. 각 step은 live RON에서 `source_kind`를 가지며, 초기 source kind는 `Event`와 `Combat`이다.
- `Event` source step은 Event node 위에 덧씌운다.
- `Combat` source step은 일반 Combat node 위에 덧씌운다.
- Gate, Boss, FinalBoss, Start, Maintenance는 기본 overlay 대상이 아니다. Support, Shop, Reward, HeadquartersContact 기반 전조는 보스별 특수 기믹이 필요할 때 별도 정책으로 추가한다.
- 보스 전조 source는 새 node를 추가하지 않고 기존 node 위에 overlay된다. Overlay된 node는 기존 slot, graph connectivity, state, 선택 구조를 유지하되 표시와 source content가 보스 전조 step으로 바뀐다.
- 현재 Floor에 필요한 `source_kind`를 덧씌울 node가 없으면 다른 source_kind로 fallback하지 않고, 부적절한 node에 강제로 overlay하지 않는다. 선택된 provisional boss와 예정 step은 유지하고 다음 Floor로 미루며, core는 QA가 원인을 추적할 수 있도록 chain/step/source/floor 정보를 로그로 남긴다.
- Boss omen source는 선택형이다. 플레이어가 첫 source를 무시하고 Gate로 다음 Floor에 진입하면 provisional boss는 boss pool에 다시 push 예약되고 체인은 확정되지 않는다.
- 플레이어가 boss omen source를 해결하면 해당 boss chain이 확정된다.
- 확정된 chain은 모든 omen step이 소비될 때까지 다음 Floor마다 같은 chain의 omen source를 계속 배치한다.
- Boss omen step은 node에 진입한 순간 소비되지 않는다. 해당 omen node의 선택지/전투/결과 처리가 끝나고 node가 `Completed`가 될 때 소비된다.
- Omen chain이 boss 조건을 만족하면 다음 node selection 단계에서 별도 동적 Boss node가 현재 위치 앞에 생성/고정된다. 이 node는 기존 Gate나 다른 node를 덮어쓰지 않는다.
- Forced boss node가 생성되면 다른 node는 선택할 수 없고 boss node만 선택 가능하다. 단, Safezone/loadout 정비는 가능하다.
- 전조 노드와 보스 노드는 다르다. 전조 노드는 보스 노드를 열기 위한 선행 흐름이고, 전조 체인 이후 생성된 보스 노드는 별도 보스 노드다.
- Forced BossOmen boss node에서 패배하면 run failure 상태가 된다. SavePoint rollback은 core 자동 복구가 아니라 Unity가 남은 rollback 횟수에 따라 표시하는 사용자 선택이다.
- Forced BossOmen boss node에서 승리하면 RON에 명시된 보상/연구/파편/파편 가루/아이템 결과를 따르고, 해당 boss omen chain은 종료된다.

필드 위험요소:

- 필드 위 위험요소는 적 기물이 아니라 전장 요소로 분리한다.
- 예시는 가스 누출, 침식 장판, 함정 장치, 오작동 격리문, 환청 방송 등이다.
- 현재는 구현하지 않는다.
- 전장 생성기와 전투 런타임이 안정된 뒤 `BattlefieldHazard` 같은 별도 타입으로 추가한다.

레거시 이스터에그:

- 이전 런에서 플레이어가 진심으로 돌본 직원이 사망하면, 조건을 만족할 때 최대 3개까지 레거시 기록으로 저장할 수 있다.
- 아무 사망 직원이나 저장하지 않고, 신뢰도, 성장, 장비, 파편, 출전 기록 같은 애착/투자 기준을 통과한 직원만 후보가 된다.
- 관련 환상체 전투 노드에서 설치/계정 단위 1회성 특수 선택지 `잔향 추적`이 나타날 수 있다.
- 이 시스템은 일반 반복 콘텐츠가 아니라 희귀한 기억 이벤트다.
- 현재 구현 우선순위가 아니다.

## 레거시 제거 원칙

새 정책과 충돌하는 레거시는 compatibility layer나 dual schema로 보존하지 않는다.

- 구형 `timeline_delta`/`battle_delta` 전송을 다시 만들지 않는다.
- 제거된 전투 모드와 RON wrapper는 필요하다는 이유만으로 부활시키지 않는다.
- 과거 test expectation만 맞추기 위해 최신 정책과 다른 동작을 유지하지 않는다.
- 레거시 동작이 정말 필요하면 이유, 제거 조건, 사용자 확인을 문서에 남긴 뒤 제한적으로만 허용한다.
