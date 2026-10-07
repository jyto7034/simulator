# Core Enhanced Refactoring Audit — Final Candidate Catalog

Catalog Date: 2026-08-07
Status: complete
Source candidates: 53
Final candidates: 51

Every unchanged row inherits the complete Problem Evidence, Current/Target Authority, Preserved Invariants, Removal
Scope, seven axes, Safety Net, Policy/External Decision, Readiness and Disposition from its Phase catalog. This Final
catalog does not replace those details; it normalizes cross-Phase identity, tier, dependencies and gates. The two
merged candidates below restate the full union explicitly.

Tiers are not score bands: `A` is high-impact test-first correction, `A-gated` has comparable importance but a policy
or compatibility gate, `B` is ordered correction/refactor, `C` is contained follow-up, `D` is cleanup/external-gated,
and `E` is a low-risk safety-net/mechanical enabler. Change frequency remains `unknown` for every row.

## Merged Candidates

### CRA-F-C01 — Validate The Complete Checkpoint Invariant Graph Before Restore

- Source IDs: `CRA-P02-C02`, `CRA-P03-C01`.
- Work Type: `correctness_correction`.
- Problem Evidence: current V3 validation checks many references but never composes seed/progression/map state with
  employee/equipment/loadout/fragment/economy ownership into one internally consistent graph. Release restore relies
  on a debug-only UUID-seed equality check and can install a checksummed inconsistent document.
- Current Authority And Consumers: map/progression, UUID manager, roster, inventory and fragments each own local
  predicates; `continue_from_active_run_checkpoint` and explicit load are the common pre-mutation consumers.
- Target Authority: one context-aware, typed pre-restore graph invokes domain validators for seed/progression and
  all persistent asset edges before any Core mutation. Continue may adopt a validated payload seed; explicit in-run
  load additionally matches current Core identity.
- Preserved Invariants: exact valid V3 shape/result, fixed map/content records, deterministic IDs, fail-closed error,
  no repair/rebind/migration, no partial state mutation and current checkpoint lifetime exclusions.
- Removal Or Consolidation Scope: remove partial cross-field admission and release reliance on debug assertion;
  consolidate the validation entry, not the domain predicate owners or persisted/runtime representations.
- Priority Axes And Total: `3 / 3 / 3 / 3 / 3 / 3 / unknown`; known subtotal `18/18` after union.
- Validation Safety Net: mutate one edge at a time across seeds, progression/node lists, roster UUID/definition,
  equipment reverse binding, fragment progress/pending/loadout and inventories; assert exact structured rejection and
  unchanged Core/export for Continue and explicit load; retain all valid round trips.
- Policy/External Contract Decision: none for rejecting internally inconsistent current-version documents. Repair,
  rebind or old-version migration remains separate policy work.
- Readiness: `test_first_then_implementation`; define domain predicate table before centralizing.
- Final Tier/Disposition: `A / priority_candidate`.

### CRA-F-C02 — Use The Common World Transaction For Headquarters Completion

- Source IDs: `CRA-P03-C05`, `CRA-P04-C02`.
- Work Type: `behavior_preserving_refactor`.
- Problem Evidence: `with_headquarters_transaction` clones/restores ten handwritten fields already covered by the
  common non-battle transaction, while completion temporarily removes active content to bypass HQ rejection. New
  root fields can silently escape rollback.
- Current Authority And Consumers: HQ branch logic owns recruit/emergency/open-shop policy; two transaction helpers
  compete for rollback coverage; common node completion owns map/progression/cleanup.
- Target Authority: the common non-battle transaction owns rollback; a thin HQ wrapper supplies explicit resolved-HQ
  completion context without hidden `take()` state manipulation.
- Preserved Invariants: queue/candidate/result identity, roster/UUID/currency/grants, Shop handoff, checkpoint/node
  cleanup, original errors, active-battle rejection and monotonic compendium exclusion.
- Removal Or Consolidation Scope: delete only the HQ rollback clone list and hidden bypass. Do not genericize all
  sessions or change branch/completion policy.
- Priority Axes And Total: `2 / 2 / 2 / 2 / 2 / 2 / unknown`; known subtotal `12/18`.
- Validation Safety Net: full-state success and injected late failure for recruit, emergency and open-shop/common
  completion; compare every non-battle field and monotonic exclusion.
- Policy/External Contract Decision: none for private parity.
- Readiness: `full_state_characterization_then_implementation`.
- Final Tier/Disposition: `B / candidate`.

## Exact Final Matrix

