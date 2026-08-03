# Goal Lifecycle And Probe Discoverability

Goal: G25

Findings: DCA-M-23, DCA-M-25

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Give maintainers one durable goal lifecycle/review order and one discoverable registry of executable
external probes, then reconcile the master registries and canonical ownership before R07.

## Scope

- Define active, implementation-complete, independently reviewed, correction-required, deferred and
  historical goal states plus the order from child work through phase review and final review.
- Make it explicit that completed goal documents are evidence/history, never the sole owner of live
  gameplay, data or wire policy.
- Publish exact external Unity probe paths, purpose, required server mode/credentials and adjacent
  contract document.
- Replace the workflow's `<available-probe-path>` placeholder with the registry and concrete examples.
- Reconcile DCA/CPR/MSTR/PD/child/phase status tables against completed child/review evidence while
  preserving G05/G22 as user-deferred.
- Audit current canonical documents so no implemented active-scope policy exists only in a goal.

## Non-Goals

- Do not delete historical goal directories or rewrite their observations as current policy.
- Do not change runtime, live data, server or Unity gameplay behavior.
- Do not run probes that require starting external servers unless final evidence reveals a missing
  behavior boundary not covered by the completed executable suites.
- Do not activate G05 or G22.

## Validation

- Link/path audit for every registered probe and canonical document.
- Master registry audit: every active-scope finding and child has one terminal or active disposition.
- Stale-status and source-of-truth phrase searches across current durable docs.
- Documentation diff/whitespace checks and a safety `cargo check --all-targets`.
- Completion review before R07.

## Exit Gate

Complete / High only when lifecycle order and correction rerun semantics are durable, probe paths are
concrete and verified, current policy is owned by canonical documents, master registries agree, and
G05/G22 remain explicitly deferred with their activation triggers.
