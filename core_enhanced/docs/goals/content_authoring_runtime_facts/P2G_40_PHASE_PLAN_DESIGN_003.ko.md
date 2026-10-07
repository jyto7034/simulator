# R04 최소 Phase 계획 — P2G-40

2026-09-11 KST. Plan `PP-CARF-P2G40-003`, bound Freeze `FPF-CARF-P2G30-002`, WR `WR-CARF-P2G20-001`. 구조 authority는 동명의 JSON이다. 표준 receipt와 complete result 전에는 non-current candidate다. 구현/게임 검증 완료가 아니다.

Repair Basis: COD-CARF-001/O40, prior O20/O30 resolved; RR-CARF-O40-001 closes O40 together with passed receipt and complete result. No supersession. Parallelization: none approved.

정책·Decision·acceptance는 Bound Freeze의 stable selector로 파생하며 여기서 재작성하지 않는다. [CRA](CODE_REVIEW_002.ko.md)의 실제 B01–05, 제안 P01–07, validation V01–10을 인수한다.

## Work Package와 최소 동시 전환

- `WP-CARF-01` — BattleCore actual Damage/Heal/Cost/Death producers, predeath observation and mandatory death/wave vs optional dispatch; immutable current causal context through basic/skill/proc/buff/projectile/area; real equipment-copy binding through both player drafts/build constructors and stat/source consumers. V01–V04/V07 producer-level tests and real public consumers: overkill/overheal/negative Heal, real Cost/failure, mandatory once, predeath own-source vs killer, segment/strike, delayed source/log caps, two equipment copies and actual constructors.
- `WP-CARF-02` — Fallible current request/action store, opaque attempt validity/epoch and Accepted/Released/FlowFinished/EffectsSettled with exactly-once control terminal; cause-specific cancellation and surviving released deliveries; public operation clone/commit, entry/result/retreat/reconnect/durable applicability, native/authoring bounds and public API/CLI projection. V05/V06/V08/V09/V10: typed no-target/limit failure, actual t0/100/300 and cancel50/150, duplicate/stale/UUID-reuse/rollback, finite carrier settlement and native bounds, save/live/store lifetime, determinism and real default-release consumers.
- `WP-CARF-03` — Durable focused/integration tests, isolated literal RON fixtures, actual public library/CLI consumers, lifecycle/determinism/native-bound/cost measurements and affected predecessor currentness for all frozen R04 acceptance. All frozen acceptance mapped by IVA01–10 with exact source/toolchain/fixture/command/expected-actual/raw output; meaningful full core/all-target/default-release integration and relevant current predecessor evidence; no skip/0-test/runtime placeholder accepted.

`AG-CARF-01` = WP01 + WP02. 실제 결과/출처는 action/attempt identity와 bounded transactional store를 필요로 하고, action EffectsSettled/public result는 실제 mutation/source와 carrier 완료 사실을 소비한다. 두 방향 관계를 끊어 log-only fallback이나 request-success를 effect-success로 내놓는 중간 상태를 승인할 수 없다. 이는 구현 cutover의 최소 묶음이며 모든 gameplay command를 하나의 새 거대 transaction으로 바꾸는 의미가 아니다. Equipment AT04의 두 draft/build/stat/source 전환은 WP01 안에서 완결한다.

응축 그래프 `AG-CARF-01 → WP-CARF-03`를 `PHASE-CARF-01` 하나에 배정한다. 새 durable 검증 seam이 아직 없으므로 WP03을 별도 완료 Phase로 미루지 않는다. 구현 중 focused 검증은 가능하지만 Phase 완료에는 실제 integration/currentness까지 필요하다. 미래 Flow/boss adapter/ResourceStore/fragment copy를 선행으로 만들지 않는다.

## 공유 계약과 유효 중간 상태

