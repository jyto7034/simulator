# 로보토미 환상체 콘텐츠 1차 조사

이 문서는 로보토미 코퍼레이션 위키를 참고해, 향후 게임 콘텐츠로 옮길 환상체/장비/기프트/스킬 파편 설계의 껍데기를 정리한 것이다.

주의:

- 원작 위키는 환상체를 `스킬 목록` 형태로 정리하지 않는다. 따라서 이 문서의 `원본 행동 키워드`는 작업 피해, 탈출 패턴, 특수 기믹, 장비 컨셉을 우리 게임의 액티브 스킬 설계용으로 요약한 것이다.
- 세부 수치, 피해량, 쿨타임, 범위, 발동 조건은 아직 정하지 않는다.
- 사용/도구형 환상체는 전투 조우 후보에서 제외하거나 특수 이벤트로 보류한다.
- 공개/배포/상업화 계획이 있다면 실제 IP 사용 권한 문제를 별도로 확인해야 한다.

## 참고한 위키 기준

- Fandom `Abnormalities` 문서는 출시 환상체를 위험 등급별로 정리하고, 도감 산정 제외 대상도 설명한다.
- Fandom `Equipment` 문서는 E.G.O Weapon, Suit, Gift의 기본 성격과 환상체별 E.G.O 이름을 정리한다.
- wiki.gg `Abnormalities`, `Equipment`, `Tool Abnormalities` 페이지도 대조하려 했지만 현재 자동 접근에서 403이 발생해, 접근 가능한 Fandom 페이지를 주 기준으로 삼았다.

출처:

- https://lobotomycorp.fandom.com/wiki/Abnormalities
- https://lobotomycorp.fandom.com/wiki/Category:Abnormalities
- https://lobotomycorp.fandom.com/wiki/Equipment
- https://lobotomycorporation.wiki.gg/wiki/Abnormalities
- https://lobotomycorporation.wiki.gg/wiki/Equipment
- https://lobotomycorporation.wiki.gg/wiki/Category:Tool_Abnormalities

## 콘텐츠 분류 원칙

전투 출현 후보:

- 생물형, 침식형, 탈출형, 직접 공격형 환상체.
- 일반 전투, 정예 전투, 보스 전투의 원천으로 사용한다.
- 격파/격리 보상으로 장비 재료, 연구 진행도, 확률적 스킬 파편을 제공할 수 있다.

도구형/특수형 보류:

- 장비처럼 사용하거나 시설 효과를 내는 환상체.
- 전투 유닛으로 직접 등장시키기보다 이벤트, 지원 노드, 정비 노드, 리스크 선택지로 다룬다.
- 예: Don't Touch Me, Mirror of Adjustment, Old Faith and Promise, We Can Change Anything, You Must Be Happy, Behavior Adjustment, Luminous Bracelet, Giant Tree Sap, Notes from a Crazed Researcher, Portrait of Another World, Shelter from the 27th of March, Backward Clock, Express Train to Hell, Flesh Idol 등.

스킬 파편 설계 원칙:

- 환상체 원본 스킬과 직원용 파편 스킬은 같은 원천에서 파생되지만 독립된 스킬 데이터로 둔다.
- 파편은 원본 효과를 그대로 복사하지 않는다. 직원이 감당 가능한 모방, 불완전한 재현, 또는 안전화된 변형이어야 한다.
- 고위험 파편은 신뢰도, 트라우마, 조기 개화와 연결할 수 있다.

## 1차 우선 구현 후보

