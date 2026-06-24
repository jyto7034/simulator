# Policy Decisions

이 문서는 `core_component_refactor_master` 감사 중 `사용자와 정책 논의 필요`로 남긴 항목에 대해 사용자와 순차 논의한 결정을 기록한다.

기준 문서: `docs/refactor_preparation_plan.md`

## world_run_progression

### RUN_SYSTEM_POLICY data source

결정: `RUN_SYSTEM_POLICY`의 setup, support, post-battle, headquarters, live deployment 수치는 장기적으로 live RON/data-driven policy로 이동한다.

근거:
- 사용자가 2번, live RON/data-driven policy 이동이 더 낫다고 결정했다.
- 후속 구현 시 RON schema, validation, Unity 표시 계약을 함께 설계해야 한다.

운영 방향:
- 구현 schema는 `game_resources/data/run/policy.ron` 단일 통합 파일로 둔다.
- 이 파일은 setup, support, post-battle, headquarters, live deployment policy를 한 객체로 관리한다.
- 통합 파일은 기존 `RUN_SYSTEM_POLICY` bundle과 1:1로 대응하므로 우선 source of truth를 코드 상수에서 live data로 옮기는 데 적합하다.

### battle_records export

결정: `battle_records/run_<seed>/*.json` export는 게임 개발 중 필요한 디버깅 아티팩트로 유지한다.

운영 방향:
- 목적은 디버깅 전용이다.
- 개발 중에는 매번 환경 변수를 설정하지 않도록 기본적으로 켜둔다.
- 공식 gameplay source of truth나 replay/golden contract로 승격하지 않는다.

## map_node

### map authoring/generation policy source

결정: map authoring/generation policy는 장기적으로 live RON/data-driven source로 완전 이동한다.

범위:
- node definition
- category distribution
- row repair rule
- safe-node preference
- elite/tag semantics
- depth별 등장 정책
- act/boss/route 구조 정책

비범위:
- 현재 선택된 노드, 완료/사용 가능 노드 목록, active node session, node 진행 중 상태, seed로 resolve된 generated map instance 같은 runtime state는 코드/runtime state로 유지한다.

운영 방향:
- 장기 방향은 data-driven 완전 이동이다.
- 실제 구현은 RON schema, validation, Unity-facing 표시/계약을 확인하면서 단계적으로 진행한다.

### NodeSessionKind

결정: `NodeSessionKind`는 장기적으로 제거하고 `MapNodeCategory`로 대체한다.

근거:
- 현재 `NodeSessionKind`는 `MapNodeCategory`와 1:1에 가까운 파생 projection이다.
- 사용자 확인에 따라 Unity-facing DTO는 장기 리팩토링에서 수정 가능하다.

운영 방향:
- 내부 source of truth는 `MapNodeCategory`로 모은다.
- 실제 제거는 Unity 소비 코드와 snapshot/result 계약을 함께 갱신하는 작업으로 처리한다.

### MapViewDto progression projection

결정: 장기적으로 `MapViewDto`는 `per-node state`를 canonical projection으로 두고 `available_node_ids`/`completed_node_ids` 같은 id list 중복 projection을 제거한다.

근거:
- Unity map rendering에 필요한 단위는 최종적으로 각 node의 표시 상태다.
- `node.state == Available`과 `available_node_ids.contains(node.id)`처럼 같은 의미가 두 곳에 있으면 drift 위험이 있다.
- 내부 runtime progression 계산 구조와 Unity 표시용 projection을 분리하되, Unity DTO 안에서는 `per-node state`를 신뢰하게 하는 방향이 더 단순하다.

운영 방향:
- 실제 변경은 Unity-facing DTO shape 변경으로 다룬다.
- 사용자가 1번을 확정했다: `MapViewDto.available_node_ids` / `completed_node_ids`를 제거하고 `MapNodeDto.state`를 canonical source로 사용한다.
- `MapProgression` 내부 계산 구조는 별도 리팩토링 전까지 유지 가능하다.

### map node tags

결정: map node definition의 범용 `tags`는 제거한다.

근거:
- `category`, `payload`, `weight`, `min_depth`, `max_depth`, `kind_id`가 이미 대부분의 분류/authoring 의미를 표현한다.
- 현재 `elite`처럼 gameplay에 영향을 주는 의미도 `tags`가 아니라 `kind_id.contains("elite")`로 판정되고 있어 source-of-truth가 흐리다.
- `support`, `medical`, `choice`, `full`, `shop`, `reward`, `boss` 같은 tag는 이미 `category`/`payload`/`SupportNodeMode`로 표현 가능하다.

운영 방향:
- 범용 문자열 tag를 공식 gameplay metadata로 승격하지 않는다.
- 실제 로직에 필요한 의미는 `combat_tier`, `node_difficulty_band`, `encounter_profile` 같은 명시 필드로 승격한다.
- flavor/authoring note가 필요하다면 gameplay schema와 분리된 문서/주석성 metadata로 다룬다.

## behavior_state_gate

### allowed_actions source

결정: `allowed_actions`는 Unity-facing UI affordance용 canonical projection으로 유지한다.

근거:
- Unity가 가능한 행동을 미리 알고 버튼 disabled 상태를 표현할 수 있다.
- 제거하면 사용자가 버튼을 눌러야만 reject 응답으로 불가능 여부를 알 수 있어 UX가 나빠진다.

운영 방향:
- `allowed_actions`는 UI 표시/상호작용 affordance source로 본다.
- 보안/최종 검증 source로 보지 않는다.
- command handler validation은 최종 방어선으로 계속 유지한다.

### action validation gates

결정: action validation은 1차 phase/action-kind gate와 2차 handler-specific validation으로 분리한다.

역할:
- 1차 게이트: 현재 `GameState`/phase에서 해당 `ActionKind` 자체가 가능한지 검증한다.
- 2차 게이트: handler 내부에서 payload, target, resource, 세부 state가 유효한지 검증한다.

근거:
- `allowed_actions`는 Unity 버튼 disabled 같은 UI affordance를 표현하기 좋다.
- handler validation은 잘못된 payload/target에 대해 더 구체적인 에러를 낼 수 있다.
- 모든 검증을 pre-dispatch로 올리면 에러 의미가 `InvalidAction`으로 뭉개질 수 있고, 모든 검증을 handler로 내리면 Unity가 불가능한 버튼을 미리 표현하기 어렵다.

운영 방향:
- `allowed_actions`는 1차 게이트와 Unity UI affordance의 source다.
- handler validation은 최종 방어선이다.
- 리팩토링 시 error timing/error code가 바뀌는 변경은 계약 변경으로 다룬다.

### RequestDeploymentRangePreview gate

결정: `RequestDeploymentRangePreview`는 gameplay `allowed_actions` gate 안에 유지한다.

근거:
- 배치 범위 preview도 현재 battle state, deployment state, unit 정보에 의존한다.
- 아무 state에서나 가능한 read/query로 두면 요청 가능 경계가 흐려진다.
- gate 안에 두면 Unity가 preview 버튼/hover 가능 여부를 명확히 알 수 있다.

운영 방향:
- 필요하면 장기적으로 action/query naming은 정리할 수 있지만, availability gate 자체는 유지한다.

## movement_blocking

### forced movement clears engagement

결정: Forced movement / knockback / teleport는 기존 block engagement를 즉시 해제하고, 다음 movement/AI tick에서 새 위치 기준으로 movement intent와 engagement를 재평가한다.

근거:
- 여기서 block engagement는 저지 관계, 즉 다음 이동/행동 의사결정에 영향을 주는 engagement state를 뜻한다.
- 강제 이동이나 teleport로 위치가 바뀌면 이전 위치 기준의 저지 관계를 유지하는 것은 source-of-truth와 맞지 않는다.
- 새 위치에서 이동 계속, 새 저지/교전 발생, 타겟 재선정, 공격/스킬 가능 여부를 다시 판단하는 편이 단순하고 정확하다.

운영 방향:
- forced movement/knockback/teleport 적용 시 기존 block engagement를 끊는다.
- 즉시 예전 engagement를 유지하거나 이전 blocker를 재사용하지 않는다.
- 다음 movement/AI tick에서 현재 위치와 현재 target/path 기준으로 movement intent와 engagement를 다시 계산한다.
- 필요하면 engagement 해제/재평가를 Unity-facing event로 노출할지 별도 DTO 정책에서 다룬다.

### MovementStopped event

결정: Active movement segment가 의미 있게 종료되는 순간은 Unity-facing `MovementStopped` event로 보낸다.

근거:
- 이동 종료는 target 도착, 장애물/저지, 공격/스킬/CC/사망 interrupt, goal 상실 등 서로 다른 gameplay 의미를 가질 수 있다.
- Unity 연출, debug, replay가 이동이 왜 멈췄는지 알 수 있어야 한다.
- 단순히 위치가 더 이상 변하지 않는 상태만으로는 종료 사유를 구분하기 어렵다.

운영 방향:
- `MovementStopped` event에는 `reason`을 포함한다.
- reason 후보: `TargetReached`, `BlockedByObstacle`, `InterruptedByAttack`, `InterruptedBySkill`, `InterruptedByCrowdControl`, `InterruptedByDeath`, `GoalLost`.
- 애초에 active movement가 없던 no-goal tick은 `MovementStopped` event로 만들지 않는다.
- 이전 active segment가 있었는데 goal이 사라진 경우에만 `GoalLost` reason으로 보낸다.

### movement tick policy source

결정: `DEFAULT_MOVEMENT_TICK_MS` 50ms cadence는 장기적으로 RON/data-driven battle movement policy로 이동한다.

근거:
- movement tick cadence는 전투 연출, collision/engagement 반응성, performance에 영향을 주는 runtime policy다.
- 코드 상수로 남기면 battle runtime numeric policy와 같은 source-of-truth 문제가 생긴다.

운영 방향:
- battle movement policy RON/schema에 tick cadence를 명시한다.
- engine invariant나 안전장치 성격의 최소/최대 guard는 코드에 남길 수 있다.

### opponent spawn occupancy policy

결정: Opponent spawn nearest-open fallback은 제거하고, 전투의 continuous overlap 허용 모델에 맞춰 authored spawn position에 그대로 생성한다.

근거:
- 적 유닛은 continuous movement 모델에서 겹침이 허용되므로, spawn tile occupancy 때문에 nearest-open fallback을 수행할 필요가 없다.
- fallback은 authored spawn position source를 런타임에서 조용히 바꾸는 동작이다.
- 이를 spawn-zone capacity validation failure로 바꾸는 것도 현재 겹침 허용 모델과 맞지 않는다.

운영 방향:
- authored spawn position을 그대로 사용한다.
- 초기 spawn placement가 tile occupancy 때문에 막히지 않도록 battle setup placement와 runtime continuous movement overlap 규칙을 일관화한다.
- 필요하면 presentation용 formation offset은 별도 명시 정책으로 다룬다.

### remove single tile occupant projection

결정: `Battlefield` tile의 단일 `occupant` projection과 그 의미를 가진 코드는 강하게 제거한다.

근거:
- 전투 위치 source of truth는 unit별 위치/continuous body state다.
- 현재 장기 정책은 unit overlap을 허용하므로, 한 tile에 단일 occupant만 있다는 모델은 공식 runtime 의미와 충돌한다.
- 단일 occupant projection이 남아 있으면 target selection, AoE, spawn/deployment validation, movement/blocking code가 "타일 단일 소유"를 암묵적으로 다시 도입할 수 있다.

운영 방향:
- tile은 terrain/valid/static obstacle 같은 전장 정보만 가진다.
- unit 위치와 tile membership은 unit position/body state에서 계산한다.
- tile별 조회가 성능상 필요하면 단일 occupant가 아니라 `units_by_tile`/multi-occupant derived index로 설계한다.
- derived index는 unit position/body state에서 재계산되는 보조 인덱스이며 gameplay source of truth가 아니다.
- `occupant.is_some()` 류의 spawn/deployment/targeting/blocking 판정은 제거하고, 필요한 경우 명시적인 unit query 또는 collision/engagement system을 사용한다.
- debug/display-only 이름 변경으로 단일 occupant 의미를 보존하지 않는다. 관련 의미를 가진 코드까지 제거한다.

