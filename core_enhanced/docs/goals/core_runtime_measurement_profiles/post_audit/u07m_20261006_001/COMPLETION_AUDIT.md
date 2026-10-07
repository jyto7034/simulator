# U07-M 독립 전체 완료 감사

판정: **Complete / High, open findings 0**. GOAL-CMRP-01 / GS-CMRP-001 / PHASE-CMRP-01의 단일 phase 전체를 감사했다. 최종 효력과 출구 현재성은 이 디렉터리의 `AUDIT_CLOSURE.json`이 결속한다. 독립 감사가 닫힌 이후 기존 연속 권한에 따라 U08 진입 가능하며 별도 GEX-30은 없다.

대상은 `execution/single_goal/target/u07m_001`이다. MANIFEST SHA256은 `50c3c228d2c8f0915ab6133014bcf7783bb44d3b8880a8ba7c784ffe130a5733`, product SHA256은 `f2144e9a4c5ddeb41245799294a75b41de9d843018cb2166f711b7f07821cda8`이다. 외부 동결 실행 `.cache/u07m-freeze-001/EXECUTION.json`의 `1fa9c128a53b575bc43bf956ab468ba96cd00a0cab6bcc5e1f5bec300797b332`도 독립 확인했다.

감사자는 U07-M 설계·구현·제품 시험에 참여하지 않았다. 이전 U07 감사 참여는 다른 대상의 선행 검토이며 현행 구현 참여가 아니다. 지정 독립 Astra/High 역할로 source/target/로드맵 변경 없이 이 감사 디렉터리에만 검증 스크립트와 판정을 작성했다. 제품 agent는 동결 후 쓰기 중단, 다른 worker 없음. 제품 빌드·전체 시험을 다시 실행하지 않았으며 현재 실제 raw 결과를 독립 해독·재계산하고 변경 owner/caller/test의 의미를 직접 검토했다. 전체 source를 다시 읽었다거나 모든 조합을 시험했다는 주장은 하지 않는다.

## 정책·목표와 현재성

WORKING_POLICY, CODE_REVIEW TD01–06, 최종 freeze10개 조건, P2G40의 PROFILE→ACCOUNTING→COLLECTOR→PLAYBACK→VERIFY 5node 및10integration seam, P2G50 Goal, P2G60 handoff와 GEX10 최종 selection/coverage/impact/adaptation을 검토했다. canonical artifact payload digest와 freeze→phase→goal→handoff→GEX exact refs를 재검증했다. 별도 invalid 초기 planning/compile/sample 시도는 성공으로 계수하지 않는다.

`CONDITION_JUDGMENTS.json`은 현행10조건·10필수차원·181선행조건·31R06 각각의 원문, selector/direct evidence와 독립 Complete 이유를 기록한다. 선행161조건 및 U07의20조건은 기존 원문과 일치한다. 별도31R06의 acceptance/expected/IVA 연결도 원래 의미와 동일하고 모든 연결 IVA에 실제 현재 통과 selector가 있다. 이전 U07 closure `d371f43c1122826347261ed68f0817e66432bbc97e85fdb6759deaf686a59cb9`의 exact 불변 owner 판단은 재사용하되, 변경된 profile/native/observation/application/world/host consumer와 최종 baseline은 이번에 직접 검토했다.

입구에서 membership4463개, source-before395/source-after405, product398, records2829, references1228을 확인했다. 현재4archive와 외부 U07 target002의4archive member를 hash/size/membership로 확인했다. 별도 임시 경로에서 `patch --fuzz=0` 정방향·역방향의 exact 파일 집합과 바이트가 일치했다. 출구에서도 같은 현재성을 확인하며 감사 디렉터리 외 Git 상태는 시작과 동일해야 closure를 발행하도록 했다. 원래 dirty/untracked 상태를 정리하지 않았다. 첫 출구에서 감사 중 Python import가 새로 만든 제외대상 cache2개만 발견해 exact 파일을 제거하고 bytecode 작성을 껐으며 출구를 다시 검증했다(`AUDIT_CACHE_CLEANUP.json`). 제품·사용자 파일은 제거하지 않았다.

