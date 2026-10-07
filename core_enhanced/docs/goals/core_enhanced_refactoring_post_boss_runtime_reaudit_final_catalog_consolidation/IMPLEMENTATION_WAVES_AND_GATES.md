# Implementation Waves And Gates

These waves cover the 44 `still_valid` final identities exactly once. They are a recommendation for future,
candidate-scoped `implement_and_verify` Goals, not authorization in this read-only campaign. The seven closed rows are
excluded from implementation waves and retain their regression tests.

The five previously open choices are approved in `APPROVED_POLICY_DECISIONS.md` as CRA-POL-001~005.
CRA-POL-006 establishes the compatibility-preserving default. Wave placement is unchanged, but Wave 2 now waits on
decision-table tests and dependencies rather than further policy selection.

For one-candidate-at-a-time execution, `EXACT_LINEAR_IMPLEMENTATION_ORDER.md` is the authoritative recommended total
order. The Wave table below remains a work-type/risk classification and exact coverage proof; it is not a strict
temporal barrier. In particular, P09-C06 must precede P09-C03/C04, and low-value P10-C07 is placed at linear position
35 because it enables no correction.

## Exact Wave Coverage

| Wave | Purpose | Exact IDs | Count |
| --- | --- | --- | ---: |
| 0 | low-risk safety/test enablers | P09-C08, P10-C07 | 2 |
| 1 | policy-free correctness and atomicity | F-C01, P02-C01, P03-C02, P04-C01, P05-C01, P06-C05, P07-C01, P07-C02, P08-C01, P08-C03, P09-C02 | 11 |
| 2 | correctness under approved policy/compatibility contracts | P08-C02, P08-C04, P09-C01, P09-C03, P09-C04 | 5 |
| 3 | behavior-preserving authority consolidation | F-C02, P01-C03, P02-C03, P02-C04, P03-C03, P03-C04, P04-C03, P04-C04, P05-C03, P08-C05, P09-C05, P09-C06, P10-C01, P10-C02, P10-C03, P10-C04, P10-C05 | 17 |
| 4 | cleanup after assertion transfer and consumer inventory | P05-C05, P06-C04, P06-C06, P06-C07, P07-C03, P07-C04, P08-C06, P09-C07, P10-C06 | 9 |
| **Total** | all open identities |  | **44** |

## Recommended Order Inside Waves

### Wave 0 — safety-net/mechanical enablers

1. P09-C08 repairs false-positive negative fixtures and makes later publication/targeting corrections trustworthy.
2. P10-C07 can remove only the disconnected empty test file after before/after metadata/test listing. It remains a
   Wave 0 risk classification, but the exact linear plan defers it to position 35 so it cannot delay corrections.

### Wave 1 — policy-free correctness

- Start F-C01, P02-C01, P03-C02, P04-C01 and P05-C01 independently with their named failing/negative tests.
- P06-C05 and P08-C03 are contained numeric corrections after boundary tables.
- P07-C01 and P07-C02 may share staging/final-state utilities but remain separate Goals/authorities from the closed
  P06 advance transaction.
- P08-C01 precedes P08-C05 and P08-C06. P09-C02 precedes the P09-C05 motion refactor.

### Wave 2 — approved policy contracts, then corrections

- P08-C02 follows CRA-POL-001: lethal MaxHealth clamping enters normal death finalization and uses causal source
  attribution/kill credit.
- P08-C04 follows CRA-POL-002: OnHit Counterpart BonusDamage is one non-recursive retaliation against the attacker;
  missing-reference validation remains independently implementable.
- P09-C01 follows CRA-POL-003: invalid basic/skill delivery definitions fail strict atomic publication without
  normalization or fallback.
- P09-C03 follows CRA-POL-004: cast admission is useful when any step is useful over its actual delivery set; runtime
  execution gates the current step.
- P09-C04 follows CRA-POL-005: publish the deterministic union of actual step footprints without changing the DTO.

Each implementation Goal must cite its approved policy ID and land the named decision-table tests first. No
permissive fallback or indefinite dual behavior is authorized.

### Wave 3 — authority consolidation

- Private foundations can start after characterization: P01-C03, P02-C04, P04-C03/C04, P09-C06 and
  P10-C01/C02/C04/C05.
- P02-C03 and P03-C03 follow F-C01 so invalid imported graphs are rejected before mapping/transition cleanup.
- P03-C04 and F-C02 use caller/full-state parity and remain distinct grant-state versus HQ-transaction targets.
- P05-C03 follows P05-C01; P08-C05 follows P08-C01 and the P08-C02 policy; P09-C05 follows P09-C01/C02 plus the
  P08-C05 delivery/effect boundary.
- P10-C03 follows P10-C02 and exact serde parity. Its ordinary DTO refactor does not wait for a Boss wire; any future
  Boss-specific expansion is a separate conditional policy event.

