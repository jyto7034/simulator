# DefenseRoute 전술 지휘 Phase 6 — Experiments

Status: complete

이 문서는 Phase 6에서 실제 수행한 조사, 변경과 검증만 시간순으로 기록한다.

## 2026-08-06 — Phase Goal 초기화

### 검증하려던 가설

Phase 5가 Complete/High이고 Phase 6만 next이며 새 child path가 기존 Goal과 충돌하지 않는다.

### 실제 결과

- Predecessor audit는 Complete/High다.
- `defense_route_tactical_command_phase_06_relocation_occupancy`는 새 path다.
- Start commit과 Phase 0~5/user dirty 보존 경계를 유지한다.

### 결과에 따른 판단

Master에서 Phase 6만 active로 전환하고 production discovery를 시작한다.

## 2026-08-06 — Runtime/data/server seam discovery와 baseline

### 검증하려던 가설

같은 `RuntimeUnit` body를 tile center로 원자 갱신하고 기존 tracking consumer가 live/stored geometry를 계속
사용하게 하면 별도 player movement/reservation system 없이 정책을 구현할 수 있다.

### 수행한 확인

- `UnitBody`, active movement segment, block map/target preference와 player fixed-defense lifecycle 조사
- deploy/redeploy world zone/affinity/resource/revision path와 final Core spawn commit 조사
- basic/skill projectile guidance와 persistent area GroundFixed/FollowCaster/FollowTarget geometry 조사
- Phase 5 tactical state/focus/Rest epoch와 event/checkpoint/server projection 조사
- focused baseline:
  - shared allied deployment가 기존에 성공하는지 확인
  - fixed-defense block priority가 유지되는지 확인
  - Pause가 battle clock을 동결하는지 확인

### 실제 결과

- 기존 deploy는 같은 allied logical tile을 허용해 `DR-05` correction이 필요했다.
- `UnitBody`가 authoritative continuous position이며 range/target/projectile/follow-area consumer는 body를 읽었다.
- `GroundFixed`/Fixed는 stored geometry/aim을, Follow/Homing은 live body를 읽어 relocation이 delivery를 재생성할
  필요가 없었다.
- Active battle은 durable run save 밖이고 checkpoint/resync는 같은 in-memory Core state projection이다.

### 결과에 따른 판단

공통 logical occupancy validator와 같은-instance body transaction을 만들고 event-log만 strict bump한다.

## 2026-08-06 — Relocation authority와 common occupancy 구현

### 수행한 변경

- required `tactical_relocation(cost: 5, cooldown_ms: 10000)` policy와 employee-local tactical cooldown 추가
- `TacticalRelocateUnit` Core/world command, outcome/result, action capability와 typed reasons 추가
- `player_tactical_slot_occupant`/`validate_player_tactical_destination`을 구현하고 final deploy/redeploy commit에도 적용
- Active player combatant/defense-object occupancy, source exclusion, enemy overlap과 no push/swap 의미 구현
- exact no-op reject와 same-position facing-only success 구현
- zone/weapon affinity는 world+Core에서, valid/static/occupancy는 Core에서 final revalidation

### 실제 결과

- Attack, Hard CC, basic windup, pending skill cast, cooldown, resource, invalid/static/zone/affinity/occupied/no-op이
  mutation 전에 실패한다.
- Silence와 현재 block은 gate가 아니며 paused frozen battle time에서도 성공한다.
- 실패는 body/facing/resource/cooldown/effect/event/revision/salt/group을 바꾸지 않았다.
- 최초/재배치는 DefenseObject가 점유한 live route endpoint도 피하도록 smoke test가 current rule을 사용한다.

### 결과에 따른 판단

Occupancy는 player-only logical slot authority이고 physics collision이나 enemy body overlap과 결합하지 않는다.

## 2026-08-06 — Spatial/engagement/tracking과 tile effect transaction

### 수행한 변경

- relocation source의 old block 관계/target만 release하고 body position/previous position/facing/anchor를 같은
  transaction에서 갱신한 뒤 deterministic `refresh_block_state` 수행
- HP, buff, resonance, stance, stance cooldown, basic/ranged cooldown과 runtime identity 보존
- active projectile/area runtime을 건드리지 않고 기존 guidance/tracking consumer 재사용
- Any-profile Ground↔Platform DefenseFocus/Rest/block 전환과 Rest epoch/full interval 처리
- same-tile facing-only에는 tile effect/Rest epoch를 보존

### 실제 결과

- Old enemy engagement는 해제되고 새 위치 enemy가 같은 blocker를 deterministic하게 획득했다.
- Homing aim은 relocated target, Fixed aim은 launch 좌표를 유지했다.
- GroundFixed center는 고정되고 FollowCaster/FollowTarget center는 relocated live body로 바뀌었다.
- Ground→Platform은 focus 10/block 5를 제거하고 block 0/새 Rest deadline, Platform→Ground는 focus/block을
  재계산했다. Stance cooldown은 변하지 않았다.

### 결과에 따른 판단

새 spatial registry, walking, arrival event 또는 delivery compatibility layer는 필요하지 않았다.

## 2026-08-06 — DTO/event/server/data strict cutover

### 수행한 변경

- deployed/checkpoint tactical DTO에 relocation deadline과 resource-aware typed eligibility 추가
- `UnitTacticallyRelocated` internal/presentation event와 event-log strict version 29 추가
- death/spawn validators에 exact unit reference/dead-operation 의미 추가
- game_server request deserialize와 result battle-update mapping 추가
- required policy missing/zero/bounds/unknown future-focus rejection 추가
- rulebook/runtime/data/skill-target canonical 갱신

### 실제 결과

- World result/checkpoint/event가 같은 destination/facing/cooldown/resource/revision을 투영했다.
- Source command ID가 relocation presentation event에 보존됐다.
- `path` command field와 Phase 8 `focus_ms` policy field는 strict unknown field로 거부됐다.
- Redeploy의 새 runtime instance는 relocation deadline 0으로 시작했다.

### 결과에 따른 판단

Active-run save migration이나 setup placeholder는 없고 Core/game_server current release가 하나의 strict shape다.

## 2026-08-06 — Focused/broad/static/canonical 완료 감사

### 수행한 검증

- `cargo test -p game_core --lib relocation --quiet`
- occupied deploy, strict relocation policy와 live paused world command focused tests
- `cargo test -p game_core --lib --quiet`
- `cargo test -p game_core --quiet`
- `cargo test -p game_core --test ron_loading --quiet`
- `cargo test -p game_server --quiet`
- Core/server `cargo check --all-targets`
- Core/server `cargo fmt --all -- --check`
- `git diff --check`와 Phase 7~9/legacy/placeholder 검색

### 실제 결과

- Relocation focused 8 passed; occupied deploy/policy/world/server focused도 pass
- Core library 847, Core 전체 895, server 55, RON 24 passed; failed/ignored 0
- Core/server all-target check warning 없이 pass
- fmt/diff check pass
- `focus_ms`는 rejection fixture, unrelated `FreeSwap`은 기존 skill-fragment 정책뿐이며 relocation swap/path/
  FieldWork placeholder는 없다.
- 기존 Phase 0~5와 unrelated user dirty, 외부 Unity protocol dirty를 보존했고 Unity 범위를 읽거나 수정하지 않았다.

### 결과에 따른 판단

Phase 6를 [Complete / High](COMPLETION_AUDIT.md)로 닫고 Phase 7만 다음 active phase로 전환한다.
