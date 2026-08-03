# Documentation–Code Alignment Reaudit Experiments

Status: complete

Document class: non-canonical execution log

Started: 2026-07-18

Record searches, inspections, tests, probes, failures, environmental constraints, and reruns in
chronological order. This file records evidence collection; it does not define current behavior or
policy.

## Recording Format

```md
## EXP-<number> — <Purpose>

Date/time:

Claim/finding IDs:

Command or inspection:

Expected evidence:

Result:

Interpretation:

Files changed:
- none / goal-document paths only

Next action:
```

## EXP-000 — Goal Initialization

Date/time: 2026-07-18 KST

Claim/finding IDs: none

Command or inspection:

- Read the current documentation index, engineering workflow, goal completion review guide, and
  historical documentation/code alignment audit header.
- Inspected existing audit goal conventions and the current dirty worktree.

Expected evidence:

- Establish the current governance, source-of-truth order, audit output format, and safe location for
  resumable notes before beginning code inspection.

Result:

- Current governance assigns durable policy to the top-level canonical documents and treats goal
  files as working memory/history.
- `EXPERIMENT_NOTES.md` is the established location for source-of-truth conflicts, judgments, policy
  questions, and out-of-scope follow-up candidates.
- The earlier `documentation_runtime_contract_audit` references removed or superseded document paths
  and is not suitable as the active plan for this reaudit.
- The worktree contains extensive pre-existing modified, deleted, and untracked files across Core,
  data, server, and docs. They are preserved and are not attributed to this goal.

Interpretation:

- A new audit-only goal is appropriate. It must inspect the current working tree as the behavior
  under review while avoiding edits outside its own goal directory.
- Historical audit results can be used as regression prompts but not inherited as current verdicts.

Files changed:

- `docs/goals/documentation_code_alignment_reaudit/PLAN.md`
- `docs/goals/documentation_code_alignment_reaudit/EXPERIMENT_NOTES.md`
- `docs/goals/documentation_code_alignment_reaudit/EXPERIMENTS.md`

Next action:

- Build the Pass 0 document-section and implementation-surface inventory.

## EXP-001 — Pass 0 Inventory And Focused Baseline

Date/time: 2026-07-18 KST

Claim/finding IDs: all initial claim groups; DCA2-DATA-CLAIM-001,
DCA2-DATA-CLAIM-005, DCA2-EVID-CLAIM-001

Command or inspection:

- Enumerated headings in every primary durable, specialized content, and decision document.
- Enumerated current files under `core_enhanced/src/game`, `core_enhanced/tests`,
  `game_resources/data`, `game_server/src`, and relevant external Unity docs/Assets paths.
- Ran `cargo check -p game_core`.
- Ran `cargo test -p game_core --test ron_loading -- --nocapture`.
- Ran `cargo test -p game_core --test live_skill_catalog_audit -- --nocapture`.

Expected evidence:

- Establish complete section-level audit routing and a clean focused baseline before interpreting
  later failures as alignment findings.
- Confirm that the official embedded live bundle and the published live-skill membership manifest
  currently pass their executable gates.

Result:

- The initial coverage ledger contains 41 claim groups spanning governance, gameplay, battle,
  runtime, data, evidence, content, workflow, probes, and decision records. Claims will be split
  further when tracing reveals independent producers or outcomes.
- Current live data and `GameplayRonSourceSet::embedded` consist of `bundle.ron` plus 21 gameplay
  RON sources. The durable data contract says 23, producing finding `DCA2-DATA-001`.
- Core, server, and external Unity producer/consumer files are present. No separate
  `game_server/tests` directory was found; server tests are expected inline in current modules.
- `cargo check -p game_core`: pass.
- `ron_loading`: 20 passed, 0 failed, 0 ignored.
- `live_skill_catalog_audit`: 4 passed, 0 failed, 0 ignored.
- An initial attempt to start the three Cargo commands concurrently caused Cargo package/build lock
  contention. The commands were rerun or observed sequentially; the final results above are the
  authoritative baseline.

Interpretation:

- The green loading suite establishes the specific asserted live-data contracts, not full document
  alignment or full behavior coverage.
