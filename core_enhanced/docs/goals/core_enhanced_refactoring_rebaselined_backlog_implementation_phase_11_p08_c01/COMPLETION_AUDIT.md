# Phase 11 CRA-P08-C01 Completion Audit

Audit Mode: phase_exit
Audit Target: Phase 11 `CRA-P08-C01` typed item-trigger occurrence dispatcher
Audit Date: 2026-08-10 (`Asia/Seoul`)
Current Status At Audit Start: `active`
Implementation Complete Claim At Audit Start: `false`
Policy Source: `CRA-POL-002`, cross-cutting `CRA-POL-006`
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Repository/Commit/Dirty State: `/mnt/f/work/simulator`, HEAD
`f318414fb0156a67e04c5479000c2a2683fc0c09`, audit-start expanded status 426 entries with normalized
SHA-256 `d0cbdad7950fa556dc1c0e647089bd61740852956b34bf5f4a392252ae3821b3`

The full 567-line `docs/goal_completion_review_guide.md` was read before the initial exit review. The review found
two test-evidence gaps, correction was authorized by the active implementation Goal, and each correction was kept
separate from the audit. After the last correction, all 567 lines were read again and this independent pass was
restarted from the current tree. Child claims and E03 results were used only as evidence pointers; runtime, live RON,
canonical, scoped dirty state and tests were independently rechecked.

## Completion Evidence Matrix

| ID | 완료 조건 또는 불변식 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P11-C01 | 여섯 occurrence의 entry→plan→command/state 경로가 완전하다 | Child objective/in-scope | `sim.rs::run_on_battle_start_hooks`, instant `damage_runtime.rs`, `basic_attack_projectile.rs`, `death_runtime.rs`의 OnDeath/OnKill/OnAllyDeath가 모두 `dispatch_item_trigger_occurrence`를 호출한다 | current caller/materializer search와 Core 966/966 | pass |
| P11-C02 | 기존 dirty 변경과 범위 밖 파일을 보존한다 | preserved invariant 10 | E01의 422-entry 초기 baseline과 prospective hashes; current scoped status/fingerprints; live RON hash는 E01과 동일 | `git diff --check`; 426-entry audit-start status reconciliation | pass |
| P11-C03 | live OnBattleStart effect 누락을 재현하고 effect-before-activation을 구현한다 | Child completion condition | sorted unit dispatch가 effect+activation plan을 실행하고 live `parasite_seed` RON은 unchanged `AllAllies Attack +6%`를 공급한다 | E02 red 1/1; current live item 5/5; release battle-start 3/3 | pass |
| P11-C04 | OnHit Counterpart BonusDamage는 소유자 귀속 1회 비재귀 반격이다 | CRA-POL-002 | `is_current_hit_effect`가 해당 조합을 원 hit에서 제외하고 `retaliation_command`가 committed snapshot component를 owner snapshot `ApplyDamage`로 전달한다; leaf는 OnHit 입력이 없다 | commands 36/36; release retaliation 3/3; instant/projectile/multiple/dead/missing/fatal assertions | pass |
| P11-C05 | 하나의 typed dispatcher가 effect와 activation assembly를 소유한다 | Child target authority | private `TriggerOccurrenceContext`, `TriggerOccurrencePlan`, one-pass sorted `collect_item_trigger_payload`; effect materializer와 activation materializer의 production caller는 dispatcher뿐이다 | static caller search; trigger payload 1/1 | pass |
| P11-C06 | target, dead participant, proc identity, order와 released lifecycle가 정확하다 | CRA-POL-002; invariants 1, 4~7 | target resolver는 active counterpart/allies/enemies를 UUID sort하고, death runtime만 OnDeath dead caster를 허용한다; fatal-hit plan은 사망 전에 release된다 | AllAllies live, AllEnemies release 1/1, typed five-case table, sorted death test, fatal retaliation test | pass |
| P11-C07 | focused와 영향 범위 broad 검증이 모두 통과한다 | Child validation plan | current dirty tree의 Core/runtime/live data와 server consumer를 실행 | commands 36, trigger 1, live item 5, RON 25, skills 4+13+4, Core 966, server 59; 모두 fail/ignore 0 | pass |
| P11-C08 | canonical과 compatibility가 현재 runtime과 일치하고 later seam을 침범하지 않는다 | CRA-POL-006; Child out-of-scope | `game_rulebook.md`의 장비 전투 트리거 절; RON hash unchanged; 새 context/plan은 core-private, arithmetic helper는 crate-private, serde/save/wire carrier 변경 없음 | RON 25/25, server 59/59, public/serde diff search, check/fmt/diff checks | pass |
| P11-C09 | phase-exit review가 `Complete / High`다 | completion review guide | 이 독립 감사의 runtime/data/consumer/canonical/legacy/dirty/fit 검토 | 아래 verdict와 transition | pass |

