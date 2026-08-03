# Live E.G.O 장비·스킬 파편 색인

이 문서는 확정 환상체 로스터에서 현재 live data에 연결된 E.G.O placeholder 장비와 스킬
파편을 한곳에서 찾기 위한 파생 색인이다. 30개 abnormality-derived 파편을 모두 싣되, 그중
22개만 현재 완전한 3-slot placeholder E.G.O set과 함께 연결된다. 콘텐츠 로스터 정책은
`lobotomy_content_catalog.md`가 담당한다.

이 색인은 구현 계약이나 field shape의 source of truth가 아니다. 실제 사용 가능 여부, ID, rarity, 효과, 장비 field shape는 아래 live RON과 runtime loader/validation을 최종 근거로 확인한다.

- E.G.O 장비: `game_resources/data/equipments/base.ron`
- 스킬 파편: `game_resources/data/skill_fragments/base.ron`

현재 E.G.O 세트와 trace 계열 파편 대부분은 콘텐츠 제작, admin catalog, Unity inventory/loadout 흐름을 검증하기 위한 placeholder다. 색인의 요약과 표가 live RON과 다르면 live RON을 기준으로 이 문서를 갱신한다.

## 읽는 법

- `Legacy Grade`는 placeholder data 작성 당시의 과거 분류이며 최종 encounter tier가 아니다.
- `E.G.O Rarity`와 장비 ID는 현재 `equipments/base.ron`의 값이다. 각 Weapon은 schema validation을 위한 최소 `weapon_profile`을 가진다. 그중 필수 `block_capacity`는 승인된 현재 gameplay authoring 값이며, 나머지 placeholder profile 수치를 최종 밸런스 의도로 해석하지 않는다.
- `Fragment Rarity`와 `Current Effect`는 현재 `skill_fragments/base.ron`의 값이다.
- `ActiveSkill` 파편은 독립 스킬을 가리킨다. trace 파편은 dedicated skill 설계 전까지 작은 기본 공격 보정만 제공한다.
- Formal fragment 정책은 active-skill-centered다. 표의 직접 `BasicAttackModifier` 29개는 정확한
  현재 placeholder 관측값일 뿐 정식 fragment 설계의 예시가 아니다.
- WhiteNight 행은 현재 one-step/research-acquisition placeholder만 기록한다. 12표식, 사도,
  제3세력과 boss-clear fragment는 미래·미정이며 이 색인이 구현을 약속하지 않는다.
- 도구형 환상체는 E.G.O placeholder 세트나 일반 스킬 파편의 대상이 아니다. 도구 로스터와 데이터 정책은 `lobotomy_content_catalog.md`의 도구형 콘텐츠 섹션에서 관리한다.

## Summary

| 항목 | 값 |
| --- | --- |
| 환상체 범위 | 현재 live abnormality-derived fragment 30개; full placeholder E.G.O set 연결 22개 |
| 환상체 수 | 30 (E.G.O 3-slot set 동반 22) |
| E.G.O 장비 수 | 66 (Weapon/Armor/Accessory 각 22) |
| 스킬 파편 수 | abnormality-derived 30 (concept starter 포함 전체 live 31) |
| 파편 효과 구성 | abnormality-derived: ActiveSkill 2, BasicAttackModifier 28; concept starter: BasicAttackModifier 1 |
| 색인 상태 | Derived placeholder index |

## Live Content Index

