# Experiments — Game Rulebook Full Runtime Alignment Audit

## 상태

- Goal mode: `audit_only`
- Final audit status: `complete`
- 실제로 수행한 read-only 조사: 19건
- Production/canonical 변경: 없음
- Test 실행: 10 jobs (9 completed, 1 compile ICE) / 1,213 tests

Goal 경로와 적용 지침 존재 여부 확인은 runtime 가설을 검증한 실험이 아니므로 experiment entry로 기록하지
않는다.

## 기록 규칙

실제로 수행한 read-only 조사, probe 또는 test만 발생 직후 시간순으로 추가한다. 각 기록에는 다음을 포함한다.

- 연결된 coverage row 또는 finding ID
- 검증하려던 가설
- 수행한 실험 또는 검증
- 변경한 내용
- 실행한 정확한 명령 또는 test runner job
- 실제 실행된 test 수
- pass/fail/skip 수
- 실제 결과
- 실패 원인
- 되돌리거나 수정한 내용
- 결과에 따른 판단

Runner가 수치를 보고하지 않으면 추측하지 않고 `runner가 해당 수치를 보고하지 않음`으로 기록한다. 계획했지만
실행하지 않은 명령은 결과로 기록하지 않는다.

## E-001 — Repository 및 canonical source basis 고정

- 연결된 coverage row 또는 finding ID: 전체 감사의 공통 basis `GRFRA-ASB-001`
- 검증하려던 가설: 새 Goal 경로가 독립적이며 current repository revision, dirty impact surface와 핵심 canonical
  content identity를 재식별 가능하게 고정할 수 있다.
- 수행한 실험 또는 검증:
  - 적용되는 `AGENTS.md`와 새 Goal 경로 충돌 여부를 확인했다.
  - repository root/revision과 Core·live data·server scoped status를 확인했다.
  - 룰북과 세 인접 canonical의 SHA-256을 계산했다.
- 변경한 내용: E-001 전에 세 Goal 문서를 신규 생성; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
pwd
rg --files | rg '(^|/)AGENTS\.md$'
find core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit -maxdepth 1 -type f -printf '%f\n'
git rev-parse --show-toplevel
git rev-parse HEAD
git status --short -- core_enhanced game_resources game_server
sha256sum core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md core_enhanced/docs/skill_target_contract.md
```

- 실제 실행된 test 수: N/A — test runner를 실행하지 않음
- pass/fail/skip 수: N/A
- 실제 결과:
  - 적용되는 `AGENTS.md`는 없고 Goal 경로는 생성 전 존재하지 않았다.
  - Repository root는 `/mnt/f/work/simulator`, revision은
    `f318414fb0156a67e04c5479000c2a2683fc0c09`다.
  - 감사 impact surface에 광범위한 기존 modified/deleted/untracked 변경이 있다.
  - SHA-256은 rulebook `8aa954cc…`, core runtime `73bc18d4…`, data loading `58c636ce…`, skill target
    `a35a2024…`다.
- 실패 원인: 없음. 첫 복합 경로 확인 명령은 `rg`의 no-match exit 때문에 뒤 명령이 실행되지 않아 각 확인을
  별도 read-only 명령으로 다시 수행했다; runtime verdict에는 사용하지 않는다.
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: clean HEAD가 아니라 `GRFRA-ASB-001` current dirty working tree를 감사 actual로 사용한다.
  기존 변경을 이번 감사 성과로 귀속하거나 정리하지 않는다.

## E-002 — 룰북 전체 heading 및 coverage group inventory

- 연결된 coverage row 또는 finding ID: `GR-001`~`GR-050`
- 검증하려던 가설: 룰북의 모든 직접 소유 subsection을 누락 없이 안정적인 coverage group으로 식별할 수 있다.
- 수행한 실험 또는 검증: current `game_rulebook.md`의 `##`~`####` heading과 line locator를 추출하고 `###` 및
  `####` 개수를 확인했다. 문서 책임, gameplay subsection, 후속 authoring과 legacy 경계를 rule group으로 분류했다.
- 변경한 내용: Goal 문서에 coverage inventory 추가; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
rg -n '^#{2,4} ' core_enhanced/docs/game_rulebook.md
rg -c '^### ' core_enhanced/docs/game_rulebook.md
rg -c '^#### ' core_enhanced/docs/game_rulebook.md
```

- 실제 실행된 test 수: N/A — test runner를 실행하지 않음
- pass/fail/skip 수: N/A
- 실제 결과: `###` subsection 44개와 `####` subsection 4개를 확인했다. 상위 `##` 중 직접 disposition이 필요한
  문서 책임과 후속 authoring을 더해 50개 coverage group을 만들었다.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: 모든 group을 우선 `pending`으로 등록한다. 한 section 안에서 current normative claim과
  future/explicitly-unimplemented claim이 갈리면 조사 즉시 하위 evidence row로 분리한다.

## E-003 — 기존 종합 감사의 history-only disposition

- 연결된 coverage row 또는 finding ID: 전체 coverage의 locator 후보
- 검증하려던 가설: 기존 `game_rulebook_runtime_alignment_audit`가 current verdict가 아니라 조사 경로를 찾는
  제한된 history로 사용할 수 있는지 판정한다.
- 수행한 실험 또는 검증: 기존 claim matrix의 basis, evidence index, coverage와 verdict를 읽고 current
  `GRFRA-ASB-001`과 비교했다.
- 변경한 내용: Goal 문서에 history disposition 기록; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
wc -l core_enhanced/docs/goals/game_rulebook_runtime_alignment_audit/RULEBOOK_CLAIM_MATRIX.md core_enhanced/docs/goals/game_rulebook_runtime_alignment_audit/MISMATCH_CATALOG.md
sed -n '1,260p' core_enhanced/docs/goals/game_rulebook_runtime_alignment_audit/RULEBOOK_CLAIM_MATRIX.md
```

- 실제 실행된 test 수: N/A — test runner를 실행하지 않음
- pass/fail/skip 수: N/A
- 실제 결과: 기존 matrix는 rulebook SHA-256 `c11a0940…`, 1,210줄과 당시 15개 policy implementation gap을
  전제로 한다. Current rulebook은 SHA-256 `8aa954cc…`, 1,275줄이며 정책 구현 완료를 명시한다.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: 과거 verdict와 line locator는 current evidence로 재사용하지 않는다. Evidence index의
  module/test 후보만 current source에서 다시 확인할 조사 lead로 제한한다.

## E-004 — 정책 구현 Goal history locator 확인

- 연결된 coverage row 또는 finding ID: 주로 `GR-003`, `GR-030`~`GR-037`, `GR-040`, `GR-049`
- 검증하려던 가설: 과거 15개 gap을 구현한 후속 Goal history에서 current source 조사에 필요한 module/test
  locator를 얻을 수 있다.
- 수행한 실험 또는 검증: Master 및 P01~P04 completion audit의 heading, verdict, finding/gap locator를
  검색했다.
- 변경한 내용: 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
rg -n '^(#|##|###)|Verdict|verdict|Complete|Not Complete|finding|Finding|mismatch|Mismatch|gap|Gap' \
  core_enhanced/docs/goals/game_rulebook_policy_implementation_master/COMPLETION_AUDIT.md \
  core_enhanced/docs/goals/game_rulebook_policy_implementation_master_phase_01_monolith_tuning/COMPLETION_AUDIT.md \
  core_enhanced/docs/goals/game_rulebook_policy_implementation_master_phase_02_encounter_progression_v4/COMPLETION_AUDIT.md \
  core_enhanced/docs/goals/game_rulebook_policy_implementation_master_phase_03_fixed_standard_and_profile/COMPLETION_AUDIT.md \
  core_enhanced/docs/goals/game_rulebook_policy_implementation_master_phase_04_mastered_elite_sweep/COMPLETION_AUDIT.md
```

- 실제 실행된 test 수: N/A — test runner를 실행하지 않음
- pass/fail/skip 수: N/A
- 실제 결과: 과거 Master/P01~P04 문서는 `Complete / High`를 기록하며 Standard/profile, encounter V4,
  monolith와 mastered sweep의 source/test locator를 제공한다. Unity는 당시에도 비범위다.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: 과거 완료 verdict는 current evidence가 아니다. 해당 locator를 사용해 current source와
  test를 독립적으로 재검증한다.

## E-005 — 게임 모드·탐사·초기 경제 current source trace

- 연결된 coverage row 또는 finding ID: `GR-002`~`GR-013`
- 검증하려던 가설: 룰북의 게임 정체성, 모드, 시작 roster, facility/navigation/difficulty, Event, inventory,
  consumable, Shop과 briefing 규칙이 current Core/live RON/server 경계에서 실제 owner까지 연결된다.
- 수행한 실험 또는 검증:
  - `StartNewGame`부터 starter commit, `RunProgression`, fixed Standard map과 RunComplete/profile flag를 추적했다.
  - employee definition catalog, recruitment queue/session/roster ownership과 live definitions를 확인했다.
  - facility schedule/fixed assignment, navigation, concealed manifestation, certified power/anchor와 Gate recovery를
    추적했다.
  - Event committed check, inventory reserve/stack, consumable context/duration, Shop transaction과 combat preview
    briefing producer를 확인했다.
