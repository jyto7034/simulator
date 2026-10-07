# U07-M 실측 기반 실행 예산 조정 준비

Document Identity: `WPD-CMRP-20261005` · Target Documentation Root: `core_enhanced/docs`.
Policy Directory: `core_runtime_measurement_profiles` · Current Working Revision: `WR-CMRP-003`.

[초기 raw 인수](P2G_00_INITIAL_001.json)와 [초기화 검증](P2G_00_RESULT_001.json)이 승인 범위와 exact refs를 소유한다.
현재 P2G-20 완료, 다음은 P2G-30 정책 동결이다.10정책·13candidate(승격10/범위외3), 미결질문0.
P2G-10 Completion Predicate: true. [정책/결정](P2G_10_POLICY_001.json)과 [검증](P2G_10_RESULT_001.json)이 근거이며 제품 구현/실측 완료는 아니다.

[선행 인수](PREDECESSOR_INTAKE_001.json)는 U07 독립 Complete/High/open0와 현황 문서만의 비영향 delta를 인수한다.
원래 배포 기준과 실패 의미를 보존하면서 trusted 유한 개발 측정 경로·bounded 집계·재현·후속 조정 준비를 닫는다.
전체 콘텐츠/Unity 실제 플레이와 최종 배포값 적정성은 U30-C/U30-P로 남긴다. 다음 순서는 U07-M→U08이다.
root는 계획/설계·조정, GPT-6.1 Sol/xHigh는 제품/포맷/빌드/시험·교정, 비참여 GPT-6 Astra/High는 독립 감사다.
원 연속 요청에 따라 P2G 이후 GEX와 finding 교정·감사·후속 작업까지 계속하며 단계 재승인을 요구하지 않는다.

## PE-CMRP-01 — 배포 기본과 trusted 선택

기존 배포 기본 profile의8MiB 포함 모든 기본 한도·기본 거절 의미를 보존한다. 개발 측정 profile은 trusted host가 세션/실행 경계에서 명시 선택하는 유한 설정이다. 콘텐츠·저장·일반 게임 명령은 모드나 한도를 올리지 못한다. 잘못된/과대/미지원 설정은 구체적으로 거절하고 기본값으로 조용히 대체하지 않는다. 플레이 중 자동 전환하지 않는다.

근거: FOLLOWUP4.1/A1. AI 위임 결정 이유: 후속 요청이 허용한 명시 개발 경로만 추가한다. 기존8MiB 실패나 모든 정상 실행을 새 한도로 재해석하지 않는다.

- 기존 설정과 생략 입력의 기본 hash·성공/거절이 유지되고 명시 개발 설정만 다른 effective identity와 동작을 갖는다.
- 잘못된/과대/비신뢰 입력 완화 시도를 정상 공개 로더/host에서 거절하고 원본을 보존한다.

## PE-CMRP-02 — 경고선·유한 비상 상한

배포 profile의 정상 retained 예산을 개발 실행의 경고 기준으로 유지하고, 별도로 검증된 유한 비상 cap만 실제 admission 상한으로 적용한다. 코드 허용 상한·선택 cap·경고선·단위·계량 범위를 분리한다. 경고를 hard fault로 먼저 걸었다가 무시하는 방식은 금지한다. 모든 관련 native retained owner/operation/result 경계가 선택 profile에 일치한다. work·clone·객체/입력·expression scratch·overflow 등 별개 보호는 유지한다.

근거: FOLLOWUP4.1/A2. AI 위임 결정 이유: 개발 cap은 측정 창 확보용 engineering 선택이며 최종 배포 적정값이 아니다. 후속 계약의 명시 허용 차이와 기본 보존을 구분한다.

- 기본/개발 각각 경고선·cap의 exact/one-over와 owner/operation/result 경계에서 의도된 성공·경고·typed 거절 차이를 입증한다.
- 다른 제한과 overflow가 개발 모드에서도 거절되며 유한 비상 cap 이상의 요청은 실행되지 않는다.

## PE-CMRP-03 — 누적 allowance·원자 fault

operation 시작 보유 extent와 이후 양의 retained growth를 누적한다. 중첩·candidate discard·rollback·해제가 같은 operation의 allowance를 초기화하거나 환급하지 않는다. 새로운 public operation만 현재 owner extent로 시작한다. 비상 cap 또는 다른 hard fault는 기존 sticky typed 오류·candidate 원자 복원·U03 세션 종료/저장 보호를 따른다. 개발 경고만 발생한 정상 실행은 fault 상태가 아니다.

