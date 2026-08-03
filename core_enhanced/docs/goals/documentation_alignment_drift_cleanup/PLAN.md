# Documentation Alignment Drift Cleanup

Status: complete

Implementation complete: true

Started: 2026-07-18

Master: `docs/goals/documentation_code_alignment_corrections/PLAN.md` G07

Findings: `DCA2-DATA-001`, `DCA2-DOC-001`, `DCA2-WIRE-002`

## Objective

Correct the final unambiguous current-document drift against the post-G04/G06 code and loader:
gameplay source count, historical goal labels used as domain vocabulary, and external enum
inventories/display labels.

## In Scope

- Change the durable gameplay RON source count from stale 23 to actual loader/test count 21.
- Replace current-domain `G15`/`G24` labels with descriptive domain terms while preserving genuine
  historical goal records.
- Make external Unity current `MapNodeCategory` and `TargetingProfile` inventories match Core and
  Unity display labels after prior children.
- Exact source-count/enum/search evidence and Markdown/diff validation.

## Out Of Scope

- Production code, live RON, tests or gameplay policy changes.
- Historical goal/audit log rewriting.
- Excluded duplicate `docs - 복사본` tree.
- P1 Unity presentation implementation.

## Implementation Plan

1. Recount loader sources and enumerate final Core enums/Unity label mappings.
2. Locate only current durable G15/G24 vocabulary and classify history separately.
3. Apply minimal Core/external document corrections.
4. Run exact stale/current inventory searches and diff checks.

## Completion Conditions

- Durable source count matches loader and executable test: 21.
- Current canonical uses domain vocabulary, not G15/G24 labels.
- External current enum inventories exactly match current Core; removed values are absent.
- Historical records and excluded duplicate tree remain untouched.

## Stop Conditions

- Loader/source count or enum inventories are ambiguous after direct enumeration.
- A target reference is historical rather than current-domain vocabulary.

## Completion Summary

- Durable data-loading canonical now matches the executable loader/test count of 21 non-manifest
  gameplay RON sources.
- Current runtime canonical uses descriptive transaction/stack terms instead of historical `G15` and
  `G24` labels; historical goal records were not rewritten.
- External current inventories now list all ten `MapNodeCategory` values and exactly the four current
  `TargetingProfile` values; removed `SplashClusterFirst` wording is absent.
- No production code, live RON or test was changed by this child.
