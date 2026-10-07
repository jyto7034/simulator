# Game Rulebook Policy Implementation Preflight Matrix

Status: complete_for_pf01_pf02_pf03_pf04_pf05_pf07
Evidence Date: 2026-08-12
Repository HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Production Mutation: none

## Purpose And Evidence Boundary

이 문서는 [Game Rulebook](../../game_rulebook.md)의 승인 정책과
[readiness audit](../game_rulebook_policy_implementation_readiness_audit/PLAN.md)을 다음 production Goal이
바로 test-first로 사용할 수 있는 상태 전이, 원자성, 수명주기, migration, bootstrap 및 activation
계약으로 정규화한다. 현재 runtime이 이미 아래 목표 동작을 구현했다는 증거가 아니다.

Current-tree 사실은 runtime code, production RON, DTO producer와 store를 우선해 판정했다. 목표 의미는
rulebook과 `GRB-PD-001`~`010`을 따른다. Unity current-tree/runner는 사용자 지시로 이번 Preflight의
증거 범위에서 제외하며, Unity consumer가 일치하거나 production package에서 제외됐다고 추론하지 않는다.

## PF-01 — Entry Drift And Collision Reconciliation

### Evidence freeze

| Evidence | Current value | Disposition |
| --- | --- | --- |
| Repository HEAD | `f318414fb0156a67e04c5479000c2a2683fc0c09` | Preflight baseline anchor |
| `game_rulebook.md` SHA-256 | `d222e4cc937c972ed13472901e1dd6aa91d46f8c1d3b64b9d9311ea572d80b53` | current approved gameplay target |
| Readiness `PLAN.md` SHA-256 | `9911e4293da5ebc54b983180e792bfde08cba5e82350af19abd1f2c1d1b4ff4e` | ATI-03 input |
| `POLICY_DECISION_LOG.md` SHA-256 | `af8e47c173e9d3da2537e09750768a7d10f6652fdcb62ed927c0753a0c890e72` | closed decisions input |
| `WORK_PACKAGE_MAP.md` SHA-256 | `8442dd2dd95185a93bab4c66b94de0b1cdaf662622d664918e93a8b7dd9b68a0` | package boundary input |
| `IMPLEMENTATION_SEQUENCE.md` SHA-256 | `5d68d1f502809bd3d7ce2b2d1fd4b41a31e23249d67f6cd9f6dcbbd87b939de6` | ATI-04 input |
| `CONTENT_INPUT_CANDIDATES.md` SHA-256 | `884249ff92488e07dd19a09ec6160e91fefc1e79f7306e9c4277f10cf707ead1` | minimum bootstrap input |
| Active simulator Goal | this Preflight only | no active production owner collision |

### Current tree versus approved target

| Area | Current authoritative fact | Approved target | Collision/rewind disposition |
| --- | --- | --- | --- |
| `WP002` Monolith | Live RON is stat `100/125`; validation is positive and ordered only. Resolver uses u64 composition and saturates the projected u32. | Initial `150/200`, RON-tunable, positive/ordered/safe. Reward values and wave composition unchanged. | `run_policy_data.rs` and `run/policy.ron` are dirty, but their current diff has 0 target Monolith-field hunk hits. No ATI rewind. Activation must recheck exact hunks and preserve unrelated user edits. |
| `WP001` progression/economy | Research/pending delivery, old reward effects, generated encounter weighting and current BossOmen state remain live; V3 is current. Target reward-source/cycle fields are absent. | One V4 encounter ledger, no research/pending path, explicit source/repeat authoring, target cycles and BossOmen failure semantics. | Expected implementation gap, not target drift. Broad dirty overlap requires package-local merge review and fresh baseline, but current authority/package boundary is unchanged. |
| `WP003` Standard/profile | Standard is generated/adaptive and ends in a FinalBoss; persistent store is compendium V1 only. | Fixed `ST1-NT60`, F3 Nothing There NormalBoss completion, unified monotonic successor profile and Endless gate. | Three new tutorial reward-pool IDs are intentionally authored inside WP003; reused IDs exist. No ATI-02 rewind or replacement content selection. |
| `WP004` sweep | No typed sweep command/result exists. Existing consumables have `NextCombatNode` or `CombatNodes(n)` duration. | Mastered Elite direct/sweep choice and synthetic atomic result with normal active-consumable node-duration decrement. | Rulebook does not require a dedicated sweep consumable. “Sweep item” is resolved as the already-active consumable modifier whose duration is decremented; inventing a special cost/item would require a new content decision. |
| BossOmen bootstrap | One live `white_night` chain, min Floor 3, one Event step and `suppress_white_night`; current runtime uses a global any-awakened gate. | Same one-step development chain as sole live membership, generic per-chain eligibility/cycle/failure engine, no global fallback. | Minimum bootstrap authors no required fragment because none was approved and the old global gate is explicitly removed. Synthetic test-only chains cover DAG/cycle/failure validation; they are not live content. |
| Fragment skills | 29 `BasicAttackModifier` placeholder entries remain live. | Preserve placeholders in this campaign; dedicated skills are future `WP005`. | Intentional and non-blocking. No conversion in current sequence. |
| Unity | Adjacent project is materially drifted and unverified in this Preflight. | No target behavior claim in this dossier. | `explicitly_excluded_by_user`; not a `WP002` blocker. Later public-contract package activation must state its own consumer boundary. |

