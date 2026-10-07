# R02 current implementation notes

2026-09-10 KST. GEX-00 complete; GEX-10 pass 001 implementation and WP03 validation are complete. Full core002 has 1,214 passing tests; current release A/B/A003, API currentness proof, plain/allocator V6 guards, acceptance map and predecessor impact are recorded in EXPERIMENTS. Source/evidence freeze and separate R01/R02 fresh audits follow PLAN; no audit conclusion is prewritten. The design and “not yet implemented” statements below describe their original intermediate checkpoints, not remaining implementation at closure. Exact upstream, directory roles and preservation baseline are in PLAN and BASELINE_001.

Adopted choices from P2G: typed full definitions before final lowering; exact global/namespaced string keys with separate UUID/local/reference-keyed/singleton handling; actual array-order semantic hash; source receipts separate; existing-gate-preserving generation-aware prepared commit; latest-only content-bound storage; separate official packaging; trusted bounded input acquisition. The exact adapter list/API/resource defaults must be detailed in GEX-10 before mutation.

R01 old audit is historical after the first owned-source mutation. Any R02 source/parser/publication/store change must be mapped to its IVA02/04/05/06 and fresh current audit restored before R02 completion. R01 historical V4 retention is not R02 compatibility policy. No personal saves or server/Unity source will be mutated.

Current detailed application/save/token and snapshot/typed-resolution design is in PLAN §GEX-10 pass001. The owned-source delta is implemented and focused-tested; raw adapters and safe filesystem primitives are the next implementation closures. Official input measured 408,797 bytes across 24 files; largest 114,717 bytes.

## GEX-10 typed catalog inventory / before first mutation

Bounded read-only `/root/r01_source_seams` inspected all 24 source roots plus separate bundle manifest. Root accepted the following exact adapter inventory as technical detail within L02–04; no policy acceptance changed.

| Domain | Typed collections / authoritative key |
| --- | --- |
| shops | ShopMerchandisePreset, ShopMetadata, ShopPoolMetadata / id; shop UUID retains owner checks |
| rewards | RewardMetadata, RewardPoolMetadata, EquipmentRewardPoolMetadata / id; reward UUID retains owner checks |
| events | EventDefinition / EventId string |
| visual_novels | VisualNovelDefinition / VisualNovelId string |
| visual_novel_triggers | VisualNovelTrigger / VisualNovelTriggerId string |
| boss_omen | BossOmenChain / BossOmenChainId string |
| boss_runtime | BossPatternDefinitionData, BossActorBehaviorDefinitionData, BossEncounterDefinitionData, BossSignalDefinitionData / top-level id |
| summoned_units | SummonedUnitDefinitionData / SummonedUnitDefinitionId; base_uuid retained |
| abnormalities | RawAbnormalityMetadata / id; uuid and strict AuthoredBasicAttackDef/RequiredOption retained |
| corroded_employees | CorrodedEmployeeBasicAttackRangePresetDef and RawCorrodedEmployeeProfileMetadata / id; final preset lower only |
| corroded_waves | CorrodedWavePreset / id |
| employees | EmployeeDefinition / employee_definition_id |
| equipment | EquipmentMetadata and EquipmentMaterialMetadata / id; EquipmentDismantleRecipeMetadata and EquipmentEnhancementRecipeMetadata / referenced equipment_id |
| consumables | ConsumableMetadata / id; uuid retained |
| buffs | BuffMetadata / name; BuffId::from_name is derived runtime identity |
| skills | SkillRangePresetDef and RawSkillDef / id; nested cast/step/effect typed payload, local action/effect IDs |
| skill_fragments | SkillFragmentMetadata / SkillFragmentId; uuid and BTreeMap upgrade_skill_ids retained |
| pve | PveEncounter / id; PveEncounterRewardSource / reward_source_id, with separate encounter_id/fragment_id reference and uniqueness checks |
| run_policy | complete RunPolicyData singleton |
| node_definitions | MapNodeDefinition / kind_id |
| facility_templates | FacilityTemplate / id; FacilityTemplatePool / id; tag_catalog as complete ordered typed vocabulary setting |
| node_assignment_profiles | NodeAssignmentProfile / id; StandardFacilityScheduleEntry / floor; endless_stages as complete ordered interval/cycle setting |
| battlefield_archetypes | BattlefieldSizeClassDefinition / size_class enum; BattlefieldArchetypeGenerationDefinition / archetype enum; id string is not runtime lookup authority |
| battlefield_templates | source is Vec<BattlefieldTemplateDefinition>, then typed database wrapper; id / local route IDs |

