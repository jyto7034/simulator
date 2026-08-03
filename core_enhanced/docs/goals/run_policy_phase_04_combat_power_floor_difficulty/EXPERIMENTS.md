# Certified Combat Power And Floor Difficulty Experiments

Status: complete

Implementation complete: true

Created: 2026-07-27

## 2026-07-27 — Readiness audit

### Predecessor and baseline

- P03 completion review: Complete / High.
- Core baseline after public facility version convergence: 761 unit + integrations 3/4/23/13/4 = 808 pass.
- `cargo check -p game_server`: pass.
- Unity baseline: EditMode 254/254, non-live PlayMode 62/62, compile error 0.

### Current implementation evidence

- `UnitCombatProfile` and battle stat pipeline already resolve base stats, equipped skill fragments, weapon
  interval/range/block, level/growth, equipped permanent effects and enhancement modifiers.
- The current battle pipeline also applies active consumables and Trauma-derived starting HP; certified power
  needs a sibling persistent-only projection instead of reading final battle current HP.
- `RunPolicyData.floor_combat_scaling` selects by Floor number, clamps overflow to the final stage, and supplies
  enemy stat/generated-wave multipliers directly to combat preview.
- Facility assignment profiles have no difficulty reference and `MapNode` stores no band/role/target/reward
  contract. Checkpoint/active-run therefore cannot preserve such decisions yet.
- Unity has no P04 node band/relative threat/readiness model or presentation.

### Readiness verdict

- Provisional coefficients and profile values are authorized balance authoring, not an unresolved policy choice.
- Required Core/server/Unity/persistence surfaces are accessible and P03 seam is ready.
- Result: ready; begin schema and persistent-profile design without a compatibility layer.

### Dirty overlap gate

- Core power/scaling/stat/map/world files, game_server actor files and Unity Core/Presentation/View/Test paths are
  already dirty from the active P00-P03 campaign and older user work. Several P03 files are untracked because the
  campaign has not been committed; their current green state is the working baseline.
- `run_policy_data.rs`, `policy.ron`, `combat_preview`, `battle/stat_pipeline`, `map/types`, `world/state`,
  active-run, snapshot and Unity node-map paths materially overlap P04 and cannot be avoided.
- Current diffs are internally consistent with the completed predecessor reviews and broad tests. P04 will extend
  those exact current files with scoped patches, will not reset/stash/restore them, and will not run repository-wide
  formatting or cleanup.
- No independently identifiable conflicting power/difficulty implementation exists in the dirty diff. Overlap is
  safe to continue under the campaign-owned source-of-truth migration.

## 2026-07-27 — Production implementation and validation

### Live authority and certified power

- Replaced `floor_combat_scaling` with required `combat_difficulty`: model version, non-zero power weights
  including explicit level contribution, threat thresholds, Standard/Endless anchor schedules, and complete
  ordinary/role profiles. Assignment profiles now require a cross-validated `difficulty_profile_id`.
- Added persistent-only roster certification. It includes level, effective permanent stats, equipped
  equipment/enhancement, equipped fragments, interval/range/block/active skill; Trauma/current HP/Breakdown and
  active consumables do not change raw power. Dead employees are excluded.
- Focused evidence: combat-power 3/3; persistent loadout integration 1/1; anchor formula 1/1; policy rejection
  1/1. A focused test initially mutated profile metadata instead of the effective battle profile and failed to
  raise power; the fixture was corrected and reran green.

### Floor manifestation, combat and rewards

- Facility generation now fixes anchor, deterministic largest-remainder ordinary band quota, typed special role,
  target power and model/profile identity on every Combat/Boss node in the same creation transaction.
- Preview/runtime consume the stored node contract for enemy stat scale and generated-wave budget. A manifested
  Combat/Boss without authority rejects; same authority at different Floor indices yields the same scale.
- Quantity-bearing combat rewards use deterministic nearest-integer percentage scaling with a minimum of one.
  Singular equipment/pool/fragment identity grants stay singular; policy validation requires Hard and all roles
  to exceed Standard reward multiplier.
- Focused evidence: reward scaling 2/2; stored stat scale 1/1; generated-wave budget 1/1; missing authority 1/1;
  facility quota/final boss and BossOmen capstone focused tests green.

### Monotonic persistence and projection

- `RunPowerAuthorityState` initializes after starter/loadout commit, records max only on committed Combat entry,
  and manifests each new Floor from `max(authored minimum, previous anchor + rise, run peak)`.
- Checkpoint payload and battle-entry rollback do not own peak/anchor. Active-run envelope v2 signs model/peak/
  anchor separately; Core rollback and server rolling backup max-merge monotonic values.
- Core map DTO publishes band/relative threat/readiness only and omits model/profile/anchor/target. Readiness can
  change without rewriting node authority. Unity reader, marker VM, detail rows and compact threat glyph consume
  only those qualitative fields.
- Focused evidence: Core facility checkpoint group 9/9; rollback 1/1; server active-run store 8/8; DTO secrecy and
  dynamic readiness 1/1; Unity model/presenter 2/2; UGUI PlayMode marker 1/1; compile Console errors 0.

### Broad regression

- First `cargo test --lib -- --nocapture`: 770 pass / 5 fail. All five failures were manual recovery-map fixtures
  that created Combat nodes with `combat_difficulty: None`; production generation was not implicated. Fixtures
  were upgraded to explicit Standard/Elite typed authority and the focused failures reran green.
- Final `cargo test --lib`: **775 passed, 0 failed, 0 ignored**.
- `game_server cargo test`: **52 passed, 0 failed**; doc tests 0/0.
- Unity MCP broad EditMode job `07197ac99b074dadb7e9a530d1fe712d`: **254 passed, 0 failed, 0 skipped**.
- Unity MCP focused PlayMode job `6b0fc75ee5284a31a51ac0c1f38b31e4`: **1 passed, 0 failed**.
- P03 facility determinism/assignment/save paths are included in the 775 Core tests and 254 Unity EditMode set;
  no reroll, topology or version regression remained.
