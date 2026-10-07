#!/usr/bin/env python3
"""Freeze the U04 review requirements before formal source-bound review."""
import json
import subprocess
from initialize import HERE, ROOT, artifact, write

C = 'core_enhanced/src/'
W = C + 'game/world/'
S = 'game_server/src/'
D = 'core_enhanced/docs/goals/core_runtime_fault_isolation/'
GROUPS = [
 ('RQ01', [1, 2, 3, 10], 'BattleVerdict→employee/resource/reward/progression settlement→ack ownership',
  [W+'combat.rs', W+'reward.rs', W+'grant_state.rs', W+'helpers.rs', C+'game/resources/selection.rs', C+'game/reward.rs', C+'game/reward_policy.rs', C+'game/combat_setup/rewards.rs', C+'game/data/validation.rs', C+'game/employee.rs', C+'game/battle/result_stats.rs'],
  'actual automatic/manual finish, defeat retry removal, ChooseOne context, incapacitation idempotence and consumable finish'),
 ('RQ02', [4, 5, 6, 7, 8, 9, 15], 'node entry/unfinished/complete→retreat/stable Continue/SavePoint roots',
  [W+'node_flow.rs', W+'state.rs', W+'active_run.rs', W+'active_run/v7.rs', W+'active_run/v8.rs', W+'event_node.rs', W+'shop.rs', W+'maintenance.rs', W+'headquarters.rs', W+'map_content.rs', W+'mastered_elite_sweep.rs', C+'game/resources/state.rs'],
  'all current node entry/completion routes, save/load fields and validation, pre-entry versus prebattle restore, latest confirmed before first SavePoint'),
 ('RQ03', [1, 7, 11, 12, 13, 15], 'exclusive CoreApplication preparation/commit→fault/observation→all public mutation ports',
  [C+'application.rs', C+'application/fault.rs', W+'application.rs', C+'game/world.rs', C+'runtime_observation.rs', C+'game/behavior.rs', C+'game/world/admin.rs', C+'game/world/admin/commands.rs', C+'authoring/clone_shape.rs', C+'game/map/progression.rs', C+'game/monolith.rs'],
  'extend sole existing prepared root; tick/manual durable publication without postcommit rollback or speculative fault leakage; performance-sensitive live ticks'),
 ('RQ04', [1, 3, 4, 6, 7, 9, 11, 12, 13, 14], 'actual host socket/actor→command/tick/management→shared persistence',
  [S+'main.rs', S+'game/player_game_actor/mod.rs', S+'game/player_game_actor/handlers.rs', S+'game/player_game_actor/messages.rs', S+'game/player_game_actor/state.rs', S+'game/player_game_actor/session.rs', S+'game/load_balance_actor/mod.rs', S+'game/load_balance_actor/handlers.rs'],
  'tick finishing has no current persistence; early resets, synchronization, disconnect/reconnect, stale actor writes and shared store ownership'),
 ('RQ05', [7, 8, 9, 12, 13, 14], 'V9/Core and V4/active + V3/profile→file publication/journal/recovery',
  [S+'persistence/active_run_store.rs', S+'persistence/player_profile_store.rs', S+'persistence/mod.rs', W+'active_run.rs', 'game_server/config/development.toml', 'game_server/config/production.toml'],
  'actual file sync/rename/lock, unsupported backup/reset/publish hazards, cross-file durable intent and monotonic load counter, local paths/deployed limitation'),
 ('RQ06', [1, 2, 3, 4, 7, 9, 11, 15], 'result/reward/retreat/Continue capability DTO→public/transport consumers',
  [W+'action_capability.rs', W+'battle_capability.rs', W+'snapshot.rs', C+'game/behavior.rs', C+'application.rs', S+'game/player_game_actor/state.rs', S+'game/player_game_actor/messages.rs', 'game_server/docs/unity_bridge_protocol.md', 'core_enhanced/docs/core_runtime_contract.md'],
  'single authoritative eligibility and ack/choice output; preserve normal public calls and update genuine transport consumers'),
 ('RQ07', list(range(1,16)), 'existing unit/public/socket/crash fixtures→U04 acceptance execution seams',
  [W+'tests/combat.rs', W+'tests/node_sessions.rs', W+'tests/checkpoint_validation.rs', W+'tests/map_flow.rs', W+'tests/snapshots_and_start.rs', C+'application/fault_tests.rs', S+'test_support.rs', S+'persistence/active_run_store.rs', S+'persistence/player_profile_store.rs', D+'tools/final_fault_probes.py', D+'tools/matrix.py', 'Cargo.toml', 'Cargo.lock', 'core_enhanced/Cargo.toml', 'game_server/Cargo.toml'],
  'full baseline/current semantic regression; real subprocess crash per publication stage; concurrent choice and observer parity'),
 ('RQ08', [11, 12, 15], 'independent U03 closure→112 inherited conditions and R06 semantic evidence',
  [D+'post_audit/u03_20261004_003/COMPLETION_AUDIT.md', D+'post_audit/u03_20261004_003/END_CURRENTNESS.json', D+'post_audit/u03_20261004_003/AUDIT_CLOSE.json', D+'execution/single_goal/corrections/f02_origin_001/target/u03_003/MANIFEST.json', 'core_enhanced/docs/roadmaps/core_first/execution/u03_closure_20261004_001/PUBLICATION_RECORD.json'],
  'exact unchanged source baseline reuse, protected original proof, impact-specific new verification, no recursive archive duplication'),
]


