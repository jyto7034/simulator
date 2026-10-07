# Boss Runtime v1 P00 — Active Work Collision Audit

Status: complete

Last Updated: 2026-08-09

## Scope and interpretation

이 문서는 Boss Runtime P01~P07이 현재 사용자 소유 working tree와 충돌할 수 있는 경로를 기록한다.
Goal status 충돌과 dirty-path 충돌은 별개로 판정한다. `complete` Goal의 변경이라도 commit되지 않은 파일은
Boss 구현이 덮어쓰거나 되돌려서는 안 되는 사용자 변경이다.

## Active Goal inventory

정확한 nonterminal status 검색 결과 현재 active Goal은 다음 둘뿐이다.

- `boss_runtime_v1_implementation_master`
- `boss_runtime_v1_implementation_master_phase_00_execution_preflight`

별도 active/incomplete Goal owner는 발견되지 않았다. Defense Route Tactical Command Master는
`Status: complete`, `Implementation Complete: true`, `Current Phase: none — all phases complete`이며 Phase 0~9
registry도 모두 complete다. 따라서 동시 실행 owner 충돌은 없지만, 그 구현은 현재 simulator working tree에
대규모 dirty/untracked 변경으로 남아 있다.

## Current dirty-path collision registry

| Boss phase | Current overlapping user-owned paths | Collision class | P00 disposition |
| --- | --- | --- | --- |
| P01 lifecycle correctness | `battle/core/mod.rs`, `battle/enums.rs`, `world/combat.rs`, 관련 `world/tests/*` | direct production and test overlap | blocker 아님. current file을 새 baseline으로 읽고 symbol-level patch 및 사전 focused characterization을 handoff에 고정해야 함 |
| P02 catalog/setup | `battle/core/build.rs`, `battle/scenario.rs`, `battle/types.rs`, `combat_setup/*`, `data/live_ron.rs`, `data/pve_data.rs`, `data/validation.rs`, `game_resources/data/pve/encounters.ron`, `tests/ron_loading.rs` | direct schema/setup/live-data overlap | P02 시작 시 current typed setup/version을 재확인하고 같은 파일의 기존 변경을 보존해야 함 |
| P03 Boss runtime | `battle/core/mod.rs`, `battle/core/types.rs`, `battle/enums.rs`, `battle/event_log.rs`, `battle/scenario.rs` | direct runtime/state/event overlap | P03 atomic slice가 기존 전술 runtime과 같은 authority를 공유함. 별도 병렬 patch 금지, current abstractions 위에 통합 |
| P04 tracking vertical slice | `battle/core/movement/*`, `battle/core/targeting.rs`, `battle/core/target_usefulness.rs`, `battle/core/skill_runtime/area.rs` | direct movement/target overlap | current tactical relocation/manual-target contracts를 preserved invariant로 특성화한 뒤 Boss opt-in seam만 추가 |
| P05 first live Boss | `game_resources/data/pve/encounters.ron`, loader/validation/setup files | direct live-data publication overlap | content 승인 뒤 strict current schema로 한 번에 publish; 기존 encounter 변경을 덮어쓰지 않음 |
| P06 conditional public wire | `game_server/src/game/player_game_actor/{handlers,messages,state}.rs`, `game_server/docs/unity_bridge_protocol.md`; Unity `GameWireMessage.cs`, `BattleRuntimeState.cs`, `GameSnapshotStore.cs` 및 contract/live-flow tests | producer/consumer and external dirty overlap | 실제 public contract가 필요할 때만 activation. Core/server/Unity를 하나의 동기화 단위로 재-baseline |
| P07 final audit | 위 전체 및 canonical 문서 | audit evidence overlap | 최종 diff ownership을 Boss-owned hunk와 pre-existing user change로 분리해 보고 |

Simulator의 production overlap에는 Defense Route Tactical Command가 추가한 untracked runtime 파일
`field_work.rs`, `tactical_relocation.rs`, `tactical_stance.rs`, `tactical_swap.rs`도 포함된다. 이 파일들은 삭제·이동·
정리 대상이 아니며 현재 runtime dependency로 취급한다.

Unity repository도 별도 dirty 상태다. P00은 이를 읽기만 하며, simulator 저장소의 `git status` 대상에 외부
Unity 경로를 섞지 않는다.

## Ownership decision

- 현재 active production Goal owner 충돌: 없음.
- 현재 dirty-path 충돌: 있음, 모든 관련 변경은 사용자 소유 baseline.
- P01 진입 가능성: symbol-level current-tree audit와 safety-net command가 확정되면 가능. dirty 상태 그 자체는
  중단 사유가 아니지만 whole-file replacement, restore, cleanup 또는 과거 commit 기준 patch는 금지한다.
- P02~P06: 각 Phase activation 시 이 registry를 current status와 다시 대조한다.

## Closure

- P01 세 G0 defect는 current source에서 모두 재확인됐고 `P01_G0_HANDOFF.md`에 exact owner/test/failure seam이
  고정됐다.
- P02/P03 setup/runtime 경계와 active-battle lifetime은 current startup-only immutable data 및 in-memory
  session topology에 맞는다.
- P04는 ordinary movement/target leaf를 보존하는 Wire-neutral opt-in seam으로 유지된다.
- P06 여부는 승인된 live reference의 actual public presentation contract가 생기는 P05 latest decision point로
  명시됐으며 P01을 막지 않는다.
- P00 경로를 제외한 final precheck status는 초기 `221` entries 및 SHA-256
  `41f02b5b73145453f032c3dfcf2931f1225db3225cb70a7ccaf82de5e882bd8d`와 정확히 같다.

No ownership ambiguity or dirty-preservation blocker remains for P01.
