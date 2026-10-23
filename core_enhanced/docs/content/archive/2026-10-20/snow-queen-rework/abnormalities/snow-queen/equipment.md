# 눈의 여왕 · 장비 · 눈의 여왕 리워크 전 기록

<!-- wiki-category: history -->
<!-- wiki-archive-of: content/abnormalities/snow-queen/equipment -->

> 2026-10-20 눈의 여왕 냉기·혹한 리워크 직전의 보존 자료. 현재 규칙으로 사용하지 않는다.

[최신 합의](../../../../../abnormalities/snow-queen/equipment.md) · [보존 원문](equipment.source.txt)

# 눈의 여왕 · 장비

[콘텐츠 위키](../../../../../README.md) · [개요](../../../../../abnormalities/snow-queen/README.md) · [조우](../../../../../abnormalities/snow-queen/encounter.md) · [파편](../../../../../abnormalities/snow-queen/fragment.md) · [아카이브](../../../../../abnormalities/snow-queen/review.md)

> 현재 합의한 설계와 남은 미정 사항. 구현 완료를 뜻하지 않는다.


<a id="source-frag-3377-3429"></a>
<!-- wiki-source:frag-3377-3429 begin -->
<a id="frag-무기--서리-마검-가칭"></a>
## 무기 — 서리 마검 (가칭)

| 항목 | 값 |
| --- | --- |
| `range_role` | 원거리 광역 마법 딜러 |
| `weapon_archetype` | `Sword` |
| `deployment_affinity` | Platform 전용 |
| `block_capacity` | `0` |
| `damage_type` | Magic |
| `targeting_profile` | 범위 안의 eligible 적 타일, exact 우선순위 open design |
| `air_capable` | open design |
| `interval_ms` | open design |
| `windup_ms` | open design |
| `delivery` | 원거리 대상 타일에 발생하는 얼음 참격 |

검의 외형과 `Sword` archetype을 유지하지만, 근접 공격 대신 마력으로 검을 휘둘러 대상 타일에 얼음
참격을 발생시키는 Platform 전용 마검이다.

- 참격 범위 안의 모든 eligible 적에게 Magic 피해를 준다.
- 피해를 받은 적에게 `냉기`를 부여해 일정 시간 이동 속도를 낮춘다.
- 냉기는 중첩하지 않으며 다시 부여하면 지속 시간만 갱신한다.
- 공격 범위·타일 형태, 피해량, 둔화율·지속 시간과 지상·공중 target eligibility는 open design이다.

`weapon_archetype`과 `deployment_affinity`는 서로 다른 authored 필드이므로 `Sword`이면서 Platform
전용·저지 `0`인 예외 profile로 취급한다. 파편 자체는 배치 affinity를 바꾸지 않으므로 다른 `Sword`에
장착하면 그 무기가 소유한 배치·저지 규칙을 그대로 따른다.

<a id="frag-방어구--눈꽃의-예복-가칭"></a>
## 방어구 — 눈꽃의 예복 (가칭)

착용자의 HP가 전투 중 처음으로 authored 비율 이하가 되면 `빙정 보호`를 발동한다.

- 짧은 시간 착용자가 얼음 속에 갇혀 eligible 피해를 받지 않는다.
- 빙정 보호 중 기본 공격과 active 파편 시전은 할 수 없다.
- 적의 target으로 남고 배치 타일과 저지를 유지한다. 따라서 지상 무기에 장착해도 저지 중인 적을
  자동으로 통과시키지 않는다.
- 한 전투에 한 번만 발동하며 종료 시 HP 회복, 공격 강화나 재발동 준비를 제공하지 않는다.
- 발동 HP 비율, 지속 시간, 무효화할 피해 source와 이미 시작된 공격·파편 시전 처리는 open design이다.

전용 세트에서는 Platform 마검 사용자가 집중 공격을 받을 때 한 번 버티는 안전장치다. 다른 무기와
조합하면 짧게 공격을 포기하는 대신 전선을 유지하는 독립 방어구로 사용한다.

<a id="frag-액세서리--얼음-왕관-가칭"></a>
## 액세서리 — 얼음 왕관 (가칭)

착용자가 현재 이동 방해 상태인 적에게 가하는 eligible 피해를 증가시킨다.

- 냉기에 의한 이동 속도 감소, 빙결과 속박 등 명시적인 movement-impairing tag가 있는 상태를 대상으로
  인정한다.
- 전용 마검의 기본 공격, 빙설의 파도 직접 피해와 착용자가 소유한 빙결 잔영 폭발에 적용한다.
- 다른 무기·파편이 부여한 eligible 이동 방해 상태에도 사용할 수 있다.
- 이동 방해의 강도·지속 시간이나 빙결 잔영의 생성 수를 직접 늘리지 않는다.
- 피해 증가율, eligible 피해 source, 이동 방해 면역인 적의 상태 인정 여부와 modifier 계산 단계는
  open design이다.
<!-- wiki-source:frag-3377-3429 end -->


관련 상태·공통 미정: [환상체 개요](../../../../../abnormalities/snow-queen/README.md).

[이관·원문 보존 기록](../../../../../maintenance/README.md)