- The green manifest audit establishes catalog membership/delivery/reference assertions only, as
  the evidence contract itself states.
- Inventory is sufficient to start claim-level source tracing; no alignment verdict is issued from
  file existence or green baselines alone.

Files changed:

- `docs/goals/documentation_code_alignment_reaudit/EXPERIMENT_NOTES.md`
- `docs/goals/documentation_code_alignment_reaudit/EXPERIMENTS.md`

Next action:

- Trace the data loader/publication and skill-manifest claims already covered by the baseline, then
  begin gameplay/run-flow claims from the rulebook.

## EXP-002 — Modes, Starter Recruitment, And Map Progression

Date/time: 2026-07-18 KST

Claim/finding IDs: DCA2-GAME-CLAIM-001, DCA2-GAME-CLAIM-002,
DCA2-GAME-CLAIM-003

Command or inspection:

- Read the complete gameplay rulebook and traced mode/start/map claims through `world.rs`,
  `behavior.rs`, `map/progression.rs`, `map/types.rs`, `map/generator.rs`, `recruitment.rs`, employee
  data, run/map live policy, node completion, support Gate effect, and relevant snapshots.
- Ran `cargo test -p game_core game::world::tests::snapshots_and_start --lib --
  --test-threads=1`.
- Ran `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1`.
- Ran `cargo test -p game_core game::world::tests::node_sessions --lib -- --test-threads=1`.

Expected evidence:

- Prove required game-mode propagation and termination rules, unified employee identity/ownership,
  and Core-owned map/Gate progression at runtime rather than from document wording alone.

Result:

- `snapshots_and_start`: 22 passed.
- `map_flow`: 32 passed.
- `node_sessions`: 16 passed.
- Runtime, live policy, projections, and focused assertions agree with the three claim groups.
- The first start-test invocation yielded while compiling and was later polled to completion; its
  authoritative result is the 22-test pass above.

Interpretation:

- DCA2-GAME-CLAIM-001/002/003 are `Aligned`. Green tests were paired with direct producer/state
  tracing and are not used as standalone proof.

Files changed:

- Goal audit documents only.

Next action:

- Audit Event/inventory/consumable/shop/briefing and non-combat node claims.

## EXP-003 — Event, Inventory, Shop, Briefing, And Non-Combat Nodes

Date/time: 2026-07-18 KST

Claim/finding IDs: DCA2-GAME-CLAIM-004, DCA2-GAME-CLAIM-005,
DCA2-GAME-CLAIM-010

Command or inspection:

- Traced Event session/check/grant/combat code; ordinary inventory reserve and definition-stack
  code; consumable validation/use/result-duration code; Shop page/quote/purchase/sale/refresh code;
  combat-preview versus spawn-wave ownership; Support/Maintenance/Headquarters state and capability
  paths.
- Ran focused equipment, support, Shop, and committed-consumable-duration tests.

Result:

- `game::world::tests::equipment`: 26 passed.
- `game::world::tests::support`: 12 passed.
- `game::world::shop::tests`: 6 passed.
- `consumable_modifier_survives_abandonment_and_expires_on_committed_result`: 1 passed.
- Existing `map_flow` and `node_sessions` passes provide the adjacent Event, preview, shop-session,
  Headquarters, and planning-return assertions.

Interpretation:

- The three claim groups are `Aligned`; no new material drift was found.

Files changed:

- Goal audit documents only.

Next action:

- Audit battle entry/player-visible combat, reward/research/growth, resolution/failure, Psychology,
  and BossOmen claims.

## EXP-004 — Battle Gameplay, Growth, Failure, Psychology, And BossOmen

Date/time: 2026-07-18 KST

Claim/finding IDs: DCA2-GAME-CLAIM-006, DCA2-GAME-CLAIM-008,
DCA2-GAME-CLAIM-009, DCA2-GAME-CLAIM-011, DCA2-GAME-CLAIM-012,
DCA2-GAME-CLAIM-013; DCA2-GAME-001, DCA2-GAME-002, DCA2-TEST-001

Command or inspection:

- Traced battle mission/setup/deployment/wave policy; reward/growth/research/compendium; staged
  result/retreat/failure/condition; Psychology; BossOmen state/map/result/recovery; equipment final
  stat and skill-fragment profile application.
