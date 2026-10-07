import json
from pathlib import Path
import subprocess
import sys
C = Path(__file__).resolve().parent
D = C.parents[3]
R = D.parents[3]
E = C / 'evidence'
def run(name, command, allowed=(0,)):
    print('Starting s1f02_' + name, flush=True)
    result = subprocess.run([sys.executable, str(D / 'tools/run.py'), 's1f02_' + name + '_001', '--', *map(str, command)])
    assert result.returncode in allowed, (name, result.returncode)
result = subprocess.run([sys.executable, str(D / 'tools/run.py'), 's1f02_pc_immediate_002', '--', 'python3', str(C / 'compare_pc.py')])
assert result.returncode == 0
run('r05', ['env', 'CARGO_TARGET_DIR=/tmp/wp02-d07-b-r05-target', 'cargo', 'run', '--manifest-path', 'core_enhanced/tools/runtime_architecture_final_validation/Cargo.toml', '--bin', 'final_performance', '--release', '--features', 'allocation-observer', '--offline', '--locked', '--', R / 'core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/evidence/P01_LOG_V32_BASELINE_001.json', E / 'r05_raw_current_002.json'], (0, 101))
run('r05_compare', ['python3', C / 'summarize_r05.py'])
