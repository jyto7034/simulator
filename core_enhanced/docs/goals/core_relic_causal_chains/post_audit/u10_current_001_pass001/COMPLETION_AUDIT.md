# U10 전체 Goal 독립 감사 — Not Complete / High / open 2

GOAL-CRCE-01 / GS-CRCE-001, frozen target `u10_current_001`을 전체 범위로 검토했다. 28개 정책 문장, 19개 통합 경계, 10개 필수 차원, 3개 실행 node와 선행 288행 모두 검토했으며, 두 증거 공백 때문에 Complete/High/open0를 부여하지 않는다. 두 finding은 실제 제품 오동작을 입증한 결과가 아니다.

감사자는 사전 확인된 gpt-6-astra/high이며 U09 독립 감사 이력만 있다. U10 계획·설계·제품·시험 작성·실행에 참여하지 않았다. 이번에는 소스·문서·oracle·실행 원문·receipt·재사용과 선행 영향을 독립 검토하고 read-only 해시/원시 데이터 대조를 했다. 제품·시험·도구·기존 문서·target을 수정하거나 build/test/collector/restore를 실행하지 않았다. 새 감사 출력만 작성했다.

## 열린 항목과 담당

- **U10-AUD-F01 / PE-CRCE-07.1:** 정상 기본 공격의 장비 반격과 active causal subscriber가 같은 실제 전투에서 공존하는 선정 실행이 없다. ancestry_active의10+7=17은 기본 공격과 subscriber만 검증하고, equipment death/reflection 시험은 relic 없는 별도 fixture다. 두 성공을 합쳐 공존을 입증할 수 없다. 독립 시험 Astra가 정상 published relic + 실제 basic producer + defender counter를 함께 구성하고 HP/source/count, 반사 재귀·기본 proc 재생성 없음, dead/missing attacker 경계를 검증해야 한다.
- **U10-AUD-F02 / PE-CRCE-08.2:** 실제 Corrosion과 active causal owner를 함께 실행한 검증이 없다. 기존 Corrosion fixture는 skills/fragments/equipment만 만들므로 새 `causal.is_some()` 경로와 suppression-carrying causal root가 실행되지 않는다. 독립 시험 Astra는 normal kill의 Death/Kill/AllyDeath queued/delayed 반응을 positive control로 먼저 입증한 뒤, 같은 active relic을 둔 실제 Corrosion에서 필수 death/wave/facts/cost 유지와 선택 반응 억제·해당 derived context 보존을 확인해야 한다.

Root가 시험 소유권과 범위를 조정하고 Sol이 고정 입력의 정확 selector를 실행한다. 실제 실패가 발생할 때만 제품 담당에게 교정한다. source/raw/coverage/선행 영향 결속 및 successor freeze 후 독립 재감사가 필요하다. 기존 실패 target과 이 감사는 보존한다. 자세한 위치·원문 reference·닫기 조건은 [FINDINGS.json](FINDINGS.json)에 있다.

## 수용한 구현과 실제 증거

Immutable ancestry의 copy UUID+authored rule 방문, 형제 Fork 격리, 실제 참가자 Join union 및 취소/receipt 정산, 독립 Periodic fresh root와 파생 지연 ancestry, 실제 HP-loss source/killer, lifetime/clone/rollback을 소스와 독립 oracle에서 검토했다. pending timestamp drain slice는 조기 Join/Wedge 정리를 막고, selected immutable program 소유는 generation/actor/kind/정확 plan binding을 유지한다. Wedge는 normalized signed 프로그램의 공통 산술에 n×r을 합쳐 단일 절삭한다.110/120/260, numeric162, structural68/76, same-strike220 대 distinct230과 방출 후230 등의 실제 결과가 이를 지지한다. 26개 문장은 기록한 범위에서 수용하며 두 조합 문장만 열린다.

선정94행의 raw block과 source/receipt, 서로 다른 실행6개의 archive membership을 확인했다.94는41+4+2+1+46의 명시적 재사용 합계로 전체 fresh suite가 아니다. 최종46은46PASS/0FAIL/0ignored이며 이후 Rust/config/fixture 변경이 없다. 과거 selected PASS가 포함된 enclosing101과 기존 실패 이력은 그대로 유지한다. 전체 suite를 다시 돌리지 않았다는 사실 자체를 finding으로 삼지 않았다.

