# DefenseRoute 전술 지휘 Phase 7 — Experiments

Status: complete

이 문서는 Phase 7에서 실제 수행한 조사, 변경과 검증만 시간순으로 기록한다.

## 2026-08-06 — Phase Goal 초기화

### 검증하려던 가설

Phase 6가 Complete/High이고 Phase 7만 next이며 새 child path가 기존 Goal과 충돌하지 않는다.

### 실제 결과

- Predecessor audit는 Complete/High다.
- `defense_route_tactical_command_phase_07_atomic_swap`은 새 path다.
- Start commit과 Phase 0~6/user dirty 보존 경계를 유지한다.

### 결과에 따른 판단

Master에서 Phase 7만 active로 전환하고 production discovery를 시작한다.

## 2026-08-06 — Batch seam과 command 경계

### 검증하려던 가설

Phase 6 relocation의 occupancy/effect mutation을 pair-aware preflight와 fallible-free commit helper로 분리하면
중간 위치를 노출하지 않고 두 unit을 한 Core command에서 교환할 수 있다.

### 실제 결과

- Occupancy helper는 제외 ID slice를 받아 pair 자신은 제외하되 같은 tile의 제3 Player actor를 계속 찾는다.
- Relocation의 destination profile, engagement release, body/facing/effect/cooldown mutation을 재사용 가능한 Core
  seam으로 분리했다. Single relocation regression과 전체 library가 그대로 통과했다.
- `validate_tactical_swap`은 두 source의 Defense/Hard CC/unreleased action/cooldown/exclusive movement와 두 current
  destination의 valid/static/occupancy/affinity/profile을 첫 mutation 전에 검사한다.
- Commit은 양쪽 관계 release → 두 position/effect update → one block refresh/event/basic/autocast scheduling 순서다.

### 결과에 따른 판단

Recoverable validation failure 뒤 partial first-unit state가 없고 sequential relocation/fallback도 필요하지 않다.

## 2026-08-06 — World/DTO/server current cutover

### 검증하려던 가설

World가 current positions에서 destination zone kind를 해석하고 existing relocation policy를 한 번만 결제하면
별도 swap tuning/save state 없이 strict transaction을 projection할 수 있다.

### 실제 결과

- `PlayerBehavior::TacticalSwapUnits`는 두 employee UUID와 두 explicit facing만 받는다. Destination/path extension은
  strict serde에서 거부한다.
- World는 서로 다른 roster employee와 deployed instance를 확인하고 양쪽 zone kind/Core preflight 후 cost 5를
  한 번 지불하며 deployed facing 두 개를 바꾸고 projection을 한 번 commit한다.
- Result/action capability/presentation event/game_server generic battle-update mapping을 current shape로 cutover했다.
- `UnitsTacticallySwapped` semantic event 하나에 양쪽 logical/world/facing과 shared cooldown을 기록하고 event-log
  version을 29에서 30으로 올렸다. Spawn/death validators는 두 unit reference를 모두 검사한다.

### 결과에 따른 판단

Active battle은 durable run save payload가 아니며 existing checkpoint tactical state가 두 per-unit deadline을
소유하므로 save/data migration과 compatibility reader가 필요 없다.

## 2026-08-06 — Focused와 broad 검증

### 실제 결과

- `cargo test -p game_core --lib tactical_swap --quiet` — 3 passed
- paused live swap success/same/resource/cost-once/revision-once/DTO/source-event regression — pass
- `cargo test -p game_server deserializes_live_battle_requests --quiet` — 1 passed
- `cargo test -p game_core --lib --quiet` — 850 passed
- `cargo test -p game_core --quiet` — 898 passed total
- `cargo test -p game_server --quiet` — 55 passed
- `cargo test -p game_core --test ron_loading --quiet` — 24 passed
- Core/game_server all-target check, fmt check와 `git diff --check` — pass
- Failed/ignored — 0

`cargo clippy --all-targets -- -D warnings`는 Phase 7과 무관한 existing tree-wide lint debt 때문에 gate로 사용할 수
없었다. Required compile/test/fmt/diff gate는 모두 warning 없이 통과했다.