Skill raw root/definition/cast/step require Clone/Serialize and pub(crate) parts/ID access, then existing fallible lower. Corroded raw root/profile requires same. Abnormality raw conversion is infallible and must retain authored field requirements; RequiredOption Serialize must be transparent Option, not a wrapper object. Other derived catalog Deserialize skips OnceLock, whereas new(...) may eagerly index/expect; assemble fresh unindexed typed roots and use final fallible validation. Combat preview private definition/collection fields need only crate-level access/parts helpers. Boss/summon top-level canonical namespaces must be accepted separately from stricter definition-local task/state/parameter/handle grammar.

UUID currentness clarification: preserve existing typed identity and actual owner validations. The observed cross-domain UUID check is equipment-item vs consumable-item; other domains have owner-specific validation. The earlier broad phrase “existing global uniqueness” is not evidence of one all-catalog UUID registry and does not authorize adding new cross-owner rejection. UUIDs never receive a namespace prefix. Agent initially described PvE reward_sources as encounter-keyed but corrected it before this accepted inventory; reward_source_id is the independent namespace key.

First production delta is limited to owned source diagnostics: LiveRonError stores String, parse_live_ron and SkillDatabase::from_ron/live_error accept borrowed &str and copy into errors/compiled EffectSource. Closed domain names remain static. A durable test passes a temporary owned source path through both valid skill compilation and invalid opcode parsing, drops the input path, and observes retained path plus definition/action/effect/field identity. Existing literal parse behavior remains. This is only one implementation seam; full external snapshot/resolve/generation/save are still required and unimplemented. Before this delta, relevant source hashes equal baseline; after it, affected R01 evidence is historical until focused revalidation/fresh audit restoration.

### Recorded terminal module regression

Formatted owned-source delta passed 81 live_ron tests and 14 skill_data tests (95 distinct, the earlier single test is a subset); terminal exits 0. VALIDATION_INPUT_002 captures the unchanged formatted source used by both commands. External pack and generation/store implementation remains open.

### Raw assembly / canonical serializer implementation detail

The three custom raw roots gain Clone/Serialize and only crate-visible collection/identity access; RequiredOption serializes transparently without relaxing its existing Deserialize requirement. Corroded with_range_presets only creates fresh OnceLocks and is safe before final validation. Combat-preview typed collection/key fields gain crate visibility only. Other eager-indexing owner constructors must not receive unresolved or invalid final candidates; use owner parts assembly with fresh caches.

Actual authored unordered data exists: EquipmentMetadata.triggered_effects is HashMap<TriggerType, Vec<TriggeredEffect>>. Fingerprints therefore use an explicit serde serializer over typed raw winners: sort map entries by canonical serialized key, preserve all sequences, encode enum/Option identity, exclude serde-skipped source/cache fields, retain exact finite float bits. Reject nonfinite floats during canonical serialization instead of allowing serde_json to collapse them to null; no JSON roundtrip is used to construct gameplay types. The same fixed canonical format/version will include resolved selection/writer records and current contracts. Meaningful serializer probes compare different HashMap insertion orders, changed execution Vec order, and nonfinite rejection. This is technical realization of SC02, not a new identity policy.

### External snapshot implementation detail / before mutation

Trusted InputLimits defaults are now concrete: 128 packs, 4,096 files including manifests, 8 MiB per file, 64 MiB aggregate first-read bytes, 4,096 combined dependency/writer edges, nesting 128. Host parser depth must be 1..=128, with the same bounded lexical check for comments/raw-string delimiters; manifests cannot deserialize budgets. Normal relative UTF-8 paths reject empty/dot/dotdot/absolute components, backslashes/drive syntax/control characters. Track case aliases for each directory prefix and file, duplicate inventory and duplicate file identity.

On the current Unix/WSL target, rustix 1.0.7 safe open/openat uses pinned trusted-root directory handles and NOFOLLOW on each untrusted component, DIRECTORY for intermediate components and NONBLOCK for leaf opening before regular-file checks. Every file has bounded owned UTF-8 bytes, matching metadata before/after reading, and a SHA-256 byte receipt. After all manifests/catalogs are acquired, reopen every inventory path through the same trusted-root handle and compare identity/metadata and full bytes. No data writes or retries occur; observed change fails the candidate. This is finite observed consistency, not an atomic filesystem snapshot. Non-Unix builds must fail explicitly if this no-follow backend is unavailable; no weaker path-string fallback or unverified cross-platform support claim. A later host port can supply an equally strong backend within its own verified scope.

The local cached rustix 1.0.7 fs API was inspected directly. Add it as a Unix-only core dependency; no unrelated lock upgrade. Durable isolated tests will probe regular/UTF-8 files, exact byte/file/aggregate/depth boundaries, syntax-aware quoted/raw/comment handling, path/case/hardlink/symlink rejection and a controlled mutation between acquisition and recheck. Official packaging and public generation/app/storage are still unfinished.

