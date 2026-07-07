# Core Runtime Contract

이 문서는 core runtime 구현자가 지켜야 하는 source-of-truth, 결정론, 관측/전송 경계 계약을 정리한다.

`docs/game_rulebook.md`는 플레이 규칙을 설명하고, 이 문서는 그 규칙을 runtime code, event log, snapshot, Unity-facing DTO로 구현할 때의 금지/허용 경계를 설명한다. 문서보다 실제 runtime code와 live RON/data가 우선이며, 문서와 코드가 충돌하면 코드를 먼저 확인하고 정책 판단이 필요한 부분은 사용자와 의논한다.

## Determinism And RNG

전투 판정 RNG는 event log 관측 순서와 분리한다.

- 치명타, proc, 명중, 회피, 특수 발동처럼 gameplay 결과를 바꾸는 모든 랜덤 판정은 `event_log_seq`나 로그 기록 순서를 seed 재료로 사용하지 않는다.
- 새 랜덤 판정을 추가할 때는 공격 원천, caster/target, projectile/cast/step/hit identity, battle time 같은 안정적인 gameplay identity에서 seed를 파생한다.
- `BattleEventLog`는 결과를 기록하고 Unity presentation에 전달하는 관측 채널이며, 판정 source of truth가 아니다.
- 로그 이벤트를 추가, 삭제, 분할, 병합하는 작업이 이미 존재하는 전투 판정 결과를 바꾸면 안 된다.
- proc 성공 횟수, cooldown 상태, max trigger count 같은 runtime state는 발동 제한과 디버그에 사용할 수 있지만, proc roll identity의 주재료로 사용하지 않는다.

맵 생성, 보상 후보 생성처럼 생성 시점에 결과가 run state로 고정되는 절차적 생성은 seeded `StdRng` 순차 draw를 사용할 수 있다. 단, 생성 결과를 저장하지 않고 seed만으로 재생성하는 구조를 도입할 때는 별도 계약과 재현성 테스트가 필요하다.

## Live RON Authoring

Live RON은 gameplay source data다. 새/core live schema는 가능한 한 authoring typo를 deserialize 단계에서 잡아야 하며, gameplay 의미가 있는 필드는 조용히 무시하거나 legacy alias로 받아들이지 않는다.

- Core gameplay schema는 `#[serde(deny_unknown_fields)]`를 기본으로 사용한다.
- Unknown field, stale field, removed alias는 compatibility layer 없이 reject한다.
- Gameplay 의미가 바뀌는 필드는 명시 authoring을 우선한다.
- 단순 표시, optional presentation hint, harmless authoring convenience는 명시적 `serde(default)`를 둘 수 있다.
- `serde(default)`를 둔 gameplay field는 코드 이름, validation, 테스트가 기본값의 의도를 설명해야 한다.
- Live-authored `basic_attack` data does not rely on `BasicAttackDef` runtime defaults. Abnormality and corroded employee live RON must pass through authored input validation where combat-meaningful fields are explicit: `range_units`, `range_policy`, `defense_tile_range`, `damage_type`, `targeting_profile`, `air_capable`, `range_role`, `interval_ms`, `windup_ms`, `ranged_reposition_ms`, and `delivery`.
- `defense_tile_range: None` is still an explicit authored value. `range_policy: Pattern` may use a preset/resolution layer to fill the runtime tile pattern, but the authoring field itself must be present.
- Enum casing은 도메인 내부에서 일관되어야 한다. 별도 `rename_all`이 없는 live RON enum은 Rust variant 이름을 그대로 사용하고, `snake_case`로 선언된 enum은 RON도 snake_case를 사용한다.
- String id는 기존 live data convention을 따른다: item/profile/skill/event/preset id는 lower snake_case, Lobotomy-style abnormality id는 catalog id 형식을 유지한다.
- Live RON loading과 focused schema tests는 gameplay 시작 전 authoring mistake를 잡는 gate다.