- Searched removed combat variants and block/fragment production vocabulary in code and live RON.
- Ran the complete world combat module plus Psychology, reward, and abnormality-research unit tests.

Result:

- `game::world::tests::combat`: 59 passed.
- `game::employee_psychology::tests`: 5 passed.
- `game::reward::tests`: 2 passed.
- `game::abnormality_research::tests`: 2 passed.
- Battle, resolution, Psychology and BossOmen claims align with runtime/data/tests.
- Two gameplay-policy mismatches and one direct regression-test gap were identified and recorded.

Interpretation:

- Equipment block effects and active-only fragment semantics cannot be corrected within this audit;
  they require an explicit gameplay/content policy choice.
- Direct code tracing supports the Endless completion branch, but absence of an exact test is not
  treated as executable proof, producing `DCA2-TEST-001`.

Files changed:

- Goal audit documents only.

Next action:

- Begin Pass 2 targeting and runtime lifecycle audit; split range/weapon archetype claims where
  independent drift appears.

## EXP-005 — Targeting, Cast, Projectile, And TileArea

Date/time: 2026-07-18 KST

Claim/finding IDs: DCA2-BATTLE-CLAIM-001 through DCA2-BATTLE-CLAIM-006;
DCA2-BATTLE-001, DCA2-BATTLE-002

Command or inspection:

- Read the complete targeting canonical and traced effective basic range, range preview, normal
  corroded presets, abnormality fallbacks, ranged route behavior, usefulness, cast/step targeting,
  interrupt epochs, basic/skill projectile runtime, TileArea geometry and legacy vocabulary.
- Compared live RetargetOnStep occurrences, abnormality ranges, equipment archetype use, and live
  delivery classification.
- Ran Core battle, targeting, range-preview, and skill-runtime integration suites.

Result:

- Core battle tests: 75 passed.
- Targeting tests: 11 passed.
- Range preview tests: 7 passed.
- `skill_runtime_contract`: 13 passed.
- Usefulness, cast/step, interrupt/projectile, and TileArea claims align.
- Abnormality generic range fallback remains contrary to the canonical but explicitly
  user-deferred; GrenadeLauncher remains enum-only while docs classify it as future.
- Core-side projection is traced, but Unity consumption is deferred to the wire pass.

Interpretation:

- DCA2-BATTLE-CLAIM-001 is `Policy decision needed`; claims 002–005 are `Aligned`; claim 006 remains
  `Insufficient evidence` pending external producer/consumer inspection.

Files changed:

- Goal audit documents only.

Next action:

- Audit the runtime canonical: RNG identity, event/checkpoint, movement/route, result/diagnostics,
  atomic commit/recovery, VN and projection ownership.

## EXP-006 — Core Runtime Lifecycle And Projection Producers

Date/time: 2026-07-18 KST

Claim/finding IDs: DCA2-RUNTIME-CLAIM-001 through DCA2-RUNTIME-CLAIM-010;
DCA2-DOC-CLAIM-001, DCA2-DOC-001

Command or inspection:

- Read the complete runtime canonical and traced deterministic helpers/RNG regressions, event log,
  checkpoint filtering, setup routes, movement/collision, result aggregation, developer trace,
  staged commit/recovery, research/compendium, inventory/shop/reward and top-level VN trigger paths.
- Searched live code/data and durable docs for removed `battle_records`, delta aliases,
  `default_blocking`, obsolete research delivery and Shop vocabulary, plus historical goal labels.
- Ran focused deterministic, event-log, result-stats and compendium unit tests.

Result:

- Determinism tests: 3 passed; event-log-sequence independence regressions: 2 passed.
- Event-log tests: 2 passed; result-stats tests: 4 passed; compendium tests: 2 passed.
- Core runtime claims for determinism, result/diagnostics, atomic commit, research/compendium,
  inventory/shop, reward/growth and VN lifecycle align with code/data/tests.
- Event/checkpoint, route/movement and final projection claims remain open until game_server and
  external Unity consumers are traced.
