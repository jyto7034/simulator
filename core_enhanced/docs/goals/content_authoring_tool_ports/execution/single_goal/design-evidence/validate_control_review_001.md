# R03 validate/control 읽기 전용 보조 검토 001

Two concrete diagnostic source-retention defects; no concrete validate RAII/nesting/arithmetic/A-token/fingerprint defect found in inspected paths. No completion or runtime validation verdict.

검토 대상은 현재 도입된 authoring::validate/control, content metadata/diagnostics, queue wrapper 및 generated setup 연결이다. Root가 구현 중인 runtime leaf coverage와 미구현 simulate/trace/CLI는 findings로 세지 않았다. 이 문서는 GEX-10 보조 검토이며 완료 감사가 아니다. 코드 수정·시험·build·runtime 실행은 0회다.

## 구체적 findings

### VCR01 (P2) — Final snapshot recheck drops an already known pack identity

Source: `core_enhanced/src/game/data/content_pack/snapshot.rs` lines 359–373; producer identity assignment288–290; SHA-256 `b7d729e6cc18d878220f781f7e501f5867baf0ef7c96cf6d756c44f210652229`.

Trigger: A catalog or manifest is replaced, changed, removed or becomes unreadable after initial acquisition and before Inventory::finish reopens it.

Observed: The failure has the actual path and Acquisition phase, but pack_id is None. FileRecord.pack_id was already assigned from the successfully decoded/validated manifest. The propagated finish error is returned before SourceReceipt exists and never reaches bind_diagnostic.

Expected: Preserve the already observed manifest identity as the candidate snapshot owner on recheck failure; do not infer a new identity from changed bytes or directory name.

Minimal fix: Bind record.pack_id at both the root.read map_err and changed-file error producer. Preserve catalog kind too where known, by recording that acquisition provenance if necessary. Unknown pre-manifest identity remains None.

Suggested focused check (not run): Extend existing acquire_with_hook race tests (snapshot/tests.rs236–277) to assert code/phase/path/actual original pack_id for changed contents, same-byte replacement and reread/no-follow failure. Also keep malformed first manifest pack_id=None.

### VCR02 (P2) — Aggregate writer-resolution failures discard all known contributing source addresses

Source: `core_enhanced/src/game/data/content_pack/resolver.rs` lines 358–359,400–406,426–432; Writer source25–31,278–289,310–318; wrapper content_pack/mod.rs111–112; SHA-256 `6e330f4ed46a6f7ee149902eb925c15351894014c35d9cea1f984a4211aaf400`.

Trigger: Two maximal writers require a selection, duplicate Add writers exist, or the writer graph is disconnected/cyclic.

Observed: The new diagnostic preserves target kind/id and Resolution phase but has source_file="selection", pack_id=None and an empty related_sources array. Every participating Writer already owns its actual pack/file/field source. SourceReceipt::bind_diagnostic can only enrich existing exact paths and cannot recover contributors from this aggregate address.

Expected: Leave aggregate primary pack absent, while attaching bounded actual contributing writer sources so the consumer can locate the conflicting inputs without reopening files or parsing the cause.

Minimal fix: At these aggregate producers construct related_sources from the relevant actual Writer.source records (heads for ambiguity; Add writers for duplicate base; relevant graph writers for connectivity). Deduplicate deterministically and preserve target DefinitionKind/CatalogKind. Do not pick an arbitrary primary pack.

Suggested focused check (not run): Use two replacement heads and duplicate Add sources with different manifest identities/paths; assert Resolution phase, target, no fabricated primary pack, and exact related pack/file/operation-field rows through authoring::validate and plain load. Existing per-writer error tests do not cover this aggregate path.

## 확인한 경계와 한계

- **RAII and nested scopes**: No concrete defect found in current synchronous validate path. ScopeGuard is constructed immediately after ACTIVE installation and before entry poll; early Err and unwind drop it. Nested run returns before replacement/guard installation, so ignored nested failure cannot reset the outer budget. Limits are validated before nested detection; this affects error precedence but not scope ownership. Phase guards restore in lexical reverse-drop order. No RefCell borrow is held across operation or serialization callbacks. Source: authoring/control.rs134–240.

- **Cancellation/sticky error precedence**: Existing sticky execution failure wins later polls and cancellation; cancellation is checked before each newly attempted charge/admit and at acceptance. An execution fault is recovered as structured AuthoringError::Execution even where intermediate domain code wraps/swallow its GameError. If cancellation first appears at acceptance it can supersede an operation-returned Content error; current code makes that precedence explicit through run ordering, and no contradictory approved rule was established in this limited review. CancellationToken is host-owned and not retained by AuthoringContent. Source: authoring/control.rs147–179,202–240; authoring/mod.rs103–114.

