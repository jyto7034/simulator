# Game Rulebook Policy Implementation Master Phase 01 — Monolith Tuning

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes; limited to this Phase contract
Master Goal: `../game_rulebook_policy_implementation_master/`
Phase: P01
Package: `GRB-WP-002`
Primary Finding: `GRB-M-006`
Policy Decision: `GRB-PD-009`
Activated: 2026-08-12
Repository Root: `/mnt/f/work/simulator`
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

Monolith Stage 2/3 stat multiplier의 initial live-RON 값을 `150/200`으로 조정하고, 모든 적용 stat의 percentage
composition이 positive, ordered, checked와 `u32` representable임을 loader에서 검증한다. Reward multiplier,
wave composition과 rounding 의미는 바꾸지 않는다.

## User/Contract-Visible Problem

현재 live 값은 승인된 initial tuning `150/200`이 아니라 `100/125`이고, loader는 positive/ordered만 확인한
뒤 runtime projection에서 saturation을 허용한다. 너무 큰 authored 값이 조용히 `u32::MAX`로 바뀌면 live
RON이 표현할 수 없는 combat stat을 정상 값처럼 publish할 수 있다.

## Audit Finding ID

`GRB-M-006`: Monolith Stage 2/3 initial stat `150/200`과 safe derivation.

## Current Runtime And Live Data Evidence

Completed Preflight 시점 evidence:

- `game_resources/data/run/policy.ron`: stat `100/125`.
- `core_enhanced/src/game/data/run_policy_data.rs`: positive/ordered validation과 saturating projection.
- Focused run-policy 17, full Core 1063, game_server 63 tests green.
- 두 target file은 이미 dirty였지만 target multiplier hunk는 당시 user diff와 겹치지 않았다.

위 항목은 activation baseline을 대체하지 않는다. Production 수정 전 current diff, exact hunk, loader/resolver와
baseline을 다시 확인한다.

## Source Of Truth And Ownership

- Gameplay target: `game_rulebook.md`, `GRB-PD-009`, Master P01 boundary.
- Live values: `game_resources/data/run/policy.ron`.
- Parse/strict validation/derived projection: `core_enhanced/src/game/data/run_policy_data.rs`와 실제 resolver.
- Test evidence: run-policy unit tests와 production RON integration tests.
- Goal docs는 execution evidence이며 runtime/data authority를 대신하지 않는다.

## Invariants

- Stage 2/3 stat multiplier는 모두 `> 0`이고 Stage 3가 Stage 2보다 크다.
- Monolith role에 적용되는 target power, max HP, attack과 defense percentage composition은 checked 계산에
  성공하고 결과가 `u32`에 정확히 표현돼야 한다.
- Non-representable 값은 live bundle publication 전에 reject하며 saturation으로 수용하지 않는다.
- Initial production RON 값은 Stage 2 `150`, Stage 3 `200`이다.
- Stage 2/3 reward multiplier, generated wave budget, extra waves, rounding, node topology와 public DTO/save는
  변경하지 않는다.
- Gameplay maximum을 새로 정하지 않는다. Safe range는 현재 composition 대상의 representability 경계다.
- Existing unrelated dirty changes를 보존한다.

## Approved Policy And Evidence

- Master P01 contract boundary와 `GRB-PD-009`.
- Preflight `PF-01 Safe-range technical contract`, `MON-06`, `BOOT`와 `PF-07 WP002` handoff.
- Preflight completion verdict `Complete / High`, `ready_for_wp002`.
- 사용자의 initial tuning 결정: `150/200`, 개발 과정에서 live RON으로 조절 가능.

## Open Policy Decisions

없음. Arbitrary gameplay cap, reward/wave change 또는 새로운 balance 의미가 필요해지면 진행하지 않는다.

## In Scope

- Current dirty-hunk/authority와 focused/full baseline 재확인
- Target-first tests: live `150/200`, every derived stat, zero/reversed/non-representable rejection
- Live RON stat 값 변경
- Loader/resolver의 checked representability validation
- 필요한 focused test fixture와 durable canonical clarification
- Focused and broad Core/live-RON validation 및 Phase completion audit

## Out Of Scope

