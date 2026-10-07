# R02 implementation evidence map

Exact child `(GS-CAPG-P2G50-001, GOAL-CAPG-01)`, PHASE-CAPG-01 / AG-CAPG-01 → WP-CAPG-03. This is the current implementation owner's evidence map, not a completion verdict. Original audit on ATR002 is Not Complete/F01; correction003 supplies exact custom generated source addresses and related contexts. Policy/Goal meanings are unchanged. R01 predecessor source/publication currentness and R02 completion both require successor fresh reviews.

## Evidence identities and consumer boundary

- [Final full core](evidence/full_core_004.json): 1,217 passed (1,165 lib + 52 integration), zero failed/ignored, no warnings; [input 027](VALIDATION_INPUT_027.json) records 279 current source/test/data/tool/config hashes. All added pre-freeze assertions execute here.
- [All targets](evidence/all_targets_003.json), [release library](evidence/release_check_003.json), and final default-release binary/build/execution records in [EXPERIMENTS](EXPERIMENTS.md) cover real production configurations. Public external loader/packager/consumer binaries explicitly appear in Cargo's autobins=false inventory.
- [Final release A/B/A](evidence/release_ab_005.json): ten actual subprocesses, currency 500/733/500, repeat A complete-value equality, unchanged source/custom/output bytes and executable identities. This measures actual startup through briefing and starters; the separate world test below measures B battle and saves.
- [Independent API consumers](evidence/api_batch_002.json), with [currentness proof](evidence/CORRECTION_REUSE_003.json): five real default-release runtime consumers with explicit external pack plus six original forbidden-API compile negatives; all run offline/locked. A missing loader alone or Cargo failure cannot satisfy the negative oracle.
- [Plain public guard](evidence/public_guards_002.json) and [allocation guard](evidence/public_guards_allocation_001.json): original P08 algorithms/caps/assertions; only the R01-required source-free event fixture field is added. Current V6 snapshot/export, bounded history and staging observations are recorded separately from historical event-heavy measurements.
- [Official input observations](evidence/RESOURCE_OBSERVATION_001.json): 24 catalogs 408,797 bytes; source with bundle 408,824; packaged input with manifest 410,472; largest catalog 114,717. Time measurements include process startup and real consumers, not just parsing. No invented RSS/product SLA.

All workspace core data/application/battle/save/projection consumers assigned by the Goal are required and accessible. Server embedded-call migration, Unity/deployed consumers, network reconnect, personal stores and R03's general CLI/reload are explicitly outside this Goal; their success is not inferred. Test fixture capability is explicit and never substitutes for actual default-release external loading.

## Direct test locators

Repository-relative prefixes below: `P=core_enhanced/src/game/data/content_pack/`, `W=core_enhanced/src/game/world/tests/`, `T=core_enhanced/tests/`. Each named test is present in the current full-core target; focused intermediate results remain historical with exact inputs in EXPERIMENTS.

