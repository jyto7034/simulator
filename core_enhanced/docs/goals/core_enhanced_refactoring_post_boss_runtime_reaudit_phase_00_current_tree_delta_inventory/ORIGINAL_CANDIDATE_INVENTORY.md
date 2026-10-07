# Phase 00 Original Candidate Inventory

Historical baseline: 2026-08-07 final refactoring audit
Source candidates: 53
Final candidates: 51
Current disposition authority: P01~P10 and Final of this reaudit, not this inventory

## Historical Sources

- Final identity/tier/readiness:
  `../core_enhanced_refactoring_audit_final_catalog_consolidation/FINAL_CANDIDATE_CATALOG.md`
- Merge/dedupe evidence:
  `../core_enhanced_refactoring_audit_final_catalog_consolidation/CROSS_BOUNDARY_AUDIT.md`
- Test families:
  `../core_enhanced_refactoring_audit_final_catalog_consolidation/TEST_CLEANUP_CATALOG.md`
- Full Problem Evidence, authority, invariant, removal scope, axes and safety net for unchanged rows:
  `../core_enhanced_refactoring_audit_phase_01_live_data_loading_and_validation/CANDIDATE_CATALOG.md` through
  `../core_enhanced_refactoring_audit_phase_10_command_projection_and_server_seam/CANDIDATE_CATALOG.md`
- Boss relation claim:
  `../boss_runtime_v1_implementation_master_phase_07_completion_audit/COMPLETION_AUDIT.md`

These are historical evidence. Current code/data/test/consumer evidence must confirm or change every disposition.

## Exact 53 To 51 Reconciliation

| Source Phase | Source IDs | Source count | Current final identity treatment |
| --- | --- | ---: | --- |
| P01 | C01~C03 | 3 | 3 unchanged final rows |
| P02 | C01~C04 | 4 | C02 contributes to F-C01; other 3 unchanged |
| P03 | C01~C05 | 5 | C01 contributes to F-C01, C05 to F-C02; other 3 unchanged |
| P04 | C01~C04 | 4 | C02 contributes to F-C02; other 3 unchanged |
| P05 | C01~C05 | 5 | 5 unchanged final rows |
| P06 | C01~C07 | 7 | 7 unchanged final rows |
| P07 | C01~C04 | 4 | 4 unchanged final rows |
| P08 | C01~C06 | 6 | 6 unchanged final rows |
| P09 | C01~C08 | 8 | 8 unchanged final rows |
| P10 | C01~C07 | 7 | 7 unchanged final rows |
| Total | 53 source IDs | 53 | 49 unchanged + 2 merged final rows = 51 |

The two final merged identities consume four source IDs exactly once:

- `CRA-F-C01` ← `CRA-P02-C02` + `CRA-P03-C01`
- `CRA-F-C02` ← `CRA-P03-C05` + `CRA-P04-C02`

## Exact Final 51 And Reaudit Owner

`Historical Boss relation` records only the past Boss audit claim. `none (impact review)` means one of the other 32
rows and still requires a current-tree impact review. Full original invariants remain in the linked detailed Phase
catalog; this table freezes identity, work type, readiness and review ownership without copying prose selectively.

