# DefenseRoute 전술 지휘 Phase 5 — Experiment Notes

Status: complete

## 초기 구조 판단

- Child 이름은 stance state와 Ground/Platform derived effect 경계를 함께 표현하는
  `defense_route_tactical_command_phase_05_stance_tile_effects`로 정했다.
- Phase 5는 `DR-06`~`DR-09`만 소유하며 relocation/swap/manual targeting/FieldWork를 선행하지 않는다.
- Phase 4 authored weapon profile은 immutable input이며 base-block-only consumer 의미를 유지한다.

## 초기 dirty worktree

- Phase 0~4 verified production/data/server/canonical/test/Goal output이 dirty 상태로 함께 존재한다.
- 기존 사용자 README/workflow/audit/skill 문서와 Unity protocol dirty를 보존한다.
- Production 변경 전 대상 파일별 diff를 읽어 선행 Phase hunk와 구분한다.

## Source-of-truth 판단

- 정책 source는 decision record `DR-06`~`DR-09`다.
- Authoritative stance/cooldown/effect scheduler owner는 `BattleCore::tactical_states`다.
- Deployment zone kind는 world가 validated preview에서 확정해 Core command로 넘기고 Core는 draft weapon profile을
  다시 확정한다.
- Focus damage는 existing `RuntimeUnit.incoming_damage_modifiers`, focus block은 runtime `block_capacity`, Rest는
  event heap이 실행 authority다.
- Phase 1 committed/released action, Phase 2 Hard CC/Silence, Phase 3 tactical resource와 Phase 4 weapon
  authority가 predecessor invariants다.

## 외부 소비자와 비범위

- game_server current transport/persistence는 접근 가능한 consumer로 검증한다.
- Unity runtime/contract/tests는 사용자 승인 비범위라 읽거나 변경하지 않는다.

## 구현 판단

- `UnitTacticalState`는 stance, tile, weapon profile, base block, cooldown과 Rest epoch/deadline만 저장한다.
  `DefenseFocus`와 `Rest` enum은 projection 표현일 뿐 authoritative state가 아니다.
- Stance 전환은 action lock을 검사하지 않으므로 windup/focus 중 허용된다. Hard CC만 전환을 막으며 Silence는
  막지 않는다.
- Defense gate는 새 regular basic attack과 auto/manual skill admission에만 있다. Committed resolve/end와
  triggered attack/delivery 경로는 gate를 다시 통과하지 않는다.
- Focus reduction은 현재 runtime general reduction에 bonus를 가감해 다른 runtime modifier를 보존한다.
- Rest event priority 12는 AttackResolve 11 뒤, AutoCastStart 13 앞이다. Projectile/area/buff/skill damage도 더
  앞 priority이므로 same-time damage-first가 하나의 heap ordering으로 보장된다.
- Rest max resonance는 pending cast를 만들지 않고 Attack 복귀가 current/max를 재평가한다.
- Hard CC는 Rest HP/resonance 모두 skip하고 cadence를 유지한다. Resonance gain lock은 Core add-resonance gate로
  resonance만 skip하므로 HP Rest는 유지된다.
- Active battle이 durable save payload가 아니므로 Core active-run/save version은 Phase 4 V3 그대로다. Event-log
  shape는 `StanceChanged` 때문에 strict version 28로 올렸다.

## DTO와 compatibility 판단

- Player command는 exact `set_engagement_stance { employee_uuid, stance: attack|defense }`다. Derived effect 이름은
  deserialize되지 않는다.
- Deployment는 tactical state를 required로, mixed owner checkpoint는 player live-deployed unit에 `Some`, 그 외
  unit에 `None`으로 투영한다.
- Legacy alias/default/dual schema는 없다. Run policy `tactical_stance` 두 field도 required다.

## Dirty와 범위 판단

- Phase 0~4 production/data/test/canonical diff를 보존했다.
- 기존 사용자 README/workflow/review/skill 문서와 Unity protocol dirty를 수정 대상으로 삼지 않았다.
- Phase 6 relocation/occupancy, Phase 7 swap, Phase 8 targeting, Phase 9 FieldWork placeholder는 추가하지 않았다.

## 다음 안전한 작업

1. all-target check, fmt/diff/static gate를 완료한다.
2. Phase 5 completion audit를 Complete/High로 닫는다.
3. Master에서 Phase 5를 complete, Phase 6만 active로 전환한다.
