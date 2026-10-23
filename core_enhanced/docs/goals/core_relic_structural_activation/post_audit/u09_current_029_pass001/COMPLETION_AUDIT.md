# U09 independent whole-Goal completion audit — target 029

Verdict: **Not Complete / High confidence / 1 open finding (F01)**. The review is complete; the Goal is not complete. This is a whole `GS-CRSA-003 / GOAL-CRSA-01` audit, not a review limited to Product023. No source, tests, fixtures, original evidence, canonical documentation, or frozen target were changed by the auditor.

## Exact target and auditor

- Target: `execution/single_goal/target/u09_current_029` under this Goal.
- `MANIFEST.json`: SHA256 `9ea2e2bc6b6687ba99649d1b9bc7d6d4f8922429ef7b0785fb2fadad0557a1f3`, 195330 bytes.
- Product/test/config membership: 492 files, source SHA256 `1ca0fbe68a2ba80603470dde38a6c50e67c9d0ca3938391dbc4b211e875c492e`.
- Restoration membership: 495 files, including three canonical documentation files; before membership: 441, including their preimages. The scoped delta has 97 changed paths, 54 added paths, and no deleted paths.
- Auditor: `/root/u09_independent_audit029`, nonparticipant in this Goal's planning, design, product implementation, and test authoring. Root confirmed actual session model metadata `gpt-6-astra / high`; this declaration is not inferred solely from the assignment label.
- Audit authority: the user's explicit delegation of final whole-Goal audit. No additional agents were created.
- Entry and exit comparisons read all 2626 frozen membership references, 14 target archive/patch references, and the manifest: 2641 exact byte-size/SHA256 comparisons per stage. See `ENTRY_CURRENTNESS.json` and `EXIT_CURRENTNESS.json` for actual results and times.

The auditor read policy freeze FPF002, GS003, P2G result/handoff002, code review002, design delta002, current/final PLAN and experiment/state evidence, the generated003 coverage/selection/predecessor records, scoped source delta and actual source/fixture/oracle/raw evidence. GEX currentness, restoration and GEX-20 rules were applied, with the current roadmap usage distinction between implementation and independent completion. Historical U08 `post_audit/u08_current_013_pass002/AUDIT_CLOSURE.json` was read and accepted as Complete/High/open0 for its historical target; its original preaudit target is not silently promoted into a completion receipt.

No auditor build, test, restore, or evidence collector was run. Read-only currentness comparisons and serialization of these independent judgments are the only audit-side computations. Sol's existing reaped freeze execution, raw output, and `RESTORE_VERIFICATION.json` establish isolated forward/reverse membership and byte equality. Both directions are true; no reverse patch was applied to the workspace.

## F01 — generated effects bypass selected-reach conflict validation

Severity: completion-blocking correctness/contract defect. Confidence: High. Owner: structural compiler/lowering product owner; independent regression author and Sol runner must provide the verification. There is one defect with multiple affected acceptance statements, not multiple separate defects.

`core_enhanced/src/game/relic/structural/execution.rs:539-548` merges the selection reach returned by each original effect and returns `InvalidReference` / `selected support reach conflict` when two reaches differ. The generated `InlineEffects` path at lines 599-601 and generated `FiniteFlow` path at lines 638-640 instead assign `selected_recipient_reach = Some(reach)` unconditionally. The last generated overlay silently replaces a reach already supplied by another original or generated effect.

This is externally meaningful. `EffectiveProviderPlan` has one shared selection reach. `target_usefulness.rs:497` reads it for fixed support admission, and `sustained/structural.rs:128-146` reads it again and stores it in the actual fixed recipient connection. The authored channel's original reach is therefore not protected from an unrelated generated selected-recipient effect.

Concrete valid-path construction, established by source inspection:

