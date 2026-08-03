# Facility Map, Assignment, And Manifestation Experiments

Status: complete

Implementation complete: true

Created: 2026-07-27

## 2026-07-27 — Child Goal 초기화와 readiness audit

### Dependency와 baseline

- P01과 선호 직렬 predecessor P02 completion review는 모두 `Complete / High`다.
- P02 최종 Rust broad: Core 765, integrations 3/4/23/13/4, server 51, 실패 0.
- P02 최종 Unity broad: EditMode 251 pass, PlayMode 62 pass/4 explicit live opt-in skip, 실패 0.
- 재기동된 Unity MCP `ark@b49d32f6f2f70bbe`는 Editor ready, compiling false, Console error 0이다.

### Current-state evidence

- `MapGenerator`는 `depth_count`, lane width와 category weight/row repair로 topology와 content를 함께
  만든다.
- `RunMap`은 `map_template_id`와 `slot_id` foundation이 있지만 slot은 depth/lane 문자열이어야 하고
  runtime edge list가 독립 graph authority로 저장된다.
- live map data는 `generation_policy.ron`, `node_definitions.ron`뿐이며 facility template/pool/assignment
  profile RON이 없다.
- BossOmen completion은 일반 map에 forced boss와 exit Gate node/edge를 동적으로 추가한다.
- Unity `NodeMapTemplateCatalog`는 `act_01_floor_a`와 계산형 9×5 depth/lane anchor만 제공한다.

### Readiness verdict

- actual templates와 Unity anchors는 준비된 입력이 아니라 P03이 만들어야 하는 production content다.
- 승인 decision은 초기 room 수/topology/weight authoring을 이 phase에서 허용하고 별도 정책 질문이
  없다고 명시한다.
- readiness: ready. Placeholder/fallback 없이 actual RON/resource와 cross-project validation을 함께
  구현한다.

### 다음 작업

- current map types/progression/checkpoint의 topology/state ownership 상세 감사
- live loader publication/validation extension 지점과 node-definition category 후보 확인
- 최소 actual template/profile 세트 및 Unity anchor/reveal resource 설계

## 2026-07-27 — Authored schema, live data와 production selection 기반

### 구현

- `FacilityTemplateDatabase`에 semantic slot, allowed category, tag, typed terminal, directed connection,
  template kind/pool과 v1 manifestation rule을 추가했다.
- `NodeAssignmentProfileDatabase`에 category min/max/exact, remaining weight, tagged fixed assignment,
  Standard exact schedule와 Endless contiguous/open-ended cycle을 추가했다.
- duplicate/unknown tag, invalid role/category, missing connection, directed terminal dead-end, mandatory
  manifestation target, schedule gap/overlap/reference를 strict validation한다.
- live `facility_templates.ron`에 Standard intro/branching/final, Endless two exploration + dual-control
  puzzle, BossOmen capstone actual templates를 저작했다.
- live `node_assignment_profiles.ron`에 Standard Floor 1~3와 Endless odd/even early/open profiles 및
  pool schedule을 저작했다. P04 `difficulty_profile_id`는 선행 추가하지 않았다.

### Generator 전환

- deterministic pool shuffle/cycle/no-repeat state와 backtracking constrained assignment를 구현했다.
- tagged `terminal_guard -> combat_elite`, fixed Start/typed terminal, node-definition weighted selection과
  semantic slot node ID가 Floor 생성 시 한 번 확정된다.
- `FacilityRunState`를 RunState/checkpoint/active-run에 포함하고 다음 Floor 선택에서 보존한다.
- production new-run/Floor transition을 authored facility generator로 전환했다.
- 구 `MapGenerator`, depth/lane topology generation, category row repair와
  `data/map/generation_policy.ron`을 제거했다.

### Focused 결과

- live source set strict parse: pass
- facility schema tests: 3 pass
- facility generator determinism/pool/schedule tests: 3 pass
- procedural generator 제거 뒤 `cargo test -p game_core --lib --quiet`: 759 passed, 0 failed
- `cargo check -p game_server`: pass

### 아직 완료하지 않은 경계

- `RunMap.edges`를 authored slot connection lookup 하나의 authority로 교체하고 display depth/lane 제거
- manifestation checkpoint rollback/reconnect와 성공/retreat/failure/cancel production flow 회귀 확장
- Unity actual template anchor/reveal/ForwardOnly resource 및 cross-project validator
- canonical/legacy search와 broad completion review

