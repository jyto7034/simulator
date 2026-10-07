# DefenseRoute 전술 지휘 Phase 1 — Experiments

Status: complete

이 문서는 Phase 1에서 실제 수행한 조사, 변경과 검증만 시간순으로 기록한다.

## 2026-08-06 — Phase Goal 초기화 전 경로·dependency·dirty 확인

### 검증하려던 가설

Phase 0이 `Complete / High`, Master Phase 1이 `ready`, 다른 active Phase가 없고 제안 child 경로가
충돌하지 않는다.

### 수행한 확인

- Master registry와 Phase 0 completion audit 확인
- Phase 1 후보 디렉터리와 필수 문서 3개 존재 여부 확인
- repository root, 시작 commit과 current dirty 상태 확인

### 변경한 내용

- Phase 1 필수 문서 3개를 생성했다.
- Master Current Phase와 registry Phase 1을 `active`로 갱신했다.
- Production code/data/server/canonical은 아직 변경하지 않았다.

### 실행한 정확한 명령

```bash
for p in core_enhanced/docs/goals/defense_route_tactical_command_phase_01_withdraw_redeploy_lifecycle core_enhanced/docs/goals/defense_route_tactical_command_phase_01_withdraw_redeploy_lifecycle/PLAN.md core_enhanced/docs/goals/defense_route_tactical_command_phase_01_withdraw_redeploy_lifecycle/EXPERIMENTS.md core_enhanced/docs/goals/defense_route_tactical_command_phase_01_withdraw_redeploy_lifecycle/EXPERIMENT_NOTES.md; do if [ -e "$p" ]; then echo "EXISTS $p"; else echo "MISSING $p"; fi; done
git rev-parse HEAD
git status --short
```

### 실제 test 수

상태 확인이며 test runner는 실행하지 않았다. 해당 없음.

### Pass / Fail / Skip

해당 없음.

### 실제 결과

- 후보 경로는 모두 존재하지 않았다.
- Start commit은 `f318414fb0156a67e04c5479000c2a2683fc0c09`이다.
- Phase 0은 complete, Phase 1만 ready였고 다른 active Phase는 없었다.
- Current dirty state는 `EXPERIMENT_NOTES.md`에 기록했다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

Phase 1 문서를 먼저 초기화하고 이후에만 `DR-03` current runtime과 production diff를 조사한다.

## 2026-08-06 — Runtime discovery와 production 변경 전 baseline

### 검증하려던 가설

현재 withdraw/redeploy 기반은 동작하지만 release 경계와 비용 반올림에만 material gap이 있고 기존 관련
test는 변경 전에 통과한다.

### 수행한 확인

- Core withdraw cleanup, basic resolve/projectile, skill cast/projectile/area와 stale event 처리
- World withdraw handler, defeated-unit reconciliation, employee attempt ledger와 checkpoint projection
- Employee deploy cost reduction과 game_server battle update/request mapping
- 수정 대상 production 파일의 Phase 시작 diff

### 변경한 내용

없음.

### 실행한 정확한 명령

```bash
git diff -- core_enhanced/src/game/battle/core/build.rs core_enhanced/src/game/battle/core/basic_attack.rs core_enhanced/src/game/battle/core/types.rs core_enhanced/src/game/battle/core/skill_runtime/area.rs core_enhanced/src/game/world/combat.rs core_enhanced/src/game/world/state.rs
cargo test -p game_core withdraw -- --nocapture
```

### 실제 test 수

9 passed, 0 failed, 0 ignored. Filtered 0-test binary는 제외했다.

### Pass / Fail / Skip

Pass.

### 실제 결과

- Active-only withdraw, buff cleanup, pending/active cast cancel, hostile projectile miss, engagement release,
  world lock/Pause 기반은 통과했다.
- Basic instant resolve는 source withdrawal을 구분하지 않았고 active cast cleanup은 released skill
  projectile와 모든 area를 삭제했다.
- Persistent area tick은 매 tick live source template을 만들었으므로 GroundFixed를 보존하려면 생성 시점
  source context를 private runtime에 보관해야 했다.
- 두 redeploy lock 경로는 `base * pct / 100` 내림을 중복했다.

### 실패 원인

Baseline failure 없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

DTO/RON/save migration 없이 private delivery runtime과 공통 world helper를 수정한다.

## 2026-08-06 — Withdraw release boundary와 redeploy lifecycle 구현

### 검증하려던 가설

Unreleased action만 취소하고 detached delivery context를 보존하며 두 lock 경로가 공통 ceiling 계산을
사용하도록 기존 authority seam 안에서 correction할 수 있다.

### 수행한 확인

- Withdrawn과 Dead source의 committed basic attack 차이
- Queued delayed `SkillStep`의 active-cast guard
- Released basic/skill projectile impact와 GroundFixed tick
- GroundFixed/FollowCaster/FollowTarget cleanup matrix
- HP 30% floor lock, new instance initial resonance/cooldown, condition penalty idempotence
- Odd base 3의 150% cost 5, overflow saturation과 50% consumable reduction
- Paused lock ready time과 reconnect projection

