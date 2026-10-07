# R05 Single Goal 실행 계획

## Current execution

2026-09-12 · GEX-00 completed; GEX-10 implementation/validation closed for exact-target preparation · `execute / goal-run`. Root/selected member는 `(GS-CARL-P2G50-001, GOAL-CARL-01)`, `PHASE-CARL-01`, assigned nodes `AG-CARL-01`과 `WP-CARL-04`다. Source mutation을 멈췄다. 최종102 원래 public/kernel/native guard,103 통합1410tests,104/105 default-release,106 공개15cases,107 기존소비자13묶음,108 진단10commands,109 음성13cases,110 예산8cases,111 순서/부족,112–116 plain1025/allocator615samples가 통과했다. [수용 증거](ACCEPTANCE_EVIDENCE_001.md)는33acceptance/12IVA/10차원을 대조하며 필수 독립 감사는 pending으로 남긴다. GEX-10 종료는 구현 pass 종료이고 Phase/Goal 완료가 아니다. 필수 선행 R01→R02→R03→R04의 exact successor 감사 복원과 R05 fresh GEX20, parent reconciliation을 계속한다. source별 정확한 증거와 보존 실패는 [EXPERIMENTS](EXPERIMENTS.md), 최신 요약은 [NOTES](EXPERIMENT_NOTES.md)를 따른다. [GEX-00 result](GEX_00_RESULT_001.json)가 실행 eligibility를 확인했다.

요청 근거는 현재 사용자가 [AUTORUN single-r-v2](../../../../roadmaps/content_authoring/AUTORUN_REQUEST.ko.md)를 채택하고 필요한 P2G·eligible GEX 구현·검증·감사를 승인한2026-09-12 active request다. [요청 인수](../../REQUEST_INTAKE_001.ko.md)가 R05를 고정한다. Terminal은 이 R05 Goal의 실제 Complete / High와 상위 R05 요구 대조 또는 실제 중단 조건이다. 다음 R 신규 작업은 승인 범위가 아니다.

적용 지침: P2G `2026-09-11-r5`, GEX `2026-09-11-gex-r4` Reader Contract, GEX-00 및 §§5/6.1–6.8, GEX-10/20. GEX-10 진입 전에 해당 계약과 구현 guardrails를 읽었다. AGENTS는 intake의 상위/관련 workspace 확인에서 없음. 새 canonical/global 지침을 만들지 않는다.

## Exact upstream and directory

- Accepted Handoff: `HA-CARL-P2G60-001`, [원문](../../P2G_60_HANDOFF_001.json) / [P2G-60 result](../../P2G_60_RESULT_001.json).
- Working/Freeze/Plan: `WR-CARL-P2G20-001`, `FPF-CARL-P2G30-001`, `PP-CARL-P2G40-002`; passed `PVR-CARL-P2G40-002`.
- Goal: [GS-CARL-P2G50-001 / GOAL-CARL-01](../../P2G_50_GOAL_SET_001.ko.md), Quality2;9elements/33acceptance/12IVA/10required dimensions/C01–08.
- 실행 디렉터리: `/mnt/f/work/simulator/core_enhanced/docs/goals/content_authoring_resource_lifetimes/execution/single_goal`.
- 역할: 같은 디렉터리 `PLAN.md` current 절, `EXPERIMENTS.md` 실제 시간순 기록, `EXPERIMENT_NOTES.md` 현재 판단. `COMPLETION_AUDIT.md`는 future GEX-20 reviewer가 작성; 현재 verdict 없음.

새 exact Goal은 이번 세션에서 생성됐다. 기존 Topic 파일 inventory와 Goal 문서/전체 goals의 PLAN/EXPERIMENTS/NOTES/AUDIT exact-member 검색에 기존 실행 기록이 없고 선택 디렉터리가 생성 전 존재하지 않았다. source에 기능이 있다는 이유로 과거 실행 이력을 추정하지 않는다. 이 유일 경로에서 후속 pass를 계속한다.

## Baseline and dependencies

