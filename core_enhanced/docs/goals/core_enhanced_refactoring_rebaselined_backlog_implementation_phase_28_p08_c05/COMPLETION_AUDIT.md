# Phase 28 CRA-P08-C05 Completion Audit

Audit Mode: phase_exit
Audit Target: Phase 28 — CRA-P08-C05 One Committed-Damage Authority
Current Status: active
Implementation Complete Claim: false
Candidate / Work Type: CRA-P08-C05 / behavior-preserving refactor
Policy Source: CRA-POL-001, CRA-POL-002 and CRA-POL-006 in the approved final policy record
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Master Registry: Phase 28 active; Phase 29 CRA-P09-C05 pending on Phase 28 `Complete / High`
Repository / Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Dirty State Before Audit Artifact: 516 expanded rows, normalized C-sort SHA-256
`8c67776899e009651dfb5e75110fbae4926c9344ce2e251f4e45be6de7f3c83d`
Audit Date: 2026-08-10 (`Asia/Seoul`)

## Audit Procedure And Independence

- This is a separate read-only review pass after implementation evidence was stabilized. Production code, live
  data, public contracts and canonicals were not corrected during the audit.
- The complete 874-line `goal_authoring_and_execution_guide.md`, complete 567-line
  `goal_completion_review_guide.md` and complete 198-line resume attachment were reread in bounded sections before
  the verdict.
- The finalized 222-line PLAN, 223-line experiments and 135-line notes were then reread from beginning to end.
  Master Phase 28/29 registry and phase-exit gate were checked separately.
- Current production authority, all three committed-damage callers, formula, occurrence dispatcher, HP/death and
  resonance owners, public carriers, live data, server/Unity consumers, static removals and dirty state were
  inspected directly. Existing implementation claims were used only as pointers.
- Fresh audit-time focused tests were executed after those reads. The earlier broad results remain current because
  no source, test, data, dependency or environment changed between them and this audit; only Goal evidence text was
  updated.

## Approved Policy Restatement

This Phase may consolidate internal orchestration only. The public `calculate_damage` function remains a pure
arithmetic authority, while committed HP and normal death remain owned by the existing runtime mutation path.
Instant and projectile basic attacks must preserve launch/source snapshots, live target state at commit, explicit
per-occurrence identity, basic-only OnAttack/OnHit behavior, actual-delta resonance and established event order.
General skill, area, Buff and retaliation leaves must not acquire basic policy. CRA-POL-001 requires one causal
normal death/kill path; CRA-POL-002 requires owner-attributed non-recursive retaliation; CRA-POL-006 requires public
Rust and serialized compatibility when deployed consumers are unknown.

The current canonical contract already describes the behavior that must remain. The approved target changes only
the private ownership structure. No gameplay, arithmetic, data, save, DTO, wire or deployment migration is part of
the target policy.

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy basis | Runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| P28-C01 | Complete formula/commit/trigger/resonance/death/caller graph identified | Phase objective; CRA-POL-001/002 | `damage.rs`, `damage_runtime.rs`, `triggers.rs`, `death_runtime.rs`, `resonance_runtime.rs`; all producers inventoried | static caller searches plus commands 42, skill 14, Boss 22 | pass |
| P28-C02 | Dirty targets and concurrent work preserved | authoring guide dirty gate | activation and pre-audit status are the same 516 rows/hash; protected marker exact | `git diff --check`, status/hash and marker checks pass | pass |
| P28-C03 | Old behavior characterized before production refactor | behavior-preserving boundary | three new observable tests passed on old production before authority edit | old formula 22, commands 39 then 42, skill 14, live item 4; 0 failed/ignored | pass |
| P28-C04 | One private committed-damage authority surrounds retained formula | Phase target authority | private typed input/policy; three callers; sole mutating formula and HP-commit calls | negative duplicate and removed-name searches; audit commands 42 and formula 22 | pass |
| P28-C05 | Source/target/roll/delivery lifetime is exact | delivery invariant | explicit occurrence UUID/time, retained source snapshot, live active-target read; motion stays caller-owned | drift/crit parity tests, detached-source/projectile tests and Boss 22 | pass |
| P28-C06 | Occurrence, retaliation and effect-before-activation order are exact | CRA-POL-002; Phase 11/13 seam | typed Basic policy alone dispatches OnAttack/OnHit; dispatcher still owns classification; retaliation emits GeneralCommand leaf | command family includes occurrence/order/fatal/projectile retaliation and no-recursion cases | pass |
| P28-C07 | HP/death/resonance/lineage is exact | CRA-POL-001 | actual delta precedes dealt-resonance calculation; retained HP helper records one HpChanged and invokes one death owner | overkill/single-death/parent-lineage test, command 42 and result stats 7 | pass |
| P28-C08 | Carrier retention/removal and public compatibility are closed | CRA-POL-006 | obsolete private snapshots removed; public request/context/result/command/snapshot retained with consumer disposition | public/static searches and Core/server compile/broad tests | pass |
| P28-C09 | DTO/RON/save/server/Unity compatibility is exact | no-migration policy; CRA-POL-006 | no serialized/private-plan change; live data and canonical hashes equal activation; server pass-through and Unity readers unchanged | live item 4, RON 25, Core/server broad debug/release | pass |
| P28-C10 | Focused and broad debug/release/static validation is green | Master validation gate | current source/data state matches the tested hashes | Core 1,070 debug/release; server 59 debug/release; checks/fmt/Clippy exits 0 | pass |
| P28-C11 | Downstream boundary and independent audit are current | Master linear order | no Phase 29 directory, placeholder or production work; private committed-hit seam is available to Phase 29 | this independent audit: `Complete / High` | pass |

