"""Run PC36 and R05 sequentially after product validation/builds finish."""
import json
import os
import subprocess
import sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = Path('/mnt/f/work/simulator')
E = HERE / 'evidence'
CARGO = '/home/blast/.cargo/bin/cargo'
for name in ['all_targets_final_001', 'clippy_final_001', 'format_final_001', 'pc_build_final_001', 'r05_build_final_001']:
    assert json.loads((E / (name + '.json')).read_text())['exit_code'] == 0
isolate = json.loads((E / 'pc_isolate_final_001.json').read_text())['isolate']
env = dict(os.environ, CARGO_TARGET_DIR='/tmp/wp02-d07-b-pc-target')
subprocess.run([sys.executable, str(HERE / 'run.py'), 'pc_final_001', '--cwd', isolate, '--',
                '/usr/bin/env', 'R06_BASELINE_OUTPUT=' + str(E / 'pc_raw_final_001.json'), 'R06_BASELINE_SAMPLES=11',
                CARGO, 'test', '-p', 'game_core', '--lib', '--release', '--offline', '--locked', 'pc_load_baseline', '--', '--ignored', '--nocapture', '--test-threads=1'], env=env, check=True)
subprocess.run([sys.executable, str(HERE / 'compare_pc.py'), str(E / 'pc_raw_final_001.json'), str(E / 'PC36_COMPARISON.json')], check=True)
env = dict(os.environ, CARGO_TARGET_DIR='/tmp/wp02-d07-b-r05-target')
result = subprocess.run([sys.executable, str(HERE / 'run.py'), 'r05_final_001', '--cwd', str(ROOT / 'core_enhanced'), '--',
                CARGO, 'run', '--manifest-path', 'tools/runtime_architecture_final_validation/Cargo.toml', '--bin', 'final_performance', '--release', '--features', 'allocation-observer', '--offline', '--locked', '--',
                'evidence/P01_PERFORMANCE_BASELINE.json', str(E / 'r05_raw_final_001.json')], env=env)
assert result.returncode in (0, 101), result.returncode
subprocess.run([sys.executable, str(HERE / 'summarize_r05.py')], check=True)
