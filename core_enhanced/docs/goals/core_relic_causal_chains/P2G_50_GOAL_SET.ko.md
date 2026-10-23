# U10 실제 사건 기반 유물 연쇄와 장비 효과 통합 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CRCE-001
- Predecessor Goal Set Ref: N/A
- Lifecycle Disposition: current
- Exact Prerequisite Composite Basis: WR-CRCE-003; FPF-CRCE-001; PP-CRCE-001; PVR-CRCE-001. Exact digests는 P2G_30_FREEZE_001.json 및 P2G_40_RESULT_001.json에 결속한다. Changes/obligations explicit empty.
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CRCE-001
- Final Policy Freeze Ref: FPF-CRCE-001
- Complete Goal Member Set: GOAL-CRCE-01
- Applicable P2G-50 Obligation Key Set: explicit empty
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

Goal 작성의 상태다. 후속 P2G-60과 GEX 구현·검증·감사는 기존 사용자 연속 지시의 별도 권한으로 이어간다. 단계 완료 자체가 새 권한을 만들지 않는다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CRCE-01 | GOAL-CRCE-01 | AG-CRCE-RUNTIME, WP-CRCE-PUBLIC, WP-CRCE-VERIFY | 1 |

### Shared Validation Baseline Reuse Contract

- User-Approved Directive Ref: PIS-CRCE-001 IN-001/003/004의 현재 연속 요청 및 이전 직접 사용자 역할·검증 범위 지시. 현재 목표와 변경 영향에 필요한 검사만 실행하고 유효한 기존 증거를 재사용한다. 이 직접 지시는 옛 일괄 full-suite 문구보다 우선한다.
- Baseline Evidence Key: owning Goal, 실제 command/selector, fixture/input, relevant source/data/test/config, toolchain/lock/dependency, raw result와 claim을 함께 식별한다.
- Initial Full Baseline Owner / Timing / Command Set: GOAL-CRCE-01이 제품 mutation 전에 U09 target038의 독립 Complete/High/open0와 현재 source 관계를 인수한다. 이 필드의 역사적 Full 명칭은 새 전체 검사 지시가 아니다. U09 final selection과 원 receipts에서 관련 선행 계약을 소비하며 실제 변경/누락 seam만 실행한다.
- Final Full Baseline Owner / Timing / Command Set: GOAL-CRCE-01/WP-CRCE-VERIFY가 최종 통합 source에서 acceptance와 변경 영향에 필요한 집중·회귀·공개 경로·설정·자원/성능 검사를 완료한다. 준비된 묶음 실행은 Sol 한 명이 담당한다. 실제 source 영향에 따라 명령·selector를 PLAN/EXPERIMENTS에 고정하며 전체 suite는 구체 근거가 있을 때만 확대한다.
- Production Input Invalidation / Focused Revalidation Map: schema/plan/publication→V01/02/04/05/10/11/14 및 관련 U08/U09; fact/source/queue/death/resource→V01–09 및 R06/R09/U05; Wedge→V10–12 실제 strike·numeric arithmetic; lifetime/clone/budget/save→V06/12/13/14 및 공유 hot-path 비용. 제품 입력 변경은 해당 seam의 기존 PASS를 current 증거로 자동 유지하지 않는다.
- Test/Fixture/Probe Change Invalidation / Rebaseline Rule: 변경된 seam을 현재 입력으로 nonzero 실행한다. 원 실패·fixture·oracle를 보존하고 한도 상향, workload 축소, 기대값 완화로 실패를 숨기지 않는다.
- Shared Harness/Schema/Persistence/Transport/Dependency Invalidation Rule: 직접·간접 consumer를 따라 영향 집합을 넓힌다. 시험 파일 불변만으로 제품 변경을 비영향으로 판정하지 않는다. successor evidence의 consumer도 새 결속을 읽게 한다.
- Phase-Start Rule: 역할·세션·단계 전환만으로 재실행하지 않는다. 같은 실행 입력은 한 명이 소유하고 실행 중 관련 파일을 고정한다.
- Normal Full-Validation Count: 고정 횟수의 전체 재실행을 요구하지 않는다. initial 유효 선행 증거를 인수하고 final current 필수 범위를 닫는다. 실패·입력 변화·미해결 영향에 필요한 재실행은 같은 묶음에서 처리한다.
- Failure/Zero-Execution/Post-Run-Change Retry Rule: compile, 0시험, skip, 중단, 환경 실패는 기능 PASS가 아니다. 성능은 기능 검사와 격리하고 기존 PC/R05의 workload·한도·과거 경고/환경 한계를 유지한다.

