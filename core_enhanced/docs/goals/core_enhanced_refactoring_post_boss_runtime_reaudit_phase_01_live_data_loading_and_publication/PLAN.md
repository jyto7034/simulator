# Core Enhanced Refactoring Post-Boss Reaudit Phase 01 Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Execution Mode: investigation_only
Production Mutation Authorized: no
Master Phase: 1 — live data loading/publication and CRA-P01/Boss source inventory impact
Direct Dependency: P00 `Complete / High`

## Objective

Current live-data source inventory, parse/compile/validation/publication path and official tests against the full
original scope of CRA-P01-C01~C03, deeply reverify the Boss coimplementation/prerequisite claims, and record current
dispositions, test safety nets, gates and any evidence-confirmed new candidate.

## User/Contract-Visible Problem

Boss Runtime added a new embedded source, raw compiler and PVE reference/setup validation to an already duplicated
publication pipeline. Historical claims say P01-C01 was coimplemented and P01-C02 was prerequisite-complete, but a
Boss-only source path or one manifested encounter is not proof that the original all-source/all-difficulty invariant
closed. P01-C03's private parsed→published mapping may also have changed without being named by Boss work.

## Audit Finding ID

None. Reaudit scope: CRA-P01-C01, CRA-P01-C02, CRA-P01-C03 and new `CRR-P01-*` proposals if proven.

## Current Runtime And Live Data Evidence

P00 confirms a current embedded Boss source, compile/reference validation, one production Boss reference and a
96-file relevant content baseline. Candidate-specific current evidence is `discovery pending` until this Phase reads
the original P01 documents and current loader/data/test/canonical path.

## Source Of Truth And Ownership

- current loader and database publication mutation
- production live RON actually embedded/read
- `data_loading_contract.md` and topic canonical selected by `README.md`
- official embedded/live integration tests plus current domain tests
- supporting setup/consumer evidence only as required to determine publication feasibility

P01 owns source inventory and atomic publication. P05 owns resolved combat setup; P08/P09 own effect/delivery
semantics. P01 records their publication contributions without closing their candidates.

## Invariants

- valid current bundles publish the same complete `GameDataBase` contract.
- invalid/unsupported source, reference or feasibility input rejects the candidate before live swap.
- every actual embedded source is represented exactly once without prose/manual-count authority.
- every production-manifested difficulty/seed required by the original contract is validated, not only Boss data.
- private parse→published mapping does not become public/compatibility surface without evidence.
- no production, live RON, test, canonical, server, Unity or historical Goal mutation.
- exact CRA identity and P00 dirty/content baselines remain preserved.

## Candidate Reconciliation Requirements

For each CRA-P01-C01~C03 record current problem evidence, authority/producer/consumer, Boss impact, preserved
invariant/removal scope, work type, axes/readiness, safety net, policy/external gate and one Master-approved current
disposition. Candidate-specific evidence must cover its original full scope.

New proposals use `CRR-P01-Cyy` and require current root-cause evidence, not a search hit.

## Approved Policy And Evidence

Master RBA-01~RBA-14; P00 exact inventory/baseline/boundary artifacts; original P01 historical catalog and completion
artifacts; current workflow/authoring/completion-review guides.

## Open Policy Decisions

None at activation. A required RON schema, compatibility, content, difficulty/seed or consumer rollout decision gates
the affected conclusion without blocking policy-independent rows.

## In Scope

- original P01 Goal/catalog/boundary/test evidence as history
- `data/live_ron.rs`, `data/mod.rs`, domain source inventory/validation helpers and `GameDataBase` publication
- production live RON set, including Boss source and PVE binding
- official `ron_loading` and relevant data/unit/publication/preview setup assertions
- current data-loading canonical and directly necessary supporting canonical
- P05/P08/P09 publication contribution and P10 public/private signal only as boundary handoff
- P01 audit artifacts and completion audit

## Out Of Scope

- production/canonical/test/data changes or formatter
- P05 full scenario/setup, P08 effect semantics, P09 targeting/delivery or P10 external contract disposition
- gameplay/content/schema/migration policy decisions
- independent game_resources/server/Unity refactor discovery
- historical Goal rewrite or actual implementation

## Investigation Artifacts

- `CANDIDATE_REAUDIT.md`
- `TEST_REAUDIT.md`
- `BOUNDARY_AUDIT.md`
- `COMPLETION_AUDIT.md`

## DTO/Data/Save Migration

Not applicable to this read-only Phase. Current schema/publication gates are recorded as candidate evidence only.

## Deployment And Compatibility

No deployment. Public/private and inaccessible consumer evidence is classified; compatibility policy is not inferred.

## Focused Validation

- exact current embedded source set versus production files and loader fields
- candidate publication call graph and atomicity/failure path source inspection
- actual manifested-difficulty/seed enumeration and setup-validation coverage
- P01 test setup/action/assertion and production reachability audit
- candidate current evidence and cardinality checks
- baseline/content/dirty preservation checks

A narrow runner is used only if static/test-source evidence leaves a concrete current behavior gap.

## Broad Validation

Only the smallest current live-loading runner needed by a candidate disposition. A broad workspace run is not a
substitute for source/test assertion audit.

## Omitted Validation

Formatter, server process, WebSocket, Unity build and performance/balance tests are omitted absent a current public
contract or behavior gap requiring them.

## Completion Conditions

- CRA-P01-C01~C03 each have one evidence-backed current disposition over the original full scope.
- P01-C01/P01-C02 Boss historical claims are deeply verified, changed or rejected rather than copied.
- P01-C03 current mapping/public scope is independently checked.
- current production source/publication path and test families are fully allocated.
- P05/P08/P09/P10 handoff evidence is explicit without premature closure.
- any new candidate meets the `CRR-*` evidence contract or absence is stated with evidence.
- production/history/user dirty state remains unchanged.
- completion audit is `Complete / High`.
- user policy decisions, if required, prevent affected completion.

## Completion Audit Evidence Matrix

| ID | Condition | Evidence | Status |
| --- | --- | --- | --- |
| P01-C01 | CRA-P01-C01 full current disposition | candidate reaudit | pass |
| P01-C02 | CRA-P01-C02 full current disposition | candidate reaudit | pass |
| P01-C03 | CRA-P01-C03 full current disposition | candidate reaudit | pass |
| P01-C04 | Boss publication impact and cross-Phase handoff complete | boundary audit | pass |
| P01-C05 | test families have evidence-backed disposition/safety net | test reaudit | pass |
| P01-C06 | new candidate decision is evidence-based | candidate/boundary audit | pass |
| P01-C07 | dirty/production/history preservation proven | hash/status comparison | pass |
| P01-C08 | independent audit is Complete / High | completion audit | pass |

## Stop Conditions

- current schema/content/compatibility policy is needed to disposition a row
- required consumer or source is inaccessible and material to completion
- production mutation/historical rewrite is needed
- P00 baseline drift cannot be explained or user changes overlap authorized audit artifacts
- original candidate full scope cannot be traced
