#!/usr/bin/env python3
"""Independent read-only frozen target and actual predecessor comparison."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
from datetime import datetime, timezone

ROOT = Path('/mnt/f/work/simulator')
BASE = ROOT / 'core_enhanced/docs/goals/content_authoring_pack_generation/execution/single_goal'
OUT = BASE / 'audit-evidence/fresh_002'
TARGET = BASE / 'AUDIT_TARGET_003.json'
TARGET_HASH = '01bdf3ee5f8a85ad5cf6713ecba032783ab56019b922d1eeca318ae6c9d04345'
R01 = ROOT / 'core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal'

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def check_file(path, expected, problems):
    actual = sha(path) if path.is_file() else None
    if expected is None:
        if path.exists() or path.is_symlink(): problems.append({'path': str(path), 'expected': 'absent'})
    elif actual != expected:
        problems.append({'path': str(path), 'expected': expected, 'actual': actual})
    return actual

def main():
    stage = sys.argv[1]
    assert stage in ('begin', 'end')
    target = json.loads(TARGET.read_text())
    problems = []
    check_file(TARGET, TARGET_HASH, problems)
    for row in target['manifest']:
        check_file(ROOT / row['path'], row['sha256'], problems)
    inventories = []
    for fixture in target['external_actual']:
        directory = Path(fixture['directory'])
        expected = {r['path'] for r in fixture['files']}
        actual = {str(p.relative_to(directory)) for p in directory.rglob('*') if p.is_file() or p.is_symlink()}
        if actual != expected:
            problems.append({'directory': str(directory), 'missing': sorted(expected-actual), 'extra': sorted(actual-expected)})
        for row in fixture['files']:
            check_file(directory / row['path'], row['sha256'], problems)
        inventories.append({'directory': str(directory), 'files': len(expected), 'exact_inventory': actual == expected})
    for row in target['default_release_binaries']:
        check_file(ROOT / row['path'], row['sha256'], problems)
    pred_outputs = R01 / 'audit-evidence/successor_003/AUDIT_OUTPUTS.json'
    pred_audit = R01 / 'COMPLETION_AUDIT_003.md'
    check_file(pred_outputs, '32b498e4203c46487239f7696dd433a6d309d66945207a507891af6262f93add', problems)
    check_file(pred_audit, '726eddf9ac94ebb64b355a31a09bf6716b9f7fdd49dffa15b70fcd8dfda690d9', problems)
    pred = json.loads(pred_outputs.read_text())
    for row in pred['files']:
        check_file(ROOT / row['path'], row['sha256'], problems)
    pred_target = R01 / 'AUDIT_TARGET_003.json'
    check_file(pred_target, '7800068bb87b0e3dd87d9ec99920600867714d35eef9dabb4f26a0bdf137e0e6', problems)
    # R02 contains the predecessor implementation surface. Also check the predecessor independently.
    pred_target_data = json.loads(pred_target.read_text())
    for row in pred_target_data['manifest']:
        check_file(ROOT / row['path'], row['sha256'], problems)
    def run(args): return subprocess.check_output(args, cwd=ROOT, text=True).strip()
    actual_env = {key: os.environ.get(key) for key in target['toolchain']['selected_environment']}
    checks = {'head':run(['git','rev-parse','HEAD']), 'branch':run(['git','branch','--show-current']),
              'rustc':run(['rustc','--version']), 'cargo':run(['cargo','--version']), 'selected_environment':actual_env}
    for key in ('head','branch'):
        if checks[key] != target[key]: problems.append({'environment': key, 'actual': checks[key], 'expected':target[key]})
    for key in ('rustc','cargo','selected_environment'):
        if checks[key] != target['toolchain'][key]: problems.append({'environment': key, 'actual':checks[key], 'expected':target['toolchain'][key]})
    if pred['verdict'] != 'Complete / High': problems.append({'predecessor_verdict':pred['verdict']})
    result = {'reviewer':'/root/r02_correction_fresh_audit', 'stage':stage, 'captured_at':datetime.now(timezone.utc).isoformat(),
        'target_record':target['record_id'], 'target_sha256':sha(TARGET), 'direct_members':len(target['manifest']),
        'hashed_members':sum(r['sha256'] is not None for r in target['manifest']),
        'required_absences':sum(r['sha256'] is None for r in target['manifest']),
        'external_files':sum(len(f['files']) for f in target['external_actual']), 'external_inventories':inventories,
        'default_release_binaries':target['default_release_binaries'], 'environment':checks,
        'predecessor':{'target_sha256':sha(pred_target), 'direct_members':len(pred_target_data['manifest']),
          'actual_audit_sha256':sha(pred_audit), 'actual_output_registry_sha256':sha(pred_outputs),
          'evidence_members':len(pred['files']), 'verdict':pred['verdict']},
        'problems':problems, 'currentness':'current / unchanged' if not problems else 'stale'}
    path = OUT / ('currentness_' + stage + '.json')
    path.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))
    return bool(problems)

if __name__ == '__main__': sys.exit(main())
