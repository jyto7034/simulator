"""Sequential current-source validation; stop at the first unexpected result."""
from pathlib import Path
import json
import subprocess
import sys

H = Path(__file__).resolve().parent
R = H.parents[7]
E = H / 'evidence'
OLD = H.parents[1]

def run(name, command, allowed=(0,)):
    result = subprocess.run([sys.executable, str(H / 'run.py'), name, '--', *map(str, command)])
    if result.returncode not in allowed:
        raise SystemExit(result.returncode)

assert json.loads((E / 'all_default_002.json').read_text())['exit_code'] == 0
run('all_feature_001', ['cargo', 'test', '-p', 'game_core', '--all-targets', '--features', 'test-fixtures', '--offline', '--locked'])
run('check_001', ['cargo', 'check', '-p', 'game_core', '--all-targets', '--offline', '--locked'])
run('event_log_check_001', ['cargo', 'check', '-p', 'game_core', '--all-targets', '--features', 'event_log_checks', '--offline', '--locked'])
run('clippy_001', ['cargo', 'clippy', '-p', 'game_core', '--all-targets', '--offline', '--locked', '--', '-A', 'clippy::never_loop', '-A', 'clippy::eq_op'])
run('fmt_001', ['cargo', 'fmt', '--all', '--', '--check'])
run('release_001', ['cargo', 'build', '-p', 'game_core', '--release', '--bins', '--offline', '--locked'])
run('public_oracle_after_001', ['python3', '-B', H / 'public_probe.py', 'after'])
run('public_cli_001', ['python3', '-B', H / 'public_cli.py'])
isolate = json.loads((E / 'pc_isolate_current_001.json').read_text())['isolate']
run('pc_current_001', ['env', 'CARGO_TARGET_DIR=/tmp/wp02-d07-b-pc-target', f'R06_BASELINE_OUTPUT={E / "pc_raw_current_001.json"}', 'R06_BASELINE_SAMPLES=11', 'cargo', 'test', '--manifest-path', Path(isolate) / 'Cargo.toml', '-p', 'game_core', '--lib', '--release', '--offline', '--locked', 'pc_load_baseline', '--', '--ignored', '--nocapture', '--test-threads=1'])
run('pc_compare_001', ['python3', '-B', OLD / 'compare_pc_v32.py', E / 'pc_raw_current_001.json', E / 'PC_SUMMARY_CURRENT_001.json'])
run('r05_current_001', ['env', 'CARGO_TARGET_DIR=/tmp/wp02-d07-b-r05-target', 'cargo', 'run', '--manifest-path', R / 'core_enhanced/tools/runtime_architecture_final_validation/Cargo.toml', '--bin', 'final_performance', '--release', '--features', 'allocation-observer', '--offline', '--locked', '--', OLD / 'evidence/P01_LOG_V32_BASELINE_001.json', E / 'r05_raw_current_001.json'], allowed=(0, 101))
run('r05_compare_001', ['python3', '-B', H / 'summarize_r05.py'])
