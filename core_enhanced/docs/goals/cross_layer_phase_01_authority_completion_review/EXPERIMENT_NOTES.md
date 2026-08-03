# Cross-Layer Phase 01 Authority Completion Review Notes

## Review Boundary

- R01 reviews G01/G06/G07/G12; it does not extend their contracts.
- Runtime/live data/Unity evidence outranks goal documentation.
- Child reviews are inputs, not proof.
- No Phase 2 implementation begins before R01 passes.
- Broad defects discovered here become explicit immediate correction candidates; they are not hidden
  by changing review wording.

## Required Cross-Component Questions

- Does every complete battle projection commit revision, capability, and eligibility together?
- Can any mutation execute without exact UUID/revision or with a different validator from projection?
- Can server ordering/correlation or reconnect expose a stale projection without typed resync?
- Can Unity recreate an absent action, payload eligibility, target/range, or revision locally?
- Can blocking VN hide query/setup-loss recovery or expose unrelated gameplay mutation?
- Can any production startup/runtime path obtain gameplay data outside the strict validated bundle?
- Do canonical runtime/data/transport documents still describe removed fields or fallback behavior?

## Known Inputs To Verify

- G01: Complete / High, battle revision authority.
- G06: Complete / High, common capability authority.
- G07: Complete / High, battle eligibility/reconnect/stale authority.
- G12: Complete / High, exact-version strict live-data authority.
- Unity MCP instance was available and ready during G12; availability must be rechecked for R01.

## Follow-Up Candidates

None yet. Record only evidence-derived candidates during review.

## Source Trace Decisions

- A private candidate database allocated before generated preview validation is not publication; the
  only observable authority boundary is successful loader return.
- Unity's defensive resync request on a stale-authority error is redundant with the server's ordered
  proactive resync but remains a read-only idempotent recovery path, not a second mutation authority.
- Initial deployment highlighting may filter Core-projected deployment cells for presentation, but
  live confirmation requires the command capability and a matching Core deployment preview with
  enabled eligibility; it cannot authorize gameplay locally.
- Historical goal/audit files may describe superseded behavior. Canonical root/server/Unity contract
  documents must not.
- The Master registry must distinguish DCA-M-19 shared seam completion from G24's remaining concrete
  inventory preview/eligibility work.

## Historical Blocking Findings — Resolved 2026-07-14

- R01-H-01: battle capability projection and Core execution do not share a context evaluator. Blocking
  VN hides mutations from honest Unity but direct current-authority commands can reach handlers.
- R01-H-02: server recovery batches stale authority but not missing authority.
- These are technical gaps under accepted policy, not new gameplay/UX questions.
- Broad green tests do not close untested authority paths; R01 cannot be Complete until a correction
  goal adds behavior tests and passes review.

Resolution:

- G30 added the shared Core battle-context evaluator/preflight and direct-command coverage.
- G30 added the shared missing/stale server recovery builder and paired server/Unity coverage.
- G30 and the R01 correction rerun both passed Complete / High with no immediate correction.

## Non-Blocking Follow-Up

- Deep deploy atomicity: `BattleCore::deploy_player_unit` appends the scenario group before fallible
  spawn work. Current validated callers make later failure remote, but a future staging/rollback
  refactor would make the transaction structurally complete.
