"""Read-only independent checks of frozen bytes and raw execution observations."""
from pathlib import Path
import hashlib
import json
import re
import tarfile

A = Path(__file__).resolve().parent
E = A.parent
R = next(p for p in A.parents if (p / 'game_server/Cargo.toml').is_file())
T = E / 'target/u164_current_001'
load = lambda p: json.loads(p.read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
rows = lambda n: [json.loads(s) for s in (E / 'runs/completion_host001' / n / 'trace.jsonl').read_text().splitlines()]
checks = []

def passed(name, **details):
    checks.append({'check': name, 'passed': True, **details})

def archive(name):
    with tarfile.open(T / name) as t:
        return {m.name: t.extractfile(m).read() for m in t if m.isfile()}

before, source = archive('BEFORE.tar.gz'), archive('SOURCE.tar.gz')
delta = load(T / 'DELTA.json')
forward = dict(before)
for d in delta:
    p = d['path']
    assert (sha(forward[p]) if p in forward else None) == d['before_sha256'], p
    if d['operation'] == 'delete':
        del forward[p]
    else:
        assert sha(source[p]) == d['after_sha256'], p
        forward[p] = source[p]
assert forward == source
for d in reversed(delta):
    p = d['path']
    assert (sha(forward[p]) if p in forward else None) == d['after_sha256'], p
    if d['before_sha256'] is None:
        del forward[p]
    else:
        forward[p] = before[p]
assert forward == before
passed('independent exact-byte forward/reverse reconstruction', before=len(before), source=len(source), delta=len(delta))

index = load(E / 'RE_REVIEW_RESULTS_001.json')
successes = {}
for key, item in index.items():
    if 'path' not in item:
        continue
    raw = (R / item['path']).read_bytes()
    assert sha(raw) == item['sha256'], key
    successes[key] = set(re.findall(r'^test (\S+)(?: - should panic)? \.\.\. ok$', raw.decode(), re.M))
lib = set.union(*(successes[k] for k in ['review-core001', 'review-regression-fixes001', 'review-socket001']))
integration = successes['review-integration001'] | successes['review-integration002']
assert len(lib) == 2150 and len(integration) == 72
passed('prior broad unique successes and preserved failures', lib=len(lib), integration=len(integration), ignored=6,
       server_pass=len(successes['review-server001']), server_preexisting_fail=1, rerun=False)

base = load(E / 'completion001/PREPARED_BASE.json')
employees = lambda r: {e['employee_definition_id']: e for e in r['run']['roster']['employees']}
materials = lambda r: {e['material_id']: e['amount'] for e in r['inventory']['equipment_materials']}
assets = lambda r: [r['run'][k] for k in ['resources', 'roster', 'inventory']] + [r['inventory']]
receipts = load(E / 'completion001/RESULT.json')['host_processes']
for receipt in receipts:
    trace = E / 'runs/completion_host001' / receipt['name'] / 'trace.jsonl'
    assert sha(trace.read_bytes()) == receipt['trace_sha256']
    assert receipt['exit_code'] == 0 and receipt['operations'] == len(rows(receipt['name']))
assert len(receipts) == 17 and sum(r['operations'] for r in receipts) == 192
passed('raw host process receipts', processes=17, operations=192, rerun=False)

for name, amount, multiplier in [('full002', 15, 1), ('unrestrained002', 15, 2), ('extract001', 1, 1)]:
    trace = rows(name)
    terminal = next(r['response'] for r in trace if r['response']['run']['game_state'] == 'combat_result')
    assert terminal['run']['resources']['enkephalin'] - base['run']['resources']['enkephalin'] == 10 * amount * multiplier
    for material in ['equipment_dust', 'u15_part_a']:
        assert materials(terminal)[material] - materials(base)[material] == amount * multiplier
    for employee in ['scout_runner', 'breach_guard', 'field_medic']:
        expected = (10 + amount) * multiplier if employee != 'field_medic' else 0
        assert employees(terminal)[employee]['experience'] - employees(base)[employee]['experience'] == expected
    for row in trace:
        r = row['response']
        if r['run']['game_state'] == 'in_battle':
            assert r['run']['resources'] == base['run']['resources']
            assert r['inventory'] == base['inventory']
            assert all(e['experience'] == employees(base)[n]['experience'] for n, e in employees(r).items())
    passed('raw success arithmetic and no interim payment', host=name, multiplier=multiplier)

continues = 0
trace = rows('full002')
for i, row in enumerate(trace):
    q, r = row['request'], row['response']
    command = q.get('command', {})
    if command.get('type') == 'stress_combat_act' and not command['extract'] and r['accepted']:
        prior = trace[i-1]['response']['result']['payload']['battle_update']['checkpoint']
        after = r['result']['payload']['battle_update']['checkpoint']
        for key in ['units', 'deployment', 'tactical_resource', 'sustained_effects', 'battle_time_ms']:
            assert prior[key] == after[key], key
        continues += 1
assert continues == 4
passed('four continuations retain exact checkpoint state')

faults = []
for name in ['full002', 'extract001', 'retreat001', 'savepoint003']:
    trace = rows(name)
    for i, row in enumerate(trace):
        if not row['request'].get('fail'):
            continue
        assert 'os error 21' in row['response']['error'] and not row['response']['accepted']
        assert trace[i+1]['response']['accepted']
        assert {k:v for k,v in row['request'].items() if k != 'fail'} == {k:v for k,v in trace[i+1]['request'].items() if k != 'fail'}
        assert assets(row['response']) == assets(trace[i-1]['response'])
        faults.append(name)
assert len(faults) == 4
passed('four actual IO errors with unchanged assets and identical retry', hosts=faults,
       disk_atomicity='host.rs asserts complete byte equality for both roots')

defeat = next(r['response'] for r in rows('defeat001') if r['response']['run']['game_state'] == 'combat_result')
assert defeat['run']['selected_event']['verdict'] == 'defeat'
assert defeat['run']['selected_event']['rewards'] == []
assert defeat['run']['resources'] == base['run']['resources'] and defeat['inventory'] == base['inventory']
for name in ['scout_runner', 'breach_guard']:
    assert employees(defeat)[name]['experience'] - employees(base)[name]['experience'] == 10
retreat = rows('retreat001')[-1]['response']
assert assets(retreat) == assets(base)
assert assets(rows('retreat001_fresh')[-1]['response']) == assets(retreat)
unfinished = rows('unfinished002')[-1]['response']
assert assets(unfinished) == assets(base) and unfinished['run']['run_progression'] == base['run']['run_progression']
passed('defeat pending loss/base XP, retreat restoration, fresh unfinished rollback')

for original, fresh in [('full002','full_fresh002'),('unrestrained002','unrestrained_fresh002'),('extract001','extract001_fresh'),('defeat001','defeat_fresh001'),('savepoint003','savepoint_fresh003'),('expanded006','expanded_fresh006')]:
    a, b = rows(original)[-1]['response'], rows(fresh)[0]['response']
    assert assets(a) == assets(b) and a['run']['map'] == b['run']['map']
    assert a['run']['game_state'] == b['run']['game_state']
passed('six fresh process outcomes including unconfirmed CombatResult')

cost = load(E / 'completion001/cost001/RESULT.json')
binary = R / '.cache/u164-review-dev-target/debug/u164_completion_cost'
assert sha(binary.read_bytes()) == cost['binary_sha256']
for receipt in cost['rows']:
    p = R / receipt['result']
    assert sha(p.read_bytes()) == receipt['result_sha256']
    result = load(p)
    assert result['passed'] and result['exact_requested_bytes_recovered']
    assert result['baseline']['live_requested_bytes'] == result['after']['live_requested_bytes']
assert len(cost['rows']) == 6
passed('six cost raw records, same binary and exact requested-heap recovery', rerun=False)
(A / 'EVIDENCE_CHECK.json').write_text(json.dumps({'passed': True, 'checks': checks}, indent=2, ensure_ascii=False) + '\n')
print(json.dumps({'passed': True, 'checks': len(checks)}))
