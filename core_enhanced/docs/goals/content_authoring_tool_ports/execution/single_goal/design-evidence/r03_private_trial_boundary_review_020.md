# R03 private trial boundary — bounded source review 020

완료 감사가 아니다. INPUTS_061에 고정된 private initial/one-step 경계, 직접 호출하는 public ports와 resolver/output/control의 관련 부분만 읽었다. 공개 prepare/simulate, 실제 battle readiness, 전체 native clone visitor·heap proof 및 optional trace 구현 여부는 판정하지 않았다. 소스·문서·Git 변경, cargo/build/test 실행은 하지 않았다. Root가 전달한 focused080 compile 실패/수정 및 focused081 실행 중이라는 정보는 독립 시험 결과가 아니다.

초기 capture: 2026-09-10T11:13:04.563658+00:00. clone_shape 추가 capture: 2026-09-10T11:17:20.563532+00:00. 마지막 해시와 drift는 동반 JSON에 기록한다. 읽은 8개 소스의 SHA는 INPUTS_061의 해당 entry와 일치했다. receipt 구현은 output.rs이며 탐색한 authoring/receipt.rs는 존재하지 않는다.

## PT020-F01 — 최초 public projection은 trial의 output admission보다 먼저 할당된다

**분류: 중간 중요도 integration gap.** 새 private 경계의 atomic discard 결함과 구분한다. trial.rs 255–261은 execute/snapshot이 반환한 owned result를 받은 후 public_value를 호출한다. public_value(368–379)는 encode_result의 제한 writer를 통과한 문자열 길이만큼 parse work를 먼저 charge하고 두 번째 Value를 만든다. 그러나 application.rs 887–923의 snapshot은 그 전에 serde_json::to_value로 PlayerProfile/EncounterCompendium 등을 owned Value로 만든다. battle result 경로도 application.rs 355–385, 418–430의 behavior_variant_payload에서 먼저 serde_json::to_value(result)를 실행한다. 공개 API 자체의 기존 동작이며, trial.rs에 새로 무제한 parser가 들어간 문제는 아니다.

따라서 max_result_bytes를 낮춰도 최초 projection의 traversal/allocation은 이미 실행된 뒤다. whole application fork admission은 native clone 비용이고 이 JSON projection의 bytes/work preadmission을 대신하지 않는다. allocation이 실제 결과 크기에 비례한다는 구체 경로는 확인했지만 최대 RSS 또는 무제한 공격을 입증한 것은 아니다. 초기 기본 profile도 작은 고정 객체이므로 이 지적을 초기 생성의 무제한 메모리 문제로 확대하지 않는다.

**최소 후속 후보:** 실제 public result projection/각 snapshot producer에 active-scope counting Serialize 또는 bounded writer 경로를 연결해 owned JSON 생성 전에 source를 계수/charge한다. ordinary public 결과·disposition 의미는 유지하고 실패는 sticky control을 통해 enclosing candidate discard에 전달한다. 이미 owned ApplicationResult를 다시 검사하는 것만으로 이 최초 할당 경계를 닫았다고 표시하지 않는다. 해당 producer 연결 전에는 이번 구현의 보장을 “bounded secondary encoding/accepted Value materialization”으로 제한한다.

관련 SHA: trial.rs `2603eb5c2a05b141529f2c570f309b50bfa089771f430a3be959c1c62d474faa`; application.rs `6e77a86f121fd092c83b68a6021e86a99c2d24d657fd8a7d18247f5c81e54f8e`; output.rs `d56c244833316a3049409cdd85bc8a203748f79d57b15788ebe484ed2b588e5e`.

## 확인한 경계

