"""Verify current correction suites and explicitly limited predecessor reuse."""
import copy
import difflib
import json
import re
import sys
import tarfile
from pathlib import Path

C = Path(__file__).resolve().parent
D = C.parents[3]
sys.path.insert(0, str(D / 'tools'))
import run

R, E = run.ROOT, D / 'evidence'
prior = json.loads((D / 'execution/s1/COVERAGE.json').read_text())
manifest = json.loads((D / 'execution/s1/target/s1_001/MANIFEST.json').read_text())
current = run.identity()
old = {r['path']: r for r in manifest['product_source']['files']}
now = {r['path']: r for r in current['files']}
assert set(old) == set(now)
changed = sorted(name for name in old if old[name] != now[name])
assert changed == ['core_enhanced/src/application.rs', 'game_server/src/game/player_game_actor/handlers.rs']
with tarfile.open(C / 'baseline/SOURCE_BEFORE.tar.gz') as archive:
    before = {name: archive.extractfile(name).read().decode() for name in changed}
after = {name: (R / name).read_text() for name in changed}
guard = '''        if self.has_active_battle() {
            return Err(ApplicationError::invalid_input(
                ApplicationErrorKind::InvalidAction,
                "active_battle_persistence_rollback",
                "persistence rollback is unavailable during an active battle",
            ));
        }
'''
assert after[changed[0]].count(guard) == 1
assert after[changed[0]].replace(guard, '') == before[changed[0]]
# The second changed file has no production delta; only one existing test expands.
test_name = '    #[test]\n    fn first_public_battle_entry_uses_host_work_limit_and_active_battle_has_no_rollback_token()'
def outside_test(text):
    begin = text.index(test_name)
    end = text.index('    fn start_new_game_with_default_starters(', begin)
    return text[:begin] + text[end:]
assert outside_test(after[changed[1]]) == outside_test(before[changed[1]])
patch = ''.join(''.join(difflib.unified_diff(before[name].splitlines(True), after[name].splitlines(True), 'a/' + name, 'b/' + name)) for name in changed)
(C / 'SCOPED.patch').write_text(patch)

suite_names = ['s1f01_all_default_001', 's1f01_all_feature_001', 's1f01_server_001']
checks = ['s1f01_check_001', 's1f01_event_log_001', 's1f01_clippy_core_001', 's1f01_clippy_server_001', 's1f01_fmt_002', 's1f01_release_001', 's1f01_public_probe_002']
passed, totals, bindings = {}, {}, []
for name in suite_names + checks:
    path = E / (name + '.json')
    value = json.loads(path.read_text())
    assert value['exit_code'] == 0 and value['source_unchanged'] and value['source_after'] == current, name
    log = E / (name + '.log')
    assert value['log_sha256'] == run.sha(log)
    bindings.append(dict(name=name, sha256=run.sha(path), current=True))
    if name in suite_names:
        text = log.read_text()
        passed[name] = set(re.findall(r'^test (\S+) \.\.\. ok$', text, re.M))
        rows = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored', text)
        totals[name] = [sum(int(row[i]) for row in rows) for i in range(3)]
        assert totals[name][0] > 0 and totals[name][1:] == [0, 0]
core = passed[suite_names[0]] & passed[suite_names[1]]
host = passed[suite_names[2]]
validations = copy.deepcopy(prior['validations'])
for validation in validations:
    assert set(validation['core_selectors']) <= core
    assert set(validation['host_selectors']) <= host
    validation['suite_receipts'] = suite_names
    validation['status'] = 'current final selectors verified; fresh independent correction audit pending'
    if validation['id'] in ('V04', 'V08', 'V12'):
        validation['correction_receipts'] = ['s1f01_public_probe_002', 's1f01_server_001', 's1f01_all_default_001', 's1f01_all_feature_001']

reused = []
new_tests = {row['name'] for row in bindings}
for entry in prior['receipt_bindings']:
    name = entry['name']
    receipt_path = E / (name + '.json')
    assert run.sha(receipt_path) == entry['receipt_sha256']
    value = json.loads(receipt_path.read_text())
    assert value['exit_code'] == 0 and value['source_unchanged']
    assert value['log_sha256'] == run.sha(E / (name + '.log'))
    for row in value['source_after']['files']:
        assert row['path'] in changed or now.get(row['path']) == row
    reused.append(dict(name=name, receipt_sha256=entry['receipt_sha256'], disposition='historical exact evidence retained; final suites/checks/release superseded by current correction receipts; unaffected profile/CLI/API/cost results reused only under verified guard-only production delta', effect='The sole production addition rejects public persistence restore during an active battle before revision advance or private route. Native/authoring/expression/history algorithms, operational defaults, costs and CLI commands are byte-identical. Host production code is byte-identical; normal non-battle persistence failure behavior runs in current server suite. Public negative API traits/signatures are unchanged.'))

impact = json.loads((D / 'execution/s1/PREDECESSOR_IMPACT.json').read_text())
for condition in impact['inherited_conditions']:
    assert set(condition['selectors']) <= core
    condition['receipts'] = suite_names[:2]
    condition['disposition'] = 'current selectors rerun after guard-only correction; unaffected semantic evidence retained; independent final predecessor disposition pending'
assert set(impact['f01_selectors']) <= core
for key in ('public_oracle', 'public_trace', 'public_cli'):
    impact[key] = impact[key].replace('../../evidence/', '../../../../evidence/')
impact.update(current_source_sha256=current['sha256'], correction_changed=changed, prior_audit='../../../../post_audit/s1_20261004_001/COMPLETION_AUDIT.md', independent_audit=False)
result = copy.deepcopy(prior)
result.update(source_sha256=current['sha256'], status='F01 implementation/self-verification complete; independent correction audit pending', validations=validations, suites=totals, receipt_bindings=bindings, preserved_prior_evidence=reused, correction=dict(finding='S1 audit F01', production_delta='Active-battle guard at public restore boundary', changed=changed, patch_sha256=run.sha(C / 'SCOPED.patch'), public_probe='public_probe_002/RESULT.json', host_entry_persistence='The public probe verifies unchanged exported durable checkpoint across battle entry. Host handlers compare checkpoint_after to checkpoint_before and publish only on inequality; entry therefore does not enter the persistence rollback branch. Current host suite also exercises independent profile and active-run write failures.'), previous_target_sha256=run.sha(D / 'execution/s1/target/s1_001/MANIFEST.json'))
result['cost_evidence'] = {key: value.replace('../../evidence/', '../../../../evidence/') for key, value in result['cost_evidence'].items()}
for name, value in [('COVERAGE.json', result), ('PREDECESSOR_IMPACT.json', impact)]:
    with (C / name).open('x') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')
print(json.dumps(dict(source_sha256=current['sha256'], changed=changed, suites=totals, validations=len(validations), inherited_conditions=len(impact['inherited_conditions']))))
