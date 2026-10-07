# GOAL-CMRP-01 실행 — U07-M

현재 위치 — GEX10 자체검증 완료, exact target 동결 및 독립 GEX20 인계. 제품은398파일/f2144e9a4c5ddeb41245799294a75b41de9d843018cb2166f711b7f07821cda8이다. PROFILE→ACCOUNTING→COLLECTOR→PLAYBACK→VERIFY 5개 node,10acceptance/10필수차원,181선행조건+별도31R06을 실제 증거로 연결했다. finalization은 selection001→predecessor_impact001→adaptation002→coverage002→gex10001 모두exit0다. 독립 Complete/High/open0는 아직 주장하지 않는다.

Mode goal-run. Exact GS-CMRP-001/GOAL-CMRP-01/PHASE-CMRP-01, FPF-CMRP-001/PP-CMRP-001/PVR-CMRP-001/HA-CMRP-001은 [P2G60](../../P2G_60_HANDOFF_001.json)의 exact chain을 따른다. P2G2026-10-04-r8/GEX2026-10-04-gex-r8/quality2/usagev2.1.3을 적용한다. 현 연속 사용자 요청은 P2G 이후 구현·자체검증·비참여 독립감사·finding교정·다음eligible작업까지 별도로 승인했다. 단계 인계의 재승인을 요구하지 않는다.

## 목적과 완료 경계

배포 기본8MiB/hash/오류/기존guard를 보존하면서 trusted finite개발 profile과 정상Core/host에서 실제 동작하는 bounded 메모리 측정·재현을 완성한다. warning과 emergency cap을 분리하고 operation accounting·실제allocator·OS RSS의 서로 다른 scope를 원시표본/요약에서 드러낸다. V01–10/10필수차원/181선행조건+별도31R06 적용성을 current 증거와 독립 Complete/High/open0로 닫으면 U08로 계속한다. 임시32MiB는 측정창이며 최종 배포값/전종·Unity플레이 입증은 U30-C/P 소유다. 단일phase라 Master GEX30은 없다.

## 역할과 기존 상태 인수

root는 계획·설계·진행조정/문서만 수행한다. 제품코드·fmt/build/test·실행도구·finding교정은 Sol/xHigh 단독 writer다. 기존 `/root/u07_independent_audit` Astra/High는 U07-M 계획·설계·조사에 참여하지 않았으며 새target동결 뒤 전체 GEX20을 담당할 수 있다. 감사 중 input mutation은 정지한다. 모델 실패 시 임의 대체하거나 지정모델이라고 주장하지 않는다.

U07 target002/audit002의 Complete/High/open0 closure SHA는 `d371f43c1122826347261ed68f0817e66432bbc97e85fdb6759deaf686a59cb9`다. 선행 product389파일 hash `886d427882319b3147260f394f1204acf903af654f772709bdaaa045f1c62146`; 새 작업은 기존변경을 인수하고 reset/clean/stash하지 않는다. PREDECESSOR_INTAKE_001은7143members를 대조했으며 두roadmap publicationdelta만 별도로 결속했다. closed U07 target/감사/receipt/실패·철회된HP0조사는 불변이력이다.

구현자는 첫 mutation 전에 새 goal namespace 안에 PRODUCT_BEFORE/SOURCE_BEFORE manifest+archive, 사용될 실제 검증입력·tool원본, git상태/실행process 인수를 기록한다. source capture81은 정적검토 입력이고 fullproduct baseline 대체가 아니다. 이전 artifact생성script를 그대로 재실행하지 않는다. 새 runner는 고유label/create-new 출력과 command/toolchain/source/input/rawexit/receipt를 결속한다. frozen U07/U06 archive는 exact참조해 재귀 중첩하지 않는다. 초기 전체test는 선행 exact성공을 영향검토로 재사용하고 단계시작만으로 반복하지 않는다.

## 구현서와 실행 순서

권위 설계는 [CODE_REVIEW_001](../../CODE_REVIEW_001.ko.md)의 Current Actual, TD01–06, Element Coverage다. 아래 상세는 같은 설계를 실행에 결속하며 의미를 변경하지 않는다. 순서는 PROFILE→ACCOUNTING→COLLECTOR→PLAYBACK→VERIFY다.

### PROFILE / TD01–02

`RuntimeSafetySettings`에 optional revision1 development_measurement/emergency_retained_bytes를 strict 추가한다. None canonical생략으로 기존raw/effectivehash·embedded파일 유지. 원래 DTO/native.validate는8MiB; emergency는 nominalwarning보다크고32MiB이하만 허용한다. `native_runtime()`는 effective ownerlimits를 돌려주며 settings는 nominal/warning/identity를 따로 소유한다. perbattleinline limits11필드/ENGINE_MAX/공개validate는 바꾸지 않는다.

private `run_with_profile(owner_limits, settings, closure)`는 owner값을 재설정하지 않고 nonretained필드를 기존validate로, retained는0<owner<=trustedselectedcap으로 검증한다. 공개 `with_runtime_fact_limits`가 저장한 작은cap은 개발모드도 그대로 지킨다. sim6entry/runtime_transaction2/Worldactive·pending·deferred와 directowner/result checks를 대조한다. nested는 existingBudget을 재사용하고fault를poll한다. battle전 deferred이후 최초budget은 실제immutableGameData설정을 사용한다.

