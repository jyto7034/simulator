# Rebaselined Candidate Catalog

Catalog date: 2026-08-09
Source candidates: 53
Final identities: 51
Confirmed new candidates: 0

## Reading The Matrix

Axes are `ownership / correctness-atomicity / lifecycle / change amplification / test fragility / failure blast
radius / change frequency`. Every open row has current `unknown` change frequency because the dirty snapshot is not a
frequency history. Tier and readiness are separate from the score. Closed rows retire priority/readiness rather than
pretending completed work is still implementation backlog.

Each row inherits its full current Problem Evidence, current/target authority, preserved invariant, removal scope,
Boss impact, safety net and gate from the matching P01~P10 `CANDIDATE_REAUDIT.md`, `TEST_REAUDIT.md` and
`BOUNDARY_AUDIT.md`. The two cross-phase identities are restated below because they consume four source IDs.

Post-audit policy approval is recorded in `APPROVED_POLICY_DECISIONS.md`. CRA-POL-001~005 close the five direct
gameplay/publication choices; CRA-POL-006 freezes compatibility-preserving defaults. Candidate dispositions and
counts do not change: approval makes the five policy-gated corrections test-first implementation-ready, while
consumer inventories and dependency edges remain evidence gates where named.

The dependency-researched one-candidate-at-a-time order is recorded in `EXACT_LINEAR_IMPLEMENTATION_ORDER.md`.
It contains every open identity exactly once, distinguishes required edges from conservative rework-avoidance
ordering, and supersedes interpreting Wave numbers as a strict temporal sequence.

## Exact Current 51