### 변경한 내용

- Withdrawn source의 unreleased `AttackResolve`를 semantic event 없이 폐기했다.
- Active skill cast 제거 뒤 queued `SkillStep`을 no-op했다.
- Released projectiles는 cleanup에서 보존하고 source-bound area만 제거했다.
- Persistent area가 생성 시 source snapshot/facing을 보존하고 GroundFixed tick마다 area/time 기반 독립 roll
  identity를 만들도록 했다.
- Overflow-safe `LiveBattleDeploymentState::redeploy_cost()`를 두 lock 경로가 공유하게 했다.
- HP 30% 계산도 `u64` 중간값으로 exact floor를 유지했다.
- Current rulebook/runtime canonical을 `DR-03`과 동기화했다.

### 실행한 정확한 명령

```bash
cargo fmt --all
cargo test -p game_core withdraw -- --nocapture
cargo test -p game_core redeploy_cost_uses_ceiling_for_odd_base_and_saturates -- --nocapture
cargo test -p game_core live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock -- --nocapture
cargo test -p game_core deploy_cost_reduction_consumable_reduces_live_deployment_cost_for_employee -- --nocapture
cargo test -p game_core paused_live_battle_commits_deploy_and_withdraw_without_advancing_time -- --nocapture
cargo test -p game_core live_deployment_reconciles_defeated_player_unit_during_authoritative_advance -- --nocapture
```

### 실제 test 수

최종 unique focused: 19 passed, 0 failed, 0 ignored.

### Pass / Fail / Skip

최종 Pass.

### 실제 결과

- Status와 lifecycle에 무관한 active withdraw, dead/already-withdrawn 무변경 거부를 증명했다.
- Windup instant attack과 queued skill step은 취소되고 released projectile/GroundFixed는 snapshot으로 hit했다.
- FollowCaster/FollowTarget는 즉시 종료하고 buff/target/engagement cleanup을 유지했다.
- Redeploy는 같은 employee의 새 instance에서 initial runtime state를 다시 구성했다.
- Pause/resync는 frozen time의 ready/cost를 보존했고 consumable reduction은 redeploy에도 적용됐다.

### 실패 원인

첫 새 projectile test 실행에서 두 fixture가 `speed_units_per_ms = 1_000`을 사용해 1ms에 target까지
도달하지 않아 2 tests가 실패했다. Runtime failure가 아니라 data-unit scale이
`1 world = 1_000_000 data units`인 fixture 오류였다.

### 되돌리거나 수정한 내용

두 fixture speed를 `1_000_000`으로 수정하고 같은 focused filter를 재실행해 통과했다. 이어 source cast를
제거해도 stale `SkillStep`이 실행되는 gap을 발견해 active-cast guard와 direct regression을 추가했다.

### 결과에 따른 판단

Material lifecycle gap은 닫혔고 새 authoring option이나 serialized shape가 필요하지 않았다.

## 2026-08-06 — Broad validation과 static exit audit

### 검증하려던 가설

최종 runtime/canonical/test 변경이 전체 Core와 server consumer를 회귀시키지 않고 production RON과 strict
compatibility boundary를 유지한다.

### 수행한 확인

- Core/server 전체 package, explicit production RON, formatting
- 이전 floor 계산과 cast-wide projectile/area 삭제 pattern 제거
- 새 fallback/alias/compatibility/TODO와 future Phase production symbol 부재
- DTO/data/save/version 및 Unity 비범위, initial dirty 보존

### 변경한 내용

검증 중 production 변경 없음. Phase evidence와 completion audit만 갱신했다.

### 실행한 정확한 명령

```bash
cargo test -p game_core
cargo test -p game_server
cargo test -p game_core --test ron_loading
cargo fmt --all -- --check
rg -n 'base_deploy_cost\s*\.saturating_mul\(|saturating_mul\(deployment\.redeploy_cost_multiplier_pct\)|active_projectiles\s*\.retain|area\.cast_seq != cast_seq' core_enhanced/src game_server/src
git diff -U0 -- core_enhanced/src game_server/src | rg -n 'fallback|legacy|compat|alias|serde\(alias|TODO|FIXME|placeholder'
git diff --check
```

### 실제 test 수

- Core package: 855 passed (`808` unit + `47` integration), 0 failed, 0 ignored
- game_server: 54 passed, 0 failed, 0 ignored
- explicit RON rerun: 23 passed, 0 failed, 0 ignored

### Pass / Fail / Skip

모두 Pass. Unity runtime/contract/tests는 승인 비범위로 Skip.

### 실제 결과

Old floor/delete pattern과 새 compatibility/placeholder는 0건이었다. Production RON, DTO, save/checkpoint,
event/setup/server envelope version을 변경하지 않았고 `git diff --check`가 통과했다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

Phase 1은 completion audit `Complete / High`이며 Phase 2 activation이 허용된다.
