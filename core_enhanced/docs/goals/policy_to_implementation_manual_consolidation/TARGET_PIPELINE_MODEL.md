# Policy-To-Implementation Target Pipeline Model

Model status: closed for manual authoring  
Normative destination: `policy_to_implementation_manual.md`  
Primary traceability unit: final approved policy Decision ID/revision

## 1. Evidence and authority model

| Axis | Meaning | Required use | Never infer |
| --- | --- | --- | --- |
| Workspace actual | Current checked-out code, data, store behavior, reachable consumers, and dirty state | Mandatory for repository implementation and audit | Released or deployed behavior |
| Released actual | Contents/contract of an identified built or published revision | Required when release compatibility or artifact correctness is claimed; otherwise `unknown` or `N/A` | Deployment reach or traffic |
| Deployed actual | Revision/config/data/consumer set operating in an identified environment | Required when rollout, live migration, or operational behavior is claimed; otherwise `unknown` or `N/A` | Workspace or canonical equality |
| Current canonical | Durable description of the currently supported contract plus its realization index | Mandatory when an owning canonical exists | Current behavior without actual evidence |
| Approved target | Immutable Decision ID/revision and approved bytes/record | Mandatory before material implementation | That implementation already exists |
| Proposed implementation | Goal-local design, package boundary, diff plan, and acceptance evidence | Mandatory before mutation | Product authority or current contract |
| Unknown evidence | A named axis that was not accessible or not required | Mandatory when a relevant claim cannot be made | Pass, current, compatible, or harmless |

Authority precedence is claim-specific: actual evidence owns current behavior, approved policy owns target meaning, current canonical owns the supported durable contract, and Goal artifacts own execution/audit history only.

## 2. Policy and realization lifecycle

### 2.1 Policy revision flow

```text
initial_direction
  -> code_informed_closure
  -> final_frozen
  -> realization_pending
  -> realized
  -> superseded
             \
              -> corrected_by(new Decision/revision)
```

`initial_direction` and `code_informed_closure` may still contain explicit questions. `final_frozen` has immutable approval evidence. `realization_pending` means an authorized implementation path exists but is not current. `realized` requires a stable implementation, current canonical realization index, and Complete/High audit. `superseded` never deletes history.

### 2.2 Implementation Realization Index

The owning current canonical contains a structured index. Each row has:

| Field | Requirement |
| --- | --- |
| Decision | Stable Decision ID and immutable revision/content reference |
| Boundary | Independent policy or explicit atomic policy group |
| Implementation | Single Goal or Master/Phase IDs |
| Effective revision set | Repository IDs and exact code, data, schema/store, server, and required consumer revisions sufficient to reproduce the contract |
| Current canonical revision | Exact canonical revision/content reference containing the row |
| Actual evidence | Workspace plus relevant released/deployed observations or explicit unknown/N/A |
| Realization state | `unrealized`, `realization_pending`, `realized`, `superseded`, or `corrected_by:<Decision>` |
| Audit | Phase/Campaign Complete/High evidence reference |

`partially_realized` is not a state for an atomic policy. Separate rows may be `realized` only when their policy boundaries can independently be current and their consumer/release contracts are valid.

## 3. Minimal orthogonal state axes

### 3.1 Axis vocabulary