## Runtime And State Evidence

### Current Authority Path

| Entry class | Caller-owned work | Shared committed authority | Authoritative mutation |
| --- | --- | --- | --- |
| instant basic | validation/range, release resonance, occurrence UUID/time, autocast scheduling | `execute_committed_damage(BasicAttack { ... })` | `apply_damage_result_and_record` then `finalize_unit_death` if lethal |
| basic projectile impact | movement, collision, miss/hit log, impact UUID/time, autocast scheduling | `execute_committed_damage(BasicAttack { ... })` | same retained HP/death path |
| general `ApplyDamage` | skill/area/Buff/retaliation producer lifecycle and explicit source snapshot | `execute_committed_damage(GeneralCommand)` | same retained HP/death path |

- `damage_runtime.rs::execute_committed_damage` is private to `crate::game::battle::core`. Its typed policy makes
  basic versus general behavior unrepresentable as an accidental source-enum inference.
- The function reads target owner, defense, magic resistance, incoming modifiers and HP only from the current
  active target at commit. `DamageSourceSnapshot` continues to carry source-side, source-attack, base damage,
  release-time OnAttack damage inputs and stable roll identity.
- Basic policy explicitly receives occurrence ID and occurrence time. Instant uses the source-instance UUID and
  snapshot commit time; projectile uses projectile UUID and impact time. General policy has no occurrence fields.
- Snapshot OnAttack damage inputs feed the formula even after source death. Only a currently active source releases
  OnAttack targeted effects/activations; the current active target supplies OnHit. This preserves the established
  committed-versus-released lifetime rather than silently normalizing absence.
- Trigger occurrence classification and retaliation assembly remain in `triggers.rs`. Effects are batched before
  activations. Retaliation snapshots the defender as owner/source and produces a non-recursive general
  `ApplyDamage` with OnAttack snapshot collection disabled.
- `calculate_damage` is called once by the committed authority. The other production formula callers are read-only
  target-usefulness/tactical-stance previews and cannot mutate battle state.
- Actual damage is `target_current_hp - target_remaining_hp`. Basic dealt resonance uses that value; the retained HP
  helper derives received resonance from its actual before/after delta. Overkill therefore cannot grant planned
  damage resonance.
- `apply_damage_result_and_record` remains the only direct committed-damage HP application caller. It records the
  metadata-complete `HpChanged` and invokes `finalize_unit_death` once at zero HP. The formula's public `UnitDied`
  command remains a Core no-op and cannot become a second lifecycle path.
- Death finalization still owns lifecycle, movement interruption, Buff cleanup, graveyard snapshot, `UnitDied`,
  causal killer, death/kill/ally-death occurrences and same-time nested command lineage.

### Formula And Carrier Disposition

| Surface | Current disposition | Audit result |
| --- | --- | --- |
| `calculate_damage` | retained public pure formula | one mutating caller plus read-only preview callers |
| `DamageRequest` / `DamageContext` / `DamageResult` | retained public Rust carriers | no field narrowing; unknown deployed readers respected |
| `DamageSourceSnapshot` / `BattleCommand` | retained public Rust carriers | no shape/variant removal; all Core behavior classified |
| formula `BattleCommand::UnitDied` output | retained compatibility behavior | explicitly non-authoritative in Core; removal would change public output |
| old basic private snapshot/resolution layer | removed | all old symbol searches return no match |
| committed private input/policy | new internal seam | not serde/save/wire and only three intended callers |