근거: FOLLOWUP4.1/A3. AI 위임 결정 이유: 관측용 모드가 실행 자격이나 복원 정책을 우회하지 않게 기존 예산 owner를 재사용한다.

- 동일operation 중첩/실패candidate/재시도 누적값과 exact/부족cap, 다음operation 새 시작을 관찰한다.
- 늦은 emergency failure에 상태·queue·facts·RNG·저장 원본이 보호되고 U03 정상 fault/재접속/Continue 정책을 유지한다.

## PE-CMRP-04 — 서로 다른 메모리 지표

시작/current/최대 accounted retained extent, 누적 admitted growth와 최대 요청/초과 site, allocator live/peak/종료 후 delta와 할당 수, 지원 OS의 process RSS/peak를 구분한다. 각 값은 범위·단위·방법·지원 여부·누락/관측 overhead를 가진다. 서로 더하거나 대체해 가짜 단일 메모리 peak를 만들지 않는다. 실제 heap/RSS는 관측만 하며 게임 admission이나 결정론의 입력이 아니다.

근거: FOLLOWUP4.2/A4,A5. AI 위임 결정 이유: 현재 retained admission과 heap peak의 의미 차이를 보존한다. 측정 최대를 이론적 최대나 RSS로 과장하지 않는다.

- 정상 샘플에 누적 예산과 실제 allocator peak가 독립 필드·범위로 존재하고 시작/종료·실패 최대 요청도 추적된다.
- 지원환경 RSS는 실제 OS 측정이고 미지원/실패는 명시적 missing이며 allocator 수치로 채우지 않는다.

## PE-CMRP-05 — bounded 수집과 관측 비간섭

기존 runtime observation/host 수집을 우선 재사용하여 샘플·버퍼·파일·출력·집계와 cardinality를 유한하게 제한한다. 표본 수·드롭·누락·선택 범위를 표시한다. 평균·상위구간·최대·조건별 요약은 표본 정의와 함께 제공한다. 관측 on/off·collector 부재/포화/실패가 gameplay·영속 결과를 바꾸지 않는다. 수집 속도·RSS·벽시계로 runtime 예산을 조정하지 않는다.

근거: FOLLOWUP4.2/A4,A5. AI 위임 결정 이유: 진단 파이프의 실패를 게임 오류로 승격하지 않는다. 새 원격서비스나 dashboard를 전제로 삼지 않는다.

- normal/off/detached/full 관측 모드에서 같은 profile·seed·입력의 gameplay/저장 결과가 동일하다.
- 포화·종료·파일상한에서 bounded 크기와 drop/missing이 확인되며 원시 표본과 요약의 수량/분포가 일치한다.

## PE-CMRP-06 — profile identity·복원 권한

선택 profile의 raw/effective hash·revision을 현재 엔진/콘텐츠/실행 identity와 연결한다. 같은 profile/seed/input은 관측 모드와 무관하게 같은 결과다. replay/승인에서 다른 profile을 같은 실행으로 인수하지 않는다. 저장 데이터가 host profile을 선택하지 않으며 기존 Continue/pre-entry 복원은 현재 trusted host profile로 재검증하고 원본을 보존한다. live clone은 선택된 immutable profile과 실행 의미를 유지한다. 새로운 live-battle disk codec은 도입하지 않는다.

근거: FOLLOWUP4.1/4.2/A6. AI 위임 결정 이유: 기존 U02 effective hash와 U04 현재host 복원 계약을 따른다. 저장값으로 개발 모드가 켜지는 권한 경로를 추가하지 않는다.

- 생략/명시기본 canonical identity와 개발 profile mismatch 거절을 실제 승인/trial/replay 소비 경계에서 검증한다.
- clone·같은host reconnect·현재host의 Continue/SavePoint 복원에서 profile 선택 권한과 원본 보호가 유지된다.

## PE-CMRP-07 — 정상 Core/host 실측과 재현

현재 지원 정상 CoreApplication과 실제 host 경로에서 bounded 샘플·원시 결과·요약·재현 명령을 제공한다. engine/content/profile/build·관측모드·기기/OS·seed·입력·전투 규모/조합·동시 세션·측정 창·성공/거절/중단을 연결한다. private 직접호출만으로 정상 경로 완료를 주장하지 않는다. U07 원래PC 입력/실패는 비교 가능한 이력으로 보존하고 default/development 결과를 혼합하지 않는다.

근거: FOLLOWUP4.2/4.3/A4. AI 위임 결정 이유: 이번 준비 단계에서 현재 실제 수집 경로를 완성하되 모든 콘텐츠와 Unity 플레이 표본을 요구하지 않는다.

