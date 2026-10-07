# R02 execution evidence

Current correction003 final runtime input is VI027. Final core004 has 1,217 passes; generated003 has 16. Current release build007/AB005/source-diagnostic004 and alltargets003/releasecheck003 are actual terminal results. API batch002 and prior observer measurements have bounded exact reuse in CORRECTION_REUSE_003. Earlier sections preserve their historical input/result boundaries. R02 original audit is Not Complete; successor fresh audits remain required.

## GEX-00 pass 001 — completed

2026-09-10 KST. Exact child `(GS-CAPG-P2G50-001, GOAL-CAPG-01)`, R02 / PHASE-CAPG-01 / HOF-CAPG-P2G60-001. Current upstream location hashes, reviewed source hashes, R01 291 target members and two transitive fixtures matched. New child history search returned no matches; directory was absent before intake. Rust/Cargo 1.91.1. Workspace disk had 88 GiB available and /tmp 821 GiB at read-only intake. No actual resource blocker observed.

P2G standard Plan receipt generation and verification both terminal 0/passed; Goal quality-contract 2 terminal 0/passed. Separate read-only Goal semantic review found no findings. These are document validation, not new game tests. Prior game evidence is linked in BASELINE_001, not recounted as R02 execution.

Scoped originals were copied to /tmp/capg-r02-baseline and each copy rehashed. Existing porcelain status was captured separately. GEX-00 changed only these execution records. No production mutation; no game tests run. Next: separate GEX-10 detailed design, implementation and meaningful focused validation.

## GEX-10 pass 001 — owned source first delta

`cargo test -p game_core --lib dynamic_source_path_survives_input_drop_in_compiled_effect_and_error` terminal 0: 1 passed, 0 failed/ignored, 1,114 filtered. Compilation 23.48s, test 0.06s. [Result](evidence/owned_source_001.json) / [log](evidence/owned_source_001.log) / [input](VALIDATION_INPUT_001.json). Source path survives scope drop in compiled effect and unsupported-opcode diagnostic with object/action/effect/field identities. This is a bounded positive/negative ownership seam, not full external loader/GEX completion.

## GEX-10 pass 001 — formatted module regression

After scoped rustfmt, `cargo test -p game_core --lib game::data::live_ron` terminal 0: 81 passed, 0 failed/ignored, 1,034 filtered, 8.77s. [Result](evidence/live_ron_001.json) / [log](evidence/live_ron_001.log). `cargo test -p game_core --lib game::data::skill_data` terminal 0: 14 passed, 0 failed/ignored, 1,101 filtered, 0.03s, including the owned path lifetime test. [Result](evidence/skill_data_001.json) / [log](evidence/skill_data_001.log). Both bind [formatted inputs](VALIDATION_INPUT_002.json). These are 95 distinct module tests; the earlier single ownership test is a subset and is not added to that count. Source did not change between these executions and input capture. Full external loader, generation and storage still require implementation and direct validation.

## GEX-10 pass 001 — typed raw assembly / canonical identity

`cargo test -p game_core --lib game::data::content_pack` terminal 0: 8 passed, 0 failed/ignored, 1,115 filtered; compile 44.78s, tests 0.15s. [Result](evidence/raw_canonical_001.json) / [log](evidence/raw_canonical_001.log) / [inputs](VALIDATION_INPUT_003.json). All 24 official typed catalogs were parsed, reassembled and domain/reference validated. Separate skill/corroded definition files consume later preset files only at final assembly; duplicate UUID reaches a fallible error, and filtered/appended winning skill effects preserve their original file/index. Canonical serializer tests distinguish ordered effect arrays and Option/enum identity while ignoring map insertion order/source metadata, and reject nested nonfinite values. No external filesystem/token/save success is inferred. Two currently-unused resolver key-scope warnings remain; the next implementation consumes them.

## GEX-10 pass 001 — explicit writer resolver

