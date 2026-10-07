# Finding Crosswalk — Structural Integrity × Architecture Deep Review

- 상태: `complete / adjudicated`
- 목적: 두 source audit의 exact-set 보존과 canonical finding 관계 판정
- 주의: 아래 counterpart는 초기 routing hypothesis이며 통합 결론이 아니다.

## 1. Relationship vocabulary

| Relationship | 의미 |
|---|---|
| `same defect` | trigger, authority, failure mode와 impact가 실질적으로 동일함 |
| `corroborates` | 다른 방법 또는 축에서 같은 문제의 존재를 강화함 |
| `cause/effect` | 한 finding이 다른 finding의 구조적 원인 또는 구체적 증상임 |
| `same root cause` | 공통 원인이 있으나 독립된 trigger/impact를 유지함 |
| `independent` | 같은 subsystem일 수 있으나 별도 문제임 |
| `conflicts` | 사실, severity, disposition 또는 rewrite 판단이 양립하지 않음 |
| `no counterpart` | 다른 감사가 직접 다루지 않았음 |

## 2. Structural audit source catalog

준비 시점 catalog: Active 8, Rejected 1, Candidate 0, Superseded 0.

| Source ID | Status | Severity / confidence | Source claim | Initial counterpart hypothesis | Adjudicated relation | Canonical ID |
|---|---|---|---|---|---|---|
| `RUNTIME-AUDIT-001` | Active | S3 Low / Strong inference | test-only currency setter가 production public authority를 우회 | ARCH-REVIEW-001 또는 006의 boundary symptom 가능 | `cause/effect` with ARCH-REVIEW-001; exact architecture counterpart 없음 | `CA-010` |
| `RUNTIME-AUDIT-003` | Active | S2 Medium / Strong inference | BattleCore/RuntimeUnit authority와 event injection이 public mutable | ARCH-REVIEW-003 | `same defect` for the public-authority facet; architecture finding is broader | `CA-005` |
| `RUNTIME-AUDIT-004` | Active | S2 Medium / Strong inference | non-finite BattleEvent가 manual Eq/Ord premise를 위반 가능 | ARCH-REVIEW-003의 porous kernel boundary와 관련 | `cause/effect`: concrete invalid-event contract enabled by the broader porous boundary | `CA-006` |
| `RUNTIME-AUDIT-005` | Active | S1 High / Strong inference | 여섯 battle command가 future-cursor 실패 전 authority commit | ARCH-REVIEW-004, 005 | `same defect` with ARCH-REVIEW-004; `cause/effect` with ARCH-REVIEW-005 | `CA-003` |
| `RUNTIME-AUDIT-006` | Active | S2 Medium / Strong inference | StartNewGame이 fallible RunStart trigger 전에 root mutation | ARCH-REVIEW-005 | `cause/effect` with ARCH-REVIEW-005; ARCH-REVIEW-006 is a reachability contributor, not the same defect | `CA-004` |
| `RUNTIME-AUDIT-007` | Active | S2 Medium / Confirmed | public alternate GameData construction이 invalid input에서 panic | ARCH-REVIEW-006 | `same defect`: concrete panic path and erased validation type-state | `CA-007` |
| `RUNTIME-AUDIT-008` | Active | S2 Medium / Strong inference | result/error wire schema가 domain representation과 직접 결합 | ARCH-REVIEW-001/003/005의 protocol pressure와 비교 필요 | `corroborates` protocol/dependency pressure; exact standalone architecture finding 없음 | `CA-008` |
| `RUNTIME-AUDIT-009` | Active | S2 Medium / Confirmed | recent battle delta가 unbounded history를 매번 전체 순회 | ARCH-REVIEW-002 | `same defect` for retained-history scan; `same root cause` for full-session clone scaling | `CA-009` |
| `RUNTIME-AUDIT-002` | Rejected | rejected | shop sale addition은 선행 checked evaluation으로 overflow 불가 | 직접 counterpart 없음 | `no counterpart`; rejected history only | `REJECTED-001` |

## 3. Architecture review source catalog

준비 시점 catalog: Active 6, Rejected 0, Candidate/Hypothesis 0, Superseded 0.

| Source ID | Status | Severity / confidence | Source claim | Initial counterpart hypothesis | Adjudicated relation | Canonical ID |
|---|---|---|---|---|---|---|
| `ARCH-REVIEW-001` | Active | S1 High / Confirmed | broad application root가 change amplification 중심 | Structural RC-01/02/03과 여러 symptoms의 상위 pressure 가능 | `no counterpart` at finding level; `cause/effect` parent of several structural symptoms | `CA-001` |
| `ARCH-REVIEW-002` | Active | S2 Medium / Confirmed | full battle clone과 retained history로 correctness cost가 확장 | RUNTIME-AUDIT-009; 005와 transaction mechanism 관계 | `same defect` with RUNTIME-AUDIT-009 for history; independent full-clone facet retained | `CA-009` |
| `ARCH-REVIEW-003` | Active | S2 Medium / Confirmed | battle kernel의 data/application/projection dependency가 porous | RUNTIME-AUDIT-003/004/007/008 일부 | `same defect` with 003's authority facet; `cause/effect` for 004 and contributes to 008 | `CA-005`; contributes `CA-006`, `CA-008` |
| `ARCH-REVIEW-004` | Active | S2 Medium / Confirmed | mutating battle use case에 uniform transaction boundary 없음 | RUNTIME-AUDIT-005 | `same defect` with 005 and specialization of ARCH-REVIEW-005 | `CA-002`, `CA-003` |
| `ARCH-REVIEW-005` | Active | S1 High / Confirmed | application transaction atomicity가 command invariant가 아닌 handler convention | RUNTIME-AUDIT-005/006; Structural RC-02 | `cause/effect` parent of 005/006; broader than battle-only 004 | `CA-002`; contributes `CA-003`, `CA-004` |
| `ARCH-REVIEW-006` | Active | S2 Medium / Confirmed | validated live data와 fixture data가 같은 runtime type으로 erase | RUNTIME-AUDIT-007; Structural RC-01 | `same defect` with 007; reachability contributor to 006 | `CA-007`; contributes `CA-004` |

