# 환상체 콘텐츠 카탈로그

이 문서는 환상체 원본 스킬, 직원용 스킬 파편, 장비/기프트 껍데기를 설계하기 위한 현재 콘텐츠 기준이다. 과거 조사 로그나 구현 완료 보고가 아니라, 앞으로 콘텐츠를 추가할 때 유지해야 할 분류와 설계 원칙만 남긴다.

주의:

- 원작 위키는 환상체를 `스킬 목록` 형태로 정리하지 않는다. 이 문서의 행동 키워드는 작업 피해, 탈출 패턴, 특수 기믹, 장비 컨셉을 우리 게임의 전투/스킬 파편 구조에 맞게 요약한 것이다.
- 세부 수치, 쿨타임, 범위, 발동 조건은 별도 밸런싱 단계에서 정한다.
- 사용/도구형 환상체는 전투 조우 후보에서 제외하거나 특수 이벤트/정비/지원 노드 후보로 보류한다.
- 공개/배포/상업화 계획이 있다면 IP 사용 권한 문제를 별도로 확인해야 한다.

## 콘텐츠 분류 원칙

전투 출현 후보:

- 생물형, 침식형, 탈출형, 직접 공격형 환상체.
- 일반 전투, 정예 전투, 보스 전투의 원천으로 사용한다.
- 격파/격리 보상으로 장비 재료, 연구 진행도, 낮은 확률의 스킬 파편을 제공할 수 있다.

도구형/특수형 보류:

- 장비처럼 사용하거나 시설 효과를 내는 환상체.
- 전투 유닛으로 직접 등장시키기보다 이벤트, 지원 노드, 정비 노드, 리스크 선택지로 다룬다.
- 예: `Don't Touch Me`, `Mirror of Adjustment`, `Old Faith and Promise`, `We Can Change Anything`, `You Must Be Happy`, `Behavior Adjustment`, `Luminous Bracelet`, `Giant Tree Sap`, `Notes from a Crazed Researcher`, `Portrait of Another World`, `Shelter from the 27th of March`, `Backward Clock`, `Express Train to Hell`, `Flesh Idol`.

스킬 파편 설계 원칙:

- 환상체 원본 스킬과 직원용 파편 스킬은 같은 원천에서 파생되지만 독립된 `SkillDef`다.
- 기본 파편은 원본의 하위 호환이 아니라, 인간이 안전화/왜곡해서 쓰는 독립 스킬이다.
- 개화 전에는 원본을 그대로 쓰지 않는다. 개화 후에만 원본에 가까운 완전 모방 또는 별도 파생 스킬을 허용한다.
- 고위험 파편은 신뢰도, 트라우마, 조기 개화와 연결할 수 있다.
- 스킬 파편은 직업 스킬이 아니라 환상체 테마 효과 모듈이다.
- 무기 아키타입은 파편의 장착 조건과 일부 변형/강화 조건으로만 작동한다.
- 파편 하나를 모든 무기 아키타입에 완벽 대응시키지 않는다. 호환되지 않는 무기는 장착 불가로 처리한다.
- 파편 요구 조건은 직업명보다 전투 프로필 기준으로 쓴다. 예: `range_role`, `weapon_archetype`, `damage_type`, `targeting_profile`, `air_capable`, `block_capacity_min`.
- 파편 효과는 `BasicAttackModifier`, `ActiveSkill`, `PassiveAura`, `OnBlock`, `OnKill`, `OnHit`, `Periodic` 같은 적용 모드로 분류한다.
- 파편별 variant rule은 1~2개 수준으로 제한한다. 예: 유탄은 폭발 반경 증가, 근거리는 저지 반응, 원거리는 명중 반응, 마법 무기는 마법 저항 감소 추가.
- 유명하거나 스킬 파편화하기 좋은 환상체는 특정 무기 아키타입 또는 아키타입 그룹 전용 파편으로 설계해 개성을 살린다.
- 환상체 정체성이 강한 파편일수록 장착 제한을 좁게 둔다.
- 환상체 정체성이 약하거나 빌드 보조, 안정화, 자원 순환에 가까운 파편은 범용 호환을 유지한다.
- 아키타입 전용 파편은 플레이어가 특정 무기를 준비하게 만드는 빌드 목표로 사용한다. 단, 보상 풀은 현재 보유 무기와 맞지 않는 전용 파편만 반복 지급하지 않도록 보정해야 한다.

