# Core Enhanced Refactoring Audit — Cross-Boundary Audit

Audit Date: 2026-08-07
Status: complete
Input: 53 confirmed candidates from Phase 1~10
Output: 51 final candidates after two proven merges

## Exact Reconciliation

| Phase | Source candidates | Final rows | Reason |
| --- | ---: | ---: | --- |
| P01 | 3 | 3 | no duplicate |
| P02 | 4 | 3 + F-C01 share | C02 merges with P03-C01 |
| P03 | 5 | 3 + F-C01/F-C02 shares | C01 merges with P02-C02; C05 merges with P04-C02 |
| P04 | 4 | 3 + F-C02 share | C02 merges with P03-C05 |
| P05 | 5 | 5 | no duplicate |
| P06 | 7 | 7 | no duplicate; staging primitive coordinated with P07 |
| P07 | 4 | 4 | no duplicate |
| P08 | 6 | 6 | no duplicate |
| P09 | 8 | 8 | no duplicate |
| P10 | 7 | 7 | no duplicate |
| Total | 53 | 51 | every source ID consumed exactly once |

## Proven Root-Cause Merges

### CRA-F-C01 — Validate The Complete Checkpoint Invariant Graph Before Restore

Sources: `CRA-P02-C02`, `CRA-P03-C01`.

Both candidates target the same current-version checkpoint admission boundary, fail for the same structural reason
(cross-field invariants are distributed among domain owners but never composed before mutation), and require the
same typed pre-restore validation graph. The final candidate keeps distinct predicates: run/UUID seed and map
progression consistency from P02; employee/equipment/loadout/fragment/economy consistency from P03. It does not merge
active state with the persisted document or choose content rebind/migration policy.

### CRA-F-C02 — Use The Common World Transaction For Headquarters Completion

Sources: `CRA-P03-C05`, `CRA-P04-C02`.

Both point to the same `with_headquarters_transaction` handwritten rollback list, same common transaction target and
same recruit/emergency/open-shop callers. P04 adds the hidden `take()` bypass detail; P03 adds full ledger/UUID/
checkpoint field evidence. One final candidate retains both and removes one duplicate source candidate.

## Reviewed Shared Seams Kept Separate

| Candidates | Shared seam | Why they remain separate |
| --- | --- | --- |
| P01-C01 / P01-C03 | live publication boilerplate | source-file inventory and parsed-catalog conversion have different authorities, failure evidence and removal scopes |
| P01-C02 / P05-C02 | startup publication | P01 owns finite difficulty/seed enumeration; P05 consumes it to validate full scenario feasibility |
| F-C01 / P02-C03 | checkpoint | validation rejects invalid documents; capture/restore conversion preserves valid V3 field parity |
| P04-C03 / F-C02 | world transaction | paired Event/Shop session commits and HQ branch rollback have distinct persisted/active lifetimes |
| P05-C01 / P05-C03 | preview cache | C01 corrects serialized internal-scale authority; C03 consolidates valid resolution/conversion after correction |
| P06-C01 / P06-C03 | battle execution | terminal state ownership precedes, but is not the same as, world advance transaction atomicity |
| P06-C03 / P07-C02 | preflight/staged commit | production tick/finalization and individual client tactical commands have different inputs, results and transaction lifetimes; share primitive/design only |
| P07-C01 / P08-C05 | tactical/damage state | tactical-effect reconciliation and committed hit resolution own different mutations despite shared modifiers |
| P08-C01 / P08-C04 | trigger occurrence | dispatcher executes published effects; publication validator authorizes references/combinations |
| P08-C05 / P09-C05 | projectile hit | committed damage owns effect/HP/death; motion kernel owns collision/time facts |
| P09-C04 / P09-C06 | range projection | typed reach query is behavior-preserving prerequisite; effect-cell meaning still needs projection policy |
| P10-C02 / P10-C03 | wire amplification | Core result classification/payload and server battle envelope are independent serialization authorities |
| P10-C01 / P10-C04 | command/capability | command descriptor owns route metadata; typed host overlay owns persistence-derived snapshot merge |

## Competing Authority Resolution

