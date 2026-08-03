# Compendium Transaction Rollback Integrity Correction Experiment Notes

Status: complete notes

Implementation complete: true

Created: 2026-07-28

이 문서는 P00의 source-of-truth 충돌, dirty overlap, 기술 판단, 실험과 범위 밖 후보를 기록한다.
Durable gameplay/save/wire source of truth가 아니다.

## 1. Dependency

- Master: `run_persistence_facility_monolith_policy_correction_master`
- Phase: P00
- 직접 선행 dependency: 없음
- 직접 후속 consumer: correction P01 certified power, 이후 P03 monolith conversion transaction
- 기존 구현 근거: original P02 local compendium 및 P00/P01 transaction/persistence completion records

## 2. Approved Boundary

- Profile knowledge는 run-local rollback 밖의 monotonic set union이다.
- Common transaction과 active-run rollback은 같은 lifetime 의미를 사용해야 한다.
- Server AttachSession은 snapshot 전에 durable singleton profile을 Core에 merge해야 한다.
- Profile file schema/lifecycle와 Unity DTO shape는 현재 의미를 유지한다.
- Account/multi-profile, producer timing 재설계와 다른 correction phase는 범위 밖이다.

## 3. Current/Target Conflict From Reaudit

- 재감사 당시 `clone_non_battle_transaction_state()`는 rollback copy의 compendium을 default로 만들었다.
- active-run host rollback은 live profile을 별도 보존·merge했지만 공통 `with_state_transaction()`은 실패 시
  default copy를 그대로 복원했다.
- server는 일반 player behavior 전에는 profile을 sync했지만 AttachSession snapshot 전에는 sync하지 않아
  실패 직후 reconnect에서 빈 projection이 노출될 수 있었다.
- 이는 새 정책 질문이 아니라 기존 local-install monotonic profile 계약의 누락 구현이다.
- 위 사실은 child current-state 조사에서 실제 파일과 최신 diff로 다시 확인해야 한다.

## 4. Dirty Worktree

- 2026-07-28 child 초기화 시 workspace와 Unity repository 모두 광범위하게 dirty하다.
- 기존 implementation campaign 전체가 Git에서 untracked/modified로 남아 있어 단순 `git diff` 부재를
  clean evidence로 사용할 수 없다.
- P00 대상 파일별 current content, tracked/untracked 상태와 diff를 읽어 기존 의미를 보존한다.
- 안전하게 병합할 수 없는 overlap은 사용자 입력 없이 덮어쓰지 않는다.

## 5. Technical Decision Boundary

- Core compendium domain의 기존 idempotent merge API를 우선 재사용한다.
- Rollback snapshot에 profile full copy를 넣어 active-run payload ownership과 혼동시키는 방식과,
  restore 직전 live profile을 보존해 restore 뒤 merge하는 방식은 실제 call graph/serialization 경계를
  확인한 뒤 결정한다.
- 선택은 profile identity, wire/save shape와 사용자-visible 의미를 바꾸지 않는 내부 transaction detail로
  한정하며 판단 근거를 구현 전에 이 문서에 기록한다.

### 5.1 Current call graph에 따른 선택

- Common rollback snapshot은 profile copy를 계속 소유하지 않는다. `with_state_transaction()`이 실패
  restore 직전 live profile을 clone하고 state restore 뒤 Core domain `merge()`로 union한다.
- 이는 active-run restore가 이미 사용하는 방식과 동일하며 checkpoint/active-run serialization에 profile
  copy를 추가하지 않는다.
- AttachSession은 snapshot 생성 전에 readable shared profile을 merge한다. `Fatal` persistence는 읽을 수
  있는 profile authority가 없고 기존 title Retry/Reset projection이 필수이므로 fatal일 때는 merge를
  생략하고 snapshot/status를 계속 반환한다.
- 기존 command-time sync는 fatal을 명시적 error로 유지한다. Attach 전용 fatal 허용은 gameplay command
  fallback이 아니라 recovery UI bootstrap ordering이다.
- Wire DTO, profile envelope와 Unity reader shape는 바뀌지 않는다.

## 6. Policy Questions

현재 없음. 새 identity/lifetime, compatibility 또는 external DTO 의미가 필요하면 임의로 결정하지 않고
P00을 `policy_decision_required`로 전환한다.

## 7. Out-Of-Scope Candidates

현재 없음. 독립 문제는 이 절에만 기록하고 새 Goal을 자동 생성하거나 P00 production diff에 섞지 않는다.

## 8. Final Interpretation

- F-01은 공통 restore와 AttachSession ordering 두 지점에서 수정됐고 profile schema/lifetime을 새로
  정의하지 않았다.
- Attach의 fatal-profile 예외는 empty-profile fallback이 아니다. Durable authority를 읽을 수 없을 때
  기존 fatal status와 명시적 Retry/Reset만 보여주는 bootstrap 경계다.
- Unity는 기존 exact root compendium과 persistence status shape를 계속 소비하므로 P00 production/contract
  변경이 없다.
- Completion review verdict: `Complete / High`.
