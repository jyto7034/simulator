# U07-M 선택·실패·재개 기록

P2G00→60 완료, 제품미변경. GEX00 실제baseline/process 인수 후 지정Sol/xHigh가 PLAN대로 GEX10을 진행한다. root는계획/조정, Astra/High비참여감사는동결뒤. exact source81 staticreview와U07target002 product389를구분한다.

read-only조사교정: 밖에서호출되는runtime_fact_snapshot은inactiveadmission8MiBfallback이있다. persistence rollback resourceclone은inactive계량을생략하므로같은fallback거절이아니다. 전자는trustedownercap연결,후자는기존native범위보존/명시scope로설계했다. 이는제품검증결과가아니며실행검증대상이다.


GEX00 역할 인수 후 PROFILE 구현 시작. private profile entry는 effective owner cap을 저장된 limits에서 그대로 사용하고 public 낮은 override를 높이지 않는다. inactive snapshot만 narrowed local helper로 owner cap을 사용한다. persistence rollback/nonbattle capture의 기존 native inactive 범위는 확대하지 않으며 process heap 포함과 scope/missing으로 구분한다. generation/rollback metadata는 실제 현재 owner에서 읽는다.


현재 PROFILE 작은검증10/10PASS이며 source389/5ab782…를 PC001 동안 고정한다. PC비교는 immediateU07 non-timing delta0과 historicalU06200-byte owner delta/17negative를 함께 검증한다. PC종료 전 ACCOUNTING/host product mutation 또는 기능 compiler 실행을 하지 않는다. 새 ACCOUNTING metrics는 기존 admitted_bytes failed-seed attempted의미를 유지하고 accepted/attempted/committed를 별도로 분리한다.


## ACCOUNTING 착수 전 source 경계 — PC001 진행 중

제품은 PROFILE source389/5ab782ef9b2c4c57b247e6832574551424b7874af953ccd548c65a67c10f64ec로 고정했다. ACCOUNTING의 Budget/Accumulator 필드는 observation feature에만 추가하고 gameplay CloneShape/RuntimeFactLimits/owner state에 넣지 않는다. scope가 없는 rollback capture/resource preflight는 native accounting unavailable이며 process allocator scope에는 포함된다. 초기 admitted_bytes의 attempted seed 의미는 보존하고 새 accepted field와 분리한다.

profile.nominal warning은 실제 immutable settings 값이고 owner hardcap은 별도 실제 limits 값이다. 작은 override에 warning을 min하지 않는다. settings를 알 수 없는 raw native helper에서는 warning unavailable을 표시한다. 정상 Core/World entry의 observation feature 경로만 Copy warning scalar를 전달하며 feature-off default 경로는 기존 run/body를 유지한다. dev nominal8MiB/owner255에서256 요청은 hard 거절이고 soft 초과가 아니다.

성공 whole-owner scan의 extent와 allocation/admission 요청 최대를 별도 보관한다. 실패 candidate peak 또는 성공 preparation은 live commit current가 아니다. 실제 generation의 profile/content fixed32-byte hashes 및 run seed를 entry/prepared/committed 시점에 각각 결속하고 publication의 원래 root-address guard와 rollback을 따른다. collector에는 build metadata를 bounded export receipt로 연결하고 identity/site를 metric label로 쓰지 않는다. 구체 product 구현/시험은 PC001 종료 및 strict parity 판정 뒤에 시작한다.


## ACCOUNTING 교정과 extent 범위

root의 읽기 검토로 두 경계를 보강했다. default BattleCore wrapper는 entry 당시 ACTIVE=false일 때에만 nominal warning을 설치한다. raw outer의 unknown profile은 nested owner가 채우지 않는다. NativeOwnerExtent는 Unmeasured/NoNativeOwner/Battle/PendingResources/BattleAndPendingResources를 구분하며 Some0 owner와 없는owner가 다르다. staged current는 World 끝의 이미 실행된 battle scan/pending_resource_extent 반환값을 재사용하고 추가 scan/charge를 실행하지 않는다. max_battle_owner_extent_bytes와 max_pending_owner_extent_bytes는 각각 성공한 기존 계산 hook만 사용하며 요청 최대는 RetainedAccounting의 별도 필드다. 기존 Usage[EXTENT]와 host aggregate는 battle 범위 그대로다. 여러 owner ledgers/operation admission/process allocator/RSS를 합산하지 않는다. NoNativeOwner는 성공한 World 검사에서 확인한 부재이며 native accounting None(해당 scope가 계량되지 않음)과 다르다.

