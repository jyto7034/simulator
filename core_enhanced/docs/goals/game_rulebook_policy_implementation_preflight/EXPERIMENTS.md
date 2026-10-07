# Game Rulebook Policy Implementation Preflight — Experiments

## 2026-08-12 — Goal Initialization

### Hypothesis

승인된 readiness 결과는 production 구현과 분리된 single investigation Preflight Goal로 안전하게
구조화할 수 있으며, 필수 문서 3개를 먼저 초기화하면 이후 scoped evidence를 durable하게 기록할 수 있다.

### Performed work

- `goal_authoring_and_execution_guide.md`를 878줄 EOF까지 읽었다.
- Existing Goal naming convention, target path collision, repository root/start commit과 initial dirty state를
  확인했다.
- `game_rulebook_policy_implementation_preflight` 경로가 존재하지 않음을 확인했다.
- Required `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`를 생성했다.

### Exact commands

```text
rg --files -g 'AGENTS.md' -g 'goal_authoring_and_execution_guide.md' -g 'PLAN.md' core_enhanced .agents .codex
wc -l core_enhanced/docs/goal_authoring_and_execution_guide.md
sed -n '1,260p' core_enhanced/docs/goal_authoring_and_execution_guide.md
sed -n '261,560p' core_enhanced/docs/goal_authoring_and_execution_guide.md
sed -n '561,878p' core_enhanced/docs/goal_authoring_and_execution_guide.md
git rev-parse --show-toplevel
git rev-parse HEAD
git status --short
test ! -e core_enhanced/docs/goals/game_rulebook_policy_implementation_preflight
mkdir -p core_enhanced/docs/goals/game_rulebook_policy_implementation_preflight
```

### Result

- Repository root: `/mnt/f/work/simulator`
- Start commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Target path collision: none
- Worktree: broadly dirty before Goal creation; details are summarized in `EXPERIMENT_NOTES.md` and must be
  rechecked before any scoped baseline.
- Tests/probes executed: 0
- Pass/fail/skip: 0/0/0; no test runner was invoked.
- Production changes: none. Only the new Goal directory was created.

### Interpretation

Goal initialization is valid. Current runtime evidence, bootstrap references, runner availability and package
activation verdicts remain `discovery pending`; no completion claim is made.

## 2026-08-12 — Authoring-Guide And Readiness Input Cross-Check

### Hypothesis

Goal skeleton을 repository의 문서 governance, workflow, readiness와 completion rules 및 선행 audit package
boundary에 대조하면 implementation을 선행하지 않고도 Preflight의 필수 artifact, gate와 stop condition을
완전하게 author할 수 있다.

### Performed work

- `README.md`, `engineering_workflow.md`, `policy_implementation_readiness_review_guide.md`와
  `goal_completion_review_guide.md`를 각각 EOF까지 읽었다.
- Existing execution-preflight Phase PLAN을 EOF까지 읽어 investigation-only Preflight의 artifact/gate 관례를
  비교했다.
- Readiness decision log, work-package map, implementation sequence와 bootstrap manifest를 대조했다.
- Preflight required artifacts, package handoff verdict, activation-time baseline rerun 규칙과 known stale
  summary reconciliation item을 PLAN에 보강했다.

### Exact commands

```text
wc -l core_enhanced/docs/README.md core_enhanced/docs/engineering_workflow.md core_enhanced/docs/policy_implementation_readiness_review_guide.md core_enhanced/docs/goal_completion_review_guide.md core_enhanced/docs/goals/boss_runtime_v1_implementation_master_phase_00_execution_preflight/PLAN.md
sed -n '1,159p' core_enhanced/docs/README.md
sed -n '1,240p' core_enhanced/docs/engineering_workflow.md
sed -n '241,407p' core_enhanced/docs/engineering_workflow.md
sed -n '1,270p' core_enhanced/docs/policy_implementation_readiness_review_guide.md
sed -n '271,560p' core_enhanced/docs/policy_implementation_readiness_review_guide.md
sed -n '561,798p' core_enhanced/docs/policy_implementation_readiness_review_guide.md
sed -n '1,310p' core_enhanced/docs/goal_completion_review_guide.md
sed -n '311,567p' core_enhanced/docs/goal_completion_review_guide.md
sed -n '1,252p' core_enhanced/docs/goals/boss_runtime_v1_implementation_master_phase_00_execution_preflight/PLAN.md
sed -n '1,305p' core_enhanced/docs/goals/game_rulebook_policy_implementation_readiness_audit/POLICY_DECISION_LOG.md
sed -n '1,173p' core_enhanced/docs/goals/game_rulebook_policy_implementation_readiness_audit/WORK_PACKAGE_MAP.md
sed -n '1,200p' core_enhanced/docs/goals/game_rulebook_policy_implementation_readiness_audit/IMPLEMENTATION_SEQUENCE.md
sed -n '1,190p' core_enhanced/docs/goals/game_rulebook_policy_implementation_readiness_audit/CONTENT_INPUT_CANDIDATES.md
```

