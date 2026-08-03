# Simulator Dirty Worktree Commit Cleanup Experiment Notes

Status: complete

Implementation complete: true

Created: 2026-08-03

이 문서는 구조적 판단, Git source 경계, 정책 질문, 외부 의존성, dirty 상태와 남은 위험을 기록한다.
Production/runtime/canonical의 source of truth가 아니다.

## 1. 구조적 판단

이 Goal의 단일 책임은 simulator Git root의 기존 content를 local commit history에 손실 없이 보존해 clean
baseline을 만드는 것이다. Dirty tree 안의 여러 기능을 하나의 기능 Goal로 합치지 않는다. 변경 이유별로
여러 commit을 만들 수 있지만 기존 기능을 구현하거나 수정하지 않는다.

Unity는 별도 Git root와 별도 commit history를 가지므로 함께 정리하지 않는다. 이 분리는 사용자가
승인했다.

## 2. Source-Of-Truth 충돌

초기화 단계에서는 production/canonical diff를 읽지 않았으므로 구체 충돌은 확인하지 않았다.

- Git status는 변경 존재와 상태만 보여주며 변경 의도나 올바른 정책을 증명하지 않는다.
- Goal 문서는 commit plan과 실행 기록을 소유하지만 각 변경의 gameplay/runtime/data 의미를 결정하지 않는다.
- 기존 canonical, production source와 test가 충돌하면 어느 하나를 자동 정답으로 선택하지 않는다.

## 3. 정책 질문

### 초기 명령별 승인 정책 — 이후 정책으로 대체됨

- 모든 file/directory/working-tree mutation과 Git index/ref/object mutation은 실행 직전에 사용자 승인을
  받아야 한다.
- Test, build, formatter와 generator처럼 주목적이 검증이어도 file/build/log output을 쓸 수 있으면
  mutation 명령으로 취급한다.
- 승인 요청은 exact command, target, 예상 변경, 복구 가능성과 이유를 포함한다.
- 일반적인 Goal 실행 승인, commit plan 승인 또는 과거 command 승인은 다음 mutation의 허가가 아니다.
- 당시 사용자의 요청은 이 정책을 Goal 문서 3개에 기록하는 edit만 승인했다.
- Read-only 명령은 별도 승인 없이 수행할 수 있지만 write 여부가 불명확하면 먼저 승인받는다.

이 초기 정책과 아래 미결정 목록은 후속 사용자 지시 및 discovery 결과로 대체됐다. 최종 결정은
11절에 기록한다.

Discovery 당시 commit 전에 다음 결정을 확정해야 했다.

- Current branch commit과 cleanup 전용 local branch 중 어느 것을 사용할지
- 44개 tracked deletion의 실제 intent
- 생성물/로그/machine-local 후보의 삭제·ignore·보류 정책
- Mixed-responsibility file의 patch split 허용 여부
- 검증 실패/의미 불명 group의 WIP checkpoint commit 허용 여부
- Commit message convention과 ordering

결정 전에는 read-only inventory와 plan만 수행한다.

## 4. 외부 시스템과 직접 의존성

- Local Git repository와 object database
- 변경 범위에 따라 Rust/Cargo, data validator와 game_server test runner가 필요할 수 있음
- Unity 저장소와 runner는 이 Goal의 production 검증 대상이 아님
- Remote repository와 CI는 비범위이며 접근하거나 변경하지 않음

## 5. 범위 밖 후속 후보

- `/mnt/f/unity projects/ark`를 위한 독립 dirty-worktree cleanup Goal
- 현재 simulator dirty content에서 발견되는 각 독립 기능의 미완성/정책/회귀 Goal

후속 Goal이나 Master는 자동 생성하지 않는다.

## 6. 적용하지 않은 대안과 이유

### 두 저장소를 하나의 cleanup Goal로 처리

적용하지 않는다. Git root, history와 변경 책임이 독립적이다.

### 모든 변경을 하나의 catch-all WIP commit으로 저장

기본안으로 사용하지 않는다. 이후 회귀 추적, 리뷰와 선택적 재사용이 어려워진다.

### Root `git add -A`

적용하지 않는다. 44개 deletion과 123개 untracked의 intent를 확인하지 않은 채 모두 stage할 수 있다.

### Stash, reset, restore 또는 clean으로 status를 비우기

적용하지 않는다. 사용자 content를 숨기거나 손실시킬 수 있고 의미 있는 history를 만들지 않는다.

### 테스트를 통과시키기 위한 content 수정

적용하지 않는다. 기존 기능 구현은 이 operational cleanup Goal의 범위 밖이다.

## 7. Initial Dirty Worktree 주의사항

- 신규 Goal 생성 전 exact `git status --short`는 306 entries였다.
- 상태별 수는 modified 139, deleted 44, untracked 123이다.
- Top-level 분포는 `core_enhanced` 257, `game_resources` 17, `game_server` 31,
  `Cargo.lock` 1이다.
