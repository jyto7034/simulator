# Phase 07 CRA-P06-C05 Experiments

Status: complete
Implementation Complete: true

## 2026-08-09 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's Master execution request remains active and authorizes the single active Child.
- Phase 06 CRA-P05-C01 has a current `Complete / High` audit with improvement class `none`.
- Phase 07 is next in the exact registry; Phases 08~44 remain pending and unauthorized.
- No direct policy ID applies. Weight-sign/duplicate/range authoring decisions remain explicitly out of scope.

### Evidence Carried Into Initialization

- CRA-P06-C05 remains a correctness correction with `2/2/1/2/3/2/unknown = 12/18`.
- Historical defect: candidate comparison uses a non-saturating total while DTO publication separately uses
  `saturating_add`; accepted signed extreme weights can panic/wrap or disagree.
- Required first proof: extreme positive and negative multi-weight cases must make selection and published total use
  the same saturated result under debug/release semantics.
- Preservation boundary: live balance, title/highlight/tie, eligibility/redeployment aggregation and every data/
  DTO/wire/public shape.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 07 coordination changes from `ready` to `active`; no other Child is active.
- Phase 07 production status/source/data investigation: not started.
- Test executions: none.
- Production/test/data/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents completely, capture expanded dirty status and prospective target hashes, then trace
current aggregation/scoring/comparator/DTO and establish nonzero focused baselines.

## 2026-08-09 — E01 Dirty Baseline, Authority Trace And Focused Baseline

### Dirty Baseline

- Repository commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded status after Child initialization: 403 entries; status SHA-256
  `978e35ad4562560e73064817dfc3c573da71c701bf7e80f2b1303e5d75d62e0a`.
- `src/game/battle/result_stats.rs` is clean with SHA-256
  `9597992ca3f5799f5e94ab9b7a3201104c9345e89c9d5c56b9f3cfeceee0822c`.
- `run_policy_data.rs`, world combat tests and `core_runtime_contract.md` already contain user-owned changes and
  were fingerprinted. The Phase can remain isolated to the clean result module plus a separable canonical hunk.

### Current Authority Trace

- `collect_battle_result_stats` aggregates runtime events and participant results into one UUID-sorted employee row,
  then calls private `select_mvp`.
- `select_mvp` calls `compare_mvp_candidates` during `max_by`; the Score tiebreak calls `score_total` separately for
  each comparison. `score_total` saturates each metric product but finishes with ordinary iterator `.sum()`.
- After selection, `select_mvp` independently calls `score_employee`; it uses `saturating_add` and also derives the
  highlight/title. The selected identity can therefore be based on arithmetic different from the published score.
- Metric values are nonnegative `u64` projections converted to `i64::MAX` if needed; weights are accepted signed
  `i64`. Policy validation requires only nonempty weights/tiebreakers, so positive overflow and negative underflow
  are valid current inputs.
- The production live policy uses small positive Damage/Kill/Skill/Survived weights. World result construction calls
  the collector once and server transports the resulting typed field without recomputing MVP.

### Nonzero Baseline

| Command family | Result |
| --- | --- |
| result-stats unit family | 4 passed, 0 failed |
| run-policy unit family | 17 passed, 0 failed |

### Next Experiment

Add separate positive-overflow and negative-underflow candidate cases that assert saturated selected/published
identity, score and highlight; run them against the unmodified comparator before introducing one shared evaluation.

## 2026-08-09 — E02 Extreme Signed Score Red Proof

### Added Characterization Cases

- Positive saturation: candidate 1 has two `i64::MAX` weighted metrics and candidate 2 has one. Both totals must
  saturate to `i64::MAX`, after which the preserved DamageDealt tiebreak selects candidate 1 and publishes that
  same saturated score/highlight.
- Negative saturation: candidate 1 has two `i64::MIN` weighted metrics and candidate 2 has zero. Candidate 2 must
  win Score with zero and publish the first authored zero-weighted highlight.

### Pre-Fix Result

| Build semantics | Result |
| --- | --- |
| debug exact positive case | failed before assertion: ordinary total addition overflowed |
| debug exact negative case | failed before assertion: ordinary total addition overflowed |
| release two-case filter | 0 passed, 2 failed |