1. Use an original `SelfUnit / Instant` action with a `ChannelAura` authored radius of 2,000,000 data units, transformed by `FixedRecipientSupport`.
2. Grant an area `PersonalShieldGrant` with radius 4,000,000 on that action through `InlineEffects`; select that generated effect with `PersonalShieldSingle`.
3. `compiler.rs:1121-1144` validates the grant's original area primitive before overlay. `validate_grant_carrier` at lines 516 onward accepts the original area grant on `SelfUnit / Instant`.
4. The recipe converts that generated grant to selected-recipient form. Generated sites are included in `single_recipe_sites`; `skill_data.rs:1316-1320` permits this transformed selected grant on the original SelfUnit carrier with PlayerUnit selection. Publication calls this same lowering path and adds no later conflicting-reach check.
5. Original lowering records 2,000,000; generated lowering replaces it with 4,000,000. A qualified ally whose distance is greater than 2,000,000 and at most 4,000,000 can consequently satisfy both the generated personal grant's own reach and the fixed support consumer's incorrect shared reach. The latter stores the enlarged range. A smaller last generated reach can instead contract admission.

The same missing check is visible in the finite generated path. This is a static source-path finding, **not a claim that the auditor ran this scenario or observed its runtime output**. The existing `rv004_publication_lowering_still_rejects_conflicting_selected_reaches` (`structural/tests/publication_cases.rs:228-268`) tests two original personal effects and the equal-reach positive control. It never reaches either generated assignment, so its recorded PASS does not refute this finding. The canonical U09 authoring section also explicitly promises publication refusal for conflicting reaches within one skill; F01 is an implementation gap under the existing contract, not a request for a new policy decision.

Required correction and closure evidence:

- Apply one consistent typed conflict rule to original, inline-generated, and finite-generated reach contributions before publication succeeds. Preserve equal-reach compositions and existing diagnostics.
- Add independently specified negative cases for original/generated and generated/generated mismatches, both grant forms, and effect/grant/copy ordering permutations. Include the mixed fixed-support plus generated personal-single construction above, both greater and smaller conflicting reaches, and equal-reach positive controls.
- Verify public content publication refuses the invalid combination without partial catalog or loadout commit. Where the world transition path is applicable, retain original state/resources/owner on refusal. Validate legal equal-reach commands and stored connection geometry against authored reach.
- Keep the original-only regression; select necessary affected compiler/publication and actual-consumer regressions. New exact source/evidence bindings and a newly frozen target are required before a completion re-audit. Do not edit target029 or its original failed evidence in place.

Primary affected sentences: `PE-CRSA-04.2`, `07.1`, `08.1`, `13.2`; `17.2` cannot pass. `07.2`'s literal damage-only footprint scenario is supported, but general sibling-preservation quality is not complete. The fixed channel control/lifetime checks in `10.2` remain valid for their unambiguous tested geometry. `16.1`'s review is complete but current aggregate completion remains open. This precision avoids incorrectly labeling every related legacy condition as a reproduced failure.

## Full coverage and quality judgments

`CONDITION_REVIEW.json` preserves all 34 exact policy literals and policy references, with individual judgments, independent reasons, source anchors and evidence selectors. It also reviews all 17 seams, 10 dimensions and three nodes. All conditions were reviewed after F01 was found. Runtime, public authoring and verification nodes remain unclosed through this same blocker.

The implementation has a coherent typed, immutable effective-plan architecture. Same-effect selection avoids sibling tag union; a finite, stratified closure avoids seedless bootstrap and arbitrary iteration success. Checked composition uses one aggregate scale/truncation step. Legal provider/form publication contexts are evaluated instead of an impossible union of forms. Private generation/provider bindings preserve accepted/released payload owners and do not treat DTOs or serialized runtime facts as authority. The defect is an inconsistent enforcement point inside that architecture.

Actual consumer review included effect-local footprint selection and hit history, per-effect settled HP-loss/result prefixes, generated side source/resource lifetimes, original versus generated target cancellation, personal group lifetime/replacement, fixed recipient identity, and regular-only support eligibility before family winner selection. Personal source creation is evaluated once; recipient count and recipient modifiers do not multiply creation amount. Legacy shield recharge excludes personal groups. Priority, HP-cost bypass, reentry/no-refill, late cancellation and source cleanup have independent numeric/lifetime oracles.

The retained-plan visitor uses exhaustive field destructuring, borrowed traversal, checked extents, capacity rather than just length, and conservative shared owning edges. Its tests vary spare capacity, serde-skipped Flow source/IDs and catalog source payload while preserving observed semantics; they are stronger than a second implementation of the visitor. Native exact/one-short installation, cast, clone and late-fault tests exercise actual admission and staged rollback. Saved primitive configuration is recompiled during Continue rather than accepting a saved effective graph.

