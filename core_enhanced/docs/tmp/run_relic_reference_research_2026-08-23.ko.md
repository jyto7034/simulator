# Run 유물 레퍼런스 조사

> 상태: 임시 조사 자료 / gameplay canonical, 구현 지시 또는 live RON 계약이 아님
>
> 조사일: 2026-08-23

## 목적

신규 `Run 유물` 시스템의 effect vocabulary, combat tag, compatibility와 획득 구조를 정하기 전에
유명 로그라이크·로그라이트 게임의 유사 시스템을 비교한다.

이 문서는 개별 효과나 수치를 우리 게임 정책으로 채택하지 않는다. 레퍼런스에서 반복되는 설계
패턴과 실패 위험을 추출하는 입력 자료이며, 실제 정책은 후속 전용 설계 문서에서 별도로 승인한다.

연결 문서는 다음과 같다.

- [`run_relic_system_design_notes_2026-08-21.ko.md`](run_relic_system_design_notes_2026-08-21.ko.md)
- [환상체 파편·장비 설계 문서](../skills/abnormality_skill_fragments_design_notes.md)
- [`ego_corrosion_system_design_notes_2026-08-19.ko.md`](ego_corrosion_system_design_notes_2026-08-19.ko.md)

## 조사 범위와 용어

엄밀한 장르 분류보다 run 동안 획득한 선택지가 전투 규칙과 빌드를 바꾸는 구조를 기준으로 대상을
선정했다. 따라서 전통적 roguelike뿐 아니라 deckbuilder, action roguelite와 run-based build game도
포함한다.

작품마다 명칭이 다르므로 이 문서에서는 다음을 모두 비교 대상으로 본다.

- Relic / Artifact
- Boon
- Item / Trinket
- Mutation / Affix
- Joker
- 명시적 item synergy

## 비교 관점

각 작품은 다음 질문으로 비교한다.

1. 효과가 어떤 gameplay 대상에 적용되는가.
2. 어떤 사건에 발동하는가.
3. 효과가 수치를 올리는가, 규칙을 교체하는가.
4. 동시에 활성화할 수 있는 수가 제한되는가.
5. 중복 획득은 어떻게 성장하는가.
6. 호환·선행·금지 조합을 어떻게 표현하는가.
7. 현재 build가 획득 후보에 영향을 주는가.
8. 대가, 위험 또는 교체 비용이 있는가.
9. 플레이어가 결과를 얼마나 쉽게 예측할 수 있는가.

## 우선 레퍼런스

### Slay the Spire — Relic

가장 먼저 참고할 유물 체계다.

관찰 사항:

- Common, Uncommon, Rare, Boss, Shop, Event, Special 등 획득 문맥과 희귀도 성격이 분리된다.
- 전투 시작, 턴 시작, 피해, 카드 사용 횟수와 같은 발동 시점이 짧은 규칙 문장으로 드러난다.
- 캐릭터 전용 유물과 범용 유물이 공존한다.
- Boss Relic은 강한 효과와 눈에 보이는 제약 또는 대가를 교환하는 사례가 많다.
- 일부 유물은 전투가 아니라 보상, 상점, 이벤트 또는 덱 관리 규칙을 바꾼다.

우리 프로젝트에 유용한 관점:

- effect family와 acquisition source를 분리하는 기준
- 조건부 발동을 읽을 수 있는 한 문장으로 표현하는 방식
- 강한 유물을 단순 상위 등급이 아니라 대가가 있는 선택으로 만드는 방식
- 파편·무기 전용과 범용 유물을 한 catalog에서 구분하는 방식

주의점:

- 턴 경계가 명확한 카드 게임이므로 실시간 전투에 `turn start/end` 문법을 그대로 옮길 수 없다.
- 대부분 전역 passive이므로 직원별 loadout과 장비 연동 slot은 별도 설계가 필요하다.

자료:

- [Slay the Spire Relics](https://slaythespire.wiki.gg/wiki/Relics)
- [Slay the Spire Relics List](https://slaythespire.wiki.gg/wiki/Relics_List)

### Dead Cells — Mutation과 Affix

태그와 compatibility 설계에 가장 직접적인 레퍼런스다.

관찰 사항:

- Mutation은 run 안에서 제한된 수만 활성화하며 구간 사이에서 선택·교체한다.
- 일반 규칙에서 Mutation 활성 상한은 세 개다.
- 일부 Mutation은 특정 stat 계통과 연결되고, 범용 계통도 별도로 존재한다.
- Affix는 적용 가능한 item class를 제한할 수 있다.
- Affix 생성 조건에 required tag, forbidden tag, forbidden affix, forbidden item이 사용된다.
- 같은 효과라도 무기·스킬·액세서리 중 어디에 붙는지에 따라 후보가 달라진다.

우리 프로젝트에 유용한 관점:

- `required_tags`와 `forbidden_tags`의 분리
- 같은 mutation hook을 점유하는 효과의 금지 조합
- 직원별 최대 3-slot과 교체 정책
- 무기, 파편, 계통, 범용 후보를 loadout에 맞게 거르는 방식

주의점:

- 한 플레이어 캐릭터 중심이므로 여러 직원별 독립 loadout과 roster 전체 상한은 다루지 않는다.
- Affix와 Mutation은 서로 다른 층위이므로 둘을 하나의 우리 유물 category로 무리하게 합치지 않는다.

자료:

- [Dead Cells Affixes](https://deadcells.wiki.gg/wiki/Affixes)
- [Dead Cells Mutations](https://deadcells.wiki.gg/wiki/Mutation)

### Hades — Boon

파편·무기 문법과 계통 효과의 결합을 검토할 때 유용하다.

관찰 사항:

- Attack, Special, Cast, Dash, Call 같은 핵심 행동에는 각자 Boon slot이 있다.
- 같은 핵심 slot에는 여러 핵심 Boon을 동시에 둘 수 없다.
- 각 신은 일관된 효과 family와 상태 효과 정체성을 가진다.
- Legendary Boon은 같은 신의 특정 선행 Boon을 요구한다.
- Duo Boon은 서로 다른 두 신 계통의 선행 조건을 요구하고 두 문법을 결합한다.
- 일부 효과는 기존 행동에 수치를 더하기보다 행동의 전달 방식이나 후속 효과를 교체한다.

우리 프로젝트에 유용한 관점:

- 파편 또는 무기의 특정 mutation slot을 한 효과만 점유하게 하는 방식
- 계통 유물의 prerequisite와 조합 유물
- 서로 다른 tag family가 만났을 때만 등장하는 offer
- 기존 스킬을 폐기하지 않고 행동 slot의 표현만 변형하는 방식

주의점:

- 한 캐릭터가 모든 Boon을 소유하므로 직원별 유물 귀속과는 다르다.
- 신 계통은 서사와 효과 vocabulary가 이미 일치하지만, 우리 combat tag는 콘텐츠 catalog를 근거로
  별도 정의해야 한다.

자료:

- [Hades Boons](https://hades.fandom.com/wiki/Boons)
- [Supergiant Games Hades 업데이트 기록](https://www.supergiantgames.com/blog/6/)

### Risk of Rain 2 — Item

중복 획득, stack 공식과 proc chain을 검토할 때 유용하다.

관찰 사항:

- 대부분의 item은 같은 item을 다시 획득해 stack을 늘릴 수 있다.
- stack으로 증가하는 값은 발동 확률, 피해량, 대상 수, 지속 시간 등 item마다 다르다.
- 첫 획득이 새 문법을 열고 이후 stack은 수치만 올리는 사례가 존재한다.
- item 발동이 다른 item 발동의 입력이 되는 proc chain이 중요한 빌드 축이다.
- 무한 선형 증가 대신 감소 효율, 확률 접근, 별도 상한을 사용하는 사례가 있다.

우리 프로젝트에 유용한 관점:

- 동일 유물 중복을 level로 전환할 때의 성장 family
- `첫 획득: 규칙 개방`, `추가 획득: 제한된 수치 성장`의 분리
- trigger source와 proc 횟수·cooldown·확률의 명시
- stack마다 증가하는 변수를 effect definition에 명시하는 방식

주의점:

- 다수 item을 사실상 슬롯 제한 없이 누적하는 게임이므로 직원별 3-slot에 그대로 적용할 수 없다.
- proc chain을 자유롭게 허용하면 재귀 발동, 성능과 예측 가능성 문제가 커진다.

자료:

- [Risk of Rain 2 Item Stacking](https://riskofrain2.wiki.gg/wiki/Item_Stacking)
- [Risk of Rain 2 Proc Coefficient](https://riskofrain2.wiki.gg/wiki/Proc_Coefficient)

### Monster Train — Artifact

구획된 전장과 전역 전투 규칙을 바꾸는 유물의 레퍼런스다.

관찰 사항:

- Artifact는 run 동안 지속되는 passive 보너스를 제공한다.
- 보관함, 상인, 시험 보상, 이벤트와 주요 보스 뒤 선택 등 획득 source가 나뉜다.
- 일부 획득처는 두 후보 중 하나를 선택하거나 금으로 바꾸는 skip을 제공한다.
- 특정 clan과 연결된 효과와 범용 효과가 공존한다.
- 카드 비용, 배치, 층, 버프, 적 진입 등 전투 규칙의 한 문장을 교체하는 효과가 많다.
- Monster Train 2의 공개 사례에서는 결과를 추론하기 어려운 이동 효과를 특정 복제 Artifact의 적용
  대상에서 명시적으로 제외했다.

우리 프로젝트에 유용한 관점:

- DefenseRoute와 같은 구획 전장에서 작동하는 전역 규칙 변형
- `RelicCache 선택 또는 skip 보상` 후보
- 지나치게 해석하기 어려운 effect family를 compatibility에서 거부하는 방식
- 직원 귀속 효과와 진짜 player-side 전역 효과를 구분할 근거

주의점:

- 카드와 층 단위 전투이므로 실시간 projectile, 저지, 이동 문법은 직접 제공하지 않는다.

자료:

- [Monster Train Artifacts](https://monster-train.fandom.com/wiki/Artifacts)
- [Monster Train 2 Artifact 설계 사례](https://www.pcgamer.com/games/card-games/monster-train-2-echoes-from-the-void-interview/)

### Balatro — Joker

제한된 slot, 효과 실행 순서와 조건부 성장의 레퍼런스다.

관찰 사항:

- Joker는 일반 카드와 별도의 전용 slot에 보관한다.
- 기본 활성 상한이 있어 유지, 판매와 교체 자체가 중요한 선택이 된다.
- 단순 가산, 곱산, 조건부 발동, 경제, 카드 규칙 변경이 같은 시스템 안에 공존한다.
- Joker 배치 순서가 계산 결과에 영향을 주는 사례가 있다.
- 개별 효과는 단순해도 여러 Joker가 서로의 조건과 출력을 증폭한다.

우리 프로젝트에 유용한 관점:

- 직원별 3-slot에서 각 유물이 충분한 정체성을 가져야 한다는 점
- 합연산과 곱연산의 적용 순서 명시
- 조건 충족 여부를 플레이어가 읽을 수 있는 상태 투영
- 유물 유지와 교체의 opportunity cost

주의점:

- 점수 계산 게임의 순서 의존을 실시간 전투 mutation에 그대로 허용하면 결과 설명이 어려워진다.
- 우리 시스템은 기본적으로 명시적 priority 또는 순서 비의존 합성을 우선 검토해야 한다.

자료:

- [Balatro 공식 FAQ](https://www.playbalatro.com/faq)
- [Balatro 공식 사이트](https://www.playbalatro.com/)

### The Binding of Isaac: Rebirth — Item

공격 전달 방식의 과감한 변형과 emergent synergy를 연구할 레퍼런스다.

관찰 사항:

- passive item은 능력치뿐 아니라 projectile의 수, 방향, 관통, 유도, 반사, 분열과 충전 문법을 바꾼다.
- active item은 별도 slot과 충전 조건을 사용한다.
- item pool과 item tag가 획득 가능 장소와 조건에 영향을 준다.
- 여러 item 조합이 원래 효과와 크게 다른 공격 형태를 만들 수 있다.

우리 프로젝트에 유용한 관점:

- `Projectile`, `Trajectory`, `Delivery` 계통 유물의 변형 후보
- 기본 공격을 다른 공격 문법으로 바꾸는 무기 전용 유물
- 단일 효과보다 조합 결과가 빌드 identity가 되는 사례

주의점:

- 조합 수가 매우 많고 예외가 축적되기 쉬워, 우리 프로젝트의 typed hook과 설명 가능성 원칙에는
  경고 사례이기도 하다.
- 개별 item ID에 대한 특수 분기를 복제하면 새로운 legacy subsystem이 될 위험이 있다.

자료:

- [The Binding of Isaac: Rebirth Items](https://bindingofisaacrebirth.wiki.gg/wiki/Items)
- [The Binding of Isaac: Rebirth Item Pool](https://bindingofisaacrebirth.wiki.gg/wiki/Item_Pool)

### Enter the Gungeon — Item Synergy

명시적 조합과 현재 loadout 기반 후보 보정의 레퍼런스다.

관찰 사항:

- 특정 총과 item을 함께 보유하면 이름이 있는 synergy가 활성화된다.
- synergy는 피해 증가뿐 아니라 탄환, 재장전, 폭발, 유도와 무기 외형을 바꿀 수 있다.
- 현재 보유 item과 조합되는 보상 후보의 확률을 보정하는 `Synergy Factor`가 존재한다.
- 활성 synergy의 구성 item을 게임 내 사전에서 확인할 수 있다.

우리 프로젝트에 유용한 관점:

- 현재 파편·무기와 호환되는 유물을 RelicCache에 우선 제시하는 방식
- 전용 조합에 고유 이름과 명시적 compatibility를 부여하는 방식
- 획득하지 않은 상대 item까지 요구하는 무의미한 후보를 줄이는 방식

주의점:

- 효과 설명이 외부 wiki 의존적이라는 사용자 경험 비판도 있으므로, 우리 tooltip은 적용 대상과 변경
  규칙을 자체적으로 설명할 필요가 있다.

자료:

- [Enter the Gungeon Synergies](https://enterthegungeon.fandom.com/wiki/Synergies)

## 보조 레퍼런스

### Rogue Legacy 2 — Relic과 Resolve

- Relic 획득이 `Resolve`를 소비하고 장비 무게가 Resolve 예산에 영향을 준다.
- 유물을 더 얻는 행위가 최대 체력 감소와 연결되어 run 전체 위험 예산이 된다.
- 동일 Relic 중복과 저주를 해소해 보상으로 바꾸는 사례가 있다.

우리 프로젝트에서는 roster 전체 활성 상한 또는 강한 유물의 공통 비용을 검토할 때 참고할 수 있다.
현재 Run 유물 초안에는 이와 같은 전역 예산을 도입하기로 합의하지 않았으므로 open design이다.

자료:

- [Rogue Legacy 2 Relics](https://rogue-legacy-2.fandom.com/wiki/Relics)
- [Rogue Legacy 2 Resolve](https://rogue-legacy-2.fandom.com/wiki/Resolve)

### 추가 후보

아래 작품은 첫 vocabulary를 만든 뒤 필요한 축이 생겼을 때 추가 조사한다.

- `Vampire Survivors` — 무기와 passive 조합을 통한 evolution prerequisite
- `Brotato` — item tag, 캐릭터 고유 규칙과 대량 stack
- `Noita` — spell 구성 순서와 emergent interaction의 극단적 사례
- `Darkest Dungeon` — 캐릭터별 Trinket slot과 명시적 장단점

## 교차 패턴 정리

| 설계 축 | 대표 레퍼런스 | 관찰된 패턴 | 우리 시스템에서 검토할 질문 |
| --- | --- | --- | --- |
| 획득 source | Slay the Spire, Monster Train | 일반, 상점, 이벤트, 보스 후보 분리 | 파편 최초 획득, RelicCache, Shop의 역할을 어떻게 나눌까 |
| 제한 slot | Dead Cells, Hades, Balatro | 적은 slot이 교체 비용과 build identity를 만듦 | 직원별 3-slot 외 roster 상한이 필요한가 |
| tag compatibility | Dead Cells, Isaac | required/forbidden tag와 item pool | tag가 적용 가능성만 뜻할지 runtime 효과까지 뜻할지 |
| 전용 variant | Hades, Gungeon | 특정 행동·무기 조합이 규칙을 교체 | 파편·무기별 authored skill variant를 우선할까 |
| 중복 성장 | Risk of Rain 2, Rogue Legacy 2 | 첫 획득과 추가 stack의 증가 변수가 다름 | copy, level, 추가 mutation 중 무엇을 채택할까 |
| trigger/proc | Slay the Spire, Risk of Rain 2 | 발동 시점, 확률, cooldown, 상한 명시 | 재귀 proc와 한 전투 발동 상한을 어떻게 제한할까 |
| 전역 규칙 | Monster Train, Slay the Spire | 전투·보상·상점 규칙 전체를 바꿈 | 직원 귀속과 player-side 전역 유물을 분리할까 |
| 효과 순서 | Balatro | 계산 순서가 결과와 build 의미를 바꿈 | 명시적 priority와 순서 비의존 합성 중 무엇을 우선할까 |
| emergent synergy | Isaac, Gungeon | 조합이 새로운 공격 문법을 만듦 | 명시적 조합만 허용할지 일반 hook 합성을 허용할지 |
| 위험 예산 | Slay the Spire, Rogue Legacy 2 | 강한 효과가 제약·자원·체력을 요구 | 개별 대가와 roster 공통 예산 중 무엇이 맞을까 |

## 레퍼런스에서 추출되는 effect family 후보

아래는 조사 분류 후보이며 아직 우리 gameplay 정책이 아니다.

### 수치와 피해 계산

- 고정 수치 증가
- 합연산 비율 증가
- 최종 곱연산 증폭
- 특정 damage type 또는 source만 증폭
- 특정 상태·거리·위치·저지 조건에서 증폭
- 방어·저항 관통 또는 피해 감소

### 발동 사건

- 전투 시작
- 공격 시도 또는 적중
- 피격
- 처치
- 아군 사망
- 스킬 시전 또는 종료
- projectile 충돌
- stack 임계치 도달
- 특정 tile 진입·이탈
- 저지 시작·유지·종료

### 대상 선정

- 최근접, 현재 대상, 최저 체력
- 표식 대상 우선
- 수동 지정 허용
- 단일 대상을 다중 대상으로 확장
- 같은 tile 또는 특정 범위 우선
- 지상·공중·저지 대상 필터

### 전달 방식

- Instant와 Projectile 교체
- projectile 수, 속도, 사거리, 수명
- 관통, 최대 타격, 최대 처치
- 유도, 반사, 분열, 재귀 projectile
- projectile 충돌 뒤 area 생성
- tile area의 anchor, tracking, tick 변경

### stack과 장기 상태

- 처치·피격·시전으로 stack 획득
- stack 상한 변경
- stack 소비 또는 보존
- 전투 종료 초기화와 run 유지
- threshold 도달 시 skill variant 전환

### 자원과 대가

- 공명 비용 또는 획득 방식 변경
- 전술 자원 소비·회복
- 사거리·대상 수 감소와 피해 교환
- HP, 트라우마, 유닛 철수와 교환
- slot 또는 roster 공통 예산 점유

### 전투 외 효과

- Shop 가격·후보·refresh
- Reward·RelicCache 후보 수
- map 또는 node 생성
- 치료·회복·경제

전투 외 범용 효과를 직원 장비 연동 slot에 허용할지는 기존 Run 유물 초안과 동일하게 open design이다.

## tag와 mutation hook을 분리해야 하는 이유

레퍼런스 조사에서 `무엇인가`와 `무엇을 바꾸는가`는 별도 축으로 반복된다.

```text
tag           = 현재 파편·스킬·무기 문법의 성질 또는 적용 가능성
effect        = 유물이 실제로 제공하는 결과
mutation hook = effect가 교체하거나 점유하는 규칙 지점
```

예시:

```text
Projectile         tag: 이 스킬이 projectile 전달을 사용한다.
ProjectileDelivery hook: 유물이 projectile 전달 규칙을 교체한다.
PierceIncrease     effect: 최대 관통 수를 올린다.
```

`Projectile` tag 자체를 실행 가능한 효과로 취급하면 같은 tag를 사용하는 모든 콘텐츠가 runtime 내부
필드를 임의로 수정하게 된다. 후속 문서에서는 tag vocabulary와 mutation hook vocabulary를 별도 표로
정의해야 한다.

## 우리 프로젝트에 대한 잠정 결론

정책 결정이 아니라 후속 설계의 출발점으로 다음을 기록한다.

1. 전체 시스템 뼈대는 Slay the Spire의 획득·효과 분류를 우선 참고한다.
2. tag, compatibility와 금지 조합은 Dead Cells를 우선 참고한다.
3. 파편·무기의 규칙 교체와 선행 조합은 Hades를 우선 참고한다.
4. 중복과 level 성장은 Risk of Rain 2를 우선 참고한다.
5. DefenseRoute 전역 규칙은 Monster Train을 우선 참고한다.
6. 제한된 3-slot과 계산 순서는 Balatro를 보조 참고한다.
7. Isaac과 Gungeon은 과감한 변형의 아이디어 원천이면서 조합 폭발의 경고 사례로 사용한다.

현재 Core 스킬 runtime은 다단계 step, 조건·반복, Instant/Projectile/TileArea, trigger와 skill ID 전체
교체를 이미 지원한다. 따라서 레퍼런스 효과를 조사할 때 다음 셋으로 나눠야 한다.

- 기존 skill definition 또는 별도 skill variant로 표현 가능
- 기존 trigger/effect를 Run 유물 source로 일반화하면 표현 가능
- 새 typed mutation hook 또는 domain state가 필요

이 분류 없이 모든 레퍼런스 효과를 범용 DSL로 먼저 구현하지 않는다.

## 피해야 할 복제 방식

- 작품의 유물 이름, 수치와 문장을 그대로 옮기기
- `relic_id`별 runtime 조건문을 전투 커널에 추가하기
- tag를 실행 가능한 mutation hook처럼 사용하기
- 모든 조합을 허용하고 runtime이 적용 순서를 추측하게 하기
- 이해하기 어려운 조합을 tooltip 없이 숨기기
- 슬롯 제한 게임과 무제한 stack 게임의 밸런스 전제를 섞기
- 전투 전역 효과와 직원 귀속 효과를 같은 scope로 취급하기

## 후속 조사·설계 작업

1. 우선 레퍼런스에서 대표 효과를 각 10개 안팎으로 추린다.
2. 각 효과를 발동 시점, 적용 대상, 지속 범위, 중복, 대가와 획득처로 분해한다.
3. 현재 Core skill runtime capability와 대조한다.
4. 반복되는 특성에서 candidate combat tag를 추출한다.
5. tag와 별도로 candidate mutation hook을 추출한다.
6. 같은 hook을 요구하는 효과끼리 합성·배타 사례를 만든다.
7. 실제 파편·무기 catalog에 대입해 사용되지 않는 후보 tag를 제거한다.
8. 결과를 별도 `effect vocabulary / tag taxonomy` 임시 문서로 승격한다.

## 미결정 사항

- 최종 조사 대상과 작품별 표본 수
- 원작 효과를 어느 수준까지 수치 없이 추상화할지
- 전투 외 player-side 전역 유물을 첫 버전에 포함할지
- tag를 definition에 직접 authored할지 validation 단계에서 일부 파생할지
- 동일 유물 중복을 copy, level 또는 추가 mutation으로 처리할지
- explicit synergy와 일반 hook 합성 중 어느 쪽을 우선할지
- 유물 effect 계산 순서와 priority를 플레이어에게 노출할지
- 강한 유물의 공통 위험 예산을 둘지