`cargo test -p game_core --lib game::data::content_pack` terminal 0: 16 passed, 0 failed/ignored, 1,115 filtered; compile 37.44s, tests 0.15s. [Result](evidence/resolver_001.json) / [log](evidence/resolver_001.log) / [inputs](VALIDATION_INPUT_004.json). Eight new resolver probes cover sibling Remove/Replace explicit heads, tombstone succession, explicit diamond merge and remaining competition, dependency order vs Add collision, structural invalid losers, absent/duplicate dependency and cycle/edge budget, payload/operation/winner bijection, numeric array order vs replacement source, namespace exactness and recipe reference keys. Earlier eight raw/canonical probes also passed. The previously unused key-scope warnings disappeared through actual resolver use. External safe acquisition, published generation, app/save integration and full Goal validation remain open.

## GEX-10 pass 001 — snapshot first attempt / environment failure

`cargo test -p game_core --lib game::data::content_pack` terminal 101: 26 passed, 1 failed, 0 ignored, 1,115 filtered, 0.15s. [Result](evidence/snapshot_001.json) / [log](evidence/snapshot_001.log) / [inputs](VALIDATION_INPUT_005.json). The failed Unix socket case received EPERM at isolated `UnixListener::bind` before passing that socket to the loader; other path/byte/depth/change checks passed. Preserve this failure. The identical suite is rerunning with approved sandbox escalation as snapshot_002; no case skipped or weakened. Rustix was already cached at 1.0.7; Cargo.lock changed only the new game_core dependency row against GEX00.

## GEX-10 pass 001 — snapshot approved retry

The same `cargo test -p game_core --lib game::data::content_pack`, with approved require_escalated execution, reached terminal 0: 27 passed, 0 failed/ignored, 1,115 filtered, 0.18s. [Result](evidence/snapshot_002.json) / [log](evidence/snapshot_002.log), unchanged [input](VALIDATION_INPUT_005.json) hashes rechecked. Includes the socket fixture denied by sandbox in 001, with no skip or weakening. Eleven new bounded acquisition/lexical tests plus the sixteen prior canonical/raw/resolver tests are now passing; this is still not full publication/app/save or release completion.

## GEX-10 pass 001 — public external generation publication

`cargo test -p game_core --lib game::data::content_pack` (approved escalation for the Unix socket fixture) terminal 0: 33 passed, 0 failed/ignored, 1,115 filtered; compile 51.58s, tests 9.44s. [Result](evidence/publication_001.json) / [log](evidence/publication_001.log) / [input](VALIDATION_INPUT_006.json), unchanged hashes rechecked. Six new tests stage actual isolated external files and use public `content_pack::load`: same binary loads A/B with same IDs/different price and different identity; comment/file-enumeration changes preserve identity but change receipts; final winning skill is lowered while a discarded dangling reference is ignored; unsupported opcode in a losing file still rejects with object/action/effect/source; GameCore retains the generation until its final reference drops; fixture identities track actual data independently of Arc addresses. Existing 27 tests pass. Current GameCore/prepared commit retains the token; candidate install/latest saves and official packager/release gameplay execution are not claimed by these tests. Two obsolete world imports are identified for the next scoped edit.

## GEX-10 pass 001 — application generation commit / namespace consumers

`cargo test -p game_core --lib game::data::content_pack` (approved cargo test execution) terminal 0: 36 passed, 0 failed/ignored, 1,115 filtered; compile 28.84s, tests 9.83s. [Result](evidence/application_generation_001.json) / [log](evidence/application_generation_001.log) / [input](VALIDATION_INPUT_007.json), unchanged hashes rechecked. New public application probes retain A during its blocking briefing while B is independently published, reject a B Start even with replace_active_run=true, complete A's normal briefing/starter flow before the existing eligible Abandon, then commit B before its different script ID/briefing and starter flow. Wrong command and post-gate Endless tutorial preparation error preserve A identity and five public state/profile projections. A separate real boss/summon/signal compile probe accepts namespaced top-level keys while a namespaced local state is rejected. Existing 33 tests pass; obsolete world imports were removed, no warnings in this run. V6 storage, official packaging, actual release gameplay A/B, full required validation and separate audits remain unfinished.

