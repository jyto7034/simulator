# Core Policy Implementation Master Goal

이 master goal은 2026-06에 확정된 core 정책들을 독립 goal 단위로 순서대로 수행하고, 각 goal 완료 결과를 통합 검증하는 상위 진행 문서다.

개별 구현 상세는 아래 goal 문서들이 source of truth다.

1. `docs/damage_feedback_dto_goal.md`
2. `docs/combat_preview_threat_warnings_goal.md`
3. `docs/abnormality_attempt_retreat_goal.md`
4. `docs/consumable_modifier_reentry_duration_goal.md`
5. `docs/unit_overlap_blocking_goal.md`
6. `docs/movement_backend_policy_refactor_goal.md`
7. `docs/rapier_backend_quality_refactor_goal.md`
8. `docs/airborne_enemy_mobility_goal.md`
9. `docs/weapon_archetype_targeting_goal.md`
10. `docs/skill_fragment_compatibility_goal.md`
11. `docs/ad_ap_balance_validation_goal.md`

관련 정책 요약은 `docs/core_policy_decisions_2026_06.md`를 따른다.

Unity-facing 계약/구현 문서의 canonical 위치는 `F:\unity projects\ark\docs`다. Linux/WSL 경로로는 `/mnt/f/unity projects/ark/docs`다. 이 저장소 안의 `docs/unity_core_contract.md`, `docs/unity_client_implementation_goal.md`는 stale copy일 수 있으므로, Unity 계약을 확인하거나 갱신할 때는 반드시 외부 canonical 문서를 사용한다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

master goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/core_policy_implementation_master/PLAN.md
docs/goals/core_policy_implementation_master/EXPERIMENTS.md
docs/goals/core_policy_implementation_master/EXPERIMENT_NOTES.md
```

각 하위 goal을 시작할 때는 해당 goal 문서가 요구하는 작업 기억장치도 별도로 만든다.

예:

```text
docs/goals/damage_feedback_dto/PLAN.md
docs/goals/damage_feedback_dto/EXPERIMENTS.md
docs/goals/damage_feedback_dto/EXPERIMENT_NOTES.md
```

master 작업 기억장치의 역할:

- `PLAN.md`: 전체 goal 순서, 현재 진행 중인 하위 goal, 완료/중단 상태, 통합 체크리스트를 기록한다.
- `EXPERIMENTS.md`: 각 하위 goal에서 수행한 주요 실험/테스트 결과 요약과 통합 검증 결과를 기록한다.
- `EXPERIMENT_NOTES.md`: 하위 goal 사이에서 발견한 의존성, 충돌, 정책 질문, 후속 goal 후보를 기록한다.

하위 goal의 작업 기억장치는 해당 goal 내부 구현 판단과 실험 기록을 남기는 데 사용한다.

## Global Execution Contract

- source of truth 우선순위는 실제 runtime code, live RON/data, Unity-facing snapshot/command 계약, 최신 정책 문서 순서로 본다.
- 문서와 코드가 충돌하면 코드를 먼저 읽고, 최신 정책과 코드의 차이를 기록한 뒤 수정 방향을 정한다.
- compatibility layer, fallback path, dual schema는 기본적으로 만들지 않는다. 정말 필요하면 이유, 제거 예정 조건, 테스트 범위를 `PLAN.md`에 기록하고 사용자 확인을 받는다.
- 테스트는 내부 구현 모양보다 사용자-visible behavior, Unity-facing DTO, data validation, live RON loading, 실제 gameplay flow를 고정한다.
- 새 정책과 충돌하는 테스트는 기대값만 바꾸지 않는다. 그 테스트가 무엇을 보호하던 것인지 확인한 뒤 삭제하거나 최신 정책 테스트로 교체한다.
- ignored test로 레거시를 보존하지 않는다.
- `git restore`, `git reset` 등 git file modifier 명령은 수행하지 않는다.
- git의 과거 내역 코드를 복사하여 현재 파일에 `cp`하지 않는다. 만약 꼭 필요하다면 사용자의 허락을 무조건 먼저 받는다.
- 파일을 과거 상태로 돌려야 한다고 판단한 경우, 이유를 설명하고 사용자의 허락을 무조건 먼저 구한다.
- 하위 goal 범위를 벗어난 개선안은 바로 구현하지 말고 master `EXPERIMENT_NOTES.md`에 후속 후보로 기록한다. 현재 하위 goal 완료에 필수이거나 blast radius를 줄이는 경우에만 적용한다.
- Unity-facing DTO shape, live RON schema, 저장 데이터 migration, UX 의미 변화, 밸런스 기준, 기존 콘텐츠 삭제/대체, 실패/보상/소비 시점 변화처럼 사용자 결정이 필요한 정책이 발견되면 임의로 결정하지 않고 master goal을 종료하고 질문 목록을 보고한다.
- 작은 변경 단위마다 빠른 focused test 또는 `cargo check`를 먼저 돌리고, 각 하위 goal 완료 시 해당 goal의 검증 명령을 수행한다.
- 여러 하위 goal을 완료한 뒤에는 통합 검증을 수행한다.
- 테스트 실패는 해당 하위 goal `EXPERIMENTS.md`와 master `EXPERIMENTS.md`에 실패 원인, 수정 내용, 재검증 결과를 기록한다.
- 각 하위 goal 완료 시 변경 요약, 제거한 레거시, 새로 고정한 계약, 갱신한 테스트, 남은 위험, 실행한 검증 명령을 보고한다.

## Sequencing Rationale

순서는 작은 DTO/계약 변경에서 시작해, lifecycle, runtime, data schema, validation으로 이동한다.

초기 두 goal은 비교적 독립적이고 Unity-facing 계약을 먼저 안정화한다.

이후 이상현상 attempt lifecycle을 구현해야 consumable modifier duration과 false rumor `Disproved` 처리를 제대로 연결할 수 있다.

unit overlap/blocking은 BattleCore runtime 변화가 크므로 lifecycle 정리 이후 독립적으로 수행한다.

movement backend policy refactor는 Rapier/backend physics의 책임을 지상 static obstacle 보정 helper로 축소하고, 유닛별 movement/collision policy를 typed input으로 흐르게 하는 작업이다.

rapier backend quality refactor는 이 정책 기반 위에서 Rapier 내부 구조, 테스트 표면, 진단 가능성을 장기적으로 정리하는 작업이다. airborne enemy mobility 전에 수행하면 공중 지형 무시 구현이 backend 레거시와 섞이지 않는다.

airborne enemy mobility는 이 기반 이후에 구현해야 terrain immunity를 예외 패치로 숨기지 않는다.

airborne enemy mobility는 지형 무시 이동, 저지 불가, 대공 타겟팅의 source of truth를 먼저 세우는 작업이다. weapon archetype/targeting의 `AirFirst`와 `air_capable`은 이 goal 이후에만 안정적으로 구현할 수 있다.

weapon archetype/targeting은 equipment schema와 battle targeting을 바꾸는 큰 작업이며, skill fragment compatibility는 이 작업 이후에만 의미가 있다.

AD/AP validation은 앞선 schema와 warning 계약이 안정된 뒤 최소 금지 패턴 중심으로 수행한다.

## Ordered Goal Chain

### 1. Damage Feedback DTO

문서: `docs/damage_feedback_dto_goal.md`

목적:

- `HpChanged.feedback_tags`를 추가한다.
- 피해 결과 표시 태그를 core가 계산한다.
- Unity가 피해 피드백을 재계산하지 않게 한다.

선행 조건:

- 없음.

완료 후 확인:

- `HpChanged` event shape가 최신 Unity 계약과 일치한다.
- `critical`, `mitigated`, `fixed_damage`, `immune` focused tests가 있다.
- 이후 goal에서 damage event를 참조할 수 있다.

### 2. Combat Preview Threat Warnings

문서: `docs/combat_preview_threat_warnings_goal.md`

목적:

- `CombatPreview.threat_warnings` typed DTO를 추가한다.
- warning status/source를 표현한다.
- seed 기반 rumor warning의 기초를 만든다.

선행 조건:

- 없음. 단, 1번과 충돌하지 않는 독립 DTO 작업이다.

완료 후 확인:

- `threat_warning_tags` 단순 배열이 공식 계약으로 남지 않는다.
- `threat_warnings`가 NodePreview/CombatPreview snapshot에 포함된다.
- false rumor `Disproved` 처리는 별도 false-rumor lifecycle goal에서 이어받을 수 있다.

### 3. Abnormality Attempt / Retreat / Re-entry

문서: `docs/abnormality_attempt_retreat_goal.md`

목적:

- 전투 노드를 이상현상 3회 시도 구조로 바꾼다.
- retreat 후 Safezone/NodeConfirm 복귀와 재진입을 구현한다.
- 실패/성공/3회 소진 시 노드 소비 정책을 구현한다.

선행 조건:

- 2번이 완료되어 있으면 false rumor `Disproved` 처리를 연결하기 쉽다.
- 2번이 정책 질문으로 중단되었다면 3번도 시작하지 않고 사용자와 의논한다.

완료 후 확인:

- 기존 “retreat consumes node” 레거시 테스트가 최신 정책으로 교체된다.
- `BattleEnd` 전 retreat 가능, `BattleEnd` 후 retreat 불가가 테스트된다.
- remaining attempts가 snapshot/allowed action에서 표현된다.
- threat warning false rumor의 `Disproved` 처리가 가능하거나, 최소 hook과 후속 TODO가 명확히 기록된다.

### 4. Consumable Modifier Re-entry Duration

문서: `docs/consumable_modifier_reentry_duration_goal.md`

목적:

- 소비 아이템은 `use_consumable_item` 성공 시 즉시 소비된다.
- retreat/re-entry 동안 modifier가 유지된다.
- duration은 진입 시도가 아니라 전투/보스 노드 해결 시 감소한다.

선행 조건:

- 3번 완료 권장.

완료 후 확인:

- node cancel, retreat, re-entry, overwrite에서 환불 없음이 테스트된다.
- 같은 이상현상 재진입 동안 modifier가 유지된다.
- 성공/실패/3회 소진 시 duration 감소가 테스트된다.

### 5. Unit Overlap and Blocking Runtime

문서: `docs/unit_overlap_blocking_goal.md`

목적:

- 유닛 간 물리 충돌을 길막/저지 source of truth에서 제거한다.
- 적/전투 중 유닛 좌표 겹침을 허용한다.
- block state 기반 저지와 route progress 우선순위를 구현한다.

선행 조건:

- 3번 완료 권장. battle lifecycle이 안정된 뒤 runtime 테스트를 정리하기 쉽다.

완료 후 확인:

- 아군 배치 겹침 금지는 유지된다.
- 적끼리/적과 아군/저지 중 유닛 좌표 겹침이 가능하다.
- block capacity 초과 enemy가 통과한다.
- route progress/spawn order/unit id tie-break가 테스트된다.

### 6. Movement Backend Policy Refactor

문서: `docs/movement_backend_policy_refactor_goal.md`

목적:

- Rapier/backend physics가 blocking, targetability, airborne terrain immunity의 source of truth가 아니도록 역할을 정리한다.
- movement input에 유닛별 movement/collision policy를 명시한다.
- Rapier를 ground static obstacle correction helper로 제한하고, airborne terrain immunity를 구현할 기반을 만든다.

선행 조건:

- 5번 완료 필요. 유닛 겹침/저지 source-of-truth 정책이 정리된 뒤 backend 책임을 재정의한다.

완료 후 확인:

- Ground/Airborne movement policy를 typed input/runtime path로 표현할 수 있다.
- unit colliders는 movement/blocking source of truth가 아니다.
- Rapier static obstacle correction은 ground policy에만 적용 가능하다.
- Direct/Rapier backend가 policy-visible movement semantics에서 충돌하지 않는다.

### 7. Rapier Backend Quality Refactor

문서: `docs/rapier_backend_quality_refactor_goal.md`

목적:

- Rapier backend 내부 책임을 ground static obstacle correction helper로 명확히 제한한다.
- Direct/Rapier backend의 policy-visible behavior를 tests로 고정한다.
- stale local avoidance, 중복 board bounds path, unit collider confusion을 정리한다.

선행 조건:

- 5번 완료 필요. unit overlap/blocking source-of-truth 정책이 먼저 정리되어야 한다.
- 6번 완료 필요. movement policy carrier와 board clamp source of truth가 먼저 정리되어야 한다.

완료 후 확인:

- Rapier가 blocking, targetability, deploy occupancy, route progress, airborne terrain immunity의 source of truth가 아니다.
- unit collider exclusion과 board clamp 역할이 tests/docs로 고정된다.
- route-blocked ground movement가 암묵 우회로 숨겨지지 않는다.

### 8. Airborne Enemy Mobility

문서: `docs/airborne_enemy_mobility_goal.md`

목적:

- enemy `mobility_kind: Ground | Airborne`을 도입한다.
- `Airborne` enemy를 지형지물/장애물/walkable tile 영향을 받지 않는 별도 이동 계층으로 구현한다.
- `Airborne` enemy의 저지 불가, 보호 목표 우선 공격, 대공 필요 단일 타겟팅, 광역 피격 정책을 core source of truth로 고정한다.

선행 조건:

- 5번 완료 필요. unit overlap/blocking 정책이 정리된 뒤 공중 저지 불가를 얹는다.
- 6번 완료 필요. airborne terrain immunity는 movement backend policy를 통해 구현한다.
- 7번 완료 권장. Rapier backend 내부 부채를 먼저 줄이면 공중 이동 구현이 backend 레거시와 섞이지 않는다.

완료 후 확인:

- 공중 적은 route를 가지되 route 끝을 넘지 않는다.
- 공중 적은 blocked state에 들어가지 않는다.
- 공중 적은 사거리 안 보호 목표를 최우선 공격하고, 없으면 가장 가까운 player combat unit을 공격한다.
- 대공 불가능 단일 공격/스킬은 공중 적을 후보로 삼지 않고, 광역 스킬은 공중 적을 피격시킨다.
- `air_enemy_possible` warning이 core preview에서 생성된다.

### 9. Weapon Archetype and Targeting Profile

문서: `docs/weapon_archetype_targeting_goal.md`

목적:

- 직원 고정 직업 대신 장착 무기가 전투 역할을 결정한다.
- weapon profile, archetype, targeting profile을 data/runtime에 도입한다.
- basic attack damage type과 target selection이 weapon profile을 따른다.

선행 조건:

- 5번 완료 권장. 근거리 blocked target 우선 정책이 block state와 연결된다.
- 8번 완료 필요. `AirFirst`와 `air_capable`은 `Airborne` mobility source of truth를 사용한다.

완료 후 확인:

- 초기 7종 archetype이 schema/validation/test에 반영된다.
- `DefaultForward`, `AirFirst`, `LowDefenseFirst`, `LowMagicResistFirst`, `SplashClusterFirst`가 구현 또는 명확한 단계적 구현 상태를 가진다.
- Unity snapshot에 current weapon combat profile이 노출된다.

### 10. Skill Fragment Compatibility

문서: `docs/skill_fragment_compatibility_goal.md`

목적:

- 스킬 파편 장착 조건을 current weapon combat profile 기준으로 검증한다.
- Unity가 호환/비호환 상태와 실패 사유를 표시할 수 있게 한다.

선행 조건:

- 9번 완료 필요.

완료 후 확인:

- fragment requirement schema가 구현된다.
- equip validation이 core source of truth가 된다.
- 기존 자유 장착 정책 테스트가 최신 정책으로 교체된다.
- live RON이 새 validation을 통과한다.

### 11. AD/AP Balance Validation

문서: `docs/ad_ap_balance_validation_goal.md`

목적:

- AD/AP가 하드 게이트가 되지 않도록 명확한 금지 패턴 validation을 추가한다.
- threat warning과 실제 enemy composition의 큰 불일치를 잡는다.

선행 조건:

- 2번 완료 필요.
- 9번 완료 권장.

완료 후 확인:

- 일반/정예 enemy의 permanent type immunity 금지 validation이 있다.
- boss 예외는 명시 metadata 없이 허용되지 않는다.
- high armor/high magic resist warning 누락 validation 또는 audit가 있다.
- live RON이 최신 validation을 통과한다.

## Integration Checkpoints

### Checkpoint A: DTO Foundation

1번과 2번 완료 후 수행한다.

검증 후보:

```text
cargo test -p game_core damage -- --nocapture
cargo test -p game_core combat_preview -- --nocapture
cargo test -p game_core node_preview -- --nocapture
cargo check -p game_core
```

확인:

- `HpChanged.feedback_tags`와 `CombatPreview.threat_warnings`가 동시에 serialization된다.
- Unity 계약 문서가 두 DTO와 일치한다.

### Checkpoint B: Re-entry Lifecycle

3번과 4번 완료 후 수행한다.

검증 후보:

```text
cargo test -p game_core retreat -- --nocapture
cargo test -p game_core consumable -- --nocapture
cargo test -p game_core node_flow -- --nocapture
cargo test -p game_core live_defense -- --nocapture
cargo check -p game_core
cargo check -p game_server
```

확인:

- retreat/re-entry와 consumable modifier duration이 충돌하지 않는다.
- 3회 attempt 소진과 node consumption이 일관된다.

### Checkpoint C: Battle Runtime

5번 완료 후 수행한다.

검증 후보:

```text
cargo test -p game_core blocking -- --nocapture
cargo test -p game_core movement -- --nocapture
cargo test -p game_core live_defense -- --nocapture
cargo check -p game_core
```

확인:

- unit overlap 정책이 blocking/targeting/death validation과 충돌하지 않는다.

### Checkpoint D: Movement Backend and Airborne Runtime

6번, 7번, 8번 완료 후 수행한다.

검증 후보:

```text
cargo test -p game_core movement -- --nocapture
cargo test -p game_core airborne -- --nocapture
cargo test -p game_core blocking -- --nocapture
cargo test -p game_core live_defense -- --nocapture
cargo check -p game_core
cargo check -p game_server
```

확인:

- Rapier/backend physics가 blocking, targetability, airborne terrain immunity의 source of truth가 아니다.
- airborne mobility가 movement backend policy를 통해 terrain/static obstacle correction을 우회한다.
- Rapier backend 내부 refactor가 Direct/Rapier user-visible movement semantics를 갈라놓지 않는다.

### Checkpoint E: Equipment and Fragment Build System

9번과 10번 완료 후 수행한다.

검증 후보:

```text
cargo test -p game_core equipment -- --nocapture
cargo test -p game_core targeting -- --nocapture
cargo test -p game_core skill_fragment -- --nocapture
cargo test -p game_core ron_loading -- --nocapture
cargo check -p game_core
cargo check -p game_server
```

확인:

- airborne mobility와 weapon targeting profile이 같은 air-capable source of truth를 사용한다.
- Unity snapshot에서 공중 표시와 weapon combat profile 표시가 가능하다.
- weapon profile과 fragment compatibility가 같은 source of truth를 사용한다.
- Unity snapshot에서 loadout/compatibility 표시가 가능하다.

### Checkpoint F: Final Data Validation

11번 완료 후 수행한다.

검증 후보:

```text
cargo test -p game_core ron_loading -- --nocapture
cargo test -p game_core data -- --nocapture
cargo test -p game_core combat_preview -- --nocapture
cargo test -p game_core -- --nocapture
cargo check -p game_core
cargo check -p game_server
```

server mapping을 변경했다면:

```text
cargo test -p game_server -- --nocapture
```

## Master Completion Conditions

master goal은 아래 조건을 모두 만족해야 완료된다.

1. 11개 하위 goal이 완료되었거나, 사용자와 의논해 명시적으로 범위에서 제외되었다.
2. 각 하위 goal의 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`가 작성되어 있다.
3. 각 하위 goal의 완료 요약이 master `EXPERIMENTS.md` 또는 `PLAN.md`에 기록되어 있다.
4. 최신 정책과 실제 구현이 `docs/core_policy_decisions_2026_06.md`, `docs/game_rulebook.md`, `docs/skill_target_contract.md`, canonical `F:\unity projects\ark\docs\unity_core_contract.md`, canonical `F:\unity projects\ark\docs\unity_client_implementation_goal.md`에서 충돌하지 않는다.
5. 레거시 정책을 고정하는 ignored test가 남아 있지 않다.
6. live RON/data loading이 최신 validation을 통과한다.
7. Unity-facing DTO 변경이 문서와 tests에 반영되어 있다.
8. final integration 검증 명령을 실행했고 결과를 보고했다.
9. 남은 위험, 후속 goal 후보, 사용자에게 확인받은 정책 결정이 기록되어 있다.
10. 사용자와 의논하여 정해야 할 정책이 있을 경우 master goal을 종료한다.

## Master Stop Conditions

다음 경우 전체 진행을 임의로 계속하지 말고 master goal을 종료하고 질문 목록을 보고한다.

- 하위 goal 중 하나가 사용자 정책 결정을 요구하는 stop condition에 도달했다.
- 하위 goal 사이에서 DTO shape, RON schema, 저장 상태, UX 의미가 서로 충돌한다.
- 기존 live data를 대량 삭제/대체해야 하는데 정책 확인이 필요하다.
- migration 또는 backward compatibility를 정말로 유지해야 할 외부 저장 데이터가 발견된다.
- 단일 하위 goal이 예상보다 훨씬 커져 독립 goal 재분할이 필요하다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 master goal을 종료한다.