추가 실제 seam: scope밖 `runtime_fact_snapshot`의 localallocation은 현재 inactive fallback8MiB가 있다. private helper에 이미 검증된ownercap을 전달하되 activeBudget우선과 같은누적admission을 유지한다. default projection에 새seed/extra work나전체operation계량을 추가하지 않는다. 반면 persistence rollback capture/resourceclone의 outside-native계량은 기존비활성범위다. 기본보호/비용을 몰래바꾸지 말고 native scope/missing과 processheap포함을 문서화한다. `StartNewGame` generation은NotStarted draft에서교체되며livebattle전환이아니다. rollback은기존generation을복원한다. metadata는 실제그때의generation/settings를 읽어옛profile로샘플을오표기하지않는다.

첫 작은검증: parserpositive/negative/defaultcanonical·effectivehash; actual8/16/32MiB와scaled경계; warningexact/oneover·hardexact/oneover·owneroverride; no-content/saveauthority; directsnapshot/confirmedresult/pending/deferred; work/clone/expression/overflow여전히거절. 기본PC는 비싼full통합전에 재측정해새거절/비용회귀를먼저교정한다.

### ACCOUNTING / TD02–03

Budget과scope종료publication을재사용한다. observationfeature고정크기seedattempted/accepted, acceptedgrowth/total, maxcumulativerequest/maxownerrequest, firstsoft/hardsite(kind/request/limit)를넣는다. 기존 `admitted_bytes`의 failedseed attempted의미를 바꾸지않는다. soft는독립 `>`기록이고hardfaultlatch→무시/clear는금지다. allocation이허용되지않은요청을accepted로세지않고overflow상태를정상최대숫자로가장하지않는다.

currentextent는committedowner, peak는계량된범위의max다. failedcandidate/proposedpeak와현재값을분리한다. nativeoutside범위/불완전scan은지원상태로표시한다. metrics는새gameplaystate아니며CloneShape/retainedcost에고정metadata를끼워넣지않는다. 같은operation시작extent+양의growth누적과중첩/rollback/해제비환급은보존한다. sticky emergency/기타fault는candidate복원·U03세션종료/원본저장보호로전달한다.

### COLLECTOR / TD04

Core는OS/fileIO/globalallocator없음. host optionalruntime-measurementfeature와module이allocator/RSS/window를소유하며defaultfeature는변경하지않는다. 실제binary당globalSystemwrapper하나,library는global선언없음. 시작부터연속live/count로창이전allocation의free까지정확히추적한다. hook내alloc/log/lock금지. failedrealloc/zeroed/crosswindowfree/concurrentthread와baseline/windowpeak/end/signeddelta를시험한다. processwide요청bytes이지sessionheap/RSS가아니다. Linuxbounded/proc VmRSS/VmHWM(lifetime)만OS표본으로읽고미지원/실패는명시missing이다.

queue1024/batch128/ring300재사용; liveobservedregistry/worker/detail256, rawoperationring300, fixedhash/seed·boundedmetadata를연결한다. 포화는관측drop이지gameplayfailure가아니다. identity/site는unboundedmetriclabel로추가하지않는다. weaksessionretirement와종료후release, outputsample/filecap/truncation/count분포를입증한다.

### PLAYBACK / TD05

권장 신규binary `game_server/src/bin/measure_runtime.rs`는externalpack+trustedsettings를받고expectedprofilehash를첫mutation전에verify한뒤공개CoreApplicationcommand/admin/tick/checkpoint를실행한다. normal/none/detached/full모드와유한step/repeat/session/sample/output를지원한다(1MiBmanifest/10000steps/64repeat/16session/4096sample/16MiBexport engineering상한). outputcreate-new·bounded writer. private state주입만으로성공판정하지않는다. 표본요약mean/p95/max는actualretainedsamples와count/drop을분리한다.

별도실제game_server/WebSocket/actor/attach/loopbackdetail표본을만든다. build/content/profile/rawinput/seed/deviceOS/feature/concurrency/window/조합규모/정상·typed거절·중단을묶는다. normal전투·U07objects/벽/함정/field와반복만료/해제·Pause/reconnect·liveclone/현재hostContinue/SavePoint·동시profileisolation을현재지원최소시나리오로검증한다. mismatch은측정replay입구거절이며일반저장복원에새profile권한을주지않는다.

### VERIFY / TD06

10정책acceptance/10차원와181선행+31R06각각 reuse/affected/명시dev차이/currentevidence를작성한다. 최종Coredefault/fixtures/observation,hostdefault/observation/measurement+heap,fmt/check/event_log/clippy/release/public/numeric/native/fault/persistence/allocator를영향에맞춰실행한다. 기존sharedcharge→native전달을확인해중복계량하지않는다. 기능workers끝난뒤PC/R05. default576/36조건31성공5거절·code/site/cap/gameplay/oracles유지; 개발동일입력비교는별도표본이고모든옛거절성공을요구하지않는다. R05원래peak-only경고/raw101외새hardfail허용없음.

