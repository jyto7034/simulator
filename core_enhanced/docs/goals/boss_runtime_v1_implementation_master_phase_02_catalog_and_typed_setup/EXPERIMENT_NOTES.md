# Boss Runtime v1 Phase 02 — Notes

## Activation

- P01 correctness substrate is complete and remains a prerequisite, not P02 ownership.
- P02 owns CRA-P01-C01 and CRA-P05-C04 only.
- Boss Runtime is still the main track; these two refactors are coimplemented because the new Boss source/setup seam
  would otherwise amplify their existing duplication.
- The live Boss catalog may be structurally empty. Official encounter identity/content and public presentation remain
  deferred to P05/P06.
- Runtime scheduling, targeting and movement execution begin only in P03.
- Evidence is written after each bounded discovery/test/implementation unit so context compaction cannot erase it.

## Initial design constraints

- Separate physical/logical data ownership is not runtime fault isolation; every cross-catalog reference must validate
  before one candidate `Arc<GameDataBase>` is returned.
- The compiler produces immutable resolved IDs and typed argument frames. Runtime lookup by raw string or generic
  RON value is prohibited.
- Only implemented closed variants are accepted. Future `RepathToProtectedObjective`, generic scripts, deep override,
  arbitrary expression and unsupported mechanics must fail publication.
- No-binding parity is a first-class acceptance condition, not an inferred consequence of `Option::None`.
- P02 must not introduce public graph/checkpoint fields merely to make later runtime implementation convenient.

## Exit conclusions

- The source inventory and combat-setup aggregate refactors are complete only to the P02 seam. This does not authorize
  or imply completion of the broader refactor audit catalog.
- The compiled encounter definition and per-call argument frames are retained by the resolved scenario after the
  database handle is dropped. P03 must consume those values directly and must not reopen raw Boss authoring data.
- The production Boss source is deliberately empty, so P02 changes publication capability without changing live
  gameplay. First official opt-in remains a P05 concern.
- Runtime-only policy obligations—checked deadline admission, activation epochs, arbitration, microstep rollback,
  target/path evaluation and Task-result delivery—remain explicit P03 exit requirements, not silent P02 claims.
- All P02 focused and broad gates are green and the exact dirty delta contains only planned P02 ownership. The Phase
  is eligible for `Complete / High` and `ready_for_p03` after the independent completion artifact is written.