| 환상체 | 등급 | 원본 행동 키워드 | 직원용 스킬 파편 껍데기 | E.G.O 장비/기프트 |
| --- | --- | --- | --- | --- |
| Der Freischütz | HE | 직선 마탄, 관통, 아군 오사 위험 | `Black Round Fragment`: 직선 탄환, 관통 후보, 아군 경유 시 위험/증폭 | Magic Bullet |
| One Sin and Hundreds of Good Deeds | ZAYIN | 참회, 심판, 회복성 보조 | `Penitence Fragment`: 약한 단일 심판 + 자기 회복 | Penitence |
| The Red Shoes | HE | 유혹, 폭주, 연속 공격 | `Sanguine Impulse Fragment`: 추가 공격 + 짧은 이동/공속 상승 | Sanguine Desire |
| Scorched Girl | TETH | 불꽃, 자폭성 폭발 | `Scorched Spark Fragment`: 작은 투사체/폭발 | Fourth Match Flame |
| Warm-hearted Woodsman | HE | 심장 강탈, 근접 처형 | `Borrowed Heart Fragment`: 체력 흡수/방어 저하 | Logging |
| Scarecrow Searching for Wisdom | HE | 지혜 갈망, 정신/지식 흡수 | `Wisdom Reap Fragment`: 낮은 정신/체력 대상 추적, 적중 시 자원 회복 후보 | Harvest |
| The Burrowing Heaven | WAW | 시선/감시, 응시 실패 처벌 | `Heavenly Gaze Fragment`: 시선 표식, 표식 대상 추가 피해 | Heaven |
| Nothing There | ALEPH | 육체 모방, 변신, 강력한 근접 | `Mimicry Fragment`: 고위험 자가 강화/흡혈성 베기 | Mimicry |
| The Silent Orchestra | ALEPH | 악장 진행, 광역 정신 피해 | `Da Capo Fragment`: 시간차 광역 파동/침묵 | Da Capo |
| WhiteNight | ALEPH | 사도, 축복, 부활/심판 | `Paradise Lost Fragment`: 고위험 회복/심판 복합 | Paradise Lost |

## 전체 환상체 초안 카탈로그

### ZAYIN

| 환상체 | 전투 사용 | 원본 행동 키워드 | E.G.O / Gift | 파편 설계 방향 |
| --- | --- | --- | --- | --- |
| One Sin and Hundreds of Good Deeds | 가능 | 참회, 낮은 위험, 회복성 심판 | Penitence weapon/suit/gift | 초반 안정형 단일 공격 + 소량 회복 |
| Fairy Festival | 가능 | 치유, 보호, 과보호 후 포식 | Wingbeat weapon/suit/gift | 치유 후 조건부 리스크를 가진 보조 액티브 |
| Opened Can of WellCheers | 가능/이벤트 | 유인, 실종, 음료/해양 테마 | Soda weapon/suit/gift | 위치 이동/강제 후퇴/회피형 파편 |
| Plague Doctor | 특수 | 축복, 사도화, WhiteNight 전조 | Benediction gift | 직접 파편보다 축복/낙인 이벤트로 보류 |
| Army in Black | 보스/특수 | 보호자, 시설 붕괴, 집단성 | Pink weapon/suit/gift | 아군 피해를 조건으로 발동하는 방어/희생형 |
| You're Bald... | 보류 | 외형 변화, 개그성 도구 | Tough weapon/suit/gift | 전투 파편보다는 이벤트/장비 개그 효과 |
| Don't Touch Me | 제외 | 조작 금지, 즉사/리셋성 도구 | 없음/보류 | 전투 유닛 제외, 위험 이벤트 후보 |
| Mirror of Adjustment | 제외 | 스탯 재배치 도구 | 없음/보류 | 정비 노드 특수 선택지 후보 |
| Old Faith and Promise | 제외 | 장비 강화/파괴 도구 | 없음/보류 | Maintenance 강화 리스크 이벤트 후보 |
| We Can Change Anything | 제외 | 에너지 생산/희생 도구 | 없음/보류 | 고위험 자원 전환 이벤트 후보 |
| You Must Be Happy | 제외 | 행복/불행 수치 변동 도구 | 없음/보류 | 신뢰도/트라우마 이벤트 후보 |

### TETH