currentselection→predecessorimpact→harnessadaptation→coverage→GEX10→PLAN/NOTES완결→exacttarget/fuzz0정역복원→비참여독립whole-goalGEX20. 최종화도구작성과실제실행을구분한다. freeze stdout/receipt는동결membership밖에서기록해자기변경을피한다. 실패finding교정은기존Sol,재감사는독립성을유지한Astra다. CompleteHigh/open0후U07-Mclosure와roadmap게시·U08계속.

## 자율성·재검토·문서

일반helper분리/타입명/fixture/runnerlabel/compile오류·bounded더작은collector상한은구현자자율이며NOTES에이유/영향을남긴다. defaultpolicy/publiccontract/stateowner/누적의미/완료기준변경또는핵심설계반증은코드/영향/대안을root에반환한다. 이는자동사용자승인대기가아니다. 이연속요청이끝날때까지단계완료보고만으로종료하지않는다.

canonicalruntime계약과measurement사용문서/hostREADME를실제포트에맞춘다. 후속수치조정은새profile/revision, 원시전후값/이유/영향/독립인수로기록하며U08새owner/U11identity/U30-Ccore표본/U30-PUnityplay/U31최종인수책임을남긴다. 사용자중지·승인전체범위완료·현재권한환경의해결불가능실제장애만최종중단조건이다.


## GEX10 최종 자체검증·감사 인계 경계 — 2026-10-05 UTC

실행값과 exact 재사용은 [EVIDENCE_SELECTION.json](EVIDENCE_SELECTION.json)의54receipt bindings가 구분한다. 실제CLI003은4bf 원소스/원출력이고 server-main 한파일 비영향 및 hostREADME 한파일 docs_delta proof로 current에 연결했다. integrated001/host002/lifecycle003의2e7 원실행도 docs-only proof로 연결한다. 신규 public/release/numeric/fault/measurementheap/PC/R05는 f214를 직접 실행했다. 원래 실패·raw·첫 측정값을 성공 label로 바꾸거나 덮어쓰지 않았다.

Core default1875/fixtures1875/observation1892,host default93/observation103/measurement122 PASS 및 check/event_log/clippy/fmt/current measurementheap PASS다. 관측 heap의 ignored row는 실제 isolated heap receipt와 분리해 연결한다. 공개16단계·objects/combat·원래 숫자 guard·fault5mode/realWebSocket PASS다. actualCLI26case/host4case×2phase/lifecycle10행 및 낮은 engineering256KiB strict-hard/dev16-soft를 raw/input/binary/profile/seed에 결속했다. 8KiB loader 거절·13KiB 무초과·CLI4typed authority/context 거절은 budget초과 성공 근거가 아니다.

최종PC002는576행/36조건31성공5거절,immediate U07 시간외 모든 필드exact/allowed_delta0와17negative PASS다. 역사U06+200은 상속된 emptyowner152+Noneplacement48의 보조 guard다. 최종R05002 raw101은 기존 concurrent peak-only warning(excess638972bytes,peak9669160/cap9030188)이며 다른 hard guard 실패0다. summary와 fresh-cache actualdepinfo는 baseline/lock/cap/workload 및 currentfinalbbbae/historicalinclude64deb exactbytes를 결속한다. timing값과 원시101을 숨기지 않는다.

[PREDECESSOR_IMPACT.json](PREDECESSOR_IMPACT.json)181+31, [ADAPTATION_FINAL.json](compatibility/current_001/ADAPTATION_FINAL.json)448 exact inputs, [COVERAGE.json](COVERAGE.json)10acceptance/10필수차원, [GEX10_RESULT.json](GEX10_RESULT.json)5node를 최종 자체검증으로 남겼다. adaptation001 None행 tool-error와 dependent coverage001 실패 원본도 보존하고 adaptation002/coverage002만 최종 실행으로 선택한다. V10은 goal README/canonical 계약/host README/상태 인벤토리의 실제 측정·수치변경·후속 owner 절차로 연결한다.

이 문서·NOTES·EXPERIMENTS의 최종 바이트를 유지한 채 direct freeze는 target/u07m_001에서 SOURCE_BEFORE/AFTER,RECORDS,REFERENCES의 실제 member bytes와 fuzz0 정역 복원을 확인한다. 이전 U07 target002의4archive는 외부 exact참조로 확인하며 재귀 중첩하지 않는다. 실행stdout/stderr/exit/MANIFESThash·postprocess확인은 repository .cache/u07m-freeze-001/EXECUTION.json 및 같은 폴더 로그에 두므로 동결 RECORDS를 자기포함하지 않는다. freeze 성공 뒤 제품·도구·시험·문서 입력을 모두 mutation STOP하며 비참여 Astra/High가 전체 GEX20을 인수한다. 동결의 실제 성공/manifest membership은 target MANIFEST와 외부 EXECUTION receipt가 권위이며 이 사전 인계 기록은 실행 성공을 대신하지 않는다. U08은 독립 Complete/High/open0 뒤 root가 선택하며 현재 구현자는 시작하지 않는다.