## GEX-10 pass 001 — V6 content-bound checkpoint

`cargo test -p game_core --lib game::world::tests::checkpoint_validation` terminal 0: 17 passed, 0 failed/ignored, 1,136 filtered; compile 39.91s, tests 7.45s. [Result](evidence/checkpoint_v6_001.json) / [log](evidence/checkpoint_v6_001.log) / [input](VALIDATION_INPUT_008.json), unchanged hashes rechecked. Legacy semantic corpus hash remains exactly pinned after removing only the new identity field from its test projection. Existing 19 semantic graph mutations execute with valid V6 checksums, alongside exact fields, strict rejects, load/seed/inventory boundaries and V6 recapture. New direct tests reject all five contract components and the fingerprint with recomputed valid checksum, reject V4/V5 repeatedly without mutation, restore actual UUID streams and replay an actual Gate, and exercise external A/B save mismatch, equivalent A reacquisition, cached export, explicit load, and generation/profile/peak rollback. The tool wrapper initially tried storing an undefined exit code after starting the command; root recovered the live session and captured its actual terminal result without rerunning. Broader dependent checkpoint consumers and full current core/release remain required.

## GEX-10 pass 001 — dependent checkpoint first pass

`cargo test -p game_core --lib checkpoint` terminal 101: 43 passed, 1 failed, 0 ignored, 1,109 filtered; compile 1.18s, tests 7.83s. [Result](evidence/checkpoint_dependents_001.json) / [log](evidence/checkpoint_dependents_001.log), unchanged [input](VALIDATION_INPUT_008.json). The failure is the facility roundtrip test's remaining version-5 assertion; actual writer is 6. Its substantive monotonic load/profile/seed checks remain and will run after the approved version expectation is updated. Other dependent checkpoint/facility/Monolith/shop/recruitment/battle decision tests passed. Failure retained, no weakened validator or fallback added.

## GEX-10 pass 001 — dependent checkpoint corrected version expectation

`cargo test -p game_core --lib checkpoint` terminal 0: 44 passed, 0 failed/ignored, 1,109 filtered; compile 20.25s, tests 7.80s. [Result](evidence/checkpoint_dependents_002.json) / [log](evidence/checkpoint_dependents_002.log) / [input](VALIDATION_INPUT_009.json), unchanged hashes rechecked. Only the obsolete facility version expectation and combat assertion description changed since the first dependent run. All substantive monotonic profile/load/seed/VN/Gate/Monolith/shop/recruitment/battle-decision assertions remain. This includes the 17 prior checkpoint_validation tests; they are not additional distinct coverage. Packaging, release execution, remaining Goal IVAs and integrated audits are still open.

## GEX-10 pass 001 — official external source packaging

`cargo test -p game_core --lib game::data::content_pack` (approved execution for retained Unix socket fixture) terminal 0: 39 passed, 0 failed/ignored, 1,117 filtered; compile 35.70s, tests 12.73s. [Result](evidence/packaging_001.json) / [log](evidence/packaging_001.log) / [input](VALIDATION_INPUT_010.json), unchanged hashes rechecked. Three new packaging tests read actual isolated copies of the official 25 source files rather than embedded bytes, retain comments/line endings and existing custom bytes, reject existing empty/occupied/symlink/source-tree output, and reject absent/stale/invalid source before output. Public loader revalidates new output against both expected semantic identity and exact byte receipts. All 36 previous module tests pass. The registered minimal default-release binaries have not yet been built/executed; that evidence is next.

## GEX-10 pass 001 — first default-release binary build

`cargo build -p game_core --release --no-default-features --bin package_official_content --bin consume_content_pack` terminal 0, 48.75s. [Result](evidence/release_build_001.json) / [log](evidence/release_build_001.log), unchanged [input](VALIDATION_INPUT_010.json). Both actual binaries build without enabling fixture features. Eight dead-code warnings identify fixture-only source structs/parsers and obsolete private wrappers still compiled into release. Apply their existing fixture cfg consistently, then rebuild before recording the current A/B execution. This first build is not gameplay evidence.