무기 아키타입과 타겟팅 프로필:

- 직원의 현재 전투 역할은 고정 직업이 아니라 장착 무기가 만든다.
- 무기 대분류는 `Melee`와 `Ranged`다.
- 근거리 예시는 검, 창, 도끼, 방패다.
- 원거리 예시는 활, 석궁, 총, 샷건, 유탄, 스태프다.
- 무기 아키타입은 기본 공격 타입, 사거리, 공격 범위, 공중 공격 가능 여부, 기본 `targeting_profile`을 제공한다.
- 초기 타겟팅 프로필은 `DefaultForward`, `AirFirst`, `LowDefenseFirst`, `LowMagicResistFirst`, `SplashClusterFirst`로 시작한다.

초기 무기/타겟팅 매핑:

| 무기 아키타입 | 권장 프로필 | 설계 메모 |
| --- | --- | --- |
| Sword | DefaultForward | 표준 근접. 저지 대상 우선 후 경로 진행도 높은 적 |
| Spear | DefaultForward | 전방 긴 범위. route progress 높은 적을 찌르는 감각 |
| Axe | DefaultForward | 느린 고화력 또는 소규모 광역 |
| Shield | DefaultForward | 저지/생존 중심. 저지 대상 고정 우선 |
| Bow/Crossbow | AirFirst | 대공/표준 원거리 |
| Gun | LowDefenseFirst 또는 DefaultForward | 정밀 사격, 낮은 방어 적 처리 |
| Staff | LowMagicResistFirst | 마법 원거리, 고방어 적 대응 |
| Shotgun | DefaultForward | 짧은 사거리, 전방 확산/다중 타격 |
| GrenadeLauncher | SplashClusterFirst | 폭발 중심, 뭉친 적 처리 |

## 팬덤 인지도와 초기 우선순위

이 분류는 정확한 인기투표가 아니라 팬덤 인지도, 디자인 훅, 스토리/밈성, 게임 내 임팩트를 기준으로 한 콘텐츠 우선순위 참고표다. S/A급은 초반 파편, 정예 조우, 보스 후보, 마케팅용 스크린샷에서 먼저 고려하고, B/C급은 조우 다양화, 이벤트, 재료, 확장 파편 후보로 천천히 푼다.

파편 호환성 기준:

- S/A급 중 환상체 정체성이 뚜렷한 후보는 전용 아키타입 파편을 우선 고려한다.
- B급은 테마가 선명하면 아키타입 그룹 전용, 보조 효과가 자연스러우면 범용 파편으로 둔다.
- C급과 도구형 후보는 범용 보조 파편, 이벤트, 정비, 리스크 선택지로 우선 검토한다.
- 같은 환상체라도 기본 파편은 아키타입 그룹 전용, 개화 파편은 더 좁은 특정 아키타입 전용으로 좁힐 수 있다.

S급: 유저가 바로 반응할 가능성이 큰 얼굴마담

| 환상체 | 활용 이유 |
| --- | --- |
| WhiteNight / Plague Doctor | 대표급 기믹, 종교적 이미지, 강한 기억 |
| Nothing There | 바디호러, 인상적인 대사/밈, ALEPH 대표성 |
| Apocalypse Bird | 새 3종 집합 보스급 상징성 |
| Big Bird / Judgement Bird / Punishing Bird | Apocalypse Bird와 연결되고 개별 인지도도 높음 |
| Queen of Hatred | 마법소녀 계열 대표, 캐릭터성과 팬아트 친화성 |
| King of Greed | 마법소녀 계열, 탐욕 테마와 전투화가 쉬움 |
| Knight of Despair | 마법소녀 계열, 보호/절망 테마가 선명함 |
| Mountain of Smiling Bodies | 강한 호러 비주얼, 성장형 괴물로 기억에 남음 |
| Blue Star | 압도적 이미지, 종교/심연/정신 피해 테마 |
| CENSORED | 이름 자체가 강한 밈/공포 장치 |
| The Silent Orchestra | 음악, 페이즈, 시설 붕괴 감각이 강함 |
| Melting Love | 감염/전염 테마가 매우 선명함 |

