# R06 WP02 비용 회귀 측정·교정 구현 지시서

> **2026-09-27 후속 사용자 결정으로 대체됨.** 약9MB concurrent peak 초과만을 해소하기 위한 필수 교정·반복 실행은
> 다음 작업에서 제외한다. [현재 비용 판정 정책](R06_COST_POLICY_UPDATE_20260927.ko.md)과
> [별도 안전 설정·관측 계획](../../../../../plans/runtime_safety_observability/README.ko.md)을 먼저 읽는다.
> 아래 본문은 당시 제안과 조사 맥락을 보존한 것이며 그대로 재개할 지시가 아니다.

작성일: 2026-09-27 KST.
대상: `GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01`, `WP-CAVER-02`.
상위 인계: `HA-CAVER-P2G60-001`. 해결 대상: 독립 검토의 **R05-E01 비용 증거 미해결**.

## 구현 담당자에게 전달할 요청

> 이 지시서와 조사 보고서를 읽고 R06 WP02의 비용 상한 미해결 사항을 조사·교정해줘.
> D04-F01 기능 PASS를 보존하고, 기존 guard에서 peak를 만드는 소유자와 공존 구간을 먼저 측정해줘.
> 확인한 원인에 맞춰 필요한 runtime 구조만 교정하고, 원 cap과 workload를 유지한 최종 검증을 해줘.
> 성공할 때까지 반복하지 말고 실행 전에 비교·확인 횟수를 고정하고 모든 실패를 보존해줘.
> 구현·자체 검증 뒤 새 successor를 동결하고 독립 재검토 인계에서 중단해줘.
> 원인이나 비용 근거가 닫히지 않으면 미해결 조사 결과를 인계해줘. ByOwner·D07·WP03/04는 이어서 구현하지 마.

계획·독립 검토는 원 대화, 제품 구현과 신규 부하 측정은 별도 구현 대화에서 수행한다.
이번 작업은 R06 비용 회귀 교정이다. R05 기능 개발이나 R05 완료 절차를 재개하지 않는다.

## 1. 시작점과 읽을 자료

1. [조사 보고서](../investigations/wp02_cost_regression_001/INVESTIGATION.ko.md), [기존 raw 재분석](../investigations/wp02_cost_regression_001/analysis.json).
2. [D04-F01 독립 검토](../reviews/wp02_d04_f01_correction_001_review_001/REVIEW.md), [최종 구현 인계](../corrections/wp02_d04_f01_001/REVIEW_HANDOFF.ko.md).
3. [시작 manifest](../corrections/wp02_d04_f01_001/target/d04_f01_correction_001/MANIFEST.json).
4. [CAP 성장 설계·실패 기록](../corrections/wp02_cap_01_02_001/REVIEW_HANDOFF.ko.md), [CAP 독립 검토](../reviews/wp02_cap_01_02_correction_001_review_001/REVIEW.md).
5. [D08 비용 계약](../../../CODE_REVIEW_003.ko.md), [기존 D04-F01 기능 계약](WP02_D04_F01_CORRECTION_REQUEST_20260926.ko.md).

시작 manifest SHA-256: `e4f0d77b1bd3066493d605271605d7a4b6e16317a6d540d303909b076b480104`.
Source 333개 digest: `34ceedbebc84b9396d42f92fa31033fe0b831a90e5325c8c4463e0c88f067145`.
기존 동결본·검토·실패는 덮어쓰지 않는다. 시작 시 `corrections/wp02_d04_f01_001/freeze.py verify`를 실행하고
branch/HEAD·dirty/untracked와 실제 source를 확인한다. 다르면 차이를 먼저 귀속하며 reset/clean으로 제거하지 않는다.
새 correction 구간에 before bytes/archive·manifest·PLAN을 보존한다.

## 2. 측정 계약

- 원 tool: `core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs`.
  SHA-256 `9ab90be6e9049ee443075b8dbb7c202c2961e9ebe506166893870f94a4165ae3`.
- 원 baseline: `core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json`.
  SHA-256 `86ccd5c78b0dcdfab502b1c5842a48c8a9df8963557e36cbf49b3ea5ec2b5848`.
