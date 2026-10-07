# R07 사본별 파편 — 정식 Single Goal

## Goal Set

- Goal Set Ref: GS-CAFCP-001
- Predecessor Goal Set Ref: N/A
- Lifecycle Disposition: current
- Exact Prerequisite Composite Basis: WR-CAFCP-001; FPF-CAFCP-001; PP-CAFCP-001; PVR-CAFCP-001
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CAFCP-001
- Final Policy Freeze Ref: FPF-CAFCP-001
- Complete Goal Member Set: GOAL-CAFCP-01
- Applicable P2G-50 Obligation Key Set: explicit empty
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

P2G 자체는 구현 승인을 하지 않는다. 별도 연속 사용자 요청이 GEX-00/10까지 승인하며 R06 경계 뒤 계속 진행한다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CAFCP-01 | GOAL-CAFCP-01 | AG-CAFCP-01, WP-CAFCP-06 | 1 |

### Integration Acceptance-to-Goal Mapping

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CAFCP-01 / F01 | assigned_execution_node | GOAL-CAFCP-01 / WP-CAFCP-06 | 연속 구현 요청 §5 actual seam + 최종 COVERAGE; source/input/command/raw result |
| IVA-CAFCP-02 / F02 | assigned_execution_node | GOAL-CAFCP-01 / WP-CAFCP-06 | 연속 구현 요청 §5 actual seam + 최종 COVERAGE; source/input/command/raw result |
| IVA-CAFCP-03 / F03 | assigned_execution_node | GOAL-CAFCP-01 / WP-CAFCP-06 | 연속 구현 요청 §5 actual seam + 최종 COVERAGE; source/input/command/raw result |
| IVA-CAFCP-04 / F04 | assigned_execution_node | GOAL-CAFCP-01 / WP-CAFCP-06 | 연속 구현 요청 §5 actual seam + 최종 COVERAGE; source/input/command/raw result |
| IVA-CAFCP-05 / F05 | assigned_execution_node | GOAL-CAFCP-01 / WP-CAFCP-06 | 연속 구현 요청 §5 actual seam + 최종 COVERAGE; source/input/command/raw result |
| IVA-CAFCP-06 / F06 | assigned_execution_node | GOAL-CAFCP-01 / WP-CAFCP-06 | 연속 구현 요청 §5 actual seam + 최종 COVERAGE; source/input/command/raw result |
| IVA-CAFCP-07 / F07 | assigned_execution_node | GOAL-CAFCP-01 / WP-CAFCP-06 | 연속 구현 요청 §5 actual seam + 최종 COVERAGE; source/input/command/raw result |
| IVA-CAFCP-08 / F08 | assigned_execution_node | GOAL-CAFCP-01 / WP-CAFCP-06 | 연속 구현 요청 §5 actual seam + 최종 COVERAGE; source/input/command/raw result |
| IVA-CAFCP-09 / F09 | assigned_execution_node | GOAL-CAFCP-01 / WP-CAFCP-06 | 연속 구현 요청 §5 actual seam + 최종 COVERAGE; source/input/command/raw result |
| IVA-CAFCP-10 / F10 | assigned_execution_node | GOAL-CAFCP-01 / WP-CAFCP-06 | 연속 구현 요청 §5 actual seam + 최종 COVERAGE; source/input/command/raw result |
| IVA-CAFCP-11 / F11 | assigned_execution_node | GOAL-CAFCP-01 / WP-CAFCP-06 | 연속 구현 요청 §5 actual seam + 최종 COVERAGE; source/input/command/raw result |
| IVA-CAFCP-12 / F12 | assigned_execution_node | GOAL-CAFCP-01 / WP-CAFCP-06 | 연속 구현 요청 §5 actual seam + 최종 COVERAGE; source/input/command/raw result |

### Single Goal Member — GOAL-CAFCP-01

