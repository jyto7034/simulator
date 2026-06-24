# Skill Fragment Wiki

이 문서는 현재 live placeholder data 중 확정 로스터에 남아 있는 스킬 파편만 정리한다.

현재 일부 파편은 독립 active skill을 가진 구현 파편이고, 나머지는 콘텐츠 제작과 admin catalog, Unity loadout UI 검증을 위한 단순 basic attack modifier 파편이다.

live data의 파편은 확정 로스터에 포함된 환상체 파생 항목만 유지한다. 확정 로스터 밖 legacy 파편은 제거 대상이며, 다시 추가하려면 먼저 `lobotomy_content_catalog.md`의 로스터 정책을 갱신해야 한다.

도구형 환상체는 이 문서의 대상이 아니다. 도구형은 `tool_abnormality_wiki.md`에서 관리한다.

## 읽는 법

- `Legacy Grade`: 현재 placeholder data가 만들어질 때 사용한 과거 구분이다. 최종 encounter tier가 아니다.
- `Fragment ID`: admin grant catalog, inventory, loadout command에서 쓰는 skill fragment id다.
- `Effect`: 현재 RON에 적힌 파편 효과 유형이다.
- `Simple Effect`: active skill 파편은 독립 스킬을 가리키며, trace 파편은 작은 평타 보정만 제공한다.

## Summary

| 항목 | 값 |
| --- | --- |
| 환상체 범위 | 확정 로스터에 포함된 live placeholder 후보 |
| 환상체 수 | 22 |
| 파편 수 | 22 |
| ActiveSkill 파편 | 2 |
| BasicAttackModifier 파편 | 20 |
| Source data | `game_resources/data/skill_fragments/base.ron` |
| Policy source | `docs/skills/lobotomy_content_catalog.md` |

## Fragment Index

| Legacy Grade | Abnormality | Fragment ID | Rarity | Effect | Simple Effect |
| --- | --- | --- | --- | --- | --- |
| S | WhiteNight | `fragment_white_night_trace` | Exceptional | BasicAttackModifier | +6 atk / -60 ms |
| S | Nothing There | `fragment_nothing_there_trace` | Exceptional | BasicAttackModifier | +6 atk / -60 ms |
| S | Apocalypse Bird | `fragment_apocalypse_bird_trace` | Exceptional | BasicAttackModifier | +6 atk / -60 ms |
| S | Queen of Hatred | `fragment_queen_of_hatred_trace` | Exceptional | BasicAttackModifier | +6 atk / -60 ms |
| S | King of Greed | `fragment_king_of_greed_trace` | Exceptional | BasicAttackModifier | +6 atk / -60 ms |
| S | Knight of Despair | `fragment_knight_of_despair_trace` | Exceptional | BasicAttackModifier | +6 atk / -60 ms |
| S | Mountain of Smiling Bodies | `fragment_mountain_smiling_bodies_trace` | Exceptional | BasicAttackModifier | +6 atk / -60 ms |
| S | CENSORED | `fragment_censored_trace` | Exceptional | BasicAttackModifier | +6 atk / -60 ms |
| S | The Silent Orchestra | `fragment_silent_orchestra_trace` | Exceptional | BasicAttackModifier | +6 atk / -60 ms |
| S | Melting Love | `fragment_melting_love_trace` | Exceptional | BasicAttackModifier | +6 atk / -60 ms |
| A | Der Freischutz | `fragment_freischutz_black_round` | Rare | ActiveSkill | Independent active skill |
| A | The Funeral of the Dead Butterflies | `fragment_funeral_butterfly_eulogy` | Rare | ActiveSkill | Independent active skill |
| A | Little Red Riding Hooded Mercenary | `fragment_little_red_trace` | Rare | BasicAttackModifier | +4 atk / -40 ms |
| A | Big and Will be Bad Wolf | `fragment_big_bad_wolf_trace` | Rare | BasicAttackModifier | +4 atk / -40 ms |
| A | Laetitia | `fragment_laetitia_trace` | Rare | BasicAttackModifier | +4 atk / -40 ms |
| A | The Burrowing Heaven | `fragment_burrowing_heaven_trace` | Rare | BasicAttackModifier | +4 atk / -40 ms |
| B | Warm-hearted Woodsman | `fragment_warm_hearted_woodsman_trace` | Common | BasicAttackModifier | +3 atk / -20 ms |
| B | Queen Bee | `fragment_queen_bee_trace` | Common | BasicAttackModifier | +3 atk / -20 ms |
| B | Dream of a Black Swan | `fragment_black_swan_trace` | Common | BasicAttackModifier | +3 atk / -20 ms |
| B | Yin | `fragment_yin_trace` | Common | BasicAttackModifier | +3 atk / -20 ms |
| B | Yang | `fragment_yang_trace` | Common | BasicAttackModifier | +3 atk / -20 ms |
| B | Schadenfreude | `fragment_schadenfreude_trace` | Common | BasicAttackModifier | +3 atk / -20 ms |

## Notes

- 새로 추가한 trace 계열 파편은 dedicated skill 구현 전까지 쓰는 단순 파편이다.
- dedicated active skill을 설계하면 해당 trace 파편을 `ActiveSkill` 파편으로 교체하거나 별도 상위 파편으로 분리한다.
- 확정 로스터 밖 파편은 이 문서에서 열거하지 않는다.