## 2026-07-27 — Typed terminal, authored capstone과 manifestation runtime

### Typed terminal

- 단일 `terminal_node_id`를 `MapTerminalNode { node_id, resolution }` 1..N 계약으로 교체했다.
- terminal은 room category의 암시가 아니라 `FloorGate`, `RunFinalBoss`, `BossOmenExitGate`,
  `SpecialRunCompletion` 역할이다. 하나를 완료하면 같은 map의 다른 terminal은 잠긴다.
- RunMap은 facility template version을 저장하고 terminal ID 중복/누락을 strict 검증한다.

### Generic capstone

- `FacilityCapstoneRequest`/`FacilityCapstoneEntryInput` typed seam을 추가했다. Capstone kind와 Entry input
  cardinality/slot identity를 검증하고 template의 모든 slot/connection/terminal을 한 번에 생성한다.
- BossOmen 완료 시 일반 map에 node/edge를 추가하지 않고 `boss_omen_capstone`으로 map/progression을
  원자 교체한다.
- 승리 뒤 새 Gate를 만들지 않고 미리 배정·conceal된 `exit_gate` terminal만 공개한다.

### Manifestation

- live template의 prerequisite slot들이 모두 Completed인 성공 commit에서만 target slot을 공개한다.
- target의 category/kind/payload/node ID는 Floor 생성 때 이미 배정되어 공개 시 reroll하지 않는다.
- 공개는 monotonic/idempotent하며 진행도의 available node 집합을 authored connection 위에서 다시
  계산한다. Concealed 상태와 prerequisite Completed 상태가 checkpoint/active-run current map에 그대로
  직렬화된다.

### 검증

- BossOmen focused: 12 passed, 0 failed
- manifestation focused: 1 passed, 0 failed
- map flow focused: 38 passed, 0 failed
- Core broad: 762 passed, 0 failed
- `cargo check -p game_server`: pass

## 2026-07-27 — Topology authority convergence와 Unity exact resource

### Core convergence

- `RunMap.edges`와 node `depth`/`lane`를 제거했다. Navigation, node completion, recovery-path evaluator와
  manifestation은 current live `FacilityTemplate.connections`를 ID/version/slot으로 resolve한다.
- `MapNodeDefinition.min_depth/max_depth`, grid slot factory, default template serde fallback과
  `generation_policy.ron`을 제거했다. Structural Start는 content pool 밖에서 template Start slot로
  합성한다.
- Checkpoint/active-run import는 template ID/version, exact slot coverage, slot allowed category,
  derived start IDs와 typed terminal set을 current live catalog와 strict 대조한다.
- Public `MapViewDto`에도 `facility_template_version`을 투영해 Unity가 local resource version drift를
  fail-closed 할 수 있게 했다.

### Unity exact presentation resource

- `NodeMapTemplateCatalog`에 Core v1 template 7종의 exact semantic slot/anchor뿐 아니라 authored
  connection direction과 `reveal_with_slot_id`를 모두 저작했다.
- Presenter는 Core visibility로 reveal-gated corridor를 숨김/공개하고, UGUI는 static corridor와
  `ForwardOnly` 화살표를 marker 뒤에 렌더한다. Local connection은 presentation 전용이며 selectable은
  계속 `map_navigation.selectable_node_ids`만 소비한다.
- 알 수 없는 template/slot, 누락/불일치 version은 procedural coordinate/corridor fallback 없이 진단한다.

### 최종 검증

- Core broad: 761 unit + integration 3/4/23/13/4, 총 808 pass, 실패 0
- `cargo check -p game_server --quiet`: pass
- Unity script validation 7 files: error 0; Editor compile/Console error 0
- Unity focused: EditMode 6/6, PlayMode 1/1 pass
- Unity broad: EditMode 254/254, non-live PlayMode 62/62 pass
- 별도 `[Explicit][Category("LiveServer")]` PlayMode 4개는 외부 game_server가 없는 broad assembly 실행에서
  연결 실패했으며, non-live group을 분리 실행해 62/62를 확정했다.
- Core/Unity legacy search와 scoped `git diff --check`: 0 finding
