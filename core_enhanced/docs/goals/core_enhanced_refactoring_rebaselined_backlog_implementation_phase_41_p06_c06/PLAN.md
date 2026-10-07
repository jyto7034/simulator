# Phase 41 CRA-P06-C06 Ghost Scenario Event Once Input Cleanup Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-11 instruction approves all remaining registered cleanup
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 41 of 44
Candidate / Work Type: CRA-P06-C06 / `cleanup`
Direct Linear And Technical Dependency: Phase 40 CRA-P07-C04 `Complete / High`
Direct Policy: CRA-POL-006
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

Remove the public `ScenarioEvent.once` input and all constructor/fixture writes only if a complete current inventory
confirms it remains unread and cannot express repeat behavior. Preserve exact one-time scenario enqueue semantics,
event ordering, group-spawn identity/deduplication, delayed-spawn and forced-end behavior, validation, live content,
serialization boundaries and every current scenario consumer.

## User/Contract-Visible Problem

Historical evidence reports that every workspace constructor writes `once: true` while neither runtime nor
validation reads it. That public field then advertises a repeat/one-shot choice which the engine does not implement:
scenario events are prospectively admitted once, while spawned groups have their own identity-based deduplication.

The report is locating evidence, not current deletion authority. After activation this Phase must reconstruct the
current definition, derives/serde, all constructors, deserialization/data paths, readers, validators, event queue and
spawn lifecycle, plus accessible external Rust consumers. Any real `false` authoring, defaulting, repeat semantic or
serialized consumer stops mechanical removal.

## Audit Finding ID

CRA-P06-C06. The rebaselined catalog classifies it `still_valid`, `cleanup`, Tier D, priority axes
`1/0/1/1/1/0/unknown = 4`. Exact linear item 41 requires CRA-POL-006 public/external inventory and exact constructor,
delayed-spawn, forced-end and scenario-validation behavior before field removal.

## Historical Evidence And Current-Proof Gate

- Original P06 audit: no runtime or validator reads the field; every workspace constructor writes `true`; scenario
  enqueue and group identity already own one-shot lifecycle.
- Post-Boss reaudit: the public field still existed unread and every constructor still wrote `true`; Boss scheduling
  did not create repeat-event semantics.
- Final catalog/order/test records: retain compile/behavior guards, inventory every constructor and prove delayed
  spawn, forced end and validation exact before removal.

None of these statements substitutes for current proof. Sole activation and prospective fingerprints precede current
production discovery, baselines or mutation.

## Source Of Truth And Ownership

### Prospective current authority

- Scenario event collection/enqueue prospectively owns admission and order.
- Runtime group identity/deduplication prospectively owns one-time spawn materialization.
- Scenario validation prospectively validates timing/actions without consulting `once`.
- Exact current definitions, data/serde reachability and consumers remain `discovery pending`.

### Target authority

The scenario model exposes only timing/action data the current runtime implements. Event enqueue/order and runtime
group identity remain the sole one-shot lifecycle authorities. No replacement boolean, mode enum, default, alias or
repeat-event abstraction is introduced.

## Invariants

- Every current scenario event is admitted, ordered and executed exactly once under the same current runtime rules.
- Delayed group spawn timing, group identity/deduplication and produced battle events remain exact.
- Forced battle end timing/outcome, same-time ordering and terminal behavior remain exact.
- Scenario/static validation acceptance and rejection remain exact.
- Every live RON scenario and current fixture continues to deserialize/compile/execute with no semantic migration.
- Boss scenario/setup/scheduling behavior and ordinary non-Boss scenarios remain exact.
- No repeating-event feature, event ordering change, new lifecycle flag, fallback, alias or ignored test is added.
- No Phase-42 battle-log I/O, Phase-43 range/delivery or Phase-44 package work is pulled forward.
- Existing dirty work and the protected Phase-27 marker remain exact.

## Refactor Candidate And Behavior-Preservation Boundary

- Candidate: CRA-P06-C06; `cleanup`; `still_valid`; Tier D; known score `4/18`.
- Prospective removal: current `ScenarioEvent.once` definition plus constructor, fixture/mock and serialized authoring
  writes proven to have no reader or distinct contract.
