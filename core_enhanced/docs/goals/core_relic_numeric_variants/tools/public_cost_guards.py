from release_gate import require
require('release')
"""Retain the predecessor's public performance and allocator numeric gates."""
import subprocess,sys
from run import ROOT,HERE
# Functional suites may share compilation with release consumers; numeric guards
# wait for the integrated scheduler to exit, so our tests cannot perturb timing.
import time
from pathlib import Path
def integrated_running():
 for process in Path('/proc').iterdir():
  if not process.name.isdigit():continue
  try:
   arguments=(process/'cmdline').read_bytes().split(b'\0')
   if len(arguments)>1 and any(a.endswith(suffix) for a in arguments for suffix in (b'/core_relic_numeric_variants/tools/integrated_matrix.py', b'/core_relic_numeric_variants/tools/public_matrix.py', b'/core_relic_numeric_variants/tools/public_object_schema.py')):return True
  except (FileNotFoundError,PermissionError,ProcessLookupError):pass
 return False
while integrated_running():
 print('Waiting for current integrated functional suites before public numeric guards',flush=True)
 time.sleep(5)
label=sys.argv[1] if len(sys.argv)>1 else '002';assert label.isdigit()
for allocation in [False,True]:
 name=('public_guards_allocator' if allocation else 'public_guards')+label
 command=['env','CARGO_TARGET_DIR=/tmp/wp02-d07-b-r05-target','cargo','run','--jobs=1','--manifest-path','core_enhanced/tools/runtime_architecture_final_validation/Cargo.toml','--bin','public_guards','--release',*(['--features','allocation-observer'] if allocation else []),'--offline','--locked','--',str(HERE/'evidence'/(name+'_raw.json'))]
 result=subprocess.run([sys.executable,str(HERE/'tools/run.py'),name,'--',*command])
 assert result.returncode==0,(name,result.returncode)