## 4. Root-cause crosswalk

아래는 source root cause의 초기 routing table이다. 실행 중 관계와 canonical root cause를 확정한다.

| Structural root | Architecture root candidate | Initial shared pressure | Final relation | Canonical root |
|---|---|---|---|---|
| `RC-01` fixture/debug와 production type/visibility 미분리 | `RC-B` compiler-enforced data/domain/protocol 경계 부족 | invalid/public construction과 authority surface | `overlap`, but structural RC-01 also contributes one transaction reachability case | `CR-03`; contributes `CR-02` |
| `RC-02` transaction/preflight/projection ordering이 handler별 convention | `RC-A` orchestration/state/transaction/projection broad root | command atomicity와 root enforcement | `same root cause` for the transaction/publish portion | `CR-02` |
| `RC-03` transport projection/history가 domain representation을 다목적으로 재사용 | `RC-B`, `RC-C` | protocol coupling과 retained history scaling | one structural root splits into compiler/protocol and retained-state responsibilities | `CR-03`, `CR-04` |
| no single structural finding-level root | `RC-A` broad aggregate/state portion | application root and parallel lifecycle authority | architecture-only root pressure, structural design draft corroborates | `CR-01` |
| no single structural root | `RC-C` complete battle state/history reuse | staging/reconnect cost | architecture finding broadens structural history-scan evidence with full-clone cost | `CR-04` |

architecture source roots:

- `RC-A`: application orchestration, phase/session state, transaction selection, projection
- `RC-B`: validated authoring, domain kernel, external protocol의 compiler boundary
- `RC-C`: complete retained battle state/history를 correctness staging과 reconnect에 재사용

## 5. Gap crosswalk

Gap은 finding과 분리한다.

| Source gap | Meaning | Related finding candidates | Canonical verification debt |
|---|---|---|---|
| Structural `GAP-001` | 여섯 live command future-cursor before/after equality 부재 | RUNTIME-AUDIT-005, ARCH-REVIEW-004/005 | `VD-01` failed-command authority equality/retry matrix |
| Structural `GAP-002` | StartNewGame missing-trigger full equality 부재 | RUNTIME-AUDIT-006, ARCH-REVIEW-005 | `VD-02` full-session rejection equality/retry |
| Structural `GAP-003` | broad continuation equivalence 부재 | persistence/operations boundary | `VD-03` canonical multi-domain continuation command tape |
| Structural `GAP-004` | permanent event-history scale budget guard 부재 | RUNTIME-AUDIT-009, ARCH-REVIEW-002 | `VD-04` retained-history work/byte/retention guard |
| Architecture `GAP-001` | exact retention/tick/concurrency cap 미정 | ARCH-REVIEW-002 | `VD-04` product policy values required before release certification |
| Architecture `GAP-002` | semantic vs bitwise cross-platform physics replay 미정 | determinism boundary | `VD-05` cross-platform replay guarantee policy |

같은 번호는 같은 gap을 뜻하지 않는다. canonical 문서에서는 source prefix를 유지한다.

## 6. Canonical finding schema

각 canonical finding은 다음 필드를 가져야 한다.

- canonical ID와 제목
- linked source finding IDs
- relation/merge rationale
- status
- normalized severity
- confidence와 evidence class
- affected subsystem/authority
- trigger와 failure/change scenario
- impact
- root cause
- counter-evidence/limits
- preservation constraints
- disposition
- remediation dependency group 또는 explicit defer
- verification debt

## 7. Exact-set completion checks

- [x] Structural Active 8개가 모두 canonical row에 연결됨
- [x] Structural Rejected 1개가 historical row로 유지됨
- [x] Architecture Active 6개가 모두 canonical row에 연결됨
- [x] source count와 실제 final catalog count 재확인
- [x] `pending` relation 없음
- [x] `pending` canonical ID 없음
- [x] conflict/no-counterpart가 숨겨지지 않음
- [x] source root cause 6개(RC-01~03, RC-A~C)가 모두 canonical model에 연결됨
- [x] source gap 6개가 finding과 분리되어 추적됨

## 8. Adjudication summary

- factual conflict: none
- canonical active findings: 10
- canonical rejected history: 1
- canonical root causes: 4
- canonical verification debts: 5
- source architecture findings are often causal parents or broader pressures; they were not deleted merely because a concrete structural symptom exists.
- source severity/confidence is preserved in the tables; normalized canonical values and any confidence adjustment are explained in `CONSOLIDATED_AUDIT.md`.
