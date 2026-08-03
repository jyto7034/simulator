# Gameplay RON Strictness And Bundle Validation Experiment Notes

Status: active

Implementation complete: false

Created: 2026-07-13

## Accepted Policy Boundary

- Gameplay RON is generated, exact-version, canonical, and maximally strict.
- Runtime rejects aliases, unknown/removed/missing required fields, duplicate IDs, invalid bounds and
  combinations, invalid references, unsupported/runtime-panic variants, and partial bundles.
- Authoring validation and Core validation should share authority where actual repository boundaries
  support it, but G12 does not invent the full authoring program.
- Diagnostics require file/domain/object/field-path/cause context.
- Save migration is separate and cannot justify permissive live-data parsing.

## Finding Ownership

- DCA-H-04: make implementation match the documented strictness guarantee.
- CPR-H-08: reject late-invalid Event RewardEffect references and any accepted runtime-panic variant
  before play; do not invent replacement semantics.
- CPR-M-01: validate required supported-mode RunStart presence/cardinality/policy. G04 already closed
  the test-fixture regression portion.
- CPR-M-02: validate extra-wave and map/stage/mission Event references at final bundle construction.

## Predecessor Constraints

- Preserve G04/R00's production rejection semantics and trustworthy broad baseline.
- Do not change G01 battle revision, G06 common capability, or G07 battle authority/eligibility while
  restructuring data loading.
- Phase 2 remains blocked until G12 completes and R01 independently reviews Phase 1.

## Structural Direction To Verify

The intended long-term shape is one typed parse-and-validate pipeline that builds local domain values,
runs duplicate/domain checks, runs final cross-bundle checks, and publishes one immutable complete
bundle only on success. This is a hypothesis until the actual production loader and live data are
read. Do not force a new wrapper if current code already has a better atomic construction seam.

## Policy Stop Boundary

Technical strictness and deterministic diagnostics are approved. Stop if source review requires a
decision to remove/replace live content, define behavior for a currently unsupported live-referenced
variant, change balance/cardinality meaning beyond required structural validity, migrate saves, expose
new Unity-facing schema/UX, or retain compatibility.

## Deferred/Out-Of-Scope Boundary

- Full authoring-program implementation and UX.
- Event presentation/d20 flow and new Event effects.
- Later targeting, economy, inventory, Equipment/Maintenance, compendium, and balance policies.
- Save migration.

Record newly discovered out-of-scope improvements here and move to the owning child.

## First Runtime Ownership Judgment

- The long-term direction does not require a second `LiveGameDataBundle`: `GameDataBase::new()` already
  validates all parts before returning the immutable database. G12 should make parsing/construction
  fallible and complete at this seam, then leave fixture-oriented `GameDataBuilder` ergonomics
  separate from the one production loader.
- `load_live_embedded()` is compile-time embedded, so “partial publication” currently means accepting
  an incompletely validated but successfully constructed database, not exposing half the local
  variables during parsing. The fix is complete validation before `Arc` return, not transactional
  filesystem hot reload.
- A bundle-level exact schema version is absent and remains a design detail to select only after all
  18 live file shapes and existing envelope structs are inspected.
- Existing panic-based duplicate/reference protection is useful behavioral evidence but not the
  accepted diagnostic contract. Convert/compose it rather than weakening it or layering catch-all
  panic parsing around it.
- The production bundle boundary currently excludes four authored map/battlefield files. Domain
  loader ownership does not need to be merged into `GameDataBase`: the existing `OnceLock` builtins
  can remain their runtime source if the final startup seam initializes those exact objects and
  cross-validates them with `GameDataBase` before returning. A second parse/copy or validator-only
  shadow object would be a competing authority and is not acceptable.
- Removing `DamageType` aliases is a safe canonical-schema correction because live RON already uses
  `Physical`, `Magic`, and `True`. Do not retain aliases for hypothetical manual authors.
- Event `ApplyBossOmenStepResult` removal is already directed by the master and live data has no use.
  This does not authorize inventing a replacement BossOmen transition.
- General `RewardEffect` validation should become one reusable path parameterized by source context,
  then be applied to Reward records and every Event scene/choice effect. Copying the match into an
  Event-only validator would create drift.
- RunStart exact-cardinality policy is structural for the two current supported modes: exactly one
  Standard and one Endless trigger, with current approved blocking pre-run policy. Do not treat an
  empty mode as a runtime `InvalidStaticData` fallback after G12.
- The version-placement trial supports a single strict bundle manifest. This avoids duplicating the
  same version field across 23 domain roots while still rejecting an unsupported bundle before any
  publication. Fixed source membership remains code-owned; do not add a manifest file list unless it
  becomes the actual loader driver.
- `serde_path_to_error` works with the current RON deserializer and provides nested parse paths. Keep
  semantic object/reference paths explicit in the same `LiveRonError` rather than trying to infer
  them from free-form panic text.
