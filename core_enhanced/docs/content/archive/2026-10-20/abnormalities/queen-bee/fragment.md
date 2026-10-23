# 여왕벌 · 스킬 파편 · 정리 전 기록

<!-- wiki-category: history -->
<!-- wiki-archive-of: content/abnormalities/queen-bee/fragment -->

> 2026-10-20 정리 전 기록. 합의·제안·미정이 섞여 있던 원문을 보존하며, 현재 규칙으로 사용하지 않는다.

[최신 합의](../../../../abnormalities/queen-bee/fragment.md) · [보존 원문](fragment.source.txt)

# 여왕벌 · 스킬 파편

[콘텐츠 위키](../../../../README.md) · [개요](../../../../abnormalities/queen-bee/README.md) · [조우](../../../../abnormalities/queen-bee/encounter.md) · [장비](../../../../abnormalities/queen-bee/equipment.md) · [검토 기록](../../../../abnormalities/queen-bee/review.md)

> 콘텐츠 설계. 원문의 합의·초안·미정 구분을 유지한다. 구현 완료를 뜻하지 않는다.

본문의 공명·침식 공통 정책은 [파편 공통 기준](../../../../principles/fragments.md#current-corrosion)에서 확인한다. 이관 전 문구와 현재 공통 규칙의 차이는 후속 검토 대상으로 남긴다.


<a id="source-frag-3220-3248"></a>
<!-- wiki-source:frag-3220-3248 begin -->
<a id="frag-필수-항목-11"></a>
## 필수 항목

| 항목 | 값 |
| --- | --- |
| `rarity` | open design |
| `equip_limit` | open design |
| `compatibility` | `Staff` |
| `cast_selection` | eligible 아군 unit |
| 역할 | 힐 / 반응형 단일 유지 회복 |

<a id="frag-파편--일벌의-가호-가칭"></a>
## 파편 — 일벌의 가호 (가칭)

공명이 가득 차면 eligible 아군 한 명을 선택해 일정 수의 일벌이 저장된 `왕실의 벌집`을 부여한다.
벌집의 일벌 사용 조건을 충족할 때마다 일벌 하나를 소비하고 그 대상을 회복한다.

- 일벌 한 마리의 회복량은 대상 최대 HP의 `10~20%`다. exact 비율은 후속 수치 설계에서 정한다.
- 벌집에 저장되는 최초 일벌 수는 open design이다.
- 어떤 사건이 일벌을 소비하는지는 open design이다. 적 공격으로 대상의 실제 battle HP가 감소한 뒤
  반응하는 안을 우선 후보로 두되 아직 확정하지 않는다.
- 일벌 사용 조건을 한 공격 action당 한 번만 인정할지, 내부 cooldown을 둘지와 다단 공격 처리는
  사용 조건과 함께 정한다.
- 회복은 최대 HP를 넘지 않으며 death finalization이 확정된 대상을 부활시키지 않는다.
- 화면에 나타나는 일벌은 회복의 presentation과 delivery일 뿐, 독립 유닛이나 소환물로 취급하지 않는다.
- 벌집의 지속 시간, 모든 일벌을 쓰기 전에 시간이 끝났을 때 남은 일벌의 처리는 open design이다.

증오의 여왕이 공격·회복 광선, 음/양이 지정 위치의 지속 광역 회복이라면 여왕벌은 선택한 아군을
따라가는 제한 횟수 단일 회복을 맡는다. 기본 규칙은 `조건 충족 -> 일벌 1마리 소비 -> 최대 HP 비례
회복`으로 유지하고 별도 감염·전염이나 변이체 생성은 붙이지 않는다.
<!-- wiki-source:frag-3220-3248 end -->


관련 상태·공통 미정: [환상체 개요](../../../../abnormalities/queen-bee/README.md).

<details>
<summary>이관 출처</summary>

- [abnormality_skill_fragments_design_notes.md · 3220–3248행](../../../../../skills/abnormality_skill_fragments_design_notes.pre_wiki_2026-10-18.md) — 파편·장비·공통 미정의 소유 분리

이관 대응표는 [보존 기록](../../../../maintenance/README.md)에서 확인한다.

</details>
