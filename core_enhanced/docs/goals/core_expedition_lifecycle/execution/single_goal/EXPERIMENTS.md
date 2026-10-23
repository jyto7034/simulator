# U12 실행과 검증 기록

2026-10-12 GEX00 인수 당시 U12 제품/시험 runtime 변경·실행은0이었다. 이전 기능 실패·검사·감사 원본은 해당 U11 경로에 유지한다.

- [resume source4678현재성](../../RESUME_20261012_001.json): 메인 읽기/해시 비교,불일치0. 새 게임 시험이 아니다.
- [P2G40 준비된 실행](../../P2G_40_VALIDATION_001.json): Sol 단독 receipt생성→읽기검증,각exit0/passed/failures[].6WP/19edges/1SCC/3nodes/1phase/19integration. 문서 구조 검사다.
- [P2G50 준비된 실행](../../P2G_50_VALIDATION_001.json): Sol 단독 quality_contract2,exit0/passed/failures[].15정책/1Goal/15traceability/19integration/10dimensions. 제품 PASS가 아니다.

다음 기록은 baseline 실제 복원/currentness와 제품 상세 준비 인수다. 이후 각 실행 묶음의 정확 command/selector/fixture/hash/toolchain/exit/nonzero 수/raw 결과·실패/교정/재실행/재사용을 product_parts 및 test_parts 원본으로 연결한다. 일치한 성공 로그 전체를 본문에 복사하지 않는다.

- GEX00 baseline/준비 인수 완료: [RESULT001](GEX_00_RESULT_001.json). actual archive4732일치,영향633path(재사용591/추가11/부재31),runtime mismatch0. global status30초 timeout을 보존하고 scoped status로 한정했다. 새로운 게임 검사 없이 GEX10으로 진입한다.

## 2026-10-12 로드맵 검토 반영과 GEX10 재개

