# 시설 맵 템플릿·노드 배정·조건부 발현 정책 — 2026-07-25/26

- 논의·작성일: 2026-07-25~26
- 분리일: 2026-07-25
- 마지막 정책 검토: 2026-07-27
- 상태: **Policy closed / P03 구현 완료 (2026-07-27)**
- 문서 등급: 구현 이력·정책 근거 문서
- 분리 출처:
  [save_point_trauma_recovery_policy_draft_2026-07-25.md](save_point_trauma_recovery_policy_draft_2026-07-25.md)
- 주의: 현재 runtime, live RON, save/wire 계약의 source of truth는 `archive/game_rulebook_legacy_2026-10-04.md`,
  `core_runtime_contract.md`, `data_loading_contract.md`와 외부 Unity canonical이다. 이 문서는 P03이
  구현한 승인 정책의 역사적 입력으로 유지한다. 완료 근거는
  `../goals/run_policy_phase_03_facility_map_manifestation/COMPLETION_REVIEW.md`에 있다.

## 1. 목적과 범위

기존 depth/lane 기반 절차적 graph generator를 제거하고, 시설 맵 템플릿이 방 수, room slot,
허용 node category와 corridor/connection topology를 함께 소유하는 구조로 전환한다. Core는 선택한
템플릿의 고정 topology 위에서 결정론적으로 node content를 배정하고 navigation을 계산하며,
Unity는 동일 template identity에 대응하는 정적 시설 도면 위에 Core marker 상태만 합성한다.

이 문서는 다음을 결정한다.

- authored room slot과 connection topology의 authority
- authored room slot tag와 조건부 assignment role의 authority
- mode·Floor 조건에 따른 template pool과 결정론적 소진·초기화 규칙
- 결정론적 constrained node 배정과 validation failure 정책
- Standard Floor별 assignment profile과 Endless 구간·cycle profile
- 완료 room을 transit으로 사용하는 navigation과 direction 의미
- Core 내부 template와 Unity-local 정적 도면의 책임 분리
- SavePoint 배치 주기와 template slot 요구사항
- BossOmen chain 완료 뒤 전용 capstone map으로 전환하는 규칙
- 선행 node 성공 집합에 따른 선택형 보너스 node의 조건부 발현
- Standard의 입문형 -> 분기형 -> 최종형 template progression과 `Elite -> Elite -> Boss` 학습 순서
- 기존 절차적 graph, Support 보장·가중치·repair 규칙의 제거

SavePoint의 완료 효과, Trauma 회복, checkpoint와 Run Failed 정책은 분리 출처 문서가 소유한다.
이 문서는 SavePoint를 시설 맵에 배정하는 조건만 소유한다.

## 2. 시설 맵 템플릿 authority

- 선택된 시설 맵 템플릿은 방의 수, 각 room slot의 identity·위치·허용 node category와 방 사이
  corridor/connection을 함께 명시한다. `allowed_categories`가 하나뿐인 slot은 사실상 고정 category며,
  여러 개인 slot은 그 집합 안에서만 node를 배정할 수 있다.
- 실제 탐사 대상으로 authored된 모든 room slot에는 정확히 하나의 node를 배정한다. 배경에만 그려진
  장식 공간은 room slot으로 authoring하지 않는다. 따라서 한 Floor의 node 수는 선택한 템플릿의
  room slot 수가 결정한다.
- Core는 선택한 템플릿에 이미 authored된 corridor/connection topology를 그대로 사용하며 별도의
  런 graph를 다시 생성하지 않는다. 구현상 탐색 index/cache를 만들 수는 있지만 두 번째 authority나
  checkpoint에 중복 저장되는 독립 graph가 되어서는 안 된다.
- 위치만으로는 벽 너머의 가까운 방과 실제 연결된 먼 방을 구분할 수 없으므로 topology 자체는
  필요하다. 각 connection은 slot identity 쌍과 `Bidirectional` 또는 `ForwardOnly` direction을 가지며,
  Core는 이미지 픽셀이나 좌표에서 연결을 추론하지 않는다.
- direction은 runtime node 하나의 단일 field가 아니라 template connection의 속성이다. 한 room에는
  방향이 서로 다른 여러 출입구가 있을 수 있다. Runtime node는 자신의 `slot_id`로 template
  connection을 조회하며 node별 outgoing/adjacency 복사본을 두 번째 authority로 저장하지 않는다.
- Room slot의 `tags`는 topology나 node category가 아니라 assignment·조건부 시스템이 사용할 수 있는
  authored semantic role/capability다. `allowed_categories`는 배정 가능한 category를 제한하고,
  `tags`는 그중 `terminal_guard`, `Shop` 조건부 보장처럼 특정 정책이 사용할 수 있는 위치를
  표시한다. Core만 tag를 해석하며 Unity는 tag로 category나 reachability를 재계산하지 않는다.

### 2.1 Template pool과 선택

