# Minimum Bootstrap Content Manifest

이 문서는 `GRB-PD-004/005/006/010`의 구현을 빈 참조 없이 구동하고 contract test를 작성하기 위한 read-only
**minimum bootstrap manifest**다. Production RON이나 구현 자체가 아니며 최종 콘텐츠 명세도 아니다.
2026-08-12 사용자 결정은 PD004의 `R1-continuity`, PD005의 one-step WhiteNight live bootstrap,
PD006의 modified `ST1-NT60`, PD010의 sweep reacquisition을 bootstrap fixture/policy로 선택했다. 표의 ID는
새 콘텐츠 설계가 아니라 current catalog/roster에서 재사용 가능값을 유도한 것이다.

## Authority And Replacement Rule

- 이 문서가 확정하지 않는 것: 최종 encounter roster/source pairing/repeat balance, 최종 tutorial map,
  encounter/tuning/reward/script/narrative/assets, dedicated skills와 최종 BossOmen chains.
- 이 문서가 보장하는 것: 모든 활성 bootstrap source가 strict-valid explicit row를 갖고, Standard가
  three-Floor fixed/no-adaptive 형태로 끝까지 실행되며 F3 completion/profile transaction을 검증할 수 있다.
- 별도 content Goal은 policy invariant와 strict validation을 유지하면서 아래 ID, 수치, slot과 encounter를
  교체할 수 있다. Bootstrap 교체는 그 자체로 policy/rulebook 변경이 아니다.

## GRB-PD-004 — 29-Source Bootstrap Fixture

Bootstrap roster는 current target projection인 Elite 13 + NormalBoss 13 + FinalBoss 3이다.
`reward_source_id`는 PD003의 explicit stable identity 결론을 검증하기 위한 초기 authoring 값이며 encounter
ID rename과 분리한다. Fragment identity는 각 current abnormality의 response-complete catalog entry에서
one-to-one으로 재사용했다. 이 표는 final content roster/pairing 승인이 아니다.

