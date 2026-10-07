"""Current public compatibility consumers, sequential and immutable receipts."""
import subprocess,sys
from pathlib import Path
H=Path(__file__).resolve().parents[1]
C=H/'execution/single_goal/compatibility/current_001'
R=H.parents[3]
def run(name,args):
 print('Starting '+name,flush=True)
 result=subprocess.run([sys.executable,str(H/'tools/run.py'),name,'--','env','CARGO_BUILD_JOBS=1',*map(str,args)])
 if result.returncode:sys.exit(result.returncode)
commands=[
 ('compat_prepared_public_001',['python3',C/'probe.py','001']),
 ('compat_prepared_boundary_001',['python3',C/'boundary.py','001']),
 ('compat_input_validation_001',['python3',H/'tools/validate_public_inputs.py','001']),
 ('compat_default_parity_001',['python3',C/'default_parity.py']),
 ('compat_public_oracle_001',['python3',C/'public_probe.py','after']),
 ('compat_public_trace_001',['python3',C/'public_trace_probe.py']),
 ('compat_public_cli_001',['python3',C/'public_cli.py']),
 ('compat_public_profile_001',['python3',C/'public_profile_probe.py']),
 ('compat_public_boundary_001',['python3',C/'public_boundary_probe.py']),
 ('compat_public_api_001',['python3',R/'core_enhanced/docs/goals/core_runtime_safety_observability/tools/public_api_cli_probe.py','--repository',R,'--output-dir',C/'evidence/public_api']),
 ('public_starter_schema_002',['python3',H/'tools/public_starter_schema.py','002']),
 ('public_schema_001',['python3',H/'tools/public_schema_probe.py','001']),
 ('public_independent_area_001',['python3',H/'tools/public_independent_area_probe.py','001']),
 ('public_continuation_001',['python3',H/'tools/public_continuation_probe.py','001']),
 ('public_rewind_001',['python3',H/'tools/public_rewind_probe.py','001']),
]
start=int(sys.argv[1]) if len(sys.argv)>1 else 0
for name,args in commands[start:]:run(name,args)
