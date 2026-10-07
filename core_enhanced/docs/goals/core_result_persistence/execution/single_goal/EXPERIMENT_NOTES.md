# U04 기술 판단·실패

P2G20 initial reviewed refs의 PE-* 이름을 표준 Goal reader PER-*에 맞춰 동결 전에 immutable P2G10/20 successor로 교정했다. 원본 보존, 정책/결정/acceptance 의미 불변.
Baseline367 모두 불변 가정은 기존 U03 ROADMAP status publication을 검출하여 authorized successor를 명시 결속했다. 제품363은 불변.
구체 구현 선택은 CODE_REVIEW TD01–08의 owner/수명/원자성/검증 경계 안에서 아래에 기록한다.

### GEX10 Continue/cache refinement (2026-10-04)

Continue V10 carries a current confirmed payload plus an optional separate SavePoint, result receipt,
pending ordinary reward session/original grant context, explicit phase and recovery restriction.
SavePoint and Continue have no recursive relationship. An immutable Arc payload/encoded byte cache
is part of the native GameCoreState capture schema; live ticks retain the pre-entry root. Each actual
ConfirmEnterNode captures the pre-entry state for all node categories. Successful retreat overlays
only its count on that root; process quit never consumes a retreat. Ordinary unfinished node effects
are discarded on Continue. A final battle with a mandatory reward choice retains that right until
claim/exit; terminal invalidation follows the completed entitlement. This implements DR03/DR07
jointly rather than dropping a final reward because the map is already complete.

New format accepts V10 only. Historical V7/V8/V9 source readers remain historical helpers and are
not silently promoted. Full malformed-input/current-identity validation and host protection remain
required before readiness. No persistence/completion claim is made for the partially implemented
producer until generic durable intent and host consumer are finished and independently audited.

Terminal runtime-fact views remain a supported live-process observer after settlement. The prior raw
CombatBattle owner served this role; immediate consumption exposed a real R07 observation regression.
The confirmed receipt now retains only the existing immutable RuntimeFactSnapshot transiently, skips
it in serde, and admits its retained extent together with settled Run resources. Ack releases it;
Continue restores the compact result summary without reconstructing runtime history.

ChooseOne zero candidates exposes `no_reward_candidates=true` in the Core-selected result DTO and
creates no claim session. A test authoring error (Boss featured reward kind absent) was fixed in the
fixture; existing mission reward validation is not relaxed to make the fixture pass.

### V10 fixture and codec follow-through

Direct test-root mutation now uses `save_run_checkpoint_for_test` to explicitly capture both the test SavePoint and current Continue; production SavePoint remains independent. Existing historical input bytes and conversion semantics are preserved. Current graph mutations address `continuation.current`; explicit Load injects its own SavePoint and proves cached Continue remains unchanged after rejected Load. Unsupported-version expectations now name the continuation codec.

V10 encoding preflights metadata and encoded-shape work before allocating JSON and bytes, polls afterward, and deterministically orders checkpoint VN consumed sets and shop purchased-merchandise sets. Missing nullable continuation fields are errors. Both SavePoint and current validate retreat bounds and rewind-trigger definitions. Pending ChooseOne candidates must match the retained catalog, including the manifested node's supported quantity scaling; substituted effects/IDs and invalid grant participants reject. Further actual nonzero-set/pending/restart/resource/host tests remain required.

Administrative application review found an unconverted host path: ExecuteAdminCommand could still mutate Core during accepted application recovery and skip the generic durable publication. Under TD05/TD07 and CF-D06, its existing ApplicationRoute::AdminCommand now uses prepare-for-publication, captures profile in the existing budget, preserves the U03 first-fault entry gate, and shares profile/active intent acceptance with ordinary commands. No new domain policy or admin permission is added. Prepared admin drop/preintent/accepted-pending and forced admin while pending are covered below before closing U04.

### Compact SavePoint codec refinement after actual public guard failure

