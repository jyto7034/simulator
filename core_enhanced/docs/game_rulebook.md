# 게임 룰북

이 문서는 현재 게임을 플레이 가능한 규칙 단위로 정리한 룰북이다. 실제 플레이 예시는 `gameplay_flow_example.md`, Unity 연동 계약은 `unity_core_contract.md`, 리팩토링/문서 정리 원칙은 `refactor_preparation_plan.md`를 따른다.

## 게임 목적

이 게임은 환상체를 소유 기물로 쓰는 게임이 아니라, 소수의 직원을 이끌고 봉쇄된 시설을 탐사하는 로그라이크 전술 전투 게임이다. 현재 공식 전투 축은 명일방주식 실시간 배치 방어인 `DefenseRoute` 하나로 수렴한다.

핵심 목표:

- `DefenseRoute`에서는 전투 전 정보 해석과 전투 중 배치/철수/수동 스킬/후퇴/배속 조작이 중요하다.
- 직원은 HP, 트라우마, 장비, 스킬 파편, 성장, 신뢰 기억을 가진 지속 캐릭터다.
- 환상체는 핵심 조우, 정예/보스 위협, 보상 원천이다.
- 일반 웨이브의 주력 적은 침식된 전 탐사 직원이다.
- 플레이어는 모든 직원을 두루 키우기보다 소수 엘리트에게 자원을 집중한다.
- 노드 선택, 정비, 회복, 파편 강화, 리트라이 학습이 장기 리스크 관리로 연결되어야 한다.

## 핵심 전제

- 플레이어는 환상체를 소유해 전투 기물로 쓰지 않는다.
- 플레이어의 전투 유닛은 직원이다.
- 직원은 런 동안 유지되는 HP, 트라우마, 장비, 스킬 파편, 성장 상태를 가진다.
- 환상체는 적, 조우 대상, 보상 원천이다.
- `DefenseRoute`는 실시간 배치 전략이다.
- 따라서 전투 전 정보 해석, 배치, 장비, 스킬 파편 선택을 기본 판단으로 두되, `DefenseRoute`에서는 전투 중 안정화치 기반 배치/철수와 수동 스킬 발동도 핵심 판단으로 둔다.

## 런 진행

```text
새 런 시작
-> 시작 직원 후보 제시
-> 직원 3명 선발
-> Act 맵 생성
-> 연결된 노드 선택
-> 노드 미리보기
-> Safezone 진입 전 준비
-> 노드 진입 확정
-> 노드 해결
-> 보상/피해/성장 반영
-> 다음 노드 선택
-> Act 보스 격파
-> 다음 Act 또는 런 종료
```

## Safe Node와 Safezone 용어

비전투 흐름에서 `Rest`, `Maintenance`, `휴식`, `휴식 정비`, `support`라는 말이 섞이면 실제 노드 효과와 진입 전 준비 장면이 혼동된다. 공식 용어는 아래처럼 분리한다.

`Safe Node`:

- 맵 위에 존재하는 비전투 노드 계열이다.
- 방문 후 완료하면 노드가 소비된다.
- `Medical`, `Rest`, `Maintenance`, `HeadquartersContact`, `Shop`, `Reward`가 현재 Safe Node 계열이다.
- `Support Node`는 Safe Node 중 core의 `SupportState`를 쓰는 하위 계열이며, 현재 `Medical`, `Rest`만 포함한다.
- `Maintenance`는 `SupportState`가 아니라 독립 `MaintenanceState`와 `selected_event.type == "maintenance"`를 쓰는 정비 노드다.

`Safezone`:

- 맵 노드가 아니라 다음 노드에 들어가기 전 나타나는 준비 장면/패널이다.
- 노드를 소비하지 않고, 별도 보상이나 Rest/Maintenance 효과를 지급하지 않는다.
- Safezone에는 `노드 탐색`, `아이템 사용`, `장비 장착/해제` 세 장면이 있다.
- `노드 탐색`은 다음 노드 preview, 맵 탐색, 진입 후보 확인을 담당한다.
- `아이템 사용`은 인벤토리와 사용 아이템을 보여주는 장면이다. 하단바 좌측 버튼으로 `노드 탐색` 장면과 오갈 수 있다.
- `장비 장착/해제`는 하단바 독립 버튼이 아니라, `아이템 사용` 장면에서 유닛을 길게 눌렀을 때 열리는 loadout 장면이다. 여기서 장비 장착/해제와 스킬 파편 장착/해제를 다룬다.
- 현재 계약에서 live 연결 가능한 준비 행동은 사용 아이템 적용, 장비 장착/해제, 스킬 파편 장착/해제, 인벤토리 확인, 다음 노드 preview 확인이다.
- `휴식 정비` 화면은 Safezone의 `장비 장착/해제` 장면 표현으로 본다. 여기서는 loadout 조정만 가능하다.
- `Maintenance` 노드에서만 가능한 파편 강화/개화/분쇄, 장비 분쇄/강화는 Safezone에 노출하지 않는다.
- `Maintenance` 내부에서도 장비/스킬 파편 장착 교체를 편의 기능으로 제공할 수 있다. 단, 이는 Maintenance의 주 기능이 아니라 정비 작업 중 대상 정리를 돕는 부가 기능이며, Loadout과 같은 호환/교체 규칙을 사용한다.
- `아이템 사용` 장면에서 섭취 아이템을 직원에게 드래그해 다음 전투/보스 노드용 버프를 적용할 수 있다.

## 섭취 아이템

섭취 아이템은 쉬운 클리어를 위한 단순 도핑보다 다음 노드의 손실과 트라우마 리스크를 줄이는 준비 자원이다.

정책:

- 사용 가능 시점은 `ViewingMap`과 `NodeConfirm`이다. 전투 중, 전투 결과 처리 중, Safe Node 내부, Shop, Reward에서는 사용할 수 없다.
- Safezone 내부 장면 전환은 Unity UX 상태다. core command는 `UseConsumableItem`만 추가한다.
- 소비 시점은 전투 진입이 아니라 드래그앤드롭으로 `UseConsumableItem` command가 성공한 순간이다.
- 드래그앤드롭이 성공하면 즉시 owned consumable이 inventory에서 제거되고 대상 직원의 `active_consumable_modifier`가 갱신된다. 사용된 아이템은 노드 취소, 퇴각, 재진입, modifier 덮어쓰기 상황에서도 환불되지 않는다.
- 대상은 살아 있고 출전 가능한 직원 1명이다.
- 직원 1명은 active consumable modifier를 1개만 유지한다. 새 아이템을 쓰면 기존 modifier는 환불 없이 덮어쓴다.
- duration은 `NextCombatNode` 또는 `CombatNodes(n)`이다.
- duration은 전투/보스 노드가 해결될 때만 감소한다. Medical, Rest, Maintenance, HeadquartersContact, Shop, Reward로는 감소하지 않는다.
- 같은 이상현상에서 퇴각 후 재진입하는 것은 새 전투 노드 해결로 보지 않는다. 남은 시도 횟수가 있는 한 적용된 modifier는 같은 이상현상 재진입에도 유지된다.
- 퇴각 없이 실패하거나, 성공하거나, 3번의 시도를 모두 사용해 이상현상이 사라지면 해당 전투 노드가 해결된 것으로 보고 duration을 감소시킨다.
- 전투 상태이상 `Buff`는 poison/stun/freeze/silence처럼 BattleCore 안에서 ms/tick 단위로 처리되는 효과다. 섭취 아이템 효과는 직원에게 저장되는 런/다음 전투 `ConsumableModifier`이며 전투 buff runtime과 합치지 않는다.
- consumable modifier 적용 순서는 전투 시작 profile 보정, 배치 비용 보정, 전투 중 스킬/행동, 전투불능 후 Trauma/Run HP 감소 보정, DeathPrevent 판정, 전투/보스 노드 완료 후 duration 감소다.
- 섭취 아이템의 `DeployCostReduction`은 배치 코스트 감소만 의미한다. 시작 안정화치 증가, 안정화치 회복량 증가, 재배치 비용 완화는 별도 효과로 확정하기 전까지 포함하지 않는다.
- 섭취 아이템의 `InitialSkillCharge`는 전투 시작/배치 시 해당 직원의 스킬 게이지(`resonance`)를 퍼센트 기반으로 충전하는 효과다. 기본 상한은 Common 20%, Uncommon 40%, Rare 70%, Critical/Forbidden 100%다.
- `Common`/`Uncommon`은 트라우마, 전투불능, 피해 완화 중심이다.
- `Rare`/`Critical`은 강한 방지 효과와 제한적 공격력 증가를 담당한다.
- `Forbidden`은 부작용/침식/신뢰도 정책이 확정되기 전까지 live pool에서 제외한다.
- 기본 획득처는 본사 연락에서 `Common` 및 낮은 확률 `Uncommon`, 일반 Shop에서 `Uncommon` 및 낮은 확률 `Rare`, 희귀 상점/고위험 보상에서 `Rare` 및 낮은 확률 `Critical`이다.

장비 해제와 귀속:

- Safezone Loadout에서는 일반 장비를 자유롭게 해제할 수 있다.
- 귀속 여부는 owned instance가 아니라 장비 metadata의 `bound` flag가 source of truth다.
- `bound=false` 장비는 해제 가능하다.
- `bound=true` 장비는 해제할 수 없고, Unity-facing snapshot은 `can_unequip=false`와 `cannot_unequip_reason`을 노출한다.
- 같은 장비 id가 획득 경로에 따라 귀속/비귀속으로 갈라지지 않는다. 특수 귀속 장비가 필요하면 별도 장비 id를 사용한다.

## Act 맵 생성

- Act 맵은 시각적 시작점 `Start`에서 첫 선택 행으로 이어지는 구조다.
- `Start`는 플레이어가 해결해야 하는 노드가 아니라 이미 완료된 시각적 출발점이다.
- 실제 선택 가능한 첫 노드는 `Start`의 outgoing에 연결된 depth 1 노드들이다.
- 맵 생성은 행을 먼저 무작위로 채우는 방식이 아니라, 여러 개의 경로를 먼저 긋고 그 경로 위의 격자점을 노드로 승격하는 path-first 방식이다.
- 각 경로는 다음 depth로 이동할 때 lane을 좌/유지/우 중 하나로 움직이며, 행마다 최소 노드 수를 보장한다.
- 각 행은 전투 노드 과밀을 막기 위해 전투 노드 수를 행 너비의 절반 이하로 보정한다.
- 각 비보스 행에는 최소 하나의 전투 노드를 둔다. 전투가 완전히 없는 행이 반복되면 탐사 압박이 사라지기 때문이다.
- 보스 직전 행에는 최소 하나의 지원 노드를 보장한다.
- 이 정책은 “전투만 빽빽한 맵”이 아니라, 전투/지원/본사 연락/보상/상점이 섞인 탐사 경로를 만들기 위한 기본 규칙이다.

## 본사 연락 노드

본사 연락 노드는 런 중 본부와 연결되는 별도 Safe Node다. 기존 `Medical`, `Rest`, `Maintenance` 지원 노드와 역할이 다르므로 별도 노드 계열로 다룬다.

정책:

- 한 번 방문하면 하나의 행동만 선택할 수 있다.
- 선택지는 직원 충원, 긴급 구호품 요청, 본사 보급 구매다.
- 연구 완료 파편은 Safe Node 도착 시 자동 수령 정책을 유지하며, 본사 연락 행동권을 소모하지 않는다.
- 런 중 채용 후보는 시작 직원 후보와 분리된 데이터에서 제공한다.
- 본사 보급 구매는 기본 구급품과 저등급 재료 중심이다.
- 일반 상점 노드는 무너진 회사 폐허 속 수수께끼의 상인 컨셉으로 유지한다. 일반 상점은 등장 확률이 낮고, 고가치/고밸류 상품을 판매한다.

현재 core 계약:

- 맵 노드 카테고리는 `HeadquartersContact`다.
- 노드 payload는 `HeadquartersContact(shop_pool_id, candidate_count)`다.
- 선택지 enum은 `RecruitEmployee`, `RequestEmergencySupplies`, `OpenHeadquartersShop`다.
- 런 중 채용 후보 source of truth는 `game_resources/data/employees/recruitment_candidates.ron`이다.
- 본사 보급 상점 pool 기본값은 `headquarters_basic_supplies`다.
- 긴급 구호품 기본값은 소량 엔케팔린 지급이다. 구급품/저등급 재료 지급은 밸런스 단계에서 확장할 수 있다.

```text
본사 연락 노드 진입
-> 직원 충원 / 긴급 구호품 / 본사 보급 구매 중 하나 선택
-> 선택한 행동 해결
-> 노드 완료
```

## 시작 직원 선발

- 새 런을 시작하면 본부가 시작 직원 후보 5~7명을 제시한다. 현재 기본 구현은 6명 후보다.
- 플레이어는 후보 중 3명을 선발해 이번 탐사팀을 구성한다.
- 시작 후보 목록의 source of truth는 shared RON `game_resources/data/employees/starter_candidates.ron`이다.
- 모든 시작 직원은 기본적으로 액티브 스킬을 갖지 않고, 기본 평타 강화 파편만 가진다.
- 후보는 이름, 역할, 짧은 배경, 등급/성장 방향을 통해 서로 다른 인상을 가져야 한다.
- 시작 선택은 비용 최적화보다 “이번 런에서 누구에게 애정을 쏟을지”를 정하는 단계다.
- 고코스트/저코스트 구매식 시작은 일반 모드 기본값으로 쓰지 않는다. 직원이 상품처럼 느껴지고, 초반부터 효율 계산이 과해질 수 있기 때문이다.
- 장기적으로 끝없는 탐사나 고급 모드에서는 시작 자원 기반 고용 규칙을 별도 모드 정책으로 추가할 수 있다.

초기 구현 기준:

- 후보 수: 6명
- 선발 수: 3명
- 시작 파편: 기본 평타 강화 파편
- 시작 자원: 선발 완료 후 지급
- Act 맵 생성: 선발 완료 후 생성

## 노드 선택과 진입

- 플레이어는 현재 연결된 노드 중 하나를 선택할 수 있다.
- 노드를 선택해도 즉시 진입하지 않는다.
- 노드 선택은 `NodePreview`를 생성하고, 게임은 `NodeConfirm` 상태가 된다.
- 전투/보스 노드의 `NodeConfirm` 상태에서는 해당 노드 전장, route, 배치 가능 구역, 주요 위협을 확인한다. 직원 배치는 전투 시작 후 live command로 수행한다.
- 장비와 스킬 파편은 Safezone 또는 `NodeConfirm` 같은 노드 진입 전 준비 구간에서 조정할 수 있다.
- `ConfirmEnterNode`를 선택하면 실제로 노드에 진입한다. 전투 노드는 이 시점에 즉시 소비되지 않고, 이상현상 시도 상태를 만든다. 노드 소비 시점은 노드 종류와 결과 정책을 따른다.
- 전투/보스 노드의 `ConfirmEnterNode`는 진입 전에 출전 가능한 직원이 있는지와 조우 데이터가 유효한지만 검증한다. 배치 좌표 검증은 전투 중 `DeployUnit` 명령을 처리할 때 수행한다.
- `CancelSelectedNode`를 선택하면 노드를 소비하지 않고 맵으로 돌아간다.
- 공식 전투 흐름은 replay-only 타임라인 export가 아니라 `InBattle` live state와 battle event log delta push를 기준으로 검증한다. 코드/JSON에 남은 `Timeline`, `timeline_delta` 이름은 현재 “전투 사건 로그”를 뜻한다.

## 전투 조우 배정

맵의 전투 노드 종류는 단순 표시가 아니라 실제 조우 풀을 고르는 기준이다.

- 일반 전투 노드(`combat_monster`)는 현재 act와 depth에 맞는 낮은/중간 난이도 조우를 우선 배정한다.
- 정예 전투 노드(`combat_elite`)는 일반 전투보다 높은 난이도 조우를 우선 배정한다.
- 보스 노드(`boss_abnormality`)는 `CombatNodeType::Boss` 조우를 우선 배정하고, 보스 조우가 없을 때만 가장 높은 난이도 조우로 fallback한다.
- 같은 난이도 범위 안에 여러 조우가 있으면 맵 노드 의도에 맞는 `DefenseRoute` 임무 변형을 우선한다. 예를 들어 정예 노드는 일반 방어보다 `DefenseRoute / Encirclement` 같은 고압박 조우를 먼저 볼 수 있다.
- 조우 배정은 seed, act, depth, lane, node id를 기반으로 재현 가능해야 한다.
- 이 정책은 “초반에 무작위 고위험 조우가 튀어나오는 문제”와 “정예 노드가 일반 노드와 다르지 않은 문제”를 막기 위한 최소 장치다.

## 전투 노드 기본 브리핑

모든 전투/보스 노드는 진입 전 기본 브리핑을 제공한다.

기본 브리핑은 무료이며, 노드 선택만으로 확인할 수 있다.

기본 브리핑에 포함되는 정보:

- 전장 형태 또는 전장 템플릿.
- 아군 배치 가능 구역.
- 주요 적 또는 핵심 위협.
- 대략적인 적 출현 구역.
- 위험 구역, 장애물, 특수 지형.
- 위험도와 예상 피해.
- 보상 후보.

기본 브리핑의 제한:

- 모든 적의 정확한 시작 위치를 항상 공개하지 않는다.
- 잡몹, 증원, 일부 변칙 조건은 불확실성으로 남을 수 있다.
- 단, 플레이어가 납득 가능한 배치를 할 수 있을 정도의 정보는 항상 제공해야 한다.

## 전투 모드 원칙

전투는 여러 목적 타입을 무작정 늘리지 않고, 플레이어가 체감하는 조작 규칙을 기준으로 `DefenseRoute` 단일 모드에 집중한다.

```text
CombatMode
- DefenseRoute
```

`DefenseRoute`는 명일방주식 실시간 배치 전략 전투다. 전투 중 시간이 흐르고, 플레이어는 `공간 안정화치`를 사용해 직원을 배치/철수/재배치하며, 수동 스킬과 전술 판단으로 웨이브와 기믹에 대응한다. 명일방주의 기본 규칙은 큰 틀에서 따른다.

`공간 안정화치`는 단순 코스트가 아니다. 환상체, 침식체, 잔류 E.G.O 반응이 머무른 공간은 좌표와 정신 보호 프로토콜이 불안정하다. 본사는 현장 공간을 안정화하고 직원의 투입/귀환 좌표를 붙잡을 수 있는 만큼만 순차 투입을 허가한다. 시간이 흐르면 현장 좌표가 조금씩 안정되어 안정화치가 회복되고, 일부 스킬 파편은 해당 이상 현상을 해석하는 동조 키로 작동해 공간 안정화치를 추가 회복할 수 있다.

전투 모드와 임무 변형은 분리한다.

```text
DefenseRoute MissionVariant
- Defense
- Encirclement
- SplitRoom
- Boss(TODO)
```

`Defense`는 블랙박스, 관측 장치, 격리 기록 매체 같은 고정 목표를 보호하는 전투다. 방어 대상이 파괴되면 임무가 실패한다.

`Encirclement`는 포위된 상태에서 생존이 우선인 DefenseRoute 변형이다. 방어 대상 없이 여러 route에서 압박이 들어오며, 일정 시간 생존하고 최종 웨이브를 정리하는 방향을 기본값으로 한다.

`SplitRoom`은 DefenseRoute의 고급 변형이다. 한 방에서는 본대가 전투를 유지하고, 다른 방에서는 제어장치나 기믹을 관리한다. 제어 실패는 즉시 패배보다 본대 전체 디버프와 추가 웨이브를 먼저 유발하고, 반복 실패 시 패배하는 방향을 기본값으로 한다.

보스는 당장 별도 CombatMode로 확정하지 않는다. 보스는 TODO로 남기며, 향후 실시간 기믹 파훼가 중요한 `DefenseRoute` 파생 전투로 설계할 가능성이 높다.

기본 정보 공개 정책:

- DefenseRoute는 route 전체, 방어 목표 위치, 배치 가능 타일, 예상 적 계열을 전투 전에 공개한다.
- DefenseRoute의 정확한 적 수, 웨이브별 출현 시간, 특수 적 상세, 기믹 발동 조건은 일부 불확실성으로 남길 수 있다.
- 정보가 부족해서 억울하게 패배하는 상황은 피하고, 숨겨진 정보는 전투 중 대응 가능한 형태로만 사용한다.

실패 정책:

- 비보스 DefenseRoute 실패는 즉시 런 실패가 아니다.
- 전투 노드는 이상현상이다. 탐사팀은 이상현상에 진입해 임무를 해결하고 빠져나오며, 같은 이상현상에는 최대 3번까지 진입을 시도할 수 있다.
- 전투 중 완전 실패 조건이 확정되기 전에는 미션 퇴각을 선택할 수 있다. 퇴각은 현재 진입 시도 1회를 소모하지만, 남은 시도 횟수가 있으면 이상현상 노드는 아직 소비되지 않는다.
- 퇴각 없이 임무 실패 조건이 확정되면 해당 이상현상 노드는 즉시 소비된다.
- 3번의 시도가 모두 사용되면 이상현상은 스스로 사라지고 노드는 소비된다. 이 경우 임무 성공 보상과 연구 진행도는 지급하지 않는다.
- 핵심 보상과 연구 진행도는 지급하지 않는다.
- 전투불능 직원은 전투 후 트라우마와 런 HP 피해를 받는다.
- 전투 HP는 매 전투 새로 생성되는 runtime HP다. 시작 전투 HP는 `(기본 전투 컨디션 60% + 런 HP 비율 40%) × 트라우마 임계치까지 남은 비율`로 산출한다.
- 생존자의 전투 중 HP 손실은 런 HP로 되돌려 쓰지 않는다. 런 HP/트라우마 변화는 전투불능 같은 장기 손상 사건에서만 적용한다.
- 출전 가능한 직원이 더 이상 없고 다음 선택 가능한 노드에 치료/기사회생 루트가 없을 때 런 실패가 된다.

보상 정체성:

- DefenseRoute/Defense는 연구자료, 분석 데이터, 파편 복원 진행도처럼 본사 지식 축적과 연결한다.
- DefenseRoute/Encirclement는 높은 위험을 감수한 만큼 희귀 강화 재료, 추가 연구 보상, 위기 극복형 보상을 제공할 수 있다.
- DefenseRoute/SplitRoom은 직원 운용, 신뢰도, 트라우마, 특수 장비 설계도와 연결하기 좋은 고급 임무로 남긴다.
- 저위험/성장/자원 수급 전투는 짧고 단순한 `DefenseRoute / Defense` 조우가 담당한다.
- 보스 보상은 TODO다. 고급 E.G.O 장비, 대량 연구도, 상위 분석 해금의 주된 출처가 될 가능성이 높다.

이 보상 정체성은 방향성이다. 스킬 파편 완제품은 고가치 보상이므로 모든 전투 성공에서 확정 지급하지 않는다. 대신 환상체 장비, 연구도, 파편 복원 재료, 낮은 확률의 파편 후보를 조합해 빌드 형성 속도를 조절한다.

설정 노트:

- 방어형 전투의 대표 임무는 죽은 관리자의 블랙박스 회수다.
- 블랙박스는 이전 관리자의 몸속에 심어진 데이터 기록 장치이며, 사망 후 탐사 구역에 남겨질 수 있다.
- 본사는 블랙박스 안의 녹화 데이터, 명령 기록, 환상체 접촉 로그를 회수하기 위해 신규 관리자에게 현장 방어를 지시한다.
- 플레이어는 본사가 블랙박스를 회수하는 동안 지정 지점 또는 회수 장치를 방어한다.
- 블랙박스 회수에 성공하면 완성 스킬 파편을 바로 지급하기보다, 해당 기록과 연결된 특정 스킬 파편의 연구 진행도를 증가시킨다.
- 이 구조는 방어형 전투를 단순 생존전이 아니라 스킬 파편 분석과 본사 지식 축적의 주요 경로로 만든다.
- 현재 core 기본값에서 `CombatNodeType::Defense` 전투는 별도 작성 전술이 없으면 `black_box_anchor` 지점 근처에 `black_box_device` 보호 오브젝트를 주입한다.
- 이 보호 오브젝트는 플레이어가 임의로 이동/배치할 수 없고, 움직이지 않으며, 공격하지 않고, 스킬도 사용하지 않는다.
- 보호 오브젝트는 전투 타임라인에서 `role: defense_object`로 스폰된다. 클라이언트는 이를 일반 직원/아군 전투원이 아니라 고정 방어 목표 UI로 표시한다.
- `DefenseRoute / Defense`는 명일방주식 고정 방어 전투로 확정한다. 기존 제한 반경 자동전투형 방어는 공식 방어 모드로 사용하지 않고, 포위 생존은 `DefenseRoute / Encirclement` 변형이 담당한다.
- 방어형 전투에서 아군은 배치한 위치에 고정된다. 전투 중 자율 이동이나 즉시 위치 변경은 허용하지 않지만, 후퇴 후 재배치 대기 시간이 지나면 증가한 안정화치 비용으로 다시 배치할 수 있다.
- 현재 core 기본값에서 `CombatNodeType::Defense` 아군은 `FixedDefense` 이동 정책을 사용한다. 이 정책은 본대 전진/집결 같은 그룹 이동 목표를 무시하고, 사거리 안에 들어온 적에게만 공격 목표를 만든다.
- 직원은 배치 허용 타입을 가진다. 기본 계약은 `GroundOnly`, `PlatformOnly`, `Any`이며, 기본 직원은 `GroundOnly`와 저지력 1을 가진다. 지상 배치 직원만 저지할 수 있다.
- 플랫폼 배치 직원은 적 이동 경로를 막지 않고 공격/스킬만 수행한다. 플랫폼 타일은 적 이동 가능 타일로 취급하지 않는다.
- 방어형 적은 지정된 `route_id`를 따라 이동한다. route 끝은 누수 지점이 아니라 보호 오브젝트 접근/공격 지점이다.
- 적이 저지되지 않고 route 끝에 도달하면 보호 오브젝트를 공격 대상으로 삼는다. 보호 오브젝트가 어떠한 이유로든 파괴되면 임무 실패다.
- 방어형 전투의 성공 조건은 모든 웨이브 종료, 필수 적 전멸, 보호 오브젝트 생존이다.
- 현재 core 기본 Defense 승리 조건은 시간 생존형이 아니라 `ProtectUnit`이다. 기본 Defense는 필수 적이 모두 정리되고 보호 오브젝트가 살아 있으면 성공한다.
- 명일방주식 라이프 누수 모델은 사용하지 않는다. 관문을 통과한 적 때문에 별도 라이프가 감소하는 전투가 필요해지면 현재 Defense 계약과 분리된 leak-runner 임무로 새로 설계한다.
- 저지는 `block_radius` 기반 논리 상태로 판정한다. `block_capacity`가 남은 지상 직원이 `blockable` 적을 반경 안에서 붙잡는다.
- 전투 중 이동 유닛끼리의 좌표 겹침은 허용한다. 적끼리, 적과 아군, 저지 중인 유닛끼리도 좌표상 겹칠 수 있다.
- 아군 배치 위치 겹침은 금지한다. `DeployUnit` 시점에 이미 점유된 배치 좌표는 `position_occupied`로 거절한다.
- 유닛 간 물리 충돌은 길막과 저지의 source of truth가 아니다. 저지와 통과 여부는 `block_state` 같은 명시 전투 상태가 결정한다.
- 동시 저지 우선순위는 route progress가 큰 적 우선이다. route progress는 해당 적이 지정 route를 따라 방어 목표/종료 지점에 얼마나 가까이 진행했는지를 뜻한다.
- route progress 동률은 먼저 spawn된 적, 그래도 동률이면 unit id 순으로 deterministic 처리한다.
- 여러 지상 직원이 같은 적을 저지할 수 있으면 가장 가까운 직원이 저지한다. 동률이면 deterministic id 순으로 처리한다.
- 저지 용량을 초과한 적은 해당 직원을 통과한다. 초과 적은 유닛 충돌 때문에 뒤에 영구 정체되지 않고 route 진행을 우선한다.
- 저지된 적은 저지자를 우선 공격하고, 저지자는 자신이 저지한 적을 우선 공격한다.
- 저지는 적 사망, 저지자 사망/전투불능, 강제 이동/넉백, `block_radius` 이탈 시 해제된다.
- 현재 core의 `FixedDefense` 저지 런타임은 물리 충돌 결과가 아니라 `BattleCore`의 명시 상태다. 저지 가능한 직원과 저지 가능한 적을 매 movement tick 전에 deterministic하게 매칭하고, 저지되지 않은 route 적은 주변 직원과 우발 교전하지 않고 경로 진행을 우선한다.
- Unity는 같은 좌표에 겹친 유닛을 표시 전용 fan-out, sorting, 체력바 offset으로만 보정한다. 이 보정은 실제 판정 좌표, 저지, 공격 범위, 피해 판정을 바꾸지 않는다.
- `DefenseRoute`의 평타 사거리와 액티브 스킬 범위는 타일 패턴 기반으로 판정한다. 전투 시뮬레이션 좌표계와 투사체/충돌/피해 처리는 계속 2D 연속좌표를 사용하지만, 대상 후보 선별은 타일 패턴으로 제한한다.
- `DefenseRoute`에 배치되는 직원은 배치 시 `Up`, `Right`, `Down`, `Left` 중 하나의 방향을 반드시 선택한다. 배치 후 방향은 변경할 수 없고, 방향을 바꾸려면 후퇴 후 재배치해야 한다.
- 범위 패턴은 `Up` 방향 기준 ASCII로 작성한다. `@`는 시전자 중심 anchor 타일이며 정확히 하나만 허용한다. `X`는 영향을 받는 타일, `.`은 범위에 포함되지 않는 빈 타일이다.
- `@` 타일은 기본적으로 anchor 전용이며 범위에 포함하지 않는다. 자기 타일도 범위에 포함해야 하는 스킬/공격은 `include_anchor_tile: true` 옵션을 명시한다.
- 타일 좌표 회전 기준은 `Up = y - 1`, `Down = y + 1`, `Left = x - 1`, `Right = x + 1`이다.
- 범위 판정은 대상의 연속좌표 중심점을 타일로 투영한 뒤, 배치 방향에 맞게 회전된 패턴의 `X` 타일 안에 들어오는지 확인한다. 이후 `nearest`, `lowest_hp` 같은 기존 타겟팅 규칙은 필터링된 후보 안에서만 적용한다.
- `DefenseRoute`에서 플레이어가 배치하는 유닛의 기본 공격과 장착된 액티브 스킬은 타일 범위 패턴을 가져야 한다. 패턴이 없는 경우 continuous `range_units`로 조용히 fallback하지 않고 데이터/설정 오류로 처리한다.
- 이 타일 범위 정책은 공식 전투 모드인 `DefenseRoute` 전용이다. 전투 시뮬레이션 백엔드는 계속 연속좌표/Rapier2D를 사용하지만, 플레이어 유닛의 공격/스킬 대상 후보는 타일 범위 패턴으로 제한한다.
- 스킬 RON 작성 시 반복되는 타일 범위는 `range_presets`로 분리할 수 있다. preset은 데이터 작성 편의 기능이며, 런타임 정책의 source of truth는 해석된 `defense_tile_range`다.
- 과거 `DefendPoint` 누수형 방어 계약은 live/runtime/data 계약에서 제거됐다. 관문, 탈출로, 침투 저지처럼 “적이 특정 지점에 도달하면 실패”하는 전투가 필요해지면, 현재 방어 오브젝트 계약을 우회하지 말고 별도 leak-runner 임무 계약으로 새로 설계한다.
- 현재 live RON에는 `defend_black_box_relay`가 첫 방어형 조우로 존재한다. 이 조우는 ChokePoint 전장에서 본사 회수 장치가 블랙박스 기록을 추출하는 동안 침식 직원 웨이브를 막는 구조이며, 보상은 장비 회수와 초기/특정 파편 연구 진행도에 맞춰져 있다.
- `Recovery`/`DefendAndEscape`는 공식 전투 모드에서 내린다. 회수/탈출 컨셉이 필요하면 `DefenseRoute / Defense`, `DefenseRoute / SplitRoom`, 또는 비전투 보상/이벤트 노드로 재해석한다.
- `Suppression`과 사전 배치 자동전투형 진압은 공식 전투 모드에서 내린다. 일반 진압/섬멸도 짧은 `DefenseRoute / Defense` 조우로 재해석한다.
- `Encirclement`는 별도 자동전투 모드가 아니라 `DefenseRoute`의 포위 생존 변형이다. 여러 route에서 압박을 받고, 일정 시간 생존한 뒤 최종 웨이브를 정리하는 방향을 기본값으로 한다.
- live 작성 가능 전투 모드는 `DefenseRoute` 하나로 둔다. `Boss`는 TODO이며 높은 확률로 `DefenseRoute` 파생 전투로 설계한다.

## 스킬 파편 획득 정책

스킬 파편은 직원 빌드의 핵심을 바꾸는 고가치 보상이다. 따라서 환상체를 한 번 진압했다고 완성 스킬 파편을 항상 지급하지 않는다.

스킬 파편 획득 경로:

- `확률 획득`: 환상체 진압, 고위험 전투, 희귀 보상에서 완성 스킬 파편을 낮은 확률로 획득할 수 있다.
- `연구 진행도 획득`: 노드 진행을 통해 특정 스킬 파편의 연구 진행도를 올리고, 진행도가 충족되면 본부가 해당 스킬 파편을 지급한다.

기본 흐름:

```text
환상체 조우 / 관련 전투 노드
-> 확률적으로 완성 스킬 파편 획득 가능
-> 임무 성공 시 연구도, 분석 데이터, 파편 복원 진행도 획득
-> 본부 분석 진행
-> 연구 진행도 충족 시 스킬 파편 지급
```

정책 의도:

- 완성 스킬 파편의 희소성과 기대감을 유지한다.
- 확률 운이 나빠도 플레이어가 장기적으로 원하는 파편에 접근할 수 있게 한다.
- `DefenseRoute` 노드를 파편 성장 루프에 연결한다.
- 본부는 단순 배경이 아니라 데이터를 분석하고 결과물을 보급하는 조직으로 기능한다.
- 일반 모드에서는 스킬 파편 시스템을 맛보게 하고, 끝없는 탐사에서는 장기 빌드 성장을 지원한다.

즉시 드랍은 운 좋은 빠른 획득이고, 연구 진행도는 안정적인 장기 획득 루트다.

### 연구 완료와 본사 송신

연구 진행도가 임계치에 도달해도 완성 스킬 파편을 전투 직후 즉시 지급하지 않는다.

기본 정책:

- 연구 진행도 보상은 먼저 `research_progress`를 증가시킨다.
- 진행도가 임계치에 도달하면 해당 파편은 `pending research delivery` 상태가 된다.
- pending 상태는 "본사 분석은 완료됐지만 현장 수령/송신이 아직 끝나지 않은 상태"를 의미한다.
- 다음 노드가 전투/보스가 아닌 Safe Node라면, 진입 또는 노드 결과 처리 시점에 본사 송신으로 자동 수령한다.
- 다음 노드가 `Combat` 또는 `Boss`라면 수령하지 않고 계속 보류한다.
- 수령은 플레이어 선택형 보상이 아니라 자동 지급이다.

기본 흐름:

```text
전투/방어/회수 노드 종료
-> 자동 보상 지급
-> 연구 진행도 증가
-> 임계치 도달 항목은 pending research delivery로 등록
-> 결과창에 연구 완료/수령 대기 표시
-> 맵으로 복귀
-> 다음 Safe Node 진입
-> pending 파편 자동 수령
```

수령 가능한 노드 범위는 고정 하드코딩하지 않고 정책으로 관리한다.

권장 정책 구조:

```text
ResearchDeliveryPolicy
-> deliver_on_node_categories: [Support, HeadquartersContact, Shop, Reward]
```

