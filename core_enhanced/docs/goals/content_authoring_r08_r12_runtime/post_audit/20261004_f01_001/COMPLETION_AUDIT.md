# U01 F01 독립 완료 감사 — Complete / High

2026-10-04 · GEX-20 · `GS-CARX-001 / GOAL-CARX-01 / HA-CARX-001`

**판정: Complete / High.** 새 동결 target에서 F01을 해소했고, 55개 필수 조건 및 R06/R07의 현재 통합 적용성을 확인했다. 완료 차단 finding은 없다. 기존 2026-10-03 Not Complete와 원자료는 역사적 결과로 보존한다. 이 판정은 아래 target 및 종료 [AUDIT_CLOSURE.json](AUDIT_CLOSURE.json)에 귀속된다.

## 권한·독립성·범위

사용자의 [연속 실행 요청](../../../../roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md)은 독립 감사·교정·재감사와 선행 통과 후 다음 작업을 포함한다. 옛 정식 인계의 ‘감사 제외’는 당시 요청 범위다. 현재 요청에 중간 사용자 승인 대기는 없다.

별도 reviewer agent `/root/u01_f01_audit`는 요청된 GPT-6 Astra / High 감사자로 배정되었으며 제품 설계·구현·교정·시험 작성에 참여하지 않았다. GEX `2026-09-13-gex-r7` §5·6.4·6.7 및 GEX-20의 fresh read-only 경계를 적용했다. 감사 중 새로 작성한 것은 이 디렉터리의 감사 문서·검사 코드·증거와 격리 임시 복원본뿐이다. 제품/새 target/기존 감사/U00 기록은 수정하지 않았다.

기준은 [정식 Goal](../../P2G_50_GOAL_SET.ko.md), [정식 인계](../../PLAN_HANDOFF.ko.md), [원 55조건](../../execution/single_goal/COVERAGE.json), [원 설계·수용](../../../../roadmaps/content_authoring/handoffs/R08_R12_DESIGN_AND_ACCEPTANCE_20260930.ko.md), [이전 F01](../20261003_001/COMPLETION_AUDIT.md), [이번 교정 인계](../../execution/single_goal/corrections/f01_action_flow_20261004_001/AUDIT_HANDOFF.md)다. Single Goal이므로 별도 Master/GEX-30을 만들지 않는다.

U00는 현재성·기존 자료 접근성·정책 위임·범위 연결의 인수로 적합하다. 이전 감사 source 387개에서 교정 전 baseline까지 제품 변경은 없고 현황 문서만 달랐다. U00의 새 원정/보상/도구/구저장 결정은 후속 owner의 정책 입력이며, 이번 Goal의 승인된 V9/명시 V7·V8 변환 계약을 소급 폐기하지 않는다. U00 인수를 후속 제품 전체 완료로 취급하지 않는다.

## exact target·현재성

- Target: [f01_action_flow_001/MANIFEST.json](../../execution/single_goal/corrections/f01_action_flow_20261004_001/target/f01_action_flow_001/MANIFEST.json)
- Manifest SHA-256: `3a7bd83b904d1f8367590b7c2d3f4616a9cce8ca6a8a9cb0c7e0cc767d12b50a`
- Source digest: `5b05f2454b83b9b1dc4a4db50b7d4fc066efeca84d319c6095212c5c505ac876`, **388 files**.
- 교정 전: `36284fada33057409f61fe201fe0eddca1560136e5f4be026994fbad77c101cb`, **387 files**.
- 실제 교정은 `boss_runtime.rs`, `queue.rs` 두 제품 파일과 신규 `boss_action_cancellation_tests.rs` 한 파일이다. 이전 감사 current source에서의 누적 차이는 이 세 파일과 `ROADMAP_STATUS.md`다.
- 새 target 대비 **current / unchanged**. source membership을 독립 재열거했고 모든 bytes가 일치했다. records **173개**, references **35개**, artifacts **9개** 및 archive member/bytes도 일치했다.
- 실제 before→after→before를 `/tmp` 격리 복원본에서 `patch --fuzz=0`로 다시 수행했다. 추가/삭제 membership과 내용 모두 일치했다. 원 작업 트리에 patch를 적용하지 않았다.
- 이전 동결 records **1,435개**와 archives/patches **9개**의 hash도 현재 실제 파일과 다시 대조했다. 기존 dirty/untracked를 감사 성과로 귀속하거나 정리하지 않았다.

근거: [CURRENTNESS.json](CURRENTNESS.json), [검사 코드](verify_evidence.py), [정방향](restore_forward.log), [역방향](restore_reverse.log), [과거 기록 보존·baseline 대조](BASELINE_REVIEW.json). Core 공개 API/CLI·로컬 데이터·checkpoint가 consumer 경계이며 변경 가능한 운영 DB/배포 서비스의 현재성을 주장하지 않는다.

## F01 해소와 소유권 검토

