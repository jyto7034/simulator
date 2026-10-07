# Core Enhanced Refactoring Post-Boss Runtime Reaudit Phase 00 Experiments

Status: complete

## 2026-08-09 — Phase initialization preflight

### Hypothesis

Master Phase 0이 `ready`이고 direct predecessor가 없으며, 등록된 child 경로에 충돌이 없다면 필수 문서
3개를 먼저 초기화한 뒤 current-tree inventory를 시작할 수 있다.

### Performed checks

- Master 필수 문서 3개를 처음부터 끝까지 읽었다.
- Master Phase registry에서 Phase 0의 dependency가 `Master initialized`, status가 `ready`임을 확인했다.
- Phase 0에는 직접 선행 Goal 또는 completion review가 없음을 확인했다.
- current authoring/workflow/completion-review guide를 처음부터 끝까지 읽었다.
- 등록된 Phase 0 경로의 부재, repository HEAD와 `git status --short`를 확인했다.

### Exact commands

```text
wc -l <Master PLAN.md> <Master EXPERIMENTS.md> <Master EXPERIMENT_NOTES.md>
sed -n '<full segmented ranges>' <each Master document>
wc -l core_enhanced/docs/goal_authoring_and_execution_guide.md core_enhanced/docs/engineering_workflow.md core_enhanced/docs/goal_completion_review_guide.md
sed -n '<full segmented ranges>' <each guide>
test ! -e core_enhanced/docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_phase_00_current_tree_delta_inventory
git rev-parse HEAD
git status --short
```

### Actual result

- Master는 `active`, Phase 0은 `ready`, 이후 Phase는 `pending`이다.
- registered Phase 0 경로는 초기화 직전 존재하지 않았다.
- HEAD는 `f318414fb0156a67e04c5479000c2a2683fc0c09`다.
- current tree에는 modified Core source/test/canonical, live data, server source와 untracked Boss/tactical source,
  기존 Goal 문서 및 unrelated server log가 존재한다.
- 모든 pre-existing change는 사용자 소유이며 P00은 이를 수정하지 않는다.

### Test result

Test runner는 실행하지 않았다. 실행 test 수, pass/fail/skip은 모두 `not_applicable`이다. Phase 문서
초기화 전에는 runtime 조사나 test가 허용되지 않았다.

### Decision

Phase 0 필수 문서 3개를 `investigation_only`, production mutation `no`로 초기화하고 Master registry를
`active`로 전환한 뒤 inventory를 시작한다.

## 2026-08-09 — Current-tree baseline capture

### Hypothesis

Current path/status identity와 relevant production content manifest를 함께 고정하고 new reaudit artifact를
제외한 상태가 Boss completion baseline과 같다면, large untracked tree에서도 후속 Phase가 drift와 문서
추가를 구분할 수 있다.

### Exact commands

```text
git status --short --untracked-files=all | LC_ALL=C sort | sha256sum
git status --short --untracked-files=all | wc -l
git status --short --untracked-files=all | awk '{k=substr($0,1,2); count[k]++} END {for (k in count) print k, count[k]}' | LC_ALL=C sort
git status --short --untracked-files=all | rg -v '<P00 path>' | LC_ALL=C sort | sha256sum
git status --short --untracked-files=all | rg -v '<Master or P00 path>' | LC_ALL=C sort | sha256sum
git status --short --untracked-files=all | cut -c4- | rg '<relevant production/current-method path expression>' | LC_ALL=C sort | xargs sha256sum | LC_ALL=C sort | sha256sum
sha256sum <18 Boss-critical current files>
git -C '/mnt/f/unity projects/ark' rev-parse HEAD
git -C '/mnt/f/unity projects/ark' status --short --untracked-files=all | LC_ALL=C sort | sha256sum
git -C '/mnt/f/unity projects/ark' status --short --untracked-files=all | wc -l
```

### Actual result

- P00 documents initialized: 278 status entries = 86 modified + 192 untracked files; sorted status hash
  `1bb7417418cd38f62719d7021f98876fcae657b6a2c4261aa7e212f6ccacbb96`.