| Axis | Owner question | Values | Terminal values |
| --- | --- | --- | --- |
| Policy/realization (`PR`) | What is the approved revision and realization state? | `direction`, `closure`, `frozen`, `pending`, `realized`, `superseded`, `corrected_by` | `superseded`, `corrected_by` for that revision |
| Goal lifecycle (`GL`) | What can this Goal honestly claim? | `draft`, `active`, `policy_decision_required`, `scope_decision_required`, `externally_blocked`, `complete`, `cancelled`, `superseded` | `complete`, `cancelled`, `superseded` |
| Phase activation (`PA`) | May the selected Single/Phase mutate now? | `inactive`, `preflight`, `ready`, `executing`, `audit_candidate`, `closed`, `correction_required` | `closed` only with terminal Goal |
| Campaign coordination (`CC`) | Which dependency/Phase action is eligible? | `N/A`, `planning`, `blocked`, `phase_ready:<ID>`, `phase_active:<ID>`, `phase_audit:<ID>`, `campaign_audit`, `complete`, `correction_required` | `complete` |
| Authorization validity (`AV`) | Is the exact mutation grant usable? | `N/A`, `pending`, `current`, `consumed`, `stale`, `revoked` | `consumed`, `revoked` for that grant |
| Gate/evidence validity (`GE`) | May the named evidence be reused on this tree/input? | per gate: `missing`, `valid`, `stale`, `failed`, `N/A` | none; evidence can be superseded by newer evidence |

Axes remain separate because each independently changes allowed work. Evidence validity is recorded per named gate (`freeze`, `sequence`, `goal_gate`, `preflight`, `phase_audit`, `campaign_audit`), not as one lossy boolean.

### 3.2 Derived metadata and compatibility

| Historical/summary field | Canonical interpretation |
| --- | --- |
| `Status` / `Primary Status` | Alias of `GL`; must not encode activation or audit verdict |
| `Implementation Complete` | Derived: true only for an implementation Single/Phase with `GL=complete`, `PA=closed`, and phase audit `valid`; false while active; N/A for investigation/Master |
| `Completion Verdict` | Evidence result, not state; `Complete / High` is required input to the completion transition |
| `Production Mutation Authorized` | Historical grant fact; usable authority is only `AV=current` plus exact scope match |
| `Activation Status` | Alias of `PA` |
| Master/Phase status text | Alias of `CC`, never a substitute for the child `GL/PA` |

Forbidden combinations:

- `GL=complete` with `PA=executing|audit_candidate|correction_required`.
- `PA=executing` without `AV=current` and `GE.goal_gate=GE.preflight=valid`.
- `PR=realized` without current canonical realization row and applicable Complete/High audit.
- `CC=complete` without `GE.campaign_audit=valid`.
- terminal `GL` with `AV=current`.
- `Implementation Complete=true` when any derived premise is false.

## 4. Execution target, collision, and archive

1. Only a Goal explicitly selected by the current user request or by the selected Master is the execution target.
2. Before activation, scan Goal metadata and scoped owner/path declarations for other nonterminal Goals that could own the same artifact, dependency, Phase, or external effect.
3. Read a collision candidate only far enough to compare identity, nonterminal state, activation, authorization, direct dependencies, baseline, and ownership. Do not execute or inherit it.
4. A terminal Goal (`complete`, `cancelled`, `superseded`) is historical/archive evidence. An unselected nonterminal Goal is dormant for this request but not archive.
5. Past authorization, Gate, preflight, and completion evidence never become current execution authority merely because their Goal was found.
6. A collision without clear ownership sets the selected Goal to the appropriate decision/block state and `PA=correction_required|inactive`; no mutation proceeds.

This limited discovery is the registry-equivalent procedure; no central registry is introduced by this Goal.

## 5. Full Path and Fast Track

| Contract | Full Path | Fast Track |
| --- | --- | --- |
| Campaign/readiness evidence | Separate Campaign/readiness Goal or artifacts allowed | Inline in one Single Goal |
| Handoff | Explicit H1/report-author-activate artifact | Inline handoff record |
| Phase structure | Single or Master/Phase according to dependency model | Single only unless analysis proves multiple independently auditable stages, in which case use Full Path |
| Required invariants | All `PIP-00` rules and PIP-10→70 gates | Identical |
| May omit | Nothing safety-relevant | Separate document/Goal boundaries only |
| Activation | Explicit selected Phase/Single Goal, Goal Gate, current authorization, current preflight | Same |
| Completion | Phase and, when applicable, Campaign Complete/High | Single Complete/High plus realization index |

Fast Track is disallowed when material questions are open, multiple independent Phase gates are needed, a migration/rollout compatibility choice is unresolved, or evidence cannot fit one auditable atomic Goal.

