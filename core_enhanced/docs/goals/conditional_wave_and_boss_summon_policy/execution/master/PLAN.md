# Conditional Wave and Boss Summon — Master Goal Execution Plan

- Root Goal ID: `GOAL-CWBS-MASTER`
- Goal Set Ref: `GS-CWBS-P2G50-002`
- Accepted Handoff Ref: `HA-CWBS-P2G60-002`
- Run Mode: `goal-run`
- Authorization Ref: `GEXAUTH-CWBS-001 / 2026-08-16 user-requested Master goal-run`
- Authorization Scope: selected Goal Set 안의 코드·데이터·문서 수정과 필요한 테스트
- Excluded Authorization: deployment, production promotion, 외부 환경 변경, policy/Goal/Phase 의미 확대
- Current Pass: `GEX-30 complete / AUDIT-CWBS-MASTER-GEX30-001`
- Master Completion: `Complete / High`
- Execution Directory: `core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/execution/master`
- Document Set: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`; `COMPLETION_AUDIT.md`는 GEX-30에서 생성

## Exact Accepted Basis

- Result Composite Basis:
  `WR-CWBS-P2G20-002 + FPF-CWBS-P2G30-002 + PP-CWBS-P2G40-003 + GS-CWBS-P2G50-002 + Declaration Set {COD-CWBS-001}`
- Validation Receipt: `PVR-CWBS-P2G40-003 / passed`
- Upstream structural validation: `P2G-40 passed / P2G-50 passed / 2026-08-16 current observation`
- Goal status at P2G handoff: `Authored / Not Activated / Implementation Authorized: No`
- Current GEX authorization: `GEXAUTH-CWBS-001`이 이 exact Goal Set 범위의 GEX-10 mutation을 새로 승인

## Master Audit Target Record

- Record ID: `ATR-CWBS-MASTER-FINAL-001`
- Capture: `2026-08-17 / fresh GEX-30`, repository HEAD
  `f318414fb0156a67e04c5479000c2a2683fc0c09`, dirty current actual
- Accepted-basis identity:
  - Final Freeze `4c94407a83e3252635d2e033fe792e98dbafa6ce202b0cfc8f52d910780838e8`
  - Phase Plan `ae06c9032419805ea773cf8645fee38c5583f034db590161f55e13b79baa3fea`
  - Phase Plan receipt `0fb94267b9c4c69eff01addcb0d875e0580496db1af662ace796c9342d761958`
  - Goal Set `209dcc5346a31c4d6d0b3c6eada33ca2254ef1954fd64291ede739b38405060b`
  - Handoff `f070c1a9c3cb41846222b5253f4023d784a1f84b2fd54eff9c4e0dece0f63ead`
- Phase-audit evidence identity:
  - P01 `931828afa65da61170a36a6a7558de8fd4a5c0fdd9256bc499e0d6006dab0829`
  - P02 `81f53190ac582529d1dd35b986145b8df2eab7b5561cdca3af4103e8c3543f81`
  - P03 `90a4744e1adc1ab9afc1576e2cc2634594492220934ceccc16152d2720f5b2d9`
  - P04 `34fdd87c3d0ce39ab502537c4e1d235aff762139d9a6c1262804f56729cf9360`
  - P05 `956be3e9e4b6291e28726de92de8793cbe92a76e000341421a0e453d9df511b1`
  - P06 `2acb6a8b20e5e7d8d22be0f3d56e6ea81205b7c20b86342a10902229c3e0c744`
- Final implementation/data/test/canonical identity: P06
  `ATR-CWBS-P06-FINAL-001`의 exact 25-member SHA-256 manifest. GEX-30 intake와 direct-review 종료 시 모두
  `current / unchanged`였다.
- Predecessor target continuity: P01..P05의 frozen target 이후 바뀐 member는 승인된 순차 successor Phase가
  변경했고, 각 successor의 GEX-20 dependency review와 P06 final integrated evidence가 그 seam을 인수했다.
  phase-local 의미를 덮는 owner 없는 delta는 없다.
- Integrated evidence identity: `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_core` = `1,131 / 0`,
  `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_server` = `73 / 0`; total `1,204 / 0`, exact pre/post
  25-member comparison `MATCH`; scoped rustfmt/diff/check passed.
- External actual: network service, deployed runtime, external profile store와 production environment는 이 Goal에
  applicable하지 않으며 접근하거나 변경하지 않았다. Live actual은 repository loader가 읽는 current RON이다.
- Tracked dirty, untracked and deleted paths outside the Phase manifests remain pre-existing user-owned current actual.
  They are excluded from this completion claim; reviewed overlap is represented by the Phase records and final manifest.
- Master `COMPLETION_AUDIT.md`, execution-document updates and audit-local output are implementation target에서 제외한다.

## Objective and Phase Registry

Master는 구현을 직접 소유하지 않고 다음 Phase Goal을 순서대로 조정한다.

| Order | Phase Goal | Phase | Execution directory | Current result |
| --- | --- | --- | --- | --- |
| 1 | `GOAL-CWBS-P01-RUNTIME-FOUNDATION` | `PHASE-CWBS-01-RUNTIME-FOUNDATION` | `../p01_runtime_foundation` | `Complete / High — Audit Pass 3` |
| 2 | `GOAL-CWBS-P02-WAVE-CUTOVER` | `PHASE-CWBS-02-WAVE-CUTOVER` | `../p02_wave_cutover` | `Complete / High — Audit Pass 1` |
| 3 | `GOAL-CWBS-P03-SUMMON-CUTOVER` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `../p03_summon_cutover` | `Complete / High — Audit Pass 1` |
| 4 | `GOAL-CWBS-P04-OUTCOME-CUTOVER` | `PHASE-CWBS-04-OUTCOME-CUTOVER` | `../p04_outcome_cutover` | `Complete / High — Audit Pass 1` |
| 5 | `GOAL-CWBS-P05-COMPENDIUM-CUTOVER` | `PHASE-CWBS-05-COMPENDIUM-CUTOVER` | `../p05_compendium_cutover` | `Complete / High — Audit Pass 1` |
| 6 | `GOAL-CWBS-P06-REFERENCE-INTEGRATION` | `PHASE-CWBS-06-REFERENCE-INTEGRATION` | `../p06_reference_integration` | `Complete / High — Audit Pass 1` |

Relations: `P01 -> P02 -> P03 -> P04 -> P05 -> P06`; parallel execution은 승인되지 않았다. 각 Phase는 별도
`GEX-00 -> GEX-10 -> fresh GEX-20` pass를 거치고, 전 Phase `Complete / High` 뒤 fresh GEX-30을 수행한다.

## Shared Validation Contract

- P01은 production mutation 전에 `game_core`와 `game_server` full baseline을 정확히 한 번 실행한다.
- Phase 시작 자체로 full baseline을 반복하지 않는다.
- production/test/shared-boundary input 변경으로 invalidated된 seam만 focused rebaseline한다.
- P06은 모든 구현과 focused rebaseline 뒤 integrated full command set을 정확히 한 번 실행한다.
- 실패, zero execution 또는 relevant input 변경 뒤의 결과는 reusable success baseline이 아니다.

## Master Completion Conditions

- P01..P06이 dependency 순서대로 current `Complete / High` GEX-20 감사를 가진다.
- `IVA-CWBS-01..12`가 current implementation/validation evidence에 연결된다.
- P06 final full baseline과 canonical/data alignment audit가 성공한다.
- fresh GEX-30에서 final current tree, cross-Phase invariants와 integration boundary가 `Complete / High`다.

## Stop Conditions

- material policy나 사용자-visible 의미 결정 필요
- Goal/Phase/dependency/Work Package/Atomic Group 경계 변경 필요
- exact upstream basis stale/contradiction
- existing dirty work와 remaining mutation을 안전하게 분리할 수 없음
- 배포·production promotion·외부 환경 권한 필요
- run-global blocker 또는 필수 consumer/evidence 부재