## battle_runtime

### UnitWithdrawn event log event

결정: `UnitWithdrawn` event log event를 추가한다.

근거:
- 철수는 전투 중 실제 상태 변화이며, 배치 상태와 이후 타겟팅/피격/스킬 가능성에 영향을 준다.
- command result와 snapshot만으로는 최종 상태는 알 수 있지만 event log에서 "언제 누가 철수했는가"라는 사건을 알기 어렵다.
- 전투 replay, debug, Unity 연출 관점에서 event log event로 남기는 것이 자연스럽다.

운영 방향:
- Unity-facing event DTO 추가로 다룬다.
- command result와 snapshot은 상태 확인 용도로 유지하되, 전투 사건 순서의 source는 event log event에 둔다.

### BehaviorResult boundary

결정: `BehaviorResult`는 장기적으로 command/domain별 result DTO로 분리한다.

근거:
- 현재 `BehaviorResult`는 게임 시작, 맵 노드 선택, node preview, support/maintenance/shop/reward, 장비/아이템/스킬 파편, 전투 업데이트, 배치/철수/스킬 발동 등 너무 많은 응답 의미를 한 enum에 담고 있다.
- command별로 반환 가능한 result 의미가 타입 표면에서 잘 드러나지 않는다.
- server mapping과 Unity `result_type` 분기가 계속 커질 수 있다.

운영 방향:
- 예: `MapCommandResult`, `NodeCommandResult`, `InventoryCommandResult`, `RewardCommandResult`, `BattleCommandResult`, `SupportCommandResult` 같은 domain별 result contract로 분리한다.
- 실제 변경은 game_server mapping과 Unity-facing result JSON 계약 리팩토링으로 다룬다.
- server `result_type`/payload mapping도 이 분리 작업에 포함해 domain별 result contract로 이동한다.
- 외부 JSON shape를 유지하기 위한 compatibility layer를 기본 방향으로 삼지 않는다.

### battle runtime numeric policy

결정: `MAX_BATTLE_TIME_MS`, deployment economy, playback speed 같은 battle runtime 수치는 장기적으로 live RON/data-driven policy로 이동한다.

근거:
- `RUN_SYSTEM_POLICY`와 마찬가지로 밸런스/UX 의미를 가진 수치는 코드 상수보다 명시적인 policy data source가 적합하다.
- 밸런스 조정과 Unity 표시 의미를 validation 가능한 데이터 계약으로 고정할 수 있다.

운영 방향:
- 밸런스/UX 수치는 RON/data-driven policy로 옮긴다.
- engine invariant나 안전장치 성격의 hard guard는 코드에 남길 수 있다.
- 실제 구현 시 schema, validation, Unity display 계약을 함께 설계한다.

### Battle event log naming

결정: 기존 `Timeline` 타입/JSON 이름은 전투 event log 성격에 맞게 `BattleEventLog` / `event_log` 계열 이름으로 rename한다.

근거:
- 기존 `Timeline`은 실제로 전투 중 발생한 사건 목록/event sequence에 가깝다.
- `Timeline`은 시간축 전체 모델, replay 데이터, 이벤트 리스트를 모두 암시할 수 있어 의미가 넓다.
- 역할이 event log라면 타입/JSON 명칭도 그에 맞추는 것이 장기적으로 명확하다.

운영 방향:
- Rust 타입과 Unity-facing JSON 계약을 함께 정리한다.
- 내부/외부 명칭 불일치를 남기는 compatibility layer를 기본 방향으로 삼지 않는다.

## battlefield_scenario_wave

### encounter-less combat preview and empty wave fallback

결정: encounter 없는 combat preview와 empty wave fallback은 당장 완전 금지한다.

근거:
- source-of-truth 정리 중 fallback을 넓게 열어두면 data 오류와 의도된 "웨이브 없는 전투 노드"를 구분하기 어렵다.
- 현재는 live encounter/wave data가 gameplay combat의 source여야 한다.
- 웨이브 없는 전투 노드가 실제 디자인으로 필요해질 수는 있지만, 그때는 fallback이 아니라 명시 타입/정책으로 여는 편이 안전하다.

운영 방향:
- official runtime과 debug/fixture path 모두 우선 강한 guard를 둔다.
- 나중에 필요하면 `NonWaveCombat` 같은 별도 명시 타입이나 node policy로 다시 연다.

### battlefield archetype random selection source

결정: Battlefield archetype random selection은 RON `weight`/`enabled` policy로 이동한다.

근거:
- Hard-coded `seed % N` selection table은 live battlefield authoring policy의 장기 source of truth로 부적합하다.
- archetype 등장 비율과 활성 여부는 content/balance data로 검증 가능해야 한다.

운영 방향:
- battlefield archetype RON에 `weight`와 `enabled` policy를 둔다.
- disabled archetype은 official runtime selection 후보에서 제외한다.
- validation은 enabled 후보가 비어 있지 않은지 확인한다.

### SplitRoom content policy

결정: `SplitRoom`은 live/random generation의 future placeholder로 유지하지 않는다.

근거:
- 실제 콘텐츠로 쓰지 않는 room type을 live generation 표면에 남기면 authoring source가 흐려진다.
- future placeholder는 validation과 runtime branch를 불필요하게 넓힌다.

운영 방향:
- `SplitRoom`이 실제 콘텐츠로 필요해지면 RON에서 명시적으로 enabled하고 validation을 추가한다.
- 그 전까지 official generation/runtime 후보로 유지하지 않는다.

### FacilityEntity future schema

결정: `FacilityEntity` PVE enemy schema variant는 미래의 상호작용 가능한 구조물/시설 entity를 위한 future schema로 유지하되, 현재 live PVE wave에서는 금지한다.

근거:
- 추후 데미지를 입히거나 힐을 사용할 수 있는 상호작용 가능한 구조물이 필요할 수 있다.
- 하지만 현재 live PVE wave에서 구현되지 않은 entity variant를 허용하면 runtime semantics, targetability, Unity 표시 계약이 비어 있는 상태가 된다.

운영 방향:
- `FacilityEntity` 정식 활성화 전까지 validation은 실패해야 한다.
- 구현 시에는 `FacilityEntityProfile`, battle entity semantics, targetability, damage/heal rules, Unity DTO 표시 계약을 함께 추가한다.

### authored defense route required

결정: Defense route는 official runtime에서 authored route를 의무로 한다.

근거:
- generated defense route를 runtime fallback으로 쓰면 live encounter/template authoring 오류를 숨길 수 있다.
- defense route는 이동/목표/전장 의미를 결정하는 gameplay source다.

운영 방향:
- live defense encounter/template에 route가 없으면 validation failure로 처리한다.
- generated defense route는 official runtime fallback으로 쓰지 않는다.
- 필요하면 editor/debug scaffold로만 둔다.

## skill_targeting_range

### explicit cast_targeting

결정: live RON과 내부 `SkillDef` construction 모두 cast-level target과 step execution target을 명시한다. `SkillCastTargetingDef::FirstStepTarget` backward-compatible default는 제거한다.

근거:
- `cast_targeting`은 스킬을 시전할 때 선택하는 대상/지점의 source of truth다.
- `steps[].target`은 각 step이 실제 효과를 적용하는 실행 규칙이며, 첫 step이 스킬 전체의 cast target을 대표한다고 볼 수 없다.
- multi-step skill에서는 첫 step이 self buff, delay, setup effect일 수 있고 이후 step이 cast target을 재사용할 수 있다.
- 첫 step에서 cast target을 추론하면 cast-level targeting과 step execution targeting의 source가 섞인다.

운영 방향:
- live skill RON은 모든 스킬에 explicit `cast_targeting`을 작성한다.
- 각 step은 독자적인 `target`/`targeting` 규칙을 계속 가질 수 있다.
- internal tests/fixtures도 첫 step에서 cast target을 추론하지 않는다.
- migration 후 `FirstStepTarget` variant는 제거한다.
- 작성 편의를 위한 helper/builder는 허용하지만, helper는 step을 읽어 추론하지 않고 caller가 cast target, range policy, tile range, air capability를 명시적으로 넘긴다.
- 장기적으로 `Explicit` bag-of-fields가 계속 장황하면 `SelfCast`, `EnemyUnit`, `TargetTile`, `WholeFieldEnemy` 같은 domain-specific cast targeting enum으로 재설계하는 것을 검토한다.
- Unity skill catalog/range preview/manual activation은 explicit cast target definition을 기준으로 검증한다.

### remove range_units from skill targeting

결정: live skill RON과 Unity skill catalog의 skill targeting/step targeting 표면에서 `range_units`를 제거한다.

추가 확정: `SkillStepDef.range_units` 자체도 장기적으로 제거한다.

근거:
- 현재 공식 스킬 판정은 타일 거리 기반이며, `range_policy`/`defense_tile_range`/runtime `range_previews.final_cells`가 source of truth다.
- `range_units`가 RON/catalog에 남아 있으면 "스킬 사거리"처럼 오해될 수 있지만 DefenseRoute target eligibility에서는 사용하지 않는다.
- 범용 스킬 사거리 source가 두 개처럼 보이는 것을 막아야 한다.
- step-level 공용 `range_units`를 유지하면 표시 범위, target 검색 반경, 전염/체인 거리, projectile 거리 의미가 다시 한 필드에 섞인다.

운영 방향:
- live skill RON의 cast/step targeting에서 `range_units`를 삭제한다.
- Unity skill catalog에서 범용 `range_units` 노출을 제거한다.
- 사거리 표시와 타겟 가능 여부는 tile range policy 및 runtime range preview를 기준으로 한다.
- 미래에 전염, chain, contact, aura, continuous mechanic이 거리 제한을 필요로 하면 `SkillStepDef.range_units`를 재사용하지 않는다.
- 해당 기능의 targeting/delivery 정책 안에 `max_distance_tiles`, `jump_range_tiles`, `max_jumps`, `target_count` 같은 목적별 명시 필드를 추가한다.
- 예: "가장 가까운 적 2명에게 감염"은 step-level `range_units`가 아니라 target rule의 `count`와 선택적 `max_distance_tiles`, 또는 `DeliveryDef::Contagion`/`Chain`의 전용 거리 필드로 표현한다.

### TileArea affected_tiles clipping

결정: runtime `SkillAreaDeclared.affected_tiles`는 preview처럼 valid tile로 clip한다.

근거:
- `affected_tiles`는 실제 gameplay affected tiles의 source of truth다.
- out-of-field/void tile을 포함하면 실제 피격 가능 영역과 Unity-facing event shape가 달라져 혼동이 생긴다.
- `range_previews.final_cells`와 runtime `SkillAreaDeclared.affected_tiles`가 같은 valid-tile clipping 규칙을 쓰는 편이 단순하다.

운영 방향:
- `SkillAreaDeclared.affected_tiles`에는 실제 유효 타일만 포함한다.
- out-of-field/void tile은 포함하지 않는다.
- Unity 시각 연출도 이 값을 기준으로 한다.
- 별도 "맵 밖까지 뻗는 시각적 투영"은 만들지 않는다.
- 곧 있을 tile/range 수정 작업에서 `Battlefield::in_bounds` 호출처를 감사한다.
- 즉시 wrapper를 추가하지 않는다. 실제 구현이 단순 bounds와 valid-tile policy를 혼동하고 있거나 호출 의도가 불명확한 경우에만 rename 또는 `is_valid_battle_tile` 계열 wrapper를 도입한다.

### long line and piercing skill representation

결정: 공식 DefenseRoute의 long line/piercing 계열 스킬은 기본적으로 `TileRangePattern`/`WholeFieldValidTiles`와 valid-tile clipping 조합으로 표현한다.

근거:
- 맵 끝, void tile, non-rectangular battlefield에서 잘리는 긴 직선/관통형 타일 스킬은 기존 tile range와 valid-tile clipping 조합으로 표현 가능하다.
- "맵 끝까지 날아감"을 별도 projectile range/source로 만들면 reach source가 중복된다.
- Unity preview와 runtime `SkillAreaDeclared.affected_tiles`가 같은 source를 보는 편이 단순하다.

