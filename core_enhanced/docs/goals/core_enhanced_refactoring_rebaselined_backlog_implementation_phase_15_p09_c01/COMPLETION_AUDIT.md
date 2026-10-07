# Phase 15 CRA-P09-C01 Completion Audit

Audit Mode: phase_exit
Audit Target: Phase 15 CRA-P09-C01 — Close Projectile Publication And Basic-Delivery Compatibility
Current Status At Audit Start: `review_pending`
Implementation Complete Claim At Audit Start: `false`
Audit Date: 2026-08-10 (`Asia/Seoul`)
Policy Source: CRA-POL-003 strict projectile/basic publication; CRA-POL-006 compatibility boundary
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`
Direct Predecessor: Phase 14 CRA-P09-C02, `Complete / High`
Repository: `/mnt/f/work/simulator`
Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Dirty State At Audit Start: 445 expanded entries; normalized status SHA-256
`d12b9e27a48eb6c4bd2fa57ed97593fd9bd87f40a1d3c5dc1be219c61ed466c7`
Completion Guide: all 567 lines of `core_enhanced/docs/goal_completion_review_guide.md` reread immediately
before this independent audit

## Audit Scope And Approved Meaning

CRA-POL-003 requires one strict pre-publication contract for RON and programmatic candidates. Projectile speed and
every present optional range/time/count limit must be positive. Skills retain valid TargetLocked and
DirectionalCollision policy. Basic attacks and equipment weapon profiles accept only the target-locked projectile
fields their runtime executes and reject targetless, directional, stop-policy, nondefault collision and area fields
instead of ignoring them. Failures must retain exact object/path/cause and reject the whole candidate without
normalization, instant fallback or partial startup.

CRA-POL-006 forbids an unapproved public/RON/save/wire migration. Valid current live content, positive projectile
ceiling timing, skill lifecycle and basic weapon override behavior must therefore remain exact. Phase 16 query
consolidation, Phases 17–18 usefulness/preview, Phase 28 committed damage and Phase 29 motion extraction are outside
this audit except for verifying that Phase 15 leaves a trustworthy publication seam.

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy basis | Runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| P15-A01 | Every projectile scalar is positive before publication | CRA-POL-003 | `DeliveryDef::try_validate_projectile_scalar_contract` checks speed, range, lifetime, kills, pierces and collision hits; skill validation maps its typed suffix into the owning step path | exact RON/programmatic six-row tables; debug/release Core 978/978 | pass |
| P15-A02 | RON and programmatic candidates share semantic decisions and typed provenance | CRA-POL-003 | official Skill RON parses `RawSkillDatabase`, resolves an unpublished candidate, then `validate_domain_catalogs` maps `DataValidationError` to `LiveRonError`; direct `SkillDatabase` serde invokes the same semantic validator | RON zero table through `GameDataBase::load_live_sources`; programmatic parsed-candidate table; exact source/domain/object/path/cause | pass |
| P15-A03 | Basic and weapon consumers reject every ignored delivery field | CRA-POL-003 | `try_validate_basic_attack_consumer_contract` is used by `BasicAttackDef` and `WeaponCombatProfile`; basic runtime destructures only speed | 12-row programmatic matrix for both consumers plus RON parity proof | pass |
| P15-A04 | Invalid candidates fail atomically without normalization/fallback | CRA-POL-003 | `GameDataBase::load_live_sources` validates the complete parsed candidate before `from_validated_parts`; constructors validate before returning databases | full loader negatives return `Err`; existing atomic publication suite and Core 978/978 | pass |
| P15-A05 | Valid skill/basic projectile behavior and positive ceiling timing remain exact | CRA-POL-003 | skill and basic projectile runtime code is unchanged; runtime zero guards remain integrity defense only | projectile math 3/3, weapon ceiling/override 1/1, skill runtime debug/release 14/14, live catalogs green | pass |
| P15-A06 | Current production live data remains valid without rewrite | CRA-POL-003, CRA-POL-006 | official Skill/Abnormality/Equipment RON still authors positive speed and default basic/weapon policy | RON loading debug/release 25/25; live skill 4/4; live item skill 4/4; byte hashes unchanged | pass |
| P15-A07 | Public/RON/save/wire/event shape remains exact | CRA-POL-006 | `DeliveryDef`, `BasicAttackDef`, `WeaponCombatProfile` fields, variants, defaults and derives unchanged; no DTO, persistence, server message or event file changed by Phase 15 | Core snapshot/server suites; server 59/59; static shape/diff inspection | pass |
| P15-A08 | Canonicals describe the implemented authority | completion guide §12 | `data_loading_contract.md` owns atomic authoring/publication; `skill_target_contract.md` owns projectile publication-to-runtime boundary | direct sentence/field comparison against current validators and consumers | pass |
| P15-A09 | Dirty worktree and unrelated campaign work are preserved | CRA-POL-006 and Goal invariant | no reset/restore/clean/stash; scoped edits only; live content unchanged | activation/final status hashes, scoped hashes, `git diff --check` | pass |
| P15-A10 | Phase 29 receives a trustworthy publication seam without prebuilding later work | Master dependency | published projectiles now have positive motion scalars; terminal lifecycle remains Phase 14 authority; no motion extraction/query/usefulness/damage code changed | static producer/consumer search and unchanged runtime suites | pass |

## Runtime And State Evidence

- Basic projectile execution in `game/battle/core/commands.rs` reads `speed_units_per_ms` and always launches homing
  guidance against the locked target. It does not execute hit policy, targetless, range/lifetime, kill/pierce or
  collision configuration. The rejected basic/weapon matrix therefore matches the actual ignored field set.
- Skill runtime was not changed. TargetLocked, DirectionalCollision, targetless facing, max-hits, max-kills and
  same-time stop composition remain covered by the 14-test integration binary in debug and release.
- `projectile_flight_ms` still uses ceiling division for positive speed. Its speed-zero return remains a defensive
  integrity guard; no official Skill/basic/weapon database can now publish zero speed.
- No command cost, cooldown, event ordering, committed/released lifecycle, pause, checkpoint or rollback state was
  changed. Phase 14 terminal delivery behavior remains the sole lifecycle authority.
- No invalid value is normalized, converted to Instant, clamped or accepted through a legacy branch.

## Data And Publication Evidence

- `DeliveryDef` owns one typed scalar validator. Present `Some(0)` values are now semantic candidate errors rather
  than field-local serde errors, allowing the owning database to attach object ID and full field path.
- `SkillDatabase::new` and direct `Deserialize<SkillDatabase>` validate before returning. The official live loader
  intentionally parses the raw Skill candidate without early semantic stringification and validates it at the
  bundle domain gate, where `LiveRonError` retains structured provenance.
- `AbnormalityDatabase` and `CorrodedEmployeeProfileDatabase` attach their item/profile path to the typed
  `BasicAttackValidationError`. `EquipmentDatabase` attaches the item/weapon-profile path to the typed
  `WeaponProfileValidationError`.
- `GameDataBase::load_live_sources` calls `validate_domain_catalogs` before compiling/publishing `GameDataBaseParts`.
  The RON negative tests exercise this full entrypoint, not a helper-only predicate.
- Raw partial `DeliveryDef`/collision structs are unpublished candidates. The rebaselined collision test proves a
  raw zero can be represented but must fail semantic validation; official database paths cannot skip the gate.
- Production files are byte-identical to activation:
  - `game_resources/data/skills/base.ron`:
    `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`
  - `game_resources/data/abnormalities/base.ron`:
    `3d7ffdb9cf4bbe6fadb09661dc4c4658cafb699885f35f0e3efc656d59a45de2`
  - `game_resources/data/equipments/base.ron`:
    `322d7455a256082f03804c0dcc7a77a4caa3a06919d5e34b667042b411bcd1a6`.

## DTO, Transport And Persistence Evidence

- Employee snapshot DTOs serialize `BasicAttackDef` and `WeaponCombatProfile`; their positive serialized shape is
  unchanged because no public field, enum tag, nullability, default or derive changed.
- Phase 15 changed no `behavior.rs`, server message/mapping, checkpoint envelope, active-run payload, event log or
  Unity protocol hunk. Existing dirty changes in those paths predate and remain outside this Phase.
- Server all-target tests pass 59/59, including actual Core snapshot-to-server mapping tests. No message-order or
  request-correlation change exists to require a live WebSocket or Unity runtime probe.
- No save/checkpoint version bump, reader/writer migration, dual-read or compatibility interval is needed because
  projectile authoring definitions are static published data rather than persisted run state.

## Consumer Evidence

- Confirmed workspace producers: live RON parser, Skill/Abnormality/Corroded/Equipment database validators and
  programmatic constructors.
- Confirmed workspace consumers: skill projectile runtime, basic projectile release/runtime, weapon-profile
  application, employee snapshot DTO production and game_server snapshot mapping.
- Canonical external consumer: Unity consumes existing snapshot/event shapes. Those shapes did not change; no Unity
  code or deployment coordination is part of the approved strict invalid-data cutover.
- Inaccessible/unknown required consumer: none identified for this static-data validity boundary.
- Explicitly excluded: query/usefulness/preview, committed damage, shared projectile motion extraction and terminal
  lifecycle changes assigned to other Phases.

## Legacy And Compatibility Audit

- Removed projectile-specific u8/u64 early-zero serde helpers have no remaining reference. The remaining
  `deserialize_nonzero_option_u32` belongs only to TileArea tick interval authoring.
- Scalar validation has one definition. Basic-consumer compatibility has one definition and exactly two domain
  adapters; there is no parallel permissive validator.
- No legacy field, alias, dual schema, normalization, implicit Instant fallback, ignored/disabled test or debug-only
  publication bypass was introduced.
- Runtime speed-zero and collision validation assertions remain defense in depth, not accepted-input authorities.
- The direct raw-collision test was not weakened to catch any result: it requires raw parse success and the exact
  semantic failure string. Official publication tests separately require exact structured provenance.
- Strict rejection applies only to values that were invalid under the approved policy. Valid RON/public Rust/wire
  representation remains compatible under CRA-POL-006.

## Scoped Diff And Dirty Preservation

- Activation after Child initialization: 443 entries,
  `31f2c5de78e8d2bf9698ba51aae95a9e3e728254914a5c65e63ff5a6424ffd71`.
- Pre-audit implementation/docs state: 445 entries,
  `d12b9e27a48eb6c4bd2fa57ed97593fd9bd87f40a1d3c5dc1be219c61ed466c7`.
- The two-entry increase is explained by previously clean
  `game/data/abnormality_data.rs` and `game/data/corroded_employee_data.rs` becoming scoped validator edits. Tests
  added no tracked/untracked artifact.
- Already-dirty ability, skill, equipment, live-loader and canonical files contained earlier campaign work. Phase 15
  edits are localized to delivery validation/publication, exact tests and the two nearest canonical paragraphs.
- No live data, server, checkpoint, Unity, generated fixture or unrelated runtime file was edited for Phase 15.
- No destructive Git/worktree command, deletion, move, broad formatter or unrelated cleanup was used.
- Audit-inclusive expanded status: 446 entries; normalized SHA-256
  `224ef55530ffddedf1711e83728253523561e429d2ca99beb2425e61dfd5ff49`. The one additional entry is this required
  `COMPLETION_AUDIT.md`; no production, data, test or unrelated artifact appeared during audit.

## Canonical Alignment

- `data_loading_contract.md` now states positive speed/optional limits, raw-candidate versus publication semantics,
  exact provenance, no runtime fallback and the complete basic/weapon compatibility boundary.
- `skill_target_contract.md` places the same publication rule immediately before existing projectile target/runtime
  lifecycle, without changing Phase 14 terminal semantics.
- `game_rulebook.md` continues to delegate detailed projectile lifecycle and authoring to
  `skill_target_contract.md`; no competing policy text was found.
- Goal documents are evidence/coordination records rather than the sole owner of the implemented contract.

## Validation Results

All commands ran against the current dirty workspace.

- Pre-change focused baselines:
  - skill-data module: 13 passed, 0 failed, 961 filtered at that baseline;
  - abnormality module: 4 passed, 0 failed;
  - equipment module: 8 passed, 0 failed;
  - `ron_loading`: 25 passed, 0 failed.
- Exact historical red filters, each selecting one test:
  - programmatic scalar matrix: 0 passed, 1 failed because speed zero returned `Ok(())`;
  - RON scalar matrix: 0 passed, 1 failed because speed zero returned `Ok(())`;
  - programmatic basic/weapon matrix: 0 passed, 1 failed because the path stopped at `basic_attack`;
  - RON basic/weapon parity: 0 passed, 1 failed because basic `max_kills` was accepted.
- Corrected exact filters: 4 passed, 0 failed; one selected test each.
- Focused module families: Skill 13/13, Abnormality 4/4, Corroded Employee 6/6, Equipment 8/8,
  live RON 70/70, RON integration 25/25.
- Positive/runtime families: projectile math 3/3, weapon ceiling/override 1/1, skill runtime 14/14,
  live skill catalog 4/4, live item skill activation 4/4.
- Complete debug Core library: 978 passed, 0 failed, 0 ignored.
- Complete release Core library: 978 passed, 0 failed, 0 ignored.
- Release skill runtime: 14 passed, 0 failed; release RON loading: 25 passed, 0 failed.
- `cargo test -p game_server --all-targets`: 59 library tests passed, 0 failed; main/example targets reported zero
  tests and no failure.
- `cargo check --workspace --all-targets`: passed for Core and server; compile success is static evidence only.
- `cargo fmt --check`: passed.
- Scoped `git diff --check`: passed.
- Final full-loader RON scalar and compatibility filters after converting them to
  `GameDataBase::load_live_sources`: 2 passed, 0 failed, one selected test per filter.

## Baseline Failures

- No pre-change focused baseline failure existed.
- The first post-change complete Core run was 977 passed / 1 failed. The sole failure was an outdated partial-raw
  serde assertion, not a runtime/data regression. It was rebaselined to the approved raw-candidate/semantic-
  publication boundary with an exact semantic assertion, then the complete Core suite passed 978/978 in debug and
  release.
- No failure is suppressed, ignored or left outstanding.

## Cross-Component Findings

- No competing source of truth remains between Skill RON, programmatic Skill construction and outer bundle
  publication: all converge on `validate_skill_contracts` and the shared scalar authority.
- Basic and weapon definitions use the same compatibility authority before the shared basic projectile runtime.
- No schema/version assumption conflicts with server, DTO or persistence because shape is unchanged.
- Phase 14 terminal delivery remains intact. Phase 29 may now rely on positive published motion scalars without a
  temporary shim. Phase 16 remains independent and was not prebuilt.
- Material cross-component finding: none.

## Long-Term Direction Review

| Candidate ID | Work type | Current→target authority result | Preserved invariant result | Removal/consolidation result | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P09-C01 | correctness correction | fragmented serde/skill/basic/weapon checks → one typed scalar/basic-consumer authority with owner adapters | live data, runtime timing/lifecycle, DTO/wire/save shapes preserved | early lossy zero gates and ignored basic/weapon policy acceptance removed; no compatibility branch | exact matrices, Core debug/release, live/runtime/server/static suites | pass |

- Fit: `High`.
- Improvement class: `none`.
- Reason: domain ownership now sits on `DeliveryDef`, while databases supply only owner-path context and runtime keeps
  integrity guards. The structure eliminates divergent RON/programmatic decisions without a public type migration,
  preserves all valid behavior, and creates the positive-scalar seam required by later motion extraction. Tests
  protect official publication and runtime contracts rather than helper placement alone.

## Primary Verdict

- `Complete`.
- Every approved completion condition has current production/data/contract evidence and nonzero focused/broad
  validation. No material legacy, compatibility shim, unverified required consumer, canonical drift, scope violation
  or unresolved failure remains.

## Status Transition

- From: `review_pending`, `Implementation Complete: false`.
- To: `complete`, `Implementation Complete: true`.
- Master Phase 15 registry: may transition to `complete — Complete / High`.
- Next Phase activation: allowed only after this audit's final record-only refresh and Master coordination update.

## Remaining Risk And Required Action

- Residual risk is low: a caller can construct a raw invalid public value, as before with programmatic structs, but
  every official publication path validates it. Runtime zero guards remain to contain invariant violations.
- No immediate correction, policy question, compatibility migration or follow-up refactor is required.
- After recording audit-inclusive status/hash, update Child/Master status and activate only Phase 16 according to the
  exact linear registry.
