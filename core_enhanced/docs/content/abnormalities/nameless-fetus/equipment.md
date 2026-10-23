# 이름이 없는 태아 · 장비

[콘텐츠 위키](../../README.md) · [개요](README.md) · [조우](encounter.md) · [파편](fragment.md) · [태그·유물](synergies.md) · [아카이브](review.md)

> 현재 합의한 설계와 남은 미정 사항. 구현 완료를 뜻하지 않는다.


<a id="source-frag-3592-3670"></a>
<!-- wiki-source:frag-3592-3670 begin -->
<a id="frag-무기--울대"></a>
## 무기 — 울대

| 항목 | 값 |
| --- | --- |
| `range_role` | 장거리 / 좁은 직선 다수 공격 |
| `weapon_archetype` | `Gun` |
| `deployment_affinity` | Platform 전용 |
| `block_capacity` | `0` |
| `damage_type` | Magic |
| `targeting_profile` | 공격 범위 안에서 route progress가 가장 높은 eligible 적 우선, exact tie-break open design |
| `air_capable` | 가능 초안 |
| `interval_ms` | 매우 빠름, exact open design |
| `windup_ms` | 짧음, exact open design |
| `delivery` | 시전자에서 주 대상 방향으로 빠르게 퍼지는 좁은 직선 음파 |

원작 E.G.O의 `낮은 단발 위력 / 매우 빠른 속도 / 긴 사거리`를 유지하는 Platform 원거리 무기다. 손에
장착한 태아의 입이 주 대상을 향해 울음을 발사한다.

- 기본 공격은 시전자와 주 대상 타일을 잇는 좁은 선을 따라 진행하며, 선 위의 eligible 적에게 각각
  한 번 Magic 피해를 준다.
- 단일 대상 피해는 느린 Staff·Bow·Gun보다 낮게 두고, 빠른 주기와 일렬로 겹친 웨이브 공격으로
  총 화력을 만든다.
- 같은 음파가 큰 collision body나 겹친 타일 때문에 동일한 적을 여러 번 공격하지 않는다.
- 직선 폭, 최대 관통 대상 수, 주 대상 뒤로 남는 사거리, 대상별 피해 감쇠 여부와 projectile 또는
  즉시 line 판정 중 어느 delivery를 쓸지는 open design이다.
- 파편을 장착하지 않아도 빠른 장거리 선형 Magic 무기로 완결한다. 기본 공격에 박동, 룰렛,
  인신공양이나 별도 처치 스택을 붙이지 않는다.

<a id="frag-방어구--양막의-수의-가칭"></a>
## 방어구 — 양막의 수의 (가칭)

높은 Magic 저항과 낮은 Physical 방어를 가진 편향형 방어구다. 원작 `울대` 방어구가 정신 피해에는
강하지만 물리적 손상에는 취약한 구조를 현재 피해 타입으로 번역한다.

- Magic 피해에 대한 생존 stat을 같은 등급의 범용 방어구보다 높게 제공한다.
- Physical 방어와 최대 HP는 같은 등급의 범용 방어구보다 낮게 둔다. True 피해에는 별도 저항을
  제공하지 않는다.
- 피격 시 박동을 얻거나 울음을 발사하는 trigger는 없다. 반복 피격으로 저항이 중첩되거나 장벽이
  재생하지도 않는다.
- 전용 무기·파편 없이도 Magic 중심 encounter를 대비하는 독립 장비로 사용한다.

정확한 방어력·Magic 저항·최대 HP 배분은 certified power와 같은 등급 방어구의 effective health를
함께 비교해 정한다. 특정 피해 타입을 완전 면역으로 만들지는 않는다.

<a id="frag-액세서리--울대-수신기-가칭"></a>
## 액세서리 — 울대 수신기 (가칭)

태아의 울음을 계속 들려주는 크고 끈적한 헤드셋이다. 착용 직원은 전투 중 액세서리와 상호작용해
자신의 최대 HP를 어느 단계까지 희생할지 선택하고, 선택한 단계에 비례한 regular 기본 공격 속도
증가를 얻는다.

| 단계 | 최대 HP 희생 | 기본 공격 속도 증가 |
| --- | --- | --- |
| `0` | 없음 | 없음 |
| `1` | 낮음, exact open design | 낮음, exact open design |
| `2` | 중간, exact open design | 중간, exact open design |
| `3` | 높음, exact open design | 높음, exact open design |

- 상호작용할 때 현재 단계보다 높은 authored 단계 하나를 선택한다. `0 -> 3`처럼 중간 단계를
  건너뛰어 한 번에 올릴 수 있다.
- 단계는 해당 전투 동안 같은 직원 identity에 남는다. 한 번 올린 단계는 전투 중 낮추거나 취소할 수
  없고, 철수·재배치와 runtime instance 교체로 초기화하지 않는다.
- 최대 HP 희생량은 battle entry에서 장비와 다른 persistent stat을 합성한 기준 최대 HP에 선택 단계의
  authored 비율을 적용해 계산한다. 단계 `1 -> 3` 전환은 단계별 희생을 누적하지 않고 기준 최대 HP에
  대한 최종 3단계 희생량으로 다시 계산한다.
- 단계 상승으로 새 최대 HP가 현재 HP보다 낮아지면 공통 MaxHealth 변경 규칙에 따라 현재 HP를 새
  최대치로 clamp한다. 이 감소는 회복할 수 없고, 단계 상승 command가 death finalization을 일으키는
  단계는 선택할 수 없다.
- 공격 속도 증가는 regular 무기 기본 공격에만 적용한다. Active 파편, 장비 proc, persistent area,
  Field Work, Platform Rest, 전술 포인트 회복과 재배치 대기는 가속하지 않는다.
- 단계 선택은 공격 중에도 가능하지만 이미 시작한 windup을 즉시 완료하거나 release 시점을 소급하지
  않는다. 상승 뒤 새로 시작하는 regular 기본 공격부터 새 공격 속도를 사용한다.
- 전투 종료 시 단계와 최대 HP 희생을 폐기한다. 다음 전투는 다시 0단계에서 시작하며, battle Retreat는
  공통 rollback에 따라 상호작용 전 상태를 복원한다.
- 원작 기프트의 이동 속도와 WHITE 피해 감소는 이 액세서리에 함께 넣지 않는다. 전자는 player unit이
  자동 보행하지 않는 현재 전투와 맞지 않고, 후자는 Magic 방어 특화인 양막의 수의가 담당한다.

플레이어는 안전한 낮은 단계로 지속 화력을 얻거나, 위험한 순간 높은 단계를 선택해 생존 여유를 직접
포기하고 공격 빈도를 끌어올린다. 단계 상승을 회복 수단처럼 악용하지 않도록 현재 HP는 비율로
보존하지 않고 새 최대 HP에 단순 clamp하며, 잃은 최대 HP와 현재 HP를 전투 중 되돌려주지 않는다.
<!-- wiki-source:frag-3592-3670 end -->


관련 상태·공통 미정: [환상체 개요](README.md).
유물 적용과 태그 목록: [태그·유물 연결](synergies.md).

[이관·원문 보존 기록](../../maintenance/README.md)