| 환상체 | 전투 사용 | 원본 행동 키워드 | E.G.O / Gift | 파편 설계 방향 |
| --- | --- | --- | --- | --- |
| Scorched Girl | 가능 | 성냥, 화상, 자폭성 폭발 | Fourth Match Flame weapon/suit/gift | 작은 폭발 투사체, 강화 시 범위화 |
| Old Lady | 가능 | 고독, 정신 피해, 방치 | Solitude weapon/suit/gift | 고립된 대상에게 정신/약화 효과 |
| The Lady Facing the Wall | 가능/이벤트 | 벽 응시, 비명, 정신 피해 | Screaming Wedge weapon/suit | 시선 조건 광역 공포/침묵 |
| 1.76 MHz | 보류/가능 | 잡음, 전파, 정신 교란 | Noise suit/gift | 장판형 혼선/명중 저하 |
| Spider Bud | 가능 | 거미, 매복, 포획, 독 | Red Eyes weapon/suit/gift | 표식 + 지속 피해/추적 |
| Beauty and the Beast | 가능 | 변이, 저주, 꽃/짐승 | Horn weapon/suit/gift | 상태 변환형 돌진/방어 전환 |
| Bloodbath | 가능/이벤트 | 피, 욕조, 유혹/자해 | Wrist Cutter weapon/suit/gift | 체력 소모형 공격 또는 출혈 |
| Forsaken Murderer | 가능 | 살인 충동, 둔기, 분노 | Regret weapon/suit/gift | 근접 단타 + 방어 파괴 |
| Punishing Bird | 가능 | 작은 공격, 과잉응징, 반격 | Beak weapon/suit/gift | 약한 선공 + 피격/공격 조건 강반격 |
| Fragment of the Universe | 가능 | 우주 파편, 정신 파동 | Fragments from Somewhere weapon/suit/gift | 원거리 정신 탄환/광역 파동 |
| Crumbling Armor | 보류/가능 | 용기, 무모함, 금기 행동 즉사 | Life for a Daredevil weapon/suit/gift | 고위험 공속/이속 버프, 금기 조건은 추후 |
| Meat Lantern | 가능 | 미끼, 매복, 포식 | Lantern weapon/suit/gift | 적 유인/함정 설치 |
| Today's Shy Look | 가능/이벤트 | 표정 변화, 기분 상태 | Today's Expression weapon/suit/gift | 상태에 따라 다른 효과가 나가는 변동형 |
| Void Dream | 가능 | 수면, 꿈, 정신 이탈 | Engulfing Dream weapon/suit/gift | 수면/무력화/느린 추격 |
| Grave of Cherry Blossoms | 가능 | 벚꽃, 묘지, 유혹/매장 | Cherry Blossoms weapon/suit/gift | 적을 끌어들이는 치유/매혹 장판 |
| Ppodae | 가능 | 개, 물어뜯기, 낮은 위험 | SO CUTE!!! weapon/suit/gift | 단순 돌진/물기 초반 파편 |
| Standard Training-Dummy Rabbit | 제외/튜토리얼 | 훈련용 더미 | Standard Training E.G.O | 튜토리얼 전용 |
| Behavior Adjustment | 제외 | 행동 보정 도구 | 없음/보류 | 장비/신뢰도 이벤트 후보 |
| Luminous Bracelet | 제외 | 회복/위험 도구 | 없음/보류 | 회복 선택지 리스크 후보 |
| Skin Prophecy | 제외 | 예언/피부/정신성 도구 | 없음/보류 | 정보/신뢰도 이벤트 후보 |
| The Heart of Aspiration | 제외 | 열망, 강화, 부작용 도구 | 없음/보류 | 고위험 버프 이벤트 후보 |
| Theresia | 제외 | 음악, 정신 회복/위험 도구 | 없음/보류 | 지원 노드 회복 이벤트 후보 |

### HE

| 환상체 | 전투 사용 | 원본 행동 키워드 | E.G.O / Gift | 파편 설계 방향 |
| --- | --- | --- | --- | --- |
| Happy Teddy Bear | 가능 | 애착, 반복 대상 집착 | Bear Paws weapon/suit/gift | 같은 대상 반복 공격/보호 집착 |
| The Red Shoes | 가능 | 유혹, 폭주, 연속 공격 | Sanguine Desire weapon/suit/gift | 추가 공격 + 짧은 충동 버프 |
| Nameless Fetus | 가능/이벤트 | 울음, 집착, 자원 요구 | Syrinx weapon/suit/gift | 음파/불안 누적/강제 타겟 변경 |
| Singing Machine | 가능/이벤트 | 기계, 음악, 희생 | Harmony weapon/gift | 아군 희생/자원 소모형 광역 효과 후보 |
| Warm-hearted Woodsman | 가능 | 심장, 근접 강타, 결핍 | Logging weapon/suit/gift | 체력 흡수형 베기 |
| The Snow Queen | 가능/보스 | 결투, 얼음, 구출 | Frost Splinter weapon/suit/gift | 빙결/속박, 고립 대상 추가 효과 |
| All-Around Helper | 가능 | 청소 로봇, 회전 칼날 | Grinder Mk4 weapon/suit/gift | 회전 근접 광역/추격 |
| Rudolta of the Sleigh | 가능 | 선물, 썰매, 정신 피해 | Christmas weapon/suit/gift | 무작위 선물/저주 패키지 |
| Child of the Galaxy | 가능/이벤트 | 조약돌, 유대, 보호와 상실 | Our Galaxy weapon/suit/gift | 보호 표식, 표식 대상 피해 공유 |
| Laetitia | 가능 | 선물 상자, 지연 폭발 | Laetitia weapon/suit/gift | 지연 폭탄/저주 선물 |
| The Funeral of the Dead Butterflies | 가능 | 장례, 나비, 원거리 총격 | Solemn Lament weapon/suit/gift | 이중 탄환/처형 표식 |
| Der Freischütz | 가능 | 마탄, 관통, 오사 위험 | Magic Bullet weapon/suit/gift | 직선 관통 고위험 탄환 |
| Schadenfreude | 가능 | 시선, 관찰 금지, 응시 | Gaze weapon/suit/gift | 시야/응시 조건 반격 |
| Porccubus | 가능 | 쾌락, 독, 중독 | Pleasure weapon/suit/gift | 독/쾌락 스택, 장기전 약화 |
| Scarecrow Searching for Wisdom | 가능 | 지혜 갈망, 정신 흡수 | Harvest weapon/suit | 약한 대상 지식 흡수/쿨다운 회수 |
| Giant Tree Sap | 제외 | 회복/독성 도구 | 없음/보류 | 회복 리스크 이벤트 |
| Notes from a Crazed Researcher | 제외 | 연구 기록 도구 | 없음/보류 | 연구 진행도/트라우마 이벤트 |
| Portrait of Another World | 제외 | 피해 전가 도구 | 없음/보류 | 고위험 방어 계약 이벤트 |
| Shelter from the 27th of March | 제외 | 대피소 도구 | 없음/보류 | 안전 노드/방치 이벤트 |

