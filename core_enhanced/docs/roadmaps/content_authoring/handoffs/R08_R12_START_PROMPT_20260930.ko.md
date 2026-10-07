# 새 구현 대화 시작 요청 — R08~R12

2026-09-30 · v1 · 사전 대화가 없는 구현 담당자에게 아래 요청문을 전달한다.
정식 실행 기준은 [상세 지시서](R08_R12_CONTINUOUS_IMPLEMENTATION_REQUEST_20260930.ko.md)다.
현재 R06/R07은 구현·자체 검증과 동결만 확인되며 독립 감사는 대기다. 선행 감사를 확인하지 않은 채 제품 구현을 시작하는 요청이 아니다.

## 전달할 요청문

```text
이 저장소의 R08~R12를 아래 지시서에 따라 구현해줘.
이전 대화 내용은 몰라도 된다. 다음 문서가 작업 범위와 기준이다.

저장소: F:\work\simulator
WSL 작업 디렉터리: /mnt/f/work/simulator
Rust 패키지: game_core (core_enhanced/)

상세 구현 지시서:
core_enhanced/docs/roadmaps/content_authoring/handoffs/R08_R12_CONTINUOUS_IMPLEMENTATION_REQUEST_20260930.ko.md

필수 구현 설계와 수용 시험표:
core_enhanced/docs/roadmaps/content_authoring/handoffs/R08_R12_DESIGN_AND_ACCEPTANCE_20260930.ko.md

조사 기준선:
core_enhanced/docs/roadmaps/content_authoring/handoffs/R08_R12_CODE_EVIDENCE_20260930.json

개발 현황의 단일 진입점:
core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md

실행 지침:
core_enhanced/docs/goal_execution_pipeline(1).md
core_enhanced/docs/policy_to_goal_pipeline_revised_v2.md

프로젝트는 타일에 직원을 배치하는 실시간 방어 게임의 Core다.
현재 로드맵은 외부 데이터를 조합해 스킬·보스·장비·파편·유물을 제작하는 기반을 만든다.
R06은 수식/실제 결과/Attack·SpellPower, R07은 파편 사본/개화/형태/저장의 기반이다.
이번에는 R08 보스 스킬 호출, R09 침식, R10 구조화 스킬 흐름,
R11 지속 효과·집중·기본 공격 교체, R12 보호막·생존·제어·HP 비용을 연결한다.

먼저 지시서 §2의 R06/R07 선행 상태를 확인해줘.
기존 인계서는 독립 감사 완료가 아니라 구현·자체 검증 종료/감사 대기다.
별도로 수행된 감사가 있으면 exact 문서·verdict·target·미해결 finding·현재성을 확인해 인수해줘.
필수 감사가 여전히 없다면 준비 가능한 조사·계획까지 진행하고, 필요한 감사 범위를 정확히 보고해줘.
감사 PASS를 추정하거나 이번 요청만으로 독립 감사를 자동 수행하지 마.

선행이 충족되면 필요한 P2G 계획 준비·검증과 GEX-00/10 구현을 이어서 수행해줘.
기존 정식 계획이 있으면 재사용하고, 없으면 지시서의 권장 위치에 실제 산출물을 작성해줘.
로드맵 카드·이 요청문·문서 링크 검사는 정식 P2G 검증 결과가 아니다.
계획만 설명하고 끝내지 말고 허용된 구현과 자체 검증을 계속해줘.

R08~R12는 중간 부분 검토를 매번 기다리지 않는 연속 구현 요청이다.
권장 Single Goal 안에서 WP별 책임·수용표·경계 source를 보존하고 최종 통합 target을 만들어줘.
실제 정식 계획의 필수 Phase 감사나 atomic group을 무시해서 연속 실행하지는 마.
기존 템플릿의 '첫 부분 구현에서 중단' 문구 대신 이번 상세 지시서의 종료점을 따라줘.

현재 승인된 계약과 해당 카드가 구현 기준이다.
차기 룰북의 원정 지도·억제력·황금가지·생환·회수품 대여는 이번에 구현하지 마.
모딩 공통 기반을 먼저 완성하고 차기 원정 전환은 나중에 별도 수행할 계획이다.
실험실 스트레스 인증·metrics·안전 설정 RON 분리·편집기·R13 이후도 제외한다.
20분 시험은 모든 단계에서 제외한다.
예산 초과의 새 자동 스킬 취소 정책은 만들지 말고, 기존 typed fault/rollback을 유지해줘.

기존 사용자 위임 범위의 통상적 세부 정책·기술 판단은 스스로 결정하고 진행해줘.
구현 설계의 정책 표는 추천안이므로 기존 계약과 대조해 P2G에 채택 이유·대안·영향·시험을 기록해줘.
사용자가 직접 승인한 값인 것처럼 쓰거나 차기 원정의 미정 정책까지 확정하지 마.
범위 밖의 큰 정책 변경·필수 증거 부재·안전하게 구분할 수 없는 병행 변경은 정확히 보고해줘.

제품 변경 전 실제 dirty/untracked 파일까지 before로 보존하고 사용자 작업을 되돌리지 마.
구현 중 PLAN.md, EXPERIMENTS.md, EXPERIMENT_NOTES.md 세 기록을 계속 갱신해줘.
실패·중단·재시도 원자료를 보존하고 현재 위치는 PLAN 한 곳에서 안내해줘.
작성된 코드 근거를 현재 source와 다시 대조해 생산자→검증→실행→결과/저장→조회 경로를 닫아줘.

필수 A08~A12/X 수용 조건을 실제 공개 RON·정상 명령·전투·저장 경로로 증명해줘.
기본 병렬 시험을 사용하고, 집중 시험은 변화마다, 전체 회귀는 영향과 최종 통합 기준으로 수행해줘.
긴 명령은 진행 로그를 보여줘. 0개 시험·중단·결과 불명은 통과가 아니다.
R05 concurrent peak의 원 cap 9,030,188 bytes 단독 초과는 경고이며 다른 hard failure는 유지한다.
동결된 R06/R07 실행 디렉터리에 새 로그를 쓰지 말고 도구의 출력 위치를 새 실행 경로로 연결해줘.

최종 소스에서 전체 자체 검증과 source currentness·격리 양방향 복원을 마친 뒤 인계서를 작성해줘.
종료점은 'R08~R12 구현·자체 검증 종료 / 최종 통합 target 동결 / 독립 감사 요청 대기'다.
필수 기능이 남으면 전체 완료 대신 미완료 조건을 명시해줘.
독립 감사 PASS/Complete를 스스로 발행하거나 R13을 시작하지 마.
```

감사 담당자에게 전달할 요청문이 아니다. 구현 종료 후 별도 대화에서 최종 target을 대상으로 감사를 요청한다.
