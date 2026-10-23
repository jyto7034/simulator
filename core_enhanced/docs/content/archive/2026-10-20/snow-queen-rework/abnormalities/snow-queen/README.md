# 눈의 여왕 · 눈의 여왕 리워크 전 기록

<!-- wiki-category: history -->
<!-- wiki-archive-of: content/abnormalities/snow-queen/README -->

> 2026-10-20 눈의 여왕 냉기·혹한 리워크 직전의 보존 자료. 현재 규칙으로 사용하지 않는다.

[최신 합의](../../../../../abnormalities/snow-queen/README.md) · [보존 원문](README.source.txt)

# 눈의 여왕

[콘텐츠 위키](../../../../../README.md) · [조우](../../../../../abnormalities/snow-queen/encounter.md) · [파편](../../../../../abnormalities/snow-queen/fragment.md) · [장비](../../../../../abnormalities/snow-queen/equipment.md) · [아카이브](../../../../../abnormalities/snow-queen/review.md)

> 현재 합의한 설계와 남은 미정 사항. 구현 완료를 뜻하지 않는다.

**분류:** 엘리트

높은 기본 스탯으로 Route를 진행하며 근접 범위기·전방 강공격·단일 빙결을 사용하는 정공법 전투

## 문서 안내

| 주제 | 내용 |
| --- | --- |
| [조우](../../../../../abnormalities/snow-queen/encounter.md) | 등장 조건·전투 기믹·대응·보상 |
| [스킬 파편](../../../../../abnormalities/snow-queen/fragment.md) | 직원용 기본형·개화형과 상태·종료 조건 |
| [장비](../../../../../abnormalities/snow-queen/equipment.md) | 무기·방어구·액세서리 |
| [아카이브](../../../../../abnormalities/snow-queen/review.md) | 이전 합의·미채택 검토안 보존 |

각 문서의 상태와 미정 사항은 이관한 원문의 구분을 유지한다.

관련 유물: [겨울의 유언](../../../../../relics/snow-queen-remnant.md).


<a id="source-frag-3318-3328"></a>
<!-- wiki-source:frag-3318-3328 begin -->
<a id="frag-눈의-여왕"></a>
## 눈의 여왕

> 상태: 전방 빙설의 파도와 처치 시 빙결 잔영, Platform 광역 마법 Sword의 냉기, 자기 동결 방어구와
> 이동 방해 대상 피해 증가 액세서리 구조 합의 / 파편·무기의 냉기로 둔화된 적 사망 시 잔영을 만드는
> 눈의 여왕 전용 Run 유물 합의 / 이름·수치·세부 판정 미정

조우 설계는 `abnormality_design_notes.ko.md`의 `눈의 여왕` 절이 소유한다. 조우의 세 행동 중 전방
예고 범위기 `빙설의 파도`를 플레이어 파편의 대표 active skill로 옮기고, `얼음 감옥`은 주 대상 한
명의 짧은 빙결과 방어구의 자기 동결로 나누어 사용한다. 원본 조우에는 없는 빙결 잔영은 리산드라의
얼음 노예처럼 냉기·처치·지연 폭발을 잇는 참고 문법이다.
<!-- wiki-source:frag-3318-3328 end -->


<a id="source-frag-3430-3448"></a>
<!-- wiki-source:frag-3430-3448 begin -->
<a id="frag-전용-조합과-독립성-8"></a>
### 전용 조합과 독립성

- 파편은 다른 `Sword`와 조합해도 해당 무기의 공격력을 참조해 빙설의 파도와 빙결 잔영을 완결한다.
- 서리 마검은 다른 `Sword` 호환 파편과 조합해도 Platform 광역 Magic 공격과 냉기 둔화를 제공한다.
- 눈꽃의 예복은 파편·무기와 무관하게 HP 조건에서 자기 동결 생존기를 한 번 제공한다.
- 얼음 왕관은 전용 냉기뿐 아니라 다른 eligible 이동 방해와 조합해도 조건부 피해 증가를 제공한다.
- 전용 조합은 `마검의 광역 냉기 -> 얼음 왕관의 피해 증가 -> 빙설의 파도 처치 -> 빙결 잔영의 둔화와
  강화된 폭발`로 이어진다. 눈꽃의 예복은 이 공격 순환에 새 자원을 추가하지 않고 생존만 보조한다.

<a id="frag-미결정-사항-15"></a>
### 미결정 사항

- 파편과 장비의 최종 이름, `rarity`, `equip_limit`, 획득 경로와 강화 recipe
- 빙설의 파도 cast selection, 직선·부채꼴 범위, 피해량, 냉기·주 대상 빙결의 지속 시간과 면역 처리
- 빙결 잔영의 시전당 생성 상한, 탐색·이동·target 부재 처리, 둔화 범위·수치와 폭발 시간·피해량
- 냉기 둔화 사망 전용 유물의 희귀도, 냉기 복수 출처·갱신·면역 판정, 생성 상한과 연쇄·중복 처리
- 서리 마검의 공격 범위·타일 형태·피해량·주기, 냉기 둔화율·지속 시간과 공중 target eligibility
- 눈꽃의 예복의 HP 임계치·지속 시간·eligible 피해 source, target·저지와 진행 중 action 처리
- 얼음 왕관의 피해 증가율, eligible 피해 source, movement-impairing 상태·면역 판정과 계산 단계
<!-- wiki-source:frag-3430-3448 end -->


[이관·원문 보존 기록](../../../../../maintenance/README.md)
