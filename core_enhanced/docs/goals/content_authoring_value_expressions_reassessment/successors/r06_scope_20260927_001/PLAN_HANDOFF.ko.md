# R06 집계 정책·범위 successor 계획 인계

2026-09-27 · 계획/문서 인계. 제품 구현은 별도 대화에서 수행한다.

**다음 작업은 WP02 D04의 유닛 관계·명시적 필터·장비 슬롯 전달 구현이다.**
이 대화에서는 정책·코드 검토와 구현 지시서를 작성했다. WP02/R06 전체는 아직 완료되지 않았다.

## 읽을 문서

1. [검토·정책·범위·완료 기준](REVIEW_AND_PLAN.ko.md): U01–U04, E01–E09 대조, 현재 구현과 잔여, 후속 R의 책임.
2. [다음 구현 지시서](IMPLEMENTATION_REQUEST.ko.md): 설계 방향, producer/consumer 동시 변경, O01–O15 시험과 중단 지점.
3. [전체 R06 successor Goal](P2G_50_GOAL_SET.ko.md): 이번 구간 이후 D07/WP03/WP04와 최종 완료 기준을 포함한다.

개발 진입점은 [ROADMAP_STATUS](../../../../roadmaps/content_authoring/ROADMAP_STATUS.md) 하나다.
이 문서는 선택된 R06의 인계 근거이며 전체 프로젝트의 별도 로드맵이 아니다.

## 확정한 정책

- 실제 직원만 직원별 집계 입력으로 허용한다. 소환물은 소환자 직원에게 자동 합산하지 않는다.
- 효과의 출처 장비와 대상 유닛이 착용한 지정 슬롯의 장비는 서로 다른 관계이며 실제 사본으로 식별한다.
- 적용 불가능한 member를 제외하려면 제작자가 조건을 명시한다. 조건 없이 섞으면 검증/실행 오류다.
- R06은 이미 resolve된 유닛의 진영·직원·지정 슬롯 관계와 필터까지 구현한다.
  효과·상태 출처 집계는 R11 조회 기반 및 최초 소비자/R17 통합에 배정한다.

기존 27개 수용 기준은 모두 유지하고 E03에 4개를 추가했다. 다른 8개 정책 요소는 재결정하지 않았다.
모델 전체는 구현하면서 개선하는 기준안이다. 실험실·편집기·구조화 Flow 전체를 R06에 넣지 않는다.
특정 약9MB concurrent peak의 경고 전환은 기존 사용자 결정을 계승했고, 다른 hard guard를 해제하지 않았다.

## Exact successor basis

| 단계 | 현재 계획 identity | 파일 |
| --- | --- | --- |
| 정책 | WR-CAVER-SCOPE10-001, DR-CAVER-E03-SCOPE-001 | [정책](P2G_10_POLICY.json) |
| 코드 검토 | WR-CAVER-SCOPE20-001, RB-CAVER-SCOPE20-001, CRA-CAVER-SCOPE20-001 | [검토](P2G_20_REVIEW.json) |
| 정책 동결 | FPF-CAVER-SCOPE30-001 | [Freeze](P2G_30_FREEZE.json) |
| Phase | PP-CAVER-SCOPE40-001, PVR-CAVER-SCOPE40-002 | [Plan](P2G_40_PHASE_PLAN.json), [최종 receipt](P2G_40_RECEIPT_FINAL.json) |
| Goal | GS-CAVER-SCOPE50-001 / GOAL-CAVER-01 | [Goal](P2G_50_GOAL_SET.ko.md) |
| 인계 | HA-CAVER-SCOPE60-001 | [Handoff](P2G_60_HANDOFF.json) |

[변경 선언](CHANGE_RECORDS.json)은 E03 정책 변경과 기존 비용 판정 변경의 영향 범위를 구별한다.
[해소 관계](CHANGE_RESOLUTIONS.json)는 각 owner 단계의 결과를 이 successor basis에 연결한다.
자기 artifact를 가리키는 resolution event가 다시 그 artifact의 구성원으로 포함되는 순환을 피하도록
해소 관계는 `complete_stage_result`의 외부 관계로 기록했다. [최종 검증](VERIFICATION.json)이
declaration 완전성·11개 obligation 대응·basis·참조·표준 검증을 함께 확인한다.
이 결과가 통과하기 전에는 개별 후보 파일의 존재만으로 current 계획을 선택하지 않는다.

기존 `GS-CAVER-P2G50-001` 등 원본을 덮어쓰지 않았다. 동결 원본과 과거 구현·감사는 당시 증거로 보존하고,
이후 작업은 위 successor와 기존 D04-F01 target을 함께 인수한다.
`PREDECESSOR_REFERENCES.json`은 표준 검증기의 같은 디렉터리 Ref 해석에 필요한 원본 artifact 사본이다.
원본 파일/hash와 canonical payload를 검증하며 새 정책·새 진입점으로 취급하지 않는다.

## 검증과 한계

[SOURCE_EVIDENCE](SOURCE_EVIDENCE.json): F01 manifest의 source 333개를 현재 크기/hash와 읽기 전용 비교했다.
빌드·게임 시험·양방향 복원·독립 감사는 재실행하지 않았다. 이전 시험 1,509개 등의 결과는 이전 target에만 귀속한다.

[VERIFICATION](VERIFICATION.json)은 P40/P50 표준 검사와 문서/정책/보존 검사를 기록한다.
이는 구현 완료나 runtime PASS가 아니다. 초안 검사에서 receipt ref 누락을 발견해 교정한 경위와
최종 complete-result 관계를 보완하기 전 receipt도 검증 이력으로 구분한다. `P2G_40_RECEIPT_FINAL.json`만 최종 기준이다.

별도 구현 대화는 mutation 전에 현재 변경·선행·scope를 재확인하고 구현 기준선을 보존한다.
이번 D04 구간의 실제 자체 검증·target 동결 후 **독립 검토 요청 대기**에서 중단한다.
D07·WP03·WP04 전체 검증과 독립 완료 감사가 남으며, 이를 이번 문서 인계로 완료 처리하지 않는다.
