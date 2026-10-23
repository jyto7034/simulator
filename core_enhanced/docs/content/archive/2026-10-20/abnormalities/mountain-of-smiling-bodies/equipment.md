# 웃는 시체의 산 · 장비 · 정리 전 기록

<!-- wiki-category: history -->
<!-- wiki-archive-of: content/abnormalities/mountain-of-smiling-bodies/equipment -->

> 2026-10-20 정리 전 기록. 합의·제안·미정이 섞여 있던 원문을 보존하며, 현재 규칙으로 사용하지 않는다.

[최신 합의](../../../../abnormalities/mountain-of-smiling-bodies/equipment.md) · [보존 원문](equipment.source.txt)

# 웃는 시체의 산 · 장비

[콘텐츠 위키](../../../../README.md) · [개요](../../../../abnormalities/mountain-of-smiling-bodies/README.md) · [조우](../../../../abnormalities/mountain-of-smiling-bodies/encounter.md) · [파편](../../../../abnormalities/mountain-of-smiling-bodies/fragment.md) · [태그·유물 연결](../../../../abnormalities/mountain-of-smiling-bodies/synergies.md) · [검토 기록](../../../../abnormalities/mountain-of-smiling-bodies/review.md)

> 콘텐츠 설계. 원문의 합의·초안·미정 구분을 유지한다. 구현 완료를 뜻하지 않는다.


<a id="source-frag-2336-2436"></a>
<!-- wiki-source:frag-2336-2436 begin -->
<a id="frag-무기--웃음"></a>
## 무기 — 웃음

> 상태: Shield 분류·공격 presentation·run-local 막타 성장 구조 합의 / 이름 외 수치·세부 판정 미정

| 항목 | 값 |
| --- | --- |
| `range_role` | 근거리 |
| `weapon_archetype` | `Shield` |
| `deployment_affinity` | 지상 전용 |
| `block_capacity` | `3` 후보 |
| `ground_defense_focus` | open design |
| `platform_rest` | 해당 없음 |
| `damage_type` | Physical 우선, exact open design |
| `targeting_profile` | 저지 중인 주 대상 우선 |
| `air_capable` | 불가능 |
| `defense_tile_range` | 전방 근거리와 좁은 주변 범위, exact open design |
| `interval_ms` | 느림, exact open design |
| `windup_ms` | open design |
| `delivery` | 거대한 망치의 직접 내려찍기와 주변 충격파 |

외형은 웃는 얼굴과 입이 붙은 거대한 망치지만 gameplay archetype은 `Shield`다. 기본 공격은 주 대상을
망치로 내려찍고 주변 적에게 더 약한 충격파 피해를 준다. 느린 공격 주기와 막타 timing을 읽을 수
있는 분명한 windup을 가지며, 기본 저지와 방어 성능은 스택이 없는 획득 직후에도 탱커로 기능해야
한다.

**무기 소유 run-local 스택 — 시체 질량 (가칭)**

- `웃음`의 eligible 직접 기본 공격으로 주 대상의 death finalization을 확정하면 `시체 질량`을 얻는다.
- 일반 기본 공격과 `웃음`의 authored replacement attack은 포함한다. 파편 독립 피해, 지속 피해,
  반사, 유물·방어구 proc와 환경 피해는 포함하지 않는다.
- 주변 충격파로 죽은 부 대상은 시체 질량을 주지 않는다. 한 공격 action에서 성장 막타 후보는
  확정한 주 대상 하나다.
- 시체 질량은 직원이나 장착한 파편이 아니라 **개별 무기 인스턴스**가 소유한다. 무기를 다른 직원에게
  옮겨도 유지하며, 같은 무기를 여러 개 보유하면 각 인스턴스가 별도 값을 가진다.
- 전투와 node 사이, Maintenance와 checkpoint save/load를 넘어 해당 run 동안 유지한다. Retreat는
  진입 전 savepoint의 무기 스택으로 복원하고 run 종료 시 초기화한다.
- 같은 무기 인스턴스를 강화해도 스택을 보존한다. 합성·교체·파괴처럼 instance identity가 바뀌는
  operation의 승계 여부는 장비 lifecycle 정책에서 확정한다.
- gameplay 상한 없이 선형으로 누적하는 방향을 우선한다. Endless의 장기 누적을 위한 정수 포화와
  최종 피해 safety clamp는 구현 계약으로 별도 보장한다.

일반 적은 막타당 `+1`을 기본 후보로 둔다. 엘리트·환상체 막타에 더 큰 값을 줄지와 무기 획득 이후
남은 node 수를 고려한 성장 속도는 수치 설계에서 정한다. 보상·Wave authority가 없는 임시 소환물,
non-kill despawn과 반복 생성 object는 시체 질량 source에서 제외한다.

**스택 비례 피해**

시체 질량은 퍼센트 배율이 아니라 기본 공격에 고정 피해를 선형으로 더한다.

```text
주 대상 피해 = 기본 주 대상 피해 + 시체 질량 x 스택당 추가 피해
주변 충격파 피해 = 기본 충격파 피해 + 시체 질량 x 스택당 추가 피해 x 주변 계수
```

주 대상은 스택 피해를 온전히 받고 주변 계수는 첫 prototype에서 `30~50%`를 검토한다. 모든 주변
대상에게 스택 피해를 100% 적용하지 않아 광역 막타와 성장 속도가 서로 폭증하지 않게 하며, 정확한
기본 피해·스택당 피해와 주변 계수는 run 길이 및 평균 막타 수를 기준으로 정한다.