### Historical documentation drift

`CONTENT_INPUT_CANDIDATES.md`의 detailed PD005 resolution, decision log와 rulebook은 one-step WhiteNight
`A-bootstrap`에 일치한다. 같은 문서 마지막 `Bootstrap Resolution`의 “Option B closed” 한 줄은 stale
summary다. 완료된 과거 Goal은 수정하지 않고, 이 Preflight는 더 최신이고 구체적인 세 근거를 적용한다.
이는 policy 재개방이나 ATI-02 rewind 사유가 아니다.

### Safe-range technical contract for WP002

승인 정책은 gameplay 상한 숫자를 정하지 않았다. 따라서 Preflight는 임의의 밸런스 cap을 추가하지
않고 다음 representability invariant를 `safe-range`로 고정한다.

- 두 stat multiplier는 `> 0`이고 Stage 3가 Stage 2보다 커야 한다.
- 각 multiplier를 Monolith role의 `target_power`, max HP, attack, defense percentage에 합성할 때 checked
  integer calculation이 성공하고 결과가 `u32`에 정확히 표현돼야 한다.
- overflow/saturation으로 `u32::MAX`를 조용히 만드는 값은 loader에서 거부한다.
- Initial `150/200`은 위 contract와 실제 derived tuning assertion을 통과해야 한다.
- Reward multiplier, generated wave budget, extra waves와 rounding policy는 이번 package에서 바꾸지 않는다.

이는 표현 가능한 RON tuning만 허용하는 기술 안전 조건이며, 사용자-visible 밸런스 범위를 새로 정하지
않는다.

PF-01 verdict: ATI-01~04로 rewind할 material drift 없음. Current dirty tree는 package activation 시점의
scoped ownership recheck를 요구하지만 `WP002`의 Preflight handoff를 막지 않는다.

## PF-02 — Exact Traceability

### Policy decisions

| Primary decision row | Closed meaning | Findings | Package owner | Acceptance group |
| --- | --- | --- | --- | --- |
| `GRB-PD-001` | V3 strict reject; no semantic transform/dual reader | M002, M005, M008, M010–M014 | `GRB-WP-001` | MIG-01~03 |
| `GRB-PD-002` | V1 compendium sets preserved; new fields false/0 | M001, M004 | `GRB-WP-003` | MIG-04~06 |
| `GRB-PD-003` | explicit stable authored reward-source identity | M005, M007, M008, M010, M012 | `GRB-WP-001`, `GRB-WP-004` | BOOT-01~04, AT-01~04 |
| `GRB-PD-004` | 29-row fixture and explicit dust 5/10/20 | M002, M007–M010, M012 | `GRB-WP-001`, `GRB-WP-004` | BOOT-01~04, AT-01~04 |
| `GRB-PD-005` | sole live one-step WhiteNight bootstrap | M011, M012 | `GRB-WP-001` | BOOT-05~07, BO-01~12 |
| `GRB-PD-006` | fixed `ST1-NT60` minimum tutorial | M001 | `GRB-WP-003` | BOOT-08~10, ST-01~04 |
| `GRB-PD-007` | dedicated fragment skills deferred | M015 | `GRB-WP-005` | BOOT-11 |
| `GRB-PD-008` | unmastered-first mixed Elite fill | M005 | `GRB-WP-001` | EL-01~08 |
| `GRB-PD-009` | initial RON `150/200`, development-tunable | M006 | `GRB-WP-002` | MON-01, MON-06 |
| `GRB-PD-010` | unlimited mastered-sweep dismantle/reacquire cycle | M007, M008 | `GRB-WP-001`, `GRB-WP-004` | SW-01~06, AT-03 |