운영 방향:
- tile-based long line/piercing skill은 `TileRangePattern` 또는 `WholeFieldValidTiles`를 우선 사용한다.
- directional projectile collision은 실제 이동하는 projectile, 충돌 순서, hit/pierce/kill stop policy, 시간차/속도 의미가 gameplay에 필요할 때만 사용한다.
- projectile `max_range` 류 필드를 tile-based map reach의 second source로 쓰지 않는다.

### runtime range preview source

결정: 기본 공격 범위와 스킬 범위 모두 runtime range preview를 실제 표시/타겟 가능 범위의 source로 사용한다.

근거:
- 우리 유닛은 기본 공격 범위와 스킬 범위가 서로 다를 수 있고, 스킬별로도 cast/area 범위가 다를 수 있다.
- 기본 공격 범위도 장비, 무기 profile, 위치, facing, battlefield valid tile, 상태 변화에 영향을 받을 수 있다.
- static skill catalog만으로 실제 클릭 가능 타일/대상을 확정하면 runtime state와 drift할 수 있다.

운영 방향:
- 평상시 유닛 선택/hover 상태에서는 runtime `basic_attack_range_preview`를 표시한다.
- 스킬 버튼 hover 또는 스킬 선택 모드 진입 시 Unity는 해당 `skill_id`의 runtime skill range preview command/query를 요청한다.
- 스킬 target 선택 중에는 runtime skill preview의 castable tiles, target candidates, affected tiles, unavailable reason을 source로 사용한다.
- static skill catalog는 설명, 아이콘, 기본 targeting metadata, 대략적인 UI 표시용으로만 사용한다.
- 유닛 위치/facing, 상태이상, cooldown/resource, 배치/철수, battlefield 변경, skill equip 변경이 있으면 기존 preview cache는 폐기한다.
- preview unavailable 상태에는 reason을 포함해 Unity disabled/tooltip 표시 source로 사용한다.

## basic_attack_projectile_judgement

### projectile launch owner snapshot

결정: 공격자가 projectile 발사 후 사망해도 projectile damage context는 launch owner snapshot을 유지한다.

근거:
- Projectile은 발사 시점에 owner/side/damage context가 정해진다.
- 발사 후 공격자가 사망해도 이미 날아가던 projectile의 소속과 피해 문맥이 사라지면 friendly fire, target side, replay/debug 의미가 흔들린다.
- "죽기 전에 쏜 공격이 이후 명중한다"는 전투 감각을 보존한다.

운영 방향:
- `attacker_side`/owner side/damage attribution은 launch owner snapshot을 따른다.
- 단, on-attack/on-hit trigger, lifesteal, chain effect 등 공격자 생존이 필요한 후속 proc은 impact 시점에 별도로 gated한다.

### projectile miss event source

결정: `BasicAttackProjectileImpacted(hit=false)`를 projectile miss의 canonical Unity-facing event로 두고, 중복 `ProjectileMiss` event는 제거한다.

근거:
- 현재 miss path는 `BasicAttackProjectileImpacted { hit: false }`와 `ProjectileMiss`를 모두 기록해 같은 의미를 두 이벤트가 동시에 말한다.
- Unity/replay가 miss VFX나 상태 전환을 어느 이벤트 기준으로 처리해야 하는지 애매해진다.
- Projectile lifecycle은 `BasicAttackProjectileLaunched` -> `BasicAttackProjectileImpacted(hit=true|false)` 한 쌍으로 보는 편이 자연스럽다.

운영 방향:
- Unity는 projectile 종료/명중/빗나감을 `BasicAttackProjectileImpacted`의 `hit` 필드로 판단한다.
- `ProjectileMiss` variant와 관련 validation/test는 제거하거나 `BasicAttackProjectileImpacted(hit=false)` 기준으로 교체한다.
- 추후 필요하면 `BasicAttackProjectileImpacted`에 `miss_reason` 같은 명시 필드를 추가한다.

### same timestamp attack resolve batch

결정: 같은 `time_ms`에 이미 scheduler queue에 들어간 `AttackResolve` batch는 battle end 조건이 발생해도 모두 drain한다.

근거:
- 같은 timestamp의 공격 resolve는 같은 순간에 발생한 전투 사건으로 보는 편이 combat feel이 자연스럽다.
- 첫 resolve로 battle end 조건이 성립하더라도, 같은 tick에 이미 예약된 공격 교환/동시 처치 가능성을 보존할 수 있다.
- 다만 battle end 이후 새 event scheduling이나 무조건 피해 적용을 허용해서는 안 된다.

운영 방향:
- 같은 `time_ms`의 `AttackResolve` batch는 drain한다.
- 각 resolve는 처리 시점의 live validity를 다시 검사한다.
- target dead, attacker dead, invalid target, battle-ending 상태 등으로 유효하지 않으면 canonical miss/no-op event로 정리한다.
- batch 처리 후 battle end를 확정한다.
- battle end 확정 이후 새 attack/effect scheduling은 금지한다.

### remove SplashClusterFirst targeting profile

결정: `SplashClusterFirst` targeting profile은 제거한다.

근거:
- splash/cluster 효율 판단은 플레이어의 스킬 사용/타겟 선택 의사결정 영역에 가깝다.
- 시스템이 "더 많이 맞는 지점/대상"을 자동으로 고르는 기능은 밸런스와 조작 의도를 침범할 수 있다.
- runtime은 스킬 preview로 affected tiles/targets를 제공하되, 최적 사용 판단을 대신하지 않는다.

운영 방향:
- `SplashClusterFirst` targeting profile과 관련 hard-coded cluster 평가식을 제거한다.
- 스킬은 runtime range/area preview를 통해 영향을 받을 타일/대상을 Unity에 제공한다.
- 자동 기본 공격 targeting도 cluster 효율 우선순위를 공식 profile로 두지 않는다.
- 향후 자동 타겟팅 보조가 필요하면 별도 명시 feature/policy로 재논의한다.

## damage_stats_entry

### move speed stat source

결정: `MoveSpeedUnitsPerMs` modifier는 실제 이동 속도에 영향을 주는 공식 stat이다.

근거:
- `MoveSpeedUnitsPerMs`는 이미 `UnitStats`와 stat modifier pipeline에 존재한다.
- live equipment data도 이동 속도 modifier를 사용하므로 표시/미래용 dead stat으로 유지하면 data 의미와 runtime behavior가 어긋난다.
- `UnitStats.move_speed_units_per_ms`와 `UnitBody.move_speed`가 다른 source를 보면 "움직일 수 있는지"와 "얼마나 움직이는지"가 분리된다.

운영 방향:
- battle spawn 시 `effective_stats().move_speed_units_per_ms`가 `UnitBody.move_speed`의 source가 된다.
- runtime `ModifyStats(MoveSpeedUnitsPerMs)`는 unit stats와 body move speed를 함께 갱신한다.
- movement speed source는 final stat으로 모으고, combat profile movement speed는 base stat input으로만 사용한다.

### move speed runtime update timing

결정: 이동 중 `MoveSpeedUnitsPerMs` modifier가 적용되면 다음 movement tick부터 실제 이동 속도에 반영한다.

근거:
- Unity-facing movement segment는 이미 받은 `MovementSegmentStarted.target`/`ends_at_ms`가 중간에 바뀌지 않는 immutable event로 보는 편이 안전하다.
- 다음 tick부터 반영하면 현재 segment를 즉시 중단/재발행하지 않아도 되어 presentation churn이 줄어든다.
- 전투 연출상 speed buff/debuff가 다음 movement tick부터 보이는 것은 자연스럽다.

운영 방향:
- 현재 active movement segment를 즉시 수정하지 않는다.
- 다음 `ContinuousMovementTick` 입력 생성 시 갱신된 `UnitBody.move_speed`를 사용한다.
- 필요하면 새 segment는 변경된 속도 기준으로 생성한다.

## buff_status_effect

### hard CC action restriction source

결정: Hard CC 행동 제한은 active hard CC buff expiry를 유일한 source of truth로 한다.

근거:
- Stun/Freeze 같은 hard CC의 시간은 gameplay와 UX에서 매우 정확해야 한다.
- active buff는 Unity 표시와 runtime status의 source이고, 별도 `ActionLocks`에 hard CC 시간이 남으면 표시 상태와 실제 행동 제한 시간이 drift할 수 있다.
- 긴 Stun 뒤 짧은 Freeze처럼 replacement가 발생할 때, 이전 hard CC lock이 몰래 남아 행동을 더 오래 막으면 source-of-truth가 깨진다.

운영 방향:
- Stun/Freeze 같은 hard CC는 active buff state가 공식 상태다.
- hard CC의 `expires_at_ms`가 movement/basic attack/cast/resonance gain 제한 종료 시각을 결정한다.
- `ActionLocks`는 공격 windup, skill focus/cast, resonance gain delay 같은 action sequence lock에만 사용한다.
- `ActionLocks`에 hard CC 시간을 별도로 max 누적하지 않는다.
- hard CC 교체/갱신 정책은 active buff expiry 갱신으로만 표현한다.
- 더 짧은 hard CC가 기존 hard CC를 교체하는 정책이라면 실제 행동 제한도 짧아질 수 있다.

### hard CC replacement event

결정: Hard CC replacement 시 기존 hard CC 종료를 Unity-facing event log에 명시한다.

근거:
- active hard CC buff가 행동 제한의 source of truth라면, replacement로 기존 CC가 사라지는 순간도 event log presentation에 명시되어야 한다.
- 새 `BuffApplied`만 보고 Unity가 기존 hard CC icon 제거를 추론하게 만들면 hard CC exclusivity 규칙이 client에 암묵적으로 새어 나간다.
- replay/debug/validator가 "왜 기존 hard CC가 원래 expiry 전에 끝났는지"를 명확히 알 수 있어야 한다.

운영 방향:
- 기본 방향은 기존 `BuffExpired` event를 사용하되 `reason: Replaced` 같은 종료 사유를 추가하는 것이다.
- 예: `BuffApplied(Stun)` -> `BuffExpired(Stun, reason=Replaced)` -> `BuffApplied(Freeze)`.
- 별도 `BuffReplaced` event는 필요성이 더 커질 때만 검토한다.

### clear active buffs on death

결정: 유닛 사망 시 해당 유닛의 active buff는 즉시 정리한다.

근거:
- 사망한 유닛에게 active buff가 남아 있으면 이후 scheduled `BuffTick`이 event log에 남아 Unity/replay가 사망 후 효과가 계속 발생하는 것처럼 볼 수 있다.
- 사망은 해당 유닛의 combat lifecycle 종료이므로, buff lifecycle도 함께 종료되는 편이 명확하다.
- dead target path에서 damage를 무시하더라도 `BuffTick` presentation event가 남으면 source-of-truth가 흐려진다.

운영 방향:
- 사망 처리 시 target/caster 관련 active buff를 정리한다.
- 사망으로 제거되는 buff는 event log에 종료 event를 명시한다. 기본 방향은 `BuffExpired(reason=TargetDied)` 같은 종료 사유를 쓰는 것이다.
- 사망 후 scheduled `BuffTick`/`BuffExpire`가 도착하면 active buff가 없으므로 event log event를 만들지 않는다.

### non-periodic control status max stacks

결정: Stun/Freeze/Silence 같은 non-periodic control status는 `max_stacks == 1`을 data validation invariant로 고정한다.

근거:
- Stun/Freeze는 hard CC replacement/exclusivity 정책을 갖고 있어 stackable status로 열어두면 행동 제한 source가 흐려진다.
- Silence도 현재 정책상 새 skill start blocker이며, stack 수가 gameplay 의미를 갖지 않는다.
- runtime이 stack을 1처럼 처리하면서 data에는 `max_stacks > 1`이 허용되면 authoring source와 runtime behavior가 어긋난다.

운영 방향:
- Buff metadata validation에서 Stun/Freeze/Silence의 `max_stacks == 1`을 강제한다.
- 향후 stackable silence 같은 디자인이 필요해지면 기존 Silence를 확장하지 말고 별도 status/effect policy로 재논의한다.

## employee_growth_trust

### remove unused growth ids

결정: 현재 실제 효과가 없는 `PveWinStack`/`QuestRewardStack`은 제거한다.

