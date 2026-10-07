# DefenseRoute 전술 지휘 Phase 5 — 태세와 타일 효과

Status: complete

Implementation Complete: true

Goal Type: phase

Execution Mode: implement_and_verify

Production Mutation Authorized: yes — 이 Phase의 `DR-06`~`DR-09` 범위에 한정

Parent Master: [defense_route_tactical_command_master](../defense_route_tactical_command_master/PLAN.md)

Predecessor Audit:
[Phase 4 Complete / High](../defense_route_tactical_command_phase_04_weapon_tactical_contract/COMPLETION_AUDIT.md)

Phase: 5

Goal Name: `defense_route_tactical_command_phase_05_stance_tile_effects`

Target Repository: `/mnt/f/work/simulator`

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

Audit Finding ID: 없음

## 목표

Authoritative `Attack | Defense` stance와 employee별 1,500ms battle-time cooldown을 추가하고, Defense의
현재 tile/weapon profile에서 Ground DefenseFocus 또는 Platform Rest를 파생한다. 기존 committed action의
완료, 새 action 시작 차단, block/engagement 감소, damage reduction, 1,000ms Rest cadence와
Pause/Hard CC/resonance-lock/checkpoint/server 불변식을 Core authority로 완결한다.

## 승인된 정책과 근거