- Under release wrapping, the positive case incorrectly selected UUID 2 instead of UUID 1.
- Under release wrapping, the negative case incorrectly selected UUID 1 instead of UUID 2.
- The failures prove that product saturation alone is insufficient and that comparator selection does not share
  the separately saturated published evaluation.

### Next Experiment

Replace the split comparator/publication calculations with one per-candidate saturating evaluation reused by both
selection and DTO construction, without changing tiebreak, highlight, eligibility or public-shape policy.

## 2026-08-09 — E03 One Shared Saturating Evaluation

### Production Change

- `evaluate_mvp_score` now returns one private `MvpScoreEvaluation` containing the explicitly saturating signed
  total and the existing first-best highlight.
- `select_mvp` evaluates each aggregated employee row once, stores that result beside the employee, gives the
  stored total to the unchanged ordered tiebreak loop, and moves the selected stored total/highlight into the DTO.
- The divergent ordinary-`.sum()` helper was removed. `metric_score` continues to saturate the signed product and
  the shared evaluator continues to `saturating_add` terms in authored order.
- Eligibility, aggregation, representative instance, title/highlight replacement, every non-Score tiebreak and
  final lower-UUID fallback remain unchanged.

### Exact Green Proof

| Command family | Result |
| --- | --- |
| debug positive/negative extreme filter | 2 passed, 0 failed |
| release positive/negative extreme filter | 2 passed, 0 failed |
| result-stats family after implementation | 6 passed, 0 failed |

### Compatibility

- No public type, serialized field, policy schema, RON source or live value changed.
- The nearest runtime canonical now states that selection and published score/title/highlight consume the same
  authored-order saturating evaluation.

## 2026-08-09 — E04 Preservation And Broad Validation

### Focused Preservation

- Added an exact equal-candidate regression in both input orders; both select the lower employee UUID and preserve
  score `50` plus title `Survivor`.
- Result-stats final family: 7 passed, 0 failed, covering the two extremes, UUID fallback, ordinary score `160`,
  incapacitated eligibility and chronological multi-instance aggregation.
- Run-policy family: 18 passed, 0 failed.
- PvE bonus-objective consumer family: 3 passed, 0 failed.
- Combat-result world family: 6 passed, 0 failed.
- Live RON family: 63 passed, 0 failed.

### Broad And Static Validation

| Check | Result |
| --- | --- |
| complete Core library | 944 passed, 0 failed, 0 ignored, 0 filtered |
| game_server `cargo check --all-targets` | pass |
| `cargo fmt --check` | pass |
| repository `git diff --check` | pass |
| score arithmetic/caller inventory | one total accumulator; no MVP ordinary `.sum()` remains |
| policy/data/public-shape inventory | signed weights and DTO/RON/server shapes unchanged |

### Dirty Preservation

- Final expanded status contains 405 entries versus 403 at activation; the additional tracked production target is
  the previously clean `result_stats.rs`, while Child documentation and the separable pre-dirty canonical hunk are
  intentional Phase evidence. Final expanded-status SHA-256:
  `94905ae23c2457e227ce1ed8c797decf3824b219322e703090145e0e7093fd8c`.
- Final `result_stats.rs` SHA-256:
  `ea6202a12362ffcc23f556eeb001bc7e125d21907c73bbecd6191966681b2762`.
- No reset, restore, clean, stash, broad formatting or unrelated data mutation was performed.

### Next Experiment

Capture final hashes/status, reread the completion-review guide completely, then perform the independent Phase exit
audit. Do not activate Phase 08 unless the verdict is `Complete / High`.

## 2026-08-09 — E05 Independent Phase Exit Audit

- Re-read all 567 lines of `goal_completion_review_guide.md` before the audit pass.
- Independently rechecked runtime/data/DTO/server/external-consumer/canonical, legacy symbols, scoped diff and dirty
  preservation rather than treating implementation notes as proof.
- Refreshed the exact extreme filters after the UUID-order regression was added: debug 2/2 and release 2/2, each
  with 942 filtered.
- `COMPLETION_AUDIT.md` records every evidence-matrix row as pass, primary verdict `Complete`, fit `High` and
  improvement class `none`.
- Phase 07 status changes to `complete`; Master may make Phase 08 ready.