### Result

- Required documents/guides were read from the current dirty working tree, not assumed from HEAD.
- Single investigation Goal remains appropriate; no Master/Phase or production activation was authorized.
- Package order remains `WP002 → WP001 → WP003 → WP004`; WP005 remains future/dormant.
- `CONTENT_INPUT_CANDIDATES.md` final Bootstrap Resolution contains a stale Option B sentence that conflicts with
  its detailed PD005 resolution and current rulebook/decision log. This is recorded for `PF-01`; it does not reopen
  the approved WhiteNight bootstrap policy.
- Tests/probes executed: 0
- Pass/fail/skip: 0/0/0; no runner was invoked.
- Production changes: none. Only the new Goal documents were edited.

### Interpretation

The Goal is authoring-complete but execution remains active and unstarted. Supplemental result artifacts must be
created only after their bounded investigation units produce evidence.

## 2026-08-12 — Goal Authoring Validation

### Hypothesis

The initialized Goal contains the three required documents, all mandatory PLAN sections, exact policy/package
source coverage and no malformed primary link or trailing whitespace.

### Exact command

```text
goal_dir=core_enhanced/docs/goals/game_rulebook_policy_implementation_preflight
source_dir=core_enhanced/docs/goals/game_rulebook_policy_implementation_readiness_audit
find "$goal_dir" -maxdepth 1 -type f -printf '%f\n' | sort
rg '^\| `GRB-PD-[0-9]{3}` \|' "$goal_dir/PLAN.md" | wc -l
rg '^\| `GRB-PD-[0-9]{3}` \|' "$source_dir/POLICY_DECISION_LOG.md" | wc -l
rg '^## GRB-WP-[0-9]{3} ' "$source_dir/WORK_PACKAGE_MAP.md" | wc -l
rg '^## (Objective|User/Contract-Visible Problem|Audit Finding ID|Current Runtime And Live Data Evidence|Source Of Truth And Ownership|Invariants|Approved Policy And Evidence|Open Policy Decisions|In Scope|Out Of Scope|Implementation Scope|DTO/Data/Save Migration|Deployment And Compatibility|Focused Validation|Broad Validation|Omitted Validation|Completion Conditions|Completion Audit Evidence Matrix|Stop Conditions)$' "$goal_dir/PLAN.md" | wc -l
rg -n '[[:blank:]]+$' "$goal_dir" | wc -l
test -f "$source_dir/PLAN.md"
git status --short -- "$goal_dir" | wc -l
```

### Result

- Required initial files: 3 (`PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`)
- PLAN policy rows/source decision rows: 10/10
- Source work-package sections: 5
- Mandatory PLAN sections found: 19/19
- Trailing whitespace findings: 0
- Broken primary readiness link: 0
- Scoped Git status entries: 1 untracked Goal directory
- Tests/probes executed: 0
- Pass/fail/skip: 0/0/0; this was a static document validation, not a test runner.
- Production changes: 0

### Interpretation

The Preflight Goal document set is structurally ready for a later execution request. Status remains `active` and
`Implementation Complete: not_applicable` because PF-01~PF-08 have not been executed.

## 2026-08-12 — ATI-05 Lifecycle Reconciliation

### Hypothesis

병합된 `policy_implementation_readiness_review_guide.md`의 ATI lifecycle을 현재 active Preflight Goal에만
적용하면 과거 completed Goal을 수정하지 않고 entry evidence, exit gate, rewind와 activation authority를
명확히 할 수 있다.

### Performed work

