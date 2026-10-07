# R03 diagnostic projection bridge — bounded design 027

INPUTS_069의 성공 JSON preadmission 다음에 첫 diagnostic JSON을 연결하는 제한 설계다. 소스·문서·Git 변경과 시험/build 실행은 하지 않았다. 이 보고서는 제안이며 완료 감사나 successor 구현 검증이 아니다. Root는 중간 제안 이후 구현 시작을 알렸으므로 읽은 원본과 종료 drift를 구분한다.

**가장 작은 경계는 성공 mapper만 Result로 바꾸고, 오류 builder는 Self를 유지하는 것이다.** 전자는 제어 실패를 가짜 accepted result로 만들면 안 되고, 후자는 이미 실패한 outer scope가 폐기할 고정 오류 envelope로 끝낼 수 있다. 정상 native 오류의 kind/code/details와 recovery disposition은 그대로 보존한다.

## 1. 실제 문제와 interruption bridge

현재 projection::to_value는 active scope에서 Counter로 bytes/work를 먼저 검사한 뒤 native Value를 만들고 post poll한다. 하지만 application.rs 355–405는 projection 실패를 infallible AcceptedRecoveryPending으로 바꾸며, 그 안에서 error.to_string()과 json!을 호출한다. 현재 test 1192–1202도 내부 disposition이 AcceptedRecoveryPending이고 outer run이 이를 최종 거절하는 상황을 실제로 기대한다. 새 연결에서는 그 내부 기대값도 Err로 바꿔야 한다.

control.rs 310–348의 inspect는 이미 typed first failure, cancellation, phase를 소유한다. 여기에 **문자열을 만들지 않는** 작은 crate-private checkpoint(site) -> Result<(), ExecutionFailure>를 붙이면 된다. inspect(site, |_| Ok(()))를 직접 반환한다. 기존 poll의 domain 변환(control.rs 352–355)은 GameError::InvalidStaticData(failure.to_string())를 생성하므로 이 진단 bridge의 최초 확인에는 쓰지 않는다. 새 checkpoint는 scope 밖에서 Ok, 실패한 scope에서 기존 실패를 보존하고 새로운 cancellation도 latch한다. 단순 is_active 또는 failed flag만으로 cancellation을 놓치지 않는다.

ApplicationError 안의 private constructor 후보:

`interrupted_unpublished() -> Self`: kind=ProjectionFailure, code=String::new(), details=ApplicationDocument(Value::Null).

이것은 **Err 안에서만 사용하는 zero-allocation envelope**다. code/details의 빈 값은 공개 진단의 대체물이 아니다. checkpoint가 typed failure를 확인한 분기에서만 만들고 ApplicationResult/Ok 안에는 넣지 않는다. run_with_provenance의 final inspect가 실제 ExecutionFailure를 반환하므로 임시 ApplicationError와 candidate는 폐기된다. trial.rs 94–123, 310–326의 outer map_err(execution_error)와 inner candidate? 순서가 그 소유 경계다.

bridge는 core가 이미 만든 local domain commit을 스스로 되돌리지 않는다. 현재 trial의 opaque candidate와 외부 final acceptance가 rollback을 소유한다. 일반 public application 호출 전체가 이 변경만으로 새로운 transaction을 얻는다고 주장하면 안 된다.

## 2. 필요한 fallibility와 실제 caller 수정

| 현재 private 함수/호출 | 최소 변경 |
| --- | --- |
| ApplicationResult::from_behavior, 355 | Result<Self, ApplicationError>. entry/checkpoint 실패는 즉시 Err envelope. 정상 payload는 기존 값으로 Ok. |
| from_read_only_behavior, 389 | 같은 Result. from_behavior? 후 AcceptedRecoveryPending은 그대로 두고 기존 readonly 변환만 수행. |
| projection_recovery_pending, 397 | 같은 Result. 제어 실패를 먼저 확인하고 genuine serde projection error만 bounded message와 기존 recovery payload/disposition으로 Ok. |
| execute_internal, 895–896 | 마지막 Ok(from_behavior(...))를 해당 Result 직접 반환. |
| active_battle_reconnect, 940–941 | from_read_only_behavior Result 직접 반환. |
| elapsed/next-event clock, 952–953 / 973–974 | from_behavior Result 직접 반환. |
| HostTick, 961–964 | domain .map_err(from_domain)? 뒤 `result.as_ref().map(ApplicationResult::from_behavior).transpose()`. None은 그대로 None. |
| checkpoint continuation, 1022–1023 | from_behavior Result 직접 반환. |

