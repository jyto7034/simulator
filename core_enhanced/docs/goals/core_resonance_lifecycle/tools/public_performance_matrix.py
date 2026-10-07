"""Run timing-sensitive inherited guards after all other validation has stopped."""
import json, os, subprocess, sys
from initialize import ROOT, HERE
from run import identity

current = identity()
for name in ('integrated_heap002', 'integrated_host_observation002'):
    receipt = json.loads((HERE / 'evidence' / (name + '.json')).read_text())
    assert receipt['exit_code'] == 0 and receipt['source_before'] == current == receipt['source_after']
env = os.environ.copy()
env['PATH'] = '/home/blast/.cargo/bin:' + env['PATH']
env.setdefault('CARGO_HOME', '/tmp/wp02-cargo-home')
env['CARGO_BUILD_JOBS'] = '1'
for name, features in [('public_guards001', []), ('public_guards_allocator001', ['--features', 'allocation-observer'])]:
    print('Starting ' + name, flush=True)
    command = [sys.executable, HERE / 'tools/run.py', name, '--', 'env',
        'CARGO_TARGET_DIR=/tmp/wp02-d07-b-r05-target', 'cargo', 'run', '--jobs=1',
        '--manifest-path', 'core_enhanced/tools/runtime_architecture_final_validation/Cargo.toml',
        '--bin', 'public_guards', '--release', *features, '--offline', '--locked', '--',
        HERE / 'evidence' / (name + '_raw.json')]
    result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env)
    if result.returncode:
        sys.exit(result.returncode)
result = subprocess.run([sys.executable,
    str(HERE / 'execution/single_goal/compatibility/current_001/regressions.py'), 'performance'],
    cwd=ROOT, env=env)
sys.exit(result.returncode)
