# U08 실행 기록

## 최신 실행 — 2026-10-07 / 감사 교정·최종화013

[공개 paused003](../../evidence/audit_f01_paused_probe001/attempt003/HANDOFF.json)은84개 명시적 검사/20명령/57host ticks, 실제 Normal144·Corrupt210 HP 손실과 출처 인과관계를 통과했다. 실패001의 lock·과도한 acceptance 기대와002의 JSON 필드 파싱 오류는 보존했다. 제품 source438/4c7bbd31·기존1322 입력은 불변이다.

새 증거 검증 도구는 실제 정상1·변조거부10 검사를 통과했고, Sol의 [보충 검증013](../../evidence/supplemental_paused_validation013_boundary/CLOSED.json)은41.017초 exit0다. [최종화013](../../evidence/finalizer_execution013/CLOSED_INTERVAL.json)은 selection125.151초·impact114.432초·adaptation53.676초·coverage81.230초 모두 exit0, [GEX013](../../evidence/finalizer_execution013/GEX_CLOSED.json)은47.659초 exit0다. 구성 작성 중 집계 기준 오류는 보존·정정했으며 제품/최종화 실패가 아니다.

기존58역할·다섯 실행 구간·성능8과 원50기능 quiescence는 그대로다. 새 공개 probe와 보충 검증은 별도 producer이며 측정 이전 실행으로 재표기하지 않았다. 현재 source와 고정 참조가 전후 동일하며, 다음은013 동결·격리복원·전체 독립 재감사다.

## 직전 최종화012 기록

실제 제품 검증58역할은007의17·008의7·집중005의1·009의24·010의9라는 다섯 구간으로 선택했다. 원 BEFORE/AFTER와 중간 gap을 보존하며 하나의 연속 실행이라고 주장하지 않는다. [전체 실행 종료](../../evidence/execution_harness_binding010/CLOSED_INTERVAL.json), [실제 역할 결과](../../evidence/execution_harness_binding010/ROLE_RESULTS.json), [입력 구간 결속](../../evidence/finalizer_ready011/COMPOSITE_BINDING.json).

실패한 WebSocket007과 준비 시도들, optional009의 manifest 경로 오기, final_adaptation011의 역사 predecessor 참조 오류를 원본 보존했다. 집중005, publication010과 READY012가 실제 후속이다. adaptation 교정은 probe003의 predecessor를 역사probe002로 복구했으며 기존 diff·oracle·제품은 유지했다. 동시에 발견한 외부 정책6문서 변경은 U16후속 범위 검토 뒤 현재 참조로 결속했다.

[최종화012 종료](../../evidence/finalizer_execution012/CLOSED_INTERVAL.json): selection119.227초·impact106.130초·adaptation52.050초·coverage56.685초, 모두 exit0. [GEX10 원receipt](../../evidence/final_gex012.json)는44.769초 exit0다. 각 단계 source unchanged. 문서 갱신 다음은 직접 동결과 전체 독립 감사이며 제품·성능 재검사는 추가하지 않았다.

## 이전 실행·계획 이력

아래의 현재·진행 중·pending 표현은 각 기록 시점의 상태이며 위 최신 결과를 대체하지 않는다.

**2026-10-07 집중 검증 인수034:** 단일 wave·3 actor의 집중005가 정상 입장 후 8,800ms까지 진행하고 실제 HostTick native work 1,148 > 원한도1,000으로 fault를 관측했다. 기존 명령 거절·저장 보존·재접속·교체 연결·quit/Continue oracle가 모두 통과했고 actual snapshot_v1 네 개와 canonical runner를 닫았다. source438/4c7bbd31·원1322는 불변이다. Supplement Astra가 새 fault_off 집계기와 ResourceSum probe003 참조를 준비하며 Sol 통합은 준비 인수 전 대기한다. 전체58 역할/기능50·성능8과 두50 quiescence, 최종화·전체 독립감사는 아직 미완료다. [현재 기록](../../../../roadmaps/core_first/execution/u08_public_20261007_034/RECORD.json)

**2026-10-06 fault 입력 교정 인수033:** 008 WebSocket 검사는 실제 100초 진행 중 fault 미관측으로 실패했다. 원 cap1000과 Host100ms operation의 차이를 확인하여 새 정상 RON 지연 workload를 main Astra가 집중 교정한다. 기존 source438/4c7bbd31과 입력1322는 불변이며, 19개 판정 연결 성공과 native 5개 exit0·aggregate 미완료를 구분한다. 네 계약 문서 보완은 root가 인수했다. 원 실패·성공은 보존하고 전체 U08·성능·독립 감사·U09는 미완료다. [현재 기록](../../../../roadmaps/core_first/execution/u08_public_20261006_033/RECORD.ko.md)

최신 후속 인수: docs_checkpoint026는 통합005 fmt/Core default 1,957 PASS(lib1,884, ignored0)와 비유한 RON 검증 누락에 따른 outer -9 계획 중단, 사용자 중단 nonfinite001 raw-only, nonfinite002 exact1시험/16사례 PASS를 구분한다. 현재 source438/9af38a90와 Cargo.lock은 unchanged이며 dependency_fetch001은 locked 캐시 준비 증거다. 현재 danger-full-access/network enabled/approval never를 적용하고 기존025 EPERM·002/003/004 실패는 당시 이력으로 보존한다. fresh harness004와 전체통합006은 준비 중이며 현재 전체 기능·성능·16.2 독립감사는 미완료다. 아래 이전 최신/pending 문구는 당시 이력이다.

최신 후속 인수: docs_checkpoint025는 통합004 Core default 1,872 PASS/12 FAIL 원본과 ABORTED_INTERVAL을 보존한다. 그중 catalog·capability 11개 exact selector는 Astra 교정 후 각1/1 PASS, Sol post_correction_fmt001 PASS/변경0이다. 닫힌 교정 source438/8734111c이며 AF_UNIX bind·AF_INET socket EPERM 환경 차단은 그대로다. 전체 기능통합·성능·동결·독립감사는 미완료이며 실패를 skip/PASS로 승격하지 않는다. 아래 이전 최신/pending 문구는 당시 이력이다.


최신 후속 인수: docs_checkpoint024의 G6 lifetime0021/1(실제3회)·carrier_module00112/12 PASS. 동일 닫힌438/ce0e0f2f 경계에서 affected module까지 재실행했다. Weak는 strong owner/T·소유 값의 drop을 확인하며 backing allocation 전체 해제·총heap상계·admission환급을 뜻하지 않는다. prep002는 준비 교정 인수이며 G7/현재 통합·성능·동결·독립완료는 아직 잔여다.



최신 후속 인수: docs_checkpoint023의 G5 retreat002·wire002·host003 각1/1 PASS. 동일 닫힌438/eae98e63 경계에서 세 selector를 모두 재실행했고 초기 host 카탈로그 준비/Retreat counter 기대 오류는 원본으로 보존한다. Astra G6 읽기 preflight와 finalizer prep002 교정 중의 live source 고정·전체 U08 완료·실제 finalization PASS를 뜻하지 않는다.



최신 후속 인수: docs_checkpoint022/public020의 G3 final carrier_module00111/11·lifecycle_module0013/3 PASS. 동일 닫힌438/b3f0af7e 경계에서 공통 helper 영향까지 재실행했고 초기3FAIL/교정은 보존한다. G5 preflight와 별도 신규 finalizer 작성 중의 live source 고정·전체 U08 완료를 뜻하지 않는다.


최신 후속 인수: docs_checkpoint021/public018의 G2 restore0021/1과 matrix0031/1·14실제사례 PASS. 동일 닫힌438/0eb57194 입력에서 실행했고 초기 oracle/fixture 실패와 이전 exact 성공은 보존한다. Astra G3 preflight/후속 변경 중의 live source 고정이나 전체 U08 완료를 뜻하지 않는다.


최신 후속 인수: docs_checkpoint020/public017의 G4 atomic0011/1·assets0011/1·module00113/13 PASS. 최종 module은 새2시험을 포함해 닫힌438/4f289cee에서 재실행했다. 이 경계는 Astra G2 preflight/후속 변경의 live source 고정을 뜻하지 않으며 앞선 역사·원실패는 보존한다.


최신 후속 인수: docs_checkpoint019/public016의 G1 duplicate002·battle002·weapon002 각1/1 PASS. 실행 source는 각각8f5055d9/d0f3b26a/04a14334이며 후속 시험-only delta의 명시 비영향 재사용을 구분한다. 마지막 닫힌438/04a14334 경계는 Astra G4 작성·집중 실행 중의 live 고정을 뜻하지 않는다. 이전 현재·pending 문구와 실패는 이력으로 보존한다.


최신 후속 인수: docs_checkpoint018의 통합002 fmt 실패/통합003 stack abort와 world scope 한파일 교정 후 R07 exact1/1. 닫힌 검증 source438/73a3d7b6이며 전체 통합을 대신하지 않는다. Astra G1–G7 보강 단계에서 제품·통합·성능 HOLD, 앞선 current/pending 표현은 당시 이력이다.


최신 후속 인수: docs_checkpoint017/public012의 basic001·epoch005·carrier complete001 종료 경계. 최종 carrier7은 기존5+기본공격1+epoch1을 포함한다. 검증 source438/0df25480은 닫힌 실행 입력이며 Astra 도구 작성 중의 live source/전체 U08 완료를 뜻하지 않는다. 앞선 현재·pending 문구는 당시 이력으로 보존한다.


최신 후속 인수: docs_checkpoint016/public011의 관련12·resource projectile1/fixed1 종료 receipt. Astra basic-stage 작성 중이며 검증 경계939a61e1을 live source로 쓰지 않는다.

최신 문서 인수: docs_checkpoint015의 E009/Continue 및 private004·carrier_stage2002 종료 경계. 해당 source는 과거 검증 입력이며 실시간 current/전체 U08 완료가 아니다.

현재 P2G-00→60 완료, quality2 PASS. 아래 actual execution별 국소 제품 검증을 수행했다. 전체목표 자체검증은 아직 미완료.
GEX00 baseline 이후 구현자가 새 고유 label별 source·command·선택 건수·exit·raw·기대/관찰·실패/교정을 기록한다.
기존 U07/U07-M receipt는 exact 선행 인수이며 새 유물 실행 성공이 아니다.

## GEX00_002 — 실제 기준 보존

2026-10-06 지정 Sol/xHigh 실행 인수. `evidence/gex00_002/GEX00.json`에 product398/f214, harness129, 문서52, Git dirty/index/worktree 및 정역 비교용 원 bytes를 보존했다. 기존54receipts의 hash를 확인했고 새 유물 성공으로 세지 않는다. cargo/rustc0.
초기001은 PRODUCT_BEFORE.json 이름 충돌로 exit1이며 product mutation0, FAILED_ATTEMPT.json과 successor002를 보존했다.


- relic_world001: compile101/45.579s/source unchanged0ee01221, new public command/result exhaustive 및 mut closure 누락. 원본 보존.
- relic_world002: exit0/126.349s/6 tests PASS/source unchanged285d06af. normal grant/transfer/corrupt preview/maintenance cancel+confirm/overflow/first cumulative exact+short/next seed/authored dust-zero 진단. 수치·durable 전체 증거 아님.
- continuation_owner_fmt001/002: raw rustfmt exit0, intended mutation runner99. preimages와 source archive 보존.
- continuation_owner001: compile101/34.006s/source unchangedadb9fad1; allocation::admit_json_buffer checked Option chain와 다른 module의 private u04 fixture helper 두 오류. fresh002가 실제 후속.
- continuation_owner002: relic selector 실행 중. 입력 고정; 종료 전 PASS 예고 없음.

- continuation_owner002: sticky io Interrupted 재시도 hang으로 owned interrupt130. 15부분PASS/1fixtureFAIL(미완료Maintenance 재개점 기대오류); normal receipt 미완결 대신 continuation_owner002_INTERRUPTED.json에 실제 raw/source/command를 결속. 원본 불변.
- continuation_owner003: nonretry io Other 및 정상 CompleteNode 재개점 교정 후 exit0/70.720s/17 PASS/source406 b043f07c unchanged. 첫 grant1055357/후속 seed22127/Continue5973192는 독립 Value/Vec/Arc 보수 admission을 포함하며 실제 heap과 구별.
- relic_boundary_fmt001: rawfmt0/runner99; recursive child formatter delta47개를 exactpreimages로 선택복원. formatter_children_restore001/RESTORATION.json 및 POST_FORMAT_CHILDREN.tar.gz 보존. 이후 skip_children=true.
- relic_boundary001: test-only compile101/72.906s/source192b71ec unchanged; private fixture helper/validatedArc 호출/Debug bound 일반 교정.
- relic_boundary002: runtime-observation relic selector exit0/122.240s/22 PASS/source406 48ba52f13e0491411fb7e178a1bf649939bbc08f322e836a6eca6083e6581382 unchanged; raw4336ce0d365843bfe67dfdcebc3f21cf0baaf5e902f8393d03afdcad10260fb6. 초기R04 bound8279·normalfactory 선등록·emptyclone0·presence roundtrip·공개staged/discard/commit 포함. enum32/CommittedSnapshot264/OperationSummary1072/SessionState384/encode32는 actual현재size이고 이전차이는 아직 미측정.

- relic_boundary_off001: default features exit0/110.218s/20 PASS/source406 48ba52f1 unchanged, rawdcf9b3db987f25d15838c41756eeb975fac12fa42cba1b0c6cd1e5d5197901f0. on22/off20 shared owner/save/registration assertions 동일.

- relic_numeric_check001: newowner Debug derive demanded existing GameDataBase Debug; exit101/40.040s/source unchanged062a909e. owner-only Debug successor, 원 catalog 공개타입 변경 없음.

- relic_numeric_focused001: test-only ron parser/UnitStats equality 2compile errors; exit101/66.989s/source61bbd8a8 unchanged/raw3ec9f226. production lib compiled but test execution0, PASS 아님. fixture-only successor002.

- relic_numeric_focused002: exit101/94.728s/source2f40a42b unchanged;27 PASS/1fixtureFAIL. 정상static constant4294967296은 원compile overflow로 거절되어 runtimebase guard까지 못 갔다. dynamic Source.Attack×42949673로 정상compile→runtimeoverflow 경계를 선택해003에서 재검증한다. literal-off/Expr두moment·twoemployees·querybase/effective, basiconce·attack보존·otherproducer제외,sourcecoexist,finalpenalty 네 경계는 이 실패run 부분관측으로 보존. common source136/capture192/cold24 실측.

