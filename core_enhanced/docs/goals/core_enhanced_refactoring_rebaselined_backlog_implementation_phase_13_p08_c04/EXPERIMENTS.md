# Phase 13 CRA-P08-C04 Experiments

Status: complete
Implementation Complete: true

## 2026-08-10 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's Master execution request remains active and authorizes the single active Child.
- Phase 12 CRA-P08-C02 has a current `Complete / High` audit with improvement class `none`; its 226-line audit was
  fully reread after the final record-only correction.
- Phase 11 CRA-P08-C01 supplies the executable typed occurrence dispatcher and approved CRA-POL-002 semantics.
- Phase 13 is next in the exact registry; Phases 14~44 remain pending and unauthorized.
- CRA-POL-002 closes the OnHit Counterpart BonusDamage matrix choice; CRA-POL-006 freezes unrelated
  public/RON/save/wire shape.

### Evidence Carried Into Initialization

- CRA-P08-C04 remains correctness correction with `2/2/1/3/3/2/unknown = 13/18`.
- Historical defect: publication does not resolve equipment ApplyBuff/activation IDs or reject trigger/effect/target
  rows runtime drops, so invalid authored data can become a late silent no-op.
- Approved behavior: OnHit Counterpart BonusDamage is owner-attributed once-per-effect non-recursive retaliation,
  not current-hit amplification; the publication matrix must match Phase 11.
- Required proof: exact missing buff/skill, invalid Permanent and occurrence-matrix negatives; complete official
  bundle positive; deterministic equipment/path/cause; atomic no-publication on failure.
- Runtime occurrence meaning, future projectile/basic publication, committed-damage consolidation and disconnected
  death cleanup remain outside this Phase.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 13 coordination changes from `ready` to `active`; no other Child is active.
- Phase 13 production status/source/data investigation: not started.
- Test executions: none.
- Production/test/data/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents completely, capture expanded dirty status and prospective target hashes, then trace the
actual source/parser/candidate/publication path, every equipment effect/activation edge and the Phase 11 executable
matrix before running nonzero baselines or adding tests.

## 2026-08-10 — E01 Dirty Baseline, Publication Graph And Pre-Mutation Tests

### Dirty Boundary

- HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded post-initialization status: 435 entries; normalized SHA-256
  `eec76077b533d928e3dc9e2eaa5e4af4d6f8c5f45c075e4639c4fdf4e6423de1`.
- Prospective fingerprints before Phase 13 content mutation include:
  - `equipment_data.rs` `5b93afc826e4b0d2bf04e54bcf70783e8b36cefac1b5b67679d254df4e714003`
  - `data/mod.rs` `70f6fbf61cb9e5e26e1aea51df94656998a7c3d4530c4362a1ab3a3b8b535e64`
  - `data/validation.rs` `adc36cb574f0f6693ee211ded7ed9f17a9036f9335b1887b0f6876630be53343`
  - `data/live_ron.rs` `07fb40a52ccb86c83176b80ff994d5e9a62028d2321aa04e16b95458062dd408`
  - `stats.rs` `9c4d2c5645a5a701c041eae8ba14edbdfca232182eff0eca1da152c9b535fc1f`
  - `triggers.rs` `84928567ad0e9fb438896cb40b49592c77a7d07b7d23e84b5c163048e77ac01a`
  - `ron_loading.rs` `75558ff28be83cafcfe535252f552903b2077249c2cd81ef00b41d63195854b3`
  - live item test `fae73da9c9fe30b3b4f919828100d6b1342a20677a4fe68396970c877d03d5a0`
  - live equipment RON `322d7455a256082f03804c0dcc7a77a4caa3a06919d5e34b667042b411bcd1a6`
  - live buff RON `e7cc335f4243b4fc0ac907a72356ba44a1eae2669aa340147a0fbbc14f9c448b`
  - live skill RON `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`
  - data canonical `f6914e6c4fea82a0c63fa913c34b642b257c6c44584474a5036313a446174271`.
