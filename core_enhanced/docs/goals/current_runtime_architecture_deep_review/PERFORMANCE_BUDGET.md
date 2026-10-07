# Performance And Resource Budget

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Baseline: `BASELINE-20260822-064427-KST`
- Primary evidence: E-020

## Measurement Contract

Environment: WSL2 Linux x86_64, AMD Ryzen 9 7950X 16c/32t, rustc 1.91.1, optimized release, one unpinned process. The
`/tmp/core_arch_perf` package depends on this workspace and ran offline. Values below are local characterization, not a product
SLA. A target implementation must be compared on the same runner, fixture, build and sampling protocol.

## Current Representative Results

| Workload | Samples | Current median | Current p95 | Size/context |
| --- | ---: | ---: | ---: | --- |
| embedded live-data load | 10 | 142.565ms | 144.110ms | 24 gameplay sources + manifest; full validation/setup gate |
| common map query | 2,000 | 3.902us | 4.122us | initialized Standard run |
| run snapshot JSON | 500 | 169.331us | 196.552us | 46,984 bytes |
| nonbattle rollback capture | 2,000 | 3.387us | 3.492us | current small run, no active battle |
| checkpoint export | 500 | 20.694us | 22.621us | 17,137-byte Core payload |
| initial live battle zero tick | 2,000 | 9.130us | 10.200us | current initial event history |
| initial live battle one-ms tick | 200 | 10.390us | 10.848us | no new events in fixture |
| current reconnect tail | 500 | 1.250us | 1.298us | last 10 from tiny history |

## Scaling Results

| Retained event entries | Event-log JSON | `BattleCore` clone p95 | last-10 query p95 | full-delta p95 |
| ---: | ---: | ---: | ---: | ---: |
| 1,000 | 123,809 B | 13.576us | 0.802us | 25.436us |
| 10,000 | 1,257,810 B | 125.243us | 3.940us | 222.856us |
| 100,000 | 12,777,811 B | 1.879ms | 96.916us | 2.741ms |

| Retained units | Initial event entries | clone + staged 1ms p95 |
| ---: | ---: | ---: |
| 10 | 11 | 2.930us |
| 100 | 101 | 20.341us |
| 1,000 | 1,001 | 207.524us |

## Complexity Model

- `BattleEventLog.entries` is an unbounded `Vec`; `entries_after_seq` filters from the beginning and clones matches. A
  constant-size tail is therefore O(total retained entries + returned entries), not O(returned entries).
- Battle advance/tactical safety clones an `ActiveBattleSession`; the probe cloned only `BattleCore`, so its history/entity
  result is a lower bound. Current correctness cost is at least O(retained mutable battle state).
- Full delta serialization is legitimately O(returned events), but clients requesting a recent cursor should not pay for older
  retained events.
- Unit staging showed near-linear retained-entity scaling. This is expected for a full clone, not necessarily for the domain
  simulation work itself.
- Snapshot/checkpoint sizes are currently small. No current event/time/byte cap, allocation measurement or concurrent-session
  budget was found.

## Provisional Non-Regression Gates

The primary gate is current-relative. Because this was not a pinned benchmark runner, an implementation comparison should use
multiple interleaved runs and fail on a statistically stable p95 regression greater than 20%; smaller changes remain
`inconclusive`, not automatic PASS. Payload bytes must not grow unless a policy feature accounts for them.

| Workload | Migration ceiling on same runner | Desired target/headroom | Status |
| --- | --- | --- | --- |
| common query | <= 1.20x current p95 | remain well below 0.1ms | provisional |
| ordinary battle tick/command | <= 1.20x matching current p95 | <= 0.5ms p95 for ordinary fixtures, leaving host/tick headroom | provisional engineering target |
| run snapshot | <= 1.20x 196.552us and <= current bytes for same semantics | <0.5ms and <64KiB for current fixture | provisional |
| checkpoint export | <= 1.20x 22.621us and <= current bytes for same semantics | <0.1ms and <32KiB for current fixture | provisional |
| 100K constant-tail query | never worse than current 96.916us during migration | O(returned entries), desired <=10us p95 and insensitive to retained history | architecture target |
| 100K staged mutation | never worse than current 1.879ms `BattleCore` lower bound | history-independent transaction staging; desired <=0.5ms before simulation | architecture target |
| 1,000-unit staged step | <= 1.20x 207.524us for same synthetic semantics | transaction overhead proportional to touched state, not all retained state | architecture target |

The absolute headroom values are explicitly provisional engineering limits derived after observing that current ordinary work is
orders of magnitude below 1ms. They are not canonical gameplay/server policy.

## Required Target Changes

- Index event sequence to start a delta query directly; separate bounded transport/presentation archive from facts needed for
  final result/debug export.
- Put transaction semantics behind one port, then replace full-session copy with touched-root staging, copy-on-write immutable
  catalogs or a validated mutation journal without weakening rollback.
- Define deterministic retention by event count, battle time or byte size. Compaction must preserve result statistics,
  reconnect window and any authoritative cause identities.
- Add an event-heavy boss/projectile/effect fixture and measure allocation/peak retained bytes before release.

## Verdict And Policy Gap

Provisional A09 verdict: `FAIL`. Current ordinary performance is good, but long-session work/state has neither an explicit bound
nor the required delta-cost shape, and correctness staging scales with retained battle state. This supports
ARCH-REVIEW-002 and a battle-session boundary refactor/rewrite; it does not support replacing the movement, skill or Boss
algorithms. Product hard caps and concurrency/tick SLA remain isolated in GAP-001.