- Active Preflight의 필수 문서 3개를 EOF까지 다시 읽었다.
- PLAN을 `ATI-05 — Execution Preflight`로 명시하고 entry gate `sequence_safe`, exit gate
  `ready_to_activate`, first-package handoff `ready_for_wp002`를 분리했다.
- ATI-01~04 entry evidence를 current proof가 아닌 input pointer로 분류하고 PF-01 current-drift 확인을
  gate로 추가했다.
- PF-01~08을 merged guide의 drift/collision, transition/atomicity/lifetime, bootstrap, runner와 activation
  기준에 맞췄다.
- Policy, boundary, dependency, runner와 dirty collision별 rewind owner를 PLAN에 추가했다.
- Completed readiness/alignment Goal과 다른 과거 Goal은 수정하지 않았다.

### Exact commands

```text
wc -l core_enhanced/docs/goals/game_rulebook_policy_implementation_preflight/PLAN.md core_enhanced/docs/goals/game_rulebook_policy_implementation_preflight/EXPERIMENTS.md core_enhanced/docs/goals/game_rulebook_policy_implementation_preflight/EXPERIMENT_NOTES.md
sed -n '1,360p' core_enhanced/docs/goals/game_rulebook_policy_implementation_preflight/PLAN.md
sed -n '1,300p' core_enhanced/docs/goals/game_rulebook_policy_implementation_preflight/EXPERIMENTS.md
sed -n '1,260p' core_enhanced/docs/goals/game_rulebook_policy_implementation_preflight/EXPERIMENT_NOTES.md
```

### Result

- Modified Goal scope: current active `game_rulebook_policy_implementation_preflight` only
- Historical Goal modifications: 0
- Tests/probes executed: 0
- Pass/fail/skip: 0/0/0; this pass reconciled planning documents and invoked no runner.
- Production changes: 0

### Interpretation

The active Goal now follows the merged lifecycle guide. This is a document reconciliation only; ATI-05 execution
and its `ready_to_activate` verdict remain pending.

## 2026-08-12 — ATI-05 Document Validation

### Hypothesis

The active Goal contains the merged guide's ATI-05 stage metadata, four prior-stage entry rows, eight Preflight
work units, all ten policy inputs and all required exit verdicts without stale standalone-guide references.

### Exact command

