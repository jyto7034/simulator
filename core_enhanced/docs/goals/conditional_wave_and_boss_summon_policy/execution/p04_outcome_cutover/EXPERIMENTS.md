# P04 Outcome Cutover Experiments

## 2026-08-17 — GEX-00 intake

- Exact selected member: `GS-CWBS-P2G50-002 / GOAL-CWBS-P04-OUTCOME-CUTOVER`.
- Phase/nodes: `PHASE-CWBS-04-OUTCOME-CUTOVER / AG-CWBS-04-OUTCOME-CUTOVER`.
- Direct predecessor: P03 `COMPLETION_AUDIT.md / Audit Pass 1 / Complete / High`; its audit explicitly leaves named
  terminal selection to P04 and confirms Wave/summon settlement seams current.
- Initial full baseline is inherited: `game_core 1,080 + game_server 72 / 0 failed`; P04 does not repeat it.
- Relevant files are pre-existing dirty current actual. P01-P03 and user changes are preserved; no reset, restore, stash,
  cleanup or whole-tree format is authorized.
- Selective source inspection confirms the current delta:
  - `BattleScenario` owns one `WinCondition`; `compute_winner` directly returns Player/Opponent/Draw;
  - `ScenarioAction::EndBattle` and `BattleEvent::EndBattle` are direct verdict producers;
  - `battle_runtime.max_battle_time_ms` caps the simulation and falls back to Draw;
  - `BattleResult` and `CombatBattleState` carry winner but no outcome ID;
  - encounter-level `reward_mode/reward_uuids` and fixed-node `reward_ids` are competing flat authorities;
  - reward/progression and BossOmen flow branch on `winner == Player`.
- Current pre-mutation hashes:
  - `battle/scenario.rs` `7c72b27663e49369b7fa4dfd7f0cdb683f587246b1394748ca2d1e64220de1fe`
  - `battle/types.rs` `3567bbdb2105074258e024bf1504c54fdaf352eb686166cd315abe79dcdd11fb`
  - `battle/enums.rs` `0621a31ace38dbda7621af24cf9bd5299c7ef2c57cc5a9507fe3e41332f5af81`
  - `battle/core/sim.rs` `8aa218da7b2e8df01d0aa851e4aacdf0088de5d5f5c208e0f4e21ef9f059b794`
  - `data/pve_data.rs` `9cd24efebe3a206e5144bfa28f249827bf3716e89e0f0b9ad42b409ff4e7cf1d`
  - `events/combat.rs` `b0dd16350acd3f5d975661fae5c36a3da94ad854501eac5584a76e59a017a708`
  - `map/types.rs` `3f3f7794264be0a55799ac40f7303dceed8daf0b03605c9adc036db8eb36f6a6`
  - `map/facility.rs` `b170723ba9ee1570df8124a68f434fc46372f7e2105111ca1822d4da202ea416`
  - `combat_setup/rewards.rs` `86c0170378683c5cdbb8ab43657239d449893560743b15d24bfbef25bd9e9076`
  - `world/state.rs` `d012e0430c973ded43dba1641d8cd5f2a8c2c7a5b2d3a6527299d5a6b97ce5a5`
  - `world/combat.rs` `d65a5f26f9011f88bbce71b6321623de5b60ba6eb58ecaae210bf333203d1de5`
  - `resources/selection.rs` `7e2d788869f7e7f2893e51155f829262cdfdf253f456a2f211d7db8e65d7275f`
  - `behavior.rs` `bc147163931678140715bc1b9130231cb61e272865a52ca061f42a7de1fe7171`
  - `data/live_ron.rs` `f1c9f4beb34e9191da4d2b0a2f40f9b8b4a843b3d051672cfc8a3b174aa3db57`
  - live encounters RON `a3b38fa948208c225d90b197a2f4a449a7800c014d432caa692926d70a4a1e4d`
  - live node-assignment RON `82939e66ef7e335c06ba3442974ec85128455a29544b578477ba22dd332c0483`
  - live run-policy RON `92b9f0786530fc58406825dc24b89c6683d66ebf0cf8fd00d5f0aa0e7a0cea82`
