# P05 Elite Monolith Remanifestation Completion Review

Review date: 2026-07-27

Verdict: **Complete / High**

Implementation complete: true

## Audit Table

| Item | Source of truth | Verdict | Fit |
| --- | --- | --- | --- |
| Natural Elite lifetime | `MonolithRunState`, map encounter assignment | Complete | High |
| Wandering merchant economy | RunPolicy, live Shop RON, Shop transaction | Complete | High |
| Node conversion and attempts | NodeConfirm projection, conversion transaction, combat result | Complete | High |
| Stage reward and fragment | P04 role difficulty, combat reward transaction | Complete | High |
| Residual and exchange | residual ledger, Floor guarantee, exchange VN | Complete | High |
| Persistence and projection | run/checkpoint/active-run/battle-entry/root DTO | Complete | High |
| Unity consumer | typed readers, card selection, explicit commit, VN catalog | Complete | High |
| P06 exclusion | scoped production search and unchanged completion routing | Complete | High |

## 1. Elite And Merchant Boundary

Endless natural Elite identities become permanently unavailable in the current run only after committed victory or
final failure. Assignment also prevents same-Floor duplicates. Exhaustion never resets this hard constraint: an
Elite slot becomes an authored ordinary Combat with the Standard ordinary difficulty band. Standard keeps its own
ordinary exploration Shop; only pooled Endless Shop nodes on human Floor 4+ resolve to the distinct wandering
merchant.

The merchant owns exactly three first-page offers. Zero, one or at least two eligible identities produce zero, one
or two distinct typed monolith offers and fill the remaining slots with repeatable high-value consumables. Both
monolith offers remain buyable. One paid refresh replaces the full page with three item-only offers. Purchase uses
the current offer identity and commits currency, offer removal and the abnormality ledger atomically.

## 2. Conversion, Battle, And Reward

Core projects eligible identities in NodeConfirm. Conversion accepts both node and abnormality identity, revalidates
the projection, preserves topology/category and atomically replaces only the encounter/difficulty content while
consuming one use. An identity active in another room remains a Core-authored disabled card when the panel has other
usable options. The Unity card click is local selection; only the separate commit action sends the command. The
converted preview explicitly projects identity, stage, risk, maximum attempts and reward multiplier.

Remanifestation uses two attempts. First defeat retries the same converted node, second defeat completes it without
reward, and retreat restores the entry snapshot without consuming an attempt. Only victory advances stage. Stage 3
grants the authored unique fragment once. Stage stat and reward scaling compose with the existing P04
`MonolithRemanifestation` role rather than creating another power authority; stage 3 is strictly stronger and more
rewarding than stage 2.

## 3. Residual, Persistence, And Deferred Content

A stage-3 victory with exactly one use left consumes it into an ordered source-preserving residual. Reaching four
sets a one-shot next-Floor Shop guarantee; generation authors exactly one Shop-tag slot and consumes the guarantee
when the Floor commits. The merchant starts a blocking exchange VN before Shop interaction. Accept atomically
consumes the oldest four and records a run-local corrupted-monolith key; decline is irreversible for the run.
Leaving a Floor clears unresolved node-local mappings without refunding committed uses, and the Shop guarantee cannot
be issued again by later residuals in the same run.

All monolith state is present in checkpoint and active-run payloads and in battle-entry rollback. Root projection is
typed and derives Elite pool exhaustion from the live Elite catalog. Checkpoint validation rejects inconsistent
stages, active mappings, ordered residual sources, guarantee/exchange/key states and unresolved catalog references.
The corrupted key intentionally has no
consumer in P05. No hidden-boss actor, skill, encounter, administrator VN, capstone, final-boss reclassification or
completion interception was added; those remain P06 `deferred_by_user`.

## Validation

- Core broad: `cargo test -p game_core --lib` — **786 passed, 0 failed**.
- Live RON: `cargo test -p game_core live_ron --lib -- --nocapture` — **52 passed, 0 failed**.
- Focused monolith/policy/facility tests — **9 passed**; Shop separation, Elite fallback and checkpoint restore each pass.
- Server: `cargo test -p game_server` — **52 passed, 0 failed**; doc tests **0**.
- Unity focused EditMode job `44ed768f0d3b49e1bcc7a9e27c99217d` — **4/4 passed**.
- Unity broad EditMode job `60aa28fc93f742b1b9c0bb4cf3420aab` — **258/258 passed**.
- Unity script validation and compile — errors **0**; Console `error CS` entries **0**.
- Scoped diff checks report no whitespace errors; scoped content search reports no P06 production identifiers.

## Final Verdict

- Primary verdict: **Complete**
- Long-term fit: **High**
- Improvement class: **none required for P05**
- Remaining risk: initial economy/reward coefficients require playtesting, but are live-policy owned and do not
  change the implemented transaction or progression semantics. P06 remains intentionally unavailable until the
  user finalizes boss content and explicitly resumes it.