The retained public no-op death signal is not an unexamined private legacy. CRA-POL-006 forbids changing its public
formula output on workspace-only consumer evidence. Its explicit compatibility disposition and non-authoritative
Core handling close the reported ambiguity without creating a second death owner.

## Data Evidence

- Live Equipment still contains OnAttack current-hit/targeted effects and OnHit Counterpart retaliation; SHA-256 is
  the activation value `322d7455a256082f03804c0dcc7a77a4caa3a06919d5e34b667042b411bcd1a6`.
- Live Skill still exercises instant, projectile, tile-area and persistent-area damage; SHA-256 remains
  `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`.
- Live Buff periodic damage remains unchanged at
  `e7cc335f4243b4fc0ac907a72356ba44a1eae2669aa340147a0fbbc14f9c448b`.
- No schema, value, loader, validation, encounter or catalog migration occurred. `ron_loading` executes 25 current
  production loading tests and `live_item_skill_activation` executes 4 live trigger tests.

## DTO, Transport And Persistence Evidence

- The new committed input and policy are private, non-serde battle-runtime values. They are absent from setup,
  checkpoint V3, save, replay/export and wire DTOs.
- `HpChanged` production events retain source, target, actual delta, before/after HP, reason, damage source/type,
  raw/final damage, typed breakdown, critical flag and feedback tags. `UnitDied` retains killer and position data.
- `game_server` depends on and forwards Core battle event/checkpoint shapes; it does not construct the public damage
  calculation carriers independently. Server source and canonical hash are unchanged.
- No save/checkpoint version change, dual-read/write, schema alias, fallback or rollout sequencing is required.
  Core, server and accessible Unity remain an unchanged producer/forwarder/consumer deployment contract.

## Consumer Evidence

- Confirmed workspace consumers: instant/basic-projectile commands, skill instant/projectile/tile/persistent-area
  leaves, Buff ticks, retaliation, read-only usefulness/tactical previews, result/death aggregation and Boss runtime
  released work.
- Confirmed server consumer: Core event and checkpoint pass-through compiles and its 59-test debug/release suites
  pass without a damage-carrier adapter.
- Canonical/accessed external consumer: Unity `BattleDamageFeedbackModel` reads `HpChanged.feedback_tags`,
  `final_damage` and `damage_breakdown`; Unity transport canonical uses `UnitDied` for inactive presentation. Static
  runtime search confirms those readers and their contract tests remain present.
- Inaccessible or unknown consumer: independently deployed Rust code may construct/read the public formula carriers.
  It was not claimed as verified; its risk is neutralized for this Phase by retaining every public field, variant,
  function and observable result shape under CRA-POL-006.
- Explicitly excluded consumers: none required beyond the no-migration boundary. A live WebSocket probe and Unity
  runner were not required because no producer/serialized/server/client source changed and their exact activation
  hashes plus Core/server compilation remained stable.

## Legacy, Fallback And Placeholder Audit

- Search returns no `calculate_basic_attack_damage_snapshot`, `AttackTargetSnapshot`,
  `BasicAttackDamageSnapshot` or `BasicAttackDamageResolution` in production or tests.
- The basic projectile leaf contains no `TriggerOccurrenceContext`, `DamageContext`, `calculate_damage` or
  `RuntimeCommandBatch`; its duplicated orchestration is gone.
- Static calls show exactly three `execute_committed_damage` entries and only one
  `apply_damage_result_and_record` call, inside that authority.
- No public field/variant removal, replacement alias, dual damage path, fallback, silent normalization,
  ignored/disabled characterization test, fixture-only branch or debug bypass was introduced.
- The Phase 29 Goal directory does not exist at audit time. No P09-C05 motion kernel, placeholder DTO/state or
  future-seam production work was prebuilt.
- The disconnected public `battle::death::DeathHandler` remains intentionally owned by Phase 38 CRA-P08-C06 and
  was neither used to satisfy this Phase nor partially cleaned up.

## Scoped Diff And Dirty Preservation

- Phase 28 production attribution is limited to the already-dirty `commands.rs`, `damage_runtime.rs` and
  `basic_attack_projectile.rs`; tests are colocated in `commands.rs`. Final hashes are respectively
  `47a371a85a5d42bc899d04fcb92523c8044200600e57e6c058ec1b980f10d1fc`,
  `7b4d43792b45b08b0b3c872813059ff1ded233736e67e9c47ef11cdab154f7cb` and
  `54d1d95c72da0f17ab98d2f5cb700dbfbce457718a0b1b721335545db14258d1`.
- Formula, trigger, death, resonance, skill-delivery, tests, data and canonical paths that were not Phase edits retain
  their prospective activation hashes. No broad formatter, reset, restore, clean or stash was used.
