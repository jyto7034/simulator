# Partition Read-Set Walkthrough

Manual candidate: `core_enhanced/docs/policy_to_implementation_manual.md` at draft revision `PIP-MANUAL-2026-08-13-R1`  
Method: for each required scenario, use only `PIP-00 + current partition + explicitly declared additional section`; verify one decision, all six state axes, allowed/forbidden work, stale evidence, and next owner  
Result: 13/13 pass

## W1 — Readiness result only

- Situation/current stage: user requests code-informed readiness findings but no Goal authoring or mutation; current `PIP-20`.
- Read set: `PIP-00 + PIP-20 + PIP-90` (report format).
- Rules: `PIP-SOT-001/002/003/004/005`, `PIP-STG-002`, `PIP-OWN-004`, `PIP-SEQ-005`, `PIP-EVD-004/005/006`, `PIP-AUD-008`.
- Decision owner: agent for evidence/technical assessment; user for any material question.
- State: `PR direction→closure→frozen` only if questions close; readiness Goal `GL active→complete` under its own investigation audit, `PA=inactive`, `CC=N/A|planning`, `AV=N/A`, `GE freeze/readiness=valid`; implementation gates remain missing.
- Allowed: read-only code/data/consumer trace, assessment, decision questions, report.
- Forbidden: author/activate implementation Goal or mutate production.
- Stale evidence: none when inputs unchanged; material question leaves freeze/downstream missing/stale.
- Result: `report_only` terminates with readiness verdict and no activation. One reasonable outcome. Pass.

## W2 — Author Goal but keep production inactive

- Situation/current stage: readiness is sequence-safe; user requests documents only; current `PIP-40`.
- Read set: `PIP-00 + PIP-40 + PIP-30 + PIP-90`.
- Rules: `PIP-OWN-003/004`, `PIP-STG-004`, `PIP-GOL-001/002/004`, `PIP-AUT-001/003/004`, `PIP-TPL-003`.
- Decision owner: agent chooses Single/Master/Phase from `PIP-30`; user’s `author_only` bounds outcome.
- State: policy `PR=pending`, Goal `GL=active`, `PA=inactive`, Master `CC=phase_ready` or N/A, `AV=pending|N/A`, Goal document completeness may be valid but mutation Gate/preflight are not activation authority.
- Allowed: PLAN/Master/Phase/authorization-request artifacts and links.
- Forbidden: `PA→executing`, production edits, using “Goal exists” as authorization.
- Stale evidence: none; activation evidence is deliberately missing rather than failed.
- Result: authored inactive Goal and exact next activation inputs. Pass.

## W3 — Implement and audit one policy as Single

- Situation/current stage: one final policy is one atomic current boundary; implementation in `PIP-50`, then audit in `PIP-60`.
- Read set: execution `PIP-00 + PIP-50`; audit `PIP-00 + PIP-60 + PIP-90`.
- Rules: `PIP-GOL-001`, `PIP-AUT-001/002`, `PIP-SEQ-006/007/008/009/010`, `PIP-SOT-006`, `PIP-AUD-001`–`005`.
- Decision owner: agent implements exact approved target; user only if material meaning changes.
- State: entry `PR=pending, GL=active, PA=executing, CC=N/A, AV=current, GE entry=valid`; stable cutover → `PA=audit_candidate`; Complete/High → `PR=realized, GL=complete, PA=closed, AV=consumed, phase audit=valid`.
- Allowed: exact authorized vertical cutover, validation, owning canonical realization candidate, read-only audit, deterministic finalization.
- Forbidden: fallback/dual authority without approval, frozen target rewrite, audit-time material edit, completion below Complete/High.
- Stale evidence: implementation change during audit stales audit candidate and requires fresh audit.
- Result: Single completion; if PLAN predeclares the Single audit’s separate Campaign coverage section, Campaign closes there, otherwise `PIP-70`. Pass.

## W4 — Multiple policies in Master/Phase dependency order

- Situation/current stage: planning in `PIP-30`, repeated entry `PIP-40`, execution/audit `PIP-50/60`, final `PIP-70`.
- Read set for planning decision: `PIP-00 + PIP-30 + PIP-90`; each later stage uses its own declared set.
- Rules: `PIP-STG-001/003/006`, `PIP-GOL-001/002/003/008`, `PIP-SEQ-001`–`004`, `PIP-AUD-006`.
- Decision owner: agent owns technical graph/Phase architecture; user owns scope or material policy boundaries.
- State: `CC planning→phase_ready:P1→phase_active:P1→phase_ready:P2...→campaign_audit→complete`; each Phase has independent `GL/PA/AV/GE`. Master has no production authorization.
- Allowed: activate exactly one eligible Phase after predecessor Complete/High and fresh Gate/preflight.
- Forbidden: Master production edit, next-Phase autoactivation, same-layer parallelism without positive evidence.
- Stale evidence: a new dependency stales sequence and affected entry gates, not frozen policy evidence.
- Result: serialized or proven-safe parallel Phase campaign, then independent Campaign audit. Pass.

