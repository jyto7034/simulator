# Policy-To-Implementation Manual State-Machine Correction Goal

Primary Status: complete
Activation Status: closed
Goal Type: single
Execution Mode: documentation_only
Implementation Complete: true
Authorization Validity: consumed
Authorization Evidence: `AUTHORIZATION.md`
Change Type: governance_only
Risk Level: medium
Reviewer Requirement: same implementer, fresh separate read-only pass
Goal Execution Gate: passed — exact manual path, seven accepted findings, baseline hash, create/modify scope and existing-Goal exclusion are fixed below
Frozen Audit Candidate: manual SHA-256 `26d58b242e55842c657be53d327c7ec89109cd0dd2502821a9d7863a4d0d8ec1`; walkthrough SHA-256 `b7deefeaec7491eb495714f94fa4d9ed4dd035adaf328c55ca4a808aa1e2717c`
Reviewer Evidence: SHA-256 `3a944ad8f7306ca6208f9516b17e076068b6893c13f5c22e14677e9bf528444e`<br>
Completion Audit: SHA-256 `88305fe0f9bc14e633bb4bca55a321f2f784644ada05c6cd770d2ccccd4024c5`<br>
Completion Verdict: `Complete / High`

## Objective

Correct the seven accepted state-machine defects in `core_enhanced/docs/policy_to_implementation_manual.md` without changing its intended Policy-To-Implementation semantics or any existing Goal document.

## Approved findings

1. Add a valid `author_only` / `implementation_goals_authored` exit.
2. Add durable `ready→executing` and `phase_active→phase_audit` happy-path transitions.
3. Route technical and post-complete correction through `PIP-40` re-entry.
4. Replace ambiguous PIP-80 alternatives with guard-selected deterministic rows.
5. Keep current canonical unchanged until an audited, content-addressed candidate is promoted.
6. Define reviewer-owned deterministic Fit High/Medium/Low rubric.
7. Define `PIP-GOL-007` and normalize `PR=realization_pending`.

## User decisions

- `report_only` default exit is ATI-04 `sequence_safe`.
- Authored inactive Goals use `GL=active`, `PA=inactive`, `AV=pending|N/A` and a separate `implementation_goals_authored` gate.
- Canonical candidates are content-addressed Goal artifacts or exact patches; current canonical is promoted only after candidate `Complete / High`.
- All other correction recommendations in the accepted review are approved.

## In scope

- Modify only `core_enhanced/docs/policy_to_implementation_manual.md`.
- Create evidence only under this new Goal directory.
- Recompute routing line hints.
- Re-run transition reachability, Rule reference, local-contract and 13-scenario logic checks.

## Out of scope

- Any pre-existing `core_enhanced/docs/goals/**` file or directory.
- README, engineering workflow, runtime/server/client/data/schema/DTO/persistence/product policy.
- Archived/history document rewriting.

## Completion conditions

- All seven findings have an explicit normative owner and deterministic transition/artifact.
- `report_only`, `author_only`, `author_and_activate`, normal Phase exit, technical correction and post-complete correction each reach a valid partition Entry/Exit Gate.
- Canonical candidate/current promotion and `realization_pending→realized` are unambiguous.
- Fit outcome is reproducible from dimension evidence.
- PIP-80 contains no unguarded slash/pipe alternatives.
- All stable markers and routing ranges agree after edits.
- Existing Goal files remain byte-for-byte untouched by this correction.
- Fresh review concludes `Complete / High`.

## Stop conditions

- A correction would change material product policy or broaden production authorization.
- A pre-existing Goal file must be edited.
- The seven accepted choices cannot yield one deterministic state transition.
