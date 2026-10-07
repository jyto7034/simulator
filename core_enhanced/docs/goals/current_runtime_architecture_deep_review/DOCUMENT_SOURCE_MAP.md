# Document Source Map

> 참조 상태 정정 (2026-09-06): 일반 보스 알 메모(`SRC-07`)는 개별 연출을 AI가 임의로 공통 정책으로
> 일반화한 문서여서 사용자 요청으로 삭제했다. 아래 경로·크기·해시는 당시 수집 기록이며,
> 해당 문서를 현재 검토 입력이나 승인 정책으로 사용하지 않는다.

> 인수인계 문서 정리 (2026-09-06): 파편·장비 인수인계 문서(`SRC-06`)는 역할 중복으로 삭제했다.
> 최신 결정과 완료 상태는 유물 정책 및 파편·장비 설계 문서로 이관했다. 아래 경로·크기·해시는 당시 수집 기록이다.

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Catalog captured: `2026-08-22T06:44:27+09:00`
- Baseline: `BASELINE-20260822-064427-KST`

## Authority Rules

1. `docs/game_rulebook.md` is the latest gameplay-policy source.
2. Current Rust proves implementation structure and behavior, not intended policy.
3. Core/data/skill contracts are classified per selected section; they may be current technical detail or stale intent.
4. Tmp and boss/skill notes are future-pressure or representative stress-case sources only when a criterion needs them.
5. The Unity contract is stale historical consumer intent and is not a compatibility target.
6. A justified `skip` or `irrelevant` classification is not a gap. Completion is based on criterion/scenario evidence coverage.

## Candidate Identity Catalog

Heading scans below are routing metadata, not content evidence. No candidate was read in full during cataloging.

| ID | Path | Bytes | SHA-256 | Candidate role | Initial routing |
| --- | --- | ---: | --- | --- | --- |
| SRC-01 | `docs/game_rulebook.md` | 104002 | `8aa954cc215dccc63b736f884327ca03375f82895b5e86eb2252e597e8fcbf6f` | normative latest gameplay policy | `defer`: select core-relevant capability sections per A00/change scenario |
| SRC-02 | `docs/core_runtime_contract.md` | 97867 | `73bc18d42f11b54a69f6d861c675cda7d35f6e2090f6b34cb23e539e3d8364f6` | technical-contract candidate; known potentially stale | `defer`: select authority/lifecycle sections after Rulebook question is known |
| SRC-03 | `docs/data_loading_contract.md` | 37271 | `58c636ceacb990a328644de8bfd44cb5ca85bf744d5ef680eaba00dc081b5c7b` | technical-contract candidate | `defer`: use for A03/A08 data publication and validation |
| SRC-04 | `docs/skill_target_contract.md` | 28764 | `a35a20243be2ea64c35931292d11f26bda3cf3980e816370c7ee88b0e220789c` | technical-contract candidate | `defer`: use for skill targeting/delivery/interrupt scenario |
| SRC-05 | `docs/tmp/run_relic_system_design_notes_2026-08-21.ko.md` | 13116 | `0d97d131a941ae7ea2b2caab7d2b4a98eda058894fce2d5546a7e78cb8f9dae7` | future pressure/draft | `defer`: use selected principles for relic/monolith scenario |
| SRC-06 | `docs/tmp/handoff_skill_fragment_design_2026-08-20.ko.md` | 8506 | `ba818647fd0b52045ec4f819e84cc0bd4ff2a82b518f1ababb99562b21ac3926` | handoff/future-pressure index | `defer`: likely only rules summary and system extension list; skip duplicated content |
| SRC-07 | `docs/tmp/normal_boss_egg_design_notes_2026-08-19.ko.md` | 2260 | `9006b6455a63f436bc8665837750e3daabe85251f2a284ee2e6505db55100e41` | future boss/run pressure | `defer`: boss reward/progression scenario only |
| SRC-08 | `docs/tmp/ego_corrosion_system_design_notes_2026-08-19.ko.md` | 9090 | `e4d9d576aacdfffc2ec3b09f0455185c5cfeb927e6d1011e1c905f7cd5dcde91` | future lifecycle pressure | `defer`: corrosion lifecycle scenario only |
| SRC-09 | `docs/tmp/generic_dice_and_tool_item_rework_notes_2026-08-17.ko.md` | 6445 | `5f8ed0f9a013d484e8bea5cfa652b5b0f1bda9dae91b7dcd0539d27888c49127` | future event/tool pressure | `defer`: generic dice/tool atomic-result scenario only |
| SRC-10 | `docs/tmp/skill_activation_and_resonance_rework_notes_2026-08-17.ko.md` | 10581 | `f588899dd06c86f1705590bca6f7efd4c4549185f73927b475ebed548d81fe2a` | future skill policy pressure | `defer`: activation/resonance scenario only |
| SRC-11 | `docs/skills/abnormality_design_notes.ko.md` | 234795 | `5ddaf58d1f8f74a6e8dced236b8eb25811dac78bbd605f25a30fe9abb80a996b` | boss/skill stress-case catalog | `defer`: choose a small representative mechanic set; do not read catalog-wide |
| SRC-12 | `docs/skills/abnormality_skill_fragments_design_notes.md` | 59359 | `f35cdcdd713a0b36b780e7f5322d018ca000343ee21fdf80300ccf7452fb8e0a` | fragment/equipment stress-case catalog | `defer`: choose representative fragment/equipment combinations only |
| SRC-13 | `/mnt/f/unity projects/ark/docs/unity_core_contract.md` | 125401 | `a3224e244f9577fbb6a4d6d01e44b7ea85f3ba1969f656a22712b0ba972063f5` | stale historical consumer intent | `skip` initially; read only if A07/A08 needs historical adapter evidence |

