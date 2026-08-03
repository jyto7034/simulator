# Run Policy Master Final Completion Review

Status: complete

Implementation complete: true

Goal name: `run_policy_master_final_completion_review`

Created: 2026-07-27

Parent: `run_persistence_facility_monolith_policy_master` P07

## 1. 목표

P00~P05의 완료 문서와 과거 test 결과를 그대로 신뢰하지 않고 현재 worktree의 runtime, live RON,
game_server, persistence, 외부 Unity client와 durable canonical을 독립적으로 감사한다. 실제 hidden-boss
콘텐츠는 현재 master 밖의 후반 콘텐츠 Goal이며 이 검토에서 구현하거나 완료 조건으로 사용하지 않는다.

## 2. 검토 범위

- P00: SavePoint/Gate Trauma recovery와 Support/Rest 제거
- P01: 출전 불가 실패, checkpoint load, active-run durability와 monotonic envelope
- P02: run/checkpoint 밖 local compendium, discovered/defeated exact commit와 host storage
- P03: authored facility template, assignment, navigation과 generic manifestation/capstone seam
- P04: certified power, run peak, Floor anchor와 node/role difficulty
- P05: natural Elite exclusion, wandering merchant, monolith conversion/remanifestation, residual exchange,
  next-Floor Shop guarantee와 dormant corrupted key
- P00~P05 사이의 save, generation, result commit, host transaction과 Unity projection seam
- 결정 기록의 implementation disposition과 durable canonical의 현재 구현 일치

## 3. 명시적 비범위

- 숨겨진 보스 actor, skill, encounter, battlefield/wave, reward 또는 placeholder
- 관리자 VN, hidden capstone, corrupted key 소비, 특수 completion
- FinalBoss 재분류나 기존 Endless completion interception
- 새 콘텐츠 정책·밸런스·서사 결정
- 범위 밖 dirty worktree 정리 또는 과거 배포 save compatibility 추가

## 4. 독립 감사 질문

1. P03~P05 run-local state가 checkpoint와 active-run에 완전하게 round-trip되는가?
2. `loads_used`, run peak 등 monotonic metadata는 checkpoint rollback으로 낮아지지 않는가?
3. local compendium은 new run, failure, checkpoint load와 active-run continue에 오염되지 않는가?
4. active-run과 compendium current/backup/checksum/lock/failure transaction이 분리되는가?
5. facility assignment와 power/difficulty가 같은 Floor generation commit에서 결정되는가?
6. monolith next-Floor Shop guarantee가 generic facility requirement를 사용하고 기존 topology를
   변경하지 않는가?
7. discovered/defeated가 실제 spawn/result identity와 승인된 commit 시점에 정확히 한 번 결합되는가?
8. Core, live RON, server, Unity, mock/probe와 canonical에 승인 없이 살아 있는 legacy authority가 없는가?
9. dormant corrupted key가 hidden-boss consumer 없이 저장되며 기존 completion을 가로채지 않는가?

## 5. 검토 방법

- 각 predecessor `COMPLETION_REVIEW.md`는 주장과 evidence 위치를 찾는 데만 사용한다.
- production code, live RON, persistence file path/transaction, server mapping과 Unity reader/test를 직접
  읽고 대표 invariant를 source evidence로 연결한다.
- focused test와 broad validation을 현재 worktree에서 재실행하고 실제 runner count를 기록한다.
- material finding은 원래 owner phase와 severity에 연결한다. 현재 범위의 작고 명백한 correction만
  수정·재검증하며 정책 변경이나 hidden-boss 콘텐츠가 필요하면 완료 처리하지 않는다.
- 검색 결과는 문자열 존재만으로 legacy finding을 만들지 않고 production authority, fixture의
  negative-validation 용례와 historical document를 구분한다.

## 6. 필수 검증

- Rust formatting/check와 `game_core`, `game_server` compile
- live RON strict loading과 Core/server broad tests
- checkpoint/active-run, compendium storage, facility+difficulty, monolith guarantee/result commit focused tests
- 가능하면 localhost server probe로 실제 wire 경로 확인
- Unity 원본 Editor에서 refresh/compile, 관련 focused tests, broad EditMode와 PlayMode
- scoped legacy/canonical search와 `git diff --check`

## 7. 완료 조건

- P00~P05 각 audit verdict가 `Complete / High`다.
- 독립 감사 질문 전부가 runtime/data/persistence/server/Unity/canonical evidence에 답을 가진다.
- material finding이 없거나 현재 범위 correction 뒤 같은 surface를 재검증했다.
- 필수 focused/broad runner가 실패 없이 완료되고 미실행 항목이 명시된다.
- P06/future hidden-boss 콘텐츠를 구현하거나 현재 완료 근거로 가장하지 않았다.
- `COMPLETION_REVIEW.md`가 finding, audit table, evidence, runner count와 최종 verdict를 기록한다.

## 8. 중단 조건

- 승인되지 않은 policy/content 선택이 필요한 material mismatch
- Unity 원본 프로젝트 또는 필수 runner에 접근할 수 없어 핵심 소비자 계약을 검증할 수 없음
- 범위 밖 dirty 변경과 겹쳐 안전한 correction 또는 검증이 불가능함

## 9. 완료 결과

- P00~P05 audit verdict: 모두 `Complete / High`
- Cross-component finding: 3건을 현재 범위의 좁은 correction으로 수정하고 focused/broad 재검증 완료
- Rust broad: Core library 787, RON integration 23, server 52, 실패 0
- Unity broad: EditMode 259/259, PlayMode 62 pass + explicit live 4 skip, 실패 0
- Live wire: admin non-combat probe와 authored SavePoint checkpoint/load probe 통과
- 최종 판정과 직접 evidence: [COMPLETION_REVIEW.md](COMPLETION_REVIEW.md)
- 실제 hidden-boss 콘텐츠: 구현하지 않았으며 현재 master 밖 후반 콘텐츠 Goal로 유지
