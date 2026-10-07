# P07 Core Public Projection Experiments

## 2026-08-23 — GEX-00 Intake

- Run mode: full Goal Set `goal-run`.
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P07-CORE-PUBLIC-PROJECTION)`.
- Execution node: standalone `WP-CRAR-C2-08`; no external transport or consumer work is authorized.
- P03, P04, P05 and P06 completion/currentness evidence was read before selection. All prerequisites are
  `Complete / High`.
- Existing exact-identity P07 execution document: none; canonical directory `execution/p07_core_public_projection`
  selected.
- Production/source mutation during intake: none.

### Actual Boundary Trace

- Reviewed the frozen Goal/Phase/policy/decision/technical-decision rows, root module visibility, `GameCore` public
  methods, the private application executor, command/result/error definitions, snapshot producers, admin/battle/profile
  and checkpoint routes, actual Core-owned binaries/probes and integration-test imports.
- Confirmed `GameCoreState` and commit authority are already private, while `GameCore` construction and all root
  mutation/query operations are public. The needed cutover is therefore a façade/visibility change, not a domain-state
  rewrite.
- Confirmed 58 characterized player action kinds and 49 actual result variants. The pre-edit characterized corpus had
  47 cases and omitted the staged/completed Mastered Elite sweep results; GEX-10 corrected the oracle before using it
  for parity. The existing result projection has exhaustive source coverage but collapses distinct battle meanings and
  publicly exposes domain-shaped contract families, so it is only the semantic oracle for P07.
- Confirmed no integration test constructs `GameCore`; Core-owned valid compile probes and one package fixture binary
  are the external call sites that require migration. Direct battle/data integration tests are not substitutes for the
  new public façade and are not external consumer evidence.
- Confirmed profile/checkpoint/admin/battle-clock routes must be wrapped before `world` visibility closes. Transaction
  rollback capture/restore is implementation authority and must be removed from the public surface rather than mapped.

### Baseline Execution

Commands from the simulator workspace root:

```text
env CARGO_TARGET_DIR=/tmp/crar-p07-gex00-target \
  cargo test -p game_core --lib every_behavior_result_has_exact_old_command_projection_contract -- --nocapture
env CARGO_TARGET_DIR=/tmp/crar-p07-gex00-target \
  cargo test -p game_core --lib every_action_kind_has_one_characterized_route_scope_and_authority_contract -- --nocapture