## GEX-10 pass 001 — release fixture boundary refinement

Same default-release binary build terminal 0, 41.53s. [Result](evidence/release_build_002.json) / [log](evidence/release_build_002.log) / [input](VALIDATION_INPUT_011.json), unchanged hashes rechecked. Eight unused fixture implementation warnings are resolved; one fixture-only `BattlefieldTemplateDefinition` import now also requires the same cfg. No runtime behavior changed; current A/B execution follows the final warning-free build.

## GEX-10 pass 001 — warning-free default-release binaries

Same default-release binary build terminal 0, 46.13s, no warnings. [Result](evidence/release_build_003.json) / [log](evidence/release_build_003.log) / [input](VALIDATION_INPUT_012.json), unchanged hashes rechecked and both executable SHA-256 values recorded. Production source structs/parsers and private wrappers now follow the same explicit fixture capability boundary. Actual A/B execution is a separate next command.

## GEX-10 pass 001 — release A/B first execution / probe argument defect

`python3 core_enhanced/tools/content_pack_release_probe.py --repository . --output-dir /tmp/capg-r02-release-ab-001` terminal 1. [Result](evidence/release_ab_001.json) / [log](evidence/release_ab_001.log), unchanged [input](VALIDATION_INPUT_012.json). Actual release packager succeeded and preserved source/custom bytes. First consumer command correctly rejected probe-authored lowercase `standard`; the existing public GameMode enum requires `Standard`/`Endless`. Root will serialize typed GameMode::Standard in this minimal consumer, retaining the actual application command schema. This is not A/B success; full failure output and successful packaging output are copied into durable evidence, and the isolated first-run directory remains intact.

## GEX-10 pass 001 — typed release command correction build

Same default-release binary build terminal 0, 2.70s, no warnings. [Result](evidence/release_build_004.json) / [log](evidence/release_build_004.log) / [input](VALIDATION_INPUT_013.json). The minimal consumer now serializes the existing typed GameMode; application schema is unchanged. Exact rebuilt executable hashes are recorded before a new isolated A/B run.

## GEX-10 pass 001 — actual default-release A/B/A execution

`python3 core_enhanced/tools/content_pack_release_probe.py --repository . --output-dir /tmp/capg-r02-release-ab-002` terminal 0. [Result](evidence/release_ab_002.json) / [log](evidence/release_ab_002.log) / [actual receipt](evidence/release_ab_002/receipt.json) / [input](VALIDATION_INPUT_013.json), unchanged hashes rechecked. Ten real subprocesses used the exact two executable hashes from release_build_004: four successes and six expected nonzero rejections. Official packaging preserved all 25 source files, 24 output catalog bytes and the preexisting isolated B custom pack. The same consumer binary ran A/B/A through actual briefing/starter commands and produced Run resource currency **500/733/500**; A repeat is complete-value equal, A/B identities differ and B's actual policy path is in its receipt. Existing output, missing/invalid B catalog, old B contract, unsupported B capability and missing source reject explicitly. Source/custom/official byte hashes and both binary hashes match before/after. All command stdout/stderr and receipt files were copied to durable evidence; fixture directories remain isolated in /tmp. This closes the bounded release startup/resource probe, not B battle pin or the remaining integration/fresh audit obligations.

## GEX-10 pass 001 — additional consumer first compile

`cargo test -p game_core --lib game::world::tests::content_generation` terminal 101 at compilation; zero tests executed. [Result](evidence/integration_consumers_001.json) / [log](evidence/integration_consumers_001.log) / [input](VALIDATION_INPUT_014.json), unchanged hashes rechecked. New test helper attempted String.into for VisualNovelSceneId; the existing owner requires its explicit new constructor. Correct that call without changing production type/schema.

