# Safe Zone Research Delivery Policy

Goal: G21

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Implement accepted PD-13 by removing category-specific research delivery on node entry and making
the authoritative Core transition from resolved node content to map/planning the sole automatic
delivery boundary presented by Unity as Safe Zone.

Primary finding/policy: DCA-M-07, accepted PD-13.

## Required Resume Read Set

- `../cross_layer_contract_hardening_master/STATUS.md`
- active Phase 4 `PLAN.md` and `EXPERIMENT_NOTES.md`
- this file, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`
- G29 and G15 `COMPLETION_REVIEW.md`
- Phase 3 R03 conclusion and PD-13 in the Master plan
- current research queue/delivery state, every delivery caller, node/Event/combat/Gate completion
  seam, RunFailed transition, result DTO, server pass-through, and Unity notification consumer

## Fixed Decision

- Research completion queues pending results during node activity; it never grants on node entry.
- Remove the node-category allowlist and `deliver_research_if_safe_node`-style entry hook.
- Deliver all pending results only when Core commits the authoritative return from resolved node
  content to map/planning state presented as Safe Zone.
- Delivery and pending removal are atomic and idempotent, and the authoritative transition result
  carries the delivery facts needed for notification.
- Reconnect, snapshot rendering, repeated Safe Zone presentation, and Unity-local screen entry are
  read-only and cannot grant or duplicate delivery.
- Event-started combat and nested content deliver only after the owning node resolves. Gate delivers
  on arrival at the new floor's planning boundary. RunFailed never fabricates a return or delivery.

## Validation

- Exact queue-without-entry-delivery and once-only planning-return delivery tests.
- Matrix for ordinary node, Support/Maintenance, Headquarters/Shop, Reward, Event, Event-started
  combat, combat victory/final defeat/retry/retreat, Gate/floor advance, RunComplete, and RunFailed.
- Atomic failure/no-partial-grant and reconnect/repeated-snapshot idempotence tests.
- Result DTO/server ordering/Unity notification tests if the external delivery fact changes.
- Core focused and broad suites, relevant server/Unity suites, formatting, scoped diff, old
  allowlist/entry hook/fallback/ignored-test searches, then completion review before R04.

## Exit Gate

Complete / High when one Core-owned planning-return transaction is the only delivery authority,
every applicable return path uses it exactly once, non-return paths cannot grant, and no
category/Unity-local compatibility path remains.

Result: Complete / High / no immediate correction. See `COMPLETION_REVIEW.md`.
