# P06 Reference Integration Experiments

## 2026-08-17 — GEX-00 intake

- Exact selected member: `GS-CWBS-P2G50-002 / GOAL-CWBS-P06-REFERENCE-INTEGRATION`.
- Phase/node: `PHASE-CWBS-06-REFERENCE-INTEGRATION / WP-CWBS-11`.
- Dependencies: P01 `Pass 3`, P02..P05 `Pass 1`, all `Complete / High` and current.
- Initial full baseline is inherited: `game_core 1,080 + game_server 72 = 1,152 / 0 failed`.
- P06 owns the one final full rerun after all target changes and focused validation; no full command has been repeated
  since P01.
- Relevant data/code/docs are pre-existing dirty current actual. No reset, restore, stash, cleanup, external profile
  mutation or whole-tree format is authorized.
- Initial broad search found no current dedicated nonproduction conditional-Wave/summon placeholder ID. WhiteNight and
  Little Red remain production/current references, matching the reviewed delta.
- Production mutation: none in P06 at this point.

## 2026-08-17 — GEX-10 implementation and focused validation

- Added closed typed `PveEncounterClass::ReferenceFinalBoss`. Production selectors continue to request exact
  `Normal`/`Elite`/`NormalBoss`/`FinalBoss` classes; the reference class receives an explicit no-progression world
  branch. PVE validation rejects any reference outcome with reward UUIDs, node bonus or progression and rejects a
  reference encounter used as an abnormality reward source.
- Authored dedicated, visibly nonproduction live rows:
  - PVE `reference_cwbs_final_boss` with a time-zero boss Wave, an already-true actor-HP conditional required Wave,
    named `reference_victory`/`reference_defeat` outcomes and zero positive policy;
  - Boss Runtime `cwbs_reference_final_boss_runtime` with one atomic summon batch, battle-time wait, neutral cleanup
    signal, exact signal despawn and batch-clear path;
  - summoned definition `cwbs_reference_minion` plus separate reference abnormality/required dormant fragment catalog
    metadata. The fragment has no PVE or ordinary reward grant source.
- Added durable exclusion/data-drift validation covering map source registries, BossOmen chains, reward sources,
  all-live-FinalBoss chain membership, zero positive outcome authority and connected runtime/summon definitions.
- Added a live world vertical fixture through normal preview/setup/result consumption. It proves time-zero actor binding,
  conditional Wave first evaluation, summon provenance/counters, neutral signal removal, batch clear, named Victory,
  summon compendium isolation and final Victory-only registration of the abnormality/corrosion candidates.
- Updated only conflicting sections of `game_rulebook.md`, `core_runtime_contract.md` and
  `data_loading_contract.md`: named outcomes/no Draw, strict conditional Wave, summon lifecycle, disconnect auto-Pause/
  process rollback, SavePoint exclusion, single-set profile V3 and the excluded reference.
- Encountered and corrected two local implementation issues:
  - the first BossArena draft had no `defense_main` route and failed generated-preview validation; the reference now
    uses the existing Ambush/Medium route-bearing archetype without changing production content;
  - direct HP mutation did not constitute the prospective damage observation seam; the live fixture now authors the
    intended already-true `100%` first-binding condition and validates that supported path.
- Focused validation, current target:
  - `cargo test -p game_core cwbs_reference -- --nocapture`: `3 passed / 0 failed`;
  - `cargo test -p game_core live_publication_preserves_every_parsed_catalog_and_derived_item_registry -- --nocapture`:
    `1 / 0`;
  - `cargo test -p game_core live_pve_references_resolve -- --nocapture`: `1 / 0`;
  - `cargo test -p game_core embedded_pve_authors_exact_bootstrap_reward_sources -- --nocapture`: `1 / 0`;
  - `cargo test -p game_core all_live_final_boss_record_requires_every_current_chain_in_one_run -- --nocapture`:
    `1 / 0`;
  - Core disconnect, exact V4 checkpoint shape and explicit-load restore filters: `1 / 0` each;
  - `cargo test -p game_server detached_live_battle_has_no_ttl_and_reconnects_paused -- --nocapture`: `1 / 0`;
  - `cargo check -p game_core -p game_server`: pass, zero warnings;
  - scoped `rustfmt --check`, `git diff --check` and canonical contradiction search: pass.
