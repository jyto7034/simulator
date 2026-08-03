# Simulator Dirty Worktree Commit Cleanup Goal

Status: complete

Implementation complete: true

Goal name: `simulator_dirty_worktree_commit_cleanup`

Created: 2026-08-03

Audit finding ID: 없음

Execution authority: 사용자가 dirty worktree 정리 방법과 저장소별 분리를 승인했다. 초기의 모든 mutation
명령별 승인 정책은 이후 사용자 지시로 대체되어, 파일 삭제와 직접적인 파일 내용 변경만 실행 전 별도
승인을 받고 나머지 조사·검증·Git staging/commit 명령은 포괄 승인된 상태로 실행했다.

## 1. 승인된 Goal 설명

`/mnt/f/work/simulator` Git 저장소의 기존 tracked/untracked 변경을 손실 없이 조사하고 변경 이유별 local
commit으로 보존해, 최종적으로 이 저장소의 `git status --short`가 비어 있는 상태를 만든다.

이 Goal은 기존 변경의 의미를 새로 구현하거나 여러 기능을 하나의 계약으로 합치는 작업이 아니다. 현재
worktree와 local commit history 사이의 보존·분류·staging·commit 경계만 소유한다. 조사 중 발견한 독립
기능의 미완성, 정책 충돌 또는 테스트 실패는 이 Goal에서 수정하지 않고 해당 commit의 blocker 또는
후속 후보로만 기록한다.

## 2. 해결할 사용자-visible 문제

현재 저장소에는 많은 변경과 untracked 파일이 하나의 dirty worktree에 섞여 있어 다음 작업의 baseline,
변경 소유권과 실패 원인을 구분하기 어렵다. 보존할 변경을 의미 있는 local commit으로 확정하고 생성물,
의도 불명 삭제와 미완성 변경을 분리해 이후 Goal이 깨끗한 기준점에서 안전하게 시작할 수 있게 한다.

## 3. 사용자-visible 완료 조건

- 기존 사용자 변경이 누락, 덮어쓰기 또는 의미 변경 없이 local Git history에 보존된다.
- 모든 dirty path가 변경 이유와 소유 계층에 따라 하나의 승인된 commit group에 포함된다.
- 서로 독립적인 Core, server, data, test와 문서 변경은 하나의 catch-all commit에 섞이지 않는다.
- 삭제와 untracked 파일은 의도와 소유권이 확인된 경우에만 stage한다.
- 생성물, 로그, secret 또는 machine-local 파일을 source commit에 포함하지 않는다.
- 각 commit의 staged path와 diff, 필요한 검증 및 commit 결과가 기록된다.
- local commit 완료 뒤 `/mnt/f/work/simulator`에서 `git status --short` 출력이 비어 있다.
- 파일 삭제와 직접적인 파일 내용 변경은 실행 전에 exact command와 대상을 제시하고 사용자의 명시적
  승인을 받는다. 그 밖의 조사·검증·Git staging/commit 명령은 사용자의 포괄 승인에 따라 실행한다.
- remote push, history rewrite, rebase, reset, restore, clean 또는 임의 stash를 수행하지 않는다.
- `/mnt/f/unity projects/ark`의 변경은 건드리지 않는다.

## 4. 대상 저장소와 경로

- 대상 Git root: `/mnt/f/work/simulator`
- Goal 문서: `core_enhanced/docs/goals/simulator_dirty_worktree_commit_cleanup/`
- 초기 dirty top-level 범위: `Cargo.lock`, `core_enhanced/`, `game_resources/`, `game_server/`
- 명시적 비대상 저장소: `/mnt/f/unity projects/ark`

2026-08-03 Goal 생성 전 확인:

- `git rev-parse --show-toplevel`: `/mnt/f/work/simulator`
- Goal 디렉터리와 필수 문서 3개: 모두 존재하지 않았음. 이름 충돌 없음.
- `git status --short`: 306 entries
  - tracked modified: 139
  - tracked deleted: 44
  - untracked: 123
  - top-level 분포: `core_enhanced` 257, `game_resources` 17, `game_server` 31,
    `Cargo.lock` 1