- Goal ID: GOAL-CAFCP-01
- Goal Kind: single
- Phase ID: PHASE-CAFCP-01
- Assigned Final Execution Node Set: AG-CAFCP-01, WP-CAFCP-06
- Covered Policy Element Revision Ref Set: PER-CAFCP-E01-REVIEWED-001, PER-CAFCP-E02-REVIEWED-001, PER-CAFCP-E03-REVIEWED-001, PER-CAFCP-E04-REVIEWED-001, PER-CAFCP-E05-REVIEWED-001, PER-CAFCP-E06-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CAFCP-E01-001, DR-CAFCP-E02-001, DR-CAFCP-E03-001, DR-CAFCP-E04-001, DR-CAFCP-E05-001, DR-CAFCP-E06-001
- Derived Observable Policy Acceptance Coverage: FPF-CAFCP-001의 F01–F12 전체; 중복 요소 참조는 같은 acceptance를 재정의하지 않는다.
- Objective / Contract-Visible Result: 같은 종류 파편 A/B를 각각 소유·개화·선택·장착·이전·분쇄·회수·실전 사용·저장할 수 있다.
- User-Visible Problem: 공통 종류 progress와 count 장착은 A만 개화하거나 특정 사본을 보존할 수 없고 출처가 모호하다.
- User-Visible Completion Condition Set: F01 정상 지급 UUID 독립/실패 원자성; F02 개화·선택 독립과 이전; F03 compatibility/occupied 실패 보존; F04 재료·상태 경계; F05 exact 확인 분쇄; F06 사망 회수; F07 공개 base/awakened 전투; F08 source/lease/fingerprint; F09 새 process 저장; F10 V7 변환; F11 조회 일치; F12 한도·취소·회귀·비용.
- In-Scope / Out-of-Scope: R07 F01–F12와 최종 R06 영향 회귀 포함. GUI, R08+, passive/channel/override, 침식, 대여 전체, 유물, 새 보상 identity, 스킬 취소, 20분 soak 제외.
- Reviewed Current Behavior / Implementation Delta: CRA-CAFCP-001: stacks/progress/dust 기반 종류 강화에서 exact UUID/form 저장소로 cutover. 기존 EconomyGrantState clone/commit 재사용. active definition 대신 copy를 참조한다.
- Goal-Local Source of Truth / Owning Layer: SkillFragmentInventory.copies가 단일 mutable 소유 truth; loadout은 검증되는 참조, static form/recipe는 validated catalog, runtime은 resolver 결과 capsule.
- Goal-Local Invariant Set / Source Refs: FPF-CAFCP-001 E01–E06: copy UUID 유일; base 해금; selected∈unlocked; 양방향 owner 일치; 실패 후보 미commit; old checksum 우선; 기존 delivery identity 불변.
- Code/Data/Schema/Store/Transport/Consumer Boundary: GrantExecutor→EconomyGrantState→copy aggregate/roster→stat_pipeline/player_spawns→runtime action provenance→amount record; active_run/node_flow 검증→restore; snapshot/behavior/application exact 명령/DTO; CloneShape와 retained admission 동시 변경. RON form skill와 recipe reference validation은 load 이전에 수행.
- Consumer Verification Boundary: workspace-confirmed: CoreApplication/PlayerBehavior/world snapshot, authoring TrialSession와 세 release CLI, Rust tests. GUI는 범위 밖이며 외부 배포 consumer는 unknown; wire breaking schema와 명시 importer를 문서화한다.
- Goal-Local Dependencies / Execution Prerequisites: R01/R05 구현 근거와 R06 경계 source를 인수. R06 독립 감사는 미수행이며 blocker가 아니다. GEX PLAN에서 actual resource owner/command/converter 상세 interface를 확정하고 하나의 writer로 AT07 cutover.
- Validation Plan / Completion Matrix: 연속 요청 §5 F01–F12 입력·기대값을 실행 디렉터리 COVERAGE에 actual selector/원자료로 배정. default 공개 RON→정상 grant/equip/Maintenance→battle, actual snapshot/trace, fresh subprocess, exact limit/limit−1/overflow, PC36/R05를 포함. 제안 test를 PASS로 쓰지 않는다.
- Structural Completion / Legacy and Fallback Disposition: mutable stacks/progress/rank/dust 강화 consumer 제거. intrinsic modifier는 소유 사본으로 위장하지 않는다. legacy DTO는 checksum 검증된 명시 변환에서만 읽으며 신규 게임 계산에서 참조 금지. first copy/definition fallback 금지.
- Canonical Documentation Obligation: game_rulebook, skill_fragment_system, data_loading_contract, core_runtime_contract, skill_target_contract와 기존 authoring contract에 실제 구현된 schema/지원/호환만 반영. ROADMAP_STATUS 단일 진입점 갱신.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: R06 경계의 full baseline 인수. source/schema/runtime/save/fixture 변경은 관련 근거 무효화; focused 후 최종 전체 default/feature/check/release/clippy/fmt와 PC36/R05 재실행. PC36 기존 gameplay oracle 유지 또는 승인 rank 제거 영향 분리. concurrent peak 9030188 단독 초과 경고 외 hard guard 유지.
- Assigned Durable Integration Validation Seam Coverage: IVA-CAFCP-01–12는 WP-CAFCP-06 소유; R06 31 acceptance/21 IVA/10 dimensions 최종 current source 영향 재검증. command/env/fixture/toolchain/lock/source hashes, nonzero test수, 실패 이력 보존.
- Applicable Migration/Compatibility/Rollout/Rollback: V7 raw original/checksum/old identity 먼저 검증, explicit V8 converter. sorted definition/count/employee UUID로 새 전용 stream 사본 배정, awakened 승계, 제거 rank read-only 보존. 원본 무변경. R06 archive로 source 복원 가능; 새 save를 자동 구형 decode하지 않는다.
- Goal-Local Known Unknown / Blocker / Stop Condition: 실제 runtime 비용과 전체 검증은 미수행. 구현 결과가 정책과 충돌하면 successor 계획 교정. 환경 외 해결 불가능 blocker만 근거/재개 경로 기록. R06 경계에서 사용자 재승인 대기하지 않는다.
- Applicable Risk / Reviewer Requirement and Evidence: 사본/owner/save shared schema 위험: 후보 원자성·손상 fixture·새 process·양방향 archive 복원 필수. 구현자는 독립 GEX20/30 PASS를 선언하지 않는다.

