# Live Skill Catalog Evidence Governance

Goal: G20

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Close DCA-M-06, DCA-M-09, and DCA-M-24 by giving the live skill manifest exact non-overclaimed
semantics, a durable update/review lifecycle, and a current derived skill-fragment index.

## Fixed Boundary

- The audit manifest proves exact live skill membership, delivery classification, and actual
  reference ownership only. It does not prove skill behavior correctness.
- Behavior evidence is an assertion in a named focused test. A membership list or smoke event count
  cannot be described as complete behavior coverage.
- Manifest updates accompany the live RON/reference/delivery change that caused them and receive
  review against derived runtime data; schema meaning changes bump manifest version.
- The content index is derived/discoverability documentation and must enumerate current live
  abnormality fragments without becoming schema or balance authority.

## Validation

- Make the manifest schema strict and rename the overclaiming field/test.
- Derive each reference-owner set from loaded abnormality, equipment activation, and active fragment
  data and compare it to the manifest; verify their union equals the live skill catalog.
- Update the durable docs index/workflow and all 30 abnormality-derived fragment rows/counts; derive
  an exact index-membership assertion from loaded fragment origins.
- Run focused manifest, RON loading, formatting, searches, and broad Core/server validation before
  completion review and R05.

## Exit Gate

Complete / High when the manifest's machine-checked meaning is exact, behavior proof cannot be
inferred from membership, ownership/update workflow is durable, the fragment index matches live
data, and all validation gates pass.

Exit gate result: Complete / High on 2026-07-14. See `COMPLETION_REVIEW.md`.
