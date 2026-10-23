# 실행 기록

P2G40 구조검증과P2G50 Goal Quality2검증 passed. 기능시험0. source baseline 보관/검증 후 GEX10 구현. 실제 명령·원시실행·실패/교정은 이 기록에 추가한다.

## 구현 pilot001–004

- `/tmp/u162-check001.log`, `runs/check001.log`: cargo check exit101, 새session의 `[Uuid;2]` CloneShape 누락. 모든session필드는inline Copy이므로 session copy_shape로교정. 실제기능실패가아닌컴파일오류.
- `runs/build002.log`: lib시험빌드 exit0(1m50s). 해당시점기능시험0.
- `runs/focused003.log`: cargo test -p game_core --lib --offline --locked u162_ -- --nocapture,6통과/0실패/0ignore/2122filtered,28.50s. 정상authoredmap/RON과routedcommand, setup grant는내부fixture.
- `/tmp/u162-focused004.log`: 추가동일종류두사본/퇴장확정/새인스턴스역순재시도포함8통과/0실패/0ignore/2122filtered,30.36s. fresh 인스턴스는별도process 증거아님.
- `normal_build001`, `instrumented_build001`, `runs/core005.*` 진행. 입력고정. core005는현재공유type/loader/reward/명령변경의전체영향통합1회이며이후오류/새수정은영향범위만다시검증한다.

## 정상host 준비

`test_parts/normal_inputs001/source`는기존지원외부pack에3활동예약/등급별4유물씩/후보가중치1/가루임계치3,9,18과Event정상grant를추가했다. 기존u08_probe효과와침식penalty를대표fixture로재사용하며공식콘텐츠채택아님. 기존 Event→SavePoint 준비와작성배치를유지하고8단순선택→9합성→6가루투자경로를사용한다. calibration suite는기존24step battle준비/10clock과새17step Event준비/36step활동을구별한다. 기존approval은재사용하지않고현빌드/현pack에서재calibration/adoption한다. 원시명령/결과는run.py가exclusive생성한다.

## 통합 실패 진단과 교정005–009

`runs/core005.log` 전체 lib: 2114통과/10실패/6명시ignore,501.94s. all-targets는 lib실패에서 중단했으므로 전체통과가 아니다. 실패10개는 다음과 같이 분리했다.

- 세 command exhaustive test는 새3명령을 이미 사례에 넣었지만 개수81을 유지했다.84로교정.
- 두 legacy reward-pool fixture는 강제 Reward node에 생성된 facility `reward_activity`가 남아 새 활동으로 해석되었다. 그두fixture만metadata를None으로설정하여 원래 외부 Remove/보상사본 owner 검증을 유지했다. 현재 공식 새활동은u162/u13 별도 검사한다.
- G11네검사는 새default `relic_rewards`가 normalized catalog identity에 반영되어 기존기대identity와불일치했다. `runs/g11_identity001.json`으로 현identity와동일source_receipt를확인하고 원본source/profile/trace/limits를그대로둔 `test_parts/g11_inputs001/manifest.json` successor를만들었다. `runs/restore008.log`: 환경변수로명시선택,4통과/0실패,5.97s. 과거immutable기록은수정하지않았다.
- Unix socket1은 sandbox EPERM이며 도메인실패가아니다. snapshot/socket제품·시험소스비영향과이전 실제허용환경결과를대조하여별도재사용판정한다.

`runs/focused006.log`는cargo에복수filter를잘못전달하여명령파싱실패(시험0). 복수harnessfilter를`--`뒤로교정한 `runs/focused007.log`는22통과/0실패/0ignore,38.12s. 교정5개와새활동8개,시설9개를포함한다. `runs/integration009.log`는8target 합계72통과/0실패/ignore0; cargo의lib실패로실행되지않았던통합을별도실행했다.

## 정상입력·빌드 준비 실패의 구분

package001은data/bundle.ron누락으로거절,이후잘못실행된calibration001은pack.ron없음으로실패했다. 원본보존 후normal_inputs002에정상bundle포함,package002통과. calibration002는실제lab.prepare assertion실패. `runs/preparation_pilot001.json` 실제Core공개명령7개로slot1이HeadquartersContact임을확인했다. 추가예약이allocator추첨을바꾸었으므로normal_inputs003은slot1에기존ordinary전투를명시예약한다. 기존battle준비/10clock/모든assertion과한도를유지한다. 새활동의배정종류/입장/비용assertion도유지한다.

`runs/authoring001`: valid1과잘못된입력9개는예상대로처리했으나custom activity의reward_activity+Shop조합1개를허용했다. 실제지원작성경로의제품검증누락이다. LiveRon이모든activity의Reward category를검사하도록교정했다. 최종패키저로재검증한다.

`normal_build002`/`instrumented_build002`는기존offline Cargo cache환경누락으로fs2찾기실패(빌드0). 이어진동일exclusive출력경로재사용시도도거절(빌드0). 실패로그/tmp/u162-*-build002/003보존. 정상CARGO_HOME과새exclusive `normal_build003`/`instrumented_build003`에서최종빌드한다. 제품교정과무관한runner실패를통과로세지않는다. `/tmp/u162-diagnostic-build002.log`는현재공개Core비용진단빌드통과,정상승인권한없음.

`test_parts/host_flow.py`는Event→SavePoint의정상입력에서세활동을각각freshprocess 미완료/완료Continue로검증한다. 선택/투자/합성의실제OS21실패는메모리와두원본파일동일성을검사한뒤동일명령재시도한다. SavePoint,지불후미이용종료도별도확인한다. `expanded_inputs001`은후보300종+정상Event추가64사본,동일runtime/host한도로준비한다. 실행결과가나오기전까지준비는PASS가아니다.