### Findings

| Primary finding row | Target acceptance | Package | Direct matrix evidence |
| --- | --- | --- | --- |
| `GRB-M-001` | fixed three-Floor Standard, F3 NormalBoss completion | `GRB-WP-003` | ST-01~04, BOOT-08~10 |
| `GRB-M-002` | research/pending path removed end-to-end | `GRB-WP-001` | MIG-01~03, AT-01 |
| `GRB-M-003` | Endless never auto-completes from ledger/boss completion | `GRB-WP-001` | EN-01, BO-11 |
| `GRB-M-004` | monotonic profile, tutorial gate and records | `GRB-WP-003` | ST-02, PR-01~03, MIG-04~06 |
| `GRB-M-005` | Elite unmastered/mastered assignment cycles | `GRB-WP-001` | EL-01~08, MON-01~06 |
| `GRB-M-006` | Monolith stat initial `150/200` and safe derivation | `GRB-WP-002` | PF-01 safe-range contract, MON-06 |
| `GRB-M-007` | mastered Elite sweep vertical slice | `GRB-WP-004` | SW-01~06, AT-03 |
| `GRB-M-008` | explicit first/repeat source ledger | `GRB-WP-001` | AT-01~04, EL-07, NB-03~04, BO-10 |
| `GRB-M-009` | ordinary Reward cannot grant target fragments/research | `GRB-WP-001` | RW-01 |
| `GRB-M-010` | NormalBoss authored cycle | `GRB-WP-001` | NB-01~07 |
| `GRB-M-011` | per-chain BossOmen conditions and fail-closed DAG | `GRB-WP-001` | BO-01~02, BOOT-05~07 |
| `GRB-M-012` | FinalBoss chain completion/cycle | `GRB-WP-001` | BO-03~12, AT-04 |
| `GRB-M-013` | provisional requeue versus active-step retention | `GRB-WP-001` | BO-04~07 |
| `GRB-M-014` | exhausted remanifestation failure is run-local permanent | `GRB-WP-001` | MON-03 |
| `GRB-M-015` | 29 placeholder conversion remains deferred | `GRB-WP-005` | BOOT-11 |

### Work packages

| Primary package row | Findings owned exactly once | Preflight handoff |
| --- | --- | --- |
| `GRB-WP-001` | M002, M003, M005, M008, M009, M010, M011, M012, M013, M014 | `ready_after_wp002_completion_audit` |
| `GRB-WP-002` | M006 | `ready_to_activate` |
| `GRB-WP-003` | M001, M004 | `ready_after_wp001_completion_audit` |
| `GRB-WP-004` | M007 | `ready_after_wp001`; serialize after WP003 under current ownership |
| `GRB-WP-005` | M015 | `future_or_dormant`; not activated by this campaign |

Coverage result: 10/10 decisions, 15/15 findings and 5/5 packages have one primary row. No finding is assigned to
two package owners; shared decisions are consumed inputs, not duplicate implementation authority.

## PF-03 — State-Transition Contract

### Elite assignment, Monolith and mastered cycle