- workload input SHA-256: `6e164237a488ad196e60e3b01d710177a325fe0a81105eb97f9f1b0b0e6967aa`.
- concurrent peak delta cap: **9,030,188 bytes**. allocation operations·retained delta의 원 guard도 함께 유지한다.
- 64 sessions / 8 lanes / 100 steps, 원 seed·affinity·warmup·5 allocation samples·직렬화·관측 경계 유지.
  latency gate 미적용을 시간 성능 합격으로 표시하지 않는다.

PLAN에는 실제 source/tool/input/binary hash, rustc/Cargo·profile/features/flags, 명령/cwd/env,
CPU affinity 가능 범위, 다른 부하·가용 메모리와 build 공간을 기록한다. 측정과 빌드/전체 시험은 동시에 돌리지 않는다.
격리 빌드의 supplemental `include_bytes` 입력과 lockfile도 기록한다. 기존 D04 대조의 첫 compile 실패는
allocation 실패와 구별하고, 현재 유효한 exit 101 결과를 환경 오류로 재분류하지 않는다.

## 3. 먼저 소유자와 공존 구간을 찾는다

현재 교정본을 A로 고정한다. D04 predecessor는 이미 같은 cap 초과 증거가 있으므로 단순 재확인만을 위해
추가 반복하지 않는다. 이전 버전 비교가 특정 가설에 필요하면 비교할 정확한 target·질문·횟수를 PLAN에 먼저 적는다.

진단은 격리 사본에서 다음 순서로 수행한다.

1. setup / step / serialization / drop 단계와 lane을 구별해 global live peak 부근을 관찰한다.
   allocator 내부에서 출력·동적 collection·allocating stack capture를 하지 않는다. 고정 크기 trace/counter의
   overflow는 명시하고, 여러 atomic 읽기를 일관된 snapshot으로 오인하지 않도록 설계한다.
2. ActorSummaries, event queue, timestamp bucket의 len/capacity, 성장 요청 bytes, COW 복제와 원본의 공존을 연결한다.
   현재 6,660 reallocations는 CAP부터 관찰됐다. 횟수 감소 자체를 성공 기준으로 쓰지 않는다.
3. `ActiveAction` 및 queue/capture payload의 실제 size/alignment와 owner 수, 공유/복제 비용을 확인한다.
   literal 경로의 불필요한 식 전용 payload와 중복 owner를 우선 살핀다. 기존 optional Box·lazy witness를 존중한다.
4. 직렬화와 runtime 공존이 주원인인지 구분한다. 진단용 barrier/단독 lane 등 보조 실험이 필요하면
   원 workload와 다른 실험임을 표시하고 acceptance 증거로 사용하지 않는다.

진단용 변경과 계측 오버헤드·해석 한계를 보존한다. 계측 없는 원 binary의 수치와 혼합하지 않는다.
정확한 owner 귀속이 어려우면 추정과 확인 사실을 구분하고, 구조 변경의 근거가 없다는 점을 인계한다.

## 4. 교정 방향과 유지할 의미

확인된 중복 저장·복제·여유 용량만 줄인다. owner lifetime을 짧게 하거나 공통 불변 payload를 공유할 수 있지만,
그 payload가 어떤 action/cast/delivery/attempt/generation에 속하는지와 마지막 소비 시점을 설명해야 한다.
형태를 바꾸면 실제 retained extent, CloneShape, COW 복사 비용과 reserve 전 선청구를 함께 맞춘다.
관측 계측에서 비용을 빼거나 예산 숫자를 낮춰 적는 변경은 실제 footprint 교정이 아니다.

특히 다음을 유지한다.

- release Snapshot 고정값과 이후 Live 재평가, action 전체 완료 결과 누적 및 delivery 완료.
- D04-F01의 no-target amount 평가 생략과 후속 effect·step·delivery 진행, 실제 target capture identity 검사.
- CAP의 checked 성장·최소 합법 용량 fallback·capacity 기반 live accounting, PC 기존 36조건 성공31/거부5.
- 실제 resource authority, 오류 전파·candidate rollback·기존 commit, 정렬/동일 key 순서, source identity.