### Latest V6 storage transition intake / read-only source assistance

`/root/r01_source_seams` inspected actual codec, capture, shared validator, public checkpoint ports and old durable tests without edits or execution. Root accepted the bounded inventory. Existing V4 reader/removal-certification gate is real (`ACTIVE_RUN_V4_READER_REMOVAL_CERTIFIED=false` plus its gate test); remove the old reader/gate under this already approved R02 latest-only transition, never flip certification true. Preserve the original GEX00 V4 corpus bytes/hash and old tests in baseline/immutable prior evidence. The existing canonical V4 suite is substantive: move its 19 one-edge semantic graph defects, fragment-limit, strict field shape and no-mutation assertions to a latest writer-produced V6 envelope with checksums refreshed after each deliberate semantic mutation. Legacy rejection must not hide these checks. Replace old V4/V5 success/equivalence claims with V6 roundtrip plus separate V4/V5 rejection. An old unknown-version fixture using 6 must become 7.

RunCheckpointPayload gains required ContentIdentity with no serde default. `capture` accepts the token identity: one production call in node_flow and three test captures (tests/mod, mastered_elite_sweep, combat). Rename skipped byte cache to version-neutral encoded_core_payload, codec to V6 and constant to 6. `restore_into_state` consumes payload state only and must not choose a generation. Shared validate_run_checkpoint_payload_with_seed compares identity first, then keeps every existing seed/graph/power/profile validation. Continue may restore a saved seed different from constructor bootstrap seed, but must retain internal progression/UUID seed consistency, host-injected current player, max load=3 and monotonic profile/peak/VN counter behavior. Explicit load consumes one of three loads; continue does not. Cache is set only after encode and all restore validation precedes payload/cache/load/player/seed mutations.

Public ApplicationCheckpoint opaque byte/load/peak ports keep their signatures. Test rollback snapshots additionally capture/restore token while retaining monotonic profile/peak merge. Reuse actual Gate replay/map/UUID stream tests and add content mismatch with a recomputed valid checksum so mismatch admission is directly observed. Source token identity and skipped cache bytes need explicit no-mutation comparison, because existing continuation_authority_snapshot does not include them. Full current integrated revalidation/fresh audit remains required; the assistant's read-only inventory is not a completion audit.

### V6 implementation and minimal packaging intake

V6 code now stores required content identity in the canonical-checksummed payload, checks the retained generation before seed/graph validation, removes V4/V5 readers and the uncertified-removal scaffold, and carries the strong generation in transaction rollback. The existing 19 graph defects now use latest writer bytes with refreshed checksums. The historical V4 semantic corpus hash remains pinned by projecting only the new identity field out of V6; this test is not a legacy importer. New probes cover every identity contract component with a valid checksum, required-field omission, actual external A/B save mismatch, equivalent A re-acquisition, cached export and explicit load, profile preservation, and token rollback. Focused command is in progress; no success yet claimed.

Read-only `/root/r01_source_seams` packaging intake is accepted as technical design, not implementation/audit evidence. Reuse CatalogKind::ALL/official_path and snapshot Inventory/Root for external source `data/bundle.ron` plus the 24 catalogs. Validate the existing source bundle wrapper instead of implicitly upgrading it. Keep catalog bytes exact, generate only the current pack manifest, and validate the captured snapshot with the same post-acquisition publication path before writing. Output must be an absent directory outside the source tree, created exclusively with no merge/overwrite. Write catalogs before manifest; report any incomplete new output rather than deleting an existing path. Reopen the output using the public loader to produce the final receipt/identity.

Register minimal package_official_content and consume_content_pack binaries explicitly (autobins=false), without test-fixtures. The consumer must exercise CoreApplication's real briefing and starter selection, with fixed seed and explicit pack selection. Isolated B uses full RunPolicy Replace after official, changing starter_enkephalin 500 to 733; same release executable A/B/A must produce actual Run snapshot `/resources/enkephalin` 500/733/500 and corresponding identities. This directly exercises world resource assignment, not only metadata. Consumer does not expose BattleCore or add R03's general CLI/reload/simulate features. Direct source files and preexisting custom bytes must remain unchanged; missing/invalid external input must fail with no embedded fallback. Source acquisition and output backend support are Unix/WSL only until separately implemented and verified elsewhere.

### Standalone API consumer transition