| ID | Trigger/precondition | Successful mutation | Failure/unvisited/rollback meaning | Lifetime |
| --- | --- | --- | --- | --- |
| EL-01 | Manifest Floor Elite slots | Fill distinct unmastered identities first; only shortage slots use distinct mastered candidates; no candidate downgrades that slot to Normal combat | Never duplicate an identity on one Floor; no class fallback identity | run/checkpoint |
| EL-02 | Assigned natural Elite is not visited before Gate | None of victory/mastery/source ledgers change; identity remains/re-enters unmastered candidate set | Assignment itself is not completion | run/checkpoint |
| EL-03 | Natural Stage 1 victory commit | Remove from natural unmastered pool, set natural victory, victory count 1 and Monolith purchase eligibility; normal node/reward/XP commit | Retreat or failed result commit changes none of these | one atomic run transaction |
| EL-04 | Natural Elite exhausts all node attempts | Node becomes completed transit; identity remains unmastered and can appear on later Floors | No permanent Monolith failure and no fragment/mastery | run/checkpoint |
| MON-01 | Eligible Monolith selected on pure Available Normal combat | Preserve node identity/topology, convert encounter/difficulty to next typed stage and decrement one use | Late validation failure consumes no use and does not convert node | run/checkpoint transaction |
| MON-02 | Stage 2/3 first defeat | Consume one of the two attempts and keep same stage/node retryable | Retreat consumes no attempt and restores pre-entry state | run/checkpoint |
| MON-03 | Stage 2/3 second defeat committed | Complete node without reward; mark identity remanifestation-failed for this run | Never opens later stage, fragment, mastery, repeat pool or sweep | run/checkpoint; persists across later Floors |
| MON-04 | Stage 2 victory commit | Victory count becomes 2; normal reward/XP and node completion commit | No fragment/mastery yet | run/checkpoint transaction |
| MON-05 | Stage 3 victory commit | Victory count becomes 3, first/repeat source branch commits, identity becomes mastered and enters repeat pool | Any late participant failure rolls back count, inventory, ledger, reward and node together | run/checkpoint transaction |
| MON-06 | Resolve Stage 2/3 combat tuning | Compose every applied stat from validated RON `150/200` initial values; reward/waves remain independently authored | Zero, unordered or non-representable composition rejects load | live RON + derived node data |
| EL-05 | Manifest mastered shortage slot | Consume candidate only when its node is actually resolved | Assignment preserves current repeat cursor until resolution contract records it | run/checkpoint |
| EL-06 | Assigned mastered Elite is unvisited before Gate | Return same identity to current repeat-cycle candidates | No reward, cursor advance or node result | run/checkpoint |
| EL-07 | Mastered Elite direct-combat victory | Consume current repeat-cycle identity; direct combat uses ordinary first/repeat rule, never sweep reacquisition exception | Retreat/failed commit keeps candidate and ledgers unchanged | run/checkpoint transaction |
| EL-08 | All mastered identities consumed | Refill deterministic run-seeded cycle; same identity may occur on adjacent Floors, but not duplicate within current Floor | Refill does not reset mastery/source completion | run/checkpoint |

### NormalBoss cycle

| ID | Trigger/precondition | Successful mutation | Failure/unvisited/rollback meaning | Lifetime |
| --- | --- | --- | --- | --- |
| NB-01 | New cycle/Floor assignment | Deterministically assign an unconsumed authored NormalBoss identity | No implicit encounter-class substitute | run/checkpoint |
| NB-02 | Assigned NormalBoss unvisited before Gate | Return it to the same current cycle | No source/reward completion | run/checkpoint |
| NB-03 | Victory with source ledger absent | Consume current-cycle identity and atomically grant exactly its authored fragment | Failed commit grants nothing and does not consume identity | run/checkpoint transaction |
| NB-04 | Victory with source ledger present | Consume identity and atomically apply exactly its authored repeat reward | Never applies sweep reacquisition | run/checkpoint transaction |
| NB-05 | Non-final defeat | Consume one node attempt, retain same node for retry | Retreat consumes no attempt | run/checkpoint |
| NB-06 | Exhausted defeat | Complete node and consume identity for current cycle, with no victory reward/source completion | Identity is eligible again after cycle refill | run/checkpoint |
| NB-07 | Every authored identity consumed | Refill complete NormalBoss cycle | Does not reset source-completion ledger | run/checkpoint |

### FinalBoss/BossOmen cycle