## Runtime And State Evidence

- `collect_item_trigger_payload`는 소유 runtime item을 instance UUID로 한 번 정렬한 뒤 같은 순회에서 effect와
  activation을 수집한다. `TriggerOccurrencePlan::into_commands`는 한 occurrence의 모든 일반 effect 뒤에
  activation을 배치한다.
- OnBattleStart는 runtime unit UUID 순으로 plan을 만들고 두 payload class를 모두 처리한다. Instant와
  projectile basic hit는 OnAttack effect, OnHit effect, activation 순서를 보존하면서 동일 dispatcher를 쓴다.
- OnAttack의 released damage inputs는 source snapshot에 남는다. OnHit current-hit inputs는 plan의 명시
  `current_hit_effects`로 formula에 들어간다. 일반 skill, buff tick, area `ApplyDamage` leaf는 새 item occurrence를
  수집하지 않는다.
- Death finalization이 OnDeath→OnKill→sorted OnAllyDeath 발생 시점을 계속 소유한다. OnDeath activation만
  `allow_dead_caster = true`이며 effect target materialization은 active-target 규칙을 유지한다.
- CRA-POL-002 경로는 Counterpart BonusDamage를 원 hit `on_hit_effects`에서 제외한다. 각 authored effect의
  raw component를 triggering source snapshot과 기존 `bonus_damage_component` arithmetic으로 한 번 계산하고,
  effect owner를 source ID로 가진 `ApplyDamage`를 attacker에게 발행한다.
- Retaliation snapshot은 `include_on_attack_damage_effects = false`이고 `ApplyDamage` leaf의 OnHit slice도
  비어 있어 reflection recursion이 없다. 공격자가 없거나 inactive면 command를 만들지 않는다.
- 치명 원 hit도 retaliation plan을 먼저 release하므로 defender death 후 owner-attributed damage가 실행된다.
  이 경계는 private-shape assertion이 아니라 exact `HpChanged` order/source와 lifecycle로 검증됐다.
- Authoritative mutation은 기존 `process_commands`의 modifier/heal/buff/damage와
  `apply_damage_result_and_record`/death finalization에 남아 있다. Dispatcher가 HP, formula, ability delivery 또는
  death authority를 복제하지 않는다.

## Data Evidence

- Production `game_resources/data/equipments/base.ron`은 E01 fingerprint
  `322d7455a256082f03804c0dcc7a77a4caa3a06919d5e34b667042b411bcd1a6`과 현재 fingerprint가 같다.
- `parasite_seed`는 실제 RON에서 `OnBattleStart / AllAllies / Attack Percent 6`을 author한다. Current live test는
  loader와 BattleCore를 통해 두 active allies의 exact 10→10, 50→53 변화, UUID order와 enemy exclusion을
  확인한다.
- `thorned_lantern_guard`는 실제 RON에서 `OnHit / CounterpartUnit / Physical BonusDamage flat 8`을 author한다.
  Schema, ID와 balance value는 바뀌지 않았다.
- Full live RON 25/25가 현재 bundle load/validation을 통과했다. Candidate-specific publication matrix 강화는
  예정된 Phase 13 P08-C04 소유이며 이번 runtime correctness의 미완료 항목이 아니다.

## DTO/Transport/Persistence Evidence

- 새 occurrence context/plan은 `pub(in crate::game::battle::core)`이고 serde를 구현하지 않는다. 새 damage
  helper도 `pub(crate)`이며 public Rust/DTO field가 아니다.
- Existing `BattleCommand`, `BattleLogEvent`, save/checkpoint version, request/result tag와 server mapping shape는
  바뀌지 않았다. Retaliation은 기존 `ApplyDamage`와 `HpChanged` carrier를 사용한다.
- Runtime item occurrence는 battle-lifetime ephemeral state이므로 checkpoint reader/writer나 migration을
  요구하지 않는다. Dual read/write, alias, version fallback을 추가하지 않았다.

