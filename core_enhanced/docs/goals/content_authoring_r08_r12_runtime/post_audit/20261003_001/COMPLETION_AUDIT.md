# R08~R12 독립 완료 감사 — Not Complete

2026-10-02 시작 / 2026-10-03 종료(KST). `GEX-20`, `GS-CARX-001 / GOAL-CARX-01 / HA-CARX-001`.

**판정: Not Complete. 완료 차단 finding F01이 공개 RON·release CLI·정상 Core 전투에서 재현됐다.**
X09 환경 제한은 해소됐지만 전체 시험 성공만으로 선행 감사를 통과시키지 않는다.
사용자의 R13~R16 요청 §2에 따라 종속 구현은 시작하지 않는다. R13~R16 정식 계획 검증·구현·최종 target 동결·최종 독립 감사는 수행하지 않았다.

## 권한·독립성·원문

사용자는 선행 감사 실행을 이번 요청에 포함했고, 실패 시 근거와 교정을 보고하되 종속 구현을 시작하지 말라고 지시했다.
이 reviewer는 R08~R12 제품 설계·구현·교정·시험 작성에 참여하지 않았다. 현재 pass는 제품 변경 권한을 사용하지 않는 fresh review다.
별도 사람/agent 또는 blind review를 수행했다고 주장하지 않는다. GEX `2026-09-13-gex-r7` §6.7 및 GEX-20의 비참여 reviewer 요구를 적용했다.
감사 중 생성한 것은 신규 감사 문서·입력 복사본·실행 증거뿐이며, 제품 코드·시험·live RON·선행 target/records는 수정하지 않았다.

기준: [정식 Goal](../../P2G_50_GOAL_SET.ko.md), [정식 인계](../../PLAN_HANDOFF.ko.md),
[구현 인계](../../execution/single_goal/REVIEW_HANDOFF.md), [55조건 원 수용표](../../execution/single_goal/COVERAGE.json),
[R08~R12 설계·수용 원문](../../../../roadmaps/content_authoring/handoffs/R08_R12_DESIGN_AND_ACCEPTANCE_20260930.ko.md),
[R13~R16 연속 지시서](../../../../roadmaps/content_authoring/handoffs/R13_R16_CONTINUOUS_IMPLEMENTATION_REQUEST_20261002.ko.md).
Single Goal이므로 존재하지 않는 Master/GEX-30을 만들지 않았다.

본문 식별자 정정: 이 문서는 동결된 감사 본문의 current digest 오기 한 곳을 정정한 문서 successor다.
원 AUDIT_TARGET·각 실행 receipt·AUDIT_CLOSURE는 모두 아래 정확한 `9a0b88be…` 값을 기록했고 실행 대상의 변경은 없다.
기존 보고서/manifest를 덮어쓰지 않았다. 자세한 보존 관계는 [정정 기록](PUBLICATION_RECORD.json)에 있다.

## exact 대상·현재성·보존

- 동결 manifest SHA-256: `9842985297f11fefcfeda91cade469cee4f172a74f4498238a02e7e758686a1c`.
- 동결 source: `bd89f9c42191217a333d648d35f495e802d13e6fb1420caa9c08da3deb731690`, 387 files.
- 감사 current source: `9a0b88beb24c1371c3362837e924ccf447832efb9372e0be1b8e1c99a6f72912`, 387 files.
- membership 추가/삭제 없음. 변경은 `ROADMAP_STATUS.md`의 R13~R16 후속 요청 안내 7줄뿐이다. 제품/빌드/시험/현재 runtime 계약 bytes는 동결본과 같다.
- 참조35개 중 위 현황 및 `game_rules_next/equipment-and-fragments.md`, `open-decisions.md`에 차이가 있다.
  [실제 diff](../../reviews/r08_r12_20261002_001/evidence/reference_delta.patch)를 검토했다. 차기 원정·훈련 파편·유물 합의이며, 이번 R08~R12의 현재 전투·저장 계약을 소급 전환하지 않는다.
  R13~R16 유물 합의는 후속 구현 입력이고 이번 판정에서 구현된 것으로 취급하지 않았다.
- currentness는 **current / proven non-impacting delta**다. 과거 digest와 현재 digest를 같은 값이라고 쓰지 않는다.
- 동결 records **1,435개**, archives/patches **9개**, 네 archive의 member/bytes가 모두 일치한다.
  before368/after387/records1435/references35를 직접 해시 비교하고 격리 임시 디렉터리에서 `patch --fuzz=0` 정·역 복원을 재실행했다.
  추가·삭제 membership까지 일치했다. 원본 작업 트리에 역 patch를 적용하지 않았다.

