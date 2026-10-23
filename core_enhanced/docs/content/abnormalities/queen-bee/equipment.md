# 여왕벌 · 장비

[콘텐츠 위키](../../README.md) · [개요](README.md) · [조우](encounter.md) · [파편](fragment.md) · [아카이브](review.md)

> 현재 합의한 설계와 남은 미정 사항. 구현 완료를 뜻하지 않는다.


<a id="source-frag-3249-3296"></a>
<!-- wiki-source:frag-3249-3296 begin -->
<a id="frag-무기--여왕의-지휘봉-가칭"></a>
## 무기 — 여왕의 지휘봉 (가칭)

| 항목 | 값 |
| --- | --- |
| `range_role` | 원거리 순수 힐러 |
| `weapon_archetype` | `Staff` |
| `deployment_affinity` | Platform 전용 |
| `block_capacity` | `0` |
| `damage_type` | 해당 없음 |
| `targeting_profile` | 회복 범위 안의 복수 부상 아군, 우선순위·대상 수 open design |
| `air_capable` | 공격하지 않음. 지상·Platform 아군의 회복 eligibility open design |
| `interval_ms` | open design |
| `windup_ms` | open design |
| `delivery` | 대상마다 날아가는 일벌 회복 projectile |

일반 공격을 적 공격이 아니라 복수 아군을 동시에 치료하는 기본 회복 행동으로 교체하는 Staff다.

- 한 행동에서 선택한 각 아군에게 일벌 하나를 보내 각각 회복시킨다.
- 같은 아군에게 한 행동의 일벌 여러 마리를 중복시키지 않는다. 부상 아군이 목표 수보다 적어도 남은
  일벌을 한 대상에게 몰아 단일 회복량을 높이지 않는다.
- 단일 대상 회복량은 단일 대상 전담 Staff보다 낮게 두고, 여러 전선의 잔부상을 함께 관리하는 무기로
  차별화한다.
- 일벌은 회복 projectile과 presentation일 뿐 독립 유닛이나 소환물로 취급하지 않는다.
- 정확한 대상 수, 대상 우선순위, 대상별 회복량과 행동 주기는 후속 수치 설계에서 정한다.

<a id="frag-방어구--벌집-외피-가칭"></a>
## 방어구 — 벌집 외피 (가칭)

전투 시작 시 착용자에게 착용자 최대 HP에 비례한 `벌집 장벽`을 한 번 부여한다.

- 장벽은 착용자가 받을 eligible 피해를 battle HP보다 먼저 흡수한다.
- 장벽이 모두 소진되어 파괴되면 해당 전투에서 자동으로 재생하지 않는다.
- 다른 아군을 보호하거나 파편의 일벌 수·회복량을 직접 늘리지 않는다.
- 장벽량, 지속 시간, 피해 type별 적용 여부와 다른 barrier와의 우선순위는 open design이다.

전용 무기나 파편이 없어도 전투 초반의 생존을 보조하므로 후방 힐러와 Platform 딜러가 독립적으로
사용할 수 있다.

<a id="frag-액세서리--로열-젤리-가칭"></a>
## 액세서리 — 로열 젤리 (가칭)

착용자가 다른 아군의 battle HP를 실제로 회복하면 그 대상에게 일정 시간 공격 속도 증가를 부여한다.

- 같은 대상에게 여러 번 발동해도 수치를 중첩하지 않고 지속 시간만 갱신한다.
- 실제 회복량이 `0`인 Overheal에는 발동하지 않는다.
- 전용 Staff의 한 행동으로 여러 아군을 회복하면 실제 회복된 대상마다 각각 부여한다.
- 여왕벌 파편의 일벌 회복에도 적용한다. 다른 무기·파편의 eligible 아군 회복에도 발동한다.
- 자기 회복, 흡혈, 최대 HP 증가, barrier 부여와 다른 유닛이 소유한 회복에는 적용하지 않는다.
- 공격 속도 증가량, 지속 시간과 eligible 회복 source는 open design이다.
<!-- wiki-source:frag-3249-3296 end -->


관련 상태·공통 미정: [환상체 개요](README.md).

[이관·원문 보존 기록](../../maintenance/README.md)