| Contract | Review source | Meaning owner and producer/consumer | Implementing node | Validating node | Availability and boundary |
| --- | --- | --- | --- | --- | --- |
| SC01 facts/source | P01/P02 | E01/E02; mutation/basic/skill/proc/buff/delivery producer → current gate/mandatory bookkeeping/action summaries/public projection | AG-CARF-01 | WP-CARF-03 | Existing mutation semantics remain authority; new typed port only usable after all current producer/consumer paths converted. Strike/segment/source and actual outcome ownership fixed. |
| SC02 action/attempt | P03/P05 | E03/E05; qualified public request → engine action store → carriers → terminal/settlement query | AG-CARF-01 | WP-CARF-03 | Current carriers exist; new handle does not. Control terminal and detached settlement remain distinct, cause-specific cancellation preserved; token validity independent of gameplay IDs/logs. |
| SC03 equipment | P04 | E04; actual inventory/EquippedRef → both player drafts/builds → RuntimeItem/stat/source | AG-CARF-01 | WP-CARF-03 | Existing serialized inventory ID remains authority; paired runtime binding is proposed and must cover all constructors. Definition/runtime/copy identities remain distinct. |
| SC04 operation/lifetime | P05/P06 | E05; bare BattleCore or outer CoreApplication operation → staged state → commit/live/result/drop; V6 inventory store | AG-CARF-01 | WP-CARF-03 | New native state must join clone/admission and transaction before public exposure. Prior accepted timestamps preserved; attempt state durable N/A tested, old handles cannot affect retry. |
| SC05 bounds/consumer | P06/P07 | E02/E03/E05; bounded runtime state → mandatory observation/API/CLI/describe | AG-CARF-01 | WP-CARF-03 | Native and authoring finite faults apply at introduction. Exact cap values measured in GEX; optional log/diag retention cannot change fact meaning/RNG. No unsupported success. |
| SC06 currentness | P07/V10 | E01–E05; source delta → predecessor/direct consumer tests/probes → current proof and fresh reviewer | WP-CARF-03 | separate applicable fresh GEX-20 | R01–R03 current now. Source changes require scoped current evidence or positive non-impact proof; no reusing stale all-pass claim. |

R05 and later absent: current core emits actual Damage/Heal/Cost/Death with real source/copy and distinct strike/segment; current request handles expose correct lifecycle/cancellation/settlement; native bounded attempt state commits/rolls back/reclaims correctly; public API/CLI and existing battle semantics remain usable.

Single actual mutation/death/resource authority; one producer-consumer cutover; existing formula/hit gates/RNG/cause-specific lifetimes; all constructors match paired equipment data; same operation state rollback, live snapshot and V6 inventory meaning; exact stale identity; finite retention; no log authority or fabricated source/fallback/full future executor.

이것은 Phase 설계의 intermediate-state audit이며 실제 runtime PASS가 아니다. 후속 R이 실행되지 않아도 이 Phase는 독립적으로 유효해야 한다. 정확한 Rust spelling/동등한 자료구조/측정 safety 수치는 GEX 재량이나 owner·수명·실패·취소·공개 소비자 의미는 변경할 수 없다.

## Integration Validation Ownership

