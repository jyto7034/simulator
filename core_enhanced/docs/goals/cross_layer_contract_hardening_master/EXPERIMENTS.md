# Cross-Layer Contract Hardening Master Experiments

Status: active orchestration log

Implementation complete: false

Created: 2026-07-11

This file records master-level evidence, decomposition trials, baseline results, child review outcomes, and final cross-goal validation. Child implementation trials belong in each child goal's EXPERIMENTS.md.

## Experiment Log

| Date | Hypothesis or question | Action | Result | Decision |
| --- | --- | --- | --- | --- |
| 2026-07-10 | A broad project review may reveal runtime and cross-layer defects not visible in Core unit tests. | Reviewed Core runtime, live RON, game_server, Unity DTO consumers, tests, and durable documents. | A formal 15-finding audit was created. Core tests passed while multiple cross-layer defects remained. | Preserve the formal audit as historical evidence, not canonical policy. |
| 2026-07-10 | The temporary documentation-code audit may contain broader drift than the formal review. | Created a temporary 52-finding audit across runtime, data, server, Unity, tests, and documents. | The note found useful seams but mixed implementation defects, policy conflicts, document drift, and speculative risks under inflated severities. | Revalidate every finding before using it as goal input. |
| 2026-07-11 | The 52 temporary findings still match current code and live data. | Re-read every finding against runtime, live RON, typed DTO, server, Unity consumer, tests, and current documents. | Most underlying seams remained, but many High findings were downgraded. DCA-C-01 was broader than written. DCA-M-21 was more severe than written. DCA-M-26 was not actionable. | Use source-prefixed, revalidated findings rather than copying the temporary severity table. |
| 2026-07-11 | One executable goal could resolve all findings coherently. | Compared a single-goal structure with seam-based child decomposition. | A single goal mixes policy questions, repositories, test gates, and incompatible completion criteria. It would create false completion and high conflict risk. | Master owns orchestration only; child goals own implementation. |
| 2026-07-11 | A small set of broad child goals would reduce document overhead. | Trialed broad groupings around Unity, combat, data, and documentation. | Several groups still mixed independent authorities, such as battle revision versus capability, abnormality range balance versus skill targeting, and BossOmen rollback versus whole-stage `RetreatBattle` settlement. | Adopt bounded child candidates. Create child documents only when work begins. |
| 2026-07-11 | DCA and CPR finding IDs can be combined without prefixes. | Compared both audit identifier sets. | The reports reuse C/H/M identifiers for different findings. Nine CPR findings overlap DCA; six CPR findings are unique. | Use DCA-* and CPR-* source prefixes and an explicit alias table. |
| 2026-07-11 | Existing master-goal conventions can be reused without change. | Reviewed current master PLAN patterns. | Dependency graphs, startup protocols, and review gates are useful. Treating policy-decision reports as implementation-complete is not. Some older masters also link deleted workflow documents. | Use active, policy_decision_required, externally_blocked, and complete distinctly. Use current engineering_workflow.md paths only. |
| 2026-07-11 | The initial registry and child catalog cover every source finding exactly once. | Counted DCA rows, CPR source IDs, child owners, headings, links, and whitespace diagnostics. | DCA coverage was 52/52 and CPR coverage was 15/15. The first pass exposed conflicting DCA-M-10 ownership between G12 and G13; the catalog was corrected so G13 is the sole primary owner. Relative links and whitespace checks reported no errors. | Keep mechanical coverage validation as a master update gate. |
| 2026-07-11 | Which policy decisions require user interruption? | User delegated simple structural decisions to the documented recommendation and requested questions only for difficult UX, balance, content, failure, reward, and persistence semantics. | PD-01 was accepted as an independent monotonic update revision. G01 changed from not_started to ready. | Continue recording simple recommendations directly; ask difficult decisions one at a time. |
| 2026-07-11 | Can battle-stage abandonment (`RetreatBattle`) rewind casualties already incurred? | Presented persistent-outcome, full-rewind, and hybrid choices. | User stated that death has priority, but subsequent runtime review showed that “death” has two distinct meanings in the current model. | Keep G15 policy_decision_required until tactical `UnitDied` and persistent `EmployeeLifeState::Dead` are disambiguated. |
| 2026-07-11 | Does Battle HP 0 necessarily mean persistent employee death? | Traced runtime-unit death, redeployment, participant result, result settlement, live run policy, and employee life-state code. Ran `cargo test post_battle_resolution_applies_employee_incapacitation_trauma --lib`. | No. Battle HP 0 marks the runtime instance Dead and incapacitated, but the same employee may redeploy after 90 seconds at 60% Battle HP. At result settlement, live policy applies trauma +40, Run HP -25% of max, and an injury; persistent death occurs only when Run HP reaches 0 or trauma reaches 100, subject to death prevention. The focused test passed 1/1 and explicitly kept the employee Alive after one incapacitation. | Replace the misleading “post-calculation” wording with deterministic result settlement and ask whether the user wants to preserve or replace the two-layer life model. |
| 2026-07-11 | Is fixed 60% death redeploy HP the intended policy? | Re-read the historical redeploy policy report, current rulebook, roster-to-draft stat pipeline, death lock, deployment override, and post-battle resolution after the user clarified the Arknights-like design. | No. The historical report itself says the earlier policy used updated trauma, then rejected live trauma application and substituted fixed 60% because it was simpler. Current code builds a trauma-adjusted draft but overwrites its current HP with `PercentOfMax(60)`, while trauma is delayed until `CompleteCombatResult`. The user confirmed that tactical death must apply trauma before redeployment and the next Battle HP must reflect that trauma. | Register MSTR-01, assign it to G26, and supersede the fixed-60% policy. |
| 2026-07-11 | Does battle-stage abandonment (`RetreatBattle`) preserve trauma incurred during the active battle attempt? | Asked after separating tactical death, updated-trauma redeployment, and persistent employee state. | No. The user confirmed that trauma increases immediately inside the active attempt and affects redeployment, but battle-stage abandonment restores every numeric value to its exact pre-entry value. Therefore the intended model is a battle-attempt transaction, not an immediate irreversible roster mutation. | Mark CPR-H-09 policy-accepted/no-change and require G26 to own an attempt-local employee-condition working state with commit/discard semantics. |
| 2026-07-11 | Is persistent death or any other gameplay state exempt from battle-stage-abandonment rollback? | Asked whether a trauma-threshold `EmployeeLifeState::Dead` transition survives whole-stage `RetreatBattle` because the user had previously said death takes priority. | No exemption. The user defined whole-stage `RetreatBattle` as a quick-load that completely restores the authoritative state from immediately before battle entry. Persistent death, attempt consumption, consumable/resource mutations, RNG progression, node progress, and post-entry gameplay records therefore roll back with the attempt. | Treat battle-stage abandonment as snapshot restore, remove attempt-consuming/exhausted-abandonment behavior, and keep debug telemetry outside authoritative gameplay state. |
| 2026-07-11 | Was the word retreat referring to unit withdrawal or whole-stage abandonment? | The user corrected the ambiguous Korean wording. | The accepted exact rollback applies only to whole-stage `RetreatBattle`, described as `전투 포기`. `WithdrawUnit` remains an in-battle unit withdrawal with cooldown/redeployment and never restores the battle-entry snapshot. | Use four distinct terms everywhere: unit withdrawal, tactical defeat, battle-stage abandonment, and battle defeat. |
| 2026-07-11 | Are Run HP loss and injury applied once per employee per battle or once per tactical death? | Re-asked after separating unit withdrawal, tactical defeat, battle-stage abandonment, and battle defeat. | The user confirmed that deployment count is irrelevant: every tactical death applies the death penalty again. Victory/defeat commits the accumulated penalties; battle-stage abandonment restores the pre-entry state and discards them. | Mark PD-16 accepted and G26 ready. Keep replay instances, but do not mistake employee-level XP, summary, or MVP aggregation for per-death penalties. |
| 2026-07-11 | What does the earlier “death takes priority” decision mean after terminology is separated? | Reconciled it with the accepted whole-stage-abandonment rollback. | It governs committed non-abandonment results: if accumulated per-death penalties leave no living employees, RunFailed must preempt retry and cancel. It does not survive a whole-stage `RetreatBattle` quick-load. | Mark CPR-H-02 policy accepted and retain G15 implementation ownership. |
| 2026-07-11 | When does combat-consumable duration commit? | Asked separately from unit withdrawal and whole-stage abandonment. | The user accepted one decrement on any committed victory or defeat. Directly triggered consumption also commits. Whole-stage `RetreatBattle` restores duration and triggered consumption with the rest of the pre-entry snapshot. | Mark DCA-M-31 policy accepted. Use one coherent CombatResult for each committed result and mark G15 ready after G26. |
| 2026-07-11 | Should forced BossOmen victory advance the Floor directly or preserve Gate as the sole transition authority? | Rechecked the role of Gate, forced-boss insertion, transition reward, and player pacing rather than following the current shortcut. | The user accepted a dedicated post-boss exit Gate. Boss results and chain completion commit first; the Floor remains current until the player enters and confirms the newly opened Gate, which applies normal Gate trauma recovery. | Mark the victory half of PD-10 and DCA-H-14 accepted. Keep G14 policy_decision_required only for forced-boss defeat recovery. |
| 2026-07-11 | Is a BossOmen-chain boss the final boss of the entire run, and what happens after defeat? | Clarified terminology before deciding recovery. | It is the capstone of that BossOmen chain, not an Endless-run final boss. The user accepted convenience recovery: committed defeat enters RunFailed, but if a SavePoint checkpoint and load count remain, the player may explicitly spend one load to restore it; otherwise the run ends. | Mark PD-10 fully accepted, DCA-H-15 policy accepted, and G14 ready. |
| 2026-07-11 | Should explicit basic-attack ranges be authored before detailed abnormality settings exist? | Presented explicit reusable presets plus custom boss patterns as the eventual direction. | The user deferred the policy because detailed abnormality settings are not finalized. Premature values would turn placeholders into accidental balance policy. | Mark G05 deferred_by_user. Temporarily retain the current fallback with an explicit removal trigger; do not modify live RON until content design is approved. |

## Master Document Creation Verification

Commands and results:

~~~text
wc -l PLAN.md EXPERIMENTS.md EXPERIMENT_NOTES.md
  initial creation: 1153 total lines

DCA coverage extraction and unique count
  52 rows
  every DCA ID appeared exactly once

CPR alias and unique extraction
  15 source IDs
  every CPR ID appeared exactly once

relative Markdown link existence check
  no missing targets

git diff --no-index --check for each new file
  no whitespace diagnostics
  exit status 1 is expected because each file differs from /dev/null

git status --short -- docs/goals/cross_layer_contract_hardening_master
  one new untracked goal directory
~~~

The validation did not run gameplay tests again because this change only creates planning documents. The current gameplay baseline is recorded below.

## Revalidation Baseline

### Passed

~~~text
cd /mnt/f/work/simulator
cargo fmt --all -- --check

cd /mnt/f/work/simulator/core_enhanced
cargo test --all-targets
  681 passed
  0 failed

cd /mnt/f/work/simulator
cargo check -p game_server
~~~

### Failed Baseline

~~~text
cd /mnt/f/work/simulator/game_server
cargo test --all-targets
  19 passed
  6 failed
