# U17 실행 기록

2026-10-15: P2G40 receipt 생성/검증과 P2G50 quality contract2 검사 통과. policy6요소/22수용·1Phase·6통합 경계·10검증 차원. 현재 런타임 시험0이며 제품 완료 근거가 아니다.

실제 pre-edit 기준본515파일 archive member 검증 통과. 활성화 스크립트는 이미 존재하는 실행 디렉터리에 mkdir하여 한 번 실패했다. 제품 변경이나 artifact 발행 전에 난 오류이며 exist_ok로 교정 후 P2G50/60·GEX00 발행했다. 과거 스크립트/원본 artifact는 실행하거나 덮어쓰지 않았다.

check001: cargo check -p game_core --lib --offline --locked, exit101, runtime0. 새 action projection 배열 길이/retired reason 문자열 누락, 장비 metadata에 없는 description/icon 참조, snapshot ActionKind import 누락의5개 컴파일 오류. 현재 타입에 맞춰 교정하며 기능 기대값 변경은 없다. 원문 /tmp/u17-check001.log 보존.

check002: cargo check -p game_core --lib exit0 (/tmp/u17-check002.log). runtime0. compile-tests001은 구 모노리스 필드/정책·구 repeat_reward 시험69개 컴파일 참조 오류로 exit101, compile-tests002는 신규 시험 import/빈 struct 명령 문법3개 오류로 exit101. 원문 /tmp/u17-compile-tests{001,002}.log. 폐기된 교환/모노리스 수치/소탕 성공 시험을 그대로 통과시키는 호환 상태는 추가하지 않는다. 기존 checkpoint·비용·보상 source·배율 시험의 목적을 새 타입으로 옮기고, 제거한 모노리스 전용 시험은 명시적 retired rejection과 새 부화/보상/복원 시험으로 대체한다. 현재 pilot001 실행 중이며 기능 PASS는 아직 없다.

구조 검토: FacilityNodeMetadata는 예약뿐 아니라 모든 일반 배치에도 존재했다. 이를 부화 거절 조건으로 쓰면 실제 일반 노드 전체가 거절된다. 실제 room.reservations·omen_compatible·normal_activity identity를 대조하도록 수정했다. 부화 노드는 같은 층 room 이동에서는 유지하고 하강 시 runtime 부화 목록에서 제거하며 알은 원정에 남는다. 이전 방의 지도 기록은 기존 시설 owner에 보존한다.

콘텐츠 선택: 일반 보스12종에 공용 촉매1개, 작성 강화140/180%, 고유 파츠2/4개, 기존 전용 장비 후보를 연결했다. 고정 엔진 수치가 아니며 후속 플레이 밸런스 대상이다. 마탄의 기존 source는 도감/식별용으로 유지하되 ordinary rewards/progression/egg를 제거하고 loader에서도 이 예외를 검증한다. 엘리트13종 최초 장비와 가루5+XP8 반복 보상을 작성했으며 부족한 전용 장비 신규 설계는 이 작업에 끼워 넣지 않았다.


pilot001:9selected/6pass/3fail. 두 실패는 live 초기 selectable 노드가 일반전이라는 준비 가정 오류, 한 실패는 이제 BTreeMap이어서 구 HashMap capacity provenance를 요구하지 않는다는 기대값 이행 누락이었다. 전자는 기존 owner fixture의 지원 일반전 그래프에서 결제/복원 경계만 준비하고, 실제 authored 경로는 별도 u17_public에 유지했다. 후자는 정확 clone 경계/1미달 거절은 유지하고 BTreeMap에는 별도 provenance가 불필요함을 확인하도록 교정했다. pilot002:9pass/0fail/ignore0(5.43s). related001:101selected/99pass/2fail(16.61s), 두 실패 모두 구 파편가루 반복 보상 기대값/overflow 자산 준비였다. 새 장비가루 보상과 실제 GrantEquipmentDust 생산자로 원자성 시험 목적을 유지했다.

