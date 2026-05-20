# 로보토미 환상체 스킬 구현 가능성 분류

이 문서는 `docs/lobotomy_abnormality_content_audit.md`의 환상체 초안 카탈로그를 현재 core 스킬 시스템과 대조해, 지금 바로 구현 가능한 것과 확장이 필요한 것을 분류한다.

목표는 원작 스킬을 1:1 복제하는 것이 아니라, 이 게임의 전투/직원/스킬 파편 구조 안에서 환상체별 핵심 감각을 구현할 수 있는지 판단하는 것이다.

## 현재 스킬 시스템으로 가능한 것

현재 `SkillDef`는 RON 데이터로 작성할 수 있고, 하나의 스킬은 여러 `SkillStepDef`를 가진다.

현재 지원되는 축:

- 타겟: 자기 자신, 단일 적, 아군 범위, 적군 범위.
- 단일 적 선택 규칙: 가장 가까운 적, 현재 타겟, 체력이 가장 낮은 적.
- 범위: 전체, 체비셰프 반경, 선형.
- 전달 방식: 즉시, 투사체, 영역.
- 투사체: 유도 투사체, 고정 방향 투사체, 관통, 최대 타격 수, 아군/적/전체 충돌 필터.
- 영역: 원, 선, 박스, 사각형, 부채꼴.
- 영역 지속: 즉발, 지속 장판, 매 틱, 영역당 1회, 진입 시 1회.
- 영역 기준점: 시전 대상, 투사체/영역 충돌 지점, 시전자 중심.
- 효과: 피해, 회복, 공명 증감, 스탯 변경, 버프 적용, 추가 공격.
- 기본 버프: 독성 지속 피해, 기절, 빙결, 침묵.
- 단계 조건: 항상 발동, 이전 단계가 피해를 줬을 때, 시전자가 특정 버프를 가진 경우.
- 반복: 고정 횟수 반복, 시전자/타겟 버프 스택 기반 반복.
- 장비/아티팩트 트리거: 공격 시, 피격 시, 처치 시, 사망 시, 전투 시작 시, 아군 사망 시.

## 현재 불가능하거나 부족한 것

다음은 현재 RON 스킬 데이터만으로는 정확히 표현하기 어렵다.

- 매혹, 정신 지배, 적/아군 진영 전환.
- 강제 이동, 당기기, 밀치기, 순간이동, 위치 교환.
- 소환, 분신, 부하 생성, 사도 생성, 벌/감염체 생성.
- 전투 중 유닛 변신, 페이즈 전환, 외형/스탯 세트 교체.
- 시체 흡수, 사망자 수 기반 성장, 전장 잔해 소비.
- 특정 타겟 상태를 조건으로 한 피해 증폭. 예: 표식 대상에게만 추가 피해.
- 타겟에게 특정 버프가 있는지 검사하는 조건. 현재는 `IfCasterHasBuff`만 있다.
- 전투 외 결과 직접 변경. 예: 트라우마, 신뢰도, 연구 진행도, 파편 성장도.
- 조건부 BattleScenario 이벤트. 예: 일정 시간 후 브리칭, 특정 체력 이하 웨이브, 특정 지점 도달 시 변형.
- 시선/감시/조작 금지처럼 플레이어 입력 또는 카메라/관찰 상태를 조건으로 하는 기믹.
- 즉사, 부활, 사망 취소. 현재 전투 흐름과 밸런스 정책상 별도 결정 필요.

## 구현 가능성 등급

| 등급 | 의미 |
| --- | --- |
| A | 현재 RON 스킬 데이터만으로 구현 가능 |
| B | 현재 구조로 거의 가능하지만, 버프 추가나 작은 타겟 규칙 추가가 있으면 더 자연스러움 |
| C | 스킬 런타임 효과 확장이 필요 |
| D | 전투 스킬보다 이벤트/노드/장비 정책으로 다루는 편이 자연스러움 |

## 우선 구현 후보 분류