| ID | Trigger/precondition | Successful mutation | Failure/unvisited/rollback meaning | Lifetime |
| --- | --- | --- | --- | --- |
| BO-01 | New Floor generation, no active chain | Evaluate each incomplete chain's min Floor, optional exact fragment/awakening and dependencies; choose only eligible set | Locked chains remain incomplete; no global any-awakened fallback | run/checkpoint + live schema |
| BO-02 | Data initialization | Validate chain/step/source/reward refs, dependency DAG, no cycle and authored reachability | Unknown/cyclic/unreachable data fails startup; no fallback/unlock | live data gate |
| BO-03 | Eligible chain chosen and matching source node exists | Place provisional overlay without consuming cycle | If no same-category node exists, defer to later Floor; no cross-category fallback | run/checkpoint |
| BO-04 | Provisional first source unvisited before Gate | Move proposal behind other eligible candidates; do not activate or consume chain | Same chain can be proposed later | run/checkpoint |
| BO-05 | Provisional Event-started combat defeat or Combat source attempts exhausted | Keep chain incomplete and move proposal to back | Do not consume step under generic Event completion | run/checkpoint |
| BO-06 | Provisional first source succeeds | Convert provisional to the sole active chain and commit first step | Until victory or Run failure, cannot abandon/swap/requeue chain | run/checkpoint transaction |
| BO-07 | Active later source fails | Retain active chain and same step for next Floor | Does not end run or advance step; Event-started combat does not retry in same Event node | run/checkpoint |
| BO-08 | Active non-final step succeeds | Mark exact step complete and retain active chain | Late failure leaves same pre-step state | run/checkpoint transaction |
| BO-09 | Last step succeeds | Atomically replace current map with authored capstone containing Boss, concealed Exit Gate and fixed edge | No partially replaced map/chain state | run/checkpoint transaction |
| BO-10 | FinalBoss victory | Apply exactly first fragment or repeat reward plus base reward; complete active chain and current cycle member; expose preauthored Exit Gate | Does not advance Floor and never makes Endless `RunComplete` | run/checkpoint transaction |
| BO-11 | All live normal chains complete | Refill chain cycle; current one-chain bootstrap can recur on later Floors | Refill is not Endless completion and hidden bosses are excluded | run/checkpoint |
| BO-12 | FinalBoss defeat/retreat | Retreat restores pre-entry with no attempt/result. Committed defeat goes immediately to common Run-failure decision or failure | Active chain is never returned to pool; checkpoint load may restore earlier timeline | run/checkpoint + failure state |

### Standard, Endless and permanent profile

| ID | Trigger/precondition | Successful mutation | Failure/rollback meaning | Lifetime |
| --- | --- | --- | --- | --- |
| ST-01 | Start Standard | Manifest fixed three-Floor authored topology/tuning; no adaptive power, Endless cycles or FinalBoss | Missing fixed reference rejects publication | active Standard run |
| ST-02 | F3 reduced Nothing There NormalBoss victory final commit | Commit Standard completion and `standard_tutorial_completed: true` as one host publication transaction | A failure/crash cannot expose only one side; recovery must finish both or expose neither | run terminal + permanent profile |
| ST-03 | F3 defeat or retreat | NormalBoss attempt/retreat rules apply; no tutorial flag | No partial completion/profile grant | run/checkpoint |
| ST-04 | Start Endless | Admit only when permanent tutorial flag is true | False denies entry and reports requirement | permanent profile gate |
| EN-01 | Any Endless ledger, all FinalBoss or hidden-boss completion | Continue Floor generation through Gate | Never auto-produces `RunComplete`; only explicit abandon or unrecoverable failure ends run | active run |
| PR-01 | Highest Floor or deathless streak exceeds stored max | Immediately publish monotonic maximum | Checkpoint load, failure and new run never lower it | permanent profile |
| PR-02 | Current Endless run defeats every current live FinalBoss membership | Immediately set all-FinalBoss record true; one-chain WhiteNight bootstrap is non-vacuous | Later content additions or checkpoint rollback do not clear it | permanent profile |
| PR-03 | Active-run transaction rollback | Preserve already committed profile union/records while restoring run-local state | Only confirmed full profile reset can delete | permanent profile |

### Reward branches, dismantle, sweep and generic failure