- 변경한 내용: Goal 문서의 evidence/verdict 갱신; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
rg -n "enum GameMode|GameMode::|StartNewGame|RunProgression|standard_tutorial|starter_eligible|employee_definition_id|recruitment_queue|active_recruitment|basic_attack_speed|initial_loadout|Certified|certified_power|floor_anchor|run_peak|EventAptitude|natural_(one|twenty)|raw_roll|reserved_slot_count|active_modifier|UseConsumable|ShopSession|EnemyBriefing" core_enhanced/src/game game_resources/data game_server/src
sed -n '640,820p' core_enhanced/src/game/world.rs
sed -n '1,180p' core_enhanced/src/game/map/progression.rs
sed -n '1,240p' core_enhanced/src/game/recruitment.rs
sed -n '1,150p' core_enhanced/src/game/data/employee_data.rs
sed -n '1,220p' game_resources/data/employees/definitions.ron
sed -n '300,430p' core_enhanced/src/game/map/facility_generator.rs
sed -n '930,1040p' core_enhanced/src/game/map/facility_generator.rs
sed -n '1040,1215p' core_enhanced/src/game/map/facility_generator.rs
sed -n '330,410p' core_enhanced/src/game/map/facility.rs
sed -n '1180,1225p' core_enhanced/src/game/world/node_flow.rs
rg -n "game_mode: Standard|fixed_|standard_floor|NormalBoss|FinalBoss|combat_normal|combat_elite|boss_normal" game_resources/data/map game_resources/data/run/policy.ron game_resources/data/pve/encounters.ron
sed -n '1,60p' game_resources/data/map/node_assignment_profiles.ron
sed -n '1,225p' game_resources/data/run/policy.ron
rg -n 'id: "suppress_nothing_there"' game_resources/data/pve/encounters.ron
sed -n '1895,1965p' game_resources/data/pve/encounters.ron
rg -n 'scripted.*fragment|fragment.*scripted|tutorial.*fragment|Standard.*fragment' core_enhanced/src game_resources/data
rg -n "fn handle_.*event|natural|raw_roll|critical_success|critical_failure|equipment_modifier|EventCheck|begin_event|event.*combat" core_enhanced/src/game/world/event_node.rs core_enhanced/src/game/world/visual_novel_flow.rs core_enhanced/src/game/resources/selection.rs
sed -n '1,360p' core_enhanced/src/game/world/event_node.rs
sed -n '215,475p' core_enhanced/src/game/resources/inventory.rs
rg -n "struct Shop|handle_.*shop|refresh|sell|arbitrage|non_repeatable|offer" core_enhanced/src/game/world/shop.rs core_enhanced/src/game/data/shop_data.rs game_resources/data/shops game_resources/data/events/rewards/base.ron
sed -n '820,900p' core_enhanced/src/game/combat_preview/mod.rs
sed -n '1,190p' core_enhanced/src/game/world/maintenance.rs
sed -n '130,380p' core_enhanced/src/game/world/action_capability.rs
sed -n '430,735p' core_enhanced/src/game/world/shop.rs
rg -n "fn is_consumable_use_context|fn is_loadout_action_context|validate_use_consumable_item_payload|tick.*consumable|remaining_battles|duration" core_enhanced/src/game/world core_enhanced/src/game/employee.rs
sed -n '200,270p' core_enhanced/src/game/world/helpers.rs
sed -n '490,635p' core_enhanced/src/game/employee.rs
sed -n '790,850p' core_enhanced/src/game/world/combat.rs
rg -n "selectable_node_ids|ForwardOnly|ManifestedBonus|prerequisite|concealed|Gate|gate_trauma|complete_gate|transition.*floor" core_enhanced/src/game/map/progression.rs core_enhanced/src/game/world/node_flow.rs core_enhanced/src/game/map/facility_generator.rs core_enhanced/src/game/data/run_policy_data.rs game_resources/data/run/policy.ron
sed -n '180,340p' core_enhanced/src/game/map/progression.rs
rg -n "fn certified|run_peak_certified|floor_power_anchor|authored_fixed|threat" core_enhanced/src/game/combat_power.rs core_enhanced/src/game/world core_enhanced/src/game/map/facility_generator.rs core_enhanced/src/game/data/run_policy_data.rs
sed -n '1,150p' core_enhanced/src/game/combat_power.rs
rg -n "safe_point|SavePoint|Maintenance|HeadquartersContact|Safezone|active_node_content" core_enhanced/src/game/world/facility.rs core_enhanced/src/game/world/node_flow.rs core_enhanced/src/game/world/map_content.rs
rg -n "advance_encounter_progression|game_mode.*Endless|GameMode::Endless" core_enhanced/src/game/world/combat.rs core_enhanced/src/game/world/map_encounters.rs core_enhanced/src/game/encounter_progression.rs
```

- 실제 실행된 test 수: N/A — source/live-data trace만 수행
- pass/fail/skip 수: N/A
- 실제 결과:
  - `GameMode`는 start input→pending selection→run state에 명시되고 Endless는 profile flag 없이는 거부된다.
  - Standard는 live fixed profiles로 Floor 1 Normal, Floor 2 Elite, Floor 3 `suppress_nothing_there`
    `NormalBoss`를 사용하고 authored-fixed difficulty를 적용한다. Final 승리는 profile flag와 RunComplete를 만든다.
  - Live starter는 정확히 6명이고 authored loadout/identity/personality/aptitude/speed를 갖는다. Recruitment는
    seeded queue와 exact queue/session/roster ownership을 유지한다.
  - Facility navigation, concealed bonus, certified power/anchor, Safe Node 분리, Event 1d20 commit, inventory
    high-water reserve, consumable replacement/duration, Shop atomic grant/refresh/sell 및 briefing producer가 존재한다.
  - Scripted Standard fragment의 exact identity/timing은 live에 없으며 룰북도 후속 authoring으로 명시한다.
- 실패 원인: Shop search 중 존재하지 않는 `game_resources/data/shops` 경로가 `rg` 오류를 냈다. Core shop
  source/data module 결과는 반환됐고, live path는 후속 loader inventory에서 정확히 재확인한다. 출력이 큰 일부
  `rg`는 표시가 truncate됐으므로 해당 hit를 verdict 근거로 단독 사용하지 않고 targeted `sed`를 사용했다.
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: `GR-002`~`GR-013`은 current source 기준 충돌이 없으며 `aligned (static; focused validation
  pending)`이다. Unity presentation-only 부분은 범위 밖으로 유지한다.

## E-006 — Wave 및 summon 결정성 계약의 룰북 누락 확인

- 연결된 coverage row 또는 finding ID: `GR-025`, `GR-026`, `GRFRA-001`~`GRFRA-003`
- 검증하려던 가설: 같은 timestamp bucket에서 동시에 준비된 Wave의 순서, actor 지점 도달 반경, summon batch의
  동시 제거 조건이 runtime에는 결정적으로 정의돼 있지만 룰북의 현재 계약에는 빠져 있다.
- 수행한 실험 또는 검증: current Wave settle/activation 및 summon reconciliation 구현을 줄 번호와 함께 읽고,
  룰북의 `적 구성과 웨이브` 및 `Live Boss Runtime Reference` 본문과 직접 대조했다.
- 변경한 내용: 세 불일치를 `EXPERIMENT_NOTES.md`에 즉시 등록; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
nl -ba core_enhanced/src/game/battle/core/wave_runtime.rs | sed -n '130,190p'
nl -ba core_enhanced/src/game/battle/core/wave_runtime.rs | sed -n '270,315p'
nl -ba core_enhanced/src/game/battle/core/summon_runtime.rs | sed -n '470,535p'
nl -ba core_enhanced/docs/game_rulebook.md | sed -n '614,670p'
```

- 실제 실행된 test 수: N/A — source/canonical direct comparison만 수행
- pass/fail/skip 수: N/A
- 실제 결과:
  - Wave runtime은 ready trigger를 priority 내림차순, stable trigger ID 오름차순으로 정렬하고 한 개씩 settle한다.
    룰북은 stable ID와 priority를 요구하지만 동률 순서를 명시하지 않는다.
  - `ActorReachedPoint`는 projected tile과 authored point 사이의 Chebyshev 거리를 `radius_cells`와 비교한다.
    룰북은 지점 도달 trigger를 열거하지만 거리 척도를 명시하지 않는다.
  - summon reconciliation은 이미 제거되지 않은 batch에 대해 summoner removal, signal, lifetime 순으로 검사한다.
    룰북은 cleanup signal과 non-kill despawn만 설명하고 여러 제거 조건이 같은 commit에서 충족될 때의 원인/모드
    우선순위를 명시하지 않는다.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: 세 항목은 runtime defect가 아니라 gameplay canonical이 current observable deterministic
  behavior를 완전히 기술하지 못한 `canonical_ambiguity`다. `GRFRA-001`~`GRFRA-003`으로 즉시 등록한다.

## E-007 — 상태 면역 `immune` feedback producer 부재 확인

- 연결된 coverage row 또는 finding ID: `GR-028`, `GRFRA-004`
- 검증하려던 가설: 룰북이 공식 `immune` feedback tag의 의미에 포함한 “상태가 완전히 막힘”이 current Core
  producer와 projection에 존재한다.
- 수행한 실험 또는 검증:
  - targeting usefulness와 damage feedback 정의·계산·HP event commit을 읽었다.
  - 전체 Core에서 `feedback_tags`와 `DamageFeedbackTag` producer/consumer를 검색했다.
  - Core와 live data에서 status immunity/resistance에 연결된 `immune` 또는 `resisted_status` producer를 검색했다.