## 6. Event transition matrix

Legend: `unchanged` means the event does not alter that axis; the reason is named in the Result/owner column. `GE:x→y` updates only the listed gate evidence; all unlisted evidence remains unchanged.

| Event | Preconditions | PR | GL | PA | CC | AV | GE / stale evidence | Allowed work after event | Rewind/result owner |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `EV-01 direction recorded` | campaign input identified | `direction` | draft→active for investigation Goal | inactive | planning or N/A | N/A/pending | alignment missing | read-only alignment/closure only | `PIP-10` |
| `EV-02 code-informed material question` | reachable boundary exposes user-visible choice | closure | →policy_decision_required | inactive/correction_required | blocked | current→stale only if grant scope/meaning would change; otherwise unchanged | freeze/sequence/goal/preflight→stale; actual evidence retained if premise unchanged | independent read-only evidence and question preparation | `PIP-20`, user decides |
| `EV-03 final policy approved` | all material questions answered with immutable evidence | closure→frozen | decision Goal may complete; Campaign remains active | inactive | planning | pending/unchanged | freeze→valid; downstream gates missing/stale | readiness/dependency analysis | `PIP-20` |
| `EV-04 dependency/Phase plan fixed` | every policy covered; SCC/intermediate state audited | frozen→pending if implementation path authorized, otherwise unchanged | active | inactive | planning→phase_ready:first or N/A | pending/current per exact grant | sequence→valid; downstream missing | author Goal; no mutation until entry gates | `PIP-30` |
| `EV-05 Goal selected / Goal Gate pass` | explicit target, no unresolved collision, exact authorization/baseline | pending/unchanged | active | inactive→preflight | phase_ready→phase_active:selected, or N/A | must be current for mutation Goal; N/A for nonmutation | goal_gate→valid | preflight only; production still forbidden until EV-06 | `PIP-40` |
| `EV-06 preflight pass / implementation start` | current tree, authority, consumers, dependency, runner checks pass | unchanged: policy revision fixed | active | preflight→ready→executing | phase_active:selected or N/A | unchanged current | preflight→valid | exact authorized mutation and validation | `PIP-40` handoff to `PIP-50` |
| `EV-07 source/dirty drift, no semantic boundary change` | changed bytes affect planned/evidenced paths | unchanged | active | executing→preflight or correction_required | active→blocked for affected Phase | unchanged if grant still exact | goal_gate/preflight and dependent baseline evidence→stale; freeze/sequence unchanged | read-only rebaseline; mutation paused | `PIP-40` |
| `EV-08 authority/consumer boundary drift` | owner, required consumer, persistence lifetime, or reachable contract changes | frozen remains immutable; pending | active or scope_decision_required if boundary expands materially | →correction_required | →blocked | stale if path/operation/effect scope no longer covers work; otherwise unchanged | readiness/sequence/goal/preflight and dependent implementation evidence→stale | read-only reassessment only | `PIP-20` if material meaning; else `PIP-30/40` |
| `EV-09 dependency/atomicity drift` | new edge, SCC, unsafe intermediate state | unchanged | active or scope_decision_required | →correction_required | →blocked | unchanged unless authorized boundary changes | sequence/goal/preflight→stale | graph/Phase redesign only | `PIP-30` |
| `EV-10 technical implementation/audit failure` | target remains valid; implementation/test defect found | unchanged pending | active | executing or audit_candidate→correction_required | phase_audit→phase_active/correction_required | unchanged current if exact grant still covers correction | phase_audit→failed; affected validation/canonical candidate evidence→stale | authorized technical correction, then revalidate/reaudit | `PIP-50`, then `PIP-60` |
| `EV-11 new policy gap during audit` | passing requires a material policy choice | pending→closure for successor revision; frozen evidence retained as history | →policy_decision_required | →correction_required | →blocked | current→stale when meaning/scope changes; otherwise paused unchanged | freeze and all dependent gates→stale for successor; old evidence preserved historically | read-only impact analysis; no production correction | `PIP-20`, user decides |
| `EV-12 Phase Complete / High` | stable cutover, canonical realization candidate, fresh audit valid | pending→realized for that independent policy/group | Phase active→complete | audit_candidate→closed | phase_audit:selected→phase_ready:next or campaign_audit | current→consumed for Phase grant | phase_audit→valid; earlier gates retained as historical valid-at-revision | deterministic finalization; next Phase not auto-active | `PIP-60` / Master selects next |
| `EV-13 next Phase activation` | predecessor Complete/High; Master selects eligible next Phase | unchanged for remaining pending policies | next Phase active | inactive→preflight | phase_ready:next→phase_active:next | next exact grant must be current | next goal_gate/preflight re-evaluated | next preflight then mutation | `PIP-40` |
| `EV-14 Campaign Complete / High` | all policy rows realized or explicitly out/deferred by approved scope; integration audit passes | all in-scope rows realized | Master active→complete | closed/N/A | campaign_audit→complete | any current Campaign grant→consumed | campaign_audit→valid | deterministic finalization/report only | `PIP-70` |
| `EV-15 Campaign audit failure` | cross-Phase seam/coverage defect | realized rows remain historical claims only if defect does not invalidate them; affected row→pending/correction | Master active; affected correction Goal active | affected Phase/new correction→correction_required | campaign_audit→correction_required | new/current exact correction grant required; completed Phase grants stay consumed | campaign_audit→failed; affected seam evidence→stale | correction Goal design; no historical audit rewrite | `PIP-70` routes to `PIP-50/60/80` |
| `EV-16 Goal cancelled` | explicit cancellation; no completion claim | PR unchanged unless separate policy decision | →cancelled | →closed | remove active selection; planning/blocked as applicable | current/pending→revoked | active gates become historical, not reusable | history/report only | `PIP-80` |
| `EV-17 policy/Goal superseded` | explicit successor identity | PR→superseded with successor link | Goal→superseded if its work is replaced | →closed | successor planning | current/pending→revoked; consumed unchanged | predecessor gates historical only | successor Goal under new gates | `PIP-80` |
| `EV-18 post-complete correction` | defect found after terminal completion | realized→corrected_by:new Decision when meaning changes; otherwise realized plus technical correction revision | completed Goal unchanged; new Goal draft→active | new Goal inactive/preflight | correction_required | new exact grant pending/current | old evidence preserved; affected current claim marked stale in new Goal | new linked correction only | `PIP-80`, then appropriate owner stage |
| `EV-19 external evidence unavailable` | required repository/consumer/deploy axis inaccessible | unchanged | →externally_blocked when required for current claim | inactive/correction_required | blocked | unchanged unless time/scope condition invalidates it | affected evidence missing/stale; unrelated evidence unchanged | independent read-only work only; no affected pass claim | evidence owner / external change |

## 7. Allowed-action decision order

For every mutation decision, evaluate in this order:

1. Is the current user/Master-selected Goal exact and nonterminal?
2. Does its Goal type own production mutation?
3. Is `AV=current` for the exact paths/operations/effects?
4. Are `GE.goal_gate` and `GE.preflight` valid on the current baseline?
5. Are policy freeze and sequence evidence valid for this Phase?
6. Is `PA=executing`, and for a Master is `CC=phase_active:<this Phase>`?
7. Is there no unresolved nonterminal ownership collision?

Any “no” forbids mutation and routes through the event matrix. No summary field may override this order.

## 8. Model closure verdict

- Actual/canonical/target/implementation/unknown axes: closed.
- Realization representation: current canonical structured Implementation Realization Index.
- Orthogonal state axes: 6; duplicate historical fields made derived.
- Required event rows: 19, including every event named by the consolidation prompt.
- Full/Fast common invariants: identical; only artifact topology differs.
- Execution-target/archive/collision rule: closed without central registry.
- Unresolved model choice: none.