- P06 final full Core/server commands have not yet run. Relevant-input hash freeze and exact post-run comparison are the
  remaining GEX-10 validation actions.

## 2026-08-17 — first final-Core attempt and focused correction

- The first frozen-target `cargo test -p game_core` attempt executed all `1,080` library tests and stopped before the
  integration binaries with `1,078 passed / 2 failed / 0 ignored`. `game_server` was deliberately not run after the
  failed Core gate. The failures were:
  - `live_deployment_reconciles_defeated_player_unit_during_authoritative_advance`;
  - `track_and_rejoin_raw_live_bundle_reaches_typed_world_setup_identity`.
- Both were latent fixture-integration assumptions exposed by the approved P04/P06 behavior, not production-policy
  reversals:
  - the deployment reconciliation fixture killed its only player unit, so the now-authoritative all-player-defeated
    named terminal correctly closed the battle before its state assertion; it now deploys a second living guard and
    proves only the defeated unit moves to redeploying state;
  - the TrackAndRejoin raw-source fixture replaces the Boss catalog and strips ordinary live Boss bindings, but had
    accidentally retained the separately excluded CWBS reference encounter and its actor/summon dependencies; it now
    explicitly omits that unrelated nonproduction encounter and supplies an empty summoned catalog.
- Correction-target focused reruns:
  - `cargo test -p game_core live_deployment_reconciles_defeated_player_unit_during_authoritative_advance -- --nocapture`:
    `1 passed / 0 failed`;
  - `cargo test -p game_core track_and_rejoin_raw_live_bundle_reaches_typed_world_setup_identity -- --nocapture`:
    `1 passed / 0 failed`;
  - `cargo test -p game_core cwbs_reference -- --nocapture`: `3 passed / 0 failed`.
- Correction-target static validation:
  - scoped `rustfmt --check`: pass;
  - scoped `git diff --check`: pass;
  - `cargo check -p game_core`: pass, zero warnings;
  - `cargo check -p game_server`: pass, zero warnings.
- Because test inputs changed, the earlier hash freeze is invalid. The corrected relevant inputs must be frozen again
  before the correction-target full Core rerun; only a successful frozen-target run can become completion evidence.

## 2026-08-17 — correction-target full-run hang and test-harness correction

- A correction-target Core rerun reached the `1,080`-test library binary but did not terminate. Two additional attempts
  overlapped after the first command's output channel appeared closed without returning an exit status. Process
  inspection identified all three exact command trees; the two non-evidence duplicates were terminated, preserving only
  the run carrying an external exit-status receipt.
- The preserved run left exactly three small command-runtime tests consuming CPU indefinitely after their 60-second
  warnings. It was terminated with exit `143` after the test threads had each consumed more than 18 CPU-minutes:
  - `runtime_max_health_modifier_decision_table_preserves_hp_death_and_provenance`;
  - `sourced_lethal_max_health_modifier_runs_cleanup_and_nested_death_heals_once`;
  - `fatal_hit_releases_owner_attributed_retaliation_before_the_defender_dies`.
- A 60-second isolated run reproduced the first hang with exit `124`. Temporary event markers localized it to the
  test-only `drain_event_queue`: after a sourced lethal effect in an empty scenario with no approved named outcome, the
  helper consumed the self-rescheduling `ContinuousMovementTick` forever. The markers were removed.
- This is a test-harness assumption, not a production-policy defect. Empty command-unit scenarios intentionally have no
  invented terminal outcome under the named-outcome contract. The helper now drains finite command/ability causality
  while discarding the recurring movement clock, leaving production runtime unchanged.