public-pilot001은 AbnormalityDatabase import 누락으로 compile exit101. public-pilot002:12pass/1fail(5.38s), public-pilot003:15pass/1fail(5.44s). 모두 새 u17_public의 authored fixture 준비 중 실패이며 첫 실제 전투는 아직 도달하지 않았다. RON0.5의 internally tagged ShopMerchandise 재직렬화가 quoted tag를 써 parser에서 거절되어 원본 RON의 정확한 수량/가격 행만 수정하도록 교정했다. 이어 corroded profile parser가 range preset을 실제 tile range로 확장하므로 재직렬화에서 둘을 동시에 작성했고 duplicate contract에 거절됐다. public-pilot004:0pass/1fail(0.24s)은 preset 자체를 제거한 잘못된 교정이 정상 profile의 필수 preset 조건에 걸린 결과다. 올바른 교정은 authored preset을 유지하고 파생 defense_tile_range만 내보내지 않는 것이다. pilot005에서 이를 확인한다. 원문 /tmp/u17-public-pilot001..005.log를 보존한다.

15개 집중 통과에는 소지/창고/혼합 지불·실패 원자성, 부화 전 입장분리·Continue 보존, 2회후퇴 후 마지막 도전과 중복 입장비 방지, 미완료 일반전 내부 hatch의 Continue rollback, 부화 전/후 checkpoint 비용·단계 복원, 구명령 typedreject와 구상점 제안 부재, 실제 작성 오류 matrix가 포함된다. 공개 host/새 프로세스·실제 전투·IO·최종 통합·비용·동결은 아직 남았다.

부화 완료 record에 U15 MaterialInvestment 지불 영수증을 보존하고 조회에 해당 단계 강화/보상/후보/다음 알을 같이 노출한다. pre-hatch option만 남겨 egg 소비 뒤 보상 표시가 사라지는 공백을 닫았다. 이 새 owner의 CloneShape는 영수증도 포함한다. 시설 하강은 부화 node 목록을 비우고 eggs를 유지하며 같은층 이동에는 비우지 않는다. 마탄 rewardless outcome에서도 보상/최초 원장을 변경하지 않고 normal boss cycle 배정만 소비해 cycle이 영구 정체되지 않도록 분리했다.

public-pilot005에서 authored loader 준비는 통과했고 starter helper가 첫3명만 골라 scout를 포함하지 않아 배치 준비가 실패했다. pilot006부터 공개 SelectStarterEmployees로 scout/guard/medic를 명시한다. pilot006/007은 전투75s 제한에서 적이 살아남았다. 실제 관측은 ground(0,9)/(1,9)와 적(4..6,6)·보스(5,9)로 사거리 밖이었다. 시험 전용 정상 RON의 시작 총 범위를 전방3×5로 넓히고 중앙 양옆 배치를 선택했다. 공식 장비 밸런스를 바꾸지 않으며 실제 wave/공격/피해/승리 조건은 유지한다. pilot008은 첫 보스 승리→파편/알→상점 촉매4개→2단계 부화까지 진행했으나 새 arena의 보스가 배치 뒤쪽에 남았다. 정상 TacticalRelocateUnit 방향 전환을10s마다 사용하도록 보완했다. pilot009는 Side import 컴파일 오류, pilot010은 실제2단계 승리·파츠2·다음알까지 성공한 뒤 새 Core Continue에서 제품 오류 `unknown pending reward`를 검출했다.

제품 교정: 부활 장비 후보는 reward catalogue UUID가 아니라 해당 단계 equipment policy에서 생산된다. 원래 continuation reader는 reward catalogue만 참조했다. RevivalStagePolicy의 후보 생성자를 공유하고 결과 생산자/복원 reader가 동일 authored 후보와 순서를 대조하도록 수정했다. 이 변경은 정상 결과 저장을 되읽는 실제 경로이며 외부 변조 방어 추가가 아니다. hatch policy/progression와 출처 debit 복제도 기존 controlled_clone 경계에 포함했다. public-pilot011에서 수정된 전체2/3단계 경로를 재실행한다. 원문 /tmp/u17-public-pilot005..011.log를 보존한다.

