# P06 Reference Integration Execution Plan

- Goal ID: `GOAL-CWBS-P06-REFERENCE-INTEGRATION`
- Phase ID: `PHASE-CWBS-06-REFERENCE-INTEGRATION`
- Final Execution Node: `WP-CWBS-11`
- Accepted Handoff / Goal Set: `HA-CWBS-P2G60-002 / GS-CWBS-P2G50-002`
- Run Mode / Root Target: `goal-run / GOAL-CWBS-MASTER`
- Authorization Ref: `GEXAUTH-CWBS-001`
- Current Pass: `GEX-20 complete / Complete / High / AUDIT-CWBS-P06-GEX20-001`
- Direct Dependencies: `P01..P05 / Complete / High / current audits`
- Execution Directory:
  `core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/execution/p06_reference_integration`
- Document Set: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`; fresh GEX-20 creates `COMPLETION_AUDIT.md`

## Objective and Scope

Publish one clearly separated nonproduction placeholder FinalBoss reference that proves a representative conditional
Wave, Boss Summon/batch lifecycle, neutral signal, named outcome, outcome policy and Victory-only registration path.
Prove it cannot enter normal BossOmen pools/cycles, fragment/repeat rewards, all-FinalBoss records or ordinary
progression; align current canonical docs/data and run the final Core/server full command set exactly once.

Covered revision: `PER-CWBS-RF-01-CODE-003`; decision: `DR-CWBS-RF-01-001`.

In scope: placeholder-only live RON/reference IDs, explicit exclusion authority where needed, durable integration
fixtures, final repository/canonical audit and integrated full Core/server suites.

Out of scope: lore, final pattern/difficulty/balance/reward design, normal BossOmen pool/cycle membership, production
promotion, Unity, rollout, deployment and external environment mutation.

## Target Contract

- Placeholder identity and data state clearly say nonproduction/reference-only and remain separate from WhiteNight and
  existing Little Red runtime content.
- The live loader publishes enough connected data to set up and run one representative vertical path. Other supported
  combinations remain covered by predecessor focused fixtures rather than being forced into this content.
- Placeholder has no normal BossOmen pool/cycle, fragment/repeat source, all-live-FinalBoss aggregate or ordinary run
  progression authority. Absence alone is acceptable only where the current registry is already closed and exhaustively
  validated; otherwise add an explicit exclusion selector.
- Canonical gameplay/runtime/data documentation describes the implemented Wave/summon/outcome/session/SavePoint/
  compendium contract without claiming Unity, Released/Deployed or production balance status.
- After focused target validation and documentation alignment, run `cargo test -p game_core` and
  `cargo test -p game_server` at the unchanged integrated target. A failed attempt is not a reusable baseline: record
  it, correct only the exposed regression, re-freeze the changed target and rerun the affected full command before the
  untouched server command. A non-zero test count with zero failures is required at the final frozen target.

## Validation and Completion

- live placeholder publication and deterministic representative runtime result
- explicit assertions for every excluded registry/reward/record/progression surface
- cross-phase Kernel/binding/removal/outcome/registration integrity and process/session rollback evidence
- current canonical docs, typed loaders, RON and runtime consumer agreement
- final Core/server full suites with exact test/pass/fail/skip counts and unchanged relevant-input hashes after the run

## Stop Conditions

- placeholder exclusion cannot be proven across current production registries without inventing balance/content policy
- a required production/Unity/deployment mutation or external data write is needed
- final full runner executes zero tests, fails, or relevant inputs change after it
- policy/scope decision or unsafe overlap with existing dirty work is found

## GEX-10 Audit Target Record

- Record ID: `ATR-CWBS-P06-FINAL-001`
- Currentness: `current / unchanged`
- Frozen identity: HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`, dirty current actual, 25 scoped
  code/test/data/canonical members
- Final validation: Core `1,131 / 0`, server `73 / 0`, total `1,204 / 0`; both commands exited `0`
- Pre/post validation comparison: exact 25-member SHA-256 manifest match
- Frozen member hashes:

```text
c8ac53202e412cf34d341a350c977827bcf3ab3731b994b5fed7823c33bae750  game_server/src/game/player_game_actor/handlers.rs
84363d71c15c9c3f71a9aba75aacb45a9ecf89425c6c867d5f4c42a10d761149  core_enhanced/src/game/battle/core/commands.rs
81ea053b37dbf964859f86d1a9d949dcabe00e2608ee8775a33b6be69ffe76b2  core_enhanced/src/game/battle/enums.rs
7f808b8a9384e8677fca3babf9de7042ca1d9029a0b3ea4996f6428838dd95ae  core_enhanced/src/game/events/combat.rs
9cf14de8b9f23243b0ef8cca44656d5febaae41d095e37ac72532e2b4ef5ba2c  core_enhanced/src/game/data/pve_data.rs
6b636e9ecd48d9084435a930f3201d22d253d93b956e74e823d42469c3942e60  core_enhanced/src/game/data/validation.rs
7e6fb60f16cb1639760c42713846e91f97a3621aaa4a9409e1119e6cec0d20e5  core_enhanced/src/game/data/live_ron.rs
ad63b1c148d15bcbf86cd038b84448b1021ca3d564b04c3709259f3c4deb3cb5  core_enhanced/src/game/world/combat.rs
600fcbad2734539d90dfd55c780a5370d1b8e3109774bc6cc39846baafd29c00  core_enhanced/src/game/battle/core/boss_runtime.rs
7e1de2373484efb72f20f5ca4ebf362232870ff5e128566414b6eb08156d3861  core_enhanced/src/game/world/tests/combat.rs
749f9e718a2b85c7d20d770df0f960ac9b02cfff188901841428687d4b4846db  core_enhanced/tests/ron_loading.rs
d07a40f566d9c598a4fecb285b107f923f256b25f2770e8c52b1f90cef278b7f  core_enhanced/tests/live_skill_catalog_audit.rs
9292bd7175c043073698e724a2265a9a56b107d2a5f4884c3ddcd585e54368ac  core_enhanced/tests/skill_runtime_contract.rs
8b2472db6750dcbf883bd0d826320e5a139b9a2ef47f3f460efae378c22fecde  game_resources/data/pve/encounters.ron
58805dbc82ee3d4513c571acf8c034646b1389e070fcaebacd08f84ba4966d04  game_resources/data/boss_runtime/base.ron
2b5173ac02d88a05bdf4e54d722bf36757e36b9de5ee098da5e47937be56f1c0  game_resources/data/summoned_units/base.ron
b1c2edb2af416e754bd445d581278555f6196936832a155bed31fb4bd048c474  game_resources/data/abnormalities/base.ron
19d781da6675927931344c0bc5272ebd673cf59a3df94f5dbd577c1d667b3e74  game_resources/data/skill_fragments/base.ron
8ababe6019c5a4f371b728ec321c7bb72e213131bc93870fed482b4838bdd8b0  game_resources/data/boss_omen/chains.ron
64cc593e09712d6d619306679a97642635fad9deec6d9ad8108dcf4b0ebaa2e7  game_resources/data/map/node_definitions.ron
6387e53807aec270e74c40cec9c3a47baf6a4eb7c7476c1b4a5070f373c37d3c  game_resources/data/map/facility_templates.ron
9389e0644e4f4c4ca75a861c07f423cb484c6f11a5867cf24ee6e8b769f7d8d3  game_resources/data/map/node_assignment_profiles.ron
3aeec0e8e3eab5c57002083e0cead4f29003c31b39c00f5f877cbb40f08ac086  core_enhanced/docs/game_rulebook.md
73bc18d42f11b54a69f6d861c675cda7d35f6e2090f6b34cb23e539e3d8364f6  core_enhanced/docs/core_runtime_contract.md
58c636ceacb990a328644de8bfd44cb5ca85bf744d5ef680eaba00dc081b5c7b  core_enhanced/docs/data_loading_contract.md
```