## W5 — Atomic policy group Phase

- Situation/current stage: several policies share one transaction/schema/persistence/release unit; current `PIP-30`.
- Read set: `PIP-00 + PIP-30 + PIP-90`.
- Rules: `PIP-GOL-003`, `PIP-SEQ-001/002/003/008`, `PIP-STG-003`.
- Decision owner: agent when shared technical lifetime is proven; user if grouping changes product scope or acceptance meaning.
- State: each Decision remains `PR=frozen/pending`; one Phase covers explicit atomic group; group realization moves all included independently inseparable rows only at Complete/High.
- Allowed: contract SCC and record included policy IDs, coupling edges, individual acceptance, common completion boundary.
- Forbidden: implicit splitting, placeholder schema, `partially_realized` for the atomic group.
- Stale evidence: new edge/SCC invokes `EV-09`, staling sequence/entry evidence.
- Result: one auditable Phase with all included policy traceability retained. Pass.

## W6 — New material question during code-informed closure

- Situation/current stage: reachable DTO/save/consumer path reveals two observable behaviors; current `PIP-20`, exception `PIP-80`.
- Read set: `PIP-00 + PIP-20 + PIP-80`.
- Rules: `PIP-OWN-001/002`, `PIP-SOT-004`, `PIP-STG-007`, transition `EV-02`.
- Decision owner: user for the minimal observable choice; agent provides options/evidence/trade-offs.
- State: `PR→closure`, `GL→policy_decision_required`, `PA→inactive/correction_required`, `CC→blocked`; AV stale only if approved mutation meaning/scope changes; freeze/downstream evidence stale.
- Allowed: independent read-only investigation and concise question.
- Forbidden: select a compatibility/save/ordering meaning and proceed as technical detail.
- Stale evidence: exact downstream dependency set in `EV-02`; old actual evidence remains usable if its premise is unchanged.
- Result: stop affected conclusion until approval, then freeze new revision. Pass.

## W7 — Authority/dependency/dirty drift after readiness

- Situation/current stage: before or during Phase entry, current code owner changes, a dependency edge appears, or target hunk is dirty; current `PIP-40`, exception `PIP-80`.
- Read set: `PIP-00 + PIP-40 + PIP-30 + PIP-80`.
- Rules: `PIP-EVD-002/003`, `PIP-AUT-005`, `PIP-STG-005/007`, `EV-07/08/09`.
- Decision owner: agent for safe rebaseline/dependency analysis; user for expanded scope/material boundary.
- State: source-only drift → `PA=preflight/correction_required`; authority drift → affected readiness/sequence stale; dependency drift → sequence stale; `CC=blocked`. AV changes only when its exact grant no longer covers work.
- Allowed: read-only diff attribution, rebaseline, graph reassessment.
- Forbidden: continue mutation on stale Gate/preflight or overwrite owner changes.
- Stale evidence: matrix specifies baseline/goal/preflight vs readiness/sequence granularity.
- Result: earliest changed premise owns rewind, no all-or-nothing invalidation. Pass.

## W8 — Ownership collision with another nonterminal Goal

- Situation/current stage: selected Goal A and dormant active Goal B own overlapping docs/files; current `PIP-40`, exception `PIP-80` if unresolved.
- Read set: `PIP-00 + PIP-40 + PIP-80`.
- Rules: `PIP-GOL-004/005`, `PIP-EVD-002/003`, `PIP-STG-005`, `EV-07/08`.
- Decision owner: user/Master selects execution target; agent discovers and compares ownership.
- State: Goal A remains `GL=active`, `PA=correction_required|inactive`, `CC=blocked`; Goal B is unchanged and unexecuted. AV A unchanged only if scope still exact; Gate/preflight stale/failed for collision.
- Allowed: limited metadata/path/dependency/baseline comparison.
- Forbidden: call B archive because unnamed, execute B, inherit B authority, modify overlapping hunks without attribution.
- Stale evidence: A’s baseline/Gate and any owner-dependent preflight evidence.
- Result: resolve ownership/scope or remain decision-required; no collision missed. Pass.

## W9 — Completion audit technical defect versus policy gap

- Situation/current stage: audit in `PIP-60`; exception `PIP-80`.
- Read set: `PIP-00 + PIP-60 + PIP-80 + PIP-90`.
- Rules: `PIP-AUD-001/002/004/005`, `PIP-STG-007`, `EV-10/11`.
- Decision owner: agent corrects technical defect within exact grant; user decides material policy gap.
- Technical state: `PR` unchanged, `GL=active`, `PA→correction_required`, phase audit failed, exact AV may stay current; return `PIP-50`.
- Policy-gap state: successor `PR→closure`, `GL→policy_decision_required`, `PA→correction_required`, `CC→blocked`; affected freeze/downstream evidence and possibly AV stale; return `PIP-20`.
- Allowed: technical correction only in first branch; read-only decision evidence in second.
- Forbidden: one branch masquerading as the other or deterministic finalization after failed audit.
- Result: two unambiguous outcomes from the same matrix. Pass.