## 전체 회귀001과 교정 준비 (2026-10-16)

`/tmp/u17-workspace-tests001.log`: workspace/all-targets exit101. Core2121 PASS/37 FAIL/6 ignore, ron_loading24 PASS/1 FAIL, 나머지 통합47 PASS, server96 PASS/22 ignore, 외부 probe11 PASS. 신규 U17 모두 통과. 실패 원인은 명령87·저장17 기대값 갱신, 부활 generated source 위치, 장비만 교체하면서 material/recipe를 버린 U08/U09 builder 호출, G11 과거 RON schema, 폐기된 모노리스 상점 자동 덮어쓰기를 기대한 HQ fixture, 마탄 무보상 정책 예외 누락이다. Unix socket EPERM은 환경 실패로 실제 허용 환경 `/tmp/u17-socket001.log` 1 PASS를 확보했다.

G11은 이전 manifest/profile/원시 Return 증거를 보존하고 U17의 새 콘텐츠 pack+manifest로 현재 스키마를 검증한다. profile 3종은 byte-equal 복사하며 현재 공식 정의에서 같은 자산·형태를 restore한다. U08/U09 fixture의 5개 equipment 교체 호출은 재료·분쇄·강화 recipe를 유지한다. HQ 시험은 본래 목적(깊이5 모집거절+상점3종 구매)을 유지하고 HQ pool이 직접 3종을 작성하게 수정한다. 정책 검사·수용 조건은 완화하지 않는다.

공개 pilot012–019의 원본 로그는 /tmp/u17-public-pilotNNN.log에 보존한다. 제품 결함 두 건(stage choice를 catalog-only로 검증, CombatResult summary와 선택보상 불일치)을 교정한 뒤, ClaimReward의 명시 호출·소진 Continue·분쇄 recipe·사거리·예약별 노드·실제 타임아웃 패배 준비를 교정했다. 최종 full run에서 모든 U17 시험이 통과했으며 native 한도 옵션은 별도로 실행한다.

## 회귀 교정 종료·native 경계·host 입력 고정

집중 회귀002:36선택/33PASS/3FAIL. checkpoint field는 부화를 별도 최상위 키로 잘못 기대한 테스트 오류(실제 owner는 encounter_progression.revival)였고, G11 나머지1건은 폐기된 Monolith HashMap의 unknown-history 거절 전제였다. 해당 최상위 키 추가를 되돌리고 사라진 owner 전용 거절 assertion을 제거했다. 현재 entry/checkpoint의 exact/one-short/retry 검사는 모두 유지했다. 회귀003 3PASS, ron 보상예외1PASS. 전체 실행의37Core+1RON 실패는 socket 실제환경1PASS,회귀00233PASS,회귀0033PASS,RON1PASS로 모두 닫혔다. 단일 full-green 실행이라고 하지 않는다.

현재 public export020:1PASS(두 seed의 실제 첫 보스→2/3단계 선택), 26source+bundle 새normal_inputs001에 복사. native compile001 exit0 5m48s(초기 Cargo filesystem 탐색 대기, 컴파일 오류 없음), native001 28PASS/0FAIL/0ignore(U17 24+G11 4). 부화 작업량/clone 정확경계 성공,1short 두재시도 전체 roots불변,정상재시도 성공.

normal_build001 release exit0 6m19s; instrumented_build001 release exit0 8m04s. 서로 다른 target을 사용하고 입력 고정·메모리 여유를 확인했다. cost diagnostic build001 exit0 1m18s. current prepared sources 이후 제품/시험 소스 수정 없음. 새host Python 진행기·입력은 독립 artifact이며 historical source/receipt를 덮어쓰지 않는다.