- 일반 탐사 시설 템플릿은 assignment schedule이 참조하는 `template_pool_id`별 pool로 관리한다.
  Core는 run seed와 pool 순환 번호로 후보를 결정론적으로 한 번 섞고 Floor 전환마다 앞에서 하나씩
  꺼내 사용한다.
- 한 순환에서 꺼낸 템플릿은 pool이 소진되기 전에는 다시 넣지 않는다. pool이 소진되면 순환 번호를
  증가시키고 현재 mode에 맞는 pool을 다시 구성·셔플한다.
- 현재 Floor의 assignment profile과 필수 category 조건을 만족할 수 없는 템플릿은 그 Floor의 유효
  후보가 아니다. 남은 pool에 유효 후보가 없으면 현재 조건으로 pool을 다시 구성하되, 유효 템플릿이
  하나도 없으면 다른 mode의 템플릿이나 구 generator로 fallback하지 않고 validation error로 처리한다.
- pool의 남은 순서와 순환 번호는 run/checkpoint 결정 상태다. 같은 run seed, mode, Floor 진행과
  checkpoint state는 같은 template 선택 순서를 재현해야 한다.
- 하나의 template가 여러 pool에 속하더라도 같은 run에서의 사용 여부는 template id 기준으로
  추적한다. 현재 조건을 만족하는 미사용 template가 남아 있는 동안 이미 사용한 template를 다시
  선택하지 않는다.

### 2.2 Standard template progression

- Standard는 세 Floor의 학습 목적에 맞는 전용 pool을 사용한다.
  - Floor 1 `standard_intro`: 가장 작은 입문형 map, 짧은 분기와 합류, `ForwardOnly` 없음
  - Floor 2 `standard_branching`: 더 큰 분기형 map, 선택적 SavePoint와 `ForwardOnly` connection 정확히 하나
  - Floor 3 `standard_final`: Standard에서 가장 크고 복잡한 최종형 map, Final Boss terminal
- 각 pool의 실제 room 수는 live RON 콘텐츠가 소유한다. 다만 validation은 intro, branching, final의
  authored room-count band가 순서대로 증가해 진행할수록 map이 커진다는 관계를 보장한다.
- Floor 1은 node 선택과 완료 room transit을, Floor 2는 더 긴 분기·되돌아가기·SavePoint 포기와
  명시적인 일방통행을, Floor 3은 복잡한 분기의 최종 수렴과 Final Boss를 학습시킨다.
- Floor 3은 복수 `ForwardOnly` connection을 허용하지만 모든 합법적인 진행에서 유효 terminal을
  잃는 topology deadlock은 금지한다.

### 2.3 공통 엔진과 mode별 policy

- Standard와 Endless는 별도 map engine을 구현하지 않는다. facility template schema, template pool,
  constrained assignment solver, navigation, checkpoint/snapshot과 Unity 계약은 공통으로 사용한다.
- Mode 차이는 live RON의 schedule, template pool과 assignment profile이 소유한다. Mode별 분기를
  generator, node executor나 Unity presenter 여러 곳에 하드코딩하지 않는다.
- Standard는 3-Floor curated tutorial policy다. Floor별 authored schedule과 필수 학습 role/category를
  먼저 만족시키고, 무작위성은 비필수 slot category, 실제 node kind/content와 encounter 선택에만
  허용한다.
- Endless는 open-ended exploration policy다. Floor stage/cycle, 확장되는 template pool과 category
  weight를 중심으로 변주하며 Standard profile이나 tutorial guarantee로 fallback하지 않는다.
- Standard template/profile에는 의문의 떠돌이 상인을 배정하지 않는다. Headquarters support Shop은
  별도 context로 유지한다.
- Endless의 의문의 떠돌이 상인은 Floor 1~3에 배정하지 않고 Floor 4부터만 일반·조건부 후보가 된다.

## 3. Node 배정

- 기존 `MapGenerator`가 소유하던 topology 생성과 node 배정을 분리한다. 새 node assignment pipeline은
  선택된 authored template의 topology를 변경하지 않고 모든 room slot의 node category와 실제
  kind/content만 결정한다.
- live RON은 최소 다음 세 authority를 분리한다.
  - `facility_templates.ron`: room slot, `allowed_categories`, connection topology와 template eligibility
  - `node_assignment_profiles.ron`: mode·Floor별 category `min`/`max`/`exact` 제약과 남은 slot의 weight
  - `node_definitions.ron`: category 안에서 선택할 실제 node kind/content 후보와 후보 weight
- Standard는 각 Floor에 사용할 assignment profile을 명시적으로 지정한다. 현재 3-Floor Standard는
  Floor 1, 2, 3의 profile을 각각 저작하며 profile 누락이나 중복 매칭을 허용하지 않는다.
