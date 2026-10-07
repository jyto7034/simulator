# DefenseRoute 전술 지휘 Phase 3 — Experiments

Status: active

이 문서는 Phase 3에서 실제 수행한 조사, 변경과 검증만 시간순으로 기록한다.

## 2026-08-06 — Phase Goal 초기화 전 경로·dependency·dirty 확인

### 검증하려던 가설

Phase 2가 `Complete / High`, Master Phase 3이 `ready`, 다른 active Phase가 없고 제안 child 경로가 기존
Goal과 충돌하지 않는다.

### 수행한 확인

- Master registry와 Phase 2 completion audit 확인
- Phase 3 후보 디렉터리와 필수 문서 3개 존재 여부 확인
- Repository root, 시작 commit과 current dirty 상태 확인

### 변경한 내용

- `defense_route_tactical_command_phase_03_tactical_resource_authority` 필수 문서 3개를 생성했다.
- Production code/data/server/canonical은 아직 Phase 3 변경을 하지 않았다.

### 실행한 정확한 명령

```bash
for p in core_enhanced/docs/goals/defense_route_tactical_command_phase_03_tactical_resource_authority core_enhanced/docs/goals/defense_route_tactical_command_phase_03_tactical_resource_authority/PLAN.md core_enhanced/docs/goals/defense_route_tactical_command_phase_03_tactical_resource_authority/EXPERIMENTS.md core_enhanced/docs/goals/defense_route_tactical_command_phase_03_tactical_resource_authority/EXPERIMENT_NOTES.md; do test -e "$p" && echo "EXISTS $p" || echo "MISSING $p"; done
git rev-parse HEAD
git status --short
```

### 실제 test 수

상태 확인이며 test runner는 실행하지 않았다. 해당 없음.

### 실제 결과

- 후보 경로는 모두 존재하지 않았다.
- Start commit은 `f318414fb0156a67e04c5479000c2a2683fc0c09`이다.
- Phase 2는 complete, Phase 3만 ready였고 다른 active Phase는 없었다.
- Current dirty state는 `EXPERIMENT_NOTES.md`에 기록했다.

### 결과에 따른 판단

Phase 3 문서를 먼저 초기화하고 Master activation 뒤에만 `DR-02` current runtime과 production diff를
조사한다.

## 2026-08-06 — Current authority 조사와 focused baseline

### 검증하려던 가설

Current 전술 자원 의미가 deployment state/policy/DTO에 결합되어 있지만 active battle-local 산술과
deploy/redeploy/skill 소비 경로는 이미 존재하며, strict 분리 전에 관련 회귀가 green이다.

### 수행한 확인

- `LiveBattleDeploymentState`, `ActiveBattleSession` regen/signal/reconcile/projection을 읽었다.
- Battle start, authoritative advance, deploy/withdraw/skill transaction을 읽었다.
- Run policy Rust/RON/validator와 ability effect/sim signal을 읽었다.
- Checkpoint, root `InBattle`, game_server serialization test producer를 읽었다.
- 수정 대상 dirty diff를 읽어 Phase 1/2 변경과 겹치는 hunk를 확인했다.
- 관련 focused baseline 5개를 실행했다.

### 실제 결과

- Resource current/max/regen clock은 deployment mapping/redeploy lock과 같은 struct에 결합되어 있다.
- Deploy는 eligibility 뒤 Core 성공 시 한 번 차감하고 withdraw는 환불하지 않는다. Redeploy 비용은 lock에
  고정되어 같은 pool을 사용한다.
- Skill effect/signal은 stabilization legacy name을 사용하고 current만 signed clamp한다.
- Policy/RON과 deployment DTO도 resource/lifecycle을 결합한다.
- Reconnect/root는 same active session projection이고 active battle은 persisted save/checkpoint payload가 아니다.
- Pause는 authoritative battle time을 동결하며 active session 제거가 Retreat/battle end discard boundary다.
- Focused baseline은 각각 1 passed, 0 failed였다.

### 결과에 따른 판단

- `TacticalResourceState`를 `ActiveBattleSession`의 required battle-global field로 만들고 deployment state에서는
  resource arithmetic을 제거한다.
- Policy를 strict `tactical_resource`와 deployment lifecycle로 분리한다. Existing deploy/redeploy action price는
  resource policy가 소유하고 Phase 6/7 relocation/swap price placeholder는 추가하지 않는다.
- DTO는 separate required `tactical_resource`를 checkpoint/root에 projection하고 legacy resource fields는
  deployment DTO에서 제거한다.
- Save version/migration은 없다.

### 실행한 정확한 focused baseline

```bash
cargo test -p game_core live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock
cargo test -p game_core manual_fragment_can_restore_stabilization_and_enable_extra_deployment
cargo test -p game_core live_defense_playback_pause_freezes_server_tick_and_resume_advances
cargo test -p game_core run_policy_rejects_legacy_or_unknown_fields
cargo test -p game_core skill_step_ron_deserialization_reads_stabilization_modifier
```

### 실제 test 수

- 각 filter: 1 passed, 0 failed
- 합계: 5 passed, 0 failed

## 2026-08-06 — Tactical resource strict vertical slice 구현

### 검증하려던 가설

