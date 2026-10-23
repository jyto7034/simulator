"""Audit existing policy/evidence bindings; no product execution or target mutation."""
import datetime
import hashlib
import json
from pathlib import Path
import re

A = Path(__file__).resolve().parent
ROOT = next(p for p in A.parents if (p / 'core_enhanced/src/lib.rs').is_file())
G = A.parent.parent
E = G / 'execution/single_goal'
T = E / 'target/u12_current_002'
def read(p): return json.loads(p.read_text())
def write(n, d):
    with (A / n).open('x') as f:
        json.dump(d, f, ensure_ascii=False, indent=2)
        f.write('\n')
def canonical(d): return hashlib.sha256(json.dumps(d, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()).hexdigest()

assert read(A / 'INTEGRITY_ENTRY.json')['pass']
m = read(T / 'MANIFEST.json')
c = read(E / 'COVERAGE.json')
s = read(E / 'finalization001/EVIDENCE_SELECTION.json')
known = {r['path']: r for r in m['source_files'] + read(T / 'RECORDS.json') + read(T / 'REFERENCES.json') + read(T / 'EXTERNAL_ARTIFACTS.json')}
refs, errors = {}, []
def scan(v):
    if isinstance(v, dict):
        if {'path', 'sha256', 'bytes'} <= v.keys():
            refs[(v['path'], v['sha256'], v['bytes'])] = v
        for child in v.values(): scan(child)
    elif isinstance(v, list):
        for child in v: scan(child)
for value in [c, s, read(E / 'finalization001/PREDECESSOR_IMPACT.json')]: scan(value)
for (p, h, n), row in refs.items():
    prior = known.get(p)
    if prior is None:
        errors.append({'ref': row, 'reason': 'reference absent from verified entry membership'})
    elif (prior['sha256'], prior['bytes']) != (h, n):
        errors.append({'ref': row, 'verified_entry': prior})

freeze = read(G / 'P2G_30_FREEZE_001.json')['artifacts'][0]
policy_rows = []
for element in freeze['payload']['Observable Policy Acceptance Set']:
    ident = element['Policy Element Revision Ref']['Artifact ID'].removeprefix('PER-CEXL-').removesuffix('-REVIEWED-001')
    for index, literal in enumerate(element['Observable Policy Acceptance'], 1):
        policy_rows.append({'id': 'A-' + ident + '-' + str(index), 'exact_acceptance': literal})
actual_rows = [{'id': row['id'], 'exact_acceptance': row['exact_acceptance']} for row in c['acceptances']]
policy = {'freeze_ref': freeze['ref'], 'computed_payload_digest': 'sha256:' + canonical(freeze['payload']),
          'exact_38_rows_match': policy_rows == actual_rows,
          'policy_elements': len(freeze['payload']['Observable Policy Acceptance Set']),
          'acceptances': len(policy_rows), 'integration_seams': len(c['integration_seams']),
          'required_dimensions': len(c['required_dimensions']), 'nodes': len(c['execution_nodes'])}

runs = {v['run']: v for v in s['evidence'].values()}
# These recorded runs are cited in EXPERIMENTS but omitted from the generated
# selection index; retain exact individual passing cases and their old failures.
for name in ['functional012', 'functional013', 'data_functional001', 'functional_nonimpact001']:
    if (E / 'product_parts' / name / 'RESULT_001.json').exists():
        runs[name] = {'audit_added_locator': True}
raw_rows = []
for name, selected in runs.items():
    directory = E / 'product_parts' / name
    receipt = read(directory / 'RESULT_001.json')
    raw = (directory / 'stdout.log').read_text()
    starts = list(re.finditer(r'^test ([A-Za-z0-9_:]+) \.\.\. ', raw, re.M))
    tests = []
    for i, start in enumerate(starts):
        end = starts[i+1].start() if i+1 < len(starts) else len(raw)
        chunk = raw[start.end():end].split('\nfailures:', 1)[0].split('\ntest result:', 1)[0].strip()
        status = chunk if chunk in {'ok', 'FAILED', 'ignored'} else 'ok' if chunk.endswith('\nok') else 'ambiguous'
        tests.append({'selector': start[1], 'status': status})
    summaries = re.findall(r'^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;', raw, re.M)
    summary = dict(zip(['outcome','passed','failed','ignored','measured','filtered'], [summaries[-1][0]] + list(map(int, summaries[-1][1:])))) if summaries else None
    passed = [t['selector'] for t in tests if t['status'] == 'ok']
    failed = [t['selector'] for t in tests if t['status'] == 'FAILED']
    row_errors = []
    if summary:
        for status, key in [('ok', 'passed'), ('FAILED', 'failed'), ('ignored', 'ignored')]:
            if sum(t['status'] == status for t in tests) != summary[key]: row_errors.append('raw named count differs: ' + key)
        if any(t['status'] == 'ambiguous' for t in tests): row_errors.append('ambiguous named result')
        if '--exact' not in receipt['argv']: row_errors.append('missing exact filter')
        if any(t['selector'] not in receipt['argv'] for t in tests): row_errors.append('executed selector absent from argv')
        if selected.get('passing_selectors') is not None and passed != selected['passing_selectors']: row_errors.append('selection differs from raw passing markers')
    if not receipt['child_reaped'] or not receipt['source_unchanged']: row_errors.append('execution not reaped or inputs changed')
    before = read(directory / 'SOURCE_BEFORE.json')
    after = read(directory / 'SOURCE_AFTER.json')
    if before != after: row_errors.append('raw before/after inventory differs')
    deltas = [{'path': r['path'], 'old_sha256': r['sha256'], 'final_sha256': known.get(r['path'], {}).get('sha256')}
              for r in before if r['sha256'] != known.get(r['path'], {}).get('sha256')]
    raw_rows.append({'run': name, 'receipt': known[str((directory / 'RESULT_001.json').relative_to(ROOT))],
                     'stdout': known[str((directory / 'stdout.log').relative_to(ROOT))],
                     'exit_code': receipt['exit_code'], 'duration_seconds': receipt['duration_seconds'],
                     'summary': summary, 'tests': tests, 'passing_selectors': passed, 'failed_selectors': failed,
                     'source_delta': deltas, 'audit_added_locator': selected.get('audit_added_locator', False), 'errors': row_errors})
result = {'timestamp': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'policy': policy,
          'references': {'count': len(refs), 'hash_reuse': 'all verified at audit entry', 'errors': errors},
          'raw_runs': raw_rows, 'interpretation': 'Expected refusals, management operations, gameplay probes, measurement and Rust test cases remain separate. Failed batches contribute only named valid PASS cases; no totals across overlapping selections.'}
result['machine_pass'] = not errors and policy['exact_38_rows_match'] and policy['computed_payload_digest'] == freeze['ref']['Content Digest'] and all(not r['errors'] for r in raw_rows)
write('MACHINE_EVIDENCE.json', result)
print(json.dumps({'policy': policy, 'references': len(refs), 'runs': len(raw_rows), 'errors': errors,
                  'raw_errors': [{'run': r['run'], 'errors': r['errors']} for r in raw_rows if r['errors']], 'machine_pass': result['machine_pass']}, ensure_ascii=False))