The public external tests traverse reward/equip/query/cast and inspect actual damage/protection/support. Original30 plus generated75 causes105 damage; personal query/source amounts and actual450/825 pools are distinguished; support consumes attack × 140/100 with actual contributor provenance; Monk causes Physical40 to each eligible enemy and one Self heal7 after cost10. These fixed oracles do not merely ask the implementation to predict itself. Product023's generated side exemption is narrow: original accepted-target cancellation still applies, and direct024 regressions cover original invalidation, withdrawal, CC and released child ownership.

Canonical documentation delta was reviewed separately from executable source: three files add an explicit offline authoring preset, implemented schema/recipe/query/storage references, and fixture-versus-formal-balance boundaries. They do not claim final Goal completion. The unfulfilled conflict-refusal promise is F01. Final roadmap/Goal completion must not be recorded until a new independent Complete result exists.

## Evidence reuse and preserved limits

`EVIDENCE_REVIEW.json` records all 43 groups with exact original bindings and an independent scope/currentness judgment. Selected named PASS rows from failed parents remain bounded selections. Their enclosing failures, unselected rows and failed commands are preserved. In particular:

- 150 collected named entries are not a fresh whole-suite run.
- The original impact69 is 019 default Core64 plus observation1 and020 server4. The failed earlier server build ran zero tests.
- 022 contains the accepted same-SkillId rv006 row and two failed rv005 rows. Current024 after Product023 contains two literal boundary tests and seven direct regressions, nine exact selections in total.
- Family059 is still 0 passed / 1 failed, exit101. Only its completed closure subcase is reused, supported by its marker, exact exported generation/trial/observed records and fixed assertions. Separate family069/070/071/072 PASS records cover the later representative cases.
- CLI and repaired public comparator evidence are accepted only at the selected successful command level, preserving failed sibling commands.
- The auditor's semantic review of changed producers/consumers supplies reuse reasoning; `source_unchanged` metadata or collector exit0 alone does not.

`PREDECESSOR_REVIEW.json` retains every one of the 254 exact IDs, original literals, digest/pointer references, category, old evidence and changed-component paths. It adds individual independent dispositions informed by the literal and category source review. The 223 inherited/U08 conditions and separate31 R06 rows remain distinguishable. Historical completion is preserved; the 13 rows carrying current completion obligations now accept the actual restore evidence but retain `independent_completion/F01` unresolved. No assertion is made that all original historical selectors were freshly rerun, or that F01 dynamically fails each old condition.

PC evidence is current for the original workload: 576 rows, 36 conditions, 31 successful conditions and five typed refusals. The existing strict comparator requires exact immediate-U07M non-time equality (`allowed_delta=0`) and rejects 17 negative controls. Historical U06 +200 allowance is supplemental; it is not the current threshold. Source136/object152/placement48 layout is preserved. No new baseline or deployment timing guarantee is introduced.

R05 raw execution remains exit101, with the existing concurrent peak-only warning classification. Summary002 exit0 confirms classification, not an all-caps PASS. Event ops44206/retained266102/peak271927 fit original caps; concurrent ops370928 and retained4196928 fit original caps, but peak9709348 exceeds9030188 by679160. Concurrent p95=10014462ns exceeds the observed10ms cap; the original allocation-observer mode has `latency_gate_applicable=false` and the recorded statistical-regression result is false. Neither the warning nor a non-applicable latency gate is rewritten. Shared-host/native historical limits remain. Proposed SkillId allocation/cold-field explanations of event +12/+256/+512 are concrete candidates with exact attribution still unproved.

The explicit offline authoring preset's max_work1073741824 does not change embedded/default/native ceilings (native10m/8MiB) or permit automatic fallback. Formal protection/penalty balance remains user-undecided; fixture values are not approved production numbers. No new Guest type, Unity integration or individual final boss completion is claimed. The lack of a fresh exhaustive suite is not independently treated as a defect under the authorized impact-based scope.

## Disposition

Return F01 to the product owner and independent test owner. Keep this immutable audit and target as the record of the failed completion attempt. A corrected source, focused independent regressions, Sol execution, new exact bindings and frozen target must precede re-audit. No policy change or new user permission is needed merely to enforce the already stated selected-reach conflict contract.