- Endless는 모든 Floor를 나열하지 않는다. authored Floor 구간별 profile stage를 두고 각 stage 안에서
  필요한 홀수/짝수 등 profile cycle을 반복한다. 마지막 open-ended stage가 이후 Floor를 명시적으로
  소유하며 암묵적인 clamp나 기본 profile fallback을 사용하지 않는다.
- Core는 Floor seed를 사용하는 결정론적 constrained matching으로 category `exact`, `min`, `max`와
  slot별 `allowed_categories`를 먼저 모두 만족시킨다. 그 뒤 아직 category가 정해지지 않은 slot만
  profile weight로 채우고, 각 category 안의 실제 kind/content는 node definition weight로 선택한다.
- 일반 탐사 template의 필수 Start·terminal과 해당 Floor profile의 SavePoint 및 기타 category 제약을
  모두 만족하는 완전 배정이 없으면 다른 category fallback, 사후 repair나 빈 방을 만들지 않고
  template/content validation error로 처리한다.
- Endless의 run-local 자연 Elite pool이 모두 소비되면 Combat slot은 같은 profile이 저작한 일반
  Combat 후보를 사용한다. Profile은 `elite_pool_exhausted` 상태에서도 완전 배정이 가능해야 하며,
  소비된 Elite를 반복시키거나 category를 사후 repair하지 않는다.
- 가중치는 템플릿의 방 수·위치·연결을 바꾸거나 profile의 `min`/`max`/`exact`를 위반할 수 없다.
- Event 최소 개수처럼 모든 Floor에 공통이라고 가정하던 보장은 코드에 하드코딩하지 않는다. 필요한
  Floor profile이 Event `min` 또는 `exact`를 명시하며, 명시하지 않은 profile에서는 Event가 없을 수
  있다.
- Standard assignment profile은 다음 terminal 학습 순서를 고정한다.
  - Floor 1: `terminal_guard` slot의 Elite Combat 정확히 하나를 완료한 뒤 Gate
  - Floor 2: `terminal_guard` slot의 Elite Combat 정확히 하나를 완료한 뒤 Gate
  - Floor 3: Final Boss terminal
- Standard Floor 1·2 template의 모든 Start -> Gate 경로는 해당 Floor의 `terminal_guard`를 반드시
  통과해야 한다. profile은 guardian 이외의 slot에 추가 Elite를 무작위 배정하지 않는다. Floor 1과
  Floor 2의 난이도 차이는 서로 다른 Elite content pool과 Floor combat scaling으로 표현한다.

### 3.1 조건부 떠돌이 상인 Shop-tag 배정

- Elite 모노리스 정책에서 잔여 모노리스 네 개 조건을 처음 충족하면 현재 map을 수정하지 않고
  `monolith_exchange_pending`을 기록한다.
- 다음 Floor 진입 시 Core는 도달 가능한 `Shop` tag slot을 가진 Endless template만 유효 후보로
  취급하고, constrained assignment가 해당 tag slot 중 정확히 하나에 의문의 떠돌이 상인을 강제
  배정한다. 여러 후보는 Floor seed로 결정론적으로 하나를 선택한다.
- 이 보장은 Floor 4+에서만 동작한다. 조건을 만족하는 template가 없으면 node 추가·기존 node 교체,
  다른 mode template나 구 generator fallback을 사용하지 않고 validation/content error다.
- 보장 Shop을 방문하지 않고 Floor를 떠나도 같은 보장 배정을 반복하지 않는다. 다만 교환 VN을
  실제로 보지 않았다면 이후 무작위 떠돌이 상인에게 제안받을 수 있다.
- 교환 eligibility, VN 일회성, 잔여분 소비와 변질된 모노리스는
  [elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md](elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md)가
  소유한다.

### 3.2 Standard node 학습 schedule

- Standard의 category 보장은 플레이어에게 node를 맵에 노출하는 정책과 반드시 선택·완료하게 하는
  정책을 구분한다. Elite guardian과 Final Boss만 진행상 필수며, Safe Node·Event·Reward는 명시된
  Floor에 반드시 배치하되 선택은 강제하지 않는다.
- Floor 1은 기본 탐사와 전투·결과 읽기를 소개한다.
  - Start와 terminal guardian 사이의 필수 backbone에 일반 Combat을 최소 하나 둔다.
  - Event를 최소 하나, Reward를 최소 하나 배치해 선택형 사건과 독립 보상 node를 노출한다.
  - Elite guardian 정확히 하나와 Gate 정확히 하나를 둔다.
  - SavePoint와 Maintenance는 배치하지 않아 첫 Floor의 학습 범위를 제한한다.
- Floor 2는 run 자원 관리와 회복 선택을 소개한다.
  - 일반 Combat을 최소 하나 둔다.
  - SavePoint 정확히 하나, Maintenance 최소 하나, HeadquartersContact 최소 하나와 Shop 최소 하나를
    배치한다. 모두 선택형 분기에 둘 수 있으며 Gate 진행을 위해 이용을 강제하지 않는다.
  - Elite guardian 정확히 하나와 Gate 정확히 하나를 둔다.
  - `ForwardOnly` connection 정확히 하나를 정적 방향 표시와 함께 사용한다.
