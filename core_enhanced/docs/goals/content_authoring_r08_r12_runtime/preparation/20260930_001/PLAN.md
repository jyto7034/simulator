# R08~R12 조사·계획 준비 — 현재 위치

2026-09-30. **선행 독립 감사 대기 / 준비 조사 완료 / 제품 구현 미착수.**
정식 P2G-30/40/50/60 결과, GEX-00 인수, GEX-10 구현 완료 또는 감사 PASS가 아니다.

사용자 요청은 R08~R12의 계획 준비·구현·자체 검증을 허용하지만 R06/R07 감사가 없으면 준비까지만 수행하도록 제한한다.
[상세 지시서](../../../../roadmaps/content_authoring/handoffs/R08_R12_CONTINUOUS_IMPLEMENTATION_REQUEST_20260930.ko.md) §2와 최신 사용자 요청이 이번 경계를 소유한다.
적용 실행 지침은 GEX `2026-09-13-gex-r7`이다. 과거 계획과 감사 원문을 개정하거나 소급 승인하지 않았다.

## 현재 근거와 재개 조건

- [현재성 원자료](CURRENTNESS.json): 최종 R06/R07 source 367/367, dynamic membership 추가·누락 0. source digest `714195327ec0df7facc60d1aafe721e6e26c1296494b39cff593232c45365e6f`.
- [감사 문서 탐색](AUDIT_DISCOVERY.json): 현재 Markdown 1,871개 검색. exact 통합 target 관련 문서 및 두 구현 인계에서 별도 최종 감사 결과를 발견하지 못했다. 외부 대화의 감사 유무까지 단정하지 않는다.
- R06 단독 경계 361개 중292개 동일/69개 변경/누락0. R07 통합 manifest와는 전부 동일하므로 R06 경계를 current actual로 취급하지 않는다. 의미적 무영향 판정은 수행하지 않았다.
- [원본 보존](BEFORE_MANIFEST.json): 조사/변경 범위388개 실제 파일의 dirty/untracked bytes를 `SOURCE_BEFORE.tar.gz`에 보존, archive 내용 재해시 일치. HEAD만 보존한 것이 아니다. 관련 없는 캐시/거대 기존 산출물/타 작업은 수정하지 않고 git 상태만 기록했다.
- 기존 같은 Topic 정식 계획은 발견되지 않았다. 본 디렉터리는 새 준비 자료이며 정식 Goal/Handoff는 아직 없다.
- 재개에 필요한 exact 선행 감사 범위는 [선행 감사 요청 자료](PREREQUISITE_AUDIT_REQUEST.ko.md). 감사 실행은 이번 요청에 포함되지 않는다.

## 준비한 통합 실행 구조

권장 구조는 Single Goal 하나와 WP08~WP12, WPI(최종 통합)다. 각 WP의 parser→검증→실행→결과/저장→조회 동반 전환은 AT08~AT12로 유지한다.
공유 action/flow/lifetime/damage 계약이 최종 한 source에서 결합되므로 통합 owner가 필요하다. WP를 별도 감사 Phase로 임의 선언하지 않는다.
이는 P2G-40 구조 확정이 아니다. 정식 계획에서 SCC/atomic partition/중간 유효 상태를 검사하고 필수 Phase 감사가 실제 필요하면 그 경계를 보고한다.
제품 단일 writer, 중간 WP 사용자 검토 대기 없음, 자동 하위 agent 위임 없음. 정식 실행 경로 후보는 `../../execution/single_goal/`다.

