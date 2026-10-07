# R03 diagnostic projection bridge — bounded implementation review 028

INPUTS_071에 동결된 application/projection/control bridge와 직접 관련 test source를 읽었다. 소스·문서·Git 변경과 시험/build 실행은 하지 않았고 신규 /tmp 보고서만 작성했다. Root의 진행 중 game::behavior 회귀를 helper 실행 결과나 PASS로 인수하지 않는다. 공개 readiness/전체 native bounds/observer 완료 감사도 아니다.

**제한 범위에서 구체적인 production 결함을 발견하지 못했다.** 제어 실패의 빈 성공 전환과 ordinary post-commit Err 증가 경로는 확인되지 않았다. internal panic은 아래의 닫힌 diagnostic serialization invariant와 구분해야 한다.

초기 capture 2026-09-10T12:11:32.927481+00:00. INPUTS_071 at_utc=2026-09-10T12:10:06.708994+00:00, file count=250. manifest 전체를 검토한 것이 아니라 관련 source 4개 entry와 실제 읽은 SHA를 대조해 일치를 확인했다. 027 제안은 보존된 원문 SHA `721a2e9c498de32b8b9010a87a23c96f0e368ece7cf3d54ef9a0ad214e80a670`와 연결한다.

## 1. Result와 일반 동작

application.rs 355–388의 from_behavior는 먼저 check_projection_control을 검사하고, native payload 성공만 Ok로 감싼다. payload projection Err는 Result인 recovery mapper로 들어간다. 이미 latched/cancelled 상태라면 recovery 첫 검사(402)에서 Err로 끝나므로 AcceptedRecoveryPending/빈 accepted payload를 새로 만들지 않는다.

실제 ports는 execute_internal(948), active_battle_reconnect(993), elapsed(1005), next event(1028), checkpoint continuation(1077)에서 해당 Result를 직접 반환한다. HostTick(1013–1018)은 domain Result를 먼저 처리한 후 Option::map(...).transpose()로 None/Some/Err를 보존한다. public signatures는 이미 있던 Result 그대로다. from_read_only_behavior(390–395)는 recovery disposition을 덮어쓰지 않고 그 외에만 ReadOnly를 적용한다.

control::interruption(361–363)은 inspect(...).err()이며, inspect의 no-scope branch(317–319)는 Ok다. 따라서 scope가 없는 ordinary 경로에서 이 bridge가 ProjectionFailure Err를 새로 반환할 수 없다. genuine native payload serialization error는 398–413의 실제 message/JSON을 가진 AcceptedRecoveryPending으로 유지된다. 남아 있는 native serialization/formatting invariant panic을 Result Err로 바꾼 구현도 아니다.

제어 scope 안의 Err는 기존 core 내부 commit 이후 발생할 수 있다. 이 bridge가 그 내부 commit을 자체 rollback하는 것은 아니다. opaque trial candidate/outer final acceptance가 소유하는 오류 신호이며, ordinary API에 새로운 전체 rollback 계약을 부여하지 않는다.

## 2. 미게시 오류와 panic invariant

check_projection_control(591–600)의 envelope는 ProjectionFailure, String::new(), Value::Null뿐이다. 이 생성 자체는 heap allocation이 필요 없다. production caller는 from_behavior/recovery의 Err 또는 기존 map_err/Err error builder 경로다. 성공 ApplicationResult에 넣는 분기는 없다. typed interruption은 기존 phase/code/site를 TLS에 보존한다. checkpoint는 Display/to_string이나 두 번째 JSON을 실행하지 않는다.

diagnostic_interruption(603–609)은 checkpoint가 실패를 확인한 경우에만 envelope를 반환한다. checkpoint가 Ok인데 private formatter/diagnostic serializer가 실패하면 panic한다. **현재는 닫힌 invariant 실패 처리이며, budget/cancellation 실패를 panic하는 경로가 아니다.** from_domain details는 literal string keys와 기존 String/static str/u64/UUID authority/unit enums/typed compatibility vector이고, recovery/invalid_input details는 만들어진 String이다. 기존 json!도 그 literal payload의 native serialization 성공을 전제로 했다. 현재 caller에서 새로 도달 가능한 순수 serializer 실패는 확인하지 못했다.