현재 strictness가 고정된 핵심 도메인:

- PvE encounter/wave authoring은 unknown top-level/nested fields를 reject한다.
- Event scene/choice/effect authoring은 unknown fields를 reject한다.
- Corroded employee profile/range preset authoring은 unknown fields를 reject한다.
- Abnormality and corroded employee `basic_attack` authoring rejects missing combat fields instead of falling back to runtime `BasicAttackDef::default()`.
- Shop item raw schema, shop database, shop pool authoring은 unknown/stale stock fields를 reject한다.
- Run policy, skill raw schema, reward equipment pools, boss omen chains, corroded wave presets, map generation policy/node definitions are also strict by domain-owned validation/tests.

## Battle Event Log Boundary

`BattleEventLog`는 live `DefenseRoute` 전투의 append-only 관측 로그다. 전투 전체를 미리 산출하는 replay source가 아니며, gameplay 판정을 되돌려 계산하는 입력도 아니다.

- `battle_update.events_delta`는 아직 클라이언트가 presentation event log에 적용하지 않은 event log entry 구간이다.
- 전투 중 정확한 연출 타이밍 source는 `battle_update.events_delta`다.
- 전투 중 현재 상태/복구 source는 `battle_update.checkpoint`다.
- Unity는 event로 현재 상태를 계산하지 않고, checkpoint로 연출 타이밍을 만들지 않는다.
- `checkpoint.at_seq`는 그 seq까지의 event를 presentation timeline에서 처리한 뒤 reconcile할 권위 상태를 뜻한다.
- seq가 부여된 live presentation event는 이후 수정하지 않는다.

## Battle Setup And Movement Presentation

전투 시작 scene construction source는 `battle_setup_snapshot`이다. 전투 중 이동 연출 source는 checkpoint position이 아니라 movement event다.

- 유닛 이동 연출은 `MovementSegmentStarted`/`MovementStopped` event를 기준으로 한다.
- 실제 위치 변화가 있는 movement tick은 Unity-facing event로 노출된다.
- `checkpoint.units[*].world_position`은 이동 연출 source가 아니라 event 처리 후 보정할 reconcile target이다.
- 유닛 철수/사망 연출 위치는 `UnitWithdrawn`/`UnitDied` event의 `world_position`과 projected tile `position`을 기준으로 한다.
- 철수/사망한 유닛은 official checkpoint `units`에서 빠질 수 있으므로, Unity는 checkpoint에서 inactive unit을 찾아 연출 위치를 추론하지 않는다.
- `battle_update.checkpoint.units`는 official gameplay presentation/reconcile source이며 Active unit만 포함한다.
- inactive runtime unit 조회가 필요하면 debug/admin/replay 전용 query를 사용하고, 해당 query의 결과를 gameplay 표시 source로 사용하지 않는다.

## Battle Record And Result Sources

완료된 전투의 event log는 run-local 환상체 도감/관찰용 전투 기록으로 저장한다.

- 현재 `battle_records`는 모든 개별 전투 timeline archive가 아니라 `abnormality_uuid`별 대표 기록이다.
- 같은 환상체와 반복 전투해도 하나의 대표 기록만 유지한다.
- debug JSON export는 `abnormality_uuid` 필드와 `target/battle_records/run_<seed>/<abnormality_uuid>.json` 경로를 사용한다.
- 개별 전투 replay archive가 필요해지면 `battle_uuid` keyed 저장소를 별도 계약으로 만든다.
- 전투 결과 화면의 1차 통계 source는 core가 `BattleEventLog`와 `ParticipantBattleResult`에서 전투 종료 시 산출한 `selected_event.result_stats`다.
- Unity는 MVP, 누적 피해량, 처치 수, 받은 피해량, 기본 공격/스킬 사용 횟수, 배치/철수 횟수, 배치 시간, 최종 HP/생존/전투불능 여부를 raw event log에서 재계산하지 않는다.
- 압축 event log는 전투 기록/상세 로그/디버그용이다.
- `selected_event.bonus_objectives`는 PvE encounter가 명시한 추가 연구 목표의 결과 표시 source다.
- Unity는 bonus objective를 달성/미달성 표시와 presentation hint에 사용하되, 연구도 증가나 보상 지급을 로컬에서 재계산하지 않는다.
- 실제 연구 진행도는 combat result 완료 처리와 뒤따르는 `abnormality_research` snapshot이 source of truth다.