## GEX-10 pass 001 — actual Remove and battle/save generation consumers

`cargo test -p game_core --lib game::world::tests::content_generation` terminal 0: 3 passed, 0 failed/ignored, 1,159 filtered; compile 32.53s, tests 9.61s. [Result](evidence/integration_consumers_002.json) / [log](evidence/integration_consumers_002.log) / [input](VALIDATION_INPUT_015.json), unchanged hashes rechecked. A real battle remains on resource 20 and the A database while external B publishes; eligible generation-aware B start completes briefing/starter/SavePoint, writes B identity, and enters battle on resource 31 and the B database Arc. A snapshot/export remain exact. Deliberate fixture-only missing RunStart triggers the real late preparation rollback with complete A/profile/seed/token retry preservation, without asserting that invalid external data can publish. The Remove test populates A lookups, loads left::same/right::same with a cross-namespace recipe, rejects an uncleared pool reference, then verifies removed final catalog/ID/UUID/item registry membership, preserved right recipe and actual RewardState→ClaimReward acquisition of only the retained equipment. Three new external reference/preset/generated setup module tests still await execution.

## GEX-10 pass 001 — external integration first pass / diagnostic oracle

`cargo test -p game_core --lib game::data::content_pack` terminal 101: 41 passed, 1 failed, 0 ignored, 1,120 filtered; compile 1.42s, tests 12.72s. [Result](evidence/integration_pack_001.json) / [log](evidence/integration_pack_001.log) / [input](VALIDATION_INPUT_015.json), unchanged hashes rechecked. New cross-pack skill/corroded preset and late generated setup failure probes pass. The reward-source negative test incorrectly expected its substituted missing fragment ID in the cause; the existing owner rejects earlier at `reward_sources[0].fragment_id` because it must match the primary abnormality's source fragment. Correct that specific diagnostic oracle; source/object/field, retry, no-publication and A-preservation assertions remain. Production validation is unchanged.

## GEX-10 pass 001 — external integration completed focused pass

`cargo test -p game_core --lib game::data::content_pack` terminal 0: 42 passed, 0 failed/ignored, 1,120 filtered; compile 21.70s, tests 13.10s. [Result](evidence/integration_pack_002.json) / [log](evidence/integration_pack_002.log) / [input](VALIDATION_INPUT_016.json), unchanged hashes rechecked. The corrected reward-source oracle now checks the actual required primary-abnormality relationship; no production validator changed. All three new actual external cross-pack/reference/generated setup probes and all 39 prior tests pass. Standalone API consumer/privacy checks, stable full core/release, resource observations and separate predecessor/current audits remain required.

## GEX-10 pass 001 — standalone positive validated-data consumer

`cargo run --release --offline --manifest-path core_enhanced/tools/validated_data_api_audit/valid/Cargo.toml --target-dir target/capg-api -- /tmp/capg-r02-release-ab-002/live_data official` terminal 0, compile `1m 02s`. [Result](evidence/api_validated_001.json) / [log](evidence/api_validated_001.log) / [actual resolved input](VALIDATION_INPUT_017.json), unchanged hashes rechecked. The standalone default-release consumer now actually loads the explicit external pack and constructs CoreApplication. Its separate Cargo lock gained the rustix dependency graph and four Cargo-resolved target-specific version changes; the root lock is unchanged. This is recorded rather than treated as the prior input. Remaining positive/negative API probes and minimal lock preservation assessment are open.

## GEX-10 pass 001 — minimal standalone lock preserved

`cargo check --release --offline --locked --manifest-path core_enhanced/tools/validated_data_api_audit/valid/Cargo.toml --target-dir target/capg-api` terminal 0, 30.08s. [Result](evidence/api_lock_001.json) / [log](evidence/api_lock_001.log) / [input](VALIDATION_INPUT_018.json), unchanged hashes rechecked. The original standalone lock was reconstructed from its untouched sibling by substituting the probe package name and matched the pre-build SHA captured in 016. All four unrelated original futures/rustversion records were restored. The strict locked offline check now passes with only the required new rustix dependency graph; root lock unchanged. Remaining standalone probes should receive this minimal graph delta before locked execution, preserving their original package records.

