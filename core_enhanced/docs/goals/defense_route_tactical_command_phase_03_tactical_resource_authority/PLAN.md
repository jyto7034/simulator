# DefenseRoute 전술 지휘 Phase 3 — 전술 자원 Authority

Status: complete

Implementation Complete: true

Goal Type: phase

Execution Mode: implement_and_verify

Production Mutation Authorized: yes — 이 Phase의 `DR-02` 범위에 한정

Parent Master: [defense_route_tactical_command_master](../defense_route_tactical_command_master/PLAN.md)

Predecessor Audit:
[Phase 2 Complete / High](../defense_route_tactical_command_phase_02_hard_cc_silence_capability/COMPLETION_AUDIT.md)

Phase: 3

Goal Name: `defense_route_tactical_command_phase_03_tactical_resource_authority`

Target Repository: `/mnt/f/work/simulator`

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

Audit Finding ID: 없음

## 목표

기존 deployment state에 결합된 `current_cost`/`max_cost`/`cost_per_second`와 stabilization 이름을 active
battle player-side 전역 `tactical_resource` authority로 strict cutover한다. Initial 20, maximum 99, battle
time 1초당 1 자연 회복, 기본 배치 비용 10을 유지하고 deploy/redeploy와 명시 skill delta가 같은 pool을
원자적으로 사용하도록 Core/checkpoint/server/live RON/canonical을 완결한다.

## 해결할 사용자-visible 또는 contract-visible 문제

현재 자원 산술과 battle-local 수명은 존재하지만 deployment mapping, redeploy lock와 같은 state에 묶여
있고 code/DTO/RON/effect 이름이 `cost` 또는 `stabilization`이다. 이 상태로 이후 relocation/swap/stance/work를
추가하면 하나의 player-side pool 대신 feature별 cost state와 legacy 이름이 확산된다.

## 승인된 정책과 근거