확장 fixture001 준비는 id: substring이 uuid:까지 세는 도구 assertion으로 중단; 원본 디렉터리를 보존하고002에서 필드 들여쓰기를 포함해 교정했다. stage별64독립장비 후보, 기존3후보 대비유한확장; 전투/XP/보상량/촉매/기본한도 불변. debug package/load actual current validation 각각 exit0. 실제 승인 host packaging/calibration은 별도 현재 release 바이너드로 수행한다.

## 승인 host 준비와 pilot

normal packaging/calibration/review/adoption 모두 exit0. 기존 승인 파일을 재사용하지 않고 현재 source/compiler/dependency graph·worker 바이너드·콘텐츠·suite를 새issuer에서 결속했다. finite admission suite는10s battle warmup이며 U17 전체 기능 통과와 구분한다.

host pilot001은47명령 뒤 tactical preview에서PositionOccupied로종료(exit1, /tmp/u17-boss-pilot-t26knt0c). 진행기는 배치 직원만 점유자로 취급했고 실제 public checkpoint에는 거점 Player unit이 추가로 존재했다. 모든 살아있는 unit 칸을 제외하고 현재 사거리보다 유효 타격수가 늘 때만 이동하는 전략으로 교정했다. pilot002는48명령/4.35s/exit0(/tmp/u17-boss-pilot-1arllv5s), 실제CombatResultVictory·파편1·Stage2알1 확인. 제품·작성 입력·합격 조건은 불변이다.

## 실제 fresh Continue 기본 한도 실패와 수명 교정 계획

normal full_flow003 (/tmp/u17-full-flow-x0wzsqib)333operations/19.46s/exit0, expanded full_flow001 (/tmp/u17-full-flow-g_g4raxj)333operations/exit0. 3→64후보, 일반/엘리트/분쇄후반복/보스/2회후퇴/2·3단계선택/claim중복거절 완료. full_flow001/002는 배치에도 거점점유가 있다는 준비오류와 tactical preview DTO가 payload 바로 아래라는 진행기오류였다. 실제trace와실패원본 보존, 제품변경없음.

fresh_readers001 첫 정상 저장복원에서 실제 `battle_runtime_retained_byte_limit`, site=relic_continue_value, requested19556864>16777216. active_run file163495bytes, Corepayload121853bytes. current에Completed slot5/4/8/1의 미리보기4개+미해결6 하나, SavePoint에도Completed4개가 남아preview JSON합계86301bytes(공백 포함)를 소유했다. 소비 완료 노드의 preview는 Select/Enter대상이 아니며 종료결과/선택권은 이미별도durable owner가 보유한다. 노드 완료 시 이 도달불가 cache를 해제하는 관련 수명교정으로 복원입력을 줄인다. native한도·allocation공식·작성입력·성공조건은 변경하지 않는다. 먼저 실제public행동4전투+상점+SavePoint+hatch 후 CoreApplication fresh Continue의default거절을회귀시험으로재현한다.

io_replay001은0operations에서공유issuer cache.lock OS11로admissionheld. freshreader와같은issuer를동시에연것이원인,게임저장IO실패가아님. 이후동일issuer host실행은순차화한다. 기존스토어/trace/실패stdout stderr불변. source교정뒤 새build/issuer/trace로실제IO와freshreader를재실행한다.

## 현재 제품 교정과 새 release 검증 종료

public-restore-repro001은 동일 정상4전투/상점/SavePoint/hatch에서 기본16MiB 거절을 재현했다(1FAIL).
완료/패배 node의 combat_previews 해제 두 지점을 수정했다. preview-retirement001 223PASS와 native002
35PASS(U17 25+G11/U08 10), normal_build002 6m14s/instrumented_build002 6m57s exit0. 이후 제품 입력
변경 없이 새 packaging/calibration/adoption을 normal/expanded 각각 실행해 기존 한도로 승인받았다.
현재 normal fullflow004와 expanded fullflow002는 각각333명령 PASS,17개 실제 저장 milestone이다.
입장전 부화 Corepayload는121853→58032bytes로 줄었고 Completed preview가 제거됐다.