### Wave 4 — cleanup

- Transfer assertions before P05-C05, P06-C04, P07-C03 and P08-C06 removal.
- P07-C04 follows P07-C03 so retained Rapier/default contracts determine which fields are truly ghost state.
- P06-C06/C07, P09-C07 and P10-C06 require public/standalone/serialized consumer inventory before narrowing APIs or
  targets. Retained fixture tools must be explicit and test-harness noise removed without changing output.

## Dependency Edges

| From | To | Reason |
| --- | --- | --- |
| P09-C08 | P09-C01 | repair exact-cause negative fixtures before expanding strict publication rejection |
| F-C01 | P02-C03, P03-C03 | reject invalid imported state before consolidating conversion/loadout transitions |
| P03-C04 | F-C02 | characterize grant-ledger callers before replacing the HQ transaction wrapper |
| P04-C03 | F-C02 | freeze HQ Shop persisted/active commits before replacing the HQ transaction wrapper |
| P05-C01 | P05-C03 | durable preview-cache correctness precedes resolver/ghost-floor cleanup |
| P08-C01 + P08-C02 + CRA-POL-001/002 | P08-C05 | occurrence and HP/death/retaliation semantics bound committed-damage consolidation |
| P08-C01 | P08-C06 | transfer the only useful false-authority death assertion first |
| P09-C01 + P09-C02 + P08-C05 | P09-C05 | publication, terminal delivery and committed-hit boundaries must be frozen before shared motion extraction |
| P09-C06 | P09-C03, P09-C04 | typed footprint query precedes actual-set usefulness and approved union preview correction |
| P10-C02 + P09-C04 | P10-C03 | result transport classification and final preview content precede DTO composition cleanup |
| P07-C03 | P07-C04 | retained backend establishes real input/output consumers before ghost-field removal |

## Policy, Compatibility And External Gate Index

| Gate | Current authoritative shape/lifetime | Affected candidates | Required decision/evidence |
| --- | --- | --- | --- |
| current V3 checkpoint | strict current-version document; private active battle/Boss graph excluded | F-C01, P02-C03/C04, P03-C03 | CRA-POL-006 approves strict rejection; repair/rebind/old-version migration or serialized-field deletion requires a new policy and consumer rollout |
| live RON publication | manifest/source inventory and atomic whole-candidate publication | P04-C01, P08-C04, P09-C01/C07 | CRA-POL-002/003/006 define exact strict behavior; implement typed negative matrices with no partial startup or silent normalization |
| effect/death semantics | normal committed damage/death authority plus approved modifier/retaliation rules | P08-C02/C04/C05 | implement CRA-POL-001/002 decision tables; reference-only C04 work may remain a sub-goal, not a new catalog ID |
| targeting/projection | actual delivery-set usefulness and existing one-collection DTO | P09-C03/C04, P10-C03 | implement CRA-POL-004/005 and prove exact server/Unity union-wire parity; a per-step DTO requires a new migration policy |
| command/result wire | current result strings/payloads, direct-map RunComplete, top-level setup/update, nested resync | P10-C02/C03/C04/C05 | CRA-POL-006 requires exact parity; any RunComplete/tag/nesting change is a separately approved consumer migration |
| first public Boss wire | private non-serde runtime, no save/checkpoint/lifecycle field | P10-C03 conditional subset | CRA-POL-006 keeps it private until a real producer/consumer exists; then a new version/rollout policy is required |
| public Rust/API | workspace consumers confirmed; standalone/deployed revisions unknown | P01-C03 public-removal signal, P05-C03/C05, P06-C04/C06/C07, P07-C03/C04, P08-C05/C06, P09-C07, P10-C02/C06 | CRA-POL-006 requires symbol/build/tool inventory plus deprecation/versioned migration; private consolidation may proceed when it does not narrow API |
| Unity/external fixtures | accessible current source consumes common battle shape; deployed state unknown | P09-C04/C07, P10-C03/C06 | prove CRA-POL-005/006 parity by exact fixture/build comparison; inaccessible deployment remains explicit risk |
| admin lifecycle | separate environment/token-gated path | deferred signals around P02/P03/P10 | persistence, rollback, revision and publication semantics require feature policy; not a current 51-row implementation scope |

## Consumer Classification

- `workspace_confirmed`: Core runtime, production live data, game_server source/tests and in-workspace tools were read.
- `canonical_external`: current common Unity battle transport is documented, but its independent refactor is excluded.
- `unknown_deployment`: standalone public Rust/API/tool and deployed Unity revisions are not proven absent.
- `explicitly_excluded`: independent game_server, game_resources and Unity cleanup/refactor candidates.

Negative symbol searches never upgrade `unknown_deployment` to “no consumer.”
