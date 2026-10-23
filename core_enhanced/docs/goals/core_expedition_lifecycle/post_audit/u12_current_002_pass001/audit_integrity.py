"""Read-only U12 audit integrity; writes new audit evidence only."""
import argparse
import ast
import datetime
import hashlib
import json
import os
from pathlib import Path
import tarfile

A = Path(__file__).resolve().parent
ROOT = next(p for p in A.parents if (p / 'core_enhanced/src/lib.rs').is_file())
G = A.parent.parent
E = G / 'execution/single_goal'
T = E / 'target/u12_current_002'

def read(p):
    return json.loads(p.read_text())

def sha(p):
    h = hashlib.sha256()
    with p.open('rb') as f:
        for b in iter(lambda: f.read(1024 * 1024), b''):
            h.update(b)
    return h.hexdigest()

def ref(p):
    return {'path': str(p.relative_to(ROOT)), 'bytes': p.stat().st_size, 'sha256': sha(p)}

def write(name, value):
    with (A / name).open('x') as f:
        json.dump(value, f, ensure_ascii=False, indent=2)
        f.write('\n')

def check_rows(rows, base, cache):
    bad = []
    for r in rows:
        p = base / r['path']
        try:
            if p not in cache:
                if p.is_symlink() or not p.is_file():
                    raise ValueError('not a regular non-symlink file')
                cache[p] = ref(p)
            actual = cache[p]
            if any(actual[k] != r[k] for k in ('bytes', 'sha256')):
                bad.append({'expected': r, 'actual': actual})
        except Exception as ex:
            bad.append({'expected': r, 'error': str(ex)})
    return {'checked': len(rows), 'mismatches': bad}

def declared_membership(m):
    # Reconstruct the freezer's declared scope without invoking a mutating main.
    scoped = read(E / 'baseline/SCOPED_PREIMAGES_001.json')
    baseline = read(E / 'baseline/BASELINE_001.json')
    prior = read(ROOT / baseline['inherited_u11']['source_manifest']['path'])
    paths = {r['path'] for r in prior['files']}
    def add_tree(base, skips, only_python=False):
        for current, dirs, files in os.walk(base, followlinks=False):
            dirs[:] = sorted(d for d in dirs if d not in skips)
            for name in files:
                p = Path(current) / name
                if p.suffix in {'.pyc', '.pyo'} or only_python and p.suffix != '.py':
                    continue
                paths.add(str(p.relative_to(ROOT)))
    for name in scoped['roots']:
        add_tree(ROOT / name, set(scoped['skip_dirs']))
    paths.update(scoped['explicit'] + scoped['planned_new_paths'])
    for name in ['test_parts/runtime001', 'test_parts/vault_cost001/inputs',
                 'test_parts/vault_cost001/probe', 'finalization001']:
        add_tree(E / name, {'target', '__pycache__', 'evidence'}, name == 'finalization001')
    paths.update(str((E / 'test_parts/vault_cost001' / n).relative_to(ROOT)) for n in ['collect.py', 'prepare_inputs.py'])
    paths.update(str((E / 'product_parts' / n).relative_to(ROOT)) for n in ['COLLECT_COMMAND_001.py', 'COLLECT_COMMAND_002.py', 'COLLECT_COMMAND_003.py'])
    actual = {p for p in paths if (ROOT / p).is_file()}
    expected = {r['path'] for r in m['source_files']}
    return {'expected_count': len(expected), 'actual_count': len(actual),
            'added': sorted(actual - expected), 'missing': sorted(expected - actual)}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('stage', choices=['entry', 'exit', 'archives'])
    args = parser.parse_args()
    m = read(T / 'MANIFEST.json')
    result = {'stage': args.stage, 'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'target_manifest': ref(T / 'MANIFEST.json')}
    if args.stage == 'archives':
        result['checks'] = []
        for name, rows in [('SOURCE_BEFORE.tar.gz', m['before_files']), ('SOURCE_AFTER.tar.gz', m['source_files']),
                           ('RECORDS.tar.gz', read(T / 'RECORDS.json')), ('REFERENCES.tar.gz', read(T / 'REFERENCES.json'))]:
            expected = {r['path']: r for r in rows}
            seen, bad = set(), []
            with tarfile.open(T / name) as archive:
                for member in archive:
                    if not member.isfile() or member.name not in expected or member.name in seen:
                        bad.append({'member': member.name, 'reason': 'type/membership/duplicate'})
                        continue
                    seen.add(member.name)
                    r = expected[member.name]
                    h = hashlib.sha256()
                    with archive.extractfile(member) as f:
                        for b in iter(lambda: f.read(1024 * 1024), b''):
                            h.update(b)
                    if member.size != r['bytes'] or h.hexdigest() != r['sha256']:
                        bad.append({'member': member.name, 'reason': 'bytes/hash'})
            bad.extend({'member': p, 'reason': 'missing'} for p in sorted(expected.keys() - seen))
            result['checks'].append({'archive': name, 'members': len(seen), 'mismatches': bad})
            print(name, len(seen), 'mismatches', len(bad), flush=True)
    else:
        cache = {}
        result['checks'] = []
        groups = [('source', m['source_files'], ROOT), ('records', read(T / 'RECORDS.json'), ROOT),
                  ('references', read(T / 'REFERENCES.json'), ROOT), ('external', read(T / 'EXTERNAL_ARTIFACTS.json'), ROOT),
                  ('target_artifacts', m['archives_and_patches'], T)]
        for name, rows, base in groups:
            checked = check_rows(rows, base, cache)
            result['checks'].append({'group': name, **checked})
            print(name, checked['checked'], 'mismatches', len(checked['mismatches']), flush=True)
        result['membership'] = declared_membership(m)
        result['deleted_still_absent'] = {p: not (ROOT / p).exists() for p in m['deleted']}
        result['unique_files_hashed'] = len(cache)
        result['unrelated_design_note'] = ref(ROOT / 'core_enhanced/docs/skills/run_relic_content_design_notes.ko.md')
    result['finished_at_utc'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    result['pass'] = all(not c['mismatches'] for c in result['checks'])
    if 'membership' in result:
        result['pass'] &= not result['membership']['added'] and not result['membership']['missing'] and all(result['deleted_still_absent'].values())
    write('INTEGRITY_' + args.stage.upper() + '.json', result)
    print('PASS' if result['pass'] else 'FAIL', flush=True)
    raise SystemExit(0 if result['pass'] else 1)

if __name__ == '__main__':
    main()
