"""Independently recount frozen evidence. No product run or frozen writes.
Usage: python3 recheck_evidence.py <new-output.json>
"""
from pathlib import Path
import hashlib
import json
import re
import sys
import tarfile

A = Path(__file__).resolve().parent
E = A.parent.parent
R = next(p for p in E.parents if (p / 'game_server/Cargo.toml').is_file())
T = E / 'target/u16_1_current_001'
load = lambda p: json.loads(p.read_bytes())
sha = lambda b: hashlib.sha256(b).hexdigest()
result = {'target': 'u16_1_current_001', 'product_suites_rerun': False}

# Archive identity and all delta preimages/postimages, independently of summary.
def readtar(path):
    with tarfile.open(path) as t:
        return {m.name: t.extractfile(m).read() for m in t if m.isfile()}
before, after = readtar(T / 'BEFORE.tar.gz'), readtar(T / 'SOURCE.tar.gz')
manifest = load(T / 'SOURCE_MANIFEST.json')['files']
assert set(after) == {r['path'] for r in manifest}
assert all(sha(after[r['path']]) == r['sha256'] and len(after[r['path']]) == r['bytes'] for r in manifest)
delta = load(T / 'DELTA.json')
changed = {p for p in before.keys() | after.keys() if before.get(p) != after.get(p)}
assert changed == {r['path'] for r in delta}
for row in delta:
    p = row['path']
    assert row['before_sha256'] == (sha(before[p]) if p in before else None)
    assert row['after_sha256'] == (sha(after[p]) if p in after else None)
result['archive'] = {'before_members': len(before), 'after_members': len(after), 'delta_members': len(delta), 'archive_manifest_equality': True, 'prior_guarded_restore_reused': load(T / 'RESTORE.json')['passed']}

# Exact nonzero libtest successes; preserve original six failures and ignored tests.
logs = {p.name: p.read_text() for p in (E / 'runs').glob('*.log')}
names = lambda s, state: set(re.findall(r'^test (\S+)(?: - should panic)? \.\.\. ' + state + '$', s, re.M))
suite_names = ['core001', 'focused004', 'integration001', 'restore-regression001', 'final-focused001', 'final-public001']
passed = set().union(*(names(logs[n + '.log'], 'ok') for n in suite_names))
socket = R / 'core_enhanced/docs/goals/core_copy_growth_material_vault/execution/single_goal/runs/socket002.log'
passed |= names(socket.read_text(), 'ok')
failures = names(logs['core001.log'], 'FAILED')
assert len(failures) == 6 and failures <= names(logs['focused004.log'], 'ok')
index = load(E / 'TEST_RESULTS_INDEX.json')
assert len(passed) == index['default_core_unique'] == 2188
counts = {n: len(names(logs[n + '.log'], 'ok')) for n in suite_names}
assert counts == dict(zip(suite_names, [2108, 13, 72, 44, 28, 15])), counts
assert len(names(logs['native001.log'], 'ok')) == 4
assert len(names(logs['server001.log'], 'ok')) == 31
coverage = load(E / 'COVERAGE.json')
selector_checks = []
for group in ['acceptance', 'dimensions', 'integration_acceptance']:
    for condition in coverage[group]:
        for ev in condition['evidence']:
            path = E / ev['path']
            assert path.is_file(), ev
            for selector in ev['test_selectors']:
                matches = sorted(n for n in names(path.read_text(), 'ok') if selector in n)
                assert matches, (condition['id'], ev)
                selector_checks.append({'condition': condition['id'], 'path': ev['path'], 'selector': selector, 'passed_names': matches})
result['rust'] = {'unique_core_passed': len(passed), 'suites': counts, 'closed_failures': sorted(failures), 'native_passed': 4, 'server_passed': 31, 'existing_ignored': 6, 'single_green_invocation': False}
result['coverage_selector_checks'] = selector_checks

# Count process results and commands; OS failure must precede a successful retry.
host = E / 'runs/normal_host003'
summaries = [load(host / name) for name in ['RESULT.json', 'EVENT_RESULT.json']]
rows = sum((s['rows'] for s in summaries), [])
traces, results = {}, {}
ios = []
for row in rows:
    n = row['name']
    observed = load(host / n / 'RESULT.json')
    trace = load(host / n / 'commands.json')
    assert observed['passed'] and observed['fresh_process']
    assert observed['commands'] == len(trace) == row['commands']
    request = load(E / 'test_parts/host_replays002' / (n + '.json'))
    assert [x['command'] for x in trace] == request['commands']
    assert {x['index'] for x in trace if x.get('rejected')} == set(request.get('reject_indices', []))
    if row['actual_io']:
        failure = load(host / n / 'IO_FAILURE.json')
        assert 'os error 21' in failure['error'] and failure['memory_and_disk_unchanged']
        assert failure['index'] == request['fail_index']
        retry = trace[failure['index']]
        assert retry['command'] == failure['command'] and 'result' in retry and not retry.get('rejected')
        ios.append({'name': n, 'command': failure['command'], 'error': failure['error'], 'successful_retry': True})
    traces[n], results[n] = trace, observed
