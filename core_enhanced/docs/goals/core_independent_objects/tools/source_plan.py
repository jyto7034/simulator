#!/usr/bin/env python3
"""U07 candidate source requirements and exact actual capture, before review."""
import json
import subprocess
import tarfile
from initialize import HERE, ROOT, artifact, write, sha

C = 'core_enhanced/src/'
B = C + 'game/battle/'
K = B + 'core/'
W = C + 'game/world/'
S = 'game_server/src/'
D = 'game_resources/data/'
P = 'core_enhanced/docs/goals/core_battle_starter_tactics/'
GROUPS = [
    ('RQ01', [1, 3, 4, 5, 6, 7, 10, 11, 18], 'normal definitions/strict authoring/compilation/generation/capability',
     [C + 'game/data/' + x for x in ['summoned_unit_data.rs', 'skill_data.rs', 'skill_fragment_data.rs',
       'abnormality_data.rs', 'boss_runtime_data.rs', 'equipment_data.rs', 'live_ron.rs', 'mod.rs', 'validation.rs',
       'content_pack/schema.rs', 'content_pack/raw.rs', 'content_pack/canonical.rs', 'content_pack/mod.rs',
       'content_pack/resolver.rs', 'content_pack/snapshot.rs']] +
     [C + 'game/ability.rs', C + 'game/ability/sustained.rs', C + 'game/ability/authoring.rs',
      C + 'authoring/compiled.rs', C + 'authoring/definition.rs', C + 'authoring/view.rs', C + 'authoring/mod.rs'] +
     [D + x for x in ['bundle.ron', 'summoned_units/base.ron', 'skills/base.ron', 'skill_fragments/base.ron',
       'equipments/base.ron', 'boss_runtime/base.ron', 'abnormalities/base.ron']],
     'actual loader→validation→compiled schema→runtime publication; exact capability/strict negative and official current definitions; no parse-only support'),
    ('RQ02', [1, 2, 3, 4, 7, 8, 9, 14], 'common spawn requests and actor/skill producer admission, generation, identity and lifetime',
     [K + x for x in ['spawn_kernel.rs', 'summon_runtime.rs', 'build.rs', 'mod.rs', 'types.rs',
       'boss_runtime.rs', 'sim.rs', 'commands.rs', 'target_usefulness.rs', 'resonance_runtime.rs',
       'skill_runtime/cast.rs', 'skill_runtime/flow.rs', 'skill_runtime/mod.rs']] +
     [B + 'scenario.rs', B + 'types.rs', B + 'factory.rs'] if (ROOT / (B + 'factory.rs')).is_file() else
     [K + x for x in ['spawn_kernel.rs', 'summon_runtime.rs', 'build.rs', 'mod.rs', 'types.rs',
       'boss_runtime.rs', 'sim.rs', 'commands.rs', 'target_usefulness.rs', 'resonance_runtime.rs',
       'skill_runtime/cast.rs', 'skill_runtime/flow.rs', 'skill_runtime/mod.rs']] + [B + 'scenario.rs', B + 'types.rs'],
     'normal effect→accepted action→atomic spawn and actor behavior; old boss Side/route/authority assumptions; preflight/late failure/withdraw independent child'),
    ('RQ03', [3, 4, 5, 6, 7, 9, 12, 16], 'spatial objects, real movement/contact/targeting/field ticks and timestamp ordering',
     [K + x for x in ['basic_attack.rs', 'targeting.rs', 'queue.rs', 'field_work.rs', 'triggers.rs',
       'death_runtime.rs', 'skill_runtime/area.rs', 'skill_runtime/projectile.rs']] +
     [K + 'movement/' + x for x in ['mod.rs', 'blocking.rs', 'engine.rs', 'planner.rs', 'lifecycle.rs',
       'types.rs', 'path.rs', 'steering.rs', 'rapier_backend.rs']] +
     [B + x for x in ['battlefield/mod.rs', 'battlefield/field.rs', 'enums.rs', 'event_log.rs', 'tile_range.rs']],
     'actual segment→stop/attack/release/resume route; wall is distinct from employee and targetless field; ordering, stale invalidation and object damage observation'),
    ('RQ04', [7, 10, 11, 12, 13, 14], 'HP/shield/segmented resources, cost and death authoritative mutation/readers',
     [K + x for x in ['sustained.rs', 'damage_runtime.rs', 'death_runtime.rs', 'buffs_runtime.rs',
       'tactical_stance.rs', 'tactical_relocation.rs', 'tactical_swap.rs', 'resource_runtime.rs'] if (ROOT / (K + x)).is_file()] +
     [C + 'game/stats.rs', C + 'game/ability/sustained.rs', B + 'damage.rs', B + 'buffs.rs',
      K + 'movement/blocking.rs', K + 'commands.rs', K + 'sim.rs'],
     'single remaining pool, mitigation once/actual HP loss, partial top segment detach, nonlethal HP fallback and rollback; maxHP mutation/death/proc order'),
    ('RQ05', [2, 4, 5, 6, 7, 8, 9, 10, 14, 19], 'source/copy/action/resource authority and new retained/clone/queue cost',
     [B + x for x in ['runtime_facts.rs', 'runtime_control.rs', 'skill_source.rs', 'recording.rs']] +
     [K + x for x in ['runtime_facts.rs', 'authoring_observation.rs', 'resource_runtime.rs', 'queue.rs']] +
     [C + 'authoring/clone_shape.rs', C + 'authoring/control.rs', C + 'game/resource/runtime.rs',
      C + 'game/resource/mod.rs', C + 'game/ability/flow.rs', C + 'game/ability/authoring.rs'],
     'actual immutable source versus private lifetime lease, independent execution/source provenance, budget before allocation/copy and deterministic complete release'),
    ('RQ06', [2, 3, 6, 7, 11, 12, 14, 15, 16, 18], 'normal CoreApplication/authoring trial and host command/query consumers',
     [W + x for x in ['combat.rs', 'battle_capability.rs', 'action_capability.rs', 'helpers.rs', 'grant_state.rs',
       'application.rs', 'state.rs', 'snapshot.rs']] +
     [C + 'application.rs', C + 'game/behavior.rs', C + 'game/combat_preview/mod.rs',
      C + 'game/combat_preview/types.rs', C + 'authoring/trial.rs', C + 'authoring/query.rs',
      C + 'authoring/scenario.rs', C + 'authoring/resolve.rs'] +
     [S + 'game/player_game_actor/' + x for x in ['handlers.rs', 'messages.rs', 'state.rs', 'session.rs', 'mod.rs']] +
     ['game_server/docs/unity_bridge_protocol.md'],
     'actual accepted input→object eligibility→candidate mutation→snapshot/facts and serialized host; preview no writes and current revalidation; strict sample packs/trial'),
    ('RQ07', [2, 7, 8, 9, 10, 15, 16, 17, 19], 'live clone/fault/reconnect and persistent pre-entry Continue/SavePoint',
     [W + x for x in ['active_run.rs', 'active_run/continuation.rs', 'resource_runtime.rs', 'node_flow.rs',
       'authoring_observation.rs']] +
     [C + 'game/data/content_pack/snapshot.rs', K + 'types.rs', K + 'mod.rs', C + 'authoring/clone_shape.rs'] +
     [S + 'persistence/' + x for x in ['active_run_store.rs', 'player_profile_store.rs', 'mod.rs']],
     'actual writer/read selection, runtime/generation/source owner captures and independent rollback; existing lifecycle boundaries, no invented midbattle disk persistence'),
    ('RQ08', list(range(1, 21)), 'existing fixtures/selectors/public probes and inherited final acceptance/bounds',
     [K + x for x in ['boss_action_tests.rs', 'summon_runtime.rs', 'spawn_kernel.rs']] +
     [W + 'tests/' + x for x in ['combat.rs', 'checkpoint_validation.rs', 'u06_starter.rs', 'u04_continuation.inc.rs']] +
     [str(p.relative_to(ROOT)) for p in sorted((ROOT / 'core_enhanced/tests').glob('*.rs'))] +
     [S + 'game/player_game_actor/handlers/tests/' + x for x in ['fault.rs', 'durable.rs']] +
     ['Cargo.toml', 'Cargo.lock', 'core_enhanced/Cargo.toml', 'game_server/Cargo.toml',
      'core_enhanced/docs/core_runtime_contract.md', 'core_enhanced/docs/skill_target_contract.md',
      'core_enhanced/docs/data_loading_contract.md', P + 'CODE_REVIEW_001.ko.md',
      P + 'post_audit/u06_20261005_002/COMPLETION_AUDIT.md', P + 'post_audit/u06_20261005_002/AUDIT_CLOSURE.json',
      P + 'execution/single_goal/target/u06_002/MANIFEST.json'],
     'valid source-bound baseline reuse and affected161+31 requirements; actual expected oracles, no new tests claimed; final integrated/public/cost/restore ownership'),
]