| ID | Trigger/precondition | Successful mutation | Failure/rollback meaning | Lifetime |
| --- | --- | --- | --- | --- |
| AT-01 | Eligible direct victory, source not completed | Grant exactly one authored fragment and mark source complete | Cannot also grant repeat reward | one run transaction |
| AT-02 | Eligible direct victory, source completed | Apply exactly authored repeat reward | Direct combat never reacquires a missing fragment | one run transaction |
| AT-03 | Mastered sweep, source completed, owned count 0 | Grant exactly one same fragment; preserve completion/mastery and upgrade/awakening progress | Cannot also apply repeat reward | one run transaction |
| AT-04 | Mastered sweep, source completed, owned count 1+ | Apply exactly repeat reward; add no fragment | Cannot also apply reacquisition | one run transaction |
| DS-01 | Maintenance dismantles final fragment copy | Count becomes 0 and authored dust is granted; identity progress/source/mastery remain | Allows later AT-03 without limit | run/checkpoint transaction |
| SW-01 | Mastered Elite node with at least one living combat-available armed employee | Offer typed direct-combat and sweep choices with Core-owned disabled reasons | Standard, unmastered Elite, NormalBoss and FinalBoss never offer sweep | capability projection |
| SW-02 | Sweep selected | Build battle-independent synthetic victory result; no battle setup/update simulation | Validation/planning failure mutates nothing | staged Core result |
| SW-03 | Sweep result confirmed | Commit base reward, AT-03 or AT-04, full XP to every living roster employee, active-consumable node-duration decrement, repeat cursor and node completion | Dead excluded; no condition, damage, trauma, bonus, discovery, event log, stats or battle record | one run transaction |
| SW-04 | Any late sweep participant fails | Roll back inventory, ledger, XP, duration, cursor, node/session and result projection | Retry observes identical pre-command state | run/checkpoint transaction |
| SW-05 | Reconnect before result confirmation | Reproject the same staged result/capability without replanning or duplicate reward | Consumer cannot infer acceptance from presentation state | producer DTO projection |
| SW-06 | Checkpoint load after committed sweep | Restore inventory, ledger, XP, duration, cursor and node from checkpoint together | Permanent profile and `loads_used` do not roll back | split lifetime |
| RW-01 | Ordinary Reward node/pool load or claim | May grant only its authored non-target economy effects | Direct fragment/research acquisition variant is rejected; no fallback | live data + run transaction |
| TX-01 | Any result commit participant fails before final commit | No user-visible reward, cursor, completion, condition, inventory or result state changes | Error is retryable only from exact pre-command state | Core atomic transaction |
| CP-01 | Valid explicit checkpoint load | Restore all run-local pool/chain/source/mastery/inventory/reward/node state and increment `loads_used` exactly once | Profile and previous `loads_used` never roll back | split run/permanent metadata |

## PF-03 — Atomicity And Lifetime Matrix

| Transaction | Required participants | Commit boundary | Must remain outside rollback | Required failpoint proof |
| --- | --- | --- | --- | --- |
| Direct eligible victory | staged roster condition/XP, rewards, fragment inventory, source/cycle/mastery ledger, attempts, node/session/result | one Core transaction before projection | already committed permanent profile union | fail each grant/ledger/node participant; exact prestate restored |
| Monolith Stage 3 victory | above plus Monolith victory/failure/use state and mastered pool | same result commit | permanent profile | no state where fragment exists but count/mastery does not, or inverse |
| FinalBoss victory | reward branch, chain completion, cycle member, capstone Exit Gate visibility, node/result | one Core transaction | all-FinalBoss permanent record may publish after qualified Core event and never roll back | fail reward, chain, gate and projection in turn; no partial state |
| Mastered sweep | eligibility, synthetic result, base reward, exclusive branch, alive XP, consumable duration, cursor, node/session | one Core transaction with no battle state | permanent profile; battle log/record does not exist | late failure at every participant; no battle/condition/discovery side effect |
| Standard F3 completion | Core terminal result plus successor-profile tutorial flag and durable host publication | one crash-recoverable host transaction/commit marker | unrelated prior permanent profile fields | current/backup/write/rename/restart failpoint shows both completion+flag or neither |
| Monotonic records | qualified Core event and successor profile store | publish immediately; max/union reducer is idempotent | all committed profile state | active-run write/rollback/checkpoint failure cannot erase; duplicate event is idempotent |
| Explicit checkpoint load | validated V4 payload restore plus monotonic `loads_used` increment | validate entire graph before first mutation | profile and prior load count | every one-edge corruption rejects without mutation; success restores exact graph |
| V1 profile migration | strict V1 current/backup decode, set preservation, new defaults, successor publish | bounded one-shot durable migration | source file until successor commit/recovery is durable | current/backup/corrupt/write/restart corpus; no partial set loss |
| Host response/reconnect | Core commit, required durable stores, server mapping, ordered snapshot/result response | publish only after required durability result is known | durable profile events already committed under their own contract | persistence failure and reconnect cannot show accepted command with missing durable state |

