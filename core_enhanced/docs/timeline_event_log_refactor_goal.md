# Timeline/EventLog 리팩토링 Goal

## 배경

`Timeline`은 과거 TFT식 자동전투에서 전투 전체를 미리 산출하고 클라이언트가 재생하는 replay 중심 구조의 이름으로 시작했다. 현재 공식 전투 흐름은 `DefenseRoute` 실시간 전투이며, Unity는 서버 tick으로 생성되는 delta와 snapshot을 받아 전투를 진행한다.

따라서 `Timeline`을 완전히 제거하는 것은 맞지 않다. 현재 코드에서 `Timeline`은 다음 역할을 여전히 수행한다.

- 실시간 전투 중 발생한 사건의 append-only 기록.
- `battle_delta` push와 `RequestBattleState` 응답의 원천 데이터.
- 전투 결과창, 전투 기록 UI, 디버그 로그의 기반 자료.
- 스킬/투사체/버프/사망 처리 테스트의 관측 지점.

문제는 이름과 일부 helper/test/export 흐름이 아직 “전체 전투 replay”처럼 읽힌다는 점이다. 이 goal의 목적은 전투 로그의 책임을 명확히 하고, replay-only 의미를 제거하는 것이다.

## 목표

`Timeline`을 공식적으로 `BattleEventLog` 성격의 전투 사건 로그로 재정의한다. 새 구조는 더 멋진 추상화를 만들기 위한 것이 아니라, 현재 DefenseRoute 실시간 흐름을 더 쉽게 읽고 안전하게 바꾸기 위한 것이다.

완료 후 코드를 읽는 사람이 다음을 명확히 이해할 수 있어야 한다.

- 전투는 전체 replay 산출물이 아니라 live engine tick으로 진행된다.
- `timeline_delta`는 “리플레이 조각”이 아니라 “아직 클라이언트에 전달되지 않은 event log delta”다.
- 전투 종료 후 결과창이 보는 데이터는 replay source가 아니라 이미 발생한 battle event log다.
- 테스트는 전체 replay 재생보다 live state/delta/client-facing 계약을 우선 고정한다.

## 범위

포함한다.

- `src/game/battle/timeline.rs`의 타입 이름, 문서, public API 책임 검토.
- `BattleCore.timeline`, `timeline_entries_after_seq`, `last_pushed_timeline_seq` 등 live delta 흐름 검토.
- `game_server`의 `battle_delta` notification payload와 core event log 용어 일치 여부 검토.
- `docs/unity_core_contract.md`, `docs/game_rulebook.md`의 timeline/replay 표현 갱신.
- replay-only 또는 timeline export 중심 테스트가 아직 공식 검증처럼 남아 있는지 확인.
- 디버그 export가 필요하다면 “디버그 전용”으로 명확히 격리.

포함하지 않는다.

- 스킬 런타임 동작 변경.
- 이동 시스템 내부 리팩토링.
- Unity 클라이언트 UI 구현.
- 전투 밸런스 수치 조정.
- 조건부 BattleScenario 이벤트 2단계 확장.

## 현재 가설

1. `Timeline` 타입 자체는 보존하되, 장기적으로 `BattleEventLog`, `BattleEventLogEntry`, `BattleEventLogDelta` 같은 이름으로 변경하는 편이 낫다.
2. 한 번에 모든 타입명을 바꾸면 영향 범위가 커질 수 있으므로, 먼저 문서/주석/API 책임을 정리하고 실제 rename은 focused test가 충분할 때 진행한다.
3. `timeline_exports`는 공식 전투 흐름이 아니라 디버그 산출물이다. 계속 필요하면 `debug_event_log_exports` 같은 명칭으로 바꾸거나 테스트 전용 helper로 격리한다.
4. `run_battle()`로 전체 전투를 끝까지 실행하는 helper는 스킬/저수준 battle core 테스트에는 유용할 수 있지만, 공식 게임 흐름 테스트의 기준이 되면 안 된다.

## 위험 신호

- `Timeline` 제거를 목표로 잡아 live delta, 전투 로그, 결과창 데이터까지 같이 망가뜨린다.
- 이름만 `EventLog`로 바꾸고 replay-only 테스트/문서 의미는 그대로 둔다.
- 새 추상화 계층을 과도하게 추가해 전투 사건 하나가 어디서 기록되고 어디로 push되는지 더 읽기 어려워진다.
- Unity-facing DTO를 이유 없이 변경해 클라이언트 계약을 불안정하게 만든다.
- 디버그 export를 공식 결과 계약처럼 유지한다.