- 변경한 내용: `GRFRA-004`를 `EXPERIMENT_NOTES.md`에 즉시 등록; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
nl -ba core_enhanced/src/game/battle/core/target_usefulness.rs | sed -n '1,300p'
rg -n "feedback_tags|FeedbackTag|Critical|Mitigated|FixedDamage|Immune|final_damage|HpChanged" core_enhanced/src/game/battle/damage.rs core_enhanced/src/game/battle/core/damage_runtime.rs core_enhanced/src/game/battle/events.rs core_enhanced/src/game/world/snapshot.rs
nl -ba core_enhanced/src/game/battle/damage.rs | sed -n '240,295p'
nl -ba core_enhanced/src/game/battle/damage.rs | sed -n '510,675p'
nl -ba core_enhanced/src/game/battle/core/damage_runtime.rs | sed -n '420,490p'
rg -n "DamageFeedbackTag::Immune|feedback_tags:.*Immune|resisted_status|Status.*Immune|immune" core_enhanced/src/game/battle core_enhanced/src/game/world/snapshot.rs
rg -n "feedback_tags|DamageFeedbackTag" core_enhanced/src | sed -n '1,240p'
rg -n "resisted_status|status.*immune|immune.*status|Immunity|immune" core_enhanced/src/game game_resources/data | sed -n '1,240p'
```

- 실제 실행된 test 수: N/A — source/data search만 수행
- pass/fail/skip 수: N/A
- 실제 결과:
  - 공식 enum은 네 tag만 가지며 `damage_feedback_tags`가 raw damage `> 0`, final damage `== 0`일 때만
    `Immune`을 생성한다.
  - 모든 `feedback_tags` projection은 `HpChanged` damage result를 통하고, status가 완전히 막힌 경우
    `Immune`을 생성하는 producer는 Core/live data 검색에서 발견되지 않았다.
  - `resisted_status`도 룰북 설명대로 공식 tag/producer가 아니다.
- 실패 원인: 첫 `rg`에 지정한 `core_enhanced/src/game/battle/events.rs`가 존재하지 않아 해당 path error가
  발생했다. 실제 event 정의 경로는 후속 전체 검색에서 `battle/event_log.rs`로 확인했으며, 결과 판정은 존재하는
  source의 targeted read와 전체 producer 검색을 사용했다.
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: 피해가 완전히 막힌 경우는 aligned지만 상태가 완전히 막힌 경우라는 current normative claim은
  producer가 없어 `runtime_implementation_gap`인 `GRFRA-004`다. Unity 표현은 비범위여도 Core event producer
  부재는 범위 안이다.

## E-008 — 전투 명령·상태·전장·타겟팅·사망 current source trace

- 연결된 coverage row 또는 finding ID: `GR-014`~`GR-024`, `GR-027`, `GR-029`
- 검증하려던 가설: `GRFRA-004`와 E-006의 결정성 공백을 제외한 룰북 전투 규칙이 current Core/live
  RON/game_server producer와 authoritative mutation/projection에 존재한다.
- 수행한 실험 또는 검증:
  - live mission policy, tactical resource/cooldown, Hard CC/Silence, withdraw/redeploy를 추적했다.
  - stance/tile effect, relocation/swap의 preflight와 atomic commit을 추적했다.
  - manual selection/focus, field work object, Pause/Resume/disconnect의 server 경계를 추적했다.
  - battlefield archetype/template/live assignment, target usefulness, damage commit 및 runtime stat→HP→death
    causality를 추적했다.
- 변경한 내용: coverage verdict 갱신; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
rg -n "TACTICAL|tactical_resource|INITIAL.*20|maximum.*99|DEPLOY|REDEPLOY|WITHDRAW|30_000|90_000|Stun|Freeze|Silence|HardCrowdControl|stance_change|DefenseFocus|Rest|TacticalRelocate|TacticalSwap|SkillCastSelection|PlayerUnit|PlayerTile|FieldWorking|black_box_emergency_repair|PauseBattle|battle_state_revision|DefenseRoute|Encirclement|DefendPoint" core_enhanced/src/game/battle core_enhanced/src/game/world core_enhanced/src/game/combat_setup core_enhanced/src/game/data game_resources/data game_server/src
rg -n "tactical_resource:|initial:|maximum:|natural_recovery|base_deploy_cost|redeploy_cost|voluntary|incapacitated" game_resources/data/run/policy.ron core_enhanced/src/game/data/run_policy_data.rs core_enhanced/src/game/world/state.rs
sed -n '90,180p' core_enhanced/src/game/battle/core/sim.rs
sed -n '430,560p' core_enhanced/src/game/world/combat.rs
sed -n '1840,2215p' core_enhanced/src/game/world/combat.rs
sed -n '1,370p' core_enhanced/src/game/battle/core/tactical_stance.rs
sed -n '1,220p' core_enhanced/src/game/battle/core/tactical_relocation.rs
sed -n '1,190p' core_enhanced/src/game/battle/core/tactical_swap.rs
rg -n "voluntary_withdraw|incapacitated|redeploy.*cooldown|redeploy_ready|30_000|90_000|withdraw.*cooldown" core_enhanced/src/game/data/run_policy_data.rs game_resources/data/run/policy.ron core_enhanced/src/game/world/state.rs core_enhanced/src/game/world/combat.rs
sed -n '260,290p' game_resources/data/run/policy.ron
sed -n '450,575p' core_enhanced/src/game/world/state.rs
rg -n "pub fn withdraw|WithdrawUnit|CasterWithdrawn|TargetWithdrawn|current_hp_policy" core_enhanced/src/game/battle/core core_enhanced/src/game/world/combat.rs
rg -n "control_capability|hard_cc|silenced|cancel.*cast|cancel.*windup|engagement" core_enhanced/src/game/battle/core/mod.rs core_enhanced/src/game/battle/core/sim.rs core_enhanced/src/game/battle/core/movement/blocking.rs
rg -n "cast_selection:|PlayerUnit|PlayerTile|AutomaticUnit|SelfUnit|fragment_freischutz_black_round|freischutz_magic_bullet" game_resources/data/skills/base.ron core_enhanced/src/game/ability.rs core_enhanced/src/game/data/skill_data.rs
sed -n '620,720p' core_enhanced/src/game/battle/core/sim.rs
sed -n '750,830p' core_enhanced/src/game/battle/core/sim.rs
sed -n '2520,2790p' core_enhanced/src/game/world/combat.rs
sed -n '1,460p' core_enhanced/src/game/battle/core/field_work.rs
sed -n '620,680p' game_resources/data/pve/encounters.ron
sed -n '400,455p' game_resources/data/skills/base.ron
sed -n '1050,1110p' game_resources/data/skills/base.ron
rg -n "selection: player_tile|selection: player_unit" game_resources/data/skills/base.ron
rg -n "handle_pause_battle|handle_resume_battle|handle_set_battle_speed|playback_delta_remainder|stored.*speed|paused" core_enhanced/src/game/world/combat.rs core_enhanced/src/game/world/state.rs
sed -n '1620,1825p' core_enhanced/src/game/world/combat.rs
rg -n "disconnect|detach|PauseBattle|paused|reconnect|TTL|ttl" game_server/src/game/player_game_actor game_server/src/game/load_balance_actor
sed -n '1,180p' game_server/src/game/player_game_actor/mod.rs
sed -n '1,190p' game_server/src/game/player_game_actor/handlers.rs
rg -n "BattleSetup|battle_setup_snapshot|battle_update|checkpoint|events_delta|result_stats" core_enhanced/src/game/world/snapshot.rs core_enhanced/src/game/behavior.rs game_server/src/game/player_game_actor/state.rs
rg --files game_resources/data core_enhanced/src/game | rg '(battlefield|mission_policy|target_usefulness|damage|run_policy_data|tactical_|field_work|world/combat|snapshot\.rs$)'
nl -ba core_enhanced/docs/game_rulebook.md | sed -n '345,613p'
nl -ba core_enhanced/docs/game_rulebook.md | sed -n '668,725p'
nl -ba core_enhanced/src/game/combat_setup/mission_policy.rs | sed -n '1,240p'
rg -n "survive_timer_ms|required_for_victory|DefendPoint|Encirclement|Recovery|DefendAndEscape|DefenseRoute" core_enhanced/src/game/data/pve_data.rs core_enhanced/src/game/battle/core/mod.rs core_enhanced/src/game/combat_setup game_resources/data/pve/encounters.ron
rg -n "fn .*?(tactical_resource|hard_cc|silence|withdraw|redeploy|stance|defense_focus|rest|relocat|swap|manual_skill|focus|field_work|pause|disconnect|battle_setup|occup|split_room)" core_enhanced/src game_server/src | sed -n '1,320p'
nl -ba game_resources/data/map/battlefield_archetypes.ron | sed -n '1,260p'
rg -n "SplitRoom|split_room|archetype:" game_resources/data/map/battlefield_templates.ron game_resources/data/pve/encounters.ron game_resources/data/map/node_assignment_profiles.ron core_enhanced/src/game/combat_preview/mod.rs
rg -n "StatChanged|HpChangeReason::Command|source_instance_id|finalize_unit_death|OnKill|apply.*stat.*modifier|max.*health|current_health" core_enhanced/src/game/battle/core/commands.rs core_enhanced/src/game/battle/core/death_runtime.rs | sed -n '1,280p'
nl -ba core_enhanced/src/game/battle/core/commands.rs | sed -n '2280,2445p'
nl -ba core_enhanced/src/game/battle/core/death_runtime.rs | sed -n '1,225p'
nl -ba core_enhanced/src/game/battle/core/commands.rs | sed -n '430,585p'
nl -ba core_enhanced/src/game/battle/core/commands.rs | sed -n '2970,3205p'
```

- 실제 실행된 test 수: N/A — current source/live-data/server trace만 수행
- pass/fail/skip 수: N/A
- 실제 결과:
  - live mission은 Defense/Boss의 fixed-defense route이고 survive timer는 같은 mission outcome으로 구현된다.
  - live run policy는 initial/max/recovery/deploy/relocate/redeploy/cooldown 수치와 clamp/atomic payment를 룰북대로
    제공한다. Hard CC/Silence, 철수/재배치, stance, relocation/swap은 각각 typed eligibility와 atomic mutation을
    갖는다.
  - live manual selection은 두 PlayerUnit path뿐이며 PlayerTile은 없다. Field work live object의 위치, 작업량,
    repair target/effect가 룰북 수치와 일치한다.
  - Pause는 battle clock을 동결하고 command commit을 허용한다. Server detach는 active battle을 auto-pause하고
    same-process actor에는 TTL을 적용하지 않으며 reconnect recovery batch를 반환한다.
  - Small/Medium/Large와 8 archetype이 live catalog에 있고 SplitRoom은 template는 있으나 disabled/weight 0이며
    encounter assignment가 없다.
  - target usefulness는 expected final damage 또는 hostile target-applied effect를 사용한다. Runtime stat modifier는
    source provenance를 유지하며 `StatChanged → optional HpChanged(Command) → death` 순서와 killer credit을 지킨다.
- 실패 원인: 일부 초기 broad `rg` 출력은 표시 한도를 넘어 truncate되어 verdict의 단독 근거로 쓰지 않고 targeted
  source read로 반복했다. E-007에 별도 기록한 존재하지 않는 `battle/events.rs` 경로 외 이 entry의 targeted
  command 실패는 없다.
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: `GR-014`~`GR-024`, `GR-027`, `GR-029`는 `aligned (static; focused validation pending)`이다.
  E-006의 `GR-025`/`GR-026` 및 E-007의 `GR-028` finding은 별도 유지한다.

## E-009 — 폐기 선언된 `skill_fragment_progress` 이름의 active DTO 재사용 확인

- 연결된 coverage row 또는 finding ID: `GR-031`, `GRFRA-005`
- 검증하려던 가설: 룰북이 폐기 대상으로 명시한 `skill_fragment_progress`가 current runtime/save/wire에서
  제거됐으며, 현재 강화·개화 진행은 다른 명시적 계약 이름을 사용한다.
- 수행한 실험 또는 검증:
  - Core/live data/server에서 폐기된 연구 필드와 current fragment progression 사용을 검색했다.
  - inventory DTO 정의와 snapshot producer를 줄 번호와 함께 읽었다.
  - current fragment progression state의 type/usage를 검색했다.