Read-only search found four positive default-release API consumer mains and four negative privacy consumer mains still constructing data through load_live_embedded. Their input adapter now takes explicit trusted-root/pack CLI arguments and calls the same public external loader through tools/external_pack_probe.rs. Every existing API/behavior/privacy assertion remains. This is adaptation of direct consumers, not a change to pipeline validators or assertion policy. The explicit default_fixture_api_audit negative probe retains its forbidden load_live_embedded call so default-release API absence is itself observed. Feature-enabled preservation/final-performance tools retain their explicit fixture construction. Old no-argument positive probe invocations are no longer valid; current direct executions must supply the isolated official pack. These changes require final scoped input/lock capture and actual positive/negative probe validation.

### Remaining integration probes / bounded source review

Read-only `/root/roadmap_intake_review` identified the uncovered actual consumer seams; root accepted the technical inventory without treating it as completion review. New test source now moves skill/corroded presets into a separate selected dependency pack, exercises actual external final buff/reward/reward-source dangling references and manifested-difficulty generated setup failure, and preserves A on errors. The valid Remove fixture uses independent left::same/right::same equipment namespaces, a cross-namespace reference-keyed recipe, explicit Remove of one equipment/reward plus a full pool Replace. It first rejects an uncleared dangling pool, then checks final catalog/string/UUID/item-registry/recipe views and actual RewardState→ClaimReward inventory acquisition. Original A indexes are populated before B and must remain intact.

The generation consumer probe publishes changed external tactical-resource initial value B while A is in a real battle, retains A's snapshot/export/resource, starts B through the existing generation-aware command, completes actual briefing/starter/SavePoint, compares A/B checkpoint identities, and enters a real B battle with value 31 instead of A's 20 and the same B database Arc. A separate deliberate fixture-only missing-RunStart B exercises the real post-initialization briefing preparation error and retry rollback. It is explicitly not a claim that invalid external B can publish: public validation rejects that static defect. No production fault hook or gate weakening was added. These new tests still require their first compile/execution.

### Standalone dependency resolution observation

First adapted positive API consumer is building with explicit external arguments under cargo run --release --offline and a dedicated target/capg-api output. Its independent preexisting lock differs from the workspace lock and lacked rustix. Cargo added rustix/errno/linux-raw-sys plus target-specific Windows entries and resolved four unrelated cached target-specific versions (futures-core/task/util and rustversion) downward. The root Cargo.lock is unchanged by this tool build. This automatic standalone lock delta is recorded and must be inspected for a minimal retained dependency update; do not describe it as only one dependency row or silently count an older lock as the tested input. VALIDATION_INPUT_017 captures the actual resolved input during compilation; terminal result is pending.

## GEX-10 pass002 closure context

R01 fresh successor audit002 is Complete / High. Its fixture-helper comment observation was corrected only after audit closure in the separately recorded CORRECTION_002 pass. Executable/test/data/config inputs are unchanged by that rustdoc edit; VI023 and COMMENT_REUSE_002 provide exact reconstruction/currentness evidence. R02 target002 includes the actual predecessor result; its separate fresh audit remains required. Prior checkpoint statements above retain their historical scope.


## GEX-10 pass003 — source fidelity correction and analogous setup case

R02 fresh GEX-20 closed Not Complete with R02-AUDIT-F01 on ATR002. That original verdict is preserved.
The first repair retained typed failing wave/field and selected template context through the same generator;
publication diagnostics resolve primary and related source addresses independently. The first new test caught
the template catalog's top-level-array address (`[13]`, not `templates[13]`); that failure is retained.

Bounded read-only `/root/r01_source_seams` then identified an analogous used custom BossPattern case. Root
added a separate full Replace fixture before production repair; actual late missing-summon-zone failure omitted
its custom file and the test failed as predicted. The selected compiled BossEncounter's behavior/pattern graph
now supplies aggregate contributor identities. This runs only while constructing a failed generated diagnostic,
uses first-seen deterministic definition order, and does not choose or execute a different gameplay pattern.
The custom replacement zone and route cases both preserve repeated rejection, old A/input bytes and valid B
restoration. No string matching of GameError text determines source identity; no content-ID special case exists
in production. Fixtures use the existing nonproduction reference pattern without changing official data.

VI027 has exactly eight changed Rust inputs vs VI023; 271 other recorded inputs remain byte-identical.
Current generated focused003 passes 16 and full_core004 passes 1,217. VI025 successful API batch002 can be
reused after the final error-only helper delta: exact reconstruction proves unchanged code outside that helper
and one appended test. Existing public observer measurements are historical direct measurements of unchanged
snapshot/export/rejected-staging paths; loader/start/combat preparation is outside their measured intervals.
The bounded source review is assistance, not a completion audit. Actual fresh predecessor and R02 audits remain
separate, after implementation/evidence freeze. Original P2G meanings, all prior failures and old targets survive.