A급: 팬 선호가 있고 스킬 파편화하기 좋은 후보

| 환상체 | 활용 이유 |
| --- | --- |
| Der Freischutz | 마탄, 관통, 오사 위험. 스킬화가 매우 쉬움 |
| The Funeral of the Dead Butterflies | 총격, 장례, 처형 이미지가 강함 |
| Little Red Riding Hooded Mercenary | 사냥꾼, 표적 추적, 계약 관계가 선명함 |
| Big and Will be Bad Wolf | Little Red와 세트 설계 가능 |
| Laetitia | 귀여움과 위험성의 대비가 좋음 |
| Child of the Galaxy | 감성적이고 보호/의존 테마가 좋음 |
| The Red Shoes | 충동, 폭주, 유혹 테마가 강함 |
| One Sin and Hundreds of Good Deeds | 상징성 높고 WhiteNight와 연결됨 |
| Army in Black | 영웅성과 붕괴의 양면성이 파편화에 적합 |
| The Burrowing Heaven | 시선/관찰 기믹이 독특함 |
| Alriune | 꽃, 향기, 정신 피해 이미지가 좋음 |
| Snow White's Apple | 동화 기반, 덩굴/속박/독 테마 |

B급: 잘 만들면 좋아할 수 있는 중견/숨은 보석

| 환상체 | 활용 이유 |
| --- | --- |
| Scorched Girl | 초반 인지도, 폭발/자폭 테마 명확 |
| Spider Bud | 독, 포획, 매복 테마 |
| Fragment of the Universe | 우주 파동, 정신 충격으로 변환 쉬움 |
| Warm-hearted Woodsman | 심장, 결핍, 흡혈 테마 |
| Forsaken Murderer | 둔기, 분노, 방어 파괴에 적합 |
| Fairy Festival | 회복과 포식의 양면성 |
| The Little Prince | 포자, 감염, 성장 테마 |
| Queen Bee | 벌, 감염, 군체 테마 |
| Dream of a Black Swan | 가족, 오염, 비극성 |
| The Dreaming Current | 물고기, 전류, 돌진 이미지 |
| The Firebird | 화염, 부활, 고속 이동 |
| Yin / Yang | 세트 설계 가능, 균형/공명 테마 |
| Singing Machine | 음악, 희생, 기계 이미지 |
| Schadenfreude | 관찰/시선 조건 기믹 |

C급: 니치하거나 후순위로 두기 좋은 후보

| 환상체 | 활용 이유 |
| --- | --- |
| Old Lady | 감성은 좋지만 전투 파편화가 약함 |
| Meat Lantern | 함정/미끼로는 좋지만 주력 파편으로는 제한적 |
| Parasite Tree | 기믹형, 구현 난이도 대비 인지도 중간 |
| Clouded Monk | 테마는 좋지만 즉각적인 팬 반응은 약한 편 |
| Dimensional Refraction Variant | 기믹형, 설명 부담이 큼 |
| The Naked Nest | 감염형이지만 Melting Love/The Little Prince와 테마가 겹침 |
| 도구형 환상체 | 전투 파편보다 이벤트/정비/리스크 선택지에 적합 |

초기 활용 권장:

- 팬 반응용 대표: WhiteNight, Nothing There, Queen of Hatred, Der Freischutz, The Funeral of the Dead Butterflies.
- 보스/정예용: Apocalypse Bird, Mountain of Smiling Bodies, Blue Star, The Silent Orchestra, Melting Love.
- 초기 파편용: One Sin and Hundreds of Good Deeds, Scorched Girl, Spider Bud, The Red Shoes, Punishing Bird, Alriune.
- 감성/팬서비스용: Laetitia, Child of the Galaxy, Knight of Despair, Little Red Riding Hooded Mercenary, Big and Will be Bad Wolf.