~~~

All six server failures report the same root cause before their intended assertions:

~~~text
Invalid static data: missing RunStart visual novel trigger for Standard
~~~

This failure belongs to G04 and must not be attributed to later child changes unless the failure signature changes.

### Not Run

- Unity EditMode runner.
- Unity PlayMode runner.
- Full Core-server-Unity live session.
- Same-seq/same-time side-band reproduction in an actual Unity player.
- Blocking battle Visual Novel plus reconnect combination.
- Runtime identity ordering experiments for DCA-M-02 and DCA-M-22.

## Key Revalidation Corrections

### DCA-C-01

Original framing: battle Visual Novel update only.

Current evidence: pause, resume, speed, and battle Visual Novel projection can all change checkpoint or side-band state without increasing battle event sequence or time. Unity can return before applying the changed update.

Decision: G01 must solve the general side-band revision problem, not special-case Visual Novel updates.

### DCA-M-21

Original framing: future authoring risk; current live ranges believed explicit.

Current evidence:

- 30 live AbnormalityMetadata entries.
- 30 use defense_tile_range: None.
- 30 use range_role: Melee.
- Pattern validation permits missing data.
- Runtime supplies a role-based fallback.

Initial decision: promote to current-live High and require user-approved authored range values in G05.

User decision on 2026-07-11: defer G05 because detailed abnormality combat settings are not yet finalized. The current fallback is an explicitly temporary placeholder, not canonical balance. Do not fabricate values; restart only when content design exists, then remove the fallback.

### DCA-H-07

Original framing: High HUD contract conflict.

Current evidence: Unity implementation, external canonical, and PlayMode tests agree on screen-space overlay from world anchors. Only the Core document billboard wording is stale.

Decision: downgrade to Low documentation cleanup in G13.

### DCA-M-25

Original framing: no usable probe surface exists.

Current evidence: the simulator tree has no Python probes, but the external Unity project contains:

- unity_ws_smoke_probe.py,
- unity_savepoint_checkpoint_probe.py,
- unity_admin_debug_command_probe.py.

Decision: G25 fixes discoverability and workflow paths rather than inventing a duplicate local probe.

### DCA-M-26

Original framing: crafting-disabled wording conflicts with automatic EquipItem combination.

Current evidence: the rulebook specifically says there is no separate live Maintenance crafting action. Automatic combination during EquipItem is a distinct behavior and does not contradict that sentence.

Decision: excluded as not actionable. No child directory.

### DCA-L-04

Original framing incorrectly implied incapacitation normally means Dead plus Unavailable.

Current evidence: persistent incapacitation effects can leave an employee Alive unless trauma or Run HP reaches the death threshold.

Decision: G15 must first define terminology and gameplay meaning. Do not implement from the old rationale.

## Audit Source De-duplication

CPR overlaps mapped to DCA:

- CPR-C-01 → DCA-C-01
- CPR-H-04 → DCA-H-05 and DCA-H-06
- CPR-H-05 → DCA-H-09
- CPR-H-06 → DCA-H-10
- CPR-H-07 → DCA-H-13
- CPR-M-01 → DCA-H-08
- CPR-M-03 → DCA-M-01
- CPR-M-04 → DCA-H-11 and DCA-M-13
- CPR-M-05 → DCA-H-01

Unique CPR ownership:

- CPR-H-01 → G26
- CPR-H-02 → G15
- CPR-H-03 → G16
- CPR-H-08 → G12
- CPR-H-09 → G15
- CPR-M-02 → G12

## Child Review Log

No child has started.

## 2026-07-11 — Inventory stacking and capacity revalidation

Hypothesis:

The inventory-full reward finding might be mitigated by an existing general rule that identical items share one stack.

Action:

Re-read the runtime inventory structures, Unity-facing inventory DTOs and snapshots, reward grant path, skill-fragment inventory, and current rulebook wording.

Observed result:

- No general identical-item stack contract exists in runtime.
- Equipment uses one owned UUID and one capacity slot per copy; its enhancement and equip state are instance-specific.
- Consumables also use one owned UUID and one capacity slot per copy. Their DTO has no quantity field.
- Artifacts are unique, capacity-limited, and reject duplicate metadata UUIDs.
- Equipment materials stack by material id and skill fragments stack by fragment id; neither uses the ordinary equipment/consumable/artifact slot model.
- `grant_inventory_item` allocates a fresh owned UUID for each equipment or consumable reward and returns `InventoryFull` when the corresponding category has no free slot.
- `docs/game_rulebook.md` says consumable use decreases a duration or stack, but this is not backed by the current owned-consumable structure or DTO and does not establish a general inventory rule.

Decision and next step:

The user accepted an infinite ordinary inventory on 2026-07-11. Replace fixed category admission caps with a persisted logical `reserved_slot_count` that starts at 50 and grows atomically in 10-slot increments only when a successful addition creates a new logical slot beyond the current high-water boundary. It never shrinks; stack quantity changes do not grow it. The reserve is Unity-facing presentation metadata over one authoritative inventory, not Rust collection capacity or overflow storage. Identical consumables become definition-id quantity stacks; equipment remains per-instance; materials, fragments, and currencies remain separate non-slot resources. Artifact participation was subsequently reopened because the user identified that subsystem as legacy; G24 must consume the boundary established by G27 rather than preserve Artifact as an ordinary slot.

## 2026-07-11 — Artifact rulebook identity and live-runtime audit

Hypothesis:

The current Artifact uniqueness and duplicate-acquisition rules might be worth preserving as part of the accepted infinite-inventory work.

Action:

Read all 779 lines of `docs/game_rulebook.md`, then trace live Artifact RON, shop reachability, inventory ownership, Core snapshots, Unity inventory reading, scenario construction, stat and trigger application, reward/admin paths, checkpoint behavior, and focused tests.

Observed result:

- Artifact is never defined as a canonical system in the rulebook. The named build axes are employees, equipment, skill fragments, consumables, research, rewards, and long-term loss management.
- Ten live Artifacts are purchasable through live shop pools, so this is not dormant data.
- Every owned Artifact is automatically inserted into every player battle scenario. Permanent effects modify every allied unit and side-global trigger effects participate in battle.
- The current schema reuses equipment-like `triggered_effects` and `ability_activations`; live content is almost entirely direct team stats, healing, or combat triggers.
- Normal Unity inventory ignores the Core `inventory.artifacts` list, so players can buy a permanent global modifier without a proper ownership/effect surface.
- The system overlaps equipment and accessories rather than adding a distinct decision layer. Removing it affects live shops, loader/registry, inventory/DTOs, reward/admin paths, scenario/runtime artifacts, stat/trigger pipelines, event logs, validation, server/Unity contracts, and tests.

Decision and next step:

The earlier duplicate-artifact conversion recommendation is withdrawn. Track the issue as MSTR-02/G27. The assistant then proposed `ContainmentRelic`/격리 유물 as a possible replacement, but the user correctly noted that this term and setting had never existed. Treat that proposal as an unaccepted design sketch, not policy. The current recommendation is to remove the legacy Artifact subsystem without replacement and defer any future run-wide rule-modifier system until its setting and first real content designs are approved. Do not migrate the current effects or schema.

User decision on 2026-07-11: remove Artifact completely without replacement. No current Artifact name, effect, price, identifier, data shape, record variant, or ownership behavior is a migration requirement. The only remaining content decision is how to replace live shop stock: the Artifact merchant and Headquarters basic-supply shop are Artifact-only, mixed late shops contain Artifact entries, and the existing live consumables otherwise have no live acquisition source.

## 2026-07-11 — Shop runtime, live-data, rulebook, and Unity audit

Hypothesis:

Artifact removal might only require replacing the affected stock entries while retaining the current Shop system.

Action:

Traced Shop RON deserialization, pool resolution, active session state, map and Headquarters entry, purchase, sale, reroll, exit, allowed-action projection, typed snapshots and command results, live map/shop/equipment/consumable data, canonical rulebook and UI policy, game_server post-command snapshots, and the actual Unity reader, composer, selection state, mocks, and tests. Ran `cargo test shop --lib`.

Changed files:

- Master planning documents only; no runtime, live RON, server, or Unity implementation changed.

Commands and actual test counts:

- `cargo test shop --lib`: 15 passed, 0 failed, 0 ignored, 625 filtered out.
- Additional delegated focused Shop data/session checks: 17 passed across Shop data, node/session, and hidden-stock snapshot coverage.

Observed result:

- The structural shell works: a map or Headquarters choice resolves a Shop, clones it into an active session, enters `InShop`, accepts purchase/sale/reroll/exit commands, and exit completes the source node.
- Live merchant selection is deterministic and uniform at the whole-Shop level. Each merchant itself is a fixed authored UUID page; there is no floor/risk-weighted offer generation or listing quantity.
- `DiscountShop` is an inert enum/DTO label. Purchase always charges the item's base price.
- `can_reroll: false` shops may still author hidden stock. Ain has four and Gebura three permanently unreachable items.
- Reroll is one free `visible_items`/`hidden_items` swap, not a same-size redraw. Live page sizes change Rachel 1→9, Benjamin 1→4, Netzach 3→5, Yesod 0→3, and Binah 2→6.
- Yesod opens with an empty page; Yesod and Binah use placeholder equipment, including zero-price definitions. Artifact-only and Artifact-mixed merchants occupy the normal and Headquarters pools.
- The rulebook defines Shop only as a one-use Safe Node. It defines no merchandise scope, final-price rule, sale rule, refresh rule, discount meaning, merchant progression, or Headquarters-Shop relationship.
- Core sends visible item metadata, but Unity reads only `visible_item_uuids` and renders UUID labels. It drops name, price, rarity, description, effect, shop identity, and purchase/sale previews.
- `RerollShop` is advertised for every `InShop`; Unity also always renders it. Runtime rejects it for non-reroll shops and after the first swap.
- Unity lists only unequipped equipment for sale, despite Core also accepting consumables and rejecting bound equipment. It shows no authoritative sell price or disabled reason.
- Unity selection is not reconciled after purchase, sale, reroll, exit, or entry into another Shop, so stale UUIDs can remain actionable.
- The canonical Unity contract is stale about exposing hidden stock; current Core intentionally keeps hidden stock server-side and tests that behavior.
- Seven live consumables have prices and effects but no live acquisition reference, so a future Shop is a plausible acquisition route, not a reason to preserve the current merchant schema.

Failure cause:

The hypothesis treated Shop content and Shop mechanics as separable. They are not: Artifact-only stock exposes a broader prototype economy whose price, refresh, capability, offer DTO, Unity UX, and Headquarters semantics were never canonically designed.

Decision and next step:

Withdraw the narrow post-Artifact stock-replacement recommendation. Register MSTR-03 and assign a separate G28 `shop_economy_and_offer_contract`. G28 must settle PD-18 before G27 edits Artifact-dependent Shop data. Preserve only the proven node/session/command boundary by default; do not retain `DiscountShop`, fixed visible/hidden pages, free whole-page swapping, or UUID-only Unity presentation merely because focused tests currently pass.

## 2026-07-12 — Shop encounter identity policy

Question:

Should Headquarters Shop access be removed or treated as a separate system from an exploration Shop?

User decision:

- Keep one shared Shop system.
- Headquarters support can lead to a Headquarters-support Shop.
- Exploration can randomly produce an Unknown Shop encounter.
- They share transaction/session mechanics but use different merchandise profiles.
- The Unknown Shop offers higher-value items.

Correction:

Withdraw the recommendation to remove Headquarters Shop access. G28 must model encounter context as an offer-pool/profile input without duplicating Shop commands, DTOs, purchase/sale logic, or Unity screens. The exact meaning of “higher-value” remains a policy gate; do not infer that it includes abnormality-specific finished E.G.O, skill fragments, or any other progression-bypassing category.

## 2026-07-12 — Shop merchandise extensibility policy

Question:

Should the first G28 catalog decision become a permanent contract that excludes skill fragments or similar future merchandise from every Shop?

User decision:

- No. Keep future Shop merchandise categories open.
- The initial authored catalogs may omit skill fragments or other currently undecided goods.
- Do not create contract tests or validators asserting that such categories can never appear in Shop offers.

Implementation consequence:

G28 tests must protect current authored data validity, authoritative price/eligibility projection, purchase results, and encounter-profile behavior. They must not protect a closed merchandise-category allowlist. Prefer an extensible typed offer payload or grant representation over separate hard-coded purchase paths per current item category. Adding a future merchandise kind may require adding an explicit typed variant and behavior tests, but must not require deleting a test whose purpose was to prohibit that kind globally.

## 2026-07-12 — Shop refresh availability policy

Question:

Should refresh availability be fixed by Shop value tier, or authored by each Shop?

User decision:

- Refresh exists as an optional Shop capability.
- Author its availability in RON per Shop.
- The default content direction is refresh enabled for lower-value Shops and disabled for higher-value Shops.
- That direction is not a schema or runtime invariant: a higher-value Shop may enable refresh, and a lower-value Shop may disable it, when its authored situation requires it.

Implementation consequence:

Do not branch runtime behavior on a high-value/low-value enum to decide refresh. Replace the legacy `can_reroll` plus visible/hidden whole-page swap with an explicit optional refresh policy owned by Shop data and copied into the active session. Core projects current availability and disabled reason to Unity. Tests cover enabled, disabled, and authored-exception examples; they must not assert that all high-value Shops forbid refresh. Cost, use limit, redraw size, and purchased-offer reappearance remain separate policy questions.

## 2026-07-12 — Shop refresh cost and use-limit policy

User decision:

- An enabled refresh policy authors `base_cost` and `max_uses` per Shop in RON.
- `base_cost: 0` is valid and explicitly represents free refresh.
- Refresh availability depends on the authored policy, remaining uses, and sufficient Enkephalin.

Implementation consequence:

Copy the authored refresh policy into the active Shop session and persist its remaining-use state. Core, not Unity, computes and projects current cost, remaining uses, `can_refresh`, and the disabled reason. Refresh must deduct cost and mutate the offer page atomically; failure leaves currency, remaining uses, and offers unchanged. Do not infer cost or use count from Shop value tier. No escalating-cost rule is accepted at this stage: `base_cost` is the current per-use cost unless a later policy explicitly adds progression.

## 2026-07-12 — Shop refresh page-replacement policy

User decision:

- Each Shop authors a stable `offer_count` in RON.
- Initial entry generates exactly `offer_count` offers.
- Refresh discards the entire current page, including empty slots created by purchases.
- A successful refresh generates exactly `offer_count` new offers.

Implementation consequence:

Remove visible/hidden list swapping. Generate the replacement page deterministically from authoritative run, node, Shop-session, and refresh-use state. Cost deduction, remaining-use decrement, and complete page replacement are one transaction; any failure preserves all three. Tests must fix stable offer count across entry and refresh, including refresh after one or more purchases. Whether a definition already purchased in the same session may appear again remains a separate policy decision.

## 2026-07-12 — Shop item-pool composition policy

User decision:

- Author reusable global item-pool presets.
- Each Shop composes one or more preset references and may add optional local entries for bespoke merchandise.
- Preset references may apply Shop-specific weight multipliers.
- Preset entries and local entries use the same typed merchandise-entry shape.
- Entry and refresh sample deterministically from the flattened composed pool.
- A generated page cannot contain duplicate merchandise.
- Duplicate merchandise references across composed presets and local entries are rejected during data validation rather than silently merging weights.

Implementation consequence:

Replace per-merchant copied UUID stock lists with one canonical pool-composition schema. Do not retain fixed `visible_items`/`stock_items` as a fallback or alternate live schema. Startup validation resolves every preset reference, rejects recursive preset composition if presets are later allowed to reference other presets, rejects duplicate preset IDs and duplicate composed merchandise, and verifies that the eligible unique candidate count can satisfy the Shop's stable offer count. Value tier and merchandise category are authored data concerns, not sampling branches in runtime code.

## 2026-07-12 — Shop cross-refresh reappearance policy

User decision:

- Every pool entry has a `repeatable` RON option with default `false`.
- The active Shop session records merchandise identities purchased during that session.
- Purchased entries with `repeatable: false` are excluded from later refresh pages.
- Purchased entries with `repeatable: true` may appear again after refresh.
- A generated page still cannot contain duplicate merchandise, regardless of `repeatable`.

Implementation consequence:

Apply repeatability to merchandise identity, not to hard-coded item categories. Persist the session's purchased-identity set together with current offers and refresh state so reconnect/load cannot reset it. Candidate filtering happens before deterministic weighted sampling. Startup validation must account for the minimum eligible unique candidates needed at entry; runtime must also handle later pool exhaustion explicitly when non-repeatable purchases reduce the candidate set below the stable offer count. That exhaustion behavior remains a separate policy decision.

## 2026-07-12 — Shop refresh pool-exhaustion policy

User decision:

- `max_uses` is the maximum possible refresh count, not a guarantee that all remaining uses stay executable.
- After applying purchased non-repeatable filtering, refresh requires at least `offer_count` eligible unique candidates.
- If fewer candidates remain, refresh is disabled with an explicit pool-exhaustion reason.
- Do not generate a partial page and do not ignore `repeatable: false`.
- A rejected refresh consumes no Enkephalin, no refresh use, and does not alter the current page.

Implementation consequence:

Core computes pool exhaustion as part of authoritative refresh capability and projects it to Unity as a stable disabled reason. Validate initial eligibility at startup and reject obviously unusable authored configurations, while recognizing that runtime purchases can legitimately exhaust later refreshes. Focused tests cover exhaustion before command dispatch, direct-command rejection, zero mutation on failure, and exact full-page generation when enough candidates remain.

## 2026-07-12 — Shop selling availability and rate policy

User decision:

- Selling is optional per Shop and authored in RON.
- `sell_policy: None` means the Shop does not buy player merchandise.
- `sell_policy: Some((price_percent: N))` enables selling using that Shop's authored percentage of base price.
- Remove the runtime-global fixed 50% resale rule.
- Core computes and projects the authoritative sale quote and executable/disabled state.

Implementation consequence:

The Shop session copies its sale policy at entry. Unity never calculates the percentage locally. A successful sale atomically validates eligibility and quote, removes the exact owned merchandise or quantity, and grants checked Enkephalin; any failure preserves both inventory and currency. Capability projection must omit or disable Sell when the Shop has no sale policy. Merchandise eligibility, percentage bounds and rounding, and buy-then-sell arbitrage prevention remain separate policy questions.

## 2026-07-12 — Shop sale-eligibility filter and basic defaults

User decision:

- A Shop sale policy may author an optional merchandise-tag filter.
- The assistant may settle basic mechanical policies and should reserve user questions for decisions with material gameplay, progression, content, or economy consequences.

Basic policies selected:

- `accepted_tags: None` means every merchandise definition/current owned state that is intrinsically sellable.
- `accepted_tags: Some([...])` narrows acceptance to the listed validated typed tag IDs.
- Shop tags can never override definition or state denial such as equipped, bound, quest-locked, or otherwise non-removable ownership.
- Do not hard-code an equipment/consumable/fragment allowlist in Rust.
- Calculate `floor(base_price * price_percent / 100)` with checked integer arithmetic.
- A computed zero quote is disabled with an explicit reason; do not remove merchandise for zero Enkephalin.
- Core projects the exact quote and eligibility, and sale commit remains atomic.

Remaining hard policy:

Whether deliberate price differences between Shops may create profitable cross-Shop trading is an economy decision, not a rounding or validation detail.

## 2026-07-12 — Shop no-arbitrage policy

User decision:

- Do not allow profitable buy/resell within the same Shop.
- Do not allow profitable trading between different Shops.
- Shop remains an Enkephalin sink for preparation, not a currency-generation trading loop.

Basic implementation policies selected:

- Buy quote: checked ceiling division of `base_price * effective_buy_percent / 100`.
- Sell quote: checked floor division of `base_price * sell_percent / 100`.
- Reject zero-price Shop purchase authoring; disable zero-value sales.
- Shop RON authors a default buy percentage and a pool entry may optionally override it using the same typed percentage field.
- For every merchandise reachable from any live Shop pool, startup validation computes the minimum possible live buy quote and maximum possible live sell quote across every accepting Shop.
- Require `max_live_sell_quote < min_live_buy_quote`; equality is also rejected so trading always loses value.
- Merchandise that no Shop sells may still have a sale quote because reward- or gameplay-acquired goods need not have a purchase route.
- Same-session resale is allowed only when it satisfies the same strict loss-making invariant.
- Do not silently clamp bad authored prices. Startup rejects static violations; a defensive runtime violation disables the quote with an invalid-data reason and preserves state.

Validation consequence:

Cross-Shop validation must resolve composed presets, local entries, entry price overrides, Shop buy percentages, sale tag filters, and Shop sale percentages before accepting the live bundle. Tests include same-Shop and cross-Shop arbitrage rejection, rounding boundaries, overflow, merchandise with no purchase route, and zero mutation on a defensively rejected runtime quote.

## 2026-07-12 — Remaining user-facing policy decisions

User decisions:

- Boss defeat enters RunFailed immediately. A remaining checkpoint-load action may still be offered from RunFailed under the previously accepted BossOmen/checkpoint policy.
- After a non-boss combat failure leaves zero deployable employees, defer RunFailed only when at least one currently enterable node has explicit Treatment or Recruitment capability. Only qualifying nodes are enterable during deferral, and deployability is re-evaluated after resolution.
- Remove enemy range preview completely from Core projections, checkpoints, server/Unity contracts, Unity presentation, documents, mocks, and tests. Do not conflate this with actual enemy actions or separately authored attack telegraphs.
- Remove the player-facing/authoritative gameplay battle-record system rather than preserving rewound records or choosing a new record identity. Story documents and enemy-compendium collection are separate systems. Developer diagnostics, if retained, are non-authoritative and export failure is non-fatal.
- Maintenance delivers pending research through an explicit capability, with coherent delivery/node completion.
- Defer Equipment/Maintenance content scope and balance. Do not invent block values, generic recipes, or placeholder schema policy while deferred.

Planning consequence:

G11, G15, G16, and G21 no longer require user policy decisions. G22 is deferred. The remaining policy-decision markers in the Master are technical defaults delegated to the assistant or stale status to be resolved at child startup, not unanswered user-facing design questions.