Repository `/mnt/f/work/simulator`, branch `cleanup/simulator-dirty-worktree-20260803`, HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`; rustc/cargo1.91.1. [baseline](evidence/BASELINE_001.json)은333 source/data/test/tool/contract/lock files,8,551,924bytes를 고정한다. `SOURCE_BEFORE_001.tar.gz`, full untracked status와 tracked diff를 같은 evidence 경로에 보존했다. 원래 dirty/untracked는 사용자 작업이며 clean HEAD를 기준선으로 대체하지 않는다. source/data canonical 변경은 R05 필요 부분만 추가하고 원래 의미·내용을 보존한다. `core_enhanced/.cargo/config.toml` 원래 삭제 상태도 보존한다.

Goal Set 내 direct predecessor Phase dependency: none. 외부 실제 R01–R04 완료는 required reused foundations다. R04 target003/audit002, R01 target006/audit006, R02 target006/audit005, R03 target004/audit004의 원래 Complete / High와 scoped effect/pack/public/action-source 의미를 인수한다. [이전 영향 판정](../../intake/PREDECESSOR_REUSE_001.json)은4지침 개정과 rapier3줄바꿈만의20 raw hash mismatch를 보존한다. GEX-00 새 대조6,946파일에서 raw20/고유5경로 외 변경0을 확인했다. 네 actual hash는 P2G 인수와 같고 OPERATING_RULES는 이번 요청 연결 단락만 추가됐음을 exact reconstruction으로 확인했다. 과거 manifest를 hash-equal이나 새 감사로 바꾸지 않는다.

워크스페이스 파일·Cargo/기존 tests·external probe·official data 접근 가능. 확인 시 F드라이브 약74GiB, /tmp 약810GiB 여유, 메모리 약27GiB available이었다. 의존 다운로드가 필요하면 sandbox 승인 절차를 따른다. 개인 store/외부 서비스 접근이 필요한 테스트는 실행하지 않는다. 실제 local fixture/temporary output side effect는 첫 실행 전 확인한다.

## Scope, design handoff and validation

Goal의 In-Scope/Out-of-Scope, C01–08/I01–08,10차원과 Plan SC01–03/IVA01–12를 exact 인수한다. [CRA C01–09](../../CODE_REVIEW_001.ko.md)의 proposed store/registry/Run projection/public adapter는 새 구현 전 설계다. GEX-10은 실제 types, key/epoch, causal ownership, ordering, sparse initialization equivalence, result/save field transition과 finite cost 항목을 이 PLAN에 상세화하고 준비를 확인한다. 정책/Phase 경계를 바꾸는 선택은 P2G owner로 반환한다.

AG01의 runtime store·실제 Run owner/save·external declaration/executor는 함께 전환한다. WP04는 모든 durable probes/tests·actual execution·evidence를 소유한다. 감사자가 미완성 source/test를 만들지 않는다. 다른 R의 full Flow/유물·fragment-copy 기능 없이 모든 R05 actual lifetime consumer를 입증해야 한다.

별도 shared baseline 횟수 directive N/A. 기존 유효 baseline은 source/test/fixture/toolchain/lock/claim identity와 non-impact로 재사용하며 새 R05 PASS로 부르지 않는다. source/data/schema/runtime/test/fixture/probe 변경은 영향 seam을 invalid한다. GEX-10에서 Goal의 command/input-to-seam map에 따라 focused nonzero 검증, final integrated core/all-target/default production release·외부 소비자·negative/rollback/비용·결정론을 수행한다. 실패·skip·0 tests를 성공으로 세지 않는다.

## Completion and stop

GEX-10은 C01–08/33acceptance/12IVA/10차원의 실제 source/expected-actual/command/count/outputs를 채우고 canonical 문서와 영향받은 predecessor evidence를 정렬한다. 그 뒤 exact audit target을 고정하고 mutation을 멈춘다. 별도 구현 비참여 reviewer가 fresh GEX-20에서 전체 original 기준을 읽고 verdict를 발행한다. 현재 자체 계획/구조 검사는 독립 감사가 아니다. 필수 증거·reviewer·정책·소비자가 미완료면 전체 Goal 완료가 아니다.

새 사용자 정책 선택·scope/Phase 변경·해소 불가능한 필수 증거/환경·안전하지 않은 dirty 충돌은 해당 owner로 반환/중단한다. 해결 가능한 같은 범위 기술 실패는 교정·재검증한다. 질문이 필요한 경우 주 작업을 멈추고 사용자 답변을 기다리며 elapsed time을 승인으로 해석하지 않는다.

## GEX-10 상세화 — 선언 경계부터

첫 구현 단위는 `game::resource`의 typed 선언과 식별자다. 실제 요구되는 owner kind는 Employee/EquipmentCopy이며 여섯 lifetime을 별도 field로 둔다. source selector는 SourceEmployee/SourceEquipment/TargetEmployee로 역할을 명시하고 선언 owner kind와 검증한다. 정의ID와 실제 owner UUID를 다른 자료형으로 구별한다. `ResourceCatalog.resources` 및 각 선언의 id/owner/lifetime/initial/cap은 strict required RON이며 누락 default를 허용하지 않는다. initial≤cap, stable nonempty ID와 중복 정의를 검증하고 native clone visitor를 함께 둔다.

이 단위는 아직 production loader나 effect opcode에 게시하지 않는 AG01의 진행 중 코드다. 현재 지원 capability나 R05 완료로 세지 않는다. 다음으로 candidate-owned 값/예약/lifetime token을 상세화·구현하고 raw catalog와 real execution, Run AT05를 같은 AG 전환 안에서 연결한다. 새 Run 값이 실제 생성되는 producer 연결 전 writer/reader/result/restore를 준비한다. 내부 선언 검사 테스트는 strict/owner 경계를 직접 판정하며 실제 public 여섯 lifetime 인수를 대신하지 않는다.

## GEX-10 상세화 — authority와 예약 경계

ResourceStore는 immutable catalog를 참조하고 실제 employee/equipment-copy owner 집합, private token을 가진 scope registry, sparse entry, reservation을 candidate 값으로 소유한다. BTree 순서, 명시 finite entry/scope/reservation/ID bound와 native clone visitor를 도입한다. 같은 ordinal을 다시 발급해도 token pointer identity가 다르면 stale이며 종료된 scope/삭제된 owner의 lease는 다시 살아나지 않는다. 포인터는 durable 저장이나 관찰 권한으로 변환하지 않는다. sparse 미기록 값은 선언 initial이다.

원자적 consume의 reserve/commit/abort는 동기 공개 transaction 내부 작업이다. authored grant/decay와 미확정 reserve를 비동기적으로 interleave하는 새 gameplay 문법은 만들지 않는다. 여러 비용을 모두 reserve한 뒤 같은 candidate에서 정확히 한 번 commit하며 중간 실패는 외부 candidate를 버린다. 내부 경쟁 예약은 available을 중복 점유하지 못하고, outstanding hold가 있는 값에 대한 일반 감소는 conflict로 거부해 locked 금액을 침해하지 않는다. hold 종료는 abort이고 추가 grant/refund가 아니다. store의 자체 검사는 게임 RNG/queue/observation의 외부 transaction 검증을 대체하지 않는다.

## GEX-10 상세화 — durable Run projection

RunResourceState는 required revision/closed/entries를 가진 strict 구조다. entry는 definition ID·actual owner UUID·balance·entry revision만 저장하며 pointer/scope/reservation token을 제외한다. 정렬·중복·unknown key/owner·lifetime·cap를 검증한다. battle-local mount는 별도 fresh Run scope를 발급하고 기존 값을 복원한다. terminal capture는 pending reserve를 허용하지 않고 full before basis와 after를 보존하며 값 변화가 없더라도 Run revision을 한 번 증가시킨다. 실제 world의 phase/outcome correlation과 함께 result를 한 번 반영해야 한다. floor/checkpoint/entry는 동일 state를 보존하고 terminal은 최종 값을 유지하면서 closed를 설정한다. 아직 이 내부 projection을 실제 save/Run producer에 연결하지 않았으므로 AT05 완료가 아니다.

## GEX-10 상세화 — 실제 runtime 연결

새 Resources catalog는 resolver semantic bytes와 fixture identity, final database/public definition projection에 포함하며 catalog schema3으로 전환했다. official catalog는 빈 목록으로 기존 공식 수치 변경을 피한다. capability의 실제 resource operation 지원은 executor와 AT05가 연결될 때 게시한다.

실제 수명은 기존 RuntimeFactState action acceptance/control terminal/finite delivery admission/close에 자원 registry를 연결한다. 별도 resource context는 RuntimeFactSource의 권한 필드로 전달하고 observation에서는 제거한다. 새 action은 새 Activation과 기존 Chain의 child hold 또는 새 Chain root를 가지며, terminal은 Activation 종료와 자신의 Chain hold 해제를 분리한다. finite delivery는 별도 Delivery scope/Chain hold를 가지므로 caster 철수·control 취소 이후 방출된 projectile/fixed area를 유지한다. 실제 deployment 생성/철수/사망은 unit→deployment scope registry를 갱신한다. Run mount는 최종 application inventory/roster owner graph로만 승인하고 standalone scenario의 임의 UUID로 owner 권한을 생성하지 않는다. 새 fault는 기존 structured runtime fault port로 전달하며 void cleanup의 무시된 오류도 enclosing budget에 latch해 성공으로 게시되지 않게 한다.

## GEX-10 상세화 — AT05 실제 삽입점

실제 setup은 battle startup에서 registry를 reset하고 time0 사건을 queue에 넣을 뿐 아직 dispatch하지 않는다. 따라서 commit_planned_live_battle_start에서 entry snapshot을 capture하기 전 최종 owner graph를 reconcile하고, reset이 끝난 planned BattleCore에 Run overlay를 mount한 뒤 기존 time0 advance로 들어간다. terminal payload는 active core를 없애기 전에 capture하며 CompleteCombatResult의 기존 with_state_transaction 내부에서 resource result를 먼저 적용한다. 그 뒤 어느 staged reward/condition/node operation이 실패해도 같은 Run candidate를 되돌린다. standalone scenario의 UUID로 Run owner를 생성하지 않는다.

RunState의 새 typed_resources root는 checkpoint macro와 native clone visitor에 포함된다. BattleEntrySnapshot은 원래 RunState 전체를 capture/restore하므로 해당 root를 그대로 인수한다. floor helper는 root를 복사하며 최종 application commit은 최종 owner graph에 대해 한 번 reconcile/validate하고 RunComplete/RunFailed/GameOver의 spending을 닫는다. latest-only active-run codec는 V7로 옮기고 V6 이하를 변환 없이 거부한다. checkpoint reader는 root의 strict fields/정렬/owner/key/cap/open 상태를 복원 전에 검증한다. 이것들은 실제 producer/save/result와 함께 이어서 구현할 대상이며 아직 적용 완료가 아니다.