## Heading-Level Routing Inventory

- SRC-01 groups the policy into game loop, exploration/nodes, battle, growth/fragments/equipment, non-combat nodes,
  failure/result, authoring, boss/legacy policy. These groups will drive the requirement map.
- SRC-02 groups runtime intent into determinism, battle lifecycle, tactical authority, run commit, inventory/shop/monolith,
  visual novel, boss runtime and Unity projections. Each selected section must be checked against SRC-01 and code.
- SRC-03 exposes source-of-truth, live RON, bundles/catalogs, fixture builders and validation gates.
- SRC-04 exposes targeting foundations, skill lifecycle, interrupts/projectiles, area delivery and projection/authoring.
- SRC-05~10 expose bounded future scenarios already named in the Plan; only scenario-relevant sections will be used.
- SRC-11 and SRC-12 are large content catalogs. Their headings are sufficient to select representative cases later.
- SRC-13 exposes protocol/snapshot/core-flow and consumer contracts but is intentionally skipped until a historical boundary
  question requires it.

## Section Routing Records

No content section has been promoted to evidence yet. Add one record per bounded read before moving to another question.

| Record | Source | Criterion/scenario question | Decision | Heading/range read | Section status | Conflict/drift | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ROUTE-000 | SRC-01~13 | What sources exist and how should later reads be routed? | metadata scan | headings only | unclassified except declared authority | none assessed | E-001 |
| ROUTE-001 | SRC-01 | What is the canonical game identity, mode and run-start loop? | `read` | lines 1-141: responsibility, identity, modes, core loop, starter selection | `normative` | external Unity links are stale under approved Goal precedence | E-006, REQ-001 |
| ROUTE-002 | SRC-01 | What does exploration/node selection, safe content, event/inventory/shop and briefing mean? | `read` | lines 142-337 | `normative` | technical/wire references not accepted without separate freshness check | E-007/E-016/E-017, REQ-002 |
| ROUTE-003 | SRC-01 | What are player-visible DefenseRoute, tactical, setup/wave, damage and death semantics? | `read` | lines 338-718 | `normative` | external wire ownership ignored; SRC-04 targeting detail deferred | E-008/E-009, REQ-003 |
| ROUTE-004 | SRC-02 | Do selected determinism, atomic battle advance, setup/movement/result sections describe current code? | `read` | lines 20-118 and 404-455 | `current technical contract` for selected sections | document-wide freshness not inferred | E-009, REQ-003 |
| ROUTE-005 | SRC-04 | How are acquisition, player selection, step retarget, delivery geometry and interrupt identity divided? | `read` | lines 14-416: Targeting Foundations through Area Delivery/Projection boundary | `current technical contract` for selected sections | code matches inspected responsibilities; external Unity transport ownership was not followed | E-010, S03, REQ-003 |
| ROUTE-006 | SRC-10 | What future activation/resonance pressure should S03 exercise? | `read` | lines 50-107: authored mode/toggle alternatives, resonance pressure and invariants | `future pressure/draft` | current active fragment hard-codes Manual; `Triggered` enum is present but draft says selectable authority is not designed | E-010, S03 |
| ROUTE-007 | SRC-01 | Which FinalBoss/BossOmen/run-continuation rules constrain the boss scenario? | `read` | lines 1188-1266: FinalBoss pool/chain/capstone/resolution and hidden boss | `normative` | current production chain is explicitly bootstrap, not final content proof | E-011, REQ-007, S01 |
| ROUTE-008 | SRC-02 | What current battle-local Boss Runtime foundation exists? | `read` | lines 875-958: compile/setup/runtime/task/summon/movement/cancellation boundary | `current technical contract` for selected section | explicitly says Little Red/CWBS are references and public boss lifecycle is out of scope | E-011, S01 |
| ROUTE-009 | SRC-07 | What normal-boss rematch pressure belongs outside battle-local AI? | `read` | complete small note, lines 1-48: egg/rematch/reward/later-phase/open design | `future pressure/draft` | many policy values remain open; used only to test boundary placement | E-011, S01 |
| ROUTE-010 | SRC-11 | Which representative boss mechanics stress the current boundary? | `read` | lines 1157-1293 Little Red; 1425-1507 Nothing There | `future pressure/representative design` | placeholder/live data is not implementation evidence; only bounded designs selected | E-011, S01 |
| ROUTE-011 | SRC-01 | Which current growth, reward, Monolith, equipment and fragment semantics must S04/S05 preserve? | `read` | lines 719-1005 | `normative` | current ordinal fragment progression may conflict with future draft, but remains policy until changed | E-013/E-014, REQ-004, S04/S05 |
| ROUTE-012 | SRC-05 | What Relic compatibility/loadout/effect pressure should the run aggregate handle? | `read` | lines 14-232: principles, categories, compatibility, hooks, slots, acquisition and open policy | `future pressure/draft` | no current Relic runtime exists and open values are not requirements | E-013, S04 |
| ROUTE-013 | SRC-06 | Which fragment-form and linked-system changes are intended future pressure? | `read` | lines 39-140: rules summary, growth/form direction and extension handoff | `future pressure/draft` | summary can supersede current growth only after policy approval; used to test schema fitness | E-013/E-014, S04/S05 |
| ROUTE-014 | SRC-08 | Which EGO corrosion lifecycle facts cross equipment, battle and run consequences? | `read` | lines 15-136: trigger set, input, withdraw/summon/despawn, result and Breakdown pressure | `future pressure/draft` | lifecycle and timing policy are not implemented; used only as a boundary scenario | E-015, S06 |
| ROUTE-015 | SRC-09 | What generic dice/tool requirements should Event d20 and item paths be tested against? | `read` | lines 16-114: stable request/receipt, consumer critical/effect, Giant Sap and atomicity | `future pressure/draft` | generic service/tool ledger does not exist; current Event d20 is only a characterization oracle | E-016, S07 |
| ROUTE-016 | SRC-01 | What current non-combat node contracts constrain a new node/session design? | `read` | lines 1006-1074: SavePoint, Maintenance, HeadquartersContact | `normative` | selected core-runtime/wire links do not override Rulebook policy | E-017, REQ-005, S08 |
| ROUTE-017 | SRC-01 | Which completion, retreat, condition and failure facts must an application transaction preserve? | `read` | lines 1075-1173 | `normative` | `breakdown_collapse_node_limit` is intended 2 but explicitly not live-authored; implementation gap recorded separately | E-015/E-017, REQ-006, S06/S08 |
| ROUTE-018 | SRC-03 | Does live data have a parse/validate/compile/publish boundary, and how independently can it evolve? | `read` | lines 15-58, 242-270 and 349-378: source of truth, durable-document distinction, bundle index, fixture and validation gates | `current technical contract` except noted count drift | the prose says 23 gameplay sources, while the single macro inventory and exact physical-inventory test currently contain 24; bundle plus gameplay sources total 25 RON files | E-019, A08 |
| ROUTE-019 | SRC-02 | Which durable-save responsibilities belong to Core and which belong to the host? | `read` | lines 455-483: strict V4 Core export, host checksum/lock/fsync/backup/journal/tombstone and rollback requirements | `current technical contract` for the Core/host responsibility split; host implementation unverified | Core code matches strict payload/export/continue/rollback portions. `game_server` persistence behavior is out of review scope and is not certified | E-019, A07/A08/A10 |