- 변경한 내용: `GRFRA-005`를 `EXPERIMENT_NOTES.md`에 즉시 등록; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
rg -n "ResearchProgress|research_progress|skill_fragment_progress|pending_research|ChooseOne|ClaimAll|GrowthStack|survival.*10|experience_per_level|battle_tier|GrantExperience|CombatParticipants|AliveRoster" core_enhanced/src game_resources/data game_server/src | sed -n '1,360p'
nl -ba core_enhanced/src/game/behavior.rs | sed -n '940,1010p'
nl -ba core_enhanced/src/game/world/snapshot.rs | sed -n '620,745p'
rg -n "struct SkillFragmentProgress|skill_fragment_progress|SkillFragmentProgress" core_enhanced/src/game/skill_fragment.rs core_enhanced/src/game/behavior.rs core_enhanced/src/game/world core_enhanced/src/game/save.rs core_enhanced/src/game/checkpoint.rs
```

- 실제 실행된 test 수: N/A — source/wire search만 수행
- pass/fail/skip 수: N/A
- 실제 결과:
  - legacy `ResearchProgress` producer는 없고 checkpoint validation에는 제거된 legacy payload rejection이 남아 있다.
  - 그러나 `InventorySnapshotDto`는 active `skill_fragment_progress` 배열을 노출한다. Producer는 보유 여부와
    독립적으로 `SkillFragmentProgress { upgrade_level, awakening_progress, awakening_available, awakened }`를
    투영한다.
  - 동일 진행은 보유 fragment 항목의 `progress`에도 들어가지만 별도 top-level 배열 이름은 룰북이 폐기했다고
    명시한 exact identifier와 같다.
- 실패 원인: 마지막 `rg`에 지정한 `core_enhanced/src/game/save.rs`와 `core_enhanced/src/game/checkpoint.rs`가
  존재하지 않아 path error가 발생했다. DTO와 실제 producer는 존재하는 targeted source에서 직접 확인했으므로
  verdict에는 영향이 없다.
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: 의미가 연구도에서 강화·개화 진행으로 바뀌었더라도 current canonical의 exact removal
  선언과 active wire 이름이 충돌한다. `runtime_implementation_gap`인 `GRFRA-005`로 등록한다.

## E-010 — Bonus objective runtime/schema 부재 확인

- 연결된 coverage row 또는 finding ID: `GR-036`, `GRFRA-006`
- 검증하려던 가설: 룰북의 current Bonus objective 계약인 `ClearWithin`과 `DecisiveDamage`가 production PVE
  schema, live RON, result evaluation 및 reward commit에 존재한다.
- 수행한 실험 또는 검증: PVE encounter schema와 legacy-field rejection test를 직접 읽고 Core/live data 전체에서
  bonus objective type·variant·producer를 대소문자 비구분 검색했다.
- 변경한 내용: `GRFRA-006`을 `EXPERIMENT_NOTES.md`에 즉시 등록; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
nl -ba core_enhanced/src/game/data/pve_data.rs | sed -n '230,335p'
nl -ba core_enhanced/src/game/data/pve_data.rs | sed -n '2325,2375p'
rg -n -i "bonus.?objective|clear.?within|decisive.?damage" core_enhanced/src game_resources/data
```

- 실제 실행된 test 수: N/A — source/schema/live-data search만 수행
- pass/fail/skip 수: N/A
- 실제 결과:
  - `PveEncounter` current schema에는 bonus objective field가 없고 `ClearWithin`/`DecisiveDamage` type도 없다.
  - `removed_suppression_research_field_is_rejected` test source는 이전
    `suppression_research: Some((bonus_objectives: []))` 입력이 `deny_unknown_fields`로 실패해야 한다고 명시한다.
  - Core/live data 전체 검색에서 실제 objective evaluator, result marker 또는 reward producer가 발견되지 않았다.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: 룰북이 current gameplay로 규정한 Bonus objective 전부가 runtime/schema에 없어
  `runtime_implementation_gap`인 `GRFRA-006`이다.

## E-011 — 성장·도감·Endless pool·모노리스·소탕·장비·파편 current source trace

- 연결된 coverage row 또는 finding ID: `GR-030`~`GR-040` (`GRFRA-005`, `GRFRA-006` 제외 항목)
- 검증하려던 가설: 룰북의 reward/growth, run-local encounter progression, permanent profile, assignment pool,
  monolith/sweep, equipment/trigger 및 fragment 규칙이 current Core/live RON/server에 존재한다.
- 수행한 실험 또는 검증:
  - reward effect/target staging, live ClaimAll/XP policy와 ordinary reward exclusion을 추적했다.
  - fragment reward ledger, Elite/NormalBoss/FinalBoss branch와 mastered sweep 재획득 branch를 추적했다.
  - compendium/profile monotonic merge와 server persistence, Endless cycle/pool/checkpoint 경계를 추적했다.
  - monolith state/merchant/residual/exchange, sweep staging/commit, equipment live catalog와 trigger occurrence,
    fragment compatibility/placeholder catalog를 추적했다.
- 변경한 내용: coverage verdict 갱신; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
nl -ba core_enhanced/docs/game_rulebook.md | sed -n '726,1010p'
rg --files core_enhanced/src/game game_server/src game_resources/data | rg '(reward|encounter_progress|compendium|profile|monolith|sweep|fragment|equipment|item|policy\.ron|boss_omen|encounters\.ron)' | sed -n '1,320p'
rg -n "GrantSkillFragment|SkillFragment\(|fragment_id:|repeat_reward|grant_node_bonus" game_resources/data core_enhanced/src/game/data core_enhanced/src/game/combat_setup/rewards.rs core_enhanced/src/game/world | sed -n '1,380p'
nl -ba game_resources/data/run/policy.ron | sed -n '1,45p'
nl -ba core_enhanced/src/game/reward.rs | sed -n '1,285p'
nl -ba core_enhanced/src/game/reward.rs | sed -n '410,480p'
rg -n "response_complete_skill_fragment_id" core_enhanced/src | sed -n '1,200p'
rg -n "GrantSkillFragment" game_resources/data/events/rewards/base.ron game_resources/data/map/node_assignment_profiles.ron game_resources/data/boss_omen/chains.ron game_resources/data/pve/encounters.ron
nl -ba core_enhanced/src/game/encounter_compendium.rs | sed -n '1,340p'
rg -n "standard_tutorial_completed|max_floor|highest|deathless|all_final|encounter_compendium|registered_ids|profile.*reset|ReferenceFinalBoss" core_enhanced/src/game game_server/src | sed -n '1,360p'
nl -ba core_enhanced/src/game/encounter_progression.rs | sed -n '1,420p'
nl -ba core_enhanced/src/game/monolith.rs | sed -n '1,420p'
rg -n "unmastered|mastered_elite|normal_boss|reserve_mastered|reserve_normal|same.*floor|excluded_source|fallback|Monolith|wandering_monolith|residual|bonus objective|ClearWithin|DecisiveDamage" core_enhanced/src/game/map core_enhanced/src/game/world core_enhanced/src/game/combat_setup game_resources/data | sed -n '1,420p'
rg -o 'weapon_archetype: [A-Za-z]+' game_resources/data/equipments/base.ron | sort | uniq -c
rg -c 'weapon_archetype:' game_resources/data/equipments/base.ron
rg -n 'id: "standard_armor"|equipment_type: Armor|equipment_type: Weapon|equipment_type: Accessory' game_resources/data/equipments/base.ron | sed -n '1,220p'
nl -ba core_enhanced/src/game/battle/core/triggers.rs | sed -n '1,420p'
rg -n "counterpart|BonusDamage|OnBattleStart|OnAttack|OnHit|OnKill|OnDeath|OnAllyDeath|instance_uuid" core_enhanced/src/game/battle/core/triggers.rs core_enhanced/src/game/battle/core/commands.rs | sed -n '1,340p'
rg -n "BasicAttackModifier|source_abnormality_id|compatibility|representative_skill|active_skill|placeholder" game_resources/data/skill_fragments/base.ron core_enhanced/src/game/data/skill_fragment_data.rs core_enhanced/src/game/skill_fragment.rs | sed -n '1,360p'
rg -c 'effect: BasicAttackModifier' game_resources/data/skill_fragments/base.ron
nl -ba game_resources/data/skill_fragments/base.ron | sed -n '1,125p'
nl -ba game_resources/data/skill_fragments/base.ron | sed -n '545,640p'
rg -n "growth_stacks\.|GrowthStack::|add_stack|increment|apply.*growth" core_enhanced/src/game | sed -n '1,260p'
nl -ba core_enhanced/src/game/growth.rs | sed -n '1,180p'
rg -n "post_battle_survival_xp|grant_survival|survival.*xp|combat_participant_employee" core_enhanced/src/game/world/combat.rs core_enhanced/src/game/reward.rs | sed -n '1,260p'
nl -ba core_enhanced/src/game/world/combat.rs | sed -n '2880,3175p'
nl -ba core_enhanced/src/game/world/mastered_elite_sweep.rs | sed -n '1,455p'
rg -n "fn is_battle_deployable|is_battle_deployable\(" core_enhanced/src/game/employee.rs core_enhanced/src/game | sed -n '1,220p'
nl -ba core_enhanced/src/game/employee.rs | sed -n '450,510p'
nl -ba core_enhanced/src/game/world/map_encounters.rs | sed -n '1,210p'
nl -ba core_enhanced/src/game/world/map_encounters.rs | sed -n '245,390p'
rg -n "loads_used|encounter_progression|completed_reward_source_ids|mastered_elite_cycle|normal_boss_cycle|monolith|boss_omen" core_enhanced/src/game/world/active_run.rs core_enhanced/src/game/world/state.rs core_enhanced/src/game/world/node_flow.rs core_enhanced/src/game/world/tests/checkpoint_validation.rs | sed -n '1,320p'
nl -ba core_enhanced/src/game/data/validation.rs | sed -n '70,220p'
nl -ba core_enhanced/src/game/data/validation.rs | sed -n '930,975p'
nl -ba game_resources/data/events/rewards/base.ron | sed -n '1,180p'
nl -ba game_resources/data/pve/encounters.ron | sed -n '2750,2845p'
nl -ba game_resources/data/pve/encounters.ron | sed -n '2845,3035p'
rg -n "CompendiumCandidate|compendium_candidates|SummonedUnit|ReferenceFinalBoss" core_enhanced/src/game/battle/core core_enhanced/src/game/world/combat.rs core_enhanced/src/game/combat_setup | sed -n '1,300p'
nl -ba core_enhanced/src/game/world/combat.rs | sed -n '720,820p'
```

- 실제 실행된 test 수: N/A — current source/live-data/server trace만 수행
- pass/fail/skip 수: N/A
- 실제 결과:
  - Live combat outcomes는 ClaimAll이고 ordinary reward catalog에는 fragment grant가 없다. XP는 대상별 전액,
    live level requirement 100, survivor 10, tier 1/3/6이며 GrowthStack production mutation은 없다.
  - Encounter progression은 first fragment/repeat reward, mastered sweep zero-count 재획득, NormalBoss/FinalBoss 및
    ReferenceFinalBoss 제외를 원자 plan/commit으로 분기한다.
  - Permanent profile은 단일 registered set, tutorial flag와 세 기록을 monotonic merge하고 server store가 active
    run/checkpoint와 분리해 보존한다. Spawn kernel은 summoned unit을 compendium candidate에서 제외한다.
  - Elite는 unmastered 우선→distinct mastered cycle→ordinary fallback, NormalBoss는 deterministic cycle을 쓴다.
    Gate는 미해결 assigned cycle 항목을 되돌리고 encounter progression은 checkpoint/run-local이다.
  - Monolith stage/use/failure/residual/exchange와 mastered sweep staged commit은 룰북과 일치한다. Bonus objective만
    E-010 finding으로 분리된다.
  - Live weapon profile은 정확히 30개이며 Staff 16/Bow 1/Gun 1/Sword 10/Spear 1/Shield 1이다. Trigger collection은
    item UUID 순서, effect-before-activation 및 non-recursive retaliation을 구현한다.
  - Fragment catalog는 production abnormality BasicAttackModifier placeholder 28개, concept starter 1개를 갖고,
    두 live ActiveSkill fragment 및 별도 nonproduction CWBS dormant reference를 명시적으로 구분한다.
- 실패 원인: 넓은 reward/profile/bonus 검색 일부는 표시 한도로 truncate되어 targeted source read로 반복했다.
  `rg -c`에서 존재하지 않는 `source_abnormality_id` spelling을 찾은 두 명령은 no-match였고 actual field는
  `origin: Abnormality(abnormality_id: ...)`로 후속 확인했다. Verdict는 no-match count를 근거로 사용하지 않았다.
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: `GR-030`, `GR-032`~`GR-035`, `GR-037`~`GR-040`은
  `aligned (static; focused validation pending)`이다. `GR-031`은 `GRFRA-005`, `GR-036`은 `GRFRA-006` 때문에
  mismatch로 유지한다.

## E-012 — Breakdown collapse counter의 runtime/data 부재 확인

- 연결된 coverage row 또는 finding ID: `GR-046`, `GRFRA-007`
- 검증하려던 가설: 룰북이 현재 condition 계약으로 규정한 Breakdown node-completion counter가 직원 상태,
  RunPolicy, snapshot 및 checkpoint runtime에 존재한다.
- 수행한 실험 또는 검증:
  - Core/live policy에서 collapse counter, remaining-node 상태와 Breakdown 관련 전이를 검색했다.
  - current `RunPolicyData` shape 및 production `policy.ron`을 읽고 룰북의 `GR-046` 본문과 대조했다.
  - Psychology의 `remaining_nodes`가 Breakdown counter가 아니라 durable memory duration임을 직접 확인했다.
- 변경한 내용: `GRFRA-007`을 `EXPERIMENT_NOTES.md`에 즉시 등록; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
rg -n "breakdown_collapse|collapse_node|Breakdown.*node|remaining.*node|trauma.*100|Psychology|trust_score|recent.*reaction|memory" core_enhanced/src/game game_resources/data/run/policy.ron game_server/src | sed -n '1,360p'
rg -n "save_point_trauma_recovery|CurrentPercent|loads_used|RUN_CHECKPOINT_MAX_LOADS|LoadRunCheckpoint|SavePoint" core_enhanced/src/game game_resources/data/run/policy.ron game_server/src | sed -n '1,360p'
rg -n "breakdown_collapse|collapse_node_limit|breakdown.*counter|nodes_remaining|remaining.*nodes|remaining.*node" core_enhanced/src/game game_resources/data/run/policy.ron
sed -n '1012,1275p' core_enhanced/docs/game_rulebook.md
sed -n '1,150p' core_enhanced/src/game/employee_psychology.rs
rg -n "PsychologyState|PsychologyMemory|trust_score|recent_reactions|record_.*reaction|add_.*memory|remaining_nodes" core_enhanced/src/game --glob '!**/tests/**'
rg -n "struct RunPolicy|save_point_trauma_recovery|collapse|breakdown" core_enhanced/src/game/data core_enhanced/src/game/world game_resources/data/run/policy.ron --glob '!**/tests/**'
sed -n '1,240p' core_enhanced/src/game/data/run_policy_data.rs
sed -n '1,220p' game_resources/data/run/policy.ron
```