## 5. 현재 runtime 및 live data 근거

초기화 뒤 전체 dirty path를 파일 단위로 조사하고 runtime, live RON, schema, DTO, save, test와 canonical
문서가 포함된 변경을 소유 계층과 기존 Goal 경계에 따라 분류했다.

- `cargo fmt --all -- --check`가 통과했다.
- `CARGO_TARGET_DIR=/tmp/simulator_dirty_cleanup_target cargo test --workspace --all-targets --quiet`에서
  900 tests가 통과했고 fail/ignored는 0이었다.
- 테스트 통과를 위한 production content 수정은 없었다. 허용받은 직접 내용 변경은 기존 문서 2개의 EOF
  여분 빈 줄 제거와 이 Goal 실행 기록 갱신뿐이다.

## 6. Source Of Truth와 소유 계층

| 책임 | Source of truth / owner |
| --- | --- |
| 현재 dirty 상태 | 대상 저장소의 Git index, HEAD와 working tree |
| 변경 내용의 의미 | 각 Core/server/data/test/canonical 파일이 속한 기존 subsystem |
| commit group 경계 | 조사한 변경 이유, producer-consumer 계약과 사용자 승인 |
| 보존 완료 증거 | local commit object, staged diff 기록과 최종 clean status |
| gameplay/runtime/data/wire 정책 | 기존 canonical 및 production source; 이 Goal 문서가 대체하지 않음 |

이 Goal은 Git 정리의 operational source of truth만 소유한다. 독립 기능의 정책이나 runtime 의미를 Goal
편의를 위해 재해석하지 않는다.

## 7. 유지할 불변식

- Working tree의 사용자 content를 staging 또는 commit 과정에서 의미 변경하지 않는다.
- 파일 삭제와 직접적인 파일 내용 변경은 승인 없이 수행하지 않는다. 이번 실행에서는 정확히 14개 로그
  삭제와 기존 문서 2개의 EOF 여분 빈 줄 제거, Goal 문서 3개 갱신만 각각 승인받아 수행했다.
- 조사·검증·Git staging/commit은 사용자가 명시한 포괄 승인 범위에서 수행한다.
- 승인된 삭제 또는 직접 편집의 path 범위를 넘는 추가 변경은 새 승인을 받기 전 수행하지 않는다.
- 하나의 commit은 하나의 변경 이유 또는 밀접하게 결합된 하나의 계약 경계만 가진다.
- 생성물과 machine-local state를 source commit에 숨겨 넣지 않는다.
- Secret 후보가 있으면 내용을 출력하거나 commit하지 않고 즉시 중단·보고한다.
- 실패한 test를 성공처럼 기록하거나 테스트를 통과시키기 위한 코드를 이 Goal에서 수정하지 않는다.
- 기존 Goal/experiment 기록은 history로 보존하고 최신 canonical처럼 다시 쓰지 않는다.
- `Cargo.lock`은 실제 dependency 변경과 연결되는 경우에만 대응 commit에 포함한다.
- 대상 저장소 밖의 Unity 변경과 remote 상태를 바꾸지 않는다.

## 8. 범위

- 초기 및 재개 시 dirty status와 변경 경계 조사
- tracked modification/deletion, rename 후보와 untracked 파일의 전수 분류
- 생성물, 로그, secret/machine-local 후보 식별
- 변경 이유별 commit group과 실행 순서 제안
- group별 staged diff 검토, 필요한 검증과 local commit 생성
- commit 결과와 최종 clean status 검증
- cleanup 과정에서 생성한 이 Goal 문서 자체의 적절한 docs commit 포함

범위는 기존 content의 보존과 Git history 확정이다. 기존 기능을 완성하거나 수정하는 일은 포함하지 않는다.

## 9. 명시적 비범위

