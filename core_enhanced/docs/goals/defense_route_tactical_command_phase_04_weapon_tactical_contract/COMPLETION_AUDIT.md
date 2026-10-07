# DefenseRoute 전술 지휘 Phase 4 — Completion Audit

Status: Complete

Confidence: High

Goal:
[Phase 4 무기 전술 계약](PLAN.md)

Policy Authority:
[DR-10, DR-11](../../decisions/defense_route_tactical_command_policy_2026-08-06.md)

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

Audit Date: 2026-08-06

## 판정

Phase 4는 `Complete / High`다. First-live weapon 30개가 required affinity/base block/Ground
DefenseFocus/Platform Rest profile을 직접 소유하며 exact tuning과 invalid schema가 startup gate로 검증된다.
Starter 6명은 일반 reward pool 밖 training sword를 받고, 비무장 직원은 safe-zone에 남되 condition-priority
`weapon_required`로 battle gameplay에서 거부된다. Active-run Core payload는 V3-only로 strict cutover됐고
Core/server/live RON/canonical과 broad suite가 같은 계약을 증명한다.

## Evidence Matrix

| ID | 완료 조건 | Runtime/data/contract evidence | Validation evidence | 판정 |
| --- | --- | --- | --- | --- |
| P4-01 | Required per-weapon profile와 exact 30 catalog | `WeaponCombatProfile`, live equipment RON 30개 | exact count/archetype/tuning, missing/unknown/mismatch/zero tests | pass |
| P4-02 | Affinity와 base block single authority | weapon apply path, base-only fragment/certified consumers | all current archetypes와 certified-power regressions | pass |
| P4-03 | Starter sword와 reward exclusion | 6 employee initial loadouts, exclusion filter/validation | live catalog + new-run owned/equipped grant | pass |
| P4-04 | Weaponless safe-zone와 typed rejection | safe-zone compatibility helper, armed battle draft/admission | visibility/rearm, deploy/range/manual/reason/immutability tests | pass |
| P4-05 | No-deployable armed meaning | `is_battle_deployable`, recovery path audit | all-weaponless defeat/victory/HQ regressions | pass |
| P4-06 | Strict save/DTO/server cutover | Core payload V3, effective-only affinity DTO, server mapping | current round-trip, V2/missing-field reject, server 55 | pass |
| P4-07 | Canonical/legacy/compatibility/dirty/broad | gameplay/runtime/data canonical, no dual reader/fallback | Core 880, RON 24, static/fmt/diff | pass |

## 핵심 구현 근거

- `core_enhanced/src/game/data/equipment_data.rs`: required tactical fields, required Option deserializer와
  affinity/profile validation
- `game_resources/data/equipments/base.ron`: exact 30 weapon catalog와 approved tuning
- `game_resources/data/employees/definitions.ron`, `events/rewards/base.ron`: six starter swords와 general pool
  exclusion
- `core_enhanced/src/game/battle/types.rs`, `world/combat.rs`: armed employee battle source와 deploy/range gate
- `core_enhanced/src/game/employee.rs`, `world/helpers.rs`, `world/snapshot.rs`: condition-first reason와 safe-zone
  visibility
- `core_enhanced/src/game/combat_power.rs`, `data/skill_fragment_data.rs`: authored base block-only consumers
- `core_enhanced/src/game/world/active_run.rs`: Core payload V3-only header/body contract
- `game_server/src/game/player_game_actor/state.rs`, `handlers.rs`: `weapon_required` mapping와 valid starter fixture
- `core_enhanced/docs/game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md`: current canonical

## 검증 결과

Focused:

- Required tactical schema/zero/mismatch/starter validation unit tests — pass
- Exact catalog/tuning와 starter/reward live RON tests — pass
- Current archetype base block, weaponless safe-zone/rearm와 battle command immutability — pass
- All-weaponless no-deployable and non-recovery HQ paths — pass
- Active-run V3 round-trip, Core V2와 missing tactical field rejection — pass
- Core/server typed `weapon_required` reason mapping — pass

Broad:

- `cargo check -p game_core -p game_server --all-targets` — pass, warning 없음
- `cargo test -p game_core --lib --quiet` — 832 passed
- `cargo test -p game_core --quiet` — 880 passed total
  (library 832 + integration 3 + 4 + 24 + 13 + 4)
- `cargo test -p game_core --test ron_loading --quiet` — 24 passed
- `cargo test -p game_server --quiet` — 55 passed
- `cargo fmt --all -- --check` — pass
- `git diff --check` — pass
- Failed/ignored — 0

## Migration과 compatibility

- Live RON: all weapon tactical keys required; missing/unknown/mismatched fields reject; no alias/default inference
- Starter/reward: explicit sword references와 exclusion list; missing/duplicate exclusion rejects
- DTO: employee base affinity removed, equipped effective affinity optional; stable `weapon_required` reason added
- Save: Core payload V2→V3 strict cutover; host envelope remains strict V3; no dual reader or implicit migration
- Checkpoint/event: battle event-log version change 없음; checkpoint embedded equipment follows Core V3
- Core/game_server는 같은 repository release에서 원자 배포

## Legacy 검색 분류

- `ActiveRunCorePayloadV2`, Core version 2 constant와 production/canonical weapon fallback 검색 hit는 없다.
- Live equipment RON의 `deployment_affinity`, `ground_defense_focus`, `platform_rest` count는 각각 exact 30이다.
- Employee public combat DTO의 affinity hit는 equipped weapon에서 계산한
  `effective_deployment_affinity`뿐이다.
- `WeaponCombatProfile::default`는 Rust fixture convenience이며 live RON missing field reader가 아니다.
- Corroded enemy/defense object affinity는 다른 unit source이고 employee fallback authority가 아니다.

## 비범위 확인

- Unity runtime/contract/UI/tests는 읽거나 변경하지 않았다.
- Phase 5 stance/effect state, cooldown, Rest tick과 DefenseFocus runtime modifier를 추가하지 않았다.
- Phase 6~9 relocation/swap/manual-target/FieldWork placeholder를 추가하지 않았다.
- Phase 0~3 verified output과 unrelated user dirty를 보존했다.

## 잔여 위험

현재 Phase 4 범위의 알려진 correctness gap은 없다. Phase 5는 weapon profile의 authored
DefenseFocus/Rest 값을 runtime effect authority로 사용해야 하며 base-block-only fragment/certified power,
condition-first `weapon_required`와 strict Core V3 의미를 유지해야 한다. Unity consumer adoption은 승인된
비범위다.