- 실제 실행된 test 수: N/A — source/live-data search 및 direct comparison만 수행
- pass/fail/skip 수: N/A
- 실제 결과:
  - `RunPolicyData`에는 setup/growth/battle/maintenance 인접 정책과 trauma recovery는 있지만
    `breakdown_collapse_node_limit`가 없고 live `policy.ron`에도 해당 값이 없다.
  - 직원 condition/run/checkpoint/snapshot 경로에서 Breakdown collapse remaining-node 상태 또는 node 완료 시
    증가·폐기·사망시키는 producer를 발견하지 못했다.
  - 유일한 유사 `remaining_nodes`는 `PsychologyMemory`의 optional duration이며 `advance_node`에서 감소한다.
- 실패 원인: 최초 두 광범위 검색은 표시 한도를 넘겨 출력이 truncate됐다. 해당 출력은 verdict 근거로 쓰지 않고
  작은 scope 검색과 직접 source read로 반복했다. 문서 locator용 `rg` 한 번은 shell quote가 닫히지 않아 exit 2였고
  runtime 가설 검증에는 사용하지 않았다.
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: 룰북도 live 값이 아직 authoring되지 않았다고 명시하지만 counter 동작 자체는 current
  normative 문장이다. 따라서 future 후보로 숨기지 않고, 명시된 미구현 상태의 `runtime_implementation_gap`
  `GRFRA-007`로 기록한다.

## E-013 — SavePoint·Maintenance·HeadquartersContact current source trace

- 연결된 coverage row 또는 finding ID: `GR-012`, `GR-041`~`GR-043`
- 검증하려던 가설: 세 Safe Node의 현재 효과, checkpoint/persistence 경계, 대상별 Maintenance 가능 여부와 live
  recipe coverage, HeadquartersContact 선택지가 룰북과 일치한다.
- 수행한 실험 또는 검증:
  - SavePoint recovery→node completion→checkpoint capture, V4 validation, explicit load와 continue, server durable
    current/backup publish 및 failure rollback을 추적했다.
  - Maintenance preview/capability/command validation, equipment recipe aggregate/atomic staged inventory와 live recipe
    catalog를 추적했다.
  - HeadquartersContact의 live node/shop data와 recruitment/emergency supply/shop handler를 추적했다.
- 변경한 내용: coverage verdict 및 `GR-012` live-path caveat 갱신; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
rg --files core_enhanced/src/game game_server/src | rg '(checkpoint|active_run|save_point|maintenance|facility|player_profile_store|active_run_store)'
rg -n "RUN_CHECKPOINT_MAX_LOADS|loads_used|LoadRunCheckpoint|continue|rolling|backup|publish|active.run|V4|RunCheckpoint|save_point_trauma_recovery" core_enhanced/src/game game_server/src --glob '!**/tests/**' | sed -n '1,420p'
nl -ba core_enhanced/src/game/world/active_run.rs | sed -n '1,520p'
nl -ba core_enhanced/src/game/world/facility.rs | sed -n '1,180p'
nl -ba core_enhanced/src/game/world/facility.rs | sed -n '520,700p'
rg -n "RUN_CHECKPOINT_MAX_LOADS|capture_run_checkpoint|restore_run_checkpoint|validate_run_checkpoint|RunCheckpointPayload|StagedRecoveryEffect|export_active_run|durable" core_enhanced/src/game/world game_server/src/game/player_game_actor/handlers.rs --glob '!**/tests/**'
nl -ba core_enhanced/src/game/world/state.rs | sed -n '90,220p'
nl -ba core_enhanced/src/game/world/node_flow.rs | sed -n '1180,1455p'
nl -ba game_server/src/game/player_game_actor/handlers.rs | sed -n '270,470p'
nl -ba core_enhanced/src/game/world/state.rs | sed -n '214,345p'
nl -ba core_enhanced/src/game/world/node_flow.rs | sed -n '100,185p'
nl -ba game_server/src/persistence/active_run_store.rs | sed -n '70,305p'
rg -n "maintenance|UpgradeSkillFragment|AwakenSkillFragment|DismantleSkillFragment|DismantleEquipment|EnhanceEquipment|NoEligiblePayload|cannot_dismantle_equipment|cannot_enhance_equipment" core_enhanced/src/game/world/action_capability.rs core_enhanced/src/game/world/maintenance.rs core_enhanced/src/game/world.rs --glob '!**/tests/**'
rg -n "id: \"standard_armor\"|dismantle_recipe|enhancement_recipe|equipment_dust|recipe" game_resources/data/equipments/base.ron core_enhanced/src/game/data/equipment_data.rs
nl -ba core_enhanced/src/game/world/maintenance.rs | sed -n '180,530p'
nl -ba core_enhanced/src/game/data/equipment_data.rs | sed -n '1,460p'
nl -ba core_enhanced/src/game/world/action_capability.rs | sed -n '55,190p'
nl -ba core_enhanced/src/game/world.rs | sed -n '290,480p'
nl -ba core_enhanced/src/game/world/maintenance.rs | sed -n '325,458p'
nl -ba game_resources/data/equipments/base.ron | sed -n '1900,1955p'
rg -n "HeadquartersContact|본사 연락|RecruitEmployee|RequestEmergencySupplies|OpenHeadquartersShop|headquarters" core_enhanced/docs/game_rulebook.md core_enhanced/src/game game_resources/data --glob '!**/tests/**' | sed -n '1,360p'
nl -ba core_enhanced/src/game/world/headquarters.rs | sed -n '1,420p'
rg --files core_enhanced/src/game/world | rg 'head|recruit'
rg -c 'EquipmentDismantleRecipeMetadata\(' game_resources/data/equipments/base.ron
rg -c 'EquipmentEnhancementRecipeMetadata\(' game_resources/data/equipments/base.ron
rg -n "V2|V3|version.*2|version.*3|missing.*version|unknown.*field|extra.*field|research|pending" core_enhanced/src/game/world/tests/checkpoint_validation.rs core_enhanced/src/game/world/active_run.rs | sed -n '1,360p'
rg -n "save_point.*atomic|publish.*failure|checkpoint.*rollback|loads_used|continue" core_enhanced/src/game/world/tests game_server/src/game/player_game_actor/handlers.rs game_server/src/persistence/active_run_store.rs | sed -n '1,360p'
```

- 실제 실행된 test 수: N/A — source/live-data/server trace만 수행
- pass/fail/skip 수: N/A
- 실제 결과:
  - SavePoint는 live current-percent 15 recovery를 살아 있는 roster에 적용하고 완료 상태를 V4 strict payload로
    capture한다. Explicit load만 최대 3회 monotonic count를 증가시키고 continue는 증가시키지 않는다.
  - Checkpoint는 battle/session을 payload에 포함하지 않고 restore 시 transient state를 비운다. Server store는
    checksum current/rolling backup과 monotonic loads merge를 사용하며 publish 실패 시 Core transaction을 되돌린다.
  - Maintenance 다섯 action은 node 밖에서 생략되고 대상 하나라도 실행 가능할 때만 enabled다. Equipment recipe
    aggregate는 first-seen order와 checked add를 사용하고 staged inventory를 최종 교체한다.
  - Production equipment RON의 dismantle/enhancement recipe는 각각 정확히 1건이며 둘 다 `standard_armor`다.
  - HeadquartersContact는 live node payload와 production shop pool을 통해 채용, emergency 120 enkephalin 및 보급
    상점을 제공한다. 룰북의 starter/recovery 문장도 current recruitment를 명시하므로 subsection의 “확장” 문장은
    추가 선택지 확장 경계로 해석 가능하며 충돌하지 않는다.
  - Shop production live path는 `game_resources/data/events/shops/base.ron`으로 확인했다.
- 실패 원인: 첫 SavePoint 광범위 검색 출력 일부가 표시 한도로 truncate되어 targeted `nl`/`sed`로 반복했다.
  Maintenance 광범위 source read도 일부 truncate되어 handler/action/live RON을 작은 범위로 다시 읽었다.
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: `GR-012`, `GR-041`~`GR-043`은 `aligned (static; focused validation pending)`이다.

## E-014 — 전투 결과·후퇴·condition/failure recovery source trace

- 연결된 coverage row 또는 finding ID: `GR-044`~`GR-046`, `GRFRA-006`~`GRFRA-008`
- 검증하려던 가설: CombatResult 원자 commit, retreat rollback, condition/death cleanup과 모든 recovery path가
  룰북대로 동작한다.
- 수행한 실험 또는 검증:
  - outcome/result staging, defeat/victory branch, survivor XP, attempt exhaustion, retreat entry-snapshot restore와
    state transaction을 추적했다.
  - persistent condition/incapacitation/death cleanup 및 no-living/no-deployable failure decision을 추적했다.
  - NoDeployableEmployees recovery evaluator가 계산하는 item/node path를 룰북의 열거와 직접 대조했다.
- 변경한 내용: `GRFRA-008`을 `EXPERIMENT_NOTES.md`에 즉시 등록; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
rg -n "CompleteCombatResult|terminal outcome|terminal_outcome|Defeat|Victory|TimeReached|attempts_started|max_attempts|remaining_attempts|RetreatBattle|rollback|survival_xp|combat_recovery_restriction|AwaitingRunFailureDecision|NoLivingEmployees|NoDeployableEmployees|BossDefeated|ConfirmRunFailure|RunFailed" core_enhanced/src/game/world/combat.rs core_enhanced/src/game/world/node_flow.rs core_enhanced/src/game/world/state.rs core_enhanced/src/game/behavior.rs --glob '!**/tests/**' | sed -n '1,520p'
rg -n "physical_damage|trauma|is_breakdown|MentalState|PhysicalCondition|DeathPrevent|death_prevent|recover.*equipment|shared.*fragment|TraumaRecovery|Physical Treatment|PhysicalTreatment" core_enhanced/src/game/employee.rs core_enhanced/src/game/world core_enhanced/src/game/battle --glob '!**/tests/**' | sed -n '1,480p'
rg -n "fn handle_complete_combat_result|fn handle_retreat_battle|fn .*retreat|BattleRetreated|Retreat" core_enhanced/src/game/world/combat.rs
nl -ba core_enhanced/src/game/world/combat.rs | sed -n '1,215p'
nl -ba core_enhanced/src/game/world/combat.rs | sed -n '700,930p'
nl -ba core_enhanced/src/game/world/combat.rs | sed -n '960,1325p'
nl -ba core_enhanced/src/game/world/combat.rs | sed -n '2780,3330p'
nl -ba core_enhanced/src/game/world/combat.rs | sed -n '2720,3010p'
nl -ba core_enhanced/src/game/world/combat.rs | sed -n '960,1325p'
rg -n "fn with_state_transaction|with_state_transaction" core_enhanced/src/game/world.rs core_enhanced/src/game/world/*.rs | sed -n '1,220p'
```

