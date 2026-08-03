# Simulator Dirty Worktree Commit Cleanup Experiments

Status: complete

Implementation complete: true

Created: 2026-08-03

실제로 수행한 조사, staging, commit과 검증만 시간순으로 기록한다.

## 2026-08-03 — Goal 경로와 Initial Dirty Baseline

### 검증하려던 가설

- 권장 Goal 이름이 기존 경로와 충돌하지 않는다.
- `/mnt/f/work/simulator`가 독립된 대상 Git root다.
- 신규 Goal 문서를 만들기 전에 현재 dirty 상태를 정확히 기록할 수 있다.

### 수행한 실험 또는 검증

- Goal 디렉터리와 필수 문서 3개의 존재 여부를 확인했다.
- Git root를 확인했다.
- 신규 Goal 규칙에 따라 production 조사 전 `git status --short`를 실행했다.

### 실행한 정확한 명령

```bash
for path in core_enhanced/docs/goals/simulator_dirty_worktree_commit_cleanup \
  core_enhanced/docs/goals/simulator_dirty_worktree_commit_cleanup/PLAN.md \
  core_enhanced/docs/goals/simulator_dirty_worktree_commit_cleanup/EXPERIMENTS.md \
  core_enhanced/docs/goals/simulator_dirty_worktree_commit_cleanup/EXPERIMENT_NOTES.md
do
  if test -e "$path"; then ls -ld "$path"; else echo "ABSENT $path"; fi
done

git rev-parse --show-toplevel
git status --short
```

### 실제 test 수와 pass/fail/skip

- Test runner를 실행하지 않았다.
- 실제 test 수: runner 실행 없음.
- Pass/fail/skip: 해당 없음.

### 실제 결과

- Git root: `/mnt/f/work/simulator`
- Goal 디렉터리와 필수 문서 3개는 모두 없었고 이름 충돌이 없었다.
- `git status --short`: 306 entries
  - ` M`: 139
  - ` D`: 44
  - `??`: 123
- Top-level 분포:
  - `core_enhanced`: 257
  - `game_resources`: 17
  - `game_server`: 31
  - `Cargo.lock`: 1

### 변경한 내용

- Initial baseline 확인 뒤 Goal 필수 문서 3개를 생성했다.
- Production source, data, test와 canonical은 수정하지 않았다.

### 실패 원인

- 명령 실패 없음.

### 되돌리거나 수정한 내용

- 없음.

### 결과에 따른 판단

- Status는 `active`다.
- Target root와 Goal 이름은 확정됐다.
- 44개 deletion과 123개 untracked를 포함하므로 staging/commit 전에 전수 분류와 정책 결정이 필요하다.

## 2026-08-03 — Goal 문서 구조 검증

### 검증하려던 가설

- 필수 문서 3개가 생성됐고 Status 및 필수 섹션을 읽을 수 있다.
- 문서에 trailing whitespace가 없고 Git은 새 Goal 경로만 untracked로 인식한다.

### 수행한 실험 또는 검증

- 세 문서의 line count, Status와 section heading을 확인했다.
- 세 문서의 trailing whitespace를 검색했다.
- Goal 경로만 대상으로 Git status를 확인했다.

### 실행한 정확한 명령

```bash
wc -l PLAN.md EXPERIMENTS.md EXPERIMENT_NOTES.md
rg -n '^Status:|^Implementation complete:|^## ' PLAN.md EXPERIMENTS.md EXPERIMENT_NOTES.md
rg -n '[[:blank:]]+$' PLAN.md EXPERIMENTS.md EXPERIMENT_NOTES.md
git status --short -- core_enhanced/docs/goals/simulator_dirty_worktree_commit_cleanup
```

### 실제 test 수와 pass/fail/skip

- Test runner를 실행하지 않았다.
- 실제 test 수: runner 실행 없음.
- Pass/fail/skip: 해당 없음.

### 실제 결과

- `PLAN.md`: 240 lines, `EXPERIMENTS.md`: 77 lines,
  `EXPERIMENT_NOTES.md`: 108 lines였다.
- 세 문서 모두 `Status: active`, `Implementation complete: false`이며 필요한 section을 읽을 수 있었다.
- Trailing-whitespace 검색은 match 없음(exit 1)이었다.
- Git status는 `?? core_enhanced/docs/goals/simulator_dirty_worktree_commit_cleanup/`만 보고했다.

### 변경한 내용

