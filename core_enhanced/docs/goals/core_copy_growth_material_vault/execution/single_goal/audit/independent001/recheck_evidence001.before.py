"""Read frozen raw evidence; independently recount results and check audit probes.
Usage: python3 recheck_evidence.py <new-result.json>
No product execution or frozen artifact writes.
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
load = lambda p: json.loads(p.read_bytes())
sha = lambda b: hashlib.sha256(b).hexdigest()
result = {'target': 'u15_current_001', 'product_tests_rerun': False}

# Raw libtest names, not summed overlapping headline counts.
logs = {name: (E / f'runs/{name}.log').read_text() for name in
        ['core002', 'focused005', 'server001', 'g11-native001', 'socket002']}
names = lambda text, status: re.findall(r'^test (\S+) \.\.\. ' + status + r'$', text, re.M)
failed = set(names(logs['core002'], 'FAILED'))
assert len(failed) == 6 and failed <= set(names(logs['focused005'], 'ok'))
passed = set().union(*(set(names(logs[n], 'ok')) for n in ['core002', 'focused005', 'socket002']))
assert len(passed) == 2180, len(passed)
assert len(names(logs['focused005'], 'ok')) == 37
assert len(names(logs['server001'], 'ok')) == 29
assert len(names(logs['g11-native001'], 'ok')) == 4
assert len(names(logs['socket002'], 'ok')) == 1
result['rust'] = {'unique_default_core_passed': len(passed), 'closed_failure_names': sorted(failed),
                  'focused_passed': 37, 'native_passed': 4, 'server_passed': 29,
                  'socket_passed': 1, 'existing_ignored': 6, 'single_full_green_claimed': False}

# Retained public traces: actual payment and independent small-number refund oracle.
traces = {n: load(E / f'runs/normal_host001/phase{n}/commands.json') for n in range(1, 8)}
assert sum(map(len, traces.values())) == 165
for n in range(1, 8):
    observed = load(E / f'runs/normal_host001/phase{n}/RESULT.json')
    assert observed['passed'] and observed['fresh_process'] and observed['commands'] == len(traces[n])
for n in [2, 3]:
    for row in traces[n]:
        if row.get('result', {}).get('kind') == 'skill_fragment_awakened':
            payload = row['result']['payload']
            assert payload['enkephalin_spent'] == 7
            assert payload['copy']['investment']['materials'] == {
                'u15_part_a': {'carried': 3, 'warehouse': 2},
                'u15_part_b': {'carried': 3, 'warehouse': 0}}
for n, expected in [(3, {'u15_part_a': 4, 'u15_part_b': 1}),
                    (5, {'u15_part_a': 7, 'u15_part_b': 4})]:
    rows = [r for r in traces[n] if r.get('result', {}).get('kind') == 'skill_fragment_dismantled']
    assert len(rows) == 1
    payload = rows[0]['result']['payload']
    balances = {v['material_id']: v['amount'] for v in payload['inventory_diff']['material_stacks']}
    assert all(balances[k] == v for k, v in expected.items())
    assert payload['enkephalin_gained'] == 0
old_io = [load(E / f'runs/normal_host001/phase{n}/IO_FAILURE.json') for n in [2, 3, 4]]
assert ['os error 21' in v['error'] for v in old_io] == [True, True, False]
assert old_io[2]['index'] == 29 and old_io[2]['command']['materials'][0]['amount'] == 3
result['historical_host'] = {'processes': 7, 'commands': 165, 'actual_io_failures': 2,
                             'phase4_is_domain_rejection_not_io': True,
                             'refund_oracle': 'part A paid 5 -> 2, part B paid 3 -> 1; carried 2/0 -> 4/1 and 5/3 -> 7/4'}

# Audit supplement uses the valid return and requires filesystem error, then retry.
fresh = A / 'return_io001'
for n in range(1, 6):
    assert load(fresh / f'phase{n}.process.json')['exit_code'] == 0
    assert load(fresh / f'phase{n}/RESULT.json')['passed']
for n in [2, 3, 4]:
    failure = load(fresh / f'phase{n}/IO_FAILURE.json')
    assert 'os error 21' in failure['error'] and failure['memory_and_disk_unchanged']
assert load(fresh / 'phase4/IO_FAILURE.json')['index'] == 30
result['audit_host'] = {'fresh_processes': 5,
                        'commands': sum(load(fresh / f'phase{n}/RESULT.json')['commands'] for n in range(1, 6)),
                        'actual_io_failures_and_retries': 3,
                        'fresh_reloan_and_refund_after_return_retry': True}

# Raw allocator bytes, operation success and finite workload; no retained/RSS conflation.
cost = {}
for batch, count in [('current_cost001', 9), ('paired_cost001', 18), ('paid_cost001', 6)]:
    rows = [(p, load(p)) for p in (E / 'runs' / batch).glob('*.json')]
    rows = [(p, v) for p, v in rows if isinstance(v, dict) and 'stages' in v and 'release' in v]
    assert len(rows) == count
    for p, v in rows:
        assert v['passed'] and len(v['stages']) == 12 and all(s['operation_ok'] for s in v['stages']), p
        release = v['release']
        assert release['warm_generation_baseline']['live_requested_bytes'] == release['final_after_all_owners']['live_requested_bytes'], p
        assert all(not s['heap']['baseline']['counter_overflowed'] and not s['heap']['end']['counter_overflowed'] for s in v['stages'])
    cost[batch] = {'processes': count, 'operation_stages': count * 12, 'exact_requested_bytes_recovered': True,
                   'max_stage_seconds': max(s['wall_seconds'] for _, v in rows for s in v['stages'])}
result['cost_raw_recount'] = cost

authored = load(E / 'runs/authoring001/RESULT.json')['cases']
assert len(authored) == 10 and sum(c['accepted'] for c in authored) == 1
legacy = load(E / 'runs/old_save_reject002/RESULT.json')
assert legacy['profile_version'] == 5 and legacy['bytes_unchanged'] and 'unsupported player profile version 5' in legacy['unsupported_error']
with tarfile.open(E / 'runs/old_save_reject002.originals.tar.gz') as archive:
    originals = {m.name: sha(archive.extractfile(m).read()) for m in archive if m.isfile()}
for name, digest in legacy['original_files'].items():
    assert any(p.endswith(name) and h == digest for p, h in originals.items()), name
result['authoring_and_legacy'] = {'valid': 1, 'invalid_rejected': 9, 'legacy_original_files_verified': 4}

manifest = load(E / 'target/u15_current_001/SOURCE_MANIFEST.json')['files']
member_paths = {r['path'] for r in manifest}
extra = []
for folder in ['core_enhanced/src', 'game_server/src', 'game_resources/data']:
    extra += [str(p.relative_to(R)) for p in (R / folder).rglob('*') if p.is_file() and str(p.relative_to(R)) not in member_paths]
assert not extra, extra
result['production_root_omissions'] = extra
result['passed'] = True
with Path(sys.argv[1]).open('x') as output:
    json.dump(result, output, ensure_ascii=False, indent=2)
    output.write('\n')
print(json.dumps(result, ensure_ascii=False, indent=2))