- 실제 실행된 test 수: N/A — source direct trace만 수행
- pass/fail/skip 수: N/A
- 실제 결과:
  - Combat result는 cloned grant/condition state에서 condition, death cleanup, survivor XP와 reward를 stage하고
    `with_state_transaction` 안에서 node/session 결과와 함께 commit한다. Defeat는 rewards를 건너뛰고 attempt가 남으면
    retry로 돌아가며 exhausted 시 node를 완료한다. Bonus objective 부분은 기존 `GRFRA-006`이 그대로 적용된다.
  - Retreat는 `active_battle.entry_snapshot`을 복원해 NodeConfirm으로 돌아가며 result, reward, node completion과
    attempt 추가 소비를 만들지 않는다.
  - Incapacitation은 instance별 처리 ledger를 통해 physical damage 1과 authored trauma를 한 번씩 적용하고, 새 사망은
    equipment/shared fragment/consumable을 회수한다. Breakdown collapse만 `GRFRA-007`로 부재한다.
  - `recovery_node_ids_and_path_exists`는 inventory의 Physical Treatment 총량과 selectable SavePoint/Gate recovery만
    계산한다. Live HeadquartersContact recruitment session/candidate는 확인하거나 허용 node에 넣지 않는다.
- 실패 원인: 전투 결과 광범위 `nl` 출력 일부가 표시 한도로 truncate되어 result/retreat/failure 구간을 작은 범위로
  다시 읽었다.
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: HeadquartersContact recruitment 누락은 current player recovery 경로를 줄이는
  `runtime_implementation_gap` `GRFRA-008`이다. `GR-044`의 Bonus objective 외 결과 및 `GR-045`는 static상
  일치하며, `GR-046`은 `GRFRA-007`·`GRFRA-008` 때문에 mismatch다.

## E-015 — Psychology 보존 및 checkpoint roster scalar validation 확인

- 연결된 coverage row 또는 finding ID: `GR-041`, `GR-046`, `GR-047`, `GRFRA-009`
- 검증하려던 가설: Psychology current state는 룰북대로 보존되고, V4 strict checkpoint가 condition/trust의
  authoritative range를 검증한다.
- 수행한 실험 또는 검증:
  - Psychology state의 constructor, 전체 producer/mutation, node duration advance, snapshot projection과 roster
    checkpoint/retreat 포함 관계를 추적했다.
  - `EmployeeCondition`/`EmployeePsychology` serde shape와 checkpoint roster validator를 range constant와 대조했다.
- 변경한 내용: `GRFRA-009`를 `EXPERIMENT_NOTES.md`에 즉시 등록; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
rg -n "psychology|trust_score|memories|recent_events|advance_node" core_enhanced/src/game game_server/src game_resources/data --glob '!**/tests/**'
nl -ba core_enhanced/src/game/employee.rs | sed -n '350,470p'
rg -n "validate_checkpoint_identities|trust_score|recent_events.*8|MAX_RECENT|occurrence_count|intensity" core_enhanced/src/game/employee.rs core_enhanced/src/game/employee_psychology.rs core_enhanced/src/game/world/node_flow.rs
nl -ba core_enhanced/src/game/employee.rs | sed -n '650,735p'
nl -ba core_enhanced/src/game/world/snapshot.rs | sed -n '790,845p'
nl -ba core_enhanced/src/game/world/node_flow.rs | sed -n '700,755p'
nl -ba core_enhanced/src/game/world/node_flow.rs | sed -n '1080,1180p'
nl -ba core_enhanced/src/game/employee.rs | sed -n '190,315p'
nl -ba core_enhanced/src/game/world/node_flow.rs | sed -n '1375,1485p'
rg -n "validate_checkpoint.*condition|physical_damage.*MAX_PHYSICAL|trauma.*MAX_TRAUMA|MIN_TRUST_SCORE|MAX_TRUST_SCORE|recent_events\.len|memories\.len" core_enhanced/src/game --glob '!**/tests/**'
```

- 실제 실행된 test 수: N/A — source/checkpoint validation trace만 수행
- pass/fail/skip 수: N/A
- 실제 결과:
  - 새 직원 Psychology는 trust 0/empty memory/event로 시작하고 production mutation producer는 없다. 모든 node
    completion은 optional memory duration만 감소시키며 snapshot은 state를 그대로 투영한다.
  - Psychology는 `Employee` 안에 있어 result/retreat/checkpoint roster clone에 보존되고 death cleanup도 수정하지
    않는다. 현재 empty `PsychologyEventKind` 때문에 durable merge와 recent-8 규칙은 dormant다.
  - `EmployeeCondition::new`와 mutation methods는 범위를 clamp하지만 derived `Deserialize`는 constructor를 거치지
    않는다. `EmployeePsychology`도 raw `i16` trust를 deserialize한다.
  - V4 checkpoint validation의 roster check는 map key와 `employee.uuid` identity만 확인하고 condition 0..6/0..100,
    trust -100..100 범위를 검증하지 않는다.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: Psychology의 reachable production 보존은 룰북과 일치하지만 strict checkpoint가 권위 범위 밖
  roster scalar를 restore할 수 있어 `runtime_implementation_gap` `GRFRA-009`다. `GR-047`은 이 finding 때문에
  mismatch다.

## E-016 — BossOmen Event source defeat/retry semantics 확인

- 연결된 coverage row 또는 finding ID: `GR-049`, `GRFRA-010`
- 검증하려던 가설: provisional/active BossOmen의 Event source 전투는 첫 패배에 source failure가 되고 Event node
  안에서 재시도하지 않는다.
- 수행한 실험 또는 검증:
  - BossOmen state/source placement/failure handler와 live one-step chain을 읽었다.
  - Event-authored battle start, 공통 attempt ledger, defeat result branch 및 `handle_source_failure` call site를
    end-to-end로 추적했다.
- 변경한 내용: `GRFRA-010`을 `EXPERIMENT_NOTES.md`에 즉시 등록; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
rg --files core_enhanced/src/game game_resources/data | rg '(boss_omen|final_boss|hidden|corrupt|monolith|chains\.ron)'
rg -n "BossOmen|boss_omen|FinalBoss|final_boss|hidden boss|hidden_boss|corrupted|변질|shooter|capstone|provisional|active_chain|completed_chain" core_enhanced/src/game game_resources/data --glob '!**/tests/**' | sed -n '1,520p'
nl -ba game_resources/data/boss_omen/chains.ron | sed -n '1,280p'
nl -ba core_enhanced/src/game/boss_omen.rs | sed -n '1,690p'
nl -ba core_enhanced/src/game/data/boss_omen_data.rs | sed -n '1,320p'
nl -ba core_enhanced/src/game/data/validation.rs | sed -n '1030,1260p'
nl -ba core_enhanced/src/game/boss_omen.rs | sed -n '330,690p'
nl -ba core_enhanced/src/game/data/boss_omen_data.rs | sed -n '40,200p'
rg -n "corrupted_monolith_acquired" core_enhanced/src/game game_resources/data --glob '!**/tests/**'
rg -n "hidden|Hidden|shooter|Shooter|corrupt.*monolith|monolith.*corrupt|corrupted_monolith" core_enhanced/src/game game_resources/data --glob '!**/tests/**'
nl -ba core_enhanced/docs/game_rulebook.md | sed -n '885,920p'
nl -ba core_enhanced/docs/game_rulebook.md | sed -n '1170,1268p'
nl -ba core_enhanced/src/game/monolith.rs | sed -n '350,455p'
rg -n "corrupted_monolith_acquired|CorruptedMonolith|corrupted monolith" core_enhanced/src/game/monolith.rs core_enhanced/src/game/world core_enhanced/src/game/behavior.rs --glob '!**/tests/**'
rg -n "abnormality_attempt_state|begin_abnormality_attempt|record_abnormality_attempt|start_attempt|attempts_started|Event.*combat|event.*combat" core_enhanced/src/game/world/combat.rs core_enhanced/src/game/world/event_node.rs core_enhanced/src/game/world/node_flow.rs core_enhanced/src/game/world/state.rs --glob '!**/tests/**'
nl -ba core_enhanced/src/game/world/state.rs | sed -n '1280,1365p'
rg -n "handle_source_failure" core_enhanced/src/game --glob '!**/tests/**'
nl -ba core_enhanced/src/game/world/combat.rs | sed -n '240,510p'
nl -ba core_enhanced/src/game/world/combat.rs | sed -n '620,700p'
nl -ba core_enhanced/src/game/world/state.rs | sed -n '360,450p'
```

