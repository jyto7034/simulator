# U11 실험실 검증에서 실제 mod 적용까지 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CMVA-001
- Predecessor Goal Set Ref: N/A
- Lifecycle Disposition: current
- Exact Prerequisite Composite Basis: WR-CMVA-003; FPF-CMVA-001; PP-CMVA-001; PVR-CMVA-001. Exact digests는 P2G_30_FREEZE_001.json 및 P2G_40_RESULT_001.json에 결속한다. Changes/obligations explicit empty.
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CMVA-001
- Final Policy Freeze Ref: FPF-CMVA-001
- Complete Goal Member Set: GOAL-CMVA-01
- Applicable P2G-50 Obligation Key Set: explicit empty
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

P2G 작성 상태다. 후속 handoff와 GEX는 기존 직접 사용자 연속 지시의 별도 권한을 소비하며, 이 문서 자체는 activation/구현 승인이 아니다. P2G-40 passed receipt와 의미 검토를 인수했으며, 구조 검증 성공 후 complete Goal Set으로 선택한다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CMVA-01 | GOAL-CMVA-01 | AG-CMVA-ADMISSION, WP-CMVA-PUBLIC, WP-CMVA-VERIFY | 1 |

### Shared Validation Baseline Reuse Contract

- User-Approved Directive Ref: PIS-CMVA-001 IN-001/002/004/005의 직접 연속 사용자 지시와 실측 위임. 현재 Goal 수용 조건과 변경 영향에 필요한 검사만 실행하고 기존 유효 evidence를 재사용한다. 직접 사용자 지시는 과거 일괄 full-suite 문구보다 우선한다.
- Baseline Evidence Key: owning Goal, command/selector, fixture/input, relevant source/data/test/config, toolchain/lock/dependency, 실제 raw 결과와 claim을 함께 식별한다.
- Initial Full Baseline Owner / Timing / Command Set: GOAL-CMVA-01이 mutation 전에 U10 target u10_current_002/pass001 Complete/High/open0 및 현재 source 관계를 인수하고 recoverable baseline을 고정한다. 역사적 Full 필드명은 새 전체 검사 지시가 아니다. 관련 U02/U03/U04/U07M/U10 계약의 기존 evidence를 인수하며 실제 변경/누락 seam만 실행한다.
- Final Full Baseline Owner / Timing / Command Set: GOAL-CMVA-01/WP-CMVA-VERIFY가 최종 source에서 30개 수용 항목·필수차원·영향받는 선행 조건을 닫는다. local/server default-release 외부 경로, 승인 cache와 fresh persistence, 실제 calibration/경계 및 normal Apply가 필수다. 실제 명령과 selector는 제품 영향에 맞춰 PLAN/EXPERIMENTS에 고정하고 Sol 한 명이 실행한다.
- Production Input Invalidation / Focused Revalidation Map: registry/resolver/build identity→V01/02/06/07/13 및 U02; preparation/coverage/metrics→V02–05/11/13/14 및 U03/U07M/U10 실제 producer; issuer/cache/revision→V04–09/12–14; host/generation/save→V08–12 및 U02/U03/U04. hot producer/clone/layout 변경은 해당 자원/성능 회귀를 포함한다.
- Test/Fixture/Probe Change Invalidation / Rebaseline Rule: 변경된 입력/oracle seam을 current source에서 nonzero 실행한다. 원실패·출력은 보존하고 workload 축소·기대값 완화·계량 누락으로 성공을 만들지 않는다. 정당한 한도/criterion 채택은 actual 전후 측정과 근거를 별도 기록한다.
- Shared Harness/Schema/Persistence/Transport/Dependency Invalidation Rule: 실제 직접·간접 소비자를 따라 영향 집합을 확대한다. 시험 파일 불변만으로 제품 변경을 비영향으로 취급하지 않는다. 새 evidence consumer도 successor 결속을 읽게 한다.
- Phase-Start Rule: 세션·역할·단계 전환만으로 재실행하지 않는다. 실행 중 관련 source/test/fixture/config/tools를 고정하고 동일 입력에 한 실행 owner만 둔다.
- Normal Full-Validation Count: 고정 전체 suite 횟수는 요구하지 않는다. initial 유효 선행 근거 인수와 final current 필수 범위를 완료한다. 수정·실패·불확실 영향에 필요한 재검증은 같은 묶음 안에서 처리한다.
- Failure/Zero-Execution/Post-Run-Change Retry Rule: build·0시험·skip·중단·환경실패는 functional PASS가 아니다. 성능/heap은 기능 검사와 격리하며 plain/instrumented 결과를 구별한다. 기존 실패와 환경 한계는 소급 변경하지 않는다.