| Final/source ID | Candidate | Type | Known subtotal | Normalized readiness | Tier / final disposition |
| --- | --- | --- | ---: | --- | --- |
| CRA-F-C01 | complete checkpoint invariant graph | correction | 18 | test first | A / priority |
| CRA-F-C02 | common HQ transaction | refactor | 12 | full-state characterization | B / candidate |
| CRA-P01-C01 | embedded source inventory authority | refactor | 9 | exact-set characterization | B / candidate |
| CRA-P01-C02 | every manifested difficulty at publication | correction | 13 | negative matrix test | A / priority |
| CRA-P01-C03 | private parsed→published mapping | refactor | 8 | private scope frozen | C / candidate |
| CRA-P02-C01 | host monotonic metadata on every current publish | correction | 14 | real-file lower-input tests | A / priority |
| CRA-P02-C03 | RunState↔V3 checkpoint conversion | refactor | 13 | golden field parity after F-C01 | B / candidate |
| CRA-P02-C04 | MapProgression mutation/availability | refactor | 13 | transition characterization | B / candidate |
| CRA-P03-C02 | atomic enhancement material aggregation | correction | 13 | duplicate-row tests | A / priority |
| CRA-P03-C03 | bidirectional equipment/employee loadout | refactor | 16 | after F-C01; transition table | B / priority refactor |
| CRA-P03-C04 | five-ledger grant transaction state | refactor | 14 | caller parity | B / candidate |
| CRA-P04-C01 | VN repeat/source publication matrix | correction | 9 | structured negative tests | A / priority |
| CRA-P04-C03 | Event/Shop persisted-active commits | refactor | 11 | paired-state characterization | B / candidate |
| CRA-P04-C04 | VN assembly and explicit queue placement | refactor | 11 | world Monolith characterization | C / candidate |
| CRA-P05-C01 | keep serialized preview cache out of setup authority | correction | 18 | exact durable-Continue test | A / priority |
| CRA-P05-C02 | full battle-setup publication feasibility | correction | 16 | after P01-C02 matrix | A / priority |
| CRA-P05-C03 | preview resolution/ghost Floor authority | refactor | 14 | after P05-C01; public inventory | B / candidate |
| CRA-P05-C04 | one typed combat setup plan | refactor | 14 | after P05-C02; parity goldens | B / candidate |
| CRA-P05-C05 | no-op mission branches/shape-only tests | cleanup | 7 | assertion transfer/caller inventory | D / cleanup |
| CRA-P06-C01 | one terminal max-time outcome | correction | 17 | test first | A / priority |
| CRA-P06-C02 | total deterministic same-time event order | correction | 17 | cross-variant semantic-key review | A / priority |
| CRA-P06-C03 | atomic world battle-advance pipeline | correction | 18 | after/with P06-C01 | A / priority |
| CRA-P06-C04 | disconnected event-log validator | cleanup | 10 | test transfer/external inventory | D / cleanup |
| CRA-P06-C05 | one saturating MVP score | correction | 12 | extreme-value tests | C / candidate |
| CRA-P06-C06 | ghost `ScenarioEvent.once` | cleanup | 4 | external consumer inventory | D / cleanup |
| CRA-P06-C07 | battle-log I/O at debug boundary | cleanup | 3 | external inventory/helper merge | D / cleanup |
| CRA-P07-C01 | tactical effects from full final state | correction | 16 | failing Any/worker transition tests | A / priority |
| CRA-P07-C02 | atomic tactical world mutations | correction | 18 | staged commit; coordinate P06-C03 | A / priority |
| CRA-P07-C03 | Rapier-only private production backend | cleanup | 13 | assertion transfer/external inventory | D / candidate |
| CRA-P07-C04 | ghost movement tick state | cleanup | 9 | after P07-C03/external inventory | D / cleanup |
| CRA-P08-C01 | one typed trigger occurrence dispatcher | correction | 17 | live parasite-seed test first | A / priority |
| CRA-P08-C02 | HP/death-aware runtime stat mutation | correction | 16 | kill-credit policy required | A-gated / priority blocked |
| CRA-P08-C03 | saturating stat percentage arithmetic | correction | 10 | boundary tests | C / candidate |
| CRA-P08-C04 | effect/activation publication validation | correction | 13 | references ready; OnHit matrix gated | B-gated / split-ready candidate |
| CRA-P08-C05 | committed damage authority | refactor | 14 | after P08-C01/C02 characterization | B / priority refactor |
| CRA-P08-C06 | disconnected DeathHandler | cleanup | 10 | after P08-C01/assertion transfer/external | D / cleanup |
| CRA-P09-C01 | projectile/basic compatibility publication | correction | 15 | compatibility approval and tests | A-gated / priority blocked |
| CRA-P09-C02 | max-kills terminal delivery exactly once | correction | 14 | test first | A / priority |
| CRA-P09-C03 | usefulness over actual delivery targets | correction | 14 | multi-step policy first | A-gated / candidate |
| CRA-P09-C04 | effect preview from footprints | correction | 13 | after P09-C06 and projection policy | A-gated / candidate |
| CRA-P09-C05 | continuous projectile motion kernel | refactor | 14 | after P09-C01/C02 and P08-C05 boundary | B / priority refactor |
| CRA-P09-C06 | typed tile reach query/projection | refactor | 11 | characterization | B / candidate |
| CRA-P09-C07 | ghost range/public delivery surfaces | cleanup | 10 | serialized/external inventory | D / deferred cleanup |
| CRA-P09-C08 | exact-cause negative target tests | test correction | 11 | implementation ready | E / safety-net enabler |
| CRA-P10-C01 | typed command route/owner metadata | refactor | 13 | exhaustive descriptor tests | B / priority refactor |
| CRA-P10-C02 | one command-result projection authority | refactor | 17 | all-variant goldens/external API inventory | B / priority refactor |
| CRA-P10-C03 | reuse Core battle DTOs in server transport | refactor | 14 | exact serde parity after P10-C02 | B / candidate |
| CRA-P10-C04 | typed host snapshot overlay | refactor | 17 | host-state table tests | B / priority refactor |
| CRA-P10-C05 | stable GameError code authority | refactor | 11 | exhaustive code table | C / candidate |
| CRA-P10-C06 | Core package targets/dependencies | cleanup | 12 | external/standalone inventory | D / external-gated cleanup |
| CRA-P10-C07 | empty disconnected test module | cleanup | 3 | implementation ready | E / low-priority cleanup |