- 실제 실행된 test 수: N/A — source/live-data direct trace만 수행
- pass/fail/skip 수: N/A
- 실제 결과:
  - Live `white_night` chain의 first step은 Event source다. Event combat도
    `start_abnormality_attempt`를 호출하며 monolith 외 모든 node에 공통 최대 3회를 사용한다.
  - Defeat result는 attempt가 남으면 source kind를 구분하지 않고 NodeConfirm retry로 돌아간다.
  - BossOmen `handle_source_failure`의 production call site는 attempt exhaustion 이후 한 곳뿐이다. 따라서 Event omen
    전투는 첫 패배가 아니라 세 번째 패배 뒤에야 provisional defer 또는 active same-step retention을 적용한다.
- 실패 원인: 최초 BossOmen 전체 검색과 source read 일부가 표시 한도로 truncate되어 targeted 함수/구간을 다시
  읽었다.
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: Event source의 no-retry/current failure boundary가 룰북과 달라
  `runtime_implementation_gap` `GRFRA-010`이다.

## E-017 — BossOmen 나머지 흐름·future authoring·legacy 및 문서 책임 경계 trace

- 연결된 coverage row 또는 finding ID: `GR-001`, `GR-048`~`GR-050` (`GRFRA-010` 제외 항목)
- 검증하려던 가설: 일반 FinalBoss chain의 나머지 current 계약이 runtime/live data와 일치하고, hidden boss와
  authoring 후보는 명시적 future seam이며, 제거한 gameplay legacy가 production 경로에 없다.
- 수행한 실험 또는 검증:
  - BossOmen cycle/eligibility/dependency/placement/defer/active/capstone/FinalBoss resolution과 permanent aggregate,
    live chain/template를 추적했다.
  - corrupted monolith key의 consumer reachability와 룰북의 명시적 implementation-follow-up 경계를 대조했다.
  - legacy exact names를 production Core/live RON/server에서 검색하고 동명 `Rest`의 현재 tactical 의미를 분리했다.
  - 룰북의 문서 책임 링크와 인접 canonical owner 표를 확인했다.
- 변경한 내용: coverage verdict 갱신; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
rg -n "timeline_delta|battle_delta|ChooseSupport|Support|support choice|support_choice|ChooseRest|Rest" core_enhanced/src game_resources/data game_server/src --glob '!**/tests/**' --glob '!**/docs/**' | sed -n '1,360p'
rg -n "enum (GameMode|BattleMode)|removed.*mode|legacy.*mode|wrapper|Support|Rest|ChooseSupport" core_enhanced/src/game game_resources/data --glob '!**/tests/**' | sed -n '1,320p'
rg -n "BossOmen|boss_omen|FinalBoss|forced_boss|complete_forced_boss|clear_after_forced|handle_gate_advance" core_enhanced/src/game/world/node_flow.rs core_enhanced/src/game/world/combat.rs core_enhanced/src/game/encounter_progression.rs core_enhanced/src/game/encounter_compendium.rs | sed -n '1,420p'
nl -ba core_enhanced/src/game/world/node_flow.rs | sed -n '820,965p'
nl -ba core_enhanced/src/game/encounter_progression.rs | sed -n '330,375p'
nl -ba core_enhanced/src/game/encounter_compendium.rs | sed -n '50,85p'
nl -ba core_enhanced/src/game/map/facility.rs | sed -n '620,655p'
nl -ba game_resources/data/map/facility_templates.ron | sed -n '185,210p'
rg -n 'timeline_delta|battle_delta|ChooseSupport|choose_support|support_choice|SupportChoice' core_enhanced/src game_resources/data game_server/src --glob '!**/tests/**' --glob '!**/docs/**'
rg -n '\bSupport\b|\bRest\b' core_enhanced/src/game/behavior.rs core_enhanced/src/game/map core_enhanced/src/game/resources core_enhanced/src/game/world game_resources/data/map game_resources/data/run --glob '!**/tests/**'
nl -ba game_server/src/game/player_game_actor/messages.rs | sed -n '530,590p'
rg -n 'removed.*(Support|Rest|ChooseSupport)|unknown.*(support|rest)|legacy.*(support|rest)|choose_support' core_enhanced/src game_server/src --glob '!**/docs/**'
nl -ba core_enhanced/docs/game_rulebook.md | sed -n '1,45p'
for audit_doc in core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md core_enhanced/docs/skill_target_contract.md core_enhanced/docs/engineering_workflow.md; do test -f "$audit_doc" && echo "present $audit_doc"; done
rg -n '^#|source of truth|Source of Truth|소유|책임|Unity|RON|save|wire' core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md core_enhanced/docs/skill_target_contract.md | sed -n '1,260p'
```

- 실제 실행된 test 수: N/A — source/live-data/canonical trace만 수행
- pass/fail/skip 수: N/A
- 실제 결과:
  - 일반 BossOmen state는 authored 전체 chain cycle, locked member retention, deterministic defer, one-active invariant,
    checkpoint validation과 completed ledger를 가진다. Live membership은 min Floor 3의 one-step Event `white_night`다.
  - Source category overlay는 category fallback을 하지 않고, capstone template는 Boss→concealed Exit Gate의
    ForwardOnly connection을 미리 authoring한다. 승리 시 FinalBoss reward/chain ledger와 Gate reveal이 같은 result
    transaction에 포함되고 패배는 immediate common failure candidate다.
  - Corrupted monolith는 acquire/validate/snapshot되는 dormant key뿐이고 hidden-boss consumer/schema/live data는 없다.
    룰북 `895-896`이 이를 명시적으로 “consumer 없음, 구현은 후속”으로 분류하므로 `1254-1264`는 target policy이지
    current runtime claim으로 판정하지 않았다.
  - Exact legacy `timeline_delta`, `battle_delta`, Support/ChooseSupport state/command는 production path에 없다.
    Server inline test는 `choose_support` deserialize rejection을 확인한다. Current `Rest` hit는 tactical platform effect와
    event reason으로 제거된 support choice와 다른 domain이다.
  - 룰북이 위임한 네 local canonical link가 존재하고 각 owner 표가 룰북의 책임 경계와 호응한다. 외부 Unity
    canonical은 승인된 비범위라 접근하지 않았다.
- 실패 원인: 최초 legacy 검색은 substring `Support`/`Rest` 때문에 HeadquartersSupport와 tactical Rest를 함께
  반환했다. Word/exact legacy 검색과 source read로 도메인을 분리했다.
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: `GR-001`, `GR-048`, `GR-050`은 boundary/future/absence 기준으로 aligned다. `GR-049`는
  `GRFRA-010`만 mismatch이고 나머지는 `aligned (static; focused validation pending)`이다. 50개 coverage group의
  static disposition이 모두 완료됐다.

## E-018 — Facility SavePoint/Maintenance focused suite

- 연결된 coverage row 또는 finding ID: `GR-041`, `GR-042`, `GRFRA-009`
- 검증하려던 가설: SavePoint recovery/checkpoint/load 및 Maintenance capability의 existing focused tests가 current
  정상 경로를 보호한다.
- 수행한 실험 또는 검증: facility module test filter를 실행했다.
- 변경한 내용: 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
cargo test -p game_core game::world::tests::facility -- --nocapture
```

- 실제 실행된 test 수: 12
- pass/fail/skip 수: 12 pass / 0 fail / 0 ignored; 다른 test binary는 0 tests 실행
- 실제 결과: SavePoint trauma/checkpoint, inventory restore, V4 round-trip/monotonic load, three-load limit,
  Maintenance 독립 node/action/capability 등 12개 test가 모두 통과했다.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: `GR-041`/`GR-042`의 정상 경로 static verdict를 focused validation이 지지한다. 이 filter에는
  out-of-range condition/trust payload case가 없으므로 `GRFRA-009`는 반증되지 않고 유지된다.

## E-019 — Strict V4 checkpoint validation focused suite

- 연결된 coverage row 또는 finding ID: `GR-041`, `GRFRA-005`, `GRFRA-009`
- 검증하려던 가설: existing V4 validation tests가 문서 shape, graph, seed, ownership 및 제거된 legacy shape를
  mutation 전에 거부한다.
- 수행한 실험 또는 검증: checkpoint validation module test filter를 실행했다.
- 변경한 내용: 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
cargo test -p game_core game::world::tests::checkpoint_validation -- --nocapture
```

- 실제 실행된 test 수: 7
- pass/fail/skip 수: 7 pass / 0 fail / 0 ignored; 다른 test binary는 0 tests 실행
- 실제 결과: strict V4 exact fields, old research/pending rejection, semantic round-trip, 19개 one-edge graph mutation의
  continue/explicit-load no-mutation rejection, foreign seed 및 fragment equip-limit 검증 test가 모두 통과했다.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: V4의 기존 graph/legacy strictness는 정상이지만 suite와 mutation matrix에 condition/trust range
  case가 없다. 따라서 `GRFRA-009`는 독립 validation gap으로 유지된다.

## E-020 — Recovery/BossOmen finding-boundary focused tests

- 연결된 coverage row 또는 finding ID: `GR-046`, `GR-049`, `GRFRA-008`, `GRFRA-010`
- 검증하려던 가설: existing tests가 current Headquarters no-deferral, Event battle retry 및 BossOmen state failure/cycle
  동작을 재현한다.
- 수행한 실험 또는 검증: 두 exact world test와 BossOmen unit-test module filter를 실행했다.
- 변경한 내용: 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
cargo test -p game_core weaponless_headquarters_does_not_defer_failure -- --nocapture
cargo test -p game_core event_started_combat_defeat_reenters_while_attempts_remain -- --nocapture
cargo test -p game_core game::boss_omen::tests -- --nocapture
```

