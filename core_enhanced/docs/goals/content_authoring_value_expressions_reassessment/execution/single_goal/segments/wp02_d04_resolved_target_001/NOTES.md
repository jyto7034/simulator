# R06 WP02 D04 단계검토 메모 — 실제 resolved target

## 선행과 변경 소유권

- 시작점은 action 결과·release Snapshot target manifest SHA-256 `b16697bfe8c518944f0bddcf800b0f1e005ed639edd6fe915e4d96c726a6e1f4` 및 그 독립 부분 PASS다. 시작 333개 실제 source digest는 `60fe6643fa136a16773e93042d9db7d3e5b9398088c495771796abd410708717`이다. 전체 dirty worktree는 baseline archive에 그대로 보존했고 reset·stash·clean하지 않았다.
- `execute_ordered_skill_effects`가 해당 effect에 확정된 target 배열을 소유한다. 이 배열은 command 전달 순서대로 유지한다. battle adapter가 별도 복사본의 UnitInstanceId를 정렬·중복 제거해 집계 frame을 만든다. identity는 UUID의 표준 문자열이고, 값이 같은 다른 unit은 합치지 않는다.
- 실제 member는 active RuntimeUnit의 명시적 Attack 또는 HP Current/Max/Missing producer만 읽는다. `UnitBody::projected_tile`을 해당 시각의 grouping key로 사용한다. evaluator의 기존 reduction·empty·overflow·trace 기능을 그대로 호출한다. 실제 unit에 임의 scalar를 부여하지 않는다.
- set 미제공은 `UnsupportedMoment`, 확정된 빈 set은 유효한 frame이다. resolved ByOwner는 domain 선택이 없어 `InvalidOwner`, 실제 unit의 `AggregateMember { unit }`는 명시 scalar producer가 없어 `MissingContext`다. WP03 stat은 그 producer가 생길 때까지 typed refusal이다.

## 시각, 비용, 오류 경계

- Snapshot은 release에서 확정된 대상의 identity·필요 값·tile key를 immutable Arc member payload에 담는다. target별 scalar frame은 별도로 유지하며, 같은 Snapshot payload를 target 수만큼 deep clone하지 않는다. directional projectile 및 area는 release set이 없으므로 resolved Snapshot이 거부된다.
- Live는 impact, tick, immediate 소비 때 확정된 set을 effect 시작마다 읽는다. 같은 effect의 모든 대상 amount 계산 후 mutation을 시작하고, 다음 effect나 tick은 새 member frame을 사용한다. Source, 현재 Target, action result prefix와 AggregateMember는 서로 다른 frame slot을 쓴다.
- adapter는 raw target 수, 복사·정렬·필요 reference·member payload의 checked 크기와 work를 할당 전에 승인받는다. 중복 입력의 raw 준비 work와 canonical member 수는 구분된다. native retained·authoring work/cancel 검사는 별개이며 evaluator의 nested/group budget도 적용된다. release carrier는 공유 payload 보존량을 한 번만 계산한다.
- typed fault는 기존 candidate transaction으로 전달된다. 성공한 앞 effect 뒤 집계 오류를 넣어 현재 candidate의 HP·fact·witness 등 전체 복원과 정상 retry를 시험했다.

## checkpoint 경계

실제 WholeSet·ByProjectedTile의 immediate, projectile impact, area tick, release Snapshot 연결을 이번 첫 검토 단위로 닫는다. resolved ByOwner는 현재 `Grouping::ByOwner`에 Side/Employee/EquipmentCopy domain 인자가 없으므로 원 대화에서 schema 및 plan 변경 범위를 정해야 한다. 실제 unit scalar producer, WP03 SpellPower/cadence, D07 preview/explain, WP04, 서버·Unity, production publication은 이번 target에 포함하지 않는다. 자체 검증 결과를 독립 검토 PASS나 WP02/R06 전체 완료로 주장하지 않는다.