- **한 번의 성공 swap:** trial.rs 190–204에서 committed ledger를 새 TLS attempt에 복사하고 prepare_step은 &self만 받는다. fork_for_authoring(application.rs 786–795)는 실제 GameCore controlled_clone을 사용한다. raw core/public snapshot 대체가 아니다. inner Err는 candidate?에서 반환되어 app/ledger/records/copy/receipt/work를 바꾸지 않는다. outer control Err도 동일하다. 성공한 경우만 반환된 ledger·usage를 candidate에 넣고 *self = candidate 한다. control.rs 215–251의 마지막 acceptance poll 뒤에는 계수 작업이 없다. 실패 시 private locals와 임시 Arc refcount 증가는 drop된다. 전체 GameCore visitor의 transitive 정확성은 이번 범위에서 재검증하지 않았다.
- **초기 provenance/lifetime:** initialize 103–170은 새 메모리 application을 fresh provenance scope 안에서 만든다. 완료 시 실제 candidate ledger로 덮어쓰므로 임시 fresh 값이 변경 이력을 지우지 않는다. content/scenario는 owned immutable Arc이며 이전 record는 private Arc<AcceptedStep>이다. copy는 clone_shape.rs 152–157에서 handle만 계수한다. accepted 기록을 mutable getter로 노출하지 않는다.
- **유한 index·result slot:** scenario.rs 437–449는 preparation+execution 합을 max_steps 이내로 제한한다. trial.rs 218–235의 ref Vec는 3*index+1을 checked 계산, admission·charge한 뒤 할당한다. filter가 없어 assertion/HostTick(None)의 None도 원래 step index를 차지한다. resolver.rs 80–122, 150–164, 232–248은 index와 available source를 확인한다. receipts/validation metadata는 binding 입력이 아니다.
- **prefix allocation 순서:** trial.rs 295–308은 Vec record 1 + next_len element slots + existing/new Arc handle 총 2*next_len+1을 하나의 meter에 admission/scan/copy charge 후 with_capacity(next_len)/extend 한다. content/scenario/initial receipt handle도 309–311에서 별도로 계수한다. 새 record Arc owner는 final encoding 전 생성되고, 완료 후 push는 예약 capacity 안이다. Arc::get_mut의 unique-owner 전제는 이 코드 안에서 유지된다.
- **mandatory encoding/header:** output.rs 32–52는 각 chunk를 ResultBytes/work admission 후 append한다. trial의 public_value는 그 bounded 문자열의 byte work를 charge한 뒤 parse한다. output.rs 70–125는 고정 폭 usage prefix를 먼저 예약하고 payload·종료문자·header fill work까지 계수한 뒤 usage를 읽는다. trial.rs 348–363은 그 뒤 추가 control 계수를 하지 않는다. receipt owner나 prefix Vec capacity를 뒤늦게 할당하지 않는다. 이전 receipt byte 합과 새 문서 길이는 ceiling 검사로 64 MiB 이내라 total_bytes 덧셈이 안전하다.
- **fuel/precedence:** trial.rs 35–64가 original ExecutionLimits를 먼저 validate한 뒤 total work 잔여량으로 clip한다. exhausted 상태에서 invalid u64::MAX가 유효한 작은 limit로 바뀌지 않는다. used>=total은 전용 오류로 종료하며 max_work=0 scope를 만들지 않는다. 각 scope usage.work는 checked charge와 effective.max_work 이하(control.rs 473–485), total ceiling은 2^38 이하이므로 accepted_work+usage.work가 overflow하지 않는다. validate의 pub(super) 확대는 authoring 내부만 허용하며 no-scope gameplay 동작을 바꾸지 않는다.
- **의미/오류:** 네 branch 모두 실제 public command/snapshot/clock 또는 bounded assertion resolver를 호출한다. native host tick의 Option 결과를 그대로 유지한다. App domain Err는 outer Ok이어도 commit되지 않는다. sticky control 오류는 run_with_provenance의 acceptance 검사에서 임시 Request/Application 오류보다 우선한다. 선택적 trace는 이 경로에 참여하지 않는다.

## 테스트 oracle의 실제 범위