| Legacy Grade | Abnormality | E.G.O Rarity | Equipment IDs (Weapon / Armor / Accessory) | Fragment ID | Fragment Rarity | Current Effect |
| --- | --- | --- | --- | --- | --- | --- |
| S | WhiteNight | ALEPH | `placeholder_white_night_weapon`<br>`placeholder_white_night_armor`<br>`placeholder_white_night_accessory` | `fragment_white_night_trace` | Exceptional | BasicAttackModifier (+6 ATK / -60 ms) |
| S | Nothing There | ALEPH | `placeholder_nothing_there_weapon`<br>`placeholder_nothing_there_armor`<br>`placeholder_nothing_there_accessory` | `fragment_nothing_there_trace` | Exceptional | BasicAttackModifier (+6 ATK / -60 ms) |
| S | Apocalypse Bird | ALEPH | `placeholder_apocalypse_bird_weapon`<br>`placeholder_apocalypse_bird_armor`<br>`placeholder_apocalypse_bird_accessory` | `fragment_apocalypse_bird_trace` | Exceptional | BasicAttackModifier (+6 ATK / -60 ms) |
| S | Queen of Hatred | ALEPH | `placeholder_queen_of_hatred_weapon`<br>`placeholder_queen_of_hatred_armor`<br>`placeholder_queen_of_hatred_accessory` | `fragment_queen_of_hatred_trace` | Exceptional | BasicAttackModifier (+6 ATK / -60 ms) |
| S | King of Greed | WAW | `placeholder_king_of_greed_weapon`<br>`placeholder_king_of_greed_armor`<br>`placeholder_king_of_greed_accessory` | `fragment_king_of_greed_trace` | Exceptional | BasicAttackModifier (+6 ATK / -60 ms) |
| S | Knight of Despair | WAW | `placeholder_knight_of_despair_weapon`<br>`placeholder_knight_of_despair_armor`<br>`placeholder_knight_of_despair_accessory` | `fragment_knight_of_despair_trace` | Exceptional | BasicAttackModifier (+6 ATK / -60 ms) |
| S | Mountain of Smiling Bodies | ALEPH | `placeholder_mountain_smiling_bodies_weapon`<br>`placeholder_mountain_smiling_bodies_armor`<br>`placeholder_mountain_smiling_bodies_accessory` | `fragment_mountain_smiling_bodies_trace` | Exceptional | BasicAttackModifier (+6 ATK / -60 ms) |
| S | CENSORED | ALEPH | `placeholder_censored_weapon`<br>`placeholder_censored_armor`<br>`placeholder_censored_accessory` | `fragment_censored_trace` | Exceptional | BasicAttackModifier (+6 ATK / -60 ms) |
| S | The Silent Orchestra | ALEPH | `placeholder_silent_orchestra_weapon`<br>`placeholder_silent_orchestra_armor`<br>`placeholder_silent_orchestra_accessory` | `fragment_silent_orchestra_trace` | Exceptional | BasicAttackModifier (+6 ATK / -60 ms) |
| S | Melting Love | ALEPH | `placeholder_melting_love_weapon`<br>`placeholder_melting_love_armor`<br>`placeholder_melting_love_accessory` | `fragment_melting_love_trace` | Exceptional | BasicAttackModifier (+6 ATK / -60 ms) |
| A | Der Freischutz | HE | `placeholder_der_freischutz_weapon`<br>`placeholder_der_freischutz_armor`<br>`placeholder_der_freischutz_accessory` | `fragment_freischutz_black_round` | Rare | ActiveSkill (independent skill) |
| A | The Funeral of the Dead Butterflies | HE | `placeholder_funeral_butterfly_weapon`<br>`placeholder_funeral_butterfly_armor`<br>`placeholder_funeral_butterfly_accessory` | `fragment_funeral_butterfly_eulogy` | Rare | ActiveSkill (independent skill) |
| A | Little Red Riding Hooded Mercenary | WAW | `placeholder_little_red_weapon`<br>`placeholder_little_red_armor`<br>`placeholder_little_red_accessory` | `fragment_little_red_trace` | Rare | BasicAttackModifier (+4 ATK / -40 ms) |
| A | Big and Will be Bad Wolf | WAW | `placeholder_big_bad_wolf_weapon`<br>`placeholder_big_bad_wolf_armor`<br>`placeholder_big_bad_wolf_accessory` | `fragment_big_bad_wolf_trace` | Rare | BasicAttackModifier (+4 ATK / -40 ms) |
| A | Laetitia | WAW | `placeholder_laetitia_weapon`<br>`placeholder_laetitia_armor`<br>`placeholder_laetitia_accessory` | `fragment_laetitia_trace` | Rare | BasicAttackModifier (+4 ATK / -40 ms) |
| A | The Burrowing Heaven | WAW | `placeholder_burrowing_heaven_weapon`<br>`placeholder_burrowing_heaven_armor`<br>`placeholder_burrowing_heaven_accessory` | `fragment_burrowing_heaven_trace` | Rare | BasicAttackModifier (+4 ATK / -40 ms) |
| B | Warm-hearted Woodsman | HE | `placeholder_warm_hearted_woodsman_weapon`<br>`placeholder_warm_hearted_woodsman_armor`<br>`placeholder_warm_hearted_woodsman_accessory` | `fragment_warm_hearted_woodsman_trace` | Common | BasicAttackModifier (+3 ATK / -20 ms) |
| B | Queen Bee | WAW | `placeholder_queen_bee_weapon`<br>`placeholder_queen_bee_armor`<br>`placeholder_queen_bee_accessory` | `fragment_queen_bee_trace` | Common | BasicAttackModifier (+3 ATK / -20 ms) |
| B | Dream of a Black Swan | WAW | `placeholder_black_swan_weapon`<br>`placeholder_black_swan_armor`<br>`placeholder_black_swan_accessory` | `fragment_black_swan_trace` | Common | BasicAttackModifier (+3 ATK / -20 ms) |
| B | Yin | WAW | `placeholder_yin_weapon`<br>`placeholder_yin_armor`<br>`placeholder_yin_accessory` | `fragment_yin_trace` | Common | BasicAttackModifier (+3 ATK / -20 ms) |
| B | Yang | WAW | `placeholder_yang_weapon`<br>`placeholder_yang_armor`<br>`placeholder_yang_accessory` | `fragment_yang_trace` | Common | BasicAttackModifier (+3 ATK / -20 ms) |
| B | Schadenfreude | HE | `placeholder_schadenfreude_weapon`<br>`placeholder_schadenfreude_armor`<br>`placeholder_schadenfreude_accessory` | `fragment_schadenfreude_trace` | Common | BasicAttackModifier (+3 ATK / -20 ms) |
| — | The Snow Queen | — | — | `fragment_snow_queen_trace` | Common | BasicAttackModifier (+3 ATK / -20 ms) |
| — | Rudolta of the Sleigh | — | — | `fragment_rudolta_trace` | Common | BasicAttackModifier (+3 ATK / -20 ms) |
| — | Parasite Tree | — | — | `fragment_parasite_tree_trace` | Common | BasicAttackModifier (+3 ATK / -20 ms) |
| — | Porccubus | — | — | `fragment_porcubbus_trace` | Common | BasicAttackModifier (+3 ATK / -20 ms) |
| — | Scarecrow Searching for Wisdom | — | — | `fragment_scarecrow_trace` | Common | BasicAttackModifier (+3 ATK / -20 ms) |
| — | Moonlit Wail | — | — | `fragment_moonlit_wail_trace` | Rare | BasicAttackModifier (+4 ATK / -40 ms) |
| — | Nameless Fetus | — | — | `fragment_nameless_fetus_trace` | Common | BasicAttackModifier (+3 ATK / -20 ms) |
| — | Clouded Monk | — | — | `fragment_clouded_monk_trace` | Common | BasicAttackModifier (+3 ATK / -20 ms) |

## Maintenance Policy

- 확정 로스터 밖 legacy placeholder 장비와 파편은 색인에 유지하지 않는다. 다시 추가하려면 먼저 `lobotomy_content_catalog.md`의 로스터 정책을 갱신하고 live RON/validation을 함께 변경한다.
- dedicated active skill을 설계하면 해당 trace 파편을 `ActiveSkill` 파편으로 교체하거나 별도 상위 파편으로 분리한다.
- 색인은 live RON의 일부 field를 탐색용으로 투영할 뿐이다. 새 field나 효과 의미를 이 문서에서 먼저 정의하지 않는다.
- live 파편/장비 membership 변경 시 `docs/audit/live_skill_catalog_evidence_contract.md`의 lifecycle에
  따라 이 파생 색인과 manifest/test evidence를 같은 변경에서 갱신한다.