| Key | Actual consumer/oracle |
| --- | --- |
| PACK | `P/tests.rs::same_binary_external_generations_retain_exact_content_and_sources_after_files_change`, `comments_locations_and_inventory_enumeration_do_not_change_semantic_identity`; actual external bytes, retained A and source receipt vs semantic identity |
| RAW | `P/raw/tests.rs::all_24_typed_catalogs_reassemble_and_validate_with_original_sources`, `duplicate_uuid_reaches_fallible_final_validation_without_constructor_panic`, `winning_effect_keeps_original_definition_index_after_filtering_and_append`; `P/tests.rs::external_skill_and_corroded_definitions_resolve_presets_in_another_pack`, `winning_raw_reference_only_is_lowered_and_published_from_actual_external_packs` |
| RESOLVE | Eight `P/resolver/tests.rs` tests: sibling Remove/Replace explicit head, tombstone/diamond succession, dependency is not override, invalid losers, missing/duplicate/cycle/edge budget, payload/operation/winner bijection, numeric execution order vs own source, independent namespaces/reference-key recipe |
| REMOVE | `W/content_generation.rs::valid_external_remove_updates_namespaced_lookup_registry_recipe_and_reward_acquisition`: real namespaced packs with distinct UUIDs and cross-namespace recipe; populate A ID/UUID/item registry; dangling pool rejects; valid Remove clears final B catalogs/lookups/registry; only retained right equipment appears in actual RewardState → ClaimReward inventory. A indexes stay intact, reversed selection has equal identity |
| REJECT | `P/tests.rs::unsupported_unused_external_opcode_cannot_publish_even_in_a_losing_writer`; `external_final_buff_reward_and_registration_references_reject_without_publication` checks missing buff, reward, bad registration fragment relationship and a truly missing registered encounter; twice-identical diagnostics, original file/object/field, token absent, A unchanged, restored valid input has A identity. `external_late_generated_setup_failure_cannot_publish_or_replace_a` exercises generated combat setup failure after compile |
| GENERATED SOURCE | `P/tests.rs::custom_policy_generated_failures_keep_winning_wave_fields_and_contributors_on_retry`, `custom_encounter_generated_failure_keeps_original_field_and_policy_template_context`, `custom_boss_pattern_generated_setup_failure_keeps_used_pattern_and_behavior_sources`: separate full replacement files, actual primary wave field/original template array and consumed boss behavior/pattern contributors; repeat invalid/no-publication, A/input preservation and restored valid generation. [Current release source probe](evidence/correction_release_diagnostic_004.json) directly covers the audit fixture and analogous BossPattern setup failures. |
| PIN | `P/tests.rs::active_briefing_pins_a_and_next_eligible_start_commits_b_before_briefing_and_starters`, `rejected_command_and_post_gate_start_preparation_preserve_a_identity_and_public_state`; actual blocking/gating and typed public candidate API. `W/content_generation.rs::external_b_start_pins_actual_battle_resource_and_export_while_a_remains_pinned`: A battle resource20/ArcA persists while B publishes; eligible B briefing/starter/SavePoint/battle resource31/ArcB and B export identity; full A snapshot/export unchanged; populated live+dormant compendium and profile floor survive |
| ROLLBACK | `W/content_generation.rs::fixture_b_late_briefing_preparation_failure_preserves_complete_a_authority_on_retry`: deliberate fixture-only invalid B missing RunStart, real late draft preparation failure twice; complete A state/profile/seed/token and nonempty compendium unchanged, no output/player/VN commit. External invalid B cannot publish and is separately rejected by REJECT; invalid fixture publication is not claimed |
| SAVE | `W/checkpoint_validation.rs` 17 tests: strict V6 envelope and required 20 semantic fields, unchanged historical semantic corpus hash after removing only identity from the test projection, 19 one-edge graph defects with refreshed valid checksums, every content-contract component/fingerprint mismatch, required identity fields, V4/V5/unknown/checksum reject on retry, inventory/fragment/load-count/seed boundaries, exact UUID streams and actual next Gate replay, fresh public CoreApplication port, external A/B continue/explicit load/cached export/reacquired equivalent A and token rollback. Broader checkpoint/facility/Monolith/decision regressions also run in full core |
| PATH | `P/snapshot/tests.rs`: strict relative path grammar, no-follow root/directory/leaf, case prefix/hardlink/inventory collisions, UTF-8/nonregular/socket, detected bytes/inode race, exact/over file/per-file/aggregate/pack/depth bounds, host usize overflow and malformed RON lexical inputs. Socket requires approved execution; the original sandbox EPERM and successful retry are retained |
| PACKAGE | Three `P/packaging.rs::tests` cases read actual isolated official source, preserve 24 catalog byte strings/comments/line endings/custom sentinel; exclusive output rejects source/subtree/existing empty/occupied/symlink; old/missing/invalid input fails before output. Real release packager uses the same public loader and compares semantic identity and exact receipts |
| LIFETIME | `P/tests.rs::generations_are_reclaimed_after_the_last_token_and_core_reference`: weak-reference observation after token/core drops; no permanent generation registry. `fixture_identity_tracks_actual_content_without_arc_or_source_addresses` verifies actual fixture identity |
| R01 | `T/skill_runtime_contract.rs::authored_literal_ron_executes_recomposed_definitions_with_exact_effect_sources`; live item/catalog and skill suites; `ability/authoring/tests.rs`, `data/skill_data.rs`, `data/live_ron.rs`, proc/withdraw/projectile/area/cancel and checkpoint regressions execute in final full core. Detailed successor impact record and a separate fresh predecessor audit are required; old R01 verdict is historical |