### WAW

| 환상체 | 전투 사용 | 원본 행동 키워드 | E.G.O / Gift | 파편 설계 방향 |
| --- | --- | --- | --- | --- |
| Judgement Bird | 가능/정예 | 심판, 저울, 처형 | Justitia weapon/suit/gift | 낮은 체력/죄 표식 대상 처형 |
| Snow White's Apple | 가능 | 뿌리, 덩굴, 속박 | Green Stem weapon/suit/gift | 속박 장판/뿌리 추적 |
| Big and Will be Bad Wolf | 가능 | 늑대, 포식, 추격 | Cobalt Scar weapon/suit/gift | 추격 돌진/포식 표식 |
| Big Bird | 가능/정예 | 등불, 매혹, 즉사성 부리 | Lamp weapon/suit | 등불 표식/매혹 후 처형 |
| Dream of a Black Swan | 가능 | 형제, 검은 물, 광역 | Black Swan weapon/suit | 파동형 광역/피해 공유 |
| The King of Greed | 가능/정예 | 탐욕, 돌진, 황금 | Gold Rush weapon/suit/gift | 돌진 관통/보상 탐욕 조건 |
| The Dreaming Current | 가능 | 돌진, 전류, 어린 개체 | Ecstasy suit | 빠른 돌진/충돌 피해 |
| Dimensional Refraction Variant | 가능 | 투명화, 차원 왜곡 | Diffraction weapon | 은신/위치 왜곡 타격 |
| The Knight of Despair | 가능/이벤트 | 축복, 절망, 검 | The Sword Sharpened with Tears weapon/suit/gift | 보호 축복 후 붕괴 리스크 |
| The Burrowing Heaven | 가능/정예 | 감시, 시선 실패 처벌 | Heaven weapon/suit/gift | 시선 표식/감시 장판 |
| Queen Bee | 가능 | 벌, 감염, 일벌 생성 | Hornet weapon/suit | 감염 표식/소환형 |
| Alriune | 가능 | 꽃, 향기, 정신 피해 | Faint Aroma weapon/suit/gift | 향기 장판/정신 약화 |
| The Queen of Hatred | 가능/정예 | 마법소녀, 사랑/증오, 빔 | In the Name of Love and Hate weapon/suit/gift | 원거리 빔/감정 상태 전환 |
| The Naked Nest | 가능 | 기생, 둥지, 감염 | Exuviae weapon/suit | 감염 누적/변이 위험 |
| Yin | 가능/특수 | 흑백 공명, Yang과 결합 | Discord weapon/suit | 공명 디버프/쌍둥이 이벤트 |
| Yang | 가능/특수 | 흑백 공명, Yin과 결합 | 장비 미확인 | 공명 회복/쌍둥이 이벤트 |
| Little Red Riding Hooded Mercenary | 가능/특수 | 사냥꾼, 총격, 늑대 적대 | Crimson Scar weapon/suit/gift | 지정 표적 추격/복수 |
| The Firebird | 가능 | 화염, 부활, 폭발 | Feather of Honor weapon/suit | 화염 돌진/부활성 버프 |
| The Little Prince | 가능 | 포자, 감염, 변이 | Spore weapon/suit | 포자 감염/지속 피해 |
| Il Pianto della Luna | 가능 | 달빛, 음악, 정신 피해 | Moonlight weapon/suit/gift | 달빛 파동/정신 약화 |
| Clouded Monk | 가능 | 탐식, 구름, 구슬 | Amita weapon/suit/gift | 누적 섭취/체력 흡수 |
| Parasite Tree | 가능/이벤트 | 축복, 기생, 나무 | 장비 미확인 | 축복 표식이 누적되면 기생으로 전환 |
| Backward Clock | 제외 | 시간 되감기 도구 | 없음/보류 | 강력한 리셋 이벤트 후보 |
| Express Train to Hell | 제외 | 열차, 시간표, 대가 | 없음/보류 | 노드 진행/시간 리스크 이벤트 |
| Flesh Idol | 제외 | 숭배, 피해 전파 도구 | 없음/보류 | 고위험 광역 버프/희생 이벤트 |