- The first production cutover proves all 23 files can share one parse source set without moving map
  type ownership. It is not yet the final publication design because map/battlefield runtime builtins
  still reparse. Do not describe startup smoke validation as eliminating their authority until the
  runtime consumes the parsed objects.
- The runtime cutover now resolves that intermediate caveat: `GameDataBase` owns one published
  `GameplayMapCatalogs`, while map/combat-preview modules still own their types and algorithms. This
  satisfies the prior loader-ownership decision without retaining global lazy readers or duplicated
  RON parsing.
- Unknown-field diagnostics now prove exact nested authored paths rather than only parse positions.
  Keep this stable source/domain/path shape when semantic validators are converted; field-qualified
  paths are more useful than free-form object-only panic messages.
- Supported-mode `RunStart` cardinality and policy are now structural bundle invariants rather than
  runtime recovery behavior. This closes only G12's CPR-M-01 portion and does not alter G04's server
  fixture ownership.
- Pool uniqueness belongs before publication even while domain lookup methods remain linear. The
  final validator now prevents any Shop or Reward first-match winner from being observable; later
  domain goals may change indexing without redefining this invariant.
- RewardEffect validity is now one composable authority shared by Reward and Event data. The old
  assertion-based database constructor temporarily calls the same fallible helper while G12 converts
  the publication boundary; it is not a compatibility path or a second rule table.
- Map payload validation resolves only authored `Some(id)` values. `None` remains the current
  intentional random-selection contract, so strictness does not invent a concrete target or change
  map-generation policy.
- Run-policy extra waves are not a separate content domain: runtime appends them to every encounter
  at a stage. They therefore share encounter-wave validation, must not collide with encounter wave
  IDs, and must satisfy the Defense route requirement whenever a Defense encounter is supported.
- Direct reference validation cannot prove authored spawn-zone/route IDs against selected templates.
  Startup therefore exercises each encounter at every scaling-stage representative floor across the
  existing deterministic seed set; this is production-path validation, not a mock-only substitute.
- The public live loader now exposes a structured failure and game_server aborts before binding. Test
  helpers may explicitly expect valid embedded fixtures, but production must not regain a fallback
  or panic-wrapper API while the remaining domain validators are converted.
- Parsed-bundle key validation is deliberately not a second index. It proves uniqueness/emptiness on
  authored collections, then the existing immutable domain indexes are built from those same values.
  Nested identifiers and non-key semantic assertions still need fallible conversion.
- Serde defaults are being classified by boundary. Defaults on Core-only generated RON fields should
  be removed and live output made explicit. Defaults on types also used as runtime/Unity DTOs must
  not be changed mechanically; use a strict authored raw type if needed so wire compatibility is not
  altered accidentally.
- `Option<T>` is a special strictness case: Serde accepts a missing field as `None` even without a
  `default` attribute. Core-only generated Option fields need an explicit required-option
  deserializer so the writer must emit `Some(...)` or `None`; shared runtime/Unity DTO Options remain
  outside mechanical conversion.
- Required authored Options must also serialize explicit `None`; retaining `skip_serializing_if`
  would make the canonical typed writer produce output the strict reader rejects. BossOmen now
  round-trips this invariant without a dual schema.
- `DataValidationError` is the reusable domain-local shape: domain code owns object/path/cause,
  while the live bundle boundary supplies source file/domain. This keeps one semantic validator
  usable by production and fixtures and avoids both free-form panic parsing and duplicated rules.
- G12 validates only the safety of the currently accepted Event schema. It does not treat current
  preview/hint/effect shape as durable G03 policy; the later Event vertical slice must migrate its new
  schema through this same strict bundle authority.
- PVE validation remains split by ownership, not by error mechanism: `pve_data.rs` owns local
  authoring structure, while `validation.rs` still owns cross-catalog references and mission/reward
  compatibility. Both must feed the same production `LiveRonError`; moving cross-domain rules into
  the local database would create a second bundle authority.
- Visual Novel trigger validation follows the same split: local payload shape belongs to the trigger
  database, while referenced VN/Event/map/PVE existence and RunStart set cardinality remain bundle
  concerns. The G12 conversion does not reopen the completed G01/G04 trigger behavior policy.
- Visual Novel graph conversion preserves the completed schema verbatim; only failure transport
  changed. It therefore does not authorize new choice effects, Event/VN convergence, or Unity DTO
  changes, all of which remain outside G12.
- Corroded-employee validation cannot be safely converted as an assertion-only batch: its custom
  deserializer resolves authored range presets and can panic on invalid preset shape, unknown preset,
  or dual inline/preset range authoring before the published database exists. Convert that raw-to-
  runtime transformation and the later local validator together; a partial post-parse validator
  would leave a hidden startup panic and falsely claim closure.
- Skill Fragment static compatibility validation now owns field selection locally, while the bundle
  supplies file/domain context. Runtime compatibility reports and future balance/content policy were
  deliberately not changed.
- Equipment recipe indexes must be derived only after recipe validation. Lazy index construction is
  the durable shape because it lets the same domain contract serve production-deserialized data and
  fixtures; eager `build_recipe_index` was a competing panic boundary.