## 최종 입력 준비·calibration004–007

package003은6예약 뒤남은3자리로support2+event/reward를실현할수없어거절했다. normal_inputs004에서지원활동수0/1(40/60)로작성하고6필수예약/전투/모든assertion은보존했다. package004와공개preparation_pilot002 통과(slot1Combat/Available); battle_pilot001은실제InBattle진입을확인했다.

calibration004는Trial준비가전투상태로끝나야한다는계약을활동시나리오가충족하지않아거절. calibration005는60step 결합준비의receiptwork10M초과. calibration006은활동별37/44/43step으로분리했으나coverage포함fusion이한도초과. `runs/prepare_diagnostic001.json`에서coverage없는4case는모두준비가능하며fusion work9959313으로한도에근접함을확인했다. calibration007은활동필드만읽는동일관측을기존SelectedEvent snapshot으로바꾸고모든연산/assertion/전투/10clock을보존했다. 정상4case×3repeat×plain/instrumented의교정/기준검토/채택통과. raw는normal_calibration007/normal_limit_review007/normal_adoption007, 실제기본profile과최종복사binary에귀속한다.

`normal_build003` exit0 6m21s; `instrumented_build003` exit0 6m04s. 이후제품코드변경없음. G11 cfg(test) 기본manifest문자열만successor로정합화했고 `default_restore010`4/4통과(환경변수없이5.69s). 당시compile preimage는test_parts/preimages, 이비영향차이는BASELINE_REUSE에명시했다. 전체testsourcefingerprint가새build에서달라질수있으므로기존binary승인을새binary로옮기지않는다.

## 실제host·작성·비용

- `runs/authoring002/RESULT.json`: valid1/invalid10 모두예상대로처리. 잘못된category누락검증을포함한다.
- `runs/host001/RESULT.json`:18fresh process67명령,OS21 6번. 세활동미완료/완료Continue,SavePoint,최고/일반등급수령,투자후미이용완료를검증했다. raw진행기오류:0가루스택생략을0으로해석하도록교정; 투자중간publication가정실패는실제Claim경계로교정; 미이용RelicState의잔여가루15를기대하도록교정. 실패원문/preimage/실패store는보존,유효phase는재실행하지않고기존결과를인수했다.
- `runs/host_edges001/RESULT.json`:2fresh process11명령. 실제CorruptRelic→역순입력합성→Normal결과→completedContinue. 같은입력사본의기존Normal입력결과와출력UUID/정의가동일하다.
- `expanded_calibration001`은300후보/추가64사본을기본authoring10M GeneratedSetup의queued_fact_source10000014에서거절했다. 실패를성공으로세지않았다. 동일입력을유지하고별도trusted authoring64M 측정profile로선택한expanded_calibration002/limit_review002/adoption002가통과했다. native16MiB/10M과제품config는그대로다. `runs/expanded_host001`:3fresh process29명령/IO2,세활동원자확정·완료복원·SavePoint. 기본authoring승인주장없음.
- `runs/activity_cost001`:정상/확장각3개fresh표본,26명령+26fullRun조회=총312창. 별도6개준비pilot. 실제시간/allocatorpeak/RSS/nativework·누적admission을구분하고app/trace해제뒤모든표본의live requested bytes정확회수를확인했다. expanded본래300후보/67초기사본을줄이지않았다. 공개Core진단이며정상host승인을대신하지않는다.
- `runs/shared_cost001`:기존U16-1과동일정상12+12/확장768+768 vault프로필,12공개연산×6fresh표본=72창. 원본profile/source_receipt동일,현재정규화identity만successor. 종료후모든표본정확회수,상세before/after는COST_REVIEW. native가비활성인공통연산의null계측값을0이나CPU로해석하지않는다.
- 요약도구초기regex는`- should panic`12개를누락해집계assertion이실패했다. raw test결과를대조해parser만교정,summary002 exit0:기본Core고유2196,기존ignore6,알려진현재실패0. 한꺼번에all-targets가green이었다고기록하지않는다. native4/server31/socket1 재사용과현재실행을구분했다.

완료판정전검토는IMPLEMENTATION_REVIEW/RESTORE_MEMORY_REVIEW,17수용/10차원/5IVA/2실행node는COVERAGE,원시원문과전후비용은TEST_RESULTS_INDEX/BASELINE_REUSE/COST_REVIEW에연결했다. final target 동결과격리정역복원성공뒤GEX10결과/최신인계만추가하며감사는자동실행하지않는다.

## 재현 진입점

기능 집중 검사: `PATH=/home/blast/.cargo/bin:$PATH CARGO_HOME=/tmp/wp02-cargo-home CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/u12-vault-cost-target cargo test -p game_core --lib --offline --locked -- u162_ u13_facility --test-threads=8`.
기본 G11: 같은 환경에서 `cargo test -p game_core --lib --offline --locked u12_vault_ -- --test-threads=4`.
실제host의argv/입력/원본store별경로는각 `runs/host_*.result.json`과 `test_parts/host_replays001`에있다. 실행기는exclusive출력과실제store를쓰기때문에재현시새 /tmp store와출력명을사용한다. 완결26명령은`test_parts/activity_cost_inputs001/normal.0.json`과expanded.0.json. 승인config는normal_inputs007/admission.json,확장측정은expanded_inputs002/admission.json이다. 새환경에서는자기issuer로현재binary를재calibration/adoption하며privatekey는증거archive에넣지않는다.