- **A/B externally retained state**: validate does not take an existing content/application mutably; candidate loader state is local, and AuthoringContent owns only its token and usage. Failed validation drops the local candidate and control scope. Source metadata is immutably borrowed through the owning generation Arc; no cycle or global metadata retention found. This only establishes current validate/token ownership, not future trial-session rollback. Source: authoring/mod.rs47–114; data/mod.rs224–278; content_pack/mod.rs124–143.

- **Limit arithmetic and metadata units**: charge/admit use checked addition and inclusive >limit rejection. Metadata Counter admits each JSON writer chunk before incrementing, so its encoded byte total has no arithmetic bypass found. It counts the serialized (source table,writers) tuple, not heap/capacity/source-receipt bytes. SourceTable and writer records are allocated before this serialization count; the metric must not be presented as preallocation or whole-process heap enforcement. Definitions peak is the maximum admitted selected-raw/final count, not measured allocation count. No encoded-cap arithmetic bug established. Source: authoring/control.rs90–123,255–352; content_pack/mod.rs84–125; content_pack/raw.rs238–259,574–580.

- **Queue and generated setup bridge**: The wrapper preserves BinaryHeap order and suppresses a rejected push while leaving sticky failure in the attempt. In the inspected generated startup path enqueue_scenario_events polls after each enqueue, startup polls before success, setup polls before return, and run polls before acceptance; suppressed enqueue cannot publish a successful validate result through that path. All helpers are no-ops outside an ACTIVE scope, preserving ordinary push success semantics. Broader runtime leaf coverage was intentionally not audited. Source: battle/core/queue.rs7–34; battle/core/sim.rs1489–1515,1790–1827; events/combat.rs142–221; data/validation.rs1492–1621.

- **Fingerprint and ordinary successful loading**: Current canonical resolver input remains ContentContract, pack identities, WriterReceipt semantic fields and final definitions in existing sorted slots. New retained SourceTable is outside that hash and generation metadata has no mutable public accessor. New scope checks are inactive on ordinary calls. No static fingerprint or ordering regression found; authoring/tests.rs24–62 asserts parity but was not executed by this helper. New retained metadata costs remain a measurement obligation for root. Source: content_pack/resolver.rs43–66,435–460; content_pack/mod.rs120–143; data/mod.rs259–262.

## 실제 읽은 소스와 drift

Root의 source 변경이 진행 중이므로 단일 저장소 freeze를 주장하지 않는다. Main source는 첫 읽기 전 해시와 보고서 종료 해시를 대조했다. 보충 파일은 표기한 시점에만 해시를 확보했다. 읽기 범위는 해당 해시 파일 전체의 의미 감사를 뜻하지 않는다.

