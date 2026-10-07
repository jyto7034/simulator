# DefenseRoute 전술 지휘 Phase 8 — 수동 지정 스킬

Status: complete

Implementation Complete: true

Goal Type: phase

Execution Mode: implement_and_verify

Production Mutation Authorized: yes — 이 Phase의 `DR-16` 범위에 한정

Parent Master: [defense_route_tactical_command_master](../defense_route_tactical_command_master/PLAN.md)

Predecessor Audit:
[Phase 7 Complete / High](../defense_route_tactical_command_phase_07_atomic_swap/COMPLETION_AUDIT.md)

Phase: 8

Goal Name: `defense_route_tactical_command_phase_08_manual_skill_targeting`

Target Repository: `/mnt/f/work/simulator`

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## 목표

기존 effect `SkillTarget`과 별도의 required `SkillCastSelection` authority를 추가해 Self/AutomaticUnit/PlayerUnit/
PlayerTile activation shape를 엄격히 구분한다. Explicit target acceptance와 tactical focus lifecycle을 원자 구현하고
Black Round progression만 첫 live PlayerUnit content로 전환하며 PlayerTile은 engine/fixture만 제공한다.

## 승인된 정책과 근거

- Primary policy: `DR-16`
- Target source:
  [DefenseRoute 전술 지휘 목표 정책 결정](../../decisions/defense_route_tactical_command_policy_2026-08-06.md#dr-16--수동-지정-스킬)
- Master boundary:
  [Phase 8](../defense_route_tactical_command_master/PLAN.md#phase-8--수동-지정-스킬)
- Predecessor:
  [Phase 7 Complete / High](../defense_route_tactical_command_phase_07_atomic_swap/COMPLETION_AUDIT.md)

## 확정 정책

- `SkillCastSelection::Self | AutomaticUnit | PlayerUnit | PlayerTile`은 required live skill field다. Default 또는
  effect/archetype 추론, target kind fallback은 없다.
- Self는 caster, AutomaticUnit은 existing Core targeting, PlayerUnit/PlayerTile은 explicit Unit/Tile payload를
  요구한다. Missing/mismatch는 typed rejection이고 focus/state/cost를 바꾸지 않는다.
- Acceptance는 caster/manual/Attack/activity/Hard CC/Silence/resonance/cooldown 뒤 explicit target lifecycle,
  side/air/usefulness/reach/range 또는 valid tile/reach/range를 전부 사전 검증한다.
- Accepted Unit/Tile identity/coordinate는 고정한다. 실행 시 무효는 no-op이고 retarget/refund/rollback하지 않는다.
  Player selection과 `RetargetOnStep` 결합은 strict reject한다.
- PlayerUnit/PlayerTile 선택은 server-authoritative tactical focus를 사용하고 playback speed를 X0_5로 임시 설정한다.
  이미 Pause면 frozen을 유지한다. Focus state는 purpose/caster/skill/restore playback만 소유한다.
- Preview는 read-only candidate, final `ActivateSkill`이 다시 검증한다. Cancel은 비용 없이 focus를 끝내고 playback을
  복원한다. 성공은 skill start/playback restore/focus end를 한 commit, 실패는 focus 포함 무변경이다.
- Reconnect/checkpoint는 focus purpose/caster/skill/restore playback을 보존한다. Pause에서 시작한 focus 종료 뒤 Pause다.
- Self/AutomaticUnit은 focus 없이 즉시 command한다. Manual basic attack과 별도 active-skill cooldown은 없다.
- 첫 live PlayerUnit은 `fragment_freischutz_black_round`와 awakened `freischutz_magic_bullet` progression이다.
  Existing range/focus/damage/delivery 값은 유지한다. 첫 live PlayerTile skill은 없다.

## 범위

- Required skill selection data/schema/validation and exact live migration
- Manual target preview/focus/cancel/activate command/state/result/event/capability
- Unit/tile acceptance, capture and no-retarget execution semantics
- Pause/playback/reconnect/checkpoint/server current contract
- Black Round progression live PlayerUnit and PlayerTile engine fixtures
- Focused/broad/static/canonical/Complete-High audit

## 명시적 비범위

- Manual basic attack, cursor/hover/local Unity UI
- New live PlayerTile skill or retarget policy
- New active-skill cooldown or skill-cost tuning
- FieldWork data/state/activity and Phase 9 integration
- Unity runtime/contract/tests

## 구현 계획

1. Existing skill targeting/manual activation/playback/checkpoint/data schemas and Black Round progression을 조사한다.
2. Required selection carrier and strict invalid combinations을 implement/migrate한다.
3. Read-only target candidate and authoritative focus lifecycle을 add한다.
4. Explicit unit/tile final acceptance and captured execution semantics을 wire한다.
5. World/server result/action/checkpoint/reconnect/current strict DTO를 cutover한다.
6. All failure immutability, Pause/restore, invalidation/no-retarget and PlayerTile fixtures를 direct test한다.
7. Canonical/legacy/dirty/broad gate and Complete/High audit을 수행한다.

## 완료 조건

- All live skills explicitly own one selection and invalid/missing/legacy shapes reject startup.
- Self/AutomaticUnit/PlayerUnit/PlayerTile target requirements and acceptance are direct-tested without fallback.
- Focus start/cancel/success/failure/Pause/reconnect state and playback restoration are atomic and deterministic.
- Captured target invalidation no-ops without retarget/refund/rollback; RetargetOnStep combination rejects.
- Black Round progression is the only first live PlayerUnit path; no live PlayerTile content exists.
- Core/server/RON/canonical/static/broad gates and Complete/High audit pass.

## DTO/data/save migration

Required selection was cut over in all 25 live skill definitions with no compatibility reader. Focus is battle-local
`ActiveBattleSession`/checkpoint state and is not part of the durable run payload, so no save version changed. Skill catalog
shape is version 3. Player behavior/result/checkpoint/server transport changed as one strict repository release.

## 필수 focused 검증

- required selection migration/missing/unknown/invalid combination
- Self/AutomaticUnit immediate command and PlayerUnit/PlayerTile required payload mismatch
- caster/manual/Attack/activity/Hard CC/Silence/resonance/cooldown and target side/air/usefulness/range gates
- focus preview/start/cancel/success/failure, Pause restore, reconnect/checkpoint/root/server
- captured unit/tile invalidation, no retarget/refund/rollback and strict RetargetOnStep reject
- exact Black Round progression live data and PlayerTile engine-only fixture

## 필수 broad 검증

- `cargo test -p game_core`
- `cargo test -p game_server`
- `cargo test -p game_core --test ron_loading`
- `cargo fmt --all -- --check`

## 중단 조건

- Approved target/focus/playback/invalid-target semantics cannot map to existing runtime without a new user-visible choice
- Required migration would need compatibility/dual schema or durable save version not authorized here
- Core/server/live data/test runner is inaccessible
- Predecessor/user dirty cannot be preserved safely

## Compatibility

승인된 compatibility는 없다. Strict current required selection/command/state only.

## 완료 감사 Evidence Matrix

| ID | 완료 조건 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P8-01 | Required selection/live migration | `DR-16` | required raw/runtime field, pair validation, exact 25-skill RON cutover | missing/unknown/pair/Retarget validation + exact live matrix | pass |
| P8-02 | Explicit unit/tile acceptance | `DR-16` | Core base/candidate validator and typed reasons | Self/Automatic/player unit/player tile, side/air/usefulness/range fixtures | pass |
| P8-03 | Focus/playback/Pause/reconnect | `DR-16`, `DR-15` | active session focus, begin/cancel/preview/activate handlers | slow focus, Pause focus, failure/cancel/success/reconnect/capability world test | pass |
| P8-04 | Capture/no-retarget lifecycle | `DR-16` | pending target identity + acceptance tile snapshot | dead direct target no-op/no fallback/refund; moving unit area fixed tile | pass |
| P8-05 | DTO/event/server/version | Master Phase 8 gate | current action/result/checkpoint/catalog v3/server mapping | strict serde, focus serialization, read-only preview and battle update mapping | pass |
| P8-06 | Canonical/legacy/dirty/broad | Master common gate | four canonical docs, no compatibility/Phase 9 placeholder | Core 903/server 56/RON 24/check/fmt/diff/static | pass |

## 현재 진행 상태

- Phase 7 completion audit 확인: 완료
- Child path/dirty 확인과 문서 초기화: 완료
- Required selection schema/live migration: 완료
- Core explicit target acceptance/capture/no-retarget: 완료
- Tactical focus/playback/checkpoint/world/server current contract: 완료
- Canonical/focused/broad/static audit: 완료
- Completion audit: [Complete / High](COMPLETION_AUDIT.md)
