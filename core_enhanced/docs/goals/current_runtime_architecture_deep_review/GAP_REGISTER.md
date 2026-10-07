# Architecture Gap Register

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Baseline: `BASELINE-20260822-064427-KST`

## 기록 규칙

Evidence 수집 후에도 판정할 수 없는 source, scenario, workload, consumer intent 또는 behavior oracle만 gap으로
등록한다. 실행 전 계획과 아직 읽지 않은 범위를 gap finding처럼 부풀리지 않는다.

각 gap은 다음을 포함한다.

```markdown
## GAP-###. <제목>

- 최초 기록 시각:
- criterion:
- 현재 evidence:
- missing evidence/source/policy:
- 결론에 미치는 영향:
- UNKNOWN으로 격리 가능한지:
- 후속 조사 방법:
- 사용자 입력 필요 여부:
```

## Open Gaps

## GAP-001. Product long-session and host workload caps are undecided

- 최초 기록 시각: `2026-08-22T07:31:00+09:00`
- criterion: A07, A09
- 현재 evidence: E-020 characterizes common operations and history/entity scaling; current event archive has no explicit cap.
- missing evidence/source/policy: target server tick deadline, concurrent active sessions, maximum battle time/event/byte
  retention and reconnect window; allocation/peak RSS and event-heavy boss workload.
- 결론에 미치는 영향: exact production SLA, archive retention number and capacity plan remain provisional. It does not change
  the evidence that delta lookup should be O(returned range) and transaction staging should not copy unrelated retained history.
- UNKNOWN으로 격리 가능한지: yes; A09 can judge current unbounded complexity while exact cap remains policy-undecided.
- 후속 조사 방법: after architecture approval, choose server workload policy and add pinned allocation/latency benchmarks with
  a representative event-heavy boss/projectile/effect fixture.
- 사용자 입력 필요 여부: not for this rewrite decision; required before release-cap policy becomes canonical.

## GAP-002. Cross-platform physics equivalence guarantee is undecided

- 최초 기록 시각: `2026-08-22T07:31:00+09:00`
- criterion: A10
- 현재 evidence: deterministic seeds, stable ordering, focused movement tests and E-021 same-runner continuation equivalence.
- missing evidence/source/policy: whether semantic server-authoritative equivalence is sufficient or bitwise Rapier/float
  replay across OS/CPU/toolchain is required; no cross-platform replay was run.
- 결론에 미치는 영향: determines the eventual replay certification level, not whether current domain algorithms or
  checkpoint authority should be preserved.
- UNKNOWN으로 격리 가능한지: yes.
- 후속 조사 방법: define the guarantee, record a canonical battle command tape and compare semantic event/final-state hashes
  across supported targets.
- 사용자 입력 필요 여부: not for this architecture choice; required before cross-platform certification scope is fixed.

## Closed Gaps

아직 없음.
