# P01 Failure, Checkpoint, And Active-Run Persistence Completion Review

Reviewed: 2026-07-27

Review basis: Goal 문서의 완료 표시를 증거로 사용하지 않고 현재 Core runtime, game_server 실제 파일
store/actor lifecycle, Unity 원본 Editor consumer, canonical과 broad validation을 다시 확인했다.

## Audit Table

| Item | Verdict | Long-term fit | 핵심 증거 |
| --- | --- | --- | --- |
| Recovery evaluator와 restriction | Complete | High | 한 Core evaluator, recovery-only command matrix와 mutation 후 재평가 |
| Failure 사전 선택과 Treatment | Complete | High | 공통 Awaiting state, load/confirm 제한, damage-0 eligibility와 원자 modifier 교체 |
| Checkpoint monotonic round-trip | Complete | High | checkpoint-owned state round-trip, successful load only, profile/player identity 제외 |
| Durable active-run lifecycle | Complete | High | version/checksum/current+backup/lock/atomic publish/continue/invalidation 실제 파일 test |
| Server·Unity·canonical 수렴 | Complete | High | local-install singleton, explicit replace/reset, strict Unity projection과 broad Editor 통과 |

## Recovery Evaluator And Restriction

Master item:
- P01-A deployability, current-Floor recovery path evaluator와 restriction command matrix

Subgoal:
- `run_policy_phase_01_failure_checkpoint_active_run_persistence`

Policy source:
- `save_point_trauma_recovery_policy_draft_2026-07-25.md` 8~11, 14~15절

Expected behavior:
- 출전 가능한 직원이 없을 때 Core가 실제 현재 Floor 회복 경로를 한 번 정의하고, 경로가 있으면
  failure를 확정하지 않은 채 필요한 action만 허용하고 매 mutation 뒤 다시 평가한다.

Runtime evidence:
- `src/game/world/combat.rs`: SavePoint/Gate/HQ recruitment/Treatment/consumable을 조합하는 common
  evaluator와 `NoDeployableEmployees` restriction
- `src/game/world/helpers.rs`, `maintenance.rs`: restriction action gate와 consumable 이후 재평가
- `src/game/employee.rs`: living employee의 Treatment 구조적 eligibility

Data evidence:
- 기존 P00 `TraumaRecovery` live authoring을 재교체하지 않으며 P01 전용 recovery fallback data를
  추가하지 않았다.

External contract evidence:
- Unity node map은 `combat_recovery_restriction.reason/allowed_node_ids`만 읽고 경로를 재계산하지 않는다.

Test evidence:
- common failure/recovery node/HQ/consumable/treatment-only positive·negative와 mutation 재평가 Core test
- Unity recovery restriction reader/presentation EditMode test

Legacy/fallback audit:
- BossOmen 특례 evaluator와 Unity-local deployability/recovery 계산이 없다.

Long-term direction review:
- Fit: high
- Improvement class: none
- Reason: 선택, mutation과 snapshot이 하나의 Core authority를 공유하고 P03 facility가 같은 semantic
  node 입력으로 확장될 수 있다.

Verdict:
- Complete

Remaining risk:
- 없음.

## Failure Decision And Physical Treatment

Master item:
- P01-B 모든 failure 후보 사전 선택, Physical Treatment와 blocking UX

Subgoal:
- `run_policy_phase_01_failure_checkpoint_active_run_persistence`

Policy source:
- SavePoint/Trauma decision 5.1, 8~11절

Expected behavior:
- loadable checkpoint가 있는 failure 후보는 terminal state 전에 사용자가 load 또는 failure 확정을
  선택한다. Treatment는 살아 있는 직원이면 현재 이득이 0이어도 사용할 수 있고 modifier 교체는
  추가 modal 없이 command와 함께 원자 commit된다.

Runtime evidence:
- `src/game/world/combat.rs`: `AwaitingRunFailureDecision`, loadable 사전 선택과 exact transition
- `src/game/world/action_capability.rs`: Awaiting state에는 load와 confirm만 허용
- `src/game/world/maintenance.rs`: consumable/Treatment mutation과 modifier replacement

External contract evidence:
- Unity `RunFlowPresenter`/shell modal은 checkpoint reason/location/count, 선택적 load와 항상 가능한
  confirm failure를 표시한다.
- `CommandButtonHandler`와 selection state는 modifier overwrite pending state 없이 즉시 command를
  보내며 AbandonRun은 별도 확인한다.

Test evidence:
- invalid checkpoint load 무변경, confirm 성공, final boss common Awaiting, post-terminal load 불가
- damage 0 Treatment, Dead rejection, modifier 교체와 Unity modal/command tests