## 시험 역할 분리 이후 실제 결과 — 2026-10-06/001

사용자 지시로 Sol은 제품·제품 문서·포맷·합의된 실행·raw·제품 교정을, Astra/high는 모든 시험 코드·fixture·기대값·probe·oracle·comparator를 담당한다. 이전 Sol 시험 작성 이력은 보존하며 과거 기록을 새 독립 시험으로 재명명하지 않는다. 실제 역할 기록은 `core_first/execution/u08_roles_20261006_001/RECORD.ko.md`이다.

- relic_numeric_focused003: 32 PASS/1 fixture FAIL, exit101/61.531s/source unchanged=True, sourcec9d11fd119d94897532af3afe71e9bf16cb612e7740d5418b8033860196ff72b. raw SHA6f3a356d9d71b20b508222d0653ddfab872f209130230f0ef7bdb7a7634d3690. 기존 i32 literal parser 범위를 넘긴 시험 입력이다. 제품 parser를 확장하지 않고 Astra가 typed Const Magnitude로 교정했다.
- relic_numeric_focused004: 33/33 PASS, exit0/46.074s/source unchanged=True, sourceeca6c373753233e5b3810ba06f40f276508e9281f5f5ba5d19580d86d8c83068. raw SHA5defbf5adf79da55183922f772e84cb3c19a1e69bdf1660bd38ac1f40112077b. 기존 Sol 6개와 Astra 4개의 numeric 경계 포함. literal observation-off, Snapshot/Live·provider ALLAND·원 base/final guard·released carrier·공존 source·rollback을 국소 검증했다.
- relic_numeric_off001: 31/31 PASS, exit0/97.751s/source unchanged=True, sourceeca6c373753233e5b3810ba06f40f276508e9281f5f5ba5d19580d86d8c83068. raw SHAaea150e71e50a9dad43a0cdf34921867d1121300f0417e43e05bc48a94ec756e. 같은 numeric 10개 및 owner/save 공유 범위. on/off 건수 차이는 기존 observation 전용 2개이다.
- relic_host_layout001: 1/1 PASS, exit0/78.615s/source unchanged=True, source6ea1c1d78b9f495cf72040cf8f61ddf1cd2cceb256f43f7b87cebcf5f107f2cb. raw SHAa8da199e8ca61134781b5e292d0be3cb5189c5521d0299d97f08c956e3db0246. 실제 OperationSummary1072/Committed264/SessionSnapshot352/RingRow216/Detail528/SessionOwner360. 논리 ring cap300, 실제 VecDeque capacity512/backing548864bytes. raw303/retained300/overwrite3. RawRecord를 실제 owner라고 부르지 않는다.
- relic_host_heap001: 1/1 PASS, exit0/10.356s/source unchanged=True, source6ea1c1d78b9f495cf72040cf8f61ddf1cd2cceb256f43f7b87cebcf5f107f2cb. raw SHA32c6f0bea3e94d0078e91a44a36009e479a35eca1c1df671fec12ba6fa7096fe. 원 ignored 단일 thread bounded exporter 검증. baseline1639274/held1658170→1658170/idle1643418, deferredBound3673088/samples1024512. 원 cap/기대값 보존.
- relic_public_a001: test0/compile FAIL, exit101/38.584s/source unchanged=True, source6e145c8153df5c8d817d7cef8d2ef7a4bd737afc70db5f6bfb7489fcf82a27a8. raw SHA528b38932e74e40511759bf7d687d6c3c42f83eec4f9aa000037a3b1c94bd865. 제품 연결 8진단: 출력 전용 DTO Deserialize 요구3, GameDataBase identity accessor4, 고정 action 배열길이1. 후속에서 output variant만 skip_deserializing/actual ValidatedGameData.identity 전달/배열47로 교정했다.
- relic_public_a002: 1 PASS/6 fixture FAIL, exit101/134.518s/source unchanged=True, sourceba271c2b67d4a464f532b95dcd6d07dd8553e23b90f0cc66c16fbd9aaf4e3e09. raw SHAd76df4eae8d47b6c331cfe41233e13d2a56a4f303c0d10e79f6952ee9424657c. 제품 lib compile은 통과했다. 6개는 support.rs의 equip_item에서 기존 training_last_line와 신규 무기의 weapon_archetype_mismatch로 query 전에 거절됐다. 원 support/기대값은 Astra 담당이며 제품 호환 guard를 완화하지 않는다.

초기 noRelic PC는 `evidence/compat_pc_001.json`, gameplay 및 `compat_pc_cost001.json` 원 실행에 결속된다. source409/6ea1c1d78b9f495cf72040cf8f61ddf1cd2cceb256f43f7b87cebcf5f107f2cb에서 576행/36조건/31성공/5거절, immediate U07-M 비시간 exact·허용 delta0 및 17 negative 전부 거절이다. 역사 U06 owner+200 및 requested80 차이는 U08 신규 변화가 아니다. hp6000/log6525/queue25/native[[1063413,6399,8314608]]/retainedEnd1245816 복구와 common source136/capture192를 확인했다. 이 결과는 이후 공개 A source의 성능 재검증이나 전체 U08 완료를 대체하지 않는다.

`relic_public_a_fmt001`은 skip_children=true/명시15 Rust파일 중 8개만 syntax format, unexpected child delta0, raw0/runner99(의도한 mutation)이다. `relic_public_a_fmt002`는 명시4 제품 Rust파일/raw0/변환0이다. source/command/preimages는 각 runner receipt 및 source_versions/command_inputs archive에 보존한다. A002 종료 뒤 owned session0; process 검사는 sandbox namespace 범위이다. B 무료 Event·역사 receipt와 C 명시 판매 거절은 아직 미구현/미실행이다. 정상 수명주기의 기존 reconcile/reset 제품 흐름은 존재하며 해당 U08 공개 경로 검증이 잔여이다.

- relic_public_a_fmt003: tester setup 교정 prospective.rs만 명시한 syntax format/raw0/변환0. 제품 bytes는 A002와 동일하다.
- relic_public_a003: 7/7 PASS, exit0/18.615s/source unchanged=True, source79ed7b5f89ee786507f2276392bdda95f3ea1559dd4d3458605716022f5b3e3a. raw SHAe1fc1c453d214229a9281ef183213f29308a4d20f53191cb3c0eb7935e9dba9b. 외부 RON 정상 등록→CoreApplication wire의 두 직원·실제 provider/form AND·tags·constant/Unresolved·Damage/Heal basic 구별·final stats penalty/certified 보존·baseMAX/final overflow·slot canonical·missing weapon·UUID/RNG/save 등 공개 roots 읽기 불변을 확인했다. 초기 native owner·released carrier·PC 전체를 새롭게 검증한 결과는 아니다. 기존 A001/A002 실패와 원 input은 보존한다.


## 공개 A 이후 공통 회귀 — 2026-10-06/002

- relic_public_a_fmt004: Astra가 작성한 behavior.rs cfg(test) 변경만 명시한 skip_children=true syntax format. 기존 A003 archive와 root의 비교에서 제품 영역 변화0이며 A003의 외부 7/7 증거를 그 범위에 재사용한다. 새 제품 source417/6076ed935e7e99fe26db51bb2bcbc883bdfe6ee63832ef118473d95d41827af1에 후속 공통 회귀를 실제 실행했다.
- relic_numeric_focused005: runtime-observation relic selector 33/33 PASS, exit0/191.870s/source unchanged. raw SHAe4425dcf41716becfcf4f94f2188f4fae7d9254b8b2deb5d3af3023e42eb9439. 기존 literal-off/Expr/basic once/공유 selector/source clone·reinput 공존/원 base·final guard/owner/save/관측 경계가 유지된다.
- relic_numeric_off002: default relic selector 31/31 PASS, exit0/173.445s/source unchanged. raw SHA42ee88c2391b50275e43194aff2b09e168560b9964b96cde146a9168aa6b86c6. on/off 공통 범위는 동일하며 관측 전용 2개만 건수 차이이다.
- relic_behavior001: game::behavior::tests 11/11 PASS, exit0/3.224s/source unchanged. raw SHAcf8b0581fe77168706c82402b95dddcc8a24ece5fb70b4ad8366114e0efcbd50. Astra가 소유한 action74/result54 exact 계약표와 transport/direct payload 검증을 포함한다.

세 실행 receipt의 현재 source는 모두417/6076ed935e7e99fe26db51bb2bcbc883bdfe6ee63832ef118473d95d41827af1이며 종료 후 identity 재측정도 동일하다. owned runner/session0; 다른 process namespace의 전역 부재를 주장하지 않는다. common RuntimeFactSource136/EvalCapture192/cold extension24는 유지된다. runtime_observation.rs의 SessionState384와 game_server observability의 SessionSnapshot352는 서로 다른 타입이며 layout 증감 비교로 사용하지 않는다. B/C 신규 제품 구현과 공개 Event·판매·수명주기 검증은 아직 잔여이다.


## B/C 제품 source-ready — 미실행 경계, 2026-10-06/003

root 공개 계약과 EVENT_OWNER_BOUNDARY_002_CORRECTION에 따라 제품15파일을 작성했다. 실제424/b79ffe05fd14126ffa2647be142c1ecd7afe84f84e55b52367a1d953df11c26d의 source-ready/preimages/diffs는 evidence/product_bc_checkpoint001에 보존한다. 아직 포맷·빌드·시험0이며 성공을 예고하지 않는다. Sol은 기존 cfg(test) 코드/fixture/기대값 변경0이고 mixed-file의 새 literal/enum 보강을 Astra에 인계한다.

- B: typed ChooseRelicCorruption/current source scene, 다중 Option<Vec> history, 같은 active Event의 payload-qualified historical retry, eligible-only 후보/후보0 decline=true/catalyst0. 기존 Advance/Resolve/조회·commit 및 각 capture/Continue 독립 history clone를 사전 admission한다. Someempty/중복scene·선택copy/nilcopy/순서·분기/current path/nonEvent map key·다른Event payload와 모순 check 산술을 historySome에서만 거절한다. 현재 소유된 historic copy는 Corrupt, 합법 분해 뒤 부재는 허용한다.
- 실제 RunState와 RunCheckpointPayload의 event_sessions는 state.rs의 OwnedHashMap as HashMap alias이다. 앞선 raw HashMap 추론은 잘못된 읽기이며 root 정정 문서를 인수했다. 새 native retained membership은 각 receipt Vec backing/source_scene 문자열이고 기존 table backing 확대나 migration은 없다. 기존 OwnedHashMap closed-history/CloneShape 순회가 새 leaf를 포함한다. EventSession inline 필드/layout 변화는 별도 실제 측정과 allocator/R05 영향 검증 대상이다.
- First decline부터 typed domain native scope를 활성화하고 active/run-map/checkpoint/entry/Continue의 독립 추가 payload를 각각 합산한다. 공유 Continue/SavePoint Arc는 handle마다 deep-count하지 않는다. presence-only Continue hint가 decline history를 typed decode 전에 감지한다. 기존 World rollback capture는 기존 outside-native 범위다.
- RetreatBattle의 기존 ContinuePayload transient는 current relic/UUID/history를 복사 전에 preflight하며 후속 install destination과 별개다. prospective query는 stable phase의 기존 read-only Continue 재작성 제외 경계에 연결했다.
- C: qty0/NotInShop/SellingUnavailable 기존 우선순위 뒤 sell-enabled shop의 known relic qty1은 RelicNotSellable이다. derived sorted not_sellable_copy_uuids는 empty skip이고 저장 authority/구매 풀을 만들지 않는다.

A003의 이전7/7은 과거 A 경계 증거로 보존한다. 위 query read-only 연결과 B/C 제품 delta 이후 공개13 및 query owner2를 새 source에서 검증해야 한다. Event owner·restore·layout/lifecycle 및 최종 기능/성능 검증은 모두 잔여이다.


## B/C 첫 공개 실행과 owner 컴파일 경계 — 2026-10-06/004

- relic_public_bc_fmt001: source424/fecd7733f28338549ad3d92cbaaca6f4ffa2e002dc8826d64e907e528e0088ab에서 명시25 Rust파일만 skip_children=true 문법 포맷했다. 실제22파일 변경, 목록 외 변경0/membership delta0, raw0/2.684s, runner99는 사전 승인한 포맷 mutation이다. FORMAT_SCOPE/FORMAT_DELTA와 exact source before/after는 evidence/relic_public_bc_format_scope001 및 runner receipt에 보존한다. 포맷 후 source424/6005757cd6804fc7fb9fb6b6bd76516e00dc9aebafb7314a2941d4a4c5d11e58.
- relic_public_bc001: default 외부 u08_relic_public target 13/13 PASS, exit0/123.618s(시험9.14s), 위 source6005757c before/after unchanged. receipt SHA6bba3f39c433398d3bf42623886fdb0bd58f755985c34f29797831f0897e90fa/raw SHAcf8ff62050bbec15d81c2fca2decb71caa4fe9113bd62d728bab9f43b03a8f0c. 현재 A7/B3/C3를 실제 정상 외부 RON·CoreApplication wire에서 다시 확인했다. B는 후보0 decline·정확 historical retry, 정상 reward와 여러 무료 선택, 잘못된 권한과 committed check 분기를; C는 derived empty omission, 기존 판매 policy 우선순위, normal/corrupt와 장착 조합의 특유 판매 거절을 확인했다. 기존 A003을 현재 제품 성공으로 재명명하지 않는다.
- relic_query_owner001: libtest compile1오류/test0, exit101/40.859s/source6005757c unchanged. receipt SHAd23fb182ecb696609429ed6fc11911fa65d756afb66ed920c825b7ced9d8239a/raw SHA15f4bfd186dd798f7f73801ef9c418647d6a843dd3e89594307bc1c613838f2b. u08_event_owner.rs의 ApplicationCheckpoint::new(Vec<u8>) fixture가 Arc<[u8]>를 전달한 E0308이며 Astra에 테스트 교정을 반환했다. 공개 constructor 제품 계약을 바꾸지 않는다. query2/Event owner4는 실행 전이므로 PASS로 세지 않는다. 후속 event_owner001은 시작하지 않았다.

