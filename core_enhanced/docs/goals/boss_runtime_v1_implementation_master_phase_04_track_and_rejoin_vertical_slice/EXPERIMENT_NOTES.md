# Boss Runtime v1 Phase 04 — Notes

## Activation

- Boss Runtime remains the main track. P04 is an integration-proof Phase, not official content publication and not a
  general refactor/fixture-framework campaign.
- P02 authoring/setup and P03 runtime are audited inputs. Corrections found by the vertical slice must be recorded as
  P04 findings and remain inside approved semantics; their prior completion claims are not blindly reused as proof.
- The paired fixture differs only by explicit Boss binding/setup. Ordinary route/combat and public projection remain
  the comparison authority for the no-binding side.
- Production Boss RON remains empty. Fixture data must stay under tests or existing test builders and may not acquire
  a production-only special-case branch.
- The external Unity tree is a stale read-only consumer baseline. Because P04 forbids Wire changes, source evidence is
  sufficient to classify existing movement/damage projection; no Unity mutation or current-parity claim is allowed.
- Findings, failed runs and corrected results are written immediately so conversation compaction cannot erase audit
  evidence.

## Initial risks and discovery questions

- Determine whether the smallest real vertical slice belongs in Core world tests, a game_server contract fixture, or
  a combination; avoid a private-helper-only test that bypasses setup/projection.
- Determine how the current reconnect API exposes the same in-memory active Core without implying durable restore of
  private Boss graph state.
- Determine the normalized comparison boundary for normal/update/setup server transport when the public shape is
  intentionally unchanged.
- Measure whole-Core Boss-boundary clone cost only if the representative fixture makes it observable; performance is
  a watch, not permission for an unrelated scheduler rewrite.
- Any need for new public lifecycle identity, checkpoint field or Unity behavior is a P04 stop condition and P06
  decision input, not an implicit implementation task.
