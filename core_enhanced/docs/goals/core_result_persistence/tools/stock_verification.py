"""Current release stock consumers after the V10 compaction correction."""
from pathlib import Path
import subprocess,sys
H=Path(__file__).resolve().parents[1]
def run(name,command):
 result=subprocess.run([sys.executable,str(H/'tools/run.py'),name,'--',*map(str,command)])
 if result.returncode:sys.exit(result.returncode)
run('integrated_release002',['cargo','build','-p','game_core','-p','game_server','--release','--bins','--features','game_server/runtime-observation','--offline','--locked'])
run('public_continuation003',['python3',H/'tools/public_continuation_probe.py','003'])
result=subprocess.run([sys.executable,str(H/'tools/final_fault_probes.py'),'002'])
if result.returncode:sys.exit(result.returncode)
run('public_guards_allocator001',['env','CARGO_TARGET_DIR=/tmp/wp02-d07-b-r05-target','cargo','run','--manifest-path','core_enhanced/tools/runtime_architecture_final_validation/Cargo.toml','--bin','public_guards','--release','--features','allocation-observer','--offline','--locked','--',H/'evidence/public_guards_allocator001_raw.json'])
