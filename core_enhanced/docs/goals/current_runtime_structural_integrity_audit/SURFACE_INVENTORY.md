# Runtime Surface Inventory

- 기준선: `B-001`
- 상태: 최종
- 원칙: production entry에서 authority mutation과 projection까지 확인된 표면만 증거 수준을 명시한다.

## S-001. Crate와 Cargo 표면

- crate: `game_core` 0.1.0, Rust 2021, library root `src/lib.rs`
- root export: `pub mod game`
- feature: `event_log_checks` 하나; opt-in이며 E-002에서 all-features로 실행됨
- explicit binary: encounter-compendium/event/inventory/shop contract fixture generator 4개; `test = false`,
  `bench = false`; E-002에서는 각 0-test runner로 관측됨
- direct dependency: `rand`, `rapier2d`, `ron`, `serde`, `serde_json`, `serde_path_to_error`, `tracing`, `uuid`
- 증거: E-002, E-003, `Cargo.toml`, `src/lib.rs`

## S-002. Public top-level modules

`src/game/mod.rs`는 다음 26개 module을 모두 `pub`으로 노출한다.

`ability`, `battle`, `behavior`, `boss_omen`, `combat_power`, `combat_preview`, `combat_setup`, `data`,
`determinism`, `employee`, `employee_psychology`, `encounter_compendium`, `encounter_progression`, `enums`,
`events`, `growth`, `managers`, `map`, `monolith`, `range_preview`, `recruitment`, `resources`, `reward`,
`reward_policy`, `skill_fragment`, `stats`, `visual_novel`, `wave_resolution`, `world`.

정정: 위 문장에 열거된 실제 module은 29개다. E-003 최초 요약의 26개는 수동 계수 오류이며 module source
열거를 권위 증거로 사용한다. 이 정정은 runtime finding이 아니라 감사 기록 정정이다.

- entry/authority 의미: GameCore orchestration, standalone BattleCore simulation, public data/domain catalogs and DTOs are
  all exported from one broad crate namespace; findings 001/003/004/007/008 classify the unsafe boundary portions.
- 증거: E-003, `src/game/mod.rs`

## Direct Dependency Responsibility Signals

| dependency | 확인된 production 책임 표면 | 현재 판단 |
| --- | --- | --- |
| `rand` | seeded map/recruitment/reward/event/shop selection | sorted/authored-order deterministic inputs; E-017 |
| `rapier2d` | private movement backend | same-process reversed-insertion probe pass; cross-platform unproven |
| `ron` | raw authoring parser | strict official publication; E-013 |
| `serde` | domain/data/DTO/persistence | direct domain/wire coupling RUNTIME-AUDIT-008 |
| `serde_json` | projection/snapshot/checkpoint/debug export | central projection and strict V4 checkpoint; debug I/O non-authoritative |
| `serde_path_to_error` | live RON loader | source+nested path preservation; E-013 |
| `tracing` | observation | no gameplay authority writer; trace export failure warning-only |
| `uuid` | deterministic run/battle identity plus external player IDs | namespaced deterministic core helpers; E-008/E-017 |

## S-003. GameCore production entry

- constructors/profile: `new`, `new_with_*profile`, profile getter/merge/reset
- command: `execute`, `execute_with_source_command_id`,
  `execute_with_source_command_id_and_battle_cursor`, `execute_with_transport_context`
- reconnect query/result: `active_battle_reconnect_projection`
- state queries: state name/value, run/inventory/roster/event snapshots, battle update/result attachment
- active-run persistence/rollback: capture/restore/export/continue
- admin: typed command surface(후속 세부 조사)
- leaked test mutation: `set_enkephalin` (RUNTIME-AUDIT-001)
- evidence: E-006

## S-004. Battle simulation public surface

- construction: `BattleCore::new_from_scenario`
- setup/harness: `enqueue_event`
- execution: run/start/step/until/by/finalize, `apply_live_command*`
- query: scenario, world/tile position, live bodies, counts, skill/control/readiness, event updates
- direct public fields: units/battlefield/game-data/event-log/counters/seed/recording stacks
- exports: `battle::core`, `battle::enums`, movement/type DTOs are public through `game`
- evidence/findings: E-008, RUNTIME-AUDIT-003/004

## S-005. Player behavior and admin command surface

- player commands: macro-defined typed `ActionKind` set and strict `PlayerBehavior`; route ownership is exhaustive across
  CoreCommon/CoreDomain/CoreBattle/CoreContextual/four HostRecovery actions.
- projections: 47 characterized `BehaviorResult` shapes; command payload/read-only preview/battle side-update are centrally
  separated, but payloads reuse domain serde and envelope is unversioned (RUNTIME-AUDIT-008).
- errors: typed `GameError` with stable `code()`; adjacent adapter maps every variant to explicit code/message.
- admin: 20 deserializable `AdminCommand` variants and exhaustive public `execute_admin_command`; mutations are explicit
  admin authority, unlike the stray `set_enkephalin` helper.
- evidence: E-015/E-016/E-019/E-020.

## S-006. Data and persistence surface

- official data: strict schema-1 manifest + 24 embedded live RON files, parse/validate/compile/private validated wrapper,
  immutable `Arc<GameDataBase>` publication (E-013/E-014).
- alternate data: public fixture builder/new publishes the same runtime type and panics on invalid contracts
  (RUNTIME-AUDIT-007).
- core persistence: strict V4 bytes/sideband export, graph validation, checkpoint restore and rollback capture (E-011).
- adjacent direct durable consumer: checksummed current/backup store, lock/temp/sync/replace/recovery and actor rollback
  (E-012); no server-wide claim.

## 조사하지 않은 소비자

- old/current Unity implementation and compatibility: explicit audit exclusion, not accessed.
- unrelated game_server routes/stores/network/auth behavior: not audited.
