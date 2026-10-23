# 눈의 여왕 · 스킬 파편 · 눈의 여왕 리워크 전 기록

<!-- wiki-category: history -->
<!-- wiki-archive-of: content/abnormalities/snow-queen/fragment -->

> 2026-10-20 눈의 여왕 냉기·혹한 리워크 직전의 보존 자료. 현재 규칙으로 사용하지 않는다.

[최신 합의](../../../../../abnormalities/snow-queen/fragment.md) · [보존 원문](fragment.source.txt)

# 눈의 여왕 · 스킬 파편

[콘텐츠 위키](../../../../../README.md) · [개요](../../../../../abnormalities/snow-queen/README.md) · [조우](../../../../../abnormalities/snow-queen/encounter.md) · [장비](../../../../../abnormalities/snow-queen/equipment.md) · [아카이브](../../../../../abnormalities/snow-queen/review.md)

> 현재 합의한 설계와 남은 미정 사항. 구현 완료를 뜻하지 않는다.

본문의 공명·침식 공통 정책은 [파편 공통 기준](../../../../../principles/fragments.md#current-corrosion)에서 확인한다. 이관 전 문구와 현재 공통 규칙의 차이는 후속 검토 대상으로 남긴다.


<a id="source-frag-3329-3359"></a>
<!-- wiki-source:frag-3329-3359 begin -->
<a id="frag-필수-항목-12"></a>
## 필수 항목

| 항목 | 값 |
| --- | --- |
| `rarity` | open design |
| `equip_limit` | open design |
| `compatibility` | `Sword` |
| `cast_selection` | 전방 직선 또는 부채꼴 방향, exact open design |
| 역할 | 딜 / 전방 광역 버스트·이동 제어 |

<a id="frag-파편--빙설의-파도-가칭"></a>
## 파편 — 빙설의 파도 (가칭)

공명이 가득 차면 플레이어가 지정한 전방 직선 또는 부채꼴 타일 범위에 강력한 광역 Magic 피해를
가한다.

- 피격된 모든 적에게 이동 속도를 낮추는 `냉기`를 부여한다.
- 공격 중심의 주 대상 한 명에게는 냉기 대신 또는 냉기와 함께 짧은 `빙결`을 부여한다. 빙결은 이동,
  공격과 스킬 사용을 막는 제어 후보이며 exact 허용·금지 행동은 open design이다.
- 이 파편의 직접 피해로 death finalization을 확정한 적은 그 자리에 `빙결 잔영`을 남긴다.
- 빙결 잔영은 가장 가까운 eligible 적을 향해 이동하며 주변 적의 이동 속도를 낮춘다. 일정 시간이
  지나면 산산이 부서져 주변에 광역 Magic 피해를 준다.
- 유물 없는 기본 효과에서는 빙결 잔영의 폭발로 처치한 적은 새로운 잔영을 만들지 않는다.
  다른 잔영과 장비 proc 등 파편의 최초 직접 피해가 아닌 파생 피해도 기본 잔영 생성 source에서
  제외한다. 아래 전용 유물의 냉기 둔화·사망 조건은 별도 확장 규칙이다.
- 빙결 잔영은 저지·HP·피격·직접 명령이 없는 파편 object이며 일반 소환물과 동시 배치 상한에 포함하지
  않는다.
- 시전당 잔영 생성 상한, 이동 속도·탐색 범위·지속 시간·둔화와 폭발 피해는 open design이다.

기본 파편의 직접 처치에 따른 잔영 생성은 유지한다. 파편·무기가 부여한 냉기 둔화 상태의 적이
사망할 때도 잔영을 생성하는 확장은 아래 전용 유물이 제공한다.
<!-- wiki-source:frag-3329-3359 end -->


관련 상태·공통 미정: [환상체 개요](../../../../../abnormalities/snow-queen/README.md).

[이관·원문 보존 기록](../../../../../maintenance/README.md)