- Retained scope: scenario event timing/action, collection order, queue/runtime identity, group dedupe, validation,
  delayed-spawn/forced-end behavior and all live scenario data.
- First safety net: exact constructor/data/serde/reader matrix and nonzero delayed-spawn, forced-end and scenario
  validation controls before removal.
- Boundary: any current `false` value, read/default, repeated scheduling, DTO/save/deployed authoring consumer or
  semantic distinction stops cleanup and is not reinterpreted as ghost state.

## Approved Policy And Evidence

- CRA-POL-006 normally requires consumer inventory plus deprecation/versioned migration or explicit break approval.
- User approval on 2026-08-11 authorizes every remaining registered verified public/serde cleanup with no shim or
  deprecation pause after its own inventory and behavior guards.
- Original/post-Boss/final records authorize only mechanical field/write removal while one-shot lifecycle, delayed
  spawn, forced end, validation and ordering remain exact.
- Phase 40 audit is 229 lines at SHA-256 `ea94b13a...`, verdict `Complete / High`, and makes this Phase ready.

## Open Policy Decisions

None at initialization. Repeating scenario events are explicitly an excluded feature. If current discovery finds a
real repeat/once distinction or unknown serialized/deployed content consumer, stop that member and record the exact
contract rather than inventing a migration.

## In Scope

- exact current `ScenarioEvent` definition, visibility, derives, defaults and serde/data reachability;
- all literal/update/default constructors, fixture builders, mocks, tests, tools, docs and external consumers;
- all runtime/validator reads, pattern matches, enqueue/order/materialization and group-dedupe paths;
- complete actual-value inventory, including any `false`, omission/default or dynamic assignment;
- delayed-spawn, forced-end, same-time order, group dedupe and scenario-validation behavior before and after cleanup;
- live RON scenario loading/validation and Boss/ordinary scenario families if current coupling exists;
- public Rust/serde/package/deployment classification under CRA-POL-006;
- focused/broad debug/release/static validation and separate completion audit.

## Out Of Scope

- implementing repeating, periodic, reusable or rescheduled scenario events;
- changing scenario event timing/action schema, ordering priority, group identity or spawn deduplication;
- changing terminal/forced-end policy, battle result, Boss scheduling or scenario validation rules;
- Phase 42 battle-log file-I/O/debug helpers, Phase 43 range/delivery surfaces or Phase 44 package targets;
- DTO/wire, save/checkpoint, Unity/server, live content or canonical semantic migration;
- unrelated lint, formatting or dirty-worktree cleanup.

## Implementation Scope

1. initialize and solely activate this Child before current production inspection;
2. capture prospective filename/status/hash boundaries for scenario, data, tests and consumers;
3. enumerate exact definition/derive/default plus every constructor, value, reader, match and serialized consumer;
4. establish nonzero delayed-spawn, forced-end, validation, dedupe/order and live-loading baselines;
5. add or strengthen retained production-path controls only if current useful behavior lacks coverage;
6. remove only the verified ghost field and mechanical writes under the user's approval;
7. repeat exact searches, focused/broad profiles, workspace/static checks and dirty/marker verification;
8. perform a separate completion audit; make Phase 42 ready only on `Complete / High`.

## DTO, Data, Save And Migration

- DTO/wire migration: none planned.
- Live RON/data migration: no semantic migration planned; exact syntax impact remains discovery pending. If live RON
  explicitly writes the field, removal must update only that now-rejected ghost key after loader/consumer proof.
- Save/checkpoint migration: none planned; scenario definitions are prospectively static setup data, to be verified.
- Runtime event migration: none planned; queue/order and produced event shapes remain exact.
- Public Rust/serde migration: immediate field removal is approved only after current consumer/value inventory.
- Unity/server migration: none planned.

## Deployment And Compatibility

Core, `game_server`, in-workspace tools/tests/data and accessible repository consumers form one validation unit.
Package publishability, public derives, serialized authoring and accessible deployment evidence must be inventoried.
Unknown external existence alone does not prove use, but approval applies only after affirmative evidence that the
field carries no current contract. No dual shape, alias, ignored key, compatibility fallback or deprecation wrapper
is authorized.

## Focused Validation

