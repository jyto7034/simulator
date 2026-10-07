# P01 Preservation Baseline Current Notes

## Current Summary

- Current pass/result: `P01 GEX-10 stopped / run-global scope change`.
- Original execution basis: `HA-CRAR-P2G60-001` / `GS-CRAR-P2G50-001` goal-run.
- Current user decision: Unity를 검사하지 않고 `core_enhanced` 리팩토링에만 집중한다.
- Current write/read/validation target: `/mnt/f/work/simulator/core_enhanced` only.
- GEX-20 and later phases: not started.
- Required next owner: `P2G-10` successor scope revision.

## Binding Scope Rules

1. Unity repository, source, documents, fixtures, editor/MCP, status/hash와 tests를 읽거나 실행하지 않는다.
2. `game_server`를 remediation target이나 completion consumer로 사용하지 않는다.
3. Core behavior와 public projection은 Core-owned source와 executable Core tests로만 검증한다.
4. stale external consumer에 맞추기 위한 Core shim, compatibility fallback 또는 contract 약화를 만들지 않는다.
5. scope 정정 전 external test observation은 acceptance, blocker, baseline과 completion evidence에서 제외한다.
6. existing dirty Core work는 사용자 작업으로 보존하고 task-owned hunk와 분리한다.

## Current Technical Judgment

기존 `001` chain은 Unity work package, server/Unity atomic protocol Phase와 final cross-runtime suite를 mandatory로
만들었다. 사용자의 core-only decision은 단순 validation omission이 아니라 accepted policy scope, Phase graph와
Goal mapping을 변경한다. 따라서 GEX 문서가 Unity row만 삭제한 뒤 기존 Goal을 완료 처리할 수 없다.

가장 이른 올바른 owner는 `P2G-10`이다. 그 successor가 core-only policy acceptance를 확정한 뒤 P2G-20의 Core
source review, P2G-30 Freeze, P2G-40 Phase Plan, P2G-50 Goal Set과 P2G-60 Handoff를 각 단계별로 갱신해야 한다.
기존 `001` artifact는 역사 기록으로 보존한다.

## Preserved Core-Only Candidate Evidence

- 57개 Rulebook heading stable feature manifest
- Core-only locator verifier; external Unity/game_server/absolute locator rejection
- Little Red/projectile/effect event-heavy repeatable baseline
- 64-session/8-way interleaved repeatable baseline
- current `game_core` non-zero 1,131 tests passed, failed/ignored 0
- Core production behavior를 바꾸지 않는 Rulebook locator와 support-only harness

이 evidence는 successor basis가 채택하고 currentness를 다시 검증하기 전까지 completion evidence가 아니라
candidate input이다.

## Remaining Core Evidence Gaps

- successor core-only acceptance와 Goal mapping
- manifest/verifier를 scope 정정 후 다시 실행한 current receipt
- successor P01 Audit Target Record와 fresh GEX-20
- P02 이후 Core-only remediation Phase/Goal 구조

Unity execution evidence 부재나 external compatibility는 gap이 아니다.

## Deferred Work

EGO corrosion, normal boss egg, skill-fragment handoff/content와 run relic 구현은 계속 제외한다. 배포, release,
production promotion과 external client/server 재작성도 이번 core-only architecture remediation 밖이다.