- Primary policies: `DR-06`, `DR-07`, `DR-08`, `DR-09`
- Target source:
  [DefenseRoute 전술 지휘 목표 정책 결정](../../decisions/defense_route_tactical_command_policy_2026-08-06.md#dr-06--공격방어-태세)
- Master boundary:
  [Phase 5](../defense_route_tactical_command_master/PLAN.md#phase-5--attackdefense와-groundplatform-효과)
- Predecessor weapon contract:
  [Phase 4 Complete / High](../defense_route_tactical_command_phase_04_weapon_tactical_contract/COMPLETION_AUDIT.md)

## 현재 runtime 및 live data 근거

- `BattleCore::tactical_states`가 live player instance stance/tile/profile/cooldown/Rest cursor를 소유한다.
- Deploy/redeploy는 world의 validated zone kind와 draft weapon profile로 Attack+1500ms를 초기화한다.
- Basic/auto/manual admission은 Defense를 검사하고 committed resolve/end 및 triggered delivery는 보존한다.
- Ground는 runtime block/damage modifier와 priority-preserving block refresh, Platform은 epoch-tagged Rest event를
  사용한다.
- Required `RunPolicyData.tactical_stance`와 live RON은 exact `1500/1000`을 소유한다.
- Checkpoint/deployment/event/action capability와 game_server transport가 같은 typed projection을 사용한다.

## 유지할 불변식

- 저장 stance는 `Attack | Defense` 두 값뿐이고 DefenseFocus/Rest는 current tile+weapon에서 파생한다.
- Defense는 새 regular basic attack과 automatic/manual skill 시작만 막고 committed action/delivery를
  취소하지 않는다. Passive/reactive Triggered는 유지한다.
- Same stance, cooldown, Hard CC, weaponless와 invalid context 실패는 resource/event/revision/state 무변경이다.
- 최초/재배치는 Attack으로 시작하고 1,500ms stance cooldown을 시작한다.
- Ground DefenseFocus는 block bonus와 additive incoming damage reduction만 적용하며 resource/timer를 만들지
  않는다. Capacity 감소는 deterministic survivor ordering으로 reconcile한다.
- Platform Rest는 온전한 1,000ms 뒤 자신의 HP/resonance만 clamp 회복하며 death를 되돌리지 않는다.
- Pause/Hard CC/비활성 interval과 resonance lock skip은 catch-up하지 않는다. Same timestamp damage가 Rest보다
  먼저다.
- Phase 4 base-block-only fragment/certified power와 strict V3/weapon-required 의미를 보존한다.
- Phase 6 relocation/occupancy, Phase 7 swap, Phase 8 manual target shape와 Phase 9 FieldWork placeholder를
  추가하지 않는다.
- Phase 0~4 output과 기존 사용자 dirty 파일을 보존한다.

## 범위

- Stance state, cooldown, command/capability/result/event/checkpoint/root/server projection
- Deploy/redeploy Attack initialization과 cooldown
- Defense new-action admission plus committed lifecycle preservation
- Ground effective block와 additive damage reduction, deterministic engagement reconciliation
- Platform Rest event-time cadence와 HP/resonance mutation
- Hard CC/Silence/Pause/resonance-lock/timestamp integration
- Required run policy live RON and strict validation
- Canonical update, focused/broad/static validation and Complete / High audit

## 명시적 비범위

- Relocation/occupancy/facing command와 tile transition (Phase 6)
- Swap command (Phase 7)
- Manual targeting DTO redesign (Phase 8)
- FieldWork (Phase 9)
- Any first-live weapon content
- Unity runtime/contract/UI/tests
- 승인되지 않은 save/event/wire compatibility

## 미결정 정책

현재 알려진 사용자-visible 미결정 항목은 없다. Existing damage/event scheduler가 exact same-timestamp ordering을
표현할 수 없거나 새 version 선택이 필요하면 영향 변경을 중단하고 `policy_decision_required`로 전환한다.

## 구현 계획

1. State→command→action scheduler→block/damage→Rest scheduler→DTO/server/version path를 읽는다.
2. 대상 diff와 focused/broad baseline을 확보한다.
3. Stance policy/state/command/cooldown와 deploy/redeploy initialization을 구현한다.
4. Defense의 새 action gate와 committed lifecycle을 검증한다.
5. Ground DefenseFocus effective block/damage reduction/reconciliation을 구현한다.
6. Platform Rest cadence/ordering/Pause/CC/lock를 구현한다.
7. Checkpoint/root/server/live policy를 strict cutover한다.
8. Focused/broad/static/canonical과 completion audit을 수행한다.

## 완료 조건

- `DR-06`~`DR-09` exact stance/effect/cadence가 구현된다.
- Attack/Defense 양방향 command, same/cooldown/CC/weapon rejection과 atomic immutability가 증명된다.
- Committed action은 완료하고 Defense 이후 새 regular attack/skill만 차단된다.
- Ground block/damage reduction과 capacity-shrink engagement survivor ordering이 direct regression을 가진다.
- Platform Rest의 full interval, clamp, reset, no revive, no tactical resource, Pause/Hard CC/lock/no-catch-up와
  damage-first ordering이 direct regression을 가진다.
- Checkpoint/reconnect/root/server/live policy와 필요한 version cutover가 strict하게 일치한다.
- Core/server/live RON/canonical/static/broad gate가 통과한다.
- Exit audit가 `Complete / High`이고 Master가 Phase 6만 ready로 전환한다.

## DTO migration

필요함. Exact current producer/consumer를 조사해 stance/cooldown/effect projection과 command/result를 같은
release에 추가한다. Unity는 비범위다.

## Data/live RON migration

필요함. `run/policy.ron`에 stance cooldown과 Rest interval의 required ownership을 추가하며 Phase 4 weapon
수치는 그대로 소비한다. Alias/default/dual schema는 추가하지 않는다.

## Save/checkpoint migration

완료. Active battle은 durable active-run payload가 아니므로 Core V3/save version은 유지했다. Live checkpoint와
deployment에 tactical projection을 추가했고 event log는 `StanceChanged` 때문에 strict version 28로 올렸다.
Alias/default/dual event version은 없다.

## 필수 focused 검증

- Attack↔Defense success, same/cooldown/Hard CC/weapon/context rejection와 immutability
- Deploy/redeploy Attack+cooldown initialization, Pause clock behavior, checkpoint/reconnect
- Attack/cast committed-before-toggle completion and no new starts in Defense; Triggered preservation
- Ground focus block/reduction/clamp and deterministic engagement shrink
- Rest full interval/reset/clamp/no revive/no resource/no event-on-zero
- Rest Pause/Hard CC/resonance lock skip/no catch-up and damage-before-Rest timestamp ordering
- Required policy RON/unknown/legacy rejection and exact DTO/server mapping

## 필수 broad 검증

- `cargo test -p game_core`
- `cargo test -p game_server`
- `cargo test -p game_core --test ron_loading`
- `cargo fmt --all -- --check`

## 중단 조건

- 승인되지 않은 stance/effect/timestamp/tuning 선택이 필요함
- Committed action lifetime를 Phase 1/2 contract와 안전하게 결합할 수 없음
- 필수 Core/server/live data/test runner에 접근할 수 없음
- 선행 Phase 또는 user dirty와 안전하게 병합할 수 없음

## Compatibility

승인된 compatibility는 없다. Strict current schema/command/state만 지원한다.

## 완료 감사 Evidence Matrix

| ID | 완료 조건 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P5-01 | Attack/Defense state/command/cooldown | `DR-06` | tactical state/Core/world command | toggle/cooldown/same/CC/context/atomic regressions | pass |
| P5-02 | Committed action와 Defense admission | `DR-07` | regular start gates, pending resume | committed attack/cast + triggered attack tests | pass |
| P5-03 | Ground DefenseFocus | `DR-08` | block/modifier/reconciliation | base+bonus/clamp/Hard CC/shrink regressions | pass |
| P5-04 | Platform Rest cadence/lifecycle | `DR-09` | RestTick epoch/event priority | interval/clamp/skip/no-revive/no-catch-up tests | pass |
| P5-05 | Checkpoint/DTO/server/data/version | Master Phase 5 gate | DTO/event v28/required policy | world/server/RON focused + broad | pass |
| P5-06 | Canonical/legacy/dirty/broad | Master 공통 gate | three canonical docs, strict schema | Core 887/server55/RON24/static/fmt/diff | pass |

## 현재 진행 상태

- Phase 4 completion audit와 production baseline: 완료
- Stance/action/Focus/Rest/data/DTO/server/canonical 구현: 완료
- Focused/broad/static validation: 완료
- Completion audit: [Complete / High](COMPLETION_AUDIT.md)
