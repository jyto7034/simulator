# R03 BTree native clone design010

Bounded read-only source investigation under AUTORUN §8; not a completion audit. This helper changed no repository/source/Git files and ran no cargo/build/tests. Eleven exact Rust 1.91.1 source files were downloaded to new /tmp paths and inspected. The proposal concerns logical storage slots and structural copy work, not installed allocator bytes, padding, RSS or a complete heap proof.

## Recommended O(1) bound

For a normally valid BTreeMap/BTreeSet observed between operations, let n = len(). The native clone destination has at most:

```text
Nclone(0) = 0
Nclone(n > 0) = 1 + (n - 1) / 5       // integer division

slots_clone(n) = checked(40 * Nclone(n) + 1)
```

The +1 denotes **one outer collection/root-descriptor record**, matching a logical collection accounting convention. It does not mean that the native root consists of one scalar machine field. If the meter counts every outer scalar independently instead, give the outer descriptor its own typed-field charge.

Root's proposed 40 slots per node is conservative: the exact internal-node field capacity totals 37 typed slots. Key/value children must still be visited separately. This bound needs no capacity query, tree traversal, mutation history, allocator cache or re-insertion. Use the existing native Clone after preflight.

For already-retained source storage, use Nsource(0)=1 conservatively, because an empty source can retain one leaf. For n>0 use the same N bound. If source+candidate coexistence is being counted, add their bounds separately; a clone-destination-only limit must not be presented as that combined limit.

Examples (conservative, not exact allocation counts):

| len | Nclone bound | 40*Nclone+1 |
| ---: | ---: | ---: |
| 0 | 0 | 1 |
| 1 | 1 | 41 |
| 6 | 2 | 81 |
| 11 | 3 | 121 |
| 100 | 20 | 801 |

For len=6, a real valid tree can fit in one leaf; the formula intentionally overcounts. This is the smallest simple occupancy-derived expression proposed here, not a claim of the tightest achievable node count for every n.

## Why len bounds native nodes

The pinned BTreeMap source requires at least one element in each internal node and at least MIN_LEN in every non-root node; empty maps have no root or one empty leaf (map.rs:29–39). node.rs:42–44 sets B=6 and MIN_LEN_AFTER_SPLIT=5. For a nonempty tree with N nodes, its root contributes at least one key and the other nodes at least five:

```text
n >= 1 + 5*(N - 1)
N <= 1 + floor((n - 1)/5)
```

This is a bound over all normally valid histories, including deletion and clear, rather than only freshly populated trees. len itself reads the stored length (map.rs:2627–2628). The public std collection names re-export alloc's implementations (std/collections/mod.rs:421,425). Sources: [map.rs](https://raw.githubusercontent.com/rust-lang/rust/1.91.1/library/alloc/src/collections/btree/map.rs), [std re-export](https://raw.githubusercontent.com/rust-lang/rust/1.91.1/library/std/src/collections/mod.rs).

## Exact node field accounting

The pinned node definitions contain these retained typed locations:

| Node storage | Slots |
| --- | ---: |
| parent: Option<NonNull<InternalNode>> | 1 |
| parent_idx: MaybeUninit<u16> | 1 |
| len: u16 | 1 |
| keys: [MaybeUninit<K>; 11] | 11 |
| vals: [MaybeUninit<V>; 11] | 11 |
| Additional internal-node child pointers | 12 |
| Maximum total per node | **37** |

