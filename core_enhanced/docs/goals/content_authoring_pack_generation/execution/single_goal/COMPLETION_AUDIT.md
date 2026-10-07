# R02 independent GEX-20 completion audit

**Primary verdict: Not Complete.** Finding **R02-AUDIT-F01** fails the required source-diagnostic portion of C03 / SC01. The invalid candidate is rejected, but a late generated combat failure caused by a winning custom RunPolicy definition is attributed only to the official encounter file. This verdict belongs to `ATR-CAPG-R02-002`, not a later corrected tree.

## Exact target, authority and separation

Selected child: `(GS-CAPG-P2G50-001, GOAL-CAPG-01)`, `PHASE-CAPG-01`, assigned `AG-CAPG-01` and `WP-CAPG-03`. Basis: `WR-CAPG-P2G20-001`, `FPF-CAPG-P2G30-001`, `PP-CAPG-P2G40-001`, passed receipt `PVR-CAPG-P2G40-001`, Handoff `HOF-CAPG-P2G60-001`. The [Goal](../../P2G_50_GOAL_SET_001.ko.md), exact Freeze/Plan/Handoff, PLAN, current EXPERIMENTS/EXPERIMENT_NOTES and correction002 were read. The six Handoff source/context hashes match. Goal Quality Contract 2 applies; this is `single_phase`, so the entire fourteen-member acceptance, eight IVA rows, ten required dimensions, eight completion conditions and applicable final broad baseline belong here. No GEX-30 can complete missing R02 work later.

Authority is the user's adopted [AUTORUN request](../../../../roadmaps/content_authoring/AUTORUN_REQUEST.ko.md), especially §§3/6/8, and the bounded audit delegation. Applied GEX Reader Contract, §5, §6.4, §6.7 and GEX-20 are revision `2026-09-09-gex-r1` in `docs/goal_execution_pipeline(1).md`.

Reviewer `/root/r02_fresh_audit` is a separate same-model agent with no conversation fork, uninvolved in implementation and the R01 successor audit. This is neither human nor different-model review. The implementation owner remained stopped. Only this audit, new `audit-evidence/fresh_001/` files and isolated audit-owned `/tmp` probes were written. Source, tests, data, canonical contracts, upstream artifacts, prior target/audit/evidence, PLAN and parent views were not edited. The owner's evidence map was checked against actual bodies and logs, not treated as a prescribed verdict.