범위:
- `GrowthStack` 구조 전체를 제거하는 결정은 아니다.
- 현재 final stat pipeline에서 실제 효과가 있는 `KillStack`은 유지한다.
- `PveWinStack`은 현재 level-up 때 쌓이지만 전투/성장 효과가 없으므로 제거 대상이다.
- `QuestRewardStack`은 현재 공식 사용 경로와 효과가 없는 future placeholder이므로 제거 대상이다.

근거:
- no-op growth id가 runtime persistent state에 남으면 성장 보상 source처럼 보이지만 사용자-visible 효과가 없다.
- future hook을 미리 enum/runtime state로 유지하면 실제 정책이 확정되기 전에 계약 표면만 굳어진다.
- 향후 PVE 승리 보상이나 퀘스트 보상 성장이 필요하면 live RON/data-driven growth reward/effect로 명시적으로 다시 추가하는 편이 source-of-truth 기준에 맞다.

운영 방향:
- `GrowthId` enum과 stat pipeline/test에서 `PveWinStack`/`QuestRewardStack`을 제거한다.
- level-up은 더 이상 `PveWinStack`을 누적하지 않는다.
- 성장 보상 정책은 별도 data-driven growth policy에서 정의한다.

### trust feature surface

결정: 신뢰도 시스템의 dormant feature flag와 관련 타입은 당장 제거하지 않는다.

근거:
- 신뢰도 시스템은 공식 기능으로 추가할 계획이 있다.
- 현재 미연결/비활성 표면이 있더라도, 가까운 장래의 공식 기능화 대상이라면 이번 리팩토링에서 제거해 되돌림 비용을 만들 필요가 없다.
- 다만 실제 활성화 시에는 어떤 flow에서 어떤 flag가 켜지는지 명시 policy/source를 가져야 한다.

운영 방향:
- 이번 refactor audit/cleanup에서는 `TrustFeatureFlags`, trust event/modifier/cue 관련 표면을 보존한다.
- 신뢰도 기능을 공식 활성화할 때 live RON/data-driven policy 또는 명시 runtime policy gate를 추가한다.
- 그때 Unity-facing snapshot에 cue payload를 내릴지, Unity가 trust score/band/memory로 직접 분기할지도 함께 결정한다.

### growth curve and XP policy source

결정: level curve, grade 승급 구간, survival/reward XP는 장기적으로 data-driven growth policy로 이동한다.

범위:
- `experience_required_for_next_level()` 같은 level curve
- level에 따른 grade 승급 구간
- combat survival XP
- reward/session에서 지급되는 XP

근거:
- 성장 곡선, 등급 승급, XP 지급량은 밸런스 정책이다.
- 코드 상수/분기 안에 남기면 밸런스 조정과 Unity 표시 계약의 source가 흩어진다.
- live RON/data-driven policy로 옮기면 validation, content review, UI 표시를 같은 source에서 맞출 수 있다.

운영 방향:
- 실제 구현 시 growth policy RON/schema를 추가한다.
- runtime은 해당 policy를 읽어 level requirement, grade threshold, XP grant를 계산한다.
- policy 누락/잘못된 threshold는 validation failure로 처리한다.

### remove legacy employee grade

결정: employee `grade`는 레거시 별 등급 표면으로 보고 제거한다.

근거:
- `grade`는 원래 롤토체스식 1성/2성/3성에 가까운 레거시 개념이었다.
- `level`은 이후 추가된 성장 개념이며, grade와 level을 함께 유지하면 성장 source가 이중화된다.
- level-up 때 grade를 덮어쓰는 현재 구조는 starter candidate/data가 가진 grade와 충돌할 수 있다.

운영 방향:
- `EmployeeCombatProfile.grade`, starter/recruitment candidate grade, grade 기반 battle tier 계산을 제거하거나 level/명시 policy 기준으로 대체한다.
- level은 성장 진행도의 source로 유지한다.
- 별 등급/rarity/승급 시스템이 다시 필요하면 `grade`를 유지하지 말고 새 명시 시스템으로 재설계한다.
- growth policy에서 grade 승급 구간은 제거 대상이며, 필요한 성장 보정은 level/growth policy로 표현한다.

### trauma applies to final battle max HP

결정: 전투 시작 current HP는 final battle max HP 전체에 run HP ratio와 trauma penalty를 적용해 계산한다.

근거:
- trauma는 단순 장비 외부의 상처가 아니라 직원의 전투 수행 컨디션 패널티다.
- 장비 추가 HP만 trauma 영향을 피하면 tank 장비로 trauma를 우회할 수 있어 컨디션 관리 의미가 약해진다.
- final battle max HP 전체에 trauma를 적용하면 부상/트라우마가 전투 시작 생존력에 더 어렵고 명확하게 반영된다.

운영 방향:
- level/growth 같은 permanent 성장은 base/run max HP를 변경할 수 있다.
- 장비/일시 효과는 final battle max HP에 반영한다.
- 전투 시작 current HP는 `final_battle_max_hp * run_hp_ratio * trauma_factor` 계열의 정책으로 계산한다.
- run health와 battle health의 수명주기는 분리하되, 전투 진입 시 final battle max HP 전체에 컨디션 비율을 적용한다.

## item_equipment_skill_fragment

### remove automatic duplicate fragment conversion

결정: `SkillFragmentStackingPolicy::ConvertAdditionalCopiesToResource` 자동 변환 정책은 폐기하고 제거한다.

근거:
- 중복 스킬 파편은 자동으로 자원화하지 않는다.
- 플레이어는 Maintenance에 들러 직접 분쇄해서 fragment dust를 얻어야 한다.
- enum variant만 남겨두면 중복 획득 즉시 자원 변환 flow가 공식 정책처럼 보이지만, 실제 장기 UX와 다르다.

운영 방향:
- `ConvertAdditionalCopiesToResource` variant와 `NotImplemented` branch를 제거한다.
- 중복 파편 획득은 공식 정책에 따라 stack/보유 상태로 남긴다.
- fragment dust 획득은 Maintenance dismantle command/result를 canonical flow로 둔다.

### skill fragment equip limit policy

결정: 모든 skill fragment는 장착 copy 제한을 가지며, 높은 등급 fragment는 copy 수와 무관한 global exclusive 제한을 가질 수 있다.

정책:
- 하나의 skill fragment copy는 한 직원에게만 장착 가능하다.
- 같은 fragment를 N명의 직원이 동시에 장착하려면 기본적으로 inventory count가 N 이상이어야 한다.
- 높은 등급 fragment는 copy가 여러 개 있어도 동시에 한 직원만 장착 가능한 `GlobalExclusive` 정책을 가질 수 있다.

운영 방향:
- fragment metadata에 `equip_limit` 같은 명시 필드를 둔다.
- 기본 정책 후보는 `PerOwnedCopy`와 `GlobalExclusive`다.
- `PerOwnedCopy`: simultaneous equipped count <= owned count.
- `GlobalExclusive`: simultaneous equipped count <= 1, owned count와 무관.
- rarity 기반 default/helper는 둘 수 있지만 최종 source는 fragment RON의 explicit `equip_limit`로 둔다.
- 장착/분쇄/보상/스냅샷 검증은 equipped count와 owned count 관계를 같은 evaluator로 확인한다.

### skill fragment dismantle auto unequip

결정: Skill fragment 분쇄는 허용하되, 분쇄 후 장착 제한을 위반하는 경우 runtime이 필요한 만큼 deterministic auto-unequip을 수행한다.

근거:
- 플레이어가 Maintenance에서 분쇄를 선택한 경우 분쇄 flow 자체는 막지 않는 편이 UX가 가볍다.
- 다만 분쇄 후 owned count와 equipped count invariant는 runtime source of truth가 반드시 맞춰야 한다.
- Unity warning은 UX 보조이며, 최종 일관성 보장은 runtime command가 담당한다.

운영 방향:
- Maintenance preview/evaluator는 분쇄 시 자동 해제될 직원/slot 목록을 계산한다.
- Unity는 preview의 `will_unequip_employee_ids`/`affected_loadouts` 계열 정보를 보고 경고/확인 UI를 표시한다.
- command 실행은 같은 evaluator를 사용해 deterministic auto-unequip을 적용한다.
- command result에도 실제 auto-unequip 대상과 남은 equipped state를 명시한다.
- auto-unequip 순서는 roster order 뒤쪽부터 같은 deterministic rule로 고정한다.

### fragment dust resource source

결정: `fragment_dust`는 equipment material DB/schema에 합치지 않고 skill fragment economy의 독립 resource로 유지한다.

근거:
- `fragment_dust`는 장비 제작/강화 재료가 아니라 skill fragment 강화/개화 전용 자원이다.
- equipment material DB에 합치면 장비 재료 taxonomy가 skill fragment progression 경제까지 소유하게 되어 경계가 흐려진다.
- 독립 resource로 유지하면 파편 전용 획득처, cap, 비용, UI 분류, 성장 정책을 별도로 조정하기 쉽다.

운영 방향:
- `fragment_dust`의 source of truth는 `SkillFragmentInventory`/fragment economy에 둔다.
- Maintenance/Unity 표시에서는 typed resource/material entry처럼 함께 노출할 수 있다.
- 문자열 `"fragment_dust"` 하드코딩은 canonical constant/typed DTO로 통일한다.
- `equipment_dust`와 `fragment_dust`는 UI에서 나란히 보일 수 있지만 내부 resource ownership은 분리한다.

### bound equipment interaction lock

결정: Bound 장비는 전용 해제 이벤트/정책 전까지 분해를 포함한 모든 일반 상호작용을 금지한다.

근거:
- 모든 장비가 귀속인 것은 아니며, 일부 장비만 `bound` 의미를 가진다.
- bound 장비는 단순히 unequip만 막는 장비가 아니라, 전용 해제 조건 전까지 플레이어가 임의로 조작할 수 없는 장비다.
- Maintenance dismantle로 bound 장비를 제거할 수 있으면 bound 의미를 우회하게 된다.

운영 방향:
- bound 장비는 unequip, dismantle, combine, enhance, sell, consume-like operation 등 일반 상호작용 대상이 될 수 없다.
- bound 해제는 별도 명시 이벤트/정책으로만 가능하다.
- Maintenance preview와 command validation은 bound 장비 작업을 disabled/reject하고 reason을 제공한다.
- bound 해제 이벤트가 추가될 때 제거 조건과 후속 가능 interaction을 별도 정책으로 정의한다.

## reward_shop_support

### canonical grant executor

결정: 현재 `RewardExecutor`는 이름을 변경하고 상위 개념으로 승격하여, 게임 내 모든 획득/지급 mutation의 canonical executor로 둔다.

범위:
- Enkephalin 지급
- equipment/artifact/consumable 지급
- equipment material 지급
- skill fragment 지급
- skill fragment research progress 지급
- fragment dust 지급
- employee XP 지급
- starter grant, combat reward, event/reward node, shop purchase/sell, headquarters supply, admin grant, maintenance dismantle result 같은 모든 획득/지급 flow

근거:
- 현재 지급 mutation은 `RewardExecutor`, combat XP path, shop, headquarters, admin grant, maintenance 등 여러 곳에 흩어져 있다.
- 같은 아이템/재화 지급에도 inventory capacity, artifact duplicate, owned UUID, XP target, Enkephalin overflow, fragment policy 같은 규칙을 여러 곳에서 다시 구현할 위험이 있다.
- 모든 획득/지급을 하나의 executor로 모으면 source-of-truth와 diff/result/audit/debug 경계가 명확해진다.

운영 방향:
- `RewardExecutor`는 `GrantExecutor`, `AcquisitionExecutor` 등 더 넓은 이름으로 rename한다.
- 모든 지급 flow는 typed grant instruction/result를 만들고 canonical grant executor를 통해 적용한다.
- reward node는 grant instruction authoring source 중 하나로 남는다.
- shop purchase처럼 비용 소비와 stock 제거가 있는 flow에서 domain-specific cost/stock validation은 shop domain이 담당하되, 최종 아이템 지급 mutation은 grant executor가 담당한다.
- maintenance dismantle처럼 자원 획득과 장착 해제가 함께 있는 flow는 domain evaluator가 auto-unequip/소비를 결정하고, 지급 mutation은 grant executor를 사용한다.
- admin grant도 dev/debug wrapper일 뿐 지급 mutation은 같은 executor를 사용한다.
- executor result는 Unity-facing diff와 audit/debug log의 source가 된다.