제품 신규 redundant CloneShape import 경고는 selection.rs 자체 trait impl의 한 줄이며 root의 분리 승인으로 제거했다. evidence/product_bc_cleanup001에 실제 before/after/patch를 보존했고 의미 변경이나 포맷·빌드 실행은 없다. 테스트 담당자가 별도 파일을 교정하는 동안 전체 source identity를 새로 확정하지 않는다. 기존 cfg(test)/fixture/oracle를 Sol이 교정하지 않았고, query001 실패와 기존 모든 결과를 보존한다. 정상 수명주기·restore/owner/fault/layout 및 현재 최종 통합·PC/R05는 잔여이다.


## B/C owner 실제 실패 분류 — 2026-10-06/005

- relic_public_bc_fmt002: 명시 selection.rs/u08_event_owner.rs 두 파일만 skip_children=true 포맷, 실제 변환0/목록외0/membership0, raw0/2.014s. 이전 공개13에 대한 두 한 줄의 의미 비영향을 root가 확인했다. 소스424/31b4b0150dd181e43e49ae177c6ccfdfed3027f96dbddfde51e3ece2a805667d.
- relic_query_owner002: 위 소스 unchanged, exit101/140.686s, 0 PASS/2 FAIL. raw SHAf1593941b84bb599e20d3da36ee46e766a8f58b99868f4dd7477bd70dd209091/receipt SHA f24dd4e3dd158c9e56a04005493ca7a3bee00788814860022cafb2b1543a485a. 정상 공개query의 누적 admission은 targets296302→296686, effects→296789, penalties→296903, generation→296967이다. 실패1은 lowwork에서 clone 전 relic_owner_extent가 첫 거절인데 fixture가 fact_clone_scan_and_copy를 기대했다. 실패2는 외부 private native::run 안쪽의 ApplicationError를 최종 공개 오류로 검사했다. application.rs check_projection_control의 기존 sticky sentinel(ProjectionFailure/emptycode/Null)은 outertyped firstfault로 폐기되며 기존 tests도 이 경계를 확인한다. root·Astra·Sol은 이 두 실패를 외부 budget/oracle 경계 문제로 분류했고 제품 오류 mapping 변경 근거는 없다. 아직 새 시험 교정/재실행 PASS는 없다.
- relic_event_owner001: 같은 소스 unchanged, exit101/10.144s, 0 PASS/4 FAIL. raw SHA33e78bf571eb3da04512022f5f2120b598fba41d1b1117be1e5c51cba2b081ab/receipt SHA39de4dc07abe0d70fb0ae29f2832f3497ea55ac9ca85f6b438375bfa8292a217. Event 정상 End가 node_flow::commit_node_completed_with_recovery의 기존 무조건 event_sessions.remove를 통과해 새 history를 제거했다. postcompletion Continue history Null 및 decline owner false에서 멈췄으므로 strict restore 반례나 layout print를 통과했다고 세지 않는다. decline 초기 admission은 session_clone0→4260/append→4319/history_projection→4372/session_clone→13065의 부분 관측이다.

모든 runner는 종료했고 이전 raw/receipt/source archive를 보존했다. 입력 writer HOLD 상태에서 현재 floor의 historySome session 보존/legacyNone 제거 유지 및 기존 floor/newrun/terminal 해제 보존을 root에 제품 수명 교정안으로 반환했다. 제안과 실제 승인·구현·검증을 구분하며 이 기록 시점에는 제품 교정 실행0이다. Event map은 기존 OwnedHashMap이며 새 전체 table owner나 전run 역사ledger를 자동 추가하지 않는다.


## 완료 Event 수명 교정과 query green — 2026-10-06/006

root EVENT_LIFETIME_DECISION_001.ko.md(SHA9736b3189b5f652ece2d714d638c86bfea286a5dca7c6fbf4fdae74a95737ebc)에 따라 현재 floor 일반 Event completion에서 historySome session만 보존했다. historyNone의 기존 remove와 다음 floor/terminal/newrun 생성 코드는 바꾸지 않았다. 제품 단일 node_flow.rs actualpreimage/diff/after는 evidence/relic_event_lifetime_correction001이며 cfg(test)/fixture/기대값 변경0이다. 전run 새 역사 ledger를 추가하지 않았고 기존 map/node/Event/DAG restore 결속을 유지한다.

- relic_public_bc_fmt003: node_flow.rs/u08_query_owner.rs 두 명시파일만 문법 포맷, raw0/2.677s, query시험1파일변환·목록외0/membership0. source424/f23454f980647a703c58716eb99a52b177a0e271e77f9dd62efa8bea5709ba12 고정.
- relic_event_owner002: exit101/78.826s/source unchanged, 3 PASS/1 FAIL. receipt SHA15fb23056a02707a3d5d176ae5adf8791c46f3db2365c3317d949cdafd7dd1b4/raw SHAd58e8e0fea63fe8be5db822007a40639b92fab9f08f64d915969192c21f924a6. completed history positive Continue와 strict restore10cases, EmployeeCheck branch3cases, decline-only active/run/checkpoint/entry/Continue 독립 owner·append one-short 및 predecode admission을 통과했다. actual size_of: EventSessionState296/ActiveNodeContent640/EventSceneSnapshotDto288/CommittedRelicCorruptionSelection48. 값은 현재 실제 타입이며 원 baseline layout 증감과 동일하지 않다. 마지막 합법 분해 후 Continue는 시험 Maintenance map slot admin_fixture가 authored standard_intro_a 밖이라는 기존 strict map guard에서 실패했다. 이 실패도 원본 보존하며 무효 map을 허용하는 제품 완화는 하지 않는다.
- relic_query_owner003: 3/3 PASS, exit0/97.863s/source unchanged. receipt SHA3345be7cfb8552b17bc80a9e4021452e1438a63aef41d86e8d5bfb7e3580ba40/raw SHA184e067b33728d3658667ba17918acb1a057596e92218afc5ad3118d6128e480. Astra가 private outer scope sentinel/firstfault와 정상 독립 host settings 공개 오류를 분리했다. 정상 공개 typed fault의 kind/code/site/cap/requested·roots 불변·정상 저장 복원/재시도, work/clone 한도 및 targets/effects/penalties/generation retained exact/one-short를 검증했다. 관측된 query cumulative final296967은 운영8MiB 초과나 process heap 수치가 아니다. 제품 오류 mapping을 바꾸지 않았다.

두 runner 종료/owned session0와 source HOLD를 회수했다. 모든 이전 실패·역할 기여·raw/input을 보존한다. 잔여 Event fixture1과 현재 completed-retention 제품 delta의 공개13 재검증, 정상 수명주기·released actual battle 및 전체 기능/최종 성능·동결·독립 감사는 다음 경계다.


## B/C 집중 경계 현재 green — 2026-10-06/007

- relic_public_bc_fmt004: u08_event_owner.rs 한 명시파일만 syntax format, raw0/2.254s/목록외0/membership0. Astra의 합법 분해 fixture는 history 생성 전에 Maintenance 지원 authored map을 준비하고 같은 map의 정상 Select/Confirm/Complete 경로를 사용했다. 기존 restore guard와 기대값을 바꾸지 않았다. 실행 소스424/2cfcdfe0555274844b85382ac1672c3315b3ca71c0f2701ba24271e64fef26ca.
- relic_event_owner003: 4/4 PASS, exit0/54.300s/source unchanged. receipt SHA3fccea29cf849deee05326db3698866548eb18a8254072bc215bd50aab42d832/raw SHA140218f73873b7646e291d46df231abaf9ea91b2aea7b492cccd2de50b73f49c. 완료 Event history positive restore, strict mutation10+branch3 반례, decline-only 독립 active/run/checkpoint/entry/Continue/DTO owner와 append 한도, 선택된 copy를 이후 정상 Maintenance에서 합법 분해한 뒤 absent historical copy의 공개 Continue를 확인했다. 현재 typed layout296/640/288/48이다.
- relic_public_bc002: 외부 공개13/13 PASS, exit0/68.361s/source unchanged. receipt SHAee9d9d2c15264cf24a4128d792d11f019503ee22b4daa1c6d2a9b948907d3d10/raw SHA1e6f3009bee200747fbe27996131e6152cde9ca5fa095686854604f8ca0252a9. completed history retention의 제품 변경 뒤 A7/B3/C3를 current source에서 다시 실행했다. 이전 공개13을 단순 재명명하지 않았다.

query_owner003의3/3은 root의 명시 인수에 따라 재사용한다. 해당 source424/f23454f9와 위 current2cfcdfe의 전체 source manifest 차이는 u08_event_owner.rs의 fixture-only 한 파일뿐이며 제품·query 시험·limits/input bytes는 동일하다. 이 실제 delta/reuse 결속을 docs_checkpoint010/RECORD.json에 남겼다. 현재 공개13/Event4/query3의20 집중 경계가 green이며, 모든 이전 compile/assert 실패와 실제 제품 교정·Astra fixture 교정을 구분 보존한다. 정상 D lifecycle·E released queued/projectile/area의 정상 공개 소비와 full functional/public/numeric/fault 및 현재 PC/R05·동결·독립 감사는 미완료다.


## D 수명주기 첫 공개 검증 — 2026-10-06/008

- relic_public_d_fmt001: Astra D stage1 source-ready를 인수한 뒤 lifecycle.rs와 외부 u08_relic_public.rs 두 Rust 파일만 skip_children=true 문법 포맷했다. 실제 lifecycle.rs 한 파일 변경, 목록외0/membership delta0, raw0/1.992s이며 runner99는 사전 승인된 syntax mutation이다. source427/3d40764c6f69f23e6f83cd7bc523d385482409a0f897a8dfea24e9eee6fdf7e1에서 source427/ad76cd8138b96630bb45f47784bad2df9a9cab36608213a3df6c7303b1696756으로 이동했고 실제 pre/post/diff는 evidence/relic_public_d_format_scope001에 보존한다.
- relic_public_d001: 외부 lifecycle:: 필터3/3 PASS, exit0/20.609s(시험11.41s), 위 sourcead76 before/after unchanged. receipt SHAe188ac074769eec068771c055e40cd2c9d529186c79e0924cb21dd6938a20e50/raw SHA11f7086c9309053c43684270582eb76f11c9d177840528366b425d4c5d1f9a13. normal/corrupt exact copy의 직원 간 정상 이전과 기여 이전, active StartNewGame 거절 뒤 AbandonRun→NotStarted→StartNewGame의 사본/슬롯/catalyst/equipment dust 초기화, 실제 authored map Request/Select/Confirm Event의 unfinished Continue rewind 및 진입 전 유료 corruption 비용 보존을 확인했다. 완료 후 history 보존/Continue도 같은 정상 경로에서 확인했다.

이전 public_bc00213/13과 D 실행 source의 전체 manifest 차이는 D 전용 RON2/lifecycle.rs 신규3 및 외부 test root 등록 한 파일뿐이다. 기존 public001/support와 제품 bytes는 동일하다. root의 명시 인수에 따라 이전13은 이 비영향 delta에 연결하여 재사용하고 별도 public16을 지금 중복 실행하지 않았다. 실제 source maps/추가·변경 목록과 receipt hashes는 docs_checkpoint012/RECORD.json에 결속한다. full current 통합에는 전체16을 포함할 예정이다.

D001 종료 때 owned session0이며 검증 source는427/ad76이다. 이후 root가 Astra stage2의 별도 시험 파일 쓰기를 허용했으므로 이 값은 마지막 검증 경계이고 이후 실시간 identity라고 주장하지 않는다. 제품/format/build/run은 HOLD이며 테스트 작성·fixture·oracle 교정은 Astra 전담이다. 실제 death/withdraw/redeploy 및 E released queued/projectile/area, current full 기능·PC/R05·최종화·독립 감사는 아직 잔여이다.


## D 내부 Stage2 실패 이력과 역할 인계 — 2026-10-06/009

Astra가 작성한 u08_lifecycle.rs 두 시험을 Sol이 기존 합의된 순서로 실행했다. 아래 원 raw/receipt/source archive와 앞선 작성·교정 이력을 모두 보존하며, 실패 run의 부분 도달을 전체 PASS로 재명명하지 않는다.

| 실행 | 실제 결과 | 첫 실패와 분류 |
|---|---|---|
| relic_lifecycle001 | test0/compile E0308, exit101/17.337s, source428/85a510b3 unchanged | setup의 Arc<GameDataBase>를 ValidatedGameData용 builder에 전달한 시험 타입 연결 오류. 기존 GameCore fixture adapter로 Astra가 교정했다. |
| relic_lifecycle002 | 0 PASS/2 FAIL, exit101/65.232s, source428/3580e6af unchanged | 첫 deploy에서 PositionOccupied. Confirm 이후 실제 allied logical slot occupant None인 Ground 후보를 선택하도록 Astra가 교정했다. enemy overlap은 기존 계약에서 허용하며 앞선 enemy 충돌 추측은 철회한다. |
| relic_lifecycle003 | 1 PASS/1 FAIL, exit101/53.240s, source428/c69fd95b unchanged | living withdraw/쿨다운 거절/실제 redeploy는 PASS. death는 사망 확인 전 active_battle None인 terminal에 도달했다. 당시 raw만으로 verdict/target 원인은 미확정이다. 후속 보존된 별도 actual trace에서50000ms Victory/enemy 공격0/실제 guardHP100을 확인했고 fixed-route melee 입력을 lethal분기 Ranged+LowDefenseFirst 및 prebattle captured guard stats1m으로 Astra가 교정했다. |
| relic_lifecycle004 | 1 PASS/1 FAIL, exit101/54.728s, source428/b90d9aff unchanged | living PASS. death는 실제 enemy lethal damage/HP0/physical death/전투 중 persistent roster 및 슬롯 보존과 CombatResult 상태에 도달한 뒤, active_node_content 조회 None에서 실패했다. 이후 result/recovery/acknowledgement 기대는 아직 미검증이다. |

fmt001은 명시 두 Rust파일 중 u08_lifecycle.rs만 문법 포맷했고 목록외0/membership0/raw0이며 runner99는 승인된 syntax mutation이다. fmt002/003/004는 같은 파일의 --check만 실행하여 raw0/source unchanged, 추가 mutation0이다. Sol의 fixture/기대값 교정은0이며 세 교정의 actual preimages/diffs/원 진단 trace는 각각 test_internal_lifecycle_compile_fix001/occupancy_fix001/enemy_fix001에 있다. 003 actual trace SHA5fa42cc1b45e4f8e25fd685ba40584d46b3683c814220c1b518993fbe8be0c77은 최초 raw와 별도 증거이며, 당시 원인 판정을 소급 확정하지 않는다.

