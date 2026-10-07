# Boss Runtime v1 Phase 07 — Experiments

Goal: `boss_runtime_v1_implementation_master_phase_07_completion_audit`  
Execution Mode: `audit_only`  
Production Mutation Authorized: no

## 2026-08-09 — Activation

### Evidence

- P05 completion audit is `Complete / High` and explicitly hands off to P07.
- P06 is `not_applicable_to_v1` from actual live serialization, server and current external Unity evidence.
- P00~P05 are all completed and no other child is active.
- Precreation Simulator state is 266 sorted entries with SHA-256
  `c629310e0069b62c40fc1b931668b6464abf0d045f3944b3b9ae45240b37aaf9`.

### Decision

- P07 is active in audit-only mode. Created only its required `PLAN.md`, `EXPERIMENTS.md` and
  `EXPERIMENT_NOTES.md` plus Master coordination records.
- No production, live data, canonical, server or Unity mutation is authorized.
- Final evidence collection starts by reading the policy/coverage sources and completed Phase audits before making a
  verdict.

## 2026-08-09 — Policy, Phase and refactor closure

### Evidence

- Read the complete BP policy record and mechanically confirmed exactly 34 unique headings, BP-01 through BP-34.
- Re-read the readiness coverage/report/boundary/completion artifacts and all P00~P05 completion audits. Every Phase
  verdict remains `Complete / High` and each source/setup/runtime/live seam points to the next Phase's actual owner.
- Correlated all 19 named CRA candidates with the final catalog and readiness boundary: 5 prerequisite, 2
  coimplementation, 4 deferred/conditional and 8 independent. The seven Boss-required/coimplemented candidates are
  complete in P01/P02/P05; the remaining candidates are not falsely claimed implemented.
- Created `FINAL_POLICY_COVERAGE_MATRIX.md` with one final production/test/canonical row for every BP policy.

### Decision

- BP coverage is exact and has no material semantic gap. Future objective repath, selector variants, hot reload/LKG,
  public Boss lifecycle and final content stay strict-reject or explicit future-owner boundaries rather than hidden
  v1 work.

## 2026-08-09 — Final production, canonical and consumer audit

### Evidence

- Production Boss RON contains exactly one Pattern, actor behavior and encounter. Only `suppress_little_red` binds
  `primary_abnormality[0]`, and only its primary wave authors the required route.
- Current source retains one atomic live candidate, full manifested-difficulty/setup feasibility, captured compiled
  setup, private two-layer runtime and existing movement/combat leaf authority.
- `BattleEvent::BossRuntimeWake` has no serde derive and is explicitly excluded from the public event log. Searches of
  game_server and current external Unity source/canonicals find no `boss_runtime` consumer.
- `game_rulebook.md`, `core_runtime_contract.md` and `data_loading_contract.md` name the same disposable Little Red
  reference and its exact behavior/binding. `skill_target_contract.md` correctly retains leaf skill target authority.
- External Unity is still HEAD `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`, 2154 sorted entries, SHA-256
  `d5cb19cf8fbc9ef6664e8b0a0772602853e725adedccdb1f664481824fe20b69`. Its three evidence owners remain exactly
  `acf725a23ddc29b351c4eefc3b4f7e6ac79c9d88d694eaab20d76e92753ab349`,
  `45d13f8fb1a2737ec620867912344fa868816855eefa625cc3efc94a9ca6f599` and
  `24164877c3b2c9c6c57a1874ae31a6d47e5771f8e18136e79da55e0723d6c83f`.

### Decision

- P06 is confirmed `not_applicable_to_v1`. It is neither silently skipped nor treated as implemented; an actual
  future public semantic lifecycle must open a synchronized Core/server/Unity contract.
- No production/canonical correction is needed, so the audit-only Phase may continue toward a complete verdict.

## 2026-08-09 — Independent final validation

### Command and result

- `/usr/bin/time -f 'elapsed=%e max_rss_kb=%M exit=%x' cargo test --workspace --all-targets -- --quiet`
  - Core library: `930 passed`, `0 failed`, `0 ignored`.
  - Core integrations: `3 + 4 + 25 + 13 + 4 passed`, all zero-test binary targets pass.
  - game_server: `56 passed`, `0 failed`, `0 ignored`; zero-test binary/examples pass.
  - Command: exit `0`, elapsed `42.89s`, maximum RSS `102524 KB`.
- `cargo fmt -p game_core -p game_server -- --check`: pass, no output.
- `git diff --check`: pass, no output.

### Decision

- The final broad result is a current P07 rerun after all production/canonical changes, not only reused P05 evidence.
- P07 encountered no new failure. P05's earlier failed/superseded runs and corrections remain durable in P05
  `EXPERIMENTS.md`.

## 2026-08-09 — Completion review

### Evidence

- P07 completion matrix is 10/10 pass and its independent verdict is `Complete / High`.
- Master completion matrix is 14/14 pass and its separate audit verdict is `Complete / High`.
- The only P07 mutations are its audit artifacts and Master coordination/audit artifacts; no production, live RON,
  canonical, server or external Unity file changed during P07.
- Exact final status/hash and artifact subtraction are appended after all audit artifacts exist.

### Decision

- P07 transitions `active` → `complete`; Master transitions `active` → `complete` with no active or required next
  Phase.

## 2026-08-09 — Final dirty subtraction

### Evidence

- Final Simulator status with every P07/Master audit artifact present is exactly `272` sorted entries with SHA-256
  `8d03ae2f1f4777976adc9a0f05c7b493119433f18cc33addd3ff6768fc780481`.
- Excluding exactly these six new audit artifacts restores the P07 precreation baseline exactly: `266` entries,
  SHA-256 `c629310e0069b62c40fc1b931668b6464abf0d045f3944b3b9ae45240b37aaf9`:
  - P07 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, `FINAL_POLICY_COVERAGE_MATRIX.md`,
    `COMPLETION_AUDIT.md`
  - Master `COMPLETION_AUDIT.md`
- Master `PLAN.md`, `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md` already belonged to the precreation status set; P07
  changed only their orchestration records. The unrelated server log remains present.

### Decision

- P07 introduced no unexplained status path and preserved the exact user/campaign baseline entry set outside its six
  new audit artifacts.