The matrix contains 51 rows: 2 merged plus 49 unchanged. The four source IDs absorbed by F-C01/F-C02 do not appear
again.

## Recommended Implementation Waves

### Wave 0 — Safety nets and decisions

- P09-C08 exact-cause negative fixtures.
- Freeze P06-C02 semantic ordering key; P08-C02 kill credit; P09-C01 compatibility; P09-C03 multi-step usefulness;
  P09-C04 per-step/union DTO; P08-C04 OnHit meaning.
- Inventory public/external Rust, Unity fixture, standalone Core and admin consumers before any public cleanup.

### Wave 1 — Publication and persistence corrections

- P01-C02 then P05-C02; P04-C01; reference-only part of P08-C04.
- F-C01, P02-C01 and P03-C02 independently test-first.
- P05-C01 durable cache/setup authority correction.

### Wave 2 — Battle correctness

- P06-C01 then P06-C03; P06-C02 after key review.
- P07-C01 and P07-C02 with a coordinated staging primitive, not one transaction.
- P08-C01; P09-C02.

### Wave 3 — Corrected-authority refactors

- P02-C03/P03-C03 after F-C01; P03-C04/F-C02/P04-C03.
- P05-C03/P05-C04 after their respective corrections.
- P08-C05 and P09-C05 after effect/delivery corrections.
- P10-C01/C02/C04, then P10-C03; P10-C05 independently.

### Wave 4 — Cleanup

- P10-C07 is ready but low impact.
- All other cleanup waits for assertion transfer, dependency order and/or external compatibility inventory.

## Policy And External Gate Index

| Gate | Affected candidates/signals | Required decision |
| --- | --- | --- |
| save/content | Phase signals P02/P03 | strict rejection vs migration/repair/rebind/lease |
| admin | P02/P03/P10 signals | persistence, rollback, revision and publication semantics |
| public Rust/API | P01-C03-related API signal, P05-C03/C05, P06-C04/C06/C07, P07-C03/C04, P08-C05/C06, P09-C07, P10-C02/C06 | inventory, deprecate or approve break |
| effect/death | P08-C02/C04 | modifier kill credit and OnHit counterpart meaning |
| targeting/projection | P09-C01/C03/C04/C07 | compatibility, multi-step usefulness, per-step/union and serialized migration |
| command wire | P10 signal S01 | preserve direct-map `RunComplete` or approve consumer migration |
| external fixtures/Unity | P05/P09/P10 cleanup signals | current consumer and exact fixture comparison |

## Final Disposition Summary

- 14 policy-free Tier A correction rows are test-first priorities.
- 5 correctness rows are policy/compatibility gated (`A-gated`/`B-gated`), with P08-C04's reference-only subset
  independently actionable.
- Behavior-preserving refactors are ordered behind their correction/characterization prerequisites.
- Cleanup is not urgency-ranked above correctness; most public cleanup remains external-gated.
- Two low-risk enablers are ready: P09-C08 test correction and P10-C07 empty-file cleanup.

No candidate in this catalog is implemented by the audit itself.
