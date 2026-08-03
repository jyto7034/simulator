# Local Encounter Compendium Persistence Goal

Status: complete

Implementation complete: true

Goal name: `run_policy_phase_02_local_compendium_persistence`

Master Goal: `run_persistence_facility_monolith_policy_master`

Master Phase: P02

Created: 2026-07-27

Execution authority: 2026-07-27 사용자 요청

Direct predecessor:
- `../run_policy_phase_01_failure_checkpoint_active_run_persistence/COMPLETION_REVIEW.md`
  — Complete / High

## Objective

P01 active-run aggregate와 transaction을 결합하지 않고, 로컬 설치 단위의 영구 조우 도감을
`discovered`/`defeated` set, authored placeholder projection, dormant ID 보존, atomic current/backup,
bootstrap recovery와 dirty retry까지 Core, host/server, Unity와 canonical에 수직 구현한다.

## Execution Sources

- Master P02 registry/scope/exit gate:
  `../run_persistence_facility_monolith_policy_master/PLAN.md` 9~13절
- P01 completion baseline:
  `../run_policy_phase_01_failure_checkpoint_active_run_persistence/COMPLETION_REVIEW.md`
- 승인 persistence policy:
  `../../decisions/local_encounter_compendium_persistence_policy_draft_2026-07-25.md`
- 승인 defeated/placeholder 의미:
  `../../decisions/elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md` 9~10절
- governance/canonical:
  `../../README.md`, `../../engineering_workflow.md`, `../../goal_completion_review_guide.md`,
  `../../game_rulebook.md`, `../../core_runtime_contract.md`, `../../data_loading_contract.md`
- external Unity canonical:
  `/mnt/f/unity projects/ark/docs/unity_core_contract.md`,
  `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

## Approved Boundary

- profile scope는 OS application persistent-data의 local-install 하나이며 transport `player_id`, run mode,
  active-run file별로 분리하지 않는다.
- Core가 ID format/subset, 현재 catalog 신규 commit, idempotent union과 authored projection을 소유한다.
- host는 version/checksum/lock/current/backup/atomic publish/bootstrap/dirty retry lifecycle만 소유한다.
- dormant ID는 유효한 profile 구성원으로 보존하고 현재 snapshot에는 current catalog 항목만 authored
  순서로 투영한다.
- active-run과 profile write는 서로 다른 transaction이다. 한쪽 실패가 다른 쪽 gameplay/file
  commit을 rollback하지 않는다.
- P05 신규 monolith/hidden-boss producer는 만들지 않고 기존 대표 combat producer가 generic seam을
  실제로 사용하게 한다.

## Out Of Scope

- cloud/account/multi-device merge와 게임 내부 다중 profile
- 기존 배포 profile migration 또는 ID rename table의 실제 항목
- P03 facility, P04 difficulty, P05 monolith/hidden-boss content
- anti-cheat, 암호화와 계정 서명
- current catalog에 아직 없는 hidden boss asset/content를 임시 placeholder로 발명하는 일

## Implementation Plan

1. 승인 문서, P01 review, canonical과 Unity 계약을 읽고 current/target/legacy를 분리한다.
2. Core profile/producer/projection, server actor map, host persistence와 Unity consumer의 dirty diff와
   baseline을 감사한다.
3. Core profile을 discovered/defeated subset과 dormant-preserving injection/current-catalog commit API로
   확장한다.
4. catalog authored 순서 전체의 placeholder/discovered/defeated snapshot과 기존 대표 encounter의
   primary/non-primary defeat producer를 구현한다.
5. 별도 local profile envelope/store, current/backup/checksum/lock/bootstrap/reset과 실제 파일 test를
   구현한다.
6. host가 profile mutation 뒤 최신 전체 set을 write하고 실패 시 gameplay를 유지한 채 dirty retry,
   bounded flush와 status/notification을 관리하게 한다.
7. Unity strict reader, compendium 표시와 recovered/dirty/fatal reset UX를 구현한다.
8. active-run/checkpoint 분리, dormant/legacy/fallback과 canonical을 감사하고 focused/broad 검증한다.
9. completion review가 `Complete / High`일 때만 후속 phase를 연다.

## Test Requirements

- discovered/defeated subset, malformed ID, current catalog 신규 commit과 dormant import/round-trip
- idempotent union, checkpoint/new-run/RunFailed/active-run continue 불변
- authored 전체 순서와 placeholder metadata 은닉, discovered/defeated projection
- primary victory와 승리 attempt의 non-primary `UnitDied`; defeat/retreat/survivor negative
- valid current, missing current+valid backup, both missing, dual corrupt, unsupported version, checksum
- publish failure, single writer, backup rotation, reset preservation과 dirty retry
- active-run/profile write failure 독립성 및 player UUID와 무관한 singleton scope
- server status/notification ordering과 Unity strict reader/recovery/fatal UI
- live RON, Core/server, actual filesystem, Unity focused/broad와 legacy search

## Completion Conditions

- Master P02 exit gate를 모두 충족한다.
- profile set과 current catalog metadata authority가 분리되고 dormant ID가 보존된다.
- write failure가 gameplay를 rollback하지 않고 최신 전체 in-memory profile의 dirty retry가 검증된다.
- local profile과 active-run/checkpoint에 competing copy/transaction이 없다.
- canonical과 구현이 일치하고 material alias/fallback/player-keyed authority가 없다.
- completion review verdict가 `Complete / High`다.

## Stop Conditions

- 배포된 기존 profile compatibility가 필요하나 migration/version 정책이 승인되지 않음
- current catalog에서 authored 순서/placeholder를 결정할 수 없는 필수 content metadata
- profile reset 또는 fatal corruption UX에 승인 decision 밖의 destructive 의미가 필요함
- host storage path/lock/actual file test를 수행할 수 없음
- 기존 battle identity로 승인된 primary/non-primary defeated 의미를 표현할 수 없음

정책 결정이 필요하면 Status를 `policy_decision_required`로 바꾸고 영향받는 mutation/persistence를
중단한다.