### Integration Acceptance-to-Goal Mapping

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CMVA-V01 / FPF-CMVA-001 PE-CMVA-01: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V01; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-V02 / FPF-CMVA-001 PE-CMVA-02: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V02; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-V03 / FPF-CMVA-001 PE-CMVA-03: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V03; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-V04 / FPF-CMVA-001 PE-CMVA-04: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V04; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-V05 / FPF-CMVA-001 PE-CMVA-05: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V05; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-V06 / FPF-CMVA-001 PE-CMVA-06: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V06; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-V07 / FPF-CMVA-001 PE-CMVA-07: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V07; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-V08 / FPF-CMVA-001 PE-CMVA-08: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V08; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-V09 / FPF-CMVA-001 PE-CMVA-09: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V09; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-V10 / FPF-CMVA-001 PE-CMVA-10: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V10; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-V11 / FPF-CMVA-001 PE-CMVA-11: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V11; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-V12 / FPF-CMVA-001 PE-CMVA-12: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V12; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-V13 / FPF-CMVA-001 PE-CMVA-13: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V13; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-V14 / FPF-CMVA-001 PE-CMVA-14: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V14; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-V15 / FPF-CMVA-001 PE-CMVA-15: both exact acceptance items; CODE_REVIEW policy-element delta/seam row | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal/COVERAGE.json V15; independent tests/public probes/measurement evidence to be authored; Nonzero actual current execution and independently derived oracle, exact inputs/raw result; public requirements use real normal consumers. Explicit unchanged/nonimpact reuse only. Failed/incomplete/zero cannot be PASS. |
| IVA-CMVA-CALIBRATION / PE04/05/13; T03 | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal test_parts and finalization evidence; exact test/measurement roster authored before execution; Actual representative normal/expanded/pathological raw measurements→Root concrete versioned criterion adoption→independent exact-boundary/one-over→normal Lab→Apply. No synthetic-only threshold closure. |
| IVA-CMVA-PUBLIC / PE02/08/12; T05 | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal test_parts and finalization evidence; exact test/measurement roster authored before execution; Default-release external RON through local and actual server owner into actual gameplay/query; A retained/B future, raw diagnostic route never counted as normal approval. |
| IVA-CMVA-SAVE / PE08/09/10/11; T06 | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal test_parts and finalization evidence; exact test/measurement roster authored before execution; Writer-capable snapshot/backup/intent/profile/start preflight refusal preserves original bytes; restore required approved configuration and Continue same original in fresh process. |
| IVA-CMVA-REGRESSION / PE11/13/14/15; U02/U03/U04/U07M/U10 | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal test_parts and finalization evidence; exact test/measurement roster authored before execution; Impact-based strict loader/winner/source identity, fault/transaction/clone, paired-intent/strict persistence, native accounting and actual feature/coverage hot-path parity/cost; historical valid evidence reused with explicit nonimpact. |
| IVA-CMVA-STRUCTURE / PE15; GEX finalization | assigned_execution_node | GOAL-CMVA-01 / WP-CMVA-VERIFY | execution/single_goal test_parts and finalization evidence; exact test/measurement roster authored before execution; Current requirement/acceptance/dimension/node and predecessor coverage; source/evidence currentness, isolated exact forward/reverse restoration, nonparticipant whole Goal Complete/High/open0. |

## Single Goal Member `GOAL-CMVA-01`

