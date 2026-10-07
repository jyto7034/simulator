"""Compile isolated performance consumers; never execute their measurements."""
import json, subprocess, sys
from initialize import ROOT, HERE
from run import identity

mode = sys.argv[1]
assert mode in ('pc', 'guards')
def run(name, command):
    result = subprocess.run([sys.executable, str(HERE / 'tools/run.py'), name, '--',
                             *map(str, command)], cwd=ROOT)
    if result.returncode:
        sys.exit(result.returncode)
if mode == 'pc':
    prepared = json.loads((HERE / 'evidence/compat_pc_prepare_002.json').read_text())
    assert prepared['exit_code'] == 0 and prepared['source_before'] == identity() == prepared['source_after']
    isolation = json.loads((HERE / 'execution/single_goal/compatibility/current_001/evidence/pc_isolate_001.json').read_text())
    run('compat_pc_prebuild001', ['env', 'CARGO_TARGET_DIR=/tmp/wp02-d07-b-pc-target',
        'cargo', 'test', '--jobs=1', '--manifest-path', isolation['isolate'] + '/Cargo.toml',
        '-p', 'game_core', '--lib', '--release', '--offline', '--locked', '--no-run'])
else:
    command = ['env', 'CARGO_TARGET_DIR=/tmp/wp02-d07-b-r05-target', 'cargo', 'build', '--jobs=1',
        '--manifest-path', 'core_enhanced/tools/runtime_architecture_final_validation/Cargo.toml',
        '--release', '--offline', '--locked']
    run('public_guards_prebuild001', command + ['--bin', 'public_guards'])
    run('public_allocator_prebuild001', command + ['--features', 'allocation-observer',
        '--bin', 'public_guards', '--bin', 'final_performance'])
