# Determinism, Identity And Continuation Map

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Baseline: `BASELINE-20260822-064427-KST`
- Primary evidence: E-006~E-017, E-019~E-021

## Assessment

Determinism is designed into the current runtime rather than added only in tests. Seeded generation, namespaced UUID streams,
stable ordering before HashMap-derived outputs, ordered battle events and strict checkpoint validation are valuable assets. The
public continuation probe reproduced both restored state and the next two observable results. The remaining gap is breadth:
there is no canonical command-tape equivalence suite spanning future Relic/corrosion/boss flows or cross-platform Rapier/float
behavior.

Provisional A10 verdict: `PARTIAL` with High confidence in current seeded run/checkpoint semantics.

## Authority And Identity Streams

| Concern | Current mechanism | Preservation requirement | Risk/change rule |
| --- | --- | --- | --- |
| run/map generation | run seed plus stable namespaces/text seeds; explicit candidate sorting/shuffle | same policy inputs produce same graph/order/IDs where contract requires it | adding unrelated random draws must not shift another subsystem |
| application UUID | `UuidManager { run_seed, counters: BTreeMap<namespace,counter> }` | stable identity and namespace independence | new feature receives a named namespace; observation may not consume authority IDs |
| battle identity/RNG | battle/scenario seed, deterministic UUID helpers, explicit sequence/cause IDs | same initial state/commands yield ordered events/final state | keep scheduling/tie-break ordering explicit before map iteration/physics results |
| event ordering | monotonic battle `seq`, typed parent/root cause, versioned append-only log | reconnect and result oracles compare order and identity | archive/index may change storage, not public order/cause semantics |
| checkpoint | strict V4 payload owns map/progression/UUID and current run facts | validate entire graph before mutation; restore exact run seed | target schema may change only through validated conversion |
| monotonic exceptions | profile knowledge, load count/run peak according to selected contract | checkpoint rewind must not erase permanent/global facts | declare exceptions in transaction/continuation oracle |

## Existing Evidence

- Facility generator tests prove same seed/state reproduces assignment and IDs and that authored Standard layout is
  seed-independent where policy says so.
- Movement, spawn, wave, summon, cast and result paths commonly sort stable IDs/coordinates or use `BTreeMap/BTreeSet` before
  order becomes observable.
- `UuidManager` separates counters by namespace and has tests showing checkpoint-local rewind versus named monotonic streams.
- Strict checkpoint tests compare exact V4 shape, reject 19 single-edge graph corruptions before destination mutation, compare
  restore-recapture semantic equality and verify deterministic gate replay.
- E-021 exported a public SavePoint from a live Standard run, continued into a fresh core created with a different seed, and
  observed equal run snapshot, equal next-node preview and equal node-confirm result.

## Canonical Equivalence Representation

For a migration or subsystem replacement, compare:

1. normalized run/profile snapshot with explicit exclusion or max-merge rules for monotonic metadata;
2. command result/error and capability projection;
3. ordered battle event `(version, seq, cause, source_command_id, semantic payload)`;
4. battle/run terminal result and deterministic receipt IDs;
5. a new export followed by another continuation command, so equivalent projections cannot hide divergent authority.

Raw internal structs and HashMap serialization order are not the canonical oracle. Exact raw bytes remain required only for a
declared wire/save contract version.

## S10 Continuation Scenario

Current disposition: retain the strict checkpoint validator, seed/UUID authorities and public export/continue seam; refactor the
checkpoint representation behind versioned aggregate DTOs and add a command-tape differential oracle. A whole-core rewrite
would discard the very mechanism needed to prove equivalence and is therefore higher risk than staged replacement.

Target command tapes should include at least:

- uninterrupted vs continued run through node selection, event/shop/reward and next SavePoint;
- battle setup/commands/result after continue, including event ordering and reconnect cursor;
- Monolith plus future Relic/form/corrosion state across checkpoint and explicit load;
- failure/rollback followed by retry, proving IDs and receipts do not drift;
- archive/diagnostic enablement on/off, proving observation does not consume authority state.

## Remaining Uncertainty

- Cross-platform bitwise Rapier/float equivalence was not measured. Server-authoritative semantic equivalence may be sufficient,
  but the target guarantee must be chosen (GAP-002).
- The probe exercised one noncombat future sequence, not long battle/projectile/effect or all future mechanics.
- Strict V4 has no migration reader. This is acceptable before release but requires an explicit importer/equivalence gate when a
  target schema is adopted.
- Product event-retention changes must keep every cause/receipt needed for result computation and reconnect continuity.