The Standard row is stricter than ordinary monotonic record publication because the rulebook explicitly couples F3
completion and tutorial flag. WP003 may choose the concrete journal/combined-store mechanism, but it may not weaken
the two-outcome contract.

## PF-04 — Bootstrap Referential Integrity

| ID | Input | Current check | Activation meaning | Verdict |
| --- | --- | --- | --- | --- |
| BOOT-01 | 29 manifest encounter IDs | 29/29 found in `pve/encounters.ron` | WP001 authors explicit source rows; no substitute | pass |
| BOOT-02 | 29 primary abnormality IDs | 29/29 found in abnormality catalog | exact primary match remains a loader invariant | pass |
| BOOT-03 | 29 fragment IDs | 29/29 found in fragment catalog | exact one fragment per bootstrap source | pass |
| BOOT-04 | class/repeat authoring | Current classes are Elite 13, NormalBoss 12, FinalBoss 5 and target fields have 0 hits | Reclassify Queen/Mountain to NormalBoss, exclude Moonlit Wail, author explicit 5/10/20; no class fallback | target-authoring-ready |
| BOOT-05 | live WhiteNight refs | one chain and one `white_night`; Event and boss refs load in current full bundle | sole reachable live general FinalBoss chain | pass/current-shape |
| BOOT-06 | target BossOmen condition/DAG schema | required fragment/awakening/dependency/cycle validation fields absent | WP001 authors `required_fragment: None` for bootstrap and test-only valid/unknown/cyclic/unreachable fixtures | target-authoring-ready |
| BOOT-07 | dormant FinalBoss rows | Silent Orchestra/Apocalypse Bird encounter/abnormality/fragment IDs exist but no live chain | stay authored dormant identities; not cycle membership or all-boss denominator | pass/dormant |
| BOOT-08 | reused `ST1-NT60` identities | all listed topology, encounter, event, reward, shop, HQ and briefing IDs found | reuse exact IDs; no adaptive assignment | pass |
| BOOT-09 | new tutorial reward pools | `tutorial_f1_training_budget`, `tutorial_f3_salvage`, `tutorial_f3_budget` absent | WP003 must author these exact single-entry pools; loader omission is a package failure, not Preflight policy blocker | target-authoring-ready |
| BOOT-10 | F3 reduced Nothing There | encounter and abnormality exist; target actor-only 720/53/36/33 override absent | WP003 authors boss-actor-only exact values and isolation tests | target-authoring-ready |
| BOOT-11 | temporary fragments | 29 live `BasicAttackModifier` entries | preserve unchanged; `WP005` future only | pass/intentional |

Referential result: `rows=29`, `missing_encounters=0`, `missing_abnormalities=0`, `missing_fragments=0`. The three
new tutorial pools and target schema fields are explicit package-owned authoring work, not silently missing current
references or substitutes.

## PF-05 — Migration, Rollout And Consumer Test Inventory

| ID | Contract | Owner/package | Required fixture/assertion |
| --- | --- | --- | --- |
| MIG-01 | Current Core/server active-run version is V3 | `WP001` Core + active-run store | writer-produced V3 current and backup are rejected as incompatible by V4 reader; bytes remain until explicit reset/new run |
| MIG-02 | V4 exact restore | `WP001` Core | writer-produced V4 round trip; source/cycle/chain/mastery/inventory/economy/node graph identical; one-edge mutation corpus rejects before mutation |
| MIG-03 | No compatibility fallback | `WP001` Core/server | V2/V3/unknown, missing/extra field and research/pending payloads reject; no dual-write, semantic transform or ignored field |
| MIG-04 | Compendium V1 preservation | `WP003` successor profile store | four same-semantic discovered/defeated sets union exactly; tutorial false, numeric records 0, all-FinalBoss false |
| MIG-05 | Successor current/backup/restart | `WP003` server/store | clean, backup recovery, corrupt both, interrupted publish and restart matrix; dormant IDs preserved |
| MIG-06 | Full profile reset only | `WP003` Core/server | checkpoint/new run/failure never delete; confirmed title reset deletes entire profile, not individual records |
| CON-01 | Core DTO producer | `WP001/003/004` | typed source/cycle/result/capability/profile fields serialize exact shape; removed research fields absent |
| CON-02 | game_server mapping | same package as producer | exhaustive behavior/result mapping; unknown target action has no fallback; mappings compile and exact JSON fixtures pass |
| CON-03 | persistence-before-publication | `WP001/003/004` host boundary | success response/snapshot follows required durable commit; write failure returns explicit error and reconnect is consistent |
| CON-04 | result/reconnect idempotence | `WP001/004` | staged/committed result reprojects without duplicate reward, cursor or node completion |
| CON-05 | Unity | package activation owner, not this Preflight | explicitly unverified here; later package records include/split decision and runner evidence without using ignored-field fallback |