2026-10-20 리워크로 파편의 웃는 얼굴과 자동 처형은 대체되었다. 현재 광역 도발은 적을 모으지만
직접 피해를 주거나 무기 스택을 생성하지 않는다. 실제 주 대상 기본 공격 막타라는 성장 조건은 유지한다.

**합의한 유물 연결 — 스택획득**

스택획득을 적용하면 유효한 막타에서 얻는 시체 질량이 2배가 된다. 기본 획득량이 `q`이면 `2q`를
장착 무기 인스턴스에 더한다. 기본 +1 후보를 채택하면 +2가 되며, 엘리트·환상체 기본 획득량은
미정이다. 이미 가진 질량이나 스택당 피해를 다시 증폭하지 않고, 무기 이전·강화로 재획득 처리하지 않는다.

[응징의 표식](../../../../relics/taunted-enemy-basic-attack-boost.md)는 도발 조건을 만족하는
주 대상 타격과 기본 공격 충격파의 피해를 높인다. 충격파로 죽은 부 대상은 여전히 시체 질량을 주지 않는다.
유물 5개의 구분은 [연결표](../../../../abnormalities/mountain-of-smiling-bodies/synergies.md)를 따른다.

<a id="frag-방어구--웃음"></a>
## 방어구 — 웃음

> 상태: 시체 포식 회복 방향 합의 / 수치·시체 소유권 세부 판정 미정

착용자가 저지하던 eligible 적이 사망하면 그 시체를 먹어 실제 battle HP를 회복한다.

- 시체 하나당 최대 HP의 `10%` 회복을 첫 후보로 둔다.
- 현재 HP가 최대라면 시체를 소비하지 않는다.
- 한 시체는 한 번만 소비할 수 있으며 다른 시체 흡수 장비와 중복 보상을 만들지 않는다.
- 일반 회복이므로 max HP를 넘지 않고 physical damage와 trauma를 회복하지 않는다.
- 다수의 적이 같은 resolution에서 죽을 때의 소비 수와 짧은 내부 cooldown은 open design이다.
- 임시 소환물, non-kill despawn과 시체를 남기지 않는 authored 적은 대상이 아니다.

녹아내리는 사랑 무기는 감염된 시체로 별도 고기방패를 수복하지만, 이 방어구는 감염 조건 없이
착용자의 실제 HP만 회복하고 추가 저지·고기방패·고기벽을 만들지 않는다. 같은 시체에 여러 소비자가
경합할 때의 deterministic ownership은 공통 corpse lifecycle 정책에서 정한다.

<a id="frag-액세서리--웃는-가면-가칭"></a>
## 액세서리 — 웃는 가면 (가칭)

> 상태: 직접 공격으로 부패를 쌓아 방어력을 낮추는 구조 합의 / 수치·중첩 ownership 세부 판정 미정

시체가 썩어 전장을 오염시키는 특징을 `부패` 디버프로 옮긴 공격형 액세서리다.

- 착용자의 eligible 직접 공격이 적중한 대상에게 `부패` 1스택을 부여한다.
- 같은 공격 행동은 같은 대상에게 최대 1스택만 부여한다. 다단 공격 한 번으로 여러 스택을 얻지
  않으며, 광역 공격은 실제로 적중한 서로 다른 각 대상에게 1스택씩 부여할 수 있다.
- 지속 피해, 반사, 장비 proc와 환경 피해는 부패를 부여하지 않는다. 기본 공격을 교체하는 authored
  replacement attack과 스킬 직접 공격을 포함할지는 exact source tag로 정한다.
- 부패는 대상별로 보유하고 새 스택을 부여하면 지속 시간을 갱신한다.
- 1스택마다 대상의 방어력이 감소한다. 감소한 방어력은 액세서리 착용자뿐 아니라 모든 아군의 공격에
  적용된다.
- 첫 prototype 후보는 최대 `5`스택, 지속 시간 `6초`, 1스택당 방어력 `5%` 감소다. exact 상한,
  지속 시간과 방어력 계산 단계는 수치 설계에서 확정한다.
- 2026-10-20 파편 리워크 후에도 부패는 액세서리의 독립 상태로 유지한다. 파편은 웃는 얼굴을
  부여하지 않으며 광역 도발 시 부패의 수량과 지속 시간을 바꾸지 않는다.

무기 시체 질량의 획득량이나 스택당 피해를 직접 늘리지 않는다. 따라서 액세서리를 다른 loadout에서
사용해도 아군 전체의 물리 화력을 지원할 수 있다. `웃음`의 run-local 성장은 유효한 주 대상
기본 공격 막타에서만 발생하며, 획득량 2배는 별도 두 겹의 수확의 효과다.
<!-- wiki-source:frag-2336-2436 end -->


관련 상태·공통 미정: [환상체 개요](../../../../abnormalities/mountain-of-smiling-bodies/README.md).

<details>
<summary>이관 출처</summary>

- [abnormality_skill_fragments_design_notes.md · 2336–2436행](../../../../../skills/abnormality_skill_fragments_design_notes.pre_wiki_2026-10-18.md) — 파편·장비·공통 미정의 소유 분리

이관 대응표는 [보존 기록](../../../../maintenance/README.md)에서 확인한다.

</details>