| 환상체 | 분류 | 이유 |
| --- | --- | --- |
| One Sin and Hundreds of Good Deeds | A | 단일 투사체 피해 + 자기 회복은 이미 `one_sin_penitence`와 파편 스킬로 구현 가능 |
| Scorched Girl | A | 단일/범위 마법 피해, 폭발, 지연 폭발 모두 현재 투사체/영역/단계 스킬로 가능 |
| Red Shoes | A | 추가 공격, 공속/이속 버프, 짧은 폭주 효과는 가능 |
| Der Freischutz | A | 직선 고정 투사체, 관통, 아군 포함 충돌 필터가 가능 |
| Spider Bud | A | 독 부여, 매복형 단일 공격, 지속 피해 가능 |
| Fragment of the Universe | A | 광역 파동/정신 피해를 마법 피해 영역으로 표현 가능 |
| Fairy Festival | A/B | 회복+버프는 가능. 과보호 후 포식 같은 역효과는 조건부 확장이 있으면 자연스러움 |
| Punishing Bird | A/B | 연속 공격은 가능. 공격하면 크게 반격하는 정확한 징벌 패턴은 트리거/조건 보강 필요 |
| Big Bird | B/C | 등불 표식/매혹/처형 중 피해·침묵·장판은 가능, 매혹/강제 이동은 불가 |
| Judgement Bird | B | 체력 낮은 적 처형형은 가능. 죄/저울 스택을 쓰려면 표식/타겟 버프 조건이 필요 |
| Queen of Hatred | B/C | 빔/광역/감정 상태 일부 가능. 사랑/증오 상태 전환 AI는 별도 상태 확장 필요 |
| Little Red Riding Hooded Mercenary | B/C | 지정 표적 추격 컨셉은 일부 가능. 특정 원수 추적/계약형 AI는 전술 목표 확장 필요 |
| Mountain of Smiling Bodies | C | 기본 광역 포식은 가능하지만 시체 흡수 성장, 단계별 거대화는 전용 시스템 필요 |
| Melting Love | B/C | 독/감염 장판은 가능. 감염체 생성·확산은 소환/전염 확장 필요 |
| Nothing There | B/C | 강력한 근접/흡혈형 베기는 가능. 변신/단계 전환/모방은 별도 페이즈 시스템 필요 |
| The Silent Orchestra | A/B | 악장별 시간차 광역 파동은 다단계 스킬로 가능. 전장 전체 규칙 변화는 확장 필요 |
| WhiteNight | C | 회복/심판 파동은 가능. 사도 생성, 부활, 축복 누적 변환은 현재 불가 |
| Apocalypse Bird | C | 복합 보스 스킬 일부는 가능하지만 눈/등불/부리 복합 페이즈와 특수 승리 조건은 별도 보스 시스템 필요 |

## 전체 카탈로그 기준 분류

### A. 현재 데이터만으로 우선 구현하기 좋은 환상체

이 그룹은 수치와 VFX만 정하면 바로 RON 스킬로 옮길 수 있다.

| 환상체 | 가능한 표현 |
| --- | --- |
| One Sin and Hundreds of Good Deeds | 단일 심판 투사체, 피해 후 자기 회복 |
| Scorched Girl | 폭발 피해, 화염 장판, 지연 폭발 |
| Old Lady | 고립 대상 마법/정신 피해, 약화 |
| Forsaken Murderer | 근접 강타, 방어 감소 |
| Punishing Bird | 빠른 연속 공격, 반격형 장비 트리거 |
| Fragment of the Universe | 원거리 투사체, 광역 파동 |
| Spider Bud | 독, 단일 포획형 공격 |
| Ppodae | 돌진 대신 빠른 근접 연타로 단순화 |
| The Red Shoes | 추가 공격, 공속/이속 버프 |
| Warm-hearted Woodsman | 단일 피해 + 자기 회복 |
| All-Around Helper | 자기 주변 회전 광역 |
| Laetitia | 지연 투사체/폭발 |
| The Funeral of the Dead Butterflies | 다단 원거리 탄환, 처형형 저체력 타겟 |
| Der Freischutz | 직선 관통 탄환, 아군 오사 가능 |
| Porccubus | 독/쾌락 스택을 독과 스탯 약화로 단순화 |
| The Dreaming Current | 직선 돌진형 투사체/라인 영역으로 단순화 |
| Alriune | 향기 장판, 지속 마법 피해/침묵 |
| Il Pianto della Luna | 광역 파동, 침묵/빙결 대체 CC |
| Clouded Monk | 체력 흡수형 공격 |