| 직접 검토 경계 | 판정 근거 |
| --- | --- |
| 취소 producer | `cancel_current_state`의 task-owned `DetachAction`과 `release_finished_boss_waiters`의 TargetInvalidated / TimedOut+CancelUnreleased가 새 `cancel_boss_action`으로 연결된다. 준비 완료 뒤 active Flow도 제거한다. |
| 정확 action 격리 | pending은 같은 actor의 동일 opaque handle만 take한다. active는 actor와 handle을 모두 비교한다. `ActionHandle::eq`는 ordinal 숫자가 아닌 Arc occurrence identity다. 취소할 cast를 최소 sequence부터 선택한다. 다른 cast/이후 state marker를 보존한다. |
| 준비 중 locks·actor work | matching pending cast의 focus lock만 시간·permission을 확인해 해제한다. 정확 취소는 `cancel_boss_owned_work`를 부르지 않으며 이동/basic epoch·windup·다른 action을 일괄 취소하지 않는다. 기존 state 자체의 RouteCombat/movement 소유 정리는 별도 경로다. |
| legacy·structured control | 해당 active cast를 제거하고 그 sequence의 `SkillStep`/`SkillFlowWake`만 retain에서 제외한다. 이미 bucket에 추출된 step은 `sim.rs`의 owner 부재 검사, wake는 `drive_skill_flow`의 owner/generation 검사에서 no-op다. 새 상태의 owner를 재생성하지 않는다. |
| 방출된 projectile/area | active carrier와 완료 event는 pruning 대상이 아니다. carrier의 `DamageSourceSnapshot`, cast/source-copy, expression capture 및 delivery/resource ticket이 control owner와 독립적으로 남는다. impact/tick은 원 source로 효과를 실행하고 expiry/advance가 ticket을 닫는다. |
| terminal·resource | `runtime_facts.terminal`은 미실행 control steps를 닫지만 outstanding delivery를 지우지 않고 기존 terminal도 덮어쓰지 않는다. resource terminal은 control scope/completion을 닫으며 finite delivery의 chain hold는 유지한다. `close_delivery` 후에만 실제 settlement가 성립한다. |
| Detach·Released | 명시 Detach와 정상 Completed/Released는 `cancel_unreleased=false`다. task handle을 take한 뒤 state 이탈하므로 정상 release 뒤의 Flow를 다시 취소하지 않는다. 기존 t100 Released/t300 Settled와 새 Detach 회귀가 이를 확인한다. |
| work·retention·rollback | active/queue 길이를 charge하고 매 mutation 뒤 poll한다. 새 scan/terminal 오류를 삼키지 않는다. boss boundary는 preflight 후 staged clone에 적용하며 실패 시 publish하지 않고 outer step도 복구한다. control owner/event/source refs는 drop되고 surviving carrier만 자기 ticket을 유지한다. heap capacity를 물리적 0으로 줄인다는 주장은 하지 않는다. |

새 7개 시험의 실제 assertions를 읽었다. pending/active 취소, legacy delayed step, stale wake/추출 step, 같은 시각과 분할 진행, foreign same-ordinal capability·무관 actor work, 지속 area 3회 tick과 최종 outstanding delivery 0, **6개 취소 fault 지점의 core/queue/execution 전체 복구 및 clean retry**를 확인한다. synthetic foreign cast는 내부 격리 증거이며 공개 제작 지원 증거를 대신하지 않는다.

이전 공개 입력 5개는 **140개 입력 파일 bytes 그대로**다. 저장된 raw JSONL을 독립 파싱하여 marker 시간·projectile/area 횟수·HP 전후·원 boss source까지 확인했다.

| 입력 | marker | projectile HP 손실 | 후속 area | 판정 |
| --- | ---: | --- | --- | --- |
| Cancel50 | 5051ms | 없음 | 없음 | PASS |
| Cancel1500 | 6501ms | 5885ms, 정확 1회 −1 | 없음 | PASS |
| Cancel500 | 5501ms | 5885ms, 정확 1회 −1 | 없음 | PASS |
| 강제 state exit1500 | 6501ms | 5885ms, 정확 1회 −1 | 없음 | PASS |
| Detach1500 | 6501ms | 5885ms, 정확 1회 −1 | 6885ms, 두 직원 각각 −1 | PASS |

다섯 event 집합은 trace off/on에서 각각 완전히 같다. 이전 **2 PASS/3 FAIL**을 유지하면서 새 **5 PASS/0 FAIL**을 판정했다. 직접 실패했던 A08-03/A08-04/A10-04/A10-06 및 X10 완료 conjunction이 해소됐다.

## 전체 조건·선행 통합·구조

[CONDITION_REVIEW.json](CONDITION_REVIEW.json)은 원 55조건을 누락 없이 보존하며 각각의 의미, 실제 producer/consumer, 변경 영향, selector와 이번 독립 판단을 기록한다. **55 PASS / 0 FAIL**이다. 이전 Not Complete의 테스트 성공행을 개별 조건의 무조건 승인으로 인수하지 않았다. 원 요구와 source 의미를 읽고 불변 부분의 유효성을 확인한 뒤 현재 raw evidence를 재사용했다.