- `core_enhanced/src/authoring/mod.rs`; inspected 1–118; first `fe4a6d69b11d0fcedc4e49625c760850dd12517cfd8a3eb6ca1da1c03c941ca6`; close `fe4a6d69b11d0fcedc4e49625c760850dd12517cfd8a3eb6ca1da1c03c941ca6`; drift=false; Before main content read.
- `core_enhanced/src/authoring/control.rs`; inspected 1–459; first `4ecfe2d8f5109fc8eefef3be02fcf645b755788011ad093791ea00de044adf38`; close `4ecfe2d8f5109fc8eefef3be02fcf645b755788011ad093791ea00de044adf38`; drift=false; Before main content read.
- `core_enhanced/src/authoring/tests.rs`; inspected 1–161; first `e5491524dcd16e354cc560c12f8582e35b38c1d3cbd8b7bdaa8850f2ed640ad6`; close `e5491524dcd16e354cc560c12f8582e35b38c1d3cbd8b7bdaa8850f2ed640ad6`; drift=false; Before main content read.
- `core_enhanced/src/game/data/content_pack/mod.rs`; inspected 1–214; first `cdca9db0f33f6590406162db40c8a238fe195f55c5e4754383fb86065fa03fab`; close `cdca9db0f33f6590406162db40c8a238fe195f55c5e4754383fb86065fa03fab`; drift=false; Before main content read.
- `core_enhanced/src/game/data/content_pack/resolver.rs`; inspected 1–130, 188–464; first `6e330f4ed46a6f7ee149902eb925c15351894014c35d9cea1f984a4211aaf400`; close `6e330f4ed46a6f7ee149902eb925c15351894014c35d9cea1f984a4211aaf400`; drift=false; Before main content read.
- `core_enhanced/src/game/data/content_pack/raw.rs`; inspected 210–273, 562–691; first `e6db361ad983b1fcb4f0a33b827b99ec2ecf4f62dfbbd51e578dfa874b71b7d7`; close `e6db361ad983b1fcb4f0a33b827b99ec2ecf4f62dfbbd51e578dfa874b71b7d7`; drift=false; Before main content read.
- `core_enhanced/src/game/data/content_pack/snapshot.rs`; inspected 1–168, 220–431; first `b7d729e6cc18d878220f781f7e501f5867baf0ef7c96cf6d756c44f210652229`; close `b7d729e6cc18d878220f781f7e501f5867baf0ef7c96cf6d756c44f210652229`; drift=false; Before main content read.
- `core_enhanced/src/game/data/content_pack/schema.rs`; inspected 85–178; first `3c44c1cb3fc4df3ef5ebea29cfbdb939a3614735ec77e2391f2ee4b4a1fd0616`; close `3c44c1cb3fc4df3ef5ebea29cfbdb939a3614735ec77e2391f2ee4b4a1fd0616`; drift=false; Before main content read.
- `core_enhanced/src/game/data/live_ron.rs`; inspected 45–227; first `36a6fdcaa3542f552a6d9746b250c35170a023762e2ce54d2cba619947308529`; close `36a6fdcaa3542f552a6d9746b250c35170a023762e2ce54d2cba619947308529`; drift=false; Before main content read.
- `core_enhanced/src/game/data/mod.rs`; inspected 224–310; first `84d12f3cd0a9d3bbe7df5910ce4e83964fd3152a7409b1f0bbcb23b46cae4644`; close `84d12f3cd0a9d3bbe7df5910ce4e83964fd3152a7409b1f0bbcb23b46cae4644`; drift=false; Before main content read.
- `core_enhanced/src/game/data/validation.rs`; inspected 1480–1781; first `656e46763c9f35a5e3201992866a928664c62467a0936439027ea8584517eb80`; close `656e46763c9f35a5e3201992866a928664c62467a0936439027ea8584517eb80`; drift=false; Before main content read.
- `core_enhanced/src/game/events/combat.rs`; inspected 126–233; first `d88719417886eea018ae08a66c750884f163051f160d0b9b21ece3c1eddffc66`; close `d88719417886eea018ae08a66c750884f163051f160d0b9b21ece3c1eddffc66`; drift=false; Before main content read.
- `core_enhanced/src/game/battle/core/queue.rs`; inspected 1–44; first `a2b3edc11f43590fc368cf3c9ae7be8919055bdd3d5c2aa448d533a9372714d5`; close `a2b3edc11f43590fc368cf3c9ae7be8919055bdd3d5c2aa448d533a9372714d5`; drift=false; First hash captured after targeted content read; no earlier byte snapshot claim.
- `core_enhanced/src/game/data/content_pack/diagnostic_tests.rs`; inspected 1–240; first `44f71a76740ea775834c3101b989c3bb516bd33fd5fc537d51ab3e6999fe15d9`; close `44f71a76740ea775834c3101b989c3bb516bd33fd5fc537d51ab3e6999fe15d9`; drift=false; First hash captured after targeted content read; no earlier byte snapshot claim.
- `core_enhanced/src/game/battle/core/sim.rs`; inspected 1484–1517, 1784–1827, control call symbol search only elsewhere; first `0aee574a7e766b34de84c6be573b801697a19485e49af457fe2fac09ce8863c7`; close `0aee574a7e766b34de84c6be573b801697a19485e49af457fe2fac09ce8863c7`; drift=false; Before main content read.
- `core_enhanced/src/game/data/content_pack/snapshot/tests.rs`; inspected 215–288; first `8d358e4c0769a391e4f4fdbfdf948082efb6bde320dc401cec6f58de6631548e`; close `8d358e4c0769a391e4f4fdbfdf948082efb6bde320dc401cec6f58de6631548e`; drift=false; First hash captured after targeted content read; no earlier byte snapshot claim.

Observed drift: none among captured hashes.

Root만 수정하며 본 helper는 새 /tmp 보고서 2개만 기록했다. 후속 교정은 이 원문을 덮지 않고 별도 source/검증 evidence로 처분해야 한다.