trial.rs 462–524의 첫 검사는 실제 StartNewGame 결과를 만든 뒤 receipt cap 한 바이트 부족으로 실패하고 A의 Run snapshot, 빈 records, accepted_work, initial receipt pointer를 비교한다. 충분한 limit로 재시도한 결과/receipt를 독립 초기 fixture와 비교하고 step work exact/one-under도 검사한다. header의 work와 prior_accepted_work를 직접 비교하므로 뒤늦은 계수가 생기면 잡을 수 있다. output.rs 203–319의 별도 tests는 전체 usage 객체·고정 header 폭·receipt 누적 bytes·마지막 cancellation을 다룬다. 이들은 읽은 test source이지 이번 helper가 실행한 결과가 아니다.

trial.rs 527–574는 command → assertion(None) → snapshot(Some) 뒤 실제 clock domain Err와 이미 취소된 token을 검증하며 immutable first record pointer를 보존한다. 다음 제한은 별도 시험 보강 후보이며 현재 구현 오류로 판정하지 않는다.

1. binding 없는 fixture라 copied_bytes=0 보존은 양수 quota commit/rollback을 증명하지 않는다. 이전 public 값 binding을 포함한 성공/receipt 실패/retry가 필요하다.
2. 실제 Monolith insertion/import가 없는 fixture라 candidate ledger 변경의 commit/discard를 직접 관찰하지 않는다. 변한 provenance를 만드는 실제 command 후 late receipt 실패 및 retry에서 visitor bound를 비교하면 이 경계를 검증할 수 있다.
3. clock은 battle 부재 오류만 통과한다. 성공 HostTick(None)이 slot을 남기고 이후 index가 어긋나지 않는지는 현재 trial fixture에 없다.
4. preparation_steps가 비어 있어 preparation→execution 경계, 성공한 마지막 step 다음 finished, 초기 receipt의 전체 final usage, 양수 total-work remainder의 exact/one-under clip은 trial-level 직접 oracle가 없다. 이미 있는 output 단위 시험과 구분해야 한다.

위 integration gap 외에 이 제한 source slice에서 구체적인 rollback/header/result-slot/Arc 수명 또는 산술 결함은 찾지 못했다. 이는 공개 readiness나 전체 Goal 승인 결론이 아니다.

## 읽은 소스와 정확한 SHA-256

- core_enhanced/src/authoring/trial.rs: `2603eb5c2a05b141529f2c570f309b50bfa089771f430a3be959c1c62d474faa`; /files/14; 읽은 범위 1–575.
- core_enhanced/src/authoring/control.rs: `1a15e8bdd7618572950ddc69ad25b5cda674eebfecd0f544010abb3477b2fed8`; /files/6; 읽은 범위 1–489, 490–572.
- core_enhanced/src/authoring/scenario.rs: `0152fd9a1f33ad5d061102da34b5bfc13f8e282b697f4e0030dcb741320da859`; /files/12; 읽은 범위 1–302, 399–455.
- core_enhanced/src/authoring/resolve.rs: `cb5665565b5680d5f7e509c0630f315633e91596e77b6973a2f9c91ca1c01ef3`; /files/11; 읽은 범위 1–325.
- core_enhanced/src/authoring/output.rs: `d56c244833316a3049409cdd85bc8a203748f79d57b15788ebe484ed2b588e5e`; /files/9; 읽은 범위 1–346.
- core_enhanced/src/authoring/mod.rs: `ac6470f58ad965e6ba7b0c6e10202b362a44a8b7bcb573997616dd646ce5bad2`; /files/8; 읽은 범위 1–130.
- core_enhanced/src/application.rs: `6e77a86f121fd092c83b68a6021e86a99c2d24d657fd8a7d18247f5c81e54f8e`; /files/3; 읽은 범위 1–45, 317–443, 765–970.
- core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/evidence/INPUTS_061.json: `0a4fd2b56a8a1ffa23242cf0383600682c5f0ae1c47594528597400231ed4d89`; manifest 자체; 읽은 범위 JSON entries.
- core_enhanced/src/authoring/clone_shape.rs: `6e56d3990b14357c98663ace941fc44b6cde7c41667829b4dddec78e71adbc6b`; /files/4; 읽은 범위 1–174.

최종 rehash: 2026-09-10T11:20:47.119925+00:00; drift 1개. 후속 source 변경은 이 검토에 포함되지 않는다.