정산 시점의 전제를 정정한다. combat.rs::transition_finished_live_battle_to_result는 terminal에서 settle_combat_result를 실행한 뒤 이미 정산된 summary/acknowledgement/continuation phase를 confirmed_combat_result에 설치한다. application.rs 후처리는 reconcile_relic_loadouts를 수행한다. 사망 정산과 슬롯 회수는 이 terminal→settlement→application commit 경계이며, CompleteCombatResult는 receipt acknowledgement를 반환하는 단계다. 앞서 Complete를 최초 정산 commit으로 보았던 root/구현 조사 전제는 잘못이었다. 이는 기존 제품 소비·시험 기대 시점의 정정이며 제품 정책/owner 변경 근거가 아니다. 004의 active_node_content와 settled receipt owner 혼동도 시험 담당자가 교정할 대상이다.

004 정상 종료 뒤 owned session0를 회수했다. 새 사용자 역할 지시에 따라 Astra는 시험 묶음의 scoped 포맷·작은 compile/focused 실행·일반 fixture 교정을 직접 반복하고, Sol은 기존 지정 GPT-6.1 Sol/xHigh 제품·제품 문서·전체 통합·회귀·성능을 유지한다. 이후 작은 실행을 Sol이 중복 시작하지 않는다. PLAN/EXPERIMENT_NOTES는 root 소유이며 여기서는 입력외 EXPS/STATE만 갱신했다. 현재 제품·통합·성능 HOLD이고 마지막 실행 경계428/b90d9aff는 후속 시험 writer 활성 이후 실시간 identity라고 주장하지 않는다. D stage2 closure·E actual carriers·전체 기능·현재 PC/R05·독립 감사는 잔여이다.


## E003/E004 제품 교정과 실제 입력 분석 — 2026-10-06/014

E003 최초 정상 combat reward 준비 거절은 mission_policy 허용kind 누락으로 확인하여 Relic/RelicCorruptionCatalyst 두 행만 연결하고 product_combat_reward_kinds_fix001에 보존했다. E004는 정상 reward/equip/savepoint/Event/load-replay를 지나 fresh-process Continue에서 retained8MiB/requested11527716/relic_continue_value로 거절됐다. 원본을 성공으로 표시하지 않는다. 실제 checkpoint49775bytes와 outer save145444bytes를 구별하며 같은 op의 decode/checksum Value/post-restore Continue 재생성 누적 비용이다.

product_continue_allocation_fix001은 typed decode·Value storage와 실제 named-array normalization sort workspace의 admission을 분리한 두파일 국소 교정이다. generic profile/owned projection, encoded Vec/Arc, VERSION11/checksum, cap/누적/무환급은 유지했다. 원 입력의 두 정렬 대상 len0이며 모델 세 shape 합11,411,012→7,789,876은 실제 새로운 admission/heap/성공 측정이 아니다. 고유 원본·patch·source manifest·dependency 근거·model/raw 구분은 해당 기록에 결속한다. relic_continue_allocation_fmt001 raw0/3.287684s; wrapper99는 의도한 한 파일 syntax formatting이고 목록밖변경0/membership0/cfgtest0다. 현재 source434/758179d74cf0bec51270481173856c0ee378d8ee78dcafce9957a60544e96ce1은 포맷 후 검증경계이며 빌드·기능·성능은 미실행이다. Sol writer/ownedrunner0로 HOLD 후 Astra가 같은 원 입력 actualchild와 allocation/durable/Event owner·nonempty normalization 회귀를 검증한다.


## E1·Continue 완료 경계와 carrier 교정 후 실제 검증 — 2026-10-06/015

이 묶음은 root public009/010과 실제 종료 receipt/raw를 인수한 입력외 문서 갱신이다. 아래 source는 각 실행 당시 고정된 검증 경계이며 후속 Astra 시험 작성/회귀/basic 단계의 실시간 source나 전체 완료를 뜻하지 않는다. 시험 코드·fixture·oracle와 작은 포맷/집중 실행은 Astra/high, 두 제품 파일 교정과 이 문서는 기존 Sol6.1/xHigh가 소유했다. 이전 Sol/Astra 기여 이력과 실패 원본은 그대로다. 상세 receipt/source/건수·시간·hash는 `evidence/docs_checkpoint015/BOUND_RECEIPTS.json`에 결속한다.

- `relic_public_e009`: source435/984f8e93da698a55baa2f4c41f53fd9934dc9f3a884e4e522669c552f1bbf53b unchanged, exit0/54.006744초, 1/1 PASS. 정상 외부 RON 지급/장착→SavePoint→무료 Event select+decline→Load rollback/replay와 fresh OS child Continue를 확인했다. 실제 t8001 대상 HP422→212, requested/raw/calculated210 한 True segment의 비클램프 양성이며 원 Original source·실제 fragment copy/form/직원/generation·Corrupt relic 기여가 유지된다. trace false/true의 DamageFact와 immutable preview/explain은 동일하다. E007의 HP clamp 양성과 E008의 대상 부재 실패는 이 success로 소급 재명명하지 않는다. 두 실행 모두 amount_observation=true/Snapshot Const100이므로 literal-off·Live·carrier 근거는 뒤의 E2와 구분한다.
- `evidence/public_trial_cli009`: export generation3dc45a9376e9e85f3d42a4760758fecb2cf84ed5fdf4bef4d94967eb6786bff6, manifest SHA037537ae0f221da0c29fb541fc07ba3d2165586de239b1b84e42d3c6a11a9f26의35개 member를 크기/SHA로 다시 대조했다. 준비된 공개 release CLI 입력이며 실제 release binary/current build로 CLI를 소비한 결과는 아직 없다. 별도 OS childPID319/parent317의 Continue1PASS/2.79초는 CoreApplication 증거이고 CLI simulate의 Continue 실행으로 표시하지 않는다.
- `relic_continue_owner001`: source435/cc33acfd2a9e3419023ae0205efec23f67690cbc5b9ce4f8989b9a39b319d101 unchanged, exit0/69.728216초, 2/2 PASS. current/독립 SavePoint 각각 두 원소인 checkpoint-rewind VN 및 purchased-merchandise set의 순열에서 checksum/전체 export/public roots가 유지되고 실제 normalization8회가 양수였다. 누적 exact cap4,518,759, Value stage indices0/2/15 및 SavePointArc를 포함한 선택6경계 one-short refusal의 typed outer fault·atomic roots/input 불변을 확인했다. 이 admission 값은 실제 allocator/RSS 충분성 측정이 아니다.
- 관련 회귀: allocation001 2/2(source434/758179d7…/4.667739초), durable001 2/2(11.999887초), UUID001 1/1(7.616683초), Event001 4/4(8.558688초; 후자3개 source435/cc33acfd…), application-owner001 1/1(source435/0fda9545…/46.073595초) 모두 exit0/source unchanged. 마지막 시험은 원 선언75의 owner 계약표이며 제품 routing 변경이 아니다. 각 과거 입력 경계를 현재 전체 통합 성공으로 세지 않는다.

`relic_carrier_stage2001`은 source438/0f7ca264…에서 literal1PASS/carrier4FAIL/exit101 원본이다. Snapshot projectile/GroundFixed/Follow는210 대신100, Live 생존 양성은78 대신12였다. release 결속/consume 적용은 관측 gate 밖이며 실제 원인은 `ValueExprDeliveryCapture::rebind_consumption_source`가 incoming+authored만 복사해 각 원 capture의 numeric을 잃은 제품 누락이다. Sol의 `product_carrier_numeric_rebind_fix001`은 승인된 value_expr_runtime.rs/runtime_facts.rs 두 파일만 바꿨다. incoming action/new delivery/provider/independent/reinput은 보존하고 각 capture의 exact numeric Some/None로 합성한다. 새 wrapper는 사전 admission, Arc binding/plan/frame 공유와 합성 retained cache 재계산, None/None 기존 clone·비용, original 권한·cap·누적 무환급은 유지한다. 두 파일 외/fixture/cfg/실제 world5 bytes delta0, 포맷 raw0/2.153883초/포맷 자체 delta0이다. 제품 교정 경계438/bc2f0de2…와 아래 시험 후 경계438/939a61e1…는 서로 다른 manifest다.

Astra private001/002는 fixture 준비의 StaleOccurrence 및 OptionNone unwrap으로 각각0/2 실패했고003은 RuntimeResources Box type 연결 compile 실패/test0였다. 모든 원본을 보존했다. `relic_rebind_private004`는 source438/939a61e13b381c034c5ecf1efcb80a79091038592e46df090107aa60ddbf47a4 unchanged, exit0/50.689448초, 2/2 PASS다. mixed selector captureSome/None와 incomingSome/None, 다른 independent/reinput의 exact Arc/authored/current action·delivery와 원본 불변을 검증했다. 실제 전체 rebind admission은 incoming numeric없음12,970→wrapper13,058, 있음14,438→numeric wrapper14,526→cleared-row wrapper14,614이다. 두 경우 exact cap 성공/전체 사전 bound 및 wrapper one-short typed refusal/atomic 입력을 확인한 private 경계이며 heap 또는 normal scheduler 증명으로 확대하지 않는다.

같은939a61e1 source의 `relic_carriers_stage2002`는 exit0/9.163528초, 기존 world5/5 PASS다. 내부 Admin 준비 이후 정상 ActivateSkill/Withdraw/실제 scheduler·runtime facts 소비이며 E1 정상 획득과 역할을 구분한다. 관측 off literal 두 직원144/210와 서로 다른 원 fragment/relic binding, released Snapshot projectile의 t1001withdraw→t2093impact210 및 unreleased 지연 step 취소, GroundFixed의withdraw 뒤 실제3 tick의 동일210 binding, Follow의 초기210 양성 뒤withdraw로 ticks/pending 취소를 확인했다. Live Source.Attack 생존78 양성 및withdraw 뒤 consume의 MissingContext/failed bucket roots 원복도 통과했다. 이미 commit된 철수는 원복하지 않는다.

U08은 GEX10 partial이고 whole-goal 독립 감사는 아직 없다. 이어지는 관련 selector/resource-lifetime 회귀·실제 basic auto/triggered/windup 단계, 공개 release CLI 소비, 전체 기능/공개/save/fault/numeric guards 및 current PC/R05, evidence→impact→adaptation→coverage→자체검증→정확 동결/복원→비참여 독립 감사는 잔여다. 초기 noRelic PC31success/5refusal과17negative는 이전409/6ea1 경계 그대로이며 현재 전체 source의 최종 성능 성공으로 쓰지 않는다. 이번 문서 묶음은 포맷/빌드/시험/성능 runner0/제품·시험·도구 쓰기0이고 PLAN/NOTES는 root 소유 그대로다.


## carrier 교정 관련 회귀의 후속 인수 — 2026-10-06/016

root public011과 종료 receipt/raw의 실제 bytes를 docs_checkpoint016에서 대조했다. 이전 docs_checkpoint015의 모든 원본/preimage/기록을 보존한다. Astra의 `relic_rebind_related001`은12/12 PASS/exit0/10.591570초, 기존 resource-lifetime의 `relic_rebind_resource_projectile002`는1/1 PASS/exit0/7.382705초, `relic_rebind_resource_fixed001`은1/1 PASS/exit0/10.254429초다. 세 실행은 private0042/2 및 world stage2002 5/5와 같은 source438/939a61e13b381c034c5ecf1efcb80a79091038592e46df090107aa60ddbf47a4에서 source unchanged다. exact label/source/raw/receipt·건수는 이 checkpoint의 BOUND_RECEIPTS에 있다. 앞선 projectile001은 selector의 실제 combat module을 빠뜨려0시험/exit0이며 기능PASS로 계산하지 않는다. 원본은 유지하고002를 실제1시험의 성공으로 인수한다.

private 합성 fixture는 incoming provider를 의도적으로 바꿔 exact 보존을 확인하므로, private2의 결과를 consume의 유효 provider 권한으로 주장하지 않는다. validate_skill 원 guard는 그대로이며 정상 권한 소비의 증거는 별도 실제 world5다. root가 전체 manifest를 대조해 Sol 제품 handoff438/bc2f0de2… 뒤939a61e1…까지의 추가 입력은 Astra relic_amount.rs 시험 파일 하나뿐이고 실패 때 world5 입력·기대값은 그대로임을 확인했다. related12/resource2는 이 국소 교정 영향 회귀이며 whole-goal/native heap/최종 PC/R05를 대체하지 않는다.

현재 Astra가 기본공격 stage의 승인된3파일을 작성하므로939a61e1은 마지막 closed receipt 경계일 뿐 live source가 아니다. 제품/전체 통합·성능은 HOLD, Sol의 이번 작업은 입력외 문서2개뿐이며 모든 실행0이다. U08은 GEX10 partial/전체 독립 감사 전이다.


## 기본공격·epoch와 carrier 전체 실제 회수 — 2026-10-06/017

root public012의 RECORD.ko.md/RECEIPTS.json과 실제7개 receipt/raw hash·result행을 docs_checkpoint017에 결속했다. 시험·RON 작성과 집중 실행은 Astra/high, 이 문서 인수는 기존 Sol6.1/xHigh다. 이전 Sol 시험 작성 이력, 실패 raw와 제품 교정의 provenance는 그대로 보존한다.