- Production mutation: none in P04 at this point.

## 2026-08-17 — GEX-10 implementation and focused validation

### Implemented cutover

- Replaced `WinCondition`, direct scenario/battle `EndBattle`, `forced_winner`, `max_battle_time_ms` and implicit Draw with
  stable `BattleOutcomeId`, two-value `BattleVerdict`, closed neutral `TerminalCondition` and authored
  `TerminalOutcome` definitions.
- Outcome selection now runs only after the complete same-time event/Wave/Boss/summon bucket. Matching candidates sort
  Defeat first, then priority descending, then stable outcome ID ascending. The finish signal and `BattleResult` latch
  the selected ID/verdict; `BattleWinner` is a derived compatibility projection.
- `AllPlayerCombatantsDefeated` now requires at least one player combatant and every such combatant to be `Dead`.
  Predeployment emptiness and `Withdrawn` units do not satisfy Defeat.
- Migrated all 33 live encounters to 66 named terminal outcomes and 66 exhaustive outcome policies. Protected-object
  encounters use explicit `UnitDefeated` Defeat, the authored survival encounter uses `TimeReached` Victory, and every
  Defeat policy is non-positive.
- Migrated all seven fixed authored combat-node policies to per-outcome overrides. Publication validates exact encounter
  outcome coverage, reward references and non-positive Defeat policies.
- Result setup resolves every policy before battle start and selects only the committed ID after battle. Reward scaling,
  node bonus and encounter progression come from the selected policy; world completion remains under the existing
  `with_state_transaction` boundary. Retreat remains separate.
- Projected outcome ID/verdict through active result state, selected-event/run snapshots, completion summary, event-log
  attachment and compressed server transport. BossOmen Victory/Defeat continues through the selected verdict path.
- Removed the run-policy battle cap from typed data and live RON. No replacement default, alias or compatibility reader
  was added.

### Corrected implementation findings

- The first test-fixture mechanical migration matched `impl PveEncounter` as well as struct literals and changed three
  helper parameter lines. The exact lines were detected immediately, restored before compilation, and production data
  was not affected. A later occurrence-based protected fixture replacement also touched two wrong test literals; both
  were corrected by encounter ID before focused execution.
- Initial generated live RON wrote `Victory/Defeat`, while `BattleVerdict` has `snake_case` serde names. The first live
  loader execution rejected `encounters[0].terminal_outcomes[0].verdict`; all 66 values were corrected to
  `victory/defeat`, after which live publication passed.
- The first runtime implementation treated no active player combatant as `AllPlayerCombatantsDefeated`. Live battle
  entry therefore ended before deployment. The predicate was corrected to require an existing set whose members are all
  actually dead; a dedicated zero/Withdrawn/Dead regression now passes.
- Test fixture migration produced compile-only failures for removed fields. No production compatibility field was
  restored; fixtures and integration readers were migrated to named outcomes instead.

### Focused validation evidence

- `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_core --no-run`: pass; all unit and five integration targets build.
- `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_server --no-run`: pass; lib/main test targets build.
- `cargo test -p game_core --lib terminal_ -- --nocapture`: final `11 passed / 0 failed`; includes deterministic terminal,
  no implicit Draw, exact clock, live entry and Little Red terminal regression.
