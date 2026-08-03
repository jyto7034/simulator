# Visual Novel Authoring And Local Presentation Notes

Goal: G19

Status: complete

## Fixed Predecessors

- G03: Event is first-class typed panels/employee-check state and has no VN trigger/fallback.
- G12: live gameplay RON is exact-version, deny-unknown, all-or-nothing publication.
- Pre-run/global VN predecessors: top-level VN runtime, trigger priority/repeat/checkpoint ownership,
  and Unity-local cue presentation remain fixed unless this goal proves a direct contradiction.

## Non-Negotiable Decisions

- Trigger `blocking` is the only authoring authority.
- Remove `default_blocking`; do not consume it or alias it.
- Exact live ids must resolve in Unity without a silent placeholder script.
- No new story, branching, asset, balance, trigger, or Event behavior.

## Discovery Questions

- What exact VN, scene, script, speaker, portrait, and background ids are present in the live bundle?
- Which of those ids are missing from the Unity-local catalog or mocks?
- Does the current catalog fallback merely diagnose missing content or silently create playable cues?
- Which existing live-flow fixture can run both Standard and Endless without duplicating setup code?

## Implementation Judgment

- Runtime already used trigger `blocking`; removing the unused definition field was a true authority
  deletion and required no behavior migration or fallback.
- Trigger-reachable run-start definitions are Standard and Endless. Both now have exact Unity-local
  scripts and actual server coverage; dormant unreferenced VN definitions are not treated as active
  trigger presentation and were neither deleted nor given invented story content.
- Missing Unity-local scripts now produce an explicit non-advancing diagnostic overlay. They cannot
  silently display `"..."` and let a player advance an unpresented VN.
- Focused and broad validation are green. Remaining work is the formal completion review, then R03.