고정 metadata는 observer/Budget에만 있으며 GameCore/BattleCore gameplay CloneShape/retained 원가/limits layout에는 삽입하지 않았다. generation은 실제 owner에서 entry/prepared/committed별 읽고, 실패candidate는 live current가 아니다. raw digest가 없는 fixture는 None으로 표시한다. identity004가 검증한 fork는 기존 private authoring capability이며 공개 application Clone 지원을 주장하지 않는다. 일반 outside-native rollback capture는 새 native 계량을 추가하지 않았고 actual process allocator 측정에서는 전체 process scope에 포함된다.


COLLECTOR 구현 경계: host library는global allocator를 지정하지 않고 main/격리lib test만System wrapper를 설치한다. 미설치의live/lifetimepeak는None이다. live는프로세스시작부터tracked 요청bytes이며RSS와구분한다. 창hook은SeqCst epoch 사전capture→inflight등록→재확인으로oldlatewrite를 차단한다. 종료64boundedspin후retiring이면peak unavailable이고다음begin busy는관측상태일뿐gameplay를실패시키지않는다. baseline/end는validpeak에포함한다. active-window view는새epoch를열지않고최대3readretry하며inflight시peak unavailable을유지한다. rawoperation은lease가만료되어도유한300ring에남고registry256/queue1024포화는관측drop이다. 원래heap회수시험은ignored목록과실행004를분리결속한다.


## 실제 표본·현재성·최종 검증 범위

실제 CLI003은기존002의exact17출력재판정과새9objects+mismatch로26case를판정했다. CLI source4bf는서버main종료교정의비영향을host_exit_correction002로증명하고README-only docs_delta001로현재398/f214에연결한다. 실제host002·lifecycle003·전체integrated001은2e7원실행manifest를유지하고같은doc-only proof로재사용한다. 바이트코드/입력/fixture/currentbinary불변과actualrawheader를함께확인하며원hash를새값으로바꾸지않는다. 이후public/numeric/fault/PC/R05는f214원실행이다.

low256KiB nativebudget표본의seed15701/ownermax35461/cumulativemax634913은서로다른scope다. strict-hard와dev-soft는같은site queued_fact_source/request262165에결속된다. 8KiB loader preflight(request8323)는실행전거절로보존한다. 이값을배포8MiB최대/초과나최종32MiB필요근거로해석하지않는다. 실제공식normal전투의4typed거절은authority/context/InvalidAction이며budgetrefusal로세지않는다. 실제host freshContinue성공과CLI liveContinue거절도현재지원수명으로분리한다.

actualfield101ms와actorTTL1000ms의별도유효fixture는각3freshgeneration×4mode를공개응답assertion으로검증했다. area readiness 해제/activeactor존재→만료후empty를검증하고공식balance/TTLs를수정하지않았다. deferredfull모드의해제표본자체는포화가아니다. 실제queue1100scenario full76drop/raw1024와normal1100raw의동일finalstate로별도포화입증을한다.

후속profile튜닝은원수치·새수치·측정window/입력/코드/binary/profileidentity·명시지원범위·변경이유·다른guard영향과독립인수를남긴다. U30-C/P가플랫폼·최대부하·Unity플레이와최종한도를소유한다. 이번수행자는observedmaximum을전체메모리상한으로일반화하지않는다.


## 최종 자체검증 결론과 감사 입력

54개 actualreceipt binding과181+31적용성은EVIDENCE_SELECTION/PREDECESSOR_IMPACT/COVERAGE에결속했다. 모든시간외PC576행은immediateU07exact0이며17negative가회귀은폐를거절한다. historicalU06+200는새costdelta가아니다. freshR05002의raw101/peak초과638972bytes와concurrentlatencyabsolutefalse관측값을그대로남기고기존peak-only예외및timing비적용범위를유지한다. 제품398f214에는새한도정책이나oracle완화가없다.

최종수행은Sol/xHigh의자체검증이며비참여Astra/High독립GEX20은동결뒤별도다. 새target의actualmembers·fuzz0정역·외부freezeEXECUTIONreceipt를인계한뒤모든writer를중지한다. U07targets/audits·실패fixture/판정tool·원래input/상한/첫표본은보존되었다. 최종32MiB배포필요성,전콘텐츠최대,플랫폼/Unity보장은현재결론이아니고U30-C/P와U31책임이다.
