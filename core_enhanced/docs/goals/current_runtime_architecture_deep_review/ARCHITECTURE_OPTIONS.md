# Architecture Options

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Baseline: `BASELINE-20260822-064427-KST`
- Decision evidence: E-024

All options preserve Rulebook capabilities and are compared against the same assets, scenarios and gaps. Relative size means
semantic/change surface, not invented calendar estimates.

## Option Comparison

| Option | What changes | Legitimate advantage | Main residual/risk | Oracle and migration | Performance | 1-person + AI fit | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- |
| O1. Maintain + defect remediation | patch known failed-command ordering, visibility and event indexing in current owners | smallest immediate code delta; fastest correctness relief | handler conventions, parallel phase state, protocol concentration and broad context keep recreating the same class | strongest short-term oracle, but no structural migration seam is exercised | event patch can help; full-state staging remains | low short-term context, high recurring policy memory | insufficient as end state; useful first patches only inside O3 |
| O2. Module/visibility refactor | make authority private, split files/modules, add validated wrappers while retaining root control flow | improves compiler boundary and discoverability with moderate semantic risk | root still chooses transaction per handler; phase/session invalid combinations and protocol/app coupling remain | highly reversible and useful precursor | can enable scoped clones; does not guarantee resource shape | better bounded reading, still broad changes | necessary enabling work, insufficient alone |
| O3. Staged application/protocol shell rewrite | preserve `GameCore` façade/domain engines; replace use-case transaction, typed session state, protocol adapters, battle session/archive and data-construction ports slice by slice | removes all three consolidated root causes while old behavior remains executable | needs disciplined route-by-route migration, normalizers and temporary adapters | strongest: MS-001~008 allow one authority and old/new command-tape comparison | can change staging/history behind stable port and enforce E-020 gates | best bounded contexts and compiler-enforced use cases | **recommended** |
| O4. Subsystem rewrite combination | replace selected movement/skill/boss/map or battle kernels in addition to boundaries | justified if a particular algorithm cannot express approved policy or meet measured budget | current scenario evidence says these engines are cohesive; rewriting them adds meaning rediscovery without removing root shell pressure | feasible per subsystem, but duplicates tested algorithms and expands parity matrix | may improve a proven hotspot, none currently proven inside algorithms | bounded if truly isolated; needless rewrites burden policy review | reserve for a future failed subsystem gate, not current plan |
| O5. Entire Core big-bang rewrite | new domain/application/data/protocol/persistence implementation developed as one replacement | maximum layout freedom and no temporary legacy adapters | highest Rulebook rediscovery, determinism/save/data/test parity risk; features stall or two cores diverge | weak: old oracle must be recreated while every owner changes; rollback is whole-program | can design budgets cleanly but must rediscover current baseline and may regress invisibly | largest simultaneous context and human policy review surface | not justified |

## Why O3 Is Not Merely A Large Refactor

The following boundaries are intentionally new implementations behind retained ports:

- root-enforced command classification and `Prepared → Projected → CommitReceipt` transaction kernel;
- typed `SessionState`/`RunPhase` and `ActiveNodeSession` authority replacing parallel root options;
- versioned external contract adapters separated from domain commands/results/errors;
- battle mutation/session staging plus bounded indexed reconnect/archive storage;
- opaque validated game data and battle-scoped compiled context.

Those are `REWRITE_BOUNDARY` dispositions. The facility graph, movement, skill execution, Boss DSL, Monolith, reward planners,
strict data validation, deterministic identity and checkpoint validator stay authoritative until each adapter slice passes its
equivalence gate.

## Full-Rewrite Evidence Test

| Required condition from criteria | Current evidence | Met? |
| --- | --- | --- |
| no realistic seam can separate core state/domain boundaries | façade routing, staged helpers, compiled setup/data and checkpoint ports are proven seams | no |
| current tests/contracts cannot serve as new implementation oracle | 1,131 baseline tests plus public snapshot/result/event/continuation oracles exist | no |
| preserved algorithm/data/persistence value is lower than rediscovery cost | PA-001~012 include most gameplay algorithms and validation/identity contracts | no |
| incremental migration necessarily creates dual authority or larger long-term risk | route-level adapter with one commit owner is feasible; dual write is explicitly forbidden | no |
| full-rewrite target and whole-system equivalence are more concrete than bounded migration | target layers are clear, but all-feature parity remains broader than slice gates | no |

None of the mandatory conditions is met. Pre-release schedule freedom increases the amount of boundary redesign that can be done;
it does not reverse the semantic-risk comparison.

## Target Dependency Direction

```text
host persistence / network / observation adapters
  -> versioned contract adapters
    -> GameCore facade + application use-case registry
      -> transaction kernel + typed session/run aggregates
        -> domain engines (map, growth, reward, battle application policy)
          -> battle kernels (skill, movement, boss, wave, spawn)
            -> immutable compiled scoped catalogs / value types

authoring RON -> parse -> validate -> compile -> scoped catalogs
domain events/results -> projection mapper -> contract DTO
```

Only arrows in this direction are allowed. Domain/battle leaves do not import transport DTO/error, world orchestration,
filesystem or raw authoring types. Physical crate splitting is optional after module visibility proves the direction.

## Migration Stages And Gates

| Stage | Change | Authority/reversibility | Required gate |
| --- | --- | --- | --- |
| M0. Freeze observations | canonical command tapes, semantic snapshot normalizer, E-020 benchmark fixtures, V4 importer oracle | no runtime behavior change | current tests + continuation/result/event fixtures |
| M1. Transaction kernel | classify every route; run mutations through typed scope and one prepared commit on current state | one dispatcher selects old adapter or new use case; revert per route | all rejection equality/retry tests; no post-commit Core failure |
| M2. Battle session/archive | stage touched battle state; direct command delta; indexed bounded reconnect/archive | preserve battle kernel/event semantics and cursor adapter | current tactical/battle tests; constant-tail/resource budgets |
| M3. Validated/scoped data and kernel boundary | opaque validated runtime data; compiled battle context; private authority/local errors | official loader remains source; fixture contexts explicitly separate | live publication parity/generated setup; kernel tests without full DB |
| M4. Typed session/run aggregate | migrate one phase/node family at a time from parallel roots to `RunPhase`/`ActiveNodeSession` | only one phase implementation handles a routed command | old/new command tape, checkpoint round trip and failure retry per slice |
| M5. Versioned protocol | map domain commands/results/errors to new envelope/capability modules | current contract adapter remains until consumers intentionally switch; no dual domain authority | schema fixtures and adapter equivalence; approved compatibility cutoff |
| M6. Future systems | Relic/forms/corrosion/dice and expanded boss orchestration through new ports | new policies land only after their owner/compiler/transaction is defined | scenario-specific Rulebook acceptance plus continuation/performance gates |

Every stage can stop with a shippable single authority. A failed slice can route back to the previous implementation without
rolling back unrelated migrated domains.
