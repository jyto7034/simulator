# R04 실제 결과·출처·현재 ActionHandle — 실행 계획

## Current 인수와 실행 권한

2026-09-11 KST. [correction002](CORRECTION_002.md)의 GEX10 구현·검증을 종료했다. Initial R04 [Not Complete 감사](COMPLETION_AUDIT.md)의 F01 계산 전 Strike 및 F02 실제 t300 적중 검증을 수정했고 새 경계 테스트와 full1,363개, 원래 한도/가드, all-target/default release 및 실제 API/CLI/소비자가 통과했다. [수용 증거002](ACCEPTANCE_EVIDENCE_002.md), [변경 귀속002](evidence/CORRECTION_002_ATTRIBUTION.json), [영향002](PREDECESSOR_IMPACT_002.md)를 고정한 뒤 source mutation을 멈추고 선행 R01→R02→R03 및 별도 R04 fresh GEX20을 복원한다. 초기 감사와 모든 과거 증거는 보존하며 R04 감사 완료 및 전체 R00–R24는 미완료다.

상위 사용자가 채택한 [AUTORUN](../../../../roadmaps/content_authoring/AUTORUN_REQUEST.ko.md) §§3/5/6/8의 별도 자식 선택으로 `(GS-CARF-P2G50-002, GOAL-CARF-01)`을 `execute / goal-run`의 root/selected Single로 지정한다. 대응 R04, `PHASE-CARF-01`, assigned final nodes `AG-CARF-01`(WP01+WP02)와 `WP-CARF-03`이다. 자식 terminal은 GEX00→10→fresh20의 Complete/High 또는 실제 중단 조건이며 R05–R24 완료를 뜻하지 않는다. 다음 자식 선택은 원 상위 요청이 소유한다.

Exact accepted upstream: [HOF-CARF-P2G60-002](../../P2G_60_HANDOFF_002.json), [FPF-CARF-P2G30-003](../../P2G_30_FREEZE_003.json), [PP-CARF-P2G40-004](../../P2G_40_PHASE_PLAN_004.json), [PVR-CARF-P2G40-004](../../P2G_40_VALIDATION_RECEIPT_004.json), [GS-CARF-P2G50-002](../../P2G_50_GOAL_SET_002.ko.md), `WR-CARF-P2G20-002`, `CRA-CARF-P2G20-003`. Corrected upstream hashes are in [successor baseline002](baseline/BASELINE_002.json); original353file source archive remains baseline001. P2G Authored/Not Activated/Implementation Authorized No는 당시 인계 상태로 불변 보존한다. 구현 권한은 위 사용자 채택 요청에서 별도로 온다.

적용 지침은 [GEX](../../../../goal_execution_pipeline%281%29.md) `2026-09-09-gex-r1`, Reader Contract/§5/6.1–6.7/GEX00/10/20과 accepted [P2G](../../../../policy_to_goal_pipeline_revised_v2.md) `2026-09-09-r2`다. 추가 ancestor/workspace AGENTS 검색 결과는 없음이며 pipeline/validator/AGENTS/전역 설정을 변경하지 않는다.

Canonical execution directory: `core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal`. PLAN 역할은 이 파일, timeline은 [EXPERIMENTS.md](EXPERIMENTS.md), current 판단은 [EXPERIMENT_NOTES.md](EXPERIMENT_NOTES.md). 초기 감사는 `COMPLETION_AUDIT.md`로 Not Complete 종료했고 교정 후 별도 감사 역할은 `COMPLETION_AUDIT_002.md`다. 최초 GS001 인수 때 초기화 전에 79개 기존 실행 역할 파일의 exact member 부재와 directory/VCS history 부재를 확인했고, 이 활성 요청에서 방금 생성한 exact Goal/Handoff 및 모든 P2G의 no-execution 기록과 대조했다. 유실된 실행을 임의 초기화한 것이 아니다.

## 초기 GEX00의 Actual baseline·선행·격리 (historical intake)

HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`, branch `cleanup/simulator-dirty-worktree-20260803`. rustc/cargo 1.91.1 및 실제 환경이 R03 target002와 일치했다. [BASELINE001](baseline/BASELINE_001.json)은 353개의 현재 source/test/tool/config/data/lock/계약/R04 planning 파일 9,360,468 bytes를 [durable tar](baseline/SOURCE_BEFORE_001.tar.gz)에 byte-exact 보존하고 재검증했다. 관련 preexisting [diff](baseline/PREEXISTING_SCOPED_DIFF_001.patch)와 전체 [status](baseline/PREEXISTING_STATUS_001.txt)를 기록했다. Clean HEAD와의 차이는 사용자/선행 작업이며 R04 성과로 세지 않는다. 새 source는 생성 전 부재를, 기존 source는 이 사본 대비 실제 delta를 기록한다. 이 사본에 없는 기존 파일을 바꿔야 하면 mutation 전에 현재 bytes/owner를 추가 보존한다.

Actual R03 target002/audit002는 **current / unchanged, Complete / High**다. 3,682manifest, 22external inventories/1,263files, 318validation inputs, 3release binaries와101audit outputs가 동일하다. R01audit004/R02audit003은 R03 별도 감사가 판정한 **current / proven non-impacting delta**를 인수한다. 이번 인수에서도 동일한5changed members와3changed binaries, 나머지 target·각14external inventory/983files·합18audit outputs가 원 independent closure와 동일함을 확인했다. oldtargets 전체를 unchanged로 표현하지 않는다.

기능 선행은 R01 typed effect/source이고 R02 generation/load/store, R03 authoring/public/observer/control을 실제 소비한다. 단일 Phase이므로 같은 Goal Set 안의 direct predecessor Phase는 없다. GEX00 소비 판정은 새 predecessor 감사 verdict가 아니다. R04 expected mutation이 선행 경계를 침범하면 영향 검증과 필요한 fresh 감사를 복원한 뒤 완료한다.

별도 Shared Validation Baseline Reuse Directive는 N/A다. Actual R03 full1327/focused59와12selected API/CLI/replacement/cost/source receipts는 exact unchanged 입력의 초기 근거로 재사용한다. 새 R04 동작/비용의 증거가 아니다. 새 production/test/harness/fixture/config/lock 변화는 해당 seam을 invalid한다. WP03이 최종 integrated full core/all-target/default-release/current consumers와 새 native/source/clone 비용을 실제 검증한다. 날짜·Phase 진입만으로 full suite를 반복하지 않는다.

Root sole writer. Read-only bounded 조사/검증 보조만 위임할 수 있고 계획 보조는 completion reviewer로 사용하지 않는다. 로컬 Cargo/registered consumers/fixture source와 메모리 profile·격리된 external RON 출력에 접근 가능하다. 현재 disk/memory/toolchain 관찰은 baseline에 있다. 향후 command의 실제 fixture·출력/side effect·자원 범위를 확인하고 source와 personal store·외부 서비스를 격리한다. 본 인수에서는 build/test/probe를 새로 실행하지 않았다. 별도 fresh reviewer는 현재 도구에서 사용 가능하며 최종 시점에 실제 다시 확인한다.

## 목표·경계·상세 설계 인계

Goal의 C01–08/20acceptance/10IVA/10required dimension과 I01–06, [CRA](../../CODE_REVIEW_003.ko.md) B01–05/P01–07, Plan SC01–06을 인수한다. 정책/acceptance를 실행 문서로 다시 결정하지 않는다.

AG01의 WP01은 actual damage/heal/cost/death/predeath/mandatory-vs-optional·source/strike/segment·실제 equipment copy를, WP02는 current request/action/attempt identity·milestone/terminal/settlement/cause cancellation·public atomicity/clone/lifecycle/native bounds·consumer projection을 함께 전환한다. WP03은 durable validation을 소유하며 실패 source 교정은 AG로 반환한다. 둘을 함께 구현해야 하는 이유는 actual facts가 action/attempt state와 transaction을 필요로 하고 action settlement가 actual mutation/carrier facts를 소비하기 때문이다.

기존 formula/basic-only OnAttack/OnHit/HP-loss gate/공명/signed Heal/actual tactical pay와 death/wave once, cause-specific cancellation·released projectile/GroundFixed 수명을 보존한다. source는 cleanup 전 victim buff 원천과 killer를 구별하고 action/strike/segment는 log retention과 독립이다. 장비 사본은 실제 inventory ID를 두 draft/build/deploy/source에 전달하며 기존 transient runtime RNG/cooldown/stat ID를 유지한다. 정의/원천이 없는 경우 absence를 명시한다.

GEX10은 현재 producer→state→consumer/clone/restore/종료 경계를 상세 조사하여 다음을 production 전에 확정한다: exact Rust 타입/입출력, action/attempt opaque validity와 deterministic logical identity 분리, direct strike allocation/segment mapping, signed actual results/predeath snapshot 시점, source context 전달·유한성, 각 carrier의 Accepted/Released/FlowFinished/EffectsSettled 전이표, cause별 취소/failed/late/duplicate 전이, bare BattleCore와 outer public operation atomicity, native+authoring preflight/clone/retained limits, result/live/retreat/drop/durable 적용성, paired equipment constructors, 변경 symbols와 test oracle. 내부 표현/함수 선택은 기술 재량이나 승인된 소유·수명·실패·저장 의미는 변경하지 않는다.

현재 승인이 없는 true join/periodic root/zero-HP 신규 hit/predeath 새 콘텐츠 반응·shield/HPcost/HP1/parry/special kill, 후속 ResourceStore/Flow/boss adapter/fragment copy/새 relic 기능은 추가하지 않는다. Server/Unity/UI/외부 배포/personal store 삭제·변환/전용 콘텐츠 runner/미지원 placeholder/global 변경은 제외다. R04의 상태 저장 적용성·budget·직접검증을 R20/R23으로 미루지 않는다.

## 검증·완료·중단

[Goal Acceptance Completion Matrix와 Command/Changed Input-to-Seam Map](../../P2G_50_GOAL_SET_002.ko.md), Plan IVA01–10이 exact 기준이다. 정상값만이 아니라 invalid input·partial failure/rollback·retry/stale/duplicate·same-time order·empty/cap/overflow·live reconnect/restart/save·literal production RON·real producer-consumer·legacy absence를 모두 검증한다. Native state와 source/context/retained results는 finite faults/admission 및 actual allocator/cost/determinism evidence가 필요하다.

Focused 시험으로 구현을 확인하고 관련 실패를 교정한다. 최종 full core/lib/tests, all-target/default release/current public API/CLI 및 affected predecessor tests/probes는 실제 nonzero counts와 expected/actual을 기록한다. Cost의 plain timing과 allocator instrumentation을 구분한다. 로그/trace 보관량이 gameplay facts/RNG/receipt 의미를 바꾸면 실패다. 테스트 완화/삭제·golden 기계갱신·fixture 전용 production 분기·silent fallback으로 성공을 만들지 않는다.

새 사실/handle/상태와 실제 소비자·두 equipment 경로·동반 contracts가 완결되고 C01–08/20acceptance/10IVA/10dimension에 current direct evidence가 있어야 GEX10을 닫는다. 변경된 Core Runtime/Skill Target/Data Loading contract를 실제 동작과 맞춘다. Stable source와 scoped delta/fixture/actual output을 Audit Target에 고정한 뒤 mutation을 멈추고 required predecessor currentness 복원과 별도 fresh GEX20을 수행한다. 긍정 verdict는 미리 작성하지 않는다.

Material gameplay/balance/observable lifetime/storage-loss/compatibility/scope 변경, upstream 의미/Phase 전제 오류, required source/consumer/reviewer/evidence 부재·unsafe dirty overlap·실제 자원 한계에서 해당 owner로 반환/중단한다. 같은 범위 기술 실패·해결 가능한 검증 오류·단계 전환만으로 사용자 재승인을 요구하지 않는다. 원 R00–R24 전체 완료 여부는 상위 요청의 실제 종료 조건으로 판정한다.

## Current 상세 설계

[DESIGN001](DESIGN_001.md)과 [Preparation001](GEX_10_PREPARATION_001.json)로 실제 타입/전이/시험 설계 및 첫 mutation scope 기준선을 확인했다. AG01 구현과 WP03 current 검증은 완료됐으며 아래 최종 evidence map과 별도 fresh 감사로 연결한다.


## Source-premise correction and resumed design

Historical GS001/HOF001 GEX00→EXP002 design returned at EXP003. PFC001/COD002/DS001 and all O20–60 RR002 complete results now select WR002/CRA003/Freeze003/PP004/GS002/HOF002. No runtime implementation existed to migrate. Same5PER/5DR/20acceptance/Phase/Goal key and execution directory preserved. Source/baseline001 remains exact except owned mutable Working Policy; baseline002 retains corrected planning bytes.

GEX10 must preserve death pending/queued/Follow behavior and actual resonance reset after dead invocation no-op. Do not reuse withdraw cleanup for death or suppress allow-dead OnDeath. Piercing runtime removal may lack old terminal callback: observe actual finite removal and remaining queued effects independently; do not synthesize gate release or FlowFinished while deferred control remains. Exact successor IVA01/02/05/06/08/09 include these oracles. Separate planning helper findings are reused, not completion audit evidence.
