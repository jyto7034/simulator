# U11 현재 기술 판단

현재 P2G20 actual 조사와 P2G30/40 구조 검토를 인수한 GEX00 준비다. 정책15/수용30은 동결되었고 제품·시험 변경/게임 실행은 아직0이다. 코드 기반 actual/제안/technical choices는 ../../CODE_REVIEW.ko.md와 immutable CRA-CMVA-001이 소유한다.

핵심 미완료는 실제 mod registry, producer coverage와 정상 bulk preparation, 근거 있는 concrete Lab criterion, issuer-local authenticated cache, 정상 local/server approved facade 및 writer-capable 저장 동작 이전의 required-mod 보호다. P2G상 evidence gap이 없다는 것은 이 기능들이 구현되었다는 뜻이 아니다.

실측 수치는 현행 직접 사용자 위임 아래 Root가 실제 normal/expanded/pathological raw와 분포·peak·회수·여유를 검토하여 채택한다. diagnostic-only profile이나 synthetic threshold 테스트만으로 정상 Apply 완료를 선언하지 않는다. plain default-release와 instrumented 결과 parity/성능 범위, native와 process heap/RSS 및 lifetime baseline을 구별한다.

원본 save 보호는 현재 V11 continuation/V12 SavePoint·paired intent·backup recovery의 actual 순서에 연결한다. 캐시 권한은 untrusted mod/client/다른 issuer 경계이며 악의적 arbitrary in-process Rust host를 격리한다고 주장하지 않는다. 기존 low-level Core/static loader는 진단 기반으로 유지하되 정상 mod launcher/server는 approved facade를 의무 소비한다.

현재 정책 질문/필수 source gap0. mutation 전 baseline과 concrete public/auth/save/build 설계 인수는 아직 pending이다. 기준선/구체 구현의 확인 결과는 후속 기록으로 추가한다.

P2G50/60 actual 완료를 인수하여 GEX00을 시작했다. 정상 admission의 다섯 owner를 함께 전환하는 single phase와 PUBLIC/VERIFY의20통합/10차원 매핑을 수용했다. 현재 Sol이 baseline/제품 준비, author가 독립 시험 설계 문서를 담당하며 source writer/runner0이다.

## 독립 시험 설계 인수

Root가 test_parts/TEST_PLAN_001.ko.md의192줄 전체를 실제 읽고 A–H 설계를 수용했다. 필수 목록과 actual collector의 동일 누락을 막는 독립 authored oracle, same-bucket 생성/종료 양성, meaningful cache HIT/MISS와 조합 actual 실패, side-effect 이전 preflight 및 same-original fresh Continue를 포함한다. wall-clock exact 경계는 재현을 가장하지 않고 판정 산술 보조와 실제 raw criterion 소비/정상·초과 종단을 함께 요구한다. 현재 이 문서는 설계뿐이며 시험/fixture/runner 실행0이다. Sol의 baseline/구체 API 인수 뒤 같은 묶음에서 작성한다.

## GEX00 baseline·제품 준비 인수와 GEX10 판단

Root가 BASELINE001(8031B), SOURCE_CURRENTNESS001/EXTRA_RESTORE001/INTAKE001 및 PREPARATION001 전문을 읽었다. U10 source543 exact/current drift0와 기존 archive의 실제 unique membership/격리 bytes 복원, 추가16 current preimage의 exact 격리 복원, 전체559 preparation 전후 불변을 수용한다. 가시 runner0는 해당 namespace snapshot이며 외부 실행 부재 주장은 아니다. proposed20새파일 부재를 결속했고 workspace 복원·제품/test/build는0이다. baseline의 E4문서 snapshot은 당시 원문 참조이며 후속 Root 문서 변경을 runtime drift로 보지 않는다.

PREPARATION T01–07의 concrete 구조를 CODE_REVIEW와 대조했다. Root는 actual active_run_store의 V5 reader/strict Core11/backup 순서와 player_profile_store/application_intent.rs의 journalV1·rename 이후 accepted/pending 및 복구 순서를 다시 읽었다. **host envelope V6/journal V2에 RequiredConfiguration을 checksum 결속하고 Core Continue11/SavePoint12는 유지하는 전환을 수용한다.** 이전 지원 V5/V1은 read-only full CoreIdentity가 현재 승인 구성과 exact인 경우에만 읽는다. 원wire bytes를 유지한 복구 및 실제 새 durable acceptance에서만 V6/V2 발행을 허용한다. unsupported Core10/host4/기타 version과 legacy marker의 원본 보호를 완화하지 않는다. 이는 T06의 허용된 상세 metadata 전환이며 기존 승인/저장 의미를 변경하는 migration이 아니다.

구현에서 아래를 명확히 해야 한다. 첫째, unsupported/malformed authoritative intent 또는 valid-but-incompatible current/backup은 보호하며 다른 복사본으로 숨기지 않는다. 일반 손상 current+유효 호환 backup은 기존 복구 양성을 유지한다. read-only preflight가 actual recoverable copy를 먼저 판단하고 해당 approved configuration과 원본을 검사한 뒤만 수선한다. 양쪽 손상·불명확한 필수 identity는 쓰기 없이 거절한다. 둘째, ActiveRunStore::open의 directory/lock 생성과 PlayerProfilePersistence 초기 load를 포함해 사전 side effect를 열거하고, 기존 저장 디렉터리에서는 read-only 검사를 lock 생성보다 먼저 수행한다. 이미 존재하는 lock 취득과 original save/profile mutation을 구별하며 lock 아래 재검사를 유지한다.

