# U16-2 실행 계획 — GOAL-CBRR-01

2026-10-15 사용자의 `CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md 문서를 지침 삼아 u16-2 작업에 착수해줄래?` 요청으로 P2G bounded preparation 후 GEX00→10을 수행한다. P2G2026-10-04-r8 / GEX2026-10-04-gex-r8 / 현재 연속 실행 지시서를 적용한다. 중간 단계검토 opt-in 없음. 구현·자체검증·최종 복원 가능한 target 고정 후 감사 요청 대기에서 종료한다. 자동감사/서브에이전트 없음.

Upstream: ../../P2G_60_HANDOFF_001.json, GS-CBRR-001/PHASE-CBRR-01/GOAL-CBRR-01, FPF-CBRR-001, PP-CBRR-002. 수용17/차원10/IVA5, 실행노드 AG-CBRR-RUN·WP-CBRR-VERIFY.

초기 predecessor는 U16-1 independent001 Complete/High/open0, u16_1_current_001. 같은 실행자가 감사 후 제품을 변경하지 않았고 새 writer 징후 없음. 기존2188 Core고유/4native/31server와 실제host/비용은 입력동일/비영향 범위만 재사용한다. BASELINE_SOURCE_001.*에 구현 전 source/test/data/config/계약의 기존dirty를 보존하며 전체 reset/stash/patch재적용 없음.

## 구현과 상태 소유

1. layout에 typed 후보/가중치/가루 누적 임계치. 정상 loader cross-reference와 불법 작성 거절. 현재 공식 유물catalog는 비어 있으므로 빈pool 유효, 대표 효과는 기존 지원 fixture를 사용한다.
2. 기존 RewardSessionState에 optional activity state: 종류, 투자등급/누적가루, 정규화 합성선택. 기존 일반/전투 보상은 None. 시설 활동 지원/입장, 단순선택·투자·합성명령, 공통 grant·노드완료를 동시에 연결.
3. DTO에 현재 후보의 실제 유물효과·현재 등급/누적지불·다음등급/차액/불가이유·선택입력/확률/확정자격. future candidates 미공개. 입력사본별 자격과 입장 전 조건을 제공한다. 정상 command metadata/capability/application/host 모두 연결한다.
4. Claim/융합 성공은 같은 transaction에서 비용/사본/노드완료. 같은프로세스 조회는현재세션, 새프로세스 미완료Continue는입장전, 완료Continue는결과보존. SavePoint와실제IO실패까지 검증.

가루 초기누적3/9/18, 후보대표가중치1은 CF-D06 위임 초기값이다. common/rare/special은 이 활동 pool의 지원등급이며 다른 기존 유물 rarity를 전역으로 제한하지 않는다. allow_duplicates는 장착제한, 보유 종류를 추가 제외하지 않는다. 무관 기반개선/리팩토링은 필요하지 않다.

## 검증 준비와 실행 순서

먼저 routed Core 테스트로 세 활동의 실제 전이/반환형태를 확인한다. 초기 자원·사본 준비는 내부 fixture와 정상 host를 구별한다. 공개경로는 정상 external pack의 Event grant 또는 상점 구매로 입장 전에 가루와 입력사본을 마련하며, 세 활동/SavePoint를 예약한 유한 경로를 사용한다. 지원 진행기에 행동 전후 자원/노드/세션을 assertion하고 예상밖 전이는 건너뛰지 않는다. 작은 pilot로 입장·투자·확정과 fresh Continue 복구 도달을 확인한 뒤 확장 실행한다.

- 선택:3/2/1/0개, 서로다른종류, 동일조회/선택변경, 미이용퇴장, 중복수령거절.
- 투자:잔량2/3경계·3→9차액6→18차액9, 단계교체/옛선택거절, 후보0차단/기존권리보존, 최고등급추가투입거절, 최종1회지급.
- 합성:6개확률표와입력순서, 0%불가능, 다른사본같은종류허용/동일사본거절, 장착/없는입력거절, 침식입력/Normal출력, 투입종류제외/다른보유종류허용, 양수확률각등급후보필수, 사전표시와실제원자확정.
- 저장:같은프로세스현재진행, fresh-process 미완료/완료 Continue, SavePoint 당시비용/결과, 실제host 저장OS실패 후 메모리/active/profile 불변과 정상재시도. 목표IO전 자격충족을확인한다.
- 작성:정상pack과누락/중복/등급/가중치0/합계overflow/임계치오류. 외부임의변조는제외.
- 영향:공통reward/session·map/loader·command契約·clone/Continue·grant·native/server 경로. 최종 소스가 고정된 뒤 필요한통합1회, 실패/수정의영향만재실행.
- 비용:정상/확장 후보와사본을 고정profile에서측정. 내부work/retained와실제시간/heap/RSS/회수를 구별하며16MiB한도를자동상향하지않는다. 기존측정runner재사용.

실행 중 제품/시험/fixture/config입력 고정. 빌드와 비용 실행은 기존실행시간에 따라 수분이 걸릴 수 있으며 최초60초에 출력/CPU/로그로진단한다. 무출력만으로중단하지않는다. 전략전제가깨지면소유세션을정리하고원시실패를보존한뒤교정한다.

## 역할과 잔여

STATE_INVENTORY.md: 상태수명/경계. EXPERIMENTS.md: 명령·원시결과·교정·재사용. EXPERIMENT_NOTES.md: 기술판단. COVERAGE.json:17수용/10차원/5IVA/2실행node. 최종 target과정역복원/구현검토 뒤 GEX10 exit 인계. 제품 구현과 자체검증을 마쳤다. 17수용/10차원/5IVA/2실행노드의 직접 근거는 COVERAGE.json, 비용·한도 판단은 COST_REVIEW.json, 새 실행과 비영향 재사용은 BASELINE_REUSE.json을 따른다. 마지막 target 정역복원 성공 후 GEX10 exit를 기록하고 감사 요청 대기에서 종료한다.
