# Implementation Sequence Audit

이 문서는 `WORK_PACKAGE_MAP.md`의 package만 graph node로 사용한다. A → B는 A completion audit가
통과해야 B를 안전하게 활성화할 수 있다는 뜻이다. Open policy/bootstrap input은 package node가 아니라
activation gate다. Final content authoring은 이 runtime sequence 밖의 별도 Goal이며 bootstrap gate와
혼동하지 않는다.

## Sequence Summary

| Sequence | Package | Prerequisites/edge | Coimplementation | Activation gate | Completion gate | Parallel safety | Intermediate-state safety |
| ---: | --- | --- | --- | --- | --- | --- | --- |
| 1 | `GRB-WP-002` | graph prerequisite 없음 | initial live values+invariant validation+tests | baseline/current live loader accessible | 150/200 initial live load and regression pass | WP001과 semantic independent이나 shared RON 때문에 serialize | existing schema와 consumers unchanged |
| 2 | `GRB-WP-001` | graph prerequisite 없음; Sequence 1 shared-file tie-break | Core/data/V4 save/server/Unity atomic | PD004 bootstrap fixture ready; closed PD001/003/005/008/010 fixtures applied | one ledger, no research/dual path, live WhiteNight bootstrap E2E and completion audit pass | no parallel lane | cutover unit 전체가 runnable/loadable/restorable/consumer-compatible |
| 3 | `GRB-WP-003` | WP001: authority+persistence+producer_consumer | Standard RON/Core/profile store/server/Unity atomic | WP001 complete, PD006 bootstrap tutorial ready; closed PD002 preservation fixtures applied | fixed Standard+durable profile E2E and audit pass | WP004와 shared public boundaries 때문에 serialize | successor profile and Standard are fully usable; no placeholder record |
| 4 | `GRB-WP-004` | WP001: authority+schema_data+persistence | command/planner/result/server/Unity atomic | WP001 complete, sweep content/item contract available | synthetic result invariants and consumer E2E pass | WP003/005와 shared behavior/Unity 때문에 serialize | additive action is accepted and consumed in same release |
| future | `GRB-WP-005` | WP001: persistence+validation if separately activated | each approved content batch vertical | separate future Master request, design/assets/runner available | each batch complete; final placeholder count zero | broad skill/battle/Unity ownership overlap, default serialize | each batch production-valid; no schema-only or missing-asset state |

WP005는 현재 production sequence의 stage가 아니라 exact coverage를 보존하는 `future_or_dormant` node다.
룰북이 이번 correction의 live conversion을 제외했으므로 별도 future Master 요청 전에는 활성화하지
않는다.

## Package Node Registry

| Node | Scope | Registered count | Current graph disposition |
| --- | --- | ---: | --- |
| `GRB-WP-001` | encounter progression/economy V4 | 1 | current, ready for activation preflight |
| `GRB-WP-002` | Monolith exact tuning | 1 | ready |
| `GRB-WP-003` | profile + Standard | 1 | current, prerequisite-gated |
| `GRB-WP-004` | mastered Elite sweep | 1 | current, prerequisite-gated |
| `GRB-WP-005` | 29 skill campaign | 1 | future_or_dormant; outside current sequence |

Node audit: expected 5, observed 5, duplicates 0, omissions 0.

## Full Directed Dependency Graph

```text
GRB-WP-002     GRB-WP-001
                  |   |  \
                  |   |   \ persistence + validation
                  |   |    v
                  |   |  GRB-WP-005 (future; only if separately activated)
                  |   v
                  | GRB-WP-004
                  v
               GRB-WP-003
```

`WP002`에는 directed dependency가 없다. Sequence 1 선택은 semantic edge가 아니라 same-file ownership
conflict를 없애는 deterministic serialization tie-break다.

### Edge Evidence