### B. 작은 확장으로 자연스러워지는 환상체

현재도 단순화 구현은 가능하지만, 아래 확장을 추가하면 원작 감각이 훨씬 살아난다.

필요한 작은 확장 후보:

- `IfTargetHasBuff` 또는 `IfStepTargetHasBuff`.
- `ApplyMark`를 새 버프로 처리할 수 있도록 버프 레지스트리 데이터화.
- `DamageIfTargetHasBuff` 또는 조건부 `ModifyDamage`.
- `Taunt/Fear/Charm`을 하드 CC로 추가할지 여부 결정.
- `Root` 또는 이동 제한 버프.

| 환상체 | 현재 가능한 표현 | 있으면 좋은 확장 |
| --- | --- | --- |
| Fairy Festival | 회복 + 공격 버프 | 보호받은 대상에게 나중에 리스크 발생 |
| Happy Teddy Bear | 같은 대상 반복 공격 | 특정 대상 집착/반복 대상 추적 |
| Nameless Fetus | 광역 정신 피해 | 자원 요구/울음 누적 이벤트 |
| The Snow Queen | 빙결, 고립 대상 피해 | 결투/구출 목표 |
| Child of the Galaxy | 보호 표식, 회복 | 표식 상실 시 반동 |
| Schadenfreude | 피해/침묵/반격 | 관찰 조건, 시야 조건 |
| Scarecrow Searching for Wisdom | 저체력/낮은 대상 공격 + 회복 | 정신/지식 자원 흡수 |
| Judgement Bird | 저체력 처형형 피해 | 죄 표식/저울 스택 |
| Snow White's Apple | 속박 장판 | 뿌리 생성/확산 |
| Big and Will be Bad Wolf | 추격 돌진형 단일 공격 | 특정 대상 포식/삼킴 |
| Big Bird | 광역 침묵/표식 | 매혹/강제 접근 |
| Dream of a Black Swan | 광역 파동 | 형제 수/피해 공유 |
| The King of Greed | 직선 돌진/관통 | 탐욕 조건/보상 조건 |
| Dimensional Refraction Variant | 위치 왜곡 느낌의 투사체 | 은신/타겟 불가 |
| The Knight of Despair | 보호 버프 + 후속 붕괴 | 축복 대상 사망 반응 |
| The Burrowing Heaven | 시선 표식 피해 | 관찰 실패 처벌 |
| The Queen of Hatred | 빔, 광역 피해 | 감정 상태 전환 AI |
| Little Red Riding Hooded Mercenary | 지정 대상 선호형 공격 | 계약 표적/원수 추적 |
| The Firebird | 화염 돌진/장판 | 부활성 버프 |
| Parasite Tree | 축복 버프 | 축복 누적 후 기생 전환 |

### C. 전용 시스템 확장이 필요한 환상체

이 그룹은 단순 피해/버프만으로 껍데기는 만들 수 있지만, 핵심 재미를 살리려면 새 효과나 전투 시나리오 확장이 필요하다.

| 환상체 | 필요한 확장 |
| --- | --- |
| Plague Doctor | 축복 누적, WhiteNight 전조, 사도화 |
| Army in Black | 아군 피해/사망에 반응하는 집단 붕괴, 희생형 보호자 AI |
| Beauty and the Beast | 저주/변이 상태 전환 |
| Bloodbath | 유혹/자해/흡혈을 전투 상태로 표현할 방법 |
| Meat Lantern | 함정 설치, 적 유인, 매복 위치 |
| Today's Shy Look | 상태가 계속 바뀌는 표정/기분 테이블 |
| Void Dream | 수면 상태. 현재는 기절/침묵으로 대체 가능하지만 정확한 수면은 없음 |
| Grave of Cherry Blossoms | 유혹 장판, 매장/끌어당김 |
| Singing Machine | 희생/기계 투입 같은 전투 외 비용 |
| Queen Bee | 감염 후 일벌 생성 |
| The Naked Nest | 기생/변이/전염 |
| Yin | Yang과의 공명/합체 이벤트 |
| Yang | Yin과의 공명/합체 이벤트 |
| The Little Prince | 포자 감염 후 변이 |
| Nothing There | 변신 페이즈, 모방, 재생 |
| Blue Star | 흡인/강제 이동, 정신 붕괴 |
| CENSORED | 정보 은폐/공포 노출 규칙 |
| Mountain of Smiling Bodies | 시체 흡수, 성장 단계 |
| The Silent Orchestra | 악장별 전장 규칙 변화 |
| Melting Love | 감염체 생성, 전염 확산 |
| WhiteNight | 사도 생성, 부활, 축복 변환 |
| Apocalypse Bird | 복합 페이즈, 부위/눈/등불/부리 기믹, 특수 보스 조건 |

