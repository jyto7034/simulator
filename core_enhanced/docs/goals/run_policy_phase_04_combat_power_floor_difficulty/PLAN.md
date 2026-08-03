# Certified Combat Power And Floor Difficulty Goal

Status: complete; Complete/High 2026-07-27

Implementation complete: true

Goal name: `run_policy_phase_04_combat_power_floor_difficulty`

Master Goal: `run_persistence_facility_monolith_policy_master`

Master Phase: P04

Created: 2026-07-27

Execution authority: 2026-07-27 사용자 요청

Direct predecessor:
- `../run_policy_phase_03_facility_map_manifestation/COMPLETION_REVIEW.md`
  — Complete / High

## Objective

실제 합법적인 roster/loadout에서 versioned provisional certified combat power를 Core가 계산하고,
run-wide non-rewind peak와 새 Floor의 monotonic power anchor를 저장한다. P03의 한 번뿐인 Floor generation
transaction에서 ordinary Combat band quota, role difficulty, target power, encounter/wave budget과 reward
contract를 확정하고 Unity에는 exact 숫자가 아닌 band/relative threat/readiness만 투영한다.

## Execution Sources

- Master P04 scope/exit gate/seams: `../run_persistence_facility_monolith_policy_master/PLAN.md`
  8~10, 15, 18~24절
- Approved policy: `../../decisions/combat_power_floor_difficulty_policy_draft_2026-07-26.md`
- P03 review: `../run_policy_phase_03_facility_map_manifestation/COMPLETION_REVIEW.md`
- Canonical: `../../game_rulebook.md`, `../../core_runtime_contract.md`,
  `../../data_loading_contract.md`
- External Unity: `/mnt/f/unity projects/ark/docs/unity_core_contract.md`,
  `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`,
  `/mnt/f/unity projects/ark/docs/ui/screen_spec.md`

## Readiness Result

- P03가 complete/High이고 facility template/profile selection, deterministic full assignment,
  checkpoint/active-run map state와 Unity exact template resource가 녹색이다.
- 현재 persistent combat profile pipeline은 직원 base stats, level/growth, equipped equipment/enhancement,
  equipped skill fragments, weapon attack interval/range/block를 결정론적으로 resolve할 수 있다.
- 현재 `floor_combat_scaling`은 Floor 번호만으로 enemy stats/generated-wave budget을 정하고 overflow를
  last stage로 clamp한다. Certified power, run peak, anchor, node band/role target/reward contract는 없다.
- Policy 2.2와 9절은 provisional coefficient/model version/Floor minimum-rise/band-role 수치를 이 phase의
  live balance authoring으로 명시한다. 최종 playtest coefficient 승인이 아니라 deterministic provisional
  system을 구현하는 범위이므로 추가 정책 질문 없이 진행 가능하다.
- Unity MCP `ark@b49d32f6f2f70bbe`는 compile/runner를 제공하며 P03 broad baseline이 확보됐다.

Readiness: **ready**.

## Approved Boundary

- Certified raw power는 persistent battle inputs만 사용한다. HP/Trauma/Breakdown, active consumable,
  temporary battle buff와 미장착 inventory는 제외하고 별도 readiness/risk에만 반영한다.
- Floor 1은 starter selection + initial loadout commit의 certified snapshot을 사용한다. 이후 run peak는
  실제 Combat 진입을 확정할 때 현재 출전 가능한 roster/loadout의 certified power로만 갱신한다.
- Run peak는 checkpoint payload가 되감지 못하는 active-run monotonic metadata다. Current checkpoint보다
  낮은 import가 peak를 낮추지 않는다.
- Floor anchor와 모든 node difficulty는 Floor creation transaction에서 한 번 확정한다. Same-Floor
  loadout/roster 변화, reconnect/map request/checkpoint load는 다시 계산하지 않는다.
- Ordinary Combat만 Easy/Standard/Hard quota를 소비한다. Elite/NormalBoss/FinalBoss/BossOmen과 P05
  remanifestation은 typed role profile을 쓴다.
- `floor_combat_scaling`과 clamp fallback은 final model과 경쟁하지 않게 제거한다. 구 schema alias,
  dual-read와 default difficulty를 두지 않는다.