| Edge | Types | Source/contract evidence | Without predecessor | Release condition |
| --- | --- | --- | --- | --- |
| `WP001 -> WP003` | authority, persistence, producer_consumer | `boss_omen.rs`는 현재 completed-chain ledger가 없고 `world/node_flow.rs`는 victory 뒤 state를 clear한다. WP003 all-FinalBoss record는 WP001의 V4 chain-completion event와 checkpoint identity를 소비해야 한다. | Profile이 PVE catalog/compendium을 독자 재계산하거나 unused placeholder field를 먼저 publish해 false achievements/second migration이 생긴다. | WP001 completion audit에서 V4 FinalBoss event, rollback and no-auto-complete seam이 stable. |
| `WP001 -> WP004` | authority, schema_data, persistence | Sweep eligibility/reward/cursor는 M005/M008 mastered/source ledger를 요구한다. Current command/result가 없고 V3에는 target mastery가 없다. | Fake mastery predicate, duplicate reward dedupe/cursor 또는 다음 V4 cutover에서 잃는 sweep state가 생긴다. | WP001 source identity, mastery, repeat planner and V4 restore contracts pass. |
| `WP001 -> WP005` | persistence, validation | Skill fragment owned/progress/equipped state가 active-run payload에 있고 WP001이 V3→V4 inventory/research boundary와 BossOmen fragment condition validation을 바꾼다. | 29 content batches를 V3 save/old eligibility로 검증한 뒤 즉시 migration/retest하거나 V4에서 equipped skill regressions가 생긴다. | WP001 V4 equipped-fragment replay and catalog/eligibility validation are stable. |

No edge is added between WP003 and WP004: each can semantically consume completed WP001 independently. They are
still serialized because both edit public behavior/snapshot/server/Unity ownership, not because one contract
defines the other.

## Raw Cycles And SCC Disposition

Package clustering 전에 다음 raw finding cycles가 있었다.

| Raw component | Cycle reason | Disposition |
| --- | --- | --- |
| M002 ↔ M008 ↔ M009 | old research removal needs replacement acquisition; new strict source needs old reward/research removal to avoid double authority | coimplemented in `WP001` |
| M005 ↔ M008 ↔ M014 | Elite mastery/reward/failure share victory count, source completion and serialized transition | coimplemented in `WP001` |
| M008 ↔ M010 | NormalBoss cycle resolution and first/repeat reward must commit together | coimplemented in `WP001` |
| M011 ↔ M012 ↔ M013 | eligibility/cycle, active chain and outcome-aware failure are one BossOmen state machine | coimplemented in `WP001` |
| M003 → M002/M008 commit path | Endless completion branch disappears only when result acquisition authority is replaced | coimplemented in `WP001`; no reverse package edge remains |

After contraction, Tarjan-style inspection of package edges yields SCCs `{WP001}`, `{WP002}`, `{WP003}`,
`{WP004}`, `{WP005}` only. Multi-node SCC count is 0. All runtime policy/bootstrap decisions are closed; final
content is intentionally deferred. Closed PD001/003/004/005/008/010 fix cutover/source identity/bootstrap repeat,
live-chain bootstrap, Elite mixed fill and sweep reacquisition and are not hidden policy cycles.

## Topological Layers And Parallel Audit

| Layer | Nodes | Graph basis | Parallel verdict |
| ---: | --- | --- | --- |
| 0 | `WP001`, `WP002` | indegree 0 | **not parallel-safe**: both edit live `run/policy.ron`, `RunPolicyData` validation/tests; serialize WP002 then WP001 |
| 1 | `WP003`, `WP004` | each consumes WP001; future WP005 omitted | **not parallel-safe by default**: shared behavior/snapshot/server/Unity; serialize WP003 then WP004 |

There are no approved parallel lanes. A future implementation owner may prove disjoint ownership after concrete
diff design, but absence of such evidence here does not authorize parallel work.

## Stage 1 — GRB-WP-002 Monolith Initial Stat Tuning

- **Prerequisite/edge:** none. Chosen first only to isolate shared RON/loader edits before the large cutover.
- **Atomic boundary:** live Stage 2/3 initial stat values, invariant validation and derived tuning tests.
- **Activation gate:** initial target 150/200 confirmed; production loader/current test runner accessible; clean ownership
  of the exact live fields established against dirty worktree.