- Goal ID: GOAL-CMVA-01
- Goal Kind: single
- Phase ID: PHASE-CMVA-01
- Assigned Final Execution Node Set: AG-CMVA-ADMISSION, WP-CMVA-PUBLIC, WP-CMVA-VERIFY
- Covered Policy Element Revision Ref Set: PER-CMVA-01-REVIEWED-001, PER-CMVA-02-REVIEWED-001, PER-CMVA-03-REVIEWED-001, PER-CMVA-04-REVIEWED-001, PER-CMVA-05-REVIEWED-001, PER-CMVA-06-REVIEWED-001, PER-CMVA-07-REVIEWED-001, PER-CMVA-08-REVIEWED-001, PER-CMVA-09-REVIEWED-001, PER-CMVA-10-REVIEWED-001, PER-CMVA-11-REVIEWED-001, PER-CMVA-12-REVIEWED-001, PER-CMVA-13-REVIEWED-001, PER-CMVA-14-REVIEWED-001, PER-CMVA-15-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CMVA-01-001, DR-CMVA-02-001, DR-CMVA-03-001, DR-CMVA-04-001, DR-CMVA-05-001, DR-CMVA-06-001, DR-CMVA-07-001, DR-CMVA-08-001, DR-CMVA-09-001, DR-CMVA-10-001, DR-CMVA-11-001, DR-CMVA-12-001, DR-CMVA-13-001, DR-CMVA-14-001, DR-CMVA-15-001
- Derived Observable Policy Acceptance Coverage: FPF-CMVA-001의 15개 exact member에서 그대로 파생한 30개 항목. V01–15는 각 요소 두 항목, 나머지 5개 IVA는 calibration/public/save/regression/structure 통합을 연결한다. 새로운 수용 의미를 여기서 만들지 않는다.
- Objective / Contract-Visible Result: 사용자가 외부 mod를 정상 실험실 경로에서 실제 실행·계측하여 승인한 뒤 로컬 또는 실제 서버에 적용한다. 정확한 기존 승인만 재사용하고 실패 mod/조합·불완전 계측·위조·stale 결과를 구별하며, 실행 중 세대와 필요한 mod가 없는 저장 원본을 보존한다.
- User-Visible Problem: 현재 정적 pack 검증과 actual Trial은 존재하지만 mod 단위·실제 기능 coverage·근거 있는 합격 기준·주체별 승인/cache·실제 host Apply가 연결되지 않았다. 단순 입력 소진이나 PASS 문자열은 실제 기능과 안전한 적용을 보장하지 못하고 기존 host의 writer-capable 복원은 content 거부 전에 원본을 바꿀 수 있다.
- User-Visible Completion Condition Set: (1) 여러 pack의 mod 및 의존/조합을 올바르게 선택하고 실제 normal/expanded workload에서 필수 기능을 실행한다. (2) 원시 calibration에서 채택한 구체 versioned 기준·경계 판정과 정상 Lab→Apply가 연결된다. (3) 정확 issuer cache는 restart 재사용하고 변조/변경/stale/취소·중단은 승인하지 않는다. (4) 실제 local/server 새 run은 승인한 B를 소비하고 active A는 자기 세대를 유지한다. (5) 필요한 mod가 없는 저장을 변경 없이 거부하고 구성 복구 뒤 같은 원본을 Continue한다. (6) U03/U04 등 영향 계약과 전체 current coverage가 독립 Complete/High/open0로 닫힌다.
- In-Scope / Out-of-Scope: FPF15요소, PP7WP/3node, 정상 public authoring·mod/pack/issuer·actual host/save·측정/합격선·canonical 문서 및 관련 회귀를 포함한다. 중앙 인증 강제, mod 동봉 PASS 신뢰, 파일=mod 가정, 자동 의존 삭제/원본 재작성, 20분 시험, gameplay 인원 cap·저사양 예외·미교정 승인, Unity/개별 최종보스/전체 미래 roster, 자동 다중 저장 슬롯과 숨은 migration은 제외한다.
- Reviewed Current Behavior / Implementation Delta: CRA-CMVA-001과 CODE_REVIEW T01–T07가 actual과 미구현 제안을 구별한다. PackManifest/ContentSelection/Snapshot/resolver와 immutable ValidatedGameData, actual World combat plan/commit, TrialSession 및 mandatory/native observations, measurement window, Core staged generation, real server actor/persistence를 재사용한다. Multi-pack mod owner, 자동 필수 coverage, 교정된 profile, issuer job/cache, normal approved facade와 read-only required-mod preflight가 새 delta다.
- Goal-Local Source of Truth / Owning Layer: authored mod/pack와 검증된 immutable snapshot이 콘텐츠·writer identity를, issuer suite/profile/build와 private owner가 필수 검증 및 승인 권한을 소유한다. 실제 gameplay producer가 coverage/결과를 소유하며 DTO·mod 기대값은 authority가 아니다. host manager가 future selection을, CoreApplication/World가 active generation을, 기존 persistence/paired intent가 durable 수락을 소유한다. cache는 신뢰된 identity/result일 뿐 player save나 live generation을 소유하지 않는다.
- Goal-Local Invariant Set / Source Refs: FPF-CMVA-001 PE01–15와 PP-CMVA-001. Mod 전체 보류와 dependency closure, 최종 조합 actual 검증, exact source/engine/profile/suite binding, trigger/부적격/no-op/효과/종료 구분, 미측정/불완전 승인 금지, plain/instrumented gameplay 동일성, stale/cancel/duplicate 정확 한 번, 승인 A의 파일을 B로 다시 읽어 적용 금지, active A/future B 격리, U03 첫 fault와 U04 accepted intent, writer-capable 동작 전 원본 보호, 유한 상태/계량·마지막 owner 회수.
- Code/Data/Schema/Store/Transport/Consumer Boundary: explicit registry→기존 pack acquire/resolve/publish→immutable candidate→normal authoring World 준비 및 actual producer coverage→finite process-isolated measurement/profile→issuer job/result/authenticated cache→approved normal local/server facade→staged new generation→required-mod read-only persistence preflight→normal Continue. AG-CMVA-ADMISSION의 다섯 WP가 이 authority/transaction 전환을 함께 소유한다. T01–T07의 구체 Rust 타입·모듈·schema·암호 primitive·build wiring은 Sol이 actual API에 맞춰 상세화한다. coverage를 마지막 snapshot 한 개나 재실행 collector로 대체하지 않는다.
- Consumer Verification Boundary: workspace-confirmed는 Core public authoring/Trial/World producer, local CLI, game_server bootstrap/AppState/load-balance/player actor, ActiveRunStore/paired intent와 실제 Continue다. 독립 external probe가 default-release 실제 소비까지 확인한다. 내부 fault helper는 보조 증거이며 정상 host 경로를 대체하지 않는다. canonical-external Unity는 후속 범위이며 접근하지 않은 deployed server/Unity 호환을 완료로 주장하지 않는다. known-inaccessible required consumer는 현재 없고 unknown 배포 상태를 로컬 실제 server 검증과 혼동하지 않는다.
- Goal-Local Dependencies / Execution Prerequisites: U10 current002/pass001 독립 완료 및 관련 U02/U03/U04/U07M 계약을 인수한다. current FPF/PP/passed PVR·P2G60→GEX00의 recoverable baseline 이후 구현한다. Sol은 actual API/examples를 확인하고 concrete schema/metadata version·issuer auth/private-root·build identity·normal preparation와 shared test-facing contract를 짧게 명시한다. Root는 공개 권한/저장 전환을 실제 source에 맞춰 검토하고, 일반 내부 구현은 추가 승인 없이 진행한다. 전체 파일 소유로 embedded test/등록부 충돌을 피한다. Root Astra/high 계획/검토, Sol/xhigh 제품+sole runner, 독립 Astra/high 시험/fixture/oracle, 미참여 Astra/high 최종 감사다.
- Validation Plan / Completion Matrix: V01–15와 5 integration 및 CODE_REVIEW 행을 독립 시험 입력·oracle·공개 경로에 연결한다. 정상 외부 mod와 actual trigger/효과/cleanup, 다중 pack·의존·조합·writer 재선택, cache hit/miss/foreign/tamper/restart, stale/cancel/duplicate/IO 실패, actual host A/B/권한/fault, same-save 복구와 원본 bytes 보호를 묶어 실행한다. 대표 normal/expanded/pathological calibration은 기능 검사와 격리하고 source/build/environment/seed/repeat/window별 raw를 남긴다. Root가 그 근거로 concrete profile을 채택한 뒤 exact/one-over 및 normal Lab→Apply를 검증해야 한다. coverage 실패는 content failure와 instrumentation/setup incomplete로 분류한다. 현재 변경 영향의 U02/U03/U04/U07M/U10을 선정하고 shared producer/clone 비용 변화면 해당 성능도 검증한다.
- Structural Completion / Legacy and Fallback Disposition: 정상 launcher/server에 raw static-load 승인 우회, public PASS→opaque capability 변환, caller 지정 trusted engine/profile, foreign cache trust, per-mod PASS→combination 자동 승인, stale callback 게시, latest-only coverage, 필수 관측 overflow PASS, hidden low-PC relaxation, save preflight 뒤늦은 호출, 임의 migration/slot fallback, test-only 제품 분기가 없어야 한다. 기존 trusted low-level Core/정적 loader는 진단/embedding 기반으로 명확히 유지하며 임의 in-process Rust host 자체를 격리한다는 주장은 하지 않는다.
- Canonical Documentation Obligation: docs/data_loading_contract.md와 docs/core_runtime_contract.md 및 현재 authoring/실험실 API·CLI/server 사용 문서에 actual mod grouping/identity, qualifying approval와 진단 구별, 지원 suite/profile/환경·측정 범위, cache/host 권한, A/B 수명, required-mod 저장 보호와 명시적 별도 persistence root 사용법을 반영한다. 제품의 actual 지원만 기록하며 독립 완료 이후 roadmap/status를 갱신한다.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: 위 공통 계약을 상속한다. 선행 원본과 실패를 보존하며 실제 producer/공유 저장/clone/loader 영향에 따라 선정한다. 새로운 coverage가 old battle hot path에 영향을 주면 그 경로의 actual 비용과 관측 on/off parity를 검증한다. old R05 수치를 새 Lab criterion으로 복사하지 않는다.
- Assigned Durable Integration Validation Seam Coverage: PP의 20 IVA를 GOAL-CMVA-01/WP-CMVA-VERIFY가 소유한다. 제품 PUBLIC 예제·문서는 Sol, 독립 시험/측정 fixture/oracle/comparator는 Astra가 소유한다. prepared 묶음의 일반 교정은 같은 작업 안에서 수행하고 입력 변화/소유만 기록한다.
- Applicable Migration/Compatibility/Rollout/Rollback: V11 continuation/V12 SavePoint와 현재 checksum/full identity, unsupported-original 보호 및 paired intent/CAS를 보존한다. 필요한 metadata/codec/envelope 변경은 version과 기존 reader/writer 전환을 명시해 검토하고, 승인 구성 미일치 원본은 writer-capable read/recovery/profile merge/새 run 덮어쓰기 전에 보호한다. 필요한 구성을 복구하면 같은 원본을 정상 읽는다. 명시적 독립 persistence root가 다른 호환/새 세션을 소유하며 자동 슬롯 정책은 추가하지 않는다. accepted-intent 이후 recovery_pending을 새 전체 rollback 보장으로 바꾸지 않는다. 정확 정·역 source 복원은 격리 경로에서 수행한다.
- Goal-Local Known Unknown / Blocker / Stop Condition: 현재 required source/material gap은 없다. 실제 workload 유효성/측정분포와 concrete numerical criteria는 GEX의 필수 미완료 작업이다. numeric criterion 채택은 현행 사용자 실측 위임 범위에서 Root가 근거와 여유를 검토한다. 새로운 gameplay/저장/공개 권한 의미 또는 필수 정상 경로 불가능성이 나타나면 earliest P2G owner로 반환한다. 일반 compile/import/fixture·명확한 제품 결함은 소유 담당자가 자율 교정한다. 기준 미정 상태를 diagnostic만으로 닫거나 U30으로 넘기지 않는다.
- Applicable Risk / Reviewer Requirement and Evidence: 주요 위험은 가짜 coverage, actor 준비 실패의 오분류, 미교정/혼합 측정 PASS, 불완전 cache key, 선택/파일 TOCTOU, stale result 부활, raw host bypass, save 복구 선행 쓰기, actor fault 우회와 무한 수집 상태다. 본 Goal의 계획·제품·시험에 참여하지 않은 Astra/high가 전체15요소/30항목/10차원/3node와20통합·관련 선행 evidence·원시 calibration/실행·source/복원을 독립 검토하여 Complete/High/open0를 판정해야 한다.

