# DefenseRoute 전술 지휘 Phase 8 — Experiments

Status: complete

이 문서는 Phase 8에서 실제 수행한 조사, 변경과 검증만 시간순으로 기록한다.

## 2026-08-06 — Phase Goal 초기화

### 검증하려던 가설

Phase 7이 Complete/High이고 Phase 8만 next이며 required selection/focus scope를 FieldWork와 분리할 수 있다.

### 실제 결과

- Predecessor audit는 Complete/High다.
- 새 child path는 기존 Goal과 충돌하지 않는다.
- DR-16은 selection, focus, capture/no-retarget, Black Round live migration을 모두 확정한다.

### 결과에 따른 판단

Master에서 Phase 8만 active로 전환하고 production discovery를 시작한다.

## 2026-08-06 — Selection schema와 live cutover

### 검증하려던 가설

Effect `SkillTarget`을 유지한 채 required `SkillCastSelection`만 별도 authority로 추가하면 기존 자동 targeting
tuning을 바꾸지 않고 player-selected activation shape를 엄격히 분리할 수 있다.

### 실제 결과

- `SkillCastTargetingDef::Explicit`과 raw RON schema에 required selection을 추가했다. Live loader는 omission,
  unknown selection, target-incompatible pair와 player-selected `RetargetOnStep`을 거부한다.
- 25개 live skill 모두 selection 한 줄만 추가했다. Self 4, AutomaticUnit 19, PlayerUnit 2, PlayerTile 0이며
  PlayerUnit은 `fragment_freischutz_black_round`와 `freischutz_magic_bullet`뿐이다.
- Existing range/focus/damage/delivery 값은 변경하지 않았다. Skill catalog는 selection을 노출하도록 version 3이 됐다.
- Rust fixture convenience constructor는 target으로 fixture selection을 만들 수 있지만 live deserializer와 production
  runtime은 그 helper를 사용해 누락 field를 보정하지 않는다.

### 결과에 따른 판단

Strict current RON cutover만 필요하고 compatibility/dual read나 durable save migration은 없다.

## 2026-08-06 — Explicit acceptance와 capture seam

### 검증하려던 가설

Existing manual base preflight와 cast target resolver를 selection-aware final validator로 분리하면 target kind fallback
없이 Self/AutomaticUnit/PlayerUnit/PlayerTile을 같은 Core authority에서 처리할 수 있다.

### 실제 결과

- Base preflight는 player active combatant/manual/Attack, Hard CC/Silence, action lock, exact skill과 full resonance를
  먼저 검사한다.
- Self/AutomaticUnit은 no-payload만 허용한다. PlayerUnit/PlayerTile은 exact Unit/Tile을 요구하고 missing/type
  mismatch를 별도 typed reason으로 반환한다.
- PlayerUnit은 active hostile lifecycle, air eligibility, usefulness와 reach/range, PlayerTile은 valid tile과
  reach/range를 검사하며 empty tile도 허용한다.
- `PendingSkillCast`가 target identity와 acceptance tile을 함께 저장한다. Direct invalid Unit은 execution no-op,
  TileArea는 target 이동/사망과 무관하게 accepted tile에 남고 cast/resonance는 refund/rollback하지 않는다.
- Automatic/manual pending casts도 같은 accepted-tile carrier를 사용해 canonical area capture 의미를 공유한다.

### 결과에 따른 판단

Player-selected retarget runtime은 필요하지 않으며 data validation에서 `RetargetOnStep` 조합을 금지하는 것이
정책과 정확히 맞는다.

## 2026-08-06 — Authoritative focus와 server current contract

### 검증하려던 가설

Battle-local focus에 purpose/caster/skill/restore playback만 저장하면 hover state나 durable save를 추가하지 않고
slow-motion selection, Pause와 reconnect를 원자 구현할 수 있다.

### 실제 결과

- Begin은 matching player selection의 base preflight 뒤 focus를 저장하고 unpaused effective speed를 X0_5로 만든다.
  Pause에서 시작하면 frozen speed/state를 유지한다.
- Preview는 matching focus/current revision의 candidate를 읽기만 한다. Cancel은 exact playback을 복원하고 성공한
  activation은 ManualCastStart/focus clear/playback restore를 one revision에 commit한다. 실패는 focus까지 보존한다.
- Checkpoint/resync/root state는 focus purpose/caster/skill/restore playback을 그대로 투영한다. Focus 중
  Pause/Resume/SetSpeed는 restore intent와 effective focus speed를 분리한다.
- Action capability는 focus 전 Begin, focus 중 Activate/Preview/Cancel을 현재 update에 투영한다. Audit 중 발견한
  stance/relocate/swap/focus result의 capability attachment 누락도 common battle update path에서 바로잡았다.
- game_server strict behavior serde, checkpoint focus serialization, generic battle update mapping과 read-only
  `SkillTargetPreview` result/no trailing snapshot을 고정했다.

### 결과에 따른 판단

Focus는 active battle state뿐이며 별도 semantic event나 battle event-log version 증가가 필요 없다. Manual cast
start event는 기존 version 30 shape로 exact accepted target을 기록한다.

## 2026-08-06 — Focused와 broad 검증

### 실제 결과

- Core explicit selection/target/capture/invalidation focused tests — pass
- World focus preview/failure/cancel/Pause/reconnect/success/capability atomic test — pass
- Strict selection RON/live matrix and game_server request/result/serialization tests — pass
- `cargo test -p game_core --quiet` — 903 passed total
  (library 855 + integration 3 + 4 + 24 + 13 + 4)
- `cargo test -p game_server --quiet` — 56 passed
- `cargo test -p game_core --test ron_loading --quiet` — 24 passed
- `cargo check --workspace --all-targets` — pass
- `cargo fmt --all -- --check`, `git diff --check`, legacy/placeholder/static searches — pass
- Failed/ignored — 0

Strict `cargo clippy --workspace --all-targets -- -D warnings`는 81 lib/130 test 수준의 existing tree-wide lint
debt로 실패했다. Phase 8에서 새로 만든 too-many-arguments/clone-on-copy lint는 제거했으며 clippy는 required gate가
아니다.