Legacy/fallback audit:
- `NoPhysicalDamage`, modifier overwrite confirmation과 BossOmen post-`RunFailed` load 예외가 official
  runtime/contract에 없다.

Long-term direction review:
- Fit: high
- Improvement class: none
- Reason: command eligibility와 usefulness가 분리되고 모든 failure reason이 같은 state machine을 쓴다.

Verdict:
- Complete

Remaining risk:
- 없음.

## Checkpoint Envelope And Durable Active Run

Master item:
- P01-C monotonic checkpoint envelope, atomic current/backup/lock, continue와 invalidation

Subgoal:
- `run_policy_phase_01_failure_checkpoint_active_run_persistence`

Policy source:
- SavePoint/Trauma decision 5.1, 9~11, 14~15절

Expected behavior:
- 마지막 완료 SavePoint의 checkpoint-owned state가 restart 뒤 이어지고 성공한 load 횟수만 단조
  증가한다. 손상이나 publish 실패는 조용한 reset/partial commit을 만들지 않으며 terminal run은
  durable active-run을 무효화한다.

Runtime evidence:
- `src/game/world/active_run.rs`: versioned Core export/import, validation, identity/profile 제외와 rollback
- `src/game/world/node_flow.rs`: validated checkpoint restore와 successful-load-only monotonic update
- `game_server/src/persistence/active_run_store.rs`: version/checksum/current/backup/temp/tombstone/lock,
  fsync publish와 backup recovery
- game_server player/load-balance actor: local-install singleton, Continue/Reset/new-run replacement와
  terminal invalidation

Data evidence:
- active-run은 authored live RON이 아니라 명시적 persistence root의 versioned runtime envelope다.
  `data_loading_contract.md`가 두 boundary를 구분한다.

External contract evidence:
- Unity main menu와 modal은 `active_run_continue` status/capability와 explicit replacement/reset action을
  소비한다.

Test evidence:
- Core round-trip이 map/recruitment/inventory/roster/research/VN/run state를 보존하고 다른 player ID와
  local profile을 유지한다.
- real-filesystem tests가 current, backup recovery, both-corrupt, unsupported version, checksum mismatch,
  publish failure, single writer, monotonic backup과 terminal backup 비부활을 검증한다.
- actor tests가 explicit replacement, corruption/reset과 singleton projection을 검증한다.

Legacy/fallback audit:
- player UUID별 active-run authority, silent reset, compatibility alias, dual-read/write와 profile copy가 없다.

Long-term direction review:
- Fit: high
- Improvement class: none
- Reason: Core는 gameplay payload/validation을, host는 file lifecycle을 소유하고 P02 profile aggregate와
  transaction을 공유하지 않는다.

Verdict:
- Complete

Remaining risk:
- 배포 OS별 전원 차단 crash 시험은 별도 플랫폼 validation 영역이다. 현재 실제 filesystem failure
  test와 승인된 ownership/atomic ordering에는 material gap이 없다.

## Cross-Component Findings

- P00 SavePoint/Gate recovery 의미는 그대로 유지되고 P01 evaluator/active-run이 이를 소비한다.
- checkpoint payload, host envelope와 Unity status에 local compendium의 competing copy가 없다.
- active-run lifecycle은 transport player actor와 분리된 local-install singleton이고 Core import 때만
  현재 player identity를 주입한다.
- failure candidate, durable continue와 terminal invalidation이 같은 loadability/count 의미를 사용한다.
- canonical과 official Unity contract가 runtime action/DTO 이름 및 modal ordering과 일치한다.

## Immediate Correction Candidates

- 없음.

## Follow-up Refactor Candidates

- 없음. Local compendium persistence/dirty retry와 actual encounter producer는 계획된 P02 기능이지 P01
  active-run의 미완료 refactor가 아니다.

## Final Validation

- `cargo test -p game_core --lib`: 762 passed
- `cargo test -p game_server --lib`: 40 passed
- `cargo test --workspace --all-targets`: Core 762, integrations 3/4/23/13/4, server 40, 실패 0
- Unity 원본 Editor MCP EditMode: 246 passed, 0 failed
- Unity 원본 Editor MCP PlayMode: 61 passed, 0 failed, 4 explicit live opt-in skipped
- Unity P01 focused SavePoint PlayMode batch run: 2 passed

## Policy Questions

- 없음.

## Final Verdict

- Verdict: Complete
- Long-term fit: High
- Improvement class: none
- P02 activation recommendation: 승인. P01의 local-install active-run/file substrate를 별도 profile
  aggregate로 재사용하되 payload/transaction은 결합하지 않는다.
