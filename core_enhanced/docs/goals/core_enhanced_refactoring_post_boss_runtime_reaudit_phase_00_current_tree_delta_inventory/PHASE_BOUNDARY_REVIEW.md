# Phase 00 Boundary Review

Verdict: the Master Phase 1~10 boundaries remain materially correct for the approved 51-candidate + Boss reaudit.
No scope reduction, production activation or Master restructure is required.

## Coverage Reconciliation

| Reaudit Phase | Original final rows primarily owned | Shared final contribution | Boss/current delta responsibility | Coverage result |
| --- | ---: | --- | --- | --- |
| P01 live data loading/publication | 3 | publication contribution to P05/P08/P09 | embedded Boss source, compile and atomic candidate | sufficient |
| P02 run/map/persistence | 3 | F-C01 with P03 | private/no-save Boss impact and current world/server persistence changes | sufficient |
| P03 roster/inventory/economy | 3 | F-C01 with P02; F-C02 with P04 | non-Boss current-tree impact | sufficient |
| P04 interactive node/VN | 3 | F-C02 with P03 | non-Boss current-tree impact | sufficient |
| P05 combat preview/setup | 5 | P01 publication seam | Boss binding, typed setup and scenario publication | sufficient |
| P06 lifecycle/event/result | 7 | atomic seam with P07 | Boss wake/order/atomic boundary/terminal behavior | sufficient |
| P07 spatial/tactical | 4 | movement and staging seams | Boss target lock/path/movement plus current tactical shared files | sufficient |
| P08 effects/stats | 6 | occurrence/publication/delivery seams | Boss cancellation and retained leaf authority | sufficient |
| P09 targeting/delivery/projectile | 8 | effect and public cleanup seams | Boss target/path/motion intersection; P09-C05 type normalization | sufficient |
| P10 command/result/server | 7 | public DTO/consumer seam | private/public Boss boundary and accessible server/Unity evidence | sufficient |
| Final | 2 merged identities plus all 49 unchanged rows | all cross-boundary dedupe | new candidate dedupe, test/wave/gate catalogs | sufficient |

There are 49 unchanged final rows allocated directly by their `CRA-Pxx-*` prefix and two merged final identities.
The contribution links preserve all 53 source IDs while Final owns exactly 51 current dispositions.

## Boundary Decisions

### Keep P01 and P05 separate

P01 owns raw source inventory, compile/reference validation and atomic database publication. P05 owns resolved
encounter/actor setup and scenario/world feasibility. The current Boss path connects these stages but does not make
raw data validation and battle setup one authority.

### Keep P06, P07, P08 and P09 separate

P06 owns lifecycle, same-time event ordering, terminal outcome and atomic advance. P07 owns spatial/tactical and
movement selection. P08 owns effects/stats/damage/death leaf seams. P09 owns skill target/delivery/projectile and
reach projection. `boss_runtime.rs` touches all four, but its shared module placement is not root-cause identity.

### Keep P10 as the public/external owner

Current scoped search finds private Core Boss state and no Boss-specific server/Unity mapping. P10 must prove this
against the actual producer DTO, server result/snapshot mapping, external canonical/source and unknown deployed
consumers. Earlier Phases may record internal shape but cannot declare the external contract verified.

### Do not absorb the tactical campaign

Untracked `field_work.rs`, `tactical_relocation.rs`, `tactical_stance.rs`, `tactical_swap.rs` and their shared-file
changes are user-owned current-tree evidence. They can change the validity/dependency of existing P07/P08/P10
candidates, but their independent policy/feature backlog is outside this reaudit. Shared-file impact is assigned to
the relevant Phase without expanding the Master.

### Preserve merged-candidate contribution

- P02 and P03 both audit the full current F-C01 boundary; Final confirms whether one merged identity still holds.
- P03 and P04 both audit the full current F-C02 boundary; Final confirms whether one merged identity still holds.
- Neither contribution may independently mark the final candidate fully implemented.

## Sequential Dependency Gate

The Master registry order remains:

```text
P00 -> P01 -> P02 -> P03 -> P04 -> P05 -> P06 -> P07 -> P08 -> P09 -> P10 -> Final
```

