# R03 OwnedHashMap review 008

Result: no concrete defect was established in the captured OwnedHashMap implementation and its first four private-map consumers. This is a bounded source review, not approval of unfinished whole-clone coverage or a completion audit.

Scope: core_enhanced/src/owned_collections.rs; EmployeeRoster in game/employee.rs; EquipmentMaterialInventory, EquipmentInventory and ConsumableInventory in game/resources/inventory.rs; crate-private module declaration in lib.rs. No repository/source/Git/build/test mutation or execution was performed. Only this report and its JSON companion are written.

The four workspace sources were captured at 2026-09-10T08:36:43.592353Z and rehashed at close. Root reported INPUTS038 owned_collections 4, inventory 5, employee 17 and checkpoint 44 tests passed. This helper read test source but did not rerun those tests or independently verify their output.

## Source-backed accounting review

OwnedHashMap:14–38 stores saturating lifetime allocation credit and computes an Option bucket bound. Credit zero gives an empty bound. For nonzero P, checked 2P, checked multiplication by 8, division by 7 and checked next-power-of-two with minimum 16 match the pinned model. Overflow returns None; it does not wrap into zero. The current mutation surface has no reserve/import/extend path other than fresh Deserialize, so this model's construction assumptions match the exposed operations.

The pinned std/hashbrown connection was inspected directly:

- Rust 1.91.1 std HashMap::clone delegates to the underlying map Clone (local /tmp/r03-rust-1.91.1-std-hash-map.rs:1301–1315). RawTable Clone copies the retained source bucket allocation rather than compacting entries (/tmp/r03-hashbrown-0.15.5-raw-mod.rs:3155–3185).
- In raw/mod.rs:2627–2681, growth requests are at most twice the post-reservation logical item count; capacity_to_buckets at 103–161 applies the 8/7 load adjustment and power-of-two rounding. The small-table result is at most 16.
- insert at OwnedHashMap:77–81 credits before forwarding exactly once. This also covers replacements: pinned hashbrown map.rs:1790–1816 uses find_or_find_insert_slot, whose raw implementation reserves before lookup. Crediting replacements is therefore a defensible conservative rule, not an omitted-path workaround.
- std entry uses base.rustc_entry, not hashbrown's ordinary entry (/tmp std source:887–889). The exact pinned rustc_entry source at 34–52 searches first; Occupied has no reserve, while Vacant reserves one slot before returning. OwnedHashMap:83–88 calls this native operation once, credits the returned Vacant, and returns the same Entry. On normal return, the credit covers the possible reservation even if the caller never inserts a value. It is recorded before a later or_insert_with factory runs. Occupied chains do not add unnecessary credit and cannot add another key through that Entry.
- The four current consumer maps use Uuid/String keys and the default RandomState; this review does not generalize to hypothetical future raw mutation APIs or arbitrary key/hasher behavior.

The exact rustc_entry source URL is https://raw.githubusercontent.com/rust-lang/hashbrown/v0.15.5/src/rustc_entry.rs. Root's saved file /tmp/r03-hashbrown-0.15.5-rustc_entry.rs has SHA-256 9c189a957af1ec5ff3549d15176c1608ca1c68330604a50b2ea19316a5dd14e2. This helper also independently found the existing local hashbrown-0.15.5.crate archive and read its rustc_entry.rs member in memory, without extracting/installing it. The archive SHA-256 is 9229cfe53dfd69f0609a49f65461bd93001ea1ef889cd5529dd176593f5338a1, matching the Rust 1.91.1 Cargo.lock checksum. Its rustc_entry member matches root's saved file. An earlier availability concern about this member is resolved; no neighboring 0.15.4 source is used as the pinned proof.

## Mutation and state preservation