[R06_R07_IMPACT_REVIEW.json](R06_R07_IMPACT_REVIEW.json)은 R06 **31수용→21 integration**, R07 **F01–F12**의 현재 통합 적용성을 별도로 복원한다. 과거 367파일 Complete/High의 digest가 현재와 같다고 주장하지 않는다. R08–R12의 Flow/sustained/protection/V9 확장과 이번 cancellation을 포함한 실제 소비 관계에서 typed unit·checked evaluation, Snapshot/Live 시점, exact copy/resource lease, 실제 HP-loss, 관측 replay, admission/rollback, 사본·형태·회수·저장 의미를 확인했다. 특히 control cleanup 뒤 delivery frame과 source withdrawal, 위조 fragment Arc 거부의 기존 회귀가 현재 양쪽 suite에 존재한다.

새 source에는 RON/시험 이름별 분기, 상수 시각 우회, Player 위장, silent fallback, 두 번째 evaluator/Flow authority, schema/save version 변경이 없다. legacy step lowering과 공통 Flow 소비 경로, source-local 지속 효과, 개인→공유/priority/activation 순서의 shield arbitration, 특수 kill의 optional dispatch 억제, V9 strict checksum 및 명시 converter가 유지된다. canonical runtime/data/target/rule 문서는 byte 동일하며 이번 교정은 문서에 이미 규정된 취소 계약을 구현한다.

Goal의 10개 validation dimensions는 모두 applicable이다. success/live-data/producer-consumer는 공개 CLI와 정상 Core 통합, invalid-input은 compiler·typed refusal, failure/retry는 후보 rollback/fault, ordering은 같은 시각/occurrence 순서, boundary는 native/work/retained admission, restart는 host/checkpoint fresh process, legacy 부재는 공통 Flow/명시 converter로 연결된다. 근거 없는 N/A는 없다.

## 검증 재사용과 제한

| 검증 | 이번 감사 처리 |
| --- | --- |
| default / test-fixtures 전체 | exact source/runner/raw log 확인 후 재사용. 각각 **1684 PASS, 0 FAIL, 0 ignored/filtered**. 모든 88개 Goal/선행 selector 행과 신규 7 selectors를 성공행에 대조 |
| check / event_log_checks / clippy / fmt / release | 현재 source 전후 동일인 실제 exit0 receipt 재사용. rustc1.98.1과 command/env 기록 유지. clippy 기존 never_loop/eq_op 허용 및 경고 존재를 숨기지 않음 |
| 공개 CLI | 9팩45명령 exit0, 실제 amount 문서·trace parity 확인. 기존 OBS001 회복 preview 직원 배열만 UUID 정렬; 일반 gameplay/amount normalization은 없음 |
| PC36 | 새 격리 사본 source 일치·2파일 계측 hook·보조 probe identity 검증. 원 comparator를 새 감사 출력으로 재실행: **576행/36조건/31success/5typed refusal**, gameplay/completion/error-code 차이0, 기존 requested-detail 차이80. v32→v31 비교는 leading version만 바꾸는 기존 envelope 계약 |
| R05 | 원 raw/cap/workload/tool identity를 재계산. raw **exit101** 보존. concurrent peak **9,373,775 − 9,030,188 = 343,587 bytes**만 기존 승인 경고. allocation count/retained/event-heavy peak/parity/repeatability는 모두 통과. latency gate는 원 계약상 비적용 |
| 실패·중단 이력 | focused_002의 Debug HashMap 순서 false failure, all_default_001 중단, r05_current_001 offline dependency 실패와 raw 부재 비교 실패는 보존. 해당 실패를 PASS로 바꾸지 않음 |

[EVIDENCE_REUSE.json](EVIDENCE_REUSE.json), [BASELINE_REVIEW.json](BASELINE_REVIEW.json), [PC_RECOMPARE.json](PC_RECOMPARE.json)을 참조한다. 감사자는 전체 suite/CLI를 새로 실행했다고 주장하지 않는다. 독립적으로 새 실행한 것은 identity/archive·정역 복원·raw oracle/selector 확인과 PC 재비교 등 감사 검사다. 20분 시험은 수행하지 않았다.

R06의 effect/state-source 집합 집계 후속 범위, 미작성 공식 콘텐츠, R13+·새 원정·Unity/서버 확장·배포는 이번 Complete의 대상이 아니다. OBS001과 승인된 R05 peak 경고는 그대로 남는다. 이 두 제한을 임의로 넓히지 않았고 현재 필수 계약의 미해결 gap은 없다.

## 종료·후속 자격

**F01 교정 독립 감사 통과.** 이 exact target의 U01 및 현재 R08–R12 Goal은 Complete / High이며 R06/R07 선행 통합 적용성도 복원됐다. [종료 currentness](AUDIT_CLOSURE.json)가 같음을 확인한 시점부터, 연속 요청의 U02 및 후속 작업은 각자의 실제 선행·owner 범위를 인수하여 진행할 자격이 있다. 후속 작업 전체가 이미 구현·감사됐다는 뜻은 아니다. 이후 제품/입력 변경은 해당 consumer pass에서 currentness를 다시 판정해야 한다.
