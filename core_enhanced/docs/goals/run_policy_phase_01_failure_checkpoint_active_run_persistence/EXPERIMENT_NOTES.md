# Failure, Checkpoint, And Active-Run Persistence Experiment Notes

Status: complete notes

Implementation complete: true

Created: 2026-07-27

이 문서는 P01의 조사, 판단, 실험, 실패, persistence 위험과 범위 밖 후보를 기록한다.
Durable gameplay/save/wire source of truth가 아니다.

## 1. Dependency

- Master: `run_persistence_facility_monolith_policy_master`
- Phase: P01
- 직접 선행: P00 completion review `Complete / High`
- 후속 consumer: P02 local profile persistence와 P03~P05

## 2. 승인 경계

- P00 recovery schema/값/preview/transaction은 유지한다.
- P01은 failure/recovery decision과 durable active-run을 추가한다.
- local profile은 P02 소유이며 active-run payload에 복제하지 않는다.
- 실제 save compatibility 요구가 발견되면 승인 없이 alias/migration을 만들지 않는다.

## 3. 최종 상태

- Core common evaluator가 출전 가능성, 현재 Floor recovery path와 restriction command matrix를
  소유한다.
- 모든 failure 후보는 loadable checkpoint가 있으면 `AwaitingRunFailureDecision`을 거치고, terminal
  failure 뒤 checkpoint load 예외는 없다.
- Physical Treatment eligibility와 실효 이득은 분리됐으며 modifier 교체는 command transaction
  안에서 원자적이다.
- Core active-run export는 checkpoint-owned run state와 monotonic `loads_used`만 포함한다.
  transport player identity와 local compendium profile은 포함하지 않는다.
- game_server local-install singleton store가 checksum, backup, lock, atomic publish, corruption/reset,
  continue와 terminal invalidation lifecycle을 소유한다.
- Unity는 Core/server projection만 소비하며 continue/reset/new-run replacement, recovery restriction과
  common failure modal을 표시한다.
- Rust workspace와 원본 Unity Editor broad validation이 통과했다.

## 4. Dirty Worktree

P00과 선행 구현의 관련 변경을 보존한다. P01 대상별 diff를 읽기 전에는 production 수정하지 않는다.

2026-07-27 scoped status에서 Core `behavior.rs`, `world.rs`, `world/combat.rs`, `world/node_flow.rs`,
`world/state.rs`, consumable/employee, game_server actor와 외부 Unity reader/presenter/view/test가 모두
기존 대규모 변경과 겹친다. 변경은 P00 및 여러 선행 Goal의 누적 current state로 보이며 전체
restore/format/stash를 하지 않는다. 각 P01 edit는 현재 파일의 의미를 보존하는 국소 patch로만 한다.

## 5. 정책 질문

없음. P01 구현과 완료 리뷰에서 승인 decision 밖의 새 gameplay/save/UX 판단은 필요하지 않았다.

## 6. 범위 밖 후보

- 임시 Unity batchmode에서의 `WaitForEndOfFrame` 제약은 원본 non-batch Editor MCP broad 결과로
  분리했다. production/test 변경 후보로 승격하지 않는다.
- host atomic rename의 각 OS별 crash semantics를 실제 모든 배포 OS에서 검증하는 일은 배포/플랫폼
  검증 범위다. 현재 Linux real-filesystem store test와 Windows Unity consumer 검증은 통과했으며
  승인된 P01 의미를 바꾸는 미완료 항목은 아니다.
