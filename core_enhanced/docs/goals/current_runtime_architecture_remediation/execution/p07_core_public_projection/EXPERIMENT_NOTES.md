# P07 Core Public Projection Experiment Notes

## Intake Decisions

- “Transport-neutral” means Core-owned semantic types that do not promise a server/Unity wire version, transport
  envelope, field naming compatibility or consumer rollout. Serde may encode an immutable application document for
  tests/adapters, but the Core type and exhaustive semantic kind own its meaning.
- `CoreApplication` is an application service, not a domain aggregate handle: it owns private authority and exposes
  only commands/queries. It must provide no inner borrow, dereference, raw-state conversion, commit token or rollback
  capability.
- The 58 player behavior meanings remain the behavioral oracle. A public `ApplicationCommand` is not an alias or
  re-export of `PlayerBehavior`; its payload is validated/mapped before the existing authority executor is entered.
- The 49 result identities are distinct application meanings. In particular, `BattleAdvanced`, `BattleState`,
  playback/deploy/withdraw/stance/relocate/swap/focus/activation results may share update content but cannot collapse to
  one unnamed public result.
- Existing result-contract types can support parity tests internally, but they are legacy public projection families
  and must not survive the hard cutover as public aliases.
- `GameError::code()` is an oracle, not the final public error type. `ApplicationError` owns stable code/detail and its
  conversion must name every domain error variant; `_` fallback or serialization-only identity is insufficient.
- Snapshot documents are read models, not authority. They are immutable, replace-whole values; callers cannot mutate
  the Core through them. Explicit null represents an absent selected event.
- Battle advance, admin, profile and checkpoint operations are application ports even when they are not one of the 58
  player behaviors. They must return the same application result/error/read-model vocabulary before `world` becomes
  private.
- Active-run transaction rollback is not a consumer feature. Publishing it would expose mutation authority, so P07
  closes it instead of wrapping it.
- Default-debug/test battle constructors remain P04 test capabilities and are not the P07 public application root.
  P07 does not reopen or broaden them, and default-release privacy probes remain required.

## Evidence Rules

- Exhaustiveness needs both compiler-enforced matches and a cardinality/identity corpus. Merely serializing an enum
  through `serde_json::Value` does not prove every meaning has a deliberate application mapping.
- Semantic parity compares result kind, payload, error code/detail, snapshot value, event/revision ordering and next
  decision. Matching only JSON parse success or variant count is insufficient.
- Invalid command decoding must happen before authority access. The exact pre/post snapshot plus event/revision/durable
  intent and retry result must match.
- Result projection is prepared before commit. A post-commit delivery/diagnostic issue may be represented only as an
  accepted recovery-pending result, never as `Err`.
- Large boundary evidence uses existing deterministic Core limits/corpora. P07 must not invent an arbitrary snapshot
  truncation or byte cap; P08 owns measured serialization/memory guards.
- Producer-consumer means an actual independent Core compile probe calling the façade. It does not mean inspecting or
  claiming `game_server`/Unity behavior.
- Legacy absence must be demonstrated from default-release compilation and static API locators: no public `GameCore`,
  raw behavior/result, rollback token, domain-result serde entry or dual façade.
- GEX-20 will audit an immutable target in a separate fresh pass. Any defect returns to correction GEX-10; it is not
  edited silently into a passing verdict.

## GEX-10 Resolutions

- The actual result cardinality is 49. The old 47-case test was incomplete evidence, not authority to erase the two
  Mastered Elite result meanings.
- A result kind identifies the established Core payload meaning; disposition identifies whether this invocation
  mutated authority. Consequently read-only reconnect may correctly be `BattleAdvanced + ReadOnly`.
- Release rustdoc plus an independent compile-fail probe is the public-surface oracle. A `pub` item lexically inside a
  crate-private module is not an exposed API, while a test-only rollback helper remains deliberately available to the
  internal transaction oracle.
- The manifest's legacy inventory must remain verifiable after removal. A removed target now requires both absence of
  its old locator and presence of its canonical replacement, instead of preserving stale source text merely to satisfy
  the verifier.

## Current Correction GEX-10

- Trigger: Master `GEX-30` Attempt 2 / `ATR-CRAR-C2-MASTER-GEX30-001` returned `Not Complete` after direct source
  review found `has_event_log: cfg!(any(test, debug_assertions, feature = "test-fixtures"))` in the selected-event
  public projection while P08 structural evidence claimed runtime test switches were absent.
- Owner: P07 / `WP-CRAR-C2-08`; this is an in-scope public-projection representation defect. No policy, public field
  meaning, Phase/Goal structure, external schema or consumer decision changes.
- Correction: retain the approved compile-time debug/test/`test-fixtures` combat-trace availability boundary with
  mutually exclusive `#[cfg]` constants, and remove the runtime `cfg!` switch from the projection path.
- Configuration behavior remains unchanged: default release reports no trace capability; debug/test/fixture builds
  report the capability compiled into those builds. A replacement target and fresh P07 GEX-20 are required.
- Current result: correction validation passes, and affected P01/P03/P05/P06 predecessor audits have fresh
  `Complete / High` records. P02/P04 targets were unchanged and remain current. Replacement P07 target freeze is next.