전체 누적 source delta는 수정19개/신규10개이며 `SOURCE_DELTA.json`에 exact 전후 hash가 있다. 직접 production 검토는 settings canonicalization/strict validation, private budget entry 및 deferred/nested/seed/growth, battle snapshot/transaction caller, World active/pending ownership, application preparation/commit/rollback, observation fixed summaries/identity/retained metrics, host registry/worker/shutdown, allocator/RSS, 전체 playback/export/script까지 포함한다. 기존 mutable budget owner를 재사용하고 측정 값은 admission/gameplay 결정으로 역류하지 않는다. unsafe 처리는 System allocator wrapper에 국한되며 재진입 allocation/lock/log 없는 hook과 epoch 경계를 검토했다.

## 검증한 실행 근거

`EVIDENCE_VERIFICATION.json`은 selected54 receipts의 실제 command/toolchain/environment, log/runner/실행 script archive, source-before=after, exit와 selector를 검증한다. 39개는 final product exact,14개는 README-only delta, CLI playback1개는 server-main 한정 수정 후 README-only delta다. 전체 source manifest를 비교해 허용된 변경 파일만 다름을 확인했다. server-main은 독립 CLI library/binary source에 영향을 주지 않고 그 실제 출력의 binary/input/profile/source identity도 보존된다. receipt의 실제 실행 source를 final 값으로 덮어쓰지 않았다.

최종 suite 실제 통과 수는 Core default1875 / fixtures1875 / observation1892, host default93 / observation103 / measurement122다. suite 집계와 서로 다른 selector 수를 구분했다. release/check/event-log/clippy/fmt/heap, allocator 없는 library support, 정상 public input/schema/profile/defaults/fault/objects/Continue/rewind 및 성능 receipt도 실제 raw exit에 결속된다. 감사가 시험을 새로 실행한 것으로 표시하지 않는다.

448개 immediate 보존 입력은 원본=copy=기존 hash이고239개 historical harness 파일은 capture archive와 현재 원본이 일치한다. prepared harness adaptation과 실제 실행 archive가 연결되며 원래 workload/cap/oracle를 새 개발 모드에 맞춰 재작성하지 않았다. 공개 입력 hash1509개, schema32case(26negative), 실제 object6종과 combat3종 raw를 독립 검증했다. actor/trap/wall의 source creator 철수 후 생존·실제 damage/소모·HP3벽 파괴 의미를 공개 결과에서 확인했다.

PC002는576행/36조건/31성공5typed거절이며 immediate U07 PC005와 **elapsed_ns만 제외한 모든 필드가 완전히 동일**하다. U06 대비 고정200byte owner 설명은 보조 역사 검증이다. work/clone/allocator 요청/evaluator/gameplay/log/completion/code/site/cap 변경을 허용하지 않는 비교와17negative controls도 독립 재계산했다. 관측 없는 default의 cap·수치·비용회귀를 dev 설정으로 면제하지 않았다.

R05002는 fresh target과 실제 depinfo의 main/include 경로, binary, source, 환경, raw, summary hash까지 독립 확인했다. 원본 `bbbae769…` main 및 `64debfd6…` include, 동일 workload/원래 guard가 유지된다. **raw exit101은 그대로 실패**다. event-heavy operations44194≤70671, retained265846≤276174, peak271415≤276174. concurrent operations370928≤389240, retained4196928=4196928, peak9669160>9030188로 **+638972byte만 기존 concurrent-peak-only 경고**다. 다른 hard guard 실패는 없다. allocation observer 실행의 latency gate는 비적용이고 shared-host timing을 배포 성능 보장으로 해석하지 않는다. 새로운 cap/예외는 만들지 않았다.

## 정상 측정 경로와 경계 판단