## 2026-07-12 — PD-01 battle revision implementation audit

Question:

Is the accepted independent monotonic battle-update revision sufficient as currently worded, or does the runtime/transport/Unity boundary require a stronger contract?

Action:

Traced `ActiveBattleSession`, event-log drain and resync projection, battle start/tick/playback/deploy/VN mutation paths, Core update DTOs, root battle context, game_server command mapping and tick push paths, Unity `BattleRuntimeState` ordering/reconciliation, mocks, and focused tests.

Observed result:

- `LiveBattleUpdateDto` has event seq and battle time but no independent state revision.
- Pause, resume, and speed mutate playback and emit a battle update without advancing event seq or battle time.
- Battle VN progression is converted into a same-seq battle update after mutating top-level VN state; blocking VN can also stop subsequent ticks.
- Unity accepts a same-seq empty delta only when `server_battle_time_ms` is strictly newer. Therefore a valid same-seq/same-time playback or VN update can be silently discarded.
- Core `drain_battle_update_dto` mutates `last_pushed_event_log_seq`, mixing a transport delivery cursor into `ActiveBattleSession` and the root snapshot. Read-only `battle_update_dto_after` already demonstrates the preferable explicit-cursor projection seam.
- Battle updates do not carry exact Core battle capabilities; Unity substitutes a static battle action whitelist, so a revision that omits capabilities would still leave authoritative side-band state unordered.
- Command-triggered side messages return through the WebSocket session actor, while automatic ticks push directly from the player actor to the socket recipient. Revision detects stale state but does not itself guarantee dependent event-delta delivery order.
- `battle_resync` currently delegates to ordinary Unity `ApplyUpdate`, so it cannot authoritatively repair the same revision and cannot restore a missing setup payload.

Commands and actual results:

- `cargo test live_defense_playback_pause_freezes_server_tick_and_resume_advances --lib`: passed 1/1. It proves Core changes playback while battle time remains 0, but does not test Unity ordering.
- `cargo test paused_live_battle_tick_does_not_push_delta_until_resumed --lib` in game_server: failed before the target assertion because the server fixture lacks the required Standard RunStart Visual Novel trigger (`InvalidStaticData("missing RunStart visual novel trigger for Standard")`). This is the already tracked G04 fixture drift, not evidence against the revision finding.

Decision and next step:

The user accepted the refined policy on 2026-07-12. Promote it to the binding PD-01 PLAN contract. G01 implements a battle-UUID-scoped authoritative commit revision, increments once after a successful complete mutation transaction, and never increments during DTO construction/read/resync. Carry it through update, resync, and command acknowledgement; make Unity use revision for staleness, event seq for delta continuity, and time only for presentation. G01 also owns ordered server emission, explicit same-revision resync override semantics, exact revisioned battle capabilities, removal of the Core-owned delivery cursor, and actual Rust-to-Unity fixture coverage.

## 2026-07-12 — PD-02 capability semantics implementation audit

Question:

Can `allowed_actions` become an exact executable-command contract by improving its current state-derived list, or does payload-sensitive execution require a different shape?

Action:

Traced `ActionScheduler`, `AllowedActionContext`, cached `ActionValidator`, `GameCore::get_allowed_actions`, action gating through `PlayerBehavior::kind`, payload validators and handler checks across map/inventory/shop/reward/VN/battle flows, root snapshot projection, game_server forwarding, Unity `GameSnapshotStore`, command composition, static battle whitelist, mocks, and tests.

Observed result:

- `ActionKind` is explicitly documented as a payload-less state-gate capability. `PlayerBehavior::kind()` discards every concrete payload before the first gate.
- Core's first gate only checks whether the kind is present. Item ownership/binding/equip state, employee availability, selected node/reward/choice identity, Shop stock/funds/refresh state, battle cost/cooldown/tile, skill readiness/target, and other real checks occur later and can reject the command.
- `InNode` advertises event, support, recruitment, emergency-supply, Headquarters-Shop, map, and completion commands together regardless of the actual node payload.
- `InShop` always advertises purchase, sale, refresh, and exit. The runtime then rejects missing/hidden stock, insufficient funds, non-removable inventory, or non-refreshable sessions.
- `InBattle` advertises both Pause and Resume, every speed selection, recovery, deployment, withdrawal, and activation command regardless of playback or concrete candidates.
- Blocking VN exposes both Advance and SelectChoice kinds, even though a scene accepts only one progression shape.
- `GameCoreState.action_validator` stores a HashSet updated on transitions, while public gating recomputes `get_allowed_actions`; this is duplicated state rather than one execution authority.
- Root snapshot carries only action names and no disabled reasons or payload eligibility.
- Unity replaces all battle capability with a static `battleRuntimeActions` whitelist whenever an active battle runtime exists. Screen composers then combine that coarse gate with local selection checks and hard-coded English disabled reasons.
- Existing tests intentionally lock this coarse behavior, including “all ten battle actions,” “all four Shop actions,” and both Pause/Resume presence. Passing them proves state-table consistency, not command executability.

Commands and actual results:

- `cargo test action_scheduler --lib`: passed 14/14, confirming the current coarse state table and its legacy expectations.
- `cargo test game_core_rejects_disallowed_actions_and_updates_allowed_actions_on_state_transition --lib`: passed 1/1, confirming kind-level state gating but not payload-level parity.

Decision and next step:

The user accepted the refined policy on 2026-07-12. Promote it to binding PD-02. Keep Core authority but replace the single payload-less list with a canonical two-level contract: typed context `action_capabilities` plus typed payload eligibility in domain projections/previews. Projection and commit share the same preflight validators; execution revalidates atomically. Remove the cached duplicate ActionValidator authority, state-only execution gating, root `allowed_actions`, Unity's static whitelist, local eligibility inference, and legacy completeness tests without compatibility fields. Battle capabilities participate in PD-01's `battle_state_revision`. Keep G06 focused on the shared schema/evaluator/migration and assign concrete payload DTOs to their existing domain children rather than creating one unbounded refactor. G06 is now policy-ready after G01 establishes revision semantics.

## 2026-07-12 — PD-03 Event support implementation audit

Question:

Is “treat Event as supported live functionality and add the Unity route” a sufficient long-term policy, or does the existing Event state machine require a wider cross-layer contract?

Action:

Traced generated-map insertion and validation, live map/Event/Visual-Novel/BossOmen RON, Core node entry, persistent `EventSessionState`, scene advance, choice resolution, staged/immediate effects, Event-started combat and reward policies, typed snapshots/command results, server forwarding, Unity routing/readers/action names/command bus/composers, presentation-resource references, and focused Event tests.

Observed result:

- Map generation calls `ensure_event_node`; live map definitions contain one Event mapping, `event_story -> white_night_confession_01`.
- Core already owns a persistent scene-chain state machine and restores a committed Event combat choice for retry. Server transport forwards the typed snapshot/commands rather than being the missing domain layer.
- Unity has no Event route, typed nested Event reader, Event commands, or Event composer. `selected_event.type == "event"` falls through to `Unknown`.
- Event and global Visual Novel are separate runtime states. `EventSceneEntered` may open an overlay, so the client must return to the unchanged Event state after the overlay completes.
- The current Event choice DTO exposes only `selectable: bool`; the coarse action scheduler advertises both advance and choice for every InNode. Choice commands omit `current_scene_id`, unlike advance commands.
- `ApplyBossOmenStepResult` is accepted by the Event schema but both immediate and staged execution paths panic. It is absent from live Event RON.
- Missing Event combat encounter references also reach runtime panic rather than complete live-bundle rejection.
- The Unity repository contains no matching references for the live Event background, page, and choice label IDs, so route existence alone does not prove renderable live content.
- Current live data contains one Event while map generation guarantees at least one Event per generated map; support policy and repeat/frequency content policy are therefore distinct.
- `OnAnyResolution` applies staged Event rewards during post-battle resolution on a retryable defeat while retaining `started_combat`. The victory reward path applies `OnAnyResolution` staged rewards again. The existing test checks one defeat only, so it does not rule out repeated defeat/victory grants.

Commands and actual results:

- `cargo test event_scene --lib`: passed 2/2.
- `cargo test event_started_combat --lib`: passed 3/3. This confirms the currently tested single-resolution behavior but not once-only reward semantics across retry sequences.
- `cargo test event_node_commands_advance_scene_select_choice_and_complete_node --lib`: passed 1/1 against live RON.

Decision and next step:

Historical interim decision on 2026-07-12: the user initially accepted a first-class persistent Event state-machine refinement. The later direct interaction reference proved that its Event/Visual-Novel return and staged Event-combat reward framing did not match the intended feature. Preserve this paragraph only as audit history; the corrected binding PD-03 is recorded in PLAN and in the later correction experiment below.

## 2026-07-12 — PD-04 Event DTO implementation audit

Question:

Should the existing Core `EventSceneSnapshotDto` be frozen as the canonical Event wire shape while only the stale Unity document is corrected?

Action:

Traced Event RON authoring types and live data, Event projection, selected-event snapshot and command-result serialization ownership, server enrichment/forwarding, Unity contract examples, route/read-model/command consumers, presentation resource references, and existing Core/Unity tests.

Observed result:

- The current Core direction is newer than the Unity document: the wire uses ordered `script_ids[]` and no Event-local speaker/portrait, while the document requires singular `script_id`, speaker, and portrait.
- `selected_event.event` and `EventState.event` already share `EventSceneSnapshotDto`. The server does not invent a second Event shape; its selected-event enrichment applies compressed logs only to combat results.
- The current DTO nevertheless leaks `crate::game::data::event_data::EventChoicePreview` directly from RON authoring into the public wire contract.
- Authored preview `starts_combat` duplicates the actual `EventImmediateEffect::StartCombat` authority. Validation limits the number of StartCombat effects but does not require preview/effect agreement.
- `risk: Option<String>` is arbitrary protocol text. Live RON authors `High`/`Low`, which are neither stable localization IDs nor the game's abnormality `RiskLevel` enum.
- Flat `choices: []` requires Unity to infer whether the current interaction is advance or choice. It cannot structurally express PD-02's mutually exclusive interaction modes.
- `SelectEventChoice` lacks `current_scene_id`, while advance already carries it.
- Unity has no Event route or typed nested Event reader and retains only generic raw selected-event access.
- Core validates Event presentation IDs only for non-empty strings. Searches found no Unity Asset references for the current live Event background, page, or label IDs.
- Existing Core tests prove ordered script/page arrays and live scene progression, but no exact Rust-serialized Event fixture is consumed by Unity.

Commands and actual results:

- No new executable test was run for this read-only DTO audit. The immediately preceding PD-03 audit passed the live Event flow, but those tests do not serialize the final public shape or exercise a Unity Event reader.

Decision and next step:

Historical interim decision on 2026-07-12: the user initially accepted the `presentation/page_ids`, tagged interaction, derived preview, and choice-eligibility cleanup. The later reference established a different final contract: explicit image/text panels, employee selection, one-shot raw d20, and no initial preview/modifier system. Preserve the earlier result only as audit history; the corrected binding PD-04 is recorded in PLAN and in the correction experiment below.

## 2026-07-12 — PD-03/PD-04 correction from final Event interaction reference

Question:

Did the accepted interim Event contracts accurately represent the user's intended feature?

