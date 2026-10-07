# U07-M 상태·수명 인벤토리

| Owner | 상태·수명 | 실패·복원·계량 |
| --- | --- | --- |
| RuntimeSafetySettings / immutableGameDataArc | nominal/native effective limits, dev revision/emergency, raw/effective identity; generation수명 | trustedload검증; content/save권한없음; defaulthash불변 |
| RuntimeFactState / RunResourceResult | 기존11필드ownerlimits; battle/pendingresult수명 | privateprofileentry검증; 작은publicoverride유지; clone/transfer기존경계 |
| TLSBudget / deferredowner | operationseed+growth/stickyfault; outerpublicoperation | nested/rollback비환급; scope끝publication후제거 |
| observer accumulator/weaklease | fixed attempted/accepted/extent/site·commitmetadata; operation/session | failedcandidate와current분리; 없는collector동등; gameplay계량에혼합안함 |
| host collector | boundedregistry256/queue1024/batch128/ring300와rawsample; collector수명 | drop/missing명시; weakretire; game/application을붙잡지않음 |
| host measurement allocator/RSS | processwide연속live/count·window·OSsample | hook무할당/무로그; processscope/unsupported명시; noadmissionfeedback |
| bounded replay/export | input/identity/step·sample4096/output16MiB; 한실행 | expectedhashbeforemutation; create-new; failure/truncation기록; normalhost별도검증 |

실제 구현은 아래 source owner로 결속된다. 이 metadata는 게임 상태의 새 소유자가 아니며 원래 retained/clone layout에 포함하지 않는다.

| 실제 파일/심볼 | 실행·실패·계량 경계 |
| --- | --- |
| runtime_safety.rs::RuntimeSafetySettings, DevelopmentMeasurementProfile | strict originalDTO8MiB → optionalrevision1≤32MiB → immutable effective owner limits; None canonicaldefaulthash exact |
| battle/runtime_control.rs::run_with_profile, admit_retained_growth_with_owner_limits | outer budget 재사용; private trusted cap검증; outside snapshot은 trusted ownercap, active budget 우선; 공개raw runstrict8MiB |
| battle/core/runtime_facts.rs::run_profile_budget | 최초 inactive scope에만 nominal warning설치; rawouter/nestedwarningunknown 보존; 기본cost unchanged |
| runtime_observation/retained.rs::RetainedAccounting | first seedattempt/accepted, acceptedgrowth/total, cumulative/ownermax 및 firstsoft/hard; overflow/multiplepolicyexplicit |
| runtime_observation/identity.rs::MeasurementIdentity, NativeOwnerExtent | fixed32bytehash·seed·revision·warning·selectedcap; unmeasured/noowner/battle/pending/both typedscope; failedcandidate≠committedcurrent |
| application.rs/runtime_observation.rs prepare/commit publication | 실제generation과committedowner identity; discard/rollback/controlledfork lease분리; World기존finalpendingextent 재사용 |
| game_server/observability.rs bounded registry/rawoperation |256weak session/300raw/1024queue/128batch; end원시결과보존/weakretire; 미관측·detached·full 비간섭 |
| runtime_measurement/allocator.rs::CountingAllocator, begin_process_window, process_active_window | hookcontinuous ledger; register beforelive→oldoddpublish→retire→nextbegin; bounded close/opening unavailable; processwide only |
| runtime_measurement/rss.rs::process_rss | bounded64KiBLinuxstatus; actualVmRSS/VmHWMbytes, unavailable/unsupportedseparate; noheapfallback |
| runtime_measurement/playback.rs::execute, Manifest; playback/script.rs | allprofilegatesbeforeCore/content/output; ≤10000weightedcalls,16sessions,64repeats,4096samples,16MiBfile,64KiBrecord/footer; publichistory256/1MiBencodedJSON andbinding64 |
| game_server/src/main.rs common shutdown_result | serverfuture/signal both preserveResult, dropobserver→windowfooter; abruptkill unsupported |

전체398 product identity의최종값은EVIDENCE_SELECTION/MANIFEST가소유한다. 실제CLI003은4bf소스실행값을보존했고hostmain한파일→canonicalREADME한파일 비영향proof로현재입력과연결한다. currentintegrated001/host002/lifecycle003은2e7실행값을유지하고docdelta001로재사용한다. 새로운public/numeric/fault/PC/R05는현재f214manifest를직접실행한다. processheap에는native밖rollbackcapture와observer/driverretainedhistory가포함되며sessionheap이라고하지않는다.
