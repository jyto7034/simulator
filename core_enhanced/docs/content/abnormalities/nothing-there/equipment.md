# 아무 것도 없는 · 장비

[콘텐츠 위키](../../README.md) · [개요](README.md) · [조우](encounter.md) · [파편](fragment.md) · [아카이브](review.md)

> 현재 합의한 설계와 남은 미정 사항. 구현 완료를 뜻하지 않는다.


<a id="source-frag-2126-2210"></a>
<!-- wiki-source:frag-2126-2210 begin -->
<a id="frag-무기--미미크리"></a>
## 무기 — 미미크리

| 항목 | 값 |
| --- | --- |
| `range_role` | 근접 |
| `weapon_archetype` | `Sword` |
| `deployment_affinity` | 지상 전용 |
| `block_capacity` | `1` |
| `ground_defense_focus` | open design |
| `damage_type` | Physical |
| `targeting_profile` | open design |
| `air_capable` | 불가능 |
| `defense_tile_range` | 일반 `Sword`보다 조금 긴 전방 근거리, exact open design |
| `interval_ms` | open design |
| `windup_ms` | open design |
| `delivery` | 즉시 근접 타격 |

미미크리는 살과 뼈가 거대한 칼날을 이루는 생체 대검이다. 별도 weapon archetype을 만들지 않고
기존 `Sword`로 분류한다. 일반 공격은 큰 생체 칼날을 휘둘러 전방 범위의 적에게 피해를 준다.
정확한 범위와 동시 타격 상한은 open design이다.

**무기 자체 효과 — 흡혈**

미미크리로 적의 HP를 실제로 감소시킨 피해량의 `25%`만큼 착용자의 battle HP를 회복한다. Overkill은
제외하고 다수 대상 공격은 같은 공격에서 실제로 감소시킨 HP를 합산한 뒤 공격당 회복 상한을 한 번
적용한다. 일반 회복이므로 max HP를 넘지 않고 physical damage와 trauma는 회복하지 않는다.

미미크리를 장착한 상태에서 파편이 일반 공격을 다른 형태로 교체하더라도 그 공격이 미미크리의
직접 기본 공격으로 분류되면 흡혈을 적용한다. 위 Hello 관통 기본 공격은 포함하고 Goodbye 마무리는
제외한다. Skill 직접 피해, 지속 피해, 반사, 장비 proc와 환경 피해도 흡혈 source가 아니다.
정확한 공격당 회복 상한과 다른 파편의 eligible replacement attack은 후속 설계에서 확정한다.

<a id="frag-방어구--미미크리"></a>
## 방어구 — 미미크리

높은 방어력과 Physical 피해 감소를 제공하는 생체 방어구다.

**방어구 자체 효과 — 재생**

착용자가 실제 HP 피해를 받은 뒤 authored 지연 시간 동안 새 피해를 받지 않으면 재생을 시작한다.
재생 중에는 온전한 `1초`마다 최대 HP 비례량을 회복하고, 새 실제 HP 피해를 받으면 즉시 중단한 뒤
지연 시간을 처음부터 다시 센다.

피해가 0으로 막힌 공격은 지연 시간을 초기화하지 않는다. 재생은 battle HP만 회복하고 physical
damage·trauma와 사망을 되돌리지 않는다. 방어력, Physical 피해 감소량, 시작 지연과 초당 회복량은
open design이다.

<a id="frag-액세서리--미미크리"></a>
## 액세서리 — 미미크리

매 전투에서 착용 직원 identity당 한 번 치명적인 HP 감소를 사망 확정 전에 가로채는
`death-prevent`를 제공한다. 철수·재배치로 새 runtime instance를 만들어도 같은 전투의 사용 여부는
초기화하지 않는다.

첫 번째 또는 두 번째 누적 부활에서 성공한 효과:

- HP가 `0`이 되는 전이에서 death finalization을 취소한다.
- 같은 runtime instance와 배치 위치를 유지하며 최대 HP의 `50%`로 회복한다.
- `1초` 동안 새 피해에 무적이 된다.
- attempt-local physical damage와 trauma를 추가하지 않는다.
- 전투불능, 철수, 재배치 대기·비용과 장비·파편 회수를 발생시키지 않는다.
- 보유 장비·파편·buff와 이미 흐른 cooldown은 그대로 유지한다. 진행 중 action, target, engagement와
  같은 battle-local 세부 보존 범위는 후속 판정에서 확정한다.

Breakdown collapse와 다른 전투 밖 영구 사망은 이 battle death-prevent의 대상이 아니다.

**숨겨진 기능 — 변태**

성공한 부활 횟수는 액세서리 instance가 아니라 직원 identity별 run-local 누적치다. 액세서리를
벗거나 다른 직원에게 옮겨도 기존 직원의 누적치는 남고, 새 착용자는 자신의 누적치에서 시작한다.
Retreat는 진입 전 상태와 함께 해당 attempt의 부활 누적을 되돌리며 checkpoint load는 저장 시점의
직원별 누적치를 복원한다.

정확한 누적 숫자는 일반 UI에 표시하지 않는다. 대신 첫 번째와 두 번째 부활 뒤 직원의 피부, 눈,
초상화와 음성에 점차 강해지는 이상 징후를 남겨 영구 변태 위험 자체는 예고한다.

세 번째 death-prevent가 발동하면 직원은 정상적으로 부활하지 않는다.

1. 치명 피해와 일반 death finalization을 잠시 보류한다.
2. 직원이 일어난 뒤 `Hello`를 말하고 신체가 붕괴한다.
3. 같은 위치에서 적대적인 아무 것도 없는 변태체로 전환한다.
4. 원래 직원은 run-local 영구 사망하고 변태체는 새로운 승리 필수 적으로 전투에 참가한다.

변태체를 제압해도 직원은 돌아오지 않는다. 이 변태체는 encounter primary, 보상, 스킬 파편,
repeat reward와 영구 도감 등록 source가 아니다. 변태체의 형태·스탯·공격, 전환 중 timing,
원래 직원의 장비·파편 회수와 미미크리 액세서리의 소비 여부는 open design이다.
<!-- wiki-source:frag-2126-2210 end -->


관련 상태·공통 미정: [환상체 개요](README.md).

[이관·원문 보존 기록](../../maintenance/README.md)
