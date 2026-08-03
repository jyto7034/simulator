# Global Encounter Compendium Completion Review

Goal: G29

Review date: 2026-07-14

Guide: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

Immediate correction: none

## Item Audit

| Item | Integrated result | Verdict | Fit | Correction |
| --- | --- | --- | --- | --- |
| MSTR-04 permanent encounter collection | one strict monotonic abnormality/corrosion profile independent from battle history | Complete | High | None |
| PD-12 actual-spawn identity | opponent `UnitSpawned` canonical identities only; preview/authored unspawned waves excluded | Complete | High | None |
| PD-12 consuming commit | victory and final/fatal defeat only; retry and retreat excluded | Complete | High | None |
| PD-12 global rollback boundary | profile lies outside run/checkpoint/battle-entry state and merges by union only | Complete | High | None |
| External projection | current catalog metadata in one required root DTO, generic server pass-through, strict Unity reader | Complete | High | None |

## Expected Source Of Truth

The durable domain value is `EncounterCompendiumProfileState`: sorted sets of canonical abnormality
and corrosion profile IDs. The current combat result's actual event log is the only discovery input,
and the current strict catalogs are the only display-metadata authority. Run state, checkpoints,
previews, debug traces, game_server mappings, and Unity never maintain competing discovery state.

## Runtime Evidence

- `src/game/encounter_compendium.rs` filters opponent `UnitSpawned` events to Abnormality and
  CorrodedEmployee source identities, validates all IDs against live catalogs, rejects unknown
  serialized fields, and exposes union rather than replacement.
- `src/game/world/state.rs` owns the profile in `GameCoreState`. `RunState`,
  `RunCheckpointPayload`, and `BattleEntrySnapshot` contain no profile copy, so their restore paths
  cannot erase a later union.
- `src/game/world.rs` exposes strict profile construction/import and cloned export. Invalid imports
  fail before mutation.
- `src/game/world/combat.rs` derives and validates discoveries from the completed event log. It skips
  retryable defeat, while consuming defeat/fatal paths and planned victory carry the validated set
  and merge it only after the authoritative completion/RunFailed transition succeeds.
- `src/game/world/snapshot.rs` validates the profile again and resolves name/risk metadata from the
  current catalogs in canonical ID order.

## Data Evidence

- No live RON schema changed and no placeholder compendium content was invented.
- Existing strict abnormality and corroded-employee catalogs already own canonical IDs and names;
  abnormality data also owns risk level. The profile persists none of that metadata.
- The Rust fixture generator loads the actual embedded catalogs and serializes the public DTO.

## External Contract Evidence

- `RunSnapshotDto.encounter_compendium` is required and survives `map_selected_event`; there is no
  serde default or alias for the new root field.
- game_server's flattened typed root snapshot passes the DTO through without field remapping. Its
  load balancer keeps a monotonic player-keyed profile across actor recreation and constructs Core
  through the validating profile constructor.
- The current server has no durable player database. G29 therefore provides the strict serializable
  Core import/export boundary and process-lifetime actor-recreation preservation; inventing a new DB
  or save migration would exceed the accepted goal. A future durable adapter must persist this exact
  value and call the same validate-and-union import.
- Unity `EncounterCompendiumModel` reads only the required root object. It rejects missing/unknown
  fields, incomplete metadata, and duplicate IDs, and contains no preview/event-log discovery or
  empty legacy fallback.

## Test Evidence

- Core focused coverage proves source filtering/deduplication, unknown profile-field rejection,
  strict monotonic import, catalog-resolved projection, invalid-import no mutation, serialization,
  victory, preview-only exclusion, checkpoint/new-run preservation, retry, retreat, exhausted
  defeat, and fatal RunFailed retention.
- Server focused coverage proves actor recreation restores the profile; its existing root contract
  test proves generic pass-through.
- The generated Rust fixture matches Unity's tracked fixture exactly. Unity focused tests prove the
  valid catalog projection and fail-closed missing, duplicate, unknown, and legacy-root cases.
- Broad suites: Core 743, game_server 29, Unity EditMode 206; all passed with zero failures or
  ignored/skipped tests in these runs.

## Legacy And Fallback Audit

- Production/current contracts contain no gameplay battle record, trace linkage, abnormality-keyed
  diagnostic path, replacement profile load, alias/default, Unity local discovery calculation, or
  ignored G29 test.
- The sole `battle_records` production-tree search match is the intentional negative profile-schema
  test. Historical goal/audit documents remain evidence only.
- G16 developer traces still write from active result data and cannot read or mutate compendium
  state.

## Cross-Component Findings

- Permanent encounter knowledge and Endless run-local abnormality research have distinct owners and
  lifetimes; neither derives the other.
- Node consumption remains G15's authoritative transaction seam. G29 adds one validated, infallible
  monotonic merge after successful consumption without changing rewards, attempts, failure, or
  rollback semantics.
- Server transport and Unity presentation consume Core's typed projection instead of duplicating
  catalog lookup or discovery policy.

## Long-Term Direction Review

- Fit: High.
- Improvement class: none.
- Reason: one small profile value, one actual-spawn ledger, one consuming commit seam, and one
  catalog-resolved projection cover all layers. There is no dual schema, compatibility layer, or
  rollback copy to synchronize. The explicit serializable profile boundary is suitable for a future
  durable storage adapter without changing gameplay state ownership.

## Immediate Correction Candidates

None. The review-discovered Unity unknown-field gap was corrected and fully revalidated before this
verdict.

## Follow-Up Refactor Candidates

None inside G29. Process-durable player storage is a platform capability absent from the current
server, not a hidden compendium-specific fallback to add here.

## Policy Questions

None.

## Review Conclusion

G29 is Complete / High with no immediate correction. G21 may begin.
