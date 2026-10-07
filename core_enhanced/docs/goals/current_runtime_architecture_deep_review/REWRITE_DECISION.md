# Runtime Rewrite Decision

- Goal: `current_runtime_architecture_deep_review`
- Goal status: `complete`
- Baseline: `BASELINE-20260822-064427-KST`
- Decision date: `2026-08-22`
- Primary evidence: E-001~E-024

## Decision

Do not perform an entire `core_enhanced` big-bang rewrite.

Perform a controlled staged partial rewrite of the application/state/transaction shell and external protocol boundary, plus a
bounded rewrite of the active-battle session/history and validated-data construction boundaries. Retain and progressively
encapsulate the gameplay/domain engines, data validation, determinism, checkpoint validation and test corpus.

This is not a recommendation to keep the present architecture. Current Architecture Assessment is `FAIL`: A02, A03, A05,
A07 and A09 fail. The distinction is that the failures concentrate in separable boundaries while the costly gameplay meaning
and its oracles remain valuable.

## Decision Summary

```text
Goal Status: complete
Current Architecture Assessment: FAIL
Requirement Preservation Confidence: High for routed Rulebook/current semantics; open future values isolated
Incremental Refactor Viability: High as enabling work, insufficient alone
Partial Rewrite Viability: High and recommended
Full Rewrite Justification: Not met
Recommended Target Architecture: retained domain engines behind a new typed application/transaction/protocol shell
Migration Feasibility: High with route-level single authority and differential gates
```

## Subsystem Disposition

| Subsystem/responsibility | Current value/pressure | Disposition | Target |
| --- | --- | --- | --- |
| `GameCore` external façade and behavior semantics | single typed host entry and strong tests; current DTO layout broad | `RETAIN` semantics | stable compatibility façade delegating to use-case registry/versioned adapters |
| world command routing/transaction | central exhaustive routing; handler-local transaction/projection policy | `REWRITE_BOUNDARY` | compiler-classified Query/Run/Battle/Profile mutation and prepared commit receipt |
| root session/run/node state | correct engines behind parallel phase/session options and hidden `Deref` owner | `REWRITE_BOUNDARY` | typed `SessionState`, `RunPhase`, `ActiveNodeSession`, explicit Run mutation ports |
| protocol/result/error/snapshot | purpose-built projections but centralized/selectively versioned | `REWRITE_BOUNDARY` | versioned capability adapters outside domain/application models |
| active-battle application session | strong revision/staging/reconnect semantics; whole clone and inconsistent direct handlers | `REWRITE_BOUNDARY` | one battle mutation port, touched-state stage and direct command delta |
| battle event presentation/archive | ordered semantic log; unbounded multi-purpose scanned Vec | `REWRITE_SUBSYSTEM` | bounded indexed reconnect buffer plus result/debug archive sinks |
| battle simulation/execution | deterministic typed algorithm and broad tests; porous imports/public authority | `REFACTOR` | private kernel façade, local models and compiled `BattleContext`; retain algorithms |
| movement | cohesive typed pipeline and deterministic tests | `RETAIN` + small `REFACTOR` | explicit route/collision/occupancy policies; optional backend injection |
| skill/cast/delivery/interrupt | strong authored/compiler/runtime separation | `RETAIN` + small `REFACTOR` | authored activation/form policy and scoped compiled catalog |
| Boss Runtime | useful compiled battle-local DSL; cross-run policy belongs elsewhere | `RETAIN` | add `EncounterOrchestration`/run arc outside DSL |
| map/facility progression | deterministic graph/topology and node lifecycle tests | `RETAIN` | expose behind run/node use cases; registry maps authored kind to handler |
| Monolith/encounter progression | cohesive deterministic ledger | `RETAIN` | explicit owner port; separate new Relic aggregate |
| fragments/growth | ownership/loadout/compiler strong; Boolean/ordinal form schema limited | `REFACTOR` | versioned stable form IDs and explicit selection/compiler |
| future Relic/corrosion/dice/tool | absent policies with reusable primitives | new bounded subsystems, not rewrite | explicit owner/compiler/cross-layer use cases through new transaction port |
| live-RON loader/validation/compiler | strongest fail-closed publication asset | `RETAIN` | opaque validated output and consumer-scoped compiled catalogs |
| public fixture/data construction | same bare runtime type and panic-oriented alternate path | `REWRITE_BOUNDARY` | typed fallible validated production data vs explicit test fixture context |
| checkpoint/deterministic identity | strict validation, namespaced streams and public continuation equality | `RETAIN` + representation `REFACTOR` | V4 import oracle, target aggregate DTO, command-tape equivalence |
| tests/oracles | extensive behavior/live-data/checkpoint corpus; missing uniform conformance/benchmark | `RETAIN` + extend | differential command tape, all-mutation failure harness and stable performance suite |
| host I/O/observation | correct conceptual external ownership; implementation not audited | `RETAIN OUTSIDE CORE` | explicit receipt/persistence/observation adapter contracts |