fresh_readers002는 첫 부화 복원 성공 뒤 retreat1 map 비교에서 실패했다. 실제 차이는 입장비 rollback에
따른 entry_preview 억제력/비용뿐이며 최신 results-and-save 정책과 일치했다. 기대 map을 실제 입장 전
Stage2_hatched map 전체로 대조하도록 교정했다. fresh_readers003은14시나리오/15개 실제 프로세스 PASS.
결과/선택대기/선택/claim, 미완료 일반전 입장→후퇴→부화 rollback, 실제 부화 전후 SavePoint, 같은층
이동을 확인했다. 실제 SavePoint load/complete의 IO 실패·재시도도 포함한다.

io_replay002는216연산 뒤 구매에서 IO 장애가 발생하지 않아 진행기 assertion 실패(exit101). 구매는
미완료 상점 내부 상태이며 기존 confirmed Continue가 유지된다. 실제 durable boundary인 ExitShop으로
주입을 옮겼다. io_replay003은333원명령+17실제 OS21 실패/동일재시도+2중복부화거절=352연산 PASS.
Core 전체 roots 및 active/profile bytes가 실패 전후 같고 각 성공 재시도가 원래 정상trace와 일치했다.

## 비용 측정·측정기 교정

cost001 정상 forward3회는 각333연산과 정확 heap 회수 PASS. 첫 Continue는 복원/공개값 일치에
성공했으나 측정기의 반환값 보유로 회수량 비교가 실패했다. restore probe002는 반환값31,911bytes를
해제하고 남은128bytes가 data loading을 포함한 초기화 경계에 있음을 분리했다. probe003은 forward와
같은 loaded-content baseline에서 app/반환값을 해제해 정확 회수했고 로더 전/데이터 해제 후128byte
차이도 별도 관측에 남겼다. 제품 한도나 합격값을 늘리지 않았다. 이전 결과/바이너리는 diagnostic002/003,
현재는 diagnostic004이며 forward binary는 byte-identical이다.

cost002는 정상 forward3회 비영향 재사용+새 Continue6회/확장 forward3회 모두 PASS. 일반/64후보에서
333명령 Core 처리 합1.29–1.33초, 최대 단일 명령30.7–39.7ms, pre-app 대비heap peak 약4.17–4.19MB.
Continue59–63ms, heap window 약904KB, native 최대단일요청2,844,710bytes/누적6,287,138bytes,
work1,481,111. 모든 게임 app/반환값의 종료후delta0. dev opt0 진단이며 실제 release 승인과 구분한다.
trace 전체를 파싱한 약400MB baseline과 OS RSS도 원문에 기록하며 게임 단독 메모리로 표시하지 않는다.
기본16MiB와 admission/work limits를 유지한다. COST_REVIEW.json이 정확한3회별 수치를 소유한다.

현재 원문은 runs/current001 및 raw_logs에, 이전 host/cost 실패는 historical tar에 보존했다.
22수용/10차원/6IVA/3실행node 결속을 기계 검사했다. canonical 문서의 남아 있던 V14/V12 현재지원
문구를 실제V17 및 구형명시거절로 정정했다. 제품·시험 재실행이 필요한 동작 변경은 없다.

최종화 preflight001은 외부 normal_host_tests.rs를 오래된 U16-4 target과 비교해 중단했다. archive 생성 전이며 제품 오류가 아니다. 해당 파일은 U17 이전 서버시험 교정에 포함되어 최신 U16 종합감사 ENTRY와 일치한다. 최신 유효 baseline을 우선하는 비교로 교정했다. 빈 미발행 target과 실패로그를 보존하고 실제 snapshot은 다음 실행에서 한 번 생성한다.