Resource state/policy/DTO를 deployment lifecycle과 분리하고 모든 existing deploy/redeploy/skill producer를 새
authority로 한 release에서 전환하면 legacy compatibility 없이 `DR-02`를 완결할 수 있다.

### 변경한 내용

- `TacticalResourcePolicy`와 battle-local `TacticalResourceState`를 추가하고 current/maximum/recovery cursor,
  deploy/redeploy price를 deployment mapping에서 제거했다.
- Whole-second saturating recovery, current clamp, full `can_pay/pay`, ceiling redeploy price와 signed delta를
  중앙화했다.
- Deploy/redeploy, natural recovery와 skill signal consumer를 새 state에 연결했다.
- `ModifyStabilization`/`StabilizationDelta`를 `ModifyTacticalResource`/`TacticalResourceDelta`로 strict cutover했다.
- Live run policy RON을 required top-level `tactical_resource`와 lifecycle-only `live_deployment`로 분리했다.
- Required `LiveBattleTacticalResourceDto`를 checkpoint와 root InBattle에 추가하고 deployment DTO resource
  field를 제거했다. game_server fixtures/serialization을 같은 shape로 갱신했다.
- Consumable player description과 gameplay/runtime/data canonical을 전술 포인트 의미로 갱신했다.
- Future relocation/swap price placeholder, maximum mutation, implicit drain과 compatibility는 추가하지 않았다.

### 실패 원인과 수정

- 첫 combined patch는 live `first_instance_salt`를 `1000`으로 잘못 예상해 context verification에서 전체
  거부됐다. 실제 `10000`을 읽고 다시 적용했으며 partial mutation은 없었다.
- 첫 `cargo fmt --all -- --check`는 새 줄바꿈만 보고했다. `cargo fmt --all`을 실행하고 final check를
  재실행해 통과했다.
- Compile/test correctness failure는 없었다.

### 결과에 따른 판단

Resource ownership, authoring, effect, projection과 transport가 하나의 strict `tactical_resource` contract로
수렴했다. Active battle은 persisted payload가 아니므로 save/event version migration 없이 DTO/RON만 같은
release에서 cutover한다.

## 2026-08-06 — Focused, broad와 compatibility 감사

### 수행한 정확한 명령

```bash
cargo check -p game_core -p game_server --all-targets
cargo test -p game_core tactical_resource
cargo test -p game_core live_defense_playback_pause_freezes_server_tick_and_resume_advances
cargo test -p game_core live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock
cargo test -p game_core paused_live_battle_commits_deploy_and_withdraw_without_advancing_time
cargo test -p game_core control_status_checkpoint_reconnect_projects_exact_derived_state_and_requires_shape
cargo test -p game_core retreat_battle_restores_exact_pre_entry_snapshot_and_rng_continuation
cargo test -p game_core deploy_cost
cargo test -p game_core skill_step_ron_rejects_legacy_stabilization_modifier
cargo test -p game_core --test ron_loading
cargo test -p game_server serializes_battle_update_as_top_level_transport_message
cargo test -p game_server behavior_result_to_command_result_uses_battle_update_side_message
cargo test -p game_core
cargo test -p game_server
cargo fmt --all -- --check
git diff --check
rg -n "ModifyStabilization|StabilizationDelta|stabilization|initial_cost|max_cost|cost_per_second|current_cost" core_enhanced/src/game core_enhanced/tests game_server/src game_server/examples game_resources/data --glob '!**/docs/**'
rg -n "TacticalResourceState|TacticalResourcePolicy|ModifyTacticalResource|TacticalResourceDelta|LiveBattleTacticalResourceDto|tactical_resource:" core_enhanced/src game_server/src game_server/examples game_resources/data/run/policy.ron --glob '!**/docs/**'
```

### 실제 결과와 test 수

- All-target compile: pass, warning 없음
- `tactical_resource`: 8 passed
- Pause/no-catch-up: 1 passed
- Deploy/redeploy: 1 passed
- Paused deploy/withdraw: 1 passed
- Reconnect/checkpoint/root strict shape: 1 passed
- Retreat discard/reinitialize: 1 passed
- `deploy_cost`: 2 passed, tactical filter와 1개 중복
- Legacy effect rejection: 1 passed
- Live RON: 23 passed
- Server serialization/mapping: 각 1 passed
- Final Core broad: library 825 + integration 3 + 4 + 23 + 13 + 4 = 872 passed
- Final game_server broad: 55 passed
- Failed/ignored: 0
- Formatter와 diff whitespace: pass

### Static 결과 분류

- Old effect/policy field hits는 rejection fixture에만 존재한다.
- Production `current_cost` hit는 Shop refresh quote이며 tactical resource domain이 아니다.
- New state/policy/effect/signal/DTO는 battle start→advance/payment/signal→checkpoint/root→server 경로에 각각
  하나의 producer/consumer chain으로 연결됐다.
- `RunCheckpointPayload`/`ActiveRunCorePayloadV2`에는 active battle/resource가 없고 version change가 없다.
- Future relocation/swap/stance/FieldWork placeholder와 compatibility alias/default/dual state는 없다.

### 최종 판단

모든 evidence row가 pass다. Phase 3은 `Complete / High`이고 Phase 4만 ready로 전환할 수 있다.