외부 immutable31 입력의4mode HP100, Periodic8회/비용24, fresh Continue와 위조/changed-generation 거부를 검토했다. 원step20와33 비교 실패는 reward/node 진행을 누락한 시험 오류였고, 실제 같은 mode의step25 입장 전 snapshot과26/33 전체동등성으로 고친 근거를 수용한다. 원실패와 step20 fresh-process 검증은 보존됐다.

PC strict002의 원FAIL은 유지한다. 새 비교기는 empty state inline8B 및 실제 clone당 work2/slot1만 family의 고정 복제수로 분리한다. 독립 read-only 재계산에서576개 모든 non-time row가 즉시선행 U07M과 일치했고36조건31성공/5거부, 기존8MiB 실험한도, U06의200/source136과43개 실제 negative 거부가 유지됐다. 새로운 allocator/gameplay/evaluator 회귀를 숨기는 허용폭은 발견하지 못했다. 일반 입력 전체의 무비용 보장은 아니다.

R05 원실행101은 concurrent peak9,703,429>9,030,188의 기존 경고로 보존됐다. summary0은 기존 peak-only warning 판정이며 전체 원실행PASS가 아니다. 다른 hard guard 실패0, parity/repeatability true이고21 latency/5 allocator×2를 읽었다. P95는 공유환경 관측이며 production latency 보장이 아니다.

사용자가 위임한 실측 기반 자원 판단 안에서 native retained 기본값과 ENGINE_MAX 두 값의8→16MiB 변경을 수용한다. 원legacy8 actor_witness8,611,169 거부/rollback과 current16 devfalse 성공이 같은 입력에 결속됐다. 누적 admission10,762,211, 최대 단일 owner3,110,892, parent heap17,954,703은 다른 지표다.002 plain/allocator를003 새 측정으로 표현하지 않으며 최종 ledger128도0으로 표현하지 않는다. explicit authoring_dev8hard, current16, legacy8+trusted16 및 개발32의 설명을 구분했다.

## 선행 영향과 복원·현재성

288개 모든 원문/ID를 보존한 [PREDECESSOR_REVIEW.json](PREDECESSOR_REVIEW.json)은 기존 의미와 새 소비자 영향을 분리한다.254개 inherited와 U09 추가34의 원문/ID 결속을 확인했다. 이전 U09 current038/pass002의 유효 Complete/High/open0는 유지된다. 현재 Corrosion12행의 새 causal suppression 확장은 F02로 한정하고, 완료16행은 이번 open2 때문에 현재 완료를 주장하지 않는다. 나머지260행은 기록한 범위의 재사용과 현재 소비자 증거를 수용한다.

MANIFEST SHA256 `c24c6e90c48b81f432deeaf78258bb6b0e5110bd5b770d0851927c4b37961c95`, runtime scanner532 SHA256 `02e185fad4820f3366c465af9904284fc33394d5f6d59ec65e0dc33710d329ac`를 감사 대상으로 고정했다. 복원 membership은 별도의 before497→after542이며85변경/45추가/0삭제다. 실제 격리 정·역복원 receipt와 결과를 읽었고, before497/after542/records1668/references28 네 archive의 모든 exact member·size·SHA를 독립 확인했다. workspace reverse는 없다. 경로 alias 교정은 두 hunk의 canonicalization이며 원실패·tool preimage와 safe path/exact membership 가드는 보존됐다. outside freeze002 receipt/stdout/stderr도 이 감사에서 추가로 결속한다.

입구2360개 exact size/SHA 확인은 mismatch0이었다. 동일 방식의 출구 확인은 [EXIT_CURRENTNESS.json](EXIT_CURRENTNESS.json), 모든 감사 산출과 정확한 최종 판정의 결속은 [AUDIT_CLOSURE.json](AUDIT_CLOSURE.json)에 기록한다. [CONDITION_REVIEW.json](CONDITION_REVIEW.json)은 모든 문장·통합·차원·node별 판단을, [EVIDENCE_REVIEW.json](EVIDENCE_REVIEW.json)은 실제 결과/재사용/비용/복원 범위를 제공한다. 전체 검토는 끝났지만 Goal 완료는 두 finding의 보완과 독립 재판정 전까지 열려 있다.
