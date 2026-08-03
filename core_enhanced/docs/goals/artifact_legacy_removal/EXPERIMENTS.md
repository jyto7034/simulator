# Artifact Legacy Removal Experiments

Goal: G27

Updated: 2026-07-14

## Entry Evidence

- G28 live Shop content and runtime/DTO/Unity contracts contain no Artifact dependency.
- Artifact remains isolated in live RON/loading, the item registry and dedicated inventory slots,
  rewards/admin grants, root/display DTOs, battle scenario/runtime stat/trigger/event-log surfaces,
  fixtures and Unity admin tooling.
- Entry executable boundary inherited from G28: Core 768, game_server 30, Unity EditMode 220, Unity
  PlayMode 52 plus 4 explicit external-server skips, and zero cleared-console errors.

## Dirty Baseline

- Core/server repository HEAD: `c2c5dc5ec54bd26530e46f0d7ee7909de3ed7a1f`.
- Core status fingerprint: `d8b0d8703125a9564afbdc5c8ed996275f79f4d6a1a3454cf098cee30fa62f3e0`.
- game_server status fingerprint: `08b76e70c6b65187a2dbae7b9992a7297590130e44d9c45d415c992e2101aa89`.
- Unity HEAD: `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`.
- Unity status fingerprint: `c54b95068819a8a27a66d98980169fadc811dd3403d4431fcbcdeef29134eb40`.
- Preserve every pre-existing dirty change. G27 owns only Artifact deletion and required direct
  schema/fixture/canonical fallout.

## Planned Experiments

| Date | Experiment | Expected Evidence | Status |
|---|---|---|---|
| 2026-07-14 | Delete Artifact live source and loading/catalog schema | Live bundle loads without an empty compatibility catalog | passed |
| 2026-07-14 | Delete inventory/reward/admin/public DTO surfaces | Removed vocabulary fails closed and ordinary G24 behavior is unchanged | passed |
| 2026-07-14 | Delete battle scenario/stat/trigger/cooldown/log surfaces | Battle setup and execution contain no Artifact instance or effect path | passed |
| 2026-07-14 | Regenerate Rust-to-Unity inventory/Shop fixtures | Byte-exact fixtures contain no Artifact field or item variant | passed |
| 2026-07-14 | Remove Unity admin/model/mock/test surfaces | Current client compiles and full suites pass without placeholder fields | passed |
| 2026-07-14 | Audit current source/live/canonical surfaces | No Artifact match outside historical goal records | passed |

## Completion Evidence

- Deleted `artifact_data.rs` and live `data/artifacts/base.ron`; the embedded source set is 22 and
  has no empty or optional replacement field.
- Item registry/inventory, reward grant kind/effect, admin command/catalog, root/display DTO, server
  error mapping, Unity admin catalog/command, and all fixture fields are absent. A strict RON test
  proves the removed `GrantArtifact` variant is rejected.
- Battle scenario/runtime instances, deterministic ID namespace, permanent stat application,
  side-global trigger/activation collection, cooldown source, spawn log/DTO and validator counters
  are absent. Equipment trigger ordering remains explicitly tested.
- The inventory, Shop and action-capability fixtures regenerate byte-exactly from Rust. The action
  fixture was synchronized to its current generator, including accumulated compendium/reserve/catalog
  v2 fields, instead of preserving a hand-edited stale shape.
- Final executable boundary: Core 715 library plus 44 integration/binary tests (759 total),
  game_server 30, Unity EditMode 220, Unity PlayMode 52 passed plus 4 explicit external-server skips,
  zero failures, and cleared Unity console zero errors.
- Formatting, Core scoped diff, Unity scoped whitespace/diff, live-data/current-source/server/Unity/
  canonical searches and fixture drift checks pass. The sole current-source name match is the
  deliberate strict removed-vocabulary rejection test; historical goal records remain history.