## Runtime Data Evidence Identity

Direct live-data reads are recorded separately from candidate policy sources.

| ID | Path | Bytes | SHA-256 | Read scope/use |
| --- | --- | ---: | --- | --- |
| DATA-001 | `../game_resources/data/boss_runtime/base.ron` | 15863 | `58805dbc82ee3d4513c571acf8c034646b1389e070fcaebacd08f84ba4966d04` | lines 1-480; current Little Red production reference and nonproduction summon graph |
| DATA-002 | `../game_resources/data/pve/encounters.ron` | 114717 | `8b2472db6750dcbf883bd0d826320e5a139b9a2ef47f3f460efae378c22fecde` | lines 1465-1565; `suppress_little_red` binding/outcomes/waves |

At first direct access, the aggregate SHA-256 of 25 adjacent RON/JSON files under `../game_resources/data` was
`58fa32ef690e4f5e3d2c3062ecb184da99907b3c5c796c77b7e4e3b25cc5d0d0`. Only DATA-001/002 content was selected.

## Skips And Deferrals

- SRC-13 is a justified final skip because Unity compatibility is not a rewrite constraint, the document is stale, and current
  protocol/save evolution could be judged from the Rust boundary plus selected current contracts. No adapter-history question
  arose that could change A00~A11 or the rewrite decision.
- SRC-11 was read only for the two S01 representative sections. SRC-12 remains deferred; catalog-wide reading would not improve
  runtime architecture coverage.
- SRC-05, SRC-06, SRC-08 and SRC-09 were read only for S04~S07 bounded pressure. SRC-10 was selected only for S03. SRC-12
  remains deferred because the chosen runtime/schema questions were answered without a catalog-wide fragment/equipment read.
- Unselected SRC-02/SRC-03 sections remain unclassified rather than being promoted document-wide. Their selected sections were
  sufficient for battle lifecycle, durable-host, live-data publication and fixture-boundary questions.
