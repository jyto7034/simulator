"""Retain the predecessor's public performance and allocator numeric gates."""
import subprocess,sys
from run import ROOT,HERE
for allocation in [False,True]:
 name='public_guards_allocator001' if allocation else 'public_guards001'
 command=['env','CARGO_TARGET_DIR=/tmp/wp02-d07-b-r05-target','cargo','run','--jobs=1','--manifest-path','core_enhanced/tools/runtime_architecture_final_validation/Cargo.toml','--bin','public_guards','--release',*(['--features','allocation-observer'] if allocation else []),'--offline','--locked','--',str(HERE/'evidence'/(name+'_raw.json'))]
 result=subprocess.run([sys.executable,str(HERE/'tools/run.py'),name,'--',*command])
 assert result.returncode==0,(name,result.returncode)
