# R08~R12 선행 감사 대상과 잔여 조건

R13~R16 제품 변경 전에 필요한 것은 R08~R12 Single Goal의 **독립 GEX-20**이다. 이 문서는 감사 인계 준비이며 감사 실행·승인·판정이 아니다. 현재 사용자 요청과 [지시서 §2](../../../../roadmaps/content_authoring/handoffs/R13_R16_CONTINUOUS_IMPLEMENTATION_REQUEST_20261002.ko.md)가 경계를 정한다.

## 정확한 대상

- Goal/Handoff: `GS-CARX-001 / GOAL-CARX-01 / HA-CARX-001`.
- [계획 인계](../../../content_authoring_r08_r12_runtime/PLAN_HANDOFF.ko.md), [실행 인계](../../../content_authoring_r08_r12_runtime/execution/single_goal/REVIEW_HANDOFF.md).
- [manifest](../../../content_authoring_r08_r12_runtime/execution/single_goal/target/r08_r12_integrated_001/MANIFEST.json): SHA-256 `9842985297f11fefcfeda91cade469cee4f172a74f4498238a02e7e758686a1c`.
- frozen source `bd89f9c42191217a333d648d35f495e802d13e6fb1420caa9c08da3deb731690`, 387 files. source/record/reference archives와 추가·삭제를 포함한 FORWARD.patch는 manifest 옆에 있다.
- [현재와의 차이](CURRENTNESS.json): 소스 목록 중 현황 문서1개만 다르다. 감사는 격리 복원한 exact target 또는 명시적으로 기록한 successor를 대상으로 한다. 현황 문서를 원 작업 트리에서 되돌리지 않는다. 참조 문서3개 차이도 과거 basis/최신 계획 입력으로 구분한다.

## 필수 확인

1. [55조건 수용표](../../../content_authoring_r08_r12_runtime/execution/single_goal/COVERAGE.json)의 요구·실제 코드·공개 입력·receipt·oracle를 대조한다. 현재54 자체 검증을 독립 PASS로 복사하지 않는다.
2. X09: `game::data::content_pack::snapshot::tests::unix::invalid_utf8_directories_and_socket_inputs_return_errors`가 허용 환경에서 실제 nonzero 실행/통과해야 한다. 보고된 실패는 Unix socket 생성의 `Operation not permitted`다. 먼저 정확한 source를 확인한 뒤 이 fixture와 default/test-fixtures 전체 검증의 적용성을 감사한다. 제품 시험 삭제·skip·예외 처리로 완료하지 않는다.
3. [R06/R07 영향표](../../../content_authoring_r08_r12_runtime/execution/single_goal/R06_R07_IMPACT_COVERAGE.json): Snapshot/Live, copy/form, 개인 출처, action/result·resource 수명, 공개 조회·저장 경계. 이전 R06/R07 판정은 R08~R12 공유 코드 변경까지 포함하지 않는다.
4. R08 요청 milestone·취소, R09 특수 kill/host freeze/Breakdown, R10 flow, R11 source modifier/channel, R12 shared pool·비용·생존·parry·actual HP lifesteal의 정상·실패·원자성·admission/retained/clone 검증을 확인한다.
5. V9와 명시 V7/V8 변환, 현재 RON corrosion 정책 추가, 공개 CLI9팩45명령의 trace off/on 비교를 대조한다. OBS001의 직원 preview 정렬은 명시된 비교 예외로만 취급한다.
6. PC36의36조건/576행, 로그 envelope31→32 외 byte 동등 근거, R05 workload·cap9,030,188 유지와 다른 hard guard를 확인한다. concurrent peak342,658 초과만 기존 허용 경고이며 새 cap이 아니다.
7. 감사 중 source 무변경, raw/receipt/manifest 현재성과 archive 무결성, 격리 양방향 복원, 모든 차단 finding 해소를 확인한 뒤 범위에 맞는 독립 판정을 남긴다. X09 하나의 통과만으로 이 전체 감사를 대신하지 않는다.

필요한 감사가 성립하면 R13~R16 정식 계획·연속 구현을 위한 선행으로 인수할 수 있다. 감사가 발견한 제품 교정은 감사와 분리해 수행하며 기존 target을 덮어쓰지 않는다. 이번 준비에서는 감사나 제품 교정을 수행하지 않았다.