[PLAN의 반영 계획](PLAN.md#로드맵-검토-반영-계획--2026-10-12)에 따라 현재/이력 안내와 후속 계약 책임·후퇴 문서를 보완했다. 이는 문서 검토이며 게임 기능 PASS가 아니다. 연속 지침을 다시 읽고 마지막 제품/시험 안전중지 보고와 관련 source/기존 기록을 대조했다. baseline 영향 목록633경로의 파일 수정 시각에는 마지막 안전중지(11:45:16 UTC) 이후 변화가 없었다. 시각 확인은 byte manifest 비교를 대신하지 않으며 후속 실제 변경은 기존 baseline과 diff로 인수한다. 가시 프로세스는 sandbox 내부의 조회만 확인돼 OS 전체 부재를 주장하지 않는다.

재개 입구는 GEX10 부분 구현·신규 lifecycle 시험30개/public·host probe 작성이며 compile/build/test 실행0이다. 과거 P2G 문서 validator PASS와 U11 동결 결과를 현재 U12 제품 PASS로 사용하지 않는다. Sol 제품/단독 runner와 독립 Astra 시험 작성의 기존 역할·파일 소유를 유지하고 pending 결속·회복/HQ·reset·소비자/구형 경계·관련 시험의 미완료부터 진행한다. 첫 실행의 정확 명령·입력·선택 수·raw 결과는 기존 product_parts 실행 기록으로 연결한다.

시험 담당 재개 호출은 thread limit로 시작하지 못했다. 같은 지정 Astra high `/root/u12_tests_resume`가 기존 author의30시험/probe와 미완료를 인수했다. 모델/기대값 변경이나 시험 재작성 사유가 아니며 첫 실행 전 제품 파일 cfg(test)의 전체 파일 소유를 Sol과 순차 인계한다.

## 첫 제한 컴파일의 의존성 준비 실패

Sol 단독의 [compile001 원본](product_parts/compile001/RESULT_001.json)은 `cargo check -p game_core --lib --offline --locked`, 0.195초, exit101이다. 메인은 실제 결과와 stderr를 읽었다. `/tmp/wp02-cargo-home`의 오프라인 index에 `fs2`가 없어 의존성 해석에서 종료했으며 Rust 소스 컴파일이나 기능 시험에는 도달하지 않았다. 입력617경로 manifest의 전후 SHA-256은 `a7e89359c695ee6e9613fc6ae07f1b99f9f70eadf68d1240021b4d6b9cdc8c73`으로 같다. U11 compile004의 실제 CARGO_HOME/target도 같은 경로였음을 원본에서 확인했다. 해당 실패와 raw는 보존하고 의존성 준비를 복구한 뒤 같은 제한 검사를 수행한다. 이 결과를 제품 결함이나 기능 PASS로 분류하지 않는다.

그 뒤 동일 검사의 `compile002` 승격 도구 호출은 승인 대기661.9초 후 운영 중단에서 `aborted by user`를 반환했다. 이 문구만으로 사용자의 구현 거절을 추론하지 않는다. Sol의 상태 확인에서 해당 출력 디렉터리·PID·command session·raw가 없었고 실제 실행 시작 증거도 없었다. compile001 collector 세션은 종료·회수됐으며 남은 exec cell도 없었다. 가시 프로세스 조회는 sandbox 범위로 한정한다. 승인 대기 시간은 컴파일 시간에 합산하지 않는다. 입력 동결과 Sol 단독 실행을 유지하고 승인 사유를 분명히 안내한 뒤 같은 구체 명령의 승인 요청으로 복구한다.

두 번째 하위 agent 승인 호출도404.4초 동안 명령을 시작하지 못했다. 사용자가 승인 요청이 보이지 않는다고 답해 UI 미노출을 확인했고, Sol 턴의 승인 대기만 회수한 뒤 compile002 산출물·세션이 여전히 없음을 확인했다. 운영 복구로 root가 **Sol이 준비한 고정 collector와 같은 제한 명령의 승인 UI를 한 번 중계**했다. 이 요청은 승인돼 실제 exec 세션68865가 시작됐다. 명령 선정·collector·raw 결과 인수 책임은 Sol, 실제 도구 시작은 root로 구분한다. 이를 전부 Sol이 직접 도구 호출한 것으로 기록하지 않는다. source/test/fixture는 계속 동결했으며 root는 별도 검사나 기대값을 선택하지 않았다.

[compile002 결과](product_parts/compile002/RESULT_001.json)는0.250초/exit101/child_reaped=true이며 source617 전후 manifest는 compile001과 같다. 메인이 결과와 stderr를 읽었고 root가 세션68865의 collector 종료exit0을 회수했다. 승격 환경에서도 같은 `fs2` 누락이 재현돼 기존 캐시가 단순히 보이지 않는다는 가정은 성립하지 않았다. 두 검사는 모두 의존성 해석에서 끝났으며 기능 시험은0이다. 기존 제품 agent는 이후 thread limit로 재호출할 수 없고 live 목록에서도 사라졌다. 같은 지정 Sol6.1 xhigh `/root/u12_product_resume`가 원본·부분 소스·책임을 인수했고, 기존에 승인된 정확한 locked fetch로 의존성을 준비한 뒤 새 실행 ID로 제한 컴파일을 이어간다. Cargo.lock과 동결 입력을 준비 단계에서 바꾸지 않는다.

## 의존성 복구 후 첫 Rust 진단

새 Sol이 기존에 승인된 정확한 locked fetch를 직접 실행한 [fetch001](product_parts/fetch001/RESULT_001.json)은 exit0이며 `fs2 v0.4.3` 등 잠금된 의존성을 내려받았다. tool의 합쳐진 원문을 보존하며 별도 stdout/stderr 수집으로 가장하지 않는다. 이어진 [compile003](product_parts/compile003/RESULT_001.json)은194.919초/exit101/child_reaped=true이고, 실제 의존성 다음 `game_core` 컴파일에 도달했다. source617 전후 SHA는 여전히 `a7e89359c695ee6e9613fc6ae07f1b99f9f70eadf68d1240021b4d6b9cdc8c73`이다. 메인이 RESULT와17334바이트 stderr를 읽었다. 오류14개는 `MapNodeId`의 정렬 trait 누락, 실제 admin 소비 경로의 옛 모드 두 곳, 장비 메타데이터 Arc 인자, 새 프로필 설치 route의 exhaustive match 누락 네 원인이다. 이 실패 원문을 보존하고 제품 담당이 연결을 교정한다. 기능 시험은 아직0이며, 다음 작성 창은 독립 author와 전체 파일 소유를 조율하고 다음 실행 전 다시 writer0으로 묶는다.

## 첫 제품·시험 결합 컴파일

제품 연결 교정과 독립 시험36개 작성 뒤 [compile004](product_parts/compile004/RESULT_001.json)는 `cargo test -p game_core --lib --offline --locked --no-run --message-format=json`,56.571초/exit101/child_reaped=true다. source617 전후 SHA는 `a57ecfcda679e104893ff55d9913c618e45207423bf085929fd61d92727fb4ef`로 같다. 메인이 RESULT와 JSON compiler diagnostics를 읽었다. 기존 시험 여섯 파일의 제거된 심리/튜토리얼 완료 필드 참조 및 새 role/pending session/loan 필드 누락으로 오류7개가 남아 독립 author에게 파일 소유를 반환했다. 앞선 제품 오류가 이 진단에 없다는 사실만으로 컴파일 성공을 선언하지 않는다. `build-finished`는 false이며 실제 기능 실행은0이다. profile-only 거절 뒤 출발 전 Abandon을 성공으로 기대하던 시험 정리 코드는 공개 계약에 맞춰 새 title actor의 Tutorial 양성과 분리했고, 그 때문에 제품에 새 취소 기능을 추가하지 않았다.

같은 범위의 [compile005](product_parts/compile005/RESULT_001.json)는120.003초/exit0/child_reaped=true, source617 SHA `30527db996b931941623359d0265e4a6cb5aa9a8fac8fe58c9336b8281c6f0bc` 전후 동일이다. 메인이 RESULT와 `build-finished: success=true`, 실제 lib test artifact를 확인했다. executable은 `.cache/u11-mod-validation-target/debug/deps/game_core-705cc9514d94a1fe`이며 features=[], opt0/debug2/debug_assertions=true/test=true다. 이는 현재 제품+단위시험의 컴파일 성공이고 기능/성능·normal release 승인 증거는 아니다. Sol은 이 동일 binary에서 선택 목록을 확인한 뒤 lifecycle 및 직원 데이터의 준비된 표적을 실행한다.

## 첫 표적 실행과 공통 fixture 교정

같은 binary의 [functional001](product_parts/functional001/RESULT_001.json)은 lifecycle36+직원 데이터6의 두 filter를 `--test-threads=1`로 실행했다.42개 실행/14통과/28실패/0ignored/2015filtered,exit101/0.296초/child_reaped=true이며 source617 SHA는 compile005와 같고 전후 불변이다. 메인이 결과와 원문을 읽고 실패28건 모두 `candidate_2`의 역할 Tank와 검 무기 불일치인 동일 data validation 오류임을 확인했다. 이28건은 목표 행동 전에 공통 fixture 구축에서 실패했으므로 귀환·대여·저장 등 목표 행동에 도달한 근거로 쓰지 않는다. 독립 author가 역할별 장비와 실제 참조를 교정하며 제품 validator나 기대값을 낮추지 않는다. 통과14건도 각 typed/data oracle의 범위이며 전체 lifecycle 성공은 아니다.

역할별 fixture 교정 후 [compile006](product_parts/compile006/RESULT_001.json)은64.010초/exit0/reaped=true이며 source617 SHA `3ff2fe6284c4c82026a49cc08a571564455b7c87b110200e5733a4d461aae960` 전후 불변이다. 같은42개를 실행한 [functional002](product_parts/functional002/RESULT_001.json)는34통과/8실패/0ignored/2015filtered,exit101/4.855초/reaped=true이며 입력 SHA도 같다. 메인이 RESULT와11307바이트 stdout을 읽었다. 여섯 실패는 writer가 만든 현재 Continue의 지도 난이도와 전투력 authority 불일치이며, 강제 depth fixture가 없는 공개 출발→fresh Continue도 포함한다. 코드를 추적해 고정 슬롯 배치를 난이도 `authored_fixed`와 혼동하는 generator 분기 및 신규 원정 고정 배치/default_facility 조합을 확인했다. 제품은 배치와 난이도 정책을 각각 유지하는 방향으로 교정한다. SavePoint의 `CheckpointUnavailable` 한 건은 내부 검증 실패를 숨기는 wrapper이므로 독립 author가 지식 변경 전 writer payload의 검증 양성을 추가했다. HQ 인원 경계 한 건은 arbitrary 직원을 roster에만 더한 fixture의 queue/roster 불일치라 실제 정의와 정확 소유 목록으로 교정했다. 이 교정들의 실행 효과는 다음 결과에서 확인하며8실패를 해결됐다고 선기록하지 않는다.

교정 뒤 [compile007](product_parts/compile007/RESULT_001.json)은65.058초/exit0이고 [functional003](product_parts/functional003/RESULT_001.json)은42실행/42통과/0실패/0ignored/2015filtered,5.937초/exit0다. 두 child는 회수됐고 source617 SHA `77ac79108a003296fc2c8eb8cdfa9472c7f1fc5211e5c2bfbaa97fb0fe7f4b45`가 전후 같다. 메인은 두 RESULT와 기능 stdout의 실제 선택/통과 결과를 읽고 역할 fixture, HQ 정확 소유 및 generator의 `authored_fixed` 분리를 코드로 검토했다. 저장/Continue 양성에 도달하지 못했던 앞선 실패들은 이 동일 표적에서 해결됐다. 이42개는 lib test 범위이며 정상 공개3층 진행·실제 귀환, 서버 accepted crash/reset·byte 경계, 회복 행동·영향 회귀, 비용 및 최종 독립 감사는 남는다. 실행을 모두 회수한 뒤 독립 author에게 실제 fragment grant 충돌과 accepted reset crash/tamper의 다음 작성 창을 반환한다.

## 서버 결합과 추가 사본·본부 경계

정적 검토에서 깊이5 이후 HQ가 채용 세션 자체를 만들지 않아 비채용 보급/상점도 종료할 수 없는 연결을 발견했다. 공식 개발 원정은 매층 HQ를 가지므로 제품 `begin_recruitment_session`은 깊이 제한에서 후보0의 정상 세션을 만들도록 교정했다. 채용 가격/깊이 제한과 queue 보존은 유지한다. 독립 author는 실제 깊이5에서 HQ에 들어가는 두 활동 대조, 실제 reward fragment grant의 같은/이종 vault UUID 충돌·롤백·재시도, accepted corrupt-reset crash/tamper와 byte 경계를 보강했다. 서버 기존 cfg fixture의 제거 필드·역할 무기·모드/선발·구 opaque version 잔여도 현재 계약으로 이관했다.

[compile008](product_parts/compile008/RESULT_001.json)은 `cargo test -p game_core -p game_server --lib --offline --locked --no-run --message-format=json`,130.293초/exit101/reaped=true이며 source617 SHA `790f636465d100e5d1a62b086a77dba2182cba1746948ff2428806c16baa9dc2` 전후 불변이다. 메인이 JSON 원문 여섯 오류를 읽었다. server 시험/공용 support의 HostApplicationError 변환 두 곳과 현재 Result를 아직 bare outcome으로 비교하는 네 곳의 연결 오류다. 전체 build-finished는 false이며 server 실행은0이다. 이 invocation에서 실제 완성된 Core artifact만 별도로 인수해44개를 실행했다.

[functional004](product_parts/functional004/RESULT_001.json)은44실행/43통과/1실패/0ignored/2015filtered,6.291초/exit101/reaped=true이며 입력 SHA는 compile008과 같다. 메인이 RESULT·6912바이트 stdout을 읽었다. 신규 fragment grant 충돌의 원상태/재시도는 통과했고, 깊이5 HQ의 보급 분기 다음 상점 진입은 시험 shop catalog에 `wandering_monolith_shop`가 없어 실패했다. 상점 목표 행동 완료 증거로 쓰지 않으며 제품의 실제 참조를 유지한 채 독립 author가 fixture를 교정한다. 두 실행을 모두 회수한 뒤 server 여섯 연결 오류 및 이 fixture의 좁은 작성 창을 반환한다.

좁은 교정 후 같은 결합 명령의 [compile009](product_parts/compile009/RESULT_001.json)은81.881초/exit0/reaped=true이고 source617 SHA `00ca3bed485efdf5f5ea2870814d48da28bb96e0ab9cb42aa2182dde45836bdc` 전후 불변이다. 메인이 RESULT, compiler error0와 build-finished true, 실제 Core/server test executable 각각을 읽었다. 두 artifact는 features=[], opt0/debug2/debug_assertions=true/test=true다. Sol 턴의 일시적 모델 capacity 오류 중에도 입력을 동결했고, 같은 지정 Sol이 기존 exec 세션84406을 최종 회수했다. 중복 빌드나 별도 runner를 시작하지 않았다. 이 결과는 두 lib 시험 코드의 컴파일 성공이며 서버 기능·normal release 승인 증거는 후속 실제 실행에 결속한다.

[functional005](product_parts/functional005/RESULT_001.json)은 Core44개 중43통과/1실패로 남았다(6.469초/exit101). merchant 정의 누락은 해소됐지만 현재 깊은 층 첫 페이지가 상품3개를 요구하는데 fixture는1개만 제공해 `PoolExhausted`에서 끝났다. 메인은 `generate_wandering_first_page`와 `generate_shop_page_with_count`의 실제 후보 수/중복 없는 page 조건까지 읽었다. repeatable은 같은 페이지 후보 수 부족을 해결하지 않으므로, 관련 fixture의 상품 정의·참조·3개 offer·활동 종료를 묶어 점검하도록 다음 author 창에 반환했다. 이 상점 분기는 여전히 미완료다.

같은 입력의 서버 [functional006](product_parts/functional006/RESULT_001.json)은 등록11개 중 실제 비ignored3개를 실행해3통과/0실패/8ignored/102filtered,28.099초/exit0/reaped=true다. 메인은 RESULT와2243바이트 stdout을 읽었다. profile-only actor의 출발 미수락 및 memory/Tutorial 대조, current profile checksum 양성 뒤 raw duplicate UUID 거절, 정확16MiB writer/reload와+1byte direct/paired preaccept 거절의 세 범위다. paired byte 시험은 빈 영구 지식에서 출발해 합침으로 payload가 두 배가 되던 전제를 제거했다. 승인 설정이 필요한8개와 실제 crash child는 아직 실행되지 않았다. functional005/006의 source617 SHA는 compile009와 같고 전후 불변이다.

두 실행을 회수한 뒤 다음 작성 창을 연다. 제품 owner는 root workspace/lock 및 기존 CMVA build 지원의 정확 U12 package/manifest 등록, 독립 author는 공개/normal 공통 진행기·현재 suite 준비와 남은 HQ fixture다. 작은 시작→HQ→전투→저장/Continue 준비를 같은 실제 경로에서 먼저 확인하고 전체3층으로 이어간다. calibration의 기존 lab16/500/1000·배치1/8/8·60×1000ms는 일반 게임 보유10명과 다른 authoring 부하이므로 축소하지 않는다. 새 본부 선행 명령과 명시 mode/loan 입력만 현재화하며 실제 측정/채택 전에는 그 준비를 비용 또는 정상 승인 PASS로 쓰지 않는다.

## 회복 제품 교정 후 좁은 재검증

[compile010](product_parts/compile010/RESULT_001.json)은 같은 Core/server lib no-run 명령으로113.719초/exit0/reaped=true다. source618 SHA `12a2aaf98782e311afde5e34a886eec0ef233bc6fe059aaf4eb16b65cd9e2ce4` 전후 불변이며, 메인이 compiler error0/build-finished true 및 [실제 두 artifact](product_parts/compile010/ARTIFACTS_001.json)를 확인했다. Core SHA `02977f73…`, server `d71886ef…`, 각각 features=[],opt0/debug2/debug_assertions=true/test=true다. correction005의 제품7파일은 이 입력에 들어갔지만 신규 G07 독립 회복 사례는 아직 작성/실행 전이다.

[functional007](product_parts/functional007/RESULT_001.json)은 Core44개 중43통과/1실패/0ignored/2015filtered,6.282초/exit101/reaped=true다. 메인은 stdout과 실제 `DataValidationError`를 읽었다. 기존 무료 starter weapon을 상점 상품으로 사용해 base price0에서 fixture 생성이 거절됐고, 목표 HQ 진입/종료에 도달하지 못했다. 앞선 정의·상품 수 교정만으로 성공 fixture 전제가 완결되지 않았음을 기록한다. 별도 유효 상품/상점 fixture의 참조·가격·페이지 및 종료 조건을 함께 대조하고 다음에는 문제1case를 먼저 exact 선택한다. 기존43개를 다시 실행해야 한다는 범위로 자동 확대하지 않는다.

[functional008](product_parts/functional008/RESULT_001.json)은 같은 입력의 server3통과/0실패/8ignored/102filtered,28.133초/exit0/reaped=true다. 메인이2243바이트 stdout과 RESULT를 직접 확인했다. 이전 세 boundary scope를 현재 shared adapter 변경 뒤 재검증한 것이며, 승인 설정을 요구하는 ignored 부모/child는 여전히 미실행이다. functional007/008은 compile010과 같은 source SHA를 전후 유지했고 두 세션 모두 회수했다.

다음 좁은 작성 창은 HQ fixture와 공개 direct probe의 외부 loader 입력이다. 독립 author가 기존 `load_live_embedded`의 cfg(test/debug/test-fixtures) 제한을 발견했고 메인이 실제 선언을 확인했다. 요구된 default-feature release는 현재 공식 외부 pack과 runtime profile을 공개 loader로 읽어야 하며, test feature나 embedded fallback으로 회피하지 않는다. 작은 공개 경로·normal approval/calibration과 남은 G07/G09/G11은 아직 완료하지 않았다.

교정은 기존 유효 priced Armor metadata 전체를3고유 definition/UUID로 복제한 merchant fixture를 재사용했다. 실제가격80×3·구매80차감·새사본 정확1개/definition·종료/큐 보존을 한 성공 사례에 연결했고, 메인이 source와 실제 command/DTO를 대조했다. [compile011](product_parts/compile011/RESULT_001.json)은 Core-only no-run으로94.688초/exit0/reaped=true, source618 SHA `f64f54f0e581212051a4c0b3c733f6aa4983f7940f5a834e4be6284055cd3d28` 전후 불변이다. 실제 Core artifact는 `game_core-705cc9514d94a1fe`, SHA `47379be3…`, features=[],opt0/debug2/test=true다.

[functional009](product_parts/functional009/RESULT_001.json)은 그 새 바이너리의 문제 HQ selector만 `--exact`로 실행해1통과/0실패/0ignored/2058filtered,0.560초/exit0/reaped=true다. 메인이261바이트 stdout과 RESULT를 읽었고 source SHA가 compile011과 같음을 확인했다. 앞선 실패 목표인 상점 진입·실제구매·종료가 이제 도달했다. 이번 delta는 해당 시험과 direct probe 외부 loader 및 build selector 준비이므로 functional007의 다른43개와 functional008의 server3를 관련 비영향 근거로 유지한다. 서로 다른 invocation을 단일44개 동시 통과로 재기록하지 않는다.


## 첫 공개 release 경로의 실제 전제 확인

[release_compile001](product_parts/release_compile001/RESULT_001.json)은 새 cold target에서636.631초/exit0/reaped=true, source618 SHA `f64f54f0e581212051a4c0b3c733f6aa4983f7940f5a834e4be6284055cd3d28` 전후 불변이다. 메인이 prepared proof(default Core features, release, 동일4package graph, 정확3bin)와 RESULT/실제 artifact 목록을 읽었다. public probe SHA `fbb5b92f…`, normal host `d879a4f3…`, 공식 packager `2fd78f0a…`다. 최초 dependency/Core/server 컴파일 동안 동일 collector의60초 CPU/RSS 진단만 기록했고, 초반 stderr0에서 graph 단계라고 추정한 보고는 실제 proof가 시작1초 뒤 생성된 사실로 교정했다. 이 빌드 시간/RSS는 게임 실행 비용 판정으로 사용하지 않는다.

[official_packaging001](product_parts/official_packaging001/RESULT_001.json)은 실제 지원 CLI로 현재 game_resources를 새 official pack에 구성해1.123초/exit0/reaped=true다. 결과27files/433123bytes와 현재 catalog6/execution3 content identity `e504fdd1…`를 보존했다. 초기 tactical20/base cost10과 적 능력치를 바꾸지 않았고, selection은 official 하나/winners없음, runtime은 기존1605bytes SHA `a1bb554d…` 그대로다. 별도 lab target/funding160/cost calibration의 입력과 구분한다.

[public_hq001](product_parts/public_hq001/RESULT_001.json)은 동일 release/source의 실제 첫 공개 Expedition HQ 경로다.26.727초/exit1/reaped=true, source 전후 불변이며 start→VN advance→6후보 중 Dealer/Tank/Healer3명 선택→ViewingMap의3명령/0tick을 실제 수행했다. 첫 fresh application Continue에서 `Continue changed saved checkpoint bytes`로 실패해 HQ에 아직 도달하지 않았다. result passed=false와 trace4records를 보존했으며, 이 결과로 HQ 준비·전투·calibration·normal admission을 통과했다고 하지 않는다.

메인은 캐시 export·canonical encode·restore/reconcile 경로를 읽었다. 직렬화 순서 차이를 가정하거나 oracle를 약화하지 않는다. 실제 최종 Continue의 uuid_manager에 VN counter `6220401612985551955:0`이 있고, `restore_checkpoint_preserving_counters`가 fresh manager에 없던 namespace를0으로 수집한 뒤 `entry(...).or_insert(0)`으로 넣는 경로가 있다. 최초 출발은 UUID manager를 instance별로 초기화한다. 이는 유력 원인이지만 아직 before/after payload쌍으로 확정하지 않았다. runner0에서 독립 author의 Trace.restart1파일에 실제 checkpoint쌍·loads/peak를 assertion 전에 보존하는 좁은 진단 작성만 열었다. 기존 실패 oracle는 유지하고 다음 실행도 같은 HQ1경로로 제한한다.


checkpoint 진단17줄만 포함한 [release_compile002](product_parts/release_compile002/RESULT_001.json)는 public probe만 선택한 동일 default release graph에서300.743초/exit0/reaped=true, source618 SHA `5533feafbf08e6f4fb1eaf0cb4f0d2a82f7451af1ff79073758e1be5b941bc96` 전후 불변이다. strict 새 prepared proof 경로로 Core 의존 crate의 재컴파일이 실제 발생했으며 임의 캐시 우회는 없었다. 두 진단의 UTC clock이약36,807초 불연속 이동했지만 monotonic/CPU는약60초 증가했다. [CLOCK_OBSERVATION](product_parts/release_compile002/CLOCK_OBSERVATION_001.json)을 보존했고 원인이나 wall gap의 작업 시간을 추정하지 않는다.

[public_hq002](product_parts/public_hq002/RESULT_001.json)는 같은 공식 입력에서60.223초/exit1/reaped=true로 동일 실패를 재현했다. 메인이 실제 trace의 before/after raw bytes를 decode하고 parsed JSON을 직접 비교했다.17757→17781bytes 차이는 `/continuation/current/uuid_manager/counters/6220401612985551955`의 absent→0과 그로 인한 continuation checksum뿐이며 loads0/peak332는 같다. [원시 쌍](product_parts/public_hq002/result.trace.jsonl)을 보존한다. 이는 기존 canonical restore 계약에 불필요한 상태가 추가되는 제품 결함이며 comparator를 완화하지 않았다.

[correction007](product_parts/correction007/RESULT_001.json)은 기존 baseline 범위의 uuid_manager.rs 원본을 보존한 뒤 Sol이 정확1제품파일을 교정했다. 복구할 현재 counter가 저장 counter(없으면0)보다 클 때만 insert하여 없는0stream을 만들지 않으면서 기존 explicit0·positive high-water·선택 외 local rewind를 유지한다. 메인이 실제 diff와 단일 호출 경계를 읽고 이 코드 변경을 수용했다. cfg(test) 영역은 그대로이며 기능 재검증은 아직 아니다. 독립 Astra는 canonical 복원 회귀, G07 회복7범주, G09 실제 story ack와 stale checkpoint, 현재 workload successor를 한 작성 묶음으로 보강한다. 모든 writer0 뒤 기존 monotonic UUID 및 U08 canonical-set 회귀와 필요한 새 시험을 함께 검증한다.

G07 작성 중 독립 author가 정상 Gun+active fragment에서 공개 `UnEquipItem`을 거친 weaponless 상태를 Continue가 거절함을 발견했다. 메인이 실제 maintenance/helper 및 durable validator를 대조했다. 공개 해제는 active binding을 유지하지만 복원 검증은 무기 부재의 `WeaponRequired` 때문에 실패하여, 합의된 조각 해제→보유 무기 장착 회복의 중간 상태가 저장되지 않는다. [correction008](product_parts/correction008/RESULT_001.json)은 node_flow.rs의 정확1hunk로, 실제 weaponless employee/profile이며 실패가 `WeaponRequired`와 선택적 `BlockCapacityTooLow`뿐인 유효 form을 durable intermediate로 인정한다. block 최소값도 무기 부재의 base0에서 파생됨을 확인했다. binding/copy/form 존재, nonweapon compatibility, 무기가 있는 불일치는 계속 검증하고 공개 장착/조각/배치 경로는 변경하지 않았다. 메인은 diff를 검토하여 의미를 수용했고, 기능 판정은 독립 공개 명령 양성 및 기존 checksum-valid 음성 실행 뒤에 한다. 제품/runner는0이고 author 작성만 진행 중이다.

정상 official 승인 경로는 기존 G11 비용 후보와 분리된 실제 증거가 필요하다. Sol 조사와 메인 소스 검토에서 criterion은 suite/profile/runtime/실제 host·worker context에 결속되고, 다른 candidate에 적용하더라도 그 candidate 자체의 새 paired 실행을 요구함을 확인했다. 원형 비용 suite의 `u11_parallel_patrol` 참조와8명 tick 전 배치 비용은 수정하지 않은 official의 encounter·초기 tactical20 전제로 충족되지 않는다. 따라서16/500/1000명·1/8/8배치·60×1000ms G11 workload와 원형 비용 증거는 유지하고, 원본 official의 정상 승인/실제 진행에는 별도 기능 suite/context의 실제 calibration·기준 검토·admission을 준비한다. 별도 기능 suite를 full60 비용 증거로 대체하거나 측정용 콘텐츠를 원본 official 진행으로 표시하지 않는다. 현재 둘 다 새 실제 승인 증거는 아직0이다.

독립 author의8파일 묶음은 G07 회복10개, codec 신규2개와 기존 G09의 실제 Ack 검증, 과거 수치 회귀의 명시적 출발 후 Sword fixture, 현재 workload 준비기를 포함한다. 원본·diff·현재 SHA는 [author_batch001](test_parts/author_batch001/)에 보존했다. 준비기는 현재 official에서6적×7스칼라만 측정용으로 바꾸고 별도 pack에 현재 정책의 tactical initial/maximum160을 둔다. 옛 geometry/encounter만 그대로 재사용하며 실행·승인 증거를 만들지 않는다.

[compile012](product_parts/compile012/RESULT_001.json)는 Core-only no-run31.488초/exit101/reaped=true, source619 SHA `1dc75635d086a71c272e26b86212c734c22f669300d2292deec3da0bf1e9e605` 전후 동일이다. 새 Python 준비기는 기존 collector의 Core 입력 범위 밖이므로 보조 SHA 전후 확인으로 별도 결속했다. 메인이 실제 compiler diagnostics를 읽었고 codec.rs의 지원되지 않는 `BehaviorResult.payload_value()` 네 호출만 E0599로 보고됐다. 기능 시험은0이다. 독립 후속 Astra author는 실제 Serialize API를 확인해 네 호출을 전체 enum/field의 `serde_json::to_value` 비교로 교정했고 메인이 실제 diff를 수용했다. canonical bytes·roots·후속 채용 UUID 비교는 그대로다. 이전8파일 저자 이력과 교정 원본을 유지하며 제품 코드나 oracle는 이 컴파일 오류 때문에 변경하지 않았다.

[compile013](product_parts/compile013/RESULT_001.json)은74.568초/exit0/reaped=true, source619 SHA `9187b8e5d63004eee6ce8707ad71af61acdf56329ffa8d6c2efdc55b7bb8e33d` 전후 동일이며 compiler error0/build-finished true다. 메인이 실제 RESULT와 Core test artifact SHA `6bc70274…`, features=[]/opt0/debug2/test=true를 읽었다. codec API1파일만 compile012와 달라졌고 독립 교정 원본은 [codec_api_correction001](test_parts/codec_api_correction001/)에 보존했다.

같은 바이너리의 [functional010](product_parts/functional010/RESULT_001.json)은259.736초/exit101/reaped=true, 실제98unique 중68통과/30실패/0ignored/1973filtered다. 원시 --list2071개에서 승인한 prefix/exact 집합을 그대로 선택했고, 초기105 예상은 직원6개가 과거 별도 filter였다는 점과 U08 실제14개를 반영해98로 정정했다. 숫자를 채우려고 관련 없는 시험을 추가하지 않았다. source619와 artifact 및 보조 Python SHA는 전후 같다. 기존 슬롯을 가정한 공유 fixture14건, consumable tier3건, U08/query/trace 등의 오래된 기대값과 신규 G07 경계 실패를 분류 중이며 전체 통과로 기록하지 않는다.

메인이 raw 실패 위치와 [실제 구조 차이](product_parts/functional010/ROOT_DIFFERENCES_001.json)를 읽었다. 신규 codec 및 G07의5건은 저장 byte/profile 변경이 아니라 source에 남은6명 `starter_candidates`와 fresh Continue의 빈 후보 차이 하나다. codec의 앞선 canonical bytes/loads/peak 비교는 실제 통과했지만 전체 case는 아직 실패다. [correction009](product_parts/correction009/RESULT_001.json)는 성공 선발 뒤 단계 전용 후보를 지우는 world.rs1줄과 회복 제한 중 `CompleteCombatResult`를 허용하는 capability1줄이다. 후자는 실제 settlement가 회복 제한을 정한 뒤 CombatResult receipt를 만들지만 기존 allowlist가 확인 명령을 막던 제품 결함이며, 기존 phase/receipt/battle UUID 검사는 유지한다. 메인은 두 diff와 호출 경계를 검토해 수용했으며 아직 재검증하지 않았다. G07의 Elite difficulty140%를 floor anchor100%로 만든 지도는 시험 fixture 오류이므로 제품 validator를 바꾸지 않는다.


독립 원 author의 [실패 fixture 교정](test_parts/functional010_fixture_correction001/)은 실제6파일 원본·diff·SHA를 보존했다. 메인은 전 diff와 관련 제품 계약을 읽고 수용했다. 공유 category helper는 실제 authored schedule/pool/template을 찾아 지도 입력을 구성하며 mode/profile/depth를 유지하고, 기존 event/shop 이력을 지우는 대신 교체 전 부재를 검증한다. Sword fixture의 직원 UUID 배분 순서를 고정하고, G07 consumable tier와 Elite의 현재 policy target%, U04의 실제 Tutorial 시작을 맞췄다. VN query 목록은 현재 세 항목과 정확히 비교한다. U08의 옛8MiB 상수는 현재 정식 embedded16MiB/default 일치·측정 override 부재와 비교하되 one-short 입력 및 원자성 oracle를 유지한다. smoke는 초기 BattleStart/UnitSpawned를 실제 bucket에서 보존하고 모든 sequence의 연속성·BattleEnd 및 현재 count/byte/span 세 한계로 정해지는 정확 retained suffix를 검증한다. 제품 log 보유 범위나 core_roots/canonical 비교를 완화하지 않았다.

writer0에서 다음 선택은 functional010의 실제 실패30개와 선발중 후보 유지·잘못된 선발 원자성·확인 receipt/phase 음성7개를 포함한 추가3개, 총33개 exact 집합이다. [선택 목록](test_parts/functional010_fixture_correction001/SELECTORS.json)을 보존했다. 기존68개를 역할 재개만으로 자동 반복하지 않는다. 지정 author/runner 역할은 유지하며, 도구의 동시 thread 제한으로 같은 author가 턴을 종료한 후 기존 Sol runner를 재개하는 직렬 운용을 한다. 이 교정은 아직 컴파일/실행 PASS가 아니다.


[compile014](product_parts/compile014/RESULT_001.json)은75.458초/exit0/reaped=true, source619 SHA `9846ddac5b06de2d6e2153968eb7e2935486f8e7bd3364bd8631682882c5795b` 전후 동일이다. 메인이 RESULT·실제 artifact·후속 선택 proof를 읽었고 Core test SHA `2aeaa0ca…`, features=[]/opt0/debug2/test=true를 확인했다. 이전 입력 대비 변경은 승인된 제품2파일과 fixture6파일뿐이다. 후속 functional011은 같은 바이너리의 승인33개를 exact로 실행하며, 공개 release/HQ/normal 승인과 비용 측정은 이 결과에 포함하지 않는다.


[functional011](product_parts/functional011/RESULT_001.json)은187.018초/exit101/reaped=true, 실제33unique 중27통과/6실패/0ignored/2038filtered다. 실패30 재선택 중24통과/6실패이고 추가3은 모두 통과했다. source619·Core artifact·보조 Python SHA 전후 동일이다. 메인은 RESULT와 실제6실패 원문을 읽었다. canonical Continue/다음 채용 UUID, 무기 없는 active fragment의 합법 복원·부적합 음성, bounded log, 추가 선발/receipt 음성이 통과했지만 남은6을 완료로 표시하지 않는다.

남은 실패 중 G07 세 건은 명시 지도 fixture의 start_node_ids가 Start 장식 노드인 entry를 가리킨다. 실제 시설 계약의 navigation_start_slot_ids는 이웃 dispatch_hall이므로 fixture를 교정한다. U08 UUID 소유권 사례는 unfinished pre-entry 복원에 현재 profile을 설치하지 않은 입력을 사용했다. Gate/HQ 마지막 동등 비교는 공개 채용 뒤 본부 활동 CompleteNode 이전이라 현재 미완료 노드 rollback 계약상 진입 전 ViewingMap/wallet100/직원3으로 복원된다. [구조 차이](product_parts/functional011/ROOT_DIFFERENCES_001.json)와 continuation의 stable 단계 목록을 읽었으며 저장 정책을 바꾸지 않고 완료 뒤 동등 복원을 검증한다.

U04 Tutorial terminal ChooseOne은 제품의 terminal completion이 아직 남은 보상 세션을 붙이기 전에 roster/inventory를 정리하여 recruitment queue와 roster 소유권이 불일치하는 경계다. 메인이 node_flow/combat/reward 실제 호출 순서를 확인했고, Sol이 최종 보상 종료까지 정리를 미루는 최소 변경 경계를 검토 중이다. durable validator나 전체 roots oracle는 완화하지 않는다.


[correction010](product_parts/correction010/RESULT_001.json)은 제품 node_flow/combat/reward 세 파일만 수정했다. 메인이 모든 diff와 실제 호출 경계를 읽었다. live combat 전용 completion wrapper는 RunComplete 정리를 지연하고 기존 shared/sweep/비전투 완료는 즉시 정리를 유지한다. 후속 보상이 없는 RunComplete ACK 및 combat reward의 RunComplete Exit에서만 terminal 정리를 수행한다. Claim 중에는 직원·장비·보상 권리를 보존하고 기존 RunFailed·Expedition Return 정책과 validator는 유지한다. cfg(test)/시험 코드 변경·빌드·실행은0이며 기능 판정은 다음 독립 사례에 결속한다.

같은 작성 창에서 `equipment-and-fragments.md`가 외부에서16041→16947바이트로 바뀌었다. 메인과 두 작성자는 이 변경을 하지 않았다. functional011과 정확 SHA가 같은 보존 원본을 읽어 [현재 바이트와 diff](product_parts/correction010/external_document_review001/)를 별도로 보존했다. 흑조 패링 성공 뒤500ms 면역의 목표 규칙 추가이며 문서가 구현·검증 미완료라고 명시한다. U12 lifecycle 수용기준/제품 행위와의 충돌은 없고 현재 내용을 유지한다. 전체619 입력 불변이라고 주장하지 않으며 다음 실행에 현재 문서를 결속한다.


독립 author의 [functional011 fixture 교정·terminal 보강](test_parts/functional011_fixture_correction001/)은 세 파일의 원본·최종 diff·SHA·exact 선택 목록을 보존했다. 메인은 전 diff와 source 계약을 읽었다. G07 지도는 실제 template의 navigation start를 도출하고 전체 graph/progression 유효성을 먼저 검증한다. 두 HQ 사례는 미완료 시 실제 pre-entry wallet/roster/inventory rollback을 검증한 뒤 CompleteNode 후 전체 동등 복원을 검증한다. U08 unfinished 복원은 현재 profile을 설치한다.

Terminal ChooseOne은 비어 있지 않은 유효한 영구 profile을 출발 전에 설치하고 ACK/Claim 중 직원·장비 copy binding·채용 소유권 및 실제 Continue를 보존하며 Exit 뒤 정확 정리/profile 불변/재요청 원자성을 검증한다. 새 zero-candidate terminal은 현재 계약대로 ACK 전부터 export None이며 runtime actor는 ACK까지 유지하고, 명시 CombatRewardsGranted→RunComplete 결과 뒤 정리·반복 거절을 검증한다. 이 두 계약은 메인 source 검토에서 author 초안의 불필요한 no-choice Continue 기대와 직접 RunComplete 반환 기대를 정정했으며 실행 실패로 소급 기록하지 않았다. 다음 선택은 실제 실패6+새 terminal1+비종료 ChooseOne/empty 및 shared sweep3, 총10개이며 기존33/98 전체 반복은 하지 않는다.


[compile015](product_parts/compile015/RESULT_001.json)은71.461초/exit0/reaped=true, source619 SHA `2f4e64ed7122dd0fc0af862e84a534b2925c61799f37ee1e3d76edc4452a94eb` 전후 동일이다. 메인이 실제 RESULT와 Core artifact SHA `81e6cb29…`, features=[]/opt0/debug2/test=true 및 후속 exact10 proof를 읽었다. [functional012](product_parts/functional012/RESULT_001.json)은25.777초/exit101/reaped=true,10실제 중8통과/2실패/0ignored/2062filtered다. source/artifact/보조Python SHA 전후 동일이며 새 no-choice 포함 등록수2072다. Terminal ChooseOne과 무보상 ACK·최종 정리/profile 보존, 비종료 보상2, shared sweep 및 U08/나머지 회복2가 실제 통과했다.

남은 두 HQ 사례는 main이 앞서 지정한 CompleteNode가 기존 본부에서 ContextIrrelevant라 거절됐다. 메인은 capability와 headquarters의 실제 완료 owner를 읽었다. 본부는 RequestEmergencySupplies 또는 실제 shop 종료로 완료하며 일반 CompleteNode를 허용하지 않는다. 제품 계약을 바꾸지 않고 두 fixture가 실제 보급 완료를 소비하도록 교정한다. 채용 직후 wallet0 검증은 유지하고 이후 보급 증가분은 현재 policy의 정확 값과 별도로 비교한다. 다음 재실행은 실패2개만이며 이번8개 통과를 하나의10통과 invocation으로 바꾸지 않는다.


[독립 HQ 종료 교정](test_parts/functional012_fixture_correction001/)은 recovery.rs 한 파일의 두 실패 사례만 수정했다. 메인이 actual diff를 읽고 현재 SHA를 확인했다. 실제 RequestEmergencySupplies 결과의 지급후 총액·NodeCompleted/ViewingMap을 검증하며 채용 직후0, 미완료 rollback, 채용 UUID/definition/deployable, 이전 SavePoint·profile 보존과 마지막 전체 동등 복원은 유지한다. 제품/정책/수치 변경0이며 선택은 실패2 exact다.


[compile016](product_parts/compile016/RESULT_001.json)은62.454초/exit0/reaped=true, source619 SHA `c6bfd6d515ccbc1e61133f8f0c3827f020d40a2e7e16f197d151bc907b6ee7ed` 전후 동일이다. 메인이 실제 RESULT와 Core test artifact SHA `e30074cf…`, features=[]/opt0/debug2/test=true를 읽었다. [functional013](product_parts/functional013/RESULT_001.json)은8.489초/exit0/reaped=true, 선택2개 모두 통과/0실패/0ignored/2070filtered다. 메인은 전체 stdout과 RESULT를 읽었고 source/artifact/보조 Python SHA가 동일함을 확인했다. 두 HQ 채용→미완료 rollback→정식 보급 완료→동일 Continue가 이제 도달했다. 이전8PASS는 별도 invocation의 실제 근거로 유지한다. 이 좁은 실패 교정 묶음은 닫혔고, 새 current default-feature release 공개 HQ 경로를 재개한다. 공개·정상 admission·비용·전체 목표 완료는 아직 별도 남아 있다.


[release_compile003](product_parts/release_compile003/RESULT_001.json)은299.523초/exit0/reaped=true, 현재 source619 `c6bfd6d5…` 전후 동일이다. 메인이 실제 지원 build argv/proof·Core build output·artifact를 읽었다. public_lifecycle SHA `c77d893c…`는 default Core features, release, 기존 정확4package dependency graph에 결속된다. [public_hq003](product_parts/public_hq003/RESULT_001.json)은99.423초/exit0/reaped=true, 실제6명령/0tick/trace9개로 빈 출발→VN→Dealer/Tank/Healer3명→fresh Continue→HQ 진입→보급 완료→ViewingMap에 도달했다. 메인은 trace 명령·단계와 [checkpoint 원시 쌍](product_parts/public_hq003/result.trace.jsonl)을 읽고17757바이트 SHA `5d92799d…` 전후 동일 및 loads0/peak332 불변을 직접 확인했다. 공식27파일/selection/runtime과 현재 source·binary·보조Python은 전후 같다. 이는 첫 공개 HQ 경계 완료이며 전체 lifecycle·normal approval 증거로 확대하지 않는다.

다음 실행은 같은 공개 바이너리의 Expedition save 경계로 실제 전투·결과 확인·SavePoint/fresh Continue를 확인한다. 많은 tick의 진단 출력은 새 /tmp 디렉터리에 쓰고 완료 또는 실패 후 원문을 durable evidence에 그대로 복사·SHA 결속한다. 현재 writer의 작은 직접 파일 쓰기와 HQ에서 관측한99초/CPU약5초/stateD 때문에 출력 위치를 조정했으며 I/O 원인을 확정하거나 게임 비용으로 해석하지 않는다. 기능 oracle,100ms cadence,3000tick/512decision 한계, 공식 콘텐츠와 수치는 그대로다.


[public_save001](product_parts/public_save001/RESULT_001.json)은8.787초/exit1/reaped=true,25명령·3000회100ms tick 뒤 진단 한계로 실패했다. 실제 전투 시각300000ms이며 SavePoint에는 도달하지 않았다. 새 /tmp의 result480032바이트와 trace45063515바이트는 [양 경로 provenance](product_parts/public_save001/OUTPUT_PROVENANCE_001.json)와 함께 durable 경로에 byte 동일 복사해 보존했다. source619·release binary·공식 입력은 전후 불변이다. 이 실행을 게임 비용이나 전체 전투 성공으로 해석하지 않는다.

메인은 실제 battle setup, 배치 명령 및 마지막 checkpoint를 읽었다. route end는(4,6), Tank는(3,6)/up, Dealer는(2,6)/right였고 공개 기본공격 범위가 실제 적의(4,6)를 덮지 않았다. 두 적과 아군은 마지막 시점에도 HP 손실 없이 남았다. 진행기가 penultimate(4,5)를 목표로 삼고 가능한 사거리 배치가 없으면 첫 cell/facing으로 fallback하며 첫 Tank는 preview를 건너뛰던 문제다. 제품/적 능력치/시간 한도를 바꾸지 않고 독립 author가 공개 사거리 preview로 실제 route endpoint/경로 및 아군 지원을 덮는 유효 배치를 선택하도록 교정한다.


독립 author의 [public save 배치 교정](test_parts/public_save001_placement_correction001/)은 probe lib.rs 한 파일만 변경했고 원본·최종 diff·SHA를 보존했다. 메인은 전체 diff를 읽고 현재 SHA `c9d3dc42…`를 확인했다. 첫 Tank도 공개 eligibility/available/기본공격 범위를 확인하며 실제 route endpoint 우선으로 선택하고 Dealer가 같은 공격점을 덮게 한다. Healer는 checkpoint의 실제 Tank 위치를 반드시 포함하며 아군 coverage를 비교한다. route 없는 수동 전투는 실제 살아 있는 적 위치를 사용하고 미spawn은 기존 유한 tick 안에서 기다린다. 유효 범위를 찾지 못하면 상세 오류를 남기며 임의 좌표 fallback은 제거했다. 자동 배치·3명·100ms/3000tick·512decision·공식 콘텐츠와 정책·strict Continue는 유지했다. 이 교정은 아직 빌드/실행 검증 전이며 다음 범위는 current public release와 같은 save 경계1개다.


[release_compile004](product_parts/release_compile004/RESULT_001.json)은301.697초/exit0/reaped=true, source619 SHA `15dd09cb…` 전후 동일이다. 메인이 실제 RESULT·artifact·지원 build/Core proof 결속을 읽었다. default-feature release public binary SHA `9aa644ae…`와 동일 Core/4package graph를 확인했다. 제품 변경 없이 공개 배치 진행기만 교정됐으며 다음은 같은 official/selection/runtime의 save 경계1개다. DirectHost Continue는 동일 프로세스의 새 CoreApplication에서 profile/checkpoint를 복원하는 검증이며, 별도 프로세스 파일 복원과 승인 host 검증은 후속 근거가 필요하다.


[public_save002](product_parts/public_save002/RESULT_001.json)은1.371초/exit0/reaped=true, 실제24명령·204회100ms tick으로 공식 첫 전투 승리→결과 ACK→SavePoint→새 CoreApplication Continue를 완료했다. 메인은 raw 배치3건과 실제 명령·결과를 읽었다. Tank(3,6)/right·Dealer(5,6)/left가(4,6)를 덮고 Healer(3,7)/up이 실제 Tank와 Dealer를 덮는다. checkpoint 원문 두 쌍을 직접 decode/비교해17757/24375바이트, SHA `5d92799d…`/`e1f2a94b…` 전후 동일, parsed JSON 동일·loads0/peak332 불변을 확인했다. 원래 /tmp trace5023220바이트 SHA `be84e6a2…`와 result451832바이트는 durable 경로에 byte 동일 보존했다. full_lifecycle_complete=false 및 approval=false 범위를 유지하며 다음은 같은 입력의 원정 complete 경계다.


[public_expedition_complete001](product_parts/public_expedition_complete001/RESULT_001.json)은3.640초/exit0/reaped=true, source619 `15dd09cb…` 전후 동일이다. 메인이 raw83명령·612tick·720records의 실제3전투/3HQ/3SavePoint와 terminal을 읽었다. 깊이3의 reward 진입에서 deterrence0/Exhaustion 선택을 거쳐 Return했으며 Gate 귀환으로 바꾸어 기록하지 않는다. 귀환 전 보유6장비/3파편의 정확 UUID·정의·상태가 vault로 이관됐고 active marker=None, sequence1, first_return_story=Pending, highest_floor3, Continue=None다. 여섯 checkpoint 쌍의 원문·parsed JSON 동일 및 loads0/peak332를 메인이 직접 비교했다. trace16206757바이트 SHA `4002ef23…`와 result106363바이트는 원본과 byte 동일 보존했다. 이 direct 원정의 terminal 경로는 통과했지만 이야기 ACK·다음 출발·정상 승인·별도 프로세스 저장은 아직 이 결과의 범위 밖이다. 다음은 동일 binary/input의 Tutorial complete이며 이번 Tutorial은 초기 빈 profile의 불변 검증이다.


[public_tutorial_complete001](product_parts/public_tutorial_complete001/RESULT_001.json)은1.621초/exit1/reaped=true, source619 `15dd09cb…` 전후 동일이며 첫 전투 이후 둘째 normal_breach_response에서 진행기 배치 검색 실패다. 메인은 실제 둘째 surrounded_cross_pressure_01의 setup과 Tank/Dealer 배치 및 세 Healer preview를 읽었다. endpoint(4,5)를 덮는 Tank(4,6)/up을 먼저 선택했지만 Healer의 합법 platform은 y2이고 반환된 범위는 y4까지만 덮어 이 Tank를 지원할 수 없다. 세 역할을 함께 고려하기 전에 첫 배치를 확정한 진행기 오류이며 콘텐츠/HP/정책/시간을 바꾸지 않는다. 원문 result564591바이트와 trace6498351바이트는 /tmp 원본과 byte 동일 보존했다.30명령/312tick/2전투 진입이며3층 완주를 통과로 표시하지 않는다. 독립 author가 최초 배치 전에 세 역할의 실제 공개 범위·별도 위치를 함께 만족하는 배치를 준비하도록 교정한다.


독립 author의 [joint 배치·정상 승인 입력 준비기](test_parts/tutorial_joint_placement_normal_preparation001/) 두 파일 원본·diff·SHA를 보존했다. 메인은 두 파일 전체를 읽었다. 배치는 실제 공개 preview의 세 역할 distinct 위치·공통 공격점·Tank 치료를 함께 요구하고, 두 공격자를 함께 치료하는 조합을 우선한다. 실제 배치 직전 비용·eligibility·점유·범위를 재확인하며 재원 회복은 기존 tick으로 기다린다. 신규 준비기는 실제 성공 HQ의6명령을 별도 Normal의 measured steps로 만들고 LabPreparation=None·preparation_steps=[] 및 run policy/선발3정의의 실제 소비 의무를 명시한다. source receipt의 공식 파일과 runtime bytes를 정확 대조하며 입력 diagnostic profile bytes도 그대로 보존한다. 신규 경로의 사전 부재는 supplemental baseline002에 결속했다. G11 collect/prepare_workload와 모든 정책·수치는 변경하지 않았다. 빌드·준비기 실행·새 Tutorial 검증·정상 승인은 아직 미실행이다.


[release_compile005](product_parts/release_compile005/RESULT_001.json)은309.863초/exit0/reaped=true, source619 SHA `c1bd1dee…`와 보조 Python 두 파일이 전후 동일이다. 메인이 실제 RESULT·지원 build·artifact를 읽고 default plain/동일 Core build·graph에 결속된 public SHA `ec29553c…`, normal_host SHA `f51afe35…`를 확인했다. 두 실행 파일 빌드만 완료한 상태이며 normal admission이나 calibration으로 해석하지 않는다. 다음 실행은 교정한 joint 진행기로 Tutorial complete 한 건이다.


[public_tutorial_complete002](product_parts/public_tutorial_complete002/RESULT_001.json)은9.793초/exit1/reaped=true, source619 `c1bd1dee…`와 binary·공식 입력·보조Python이 전후 동일이다.73명령·3212tick·3298records에서 첫 전투 승리 후 둘째 전투가 기존3000tick 한도에 도달했다. 원문 result566254바이트와 trace47107956바이트 SHA `6ca7f0fd…`는 /tmp와 durable 양쪽에 byte 동일 보존했다. 초기 checkpoint18154바이트 SHA `8ffa89b5…`의 원문·parsed JSON 동일 및 loads0/peak311을 메인이 직접 확인했다.

메인은 실제 joint plan과 둘째 전투의 모든 endpoint 지원 preview를 읽었다. 현재 진행기는 두 공격자를 함께 치료하는(4,4) 접점을 우선했지만 적은 실제 endpoint(4,5)에 살아 남아 둘 다 공격할 수 없었다. 실제 preview에는 Tank(4,4)/down·Dealer(4,6)/up이 endpoint를 덮고 Healer(4,2)가 Tank를 치료하는 조합이 존재한다. 세 역할의 공동 유효성 검증은 유지하되 endpoint/route 순서를 선택 점수의 첫 기준으로, 선택적 Dealer 치료를 다음 기준으로 교정한다. 좌표 특례·제품/콘텐츠/수치/시간 변경 없이 진행기의 우선순위만 수정하며, 이번 실패를 Tutorial 완주나 정상 승인 증거로 표시하지 않는다.


독립 author의 [endpoint 우선 교정](test_parts/tutorial_endpoint_priority001/)은 lib.rs의 점수 순서·tuple 해석·관련 주석만 바꿨다. 메인은 전체 diff와 원본/최종 SHA를 읽고 현재30,556바이트 SHA `cdaa1248…`를 확인했다. 모든 필수 공동 배치·eligibility·직전 범위 검증과 유한 한계는 그대로이며 제품 변경0이다. 다음은 같은 지원 plain build로 public/normal_host 두 실행 파일을 결속한 뒤 Tutorial complete 한 건을 재실행한다.


사용자의 연속 실행 재개 때 `release_compile006`의 기존 session25883 회수는 wrapper exit143 뒤 Unknown process였고, RESULT/SOURCE_AFTER가 없어 실제 Cargo 종료·reaped·성공은 미확인으로 보존한다. /tmp target/cache가 초기화됐지만 workspace 원문·source·preimage는 유지됐다. Sol은 동일 lock의343개 crate checksum 일치를 확보하고 새 [release_compile006_retry001](product_parts/release_compile006_retry001/)에서 cold build를 시작했다. 메인은 실제 새 prepared proof를 이전005와 대조해 compiler1.98.1/target/release/default features/6manifest SHA·환경/전체 및 selected tree·Core graph가 동일함을 확인했다. 이는 복구 입력의 검토이며 빌드 성공·기능 PASS가 아니다.

현재 두 probe가 소비하지 않고 기존619 source manifest에도 없는 신규 server-only 시험 모듈만 독립 Astra가 병행 작성한다. 기존 등록부·lib/bin/support와 현재 빌드 입력은 동결한다. 다음 검증 묶음은 실제 획득품의 대여·귀환·profile-only story ACK/CAS/crash, 같은 인스턴스 SavePoint와 과거 인스턴스 저장 거절, 실제 유료 HQ 채용·nonempty Tutorial이다. 일반 명령마다 메인 재승인을 요구하지 않고, 작성·실행 사실과 중요한 결과의 Astra 수용을 구별한다.


[release_compile006_retry001](product_parts/release_compile006_retry001/RESULT_001.json)은639.244초/exit0/reaped=true로 복구 빌드가 완료됐다. 메인은 실제 RESULT·artifact·지원 proof와 Core build output을 확인했다. compiler/target/default features/6manifest·전체 및 selected graph는005와 같고 source619 SHA `a0f18121…`는 전후 동일이다. public SHA `2721d1af…`, normal_host SHA `4899f94b…` 실제 실행 파일을 durable artifacts에도 보존했다. 중단된006은 성공으로 덮어쓰지 않는다.

[public_tutorial_complete003](product_parts/public_tutorial_complete003/RESULT_001.json)은12.447초/exit1/reaped=true,173명령·3560tick·3760records다. endpoint 우선 교정으로 첫3전투는 실제 승리·결과 ACK를 통과했고, 넷째 floor1 suppress_porcubbus에서 기존3000tick 한도에 도달했다. result851604바이트 SHA `4ed77de4…`/trace62485386바이트 SHA `0e7791d0…`를 원래 /tmp와 byte 동일 보존했고 source·binary·공식 입력은 불변이다. 메인은 실제 raw setup·joint plan·마지막 checkpoint와 RESULT/OBS를 읽었다. 포큐버스는(2,5),620/620HP에 남고 endpoint(4,6)를 향한 Tank(3,6)/right·Dealer(5,6)/left의 공개 사거리 밖이며 아군3명은 모두180HP다. 기존 진행기는 returned_route 목표를 최초 선택한 뒤 live 상대 위치에 따른 전술 조정을 하지 않는다. route 미설정이나 제품 이동 결함으로 아직 확정하지 않으며 실제 AI/이동 owner를 검토한다. Tutorial 완주 및 이에 의존하는 회귀는 미통과로 유지한다.

독립 Astra의 새 [Return cycle 모듈](test_parts/runtime001/shared/normal_return_cycle_tests.rs)은 메인이 전체 원문·[author AFTER](test_parts/normal_return_cycle_author001/AFTER.json)를 읽었다. 실제 public 원정의 귀환 전/후 저장을 seed로 정확 copy 대여·다음 instance·유료HQ·SavePoint 뒤 story ACK와 load·재귀환을 검증하고,22개 실제 child crash·두 번 fresh recovery·profile-only ACK CAS/no-change 및 nonempty Tutorial 두 경로를 준비했다. 기존 등록부/lib/support 변경0이며 아직 미등록·타입검사/실행 전이다. loan 재출발 checkpoint 결정성 전제의 코드 근거는 author가 추가 확인한다.


[normal_prepare001](product_parts/normal_prepare001/RESULT_001.json)은0.569초/exit0/reaped=true, source619 `a0f18121…` 불변이다. 메인은 생성된 scenario/suite/registry/PREPARATION과 RESULT를 실제 읽고 suite의 embedded JSON이 별도 scenario와 동일함을 확인했다. 공개 HQ003의 실제6명령을 snapshot3/assert4와 함께13개 measured steps로 구성했고 Normal1·LabPreparation=None·preparation_steps=[]·run policy와3선발 정의 소비 의무를 명시한다. 기존 공식27파일·HQ 원문·diagnostic profile13,633바이트 SHA `ce4204f9…`·정확 runtime bytes를 유지한다. 현재 입력 생성만 완료했으며 worker calibration/adoption/config·승인 실행·전체 전투/비용은 미실행이다.

Return cycle의 같은 sequence2 재실행은 지원 application의 동일seed/PLAYER, 실제profile, sequence기반 expedition UUID namespace 및 typed UUID stream 결정성에 근거한다. intent_temp_synced 이전에는 run/Continue가 없는 selection의 비accepted journal만 남으므로 fresh restore 뒤 Start/VN→selection 재실행이 맞다. 메인은 독립 author의 [정적 근거](test_parts/normal_return_cycle_author001/STATIC_REVIEW.json)를 읽었고 정확 checkpoint 비교 assertion을 유지한다. 런타임 통과로 표시하지 않는다.

Tutorial003의 이동 trace는 Porcubbus가5000ms side-ambush spawn 뒤6400ms에(2.651,5.038)에 도달하고 이후 움직이지 않음을 보인다. 실제 static obstacles와 공통 route PathAlongCells→직선 목표 이동→physics corrected translation 경계를 검토했다. 적의 경로 합류 중 장애물 접촉 관측으로 제한하며 route 누락·제품 결함으로 단정하지 않는다. 공개 TacticalRelocateUnit/PreviewTacticalCommand는 실제 geometry·eligibility·비용·cooldown을 제공한다. 독립 author는 살아 있는 상대에 대응하는 일반 공개 전술 진행을 보완하고 server-only module 등록을 같은 묶음으로 준비한다. 제품/콘텐츠/시간 한도 변경0을 유지한다.


독립 author의 [전술 진행기·Return 시험 등록](test_parts/tactical_driver_return_registration001/) 최종 delta를 메인이 읽고 실제3파일 byte/SHA를 대조했다. lib55,594바이트 SHA `9ca1dc87…`, 등록부40,715바이트 SHA `cfb40124…`, 등록된 Return 모듈47,263바이트 SHA `88eec1ec…`다. 초기 endpoint 공동 배치는 유지하며 실제 상대의 위치/HP 정지·사거리 밖 상태에서 공개 relocation/deployment preview로 일반 재계획한다. 이미 배치된 직원은 실제 deployed state와 tactical preview를 사용한다. 메인 검토에서 처음 발견한 “배치 완료 후 비는 deploy-cost 목록을 재배치에도 요구”하던 전제는 실행 전에 같은 묶음에서 교정했고 deployed eligibility fingerprint도 분리했다. 합법 점유·비용·cooldown·최신 authority·최종 공동 공격/치료 범위를 다시 확인하며 no-plan은 기존 유한 tick 안에서 이유를 남긴다. 보수적 중간 치료 유지·swap 미탐색은 진행기의 전략 한계로 구분한다.

같은 진행기는 현재 공개 readiness가 허용하고 실제 적 접촉이 있는 self/automatic 기본스킬과, 실제 손상 아군·공개 cast 범위·목표 preview가 허용하는 AllySingle 스킬만 사용한다. 스킬ID/효과 수치 특례는 없으며 수동 targeting focus는 발동/취소 후 같은 hosttick 전에 종료한다. Host/Trace/strict Continue 및 event/walk/정확 Return/terminal 부분은 이전과 byte 동일, 기존 등록부는 module2줄만 추가했다. 제품/공식 입력/HP/3000×100ms·512decision·실제3직원 조건은 불변이다. 아직 구문 검사만 완료했으며, Sol이 지원 server no-run→현재 두 공개 probe→Tutorial complete/영향 Expedition1회→별도 paired worker 준비를 수행한다. 신규 module은 기존619 목록 밖이므로 추가 source 결속을 필수로 한다.


[server_compile001](product_parts/server_compile001/RESULT_001.json)은266.704초/exit101/reaped=true이며 source619 SHA `f12fb5a2…`와 신규 모듈·두 Python 입력은 전후 같다. 메인은 실제 RESULT·stderr의3개 E0599와 지원 proof를 읽었다. plain/default Core features·graph `8f1ee6c6…`는 이전 공개 바이너리와 같고 서버 test의 dev dependency tree는 별도 원문 보존했다. 실패는 기존 normal_host_tests.rs의3개 `ValidatedGameData::load_live_embedded` 호출이 default release Core에 없는 API를 요구했기 때문이다. debug 환경에서 사용하던 fixture 연결 문제이며 제품/feature를 바꾸어 우회하지 않는다. 해당 시험파일만 독립 author에게 넘겨 현재 지원 공개 외부 로더와 승인 owner의 실제 generation 전달로 교정한다. artifact와 실행된 기능 시험은 없고 후속 공개 runtime은 보류한다.


독립 author의 [default release loader 교정](test_parts/default_release_actor_loader001/)을 메인이 전체 diff·AFTER·직접 SHA와 실제 승인 actor handler로 검토했다. normal_host_tests.rs42,265바이트 SHA `2e8b7517…`이며 승인된 세 actor 호출은 실제 admission.host.future_generation을 필수 message 입력으로 전달하고 실제 owner는 host.new_application을 유지한다. 비승인 profile-only fixture는 current official을 새 TempDir에 포장·공개 load하여 identity/receipt 일치를 확인하고 네 독립 Core에 clone한다. 실제 입력 receipt/runtime SHA 및 approval_claimed=false를 출력한다. 기존 oracle·제품·API·feature는 바꾸지 않았고 전술 lib `9ca1dc87…`와 Return 모듈 `88eec1ec…`는 byte 동일하다. 구문 검사만 완료했으며 아직 타입/기능 PASS가 아니다. root가 일시 capacity 오류였던 동일 Sol의 재개를 요청했고 메인은 중복 실행하지 않는다. 후속 G11의 실제 vault 규모별 비용·budget/release 누락은 독립 author가 읽기 설계만 수행한다.


[server_compile002](product_parts/server_compile002/RESULT_001.json)은312.937초/exit0/reaped=true, source619 SHA `f2d6d84d…`와 보조3입력이 전후 동일하다. 메인은 실제 RESULT/지원 proof/artifact·원문 종료와 실제 Core guard를 읽었다. release/default Core feature·graph `8f1ee6c6…` 및 Core build `9f635d2b…`를 유지하고 서버 test dev graph를 별도 결속했다. 새 Return cycle 등록까지 포함한 harness48,185,536바이트 SHA `669d4db7…`가 생성됐고 durable copy도 byte 동일이다. no-run 타입/빌드 성공이며 기능 실행0이다. Sol은 같은 동결 입력으로 현재 public/normal_host 두 probe 빌드와 합의된 공개 runtime 묶음을 계속한다.


[release_compile007](product_parts/release_compile007/RESULT_001.json)은294.322초/exit0/reaped=true이며 source619 `f2d6d84d…`/보조3입력 불변이다. 메인이 실제 RESULT/지원 command·Core guard/artifacts를 검토했다. public32,821,320바이트 `bbfd7f00…`/normal36,280,736바이트 `105369a1…`이며 동일 Core build/default graph에 결속된 durable byte 동일 실행파일을 보존했다. Sol의 일시 모델 capacity 오류 뒤 기존 session74898만 회수했고 중복 빌드는 없다. server001/002의 파생 guard.values 추출 오류는 원래 build output에서 교정했으며 실제 raw/RESULT/실행은 바꾸지 않았다.

[public_tutorial_complete004](product_parts/public_tutorial_complete004/RESULT_001.json)은25.879초/exit1/reaped=true,775명령·4437tick·5313records다. 앞3일반전투와 Porcubbus877tick은 실제 종료·결과 ACK를 통과했고 최종 suppress_nothing_there 전투에서 기존3000tick 한도에 도달했다. 실제 relocation2/skill activation4, checkpoint4쌍의 byte/parsed 일치·loads0/peak311은 원문에 결속했다. result844,179바이트 `ed1ad640…`와 trace120,949,981바이트 `b12ff25e…`를 byte 동일 보존했으며 공식 입력·runtime·binary·source는 전후 같다.

메인은 실제 마지막checkpoint·joint plans·공개 tactical previews와 적 제거 event를 읽었다. 보스 primary Nothing There는93851ms/seq959에 Tank가 HP2→0으로 실제 격파했고 defense object는31370ms/seq433에 죽었다. 최종 남은 적은(5,6)/(4,6)의 corroded_bruiser2개 각125/125HP다. Tank(4,9)/right·Dealer(6,9)/left·Healer(4,10)/up은180HP를 유지했다. command666의 Dealer(5,9)/up preview는 enabled=true이고(5,6) 사거리를 제공하지만 진행기가 Tank와 같은 접점을 필수로 요구하여 이를 버렸다. 실제 tactical destination도 deployment zone에 제한되므로 영역밖 이동으로 해결하지 않는다. 공동 접점은 시험 진행기의 전략이며 실제3직원·정상 전투·정확 terminal 요구와 구분한다. 독립 author는 일반 ranged fallback을 읽기 검토하고 시간/HP/콘텐츠/완주 판정은 유지한다. 전체 Tutorial은 미통과로 보존하고 후속 Expedition/worker 실행은 아직0이다.


Sol의 [bridge 문서 정렬](product_parts/bridge_contract_document_alignment001/)을 메인이 전체 diff·실제 command/result/snapshot·AttachSession 및 paired/profile-only publication 분기와 대조했다. game_server_core_bridge_goal.md18,534바이트 SHA `6755efa5…`는 오래된 직접 GameCore/BehaviorResult/replay 설명을 현재 CoreApplication·typed query/result·live projections·compact receipt ACK로 맞춘다. AttachSession은 vault/story/sequence와 published knowledge를 포함한 authoritative profile 전체를 정확 설치한다. 메인 검토에서 accepted intent의 precommit 설명이 profile-only 분기까지 포괄하던 문장만 paired active/profile persistence로 좁혔다. 코드·시험·실행 변경0이며 동시에 독립 작성 중인 lib delta는 문서 소유와 분리하고 다음 frozen source로 주장하지 않는다.


독립 author의 [일반 접점 fallback](test_parts/tactical_contact_fallback001/)은 lib70,912바이트 SHA `d9e58e92…`다. 메인은 전체 diff·AFTER·실제 Swap wire/preview와 후속 battle loop를 읽고 직접4파일SHA를 확인했다. 기존 공동 접점 우선, 한 공격자 접점과 Tank/단독 Dealer 치료 지원, 빈칸 계획이 없을 때만 유한3pair×16facing의 공개 Swap preview 순서다. 점유된 칸의 relocation range를 추측하지 않고 원자 Swap의 두 실제 projected range·최신 identity/authority·cost/eligibility·source poses를 다시 확인한다. 같은 상대 ID/위치/HP와 같은 최종 배치는 한 번만 시도하며 cooldown/시간만으로 진동을 재개하지 않는다. 계획/실행/최종 실제3명 검증에 동일 attacker mask를 사용한다. Host/Trace/strict Continue·skill helper·event/walk/Return/terminal과 시간/명령 한계는 byte/의미 그대로다. 아직 구문검사만 완료했으며 public 두 probe→Tutorial005→통과 시 영향 Expedition002→server harness 갱신을 Sol이 자율 연속 실행한다. G11 private-budget 추가가 Core 입력에 미치는 영향을 먼저 결정하기 위해 worker/calibration은 다음 동결 경계로 분리한다.


[release_compile008](product_parts/release_compile008/RESULT_001.json)은295.440초/exit0/reaped=true, source619 `23e063f4…`와 보조3입력 불변이다. 메인은 실제 RESULT/지원 command·Core guard/artifacts를 읽었다. public32,887,056바이트 `3ad454ed…`/normal36,344,720바이트 `cc5579f4…`는 동일 release/default Core build/graph와 durable byte 동일 복사에 결속됐다.

[public_tutorial_complete005](product_parts/public_tutorial_complete005/RESULT_001.json)은 **21.681초/exit0/reaped=true**다. 메인은 실제 result·terminal/profile과 원문 trace의 모든 전투 종료/ACK·마지막 적 사망을 확인했고 RESULT/OBS/원문 보존 결속을 읽었다. 공식3층[0,1,2]·5전투·HQ2·SavePoint1을 실제 진행했다. Porcubbus는87679ms, 최종 보스전은134117ms에 남은 일반 적까지 모두 격파했고 마지막 complete_combat_result에서 run_complete·Continue=None에 도달했다. 영구 profile은 empty vault/sequence0/storyNever/빈 knowledge 그대로다.

773명령·2779tick·3654records에 실제 배치15/재배치6/스킬4가 포함된다. Swap 명령은0이므로 새 Swap 경로는 이번 기능 실행의 통과 범위가 아니다. strict Continue4쌍은 원문/parsed 일치·loads0/peak311을 유지했다. result102,383바이트 SHA `f92a978c…`와 trace98,108,308바이트 `753d6191…`는 /tmp와 durable에 byte 동일 보존했고 공식27파일/selection/runtime/binary/source는 전후 불변이다. 이것은 빈 profile의 direct public Tutorial 완주이며 normal approved/nonempty profile/별도 프로세스 durable 검증을 대신하지 않는다. 같은 binary의 영향 Expedition002와 server 갱신 후 external-loader로 교정한 profile-only actor1exact를 이어 수행한다.


[public_expedition_complete002](product_parts/public_expedition_complete002/RESULT_001.json)은 **5.180초/exit0/reaped=true**이며 source619 `23e063f4…`가 전후 동일하다. 메인은 실제 result·원문 trace·RESULT/OUTPUT_PROVENANCE를 읽었다. direct public Expedition은 공식3층[0,1,2]·3전투 승리/ACK·HQ3·SavePoint3을 진행했고 command170의 공개 Return 선택으로 run_complete에 도달했다. 최종 profile은 sequence1/storyPending·vault 장비6/fragment3이며 active_expedition과 Continue가 없다. 영구 compendium에는 실제 만난 corroded_guard/corroded_marksman이 등록됐다. 실제170명령·612hosttick·843records다.

strict Continue6쌍은 raw17757/24375/41785/24925/42336/25057바이트와 parsed JSON이 각각 전후 일치하고 loads0·run_peak332를 보존했다. result106,363바이트 SHA `fea92c4b…`, trace22,265,525바이트 `8d4c0935…`는 /tmp와 durable에 byte 동일 보존했다. 이는 실제 빈 profile 출발→공개 Return의 완료 증거이며 first-return story ACK·다음 instance·normal approved/durable 별도 프로세스 증거는 아니다. Sol은 이미 실제 시작된 server_compile003과 external-loader actor1exact를 완료한다. 그 후 G11 cfg(test) 비용시험 및 별도 진단 입력을 작성하고 worker/normal 승인 빌드는 최종 Core source identity로 묶는다.


[server_compile003](product_parts/server_compile003/RESULT_001.json)은316.742초/exit0/reaped=true이며 source619 `23e063f4…`와 보조3입력이 전후 같다. 메인이 실제 RESULT·ARTIFACTS·지원 command/guard를 읽었다. release/default Core build `9f635d2b…`/graph `8f1ee6c6…`를 유지하고 현재 전술 진행기를 포함한 server test harness48,252,448바이트 SHA `6c6fa0e3…`를 durable에 byte 동일 보존했다. no-run 성공 자체의 기능 실행은0이다.

[actor_profile_only001](product_parts/actor_profile_only001/RESULT_001.json)은0.784초/exit0/reaped=true, 같은 source619 불변이다. 지정 nonignored exact selector는 **1passed/0failed/0ignored**이며 thread1/nocapture로 실행됐다. 메인은 실제 raw stdout·RESULT·selector333–419를 읽었다. 현재 외부 official을 새 TempDir에 package/load하여 identity/receipt 일치를 assert했고 실제 출력은 semantic `e504fdd1…`, pack/source 각27파일, runtime `7c26e5cd…`, approval_claimed=false다. profile-only durable actor의 departure는 commit 전에 거부되어 lifecycle·저장 파일·profile을 보존했고 memory actor departure는 수락됐다. 독립 title actor의 Tutorial start/skip positive control도 수행했다. 이는 external-loader fixture 교정의 정확한 기능 수용이며 normal admission·Return cycle·crash 시나리오의 실행은 아니다.


독립 author의 [G11 vault 비용 검증 묶음](test_parts/vault_cost001/AFTER.json)을 메인이 실제 source와 입력으로 검토했다. Core등록부는 mod cost1줄만 추가했고 새 cost.rs26,428바이트 SHA `36a88cc5…`, 별도 probe19,266바이트 `d3d05d99…`, collector12,672바이트 `007ec689…`다. AFTER11파일의 실제 byte/SHA를 직접 대조했다. actual Return002의 전체 profile은 그대로이고 empty/expanded 입력은 vault 이외 필드가 동일하다. expanded는 자산 내용만 정확64배·새 deterministic UUID384장비/192fragment이며, manifest SHA `5850b345…`다. 이는 측정용 입력이고 gameplay cap이나 과거 버전 전후 비교가 아니다.

Core4시험은 authoring 및 runtime-observation 활성 native의 작업량/clone at-limit·one-short 반복·full roots/UUID/continuation 불변·정상 retry를 실제 capture/entry shared restore/Continue import/prepare 경계에서 검증한다. authoring pre-cancel도 확인하며 profile capture 비용은 두 계량 모두 empty<normal<expanded를 요구한다. entry/checkpoint owned payload의 profile 제외와 native preflight scan은 구분한다. default native 비활성 시험은 unmeasured로 출력하고 실제 native 증거는 별도 feature/graph로 실행한다.

Heap 진단은 public prepare hold/discard/commit·profile JSON projection·checkpoint export·Continue·최종 owner drop의12구간이다. 입력 준비·warm-up·관측 channel 생성/해제·기대 roots JSON·로그/IO를 window 밖에 두며, consumed input 해제는 해당 실제 operation 안에 포함한다. 준비 폐기 뒤 paired baseline과 최종 모든 소유자 폐기 뒤 warm-generation baseline의 requested bytes를 정확 비교한다. release 실패도 typed JSON/partial stage와 함께 보존하고 RSS는 보조 관측이다. collector는3입력×3독립 child의 정확12stage·release·source/input/binary 결속과 raw sample/min·median·max를 보존한다. 기존 full60 부하와 normal approval graph는 대체하지 않는다.

author의 검증은 구문/AST/TOML·경로와 fixture 생성만이며 build/test/runtime0이다. 메인은 최종 source를 수용했고 Sol에 offline standalone lock/actual graph 결속→Core-only runtime-observation no-run/지정 exact4→별도 release heap build/9child 실행을 승인했다. 아직 타입/기능/회수 PASS가 아니며 실패 원문을 유지한 뒤 관련 owner만 교정한다. 성공 시 최종 default host/server·paired worker 빌드로 이어간다.


[g11_lock001](product_parts/g11_lock001/RESULT_001.json)은0.119초/exit101/reaped=true다. 메인은 실제 stderr를 읽었다. 새 standalone offline resolver가 cached `slab 0.4.10`의 yanked 표시 때문에 새 dependency 선택을 거부했으며 제품/시험 실행 실패는 아니다. source scope는 새 cost.rs를 포함한620파일 SHA `7e02815e…`로 확장됐고 전후 불변이다. 새 lock은 생성되지 않았으며 실패 원문을 보존했다.

현재 root Cargo.lock83,770바이트 `0f307fab…`을 exact seed로 새 crate에 복사하고 offline metadata로 재결속한 [g11_lock002](product_parts/g11_lock002/RESULT_001.json)는0.335초/exit0/reaped=true다. 메인은 RESULT와 두 lock의 TOML package를 직접 비교했다. 새 standalone lock73,664바이트 SHA `fe753265…`에는 diagnostic root1개만 추가됐고 기존 registry package의 name/version/source/checksum 변경은0이다.40개 비사용 workspace/기타 package 제거, 기존 dependency 이름의 버전 구별 표기 축소와 서버 dev dependency 제외만 확인했다. root lock과 source620은 불변이며 seed/전체 diff를 보존했다. 온라인 갱신·의존성 버전 임의 교정·제품 설정 변경 없이 복구했고 actual features/graph/rustc 결속 뒤 Core4·heap 실행으로 잇는다.


[g11_core_compile001](product_parts/g11_core_compile001/RESULT_001.json)은198.044초/exit101/reaped=true이며 source620 `7e02815e…`와 추가15입력이 전후 같다. 메인이 실제 JSON compiler output의4개 E0277을 읽었다. 새 cost.rs314/451의 두 로그 변환이 Display를 구현하지 않은 GameError에 ToString을 요구했다. no-run 타입 실패이며 Core4/heap 기능 실행은0이다.

독립 author는 [logging_correction001](test_parts/vault_cost001/logging_correction001/)에서 해당 두 호출만 현재 타입이 제공하는 Debug 문자열로 바꿨다. 메인은 실제2줄 diff와 새 cost.rs26,446바이트 SHA `36adbf03…`를 대조했다. ExecutionFailure의 유효한 Display 사용·모든 assertion·제품/API/feature·fixture 및 나머지10파일은 유지한다. 새 source identity로 같은 Core no-run/4exact부터 이어가며 dependency/graph/compiler의 무변경 근거는 재사용한다.


[g11_core_compile002](product_parts/g11_core_compile002/RESULT_001.json)은97.167초/exit0/reaped=true이며 source620 `225f621f…`와 추가15입력·root lock이 전후 같다. 메인은 실제 RESULT/artifact/Core build identity를 읽었다. 두 로그 수정 뒤 Core-only debug `runtime-observation` harness613,514,792바이트 SHA `2041eb3e…`가 생성됐다. 진단용 unbound build/별도 actual graph이며 normal 승인 증거가 아니다. no-run 성공 자체의 기능 실행은0이고, 같은 binary/input에서 지정4개 exact/thread1/nocapture 비용·원자성 시험을 이어 실행한다.


[g11_core_functional001_01](product_parts/g11_core_functional001_01/RESULT_001.json)은3.676초/exit0/reaped=true이며 exact1/1passed/0ignored다. 메인은 raw의 empty/normal/expanded authoring 및 활성 native 측정을 읽었다. profile capture의 work141→935→44877, authoring clone55→352→16023, native clone86→583→28854이며 각각 at-limit·one-short 반복2회·full roots 불변·정상 retry를 통과했다. native 비활성0을 측정치로 사용하지 않는다. source620 `225f621f…`/추가15입력/실행파일은 전후 동일하다.

[g11_core_functional001_02](product_parts/g11_core_functional001_02/RESULT_001.json)은3.627초/exit101/reaped=true, exact1/0passed/1failed다. empty의 첫 BattleEntrySnapshot capture가 authoring clone limit65536/requested usize::MAX로 거부됐으며 raw의 full roots 불변을 유지했다. 메인과 Sol은 실제 control의 NativeStorageProvenance·Monolith clone visitor 및 기존 owned capture 시험을 읽었다. 신규 시험은 scope 밖에서 상태를 생성한 뒤 default unknown 이력을 전달했으며, 기존 계약은 알 수 없는 native storage를 무한 요청으로 보수 거부한다. 실제 vault 비용 초과나 제품 한도 부족으로 해석하지 않는다.

독립 author가 cost.rs에서 실제 fresh construction scope로 만든 상태의 반환 이력을 같은 capture/restore family로 이어 주도록 교정한다. 외부 상태에 fresh 표시만 붙이거나 unknown을0으로 취급하지 않으며 제품·예산·입력 규모는 유지한다. 첫 PASS와 실패 원문을 보존하고 exact3/4·heap은 아직 미실행이다.

### 2026-10-13T09:11:36.545948+00:00 — 단독 실행 재개 및 provenance 교정 인수

사용자 구현 재개에 따라 서브에이전트 없이 직접 수행한다. 마지막 source620 대비 cost.rs만 바뀌었음을 대조했고 이전 활성 실행은 없다(가시 namespace 한정). 실제 diff와 control.rs의 construction-family 계약을 읽어 provenance 전달·unknown 거부 대조를 수용했다. 원본/과거 실패를 유지하고 exact1은 명시적 비영향으로 재사용, exact2/3/4는 교정 입력으로 함께 실행한다. 준비/보조 입력은 product_parts/PREPARED_G11_CORE_COMPILE_003.json, 기존 graph/compiler/lock 증거를 재사용한다. 제품·기대값·예산 변경은 없다.


단독 실행의 [g11_core_compile003](product_parts/g11_core_compile003/RESULT_001.json)은49.342초/exit0/reaped, source620 전후 동일이며 supplemental15도 직접 대조했다. 실제 harness SHA `d297d445…`를 Cargo artifact에서 결속했다. [g11_core_functional002](product_parts/g11_core_functional002/RESULT_001.json)는6.735초/exit0/reaped, exact3/3passed/0failed/0ignored다. 실제 원문의 at-limit/one-short·취소·retry·roots·prepared discard/commit 검증을 확인했다. exact1은 앞선 명시적 비영향으로 재사용한다.

다음은 G11 standalone release heap 진단이다. 기존 lock/compiler/graph를 재사용하고 별도 `.cache/u12-vault-cost-target`에서 빌드한다. 첫 빌드 시간은 미확정이며60초마다 소유 child CPU/RSS를 확인한다. 정상 child 관측은 기존 collector의600초 운영 watchdog을 따르며 제품 합격선으로 사용하지 않는다. 전체9child/12stage·정확 회수·입력 불변 판정은 기존 계약 그대로다.


[g11_heap_compile001](product_parts/g11_heap_compile001/RESULT_001.json)은246.703초/exit0/reaped·source620/supplemental15 불변으로 release 진단 실행파일을 결속했다. [g11_heap_samples001](product_parts/g11_heap_samples001/summary.json)은9/9 fresh child,각12stage,exit0/reaped 및 source/input/binary 불변을 통과했다. 각 case3회의 warm/final requested heap은 empty823105/normal823108/expanded823114 bytes로 정확 같았다. expanded profile JSON projection의 median peak-above-baseline672582bytes, command prepare310296bytes, Continue384076bytes이며 해당 wall median은0.267/0.248/0.640ms다. inactive native는 모두unmeasured이며 앞 Core native 경계검증과 구분한다. 새로운 한도나 full60/normal 승인·과거 대비 개선을 주장하지 않는다.

다음 작성은 공개 실제 패배→결과ACK→명시적포기→새원정 분기다. 실제Healer 배치·현재preview·3000×100ms기존유한tick만 사용하고 강제결과/제품수치 변경은 없다. 패배 후 실제runfailure와 사용자가 포기한terminal을 구분하여 보고한다. 원본은 test_parts/public_defeat_branch001에 보존한다.


공개 패배 분기는 test_parts/public_defeat_branch001에 두 파일 원본·diff를 보존했다. 기존 성공 전투와 별도 boundary `defeat_then_abandon`에서 실제3starter 중Healer1명만 공개range preview로 합법배치하고 기존3000×100ms 내 actual defeat를 요구한다. 결과ACK 후 활성상태면 명시적으로포기하며 이를runfailure로 바꾸어보고하지 않는다. Continue/active binding해제,vault/story/sequence보존,새출발sequence+1·서로다른instance/3직원·playablemap·strictContinue를 검사한다. 구문검사완료,기능미검증. release_compile009는 기존지원plain/default두probe 빌드이며008의295초를 참고하고60초자식진단으로진행을확인한다. 현재G11변경뒤Core source identity도 함께갱신한다.


release_compile009은296.426초/exit0/reaped·source620불변이며 supported default graph는008과같고 Core build identity는G11 cfgtest포함`21bd82f5…`다. [public_defeat001](product_parts/public_defeat001/RESULT_001.json)은6.296초/exit1/reaped,3000tick까지전투가끝나지않았다. 실제trace의Healer(4,7)170/180HP와두적(4,6)생존을확인했다. selfheal이가능한치유사선택이패배전제를만족하지않았다. 원본result/trace를byte동일보존했다. test_parts/public_defeat_branch002에서현재preview가전체적경로를덮지않는Dealer배치로교정하고,기존시간/콘텐츠/명령/terminaloracle는유지한다. 후속지원빌드는lib만변경된증분빌드로Core/서버sourceidentity는같다.


full60_content_prepare001(0.483秒)와full60_suite_prepare001는각exit0/reaped·source620불변으로기존준비기를실행했다. 현재official복사본에서허용된6적×7scalar와별도policy160만준비하고기존U11geometry/encounter bytes를재사용했다. actualHQ003의현재prefix13step을preparation에연결했으며원래roster16/500/1000·배치1/8/8·60×1000ms·seed/player/recipe를유지한다. full60_preparation001/runtime_safety.ron은원래workloads_cadence002의유한1608byte설정그대로이며정상approval의현행1605byte설정과분리한다. 생성입력준비일뿐runtime/calibration/adoptionPASS가아니다. 공개probe빌드입력과별도출력만작성하여동시빌드source는바꾸지않았다.


release_compile010은298.426초/exit0/reaped/source불변. public_defeat002는1.621초/exit1/reaped이며actualDealer(3,6)up으로666tick(66.6초)내실제Defeat→ACK→명시Abandon과profile보존을통과했다. 새출발의testhelper가교정중역할문자열을과도치환하여Dealer/Tank/Dealer중복선택을만들었고제품은InvalidAction으로거절했다. source오류를branch003에서다음선발한줄만Dealer/Tank/Healer로교정했다. 실패원문은그대로보존. compiler/env/manifests/graph와Core/serverinputs가같으므로010의지원preparedproof·정확build_command를재사용하고새collectorreceipt로lib/probe증분빌드를결속한다. 준비proof경로변경만으로Core를다시빌드하지않는다.


release_compile011은 298.477초/exit0/reaped/source 불변이다. 앞 기록의 Core 재빌드 회피는 의도였고 실제 stderr에는 Core와 서버 재컴파일이 관찰되었다. 지원 prepared proof만 재사용했으며 빌드 시간 절감을 주장하지 않는다. 최신 지시서의 도메인 규칙·짧은 관측·경로 구분 요구를 읽었다. 앞 actual Defeat 관측은 준비 전략의 근거이며 새출발까지의 전체 성공을 대신하지 않는다. 다음은 한 줄 교정의 공개 분기 검증이다.


[public_defeat003](product_parts/public_defeat003/RESULT_001.json)은2026-10-13 09:48:57 UTC 시작,1.722초/exit0/reaped/source620불변으로 전체 선택 분기를 통과했다. 실제 Defeat→결과 ACK→명시 Abandon→새 원정 sequence2/새 instance와 직원/플레이 가능한 map/strict Continue를 검증했다. result의 actual_defeat=true, explicit_abandon=true, next_expedition_playable=true, full_lifecycle_complete=false를 그대로 유지한다. 자동 run failure 증거 또는 normal admission 증거가 아니다. 실패001/002와 교정003 원본도 유지한다.


사용자의 로드맵 구현 재개 요청으로 normal/full60 잔여를 단독 진행한다. 앞 G11/공개 분기의 입력과 실제 결과를 재사용하며 새 중간 승인 조건은 없다. worker_plain_compile001은 지원 default-release worker-only 빌드이며 기존 약300초를 참고하고60초 자식 진단을 사용한다. 뒤 계측 worker는 별도 target에서 작성하며 첫 소요는 미확정이다. 측정은 빌드 종료 후 독점 실행한다. normal_contexts001은 실제 host별 독립 issuer/calibration 경로와 현행 일반 profile, 기존 full60 진단 profile을 연결한 준비 상태이며 아직 승인 결과가 아니다.


worker_plain_compile001 성공을 확인하고 실행 파일·실제 build identity를 결속했다. shared combat/node_flow/reward/world의 후속 변경 때문에 functional010/011의 마지막 PASS91개를 현 Core source와 정확히 같은 g11_core_compile003 harness로 묶어 재검증한다. 기능 검사이며 runtime-observation 구성이다. default-release 공개/normal 증거를 대신하지 않는다. 기타6 employee catalog와 생산 코드 변경 뒤의012/013 10개는 비영향 재사용 근거를 PREPARED_FINAL_CORE_REGRESSION_001에 남겼다. 기존010/011 소요는 {"functional010": 259.73596825399727, "functional011": 187.01830066300317}초이며60초마다 소유 프로세스를 진단한다.


[final_core_regression001](product_parts/final_core_regression001/RESULT_001.json)은327.705초/exit0/reaped/source620 불변으로선택91개전부통과(0failed/0ignored)했다. runtime-observation 기능검사이며 기존6+10 Core 사례와G11 4개의비영향/별도실행근거를함께유지한다. 신규실패없음. 다음계측worker의첫cold target빌드는소요미확정이며60초진단/오류종료를확인한다.


A-13-1의 공개 실패 경로를 마무리하기 위해 별도 확정 RunFailed 검증을 보강한다. public_defeat003은 실제 Defeat 뒤 명시 Abandon이며 이 결과를 자동/확정 실패로 확대하지 않는다. 실제 trace의 records_clerk는 패배 후 physical_damage1/trauma40이고 생존하므로 첫 패배가 terminal이라는 전제는 틀리다. employee.apply_incapacitation은 실제 쓰러짐마다 부상을1 증가시키며 critical5는 출전 불가다. Gate/SavePoint의 회복은 트라우마에만 적용된다. combat의 fail_run은 유효 checkpoint가 남으면 AwaitingRunFailureDecision을 거치고 ConfirmRunFailure가 최종 정산한다.

준비 전략은 공식 후보 중 회복 없는 Dealer2/Tank1을 골라 실제 공개 배치 preview가 적 경로를 공격하지 않는 합법 위치·방향에 세 직원을 배치하는 것이다. 실제 checkpoint DTO의 세 deploy cost 합에 필요한 TP만 host tick으로 기다린다. 첫 actual battle만 유한300초 안에 끝내 세 직원의 부상 증가를 관측하는 failure_battle 준비 검사를 먼저 수행한다. 그 근거가 맞으면 기존 공개 지도·HQ·SavePoint·Gate를 진행하되 깊이3 이후 계속 탐사를 선택하여 반복 부상/실제 복구 불가와 checkpoint 실패 선택에 도달하는 failure_then_depart를 수행한다. 회복 아이템/도달 가능한 회복 등 상쇄 효과가 관찰되면 짧은 원문으로 원인을 판정하며 한도/제품 수치/terminal 기대값을 완화하지 않는다. 전체512 공개 결정과 전투별300초의 기존 유한 상한을 유지한다. 현재 계측 worker 빌드 입력은 고정하고 종료 후 probe에만 추가한다.


worker_instrumented_compile001은614.622초/exit0/reaped/source620불변이다. 실제guard의feature표기는RUNTIME_OBSERVATION이며metadata점검스크립트의소문자기대를이직접출력계약에맞췄다. 제품/빌드실패가아니다. 실제plain/instrumentedsource/compiler/target/release/graph가같으며두바이너리와identity원문을각artifact에보존했다. 이제writer0실행없음에서공개failure경로만보강한다.


public_failure_branch001에기존두probe원본과diff를보존하고 failure_battle/failure_then_depart를추가했다. 기존defeat_then_abandon함수는byte그대로보존했다. 실제세선발자의public deployment cost합만큼TP를기다린뒤공개preview로합법배치하며,전투별기존3000×100ms상한에TP대기도포함한다. actual Defeat→ACK 후각동일직원의부상증가를검사하고첫전투경계는전체실패검증으로표시하지않는다. 전체경로는실제AwaitingRunFailureDecision이면ConfirmRunFailure,최종RunFailed에서만다음출발을검사한다. 제품수치와판정변경없음. rustfmt구문검사만통과했고기능은미실행이다. 지원010 proof와정확build command를재사용하며예상약298초/60초진단으로release_compile012를시작한다.


release_compile012は299.781초/exit0/source불변이며새두probe를결속했다. public_failure_battle001은3.190초/exit0/reaped/source불변으로공식3선발(records_clerk/scout_runner/breach_guard)의실제Defeat→ACK 후각physical_damage1/trauma40와viewing_map을확인했다. 첫전투준비전략이실제부상누적을만드는근거다. 전체실패판정은아직미검증이며같은입력·실행파일의failure_then_depart를이어서실행한다. 현recovery evaluator는현재선택가능한노드만살피므로물리치료없는SavePoint뒤의가상미래Gate를즉시귀환가능으로계산하지않는계약도확인했다.

### 2026-10-13 실제 RunFailed 진행 전략 교정

`public_failure001`은 10.156초/exit 1, 입력 동일·자식 회수 완료. 첫 네 실제 패배 후 physical_damage 1→2→3→4, trauma 40→70→93→100(Breakdown)을 관측했다. SavePoint/Gate의 정상 회복으로 5층 진입 시 trauma 76/Stable/physical_damage 4가 되었으며, 맵은 초기 development template에서 `endless_loop_b`로 바뀌었다. 중단은 진행기의 템플릿 목록 가정 오류이며 실제 RunFailed PASS가 아니다. 원본 result/trace와 종료 기록은 그대로 보존했다.

교정은 failure 경로에서 공개 `selectable_node_ids`와 Available 상태를 모두 만족하는 일반 Combat 노드를 선택하는 것으로 한정한다. 실제 5층 intake는 normal_corroded_patrol이며 개별 보스 경로가 아니다. 이미 관측된 physical_damage 4에서 다음 incapacitation은 5(Critical)로 진행해 물리적 배치 자격을 잃는다. 외상 회복은 물리 부상을 낮추지 않으며 목표는 여전히 실제 RunFailed/ConfirmRunFailure와 신규 원정이다. 300초 전투 진단 상한·입력·제품·판정은 유지한다. 수정 전후는 test_parts/public_failure_branch002에 보존했다. 다음 빌드는 직전 약 300초 기준, 기존 60초 CPU/RSS 진단, 완료 후 해당 경로만 실행한다.

`release_compile013`은304.652초/exit0/source 불변으로 종료했고 실제 두 실행 파일을 ARTIFACTS에 보존했다. `public_failure002`는11.644초/exit0/reaped/source 불변, 실제 결과 {"actual_defeat": true, "actual_run_failure": true, "boundary": "failure_then_depart", "defeated_battles": 5, "explicit_abandon": false, "failure_confirmed": true, "full_lifecycle_complete": false, "next_expedition_playable": true, "next_profile": {"all_live_final_bosses_defeated": false, "encounter_compendium": {"registered_ids": []}, "highest_floor_reached": 5, "lifecycle": {"active_expedition": {"player_id": "c0de1200-0000-0000-0000-000000002001", "sequence": 2, "uuid": "0e5e85ad-62c6-46cf-9215-294327847dc8"}, "expedition_sequence": 2, "first_return_story": "Never", "vault": {"equipment": {}, "fragments": {}}}, "longest_deathless_floor_streak": 4}} 이다. 다섯 전투 실제 패배→실제 실패 후보 확정→RunFailed→Continue/active 종료→다음 Expedition/새 개인/strict Continue 경로를 통과했다. explicit_abandon=false이며 직전 명시 포기 증거와 별도다. 정상 admission은 다음 별도 검증으로 남긴다.

`server_compile004` default release317.382초/exit0/source 불변. 실제 서버 harness SHA01f0327fbb5574f0ce540316a60227abe80987b6bed939668b18aa92cb9d6ca4를 별도 보존했다. `server_functional001`은 실제 exact3/3 PASS·0 fail·0 ignored,1.573초/exit0/source 불변. profile-only actor 권한 거절과 memory 양성,16MiB exact/+1byte direct 및 paired preaccept 원자성, checksum-valid duplicate raw vault 거절을 최종 harness에서 확인했다. 다음 standalone normal calibration은 공식13 measured steps/Normal1/repeats3, build와 격리하고 기존300초 worker 상한 및60초 진단을 유지한다. 최초 현재입력 실측 소요는 미확정이며 결과 검토 전 adoption하지 않는다.

### normal approval 입력의 실제 준비 계약 교정

`normal_standalone_calibration001`은7.901초/exit1/reaped/source 불변, `Incomplete at lab.prepare: authoring_trial_battle_not_prepared`로 종료했다. 기존 producer의 HQ-only13 measured steps/빈 prefix는 실제 TrialSession::finish_preparation_in_scope가 요구하는 completed battle entry에 도달하지 못한다. 채택·승인은 생성하지 않았다. 제품 결함이나 한도 초과로 분류하지 않는다.

producer를 실제 public_defeat003의 같은 공식 콘텐츠·seed·3선발·HQ·일반전 진입·실제 records_clerk 배치까지 명시 preparation prefix로 만들도록 교정했다. 그 입력은100ms×666까지 생존해66.6초에 실제 패배했으므로60초 관측의 도달 근거가 있다. runtime은60×1000ms로 하며 기존 이벤트 스케줄러의 실제 실행은 새 calibration으로 확인한다. LabPreparation/직원 주입 없이 실제3직원/1배치다. 초기13단계가 runtime 측정이었다는 기존 준비 설계는 폐기하며 실제 setup 비용과 runtime 비용을 분리한다. G11 별도 full60의16/500/1000 및1/8/8은 변경하지 않는다. 새 normal suite revision/config/증거 디렉터리를 사용하며 old normal_inputs001 및 실패 원본을 유지한다. 현재 Rust/Core/worker/host bytes는 변경되지 않아 기존 빌드를 재사용한다.

`normal_inputs_prepare002` 실제 준비0.734초/exit0 후 `normal_standalone_calibration002`7.951초/exit1은 `missing_battle_state_authority`였다. 공개 진행기 Trace::command가 in_battle 명령에 자동 제공하는 최신 battle UUID/revision context를 scenario producer가 누락했다. actual command 입력만 복사해서는 그 명령의 공개 execution context를 보존하지 못한다. 같은 직전 Run snapshot에서 UUID/revision을 typed binding하여 deploy command context에 전달하도록 교정했다. 이는 자격 우회가 아니라 원래 공개 호출의 필수 권한 결속 복원이다. prefix22/runtime60·내용·한도·제품·worker/host 빌드는 동일, 옛 input002/config002/failure는 보존하고 input003을 생성한다.

`normal_standalone_calibration003`은9.741초/exit0/source 불변. plain/instrumented 각3 fresh processes에서 acquisition/setup/execute/drop 각3표본,총24표본 complete·parity·coverage를 확인했다. calibration 결과의 incomplete 사유는 의도된 미채택 criterion 하나뿐이다. REVIEW.json에 구간별min/median/max,현재 한도 비교를 기록했다. 최대벽시간0.5632초/heap149456350B/RSS187060224B/native5630544로 기존6초/1280MiB/1536MiB/233M 한도를 유지한다. plain native는 미계측, drop 후 live110327B는 관측값이며 무조건0회수라고 주장하지 않는다. raw worker/process/input은 calibration_raw에 byte 동일 보존했고 private issuer store는 복사하지 않았다. 채택은 동일host의 새 calibration을 수행하는 정상 관리 API로 진행한다.

Standalone adoption9.640초 및 server calibration9.840초/adoption9.893초 모두exit0/source불변. 서버 전용host01f032 identity에 별도 complete24sample/coverage/parity를 확인하고 같은 기존 유한한도를 정상 API로 채택했다. 관리 exact1의 성공은 gameplay 기능PASS로 계수하지 않는다. 실제 귀환→2사본대여→유료HQ→SavePoint/ACK/복원→두번째귀환 부모를 현재 승인host에서 처음 실행한다. 초기승인실측 약10초+public direct약5초/원정 기준이나 paired 파일쓰기 누적 비용은 미확정,60초 descendant CPU/RSS 진단과512decision/전투300초 기존 한도를 유지한다. 원문 profile/active seed는 실제 실행으로만 생성한다.

`normal_return_cycle001`은32.645초/exit0/exact1 PASS0fail0ignored/source 불변. 실제 normal admission 첫귀환6장비3파편,두사본 대여·duplicate/third 거절·새개인·유료HQ·SavePoint 이후storyACK/합법이전checkpoint복원·두번째귀환Consumed 유지까지 통과했다. 실제5seed와trace/원본저장은 evidence에 byte 동일 보존했다. 이를 소비하는3부모와 나머지6저장/actor부모를 같은harness/env/thread1/exact9로 묶어 실행한다. child helper는 별도PASS수로 계수하지 않는다. 두 fullTutorial약40초+crash/restart·admissioncache 반복이며 총시간은 첫 실제측정,60초CPU/RSS진단을 유지한다. 실행argv 조립의 사전assert를 교정했으며 잘못된argv로target을 실행하지 않았다.

`normal_persistence001`은343.336초/exit101/source 불변,실제9부모 중8PASS/1FAIL/0ignored였다. 손상reset의미수락·accepted crash, title reset, nonempty Pending/Consumed 각각전체Tutorial+fresh reopen,22귀환/대여/ACK crash 경계,Abandon crash,구형원본,초기stale CAS는 통과했다. raw원본755파일을 evidence에hash동일보존했다. 단일실패는profile-only ACK CAS의마지막양성대조: 두 stale(Pending/Consumed-no-change) publication 거절은이미성공했으나B가revision을진행시키려고StartNewGame으로연blocking briefing 안에서duplicateACK를시도해 context_irrelevant로거절됐다. 전체실패를제품CAS결함이나무변경수락으로재분류하지않는다.

정상publishedprofile-only 상태를 Host::restore로다시열어같은accepted revision/checkpointNone/Consumed profile을확인한후duplicateACK 양성을검사한다. 앞두stale거절·모든roots/files/revision불변 oracle는그대로다. 수정전후는test_parts/normal_ack_correction001에보존했다. shared test 안의단일함수변경이므로8성공과Return부모/다른공개경로는비영향재사용한다. Core/worker/defaultpublichost소스는무변경이며서버harness만새빌드·새hostidentity calibration/adoption후해당1검사를재실행한다. 새harness에서도기존실제seed를정상required-configuration 복원으로소비한다. 빌드예상직전317초/60초CPU-RSS진단을유지한다.


### 2026-10-13 정상 ACK 교정 종료 및 로컬 정상 경로

`server_compile005`는319.239초/exit0/reaped/source 불변이며 supplemental shared test SHA32955b25…도 시작 전 identity와 같다. 실제 보존 harness SHAbda6225c…에 별도 `normal_contexts004`를 준비했다. calibration00210.041초/adoption0029.889초에서 각24표본 complete·parity·coverage와 기존6초/233M/1280MiB/1536MiB 한도를 유지했다. 새 calibration 최대구간0.5755초/heap149456332B로 한도 상향은 없다. 원문 worker evidence와 관리결과를 보존하고 private issuer는 복사하지 않았다.

`normal_ack002`는10.392초/exit0/source 불변, 실제exact1 PASS·0fail·0ignored다. 기존 genuine 첫귀환 seed를 새 승인host에서 정상복원하여 Pending stale 및 Consumed 무변경 stale 거절과 최신 committed title의 duplicateACK 양성을 모두 통과했다. 이전 실패는 보존하며 수정된 단일함수 외8부모·Return부모는 비영향 재사용한다. 증거9파일은 OUTPUT_PROVENANCE로 byte 동일 보존했다.

로컬 same durable normal host/config003의 `normal_local_return001`은20.329초/exit0/source 불변, 실제3전투/HQ3/SavePoint3→귀환 complete다. 종료 저장과 trace를 다음 출발 전에 별도 evidence에 보존했다. `normal_local_start001`2.575초/exit0 뒤 새 OS프로세스 Continue도exit0이며 반환 roots가 동일하다. 실제 failure_then_depart와 그 fresh process 복원은 별도 실행한다. 현재 full60은 미실행이며 Normal16/1 한 paired repeat 사전관측을 준비했다. 기존13 public HQ prefix/recipe/60×1000ms/진단 runtime bytes를 유지하고, 모든3등급×3회 측정·채택은 사전관측과 별개다.


### full60 비용과 현재 승인/거절 경계

로컬 `normal_local_failure001`은34.949초/exit0/source 불변,5전투 실제패배→ConfirmRunFailure→새 원정 sequence2/새개인으로 정상 완료했다. 별도프로세스 Continue도exit0이며 직전 roots와 revision이 정확히 같다. Return→Start→Continue19파일 및 Failure→Continue13파일은 각각 normal_local_continue001/evidence와 normal_local_failure_continue001/evidence에byte동일보존했다. 두경로는 정상공식config003·실제samehost admission을 사용했고 full60 진단입력과 별개다.

`full60_normal_calibration001`은4.730초/exit0로 Normal16/1의60×1000ms전체 onepairedrepeat를 확인했다. 이어 `full60_combined_calibration001`은30.689초/exit0/source불변이며 normal16/1,expanded500/8,pathological1000/8 모두13preparation+60runtime=73 acceptedsteps를 각plain/instrumented3 freshprocess에서 확인했다. 총24구간표본 complete·parity·누락/overflow0이다. 이cost-only suite의required-definition coverage는빈값이며 새정의사용증거로확대하지않는다.

실행구간 plain최대2.9629초/instrumented최대3.1491초,native210559102,requestedheappeak823115181B,RSS최대922963968B,drop후live218353B(3회동일)다. setup최대0.6565초·heap93.06MB; acquisition약0.575초다. 기존6초/233M/1280MiB/1536MiB한도를유지하며자동상향이나최적화주장은없다. 원문6worker와content/request를calibration_raw에보존했고기존진단runtime_fbd5…는productionconfig_a1bb…와구별한다. 계량nativework와wall/heap/RSS를동일시하지않으며 dropRSS는OS잔류이고요청liveheap과다르다.

`full60_adoption001`30.789초/exit0로samehost가freshcalibration후기준을정상채택했다. `full60_accept001`30.336초/exit0는그기준으로실제admission→정상Start/저장을수행했다. `full60_missing_required001`은예상exit1/required mods/packs/writers differ 거절로savedcurrent/backup전체hash불변이었다. `full60_stale_criterion001`은profile_revision만바꾼negative에서complete측정을하더라도criterion not calibrated/adopted로base를hold해거절했고active/profile경로도생성하지않았다. 이어원래승인config로 `full60_continue001`2.588초/exit0의별도프로세스복원과roots/revision동일성을확인했다. 전체16파일은해당evidence에보존했다. 예상거절exit를일반실패나Rust test PASS수로혼동하지않는다.

최종coverage대조중 employee성격/condition,strictcurrent/nestedcodec·V7/V8명시거절,6starter/roster10와deployment,지원draft/clone,고정tutorialdata 및 currentapproval generation의일부기존selector가현재U12결과에연결되지않았음을확인했다. 기존현재Coreharness로 `PREPARED_FINAL_COVERAGE_REGRESSION_001.json`의exact묶음만추가실행한다. 이미통과한91및nonimpact10·data6·server3·G11·normal부모는재실행하지않는다. stdout안의계량JSON이test라인과ok사이에들어가는경우를정확히구분한다. 기존simple한줄regex는PASS목록을누락했으므로최종집계는실제test시작/종료와summary 및exactargv를대조한다.

## 2026-10-13 final coverage regression and direct test correction

`final_coverage_regression001` completed112.604s/exit101, actual80 selected/72PASS/8FAIL/0ignored, child reaped and620source inputs unchanged. Raw failures remain preserved. Root reviewed the actual owners: Expedition uses endless briefing; public action set is79; fragment key mismatch is rejected first by union ownership; first-entry deterrence charge precedes immutable battle snapshot; paid HQ hire remains in session until explicit support completion; containment_trainee now receives shield/last_line for the two-per-role starter contract. These are stale test assumptions, not grounds to modify current product rules.

`test_parts/final_coverage_correction001` preserves all five preimages and exact diffs. First-entry expected projection subtracts independently specified1 and preserves every other field; all shared-helper consumers are selected. Recruitment funds seven paid hires explicitly, closes each HQ, and verifies hidden offers plus rejected command at ten. Six starter active cases use current official loadouts. Two added tests cover aptitude-only identical weapon/effective battle inputs and null/empty/structured removed psychology in current and explicit nested genuine SavePoint, with a valid checksum/positive import/retry/full-root atomicity. Original test authorship remains historical; these corrections and new assertions are root-authored under current single-executor instructions.

Prepared final_coverage_compile001 reuses original collector/offline locked runtime-observation context. Prior production behavior and72 unaffected cases remain eligible for nonimpact reuse; changed test-body cases, briefing sibling, all retreat expectation consumers and new oracles will execute together after a successful compile. cfg(test) source does change build identity; exact approved host evidence will be reconciled separately rather than silently claimed current.

`final_coverage_compile001`54.290s/exit0 and `final_coverage_regression002`46.385s/actual13PASS/0fail/0ignored, source620 unchanged/reaped. All8 earlier failures are corrected;3 sibling/shared-fixture cases and2 new oracles pass. No production code was altered in this correction. Current Core harness has2097 tests, selected13, filtered2084. The unaffected72 and91 prior cases retain explicit nonimpact reuse. `PREPARED_FINAL_IDENTITY_REBIND_001` prepares supported final worker pair, standalone and server builds because CoreBuildIdentity hashes cfg(test) bytes too. Production logic/official data/runtime limits remain unchanged; previous public, crash and cost results will retain nonimpact scope, while fresh normal approval/consumer checks bind the new exact artifacts.

Final supported workers `worker_plain_compile002`289.205s and `worker_instrumented_compile002`293s approximately bothexit0/reaped/scoped inputs unchanged; exact duration is RESULT. Runtime queried source/compiler576e095d… and dependency graph8f1ee6c6… match across plain/instrumented. Both immutable executable copies and queried BUILD_IDENTITY are retained. During final A-09-1 review, a more specific current-copy-vs-old-SavePoint counterexample was added solely to the external server test file after both builds ended. It preserves genuine public seeds, removes two unequipped current copies in a declared valid-checksum boundary fixture while retaining old SavePoint/history, then expects Return/persist/reopen/duplicate handling to return every other current copy without reissuing the absent two. It does not claim public dismantling. This external cfg(test) file is outside CoreBuildIdentity and does not change workers; final server harness will include it. Preimage/diff are in test_parts/current_copy_return001. Standalone release_compile014 reuses the exact supported worker plain proof/package graph and builds only the two probe bins; unexecuted broad preparation is superseded.

`server_compile006`319.503s/exit0/scoped and supplemental inputs unchanged/reaped includes the added current-copy-return oracle. Final standalone calibration004 stopped0.021s/exit1 before calibration because administration requires an existing empty evidence directory; current file/config/source unaffected, original failure preserved. Prepared missing directories for all3contexts xcalibration/adoption, reissued as standalone005; server003 andfull60_002 had not run. This is runner preparation, not a failed product scenario.

Standalone005 reached worker-root validation and rejected2.223s before any samples: newly created directory mode0755, while lab.rs1676–1680 requires owner uid, no symlink and0700. All6 owned empty evidence directories were checked and changed to0700; no keys/stores existed, no source/config changes. Standalone006 is the corrected invocation; server003/full60_002 remain unexecuted until it succeeds. Both preparation failures stay preserved.


## 2026-10-13 최종 host 승인·실제 소비와 최종화 입력

최종 source/compiler576e095d…에 대해 standalone calibration006 9.753초, server003 9.889초, full60 combined002 30.186초가 각각 exit0/source불변/reaped로 끝났다. 각 문맥은24표본·6개 worker의 gameplay일치·전체 측정 완료를 확인했다. 정상82step(22prep+60runtime), full60의3규모 각73step(13prep+60runtime)을 유지했다. Instrumented execute 최대는 정상 standalone0.277968초/server0.280453초, full603.093749초/native210559102/heap823115183B/RSS922693632B다. 기존6초/233M/1280MiB/1536MiB는 변경하지 않았다. 실제 min/median/max와 drop 회수·원시 worker 파일은 각 REVIEW.json/calibration_raw에 있다.

standalone adoption002 9.639초, server adoption003 9.902초, full60 adoption002 30.388초 모두 exit0다. session.rs의 채택 응답 outcome은 채택 이전 calibration의 상태이므로 이를 사후 admission 성공으로 읽지 않는다. ADOPTION_REVIEW에 구분했고, 아래 실제 후속 host open이 채택 기준 소비의 증거다.

normal_return_cycle002 32.844초/exact1PASS/0fail/0ignored는 최종 서버 context005에서 실제 귀환→혼합2사본 대여→유료HQ→SavePoint 이후 storyACK/복원→다음귀환을 수행했다. genuine5seed와 원문 trace를 evidence에 보존했다. 같은 harness/context의 normal_return_boundaries003 5.331초/exact2PASS/0fail/0ignored는 stale ACK 거절/최신양성과 현재 없어진2사본을 오래된 SavePoint로 재발급하지 않는 Return/중복/reopen을 검증했다. 후자는 유효 checksum의 명시 I fixture이며 공개 dismantling 경로가 아니다. 과거 부분 실패8/9와 교정 이력은 원문 그대로 남는다.

최종 standalone release014/context005로 normal_local_start002 10.091초→새프로세스 normal_local_continue002 2.524초, full60_accept002 30.544초→새프로세스 full60_continue002 2.625초가 모두 exit0/source불변/reaped다. 각 Continue는 직전 Start의 전체 roots/required/revision과 정확히 동일하다. OUTPUT_PROVENANCE와 evidence/committed_active/committed_profile/CONTINUE_COMPARISON을 결속했다. live private issuer/키는 복사하지 않는다.

최종화는 기존 U11 archive/patch generic helper를 재사용하는 finalization001/freeze_target.py와 요구항목-실제 selector/raw 연결용 collect_coverage.py로 수행한다. --prepare-source → collect_coverage.py → ROOT_CURRENTNESS_REVIEW 작성 → freeze_target.py 순서다. 38수용·19IVA·10차원·3node, U03/U04/U07M/U11 영향, 과거 실패 원본과 비영향 재사용을 명시한다. final source와 기록의 정확 정역 패치를 임시 격리 디렉터리에서만 적용한다. 실제 복원 성공은 target/u12_current_001/RESTORE_VERIFICATION.json, 동결 완료는 MANIFEST.json으로 확인하며 이 문장 자체는 사전 PASS가 아니다.

P2G_50_GOAL_SET의 Applicable Risk / Reviewer Requirement와 PLAN 검증·완료는 계획/제품/시험 비참여자의 whole-Goal Complete/High/open0를 요구한다. 현재 단독 지시서 §8은 이 경우에도 서브에이전트를 부르지 않고 가능한 작업 뒤 독립 판정 미충족을 보고하도록 한다. Root 자체 source/oracle/raw 검토는 이를 대신하지 않는다. 후속 독립 검토는 고정 target 외부에서 수행해야 하며 그 전 U12 완료와 종속 U13 착수는 미충족이다. 기능/계량 runner는 모두 종료했고 최종화 외 입력 writer는 없다.


최종화 교정: 첫 source inventory의 Path component 순서와 기존 U11 복원 helper의 literal path 순서가 달라 같은 집합도 비교가 일치하지 않는 것을 사전 확인했다. 제품/기능 실패가 아니다. 소유 finalizer session10875를 Ctrl-C로 중단하고 exit130을 확인했다. 미동결 manifest/coverage/review 원문과 script preimage는 finalization001/interrupted001에 보존했다. 부분 target/u12_current_001도 보존하며 성공으로 표시하지 않는다. literal path 순서로 통일하고 단계 진단 출력을 추가하여 새 target/u12_current_002에서 같은 정역복원을 수행한다. 이전001 예정 참조는 중단 이력이며 최종 성공은002의 MANIFEST/RESTORE_VERIFICATION만 소유한다. 제품·시험·worker 입력은 변경하지 않았다.
