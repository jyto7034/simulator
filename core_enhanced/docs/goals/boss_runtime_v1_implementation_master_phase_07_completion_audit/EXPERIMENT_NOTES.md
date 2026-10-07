# Boss Runtime v1 Phase 07 — Notes

## Audit boundary

- This child owns final evidence and verdict only. It cannot repair production findings.
- Current runtime/live data are authoritative; completed Phase documents point to evidence but do not override it.
- The final audit must keep three states separate: actual behavior, current canonical and approved BP target.

## Required final inputs

- Master required documents and BP-01~BP-34 policy record
- Readiness coverage/report artifacts and P00~P05 completion audits
- Final Boss/runtime/data/gameplay/target canonicals
- Current production Boss/PvE RON, loader/setup/runtime and public DTO/server path
- Current external Unity contract/renderer evidence and exact dirty baseline
- Final focused/broad test results and all failed/corrected-run records

## Known explicit boundary

- Little Red is one replaceable live integration reference. Final lore, content mechanics, art/VFX/bounty and the
  three future final bosses remain outside v1 completion.
- Active Boss graph state remains in-memory session authority; reconnect is projection, not durable cross-process
  save restore.
- P06 is not implemented. Its absence passes only if current v1 truly needs no public lifecycle semantic.

## Final policy checkpoint

- BP-01~BP-34 are represented exactly once in `FINAL_POLICY_COVERAGE_MATRIX.md`; all 34 rows pass.
- Strict-reject boundaries are part of the result: no objective repath alias, future selector fallback, arbitrary
  parameter override, partial catalog publication, hidden retarget/grace or public graph leakage.
- Runtime ownership remains layered: encounter/actor orchestration in private Core, existing movement/combat systems
  as physical/action leaves, and existing event/checkpoint projection as the v1 public boundary.

## Refactor checkpoint

- Completed prerequisites: CRA-P06-C01/C02/C03 and live gates CRA-P01-C02/P05-C02.
- Completed coimplementation: CRA-P01-C01 and CRA-P05-C04.
- Deferred/conditional without v1 claim: CRA-P07-C03/C04, CRA-P09-C05 and first-public-wire CRA-P10-C03.
- Independent without v1 claim: CRA-P07-C01/C02, CRA-P08-C01/C05, CRA-P09-C03/C06, CRA-P10-C02/C04.
- The post-slice cleanup trigger was evaluated, but cleanup is not a correctness/completion gate and P07 cannot
  authorize it.

## Final consumer checkpoint

- Actual live reference semantics are fully expressed by existing spawn/movement/skill/damage/death events and
  checkpoint projection. Private scheduler wake/graph state is not serialized.
- Current server and accessible Unity have no Boss runtime consumer. The three external evidence owners are unchanged
  even though the wider external dirty tree evolved concurrently before P07.
- P06 therefore remains explicitly `not_applicable_to_v1`; it is a future decision point, not completed work.

## Final verdict checkpoint

- P07: `Complete / High`.
- Master: `Complete / High`.
- Product correction: none required.
- Remaining work is outside this Master and needs a separately authorized content/public-lifecycle/refactor Goal.