| WP / AT | 실제 변경 경계와 책임 | 완료할 공개 경로·검증 | 유지할 중간 상태 |
| --- | --- | --- | --- |
| WP08 / AT08 | `data/boss_runtime_data.rs` raw/compiled/result/capability, `core/boss_runtime.rs` activation/admission/cancel, `core/sim.rs` 권한별 admission, `core/runtime_facts.rs` handle retention | 외부 보스 RON→명시 skill→t100 Released/t300 Settled, t50/t150 취소, 잘못된 route/late wake/fault. A08-01~07, T17/26 | 기존 RouteCombat와 wave owner 보존, 현재 delivery만으로 실행. R10 미구현을 빈 executor로 수용하지 않음 |
| WP09 / AT09 | `sim.rs` manual/auto start, `death_runtime.rs`, `world/combat.rs`, `world/application.rs`, `world/state.rs`, `employee.rs`, `world/node_flow.rs`, active_run 저장 | 정상 지급/사본 장착→침식→특수 kill/Withdrawn/trauma100→host freeze→결과→node/recovery/Breakdown→저장. A09-01~10, T19/20/29 | 일반 준비/비용 보존, 미출현 wave 유지, 즉시 terminal에서도 대가 exactly once |
| WP10 / AT10 | `ability.rs`, `data/skill_data.rs`, validation/compiler, cast/projectile/area, source address/result/descriptor/fingerprint | 공개 Sequence/Branch/Parallel/Wait/유한 Repeat→actual target/HP/결과. A10-01~09, T08/23/26/27 | 한 실행 계획, 기존 step 입력 유지 시 compile lowering. R08 waiter/R09 gate 재검증 |
| WP11 / AT11 | modifier/source ledger, `ActionLocks`, aura/channel, basic attack start/release, R05 lease, clone/retained/조회 | 외부 파편→copy/form→태아 channel/Hello→정상 기본 공격→명시 중지/재입력/만료. A11-01~09, T09/27 | source-local 제거, released profile 불변, 다른 lock 보존, 원래 공격 복귀 |
| WP12 / AT12 | effect pool/membership, `damage_runtime.rs`, DamageFact/receipt, buffs/control/movement, cost/생존 | 공개 pool600→A400/B350→HP0/150·잔량0; 개인+공유/패링/HP1/cost/제어. A12-01~10, T10/19/27 | HP-loss와 absorbed 분리, 감쇄1회, 침식 특수 kill 유지, source별 종료 |
| WPI | 최종 공유 source, 실제 CLI/World/저장/PC36/R05/동결 | X01~X10 전부 및 R06/R07 영향 수용표; before/after·정역 patch·격리 복원·currentness | 독립 감사 요청 대기에서 제품 mutation 종료, R13 미착수 |

기능 선행: R08←R01/R04; R09←R04/R05/R07; R10←R01/R04/R05/R06; R11←R05/R06/R10; R12←R04/R05/R06/R11.
이번 묶음의 착수에는 사용자 지정 R06/R07 감사가 추가로 필요하다. R08 자체의 기능 선행을 바꾼 것이 아니다.

## 코드 재대조와 남은 상세 조사

실제 본문을 읽은 범위는 [기술·정책 메모](EXPERIMENT_NOTES.md)와 `CODE_EXCERPTS.log`에 연결한다.
지시서 조사 identity23개 전부 일치해 기존 파일 위치를 재사용할 수 있다. 이것은 모든 consumer의 의미 검토 완료를 뜻하지 않는다.

1. WP08 전에 boss catalog cross-reference와 모든 transition commit 호출자, waiter reservation/release를 추적한다.
2. WP09 전에 공개 elapsed/event-bucket/trial/admin/resync route와 terminal publication, 모든 node/recovery commit을 분류한다.
3. WP10 전에 authored 주소→compiled fingerprint→actual result/amount/trace producer 전부와 lowering 호환 범위를 닫는다.
4. WP11 전에 modifier writer와 stat/attack snapshot consumer, movement/block/field-work lock reader를 전수 대조한다.
5. WP12 전에 HP 변경·attached effect batch·반사·cost·death의 전 경로와 control capability 표를 닫는다.
6. 모든 신규 상태는 clone/retained/pre-allocation admission/DTO/save restore owner와 연결한다. private unit fixture만으로 완료하지 않는다.

## 정식 계획으로 옮길 순서