- Unity는 exact power/target/recommended score를 표시하지 않고 authored band, relative threat와 readiness
  warning만 Core projection에서 읽는다.

## Out Of Scope

- 대표 콘텐츠가 완성된 뒤의 headless 승률 calibration과 최종 coefficient 승인
- 자동 편성/승패 예측/추천 power 숫자
- 완료 node replay item과 반복 보상
- P05 monolith eligibility, merchandise, remanifestation trigger, hidden-boss content
- P03 topology/manifestation 의미 변경
- 과거 배포 save compatibility 또는 schema migration alias

## Implementation Plan

1. Current effective-profile/stat/equipment/fragment pipeline, Floor creation, encounter assignment/preview,
   rewards, checkpoint/active-run monotonic host envelope와 Unity node preview를 상세 감사한다.
2. Versioned provisional power model, Floor schedule, ordinary band quota/rounding, role profiles,
   target/budget/reward modifiers를 live strict schema로 저작하고 overflow/coverage/reference를 검증한다.
3. Persistent-only employee certified profile과 roster total 계산기를 구현한다. Consumable/condition 제외,
   equipment/enhancement/fragment/attack interval/range/block 포함과 overflow determinism을 테스트한다.
4. Run difficulty state에 power model version, last certified snapshot, non-rewind run peak와 current Floor
   anchor를 추가하고 initial starter commit/Combat entry에서 갱신한다.
5. P03 assignment profile에 required difficulty profile reference를 추가하고 Floor generation이 template,
   assignment, ordinary quota/role, target power, encounter seed/reward contract를 한 번에 확정하게 한다.
6. Old `floor_combat_scaling`을 제거하고 battle preview/runtime wave/stat scale과 rewards가 node에 저장된
   difficulty contract만 사용하게 한다.
7. Checkpoint/active-run import/publish/rollback/continue가 model/peak/anchor/node difficulty를 exact 보존하고
   checkpoint rewind가 peak를 낮추지 못하게 한다.
8. Root/map/node preview에 band/relative threat/readiness projection을 추가하고 game_server/Unity
   reader/presenter/mock/render를 수직 갱신한다.
9. P03 determinism/save/assignment review를 함께 재실행하고 canonical/legacy/broad completion review로 닫는다.

## Test Requirements

- Power model unknown/missing/zero divisor/overflow/version and schedule gap/overlap/open-ended rejection
- Same persistent loadout/version determinism; level/equipment/enhancement/fragment/interval/range/block sensitivity
- HP/Trauma/Breakdown/consumable/temporary state exclusion from raw power and inclusion in readiness only
- Initial certified snapshot and Combat-entry peak update; weaker loadout/death/checkpoint/restart non-decrease
- Floor 1/min/rise/peak anchor formula, Standard coverage and Endless open-ended monotonicity
- Ordinary quota deterministic largest-remainder rounding with exact node-count sum; role exclusion
- Node difficulty stable across reconnect/map request/same-Floor loadout/checkpoint; next Floor uses current peak
- Preview/runtime enemy stat and generated-wave budget use the same stored node contract; no floor fallback
- Reward multiplier/rarity contract is fixed and Hard/special is strictly stronger than ordinary baseline
- BossOmen capstone inherits same Floor anchor and typed BossOmen role without consuming ordinary quota
- Active-run current/backup/load rollback and P03 facility state round-trip with non-rewind peak merge
- Unity exact band/threat/readiness display and absence of raw power/local recomputation

## Completion Conditions

- Master P04 exit gate is fully evidenced.
- One live versioned model owns certified power, anchor, band/role target, encounter budget and reward modifier.
- `floor_combat_scaling`, clamp/default fallback and competing difficulty source are absent.
- P03 template/assignment/difficulty generation is one deterministic transaction and saved decisions never reroll.
- Core/server/Unity/persistence/canonical contracts agree and completion review is `Complete / High`.

## Stop Conditions

- Approved policy cannot determine a required player-visible meaning or save compatibility becomes required.
- Persistent combat inputs cannot be separated from condition/consumable without changing battle semantics.
- Existing user changes in power/scaling/persistence/Unity files cannot be preserved safely.
- P03 source of truth must be materially replaced rather than extended through its typed seam.
- Unity runner or required host persistence verification becomes unavailable.