성장 배수만 반복 탐색하거나 eager Box/Arc로 옮겨 비용을 숨기지 않는다. 보유량과 operation 청구는 별개다.
원 guard의 lane 수·부하·seed·serialization·관측 범위 변경, cap 확대, 평균/percentile 채택으로 통과시키지 않는다.
도구 결함이 발견되면 최소 재현과 원/교정 observer 비교를 별도 인계한다. 교정 도구의 PASS만으로 기존 실패를 닫지 않으며,
제품 교정으로 해결할 수 없는 측정 계약 변경은 이 단계에서 확정하지 않는다.

## 5. 사전 고정 비교와 최종 검증

계측을 제거한 교정 후보 B를 고정한 뒤 **A → B → B → A** 순서로 원 guard를 실행한다.
각 실행은 원래의 5개 allocation sample을 유지한다. 이 비교는 스케줄 편향을 줄이려는 진단이며 통계적 인과 증명은 아니다.
실행 전 PLAN에 순서·횟수·A/B source/binary hash를 기록한다. 유효한 guard 실패도 raw/log/exit를 남기고 계획된 비교를 마친다.
build 실패나 입력 불일치는 별도 기록 후 해결하고, 동일 실행 번호를 덮어쓰지 않는다.

B에 유효한 실패가 있으면 그 revision은 수용 후보가 아니다. 새 가설·실제 수정·새 revision 없이 성공을 얻으려 반복하지 않는다.
B가 비교 검증을 통과하면 동일 immutable binary로 **추가 3회** 원 guard를 확인한다.
이는 과거 비용 교정의 고정 3회 확인 방식을 따른 이번 증거 계획이며, OS scheduling의 수학적 상한 보증은 아니다.
B의 총 5회/25 allocation sample을 모두 보고하고, 실패·최댓값·cap 여유를 공개한다. 최소 여유의 새 임의 cap은 만들지 않는다.
새 revision이면 이전 결과를 그대로 보존하고 최종 revision의 검증 계획을 다시 명시한다.

최종 제품 source에서 다음을 검증한다.

- 변경한 ownership/COW/growth 경계의 의미 있는 회귀, 기존 CAP 경계와 action/Snapshot/D04·D04-F01 9건.
- `cargo check -p game_core --all-targets --offline --locked`.
- `cargo test -p game_core --all-targets --offline --locked`: 현재 기준 1,509건이며 최종 discovery의 전체를 실행.
- `cargo clippy -p game_core --all-targets --offline --locked -- -A clippy::never_loop -A clippy::eq_op`, 변경 영역 format.
- 동일 최종 revision의 원 PC 36조건: 성공31/typed refusal5 및 기존 결과·오류 parity. 거부를 성공으로 바꿔 비용을 비교하지 않는다.
- 위 원 allocation guard 전체(event-heavy 포함). source·binary identity가 맞는 비교 실행은 재사용 가능하며 불필요하게 다시 실행하지 않는다.

검증마다 실제 selector/실행 수·stdout/stderr·exit·source 전후 hash를 남긴다. 진단용 계측이 최종 제품에 남지 않았는지 확인한다.

## 6. 동결과 중단

결과에 따라 새 successor 또는 미해결 조사 인계에 다음을 담는다.

- 원인으로 확인한 allocation owner·공존 구간, 후보 비교와 설계 선택, 남은 불확실성.
- 모든 과거 실패와 새 실험/교정 revision, 사전 PLAN 대비 실행 목록, 원시 receipt/log/hash.
- before/after source·manifest·실제 archive·양방향 delta, 변경 목록과 읽기 전용 currentness/복원 확인.
- 최종 source에 결속된 회귀·PC·비용 결과 및 `REVIEW_HANDOFF.ko.md`.
- D04-F01 기능 PASS 보존과 R05-E01 해결 요청 또는 미해결 사유. ByOwner·D07·WP03/04·publication 차단 유지.

자체 검증 성공은 독립 PASS나 WP02/R06 완료 판정이 아니다. 기존 target과 검토 문서를 수정하지 않는다.
**측정·교정 및 자체 검증 종료 / successor 또는 미해결 조사 인계 / 독립 재검토 요청 대기**에서 중단한다.
