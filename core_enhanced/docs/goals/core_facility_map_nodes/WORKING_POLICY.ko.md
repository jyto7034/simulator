# U13 고정 시설·노드 배정·탐사와 브리핑 조회 — Working Policy

- Working Policy Document ID: `WPD-CFMN-20261013`
- Target Documentation Root: `core_enhanced/docs`
- Policy Directory Name: `core_facility_map_nodes`
- Current Physical Path: `core_enhanced/docs/goals/core_facility_map_nodes/WORKING_POLICY.ko.md`
- Current Working Checkpoint: `WR-CFMN-005`, predecessor `WR-CFMN-004`
- Policy Intake Snapshot: `PIS-CFMN-002`
- Current Stage: **GEX-10 구현·자체 검증 종료 / 감사 요청 대기; Goal Complete 아님**
- Progression Mode: `bounded_pipeline`, 사용자의 2026-10-13 U13 착수 요청

## 권한과 현재 상태

지정 실행 지침을 읽고 U12 Complete / High와 U13 카드·최신 지도 입력을 인수했다. 최신 착수 요청은 과거 U12 종료 후 중지 경계를 U13 범위에서 해제한다. 한 실행자가 직접 수행하며 서브에이전트를 사용하지 않는다. 모델을 전환했다고 주장하지 않는다.

U13 S11/G07–G12와 해당 카드의 구현·제외·수용 조건을 범위로 한다. Core 우선, Unity 후속, 개별 최종보스 고유 구현 제외를 유지한다. 필요한 P2G 상세화와 GEX 구현·자체 검증까지 승인된 요청으로 해석한다. 감사까지의 명시 위임·중간 단계검토 opt-in은 없으므로 GEX-10 종료 시 기본 감사 인계 경계를 적용한다. U14 이후 구현은 자동 착수하지 않는다.

[초기 원문과 WR001](P2G_00_INITIAL_001.json), [P2G-00 결과](P2G_00_RESULT_001.json), [두 미결 요소·질문과 WR002](P2G_10_OPEN_001.json)가 exact Ref와 승인 원문을 소유한다. 원문은 INTAKE_SOURCE_001.tar.gz에 보존했다. P2G-00 초기 set은 모두 비어 있으며 후속 질문은 P2G-10 별도 checkpoint에 있다. 이것은 Goal 발행·제품 구현 완료 기록이 아니다.

## 확정된 배정과 현재 정책

사용자가 위치 배정을 단순하게 하고 보상5종 각20%를 채택했다. 이어 “아 계속 진행해줘.”로 재개를 지시했다. [원문·Decision·WR004](P2G_10_DECISIONS_001.json), [P2G-10 결과](P2G_10_RESULT_001.json)가10요소·30수용·열린 질문0을 소유한다. 과거 질문/미결 WR002는 당시 원문이다.

위치는 기존 예약·적격·동선 조건을 지키는 후보 사이 균등 추첨이다. 갈래별 균등 기회 보정·입구/깊이 우대·플레이어 선택 유도를 추가하지 않는다. 초반 보상은 단순 선택형 고정, 그 외5종 각20%다. 같은 원정 배정 보존·미지원 executor를 가짜로 채우지 않는 기존 계약을 유지한다. 정책 변경을 현재 룰북/로드맵에도 반영했으며 과거 동결 원본은 수정하지 않았다.

## 사전 인수 당시 관찰 — 이력

아래는 제한적 코드 읽기 관찰이며 정식 P2G-20 종료 증거가 아니다.

- `src/game/map/facility.rs`: 기존 FacilityTemplate은 방 내부 slot·connection·terminal, NodeAssignmentProfile은 category constraints/weights와 fixed/tagged assignments를 표현한다.
- `src/game/map/facility_generator.rs`: `generate_with_requirements`가 floor schedule의 pool에서 template을 고르고 floor/seed별 map을 생성한다. 전체 고정 방 graph와 방별 재방문 상태의 완성 경로라고 간주하지 않는다.
- `src/game/world/node_flow.rs`: 기존 Gate 이동이 다음 floor map 생성과 recovery로 연결된다. U13에서 authored room identity·동층 왕복·하강만 회복·방 완료/영구 지식·저장 복원을 함께 대조해야 한다.
- 최신 설계 입력은 입구 포함89방/846활동이며 제품 RON 실행 증거는 아니다. 고정 예약 후 U20 전조가 사용할 호환 일반전 자리를 보존해야 하며 모든 방에 전조를 생성하지 않는다.

현재 저장소에는 이전 작업의 다수 dirty/untracked 변경이 있다. reset/clean/stash/commit 또는 제품 입력 수정은 하지 않았다. 이 세 코드 관찰은 source freeze/완료 감사 현재성을 대신하지 않는다. 방 identity·저장·조회 전환의 동시 경계는 P2G-20/40에서 좁혀 확정한다. 지금 확인한 기존 generator와 새 전체 graph 간 차이 때문에 관련 기반 개선 검토가 필요하며, 범용 재설계는 승인하지 않는다.

## P2G-20 진입 당시 위치 — 이력

현재 WR004 고정 입력으로 P2G-20을 수행한다. 실제 producer/consumer·schema·저장·조회·지원 프로파일을 대조하고 최소 동시 전환과 P2G-40 phase 구조를 정한다. 현재 제품/시험 변경·빌드·게임 실행은0회, formal Goal/Phase 발행 및 GEX는 미실행이다. 실행 지침상 필요한 증거를 재사용하며 원정/방/노드의 새 계약은 실제 검증한다.

## 구현 인계 위치

P2G40/50 구조 검증과 P2G60/GEX00 이후 GS-CFMN-001의5개 실행 node를 구현·자체 검증했다. 현재30수용·10통합·10차원의 근거는 [COVERAGE](execution/single_goal/COVERAGE.json), 원 실패와 실제 명령은 [실행 기록](execution/single_goal/EXPERIMENTS.md), 현재 제한은 [검토](execution/single_goal/IMPLEMENTATION_REVIEW.md)에 있다. 공식 유물5종 실행은 U16 대기이므로 공식 첫 방의 첫 유물 활동은 미지원이며 전체 플레이 가능으로 주장하지 않는다. 구현 후 고정 target은 `execution/single_goal/target/u13_current_001`이다. GEX-20은 새 감사 요청 전 시작하지 않으며 U14 이후 작업도 시작하지 않는다.