### atomic reward claim

결정: Reward claim은 전체 reward session 단위로 원자적으로 처리하고, partial grant를 official success path로 허용하지 않는다.

근거:
- Reward claim은 사용자-visible 보상 지급 시점이다.
- 일부 effect만 먼저 적용된 뒤 뒤쪽 effect가 실패하면 "claim 실패인데 일부 보상은 들어간" 상태가 생긴다.
- partial grant를 허용하면 재시도, 보상 소모 여부, Unity toast/result 표시, snapshot diff 해석이 모두 애매해진다.

운영 방향:
- reward claim 전에 전체 reward session/effect를 preflight validation한다.
- 하나라도 지급 불가능하면 state를 변경하지 않고 claim 전체를 실패시킨다.
- preflight 통과 후에만 실제 reward effects를 적용한다.
- 구현상 transaction/rollback helper를 쓰더라도 외부 계약은 "전부 성공 or 전부 실패"로 고정한다.

### reward granted fragment and research diff

결정: `RewardGranted`에는 skill fragment 획득 diff와 skill fragment research progress diff를 추가한다.

근거:
- `GrantSkillFragment`와 `GrantSkillFragmentResearch`는 실제 runtime state를 변경하지만 현재 `RewardGranted.inventory_diff`에는 드러나지 않는다.
- snapshot은 최종 상태 source이지만, claim 순간의 "무엇을 받았는가" presentation/result source를 대체하기에는 부족하다.
- Unity가 reward definition과 다음 snapshot을 비교해 보상 지급 내용을 재추론하게 만들면 표시 source가 흐려진다.

운영 방향:
- `RewardGranted`/관련 combat reward result에 skill fragment diff와 research progress diff를 명시한다.
- snapshot의 `skill_fragments`, `skill_fragment_progress`, `pending_research_deliveries`는 최종 상태 source로 유지한다.
- claim 순간 toast/result UI는 `RewardGranted` diff를 기준으로 표시한다.

### reward experience target policy

결정: `GrantExperience`는 `RewardExecutor`가 공식 처리하며, reward effect에 explicit XP target policy를 둔다.

근거:
- XP는 combat reward뿐 아니라 event/reward node에서도 지급될 수 있다.
- `GrantExperience { amount }`만으로는 누구에게 XP를 줄지 알 수 없어 source-of-truth가 흐려진다.
- combat participant XP와 event node XP를 같은 reward executor 체계로 처리하되, 대상 정책을 RON에 명시하는 편이 안전하다.

운영 방향:
- `GrantExperience` effect는 `amount`와 `target` policy를 가진다.
- 기본 target 후보: `CombatParticipants`, `AliveRoster`, `SelectedEmployee`, `AllRoster`.
- combat reward는 `CombatParticipants` 같은 전투 결과 기반 target을 사용한다.
- event/reward node는 `AliveRoster` 또는 `SelectedEmployee`처럼 node 정책에 맞는 target을 명시한다.
- target이 필요한데 reward session/context에 대상 정보가 없으면 validation 또는 command failure로 처리한다.
- `RewardGranted`/combat reward result에는 XP 지급 diff를 포함한다.

### enkephalin overflow policy

결정: Enkephalin은 gameplay cap을 두지 않고, arithmetic overflow만 checked failure로 처리한다.

근거:
- Enkephalin은 현재 shop purchase, reward, sell, emergency supplies, starter grant 등에 쓰이는 run currency다.
- 명시 max cap은 경제/UX 밸런스 의미가 큰 별도 정책이다.
- 현재는 cap으로 소비 압박을 만들기보다, 비정상 overflow만 실패시키는 편이 단순하다.
- saturating add는 초과분 손실을 조용히 숨길 수 있어 source-of-truth 기준에 맞지 않는다.

운영 방향:
- Enkephalin 증감은 공통 checked helper로 통일한다.
- overflow 발생 시 command/data 오류로 실패한다.
- `saturating_add` 기반 증가 path는 제거하거나 checked helper로 교체한다.
- 향후 cap이 필요하면 explicit max cap policy와 overflow UX를 별도로 정의한다.

### remove reward tags

결정: `RewardTag`/reward `tags` field는 제거한다.

근거:
- `GrantExperience`, `GrantEquipment`, `GrantSkillFragment` 같은 typed grant instruction이 이미 보상 의미를 표현한다.
- `Currency`, `Experience`, `Equipment`, `Artifact`, `Consumable`, `SkillFragment`, `ResearchProgress` 같은 tags는 grant semantics와 중복된다.
- tag가 reward pool filtering, combat reward policy, map reward filtering에 쓰이면 effect/grant 의미와 drift할 수 있다.
- 범용 tag를 유지하면 source-of-truth가 grant instruction과 tag로 나뉜다.

운영 방향:
- reward classification은 canonical grant executor의 typed grant instruction semantics에서 파생한다.
- reward pool/mission에서 허용 보상 종류를 제한해야 하면 tag filter가 아니라 `allowed_grant_kinds` 같은 typed pool policy를 사용한다.
- UI 표시 카테고리가 필요하면 tag가 아니라 `display_category` 같은 명시 필드로 둔다.
- `Forbidden`은 tag가 아니라 live data validation failure로 처리한다.
- `Narrative`는 reward tag로 두지 않고, 필요하면 event/node metadata 또는 별도 narrative policy로 이동한다.
- 범용 `tags`를 `tags2` 같은 새 source로 되살리지 않는다.

### explicit equipment reward pools

결정: `GrantEquipment { equipment_id: None }` 전체 장비 DB 랜덤 보상은 제거하고, typed filter/weight를 가진 명시적인 equipment reward pool로 대체한다.

근거:
- 전체 장비 DB 랜덤은 tutorial-only, boss-only, bound, generated-only, test 성격 장비까지 reward 후보로 섞을 수 있다.
- reward authoring 의도는 장비 DB 전체가 아니라 reward data/pool에 명시되어야 한다.
- 완전 고정 목록만 쓰면 보상 authoring이 경직되므로, 등급/타입/제외 조건 같은 typed filter와 weight를 pool에 둘 수 있게 한다.

운영 방향:
- `equipment_id: Some(id)`처럼 특정 장비를 직접 지급하는 보상은 유지할 수 있다.
- `equipment_id: None`이 전체 DB 랜덤을 의미하는 경로는 제거한다.
- 랜덤 장비 보상은 `GrantEquipmentFromPool(pool_id)` 같은 명시 effect로 표현한다.
- equipment reward pool은 자유 문자열 조건식이 아니라 typed filter/weight schema를 사용한다.
- 기본 filter 후보: `rarity_min`, `rarity_max`, `equipment_type_in`, `exclude_bound`.
- 기본 weight 후보: rarity별 weight, explicit equipment id별 weight.
- validation은 pool이 비어 있지 않은지, filter/weight가 실제 장비 후보를 만들 수 있는지 검증한다.

### forbidden abnormality reward validation

결정: `ForbiddenAbnormalityGrant`는 live reward data에서 제거하고 data validation failure로 전환한다.

근거:
- `ForbiddenAbnormalityGrant`는 보상을 지급하는 effect가 아니라 "환상체를 인벤토리 보상으로 주면 안 된다"는 guard다.
- 실패하는 보상이 live reward data에 정규 reward처럼 남아 있으면 authoring source가 흐려진다.
- runtime filtering으로 forbidden reward를 숨기는 것보다, live data validation에서 authoring 오류로 실패시키는 편이 source-of-truth 기준에 맞다.

운영 방향:
- live reward RON에서 `ForbiddenAbnormalityGrant` reward/effect를 제거한다.
- reward pool이 forbidden reward를 참조하면 validation failure로 처리한다.
- `ForbiddenAbnormalityGrant` variant와 executor branch는 제거한다.
- 환상체는 player inventory reward로 materialize하지 않는다는 규칙을 reward executor가 아니라 data validation/schema에서 보장한다.

## data_ron_validation

### starter employee loadout source

결정: 스타팅 직원에게 부여되는 기본 장비와 기본 스킬 파편은 live RON에서 명시한다.

범위:
- 스타팅 직원의 기본 장비/장착 상태
- 스타팅 직원의 baseline skill fragment 목록
- `starter_basic_attack_enhancement` 같은 기본 파편 metadata

근거:
- 스타팅 직원의 초기 loadout은 gameplay content이며, live RON/data가 source of truth여야 한다.
- 현재 `Employee::with_combat_profile()`은 `EmployeeLoadout::default()`와 `SkillFragmentLoadout::starter()`를 코드에서 주입한다.
- `SkillFragmentDatabase::with_builtin_starter()`도 RON에 기본 파편이 없으면 코드 metadata를 자동 추가하므로, 기본 파편 정의가 RON과 코드에 나뉜다.
- 초기 지급 장비/파편이 코드에 숨어 있으면 content review, 밸런스 조정, Unity 표시 계약이 같은 data source를 보지 못한다.

운영 방향:
- starter candidate 또는 starter loadout policy RON에 기본 장비 id와 baseline skill fragment id를 명시한다.
- `starter_basic_attack_enhancement` metadata는 live skill fragment RON에 명시한다.
- 코드 builtin 주입(`with_builtin_starter`, hard-coded starter loadout source)은 제거한다.
- starter loadout이 참조하는 equipment/skill fragment가 live data에 없으면 validation failure로 처리한다.
- 기본 장비가 inventory owned instance인지, 직원에게 bound/equipped 상태로 바로 생성되는지는 구현 시 schema로 명시한다.

### remove legacy random event RON

결정: legacy random event RON files는 삭제한다.

대상:
- `../game_resources/data/events/legacy_random_events.ron`
- `../game_resources/data/events/legacy_event_pools.ron`
- `../game_resources/data/events/rewards/legacy_random_event_rewards.ron`
- `../game_resources/data/events/shops/legacy_random_event_shops.ron`
- `../game_resources/data/abnormalities/legacy_random_event_abnormalities.ron`
- official live loader가 읽지 않는 legacy/random-event 성격의 중복 event data

근거:
- 공식 live loader는 `events/shops/base.ron`과 `events/rewards/base.ron`만 읽는다.
- legacy random event 파일은 runtime source of truth가 아니면서 live data tree 안에 남아 있어 공식 데이터로 오해될 수 있다.
- 레거시/fallback 제거 기준상 참고용 dead content를 live data tree에 유지하지 않는다.

운영 방향:
- 구현 단계에서 legacy random event RON files를 삭제한다.
- 삭제 후 official live loader/test loader가 base data만 읽는지 확인한다.
- legacy 파일에만 존재하던 reward/shop/event 정책은 최신 live RON으로 migration하지 않는다. 필요하면 새 정책으로 재작성한다.

### server startup live preview validation

결정: Server startup에서 generated combat preview contract validation까지 실행한다.

근거:
- live RON/data는 gameplay source of truth이므로 잘못된 combat preview contract를 가진 서버가 뜨면 안 된다.
- 테스트/CI에서만 잡히는 live data 오류는 local/dev server 첫 runtime use까지 밀릴 수 있다.
- fail-fast startup validation이 live data authoring 오류를 더 명확하게 드러낸다.

운영 방향:
- official live data loader/startup validation entry point에 generated combat preview contract validation을 포함한다.
- server와 tests는 같은 validation entry point를 사용한다.
- validation 비용이 문제가 되면 별도 성능 측정 후 dev/prod policy를 논의하되, 기본 정책은 startup fail-fast다.

## unity_server_contract

### shared gameplay command DTO

결정: WebSocket transport envelope는 유지하되, gameplay command enum/payload source는 core/shared typed DTO로 통합한다.

범위:
- server `PlayerBehaviorRequest`와 core `PlayerBehavior`가 중복 정의하던 gameplay command payload를 하나의 shared/core source로 모은다.
- WebSocket `request_id`, top-level message type, session/routing metadata, snake_case JSON contract 같은 transport envelope는 server 소유로 유지한다.
- `request_battle_resync`처럼 transport-only 의미가 있는 요청은 gameplay command와 섞지 않고 명시적으로 server/envelope boundary에 둔다.