- 이 검증 기록을 `EXPERIMENTS.md`에 추가했다.
- Production 파일은 변경하지 않았다.

### 실패 원인

- 없음.

### 되돌리거나 수정한 내용

- 없음.

### 결과에 따른 판단

- 신규 Goal 문서 초기화가 완료됐다.
- 다음 실행은 세 문서를 먼저 다시 읽고 306개 initial dirty entry의 read-only inventory부터 시작해야 한다.
- Branch/deletion/generated/WIP 정책이 정해지기 전에는 staging이나 commit을 수행하지 않는다.

## 2026-08-03 — 사용자 승인 필수 Mutation Gate 추가

### 검증하려던 가설

- 사용자가 요청한 “파일을 건드는 명령 실행 전 반드시 허락” 정책을 working-tree file뿐 아니라 Git
  state와 write-capable test/build까지 모호하지 않게 적용할 수 있다.

### 수행한 실험 또는 검증

- 기존 Goal 문서 3개를 처음부터 끝까지 다시 읽었다.
- 사용자의 현재 요청을 이번 문서 edit에 대한 명시적 승인으로 한정해 적용했다.
- PLAN의 완료 조건, 불변식, 승인 정책, 구현 순서, focused 검증, 중단 및 완료 조건에 mutation gate를
  추가했다.
- EXPERIMENT_NOTES에 확정 정책과 안전 제약의 영향을 기록했다.

### 실행한 정확한 명령 또는 도구

```text
cat PLAN.md
cat EXPERIMENTS.md
cat EXPERIMENT_NOTES.md
apply_patch(
  PLAN.md,
  EXPERIMENTS.md,
  EXPERIMENT_NOTES.md
)
```

### 실제 test 수와 pass/fail/skip

- Test runner를 실행하지 않았다.
- 실제 test 수: runner 실행 없음.
- Pass/fail/skip: 해당 없음.

### 실제 결과

- 모든 file/directory/working-tree/Git-state mutation은 exact command와 target을 제시한 뒤 실행 직전
  사용자 승인을 받아야 한다.
- Test/build/formatter/generator도 파일을 쓸 수 있으면 같은 gate를 적용한다.
- 확인된 read-only 명령만 별도 승인 없이 실행할 수 있다.
- 현재 승인은 이번 Goal 문서 3개 edit에만 사용됐으며 향후 mutation을 포괄 승인하지 않는다.

### 변경한 내용

- Goal 필수 문서 3개에 사용자 승인 필수 mutation gate를 기록했다.
- Production 파일은 변경하지 않았다.

### 실패 원인

- 없음.

### 되돌리거나 수정한 내용

- 없음.

### 결과에 따른 판단

- Status는 `active`를 유지한다.
- Read-only inventory는 계속할 수 있다.
- Goal 문서 기록 갱신, test/build, staging, branch 생성과 commit을 포함한 다음 mutation은 사용자가 exact
  command와 target을 승인하기 전 수행하지 않는다.

## 2026-08-03 — Inventory, 정책 확정과 Generated Log 격리

### 검증하려던 가설

- 축약되지 않은 file-level inventory를 모든 path와 1:1로 reconciliation할 수 있다.
- Generated log만 source commit에서 제외하고 나머지 사용자 작성 content를 보존할 수 있다.
- High-confidence secret 후보 없이 coherent commit plan을 만들 수 있다.

### 수행한 실험 또는 검증

- Tracked와 untracked 파일을 각각 전수 열거하고 subsystem, Goal campaign, 생성물 가능성과 owner로 분류했다.
- 사용자가 승인한 전용 branch를 생성했다.
- 승인된 로그 14개를 Git metadata 아래 tar에 백업한 뒤 정확히 그 14개만 삭제했다.
- Tar member, size와 SHA-256을 재검증했다.
- High-confidence credential/secret pattern을 검색했다.

### 실행한 명령 요약

```bash
git switch -c cleanup/simulator-dirty-worktree-20260803
mkdir -p .git/cleanup-backups
tar -cf .git/cleanup-backups/generated-20260803.tar \
  core_enhanced/tmp/unity_reaudit_editmode.log \
  game_server/logs/app.log.2026-07-08 game_server/logs/app.log.2026-07-09 \
  game_server/logs/app.log.2026-07-13 game_server/logs/app.log.2026-07-14 \
  game_server/logs/app.log.2026-07-15 game_server/logs/app.log.2026-07-16 \
  game_server/logs/app.log.2026-07-17 game_server/logs/app.log.2026-07-18 \
  game_server/logs/app.log.2026-07-23 game_server/logs/app.log.2026-07-24 \
  game_server/logs/app.log.2026-07-27 game_server/logs/app.log.2026-07-30 \
  game_server/logs/app.log.2026-08-03
rm -f <위의 승인된 14개 exact paths>
tar -tf .git/cleanup-backups/generated-20260803.tar
sha256sum .git/cleanup-backups/generated-20260803.tar
stat -c '%s bytes' .git/cleanup-backups/generated-20260803.tar
```