- All prospective tracked targets except `battle/cooldown.rs` and live buff RON are already modified by
  user/campaign work. Existing loader, equipment, skill, trigger, test, RON and canonical hunks remain user-owned.

### Current Publication And Consumer Graph

- Official production entry is `GameDataBase::load_live_embedded` → source inventory/schema parse →
  `ParsedGameplayRon::validate_domain_catalogs` → `from_validated_parts`. Validation completes before any
  `GameDataBase` is constructed, so this is the failure-atomic publication boundary.
- `EquipmentDatabase::validate_contract` currently checks weapon profiles, event metadata and dismantle/enhancement
  references only. `validate_fallible_domain_contracts` joins skill→buff references but calls no equipment→buff/
  skill or occurrence-matrix validator.
- Fixture/programmatic `GameDataBase::new` validates local indexes and then `validate_data_references`; that global
  join has the same equipment-edge omission and currently panics only for other invalid catalog edges.
- Runtime item lookup skips missing equipment metadata. Trigger collection resolves authored effects/activations from
  equipment, skill invocation skips a missing skill, and ApplyBuff handling skips a missing buff. These remain
  integrity guards, but without publication validation they are also late silent authoring failure.
- Phase 11 runtime classifies OnAttack/OnHit damage effects, materializes ordinary target effects and emits OnHit
  Counterpart BonusDamage retaliation; unsupported damage occurrences are silently dropped.

### Executable Matrix Derived From Current Runtime

- Permanent: Modifier is executable only for SelfUnit/AllAllies through prebattle stat composition; activations and
  other effects are unsupported.
- OnBattleStart: ordinary Modifier/Heal/ApplyBuff supports SelfUnit/AllAllies/AllEnemies; CounterpartUnit has no
  occurrence counterpart. Damage effects are unsupported.
- OnAttack: ordinary effects support every target. BonusDamage/ModifyDamage are current-hit effects only with
  CounterpartUnit.
- OnHit: ordinary effects support every target. BonusDamage/ModifyDamage with SelfUnit are current-hit effects;
  CounterpartUnit BonusDamage alone is CRA-POL-002 retaliation. Counterpart ModifyDamage and group-target damage
  effects are unsupported.
- OnDeath: the trigger owner is already dead, so ordinary SelfUnit is unsupported; CounterpartUnit/AllAllies/
  AllEnemies may resolve active targets. Damage effects are unsupported.
- OnKill and OnAllyDeath: their counterpart is the already-dead unit, so ordinary CounterpartUnit is unsupported;
  SelfUnit/AllAllies/AllEnemies are executable. Damage effects are unsupported.
- TriggerProc activation is executable for all runtime occurrence triggers except Permanent.

### Pre-Mutation Nonzero Baselines

- Equipment local validation: 7/7 passed.
- Live publication validation: 62/62 passed.
- Full official RON: 25/25 passed.
- Phase 11 commands/occurrence path: 39/39 passed.
- Production live item integration: 5/5 passed.
- Test listing also confirmed those filters select real tests; list-only output is not used as behavior proof.

### Result And Next Experiment

The candidate remains `still_valid`. Add official-candidate missing buff/skill, invalid Permanent and exhaustive
trigger/effect/target/activation tests against the real validation gate; establish red results before introducing the
shared typed execution classifier and cross-catalog equipment validator.

## 2026-08-10 — E02 Test-First Reference, Matrix And Atomicity Reds

### Added Proofs

- Added an official parsed-candidate missing ApplyBuff reference with exact equipment ID,
  `items[i].triggered_effects.OnBattleStart[0].effect.ApplyBuff.buff_id` and cause.
- Added a missing equipment activation skill with exact
  `items[i].ability_activations[0].ability_id` provenance.
- Added an explicit 140-case trigger × effect × target table and seven activation-trigger rows. The table encodes
  Permanent/prebattle, ordinary target materialization, current-hit damage, CRA-POL-002 retaliation and dead/
  absent-counterpart constraints derived in E01.
- Added a real source-set test that serializes an invalid typed equipment candidate, calls
  `GameDataBase::load_live_sources`, requires rejection before DB construction and keeps an already published
  official `Arc<GameDataBase>` unchanged.