Action:

Compared the user's two Limbus Company-style reference images and explicit step-by-step flow against the accepted PD-03/PD-04 text, current live Event RON, `EventSceneEntered` VN trigger path, current Event DTO/commands, and the previously proposed page/preview model.

Observed result:

- The prior separation from Visual Novel was directionally correct, but retaining an optional Event-to-VN trigger/recovery contract was still wrong. The intended Event never uses VN speakers, character portraits, positions, expressions, or dialogue staging.
- Bottom employee portraits are performer selection UI. They are not characters displayed in the Event presentation.
- The intended presentation is a sequence of explicit image/text panels followed by choices, not one background plus loosely associated script/page IDs.
- The selected choice and employee are not committed while browsing the UI. Pressing the progress button is the single authoritative resolve boundary.
- The initial check is pure one-shot d20 with no modifiers. Natural 1/20 are distinct critical result kinds; 2..19 use `raw_roll >= required_roll`.
- Success executes the authored action. Failure enters a failure presentation and completes/consumes the Event only after the final player advance.
- The previous preview fields (`risk`, `starts_combat`, reward/penalty hints) and future modifier concepts are outside the initial requested feature.
- The current runtime resolves `SelectEventChoice` immediately and has no employee selection or persisted d20 result, so the accepted policy is a real implementation change rather than documentation alignment.

Commands and actual results:

- No executable tests were run because this turn changed policy documents only. Existing Event tests exercise the superseded immediate-choice flow and must be replaced rather than have expectations mechanically renamed.

Decision and next step:

Rewrite binding PD-03/PD-04 around the accepted state machine: image/text panels, local reveal cursor, choice then employee selection, one scene-qualified `ResolveEventChoiceCheck`, Core-owned one-shot unmodified d20, atomic success action or failure presentation, and final failure advance/node consumption. Remove Event-to-VN hooks and dormant future mechanics. Rename G03 to `illustrated_employee_check_event_flow`; incremental modifiers or distinct critical effects require later playtest-backed policy.

## 2026-07-12 — PD-06 global reach and target-centered TileArea audit

Question:

Does the temporary `WholeFieldValidTiles + TileArea` combination correctly express a stationary final boss selecting a unit anywhere on the battlefield and attacking around that unit, and how should a future field-wide expanding wave relate to it?

Action:

Traced cast targeting, step target reuse, range validation, TileArea geometry/execution, target usefulness, snapshot/catalog projection, live RON usage, and focused tests. Reconciled the code with the user's clarification that deployed units do not move and that a future wave is caster-centered, circular, and unblocked by void.

Observed result:

- Cast and step independently own target/range/pattern/air fields, while `defense_tile_range` is overloaded as cast reach, candidate filter, and delivered footprint.
- Validation permits a WholeField TileArea without a pattern but forbids attaching a pattern to WholeField; runtime requires the missing step pattern and otherwise finalizes an empty step.
- WholeField unit acquisition and target-anchored TileArea projection each work independently, but the mixed schema cannot connect them correctly.
- ReuseCastTarget can revalidate through the step range, while another target enum path bypasses it, making behavior depend on representational shape rather than one target-source contract.
- TileArea can anchor on a selected unit tile, but symmetric target-centered use can still inherit an unnecessary caster-facing dependency. AI usefulness can evaluate the center candidate rather than the complete footprint.
- No live gameplay RON uses WholeField, so removing the legacy schema requires no compatibility layer.
- The earlier question about a moving selected target was invalid under current gameplay. Only disappearance matters: capture the target tile so death or unit withdrawal cannot erase the announced attack location.
- A future expanding field wave is spatial tile propagation, not an instantaneous all-unit effect and not an oversized target-centered TileArea.

Commands and actual results:

~~~text
cargo test skill_enemy_single_whole_field_uses_valid_tiles_without_facing --lib
  1 passed; 0 failed; 639 filtered out

cargo test instant_tile_area_origin_anchor_projects_range_from_cast_target_tile --lib
  1 passed; 0 failed; 639 filtered out
~~~

Decision and next step:

Accept PD-06. G09 separates Global cast reach from required delivery footprint, captures the selected unit tile, removes duplicate step range authority and legacy mixed schema, and aligns Core projection/execution plus Unity DTO consumption. G10 preserves explicit authored selection semantics and introduces area-hit optimization only as an explicit rule. G18 corrects examples. Record the caster-tile-centered circular unblocked field wave as a future delivery contract only; do not add runtime or a loadable schema placeholder in this implementation.

## 2026-07-12 — PD-07 through PD-18 sequential policy revalidation

Question:

After rereading the rulebook and the not-yet-fully-decided master, which remaining policies are accepted, corrected, or deliberately deferred?

Action:

Presented PD-07 through PD-18 one at a time, separated existing runtime behavior from proposed policy, re-read current research-delivery and Maintenance runtime/Unity paths, and recorded the user's decisions without treating unresolved master entries as defects.

Observed result:

- PD-07 is accepted: basic attacks use weapon profiles; each skill explicitly owns its targeting selection without inheritance fallback.
- PD-08 remains accepted for the current version: no enemy range preview or reserved DTO field; active telegraphs and hazards remain independent.
- PD-09 is accepted at maximum strictness because a dedicated authoring program generates live RON. Writer and Core both validate exact canonical schema and the whole bundle.
- PD-10, PD-11, PD-15, and PD-16 were reconfirmed without changes.
- PD-12 was corrected: no player battle history exists, but a separate game-global monotonic abnormality/corrosion compendium commits actually spawned IDs only when the combat node is consumed. Ordinary loads and rollback never remove committed entries. Debug traces remain non-authoritative.
- PD-13 was corrected after runtime inspection: current delivery occurs on entry to Support, HeadquartersContact, Shop, and Reward, not Safe Zone and not Maintenance. The accepted replacement delivers automatically on the authoritative return to map/planning presented as Safe Zone and removes category entry delivery.
- PD-14 remains deliberately deferred. Runtime already has sufficient Maintenance foundations; concrete equipment recipes, E.G.O growth, modifiers, and balance should grow with actual content rather than placeholders.
- PD-17 was reconfirmed as complete Artifact removal with no replacement in this goal.
- PD-18 was finalized with explicit offer/sale quantities, persistent deterministic Shop session state, and constant per-use `base_cost` in the initial refresh contract. Concrete pools and balance remain content work.

Runtime evidence inspected:

- Research delivery is called from `ConfirmEnterNode` and uses the four-category Rust default allowlist. No node-completion delivery hook exists.
- Maintenance currently exposes equipment dismantle/enhance and skill-fragment dismantle/upgrade/awaken commands, Core previews with costs/gains/before/after/warnings, Unity readers/gateways, and focused Core/Unity tests.
- Live equipment content has only narrow recipe coverage centered on `standard_armor`, confirming that PD-14's remaining gap is content rather than a missing basic Maintenance loop.

Commands and actual results:

- Read-only `rg`/`sed` traces covered Core, live RON, and Unity Maintenance/research consumers.
- No executable tests were run because this pass changed policy documents only; existing focused test locations were inspected as evidence, not reported as newly executed.

Decision and next step:

Promote G10 and G12 to ready, retarget G21 to Safe Zone delivery, keep G22 deferred, retain G28 as ready under the finalized Shop contract, and add G29 for the new global compendium rather than overloading battle-record removal. G29 waits for G15's authoritative node/result commit seam. Update the rulebook and child documents during their owning implementation goals; do not implement policy through this master-only documentation pass.

## 2026-07-12 — Dependency-aware Phase decomposition and completion-review gates

Hypothesis:

Implementing PD-01 through PD-18 in numeric order or as one large execution goal would repeatedly migrate the same DTO/state/schema seams and allow child checkboxes to hide cross-component incompleteness. Dependency-aware Phases with mandatory completion reviews should reduce rework and prevent false completion.

Minimal trial:

Mapped every non-excluded G01..G29 child to one implementation Phase, traced hard dependencies among revision/capability, Event/combat transitions, node consumption/global persistence, targeting schemas, and Shop/Artifact/inventory, then compared the proposed gate with every procedure and verdict in `docs/goal_completion_review_guide.md`.

Observed result:

- Event must consume stable combat result/abandonment behavior, so the combat transaction Phase belongs before the Event vertical slice despite Event's high user-visible priority.
- G26 must precede G15; G15 then provides the transaction seam for G14, G16/G29, and G21.
- Research delivery cannot be finalized until Event/combat/Gate return-to-planning paths are stable.
- Enemy preview removal should precede the final targeting migration to shrink the public contract surface and prevent migrating a feature being deleted.
- The former G28 -> G27 -> G24 order leaves Shop quantity semantics dependent on an inventory migration scheduled last. A cleaner coordinated cutover is G24 final ordinary inventory -> G28 final Shop/removal of Artifact merchandise dependencies -> G27 isolated Artifact deletion. Artifact is not adapted into G24.
- G12 can establish the strict generated-data substrate early; each later domain migration must use it, and Phase reviews must verify strictness did not regress.
- A per-child review alone cannot prove two completed children share one source of truth. The guide's cross-component pass must therefore be a separate Phase review goal.

Document changes:

- Replaced the coarse three-phase list with Phases 0..7, exact child order, entry gates, user-visible exit conditions, and named R00..R07 review goals.
- Added a per-Phase review protocol using the guide's exact verdict, long-term-fit, evidence, correction-loop, and validation requirements.
- Reordered Phase 6 child prerequisites and updated collision notes without creating implementation or compatibility paths.
- Kept G05/PD-05 and G22/PD-14 in an explicit deferred lane with future review requirements.

Commands and actual results:

- Read all 177 lines of `docs/goal_completion_review_guide.md`.
- Used read-only `rg`/`sed` checks against the Master PLAN, policy notes, child catalog, progress table, and collision notes.
- No executable gameplay tests were run because this experiment changes orchestration documents only.

Decision and next step:

Adopt the Phase model. A Phase becomes `review_required` after implementation children finish and becomes complete only after its dedicated review goal passes. Any partial/document-only/policy-needed/low-fit result creates correction children and blocks the next Phase. Begin with G04, then create R00 only after G04's own completion review passes.

When a child completes its implementation pass, append:

| Date | Child | Baseline | Focused validation | Broad validation | Review verdict | Remaining risk |
| --- | --- | --- | --- | --- | --- | --- |
| 2026-07-12 | G04 `server_runstart_fixture_contract` | Reproduced 1/1 failure; historical server 19/25 | Server 2/2; Core 3/3 | Server 25/25; Core 681/681; fmt/check passed | Complete / High | Supported-mode RunStart cardinality remains explicitly owned by G12 |

Do not record a child as complete from its PLAN checkbox alone.

## 2026-07-12 — Phase 0 / G04 execution started

Hypothesis:

The Master can begin with its only ready child, G04, and restore the server regression baseline without entering any policy-bearing Phase 1 work.

Minimal trial:

Re-read the complete Master document set and required startup sources, record the current dirty worktree, then create the three G04 child documents with an explicit fixture-only boundary.

Observed result:

- The Master startup protocol and both audit inputs were read completely before code exploration or test execution.
- Current `git status --short` confirms a heavily dirty user worktree, including overlap in `../game_server/src/game/player_game_actor/handlers.rs`.
- `docs/goals/server_runstart_fixture_contract/PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md` were created.
- G04 retains DCA-H-08/CPR-M-01 ownership, forbids validation weakening, and requires the six blocked server assertions plus completion review.
- No production code, live data, server code, or Unity code was changed by this startup step.

Commands and actual results:

- Read all lines of the three Master documents, three common workflow/review documents, the 921-line DCA input, and the 821-line CPR input.
- `git status --short`: succeeded; existing user changes recorded.
- No executable tests were run; actual test count is 0 and is not claimed as validation success.

Decision and next step:

Mark Phase 0 and G04 active. Reproduce the current server fixture failure, trace the production invariant and reusable builder seam, then make a surgical fixture correction before broad validation.

## 2026-07-12 — G04 completed and handed to R00

Hypothesis:

A paired server Visual Novel/RunStart trigger fixture can restore the regression gate without changing production validation or entering Phase 1 strict-bundle work.

Action:

- Reproduced the missing Standard RunStart trigger failure with one nonzero focused server test.
- Changed only the server test fixture seam so definitions and typed Standard/Endless triggers are constructed together.
- Preserved the existing live policies (`OncePerRun`, `PersistAcrossCheckpointLoad`, priority 100, blocking) and all six affected assertions.
- Performed the child completion review using the guide.

Commands and actual test counts:

- One initial exact filter: 0 tests; rejected as evidence.
- Pre-change corrected server filter: 1 executed, 1 failed with the expected fixture error.
- Post-change server focused: 2 executed, 2 passed.
- Core focused: 3 executed, 3 passed.
- `cargo test -p game_server --all-targets -- --test-threads=1`: 25 passed, 0 failed.
- `cargo test -p game_core --all-targets -- --test-threads=1`: 681 passed, 0 failed.
- `cargo check -p game_server`: passed.
- Final `cargo fmt --all -- --check`: passed after one recorded formatting correction.
- Scoped `git diff --check`: passed.

Observed result:

- DCA-H-08's six-test fixture gate is restored without a bypass, fallback, alias, or weakened assertion.
- The real Rust typed snapshot serialization path in the server suite passed.
- Unity 6000.3.4f1 and both test assemblies are available; Unity was not run because this child changes no Unity surface.
- The child review verdict is `Complete` with High long-term fit.

Failure cause:

- Historical failure: server test fixture added VN definitions but left the `GameDataBuilder::empty()` trigger database empty.
- Intermediate failure: rustfmt wrapping only; corrected and revalidated.

Decision and next step:

Mark G04 complete and Phase 0 `review_required`. Create R00 as a separate read-only review goal. Preserve CPR-M-01's early supported-mode cardinality aspect under G12 rather than falsely treating the entire startup-validation finding as closed.

## 2026-07-12 — R00 independently passed Phase 0

Hypothesis:

Phase 0 can unlock Phase 1 only if a separate review reproduces the broad baselines and finds no hidden fixture bypass or ownership conflict.

Action:

Created `docs/goals/cross_layer_phase_00_baseline_completion_review/`, re-read G04 evidence, audited the current runtime/data/fixture/test seams, and independently re-ran the required gates without modifying production code.

