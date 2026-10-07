# P01 Preservation and Observability Current Notes

## Current Summary

- Current pass/result: `GEX-10 exit conditions met`; GEX-00 passed; fresh GEX-20 pending.
- Current basis: `HA-CRAR-P2G60-002` / `GS-CRAR-P2G50-002`.
- Exact target: `GOAL-CRAR-C2-P01-PRESERVATION-OBSERVABILITY` / `WP-CRAR-C2-01`.
- Direct predecessor: none.
- Scope: `core_enhanced` only. `game_server`, Unity and external consumer evidence are excluded from read, mutation,
  validation, currentness and completion claims.
- The historical `001` manifest/tool/baseline is candidate input only; current evidence must be freshly rebound.

## Current Evidence Gap

- Upstream `GAP-CRAR-P2G20-002` identified the missing durable direct retained heap/allocation observation.
- The historical candidate receipt was absent and its tool recorded only latency/encoded size.
- Current disposition: the v2 receipt and verifier close P01's pre-change observation seam. Final like-for-like guard and
  performance PASS remain explicitly open for P08 ownership, not as a P01 completion gap.

## Binding Decisions

- No prior validation success is automatically reused.
- No production gameplay behavior or data meaning changes are permitted in P01.
- Existing dirty work remains user-owned and is not reverted or attributed to this Goal.
- Policy meaning or Goal/Phase structure changes cause immediate run-global stop and owner return.
- P01 dirty isolation is currently safe because its allowed mutation surface is the dedicated support/manifest/document
  seam and production algorithm/data changes are forbidden.

## Current Technical Summary

- Stable feature inventory: 57/57, unknown/unmapped/duplicate 0; 42 current locator contents are bound by one
  verifier-emitted composite identity.
- Deterministic current Core oracle: 1,131 tests passed and both representative workloads repeat exact output identity.
- Direct heap/allocation observation: non-zero in every one of ten raw samples and reproducible across a second run for
  allocation/deallocation/reallocation/retained-delta identity.
- P01 introduced no production algorithm/data mutation, test-specific production behavior, fallback, shim or gameplay
  authority.
- Current future risk is intentionally owned by P08: the observation values are not a performance PASS and both final
  absolute latency gates still require final like-for-like measurement/optimization.
- Completion-blocking P01 technical gap, material policy question and external consumer dependency: none.
- Audit target: `ATR-CRAR-C2-P01-GEX10-001`; fresh read-only GEX-20 must independently verify completion.