- relic_carriers_basic001: parameterized1/1 PASS, exit0/65.887332s/source438/60b701c09e8dda7f326eda66a70edf177961a0727214e030d49a7a617db46576 unchanged. 근접/투사체 × Normal/Corrupt 네 경우의 Auto와 authored ExtraAttack에서 실제 base12→38/78을 한번 합성하고 raw source_attack/unit stat12, 별도 BasicAttackDamage 주소와 basic profile, fragment selector 배제, observation-off를 확인했다. 이 수치는 당시 실제 시험 결과이며 prospective 추정값이나 새 oracle가 아니다.
- relic_carriers_epoch001–004: 각각0PASS/1FAIL/exit101/source unchanged, 원본 보존. SelfUnit Stun의 정상 hostile-effect usefulness/no_valid_target 준비 전제가 성립하지 않아 epoch 취소 수용 조건까지 도달하지 못했다. 정상 target/channel 경로의 fixture 교정으로 이어갔고 제품 epoch 결함이나 기능PASS로 재분류하지 않는다.
- relic_carriers_epoch005: 1/1 PASS, exit0/54.402386s/source438/0df25480ef7cb5c787f37488f85c9c5d6069b156637132a19c703343d6d15d25 unchanged. 정상 skill command로 ChannelAura를 시작해 attack epoch0→1, actor active와 원 Auto queue 보존을 확인했다. Expired1601ms 뒤 채널은 없고 원2500ms 공격은 queue에 남아 있다. 2500ms 처리군 damage count1→1/old AttackResolve 없음, cast 전 복제 대조군1→2/같은 시각 비클램프 피해38이다. queue/epoch 직접 주입이나 제품 공개 API 변경은 없다.
- relic_carriers_complete001: 7/7 PASS, exit0/12.953149s/위 source438/0df25480 unchanged. 실제 carrier5 + 기본공격 parameterized1 + epoch1을 함께 재실행했다. 별도 epoch005를 최종7개 밖의 추가 조건으로 중복 집계하지 않는다.

root가 인수한 stage2002/source939a61e1 이후 delta는 u08_carriers.rs·carrier001/skills.ron·fragments.ron의 시험/RON3파일뿐이고 제품은 바뀌지 않았다. 이는 public012의 닫힌 source 경계에 대한 인수이며 공유 작업 디렉터리의 실시간 전체 identity를 새로 검증한 주장과 구분한다. 최종7개의 raw/receipt가 입력 unchanged임을 확인했으나 전체 기능/공개/비용 완료로 확대하지 않는다.

현재 Astra는 U08 통합/public/release/R05 도구와 CLI009 comparator를 작성 중이다. Sol은 기존 integrated/release 명령과 환경을 읽기만 했고 EXECUTION_PREFLIGHT_READONLY.md에 결속했다. Core/host feature별 독립 실행과 observation/measurement ignored heap 단일 thread, 기존 clippy allow2, 모든 공개 binary 소비 앞의 fresh release build/source-binary hash, 기능 worker 종료 뒤 PC→strictcost→fresh R05 순차 경계다. 이 문서 checkpoint에서 제품/시험/도구 쓰기·fmt/build/test/performance 실행은0이다. 도구 인수와 모든 입력 writer HOLD 및 별도 실행 전환 전에는 실행하지 않는다.

U08 GEX10은 partial이다. release CLI009는 아직 준비 입력이며 최종 통합/public/API/save/object/fault/numeric guards·현재 PC576/36/31success5refusal/R05 원 cap·peak-only 경고 확인과 evidence→impact→adaptation→coverage207+31→자체검증→정확 동결/복원→비참여 전체 독립 감사가 남았다. default8MiB·누적 무환급·원 workload/oracle를 유지하고20분 stress는 제외한다. native exact-cap은 별도 process heap/RSS 상계의 증거가 아니다. U09는 시작하지 않는다.


## 최종 통합 착수·stack 회귀와 국소 제품 교정 — 2026-10-06/018

Astra verify_harness_handoff001의 MANIFEST/source438+45tool/input hash와 EXECUTION_HANDOFF_002를 Sol이 실제 인수해 integrated_matrix002를 시작했다. fmt check002가 game/mod.rs module 및 runtime_observation.rs import 순서의 두 제품 파일 차이로 exit1/4.780919s, parent0028.530065s 종료했다. 시험0/source0df25480 unchanged이며 원본을 보존한다. 승인된 scoped skip_children formatter만으로 raw0/2.153548s(의도된 source변환 wrapper99), 두파일 정렬 외 전체membership/외부delta0·시험bytes/45toolhash 불변을 확인했다. product_integrated_format001과 닫힌 source438/fa734235cef0c4b9ab73c9acac1785d0cdc9e9dd6123d70ff36efd0fe42b18a0에 결속한다. 기록 검증기의 줄 multiset·없는 cfg marker 가정 실패는 CHECKPOINT_ATTEMPT001/002에 보존했고 정확 reorder 재구성으로 검증했다.

integrated_fmt003은 exit0/source unchanged였다. 이어 core_default003은1872시험을 시작했으나 기존 authoring::trial::tests::r07_copies::r07_f07_f08_f09_public_ron_recipe_forms_sources_resources_and_fresh_process에서 stack overflow/SIGABRT로 exit101/182.216450s, parent003193.109662s 종료했다. printed ok157줄은 부분 관측이고 suite summary가 없어 전체 PASS가 아니다. verify_remaining003과 성능은 시작하지 않았다. 동일 TEST_DEBUG0/DEV_DEBUG0/INCREMENTAL0·기본 stack·exact1thread diagnostic001도5.843204s/exit101로 재현했다. LLDB diagnostic001은 debuggee 실행 전 helper handshake ack 오류로3.305302s/exit1, 동적 backtrace를 얻지 못했다. 이를 권한거절로 단정하거나 승격·우회하지 않았다.

R07 시험20,456bytes/SHAa86d9472aa321dba6fea3795b379d8d427591c845419647b8aa4cdd56314062f는 U07-M 해당 성공 receipt와 동일하고 기록된 CARGO 환경/명령도 같다. 현재 RUST_MIN_STACK는 unset이나 과거 receipt에 없으므로 과거 unset까지 단정하지 않는다. 큰 함수의 baseline/session과 fresh child branch를 읽었고 현재 raw는 전체 부모 시험 binary abort다. child 결과는 capture/status assertion으로 처리되므로 단순 child 실패만으로 부모 SIGABRT를 설명하지 않는다. 마지막 동적 overflow frame은 여전히 미확인이다.

실제 binary의 prepare_application_internal은129880byte frame을 예약하고 run_deferred/run closure를 통해 같은 함수로 다시 들어갔다. root가 world/application.rs 한 제품 파일에서 scope 선택 wrapper와 scoped body를 나누는 내부 교정을 승인했다. 실행 body/cfg(test) exact bytes, 다른437파일·membership과45toolhash를 유지했다. has_operation_scope/limits/profile/warning/deferred/run 순서·FnOnce1회·fault/rollback/charge/admission·반환/ownerlayout은 그대로며 새 heap boxing·stack/native cap 변경은 없다. product_world_scope_split001에 before/after/diff/full maps와 한계가 있다. pre-edit 경계 assertion 실패와 그뒤 no-change formatter001은 별도 원본으로 보존했고 실제 교정은 apply_patch 후 exact body 대조로 수행했다. scopedfmt002는 raw0/2.751874s였다.

교정 후 integrated_r07_stack_diagnostic002는 동일 기본 stack·동일 환경·원 exact selector1thread에서1/1 PASS, exit0/118.793921s(시험17.84s), 닫힌 source438/73a3d7b6b22a49e6632bcc3b83a5842d614ae0cae55dda324b573ff22ab20425 unchanged다. 기존 fixture/oracle 수정은0이다. postbinary의 해당 wrapper680byte/scoped body129448byte로 두 frame 예약은259760→130128byte,129632byte 감소했다(복귀주소/다른 closure frame 제외). POST_DIAGNOSTIC.json에 actual prologue/binary SHA와 receipt를 결속했다. 기본 stack 국소 성공은 전체 통합003의 abort를 PASS로 바꾸거나 native/heap cap 실측을 대신하지 않는다.

root public015의 coverage_preflight001은32수용문장/62참조의 읽기 점검이며 실행 성공·독립 감사가 아니다. G1–G7의 직접 수용 반례 부족을 추가 보강하기로 인수했다. 현재 Astra가 작은 시험 묶음의 작성/포맷/집중 실행을 소유하고 Sol 제품/전체 통합·성능 및 Htools 쓰기/실행은 HOLD다. root PLAN/NOTES는 편집하지 않았으며 U08 GEX10 partial, 최종 현재 통합/public/measurement/PC/R05·coverage·자체검증·동결/복원·독립 감사는 남았다. 이전54선행 성공과 모든 새 실패/작성 이력·old target를 보존한다.


## G1 실제 종료·비영향 재사용 및 G6 읽기 인수 — 2026-10-06/016

docs_checkpoint019는 root public016의 RECORD.ko.md/RECORD.json과 test_g1_stage001/RESULT.json을 인수한다. 시험 작성·집중 실행은 Astra/high, 이 문서 보완은 기존 Sol6.1/xHigh다. G1 결과의 마지막 닫힌 source는438/04a14334d757eb14a41f1b30a1b1915fa3b3980aff15e1b78109e015660bafec이며 R07 성공 뒤 시험3파일만 변경됐다. 현재 Astra G4 token과 공유 live source의 고정을 주장하지 않는다.

| 실제 label | 결과/원인 | 실행 source | 시간(s) |
| --- | --- | --- | --- |
| relic_g1_duplicate001 | compile101/test0; ActionDisabledReasonCode import 누락 | 442fa9b3… | 43.860758 |
| relic_g1_duplicate002 | exit0/1/1 PASS | 8f5055d9… | 98.816126 |
| relic_g1_battle001 | exit101/0 PASS·1 FAIL; common-preflight 오류종류 기대가 실제 CoreDomain 계약과 불일치 | 8f5055d9… | 5.014752 |
| relic_g1_battle002 | exit0/1/1 PASS | d0f3b26a… | 98.514777 |
| relic_g1_weapon001 | compile101/test0; json! untyped None 추론 오류 | d0f3b26a… | 73.431969 |
| relic_g1_weapon002 | exit0/1/1 PASS | 04a14334… | 20.666203 |

6개의 receipt/raw 및 source archive hash·전후 unchanged를 BOUND_RECEIPTS.json에 결속했다. 초기 실패와 test_g1_fix001–003의 pre/after/diff는 그대로 보존한다. duplicate002는 Normal/Corrupt 장착 역순×휴면4경우의 중복 거절과 정의·사본·양직원 슬롯·UUID 무변경을 inventory 계층에서 검증했으며 absent-provider fixture를 정상 full-loader 콘텐츠 proof로 확대하지 않는다. battle002는 실제 활성전투의 EquipRelic/UnequipRelic/CorruptRelic 각2회 거절 및 world roots·실제 battle/queue/facts/events/revision/deployment/attempt·SavePoint/Continue 보존이다. weapon002는 공개 CoreApplication으로 Normal/Corrupt 실제 무기를 해제해도 RelicState와 known penalties가 보존되고 fabricated stats/provider가 생기지 않음을 확인했다.

requires_battle_authority=false를 common preflight 실행으로 해석했던 root/Astra의 원 oracle 전제를 정정했다. 실제 descriptor CoreDomain→uses_common_preflight=false→gate_behavior_action의 domain 위임→relic_runtime loadout-context 거절은 InvalidAction이다. 전투중 거절·무변경 정책과 기존 제품 API는 유지했고 제품 오류종류를 추가하지 않았다. duplicate002 뒤 변경은 carrier 오류 기대/public JSON, battle002 뒤 변경은 public JSON뿐이므로 root가 그 범위에서 exact 성공을 재사용한다. 세 성공을 최종 동일 source에서 모두 실행했다고 기록하지 않는다.

G6_READONLY_CONTRACT.json은 Sol의 제품 소유권 읽기 참고다. BattleCore의 relic_numeric_owner는 withdrawal 뒤에도 해당 battle 및 clone/candidate 수명에 묶이고, release source의 binding은 별도 immutable 기여 owner다. 지연 carrier map/queued payload/cast 종료와 현재 battle 제거, authoring last_terminal 교체는 각각 다른 해제 경계다. last_terminal은 다음 battle 진입에 지워지지 않고 다음 terminal capture에서 교체되며 최신 amount source/plan/frame을 남길 수 있다. confirmed_combat_result는 CompleteCombatResult acknowledgement에서 take되지만 외부 snapshot/Continue/candidate 공유참조는 별도다. GameData generation과 catalog의 compiled plan은 계속 소유되므로 전투 종료만으로 그 Weak 소멸을 기대하지 않는다. 실제 Weak 해제·반복 종료·boundedness를 실행 검증한 증거가 아니며 G6 PASS로 세지 않는다.

G4·G2/G3·G5/G6·G7 직접 수용 보강과 현재 전체 기능/공개·기능 종료 후 PC/R05·coverage/selfvalidation/freeze/whole 독립 감사는 잔여다. integrated003의 partial abort와 R07 국소1/1은 그대로 분리한다. 기본8MiB/PC31success5refusal/R05 cap·peak-only 경고 정책 불변이다. 현재 전체 U08 partial/독립 감사 미실행이며 U09 미진입이다. 이 bundle은 EXPS/STATE 및 새 증거문서만 쓰고 PLAN/NOTES·제품/시험/도구·포맷/빌드/시험/성능 실행은 하지 않는다.


## G4 실제 침식 실패 원자성·자산 경계 — 2026-10-06/017

docs_checkpoint020는 root public017과 test_g4_stage001/RESULT.json을 인수했다. Astra/high가 작성·집중 실행했으며 Sol6.1/xHigh는 EXPS/STATE 문서만 보완한다. 변경은 game/world/tests/u08_relics.rs 정확1파일이고 제품 변경은 없다. atomic001 이후 paired preview가 보강돼 실행 source가 바뀌었지만 최종 module13에서 새2시험 모두 다시 실행됐다.

| 실제 label | 결과 | 실행 source | 시간(s) |
| --- | --- | --- | --- |
| relic_g4_corrupt_atomic001 | exit0/1/1 PASS | 7edc5d8c… | 113.746762 |
| relic_g4_corrupt_assets001 | exit0/1/1 PASS | 4f289cee… | 85.576982 |
| relic_g4_module001 | exit0/13/13 PASS; 위 새2개 포함 | 4f289cee… | 74.783371 |

3receipt/raw 및 source archive hash·전후 unchanged를 BOUND_RECEIPTS.json에 결속했다. 마지막 닫힌 source는438/4f289cee78777f175b2046b052c0f37170d2bda4834560b3700b80e90b27423b이며 현재 Astra G2 preflight/후속 입력의 live 고정이나 전체통합 PASS가 아니다. module13과 앞선2selector를15개 독립시험으로 중복 집계하지 않는다. G4에 실패 run은 없고 기존 G1/통합003/이전 실패의 원자료는 그대로 유지한다.