- **Completion result/evidence:** live RON loads at initial 150/200, every applied stat derives from authored RON,
  positive/ordered/safe-range validation holds, reward multipliers and rounding remain unchanged; focused and broad
  combat-difficulty suites plus completion audit pass.
- **Stable seam:** validated Monolith role tuning consumed unchanged by WP001.
- **Parallel/rollback:** no parallel with WP001. Rollback is one strict value/validation package; no save/wire skew.
- **Intermediate safety:** existing Core schema reads the corrected values immediately; no new field or consumer is
  waiting for a later stage.
- **Reevaluation:** later development tuning inside the approved invariant does not reopen gameplay policy; changing
  the invariant or reward relationship does.

## Stage 2 — GRB-WP-001 Encounter Progression And Fragment Economy V4

- **Prerequisite/edge:** graph none; Stage 1 must finish for shared-file serialization.
- **Atomic coimplementation:** RunState ledger/transitions, PVE/BossOmen/reward/run RON+validation, combat/Event/
  node/map selection, V4 active-run/checkpoint, behavior/snapshot/result, server order/persistence handling and Unity
  parsers/presentation; old research/weighting/special paths removed in the same release.
- **Activation gate:** closed `GRB-PD-004` minimum runnable 29-row bootstrap and repeat dust 5/10/20; closed
  `GRB-PD-001/003/005/008/010` strict V3 rejection, authored source-key, one-step WhiteNight live bootstrap,
  unmastered-first mixed fill and sweep reacquisition fixtures plus rollback/reconnect table ready;
  Core/server/Unity repos+runners accessible; atomic deployment approved.
- **Completion result:** one source/cycle authority, exact success/failure/retry/reward semantics, no research or
  Endless auto completion, and V4 restore/reconnect across all consumers.
- **Focused/broad gate:** all class/source outcome matrices including dismantle→reacquire, live invalid-data,
  migration/reject, checkpoint, late-failure atomicity, deterministic/mixed-fill cycle, one-step WhiteNight E2E,
  server/Unity contract, old symbol/schema/live entry zero and broad
  Core/server/Unity suites; independent completion audit.
- **Stable seams:** typed FinalBoss completion candidate for WP003; mastered Elite/source reward/cursor and V4
  equipped state for WP004/WP005.
- **Parallel/rollback:** no parallel lane. Rollback must restore the entire old release and its save compatibility;
  mixed V3/V4 producer-consumer operation is not supported.
- **Intermediate safety:** Stage completes only when code/data/save/server/Unity all use V4. Unused new schema,
  dual acquisition, silent missing-content default and ignored tests are completion failures.
- **Reevaluation:** bootstrap fixture invalidity or inaccessible Unity runner pauses activation. Later final content
  replacement is a separate content Goal and does not reopen this runtime decision unless a policy invariant changes; split only if a
  new proof shows a production-valid independent lifetime, never with placeholder state.

## Stage 3 — GRB-WP-003 Unified Profile And Fixed Standard

- **Prerequisite/edge:** WP001 authority+persistence+producer_consumer completion.
- **Atomic coimplementation:** strict-valid bootstrap Standard RON/generator/difficulty/result, profile type/reducer, V2 store/
  current+backup/reset/retry, host publish ordering, snapshot/capability and Unity menu/records.
- **Activation gate:** WP001 audit passed; closed `GRB-PD-006` minimum `ST1-NT60` bootstrap ready, including F3 boss-only
  Nothing There four-axis exact actor override (`720/53/36/33`, base 60% derivation); closed `GRB-PD-002`
  V1-preservation fixtures ready; atomic
  Core/server/Unity release and host failure protocol available.
- **Completion result:** fixed three-floor Standard, durable tutorial unlock/Endless gate and monotonic permanent
  records including WP001 all-FinalBoss event.
- **Focused/broad gate:** bootstrap map/script and F3 actor-only four-axis override isolation, flag+RunComplete atomicity,
  V1/V2/corrupt/backup/restart, publish failure,
  checkpoint independence, all records, server ordering/Unity menu/read, broad map/save/profile and completion audit.
