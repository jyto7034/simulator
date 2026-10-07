# Game Rulebook Runtime Alignment Claim Matrix

Audit generation:

- rulebook: `docs/game_rulebook.md`
- rulebook SHA-256: `c11a09402febbb6b16dcc98a11dd1364bac8dfe53645627f1670e6fbd7380f1e`
- simulator commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Unity commit: `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`
- authority: the current dirty working trees, not either clean commit

## Claim ID and coverage rule

- Every normative list item has the stable ID `GRB-L####`, where `####` is its rulebook anchor line.
- Normative prose in an inclusive line interval has the group ID `GRB-P####-####`.
- In the table, `GRB-L####-####` means **every list-item anchor occurring inside that inclusive
  interval**, not every integer in the interval. The interval rows are contiguous from line 1 through
  line 1210, so every heading, prose claim and all 365 list-item anchors are mapped exactly once.
- A `mixed` row means the cited finding applies only to the stated subclaims. The remaining claims in
  that row were traced without a conflicting current path.
- `approved_pending` claims are still catalogued when current runtime/live data do not implement the
  approved target; the catalog labels them `document_ahead_expected` instead of treating them as an
  accidental regression.

## Evidence index

| Evidence | Current authority inspected |
| --- | --- |
| `E-DOC` | `docs/README.md`, `core_runtime_contract.md`, `skill_target_contract.md`, `data_loading_contract.md`, both named external Unity canonicals |
| `E-START` | `world/helpers.rs`, `world/snapshots_and_start.rs`, `employee_data.rs`, `employees/definitions.ron`, starter tests |
| `E-FACILITY` | `map/facility_generator.rs`, `world/node_flow.rs`, `world/map_encounters.rs`, `map/*.ron`, map/facility tests |
| `E-SAFE` | `world/{facility,event_node,map_content,reward,shop,headquarters,maintenance}.rs`, live Event/reward/shop RON, node-session tests |
| `E-COMBAT` | `battle/core/**`, `world/{combat,battle_capability}.rs`, live PVE/battlefield/skill RON, unit and runtime-contract tests |
| `E-RESEARCH` | `abnormality_research.rs`, `world/{state,combat,node_flow}.rs`, `run/policy.ron`, `abnormalities/base.ron`, research tests |
| `E-PROFILE` | `encounter_compendium.rs`, Core snapshot/commit paths, server compendium persistence, Unity `EncounterCompendiumModel.cs` |
| `E-POOL` | `world/map_encounters.rs`, `monolith.rs`, `pve/encounters.ron`, encounter-selection and monolith tests |
| `E-BOSSOMEN` | `boss_omen.rs`, `boss_omen_data.rs`, `boss_omen/chains.ron`, BossOmen map/combat tests |
| `E-EQUIPMENT` | `equipment_data.rs`, equipment runtime/maintenance paths, `equipments/base.ron`, equipment and live-RON tests |
| `E-TRANSPORT` | Core behavior/snapshot serialization, server message/result mapping, external Unity Core readers/command bus and named canonicals |
| `E-LEGACY` | repository-wide current-source search plus server legacy-transport rejection tests |

## Complete coverage matrix