- The durable runtime canonical uses historical `G15`/`G24` goal labels as if they were current
  domain vocabulary, producing `DCA2-DOC-001`.

Interpretation:

- Green Core producer evidence cannot close an explicitly cross-layer presentation claim. Those
  rows remain `tracing` rather than being overclaimed as aligned.
- The historical labels do not change runtime behavior, but they violate the repository's durable
  document/goal-history separation and should be corrected in a later documentation goal.

Files changed:

- Goal audit documents only.

Next action:

- Begin Pass 3 strict authoring and live-content audit, then trace the open projection seams through
  game_server and Unity in Pass 4.

## EXP-007 — Strict Authoring, Map Catalogs, And Live Content

Date/time: 2026-07-18 KST

Claim/finding IDs: DCA2-DATA-CLAIM-002 through DCA2-DATA-CLAIM-004;
DCA2-CONTENT-CLAIM-001 through DCA2-CONTENT-CLAIM-003; DCA2-DATA-002,
DCA2-CONTENT-001

Command or inspection:

- Read the complete data canonical and all four specialized content documents, including the full
  per-abnormality design notes.
- Traced every embedded source, raw/domain parser, cross-catalog validation path, map/battlefield
  parser and generated-preview publication gate.
- Built a temporary `/tmp` crate linked to current `game_core` to test unknown nested fields through
  production-shaped abnormality, skill, equipment and map databases and to derive live
  equipment/fragment counts and effect summaries. Repository code was not modified.
- Compared all 66 indexed placeholder equipment IDs with live RON and reviewed every fragment
  rarity/effect line from loader-backed output.
- Ran all Core data, map, combat-preview and live item-skill activation tests.

Result:

- The probe accepted unknown nested gameplay fields in `MovementDef`, `ResonanceDef`,
  `TileRangePattern`, `DeliveryDef`, `SkillEffectDef`, `StatModifier` and `MapNodePayload`, including
  through all four outer production databases. This reopens historical DCA-H-04/PD-09/G12 as
  `DCA2-DATA-002`.
- `game::data`: 125 passed; `game::map`: 23 passed; `game::combat_preview`: 25 passed;
  `live_item_skill_activation`: 3 passed.
- Map/catalog ownership and semantic validation align; the nested unknown-field gap is isolated
  rather than hidden by green semantic tests.
- Live counts match the derived index exactly: 30 abnormalities, 66 placeholder E.G.O equipment
  entries in 22 full slot triplets, 31 total fragments, and 30 abnormality fragments split into 2
  ActiveSkill and 28 BasicAttackModifier effects.
- Confirmed roster and tool/design-only distinctions align. The detailed WhiteNight policy does not
  match current placeholder acquisition/runtime and requires an explicit future-policy disposition.

Interpretation:

- The current bundle is valid, but “maximally strict” is not true for transitive nested authoring.
  Existing negative tests cover selected outer/domain fields, not the demonstrated gaps.
- WhiteNight is a policy/content gap rather than a safe implementation fix: the document itself
  acknowledges future DTO/effect work, so the audit cannot synthesize the 12-apostle system.

Files changed:

- Goal audit documents only. Temporary probe files exist only under `/tmp/dca2_schema_probe`.

Next action:

- Trace Core DTOs through game_server and the external Unity canonical/consumer implementation.

## EXP-008 — Core, Server And Unity Transport/Consumption

Date/time: 2026-07-18 KST

Claim/finding IDs: DCA2-BATTLE-CLAIM-006, DCA2-RUNTIME-CLAIM-002,
DCA2-RUNTIME-CLAIM-003, DCA2-RUNTIME-CLAIM-010, DCA2-WIRE-CLAIM-001 through
DCA2-WIRE-CLAIM-004; DCA2-WIRE-001, DCA2-WIRE-002

Command or inspection:

- Read both complete external canonical documents and traced Core setup/update/event DTOs through
  game_server messages/handlers/session flow into Unity WebSocket routing, snapshot store, battle
  runtime state, battlefield projection, actor/HUD/range renderers and skill catalog models.
- Enumerated every production consumer of Unity `PresentationEvents` and searched runtime/tests for
  each contract-required event type.
