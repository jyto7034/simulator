# Consolidated Runtime Audit

- 상태: `complete / canonical`
- 대상: `core_enhanced`
- 통합일: 2026-08-22
- source audits:
  - `current_runtime_structural_integrity_audit`
  - `current_runtime_architecture_deep_review`
- canonical finding catalog: Active 10, Rejected history 1
- runtime architecture verdict: `FAIL`
- rewrite decision: whole-core big-bang rewrite 기각, staged partial boundary rewrite 권고

## 1. Executive conclusion

현재 `core_enhanced`의 구조를 그대로 유지하면서 개별 결함만 patch하는 것은 최종 해법으로 불충분하다. 반면 전체 Core를 한 번에 다시 작성할 근거도 없다.

두 감사가 함께 지지하는 결론은 다음과 같다.

1. 결함과 변경 압력은 application/state/transaction, protocol, active-battle session/history, validated-data와 public authority 경계에 집중되어 있다.
2. movement, skill execution, Boss Runtime DSL, battle algorithms, map/facility, Monolith, reward planners, live-data validation, checkpoint/determinism과 1,131개 테스트는 보존 가치가 높다.
3. 따라서 기존 façade와 관찰 동작을 oracle로 유지하면서 위 경계를 single-authority vertical slice로 교체하는 `staged partial rewrite`가 적절하다.
4. atomic transaction trait 하나만 추가하는 것으로는 root bypass, broad state, data/protocol dependency, history scaling을 해결할 수 없다. 반대로 이 결론은 특정 trait/API 설계를 승인하지도 않는다.
5. 두 감사 사이에 factual conflict는 없다. Structural audit는 concrete failure와 public-surface defect를, Architecture review는 그 defect를 낳는 상위 change/ownership pressure와 보존 가능한 seam을 주로 증명했다.

현재 architecture는 여섯 축 중 의존 방향, 변경 용이성, 운영 경계가 `FAIL`이며 도메인 구조, 확장성, 테스트 가능성은 `PARTIAL`이다. 도메인 엔진의 품질과 application shell의 품질을 하나의 평균 점수로 상쇄해서는 안 된다.

## 2. Source scope and trust boundary

### 2.1 Shared baseline