projection::display에 임의의 Display가 스스로 fmt::Error를 반환하거나 panic하는 구현을 넣으면 이 invariant가 성립하지 않을 수 있다. 현재 production Display 입력은 다섯 serde_json::Error borrow와 고정 문자열 및 genuine recovery serde error다. 일반 to_string도 잘못된 Display의 fmt::Error에 대해 infallible 계약을 전제로 했으므로 이를 새 사용자 입력 오류로 추정하지 않는다. 앞으로 arbitrary fallible serializer/formatter를 이 helper에 추가할 경우 별도 설계가 필요하다.

literal json macro(638–646)는 object! 실패에서 즉시 Self::diagnostic_interruption으로 반환한다. Err를 무조건 Null 성공으로 바꾸지 않는다. 원 error.code()의 static identity와 exhaustive kind/details match(648–727)도 유지된다. failure를 확인한 뒤 진단 builder를 다시 호출하는 재귀 경로는 없다. TLS RefCell borrow는 inspect 반환 때 해제되고 이후 Display/Serialize가 실행된다.

## 3. 첫 문자열·Value admission

projection::display(53–80)는 no-scope에서 원 to_string을 쓰며, active에서는 entry poll 뒤 빈 String writer를 만든다. 매 write_str(60–71)은 ResultBytes checked admission과 text.len work charge **뒤** push_str한다. 실패한 formatting은 고정 serde error로 빠져나오며 diagnostic_interruption이 typed sticky 여부를 확인한다. 정상 종료도 complete poll 뒤 String을 반환한다.

그 raw String은 recovery(405)/invalid_input(624)의 literal object projection을 다시 통과한다. projection::to_value(36–50)는 escaped JSON Counter scan, 전체 native materialization work 선계수, native to_value, post poll 순이다. 따라서 raw UTF-8 길이만으로 JSON key/escape overhead를 통과시키지 않는다. Counter의 self.0 checked_add는 control::admit의 checked 합 검사 이후이며, byte cap과 work 실패 전에 Value native pass를 실행하지 않는다. native Number/map semantics를 바꾸는 JSON 재parse도 없다.

invalid_input의 borrowed Display 교체는 ApplicationCommand::from_document/from_json_slice(154–160, 197–203), admin decoder(818–824), 두 profile merge(1084–1089, 1101–1106)에 실제 반영되었다. constructor 호출 전에 serde error.to_string()을 만드는 다섯 경로가 남아 있지 않다. from_domain은 이미 생성된 native GameError String/Vec를 빌려 첫 diagnostic Value를 검사한다. 이 bridge가 원 domain error producer의 선행 allocation까지 소급 제한한다고 주장하지 않는다.

## 4. 실제 test oracle

다음은 source를 읽은 결과이며 helper 실행 결과가 아니다.

- application.rs 1188–1262는 모든 snapshot kind의 no-scope/충분한 active 값 equality, tree를 만드는 kind의 max_result_bytes=1 실패 code/site/phase, 이후 native snapshot 불변을 검사한다. from_behavior limit 실패는 이제 **내부에서도 Err(ProjectionFailure)**였음을 관찰한다. 이전 AcceptedRecoveryPending 관찰 oracle를 실제로 교정했다.
- 1265–1379는 genuine recovery message/disposition과 raw message 한 바이트 부족, escaped payload 한 바이트 부족, exact payload cap을 구분해 error site까지 검사한다. newline/Unicode가 들어가 raw와 JSON 길이가 다르다. 충분한 active InvalidStaticData details는 native error와 같다.
- 같은 test의 cancellation은 **scope entry 이후, Display 이전**에 token.cancel()을 호출한다. 또는 먼저 작은 work allowance를 넘긴 뒤 recovery/invalid_input을 호출한다. 내부 empty envelope의 kind/code/details, outer failure code/phase 및 Display 호출 0회를 확인한다. 이를 Display 도중/완료 후 cancellation 시험으로 확대하면 안 된다.
- 모든 domain error mapping test의 추가 구간(1514–1531)은 각 native projection과 충분한 active projection 전체 equality를 비교한다. kind/code가 아니라 details까지 함께 비교하는 oracle다.
- behavior.rs 6314–6350은 characterized result별 native/active ApplicationResult equality와 기존 expected transport payload 의미를 비교한다. from_behavior의 새 Result caller도 처리한다. 아직 진행 중이라는 root 회귀 실행의 성공 여부는 읽은 test source만으로 추정하지 않는다.
- projection.rs 142–204의 기존 Counter test는 byte/work exact/one-under에서 native Value serialization 호출 횟수가 1회/2회임을 구분하고 float/Option/native Number 결과를 비교한다.