- 정상 전투와 생성물 소비의 실제 CoreApplication/host 표본을 동일 명령/seed/profile로 재현하고 nonzero 측정·정상 결과를 연결한다.
- 기본/개발의 경고초과·hard거절 표본과 원래PC 비교를 분리하며 측정창 밖 상태나 작은 표본의 한계를 명시한다.

## PE-CMRP-08 — 반복 실행·수명·회수

현재 지원 전투·생성/해제의 반복과 종료 후 회수를 관찰하고 정상 일시 peak와 지속 누적/누락을 구분한다. Pause·재접속·원자 rollback·동시 세션의 범위와 profile별 isolation을 검증한다. 수집기 수명은 battle/profile 자산을 불필요하게 붙잡지 않는다. 기존20분 stress 제외를 유지하며 짧고 명시된 반복/동시 표본으로 준비 경계를 닫는다.

근거: FOLLOWUP4.3/A6. AI 위임 결정 이유: 정상 반복 비용의 측정과 장시간/전종 최악부하 증명을 구분한다.

- 반복 생성/만료/전투종료 뒤 accounted owner와 실제 allocator 종료delta/host session retention을 측정해 회수 근거를 남긴다.
- 동시 다른 세션·관측포화·collector 종료·clone에서 예산 또는 profile이 다른 operation으로 새지 않는다.

## PE-CMRP-09 — 현재 검증·전체 독립 완료

10정책 요소·10필수 검증차원·181선행 조건·별도31R06의 현재 적용성을 추적한다. 기본 profile 계약은 보존하고 개발 모드에 명시 승인된 허용 범위 차이는 새 증거로 구분한다. 작은 profile/native/collector 경계부터 검증하고 영향받는 최종 Core/host·공개/저장/fault/allocator/PC/R05를 닫는다. 과거 실패/target/감사·수치oracle는 보존한다. exact 동결 뒤 비참여 Astra/High 전체 감사와 finding교정을 Complete/High/open0으로 닫기 전 U08을 시작하지 않는다.

근거: FOLLOWUPA7/current continuous request. AI 위임 결정 이유: 단계/문서 통과를 제품 완료로 표시하지 않는다. 승인된 새 devprofile을 과거8MiB 실패 면제나 default회귀 은폐에 사용하지 않는다.

- 각 조건·차원에 nonzero selector 또는 근거 있는 exact 재사용, default/dev 별도 결과·baseline·원래실패가 연결된다.
- 정역 복원 가능한 current target과 독립 전체 Complete/High/open0가 존재하며 역할·쓰기가 분리된다.

## PE-CMRP-10 — 후속 수치 조정 소유

U07-M은 측정·설정 경로 준비 완료까지만 소유한다. U08 이후 도입 owner는 계량과 표본을 유지하고 U11은 profile/승인 identity를 인수한다. U30-C는 목표 Core환경·정상 최대/동시부하와 여유로 후보를 정하고 U30-P는 실제 Unity 플레이로 배포값을 재검토한다. 변경은 새 profile/revision과 전후 raw·이유·영향 검증·독립 인수에 결속한다. 평균만으로 hardcap을 정하거나 heappeak를 누적admission값에 대입하지 않는다.

근거: FOLLOWUP3/4.3/A8. AI 위임 결정 이유: 등록된 후속 작업의 단계 소유와 Core 우선 순서를 그대로 구체화한다.

- 재현·수치조정 절차에 profile version/원시전후/영향/후속owner와 미확정 플랫폼·부하·최종값이 명시된다.
- U07-M 완료 후 U08이 진행 가능하며 최종 Unity 측정을 현재 순환 선행조건으로 삼지 않는다.

현재 [검토 본문](CODE_REVIEW_001.ko.md), [완전 batch001](P2G_20_REVIEW_001.json), [검증](P2G_20_RESULT_001.json): 81 exact source, required6 confirmed/N·A2, 10 code_reviewed와 result binding. 제품 구현/실행 성공은 아직 아니다.

현재 planning Freeze는 [FPF-CMRP-001](P2G_30_FREEZE_001.json), exact10 member/decision/acceptance/binding. [P2G-30 결과](P2G_30_RESULT_001.json). 다음은 P2G-40이다.

[P2G-50 Goal](P2G_50_GOAL_SET.ko.md) quality2 검증002 PASS(초기 owner-label 실패001 보존), [P2G-60 exact 인계](P2G_60_HANDOFF_001.json) 완료. 다음은 원 연속 요청이 별도로 승인한 GEX-00/10 GOAL-CMRP-01이다. 제품 구현/독립 감사는 아직 미수행.