Current reusable fixtures include V3 exact-field/graph mutation tests, active-run current/backup recovery, compendium
monotonic merge/store failure tests, combat-result rollback, retreat exactness, BossOmen current E2E and exhaustive
server mapping tests. They are baseline seams only. Target V4/source/cycle/sweep/profile assertions must be written
first in their production package and must replace tests that currently assert research, generated Standard or old
BossOmen behavior.

## PF-07 — Package Activation Checklist

### `GRB-WP-002` — `ready_to_activate`

- Closed input: initial `150/200`; no content or policy question.
- Current loader and focused/full Core runners are available and green.
- Exact dirty target hunks do not overlap current user edits; activation rechecks this fact.
- Test-first minimum: assert live `150/200`, zero/reversed/non-representable rejection, every derived stat, and unchanged
  reward/wave/rounding behavior before RON mutation.
- Stop: target hunk begins overlapping an unowned change, loader fails current baseline, or implementation needs an
  arbitrary gameplay cap rather than representability validation.

### `GRB-WP-001` — `ready_after_wp002_completion_audit`

- Closed inputs: PD001/003/004/005/008/010 and exact 29-row/WhiteNight bootstrap.
- Must start only after WP002 completes because shared run-policy files are serialized, even though semantic graph
  dependency is absent.
- One atomic cutover includes Core/data/V4/store/server producer mapping; no research or dual authority remains.
- Target test modules cover every EL/MON/NB/BO/AT/TX/CP and MIG-01~03 row before mutation.
- Unity was not verified here; package activation must explicitly record its own consumer scope.
- Stop: any 29-row reference becomes missing, WhiteNight bootstrap cannot be non-vacuously reached, V4 needs an
  unapproved transform, package cannot reach a fully runnable atomic state, or current dirty ownership cannot be
  merged safely.

### `GRB-WP-003` — `ready_after_wp001_completion_audit`

- Closed inputs: PD002/006, exact `ST1-NT60`, explicit new pool IDs and V1 preservation defaults.
- Three new reward pools and actor-only F3 tuning are implementation-owned bootstrap authoring, not open content.
- Must coimplement fixed Standard, successor profile/store, Endless gate, Core/server projection and the crash-safe
  ST-02 publication contract.
- Stop: WP001 all-FinalBoss event is unavailable, any reused ST1 ID disappears, F3 actor-only isolation cannot be
  expressed without affecting other encounters, or completion/profile cannot be recovered atomically.

### `GRB-WP-004` — `ready_after_wp001`, serialized after WP003

- Closed inputs: mastered/source ledger, PD010 exclusive branches and existing generic active-consumable duration.
- No dedicated sweep consumable, cost or final content identity is required or approved.
- Must coimplement typed command/capability, battle-independent planner/result, Core/server consumer boundary and all
  SW/AT failpoint tests.
- Stop: implementation requires a special item/cost or new gameplay duration meaning, invokes fake battle state,
  permits partial producer-consumer publication, or cannot prove absence of condition/log/record/discovery effects.

### `GRB-WP-005` — `future_or_dormant`

No current activation. A separate user request, per-skill design/assets and its own readiness/consumer evidence are
required. The current campaign succeeds while all 29 placeholders remain valid.

## Overall Preflight Handoff

- ATI-01~04 rewind: not required.
- First package: `GRB-WP-002` is `ready_to_activate`.
- Secondary handoff: `ready_for_wp002`.
- Production mutation: still denied until the user separately activates an implementation Goal.