- Ran `cargo check -p game_server` and `cargo test -p game_server -- --test-threads=1`.
- Used the connected Unity 6000.3.4f1 editor to run the complete EditMode and PlayMode suites.

Result:

- Server compile passed and all 30 server tests passed.
- Unity EditMode passed 241/241.
- Unity PlayMode passed with 61 total, 57 passed, 0 failed and 4 explicit live-server tests skipped.
- Setup/update ordering, exact forwarding, revision/gap validation, resync/reconnect recovery,
  actor identity, canonical routes, HUD and final preview ownership align across the layers.
- Unity retains the required event envelopes but production presentation consumes only movement,
  HP damage feedback, spawn identity validation and deploy confirmation. Required TileArea,
  inactive-lifecycle, attack/cast/projectile, buff/stat/resonance and battle-end presentation is
  absent, producing `DCA2-WIRE-001`.
- External `unity_core_contract.md` omits Maintenance/Event/Gate from current MapNodeCategory and
  still lists removed `SplashClusterFirst`, producing `DCA2-WIRE-002`.

Interpretation:

- A green broad Unity suite cannot establish behavior for event types that have no consumer or
  direct test. The transport state machine is aligned while the player-visible event presentation
  contract is materially incomplete.
- The enum mismatch is documentation drift rather than a wire parse failure because node category
  is stored as a string and unknown labels fall through to raw display.

Files changed:

- Goal audit documents only.

Next action:

- Begin Pass 5: audit test claim coverage, workflow/probe paths, legacy duplicates and every
  historical disposition before cross-component closure.

## EXP-009 — Registered Probe And Legacy/Discoverability Audit

Date/time: 2026-07-18 KST

Claim/finding IDs: DCA2-DOC-CLAIM-001 through DCA2-DOC-CLAIM-004,
DCA2-WIRE-CLAIM-005, DCA2-GAME-CLAIM-014; DCA2-WIRE-003, DCA2-GAME-003,
DCA2-DOC-002

Command or inspection:

- Verified all three paths, adjacent contracts and documented environment variables in
  `docs/probe_registry.md` against the external probe scripts and server admin-access gate.
- Started the normal local game_server with the registry command and ran
  `WS_PORT=18083 python3 docs/probe/unity_ws_smoke_probe.py` from the external Unity project.
- Searched production/test/current canonical surfaces for ignored tests, removed employee schemas,
  Artifact, gameplay battle records, `default_blocking`, old targeting/result names, local Unity
  contract copies and duplicate external documentation trees.
- Compared hashes of the canonical external Unity documents with the adjacent `docs - 복사본`
  versions and traced the live Maintenance recipe catalog.

Result:

- Registry paths/contracts/env names and server admin gates exist and match the current scripts.
- The first smoke invocation inside the filesystem/network sandbox failed with local socket
  `PermissionError`. The same exact command was rerun with approved localhost access and succeeded.
- The successful flow covered auth, Standard pre-run VN, starter selection, map snapshot/selection,
  combat preview, setup/update ordering, resync, deploy authority recovery, deployment range
  preview, deploy update, HUD/unit source/range preview and ping/pong. Combat-result waiting was not
  enabled and therefore is not claimed by this run.
- The live corroded-only encounter exposed `catalog_refs.abnormality_ids: [""]`, confirming
  `DCA2-WIRE-003` against the exact external actual-abnormality-only contract.
- No Rust `#[ignore]` or Unity `[Ignore]` was found in the official inspected test trees. Four Unity
  `[Explicit]` live-server tests were reported separately by the PlayMode suite rather than hidden.
- Core contains no forbidden local Unity canonical copies. The external `docs - 복사본` tree holds
  three divergent canonical-looking copies, producing `DCA2-DOC-002`.
- Live equipment Maintenance recipes cover only `standard_armor`, while generic fragment operations
  are runtime-defined, preserving deferred content-scope finding `DCA2-GAME-003`.

Interpretation:

- The probe registry is an executable current routing index, not stale prose. The probe additionally
  found a producer-level semantic violation that shape/unit tests had not protected.
- Negative legacy tests are not competing runtime schemas; they explicitly prove removed inputs are
  rejected. Historical decision notes containing old shapes are correctly labeled non-authority.

