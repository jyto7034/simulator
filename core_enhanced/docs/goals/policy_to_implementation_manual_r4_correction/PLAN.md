# Policy-To-Implementation Manual R4 Correction

Metadata Schema: 2  
Policy Realization State: `realized`  
Primary Status: `complete`  
Activation Status: `closed`  
Goal Type: `single`  
Implementation Complete: `true`  
Execution Mode: `documentation_only`  
Production Mutation Authorized: `yes`  
Authorization Validity: `consumed`  
Handoff Mode: `author_and_activate`  
Change Type: `governance_only`  
Risk Level: `medium` — single documentation owner; state/authorization semantics affected; direct deterministic validation available  
Reviewer Requirement: `same-implementer-fresh-pass`  
Validation Evidence: `VALIDATION.md` SHA-256 `165c7487c8de42b8d0fb9afe0007c24a018724771fa9f119308449e9377e6df9`  
Reviewer Evidence: `REVIEWER_EVIDENCE.md` SHA-256 `0b8d1eaad0585be6c3333e8c26054face8f17781bbf93e041833db08b8b4b351`  
Canonical Promotion: `CANONICAL_PROMOTION_RECORD.md` SHA-256 `953ca05df8dc0b1f47ca28304429c8aac9c467343bc9a1cebc4dee0db337a337`  
Completion Audit: `COMPLETION_AUDIT.md` SHA-256 `0f1ed9db62bf5635d9ae2a9acf96134720fa22c584f9d4c8ab4566b60da26943`  
Completion Verdict: `Complete / High`

## Objective

Correct the four accepted R3 review findings without changing product policy or unrelated documentation:

1. bind the current revision claim to fresh R4 evidence;
2. give Single Campaigns one reachable terminal audit path;
3. make EV-07 ownership-collision guards exclusive, including indeterminate MH;
4. make canonical publication interruption recovery deterministic for exact previous, exact candidate, and unknown roots.

The correction also clarifies that implementation mutation belongs to PIP-50 while exact audited canonical promotion belongs to PIP-60.

## Scope and acceptance

- Only the authorized manual and this Goal-local evidence directory may change.
- Manual revision is R4 and every routing hint equals its START/END marker range.
- Single Campaign completion closes in PIP-60 with `CC=N/A` and never enters PIP-70.
- EV-07A/B require safe attribution; EV-07C/D cover known MH; EV-07E fails closed when MH cannot be derived.
- Promotion Intent precedes publication; EV-10C1/C2/C3 each have one exclusive observed-root result and owner.
- Ten partitions retain all ten local-contract headings, event IDs remain unique, and no stale operative R3/R2 transition wording remains.

## Frozen audit candidate

Manual: `core_enhanced/docs/policy_to_implementation_manual.md`  
Revision: `PIP-MANUAL-2026-08-13-R4`  
SHA-256: `a310ed3d465f4cf94b753e3bcbfe9e3d1dc259bdae6f7465c6fbdd394fc4983d`  
Size: 1,118 lines / 9,006 words

No further manual edit is allowed without replacing this hash and rerunning the fresh review.