- 모든 initial dirty content는 사용자 소유로 취급한다.
- 초기 정책에서는 Goal 문서 갱신을 포함한 모든 mutation에 실행 직전 별도 사용자 승인이 필요했다.
- Goal 문서 3개 생성으로 untracked 경로가 추가되므로 이후 baseline에는 이 문서가 별도로 식별돼야 한다.
- 외부 process 또는 사용자가 baseline 이후 파일을 수정하면 해당 path를 다시 조사하기 전 stage하지 않는다.

## 8. 접근하거나 검증하지 못한 소비자

- 초기화 단계에서는 source diff, runtime consumer, data loader, test와 remote 상태를 조사하지 않았다.
- 실제 배포/CI 상태는 local cleanup 완료에 필수 소비자로 보지 않으며 비범위다.
- 각 semantic change의 접근 가능한 consumer와 검증 범위는 inventory에서 group별로 기록한다.

## 9. 생략한 검증과 근거

- 초기화 단계에서는 code search, diff inspection, runtime, formatter와 test를 실행하지 않았다.
- 이는 신규 Goal 문서 선행 절차를 지키기 위한 의도적 생략이다.
- Commit 완료 증거로 필요한 focused/broad 검증은 inventory와 group plan 뒤 확정한다.

## 10. Initial 남은 위험 — 이후 해소됨

- 306 entries가 여러 독립 기능과 Goal history를 포함할 가능성이 높다.
- 44개 deletion은 의도를 추측하면 data loss 또는 삭제 취소를 일으킬 수 있다.
- 123개 untracked에는 신규 source와 생성물이 섞여 있을 수 있다.
- 같은 파일에 여러 변경 이유가 겹치면 content를 수정하지 않고 clean commit으로 분리하기 어려울 수 있다.
- 현재 tree가 테스트 실패 상태라면 verified commit과 checkpoint commit 정책이 필요하다.
- Secret/machine-local 파일이 있으면 출력·commit 없이 즉시 중단해야 한다.
- Goal 자체가 새 untracked 문서를 추가하므로 최종 docs commit group에 포함해야 한다.
- 승인 없는 mutation을 피하려다 기록 갱신이나 검증이 지연될 수 있다. 이는 의도한 안전 제약이며
  승인을 받기 전에는 진행 속도보다 content 보존을 우선한다.

## 11. 최종 정책 결정

- 사용자의 최종 승인 정책은 파일 삭제와 직접적인 파일 내용 변경만 실행 전 별도 승인을 받고, 나머지
  조사·검증·Git staging/commit 명령은 포괄 승인하는 것이다.
- Cleanup 전용 local branch `cleanup/simulator-dirty-worktree-20260803`를 사용했다.
- 44개 tracked deletion은 관련 source, canonical과 Goal history를 조사해 각 소유 campaign에 포함했다.
- Generated log 14개는 tar 백업 뒤 승인된 exact paths만 삭제했다.
- Mixed-responsibility 파일은 patch split으로 재작성하지 않고 최종 소유 campaign에 whole-file로 포함했다.
- WIP commit은 만들지 않았고 Conventional Commit 형식의 10개 campaign commit과 최종 Goal docs commit으로
  분리했다.

## 12. 최종 Reconciliation

- Initial file-level inventory: 551 paths (`M` 139, `D` 44, untracked files 368).
- Committed path identities: 538개(10개 campaign commit 535개 + Goal 문서 3개).
- 승인 후 삭제한 untracked generated logs: 13개.
- Tracked `game_server/logs/app.log.2026-07-08` 삭제는 첫 commit에 보존됐다.
- 따라서 `538 + 13 = 551`로 initial inventory와 일치한다. Rename-aware Git diff entry 수가 더 작은 것은
  한 rename을 단일 entry로 표시하기 때문이다.

## 13. Generated/Secret 평가

- `.git/cleanup-backups/generated-20260803.tar`에 승인된 로그 14개를 보존했다.
- 백업 크기는 798,720 bytes, SHA-256은
  `e671b11e27ba8aaa5fac5c5b9ec76198579c6ac50ce6137d16642b1f4287c263`다.
- Tar members와 승인된 삭제 대상은 정확히 일치한다.
- High-confidence secret scan은 match 없음이었다.
- 작성 evidence인 `core_enhanced/tmp/run_persistence_facility_monolith_reaudit_2026-07-28.md`는 기존 Goal
  링크의 소비자 관계를 확인해 source history에 보존했다.

## 14. 외부 저장소와 남은 범위

- `/mnt/f/unity projects/ark`는 별도 Git root이며 이 Goal에서 read-only root/HEAD/status 확인만 했다.
- Unity HEAD는 `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`였고 기존 dirty worktree는 변경하지 않았다.
- Remote push, pull, fetch, PR, rebase, reset, restore, clean, stash와 history rewrite는 수행하지 않았다.
- Simulator cleanup에 필수인 미검증, 정책 질문과 blocker는 남지 않았다.