- Primary policy: `DR-02`
- Target source:
  [DefenseRoute 전술 지휘 목표 정책 결정](../../decisions/defense_route_tactical_command_policy_2026-08-06.md#dr-02--전술-포인트)
- Master boundary:
  [Phase 3](../defense_route_tactical_command_master/PLAN.md#phase-3--전술-자원-authority)
- Predecessor control contract:
  [Phase 2 Complete / High](../defense_route_tactical_command_phase_02_hard_cc_silence_capability/COMPLETION_AUDIT.md)

## 현재 runtime 및 live data 근거

Production 조사로 다음 current authority를 확정했다.

- `LiveBattleDeploymentState`가 resource current/max/regen clock과 deployment/redeploy mapping을 함께 소유한다.
- `GameCore::handle_deploy_unit`이 eligibility 뒤 Core deploy 성공 시 pool을 차감하며 withdraw는 차감/환불 없이
  redeploy lock에 다음 비용을 고정한다.
- `SkillEffectDef::ModifyStabilization`이 `BattleLiveSignal::StabilizationDelta`를 만들고 world가 current에
  clamp 적용한다.
- `RunPolicyData.live_deployment`와 live `policy.ron`이 resource tuning과 deployment lifecycle tuning을 한
  struct에 결합한다.
- `LiveBattleDeploymentDto`가 resource와 deployment를 결합해 checkpoint와 root `InBattle`에 재사용되고
  game_server는 checkpoint를 그대로 transport한다.
- Active battle은 persisted run checkpoint/save 대상이 아니며 RequestBattleState/root snapshot은 같은
  in-memory session을 projection한다. Pause는 battle time advance를 막고 Retreat/battle end는 active session을
  제거한다.

## Source Of Truth와 소유 계층

- 목표 의미: target decision record `DR-02`
- Current/implementation authority: `ActiveBattleSession::tactical_resource`와 분리된
  `LiveBattleDeploymentState`
- Live authoring/tuning: `RunPolicyData.tactical_resource`, `RunPolicyData.live_deployment`와 live
  `game_resources/data/run/policy.ron`
- Projection/transport: 별도 required `LiveBattleTacticalResourceDto`, battle checkpoint/root snapshot와
  game_server checkpoint mapping
- 구현 뒤 gameplay/runtime/data canonical: `game_rulebook.md`, `core_runtime_contract.md`,
  `data_loading_contract.md`
- 실행 evidence: 이 Phase Goal과 completion audit

## 유지할 불변식

- Code/DTO/RON의 자원 domain 이름은 `tactical_resource`이며 새 계약에 `stabilization`이나 일반 `cost`를
  resource 이름으로 남기지 않는다.
- 전술 자원은 active battle player-side 전역 pool 하나이며 deployment state와 수명은 같아도 ownership은
  분리한다.
- Battle start initial 20, fixed maximum 99, authoritative battle time 1초당 1 자연 회복을 사용한다.
- Current는 `0..=maximum`으로 clamp하고 overflow/debt/부분 지불/초과분 저장이 없다.
- 비용은 domain mutation 전 전액 eligibility를 확인하고 성공 transaction에서만 한 번 지불한다.
- 최초 deploy와 redeploy가 같은 pool을 사용하며 withdrawal은 소비/환불하지 않는다.
- Explicit skill effect는 signed current delta만 만들고 maximum을 바꾸지 않는다.
- Pause 중 regen은 진행하지 않고 Resume 뒤 catch-up하지 않는다.
- Reconnect/resync는 같은 battle time/current/maximum을 보존하고 전투 종료/Retreat 뒤 다음 전투로 이월하지
  않는다.
- Invalid command는 resource, deployment, salt, cooldown, event와 revision을 변경하지 않는다.
- Encounter override, maximum change effect, implicit enemy/environment drain과 DefenseFocus/Rest producer를
  추가하지 않는다.
- Phase 4~9 command/state/DTO placeholder를 추가하지 않는다.
- 승인되지 않은 compatibility, alias와 dual-read/write를 추가하지 않는다.
- Phase 0~2 변경과 기존 사용자 dirty 파일을 보존한다.

## 범위

- Battle-local `TacticalResourceState`와 deployment responsibility 분리
- Initial/maximum/regen/current delta/payment의 overflow-safe authority
- Existing deploy/redeploy cost consumer의 원자 transaction
- Stabilization effect/signal/code/test naming strict cutover
- Run policy Rust/RON strict schema 분리와 first live tuning 유지
- Battle checkpoint/root in-battle projection의 별도 `tactical_resource` DTO
- game_server producer/consumer와 serialization mapping
- Pause/regen/reconnect/Retreat/battle-end lifecycle regression
- Relevant current canonical와 data contract 갱신
- Phase exit `Complete / High` 감사

## 명시적 비범위

- Phase 4 weapon tactical profile와 weapon-required gate
- Phase 5 stance/DefenseFocus/Rest runtime; 두 effect의 tactical resource 생산은 금지 불변식만 보존
- Phase 6 relocation command/cooldown/payment consumer
- Phase 7 swap command/cooldown/payment consumer
- Phase 8 manual targeting shape
- Phase 9 FieldWork
- Encounter별 resource override, maximum mutation, debt/partial payment와 implicit drain
- Unity runtime/contract/UI/tests
- 승인되지 않은 DTO/save/wire compatibility

## 미결정 정책

현재 알려진 사용자-visible 미결정 항목은 없다. Run policy를 tactical resource와 deployment lifecycle로
분리할 때 여러 user-visible tuning ownership 선택이 발견되거나 persisted active-run migration이 필요하면
영향 변경을 중단하고 `policy_decision_required`로 전환한다.

## 구현 계획

1. State→regen/payment/effect→policy RON→checkpoint/root/server 경로를 읽는다.
2. 수정 대상 diff와 focused baseline을 production 변경 전에 확보한다.
3. Resource arithmetic/lifecycle을 `TacticalResourceState`로 분리하고 deployment mapping에서 제거한다.
4. Existing deploy/redeploy payment와 signed skill delta를 새 authority에 연결한다.
5. Run policy와 live RON을 strict `tactical_resource` naming/ownership으로 cutover한다.
6. Checkpoint/root/server projection을 별도 required tactical-resource shape로 cutover한다.
7. Pause/regen/clamp/rollback/reconnect/Retreat/battle-end regressions과 canonical을 완성한다.
8. Legacy/fallback, broad validation과 completion audit을 수행한다.

## 완료 조건

- `DR-02`의 현재 Phase 의미와 first live tuning이 runtime/data/canonical에 구현된다.
- 전술 자원이 deployment mapping과 분리된 battle-local player-side authority다.
- Current/maximum/clamp/regen/payment/signed delta가 direct regression으로 증명된다.
- Deploy/redeploy insufficient/success path가 resource와 다른 state를 원자적으로 보존/commit한다.
- Pause/reconnect/Retreat/battle-end lifecycle과 no-catch-up/no-carryover가 증명된다.
- Run policy, live RON, checkpoint/root/server가 strict `tactical_resource` 단일 이름/shape를 사용한다.
- `stabilization`과 resource 의미의 generic `current_cost/max_cost/cost_per_second` legacy가 제거된다.
- Core/server focused+broad, production RON validation과 compatibility search가 통과한다.
- Phase exit audit가 `Complete / High`이고 Master가 Phase 4만 ready로 전환한다.

## DTO migration

필요함. Exact current producer/consumer 조사 뒤 deployment DTO에서 resource field를 제거하고 별도 required
`tactical_resource` DTO로 strict cutover한다. Unity DTO는 비범위다.

## Data/live RON migration

필요함. Existing `live_deployment` 안의 resource tuning을 tactical-resource policy로 분리하고 old field를
동시에 받지 않는다. Deployment/redeploy lifecycle tuning의 최종 ownership은 current schema 사용처 조사 뒤
기록한다.

## Save/checkpoint migration

없음. Active battle session은 persisted run checkpoint/save payload에 포함되지 않으며 reconnect/resync는
동일 in-memory `ActiveBattleSession`을 projection한다. 따라서 persisted reader migration이나 save version
증가는 없고 live battle checkpoint/root DTO만 strict cutover한다.

## 배포 단위와 producer-consumer rollout 가정

Core와 game_server는 같은 repository release에서 원자 배포한다. DTO/RON strict shape를 같은 Phase에서
모든 workspace producer/consumer와 fixture에 적용하며 fallback, alias와 dual-read/write를 두지 않는다.

## 필수 focused 검증

- battle start initial/maximum과 resource/deployment ownership 분리
- authoritative battle-time regen, maximum clamp, remainder/cadence와 Pause no-catch-up
- signed positive/negative skill delta의 upper/lower clamp
- deploy/redeploy full-payment success와 insufficient no-mutation
- withdrawal no-cost/no-refund
- reconnect/resync exact current/maximum/time
- Retreat/battle end discard와 next battle reinitialize
- strict run policy old/new RON acceptance/rejection
- checkpoint/root/server new shape와 old shape rejection

Baseline으로 다음 focused test를 모두 1 passed로 확보했다.

- `live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock`
- `manual_fragment_can_restore_stabilization_and_enable_extra_deployment`
- `live_defense_playback_pause_freezes_server_tick_and_resume_advances`
- `run_policy_rejects_legacy_or_unknown_fields`
- `skill_step_ron_deserialization_reads_stabilization_modifier`

## 필수 broad 검증

- `cargo test -p game_core`
- `cargo test -p game_server`
- `cargo test -p game_core --test ron_loading`
- `cargo fmt --all -- --check`

영향 조사에서 추가 version/data validator가 발견되면 포함하고 실제 수를 기록한다.

## 생략할 검증과 근거

- Unity runtime/contract/tests: 사용자 승인 비범위
- Phase 4~9 future command behavior: 선행 implementation 금지
- 관련 없는 외부 repository: 발견되지 않는 한 접근하지 않음

## 중단 조건

- Resource/deployment policy 분리에 사용자-visible tuning 선택이 새로 필요함
- 승인되지 않은 persisted active-run/save compatibility가 필요함
- 필수 Core/server source, live data나 test runner에 접근할 수 없음
- Phase 1/2 선행 lifecycle/control 계약이 current tree 변화로 무효화됨
- 기존 사용자/선행 Phase 변경과 안전하게 병합할 수 없음

## Compatibility

승인된 compatibility는 없다. `tactical_resource` strict behavior/data/DTO cutover를 수행하고 legacy alias,
fallback, dual state나 dual projection을 추가하지 않는다.

## 완료 감사 Evidence Matrix

| ID | 완료 조건 또는 불변식 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P3-01 | 분리된 battle-local tactical resource authority | `DR-02` | `ActiveBattleSession::tactical_resource`, deployment state resource field 제거 | tactical resource/state and deploy focused | pass |
| P3-02 | initial/max/clamp/regen/signed delta | `DR-02` | `TacticalResourceState`, policy/RON 20/99/1 | arithmetic/overflow, positive/negative authored skill | pass |
| P3-03 | deploy/redeploy atomic payment와 withdraw no-refund | `DR-02`, `DR-03` | full `can_pay`/`pay`, locked redeploy price | deploy/redeploy, insufficient immutable, paused withdraw, consumable | pass |
| P3-04 | Pause/reconnect/Retreat/battle-end lifecycle | `DR-02`, `DR-15` | battle-time cursor, same-session projection, active session discard | pause no-catch-up, reconnect/root, retreat reinitialization, broad battle-end | pass |
| P3-05 | Strict policy RON/DTO/checkpoint/server cutover | Master Phase 3 gate | separate policy, required DTO/root/checkpoint, server passthrough | old policy/effect/DTO rejection, RON 23, server serialization | pass |
| P3-06 | Canonical/legacy/compatibility 일치 | Master 공통 gate | gameplay/runtime/data canonical 갱신, no alias/version | static classification, save boundary, diff check | pass |
| P3-07 | Dirty 보존과 focused/broad 검증 | Master 공통 gate | Phase 0~2/user dirty 보존 | Core 872, server 55, fmt/diff pass | pass |

## 현재 진행 상태

- Predecessor audit와 child path/dirty 확인: 완료
- Phase 문서 초기화: 완료
- Production runtime/data discovery: 완료
- Focused baseline과 대상 시작 diff: 완료
- Production/resource-policy-effect-DTO-server-canonical 변경: 완료
- Focused/broad/static validation: 완료
- Completion audit: [Complete / High](COMPLETION_AUDIT.md)