Commands and actual test counts:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p game_server`: passed.
- Server all-targets: 25 passed, 0 failed.
- Core all-targets: 681 passed, 0 failed.
- Scoped `git diff --check`: passed.
- Unity Editor/version/assemblies and the Rust typed snapshot serialization path were independently reconfirmed read-only.

Observed result:

- R00 verdict: Complete.
- Long-term fit: High.
- Immediate correction: none.
- All six former server failures appear as passing tests.
- No relevant ignored test, fallback, compatibility path, definition-only server fixture, or validation weakening remains.
- G12 explicitly retains the early supported-mode RunStart cardinality concern.

Decision and next step:

Mark Phase 0 complete, activate Phase 1, and start G01 before G06/G07. Keep G12 within Phase 1 under its independent strict-data boundary.

## 2026-07-12 — Phase 1 / G01 started

Hypothesis:

Accepted PD-01 provides a complete enough public policy to begin a bounded cross-layer revision implementation without reopening gameplay decisions.

Action:

- Created and re-read the three `battle_projection_revision_contract` child documents before G01 code search.
- Began read-only tracing of Core mutation/projection, server cursor/emission, Unity apply/resync, tests, and dirty overlaps.

Commands and actual test counts:

- Read 278 lines across the three new G01 documents.
- Read-only `rg`/`sed`/diff inspection only; no G01 tests executed yet.

Observed result:

- The issue is a four-part ownership split, not a numeric DTO omission: Core consumes connection cursor state, update construction is fragmented around triggers, server command/tick output has multiple senders, and Unity uses event/time as projection freshness.
- No save migration policy has surfaced because active battle state is not part of the run checkpoint payload.

Decision and next step:

Mark G01 active. Finalize the transaction table and single commit/outbox seam, then implement Core-first with focused tests before server and Unity migration.

## 2026-07-13 — G01 completion review passes

Hypothesis:

G01 can close DCA-C-01/CPR-C-01 and unblock G06 only if Core, server, actual Unity consumption, canonical documents, and legacy searches all agree.

Action:

- Completed the Core revision transaction and pure projection seam, server actor cursor/ordered emission/reconnect seam, and Unity revision-first apply/resync seam.
- Removed the final optional root/mock revision fallback and synchronized canonical transport documents.
- Applied `goal_completion_review_guide.md` and created the G01 completion review.

Commands and actual test counts:

- Core all-targets: 682 passed.
- game_server all-targets: 25 passed.
- Unity EditMode: 182 passed through Unity MCP.
- Unity PlayMode: 49 passed through Unity MCP.
- Exact typed Rust fixture comparison, rustfmt, scoped diff checks, and legacy searches passed.

Observed result:

- G01 verdict: Complete.
- Long-term fit: High.
- Improvement class: none.
- G06 is unblocked; Phase 1 remains active.

Decision and next step:

Start G06 as the next child. Do not begin G07 until G06 receives its own Complete/High review.

## 2026-07-13 — Phase 1 / G06 started

Hypothesis:

Accepted PD-02 and the completed G01 revision seam define a bounded common capability child without absorbing battle or domain payload policy.

Action:

- Read the G06 finding rows, Phase 1 dependency, accepted PD-02 detail, and complete G01 review.
- Created `docs/goals/contextual_action_capability_projection/PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md` before G06 code search.

Commands and actual test counts:

- Documentation startup only; 0 G06 tests executed and no success is claimed.

Observed result:

- G06 is active.
- G07 and domain-child boundaries are explicit in the child plan.

Decision and next step:

Re-read the three G06 documents at the next implementation step, then trace current runtime/data/DTO/server/Unity paths before editing code.

## Failed-Experiment Recording Template

## 2026-07-13 — Phase 1 / G06 completed

Hypothesis:

G06 can replace the common/root coarse action authority atomically without absorbing battle or
domain payload policy.

Action:

- Implemented typed scoped capability projection and shared common preflight.
- Removed the root list, cached validator, state-only scheduler, old admin name, and derived
  checkpoint `can_load` field without aliases.
- Migrated game_server, Unity common consumers, actual Rust fixture, tests, canonical documents,
  and runnable probes.
- Applied `goal_completion_review_guide.md`.

Commands and actual test counts:

- Core all-targets: 669 nonzero tests passed.
- game_server all-targets: 26 nonzero tests passed.
- Unity MCP EditMode: 186/186 passed.
- Unity MCP PlayMode: 53/53 passed.
- Exact Rust fixture compare, Python probe AST parsing, rustfmt, scoped diff checks, legacy searches,
  and Unity Console error audit passed.

Observed result:

- G06 verdict: Complete.
- Long-term fit: High.
- Improvement class: none.
- G07 is unblocked; Phase 1 remains active.

Decision and next step:

Start G07 next. Preserve common capability authority and G01 revision ownership while removing the
battle-only Unity static action substitution.

## 2026-07-13 — Phase 1 / G07 started

Hypothesis:

G01 revision authority and G06 typed capability authority define a bounded G07 child.

Action:

- Re-read the startup rule, Master G07/Phase/PD-01/PD-02 sections, R00 review, G01 review, and G06
  review.
- Created `docs/goals/battle_capability_and_reconnect_authority/PLAN.md`, `EXPERIMENTS.md`, and
  `EXPERIMENT_NOTES.md` before G07 code search.

Commands and actual test counts:

- Documentation startup only; 0 tests executed and no success claimed.

Observed result:

- G07 is active with explicit G01/G06 and later-domain boundaries.

Decision and next step:

Trace current runtime/data/DTO/server/Unity paths and record the action/payload ownership matrix
before implementation.

### YYYY-MM-DD — Short experiment title

Hypothesis:

Action:

Changed files:

Commands and actual test counts:

Observed result:

Failure cause:

Revert or correction:

Decision and next step:

## Final Cross-Goal Validation

Not started.

Required final evidence:

- all registry entries have terminal dispositions,
- all policy decisions are resolved or explicitly retired by the user,
- per-child completion reviews are recorded,
- Core broad test suite passes,
- game_server broad test suite passes,
- required Unity tests and probes pass,
- shared DTO and live schema seams have no competing owners,
- legacy aliases, fallbacks, stale mocks, and ignored tests are absent,
- durable documents match runtime and live data.

## 2026-07-13 — G07 battle capability and reconnect authority completed

Child:

- `docs/goals/battle_capability_and_reconnect_authority/`

Outcome:

- Core's complete revisioned battle projection now owns context capability and typed bounded payload
  eligibility; selected combinatorial deployment payloads use Core preview/preflight.
- Every active battle mutation requires exact battle UUID/revision. Missing or stale authority
  rejects before mutation; game_server emits ordered correlated error then current battle resync.
- Unity's static battle whitelist and local deployment/skill execution inference were removed. The
  strict consumer rejects malformed or incomplete eligibility and requests resync.
- Blocking battle VN retains typed query/setup-loss recovery while exposing only valid progression.
- Server/Unity canonical documents and the runnable WebSocket probe now match actual side-message,
  eligibility, and stale-recovery behavior.

Validation:

- Core all-targets: 670 nonzero passed, 0 failed.
- game_server all-targets: 27 nonzero passed, 0 failed.
- Unity MCP EditMode: 188/188 passed.
- Unity MCP PlayMode: 51 passed, 0 failed, 2 explicit live-server skips.
- Exact Rust fixture equality, two actual live WebSocket probes, rustfmt, scoped Core/server/Unity
  diff checks, legacy/ignored-test search, and Unity Console error audit passed.

Review:

- `docs/goals/battle_capability_and_reconnect_authority/COMPLETION_REVIEW.md`
- Verdict: Complete.
- Long-term fit: High.
- Improvement class: none.
- Immediate correction: none.

Master decision:

- Close DCA-H-11, DCA-H-12, DCA-M-12 and the G07 battle portions of PD-01/PD-02.
- Keep Phase 1 active. G12 is next, followed by R01.

## 2026-07-13 — G12 initialized

- Created `docs/goals/gameplay_ron_strictness_and_bundle_validation/PLAN.md`, `EXPERIMENTS.md`, and
  `EXPERIMENT_NOTES.md` before any G12 runtime/data/test exploration.
- The child owns DCA-H-04, CPR-H-08, CPR-M-01's remaining supported-mode RunStart cardinality
  validation, and CPR-M-02 under accepted PD-09.
- G12 is bounded to the production Core generated-RON parse/final-bundle publication authority.
  New domain gameplay policy, authoring-program UX, saves, and unrelated Unity DTO work remain out of
  scope.
- Master status moved from ready to active. R01 remains blocked until G12 completes and passes its
  child completion review.

## 2026-07-13 — G12 gameplay RON strictness completed

Child:

- `docs/goals/gameplay_ron_strictness_and_bundle_validation/`

Outcome:

- Exact schema version 1 and one embedded source set now own 23 gameplay RON files.
- Local/domain/cross-catalog/RunStart/Event/extra-wave/map validation returns typed source-aware
  errors before successful publication; duplicate winners, aliases, unsupported variants, and
  fallback/partial bundles are absent.
- Map/battlefield consumers use the exact stored catalogs; generated preview/briefing validation
  covers every scaling stage and five deterministic seeds before the Arc is returned.
- game_server maps failure before bind. Unity DTO/transport is unchanged.

Validation:

- Core all-targets: 710 passed, 0 failed, 0 ignored.
- game_server all-targets: 27 passed, 0 failed, 0 ignored.
- Unity MCP EditMode: 188/188 passed, 0 failed, 0 skipped.
- Formatting, Core/server all-target checks, canonical doc, alias/fallback/ignored-test, and
  production-entry searches passed.

Review:

- `docs/goals/gameplay_ron_strictness_and_bundle_validation/COMPLETION_REVIEW.md`
- Verdict: Complete.
- Long-term fit: High.
- Improvement class: none.
- Immediate correction: none.

Master decision:

- Close DCA-H-04, CPR-H-08, CPR-M-01's remaining supported-mode cardinality portion, and CPR-M-02.
- Keep Phase 1 active until R01 independently reviews G01/G06/G07/G12 together.

## 2026-07-13 — R01 Phase 1 authority review started

- Created `docs/goals/cross_layer_phase_01_authority_completion_review/PLAN.md`, `EXPERIMENTS.md`,
  and `EXPERIMENT_NOTES.md` before R01 source search or validation.
- R01 independently reviews G01/G06/G07/G12 across Core, live data, game_server, Unity, and canonical
  documents.
- Documentation initialization executed 0 tests and claims no review success.
- Phase 2 remains blocked until R01 produces a Complete / High / no-immediate-correction review.

## 2026-07-13 — R01 found Phase 1 authority gaps

Review:

- `docs/goals/cross_layer_phase_01_authority_completion_review/COMPLETION_REVIEW.md`
- Verdict: Partially complete.
- Long-term fit: Medium.
- Improvement class: immediate correction recommended.

Findings:

- R01-H-01 / MSTR-05: revisioned battle capability is projected but not used as Core battle-context
  execution preflight. Direct current-authority commands can bypass blocking-VN mutation omission.
- R01-H-02 / MSTR-06: server emits current resync after stale authority but not missing authority.

Validation context:

- Core 710/710, game_server 27/27, Unity EditMode 188/188, and Unity PlayMode 51 passed/2 explicit
  skips are green, demonstrating that these are untested authority paths rather than broad breakage.
- Actual Rust fixture comparisons, corrected focused tests, static checks, canonical/legacy searches,
  and repository-scoped diff checks passed.
- Two zero-test focused attempts and one cross-repository diff command mistake are recorded as
  failures and not counted.

Master decision:

- Register G30 as the one bounded correction owner, but do not create or implement it without user
  authorization.
- Reclassify G07/R01 as correction-required/Partially complete.
- Keep Phase 1 active and Phase 2 blocked.

## 2026-07-14 — G15 result/abandonment transaction passes completion review

- Implemented exact pre-entry `RetreatBattle` restoration, staged committed-result settlement,
  fatal-before-retry priority, and immediate typed recruitment recovery restriction over G26.
- Synchronized Core and external Unity canonical contracts and corrected the explicit live harness
  to consume live battle transport authority before asserting restored root attempt state.
- Final validation: Core all-targets 719, game_server all-targets 28, Unity EditMode 189, and Unity
  explicit live PlayMode 1 all passed.
- G15 completion review: Complete / High / no immediate correction.

Master decision:

- Close DCA-M-17, DCA-M-20, DCA-M-31, DCA-L-04, CPR-H-02, and CPR-H-09.
- Unlock G14 `boss_omen_gate_and_rollback_policy` as the next Phase 2 child.

## 2026-07-14 — G14 BossOmen transition/recovery passes completion review

- Forced BossOmen victory now commits a deterministic same-Floor exit Gate and advances only on
  ordinary Gate confirmation.
- Qualified forced-BossOmen RunFailed now exposes/revalidates the existing checkpoint action; valid
  load restores the checkpoint once, while standard final boss and exhausted capacity stay terminal.
- Final validation: Core all-targets 720, game_server all-targets 28, Unity EditMode 189.
- G14 completion review: Complete / High / no immediate correction.

Master decision:

- Close DCA-H-14 and DCA-H-15.
- Unlock G08 `unity_recovery_and_roster_command_flows` as the final implementation child before R02.

## 2026-07-14 — G08 Unity recovery/roster flows pass completion review

- Added explicit Unity command/UI paths for setup reconstruction, checkpoint load, and roster
  movement without a new public schema.
- Repaired the existing Core `MoveRosterUnit` capability/execution context parity.
- Incomplete battle runtime now retains only revisioned query/recovery capabilities until setup is
  rebuilt; gameplay never falls back to root capability.
- Final validation: Core all-targets 722, game_server all-targets 28, Unity EditMode 193, ordinary
  PlayMode 51 passed/2 explicit skips, and explicit live Rust-server PlayMode 1 passed.
- G08 completion review: Complete / High / no immediate correction.

Master decision:

- Close DCA-H-13 / CPR-H-07.
- Activate R02 `cross_layer_phase_02_combat_completion_review`; Phase 3 remains blocked until R02
  independently returns Complete / High with no immediate correction.

## 2026-07-14 — R02 closes Phase 2 and unlocks G02

- Independently traced tactical death/working condition, fresh deployment, employee aggregation,
  staged result commit, exact entry restoration, fatal/recovery ordering, BossOmen Gate/load,
  server transport, and Unity capability consumption on the current worktree.
- Current validation: Core combat 55, map flow 28, result stats 4, all-targets 722; game_server
  all-targets 28; Unity EditMode 193, ordinary PlayMode 51 passed/2 explicit environment skips,
  and explicit current Rust-server PlayMode 1.
- Fixed-60% death override, partial rollback, direct forced Floor advance, static/root Unity
  recovery, local roster mutation, aliases, and relevant ignored tests were not found.
- R02 completion review: Complete / High / no immediate correction.

Master decision:

- Close Phase 2.
- Activate Phase 3 and create G02 `unity_wire_identity_alignment` as its first bounded Child.
- Preserve G03's Event behavior/state machine and G19's VN-authority removal as separate successors.

## 2026-07-14 — G02 wire identity alignment passes completion review

- Corrected Unity item target key, HQ candidate identity, reward option identity, combat-result
  primary abnormality identity, and active modifier display-name source without aliases.
- Corrected root/skill/completion canonical examples and pinned one shared Event inner DTO across
  root snapshot and `EventState`.
- Final validation: Core 723, game_server 28, Unity EditMode 194, ordinary PlayMode 51 passed with
  2 explicit skips.
- G02 completion review: Complete / High / no immediate correction.

Master decision:

- Close G02 findings DCA-H-09, DCA-H-10, DCA-M-14, DCA-M-15, DCA-M-16, DCA-L-02, DCA-L-03 and
  their CPR aliases.
- Activate G03 `illustrated_employee_check_event_flow`.

## 2026-07-14 — G10 explicit skill targeting authority passes completion review

- Removed the final implicit `EnemySingle` selection default and made unknown competing target
  authority fail strict deserialization.
- Pinned every live skill's exact selection and proved skill selection is unchanged across
  conflicting weapon profiles while basic attacks retain weapon-profile targeting.
- Final validation: Core all-targets 748, game_server 30, Unity EditMode 212, and Unity PlayMode 52
  passed with four explicit environment skips.
- G10 completion review: Complete / High / no immediate correction.

Master decision:

- Close DCA-H-02.
- Activate G09 `skill_global_reach_and_target_centered_delivery` under accepted PD-06.

## 2026-07-14 — G09 Global reach and target-centered delivery pass completion review

- Split skill cast acquisition into explicit `reach`/`target_range`/`air_capable` authority and
  made every TileArea own a required concrete footprint; removed step range/air authority.
- Migrated all 25 live skills to explicit reach/air authoring and all 21 live TileArea steps to
  concrete symmetric footprints without compatibility aliases.
- Proved a stationary no-facing Global caster keeps the captured target tile after both withdrawal
  and death, with edge/void, air, usefulness, event projection, and execution parity.
- Published strict catalog v2 separation through Core/server/Unity; the future expanding field wave
  remains documented only and has no loadable variant.
- Final validation: Core all-targets 750, game_server 30, Unity EditMode 215, and Unity PlayMode 52
  passed with four explicit environment skips.
- G09 completion review: Complete / High / no immediate correction.

Master decision:

- Close DCA-H-01 / CPR-M-05.
- Activate G13 `battlefield_projection_canonical_cleanup`; G18 is ready after G13 and G20 remains
  last in Phase 5.

## 2026-07-14 — G13 battlefield canonical cleanup passes completion review

- Corrected the stale Core HUD billboard sentence to the implemented lower-world-anchor to
  dedicated screen-space overlay projection contract.
- Distinguished Rust/Unity FacilityEntity wire enum/parser shapes from current production:
  validation rejects live Facility waves and setup cannot build them.
- No code, live RON, schema, server behavior, or Unity behavior changed.
- Focused live PVE Core test 1/1 and exact HUD PlayMode test 1/1 passed; preceding broad gate remains
  Core 750, server 30, Unity EditMode 215, PlayMode 52 plus four explicit environment skips.
- G13 completion review: Complete / High / no immediate correction.

Master decision:

- Close DCA-H-07 and DCA-M-10.
- Activate G18 `skill_contract_example_cleanup`; G20 remains the final Phase 5 child.

## 2026-07-14 — G18 skill example cleanup passes completion review

- Corrected final ReuseCastTarget wording to distinguish no acquisition-range recheck from
  execution-time live/hostile/air/usefulness gates.
- Replaced stale RetargetOnStep examples with all four current live skill/step pairs and labeled
  removed FirstStepTarget inference as legacy.
- Confirmed the G09 TileArea example has one concrete footprint and corrected the remaining detailed
  Unity skill catalog example from v1 to v2.
- Focused Core reuse/strict/catalog tests 1+1+3 and Unity catalog tests 3 passed; no production code
  or live data changed.
- G18 completion review: Complete / High / no immediate correction.

Master decision:

- Close DCA-M-03, DCA-M-04, and DCA-M-05.
- Activate G20 `live_skill_catalog_evidence_governance` as the final Phase 5 implementation child.

## 2026-07-14 — G20 live catalog evidence governance passes completion review

- Replaced the overclaiming v1 behavior list with strict v2 delivery/reference ownership evidence
  derived from loaded live data, including explicit unreferenced and overlapping owner states.
- Published durable author/reviewer/version/behavior-proof lifecycle and linked future catalog
  changes through the docs index and engineering matrix.
- Expanded the derived fragment index from 22 to all 30 abnormality-origin fragments and added a
  machine comparison against live origins.
- Final validation: focused audit 4, Core all-targets 751, game_server 30; no Unity surface changed.
- G20 completion review: Complete / High / no immediate correction.

Master decision:

- Close DCA-M-06, DCA-M-09, and DCA-M-24.
- Activate R05 `cross_layer_phase_05_targeting_completion_review`; Phase 6 remains blocked.

## 2026-07-14 — Initial R05 finds PlayMode isolation correction

- Independent Core composition and legacy audits passed; Core all-targets 751, game_server 30, and
  Unity EditMode 215 passed.
- Focused Unity EditMode 7 and PlayMode 2 passed, but the subsequent full PlayMode run failed the
  first starter-selection test (expected Ada, retained hover selected Cyra). Its exact rerun failed;
  seven later full-suite failures are cleanup cascade.
- R05 initial verdict: Partially complete / immediate correction recommended. Production
  targeting/data/DTO/canonical items remain Complete / High.

Master decision:

- Register MSTR-07 and activate G31 `unity_playmode_pointer_isolation_correction`.
- Set Phase 5 to correction_required and block Phase 6 until G31 and the appended R05 rerun pass.

## 2026-07-14 — G31 PlayMode isolation correction passes completion review

- Replaced queued method-local cursor warp with immediate class-level Input System state
  neutralization before/after a frame.
- Added baseline-root UnityTearDown so assertion abort cannot leak test-created UI into later cases.
- Exact failure 1/1, contaminating focused sequence 2/2, full PlayMode 52+4, immediate repeated full
  52+4, and EditMode 215 all passed without retries/order/ignore or assertion relaxation.
- G31 completion review: Complete / High / no immediate correction.

Master decision:

- Close MSTR-07.
- Reactivate R05 for its mandatory independent correction rerun; keep Phase 6 blocked until it passes.

## 2026-07-14 — R05 correction rerun closes Phase 5

- Independently repeated focused Core 13+4, focused Unity EditMode 7 and PlayMode 2, Core all-targets
  751, game_server 30, Unity EditMode 215, and focused→full→full PlayMode 52+4 twice.
- Legacy/compatibility/source, formatting, diff, ignore/workaround, and final console audits passed.
- R05 current verdict: Complete / High / no immediate correction.

Master decision:

- Close Phase 5.
- Activate Phase 6 and create G24 `inventory_reward_capacity_contract` first. G28 and G27 remain
  ordered successors; R06 remains blocked.

## 2026-07-14 — G24 closes ordinary inventory authority

Hypothesis:

One persisted Core inventory could remove capacity-based ordinary grant failure while retaining
equipment instance identity, replacing per-copy consumables with definition stacks, and giving Unity
explicit high-water presentation state without creating a second capacity authority.

Action:

Implemented and reviewed G24 across inventory storage/mutations, typed reward effects, Event/admin/
combat/legacy-Shop callers, item use, checkpoints, battle-entry rollback, root snapshots, server
pass-through documentation, Unity reading/presentation/mocks, a Rust-generated Unity fixture, tests,
and canonical contracts. Artifact remained isolated and unchanged.

Observed result:

- Ordinary equipment/consumable admission is logically unbounded. Reserve starts at 50, grows by
  sufficient 10-slot increments for successful new occupied slots, and never shrinks.
- Equipment remains owned-instance keyed. Consumables are checked positive quantity stacks keyed by
  definition ID and selected on the wire by the static metadata UUID.
- First consumable grant is `added`; repeats are `updated` with full quantity. Partial use updates;
  final use removes the static definition UUID.
- Checkpoint and whole-stage abandonment restore exact quantity and reserve. Artifact does not affect
  ordinary occupancy/reserve.
- Core all-targets passed 761/761; game_server 30/30; Unity EditMode 217/217; Unity PlayMode 52/52
  with 4 explicit external-server skips; cleared console 0 errors. Fixture drift, formatting, scoped
  whitespace, and legacy searches passed.
- G24 completion review: Complete / High / no immediate correction.

Decision and next step:

Close G24 and activate G28. G28 must compose the G24 equipment-instance/definition-stack transaction
instead of reintroducing a Shop-local quantity or capacity model. G27 remains blocked until G28
removes Artifact merchandise dependencies; R06 remains blocked until G27 closes.

## 2026-07-14 — G28 closes the final Shop economy contract

Hypothesis:

One deterministic Core Shop session could serve Headquarters and Unknown contexts with composed
typed merchandise, checked quotes, optional refresh/selling, G24 transactions, and complete Unity
decision data while deleting all prototype and Artifact merchandise dependencies.

Action:

Replaced the fixed visible/hidden page and reroll schema with reusable merchandise presets, typed
weighted offers, persistent node sessions, checked buy/sell quote validation, shared eligibility and
atomic commits, checkpoint restoration, full root projections, generated Unity fixture, and Unity
offer/sale/refresh presentation and dispatch. Authored distinct live Headquarters and Unknown
profiles using only priced equipment and consumables.

Observed result:

- Identical authoritative state reproduces exact Shop, page order, offer identities, quantities and
  quotes. Refresh is full-page, charged, use-limited, and excludes purchased non-repeatable items.
- Equipment purchase/sale preserves instance identity; consumables use explicit G24 stack quantity.
  Exhaustion, stale selectors, invalid ownership/tags/quantity/quote, funds, overflow, and arbitrage
  reject without partial mutation.
- Root DTO, action capabilities, command commits and Unity use the same Core evaluators and typed
  reasons. No client price/pool/sellability authority remains.
- Artifact and zero-price placeholders are absent from live Shop content and current Shop surfaces;
  prototype reroll/fixed-page vocabulary remains only in deliberate rejection fixtures.
- Final validation passed Core 768, game_server 30, Unity EditMode 220, Unity PlayMode 52 plus 4
  explicit external-server skips, exact generated fixture, formatting/scoped whitespace/legacy/content
  audits, and cleared console zero errors.
- G28 completion review: Complete / High / no immediate correction.

Decision and next step:

Close MSTR-03 and G28. Activate G27 `artifact_legacy_removal` to delete the now-isolated Artifact
subsystem without replacement or compatibility. R06 remains blocked until G27 closes.

## 2026-07-14 — G27 deletes the legacy Artifact subsystem

Hypothesis:

After G24 and G28 isolated all ordinary inventory and Shop authority, the remaining Artifact graph
could be deleted end-to-end without compatibility state, replacement content, or regression to
equipment/consumable acquisition and battle behavior.

Action:

Deleted live RON and embedding, data schema/builder/registry, inventory/reward/admin/public DTOs,
battle scenario/runtime/stat/trigger/cooldown/log/validation, server mappings, generated fixture
fields, Unity admin/model/mock/test surfaces and current canonical claims. Added strict rejection of
the removed reward variant and retained explicit equipment-trigger ordering coverage.

Observed result:

- No empty catalog, optional field, alias, tombstone save state, replacement relic, compensation
  item, migrated effect/name/ID/price, or ignored test exists.
- The live embedded source set is 22; ordinary inventory remains equipment instances plus consumable
  definition stacks, and Shop remains the completed G28 contract.
- Inventory, Shop and action-capability fixtures regenerate byte-exactly. The action fixture also
  absorbed accumulated current-generator compendium/reserve/catalog-v2 drift.
- Current source/live/server/Unity/current-canonical audits are clean apart from one deliberate
  strict removed-vocabulary rejection test; historical goal records remain evidence only.
- Final validation passed Core 759, game_server 30, Unity EditMode 220, Unity PlayMode 52 plus 4
  explicit external-server skips, formatting/scoped whitespace/fixture checks and console zero errors.
- G27 completion review: Complete / High / no immediate correction.

Decision and next step:

Close MSTR-02 and G27. Activate R06 `cross_layer_phase_06_economy_completion_review` for an
independent combined Phase 6 audit before Phase 7 can begin.

## 2026-07-14 — R06 independently closes Phase 6

- Re-ran 15 focused Core inventory/Shop/checkpoint/reward/deletion tests and 4 focused Unity
  Rust-fixture projection/reconciliation tests in the combined post-G27 state.
- Repeated Core all-targets 759 twice, game_server 30, Unity EditMode 220 and Unity PlayMode 52 plus
  4 explicit external-server skips.
- Reproduced inventory, Shop and action-capability Unity fixtures byte-exactly from current Rust.
- Fixed-capacity admission, prototype Shop, deleted-subsystem/replacement, formatting, scoped
  whitespace/diff, current-canonical and cleared-console audits passed. Legacy Shop strings remain
  only in deliberate negative assertions.
- R06 verdict: Complete / High / no correction child.

Master decision:

Close Phase 6. Activate Phase 7 and G25 `goal_lifecycle_and_probe_discoverability` for lifecycle/probe/
canonical/evidence governance synchronization before final review R07.

## 2026-07-14 — G25 closes lifecycle and probe discoverability

- Published one child→completion review→independent phase review→correction child and same-review
  rerun→next phase→final review order in the durable workflow.
- Reconciled README/current-policy ownership with retained historical goal evidence and explicitly
  classified the older similarly named standalone cleanup proposal as historical and unexecuted.
- Published `docs/probe_registry.md` with three exact external probe paths, their server gates,
  environment, adjacent contracts and evidence rules; verified every path and parsed every script.
- Audited PD-01..PD-18 canonical ownership. Implemented policy is held by current gameplay/runtime/
  data/targeting/external Unity contracts; PD-05/PD-14 remain deferred and PD-17 remains deletion.
- Reconciled master findings and child/review statuses while preserving G05/G22 triggers.
- Documentation/stale phrase/path checks and Core/server all-target safety checks passed.
- G25 completion review: Complete / High / no immediate correction.

Master decision:

Close G25 and activate R07 `cross_layer_master_final_completion_review`. R07 must independently
audit the whole active master rather than treating child or phase verdicts as final proof.

## 2026-07-14 — R07 independently closes the master

- Reconciled 52 DCA, 15 CPR-input, 7 MSTR and 18 PD registry entries; verified all 29 non-deferred
  child and 7 prior phase completion-review paths plus the two deferred triggers and exclusion.
- Directly sampled every phase across current runtime/data/server/Unity/test seams and found no
  competing authority or cross-phase regression.
- Corrected two stale child-directory slugs and replaced the separate stale Phase 5 status snapshot;
  both were final registry/evidence synchronization, not implementation defects.
- Current legacy/compatibility/ignored-test and durable-ownership audits passed.
- Final broad validation passed Core 759, game_server 30, Unity EditMode 220 and Unity PlayMode 52
  plus four explicit live-server skips; three fixtures are byte-exact and console has zero errors.
- Formatting and Core/server/Unity scoped whitespace/diff checks passed.
- R07 verdict: Complete / High / no correction child.

Master decision:

Close Phase 7 and the Cross-Layer Contract Hardening Master Goal. G05 and G22 remain user-deferred
under their documented triggers and are not unfinished active scope.