실제 CorruptRelic 실행 Core는 parsed immutable RuntimeSafetySettings에서 limits를 인수하고 시험 준비 상태를 이식한 fixture다. 외부 native::run scope 주입 없이 core.execute가 정상 budget entry를 생성했다. 생산 host 데이터 이관/실제 서버 process 측정과 구분한다. 장착 Normal copy+침식촉매1의 양성 대조에서 관찰한 retained_operation_start 요청30,689와 변경 이후 relic_query_snapshot32,587/relic_not_sellable_projection32,603 각각의 limit=requested-1을 사용해 정확 RetainedByteLimit/code·site·requested·limit 및 승인 admission prefix를 검증했다. 각 거절2회에서 전체 world roots·UUID·동일 copy/slot/Normal variant·촉매1·SavePoint/Continue 원본이 보존됐다. full Corrupt+Continue 누적 exact1,468,942에서 한 번 정상 성공해 copy/slot을 유지한 Corrupt와 촉매0 및 대조군 roots 동일을 확인했다. 이 값은 누적 admission enforcement 경계이며 typed payload 상계 충분성/allocator live·peak/RSS를 독립 실측한 proof가 아니다. 배포 default8MiB는 변경하지 않는다.

촉매0이며 실제 enkephalin/equipment_dust/legacyfragment_dust가 충분한 상태의 CorruptRelic는 InvalidAction·무소비다. 촉매1 성공에서 정확히1만 소비하고 다른 세 자산은 그대로다. alreadyCorrupt/missing copy/작성 corruption variant 없음은 PreviewRelicCorruption과 CorruptRelic 모두 InvalidAction·전체 roots 무변경이다. preview CoreCommon query와 실행 CoreDomain route를 구분했다. 현재 부활촉매 owner가 없으므로 두 실제 촉매 동시보유 교차시험을 주장하지 않고 U17 seam의 미실행 한계로 남긴다. frozen 혼용금지 정책과 독립 침식 자산/무대체소비 계약은 유지한다.

현재 G1/G4 국소 보강을 회수했으며 G2/G3/G5/G6/G7·현재 기능통합/공개·functional worker0 뒤 PC/R05·증거최종화/동결/독립전체감사는 잔여다. G6 읽기 참고는 docs_checkpoint019에 있고 실행 PASS로 세지 않는다. whole U08 partial/16.1·16.2 미완료/U09 미진입, PC31success5refusal/R05 원 cap·peak-only 경고 정책을 유지한다. 제품/시험/도구·포맷/빌드/시험/성능 실행은 이번 문서 bundle에서0이며 PLAN/NOTES root 소유는 편집하지 않는다.


## G2 외부 RON 진단·실제 재검증 generation 복원 — 2026-10-06/018

docs_checkpoint021은 test_g2_stage001/RESULT.json과 root public018 게시본을 인수한다. G2 시험 작성·집중 실행은 Astra/high, EXPS/STATE 문서 작성은 기존 Sol6.1/xHigh다. G4 이후 변경은 src/authoring/trial/tests/u08_relics.rs 정확1파일이며 제품/원 E009 export는 보존됐다. final source438/0eb5719496ccc828e6787c89032cb3791e84fbed06c4ea61194d4a8fde19d021은 아래 두 final selectors의 닫힌 동일 실행 입력이며 G3 preflight 이후 live 고정을 주장하지 않는다.

| 실제 label | 결과·적용 범위 | 실행 source | 시간(s) |
| --- | --- | --- | --- |
| relic_g2_api001 | exit0/1/1 PASS; 이전 exact API 경계 | 43c7d590… | 110.547195 |
| relic_g2_matrix001 | exit101/0 PASS·1 FAIL; 실제 nested parser field_path보다 좁은 None oracle | 155333ff… | 94.754114 |
| relic_g2_matrix002 | exit0/1/1 PASS·14실제사례; 이전 exact 입력 | 6a533031… | 114.854154 |
| relic_g2_restore001 | exit101/0 PASS·1 FAIL; Continue 전 목적지 profile 준비에서 encounter_compendium 누락 | ac948a32… | 106.125194 |
| relic_g2_restore002 | exit0/1/1 PASS; 실제 catalog2종×반복2거절+원 generation 양성복원 | 0eb57194… | 110.178848 |
| relic_g2_matrix003 | exit0/1/1 PASS·14실제사례; final source 재실행 | 0eb57194… | 14.763674 |

6receipt/raw 및 source archive hash·전후 unchanged를 BOUND_RECEIPTS.json에 결속했다. 초기 matrix001/restore001의 실제 FAIL을 보존하고 oracle/fixture 일반교정을 제품 오류로 재명명하지 않는다. matrix는 정확 제공된 nested field_path를 기대하도록 교정했고 restore는 typed PlayerProfileState의 Default 필드를 포함한 전체 직렬화로 정상 bootstrap 입력을 만들었다. final selector2개 안의14사례와 이전 matrix002의 동일 사례를 독립 신규시험으로 중복 집계하지 않는다.

정상 외부 RON reward/equip/SavePoint를 거친 nonempty TrialSession에서14사례를 full loader 및 authoring validate로 거절했다: unknown schema/conditional recipe/penalty variant, invalid·duplicate effect address, duplicate definition UUID, missing equipment/fragment/form, fragment 없는 form, corrupt 빈 effect/비음수 penalty, missing GrantRelic definition, authored Corrupt 지급이다. 실제 phase/code/pack/source/catalog/definition/field/cause를 확인하고 published token/catalog·전체 public snapshots/fingerprint·nonempty 저장 원본이 불변이었다. final relic validation의 field_path는 넓은 relics이고 row/effect/provider 주소는 cause에 담기는 한계가 있다. parser/reward의 실제 nested path 정밀도와 구분하며 모든 개별 field 구조화가 이미 제공된다고 주장하지 않는다.

restore는 checkpoint checksum 문자열만 변조한 시험이 아니다. 원 relic flat20→21 변경 및 정의 삭제(유효 catalog가 되도록 해당 normal grant 참조만 제거)의 두 실제 외부 catalog를 authoring validate해 각각 새 generation을 얻었다. 그 generation의 새 CoreApplication은 active run 없이 tutorial=true/highest_floor99의 nondefault profile을 가진다. 기존 active-run target은 generation validation 전에 거절되므로 active run 위에 복원하는 지원되지 않는 성공 경로를 가정하지 않는다. 각 두 generation에 동일 원 nonempty checkpoint Continue를2회 요청해 exact invalid_static_data 및 retained generation 불일치 이유와 목적지 ALL public roots/fingerprint/absent checkpoint 불변을 확인했고, 원 app·checkpoint bytes·token/catalog도 보존됐다.

원 RON을 되돌리고 재validate한 generation은 원 identity와 같았다. 별도 persistent profile/encounter_compendium의 기존 bootstrap을 실행한 뒤 Continue 양성대조에서 checkpoint exactbytes와 ALL public roots가 원본과 같았다. Continue가 별도 host profile을 자동 저장·복원한다는 의미로 확대하지 않는다. temporary fixture mutation과 원 E009 export 보존을 구분한다.

G1/G4/G2 국소 직접 보강을 회수했으며 G3/G5/G6/G7·현재 기능통합/공개·functional0 뒤 PC/R05·최종 evidence/impact/adaptation/coverage/selfvalidation/freeze/whole 독립 감사는 잔여다. G6 코드 읽기는 실행 proof가 아니고 integrated003 abort/R07 국소PASS를 전체 통합으로 대체하지 않는다. 기본8MiB/PC31success5refusal/R05 cap·peak-only 경고 정책 불변, 전체 U08/16.1·16.2 미완료/U09 미진입이다. 이번 문서 bundle은 EXPS/STATE와 새증거만 쓰며 PLAN/NOTES·제품/시험/도구 변경 및 포맷/빌드/시험/성능 실행0으로 닫는다.


## G3 원형 authored 효과·실제 재구성·순열·mitigation 회수 — 2026-10-06/019

docs_checkpoint022는 test_g3_stage001/RESULT.json 및 root public019 역할 기록/public020 완료 게시본을 인수한다. 시험 작성·집중 실행은 기존 Astra/high, 문서 보완은 기존 Sol6.1/xHigh다. final source438/b3f0af7e0243d70b9ce972ee0770c6798d2224c917ae8744f2f59f31ccb2e38e에서 carrier11와 lifecycle3을 모두 재실행하여 공통 carrier 준비/진입 helper 변화까지 닫았다. G2 이후 delta는 u08_carriers.rs/u08_lifecycle.rs 정확2시험파일이며 제품/policy/cap/owner 변경은 없다. 닫힌 검증 경계와 다음 G5 이후 live 입력을 구별한다.

| 실제 label | 결과·범위 | 실행 source | 시간(s) |
| --- | --- | --- | --- |
| relic_g3_permutation001 | exit0/1/1 PASS·6순열 | c5283500… | 109.274510 |
| relic_g3_original001 | exit0/1/1 PASS·이전 exact 경계 | c138e512… | 101.228957 |
| relic_g3_original002 | exit0/1/1 PASS·추가 step 소비/제거 음성 보강 | 9e0b376f… | 97.959169 |
| relic_g3_provider001 | exit101/0 PASS·1 FAIL·관찰 provider 경로 오류 | 9e0b376f… | 5.975410 |
| relic_g3_mitigation001 | exit101/0 PASS·1 FAIL·기본 stance 전제 오류 | 9e0b376f… | 5.117276 |
| relic_g3_provider002 | exit0/1/1 PASS | d70e408e… | 96.503303 |
| relic_g3_mitigation002 | exit101/0 PASS·1 FAIL·Defense에서 skill 준비를 기다린 순서 오류 | d70e408e… | 6.107070 |
| relic_g3_mitigation003 | exit0/1/1 PASS | b3f0af7e… | 85.350663 |
| relic_g3_carrier_module001 | exit0/11/11 PASS·같은 final 입력 전체 carrier module | b3f0af7e… | 19.447123 |
| relic_g3_lifecycle_module001 | exit0/3/3 PASS·같은 final 입력 전체 lifecycle module | b3f0af7e… | 6.050834 |

10receipt/raw와 source archive hash·전후 unchanged를 BOUND_RECEIPTS.json에 결속했고 RESULT의 oracles/preserved_failures/limitations 원문도 함께 보존했다. 앞선 selector 성공과 final module 재실행은 겹치는 시험이므로 독립 신규 건수로 합산하지 않는다. 초기3FAIL·각 source/preimage/diff와 준비-only test_g3_readiness001을 그대로 유지한다.

동일 세 사본의6슬롯순열은 actual requested/HP174, 동일 -15/-10 패널티 및 base/certified/persistent·정의 불변을 확인했다. 원형 authored 효과는 기존 AuthoredSkillEffect 항목의 add/remove이며 별도 시제품 tier/OnBattleStart/U09 created closure가 아니다. 원본/추가/제거 세 generation의 실제 skill HP vector는 [144]/[144,84]/[144], added_step release는0/1/0이었다. 추가 Instant step 지연73ms를 포함하도록 첫 hit+200ms 뒤까지 scheduler를 진행해 추가 소비와 제거 음성을 확인하고 catalog 원본·보유한 이전 facts는 그대로였다.

정상 typed AwakenSkillFragment/SelectSkillFragmentForm 및 EquipItem 뒤 fresh 실제 battle의 skill/basic HP는144/38→form교체100/38→weapon교체144/12였다. basic의 양성 provider UUID는 binding.equipment에서 읽고 unmatched contribution은 overlay가 없으므로 실제 deployed slot UUID/definition과 raw HP로 대조했다. RuntimeFactSource의 원 trigger/action 출처와 RelicNumericBinding의 exact selector provider는 별개다. 내부 synthetic catalog/admin ownership 준비를 정상 외부 acquisition proof로 재명명하지 않으며 E009의 외부 정상 경로는 별도 증거다.

outgoing Attack stance의 normal/corrupt144/210은 한번 합성된 뒤 기존 물리저항150/마법저항100, skill ModifyDamage+50% 순서에서 actual Physical85/126·Magic108/157을 만들었다. incoming은 outgoing skill 완료 뒤 정상 SetEngagementStance로 Defense를 선택하고 별도 새 enemy raw100 hit로 확인했다. 일반 Defense10/MR5의90/95와 Corrupt -5/-5의105, 실제 authored ground-focus10% 감소의81/85/94를 관찰했다. Ground/Defense/DefenseFocus tactical snapshot과 roster weapon profile, DamageFact unattributed modifier 및 실제 HP를 연결했다. legacy equipment modifier는 incoming_damage_modifiers로 합산되고 별도 equipment modifier_sources receipt가 없다는 한계를 명시한다. outgoing skill modifier와 incoming equipment modifier를 같은 출처로 표시하거나 Defense에서 skill이 성공했다고 주장하지 않는다.

provider001은 outer RuntimeFactSource.equipment에서 basic provider를 찾는 관찰 오류로 terminal까지 기다린 뒤 active_battle unwrap에 실패했다. mitigation001은 기본 Attack stance를 Defense로 가정했다. mitigation002 raw는 terminal unwrap만 보여주며 실제 readiness 직접 관측 proof가 아니다. 코드의 skill_start_disabled_reason=DefenseStance 및 manual resolve의 target/range 이전 ActionUnavailable 계약을 대조해 incoming Defense 단계를 outgoing Attack skill 뒤로 옮겼다. 세 실패는 일반 fixture/관찰 전제 교정이며 제품을 바꾸거나 기본 cap/oracle 정책을 완화하지 않았다. 별도 진단 assertion 준비는 실제 source에 적용하지 않고 추가 diagnostic compile을 만들지 않았다.

public019의 보조 u08_finalizer_author(gpt-6-astra/high)는 실제 model/effort context 확인 후 신규9Python+contract JSON/자체 prep 영역만 작성한다. 기존45harness는 HOLD이며 신규 영역은 run.identity 밖이고 별도 tool manifest로 인수한다. 준비 도구 작성은 기능/최종화 실행이나 완료 proof가 아니며 이 참여자는 최종 독립 감사자가 될 수 없다. Sol의 제품/통합·성능 writer/runner는 HOLD0이고 PLAN/NOTES는 root 소유다.

G5/G6/G7·현재 전체 기능/공개·functional0 뒤 PC/R05·evidence/impact/adaptation/coverage/selfvalidation/freeze/whole 독립 감사는 잔여다. G6 코드 읽기는 실행 proof가 아니고 integrated003 abort와 R07 국소PASS를 전체 suite로 바꾸지 않는다. 기본8MiB/PC31success5refusal/R05 원 cap·peak-only 경고 정책 불변, 전체 U08/16.1·16.2 미완료/U09 미진입이다. 이번 bundle은 EXPS/STATE와 새증거만 편집하고 제품/시험/도구·포맷/빌드/시험/성능 실행은0이다.