`SAMPLE_VERIFICATION.json`은 실제 playback26개, lifecycle10개, host4조건×initial/restarted2단계의 JSONL·메시지·detail·footer를 독립 해독한다. source/binary/input/profile/seed/공개 결과가 연결되고 native accepted 요청은 allocator/RSS와 별도다. same profile/seed/input의 none/normal/detached/full 결과 일치, queue full의76drop, interleaved16/32MiB identity/owner isolation, mismatch 출력 부재를 확인했다. `STATISTICS_VERIFICATION.json`은72분포 mean/p95/max와36파일의 retained/drop/export byte population을 raw에서 재계산한다.

실제 host는 normal WebSocket 명령/actor/checkpoint 저장 후 fresh restart Continue, 전투 진입, 권한 없는 Pause typed refusal, 권한 있는 Pause, reconnect, quit과 lease 회수를 수행했다. current profile identity 및 accepted Continue raw와 nonzero native 요청이 있고 retired session은 사라지되 raw ring은 유한하게 남는다. SIGINT/SIGTERM 두 종료 가지에 allocator footer와 exit0이 있다. process heap은 모든 observer/thread/session을 포함한 요청byte이며 세션별 물리 heap으로 귀속하지 않는다.

정확8/16/32MiB seed/growth cap와 nominal warning exact/one-over, 작은 명시 override, raw 공개16MiB 거절, nested/deferred nonrefund, 다음 operation 초기화, work/clone/overflow sticky typed 오류를 직접 test source 및 실행 selector에서 확인했다. 9MiB identifier projection 시험은 representation boundary fixture이며 정상 authored content 샘플로 세지 않는다. 일반 owner snapshot은 trusted effective ceiling을 쓰지만 active 작은 outer budget을 높이지 않는다.

별도 nominal256KiB fixture는 같은 공개 actor first12명령으로 strict hard refusal과 dev16 soft-only 진행을 보여준다. 이는 deployment8MiB 초과 실측 또는 최종값 적정성 증거가 아니다. 앞선 entry-only256KiB 무초과 및8KiB loader 실패는 실패 원본으로 보존했다. failed seed attempted와 accepted seed, cumulative requested/accepted, current committed owner와 failed candidate peak를 구분한다. pending resource owner는 battle 종료 뒤 별도로 남고 prepared/discard/rollback identity도 실제 owner를 따른다.

Allocator는 창 이전 allocation free, zeroed/realloc/failure, 경쟁 hook, 창 종료 뒤 delayed peak publication, successor epoch 오염 방지, baseline/end peak 포함, overflow/미계량을 검토했다. 종료 경계 hook이 남으면 null/busy로 표시하는 유한 관측이며 정지 snapshot을 주장하지 않는다. RSS는 bounded Linux `/proc/self/status` 실측이고 실패/미지원은 missing이다. export는 sample/record/file/footer 상한과 create-new를 가지며 driver 입력 오류/typed public 거절/측정 drop을 구분한다. support binary에서 global wrapper 없는 library를 instrumented로 허위 표시하지 않는다.

## 남는 범위와 후속 진입

U07-M은 유한 trusted 설정과 실제 측정·재현·수명/회수·후속 조정 준비의 완료다. local Linux 외 실제 배포/Unity/전 콘텐츠 정상최대 또는 leak-free 보장은 하지 않는다.16/32MiB는 유한 engineering window이며 default8MiB를 바꾸지 않았다. RSS는 process resident/lifetime HWM, allocator는 fragmentation을 제외한 requested bytes, native cumulative admission은 owner retained extent와 다르다. outside-native persistence/rollback capture는 native 계량 밖이고 process allocation에만 들어가는 기존 경계다.

기존20분 stress 제외와 R05 concurrent peak-only 경고가 남는다. 최종 플랫폼·정상 최대/동시 부하·여유와 배포값은 U30-C/U30-P, 최종 통합은 U31의 책임이다. U08 이후 owner는 지표를 유지하고 U11은 identity를 인수한다. 향후 변경은 새 profile/revision·전후 raw·이유·영향 검증·독립 인수를 필요로 하며 평균/p95나 heap peak로 누적 admission cap을 치환하지 않는다. 이 범위에서 추가 교정 필요 finding은 없다.