def main():
    source = json.loads((HERE/'P2G_10_POLICY_001.json').read_text())['artifacts']
    working = next(a for a in source if a['ref']['Artifact Type'] == 'Working Revision')
    rows = []
    for key, elements, claim, paths, validation in GROUPS:
        assert all((ROOT/p).is_file() for p in paths), (key, [p for p in paths if not (ROOT/p).is_file()])
        rows.append({'Requirement ID': key,
            'Exact Policy Element Revision Ref Set': [working['payload']['Current Policy Element Revision Map'][f'PE-CRRP-{n:02}'] for n in elements],
            'Required claim/boundary': claim, 'Source type/role': 'Workspace Actual; actual local adapter and exact predecessor proof',
            'Required identity/location constraint': {'repository': str(ROOT), 'locations': paths, 'dirty_basis': 'Preserve pre-existing work; exact actual bytes, not clean HEAD'},
            'Applicability/Necessity': 'required', 'Applicability condition': None,
            'Policy/Scope Basis Reference': {'Working Revision Ref': working['ref'], 'Scope selectors': ['I01','I02','I03','I04','I05','I06','I07']},
            'Validation requirement': validation})
    plan = artifact('Review Scope/Source Requirement Plan', 'RSP-CRRP-001', {
        'Bound Input Working Revision Ref': working['ref'], 'Requirement Set': rows,
        'Currentness Boundary': 'Workspace source/local Linux host. Configured local persistent_data paths inspected; no deployed remote supplied, no deployed or Windows crash-execution claim. No Unity implementation claim.',
        'Candidate Publication': 'non-current until complete P2G-20 batch'})
    write('P2G_20_SOURCE_PLAN_001.json', {'artifacts': [plan]})
    (HERE/'evidence').mkdir(exist_ok=True)
    with (HERE/'evidence/GIT_STATUS_INTAKE.txt').open('x') as stream:
        subprocess.run(['git','status','--short','--untracked-files=normal'], cwd=ROOT, stdout=stream, check=True)
    with (HERE/'evidence/GIT_DIFF_INTAKE.patch').open('xb') as stream:
        subprocess.run(['git','diff','--binary','--','core_enhanced/src','game_server/src','core_enhanced/Cargo.toml','game_server/Cargo.toml','Cargo.lock','game_server/config'], cwd=ROOT, stdout=stream, check=True)
    print('U04 P2G-20 candidate plan: 8 source requirements, 15 elements; dirty state captured; no current review published')


if __name__ == '__main__':
    main()
