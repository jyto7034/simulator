# E.G.O Equipment Wiki

이 문서는 현재 live placeholder data 중 확정 로스터에 남아 있는 E.G.O 장비만 정리한다.

현재 장비는 콘텐츠 제작, admin catalog, Unity inventory/loadout UI 검증을 위한 placeholder다. 세부 스탯, 고유 능력, 강화 곡선, 분쇄/강화 레시피는 아직 확정하지 않는다.

live data의 placeholder E.G.O 장비는 확정 로스터에 포함된 환상체 파생 항목만 유지한다. 확정 로스터 밖 legacy placeholder 장비는 제거 대상이며, 다시 추가하려면 먼저 `lobotomy_content_catalog.md`의 로스터 정책을 갱신해야 한다.

도구형 환상체는 이 문서의 대상이 아니다. 장착품처럼 작동하는 도구도 E.G.O placeholder 장비와 같은 의미로 보지 않으며, `tool_abnormality_wiki.md`에서 관리한다.

## 읽는 법

- `Legacy Grade`: 현재 placeholder data가 만들어질 때 사용한 과거 구분이다. 최종 encounter tier가 아니다.
- `Rarity`: 현재 `game_resources/data/equipments/base.ron`에 적힌 placeholder 장비 rarity다.
- `Weapon`, `Armor`, `Accessory`: admin grant catalog와 inventory에서 쓰는 장비 definition id다.
- `Weapon`은 schema validation을 위해 최소 `weapon_profile`을 가진다. 이 값은 밸런스 의도가 아니라 장착/표시 흐름 검증용 기본값이다.

## Summary

| 항목 | 값 |
| --- | --- |
| 환상체 범위 | 확정 로스터에 포함된 live placeholder 후보 |
| 환상체 수 | 22 |
| 장비 수 | 66 |
| 장비 상태 | Placeholder |
| Source data | `game_resources/data/equipments/base.ron` |
| Policy source | `docs/skills/lobotomy_content_catalog.md` |

## Equipment Index

| Legacy Grade | Abnormality | Rarity | Weapon | Armor | Accessory |
| --- | --- | --- | --- | --- | --- |
| S | WhiteNight | ALEPH | `placeholder_white_night_weapon` | `placeholder_white_night_armor` | `placeholder_white_night_accessory` |
| S | Nothing There | ALEPH | `placeholder_nothing_there_weapon` | `placeholder_nothing_there_armor` | `placeholder_nothing_there_accessory` |
| S | Apocalypse Bird | ALEPH | `placeholder_apocalypse_bird_weapon` | `placeholder_apocalypse_bird_armor` | `placeholder_apocalypse_bird_accessory` |
| S | Queen of Hatred | ALEPH | `placeholder_queen_of_hatred_weapon` | `placeholder_queen_of_hatred_armor` | `placeholder_queen_of_hatred_accessory` |
| S | King of Greed | WAW | `placeholder_king_of_greed_weapon` | `placeholder_king_of_greed_armor` | `placeholder_king_of_greed_accessory` |
| S | Knight of Despair | WAW | `placeholder_knight_of_despair_weapon` | `placeholder_knight_of_despair_armor` | `placeholder_knight_of_despair_accessory` |
| S | Mountain of Smiling Bodies | ALEPH | `placeholder_mountain_smiling_bodies_weapon` | `placeholder_mountain_smiling_bodies_armor` | `placeholder_mountain_smiling_bodies_accessory` |
| S | CENSORED | ALEPH | `placeholder_censored_weapon` | `placeholder_censored_armor` | `placeholder_censored_accessory` |
| S | The Silent Orchestra | ALEPH | `placeholder_silent_orchestra_weapon` | `placeholder_silent_orchestra_armor` | `placeholder_silent_orchestra_accessory` |
| S | Melting Love | ALEPH | `placeholder_melting_love_weapon` | `placeholder_melting_love_armor` | `placeholder_melting_love_accessory` |
| A | Der Freischutz | HE | `placeholder_der_freischutz_weapon` | `placeholder_der_freischutz_armor` | `placeholder_der_freischutz_accessory` |
| A | The Funeral of the Dead Butterflies | HE | `placeholder_funeral_butterfly_weapon` | `placeholder_funeral_butterfly_armor` | `placeholder_funeral_butterfly_accessory` |
| A | Little Red Riding Hooded Mercenary | WAW | `placeholder_little_red_weapon` | `placeholder_little_red_armor` | `placeholder_little_red_accessory` |
| A | Big and Will be Bad Wolf | WAW | `placeholder_big_bad_wolf_weapon` | `placeholder_big_bad_wolf_armor` | `placeholder_big_bad_wolf_accessory` |
| A | Laetitia | WAW | `placeholder_laetitia_weapon` | `placeholder_laetitia_armor` | `placeholder_laetitia_accessory` |
| A | The Burrowing Heaven | WAW | `placeholder_burrowing_heaven_weapon` | `placeholder_burrowing_heaven_armor` | `placeholder_burrowing_heaven_accessory` |
| B | Warm-hearted Woodsman | HE | `placeholder_warm_hearted_woodsman_weapon` | `placeholder_warm_hearted_woodsman_armor` | `placeholder_warm_hearted_woodsman_accessory` |
| B | Queen Bee | WAW | `placeholder_queen_bee_weapon` | `placeholder_queen_bee_armor` | `placeholder_queen_bee_accessory` |
| B | Dream of a Black Swan | WAW | `placeholder_black_swan_weapon` | `placeholder_black_swan_armor` | `placeholder_black_swan_accessory` |
| B | Yin | WAW | `placeholder_yin_weapon` | `placeholder_yin_armor` | `placeholder_yin_accessory` |
| B | Yang | WAW | `placeholder_yang_weapon` | `placeholder_yang_armor` | `placeholder_yang_accessory` |
| B | Schadenfreude | HE | `placeholder_schadenfreude_weapon` | `placeholder_schadenfreude_armor` | `placeholder_schadenfreude_accessory` |