## G5 실제 Retreat·live host 재접속·checksum-valid 그래프 거절 — 2026-10-06/020

docs_checkpoint023은 test_g5_stage001/RESULT.json과 root public022의 닫힌 G5 회수 경계를 인수한다. 시험 작성·집중 실행은 기존 Astra/high, EXPS/STATE 문서 보완은 기존 Sol6.1/xHigh이며 과거 작성자 이력을 바꾸지 않는다. G3 이후 변경은 world/tests/u08_lifecycle.rs·world/tests/u08_relics.rs·host handlers/tests/durable.rs의 승인된3시험파일뿐이다. 제품/API/cap/policy 변경은 없다. 최종 source438/eae98e63146c1e9ac2cd91ea0668fb26ec7edb66872b4e7b0f35e7dffd36e2a5에서 retreat002·wire002·host003을 모두 실제 재실행했다. 이후 G6 preflight/후속 변경의 live source 고정을 주장하지 않는다.

| 실제 label | 실제 결과·범위 | 실행 source | 시간(s) |
| --- | --- | --- | --- |
| relic_g5_host001 | compile PASS·runtime catalog 준비 FAIL, exit101/0 PASS·1 FAIL | 321da2b7… | 93.915410 |
| relic_g5_host002 | exit0/1/1 PASS; 이전 exact host 경계 | d5ef3279… | 25.256287 |
| relic_g5_retreat001 | compile PASS·counter 기대시점 FAIL, exit101/0 PASS·1 FAIL | 7b0904d4… | 79.075762 |
| relic_g5_wire001 | exit0/1/1 PASS; 이전 exact graph 경계 | 7b0904d4… | 7.617317 |
| relic_g5_retreat002 | exit0/1/1 PASS; final 동일 입력 | eae98e63… | 75.400906 |
| relic_g5_wire002 | exit0/1/1 PASS; final 동일 입력 | eae98e63… | 7.421486 |
| relic_g5_host003 | exit0/1/1 PASS; final 동일 입력 | eae98e63… | 8.806405 |

7receipt/raw·전후 unchanged·runner/source archive의438 exact member bytes를 BOUND_RECEIPTS.json에 결속했다. 초기2FAIL/각 source/preimage/diff와 이전 exact PASS를 보존하고 겹치는 host002→003·wire001→002를 독립 새시험으로 합산하지 않는다. 최종 서로 다른 selector3개이며 whole module/full-suite 통과가 아니다.

실제 live PlayerGameActor에서 배치된 actor의 damage65/time6000을 확인했다. 사전 Pause 없이 실제 DetachSession이 자동 pause했고 AttachSession은 같은 battle UUID/current unit HP·위치·배치·시각과3copy/slot·1Corrupt·촉매2·durable entry checkpoint를 보존했다. paused advance5000은 진행0이며 Resume+100은 같은 battle에서6100으로 진행했다. Continue를 호출하지 않았다. tick interval60s의 실제 Actix handler 시험이고 process 사망 뒤 save continuation으로 확대하지 않는다.

Retreat는 내부 pre-battle catalog/admin 준비 뒤 ordinary world commands/scheduler를 실행했다. time10의 actual first hit174 이후 정상 RetreatBattle이 NodeConfirm/preview와 동일 copy UUID·slot·variant·자산·roster/certified base를 복원했다. retreats_used0→1, attempts_started는 첫 Confirm 이후1→Retreat 뒤1→두번째 Confirm 뒤2다. time0 재진입의 actual first hit도174이고 target HP와 fresh binding을 확인했으며 보유한 이전 fact는 불변이다. runtime relic/HP owner를 주입한 시험이 아니다.

wire는3slot 형태가 유효한 네 위조를 다뤘다: 한 copy의 두 직원 중복, unknown copy, unknown employee, allow_duplicates=false 동일 definition의 서로 다른 유효 UUID2개. 기존 helper로 canonical checksum을 갱신한 후 각2회 exact InvalidStaticData(run checkpoint relic graph is inconsistent)를 거절했다. shape/checksum/generation 이전 거절로 잘못 집계하지 않고 typed graph validator에 실제 도달했음을 확인했다. 목적지 전체 roots와 원 nonempty source roots/token identity/definitions/저장 원bytes는 보존됐다. destination은 기존 gate가 요구하는 active run 없음이다. 무변조 양성 Continue는 Corrupt 장착 copy와 촉매2를 복원했다.

host001의 with_equipment(items)는 새 EquipmentDatabase를 만들어 공식 equipment material을 버려 equipment_dust_reward 참조 준비가 실패했다. whole EquipmentDatabase를 clone하여 weapon_profile만 바꾸고 with_equipment_data(Arc)로 교정했다. G3의 합성 item-only fixture와 공식 material 참조 범위를 구별한다. retreat001은 ConfirmEnterNode 뒤 attempts_started를 pre-entry0으로 기대했으나 combat.rs695의 시작 이후 actual1이 맞았다. retreats_used와 attempt의 전환시점을 구별하는 시험 교정이며 제품 오류나 정책 변경으로 재명명하지 않는다.

RESULT의 한계를 그대로 유지한다: synthetic internal catalog/admin grants이고 정상 authored acquisition은 E009 별도, host는 한 live actor의 real handlers이며 process-death continuation 아님, Retreat는 내부 준비 후 정상 명령/scheduler이며 runtime relic/HP 주입 없음, Continue target의 active-run 없음과 원 nonempty checkpoint/source 보존, 승인된3시험파일만의 변화이며 제품/API/cap/policy/full-suite/performance/16.1·16.2 완료 주장은 없다.

root public021의 finalizer_prep001은 준비 계약이고 실제 finalization 성공이 아니다. G7 후 fresh48+execution harness manifest/root review, 첫 required receipt 전·PC/R05 포함 마지막 receipt 후 Sol UTC/tool exact hashes, 각 PC/R05 직전 실제 quiescence와 닫힌 functional receipts, DIRECT freeze/외부 /tmp stdout 계약을 인수했다. measurement_support의 혼합 stdout은 단일 JSON행을 엄격 검증하는 format: measurement_support_stdout_json_line 방식으로 보조 Astra가 prep002 successor 교정 중이다. prep001 원본/기존45harness는 보존하며 신규 교정 완료·실행 PASS를 예고하지 않는다.

G6/G7·현재 전체 기능/공개·functional 종료 뒤 PC/R05·최종 evidence→impact→adaptation→coverage→GEX10→문서→freeze/정역복원·비참여 whole 독립 감사는 잔여다. 이전 integrated003 partial abort/R07 국소PASS와 E009 정상획득은 각 실제 경계대로 분리한다. 기본8MiB·PC31success5refusal·R05 원cap/peak-only 경고·20분 stress 제외는 불변이다. 전체 U08 partial/독립완료 미판정/U09 미진입, Sol 제품·시험·도구/포맷·빌드·검증·통합·성능 실행0/HOLD다. PLAN/NOTES root 소유는 편집하지 않는다.


## G6 실제3회 전투·지연payload·최근 terminal 소유수명 — 2026-10-06/021

docs_checkpoint024는 test_g6_stage001/RESULT.json과 root public023/prep002의 닫힌 준비 계약을 인수한다. G6 작성·집중 실행은 기존 Astra/high이며 Sol6.1/xHigh는 EXPS/STATE만 보완한다. G5 이후 변경은 value_expr_runtime/tests/relic_amount.rs·world/tests/u08_carriers.rs 정확2시험파일이다. 기존 cfg(test) 자식에 inherent Weak-only 관찰자를 정의해 private owner/delivery/EvalCapture/source/binding/generation/plan과 extent를 읽었고, 새 strong Arc나 production API/visibility를 만들지 않았다.

| 실제 label | 결과·범위 | 실행 source | 시간(s) |
| --- | --- | --- | --- |
| relic_g6_lifetime001 | exit0/1/1 PASS; 초기 실제1회 | 416a1067… | 76.308951 |
| relic_g6_lifetime002 | exit0/1/1 PASS; 최종 실제3회 | ce0e0f2f… | 79.757265 |
| relic_g6_carrier_module001 | exit0/12/12 PASS; 위3회 selector 포함 | ce0e0f2f… | 21.640218 |

3receipt/raw/source archive438 exact member bytes와 전후 unchanged를 BOUND_RECEIPTS.json에 결속했다. 최종 동일 닫힌 source는438/ce0e0f2f44a1f9ab6665acbc78f3f8903f0da46c01d9afe56b8b230e716ec79a이며 G7 이후 live source 고정이 아니다. 초기1회와 최종3회·이를 포함한 module12는 겹치는 시험이고1+1+12를 독립14시험으로 집계하지 않는다. G6 runtime FAIL은 없으며 기존 통합/G1–G5 실패는 계속 보존한다.

각3회는 synthetic pre-battle fixture를 준비한 뒤 실제 entry/deploy/cast/release/withdraw와 지연피해210, timer terminal/Complete 경로를 통과했다. runtime relic owner/capture를 주입하지 않았다. terminal 뒤 active_battle=None이며 battle relic owner·delayed delivery·EvalCapture·원 capture source의 Weak upgrade가 None이어서 해당 strong owner와 T/소유 값의 drop을 확인했다. 공유 Arc의 다른 정당한 owner는 별도로 구분한다.

최신 terminal runtime facts는 confirmed receipt와 last_terminal이 공유한다. Complete는 confirmed receipt acknowledgement를 제거하지만 최신 facts/binding은 last_terminal에 남는다. 다음 battle entry는 이전 last_terminal을 유지하며 다음 terminal이 교체할 때 더 오래된 facts/binding Weak는 모두 None이었다. 유한3회에 이전 battle owner/capture/cache의 strong owner가 누적되지 않았음을 확인했다. 마지막 GameCore drop 뒤 현재 terminal facts/binding strong owner도 끝났지만 별도로 보유한 ValidatedGameData의 generation과 compiled amount plan은 의도대로 살아 있었다. 전체 generation/plan 소멸을 전투 종료의 기대값으로 만들지 않는다.

실제 각 회의 current extent는 relic_owner4,108·delayed_delivery4,451·eval_capture3,996·terminal_runtime_facts46,619였다. 첫 세 구조의 tuple은 첫 방출 tuple과 반복 동일하다고 assert했고 terminal facts는 양수라고 assert했다. terminal46,619 세 번의 동일값은 raw 관찰이며 별도 동일성 assertion으로 재명명하지 않는다. 이 extent들의 합을 중복 참조 없는 동시heap 상계라고 주장하지 않는다. 현재 owner 구조와 누적 admission ledger는 다르고 해제·terminal·rollback에서 allowance 환급/reset을 기대하지 않는다.

Weak handle 자체는 Arc control/backing storage를 마지막 Weak drop까지 남긴다. Weak::upgrade=None는 strong owner와 값의 drop 증거이며 backing allocation의 allocator 해제·전체 leak census 증거가 아니다. 초기1회 source 주석의 allocation release 과장은 final3회 확대에서 strong-owner/payload drop과 Weak backing caveat로 교정했고, 실제 assertion은 처음부터 Weak였다. 유한3회 structural/lifetime 관찰뿐이며 total heap bound/allocator live·peak/RSS를 새로 실측한 proof가 아니다. latest last_terminal과 장기 GameData/compiled catalog는 허용 owner로 명시 구별한다.

prep002 MANIFEST bf519a16949f4d68151deb9af914510c572c9648439faedef55d0260f90f4bac/HANDOFF f95a616ff898054dad9b8bf7a24addd0212cf834ac8c36fcc82b189ab41ea7a3 및 root public023을 인수했다. 변경은 신규 finalizer evidence_common.py/freeze_current.py 두 파일뿐이다. measurement_support_stdout_json_line assertion이 선택된 자기 receipt/raw hash·정확 cargo example/feature/성공·source unchanged/실제 launch 이후 마지막 단일 JSON 객체 및 instrumented=false/live·peak=null을 요구해 혼합 stdout을 원본대로 검증한다. 별도 sidecar나 기존runner 변경은 없다. source 목록/복원 union은 실제 sorted(Path)와 원 digest를 검증하며 과거 receipt를 재정렬/재hash하지 않는다. 원 prep0016파일·신구tool archives 각각10멤버와 prep002 초기 실패는 보존한다. 준비검사42(32음성)와9py 메모리문법은 실행 proof가 아니며 finalization/측정/복원 실행0이다. stage/48+manifest/Sol before·after UTC/각PC·R05 실제 quiescence/DIRECT freeze 계약은 그대로다.

현재 보조 Astra는 G7 public_relics.py와 fresh48+execution harness handoff002를 준비 중이다. G7·root public024는 아직 닫히지 않았고 해당 결과/refs를 이 checkpoint의 완료 증거로 결속하지 않는다. 제품/시험/도구·포맷·빌드·검증·통합·성능 실행은 Sol 경계에서 HOLD0이며 PLAN/NOTES root 소유는 편집하지 않는다. G7/current 기능·공개→functional0 뒤 PC/R05→최종 증거/자체검증/문서/동결·정역복원→비참여 whole 독립감사는 잔여다. G6 selective PASS가 전체 suite/U08/16.1·16.2 독립완료를 대신하지 않으며 U09 미진입, 기본8MiB/PC31success5refusal/R05 원cap·peak-only 경고와20분 stress 제외는 유지한다.


## 통합004 실패·11개 집중 교정·환경 차단 — 2026-10-06/022

docs_checkpoint025는 새 입력외 문서 기록이다. 통합/환경 진단/제품 설명/포맷의 실행자는 기존 Sol6.1/xHigh, 7시험파일 교정·11개 집중 실행자는 기존 Astra/high다. 이전 저자·실패·source/command/raw/receipt는 원본대로 보존하며 PLAN/NOTES root 소유는 편집하지 않는다.