- Filtering three initial P00 files: 275 entries, hash
  `43f4ae0861a80f8f800fd82a676930ef10822c1e71a9cbe82b3ac5284fea17a3`.
- Filtering the entire new reaudit campaign: 272 entries, exact historical Boss completion hash
  `8d03ae2f1f4777976adc9a0f05c7b493119433f18cc33addd3ff6768fc780481`.
- Relevant current production/method/canonical manifest: 96 files, hash
  `9638cfa2d05d8d04966cf49fde7abde79b318a25a6452e38c04fc3ce3284b284`.
- External Unity: HEAD `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`, 2156 status entries, hash
  `951364f7944787d594040e71707548d2a922887d9c5a23b9d0c982e5aac2e6d2`.

### Test result

No test runner was executed. Test count and pass/fail/skip are `not_applicable`; these commands validate file/status
identity, not runtime behavior.

### Decision

`CURRENT_TREE_BASELINE.md` owns the reproducible commands, category rules and individual Boss-critical hashes.
Boundary Phases must compare the 96-file content manifest and campaign-excluded 272-entry status baseline.

## 2026-08-09 — Original candidate and Boss set reconciliation

### Hypothesis

The 2026-08-07 final matrix should contain 51 unique rows consuming 53 source IDs, and the Boss completion claim
should expand to 19 unchanged final IDs with the other 32 still requiring current impact review.

### Exact commands

```text
rg '^\| CRA-(F|P)[^|]+\|' <FINAL_CANDIDATE_CATALOG.md> | wc -l
rg '^\| CRA-(F|P)[^|]+\|' <FINAL_CANDIDATE_CATALOG.md> | cut -d'|' -f2 | tr -d ' ' | LC_ALL=C sort | uniq -d
rg -o 'CRA-P[0-9]{2}-C[0-9]{2}' <phase candidate catalogs> | sed 's/.*://' | LC_ALL=C sort -u | wc -l
for id in <expanded Boss 19>; do rg -q "^\| $id \|" <FINAL_CANDIDATE_CATALOG.md>; done
comm -3 <sorted final IDs> <sorted P00 inventory IDs>
```

### Actual result

- Final rows: 51; duplicate final ID output: none.
- Unique detailed source IDs: 53.
- Merge map: F-C01 consumes P02-C02/P03-C01; F-C02 consumes P03-C05/P04-C02.
- Expanded Boss set: 5 prerequisite + 2 coimplemented + 4 deferred/conditional + 8 independent = 19; all present.
- Other current-tree impact set: 32.
- P00 inventory rows: 51; `comm -3` between final and P00 IDs produced no output.

### Test result

No runtime test runner was executed. The exact text checks reported cardinality/membership only; pass/fail/skip test
counts are `not_applicable`.

### Decision

`ORIGINAL_CANDIDATE_INVENTORY.md` freezes every identity, historical work type/readiness, Boss relation and reaudit
owner. Historical disposition is not current closure evidence.

## 2026-08-09 — Boss authority, test and consumer surface inventory

### Hypothesis

Current source should expose enough authority→producer→consumer seams to assign P01~P10 without assuming the
historical Boss audit is still correct or creating a candidate from module shape.

### Performed checks

- Read current Boss raw schema/compiler, embedded source/atomic candidate, PVE binding, typed setup, scenario,
  private runtime, wake ordering, staged commit, movement/action arbitration and leaf delegation source.
- Read the production Boss RON and exact `suppress_little_red` binding.
- Searched current Core tests, server source/tests and accessible Unity Assets/canonicals for actual Boss consumers.
- Read current Boss sections in data-loading, runtime and gameplay canonicals.
- Read the original final test cleanup and cross-boundary artifacts.

### Actual result

- Current path is live RON → compiled immutable definitions → resolved typed scenario setup → private BattleCore
  runtime → existing movement/combat/effect/projectile/damage/death leaves.
- `BossRuntimeWake` is a non-serde private `BattleEvent` variant; no Boss-specific server mapping or Unity runtime
  consumer hit was found. P10 must verify the actual full public boundary and unknown deployed consumers.
