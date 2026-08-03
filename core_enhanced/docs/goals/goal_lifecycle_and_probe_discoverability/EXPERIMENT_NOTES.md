# Goal Lifecycle And Probe Discoverability Notes

Goal: G25

Status: complete

## Ownership

- `docs/README.md`: document hierarchy and navigation.
- `docs/engineering_workflow.md`: working lifecycle and validation order.
- `docs/goal_completion_review_guide.md`: detailed independent review procedure.
- New probe registry: routing only; exact payload/behavior remains in external Unity contracts.
- Master PLAN: active campaign finding/child/phase disposition.

## Guardrails

- Historical goal docs can retain superseded statements when clearly classified as history; bulk
  rewrite/deletion is not required for current-policy correctness.
- A completed child is not a passed phase review. A correction review must rerun independently after
  its correction child closes.
- Probe registry entries must be real files, not hypothetical commands or copied probe code.
- Do not broaden G25 into gameplay, schema, content or balance changes.

## Decisions

- The campaign does not bulk-delete completed goals. They remain historical evidence unless a
  separately authorized cleanup changes that retention policy.
- Master child/status tables and the child status header own current activity; durable domain and
  wire documents own current policy.
- Rxx reviews are independent gates, not aliases for child completion. A material correction always
  requires a named correction child and an independent rerun of the same review.
- `docs/probe_registry.md` is routing authority only. Probe code is an executable consumer and the
  external Unity contracts remain wire authority.
- PD-17's complete absence is intentional canonical state. Documenting a replacement merely to give
  the removed subsystem a policy section would violate its accepted deletion boundary.