- Cross-domain conversion should preserve the source-owning catalog in each error. Trigger-to-VN
  failures therefore identify the trigger file/path, not the referenced VN file; the missing target
  remains in the cause.
- Event-to-PVE errors likewise belong to `data/events/story/base.ron`: the Event choice/effect is the
  authored source that must be corrected, while the missing/mismatched encounter is diagnostic cause.
- `ItemRegistry` is a derived runtime lookup, not a data-validation authority. Shop references must
  resolve against the authored item catalogs first; registry construction follows only after that
  contract succeeds.
- Starter loadout layout is a cross-catalog acceptance rule because it combines candidate data,
  Equipment metadata, and Skill Fragment IDs. `ItemSlot` remains the layout behavior authority, but
  its failure is now attributed to the candidate's exact authored equipment path.
- Unity MCP is available for `ark@b49d32f6f2f70bbe` at Unity 6000.3.4f1, project
  `F:/unity projects/ark`, and exposes the Unity test runner. The 2026-07-13 editor snapshot was idle
  but stale (`ready_for_tools: false`, `stale_status`), so no Unity test was claimed as G12 evidence.
  Re-read editor state and use the MCP runner only at the broad/final boundary if the editor reports
  ready; G12 currently changes no Unity-facing DTO or reader.
- BossOmen cross-validation must be split by authored source for diagnostics: an Abnormality's
  `omen_chain_id` failure belongs to the abnormality file, while chain boss/step targets belong to the
  BossOmen file. Combining them into one file-agnostic error would lose the correction location.
- Final bundle validation order is part of deterministic diagnostics: source-local RewardEffect
  validity precedes PVE reward-policy consumption. Otherwise one malformed Reward can be wrongly
  attributed to every downstream encounter that references it.
- The complete Reward dependency order is more precise: Equipment pool definitions validate before
  RewardEffects that consume them, and RewardEffects validate before PVE reward-policy consumers.
  This is not cosmetic ordering; it determines which authored object the generator must repair.
- `BasicAttackDef` now exposes a fallible primitive so Abnormality and CorrodedEmployee can share one
  local shape authority. The legacy panic wrapper is temporarily retained only until the
  CorrodedEmployee raw-to-runtime conversion is made fallible; it is not G12 completion state.
- The temporary BasicAttack panic wrapper is now removed. CorrodedEmployee confirms the durable
  pattern for custom authored resolution: raw transformation returns Serde errors, the resolved
  database returns `DataValidationError`, and indexes are lazy derivatives after both succeed.
- Skill follows the same raw/resolved split and establishes dependency order: Buff and Skill
  definitions validate before Skill-to-Buff references, then later Fragment/unit consumers validate
  against the accepted Skill catalog. Projectile/TileArea runtime wrappers are assertions over data
  already proven by this publication boundary, not separate authored acceptance rules.
- Skill consumers are validated by authored source rather than target catalog: Fragment failures map
  to the Fragment file, Abnormality response/Skill failures map to Abnormality, and CorrodedEmployee
  Skill failures map to CorrodedEmployee. One shared DefenseRoute helper prevents range-rule drift.
- Generated combat preview is part of bundle acceptance, not a post-publication smoke assertion. Its
  typed error is mapped to the PVE encounter source because that authored encounter is the generation
  root; stage/seed/template interaction remains explicit in the cause.
- `from_validated_parts` is private and reachable in production only after `ParsedGameplayRon` final
  validation. It is not a general bypass or compatibility path. `GameDataBase::new()` remains the
  fixture convenience boundary and unwraps the same validators for invalid hand-built data.
- Strictness is semantic, not blanket annotation removal. Skill identity/timing/action fields are
  required because all generated live records author them and omission conceals generator output;
  condition/repeat/presentation/range-option defaults remain because omission intentionally means the
  canonical neutral behavior.
- Abnormality/CorrodedEmployee apply the same rule: core combat values that every generated record
  authors are required in raw RON, while empty traits and optional Skill/omen/Fragment references
  retain omission as intentional gameplay meaning. Abnormality raw deserialization is now lazy so
  semantic validation remains source-aware rather than panicking inside Serde conversion.
- CorrodedWave role weight/cost/count/tier are authored balance, not neutral optional behavior.
- PVE manual enemy `tier`/`count` and wave `time_ms` are authored combat values. Their old defaults
  were fixture convenience and are not part of the generated schema; optional references and the
  canonical `required_for_victory = true` rule remain independently classified semantics.
- Unity MCP was available for final cross-project regression evidence. G12 changes no Unity DTO or
  reader, but the actual ark EditMode suite passed 188/188 against the shared workspace state.
- Generated preview validation necessarily consumes an assembled `GameDataBase` candidate. The
  all-or-nothing boundary is therefore return/publication, not allocation: local/cross validation,
  private candidate assembly, generated validation, then successful Arc return. No candidate escapes
  on failure and no second compatibility loader is introduced.
  Requiring explicit values prevents invisible balance changes when generator fields are missing.
