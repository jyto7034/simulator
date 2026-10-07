# Preservation Asset And Migration-Seam Register

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Baseline: `BASELINE-20260822-064427-KST`
- Independent lock evidence: E-022

This register is intentionally parallel to `FINDINGS.md`. A pressure does not erase the value of the algorithms, data and
oracles behind it, and a preserved asset does not excuse an unsafe application boundary.

## Preservation Assets

| ID | Asset | Proven value | Evidence/oracle | Rewrite loss risk | Disposition |
| --- | --- | --- | --- | --- | --- |
| PA-001 | typed `GameCore` façade, `PlayerBehavior`/result/error and capability preflight | one host-facing authority and exhaustive feature entry | E-006~E-010/E-017/E-018 | high: behavior rediscovery and adapter churn | retain façade semantics while replacing internals/DTO adapter in slices |
| PA-002 | Facility graph/generator and `MapProgression` | deterministic topology, selection and staged completion | E-007/E-012/E-017 | high | retain; place behind explicit Run/Node ports |
| PA-003 | battle simulation/execution state, typed commands, ordered events and terminal outcomes | deterministic execution and rich characterization | E-008~E-012/E-018/E-020 | very high | retain kernel algorithms; narrow dependencies and session boundary |
| PA-004 | movement planner/input/engine/backend/blocking pipeline | clear algorithm seam and deterministic ordering | E-012 | high | retain/refactor policies; no subsystem rewrite |
| PA-005 | skill authoring validation, compiled combat profile and cast/delivery/interrupt runtime | correct separation of target/reach/retarget/delivery/epoch | E-010/E-014 | very high | retain; move activation policy to data and isolate compiled catalog |
| PA-006 | compiled Boss Runtime DSL plus scenario/setup validation | data-driven phase/task/summon/signal execution | E-011 | high | retain battle-local engine; add encounter/run orchestration outside it |
| PA-007 | Monolith/encounter ledgers and deterministic receipt/plan patterns | explicit progression authority and retry-safe plans | E-013/E-016 | high | retain; do not reuse as generic Relic state |
| PA-008 | reward/economy/inventory staged planners | prevalidated typed diffs and strong no-partial-effect tests | E-016~E-018 | high | retain and place inside common application transaction |
| PA-009 | live-RON parse/validate/compile/single-publication pipeline | fail-closed global data integrity and generated setup feasibility | E-011/E-019/E-023 | very high | retain pipeline; distinguish validated runtime type and publish consumer-scoped compiled contexts |
| PA-010 | strict V4 checkpoint graph validator and export/continue port | fail-closed restore, semantic recapture and public continuation equality | E-019/E-021 | very high | retain V4 importer/oracle; write a target aggregate DTO |
| PA-011 | seeded ordering, namespaced UUID and stable event/receipt identities | unrelated streams do not shift and save/retry can be compared | E-009/E-012~E-017/E-021 | very high | retain as explicit target contracts |
| PA-012 | broad test/live-content corpus | 1,131 passing characterization/contract tests at baseline | E-003/E-018 | very high | reuse as differential/release gates; do not rewrite alongside implementation |

## Migration Seams

| ID | Existing seam | First safe use | Single-authority rule | Main blocker/risk |
| --- | --- | --- | --- | --- |
| MS-001 | `GameCore::execute*` family and domain handler routing | introduce application use-case ports behind unchanged façade | one handler route selects old or new implementation, never both commit | broad root DTO/context |
| MS-002 | `with_state_transaction`, economy stages and tactical/battle staging helpers | generalize to classified Query/Run/Battle/Profile transaction | replacement port owns publish; old helper becomes adapter then is removed | handlers currently choose convention locally |
| MS-003 | `ResolvedCombatSetupPlan`, `BattleScenario`, `BattleExecutionState`, typed live command | construct a compiled battle context and kernel-local command/result | world maps at boundary; kernel no longer sees behavior DTO/full database | public/shared imports and large fixture context |
| MS-004 | live-data validated candidate and domain compilers | publish per-subsystem immutable compiled catalogs | raw/parsed data never becomes runtime authority | global cross-validation must remain available |
| MS-005 | purpose-built behavior DTO conversion sites | add versioned protocol envelope/capability adapters | only domain result is authoritative; DTO is derived | central `behavior.rs` and selective strictness |
| MS-006 | strict V4 export/continue plus canonical snapshots | one-shot V4-to-target checkpoint conversion | read old once, validate, publish only target writer | future feature fields and compatibility policy |
| MS-007 | event `seq`/revision/cursor | indexed bounded presentation archive behind same delta contract | event semantic order remains kernel authority | retention policy and result/debug consumers |
| MS-008 | public snapshots/results and test fixtures | old/new command-tape differential test | compare canonical observation, not internal state layouts | monotonic exceptions need explicit normalizer |

## Migration Blockers Or Required Gates

| ID | Blocker/gate | Why it matters | Can be removed incrementally? |
| --- | --- | --- | --- |
| MB-001 | no universal mutation transaction/conformance protocol | dual/partial publication risk during feature migration | yes: classify routes, install one port, migrate one handler family at a time |
| MB-002 | parallel root phase/session representations | a new node/application slice can temporarily create invalid combinations | yes: introduce one `ActiveNodeSession` sum type and adapt old projections |
| MB-003 | battle kernel sees full data/application models | boundary replacement retains broad compile/test context | yes: introduce compiled context and local models before any crate split |
| MB-004 | unbounded/scanned battle event history and full clone staging | new correctness layer could regress long-session cost | yes: index/archive and transaction internals behind preserved ports |
| MB-005 | no canonical all-feature continuation command tape | a big replacement cannot cheaply prove preserved meaning | yes: extend E-021 seam before migrating each feature family |
| MB-006 | product hard caps/cross-platform guarantee undecided | exact SLA/certification is unavailable | yes: GAP-001/002 are policy gates, not architecture blockers |

## Full-Rewrite Implication

No evidence shows that these seams are unusable. In particular, the façade, compiled setup, domain engines, validators,
checkpoint port and test oracles can all remain single authority while one application slice is replaced. A whole-core big bang
would simultaneously discard or reimplement nearly every PA item and remove the old/new oracle exactly when it is most needed.