| # | Final ID | Historical candidate | Original work type | Original tier/readiness | Historical Boss relation | Reaudit owner |
| ---: | --- | --- | --- | --- | --- | --- |
| 1 | CRA-F-C01 | complete checkpoint invariant graph | correctness_correction | A / test first | none (impact review) | P02 + P03 contribution; Final identity |
| 2 | CRA-F-C02 | common HQ transaction | behavior_preserving_refactor | B / full-state characterization | none (impact review) | P03 + P04 contribution; Final identity |
| 3 | CRA-P01-C01 | embedded source inventory authority | behavior_preserving_refactor | B / exact-set characterization | coimplemented | P01 deep review |
| 4 | CRA-P01-C02 | every manifested difficulty at publication | correctness_correction | A / negative matrix test | prerequisite completed | P01 deep review; P05 setup contribution |
| 5 | CRA-P01-C03 | private parsed→published mapping | behavior_preserving_refactor | C / private scope frozen | none (impact review) | P01 impact review |
| 6 | CRA-P02-C01 | host monotonic metadata on every publish | correctness_correction | A / real-file lower-input tests | none (impact review) | P02 impact review |
| 7 | CRA-P02-C03 | RunState↔V3 checkpoint conversion | behavior_preserving_refactor | B / golden field parity after F-C01 | none (impact review) | P02 impact review |
| 8 | CRA-P02-C04 | MapProgression mutation/availability | behavior_preserving_refactor | B / transition characterization | none (impact review) | P02 impact review |
| 9 | CRA-P03-C02 | atomic enhancement material aggregation | correctness_correction | A / duplicate-row tests | none (impact review) | P03 impact review |
| 10 | CRA-P03-C03 | bidirectional equipment/employee loadout | behavior_preserving_refactor | B / after F-C01; transition table | none (impact review) | P03 impact review |
| 11 | CRA-P03-C04 | five-ledger grant transaction state | behavior_preserving_refactor | B / caller parity | none (impact review) | P03 impact review |
| 12 | CRA-P04-C01 | VN repeat/source publication matrix | correctness_correction | A / structured negative tests | none (impact review) | P04 impact review |
| 13 | CRA-P04-C03 | Event/Shop persisted-active commits | behavior_preserving_refactor | B / paired-state characterization | none (impact review) | P04 impact review |
| 14 | CRA-P04-C04 | VN assembly and explicit queue placement | behavior_preserving_refactor | C / world characterization | none (impact review) | P04 impact review |
| 15 | CRA-P05-C01 | keep serialized preview cache out of setup authority | correctness_correction | A / exact durable-Continue test | none (impact review) | P05 impact review |
| 16 | CRA-P05-C02 | full battle-setup publication feasibility | correctness_correction | A / after P01-C02 matrix | prerequisite completed | P05 deep review; P01 publication contribution |
| 17 | CRA-P05-C03 | preview resolution/ghost Floor authority | behavior_preserving_refactor | B / after P05-C01; public inventory | none (impact review) | P05 impact review |
| 18 | CRA-P05-C04 | one typed combat setup plan | behavior_preserving_refactor | B / after P05-C02; parity goldens | coimplemented | P05 deep review |
| 19 | CRA-P05-C05 | no-op mission branches/shape-only tests | cleanup | D / assertion transfer/caller inventory | none (impact review) | P05 impact review |
| 20 | CRA-P06-C01 | one terminal max-time outcome | correctness_correction | A / test first | prerequisite completed | P06 deep review |
| 21 | CRA-P06-C02 | total deterministic same-time event order | correctness_correction | A / semantic-key review | prerequisite completed | P06 deep review |
| 22 | CRA-P06-C03 | atomic world battle-advance pipeline | correctness_correction | A / after-or-with P06-C01 | prerequisite completed | P06 deep review |
| 23 | CRA-P06-C04 | disconnected event-log validator | cleanup | D / test transfer/external inventory | none (impact review) | P06 impact review |
| 24 | CRA-P06-C05 | one saturating MVP score | correctness_correction | C / extreme-value tests | none (impact review) | P06 impact review |
| 25 | CRA-P06-C06 | ghost `ScenarioEvent.once` | cleanup | D / external consumer inventory | none (impact review) | P06 impact review |
| 26 | CRA-P06-C07 | battle-log I/O at debug boundary | cleanup | D / external inventory/helper merge | none (impact review) | P06 impact review |
| 27 | CRA-P07-C01 | tactical effects from full final state | correctness_correction | A / failing Any/worker transition tests | independent | P07 deep review |
| 28 | CRA-P07-C02 | atomic tactical world mutations | correctness_correction | A / staged commit; coordinate P06-C03 | independent | P07 deep review; P06 seam contribution |
| 29 | CRA-P07-C03 | Rapier-only private production backend | cleanup | D / assertion transfer/external inventory | deferred/conditional | P07 deep review |
| 30 | CRA-P07-C04 | ghost movement tick state | cleanup | D / after P07-C03/external inventory | deferred/conditional | P07 deep review |
| 31 | CRA-P08-C01 | one typed trigger occurrence dispatcher | correctness_correction | A / live parasite-seed test first | independent | P08 deep review |
| 32 | CRA-P08-C02 | HP/death-aware runtime stat mutation | correctness_correction | A-gated / kill-credit policy | none (impact review) | P08 impact review |
| 33 | CRA-P08-C03 | saturating stat percentage arithmetic | correctness_correction | C / boundary tests | none (impact review) | P08 impact review |
| 34 | CRA-P08-C04 | effect/activation publication validation | correctness_correction | B-gated / split-ready | none (impact review) | P08 impact review; P01 publication contribution |
| 35 | CRA-P08-C05 | committed damage authority | behavior_preserving_refactor | B / after P08-C01/C02 characterization | independent | P08 deep review |
| 36 | CRA-P08-C06 | disconnected DeathHandler | cleanup | D / after P08-C01/external inventory | none (impact review) | P08 impact review |
| 37 | CRA-P09-C01 | projectile/basic compatibility publication | correctness_correction | A-gated / compatibility tests | none (impact review) | P09 impact review; P01 publication contribution |
| 38 | CRA-P09-C02 | max-kills terminal delivery exactly once | correctness_correction | A / test first | none (impact review) | P09 impact review |
| 39 | CRA-P09-C03 | usefulness over actual delivery targets | correctness_correction | A-gated / multi-step policy | independent | P09 deep review |
| 40 | CRA-P09-C04 | effect preview from footprints | correctness_correction | A-gated / projection policy | none (impact review) | P09 impact review |
| 41 | CRA-P09-C05 | continuous projectile motion kernel | behavior_preserving_refactor | B / after delivery/effect boundary | deferred/conditional | P09 deep review |
| 42 | CRA-P09-C06 | typed tile reach query/projection | behavior_preserving_refactor | B / characterization | independent | P09 deep review |
| 43 | CRA-P09-C07 | ghost range/public delivery surfaces | cleanup | D / serialized/external inventory | none (impact review) | P09 impact review |
| 44 | CRA-P09-C08 | exact-cause negative target tests | correctness_correction (test) | E / implementation ready | none (impact review) | P09 impact review |
| 45 | CRA-P10-C01 | typed command route/owner metadata | behavior_preserving_refactor | B / exhaustive descriptor tests | none (impact review) | P10 impact review |
| 46 | CRA-P10-C02 | one command-result projection authority | behavior_preserving_refactor | B / all-variant goldens/external API inventory | independent | P10 deep review |
| 47 | CRA-P10-C03 | reuse Core battle DTOs in server transport | behavior_preserving_refactor | B / exact serde parity after P10-C02 | deferred/conditional | P10 deep review |
| 48 | CRA-P10-C04 | typed host snapshot overlay | behavior_preserving_refactor | B / host-state table tests | independent | P10 deep review |
| 49 | CRA-P10-C05 | stable GameError code authority | behavior_preserving_refactor | C / exhaustive code table | none (impact review) | P10 impact review |
| 50 | CRA-P10-C06 | Core package targets/dependencies | cleanup | D / external/standalone inventory | none (impact review) | P10 impact review |
| 51 | CRA-P10-C07 | empty disconnected test module | cleanup | E / implementation ready | none (impact review) | P10 impact review |

