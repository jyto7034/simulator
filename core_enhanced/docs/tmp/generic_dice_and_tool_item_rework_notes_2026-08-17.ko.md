# 범용 주사위 판정과 Tool item 재개편 메모

> 상태: 임시 정책 메모 / 구현 지시나 canonical 계약이 아님
>
> 작성일: 2026-08-17

## 목적

현재 Event 직원 판정에만 있는 BG3 유사 1d20 판정을 범용 시스템으로 확장한다. 첫 적용 대상은
거목수액 item 사용이다. item을 사용한 순간 Core가 결과를 확정하고, 플레이어가 주사위와 성공·실패를
보는 경험을 제공한다.

범용화의 목적은 모든 확률을 하나의 불투명 random helper로 합치는 것이 아니다. 각 gameplay 규칙이
명시적인 판정 규칙, stable identity, receipt 및 성공/실패 effect를 가지게 하는 것이다.

## 현재 Event D20 계약

현재 Event 판정은 다음을 이미 보장한다.

- Core만 node-seeded raw 1d20을 한 번 확정한다.
- Natural 1은 critical failure, natural 20은 critical success다.
- 선택지 DC, 직원 aptitude `+3`, 장착 장비 최고 하나의 `+1/+2`가 ordinary face를 판정한다.
- valid first resolve만 결과를 commit한다. invalid request는 RNG나 Event state를 바꾸지 않는다.
- committed result는 exact duplicate, reconnect, checkpoint와 retreat 뒤에도 재굴림되지 않는다.

이 Event 규칙을 제거하거나 Tool item에 그대로 강제하지 않는다. Event는 DC/modifier/critical을 가진
한 종류의 generic dice rule consumer로 남긴다.

## 범용 D20의 최소 계약

범용 주사위는 `dice = d20`이라는 UI 모양만 공유하고, 모든 판정에 Natural 1/20의 Event critical 규칙을
자동 적용하지 않는다. 각 rule이 다음을 명시한다.

- 판정 source와 stable request identity
- 굴릴 수 있는 상태와 invalid rejection 조건
- raw die가 속한 결과 면 집합 또는 성공/실패 임계 규칙
- 필요하다면 modifier와 critical policy
- 결과 receipt에 기록할 provenance
- 성공/실패 각각의 authoritative state mutation

첫 유효 요청만 die를 확정하고 receipt와 effect를 하나의 Core transaction으로 commit한다. 동일한 request
identity의 duplicate는 receipt를 재투영할 수 있어도 새 die를 굴리거나 effect를 다시 적용하지 않는다.
reconnect와 checkpoint restore는 committed receipt를 보존한다.

각 roll은 run seed와 gameplay source의 stable identity에서 결정적으로 파생한다. target 교체, Unity 재표시,
transport retry 또는 invalid request가 결과를 바꾸거나 RNG를 소모해서는 안 된다. exact seed layout과
sequence field는 구현 Goal의 조사에서 정하되, Event가 현재 보장하는 choice/performer 독립성과
checkpoint 재현성은 회귀시키지 않는다.

## 거목수액 consumer

### run-local state

거목수액은 profile이나 현실 날짜에 의존하지 않는 run-local ledger를 가진다.

```text
GiantSapRunState {
  total_resolved_uses,
  uses_since_last_backfire,
  committed_use_receipts,
}
```

- `total_resolved_uses`는 valid use가 결과와 함께 commit될 때만 증가한다.
- `uses_since_last_backfire`는 다음 실패 확률을 정한다. 첫 run에서는 0이고, 성공 뒤에는 증가하며,
  실패 뒤에는 1로 reset되어 다음 use의 확률이 15%가 된다.
- ledger는 모든 거목수액 item과 모든 직원이 공유한다.
- run state이므로 RunCheckpoint와 active-run export/import가 같은 ledger를 보존한다. checkpoint 전의
  같은 유효 use를 다시 시도한다면 같은 request identity가 같은 die/result를 내어 save-scumming용
  재굴림이 되지 않아야 한다.

### 실패 확률과 D20 면

| 다음 사용의 `uses_since_last_backfire` | 실패 확률 | 실패 면 |
| ---: | ---: | --- |
| 0 | 0% | 없음 — roll 없이 성공 |
| 1 | 15% | 1..=3 |
| 2 | 30% | 1..=6 |
| 3 | 45% | 1..=9 |
| 4 이상 | 60% | 1..=12 |

이 rule의 natural 1은 실패 면에 포함될 때만 실패이고, natural 20은 별도 critical success가 아니다. 그래야
사용자가 승인한 첫 사용 0%와 정확한 15% 단위 위험이 보존된다. Event consumer만 자신의 explicit
critical policy에 따라 natural 1/20을 해석한다.

### atomic result

유효한 item use는 다음을 하나의 state transition으로 처리한다.

1. item ownership, 대상 직원, run state와 request identity를 검증한다.
2. 현재 위험 단계에서 D20 결과를 확정하고 receipt를 만든다.
3. item 하나를 소비한다.
4. 성공이면 `N`회의 전투 동안 `max HP +20~30%`, 실패면 `N`회의 전투 동안 `max HP -20~40%` modifier를
   대상 직원에게 적용한다.
5. 위험 ledger, duration state, inventory, receipt 및 player-visible result를 함께 commit한다.

어느 단계도 실패하면 die, item, duration, ledger, receipt 모두 바뀌지 않는다. 결과 UI는 Core receipt를
표시할 뿐 성공·실패, 수치 또는 잔여 기간을 로컬에서 계산하지 않는다.

## 범위와 후속 결정

이 메모가 확정하지 않는 사항:

- `N`의 정확한 전투 수, 성공/실패 % 범위 안의 exact 값·분포
- 기간을 감소시키는 정확한 전투 경계와 같은 직원 재사용 시 replace/refresh/stack 정책
- 범용 dice DTO/wire 명칭, Unity의 animation/skip UX 및 full roll history 표시 범위
- Event 외의 다른 item, 장비, encounter가 범용 dice를 사용할지 여부
- advantage/disadvantage, reroll, bonus die, karmic/streak correction

특히 범용 dice가 생겼다는 이유만으로 모든 확률성 전투 효과를 D20 presentation으로 바꾸지 않는다.
사용자가 결과를 읽고 다음 선택을 바꿀 수 있는 Tool item, Event, 위험한 node 선택처럼 player-visible한
판정부터 명시적으로 추가한다.

## 필수 검증 범주

- 0/15/30/45/60%가 각각 0/3/6/9/12 D20 failure faces와 정확히 일치한다.
- Event의 natural 1/20, aptitude/DC/equipment, single committed result 계약은 변경되지 않는다.
- invalid use는 die/receipt/inventory/ledger/modifier를 바꾸지 않는다.
- duplicate, reconnect, checkpoint restore와 request retry는 같은 receipt를 재투영하고 item/effect를
  중복 적용하지 않는다.
- 동일 run state와 stable use identity는 target을 바꾸어도 같은 raw die를 사용한다.
- item 소비, dice receipt, max-HP modifier, run ledger와 duration decrement는 failure injection을 포함한
  atomic transaction으로 검증한다.
