# P07 Core Public Projection Completion Audit

Current verdict: `Complete / High — GEX-20 correction audit 002`

## Audit Identity and Separation

- Handoff / Goal Set: `HA-CRAR-P2G60-002` / `GS-CRAR-P2G50-002`.
- Audited member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P07-CORE-PUBLIC-PROJECTION)`; Phase / node
  `PHASE-CRAR-C2-07-CORE-PUBLIC-PROJECTION` / `WP-CRAR-C2-08`; Goal Quality Contract `2`.
- Policy / decisions: `PER-CRAR-007-R6-CODE-REVIEWED`, `DR-CRAR-007-R2`, realization `TD-CRAR-017`, durable seam
  `IVA-C2-08`.
- Frozen GEX-10 target: `ATR-CRAR-C2-P07-GEX10-001` / `evidence/P07_AUDIT_TARGET.json`, SHA-256
  `44d81414fe5c13e143091c9e13085f986428871f01b68359567fda63789859ae`.
- GEX-20 began only after target publication. It changed no frozen member; generated build outputs and this explicitly
  excluded completion document are outside the target.

## Target, Scope and Predecessor Currentness

- Repository revision/branch remains `f318414fb0156a67e04c5479000c2a2683fc0c09` /
  `cleanup/simulator-dirty-worktree-20260803`.
- Target and receipt JSON parse; declared content count is exact; all 68/68 members match their frozen SHA-256 before
  and after fresh GEX-20 validation.
- Core-only unrelated dirty baseline is exact after cumulative P01–P07 target/evidence/probe exclusions: 82 modified,
  23 deleted, 679 untracked, total 784; fingerprint
  `f3e50fe9dd9488f86cc338923a276ff4680aa93daba24b26087028085de9efba`.
- P01 `_007`, P02 `_006`, P03 `_005`, P04 `_004`, P05 `_003` and P06 `_001` are direct `Complete / High`
  successor-currentness records. No predecessor evidence is silently reused across a changed member.

## Completion Condition Audit

| Condition | Direct evidence | Result |
| --- | --- | --- |
| one public application root | `application::CoreApplication` owns one private `GameCore`; no borrow/conversion/inner field | pass |
| command totality | 58 `ApplicationCommandKind` values form an exact inverse bijection with 58 domain actions | pass |
| result totality | 49/49 private results map to distinct application kind/meaning/payload; twelve battle identities retained | pass |
| corrected oracle | staged/completed Mastered Elite cases close the pre-edit 47-case corpus omission | pass |
| error totality | all 28 domain errors map exhaustively plus four application-local input/projection kinds | pass |
| snapshot totality | all seven snapshot kinds are typed, immutable and exact on retry; absent selected event is `null` | pass |
| transaction meaning | invalid domain command/retry leaves exact snapshot; committed projection issue is recovery-pending, never `Err` | pass |
| cursor/reconnect meaning | legacy `BattleAdvanced` payload identity retained with `ReadOnly`; repeat cursor projection exact | pass |
| checkpoint restart | P06 V5 bytes/metadata export and continue through a fresh application to the next map decision | pass |
| actual callers | package fixture and P02/P03/P04/P05/P07 real positive callers use application ports | pass |
| raw authority absence | release cannot name `GameCore`, raw behavior/result, rollback token, legacy result contracts or inner field | pass |
| manifest/contract | removed locator plus canonical replacement verify; actual Core application API is documented | pass |
| external claim absence | external schema/adapter/compatibility/consumer inspection and PASS rows are zero | pass |

`ApplicationResultKind` preserves the established payload meaning. `ApplicationResultDisposition` records whether the
specific port invocation mutates authority, which is why reconnect is correctly `BattleAdvanced + ReadOnly`.

## Validation Dimensions

| Dimension | Audited evidence / N/A basis | Result |
| --- | --- | --- |
| `success` | live validated application start, private battle, result and seven snapshots; 58/49 exhaustive maps | pass |
| `invalid_input` | unknown/malformed/missing commands and every typed domain rejection occur before authority access/commit | pass |
| `failure_rollback` | rejected command preserves exact snapshot; post-commit projection cannot be returned as rejection | pass |
| `retry_idempotency` | repeated rejection, every snapshot and reconnect are exact-equal | pass |
| `ordering_concurrency` | one P03 writer; established revision/cursor/event ordering corpus remains green | pass |
| `boundary_empty_max` | explicit null plus existing exact history/result/checkpoint empty/max producers cross unchanged ports | pass |
| `disconnect_restart_persistence` | read-only resync and V5 export into fresh application continuation | pass |
| `live_data` | actual `ValidatedGameData::load_live_embedded` application and live private-battle probes | pass |
| `producer_consumer` | five independent default-release Core callers consume only public application/data ports | pass |
| `legacy_fallback_absence` | rustdoc/static/compile-fail audits show raw root/result/rollback/dual façade count 0 | pass |

The Goal table marked live-data loading itself N/A because P02 owns publication. P07 nevertheless uses the actual
already-validated embedded publication in its caller evidence; it does not claim ownership of authoring-data loading.

## Fresh GEX-20 Validation

- `cargo fmt --all -- --check`: exit 0.
- `git diff --check`: exit 0.
- Fresh `cargo check -p game_core --release --lib`: exit 0, warnings 0.
- Fresh application/authority focused selector: 9/9 passed.
- Fresh 49-result semantic parity and public V5 checkpoint continuation selectors: 1/1 each passed.
- Fresh release rustdoc: exit 0; application module is public; forbidden raw root/result/rollback/legacy-contract names
  absent from the public index.
- Fresh manifest verifier: 57 features, 57 headings, 14 bundles, 10 dimensions, 10 legacy targets, 44 locator contents,
  composite `3c57aedd636a7010f0678dc57d4b6d31e819f3b184ef7ac50346785eca39325b`, errors 0.
- Five fresh valid release probes: all exit 0, including live private battle and exact reconnect retry.
- Five fresh invalid release probes: all expected exit 101 with their exact capability/privacy error families.
- Fresh `cargo test -p game_core --all-targets`: library 1,106 plus integrations 51 = 1,157 passed;
  failed/ignored 0.
- Post-validation target comparison: 68/68 exact; unrelated dirty baseline exact.

## Quality, Scope and Limitations

- Application documents are replace-whole value objects. No application result, snapshot or checkpoint gives access to
  authoritative fields, mutation tokens or private domain enums.
- Compiler-exhaustive matches and cardinality/parity corpora are both required. Serde success alone is not used as
  semantic mapping evidence.
- No production branch depends on a probe identity, fixture value or test name. No lint suppression or compatibility
  shim was added.
- P08 remains the sole owner of final like-for-like performance comparison, permanent numeric/allocation/memory guards,
  optimization and performance PASS. P07 makes no performance conclusion.
- Whole-library Clippy remains blocked by the pre-existing denied `never_loop` outside P07; required release and
  rustdoc builds are warning-free.
- Dirty user work outside exact frozen/cumulative evidence members is preserved. `game_server`, Unity, sibling
  resources and external consumers were neither inspected nor used as evidence.
- Completion-blocking omission, unsafe overlap, policy/Goal/Phase change, external dependency ambiguity or automatic
  stop condition: none.

## Verdict and Next Boundary

`GOAL-CRAR-C2-P07-CORE-PUBLIC-PROJECTION` is `Complete / High` on
`ATR-CRAR-C2-P07-GEX10-001`, currentness `current / unchanged`.

Strict sequence makes P08 performance/final integration eligible for its own GEX-00. This verdict does not complete or
pre-authorize P08, and P07 requires a direct successor-currentness review if P08 changes the application surface,
mapping, visibility, read-model semantics or frozen evidence members.

## GEX-20 Correction Audit 002 — `Complete / High`

### Replacement Target and Separation

- Trigger: Master GEX-30 Attempt 2 found one runtime `cfg!(test …)` projection switch contradicting P08 structural
  evidence and returned the goal-run to P07 correction GEX-10.
- Replacement target: `ATR-CRAR-C2-P07-GEX10-002` / `evidence/P07_AUDIT_TARGET_002.json`, SHA-256
  `9fc894b57107eaf043e5601e6ff1155f9a73cdf24f8968586e539150d7ae2755`.
- The correction pass ended and froze the target before this fresh read-only audit. Initial and final comparisons are
  correction members 21/21 and integrated P08 submanifest 95/95 exact. This completion document and audit output are
  explicit target exclusions.
- No production/source/data/test/tool/manifest mutation occurred in this GEX-20.

### Correction and Completion Evidence

- `src/game/world/snapshot.rs` now uses mutually exclusive compile-time `#[cfg]` constants. The runtime
  `cfg!(test/benchmark)` source count is zero; default release remains false and enabled debug/test/fixture capability
  remains true. No public field, public authority, fallback, compatibility path or external meaning changed.