### ALEPH

| 환상체 | 전투 사용 | 원본 행동 키워드 | E.G.O / Gift | 파편 설계 방향 |
| --- | --- | --- | --- | --- |
| Nothing There | 보스/정예 | 모방, 변신, 재생, 강력한 근접 | Mimicry weapon/suit/gift | 고위험 자가 강화/흡혈 베기 |
| Blue Star | 보스/정예 | 흡인, 정신 붕괴, 별 | Sound of a Star weapon/suit/gift | 중심 흡인/정신 피해 파동 |
| CENSORED | 보스/정예 | 공포, 인지 불가, 정신 붕괴 | CENSORED weapon | 공포 노출/정보 은폐형 디버프 |
| Mountain of Smiling Bodies | 보스/정예 | 시체 흡수, 성장, 광역 포식 | Smile weapon/suit/gift | 처치/전투불능 발생 시 성장하는 파편 |
| The Silent Orchestra | 보스 | 악장, 음악, 광역 정신 피해 | Da Capo weapon/suit/gift | 단계별 광역 파동/침묵 |
| Apocalypse Bird | 최상위 보스 | 세 새의 결합, 심판, 눈/부리/등불 | Twilight weapon/suit/gift | 엔드게임 전용 복합 파편 후보 |
| WhiteNight | 최상위 보스 | 사도, 축복, 부활/심판 | Paradise Lost weapon/suit/gift | 엔드게임 전용 회복+심판 복합 |
| Army in Black | 보스/특수 | 보호자, 집단 붕괴, 희생 | Pink weapon/suit/gift | 아군 피해/사망 반응형 방어/폭발 |
| Melting Love | 보스/정예 | 점액, 감염, 확산 | Adoration weapon/suit/gift | 감염/점액 장판/지속 피해 |

## 장비/기프트 구현 메모

원작 장비는 보통 환상체별로 같은 이름의 Weapon, Suit, Gift 세트를 가진다. 일부 환상체는 Weapon만 있거나, Suit/Gift가 없거나, Gift만 존재한다.

우리 게임에서는 다음처럼 단순화하는 편이 좋다.

- `Weapon`: 공격 방식, 공격 속도, 사거리, 조건부 발동 능력.
- `Armor`: 생존력, 피해 저항, 트라우마 저항, 특정 환상체 대응.
- `Accessory`: 원작 Gift의 감정/기억/특수 조건을 계승하는 슬롯.

원작 Gift를 그대로 장착품으로 옮기기보다, 이 게임에서는 `기프트`, `기억 장식`, `침식 흔적`, `연구 부산물` 중 하나로 역할을 재정의할 수 있다. 직원 신뢰도 시스템과 연결하려면 Accessory/Gift는 단순 스탯보다 기억, 트라우마 저항, 위험 파편 안정화에 쓰는 편이 좋다.

## 다음 조사 단계

1. 각 환상체 개별 위키 페이지를 열어 실제 탈출 패턴, 작업 피해 타입, 특수 조건을 검증한다.
2. `도구형 제외` 목록을 확정한다. 일부 도구형은 전투 유닛이 아니라 이벤트/정비/지원 노드로 재설계한다.
3. 우선 구현 환상체 10개를 정하고, 각 환상체에 대해 `원본 스킬`, `파편 스킬`, `무기`, `방어구`, `악세서리` 껍데기를 만든다.
4. 이후 RON 데이터 작성은 수치 없이 ID/name/description/역할 태그부터 넣는다.