공개 CoreApplication method signatures는 이미 Result이므로 변경할 필요가 없다. execute/execute_with_generation은 execute_internal로 연결된다. admin의 기존 이미 소유된 payload factory는 이 좁은 from_behavior refactor 때문에 바꿀 필요가 없다.

테스트 caller는 application.rs 1173–1204와 behavior.rs 6320의 정상 결과를 unwrap/Err 검사에 맞춘다. 범위 검색에서 이 mapper의 다른 production 직접 caller는 발견되지 않았다. from_domain은 여러 map_err callback(805, 841, 868, 895, 926, 940, 952, 964, 973, 980, 990, 1006, 1022, 1039, 1056)에 쓰이므로 Self 유지가 작다. public error type이나 모든 port 반환형을 중첩 Result로 만들 필요가 없다.

## 3. Display 문자열을 먼저 무제한 만들지 않기

Root가 채택한 방향은 projection.rs에 private bounded fmt::Write String writer를 두는 것이다.

1. fmt 호출 **전** typed checkpoint. no-scope는 기존 to_string/into String 의미를 유지한다.
2. write_str마다 checked(current UTF-8 len + chunk len), ResultBytes admission, work charge를 모두 통과한 뒤 push_str한다. write_char/write_fmt도 이 write_str를 통해 흐르게 한다.
3. fmt::Error 발생 시 새 오류를 format하거나 JSON으로 만들지 말고 typed checkpoint에서 latched failure를 확인해 반환한다. 이미 실패한 String은 local drop된다. formatting/Serialize 중에 TLS RefCell borrow를 유지하지 않는다.
4. 완료 checkpoint 뒤에만 String을 반환한다. 그 문자열을 기존 projection::object!({"projection_error": message}) 또는 {"message": message}로 다시 검사한 뒤 첫 diagnostic Value를 만든다.

raw UTF-8 message 상한은 escaped JSON 상한과 다르다. 따옴표/역슬래시/control 문자는 JSON에서 커지고 object key/braces도 추가되므로 후속 literal object projection의 Counter를 생략하면 안 된다. formatting work와 JSON scan/native materialization work는 서로 다른 실제 작업이며 같은 attempt fuel에 각각 계수한다. 이 설계는 전체 message String 생성 전에 제한하고 각 push 전에 admission한다. 길이를 알기 위해 Display를 스트리밍하는 것까지 “format 호출 전에 전체 크기를 안다”고 주장하지 않는다.

현재 private invalid_input(582–591)의 message: impl Into<String>을 borrowed Display 입력으로 바꾸고 code는 실제 모두 고정 literal이므로 &'static str로 좁힐 수 있다. caller에서 미리 error.to_string()을 하면 내부 writer가 늦는다. 바꿀 serde 오류 호출은 **5곳**: ApplicationCommand::from_document(154–160), from_json_slice(197–203), ApplicationAdminCommand::from_document(766–772), merge_player_profile(1030–1036), merge_encounter_compendium_profile(1047–1053). error 자체 또는 &error를 넘긴다. try_new의 두 고정 메시지는 같은 Display 경로로 의미를 유지한다.

Cargo.lock 2010–2020은 serde_json **1.0.140**이다. 실제 local source에서 JSON writer Serializer::collect_str(ser.rs 408–454)는 fmt adapter를 통해 escaped chunks를 write하지만 Value Serializer::collect_str(value/ser.rs 294–300)는 value.to_string()을 호출한다. 따라서 단순 Serialize Display wrapper를 추가하는 것만으로 첫 무제한 String을 막았다고 판단하면 안 된다. 반드시 bounded first pass 또는 위 bounded String writer가 필요하다. serde_json::Error의 Display(error.rs 349–356, 399–417)는 기존 message/code와 line/column을 유지하므로 &error를 동일하게 format하면 native 문자열 내용을 바꿀 이유가 없다. 임의 사용자 Display 내부의 자체 allocation까지 통제하는 공개 generic formatter API는 제안하지 않는다.

