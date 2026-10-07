"""Current public compatibility consumers, sequential with fresh output labels."""
import subprocess
import sys
from pathlib import Path
H = Path(__file__).resolve().parents[1]
C = H / 'execution/single_goal/compatibility/current_001'
R = H.parents[3]
start = int(sys.argv[1]) if len(sys.argv) > 1 else 0
label = sys.argv[2] if len(sys.argv) > 2 else '002'
starter_label = sys.argv[3] if len(sys.argv) > 3 else '003'
trace_label = sys.argv[4] if len(sys.argv) > 4 else label
assert label.isdigit() and starter_label.isdigit() and trace_label.isdigit()
oracle_label = 'after_' + label


def run(name, args):
    print('Starting ' + name, flush=True)
    result = subprocess.run([sys.executable, str(H / 'tools/run.py'), name, '--', 'env', 'CARGO_BUILD_JOBS=1', *map(str, args)])
    if result.returncode:
        sys.exit(result.returncode)


from release_gate import require
require('release')

commands = [
    (f'compat_prepared_public_{label}', ['python3', C / 'probe.py', label]),
    (f'compat_prepared_boundary_{label}', ['python3', C / 'boundary.py', label]),
    (f'compat_input_validation_{label}', ['python3', H / 'tools/validate_public_inputs.py', label]),
    (f'compat_default_parity_{label}', ['python3', C / 'default_parity.py', label]),
    (f'compat_public_oracle_{label}', ['python3', C / 'public_probe.py', oracle_label]),
    (f'compat_public_trace_{trace_label}', ['python3', C / 'public_trace_probe.py', oracle_label]),
    (f'public_relics_{label}', ['python3', H / 'tools/public_relics.py', label]),
    (f'compat_public_cli_{label}', ['python3', C / 'public_cli.py', label]),
    (f'compat_public_profile_{label}', ['python3', C / 'public_profile_probe.py', label]),
    (f'compat_public_boundary_{label}', ['python3', C / 'public_boundary_probe.py', label]),
    (f'compat_public_api_{label}', ['python3', R / 'core_enhanced/docs/goals/core_runtime_safety_observability/tools/public_api_cli_probe.py', '--repository', R, '--output-dir', C / f'evidence/public_api_{label}']),
    (f'public_starter_schema_{starter_label}', ['python3', H / 'tools/public_starter_schema.py', starter_label]),
    (f'public_schema_{label}', ['python3', H / 'tools/public_schema_probe.py', label]),
    (f'public_independent_area_{label}', ['python3', H / 'tools/public_independent_area_probe.py', label]),
    (f'public_continuation_{label}', ['python3', H / 'tools/public_continuation_probe.py', label]),
    (f'public_rewind_{label}', ['python3', H / 'tools/public_rewind_probe.py', label]),
    (f'public_object_schema_{label}', ['python3', H / 'tools/public_object_schema.py', label]),
]
for name, args in commands[start:]:
    run(name, args)