- Immediately before this audit artifact, expanded state remains the exact activation 516-row set/hash. Creating
  this required audit adds only its own Goal-document path; it does not represent production or user-work drift.
- `git diff --check` and `cargo fmt --all -- --check` are green.
- Protected `game_server/logs/app.log.2026-08-09` remains exactly 57 bytes, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with unchanged mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`. The Phase 27 one-path approval was not generalized.

## Canonical Alignment

- `core_runtime_contract.md` remains at `6e20d6a6...`, `game_rulebook.md` at `6c53ecd5...`,
  `data_loading_contract.md` at `30ed5fd5...`, server bridge canonical at `4d734af0...` and external Unity Core
  canonical at `a3224e24...`, exactly matching activation.
- No canonical update is required: formula, HP/death, trigger/retaliation, event metadata/order, data and external
  contract meaning did not change. The new authority is private implementation ownership, documented by this Goal
  rather than promoted into a new public gameplay contract.
- Current runtime, current canonical and approved target policy therefore agree: behavior is unchanged and private
  authority duplication is reduced.

## Validation Results

All results below were produced in the current dirty worktree. `ignored` is reported as skipped where applicable;
every counted suite reported zero ignored.

### Pre-Change Baseline And Characterization

- `(Core) cargo test --lib game::battle::damage::tests --quiet`: 22 passed, 0 failed, 0 ignored.
- `(Core) cargo test --lib game::battle::core::commands::tests --quiet`: 39 passed, 0 failed, 0 ignored before new
  characterization; 42 passed after the three test-only additions and before production consolidation.
- `(Core) cargo test --test skill_runtime_contract --quiet`: 14 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test live_item_skill_activation --quiet`: 4 passed, 0 failed, 0 ignored.
- Each new characterization was also selected individually: 1 passed, 0 failed, 0 ignored for cross-delivery
  drift/crit; 1/0/0 for actual-overkill/single-death/lineage; 1/0/0 for general-command basic-policy exclusion.
- Baseline failures: none. New failures introduced by characterization: none.

### Post-Implementation Focused

- `(Core) cargo test --lib game::battle::core::commands::tests --quiet`: 42 passed, 0 failed, 0 ignored.
- `(Core) cargo test --lib game::battle::damage::tests --quiet`: 22 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test skill_runtime_contract --quiet`: 14 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test live_item_skill_activation --quiet`: 4 passed, 0 failed, 0 ignored.
- `(Core) cargo test --test ron_loading --quiet`: 25 passed, 0 failed, 0 ignored.
- `(Core) cargo test --lib game::battle::core::boss_runtime::tests --quiet`: 22 passed, 0 failed, 0 ignored.
- `(Core) cargo test --lib game::battle::result_stats::tests --quiet`: 7 passed, 0 failed, 0 ignored.
- Release reruns: commands 42, skill runtime 14 and Boss runtime 22 passed, all 0 failed/ignored.

### Broad, Compile And Static

- `(Core) cargo test --all-targets --quiet`: 1,070 passed, 0 failed, 0 ignored. Calculation is 1,019 library plus
  official integration binaries of 4 + 4 + 25 + 14 + 4; zero-test binaries are excluded from evidence.
- `(Core) cargo test --release --all-targets --quiet`: the same 1,070 passed, 0 failed, 0 ignored.
- `(server) cargo test --all-targets --quiet`: 59 passed, 0 failed, 0 ignored; four binary targets selected zero and
  are excluded.
- `(server) cargo test --release --all-targets --quiet`: the same 59 passed, 0 failed, 0 ignored.
- `(Core) cargo check -p game_core --all-targets` and `--release --all-targets`: both exit 0.
- `(server) cargo check --all-targets` and `--release --all-targets`: both exit 0.
- `(Core) cargo clippy -p game_core --all-targets`: exit 0 with established non-strict warning debt (82 library,
  135 library-test, plus integration warnings); no new committed-damage authority warning.
- `(server) cargo clippy --all-targets -- -A warnings`: exit 0. The first unbounded diagnostic output was discarded
  rather than treated as a result; this concise rerun is the recorded compile gate.
- `(Core) cargo fmt --all -- --check`: exit 0. `(root) git diff --check`: exit 0.

### Fresh Audit-Time Revalidation

