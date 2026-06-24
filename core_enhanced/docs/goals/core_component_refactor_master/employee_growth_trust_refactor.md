# 직원/성장/신뢰도 시스템 리팩토링 감사

- 컴포넌트: 직원/성장/신뢰도 시스템
- 기준 문서: `docs/refactor_preparation_plan.md`
- 상태: Initial audit documented

## 읽은 범위

- Runtime code:
  - `src/game/employee.rs`
  - `src/game/growth.rs`
  - `src/game/employee_trust.rs`
  - `src/game/world/combat.rs`
  - `src/game/world/support.rs`
  - `src/game/world/state.rs`
  - `src/game/world/snapshot.rs`
  - `src/game/combat_setup/player_spawns.rs`
  - `src/game/battle/stat_pipeline.rs`
- Data code/live data:
  - `src/game/data/employee_data.rs`
  - `../game_resources/data/employees/starter_candidates.ron`
  - `../game_resources/data/employees/recruitment_candidates.ron`
- Tests/contracts:
  - `src/game/world/tests/combat.rs`
  - `src/game/world/tests/support.rs`
  - `src/game/world/tests/equipment.rs`
  - `src/game/world/tests/snapshots_and_start.rs`
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`

## 현재 구조 요약

`Employee`는 런 지속 상태의 source of truth다. 한 객체 안에 level/experience, life_state/availability, trauma, run health, loadout, skill fragments, active consumable modifier, base combat profile, trust state를 들고 있다(`src/game/employee.rs:135`). 전투 진입 profile은 `Employee::combat_profile_for_battle()`에서 `stat_pipeline::employee_combat_profile_for_battle()`로 위임한다(`src/game/employee.rs:213`, `src/game/battle/stat_pipeline.rs:40`).

전투 시작 HP는 run health 비율과 trauma를 조합해 battle max HP에 맞춰 계산한다(`src/game/employee.rs:225`). 이후 battle draft에는 `employee.combat_profile_for_battle()` 결과와 `employee.combat_profile.growth_stacks`가 함께 들어간다(`src/game/combat_setup/player_spawns.rs:174`). 최종 battle stats는 `stat_pipeline::effective_stats_for_draft()`가 growth stack, equipment, enhancement, artifact를 적용해 계산한다(`src/game/battle/stat_pipeline.rs:65`).

경험치는 전투 후 survival XP와 reward XP로 증가한다(`src/game/world/combat.rs:250`, `src/game/world/combat.rs:366`). `Employee::add_experience()`는 100 XP마다 level을 올리고, `PveWinStack`을 1 추가하며, level 범위에 따라 grade를 갱신한다(`src/game/employee.rs:304`).

신뢰도는 `EmployeeTrustState`와 `EmployeeTrustResolver`가 담당한다. `WorldState` 기본 정책은 `EmployeeTrustPolicy::narrative_only()`이며 memory/dialogue만 켜고 high-risk, trauma, pre-battle, ally-death, combat modifier 기능은 끈다(`src/game/world/state.rs:723`, `src/game/employee_trust.rs:348`). 지원 노드 치료/휴식은 `apply_event()`로 memory/trust reaction을 기록한다(`src/game/world/support.rs:661`, `src/game/world/support.rs:679`). 전투 후 incapacitation은 `modify_trauma()`를 호출하지만 현재 narrative-only 정책에서는 trauma modifier가 꺼져 있어 실제 수치 변경이나 reaction이 없다(`src/game/world/combat.rs:228`, `src/game/employee_trust.rs:446`).

Unity-facing roster snapshot은 `roster.employees[*]`가 source of truth이며 `level`, `experience`, `life_state`, `availability`, `trauma`, `health`, `trust`, `combat_profile`, `skill_fragments`, `active_consumable_modifier`, `equipped_items`를 내려준다(`src/game/world/snapshot.rs:496`). Unity 계약도 같은 필드를 기준으로 삼는다(`/mnt/f/unity projects/ark/docs/unity_core_contract.md:2216`).

## Source-of-truth 판단

- 런 지속 직원 상태 source of truth는 `Employee`다.
- 전투용 effective profile/stat source of truth는 `stat_pipeline`과 `combat_setup/player_spawns.rs`를 통한 battle draft다. Unity는 `combat_profile.effective_*` snapshot 값을 우선 사용해야 한다.
- run health와 battle health는 서로 다른 수명주기다. `EmployeeHealthState`는 run 지속 체력이고, battle HP는 전투 진입 시 `battle_start_hp_for_max()`와 stat pipeline으로 파생된다.
- 신뢰도 표시 source of truth는 `EmployeeTrustState` snapshot이다. 다만 현재 dialogue cue는 생성될 수 있지만 snapshot에는 cue payload가 아니라 count만 내려간다.

## 리팩토링 후보

### 1. `level`과 `combat_profile.grade`의 source-of-truth split

`Employee`는 `level`과 `combat_profile.grade`를 둘 다 저장한다(`src/game/employee.rs:139`, `src/game/employee.rs:60`). `add_experience()`는 level-up 때 grade를 level 범위에서 다시 계산한다(`src/game/employee.rs:304`). 하지만 starter candidate에서 grade를 받아 생성할 때는 grade만 candidate 값으로 세팅하고 level은 항상 1이다(`src/game/employee.rs:163`, `src/game/employee.rs:180`).

그 결과 data가 Regular/Senior starter를 제공하면 snapshot의 `level`과 `combat_profile.grade`, 그리고 `battle_tier()`가 서로 다른 의미를 가질 수 있다. 현재 전투 tier는 level이 아니라 grade를 본다(`src/game/employee.rs:205`).

판단: 높은 우선순위 source-of-truth 후보. grade가 level의 파생값인지, candidate/data가 직접 정하는 rarity/tier인지 결정해야 한다. 사용자와 정책 논의 필요.

가능한 방향:

- grade가 level 파생값이면 `combat_profile.grade` 저장을 줄이고 `Employee::grade()` 같은 파생 helper로 통일한다.
- candidate grade가 독립적인 직원 등급이면 `add_experience()`에서 grade를 level로 덮어쓰지 않고, level-up 보상/승급 정책을 별도 명시한다.

필요 검증:

- Regular/Senior starter candidate가 있을 때 snapshot `level`, `combat_profile.grade`, `battle_tier()`가 의도대로 일치하거나 분리되는지 테스트.

### 2. `PveWinStack`/`QuestRewardStack`은 저장되지만 현재 전투 효과가 없다

`GrowthId`는 `KillStack`, `PveWinStack`, `QuestRewardStack` 세 가지다(`src/game/growth.rs:5`). `add_experience()`는 level-up마다 `PveWinStack`을 추가한다(`src/game/employee.rs:309`). 하지만 final stat pipeline은 `KillStack`만 attack으로 반영하고 `PveWinStack`/`QuestRewardStack`은 no-op이다(`src/game/battle/stat_pipeline.rs:75`).

따라서 현재 `PveWinStack`은 축적되지만 사용자-visible battle stat 효과가 없다. 이것이 의도된 future hook이면 goal 기준상 현재 runtime source에 불필요한 상태가 남아 있는 셈이고, 의도된 성장 보상이라면 효과가 빠져 있다.

판단: 성장 시스템 source-of-truth 후보. 사용자와 정책 논의 필요.

가능한 방향:

- `PveWinStack`/`QuestRewardStack` 효과를 정책으로 확정하고 stat pipeline/test에 반영한다.
- 아직 사용하지 않는 growth id라면 제거하거나, 저장하지 않는 event/history로 낮춘다.

필요 검증:

- level-up 후 battle draft/final stats에서 성장 보상이 실제 의도대로 반영되는지 테스트.
- no-op으로 유지한다면 snapshot/UI에서 growth stack을 성장 보상처럼 표시하지 않는지 확인.

### 3. 신뢰도 dialogue cue가 생성되지만 snapshot에는 count만 내려간다

`EmployeeTrustPolicy::narrative_only()`는 dialogue 기능을 켠다(`src/game/employee_trust.rs:348`). `EmployeeTrustResolver::apply_event()`는 dialogue cue를 `TrustReaction.cues`에 추가할 수 있다(`src/game/employee_trust.rs:400`). 하지만 `EmployeeTrustState::apply_reaction()`은 `TrustReactionSummary`만 저장하며, summary는 `cue_count`, `combat_modifier_count`, `trauma_modifier_count`만 가진다(`src/game/employee_trust.rs:123`, `src/game/employee_trust.rs:131`). Roster snapshot도 recent reaction count만 내려준다(`src/game/world/snapshot.rs:526`).

Unity 계약은 `trust`를 신뢰도 표시/대사 분기용 데이터라고 설명한다(`/mnt/f/unity projects/ark/docs/unity_core_contract.md:2224`). 현재 구조에서는 core가 생성한 cue key가 버려져 Unity가 실제 대사 key를 받을 수 없다.

판단: Unity-facing DTO/UX 후보. dialogue cue를 core가 내려야 하는지, Unity가 trust state를 보고 자체 분기해야 하는지 결정이 필요하다. 사용자와 정책 논의 필요.

가능한 방향:

- `recent_reactions`에 cue keys를 포함해 core-generated narrative cue를 source of truth로 만든다.
- core는 score/band/memory만 내려주고 dialogue cue 생성 기능을 제거한다.

### 4. 전투 incapacitation은 신뢰도 memory event를 남기지 않는다

전투 후 incapacitation path는 `EmployeeTrustResolver::modify_trauma()`를 호출한다(`src/game/world/combat.rs:228`). 현재 default policy는 narrative-only라 `trauma_modifier_enabled`가 false이고, `modify_trauma()`는 empty reaction을 반환한다(`src/game/employee_trust.rs:446`). 반면 지원 노드 치료/휴식은 `apply_event()`를 통해 memory와 trust delta를 남긴다(`src/game/world/support.rs:661`, `src/game/world/support.rs:679`).

`TrustEventKind::IncurredTrauma`와 여러 전투/위험 이벤트 enum은 존재하지만(`src/game/employee_trust.rs:147`), 현재 narrative-only 전투 후 경로에서는 memory로 기록되지 않는다.

판단: 신뢰도 event source-of-truth 후보. 전투에서 쓰러진 경험이 신뢰도 memory가 되어야 하는지, 치료/휴식 같은 safe-zone 이벤트만 memory가 되는지 정책이 필요하다. 사용자와 정책 논의 필요.

### 5. 신뢰도 feature flags 중 사용되지 않는 기능 표면

`TrustFeatureFlags`에는 `high_risk_acceptance_enabled`, `trauma_modifier_enabled`, `pre_battle_modifier_enabled`, `ally_death_reaction_enabled`, `combat_modifier_enabled`가 있다(`src/game/employee_trust.rs:293`). 현재 resolver에서 high-risk와 trauma modifier 일부는 구현되어 있으나, default runtime policy에서는 꺼져 있고 pre-battle/ally-death/combat modifier는 현재 world flow에 연결되지 않았다.

판단: dormant feature surface 후보. `docs/refactor_preparation_plan.md` 기준상 미래 기능을 위해 활성 runtime 타입에 오래 남겨둔 표면은 제거 또는 명확한 policy gate가 필요하다. 사용자와 정책 논의 필요.

### 6. `EmployeeHealthState.max_hp`와 battle max HP의 경계 명시

`EmployeeHealthState::new()`는 employee base battle profile max HP로 run max HP를 초기화한다(`src/game/employee.rs:176`). Battle max HP는 skill fragment/equipment/artifact 등으로 달라질 수 있고, stat pipeline은 run HP/trauma로 계산한 current HP를 final max HP 변경 후 ratio 보존으로 다시 스케일한다(`src/game/battle/stat_pipeline.rs:50`, `src/game/battle/stat_pipeline.rs:64`).

현재 구조는 "run health는 장비/전투 효과와 별개인 장기 컨디션"이라는 해석이면 타당하다. 다만 permanent growth가 max HP를 바꾸거나 grade/level이 base profile max HP를 바꾸는 정책이 들어오면 `EmployeeHealthState.max_hp` 갱신 시점이 별도 source-of-truth 문제가 된다.

판단: 지금은 리팩토링 대상보다 정책 경계 문서화 후보. future growth/grade가 run max HP를 바꾸는 순간 별도 goal로 승격해야 한다. 사용자와 정책 논의 필요.

### 7. Hard-coded level curve와 growth reward

`experience_required_for_next_level()`은 항상 100을 반환하고, grade 승급 구간도 코드에 박혀 있다(`src/game/employee.rs:304`, `src/game/employee.rs:390`). `RUN_SYSTEM_POLICY`의 survival XP/reward XP와 함께 성장 밸런스의 source가 코드에 있다.

판단: 데이터화 후보이지만 밸런스 정책 변경이므로 지금 구현하지 않는다. 사용자와 정책 논의 필요.

## 기존 기능 조합으로 단순화 가능한 후보

- grade가 level 파생값이라면 저장 필드 두 개를 유지하지 않고 level에서 grade/tier를 파생할 수 있다.
- 신뢰도 dialogue cue를 Unity가 직접 계산하지 않게 하려면, 이미 있는 `TrustReaction.cues`를 snapshot에 보존하는 방식으로 기존 resolver 기능을 살릴 수 있다.
- 반대로 core가 dialogue cue source가 아니라면 `dialogue_enabled`/`TrustCue` 생성 표면을 제거하고 `trust.score`, `band`, `memories`만 Unity 분기 입력으로 남길 수 있다.

## 레거시/fallback/dual schema 제거 후보

- `PveWinStack`/`QuestRewardStack`이 효과 없는 저장 상태라면 제거하거나, 적용 정책이 확정될 때까지 runtime persistent state에서 제외하는 편이 낫다.
- `TrustFeatureFlags`의 미연결 기능은 compatibility layer는 아니지만 dormant policy surface다. 실제 flow 없이 DTO/상태만 남기는 것은 장기적으로 제거 후보에 가깝다.

## 하지 않거나 보류한 항목

- run health와 battle health를 하나로 합치지 않는다. 수명주기가 다르며 Unity 계약도 `health.current_hp/max_hp`를 런 지속 체력으로 설명한다.
- active consumable modifier와 battle buff를 통합하지 않는다. `docs/refactor_preparation_plan.md`가 두 수명주기를 분리하라고 명시한다.
- starter/recruitment candidate DB가 같은 `StarterEmployeeCandidate` struct를 쓰는 것은 지금은 제거 후보로 보지 않는다. 데이터 schema가 같고 DB source가 분리되어 있어 role이 명확하다.

## 필요한 테스트와 검증 명령

구현 리팩토링 착수 시 필요한 focused test:

- `cargo test -p game_core employee::`
- `cargo test -p game_core employee_trust::`
- `cargo test -p game_core battle::stat_pipeline::`
- `cargo test -p game_core world::tests::combat`
- `cargo test -p game_core world::tests::snapshots_and_start`

이번 감사 단계에서는 코드 변경이 없으므로 Rust test는 실행하지 않았다.

## 사용자와 정책 논의 필요

- grade가 level의 파생값인지, candidate/data가 직접 정하는 독립 등급인지: 사용자와 정책 논의 필요.
- `PveWinStack`/`QuestRewardStack`을 실제 성장 효과로 만들지, 제거할지: 사용자와 정책 논의 필요.
- 신뢰도 dialogue cue를 core snapshot으로 내려야 하는지, Unity가 score/band/memory를 보고 직접 분기해야 하는지: 사용자와 정책 논의 필요.
- 전투 incapacitation/trauma를 신뢰도 memory로 기록해야 하는지: 사용자와 정책 논의 필요.
- dormant trust feature flags를 유지할지 제거할지: 사용자와 정책 논의 필요.
- future growth/grade가 run max HP를 바꾸는지, battle-only max HP만 바꾸는지: 사용자와 정책 논의 필요.
- level curve, grade 승급 구간, survival/reward XP를 데이터화할지: 사용자와 정책 논의 필요.
