# Shop Economy And Offer Contract

Goal: G28

Finding: MSTR-03

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Replace the prototype fixed-page Shop with one Core-authoritative deterministic offer, quote,
refresh, sale, persistence, and Unity interaction contract that consumes G24 inventory identity and
removes every Artifact merchandise dependency.

## Accepted Boundary

- One shared Shop system serves Headquarters-support and exploration Unknown-Shop contexts through
  distinct authored Shop profiles. Commands, session mechanics, DTOs, and Unity screens are shared.
- A Shop composes reusable global merchandise presets plus optional local entries. Preset references
  may multiply weights; duplicate merchandise across the flattened composition is invalid.
- Merchandise entries are typed and extensible. Initial live content may use equipment and
  consumables, but validation must not assert that future typed categories are permanently forbidden.
- Every Shop authors stable `offer_count`, default buy percentage, optional refresh policy, optional
  sale policy, and its composed merchandise source.
- Generated offers have stable offer identity, explicit quantity, exact checked total buy quote,
  full display data, and typed eligibility. A page contains no duplicate merchandise.
- Equipment purchase grants one owned instance. Consumable purchase grants the offer quantity into
  G24's definition stack. Purchase selects an offer, not a definition UUID.
- An enabled refresh authors constant `base_cost` and `max_uses`. Success atomically charges the
  current cost, consumes one use, and replaces the whole page with exactly `offer_count` offers.
  Insufficient funds, use exhaustion, or candidate exhaustion leaves all state unchanged.
- Purchased non-repeatable merchandise is excluded from later pages. Repeatable merchandise may
  reappear on later pages but never duplicates within one page.
- Selling is optional and authors a percentage plus optional validated merchandise tags. It can
  never override intrinsic ownership denial such as bound/equipped/non-removable state.
- Sale selects equipment by owned instance UUID at quantity one or a consumable definition stack by
  static UUID and explicit positive quantity. Core projects and commits the exact quote.
- Buy uses checked ceiling percentage arithmetic; sale uses checked floor arithmetic. Zero-price
  purchases, zero-value sales, arithmetic overflow, and any same/cross-Shop non-loss invariant
  violation are rejected without mutation.

## Session And Determinism Contract

- Initial and refreshed pages are deterministic from authoritative run/floor/node/Shop-session/page
  state and weighted sampling without replacement.
- Active session state owns current offers, purchased merchandise identities, refresh uses, and the
  deterministic page continuation. It survives root snapshot/reconnect without regeneration.
- Checkpoint restoration reproduces the same Shop selection and initial page from restored run state;
  any persisted Shop-session record included in the checkpoint restores exactly.
- Hidden generation candidates remain Core-private. No fixed visible/hidden page, whole-page swap,
  free reroll, `DiscountShop`, or client-side quote calculation remains.

## External Contract

- Root `selected_event` exposes Shop identity/context, current offers, refresh status/cost/reason,
  sale policy availability, and authoritative sale candidates/quotes.
- Context `action_capabilities` and Shop payload eligibility share the same evaluators used at commit.
- Rename the prototype `RerollShop` command/result to `RefreshShop` without a compatibility alias.
- Command results and the post-command root snapshot expose committed currency, inventory diff, and
  the current Shop session projection needed to reconcile selection.
- Unity renders names, type/rarity/effect or equipment detail, quantity, exact quote, and disabled
  reason. It reconciles stale offer/inventory selection after every snapshot and never renders a
  command from local availability guesses.

## Initial Live Content Direction

- Headquarters support uses a low-value supply profile centered on current live common/uncommon
  consumables and basic non-placeholder equipment. Refresh is enabled.
- Unknown exploration uses a higher-value profile composed from current live rare consumables and
  non-placeholder higher-rarity equipment. Refresh availability remains explicitly authored.
- Artifact UUIDs and zero-price placeholder equipment are not merchandise.
- Initial values are authored content protected for validity, deterministic behavior, and
  no-arbitrage—not a permanent closed category or tier invariant.

## Validation

- Strict RON tests for presets, composition, duplicate references, missing items, zero price/weight/
  quantity, percentage bounds, offer satisfiability, optional policies, and no-arbitrage.
- Deterministic session tests for initial page, full refresh, stable identity, reconnect projection,
  purchased repeatability, pool exhaustion, and checkpoint reproduction.
- Purchase/sale tests for equipment instance and consumable quantity semantics, exact quotes,
  bound/equipped/tag denial, insufficient resources, overflow, stale selectors, and zero mutation.
- Rust-generated Unity fixture and focused Unity read-model/presentation/selection/dispatch tests.
- Full Core, live RON, game_server, Unity EditMode/PlayMode, formatting, fixture drift, whitespace,
  legacy-name/schema, Artifact-merchandise, and console validation.
- Independent completion review before G27 begins.

## Exit Gate

Complete / High only when live Shop entry through both contexts produces deterministic typed offers,
every quote/capability/commit shares Core authority, refresh and sale are atomic and persistent, Unity
uses the full typed projection, Artifact/placeholder merchandise and prototype Shop schema are gone,
and G27 can delete Artifact without touching Shop.

Result: Complete / High. The implementation and independent review satisfy the exit gate. G27 may
delete Artifact without adapting or revisiting Shop.
