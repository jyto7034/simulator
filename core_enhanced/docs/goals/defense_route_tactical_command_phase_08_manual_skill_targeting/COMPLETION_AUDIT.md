# DefenseRoute 전술 지휘 Phase 8 — Completion Audit

Status: Complete

Confidence: High

Goal:
[Phase 8 수동 지정 스킬](PLAN.md)

Policy Authority:
[DR-16](../../decisions/defense_route_tactical_command_policy_2026-08-06.md)

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

Audit Date: 2026-08-06

## 판정

Phase 8은 `Complete / High`다. 모든 live skill이 required cast selection을 명시하고 Self/AutomaticUnit은
focus-free, PlayerUnit/PlayerTile은 exact typed payload와 authoritative tactical focus를 사용한다. Acceptance
identity/tile은 고정되며 invalidation에 fallback/retarget/refund/rollback이 없다. Black Round base/awakened만
첫 live PlayerUnit path이고 live PlayerTile content는 없다.

## Evidence Matrix

| ID | 완료 조건 | 정책 근거 | Runtime/data/contract evidence | Validation evidence | 판정 |
| --- | --- | --- | --- | --- | --- |
| P8-01 | Required selection/live migration | `DR-16` | ability/raw schema + validator; live RON 25/25 | missing/unknown/pair/Retarget rejects; exact 4/19/2/0 matrix | pass |
| P8-02 | Explicit unit/tile acceptance | `DR-16` | shared Core base/final validators and typed reasons | payload kind, side/air/usefulness/reach/range/unit/tile fixtures | pass |
| P8-03 | Focus/playback/Pause/reconnect | `DR-16`, `DR-15` | battle-local focus and world begin/cancel/preview/activate | slow/Pause/failure/cancel/success/reconnect/capability atomic world test | pass |
| P8-04 | Capture/no-retarget lifecycle | `DR-16` | pending identity + accepted tile snapshot | dead direct target and moving area target execution tests | pass |
| P8-05 | DTO/server/version | Master Phase 8 | action/result/checkpoint/readiness/catalog v3/server mapping | strict request, focus serialization, preview/no-snapshot tests | pass |
| P8-06 | Canonical/legacy/dirty/broad | Master gate | four canonical docs, current-only contract | Core 903/server 56/RON 24/check/fmt/diff/static | pass |

## Runtime 경로 감사

- Data: required `SkillCastSelection` → strict pair/step validation → exact live RON publication
- Direct: Self/AutomaticUnit `ActivateSkill` → Core base/final preflight → `PendingSkillCast`
- Player focus: `BeginSkillTargeting` → focus/X0_5 or frozen Pause → read-only preview → final `ActivateSkill` or Cancel
- Capture: accepted Unit/Tile + unit acceptance tile → pending cast → direct validity no-op or fixed TileArea geometry
- Projection: readiness selection/target requirement + checkpoint focus + action capability + catalog v3
- Server: strict PlayerBehavior → generic battle update for focus mutation; read-only preview command result

No hover/local target, implicit selection, automatic fallback, player retarget, active-skill cooldown, durable battle save or
live PlayerTile content participates.

## 검증 결과

Focused:

- selection missing/unknown/incompatible and player RetargetOnStep rejects — pass
- Self/AutomaticUnit/PlayerUnit/PlayerTile payload and target eligibility fixtures — pass
- direct invalidation/no-retarget/no-refund and accepted-area-tile fixtures — pass
- world focus preview/failure/cancel/Pause/reconnect/success/capability transaction — pass
- game_server strict request, checkpoint focus and read-only preview mapping — pass

Broad:

- `cargo test -p game_core --lib --quiet` — 855 passed
- `cargo test -p game_core --quiet` — 903 passed total
  (library 855 + integration 3 + 4 + 24 + 13 + 4)
- `cargo test -p game_server --quiet` — 56 passed
- `cargo test -p game_core --test ron_loading --quiet` — 24 passed
- `cargo check --workspace --all-targets` — pass, warning 없음
- `cargo fmt --all -- --check` — pass
- `git diff --check` — pass
- legacy/future-placeholder/static searches — pass
- Failed/ignored — 0

Strict `-D warnings` clippy는 existing tree-wide lint debt(해당 run에서 lib 81, test 130 errors) 때문에 실패했으며
Phase 8 required gate가 아니다. Phase 8 신규 lint로 확인된 helper argument count와 Copy clone은 제거했다.

## Migration과 compatibility

- Data: all 25 live skill definitions receive required selection; no default/alias/dual reader
- Command: begin/cancel/preview plus existing activate with strict typed target; unknown fields reject
- DTO: readiness selection, focus checkpoint, preview result and catalog v3 are same-release current shapes
- Event: existing `ManualCastStart.target`/event-log version 30 reused; focus-only event version change 없음
- Save: active battle/focus is outside durable run payload; save version/migration 없음
- Core/game_server atomic repository release; Unity adoption remains approved out of scope

## Dirty와 비범위

- Phase 0~7 verified output and unrelated user dirty were preserved.
- Existing dirty `game_server/docs/unity_bridge_protocol.md` was not touched. Unity runtime/contract/UI/tests were not read or
  changed.
- Phase 9 FieldWork data/state/activity/placeholders were not added.

## 잔여 위험

현재 Phase 8 범위의 알려진 correctness gap은 없다. Unity consumer adoption remains out of scope. Phase 9 may use the
same activity/capability/checkpoint seams but must not reinterpret selection, focus restore or accepted-target semantics.