| Class | `reward_source_id` | Encounter | Target primary abnormality | `fragment_id` |
| --- | --- | --- | --- | --- |
| Elite | `elite_black_swan` | `suppress_black_swan` | `f-02-70_black_swan` | `fragment_black_swan_trace` |
| Elite | `elite_censored` | `suppress_censored` | `o-03-89_censored` | `fragment_censored_trace` |
| Elite | `elite_clouded_monk` | `suppress_clouded_monk` | `d-01-110_clouded_monk` | `fragment_clouded_monk_trace` |
| Elite | `elite_laetitia` | `suppress_laetitia` | `o-01-67_laetitia` | `fragment_laetitia_trace` |
| Elite | `elite_nameless_fetus` | `suppress_nameless_fetus` | `o-01-15_nameless_fetus` | `fragment_nameless_fetus_trace` |
| Elite | `elite_parasite_tree` | `suppress_parasite_tree` | `d-04-108_parasite_tree` | `fragment_parasite_tree_trace` |
| Elite | `elite_porcubbus` | `suppress_porcubbus` | `o-04-66_porcubbus` | `fragment_porcubbus_trace` |
| Elite | `elite_queen_bee` | `suppress_queen_bee` | `t-04-50_queen_bee` | `fragment_queen_bee_trace` |
| Elite | `elite_rudolta` | `suppress_rudolta` | `f-02-49_rudolta` | `fragment_rudolta_trace` |
| Elite | `elite_scarecrow` | `suppress_scarecrow` | `f-01-18_scarecrow` | `fragment_scarecrow_trace` |
| Elite | `elite_schadenfreude` | `suppress_schadenfreude` | `o-05-76_schadenfreude` | `fragment_schadenfreude_trace` |
| Elite | `elite_snow_queen` | `suppress_snow_queen` | `f-01-87_snow_queen` | `fragment_snow_queen_trace` |
| Elite | `elite_warm_hearted_woodsman` | `suppress_warm_hearted_woodsman` | `f-05-32_warm_hearted_woodsman` | `fragment_warm_hearted_woodsman_trace` |
| NormalBoss | `normal_boss_big_bad_wolf` | `suppress_big_bad_wolf` | `f-02-58_big_bad_wolf` | `fragment_big_bad_wolf_trace` |
| NormalBoss | `normal_boss_burrowing_heaven` | `suppress_burrowing_heaven` | `o-04-72_burrowing_heaven` | `fragment_burrowing_heaven_trace` |
| NormalBoss | `normal_boss_freischutz` | `suppress_freischutz` | `t-02-43_freischutz` | `fragment_freischutz_black_round` |
| NormalBoss | `normal_boss_funeral_butterfly` | `suppress_funeral_butterfly` | `t-02-99_funeral_butterfly` | `fragment_funeral_butterfly_eulogy` |
| NormalBoss | `normal_boss_king_of_greed` | `suppress_king_of_greed` | `o-01-64_king_of_greed` | `fragment_king_of_greed_trace` |
| NormalBoss | `normal_boss_knight_of_despair` | `suppress_knight_of_despair` | `o-01-73_knight_of_despair` | `fragment_knight_of_despair_trace` |
| NormalBoss | `normal_boss_little_red` | `suppress_little_red` | `f-01-57_little_red` | `fragment_little_red_trace` |
| NormalBoss | `normal_boss_melting_love` | `suppress_melting_love` | `d-03-109_melting_love` | `fragment_melting_love_trace` |
| NormalBoss | `normal_boss_mountain` | `suppress_mountain` | `t-01-75_mountain` | `fragment_mountain_smiling_bodies_trace` |
| NormalBoss | `normal_boss_nothing_there` | `suppress_nothing_there` | `o-06-20_nothing_there` | `fragment_nothing_there_trace` |
| NormalBoss | `normal_boss_queen_of_hatred` | `suppress_queen_of_hatred` | `o-01-04_queen_of_hatred` | `fragment_queen_of_hatred_trace` |
| NormalBoss | `normal_boss_yang` | `suppress_yang` | `o-05-102_yang` | `fragment_yang_trace` |
| NormalBoss | `normal_boss_yin` | `suppress_yin` | `o-01-64_yin` | `fragment_yin_trace` |
| FinalBoss | `final_boss_apocalypse_bird` | `suppress_apocalypse_bird` | `o-02-63_apocalypse_bird` | `fragment_apocalypse_bird_trace` |
| FinalBoss | `final_boss_silent_orchestra` | `suppress_silent_orchestra` | `t-01-31_silent_orchestra` | `fragment_silent_orchestra_trace` |
| FinalBoss | `final_boss_white_night` | `suppress_white_night` | `o-01-45_white_night` | `fragment_white_night_trace` |

Required validation:

- `suppress_king_of_greed.primary_abnormality_id`는 current와 위 표 모두
  `o-01-64_king_of_greed`다. Loader가 이 exact reference를 검증한다.
- Moonlit Wail은 target boss roster와 이 manifest에서 제외한다.
- Queen of Hatred와 Mountain of Smiling Bodies는 current FinalBoss가 아니라 target NormalBoss다.
- WhiteNight row는 current one-step live development bootstrap에서 reachable하다. Silent Orchestra와
  Apocalypse Bird row는 future publication을 위한 authored dormant identity이며 아직 reachable하지 않다.

### Bootstrap repeat-reward

Bootstrap fragment identity 29개는 위 표로 bounded됐고, 사용자는 같은 source를 다시 완료했을 때 적용할
초기 `repeat_reward`로 `R1-continuity`를 승인했다. 이는 최종 economy balance가 아니다. 아래 대안은
decision history로만 보존한다.

| Choice | Elite | NormalBoss | FinalBoss | Evidence/trade-off |
| --- | --- | --- | --- | --- |
| `R1-continuity` (approved) | `GrantFragmentDust(5)` | `GrantFragmentDust(10)` | `GrantFragmentDust(20)` | Current `run/policy.ron` 값과 live `GrantFragmentDust` primitive를 보존하고 Endless의 반복 성장에 명시적 보상을 둔다. 2026-08-12 사용자 승인. |
| `R2-none` | `None` | `None` | `None` | 추가 repeat economy를 만들지 않는다. Encounter의 base reward는 남지만 fragment trigger 자체의 반복 보상은 없다. |
| `defer` | — | — | — | Research 제거/직접 획득 cutover와 dependent WP004를 시작하지 않는다. |