| Final ID | Candidate | Current disposition | Current work type | Axes / subtotal | Tier; readiness or closure | Boss relation |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-F-C01 | complete checkpoint invariant graph | `still_valid` | `correctness_correction` | `3/3/3/3/3/3/unknown = 18` | A; test first, domain predicate table | other-impact |
| CRA-F-C02 | common HQ transaction | `still_valid` | `behavior_preserving_refactor` | `2/2/2/2/2/2/unknown = 12` | B; full-state characterization | other-impact |
| CRA-P01-C01 | embedded source inventory authority | `fully_implemented` | `behavior_preserving_refactor` | `not_applicable — closed` | closed; retain exact-set safety nets | coimplemented |
| CRA-P01-C02 | every manifested difficulty at publication | `fully_implemented` | `correctness_correction` | `not_applicable — closed` | closed; retain negative matrix | prerequisite |
| CRA-P01-C03 | private parsed→published mapping | `still_valid` | `behavior_preserving_refactor` | `2/1/1/3/1/2/unknown = 10` | C; characterize private conversion | other-impact |
| CRA-P02-C01 | host monotonic metadata on every publish | `still_valid` | `correctness_correction` | `2/3/2/1/3/3/unknown = 14` | A; real-file lower-input tests first | other-impact |
| CRA-P02-C03 | RunState↔V3 checkpoint conversion | `still_valid` | `behavior_preserving_refactor` | `2/1/2/3/2/3/unknown = 13` | B; golden field parity after F-C01 | other-impact |
| CRA-P02-C04 | MapProgression mutation/availability | `still_valid` | `behavior_preserving_refactor` | `3/2/2/2/2/2/unknown = 13` | B; transition characterization | other-impact |
| CRA-P03-C02 | atomic enhancement material aggregation | `still_valid` | `correctness_correction` | `1/3/1/2/3/3/unknown = 13` | A; duplicate-row tests first | other-impact |
| CRA-P03-C03 | bidirectional equipment/employee loadout | `still_valid` | `behavior_preserving_refactor` | `3/2/2/3/3/3/unknown = 16` | B; after F-C01, transition table | other-impact |
| CRA-P03-C04 | five-ledger grant transaction state | `still_valid` | `behavior_preserving_refactor` | `3/1/2/3/2/3/unknown = 14` | B; caller parity first | other-impact |
| CRA-P04-C01 | VN repeat/source publication matrix | `still_valid` | `correctness_correction` | `1/2/1/1/2/2/unknown = 9` | A; structured negatives first | other-impact |
| CRA-P04-C03 | Event/Shop persisted-active commits | `still_valid` | `behavior_preserving_refactor` | `2/1/2/2/2/2/unknown = 11` | B; paired-state characterization | other-impact |
| CRA-P04-C04 | VN assembly and explicit queue placement | `still_valid` | `behavior_preserving_refactor` | `2/1/2/2/2/2/unknown = 11` | C; world/Monolith characterization | other-impact |
| CRA-P05-C01 | keep serialized preview cache out of setup authority | `still_valid` | `correctness_correction` | `3/3/3/3/3/3/unknown = 18` | A; exact durable-Continue test first | other-impact |
| CRA-P05-C02 | full battle-setup publication feasibility | `fully_implemented` | `correctness_correction` | `not_applicable — closed` | closed; retain manifested/Boss negatives | prerequisite |
| CRA-P05-C03 | preview resolution/ghost Floor authority | `still_valid` | `behavior_preserving_refactor` | `3/1/2/3/3/2/unknown = 14` | B; after C01, public caller inventory | other-impact |
| CRA-P05-C04 | one typed combat setup plan | `fully_implemented` | `behavior_preserving_refactor` | `not_applicable — closed` | closed; retain parity/ordinary opt-out | coimplemented |
| CRA-P05-C05 | no-op mission branches/shape-only tests | `still_valid` | `cleanup` | `1/1/1/1/2/1/unknown = 7` | D; assertion transfer/caller inventory | other-impact |
| CRA-P06-C01 | one terminal max-time outcome | `fully_implemented` | `correctness_correction` | `not_applicable — closed` | closed; retain cap/finalization matrix | prerequisite |
| CRA-P06-C02 | total deterministic same-time event order | `fully_implemented` | `correctness_correction` | `not_applicable — closed` | closed; retain all-variant order matrix | prerequisite |
| CRA-P06-C03 | atomic world battle-advance pipeline | `fully_implemented` | `correctness_correction` | `not_applicable — closed` | closed; retain staged failure matrix | prerequisite |
| CRA-P06-C04 | disconnected event-log validator | `still_valid` | `cleanup` | `2/1/1/2/3/1/unknown = 10` | D; transfer checks/external inventory | other-impact |
| CRA-P06-C05 | one saturating MVP score | `still_valid` | `correctness_correction` | `2/2/1/2/3/2/unknown = 12` | C; extreme-value tests first | other-impact |
| CRA-P06-C06 | ghost ScenarioEvent.once | `still_valid` | `cleanup` | `1/0/1/1/1/0/unknown = 4` | D; external consumer inventory | other-impact |
| CRA-P06-C07 | battle-log I/O at debug boundary | `still_valid` | `cleanup` | `1/0/0/1/1/0/unknown = 3` | D; external inventory/helper merge | other-impact |
| CRA-P07-C01 | tactical effects from full final state | `still_valid` | `correctness_correction` | `3/2/3/3/3/2/unknown = 16` | A; failing Any/worker transitions first | independent |
| CRA-P07-C02 | atomic tactical world mutations | `still_valid` | `correctness_correction` | `3/3/3/3/3/3/unknown = 18` | A; staged command tests first | independent |
| CRA-P07-C03 | Rapier-only private production backend | `still_valid` | `cleanup` | `3/1/1/3/3/2/unknown = 13` | D; assertion transfer/external inventory | deferred/conditional |
| CRA-P07-C04 | ghost movement tick state | `still_valid` | `cleanup` | `2/0/2/2/2/1/unknown = 9` | D; after C03 and consumer inventory | deferred/conditional |
| CRA-P08-C01 | one typed trigger occurrence dispatcher | `still_valid` | `correctness_correction` | `3/3/3/3/3/2/unknown = 17` | A; live parasite-seed test first | independent |
| CRA-P08-C02 | HP/death-aware runtime stat mutation | `still_valid` | `correctness_correction` | `2/3/3/2/3/3/unknown = 16` | A; CRA-POL-001 approved, decision tests first | other-impact |
| CRA-P08-C03 | saturating stat percentage arithmetic | `still_valid` | `correctness_correction` | `1/2/1/2/2/2/unknown = 10` | C; boundary tests first | other-impact |
| CRA-P08-C04 | effect/activation publication validation | `still_valid` | `correctness_correction` | `2/2/1/3/3/2/unknown = 13` | B; CRA-POL-002 approved, matrix tests first | other-impact |
| CRA-P08-C05 | committed damage authority | `still_valid` | `behavior_preserving_refactor` | `3/1/3/3/2/2/unknown = 14` | B; after C01/C02 characterization | independent |
| CRA-P08-C06 | disconnected DeathHandler | `still_valid` | `cleanup` | `3/1/2/1/2/1/unknown = 10` | D; after C01/assertion transfer/external | other-impact |
| CRA-P09-C01 | projectile/basic compatibility publication | `still_valid` | `correctness_correction` | `2/3/2/3/3/2/unknown = 15` | A; CRA-POL-003 approved, compatibility tests first | other-impact |
| CRA-P09-C02 | max-kills terminal delivery exactly once | `still_valid` | `correctness_correction` | `1/3/3/2/3/2/unknown = 14` | A; exact terminal tests first | other-impact |
| CRA-P09-C03 | usefulness over actual delivery targets | `still_valid` | `correctness_correction` | `3/2/2/3/2/2/unknown = 14` | A; CRA-POL-004 approved, actual-set tests first | independent |
| CRA-P09-C04 | effect preview from footprints | `still_valid` | `correctness_correction` | `2/2/1/3/3/2/unknown = 13` | A; after C06, CRA-POL-005 union parity tests | other-impact |
| CRA-P09-C05 | continuous projectile motion kernel | `still_valid` | `behavior_preserving_refactor` | `3/1/3/3/2/2/unknown = 14` | B; after delivery/effect boundary | deferred/conditional |
| CRA-P09-C06 | typed tile reach query/projection | `still_valid` | `behavior_preserving_refactor` | `3/1/1/2/2/2/unknown = 11` | B; characterization first | independent |
| CRA-P09-C07 | ghost range/public delivery surfaces | `still_valid` | `cleanup` | `2/1/1/3/2/1/unknown = 10` | D; serialized/external inventory | other-impact |
| CRA-P09-C08 | exact-cause negative target tests | `still_valid` | `correctness_correction (test)` | `1/3/1/2/3/1/unknown = 11` | E; implementation ready | other-impact |
| CRA-P10-C01 | typed command route/owner metadata | `still_valid` | `behavior_preserving_refactor` | `3/1/2/3/2/2/unknown = 13` | B; exhaustive descriptor tests first | other-impact |
| CRA-P10-C02 | one command-result projection authority | `still_valid` | `behavior_preserving_refactor` | `3/2/3/3/3/3/unknown = 17` | B; all-variant goldens/API inventory | independent |
| CRA-P10-C03 | reuse Core battle DTOs in server transport | `still_valid` | `behavior_preserving_refactor` | `2/1/2/3/3/3/unknown = 14` | B; exact serde parity after C02 | deferred/conditional |
| CRA-P10-C04 | typed host snapshot overlay | `still_valid` | `behavior_preserving_refactor` | `3/2/3/3/3/3/unknown = 17` | B; host-state table tests first | independent |
| CRA-P10-C05 | stable GameError code authority | `still_valid` | `behavior_preserving_refactor` | `3/1/1/2/2/2/unknown = 11` | C; exhaustive code table first | other-impact |
| CRA-P10-C06 | Core package targets/dependencies | `still_valid` | `cleanup` | `2/1/2/3/2/2/unknown = 12` | D; external/standalone inventory | other-impact |
| CRA-P10-C07 | empty disconnected test module | `still_valid` | `cleanup` | `1/0/0/1/1/0/unknown = 3` | E; implementation ready | other-impact |