## 현재 스킬 시스템 구현 가능성

현재 RON 스킬 데이터로 자연스럽게 표현 가능한 축:

- 자기 자신, 단일 적, cast target 기준 스킬.
- 단일 적 선택 규칙: 가까운 적, 현재 공격 대상, 체력이 낮은 적.
- 전달 방식: 즉시, 투사체, 영역.
- 투사체: 유도, 고정 방향, 관통, 최대 타격 수, 충돌 필터.
- 영역: 원, 선, 박스, 사각형, 부채꼴, 지속 장판.
- 영역 기준점: 시전 대상, 시전자 중심, 충돌 지점.
- 효과: 피해, 회복, 공명 증감, 스탯 변경, 버프 적용, 추가 공격.
- 기본 버프: 독성 지속 피해, 기절, 빙결, 침묵.
- 단계 조건: 항상 발동, 이전 단계가 피해를 줬을 때, 시전자가 특정 버프를 가진 경우.

현재 부족하거나 별도 확장이 필요한 축:

- 매혹, 정신 지배, 진영 전환.
- 강제 이동, 당기기, 밀치기, 순간이동, 위치 교환.
- 소환, 분신, 부하 생성, 사도 생성, 감염체 생성.
- 전투 중 변신, 페이즈 전환, 외형/스탯 세트 교체.
- 시체 흡수, 사망자 수 기반 성장, 전장 잔해 소비.
- 타겟이 특정 버프를 가졌는지 검사하는 조건.
- 전투 외 결과 직접 변경. 예: 트라우마, 신뢰도, 연구 진행도, 파편 성장도.
- 조건부 `BattleScenario` 이벤트. 예: 특정 체력 이하 웨이브, 특정 지점 도달 시 변형.
- 시선/감시/조작 금지처럼 플레이어 입력이나 관찰 상태를 조건으로 하는 기믹.
- 즉사, 부활, 사망 취소.

구현 가능성 등급:

| 등급 | 의미 |
| --- | --- |
| A | 현재 RON 스킬 데이터만으로 구현 가능 |
| B | 현재 구조로 거의 가능하지만, 버프 추가나 작은 타겟 규칙 추가가 있으면 더 자연스러움 |
| C | 스킬 런타임 효과나 전투 시나리오 확장이 필요 |
| D | 전투 스킬보다 이벤트/노드/장비 정책으로 다루는 편이 자연스러움 |

## 우선 구현 후보

현재 시스템만으로 가장 안전하게 만들 수 있는 첫 묶음:

| 환상체 | 분류 | 원본 핵심 | 파편 핵심 |
| --- | --- | --- | --- |
| One Sin and Hundreds of Good Deeds | A | 참회, 심판, 회복 | 낮은 출력의 심판 + 자기 안정화 |
| Scorched Girl | A | 화염, 폭발, 자폭성 | 작은 불씨, 지연 폭발 |
| Spider Bud | A | 독, 포획, 매복 | 독 표식/지속 피해 |
| Fragment of the Universe | A | 우주 파동, 정신 충격 | 광역 파동 |
| The Red Shoes | A | 유혹, 폭주, 연속 공격 | 충동 버프/추가 공격 |
| Warm-hearted Woodsman | A | 심장 강탈, 결핍 | 체력 흡수 |
| Forsaken Murderer | A | 둔기, 분노, 방어 파괴 | 단일 강타 |
| Punishing Bird | A/B | 작은 공격, 응징 | 연속 쪼기/반격형 |
| Der Freischutz | A | 마탄, 직선 관통, 오사 위험 | 통제된 직선 탄환 |
| The Funeral of the Dead Butterflies | A | 장례, 총격, 처형 | 다단 탄환/저체력 처형 |
| All-Around Helper | A | 청소 로봇, 회전 칼날 | 주변 회전 공격 |
| Alriune | A | 향기, 꽃, 정신 피해 | 향기 장판 |