| Reviewed path | Conclusion and exact spans |
| --- | --- |
| Empty construction | Default creates one ordinary HashMap::with_hasher(S::default()) and zero history; new delegates to it (owned_collections:47–59). No extra map rebuild or hasher reset is introduced. |
| clear/remove | clear forwards without resetting history (67–69); remove forwards without resetting history (98–103). Retained buckets and tombstones therefore remain covered. |
| Values mutation | get_mut uses the original Borrow lookup (91–96); values_mut and mutable IntoIterator return mutable values, with no mutable keys or raw map handle (71–73,158–163). They cannot grow this map's allocation. Payload-owned collections remain a separate visitor concern. |
| Native Clone | Derived Clone copies inner through std HashMap::clone and copies the history (41–45). It does not enumerate/reinsert, shrink, sort, or replace the source hasher. |
| clone_from | No native inner.clone_from override exists. The trait's default replaces the wrapper with a Clone, so it cannot pair a small source history with a retained large destination table. The test at 196–200 specifically checks this behavior. |
| Equality | PartialEq/Eq compare inner only (121–126). Different allocation histories do not change gameplay map equality. |
| Debug | Debug delegates directly to inner (115–119), preserving the original map-shaped Debug output and omitting bookkeeping. |
| Serialize | Serialize delegates directly to inner (128–132), preserving entry enumeration and map shape; history is not a serialized field. |
| Raw access | Only immutable Deref exists (106–113). No DerefMut, AsMut/ BorrowMut, mutable raw-map callback, raw constructor/import, or mutable field exposure was found. The native Entry permits only the already credited vacant insertion or occupied value operations. |

MapCloneHistory methods are crate-private, and its allocation_credit field is private. OwnedHashMap's inner/history fields are private. The module itself is private in lib.rs:4. Returning mutable values from public EmployeeRoster/inventory methods preserves their previous capability; it does not leak map-membership mutation.

## Fresh Deserialize and public consumer shape

OwnedHashMap:134–147 delegates to the existing HashMap Deserialize and moves that same result into the wrapper. It seeds history from the resulting fresh capacity, including unused reservation. This is not the forbidden use of current capacity after arbitrary deletion churn.

Pinned serde_core 1.0.228 de/impls.rs:1536–1548,1565–1571 constructs a fresh HashMap with cautious size-hint reservation and then only inserts entries. It does not delete entries. Duplicate keys replace values, and any extra allocation caused by insertion is still represented by the final no-tombstone capacity. Seeding this fresh result is consequently consistent with the source model. There is no zero-default skipped history after deserialization and no serialized history trusted from input.

The current consumer changes preserve their externally usable signatures:

- EmployeeRoster's employees field remains private (employee.rs:635–638). new/clear/add/get/get_mut/contains/len/is_empty/iter/iter_mut retain their previous signatures and operations (641–681). add reaches tracked insert; clear preserves history; later skill-fragment methods mutate Employee values through get_mut (700–735).
- EquipmentMaterialInventory.stacks is private (inventory.rs:456–460). add uses tracked Entry (469–476); consume mutates a value and removes the key when exhausted (479–496).
- EquipmentInventory.items is private (508–510). duplicate checking precedes the same insertion call; tracked insert only runs on the existing successful path (527–536). Removal and mutable item access use the controlled wrapper methods (539–549).
- ConsumableInventory.items is private (587–589). Existing stack updates use get_mut, new stacks use insert, and exhausted stacks use remove (609–635,653–679). Remaining lookups/iteration keep immutable access.
- Outer owner derives Serialize/Deserialize/Debug/Clone remain in place. Wrapper serialization is map-shaped, so the private field remains the same serialized field rather than becoming an inner/history object. Inventory's other temporary HashMap at slot ownership validation remains an ordinary temporary map, appropriately outside this conversion.

The source supports shape preservation; this review does not independently prove every V6/profile fingerprint or whole-application lifetime property. Root is exercising the existing consumer/checkpoint tests separately.

## Test review

The four wrapper tests are meaningful behavioral checks:

1. owned_collections:171–200 checks history across churn/clear, native clone capacity and enumeration preservation, raw-equivalent Debug/JSON, history-independent equality, and clone_from replacement.
2. 204–255 compares native hash invocation counts and factory behavior using the same hasher, exercises occupied Entry chains, verifies occupied Entry does not add credit, and checks get_mut/values_mut/mutable iteration.
3. 259–275 supplies a 256-entry size hint with duplicate keys collapsing to one. This specifically catches an unsound len-only Deserialize seed; it also checks JSON hides history and the restored map compares equal.
4. 279–288 checks saturated history survives ordinary insert/remove/Entry/clear/Clone while bucket_bound stays None.

Assertions that bound >= map.capacity() are useful sanity checks but do not by themselves prove a retained-bucket upper bound, because current capacity is a lower bound. The algebra and exact pinned allocation/Entry source above provide that justification. The tests must not be described as direct native bucket introspection or a full heap proof.

One optional focused regression check would cover the source nuance that a dropped, unused Vacant Entry can reserve storage while len remains zero. The current implementation already handles it by crediting every returned Vacant; no code defect is asserted because that test is absent.

## Boundaries

No concrete undercount, untracked current mutator, unintended raw mutable escape, or ordinary public-shape/Clone/equality change was established in this captured scope. There is no favorable completion verdict for the unfinished whole-owner visitor: returning a bound is separate from aggregating/admitting it at every required copy boundary. That future integration, additional collection owners, source/candidate coexistence, and exact allocator/RSS bytes are outside this review.

All findings/limitations apply to the captured version; any successor method surface needs its own source check. Exact source hashes, spans, archive provenance, and close-time drift are recorded below and in the companion JSON.

## Source ledger


Close UTC: 2026-09-10T08:42:03.929162+00:00. Initial four-file capture-to-close drift: none.

| Source | SHA-256 at capture, or close-only where specified in JSON |
| --- | --- |
| /mnt/f/work/simulator/core_enhanced/src/owned_collections.rs | 42be370f17630796d1c92fbd561d6e14691346fb013711a00a94bfdc0a733989 |
| /mnt/f/work/simulator/core_enhanced/src/game/employee.rs | 7e755c27aeef108f2c328ed2e3e25043054a41468664066e0c6d0963c3edb53d |
| /mnt/f/work/simulator/core_enhanced/src/game/resources/inventory.rs | 5c429345a210ffc303d7274c3f6bfb5bdb8d053091638d5a7af3f946497fbcc6 |
| /mnt/f/work/simulator/core_enhanced/src/lib.rs | f1f31fa710b8103123030294faf0d5c81c9400ff252e04ec5bcc6ba1caa0a4b7 |
| /tmp/r03-rust-1.91.1-std-hash-map.rs | 628357909f9fa43cfe47e03f315c868b8115483079377d964e7758b106e7c6b1 |
| /tmp/r03-rust-1.91.1-Cargo.lock | b89b588a745aa7b8e57a31dd3d674eeb2d385510da9710483cff80200313027e |
| /tmp/r03-hashbrown-0.15.5-raw-mod.rs | a88cd291be71cd2f6119a1f8b4030a0429b6b1f8771b7f2a8b140c1824788bed |
| /tmp/r03-hashbrown-0.15.5-rustc_entry.rs | 9c189a957af1ec5ff3549d15176c1608ca1c68330604a50b2ea19316a5dd14e2 |
| /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_core-1.0.228/src/de/impls.rs | 9fb7a0322af51971fe1fa8f5b56c7ca3716622a6e9127b86bddfc0742c11383c |
| /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.4/src/map.rs | 072f998a2898e0847a333b1e94a95be1ca28766a8330d923e60073501a9036f2 |
| /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.15.4/src/rustc_entry.rs | 9c189a957af1ec5ff3549d15176c1608ca1c68330604a50b2ea19316a5dd14e2 |
| /home/acr0tic/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/hashbrown-0.16.0/src/map.rs | bf069e9f391abbbfca0918f5798229fd44380afc4f126ce64afd387b5401a8e3 |

Archive/member hashes are also recorded in JSON. The .crate file was read in memory only.
