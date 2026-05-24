# 본사 연락 노드 Goal 계획

이 문서는 런 중 직원 충원과 기초 보급을 담당하는 `본사 연락 노드` 구현을 위한 goal 문서다.

목표는 기존 `Support` 노드에 채용을 억지로 끼워 넣는 것이 아니라, 별도 안전 노드 카테고리로 본사 연락 흐름을 추가하는 것이다. 본사 연락 노드는 채용만 하는 꽝 노드가 아니어야 하며, 직원 충원이 필요 없는 상황에서도 기초 보급 선택지로 최소 가치를 가져야 한다.

## Goal 원칙

- 본사 연락 노드는 `Medical`, `Rest`, `Maintenance`와 역할이 다르므로 별도 노드 카테고리로 둔다.
- 한 번 방문하면 하나의 행동만 선택할 수 있다.
- 채용, 긴급 구호품, 본사 보급 구매는 서로 대체 선택지다.
- 연구 완료 파편 수령은 안전 노드 도착 시 자동 수령 정책을 유지한다. 본사 연락 노드의 행동권을 소모하지 않는다.
- 일반 상점 노드는 유지한다. 일반 상점은 무너진 회사 폐허 속 수수께끼의 상인 컨셉이며, 낮은 등장 확률과 고가치/고밸류 상품을 가진다.
- 본사 보급 구매는 일반 상점과 역할이 겹치면 안 된다. 기본 구급품, 저등급 재료, 안정적인 저밸류 보급만 다룬다.
- 시작 직원 후보와 런 중 채용 후보는 별도 데이터 파일로 분리한다.
- 임시방편으로 `SupportNodeType`에 모든 기능을 밀어 넣지 않는다.
- 클라이언트 UI는 아직 고정 계약이 아니므로 core는 선택지, 후보, 상품, 결과를 데이터로 명확히 제공한다.
- 정책이 모호하거나 게임 감각을 바꿀 수 있는 결정은 구현 전에 사용자와 의논한다.

## 최종 목표

런 중 안전 노드로 `HeadquartersContact`를 추가한다.

완료된 흐름은 아래를 만족해야 한다.

```text
맵에서 본사 연락 노드 선택
-> NodePreview에서 본사 연락 가능 행동 표시
-> ConfirmEnterNode
-> 본사 연락 세션 진입
-> 아래 행동 중 하나 선택
   1. 직원 충원
   2. 긴급 구호품 요청
   3. 본사 보급 구매
-> 선택한 행동 해결
-> 노드 완료
-> 다음 노드로 진행
```

## 범위

포함한다.

- 새 map node category 또는 명확한 별도 node payload.
- RON node definition에 본사 연락 노드 추가.
- 별도 `recruitment_candidates.ron` 데이터 계약.
- 본사 연락 세션 상태와 행동 결과.
- 직원 충원 후보 제시 및 1명 채용.
- 긴급 구호품 요청의 최소 보상.
- 본사 보급 상점의 최소 상품 풀.
- 한 노드에서 하나의 본사 행동만 선택 가능하도록 action gate.
- 연구 완료 파편 자동 수령 정책과 충돌하지 않는지 검증.
- 일반 상점과 본사 보급 상점의 역할 분리 테스트.

포함하지 않는다.

- 고급 채용 비용/계약 협상.
- 직원 성격별 채용 이벤트.
- 본사 신뢰도, 본사 평판, 장기 파견 정책.
- 희귀 상인 고밸류 상품 재설계.
- Unity UI 구현.
- 세부 밸런스 수치 확정.

## 정책 확정 사항

- `본사 연락 노드`는 기존 `Support`가 아니라 별도 안전 노드로 구현한다.
- 한 번 방문 시 행동 하나만 선택한다.
- 선택지는 1차 구현 기준으로 `RecruitEmployee`, `RequestEmergencySupplies`, `OpenHeadquartersShop` 세 가지다.
- 연구 완료 파편은 본사 연락 선택지를 소모하지 않고, 기존 안전 노드 자동 전달 정책을 유지한다.
- 채용 후보 데이터는 `starter_candidates.ron`을 재사용하지 않고 별도 `recruitment_candidates.ron`으로 둔다.
- 일반 상점은 `수수께끼의 상인` 컨셉으로 남긴다. 낮은 등장 확률, 고가치 상품, 비정상적 거래자 감각을 담당한다.
- 본사 보급 상점은 기본 구급품과 저등급 재료만 다룬다. 안정적이지만 고밸류가 아니어야 한다.

## G1. 노드 카테고리와 데이터 계약

### 목표

맵 노드 정의에서 본사 연락 노드를 명시적으로 표현한다.

### 설계 방향

후보:

```rust
MapNodeCategory::Headquarters
MapNodePayload::HeadquartersContact { ... }
```

또는 동등하게 `Support`와 분리된 명확한 payload를 둔다.

### 종료 조건

- `game_resources/data/map/node_definitions.ron`에 본사 연락 노드가 추가된다.
- 기존 `SupportNodeType`은 `Medical`, `Rest`, `Maintenance` 중심으로 유지된다.
- map generation이 본사 연락 노드를 안전 노드로 생성할 수 있다.
- 기존 shop/reward/support/event 노드가 본사 연락 노드로 오인되지 않는다.

### 빠른 검증

```bash
cargo test -p game_core map::generator
cargo test -p game_core ron_loading
```

## G2. 본사 연락 세션과 선택지 계약

### 목표

본사 연락 노드에 진입하면 선택 가능한 행동을 core snapshot/result로 노출한다.

### 필요한 선택지

```text
HeadquartersContactOption::RecruitEmployee
HeadquartersContactOption::RequestEmergencySupplies
HeadquartersContactOption::OpenHeadquartersShop
```

### 종료 조건

- 본사 연락 노드 진입 시 별도 session state가 생긴다.
- 선택 가능한 행동 목록이 `BehaviorResult` 또는 snapshot에 노출된다.
- 선택지 하나를 해결하면 노드가 완료된다.
- 같은 노드에서 두 번째 본사 행동을 실행할 수 없다.
- `CancelSelectedNode` 정책이 기존 노드 선택/진입 흐름과 충돌하지 않는다.

### 빠른 검증

```bash
cargo test -p game_core headquarters
cargo test -p game_core game::world::tests::node_sessions
```

## G3. 런 중 직원 채용

### 목표

본사 연락 노드에서 채용 후보 중 1명을 선택해 로스터에 추가한다.

### 데이터

새 파일:

```text
game_resources/data/employees/recruitment_candidates.ron
```

후보 데이터는 시작 후보와 유사할 수 있지만 별도 파일이어야 한다.

### 종료 조건

- 채용 후보 데이터가 RON에서 로드된다.
- 본사 연락 노드가 채용 후보 2~3명을 제시한다.
- 플레이어는 후보 중 1명을 선택해 채용할 수 있다.
- 채용된 직원은 로스터/벤치에 추가된다.
- 채용 후 해당 본사 연락 노드는 완료된다.
- 시작 직원 후보와 런 중 채용 후보의 source of truth가 분리된다.

### 빠른 검증

```bash
cargo test -p game_core employee_data
cargo test -p game_core headquarters_recruit
```

## G4. 긴급 구호품 요청

### 목표

채용이 필요 없는 상황에서도 본사 연락 노드가 최소 가치를 갖도록 기초 보급 선택지를 제공한다.

### 정책

- 고밸류 보상은 주지 않는다.
- 기본 구급품, 소량 크레딧, 저등급 정비 재료 같은 안정적 보급만 준다.
- 세부 수치는 밸런스 단계에서 조정 가능해야 한다.

### 종료 조건

- `RequestEmergencySupplies` 선택 시 정해진 보급 결과가 지급된다.
- 보급 결과는 `NodeOutcomeSummary` 또는 동등한 결과로 클라이언트가 표시할 수 있다.
- 선택 후 노드는 완료된다.
- 보급은 일반 상점/보상 노드의 고밸류 역할을 침범하지 않는다.

### 빠른 검증

```bash
cargo test -p game_core headquarters_supplies
```

## G5. 본사 보급 상점

### 목표

본사 연락 노드에서 저밸류 기본 보급품을 구매할 수 있는 별도 상점 흐름을 제공한다.

### 정책

- 일반 상점과 같은 pool을 쓰지 않는다.
- 본사 보급 상점은 낮은 등급의 구급품/재료 위주다.
- 일반 상점은 수수께끼의 상인 컨셉, 낮은 등장 확률, 고가치 상품으로 남긴다.

### 종료 조건

- 본사 보급 상점 전용 상품 pool이 존재한다.
- `OpenHeadquartersShop` 선택 시 본사 보급 상점 session에 들어간다.
- 구매/나가기 흐름이 기존 shop과 재사용 가능하면 재사용하되, 일반 상점 pool과 source of truth는 분리한다.
- 본사 보급 상점에서 나가면 본사 연락 노드는 완료된다.
- 한 본사 연락 노드에서 채용/구호품/상점 중복 선택은 불가하다.

### 빠른 검증

```bash
cargo test -p game_core headquarters_shop
cargo test -p game_core shop_map_node
```

## G6. 안전 노드 자동 연구 수령과의 연결

### 목표

본사 연락 노드가 연구 완료 파편 자동 수령 정책과 충돌하지 않도록 한다.

### 정책