- `IVA-CARF-01` (V01, Actual damage/basic/Heal) → `WP-CARF-03` / `PHASE-CARF-01`: Same seed/input formula, existing basic-only OnAttack/OnHit/RNG/resonance and HP-loss step gate; overkill/overheal/negative Heal requested vs actual with unchanged old actual damage summary. Existing self-heal/repeat-barrage and basic snapshot regressions.
- `IVA-CARF-02` (V02, Actual Cost) → `WP-CARF-03` / `PHASE-CARF-01`: Current manual/auto resonance reset and deploy/relocation/swap actual tactical pay; insufficient/rejected requests typed fail/no successful receipt, resource and operation state unchanged. Negative resource modifiers remain explicit change rather than invented cost.
- `IVA-CARF-03` (V03, Death/predeath ordering) → `WP-CARF-03` / `PHASE-CARF-01`: Lethal damage/stat-HP/direct summon Kill and separate Despawn; suppression of optional dispatch still mandatory death/wave once. Victim caster-A buff vs killer-B snapshot before cleanup, unchanged OnDeath/OnKill/sorted ally order, no invented proc.
- `IVA-CARF-04` (V04, Occurrence/provenance) → `WP-CARF-03` / `PHASE-CARF-01`: Multiple component/effect segments of one direct strike vs distinct carrier iteration/target/cast; basic/skill/proc/buff tick/projectile/area after source death. Source/action/strike facts persist with logs off/evicted; real current public observer and truthful unsupported scope.
- `IVA-CARF-05` (V05, Request/milestones) → `WP-CARF-03` / `PHASE-CARF-01`: No-target/invalid request and exact finite-limit rejection on bare BattleCore and public world ports; t0 Accepted/t100 Released/t300 hit with FlowFinished and EffectsSettled observed from actual current carrier, no later Flow adapter.
- `IVA-CARF-06` (V06, Cancellation/stale/settlement) → `WP-CARF-03` / `PHASE-CARF-01`: Existing cause cancel at t50/t150, unreleased steps absent and released projectile/GroundFixed continue; Follow area and HardCC/Silence/pending InterruptCast regression. Terminal once, late settlement distinct, stale/duplicate handles cannot overwrite new action.
- `IVA-CARF-07` (V07, Equipment copy cutover) → `WP-CARF-03` / `PHASE-CARF-01`: Two actual owned copies with same definition through both player draft/build and deploy paths; source IDs equal inventory IDs and preserve runtime RNG/cooldown/stat. Scenario/fragment absence explicit; durable inventory identity remains compatible.
- `IVA-CARF-08` (V08, Atomicity/attempt/save) → `WP-CARF-03` / `PHASE-CARF-01`: Failed request/advance state clone comparison includes new handle/source/receipt plus resource/queue/RNG/ID/log and prior successful timestamp prefix. Same-attempt live observation/reconnect, candidate rollback, retreat/reentry UUID reuse, result/drop/reclaim. Current V6 loader/writer proves attempt durable N/A; inventory ID roundtrip.
- `IVA-CARF-09` (V09, Bounds/determinism/cost) → `WP-CARF-03` / `PHASE-CARF-01`: Native and authoring work/cardinality/retained result/source/predeath/clone/fanout ceilings, inclusive cap/one-under and repeated failure/recovery. Same seed/input/pack/container order and log/diag retention comparisons; measured high-water/allocator/drop/retention and plain latency separated from allocator timing, no unbounded full-capacity scan.
- `IVA-CARF-10` (V10, Current integrated acceptance) → `WP-CARF-03` / `PHASE-CARF-01`: T07/T23/T26/T28 applicable R04 with independent literal external RON through default-release public API/CLI, focused/full core/all-target/release and actual R01–R03 direct-consumer impact. Canonical contracts truthfully updated; unchanged predecessor evidence scoped or restored; stable target then separate fresh GEX-20.

모든 새 durable test/fixture/probe는 assigned_execution_node다. 기존 시험은 exact current basis일 때만 재사용하며 새 R04 acceptance를 과거 PASS로 대체하지 않는다. Source 결함 교정은 AG owner로 반환한다. 실제 source/dirty·toolchain·workload·command·nonzero count·expected/actual/raw receipt, full/default-release/current predecessor 증거와 known limits를 남긴 뒤 구현을 멈추고 별도 fresh reviewer에 인계한다.

## Known Unknown

Freeze KU30-01 exact types/carrier tables/budget values remain AG owner in reviewed P01–P07; KU30-02 actual verification/cost belongs WP03; KU30-03 impacted R01–R03 evidence must be current before completion. KU30-04 external host/deployment/personal-store content N/A. New material meaning or wrong reviewed premise returns to earliest P2G owner; no future R feature is declared complete.

Candidate001은 receipt ref를 string으로 기록해 표준 parser가 거부했다. [교정 기록](P2G_40_CANDIDATE_CORRECTION_001.json)에 따라 typed ref를 가진 immutable candidate002로 대체하며 정책·경계·Phase graph는 동일하다.


## Technical successor

Predecessor PP-CARF-P2G40-002 is stale under COD-CARF-001. Exact new Freeze002 retains all5PER/20acceptance. Same 3WP/4edges/1AG/2nodes/1Phase/10IVA and stable member mapping.  GF001/002 required source regressions: pending→death→CastEnd preserves actual resonance reset without false release; death after release preserves existing queued step/Follow behavior. Piercing earlier hit→empty final expiry settles actual delivery only after pending effects, without invoking old damage-gate completion or inventing FlowFinished while deferred control remains; include duplicate/rollback/finite retention. These concrete oracles refine existing IVA01/02/05/06/08/09 under WP03; implementation remains AG01. No new final node or scope.