| Boundary | Final authority | Removed/forbidden competitor |
| --- | --- | --- |
| live source publication | embedded source set plus validated candidate publication | manual count/prose and private mapping copies are derived only |
| active-run restore | Core context-aware checkpoint invariant graph; host envelope owns checksum/high-water only | debug-only seed assertion, partial domain restore checks and host inference of Core payload |
| persistent assets | roster/inventory/fragment ledgers and staged grant aggregate | snapshot/authored records as player-owned state |
| interactive sessions | domain Event/Shop/HQ/VN lifecycle with common transaction primitives | one generic session hierarchy |
| combat preview/setup | manifested difficulty/live policy and regenerated internal setup input | serde-lossy presentation cache as stat authority |
| battle execution | execution state owns terminal outcome and event total order; world stages publish | caller finish signal or post-commit fallible projection |
| tactical state | final `(activity, stance, tile, profile)` reconciliation | three paths independently applying Focus/block/Rest deltas |
| effects/death | typed occurrence dispatcher, HP-aware stat transition and production death path | disconnected DeathHandler and duplicate hit executors |
| targeting/delivery | typed reach/delivery queries and lifecycle-specific adapters | ghost range, anchor-only approximation and duplicate motion geometry |
| command/result/snapshot | Core typed descriptor/result/DTO/error code; server transport/persistence/overlay only | server field reauthoring and JSON mutation as implicit gameplay authority |

## Work-Type Cross-Audit

- Correctness candidates retain `correctness_correction`, including test-only P09-C08. None is recast as cleanup or
  refactor because the change restores a rejected, atomic or deterministic contract.
- Refactors retain exact current behavior and require characterization/golden parity first.
- `code_cleanup`, `cleanup` and `behavior_preserving_cleanup` normalize to `cleanup`; public/serialized removal keeps
  its external compatibility gate.
- `RunComplete` shape, active-run migration/lease, admin durability, max-health kill credit, OnHit semantics,
  multi-step usefulness/effect preview and obstacle-aware targeting remain policy or compatibility work, not
  confirmed refactors.

## Dependency Graph

1. Correct test evidence first: P09-C08 and the candidate-specific negative/atomicity tables.
2. Publication prerequisites: P01-C02 → P05-C02; P04-C01 and P08-C04 reference-only validation can proceed beside it.
3. Persistence: F-C01 → P02-C03/P03-C03; P02-C01 is an independent host correction.
4. Preview/setup: P05-C01 → P05-C03; P05-C02 → P05-C04.
5. Battle lifecycle: P06-C01 → P06-C03; P06-C02 is independent after semantic key review.
6. Tactical mutation: coordinate P06-C03/P07-C02 staging primitive; P07-C01 independently starts with failing
   full-final-state tests.
7. Effects: P08-C01 → P08-C05/C06; P08-C02 characterization and policy precede HP/death consolidation.
8. Targeting: P09-C01/P09-C02 and P08-C05 boundary → P09-C05; P09-C06 precedes P09-C03/C04.
9. Transport: P10-C01 can proceed after its descriptor table; P10-C02 precedes P10-C03; P10-C04/C05 are independent.
10. Public/package cleanups follow external inventory and assertion transfer; P10-C07 alone is mechanically ready.

## Signal Preservation

Every source catalog's rejected/deferred/unverified table remains incorporated by reference and unchanged. Final
normalization groups them as follows:

- `rejected`: false generic merges, silent fallbacks/repair, DTO/snapshot promotion, cache deletion, balance/content
  invention and physical-file-size arguments;
- `deferred_policy`: save migration/lease, admin durability, VN retry/source semantics, content expansion, modifier
  kill credit, OnHit/skill trigger meaning, obstacle/multi-step/preview semantics and `RunComplete` migration;
- `unverified_external`: public Rust APIs, serialized fields, fixture generators, Unity, standalone packaging/config
  and debug/tool consumers;
- `retained_authority`: explicit recovery/tombstone, persistent/active session dual roles, preview-fixed waves,
  battle log/seq, movement segment, effects/formulas and distinct target/delivery modes.

No source signal is promoted to a confirmed candidate merely because another Phase found a similar shape.

## Cross-Boundary Verdict

Two merges are proven; all other shared seams are dependency or primitive-sharing relationships. The final target
authorities do not compete. Source candidate reconciliation is exact at 53→51 with High confidence.
