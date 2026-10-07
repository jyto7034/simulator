# DefenseRoute 전술 지휘 Phase 2 — Experiments

Status: active

이 문서는 Phase 2에서 실제 수행한 조사, 변경과 검증만 시간순으로 기록한다.

## 2026-08-06 — Phase Goal 초기화 전 경로·dependency·dirty 확인

### 검증하려던 가설

Phase 1이 `Complete / High`, Master Phase 2가 `ready`, 다른 active Phase가 없고 제안 child 경로가
충돌하지 않는다.

### 수행한 확인

- Master registry와 Phase 1 completion audit 확인
- Phase 2 후보 디렉터리와 필수 문서 3개 존재 여부 확인
- Repository root, 시작 commit과 current dirty 상태 확인

### 변경한 내용

- `defense_route_tactical_command_phase_02_hard_cc_silence_capability` 필수 문서 3개를 생성했다.
- Production code/data/server/canonical은 아직 변경하지 않았다.

### 실행한 정확한 명령

```bash
for p in core_enhanced/docs/goals/defense_route_tactical_command_phase_02_hard_cc_silence_capability core_enhanced/docs/goals/defense_route_tactical_command_phase_02_hard_cc_silence_capability/PLAN.md core_enhanced/docs/goals/defense_route_tactical_command_phase_02_hard_cc_silence_capability/EXPERIMENTS.md core_enhanced/docs/goals/defense_route_tactical_command_phase_02_hard_cc_silence_capability/EXPERIMENT_NOTES.md; do if [ -e "$p" ]; then echo "EXISTS $p"; else echo "MISSING $p"; fi; done
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
- Phase 1은 complete, Phase 2만 ready였고 다른 active Phase는 없었다.
- Current dirty state는 `EXPERIMENT_NOTES.md`에 기록했다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음.

### 결과에 따른 판단

Phase 2 문서를 먼저 초기화하고 Master activation 뒤에만 `DR-13` current runtime과 production diff를
조사한다.

## 2026-08-06 — Runtime/data/consumer discovery와 focused baseline

### 검증하려던 가설

기존 live buff schema가 capability 분류를 표현하고, Core 내부의 직접 status-name 판정·exclusive Hard CC
replacement·cast delay와 reconnect projection 누락만 Phase 2에서 교정하면 된다.

### 수행한 확인

- `BuffKind`, `BuffDatabase`, active `BuffInstanceKey`와 reapply/expire/remove 경로
- Basic attack start/resolve, Auto/Manual cast start/end, movement, block, resonance와 trigger cooldown 경로
- Phase 1 detached projectile/area/queued-step lifecycle
- `LiveBattleUnitCheckpointDto`, world projection, game_server message/result mapping
- live `game_resources/data/buffs/base.ron`과 event-log version
- 대상 production file의 Phase 2 시작 diff와 focused baseline

### 실행한 정확한 명령

```bash
cargo test -p game_core hard_cc -- --nocapture
cargo test -p game_core silence -- --nocapture
cargo test -p game_core control_status -- --nocapture
cargo test -p game_core active_battle_reconnect -- --nocapture
cargo test -p game_server action_unavailable_preserves_typed_disabled_reason_code -- --nocapture
```

### 실제 test 수

- Hard CC: 7
- Silence: 3
- Control status: 1
- `active_battle_reconnect`: exact-name match 없음, 0
- Server typed reason: 1

### Pass / Fail / Skip

- 12개 실행 test 모두 pass
- Reconnect filter는 0 test라 evidence로 사용하지 않고 새 exact checkpoint regression을 추가하기로 했다.

### 실제 결과

- Live RON은 `Stun`, `Freeze`, `Silence`와 single-stack metadata를 이미 표현했다.
- 서로 다른 Hard CC는 새 적용이 기존 instance를 `Replaced`로 제거했다.
- Hard CC는 새 attack/skill과 movement 일부를 막았지만 pending cast completion을 release 뒤로 연기했고 basic
  windup, block engagement cleanup이 없었다.
- Silence는 새 cast를 막되 started cast를 유지하는 기존 seam이 있었다.
- Active checkpoint unit에는 control state/time/reason이 없었다. game_server는 Core DTO를 generic pass-through했다.
- Event log의 기존 `SkillCastInterrupted`가 새 event/version 없이 cancellation을 표현할 수 있었다.

### 결과에 따른 판단

새 RON/schema나 event version은 만들지 않는다. Active buff classification에서 derived capability를 만들고,
unreleased action ownership만 보강하며 unit checkpoint에 strict required control projection을 추가한다.

## 2026-08-06 — 중앙 capability, cancellation과 strict checkpoint 구현

### 변경한 내용

- `BuffKind::control_capability()`와 `UnitControlState`/`unit_control_state()`를 추가했다.
- Hard CC exclusive replacement를 제거하고 validator를 독립 overlap/expiry 모델로 바꿨다.
- Basic/automatic/manual start, movement, block와 positive resonance gain을 중앙 capability에 연결했다.
- Hard CC 적용 시 pending cast/focus를 `SkillCastInterrupted`로 취소하고 automatic intent만 재평가하도록 했다.
- Automatic basic windup에 epoch와 owned deadline을 부여해 stale resolve를 no-op하고 owned lock만 해제했다.
- Released triggered attack, skill step, projectile, area와 queued buff/effect runtime은 cleanup 대상에서 제외했다.
- `LiveBattleUnitControlStateDto`를 unit checkpoint 필수 field로 추가하고 world projection에서 max Hard CC expiry,
  Silence와 typed reason을 파생했다.
- game_server transport serialization regression과 current canonical 두 문서를 갱신했다.

### 구현 중 실패와 교정

1. 첫 `cargo check -p game_core`는 crate edition에서 Rust 2024 let-chain을 허용하지 않아 실패했다. Nested
   `if let`으로 바꾸고 check가 통과했다.
2. 첫 released-buff regression은 skill step이 확정 buff를 queue한 직후 너무 일찍 active map을 검사해
   1개 test가 실패했다. Queued `ApplyBuff`를 명시적으로 처리해 release boundary를 정확히 검증했다.
3. 첫 `cargo fmt --all -- --check`는 마지막 scheduler 식 formatting 1건을 보고했다. `cargo fmt --all` 뒤
   재실행해 통과했다.
4. Canonical 검토에서 policy expiry와 scheduler retry를 분리했다. Checkpoint는 max expiry를 보고하고 자동
   retry만 same-time 역삽입 방지를 위해 `expiry + 1ms`를 사용하도록 수정했다.

### Focused 최종 검증

```bash
cargo test -p game_core hard_cc -- --nocapture
cargo test -p game_core silence -- --nocapture
cargo test -p game_core control_status -- --nocapture
cargo test -p game_server action_unavailable_preserves_typed_disabled_reason_code -- --nocapture
cargo test -p game_server serializes_control_status_checkpoint_contract_with_typed_reason -- --nocapture
```

### 실제 test 수와 결과

- Hard CC: 15 passed
- Silence: 6 passed
- Control status: 11 passed
- Server typed reason: 1 passed
- Server control transport: 1 passed

Focused test는 overlap/max expiry, automatic basic/manual/auto cancellation, owned lock release, released delayed
step/projectile/area/buff, resonance non-refund, Silence continuation, reason precedence, immutable reject, periodic
damage, trigger cooldown, resonance gain, block와 strict reconnect shape를 포함한다.

## 2026-08-06 — Broad/data/static validation과 Phase exit

### 실행한 정확한 명령

```bash
cargo test -p game_core
cargo test -p game_core --quiet
cargo test -p game_server
cargo test -p game_core --test ron_loading
cargo fmt --all -- --check
rg -n "has_buff_kind|is_hard_cc_kind|BuffKind::Stun|BuffKind::Freeze|BuffKind::Silence" core_enhanced/src/game/battle/core core_enhanced/src/game/battle/validation core_enhanced/src/game/world game_server/src
rg -n "BATTLE_EVENT_LOG_VERSION" core_enhanced/src/game/battle/event_log.rs
git diff --name-only -- core_enhanced/game_resources game_resources
git diff --check
```

### 실제 test 수

- Core: 866 passed (`819 + 3 + 4 + 23 + 13 + 4`)
- game_server: 55 passed
- production RON: 23 passed

### Pass / Fail / Skip

- 모든 broad/data test pass
- `cargo fmt --all -- --check` pass
- `git diff --check` pass
- Unity runtime/contract/UI/tests는 승인된 비범위라 skip

### Static/compatibility 결과

- Production command/core consumer에서 `has_buff_kind`, `is_hard_cc_kind`와 concrete status variant 직접 검사가
  남지 않았다. Search hit는 validator/world test metadata fixture뿐이다.
- Live data diff는 없고 event-log version은 `27` 유지다.
- `control` field에는 serde default/alias가 없고 구 shape 누락 deserialization 실패를 regression으로 고정했다.
- Future stance/relocation/swap/FieldWork command/state/DTO placeholder를 추가하지 않았다.
- Phase 0/1과 기존 user dirty file을 보존했다.

### 결과에 따른 판단

Phase 2 완료 조건과 불변식은 direct regression, broad suite, live RON과 static search로 모두 닫혔다.
Completion confidence는 High다.