## Fourteen acceptance members

All locators use `PER-CAPG-E0N-REVIEWED-001[index]`, exactly as the frozen Goal.

| Member | Direct evidence / required observation |
| --- | --- |
| E01[0] | Actual release A/B/A + PACK/PIN; selected external definitions change startup and actual battle resource |
| E01[1] | REJECT/PATH/PACKAGE + six release negative inputs + default fixture API compile rejection; unsupported/incomplete external input is an error |
| E02[0] | RESOLVE/REMOVE/RAW; Add/full Replace/Remove reach final catalogs, lookups, registration and real acquisition |
| E02[1] | RESOLVE/PACK; explicit winner/chain, invalid dependencies/cycles, shuffled host inventory identity, actual ordered payload slots |
| E02[2] | REJECT/RAW/REMOVE; final dangling refs reject, discarded loser-only ref does not invalidate valid winner |
| E03[0] | PIN/SAVE; A active run remains pinned; B briefing/starter/battle/export binds B |
| E03[1] | REJECT/ROLLBACK/PIN; validation and late preparation failures reject and preserve A/profile/compendium/seed/token/state, no success fallback |
| E04[0] | SAVE; fresh application, equivalent reacquired external generation, actual UUID stream and next Gate result |
| E04[1] | SAVE/PACK; same IDs changed content and each contract component reject before mutation even with valid recomputed checksum; input bytes remain outside mutation ports |
| E04[2] | SAVE/default release source boundary; strict latest V6 only, no legacy conversion or guessed fingerprint; personal files untouched |
| E05[0] | PACKAGE and release before/after receipts; separate exclusive output only, existing empty/custom paths rejected/preserved |
| E05[1] | PIN/ROLLBACK/SAVE plus existing compendium/profile tests; populated live+dormant ID set and applicable profile progress survive success/failure/load/rollback |
| E06[0] | REJECT/PATH/RAW; unsupported unused opcode/capability/schema, path/race/missing/limits reject with actual source and no token |
| E06[1] | PACK/RESOLVE/LIFETIME/PIN; semantic identity, order, retained A after byte edits and actual reclamation |

## Eight IVA and eight completion conditions

| IVA / condition | Evidence owned by WP-CAPG-03 |
| --- | --- |
| IVA01 / C01 | Real release packager → exact external A/B/A binary execution; PACK/PACKAGE/RAW |
| IVA02 / C02 | RESOLVE/REMOVE/PACK; namespace/reference keys, explicit winner/order, final registry/acquisition |
| IVA03 / C03 | RAW/REJECT/REMOVE; cross-pack presets, losing references, actual missing buff/reward/registration and late generated setup |
| IVA04 / C04 | PIN/ROLLBACK/SAVE; A/B briefing/starter/battle/export, eligible commit and failed/forbidden no-mutation |
| IVA05 / C05 | SAVE; matching latest fresh application/RNG, mismatch/old/strict/load-count reject |
| IVA06 / C07 | PATH/REJECT/PACK/LIFETIME; bounded actual input and semantic/current generation identity |
| IVA07 / C06/C08 | Whole core 1,214, all-target/release/library/binaries, independent public API probes, actual official bytes/progress preservation |
| IVA08 / C08 | Final full-core R01 consumers and explicit successor delta map; **R01 successor audit002 Complete / High**, with [post-comment currentness proof](evidence/COMMENT_REUSE_002.json); separate R02 fresh audit still required. Implementation review is not either audit |

## Ten required dimensions