## Battle Actor Identity And HUD

전투 유닛 HUD 표시 정책과 actor identity는 core가 내려주는 DTO가 source of truth다.

- Unity는 `checkpoint.units[*].hud.bar_mode`와 `hud.threat_class`를 읽어 HP bar 또는 HP+공명 bar를 그린다.
- Unity는 owner/role/base_uuid/encounter를 조합해 HUD 정책을 추론하지 않는다.
- live actor 정체성은 `UnitSpawned.unit_source`와 `checkpoint.units[*].unit_source`가 제공한다.
- Unity는 선택 패널, actor visual, 디버그 표시, 복구 시 `unit_source`를 사용하고, wave 순서나 `base_uuid` 역조회로 적 종류를 추론하지 않는다.
- `unit_source`는 actor identity source이고, `hud.bar_mode`는 HUD bar source다.
- 침식 직원/환상체/방어 오브젝트 여부를 보고 Unity가 HUD bar 정책을 다시 계산하지 않는다.
- Battle actor HUD는 유닛 아래쪽 world-space anchor에 표시한다.
- `hud.threat_class`가 높을수록 HUD 크기와 bar 두께가 커지며, HP/공명 색상은 threat class로 바꾸지 않는다.
- Unity HUD는 카메라 빌보드 방식으로 정렬하되 화면 기준으로 반듯하게 보이도록 카메라 up vector를 사용한다.
- 적군 공격 범위는 플레이어 조작 preview가 아니므로 기본 DTO로 매번 내려보내지 않는다.
- 보스 경고, 광역 스킬 telegraph, debug처럼 시각적 경고가 필요한 경우에만 core가 실제 피해/효과가 적용될 타일을 별도 event/DTO로 내려보낸다.

## Range Preview Boundary

기본 공격/스킬 범위 overlay는 core가 계산한 `range_previews` 최종 cell이 source of truth다.

- Unity는 `defense_tile_range`, `effective_weapon_profile`, `effective_basic_attack`, skill catalog range metadata를 조합해 범위를 재계산하지 않는다.
- 범위 overlay는 이동 가능 tile 표시가 아니다.
- 전장 밖/void/invalid tile은 제외하지만, obstacle/blocked tile은 기본 공격과 스킬 범위 표시에서 제외하지 않는다.
- 장애물은 지상 이동/배치/pathfinding/충돌에 영향을 주는 정보다.
- 실제 피격 가능성은 공격/스킬의 target validation, 공중 대상 가능 여부, 유효 hostile target 규칙이 판단한다.

## Combat Preview Boundary

`CombatPreview`는 node confirm, preview panel, battle setup preparation에 쓰이지만, 모든 내부 계산 필드가 Unity-facing/stored JSON 계약은 아니다.

- `enemy_stat_scale`은 floor scaling을 enemy spawn draft에 적용하기 위한 core 내부 계산값이다.
- `enemy_stat_scale`은 serialized preview JSON에 포함하지 않는다.
- serialized `CombatPreview`를 다시 읽어 battle setup source로 사용하지 않는다.
- Unity에 위험도 표시가 필요하면 raw scale 값 대신 별도 표시용 DTO(`threat_level`, `floor_scaling_summary` 등)를 새 계약으로 추가한다.

Unity-facing DTO shape, command result, battle update cadence, resync 세부 계약은 외부 canonical `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`와 `/mnt/f/unity projects/ark/docs/unity_core_contract.md`를 따른다.