## GEX-10 pass 001 — independent default-release public API batch

`python3 core_enhanced/tools/public_api_external_probe.py --repository . --pack-root /tmp/capg-r02-release-ab-002/live_data --pack official --output-dir /tmp/capg-r02-api-batch-001` terminal 0. [Result](evidence/api_batch_001.json) / [receipt](evidence/api_batch_001/receipt.json) / [input](VALIDATION_INPUT_020.json), all 273 input hashes unchanged. Five standalone positive consumers ran with the actual external official pack; six negative consumers failed at their original forbidden Rust API diagnostics, including both default-release fixture API names. No missing-loader-only or Cargo/lock error substitutes for privacy coverage. All child logs/commands/receipt preserved. Eleven separate tool locks received only the required rustix graph and game_core dependency row; [delta](evidence/TOOL_DEPENDENCY_DELTA_001.json) preserves exact original/current hashes and existing package versions. This tool scope was outside the original 1,850-file GEX00 baseline and is explicitly supplemental. Full core/release integration and separate fresh audits remain open.

## GEX-10 pass 001 — inherited public performance guard compile failure

Original P08 `public_guards` release/offline/locked command reached terminal 101, zero workload samples: its manual BattleEventLogEntry fixture predates R01 and omits required skill_effect_source (E0063). [Result](evidence/public_guards_001.json) / [log](evidence/public_guards_001.log) / [input](VALIDATION_INPUT_021.json), all 279 hashes unchanged. The source was unmodified for this attempt; no current performance result exists. Required fixture shape adaptation will set None for this synthetic source-free History event, preserving the observer algorithms, thresholds, workload sizes and assertions. This is a core test tool fixture, not a P2G/GEX pipeline validator.

## GEX-10 pass 001 — first integrated whole-core baseline

`cargo test -p game_core --lib --tests` (approved socket-fixture execution) terminal 0: **1,214 passed**, zero failures/ignored, comprising 1,162 lib + 4 item activation + 4 catalog audit + 25 RON loading + 15 runtime contract + 4 skill suite. [Result](evidence/full_core_001.json) / [log](evidence/full_core_001.log) / [input](VALIDATION_INPUT_020.json), all 273 hashes unchanged. Compile 50.55s, lib 76.63s; remaining durations in result. One captured lib output chunk was truncated by the tool; all final target counts/exit are retained, and no test was skipped. A read-only coverage review now identified additional direct assertions for true missing registration target, populated compendium through generation transitions, and pack-count/overflow boundaries. Those corrections precede the final stable baseline; this result is preserved without claiming the not-yet-added assertions.

## GEX-10 pass 001 — stable integrated core and V6 public guards

Final `cargo test -p game_core --lib --tests -- --quiet` (approved socket execution) terminal 0: **1,214 passed**, zero failed/ignored, no warnings. [Result](evidence/full_core_002.json) / [complete concise log](evidence/full_core_002.log) / [input](VALIDATION_INPUT_022.json), all 279 hashes unchanged. Includes the bounded review additions for missing registration encounter, populated live+dormant compendium through B commit/repeated late rollback, exact/over pack count and usize overflow. The test-only wrapper cfg now matches its actual callers.

Original P08 public guard was rebuilt release/offline/locked with only skill_effect_source=None added to the synthetic BattleEnd fixture; byte comparison against the preserved original proves every observer algorithm, cap and assertion unchanged. [Build](evidence/public_guards_build_002.json), then direct binary [execution](evidence/public_guards_002.json), both terminal 0, same input. The [complete receipt](evidence/public_guards_002_receipt.json) retains all 101 samples for snapshot/export and existing history/staging guards. Console JSON was truncated; the separately written full receipt is authoritative. Current V6 export and snapshot meet the unchanged engineering time/size caps; this explicit test-fixtures workload does not substitute for default-release external pack execution. Allocation observations and final all-target/release checks follow.