### Integration Acceptance-to-Goal Mapping

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CRCE-V01 / FPF-CRCE-001 PE-CRCE-01: both exact member acceptance sentences; CODE_REVIEW policy-element table | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal/COVERAGE.json V01 and independent test_parts/native+external fixtures to be authored; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-V02 / FPF-CRCE-001 PE-CRCE-02: both exact member acceptance sentences; CODE_REVIEW policy-element table | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal/COVERAGE.json V02 and independent test_parts/native+external fixtures to be authored; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-V03 / FPF-CRCE-001 PE-CRCE-03: both exact member acceptance sentences; CODE_REVIEW policy-element table | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal/COVERAGE.json V03 and independent test_parts/native+external fixtures to be authored; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-V04 / FPF-CRCE-001 PE-CRCE-04: both exact member acceptance sentences; CODE_REVIEW policy-element table | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal/COVERAGE.json V04 and independent test_parts/native+external fixtures to be authored; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-V05 / FPF-CRCE-001 PE-CRCE-05: both exact member acceptance sentences; CODE_REVIEW policy-element table | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal/COVERAGE.json V05 and independent test_parts/native+external fixtures to be authored; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-V06 / FPF-CRCE-001 PE-CRCE-06: both exact member acceptance sentences; CODE_REVIEW policy-element table | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal/COVERAGE.json V06 and independent test_parts/native+external fixtures to be authored; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-V07 / FPF-CRCE-001 PE-CRCE-07: both exact member acceptance sentences; CODE_REVIEW policy-element table | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal/COVERAGE.json V07 and independent test_parts/native+external fixtures to be authored; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-V08 / FPF-CRCE-001 PE-CRCE-08: both exact member acceptance sentences; CODE_REVIEW policy-element table | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal/COVERAGE.json V08 and independent test_parts/native+external fixtures to be authored; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-V09 / FPF-CRCE-001 PE-CRCE-09: both exact member acceptance sentences; CODE_REVIEW policy-element table | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal/COVERAGE.json V09 and independent test_parts/native+external fixtures to be authored; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-V10 / FPF-CRCE-001 PE-CRCE-10: both exact member acceptance sentences; CODE_REVIEW policy-element table | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal/COVERAGE.json V10 and independent test_parts/native+external fixtures to be authored; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-V11 / FPF-CRCE-001 PE-CRCE-11: both exact member acceptance sentences; CODE_REVIEW policy-element table | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal/COVERAGE.json V11 and independent test_parts/native+external fixtures to be authored; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-V12 / FPF-CRCE-001 PE-CRCE-12: both exact member acceptance sentences; CODE_REVIEW policy-element table | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal/COVERAGE.json V12 and independent test_parts/native+external fixtures to be authored; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-V13 / FPF-CRCE-001 PE-CRCE-13: both exact member acceptance sentences; CODE_REVIEW policy-element table | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal/COVERAGE.json V13 and independent test_parts/native+external fixtures to be authored; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-V14 / FPF-CRCE-001 PE-CRCE-14: both exact member acceptance sentences; CODE_REVIEW policy-element table | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal/COVERAGE.json V14 and independent test_parts/native+external fixtures to be authored; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-INTEGRATED / FPF-CRCE-001 integration and CODE_REVIEW 검사 범위와 재사용: Normal external RON with U08 copy/U09 structural effects, actual event child/fork/join/periodic and equipment/Wedge coexistence | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal PLAN/STATE_INVENTORY/COVERAGE/finalization; independent prepared test fixtures and existing runner; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-LIFETIME / FPF-CRCE-001 integration and CODE_REVIEW 검사 범위와 재사용: New transient ancestry/join/periodic/cast counters: release/cancel/expiry/clone/attempt/fresh Continue and log independence | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal PLAN/STATE_INVENTORY/COVERAGE/finalization; independent prepared test fixtures and existing runner; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-RESOURCES / FPF-CRCE-001 integration and CODE_REVIEW 검사 범위와 재사용: Actual producer admission/fanout/queue/clone/retained and hot-path performance regressions on unchanged limits/workloads | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal PLAN/STATE_INVENTORY/COVERAGE/finalization; independent prepared test fixtures and existing runner; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-PREDECESSOR / FPF-CRCE-001 integration and CODE_REVIEW 검사 범위와 재사용: Current affected predecessor contracts; explicit nonimpact/reuse or exact selected regression evidence | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal PLAN/STATE_INVENTORY/COVERAGE/finalization; independent prepared test fixtures and existing runner; actual input/source/raw result와 독립 expected/observed. |
| IVA-CRCE-FINAL / FPF-CRCE-001 integration and CODE_REVIEW 검사 범위와 재사용: Evidence coverage, frozen exact target, isolated forward/reverse restoration and whole independent completion | assigned_execution_node | GOAL-CRCE-01 / WP-CRCE-VERIFY | execution/single_goal PLAN/STATE_INVENTORY/COVERAGE/finalization; independent prepared test fixtures and existing runner; actual input/source/raw result와 독립 expected/observed. |