- `/mnt/f/unity projects/ark` worktree 정리 또는 commit
- 기존 Core/server/data/Unity 기능의 구현·수정·리팩토링
- 테스트 실패 수정, formatter에 의한 production rewrite 또는 canonical 정책 변경
- remote push, pull, fetch, PR 생성 또는 branch publication
- rebase, squash, amend, force push와 기존 commit history rewrite
- reset, restore, checkout으로 content 되돌리기, clean, 임의 stash
- 승인 없는 파일 삭제, 삭제 복구, rename 확정 또는 `.gitignore` 정책 변경
- 새 Master/Child Goal 자동 생성

Unity worktree 정리는 후속 후보로만 기록하고 이 Goal과 함께 실행하지 않는다.

## 10. 승인된 정책과 근거

- 사용자 승인: dirty 정리는 보존할 파일/폴더를 `add → commit`해 기준점을 만드는 방식으로 수행한다.
- 사용자 승인: 서로 다른 저장소는 분리하며 적합한 구체 방법과 Goal 이름은 작업자가 정한다.
- 이 Goal의 저장소 선택: simulator를 먼저 정리하고 Unity는 별도 후속 실행으로 남긴다.
- Local commit은 파일 단위가 아니라 같은 변경 이유의 coherent group 단위로 만든다.
- Root `git add -A`를 사용하지 않고 검토한 explicit path 또는 좁은 patch 단위만 stage한다.
- Local commit까지만 허용된 범위로 보고 remote mutation은 하지 않는다.
- 기존 content를 바꾸지 않고 그대로 commit하는 것이 기본이다. 의미 수정이 필요하면 이 Goal 밖 문제다.
- 2026-08-03 초기 사용자 정책은 모든 mutation 명령을 실행 직전에 승인받는 것이었다.
- 이후 사용자가 정책을 “`rm` 같은 파일 삭제와 파일 내용 변경 명령만 별도 허락, 나머지는 모두 승인”으로
  명시적으로 대체했다. 따라서 조사·검증·branch 생성·staging·local commit은 포괄 승인으로 실행했다.
- 생성 로그 14개는 정확한 대상과 tar 백업·삭제 명령을 제시해 별도 승인받았다.
- 기존 completion 문서 2개의 EOF 여분 빈 줄 제거와 이 Goal 문서 3개의 최종 기록 갱신도 각각 별도
  승인받았다.

## 11. 확정한 정책

- 전용 local branch `cleanup/simulator-dirty-worktree-20260803`에서 작업했다.
- 44개 tracked deletion은 각 변경 campaign의 기존 문서·source 관계를 조사해 의도된 변경으로
  보존했다.
- 생성 로그 14개는 `.git/cleanup-backups/generated-20260803.tar`로 격리 백업한 뒤 승인된 대상만
  삭제했다. Source와 Goal 기록인 `core_enhanced/tmp/run_persistence_facility_monolith_reaudit_2026-07-28.md`는
  생성물로 오판하지 않고 보존했다.
- 한 파일에 여러 변경 이유가 겹친 경우 patch split으로 내용을 재작성하지 않고, 해당 파일의 최종
  소유 campaign commit에 whole-file로 포함했다.
- 의미 불명 WIP checkpoint는 만들지 않았다. 변경을 10개 coherent commit과 최종 Goal 문서 commit으로
  분리했다.
- Conventional Commit 형식으로 dependency 순서를 따라 commit했다.

## 12. 구현 범위

1. Initial dirty inventory와 관련 nested repository/submodule 경계를 확인한다.
2. 각 dirty path를 subsystem, 상태, 변경 이유, 생성물 가능성, 검증 필요성으로 분류한다.
3. 각 group의 staged path 목록, 의존 관계, 권장 commit message와 검증 명령을 제안한다.
4. 의도 불명 deletion, untracked와 mixed-responsibility file을 별도 정책 질문으로 분리한다.
5. 파일 삭제와 직접적인 파일 내용 변경 직전에 exact command, target과 예상 영향을 제시하고 별도
   승인을 기다린다.