## W10 — Next Phase activation and Campaign audit failure

- Situation/current stage: P1 exits in `PIP-60`, P2 entry in `PIP-40`, final integration in `PIP-70`, failure in `PIP-80`.
- Read set at each decision: `00+60+90`, then `00+40+30`, then `00+70+90`; failure `00+80+70`.
- Rules: `PIP-STG-006`, `PIP-GOL-008`, `PIP-AUD-005/006/007`, `EV-12/13/15`.
- Decision owner: Master selects next eligible Phase; audit identifies the owning seam/correction.
- State: P1 Complete/High consumes P1 grant and sets `CC=phase_ready:P2`; P2 is not auto-active. Campaign failure sets Master active/`CC=correction_required`, preserves completed Phase history, and opens new correction state.
- Allowed: fresh P2 Gate/preflight; new correction Goal after failure.
- Forbidden: reuse P1 grant, rewrite P1 audit, mark Master complete after Campaign failure.
- Stale evidence: affected cross-Phase seam/campaign evidence only; valid independent Phase evidence remains historical unless the defect invalidates its claim.
- Result: safe handoff and history-preserving correction. Pass.

## W11 — Realization, cancellation/supersede, post-complete correction

- Situation/current stage: realization in `PIP-50/60`, lifecycle exception in `PIP-80`.
- Read set: normal `00+50`, audit `00+60+90`, exception `00+80` plus named owner.
- Rules: `PIP-SOT-006`, `PIP-GOL-009`, `PIP-SEQ-010`, `EV-16/17/18`.
- Decision owner: user owns policy supersede/new material revision; agent can author technical correction within approval.
- State: realized policy has immutable target + canonical realization row. Cancellation changes Goal only and revokes current grant. Supersede links successor Decision. Post-complete correction leaves completed Goal/audit unchanged and starts new Goal with new gates/grant.
- Allowed: structured realization revision and linked successor/correction.
- Forbidden: overwrite frozen target, reactivate terminal Goal, edit historical audit, treat cancellation as policy supersede.
- Stale evidence: affected current realization claim is marked in the new correction; historical evidence remains retrievable.
- Result: policy and Goal history are distinct and reproducible. Pass.

## W12 — Inaccessible consumer and actual-axis mismatch

- Situation/current stage: closure or implementation evidence finds workspace green, released older, deployed unknown, required consumer inaccessible; `PIP-20` or `PIP-50`, exception `PIP-80 EV-19`.
- Read set: closure `00+20+80`; implementation `00+50+80`.
- Rules: `PIP-SOT-001/003`, `PIP-EVD-004/005/006`, `EV-19`.
- Decision owner: agent bounds evidence claims; user may explicitly exclude or change rollout scope if material.
- State: PR unchanged; required inaccessible evidence → `GL=externally_blocked`, `PA=inactive/correction_required`, `CC=blocked`; AV normally unchanged; affected evidence missing/stale.
- Allowed: report workspace actual, released actual revision, deployed unknown, and independent evidence.
- Forbidden: infer deploy from workspace, call inaccessible consumer pass, globally discard unrelated evidence.
- Result: affected completion/activation denied, exact known axes preserved. Pass.

## W13 — Fast Track

- Situation/current stage: small, closed, single atomic policy; starts `PIP-10`, inline closure/sequence/entry in one Single PLAN, then `PIP-50/60`.
- Read set for path decision: `PIP-00 + PIP-10 + PIP-90`; inline gates each use their declared partition set.
- Rules: `PIP-STG-008`, `PIP-GOL-001`, `PIP-AUT-001/003`, `PIP-SEQ-006`–`010`, `PIP-AUD-001`–`005`, `PIP-TPL-001`.
- Decision owner: agent permits Fast Track only if no material question/multi-Phase/migration-choice blocker; user owns any exposed material choice.
- State: same six-axis transitions as Full Path. Separate readiness Goal/H1 document is absent, not the `freeze/sequence/goal_gate/preflight` evidence.
- Allowed: inline `PIP-20/30/40` records in one Goal, exact authorized Single implementation.
- Forbidden: skip source separation, policy closure, dependency proof, authorization, baseline, atomic cutover, canonical realization or Complete/High audit.
- Stale evidence: same event matrix as Full Path.
- Result: fewer artifacts/hand-offs, identical safety and terminal semantics. Pass.

## Walkthrough verdict

| Check | Result |
| --- | --- |
| Required scenarios | 13/13 present |
| Scenario fields | stage, minimum read set, rules, owner, transitions, allowed/forbidden, stale evidence, result in every scenario |
| More than one reasonable outcome after declared inputs | 0 |
| Undeclared partition needed to decide | 0 |
| Product-policy choice silently inferred | 0 |
| Partition/model corrections required | 0 |
| Overall | pass |