근거:
- 현재 gameplay command shape가 server enum과 core enum에 중복되어 command 추가/수정 시 drift 위험이 있다.
- 하지만 WebSocket envelope는 Unity-facing transport contract이므로 core gameplay command와 무리하게 합치면 transport 정책이 core로 새어 들어간다.
- source-of-truth 기준상 게임 명령 payload는 하나의 typed DTO/enum이 소유하고, server는 transport wrapping과 routing을 담당하는 편이 명확하다.

운영 방향:
- `GameplayCommandDto` 또는 core `PlayerBehavior` 계열을 shared command source로 정리한다.
- server request parser는 envelope를 deserialize한 뒤 shared gameplay command payload로 변환/보관한다.
- public gameplay command가 WebSocket mapping을 갖는지, 또는 core-internal only reason이 있는지 coverage test를 추가한다.
- Unity-facing JSON casing/transport shape 변경은 별도 계약 변경으로 다룬다.

### typed run snapshot DTO

결정: Run snapshot은 typed DTO로 전면 이관한다.

범위:
- `game_state_context`
- `allowed_actions`
- `selected_event`
- `inventory`
- `roster`
- `skill_catalog`
- 그 외 Unity-facing run snapshot top-level fields

근거:
- Run snapshot은 Unity-facing state source인데 현재 대부분 `serde_json::json!`로 직접 조립된다.
- ad-hoc JSON 조립은 필드 누락, 오타, 의미 drift를 컴파일 타임에 잡기 어렵다.
- 전체 snapshot shape를 typed DTO로 표현하면 Unity-facing contract를 코드에서 명확히 볼 수 있고, snapshot shape test도 안정적으로 작성할 수 있다.

운영 방향:
- run snapshot 전체를 typed DTO struct/enum으로 구성한 뒤 serialize한다.
- serialized JSON shape는 기존 Unity-facing 계약을 유지한다.
- 필드 추가/삭제/rename, 값 의미 변경은 별도 계약 변경으로 다룬다.
- `skill_catalog`처럼 이미 typed DTO인 표면은 기존 방향을 유지하고 전체 snapshot DTO에 편입한다.
- 장기적으로 server/admin serialization 경계도 Unity/server-facing 계약 shape를 typed DTO로 표현한다.
- `serde_json::Value` 자체를 금지하지는 않는다. 최종 serialization, logging, generic passthrough처럼 계약 shape를 소유하지 않는 boundary에서는 사용할 수 있다.
- 하지만 `Value` 객체에 문자열 key로 gameplay/server-facing field를 삽입하거나 수정해 최종 계약 shape를 조립하는 구조는 제거 대상이다.
- 예: `selected_event["compressed_event_log"] = ...` 같은 server 후처리는 typed server snapshot/selected-event DTO로 이동한다.
- core DTO와 server 후처리를 합쳐야만 최종 Unity-facing snapshot shape가 보이는 구조를 장기적으로 없앤다.

### combat result event log attachment ownership

결정: combat result event log attachment의 gameplay 의미는 core가 typed DTO로 제공하고, server는 transport serialization/compression을 담당한다.

근거:
- 현재 core run snapshot은 `selected_event.has_event_log`를 제공하고, server가 snapshot JSON에 `selected_event.compressed_event_log`를 삽입한다.
- 이 구조에서는 core-only snapshot과 Unity가 실제로 받는 server snapshot의 source가 달라진다.
- `compressed_event_log`의 압축/encoding은 transport 책임이지만, "전투 결과에 어떤 event log attachment가 연결되는가"는 gameplay/result contract다.

운영 방향:
- core는 combat result selected event에 붙는 typed event log attachment 의미를 제공한다.
- attachment에는 winner, event log identity/content, availability 같은 gameplay 의미를 포함한다.
- server는 core attachment를 받아 WebSocket 전송용으로 압축/직렬화한다.
- server가 gameplay 의미를 판단해 ad-hoc JSON field를 삽입하는 구조는 제거한다.
- Unity-facing field name/encoding 변경이 필요하면 별도 transport 계약 변경으로 다룬다.

### remove dormant battle resync setup field

결정: `BattleResync.setup` field는 제거한다.

근거:
- `BattleResync.setup`은 optional field로 존재하지만 현재 constructor가 항상 `None`으로 설정한다.
- setup 손실 복구는 이미 `battle_setup_snapshot` + `battle_update` 흐름을 official path로 사용한다.
- 사용하지 않는 optional setup field를 남기면 `battle_resync` 안에 setup이 중첩될 수 있는 dual-shape contract처럼 보인다.

운영 방향:
- `BattleResync`는 update/checkpoint catch-up 전용 message로 둔다.
- `setup` field와 관련 test expectation을 제거한다.
- `request_battle_resync { need_setup: true }`는 기존처럼 setup-loss recovery 흐름으로 처리한다.
- setup 복구의 official transport는 `battle_setup_snapshot` 후 `battle_update`다.

## validation_test_harness

### remove old event log validation acceptance

결정: old event log version `6`/`7` normal validation acceptance는 제거한다.

근거:
- version `6`/`7`은 레거시 event log schema다.
- normal validation이 과거 버전을 계속 허용하면 현재 공식 event log 계약과 archive/import compatibility 경계가 흐려진다.
- 리팩토링 기준상 레거시 compatibility path는 기본적으로 유지하지 않는다.

운영 방향:
- normal event log validation은 현재 공식 schema version만 허용한다.
- version `6`/`7` acceptance와 관련 normal-path tests는 제거하거나 최신 schema test로 교체한다.
- 과거 로그를 읽어야 할 필요가 생기면 normal validator가 아니라 별도 archive/import validator로 분리하고 제거 조건을 문서화한다.

### live content audit manifest

결정: live content roster/coverage manifest는 Rust test source 안의 hard-coded list가 아니라 versioned data/audit manifest로 이동한다.

범위:
- live abnormality roster 기대 목록
- live skill catalog delivery/audit 목록
- live skill behavior-test coverage 목록
- 그 외 live content drift를 막기 위한 catalog/coverage manifest

근거:
- live content roster/coverage tests는 의도치 않은 RON drift를 잡는 중요한 gate다.
- 하지만 기대 목록이 Rust test code 안에 있으면 콘텐츠 정책 manifest가 코드에 숨어 있는 셈이다.
- 콘텐츠 추가/삭제가 의도된 변경이라면 test source보다 versioned audit manifest를 갱신하는 편이 source-of-truth가 명확하다.

운영 방향:
- 기존 roster/coverage tests는 유지한다.
- expected live content 목록은 `docs/audit/*` 또는 `game_resources/audit/*` 같은 명확한 manifest 위치로 이동한다.
- tests는 live RON을 읽고 audit manifest와 비교한다.
- live content 추가/삭제/coverage 변경은 manifest 갱신을 동반해야 한다.

## Provisional Decisions By Codex

이 섹션은 사용자가 "간단히 정할 수 있는 정책은 임시로 정하고, 어려운 것은 마지막에 정하자"고 요청한 뒤 Codex가 기존 refactor 원칙에 따라 임시 결정한 항목이다.

공통 기준:
- compatibility/fallback/dual schema는 기본적으로 제거한다.
- live RON/data를 source of truth로 옮기기로 한 영역은 validation 가능한 schema로 이동한다.
- Unity-facing DTO 변경은 장기 리팩토링에서 허용하되, 변경은 명시 계약 작업으로 다룬다.
- 게임 디자인/밸런스/UX 의미가 큰 항목은 임시 결정하지 않고 마지막 논의 대상으로 남긴다.

### employee_growth_trust provisional decisions

보류:
- 신뢰도 시스템 관련 정책은 잠시 모두 보류한다.
- Trust dialogue cue를 core snapshot이 내릴지 Unity가 분기할지는 신뢰도 공식 기능화 때 재논의한다.
- 전투 incapacitation/trauma를 trust memory로 기록할지는 감정/서사 정책이라 마지막 논의에 남긴다.
- Medical trust event 범위는 trust/지원 서사 정책이라 마지막 논의에 남긴다.

## Confirmed Policy Implementation Status

이 섹션은 정책은 확정됐지만 runtime/server/admin 구현 상태를 추적한다. 완료된 항목은 구현 goal과 검증 근거를 함께 남겨 같은 작업을 반복하지 않게 한다.

### Skill targeting cleanup

- 구현 상태: `core_unimplemented_policy_implementation_master` / `core_skill_targeting_explicit_cleanup`에서 구현 완료.
- `SkillStepDef.range_units`는 제거한다.
- 거리 제한이 필요한 전염, chain, contact, aura, continuous mechanic은 `SkillStepDef.range_units`를 재사용하지 않고 targeting/delivery 전용 필드로 설계한다.
- `SkillCastTargetingDef::FirstStepTarget`은 제거한다.
- live RON과 internal tests/fixtures 모두 cast-level target과 step execution target을 명시한다.
- 작성 편의 helper/builder는 허용하지만, step 목록을 읽어 cast target을 추론하지 않는다.
- `Explicit` bag-of-fields가 계속 장황하면 `SelfCast`, `EnemyUnit`, `TargetTile`, `WholeFieldEnemy` 같은 domain-specific cast targeting enum을 검토한다.

### Battlefield tile occupancy cleanup

- 구현 상태: `core_unimplemented_policy_implementation_master` / `core_battlefield_occupancy_cleanup`에서 구현 완료.
- `Battlefield` tile의 단일 `occupant` projection과 그 의미를 가진 코드는 제거한다.
- unit 위치/continuous body state가 전투 위치의 source of truth다.
- tile별 조회가 필요하면 단일 occupant가 아니라 multi-occupant derived index로 설계한다.
- spawn/deployment/targeting/blocking 판정에서 `occupant.is_some()` 류의 타일 단일 소유 의미를 제거한다.

### Tile valid-check naming audit

- 구현 상태: `core_unimplemented_policy_implementation_master` / `core_tile_validity_typed_dto_cleanup`에서 감사 완료. 현재 call site는 raw bounds와 valid-tile policy를 섞지 않으므로 wrapper/rename을 추가하지 않았다.
- 곧 있을 tile/range 수정 작업에서 `Battlefield::in_bounds` 호출처를 감사한다.
- 즉시 wrapper를 추가하지 않는다.
- raw bounds와 valid-tile policy가 섞였거나 호출 의도가 불명확할 때만 rename 또는 `is_valid_battle_tile` 계열 wrapper를 도입한다.

### Server/admin typed DTO boundary

- 구현 상태: `core_unimplemented_policy_implementation_master` / `core_tile_validity_typed_dto_cleanup`에서 `selected_event["compressed_event_log"]` server 후처리 경로 구현 완료. `serde_json::Value`는 final transport/admin generic payload boundary에만 남겼다.
- server/admin serialization 경계의 Unity/server-facing 계약 shape도 typed DTO로 표현한다.
- `serde_json::Value`는 최종 serialization, logging, generic passthrough처럼 계약 shape를 소유하지 않는 boundary에서만 사용한다.
- `Value`에 문자열 key로 gameplay/server-facing field를 삽입하거나 수정해 최종 계약 shape를 조립하는 구조는 제거한다.
- `selected_event["compressed_event_log"] = ...` 같은 server 후처리는 typed server snapshot/selected-event DTO로 이동한다.

## Confirmed But Unimplemented Follow-Up Policies

이 섹션은 완료 검토 중 발견한 미구현/미흡 항목을 추적한다. 확정된 정책은 별도 정책 질문 없이 구현 goal로 진행할 수 있다. Unity-facing DTO shape, external error code, 저장 데이터 migration처럼 계약 변경이 새로 드러나면 해당 goal을 완료 산출물로 종료하고 사용자와 논의한다.

### Battlefield static layout extraction

결정: 현재 `Battlefield`는 완전 삭제하지 않고 정적 전장 layout 객체로 축소/rename한다.

구현 상태: `core_followup_policy_implementation_master` / `core_battlefield_layout_extraction`에서 구현 완료.
`Battlefield`는 `BattlefieldLayout`으로 rename됐고, `unit_pos` / `position_of` / `place` / `remove` / `units_at` / tile `occupant` state는 제거됐다. live deployment/checkpoint/event snapshot tile position은 `RuntimeUnit.body.projected_tile()` 또는 `UnitBody::projected_tile()`에서 파생한다.