- Corrected focused results, each under a 60-second guard:
  - max-health decision table: `1 passed / 0 failed` in `0.02s`;
  - sourced lethal cleanup: `1 passed / 0 failed` in `0.01s`;
  - fatal retaliation: `1 passed / 0 failed` in `0.01s`.
- The overlapping and terminated runs are diagnostic evidence only and are not counted as a successful baseline. The
  test-file change invalidates the preceding freeze again; the final target must be statically checked, re-frozen and
  exercised by one clean Core command tree.

## 2026-08-17 — clean Core correction run and five stale fixture contracts

- After the event-drain fix, one clean frozen-target Core command terminated normally with
  `1,075 passed / 5 failed / 0 ignored` out of `1,080` library tests. Integration binaries did not run after the failed
  library gate. The five failures and narrow corrections were:
  - `battle_event_cmp_equal_iff_eq_for_every_live_variant`: the exhaustive ordering fixture still counted 21 variants
    but omitted the already-live `BossSummonWake` pair; added the missing unequal same-variant pair;
  - `invalid_boss_compiler_input_rejects_the_whole_candidate` and
    `live_candidate_rejects_boss_binding_that_cannot_build_runtime_setup`: replacement Boss catalogs accidentally kept
    the unrelated excluded CWBS reference encounter/summoned behavior, so cross-source errors preempted each intended
    diagnostic; both fixtures now omit the reference encounter and use an empty summoned catalog;
  - `bundle_validation_returns_typed_pve_cross_contract_errors`: source/object/field remained exact, while the validator
    now reports the tighter range message `terminal time_ms must be within 1..=4294967295`; the assertion now locks that
    current contract;
  - `surrounded_battlefield_remains_default_defense_and_does_not_end_before_deployment`: the fixture expected the
    removed implicit `UnitDefeated` condition despite authoring `fixture_terminal_outcomes(None)`; it now asserts the
    authored `AllRequiredEnemyGroupsDefeated` named Victory condition and preserves the no-premature-end check.
- Focused reruns: all five corrected tests pass `1 / 0`; the accidentally touched summoned-unit diagnostic was also
  rerun and passes `1 / 0`.
- These are exhaustive-test and fixture-isolation corrections only. No runtime/data/canonical policy was changed. The
  added test files invalidate the preceding freeze; static checks and a new 22-input freeze are required before the next
  clean full run.

## 2026-08-17 — full unit pass and production-index exclusion correction

- The next clean run passed the complete library binary: `1,080 passed / 0 failed / 0 ignored`. It then passed
  `live_item_skill_activation` `4 / 0`, but `live_skill_catalog_audit` stopped the package at `3 passed / 1 failed`:
  `live_content_index_lists_every_abnormality_derived_fragment` compared 31 abnormality-origin fragments with the 30
  production entries in `docs/skills/live_content_index.md`.
- The additional row is the explicitly nonproduction `fragment_cwbs_reference_dormant`. The production index must not
  list or count reference-only content. The audit now derives reference abnormality identities from
  `PveEncounter::is_reference_only()` and excludes only fragments originating from those identities; no ID-prefix or
  documentation-count waiver is used.
- Focused rerun: the corrected production-index audit passes `1 / 0`. Runtime/data/canonical content is unchanged.
  `tests/live_skill_catalog_audit.rs` joins the relevant target, so static checks and a 23-input re-freeze precede the
  next full run.

## 2026-08-17 — RON production-audit exclusion corrections

- The next run again passed the full library `1,080 / 0`, item activation `4 / 0` and skill catalog audit `4 / 0`.
  `ron_loading` then reported `22 passed / 3 failed` because three production-only audits still consumed reference data:
  - the exact abnormality roster included the reference boss;
  - the Little Red test treated every raw/compiled Boss definition and every opted-in PVE encounter as production;
  - the fixed reward audit required a non-empty production reward list from the zero-positive-policy reference.
- All three now select production data by `PveEncounter::is_reference_only()`:
  - the roster derives and subtracts reference primary-abnormality identities;
  - Little Red assertions find its named raw definitions and filter PVE opt-ins to non-reference encounters, without
    asserting that validation-only definitions do not exist;
  - fixed reward/effect coverage skips reference encounters, whose zero-positive policy is locked by dedicated P06
    validation.