6. 조사·검증·explicit staging과 local commit은 사용자의 포괄 승인에 따라 순서대로 수행한다.
7. 각 commit 뒤 staged/unstaged 상태, commit object와 검증 결과를 기록한다. Goal 문서 내용 갱신은
   별도 승인 뒤 수행한다.
8. 최종 clean status와 기존 content 보존을 read-only로 검증한다.

## 13. Migration

### DTO migration

없음. 이 Goal은 기존 content를 의미 변경하지 않는다.

### Data 또는 live RON migration

없음. 기존 data 변경은 해당 변경 이유의 commit에 보존할 수 있지만 이 Goal에서 schema/content를
변경하지 않는다.

### Save migration

없음. 기존 save/persistence 변경이 dirty tree에 있다면 별도 commit group으로 분류할 뿐 새 migration을
설계하지 않는다.

## 14. 필수 Focused 검증

각 commit group에 다음 공통 검증을 수행했다.

- stage 전 explicit path가 조사한 group과 정확히 일치하는지 확인
- 삭제와 직접 내용 변경에 대응하는 사용자 승인, exact target과 실행 시점을 확인
- `git diff --cached --name-status`와 `git diff --cached --check`
- staged diff가 조사한 working-tree content와 의미상 동일한지 검토
- deletion/rename/untracked가 승인된 intent와 정확히 일치하는지 확인
- commit 뒤 `git show --stat --oneline --decorate --no-renames HEAD`와 해당 commit의 path 확인
- 전체 commit에 `git show --check`를 실행해 오류가 없음을 확인
- docs-only group은 path/whitespace와 관련 문서 일치를 확인

Staging 과정에서 completion 문서 2개의 EOF 여분 빈 줄을 발견했고, 승인 뒤 해당 빈 줄만 제거해
`git diff --cached --check`를 통과시켰다.

## 15. 필수 Broad 검증

- Initial file-level inventory 551개를 538개 committed path identity와 삭제한 untracked 로그 13개로
  reconciliation했다. Tracked 로그 1개의 삭제는 committed path에 포함된다.
- `cargo fmt --all -- --check`: 통과.
- `CARGO_TARGET_DIR=/tmp/simulator_dirty_cleanup_target cargo test --workspace --all-targets --quiet`:
  900 passed, 0 failed, 0 ignored.
- 모든 생성 commit의 `git show --check`와 `git diff main..HEAD --check`: 통과.
- 최종 Goal 문서 commit 뒤 `git status --short`가 비어 있는지 확인한다.
- Unity 저장소는 root와 HEAD, 기존 dirty status만 read-only로 확인했고 변경하지 않았다.
- Remote push와 history rewrite, rebase, reset, restore, clean, stash는 수행하지 않았다.

## 16. 생략할 검증과 근거

- Goal 초기화 시 production/runtime/test 검증은 수행하지 않았다. 문서 선행 절차와 dirty baseline만
  확인한 단계다.
- Unity 저장소 검증은 별도 저장소이며 명시적 비범위다. 단, 대상 밖 상태를 변경하지 않았다는 read-only
  확인만 최종 audit에서 수행한다.
- Remote CI는 local cleanup에 필수로 승인되지 않았고 push도 비범위이므로 실행하지 않는다.

## 17. 중단 조건

- Secret, credential 또는 외부 개인 데이터 후보가 dirty content에서 발견됨
- 삭제/rename/untracked의 사용자 intent를 확인할 수 없음
- 독립 변경이 한 파일에 섞여 patch split 없이 coherent commit을 만들 수 없음
- 현재 branch 선택이나 WIP commit 정책이 결정되지 않음
- nested repository/submodule 경계가 있어 단일 Git root에서 안전하게 stage할 수 없음
- baseline 이후 외부 process나 사용자가 같은 파일을 추가 수정함
- commit 전 필수 검증이 실패하고 실패를 기존 상태와 구분할 수 없음
- 실행하려는 명령이 파일 삭제 또는 직접 내용 변경인데 사용자가 해당 exact command와 target을 아직
  승인하지 않음