## Single Goal Member `GOAL-CRCE-01`

- Goal ID: GOAL-CRCE-01
- Goal Kind: single
- Phase ID: PHASE-CRCE-01
- Assigned Final Execution Node Set: AG-CRCE-RUNTIME, WP-CRCE-PUBLIC, WP-CRCE-VERIFY
- Covered Policy Element Revision Ref Set: PER-CRCE-01-REVIEWED-001, PER-CRCE-02-REVIEWED-001, PER-CRCE-03-REVIEWED-001, PER-CRCE-04-REVIEWED-001, PER-CRCE-05-REVIEWED-001, PER-CRCE-06-REVIEWED-001, PER-CRCE-07-REVIEWED-001, PER-CRCE-08-REVIEWED-001, PER-CRCE-09-REVIEWED-001, PER-CRCE-10-REVIEWED-001, PER-CRCE-11-REVIEWED-001, PER-CRCE-12-REVIEWED-001, PER-CRCE-13-REVIEWED-001, PER-CRCE-14-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CRCE-01-001, DR-CRCE-02-001, DR-CRCE-03-001, DR-CRCE-04-001, DR-CRCE-05-001, DR-CRCE-06-001, DR-CRCE-07-001, DR-CRCE-08-001, DR-CRCE-09-001, DR-CRCE-10-001, DR-CRCE-11-001, DR-CRCE-12-001, DR-CRCE-13-001, DR-CRCE-14-001
- Derived Observable Policy Acceptance Coverage: FPF-CRCE-001의 14개 current member에서 그대로 파생한 28문장. V01–V14는 각 요소의 두 문장, 나머지 5개 integration row는 공존·수명·자원·선행 영향·최종 currentness를 연결한다. 새 acceptance를 여기서 만들지 않는다.
- Objective / Contract-Visible Result: 사용자가 정상 RON으로 작성한 유물 사건 효과가 실제 전투 결과에 반응하고, 같은 사본/효과 재방문만 제한하면서 fork·실제 join·독립 periodic·파생 지연을 올바르게 실행한다. 쐐기의 실제 타격 누적과 기존 장비/공명/suppression이 함께 작동하며 상태를 회수하고 복원 경계를 지킨다.
- User-Visible Problem: 현재 numeric/structural 유물 기반만으로는 사건 구독·실제 인과 합류·독립 주기와 쐐기 cast-target actual-strike 증가를 작성해 실행할 수 없다. source/로그/segment를 잘못 사용하면 무한 순환, 합법 발동 누락, 과대 누적 또는 기존 장비 반응의 잘못된 억제가 생긴다.
- User-Visible Completion Condition Set: (1) 정상 작성·장착·실제 사건·queued child·query가 연결된다. (2) 동일 copy/effect 재방문, 형제 독립, 실제 participant union 및 독립/파생 원인이 정확하다. (3) 실제 positive strike별 쐐기와 counter/death/공명/suppression이 공존한다. (4) 로그 유무·이탈·취소·clone·실패·재시도·정상 Continue에서 승인 의미가 유지되고 자원이 회수된다. (5) 28 acceptance·10차원·관련 선행 계약이 current evidence와 whole-Goal 독립 감사로 닫힌다.
- In-Scope / Out-of-Scope: FPF14요소, PP6WP/3node, actual RON/public Core·새 transient state·관련 선행 회귀와 canonical 문서까지 포함. 전역 gameplay cap, 동시 시각 자동 join, 로그 authority, 모든 장비 death chain 금지, Unity/개별 최종보스·후속 U11/U18/U19 고유 작업·전체 roster/획득 밸런스·새 live-battle disk resume는 제외한다. 10%p는 설명 fixture다.
- Reviewed Current Behavior / Implementation Delta: CRA-CRCE-002 및 CODE_REVIEW.ko.md가 actual owner/call graph를 고정한다. Grant/Edit/Recipe compiler, actual StrikeScope/facts, finite queue lease, source carriers, actual BasicAttack trigger/death dispatch, use-gain gate, candidate transaction과 Continue를 확장한다. subscriber/join/independent periodic/Wedge는 새 delta이며 기존 type나 test 존재로 현재 작동한다고 간주하지 않는다.
- Goal-Local Source of Truth / Owning Layer: immutable validated GameData와 개인 relic copy/effect가 작성·정체성 권위다. effective plan은 generation-bound 파생 권위, actual fact producer는 발동 원인 권위다. battle candidate의 source/queue/carrier/join/periodic/cast-target root가 transient 진행과 transaction을 소유한다. durable RunCheckpoint/Continue는 기존 사본/런 상태를 소유한다. 로그·DTO·plan pointer는 실행 권한이나 방문 identity가 아니다.
- Goal-Local Invariant Set / Source Refs: FPF-CRCE-001 PE01–14와 PP-CRCE-001. copy+authored effect 경로 방문, fork 이후 독립, 실제 참가자만 union, join own 피해 허용, independent periodic root vs derived ancestry 보존, native positive strike당 한 번 n×r, samecast/target/copysource 격리, basic-only equipment counter, actual death once와 Corrosion suppression, 공명 양수 common gate, allocation 전 admission와 no partial commit, 마지막 consumer 회수.
- Code/Data/Schema/Store/Transport/Consumer Boundary: relic_data/structural authoring→strict publication/compiled plan/capabilities→World capture/query→actual damage/death producer→native source/subscriber/queue and real join/periodic→기존 damage/resource/target/lease consumer→released projectile/area/DOT/finite flow→clone/retained/transaction/cleanup→normal external RON 및 기존 Continue. AUTHORING↔CAUSAL/STRIKE↔STATE 최소 동시 전환을 runtime group이 소유한다. CODE_REVIEW 설계 조건1–8의 Rust/RON 이름과 내부 자료구조는 Sol의 범위 내 상세 선택이다.
- Consumer Verification Boundary: workspace에서 직접 확인한 GameCore 정상 command/query, World combat effective owner, actual battle queue/trigger/fact/resource, production GameData loader 및 외부 authoring fixture, application Continue 경로를 검증한다. 내부 helper는 fault/lifetime 보조 검증이며 public producer를 대체하지 않는다. 접근/검토하지 않은 Unity나 deployed service의 호환/완료를 주장하지 않는다.
- Goal-Local Dependencies / Execution Prerequisites: U09 current038 전체 독립 Complete/High/open0와 closure publication; current FPF/PP/passed PVR·P2G60→GEX00. Root Astra/high는 설계·code review·품질 조정, 기존 Sol/xhigh는 제품과 sole prepared runner, 별도 Astra/high는 시험/fixture/oracle만 작성, 미참여 Astra/high는 최종 감사다. whole-file ownership을 지키고 test module/registration 포함 파일은 순차 이관한다.
- Validation Plan / Completion Matrix: 독립 Astra의 A–H 묶음과 CODE_REVIEW14행을 실행 가능한 테스트로 작성한다. 실제 copy loop/fork/join/periodic/derived/log/owner·Wedge·equipment/death/lock·resource fault를 native+normal World+외부 RON 경계에서 관찰한다. 관련 R06 actual strike/source, R09 suppression, U05 gain lock, U08 copy/restore, U09 plan/publication/consumption을 변경 영향으로 선정한다. 준비된 검사는 Sol이 묶어서 실행하고 실패 원인별 담당자가 같은 계약으로 교정한다. 마지막 coverage와 exact target 복원 뒤 nonparticipant 감사로 넘긴다.
- Structural Completion / Legacy and Fallback Disposition: 신규 schema의 parse-only/미사용 descriptor, 로그 의존, root-global mutable visited, same-time auto join, helper-only join/periodic, plan pointer로 방문 reset, source reconstruction loss, derived child의 새root 위조, global actor Wedge count, segment별 중복 누적, subscriber의 공명/suppression bypass, unmetered shared mutable owner가 없어야 한다. 기존 numeric/structural/equipment 경로의 허용 의미를 유지하고 새 fixture 전용 제품 분기를 만들지 않는다.
- Canonical Documentation Obligation: docs/game_rules/relics.md, 깊은 쐐기 설계 노트 및 가장 가까운 skill authoring/runtime/query 문서에 actual 지원 작성법, source/인과/진단/수명과 HP-loss 유효 타격 결정을 반영한다. 예시 밸런스와 정식 합의를 구분한다. 독립 완료 뒤 roadmap 현황을 갱신하며 미래 카드까지 설계하지 않는다.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: 위 공통 계약을 따른다. U09선행 원본 selection/receipts를 보존하고 관련 consumer와 28 acceptance/10차원을 현재 delta에 매핑한다. source/queue 크기·clone 등 hot shared path가 변하면 그 경로 비용 회귀를 측정한다. PC/R05가 새 Employee relic 기능의 직접 oracle라고 주장하지 않는다.
- Assigned Durable Integration Validation Seam Coverage: PP의 19 IVA row 모두 GOAL-CRCE-01/WP-CRCE-VERIFY가 소유한다. PUBLIC node의 제품 예제는 Sol, 시험 fixture와 oracle는 Astra가 분리된 파일에서 작성한다. 최종 감사자는 tests를 대신 구현하거나 자신의 이전 설계를 감사하지 않는다.
- Applicable Migration/Compatibility/Rollout/Rollback: 기존 RON의 의미와 source/type/copy/variant/generation, 장착 제한, 저장 권리를 유지한다. 새 transient 상태는 in-memory candidate/clone과 실제 carrier 수명에서 닫고 durable state를 임의 변환/삭제하지 않는다. fresh Continue는 기존 pre-entry/stable phase 계약이다. 무효 작성·예산 초과는 typed 오류와 상태 무변경, exact forward/reverse 복원은 격리 디렉터리에서 검증한다.
- Goal-Local Known Unknown / Blocker / Stop Condition: 현재 required source/material gap은 없다. private module/container/enum 이름·구현 알고리즘과 실제 측정 비용은 구현 중 닫는다. 정책/공개 의미/상태 소유·수명/저장/완료 기준/한도·oracle 변화는 근거와 대안을 Root에 반환해 earliest owner에서 처리한다. 일반 compile/fixture 오류·명확한 제품 결함은 해당 담당자가 자율 교정한다. 기존 위임 안의 판단은 사용자 승인 대기가 아니다.
- Applicable Risk / Reviewer Requirement and Evidence: 주요 위험은 source 재구성 유실, sibling 오염, nonparticipant union/중복 join, 늦은 tick reset, released count 조기 폐기, zero/segment 누적, death blanket suppression와 계량 누락이다. Goal 계획·제품·시험에 참여하지 않은 Astra/high가 전체 14요소/28문장/10차원/3node/현재 관련 선행 증거·원시 결과·복원을 독립 검토해 Complete/High/open0를 판정해야 한다.