The public guard exposed36674bytes against the existing32768 cap, while snapshot/export latency remained below their original caps. Under TD03/TD08 and CF-D06, V10 encodes an exactly equal SavePoint as `{current:{}}` and a distinct checkpoint as `{snapshot:{checkpoint:...}}`. This is only a strict private codec representation; import charges and materializes an independent SavePoint before later current state/retreat can change it. It neither drops historical rewind state nor raises an engineering cap. The checksum covers the canonical wire continuation, and malformed/extra/unknown references reject. Current and SavePoint comparisons/encoding stay inside native/authoring work accounting; no live tick re-encoding is introduced. This changes the not-yet-closed V10 producer and reader together and therefore requires new actual current consumer/feature/performance evidence before audit.

Fresh import accounting: shared SavePoint encoding had preserved gameplay independence but moved the first controlled clone before the former restore-time native-table provenance seed. Reproduction uses an actual serde-decoded64-entry table, compares independent clone usage against an explicitly seeded reference, and tests refusal when one clone slot short and when provenance is unknown. Capacity registration is allowed only for the closed fresh parser and monotonically increases existing private credit; general control scopes remain uncertified. This corrects U02/R06 work admission without changing codec, gameplay, budgets or public thresholds.

Canonical document follow-up: a broader V8/legacy scan found old R07 paragraphs in both contracts and a SavePoint rulebook paragraph still describing SavePoint-only Continue/V8. Updated to current V10 plus independent current/SavePoint roots and valid typed pending choices; historical explicit V7/V8 converters still produce V9 and current Continue rejects it, as the existing tests already assert. No product or test bytes changed. RULEBOOK_U04_002.patch supersedes the earlier inspection patch only; original RULEBOOK_BEFORE archive and prior patch are preserved. The final target FORWARD.patch owns the complete actual delta.

Coverage binding preflight initially rejected seven retired U02 host selector names (V08). Product executions had passed; the selector inventory still named the replaced Standard-only/split-write paths. coverage.py now records every old-to-current selector mapping with explicit acceptance12/14 semantic differences, including preintent rollback versus postacceptance recovery and protected legacy reset versus valid current full reset. Real subprocess crash and supported-reset evidence remain linked. This is an evidence-index correction, not a test/source change or an unchanged-semantics claim.

### F01 correction decision

The canonical host protocol obligation was explicitly assigned by original Goal line82, but initial source-union collection omitted the unchanged stale file. Correct the assigned documentation and add an explicit HOST_PROTOCOL_BEFORE supplemental baseline, verified against the closed auditor copy. Preserve initial source archive and target001. Current paired intent semantics differ from retained profile-only dirty/retry behavior; document both from their actual handler branches. Update old request/overlay/result field prose in the same assigned surface; no production behavior or policy changes. New coverage002 maps PE15/V12/V14 to this correction and retained runtime proofs. Product identity equality makes rerunning full suites unnecessary; fresh semantic/documentation/restoration audit is still required.

### Audit001 membership factual erratum — 2026-10-04T09:42Z

Direct comparison of original unchanged u04_001 MANIFEST/MEMBERSHIP and actual SOURCE_BEFORE/AFTER archive members disproves audit001’s claim that the host protocol was omitted. Its exact stale bytes were already in all original inputs. The valid F01 remains the required content update. MEMBERSHIP_ERRATUM.json records exact rows/hashes and actual archive checks. My preceding correction notes repeated the false omission claim; that part is superseded by this explicit correction. The supplemental HOST_PROTOCOL_BEFORE is a redundant exact original copy, not a newly added member. Original audit and unaudited targetu04_002/result002/coverage002 stay unchanged. Current correction is GEX_10_RESULT_003.json, COVERAGE_003.json and target/u04_003; product and corrected host-document bytes are unchanged from correction verification. Fresh audit must adjudicate this factual erratum as well as F01 content. No self-awarded closure or U05 advancement.