#### Validation Dimension Matrix

| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | F01–12 | 같은 정의 A/B 획득 후 형태/전투/저장 독립 |
| invalid_input | required | F02–05/F09 | unknown copy/form/reference/field, compatibility 거부 |
| failure_rollback | required | F01/F03–05/F09/F12 | 재료/UUID/roster/receipts/candidate 무변경 |
| retry_idempotency | required | F01/F04–06/F12 | 실패 뒤 정상 재시도 한 번, 중복 해금·회수 무중복 |
| ordering_concurrency | required | F01/F03/F04/F08/F10 | 정렬 UUID/canonical map, 단일 writer, 중복 재료 순서 독립 |
| boundary_empty_max | required | F04/F12 | 빈 inventory, cardinality, checked overflow, exact cap |
| disconnect_restart_persistence | required | F09/F10 | 새 process와 원본 V7 checksum/명시 변환 oracle |
| live_data | required | F07/F11 | default 외부 공개 RON와 release CLI actual battle |
| producer_consumer | required | F01–11 | grant→copy→profile→delivery→trace/DTO/save 동일 identity |
| legacy_fallback_absence | required | F10/F12 | 공통 강화 gameplay 제거, old decode 격리, implicit copy 선택 없음 |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single Goal ID |
| --- | --- | --- | --- | --- |
| CAFCP-E01 | PER-CAFCP-E01-REVIEWED-001 | DR-CAFCP-E01-001 | PHASE-CAFCP-01 | GOAL-CAFCP-01 |
| CAFCP-E02 | PER-CAFCP-E02-REVIEWED-001 | DR-CAFCP-E02-001 | PHASE-CAFCP-01 | GOAL-CAFCP-01 |
| CAFCP-E03 | PER-CAFCP-E03-REVIEWED-001 | DR-CAFCP-E03-001 | PHASE-CAFCP-01 | GOAL-CAFCP-01 |
| CAFCP-E04 | PER-CAFCP-E04-REVIEWED-001 | DR-CAFCP-E04-001 | PHASE-CAFCP-01 | GOAL-CAFCP-01 |
| CAFCP-E05 | PER-CAFCP-E05-REVIEWED-001 | DR-CAFCP-E05-001 | PHASE-CAFCP-01 | GOAL-CAFCP-01 |
| CAFCP-E06 | PER-CAFCP-E06-REVIEWED-001 | DR-CAFCP-E06-001 | PHASE-CAFCP-01 | GOAL-CAFCP-01 |
