# Evidence Matrix

- 기준선: `B-001`
- 상태: 최종
- 판정 의미: 기준 문서의 `PASS`, `PARTIAL`, `FAIL`, `UNKNOWN`, `N/A`를 그대로 사용한다.

| criterion | 현재 판정 | evidence | 확인된 범위 | 남은 범위 |
| --- | --- | --- | --- | --- |
| C00 | FAIL | B-001, E-002/E-003/E-004/E-005/E-019/E-020/E-022 | reproducible dirty identifier/toolchain, 1131/1131 tests and fmt pass; clippy -D warnings fails; final hash unchanged | verdict established |
| C01 | FAIL | E-006/E-008/E-020, A-001~A-005, RUNTIME-AUDIT-001/003 | single root owners generally clear, but public currency/BattleCore authority bypasses | verdict established |
| C02 | FAIL | E-006/E-008/E-009/E-010, T-001~T-003, RUNTIME-AUDIT-001/003/004/005/006 | exhaustive gates and staged positive paths, but invalid public state and failed-command mutations exist | verdict established |
| C03 | FAIL | E-007/E-009/E-010/E-011/E-019, T-001~T-003, RUNTIME-AUDIT-005/006 | shop/node/reward/battle staging counterexamples; six live commands and invalid-data start violate failed-command equality | verdict established |
| C04 | FAIL | E-008/E-017, DETERMINISM_INVENTORY, RUNTIME-AUDIT-003/004 | required A/B and insertion probes pass; seeded RNG/integer time/ordering strong, but public non-finite/event authority breaks comparator precondition | cross-process/platform Rapier remains unproven but not needed for fail |
| C05 | FAIL | E-003/E-015/E-016/E-020, RUNTIME-AUDIT-008/009 | compiler-enforced routing and no battle→world import, but battle→behavior error/capability coupling and mixed protocol/domain change center | verdict established |
| C06 | FAIL | E-006, E-009, E-015/E-016, RUNTIME-AUDIT-001/005/008 | central typed adapter/error code and DTO counterexamples, but unversioned domain payload coupling and cursor/projection atomicity violation | verdict established; Unity consumer intentionally excluded |
| C07 | FAIL | E-010/E-013/E-014/E-019, DATA_PUBLICATION_MAP, LIVE_DATA_INVENTORY, RUNTIME-AUDIT-006/007 | exact official typed live path and bytes vs public panic/unvalidated publication path | verdict established |
| C08 | PARTIAL | E-011/E-012, PERSISTENCE_MAP, GAP-003 | strict V4 core validation/restore plus checksummed atomic locked current/backup store and actor rollback | adjacent tests unexecuted; broad continuation equivalence |
| C09 | PASS | E-003/E-006/E-015/E-016/E-019/E-020, S-003/S-005 | exhaustive player/host/admin routing and 47-result projection; no production NotImplemented path | future/old Unity compatibility intentionally excluded |
| C10 | FAIL | E-007/E-009/E-010/E-018/E-019, RUNTIME-AUDIT-005/007/009 | typed errors and non-authoritative trace I/O positive; post-mutation error, public data panic and unbounded log path negative | verdict established |
| C11 | FAIL | E-002/E-009/E-010/E-017/E-018/E-021, TEST_TRACEABILITY, GAP-001~004 | broad non-ignored suite and strong determinism/data/checkpoint tests; S1 atomicity and other risk-specific regressions missing | verdict established |
| C12 | FAIL | E-008/E-018/E-021, RUNTIME-AUDIT-009, GAP-004 | append-only unbounded history; actual constant-tail probe scales ~10× per 10× N and full clone bytes scale with N | product budget/permanent guard requires remediation |
| C13 | FAIL | E-005/E-007/E-015/E-018/E-020/E-021, RUNTIME-AUDIT-001/003~009, RC-01~RC-03 | fixture/production type conflation, per-handler transaction adoption, mixed protocol/history owners and measured cost | verdict established |

## Final verdict totals

- PASS: 1 (`C09`)
- PARTIAL: 1 (`C08`)
- FAIL: 12 (`C00`~`C07`, `C10`~`C13`)
- UNKNOWN/N/A: 0
