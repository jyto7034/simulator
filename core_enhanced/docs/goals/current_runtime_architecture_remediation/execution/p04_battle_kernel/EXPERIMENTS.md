# P04 Battle Kernel Experiments

## 2026-08-23 — GEX-00 Intake

- Run mode: full Goal Set `goal-run`.
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P04-BATTLE-KERNEL)`.
- Execution node: standalone `WP-CRAR-C2-05`; no atomic-group split or parallel Phase is authorized.
- P01, P02 and P03 completion/currentness evidence was read before selection. All prerequisites are `Complete / High`.
- Existing exact-identity P04 execution document: none; canonical directory `execution/p04_battle_kernel` selected.
- Production/source mutation and test execution during intake: none.

### Actual Boundary Trace

- Reviewed current `battle::core` module export, `BattleCore`, `RuntimeUnit`, `BattleEvent` comparator/queue,
  `WorldVec2`/`UnitBody`, simulation mutation ports, world active-battle owner and integration skill/runtime callers.
- Confirmed the exported kernel can be independently constructed from validated data, its public fields directly
  mutated, and raw events injected without the application lifecycle. The actual world owner does not currently use an
  external bypass; the defect is capability/type-checking, not a claimed live corruption incident.
- Confirmed `BattleEvent` contains float-bearing projectile impact and area center payloads and manually asserts `Eq`.
  `WorldVec2::new`, public coordinates and derived deserialization admit NaN/infinity before heap ordering.
- Confirmed production world commands already use integer tile positions, enum playback speed and validated authored
  integer scales. The raw-float kernel surface can be removed without changing command/result semantics.
- Confirmed existing integration fixtures depend on the broad kernel only in test/debug support. The repository already
  uses `debug_assertions`/`test-fixtures` as a non-default capability gate, so the oracle can remain executable while a
  default release compile-fail audit proves production inaccessibility.

### Feasibility and N/A Recheck

- Conditional module visibility provides a hard default-release cut without moving or duplicating algorithms. Sibling
  Core owners retain crate access and no external adapter is needed.
- A checked finite position type plus checked scalar constructors can reject raw non-finite input before queue/unit
  mutation. Official integer-derived positions/speeds are representable and require no normalization.
- `disconnect_restart_persistence` is N/A for this Goal because kernel privacy/finite input changes no save, checkpoint,
  reconnect or history lifetime. Active-run and disconnect application routing remains P03 evidence; P05/P06 own the
  later lifetime/format changes.
- `live_data` and `producer_consumer` remain applicable: actual validated data must construct the private kernel through
  `GameCore`, and live boss/movement/skill paths must pass.
- Bitwise cross-platform replay is out of scope. Stable current-machine ordering/determinism and exact finite semantic
  parity remain required.
- No policy/Goal/Phase change, external dependency, unsafe dirty overlap or stop condition was found.

### GEX-00 Result

- Result: `passed`.
- Approved next pass: GEX-10 hard privacy cut, finite boundary implementation, focused/full validation, predecessor
  successor-currentness records and frozen P04 Audit Target.

## 2026-08-23 — GEX-10 Implementation and Direct Validation

### Privacy Cut and Producer Trace

- Changed `battle::core` to crate-private in default release and retained its broad test oracle only under the existing
  test/debug/test-fixtures capability condition. Internal world ownership was not duplicated.
- The valid release probe loads embedded authoring data into `ValidatedGameData`, creates `GameCore`, starts a run,
  advances the returned visual-novel session, selects live starters and a live combat/boss node, confirms entry, and
  observes the public `InBattle` snapshot state.
- The invalid release probe tries to import `BattleCore`/`WorldVec2`, clear units, enqueue a raw `BattleEvent`, and use
  unchecked NaN/infinity coordinates. Compilation fails at the private module boundary with `E0603`.

### Finite Boundary and Rollback Matrix

- Added typed finite position-component, radius, speed and time errors. Raw public coordinate fields and raw public
  construction are absent in default release; `WorldVec2` custom deserialization delegates to the finite gate.
- Both scenario group spawn paths validate movement radius/speed before incrementing spawn order or mutating placement
  and unit authority. Simulation milliseconds convert through `FiniteBattleSeconds`.
- Twelve position/radius/speed cases and three time cases cover NaN/positive infinity/negative infinity. For each case,
  the first and retry errors are equal and a complete authority snapshot—units, event heap/order, serialized log,
  battlefield, all sequence counters, seed and recording stacks—remains exactly equal.
- Finite and signed-zero `WorldVec2` values round-trip. RON rejects non-finite numeric spelling at its parser, while a
  `MapDeserializer` carrying actual NaN/infinity proves the custom deserializer itself also rejects each component.

### Differential and Full Oracles

- Same-bucket event insertion permutation, boss authored-priority independence and default movement insertion-order
  repeatability remain green.
- `skill_runtime_contract` 14/14 and `skill_test_suite` 4/4 remain green.
- P01 manifest verifier remains 57 features, 42 located contents and zero errors with current composite
  `b6999623897695ac9d84474392df8682d1d5bb78160c21533747b2454d1f3cde`.
- P02 valid release probe exits 0; invalid probe exits the expected 101 with `E0432/E0603/E0277/E0599`.
- P03 frozen member comparison is 32/33 unchanged; only the expected P04 addition to the canonical runtime contract
  differs, while every executor/lifecycle/source/probe member is byte-identical.
- Full `cargo test -p game_core`: library 1,087 plus integrations 51 = 1,138 passed, zero failed/ignored.

### GEX-10 Result

- Result: `exit criteria met`.
- No stop condition or policy/Goal/Phase change occurred. Exact target publication is the final GEX-10 mutation; a
  separate fresh GEX-20 pass will be read-only except for its excluded completion audit.