- Floor 3은 앞서 본 node를 조합하고 Final Boss 준비·결정을 검증한다.
  - 새 category를 추가로 의무 소개하지 않고 Floor 1·2에서 소개한 일반 category의 profile weight로
    비terminal slot을 채운다.
  - SavePoint는 배치하지 않고 Final Boss terminal 정확히 하나를 둔다.
  - profile은 Elite를 추가 배정하지 않는다. Standard의 Elite 학습은 Floor 1·2 guardian이 소유한다.
- 위 schedule의 실제 category weight, 일반 Combat 수의 상한과 room-count band는 live RON의 초기
  밸런스 값이며 정책 의미를 바꾸지 않는 범위에서 플레이테스트로 조정할 수 있다.
- Combat slot의 Easy/Standard/Hard difficulty-band quota와 실제 encounter 난이도 산정은 이 문서가
  직접 소유하지 않는다. Assignment profile은 별도 전투력 정책의 `difficulty_profile_id`와 band
  quota를 참조하며 세부 규칙은
  [combat_power_floor_difficulty_policy_draft_2026-07-26.md](combat_power_floor_difficulty_policy_draft_2026-07-26.md)를
  따른다.

### 3.3 조건부 보너스 node 발현

- 이 정책에서 퍼즐은 같은 map에 등록된 선행 node들을 모두 성공한 뒤 authored 위치에 선택형 후속
  node 하나가 발현되는 구조다. 열쇠·자원, Event 결과, 임의 script와 범용 topology mutation은
  포함하지 않는다.
- 각 발현 규칙은 비어 있지 않은 `prerequisite_node_ids` 집합과 정확히 하나의
  `manifest_node_id`를 가진다.
- v1 조건은 Floor 시작 시 이미 발현된 선행 node의 성공 결과만 참조하며 모든 조건을 충족하는 AND
  방식이다. `AnyOf`, Event outcome, 아이템·자원 수량과 외부 run 상태는 지원하지 않는다.
- Combat 승리나 해당 node contract가 정의한 성공 terminal outcome만 선행 성공으로 인정한다.
  Retreat, 실패, 방문, 선택 또는 단순 진입은 인정하지 않는다.
- 마지막 선행 성공을 commit하는 Core transaction 직후 조건을 평가하고, 충족되면 후속 node 발현을
  같은 authoritative 결과에 단 한 번 commit한다. 취소·실패한 transaction은 발현을 일으키지 않는다.
- 선행 node는 성공·완료 상태로 남고 발현 뒤 다시 소비하거나 초기화하지 않는다. 발현은 같은
  timeline 안에서 재잠금되지 않는 단조 전환이다.
- 후속 node의 slot, category/kind/content, difficulty와 reward/encounter seed는 Floor 생성
  transaction에서 미리 확정한다. 조건 충족 시 새 콘텐츠를 추첨하거나 reroll하지 않는다.
- 발현 node는 기존 topology가 미리 승인한 connection과 함께 나타나며 이후 일반 node와 같은
  selectable·진입 규칙을 따른다. 발현 transaction은 room·door·connection을 임의 생성하거나 방향을
  변경하지 않는다.
- 발현 node는 선택형 보너스 콘텐츠로만 사용한다. Reward, Event와 특수 Combat처럼 건너뛰어도 run을
  완료할 수 있는 category만 template profile이 명시적으로 허용한다.
- Gate, SavePoint, 필수 Boss와 그 밖의 mandatory progression node는 조건부 발현 대상으로 사용할 수
  없다. 발현 실패나 미발견 때문에 Floor 또는 run이 softlock되어서는 안 된다.
- 첫 canonical 사례는 처음부터 도달 가능한 두 보조 제어실을 순서와 관계없이 모두 성공하면 authored
  위치에 선택형 보너스 node 하나가 발현되는 map이다.

## 4. Template validation

- Template/runtime 계약은 terminal을 단일 id로 제한하지 않고 `1..N` typed terminal slot/node 집합으로
  표현한다. 각 terminal은 `FloorGate`, `RunFinalBoss`, `BossOmenExitGate` 등 완료 resolution을 가진다.
- 템플릿 검증은 모든 도달 가능한 비terminal slot에서 direction을 지키며 유효 terminal 중 하나
  이상까지 진행할 경로가 있음을 보장한다.
- 어떤 합법적인 node 완료 순서도 topology만으로 플레이어를 진행 불능 위치에 가두지 않아야 한다.
- 의도된 run failure는 node content와 typed reason으로 만들며 우연한 `ForwardOnly` deadlock으로
  표현하지 않는다.