현재 구현 기준:

- `SkillFragmentPolicy.research.completion_threshold = 100`
- 임계치 1회 충족마다 `research_completion_count`가 증가한다.
- 완료 건수는 즉시 파편 스택으로 들어가지 않고 `pending_research_deliveries`에 쌓인다.
- `ConfirmEnterNode`에서 진입 노드가 `ResearchDeliveryPolicy` 허용 범위이면 pending 배송을 자동 수령한다.
- 인벤토리 스냅샷은 보유 파편 목록과 별도로 `skill_fragment_progress`, `pending_research_deliveries`를 노출한다.
- Safe Node 진입 결과는 `research_deliveries`를 함께 반환한다. 현재 해당 필드는 `NodeEntered`, `SupportState`, `HeadquartersContactState`, `ShopState`, `RewardState`에 포함된다.
- 클라이언트는 이 값을 사용해 본사 송신/분석 완료 토스트나 결과창 항목을 즉시 표시할 수 있다.

기본 Safe Node:

- `Support`
- `HeadquartersContact`
- `Shop`
- `Reward`

기본 제외 노드:

- `Combat`
- `Boss`

이 정책은 추후 변경 가능해야 한다. 예를 들어 "Support에서만 수령", "Maintenance에서만 수령", "Shop 제외" 같은 변경은 데이터/정책 레이어에서 처리하고, 연구 진행도와 파편 인벤토리 구조를 다시 바꾸지 않는다.

## 맵 컨텐츠 풀

맵 노드의 `shop_pool_id`, `reward_pool_id`는 표시용이 아니라 실제 후보 풀이다.

- `Shop` 노드는 `shop_id`가 있으면 해당 상점으로 진입하고, 없으면 `shop_pool_id`의 상점 후보 중 하나를 seed 기반으로 선택한다.
- `Reward` 노드는 `reward_pool_id`의 보상 후보 중 하나를 seed 기반으로 선택한다.
- 맵 Reward 풀은 `Forbidden` 보상을 포함하지 않는다.
- 풀 id가 작성되어 있는데 실제 풀을 찾을 수 없으면 데이터 오류로 취급한다.

## RandomEvent 재도입 정책

기존 `Event` 노드는 공식 live flow에서 제거되었다.

제거 완료 상태:

- 기존 `Event`는 독립 노드가 아니라 `Shop` 또는 `Reward` 세션으로 즉시 라우팅하는 래퍼에 가까웠다.
- `Shop`으로 이어지는 이벤트는 `Shop` 노드로 표현한다.
- `Reward`로 이어지는 이벤트는 `Reward` 노드로 표현한다.
- `Suppress`로 이어지는 이벤트는 현재 전투/조우 노드로 재설계해야 하며, live 맵 이벤트 풀에 넣지 않는다.
- `MapNodeCategory::Event`, `MapNodePayload::Event`, `RandomEventDatabase` live 로딩/검증, `event_random`, `event_abnormality_room` 노드 정의는 live flow에서 제거되었다.

추후 재도입 방향:

- 랜덤 이벤트가 필요하면 제거된 `Event` 래퍼를 되살리지 말고, 별도 선택형 `RandomEvent` 노드로 새로 작성한다.
- 새 `RandomEvent`는 `상황 설명 -> 2~3개 선택지 -> 비용/리스크/보상 적용 -> 결과 -> 맵 복귀` 흐름을 가져야 한다.
- 새 `RandomEvent`는 `Shop`, `Reward`, `Support`, `HeadquartersContact`와 역할이 겹치지 않아야 한다.

## 장비 획득과 성장 정책

장비는 스킬 파편보다 더 자주 획득하고 교체하는 성장 축이다. 스킬 파편이 직원 빌드의 방향을 바꾸는 고가치 보상이라면, 장비는 현재 런을 버티고 전투력을 보정하는 실전 자산이다.

장비 슬롯:

- `무기`: 공격 방식, 사거리, 기본 피해 구조를 바꾼다.
- `방어구`: 생존력, 피해 저항, 트라우마 저항 보조 같은 방어 성능을 담당한다.
- `악세서리`: 조건부 효과, 유틸리티, 특수 보정을 담당한다.

무기 역할 정책:

- 직원은 고정 직업을 직접 갖지 않는다. 직원의 현재 전투 역할은 장착 무기가 결정한다.
- 무기는 `range_role`을 가진다. 큰 분류는 `Melee`와 `Ranged`다.
- 무기는 세부 `weapon_archetype`을 가진다. 예시는 검, 창, 도끼, 방패, 활, 석궁, 총, 샷건, 유탄, 스태프다.
- 무기는 기본 공격 타입, 사거리, 공격 범위 패턴, 공중 공격 가능 여부, 기본 저지 보정, 기본 `targeting_profile`을 제공한다.
- 방어구는 생존, 피해 저항, 트라우마 저항, 저지 성향 보조를 담당한다.
- 악세서리는 부족한 빌드 보완, 조건부 특수 효과, 관통/저항 감소, 공명/초기 충전, 타겟팅 보정 같은 유틸리티를 담당한다.
- 스킬 파편은 직업명을 요구하지 않고, 장착 중인 무기가 만든 전투 프로필과 무기 아키타입을 요구 조건으로 삼는다.

무기 아키타입 예시:

| 대분류 | 아키타입 | 기본 역할 | 기본 타겟팅 방향 |
| --- | --- | --- | --- |
| Melee | Sword | 표준 근접 딜러 | 저지 대상 우선, 없으면 route progress 높은 적 |
| Melee | Spear | 전방 긴 사거리 근접 | 전방 범위 안 route progress 높은 적 |
| Melee | Axe | 느린 고화력/소규모 광역 | 저지 대상 우선, 없으면 공격 범위 안 다수 압박 |
| Melee | Shield | 저지/생존 중심 | 저지 대상 고정 우선 |
| Ranged | Bow/Crossbow | 대공/표준 원거리 | 공중 우선, 그다음 route progress 높은 적 |
| Ranged | Gun | 정밀 원거리 | 방어력이 낮은 적 또는 route progress 높은 적 |
| Ranged | Staff | 마법 원거리 | 마법 저항이 낮은 적 또는 고방어 적 대응 |
| Ranged | Shotgun | 짧은 원거리/전방 확산 | 가까운 전방 다수 또는 범위 안 압박 |
| Ranged | GrenadeLauncher | 폭발/광역 원거리 | splash 기대값이 큰 뭉친 적 |

타겟팅 프로필 정책:

- 타겟팅 규칙은 무기 아키타입에 직접 하드코딩하지 않고 별도 `targeting_profile`로 분리한다.
- 초기 프로필은 작게 시작한다. 확정 초기 후보는 `DefaultForward`, `AirFirst`, `LowDefenseFirst`, `LowMagicResistFirst`, `SplashClusterFirst`다.
- `DefaultForward`: 공격 범위 안의 유효한 적 중 방어 목표까지 남은 route가 가장 짧은 적을 우선한다. 동률이면 먼저 spawn된 적, 그다음 unit id다.
- `AirFirst`: 공격 범위 안의 공중 적을 우선하고, 없으면 `DefaultForward`를 따른다.
- `LowDefenseFirst`: 공격 범위 안의 유효한 적 중 방어력이 가장 낮은 적을 우선하고, 동률이면 `DefaultForward`를 따른다.
- `LowMagicResistFirst`: 공격 범위 안의 유효한 적 중 마법 저항이 가장 낮은 적을 우선하고, 동률이면 `DefaultForward`를 따른다.
- `SplashClusterFirst`: 직접 대상 하나를 고르되, 해당 대상 주변에 함께 맞는 유효 적 수 또는 예상 피해 기대값이 큰 지점을 우선한다. 동률이면 `DefaultForward`를 따른다.
- 근거리 기본 공격은 저지 중인 적을 최우선으로 한다. 저지 중인 적이 없을 때만 무기의 `targeting_profile`을 적용한다.
- 원거리 기본 공격은 무기의 `targeting_profile`을 기본 source of truth로 삼는다.
- 스킬은 파편 또는 스킬 데이터가 별도 타겟팅을 명시하면 스킬 전용 타겟팅을 사용할 수 있다. 명시하지 않으면 장착 무기의 기본 프로필을 따른다.
- 추후 필요하면 `EliteFirst`, `LowestHpFirst`, `HighestBlockWeightFirst`, `BossFirst`, `ClosestFirst` 같은 프로필을 추가할 수 있다. 단, 새 프로필은 실제 무기/스킬 파편 수요가 확인된 뒤 추가한다.

기본 획득 구조:

- 일반 장비는 완제품 드랍보다 장비 가루와 강화 재료 축적을 중심으로 한다.
- 침식된 직원 전투에서는 장비 가루와 손상된 부품을 주로 획득한다.
- 장비 제작은 현재 live Maintenance action이 아니다. 추후 필요하면 `CraftEquipment` 같은 별도 개념으로 새로 설계한다.
- 장비를 분쇄해 장비 가루를 얻고, 장비 가루를 소비해 이미 보유한 장비를 강화할 수 있다.
- 완제품 장비는 제거하지 않지만, 일반 보상으로 흔하게 지급하지 않고 희귀 보상 또는 고위험 보상으로 사용한다.

구현 기준:

- 완성 장비는 `OwnedEquipment` 인스턴스로 보관하고 장비 인벤토리 슬롯을 차지한다.
- 장비 가루, 코어, 설계도, 잔재는 `EquipmentMaterialMetadata`와 `equipment_materials` 스택으로 보관한다.
- 장비 재료 보상은 `GrantEquipmentMaterial`로 지급한다. 이 보상은 장비 인스턴스를 만들지 않고 재료 수량만 누적한다.
- PVE 전투 보상은 낡은 무작위 완성 장비 보상에 의존하지 않는다. 전투 조우는 장비 가루, E.G.O 잔재, 연구 진행도, 낮은 확률의 스킬 파편 후보를 조합해 보상 풀을 구성한다.
- 짧은 `DefenseRoute / Defense` 조우는 장비 회수 보상을 포함해 “침식 직원 경로를 막고 장비 잔해를 회수한다”는 정체성을 가질 수 있다.
- `DefenseRoute` 고위험 변형은 완성 파편이 없어도 연구 진행도 보상만으로 조우 정체성을 가질 수 있다.
- Maintenance 장비 분쇄는 `EquipmentDismantleRecipeMetadata`를 사용한다. 레시피는 장비 id별 회수 재료를 정의하고, 실행 시 장비 인스턴스를 제거한 뒤 장비 가루/재료 스택을 증가시킨다. 장착 중인 장비는 확인 후 자동 해제하고 분쇄한다.
- Maintenance 장비 강화는 `EquipmentEnhancementRecipeMetadata`를 사용한다. 레시피는 장비 id별 최대 강화 레벨, 레벨당 재료 비용, 레벨당 스탯 보정을 정의한다.
- 강화 레벨은 장비 정의가 아니라 `OwnedEquipment` 인스턴스에 저장한다. 같은 장비 id라도 개별 인스턴스마다 강화 상태가 다를 수 있다.
- 전투 시작 시 직원이 장착한 장비 인스턴스의 강화 레벨을 `BattleScenario` 유닛 draft에 스냅샷으로 복사하고, 전투 중에는 그 스냅샷을 기준으로 스탯을 계산한다.
- 장비 강화는 장비의 정체성을 바꾸는 파편 개화가 아니라, 런 전투력을 안정적으로 끌어올리는 장기 재료 sink다.

기본 흐름:

```text
전투에서 장비 가루/강화 재료 회수
-> 완제품 장비 획득 또는 보유 장비 강화 재료로 축적
-> 필요 없는 장비를 분쇄해 장비 가루 획득
-> 장비 가루를 소비해 장비 강화
```

장비 재료 단계:

```text
장비 가루
-> 장비 코어 / 설계도 / 특수 재료
-> 장비 강화 또는 완제품 장비 보상 보조
```

획득처 방향:

- 침식 직원 중심 `DefenseRoute` 전투는 장비 가루, 손상된 부품, 엔케팔린을 제공한다.
- 정예 `DefenseRoute` 변형은 고급 장비 재료와 낮은 확률의 완제품 장비를 제공할 수 있다.
- 환상체 진압은 E.G.O 잔재, E.G.O 설계도, E.G.O 장비 연구도의 주된 출처다.
- 방어형 블랙박스 회수는 연구자료, 분석 데이터, 장비 재료와 파편 연구 보조와 연결한다.
- `DefenseRoute / SplitRoom` 같은 고급 변형은 설계도와 특수 부품을 제공해 특정 장비 계열을 열어줄 수 있다.
- 상점은 완제품 장비, 부족한 부품, 제작 재료를 판매해 운이 나쁜 런을 보정한다.
- Maintenance는 장비 강화와 분쇄를 처리하는 핵심 노드다.

강화 정책:

- 장비 제작은 현재 live Maintenance action이 아니다. 추후 필요하면 `CraftEquipment` 같은 별도 개념으로 새로 설계한다.
- 장비 강화는 장비 가루와 강화 자원을 소비해 장비 성능을 올린다.
- 상위 강화는 고급 파편, 코어, 설계도를 요구할 수 있으며 특수 옵션 개방이나 큰 수치 상승으로 연결할 수 있다.
- 스킬 파편 경제와 장비 경제는 분리한다. 스킬 파편을 분쇄해 얻은 파편 가루로 장비를 강화할 수 없고, 장비를 분쇄해 얻은 장비 가루로 스킬 파편을 강화할 수 없다.
- 필요 없는 스킬 파편은 분쇄해 파편 가루로 전환하고, 필요 없는 장비는 분쇄해 장비 가루로 전환한다.

E.G.O 장비:

- E.G.O 장비는 일반 장비보다 환상체 진압, 연구도, 잔재, 설계도 의존도가 높다.
- 환상체를 진압하면 완성 E.G.O 장비를 확정 지급하기보다 E.G.O 잔재, 설계도, 연구도, 낮은 확률의 완제품 후보를 조합한다.
- E.G.O 장비는 스킬 파편처럼 빌드 방향 전체를 뒤집는 보상은 아니지만, 일반 장비보다 개성과 위험성을 강하게 가진다.
- 장기적으로 E.G.O 오버클럭과 침식 게이지를 추가할 수 있지만, 현재 구현 대상은 아니다.

## 적 구성 원칙

탐사 중 만나는 적은 전부 환상체일 필요가 없다. 대규모 웨이브에서 환상체가 대량으로 등장하면 환상체의 희소성과 위협감이 약해지고, 세계관상 같은 시설에 환상체가 무더기로 배치된 것처럼 보일 수 있다.

기본 원칙:

- 일반 전투와 대규모 웨이브의 주력 적은 침식된 전 탐사 직원이다.
- 침식 직원은 이전 탐사에 실패했거나, 환상체 영향에 오염된 직원의 잔재다.
- 환상체는 핵심 조우 대상, 정예 위협, 보스, 특수 이벤트, 보상 원천으로 사용한다.
- 보스 전투에서는 환상체 본체와 침식 직원 증원이 함께 등장할 수 있다.

초기 적 분류 방향:

- `CorrodedEmployee`: 침식된 전 탐사 직원. 일반 웨이브, 매복, 증원에 사용한다.
- `Abnormality`: 환상체 본체. 보스, 정예, 특수 조우의 중심 위협으로 사용한다.
- `FacilityEntity`: 시설 장치나 환상체 영향으로 움직이는 비인간형 개체. 필요할 때 후순위로 추가한다.

`FacilityEntity`는 아직 구현 대상이 아니다. 자동 방어 장치, 붕괴한 격리 장치, 움직이는 보안 설비 같은 적이 필요해질 때 확장한다.

침식 직원 외형 변이:

- 침식 직원은 같은 역할군이라도 클라이언트에서 단조롭게 보이지 않도록 seed 기반 외형 변이를 허용한다.
- 외형 변이는 머리, 몸통, 팔, 다리, 침식 부위, 색상 팔레트, 장식 파츠 같은 표시 요소를 조합하는 용도다.
- 같은 `profile_id`와 같은 외형 seed는 항상 같은 외형 조합을 만들어야 한다.
- 외형 seed는 전투 스탯을 변경하지 않는다.
- preview의 `SpawnWaveEnemyEntry.appearance_seeds`는 `CorrodedEmployee` 개체 수만큼 외형 seed를 제공한다.
- 환상체와 아직 구현되지 않은 시설 개체는 이 외형 seed 목록을 사용하지 않는다.
- 침식 직원의 전투 성능은 역할군, 난이도, 전투 프로필 데이터로 결정한다.
- 같은 난이도의 같은 침식 직원 역할군은 같은 전투 성능을 가져야 한다. 그래야 플레이어가 조우를 학습하고 같은 전략을 재현할 수 있다.
- 클라이언트는 외형 seed와 파츠 풀 id를 사용해 시각적 바리에이션을 만들 수 있지만, 전투 판정의 source of truth는 코어의 전투 프로필이다.

침식 직원 웨이브 생성 정책:

- 수동 `PveWaveEnemyData` 나열만으로 모든 침식 직원 웨이브를 작성하지 않는다.
- `PveWaveData`는 수동 웨이브와 생성 웨이브를 모두 표현할 수 있는 source contract를 가진다.
- 구현 구조는 `PveWaveSource::Manual(Vec<PveWaveEnemyData>)`와 `PveWaveSource::GeneratedCorroded { preset_id, budget_override, seed_salt }`의 분리다. 기존 `enemies` 직접 작성은 하위 호환 수동 source로 유지한다.
- `GeneratedCorroded`는 별도 `CorrodedWavePreset` RON을 참조한다. preset은 `difficulty`, `pressure`, `role_mix`, `budget`, `count_range` 같은 밸런스 값을 가진다. 현재 source of truth는 `game_resources/data/enemies/corroded_wave_presets.ron`이다.
- 조우 RON은 “이 웨이브가 어떤 의도인지”를 `preset_id`로 표현하고, 구체적인 역할군 가중치와 예산은 preset RON에서 조정한다.
- 현재 생성기는 preset, preview seed, wave index, `seed_salt`, `budget_override`를 사용해 역할군 조합과 수량을 확정한다. 이후 `node_type`, `difficulty`, `risk_level`, `battlefield_archetype`, act/depth, `player_squad_power`를 입력으로 더 반영할 수 있다.
- 생성 결과는 `CombatPreview` 생성 시점에 확정되어 `SpawnWave.enemy_entries`에 저장된다. `BattleScenario`와 `BattleCore`는 이미 확정된 enemy entries만 사용하고 전투 중 재생성하지 않는다.
- 같은 조우, 같은 seed, 같은 preset은 항상 같은 웨이브 구성을 만들어야 한다.
- generator는 전투 스탯을 seed로 랜덤화하지 않는다. 랜덤성은 역할군 구성, 수량 범위, 외형 seed, 출현 변주에만 사용한다.
- 초기 구현은 기존 `enemies` 직접 작성 방식을 수동 기본값으로 유지하고, 신규/일부 live 조우만 `GeneratedCorroded`로 전환하는 하이브리드 방식이다. 현재 `black_box_archive_defense`와 `defend_black_box_relay`가 생성형 침식 직원 웨이브를 사용하는 대표 live 조우다.
- 방어형 기본 프리셋은 `black_box_breach_probe`, `black_box_breach_pressure`처럼 “목표 지점으로 압박해 들어오는 침식 직원 웨이브”를 표현한다. 새 방어형 조우는 개별 적을 길게 나열하기보다 이 프리셋을 우선 재사용하고, 특수 방어전이 필요할 때만 별도 preset을 추가한다.

필드 위험요소:

- 필드 위 위험요소는 적 기물이 아니라 전장 요소로 분리한다.
- 예시는 가스 누출, 침식 장판, 함정 장치, 오작동 격리문, 환청 방송 등이다.
- 현재는 구현하지 않는다.
- 전장 생성기와 전투 런타임이 안정된 뒤 `BattlefieldHazard` 같은 별도 타입으로 추가한다.

## 기본 브리핑과 리트라이 학습

전투 노드 미리보기는 제한 자원을 소비해 업그레이드하지 않는다. 현재 공식 흐름은 명일방주식 실시간 배치, 후퇴, 재배치, 재시도를 통한 학습을 중심으로 한다.

기본 브리핑 규칙:

- `NodePreview`와 `CombatPreview`는 항상 전투 진입에 필요한 기본 정보를 제공한다.
- 기본 브리핑에는 전투 모드, 전장 템플릿, 배치 가능 구역, 주요 route 또는 예상 진입 방향, 위험도, 보상 후보, 주요 적 또는 핵심 위협 요약이 포함된다.
- 정확한 웨이브 수, 모든 등장 타이밍, 숨은 기믹의 세부 값은 기본적으로 전투 중 관찰과 리트라이 학습 영역으로 둔다.
- 단, 플레이어가 납득 가능한 배치를 할 수 없을 정도로 정보가 부족하면 별도 소비 자원 없이 기본 브리핑 자체를 보강한다.
- 불확실한 출현 방향이나 매복 가능성은 `Suspected`, `Likely`, `Confirmed` 같은 confidence로 표현한다.
- `NodePreview`/`CombatPreview`는 정확한 적 스탯 숫자를 공개하지 않는다. 대신 `장갑형 적 출현 가능`, `마법 저항이 높은 적 출현 가능`, `공중 적 출현 가능`, `저지하기 어려운 적 출현 가능`, `보호막 적 출현 가능`, `재생 적 출현 가능`, `빠른 돌파 적 출현 가능` 같은 위협 경고문을 제공한다.
- 위협 경고는 단순 tag 배열이 아니라 `threat_warnings` 항목으로 관리한다. 각 항목은 경고 tag와 `Unverified`, `Disproved` 상태를 가진다.
- 최초 진입 전 경고는 resolved spawn wave 기준 실제 경고를 우선한다. 여기에 20% 확률로 사전 정의된 루머성 오경고를 1개까지 섞을 수 있다. 오경고는 seed 기반으로 결정해 같은 이상현상에서는 재현 가능해야 한다.
- 루머성 오경고 후보는 코드가 즉석 문장을 만들지 않고, 사전에 준비된 위협 경고 사전에서 랜덤 선택한다.
- 루머성 오경고는 실제 resolved spawn wave의 경고 tag에 없는 후보에서만 선택한다.
- 플레이어가 전투에 진입한 뒤 퇴각하면, 같은 이상현상 재진입 preview에서 false rumor를 `Disproved`로 표시한다. Unity는 `Disproved` 경고를 취소선 처리한다.
- 실제 경고는 `Unverified`로 유지한다.
- 전투 후 UI 기록은 다음 재시도에서 플레이어가 같은 실수를 줄일 수 있도록 실제로 관찰한 웨이브와 위험 요소를 보여줄 수 있다.
- 전투 기록 UI가 생기더라도 threat warning status와 별개의 표시로 둔다. 적 HP, 웨이브 진행도, 보호 목표 피해는 재진입 시 초기화한다. 사용한 소비 아이템은 사용 시점에 소모되고, 퇴각 전에 이미 발생한 전투불능/트라우마/런 HP 손상은 유지한다.

## 전장과 배치

