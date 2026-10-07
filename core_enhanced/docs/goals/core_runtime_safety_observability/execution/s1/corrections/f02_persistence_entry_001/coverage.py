"""Bind every required S1 condition to the F02 successor's current evidence."""
import copy
import difflib
import json
from pathlib import Path
import re
import sys
import tarfile
C = Path(__file__).resolve().parent
D = C.parents[3]
sys.path.insert(0, str(D / 'tools'))
import run
R, E = run.ROOT, D / 'evidence'
prior = json.loads((D / 'execution/s1/COVERAGE.json').read_text())
current = run.identity()
old = json.loads((C / 'baseline/SOURCE_BEFORE.json').read_text())
old = {row['path']: row for row in old['files']}
changed = sorted(name for name, row in old.items() if run.sha(R / name) != row['sha256'])
assert changed == ['core_enhanced/docs/core_runtime_contract.md', 'core_enhanced/src/application.rs', 'core_enhanced/src/game/world.rs', 'core_enhanced/src/game/world/application.rs', 'game_server/src/game/player_game_actor/handlers.rs']
with tarfile.open(C / 'baseline/SOURCE_BEFORE.tar.gz') as archive:
    patch = ''.join(''.join(difflib.unified_diff(archive.extractfile(name).read().decode().splitlines(True), (R / name).read_text().splitlines(True), 'a/' + name, 'b/' + name)) for name in changed)
(C / 'SCOPED.patch').write_text(patch)
# F01's public rollback guard is byte-identical to the audited correction.
with tarfile.open(C / 'baseline/SOURCE_BEFORE.tar.gz') as archive:
    before_app = archive.extractfile('core_enhanced/src/application.rs').read().decode()
after_app = (R / 'core_enhanced/src/application.rs').read_text()
def rollback_port(text):
    return text.split('    pub fn restore_persistence_rollback(', 1)[1].split('    pub fn has_active_battle(', 1)[0]
assert rollback_port(before_app) == rollback_port(after_app)

suites = ['s1f02_all_default_001', 's1f02_all_feature_001', 's1f02_server_002']
checks = ['s1f02_' + name + '_001' for name in ['fmt', 'event_log', 'clippy_core', 'clippy_server', 'release', 'public_probe', 'boundary', 'default_parity', 'public_oracle', 'public_trace', 'public_cli', 'public_profile', 'public_boundary', 'public_api', 'pc_prepare', 'pc', 'pc_compare', 'r05_compare']]
checks += ['s1f02_check_002', 's1f02_r05_001', 's1f02_pc_immediate_002']
bindings, passed, totals = [], {}, {}
for name in suites + checks:
    path = E / (name + '.json')
    value = json.loads(path.read_text())
    assert value['source_unchanged'] and value['source_after'] == current, name
    assert value['exit_code'] in ((0, 101) if name == 's1f02_r05_001' else (0,)), name
    log = E / (name + '.log')
    assert value['log_sha256'] == run.sha(log), name
    bindings.append(dict(name=name, receipt_sha256=run.sha(path), current=True, exit_code=value['exit_code']))
    if name in suites:
        text = log.read_text()
        passed[name] = set(re.findall(r'^test (\S+) \.\.\. ok$', text, re.M))
        rows = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored', text)
        totals[name] = [sum(int(row[i]) for row in rows) for i in range(3)]
        assert totals[name][0] > 0 and totals[name][1:] == [0, 0]