### Validation Dimension Matrix

| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | 실제 normal RON/장착/사건 child/fork/join/periodic/Wedge 결과와 query | 모든 신규 공개 기능이 실제 원인에서 예상 결과를 만든다 |
| invalid_input | required | typed schema/reference/interval/target/identity/unsupported payload negative fixture | 위치·원인 진단과 무부분 publication/mutation |
| failure_rollback | required | 실제 ancestry/fanout/join/queue/strike admission one-short와 late fault | HP·자원·queue·방문·receipt·count·계량에 부분 commit 없음 |
| retry_idempotency | required | duplicate participant receipt, failed transaction retry, fresh cast | exactly-once join/strike, 이전 실패 방문 오염 없음 |
| ordering_concurrency | required | 같은 시각 독립 strike, 형제 경로, 실제 join, 장비 일반 효과/능력 순서 | 비참여 경로 독립성과 결정론적 기존 사건 순서 보존 |
| boundary_empty_max | required | empty/missing actor/target, zero HP loss/full shield, fanout/interval/overflow/한도 | 안정된 skip/reject, ghost retarget와 arbitrary gameplay cap 없음 |
| disconnect_restart_persistence | required | clone, release 전후 이탈, normal application fresh Continue/attempt/retreat | transient/durable 의미 구분과 사본/plan 재결속·회수 |
| live_data | required | production loader의 실제 외부 RON과 invalid-data startup | parse-only 없이 명령·전투·조회까지 실제 실행 |
| producer_consumer | required | actual fact→subscriber→queue/real join/periodic→damage/resource/source | mock끼리 일치로 대체하지 않고 실제 원인과 최종 결과 연결 |
| legacy_fallback_absence | required | 실제 source review와 관련 U05/U08/U09/R06/R09 회귀 | duplicate authority/fallback/fixture-only branch 없이 기존 허용 동작 보존 |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CRCE-01 | PER-CRCE-01-REVIEWED-001 | DR-CRCE-01-001 | PHASE-CRCE-01 | GOAL-CRCE-01 |
| PE-CRCE-02 | PER-CRCE-02-REVIEWED-001 | DR-CRCE-02-001 | PHASE-CRCE-01 | GOAL-CRCE-01 |
| PE-CRCE-03 | PER-CRCE-03-REVIEWED-001 | DR-CRCE-03-001 | PHASE-CRCE-01 | GOAL-CRCE-01 |
| PE-CRCE-04 | PER-CRCE-04-REVIEWED-001 | DR-CRCE-04-001 | PHASE-CRCE-01 | GOAL-CRCE-01 |
| PE-CRCE-05 | PER-CRCE-05-REVIEWED-001 | DR-CRCE-05-001 | PHASE-CRCE-01 | GOAL-CRCE-01 |
| PE-CRCE-06 | PER-CRCE-06-REVIEWED-001 | DR-CRCE-06-001 | PHASE-CRCE-01 | GOAL-CRCE-01 |
| PE-CRCE-07 | PER-CRCE-07-REVIEWED-001 | DR-CRCE-07-001 | PHASE-CRCE-01 | GOAL-CRCE-01 |
| PE-CRCE-08 | PER-CRCE-08-REVIEWED-001 | DR-CRCE-08-001 | PHASE-CRCE-01 | GOAL-CRCE-01 |
| PE-CRCE-09 | PER-CRCE-09-REVIEWED-001 | DR-CRCE-09-001 | PHASE-CRCE-01 | GOAL-CRCE-01 |
| PE-CRCE-10 | PER-CRCE-10-REVIEWED-001 | DR-CRCE-10-001 | PHASE-CRCE-01 | GOAL-CRCE-01 |
| PE-CRCE-11 | PER-CRCE-11-REVIEWED-001 | DR-CRCE-11-001 | PHASE-CRCE-01 | GOAL-CRCE-01 |
| PE-CRCE-12 | PER-CRCE-12-REVIEWED-001 | DR-CRCE-12-001 | PHASE-CRCE-01 | GOAL-CRCE-01 |
| PE-CRCE-13 | PER-CRCE-13-REVIEWED-001 | DR-CRCE-13-001 | PHASE-CRCE-01 | GOAL-CRCE-01 |
| PE-CRCE-14 | PER-CRCE-14-REVIEWED-001 | DR-CRCE-14-001 | PHASE-CRCE-01 | GOAL-CRCE-01 |