## Consumer Evidence

- confirmed workspace consumer: BattleCore instant/projectile/death/start callers, usefulness preview, event log와
  `game_server`; current server all-target run은 main binary 59/59를 통과했다.
- canonical external consumer: Unity는 기존 event/wire carrier를 소비하지만 이번 Phase는 shape를 바꾸지
  않는다. CRA-POL-006과 Child PLAN은 transport rollout 및 Unity 실행을 명시적으로 비범위로 둔다.
- inaccessible/unknown consumer: 새 public contract가 없으므로 완료를 막는 미확인 필수 consumer가 없다.
- explicitly excluded consumer: committed-damage consolidation(P08-C05), publication validation(P08-C04), false
  death cleanup(P08-C06)의 future consumers/seams.

## Legacy/Compatibility Audit

- Item `TriggerSource`, `collect_trigger_activations`, `collect_all_trigger_activations`와 구 `collect_triggers(`
  production reference는 0이다. Visual-novel의 별도 `VisualNovelTriggerSource`는 동명이 아닌 다른 domain이다.
- Effect/activation command materializer의 production call은 typed dispatcher에만 있다. Activation helper의
  나머지 직접 call은 proc identity를 보호하는 unit test다.
- Affected source/test에서 ignored test나 debug-only bypass는 없다. `TriggeredEffect::legacy` 한 건은 pre-existing
  test fixture constructor이며 승인되지 않은 runtime fallback이나 이 Candidate의 old dispatcher가 아니다.
- No compatibility shim, dual occurrence assembly, placeholder DTO 또는 future-phase public carrier가 추가되지
  않았다.

## Scoped Diff And Dirty Preservation

- E01에서 Phase init 직후 422 entries와 normalized hash
  `3110fee7867552d896ca59d5e4533142dc77506f466080b1b0f02ea6c1c62b9f`를 기록했다. Phase production 전에
  prospective file fingerprints와 pre-existing commands/sim/types/death/canonical/RON hunks를 분리했다.
- Audit start는 426 entries, normalized hash
  `d0cbdad7950fa556dc1c0e647089bd61740852956b34bf5f4a392252ae3821b3`다. RON hash는 baseline과 같고 기존
  field-work/tactical/control/attack-epoch 변경은 유지된다.
- Final relevant fingerprints include `triggers.rs`
  `ae9d377dfbc988ad36b96ac44aba452b70a53175c28ff9b36d6948357a1e4ea8`, `damage.rs`
  `e92d03779bb460e370326559244cd95fd842da78050ec92493005169e5497076`, `commands.rs`
  `b21cc1419e51fa57daac0974039bab0fe6275b97cfb4fb769487fd1ac2779329`, live test
  `0d85e737e5938722321bfe55a178c479be910239a7dd87ecc8f151d2710ec5a2`, canonical
  `e1657fb76bd76d6ab831c7e511330399329972a1d7c8daafc7f634a4e3168971`.
- No reset, restore, clean, stash, unrelated deletion or broad formatting was used. `git diff --check` is clean.
- Audit-artifact-inclusive ending status is 427 entries with normalized SHA-256
  `62d865f48b214a863964a090a515f0161404dbad90803362330d7e6b7d267ca7`.

## Canonical Alignment

- `docs/game_rulebook.md` now describes all six equipment occurrences, item/unit UUID ordering,
  effect-before-activation, live parasite behavior, basic-hit-only OnAttack/OnHit scope, OnDeath dead-caster rule and
  CRA-POL-002 retaliation/dead-attacker/no-recursion meaning.
- Runtime and canonical agree at target, source attribution, timing/order and recursion level. The canonical does not
  claim Phase 13 validation or Phase 28 consolidation is already implemented.
- `stats.rs` target documentation continues to define OnHit counterpart as attacker and AllAllies/AllEnemies as all
  active same/opposite-side units, matching current target resolution and direct tests.

## Validation Results

All commands below ran on the current dirty workspace unless explicitly identified as historical red evidence.

