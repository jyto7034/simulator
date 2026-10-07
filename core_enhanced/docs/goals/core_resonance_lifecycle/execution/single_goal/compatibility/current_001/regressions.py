import json
from pathlib import Path
import subprocess
import sys
C = Path(__file__).resolve().parent
D = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_resonance_lifecycle')
R = D.parents[3]
E = C / 'evidence'
def run(name, command, allowed=(0,)):
    print('Starting compat_' + name, flush=True)
    result = subprocess.run([sys.executable, str(D / 'tools/run.py'), 'compat_' + name + '_002', '--', *map(str, command)])
    assert result.returncode in allowed, (name, result.returncode)
E.mkdir(exist_ok=True)
stage = sys.argv[1] if len(sys.argv) > 1 else 'all'
assert stage in ('all', 'functional', 'performance')
if stage != 'performance':
    run('prepared_public', ['python3', C / 'probe.py', '003'])
    run('prepared_boundary', ['python3', C / 'boundary.py', '003'])
    run('observed_release', ['cargo', 'build', '--jobs=1', '-p', 'game_core', '-p', 'game_server', '--release', '--bins', '--features', 'game_server/runtime-observation', '--offline', '--locked'])
    run('default_parity', ['python3', C / 'default_parity.py'])
    run('public_oracle', ['python3', C / 'public_probe.py', 'after'])
    run('public_trace', ['python3', C / 'public_trace_probe.py'])
    run('public_cli', ['python3', C / 'public_cli.py'])
    run('public_profile', ['python3', C / 'public_profile_probe.py'])
    run('public_boundary', ['python3', C / 'public_boundary_probe.py'])
    run('public_api', ['python3', Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_runtime_safety_observability/tools/public_api_cli_probe.py'), '--repository', R, '--output-dir', E / 'public_api'])
if stage == 'functional':
    sys.exit(0)
prepared = D / 'evidence/compat_pc_prepare_002.json'
if prepared.exists():
    sys.path.insert(0, str(D / 'tools'))
    from run import identity as product_identity
    prior = json.loads(prepared.read_text())
    assert prior['exit_code'] == 0 and prior['source_before'] == product_identity() == prior['source_after']
else:
    run('pc_prepare', ['python3', C / 'prepare_pc.py', '001'])
isolate = Path(json.loads((E / 'pc_isolate_001.json').read_text())['isolate'])
run('pc', ['env', 'CARGO_TARGET_DIR=/tmp/wp02-d07-b-pc-target', 'R06_BASELINE_OUTPUT=' + str(E / 'pc_raw_current_002.json'), 'R06_BASELINE_SAMPLES=11', 'cargo', 'test', '--manifest-path', isolate / 'Cargo.toml', '-p', 'game_core', '--lib', '--release', '--offline', '--locked', 'pc_load_baseline', '--', '--ignored', '--nocapture', '--test-threads=1'])
run('pc_compare', ['python3', R / 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/compare_pc_v32.py', E / 'pc_raw_current_002.json', E / 'PC_SUMMARY_CURRENT_002.json'])
run('pc_immediate', ['python3', C / 'compare_pc.py'])
run('r05', ['env', 'CARGO_TARGET_DIR=/tmp/wp02-d07-b-r05-target', 'cargo', 'run', '--manifest-path', 'core_enhanced/tools/runtime_architecture_final_validation/Cargo.toml', '--bin', 'final_performance', '--release', '--features', 'allocation-observer', '--offline', '--locked', '--', R / 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/evidence/P01_LOG_V32_BASELINE_001.json', E / 'r05_raw_current_002.json'], (0, 101))
run('r05_compare', ['python3', C / 'summarize_r05_current.py'])