- Reward multiplier, wave budget, extra wave, rounding, encounter/reward progression
- Active-run/save/DTO/server mapping과 Unity consumer
- P02~P04 state/schema 선행 구현
- Final content, balance cap, WP005 fragment skills와 unrelated cleanup

## Implementation Scope

Test-first로 invalid representability와 derived every-stat assertion을 추가한 뒤 최소 production loader/RON
경계를 수정한다. Resolver가 이미 validated data만 소비한다면 saturation을 제거하거나 unreachable하게
만든다. Public type/schema 변경 없이 strict live-RON publication gate에서 invalid tuning을 막는다.

## DTO/Data/Save Migration

- Data: live RON stat values `100/125 → 150/200`.
- DTO/save/server: 변경 없음.
- Old live values를 읽는 fallback/default 또는 dual schema 없음.

## Deployment And Compatibility

Development-only project이므로 public release gate는 없다. Production bundle은 strict loader를 통과한 한
set의 current values만 publish한다. Invalid data를 이전 값으로 대체하거나 saturating compatibility로
숨기지 않는다.

## Focused Validation

- `cargo test -p game_core data::run_policy_data -- --quiet`
- 필요 시 exact run-policy integration target
- Assertions:
  - production live RON `150/200`
  - zero/reversed rejection
  - target power, max HP, attack, defense every-stat derived values
  - multiplication 또는 projection non-representability rejection
  - reward/wave/rounding values unchanged

Actual commands, executed tests와 pass/fail/skip은 `EXPERIMENTS.md`에 기록한다.

## Broad Validation

- `cargo test -p game_core -- --quiet`
- Production live-RON integration tests가 full Core command에서 non-zero로 실행됐는지 확인
- Scoped `git diff --check`와 target diff review

P01은 public DTO/server를 변경하지 않으므로 game_server suite는 필수 completion runner가 아니다. Master의
최종 broad audit와 P02 activation이 current server baseline을 다시 소유한다.

## Omitted Validation

- Unity source/EditMode/PlayMode/live client: P01은 public DTO/wire를 변경하지 않으며 사용자 승인에 따라
  현재 drifted Unity project 검사를 생략한다. Unity correctness를 주장하지 않는다.
- Final balance/playability: `150/200`은 승인된 initial tuning이고 balance polish는 별도 future work다.

## Completion Conditions

1. Current target hunk가 기존 user change와 안전하게 병합 가능하고 unrelated diff가 보존된다.
2. Live RON Stage 2/3 stat 값이 정확히 `150/200`이다.
3. Positive/ordered/checked/exact representability validation이 publication 전에 실행된다.
4. Every applied target-power/HP/attack/defense derivation이 target-first test로 증명된다.
5. Zero, reversed와 non-representable input이 fallback/saturation 없이 reject된다.
6. Reward multiplier, wave budget, extra wave와 rounding 의미가 변경되지 않는다.
7. Focused non-zero suite와 full Core/live-RON suite가 unexplained failure 없이 통과한다.
8. Scoped diff/canonical/legacy/compatibility 검토와 독립 completion audit가 `Complete / High`를 판정한다.
9. Master registry와 Phase experiment 기록이 실제 결과로 갱신된다.

## Completion Audit Evidence Matrix

| Condition | Required evidence | Current verdict |
| --- | --- | --- |
| C1 dirty preservation | activation diff/hunk inspection + final scoped diff | pass |
| C2 live 150/200 | production RON assertion/integration load | pass |
| C3 strict validation | loader code + invalid fixture tests | pass |
| C4 every-stat derivation | exact derived assertions | pass |
| C5 invalid rejection | zero/reversed/non-representable tests | pass |
| C6 adjacent semantics | before/after values + regression tests | pass |
| C7 runners | focused/full exact counts | pass |
| C8 audit | `COMPLETION_AUDIT.md` | `Complete / High` |
| C9 Master handoff | Master PLAN/experiments status | pass |

## Stop Conditions

- Current target hunk overlaps an unowned existing change that cannot be safely preserved.
- Baseline runner has an unexplained failure relevant to target files.
- Safe validation requires an arbitrary gameplay cap instead of exact representability.
- Correct implementation requires reward/wave/rounding, DTO/save/server 또는 P02 authority 변경.
- Invalid values can only be hidden with fallback/default/saturation.
- Required non-zero Core/live-RON runner is inaccessible.