- `cargo test -p game_core --lib game::battle::core::commands::tests:: --quiet`: 36 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --lib game::battle::core::triggers::tests:: --quiet`: 1 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --test live_item_skill_activation --quiet`: 5 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --test ron_loading --quiet`: 25 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --test live_skill_catalog_audit --quiet`: 4 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --test skill_runtime_contract --quiet`: 13 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --test skill_test_suite --quiet`: 4 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --lib --quiet`: 966 passed, 0 failed, 0 ignored.
- `cargo test --all-targets --quiet` in `game_server`: main target 59 passed, 0 failed, 0 ignored; four additional
  binaries each ran 0 tests and passed. The zero-test binaries are reported but are not behavioral proof.
- `cargo test -p game_core --release --lib retaliation --quiet`: 3 passed, 0 failed, 0 ignored.
- release exact `item_occurrence_all_enemies_targets_are_complete_and_sorted`: 1 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --release --test live_item_skill_activation battle_start --quiet`: 3 passed, 0 failed,
  0 ignored, 2 filtered out.
- `cargo check -p game_core --all-targets`: passed.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.

## Baseline Failures And Audit Corrections

- E02 live parasite first proof selected exactly 1 test and failed with no expected StatChanged entries. The same
  test is included in current live 5/5 and release battle-start 3/3 green.
- E02 retaliation first proof selected exactly 1 test and failed because defender HP was 82 instead of 90, proving
  old incoming-hit amplification. Current instant/projectile/release proofs are green.
- One historical exact short-name filter selected 0 tests and was discarded; it is not completion evidence.
- Audit finding A11-01: AllAllies had live evidence but AllEnemies had only implementation evidence. Added the
  deterministic two-enemy/excluded-ally test. Its first compile attempt used nonexistent `ModifyStats` and failed
  E0599; the test was corrected to the actual `ApplyModifier` carrier and then passed debug/release. No production
  code changed. The audit was restarted.
- Audit finding A11-02: dead/missing attacker was covered, but released retaliation after the effect owner dies from
  the triggering hit lacked direct proof. Added exact fatal-hit lifecycle/source/order evidence; it passed 1/1 and
  in release retaliation 3/3. No production code changed. The full 567-line guide was reread and this audit restarted.
- No unresolved baseline failure or audit correction remains.

## Cross-Component Findings

- Instant and projectile now share occurrence policy while retaining their existing delivery owners; exact parity is
  demonstrated for retaliation. No competing item-trigger authority remains in death or battle start.
- Read-only basic-attack usefulness uses the same current-hit classification, so it no longer predicts Counterpart
  BonusDamage as incoming amplification.
- Phase 12 P08-C02 can consume the unchanged HP/death authority. Phase 13 P08-C04 can validate the now-executable
  target/effect matrix. Phase 28 P08-C05 may consolidate committed damage without remigrating occurrence policy.
- No material cross-component conflict, temporary shim or hidden source-of-truth split was found.

## Long-Term Direction Review

| Candidate ID | 작업 유형 | 현재→목표 authority 결과 | 보존 불변식 결과 | 제거·통합 결과 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P08-C01 | correctness correction | split start/hit/death assembly → one core-private typed occurrence dispatcher | deterministic item/unit order, proc identity, current-hit snapshot, dead/released lifecycle and public shapes preserved | old activation collectors and one-variant item TriggerSource removed; formula/delivery/death owners retained | current focused/release/live/Core/server/static suite | pass |

- Fit: High
- Improvement class: none
- Reason: occurrence ownership is narrower and singular, context is typed and core-private, no parallel public model or
  duplicated state was introduced, and tests protect live/event/lifecycle behavior rather than only helper shape.
  Explicitly scheduled P08-C04/C05/C06 work is independent later-Phase scope, not a Phase 11 fit defect.

## Primary Verdict

- Complete
- Long-term fit: High
- Combined phase-exit verdict: `Complete / High`

## Status Transition

- from: Child `active`, `Implementation Complete: false`; Master Phase 11 `active`; Phase 12 `pending`
- to: Child `complete`, `Implementation Complete: true`; Master Phase 11 `complete — Complete / High`; Phase 12
  `ready`
- next Phase activation: allowed, only after this audit is fully read and Phase 12 initialization gates run

## Remaining Risk And Required Action

- Unresolved material finding: none.
- Immediate correction: none.
- Policy question: none.
- Required next action: close Phase 11 documentation/registry, fully read this audit, then initialize only Phase 12
  CRA-P08-C02. Do not absorb P08-C04 publication validation, P08-C05 damage consolidation or P08-C06 cleanup into
  this completed Phase.