Files changed:

- Goal audit documents only. The local server wrote its normal log output; no production/data/code
  file was edited and the server was stopped gracefully after the probe.

Next action:

- Finish the historical finding/disposition matrix and current workflow/decision-record audit.

## EXP-010 — Historical Disposition And Decision-Record Revalidation

Date/time: 2026-07-18 KST

Claim/finding IDs: DCA2-DOC-CLAIM-001 through DCA2-DOC-CLAIM-004; all historical
DCA/CPR/MSTR inputs

Command or inspection:

- Read the current README/workflow/probe governance, both formal decision records, both archived
  audit registries and the cross-layer master's normalized finding/disposition registry and terminal
  status.
- Mapped all 52 DCA findings, all 15 CPR findings (nine aliases/six unique) and MSTR-01..07 to
  current producer/data/consumer/test evidence collected in Passes 1–5.
- Checked linked decision/master/completion paths and searched current production/data/current docs
  for removed schemas, ignored tests, old result/targeting/VN/Artifact/battle-record vocabulary and
  local external-contract copies.

Result:

- One completed historical disposition regressed: DCA-H-04/PD-09/G12, now
  `DCA2-DATA-002`, because nested production gameplay RON accepts unknown fields.
- DCA-M-08, DCA-M-21 and DCA-M-28 remain explicitly user-deferred and now map to
  `DCA2-GAME-001`, `DCA2-BATTLE-001` and `DCA2-GAME-003`.
- DCA-M-26 remains excluded/non-actionable. The other 47 DCA findings, all CPR findings and all
  seven MSTR findings retain their completed/not-applicable dispositions under current independent
  evidence.
- Both decision records correctly preserve old reality/recommendations as history and route current
  authority to existing durable documents and completed evidence. Current employee/VN behavior was
  independently traced rather than inferred from those records.
- Workflow package commands/paths and goal lifecycle match the repository. Probe registry is
  executable. Governance still has isolated drift from historical G15/G24 vocabulary and the
  external duplicate contract tree.

Interpretation:

- The master campaign's terminal status is broadly trustworthy under current evidence, but it
  cannot be quoted as proving transitive schema strictness after the demonstrated regression.
- New current-audit findings are tracked separately from the old registry so historical closure is
  not silently rewritten.

Files changed:

- Goal audit documents only.

Next action:

- Run the final justified broad validation, reconcile all ledger/finding counts, and write
  `AUDIT_REPORT.md`.

## EXP-011 — Final Broad Validation And Report Closure

Date/time: 2026-07-18 KST

Claim/finding IDs: all

Command or inspection:

- Ran `cargo test -p game_core --all-targets -- --test-threads=1`.
- Reconciled every coverage-ledger row and finding-registry row, removed the last provisional
  cross-layer verdicts, and verified there were no material `Insufficient evidence` claims.
- Wrote and consistency-checked `AUDIT_REPORT.md` against the working notes and execution log.

Result:

- Core all-targets: 776 passed, 0 failed, 0 ignored.
- Final ledger: 48 claims — 37 Aligned, 3 Document drift, 4 Implementation drift,
  4 Policy decision needed, 0 Insufficient evidence.
- Final registry: 14 findings — P0 0, P1 1, P2 8, P3 5.
- All findings are reported in `AUDIT_REPORT.md`; no production, live data, test, canonical or
  external Unity implementation file was changed.

Interpretation:

- Broad green validation proves the current assertions remain healthy, but it does not erase the
  source/consumer absences and schema probes recorded as findings.
- Audit completion is appropriate because every claim is classified and every policy-bearing branch
  is explicitly stopped for user decision rather than silently implemented.

Files changed:

- `docs/goals/documentation_code_alignment_reaudit/PLAN.md`
- `docs/goals/documentation_code_alignment_reaudit/EXPERIMENT_NOTES.md`
- `docs/goals/documentation_code_alignment_reaudit/EXPERIMENTS.md`
- `docs/goals/documentation_code_alignment_reaudit/AUDIT_REPORT.md`

Next action:

- None under this audit-only goal. Start separately approved correction/policy goals from the final
  report recommendations.