pilot = load(E / 'runs/normal_pilot002/RESULT.json')
assert pilot['passed'] and pilot['fresh_process']
assert len(rows) + 1 == 15 and sum(len(t) for t in traces.values()) + pilot['commands'] == 84
assert len(ios) == 4
def inv_before(name):
    return next(x['read_model'] for x in load(host / name / 'before.json')['snapshots'] if x['kind'] == 'inventory')
for name in ['shop_rewind_complete004', 'hq_rewind_complete004']:
    assert inv_before(name) == pilot['inventory']
for base in ['shop', 'hq', 'event']:
    done = results[base + '_rewind_complete004']
    continued = results[base + '_completed_continue004']
    assert inv_before(base + '_completed_continue004') == done['inventory'] == continued['inventory']
assert results['shop_unfinished004']['run']['resources']['enkephalin'] == 260
assert results['shop_rewind_complete004']['run']['resources']['enkephalin'] == 253
assert results['shop_savepoint004']['run']['resources']['enkephalin'] == 530
assert results['shop_savepoint004']['inventory'] == pilot['inventory']
assert len(results['hq_unfinished004']['run']['roster']['employees']) == 5
assert results['hq_unfinished004']['run']['resources']['enkephalin'] == 330
assert len(results['hq_unfinished004']['inventory']['equipments']) == len(pilot['inventory']['equipments']) + 4
expanded = results['expanded_purchase004']['inventory']
assert len(expanded['equipments']) == 71 and expanded['occupied_slot_count'] > 60
assert inv_before('expanded_completed_continue004') == results['expanded_completed_continue004']['inventory'] == expanded
receipt = traces['event_unfinished004'][-1]['result']['payload']['event']['committed_check']
assert receipt['raw_roll'] == 20 and receipt['outcome'] == 'critical_success'
assert all(traces['event_rewind_complete004'][i]['result']['payload']['event']['committed_check'] == receipt for i in [2, 3])
result['host'] = {'fresh_processes': 15, 'commands': 84, 'actual_io_and_retry': ios, 'rewind_completed_savepoint_and_fixed_receipt_checked': True, 'expanded_equipment': 71}

authoring = load(E / 'runs/authoring002/RESULT.json')['cases']
assert len(authoring) == 10 and sum(c['accepted'] for c in authoring) == 1
assert all(c['accepted'] == c['expected_accept'] for c in authoring)
supplement = load(E / 'runs/authoring003/RESULT.json')['cases']
assert len(supplement) == 3 and all(c['exit_code'] == 1 and c['expected_diagnostic'] in c['diagnostic'] for c in supplement)
edge = load(E / 'runs/public_edges001/RESULT.json')
assert edge['passed'] and edge['rejected'] == 11
assert all(r['expected'] == r['actual'] for r in edge['rows'])
consumable = load(E / 'runs/consumable007/RESULT.json')
assert consumable['passed'] and len(consumable['rows']) == 2
assert all(r['retreat_preserved'] and r['duplicate_ack_unchanged'] and r['before_modifier']['remaining_combat_nodes'] == 1 for r in consumable['rows'])
result['other_public'] = {'authoring_valid': 1, 'invalid_rejected': 12, 'sale_rejected': 11, 'consumable_actual_outcomes': 2}

cost = [load(E / f'runs/cost003/{name}.{i}.json') for name in ['normal', 'expanded'] for i in range(3)]
for row in cost:
    assert row['passed'] and len(row['stages']) == 12 and all(s['operation_ok'] for s in row['stages'])
    release = row['release']
    assert release['warm_generation_baseline']['live_requested_bytes'] == release['final_after_all_owners']['live_requested_bytes']
    assert not any(s['heap']['baseline']['counter_overflowed'] or s['heap']['end']['counter_overflowed'] for s in row['stages'])
bound = load(E / 'runs/restore_cost001/value_bound.json')
assert bound['passed'] and len(bound['rows']) == 30
assert all(r['actual_peak_growth'] <= r['admitted'] and r['released_exactly'] for r in bound['rows'])
restore = [load(E / f'runs/restore_cost001/continue.{i}.json') for i in range(3)]
for row in restore:
    assert row['passed'] and not row['limits_changed'] and row['checkpoint_bytes'] == 171980 and row['inventory_equipment'] == 71
    assert row['baseline']['live_requested_bytes'] == row['after']['live_requested_bytes']
    assert all(s['operation_ok'] for s in row['stages'])
result['cost'] = {'shared_processes': 6, 'shared_stages': 72, 'value_bound_cases': 30, 'expanded_continue_processes': 3, 'exact_heap_recovery': True, 'limits_changed': False}
result['passed'] = True
with Path(sys.argv[1]).open('x') as f:
    json.dump(result, f, ensure_ascii=False, indent=2)
    f.write('\n')
print(json.dumps({k: v for k, v in result.items() if k != 'coverage_selector_checks'}, ensure_ascii=False, indent=2))