근거: [AUDIT_TARGET.json](../../reviews/r08_r12_20261002_001/AUDIT_TARGET.json), [검사 코드](../../reviews/r08_r12_20261002_001/verify_target.py),
[정방향 로그](../../reviews/r08_r12_20261002_001/evidence/restore_forward.log), [역방향 로그](../../reviews/r08_r12_20261002_001/evidence/restore_reverse.log).
기존 dirty/untracked 변경을 이 감사 성과로 귀속하지 않는다. 모든 신규 명령 receipt가 전후 source 동일성을 기록한다.

## F01 — 보스 action 취소가 이미 시작된 Flow의 미방출 continuation을 종료하지 않음

**중요도: High / 완료 차단.** R08 task와 R10 Flow의 통합 수명 결함이다.

`boss_runtime.rs::detach_boss_action`(2092행 이후)은 `cancel_unreleased`가 참이어도 해당 action이
`unit.pending_skill_cast`에 남아 있을 때만 `cancel_boss_owned_work`를 부른다.
준비가 끝나면 pending cast는 사라지지만 `active_skill_casts`의 structured Flow와 이후 Wait/Action은 남는다.
따라서 timeout의 `CancelUnreleased`와 강제 state 이탈은 result waiter만 detach하고 그 Flow를 끝내지 않는다.
`cancel_current_state`(1957행 이후)의 StartAction도 이 경로를 사용하며 RouteCombat이 없으면 추가 combat 취소를 하지 않는다.
`skill_runtime/flow.rs::drive`의 Wait/Sequence는 남은 owner를 정상으로 보고 다음 Action을 방출한다.
일반 InterruptCast/사망에는 별도 `cancel_structured_flows` 연결이 있어 기존 취소 시험이 이 누락을 잡지 못했다.

계약은 R08 설계 §2의 “state 이탈은 그 activation이 소유한 미방출 작업만 취소”,
R10 설계 §4의 “cancel은 미방출 continuation을 닫고 방출된 자식의 계약상 수명을 보존”이다.
명시 `Detach` 및 정상 Released milestone 완료는 이 취소 의무와 구별해야 한다.

### 실제 공개 재현

동결 구현의 `r12_public_cli_003/effectssettled` 팩을 새 감사 위치에 복사했다.
공개 스킬의 `Sequence(projectile → Wait(1000) → area)`, 보스 `EffectsSettled` waiter,
취소/전이 확인용 별도 marker 스킬만 사용한다. 정상 새 게임·직원 선택·노드 진입·직원 2명 배치·500ms elapsed를 사용하며
private BattleCore 상태 주입, 제품/test source 수정, 결과 주입은 없다. 예시 수치는 테스트 값이다.
원 팩과 달라진 RON 및 모든 입력은 [002 팩](../../reviews/r08_r12_20261002_001/evidence/timeout_marker_probe_002/MANIFEST.json),
[003 팩](../../reviews/r08_r12_20261002_001/evidence/timeout_marker_probe_003/MANIFEST.json)에 고정했다.

| 입력 | 전이 marker | 첫 projectile 피해 | 후속 area | 계약 oracle |
| --- | --- | --- | --- | --- |
| 준비 중 timeout50 / CancelUnreleased | 5051ms | 없음 | 없음 | PASS, 조기 취소 대조군 |
| timeout1500 / Detach | 6501ms | 5885ms, HP −1 | 6885ms, 두 직원 각각 HP −1 | PASS, 명시 유지 대조군 |
| timeout1500 / CancelUnreleased | 6501ms | 5885ms, HP −1 | **6885ms, 두 직원 각각 HP −1** | **FAIL**, Wait 뒤 미실행 area가 남음 |
| 비행 중 timeout500 / CancelUnreleased | 5501ms | 5885ms, HP −1 | **6885ms, 두 직원 각각 HP −1** | **FAIL**, 비행 payload 보존은 맞지만 후속 area는 취소돼야 함 |
| Wait1500에 의한 강제 state 이탈, timeout20000 | 6501ms | 5885ms, HP −1 | **6885ms, 두 직원 각각 HP −1** | **FAIL**, timeout과 별개 이탈에서도 재현 |