- 연구 완료 파편은 전투 노드가 아닌 다음 안전 노드에서 자동 수령된다.
- 본사 연락 노드도 안전 노드이므로 자동 수령 대상이 될 수 있다.
- 자동 수령은 본사 연락 행동 선택권을 소모하지 않는다.

### 종료 조건

- 본사 연락 노드 진입 시 pending research delivery가 있으면 결과에 포함된다.
- 이후에도 본사 연락 선택지 하나를 정상적으로 선택할 수 있다.
- 자동 수령 때문에 본사 연락 노드가 즉시 완료되지 않는다.

### 빠른 검증

```bash
cargo test -p game_core safe_node_entry_delivers_pending_research_fragments
cargo test -p game_core headquarters
```

## 완료 기준

이 goal은 아래 조건을 모두 만족할 때 완료로 본다.

- 본사 연락 노드가 `Support`와 별도 카테고리/계약으로 존재한다.
- 본사 연락 노드가 RON map node definition에서 생성될 수 있다.
- 본사 연락 세션이 채용/긴급 구호품/본사 보급 상점 선택지를 노출한다.
- 한 본사 연락 노드에서 하나의 행동만 선택 가능하다.
- 런 중 채용 후보 데이터가 `recruitment_candidates.ron`에서 로드된다.
- 채용 선택 시 직원 1명이 로스터/벤치에 추가된다.
- 긴급 구호품 선택 시 기초 보급이 지급되고 노드가 완료된다.
- 본사 보급 상점은 일반 상점과 다른 상품 pool을 사용한다.
- 일반 상점 노드는 수수께끼의 상인/고밸류 희귀 노드로 유지된다.
- 연구 완료 파편 자동 수령은 본사 연락 행동권을 소모하지 않는다.
- 관련 world flow 테스트와 RON loading 테스트가 통과한다.

## 빠른 피드백 루프

```bash
cargo test -p game_core ron_loading
cargo test -p game_core map::generator
cargo test -p game_core headquarters
cargo test -p game_core shop_map_node
```

최종 검증:

```bash
cargo test -p game_core
cargo check -p game_server
```

## 실험 기록

장시간 goal mode로 진행할 경우 아래 형식을 유지한다.

### Experiment 1. 본사 연락 노드 공식 흐름 추가

- 가설: `Support`에 채용을 추가하지 않고도 별도 `HeadquartersContact` 카테고리와 session state를 추가하면, 채용/긴급보급/본사상점이 기존 지원 노드와 충돌하지 않고 안전 노드 흐름에 붙을 수 있다.
- 변경: `MapNodeCategory::HeadquartersContact`, `MapNodePayload::HeadquartersContact`, `HeadquartersContactSessionState`, `BehaviorResult::HeadquartersContactState`, `PlayerBehavior::{RecruitEmployee, RequestEmergencySupplies, OpenHeadquartersShop}`를 추가했다.
- 변경: `game_resources/data/employees/recruitment_candidates.ron`을 새 source of truth로 추가했고, 시작 후보 `starter_candidates.ron`과 분리했다.
- 변경: 본사 보급 상점 전용 pool `headquarters_basic_supplies`와 shop `headquarters_basic_supply`를 추가했다. 기존 `shop_general`은 `mysterious_merchant`/`rare` 태그와 낮은 weight를 가진 일반 희귀 상점으로 유지했다.
- 변경: 본사 연락 노드를 연구 완료 파편 자동 수령 가능한 안전 노드에 포함했다. 수령은 진입 결과에 포함되며 본사 행동권을 소모하지 않는다.
- 변경: `game_server` WebSocket command result 직렬화에 `HeadquartersContactState`, `EmployeeRecruited`, `EmergencySuppliesGranted`를 추가했다.
- 검증 명령: `cargo check -p game_core`
- 결과: 통과.
- 검증 명령: `cargo test -p game_core headquarters`
- 결과: 4개 HQ world flow 테스트 통과.
- 검증 명령: `cargo test -p game_core --test ron_loading`
- 결과: 12개 RON loading 테스트 통과. 별도 채용 후보와 HQ shop pool 해석 검증 포함.
- 검증 명령: `cargo test -p game_core map::generator`
- 결과: 5개 map generator 테스트 통과. built-in node definition 계약에 HQ 노드 포함.
- 검증 명령: `cargo check -p game_server`
- 결과: 통과. 기존 warning만 존재.
- 검증 명령: `cargo test -p game_core`
- 결과: 전체 game_core 테스트 통과.
- 다음 결정: 긴급 구호품의 현재 기본값은 소량 엔케팔린 지급이다. 구급품/저등급 재료 지급까지 확장할지는 실제 클라이언트 플레이 흐름에서 노드 밸류를 보고 조정한다.