한 class 안의 29개 source는 같은 값으로 authoring하는 후보지만 schema는 각 row에 explicit effect를
기록해야 한다. 암묵적 class fallback은 두지 않는다.

Mastered Elite 소탕에는 PD010 예외가 적용된다. Source fragment owned count가 0이면 위 `repeat_reward`
대신 같은 fragment 한 copy를 재획득하고, count가 1 이상이면 `repeat_reward`를 지급한다. 둘은 한
transaction에서 함께 지급하지 않는다. 마지막 copy를 분쇄해 dust를 얻고 다시 소탕해 재획득하는 cycle은
횟수 제한 없이 허용하며, 재획득은 first-grant/mastery ledger나 identity별 upgrade/awakening progress를
reset하지 않는다. 직접 전투와 NormalBoss/FinalBoss에는 이 예외를 적용하지 않는다.

## GRB-PD-005 — BossOmen Live Publication Reconciliation

이 decision은 generic runtime을 검증할 최소 live content와 future final-content 경계를 분리해 닫는다.

- `game_rulebook.md`는 WhiteNight 상세 체인을 향후 정책부터 다시 승인할 후보이며 현재 구현 계약이
  아니라고 명시한다.
- 완료된 placeholder canonicalization은 current one-step/research-based WhiteNight shape를 임시
  placeholder로 분류하고 상세 mechanics를 future/undecided로 남겼다.
- 완료된 BossOmen goal은 새 final-boss content를 scope 밖으로 두며, hardening notes는 Silent Orchestra가
  authored symphony source encounters/events를 얻은 뒤에만 live chain에 추가할 수 있다고 기록한다.
- Skill design notes의 Silent Orchestra 4-step/Apocalypse Bird 3-omen 방향은 exact source IDs, event graphs,
  tuning, rewards와 form transitions가 미정인 concept content다.

따라서 PD005의 closed resolution은 `A-bootstrap`이다.

1. Approved generic per-chain eligibility/cycle/source-failure/capstone engine과 strict validation은 WP001에서
   fixture로 구현한다.
2. Current one-step `white_night` + `suppress_white_night`를 유일한 live development bootstrap으로 둔다.
3. 이 bootstrap은 generic BossOmen E2E와 “현재 live FinalBoss 전부 격파” record를 실제로 검증한다.
4. 이를 최종 WhiteNight chain/content로 승격하지 않으며 상세 mechanics는 계속 future다.
5. Silent Orchestra, Apocalypse Bird와 final WhiteNight chain은 별도 승인 content Goal이 완전한 table을
   소유할 때만 live로 활성화한다.

이 resolution은 2026-08-12 사용자 승인으로 `closed_by_user`다.

## GRB-PD-006 — Fixed Standard Bootstrap Fixture

선택된 minimum `ST1-NT60` bootstrap은 기존 `ST1`의 three topology를 versioned fixed template로 재사용하고
weighted category/content assignment을 제거하되 F3 boss를 reduced Nothing There로 교체한다. 이는 최종
tutorial content/layout/balance 승인이 아니다.

### Floor 1 — `standard_intro_a` topology

| Slot | Fixed category/kind | Fixed content |
| --- | --- | --- |
| `entry` | Start | authored start |
| `intake_corridor` | Combat / `combat_monster` | `normal_corroded_patrol` |
| `records_annex` | Event / `event_sealed_archive` | `sealed_archive_recovery_01` |
| `supply_annex` | Reward / `reward_treasure` | single-entry pool `tutorial_f1_training_budget` → `training_budget_reward` |
| `containment_guard` | Combat / `combat_monster` | `normal_breach_response` |
| `floor_gate` | Gate / `gate_stairs` | fixed Floor 2 transition |

### Floor 2 — `standard_branching_a` topology

| Slot | Fixed category/kind | Fixed content |
| --- | --- | --- |
| `entry` | Start | authored start |
| `dispatch_hall` | Combat / `combat_monster` | `normal_black_box_pressure` |
| `save_archive` | SavePoint / `save_point` | existing SavePoint behavior |
| `maintenance_bay` | Maintenance / `maintenance` | existing Maintenance behavior |
| `hq_terminal` | HeadquartersContact / `headquarters_contact` | pool `headquarters_basic_supplies` |
| `supply_depot` | Shop / `shop_general` | `unknown_exploration_shop` |
| `crossroads_archive` | Event / `event_sealed_archive` | `sealed_archive_recovery_01` |
| `containment_guard` | Combat / `combat_elite` | `suppress_porcubbus` |
| `floor_gate` | Gate / `gate_stairs` | fixed Floor 3 transition |