```text
goal_dir=core_enhanced/docs/goals/game_rulebook_policy_implementation_preflight
rg --files "$goal_dir" | wc -l
rg -n '^Lifecycle Stage: `ATI-05|^Entry Gate:|^Exit Gate:' "$goal_dir/PLAN.md" | wc -l
rg '^\| `ATI-0[1-4] /' "$goal_dir/PLAN.md" | wc -l
rg '^\| `PF-[0-9]{2}` \|' "$goal_dir/PLAN.md" | wc -l
rg '^\| `GRB-PD-[0-9]{3}` \|' "$goal_dir/PLAN.md" | wc -l
for value in 'Complete / High' ready_to_activate ready_for_wp002; do rg -q "$value" "$goal_dir/PLAN.md"; done
rg -n 'policy_alignment_to_implementation_guide' "$goal_dir" | wc -l
rg -n '[[:blank:]]+$' "$goal_dir" | wc -l
git status --short -- "$goal_dir"
```

### Result

- Active Goal files: 3
- ATI-05/entry/exit metadata rows: 3/3
- ATI-01~04 entry rows: 4/4
- PF work-unit rows: 8/8
- Policy rows: 10/10
- Required verdict terms: all present
- Stale standalone-guide references: 0
- Trailing whitespace findings: 0
- Scoped status: one untracked active Goal directory
- Tests/probes executed: 0
- Pass/fail/skip: 0/0/0; static document validation only.

### Interpretation

The active Goal documents are structurally aligned with ATI-05. No Preflight evidence artifact has been executed
or completed yet, so Status remains `active` and activation remains denied pending PF-01~PF-08.

## 2026-08-12 — Unity Preflight Validation Scope Override

### Hypothesis

Unity project가 현재 별도 대규모 개발 drift를 가진 상태라면 사용자의 명시적 범위 결정에 따라 이번
ATI-05에서 current-tree/runner 검사를 제외하되, 검증하지 않은 consumer를 검증했다고 표현하지 않고
Core/live RON/game_server evidence를 독립적으로 계속 수집할 수 있다.

### Performed work

- 사용자는 Unity 개발 버전이 많이 어긋나 있고 손댈 범위가 많으므로 이번에는 Unity를 검사할 필요가
  없다고 결정했다.
- PLAN의 validation, consumer, migration, completion과 stop-condition 경계를 이 결정에 맞게 갱신했다.
- 중단된 Unity runner 요청 뒤 실행 중인 Unity process와 의도했던 workspace result artifact가 남았는지
  read-only로 확인했다.

### Exact commands

```text
ps -eo pid,ppid,args | rg '[U]nity(\.exe)?|[U]nityHub'
find tmp -maxdepth 1 -type f -name 'game_rulebook_preflight_editmode*' -printf '%f %s bytes\n'
```

### Result

- Unity current-tree/source, EditMode/PlayMode와 live client probe: explicitly omitted by user for this
  Preflight
- Completed Unity tests/probes: 0
- Pass/fail/skip: not reported; no completed Unity runner result exists
- Residual Unity process: none observed
- Intended workspace result artifact: none observed; `tmp` directory did not exist
- Production changes: none

### Interpretation

이번 Preflight는 Unity 일치를 입증하지 않는다. Unity 미검증은 사용자가 승인한 명시적 validation
비범위이므로 그 자체로 `ready_for_wp002`를 막지 않는다. 실제 production package의 Unity 포함 여부와
검증 범위는 해당 구현 Goal activation 시 별도로 확정해야 하며, 이번 제외는 compatibility/fallback을
승인하지 않는다.

## 2026-08-12 — PF-01 Current-Tree Drift And Collision Reconciliation

### Hypothesis

ATI-01~04 input을 current tree와 scoped하게 대조하면 승인 정책이나 package graph를 되돌리지 않고도
현재 구현 gap과 dirty ownership collision을 구분할 수 있다.

### Exact commands

```text
git rev-parse HEAD
git status --short -- <WP002/WP001/WP003/WP004 Core, RON, server and Goal paths>
sha256sum core_enhanced/docs/game_rulebook.md \
  core_enhanced/docs/goals/game_rulebook_policy_implementation_readiness_audit/PLAN.md \
  core_enhanced/docs/goals/game_rulebook_policy_implementation_readiness_audit/POLICY_DECISION_LOG.md \
  core_enhanced/docs/goals/game_rulebook_policy_implementation_readiness_audit/WORK_PACKAGE_MAP.md \
  core_enhanced/docs/goals/game_rulebook_policy_implementation_readiness_audit/IMPLEMENTATION_SEQUENCE.md \
  core_enhanced/docs/goals/game_rulebook_policy_implementation_readiness_audit/CONTENT_INPUT_CANDIDATES.md
