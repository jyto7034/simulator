# R05 단일 R 요청 인수 — 2026-09-12

이번 active thread goal은 사용자가 채택한 [AUTORUN single-r-v2](../../roadmaps/content_authoring/AUTORUN_REQUEST.ko.md)의 단일 R 범위다. **R05 — 타입화 자원·owner×lifetime·원자적 소비**로 선정·고정한다. 자동 재개에서 다시 선정하지 않는다. R05 전체의 applicable 요구·직접 검증·필수 감사가 모두 완료되면 이번 Goal을 종료하며 다음 R은 새 사용자 요청에서 시작한다. 과거 R00–R24 요청과 P2G-00 원본은 당시 근거로 보존한다.

## 선정과 현재 basis

- ROADMAP 2.2-review.1 순서에서 R00의 기존 인수 및 R01–R04 완료 기록 다음 첫 미완료가 R05다. NEXT-0013와 실제 Topic은 `WPD-CARL-20260911`이다.
- Branch `cleanup/simulator-dirty-worktree-20260803`, HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`. 기존 dirty 전체 목록은 [인수 상태](intake/BASELINE_GIT_STATUS_001.txt)에 보존했다. source/RON/store를 이번 P2G에서 변경하지 않는다.
- 기존 마지막 정식 결과는 `WR-CARL-P2G00-001`. 이번 raw-input successor는 [PIS-CARL-P2G10-001](P2G_10_INTAKE_001.json), policy entry checkpoint는 [WR-CARL-P2G10-ENTRY-001](P2G_10_ENTRY_001.json)이다. 이들은 초기 인수 이력이며, 현재 Goal/Handoff는 아래 P2G-60 결과를 따른다.
- 현재 프로세스에서 `.agents`/`.codex`는 비어 있고 workspace 상위/내부 검색에서 AGENTS.md를 찾지 못했다. 현재 developer/runner 지침, P2G `2026-09-11-r5` Reader Contract를 적용한다. GEX 진입 때 해당 Reader/단계 계약을 추가 확인한다.
- 이전 goal turn의 작업 결과는 이 새 active goal에 제공되지 않았다. 따라서 이전 턴 진전을 임의 판정하지 않는다. 이번 인수는 current evidence와 불변 successor를 만드는 progress다.

## 승인 범위와 종료 조건

R05 자원 store·owner×여섯 lifetime·reserve/commit/abort·cap/grant/decay·epoch/lease, 실제 장비/직원 binding, 도입 Run 상태의 writer/restore/checkpoint/entry/result exactly-once 및 T05/T20/T23/T29를 함께 소유한다. AT05를 분리해 저장이 빠진 상태를 완료로 세지 않는다. 서버/Unity/UI/외부배포, 다음 R의 새 기능, 개인 저장 삭제/자동변환, pipeline/validator/AGENTS/global 설정 수정은 제외한다.

동일 R의 필요한 P2G owner 전환과 자격 Goal의 GEX는 이번 요청으로 승인됐다. 통상적 정책은 기존 owner에 실제 위임·다섯 조건·결정을 기록한다. 사용자 선택이 필요한 의미만 질문 후 주 작업을 대기한다. 독립 리뷰가 요구되는 단계에서 구현 요약으로 대체하지 않는다.

## 확인한 의미와 다음 작업

장비 사본 자원의 유지·소멸 답변은 기존 기록0066에서 인수했다. 후퇴는 game_rulebook의 inventory/resource rollback, 최신 저장은 R02 DR-CAPG-E04-001의 계약으로 연결된다. 이 사실 확인은 [POLICY_FACTS](intake/POLICY_FACTS_001.json)가 소유하며 정식 P2G-20 검토가 아니다.

선행 currentness 세션25769는 exit 0으로 종료했다. 6,946개 파일에서 20개 불일치(고유5경로)를 찾았으며 [원시 대조](intake/PREDECESSOR_CURRENTNESS_001.json)를 보존한다. 코드 차이는 감사 source archive의 exact SHA와 대조한 세 곳의 줄바꿈뿐이다. 나머지4경로는 기록0064–0066의 실행 지침 개정이다. [영향 판정](intake/PREDECESSOR_REUSE_001.json)으로 기존 gameplay 증거를 재사용하되 historical manifest를 현재 exact 일치라고 표시하지 않는다. P2G-10의9요소/9결정·15종료조건 검증은 완료했다. 이후 P2G-20–60을 완료해 [단일 Goal](P2G_50_GOAL_SET_001.ko.md)과 [Handoff](P2G_60_HANDOFF_001.json)를 확정했다. [GEX-00](execution/single_goal/GEX_00_RESULT_001.json)에서 실행 기준선과 선행 currentness를 인수했다. 현재 [GEX-10 PLAN](execution/single_goal/PLAN.md)에 따라 typed 선언·store/registry·Run projection을 구현·검증 중이며 실제 battle/Run AT05/catalog/public 연결과 전체 감사는 남아 있다. [실제 시험 기록](execution/single_goal/EXPERIMENTS.md)을 현재 검증 상태로 사용한다.