- SavePoint 후보 slot은 Start에서 도달 가능하고 terminal까지 계속 진행할 수 있어야 한다.
- Runtime은 depth/lane, 최단경로나 terminal과의 거리로 SavePoint 위치를 추론하지 않는다.
- Standard Floor schedule은 모든 Floor에 정확히 하나의 profile이 대응하는지 검증한다. Endless
  schedule은 구간이 겹치거나 비지 않고 open-ended 마지막 stage와 유효한 cycle을 가지는지 검증한다.
- 각 profile의 `min`/`max`/`exact` 조합은 모순이 없어야 하며, template eligibility 검증은 해당
  profile을 만족하는 완전 slot 배정이 실제로 하나 이상 존재함을 보장한다.
- Slot tag ID는 비어 있지 않고 template 안에서 허용된 authored tag catalog에 속해야 한다. 조건부
  떠돌이 상인 보장 대상 Endless template는 도달 가능하고 terminal 진행을 막지 않는 `Shop` tag
  slot을 하나 이상 가져야 하며 Standard template에는 이 조건부 role을 허용하지 않는다.
- terminal 하나가 완료되어 resolution이 확정되면 같은 Floor의 나머지 terminal은 폐쇄한다. 서로
  다른 다음 지역·보상으로 이어지는 분기 terminal의 destination 정책은 후속 범위다.
- 현재 Standard 기본 콘텐츠는 학습 흐름을 명확히 하기 위해 Floor마다 terminal 하나만 저작하지만
  schema, runtime과 validation을 단일 `terminal_node_id`에 결합하지 않는다.
- Standard Floor 1·2 template는 `terminal_guard`를 제거한 topology에서 Start가 Gate에 도달할 수
  없어야 한다. 이 dominator 조건으로 어떤 분기를 택해도 Elite 학습을 우회할 수 없음을 검증한다.
- 조건부 발현 검증은 존재하지 않는 node/slot, 빈·중복 prerequisite, 자기 참조, 중복 manifest target,
  처음부터 도달할 수 없는 선행 node와 발현 node 없이는 달성할 수 없는 선행 조건을 거부한다.
- v1은 연쇄 발현을 허용하지 않는다. manifest node는 다른 발현 규칙의 prerequisite가 될 수 없으며,
  prerequisite는 모두 Floor 시작 시 발현돼 있어야 한다.
- puzzle template는 명시적인 eligibility tag와 전용 template pool/profile을 사용한다. 일반 Standard
  template pool에 암묵적으로 섞거나 일반 assignment가 manifest slot을 다른 category로 덮어쓰지 않는다.

## 5. Map navigation

- 기존 map navigation 계약을 유지한다. 완료된 방은 content를 다시 실행하거나 직접 선택하는
  목적지가 아니라 Core reachability 계산에서만 통과할 수 있는 transit 지점이다.
- Core는 현재 위치에서 완료된 방들로만 이어지는 경로를 통과해 도달 가능한 미해결 `Available`
  방을 최종 `map_navigation.selectable_node_ids`로 투영한다.
- 되돌아가기는 Floor 전체의 단일 on/off 규칙이 아니다. Core 내부 연결이 `Bidirectional`이면 완료
  경로를 양방향으로 통과할 수 있고, `ForwardOnly`이면 `from -> to` 방향으로만 통과할 수 있다.
- Unity는 완료 방으로 별도 이동 command를 보내거나 slot 좌표·복도 그림으로 연결과 선택 가능성을
  재계산하지 않는다.
- Gate NodeConfirm은 Floor 이동, 현재 Floor의 미완료 node 접근 상실과 authored Trauma recovery라는
  직접 결과를 한 번 표시한다. Confirm 자체를 단일 확인으로 사용하고 같은 내용의 2차 modal은
  만들지 않는다. 특정 SavePoint나 보상을 놓친다는 별도 경고·추천은 제공하지 않는다.

### 5.1 BossOmen capstone map 전환

- BossOmen chain의 모든 step이 완료되면 기존 일반 탐사 map에 node·slot·connection을 추가하거나
  기존 node를 덮어쓰지 않는다. Core는 현재 일반 탐사 map을 종료하고 같은 Floor의 authored
  BossOmen capstone map으로 원자적으로 전환한다.
- capstone template에는 별도 Boss room slot과 Exit Gate room slot만 존재하며 connection은
  `Boss -> Exit Gate` `ForwardOnly`다. 일반 탐사 template에는 BossOmen용 예약 slot을 두지 않는다.
- capstone map 진입 직후 Boss가 유일한 selectable node다. 이전 일반 탐사 map의 다른 node로
  돌아갈 수 없고 Safezone/loadout과 employee-targeted consumable처럼 NodeConfirm 준비 상태에서
  기존 정책이 허용하는 비map 준비 action만 사용할 수 있다.
- Boss 승리 전 Exit Gate node는 `Concealed`이며 selectable하지 않다. Boss 승리를 commit한 뒤 Gate와
  연결 경로를 공개하고 Exit Gate만 selectable로 만든다.