- 고정된 TFT식 상하 전선만 사용하지 않는다.
- 노드마다 전장 템플릿, 장애물, 배치 가능 구역, 적 출현 구역이 달라질 수 있다.
- 전장 템플릿은 shared RON `game_resources/data/map/battlefield_templates.ron`의 ASCII `rows`로 작성한다.
- ASCII row의 공백은 전장 밖이다. 따라서 전장은 반드시 사각형일 필요가 없고, ㄱ형 복도, ㄷ형 방, 무너진 비정형 폐허처럼 만들 수 있다.
- `width`는 가장 긴 row 길이, `height`는 row 개수로 계산한다. 실제 이동/배치/스폰 가능 여부는 공백이 아닌 `valid_tiles`가 결정한다.
- 공백 타일은 `Battlefield` 기준으로 `OutOfBounds`이며, Rapier 연속 이동 입력에서는 `void_tile` 정적 충돌체로 투영된다.
- ASCII 문자 규칙은 `.` 일반 전장 타일, `#` 장애물, `P` 지상 아군 배치, `T` 플랫폼 아군 배치, `N/L/Q/A/B/W/R` 적 출현 구역이다.
- 명일방주식 Defense route는 terrain row와 같은 크기의 별도 ASCII overlay로 작성한다. terrain이 실제 맵 source of truth이고, route overlay는 화살표와 시작/종료 marker만 얹는 얇은 레이어다.
- route overlay의 row 수와 column 수는 terrain과 1:1로 맞아야 한다. overlay 공백은 “route 없음”을 뜻하므로 terrain 안/밖 어디든 가능하고, 화살표와 시작/종료 marker는 terrain의 이동 가능 타일 위에만 올 수 있다.
- route overlay는 디버그/검증 시 terrain과 합성해 출력한다. 작성자는 합성된 맵을 통해 길, 장애물, 배치칸, 목표 지점을 한눈에 확인할 수 있어야 한다.
- route 하나는 초기 계약에서 단일 시작점, 단일 종료점, 무분기 선형 경로다. 분기 경로가 필요하면 하나의 route 안에서 갈라지게 만들지 말고, 별도 `route_id`를 가진 route 여러 개로 작성한다.
- 조우 RON의 웨이브는 사용할 `route_id`를 명시한다. spawn marker만으로 자동 경로를 추론하지 않는다.
- 같은 아키타입/크기 클래스에 여러 ASCII 템플릿이 있으면 seed 기반으로 하나를 선택한다. 이 방식이 현재 전장 다양성의 기본 수단이다.
- 런 동안 유지되는 공용 전투 배치판은 사용하지 않는다.
- 전투 배치는 전투/보스 노드 진입 후 `InBattle` 상태에서 해당 노드의 `DeploymentZone` 안에 live `DeployUnit` 명령으로 만든다.
- 배치 UI는 전장 전체 타일을 보여주고, 배치 가능한 타일만 파란색 등으로 표시한다.
- 전투 이동과 충돌은 2D 연속좌표 기반이지만, 배치 입력과 사거리/스킬 범위 후보 선별은 타일 기반이다.
- `BattleScenario` 안의 유효 타일, 스폰 위치, 전술 포인트, 장애물, 배치 가능 타일은 모두 같은 전장 타일 좌표계를 사용하며 런타임에서는 타일 중심 월드 좌표로 변환된다.
- `CombatPreview` 생성 시 전장 크기, 배치 구역, 출현 구역, 장애물 겹침, 출현 구역 참조, 배치 구역에서 출현 구역까지의 기본 경로를 production 경로에서 검증한다.
- `BattleScenario`는 전투 시작 전에 검증된다. 잘못된 전장 크기, 범위 밖 좌표, 중복 id, 존재하지 않는 전술 포인트/유닛 참조, 스폰되지 않는 필수 적 그룹은 런타임 상태를 만들기 전에 실패해야 한다.
- `ConfirmEnterNode`는 최소 한 명 이상의 출전 가능한 직원을 요구한다. 배치 가능 여부, 안정화치 비용, 배치 타입, 점유 상태, 방향은 `DeployUnit` 시점에 core가 검증한다.
- 조우 RON의 `static_obstacles`가 비어 있으면 전장 아키타입의 기본 장애물 패턴을 사용한다.
- 조우 RON의 `static_obstacles`가 하나 이상 있으면 해당 조우는 작성자가 장애물 배치를 직접 지정한 것으로 보고, 아키타입 기본 장애물 대신 작성된 장애물만 사용한다.
- 전장 크기는 초기 기준으로 소형, 중형, 대형을 사용한다.
- 보스 전장은 기본적으로 대형 전장으로 취급한다.
- 적 출현 위치는 정확한 좌표보다 `spawn_zone_id` 기반으로 표현한다.
- 클라이언트는 출현 구역을 정확한 칸보다 그라디언트 원형 영역처럼 대략적인 위치로 표시한다.
- 적 출현 위치는 완전 랜덤보다 `예상 구역 + 일부 변동성`을 우선한다.
- 위험한 변동성은 기본 브리핑의 confidence/summary 또는 전투 중 관찰 가능한 신호로 예고해야 한다.
- 플레이어가 패배를 정보 부족 탓으로만 느끼지 않도록 한다.

전장 아키타입:

- `OpenHall`: 넓은 홀. 접근 경로가 많고 배치 자유도가 높다.
- `Corridor`: 외길 복도. 전선이 좁고 전열/후열 구분이 강하다.
- `ChokePoint`: 병목 지형. 좁은 길목을 지키거나 뚫는 판단이 중요하다.
- `Ambush`: 매복 지형. 측면 또는 후방 출현 구역이 존재한다.
- `Surrounded`: 포위형 지형. 전방이 뚫렸거나 여러 방향에서 적이 접근한다.
- `SplitRoom`: 분리된 방. 직원 분산 배치와 합류 타이밍이 중요하다.
- `ObstacleRoom`: 장애물이 많은 공간. 사거리, 우회, 광역 스킬 가치가 달라진다.
- `BossArena`: 보스 전용 공간. 중앙 핵심 위협과 특수 지형/증원 구역을 함께 가진다.

전장 아키타입은 고정 룰이 아니라 데이터 분류다. 실제 전투 노드는 아키타입, 전장 템플릿, 배치 가능 구역, 출현 구역, 장애물 데이터를 조합해 구성한다.

전장 아키타입 메타 정보는 shared RON `game_resources/data/map/battlefield_archetypes.ron`에서 관리한다. 실제 배치/스폰/장애물/비정형 전장 형태는 shared RON `game_resources/data/map/battlefield_templates.ron`의 ASCII 템플릿이 source of truth다.

아키타입별 전술 질문:

- `OpenHall`: 넓은 공간에서 전열을 어떻게 잡고 여러 접근 경로를 감당할 것인가.
- `Corridor`: 좁은 통로에서 누가 막고 누가 뒤에서 지원할 것인가.
- `ChokePoint`: 병목을 앞에서 막을지, 병목 뒤에서 받아칠지 결정한다.
- `Ambush`: 불확실한 측면/후방 출현을 어떻게 예고하고 대응하게 할 것인가.
- `Surrounded`: 여러 route 압박 속에서 어디에 핵심 화력을 둘 것인가.
- `SplitRoom`: 본대 유지와 별도 기믹 제어를 어떻게 분리할 것인가.
- `ObstacleRoom`: 장애물 때문에 달라지는 사거리, 우회, 광역 가치를 어떻게 살릴 것인가.
- `BossArena`: 중앙 핵심 위협, 특수 지형, 증원 구역을 어떻게 조합할 것인가.

전투 목적과 이동 정책:

- 전투는 `DefenseRoute` 단일 모드로 정리한다.
- 각 전투는 `BattleScenario`의 전술 정책을 통해 route, 방어 목표, 임무 변형을 표현한다.
- 적 전진형 전투는 route 기반 이동으로 표현한다.
- `DefenseRoute / Defense`의 live 계약은 `ProtectUnit`/보호 오브젝트 기반 명일방주식 고정 방어다. 성공은 모든 웨이브 종료, 필수 적 전멸, 보호 오브젝트 생존으로 판정하고, 실패는 보호 오브젝트 파괴로 판정한다.
- `DefenseRoute` live 전투는 서버 시뮬레이션 기준의 재생/정지/배속을 지원한다. 정지는 Unity 화면만 멈추는 기능이 아니라 전투 시간, 웨이브, 이동, 공격, 투사체, 자동/트리거 스킬 진행을 멈추는 전투 상태다.
- `DefenseRoute` live 전투는 `PauseBattle`, `ResumeBattle`, `SetBattleSpeed(X0_5/X1/X2/X3)`로 제어한다. 배속은 서버 tick interval을 바꾸는 방식이 아니라, 같은 server tick에서 BattleCore에 넘기는 simulation delta를 조정하는 방식이다. `X0_5`는 fractional tick 누적으로 처리한다.
- 정지 중에는 `battle_time_ms`와 battle event log delta(`timeline_delta`)가 증가하지 않는다. 상태 조회와 허용된 전투 명령은 가능하지만, delay가 필요한 효과는 전투가 재생된 뒤 진행된다.
- `Physical`/`Magic`/`True` 피해는 빌드 색깔과 효율 차이를 만드는 축이다. UI/기획 표현에서 AD는 `Physical`, AP는 `Magic`에 대응한다.
- 특정 일반/정예 적이 한 피해 타입을 사실상 영구 무효화해 반대 타입 딜러 보유 여부만 검사하는 구조는 피한다.
- 방어력/마법 저항이 높은 적은 등장할 수 있지만, 대응 수단은 반대 타입 딜러 하나로 잠그지 않는다. 관통, 저항 감소, 고정 피해, 제어, 저지, 소비 아이템, 배치/스킬 타이밍 같은 우회 수단을 열어둔다.
- 보스는 일시 보호막, 기믹 방어, 특정 타이밍 무효를 가질 수 있다. 단, 보스도 영구적인 단일 타입 면역으로 빌드 전체를 부정하지 않는다.
- 과거 `DefendPoint` 하위 변형과 지점 누수 타임라인은 제거됐다. 관문 통과/라이프 누수형 방어가 필요하면 현재 Defense 계약과 섞지 않고 별도 임무로 새로 설계한다.
- `RecoverHoldAndExtract` 기반 회수형 전투는 공식 모드에서 내린다. 필요한 연출은 추후 `DefenseRoute / SplitRoom` 또는 별도 비전투 노드로 재작성한다.
- `DefenseRoute / Encirclement`는 포위 생존 변형이다. 방어 대상 없이 여러 route에서 압박받고, 일정 시간 생존한 뒤 최종 웨이브를 정리하는 방향을 기본값으로 한다.
- `DefenseRoute / SplitRoom`은 추후 구현 대상이다. 지금은 split-squad 배치 UI, 제어장치 상태, 본대 디버프/추가 웨이브 트리거를 별도 정책으로 확정하기 전까지 live 조우 배정에 사용하지 않는다.
- `DefenseRoute / Defense`에서는 아군이 배치 위치에 고정되고, route를 따라 들어오는 적을 저지/공격한다.
- `TacticalGroupPlan`/`GroupObjective`/포메이션 기반 group 이동 AI는 최신 공식 전투 모드에서 제거됐다. 단체 이동이 필요한 새 전투가 필요해지면 기존 레거시를 되살리지 말고, 실제 플레이 목적과 UI가 확정된 뒤 별도 계약으로 다시 설계한다.
- `SplitRoom`은 `DefenseRoute`의 고급 변형 후보로만 남긴다. 지금은 split-squad 배치 UI, 제어장치 상태, 본대 디버프/추가 웨이브 트리거를 별도 정책으로 확정하기 전까지 live 조우 배정에 사용하지 않는다.
- 보상은 `Currency`, `Equipment`, `SkillFragment`, `ResearchProgress` 같은 목적 태그를 가진다. 전투 목적은 어떤 보상 태그를 강조할지 결정하고, 실제 지급은 보상 효과가 담당한다.
- 스킬 파편 연구도 보상은 파편 자체를 지급하지 않고, 해당 파편의 `research_progress`만 올린다. 이는 개화 진행도와 별개의 본부 분석/블랙박스 회수 진행도다.
- PVE 전투 보상 후보에는 환상체를 플레이어 보상으로 지급하던 레거시 `Forbidden` 보상을 넣지 않는다.
- 실시간 전투 전환과 레거시 제거 작업의 기준은 이 문서의 `DefenseRoute` 정책과 `unity_core_contract.md`를 따른다.

전투 피해 피드백:

- core는 피해 결과를 계산한 뒤 Unity-facing battle event에 `feedback_tags`를 내려준다. Unity는 방어력/마법 저항 세부 수치를 재계산하거나 추론하지 않는다.
- 세부 적 스탯 도감은 현재 만들지 않는다. 플레이어는 노드 미리보기의 위협 경고, 적 외형, 피해 숫자 색상, 피해 피드백 라벨로 적의 성향을 읽는다.
- 피해 숫자 색상은 피해 타입을 표현한다. `Physical`은 붉은 계열, `Magic`은 푸른 계열을 기본으로 한다. `True`는 별도 고정 피해 색상을 둘 수 있지만, 초기에 필요 없으면 neutral/white 계열로 둔다.
- 피해 숫자 상단 라벨은 `critical`, `mitigated`, `fixed_damage`, `immune`만 사용한다. `weakness` 상단 태그는 초기 계약에서 사용하지 않는다.
- 피해 숫자 좌측 라벨은 그 외 보조 태그에 사용한다. 초기에는 거의 비워두고, `piercing`, `shield`, `blocked`, `resisted_status` 같은 태그가 생기면 좌측에 붙인다.
- 한 번의 피해에 여러 상단 태그 후보가 있으면 Unity는 메인 상단 라벨을 하나만 표시한다. 상단 우선순위는 `immune > fixed_damage > mitigated > critical`이다.
- `feedback_tags`는 연출/표시 계약이다. 피해량, 사망 여부, HP 변화의 source of truth는 여전히 `HpChanged.delta`, `hp_before`, `hp_after`, `final_damage`다.

초기 `feedback_tags` 판정 기준:

| tag | 표시 위치 | core 판정 기준 |
| --- | --- | --- |
| `critical` | 상단 | 피해 결과의 `critical == true` |
| `mitigated` | 상단 | `raw_damage > 0`, `final_damage > 0`, `final_damage <= raw_damage * 60%` |
| `fixed_damage` | 상단 | 주 피해 타입이 `True` |
| `immune` | 상단 | `raw_damage > 0`인데 최종 피해가 0이거나, 상태이상이 완전히 막힘 |
| `piercing` | 좌측 | 해당 피해 타입의 관통/저항 관통 modifier가 실제 저항값을 낮춤 |
| `shield` | 좌측 | 피해가 실체 HP보다 보호막/방어막에 먼저 적용됨 |
| `blocked` | 좌측 | 일회성 방어, 패링, 상쇄 효과가 피해 일부 또는 전부를 지움 |
| `resisted_status` | 좌측 | 상태이상이 들어갔지만 지속시간/효과가 감소함 |

전장 크기 기준:

- `Small`: 소규모 전투. 기본 기준은 7x7 전후다.
- `Medium`: 일반/정예 전투의 기본 크기. 기본 기준은 9x9 전후다.
- `Large`: 보스 또는 대형 조우. 기본 기준은 11x11 전후다.

적 웨이브 규칙:

- 적 등장은 `SpawnWave` 기반으로 관리한다.
- 첫 웨이브는 `time_ms = 0`으로 처리한다.
- 증원, 매복, 보스 패턴은 이후 웨이브에 다른 `time_ms`를 부여해 표현한다.
- 현재 공식 전투 이벤트는 전투 시작 또는 `time_ms` 기반으로만 발생한다.
- 특정 유닛 HP, 특정 유닛 사망, 웨이브 전멸, 지점 도달 같은 조건부 이벤트는 아직 정규 플레이 규칙이 아니다.
- 조건부 이벤트는 `BattleScenario` 2단계 확장으로 남긴다. 지금은 전투 흐름 안정화와 실제 조우 데이터 확장을 우선한다.
- `EnemyBriefing`은 플레이어에게 보여줄 정보이며 실제 스폰 수량/종류의 source of truth가 아니다.
- 실제 전투 스폰은 `SpawnWave.enemy_entries`가 담당한다.
- 조우 데이터 작성 시 실제 웨이브 구성은 `PveEncounter.waves`에 작성한다.
- 향후 `PveEncounter.waves`는 수동 `Manual` 웨이브와 preset 기반 `GeneratedCorroded` 웨이브를 함께 지원한다.
- `PveWaveEnemyData.kind`는 적의 세계관/전술 분류다. 기본값은 현재 호환성을 위해 `Abnormality`이며, 일반 웨이브를 침식 직원 중심으로 바꿀 때는 `CorrodedEmployee`를 명시한다.
- `CorrodedEmployee` 웨이브는 `profile_id`로 `CorrodedEmployeeProfileDatabase`의 전투 프로필을 참조한다.
- `Abnormality` 웨이브는 `abnormality_id`로 환상체 전투 프로필을 참조한다.
- `FacilityEntity`는 아직 전투 프로필 소스가 없으므로 조우 데이터에서 사용할 수 없다.
- 현재 기본 침식 직원 프로필은 `corroded_guard`, `corroded_rusher`, `corroded_marksman`, `corroded_bruiser`, `corroded_medic`, `corroded_veteran`이다.
- 환상체 진압 보상은 환상체 본체를 지급하지 않고, 해당 환상체에서 파생된 독립 스킬 파편을 보상 후보로 제공할 수 있다.
- 초기 라이브 파편 보상은 `One Sin`, `Scorched Girl`, `Red Shoes`, `Der Freischutz` 계열에서 시작한다.
- `PveWaveData.spawn_zone_ids`가 있으면 해당 웨이브는 지정된 출현 구역에서 나온다. 비어 있으면 전장 아키타입 기본 출현 구역을 사용한다.
- `PveWaveData.required_for_victory`가 `false`인 웨이브는 승리 조건에 필수로 포함되지 않는다. 기본값은 `true`다.

전투 시나리오 작성 규칙:

- RON에는 `BattleScenario`를 직접 작성하지 않는다.
- RON 작성자는 `PveEncounter.battlefield`, `PveEncounter.tactical_plan`, `PveEncounter.win_condition`, `PveEncounter.waves`로 조우 의도를 작성한다.
- `BattlefieldGenerator`는 `PveEncounter.battlefield`를 우선 적용하고, 없는 값은 map category/seed 기반 기본값으로 채운다.
- `CombatExecutor`는 `PveEncounter.tactical_plan`과 `PveEncounter.win_condition`을 `BattleScenario`로 변환한다.
- 전투 시작에 사용되는 `CombatPreview`는 요청한 `PveEncounter`에서 생성된 것이어야 한다. preview의 `encounter_id`가 다르거나, authored `PveEncounter.node_type`과 preview `node_type`이 다르면 전투 시작을 거부한다.
- `PveEncounter` 작성 오류는 데이터 로딩 단계에서 1차 검증하고, 최종 전투 입력 오류는 `BattleScenario::validate()`가 전투 시작 전에 검증한다.

## 지원 노드

지원 노드는 Safe Node의 하위 계열이다. core에서는 `SupportState`와 `selected_event.type == "support"`로 표현한다.

현재 사용하는 지원 효과:

- `Medical`
- `Rest`

지원 노드 공통 규칙:

- 한 지원 노드의 최종 효과는 하나다.
- `Random` 지원 노드는 사용하지 않는다.
- `Supply`, `Communications`, `Containment`, `Intel`은 현재 지원 노드 정규 효과로 사용하지 않는다.
- 대상이 없거나 효과가 없어도 노드는 소비된다.
- 지원 노드는 Safezone이 아니다. 진입 전 장비/파편 조정 장면과 노드 완료 시 적용되는 지원 효과를 섞지 않는다.

Medical:

- 직원 1명을 대상으로 한다.
- `EmergencyCare`는 HP를 회복한다.
- `Counseling`은 트라우마를 회복한다.
- `BalancedCare`는 HP와 트라우마를 절반씩 회복한다.

Rest:

- 살아있는 직원 전체에게 적용된다.
- 트라우마를 소량 회복한다.
- HP는 회복하지 않는다.
- `Rest` 효과는 노드 내부에서 `complete_node`를 선택할 때 적용된다.
- Safezone의 휴식풍 연출이나 다음 노드 준비 화면은 이 `Rest` 효과를 자동 적용하지 않는다.

## Maintenance 노드

Maintenance는 지원 노드가 아니라 독립 Safe Node다. core에서는 `MaintenanceState`와 `selected_event.type == "maintenance"`로 표현한다.

- 정비 작업 공간이다.
- 주 기능은 스킬 파편/장비의 분쇄, 강화, 개화다.
- 하단 작업 버튼은 `분쇄`, `강화`, `개화` 3개만 둔다.
- 스킬 파편을 분쇄하면 파편 가루를 획득한다.
- 무기, 방어구, 악세서리를 분쇄하면 장비 가루를 획득한다.
- 스킬 파편 강화는 파편 가루를 소비한다.
- 무기, 방어구, 악세서리 강화는 장비 가루를 소비한다.
- 개화는 1차 정책에서 스킬 파편 전용 작업이다. 무기, 방어구, 악세서리 선택 시 개화는 disabled 처리한다.
- 장착 중인 스킬 파편과 장착 중인 장비도 분쇄할 수 있다. Unity가 경고창/확인 단계를 제공하고, core는 확인된 command로 보고 자동 해제 후 분쇄한다.
- Maintenance 내부에서도 장비/스킬 파편 장착 교체를 할 수 있다. 이는 주 기능이 아니라 정비 중 편의를 위한 부가 기능이며, Loadout과 동일한 호환/교체 validation을 따른다.
- 호환되지 않는 드롭은 command를 보내지 않고 Unity에서 즉시 실패 피드백을 표시한다.
- 플레이어가 노드 완료를 선택하기 전까지 여러 정비 행동을 반복할 수 있다.
- 노드 완료를 선택하면 Maintenance 노드는 소비되고 다음 맵 진행으로 돌아간다.
- Safezone의 Loadout과 구분한다. Loadout은 장비/스킬 파편 세팅 전용 장면이고, Maintenance는 노드를 소비하는 성장/정비 화면이다.
- 화면은 3패널이다. 좌측은 직원 로스터와 장착품 목록, 중앙은 선택 대상의 작업 전/후 프리뷰, 우측은 가방이다.
- 좌측 직원 카드는 세로 목록이며, 각 카드에는 초상화와 장착 중인 스킬 파편/무기/방어구/악세서리 목록을 표시한다. 장착품 list element를 클릭하면 해당 장착품이 선택되고 중앙 프리뷰에 표시된다.
- 중앙 프리뷰는 현재 선택 대상, 대상 출처, 선택 작업, 작업 전/후 변화, 소모/획득 재료, 실행 가능/불가능 사유, 실행 전 확인을 표시한다. 대상 출처는 `가방에서 선택됨`과 `특정 직원이 장착 중`을 구분한다.
- 우측 가방은 Item Use/Loadout의 슬롯 grid와 scroll 구조를 재사용하되, 카테고리는 `스킬 파편`, `무기`, `방어구`, `악세서리` 4종이다.
- 클릭만으로 분쇄/강화/개화가 바로 실행되면 안 된다. 클릭은 대상 선택과 중앙 프리뷰 갱신만 수행한다.
- 실제 작업은 중앙 프리뷰의 실행 전 확인을 거쳐야 한다.
- 우하단 `나가기` 버튼은 `CompleteNode`에 매핑된다.
- Unity가 비용, 결과, 가능 여부를 직접 계산하지 않도록 Maintenance snapshot은 선택 가능 대상별 작업 가능 여부, disabled reason, 비용 preview, 획득 preview, before/after preview, 확인 필요 여부, 자동 해제 여부를 제공해야 한다.

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

모든 전투 보상은 현재 기본 정책에서 자동 지급한다. 선택형 보상은 아직 정규 흐름에 넣지 않는다. 공식 전투 흐름에서는 `DefenseRoute` live battle이 종료되면 전투 결과 처리와 노드 완료가 이어진다. 별도의 전투 보상 수령/확인 상태는 사용하지 않는다.

전투/방어/회수 실패는 곧바로 런 실패가 아니다.