rg -l '^Status: active$' core_enhanced/docs/goals/*/PLAN.md
rg -n 'stage_[23]_(stat|reward)_multiplier_percent' \
  <(git diff -- game_resources/data/run/policy.ron core_enhanced/src/game/data/run_policy_data.rs)
rg -n 'ACTIVE_RUN_(CORE_PAYLOAD_)?VERSION|PROFILE_VERSION' core_enhanced/src game_server/src
```

### Result

- HEAD and evidence hashes match the freeze recorded in `PREFLIGHT_MATRIX.md`.
- Only this simulator Goal has `Status: active`.
- Relevant production paths are broadly dirty, but the current diff has 0 Monolith target-field hunk hits.
- Current live Monolith stat values remain 100/125; Core/server active-run remain V3; profile store remains V1.
- Research/pending and old BossOmen gate remain live; target successor ledger/schema are absent.
- Tests/probes executed: 0; this was static source/data/diff evidence.
- Production changes: 0.

### Interpretation

The gaps are the already-approved implementation target, not new policy drift. No ATI-01~04 rewind is required.
WP002 can receive `ready_to_activate` with an activation-time exact-hunk recheck; broader packages retain their
prerequisite and dirty-merge gates.

## 2026-08-12 — PF-04 Bootstrap Referential Integrity

### Hypothesis

The minimum 29-source and ST1 bootstrap inputs can be separated into current strict-valid references and explicit
package-owned target authoring without substituting missing content.

### Exact command

```text
set -euo pipefail
manifest=core_enhanced/docs/goals/game_rulebook_policy_implementation_readiness_audit/CONTENT_INPUT_CANDIDATES.md
rows=0
missing_encounters=0
missing_abnormalities=0
missing_fragments=0
while IFS='|' read -r _ class source encounter abnormality fragment _; do
  class=$(printf '%s' "$class" | tr -d ' `')
  case "$class" in Elite|NormalBoss|FinalBoss) ;; *) continue ;; esac
  source=$(printf '%s' "$source" | tr -d ' `')
  encounter=$(printf '%s' "$encounter" | tr -d ' `')
  abnormality=$(printf '%s' "$abnormality" | tr -d ' `')
  fragment=$(printf '%s' "$fragment" | tr -d ' `')
  rows=$((rows + 1))
  rg -q -F "id: \"$encounter\"" game_resources/data/pve/encounters.ron || missing_encounters=$((missing_encounters + 1))
  rg -q -F "id: \"$abnormality\"" game_resources/data/abnormalities/base.ron || missing_abnormalities=$((missing_abnormalities + 1))
  rg -q -F "id: \"$fragment\"" game_resources/data/skill_fragments/base.ron || missing_fragments=$((missing_fragments + 1))
done < "$manifest"
printf 'manifest rows=%d missing_encounters=%d missing_abnormalities=%d missing_fragments=%d\n' \
  "$rows" "$missing_encounters" "$missing_abnormalities" "$missing_fragments"
rg -c 'encounter_class: Elite|encounter_class: NormalBoss|encounter_class: FinalBoss' \
  game_resources/data/pve/encounters.ron
rg -c 'BasicAttackModifier' game_resources/data/skill_fragments/base.ron
```

Additional exact-ID loops checked the three proposed tutorial pools and every reused ST1 identity.

### Result

- Manifest: 29 rows; missing encounter/abnormality/fragment references: 0/0/0.
- Current PVE class counts: Elite 13, NormalBoss 12, FinalBoss 5.
- Current live BossOmen chains/WhiteNight rows: 1/1.
- Reused ST1 identities missing: 0.
- New target-authored tutorial pools absent: exactly 3 (`tutorial_f1_training_budget`,
  `tutorial_f3_salvage`, `tutorial_f3_budget`).
- Current target source/repeat field hits: 0, as expected before WP001.
- Temporary `BasicAttackModifier` count: 29.
- Tests/probes executed: 0; strict read-only catalog searches only.
- Production changes: 0.

### Interpretation

Current IDs are sufficient to start target authoring. The three new pools and target schema fields are exact WP003
and WP001 deliverables, not missing current references to replace. Silent Orchestra/Apocalypse Bird remain dormant;
the one-step WhiteNight chain is the only live FinalBoss membership.

## 2026-08-12 — PF-06 Core, Live-RON And game_server Baseline

### Hypothesis

The current dirty tree has executable non-zero Core/live-data/server runners whose exact counts can anchor later
package activation without claiming the unimplemented target behavior.

### Exact commands

```text
cargo test -p game_core
cargo test -p game_server
cargo test -p game_core -- --quiet
cargo test -p game_server -- --quiet
cargo test -p game_core data::run_policy_data -- --quiet
```

### Result

- Focused run-policy unit tests: 17 passed, 0 failed, 0 ignored; 995 lib and 51 integration tests filtered.
- Full Core library: 1012 passed, 0 failed, 0 ignored.
- Core integration binaries: 4 + 4 + 25 + 14 + 4 = 51 passed, 0 failed, 0 ignored.
- Full Core actual total: 1063 passed, 0 failed, 0 ignored; doc tests 0.
- game_server library: 63 passed, 0 failed, 0 ignored; main/doc tests 0.
- Full commands were repeated with quiet output to capture exact totals; both runs were green.
- Unity tests/probes: 0, explicitly excluded by user; no result claimed.
- Production changes: 0.

### Interpretation

PF-06 is complete. Green baselines prove the current pre-cutover tree and live bundle load; tests that assert research,
generated Standard or the old BossOmen gate are legacy baseline evidence and must be replaced by target-first tests
inside the owning package.

## 2026-08-12 — PF-02/PF-03/PF-05/PF-07 Execution Dossier

### Hypothesis

Closed decisions and the current evidence can be normalized into exact traceability, transition, atomicity,
migration and package gates without designing final content or changing production.

### Performed work

- Created `PREFLIGHT_MATRIX.md` after the current-state investigation.
- Added one primary row for each of 10 decisions, 15 findings and 5 packages.
- Specified success, defeat, retry, retreat, unvisited, cycle refill, late failure, reconnect and checkpoint rows.
- Separated run/checkpoint state, permanent monotonic profile and presentation-only projection.
- Defined Monolith safe range as exact representability/no saturation, not an arbitrary gameplay cap.
- Resolved “sweep item duration” as the already-active generic consumable modifier contract; no special sweep item.
- Fixed migration/consumer owners and explicit activation verdicts through WP005.

### Result

- PF-01~PF-05 and PF-07 artifact status: complete.
- ATI rewind required: no.
- First-package handoff: `GRB-WP-002` `ready_to_activate`.
- Overall secondary handoff candidate: `ready_for_wp002`.
- Tests/probes executed in this documentation step: 0.
- Production changes: 0.

### Interpretation

Only PF-08 completion audit and final document validation remain before the Goal can claim its exit gate.

## 2026-08-12 — PF-08 Independent Completion Audit And Final Validation

### Hypothesis

The completed dossier satisfies all 12 PLAN completion conditions, and an independent guide-based audit plus
structural validation can prove `Complete / High`, `ready_to_activate` and `ready_for_wp002` without claiming target
production implementation.

### Performed work

- Applied `goal_completion_review_guide.md` in `goal_exit` mode.
- Audited runtime/data/DTO/persistence/consumer, legacy, compatibility, dirty preservation, canonical alignment,
  cross-component boundaries and long-term fit.
- Connected each of the 12 PLAN completion conditions to actual static/runner/artifact evidence.
- Created `COMPLETION_AUDIT.md` and ran exact file/ID/link/whitespace/scoped-diff checks.
- Changed only the current active Goal PLAN from `active` to `complete`; historical Goal documents were untouched.

### Exact validation command

```text
set -euo pipefail
goal=core_enhanced/docs/goals/game_rulebook_policy_implementation_preflight
find "$goal" -maxdepth 1 -type f -printf '%f\n' | sort
rg '^\| `GRB-PD-[0-9]{3}` \|' "$goal/PREFLIGHT_MATRIX.md" | wc -l
rg '^\| `GRB-M-[0-9]{3}` \|' "$goal/PREFLIGHT_MATRIX.md" | wc -l
rg '^\| `GRB-WP-[0-9]{3}` \|' "$goal/PREFLIGHT_MATRIX.md" | wc -l
rg -o '^\| (EL|MON|NB|BO|ST|EN|PR|AT|DS|SW|RW|TX|CP)-[0-9]{2}' \
  "$goal/PREFLIGHT_MATRIX.md" | sort | uniq -d
rg '^\| (BOOT|MIG|CON)-[0-9]{2} \|' "$goal/PREFLIGHT_MATRIX.md"
rg '^\| C-[0-9]{2} \|' "$goal/COMPLETION_AUDIT.md" | wc -l
while IFS= read -r link; do
  case "$link" in http:*|https:*|mailto:*|'#'*) continue ;; esac
  target=${link%%#*}
  test -e "$goal/$target"
done < <(awk 'BEGIN { fenced=0 } /^```/ { fenced=!fenced; next } !fenced { print }' \
  "$goal"/*.md | rg -o '\]\([^)]+' | sed 's/.*](//' | sort -u)
test -z "$(rg -n '[[:blank:]]+$' "$goal" || true)"
git diff --check -- "$goal"
git status --short -- "$goal"
```

### Result

- Required artifacts: 6/6.
- Primary decision/finding/package rows: 10/15/5.
- Transition rows: 55; duplicate transition IDs: 0.
- Bootstrap/migration/consumer rows: 11/6/5.
- Completion-condition rows: 12/12; all pass.
- Relative links: pass.
- Trailing whitespace: 0.
- Scoped diff check: pass.
- Scoped status: one untracked active-Goal directory containing only the six approved documents.
- Tests run in this static final validator: 0; runner totals remain PF-06 Core 1063 and server 63, all green.
- Production changes: 0.

### Interpretation

PF-08 and the ATI-05 exit gate are complete. Primary verdict is `Complete / High`, lifecycle result is
`ready_to_activate`, and the only immediate handoff is `ready_for_wp002`. Production remains unauthorized until the
user separately activates an implementation Goal.
