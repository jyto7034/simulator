# Manual Partition And Reader Routing Map

Status: implemented and verified at manual revision `PIP-MANUAL-2026-08-13-R1`  
Stable boundaries: `<!-- PIP-NN START -->` and `<!-- PIP-NN END -->`  
Default read set: `PIP-00 + one current partition`; extra partitions are allowed only when the current partition lists them.

## Partition ownership

| Section | Unique normative responsibility | Read when | Declared additional sections | Must not own |
| --- | --- | --- | --- | --- |
| `PIP-00` | Reader Contract, source/actual model, authority/ownership, state axes, authorization, evidence/dirty invariants, traceability, document ownership, ATI crosswalk | Always | none | Stage-specific procedure |
| `PIP-10` | Campaign request, path/handoff selection, initial direction/input registry | Campaign entry | `PIP-90` only when creating prompt/artifact formats | Closure, dependency, implementation |
| `PIP-20` | Alignment, reachable boundary trace, materiality questions, uncertainty/assessment, final freeze | Policy closure/readiness evidence | `PIP-80` on a decision/block event; `PIP-90` for formats | Goal activation or mutation |
| `PIP-30` | Policy-to-Phase mapping, packages, typed dependency/SCC, atomic groups, safe order/parallel/intermediate states, Goal architecture | Implementation planning | `PIP-80` on scope/dependency invalidation; `PIP-90` for formats | Preflight or implementation |
| `PIP-40` | Handoff, selected Goal, limited collision discovery, Goal Gate, authorization binding, baseline/current-tree preflight, Fast Track inline evidence | Every Single/Phase entry | `PIP-30` to confirm dependencies; `PIP-80` on drift/collision; `PIP-90` for Goal templates | Production edits before exit |
| `PIP-50` | Target-first implementation, atomic cutover, migration/compatibility, consumers, validation, canonical realization candidate | Active Phase execution | `PIP-80` on drift/failure; `PIP-90` for evidence formats | Completion transition |
| `PIP-60` | Frozen audit candidate, reviewer pass, Phase/Single Complete/High, deterministic finalization, next-Phase handoff | Phase/Single exit | `PIP-80` for any failure/correction; `PIP-90` for audit/report format | Campaign completion |
| `PIP-70` | Cross-Phase policy coverage/seams/regression/canonical realization Campaign audit and finalization | Campaign exit | `PIP-80` for integration failure/correction; `PIP-90` for report format | Historical Goal rewrite |
| `PIP-80` | Unified event transition matrix, evidence invalidation, nonterminal/terminal/archive, cancellation/supersede/post-complete correction | Only when an exceptional event occurs | The owner partition named by the transition row | Normal happy-path procedure |
| `PIP-90` | Canonical metadata, minimum artifacts, evidence reference, prompt/PLAN/audit/report formats | Only when authoring a named format | none; formats reference owners | New lifecycle rules or exceptions |

## Reader closure tests to enforce in walkthrough

| Scenario class | Current partition | Minimum read set | Why sufficient |
| --- | --- | --- | --- |
| Report readiness | `PIP-20` | `00+20+90` | Closure, verdict, and report format |
| Author inactive Goal | `PIP-40` | `00+40+30+90` | Architecture prerequisite, handoff/gate, format |
| Execute Single/Phase | `PIP-50` | `00+50`; entry was separately decided with `40` | Execution partition owns full local procedure |
| Audit Phase | `PIP-60` | `00+60+90` | Audit/finalization plus format |
| Plan Master/atomic group | `PIP-30` | `00+30+90` | Mapping/dependency/architecture plus format |
| Handle any drift/failure | `PIP-80` | `00+80+<owner named by event>` | One matrix decides all axes and rewind owner |
| Audit Campaign | `PIP-70` | `00+70+90` | Integration audit and report format |

## Cutover verification checklist

- Every partition has Purpose, Entry Conditions, Required Inputs, Additional Sections To Read, Allowed/Forbidden Actions, Procedure, Required Artifacts, Exit Gate, Rewind Conditions, and Next Stage.
- Global rules occur once in `PIP-00`; local partitions cite stable Rule IDs.
- Stable markers are present exactly once per partition.
- Routing table line ranges are generated from final markers and treated as hints, not identifiers.
- No compatibility entrypoint contains an alternative state table, checklist, stop condition, or exception.

## Implemented line ranges

| Partition | Marker range |
| --- | ---: |
| `PIP-00` | `39–196` |
| `PIP-10` | `200–266` |
| `PIP-20` | `270–338` |
| `PIP-30` | `342–410` |
| `PIP-40` | `414–506` |
| `PIP-50` | `510–593` |
| `PIP-60` | `597–670` |
| `PIP-70` | `674–735` |
| `PIP-80` | `739–813` |
| `PIP-90` | `817–966` |

Post-cutover marker/routing comparison: pass.
