# P08 Final Core Structural Audit

Result: `passed / unresolved count 0`

## Audited Categories

| Category | Current finding | Unresolved |
| --- | --- | ---: |
| manifest legacy/removal targets | all 10 readable/classified; old event-log Vec asserted absent; verifier errors 0 | 0 |
| competing application writer | one legitimate `commit_application`; no second live-root replacement owner | 0 |
| raw domain mutation exposure | raw root/kernel/behavior/result/rollback names absent from public rustdoc; negative probes reject | 0 |
| unbounded active history | authoritative event-log Vec field absent; exact `8,192 / 4 MiB / 180 s` enforcement | 0 |
| result/history lifetime coupling | result accumulator retains no diagnostic log; history API probes pass | 0 |
| V4/legacy writer | V4 serializer/construction count 0; V5 encoder is sole writer | 0 |
| required V4 importer | one explicit typed importer remains under the P06 certification/removal gate | 0 |
| compatibility fallback | checkpoint/application/history fallback branch count 0 | 0 |
| test-dependent production branch | runtime `cfg!(test)`, test env/name/value switch count 0 | 0 |
| benchmark-dependent production branch | benchmark/workload/allocator/threshold runtime switch count 0 | 0 |
| fixture capability leakage | default release E0599 and rustdoc method count 0 | 0 |

`LiveBattleEventDeltaDto.events: Vec<_>` is a bounded response value produced after cursor validation; it is not an
authoritative retained history. Internal `ActiveRunTransactionRollback` is touched active-run rollback state, not a
second application writer. `ActiveRunCorePayloadV4` is deserialized only by the explicit certified importer and is not
misclassified as a compatibility fallback.

The audit covers Core source/API/docs/tooling only. No external tree or consumer was searched.
