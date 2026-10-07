# 새 구현 대화 시작 요청 — R13~R16

2026-10-02 · v3 · 아래 요청문을 구현 담당자에게 전달한다.
범위와 수용 조건은 [상세 지시서](R13_R16_CONTINUOUS_IMPLEMENTATION_REQUEST_20261002.ko.md)가 소유한다.
작성 시 R08~R12는 자체 검증·동결 후 독립 감사 대기이며 X09 환경 제한 1건이 남아 있다.

## 전달할 요청문

```text
이 저장소의 콘텐츠 제작 기반 로드맵 R13~R16을 아래 지시서에 따라 구현해줘.
이전 대화 없이도 아래 문서에서 범위·합의 정책·수용 조건을 확인할 수 있어.

저장소: F:\work\simulator
WSL 작업 루트: /mnt/f/work/simulator
Rust 패키지: game_core (core_enhanced/)

상세 구현 지시서:
core_enhanced/docs/roadmaps/content_authoring/handoffs/R13_R16_CONTINUOUS_IMPLEMENTATION_REQUEST_20261002.ko.md

현황 진입점:
core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md

선행 구현 인계:
core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/REVIEW_HANDOFF.md

실행 절차:
core_enhanced/docs/policy_to_goal_pipeline_revised_v2.md
core_enhanced/docs/goal_execution_pipeline(1).md

먼저 지시서 §2의 선행 감사와 현재성을 확인해줘.
R06/R07의 과거 Complete/High를 R08~R12 감사로 재사용하지 마.
더 최신인 R08~R12 감사·교정이 있으면 exact target/verdict/잔여 finding/source delta를 인수해줘.
선행 감사가 없다면 이번 요청에 포함된 범위로 R08~R12 독립 완료 감사(GEX-20)를 먼저 수행해줘.
감사 준비나 요청 대기에서 멈추거나 감사 시작의 재승인을 묻지 마.
감사 pass에서는 제품 코드를 변경하지 말고, 독립성 요건을 지켜 실제 수용 조건·현재성·X09 잔여 시험을 확인해줘.
감사 판정과 근거를 문서로 남기고, 통과하여 선행이 충족되면 별도 재개 승인 없이 R13~R16 계획·구현으로 이어가줘.
감사 불합격이나 미해결 차단 조건이 있으면 근거와 필요한 교정을 보고하고, 통과로 간주해 구현을 시작하지 마.

선행이 충족되면 필요한 P2G 계획·Goal/Handoff의 실제 작성과 검증, GEX-00/10을 이어서 수행해줘.
권장 구조는 Single Goal 안의 WP13~WP16과 최종 통합 검증이야.
내부 WP마다 중간 검토 대기로 멈추지 말고 구현·자체 검증을 계속해줘.
실제 필수 Phase 감사와 AT13/14/15 원자적 전환은 생략하지 마.

R13은 소환 actor/정지 함정/field object/진화,
R14는 유물 사본 관리와 개인별 수치 합성의 모든 실제 소비자 연결,
R15는 유한 활성 조건·생성 효과·구조 recipe,
R16은 경로별 연쇄·실제 합류·독립 주기와 파생 효과의 인과 전달을 구현해줘.
침식형 작성·사본당 1회 변환·저장·실제 효과도 유물 기반에 포함하되 상점/Event 전체 배치는 제외해줘.

§4의 확정 정책을 적용해줘. 같은 효과의 피해량은 고정 증가를 더한 뒤 동일 종류 비율 증폭을 합산 적용해.
시작 효과 없는 상호 조건은 휴면이고, 실제 합류는 참여 경로 전체의 발동 기록을 이어받아.
독립 주기 효과는 매회 새 연쇄지만 반응으로 생긴 지연·지속 효과는 원인 연쇄의 제한을 유지해.
차기 룰북 전체를 구현하지 말고 지시서가 지정한 유물 정책만 이번 기반에 반영해줘.

구현 중 결정이 필요한 정책은 구현자의 재량으로 결정해줘.
이번 R13~R16 범위의 미정 gameplay 정책도 포함하며, 큰 플레이 의미의 선택이어도 정책 질문이나 승인 대기로 멈추지 마.
기존 사용자 합의와 구현 범위를 우선하고, 결정 내용·근거·대안·플레이/데이터/저장 영향·검증 조건을
EXPERIMENT_NOTES.md와 해당 정책/설계 문서에 남긴 뒤 구현을 계속해줘.
사용자 직접 결정과 구현자 위임 결정을 구별하고 PLAN.md에서 기록을 연결해줘.
필요한 정식 계약·수용 조건도 갱신·검증하고 동결 계약은 successor로 반영해줘.
이 정책 결정 위임으로 선행 감사나 검증 증거를 생략하지는 마.
제품 변경 전 실제 dirty/untracked 상태를 보존하고 사용자 작업을 되돌리지 마.
PLAN.md, EXPERIMENTS.md, EXPERIMENT_NOTES.md와 실패·중단 원자료를 유지해줘.

§6의 A13~A16/C/X 조건과 원 T 조건을 공개 RON·정상 Core 명령·실제 전투·저장으로 증명해줘.
기본 병렬 시험, 영향 회귀, 최종 전체 검사·CLI·PC36/R05를 수행하고 20분 시험은 제외해줘.
R05 concurrent peak 원 cap 단독 초과의 경고 정책과 나머지 hard guard를 유지해줘.
동결된 선행 실행 경로에 새 로그를 쓰지 말고 신규 실행 위치를 사용해줘.

최종 source currentness와 격리 양방향 복원을 확인하고 통합 target과 인계서를 만들어줘.
종료점은 'R13~R16 구현·자체 검증 종료 / 최종 통합 target 동결 / 독립 감사 요청 대기'야.
필수 기능이 남으면 실제 미완료 상태를 보고하고, R13~R16 구현의 독립 PASS/Complete 발행이나 R17 착수는 하지 마.
이번에 수행하는 선행 감사와 달리, R13~R16 구현 후 최종 독립 감사는 별도 요청 대기로 남겨줘.
```