- Boss 승리 자체는 Floor를 전환하거나 Gate 회복을 적용하지 않는다. 공개된 Exit Gate를 확인할 때
  기존 Gate transition 정책으로 다음 Floor 이동과 회복을 적용한다.
- capstone map과 일반 탐사 map은 같은 template schema와 Core authority를 사용한다. Unity에 표시용
  template id만 바꾸고 Core가 구 map topology를 계속 사용하는 이중 상태를 허용하지 않는다.
- 유효한 SavePoint를 load하면 checkpoint에 저장된 일반 탐사 map과 template pool/assignment 상태로
  복구한다.
- SavePoint active-run 이어하기도 마지막 checkpoint의 일반 탐사/capstone map, template pool 순서,
  cycle과 assignment를 정확히 복원한다. Template을 다시 추첨하거나 현재 live weight로 node content를
  재배정하지 않는다.

### 5.2 변질된 모노리스 숨겨진 보스 capstone

- 변질된 모노리스와 일반 최종 보스 3종 격파 조건이 완료되면 모노리스 정책의 관리자 독백 VN과
  단일 선택을 거쳐 authored 흉탄의 사수 capstone map으로 원자 전환한다.
- 일반 최종 보스 3종은 모노리스 정책이 확정한 백야, 고요한 오케스트라와 종말새다.
- 일반 map에 node·slot·connection을 추가하거나 기존 node를 덮어쓰지 않는다는 authority는
  BossOmen capstone과 같다.
- 현재 흉탄의 사수 capstone template에는 Boss terminal room 하나만 존재한다. 승리하면 특별
  `RunComplete`를 확정하므로 Exit Gate room이나 다음 Floor connection을 저작하지 않는다.
- 진입 뒤 이전 일반 map으로 돌아갈 수 없으며 흉탄의 사수 패배는 모노리스 정책과 공통
  RunFailed/SavePoint 정책을 따른다.
- eligibility, VN, 전투 시도, 도감과 completion reason은
  [elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md](elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md)가
  소유한다.

## 6. Unity 표시 경계

- 방의 물리적 배치, 벽, 복도와 경로선은 Unity-local 시설 맵 이미지·템플릿 리소스가 정적으로
  표현한다.
- 일반 Standard template의 시설 도면, room과 corridor topology는 Floor 시작부터 전체를 표시한다.
  일반 room은 기본 `Revealed`이며 category marker를 표시하고, 상세 encounter·보상 preview는 Core가
  현재 selectable로 투영한 node를 선택했을 때만 연다.
- 일반 runtime Node Map DTO는 Core 내부 `edges`나 `travel_path`를 공개하지 않고 Unity도 동적 edge
  overlay를 그리지 않는다.
- Unity가 리소스 위에 합성하는 동적 요소는 현재 위치와 Available/Completed/Locked 등 Core가
  투영한 node marker 상태다.
- Unity-local template는 `ForwardOnly` connection을 플레이어가 평면도에서 구분할 수 있는 정적
  방향 표현을 가진다. 이는 Unity가 reachability를 계산하기 위한 data가 아니라 플레이어가 직접
  경로를 판단하기 위한 authored visual이다.
- `ForwardOnly`는 일반 통로와 다른 색에 더해 `from -> to` 방향의 화살표/chevron과 범례를 사용한다.
  색을 구분하지 못해도 선 모양과 방향 표식만으로 의미를 판별할 수 있어야 하며 별도 경고·확인
  modal은 만들지 않는다.
- BossOmen capstone template의 Exit Gate room과 연결 경로는 authored reveal layer로 분리한다.
  Unity는 Boss 승리 뒤 Core가 투영한 Gate visibility를 근거로 이 layer를 표시하지만, 연결이나 공개
  시점을 자체 계산하지 않는다.
- 일반 visibility 의미는 `Concealed`일 때 room marker와 authored reveal group을 숨기고, `Locked`일
  때 room과 경로는 표시하되 선택할 수 없는 것으로 구분한다. 현재 구현 범위에서 일반 Standard
  template는 비밀 방이나 조건부 잠금 방을 사용하지 않으며, BossOmen Exit Gate만 합의된
  `Concealed` reveal 사례다.
- `Obscured`는 room과 marker 자리는 표시하지만 category, 보상과 상세를 숨긴다. Core가 selectable로
  투영하면 정체를 모르는 상태로 선택할 수 있다. `Revealed`/`Obscured`/`Concealed`와
  `Available`/`Locked`/`Completed`는 서로 다른 visibility/progression 축이며 Unity가 하나에서 다른
  하나를 추론하지 않는다.
- Core facility template의 모든 room `slot_id`, marker anchor와 reveal group은 Unity-local resource와
  exact하게 대응해야 한다. 누락·중복·unknown slot 또는 reveal group은 임의 좌표나 숨김 fallback을
  사용하지 않고 cross-project validation 실패로 처리한다.

### 6.1 조건부 발현 표시와 동기화