선행 감사가 제공되면 exact 문서·verdict·target·미해결 finding·currentness부터 인수한다. 변경 source가 있으면 이 준비 기준선을 재확인한다.
P2G-00/10의 승인 계약과 위임 결정, P2G-20 source binding을 준비 자료에서 완성하고, Final Freeze→Phase Plan→Goal→Handoff를 발행한다.
새 canonical 정책이나 미완성 machine artifact를 이번 준비에서 publish하지 않았다.

검증기는 실제 저장소의 `docs/tools/validate_p2g40_receipt.mjs`(1.2.0), `validate_p2g50_goal_set.mjs`(1.3.0)다.
`TOOL_DISCOVERY.json`의 --help 실행은 도구 접근 확인일 뿐 P2G 검증이 아니다. 정식 artifact 작성 뒤 아래 실제 명령을 실행해야 한다.

```bash
node core_enhanced/docs/tools/validate_p2g40_receipt.mjs --write-receipt --plan <plan> --freeze <freeze> --receipt <new-receipt>
node core_enhanced/docs/tools/validate_p2g40_receipt.mjs --plan <plan> --freeze <freeze> --receipt <new-receipt>
node core_enhanced/docs/tools/validate_p2g50_goal_set.mjs --freeze <freeze> --plan <plan> --receipt <receipt> --quality-contract 2 --goal-set <goal-set>
```

각각 exit0/result passed/empty failures를 보존한 뒤 GEX-00/10으로 이어간다. 링크 검사·이 준비 계획·기존 R07 계획 검증으로 대체하지 않는다.

## 검증·동결 계획과 미완료

[COVERAGE.json](COVERAGE.json)은 A08~A12 45개와 X 10개 전체55조건을 원문 oracle과 WP에 연결한다. **전부 not_run**이며 actual selector/receipt는 아직 없다.
선행 감사 대기 중 제품 시험을 새로 실행하지 않았다. 기존 1581+1581 결과는 인계자의 자체 검증 보고일 뿐 이번 실행이나 독립 PASS가 아니다.

제품 변경 전에 actual source before를 다시 보존하고 적용성 있는 initial regression을 확보한다. focused는 각 변경 영향으로, broad는 초기와 최종 기준으로 수행한다.
최종 필수 명령은 지시서 §7의 default/feature all-target test, check/event_log_checks, release bins, clippy(기존 두 allow), fmt다.
release author_content 실제 validate/preview/explain/simulate, fresh-process restore, PC36 36조건/576rows와 R05 hard guards를 최종 source에서 검증한다.
기본 병렬 시험을 사용하고 0개/interrupted/exit 불명은 성공이 아니다. R05 cap9,030,188 단독 초과만 경고, 과거184,281 초과를 새 cap으로 삼지 않는다.
동결된 R06/R07 run.py/freeze.py는 실행하지 않는다. wrapper·출력 경로를 새 실행 디렉터리로 옮기고 membership/환경을 검토한 뒤 사용한다.

before/after 원본·추가/삭제 membership·정역 patch와 isolated restore를 최종 수행하고 REVIEW_HANDOFF 및 최종 manifest를 작성한다.
이번에는 preparation archive만 있으며 **R08~R12 최종 구현 target은 없다**. 제품 구현·55조건·최종 통합 회귀·최종 정역 복원 모두 미완료다.
차기 원정 지도/억제력/귀환/대여, R13+, 편집기, 실험실 stress/metrics,20분 시험은 제외한다.

## 이번 준비 종료

[최종 준비 검사](FINAL_PREPARATION_CHECK.json)는 exit0: 기존 제품 target367개 무변경, 조사 근거23+13개 current,
보존388개 격리 복원 및 유일한 기존 문서 수정(ROADMAP_STATUS.md)의 양방향 patch 복원을 확인했다.
신규 준비 문서 외에는 현황 진입점만 갱신했다. **다음 필수 작업은 별도 요청의 R06/R07 독립 감사**다.
정식 계획 검증·제품 구현·통합 자체 검증·최종 target 동결은 선행 충족 후 남아 있으며 전체 완료로 보고하지 않는다.