## GEX-10 pass 001 — all-target compile and allocation observation

`cargo check -p game_core --all-targets` terminal 0, 30.68s, no warnings: [result](evidence/all_targets_001.json) / [log](evidence/all_targets_001.log). Original public guard with allocation-observer feature terminal 0, compile 3.75s: [result](evidence/public_guards_allocation_001.json) / [full receipt](evidence/public_guards_allocation_001_receipt.json). Both match [input 022](VALIDATION_INPUT_022.json). Five allocation samples at each 128/8192 history size retain exact rollback and identical 677 allocations, 0 reallocations, 84,553 peak bytes, 0 retained bytes. This is 2 allocations/144 peak bytes above historical pre-identity staging, not hidden as an unchanged absolute measurement; the original required independence comparison passes. Instrumented timing is not latency evidence. Plain current guard separately measured Run snapshot p95 241.039 µs/48,493 bytes and V6 export p95 8.770 µs/18,154 bytes, under unchanged caps. Official source/resource observations are in [RESOURCE_OBSERVATION_001.json](evidence/RESOURCE_OBSERVATION_001.json); they do not invent an RSS threshold.

## GEX-10 pass 001 — final default-release binaries and repeated external consumers

`cargo check -p game_core --release --lib` terminal 0, 22.16s, no warnings ([result](evidence/release_check_001.json)). Final default-release packager/consumer build terminal 0, 44.81s, no warnings ([result](evidence/release_build_005.json)). Rebuilt executable bytes differ from build004, so the actual external A/B/A batch was rerun on those exact binaries: [result](evidence/release_ab_003.json) / [receipt](evidence/release_ab_003/receipt.json), terminal 0, ten commands (four successes/six required rejections), actual currency 500/733/500 with exact A-repeat and byte preservation. All match [input022](VALIDATION_INPUT_022.json).

The independent API batch has separate [currentness proof](evidence/API_REUSE_001.json): only three test-only files and a wrapper cfg change differ from input020. Reconstructing that one prior cfg line matches the old world.rs SHA; the wrapper is absent from both default-release configurations. All original API mains/adapter/locks and all other compiled source remain exact. Thus the 11 actual API results are reused with proven non-impacting source delta; no rerun is fabricated.

## GEX-10 pass 002 — post-predecessor comment correction

Separate R01 successor GEX-20 finished Complete / High on its ATR002, all 568 direct/106 external/2 binary members current at closure; it independently reconstructed preservation and ran 3 additional external release authoring probes. [Audit](../../../content_authoring_minimal_effect_contract/execution/single_goal/COMPLETION_AUDIT_002.md). Its informational stale fixture rustdoc was then corrected in a distinct R02 GEX-10 pass. [Correction](CORRECTION_002.md) / [currentness and reuse proof](evidence/COMMENT_REUSE_002.json) / [VI023](VALIDATION_INPUT_023.json). Only two rustdoc lines were replaced with the real production loader; all executable/test/config/data input is otherwise exact. No new gameplay test was run for that documentation edit; the actual 1,214/core/release/API/observer evidence is reused under direct non-impact proof. R02 audit002 target is the next immutable source/evidence set, not a prewritten verdict.


## GEX-10 pass003 — audit F01 correction, failures and final validation

The first separate R02 audit closed Not Complete on ATR002; [its original verdict](COMPLETION_AUDIT.md)
and independent release reproduction remain immutable. [CORRECTION_003](CORRECTION_003.md) owns the repair.
All eight Rust deltas are compared against exact VI023 pre-correction copies in
[final impact/diff](evidence/CORRECTION_IMPACT_003_FINAL.json). No official RON, dependency, app/save/effect
owner, policy/Plan/Goal/Handoff or forbidden external consumer was changed in this correction.