운영 방향:
- 장기 이름은 `BattlefieldLayout` 또는 `BattlefieldGrid` 계열을 우선 검토한다.
- 정적 layout 객체는 width, height, valid tiles, void tiles, static obstacles, walkable/valid tile query만 소유한다.
- 전투 중 유닛 위치의 canonical source는 `RuntimeUnit.body.position`이다.
- tile 위치가 필요하면 `RuntimeUnit.body.position.project_to_tile()` 또는 `UnitBody::projected_tile()`에서 파생한다.
- tile별 유닛 조회가 필요하면 `BattleCore`가 현재 `units` / `RuntimeUnit.body`에서 계산하는 derived query로 제공한다.
- `Battlefield.unit_pos`, `position_of`, `place`, `remove`, `units_at`처럼 동적 유닛 위치를 저장하거나 생존/배치 상태를 암묵적으로 표현하는 API는 제거한다.
- 사망, 철수, 배치 여부는 `BattleCore.units`, unit stats/action state, live deployment state처럼 해당 domain runtime state에서 표현한다.

근거:
- continuous movement는 `RuntimeUnit.body.position`을 갱신한다.
- `Battlefield.unit_pos`는 spawn/deploy 당시 tile position을 별도로 보존하므로 movement 이후 `body.position`과 drift될 수 있다.
- live checkpoint가 tile `position`은 `Battlefield.position_of()`에서, `world_position`은 `RuntimeUnit.body`에서 만들면 같은 위치 의미가 두 source에서 나온다.
- 단일 `occupant` 제거만으로는 "unit 위치/continuous body state가 전투 위치 source of truth"라는 정책을 완전히 만족하지 못한다.

검증 방향:
- live checkpoint/deployment DTO의 tile `position`이 `RuntimeUnit.body`에서 파생되는지 테스트한다.
- 이동 후 tile projection, 사망 snapshot, 철수, projectile launch/impact, skill range preview가 `Battlefield.position_of()` 없이 동일한 사용자-visible behavior를 유지하는지 고정한다.
- `rg`로 `battlefield.position_of`, `battlefield.place`, `battlefield.remove`, `battlefield.units_at`, `unit_pos`가 gameplay code에서 제거됐는지 확인한다.

구현 검증:
- `rg -n "battlefield\\.(position_of|place|remove|units_at|occupant)|\\bBattlefield\\b|\\.in_bounds\\(|unit_pos" src tests -g '*.rs'`에서 gameplay/layout-owned unit position API 잔존 없음.
- `cargo check`
- `cargo check -p game_server`
- `cargo test battlefield --lib -- --test-threads=1`
- `cargo test static_obstacles_block_placement --lib -- --test-threads=1`
- `cargo test advance_basic_attack_projectile_does_not_recheck_tile_range_at_arrival --lib -- --test-threads=1`
- `cargo test instant_tile_area --lib -- --test-threads=1`
- `cargo test live_deployment_reconciles_defeated_player_unit_before_state_dto --lib -- --test-threads=1`

#### Battlefield layout / runtime lifecycle confirmed decisions

아래 결정은 `Battlefield static layout extraction` 구현 goal에서 함께 반영한다.

1. `Battlefield` rename and static-only ownership
   - `Battlefield`는 `BattlefieldLayout` 이름으로 rename하는 방향을 확정한다.
   - `BattlefieldLayout`은 정적 전장 layout만 소유한다.
   - 소유 범위: width, height, valid tiles, void tiles, static obstacles, valid/walkable tile query.
   - 비소유 범위: 유닛 위치, 배치 여부, 사망 여부, 철수 여부, tile occupant state.

2. Unity-facing tile `position` semantics
   - live checkpoint/deployment DTO의 `position` field는 유지한다.
   - `position`은 canonical 위치가 아니라 `RuntimeUnit.body.position.project_to_tile()`에서 계산한 derived projected tile이다.
   - `world_position`은 canonical continuous position인 `RuntimeUnit.body.position`이다.
   - Unity는 tile 표시와 range UI에 core가 내려준 `position`을 사용할 수 있지만, 이 값은 별도 저장된 tile position이 아니다.
   - Unity가 projection 규칙을 재구현하지 않도록 core가 projected tile을 계속 내려준다.

3. `BattleCore.units` meaning
   - `BattleCore.units`는 active unit list가 아니라 battle에 materialized 된 runtime entity registry다.
   - 유닛은 사망/철수 후에도 참조 안정성, event/debug/replay, delayed effect/projectile reference를 위해 `BattleCore.units`에 남는다.
   - 아직 배치되지 않은 직원은 `BattleCore.units`가 아니라 roster/live deployment 가능 목록에 존재한다.

4. `RuntimeUnitLifecycle`
   - `RuntimeUnit`에 `RuntimeUnitLifecycle` state를 추가한다.
   - variants: `Active`, `Withdrawn`, `Dead`.
   - active gameplay 참여 여부는 `RuntimeUnit.lifecycle`이 canonical source다.
   - movement, targeting, skill, 새 damage 후보, checkpoint `units`는 `lifecycle == Active`인 유닛만 대상으로 한다.
   - `stats.current_health`는 HP 수치 상태이고, `ActionState`는 행동/이동 상태다. 둘 다 lifecycle을 대체하지 않는다.

5. `Dead` lifecycle invariants
   - `RuntimeUnitLifecycle::Dead`는 active-exclusion canonical source다.
   - 사망 처리 시 `lifecycle = Dead`, `stats.current_health = 0`, `action_state = ActionState::Dead`를 함께 세팅한다.
   - HP 0과 `ActionState::Dead`는 lifecycle을 대체하는 source가 아니라 검증 invariant다.
   - `unit.is_dead()`는 장기적으로 `lifecycle == Dead` 기준으로 정리한다.

6. `Withdrawn` lifecycle semantics
   - `RuntimeUnitLifecycle::Withdrawn`은 active gameplay 대상이 아니다.
   - 철수 여부는 lifecycle이 canonical source다.
   - 철수 시 HP는 죽음처럼 0으로 만들지 않고 보존한다.
   - `ActionState`에 별도 `Withdrawn` variant를 추가하지 않는다.
   - 철수한 유닛의 `action_state`는 `Idle` 또는 구현상 안전한 neutral state로 정리한다.

7. Withdraw redeploy HP
   - 철수 후 재배치 시 HP는 철수 당시 HP에 최대 HP의 30%를 더한 값으로 시작한다.
   - 공식: `redeploy_hp = min(max_hp, withdrawn_hp + floor(max_hp * 0.30))`.
   - 철수 재배치는 사망 재배치와 다르며, 기존 HP 보존 + 일부 회복을 의미한다.

8. Death redeploy HP
   - 사망 후 재배치 시 HP는 기존 dead runtime unit의 HP를 복구하지 않는다.
   - 사망 재배치는 새 deployment instance를 생성한 뒤, 새 runtime unit의 최대 HP 기준 60% HP로 시작한다.
   - 공식: `death_redeploy_hp = max(1, floor(new_runtime_max_hp * 0.60))`, 이후 `new_runtime_max_hp`로 clamp한다.
   - 사망 재배치 HP 계산을 위해 live battle 중 persistent employee trauma/injury를 즉시 갱신하지 않는다.
   - persistent incapacitation consequence는 기존 post-battle resolution 경로가 담당한다.

9. Redeploy instance identity
   - 철수 후 재배치 시 기존 withdrawn `RuntimeUnit`을 재활성화하지 않는다.
   - 재배치 시 새 `RuntimeUnit` instance를 생성하고 새 `UnitInstanceId`를 부여한다.
   - `LiveBattleDeploymentState.deployed_units[employee_uuid]`는 새 `unit_instance_id`를 가리킨다.
   - old withdrawn unit은 `BattleCore.units`에 `Withdrawn` lifecycle로 남아 철수 전 launch/lock/schedule 된 projectile, delayed effect, event log, debug, replay reference를 위해 참조 가능하게 둔다.
   - battle 종료 시 `BattleCore` 전체가 폐기되므로 old withdrawn unit 누적은 battle lifetime 안에서만 유지한다.

10. Projectile / delayed effect interaction with withdrawn units
    - `Withdrawn` 유닛은 새 movement, targeting, skill, 새 damage 후보에서 제외한다.
    - 철수는 상대가 발사한 incoming hostile projectile을 피하는 의미가 강한 defensive/evasion action이다.
    - 철수 전에 이미 launch / lock 된 projectile이라도 철수하려는 target unit을 향해 날아오던 hostile projectile은 target unit이 `Withdrawn` 상태가 되면 damage/effect를 적용하지 않고 cancel/miss 처리한다.
    - 철수한 유닛을 향해 날아오던 hostile projectile은 old runtime HP와 redeploy HP lock을 갱신하지 않는다.
    - 철수 후 재배치 HP는 철수 시점에 확정된 `redeploy_hp = min(max_hp, withdrawn_hp + floor(max_hp * 0.30))` 값을 유지한다.
    - 공격자가 사망/철수해도 launch owner damage snapshot은 유지된다.
    - 대상이 `Withdrawn`이면 projectile/damage/effect resolve 대상에서 제외한다.
    - 대상이 `Dead`이면 중복 피해 방지를 위해 damage/effect resolve 대상에서 제외한다.
    - 철수 시 모든 projectile을 일괄 삭제하지는 않는다. projectile별 advance/impact 시점에서 withdrawn target을 cancel/miss로 확정한다.
    - 비투사체 delayed effect도 target이 `Withdrawn`이면 새 damage/buff/debuff/heal/stat modifier/resonance를 적용하지 않는다. 향후 예외가 필요하면 effect family별 정책으로 별도 명시한다.

11. Checkpoint / deployment snapshot visibility
    - official gameplay checkpoint `units`에는 `RuntimeUnitLifecycle::Active` 유닛만 포함한다.
    - `Dead` / `Withdrawn` 유닛은 `BattleCore.units`에 남아 참조 가능하지만 checkpoint `units`에는 포함하지 않는다.
    - 사망/철수 연출은 event log가 담당한다.
    - 재배치 쿨다운과 비용 UI는 `checkpoint.deployment.redeploying_units`가 담당한다.
    - `redeploying_units`에는 `employee_uuid`, `ready_at_ms`, `deploy_cost`가 포함된다.
    - 필요하면 debug 전용 snapshot에서 inactive runtime entity를 별도 노출할 수 있지만, 공식 gameplay checkpoint에는 넣지 않는다.

12. Withdraw cleanup
    - 철수 시 old unit의 active gameplay state를 즉시 정리한다.
    - 정리 대상: current target, movement goal, block engagement, pending cast, action locks 등 active 진행 상태.
    - 이미 launch/lock/schedule되어 snapshot/reference를 가진 projectile/delayed effect는 별도 runtime 객체로 계속 resolve 가능하다.
    - old withdrawn unit은 참조 안정성용 registry entity이지 active actor가 아니다.

13. Withdraw battle buff cleanup
    - 철수 시 battle runtime buff/debuff는 모두 제거한다.
    - 철수한 old unit은 더 이상 buff tick, DOT, stun, stat modifier, aura 대상이 아니다.
    - 재배치 새 `RuntimeUnit`은 old withdrawn unit의 battle runtime buff를 복사받지 않는다.
    - 이월되어야 하는 효과가 필요하면 battle buff를 유지/복사하지 않는다.
    - 이월 효과는 별도 `employee/run persistent modifier` 또는 `deployment carryover effect`로 명시 설계한다.
    - carryover 효과는 live RON/schema/UX 표시/밸런스 정책이 필요하므로 생기면 별도 정책으로 다룬다.

14. Redeploy skill/runtime action state reset
    - 철수 후 재배치 새 `RuntimeUnit`은 old withdrawn unit의 skill/action runtime state를 이어받지 않는다.
    - 이어받지 않는 상태: `pending_skill_cast`, `pending_cast`, `pending_cast_cause`, `current_target`, movement goal, action locks, `next_action_time`, skill 후딜/락, pending step/cast context.
    - resonance도 기존 배치 생성 로직의 초기값을 따른다.
    - 철수는 skill cooldown/resonance를 보존하는 시스템이 아니다.
    - 철수 후 재배치에서 이어받는 것은 확정된 HP 정책뿐이다.