- 승인된 command의 실제 target 또는 예상 영향이 승인 요청과 달라짐
- 기존 content를 수정해야만 clean status를 만들 수 있음
- 관련 사용자 변경을 손실 없이 commit으로 보존할 수 없음

정책 선택이 필요하면 `policy_decision_required`, 여러 독립 operational 경계를 다시 분리해야 하면
`scope_decision_required`, 필수 runner/권한/저장소가 없으면 `externally_blocked`로 변경한다.

## 18. Compatibility

승인된 compatibility는 없다. 이 Goal은 runtime/DTO/schema를 변경하지 않으므로 compatibility layer를
추가하지 않는다.

## 19. 완료 조건

- 모든 initial/current dirty path가 commit 또는 승인된 정리 동작과 1:1로 reconciliation됨
- 모든 deletion, rename과 untracked intent가 확인됨
- 모든 commit이 한 변경 이유 또는 결합된 한 계약 경계만 포함함
- 각 commit의 staged diff와 필요한 focused 검증이 기록됨
- 적절한 broad 검증이 통과하거나 기존 실패가 정확히 구분됨
- 생성물, 로그, secret과 machine-local 파일이 source commit에 포함되지 않음
- 기존 사용자 content가 의미 변경이나 손실 없이 local commits에 보존됨
- 승인되지 않은 reset/restore/clean/stash/history rewrite/remote mutation이 없음
- 모든 파일 삭제와 직접 내용 변경에 실행 전의 명시적 사용자 승인과 실제 실행 결과가 연결됨
- `/mnt/f/work/simulator`의 최종 `git status --short`가 empty
- Unity 저장소가 이 Goal에 의해 변경되지 않음
- Goal 실행 기록과 남은 범위 밖 후보가 최신 상태임
- 필수 미검증, 정책 질문과 blocker가 없음

위 조건을 모두 충족할 때만 Status를 `complete`로 변경한다.

## 20. 실행 결과

### Path reconciliation

- Initial file-level dirty inventory: 551 paths (`M` 139, `D` 44, untracked files 368).
- Local commit 보존: 최종 Goal 문서 3개를 포함한 538 committed path identities.
- 승인된 정리: untracked generated log 13개 삭제.
- Tracked `game_server/logs/app.log.2026-07-08` 삭제는 첫 commit의 committed path identity로 보존했다.
- Git rename detection 때문에 최종 rename-aware diff는 534 entries로 표시되지만 path identity 기준 합계는
  `538 + 13 = 551`로 initial inventory와 일치한다.

### Local commits

| Commit | Message | Paths |
| --- | --- | ---: |
| `d7d046c` | `chore: remove generated debug artifacts` | 19 |
| `7b01a32` | `feat: harden cross-layer gameplay contracts` | 50 |
| `ba6bab1` | `feat: implement employee and event domain contracts` | 31 |
| `ccff765` | `feat: implement run persistence and facility policies` | 89 |
| `17e35bc` | `fix: project maintenance action capabilities` | 4 |
| `e883049` | `docs: preserve cross-layer campaign evidence` | 181 |
| `ec51b1d` | `docs: preserve employee and event campaign evidence` | 69 |
| `61b3cfa` | `docs: preserve run policy campaign evidence` | 62 |
| `84cb421` | `docs: record maintenance capability completion` | 3 |
| `afa6300` | `docs: consolidate canonical project documentation` | 27 |
| final Goal docs commit | `docs: complete simulator dirty worktree cleanup` | 3 |

### Generated output와 안전성

- 승인된 삭제 전 14개 로그를 `.git/cleanup-backups/generated-20260803.tar`에 백업했다.
- Tar size: 798,720 bytes.
- SHA-256: `e671b11e27ba8aaa5fac5c5b9ec76198579c6ac50ce6137d16642b1f4287c263`.
- Tar member는 승인된 14개 로그와 정확히 일치한다.
- High-confidence secret scan에서 match가 없었다.
- Remote mutation, history rewrite, rebase, reset, restore, clean과 stash는 수행하지 않았다.