No subsystem is a `FULL_REWRITE_CANDIDATE` on current evidence.

## Why An Atomic Transaction Trait Alone Is Insufficient

A trait can standardize methods, but it cannot enforce correctness while handlers can still receive and mutate live
`GameCoreState`. Enforcement must come from the root dispatcher and visibility/type-state:

- every route has an exhaustive transaction scope;
- a mutating handler receives only its staged view/context;
- all fallible validation and projection finishes before commit;
- only a `Projected/Prepared` transition can be committed;
- commit is an infallible ownership swap returning a receipt;
- host durable publication/delivery uses that receipt and explicit rollback/outbox semantics.

The existing helpers prove the pattern; the rewrite makes bypass structurally unavailable.

## Preservation And Migration Rules

- Rulebook behavior and approved technical semantics are the target; current module/wire/save layouts are not.
- Never dual-write old and new mutable authorities. Route one command family to one owner.
- Freeze current public observations before replacing a slice, then compare old/new result, state, events and subsequent
  continuation.
- Keep V4 as a validated import source, not a permanent dual writer.
- Preserve monotonic profile/run-peak exceptions explicitly in transaction and snapshot normalizers.
- Do not split crates until dependency direction works with module visibility; crate boundaries are a final enforcement tool.
- Performance acceptance uses same-runner current-relative gates; GAP-001 exact product caps remain a release-policy decision.

## Order Of Work

1. Build canonical command-tape/state/result/event oracles and a permanent version of E-020.
2. Install root command classification and the transaction kernel over current state; migrate the known direct/late-failure
   routes first.
3. Replace battle-session full-clone/direct-command and event-history delta/archive internals behind preserved battle ports.
4. Introduce opaque validated/scoped data and encapsulate BattleCore dependencies/authority.
5. Migrate node/run phase state one vertical capability at a time.
6. Extract and version protocol adapters, intentionally cut over the small server/Unity consumers.
7. Implement new Relic/forms/corrosion/dice/boss orchestration only through the new owners and gates.

## Reconsider Full Rewrite Only If

- differential oracles show existing gameplay semantics cannot be trusted or isolated;
- the façade/compiled setup/checkpoint/data seams fail to support one single-authority vertical migration;
- approved policy discards most of the Rulebook/domain/data model;
- representative measurements prove the retained core algorithms—not session/archive boundaries—cannot meet the chosen
  budget; or
- checkpoint/identity behavior is broadly nondeterministic and cannot be used as a migration source.

None is currently true. Unlimited schedule and pre-release status favor doing the boundary rewrite carefully and completely,
not discarding tested domain meaning.

## Non-Decision Items

- GAP-001: exact battle retention, tick deadline and concurrent-session caps.
- GAP-002: semantic versus bitwise cross-platform physics replay guarantee.
- Future design-note values and details remain future pressure until promoted to policy.
- Server/Unity implementation and migration are not certified by this Core-only review.
