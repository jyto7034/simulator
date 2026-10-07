"""Rebind functional public consumers after the independent-area correction."""
import json, os, subprocess, sys
from initialize import ROOT,HERE
from run import identity

release=json.loads((HERE/'evidence/integrated_release002.json').read_text())
assert release['exit_code']==0 and release['source_after']==identity()
env=os.environ.copy();env['CARGO_BUILD_JOBS']='1'
env['PATH']='/home/blast/.cargo/bin:'+env['PATH']
env.setdefault('CARGO_HOME','/tmp/wp02-cargo-home')
env.setdefault('CARGO_TARGET_DIR',str(ROOT/'.cache/d07-runtime-amount-product-target'))
def execute(command):
    result=subprocess.run(list(map(str,command)),cwd=ROOT,env=env)
    if result.returncode:sys.exit(result.returncode)
def run(name,script,label):
    print('Starting '+name,flush=True)
    execute([sys.executable,HERE/'tools/run.py',name,'--',sys.executable,HERE/'tools'/script,label])
run('public_inputs003','validate_public_inputs.py','003')
run('public_schema004','public_schema_probe.py','004')
run('public_independent_area003','public_independent_area_probe.py','003')
run('public_continuation002','public_continuation_probe.py','002')
run('public_rewind002','public_rewind_probe.py','002')
execute([sys.executable,HERE/'execution/single_goal/compatibility/current_001/regressions.py','functional'])
execute([sys.executable,HERE/'tools/final_fault_probes.py','001'])
print('PASS functional public consumers on current source',flush=True)