```

Result: both commands exit `0`; one matching test passed in each run, zero failed/ignored, 1,098 filtered out. This
freezes the pre-edit 47-case result corpus and 58-action semantic characterization before the public cutover. Source
inspection subsequently established 49 actual result variants, so the two missing cases became a mandatory GEX-10
correction rather than accepted baseline meaning. This is not P07 completion evidence.

### Feasibility and N/A Recheck

- A façade can own `GameCore` privately and invoke the existing P03 executor. No second commit port, lifecycle owner or
  external schema is needed.
- Existing snapshot JSON conversion can occur inside the private Core boundary, allowing immutable application
  documents without exporting DTO/domain fields or reading private aggregate state from the façade.
- P06 V5 bytes can pass through an application checkpoint wrapper unchanged; P07 need not inspect or invent any host
  envelope.
- External schema, server/Unity mapper/cutover, compatibility and consumer PASS remain evidence-backed N/A. This pass
  did not inspect excluded source, fixtures or sibling resources to manufacture evidence.
- P08 performance remains applicable but non-blocking here. No stop condition or unsafe dirty overlap was found.

### GEX-00 Result

- Result: `passed`.
- Approved next pass: GEX-10 application façade/types/mappers, full root hard cutover, exhaustive parity/rollback/API
  evidence, actual Core call-site migration, runtime-contract/manifest update and predecessor currentness.

## 2026-08-23 — GEX-10 Implementation and Correction

### Public Boundary Cutover

- Added root `application` with the sole public application service and private domain conversions. Result publication
  is infallible after commit: a serialization problem is `AcceptedRecoveryPending`, never an `Err` representing a
  committed operation as rejected.
- Closed the raw release root and mutation handles. `world`, `GameCore`, player behavior/result and legacy result
  contract families are crate-private; active-run rollback is test-only. `GameError` no longer derives domain serde.
- Migrated the package event fixture and the P02/P03/P04/P05 positive probes. Added independent P07 positive/negative
  probes. No excluded consumer or external schema was inspected to design the mapping.

### Oracle Corrections and Live Finding

- Source exhaustiveness showed 49 `BehaviorResult` variants, not the pre-edit corpus's 47. Added staged/completed
  Mastered Elite sweep cases before accepting parity. The corrected corpus binds all 49 meanings and payloads.
- The first live application reconnect probe showed that the established Core read-only reconnect deliberately reuses
  `BehaviorResult::BattleAdvanced`. The application mapper therefore preserves that result kind/payload and overrides
  disposition to `ReadOnly`; two calls at cursor 0 are exact-equal and do not mutate revision/history authority.

### GEX-10 Validation

- `cargo fmt --all -- --check`, `git diff --check`, default-release library check and release rustdoc: exit 0; release
  warnings 0. Rustdoc exposes the application module and no raw root/behavior/result/rollback/legacy-contract name.
- Exhaustive façade tests: 58 command bijection, 49 result parity, all 28 domain errors, all seven snapshots, malformed
  input, rollback/retry and committed-publication recovery semantics pass.
- V5 application checkpoint export into a fresh application continuation passes. Live validated-data application and
  live private-battle/reconnect probes pass.
- P01 manifest verifier: 57 features, 57 headings, 14 bundles, 10 dimensions, 10 legacy targets, 44 locator contents,
  composite `3c57aedd636a7010f0678dc57d4b6d31e819f3b184ef7ac50346785eca39325b`, errors 0.
- Five valid release probes exit 0. Five invalid release probes exit 101 with the expected builder/private-root/private-
  kernel/private-history/private-projection error families.
- Full `cargo test -p game_core --all-targets`: 1,106 library plus 51 integration tests, 1,157/1,157; failed/ignored 0.
- Direct P01–P06 successor records: `_007`, `_006`, `_005`, `_004`, `_003`, `_001`; all `Complete / High`.

### GEX-10 Result

- Result: exit criteria met.
- Next action: publish immutable P07 target/receipt, then begin a separate fresh GEX-20 audit without changing a frozen
  member. Any finding returns to correction GEX-10.

## 2026-08-22 — Correction GEX-10 after Master Audit

- Master GEX-30 Attempt 2 identified the pre-edit `src/game/world/snapshot.rs` identity
  `704ad2dc9baa8b7ffe1f5f2c16fb94418faf46bc8c1d432145870b3ef58aa0b6` and its runtime `cfg!(test …)` public
  projection switch as contradictory to P08's zero-count structural claim.
- Replaced the runtime switch with two mutually exclusive compile-time `#[cfg]` constants. The selected-event field,
  default-release value and enabled diagnostic-capability value are unchanged; no compatibility branch or second
  projection path was added.
- Focused validation: formatting/diff checks pass; runtime test/benchmark switch search is zero; default and
  `test-fixtures` release checks pass; combat-result snapshot 1/1 and application/authority 9/9 pass.
- Broad validation: release rustdoc and manifest verifier pass; manifest remains 57/57 with current locator composite
  `07f3b56b…fb8a7`; event log 8/8, result statistics 7/7 and checkpoint validation 16/16 pass; full Core is
  1,159/1,159 with failed/ignored zero.
- Five valid API callers exit 0; five established invalid callers and the default fixture caller exit expected 101.
- Fresh affected predecessor audits: P01 `_009`, P03 `_007`, P05 `_005`, P06 `_003`; unchanged P02 `_007` and P04
  `_005` remain current.
- Correction GEX-10 exit criteria are met. Next action: publish the replacement receipt/target, then start a separated
  fresh P07 GEX-20 without implementation mutation.