## 4. infallible error builders가 보존해야 할 것

**from_domain(error: GameError) -> Self.** 가장 먼저 typed checkpoint를 검사한다. error.code()는 &'static str(behavior.rs 5267–5303)이므로 code를 String으로 복사하기 전에 제어 실패에서 빠져나올 수 있다. 현재 exhaustive match의 kind와 literal details 키는 그대로 두되, json!을 borrowed literal-object helper로 바꿔 첫 Value 전에 검사한다. Null details는 원래 Null이며 allocation이 없다.

현재 필드는 static/owned strings, u64, UUID/revision DTO, unit enums, SkillFragmentId(String), Vec<unit compatibility enum>이다. 긴 InvalidStaticData reason 또는 failure_codes를 먼저 clone/format할 필요가 없다. 해당 native error가 이미 소유하는 producer String/Vec가 만들어지는 비용까지 이 JSON bridge가 소급 통제한다고 주장하지 않는다. ActionUnavailable code가 reason.as_str()인 기존 identity 등 각 code를 일반화하지 않는다.

**invalid_input(...) -> Self.** formatter와 literal-object projection이 모두 성공하면 원래 kind/code/message details를 만든다. quota/cancel 실패가 latch됐으면 위 allocation-free envelope로 끝낸다. 이로써 기존 map_err caller를 유지할 수 있다.

**중요한 invariant:** literal-only diagnostic helper의 Err를 무조건 Value::Null로 바꾸지 않는다. serde 오류 뒤 typed checkpoint가 실제 실패를 돌려줄 때만 discard envelope를 사용한다. 체크가 Ok인데 serializer/formatter가 실패했다면 그것은 별개의 순수 serialization invariant 문제다. 현재 닫힌 literal-key/derived primitive 필드는 native json!의 infallible 전제였고, no-scope 기존 동작을 바꾸거나 새로운 허위 성공으로 덮으면 안 된다. 향후 실제 fallible field를 도입하면 별도 명시 fallibility/typed internal failure를 설계해야 한다. 이 보고서는 예상하지 않은 오류를 새 diagnostic으로 재귀적으로 포장하는 fallback을 권하지 않는다.

recovery_pending도 같은 규칙이다. **제어 오류가 없는 genuine payload serialization error**는 기존 AcceptedRecoveryPending + 원문 projection_error를 보존한다. recovery 문자열/JSON에 대한 control 실패가 생긴 경우에만 Err로 전환한다. No control failure인데 코드·메시지를 임의로 잘라내거나 빈 payload를 돌려주지 않는다.

## 5. 범위에 맞는 검증 후보

아직 helper가 실행한 시험은 없다.

- 모든 GameError mapping의 normal/no-scope와 충분한 active limits에서 kind/code/details semantic equality. 긴 InvalidStaticData/String 및 compatibility Vec에 대해 cap exact/one-under.
- 이미 실패했거나 취소된 scope에서 Display가 호출되지 않고 Err envelope만 local로 생성됨을 관찰. outer code/phase/site가 최초 failure 그대로인지 확인.
- recovery genuine serde error는 원문 message와 AcceptedRecoveryPending 유지; message writer/escaped JSON cap 실패는 내부에서도 Err이고 outer typed failure. 입력에 quote/backslash/newline을 포함해 raw와 escaped 크기 차이를 검사.
- chunked Display의 낮은 cap에서 실패 chunk 전에 append가 멈추는지, 충분한 cap에서는 원 to_string과 byte equality인지 검사. fmt::Error without sticky와 control fmt::Error를 혼동하지 않는다.
- from_behavior caller 전체, 특히 HostTick None/Some transpose와 read-only recovery disposition을 확인한다. 기존 test의 “제어 실패 중 내부 AcceptedRecoveryPending” assertion은 제거 대상이다.
- 실제 trial의 late diagnostic-size/work/cancel 실패 뒤 candidate/A 및 ledger/receipt가 보존되는 연결을 확인한다. 새 bridge를 외부 publish하는 테스트나 fake accepted empty 결과를 허용하지 않는다.

원본 읽기 중 root는 위 Result mapper/typed checkpoint/미게시 envelope/bounded fmt writer 방향을 채택했다고 알렸다. 그 메시지는 구현 검증이나 완료 승인이 아니다. 변경 후 구현은 별도 source/실행 근거로 평가해야 한다.

