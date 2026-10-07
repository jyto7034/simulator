# DefenseRoute 전술 지휘 Phase 5 — Experiments

Status: active

이 문서는 Phase 5에서 실제 수행한 조사, 변경과 검증만 시간순으로 기록한다.

## 2026-08-06 — Phase Goal 초기화 전 dependency·path·dirty 확인

### 검증하려던 가설

Phase 4가 `Complete / High`, Master Phase 5만 next이고 proposed child path가 기존 Goal과 충돌하지 않는다.

### 수행한 확인

- Master registry와 Phase 4 completion audit 확인
- Phase 5 후보 directory 존재 여부 확인
- Start commit과 current dirty state 확인

### 실제 결과

- `defense_route_tactical_command_phase_05_stance_tile_effects`는 새 path다.
- Start commit은 `f318414fb0156a67e04c5479000c2a2683fc0c09`다.
- Phase 0~4 verified output과 unrelated user dirty를 보존 경계로 유지한다.

### 결과에 따른 판단

Master에서 Phase 4를 complete, Phase 5를 active로 전환한 뒤 production discovery를 시작한다.

## 2026-08-06 — Runtime/data/server seam discovery와 baseline

### 검증하려던 가설

기존 Core scheduler와 Phase 1~4 authority 위에 별도 third stance 없이 Attack/Defense와 derived tile effect를
추가할 수 있다.

### 수행한 확인

- `BattleCore`, `RuntimeUnit`, `BattleEvent`, basic/auto/manual start와 committed resolve 경로 조사
- block refresh/route progress priority, damage modifier final clamp, HP/resonance mutation과 action lock 조사
- world deployment zone, live command/revision, checkpoint/deployment projection과 game_server generic transport 조사
- required run policy와 Phase 4 `WeaponCombatProfile` focus/rest field 조사
- focused predecessor baseline:
  - `control_status_hard_cc_releases_block_and_silence_preserves_engagement`
  - `silence_blocks_new_casts_but_does_not_interrupt_pending_casts`
  - `fixed_defense_block_capacity_prioritizes_route_progress_and_allows_excess_to_pass`
  - `live_defense_playback_pause_freezes_server_tick_and_resume_advances`

### 실제 결과

- 네 baseline이 각각 `1 passed`였다.
- Active battle은 durable run save payload가 아니며 reconnect/checkpoint는 같은 in-memory Core state projection이다.
- 별도 effect state 없이 unit별 tactical side map, existing event heap, damage modifier와 block refresh를 사용할 수
  있었다.

### 결과에 따른 판단

DTO migration은 current command/result/checkpoint/event에 strict 추가하고 save version은 변경하지 않는다.

## 2026-08-06 — Stance authority와 action admission 구현

### 검증하려던 가설

Attack/Defense 두 값, battle-time cooldown과 derived gate만으로 committed lifecycle를 보존할 수 있다.

### 수행한 변경

- `EngagementStance`, internal `TacticalTileKind`, required `TacticalStancePolicy`와 live `1500/1000` 추가
- `BattleCore::tactical_states`, deploy/redeploy Attack initialization, withdraw cleanup 추가
- `SetEngagementStance` Core/world/server command/result/action capability와 typed reason 추가
- regular basic attack와 auto/manual skill start에 Defense gate 추가
- Attack 복귀 시 pending basic/auto cast와 max-resonance auto cast 재평가
- `StanceChanged` event와 event-log strict version `28` 추가

### 실제 결과

- Same/cooldown/Hard CC/context failure는 Core mutation 전 typed error가 되고 성공만 cooldown/event/revision을
  commit한다.
- Basic attack windup과 manual skill focus 중 Defense 전환 뒤 committed resolve/cast가 완료됐다.
- Defense의 새 regular start는 보류되고 triggered `schedule_next=false` attack은 계속 시작됐다.

### 결과에 따른 판단

Transition/desire/reservation state나 DefenseFocus/Rest stored enum은 필요하지 않았다.

## 2026-08-06 — Ground DefenseFocus와 deterministic engagement shrink

### 검증하려던 가설

Phase 4 weapon 수치를 runtime base에 합성하고 기존 damage/block pipeline을 유지할 수 있다.

### 수행한 변경

- Ground Defense 진입/해제 시 base block±authored bonus와 general incoming reduction 가감
- block refresh의 existing engagement도 route progress, spawn order, unit ID priority로 정렬
- Hard CC projection effective block 0, focus reduction 유지
- actual damage calculation에서 general 10%+physical 95%가 final 100% clamp되는 regression 추가