| 실제 label·경계 | 원 결과 | source·시간(s) |
| --- | --- | --- |
| integrated_fmt004 | exit0, cargo fmt --all -- --check; 변경0 | ce0e0f2f… / 4.947912 |
| integrated_core_default004 | exit101; 1,872 PASS/12 FAIL, ignored0; libtest203.54s | ce0e0f2f… / 228.179656 |
| integrated_matrix004 | exit101; 위 실패에서 중지, 후속 단계 없음 | ce0e0f2f… / 238.985297 |
| integrated_fix_explain001 | exit101, ron::from_str namespace 준비 compile 실패; runtime test0 | 177e32b1… / 34.455008 |
| integrated_fix_explain002 | exact1/1 PASS, 412개 정의 query에 외부 Relic 포함 | 8734111c… / 79.601637 |
| integrated_fix_describe001 | exact1/1 PASS | 8734111c… / 5.840287 |
| integrated_fix_raw26001 | all_26 새 exact selector1/1 PASS | 8734111c… / 3.498830 |
| integrated_fix_packaging001 | exact1/1 PASS, source/pack27파일 | 8734111c… / 10.287973 |
| integrated_fix_generation001 | exact1/1 PASS | 8734111c… / 7.935144 |
| integrated_fix_live_ron001 | exact1/1 PASS, catalog26 | 8734111c… / 3.340201 |
| integrated_fix_map001 | exact1/1 PASS | 8734111c… / 3.541303 |
| integrated_fix_defense001 | exact1/1 PASS | 8734111c… / 6.855232 |
| integrated_fix_no_employee001 | exact1/1 PASS | 8734111c… / 3.513925 |
| integrated_fix_visual_novel001 | exact1/1 PASS | 8734111c… / 6.802219 |
| integrated_fix_omen001 | exact1/1 PASS | 8734111c… / 3.546525 |
| post_correction_fmt001 | exit0, cargo fmt --all -- --check; empty raw/변경0 | 8734111c… / 5.072868 |

통합004의12실패는 catalog/definition 관련6개, 기존 capability empty 기대5개, Unix socket 준비 EPERM1개다. Astra는 기존 Relics26 catalog·27 source/pack파일·45 definition kinds를 독립 기대에 명시했고 authoring explain에 외부-only 정상 Relic 정의를 포함해 exact source/full value를 검증했다. 공식 유물 밸런스/콘텐츠를 새로 만들지 않았다. 다섯 상태는 기존 state/transport/cursor/defeat assertion을 유지하면서 enabled Query DTO인 RequestRelicState·RequestRelicNumericPlan만의 정확 capability vector를 검증한다. raw selector의 all_25 이름은 all_26으로 이동했으며 원 실패 이름/receipt는 보존한다. 초반 ron::from_str는 ron::de::from_str로 시험 namespace만 교정했고 첫 compile 실패를 test PASS로 세지 않는다.

제품 설명 교정 product_catalog_doc001은 packaging.rs의 `and 25 official catalogs`를 `and the official catalogs`로 바꾼 doc comment 한 줄뿐이다. ce0e0f2f→76947742의 438 membership은 동일하며 의미·테스트·harness 변화0이다. 이어 Astra 승인7시험파일만의 delta로76947742→8734111c가 됐다. 이전 저자의 테스트 기여/원본은 보존하고 새 교정 저자를 구분한다. RESULT.json 2fd46eb2… 및16closed receipt/raw·3source archives의 각438 exact member를 BOUND_RECEIPTS.json에 결속했다. 집중11PASS는 전체 default/all-targets 성공이 아니다.

execution_harness_binding004/ABORTED_INTERVAL.json e00789ab…은 실제10:40:54.148416UTC에 닫힌 실패 구간이다. 첫 required receipt 전 BEFORE/UTC와438 source·122입력의 순서/bytes/hash, external archive와 required2member가 실패 종료 때 동일함을 확인했다. BEFORE는 불변이고 final execution binding/PASS가 아니다. verify_remaining004/PC/R05는 미실행, functional49 receipt set도 미완결이다. 통합12 중 fmt와 default만 실행됐고 default 실패 뒤 나머지 stage/공개17·release·actual host/measurement·final G7는 실행하지 않았다.

env_socket_probe001/RESULT.json fc52efee…은 현재 workspace-write/network-restricted/approval-never sandbox에서의 진단이다. AF_UNIX socket 생성 성공 뒤 /tmp bind가 errno1 EPERM, AF_INET loopback은 socket 생성부터 errno1 EPERM으로 bind 미실행이었다. 생성 socket 즉시 close/tmp 제거, connect/listen/앱접속/메시지/외부network/권한승격·우회0이다. 현재 namespace만 관측했으며 다른 환경의 가능성을 증명하지 않는다. snapshot/tests.rs의 원 Unix socket fixture는 byte 불변이고 해당 실패는 skip·조건부 PASS로 바꾸지 않는다. 같은 환경에서 필수 socket/host 검사 경계가 실제 차단됨을 인계한다.

post_correction_fmt001은 현재 닫힌438/8734111ce349bc36de5534aff831f89ccf8c9034b7e802e120d87cee5d0e248b에서 actual exit0/source unchanged이며 receipt7f94f59a…/rawempty e3b0c442…이다. owned runner0·visible relevant process[]를 회수했고 새005 전체통합은 시작하지 않았다. root public025/HANDOFF 및 보조 Astra fresh003 source 재결속은 작성/준비 중이며 닫힌 refs/실행 PASS로 추정하지 않는다.

현재 U08 전체 기능/공개·final PC576/36조건31성공5거절·R05·증거/impact/adaptation/coverage/GEX10·동결/정역복원·비참여 whole 독립감사는 미완료다. 기존 early PC/G1–G6/E009 국소 성공은 각 당시 경계로 유지한다. 기본8MiB/원 cap·workload·oracle, R05 원cap과 concurrent-peak 단독 경고/20분stress 제외는 불변이다. U08 partial/16.1·16.2 미완료/U09 미진입이며 환경 차단을 예외/성공으로 처리하지 않는다. 이 문서 묶음 종료 뒤 Sol product/tests/tools/docs writer0·runner0/HOLD로 반환한다.


## 통합005 중단·비유한 RON 집중검사 회수·전체권한 환경 준비 — 2026-10-06/023

이번 사용자 중단과 전체권한 전환 뒤 원 /root/u07_implementation·/root/u08_integration_runner의 현재 live 가용성이 없어 /root/u08_product_runner_resume(gpt-6.1-sol/xhigh)를 제품 교정·통합/빌드/기능/성능 담당으로 새 배정했다. 주 시험 담당은 /root/u08_test_author_resume(gpt-6-astra/high), 보조 harness 담당은 /root/u08_harness_author_resume(gpt-6-astra/high)다. 이전 제품/시험/실행 작성 이력은 해당 원 저자에게 남긴다. 실제 developer 권한은 danger-full-access, network enabled, approval never이며 sandbox_permissions 인자는 사용할 수 없다. public026/roles003의 require_escalated 지시는 이전 실행 환경 이력이다. root는 계획·읽기·문서조정만 맡고 제품/시험 작성자나 최종 독립 감사자가 아니다.

- 통합005: 원 source438/8734111ce349bc36de5534aff831f89ccf8c9034b7e802e120d87cee5d0e248b에서 fmt PASS와 Core default all-targets 1,957 PASS/lib1,884/0FAIL/0ignored를 닫았다. raw의 game::data::content_pack::snapshot::tests::unix::invalid_utf8_directories_and_socket_inputs_return_errors도 ok다. root가 PE-CRNV-06.2의 직접 flat:nan/percent:inf 음성검사 누락을 찾아 완료 leaf 뒤 다음 leaf를 시작하지 않고 부모 loop만 계획 중단했다. outer integrated_matrix005 exit-9, 원 leaf/raw/receipt와 실제 2026-10-06T11:54:05.792811+00:00 종료를 ABORTED_INTERVAL.json(SHA256 2d2fb07bcf66d9d7dcd7d9840e2a88cd1c5e26d1df19b03c8907bbec068cd50c)에 보존한다. 제품 실패가 아니며 49기능 중2개만 실행됐고 나머지47·verify_remaining·PC/R05는 미실행이다.
- 비유한 추가001: 원 Astra가 u08_relics.rs 한 시험 파일에14줄/2사례만 추가했다. source membership438 유지, aggregate9af38a9083aeccd3ad95caf6ad0bbd425cf7adf2aec0f5c20e5f183f7845ae47이다. focused001은 사용자 중단으로 Compiling 한줄68bytes raw만 남았다. SHA256 09bd4a4538575d12d90823a431b438a416f570b8c108e81823e7ac50da4d691c, JSON receipt 없음/재구성 없음으로 인수한다. before/after/diff/archive를 과거 source에 재적용하지 않는다.
- 비유한 집중002: 새 주 Astra가 기존 추가 bytes를 그대로 인수했고 직접 source edit0이다. scopedfmt002 exit0과 exact Rust시험1/1, 내부16사례 PASS(기존14+nan/inf2)를 동일9af source unchanged로 닫았다. RESULT.json SHA256 38314a5e1321ede2bff57c14c2fbd558e2823350bad40b37e1beaf462837968a, focus receipt a0d97ff7177738dd2d5dc7b2c5246cb19a8ba35fc6b7dff0115805e79c8353f0, raw2f7b4c962afe592dadeb668b72dddd87c53583401323cbb8c5051154a7edb384다. flat/percent typed필드에 code=ron_parse/phase=parse/parser_reason=ExpectedInteger/cause=Expected integer와 구체 field_path를 확인했다. 원 published token/정의/nonempty snapshot/save bytes 보존·복구/양성 재검증 assertion도 통과했다. 16개 별도 발견시험이나 새 전체통합 성공이 아니다.
- parser 근거: test_g2_nonfinite002/PARSER_BASIS.json은 game_core의 실제 직접 ron0.5.1과 i64 deserialize_i64→signed_integer의 ExpectedInteger를 소유한다. Cargo.lock에 별도로 존재하는 ron0.8.1을 이번 직접 parser로 언급한 원 표현은 이 후속 기록에서 정정한다. 제품 schema/parser와 기대 거절 의미는 바뀌지 않았다.
- 환경 준비 dependency_fetch001: 새 환경에 없던 /tmp/wp02-cargo-home는 주 Astra exactfocus --locked가 먼저 채웠다. 이어 Sol의 run.py -- cargo fetch --locked가 2026-10-06T12:11:25.810204+00:00부터4.384495초/exit0/source unchanged로 workspace 의존 캐시를 준비했다. Rust/Cargo1.98.1, CARGO_HOME=/tmp/wp02-cargo-home, CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/d07-runtime-amount-product-target다. Cargo.lock SHA256 4f4e8b179849225904f359095908b8de9c17ae590f52c93aff93fbce7b0c9358 유지, registry패키지341의 .crate SHA가 lock checksum과 일치/누락0/불일치0이다. receipt6b5b109f71ab4956b6520a9ac7cbc14e9b50862c0d8b1e1a01142fc786df555b/raw3d18b433f352a78435b54ededecdb524455431647393150d790c715a60c25ced는 환경 준비 증거이며 49functional 선정에서 제외한다. 컴파일/시험/설치/lock갱신은0이다.

기존 harness003의122 regular 입력은 완전 closure로 다시 실행하지 않는다. fresh 프로세스의 default_parity.py는 Safety/tools/initialize.py와 Safety/execution/s1/baseline/SOURCE_BEFORE.json·SOURCE_BEFORE.tar.gz의6 Rust member를 소비하며 기존003 manifest/external archive에서 빠졌다. root의 추가 읽기 인수에서는 그 밖의 regular1200 추가 누락 검토도 확정됐고 보조 Astra가 dependency-review proposal을 작성 중이다. 기존도구 bytes를 교정하거나122로 fullmatrix를 시작하지 않는다. complete fresh004 manifest가 실제 최종 source·전체 regular bytes/외부 archive·member에 결속되고 root review가 닫힌 뒤 새 미사용006 before/after/UTC/writer0로 integrated_matrix.py 006 0→verify_remaining.py 0 006을 별도 위임받는다. 1,322 입력 결속·future006 실행/PASS는 아직 주장하지 않는다.

49필수 nonperformance 역할의 정확한 receipt 경로를 선정하고 기능 종료 뒤 각 PC/R05 직전 actual source/UTC/inputwriter0/functionalrunner0/closedrefs의 quiescence를 남긴다. optional historical focused는 현재 선정 계획에 포함하지 않는다. fresh release/debug build/binding 선행, 외부 archive 전체·필수 member와 실제 실행 구간을 보존한다. 이번 Sol 범위는 환경 fetch+EXPS/STATE와 새 checkpoint뿐이며 제품/시험/기존 harness 입력을 변경하지 않았다. PLAN/NOTES/HANDOFF는 root 소유이고 편집하지 않는다.

이전 통합002 fmt 실패/003 stack 중단/004 1,872PASS12FAIL과025 EPERM·원 실패/target/archive는 보존한다. U17 actual revival catalyst 교차시험은 여전히 미구현이다. U08 GEX10 partial/16.1 자체검증·16.2 전체 독립감사·최종 기능/공개·PC/R05·impact/adaptation/coverage·동결/복원은 잔여이며 U09 미진입이다. 기본8MiB·PC31성공5거절·R05 cap9,030,188의 concurrentpeak 단독 기존 경고·20분stress 제외를 유지한다. BOUND_RECEIPTS와 DEPENDENCY_CACHE가 이번 실제 refs를 소유하며 Sol 문서 종료 뒤 입력writer0/기능runner0/HOLD다.


## 2026-10-07T02:56:10.930574+00:00 — 사용자 연속 실행 재개 점검

최신 인수034를 유지한다. 재개 점검에서 source438/4c7bbd31·원1322 입력과 최신 인수인계·직전 실행문서가 동일하며, 현재 /proc에서 관련 제품 실행 프로세스는0이었다(해당 가시성 밖의 프로세스는 판단하지 않음). 기존 tracked 변경198·삭제27은 되돌리지 않았다. 집중005 성공 및 기존25개 exit0 기록은 재사용하며 미실행 기능25·성능8을 이어간다.

실제 배정 기록이 확인된 Sol /root/u08_product_runner_resume(gpt-6.1-sol/xhigh)는 통합 실행 대기, Astra /root/u08_harness_author_resume(gpt-6-astra/high)는034의 fault 집계기·ResourceSum probe003 준비를 계속 소유한다. 이전 집중 담당 Astra는 종료 상태이며 추가 작업을 배정하지 않는다. 새 resume_off.py 작성 중 상태를 인수했고 제품/통합 실행은 아직 없다. Root는 제출 결과를 인수한 뒤 단독 통합 실행으로 연결하며, 그 전에는 추가 조회나 중복 기록 없이 결과 알림을 기다린다.
