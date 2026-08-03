# Battle Setup Catalog Ref Correction Notes

Status: complete

## Starting Evidence

- The reaudit live smoke observed `abnormality_ids: [""]` in a CorrodedEmployee-only encounter.
- Producer filters wave entries but copied every briefing `abnormality_id` without checking kind or
  emptiness.
- Existing test rejected profile ID leakage but did not assert the whole list was clean.

## Resume Point

- Child complete. Resume from Master G03.

## Final Implementation

- `ActiveBattleSession::battle_setup_snapshot_dto` owns the correction.
- All three candidate sources use the same non-empty, catalog-resolved predicate.
- `enemy_briefing` and `spawn_waves` also require `EnemyKind::Abnormality`; corroded profile
  identity remains exclusively in its typed wave/unit source fields.
- No server mapping, DTO schema, message order, Unity implementation or live RON changed.

## Regression Matrix

| Shape | Encounter | Expected refs |
| --- | --- | --- |
| actual | `suppress_warm_hearted_woodsman` | exactly `f-05-32_warm_hearted_woodsman` |
| mixed actual + corroded | `suppress_clouded_monk` | exactly `d-01-110_clouded_monk` |
| corroded-only | `normal_corroded_patrol` | empty |

## Validation Summary

- Three exact setup regressions passed.
- Core combat module: 60 passed.
- game_server: 30 passed.
- Registered live WebSocket smoke passed and reported corroded-only
  `battle_setup_summary.catalog_refs.abnormality_ids: []`.
