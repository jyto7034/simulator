# Tool Abnormality Wiki

이 문서는 확정 로스터의 도구형 환상체를 정리한다.

도구형 환상체는 전투 유닛 조우, 일반 스킬 파편, E.G.O placeholder 장비 색인의 기본 대상이 아니다. 대신 이벤트, 정비, 지원 노드, 리스크 선택지, 장비/강화 이벤트의 source of truth 후보로 다룬다.

## 읽는 법

- `Tool ID`: 후속 RON/data 작업에서 사용할 권장 slug다. 실제 schema가 생기면 live RON이 source of truth다.
- `Runtime Role`: 우리 게임에서 우선 구현할 역할이다.
- `Failure Cost`: 사용 실패, 남용, 반납 실패 등으로 발생할 위험이다.

## Tool Index

| Tool ID | 도구형 환상체 | Runtime Role | Failure Cost |
| --- | --- | --- | --- |
| `giant_tree_sap` | Giant Tree Sap / 거목수액 | 체력 회복 도구 | 부작용 폭발 |
| `portrait_of_another_world` | Portrait of Another World / 이계의 초상 | 특정 유닛 피해 전가 | 전가 대상 피해 집중 |
| `luminous_bracelet` | Luminous Bracelet / 발광 팔찌 | 장착 중 지속 체력 회복 | 체력이 감소한 상태에서 반납하면 즉시 사망 |
| `flesh_idol` | Flesh Idol / 살점 우상 | 기도 시간 기반 체력/정신력 회복 | 너무 짧거나 너무 긴 기도 시 직원 사망 |
| `crumbling_armor` | Crumbling Armor / 부서져 가는 갑옷 | 사용할수록 강해지는 위험 장비 | 남용 시 디버프, n회 퇴각 시 사망 |
| `bloodbath` | Bloodbath / 피의 욕조 | 직원을 희생해 다른 직원 상태 회복 | 선택한 직원 사망 |
| `old_faith_and_promise` | Old Faith and Promise / 오래전의 믿음과 약속 | 무기 랜덤 강화 | 강화 실패 시 대상 소멸 |
| `express_train_to_hell` | Express Train to Hell / 지옥행 급행열차 | 티켓 4개 후 다음 전투 노드에 랜덤 방향 기차 hazard 출현 | 경로 예측 실패 시 대형 피해 |
| `opened_can_of_wellcheers` | Opened Can of WellCheers / 뚜껑 따인 웰치어스 | 랜덤 버프 도구 | 랜덤 사망 |

## Data Policy

- 도구형은 `abnormalities/base.ron`의 전투 유닛 후보로 넣지 않는다.
- 도구형은 일반 `skill_fragments/base.ron` 파편 색인에 억지로 넣지 않는다.
- 도구형이 장착품처럼 작동하더라도 E.G.O placeholder 세트와 같은 의미로 보지 않는다.
- 도구형 전용 schema가 생기기 전까지 이 문서와 `lobotomy_content_catalog.md`의 도구 로스터가 콘텐츠 정책의 기준이다.