### Exact Red Results

- Missing buff: 1 selected, failed because `validate_domain_catalogs` returned `Ok(())`.
- Missing activation skill: 1 selected, failed because `validate_domain_catalogs` returned `Ok(())`.
- Matrix: 1 selected, failed on the first unsupported row
  `Permanent / SelfUnit / BonusDamage` because validation returned `Ok(())`.
- Atomic source: 1 selected, failed because the invalid source constructed a `GameDataBase`.
- One initial short filter combined with runner `--exact` selected 0 tests and was discarded; it is not evidence.

### Next Experiment

Introduce one Core-internal typed effect-execution classifier used by prebattle stats, Phase 11 current-hit/
retaliation dispatch and publication. Add equipment cross-catalog validation for buff/skill references and activation
triggers at both fallible official and programmatic builder gates, then rerun the exact four failures.

## 2026-08-10 — E03 Shared Execution Matrix, Atomic Publication And Green Validation

### Implementation

- Added Core-internal `TriggeredEffectExecutionKind` and `classify_triggered_effect_execution`. It is the single
  typed compatibility authority used by Permanent stat composition, Phase 11 current-hit/targeted-command/
  retaliation dispatch and publication validation; no public or serde carrier was added.
- Added `EquipmentDatabase::validate_effect_and_activation_contract`, which traverses items and a fixed seven-trigger
  order, rejects every unsupported trigger/target/effect row, resolves every `ApplyBuff.buff_id` against the candidate
  Buff catalog, rejects Permanent activations and resolves every activation `ability_id` against the candidate Skill
  catalog with deterministic equipment/path/cause.
- Called that contract from both official `ParsedGameplayRon::validate_domain_catalogs` and programmatic
  `validate_data_references`, before either path can return a usable `GameDataBase`.
- The Phase 11 dispatcher now consumes the shared classifier for current-hit, ordinary targeted-command and
  CRA-POL-002 retaliation routing. Unsupported rows remain defensively ignored at runtime, but can no longer enter
  through either validated publication path.
- Corrected two test fixtures newly exposed as invalid: the occurrence decision-table fixture now supplies its real
  activation skill and uses live counterparts only where the occurrence has one; the trigger-order fixture now
  supplies actual official Skill/Buff definitions. The former behavior/order assertions remain intact.
- Rebased the former runtime-only invalid-Permanent test to require earlier programmatic publication rejection with
  exact path/cause. Removed the redundant hand-written two-ID live skill-resolution assertion; complete catalog-edge
  validation and four production live-behavior integrations remain.
- Updated `data_loading_contract.md` with the exact equipment cross-catalog/matrix, atomic rejection and defensive
  runtime-lookup boundary. No RON content, schema, save/checkpoint, DTO/wire, server/Unity or public Rust shape changed.

### Exact Red-To-Green And Matrix Proof

- The four E02 reds are green with nonzero selection: missing buff 1/1, missing activation skill 1/1, exhaustive
  140 effect rows plus seven activation rows 1/1, and invalid source/unchanged installed Arc 1/1.
- Programmatic publication proofs are green: missing effect reference 1/1 and invalid Permanent target 1/1, both
  requiring the global builder gate rather than helper-only validation.
- The 140-row effect cross-product covers seven triggers × four targets × five effect kinds. The accepted OnHit
  Counterpart BonusDamage row is classified only as owner-attributed non-recursive retaliation; Counterpart
  ModifyDamage and group damage rows remain rejected.
- Full official publication validation is 66/66 and full RON integration is 25/25 without live content edits.

### Focused And Broad Proof

- Equipment local/publication tests 8/8; commands/occurrence module 39/39; trigger payload 1/1; stat pipeline 6/6;
  Skill data 13/13; Buff data 4/4; production live item 4/4; live Skill catalog 4/4; Skill runtime 13/13; Skill suite
  4/4.