15. Withdrawn unit final position
    - 철수 시 old withdrawn unit의 `body.position`을 전장 밖으로 옮기지 않는다.
    - old unit의 `body.position`을 제거하거나 `None` 처리하지 않는다.
    - old unit은 철수 순간의 마지막 world position을 보존한다.
    - derived tile position이 필요하면 마지막 `body.position.project_to_tile()`에서 계산한다.
    - active gameplay 제외는 위치 조작이 아니라 `RuntimeUnitLifecycle::Withdrawn` gate로 처리한다.
    - event/debug/replay/projectile reference는 old unit의 마지막 위치를 참조할 수 있다.

16. `UnitWithdrawn` event position
    - `UnitWithdrawn` event에 철수 순간의 `world_position`을 포함한다.
    - `UnitWithdrawn` event에 projected tile `position`도 포함한다.
    - `position`은 `world_position.project_to_tile()`에서 계산한 derived tile이다.
    - Unity는 checkpoint에서 withdrawn unit을 찾지 않아도 event만으로 철수 연출 위치를 알 수 있다.
    - debug/replay도 철수 위치를 event log만 보고 복원할 수 있다.

17. `UnitDied` event position
    - `UnitDied` event에 사망 순간의 `world_position`을 포함한다.
    - `UnitDied` event에 projected tile `position`도 포함한다.
    - `position`은 `world_position.project_to_tile()`에서 계산한 derived tile이다.
    - Unity는 checkpoint에서 dead unit을 찾지 않아도 event만으로 사망 연출 위치를 알 수 있다.
    - debug/replay도 사망 위치를 event log만 보고 복원할 수 있다.

18. Debug/admin/replay inactive unit visibility
    - official gameplay checkpoint `units`에는 `Active` 유닛만 포함한다.
    - `Withdrawn` / `Dead` 유닛은 checkpoint `units`에는 포함하지 않는다.
    - debug/admin/replay 전용 query에서는 inactive units도 조회 가능하게 한다.
    - debug/admin/replay query에는 lifecycle을 함께 포함한다.
    - 이 경로는 gameplay source나 Unity 기본 presentation 계약이 아니다.
    - Unity-facing 공식 전투 표시와 debug/admin/replay query는 명확히 분리한다.

19. Static obstacles and range/collision responsibility
    - `BattlefieldLayout`은 static obstacles를 소유한다.
    - static obstacles는 이동, pathfinding, collision, deployment blocking에 사용한다.
    - static obstacles는 기본적으로 range preview cell이나 skill affected tile에서 제외하지 않는다.
    - range preview와 skill affected tiles는 valid tile clipping을 기준으로 한다.
    - void/out-of-bounds/invalid tile은 제외한다.
    - obstacle 위/너머의 target 가능 여부는 skill target policy, `air_capable`, 실제 target validation이 판단한다.
    - 이 정책은 기존 `TileArea.affected_tiles` clipping, long line/piercing skill valid-tile clipping 정책과 일관된다.

20. Valid tile API naming
    - 기존 `Battlefield::in_bounds(pos)`는 `BattlefieldLayout::is_valid_tile(pos)`로 rename한다.
    - `is_valid_tile`은 rectangle bounds 안에 있고, authored `valid_tiles`가 있으면 그 목록에 포함되는 tile이라는 뜻이다.
    - raw rectangle-only bounds check와 혼동하지 않는다.
    - raw bounds check가 필요하면 `position_in_bounds` 같은 별도 helper를 사용한다.
    - range preview, skill affected tiles, whole-field valid-tile targeting은 `is_valid_tile`을 사용한다.
    - movement/path/collision은 필요에 따라 `is_valid_tile` + static obstacle query를 조합한다.

21. Walkable tile and void tile APIs
    - `BattlefieldLayout::is_walkable_tile(pos)` 이름과 의미는 유지한다.
    - `is_walkable_tile(pos)`는 `is_valid_tile(pos) && !is_static_obstacle(pos)`를 뜻한다.
    - movement, pathfinding, deployment blocking은 `is_walkable_tile` 또는 그에 준하는 valid + obstacle 조합을 사용한다.
    - `void_tiles()`는 정적 layout의 derived query로 유지한다.
    - `void_tiles()`는 source of truth가 아니라 width/height + valid_tiles에서 계산되는 projection이다.
    - Unity setup/debug/visualization에서 void tile 목록이 필요하면 이 derived query를 사용한다.

Remaining policy status:
- `Battlefield static layout extraction` 자체는 구현 완료.
- `RuntimeUnitLifecycle` 추가, active/dead/withdrawn participation gate, death invariant, withdraw active-state cleanup, withdraw buff cleanup, withdraw skill/action runtime reset, withdrawn unit final position 보존, official checkpoint Active-only 필터는 `core_runtime_unit_lifecycle`에서 구현 완료.
- Redeploy HP/new `UnitInstanceId`는 `core_redeploy_lifecycle_policy`에서 구현 완료.
- Inactive projectile/delayed-effect resolution은 `core_inactive_unit_effect_resolution`에서 구현 완료. 철수는 상대 hostile projectile에 대한 강한 회피 행동이며, 철수 대상 projectile은 advance/impact 시점에 cancel/miss/no-hit로 resolve한다. 철수 시 모든 projectile을 일괄 삭제하지 않는다.
- `UnitWithdrawn`/`UnitDied` position event contract, official checkpoint Active-only contract, debug/replay inactive unit visibility query는 `core_lifecycle_event_snapshot_contract`에서 구현 완료.
- 현재 이 항목에서 다음 subgoal 진행을 막는 미결 정책은 없다.
- 구현 중 Unity-facing DTO shape, external error code, save migration, 밸런스/UX 의미 변경이 새로 드러나면 해당 goal을 완료 산출물로 종료하고 사용자와 논의한다.

### Runtime lifecycle withdrawal cleanup event semantics

구현 상태: `core_followup_policy_implementation_master` / `core_runtime_unit_lifecycle`에서 구현 완료.

발견 위치: `core_followup_policy_implementation_master` / `core_runtime_unit_lifecycle`.

정책 결정 리포트: `docs/goals/core_runtime_unit_lifecycle/POLICY_DECISION_REPORT.md`

문제:
- `RuntimeUnitLifecycle::Withdrawn` 구현 시 active gameplay state와 battle runtime buff/debuff를 정리해야 한다.
- 현재 event log의 `BuffExpireReason`은 `Natural`, `Replaced`, `TargetDied`, `CasterDied`만 제공한다.
- withdrawal로 종료된 buff/cast를 death/interruption으로 기록하면 의미가 틀어지고, 조용히 제거하면 replay/debug/Unity-facing event contract가 runtime state 변화를 설명하지 못한다.

추천:
확정 정책:
- withdrawal-specific cleanup reason을 공식 event-log 계약에 추가한다.
- buff target이 withdrawn 된 경우 `BuffExpireReason::TargetWithdrawn`을 기록한다.
- buff caster가 withdrawn 된 경우 `BuffExpireReason::CasterWithdrawn`을 기록한다.
- active/pending cast cancellation은 `SkillCastInterrupted`를 재사용하지 않는다.
- withdrawal로 취소된 cast는 별도 typed event로 표현한다.
- 확정 형태: `BattleLogEvent::SkillCastCancelled { caster_instance_id, interrupted_skill_id, interrupted_cast_seq, reason: SkillCastCancelReason::Withdrawn }`.
- `SkillCastInterrupted`는 외부 interruption 의미로 유지하고, self-withdrawal cleanup 의미로 overload하지 않는다.
- 이 cleanup event들은 official event log 계약이며 validator와 Unity-facing DTO 테스트로 고정한다.

구현 결과:
- `BuffExpireReason::TargetWithdrawn` / `BuffExpireReason::CasterWithdrawn`을 추가했다.
- `SkillCastCancelReason::Withdrawn`과 `BattleLogEvent::SkillCastCancelled { caster_instance_id, interrupted_skill_id, interrupted_cast_seq, reason }`을 추가했다.
- 철수 시 target/caster가 철수 유닛인 battle runtime buff를 제거하고 withdrawal-specific `BuffExpired` event를 기록한다.
- 철수 시 pending cast는 `AutoCastStart`/`ManualCastStart` seq를, active cast는 `AbilityCast` seq를 `interrupted_cast_seq`로 기록하며 `SkillCastCancelled(reason=Withdrawn)`을 남긴다.
- validator는 `SkillCastCancelled` reference/spawn/death/autocast 관계를 검증하고 active cast cancellation의 `AbilityCast` parent seq를 허용한다.
- `SkillCastInterrupted`는 외부 interruption 의미로 유지했다.

### Player state snapshot DTO drift risk

구현 상태: `core_followup_policy_implementation_master` / `core_snapshot_error_contract_cleanup`에서 구현 완료.

운영 방향:
- `serde_json::Value` mutation으로 돌아가지 않는다.
- `RunSnapshotDto` field 추가 시 server snapshot에서 누락되지 않는 구조를 설계한다.
- 가능한 방향은 core snapshot DTO를 포함/flatten하고 selected event만 typed wrapper로 대체하는 구조, 또는 core/server shared DTO composition이다.
- 외부 JSON shape를 바꾸기 위한 compatibility layer는 만들지 않는다.

구현 결과:
- `RunSnapshotDto`는 selected-event payload 타입만 바꿀 수 있는 generic DTO가 되었다.
- server `PlayerStateSnapshotDto`는 `RunSnapshotDto<PlayerSelectedEventSnapshotDto>`를 `#[serde(flatten)]`으로 포함한다.
- server는 root run snapshot field list를 수동 serialize하지 않고, `selected_event` transport enrichment만 typed payload로 소유한다.
- `serde_json::Value` mutation이나 compatibility wrapper는 추가하지 않았다.
- `cargo test -p game_server player_state_snapshot_preserves_core_snapshot_root_fields -- --test-threads=1`로 core/server root field drift를 검증한다.

근거:
- 현재 구조는 문자열 key mutation은 제거했지만, core snapshot field list와 server wrapper field list가 함께 갱신되어야 한다.
- 이는 source-of-truth 중복은 아니지만 DTO shape maintenance drift 위험이다.

정책 결정 필요 여부:
- 현재는 새 정책 결정 없이 follow-up refactor로 처리 가능하다.
- Unity-facing snapshot field shape 변경이 필요해지면 별도 정책 논의가 필요하다.

### PositionOccupied error meaning cleanup

구현 상태: `core_followup_policy_implementation_master` / `core_snapshot_error_contract_cleanup`에서 구현 완료.

운영 방향:
- 유닛 점유로 인한 tile 단일 소유 의미를 되살리지 않는다.
- static obstacle 때문에 placement가 불가능한 경우는 `TileBlocked`, `StaticObstacleBlocked`, `BlockedByObstacle` 같은 명시적 의미로 분리/rename하는 방향을 검토한다.
- 내부 enum rename과 Unity-facing error code rename은 같은 goal에서 함께 감사한다.

구현 결과:
- `GameError::StaticObstacleBlocked`를 추가했다.
- battlefield static obstacle placement와 scenario static obstacle/spawn overlap은 `StaticObstacleBlocked`를 반환한다.
- core snapshot/server error mapping은 `StaticObstacleBlocked`를 `static_obstacle_blocked`로 노출한다.
- `GameError::PositionOccupied` / `position_occupied`는 실제 occupied board slot 경로에 유지한다.
- legacy alias나 compatibility fallback은 추가하지 않았다.

근거:
- overlap 허용 정책 이후 `PositionOccupied`라는 이름은 "다른 유닛이 있어 배치 불가"라는 오래된 의미를 암시한다.
- 현재 실제 사용 의미는 static obstacle 또는 blocked tile에 더 가깝다.

정책 결정 필요 여부:
- 내부 명명 정리는 새 정책 결정 없이 진행 가능하다.
- Unity-facing error code를 `position_occupied`에서 다른 이름으로 바꾸는 것은 외부 계약 변경이므로 구현 goal에서 영향 범위를 확인하고 필요하면 사용자와 논의한다.