두 감사는 모두 repository HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`에서 완료되었다. 양쪽 모두 동일한 broad baseline을 기록했다.

- `cargo test --all-targets --all-features`: 1,131 passed, 0 failed
- `cargo fmt --all -- --check`: PASS
- strict clippy: 기존 baseline에서 FAIL

Clippy 실패는 현재 architecture finding count에 섞지 않는다. 두 감사 모두 production bytes를 수정하지 않은 read-only 감사였다.

### 2.2 Different questions, compatible evidence

| Audit | Primary question | Strongest contribution | Important limit |
|---|---|---|---|
| Structural integrity | authority, transaction, determinism, data/publication, persistence/protocol 경계에 concrete defect가 있는가 | 8 Active concrete findings, 1 Rejected history, RC-01~03, GAP-001~004 | gameplay policy/content quality와 Unity 제외; server는 필요한 adjacent seam만 확인 |
| Architecture deep review | domain/dependency/change/test/operation 구조가 future scenarios를 지속 가능하게 수용하는가 | 6 Active architecture findings, RC-A~C, PA-001~012, migration seams, five-option rewrite decision | Core-only; server/Unity 인증 제외; exact product cap과 cross-platform physics guarantee 미정 |

Structural audit는 210-file content scope, Architecture review는 184 runtime/test manifest와 routed adjacent sources를 사용했다. 이 manifest 차이는 질문 범위 차이며 baseline conflict가 아니다.

Architecture review는 independent verdict lock(E-022) 뒤에 Structural audit를 E-023으로 cross-check했다. 따라서 두 결과의 일치는 한 감사를 단순 복사한 결과가 아니다.

### 2.3 Excluded authority

- `current_runtime_transaction_kernel_foundation`은 통합 전에 작성된 premature remediation draft이며 이 결론의 evidence가 아니다.
- stale runtime/Unity contract는 current policy authority로 사용하지 않았다.
- server/Unity 전체 구현, balance/content quality, 새 feature 상세 정책은 이 통합이 인증하지 않는다.
- source audits 사이에 해결을 위해 production 재감사가 필요한 factual conflict는 발견되지 않았다.

## 3. Canonical finding catalog

### 3.1 Summary

| ID | Severity | Confidence | Title | Primary sources | Root | Disposition |
|---|---|---|---|---|---|---|
| `CA-001` | S1 High | Confirmed | Broad application root and parallel lifecycle state amplify change | ARCH-REVIEW-001 | CR-01 | `REWRITE_BOUNDARY` |
| `CA-002` | S1 High | Confirmed | Transaction/publish policy is a handler convention | ARCH-REVIEW-004/005, Structural RC-02 | CR-02 | `REWRITE_BOUNDARY` |
| `CA-003` | S1 High | Confirmed | Six live battle commands can report failure after authority commit | RUNTIME-AUDIT-005, ARCH-REVIEW-004/005 | CR-02 | immediate correctness remediation inside boundary rewrite |
| `CA-004` | S2 Medium | Strong inference | StartNewGame can leave partial session state on trigger failure | RUNTIME-AUDIT-006, ARCH-REVIEW-005/006 | CR-02, CR-03 | transaction + validated-data boundary |
| `CA-005` | S2 Medium | Confirmed | Battle kernel authority/dependency boundary is porous | RUNTIME-AUDIT-003, ARCH-REVIEW-003 | CR-03 | `REFACTOR` / `REWRITE_BOUNDARY` |
| `CA-006` | S2 Medium | Strong inference | Public non-finite battle events violate Eq/Ord premise | RUNTIME-AUDIT-004, ARCH-REVIEW-003 | CR-03 | validated input/encapsulation remediation |
| `CA-007` | S2 Medium | Confirmed | Validated live data and fixture data erase into one panic-capable runtime type | RUNTIME-AUDIT-007, ARCH-REVIEW-006 | CR-03 | `REWRITE_BOUNDARY` |
| `CA-008` | S2 Medium | Strong inference | Protocol result/error schema is coupled to domain/Rust representation | RUNTIME-AUDIT-008; ARCH-REVIEW-001/003/005 corroboration | CR-03 | `REWRITE_BOUNDARY` |
| `CA-009` | S2 Medium | Confirmed | Battle correctness/reconnect cost scales with complete retained state/history | RUNTIME-AUDIT-009, ARCH-REVIEW-002 | CR-04 | `REWRITE_SUBSYSTEM` boundary internals |
| `CA-010` | S3 Low | Strong inference | Test-only currency setter is production-public | RUNTIME-AUDIT-001; ARCH-REVIEW-001 cause | CR-03 | encapsulation remediation |

Severity distribution은 S1 3개, S2 6개, S3 1개다. Source architecture pressure와 concrete defect를 둘 다 유지했으므로, source finding을 단순 합산하거나 중복 제거한 숫자와 같을 필요는 없다.

### 3.2 Detailed findings

#### CA-001 — Broad application root and parallel lifecycle state amplify change

- Trigger/pressure: 새 node/session/Relic/roguelike capability 또는 cross-cutting rule을 추가할 때 `GameCore`, `GameCoreState`, routing, capability, snapshot/checkpoint와 병렬 optional state를 함께 수정해야 한다.
- Impact: invalid phase/authority 조합을 type이 배제하지 못하고, 1인+AI 개발에서도 넓은 context와 암묵적 transition convention을 반복해서 기억해야 한다.
- Counter-evidence: `GameCore` state는 private이고 action routing은 exhaustive하다. domain engines와 일부 staged transition은 응집되어 있다.
- Canonical judgment: façade는 보존하되 broad application aggregate와 lifecycle/use-case boundary는 vertical slice로 재작성한다. 전체 domain rewrite의 근거는 아니다.
- Remediation group: `RU-05`.

#### CA-002 — Transaction/publish policy is a handler convention

- Trigger/pressure: 각 mutating handler가 whole-root transaction, tactical clone, economy stage, manual preflight 또는 direct mutation 중 하나를 스스로 선택한다.
- Impact: 새 command마다 fallible validation/projection/durable intent의 순서를 수동 증명해야 하며 bypass가 type/root에서 금지되지 않는다.
- Counter-evidence: `with_state_transaction`, staged battle advance/tactical mutation, reward/economy plans는 올바른 pattern이 이미 실현 가능함을 증명한다.
- Canonical judgment: 기존 pattern의 존재가 architecture PASS를 뜻하지 않는다. command/use-case root가 transaction and publish obligation을 강제하는 새 application boundary가 필요하다. 구체 trait나 state clone 전략은 미결정이다.
- Remediation group: `RU-01`.

#### CA-003 — Six live battle commands can report failure after authority commit

- Affected family: pause, resume, speed, begin/cancel skill targeting, activate skill.
- Trigger: valid domain command가 live session/revision을 변경한 뒤 future battle cursor가 response projection에서 거부됨.
- Impact: caller는 `Err`를 받지만 playback/focus/cast/event/cooldown/resource/revision authority가 달라질 수 있어 retry와 reconnect 의미가 깨진다.
- Counter-evidence: normal adjacent actor는 내부 cursor를 사용해 ordinary incidence를 낮춘다. 그러나 public transport-context API는 future cursor error를 typed recoverable error로 모델링하며 tactical commands는 동일 실패를 staged rollback으로 처리한다.
- Confidence normalization: Structural source는 static proof 때문에 Strong inference였지만 Architecture review가 동일 six-handler path를 current baseline에서 Confirmed finding으로 독립 분류했다. canonical confidence는 `Confirmed`; frequency는 측정되지 않았다.
- Canonical judgment: 현재 catalog에서 가장 직접적인 S1 correctness defect다.
- Remediation group: `RU-01`; verification debt `VD-01`.

#### CA-004 — StartNewGame can leave partial session state on trigger failure

- Trigger: public builder/new로 구성된 data에 RunStart trigger가 없을 때 root initialization 뒤 trigger evaluation이 실패함.
- Impact: player, pending mode, cleared run/session roots 등 일부 새 게임 state가 error 뒤 남아 retry 의미가 불분명해진다.
- Counter-evidence: official embedded loader는 current live bundle을 검증하므로 live-server reachability가 낮다. 그러나 public runtime type/constructor가 exclusive validated path를 강제하지 않는다.
- Canonical judgment: transaction ordering과 validated-data erasure가 함께 만든 defect다. transaction fix나 constructor restriction 중 하나만으로 다른 invariant가 자동 해결되지는 않는다.
- Remediation groups: `RU-01`, `RU-02`; verification debt `VD-02`.

#### CA-005 — Battle kernel authority/dependency boundary is porous

- Trigger/pressure: exported BattleCore/RuntimeUnit mutable fields와 event injection, full database, authored/application/projection/error types가 kernel 경계로 들어옴.
- Impact: command/event lifecycle을 우회한 mutation이 type-check하고 kernel test/compile context와 future change fan-out이 커진다.
- Counter-evidence: active GameCore battle은 private이고 current world production callers가 observed public fields를 직접 mutate하지는 않는다. typed command/scenario/setup seams와 deterministic algorithms는 강하다.
- Confidence normalization: public surface와 dependency imports는 Architecture review에서 Confirmed되었으므로 combined boundary finding은 Confirmed다. 현재 live corruption incidence는 주장하지 않는다.
- Canonical judgment: battle algorithms는 보존하고 authority/input/query/dependency boundary를 좁힌다. crate split은 logical direction이 안정된 뒤 선택 사항이다.
- Remediation group: `RU-03`.

#### CA-006 — Public non-finite battle events violate Eq/Ord premise

- Trigger: public float-bearing event/value와 event injection을 통해 NaN/non-finite payload가 `BinaryHeap<BattleEvent>`에 도달할 수 있음.
- Impact: manual `Eq` premise와 comparator consistency가 깨져 exported kernel boundary의 ordering/determinism contract가 성립하지 않는다.
- Counter-evidence: official live data와 internal conversions는 finite이고 720 permutation/order tests가 통과했다. 현재 live nondeterminism이 관찰됐다는 finding이 아니다.
- Canonical judgment: finite validated value/input boundary 또는 total-order policy가 compiler/validation으로 강제되어야 한다. CA-005와 같은 root지만 distinct language-contract failure이므로 병합하지 않는다.
- Remediation group: `RU-03`.

#### CA-007 — Validated live data and fixture data erase into one runtime type

- Trigger: strict official loader와 public panic/expect builder가 같은 bare `GameDataBase`를 만들고 `GameCore::new`가 provenance/type-state를 구분하지 않음.
- Impact: recoverable authoring error가 panic이 되거나, official loader가 만들지 않을 invalid/minimal state가 later command failure로 이동한다.
- Counter-evidence: adjacent server의 inspected startup은 official typed loader를 사용하며 fixture builder는 isolated tests에 유용하다.
- Canonical judgment: live validation/compiler pipeline과 fixture utility는 보존한다. production-validated publication type과 명시적 fixture construction boundary를 분리한다.
- Remediation group: `RU-02`.

#### CA-008 — Protocol result/error schema is coupled to domain/Rust representation

- Trigger/pressure: purpose DTO와 internal/domain structs가 `behavior.rs`의 direct serde result/error representation과 unversioned envelope에 섞임.
- Impact: internal field/variant 변경이 server/client schema와 central projection test fan-out을 직접 바꿀 수 있다.
- Counter-evidence: outer adapter와 stable `GameError::code()`가 있고 47 result projection types가 exhaustive하게 characterization되어 있다. whole enum이 아무 adapter 없이 그대로 wire라는 더 강한 가설은 반증되었다.
- Canonical judgment: existing external semantics는 보존하되 versioned adapter/mapper boundary로 ownership을 분리한다. 현재 Unity compatibility는 이 감사가 인증하지 않는다.
- Remediation group: `RU-06`.

#### CA-009 — Battle correctness/reconnect cost scales with complete retained state/history

- Mechanisms retained separately:
  - append-only event history의 recent-tail lookup이 전체 Vec를 scan함;
  - rollback correctness가 complete ActiveBattleSession/BattleCore clone에 의존함.
- Impact: long battle에서 tick/reconnect work, memory, clone cost와 result serialization이 unrelated retained state/history에 따라 증가한다.
- Evidence: constant two-entry tail이 1K/10K/100K history에서 약 4.99/47.87/489.66µs로 증가했고, Architecture E-020은 full-clone/entity scaling도 별도로 Confirmed했다.
- Counter-evidence: ordinary live fixture는 가볍고 concurrent/event-heavy boss workload와 product caps는 미측정이다. 절대 release failure를 주장하지 않고 unbounded resource shape를 finding으로 유지한다.
- Canonical judgment: battle semantics/event ordering은 보존하고 transaction staging internals, indexed reconnect buffer, archive/result/debug ownership을 분리한다.
- Remediation group: `RU-04`; verification debt `VD-04`.

#### CA-010 — Test-only currency setter is production-public

- Trigger: external Rust caller가 typed command/admin boundary 없이 authority currency를 overwrite 가능함.
- Impact: compiler-enforced authority encapsulation leak. 현재 repository production caller는 없음.
- Counter-evidence: current server의 admin mutation은 별도 typed path를 사용하고 observed callers는 tests뿐이다.
- Canonical judgment: 낮은 severity의 독립 surface defect로 유지한다. CA-001/CR-03의 한 증상이지만 다른 finding을 대표하지 않는다.
- Remediation group: `RU-02`.

### 3.3 Rejected history

`REJECTED-001`은 source `RUNTIME-AUDIT-002`를 보존한다. Shop sale commit의 unchecked-looking addition은 같은 immutable amount/quote를 바로 앞에서 `checked_add`하고 그 사이 authority mutation이 없어 overflow path가 거부된다. 표현 중복은 drift signal이지만 현재 defect가 아니며 remediation backlog에 넣지 않는다.

## 4. Finding relationship summary

### 4.1 Source-to-canonical mapping

| Source finding | Canonical destination | Treatment |
|---|---|---|
| RUNTIME-AUDIT-001 | CA-010 | 독립 low-severity encapsulation symptom |
| RUNTIME-AUDIT-002 | REJECTED-001 | 반증 이력만 보존 |
| RUNTIME-AUDIT-003 | CA-005 | ARCH-REVIEW-003의 public-authority facet과 통합 |
| RUNTIME-AUDIT-004 | CA-006 | porous boundary가 가능하게 한 distinct ordering-contract defect |
| RUNTIME-AUDIT-005 | CA-003 | ARCH-REVIEW-004와 same defect; ARCH-REVIEW-005의 concrete effect |
| RUNTIME-AUDIT-006 | CA-004 | transaction convention과 validated-data reachability의 concrete effect |
| RUNTIME-AUDIT-007 | CA-007 | ARCH-REVIEW-006과 same defect |
| RUNTIME-AUDIT-008 | CA-008 | standalone protocol finding; architecture dependency pressure가 corroborate |
| RUNTIME-AUDIT-009 | CA-009 | ARCH-REVIEW-002의 retained-history facet과 통합 |
| ARCH-REVIEW-001 | CA-001 | broad cause/pressure finding으로 독립 보존 |
| ARCH-REVIEW-002 | CA-009 | history scan과 full-clone facet을 한 resource-boundary finding에 보존 |
| ARCH-REVIEW-003 | CA-005; contributes CA-006/008 | public authority는 통합, input/protocol consequences는 distinct 유지 |
| ARCH-REVIEW-004 | CA-002/003 | battle specialization과 concrete partial commit 양쪽에 연결 |
| ARCH-REVIEW-005 | CA-002; contributes CA-003/004 | general convention finding과 concrete effects 분리 |
| ARCH-REVIEW-006 | CA-007; contributes CA-004 | data type-state defect와 StartNewGame reachability를 구분 |

### 4.2 Relationship totals

| Relationship | Result |
|---|---|
| exact/same-defect pairs | RUNTIME-AUDIT-003↔ARCH-REVIEW-003 public-authority facet; 005↔004; 007↔006; 009↔002 history facet |
| cause/effect | ARCH-001 → multiple application surface/change symptoms; ARCH-005 → 005/006; ARCH-003 → 004 and protocol pressure |
| same-root but distinct | 003 vs 004; 005 vs 006; 008 vs 009; history scan vs full clone |
| no exact counterpart | structural 001/008, architecture 001의 broad finding |
| factual conflicts | none |
| rejected import | RUNTIME-AUDIT-002 only, historical and non-remediation |

세부 양방향 mapping은 `FINDING_CROSSWALK.md`가 권위다.

## 5. Consolidated root-cause model

### CR-01 — Broad application aggregate and lifecycle orchestration

- Source roots: architecture `RC-A`의 aggregate/state portion; Structural design review의 parallel phase/authority observation.
- Findings: CA-001; CA-002의 context.
- Mechanism: façade, orchestration, aggregate storage, lifecycle phase, handler routing과 projection이 넓은 world type graph를 공유한다.
- Counter-evidence: 하나의 host-facing façade와 private root 자체는 유효하다. 문제는 façade 존재가 아니라 내부 use-case/aggregate boundary의 폭이다.

### CR-02 — Transaction, projection and publish are not a root command invariant

- Source roots: Structural `RC-02`, architecture `RC-A`의 transaction portion.
- Findings: CA-002, CA-003, CA-004; CA-008의 projection placement에 기여.
- Mechanism: handler가 staging/preflight/commit policy를 선택하며 fallible projection 또는 data-driven activation이 authority write 뒤에 남을 수 있다.
- Counter-evidence: 여러 safe staged paths가 존재하므로 machinery의 완전 부재가 아니라 compiler/root enforcement의 부재다.

### CR-03 — Compiler/visibility boundaries conflate fixture, validated data, domain authority and protocol

- Source roots: Structural `RC-01`, Structural `RC-03`의 protocol portion, architecture `RC-B`.
- Findings: CA-005, CA-006, CA-007, CA-008, CA-010; CA-004 reachability에 기여.
- Mechanism: intended caller/provenance/responsibility가 comments와 convention에 머물고 동일 public type graph로 erase된다.
- Counter-evidence: official loader, typed commands, purpose DTO, stable error code 같은 올바른 seams가 이미 존재한다.

### CR-04 — Complete retained battle state/history serves transaction and presentation responsibilities

- Source roots: Structural `RC-03`의 history portion, architecture `RC-C`.
- Findings: CA-009; CA-003 transaction mechanism에 기여.
- Mechanism: complete clone으로 rollback하고 append-only history 하나로 live delta, result archive, debug와 tests를 제공한다.
- Counter-evidence: correctness semantics와 event identity는 강하다. 교체 대상은 algorithms/event meaning이 아니라 storage/staging responsibility다.

## 6. Six-axis architecture assessment

| Axis | Verdict | Decisive evidence | Counter-evidence / confidence limit |
|---|---|---|---|
| 도메인 구조 | `PARTIAL` | broad root, parallel lifecycle authority와 hidden owner access(CA-001) | movement/skill/Boss/map/Monolith/reward engines는 응집되고 typed seams와 tests가 강함 |
| 의존 방향 | `FAIL` | battle kernel의 full data/application/projection imports, bare data type, domain/wire coupling(CA-005/007/008) | single crate direct types는 mapping overhead가 낮고 일부 compiled setup/purpose DTO seam 존재 |
| 확장성 | `PARTIAL` | 새 node/Relic/forms/corrosion/tool scenario가 application/protocol root fan-out을 반복(CA-001/008) | Boss DSL, movement pipeline, skill compiler/runtime, map registry 등 subsystem 확장은 재사용 가능 |
| 변경 용이성 | `FAIL` | handler-selected transaction, broad world/behavior ownership과 multi-representation synchronization(CA-001/002/008) | exhaustive routing과 private GameCore root는 omissions 일부를 compiler로 차단 |
| 테스트 가능성 | `PARTIAL` | fixture/runtime type erasure, broad kernel context, universal rejection/continuation/performance gate 부재(CA-005/007, VD-01~05) | 1,131 passing tests, live-data/checkpoint/determinism oracle와 staged rejection tests는 매우 강함 |
| 운영 경계 | `FAIL` | failed-command partial mutation, unbounded/scanned history와 whole-state staging(CA-003/004/009) | host I/O는 개념적으로 Core 밖이고 ordinary fixture는 가벼움; server-wide crash/delivery는 미감사 |

전체 verdict `FAIL`은 실패 축 세 개와 S1 correctness/change findings에 근거한다. 보존 자산이 많다는 사실은 failure를 취소하지 않고, failure가 있다는 사실도 domain engines 전체를 폐기할 근거가 아니다.

## 7. Preservation assets and subsystem disposition

### 7.1 Preservation assets

| Source asset | Preserve | Why |
|---|---|---|
| PA-001 | typed `GameCore` façade와 behavior/capability semantics | single host entry, exhaustive routing과 compatibility oracle |
| PA-002 | facility graph, generator, map progression | deterministic topology와 staged completion |
| PA-003~006 | battle algorithms, movement, skill runtime, Boss Runtime DSL | rich deterministic typed execution과 높은 rediscovery risk |
| PA-007~008 | Monolith/encounter ledger, reward/economy/inventory planners | explicit receipt/plan 및 no-partial-effect patterns |
| PA-009 | live-RON parse/validate/compile/single publication | fail-closed production data pipeline |
| PA-010~011 | V4 checkpoint, seeded ordering, UUID/event/receipt identity | migration/continuation/determinism oracle |
| PA-012 | 1,131-test/live-content corpus | slice-by-slice differential and release gate |

### 7.2 Subsystem disposition

| Subsystem/responsibility | Disposition | Boundary decision |
|---|---|---|
| GameCore external façade/behavior meaning | `RETAIN` | internal use-case/protocol adapter를 교체해도 host-facing semantics는 oracle로 유지 |
| application command transaction/publish | `REWRITE_BOUNDARY` | command-level authority/projection/publish invariant 필요 |
| root session/run/node lifecycle state | `REWRITE_BOUNDARY` | typed phase/active-session aggregate로 vertical migration |
| protocol result/error/snapshot | `REWRITE_BOUNDARY` | versioned mapper/adapter와 consumer cutover |
| active battle application session | `REWRITE_BOUNDARY` | single mutation port와 scoped staging |
| battle reconnect/archive storage | `REWRITE_SUBSYSTEM` | bounded indexed reconnect storage와 archive/result/debug ownership 분리 |
| battle simulation/execution algorithms | `REFACTOR` | private authority, scoped context/local models; algorithm 보존 |
| movement | `RETAIN` + small `REFACTOR` | policy seam만 명확화; subsystem rewrite 근거 없음 |
| skill/cast/delivery/interrupt | `RETAIN` + small `REFACTOR` | authored activation/form policy와 scoped catalog 강화 |
| Boss Runtime | `RETAIN` | cross-run encounter orchestration은 DSL 밖 새 owner |
| map/facility/Monolith/reward | `RETAIN` | application use-case port 뒤에 배치 |
| live data validation/compiler | `RETAIN` | opaque validated output과 consumer-scoped catalog 추가 |
| public fixture/data construction | `REWRITE_BOUNDARY` | production validated type과 fixture context 분리 |
| checkpoint/deterministic identity | `RETAIN` + representation `REFACTOR` | V4 importer/oracle와 target aggregate DTO |
| tests/oracles | `RETAIN` + extend | rejection/continuation/performance differential gates 추가 |
| host I/O/observation | `RETAIN OUTSIDE CORE` | explicit receipt/persistence/observation adapter contract; 현재 구현 미인증 |
| future Relic/corrosion/dice/tool | `DEFER_POLICY` as new bounded systems | 기존 Monolith/Boss state에 임의로 흡수하지 않음 |
| rejected shop-sale overflow candidate | `REJECTED_REMEDIATION` | 선행 checked evaluation 반증을 보존하고 새 증거 없이 backlog에 넣지 않음 |

## 8. Rewrite decision

### 8.1 Decision

```text
현재 구조 그대로 유지 + 국소 결함 patch만: 불충분
전체 core_enhanced big-bang rewrite: 기각
application/state/transaction/protocol staged boundary rewrite: 권고
active battle session/history와 validated-data construction의 bounded rewrite: 권고
domain/gameplay algorithms 재작성: 현재 근거 없음
```

### 8.2 Why structural findings do not reverse the no-big-bang decision

Structural audit의 8 Active findings는 architecture 실패를 강화하지만 다음 사실을 반증하지 않는다.

- 단일 façade와 exhaustive route라는 현실적인 migration seam이 있다.
- 올바른 staged patterns가 이미 동작한다.
- official data pipeline, checkpoint, determinism과 domain tests가 usable oracle이다.
- finding은 네 canonical root에 집중되며 gameplay algorithms 전체의 의미가 틀렸다는 증거가 없다.
- route/owner 단위로 single-authority cutover가 가능하다.

오히려 concrete defects는 “현재 내부를 그대로 보존하지 말라”는 partial boundary rewrite 근거다. 그것들을 whole-core rewrite 근거로 확대하면 가장 필요한 parity oracle을 동시에 폐기한다.

### 8.3 Evidence considered in favor of a big-bang rewrite

- application root와 protocol surface가 넓다.
- 프로젝트는 출시 전이며 일정 압박이 없다.
- server와 Unity 소비자는 상대적으로 작아 intentional cutover 비용이 제한적일 수 있다.
- 새 구조를 처음부터 정리할 자유가 있다.

이 장점들은 boundary rewrite를 충분히 수행할 여유를 높이지만, tested gameplay meaning·determinism·save/data semantics를 한 번에 재발견하는 위험보다 크다는 증거는 아니다.

### 8.4 Reconsider whole-core rewrite only if

- differential oracle이 기존 gameplay semantics를 신뢰하거나 분리할 수 없음을 증명함;
- façade/setup/data/checkpoint seam으로 single-authority vertical migration이 불가능함;
- 승인된 정책이 현재 Rulebook/domain/data model 대부분을 폐기함;
- representative workload가 session/history 경계가 아니라 retained kernel algorithms 자체의 budget 실패를 증명함;
- checkpoint/identity behavior가 광범위하게 비결정적이라 migration source로 쓸 수 없음.

현재 evidence에는 하나도 성립하지 않는다.

## 9. Remediation dependency map

이 절은 후속 Goal 후보와 종속성이다. 구현/API 승인이 아니다.

| Unit | Findings | Purpose | Prerequisite | Preservation oracle | Recommendation class |
|---|---|---|---|---|---|
| `RU-00` Observation and migration oracle freeze | all | canonical command/result/state/event/continuation 및 consumer inventory 고정 | none | PA-001, PA-010~012, MS-008 | `evidence-backed prerequisite` |
| `RU-01` Application command transaction/publish boundary | CA-002/003/004 | rejection equality, projection-before-publish, single writer를 command root invariant로 만듦 | RU-00 | staged helpers MS-001/002; VD-01/02 | `evidence-backed high priority` |
| `RU-02` Validated data, fixture and admin/test surface boundary | CA-007/010; CA-004 contributor | production validated construction과 fixture/debug authority 분리 | RU-00; coordinate CA-004 with RU-01 | PA-009, live publication parity | `evidence-backed` |
| `RU-03` Battle kernel authority and validated input boundary | CA-005/006 | public mutation/input/dependency를 좁히고 algorithms 보존 | RU-00; RU-02 scoped data seam beneficial | PA-003~006, MS-003/004 | `evidence-backed` |
| `RU-04` Active battle session/history resource boundary | CA-009; CA-003 mechanism | scoped staging, direct delta, bounded indexed reconnect/archive | RU-01 transaction semantics; RU-03 boundary; VD-04 policy | PA-003/011, MS-007, current cursor/event semantics | `evidence-backed`, release caps `policy-required` |
| `RU-05` Typed session/run aggregate and use-case extraction | CA-001 | broad parallel root를 vertical lifecycle slices로 교체 | RU-00, stable RU-01 application port | PA-001/002/007/008/010, MS-001/006 | `evidence-backed` |
| `RU-06` Versioned protocol adapter boundary | CA-008 | domain/application result와 external schema ownership 분리 | RU-00 consumer inventory; stable application result semantics; explicit consumer policy | current 47-result projections, MS-005 | `evidence-backed`; cutoff `policy-required` |

### 9.1 Recommended sequence

1. `RU-00`은 모든 migration의 필수 선행이다.
2. 첫 correctness remediation은 `RU-01`이 가장 강하게 지지된다. 이유는 premature draft가 아니라 CA-002/003의 S1 severity, direct failed-command authority impact, 그리고 다른 state/history migration이 의존할 single-writer seam이다.
3. `RU-02`와 `RU-03`은 다음 compiler boundary 묶음이다. CA-004는 RU-01 atomicity와 RU-02 validated reachability를 모두 검증해야 한다.
4. `RU-04`는 measured operational defect를 제거하되 transaction/kernel port와 product retention budget을 먼저 요구한다.
5. `RU-05`는 broad application change pressure를 해결하지만 가장 넓은 semantic surface이므로 안정된 command port와 slice oracle 뒤에 수행한다.
6. `RU-06` adapter 설계는 consumer inventory 뒤에 시작할 수 있으나 cutover는 application result semantics와 compatibility policy가 안정된 뒤 수행한다.
7. 새 Relic/corrosion/dice/tool/boss orchestration은 owner/compiler/transaction 경계가 정해진 뒤 각각 bounded feature Goal로 수행한다.

RU-01의 우선 권고는 특정 `AtomicTransaction` trait, enum 이름, clone 전략 또는 file layout을 승인하지 않는다. 후속 Goal은 design alternatives와 performance/equality gate를 다시 명시해야 한다.

## 10. Verification debt and policy decisions

| ID | Debt / decision | Type | Blocks |
|---|---|---|---|
| `VD-01` | six battle command future-cursor complete authority equality and retry matrix | remediation verification | CA-003 closure / RU-01 |
| `VD-02` | StartNewGame trigger-failure full-session equality and retry | remediation verification | CA-004 closure / RU-01+02 |
| `VD-03` | canonical multi-domain original-vs-continued command/result/event/final-state tape | migration oracle | broad RU-05/06 certification |
| `VD-04` | event/time/byte retention, tick deadline, concurrent-session caps and permanent scale guard | mixed policy + verification | RU-04 release certification |
| `VD-05` | semantic versus bitwise cross-platform physics replay guarantee | policy | cross-platform determinism certification; current architecture decision은 차단하지 않음 |

Additional limits:

- server-wide persistence/crash/delivery semantics and Unity compatibility are not certified.
- current ordinary battle workload is light, but event-heavy boss/concurrent-session allocation was not measured.
- future feature design notes express pressure and intent; unspecified values are not current runtime requirements.

## 11. Evidence traceability

| Canonical item | Structural evidence | Architecture evidence |
|---|---|---|
| CA-001 | DESIGN_REVIEW 3.1, RC-01~03 context | ARCH-REVIEW-001; domain/aggregate/change maps |
| CA-002 | RC-02, transaction catalog | ARCH-REVIEW-004/005; operations map |
| CA-003 | RUNTIME-AUDIT-005, GAP-001 | ARCH-REVIEW-004/005, S03 |
| CA-004 | RUNTIME-AUDIT-006, GAP-002 | ARCH-REVIEW-005/006, S08 |
| CA-005 | RUNTIME-AUDIT-003 | ARCH-REVIEW-003; dependency/testability maps |
| CA-006 | RUNTIME-AUDIT-004, determinism inventory | ARCH-REVIEW-003; PAST_AUDIT_CROSSCHECK |
| CA-007 | RUNTIME-AUDIT-007, data publication map | ARCH-REVIEW-006; data evolution map |
| CA-008 | RUNTIME-AUDIT-008 | ARCH-REVIEW-001/003/005; rewrite decision protocol disposition |
| CA-009 | RUNTIME-AUDIT-009, E-021, GAP-004 | ARCH-REVIEW-002, E-020, performance budget |
| CA-010 | RUNTIME-AUDIT-001 | ARCH-REVIEW-001 cause; PAST_AUDIT_CROSSCHECK |
| CR-01~04 | Structural RC-01~03 and design review | architecture RC-A~C, options and preservation register |
| rewrite decision | Structural DESIGN_REVIEW sections 8/11/12 | ARCHITECTURE_OPTIONS, REWRITE_DECISION, PA-001~012/MS-001~008 |

`FINDING_CROSSWALK.md`가 source ID 관계의 상세 권위이며, 각 source audit의 `FINDINGS.md`와 `EVIDENCE_MATRIX.md`가 raw evidence authority다.

## 12. Handoff

이 문서는 acceptance gate 통과 후 두 감사를 대표하는 canonical decision base가 된다. 원본 감사는 상세 evidence로 계속 보존한다.

다음 사용자 결정은 후속 remediation Goal 승인 여부다. 승인한다면 `RU-00` 또는 `RU-00`을 명시적으로 포함한 첫 remediation Goal부터 설계한다. 기존 `current_runtime_transaction_kernel_foundation` 초안은 이 문서보다 권위가 낮고, 그대로 실행하지 말고 CA/RU/VD mapping에 맞춰 새로 검토해야 한다.

이 통합 Goal은 production 수정, 후속 Goal 생성, commit 또는 push를 수행하지 않는다.