### 실제 결과

- Ground fixture는 base 2 → Defense 5 → Attack 2, general reduction 0 → 10 → 0이었다.
- Capacity 4 → 2 shrink에서 route progress가 높은 기존 두 적이 남고 낮은 두 적만 통과했다.
- Typed+general 105% reduction은 0 damage, focus 해제 뒤 95%는 5 damage였다.

### 결과에 따른 판단

Fragment/certified power의 Phase 4 base-block-only 입력은 변경하지 않았다.

## 2026-08-06 — Platform Rest scheduler와 lifecycle 구현

### 검증하려던 가설

Epoch-tagged event 하나로 full interval, skip/no-catch-up와 damage-first ordering을 모두 표현할 수 있다.

### 수행한 변경

- `RestTick(time, unit, rest_epoch)`와 deterministic unit/epoch tie ordering 추가
- 모든 damage-producing event 뒤, auto cast start 전 priority로 배치
- active Defense+Platform+exact epoch/next tick 검사, HP/resonance clamp와 actual-zero event suppression 추가
- Hard CC whole-tick skip, resonance-lock resonance-only skip, fixed cadence reschedule 추가
- Attack/condition loss epoch invalidation과 regain full interval 추가

### 실제 결과

- 2499ms에는 변화가 없고 2500ms 첫 tick에서 HP 3/resonance 5가 clamp됐다.
- max 상태는 event를 만들지 않았고 lock/Hard CC tick은 건너뛴 뒤 다음 cadence에서 한 번만 회복했다.
- 같은 2500ms poison tick이 Rest보다 먼저 사망 처리됐고 Rest는 revive/event를 만들지 않았다.
- Attack 이탈 뒤 stale tick은 no-op, Defense 재진입은 새 1000ms deadline을 만들었다.

### 결과에 따른 판단

Rest는 tactical resource, recently-hit cursor, bank/catch-up 또는 별도 effect state를 소유하지 않는다.

## 2026-08-06 — DTO/server/data strict cutover와 focused 검증

### 수행한 변경

- Deployment/checkpoint tactical projection에 stance, tile/effect, cooldown, 양방향 eligibility, base/effective block,
  focus reduction과 next Rest tick 추가
- checkpoint control의 skill disabled reason에 `defense_stance` 추가
- server request `set_engagement_stance`와 derived `defense_focus` stored stance rejection test 추가
- run policy missing/legacy/zero rejection과 production exact tuning test 추가
- canonical rulebook/runtime/data-loading 계약 갱신

### 실제 결과

- `cargo test -p game_core tactical_stance::tests --lib`: `4 passed`
- `cargo test -p game_core defense_blocks_new_skill_starts_but_does_not_interrupt_pending_casts --lib`:
  `1 passed`
- `cargo test -p game_core live_defense_stance_command_projects_cooldown_focus_pause_and_atomic_rejections --lib`:
  `1 passed`
- block-shrink focused: `1 passed`
- game_server request focused: `1 passed`
- `cargo test -p game_core --lib --quiet`: `839 passed`
- `cargo test -p game_core --quiet`: `887 total` (`839+3+4+24+13+4`)
- `cargo test -p game_server --quiet`: `55 passed`
- `cargo test -p game_core --test ron_loading --quiet`: `24 passed`

### 결과에 따른 판단

Production 의미, Core/root/server projection과 strict live data가 같은 Phase 5 계약을 사용한다. 최종
all-target/static/fmt gate와 completion audit만 남았다.

## 2026-08-06 — 최종 broad/static/canonical 감사

### 수행한 검증

- `cargo test -p game_core --quiet`
- `cargo test -p game_server --quiet`
- `cargo check -p game_core --all-targets`
- `cargo check -p game_server --all-targets`
- `cargo fmt --all -- --check`
- `git diff --check`
- stored third stance/legacy policy/stale canonical/static exact-tuning 검색

### 실제 결과

- Core `887`, server `55`, RON subset `24` 모두 pass, failed/ignored 0
- Core/server all-target check warning 없이 pass
- fmt/diff check pass
- authoritative state는 Attack/Defense뿐이고 `toggle_cooldown_ms`는 rejection fixture만 남았다.
- Unity와 Phase 6~9 production placeholder를 추가하지 않았다.

### 결과에 따른 판단

Phase 5를 [Complete / High](COMPLETION_AUDIT.md)로 닫고 Master의 Phase 6만 활성화한다.
