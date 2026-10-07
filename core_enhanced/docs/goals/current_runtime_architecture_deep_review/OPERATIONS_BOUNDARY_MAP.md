# Operations, Failure And I/O Boundary Map

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Baseline: `BASELINE-20260822-064427-KST`
- Primary evidence: E-008, E-010, E-015~E-019

## Assessment

Core correctly avoids owning filesystem and network I/O and exposes a usable durable-checkpoint/rollback port. The boundary is
weaker inside the application layer: commit, projection and durable-publication intent are not one uniform use-case protocol.
Consequently a host can implement the selected durable contract, but Core does not make every command's retry/publication
semantics equally explicit.

Provisional A07 verdict: `FAIL` for the current application mutation boundary, with a strong positive Core/host I/O seam. The
failure does not imply that persistence or the battle/domain kernels should be rewritten.

## Responsibility Boundary

| Concern | Current Core responsibility | Selected host responsibility | Assessment |
| --- | --- | --- | --- |
| gameplay authority | validate/execute command and own run/battle/profile state | serialize access to a player/session and send commands | clear at façade, broad inside world root |
| checkpoint content | validate last completed SavePoint; strict V4 payload; restore seed/state; return bytes and monotonic metadata | envelope, SHA-256, current/backup/temp files, lock and fsync | good port; actual host behavior not audited |
| failed durable publish | expose `ActiveRunTransactionRollback`; restore run state while monotonic profile/run peak follows explicit merge rules | capture before command, publish, call restore and suppress success on failure | workable but orchestration is a host convention rather than one commit receipt |
| Standard completion | expose authoritative Core result/state | journal successor profile intent, fsync, profile publish, run tombstone, recovery gate | selected contract is coherent; implementation outside scope |
| battle reconnect | revision, ordered event sequence, checkpoint plus `after_seq` validation | maintain cursor/connection and retry/delivery policy | typed and useful; tail query scans retained history |
| command identity | optional `source_command_id` in selected battle paths; cast/revision identities | transport request identity and duplicate-delivery policy | fragmented outside battle; no core-wide idempotency contract |
| diagnostic trace | tracing calls and non-authoritative combat trace export | collector/sink, retention and alerting | correctly non-authoritative but sparse; no Core metrics port |
| concurrency | `&mut GameCore` implies serialized mutation | actor/lock/scheduling/backpressure | safe single-owner shape, but host assumption should be stated explicitly |

## Failure Semantics

The desired operation sequence is:

`authorize → validate/plan → stage domain effects → build result/projection/publication intent → atomically publish Core state → host durable publish → deliver`

Current code implements pieces of this sequence with `with_state_transaction`, economy stages,
`with_staged_tactical_world_mutation`, battle-advance staging and the durable rollback token. Selected node/result/checkpoint paths
are strong. Other handlers can mutate before a later projection or nested commit fails, as recorded in ARCH-REVIEW-004/005.
Because transaction selection is local to the handler, adding a feature creates a proof obligation that is not visible in the
command type.

## Retry, Reconnect And Delivery

- Battle revision and `after_seq` distinguish stale/future projection cursors from gameplay commands.
- Event/cast IDs and selected committed receipts provide local dedupe, but there is no root-wide command receipt ledger or
  declared at-most-once/at-least-once contract for every behavior.
- A returned Core error should mean no mutation for the declared transaction roots, except explicitly monotonic aggregates.
  Current helpers encode this inconsistently.
- Durable success must be delivered only after the host publication stage succeeds. The current Core port can support this, but
  the host contract and the Core transaction receipt should be made one adapter-facing API.
- Battle logs/checkpoints currently have no explicit backpressure/retention cap; see GAP-001 and PERFORMANCE_BUDGET.md.

## Observation Boundary

The static scan found 44 tracing imports/macro call sites under `src/game`. High-level initialization, state transition,
rewards, combat and selected warnings are logged. Combat trace-export failure is deliberately warned and ignored, with a test
that proves it cannot alter result transition. No metrics/histogram API is present in Core.

Recommended shape: Core emits typed operation outcomes and optional diagnostic events; the host owns logs/metrics/storage.
Metrics should observe command class, outcome, latency, retained event bytes and rollback/durable-publish failure without becoming
gameplay authority.

## Target Ports

- `QueryPort`: cannot acquire a mutation transaction.
- `RunMutationPort`, `BattleMutationPort`, `ProfileMutationPort`: each supplies the correct staged authority and monotonic rules.
- `CommitReceipt`: contains observable result/projection plus optional durable publication intent; state is published only after
  all Core-fallible work succeeds.
- `ActiveRunPersistencePort` adapter contract: host-owned lock/checksum/fsync/backup/journal/tombstone protocol, tested outside
  Core against the same receipt/rollback semantics.
- `ObservationSink`: non-authoritative typed diagnostics with bounded payloads.

These ports can be introduced behind `GameCore::execute*`; they are a boundary rewrite, not a reason to move filesystem,
network or concurrency concerns into domain code.