- 발현 전 Core snapshot은 후속 node marker와 그 node에만 incident한 connection을 공개하지 않는다.
  빈 selectable slot이나 미확정 category도 노출하지 않는다.
- Unity-local authored reveal group은 빈 공간, 봉인 장치, 비활성 설비 같은 환경 단서로 발현 위치를
  암시할 수 있다. 이 단서는 node identity/category/reward를 공개하거나 Core selectable 판정에
  참여하지 않으며, 템플릿에 아무 단서가 없어도 계약상 유효하다.
- 정상 진행에서는 마지막 선행 성공 뒤 map으로 돌아올 때 reveal 연출과 접근성 대체 표시를 한 번
  재생한다. reconnect/resync와 load 직후에는 이벤트를 재생하지 않고 authoritative 최종 상태를 즉시
  복원한다.
- template id/version, 각 선행 성공 결과, 발현 여부와 미리 확정한 후속 node/connection 상태를
  active-run save와 checkpoint에 보존한다.
- checkpoint load는 같은 checkpoint에 저장된 선행 결과와 발현 상태를 함께 복원한다. 후속 node의
  완료, 보상과 inventory도 동일한 run-state 경계로 함께 되감겨 보상을 중복 보존할 수 없어야 한다.
- Continue와 reconnect는 저장된 현재 상태를 복원할 뿐 조건이나 콘텐츠를 재추첨하지 않는다.
- loader는 현재 설치본이 지원하지 않는 newer template/state version을 추측해 migration하지 않고
  명시적으로 거부한다.

## 7. SavePoint 배치 정책

- SavePoint는 짝수 Floor `2, 4, 6, ...`마다 정확히 하나 배치한다.
- 선택된 시설 맵 템플릿은 SavePoint를 배정할 수 있는 하나 이상의 authored room slot을 명시한다.
- Core는 짝수 Floor에서 그 후보 중 하나를 Floor seed로 결정론적으로 선택해 SavePoint를 배정한다.
- 홀수 Floor에서는 같은 slot의 다른 허용 category를 사용할 수 있어야 한다.
- 플레이어가 SavePoint를 선택하는 것은 강제하지 않는다. 다른 분기나 Gate를 선택해 해당 Floor의
  SavePoint에 영구적으로 접근할 수 없게 되는 것도 허용하며, SavePoint는 배치만 보장하고 이용은
  보장하지 않는다.
- Core는 미래 SavePoint 접근 상실을 위한 별도 경고·확인·추천 projection을 만들지 않는다.
- 같은 Floor에 무작위 추가 SavePoint를 생성하지 않는다.
- 현재 3-Floor Standard run에서는 Floor 2에만 SavePoint가 존재하고 Floor 3에는 존재하지 않는다.

## 8. 레거시 전환

기존 depth/lane 기반 맵 shape, node 수와 edge를 생성하는 절차적 graph generator를 제거한다.
기존 Event 강제 삽입, 행별 Combat 보장·상한, Support/SavePoint 보장, preterminal 교체와 기타 사후
repair 규칙도 mode·Floor assignment profile과 slot 기반 결정론적 constrained matching으로
교체한다. 구 graph와 새 template topology를 함께 authority로 유지하거나, template 배정 실패를
구 generator fallback으로 복구하지 않는다.

## 9. 구현 전 동기화 대상

독립 goal에서 최소 다음 표면을 함께 감사하고 갱신한다.

- `archive/game_rulebook_legacy_2026-10-04.md`: Floor 맵, node 선택, edge direction과 SavePoint 배치 의미
- `core_runtime_contract.md`: template source-of-truth, navigation projection, checkpoint 경계
- `data_loading_contract.md`: template id, room slot, allowed category, connection과 validation
- live map RON: `facility_templates.ron`의 slot tag, `node_assignment_profiles.ron`, template
  pool/schedule과 기존 `generation_policy.ron` 제거
- Core map generator/session/state/snapshot/admin command와 save/checkpoint schema
- BossOmen 일반 map -> capstone map 전환, Boss/Gate visibility와 checkpoint rollback
- 외부 Unity `unity_core_contract.md`: template identity와 marker projection 계약
- 외부 Unity `docs/ui/screen_spec.md`: 정적 시설 도면, room marker와 ForwardOnly 표현
- Unity-local template resource, slot anchor와 Core/Unity cross-project validation
- game server mapping, Core test, strict live loading과 Unity EditMode/PlayMode
- 조건부 발현 rule, precommitted manifest content, save/checkpoint와 reveal animation
- 후속 독립 정책:
  [combat_power_floor_difficulty_policy_draft_2026-07-26.md](combat_power_floor_difficulty_policy_draft_2026-07-26.md)
- 후속 독립 정책:
  [elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md](elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md)

## 10. 비목표