| Coverage ID | Rulebook location and stable claim coverage | Lifecycle | Evidence | Verdict / mismatch |
| --- | --- | --- | --- | --- |
| `GRB-C001` | L1-4, title and top-level authority (`GRB-P0001-0004`) | current | `E-DOC` | `aligned`: the rulebook accurately says runtime/live data win implementation disputes. |
| `GRB-C002` | 문서 책임, L5-31 (`GRB-P0005-0031`, `GRB-L0016-0022`) | current/delegated/approved_pending | `E-DOC`, all evidence bundles | `aligned`: ownership links are live and the pending-policy warning accurately predicts the catalogued gaps. |
| `GRB-C003` | 게임 개요 intro, L32-36 | current | `E-DOC` | `aligned`. |
| `GRB-C004` | 게임 정체성, L37-49 (`GRB-L0043-0046`) | current | `E-COMBAT`, `E-SAFE` | `aligned`: the live mode is employee/loadout-centered `DefenseRoute`; no owned-abnormality game mode was found. |
| `GRB-C005` | 게임 모드, L50-97 (`GRB-L0063-0095`) | mixed current/approved_pending/future | `E-FACILITY`, `E-RESEARCH`, `E-PROFILE`, `E-BOSSOMEN` | `mixed`: `GRB-M-001`, `GRB-M-003`, `GRB-M-004`; future Gate/WhiteNight design sentences remain `future_or_dormant`. |
| `GRB-C006` | 핵심 플레이 루프, L98-119 | current/approved_pending | `E-START`, `E-FACILITY`, `E-SAFE`, `E-COMBAT` | `mixed`: general loop and attempt policy align; Standard/Endless exits differ via `GRB-M-001`, `GRB-M-003`. |
| `GRB-C007` | 시작 직원 선발, L120-139 (`GRB-L0124-0137`) | current | `E-START`, `E-EQUIPMENT` | `aligned`: one definition catalog, six eligible candidates/three selections, run UUID ownership, queue and authored loadout are live. |
| `GRB-C008` | 탐사와 노드 intro, L140-145 | current/delegated | `E-DOC`, `E-FACILITY` | `delegated_detail`. |
| `GRB-C009` | Floor 맵과 노드 선택, L146-203 (`GRB-L0151-0192`) | mixed current/approved_pending | `E-FACILITY`, `E-TRANSPORT` | `mixed`: facility topology/navigation/concealment align; Standard fixed assignment does not (`GRB-M-001`). |
| `GRB-C010` | Certified 전투력과 Floor 난이도, L204-227 (`GRB-L0206-0223`) | mixed current/approved_pending | `E-FACILITY`, combat-power and checkpoint tests | `mixed`: power/anchor semantics align for the shared implementation, but Standard is not exempt (`GRB-M-001`). |
| `GRB-C011` | Safe Node와 Safezone, L228-241 (`GRB-L0233-0240`) | current | `E-SAFE`, `E-EQUIPMENT` | `aligned`. |
| `GRB-C012` | Event Node, L242-276 (`GRB-L0248-0274`) | mixed current/approved_pending | `E-SAFE`, `E-COMBAT`, event tests | `mixed`: Event checks and scenes align; Event-started combat failure semantics are pending (`GRB-M-013`). |
| `GRB-C013` | 일반 인벤토리, L277-288 (`GRB-L0282-0287`) | current | `E-SAFE`, inventory tests and Unity fixture | `aligned`. |
| `GRB-C014` | 소비 아이템, L289-301 (`GRB-L0293-0300`) | current | `E-EQUIPMENT`, world equipment/combat tests | `aligned`. |
| `GRB-C015` | 상점, L302-318 (`GRB-L0308-0316`) | current | `E-SAFE`, shop validation/session tests | `aligned`. |
| `GRB-C016` | 노드 진입 전 브리핑, L319-335 (`GRB-L0325-0330`) | current | `E-FACILITY`, combat-preview tests, `E-TRANSPORT` | `aligned`. |
| `GRB-C017` | 전투 intro, L336-342 | delegated | `E-DOC`, `E-COMBAT`, `E-TRANSPORT` | `delegated_detail`. |
| `GRB-C018` | 전투 모드, L343-359 (`GRB-L0347-0353`) | current/future exclusions | `E-COMBAT`, live PVE | `aligned`; excluded legacy mission variants are absent from live assignment. |
| `GRB-C019` | 전술 포인트, L360-380 (`GRB-L0366-0378`) | current | `E-COMBAT`, `run/policy.ron`, tactical-resource tests | `aligned`. |
| `GRB-C020` | Hard CC와 Silence, L381-402 (`GRB-L0388-0401`) | current | `E-COMBAT`, buff/control tests | `aligned`. |
| `GRB-C021` | 철수와 재배치, L403-430 (`GRB-L0409-0427`) | current | `E-COMBAT`, withdrawal/redeployment tests | `aligned`. |
| `GRB-C022` | Attack/Defense 태세와 타일 효과, L431-457 (`GRB-L0437-0453`) | current | `E-COMBAT`, `tactical_stance.rs` tests, weapon profiles | `aligned`. |
| `GRB-C023` | 전술 이동과 아군 점유, L458-500 (`GRB-L0463-0498`) | current | `E-COMBAT`, relocation/swap/occupancy tests | `aligned`. |
| `GRB-C024` | 수동 지정 스킬, L501-530 (`GRB-L0507-0527`) | current | `E-COMBAT`, targeting canonical, manual-target tests | `aligned`. |
| `GRB-C025` | 현장 작업, L531-558 (`GRB-L0538-0552`) | current | `E-COMBAT`, `field_work.rs`, live Freischutz tests/data | `aligned`. |
| `GRB-C026` | Pause와 직렬 전투 명령, L559-577 (`GRB-L0565-0575`) | current | `E-COMBAT`, pause/revision tests, `E-TRANSPORT` | `aligned`. |
| `GRB-C027` | 전투 시작과 Unity 경계, L578-591 (`GRB-L0582-0586`) | delegated | `E-TRANSPORT`, server transport tests, external canonicals | `delegated_detail`; no producer/consumer shape conflict found in the current source. |
| `GRB-C028` | 전장과 배치, L592-608 (`GRB-L0596-0604`) | current/delegated | `E-COMBAT`, battlefield data/validation | `aligned` with detailed authoring delegated. |
| `GRB-C029` | 적 구성과 웨이브, L609-632 (`GRB-L0616-0631`) | mixed current/approved_pending/future | `E-COMBAT`, live PVE | `mixed`: wave contract aligns; Standard Floor 3 class is `FinalBoss`, not stated `NormalBoss` (`GRB-M-001`). |
| `GRB-C030` | Live Boss Runtime Reference, L633-651 (`GRB-L0638-0649`) | current reference | `E-COMBAT`, live boss-runtime RON/tests | `aligned`. |
| `GRB-C031` | 기본 공격과 스킬 타겟팅, L652-667 (`GRB-L0656-0662`) | current/delegated | targeting canonical and `E-COMBAT` | `delegated_detail`; the stated top-level candidate rules align. |
| `GRB-C032` | 피해 타입과 피드백, L668-683 (`GRB-L0677-0680`) | current | `E-COMBAT`, feedback-tag tests | `aligned`. |
| `GRB-C033` | Runtime 스탯·HP·사망 전이, L684-702 (`GRB-L0689-0700`) | current | `E-COMBAT`, stat/death causal-order tests | `aligned`. |
| `GRB-C034` | 성장 장 intro, L703-709 | delegated | `E-DOC`, growth/reward paths | `delegated_detail`. |
| `GRB-C035` | 보상과 성장, L710-731 (`GRB-L0720-0730`) | mixed current/approved_pending | `E-COMBAT`, `E-RESEARCH`, `E-SAFE` | XP/GrowthStack and ClaimAll align; fragment-source exclusivity does not (`GRB-M-002`, `GRB-M-008`, `GRB-M-009`). |
| `GRB-C036` | 연구도 폐기와 스킬 파편 획득, L732-762 (`GRB-L0741-0760`) | approved_pending | `E-RESEARCH`, `E-POOL`, `E-TRANSPORT` | `mixed document_ahead_expected/legacy_live_path`: `GRB-M-002`, `GRB-M-008`, `GRB-M-009`. |
| `GRB-C037` | 영구 도감·튜토리얼 해금·개인 기록, L763-788 (`GRB-L0767-0786`) | mixed current/approved_pending | `E-PROFILE`, `E-TRANSPORT` | compendium aligns; tutorial unlock and three records are absent (`GRB-M-004`). |
| `GRB-C038` | Endless 격파와 반복 조우, L789-801 (`GRB-L0794-0799`) | approved_pending | `E-RESEARCH`, `E-POOL`, `E-BOSSOMEN` | `mixed`: target ledgers/pools are not the current authority (`GRB-M-003`, `GRB-M-005`, `GRB-M-008`, `GRB-M-010`, `GRB-M-012`). |
| `GRB-C039` | Elite assignment pool, L802-824 (`GRB-L0806-0821`) | approved_pending | `E-POOL` | `runtime_contradiction`: `GRB-M-005`. |
| `GRB-C040` | NormalBoss assignment pool, L825-833 (`GRB-L0827-0832`) | approved_pending | `E-POOL`, `E-RESEARCH` | `document_ahead_expected`: `GRB-M-008`, `GRB-M-010`. |
| `GRB-C041` | Elite 모노리스와 강화 재현, L834-880 (`GRB-L0840-0878`) | mixed current/approved_pending/future | `E-POOL`, monolith live policy/tests | merchant/stage/residual core exists; pool, values, permanent failure and fragment reward differ (`GRB-M-005`, `GRB-M-006`, `GRB-M-008`, `GRB-M-014`). Corrupted-monolith consumer is explicitly dormant. |
| `GRB-C042` | 숙달 Elite 소탕, L881-901 (`GRB-L0886-0899`) | approved_pending | repository-wide Core/data/Unity search | `document_ahead_expected`: `GRB-M-007`. |
| `GRB-C043` | 장비, L902-927 (`GRB-L0906-0926`) | current | `E-EQUIPMENT`, live weapon-catalog test | `aligned`, including the exact first-live weapon catalog and `standard_armor` recipe scope. |
| `GRB-C044` | 장비 전투 트리거, L928-947 (`GRB-L0933-0942`) | current | `E-EQUIPMENT`, item occurrence/activation tests | `aligned`. |
| `GRB-C045` | 스킬 파편, L948-968 (`GRB-L0952-0966`) | mixed current/approved_pending | skill-fragment RON/runtime/tests, `E-BOSSOMEN` | ownership/equip compatibility align; formal active-skill and chain-form targets remain placeholders/missing (`GRB-M-008`, `GRB-M-011`, `GRB-M-015`). |
| `GRB-C046` | 비전투 노드 intro, L969-974 | delegated | `E-DOC`, `E-SAFE` | `delegated_detail`. |
| `GRB-C047` | SavePoint, L975-996 (`GRB-L0977-0993`) | current | `E-SAFE`, checkpoint/store tests, `E-TRANSPORT` | `aligned`. |
| `GRB-C048` | Maintenance, L997-1024 (`GRB-L1001-1022`) | current | `E-EQUIPMENT`, maintenance capability/transaction tests | `aligned`. |
| `GRB-C049` | 본사 연락, L1025-1032 (`GRB-L1029-1031`) | mixed current/approved_pending | `E-SAFE`, `E-RESEARCH` | HQ role/options align, but planning-return research delivery remains reachable (`GRB-M-002`). |
| `GRB-C050` | 결과와 실패 intro, L1033-1038 | delegated | `E-DOC`, `E-COMBAT` | `delegated_detail`. |
| `GRB-C051` | 노드 결과 처리, L1039-1060 (`GRB-L1044-1058`) | mixed current/approved_pending | `E-COMBAT`, result/attempt tests | atomic result/XP/attempts align; new fragment/sweep/Event failure ledgers do not (`GRB-M-007`, `GRB-M-008`, `GRB-M-013`, `GRB-M-014`). |
| `GRB-C052` | 후퇴, L1061-1069 (`GRB-L1065-1068`) | current | `E-COMBAT`, repeated-retreat and event-retreat tests | `aligned`. |
| `GRB-C053` | 직원 condition·전투 HP·런 실패, L1070-1098 (`GRB-L1074-1083`) | current | `E-COMBAT`, checkpoint/failure-decision/recovery tests | `aligned`. |
| `GRB-C054` | 직원 Psychology, L1099-1112 (`GRB-L1104-1111`) | current/future exclusions | employee/checkpoint paths and searches | `aligned`; no production Psychology mutation was found. |
| `GRB-C055` | 후속 콘텐츠·구현 authoring, L1113-1126 (`GRB-L1118-1122`) | future authoring | live schema/data inspections | `future_or_dormant`; these missing exact content values are not independently counted as defects, while the implemented old authorities are covered by `GRB-M-008`, `GRB-M-011`. |
| `GRB-C056` | 고급 정책과 레거시 intro, L1127-1128 | current | `E-DOC` | `aligned`. |
| `GRB-C057` | FinalBoss pool과 BossOmen Chain, L1129-1200 (`GRB-L1137-1197`) | mixed current/approved_pending/future | `E-BOSSOMEN`, `E-COMBAT`, `E-PROFILE` | capstone, forced-boss failure and Exit Gate align; eligibility, cycle, source failure and rewards differ (`GRB-M-008`, `GRB-M-011`, `GRB-M-012`, `GRB-M-013`). Hidden-boss implementation is explicitly follow-up and classified `future_or_dormant`. |
| `GRB-C058` | 레거시 제거 원칙, L1201-1210 (`GRB-L1207-1209`) | current/delegated | `E-LEGACY`, `E-TRANSPORT` | `aligned`: old delta transport and Core `ChooseSupport` are rejected/absent; Unity HQ-local “support” labels are presentation names for current typed HQ actions, not the removed wire/game mode. |

## Matrix verdict

- Coverage: all 1,210 lines, 58 headings and 365 list-item claims are assigned to one contiguous
  coverage row.
- Current mismatches: 15 findings, detailed in `MISMATCH_CATALOG.md`.
- Evidence gaps: no missing source boundary prevents a gameplay finding. External Unity source and
  tests were readable, but the Unity runner was unavailable; therefore Unity statements above are
  source/test-assertion inspection, not a claimed fresh Unity test execution.
- No production code, live data, fixture/test, canonical or external Unity file was changed.
