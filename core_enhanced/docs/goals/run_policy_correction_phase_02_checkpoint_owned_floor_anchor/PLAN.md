# Checkpoint-Owned Floor Anchor Correction Goal

Status: complete

Implementation complete: true

Goal name: `run_policy_correction_phase_02_checkpoint_owned_floor_anchor`

Master Goal: `run_persistence_facility_monolith_policy_correction_master`

Master Phase: P02

Created: 2026-07-28

Direct predecessor:
- `../run_policy_correction_phase_01_combat_eligible_certified_power/COMPLETION_REVIEW.md`
  — Complete / High

## Objective

Current Floor anchor와 power model version을 checkpoint-owned payload로 이동해 map/node difficulty와 같은
세대로 복원하고, host monotonic metadata는 successful loads와 run peak만 보존하도록 strict active-run
core payload/envelope schema를 교체한다.

## Approved Target

- SavePoint checkpoint가 current Floor anchor와 power model version을 소유한다.
- Load/continue는 checkpoint map/node difficulty와 같은 generation의 anchor를 복원한다.
- Host current/backup max merge는 `loads_used`와 `run_peak_certified_power`만 수행한다.
- Current Floor anchor는 max merge하지 않고 각 current/backup checkpoint payload의 값을 보존한다.
- Core payload와 host envelope breaking version을 올리고 구 schema/field는 migration 없이 명시 거부한다.
- 같은 checkpoint를 load하고 같은 Gate branch를 재진행해도 load 자체가 next-Floor anchor를 추가 상승시키지
  않는다.

## In Scope

- `RunCheckpointPayload`, checkpoint capture/load와 run power authority restore
- `ActiveRunCheckpointExport`, Core payload schema/version와 continue validation
- game_server active-run envelope/monotonic metadata/current-backup publication
- old envelope/core payload/field rejection 및 filesystem 회귀
- Core/server/canonical/Unity-visible 영향 검증

## Out Of Scope

- checkpoint load 최대 3회 정책 변경
- run peak rewind, power formula/coefficient 변경
- save migration/dual reader/alias/default compatibility
- monolith stage stat/reward/Unity reward DTO(P03)
- 범위 밖 dirty 변경 정리

## Dirty Worktree Gate

Core world checkpoint/active-run, server persistence/actor tests와 canonical은 기존 campaign 변경과 겹친다.
수정 전 current source와 scoped diff를 읽고 기존 의미를 보존한다. reset/restore/stash/clean 또는
repository-wide formatting을 하지 않는다. 겹친 사용자 변경과 안전하게 병합할 수 없으면 중단한다.

## Implementation Plan

1. checkpoint capture/load, Floor advance/anchor 계산, active-run export/import, envelope backup merge와 schema
   validation을 current source에서 추적한다.
2. cross-Floor load/re-advance 및 actual filesystem current/backup baseline을 확보한다.
3. anchor/model version을 checkpoint payload로 이동하고 Core payload/envelope를 새 strict version으로 교체한다.
4. host monotonic anchor field/max merge를 제거하고 loads/peak만 high-water merge한다.
5. old schema/field fixture를 rejection fixture로 바꾸고 deterministic anchor/current-backup regressions를
   추가한다.
6. canonical, focused→broad 검증, 독립 `Complete / High` review 후 P03를 활성화한다.

## Test Requirements

- checkpoint anchor/model version capture와 exact restore
- cross-Floor load 후 같은 Gate 재진행의 deterministic next anchor
- run peak와 loads의 load/restart/backup non-decrease
- current/backup payload별 anchor 보존, publish failure atomicity와 recovery
- unsupported old envelope/core payload 및 removed monotonic anchor field rejection
- Core focused/broad, server real-filesystem focused/broad, workspace all-targets
- DTO/Unity 영향, scoped legacy search/diff/status 검증

## Completion Conditions

- Master P02 exit gate와 모든 test requirement가 current evidence로 입증된다.
- Checkpoint/map/node/anchor generation ownership이 일치하고 host monotonic metadata가 loads/peak만 소유한다.
- 구 schema/field reader/writer/fixture가 없고 migration/fallback이 없다.
- Completion review가 `Complete / High`다.

## Stop Conditions

- save migration/Unity-visible shape 또는 anchor generation에 새 정책 선택이 필요함
- target dirty 변경과 안전하게 병합할 수 없음
- actual filesystem/backup 검증에 접근할 수 없음
- P01 formula/version 의미를 바꿔야만 구현 가능함