### D. 전투 스킬보다 이벤트/노드/장비 정책이 어울리는 환상체

이 그룹은 전투 유닛으로 억지 구현하기보다, 지원/정비/이벤트/상점/리스크 선택지로 다루는 편이 맞다.

| 환상체 | 추천 처리 |
| --- | --- |
| You're Bald... | 개그성 이벤트 또는 외형/기프트 이벤트 |
| Don't Touch Me | 조작 금지/위험 선택 이벤트 |
| Mirror of Adjustment | 정비 노드 스탯 재배치 이벤트 |
| Old Faith and Promise | 정비 노드 장비 강화/파괴 리스크 |
| We Can Change Anything | 자원 전환/희생 이벤트 |
| You Must Be Happy | 신뢰도/트라우마/행복 리스크 이벤트 |
| Behavior Adjustment | 행동 보정 장비/이벤트 |
| Luminous Bracelet | 회복 리스크 이벤트 |
| Skin Prophecy | 정보/예언 이벤트 |
| The Heart of Aspiration | 고위험 버프 이벤트 |
| Theresia | 정신 회복/부작용 지원 이벤트 |
| Giant Tree Sap | 회복/독성 선택 이벤트 |
| Notes from a Crazed Researcher | 연구 진행도/트라우마 이벤트 |
| Portrait of Another World | 피해 전가 계약 이벤트 |
| Shelter from the 27th of March | 안전지대/방치 이벤트 |
| Backward Clock | 강력한 리셋/시간 되감기 이벤트 |
| Express Train to Hell | 시간표/노드 진행 리스크 이벤트 |
| Flesh Idol | 숭배/희생/광역 버프 이벤트 |

## 콘텐츠 제작 우선순위 제안

1. A 그룹부터 10~15개를 먼저 만든다.
2. 각 환상체마다 `원본 스킬 1개`, `파편 스킬 1개`, `무기/방어구/악세서리 껍데기`만 만든다.
3. B 그룹은 `표식/타겟 버프 조건`이 필요해지는 순간에만 최소 확장한다.
4. C 그룹은 보스/정예 콘텐츠가 실제로 필요해질 때 확장한다.
5. D 그룹은 전투 스킬 목록에 넣지 말고, 게임 흐름 문서의 이벤트/정비/지원 노드 후보로 이동한다.

## 지금 당장 구현 가능한 콘텐츠 묶음

현재 시스템만으로 가장 안전하게 만들 수 있는 첫 묶음:

- One Sin and Hundreds of Good Deeds
- Scorched Girl
- Spider Bud
- Fragment of the Universe
- The Red Shoes
- Warm-hearted Woodsman
- Forsaken Murderer
- Punishing Bird
- Der Freischutz
- The Funeral of the Dead Butterflies
- Laetitia
- Alriune

이 묶음은 현재 스킬 런타임을 크게 건드리지 않고도 서로 다른 전투 감각을 만든다.

- 단일 저격
- 광역 폭발
- 독
- 연속 공격
- 관통 투사체
- 회복/흡혈
- 장판
- 빙결/침묵 계열 CC

## 다음 확장 후보

스킬 시스템을 확장한다면 가장 먼저 필요한 것은 대규모 추상화가 아니라 아래 두 가지다.

1. 타겟 상태 조건:
   - `IfTargetHasBuff`
   - `IfPreviousStepAppliedBuff`
   - `DamageBonusAgainstBuffedTarget`

2. 위치 제어:
   - `Pull`
   - `Push`
   - `Root`
   - `ForcedMoveToPoint`

이 두 축이 생기면 매혹, 포획, 심판 표식, 뿌리, 등불, 마탄 위험성, 포자 감염 같은 환상체 컨셉 대부분을 더 자연스럽게 표현할 수 있다.