## 5. 한정된 oracle gap

production defect로 판정하지 않은 추가 경계다.

1. Display가 여러 chunk를 쓴 뒤 중간 취소되거나 마지막 write 후 token을 취소해 diagnostic_message_complete에서 실패하는 fixture는 없다. 현재 Message는 한 번 write_str하며 취소 test는 formatting 시작 전이다.
2. display 자체의 work exact/one-under 및 실패 지점은 직접 측정하지 않는다. prior work failure와 to_value의 기존 exact work oracle와 구분한다.
3. native serde error가 실제 from_behavior 실패를 통해 recovery에 도달하는 fixture, recovery disposition의 from_read_only_behavior 통과, HostTick transpose의 Some projection 실패는 새 diagnostic test에서 직접 재현하지 않는다. source의 caller wiring과 직접 recovery mapper test를 전체 port 실행 증명으로 확대하지 않는다.
4. from_domain equality의 양쪽은 현재 mapper다. 충분한 active/native parity를 강하게 확인하지만 이전 revision과의 외부 raw diagnostic golden 전체 비교는 아니다. 기존 exhaustive mapping literal source를 대조한 확인과 별개다.
5. panic invariant와 late error의 actual trial A/ledger/receipt discard는 이번 좁은 tests의 직접 oracle가 아니다. 이 보고서가 해당 전체 연결을 완료했다고 승인하지 않는다.

가장 작은 보강 후보는 multi-write Display의 중간/완료 후 cancellation과 formatter work exact/one-under, 기존 mapper caller의 실제 Err/None/read-only 경계 assertion이다. 새 public surface나 모든 heap/observer 재조사를 요구하는 결론은 아니다.

## 6. 정확한 hash·읽은 범위

- core_enhanced/src/application.rs: `86fd13745bb96c423747a7c85f22573ef8c16ac29a547859fca67066186899ad`; /files/3; 136–206, 350–432, 579–735, 808–828, 938–1113, 1185–1383, 1494–1565.
- core_enhanced/src/authoring/projection.rs: `45c500a3f414a5fd141d5d22937e911478b7b84c9bc50650025f5c33af32f19b`; /files/10; 1–204.
- core_enhanced/src/authoring/control.rs: `2937619104c4091cbc580e36fb09c40d14f8cca474fa7a885be328a3e87d6478`; /files/6; 309–364, 510–576.
- core_enhanced/src/game/behavior.rs: `652582899aa37856d7d093c71b7ac9f57248319527065de001a983bd7006e408`; /files/79; 3835–3882, 4760–4801, 6295–6350.
- core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/evidence/INPUTS_071.json: `fcee66598c014ec98b2b32e409393981232acb9cf12470f14d3b8255a37a3443`; manifest/prior design; manifest relevant entries / prior design retained in conversation.
- /tmp/r03_diagnostic_projection_bridge_design_027.md: `721a2e9c498de32b8b9010a87a23c96f0e368ece7cf3d54ef9a0ad214e80a670`; manifest/prior design; manifest relevant entries / prior design retained in conversation.

이 제한 검토의 무결함 관찰을 공개 readiness나 전체 Goal 완료 승인으로 해석하지 않는다.

종료 rehash: 2026-09-10T12:16:46.397810+00:00; drift 0개. 이후 변경은 이 검토에 포함되지 않는다.