- Focused reruns: all three corrected `ron_loading` tests pass `1 / 0`. Runtime/data/canonical content is unchanged;
  `ron_loading.rs` was already in the relevant target, but its new hash requires another freeze before package rerun.

## 2026-08-17 — finite projectile integration fixture

- The next run passed library `1,080 / 0`, item activation `4 / 0`, skill catalog `4 / 0` and RON loading `25 / 0`.
  In `skill_runtime_contract`, 13 tests completed while
  `untargeted_projectile_still_hits_later_unit_after_cast_target_dies` consumed a full CPU thread indefinitely; the
  command was terminated with exit `143` after more than eight CPU-minutes.
- The test had already observed its required launch, original-target death and later-unit collision, but its generic
  helper continued simulating to an ordinary named terminal. This particular post-observation fixture can reach a
  stalemate, so completion of the whole battle is not part of its contract.
- Added a test-only helper that appends an explicit low-priority `fixture_cutoff` TimeReached Victory and used it only for
  this projectile characterization. It does not alter production runtime or replace ordinary encounter outcomes.
- Focused rerun under a 60-second guard: `1 passed / 0 failed` in `0.04s`. The new integration test-file hash requires a
  24-input freeze before the next package run.

## 2026-08-17 — successful Core package and server recovery assertion correction

- Frozen-target `cargo test -p game_core` completed successfully in `96.90s`:
  - library: `1,080 passed / 0 failed / 0 ignored`;
  - `live_item_skill_activation`: `4 / 0`;
  - `live_skill_catalog_audit`: `4 / 0`;
  - `ron_loading`: `25 / 0`;
  - `skill_runtime_contract`: `14 / 0`;
  - `skill_test_suite`: `4 / 0`;
  - doc tests: `0`;
  - total executed: `1,131 passed / 0 failed / 0 ignored`.
- The first subsequent server full run executed 73 library tests and reported `72 passed / 1 failed / 0 ignored`.
  `attaching_to_active_battle_preserves_revision_and_returns_recovery_batch` still expected revision `1`, although
  replacing a live session now invokes the approved disconnect auto-Pause and publishes revision `2`.
- Renamed the test to describe replacement-session auto-Pause and asserted revision `2`, paused playback, full recovery
  delta and session replacement. Focused rerun: `1 passed / 0 failed`; server runtime is unchanged.
- The Core target remains frozen and its successful baseline is reusable because only a server test changed. Add the
  server test file to the target, re-freeze all 25 relevant inputs, and rerun only the affected server full command.

## 2026-08-17 — final frozen validation target

- Final `cargo test -p game_server`: library `73 passed / 0 failed / 0 ignored`; main/doc tests `0`; exit `0`, elapsed
  `7.46s` including wrapper/build time.
- Final integrated evidence at `ATR-CWBS-P06-FINAL-001`:
  - Core total `1,131 passed / 0 failed / 0 ignored`;
  - server total `73 passed / 0 failed / 0 ignored`;
  - combined non-zero execution `1,204 passed / 0 failed / 0 ignored`;
  - scoped `rustfmt --check`, `git diff --check`, `cargo check -p game_core` and
    `cargo check -p game_server`: pass, zero warnings;
  - exact pre/post SHA-256 comparison across 25 relevant inputs: `MATCH`.
- No production/test/data/canonical member changed after the final freeze. GEX-10 mutation is closed; fresh read-only
  GEX-20 starts from the exact manifest in `PLAN.md`.

## 2026-08-17 — GEX-20 completion

- Fresh read-only audit `AUDIT-CWBS-P06-GEX20-001` rechecked every completion condition against current code/data/docs,
  the scoped dirty target and non-zero final validation.
- `ATR-CWBS-P06-FINAL-001` remained exact-match current at audit close.
- Verdict: `Complete / High`; no correction return. All six Phase Goals now satisfy the Master GEX-30 prerequisite.