- exact constructor/value/reader/serde/public-consumer disposition matrix;
- delayed-spawn and group identity/deduplication controls;
- forced-end and same-time scenario event ordering controls;
- scenario validation acceptance/rejection and live RON load controls;
- Boss/ordinary scenario controls selected by current coupling;
- exact removed-field/placeholder/public search and test-list delta.

Every accepted command records selected/pass/fail/ignored counts. Zero-selected filters or compile-only success do
not prove behavior.

## Broad Validation

- complete scenario, event, battle lifecycle/result and live-data families in debug; representative release;
- `cargo test -p game_core --all-targets` in debug and release with exact distribution;
- workspace all-target checks in debug and release;
- normal Core Clippy, `cargo fmt --all -- --check` and whole-worktree `git diff --check`;
- final test-list/member reconstruction, target/non-target hashes and protected-marker verification.

## Omitted Validation

- server/Unity runtime probes are omitted only if no DTO/wire/mapping shape changes and current searches prove no
  transport coupling; otherwise they become mandatory.
- save migration probes are omitted only if current inventory proves scenario definitions are not persisted.
- canonical edits are omitted only if removal is false input shape with no gameplay or serialized contract change.
- repository-wide Stage-D integration remains a Master task after Phase 44.

## Completion Conditions

- Every current constructor/value/read/default/serde/public consumer is named and classified.
- No current `false`, omitted-default, repeated scheduling or semantic reader exists for the removed field.
- Retained production-path delayed-spawn, forced-end, validation, order and dedupe tests pass before removal.
- The field and all mechanical writes are absent with no replacement placeholder or compatibility shim.
- Live scenarios, Boss/ordinary setup and runtime invariants remain exact.
- No Phase-42+ concern or unrelated dirty work is changed.
- Focused/broad debug/release/static checks are green with exact counts and failure classification.
- Non-target hashes and protected marker are exact.
- A separate completion audit returns `Complete / High`.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P41-C01 | current definition/public graph is exact | definition, derives/defaults, readers, matches and consumers | pending |
| P41-C02 | one-shot authority graph is exact | scenario collection/enqueue/order → runtime identity/dedupe | pending |
| P41-C03 | dirty work is preserved | activation/current hashes, scoped diffs and marker | pending |
| P41-C04 | every constructor/value is classified | exhaustive workspace/data/external matrix | pending |
| P41-C05 | current behavior is characterized | delayed-spawn, forced-end, validation, order and dedupe baselines | pending |
| P41-C06 | useful assertions precede removal | retained production controls green with field present | pending |
| P41-C07 | public/serde gate is exact | consumer/package/deployment inventory and approval mapping | pending |
| P41-C08 | only ghost input is removed | exact field/write/test/data delta; no replacement | pending |
| P41-C09 | scenario behavior is exact | pre/post focused families and live content | pending |
| P41-C10 | DTO/data/save/server/Unity boundary is exact | hashes/searches/affected package checks | pending |
| P41-C11 | broad/static validation is green | both profiles, workspace, format/diff and Clippy | pending |
| P41-C12 | independent audit/next boundary is current | `Complete / High`; Phase 42 absent until exit | pending |

## Stop Conditions

Stop and do not activate Phase 42 if the field has any semantic reader, any current/dynamic `false`, omitted default
with distinct meaning, repeat scheduling contract, unknown serialized/deployed consumer, required migration beyond
approval, changed delayed-spawn/forced-end/order/dedupe/validation behavior, dirty overlap that cannot be isolated, a
Phase-42+ change, zero-selected mandatory test, changed marker, or audit verdict below `Complete / High`.

## Initialization Dirty Baseline

- Expanded status before this Child: 585 rows at canonical C-sort SHA-256
  `54b165d79d624f7ff6e992ddb7d77e238b8be38f1b2333c993ca4064a51c79da`.
- Phase 41 path was absent and active implementation Child count was zero.
- Phase 40 completion audit is 229 lines at SHA-256
  `ea94b13a10a54060d9a8e31d13d4c3b44f5dfaa4d8fb6df89e4c2d51bb56ef0d`, verdict `Complete / High`.
- Protected marker is 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`.
- These three new Child documents are the only expected status-set additions before activation evidence.
