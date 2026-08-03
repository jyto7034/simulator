# Cross-Layer Phase 03 Event Completion Review Experiments

Goal: R03

Updated: 2026-07-14

## 2026-07-14 — Review initialized

- G02 review: Complete / High.
- G03 review: Complete / High after one Unity strict-reader correction.
- G19 review: Complete / High.
- R03 does not accept those verdicts as sufficient by themselves; it will retrace current
  runtime/data/transport/Unity/canonical sources and the combined boundary.

## 2026-07-14 — Current combined source retrace

- Core Event handler validates exact node/event/scene/choice/employee identity, shares payload
  policy with capability projection, rolls only before first commit, persists the result, plans
  grant/battle before mutation, and composes the Phase 2 battle transaction.
- Live Event RON is panels/advance-only without old choices/effects. Strict authored fixtures cover
  the employee-check branch without inventing live content.
- Root selected Event and `EventState` use one `EventSceneSnapshotDto`; game_server directly
  deserializes `PlayerBehavior`, maps Core results generically, and adds compressed data only to
  CombatBattle snapshots. It has no Event/VN JSON repair.
- Unity Event reader/composer/router/handlers consume one typed route, keep only local cursor and
  uncommitted selection, and fail closed on malformed scalar/array/eligibility/roll fields.
- VN definition owns presentation graph/mode; trigger owns activation/repeat/checkpoint/priority /
  blocking. Reachable Standard/Endless scripts resolve exactly in Unity and missing scripts cannot
  advance. Event is absent from trigger sources and VN catalog consumption.

## 2026-07-14 — Independent representative validation

- Current Core Event map-flow group: 7/7 passed, including shared root/result DTO, capability,
  identity rejection, one grant/result, failure completion, and combat/retreat persistence.
- Current Core VN group from G19 fingerprint: 22/22 passed; trigger-only blocking exact test 1/1.
- Current Unity combined Event plus VN catalog focused run: 8/8 passed (job
  `414063ca486642fca0bd6cbe58a75fa8`).
- Current broad evidence: Core 730, game_server 28, Unity EditMode 202, ordinary PlayMode 52,
  explicit live Event 1, explicit live Endless VN 1.
- Scoped `git diff --check` first found one whitespace-only line at the G03 live-test insertion.
  Removed that line through Unity MCP; identical scoped diff check passed.
- Production legacy/canonical/alias/fallback/ignored searches found only intentional rejection-test
  occurrences of removed `default_blocking`, flat Event fields, and old lowercase Event command.
  Canonical `default_blocking` occurrence states that the removed field is invalid. No relevant
  ignore marker was found.
