# 시스템 간 통합 검토 준비와 남은 범위 판정

작성·갱신일: 2026-10-18

**통합 검토 준비·남은 범위 판정 완료. X02 event fixture 생성기 오류 한 건을 교정·검증했다.**
현재 선정 범위의 필수 구현 잔여는 없으며 최종 통합 검토에 진입할 근거가 준비됐다.
이번 범위는 [README](README.md)의 다음 행동과 [X01 인계](X01_COPY_DETERMINISM_OBSERVATION_REVIEW.ko.md#5-분리된-완료-검토와-다음-행동)다.
[기준](../refactoring_criteria.ko.md) §3·완료 조건과 [실행 지침](../system_refactoring_execution_guide.ko.md) §6–7을 적용한다.
최종 시스템 간 통합 완료 판단은 다음 별도 검토의 대상이며 이 준비 기록으로 대체하지 않는다.

## 1 근거의 현재성

[지도 기준선](evidence/map_001/baseline.json)의 457개 입력과 현재 source/test 목록을 한 번 대조했다.
착수 시 **458개**, 추가는 B1의 `combat_reward_quantity.rs` 한 개, 누락은 0개다.
착수 시 지도 이후 달라진 **20개 경로**는 모두 이미 수행한 작업으로 설명된다.
[현재 대조](evidence/integration_ready_001/baseline.json)는 의미 전수 재조사가 아닌 통합 준비를 위한 변경 귀속 확인이다.

과거 hash 중 하나와 일치하는 것만으로 최종 상태를 판정하지 않았다.
B1→S07 R1→S01/S04 D1→S02/S11 D1→S08 R1/R2→S09 R1→S10 R1의
[구현 순서와 최종 대상](evidence/integration_ready_001/change-chain.json)을 연결했다.
수정 source 20개와 계약 2개의 **최신 구현 대상 22개**가 일치한다.
공유 `mod.rs`, `sim.rs`, `death_runtime.rs`, `build.rs`, runtime 계약의 후속 변경은 앞선 hash를 대체하지만
이전 결과의 범위·원시 로그를 지우지 않는다.

저장/승인 경계 재사용에 필요한 외부 host·계약·설정·기준 **15개**도
[선택 추가 입력](evidence/integration_ready_001/supplementary-inputs.json)에서 일치한다.
전체 공식 데이터·모든 transitive dependency·도구 환경·Unity를 동결한 근거는 아니다.
이 대조 이후 §4의 X02-B1 교정으로 generator 한 파일을 변경했다. 현재 지도 대비 source 변경 경로는 **21개**다.
[교정 대상](evidence/integration_ready_001/correction-target.json)을 별도로 식별하며 기존 baseline을 덮어쓰지 않는다.
게임 실행·테스트·공식 fixture·설정은 변경하지 않았다.

## 2 선정 후보의 종결과 설계 효과

| 소유 영역 | 종결·유지 판단과 실제 효과 | 최종 상태의 연결 |
| --- | --- | --- |
| [S07 B1](S07_B1_IMPLEMENTATION.ko.md) | 보상 후보를 표시·지급 전에 한 번 확정. 같은 배율 정의로 Continue reader 비교. 공개 흐름 8개 입력과 실패/재호출 검증 | source와 보상 reader 최종 대상 유지. 과거 불일치 저장 migration은 미포함 |
| [S07 R1](S07_COMBAT_BOUNDARY_REVIEW.ko.md#r1-구현과-완료-검토) | 승리 후처리가 기존 공통 helper를 같은 위치에서 호출. 세 연산 수동 동기화 제거 | 이후 world 정산 실행 변경 없음 |
| [S01](S01_APPLICATION_BOUNDARY_REVIEW.ko.md), [S04](S04_PERSISTENCE_BOUNDARY_REVIEW.ko.md) D1 | 직접 실행·prepared·Trial, Continue·SavePoint·opaque host 저장의 서로 다른 실패·복원 범위를 문서화 | 실행·schema 불변. 이후 D1은 전술 위치·retained settlement 설명만 교정 |
| [S03/S06](S03_S06_NODE_ACTIVITY_REVIEW.ko.md), [S05](S05_OWNERSHIP_ECONOMY_REVIEW.ko.md) | 활동·공통 지급·node completion·사본/투자·terminal owner 유지. 수명 차이를 단일 transaction/asset registry로 합치지 않음 | 공유 생성·정산의 변경은 해당 S07/S08/S10 경계에서 영향 확인 |
| [S02/S11 D1](S02_S11_CONTENT_PUBLICATION_REVIEW.ko.md) | 게시 지원을 전면 차단한다고 적힌 주석 교정. loader·Trial·approval 권위 구분 유지 | 실제 publication 동작·승인 정책 불변. X01이 compiled plan의 실제 소비를 보완 |
| [S08 R1/R2/D1](S08_BATTLE_LIFECYCLE_REVIEW.ko.md) | 유닛 생성 조립 2곳→1곳, buff 선택/정렬/제거/기록 2곳→1곳. wrapper별 deployment·정리 시점 보존 | S09 death의 항상 실행되던 경로 보존, S10은 이 생성·정리 본문을 변경하지 않음 |
| [S09 R1/D1](S09_MOVEMENT_BLOCKING_REVIEW.ko.md) | 항상 false인 private helper와 여섯 호출·불가능 분기 제거. 실제 이동 lock/segment/epoch 정책 유지 | S10의 sim 변경은 별도 whole-step 보정 합산 두 곳에 한정 |
| [S10 R1/D1](S10_ATTACK_SKILL_EFFECT_REVIEW.ko.md) | 피해 보정 합산 3곳→1곳·호출 4곳. target usefulness와 실제 executor의 규칙 동기화 부담 제거 | source/carrier·queue·정산·예산 순서 유지. X01 식 평가·관측 조사로 소비 경계 연결 |
| [X01 실행 제어·자원](X01_RUNTIME_RESOURCE_REVIEW.ko.md), [수치식](X01_VALUE_EXPRESSION_REVIEW.ko.md), [복사·결정론·관측](X01_COPY_DETERMINISM_OBSERVATION_REVIEW.ko.md) | 예산·lease·값·실행 cursor·기록 재생·관측 owner 유지. 근거 없는 범용화/복사 제거 미선정 | 기존 실패 후 retry·retained lifetime 근거와 관측 feature 13개 집중 결과 연결 |

선행 선정 개선에서 미구현 후보나 미종결 실패를 발견하지 않았다. 이번에 새로 확인한 X02-B1도 §4에서 종결했다.
후속 source diff·구조 대조·집중 로그는 각각 S08 R2, S09 R1, S10 R1 기록이 소유한다.
동일 작업자의 분리 검토이며 비참여 독립 감사로 확대하지 않는다.
성능 측정 없이 위 구조 변경을 실행 시간·메모리 개선으로 보고하지 않는다.

## 3 통합 검토에 넘기는 연결 조건

| 연결 | 보존할 권위·실패 조건 | 기존 근거와 남은 확인 수준 |
| --- | --- | --- |
| S02/S11 → S01 → S07/X01/S10 | generation A와 B, host safety identity, compiled plan/capture를 실제 실행까지 유지 | external B start의 실제 battle/resource/export와 A pin, B 준비 실패 후 A 유지, profile별 parse/compile/evaluate, X01 실제 HP·record 재생. 정상 승인 A/B 전체 재인수와 구분 |
| S05 → S07 → S08/S10 | 사본/장착·resource source가 배치·시전·전달·철수/재배치까지 같은 owner에 속함 | S05 사본/투자, S08 외부 RON 여섯 자원 수명과 재배치, X01 lease 검사. 모든 장비/유물 조합 전수는 아님 |
| S08/S09/S10 → S07 → S06/S03/S05 | terminal 후보 확정·최종 보상 옵션·receipt를 한 번 게시하고 ACK/claim/Exit의 정책을 구분 | S07 수량·live terminal/Continue, R2 death/withdraw, S09 epoch·event 순서, S10 whole-step 실제 HP와 timestamp 실패 retry. 개별 성공을 하나의 자연 플레이 실행으로 합치지 않음 |
| S01/S04 ↔ S03/S05/S07/X01 | accepted prepared commit, stable Continue·SavePoint·entry restore, ID/소비/투자와 다음 행동 | S04 Core 8개·host 10개와 프로세스 종료 16지점, B1 새 application 복원, X01 RNG/후퇴. host opaque payload와 Core 의미 검증을 구분 |
| S11/X02 → 실제 실행·관측 | 제작 도구는 제품 API를 소비하고 optional trace/observer 실패가 gameplay 권위가 되지 않음 | Trial 실제 경로·trace 동일성, CLI 인자 검사, world trace export 실패 정산, 관측 queue 포화/collector 종료·prepared 폐기. CLI/fixture 현황은 §4 |

[재사용 원시 행 35건](evidence/integration_ready_001/reused-tests.json)을 정확한 로그·줄과 연결했다.
동일 테스트의 서로 다른 변경 시점 기록이 포함되므로 35개의 독립 테스트나 이번 실행 수로 집계하지 않는다.
기존 전수 **2,236개 통과·ignore 6**은 S08 R1 뒤, R2/S09/S10 전 결과다.
그 뒤 영향 검증 **18/17/13개**, 관측 feature **13개**와 구조 대조를 연결하며 최종 source에서 전수를 다시 통과했다고 쓰지 않는다.

## 4 남아 있던 X01/X02 범위 판정

- `lib.rs`/`game/mod.rs`는 module visibility를 선언하며 별도 실행 정책을 소유하지 않는다.
  world는 crate 내부, 컨테이너는 private, 관측은 feature 경계다. public 하위 타입 전체를 숨기는 변경은 실제 지원 API를
  바꾸므로 이번 조사에서 일괄 축소하지 않는다. bare data 생성은 test/debug_assertions/test-fixtures gate와 구분한다.
- `managers/mod.rs`의 manager는 `uuid_manager` 하나다. 앞 X01에서 peek/stream/복원·clone을 확인했으므로
  이름만으로 새 미조사 manager 묶음을 만들지 않는다.
- `runtime_safety`는 파일/환경을 직접 읽지 않는 immutable typed profile이다. embedded RON이 기본값,
  각 engine이 ceiling을 소유한다. content semantics와 admission identity를 구분하며 profile은 generation에 유지된다.
  외부 pack→선택 profile→parse/compile/plan/evaluate, 첫 battle operation, Trial admission의 기존 시험 15개를 재사용한다.
  임의 설정 주입이나 다른 profile의 plan/meter를 암묵 허용하는 새 통합 계층은 필요하지 않다.
- `combat_trace`는 debug/test/fixture 실행의 진단 파일이다. 실제 정산 caller가 export 오류를 경고로 처리한 뒤 진행한다.
  디렉터리를 파일로 막은 기존 시험은 실제 I/O 실패 뒤 CombatResult·receipt를 확인한다.
  정상 trace는 bounded event history를 내보내며 durable 결과의 `has_event_log`와 동일시하지 않는다.
  debug I/O 지연·flush 내구성·서버 exporter는 gameplay 완료의 근거로 삼지 않는다.
- `author_content`는 입력 한도·operation 선택·파일 출력을 소유하며 Trial API를 사용한다.
  `package_official_content`/`consume_content_pack`은 실제 package/load와 공개 application을 소비한다.
  기존 CLI 인자 4개와 library/Trial 결과는 재사용하되, 기본 bin의 `running 0 tests`를 CLI 성공 시나리오로 세지 않는다.
- `tests/common`의 bare 구성·embedded RON과 실제 external pack fixture는 준비 경로가 다르다.
  기존 보고서의 internal/admin setup·공개 경로 구분을 유지하고 fixture 자체를 별도 게임 엔진으로 해석하지 않는다.
- 네 `generate_*_contract_fixture` bin은 `test-fixtures`가 필요하여 기본 전수 실행의 대상이 아니다.
  inventory/shop은 typed DTO 예시, compendium은 embedded catalog 기반 예시, event는 실제 CoreApplication의
  start/briefing/starter/admin event/choice/snapshot 흐름이다. 생성 결과가 Unity 소비까지 증명하지는 않는다.

### X02-B1 — event fixture의 폐기된 시작 입력 교정

[네 bin 집중 빌드](evidence/integration_ready_001/commands.json)는 283.12초에 성공했다.
초기 실행에서 compendium/inventory/shop은 성공했지만 event는 exit 101로 실패했다.
[원시 실패](evidence/integration_ready_001/generate_event_contract_fixture.stderr)는 `Standard`를 거절한 실제 공개 command decode다.
현행 runtime 계약은 `Tutorial`/`Expedition`만 받으며 starter 선택에는 `loan_copy_ids`가 필수다.
생성기는 폐기된 mode 문자열과 누락된 대여 배열을 가진 옛 시작 절차를 사용했다.
후자는 source/현재 필수 필드 대조로 확인한 인접 결함이며 별도의 실패 실행을 했다고 주장하지 않는다.

[교정 diff](evidence/integration_ready_001/source.diff)는 한 파일의 세 지점에 한정된다.
`GameMode` import, `GameMode::Expedition` 직렬화, `loan_copy_ids: []` 명시다.
seed·fixture event·직원 정의 선택·실제 선택 명령·출력 schema를 유지한다.
mode는 임의 문자열 대신 기존 enum을 소비하므로 폐기된 variant를 조용히 유지할 여지도 줄였다.
새 bootstrap 추상 계층이나 테스트 전용 실행기를 추가하지 않았다.

[해당 bin만 재빌드·재실행](evidence/integration_ready_001/correction-command.json)했다.
[결과](evidence/integration_ready_001/correction-result.json)는 build **31.17초**, 실행 **5.32초**, 각각 exit 0이다.
세 성공 generator는 수정하지 않았으므로 반복 실행하지 않았다. compiler warning은 library 36개다.

[출력 대조](evidence/integration_ready_001/fixture-output-validation.json)에서 다음을 확인했다.

- inventory: 사본 ID, 슬롯 60/점유 2, 소비품 수량 7.
- shop: 동일 상품의 수량 3·총가격 75·단가 25, 판매 가능 7·견적 10, refresh 횟수 관계.
- compendium: 등록 항목의 metadata와 미등록 항목의 숨김 표현.
- event: 선택 전/후 같은 node, eligible 직원·장비 key evidence, 실제 roll 13 + modifier 2 = 15와 DC 16,
  그에 맞는 `failure` scene·후속 advance. 이벤트 판정 실패는 정상 fixture 결과이며 프로세스 실패와 다르다.

최종적으로 **네 generator 실행·JSON·선택 출력 조건 확인 완료**다.
이는 네 테스트 추가나 Unity 소비 검증이 아니다. 실행 중인 명령·미해결 실패는 없다.
[발견·처리 기록](evidence/integration_ready_001/finding.json)에 초기 실패와 교정을 함께 보존했다.

## 5 필수 잔여와 비필수·범위 밖

| 분류 | 처리 |
| --- | --- |
| 현재 준비 단계의 필수 잔여 | 없음. X02-B1 교정 후 네 feature fixture의 실행·출력 조건 확인 완료 |
| 전체 완료 전 필수 | §3의 연결 조건과 최종 변경을 별도 읽기 전용 통합 검토에서 대조하고 README §4 완료 조건을 판정 |
| 비필수 보류 | 중첩 store/transaction 복사·장기 queue/cache·최대 규모 비용. 측정과 실패 계약을 정한 뒤 별도 판단 |
| 알려진 검증 제한 | comparator fixture 24쌍/31 variant, 모든 스킬/활동/성장/저장 조합, 전체 feature/release 동등성. 이번 국소 변경에서 새로운 공백으로 확인된 것과 구분 |
| 외부·별도 관리 범위 | Unity 실행, server 전체·승인 cache/worker/calibration A/B matrix, 장치 전원 손실. 기본 전수의 기존 ignore 6개를 통과로 처리하지 않음 |
| 의도적으로 미포함 | B1 이전 불일치 저장 migration, 임의 public 내부 상태 변조 방어, 전면 API 비공개화·crate 분리 |

모든 branch를 조사하지 않았다는 이유만으로 전체를 다시 열지 않는다. 동시에 명시된 필수 연결의 미확인을
단순 범위 밖으로 옮겨 완료할 수는 없다. 다음 검토에서 실제 연결 공백이 발견되면 해당 owner와 좁은 검증으로 돌린다.

## 6 완료 검토와 다음 행동

후속 [최종 통합 검토](FINAL_INTEGRATION_REVIEW.ko.md)를 완료했다. 아래는 준비 단계 종료 당시의 인계다.

구현·실행 종료 뒤 별도 읽기 전용 pass에서 현재 source·계약·diff·초기 실패와 교정 결과를 대조했다.
[검토 기록](evidence/integration_ready_001/review.json), [문서·근거 확인](evidence/integration_ready_001/verification.json)을 연결한다.
같은 작성자의 검토이며 독립 감사는 아니다. 판정은 **준비 완료**, 전체 Core 완료는 아직 선언하지 않는다.
다음은 **선택한 시스템 대표 흐름과 개선 결과의 최종 통합 검토**다.
이 기록 §2–3 → 최신 source diff와 해당 owner 계약 → 원시 검증 근거 순서로 확인한다.
이미 고정·대조한 458개 입력을 이유 없이 재해시하거나 전수 테스트를 반복하지 않는다.
실제 새 변경·모순·필수 공백에만 확인을 배정한다.
결과에는 전체 완료 가능 여부, 필수 미해결, 비필수 제한과 재개 조건을 구분해 README·지도에 반영한다.