셋째, authenticated cache의 temp/rename/directory-sync 실패를 모두 '아무 변화 없는 실패'라고 뭉뚱그리지 않는다. 기존 유효 record를 불완전 새 record로 덮어쓰지 않고, durable publication의 실제 accept 지점·불확실/재시작 처리·cancel/stale 폐기와 approval 발행 순서를 구체화한다. incomplete/cancelled job이 restart에서 임의 승인으로 부활하면 안 된다. 기존 U04 journal의 accepted 의미를 새 cache에 무조건 복사하지 않으며 중요한 의미 충돌은 Root에 반환한다. Hmac<Sha256>/Mac::verify_slice 선택은 공식 hmac0.12.1 문서의 사용 예와 대조해 수용했다(https://docs.rs/hmac/0.12.1/hmac/). 실제 dependency/lock와 구현은 추후 검토한다.

넷째, trusted base는 mod_id 문자열만으로 권위를 얻지 않고 host suite의 exact base selection이 지정한다. acquisition/parse 실패 mod가 독립 정상 mod의 유효 기록이나 별도 선택 검증을 봉쇄하지 않도록 candidate별 frozen inputs와 오류 귀속을 유지한다. union snapshot이라는 구현 선택은 mod isolation과 TOCTOU 계약을 만족해야 한다. 다섯째, build identity는 생성 산출물/자기 참조/전체 거대 history 없이 실제 영향 source/config/Cargo/lock/toolchain/feature의 결정적 범위를 결속한다. plain/instrumented pair와 실제 host binary 대응, 지원 environment를 명시하며 불지원 조건을 조용한 승인 fallback으로 바꾸지 않는다.

이 조건은 기존 frozen 의미를 구현에 적용한 구체 준비 판단이다. GEX00을 완료하고 GEX10 전체 atomic admission/PUBLIC/VERIFY 구현·시험 작성을 연결한다. 일반 내부 선택/교정은 자율 수행하고 중요한 cache accept/public/save 의미만 근거와 함께 반환한다. 수치 criterion은 여전히 실제 calibration 뒤 채택하며 지금 미정 값을 채우지 않는다.

## GEX10 cache publication 경계와 실행 수집 도구 검토

Sol의 rename 이후 directory-sync 실패 제안을 frozen Goal의 승인 금지·rollback·retry 및 CODE_REVIEW T04와 대조했다. 모든 실제 validation/coverage/criteria 완료와 single-writer lock 아래 최종 revision/cancel 검사를 거친 immutable per-key+nonce result rename을 cache publication acceptance로 두는 상세화를 수용한다. acceptance 전 IO/cancel/stale는 기존 유효 record를 보존하며 Incomplete/Cancelled/Stale로 종료한다. acceptance 뒤 sync 불확실은 별도 RecoveryPending으로 표시하고 approval 발급은0, 기존 선택/live 상태를 유지한다. 이는 검증·계측 incomplete나 cancelled 결과를 다시 승인하는 경로가 아니다.

복구는 인증된 완전 accepted-format record에 한정한다. actual current candidate/issuer/build/profile/suite와 exact identity 및 revision relevance를 재검사하고 file·directory durability를 재확인한 후만 capability를 발행한다. temp/partial/preaccept 실패는 재시작 승인 후보가 아니다. acceptance 뒤 cancel은 TooLate를 명시하되 사용자 선택 변경은 future Apply를 계속 막으며 과거 record가 새 revision을 자동 활성화하지 않는다. revision/cancel과 rename의 직렬화, crash 전후, restart/tamper, 기존 valid record 보존 및 유한 회수를 독립 시험에 연결하도록 두 담당자에게 전달했다. U04 player durable accept와 같은 rollback 권한으로 확장하지 않는다. 아직 구현·시험 결과는 아니다.

Root는 runtime001/run.py·INPUTS.json·RUNNER_REVIEW_001과 preserved U09 collector 원문을 읽고 기존 raw/receipt/source archive/before-after/exclusive output 재사용을 수용했다. 새 wrapper scope는 actual core examples·crate별 .cargo 및 build identity의 실제 추가 의존과 실행 전에 대조하도록 author에게 반환했다. 현재 INPUTS가 비어 있으므로 실행 준비 완료가 아니다. 실제 binary identity·environment parity·functional 판정은 별도 probe에서 결속하며 collector가 이 의미를 대신하지 않는다.

scope 보완 원문과 bind_inputs.py 전문도 읽었다. fixture 실제 membership/bytes/hash 검증, 기존 exact ref 보존, 준비 receipt와 publication 완료 구분을 수용했다. descriptor 검사는 writer0 전제의 사전/직전 검사이며 동시 writer에 대한 atomic CAS 보장으로 표현하지 않는다. author가 이를 수용했고 Sol 준비 단계에서 이전 INPUTS 원문·actual after/stdout/hash를 보존한다. 이후 실제 실행은 Sol만 담당한다.

첫 Core compile-only 입력 고정 중 schema/error/identity/candidate/build.rs 및 content_pack의 frozen publication 경로를 실제 읽었다. 기존 load_with_runtime_safety가 적용하는 expression_parse 4한도와 caller InputLimits의 min-clamp가 새 FrozenContent::publish 경로에서 빠진 것을 확인하고 Sol에 일반 제품 교정으로 반환했다. author에는 같은 실제 profile의 normal loader/candidate/individual 경계 회귀를 연결하도록 전달했다. build.rs의 leaf rerun tracking도 source/data directory membership 추가·삭제와 실제 build config 입력을 반영하도록 반환했다. 실행 중 편집은 하지 않으며 compile 종료 뒤 동일 묶음에서 교정한다. 아직 compile/functional PASS를 확인한 기록이 아니다.

## Accepted A→B intent의 재시작 복구 범위

Sol이 V2 journal의 old/new required configuration 경계를 반환하여 Root가 application_intent의 stage rename acceptance·finish/profile-first 순서와 ActiveRunStore의 revision 검사·current/backup 설치 코드를 실제 읽었다. 이미 authoritative V2에 accepted된 A→B 전환은 B만 approved인 재시작에서 제한적으로 replay할 수 있다. 이는 A를 다시 실행하거나 일반 A 저장을 B startup로 교체하는 권한이 아니다. 미수락 temp·intent 없음·legacy V1에 이 경계를 추정하지 않는다.

V2 checksum 안에 expected old revision·old RequiredConfiguration뿐 아니라 old record의 exact logical payload identity 또는 원 envelope digest와 new B record/required configuration을 결속한다. read-only preflight는 모든 유효 current/backup이 그 accepted old record 또는 exact already-installed new record임을 확인한다. 같은 revision/config라도 payload가 다른 원본, 제3의 valid incompatible 원본, unsupported·불명확 원본은 쓰기 없이 보호한다. 승인된 B의 actual Core/required identity 확인과 lock 아래 재검사 이후에만 그 transaction의 profile/active replay를 허용한다. corrupt current와 확인된 호환 old/new backup의 복구 양성은 유지한다. 기존 legacy V1은 exact compatibility와 원wire replay만 유지한다.

이것은 U04 accepted intent를 V2 metadata로 더 엄격히 특정하는 구현 판단이며 일반 incompatible fallback 허용이 아니다. Sol과 author에게 crash-before/after acceptance, mixed old/new copies, wrong revision/payload, missing B 보호, B-only restart 양성과 same-original 보존 검사를 같은 묶음으로 연결했다. 해당 경로의 구현·실행 결과는 아직 미확인이다.

## 2026-10-11 재개 코드 검토

Root는 현재 issuer/cache/host 원문과 criterion private binding 및 실제 metrics completeness 경계를 읽었다. 이 검토는 부분 구현에 대한 것이며 기능 완료 판정이 아니다. `ApprovedHost::new`가 승인 검사의 issuer lock을 해제한 뒤 initial future를 설치하는 check/install 간격을 확인했다. 이미 `apply`가 사용하는 원자적인 현재 승인 검사·설치와 같은 경계를 생성자에도 유지하도록 Sol에 반환하고 author에 foreign/stale initial admission 검사를 연결했다. 이미 적용된 B는 미승인 C 검증 중에도 유지되어야 하며, 아직 설치되지 않은 stale capability가 새 호스트 초기화로 우회 적용되면 안 된다. 실제 정상 서버 adapter와 영속 수락 연결은 완성 후 다시 검토한다.

Root는 새 mod_requirements/ApprovedPersistence와 V2 journal encode/decode·read-only preflight, AdmissionSession 및 실제 actor 연결 지점을 읽었다. 세 가지 남은 문제를 동일 묶음으로 반환했다. 첫째, gate가 최초 A와 현재 future만 허용하면 A→B 영속 수락 뒤 future C가 설치될 때 아직 활성인 B를 거부한다. bounded durable generation pin을 accepted 전환에서 갱신하고 actor/저장 수명을 보존해야 한다. 둘째, profile 두 복사본 손상을 authoritative journal보다 먼저 거부하면 기존 U04의 검증된 accepted profile 복구를 봉쇄한다. unsupported profile은 계속 보호하며, 유효 accepted intent와 exact active/config 검사가 모두 성립한 경우에만 해당 profile replay를 허용한다. Sol은 두 문제를 수용했고 compile 입력 고정 종료 뒤 교정한다.

셋째, 현재 AdmissionSession은 최종 candidate 한 개만 검증한다. frozen T01/T04의 trusted-base+dependency 개별 actual 검증과 최종 조합 actual 검증은 정상 consumer에서도 수행·정확 재사용되어야 한다. 독립 candidate API나 시험의 수동 호출만으로 정상 경로 완료를 선언하지 않는다. mod 실패/dependency-held 귀속과 조합 실패 뒤 독립 PASS 보존을 author의 정상 launcher/server 검사에 연결했다.

독립 author의 normal_preparation125줄·fixture_coverage105줄·prepare_lab_registry54줄·bulk_coverage165줄을 실제 읽고 첫 실행에 사용할 의미를 수용했다. literal required inventory, 같은 bucket의 A/B 각각 실제 HP13/17, full-health no-op와 양성 효과 구분, object 종료 및 실패·취소 준비의 다른 trial 불변을 검증한다. 별도 비치명 target base와 employee1의 진단 결과를 representative calibration이나500/1000 workload로 표시하지 않는다. 실제 실행은 아직 없다.

정상 actor fault 회귀의 준비 근거로 `game_server/examples/fault_runtime.rs` 전체와 `handlers/tests/fault.rs`의 actual tick 경로, 선행 U04 `evidence/public_fault_final_004/SUMMARY.json` 및 tiny profile을 읽었다. seed23의 정상 StartNewGame/VN/starter/map→combat 명령 경로와 native max_work1000의 실제 host_tick fault를 재현 후보로 수용한다. 현재 embedded 설정에서 의도한 native work만 바꾸는 별도 fault-regression 구성으로 식별하며 예전8MiB 등 나머지 설정을 무심코 복사하지 않는다. 현재 issuer에서 충분한 실제 required suite로 승인한 뒤 후속 정상 gameplay가 fault를 일으켜야 한다. 승인용 coverage를 축소하거나 PASS를 조립하지 않으며 조기 fault로 Lab 승인 자체가 불가능하면 그 실제 근거로 준비 경로를 다시 판단한다. 선행 실패 기록은 현재 승인 경로의 실행 증거를 대신하지 않는다.

실제 외부 probe lock 생성에서 root와 다른 dependency 버전이 선택된 사실을 Sol이 발견했다. root Cargo.lock을 항상 해시하던 build identity는 이 외부 소비자의 실제 graph를 표현하지 못하므로 승인 correspondence가 완성됐다고 할 수 없다. Root는 실제 standalone probe Cargo를 읽고 독립 crate를 root workspace member로 통합하여 기존 root resolution과 명시적 package/config 실행을 사용하는 방안을 수용했다. 기존 standalone manifest/lock·실패 원본은 보존하며 root Cargo/lock은 Sol, probe Cargo는 author가 고정 실행 종료 뒤 순차 수정한다. 관련 verification integration이며 저자 귀속은 유지한다.

workspace 위치만으로 모든 feature/build의 동일성을 주장하지 않는다. 지원된 실제 lock/graph·관련 feature·compiler/build config를 결속하고, 외부 승인 consumer의 불명확한 provenance는 fail-closed 또는 명시적 trusted actual binding을 요구한다. normal AdmissionConfig뿐 아니라 issuer에서 사용하는 Core engine과 worker pair의 실제 대응도 확인해야 한다. 임의 버전/feature 차이를 정규화해 equality를 만들지 않으며 diagnostic-only low-level Core는 별도로 유지한다. 실제 normal-host 바이너리마다 고유 executable identity와 calibration context를 유지한다.

Root는 수정된 session.rs 전체를 읽었다. 초기 admission의 dependency 순서 개별 검증·보류 전파·남은 authored 선택 재resolve·최종 actual 조합 검증과 개별 취득 receipt의 최종 포함 대조는 T01/T04에 맞는다. 반면 공개 validate_and_apply는 첫 개별 실패에서 나머지를 중단하고 개별 suite 경로도 소비하지 않아 초기 경로와 차이가 남았다. 기존 설치 세대를 보존하면서 같은 공통 orchestration을 소비하도록 제품·독립 시험에 반환했다. 이는 실행 PASS 판단이 아니다.

CoveragePlan과 실제 skill validator도 읽었다. nonempty step의 effects가 모두 비어 있는 유효 Skill은 정의 의무가 제거된 뒤 coverage row가 0개가 될 수 있었다. 효과·rule이 없는 다른 유효 정의도 같은 경계를 확인해야 한다. 모든 변경 정의는 actual normal consumer 의무를 유지하거나 구체적인 미지원으로 Incomplete가 되어야 하며, 빈 관측 목록의 전칭 성공을 승인 근거로 삼으면 안 된다. 유효 authored content를 금지하는 방식으로 회피하지 않고 정상 legal-noop와 미실행을 구별하도록 Sol과 author에게 전달했다.

supervised/execute_worker와 실제 server worker entry도 읽었다. 현재 worker 내부의 typed Trial execution 오류가 프로세스 nonzero로 전달되면서 상위 spawn_bounded의 일반 worker.exit Incomplete로 바뀐다. bounded nonce/build/mode/candidate 결속 오류 결과를 보존하여 실제 content/runtime 실패와 환경·측정·IO 불완전을 구별하도록 반환했다. crash/timeout/읽을 수 없는 결과는 계속 Incomplete이며 부분 계측을 성공 sample로 채우지 않는다. 오류 parity는 실제 typed 진단을 소비해야 한다.

독립 calibration.rs와 prepare_workloads.rs 전체를 읽었다. 관측 전에 고정한60초 window,16/500/1000 준비 roster와1/8/16 deployment를 별도로 보고하는 입력을 첫 realization 제안으로 수용한다. 이것은 대표성·scale 성공·수치 기준 채택이 아니다.16MiB 제품 output ceiling과32MiB 제안의 단순 불일치는 author가16MiB로 교정했고, 실패한 placement/scale은 원시 기록과 UNREALIZED_CASES만 보존하며 usable suite를 발행하지 않도록 보강했다고 보고했다. 실제 준비·실행 결과를 먼저 확인한 뒤 workload 적합성과 수치를 판단한다.

Author의 실제 calibration manifest·전용 evidence directory가 필요한 inline cache/save/actor integration 시험은 이유를 명시한 ignored-by-default 등록을 수용한다. 실행 입력 고정이 해제된 뒤 author가 해당 시험만 수정한다. 이는 U11 검증 생략 권한이 아니며 독립 literal 필수 selector 목록의 전부를 Sol이 `--ignored`와 필수 환경으로 명시 실행하고 실제 nonzero 시험 수·결과를 남겨야 한다. 명시 실행에서 입력 부재/불일치는 실패다. 사전 artifact에 의존하지 않는 시험은 기본 활성 상태를 유지하고 ordinary suite의 ignored 수를 이 Goal의 검증 완료로 계산하지 않는다.

Root가 external normal_save55줄·shared normal_save_tests310줄 및 normal_actor_tests 전체를 읽었다. same-original fresh Continue·legacy V5·durable B/future C·실제 exit86 crash stage·accepted V2 exact old/new/손상 profile 복구와 wrong-payload/third-C/V1 음성 설계는 검토된 저장 계약에 맞는다. 아직 실행 근거는 없다. actor의 ReadOnly 시험 helper가 fault 이후에도 export_active_run_checkpoint 성공을 unwrap하는 문제를 actual application.rs의 ensure_battle_not_faulted와 대조하여 author에게 반환했다. 제품의 기존 export 거부를 typed 결과로 관측·검증해야 한다. Quit 후 단일 다른 actor 질의가 deregistration 완료를 보장한다는 가정도 bounded actual FindPlayer 확인으로 교정하도록 했다.

제품·시험 담당자가 supported coverage 미도달의 ContentFailed/Incomplete 차이를 반환하여 Root가 frozen PE03/PE05 수용 원문을 다시 읽었다. 정상적으로 취득됐지만 실제 consumer가 사용하지 않은 Equipment, trigger/target 미도달, 필수 scenario 부족 등 양성 witness 부재는 구체적인 stage/reason의 **Incomplete**다. 관측 부재만으로 콘텐츠 오동작을 입증하지 않는다. 실제 Application/gameplay/runtime 오류는 ContentFailed, authoring 준비/계량 한도·관측 불완전은 Incomplete, 명시 취소는 Cancelled로 구별한다. 완전한 유효 측정의 adopted criterion 초과와 complete-coverage 조합 초과는 ContentFailed를 유지한다. coverage 부족과 독립적인 측정 초과가 함께 있으면 두 진단을 모두 보존하고 승인하지 않는다. 이는 frozen PE03/05의 구현 분류 판단이며 정책 변경이 아니다.

Root는 새 WorkerResponse 실패 envelope와 supervisor의 nonce/build/mode/candidate 대조·실제 양쪽 오류 parity를 읽었다. partial sample을 성공으로 채우지 않는 방향을 수용하며 실행 판정은 남아 있다. tools/cmva_build.py와 build.rs의 실제 Cargo tree proof도 전체 읽었다. lib-tests의 dev subtree와 normal bin의 Core graph 대응은 아직 확인되지 않았다. root/nested Core subtree가 다를 수 있으므로 실제 effective runtime dependency/features와 consumer-only dev 의존의 구별을 준비 근거로 확인하도록 Sol에 반환했다. 차이를 지워 equality를 만들지 않으며 지원 matrix의 실제 대응을 기준으로 한다.

Sol은 현재 coverage 범위를45 kind 중 Add/Replace9(Employee, Equipment, SkillFragment, Relic, PveEncounter, RunPolicy, Skill, Buff, IndependentObject), Remove5(Employee, Equipment, SkillFragment, Relic, PveEncounter)로 반환했다. 나머지는 구체적인 unsupported Incomplete이며 reference/winner-invalid 입력은 기존 정적 거절이다. Root가 PE03/13과 T02를 대조하여 이 버전의 유한 지원 표면으로 구현을 계속하도록 했다. 모든45 catalog를 기계적으로 확장하지 않되, kind×operation 한계를 실제 조회·문서에 명시하고 대표 미지원 변경의 보류·원본/독립 유효 기록 보존을 확인해야 한다.9/45 자체를 전체 완료 근거로 삼지 않는다.

이 한계는 현재 U07–10 기능 profile을 생략할 권한이 아니다. 예를 들어 Resource catalog 변경의 미지원과 지원 Skill/Relic에서 실행되는 typed-resource·구조·causal 기능의 실제 required coverage는 서로 다르다. 후자는 이 Goal의 필수 검증에 남는다. 이미 지원하는 비전투/제거 consumer는 실제 양성·정확한 음성·surviving control로 확인하며 정적 publication을 runtime witness로 대체하지 않는다. 최종15요소/30수용 항목·전체 감사의 완료 조건은 유지한다.

새 update API를 읽고 final combination이 없는 경우 compatibility wrapper가 마지막 individual Passed job을 반환할 수 있음을 확인했다. 실제 final job만 반환하고 조합 취득/resolve 실패는 오류로 유지하도록 반환했다. suite 파일 재취득 대신 입력에서 한 번 포착한 suite를 세션에 보유하고, frozen-candidate API의 초기 승인 context와 fresh configuration API의 명시 새 context를 구별하도록 했다.

Author가 개별 candidate에 최종 winner 전체가 남는 문제를 반환했다. Root는 schema::content_selection·Candidate::individual_from_capture/selected_from_capture와 resolver의 absent/non-maximal writer 거절을 직접 읽었다. 개별 trusted-base+dependency closure는 그 실제 포착된 writer에 해당하는 mod winner만 투영하며 trusted-base의 명시 선택을 유지한다. 개별 검증을 위해 관계없는 손상 pack을 취득하지 않고, 포함된 writer의 부적격/모호함은 기존 resolver대로 거절한다. 최종 reduced selection은 원래 최종 winner 지시를 유지하므로 winning mod가 보류되면 typed 조합 오류를 반환한다. 임의 winner 추정/자동 fallback은 없다. 이는 T01의 서로 다른 개별/최종 구성에 기존 writer 의미를 적용한 구체 판단이다.

Root는 후속 실제 소스에서 final-only wrapper, Inputs의 suite 단회 취득/세션 보유, captured manifest pack ID 기반 개별 winner 투영과 coverage_support의45-kind 조회·한계 표시를 확인했다. 이 검토는 아직 기능 실행을 대신하지 않는다. 독립 semantic_coverage156줄 전체도 읽어5종 제거의 실제 대조군·정확한 거절·surviving 준비와 음성 대조군, Resource3operation 보류,4종 비전투 실제 외부 값·다른 trial/원본 불변 oracle를 인수했다.

측정 driver의4 phase가 suite 전체 합산인 점을 확인하여 정상/확장/병적 workload 각각의 별도 suite·paired process collection을 combined approval-suite 측정과 함께 준비하도록 했다. Author는 isolated3 suite와 명시 CLI scope를 추가했다고 보고했다. 서로 다른 suite의 criterion을 복사하지 않으며 각 실제 context identity를 유지한다. 수치 채택은 여전히 실제 raw review 이후다.

별도 native1000 fault 회귀의 A=official base/B=RunPolicy777 방향을 수용했다. 이는 대표/default 승인 기준의 대체물이 아니다. 실제 changed RunPolicy와 해당 recipe의 trusted Employee/PveEncounter 준비 witness를 요구하여 빈 base suite가 되지 않도록 한다. 해당 정확 context도 실제 raw 측정·검토 뒤 교정하며, 승인된 유한 준비 이후 normal actor 경로에서 실제 fault를 관찰한다. Lab 자체에서 먼저 fault가 나면 실패 원본과 준비 근거를 반환하고 필수 coverage나 capability를 위조하지 않는다.

현재 CMVA instrumented matrix가 기존 server runtime-measurement→runtime-observation의 exporter 의존까지 활성화하면서 실제 Core transitive feature graph가 달라졌다. Root는 Cargo manifest·worker Collector·runtime_measurement module·server entry와 실제 tree를 읽고 **cmva-lab-measurement=[game_core/runtime-observation]** 전용 feature를 수용했다. allocator/RSS/native 측정 모듈과 실제 worker executable의 allocator는 이 feature에서도 활성화하고, 기존 U03 server runtime-measurement 의미와 plain default graph는 유지한다. exporter를 일반 빌드에 무조건 추가하거나 graph 차이를 정규화해 숨기지 않는다. 독립 probe의 instrumented 연결은 author, 제품 및 build matrix는 Sol 소유다. 네 실제 graph 준비에서 일치 확인 후 release 실행하며 actual executable/context identity와 strict pair 대조는 유지한다. 이는 구현의 측정 경로 분리이며 criterion 채택·측정 PASS·선행 완료 재개가 아니다.

Root가 build005 실제 제품 patch 전체를 읽어 Lab 전용 feature·main/launcher allocator·worker Collector·모듈 연결 및 기존 feature 보존을 확인했다. 새 독립 prepare_contexts.py 전체, Core calibration 관리 entry, normal_save 관리 entry와10개 필수 selector 목록도 읽었다. 실행파일별 context·raw·adoption·실제 A/B 수락을 구분하고 Root 수치 placeholder를 실행하지 않는 준비 흐름을 인수한다. 관리 entry/실제 crash child는 독립 기능 PASS 수에 포함하지 않는다. 현재 source writer0에서 실제 graph/release로 진행한다.

prepare_workloads는 성공한 개별 scale의 isolated 진단 suite를 남기고 한 scale이라도 실패하면 combined suite를 발행하지 않는다. 실패 메타데이터는 combined_suite_exported와 realized_isolated_suites를 구분한다. 부분 isolated 성공은 combined 승인·확장 scale 성공을 대신하지 않는다. 추가 coverage 대조에서 supported Skill/Relic의 typed-resource **값 변화**와 Grant(ActionRecipe) 실제 효과 양성 oracle가 아직 필요함을 확인했다. owner shape/drop 및 InlineEffects/causal 양성만으로 이 항목을 완료하지 않으며 다음 bounded authoring batch에 보강한다. 현재 build/preparation은 먼저 진행하고 추가 source 영향에 맞춰 후속 검증을 연결한다.

Root는 첫 release의 실제 toolchain 환경 결속 실패와 교정 소스를 읽었다. 지원된 rustup 기반 빌드 도구가 actual active-toolchain을 먼저 확정하여 compiler/tree/build 모두에 같은 명시 환경을 전달한다. supplied proof가 실제 build 입력과 다르면 Core 최적화 전에 컴파일을 실패시키고, proof 자체가 없는 low-level diagnostic build는 기존대로 유지한다. RUSTUP_TOOLCHAIN 비교를 제거하거나 빈 graph를 승인하지 않는다. 이번 중단의 기록과 자식 종료를 확인한 뒤 source 쓰기를 재개했으며, 독립 author의 준비된 capability fixture를 다음 고정 입력에 통합한다. 후속 실제 nonempty build-script graph와 실행파일 worker identity를 확인해야 한다.

Root가 새 독립 capability fixture4개 RON·prepare_capability_registry·capability_oracle 및 bulk_coverage 접속 전체를 읽었다. Resource 정의가 없는 official의 별도 복제 Lab base에 초기9/cap20/deployment 자원을 명시하고 provenance를 남긴 뒤, supported Skill/Equipment/Relic candidate만 취득한다. 실제 sparse 초기값·9→4→7·live read 피해14·causal cost→5와 정확한1회 producer를 요구한다. 별도 recipe는 original41+7=48/generated53+11=64 실제 personal shield, 원본/생성 effect identity,250ms 및10+350ms 유한 만료와 실제 부재를 확인한다. 두 경로는 실제 retreat·resource owner 해제 및 독립 positive trial 보존도 소비한다. 이 설계를 다음 실제 실행에 인수하며, Resource 카탈로그 변경 승인이나 미실행 coverage PASS로 표시하지 않는다. author는 고정 입력 밖 임시 공간에서 준비하고 이전 파일 hash 대조 후 Sol이 해제한 쓰기 경계에서9개 소유 파일을 통합했다. 현재 다시 writer0다.

실행 준비에서 Sol이 발견한 /mnt/f 권한 제한을 Root가 실제 stat으로 확인했다(evidence와 release directory mode777). private issuer/worker의0700 검사를 완화하지 않고, 실제 Unix 권한이 가능한 명시 /tmp private runtime root를 사용한다. frozen config/suite/fixture는 repository-relative INPUTS 결속을 유지하도록 prepare_contexts에 별도 runtime root를 author 소유로 추가한다. 실제 path/mode·비밀정보 없는 raw 복사와 hash 대응을 보존하며 issuer key/전체 private store를 world-readable F evidence에 복사하지 않는다. 이는 환경 준비 교정이며 미실행 정상 Apply를 통과로 바꾸지 않는다. 현재 release를 중복 실행하거나 이 helper 변경 때문에 재개를 지연하지 않고, 다음 안전한 쓰기 경계에서 통합한다.

Root가 lab.rs::supervised/create_private_dir와 issuer/cache의 실제 권한 검사를 대조했다. 직접 공개 calibration의 caller work_root는 materialization 전에 실제 owned/private/nosymlink를 검증하고, 새 directory도 요청한 mode만 믿지 않고 실제 mode/uid를 확인해야 한다. Sol의 좁은 fail-closed 교정과 author의 양성·잘못된 권한/symlink 거절/쓰기 전 불변 oracle를 다음 경계에 배정했다. 현재 release를 중단하지 않으며 먼저 일반 coverage 진단을 진행하고 최종 paired/승인 증거는 교정 source에 결속한다.

첫 실제 candidate001의 raw는 candidate/GeneratedSetup/object_preview_domain_sort에서 authoring_work_limit(limit10000000, requested10000054)을 기록했다. Root가 raw와 acquire_registry·selected_from_capture_inner·FrozenContent::publish를 직접 읽었다. 최초 candidate는 이미 검증된 base를 원본 대응 확인 목적으로 다시 전부 publish하고 실제 selection을 publish하여 같은 aggregate 한도 안에서 생성 검증을 중복한다. captured subset 경로에는 base publish가 control::run 밖에 있는 문제도 있다. author는 선택된 A가 공식26개 catalog와 작은 Skill/Equipment 추가뿐이며 unselected bulk pack은 읽지 않는다고 확인했다.

다음 안전한 경계에서 Sol은 기존 immutable base publication을 Arc로 보유·재사용하고, 최초 취득에서 captured base의 정확한 전체 SourceReceipt(선택된 pack/path/bytes/hash/member 대응)를 기존 base selection/settings와 대조하도록 교정한다. 대조 비용을 계량하고 final selected publication과 단일 aggregate 작업 한도는 유지한다. bounded removal counterfactual 경로는 그대로 둔다. 한도 재설정·분할·상향은 선택하지 않았다. 독립 author의 candidate_usage 진단은 unchanged embedded limits에서 단독 official/A/B/AB publication과 실제 Candidate A/B/AB 결과를 각각 보존하고, 실패 시 사용량 부재를 명시한다. Root가 staged 진단 전체를 읽었으며 다음 영향 빌드와 함께 통합한다. 원래 실패와 교정 후 증거는 구분한다.

실제 semantic Relic 거절에 따른 author의 fixture 교정은 제거/생존 base relic에 유효한 nonzero flat1 효과를 부여한다. Root는 연관된 prepare_registry·zero_site_coverage·update_missing_combination도 읽었다. 빈 Relic는 기존 validator상 적법하지 않으므로 명시적인 ContentFailed 음성 사례로 유지하고, 적법한 empty Skill은 non-vacuous coverage 사례로 유지한다. update right-member에는 별도 유효 nonzero numeric Relic를 넣어 publication 성공 후 실제 소비되지 않은 coverage로 Incomplete가 되는 것을 요구한다. 그 뒤 stale right winner의 최종 combination 거절을 검사한다. 입력 parse/validation 실패를 runtime hold로 바꾸어 부르거나 validator를 완화하지 않는다. 기존 generated roots는 그대로 두고 새 출력에 교정 source를 결속한다.

candidate002는 정상 A/B/AB와 captured individual을 지난 뒤 첫 structural candidate의 FinalValidation/relic_structural_owned_copy에서 work10000000 한도를 넘었다(requested10001733). Root가 raw·compiler::admit_serialized_copy·Work::reserve·runtime_control의 retained growth 계량을 읽었다. 직렬화 scan과 AST의 보수적 저장 한도, growth/8 작업 debit을 실제로 수행한다. Sol/author가 확인한 fixture는 original damage1개+finite generated damage1개+숫자 edit3개의 작은 정의이며, structural rule이 처음 생기면 기존 early-return 대신 공식 provider/Relic 조합의 전체 적법성 검증이 활성화된다. 현재 중복 계산 제거가 필요한 증거는 없으며 정당한 전체 검증과 보수적 비용 산정이 남은 원인 후보이다.

사용자의 자원 위임에 따라 baseline 실패를 보존하고 별도 관찰용 설정을 허용한다. 독립 author는 명시 ROOT/registry/selection/base selection/profile 입력마다 단독 publication과 실제 Candidate의 work·elapsed·Linux RSS/VmHWM를 기록한다. VmHWM은 프로세스 누적 peak로 표시하며 phase allocation으로 부르지 않는다. baseline embedded를 먼저 실행하고, 완료 비용을 얻지 못하면 별도 profile에서 authoring_execution.max_work만100000000으로 바꾸어 유한 외부 timeout 안에서 관찰한다. native/expression/기타 한도는 그대로이며 기존 authoring_dev의 다른 차이를 끌어오지 않는다. 이 값은 기본 한도·Lab criterion 채택이 아니며 개발 구성 elapsed를 release 성능으로 쓰지 않는다. 실제 release·대표 확장 비용과 거부/취소/원자성 검증을 확보한 뒤 설정을 판단한다.

Root가 source010의 admin.rs 실제 patch 전체를 읽었다. WorldLabPreparation은 요청 encounter 존재를 먼저 확인하고 현재 정상 진행에서 선택 가능한 recipe.category의 manifested node를 사용한다. 원래 difficulty/kind/slot은 보존하고 명시 trusted Lab encounter만 draft payload에 결속한 뒤 production handle_select_map_node/handle_confirm_enter_node로 progression·session·entry snapshot·preview·plan·commit을 거친다. 적합한 노드가 없으면 실패하며 임시 노드/난이도 fallback을 만들지 않는다. 배치 grant나 deployment 실패 시 바깥 준비 draft가 전부 폐기된다. 실제 semantic/zero-site 후속 실행으로 확인한다.

독립 prepare_lab_registry의 수정도 Root가 전체 읽었다. preset이 있는 corroded profile의 직렬화에서 파생 defense_tile_range만 비우고, 재취득한 전체 resolved database가 의도한 비살상 데이터와 정확히 같은지 비교한다. 원래 preset/범위는 LAB_BASE_ORACLE에 보존하며 validator를 바꾸지 않는다. family_usage 전체와 명시 selection7개·baseline/관찰 profile의 단일필드 diff·300초 TERM/10초 KILL 직렬 실행 지침도 인수했다. 관찰 profile 실행은 앞선 baseline 작업 한도 실패에 대한 기존 조건부 권한으로 진행하며 매 호출마다 새 승인을 요구하지 않는다.

## 2026-10-11 실측에 따른 명시적 개발 profile 채택

Root는 사용자 자원 설정 위임(기존 한도는 기준선으로 보존하되 정상·예상 확장의 실제 시간/메모리/작업량 근거로 기본 또는 개발 설정 허용)을 재확인하고, `family_observation_{structural,causal,resources,recipe}001/{publication,candidate}_result.json`의8개 실제 완료 결과를 직접 읽었다. publication work=16,951,492/32,672,692/21,129,557/20,014,677; Candidate=18,198,226/33,922,641/22,372,059/21,260,053. dev Candidate elapsed=3.192021225/3.321851795/3.289649852/3.283478691초, 작업 후 RSS 및 process lifetime HWM=43,020,288/43,106,304/42,868,736/42,852,352bytes. HWM은 앞선 publication/base 및 진단 기록을 포함하는 누적값이며 phase별 할당량이 아니다. 모든 phase는 usage_available=true/completed/error=null이다.

source009 중복 base 검증은 이미 제거했고 정상 structural publication의 전체 provider/슬롯 증명 비용을 보존했다. Root는 명시적 DEVELOPMENT profile `CMVA-authoring-dev-v1`을 채택한다. authoring.max_work만100,000,000, 다른 embedded 필드·native16MiB·trial 설정은 동일하다. 실측 최대33,922,641의 약2.95배인 유한한 개발 여유이며, 확장 준비에서 초과하면 새 근거로 재판단한다. 관찰된 payload SHA256491197cc5d6e98f26a6b037fda5ac6d176ed732f7e7a3898b8061eececbf43d0를 명시적인 제품 config 이름으로 보존하고 모든 probe/context/cache에 실제 path/hash를 결속한다. 일반 authoring_dev.ron의 다른 상향 필드는 혼합하지 않는다.

현재 family/public-Lab 기능 확인 및 이후 paired 측정 개발 context를 진행할 설정이며, embedded/출하 기본값 변경·성능 합격 기준·500/1000규모 지원 완료가 아니다. 기존1천만 실패, 자원 거부·원자성·취소 기대값과 회귀 검증을 보존한다. 최종 성능 판단에는 실제 release/대표 확장/반복·여유 근거가 여전히 필요하다. Sol은 제품 config, 독립 author는 명시적 설정 전달을 포함한 시험 수정, Root는 채택 판단을 소유하며 진행 중 관찰 collector가 종료된 쓰기 경계에서 통합한다.

## 2026-10-11 geometry 교정과 clone 비용 관찰 범위

Root가 실제 공개 상태 기반 facing diff를 읽고 수용했다. 시험은 published weapon TileRangePattern::affected_tiles로 고정 Up/Right/Down/Left 순서를 투영해 실제 defense_main 경로와의 첫 교차를 선택하고 실행 전에 전체 기하를 저장한다. 회전 구현을 oracle로 복제하지 않는다. SourceTile(+1,0)는 실제 제품 코드상 절대 월드 offset이라 방향과 독립이며 기존 free/blocked/occupied 검사도 유지한다. requested8/16 위치가 전부 교차 조건을 충족하지 못하면 실제 입력/기하를 남기고 준비 불가로 기록하며 축소하지 않는다. semantic fragment의 잘못된 장착+2 가정도 앞서 읽은 실제 baseline[]/10 및 addedfragment/51 근거로 교정했다.

Root는 CloneMeter scan_storage/admit_storage/finish, controlled_clone, bulk initialize_employees_with_loadouts, BattleEntrySnapshot::capture, Arc CloneShape를 읽었다. cloneSlots는 하나의 실제 소유 복사의 상태 크기를 계량하고 Arc는1handle만 계량한다. 500/1000 roster와 각 장비·fragment를 정상 부여한 뒤 완전한 진입/트랜잭션 상태를 복사하는 비용이 증가하며, 현재65537/65541은 early crossing일 뿐 전체 필요량이 아니다. 불필요한 catalog deep copy나 누적peak reset 오류를 확인한 것은 없다.

개발profile 자체는 바꾸지 않고, OBSERVATION ONLY parent CMVA-authoring-dev-v1에서 max_clone_slots65536→1000000만 바꾼 유한 관찰을 허용한다. 원래16/500/1000 roster를 유지하며 discovery/no-deployment 비용과 가능한 정상 deployed 경로를 분리한다. 외부300초+10초 종료 한도, 실제 work/오류/시간/RSS/HWM/입력 불변을 수집하고 공개API가 노출하지 않는 peak는 unavailable로 표시한다. 배치 부족을 비용 관찰의 성공으로 숨기거나 이 관찰을 gameplay/성능/전체규모 PASS로 쓰지 않는다. author는 진행 빌드의 입력 밖에서 probe를 준비하고 Sol의 다음 안전 경계에서 통합·실행한다.

## 2026-10-11 actual terminal coverage 및 selected recipe 계약 교정

Root는 source012 PRODUCT_DIFF/PREPARATION과 실제 sim/damage_runtime producer를 읽었다. resources002 DamageExpr count2는 실제 피해 fact와 일반 applied_effect 기록의 중복이다. HealExpr도 같은 소유 관계다. 일반 기록에서 Damage/Heal의 literal·expression4종을 제외하고, 실제 targets가 있으나 applied/scheduled command가0인 damage만 producer LegalNoop로 기록한다. 실제 흡수 피해는 fact가 소유하므로 이 경로에서 두 번째 terminal을 기록하지 않으며 empty targets Ineligible는 유지한다. 독립 author는 기존 DamageExpr HP14/count1 외에 건강비용→실제회복→full-health noop→shield→흡수피해→zeroExpr의 좁은 실제 효과/terminal 회귀를 준비한다.

Root는 recipe fixture, structural/execution681의 effective PlayerUnit/AllySingle 변환, effective_skill335의 requires_explicit_structural_selection, sim4883의 automatic hold, TriggerAbility와 invoke 명시적 hint 경로, sustained/structural의 release recipient 검사를 직접 읽었다. PersonalShieldSingle은 플레이어가 수락한 명시적 선택을 요구한다. OnAttack enemy hint는 그 권한이 아니므로 이를 넘겨 runtime invariant가 된 경로는 기존 계약 위반이다. Sol의 좁은 자동 producer gate를 수용하며 effective equipment/relic provider를 검사하고 기존 legacy selection 의미는 넓히지 않는다. caster를 임의 선택하거나 recipientguard를 약화하지 않는다. 현재 proc recipe는 no-invariant/positivecoverage hold 음성 사례로 보존하고, 원본·생성 shield48/64·timing/expiry/retreat의 양성 사례는 실제 정상 accepted selected ActiveSkill 경로로 author가 준비한다.

기타 시험 교정은 Root가 실제 소스로 수용했다. zeroDamage EnemySingle step은 usefulness 필터로 제외되므로 explicit cast EnemySingle은 유지하고 step을 CastTarget으로 변경한다. zero LegalNoop는 같은 liveOpponent·직전 같은bucket BasicAttack HP·checkpoint불변·후속HP이벤트부재로 추가 증명한다. ObjectSelf/pattern에는 실제 인접(+1,0) 생성 reach가 필요해3x3cross+anchor를 명시하고 실제guard는 유지한다. causal의 empty resourcecatalog에서는 active_typed_resources=None, 별도 seeded capability의 resources/recipe는Some가 올바르며 retreat후 모두None를 확인한다. 실패 원본·창·개체수·자원 설정은 바꾸지 않는다.


## 2026-10-11 준비 비용 관찰 결과와 대규모 시나리오 후속

Root는 diagnostic004/RESULT_001의 실행 결속, 실제 outer receipt의 exit0/source불변, six-child COLLECTION 및 각 discovery/deployed/geometry 실제 결과를 읽었다. 기본 clone65536에서500/1000은 준비가 거부됐다. clone1000000 관찰에서는500/1000 zero-deployment 준비가 완료되어 최대 owned-copy shape141627/280271, accepted work2322055/3727229, dev 준비시간0.3140/1.0506초, 작업후RSS75911168/83021824bytes를 기록했다. 16명은 두 설정에서 discovery7407/deployed13545로 같았다. 이 값은 단일 dev 관찰이며 process HWM은 누적값이다. 전체 전투 비용·반복 분포·최종 한도 근거로 확대하지 않는다.

공개 patrol geometry는14개 빈 Ground cell만 제공하며 첫8개 중 실제 무기 범위와 경로가 교차하는 곳은2개다. 따라서 이 준비 입력은 요청한8/16명 전투 참여를 아직 충족하지 못한다. 독립 author는 기존 공식 encounter/template를 먼저 검토하고, 부족하면 명시적인 trusted test content의 전장/encounter를 정상 public loader로 구성한다. roster16/500/1000, deployment1/8/16,60초 창은 유지하고 실행 전 기하와 실행 후 각 배치 actor의 실제 공격/효과를 확인한다. 정상 무기 출력·판정·제품 전장 상태를 시험 편의로 변경하지 않는다. clone1m는 이 후속 deployed/full-window 비용을 알아내는 유한 관찰 설정으로만 계속 허용한다. 개발profile 추가 채택과 embedded 기본값·성능criterion 결정은 아직 하지 않았다.

수동 recipe oracle의 실제 Run DTO 연결 오류를 수정한 stage도 Root가 읽고 수용했다. equipped에는id만 있으므로 active_copy_uuid를 compatibility의 정확한copy_uuid/definition_id/base/is_compatible에 연결한다. 기존 terminal/manual batch의 기대값은 유지하고 다음 실행 종료 경계에서 통합한다.


대규모 fixture 제안 `u11_workload_context`의 pack/template/encounter/selectors를 Root가 모두 읽고 수용했다. 공식 일반 전장은16명 배치가 불가능해 새 OpenHall/Large 전장과8개의 서로 다른 north_entry spawn, 인접한16개 Ground 배치, defense_main 경로를 사용한다. normal16은 기존 patrol을 유지하고 expanded/pathological만 새 encounter를 사용한다. trusted base/후속 cache/native context에는 같은 pack 선택을 전달한다. 기존 prepare_lab_registry는 이미 LAB_BASE_ORACLE의 표적 HP10000/attack0/interval60000을 명시하며, 이번에는 그 기존 bytes를 보존한다. 초기 제안의 “공식guard HP90 유지” 설명은 현재 Lab 입력과 달라 교정하도록 반환했다. Root는 실제 workloads002 normal lifecycle의running/last_battle_time_ms60000/total_elapsed_ms60000도 확인했다. 후속에서도 요청60초와 실제 active duration/terminal 여부를 구분한다. 완전히 처리된 finite encounter가 조기 종료할 수 있다는 사실 자체를 실패로 새로 규정하지 않으며, 실제 활동시간을60초로 부풀리지 않는다.


## 2026-10-11 Lab 진입의 정상 Continue 저장 누락 교정

Retreat실패는 시험 기대값 문제가 아니다. Root는 admin.rs의select→confirm, application.rs436/468의outer prepare/refresh, continuation.rs622 이후normal helper를 읽었다. Lab outer route는AdminCommand이므로 정상ConfirmEnterNode의pre-entry Continue capture가 호출되지 않았고unfinished_node가없어Retreat refresh가InvalidAction이었다. source013의8줄diff를읽고수용했다. 실제node선택뒤/confirm전 기존prepare_continue_before_operation(Behavior ConfirmEnterNode)을호출하며helper와unfinishedguard는그대로다. 준비된명부/loadout이정상preentrypayload에포함되고outerdraft의원자성·실패rollback·활성전투refresh를유지한다. 추가statecapture의정당한자원비용은후속준비측정에포함한다. 독립author는Lab준비→Continue내보내기/정확한명부복원→재진입/퇴각ledger의집중회귀를기존externalcleanup재실행과함께준비한다.


## 2026-10-11 추가 Continue 회귀의 전제 교정

Root가 앞서 추가로 요구한 “중복 직원 bulk Lab checkpoint의 일반 Continue 복원 성공”은 기존 수용 범위를 넘는 잘못된 시험 전제였다. 실제 집중시험은1실행/0통과/1실패로보존됐고restore에서`recruitment queue/session/roster ownership is not exact`로거부됐다. Root는recruitment.rs132–162의catalog정의별queue/session/roster정확1회소유,정상starter/recruit흐름,combat.rs2949의in-memory entry snapshot restore를직접읽었다. 같은records_clerk를여러명생성하는Labbulk는이일반저장계약에맞지않으며queue에서제거하는교정으로해결할수없다. 또한실제Labworker는lab_checkpoint를결과해시로사용하고일반Continue로복원하지않는다(lab.rs860). Goal의required-mod저장보호/정상Continue는실제정상host저장을대상으로하며Labbulk의일반저장호환성을추가하지않는다.

따라서제품recruitment/restore검증을변경하지않고source013의정상preentrycapture는유지한다. author는추가회귀를preparedbulk명부/정확한장비copy를유지한실제in-memory퇴각·재진입·ledger와bulkcheckpoint일반import의명시적거부/대상불변으로교정한다. 최초실패원본은보존한다. 이교정은기존Goal판정완화나새Lab저장형식추가가아니며,Root가추가한잘못된positiveexpectation의정정이다. 기존causal/resource공개퇴각검증과이후정상host같은원본Continue양성은그대로남는다.

## 2026-10-11 workload 전술 자금과 양성 입력 교정 판단

실제 workloads003의 500/8·1000/16은 새 공개 전장 기하 확인 후 정상 DeployUnit의 자원 부족으로 실패했다. official RunPolicy의 initial20/maximum99, 회복1/s와 배치 비용10을 실제 코드·데이터로 확인했다. Lab 준비 recipe의 동시 배치를 위해 trusted u11_workload_context가 공개 singleton RunPolicy Replace(after official)로 initial/maximum만160/160으로 설정하도록 수용한다. producer는 원본의 유일한 정확 블록 치환·역치환과 전체 RunPolicy 값 비교로 나머지 정책 불변을 확인한다. 실제 공개 checkpoint의 준비 전160과 비용10, 준비 후1/8/16명에 대한150/80/0 및 time0을 독립 oracle로 검사한다. 사설 상태 주입·비용 면제·제품 자원 guard 변경은 없으며 roster16/500/1000과 배치1/8/16, 요청60초와 개별 참여 조건을 유지한다.

같은 suite의 세 클래스 모두 이 자금 context를 사용한다. 따라서 normal 클래스도160으로 시작하는 새 조건이며 기존 official normal baseline을 대체하지 않는다. trusted selection은 candidate/base/worker/calibration/cache/host 문맥으로 전달해야 한다. Root는 기존 선택값을 생략하면 official을 유지하고 명시 입력은 유한 길이·원시 bytes/hash/파싱값을 보존하는 공통 helper와5개 standalone probe 전달부를 실제 diff로 검토했다. 최종 release pair·기준·승인은 동일한 실제 문맥에 새로 결속한다.

terminal 양성은 ordered.shield를 ActionAndEffects의 명시적 유한 effect 참조로 연결한다. 수동 recipe 양성은 금지된 admission 이후 retarget을 제거하고 SelfUnit 선택을 유지하는 Caster 기준 순간 TileArea로17 피해를 준다. 원래48/생성64 shield 및 만료 조건은 유지하며 실제 HP17 감소→SkillAreaDeclared→anchor AbilityStepTriggered→동일 수동 AbilityCast의 인과와 Player/Opponent 역할·준비 직원 소유를 추가 검사한다. Root가 전체 diff 및 sim.rs/skill_runtime/flow.rs의 실제 기록 경로를 읽고 수용했다. 기존 정적 거부 원본은 보존한다. 이 교정은 콘텐츠 유효성 및 시험 근거 보강이며 자동 선택권한 guard 완화가 아니다.

## 2026-10-11 native 작업량 실패와 불변 진입 스냅샷 복사 교정

funded workloads004의 expanded는 Step6 단일 elapsed60000ms clock 작업 도중 native work10,000,000/request10,000,019(tile_pattern_scan)로, pathological은 Lab 준비 중 request10,221,327(owned_clone_copy)로 거절됐다. 초기에는 이를 첫 event-bucket 실패로 해석했으나 실제 trial.rs900–915, combat.rs1466–1484와 sim.rs2824/2850–2915를 확인해 정정했다. 단일 clock 작업 안에서 여러 event-bucket을 처리하며, 거부 후 accepted time0은 롤백 결과다. 실패 직전 내부 bucket/시간은 현재 원시 자료로 알 수 없다. Root가 실제 REALIZATION과 runtime_control의 deferred operation scope, control::charge 전달, World application staging, tactical staging 및 BattleEntrySnapshot 소유 필드를 읽었다. 실패 site는 마지막 debit이며 비용 주원인이라고 단정하지 않는다.

확인된 불필요한 비용은 immutable BattleEntrySnapshot의 전체 roster/inventory가 매 ActiveBattleSession/World staging 때 깊게 복사된다는 점이다. Root는 ActiveBattleSession의 entry_snapshot만 Arc로 공유하는 좁은 교정을 수용했다. owned capture와 event-reentry 편집은 그대로 수행한 뒤 한 번 동결한다. 공유 handle 복사에는 payload 복사를 계상하지 않고, 실제 공유 snapshot 복원은 captured run/checkpoint/event resource와 relic/UUID 소유를 사전 검사하고 controlled deep clone이 성공한 후 대상에 쓴다. 유일 소유자는 try_unwrap으로 기존 owned restore를 사용한다. 새 Arc allocation/header의 실제 비용은 별도로 계상한다. 불필요해진 entry-copy preflight는 공유 복사 지점에서 제거하고 실제 복원 복사 지점으로 옮기며, borrowed retained extent와 현재 run/resource 검사 의미는 유지한다.

world::state는 private module이고 GameCore는 pub(crate)다. 이 내부 표현 변경은 Core façade·전송·저장 schema를 바꾸지 않는다. mutable World/roster 전체 COW나 자원 재설정은 범위 밖이며 ENGINE_MAX/기본 profile/관찰 profile은 변경하지 않는다. 초기 capture 비용, 공유 복사 실패의 원자성, 원본과 mutable 상태 분리, 마지막 owner 복원, 실제 normal/event Retreat·RNG·bulk Continue 및 영향을 받는 nonempty resource/relic 경계를 독립 시험으로 확인한다. product와 test 파일 소유를 분리하고 현재 실행이 닫힌 뒤 구현한다.

같은 묶음의 manual002 실패는 original_shield가 아니라 anchor consumed2였다. 실제 하나의 시전·step·area가 같은 타일의 서로 다른 적2명에게17씩 피해를 주었고 original/generated shield는 각각1이었다. 따라서 제품 중복 계측 수정은 하지 않는다. Root는 시전 전8000ms의 독립 대상2명을 저장하고 실제 area8001ms의 같은 대상 집합·각각 HP17 감소·동일 인과를 검사하는 oracle 교정을 검토했다. expression fixture의 잘못된 unary Add는 neutral Const0를 더한 binary Add로 교정하여 depth4/5와 값1을 유지한다. 관련 실패 원본은 보존한다.


## 2026-10-12 source014 결과와 유효한60초 cadence 입력

Root가 build017/RESULT_001과 해당 원시 결과를 읽었다. 선택 Core 회귀7개는 각각1시험 통과했고 manual003/expression002/causal005/resources005도 통과했다. zero-deployment 준비의 새 Arc 비용은 roster별 고정 work582이며 clone peak는 불변이다. funded expanded8 준비 work는18,328,863에서14,763,021로 감소했다. workloads005는 같은 fixture003/profile로 normal 양성, expanded의 단일60000ms native10m 거부, pathological의9번째 배치 DeploymentLimit을 남겼다. Root가 실제 rulebook과 combat 배치 검사를 읽어 기존 최대8명을 확인했다. 이는 기존16배치 양성 입력 설계의 오류이며 제품 상한을 늘리지 않는다. 단일 elapsed 작업의 실패 직전 내부 시각은 여전히 미확인이다.

후속은 명시적 별도 suite `u11-fixed-cadence-60x1000-v1`다. roster16/500/1000, funded fixture003/profile/context/콘텐츠와60초 창은 유지하며 실제 배치는1/8/8, 준비 후 자금150/80/80이다. 같은 Trial에서60개의 elapsed1000ms를 전달하고 prefix6을 포함한66step은 기존256 이하다. 모든 checkpoint가 running이며 정확한1000…60000ms이고 조기 종료가 없어야 한다. 각 actor의 기존13/17 인과·기하 검사를 유지한다. trial 누적 work1,073,741,824와receipt64MiB 한도는 유지하고 after_seq0의 중복 응답 처리도 전체 비용에 포함한다. StepView authoring usage·경과시간·전체 창 합계·작업별 peak를 보존하되 비공개 native fuel의 정확값으로 표현하지 않는다.

이는 사용자 입력 cadence를 명시한 유효 workload를 추가하는 선택이다. 기존 단일60000ms native 실패와1000/16 배치 거부는 legacy 음성 원본으로 보존하며 새 양성 결과로 대체하지 않는다. 현재 authority8 실행의 입력은 고정하고 author는 외부 stage에서만 준비한다. Root 실제 diff 검토 및 현 실행 종료 뒤 Sol이 통합·실행한다. 최종 측정·criterion·cache/host context는 실제 채택된 suite에 일관되게 결속해야 하며, 아직 성능 기준이나 최종 지원 규모를 채택한 것은 아니다.

Root가 CADENCE.diff의2파일 전체와 EXECUTION_GUIDANCE 및 실제 StepView/usage/lifecycle 계약을 읽고 수용했다. Sol authority8 collector 종료 뒤 exact preimage를 확인해 통합했고 author writer0다. build018 실제 명령의 profile은 workloads005와 같은 `diagnostic_inputs/setup_cost001/observation_only_clone_1000000.ron`이다. CMVA-authoring-dev-v1의 work100m을 부모로 clone1m만 달리한 OBSERVATION ONLY이며, 성공하더라도 clone1m 채택이나 기본값 변경을 뜻하지 않는다. authority 원시8컴파일은 이전 source0548201b에 결속하고, 다음 묶음의 비교기는 두 workload probe 파일 변경과 무관함을 밝힌 뒤 기존 원본을 판정한다. 비교기→영향 probe build→기존8/9배치 원자성 selector→새 cadence 실현을 순차 실행하며 기능 결과는 아직 미확정이다.


## 2026-10-12 cadence 첫 결과와 실제 참여 누락

Root는 authority_compare001/RESULT.json 전체와 직접8/9배치 raw log를 읽었다. 정상1개와 권한 우회7개의 compiler diagnostic가 정확히 일치하여8사례가 통과했고, 기존 공개 배치 한계 원자성 시험은1통과/0실패/0ignored다. 이 정적 권한 결과는 runtime 승인 완료가 아니다.

workloads_cadence001은 모든 세 규모가 실제60회의1초 진행과running60000ms에 도달했으나 normal만 realized=true다. expanded/pathological은8명의 서로 다른 배치와8개 guard spawn은 확인했지만4명의 A13/B17 실제 참여가 없어서 실패했다. Root는 expanded 실제 참여 원본에서 x1–4,y4의0회와x5–8,y4의양성, 적5000ms spawn/속도2600과6750ms 양성 cast를 확인했다. 전체 경로와 정적 범위의 교차만으로 실제 공격 시점의 참여를 보장하지 못한다. 독립 author가 실제 이동·공격 시작/빗나감·대상 선택을 진단하며 인원수/시간/13·17피해 인과 조건은 유지한다. combined suite/승인/성능 기준은 아직 생성·채택하지 않는다.

prepare_contexts의 승인된 교정은 build018 종료 후 통합했다. 이제 positive cadence RESULT/FIXED_CONTRACT 및 실제 scenario/명부/배치와profile 원본을 검사하고 같은 profile bytes를 전달한다. 실패한 이번 workload에는 이 경로를 실행하지 않는다. clone1m 관찰 상태와 기존 단일60000ms·16배치 실패도 그대로 보존한다.

Root는 실제 event91/119/123/124/143/333/357을 읽어5050ms AttackStart→5250ms Resolve/Miss와 적의 타일 경계 이동,6550→6750ms의 오른쪽 교전을 확인했다. frozen template의 행4·6에 x1…8 배치 칸이 있음을 직접 확인했다. 별도 v2는 대규모 두 클래스만 `(5…8,4)`와`(5…8,6)`의8칸을 실행 전에 고정하고, 공개 free inventory 및 기존 range/facing에서 위4Down·아래4Up을 검사한다. 각 actor의 자기 A13/B17 피해 인과를 유지하며 서로 같은 적에게 피해를 주는 것은 원래 조건에 허용된다. 적 능력치/속도/공격/trigger/전장/자금은 변경하지 않는다.

Root는 TWO_FLANK.diff의 prepare_workloads/prepare_contexts 두 파일 전체를 검토했다. suite `u11-fixed-cadence-60x1000-v2`, profile `u11-fixed-cadence-60x1000-diagnostic-v2`, placement plan `u11-east-corridor-two-flank-v1`과 정확8칸을 기록·소비하도록 함께 교정했다. legacy 기본 배치와 normal 순서는 보존한다. cadence001 실패는 그대로 남기고 build019에서 영향 producer compile과 cadence002만 다시 실행한다. 제품·authority·8/9배치 계약은 바뀌지 않아 build017/018의 관련 증거를 재사용한다.


## 2026-10-12 유효 규모 실현과 release 관찰 진입

Root가 workloads_cadence002 RESULT 및 세 클래스의 actual_clock_metrics 헤더와 대표 actor A13/B17 원시 인과를 직접 읽었다. 세 규모 모두60회·running60000ms·완전 이벤트를 만족하고 실제1/8/8명 전원이 자기13/17피해 인과를 보였다. 대규모8개 guard spawn도 정확하다. 최종 authoring work는35,613,818/249,198,624/383,276,453, clock만의합계는34,213,686/234,435,618/354,961,709다. 작업별 clone peak는14,923/249,240/486,932, 작업별 work peak는748,030/4,292,971/6,301,739다. dev advance-call 합계는1.499/8.218/11.490초이며 probe 기록 IO 포함 시간2.468/11.933/15.659초와 구분한다. native fuel·release 성능 수치로 표현하지 않는다.

Root는 동일 정확 RON bytes의 clone1m 관찰 상태를 유지한 release paired 측정을 승인했다. 실제 시간·heap/RSS·work 분포 이후 정식 개발 profile과 criterion을 채택한다. 관찰 profile 자체를 먼저 채택해야 측정할 수 있는 것으로 취급하지 않는다. 향후 stable config를 추가하면 측정된 bytes/hash를 보존하고 engine build fingerprint 영향 여부도 확인한다.

후속 normal_update의 unsupported Resource3경로에는 full suite의 전장/자금이 없는 semantic root가 연결돼 있음을 사전 검토에서 확인했다. Root는 prepare_contexts의 전체 diff와44파일/450,095bytes copy 목록을 읽고 수용했다. fixture003의 official/workload context와 semantic002의 semantic_base/unsupported3팩을 원본 bytes 그대로 새 bounded root에 복사하고 원본/copy hash 및 registry pack resolution을 검사한다. trusted selection에 workload context를 포함하여 동일 full suite/profile/pair/issuer로 실제 base 검증을 수행한다. 원본 입력/registry와 unsupported delta는 그대로이며 native1000 fault의 별도 semantic context도 유지한다. 이 context producer만 교정한 변경 때문에 cadence/authority/Core 회귀를 반복하지 않는다.

## 2026-10-12 V13/V14의 남은 실행·수명 증거

기존 registry_contract/required_configuration/calibration_authority의11시험은 실제 실행 증거가 없고 release lib build에 포함되지 않는다. 해당3 integration target만 다음 기능 묶음에 추가한다. 준비된 issuer_pending/approved_lifecycle/cache_negative/cache_preaccept/worker_boundaries의5개 공개 probe도 같은 실제 Core context와 별도 finite fault context로 연결한다. Root가 Python plan 전체 diff와 실제 CLI 인수 순서를 검토했다. 현재 release matrix가 닫힌 뒤 통합하며 기능 시험과 성능 측정은 분리한다.

PE-CMVA-13의 영향받는 승인 무효화와 유효 증거 재사용에는 실제 동일 context HIT, copied source-only comment/profile revision/runtime RON의 의미가 같은 byte 변경/suite seed 변경 각각의 supervised MISS, 원복 HIT를 추가한다. Root가 lab.rs의 실제 criterion context를 확인했다. source만 변하면 같은 criterion을 사용한 새 실측·새 승인은 허용되지만 이전 accepted record의 HIT는 금지한다. profile/runtime bytes/suite가 변한 세 경우에는 실제 완료된 metrics/coverage와 미채택 criterion에 따른 승인 부재를 요구한다. transport 실패를 cache 무효화 성공으로 대신하지 않는다. 기존 authority 검사는 incompatible pair 경계를 재사용하므로 key마다 엔진 재빌드를 추가하지 않는다. accepted 원본 cache는 보존하고 각 변경은 사본에 격리한다.

PE-CMVA-14에는 실제 issuer job/approval/ApprovedHost/application fork/Trial 소유자의 마지막 해제를 Weak로 관찰하는 독립 cfg(test) 시험을 추가한다. 별도 issuer의 취소와 독립 Trial 실패가 기존 결과·승인·host/snapshot을 오염시키지 않아야 한다. 모든 정당한 owner를 해제한 뒤 generation은 회수되고 disk cache는 불변·재사용 가능해야 한다. 선행 token/Trial 수명 시험과 entry-sharing 회귀는 재사용하되 새 issuer 소유 경계를 대신했다고 하지 않는다. cache의128파일 및8MiB 정확 경계와+1 거부도 기존 공개 negative probe에서 실제 authentic record와 격리 partial/whitespace를 사용해 확인한다. job64/65 및 입력1MiB/128mod는 기존 준비된 검사를 사용한다.

Root가 build.rs의 실제 src 전역 hashing을 확인했다. cfg(test) 등록만 추가해도 EngineBuildIdentity는 변한다. 현재 release matrix는 중단하지 않고, 새 시험은 동결 입력 밖에서 작성한다. safe boundary에서 실제 diff를 검토·통합한 뒤 영향받는 plain/instrumented/test 빌드를 맞추고 expensive calibration을 시작한다. 이전 pair의 calibration을 새 identity로 옮기지 않는다. 이는 구체적으로 누락된 수용 조건을 보완하는 범위이며 전체 회귀 재실행 요구가 아니다.

첫 release libtests의32개 compiler error에 대해 Root가 원본 전체와 실제 identity.rs의 pair 제한을 확인했다. plain/instrumented 양쪽에 test-fixtures를 추가하거나 identity에서 이를 숨기지 않는다. 독립 author의6파일 FIX.diff 전체를 검토해 fixture API에 의존하는 기존 LB/player 및 observability 시험 모듈, playback의 해당1시험만 `test && (debug_assertions || test-fixtures)`로 구성했다. 기존 debug-default 시험은 동일하고, server의 명시적 test-fixtures가 Core 기능을 전달하여 해당 legacy release fixture 시험도 별도로 실행할 수 있다. test_support와 다른 parser 시험은 유지한다.

U11 normal_actor 시험은 빈 fixture token 대신 실제 production acquisition으로 얻은 반대 A/B token을 요청에 전달한다. 요청 identity가 현재 승인과 다른지 먼저 확인하고 실제 생성된 actor의 identity가 승인과 같고 요청 token과 다른지 직접 검사한다. A 복원 뒤 재생성 및 native fault 격리·원본 save 보존 검사는 유지한다. Root가 실제 A 복원 순서와 LB의 승인 generation 선택 경로를 확인했다. 제품 API/authority/기대 동작을 완화하지 않는 시험 교정이며 결과 필드도 실제 입력에 맞게 고쳤다.

다음 갱신은 실제 필요한 target으로 한정한다. instrumented worker-only는 고정된3 package/feature dependency graph를 유지한 mod_launcher build이며, plain libtests에는3 probe integration target(8+2+1)을 함께 컴파일한 뒤 실제 executable을 직접 실행한다. 새로운 bounded build-tool 옵션은 Sol이 소유하고 시험 oracle은 author가 유지한다. 모든 변경이 끝난 같은 source에서 plain/instrumented identity를 다시 확인한 후 context와 calibration을 진행한다.

## 2026-10-12 개발 profile v2 채택 및 불완전 조합·fault 교정

Root는 사용자 자원 위임과 실제 full cadence16/500/1000명·배치1/8/8명·60×1000ms 실현 및 build022의 반복 release 관측에 근거하여 명시적 DEVELOPMENT `CMVA-authoring-dev-v2`를 채택한다. 기존 v1에서 authoring_execution.max_clone_slots만65,536→1,000,000으로 달라진 정확 관찰 RON(SHA256 `fbd5a60a12dc3bc36080880f96d31e3817abe7a0247f45e45eb4fab197ef9de3`)이다. 실제 최대 clone486,932에 약2.05배의 유한 여유를 두며 정상 소유 상태 복사의 증가를 반영한다. source014의 불필요한 entry snapshot 복사는 먼저 제거했고 기존65537/65541 거부는 보존한다. release 전체 heap peak약923MB·실행약3.17초·해제 후 live약155KB도 확인했다. embedded/출하 기본값·native work10m/extent16MiB·expression·trial 및 나머지 필드는 그대로다. 전체 지원 규모·성능 criterion·승인 완료로 해석하지 않는다.

Sol은 다음 안전한 쓰기 경계에서 config/runtime_safety_cmva_authoring_dev_v2.ron에 정확 같은 bytes를 추가하고 v1/관찰 원본을 보존한다. 실제 build fingerprint 영향 여부를 확인한다. 기존 calibration의 raw profile hash·suite·profile_revision·worker identity는 바꾸지 않으며 관찰 경로는 같은 채택 bytes의 역사적 별칭으로 유지할 수 있다. 명칭 정리만으로 측정을 다시 실행하거나 이전 evidence를 새 identity로 옮기지 않는다. 정식 criterion은 아직 미채택이며 조합32의 완전한 측정·반복 안정성·분리 가능한 경계 확인 뒤 결정한다.

64효과 조합의 incomplete worker 실패를 보고 Root는 별도 이름의32효과 진단을 수용했다. 각 mod는1개의 Damage1과31개의 합법 Damage0을 사용하고 대표 명부·배치·60초 cadence·suite/profile/native10m/issuer/worker는 동일하다. 새 root와 registry/config/plan 및 원본·파생 입력 hash를 생성한다. 원래64효과 원본과 실패는 불변이고,32효과 A/B 양성과 AB 초과는 실제 관측 전 가정하지 않는다. 큰64/256/1024 stress 자동 확대는 하지 않는다.

native1000 시험은 준비 단계에서 막혔으므로 테스트용 engine 계측 hook을 추가하지 않는다. 공개 TrustedBase/Candidate·원래 finite WorldLabPreparation·정상 App Start/VN/starter/map 진입을 A/B 동일 cap에서 확인하고,1000…10m의 유한 범위에서 준비 가능 cap을 진단한다. 같은 cap의 실제100ms host tick에서 후속 WorkLimit이 나타나야 유효 gap 후보이며, 그렇지 않으면 gap 없음으로 기록한다. 원시 typed failure와 profile bytes를 보존한다. 이 진단은 승인·정상 actor 검증을 대체하지 않으며 최종 선택 profile에서 fresh calibration/criterion/approval와 원자성·fault 격리를 별도로 확인해야 한다.

source015에서 제품 v2 config가 실제 추가됐다. Root가 PRODUCT_DIFF 전체와 PREPARATION 원문 및 build.rs의101–135행을 읽었다. 해당 fingerprint는 Core/src·resources/data·server/src와 명시 manifest/lock/기본 runtime_safety.ron/Cargo 설정만 포함하며 새 명명 profile은 포함하지 않는다. 정확1608B/fbd5a60a…는 측정 원본과 같다. 현재 worker 재빌드·기존 측정 재실행 없이 같은 bytes의 개발 설정 채택을 유지한다.

## 2026-10-12 실측 기반 개발 criterion 수치 결정

Root가 새32 A/B/AB의 실제 calibration 결과에서72개 sample(각 plain3+instrumented3회×4phase)과 coverage34/34/68행을 직접 읽었다. 모두 metrics.complete=true, overflow=false, unsupported=[]이고 유일한 미완료 사유는 criterion 미채택이다. 각 mod의 effect0은17회 consumed, effect1…31은 각각17회 legal_noop이며 equipment owner/activation도 실제544회(527noop+17effect)를 기록했다. 기존15개 완료 측정과 같은 전체 suite/profile/worker/서버 owner다. 아직 approval은 없다.

Root는 개발 criterion revision `cmva-dev-cadence-v2-20261012`의 네 값을 결정한다: max_elapsed_ns=6,000,000,000; max_requested_heap_bytes=1,342,177,280(1280MiB); max_rss_bytes=1,610,612,736(1536MiB); max_native_work=230,000,000. 실제 comparator는 각각의 phase sample에 적용하며 전체4phase 합계나 native 단일 operation cap이 아니다. runtime native10m은 그대로다. 서로 다른 owner는 같은 수치를 사용하되 실제 owner의 fresh supervised calibration으로 각각 채택하고 issuer/context의 결속을 유지한다.

32 A/B/AB의 execute native work는219,997,861/219,947,875/234,667,723으로3회 반복 모두 같다.230m은 단독 A 최대에서약10.002m(4.55%) 여유, 정상 combined214,704,784에서약7.12% 여유를 두고 AB는약4.668m(2.03%) 초과한다. 따라서 불안정한 wall-time 차이를 조합 거부의 근거로 쓰지 않는다. 실제 최대 시간3.248초에6초, heap peak935,367,009B에1280MiB, RSS lifetime peak1,077,645,312B에1536MiB의 별도 여유를 둔다. 이 값은 해당 개발 환경·suite의 versioned engineering criterion이며 출하 성능 SLA나 임의 규모의 지원 선언이 아니다.64효과 A/B의 완료 관측은230m을 넘을 수 있으며 과거의 수집 완료를 성능 PASS로 고쳐 쓰지 않는다.64AB native 거부도 그대로 보존한다.

Author는 기존 EXECUTION_PLAN의 placeholder를 수정하지 않고 별도로 고정·결속할 numeric files와 실제 replacement argv/env·의존 순서를 준비한다. 새32 AB를 실제 normal_update의 complete-combination 거부 사례로 연결하고 같은 criterion 아래 base/각 단독 및 정상 A/B/C가 실제 통과해야 한다. fresh adoption/acceptance·경계/one-over·cache·save/actor/snapshot 수명 시험의 실행 결과가 필요하며 수치 결정만으로 이를 PASS 처리하지 않는다. native fault 전용 profile의 criterion은 유효 gap과 별도 실측 후 판단한다.

Root가 새26명령의 실제 argv/env 전체, data producer168행, 원래 plan과의320행 변경 검토,10파일 inventory 및 실제 numeric/args/update payload를 읽고 수용했다. 새 inputs는 별도 contexts001-reviewed-inputs에 결속하며 기존 contexts는 불변이다.9관리(adoption/acceptance),13기능,1fresh lifecycle/comparator,3cache 명령을 순차·의존 조건별로 실행한다. 이미 완료된11integration·issuer_pending·기존수명2개는 재사용한다. 이 작성·수용은 실행 또는 승인 PASS가 아니다.

공개 native-gap 첫 진단은26시도에서 준비가능17761/준비거부17760 경계를 얻었으나 뒤40×100ms에서 gap을 찾지 못했다. Root가 실제 RESULT 및17760 B의 world.prepare 실패(application_deployed_context,requested17761)를 읽었다.17761 A의 tick039 원문은battle_time4000ms/to_seq2/새event없음, 배치0·적0·defense object1, normal_corroded_patrol Defense의 계속 실행 상태다. 따라서 원래4초 시험은 실질 적 등장 이전 idle 구간에 머문 것으로 관찰된다.17761을 장애 cap으로 채택하지 않으며 첫 no-gap 원본은 보존한다. author는 실제 spawn 일정 확인 후 동일17761 및 부모10m에서 최대60초(600×100ms)의 별도 유한 진단을 준비한다. 반복24분할은 하지 않고 실제 spawn/작업 이후 fault 또는 정상 종료를 관찰한다. main26묶음과 독립이며 입력 고정 구간에는 외부 stage에서만 작성한다. 최종 fault profile·Lab 승인·정상 actor 격리 증거는 여전히 미완료다.

Sol이 literal launcher/server 진입 실행의 누락 가능성을 반환하여 Root가 P2G50의 Consumer Verification Boundary와 live_data 필수차원 원문을 확인했다. 기존26은 실제 AdmissionSession·persistence·load-balance/player actor와 깊은 상태 전환을 검증하지만 literal mod_launcher CLI dispatch 및 main game_server bootstrap 실행 증거로 대체하지 않는다. author는 기존 U02 외부 harness와 실제 CLI/boot를 읽고 해당 진입 연결·유효 외부 입력/잘못된 startup 거부의 최소 추가 검사를 준비한다. 해당 실행파일 자신의 calibration/issuer context가 필요하면 fresh로 만들며 normal_save 또는 cfgtest criterion을 이식하지 않는다. 동일 suite/profile/worker와 기존 실측 근거를 재사용하고 새 owner의 실제 완전한 metrics/승인을 확인한다. 깊은 save/actor lifecycle은 기존 묶음의 유효 결과를 재사용하며 추가 진입 검증 때문에 전체 검사를 반복하지 않는다. 현재26 실행 입력은 유지한다.

Root가 PUBLIC_ENTRY_PLAN.json과 실제 mod_launcher/main/bootstrap을 읽고 최소 범위를 수용했다. 두 실제 실행파일 각각의 새 issuer에서 combined calibration과 fresh adoption, launcher validate/query/apply/run/continue, 정상 loopback server의 READY·WebSocket Start·프로세스 재시작·Continue, 각 owner의 잘못된 content root에 대한 persistence 생성 이전 거부를 검증한다. 재사용하는 것은 수치와 기존 suite/profile/worker 및 깊은 lifecycle 증거이며 criterion/권한 cache는 이식하지 않는다. WS command correlation·복원 run identity/의미 있는 snapshot·읽기 전용 checkpoint 보존을 확인하고 timeout/강제 종료를 성공으로 세지 않는다. 기존 U02 스크립트는 import 시 실행 부작용이 있어 protocol 패턴을 재사용한 유한 stdlib driver를 외부 stage에서 작성한다. 순서는 현재26 완전 종료→수용된 WINDOW60→새 공개 진입 묶음이며 현재 입력을 변경하지 않는다.

현재26의09/12/13/14 실패 원문과10/11 의존 보류를 Root가 확인했다. 독립 author의 원인 반환 후 Root가 ApprovedHost::prepare_with_future_generation 전체·World prepare guard·actor 대응 경로를 직접 읽어 제품 결함으로 판정했다. helper는 모든 명령에 Some(future generation)을 전달하지만 World는 StartNewGame 이외에서 이를 InvalidAction으로 거부한다. 따라서 정상 Start 후 VN advance에서 멈추며 crash14도 기대한 저장 stage86 대신 준비 실패101이다. Sol은 Start에서만 미래 generation/required를 사용하고 다른 명령은 None/현재 required를 유지하도록 좁게 교정한다. 기존 실패 시험과 durable B/future C가 회귀 검증을 담당하며 기대값을 완화하지 않는다. 현재26 및 이미 준비된 WINDOW60를 같은 Engine에서 닫은 후 교정·후속 harness를 묶어 재빌드한다. EngineFingerprint가 바뀌므로 현재 Engine의 fresh calibration/issuer 승인은 다시 필요하나, 비영향 코드·기존 실측의 수치 근거는 재사용한다.

Root가 추가16/20/21 실패 원문과 해당 실제 시험 코드를 읽었다.20은 기존 host가 issuer Arc를 보유한 채 동일 exclusive cache를 재개방한 시험 수명 오류여서 기존 stale·원본 host·accepted bytes 검증 뒤 host/issuer를 모두 drop한다.21은 pre-run briefing을 fork한 뒤 두 번째 Start를 시도한 불법 준비 명령이므로 실제 VN snapshot의 합법 advance/choice로 fork만 변경하고 기존 독립 상태·모든 last-pin 검증을 유지한다. Root가 두 파일의41행 staged diff 전체를 수용했다.16의 replace 원문은 metrics complete인 base native230,992,140>230m이 유일한 실패 사유이며 change DependencyHeld가 올바르다. Root가 semantic_base의 전체 정의를 읽고 Resource 검사에 무관한 Employee/Equipment/SkillFragment/Relic/Pve removal-control 정의가 섞여 있음을 확인했다. unsupported add/replace/remove만 Resource를 보존한 별도 trusted-base successor로 격리하며 full suite/profile/230m·basePassed/changeIncomplete 기대는 유지한다. 이는 원래 과한 base의 성능 PASS를 주장하는 변경이 아니다. 기존 실패/입력을 보존하고 변경 후 실제 양성 base 및 unsupported 결과가 필요하다.

## 2026-10-12 Resource 실측 반영 criterion v3와 정상 actor 장애 입력

build030의 실제6문맥·각 plain3/instrumented3·4phase를 Root가 검토했다. 모두 complete이며 누락/overflow/불확실 창이 없고 아직 criterion 미채택이다. normal-save/server/launcher/game-server의 execute native는214,704,784, Core는 별도 candidate의213,834,244, Resource-only base는230,987,820으로 각각3회 동일하다. 최소 Resource base도 기존230m을 초과하므로 unrelated 정의 제거만으로 해결됐다는 가정은 폐기한다. 원래 Resource add의231,018,312 및 mixed base의230,992,140 실패도 보존한다. 실제 resource 준비는 nonempty catalog에서 RuntimeResources를 만들고 source occurrence와 소유 그래프·복사/복원 비용을 추가한다(runtime_facts::prepare_runtime_resources, resource/store.rs의 사전 계량). 이는 실행되는 추가 경로의 근거이며 전체 증가량의 site별 원인을 정량 증명했다고 하지는 않는다.

사용자의 실측 기반 자원 위임에 따라 새 개발 criterion `cmva-dev-cadence-v3-20261012`는 native phase 한도를233,000,000으로 결정한다. Resource base에2,012,180(약0.87%) 여유를 두고, 기존 반복 측정의32효과 AB234,667,723은1,667,723 초과로 분리된다. 이 여유는 같은 유한 suite의 결정적 작업 계량을 위한 것이며 임의 콘텐츠 성장 보장이 아니다. 32 A/B 및 AB는 현재 Engine·동일 v3 아래 실제 양성/거부 검증을 유지한다. 기존230m 결과는 재분류하지 않는다. elapsed6,000,000,000ns/heap1,342,177,280B/RSS1,610,612,736B는 유지한다. 새6관측의 최대시간3,181,456,169ns/heap923,001,836B/RSS lifetime1,064,108,032B가 이를 뒷받침한다. runtime v2 bytes, native operation10m, 전체 workload·coverage·원자성·취소·경계 기대는 불변이다. 각 owner는 자신의 fresh calibration으로 채택하며 숫자 결정 자체는 승인 PASS가 아니다.

배치 후 진단에서도 부모10m/17761의 A/B 모두600tick·실제 공격/피해를 완료했으나 fault가 없다. operation별 예산은 tick 간 누적되지 않으므로 단순 시간 연장이나 추가 cap 탐색은 하지 않는다. 정상 actor 장애 검증은 별도 expression fixture로 준비한다. records_clerk의 기존 training_berserker Skill/fragment를 명시적으로 교체하고 resonance_max1을 사용한다. 같은 후보의 Lab은 기존 healing staff(SpellPower+10)를 합법 장착해 실제 self-target DamageExpr/fragment의 전체 changed coverage를 달성하고, 정상 starter는 원래 sword/SpellPower0으로 반환된 readiness를 확인한 뒤 ActivateSkill에서 동적 DivideByZero를 관찰한다. 정확 runtime v2는 그대로이며 새 suite·candidate의 측정/기준/승인은 따로 필요하다. Root가 기존 application/fault_tests의 실제 동적 오류와 tests/combat의 typed expression 및 공식 loadout/staff 정의를 읽고 설계를 수용했다. author가 외부 stage에 작성하고 Sol이 실행한다. 준비 실패나 coverage 누락은 성공으로 대체하지 않는다.

Core private complete-root 원자성은 기존 U03 증거를 재사용한다. 실제 승인 actor는 모든 이용 가능한 공개 snapshot/fact/resource, 정확 durable bytes, 최초 typed fault, B 교체로 지워지지 않는 fault, mutation/export 거부와 무관 actor 격리를 검증한다. 공개 관찰을 private 전체 root fingerprint로 표현하거나 시험용 Core hook을 추가하지 않는다.

## 2026-10-12 합법 Start 문맥과 공개 진입 결과

현재 reviewed_adoption_functional002의 닫힌13/14/18 실패 원문과 실제 시험/명령 권한을 Root가 검토했다. 완료된 ViewingMap A에서 다시 Start를 호출하는 시험 입력 오류이며, replace_active_run=true도 Start의 NotStarted 문맥 요구를 우회하지 않는다. 제품 API를 바꾸거나 거부를 성공으로 처리하지 않는다. 정상 save 시험은 현재 A의 durable revision을 가진 새 NotStarted shell에서 명시적 교체를 진행하고, 기존 live A는 전후 상태/identity를 유지한다. B를 시작한 뒤 C를 설치한 상황의 실제 B 저장은 반환된 장착 item/owner UUID의 합법 UnEquipItem으로 수행한다. 최초 저장에는 별도 SavePoint가 없어 LoadRunCheckpoint를 그 증거로 사용한 기대를 교정하며, 원래 B required·durable revision/bytes·동일 원본 Continue와 C draft 폐기 보호를 유지한다. Crash child도 같은 합법 shell에서 실제 U04 seam의86 종료를 요구한다.

Core 공개 lifecycle은 B Apply 전에 A의 별도 NotStarted shell을 보유한다. 기존 실행 중 A는 Start(false)/Start(true)가 모두 거부되고 정확 public snapshot/identity가 유지돼야 한다. 보유한 shell에서 합법 Start(false)의 B draft/drop/commit을 확인하며 이 shell에 durable saved-run replacement 의미를 주장하지 않는다. 원래 완성된 A/B starter 흐름, issuer/capability/restart/cache/no-worker-rerun 검사는 유지한다. Root가 staged save374행 및 Core helper94행/lifecycle195행 전체를 수용했다. 현재 묶음 완전 종료 후 expression actor/worker 준비와 함께 통합하며 실패 원본은 보존한다.

닫힌 normal_update 결과에서 v3의 실제 base/A/B native는212993604/219997861/219947875이고 각각 PASS, AB는234667723으로3회 동일하며 유일한 거부 사유는233000000 초과다. AB coverage68행·계량이 완전하고 시간/heap/RSS 초과나 missing/drop/overflow/uncertain은 없다. Root가 해당 실제 raw를 읽어 수치 변경 뒤 조합 거부가 유지됨을 확인했다. Resource unsupported base/change 검사도 포함한 normal_update 전체 exact1 시험이 통과했다.

Literal smoke의 실제 RESULT/명령 및 WebSocket 기록을 Root가 확인했다. launcher는 seed1에서 Start/VNAdvance/StarterSelect의3명령과 Continue를 완료했다. 서버는 실제 auth·상관된3개 시작 명령 후 ViewingMap, 정상 SIGINT 종료0·reap, 재시작 NotStarted/Continue available, ContinueRun 뒤 동일 run_seed16733582392344865051/floor_seed12133471068290675166을 반환했다. 서버 command_count4는 시작3+Continue1이며 추가 VN 명령이 아니다. 두 invalid-content 실행은 intended trusted_root 원인으로 각각2/1 종료하고 persistence 생성 이전 거부를 검증했다. driver는 exact checkpoint 보존을 확인했고 강제 종료가 없다. 이는 bounded public entry wiring의 PASS이며 깊은 lifecycle·성능·Goal 전체 완료를 대신하지 않는다.

시험 author의 V01–15/10필수차원 중간 coverage 검토는 기존 유효 registry11, producer/entry-sharing/retreat, issuer/cancel/stale/pending/lifetime 및 U02/U03/U04/U07M/U10 seam 증거를 재사용할 수 있음을 확인했다. 남은 검증은 실제 expression actor/worker, 새 server owner의 failed13/14, Core lifecycle18와 dependent19–21, 최종 영향/currentness·복원·독립 감사다. 이 검토 자체는 최종 감사가 아니며 새 광범위 suite 실행 근거가 없다.

## 2026-10-12 준비 전제 교정과 새 서버 default 기준 유지

Root가 build033의 실패 원문·session.rs guard·author의 두 Python diff와 공통 준비 전제를 검토했다. `individual_suite_paths`에 실제 registry에 없는 policy key가 들어가 A/base가 거절됐고, Staff의 (3,2)는 seed77의 실제 배치 zone 밖이었다. 각 registry의 정확한 key만 연결하고, 실제 공개 Platform의 비점유 (3,7)을 사용한다. Staff의 SpellPower+10, 같은 caster의 HP10 감소, Sword0의 typed DivideByZero, profile·시계·전체 oracle는 유지한다. 수정은 시험 준비 두 파일에 한정되며 Rust 재빌드가 필요하지 않다. worker의 실제 준비/시전 전제를 먼저 확인한 뒤 세 측정을 수행한다. 기존 실패 원본과 policy/default 성공은 별도로 보존한다.

새 server owner의 default 관측은 native214,704,784가3회 동일해 이전 동일 workload와 같았다. Plain execute 최대3,865,314,368ns, instrumented 최대3,026,244,857ns, heap peak923,001,830B, RSS lifetime peak1,064,062,976B이고 drop live heap155,047B다. 모든 phase가 complete이며 누락·overflow·불확실 창이 없고 coverage6행이다. Root는 같은 `cmva-dev-cadence-v3-20261012`의6s/1,342,177,280B heap/1,610,612,736B RSS/native233m를 유지한다. RSS 잔류·lifetime HWM을 live allocator로 해석하지 않으며 세 표본으로 임의 부하 성능을 보장하지 않는다. 실제 새 owner가 자신의 기존 관측 issuer에서 fresh 채택해야 하고, 수치 검토를 gameplay PASS로 세지 않는다. Expression/base-policy 기준은 아직 결정하지 않았다.

교정된 Core lifecycle와 cache negative/preaccept/context의 실제 RESULT를 Root가 검토했다. 진행 중 A 보존·미래 B·재시작 exact HIT, 위조/stale capability 거부, cache 변조5개와 유한 경계4개, 실제 validation 뒤 preaccept 생성 실패의 원본/host 보호, context miss4개와 control HIT5개·복원 HIT4개가 각각의 시험 범위에서 통과했다. Preaccept 검사는 post-rename durability 실패를 주장하지 않는다. 제품·시험 코드의 기존 검토와 이번 결과를 연결하며 failed sibling이나 Goal 전체를 PASS로 승격하지 않는다.

## 2026-10-12 hostile 표적과 실제 오류 시점 교정, 저장 결과 검토

build035에서 준비는 통과했으나 self damage의 readiness가 거절됐다. Root가 sim의 final usefulness와 target_usefulness를 직접 읽어 같은 owner에 대한 DamageExpr가 합법 적대 피해 표적이 아님을 확인했다. 시험은 기존 스킬/fragment 교체 범위 안에서 player_unit/EnemySingle/global/Instant를 사용한다. 정상 첫 wave 5000ms 뒤 반환된 살아 있는 Opponent UUID를 선택하고 owner·HP>10·readiness·begin targeting·preview enabled를 확인한다. Staff는 같은 선택 표적의 HP10 감소와 caster 출처 Ability raw/final10 이벤트를 검증한다. Sword는 SpellPower0의 Evaluate/DivideByZero를 검증하며 runtime v2 한도와 기존 원자성/격리 oracle는 유지한다. 이전 self-target 설계/실패를 소급 PASS로 바꾸지 않는다.

Root는 start_admitted_skill_cast, World Activate 및 서버 ExecutePlayerBehavior/예약 tick을 직접 확인했다. Activate는 이벤트를 예약하므로 성공해야 하며, 실제 수식 오류는 그 다음 elapsed1ms(step13) 또는 정상 HostTick에서 발생한다. Actor는 paused 상태에서 시전을 승인한 뒤 ordinary ResumeBattle handler와 공개 상태/file-byte 읽기를 같은 mailbox 호출에서 수행한다. 이 읽기 adapter는 production handler를 그대로 호출하고 private state/tick을 조작하지 않는다. 실제 예약 HostTick 이전의 post-resume baseline을 비교하므로 정상 Resume 효과와 fault 원자성을 혼동하지 않으며 paused=false도 비교에 남긴다. 최초 fault의 operation=host_tick, typed cause와 후속 gate/B 교체/무관 actor/재생성 Continue oracle를 유지한다. Root가 최종3 diff를 검토해 worker 직접 전제 확인부터 순차 실행을 허용했다.

build036의 actual source와 public 결과를 Root가 검토했다. B에서 소유한 armor UUID f5725170-949c-41e8-875e-4566a5164051을 정상 UnEquipItem으로 해제해 owner가 null이 되었고, revision과 payload hash가 모두 바뀌었다. 새 durable B를 Continue한 inventory가 같은 결과다. C 설치와 C replacement draft 폐기 뒤에도 A/B의 기존 상태와 저장을 보존한다. Crash 시험은 다섯 실제 저장 seam·여섯 child exit86을 요구한다. 승인 이전 B 거부/같은 A Continue, 승인된 exact old/new 복원·두 copy 순서·profile 복원·B-only 재시작, 다른 payload/third configuration/unsupported/권한 없는 intent·legacy 거부 및 정확 원본 보존을 확인했다. Legacy V1은 실제 같은 승인 identity에서만 복원된다. 이는 저장2개와 fresh default 채택의 검토 완료이며 expression 또는 Goal 전체 완료를 뜻하지 않는다.

## 2026-10-12 hostile 실제 실행과 두 suite의 개발 기준 유지

Root가 build039의 실제 공개 입력/결과를 확인했다. 정상 표적 a1fbb727…의 HP90→80, caster14ea19d0… 출처의 seq15/time5001 True Ability raw/final10 이벤트가 하나이며 Sword Activate는 accepted BattleSkillActivated다. 다음 step13 elapsed1ms의 오류는 opponent_check.amount의 Ratio denominator zero, Evaluate/DivideByZero다. 이는 직접 전제 검증이며 paired calibration/승인은 아니다. build040의 전체 worker에서는 plain/instrumented 각3개의 실제 Failed envelope가 정상 프로세스 종료0/reap 뒤 content_failed/lab.execute/value_expression_fault를 반환했다. Crash/timeout/parse 오류를 대신 인정하지 않는다. 0755와 symlink root는 worker.root에서 materialization 이전 거부되고 실제 root/target 상태가 보존됐다. 별도 private 양성 calibration은 complete와 실제 HP10을 충족했다.

현재 server owner d4ecc73c…의 expression A/B 및 base/policy 네 관측을 Root가 phase별 plain3/instrumented3으로 검토했다. 각각 coverage5/5/3/3, unsupported 없음, 계량 complete·누락/overflow/불확실 창 없음이며 criterion 미채택만 incomplete 사유다. 네 관측의 acquisition native는5,630,544로 각3회 동일하고 expression setup20,255/execute152,638, base/policy setup18,922/execute0이다. Plain 최대434,718,868ns, instrumented 최대506,696,402ns, heap15,841,522B, RSS lifetime35,614,720B, drop live heap 최대78,969B다. Plain native0은 비계측 표지이며 실제 비용0 주장이 아니다. Base/policy의 execute0은 변경된 수식 실행을 요구하지 않는 별도 준비 suite의 범위이고 expression의 실제 효과 증거를 대신하지 않는다.

Root 결정은 **기존 개발용 v3 유지**다. 두 exact suite(3885d402… expression /1cb1688c… base-policy)에 `cmva-dev-cadence-v3-20261012`의 elapsed6,000,000,000ns, heap1,342,177,280B, RSS1,610,612,736B, native233,000,000을 각각 fresh 채택한다. 이는 이미 더 큰 정상 workload와 AB 초과 거부에서 검증된 개발 admission 예산을 유지하는 선택이다. 작은 세 표본에 맞춘 새로운 공통 성능 합격선을 만들거나 비용 감소를 최적화 성과로 주장하지 않는다. 증거상 상향 필요가 없고, 좁은 fault fixture의 비용으로 임의 콘텐츠의 성장 여유를 보장하지 않는다. Runtime v2/operation 한도와 기존 실패·coverage·원자성은 불변이다. 실제 각 suite/issuer/현재 executable 관측의 새 채택과 A/B 승인·actor 실행은 별도 필요하며 캐시 권한을 복사하지 않는다.

## 2026-10-12 정상 employee 공명 준비 교정

041의 실제 DTO는 battle_time5000, Sword records_clerk가 배치됐고 resonance_current0/max1, resonance_not_full, target_available=true였다. Root가 skill_fragment.rs407과 resonance_runtime182–225를 직접 확인했다. Active form은 start=max/5이므로 초기값은0이며, 다음 정수 초의 정상 employee pulse가1을 더한다. 이전 base20 clamp 설명은 실제 active-fragment 경로에 맞지 않아 폐기한다. Lab은 배치 뒤5초를 진행해 자연 충전됐지만 actor는5초에 적을 기다린 뒤 배치하고 즉시 pause 상태였다는 차이다.

Root가 author의 actor-only diff 전체를 수용했다. 정상 ResumeBattle→유한 public readiness 대기→같은 mailbox에서 현재 공개 authority를 읽어 ordinary PauseBattle handler 호출로 준비한다. initial0/max1·동일 unit의 유일한 ResonanceChanged0→1·최종 paused/manual-ready를 실제로 확인한다. Private 상태/tick 주입이나 오류 억제 없이 정상 pulse만 사용한다. 이후 성공 Activate·post-resume baseline·HostTick typed fault/전체 공개 원자성·B 교체/격리·정확 저장/Continue 기대는 유지하며, 후속 mutation 거부는 generic error가 아니라 정확 battle_faulted 코드로 보강한다.

제품/fixture/suite/worker/한도는 바뀌지 않는다. Root는 calibrate_and_adopt가 매번 실제 LabRun::calibrate를 수행한 뒤 해당 owner 기준을 채택함을 확인했다. 따라서 새 server owner에서는 기존040 수치 검토를 재사용하고 새 채택2에 포함되는 fresh 측정과 A/B 승인·actor를 수행한다. 형식적인 executable 변경만으로 별도 네 관측이나 worker/default를 반복하지 않는다.

## build043 실제 경계 검토 — 2026-10-12

Root는 `product_parts/build043/RESULT_001.json` 및 `test_parts/runtime001/evidence/reviewed_expression_adoption_actor002/nonsecret_reports001`의 실제 JSON을 읽고 test source `shared/normal_actor_tests.rs`의 종료 oracle와 대조했다. 새004 normal_host executable은 `12bfbc214ee06fcff53cda5c7e13a94331ec13c0b88b5d7bf46f1699fcdb6f6d`, Engine/graph는042의 실제 비영향 대조 그대로다. fresh 채택 내부 calibration은 두 suite 각각 plain/instrumented3회·4phase·complete, coverage3/5와 overflow/누락/불확실성 없음이다. 최대 plain437890153ns, instrumented507456352ns, heap15841552bytes, RSS35614720bytes, drop heap78999bytes로 기존 v3를 유지한다. calibration raw outcome의 `criterion not calibrated/adopted`는 채택 전 관측 상태이며 이를 승인으로 인용하지 않는다. 후속 A/B의 base/expression/policy 개별 job 및 최종 조합은 실제 `passed`/reasons[]/complete로 확인했다.

actor는 합법 배치 후6000ms의 actual ResonanceChanged(seq46,0→1,max1)로 준비되고 일반 Pause/target preview/Activate/Resume를 거쳤다. Resume는6000ms/revision68에서 정상 accepted되고 다음 실제 host_tick은 expression/evaluate/divide_by_zero/node2를 첫 원인으로 고정했다. public before/after 전체 비교에서 차이는7개뿐이다: reconnect의 Ok→typed Err 두 key, top/run action_capabilities 비움, run.battle_fault 추가, can_retreat true→false, run context deployment 비움. runtime facts·typed/run resources 및 나머지 snapshot bytes는 동일하다. paused/playback을 임의로 지운 비교가 아니다. 기존 Core private complete-root 회귀와 이 공개 projection 검증의 범위는 구분한다.

실제 checkpoint export는 battle_runtime_fault/battle_faulted와 null checkpoint hash로 거절하고 tick_scheduled=false다. 후속 load의 정확 code도 battle_faulted다. 실제 B 승인·Apply는 fault A를 바꾸지 않았고 독립 B actor의 정상 Start는 변경 정책 자원777과 required B를 소비했다. 별도 actor, original A active/profile 파일·metadata, 기존 cache records는 비교 oracle로 유지됐다. A를 다시 승인/Apply해도 started B는 pinned되고 fault A는 바뀌지 않았다. 실제 Quit·deregister 후 같은 원본 저장으로 recreate/Continue는 정상 MapState이며 원본 파일은 그대로다. Root는 이 범위의 기능 결과를 수용한다. Goal 독립 완료·최종 source currentness·freeze/restore는 아직 아니다.
