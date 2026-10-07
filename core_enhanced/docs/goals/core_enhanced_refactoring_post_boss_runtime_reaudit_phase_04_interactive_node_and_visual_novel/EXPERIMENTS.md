# Phase 04 Experiments

Status: complete

Initialized after P03 `Complete / High`; P04 was ready, P05 pending and path absent. No P04 production search or
tests were run before the required documents existed.

## 2026-08-09 — Current reaudit

Re-read the original P04 catalog and traced current trigger validation/runtime key derivation, Event/Shop initial and
helper commits, HQ completion/transaction flow, trigger/Monolith VN construction and Boss intersections. Executed:

```text
cargo test -p game_core --lib visual_novel -- --nocapture                    # 24 pass
cargo test -p game_core --lib game::world::event_node::tests -- --nocapture # 5 pass
cargo test -p game_core --lib game::world::tests::node_sessions -- --nocapture # 18 pass
cargo test -p game_core --lib game::world::tests::facility -- --nocapture   # 11 pass
```

Physical executions: 58 passed, 0 failed, 0 ignored; overlap is retained and nonmatches excluded. All assigned
candidates/contributions remain valid, no new candidate was found and both P00 hashes remained exact. P04 exits
`Complete / High` without production mutation.