### 실제 결과

- Initial status의 untracked directory 축약을 해제한 file-level inventory는 551 paths였다.
  - Modified tracked: 139
  - Deleted tracked: 44
  - Untracked files: 368
- Tar는 14 members, 798,720 bytes였다.
- SHA-256은 `e671b11e27ba8aaa5fac5c5b9ec76198579c6ac50ce6137d16642b1f4287c263`였다.
- 승인된 삭제 대상과 tar members가 정확히 일치했다.
- High-confidence secret scan은 match 없음이었다.
- `core_enhanced/tmp/run_persistence_facility_monolith_reaudit_2026-07-28.md`는 기존 Goal 문서가 참조하는
  작성 evidence이므로 generated log로 분류하지 않고 보존했다.

## 2026-08-03 — Coherent Local Commit 생성

### 수행한 실험 또는 검증

- Root `git add -A` 없이 각 campaign의 explicit path만 stage했다.
- 매 group에서 `git diff --cached --name-status`와 `git diff --cached --check`를 통과시킨 뒤 commit했다.
- Cross-layer completion 문서 2개에서 EOF 여분 빈 줄이 검출되어, 사용자 승인 뒤 그 빈 줄만 제거하고
  재검증했다.

### 실제 결과

| Commit | Message | Files | Diff stat |
| --- | --- | ---: | --- |
| `d7d046c` | `chore: remove generated debug artifacts` | 19 | 79,221 deletions |
| `7b01a32` | `feat: harden cross-layer gameplay contracts` | 50 | +3,519 / -2,043 |
| `ba6bab1` | `feat: implement employee and event domain contracts` | 31 | +3,518 / -2,845 |
| `ccff765` | `feat: implement run persistence and facility policies` | 89 | +32,311 / -10,399 |
| `17e35bc` | `fix: project maintenance action capabilities` | 4 | +2,872 / -173 |
| `e883049` | `docs: preserve cross-layer campaign evidence` | 181 | +25,805 |
| `ec51b1d` | `docs: preserve employee and event campaign evidence` | 69 | +15,191 |
| `61b3cfa` | `docs: preserve run policy campaign evidence` | 62 | +9,501 |
| `84cb421` | `docs: record maintenance capability completion` | 3 | +1,190 |
| `afa6300` | `docs: consolidate canonical project documentation` | 27 | +2,429 / -2,946 |

- 10개 group 모두 cached whitespace check가 통과했다.
- 위 10개 commit 뒤 남은 dirty path는 이 Goal의 문서 3개뿐이었다.

## 2026-08-03 — Broad 검증과 최종 Reconciliation

### 실행한 정확한 명령

```bash
cargo fmt --all -- --check
CARGO_TARGET_DIR=/tmp/simulator_dirty_cleanup_target \
  cargo test --workspace --all-targets --quiet
git show --check --oneline <각 생성 commit>
git diff main..HEAD --check
git status --short --untracked-files=all
```

### 실제 test 수와 pass/fail/skip

- Core library: 799 passed.
- Integration suites: 3 + 4 + 23 + 13 + 4 = 47 passed.
- `game_server` library: 54 passed.
- 합계: 900 passed, 0 failed, 0 ignored.

### 실제 결과

- Formatter check, workspace all-target tests, 모든 commit의 `git show --check`와
  `git diff main..HEAD --check`가 통과했다.
- 10개 campaign commit은 535 path identities를 보존했다. 최종 Goal 문서 commit 3개를 더하면 538이다.
- 승인 후 삭제한 untracked log 13개를 더하면 initial 551 paths와 일치한다. Tracked 7월 8일 로그 삭제는
  첫 commit의 path identity에 포함된다.
- Unity는 `/mnt/f/unity projects/ark`, HEAD
  `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`임을 read-only로 확인했고 기존 dirty tree를 건드리지 않았다.
- Remote mutation, rebase, reset, restore, clean, stash와 history rewrite는 수행하지 않았다.