Repository `/mnt/f/work/simulator`, branch `cleanup/simulator-dirty-worktree-20260803`, HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`. The immutable [target002](AUDIT_TARGET_002.json) SHA-256 is `18ae4137394def5a0454b6cd7d69e42c9570bd207fc1e7170574a9ee864818ee`. [Entry](audit-evidence/fresh_001/currentness_begin.json) and [closure](audit-evidence/fresh_001/currentness_end.json) independently compare all 581 direct members, 106 external fixture members and two default-release binaries. **Currentness: current / unchanged.** A current target can still fail completion.

## Completion-blocking finding

### R02-AUDIT-F01 — late generated failure loses its actual contributing source

**Owner:** R02 `GOAL-CAPG-01`, `AG-CAPG-01` implementation and `WP-CAPG-03` validation, correction GEX-10. This is an existing required diagnostic contract, not a new policy question or an R03 feature.

**Requirement:** the Goal's Code/Data/Schema/Store/Transport/Consumer Boundary and SC01 retain the winning definition's original pack/file/field locator; C03 requires a source error for invalid/late compilation candidates. The implemented canonical [Data Loading Contract](../../../../data_loading_contract.md:40) explicitly says winning original locations are retained and aggregate errors identify contributing sources instead of misleadingly selecting one file.

**Actual control flow:** [content_pack/mod.rs](../../../../../src/game/data/content_pack/mod.rs:61) wraps every generated-preview error with the fixed `data/pve/encounters.ron` source. [validation.rs](../../../../../src/game/data/validation.rs:1400) constructs manifested difficulty cases using RunPolicy, then maps preview/setup errors solely to the current encounter and `encounters[index]`. `SourceTable::relocate` consequently finds the encounter's source and returns, losing the winning policy source. For this cross-domain failure, the encounter remains useful context but is not the complete source attribution.

**Independent default-release reproduction:** [script and input/output identities](audit-evidence/fresh_001/source_diagnostic2_probe.json), [reproduction script](audit-evidence/fresh_001/source_diagnostic2_probe.py), [actual stderr](audit-evidence/fresh_001/diagnostic2_invalid_b_1.stderr). Copy the frozen AB003 external official+B packs to an isolated directory. The existing B manifest explicitly replaces RunPolicy. Change only `b/policy.ron`, at `combat_difficulty.profiles[1].ordinary_bands[0].tuning.extra_waves[0].spawn_zone_ids[0]`, by adding an otherwise valid timed CorrodedEmployee wave naming a nonexistent spawn zone. Run the frozen `target/release/consume_content_pack` with `official b`.

Both negative invocations exit 1 and return:

- source file: `official/data/pve/encounters.ron`
- object: `normal_corroded_patrol`
- field: `encounters[0]`
- cause: manifested `default_facility::Ordinary(Easy)` generation failed because a spawn wave references a missing spawn zone.

Neither diagnostic identifies `b/policy.ron`, its original tuning/extra-wave field, nor a contributing-source list. The official encounter bytes are unchanged. Valid B succeeds before the mutation and after restoration; complete valid stdout is identical. The two errors are identical and the probe verifies no input mutation during load. This is a reproducible diagnostics defect, not successful publication or live-state corruption.

The current [late-generated test](../../../../../src/game/data/content_pack/tests.rs:638) modifies `official/data/run/policy.ron` but explicitly asserts `official/data/pve/encounters.ron` at line 680. Its non-publication and A-preservation assertions are meaningful; its source oracle codifies the attribution gap. Therefore the 1,214 passing owner tests do not discharge this required behavior.

**Required correction:** retain origin/contributor context across generated preview/setup validation and resolve it to actual winning external source(s). For this fixture, the policy file and affected field must be identifiable; preserve the encounter context when useful. Add a real separate custom policy replacement regression, keep no-publication/A-preservation/retry assertions, and audit analogous cross-domain generated failures. Do not weaken the canonical diagnostic promise to match the current error. Restore affected validation, assess prerequisite-currentness impact from the actual fix, freeze a successor target and obtain fresh R02 GEX-20. No implementation was changed in this pass.

## Currentness, prerequisite and preservation

R01 successor target `ATR-CAMEC-R01-002` has SHA-256 `09ccc72d1b8b94fdac8024c65247a72a184228cb6e1eb30172f7889457352c62`; its immutable [audit002](../../../content_authoring_minimal_effect_contract/execution/single_goal/COMPLETION_AUDIT_002.md) has SHA-256 `40bb07c34bd63bb5e081c127fa73e9ce38878c7e771071959d75551b608fecac`, with actual Complete / High and three new external release authoring probes. It is a different fresh reviewer. Historical R01 audit001 remains historical/stale for the successor, not retroactively rewritten.

Independently comparing R01 target002 finds exactly six changed members: R02 PLAN, EXPERIMENTS, EXPERIMENT_NOTES, ACCEPTANCE_EVIDENCE, R01_SUCCESSOR_IMPACT and `data/mod.rs`. All other 562 direct members match. The only code-bearing change is exactly the rustdoc replacement recorded in COMMENT_REUSE_002. Reconstructing the old source from current bytes produces SHA-256 `bc31340706e3a9651599d9eeaa2155cd1fe2aaf3b960c10dc506d1eb76ad28d9`. Current source is `4202f83de4064bf9711d9a194f303d921b77a3d2146372f918e93b208c87ee93`. The new link names the actual public loader. All 279 VI023 inputs match; VI022→023 changes only this comment-bearing file. The five report changes record actual R01 closure/correction routing; original policy, acceptance, source behavior, tests, fixture inputs and results remain exact. Thus **R01 audit002 currentness at this target: current / proven non-impacting delta**, specifically for this comment/report delta. It supplies the prerequisite but does not pre-approve R02 diagnostics. Any subsequent correction needs its own impact decision.

[Independent preservation](audit-evidence/fresh_001/preservation_review.json) verifies all 1,850 baseline originals against `/tmp/capg-r02-baseline`, exactly forty attributed existing-path changes, no removed file, and exact reconstruction of the 52-path patch SHA-256 `20355d9f8e10f0b858505c0398ed3a3ba47bb8ee57bdace5a9a5fa0a69f5984a`. All current source/test/official data paths are represented in VI023. Original integration tests and official RON are unchanged. Parent operating records are separately attributed and outside the implementation target. All twelve [supplemental tool locks](audit-evidence/fresh_001/tool_lock_review.json) match preserved original/current hashes; existing package changes are only game_core's dependency addition, with no removed package. The public observer's original full source hash is reconstructed by removing exactly its one required `skill_effect_source: None` fixture field.

Limits remain explicit: the original 1,850-file baseline excluded tools; eight adapted API main files lack pre-edit byte hashes. Their current bodies and actual behavior/privacy assertions were read, but their pre-edit bytes are not certified. Full git status continuity does not hash every unrelated dirty repository file. These limitations do not erase the exact scoped preservation evidence or justify attributing other dirty work to R02.

## Validation review and new execution

[Validation review](audit-evidence/fresh_001/validation_review.json) checks command/input/log/receipt identities, actual terminal counts, complete output and upstream bindings. Rust/Cargo are 1.91.1; selected build environment variables match the target.

| Evidence reused after independent review | Actual result and limit |
| --- | --- |
| `full_core_002`, VI022 + exact comment proof | `cargo test -p game_core --lib --tests -- --quiet`: 1,214 passed = 1,162 lib + 4/4/25/15/4 integration, zero failed/ignored. Complete concise terminal log retained. Includes 42 pack tests, three generation world tests, seventeen strict V6 tests and broader skill/preview/map/profile/checkpoint regressions. Finding F01 shows why counts alone are insufficient. |
| `all_targets_001`, `release_check_001`, `release_build_005` | Actual all-target and optimized-library checks, then registered default-release packager/consumer binaries, all exit 0 with no warnings. Compile success is not gameplay execution. |
| `release_ab_003` | Ten real subprocesses: four success, six expected rejection. Same binary A/B/A startup yields 500/733/500 currency and complete repeated-A equality. Source/custom/output and binaries match receipts. This measures briefing/starters; actual B combat pin is separately tested. |
| `api_batch_001` + currentness proof | Five actual default-release positive consumers, six specific forbidden-API compiler negatives, offline/locked. Logs show real Rust privacy/unavailable API diagnostics, not Cargo/network failures. Reconstructing the prior private wrapper cfg gives the exact VI020 world.rs hash; both cfg forms exclude it from default release. The other VI020→022 changes are three test modules. No new API execution is fabricated. |
| `public_guards_002`, allocation guard | Original algorithms/caps and substantive assertions retained; complete receipts, 101 timing samples and five allocation samples per size are readable. Snapshot p95 241.039 µs / 48,493 bytes; V6 export 8.770 µs / 18,154 bytes. Allocation comparison keeps 677 allocations / 84,553 peak bytes / zero retained bytes at both history sizes. Instrumented timing is not latency evidence; fixture-capability workloads are separate from production external loading. |
| Resource receipt | Actual 24 catalogs: 408,797 bytes; with source bundle 408,824; packaged input 410,472; largest 114,717. No RSS or product SLA is inferred. |

New execution consists of four complete release subprocesses in the F01 reproduction, with two rejection observations and exact valid/retry/input checks. An earlier audit-local helper completed a valid and invalid invocation before its own assertion incorrectly required the missing zone ID to appear in stderr; those outputs remain, and a new isolated directory was used for the complete four-command sequence. [Audit execution notes](audit-evidence/fresh_001/audit_execution_notes.json) disclose helper/read failures and retries. They are not passing gameplay evidence. No source regeneration, formatting, golden update or broad suite rerun was performed. No mandatory shared initial/final repetition directive applies; the owning final broad baseline is R02 WP03's actual run, not this probe.

## Complete condition, acceptance and IVA review

The following source/test bodies were read, in addition to the scoped patch and original logs. `P` denotes `src/game/data/content_pack/`; `W` denotes `src/game/world/tests/`.

| Condition / assigned IVA | Current direct evidence and disposition |
| --- | --- |
| C01 / IVA01 | `P/packaging.rs`, public load and two registered binary bodies; packaging's three actual-source tests and release A/B/A. Both official and custom inputs use the same post-acquisition validation. Pass. |
| C02 / IVA02 | `P/raw.rs` all 24 adapters/42 variants, `resolver.rs` explicit Add/Replace/Remove writer graph, eight resolver tests and `W/content_generation::valid_external_remove_updates_namespaced_lookup_registry_recipe_and_reward_acquisition`. Real ID/UUID/index/item-registry/recipe/reward acquisition and retained A checked. Pass. |
| C03 / IVA03 | Final winner lowering, `ParsedGameplayRon::into_validated`, cross-pack skill/corroded presets, unsupported losing opcode, dangling buff/reward/registration/missing encounter, late generated no-publication. Ordinary source relocation passes; **late generated source fidelity fails F01**. |
| C04 / IVA04 | `world.rs` gate before candidate install; `world/application.rs` draft/commit includes state, seed, token and prepared output; public briefing gate test and actual external battle20→31 / ArcA→ArcB / SavePoint identity. Deliberate fixture-only missing RunStart exercises real late preparation rollback twice with populated live+dormant compendium/profile; no production fault hook. Pass. |
| C05 / IVA05 | V6-only strict codec, required identity and canonical checksum, shared identity validator before continue/load/cache export mutation. Nineteen original graph mutations retain fresh valid checksums; required field/contract/fingerprint/old version/load-count boundaries and exact UUID stream/next Gate/fresh public app tests. External A/B mismatch, reacquired equivalent A and rollback token tests. Pass. |
| C06 / IVA07 | Exclusive separate packaging path, actual input bytes and custom sentinel preserved; no personal-store access; existing profile/compendium merge and monotonic progress behavior retained through B success/failure/load/rollback. Pass. |
| C07 / IVA06 | `snapshot.rs` no-follow directory-relative opens, bounded UTF-8/nonregular rejection, collision/hardlink inventory, observed metadata/bytes recheck, syntax-depth and checked byte/edge limits; exact/over tests, canonical maps versus sequence order, unchanged retained generations and weak-reference reclamation. Pass for these behaviors; no atomic filesystem snapshot claim. |
| C08 / IVA07–08 | Current full core/release/API/observer evidence and separate actual R01 successor audit restored as above. Required consumers are present; no later Goal needed for R02's public interface. Pass for the regression/prerequisite obligation; F01 still prevents R02 completion. |

| Exact zero-based acceptance member (`PER-CAPG-…-REVIEWED-001`) | Evidence / result |
| --- | --- |
| E01[0] | Release A/B/A, public briefing and actual world battle B: pass. |
| E01[1] | Strict external schema/capability/opcode/path rejection and release fixture API absence: rejection/fallback behavior passes; C03 diagnostic completion retains F01. |
| E02[0] | Real namespaced Add/Replace/Remove through final registry and reward acquisition: pass. |
| E02[1] | Explicit heads/chains, duplicate/missing/cycle/edge tests; shuffled input and numeric execution slots: pass. |
| E02[2] | Only winning references lowered; actual loser missing preset ignored, selected loser/dangling winners reject: pass. |
| E03[0] | A battle state/export/Arc retained while B publishes; B briefing/starters/battle/save identity: pass. |
| E03[1] | Invalid B has no token, forbidden/late preparation restores complete A authority and populated progress: pass for rollback; F01 affects promised diagnostic attribution. |
| E04[0] | Latest roundtrip into fresh app, exact UUID streams and next Gate result: pass. |
| E04[1] | Actual changed content and each contract component with valid checksum reject before mutation: pass. |
| E04[2] | V4/V5 and unknown versions reject repeatedly; old importers/guessed identity/automatic conversion absent: pass. |
| E05[0] | Actual packaging bytes and exclusive existing/source/subtree/symlink output rejection: pass. |
| E05[1] | Populated compendium plus profile progress, failure/load/rollback comparisons: pass. |
| E06[0] | Unsupported/missing/path/race/budget candidates fail before publication with identified acquisition/schema source: pass; broader C03 source promise fails F01. |
| E06[1] | Canonical semantic repeat, comments/receipt distinction, array-order sensitivity, token immutability and reclaimability: pass. |

## Dimensions, structural contracts and boundaries

| Required dimension | Actual review disposition |
| --- | --- |
| success | Full producer→consumer startup, removal/acquisition, A/B combat and V6 continuation observed; pass. |
| invalid_input | Nonzero structural/reference/path/save negatives and deterministic source errors; **incomplete for F01 late generated attribution**. |
| failure_rollback | No token on late setup failure; real draft rejection keeps token/state/seed/profile/compendium/output/cache; pass. |
| retry_idempotency | Repeat invalid loads/starts/restores, valid restoration and A-repeat semantic/output equality; pass. |
| ordering_concurrency | Explicit dependency/writer separation and actual numeric slots; typed canonical map sorting; detected byte/inode/symlink races; A survives B publication. Pass within observed snapshot semantics. |
| boundary_empty_max | Exact/over pack/file/per-file/aggregate/depth/edge/usize, empty/missing/duplicate selection, UUID/local/reference distinctions and save inventory/load limits; pass. |
| disconnect_restart_persistence | Fresh application and equivalent external generation, continue/explicit/cache identity/RNG checks; pass. Network reconnect/deployment are excluded rather than inferred. |
| live_data | Actual official source copies, external release inputs and current diagnostics probe; gameplay path passes, F01 remains a live-data diagnostic failure. |
| producer_consumer | All 24 roots feed final resolver/compiler/generated setup/token/app/battle/save/index consumers; **source producer→diagnostic consumer incomplete under F01**. |
| legacy_fallback_absence | Default-release external loader only, explicit fixture capability, latest V6 reader, strict unknown/opcode negatives, no live generation setter or silent A-as-B success; pass. |

SC01 / I01 hold for typed keys, namespace, writer ancestry, final resolution and ordinary effect-source relocation, but fail the late generated original-source part in F01. SC02 / I02 preserve sequence order in typed semantic bytes, distinct byte receipts, one private Arc-owned database+identity+receipt and reclaimable lifetime. SC03 / I03 preserve existing eligibility and a single prepared generation-aware commit, including before-RunState briefing. SC04 / I04 use the same retained content identity across capture/cache/export/continue/explicit load. SC05 / I06 have actual current R01 evidence and required core consumers; R02 itself cannot be marked complete while F01 remains. I05's profile/personal-file and R01 executor/effect/cast/source/order/cancellation invariants are preserved by source comparison and current regression evidence.

Migration follows approved TR30-01–06: external source publication and latest-only V6 are the authorized successor, while historical R01 V4/V5 preservation describes its old delta. The removed V4 reader certification scaffold was removed, not falsely certified. The original checkpoint semantic corpus hash remains unchanged when projecting out only added content identity; graph/load/RNG/profile coverage is retained. No authored effect/cast index is persisted as a new authority. No new evaluator, permanent generation registry, compatibility shim, silent normalization or test-identity production dispatch was found in the reviewed R02 delta. The F01 source assertion must be corrected rather than used to redefine required behavior.

Excluded and unverified: server embedded-caller adaptation, Unity/deployed consumers, personal saves, non-Unix backend support, R03 general CLI/reload and R04–R24 gameplay. Unix/WSL input handling is directly covered; other platforms explicitly reject. Original socket test failure under sandbox and successful approved execution remain in owner history; it was neither skipped nor weakened here.

## Closure and next owner

Close this read-only GEX-20 as **Not Complete**, with one completion-blocking finding, R02-AUDIT-F01. Return to the exact R02 correction GEX-10 owner above. No P2G meaning change is necessary. Validation reuse and R01 currentness must be re-evaluated against the actual correction; the current successful broad evidence cannot certify changed diagnostic behavior without relevant revalidation. Preserve this verdict and target002 as historical evidence, then use a new stable target and fresh R02 auditor. Normal dependent completion is not authorized by this failed audit. No GEX-30 applies to this Single Goal.