## 6. hash·읽은 범위

주 capture 2026-09-10T12:01:18.935475+00:00; dependency/직접 error field 추가 capture 2026-09-10T12:06:18.118753+00:00. 추가 hash는 최초 관련 읽기 뒤 취득했으며, manifest에 있는 파일은 INPUT069와 일치함을 확인했다. 원자적 repository snapshot은 주장하지 않는다. 없는 serde_json-1.0.145 path는 탐색 오기였고 실제 근거는 lock-selected 1.0.140 local source다.

- core_enhanced/src/authoring/projection.rs: `80981f066e6a31e510398aef7c7911c49b67c2359f44592578d358c3a857f6a4`; /files/10; whole captured file (Counter, to_value, literal Object macro and tests).
- core_enhanced/src/authoring/control.rs: `1a15e8bdd7618572950ddc69ad25b5cda674eebfecd0f544010abb3477b2fed8`; /files/6; 236–252, 304–355.
- core_enhanced/src/application.rs: `f61cb3fa2421e96e737c63bd83791853509841bc2840e4f087dbb36d5ddd87e1`; /files/3; 136–216, 328–432, 489–684, 765–774, 875–1060, 1150–1208, 1340–1346.
- core_enhanced/src/authoring/trial.rs: `0500b82ac8bc8cfc4cbbbf076520834e27de756147fab7d97f74659cb7cdd22a`; /files/15; 94–138, 310–326, 351–378.
- core_enhanced/docs/goals/content_authoring_tool_ports/execution/single_goal/evidence/INPUTS_069.json: `b9d098edd1d6bf9b08228f06f5d2fb00bd7509fbcaa70b931fb876867cd983b8`; local source/manifest; JSON relevant entries.
- Cargo.lock: `80eed115bdcf0aa881da5b6c1525e6f19a0d6d8e1101ee4b285840ab7b3a8f19`; /files/0; 2007–2026.
- core_enhanced/src/game/behavior.rs: `adfc9b54025bb6f0b98cb790e080d76df2c31ddcca61ed8784baa5c7a1a99c7b`; /files/79; 46–60, 475–510, 623–640, 3790–3875, 5180–5320, 6308–6328.
- core_enhanced/src/game/data/skill_fragment_data.rs: `5fd2772af7d0dc8ff5c0abee5cb7b05bdcb0996ce55a67e1a717d34dff5bc9ad`; /files/125; 1–38, 368–401.
- /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.140/src/ser.rs: `5352ffeafb6397b4a837ca4d40b0353510788d445938f1bff96cde66d0467c1d`; local source/manifest; 402–458.
- /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.140/src/value/ser.rs: `b5cd3f4b4a58b2f53c4334c6d20705fc7c7b2a2e8988ef93b265a679c952cdfe`; local source/manifest; 288–305.
- /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.140/src/error.rs: `a9b5de0a82f95608b51b8e8875c7c49f94fb60b9f976fc6277aec0213926dec9`; local source/manifest; 345–359, 396–417.

종료 rehash: 2026-09-10T12:10:01.233703+00:00; drift 4개. Successor 구현 내용은 이 원본 설계 검토에 인수하지 않았다.
- core_enhanced/src/authoring/projection.rs: 80981f066e6a31e510398aef7c7911c49b67c2359f44592578d358c3a857f6a4 → 45c500a3f414a5fd141d5d22937e911478b7b84c9bc50650025f5c33af32f19b
- core_enhanced/src/authoring/control.rs: 1a15e8bdd7618572950ddc69ad25b5cda674eebfecd0f544010abb3477b2fed8 → 2937619104c4091cbc580e36fb09c40d14f8cca474fa7a885be328a3e87d6478
- core_enhanced/src/application.rs: f61cb3fa2421e96e737c63bd83791853509841bc2840e4f087dbb36d5ddd87e1 → c7895d87df864175bf698770c8dcde435626289ac02a1ab6b794a9de8a3e005b
- core_enhanced/src/game/behavior.rs: adfc9b54025bb6f0b98cb790e080d76df2c31ddcca61ed8784baa5c7a1a99c7b → 4119f381b58496e00681b049f34239afe47eb64d7f992c172db7b753f4855d3b
