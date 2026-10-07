# Past Structural Audit Cross-Check

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Independent verdict lock: E-022 at `2026-08-22T07:34:00+09:00`
- Cross-check evidence: E-023

This document is a comparison, not an imported evidence base. Every fact retained in the current review was already found
independently or was rechecked against the current production baseline after the lock.

## Sources Selected After The Lock

| Past document | Bytes | SHA-256 | Selected use |
| --- | ---: | --- | --- |
| `DESIGN_REVIEW_DRAFT.md` | 16,704 | `aeb5a83e...fc013` | complete small draft; compare target/rewrite conclusion |
| `FINDINGS.md` | 29,897 | `01fc803c...e7da` | active findings 005~009 and root-cause log; route omitted items to current checks |
| `GAP_REGISTER.md` | 3,350 | `dcf4c8f0...c8ef` | continuation/performance/test gaps |
| `TRANSACTION_BOUNDARY_CATALOG.md` | 4,102 | `b1c15c77...497` | handler-family staging comparison |
| `STATE_AUTHORITY_MAP.md` | 5,465 | `a7075348...779b` | phase/authority/public surface comparison |
| `DATA_PUBLICATION_MAP.md` | 2,553 | `c49bc715...97fa` | validated publication versus builder boundary |
| `DETERMINISM_INVENTORY.md` | 2,893 | `fa5743c9...ea2c` | ordering/physics residual comparison |
| `COMPLETION_AUDIT.md` | 11,233 | `d8476064...ebff` | baseline, final catalog and exclusions |

Past `EXPERIMENTS.md` and other maps were not read exhaustively. The selected final maps contained the comparison facts;
current code was used for any newly considered issue. Past adjacent server evidence was not imported because this Goal excludes
server audit.

## Agreement And Current Mapping

| Past item | Current independent result | Cross-check treatment |
| --- | --- | --- |
| handler-selected transaction/preflight/projection ordering | ARCH-REVIEW-004/005, S03/S08, A02/A07 FAIL | confirms root cause; current scenario analysis is broader |
| phase plus parallel optional authority | ARCH-REVIEW-001, S08, A01/A02 | confirms application-state redesign pressure |
| domain/application/wire responsibility in `behavior.rs` | ARCH-REVIEW-001/003 and data/protocol map | confirms protocol boundary rewrite |
| live delta scans total append-only history | ARCH-REVIEW-002, E-020, A09 FAIL | confirms mechanism; current release numbers supersede incomparable old debug numbers |
| official data publication is strong | PA-009, E-019, A08 PARTIAL | confirms preservation asset |
| checkpoint/determinism/tests are strong | PA-010~012, E-018/E-021 | confirms migration oracle and argues against big bang |
| application/protocol staged rewrite, not whole-core rewrite | independent preservation/seam catalog | same direction; final A12 still uses five-option comparison rather than adopting draft verdict |

## Previously Reported Items Rechecked On Current Code

| Past concern | Current production recheck | Current disposition |
| --- | --- | --- |
| fixture/public data construction shares runtime type | public `GameDataBuilder::build/build_arc`, `GameDataBase::new` with `expect` paths, and `GameCore::new(Arc<GameDataBase>)` remain | added ARCH-REVIEW-006; official loader remains PA-009 |
| mutable/public `BattleCore` fields and event injection | `units`, battlefield, full database, event log/counters and `enqueue_event` remain public | folded into ARCH-REVIEW-003 boundary, not a separate rewrite reason |
| non-finite float can violate manually asserted battle event Eq/Ord premise | manual `impl Eq`, float-bearing public event payload and Debug final tie-break remain | folded into ARCH-REVIEW-003 validation/capsulation gate; no evidence of live nondeterminism |
| `StartNewGame` can mutate before fallible trigger activation | current control flow still assigns roots/pending mode before trigger activation and has no root transaction wrapper | folded into ARCH-REVIEW-005/006 |
| public test-only `set_enkephalin` | production-visible setter remains | low-severity instance of fixture/application encapsulation; no independent architecture finding |
| rejected shop overflow candidate | past audit disproved it | no current action and no imported rejected finding needed |

## Differences And Why

- The past audit was defect/criterion oriented and did not evaluate Rulebook preservation or S01~S10 future domain changes.
  This review therefore provides stronger evidence for retaining movement, skill, Boss Runtime, Monolith, map and data engines.
- The past draft proposed a similar shell rewrite but was explicitly DRAFT. The current decision will distinguish application
  transaction/state, protocol, battle session/archive and data construction boundaries by subsystem.
- Past timing used a different synthetic/debug linkage and reported much larger absolute costs. E-020 used the current workspace
  as an optimized release path dependency. Only shared O(history) complexity is compared; absolute numbers are not mixed.
- Past persistence conclusions included bounded adjacent server reads. This review certifies only that the Core port can support
  the selected host contract, not that current server crash/delivery behavior is correct.
- Current E-021 closes part of the old continuation gap with public next-command equality, while retaining a gap for future
  multi-domain/cross-platform breadth.

## Effect On Independent Verdict

A00~A11 verdict values remain unchanged. Cross-check added one S2 data-construction boundary finding and strengthened migration
gates, but revealed no missing evidence that makes a full rewrite necessary. A12 may now be evaluated.