## Merged Identity Evidence

`CRA-F-C01` still consumes exactly `CRA-P02-C02 + CRA-P03-C01`. Current validation rejects some local references but
does not compose run/UUID seed and graph-derived progression with roster, equipment reverse bindings, fragments and
economy into one pre-restore predicate. The target remains one fail-closed current-version graph before mutation;
repair/rebind/migration stays separate policy work.

`CRA-F-C02` still consumes exactly `CRA-P03-C05 + CRA-P04-C02`. HQ completion retains the handwritten ten-root
rollback beside the common transaction and temporarily hides active content. The target remains the common private
transaction plus explicit HQ completion context, preserving HQ branch policy and compendium monotonic exclusion.

## Current Disposition Summary

- `fully_implemented`: 7 — P01-C01/C02, P05-C02/C04, P06-C01/C02/C03.
- `still_valid`: 44 — 17 corrections, 17 behavior-preserving refactors and 10 cleanup rows.
- `partially_implemented`, `superseded`, `split`, `policy_or_external_gated` as a whole-row disposition, and
  `rejected`: 0. Gates remain attributes on otherwise current candidates.
- Confirmed new `CRR-*` candidates: 0.
- Approved policy IDs: CRA-POL-001~006. The five direct policy choices are closed; compatibility/consumer evidence
  required by CRA-POL-006 remains a non-policy implementation gate.

Every open row requires a separately authorized `implement_and_verify` Goal. This audit implements none of them.