- named outcome schema rejection: `1 / 0`; empty, duplicate, no Victory, non-exhaustive and positive Defeat cases.
- unknown terminal unit reference rejection: `1 / 0`.
- zero/Withdrawn/Dead player-combatant predicate: `1 / 0`.
- exhaustive live outcome resolution/non-positive Defeat: `1 / 0`.
- `cargo test -p game_core --test ron_loading live_pve`: `4 / 0`.
- PVE encounter authoring/deserialization group: `6 / 0` before the named-outcome-specific addition; all passed.
- successful live publication/compiler smoke: `1 / 0`.
- same-time AttackResolve-before-terminal finalization: `1 / 0`.
- atomic result-completion rollback: `1 / 0`.
- retryable Defeat/Retreat/exhausted Defeat: `1 / 0`; repeated Retreat non-consumption: `1 / 0`.
- authored time outcome through live world tick: `1 / 0`; Pause clock freeze: `1 / 0`.
- result snapshot outcome ID/verdict projection: `1 / 0`.
- BossOmen capstone Victory Exit Gate and Defeat run failure: `2 / 0`.
- server compressed event-log outcome round-trip: `1 / 0`.
- `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo check -p game_core -p game_server`: pass, zero warnings.
- scoped `git diff --check`: pass.
- The inherited full baseline remains `game_core 1,080 + game_server 72 = 1,152 / 0 failed`; it was not repeated in P04.

### Current GEX-10 target identities

- `battle/scenario.rs` `1caed730779f276b0e83369d2f3bbd818272c557cf56c7775b37a883561aebc1`
- `battle/types.rs` `e1e9b8d735a9edc8b20a849d696918d0cda0de822c17ba978e956ab9b3702436`
- `battle/enums.rs` `6a2cec5725102aad03d5846cdefaeeff3f06a5eaf1facc7dd0967ae9b28178f8`
- `battle/core/mod.rs` `d0163df23a56390bbd753c9eae3b2e7ab1055d6b162e8e5d3d0cb7490d468e77`
- `battle/core/sim.rs` `df679fcaf2e1c192f7120b2ccc26cbf0d160c16f91b1b66d1bae534c953befb5`
- `data/pve_data.rs` `4df5e9755b8372666989fb4e33dc307c01f532fe1bfe8dd98a0f902cf0d625fb`
- `events/combat.rs` `bdf2a80a73061ef3d2abb5c3abb4edd430a5e4153fb0d0ae9e52c7c15185b5aa`
- `map/types.rs` `fd5d4a2900ccf5ed228d45ced89b9def0f0d60c6443e24e58ba9029fb686fc9f`
- `map/facility.rs` `0122d19e7e146ca6b3a8e94bad29416a4c8fd06a1e8f4a6a02076ab16fb2f6cd`
- `combat_setup/rewards.rs` `df3783aa968f2ea433e2305282d388fe696ee30227ac4ab4716c7f06f9cd9285`
- `world/state.rs` `b82abeb2abeadd8e5b1ae12d76aa342efffc9174d3531a1119022f303bdff9b6`
- `world/combat.rs` `97897daa4c6f4012db5e13ff231a2ce788d56ff2481181333e8b86dfc01044fa`
- `resources/selection.rs` `a61fa4d34edb2dbb49f4b9ea0dfe235fffeddc1aed0a3978eb4287f435b7035b`
- `behavior.rs` `b2c9c458caf577d667e460f8c8fbe2e962ecbfe8f7ae623983231fc9b711fd93`
- `data/live_ron.rs` `a8e6012a42b169507a636db841ab772989f00e60600b5435b62aa4dc753f9075`
- `world/tests/combat.rs` `2ea28912fbc46b3103a219112e5e59803e6aa511f8cb21f0e95dac3a4df98dc2`
- live encounters RON `8871176b39a0e8f97d9502f399e607f0a7d9c2083b35f1bce513fbf68a323167`
- live node-assignment RON `9389e0644e4f4c4ca75a861c07f423cb484c6f11a5867cf24ee6e8b769f7d8d3`
- live run-policy RON `3c45b88cf0c0801e90277ea68840d0a65df72c0ecccba46984520f9bbf97cb31`
- server result transport `c42c486de1ad194ed9360892167311d47bc07aedb95a39eae31f0ff5464d2e07`