### Floor 3 — `standard_final_a` topology

| Slot | Fixed category/kind | Fixed content |
| --- | --- | --- |
| `entry` | Start | authored start |
| `west_containment` | Combat / `combat_monster` | `normal_breach_response` |
| `east_containment` | Reward / `reward_treasure` | single-entry pool `tutorial_f3_salvage` → `field_equipment_salvage_reward` |
| `records_core` | Event / `event_sealed_archive` | `sealed_archive_recovery_01` |
| `maintenance_core` | Maintenance / `maintenance` | existing Maintenance behavior |
| `supply_core` | Shop / `shop_general` | `unknown_exploration_shop` |
| `lower_containment` | Combat / `combat_monster` | `normal_black_box_pressure` |
| `upper_containment` | HeadquartersContact / `headquarters_contact` | pool `headquarters_basic_supplies` |
| `final_approach` | Reward / `reward_treasure` | single-entry pool `tutorial_f3_budget` → `large_enkephalin_reward` |
| `final_chamber` | Boss / `boss_abnormality` | simplified NormalBoss `suppress_nothing_there`; F3 boss-actor exact override HP/ATK/DEF/MR `720/53/36/33` (base 60% derivation); terminal semantics `StandardCompletion`, not `RunFinalBoss` |

### Fixed tuning, reward and script contract

- Dedicated `tutorial_fixed_v1` tuning applies encounter-authored tiers/waves at 100% HP/attack/defense/generated
  wave budget/reward scale, adds no waves, and never reads certified power, run peak, Floor anchors, ordinary quota
  or Endless role tuning. F3 `o-06-20_nothing_there` boss actor만 base max HP/attack/defense/magic resistance의
  60%인 exact override 720/53/36/33으로 authoring한다. 60%는 authoring rationale이지 runtime/global
  difficulty multiplier가 아니다. Current generic `BattleUnitStatScale`은 magic resistance가 없고
  encounter-wide이므로 이 계약에 사용할 수 없다. 같은 encounter의 다른 enemy/reward와 다른 Nothing There
  출현에는 감쇠를 적용하지 않는다.
- Standard combat payloads use a fixed Standard-only reward override of `enkephalin_reward` plus
  `field_equipment_salvage_reward`. Existing research rewards and boss fragment rewards are not inherited; Standard
  Elite/Boss victory does not advance Endless acquisition/mastery/cycles.
- Existing `run_start_standard_hq_briefing` → `hq_briefing_standard_intro` remains the only required script trigger.
- `scripted_fragment: None` is explicit. The two current active fragment candidates require Ranged Bow/Gun, while
  every starter uses Melee Sword; granting either fragment alone would teach an unusable loadout. A future tutorial
  fragment sequence must author a compatible weapon/equipment step and its failure conditions together.
- F3 NormalBoss victory atomically commits Standard RunComplete and the permanent tutorial-complete flag. Replays
  grant no additional global reward.

`ST1-NT60`의 장점은 기존 live topology/encounter/event/shop/briefing asset을 재사용하면서 FinalBoss와
adaptive difficulty를 제거하고, Nothing There의 화력검사 정체성을 tutorial 규모로 보여 준다는 점이다.
단점은 동일 `sealed_archive_recovery_01` event를 세 Floor에서 재사용한다는 점이며 production completion은
60% boss tuning의 실제 플레이 검증을 요구한다.

## Bootstrap Resolution

1. `GRB-PD-004`: minimum bootstrap으로 `R1-continuity` 선택.
2. `GRB-PD-006`: minimum bootstrap으로 modified `ST1-NT60` 선택.
3. `GRB-PD-005`: canonical future boundary로 Option B closed.

Open bootstrap input은 0개다. Final content input은 이 audit의 open gate가 아니라 의도적으로 scope 밖에
두고 별도 content Goal이 소유한다. Implementation package는 이 manifest를 initial runnable fixture로
사용하되 production mutation과 test는 별도 승인된 Goal에서 수행한다.