## Boss 19 Exact Set

| Historical disposition | Exact members | Count |
| --- | --- | ---: |
| prerequisite completed | P06-C01, P06-C02, P06-C03, P01-C02, P05-C02 | 5 |
| coimplemented | P01-C01, P05-C04 | 2 |
| deferred/conditional | P07-C03, P07-C04, P09-C05, P10-C03 | 4 |
| independent | P07-C01, P07-C02, P08-C01, P08-C05, P09-C03, P09-C06, P10-C02, P10-C04 | 8 |
| Total | all are unchanged final CRA rows | 19 |

Each member was checked against the 51-row final matrix and was present. The remaining `51 - 19 = 32` rows are the
ones marked `none (impact review)` above. The Boss audit's slash-compressed prose is not machine-counted as proof;
the expanded set in this table is the P01/P05~P10 deep-review contract.

## Deterministic Checks Performed

```text
rg '^\| CRA-(F|P)[^|]+\|' FINAL_CANDIDATE_CATALOG.md | wc -l
# result: 51

rg '^\| CRA-(F|P)[^|]+\|' FINAL_CANDIDATE_CATALOG.md \
  | cut -d'|' -f2 | tr -d ' ' | LC_ALL=C sort | uniq -d
# result: no duplicate output

rg -o 'CRA-P[0-9]{2}-C[0-9]{2}' phase_*/CANDIDATE_CATALOG.md \
  | sed 's/.*://' | LC_ALL=C sort -u | wc -l
# result: 53
```

The original 51 IDs remain immutable historical identifiers. Current disposition can be
`fully_implemented`, `partially_implemented`, `still_valid`, `superseded`, `merged`, `split`,
`policy_or_external_gated` or `rejected`, but every final ID must reach exactly one Final disposition row.