보스 시전 접수는 5000ms, 준비 종료는 5100ms, projectile 방출은 5300ms다.
marker는 해당 TimedOut/Canceled 경로에 도착해야 실행되므로 marker 뒤 area의 HP 손실은 단순 시간 추정이나 표시 잔상 문제가 아니다.
두 직원은 area에서 각각 179→178, 180→179가 된다. `CancelUnreleased`의 전체 공개 전투 event는 trace off/on에서 동일하다.

[기계 oracle](../../reviews/r08_r12_20261002_001/evidence/CANCELLATION_ORACLE.json)은 **5 case 중2 PASS/3 FAIL**이며
[oracle 명령 receipt](../../reviews/r08_r12_20261002_001/evidence/cancellation_oracle_001.json)의 exit code는 **1**이다.
CLI validate/simulate의 exit0은 입력·명령 실행 성공이고, 계약 PASS가 아니다.
초기 marker probe는 도달 불가능 node를 남겨 compiler가 거부했다. 그 실패 원자료를 보존하고 002에서 정상 reachable graph로 다시 실행했다.

### 교정 owner·수용 조건

owner는 **GOAL-CARX-01의 R08/R10 통합 교정 GEX-10**이다. 이 감사에서 수정하지 않았다.

1. 취소된 정확 action이 소유한 미실행 Flow continuation을 종료하고 waiter/예약 wake/retention을 회수해야 한다.
2. 이미 방출한 projectile/area의 피해·출처·resource lease는 계약상 종료까지 보존해야 한다.
3. 동일 actor의 다른 action, 이후 state의 marker, 명시 Detach 및 정상 Released 완료는 취소하지 않아야 한다.
4. 위 5개 공개 입력을 회귀 조건으로 만들고 cancellation oracle의3실패를 해소해야 한다.
   state 이탈·timeout·stale wake·같은 시각 경계·fault rollback/retry와 R06/R07 provenance/resource 회귀도 재검증해야 한다.
5. 별도 교정 target·currentness·정역 복원을 만든 뒤 비참여 reviewer의 fresh GEX-20에서 F01과 영향 조건을 재감사해야 한다.
   옛 target/감사를 덮어쓰거나 이 감사를 PASS로 변경하지 않는다.

직접 영향: **A08-03, A08-04, A10-04, A10-06**. **X10의 ‘모든 필수 A조건 완료’도 충족되지 않는다.**
R08/R10의 action 수명과 결과 관측을 소비하는 R13~R16 묶음은 선행 통과 조건을 만족하지 못한다.

## 55조건·선행 영향·source 검토

[CONDITION_REVIEW.json](../../reviews/r08_r12_20261002_001/CONDITION_REVIEW.json)은55개 원 조건, 기존 selector, 새 default/feature 실행, F01 영향을 모두 추적한다.
모든 기존 selector는 새 전체 실행에서 발견됐고 통과했다. 이 사실을50개 독립 조건 전체 PASS로 확대하지 않는다.
실패가 확인된4조건과 X10 완료 conjunction을 구별하고, 다른 행에는 검토한 범위의 기존 oracle 재실행 결과를 기록했다.
기존 테스트 목록이 본 finding의 모든 취소 조합을 포함했다고 주장하지 않는다.

직접 대조한 주요 producer/consumer는 다음과 같다.

| 범위 | 검토 내용과 근거 |
| --- | --- |
| R08/R10 | StartAction catalog validation→opaque handle/waiter→pending cast/Flow occurrence→projectile/area→terminal/detach. 새 F01에서 source-level 연결 누락 확인. 기존 positive milestone/finite bounds/result branch 시험은 재실행 성공 |
| R09 | corrosion 실제 gate의 exact fragment/form·공명·trauma, capture한 적만 mandatory death, `dispatch_optional=false`, 직원 대가 기록 후 release/terminal, host elapsed freeze. node 완료의 counter·자산 회수 및 V9 명시 변환을 코드 대조하고 전체 해당 시험 재실행 |
| R11 | source record/base 재합성·expiry, aura membership/family priority, channel 제한과 실제 periodic attack 소비, Hello exact activation/remaining time·Goodbye 별도 출처, 이미 방출한 overlay payload. 기존 정상 Core/외부 RON 시험 재실행 |
| R12 | 개인→공유 priority/ordinal 흡수, actual HP receipt·생존, HP cost 분리, control 이동, parry counter/attachment와 basic HP-loss 흡혈. 소진·fault·rollback 및 두 직원/침식 교차시험 재실행 |
| 저장 | `active_run.rs` V9 version/checksum 및 V7/V8 별도 converter, 입력 bytes 유지, `node_flow`의 cloned roster/inventory/fragments 회수 commit. fresh Core/checkpoint/retreat selectors 재실행 성공. 새 전투 중 durable 저장 기능으로 확대하지 않음 |