- `cargo test --lib game::battle::core::commands::tests --quiet`: 42 passed, 0 failed, 0 ignored.
- `cargo test --lib game::battle::damage::tests --quiet`: 22 passed, 0 failed, 0 ignored.
- `cargo test --test live_item_skill_activation --quiet`: 4 passed, 0 failed, 0 ignored.
- `cargo test --test skill_runtime_contract --quiet`: 14 passed, 0 failed, 0 ignored.
- `cargo test --lib game::battle::core::boss_runtime::tests --quiet`: 22 passed, 0 failed, 0 ignored.
- `cargo test --lib game::battle::result_stats::tests --quiet`: 7 passed, 0 failed, 0 ignored.
- `cargo test --test ron_loading --quiet`: 25 passed, 0 failed, 0 ignored.

### Omitted Validation

- Live WebSocket probing was omitted because no DTO, serialization, server mapping or ordering contract changed;
  unchanged hashes and server debug/release compile plus 59-test suites directly cover the boundary.
- Unity Edit/PlayMode runners were omitted because no Unity or wire source changed. The accessible runtime consumer
  and canonical were statically reread and remain hash-identical; Core producer and server forwarder are green.
- No performance benchmark is registered for this private orchestration consolidation, and damage arithmetic,
  projectile motion and area algorithms are unchanged. Functional and broad suites are the applicable gate.

## Baseline Failures

None. Every required pre-change focused suite and every post-change focused/broad suite passed. There is no baseline
failure being reclassified as success. Non-strict Clippy warnings are established diagnostic debt, not test
failures, and strict warning denial was never claimed.

## Cross-Component Findings

- No competing source of truth exists across instant basic, basic projectile and general command execution; all
  converge on the typed private authority and retained HP/death owner.
- Skill, Buff and CRA-POL-002 retaliation producers continue to emit general leaves; they cannot accidentally enter
  the basic trigger/resonance branch.
- Boss runtime continues to release ordinary attack/skill work and introduces no formula or death fork.
- Public event/checkpoint and Unity presentation fields are unchanged, so the Core/server/client seam has no
  temporary schema, compatibility shim or version conflict.
- Phase 29 receives a stable committed-hit boundary while retaining ownership of projectile/movement lifecycle.
  No material cross-component finding remains.

## Long-Term Direction Review

| Candidate ID | Work type | Current to target authority result | Preserved invariant result | Removal/consolidation result | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P08-C05 | behavior-preserving refactor | three overlapping committed assemblies become one private typed authority around one pure formula and one HP/death owner | source/live-target, roll, occurrence, retaliation, resonance, death, lineage and public compatibility remain exact | duplicate projectile/general/private snapshot orchestration removed; public compatibility surfaces intentionally retained | characterization-first 3; focused 42/22/14/4/25/22/7; broad Core 1,070 and server 59 debug/release | pass |

- Fit: High.
- Improvement class: none.
- Reason: ownership is materially simpler, policy distinctions are typed, existing domain owners remain intact,
  tests protect observable behavior rather than helper names, and no parallel schema, compatibility shim or future
  placeholder was introduced.

## Detailed Findings

- Material findings: none.
- Nonmaterial observation: unknown deployed Rust consumers remain uninspectable, so the broader public damage
  carriers and formula `UnitDied` output remain intentionally compatible. This is the required CRA-POL-006 outcome,
  not a deferred removal claim.
- Nonmaterial observation: repository-wide non-strict Clippy debt remains outside Phase 28. It does not name the new
  authority and all required compile/test gates pass.
- No policy, scope, external dependency or correction decision is required.

## Primary Verdict

- Primary verdict: Complete.
- Long-term fit: High.
- Improvement class: none.
- Combined phase-exit verdict: `Complete / High`.

Every Phase 28 completion condition has current runtime/data/contract and nonzero validation evidence. There is no
material legacy, unauthorized compatibility, unverified required consumer, scope violation or next-Phase prebuild
hidden behind this verdict.

## Status Transition

- from: `active`, `Implementation Complete: false`.
- to: `complete`, `Implementation Complete: true`.
- Master Phase 28 registry: may transition to `complete — Complete / High` after this audit is recorded.
- Phase 29 readiness: allowed. It may become `ready` in the same record-only transition, but its production work
  remains forbidden until its own three documents are initialized, registered as the sole active Child, reread and
  prospective fingerprints are captured.

## Remaining Risk And Required Action

- Preserve all public carrier shapes until a separate consumer/migration policy provides stronger evidence.
- Preserve the 57-byte Phase 27 approval marker exactly; it is not a Phase 28 artifact or a general cleanup waiver.
- Keep Phase 29 limited to CRA-P09-C05 motion ownership. Do not reopen damage arithmetic, trigger, death or public
  carrier policy during activation without a new explicit stop-condition decision.
- Immediate correction: none.