이 묶음만으로도 안정형 단일 원거리, 폭발형 광역, 독/지속 피해, 평타 강화형 추가 공격, 관통 직선 탄환, 다단 원거리 처형, 근접 주변 광역, 지속 장판 제어를 테스트할 수 있다.

현재 라이브 데이터에서 유지해야 할 대표 ID:

| 환상체 | 원본 스킬 | 파편 스킬 |
| --- | --- | --- |
| One Sin and Hundreds of Good Deeds | `one_sin_penitence` | `fragment_one_sin_penitence` |
| Scorched Girl | `scorched_explosion` | `fragment_scorched_spark` |
| Spider Bud | `spider_bud_poison_stack` | `fragment_spider_bud_red_eyes` |
| The Red Shoes | `red_shoes_berserk` | `fragment_red_shoes_impulse` |
| Der Freischutz | `freischutz_magic_bullet` | `fragment_freischutz_black_round` |
| The Funeral of the Dead Butterflies | `funeral_butterfly_eulogy_volley` | `fragment_funeral_butterfly_eulogy` |
| All-Around Helper | `all_around_helper_grinder_mk4` | `fragment_helper_grinder_trace` |
| Alriune | `alriune_flower_burial` | `fragment_alriune_faint_aroma` |

## 확장 후보

B 그룹:

- `Fairy Festival`: 회복+버프는 가능. 과보호 후 포식 같은 역효과는 조건부 확장이 있으면 자연스럽다.
- `Judgement Bird`: 저체력 처형형은 가능. 죄/저울 스택은 표식/타겟 버프 조건이 필요하다.
- `Big Bird`: 피해, 침묵, 장판은 가능. 매혹/강제 이동은 별도 확장 필요.
- `Little Red Riding Hooded Mercenary`: 지정 표적 선호 공격은 가능. 계약 표적/원수 추적은 전술 목표 확장 필요.
- `The Silent Orchestra`: 악장별 시간차 광역 파동은 가능. 전장 전체 규칙 변화는 확장 필요.

C 그룹:

- `Nothing There`: 흡혈형 베기는 가능하지만 변신/모방/재생 페이즈가 핵심이다.
- `Blue Star`: 흡인/강제 이동과 정신 붕괴가 필요하다.
- `Mountain of Smiling Bodies`: 시체 흡수와 성장 단계가 필요하다.
- `Melting Love`: 감염체 생성과 전염 확산이 필요하다.
- `WhiteNight`: 사도 생성, 부활, 축복 변환이 필요하다.
- `Apocalypse Bird`: 복합 페이즈, 부위/눈/등불/부리 기믹, 특수 보스 조건이 필요하다.

D 그룹:

- 도구형/특수형은 전투 스킬 목록에 억지로 넣지 않는다.
- 정비, 지원, 랜덤 이벤트, 상점, 위험 선택지로 재설계한다.

가장 먼저 추가할 만한 스킬 시스템 확장:

1. 타겟 상태 조건: `IfTargetHasBuff`, `IfPreviousStepAppliedBuff`, `DamageBonusAgainstBuffedTarget`.
2. 위치 제어: `Pull`, `Push`, `Root`, `ForcedMoveToPoint`.

## 장비/기프트 방향

원작 장비는 보통 환상체별 같은 이름의 Weapon, Suit, Gift 세트를 가진다. 이 게임에서는 다음처럼 단순화한다.

- `Weapon`: 공격 방식, 공격 속도, 사거리, 조건부 발동 능력.
- `Armor`: 생존력, 피해 저항, 트라우마 저항, 특정 환상체 대응.
- `Accessory`: 원작 Gift의 감정/기억/특수 조건을 계승하는 슬롯.

원작 Gift를 그대로 장착품으로 옮기기보다 `기프트`, `기억 장식`, `침식 흔적`, `연구 부산물` 중 하나로 역할을 재정의할 수 있다. 직원 신뢰도 시스템과 연결하려면 Accessory/Gift는 단순 스탯보다 기억, 트라우마 저항, 위험 파편 안정화에 쓰는 편이 좋다.
