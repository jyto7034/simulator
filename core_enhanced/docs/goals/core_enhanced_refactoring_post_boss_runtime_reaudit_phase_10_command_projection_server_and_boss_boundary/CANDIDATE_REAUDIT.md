# Phase 10 Candidate Reaudit

## Current Dispositions

| Candidate | Current evidence and Boss impact | Disposition |
| --- | --- | --- |
| CRA-P10-C01 | command owner/scope/authority remains repeated across `ActionKind`, `PlayerBehavior`, capability evaluators, Core dispatch and four host recovery intercepts | `still_valid` |
| CRA-P10-C02 | `BehaviorResult` still expands through parallel domain result enums, result-type matches and manual JSON; roster projection remains unconstructed and all-variant parity is absent | `still_valid` |
| CRA-P10-C03 | server setup/update variants still reauthor every Core DTO field; the ordinary seam is current, while no first public Boss wire exists | `still_valid` |
| CRA-P10-C04 | server still serializes the typed Core snapshot to `Value`, then indexes, appends, clears and rebuilds host fields/capabilities | `still_valid` |
| CRA-P10-C05 | stable `GameError` codes remain independently matched by two Core snapshot helpers and the server adapter | `still_valid` |
| CRA-P10-C06 | unused build dependencies, broad direct dependencies, empty default binary/ctor, test-enabled fixture bins, logging surface and unconditional `tokio_unstable` remain | `still_valid` |
| CRA-P10-C07 | the two-byte empty `tests/test_actor/mod.rs` remains disconnected and absent from Cargo test targets | `still_valid` |

## Candidate Evidence And Readiness

### C01 — typed command ownership remains distributed

`ActionKind::scope` and `requires_battle_authority`, `PlayerBehavior::kind`, common/battle membership, projected
capability arrays and `GameCore` dispatch remain separate authorities. The server also intercepts ContinueRun,
ResetActiveRun, RetryEncounterCompendium and ResetEncounterCompendium. Current state/VN/revision tests protect
representative parity but there is no exhaustive all-variant owner descriptor. C01 stays a
`behavior_preserving_refactor`, known subtotal `13/18`, `tests_first_then_implementation`.

### C02 — one semantic result still has multiple projection representations

Core now gives the server a typed `BehaviorCommandResultContract`, which is the intended dependency direction, but
Core itself still converts `BehaviorResult` into eleven domain branches and separately derives `result_type` and
JSON. Existing tests cover representative payloads, not every result or exact `RunComplete`. Public helper consumers
remain unverified. C02 stays a `behavior_preserving_refactor`, `17/18`,
`all_variant_goldens_and_external_api_inventory_first`.

### C03/C04 — public transport and host overlay duplication remain current

`PlayerGameServerMessage::BattleUpdate` and `BattleSetupSnapshot` repeat Core fields, and their constructors destructure
those DTOs field by field. Exact top-level setup/update and nested resync tests protect the present topology but do not
remove change amplification. C03 remains a `behavior_preserving_refactor`, `14/18`,
`exact_serde_parity_tests_first`.

`build_state_snapshot` still turns `PlayerStateSnapshotDto` into untyped JSON before appending active-run and
compendium status and mutating capability arrays. The fatal-compendium branch replaces capabilities through JSON.
C04 remains a `behavior_preserving_refactor`, `17/18`, `table_tests_then_implementation`.

### C05 — error identity is not centralized

`snapshot_game_error_code`, `snapshot_game_error_message` and `From<GameError> for PlayerGameActorError` independently
match the full enum; code literals are repeated between Core and server. Representative disabled/obstacle tests are
not an exhaustive equality table. C05 stays a `behavior_preserving_refactor`, `11/18`,
`full_code_table_then_implementation`.

### C06/C07 — manifest and disconnected-file cleanup remain open

The package still has no build script while declaring four build dependencies, an empty `src/main.rs`, a no-op ctor,
auto-discovered generator bins participating as test harnesses, logger API with no workspace caller and
`tokio_unstable` despite no Core tokio use. Direct-dependency and standalone tool/API consumers remain an external
gate. Under the campaign vocabulary C06 is `cleanup` (the historical catalog called it
`behavior_preserving_cleanup`), `12/18`, `external_inventory_then_mechanical_cleanup`.

`tests/test_actor/mod.rs` contains only a newline, has no module edge and is absent from metadata. C07 is `cleanup`,
`3/18`, `implementation_ready`; production deletion remains outside this read-only campaign.

## Boss Reconciliation

`BattleEvent::BossRuntimeWake` explicitly documents that it is not projected into the public event log, carries no
serde derives and is consumed internally by simulation. Current public presentation DTOs convert `BattleLogEvent`,
not scheduler events. Server only copies Boss catalog data into the Core database; no server or accessible C# Boss
lifecycle carrier exists. Therefore C03's original ordinary transport refactor remains valid, while its
Boss-specific expansion is `deferred/conditional` until an actual first public wire. No distinct `CRR-P10-*`
candidate was found.