def main():
    source = json.loads((HERE / 'P2G_10_POLICY_001.json').read_text())['artifacts']
    working = next(a for a in source if a['ref']['Artifact Type'] == 'Working Revision')
    assert json.loads((HERE / 'P2G_10_RESULT_001.json').read_text())['is_p2g10_complete']
    rows = []; paths = set()
    for key, elements, claim, locations, validation in GROUPS:
        missing = [p for p in locations if not (ROOT / p).is_file()]
        assert not missing, (key, missing)
        paths.update(locations)
        rows.append({'Requirement ID': key,
          'Exact Policy Element Revision Ref Set': [working['payload']['Current Policy Element Revision Map'][f'PE-CIOB-{n:02}'] for n in elements],
          'Required claim/boundary': claim, 'Source type/role': 'Workspace Actual',
          'Required identity/location constraint': {'repository': str(ROOT), 'locations': locations, 'dirty_basis': 'Exact actual bytes; preserve pre-existing changes'},
          'Applicability/Necessity': 'required', 'Applicability condition': None,
          'Policy/Scope Basis Reference': {'Working Revision Ref': working['ref'], 'Scope selectors': ['I01', 'I02', 'I03', 'I04', 'I05']},
          'Validation requirement': validation})
    for key, role, reason in [
        ('RQ09', 'Released Actual', 'No external release version compatibility target in U07; local release and public authoring verification belong to GEX.'),
        ('RQ10', 'Deployed Actual', 'No deployed-service/Unity implementation scope; actual local host source required in RQ06/07.')]:
        rows.append({'Requirement ID': key, 'Exact Policy Element Revision Ref Set': list(working['payload']['Current Policy Element Revision Map'].values()),
          'Required claim/boundary': role, 'Source type/role': role, 'Required identity/location constraint': None,
          'Applicability/Necessity': 'not_applicable', 'Applicability condition': None,
          'Policy/Scope Basis Reference': {'Working Revision Ref': working['ref'], 'Scope selectors': ['O02', 'O03']},
          'Validation requirement': reason})
    plan = artifact('Review Scope/Source Requirement Plan', 'RSP-CIOB-001', {
        'Bound Input Working Revision Ref': working['ref'], 'Requirement Set': rows,
        'Currentness Boundary': 'Local dirty/untracked workspace and actual Core/host consumers. Prior U06 immutable source/audit is reusable only at exact preserved boundaries. Candidate plan is not a complete review.'})
    write('P2G_20_SOURCE_PLAN_001.json', {'artifacts': [plan]})
    (HERE / 'evidence').mkdir(exist_ok=True)
    with tarfile.open(HERE / 'evidence/REVIEW_SOURCE_001.tar.gz', 'x:gz') as archive:
        for p in sorted(paths):
            archive.add(ROOT / p, arcname=p, recursive=False)
    write('evidence/REVIEW_SOURCE_MANIFEST_001.json', {'plan_ref': plan['ref'],
        'files': [{'path': p, 'sha256': sha(ROOT / p), 'bytes': (ROOT / p).stat().st_size} for p in sorted(paths)],
        'archive_sha256': sha(HERE / 'evidence/REVIEW_SOURCE_001.tar.gz')})
    with (HERE / 'evidence/GIT_STATUS_INTAKE.txt').open('x') as stream:
        subprocess.run(['git', 'status', '--short', '--untracked-files=normal'], cwd=ROOT, stdout=stream, check=True)
    print(f'U07 P2G20 candidate source plan: {len(rows)} requirements/{len(paths)} actual sources; review pending')


if __name__ == '__main__':
    main()