## 작업 계획

1. 코드 판독

- `src/game/battle/timeline.rs`에서 타입과 이벤트 범위를 읽는다.
- `src/game/battle/core/sim.rs`, `src/game/battle/core/mod.rs`에서 `record_timeline`, `timeline_entries_after_seq`, live tick 결과 연결을 확인한다.
- `src/game/world/combat.rs`, `src/game/world/snapshot.rs`에서 battle state와 snapshot이 timeline을 어떻게 노출하는지 확인한다.
- `../game_server/src/game/player_game_actor`에서 `battle_delta` push mapping을 확인한다.

2. 분류

- live event log로 유지할 것.
- 이름/문서만 바꾸면 되는 것.
- debug-only로 격리할 것.
- replay-only 의미라 삭제할 것.
- 정책 확인이 필요한 것.

3. 리팩토링

- source of truth가 중복되는 경우 하나로 줄인다.
- `timeline_delta`가 client-facing 계약이라면 당장 이름 변경이 위험한지 판단한다.
- 내부명 변경과 외부 DTO 변경을 분리한다.
- compatibility layer를 만들지 않는다. 단, Unity 계약 변경이 필요한 경우 사용자와 의논 후 진행한다.

4. 문서 갱신

- `game_rulebook.md`는 “전투 사건 로그” 관점으로 갱신한다.
- `unity_core_contract.md`는 Unity가 `timeline_delta`를 어떻게 소비해야 하는지 최신 계약으로 정리한다.
- `refactor_preparation_plan.md`에는 replay-only를 되살리지 않는 원칙만 남긴다.

## 검증 계획

빠른 검증부터 수행한다.

```bash
cargo test -p game_core live_defense_playback_pause_freezes_server_tick_and_resume_advances -- --nocapture
cargo test -p game_core defense_live_battle_state_request_returns_timeline_delta_without_advancing_cursor -- --nocapture
cargo test -p game_server live_battle_tick_pushes_delta_and_snapshot_after_confirm_enter -- --nocapture
```

마무리 검증.

```bash
cargo test -p game_core -- --nocapture
cargo test -p game_server -- --nocapture
cargo check -p game_core
cargo check -p game_server
```

## 완료 조건

- `Timeline`의 현재 공식 역할이 replay가 아니라 battle event log임을 코드와 문서가 일관되게 설명한다.
- live battle delta, snapshot, server push가 같은 source of truth를 사용한다.
- 공식 게임 흐름 테스트가 replay-only 산출물에 의존하지 않는다.
- 디버그 export가 남는다면 디버그 전용임이 명확하다.
- 불필요한 legacy replay/helper/test가 발견되면 compatibility layer 없이 제거한다.
- Unity-facing 계약 변경이 필요하면 임의로 확정하지 않고 사용자에게 질문한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Goal 명령어 초안

```text
/goal docs/timeline_event_log_refactor_goal.md를 기준으로, 현재 `Timeline`을 replay-only 산출물이 아니라 DefenseRoute 실시간 전투의 battle event log로 재정의하는 리팩토링을 수행하라. 먼저 `src/game/battle/timeline.rs`, `BattleCore.record_timeline`, `timeline_entries_after_seq`, world snapshot, game_server battle_delta push mapping, 관련 문서를 꼼꼼히 읽고 source of truth 중복과 replay-only 의미가 남은 지점을 분류하라. live delta/result/debug/test에 필요한 event log 기능은 보존하고, 공식 게임 흐름에 없는 replay-only helper/export/test는 compatibility layer 없이 제거하거나 debug-only로 격리하라. Unity-facing DTO 이름 변경이 필요하면 임의로 바꾸지 말고 사용자에게 영향과 선택지를 보고하고 goal을 종료하라. 문서를 무조건 신뢰하지 말고 실제 코드와 live API를 기준으로 더 단순하고 안전한 개선안을 적용하라. 검증은 focused test를 먼저 실행한 뒤 `cargo test -p game_core -- --nocapture`, `cargo test -p game_server -- --nocapture`, `cargo check -p game_core`, `cargo check -p game_server` 순서로 수행하라. 완료 조건은 1) Timeline/EventLog 역할이 코드와 문서에서 일관됨, 2) replay-only 잔재가 정리됨, 3) live battle delta 계약이 유지됨, 4) 필요한 문서가 최신화됨, 5) 검증 명령이 통과함, 6) 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료함이다.
```
