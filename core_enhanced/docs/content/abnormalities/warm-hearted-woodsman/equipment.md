# 따뜻한 마음의 나무꾼 · 장비

[콘텐츠 위키](../../README.md) · [개요](README.md) · [조우](encounter.md) · [파편](fragment.md) · [아카이브](review.md)

> 현재 합의한 설계와 남은 미정 사항. 구현 완료를 뜻하지 않는다.


<a id="source-frag-2957-3014"></a>
<!-- wiki-source:frag-2957-3014 begin -->
<a id="frag-무기--벌목-가칭"></a>
## 무기 — 벌목 (가칭)

| 항목 | 값 |
| --- | --- |
| `range_role` | 근거리 |
| `weapon_archetype` | `Sword` |
| `deployment_affinity` | 지상 전용 |
| `block_capacity` | `2` |
| `ground_defense_focus` | 딜탱, exact open design |
| `platform_rest` | 해당 없음 |
| `damage_type` | Physical |
| `targeting_profile` | 저지 중인 주 대상 우선 |
| `air_capable` | 불가능 |
| `defense_tile_range` | 전방 근거리, exact open design |
| `interval_ms` | 느림, exact open design |
| `windup_ms` | open design |
| `delivery` | 양손 도끼의 무거운 단일 내려찍기 |

높은 공격력과 느린 공격 주기를 가진 양손 도끼다. 기본 공격에는 심장, 처치 스택이나 별도 발동 효과를
붙이지 않는다. 높은 무기 공격력이 기본 공격과 이 공격력을 참조하는 파편 3타를 함께 강화한다.

저지 `1`의 결투형이 아니라 저지 `2`의 딜탱으로 확정한다. Shield의 저지 `3`과 보호 효과는 갖지
않으며, 순수 탱커보다 낮은 생존력을 높은 직접 화력으로 보완한다.

<a id="frag-방어구--양철-몸-가칭"></a>
## 방어구 — 양철 몸 (가칭)

높은 방어력과 중간 수준 최대 HP를 제공하는 양철 방어구다. 착용자가 active 스킬로 eligible 적의
death finalization을 확정하면 착용자의 battle HP를 즉시 회복한다.

- 일반 공격 처치에는 발동하지 않는다.
- 나무꾼 파편뿐 아니라 다른 active 파편의 eligible 직접 피해로 확정한 처치에도 발동한다.
- 한 스킬 시전으로 여러 적을 처치했을 때 적마다 회복할지 시전당 한 번만 회복할지는 open design이다.
- 지속 피해, 소환물, 환경 피해와 장비 proc 등 스킬에서 파생된 간접 처치의 포함 여부는 exact kill
  source tag로 정한다.
- 회복은 최대 HP를 넘지 않고 persistent physical damage와 trauma는 회복하지 않는다.

처치당 또는 시전당 회복량, 한 시전의 최대 회복량, 기본 방어력·최대 HP와 다른 저항 수치는 후속 수치
설계에서 정한다. 피해 감소 trigger, 반격과 심장 스택은 별도로 두지 않는다.

<a id="frag-액세서리--빌린-심장-가칭"></a>
## 액세서리 — 빌린 심장 (가칭)

착용자가 적에게 가해 실제로 감소시킨 전체 eligible HP 피해량의 `20~30%`만큼 착용자의 battle HP를
회복한다.

- 일반 공격과 active 파편의 직접 피해에는 모두 적용한다. 따라서 나무꾼 파편의 빠른 2타와 강한
  마무리 피해도 각각 흡혈 source가 된다.
- Overkill은 피해량에 포함하지 않는다.
- 다단·다수 대상 피해는 각 대상의 실제 HP 감소량을 합산해 회복량을 계산한다.
- 일반 회복이므로 최대 HP를 넘지 않고 persistent physical damage와 trauma는 회복하지 않는다.
- 자기 피해, 환경 피해와 다른 유닛이 소유한 피해는 흡혈 source가 아니다. 지속 피해·반사·장비 proc
  등 나머지 source를 포함할지는 exact eligibility에서 정한다.
- 회복률의 exact 값, 한 공격 action당 회복 상한과 다른 흡혈 효과의 중첩 규칙은 open design이다.

아무 것도 없는 무기 `미미크리`는 자기 기본 공격의 실제 피해 `25%`만 흡혈하고 Skill 직접 피해를
제외한다. 빌린 심장은 무기 종류와 무관하게 직접 스킬 피해까지 포함하는 액세서리이므로 적용 범위가
더 넓다. 두 효과를 함께 장착했을 때 단순 합산할지, 더 높은 하나만 적용할지와 공통 회복 상한을 둘지는
수치 설계에서 확정한다.
<!-- wiki-source:frag-2957-3014 end -->


관련 상태·공통 미정: [환상체 개요](README.md).

[이관·원문 보존 기록](../../maintenance/README.md)