- 실제 실행된 test 수: 4 (각 job 1, 1, 2)
- pass/fail/skip 수: 4 pass / 0 fail / 0 ignored; 각 job의 다른 test binary는 0 tests 실행
- 실제 결과:
  - `weaponless_headquarters_does_not_defer_failure`는 immediate Headquarters가 있는 map에서도
    NoDeployableEmployees가 RunFailed가 되는 current behavior를 확인했다.
  - `event_started_combat_defeat_reenters_while_attempts_remain`은 Event-authored combat이 Defeat 후 attempts가 남으면
    재진입하는 공통 current behavior를 확인했다.
  - BossOmen unit tests는 provisional defer/active same-step retention과 forced-boss victory cycle consumption을
    확인했다.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: test failure는 없지만 앞의 두 pass가 룰북과 다른 current behavior를 고정하므로
  `GRFRA-008`과 `GRFRA-010`의 direct behavioral evidence다. BossOmen 나머지 aligned state transition도 지지한다.

## E-021 — Production live RON integration suite

- 연결된 coverage row 또는 finding ID: `GR-002`~`GR-013`, `GR-024`~`GR-040`, `GR-048`~`GR-050`
- 검증하려던 가설: production embedded RON bundle의 loader, cross-reference와 주요 current authoring invariants가
  current code에서 유효하다.
- 수행한 실험 또는 검증: `ron_loading` integration test binary 전체를 실행했다.
- 변경한 내용: 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
cargo test -p game_core --test ron_loading -- --nocapture
```

- 실제 실행된 test 수: 25
- pass/fail/skip 수: 25 pass / 0 fail / 0 ignored / 0 filtered
- 실제 결과: live starter/employee, run policy, map/content pool, PVE/ref/reward/fragment, skill/buff/consumable,
  equipment weapon profile, preview와 live boss runtime authoring tests가 모두 통과했다.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: production live RON의 parse/reference/reachability 및 다수 aligned row를 broad하게 지지한다.
  Schema 자체가 없는 Bonus objective(`GRFRA-006`)나 unauthored Breakdown counter(`GRFRA-007`)는 이 suite의 성공으로
  구현된 것이 아니므로 findings를 유지한다.

## E-022 — Server active-run store focused suite 1차 시도 (rustc ICE)

- 연결된 coverage row 또는 finding ID: `GR-041`, server durable persistence boundary
- 검증하려던 가설: active-run current/backup, monotonic loads, checksum/version rejection과 publish failure tests가
  통과한다.
- 수행한 실험 또는 검증: game_server active-run store module filter를 실행했다.
- 변경한 내용: 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
cargo test -p game_server persistence::active_run_store::tests -- --nocapture
```

- 실제 실행된 test 수: 0 — test binary compile 전에 rustc가 중단됨
- pass/fail/skip 수: 0 pass / 0 fail / 0 skip; test runner 결과 없음
- 실제 결과: `game_server` lib test compile 중 rustc 1.91.1이 incremental `try_mark_previous_green`/trait selection에서
  internal compiler error(exit 101)를 냈다. Query stack은 unrelated load-balance actor test type checking을 가리켰다.
- 실패 원인: rustc incremental metadata ICE; active-run test assertion failure가 아님
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: server persistence validation은 아직 실행되지 않았다. Source 변경 없이 incremental compilation을
  끈 동일 filter로 재시도한다.

## E-023 — Server active-run store focused suite incremental-off 재시도

- 연결된 coverage row 또는 finding ID: `GR-041`, server durable persistence boundary
- 검증하려던 가설: E-022 실패가 source/test defect가 아닌 incremental compiler state 문제이며, 같은 active-run
  store tests는 clean non-incremental compile에서 통과한다.
- 수행한 실험 또는 검증: incremental compilation을 끄고 동일 module filter를 재실행했다.
- 변경한 내용: 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
CARGO_INCREMENTAL=0 cargo test -p game_server persistence::active_run_store::tests -- --nocapture
```

- 실제 실행된 test 수: 12
- pass/fail/skip 수: 12 pass / 0 fail / 0 ignored; server main test binary는 0 tests 실행
- 실제 결과: current/backup real-file recovery, monotonic loads/run peak merge, invalidation generation isolation,
  checksum/version/removed-anchor rejection 및 publish failure preservation tests가 모두 통과했다.
- 실패 원인: 없음. E-022의 incremental ICE는 이 설정에서 재현되지 않았다.
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: `GR-041`의 server durable store 정상 경로가 focused validation으로 지지된다. E-022는 도구chain
  risk로 남기되 runtime mismatch로 분류하지 않는다.

## E-024 — Core library broad suite

- 연결된 coverage row 또는 finding ID: `GR-001`~`GR-050`, Core runtime broad regression boundary
- 검증하려던 가설: focused 검증 밖의 current Core library 동작에도 감사 findings와 별개의 broad regression이 없다.
- 수행한 실험 또는 검증: incremental compilation을 끄고 `game_core` library test 전체를 실행했다.
- 변경한 내용: 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
CARGO_INCREMENTAL=0 cargo test -p game_core --lib
```

- 실제 실행된 test 수: 1,080
- pass/fail/skip 수: 1,080 pass / 0 fail / 0 ignored / 0 filtered
- 실제 결과: `game_core` library test 전체가 통과했다. Runner 보고 완료 시간은 53.95초다.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: current Core의 broad regression baseline은 통과했다. 이 결과는 룰북과 다른 현재 동작을
  명시적으로 재현하는 `GRFRA-008`/`GRFRA-010`이나 schema/producer/validation 부재 findings를 해소하지 않는다.

## E-025 — Server library broad suite

- 연결된 coverage row 또는 finding ID: server wire·projection·durable persistence boundary 전반
- 검증하려던 가설: focused active-run 검증 밖의 current `game_server` library 동작에도 broad regression이 없고,
  legacy wire rejection과 Core snapshot projection이 유효하다.
- 수행한 실험 또는 검증: incremental compilation을 끄고 `game_server` library test 전체를 실행했다.
- 변경한 내용: 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
CARGO_INCREMENTAL=0 cargo test -p game_server --lib
```

- 실제 실행된 test 수: 73
- pass/fail/skip 수: 73 pass / 0 fail / 0 ignored / 0 filtered
- 실제 결과: server message/wire, Core projection, live battle session, actor lifecycle, active-run 및 player-profile
  persistence를 포함한 library test 전체가 통과했다. `deserializes_snake_case_node_flow_requests`와 제거된 legacy
  transport rejection도 포함된다.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: current server boundary의 broad regression baseline은 통과했다. E-022의 incremental rustc ICE는
  non-incremental focused/broad 실행에서 재현되지 않았으며 runtime mismatch가 아니라 toolchain risk로 유지한다.

## E-026 — Coverage·finding·source-currentness·Goal 경로 최종 무결성 검사

- 연결된 coverage row 또는 finding ID: `GR-001`~`GR-050`, `GRFRA-001`~`GRFRA-010`, `GRFRA-ASB-001`
- 검증하려던 가설: 모든 coverage row와 finding이 누락 없이 한 번씩 존재하고, current-state section에 pending이
  없으며, canonical identity와 Goal-only 문서 경계가 초기 basis에서 변하지 않았다.
- 수행한 실험 또는 검증:
  - 네 canonical의 SHA-256을 초기값과 다시 비교했다.
  - coverage row 수, finding heading과 stale pending 표현을 검사했다.
  - Goal 경로 file inventory와 scoped Git status를 확인했다.
- 변경한 내용: 검사 전 Goal 문서의 current-state 요약을 실제 완료 결과로 갱신; production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
sha256sum core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md core_enhanced/docs/skill_target_contract.md
rg -c '^\| `GR-[0-9]{3}` \|' core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit/EXPERIMENT_NOTES.md
rg -n '^### GRFRA-[0-9]{3} ' core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit/EXPERIMENT_NOTES.md
rg -n 'discovery pending|claim inventory 대기|관련 claim에서만 확인 예정|\|.*`pending`' core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit/PLAN.md core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit/EXPERIMENT_NOTES.md
find core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit -maxdepth 1 -type f -printf '%f\n'
git status --short -- core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit
git status --short -- core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md core_enhanced/docs/skill_target_contract.md
```

- 실제 실행된 test 수: N/A — test runner를 실행하지 않음
- pass/fail/skip 수: N/A
- 실제 결과:
  - 네 SHA-256은 E-001의 exact 초기값과 동일하다.
  - Coverage row는 50개, finding heading은 `GRFRA-001`~`010`의 10개이며 stale pending 검색 결과는 0건이다.
    No-match를 뜻하는 해당 `rg`의 exit code 1은 예상 결과다.
  - Goal 경로에는 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`만 있다.
  - Goal 경로는 새 untracked directory이고, 네 canonical은 E-001과 같은 기존 modified 상태다.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: coverage/finding 완전성, canonical currentness와 세 Goal 문서 경계가 유지됐다. 기존 dirty
  canonical 표시는 이번 감사 변경으로 귀속하지 않는다.

## E-027 — Final status 및 deliverable shape 확인

- 연결된 coverage row 또는 finding ID: audit Goal 완료 조건 전체
- 검증하려던 가설: 최종 상태 전환 뒤 세 문서가 `complete`를 일관되게 표시하고, coverage/finding 수와 요청된
  세-file deliverable shape가 유지되며 trailing whitespace가 없다.
- 수행한 실험 또는 검증: 최종 status/summary, row/heading count, trailing whitespace, file inventory와 scoped
  status를 다시 확인했다.
- 변경한 내용: 검사 전에 PLAN/EXPERIMENTS/EXPERIMENT_NOTES의 final status와 최종 요약을 갱신;
  production/canonical 변경 없음
- 실행한 정확한 명령 또는 test runner job:

```bash
rg -n '^(- Status:|- Final audit status:|- Final Status:|- Coverage:|- 불일치:|- 검증:)' core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit/PLAN.md core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit/EXPERIMENTS.md core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit/EXPERIMENT_NOTES.md
rg -c '^\| `GR-[0-9]{3}` \|' core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit/EXPERIMENT_NOTES.md
rg -c '^### GRFRA-[0-9]{3} ' core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit/EXPERIMENT_NOTES.md
rg -n '[ \t]+$' core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit/PLAN.md core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit/EXPERIMENTS.md core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit/EXPERIMENT_NOTES.md
find core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit -maxdepth 1 -type f -printf '%f\n'
git status --short -- core_enhanced/docs/goals/game_rulebook_full_runtime_alignment_audit
```

- 실제 실행된 test 수: N/A — test runner를 실행하지 않음
- pass/fail/skip 수: N/A
- 실제 결과:
  - PLAN, EXPERIMENTS와 report summary가 모두 `complete`를 표시한다.
  - Coverage row 50개, finding heading 10개가 유지됐다.
  - Trailing whitespace 검색은 0건이며 no-match exit code 1은 예상 결과다.
  - Goal 경로에는 요청된 세 문서만 있고 scoped status는 새 untracked directory다.
- 실패 원인: 없음
- 되돌리거나 수정한 내용: 없음
- 결과에 따른 판단: 사용자-visible 완료 조건과 deliverable shape가 충족됐다.
