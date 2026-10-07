# Core Enhanced Refactoring Audit — Final Catalog Consolidation Notes

Status: complete

## Consolidation rules

- A merged final candidate retains every source ID and the union of preserved invariants/safety nets.
- A shared helper opportunity does not merge distinct failure transactions or domain predicates.
- Final ranking uses reachability, correctness/failure risk, dependency and readiness; known subtotal is supporting
  evidence only and change frequency remains unknown.
- Original catalogs remain the detailed evidence authority. Final rows normalize cross-Phase disposition and link
  back to them rather than copying incomplete summaries as new authority.

## CRA-14 backfill rule

Phase 1~3 families are grouped by production lifecycle: parser/publication, run/map/persistence and persistent
asset/economy transaction. A family is retained when it observes a unique layer; it is strengthened at a confirmed
candidate gap; removal requires exact subsumption. Historical pass counts are not the disposition reason.

## External/policy rule

Unity, public Rust consumers, standalone Core packaging, admin durability, multi-step preview/usefulness and specific
combat semantic questions remain unverified or policy-gated. Final does not resolve them by inference.