- **Stable seam:** completed global profile/capability contract; WP004 does not depend on it.
- **Parallel/rollback:** serialize before WP004 due shared public files. Whole V2/Standard release rolls back
  together; generated Standard/V2 partial mix unsupported.
- **Intermediate safety:** profile fields are immediately produced, stored, exposed and consumed; all-boss is based on
  live WP001 event, not dormant placeholder. Endless gate and Unity display agree.
- **Reevaluation:** Bootstrap이 strict-valid/runnable하지 않거나 closed PD002 lifetime과 충돌하거나 profile을
  atomically publish할 수 없으면 preflight로 돌아간다. 별도 Goal의 final content 교체는 system invariant를
  바꾸지 않는 한 PD006을 reopen하지 않는다.

## Stage 4 — GRB-WP-004 Mastered Elite Sweep

- **Prerequisite/edge:** WP001 authority+schema_data+persistence completion.
- **Atomic coimplementation:** Core action/capability/preview/item validation/synthetic planner/result/node commit,
  server mapping and Unity choice/disabled/result presentation.
- **Activation gate:** WP001 audit passed; source/repeat mappings are live; sweep item/duration identity approved;
  producer/consumer repos and runner accessible; public contract atomic release approved.
- **Completion result:** eligible mastered Elite offers direct/sweep and synthetic result performs exactly approved
  reward/XP/duration/cycle mutation without battle/condition/log/bonus/discovery side effects.
- **Focused/broad gate:** eligibility/no-side-effect/atomic failure/checkpoint/reconnect, exact result/server/Unity,
  broad combat/economy/action regression and completion audit.
- **Stable seam:** complete optional gameplay vertical slice; no later package required for it to be usable.
- **Parallel/rollback:** no parallel with WP003/WP005 under current ownership evidence. Additive contract rolls back as
  a whole; server or Unity without the action/result version is unsupported.
- **Intermediate safety:** command admission and consumer ship together; no fake battle or result fallback.
- **Reevaluation:** if no existing item definition satisfies approved sweep duration semantics, create a bounded
  content decision before activation rather than inventing a default.

## Future Node — GRB-WP-005 Dedicated Skill Campaign

- **Prerequisite/edge:** If separately activated, WP001 persistence+validation stable. Closed `GRB-PD-007` keeps
  this node outside the current correction.
- **Atomic coimplementation:** each approved batch includes fragment/skill RON, required Core mechanic, tests and all
  Unity presentation/assets. Final batch includes zero-placeholder cleanup/audit.
- **Activation gate:** explicit separate future Master request; per-skill design/balance/compatibility approved;
  capability audit complete; all required assets and Unity runner available.
- **Completion result:** each batch's skills are fully selectable/usable/presented; final production has no 29 target
  BasicAttackModifier placeholders.
- **Focused/broad gate:** per-design behavior/upgrade/awakening/compatibility, catalog/asset coverage, equipped V4
  replay, Unity tests, broad battle and per-batch/final completion audit.
- **Stable seam:** each batch is current production content, not a future schema. Final seam is zero-placeholder
  catalog.
- **Parallel/rollback:** default serialized because skill/battle/Unity files overlap. A batch rolls back with its
  content/assets together; incomplete IDs or fallback visuals are not a release state.
- **Intermediate safety:** unconverted fragments remain their current valid behavior until their approved batch;
  converted fragments are complete. No batch changes an ID to ActiveSkill before referenced skill/assets exist.
- **Reevaluation:** a design requiring unsupported durable state/public wire gets its own readiness decision and
  may become a difficulty-5 subpackage before production work.

## Final Sequence Verdict

Graph is acyclic after evidence-backed coimplementation, every node is covered once, and each current stage has a
production-valid runnable-bootstrap end state. WP005 is correctly outside the current sequence. PD004/006
bootstrap inputs are closed, while final content authority is separately deferred, so the sequence and runtime
activation inputs are `ready_for_implementation`. This investigation does not itself authorize production changes;
those require a separately approved implementation Goal.