A leaf has 25 of these slots. An internal node embeds the leaf data and adds its 12-edge array. The structs allocate as individual boxed nodes; there is no hidden dynamic node-capacity vector in these definitions. The outer NodeRef carries height and a node pointer, while the map owns root, length and allocator fields. Thus 40 is a source-based upper bound for **node typed slots**, with three spare units, rather than a guessed byte multiplier. References: node.rs:50–65,80–85,97–119,182–198. [Exact node source](https://raw.githubusercontent.com/rust-lang/rust/1.91.1/library/alloc/src/collections/btree/node.rs).

One K or V slot can itself contain a large inline record or own nested allocations. The 37/40 count neither measures that record's byte width nor bounds its child storage. Visitor accounting must retain the distinction between this native node capacity and recursively owned key/value data.

## What native Clone does

BTreeMap Clone recurses over the source tree. A leaf creates one new leaf and clones its occupied key/value pairs. An internal node first clones its first subtree, adds one internal level, then clones each key/value and following subtree. Those subtrees are moved into the destination; no sorted re-insertion, comparison, compaction or old-capacity cache is involved. For a valid nonempty source, this creates one destination node per source node. An empty source instead returns new_in with no root, even when that source retains an empty leaf. Source spans: map.rs:223–305; node creation/push helpers at node.rs:579–595,654–697.

Consequently, destination node allocations accumulate toward the completed tree rather than requiring an additional full temporary tree on top. Temporary cloned keys/values, recursion frames and their own allocations still require the surrounding visitor/work model; this report does not assign them zero cost. Native Clone does not offer cancellation callbacks inside those loops. Budget and cancellation admission must occur before calling it, with finite leaf bounds supplied separately.

BTreeSet wraps BTreeMap<T,SetValZST> and delegates Clone (set.rs:78–83,117–124). The marker is zero-sized (set_val.rs:1–6), so using the same 40-node bound for sets deliberately overcounts its value slots and avoids a new representation branch. Visit each actual set element's children once. Sources: [set.rs](https://raw.githubusercontent.com/rust-lang/rust/1.91.1/library/alloc/src/collections/btree/set.rs), [set marker](https://raw.githubusercontent.com/rust-lang/rust/1.91.1/library/alloc/src/collections/btree/set_val.rs).

BTreeMap does not override clone_from in this Clone impl. Its default replaces self with source.clone() (core/clone.rs:245–249); BTreeSet's clone_from delegates to that map method. Therefore there is no separate native destination-capacity retention rule to model here, although replacement briefly coexists with the old destination and its destruction. [Default Clone implementation](https://raw.githubusercontent.com/rust-lang/rust/1.91.1/library/core/src/clone.rs).

## Deletion, empty state and construction

Deletion repairs underfull nodes by merging or stealing, then handles affected ancestors (remove.rs:25–74; fix.rs:8–71). OccupiedEntry removal decrements length and pops an emptied internal root (map/entry.rs:603–614). ExtractIf uses the same removal mechanism and root-pop callback (map.rs:2028–2040); retain delegates to extract_if (1165–1170). Merged sibling nodes and popped internal roots are deallocated rather than kept in a free-list (node.rs:607–623,1384–1444). Sources: [remove.rs](https://raw.githubusercontent.com/rust-lang/rust/1.91.1/library/alloc/src/collections/btree/remove.rs), [entry.rs](https://raw.githubusercontent.com/rust-lang/rust/1.91.1/library/alloc/src/collections/btree/map/entry.rs).

fix_top removes empty internal levels but can keep an empty leaf. This is why a source with len=0 needs a one-leaf retained-state bound if source storage is counted. Fresh new/new_in have root=None; clear replaces the map root with None and drops the former tree (map.rs:644–645,663–670). Unlike hash-table tombstone accounting, no lifetime insertion credit is needed for valid BTree nodes. [Repair source](https://raw.githubusercontent.com/rust-lang/rust/1.91.1/library/alloc/src/collections/btree/fix.rs), spans 74–99.

split_off temporarily creates empty levels, then repairs both borders before returning (split.rs:29–78). Bulk construction similarly fixes the right border on ordinary completion (append.rs:39–94). Those temporary shapes are not valid points for applying the non-root minimum occupancy invariant. The finished states satisfy the relevant tree invariant; normal deserialization through completed insertion/build APIs does not need historical metadata. Sources: [split.rs](https://raw.githubusercontent.com/rust-lang/rust/1.91.1/library/alloc/src/collections/btree/split.rs), [bulk construction](https://raw.githubusercontent.com/rust-lang/rust/1.91.1/library/alloc/src/collections/btree/append.rs).

## Preconditions and limits

- Observe an ordinary valid collection between operations, through its immutable borrow. Non-root minimum occupancy is explicitly allowed to be temporarily violated inside methods (map.rs:29–30). Internal transient trees, unsafe corruption and states with violated collection invariants are outside the proof.
- Freshness alone is not required: previously mutated or imported collections qualify if their normal construction/mutations completed and preserved the invariant. Conversely, do not certify arbitrary panic-recovered or logic-error states merely because a len method exists. bulk_push repairs its right border only after its loop (append.rs:94), so blanket claims about all possible caught panics need an additional proof. The application's closed, known-leaf flow is the practical seam.
- This bound covers the standard Global-allocator node graph. A generic user-supplied allocator's Clone can have arbitrary extra storage/work; likewise arbitrary key/value Clone implementations. Such types need separate admitted leaf implementations, not a larger unexplained coefficient.
- The formula does not bound leaked/unreachable allocations left by external unsafe code, allocator bookkeeping, resident memory, recursive stack bytes or all transitive key/value storage.
- The proof is pinned to Rust 1.91.1's implementation. Changed branching constants, node layout, allocator support, clone strategy or repair semantics require source-model review.

## Checked arithmetic and integration

Guard n==0 before subtracting one. For positive n, 1+(n-1)/5 is representable for any usize input, but using checked_sub/checked_add keeps the contract explicit. Then use checked_mul(40) followed by checked_add(1), and checked conversion into the meter's counter type. Never saturate the admission result into a seemingly valid limit; overflow means this request cannot be admitted under the bounded accounting model.

For a visitor measuring new clone storage, the empty case adds its one outer collection record and no destination node slots. For a visitor measuring existing retained state, include the possible empty source leaf as well. Iterate live entries for child accounting and polling only after the O(1) structural bound has been admitted. This keeps a large collection rejectable without first walking it.

No collection representation changes, public API changes, serialization-based storage inference or alternative clone implementation are proposed. Root's 40*N+1 formula is supported under the explicit logical-unit and valid-state conditions above; full leaf coverage and whole-copy coexistence remain separate obligations.

## Source capture

All URLs target the Rust 1.91.1 tag. Primary seven files were captured at 2026-09-10T09:07:12.537363Z. Four support files were downloaded later and first hashed with all files at 09:10:18.834122Z; the primary seven had no drift. Final report JSON records the later close rehash, complete URLs, spans and exact SHA256 for every downloaded input. Browser cache failures were followed by authorized direct downloads; reported line spans come from the downloaded bytes, not browser-rendered line numbering.

This is source analysis only. No repository implementation or test result is asserted.


Exact downloaded source hashes (URLs and inspected spans are also preserved in the JSON):

| Source | SHA256 |
| --- | --- |
| append.rs | 7e60bcd1ad1a38ae9b4c56e4a81e79a86d8da7c839f6cb9467d44b20b60a83bf |
| core-clone.rs | b78f36893a7f5a1d11a52da80a02cff698239a8db561d6a4ff012ce33709de7f |
| fix.rs | 690f95d7744400ef22f3ef23947d671c838944de0b8c9cb031455fe8efd0d71c |
| map-entry.rs | 4f2d085ec408e96bd5d9ea9bb8791887795240e6b51097c26686641ad3fa0ce4 |
| map.rs | 1072108d0922fec720a02ae0668137b6b82c15639305bfbd400c45e295761969 |
| node.rs | 66fccbf417491db57ebd7f489a3e40809b8f1435b589715d8b00ace1e20bc177 |
| remove.rs | 498bdc571e9a3426a24fd33131a087e57c900194c4dcc6cfa2cf5c0fa388fa68 |
| set-val.rs | 354ab9c955b7f068a9bb7727357aa66b104acc176170ccf37b01945d302d8ec1 |
| set.rs | 88a9e255fdc0ce291dff93268f549d2100c46ab2ca32337fb3d59aadbacbb68e |
| split.rs | 0b67b3eaad2b43825664ea8809b113a36e32fa8462d2b20fd4146f8224aad45a |
| std-collections-mod.rs | 783e035d7049ba5f9de313d5c17b14bc6e1dbf5b4f03c6996873b2aa8465b703 |

Final close rehash: 2026-09-10T09:13:44.417955+00:00. Source drift: false.