- One production reference and one PVE opt-in remain current in source/data/canonical evidence.
- Boss intersects P01/P05~P10 exactly as the Master expected; all 32 other rows have a current changed-surface owner.
- Current tactical files affect P07/P08/P10 impact review but remain an independent user-owned campaign.
- No new candidate is confirmed in P00; material seams are queued for evidence-based Phase review.

### Test result

Tests were read as evidence but not executed. Test count and pass/fail/skip are `not_applicable`. Historical green
counts were not reused as current proof.

### Decision

- Keep the existing P01~P10 boundaries and sequential dependency order.
- Use collision-free `CRR-Pxx-Cyy` and Final `CRR-F-Cyy` for evidence-confirmed new candidates.
- Assign exact test/consumer ownership and require P10 public/external normalization.
- Record the full surface and boundary decisions in `BOSS_DELTA_SURFACE_INVENTORY.md` and
  `PHASE_BOUNDARY_REVIEW.md`.

## 2026-08-09 — P00 static completion validation

### Exact commands

```text
rg '^\| [0-9]+ \| CRA-' ORIGINAL_CANDIDATE_INVENTORY.md | wc -l
comm -3 <sorted historical final IDs> <sorted P00 inventory IDs>
rg '<Boss relation row expression>' ORIGINAL_CANDIDATE_INVENTORY.md | wc -l
rg '<none impact-review row expression>' ORIGINAL_CANDIDATE_INVENTORY.md | wc -l
rg '^\| (0|1|2|3|4|5|6|7|8|9|10|Final) \|' <Master PLAN.md>
rg -n '[[:blank:]]+$' <P00 directory> || true
git status --short --untracked-files=all | rg -v '<Master or P00 path>' | LC_ALL=C sort | sha256sum
git status --short --untracked-files=all | cut -c4- | rg '<relevant path expression>' | LC_ALL=C sort | xargs sha256sum | LC_ALL=C sort | sha256sum
test -e game_server/logs/app.log.2026-08-09 && sha256sum game_server/logs/app.log.2026-08-09
```

### Actual result

- Inventory rows 51; historical-vs-P00 ID diff none; Boss rows 19; other rows 32.
- Master registry has P00 plus P01~P10 and Final, with only P00 active.
- Trailing whitespace output none.
- Campaign-excluded status remains 272 entries with exact hash `8d03ae2f...`.
- Relevant 96-file content manifest remains exact hash `9638cfa2...`.
- P00 currently owns seven files; total current status is 282 entries before its completion audit.
- Unrelated server log remains present with SHA-256
  `0f684cdb1d577ab13531952845a15609a389779d79961adf6b9185424a9a0ec2`.

### Test result

No test runner was executed. Static command results directly validate this investigation-only Phase's identity,
coverage, format and preservation conditions; pass/fail/skip counts are `not_applicable`.

### Decision

Proceed to an independent P00 completion audit. Broad runtime, server and Unity test execution remains omitted
because P00 changes no production contract and those suites cannot prove candidate-set exactness.

## 2026-08-09 — Phase 0 exit audit

### Performed checks

- Re-read the P00 completion conditions, four inventory artifacts and current campaign-excluded status/content
  hashes in a separate audit pass.
- Wrote `COMPLETION_AUDIT.md` with condition-by-condition evidence, consumer/legacy/canonical/dirty review,
  validation omissions, long-term fit and explicit verdict.
- Reproduced the campaign-excluded 272-entry status hash and 96-file relevant content manifest after the audit
  artifact existed.

### Actual result

- P00-C01~P00-C09 all pass.
- Campaign-excluded status remains exact hash `8d03ae2f...`; relevant content remains exact hash `9638cfa2...`.
- Trailing whitespace output is empty.
- Primary verdict: `Complete`; fit: `High`; improvement class: `none`.
- No policy, external, scope or production-mutation blocker prevents P01 activation.

### Test result

No runtime runner was executed. Test count and pass/fail/skip are `not_applicable` for this read-only identity and
boundary Phase. Required static focused checks all produced their recorded exact results.

### Decision

Set P00 to `complete`, update its evidence matrix to pass, hand the audit to the Master and make P01 `ready`.
