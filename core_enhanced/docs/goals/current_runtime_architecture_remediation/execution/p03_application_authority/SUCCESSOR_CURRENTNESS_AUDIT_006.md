# P03 Successor Currentness Audit 006

Verdict: `Complete / High — current / expected P08 COW staging optimization`

## Identity and Trigger

- Record: `ATR-CRAR-C2-P03-GEX20-006`; predecessor record `_005` remains the P07 currentness basis.
- P08 keeps the one `validate → clone draft → prepare → commit → publish` owner and changes diagnostic event entries to
  copy-on-write storage so root staging does not copy retained history.

## Direct Current Evidence

- `src/game/world/application.rs` remains `7d9c8f70…20df1`; there is still one `commit_application` live-root
  replacement port.
- The new event-log test proves O(1) sharing before mutation, isolation after mutation and unchanged original JSON.
- Public rejected-command staging preserves the exact Run snapshot at 128 and 8,192 retained events. Both allocation
  sets are exactly 675 allocations, zero reallocations, 84,409 peak bytes and zero retained bytes.
- Fresh release application selector passes 9/9; full Core passes 1,159/1,159; the authority invalid probe remains
  compile-fail.

## Disposition

P03 remains `Complete / High`. Commit order, error-before-commit meaning, retry equality and the sole writer are
unchanged; P08 changes only the cost of cloning diagnostic state.