This is an audit evidence order, not a claim that eventual production implementation must use the same waves.
Sequential execution is retained because each child consumes the current dirty baseline and earlier cross-boundary
findings. Exactly one child is active. Every handoff requires child `COMPLETION_AUDIT.md` = `Complete / High` and an
explicit Master registry transition.

## New Candidate ID Rule

Scoped search found no existing `CRR-(Pxx|F)-Cyy` identifier. The reaudit uses:

- Phase proposal: `CRR-P01-C01`, ..., `CRR-P10-Cyy`
- Final-only merged/deduplicated identity: `CRR-F-C01`, ...
- Test-only backlog item remains attached to its candidate; Final may use `CRR-TC-yy` only for a cross-candidate test
  consolidation that has no single candidate owner.

Rules:

- never reuse or renumber historical `CRA-*` IDs;
- never reuse Boss policy `BP-*` or tactical policy IDs;
- allocate numbers in discovery order within the owning Phase and do not fill gaps by moving identities;
- a cross-boundary duplicate keeps every Phase proposal in traceability and Final assigns one `CRR-F-*` identity;
- no ID is created for a search hit, shared file, future possibility or merely large module;
- every new ID must include current Problem Evidence, authority/producer/consumer, target authority, preserved
  invariant, removal scope, work type, seven axes/readiness, safety net, gate and disposition.

Command/result:

```text
rg -n 'CRR-(P[0-9]{2}|F)-C[0-9]{2}' core_enhanced game_server game_resources <Unity docs>
# no output
```

## Test Review Ownership

| Test surface | Primary owner | Required cross-check |
| --- | --- | --- |
| data domain/compiler, `live_ron`, `ron_loading` | P01 | P05/P08/P09 consume their specific publication predicates |
| checkpoint/map/host persistence | P02 | P03 F-C01 graph contribution; P10 host boundary |
| inventory/employee/reward/shop/equipment | P03 | P04 F-C02 session contribution |
| Event/Shop/HQ/VN/world node tests | P04 | P03 F-C02 transaction contribution |
| combat preview/setup/scenario/world start | P05 | P01 publication; P06 first execution |
| battle event/order/terminal/advance/log | P06 | P07 atomic staging; P10 public projection |
| battlefield/movement/tactical command | P07 | P08 effects and P09 path/delivery distinctions |
| ability/buff/stat/damage/death/effect | P08 | P01 publication and P09 delivery |
| target/range/skill/projectile/delivery | P09 | P08 committed effect and P10 DTO/public cleanup |
| behavior/result/snapshot/server/package/external fixtures | P10 | all public claims from earlier Phases |

Final deduplicates retain/strengthen/merge/remove/unverified decisions. A broader world scenario does not replace a
unit/data/server layer unless the unique invariant is explicitly transferred.

## Consumer Classification Gate

Every candidate with public/save/DTO/server/Unity implications records one of:

- `workspace_confirmed` — current producer and consumer source inspected;
- `canonical_external` — current canonical names an external consumer, implementation not inspected;
- `inaccessible_known` — consumer known but unavailable;
- `unknown_deployment` — existence or deployed revision unknown;
- `explicitly_excluded` — independent component refactor outside Master scope.

P10 owns final public consumer normalization. `No search hit` alone cannot upgrade a consumer to absent. A required
inaccessible consumer becomes an external gate; an optional one remains an explicit risk.

## Validation And Omission Decision

P00 does not need a runtime runner to prove identity and ownership boundaries. The current source/data/test/canonical
inspection closes the inventory questions, while behavior verdicts are deliberately delegated. The full workspace
suite would only repeat the historical green count and cannot prove the 51 mapping or test meaning.

Required P00 static checks:

- final matrix rows = 51; duplicate final IDs = none;
- unique source candidate IDs = 53;
- expanded Boss set = 19 and every ID is present in the final matrix;
- remaining impact-review set = 32;
- phase registry has P00 plus P01~P10 and Final, with only P00 active;
- new namespace collision search = no output;
- current relevant content/status baselines are reproducible.

## Boundary Verdict

`sufficient / High confidence` for starting P01 after P00 exit audit. The boundaries preserve every approved
candidate and Boss seam, expose cross-Phase contributions instead of merging shared files, and leave production
implementation unauthorized. No policy, external or scope stop condition was found at P00.