| Actual execution | Input / result / interpretation |
| --- | --- |
| [generated001](evidence/correction_generated_001.json) | VI024; 14 passed/1 failed. New test caught `[13].templates[13]` instead of the raw top-level template array address. Failure preserved, source address/oracle corrected. |
| [generated002](evidence/correction_generated_002.json) | VI025; 15 passed. Custom policy ordinary/Monolith zone/route and custom encounter failures identify source fields, related contexts, retained A and repeat/restore behavior. |
| [core003](evidence/full_core_003.json), [alltargets002](evidence/all_targets_002.json), [release006](evidence/release_build_006.json), [releasecheck002](evidence/release_check_002.json) | VI025; 1,216 core passes and actual successful compiles. Intermediate evidence before the analogous custom BossPattern case; not the final core result. |
| [release source003](evidence/correction_release_diagnostic_003/receipt.json), [AB004](evidence/release_ab_004.json) | VI025; actual 4 source-reproduction and 10 AB subprocesses pass. Valid B output equals the previous frozen binary's complete output byte-for-byte ([proof](evidence/CORRECTION_VALID_OUTPUT_CONTINUITY_003.json)). |
| [API002](evidence/api_batch_002.json) | VI025; 5 actual successful default-release API consumers and 6 expected forbidden Rust compiler failures, offline/locked. Final VI027 reuse is explicitly proven below. |
| [Boss diagnostic001](evidence/correction_boss_diagnostic_001.json) | VI026; 0 passed/1 failed. Test-only separate BossPattern Replace reaches real late missing summon zone rejection but the custom file is omitted. This confirms bounded source-review finding before production repair. |
| [generated003](evidence/correction_generated_003.json) | VI027; 16 passed, failed/ignored 0. Selected compiled behavior/pattern contributors now retained; custom summon zone and route negatives repeat identically, preserve A/input bytes and restore exact B identity/receipt. |
| [final core004](evidence/full_core_004.json) | VI027; 1,217 passed = 1,165 lib + 4/4/25/15/4 integration, failed/ignored/warnings 0. Compile 20.80s; lib 63.17s. Includes unchanged core runtime/source/save/profile regressions and all three new custom-source tests. Existing socket fixture ran with approved cargo-test escalation. |
| [alltargets003](evidence/all_targets_003.json), [releasecheck003](evidence/release_check_003.json), [releasebuild007](evidence/release_build_007.json) | VI027; actual all-target check, optimized lib check and both explicit no-default-features release binaries; terminal 0 and no warnings. |
| [final AB005](evidence/release_ab_005.json) | VI027/current binaries; 10 commands, 4 successes/6 expected rejects; actual 500/733/500, repeat A equality, preserved source/custom bytes. |
| [final source004](evidence/correction_release_diagnostic_004.json) | VI027/current consumer; 10 commands, 4 successes/6 expected rejects. Policy exact original wave field and separate BossPattern/behavior contributor errors, deterministic repeats, no success output on invalid input and exact input/valid-output restoration. |

[CORRECTION_REUSE_003](evidence/CORRECTION_REUSE_003.json) reconstructs all eight VI025 source files
from original VI023 copies plus the preserved intermediate patch and verifies their hashes. VI025→027 changes
only the diagnostic error-construction implementation and one appended test; 277 other inputs are exact.
Successful external API cases never enter that error helper; public API/privacy/feature/adapter inputs are unchanged.
Thus API002 is historical actual execution with proven current applicability, not a fabricated second run.
The unchanged snapshot/export/rejected-staging measured paths similarly retain the original public guard and
allocation measurements; loading/start/combat preparation is outside those intervals. No new loader-cost or
observer timing/allocator measurement is claimed. Changed rejection behavior is directly rerun above.

Final source, canonical docs and evidence are frozen in successor target003; the intended next work is separate
R01 fresh currentness review followed by another independent R02 GEX-20. Audit-only outputs are excluded from
implementation manifests and must be read/hash-bound by the consuming auditor when they actually exist.
No R01 restoration or R02 positive verdict is asserted by this implementation report.