### Validation Dimension Matrix

| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | actual external normal/expanded Lab→adopted concrete criterion→approved local/server Apply→gameplay/query/Continue | 미교정 진단이 아닌 실제 정상 종단 완료 |
| invalid_input | required | mod/pack/dependency/reference/opcode/schema/authority/profile/cache malformed inputs | typed 위치/원인 진단과 승인·live·원본 무변경 |
| failure_rollback | required | trial 준비/coverage/metric/cache IO/host prepare/save preflight 실패 및 accepted intent boundary | 기존 generation/cache/save와 accepted prefix 보존, U04 recovery_pending 구별 |
| retry_idempotency | required | late/duplicate result·cancel·cache restart/partial write·Apply retry | stale 부활 없이 정확 한 번 게시와 유효 cache 재사용 |
| ordering_concurrency | required | selection revision 경쟁·동일 bucket 다중 site·두 issuer/trial/actor | 결과/coverage/profile 격리와 active A/future B 원자적 경계 |
| boundary_empty_max | required | empty/missing/cycle/oversize·setup limit·coverage overflow·criterion exact/one-over | finite 명시 거절·incomplete, 임의 gameplay cap이나 거짓 PASS 없음 |
| disconnect_restart_persistence | required | fresh process cache/Continue·pending intent/backup·required mod 복원 뒤 same original | 원본 current/backup/profile bytes 보호 및 정상 복원 |
| live_data | required | 실제 외부 RON/mod 선택을 production loader와 default-release normal launcher/server가 소비 | private fixture 없는 정상 실행과 잘못된 data startup 거절 |
| producer_consumer | required | immutable snapshot→actual World producer→coverage/metrics→issuer→host/persistence | 실제 consumer 왕복과 source/engine/profile/selection 동일성 |
| legacy_fallback_absence | required | raw/static diagnostic vs normal facade source audit와 영향받는 U02/U03/U04/U07M/U10 | 승인 우회·중복 권한·hidden migration/fallback·test-only behavior 없음 |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CMVA-01 | PER-CMVA-01-REVIEWED-001 | DR-CMVA-01-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
| PE-CMVA-02 | PER-CMVA-02-REVIEWED-001 | DR-CMVA-02-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
| PE-CMVA-03 | PER-CMVA-03-REVIEWED-001 | DR-CMVA-03-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
| PE-CMVA-04 | PER-CMVA-04-REVIEWED-001 | DR-CMVA-04-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
| PE-CMVA-05 | PER-CMVA-05-REVIEWED-001 | DR-CMVA-05-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
| PE-CMVA-06 | PER-CMVA-06-REVIEWED-001 | DR-CMVA-06-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
| PE-CMVA-07 | PER-CMVA-07-REVIEWED-001 | DR-CMVA-07-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
| PE-CMVA-08 | PER-CMVA-08-REVIEWED-001 | DR-CMVA-08-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
| PE-CMVA-09 | PER-CMVA-09-REVIEWED-001 | DR-CMVA-09-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
| PE-CMVA-10 | PER-CMVA-10-REVIEWED-001 | DR-CMVA-10-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
| PE-CMVA-11 | PER-CMVA-11-REVIEWED-001 | DR-CMVA-11-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
| PE-CMVA-12 | PER-CMVA-12-REVIEWED-001 | DR-CMVA-12-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
| PE-CMVA-13 | PER-CMVA-13-REVIEWED-001 | DR-CMVA-13-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
| PE-CMVA-14 | PER-CMVA-14-REVIEWED-001 | DR-CMVA-14-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
| PE-CMVA-15 | PER-CMVA-15-REVIEWED-001 | DR-CMVA-15-001 | PHASE-CMVA-01 | GOAL-CMVA-01 |