core = passed[suites[0]] & passed[suites[1]]
host = passed[suites[2]]
remap = {'default_parity_002':'default_parity', 'public_profile_003':'public_profile', 'public_api_cli_001':'public_api', 'public_oracle_001':'public_oracle', 'public_cli_001':'public_cli', 'public_boundary_002':'public_boundary', 'pc_compare_002':'pc_compare', 'clippy_server_001':'clippy_server', 'release_001':'release', 'event_log_check_001':'event_log', 'clippy_core_001':'clippy_core', 'fmt_001':'fmt', 'public_trace_001':'public_trace', 'r05_compare_002':'r05_compare'}
validations = copy.deepcopy(prior['validations'])
for v in validations:
    assert set(v['core_selectors']) <= core
    assert set(v['host_selectors']) <= host
    v['suite_receipts'] = suites
    v['direct_receipts'] = [('s1f02_server_002' if name == 'server_all_targets_001' else 's1f02_check_002' if name == 'check_001' else 's1f02_' + remap[name] + '_001') for name in v['direct_receipts']]
    v['status'] = 'current F02 successor evidence verified; independent audit pending'
    if v['id'] in ('V04', 'V08', 'V12'):
        v['direct_receipts'] += ['s1f02_public_probe_001', 's1f02_boundary_001']
        v['core_selectors'] += ['application::tests::prepared_command_preserves_first_authoring_fault_and_original_root']
        v['host_selectors'] += ['game::player_game_actor::handlers::tests::prepared_battle_entry_drop_preserves_root_and_commit_matches_direct_execution']
        assert set(v['core_selectors']) <= core and set(v['host_selectors']) <= host
impact = json.loads((D / 'execution/s1/PREDECESSOR_IMPACT.json').read_text())
for condition in impact['inherited_conditions']:
    assert set(condition['selectors']) <= core
    condition['receipts'] = suites[:2]
    condition['current_impact'] = 'Shared GameCore application preparation/commit extraction; exact operation/validation/allowance path retained. Current full selectors, public oracles/trace/CLI and PC36/R05 rerun; independent semantic assessment required.'
    condition['disposition'] = 'current selectors verified; prior acceptance is preserved only subject to final independent semantic/restoration audit'
assert set(impact['f01_selectors']) <= core
impact.update(current_source_sha256=current['sha256'], correction_changed=changed, public_oracle='evidence/public_after/SUMMARY.json', public_trace='evidence/public_after_trace/SUMMARY.json', public_cli='evidence/cli_current/CLI_FINAL_SUMMARY_002.json', independent_audit=False, prior_audit='../../../../post_audit/s1_20261004_002/COMPLETION_AUDIT.md')
for path in ('public_probe_001/RESULT.json', 'boundary_001/RESULT.json', 'evidence/PC_IMMEDIATE_PARITY.json', 'evidence/PC_SUMMARY_CURRENT_002.json', 'evidence/R05_SUMMARY_CURRENT_002.json'):
    assert (C / path).is_file()
assert json.loads((C / 'evidence/PC_IMMEDIATE_PARITY.json').read_text())['passed']
r05 = json.loads((C / 'evidence/R05_SUMMARY_CURRENT_002.json').read_text())
assert not r05['other_hard_guard_failures']
result = copy.deepcopy(prior)
result.update(source_sha256=current['sha256'], status='F02 correction and current self-verification complete; independent audit pending', validations=validations, suites=totals, receipt_bindings=bindings, cost_evidence={'PC36':'evidence/PC_SUMMARY_CURRENT_002.json', 'PC36_immediate':'evidence/PC_IMMEDIATE_PARITY.json', 'R05':'evidence/R05_SUMMARY_CURRENT_002.json'}, correction=dict(finding='S1 audit002 F02; F01 guard retained', changed=changed, patch_sha256=run.sha(C / 'SCOPED.patch'), production_delta='Opaque exclusively borrowed prepared non-battle application; host durably publishes changed battle-entry checkpoint before infallible commit. No active-battle undo, gameplay re-execution or extra clone.', public_probe='public_probe_001/RESULT.json', public_boundary='boundary_001/RESULT.json', host_entry_persistence='Real production actor with external R06 content and existing SavePoint; peak204 -> staged213; failed tmp publish leaves original run/disk unchanged and no battle; remove fault then retry persists213 and enters battle.'), previous_target_sha256=run.sha(D / 'execution/s1/corrections/f01_active_battle_001/target/s1_f01_001/MANIFEST.json'), reuse='No previous successful execution receipt substitutes for current S1 suite/public/cost verification. Frozen policy and unchanged numeric baseline remain historical inputs.')
for name, value in [('COVERAGE.json', result), ('PREDECESSOR_IMPACT.json', impact)]:
    with (C / name).open('x') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2); stream.write('\n')
print(json.dumps(dict(source_sha256=current['sha256'], changed=changed, suites=totals, validations=len(validations), inherited_conditions=len(impact['inherited_conditions']), r05_classification=r05['classification'])))