- Complete Core lib is 974/974 after the publication-boundary fixture correction.
- `game_server --all-targets`: main target 59/59; four additional binary targets selected zero tests and passed.
- Release exhaustive equipment matrix 1/1 and release retaliation filter 3/3.
- `cargo check -p game_core --all-targets`, `cargo fmt --all -- --check` and `git diff --check` pass. The complete
  Core 974/974 rerun also passes after formatting.

### Static, Compatibility And Dirty Result

- Caller inventory finds the shared classifier only at Permanent stat execution, item occurrence dispatch and
  equipment publication validation. Both official and programmatic publication gates call the complete equipment
  contract exactly once.
- Runtime integrity guards remain: missing equipment metadata still skips trigger collection, and missing runtime
  Buff definitions still no-op in command application. They are not publication fallback paths.
- The obsolete `live_item_ability_ids_resolve_to_skill_defs` name is absent. Production live item tests still execute
  battle-start effects/activations, ally targeting and forced OnAttack procs.
- Expanded status remains 435 entries with the E01 normalized SHA-256
  `eec76077b533d928e3dc9e2eaa5e4af4d6f8c5f45c075e4639c4fdf4e6423de1`; no unrelated status entry was added or
  removed.
- Final relevant fingerprints:
  - `equipment_data.rs` `8e96dd1b531e66a5c8eec9e16f44e55b7c1b0934b2dc3699909f26d503263662`
  - `validation.rs` `c71c4208a6b7044e17dbe67986eae7cd1bb7fbf451a8fc28a3de2705d0ff2083`
  - `live_ron.rs` `343cf23fbfa427748af7b947bee6be147b7cf76924b99ac2d4c1275d07ce3897`
  - `stats.rs` `9c3d49b44c773288da84e35bc0411292e423583a4b481a1e044622c138d10269`
  - `triggers.rs` `567bc63667f47dba2e733334e39050034b989096466ef4cea642993d66c14957`
  - `commands.rs` `08c773999dac25f4e5bea3c35971b2c5aa24017b6a69d98d335cb9fd865c465a`
  - `battle/types.rs` `5d714af9d66bbdadfb0a130238210826512e72a04ae9e7baedffe5d3816f01e3`
  - live item test `84e4e600ba8816fed6ec888353bcc7c98ecd4774b17fef4969897ace4e27c61b`
  - data canonical `4c30e36c12b4e1025e8c5c928ad373cc5b8356e5bacc500076f3514c33987786`
- Official equipment, Buff and Skill RON retain their E01 hashes respectively:
  `322d7455a256082f03804c0dcc7a77a4caa3a06919d5e34b667042b411bcd1a6`,
  `e7cc335f4243b4fc0ac907a72356ba44a1eae2669aa340147a0fbbc14f9c448b` and
  `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`.

### Next Experiment

Reconcile every completion condition, reread the full completion-review guide, independently audit current
publication/runtime code, tests, compatibility, dirty preservation and canonical, then close only on
`Complete / High`.

## 2026-08-10 — E04 Phase-Exit Completion Audit

### Independent Review

- The complete 567-line completion-review guide was read in four bounded sections; the current 190-line Plan,
  222-line Experiments and 71-line Notes were then fully read before the independent review.
- Current policy, classifier, both publication gates, loader construction order, runtime consumers, production RON,
  canonical, compatibility and dirty evidence were rechecked without production correction.
- Audit-current reruns passed: official publication 66/66, commands 39/39, production live item 4/4, RON 25/25,
  release matrix 1/1 and release retaliation 3/3. Current check, formatter check and diff check also pass.
- Broad implementation evidence remains current under unchanged production/data fingerprints: Core 974/974 and
  server main 59/59.
- Audit artifact: `COMPLETION_AUDIT.md`, 234 lines, fully reread after one record-only wording correction.

### Verdict And Boundary

- Primary verdict: `Complete`.
- Long-term fit: `High`.
- Improvement class: `none`.
- Unresolved material finding, immediate correction and policy question: none.
- Child changes to `complete` with `Implementation Complete: true`. Phase 14 CRA-P09-C02 may become ready; no Phase
  14 production work is authorized until its own initialization gates complete.