- SavePoint의 Trauma 회복량, checkpoint overwrite와 Run Failed 정책
- 개별 시설 맵의 최종 아트, 연출과 사운드
- Unity가 runtime edge overlay나 동적 travel path를 그리는 기능
- 좌표나 이미지 픽셀에서 connection을 추론하는 기능
- template assignment failure를 위한 구 절차적 graph fallback
- 열쇠·자원·Event 결과 조건, `AnyOf`, 연쇄 발현, 재잠금, 동적으로 열리고 닫히거나 방향이 바뀌는
  room/door/connection과 mandatory progression node의 조건부 발현
- 플레이어 전투력 산식, Floor power anchor, Combat node difficulty band와 encounter/wave scaling
- 모노리스의 일반 Combat content 변환, 강화 재현 session, 반복 보상과 상점/도감 상태. 이 범위는
  [elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md](elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md)로
  분리한다.

## 11. 승인 후 완료 조건

1. 맵 전환 전용 영향 감사와 독립 구현 goal을 작성한다.
2. durable canonical 문서와 외부 Unity 계약을 같은 authority 구조로 갱신한다.
3. template schema, template pool, assignment profile loader와 cross-project validation을 구현한다.
4. Core constrained node assignment, navigation, BossOmen capstone 전환과 checkpoint 경계를 전환한다.
5. Unity-local 일반/capstone template, reveal layer와 marker presentation을 전환한다.
6. focused Core test, strict live loading, server mapping과 Unity test를 검증한다.
7. 두 보조 제어실의 모든 성공 순서, Retreat/실패, checkpoint load와 reconnect에 대한 조건부 발현
   contract probe를 검증한다.
8. 절차적 depth/lane graph, Support repair와 중복 connection authority가 남지 않았는지 감사한다.
9. 완료 review 뒤 이 문서를 historical decision input으로 전환한다.

## 12. 다른 작업자에게 넘길 때의 기준

이 문서의 2~8절은 합의된 target policy다. 현재 runtime, live RON과 기존 canonical이 다르게
동작하는 부분은 이 결정을 무효화하는 근거가 아니라 구현 시 함께 전환해야 할 current-state 차이다.

- authored template가 room 수·slot·허용 category·고정 connection topology를 소유한다.
- 절차적 depth/lane graph 생성이나 runtime node별 connection 복사본을 새 target에 유지하지 않는다.
- Unity는 template resource의 방·복도·경로와 slot anchor를 표시하고 Core가 투영한 marker 상태만
  합성한다. Unity가 이미지·좌표로 연결이나 선택 가능성을 재계산하지 않는다.
- node 배정은 모든 slot을 채우는 결정론적 constrained matching이며 실패 시 fallback을 만들지 않는다.
- Standard는 Floor별 exact profile mapping을, Endless는 Floor 구간과 명시적 반복 cycle을 사용한다.
- template 선택은 mode별 결정론적 소진형 pool을 사용하고 소진 뒤 새 순환으로 초기화한다.
- Slot tag는 category/topology와 구분되는 authored semantic role이며, 잔여 모노리스 조건부 상인
  보장은 다음 Floor의 `Shop` tag slot assignment로 해결하고 현재 map을 동적으로 수정하지 않는다.
- Standard에는 떠돌이 상인이 없고 Endless의 떠돌이 상인은 Floor 4부터만 등장한다.
- Standard는 `standard_intro`, `standard_branching`, `standard_final` 순으로 더 크고 복잡한 template
  pool을 사용하며 Floor 1·2의 필수 Elite guardian과 Floor 3 Final Boss로 `Elite -> Elite -> Boss`를
  학습시킨다.
- runtime과 template schema는 typed terminal을 `1..N`개 지원하지만 현재 Standard 콘텐츠는
  Floor마다 하나만 저작한다.
- SavePoint 배치와 선택 가능성은 구분하고 접근 상실을 위한 경고·추천 계약을 만들지 않는다.
- BossOmen chain 완료는 같은 Floor의 Boss/Exit Gate 전용 capstone template로 map을 교체한다. Boss만
  먼저 표시·선택 가능하고 Exit Gate와 경로는 Boss 승리 뒤 공개한다.
- 조건부 발현은 Floor 시작 시 precommit된 선택형 보너스 node만 대상으로 하며, 모든 선행 node 성공을
  같은 Core transaction 경계에서 판정한다. 발현 전 marker/connection은 숨기고 Unity-local 환경
  단서만 선택적으로 허용한다.

최종 시설 아트·연출·사운드, 실제 room 수·topology와 assignment weight는 후속 콘텐츠 authoring
범위다. 정책 검토 결과 이 문서가 소유하는 facility authority, visibility, navigation, 조건부 보너스
node 발현과 capstone 전환에는 추가 사용자 결정이 없다. 통상적인 schema·validation 세부사항은 위
authority와 fail-fast 원칙 안에서 구현 goal이 결정하고 문서에 기록하며, 문서 의미를 바꿔야만
해결되는 충돌이 확인될 때만 사용자 결정을 다시 요청한다.
