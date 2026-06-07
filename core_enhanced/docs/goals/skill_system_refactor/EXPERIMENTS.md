# Skill System Refactor Experiments

## 2026-06-02 Initial Read

시도:

- goal 문서와 현재 `ability.rs`, area runtime, timeline, skill validation, live RON 참조를 검색했다.

결과:

- 현재 AoE는 타일 기반이 아니라 `SkillAreaShapeDef` 연속좌표 shape 기반이다.
- DefenseRoute 정책과 맞추려면 schema/runtime/timeline/RON/tests를 함께 정리해야 한다.
- 단순히 `contains_area_point`만 바꾸는 접근은 Unity preview와 event log 계약을 어긋나게 하므로 폐기한다.

다음 실험:

- `defense_tile_range`에 포함된 타일의 유닛을 수집하는 focused runtime helper를 추가하고, 기존 geometric area test 중 정책과 충돌하는 것을 tile-range test로 교체한다.

## 2026-06-02 Live RON TileArea Migration

시도:

- `../game_resources/data/skills/base.ron`의 geometric `delivery: Area(area: (shape: ...))` step을 `delivery: TileArea(area: (...))`로 변환했다.
- 누락된 `defense_tile_range`에는 기존 live RON의 기본 전방 타일 패턴을 추가했다.
- `base.generated.ron`은 live 로드 경로에서 참조되지 않음을 확인하고 `legacy_base.generated.ron`으로 분리했다.

결과:

- `rg` 기준 official `skills/base.ron`에는 `delivery: Area`와 geometric shape가 남지 않았다.
- `cargo test -p game_core --test ron_loading -- --nocapture`가 통과했다.
- `GameDataBase` 조립 단계에서 abnormality/corroded employee/skill fragment가 legacy geometric Area 스킬을 참조하면 실패하도록 validation을 강화했다.

다음 실험:

- Unity-facing skill metadata/readiness 계약과 buff registry RON 전환 범위를 확인한다.

## 2026-06-02 Skill Catalog / Buff Registry

시도:

- run snapshot에 `skill_catalog`를 추가해 Unity가 스킬 이름, focus 시간, target policy, `defense_tile_range`, delivery kind를 추론 없이 읽을 수 있게 했다.
- live battle deployment DTO의 `deployed_units[*].skill_readiness`에 현재 resonance, activation mode, manual activation 가능 여부와 reason을 추가했다.
- 기존 hardcoded buff registry 값은 `../game_resources/data/buffs/base.ron`으로 이동하고, runtime API는 기존 `buffs::get`, `contains_name`을 유지했다.

결과:

- 수동 스킬 장착/배치/발동 테스트에서 snapshot `skill_catalog`와 `skill_readiness`가 함께 노출됨을 검증했다.
- buff registry focused test가 RON 로딩 후 기존 poison/stun/freeze/silence 값을 유지함을 검증했다.

다음 실험:

- 전체 game_core test와 game_server check를 돌려 DTO 추가가 서버 mapping에 영향을 주는지 확인한다.

## 2026-06-02 TileArea Anchor Semantics

시도:

- 전체 `game_core` 테스트 중 `skill_refactor_validation`에서 geometric Area를 직접 고정하던 테스트 2개가 실패했다.
- 해당 테스트는 DefenseRoute 공식 정책과 충돌하므로 ignored 처리하지 않고 삭제했다.
- live RON smoke에서 `TileArea.anchor: CastTarget`이 실제 피격 타일의 기준점까지 바꾸어, `defense_tile_range`가 단일 source of truth라는 정책과 충돌하는 문제를 확인했다.

결과:

- `TileArea`의 실제 affected tiles 계산 기준을 시전자 tile + facing으로 고정했다.
- `TileArea.anchor`는 timeline center, impact context, VFX 기준점으로만 사용한다.
- focused test가 `TileArea.anchor: CastTarget`이어도 피격 타일은 caster의 `defense_tile_range`에서 나온다는 계약을 고정한다.
- `skill_test` helper가 live skill의 `defense_tile_range`를 broad 9x9 range로 덮어쓰던 레거시 보정을 제거했다.
- live skill test 보드는 공식 player facing/right 전방 tile range에 맞춰 재배치했다.
- `cargo test -p game_core --test skill_refactor_validation -- --nocapture`가 통과했다.
- `cargo test -p game_core --test skill_test_suite -- --nocapture`가 통과했다.
- `cargo test -p game_core -- --nocapture`가 통과했다.