- 전투 패배, 방어 실패, 회수 실패처럼 퇴각 없이 실패 조건이 확정된 경우에는 이상현상 노드를 소비하고 다음 선택 가능한 노드를 연다.
- 퇴각으로 종료한 전투는 실패 보상을 지급하지 않지만, 남은 시도 횟수가 있으면 같은 이상현상에 다시 진입할 수 있다.
- 같은 이상현상의 3번의 시도가 모두 사용되면 이상현상은 사라지고 노드는 소비된다.
- 실패한 노드는 연구 진행도와 핵심 보상을 지급하지 않는다. 단, 전투에 참여한 직원 경험치처럼 직원 성장 요소 일부는 지급할 수 있다.
- 전투 중 `Battle HP`가 0이 된 직원은 즉시 사망하지 않고 전투불능 상태가 된다.
- 전투불능이 된 직원에게만 트라우마 증가와 `Run HP` 감소가 적용된다. 임무 실패 자체가 모든 직원에게 트라우마를 주지는 않는다.
- 트라우마가 임계치를 넘거나 별도 사망 조건이 충족될 때 장기 손실 또는 사망으로 이어진다.
- 런 실패 판정은 노드 실패가 아니라 남은 직원과 다음 선택 가능한 회복 경로를 기준으로 한다.
- 보스 환상체 전투에서 패배하면 즉시 런 실패다.

## 후퇴

후퇴는 전투를 이득으로 바꾸는 버튼이 아니라, 배치 실수나 전력 부족을 전투 중 확인했을 때 장기 손실을 줄이는 손절 수단이다.

기본 정책:

- 후퇴는 모든 비보스 `DefenseRoute` 전투에서 가능하다.
- 보스 전투와 특정 강제 이벤트 전투에서는 후퇴할 수 없다.
- 후퇴 가능 시점은 `BattleEnd`가 발생하기 전까지다. 모든 아군 유닛이 전투불능 상태여도 `BattleEnd`가 아직 확정되지 않았다면 후퇴할 수 있다.
- 일시정지 중, 스킬 시전 중, 웨이브 진행 중에도 후퇴할 수 있다. 단, `BattleEnd` 이후 결과 처리 상태에서는 후퇴할 수 없다.
- 후퇴를 선택하면 전투는 즉시 중단되고 현재 이상현상 진입 시도 1회를 소모한다.
- 남은 시도 횟수가 있으면 노드는 소비되지 않고, 플레이어는 Safezone/NodeConfirm으로 돌아가 로드아웃, 소비 아이템, 출전 직원, 배치 계획을 조정한 뒤 재진입할 수 있다.
- 후퇴로 3번째 시도까지 모두 사용되면 이상현상은 스스로 사라지고 해당 노드는 소비된다.
- 후퇴한 진입 시도는 핵심 보상, 스킬 파편 연구 진행도, 임무 성공 보상을 지급하지 않는다.
- 후퇴 자체는 직원을 전투불능 처리하지 않는다.
- 후퇴 자체로 전원 트라우마를 크게 올리지는 않는다. 다만 추후 신뢰도/서사 시스템에서 위험한 작전 투입 후 후퇴에 대한 낮은 강도의 반응이나 작전 손실을 줄 수 있다.
- 후퇴 후에도 런 실패 판정은 일반 실패와 동일하게 남은 직원과 다음 선택 가능한 회복 경로를 기준으로 한다.

방어형 후퇴:

- 블랙박스/회수 장치가 살아 있어도 작전을 포기한 것으로 본다.
- 블랙박스 회수 실패이며, 연구 진행도는 지급하지 않는다.

회수/탈출형 후퇴:

- `Recovery`/`DefendAndEscape`는 공식 전투 모드에서 내렸으므로, 기존 회수 체크포인트 진행도 정책은 추후 재작성 전까지 사용하지 않는다.

HP 정책:

- `Run HP`는 런 전체에 유지되는 직원의 실제 체력이다.
- `Battle HP`는 전투 노드에서만 쓰는 전투용 체력이다.
- 전투 시작 시 `기본 전투 컨디션 60% + Run HP / Max HP 비율 40%`를 기준으로 `Battle HP`를 산출하고, 여기에 트라우마 임계치까지 남은 비율을 곱한다.
- 전투 종료 후 생존자의 남은 `Battle HP`는 `Run HP`에 그대로 되쓰지 않는다.
- 전투불능 시에만 조정 가능한 정책값에 따라 `Run HP`가 감소하고 트라우마가 증가한다.

## 런 실패

- 살아있는 직원이 한 명도 없으면 런 실패다.
- 출전 가능한 직원이 없고, 선택 가능한 회복 가능 노드도 없으면 런 실패다.
- 출전 가능한 직원이 없어도 `Medical`처럼 회복 가능성이 있는 노드가 남아 있으면 런은 지속된다.
- 출전 가능한 직원이 없는 상태에서 전투/보스 노드 진입을 확정하려 하면 진입이 차단된다.
- `Rest`는 HP를 회복하지 않으므로 HP 0 직원을 출전 가능 상태로 되돌리는 회복 노드로 보지 않는다.
- `Maintenance`는 정비 노드이므로 HP 회복 가능성으로 보지 않는다.
- 보스 환상체 전투 패배는 남은 회복 경로와 무관하게 런 실패다.

## 스킬 파편

- 기본 직원은 고유 액티브 스킬을 갖지 않는다.
- 직원은 스킬 파편을 장착해야 대표 액티브 스킬을 사용할 수 있다.
- 액티브 스킬 파편 슬롯은 기본적으로 1개를 전제로 한다.
- 환상체가 쓰는 원본 스킬과, 직원이 파편으로 모방하는 스킬은 별도 스킬 데이터다.
- 사용하지 않는 파편은 강화, 개화, 분쇄, 연구, 제한 조합 같은 자원 순환으로 활용한다.
- 파편 분쇄는 `Maintenance`에서 수행한다.
- 스킬 파편은 패시브 장비가 아니다. 평타 강화, 일시 스탯 증가, 조건부 버프, 공명 변화는 상시 효과가 아니라 액티브 스킬 발동 결과로 표현한다.
- 장착/해제는 `ViewingMap`, Safezone, 전투/Boss 노드의 `NodeConfirm`처럼 노드 진입 전 준비 구간에서 가능하다. Maintenance 노드 내부에서는 정비 편의 기능으로 장착 교체를 허용한다.
- 전투, 전투 결과 처리, 보상 처리 중에는 전투 결과를 보고 즉석으로 파편을 바꾸지 않는다.
- 같은 환상체가 여러 번 존재하는 보상 구조는 피한다. 성장 재료의 기본값은 동일 파편 복수 획득이 아니라 같은 rarity의 다른 파편이다.
- 스킬 파편 강화는 파편 가루를 소비한다. 스킬 파편 개화 비용은 파편별 데이터가 정하되, 1차 정책에서는 스킬 파편 전용 작업으로 유지한다.
- 개화는 자동으로 일어나지 않는다. 조건 충족 후 별도 액션으로 확정하며, 되돌릴 수 없다.
- 개화 결과는 파편별 데이터가 결정한다. 원본 환상체 스킬에 가까워질 수도 있고, 전혀 다른 직원 전용 파생 스킬이 될 수도 있다.

스킬 파편 장착 조건:

- 스킬 파편은 직원의 고정 직업명을 요구하지 않는다.
- 스킬 파편은 장착 중인 무기가 만든 전투 프로필을 기준으로 장착 가능 여부를 판단한다.
- 파편 요구 조건은 `range_role`, `weapon_archetype`, `damage_type`, `targeting_profile`, `air_capable`, `block_capacity_min`, `required_capability_tags`, `incompatible_tags` 같은 축으로 표현한다.
- 근거리 전용 파편과 원거리 전용 파편은 허용한다. 단, 모든 파편을 모든 무기 아키타입에 억지로 대응시키지 않는다.
- 유명하거나 스킬 파편으로 녹여내기 좋은 환상체 파편은 특정 무기 아키타입 또는 아키타입 그룹 전용으로 설계해 개성을 살린다.
- 환상체 정체성이 약하거나 보조/안정화 역할이 강한 파편은 범용 파편으로 둔다.
- 강한 환상체 정체성을 가진 파편일수록 요구 조건을 좁게 둔다. 범용 강화/보조 파편은 요구 조건을 느슨하게 둔다.
- 아키타입 제한은 벌칙이 아니라 빌드 목표다. 플레이어는 특정 파편을 쓰기 위해 맞는 무기를 준비하고, 무기 교체로 직원의 역할을 바꾼다.
- 보상 시스템은 현재 보유 무기/아키타입과 완전히 동떨어진 전용 파편만 반복 지급하지 않도록 후보 풀 보정 또는 연구 진행도 보완을 제공해야 한다.
- 파편은 기본적으로 환상체 테마 효과 모듈이다. 무기 아키타입은 그 효과가 발현 가능한지와 어떤 보정이 붙는지를 결정한다.
- 파편은 필요하면 `application_mode`를 가진다. 예시는 `BasicAttackModifier`, `ActiveSkill`, `PassiveAura`, `OnBlock`, `OnKill`, `OnHit`, `Periodic`이다.
- 파편은 필요하면 제한적인 variant rule을 가진다. 예: 유탄 아키타입이면 폭발 반경 증가, 근거리면 `OnBlock` 발동, 원거리면 `OnHit` 발동, 마법 무기면 마법 저항 감소 추가.
- 파편 하나의 variant는 1~2개 수준으로 제한한다. 무기마다 별도 스킬을 전부 작성하는 방식은 피한다.
- 장착 가능 여부의 최종 source of truth는 core validation이다. Unity는 서버가 제공하는 호환성/슬롯 정보를 표시하고, 세부 규칙을 추론하지 않는다.

## 직원 신뢰도

- 신뢰도는 기본적으로 전투 수치보다 기억, 대사, 판단, 반응의 기준이다.
- 신뢰도는 선택형 부가 시스템이 아니라 직원 운용과 감정선을 구성하는 핵심 시스템이다.
- 일반 전투 조작을 자주 방해하지 않는다.
- 위험 선택, 조기 개화, 위험 파편, 부상 재출전 같은 고위험 상황에서만 강하게 반응한다.
- 기본 구현 방향은 내러티브 중심이다. 기억, 최근 반응, 대사/자막 cue는 기본 흐름에 포함하고, 명령 거부와 전투 수치 보정은 별도 정책으로 제한한다.
- 신뢰도 상승은 단순 생존 조치보다 누적된 애정과 투자에 더 민감해야 한다. 전투불능 직후 치료는 이성적 손실 방지에 가깝기 때문에 상승폭을 낮게 둔다.
- 위험 노드 전 휴식, 꾸준한 장비/파편/강화 투자, 약속한 회복/보상 선택을 지킨 기록은 더 강한 애정 신호로 본다.
- 부상 상태 반복 출전, 전투불능 방치, 조기 개화/위험 파편 강요, 동료 사망 직후 위험 노드 투입은 신뢰도 하락 후보가 된다.
- 기본 정책은 `memory_enabled`와 `dialogue_enabled`가 켜진 내러티브 중심이다. `high_risk_acceptance_enabled`, `combat_modifier_enabled`, `trauma_modifier_enabled`는 밸런스가 준비된 범위에서만 켠다.
- 신뢰도 변화 계산은 중앙 정책/Resolver에서 처리한다. 이벤트, 지원 노드, 스킬 파편 코드가 직접 점수를 중복 계산하지 않는다.

## 레거시 이스터에그

- 이전 런에서 플레이어가 진심으로 돌본 직원이 사망하면, 조건을 만족할 때 최대 3개까지 레거시 기록으로 저장할 수 있다.
- 아무 사망 직원이나 저장하지 않고, 신뢰도, 성장, 장비, 파편, 출전 기록 같은 애착/투자 기준을 통과한 직원만 후보가 된다.
- 이후 관련 환상체 전투 노드에서 설치/계정 단위 1회성 특수 선택지 `잔향 추적`이 나타날 수 있다. 발생 모드와 시점은 추후 정책으로 확정한다.
- `잔향 추적`을 선택하면 원래 조우에는 없던 침식된 전 직원 엘리트가 추가된다.
- 플레이어가 표준 진압을 선택하면 이스터에그는 소비되지 않는다.
- 성공 보상은 직원의 변질 장비, 생존일지, 연구도, 아카이브 기록 중심이다.
- 이 시스템은 일반 반복 콘텐츠가 아니라 희귀한 기억 이벤트다.
- 현재 구현 우선순위가 아니다. 상세 발생 조건, 저장 기준, 보상, 연출은 추후 별도 정책으로 다시 확정한다.