[R06_R07_IMPACT_REVIEW.json](../../reviews/r08_r12_20261002_001/R06_R07_IMPACT_REVIEW.json)은 R06 **31수용→21 integration** 및 R07 **F01~F12**를 새 실행에 연결한다.
R06 Snapshot/Live·단위/overflow·실제 HP-loss·source-copy·trace/resource 수명, R07 사본/형태/개화/장착·저장·회수를 포함한
모든 지정 selector가 양쪽 suite에서 통과했다. R06/R07 옛 Complete/High는 옛 source에만 남긴다.
현재 거절된 통합 source에 대한 별도 Complete/High를 발행하지 않으며, action/Flow 교정 후 영향 검증을 갱신해야 한다.

## X09·검증 재사용

| 항목 | 이번 감사의 실제 처리 |
| --- | --- |
| socket 단일 시험 | 정확 selector1개 PASS, 1619 filtered. 예전 EPERM 제한은 현재 환경에서 재현되지 않음 |
| 기본 all-target 원명령 | `cargo test -p game_core --all-targets --offline --locked`: **1677 PASS/0 FAIL**, 제외/ignored0 |
| feature all-target 원명령 | `cargo test -p game_core --all-targets --features test-fixtures --offline --locked`: **1677 PASS/0 FAIL**, 제외/ignored0 |
| release | `cargo build -p game_core --release --bins --offline --locked` 실제 exit0. 이 binary로 새 공개 재현 수행 |
| check/event_log_checks/clippy/fmt | 동일 제품/lock/toolchain의 동결 raw/receipt hash와 source를 확인해4명령 재사용. 새 실행으로 표기하지 않음 |
| 기존 공개 CLI | 9팩45명령의 exact receipt/log/source를 확인해 재사용. 새 공개 취소 입력의 validate/simulate/trace는 별도 실행 |
| PC36 | 동결 원자료576행을 predecessor와 다시 비교:36조건,31success/5typed refusal, gameplay/completion/error-code 차이0. requested detail 차이80은 기존 기록대로 유지 |
| R05 | 원 raw의 cap/workload/tool source·allocation/retained/parity/repeatability를 재계산. concurrent peak9,372,846, cap9,030,188의 **342,658 bytes 초과만 합의 경고**. 원 runner exit101을0으로 바꾸지 않음 |
| 복원 | 격리 정·역 patch fuzz0, bytes/membership 직접 재실행 PASS |

원자료: [실행 기록](../../reviews/r08_r12_20261002_001/EXPERIMENTS.md), [재사용 명세](../../reviews/r08_r12_20261002_001/REUSED_EVIDENCE.json),
[PC 재비교](../../reviews/r08_r12_20261002_001/evidence/PC_RECOMPARE.json), [R05 재계산](../../reviews/r08_r12_20261002_001/evidence/R05_RECHECK.json).
모든 신규 명령은 기본 병렬 시험을 유지했고 20분 시험은 수행하지 않았다. clippy/build 경고를0개라고 주장하지 않는다.

초기 offline 시도는 소실된 `/tmp` Cargo 캐시 때문에 rand 해석 이전에 종료했다(0 test, exit101).
`cargo fetch --locked`로 캐시만 복구한 뒤 요청한 offline/locked 명령을 다시 실행했다. Cargo.lock과 제품 source는 그대로다.
예전 EPERM 로그와 최초 의존성 실패를 성공으로 소급 표시하지 않았다.

## 종료와 남은 작업

**선행 감사 실행 종료 / R08~R12 Not Complete(F01) / R13~R16 미착수.**
감사 준비나 승인 대기로 멈춘 것이 아니라 공개 재현으로 확인한 필수 계약 실패 때문에 차단한 것이다.
필요한 다음 단계는 R08/R10 정확 action 취소 교정, 영향 재검증·교정 target 동결, 독립 재감사다.
이번 요청의 실패 분기에 따라 그 교정을 이 감사 pass에 섞거나 R13~R16 종속 구현을 시작하지 않는다.
R13~R16 목표 종료점인 ‘구현·자체 검증 종료 / 최종 통합 target 동결 / 독립 감사 요청 대기’에는 도달하지 않았다.
