# Experiment Notes

## Policy Notes

- Do not keep obsolete policy alive through ignored tests.
- Prefer tests around visible behavior, DTO shape, data validation, live RON loading, and gameplay flows.
- This subgoal owns the final generated-docs-versus-code cross-check.
- Live skill catalog delivery/coverage expectations now live in `docs/audit/live_skill_catalog_manifest.ron`; Rust tests are consumers, not the manifest source.
- Timeline/log normal validation now accepts only `TIMELINE_VERSION`; legacy versions `6`/`7` require an explicit future migration path if ever needed.
- `POLICY_COVERAGE.md` is the final generated-docs-versus-policy cross-check for this master run.

## Follow-Up Candidates

- None yet.
