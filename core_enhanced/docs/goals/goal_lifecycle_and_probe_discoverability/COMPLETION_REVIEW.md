# Goal Lifecycle And Probe Discoverability Completion Review

Goal: G25

Date: 2026-07-14

Verdict: Complete

Long-term fit: High

Immediate correction: none

## Outcome

The project now has one durable ordered lifecycle from active child through child completion,
independent phase review, named correction plus same-review rerun, next-phase activation and final
master review. Completed goals are explicitly evidence/history rather than live policy sources.

`docs/probe_registry.md` now routes every current external Unity WebSocket/admin probe by exact path,
purpose, required server gate/environment and adjacent contract. The workflow no longer contains a
hypothetical probe-path placeholder.

## Evidence

- `docs/engineering_workflow.md` owns lifecycle state meaning and review order.
- `docs/README.md` owns document hierarchy, current-work discovery and canonical routing.
- `docs/goal_completion_review_guide.md` remains the detailed independent audit procedure and does
  not claim gameplay policy ownership.
- All three registered probe scripts and all three adjacent probe contracts exist. The scripts parse
  as Python and their declared environment/defaults agree with the registry.
- The PD-01..PD-18 audit routes implemented policy to gameplay, runtime, loading, targeting and
  external Unity canonical contracts. PD-05/PD-14 remain user-deferred; PD-17 remains complete
  deletion rather than a replacement policy.
- The older similarly named `completed_goal_document_retirement` proposal is marked
  `historical_unexecuted`; it is not confused with G25.
- The master finding, child and phase registries agree: all active-scope implementation children and
  R00..R06 are complete, G05/G22 retain explicit deferred triggers, and R07 is the only next review.

## Validation

- Registered path audit: 8/8 script, adjacent-contract and external-canonical paths exist.
- Probe parse audit: 3/3 scripts parsed without writing cache files.
- Current durable-doc stale placeholder/status search: clean.
- Scoped documentation `git diff --check`: passed.
- Core `CARGO_TARGET_DIR=/tmp/core_g09_target cargo check --all-targets`: passed.
- game_server `CARGO_TARGET_DIR=/tmp/core_g09_target cargo check --all-targets`: passed.

## Legacy And Ownership Review

No implemented active-scope policy is owned only by a goal directory. Historical goals may preserve
superseded observations, but current work must be discovered from the master/status headers and
current policy from the nearest durable canonical document. No compatibility layer, runtime change,
data change or external DTO change was introduced by G25.

## Remaining Risk

Only filesystem/path movement can stale the probe registry; its rule requires updating the registry
when a probe moves. Runtime behavior remains covered by the completed phase evidence and is reviewed
again independently in R07.
