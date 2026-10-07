# Migration Ledger — Transaction Kernel Foundation

- 상태: `prepared`
- 기준일: 2026-08-22
- 목적: 각 authority writer의 이동을 추적하고 dual-write와 부분 migration을 방지

## 1. 상태 정의

- `planned`: 아직 production 변경 없음
- `oracle-ready`: before/after behavior와 defect가 테스트로 고정됨
- `in-progress`: 새 path 구현 또는 전환 중
- `migrated`: 새 path가 sole writer이며 narrow gate 통과
- `verified`: broad acceptance gate까지 통과
- `deferred-approved`: 사용자 승인과 이유가 있는 연기
- `blocked`: stop condition 증거가 있는 중단

## 2. Writer policy

1. 각 command/authority root 조합에는 writer가 하나만 존재해야 한다.
2. 전환 중에도 runtime dual-write를 만들지 않는다.
3. old writer를 남겨야 한다면 production call graph에서 도달 불가능하고 removal evidence가 있어야 한다.
4. fallback은 read compatibility에만 제한한다. mutation fallback은 금지한다.
5. migration row가 `migrated`가 되기 전에 old path call-site search와 rejection equality test를 완료한다.

## 3. Planned migration units

| ID | Unit | Old authority/write path | Target path | Required oracle | Commit/checkpoint unit | Status |
|---|---|---|---|---|---|---|
| `M-000` | command inventory | `ActionDescriptor` route/scope only; transaction semantics not enforced | exhaustive transaction metadata + authority inventory | exact `ActionKind::ALL` set | oracle/inventory | planned |
| `M-010` | root transaction selection | handler convention and route-specific direct calls | execute root selects required transaction runner/port | descriptor/route behavior | kernel foundation | planned |
| `M-020` | PauseBattle | direct live active-session mutation before fallible update DTO | staged battle mutation + staged projection + one publish | pause success + future-cursor failure | battle migration | planned |
| `M-021` | ResumeBattle | direct live active-session mutation before fallible update DTO | staged battle mutation + staged projection + one publish | resume success + future-cursor failure | battle migration | planned |
| `M-022` | SetBattleSpeed | direct live playback mutation before fallible update DTO | staged battle mutation + staged projection + one publish | speed success + future-cursor failure | battle migration | planned |
| `M-023` | BeginSkillTargeting | direct live focus mutation before fallible update DTO | staged focus mutation + staged projection + one publish | focus success + future-cursor failure | battle migration | planned |
| `M-024` | CancelSkillTargeting | direct live focus mutation before fallible update DTO | staged focus mutation + staged projection + one publish | cancel success + future-cursor failure | battle migration | planned |
| `M-025` | ActivateSkill | direct live skill/event mutation before fallible update DTO | staged activation + event/projection + one publish | deterministic activation success + future-cursor failure | battle migration | planned |
| `M-030` | StartNewGame | multiple live roots mutated before fallible RunStart trigger/projection | staged full-session construction + trigger/preflight + one publish | new-game success + trigger failure equality/retry | StartNewGame migration | planned |
| `M-040` | conformance | command-specific tests without universal atomicity matrix | table-driven rejection/success/retry coverage | G1–G6 evidence | conformance/final | planned |
| `M-050` | recovery/documentation | audit findings and code state are separate | completion audit + exact ledger + atomic checkpoints | G7–G9 evidence | conformance/final | planned |

## 4. Positive-control rows

이 row들은 현재 Goal에서 재작성하지 않는다. 새 kernel이 기존 올바른 경로를 깨지 않는지 확인한다.

| Control | Current pattern | Required result | Status |
|---|---|---|---|
| deploy/withdraw/stance/relocate/swap | staged tactical mutation | semantics와 atomic rejection 유지 | planned check |
| reward/shop mutations | plan/preflight/commit 계열 | root classification 연결 후 회귀 없음 | planned check |
| battle queries | read-only | authority 변화 없음 | planned check |
| host recovery | host-owned continuation | gameplay transaction으로 흡수되지 않음 | planned check |

## 5. Deferred findings

현재 Goal에서 구현하지 않지만 누락으로 간주하지 않는다.

| Deferred unit | Source findings | Future Goal candidate | Reason |
|---|---|---|---|
| battle authority/history | Structural 003/004/009, Architecture 002/003 | active battle session and history boundary | transaction foundation 이후 sole authority port를 사용해야 함 |
| validated data | Structural 007, Architecture 006 | validated game data boundary | 별도 construction/API compatibility 설계 필요 |
| protocol boundary | Structural 008 | protocol DTO/error boundary | server/Unity adapter와 version policy 필요 |
| typed runtime state | Structural 001, Architecture 001 | typed state transition boundary | broad change amplification을 별도 oracle로 다뤄야 함 |

기각된 shop overflow finding은 deferred row가 아니다.

## 6. Atomic recovery checkpoints

| Checkpoint | Contents | Required gate before advancing | Recovery meaning |
|---|---|---|---|
| C1 | oracle tests + exact inventory | Phase 1 exit | production semantics 변경 전 기준점 |
| C2 | descriptor + root transaction selection | G1/G2 narrow | handler migration 전 enforcement 기준점 |
| C3 | six battle migrations | G3/G5/G6 narrow | battle family 전체가 sole writer인 기준점 |
| C4 | StartNewGame migration | G4/G5/G6 narrow | full-session transition 기준점 |
| C5 | broad conformance + docs | G0–G9 | Goal completion candidate |

사용자가 commit을 승인한 경우 checkpoint별 staged diff를 검토하고 commit한다. push 승인까지 포함된 경우에만 C5와 전체 history를 확인한 뒤 push한다.

## 7. Activity log

실행 중 각 상태 전환을 append한다.

| Time | Unit | From | To | Evidence / note |
|---|---|---|---|---|
| 2026-08-22 | Goal documents | — | prepared | 두 감사 통합 청사진과 실행 문서 작성; production code 미변경 |

## 8. Completion invariant

현재 Goal 완료 시 `M-000`, `M-010`, `M-020`–`M-025`, `M-030`, `M-040`, `M-050`은 모두 `verified`여야 한다. 하나라도 planned/in-progress/migrated에 머물면 완료가 아니다.