| Dimension | Actual evidence |
| --- | --- |
| success | Release A/B/A, RAW/REMOVE/PIN/SAVE |
| invalid_input | REJECT/RESOLVE/PATH/SAVE; source error and absent token/no mutation |
| failure_rollback | Late generated setup and deliberate late briefing failures, valid-checksum mismatches, cached/export/load rejection, populated profile/compendium/token snapshots |
| retry_idempotency | Repeated rejected loads/starts/restores and restored valid input; semantic repeat A; existing eligibility/load-count remains |
| ordering_concurrency | Explicit writer/head rather than enumeration priority, numeric definition slots, unordered canonical maps, detected filesystem change and A active while B publishes; no atomic filesystem snapshot claim |
| boundary_empty_max | Exact/over files/bytes/packs/depth/dependency edges, usize overflow, empty/missing/duplicate IDs/selection, UUID/local/reference key and inventory/load-count limits |
| disconnect_restart_persistence | Actual fresh CoreApplication, V6 continue/explicit/cache with equivalent content, exact UUID/Gate result; network reconnect is excluded |
| live_data | Real official source packaged into external directories, actual release A/B and invalid external files |
| producer_consumer | All 24 typed adapters → final lowering/compile/generated setup → opaque token → actual briefing/starter/battle/save/index/acquisition/projection/API consumers |
| legacy_fallback_absence | Production external loader only, strict schema/current contracts, V4/V5 reader removal, unsupported unused op rejection, no automatic conversion/data setter/dual fingerprint; original supported executor and explicit fixture capability retained |

## Structural and migration review

SC01–05/I01–06: one typed raw/resolution path precedes lowering; stable full-definition operations distinguish availability and writer ancestry; canonical hash uses typed winners and actual execution order, not receipts/caches; one opaque generation owns database+identity+receipt; draft commit includes token/state/seed/output after existing gate and before briefing; all latest checkpoint consumers share retained-identity validation. Existing executor, source/cast origin, profile semantics and command gates are reused. No evaluator/plugin/OS/network execution from RON, permanent generation store, old-reader alias or future-placeholder success is added.

R02 latest-only storage is the explicitly approved successor to R01's historical V4/V5 preservation boundary. The old reader-removal certification scaffold was removed under this authority, not falsely flipped to certified. The historical semantic fixture hash and substantive graph/load/RNG/profile coverage remain; required identity is additional and all old forms now have direct no-mutation rejection. No authored effect/cast index is added to durable state. Old R01 audit/evidence and upstream P2G are immutable.

Known evidence limits: original GEX00 captured 1,850 scoped source/doc/config paths, excluding `core_enhanced/tools`. Tool locks have separately captured original bytes/hashes and minimal dependency deltas; eight startup-adapted API main files were directly read before editing but lack pre-edit content hashes. They are supplemental current consumers, not falsely included in the 1,850-file preservation proof. The original observer has an exact preserved before image and one-line fixture delta. The first full-core verbose output chunk was truncated; the final concise run has complete counts/log and zero skip. Earlier failures and superseded inputs remain visible.

Post-audit routing: [R01 actual audit002](../../../content_authoring_minimal_effect_contract/execution/single_goal/COMPLETION_AUDIT_002.md) preserves old audit001 and reports three newly executed external authoring probes. [R02 correction002](CORRECTION_002.md) changes one fixture rustdoc paragraph only; [VI023/currentness proof](evidence/COMMENT_REUSE_002.json) supports reuse of all actual validation above. The R02 auditor decides the final currentness and verdict.


## Correction003 audit disposition and currentness

R02-AUDIT-F01 is addressed by the GENERATED SOURCE row and current generated003/core004/release source004.
Its prior failed test oracle and the newly reproduced analogous BossPattern failure remain in EXPERIMENTS;
passing old rejection-only tests are not used to erase them. All fourteen acceptance, eight IVA and ten dimensions
above keep their original meanings. C03/SC01/IVA03 now have direct source-fidelity evidence; invalid_input and
producer_consumer observations include separate custom file addresses. Other execution semantics and mandatory
coverage are revalidated in the final 1,217-test target and current release tests. API/observer reuse is bounded
by exact CORRECTION_REUSE_003. [R01 impact003](R01_SUCCESSOR_IMPACT_003.md) identifies the needed predecessor
review; actual new R01 audit and distinct R02 fresh GEX-20 still own the final currentness/completion decisions.
