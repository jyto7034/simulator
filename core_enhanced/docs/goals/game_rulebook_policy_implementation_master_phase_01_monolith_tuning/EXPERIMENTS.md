# Game Rulebook Policy Implementation Master Phase 01 — Experiments

## 2026-08-12 — Phase Activation And Document Initialization

### Hypothesis

Completed Preflight와 active Master가 P01을 유일한 ready Phase로 지정했으므로, 이번 사용자 실행 요청으로
P01을 활성화하고 production 조사 전 필수 문서 3개를 만들 수 있다.

### Performed work

- 첨부된 existing Goal/Child resume 지침을 먼저 EOF까지 읽었다.
- Master 필수 문서 3개를 EOF까지 읽고 Status/registry/stop conditions를 확인했다.
- Direct dependency인 Preflight completion audit와 P01/WP002 matrix rows를 읽었다.
- Child path collision과 known target files의 scoped status를 확인했다.
- P01 필수 문서 3개를 생성하고 Master registry를 P01 active로 전환했다.

### Exact commands

```text
sed -n '1,240p' <pasted-text-1.txt>
sed -n '1,180p' <master>/PLAN.md
sed -n '181,380p' <master>/PLAN.md
sed -n '1,220p' <master>/EXPERIMENTS.md
sed -n '1,220p' <master>/EXPERIMENT_NOTES.md
test ! -e <P01 child directory>
git status --short -- <Master> <P01> core_enhanced/src/game/data/run_policy_data.rs game_resources/data/run/policy.ron
sed -n '1,220p' <Preflight>/COMPLETION_AUDIT.md
sed -n '1,120p' <Preflight>/PREFLIGHT_MATRIX.md
sed -n '241,315p' <Preflight>/PREFLIGHT_MATRIX.md
mkdir -p <P01 child directory>
```

### Result

- Master Status: active; previous active Phase: none; P01: ready.
- Direct dependency: Preflight `Complete / High`, `ready_for_wp002`.
- Child path collision: none.
- Target files: both modified before Phase; exact current hunk meaning is not yet inspected.
- Tests/probes: 0; pass/fail/skip 0/0/0.
- Production changes: 0. Child/Master documentation only.

### Interpretation

P01 document gate is complete. Production modification remains contingent on exact dirty-hunk review and current
focused/full activation baseline.

## 2026-08-12 — Dirty-Hunk Review And Activation Baseline

### Hypothesis

Current user changes can coexist with P01 if they do not touch the Monolith fields/validation/resolver, and the
then-current tree must pass non-zero focused and full Core runners before target-first mutation.

### Performed work

- Inspected scoped diffs for `run_policy_data.rs` and live `run/policy.ron`.
- Read current Monolith validation, resolver, role tuning, preview/stat-scale path and canonical policy.
- Executed focused run-policy and full Core baselines.

### Exact commands

```text
git diff --unified=20 -- core_enhanced/src/game/data/run_policy_data.rs game_resources/data/run/policy.ron
git diff --unified=0 -- core_enhanced/src/game/data/run_policy_data.rs game_resources/data/run/policy.ron
sed -n '500,620p' core_enhanced/src/game/data/run_policy_data.rs
sed -n '820,895p' core_enhanced/src/game/data/run_policy_data.rs
sed -n '1025,1125p' core_enhanced/src/game/data/run_policy_data.rs
rg -n -C 18 'monolith:' game_resources/data/run/policy.ron
rg -n 'stage_[23]_(stat|reward)_multiplier_percent|MonolithRemanifestation|percent_product_ceil' core_enhanced/src game_resources/data/run
sha256sum core_enhanced/docs/game_rulebook.md
cargo test -p game_core data::run_policy_data -- --quiet
cargo test -p game_core -- --quiet
```

### Result

- Existing target-file changes concern tactical resource/stance/relocation only; Monolith target fields,
  validation and resolver have zero existing changed hunks.
- Canonical SHA-256 remains Preflight value
  `d222e4cc937c972ed13472901e1dd6aa91d46f8c1d3b64b9d9311ea572d80b53`.
- Current live Monolith stat values: 100/125.
- Current resolver composes target-power/HP/attack/defense percentages and silently saturates non-representable
  outputs to `u32::MAX`.
- Focused: 17 passed, 0 failed, 0 ignored; 995 unit and 51 integration tests filtered.
- Full Core: 1012 unit + 51 integration = 1063 passed, 0 failed, 0 ignored; doc tests 0.
- Production changes in this unit: 0.

### Interpretation

No activation stop condition fired. P01 may add tests and modify only the non-overlapping Monolith hunks while
preserving the existing tactical changes.

## 2026-08-12 — Target-First Implementation And Validation

### Hypothesis

Live 150/200 plus checked composition at the publication and resolver boundaries will satisfy M006 without changing
reward/wave semantics or overwriting pre-existing tactical work.

### Performed work

- Added target-first tests for live values, zero/equal/reversed inputs, both stages × four non-representable stat
  branches, exact derived tuning and actual combat-preview scaling.
- Ran the focused suite before implementation and captured the expected red result.
- Changed only the Monolith RON values and validation/resolver hunks.
- Re-ran focused, direct preview and full Core suites; applied formatting only to the two lines identified by the
  read-only formatter check.

### Exact commands

```text
cargo test -p game_core data::run_policy_data -- --quiet
cargo fmt --all -- --check
cargo test -p game_core monolith_stage_changes_actual_enemy_scale_without_changing_waves -- --quiet
cargo test -p game_core -- --quiet
git diff --check -- <P01 scoped files>
```

### Result

- Target-first red: 19 tests; 16 passed, 3 failed, 0 ignored. Exact failures were old live value, accepted overflow
  and old derived values.
- Final focused: 19 passed, 0 failed, 0 ignored.
- Direct combat preview: 1 passed, 0 failed, 0 ignored.
- Final full Core: 1014 unit + 51 integration = 1065 passed, 0 failed, 0 ignored; doc tests 0.
- Format check and scoped diff check: pass.
- Live stat values: 150/200; reward values remain 100/125.
- P01 production changes: two Monolith RON fields, strict stat-composition validation/resolution and direct tests.

### Interpretation

Implementation meets the focused and broad gates. Completion audit may now independently evaluate the Phase.

## 2026-08-12 — Phase Completion Audit

### Hypothesis

Current code/data/tests/canonical and scoped history prove all nine completion conditions without a material
consumer, compatibility or dirty-preservation gap.

### Performed work

- Read `goal_completion_review_guide.md` to EOF and applied `phase_exit` mode.
- Audited loader/runtime/data, absent DTO/save boundary, consumer disposition, legacy, canonical and dirty scope.
- Created `COMPLETION_AUDIT.md` and connected all conditions to current evidence.

### Result

- Completion conditions: 9/9 pass.
- Primary verdict: `Complete / High`.
- Status transition: active/false → complete/true.
- Next handoff: P02 may become ready; P02 production was not started in this audit unit.

### Interpretation

P01 is complete. Master may record the result and open P02 as the sole next candidate.