- The new combat-result assertion directly fixes the enabled test configuration. Snapshot selector 1/1 and
  application/authority 9/9 pass.
- Affected predecessor re-audits P01 `_009`, P03 `_007`, P05 `_005`, P06 `_003` are `Complete / High`; unchanged P02
  `_007` and P04 `_005` remain current.
- Fresh default and `test-fixtures` release checks and release rustdoc pass with zero warnings and no forbidden raw or
  fixture API names.
- Fresh manifest verifier passes 57/57 features, ten dimensions, 44 contents, current composite `07f3b56b…fb8a7`,
  errors zero.
- Fresh five valid API callers exit 0; five established invalid callers and default fixture caller exit expected 101.
- Fresh full `cargo test -p game_core --all-targets`: library 1,108 plus integrations 51 = 1,159 passed; failed/ignored
  zero.

### Dimension and Guardrail Recheck

All ten P07 dimensions retain their original direct evidence. The changed path is specifically relevant to
`producer_consumer` and `legacy_fallback_absence`: both default and feature configurations compile, public snapshot
meaning is stable, and no runtime test identity switch remains. Invalid input, rollback, retry, ordering, empty/max,
restart, live data and the remaining public mapping evidence are unchanged and freshly covered by the application/full
suite and API probes.

Implementation Quality Guardrails now pass direct source review. There is no test-name/value branch, silent recovery,
duplicate authority, future abstraction, timing workaround, fallback, or unreviewed compatibility path.

### Current Verdict

`GOAL-CRAR-C2-P07-CORE-PUBLIC-PROJECTION` is `Complete / High` on
`ATR-CRAR-C2-P07-GEX10-002`, currentness `current / unchanged` at audit exit.

P08 is eligible for a fresh currentness/completion audit. The historical P07 target/verdict and Master Attempt 2
remain preserved and are not rewritten as the current target.
